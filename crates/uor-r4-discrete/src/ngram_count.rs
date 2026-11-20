//! Exact n-gram counts over u16 token streams (claude-412): the shared primitive
//! of the sealed reference R* and P1's count base.
//!
//! This file holds the host side only: the ordered key [`PrimeRouteKey`], the
//! canonical record formats, the per-order SHA-256 (the D85 replay identity),
//! the key-range partition plan and the assembly of partition outputs. The
//! counting itself runs on CUDA (`ngram_count.cu`, `ngram_count_device.rs`);
//! the CPU counter in `ngram_count_oracle.rs` is a test oracle only.
//!
//! Order k (0..=10) counts each target j, k <= j < n, under the context
//! t[j-k..j]. The canonical triple stream of an order is every distinct
//! (key, next) in ascending (key word, next) order with its count.

use crate::{Error, Result};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

include!("kn_ceiling/runtime/key.rs");
include!("kn_ceiling/runtime/count_words.rs");

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// Elements below which [`build_indexed`] fills its vector on the caller.
const BUILD_PARALLEL_MIN: usize = 1 << 16;

/// The vector whose element i is `f(i)` for i in 0..n (claude-431). It is
/// filled in at most `threads` contiguous chunks on scoped threads, or on the
/// caller below 2^16 elements or with one thread; the result does not depend
/// on `threads`.
pub fn build_indexed<T, F>(n: usize, threads: usize, f: F) -> Vec<T>
where
    T: Send,
    F: Fn(usize) -> T + Sync,
{
    if threads <= 1 || n < BUILD_PARALLEL_MIN {
        return (0..n).map(f).collect();
    }
    let mut out = Vec::with_capacity(n);
    let chunk = n.div_ceil(threads);
    std::thread::scope(|scope| {
        let f = &f;
        for (index, slots) in out.spare_capacity_mut()[..n].chunks_mut(chunk).enumerate() {
            scope.spawn(move || {
                let base = index * chunk;
                for (offset, slot) in slots.iter_mut().enumerate() {
                    slot.write(f(base + offset));
                }
            });
        }
    });
    // SAFETY: the chunks cover 0..n and each thread wrote every slot of its
    // chunk; a panic in any thread leaves the scope before this line.
    unsafe { out.set_len(n) };
    out
}

/// What one order's count produced: the replay identity and the census.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NgramOrderDigest {
    pub order: u32,
    pub targets: u64,
    pub grams: u64,
    pub contexts: u64,
    /// n1..n4: grams whose count is exactly 1, 2, 3, 4.
    pub count_of_counts: [u64; 4],
    pub triple_sha256: String,
    pub context_sha256: String,
}

/// The output of one key-range partition, as the device emits it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NgramPartition {
    pub triples: Vec<NgramTriple>,
    pub contexts: Vec<NgramContext>,
    pub count_of_counts: [u64; 4],
}

/// The bytes a worker encodes before it hashes them.
const HASH_CHUNK_BYTES: usize = 4 << 20;
/// Partitions queued per worker behind the one it is hashing (claude-428).
const WORKER_QUEUE_DEPTH: usize = 1;

/// One SHA-256 stream over a domain, the order byte and encoded records,
/// hashed whenever `chunk` bytes are buffered. SHA-256 of a byte stream does
/// not depend on where the stream is cut, so the digest equals hashing the
/// same bytes in one call.
struct ChunkedHash {
    hash: Sha256,
    buffer: Vec<u8>,
    chunk: usize,
}

impl ChunkedHash {
    fn new(domain: &[u8], order: u32, chunk: usize) -> Self {
        let mut hash = Sha256::new();
        hash.update(domain);
        hash.update([order as u8]);
        let chunk = chunk.max(1);
        Self {
            hash,
            buffer: Vec::with_capacity(chunk),
            chunk,
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
        if self.buffer.len() >= self.chunk {
            self.hash.update(&self.buffer);
            self.buffer.clear();
        }
    }

    fn finish(mut self) -> String {
        self.hash.update(&self.buffer);
        format!("{:x}", self.hash.finalize())
    }
}

/// What a worker does with each partition, in arrival order, and at the end.
trait StreamWork: Send + 'static {
    type Output: Send + 'static;
    fn feed(&mut self, partition: &NgramPartition);
    fn finish(self) -> Self::Output;
}

/// The triple stream: every triple, encoded and hashed.
struct TripleWork {
    out: ChunkedHash,
}

impl StreamWork for TripleWork {
    type Output = String;

    fn feed(&mut self, partition: &NgramPartition) {
        for triple in &partition.triples {
            self.out.update(&triple_bytes(triple));
        }
    }

    fn finish(self) -> String {
        self.out.finish()
    }
}

/// The context stream. A context may straddle a partition boundary only at
/// order 0 (the empty context); its records are added, since every field is
/// additive over disjoint follower sets. The caller has already refused any
/// other context that repeats or goes backwards across a boundary.
struct ContextWork {
    out: ChunkedHash,
    pending: Option<NgramContext>,
    emitted: u64,
    keep: bool,
    kept: Vec<NgramContext>,
}

impl ContextWork {
    fn emit(&mut self, context: NgramContext) {
        self.out.update(&context_bytes(&context));
        self.emitted += 1;
        if self.keep {
            self.kept.push(context);
        }
    }
}

impl StreamWork for ContextWork {
    /// The digest, the contexts emitted, and the kept records (tests).
    type Output = (String, u64, Vec<NgramContext>);

    fn feed(&mut self, partition: &NgramPartition) {
        for context in &partition.contexts {
            match self.pending.take() {
                Some(mut pending) if pending.key == context.key => {
                    pending.total += context.total;
                    pending.followers += context.followers;
                    pending.n1 += context.n1;
                    pending.n2 += context.n2;
                    pending.n3_plus += context.n3_plus;
                    self.pending = Some(pending);
                }
                Some(pending) => {
                    self.emit(pending);
                    self.pending = Some(*context);
                }
                None => self.pending = Some(*context),
            }
        }
    }

    fn finish(mut self) -> Self::Output {
        if let Some(pending) = self.pending.take() {
            self.emit(pending);
        }
        (self.out.finish(), self.emitted, self.kept)
    }
}

/// claude-428: one stream's worker thread. The caller hands it whole
/// partitions in one `Arc` shared with the order's other worker; the worker
/// encodes and hashes them in arrival order. Its queue holds
/// [`WORKER_QUEUE_DEPTH`] partitions behind the one in progress, which bounds
/// the partitions an order holds. A worker can only stop by panicking;
/// `finish` then re-raises that panic on the caller's thread.
enum WorkerInput {
    Partition(Arc<NgramPartition>),
    Drain(SyncSender<()>),
}

struct Worker<W: StreamWork> {
    sender: Option<SyncSender<WorkerInput>>,
    thread: Option<JoinHandle<(W::Output, u64)>>,
    /// claude-427: shared with the thread, which then times its work.
    timed: Arc<AtomicBool>,
    /// Nanoseconds a hand-off blocked on a full queue, while timed.
    wait_ns: u64,
}

/// Nanoseconds since `start`, saturated to u64.
fn nanos_since(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

impl<W: StreamWork> Worker<W> {
    fn new(name: String, mut work: W) -> Result<Self> {
        let (sender, receiver) = sync_channel::<WorkerInput>(WORKER_QUEUE_DEPTH);
        let timed = Arc::new(AtomicBool::new(false));
        let thread_timed = Arc::clone(&timed);
        let thread = std::thread::Builder::new()
            .name(name)
            .spawn(move || {
                let mut busy_ns = 0u64;
                for input in receiver {
                    match input {
                        WorkerInput::Partition(partition) => {
                            let start = thread_timed.load(Ordering::Relaxed).then(Instant::now);
                            work.feed(&partition);
                            if let Some(start) = start {
                                busy_ns = busy_ns.saturating_add(nanos_since(start));
                            }
                        }
                        WorkerInput::Drain(done) => {
                            let _ = done.send(());
                        }
                    }
                }
                let start = thread_timed.load(Ordering::Relaxed).then(Instant::now);
                let output = work.finish();
                if let Some(start) = start {
                    busy_ns = busy_ns.saturating_add(nanos_since(start));
                }
                (output, busy_ns)
            })
            .map_err(|error| invalid(format!("an n-gram hash worker did not start: {error}")))?;
        Ok(Self {
            sender: Some(sender),
            thread: Some(thread),
            timed,
            wait_ns: 0,
        })
    }

    /// A failed send means the thread has stopped; `finish` reports why.
    fn send(&mut self, partition: Arc<NgramPartition>) {
        if let Some(sender) = &self.sender {
            let start = self.timed.load(Ordering::Relaxed).then(Instant::now);
            let sent = sender.send(WorkerInput::Partition(partition));
            if let Some(start) = start {
                self.wait_ns = self.wait_ns.saturating_add(nanos_since(start));
            }
            if sent.is_err() {
                self.sender = None;
            }
        }
    }

    fn drain(&self) -> Result<()> {
        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| invalid("n-gram hash worker is unavailable"))?;
        let (done, received) = sync_channel(0);
        sender
            .send(WorkerInput::Drain(done))
            .map_err(|_| invalid("n-gram hash worker stopped before drain"))?;
        received
            .recv()
            .map_err(|_| invalid("n-gram hash worker stopped during drain"))
    }

    /// The worker's output, and its busy nanoseconds while timed.
    fn finish(mut self) -> (W::Output, u64) {
        let sent_all = self.sender.take().is_some();
        let joined = self.thread.take().map(JoinHandle::join);
        match joined {
            Some(Ok(done)) if sent_all => done,
            Some(Err(panic)) => std::panic::resume_unwind(panic),
            _ => panic!("an n-gram hash worker stopped before its stream ended"),
        }
    }
}

impl<W: StreamWork> Drop for Worker<W> {
    fn drop(&mut self) {
        self.sender = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// claude-427: where an assembler's `push` spends its time, summed over its
/// partitions, in nanoseconds. All zero unless
/// [`NgramAssembler::enable_timing`] was called. The two `send_wait` fields
/// are parts of the feed above them; the hasher `busy` fields are time on the
/// hash workers, beside the caller's. Since claude-428 the workers encode as
/// well as hash, and a feed is the caller's hand-off to its worker.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AssemblerTiming {
    /// The per-triple loop: order, vocabulary, ascent, nonzero count.
    pub triple_checks_ns: u64,
    /// The per-context loop: order, followers = n1 + n2 + n3+.
    pub context_checks_ns: u64,
    /// `check_census`, the count-of-counts comparison, the seam and
    /// cross-partition context order checks.
    pub census_ns: u64,
    /// The counters, then handing the partition to the triple worker, with
    /// any wait on its queue.
    pub triple_feed_ns: u64,
    /// Handing the partition to the context worker, with any wait on its queue.
    pub context_feed_ns: u64,
    pub triple_send_wait_ns: u64,
    pub context_send_wait_ns: u64,
    /// Filled by [`NgramAssembler::finish_with_timing`].
    pub triple_hash_busy_ns: u64,
    pub context_hash_busy_ns: u64,
}

/// A stopwatch read in splits; a no-op when off.
struct Splits(Option<Instant>);

impl Splits {
    fn new(on: bool) -> Self {
        Self(on.then(Instant::now))
    }

    /// Nanoseconds since the last split (0 when off), and restart.
    fn split(&mut self) -> u64 {
        match self.0.as_mut() {
            Some(last) => {
                let ns = nanos_since(*last);
                *last = Instant::now();
                ns
            }
            None => 0,
        }
    }
}

/// Assembles one order's partitions, in canonical order, into the canonical
/// streams and their SHA-256. Partitions cover disjoint ascending key ranges,
/// so the triple stream is their concatenation. A context may straddle a
/// boundary only at order 0 (the empty context); its records are added, since
/// every field is additive over disjoint follower sets. The context records
/// and n1..n4 are pure functions of the triples: the assembler derives them
/// from each partition's triples and refuses a partition whose own differ.
/// claude-428: every check runs on the caller's thread, and a partition that
/// passes them goes whole to the order's two hash workers.
pub struct NgramAssembler {
    order: u32,
    keep: bool,
    triple_worker: Worker<TripleWork>,
    context_worker: Worker<ContextWork>,
    last_triple: Option<(u128, u16)>,
    /// The key of the last context handed on, for the seam and order checks.
    last_context: Option<u128>,
    targets: u64,
    grams: u64,
    count_of_counts: [u64; 4],
    timing: Option<AssemblerTiming>,
    pub triples: Vec<NgramTriple>,
    pub context_records: Vec<NgramContext>,
}

impl NgramAssembler {
    /// claude-427: time `push` from now on, and its hash workers.
    pub fn enable_timing(&mut self) {
        self.timing.get_or_insert_with(AssemblerTiming::default);
        self.triple_worker.timed.store(true, Ordering::Relaxed);
        self.context_worker.timed.store(true, Ordering::Relaxed);
    }

    /// `keep` retains the streams in memory (tests); otherwise only hashes.
    pub fn new(order: u32, keep: bool) -> Result<Self> {
        Self::with_domains(order, keep, TRIPLE_DOMAIN, CONTEXT_DOMAIN, HASH_CHUNK_BYTES)
    }

    /// The assembler of adjusted counts (claude-418): the same checks and
    /// record formats under their own SHA-256 domains, so an adjusted stream
    /// never hashes like a raw one. Its digest's `targets` is the sum of the
    /// adjusted counts.
    pub fn new_adjusted(order: u32, keep: bool) -> Result<Self> {
        Self::with_domains(
            order,
            keep,
            ADJUSTED_TRIPLE_DOMAIN,
            ADJUSTED_CONTEXT_DOMAIN,
            HASH_CHUNK_BYTES,
        )
    }

    fn with_domains(
        order: u32,
        keep: bool,
        triples: &[u8],
        contexts: &[u8],
        chunk: usize,
    ) -> Result<Self> {
        if order > NGRAM_MAX_ORDER {
            return Err(invalid(format!("n-gram order {order} exceeds {NGRAM_MAX_ORDER}")));
        }
        let triple_work = TripleWork {
            out: ChunkedHash::new(triples, order, chunk),
        };
        let context_work = ContextWork {
            out: ChunkedHash::new(contexts, order, chunk),
            pending: None,
            emitted: 0,
            keep,
            kept: Vec::new(),
        };
        Ok(Self {
            order,
            keep,
            triple_worker: Worker::new(format!("ngram-triples-{order}"), triple_work)?,
            context_worker: Worker::new(format!("ngram-contexts-{order}"), context_work)?,
            last_triple: None,
            last_context: None,
            targets: 0,
            grams: 0,
            count_of_counts: [0; 4],
            timing: None,
            triples: Vec::new(),
            context_records: Vec::new(),
        })
    }

    /// Append the next partition. Refuses any break of canonical order and any
    /// context record or n1..n4 that differs from the one its triples imply.
    /// The partition is copied once for the hash workers; [`Self::push_owned`]
    /// avoids the copy.
    pub fn push(&mut self, partition: &NgramPartition) -> Result<()> {
        self.push_owned(partition.clone())
    }

    /// [`Self::push`], taking the partition, which the hash workers then share.
    /// A refused partition hands nothing on and changes no state.
    pub fn push_owned(&mut self, partition: NgramPartition) -> Result<()> {
        let mut splits = Splits::new(self.timing.is_some());
        let mut last = self.last_triple;
        for triple in &partition.triples {
            if (triple.key >> 120) as u32 != self.order {
                return Err(invalid("n-gram triple carries another order's key"));
            }
            if u32::from(triple.next) >= NGRAM_VOCABULARY {
                return Err(invalid("n-gram triple next token is outside the 12-bit vocabulary"));
            }
            let at = (triple.key, triple.next);
            if last.is_some_and(|before| before >= at) {
                return Err(invalid("n-gram triples are not in strictly ascending order"));
            }
            if triple.count == 0 {
                return Err(invalid("n-gram triple has a zero count"));
            }
            last = Some(at);
        }
        let triple_checks_ns = splits.split();
        for context in &partition.contexts {
            if (context.key >> 120) as u32 != self.order {
                return Err(invalid("n-gram context carries another order's key"));
            }
            let parts = u64::from(context.n1) + u64::from(context.n2) + u64::from(context.n3_plus);
            if u64::from(context.followers) != parts {
                return Err(invalid("n-gram context followers differ from n1 + n2 + n3+"));
            }
        }
        let context_checks_ns = splits.split();
        let derived_count_of_counts = check_census(&partition.triples, &partition.contexts)?;
        if partition.count_of_counts != derived_count_of_counts {
            return Err(invalid("n-gram count-of-counts differ from the ones the triples imply"));
        }
        let first_context = partition.contexts.first().map(|context| context.key);
        if let Some((before, first)) = self.last_context.zip(first_context) {
            if self.order >= 1 && before == first {
                return Err(invalid("an order >= 1 n-gram context straddles a partition seam"));
            }
            if before > first {
                return Err(invalid("n-gram contexts are not in ascending order"));
            }
        }
        let census_ns = splits.split();
        self.last_triple = last;
        if let Some(context) = partition.contexts.last() {
            self.last_context = Some(context.key);
        }
        for triple in &partition.triples {
            self.targets += u64::from(triple.count);
        }
        self.grams += partition.triples.len() as u64;
        for (slot, value) in self.count_of_counts.iter_mut().zip(partition.count_of_counts) {
            *slot += value;
        }
        if self.keep {
            self.triples.extend_from_slice(&partition.triples);
        }
        let shared = Arc::new(partition);
        self.triple_worker.send(Arc::clone(&shared));
        let triple_feed_ns = splits.split();
        self.context_worker.send(shared);
        let context_feed_ns = splits.split();
        if let Some(timing) = self.timing.as_mut() {
            timing.triple_checks_ns += triple_checks_ns;
            timing.context_checks_ns += context_checks_ns;
            timing.census_ns += census_ns;
            timing.triple_feed_ns += triple_feed_ns;
            timing.context_feed_ns += context_feed_ns;
        }
        Ok(())
    }

    pub fn finish(self) -> (NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>) {
        let (digest, triples, contexts, _) = self.finish_with_timing();
        (digest, triples, contexts)
    }

    /// Wait for both hash workers to release all previously submitted
    /// partition allocations without ending the canonical digest stream.
    pub fn drain_partitions(&self) -> Result<()> {
        self.triple_worker.drain()?;
        self.context_worker.drain()
    }

    /// `finish`, with the timing totals; all zero unless timing was enabled.
    pub fn finish_with_timing(
        self,
    ) -> (
        NgramOrderDigest,
        Vec<NgramTriple>,
        Vec<NgramContext>,
        AssemblerTiming,
    ) {
        let mut timing = self.timing.unwrap_or_default();
        timing.triple_send_wait_ns = self.triple_worker.wait_ns;
        timing.context_send_wait_ns = self.context_worker.wait_ns;
        let (triple_sha256, triple_busy_ns) = self.triple_worker.finish();
        let ((context_sha256, contexts, context_records), context_busy_ns) =
            self.context_worker.finish();
        timing.triple_hash_busy_ns = triple_busy_ns;
        timing.context_hash_busy_ns = context_busy_ns;
        let digest = NgramOrderDigest {
            order: self.order,
            targets: self.targets,
            grams: self.grams,
            contexts,
            count_of_counts: self.count_of_counts,
            triple_sha256,
            context_sha256,
        };
        (digest, self.triples, context_records, timing)
    }
}

/// Checks, in one pass and without a second vector, that `contexts` are
/// exactly the records a run of canonical triples implies (one record per run
/// of equal keys), and returns the n1..n4 the triples imply.
fn check_census(triples: &[NgramTriple], contexts: &[NgramContext]) -> Result<[u64; 4]> {
    let differ = || invalid("n-gram context records differ from the ones the triples imply");
    let mut count_of_counts = [0u64; 4];
    let mut running: Option<NgramContext> = None;
    let mut next = 0usize;
    for triple in triples {
        if (1..=4).contains(&triple.count) {
            count_of_counts[triple.count as usize - 1] += 1;
        }
        let h = match running.as_mut() {
            Some(h) if h.key == triple.key => h,
            _ => {
                if let Some(done) = running.take() {
                    if contexts.get(next) != Some(&done) {
                        return Err(differ());
                    }
                    next += 1;
                }
                running.insert(NgramContext {
                    key: triple.key,
                    total: 0,
                    followers: 0,
                    n1: 0,
                    n2: 0,
                    n3_plus: 0,
                })
            }
        };
        h.total += u64::from(triple.count);
        h.followers += 1;
        match triple.count {
            1 => h.n1 += 1,
            2 => h.n2 += 1,
            _ => h.n3_plus += 1,
        }
    }
    if let Some(done) = running {
        if contexts.get(next) != Some(&done) {
            return Err(differ());
        }
        next += 1;
    }
    if next != contexts.len() {
        return Err(differ());
    }
    Ok(count_of_counts)
}

/// The kernels' u32 index products (2g for keys, 4c + 3 for context records)
/// stay below 2^32 while a partition holds at most 2^30 targets.
pub const NGRAM_MAX_PARTITION_TARGETS: u64 = 1 << 30;
/// The kernels' grid-stride and tile arithmetic wraps above 2^32 - 65,536
/// positions; the resident stream is capped well below, at 2^31 tokens.
pub const NGRAM_MAX_STREAM_TOKENS: u64 = 1 << 31;

/// The target count of a partition the device may count: at most 2^30.
pub fn partition_target_bound(range: &PartitionRange) -> Result<u32> {
    if range.targets > NGRAM_MAX_PARTITION_TARGETS {
        return Err(invalid(format!(
            "n-gram partition {}..{} holds {} targets, above the 2^30 device bound",
            range.lo, range.hi, range.targets
        )));
    }
    Ok(range.targets as u32)
}

/// The length of a stream the device may hold: 1..=2^31 tokens.
pub fn stream_length_bound(len: usize) -> Result<u32> {
    if len == 0 || len as u64 > NGRAM_MAX_STREAM_TOKENS {
        return Err(invalid(format!("n-gram stream needs 1..=2^31 tokens, got {len}")));
    }
    Ok(len as u32)
}

/// One key-range partition: the partition token takes values lo..hi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartitionRange {
    pub lo: u32,
    pub hi: u32,
    pub targets: u64,
}

/// Greedy plan over the partition token's histogram: each partition is the
/// longest run of token values whose targets fit `budget`, or a single token
/// value that alone exceeds it. The plan covers 0..4096 in order.
pub fn plan_partitions(histogram: &[u64], budget: u64) -> Result<Vec<PartitionRange>> {
    if histogram.len() != NGRAM_VOCABULARY as usize || budget == 0 {
        return Err(invalid("partition plan needs a 4,096-bin histogram and a positive budget"));
    }
    let mut plan = Vec::new();
    let mut lo = 0u32;
    let mut sum = 0u64;
    for (token, &count) in histogram.iter().enumerate() {
        let token = token as u32;
        if token > lo && sum + count > budget {
            plan.push(PartitionRange { lo, hi: token, targets: sum });
            lo = token;
            sum = 0;
        }
        sum += count;
    }
    plan.push(PartitionRange { lo, hi: NGRAM_VOCABULARY, targets: sum });
    if let Some(p) = plan.iter().find(|p| p.targets > NGRAM_MAX_PARTITION_TARGETS) {
        return Err(invalid(format!(
            "partition plan: token values {}..{} hold {} targets, above the 2^30 partition bound",
            p.lo, p.hi, p.targets
        )));
    }
    Ok(plan)
}

/// The partition token of target j under order k: slot 1 (t[j-1]) for k >= 1,
/// the next token t[j] for k = 0. Its value is the most significant sort digit.
pub fn partition_offset(order: u32) -> usize {
    usize::from(order >= 1)
}

/// The 4,097 bucket starts of the slot-1 bucket sort (claude-425) from its
/// exclusively scanned bin-major [4096][tiles] histogram plus its total word:
/// bucket v holds sorted positions starts[v]..starts[v+1], and starts[4096]
/// must equal `positions`, the number of positions sorted.
pub fn bucket_starts(scanned: &[u32], tiles: usize, positions: u32) -> Result<Vec<u32>> {
    let bins = NGRAM_VOCABULARY as usize;
    if tiles == 0 || scanned.len() != bins * tiles + 1 {
        return Err(invalid("bucket histogram needs 4096 x tiles + 1 scanned words"));
    }
    let starts: Vec<u32> = (0..=bins).map(|v| scanned[v * tiles]).collect();
    if starts[0] != 0 || starts[bins] != positions || starts.windows(2).any(|w| w[0] > w[1]) {
        return Err(invalid(format!(
            "bucket starts run {}..{} over {positions} positions or decrease",
            starts[0], starts[bins]
        )));
    }
    Ok(starts)
}

/// The 64-byte UORT header, then little-endian u16 tokens.
pub fn read_uort_tokens(bytes: &[u8]) -> Result<Vec<u16>> {
    if bytes.len() < 64 || &bytes[..4] != b"UORT" {
        return Err(invalid("token stream lacks the 64-byte UORT header"));
    }
    let count = u64::from_le_bytes(bytes[8..16].try_into().expect("8 bytes"));
    let payload = &bytes[64..];
    let want = count
        .checked_mul(2)
        .ok_or_else(|| invalid(format!("UORT header names {count} tokens, past the u64 byte bound")))?;
    if payload.len() as u64 != want {
        return Err(invalid(format!(
            "UORT header names {count} tokens but the payload holds {} bytes",
            payload.len()
        )));
    }
    Ok(payload
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect())
}

/// Whether order `k` counts target `j` (k <= j): the target is not a BOS, and
/// no BOS lies inside the context except in its oldest slot `tokens[j - k]`,
/// so no gram reaches back across a document start.
pub fn ngram_admitted(tokens: &[u16], j: usize, k: usize) -> bool {
    debug_assert!(k <= j, "target {j} has no full order-{k} context");
    tokens[j] != NGRAM_BOS && (1..k).all(|s| tokens[j - s] != NGRAM_BOS)
}

/// Refuse any id outside the 12-bit vocabulary before it reaches a kernel.
pub fn check_tokens(tokens: &[u16]) -> Result<()> {
    match tokens.iter().position(|&t| u32::from(t) >= NGRAM_VOCABULARY) {
        Some(at) => Err(invalid(format!(
            "token {} at position {at} is outside the 12-bit vocabulary",
            tokens[at]
        ))),
        None => Ok(()),
    }
}

/// The census of a run of canonical triples: one context record per run of
/// equal keys, and n1..n4.
pub fn partition_from_triples(triples: Vec<NgramTriple>) -> NgramPartition {
    let mut partition = NgramPartition::default();
    for triple in &triples {
        let NgramTriple { key, count, .. } = *triple;
        if (1..=4).contains(&count) {
            partition.count_of_counts[count as usize - 1] += 1;
        }
        if partition.contexts.last().is_none_or(|h| h.key != key) {
            partition.contexts.push(NgramContext {
                key,
                total: 0,
                followers: 0,
                n1: 0,
                n2: 0,
                n3_plus: 0,
            });
        }
        let h = partition.contexts.last_mut().expect("context");
        h.total += u64::from(count);
        h.followers += 1;
        match count {
            1 => h.n1 += 1,
            2 => h.n2 += 1,
            _ => h.n3_plus += 1,
        }
    }
    partition.triples = triples;
    partition
}

/// One run of the left-extension census: the order-(k+1) triples whose
/// back-off key and next are (key, next). The triples of one run differ in
/// their oldest slot alone, so `types` is N1+(. key next).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LeftTypes {
    key: u128,
    next: u16,
    types: u32,
}

/// Sort the (suffix key, next) of every order-(k+1) triple and count each run.
/// Within an order-(k+1) partition the triples of one suffix are spread over
/// the oldest slot, which sorts before `next`, so a sort brings them together.
fn left_types(mut suffixes: Vec<(u128, u16)>) -> Vec<LeftTypes> {
    suffixes.sort_unstable();
    let mut runs: Vec<LeftTypes> = Vec::new();
    for (key, next) in suffixes {
        match runs.last_mut() {
            Some(run) if run.key == key && run.next == next => run.types += 1,
            _ => runs.push(LeftTypes { key, next, types: 1 }),
        }
    }
    runs
}

fn absent_suffix(order: u32) -> Error {
    invalid(format!(
        "adjusted counts: an order-{} triple's suffix gram is absent from the order-{order} partition",
        order + 1
    ))
}

/// Modified Kneser-Ney adjusted counts of one order-k partition, 1 <= k <= N
/// (claude-418). `counted` is order k's partition over partition tokens
/// lo..hi; `extended` is order k+1's partition over the same lo..hi, or None
/// when k = N. Both partition on slot 1 (t[j-1]) and back-off keeps that slot,
/// so the two hold exactly the grams the step needs. A gram g keeps a(g) = c(g)
/// when k = N or its oldest slot is BOS; otherwise a(g) = N1+(. g), the number
/// of order-(k+1) triples whose back-off key and next equal g's. Refuses a
/// zero adjusted count (rule 4), an order-(k+1) triple whose suffix is absent
/// or extends a document-start gram, and partitions of another order or range.
/// The output is a partition of adjusted counts for [`NgramAssembler::new_adjusted`].
pub fn adjust_partition(
    order: u32,
    highest: u32,
    lo: u32,
    hi: u32,
    counted: &NgramPartition,
    extended: Option<&NgramPartition>,
) -> Result<NgramPartition> {
    adjust_checks(order, highest, lo, hi)?;
    check_counted(order, lo, hi, counted)?;
    let runs = extension_runs(order, highest, lo, hi, extended)?;
    adjusted_from_runs(order, highest, counted, &runs)
}

/// The second half of [`adjust_partition`], over the runs
/// [`extension_runs`] took from the order-(k+1) partition (claude-422). The
/// device counter frees that partition before it counts order k, so the two
/// are never held at once; the output is the same partition.
pub(crate) fn adjust_partition_from_runs(
    order: u32,
    highest: u32,
    lo: u32,
    hi: u32,
    counted: &NgramPartition,
    runs: &[LeftTypes],
) -> Result<NgramPartition> {
    adjust_checks(order, highest, lo, hi)?;
    check_counted(order, lo, hi, counted)?;
    adjusted_from_runs(order, highest, counted, runs)
}

fn adjust_checks(order: u32, highest: u32, lo: u32, hi: u32) -> Result<()> {
    if order == 0 || order > highest || highest > NGRAM_MAX_ORDER {
        return Err(invalid(format!(
            "adjusted counts take an order-1..={highest} partition (order 0 is a whole-order step), got order {order} under N = {highest} <= {NGRAM_MAX_ORDER}"
        )));
    }
    if lo >= hi || hi > NGRAM_VOCABULARY {
        return Err(invalid(format!("adjusted counts need a token range lo < hi <= 4,096, got {lo}..{hi}")));
    }
    Ok(())
}

fn adjust_key(word: u128, want: u32, lo: u32, hi: u32) -> Result<PrimeRouteKey> {
    let key = PrimeRouteKey::from_word(word)?;
    if key.order() != want {
        return Err(invalid(format!(
            "adjusted counts: a key of order {} in an order-{want} partition",
            key.order()
        )));
    }
    let token = u32::from(key.slot(1).expect("order >= 1 has slot 1"));
    if token < lo || token >= hi {
        return Err(invalid(format!(
            "adjusted counts: partition token {token} lies outside {lo}..{hi}"
        )));
    }
    Ok(key)
}

fn check_counted(order: u32, lo: u32, hi: u32, counted: &NgramPartition) -> Result<()> {
    let mut last = None;
    for triple in &counted.triples {
        adjust_key(triple.key, order, lo, hi)?;
        let at = (triple.key, triple.next);
        if last.is_some_and(|before| before >= at) {
            return Err(invalid("adjusted counts: order-k triples are not in strictly ascending order"));
        }
        last = Some(at);
    }
    Ok(())
}

/// The left-extension runs of the order-(k+1) partition, or none when k = N
/// (the first half of [`adjust_partition`]).
pub(crate) fn extension_runs(
    order: u32,
    highest: u32,
    lo: u32,
    hi: u32,
    extended: Option<&NgramPartition>,
) -> Result<Vec<LeftTypes>> {
    adjust_checks(order, highest, lo, hi)?;
    let checked = |word: u128, want: u32| adjust_key(word, want, lo, hi);
    let runs = match (order < highest, extended) {
        (true, Some(extended)) => {
            let mut suffixes = Vec::with_capacity(extended.triples.len());
            let mut last_extended = None;
            for triple in &extended.triples {
                // Each order-(k+1) triple is one distinct left token, so a
                // repeated or zero-count triple would inflate N1+ (review MEDIUM).
                let at = (triple.key, triple.next);
                if triple.count == 0 || last_extended.is_some_and(|before| before >= at) {
                    return Err(invalid(
                        "adjusted counts: order-(k+1) triples are not strictly ascending with counts >= 1",
                    ));
                }
                last_extended = Some(at);
                let suffix = checked(triple.key, order + 1)?.back_off().expect("order >= 2");
                suffixes.push((suffix.word(), triple.next));
            }
            left_types(suffixes)
        }
        (false, None) => Vec::new(),
        (true, None) => {
            return Err(invalid(format!(
                "adjusted counts of order {order} < N = {highest} need the order-{} partition",
                order + 1
            )))
        }
        (false, Some(_)) => {
            return Err(invalid(format!(
                "adjusted counts of the highest order N = {highest} take no order-{} partition",
                order + 1
            )))
        }
    };
    Ok(runs)
}

fn adjusted_from_runs(
    order: u32,
    highest: u32,
    counted: &NgramPartition,
    runs: &[LeftTypes],
) -> Result<NgramPartition> {
    let mut at = 0usize;
    let mut adjusted = Vec::with_capacity(counted.triples.len());
    for triple in &counted.triples {
        let NgramTriple { key, next, count } = *triple;
        let mut types = 0u32;
        while let Some(&LeftTypes { key: run_key, next: run_next, types: run_types }) = runs.get(at) {
            if (run_key, run_next) > (key, next) {
                break;
            }
            if (run_key, run_next) < (key, next) {
                return Err(absent_suffix(order));
            }
            types = run_types;
            at += 1;
        }
        let document_start = PrimeRouteKey(key).slot(order) == Some(NGRAM_BOS);
        let count = if order == highest {
            count
        } else if document_start {
            if types != 0 {
                return Err(invalid(format!(
                    "adjusted counts: an order-{} triple extends an order-{order} gram whose oldest slot is BOS",
                    order + 1
                )));
            }
            count
        } else {
            if types == 0 {
                return Err(invalid(format!(
                    "adjusted counts: order-{order} gram (key {key:#x}, next {next}) has no counted left extension, adjusted count 0 (rule 4)"
                )));
            }
            types
        };
        adjusted.push(NgramTriple { key, next, count });
    }
    if at != runs.len() {
        return Err(absent_suffix(order));
    }
    Ok(partition_from_triples(adjusted))
}

/// Order 0's left-extension census (claude-418): N1+(. w) for every target w,
/// summed over all order-1 partitions. Order 0 partitions on t[j] and order 1
/// on t[j-1], so no single order-1 partition holds a unigram's extensions;
/// each order-1 triple (v, w) is one distinct left token v of w, and adding
/// every order-1 partition once gives the whole census (at most 4,096 bins).
pub struct UnigramContinuation {
    types: Vec<u32>,
    /// The last order-1 (key, next) added: partitions arrive in plan order, so
    /// the whole order-1 stream is strictly ascending, and a partition added
    /// twice or two overlapping partitions are refused.
    last: Option<(u128, u16)>,
}

impl Default for UnigramContinuation {
    fn default() -> Self {
        Self::new()
    }
}

impl UnigramContinuation {
    pub fn new() -> Self {
        Self { types: vec![0; NGRAM_VOCABULARY as usize], last: None }
    }

    /// Add one order-1 partition, in plan order. Each partition is added
    /// exactly once: a repeated or overlapping one breaks the strictly
    /// ascending order-1 stream and is refused.
    pub fn add(&mut self, extended: &NgramPartition) -> Result<()> {
        for triple in &extended.triples {
            if PrimeRouteKey::from_word(triple.key)?.order() != 1 {
                return Err(invalid("unigram continuation takes order-1 triples only"));
            }
            if u32::from(triple.next) >= NGRAM_VOCABULARY {
                return Err(invalid("unigram continuation: next token is outside the 12-bit vocabulary"));
            }
            let at = (triple.key, triple.next);
            if triple.count == 0 || self.last.is_some_and(|before| before >= at) {
                return Err(invalid(
                    "unigram continuation: order-1 triples are not strictly ascending across partitions with counts >= 1",
                ));
            }
            self.last = Some(at);
            self.types[triple.next as usize] += 1;
        }
        Ok(())
    }

    /// The adjusted unigrams of the whole order-0 triple stream (ascending,
    /// at most 4,096 triples) under highest order N. N = 0 keeps a = c and
    /// takes no order-1 partition. Refuses a zero adjusted count (rule 4) and
    /// an order-1 target absent from the order-0 stream.
    pub fn adjust(&self, highest: u32, counted: &[NgramTriple]) -> Result<NgramPartition> {
        if highest > NGRAM_MAX_ORDER {
            return Err(invalid(format!("highest order {highest} exceeds {NGRAM_MAX_ORDER}")));
        }
        if highest == 0 && self.last.is_some() {
            return Err(invalid("adjusted unigrams of the highest order N = 0 take no order-1 partition"));
        }
        let mut last: Option<u16> = None;
        let mut covered = 0u64;
        let mut adjusted = Vec::with_capacity(counted.len());
        for triple in counted {
            let NgramTriple { key, next, count } = *triple;
            if key != 0 {
                return Err(invalid("adjusted unigrams take order-0 triples only"));
            }
            if u32::from(next) >= NGRAM_VOCABULARY || last.is_some_and(|before| before >= next) {
                return Err(invalid("adjusted unigrams: next tokens are not ascending 12-bit ids"));
            }
            last = Some(next);
            let types = self.types[next as usize];
            covered += u64::from(types);
            let count = if highest == 0 {
                count
            } else if types == 0 {
                return Err(invalid(format!(
                    "adjusted unigrams: target {next} has no counted left extension, adjusted count 0 (rule 4)"
                )));
            } else {
                types
            };
            adjusted.push(NgramTriple { key, next, count });
        }
        if covered != self.types.iter().map(|&t| u64::from(t)).sum::<u64>() {
            return Err(if highest == 0 {
                invalid("adjusted unigrams of the highest order N = 0 take no order-1 partition")
            } else {
                absent_suffix(0)
            });
        }
        Ok(partition_from_triples(adjusted))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ngram_count_oracle::{
        oracle_adjusted, oracle_count, oracle_partition, plant_one_count,
    };

    /// claude-431: the parallel builder equals the sequential map for sizes
    /// around its threshold and thread counts beyond the chunk count.
    #[test]
    fn ngram_count_build_indexed_equals_the_sequential_map() {
        let f = |i: usize| (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 7;
        let sizes = [1usize << 16, (1 << 16) - 1, (1 << 16) + 1];
        for n in [0usize, 1, 7, 1_000_003].into_iter().chain(sizes) {
            let want = (0..n).map(f).collect::<Vec<_>>();
            for threads in [1usize, 2, 3, 8, 64] {
                let got = build_indexed(n, threads, f);
                assert_eq!(got, want, "n {n} threads {threads}");
            }
        }
    }

    /// The device's bin-major histogram and its exclusive scan, on the host.
    fn scanned_bucket_hist(tokens: &[u16], tile: usize) -> (Vec<u32>, usize) {
        let positions = tokens.len() - 1;
        let tiles = positions.div_ceil(tile).max(1);
        let mut hist = vec![0u32; 4096 * tiles + 1];
        for j in 1..tokens.len() {
            hist[usize::from(tokens[j - 1]) * tiles + (j - 1) / tile] += 1;
        }
        let mut sum = 0;
        for word in hist.iter_mut() {
            let v = *word;
            *word = sum;
            sum += v;
        }
        (hist, tiles)
    }

    #[test]
    fn ngram_count_bucket_starts_bound_each_slot_one_bucket() {
        let tokens: Vec<u16> = (0..1000u32).map(|i| ((i * 37 + i / 7) % 4096) as u16).collect();
        let (scanned, tiles) = scanned_bucket_hist(&tokens, 64);
        let starts = bucket_starts(&scanned, tiles, 999).expect("starts");
        assert_eq!(starts.len(), 4097);
        for v in 0..4096usize {
            let want = tokens[..999].iter().filter(|&&t| usize::from(t) == v).count() as u32;
            assert_eq!(starts[v + 1] - starts[v], want, "bucket {v}");
        }
        // A total that disagrees, a wrong length or a decreasing start is refused.
        assert!(bucket_starts(&scanned, tiles, 998).is_err());
        assert!(bucket_starts(&scanned[1..], tiles, 999).is_err());
        let mut bent = scanned.clone();
        bent[5 * tiles] = bent[6 * tiles] + 1;
        assert!(bucket_starts(&bent, tiles, 999).is_err());
    }

    #[test]
    fn ngram_count_key_round_trip_and_back_off() {
        for context in [&[][..], &[7], &[0, 4095], &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]] {
            let key = PrimeRouteKey::from_context(context).unwrap();
            assert_eq!(key.order() as usize, context.len());
            assert_eq!(key.context(), context);
            assert_eq!(PrimeRouteKey::from_word(key.word()).unwrap(), key);
            match key.back_off() {
                None => assert!(context.is_empty()),
                Some(shorter) => {
                    assert_eq!(shorter.context(), &context[1..]);
                    assert_eq!(shorter, PrimeRouteKey::from_context(&context[1..]).unwrap());
                }
            }
        }
        let key = PrimeRouteKey::from_context(&[11, 22, 33]).unwrap();
        assert_eq!((key.slot(1), key.slot(2), key.slot(3)), (Some(33), Some(22), Some(11)));
        assert_eq!((key.slot(0), key.slot(4), key.slot(11)), (None, None, None));
        let full = PrimeRouteKey::from_context(&[1; 10]).unwrap();
        assert_eq!((full.slot(10), full.slot(11)), (Some(1), None));
        assert!(PrimeRouteKey::from_context(&[4096]).is_err());
        assert!(PrimeRouteKey::from_context(&[0; 11]).is_err());
        assert!(PrimeRouteKey::from_word(key.word() | 1).is_err());
        assert!(PrimeRouteKey::from_word(11u128 << 120).is_err());
    }

    #[test]
    fn ngram_count_key_preserves_order() {
        let (a, b, c) = (17u16, 300u16, 4001u16);
        let abc = PrimeRouteKey::from_context(&[a, b, c]).unwrap();
        let cba = PrimeRouteKey::from_context(&[c, b, a]).unwrap();
        assert_ne!(abc, cba, "\"a b c\" and \"c b a\" must give different keys");
        assert_ne!(
            PrimeRouteKey::from_context(&[a, b]).unwrap(),
            PrimeRouteKey::from_context(&[b, a]).unwrap()
        );
        let the = 5u16;
        let x = 9u16;
        assert_ne!(
            PrimeRouteKey::from_context(&[the, x, the]).unwrap(),
            PrimeRouteKey::from_context(&[x, the]).unwrap()
        );
        // Counted: the stream "a b c | c b a" has order-3 context a b c once
        // and c b a once, as two distinct contexts (EOS = 1 separates them:
        // a BOS target would not be counted).
        let stream = [a, b, c, 1, c, b, a, 1];
        let (digest, _, contexts) = oracle_count(&stream, 3).unwrap();
        assert!(contexts.iter().any(|h| h.key == abc.word()));
        assert!(contexts.iter().any(|h| h.key == cba.word()));
        assert_eq!(digest.contexts, 5);
    }

    /// Hand fixture: "1 2 1 2 1 3 1 2", n = 8.
    #[test]
    fn ngram_count_oracle_hand_fixture() {
        let stream = [1u16, 2, 1, 2, 1, 3, 1, 2];
        let key = |c: &[u16]| PrimeRouteKey::from_context(c).unwrap().word();
        // Order 0: 1 x4, 2 x3, 3 x1, one context.
        let (d0, t0, h0) = oracle_count(&stream, 0).unwrap();
        assert_eq!(
            t0,
            vec![
                NgramTriple { key: key(&[]), next: 1, count: 4 },
                NgramTriple { key: key(&[]), next: 2, count: 3 },
                NgramTriple { key: key(&[]), next: 3, count: 1 },
            ]
        );
        assert_eq!(
            h0,
            vec![NgramContext { key: key(&[]), total: 8, followers: 3, n1: 1, n2: 0, n3_plus: 2 }]
        );
        assert_eq!((d0.targets, d0.count_of_counts), (8, [1, 0, 1, 1]));
        // Order 1: targets j = 1..8. Pairs (1,2) x3, (2,1) x2, (1,3) x1, (3,1) x1.
        let (d1, t1, h1) = oracle_count(&stream, 1).unwrap();
        assert_eq!(
            t1,
            vec![
                NgramTriple { key: key(&[1]), next: 2, count: 3 },
                NgramTriple { key: key(&[1]), next: 3, count: 1 },
                NgramTriple { key: key(&[2]), next: 1, count: 2 },
                NgramTriple { key: key(&[3]), next: 1, count: 1 },
            ]
        );
        assert_eq!(
            h1,
            vec![
                NgramContext { key: key(&[1]), total: 4, followers: 2, n1: 1, n2: 0, n3_plus: 1 },
                NgramContext { key: key(&[2]), total: 2, followers: 1, n1: 0, n2: 1, n3_plus: 0 },
                NgramContext { key: key(&[3]), total: 1, followers: 1, n1: 1, n2: 0, n3_plus: 0 },
            ]
        );
        assert_eq!((d1.targets, d1.count_of_counts), (7, [2, 1, 1, 0]));
        // Order 2: (1 2 -> 1) x2, (2 1 -> 2) x1, (2 1 -> 3) x1, (1 3 -> 1), (3 1 -> 2).
        let (d2, t2, _) = oracle_count(&stream, 2).unwrap();
        assert_eq!(d2.targets, 6);
        assert_eq!(d2.count_of_counts, [4, 1, 0, 0]);
        assert_eq!(t2[0], NgramTriple { key: key(&[2, 1]), next: 2, count: 1 });
        assert_eq!(t2.iter().map(|t| t.count).sum::<u32>(), 6);
        // Order 8 needs j >= 8, and n = 8: no target.
        let (d8, t8, _) = oracle_count(&stream, 8).unwrap();
        assert_eq!((d8.targets, t8.len()), (0, 0));
    }

    #[test]
    fn ngram_count_oracle_respects_document_boundaries() {
        let stream = [0u16, 5, 6, 7, 1, 0, 5, 6, 1];
        let (_, triples, _) = oracle_count(&stream, 2).unwrap();
        let key = |a: u16, b: u16| PrimeRouteKey::from_context(&[a, b]).unwrap().word();
        let count = |a: u16, b: u16, next: u16| {
            triples
                .iter()
                .find(|t| t.key == key(a, b) && t.next == next)
                .map(|t| t.count)
        };
        assert_eq!(count(0, 5, 6), Some(2));
        assert_eq!(count(7, 1, 0), None);
        assert_eq!(count(1, 0, 5), None);
        assert!(triples.iter().all(|t| t.next != NGRAM_BOS));
        let total: u32 = triples.iter().map(|t| t.count).sum();
        assert_eq!((triples.len(), total), (4, 5));
        // Order 0 drops only the BOS targets.
        let (_, unigrams, _) = oracle_count(&stream, 0).unwrap();
        let total: u32 = unigrams.iter().map(|t| t.count).sum();
        assert_eq!(total, 7);
    }

    #[test]
    fn ngram_count_partitions_assemble_to_the_whole() {
        let mut state = 0x412u64;
        let stream: Vec<u16> = (0..20_000)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                // Skewed: most mass on a few ids so high orders repeat.
                let r = (state >> 20) as u32;
                (if r.is_multiple_of(4) {
                    r % 4096
                } else {
                    r % 13
                }) as u16
            })
            .collect();
        // r % 13 plants BOS = 0 at about one position in eighteen, so the
        // partitions and the assembler see document boundaries.
        assert!(stream.iter().filter(|&&t| t == NGRAM_BOS).count() > 500);
        for order in [0u32, 1, 2, 5] {
            let (whole, triples, contexts) = oracle_count(&stream, order).unwrap();
            let offset = partition_offset(order);
            let mut histogram = vec![0u64; 4096];
            for j in order as usize..stream.len() {
                if ngram_admitted(&stream, j, order as usize) {
                    histogram[stream[j - offset] as usize] += 1;
                }
            }
            for budget in [u64::MAX, 3_000, 500] {
                let plan = plan_partitions(&histogram, budget).unwrap();
                assert_eq!(plan.first().unwrap().lo, 0);
                assert_eq!(plan.last().unwrap().hi, 4096);
                let mut assembler = NgramAssembler::new(order, true).unwrap();
                for range in &plan {
                    assert!(range.targets <= budget || range.hi == range.lo + 1);
                    assembler
                        .push(&oracle_partition(&stream, order, range.lo, range.hi).unwrap())
                        .unwrap();
                }
                let (digest, t, h) = assembler.finish();
                assert_eq!(digest, whole, "order {order} budget {budget}");
                assert_eq!((t, h), (triples.clone(), contexts.clone()));
            }
        }
    }

    /// A single partition token value cannot be split, so the plan refuses one
    /// that alone holds more than the device's 2^30 targets, at plan time.
    #[test]
    fn ngram_count_plan_refuses_a_partition_past_the_target_limit() {
        let mut histogram = vec![0u64; 4096];
        histogram[7] = NGRAM_MAX_PARTITION_TARGETS;
        histogram[8] = 5;
        let plan = plan_partitions(&histogram, NGRAM_MAX_PARTITION_TARGETS).expect("2^30 admitted");
        assert!(plan.iter().any(|p| p.targets == NGRAM_MAX_PARTITION_TARGETS));
        assert!(plan.iter().all(|p| p.targets <= NGRAM_MAX_PARTITION_TARGETS));
        histogram[7] = NGRAM_MAX_PARTITION_TARGETS + 1;
        let refusal = plan_partitions(&histogram, NGRAM_MAX_PARTITION_TARGETS).unwrap_err();
        assert!(refusal.to_string().contains("above the 2^30 partition bound"), "{refusal}");
    }

    /// A consistent plant (count, context record and n1..n4 adjusted together)
    /// moves the triple SHA: the SHA covers the count field.
    #[test]
    fn ngram_count_assembler_refuses_disorder_and_the_sha_covers_a_planted_count() {
        let stream = [1u16, 2, 1, 2, 1, 3, 1, 2];
        let (digest, triples, _) = oracle_count(&stream, 1).unwrap();
        let mut planted = oracle_partition(&stream, 1, 0, 4096).unwrap();
        plant_one_count(&mut planted, 1);
        let mut assembler = NgramAssembler::new(1, false).unwrap();
        assembler.push(&planted).unwrap();
        let (planted_digest, ..) = assembler.finish();
        assert_ne!(planted_digest.triple_sha256, digest.triple_sha256);
        let mut swapped = oracle_partition(&stream, 1, 0, 4096).unwrap();
        swapped.triples.swap(0, 1);
        assert!(NgramAssembler::new(1, false).unwrap().push(&swapped).is_err());
        assert_eq!(triples.len(), 4);
        assert!(check_tokens(&[0, 4095]).is_ok());
        assert!(check_tokens(&[0, 4096]).is_err());
    }

    /// claude-426: the threaded hashers give the digest of SHA-256 computed
    /// inline over the domain, the order byte and the encoded records, for
    /// raw and adjusted domains, at chunk sizes that cut records mid-way and
    /// one that sends the whole stream at `finish`.
    #[test]
    fn ngram_count_threaded_hash_equals_inline_sha() {
        let stream: Vec<u16> = (0..3_000u32)
            .map(|i| {
                if i % 97 == 0 {
                    NGRAM_BOS
                } else {
                    (5 + (i * i * 7 + i) % 41) as u16
                }
            })
            .collect();
        let domains = [
            (TRIPLE_DOMAIN, CONTEXT_DOMAIN),
            (ADJUSTED_TRIPLE_DOMAIN, ADJUSTED_CONTEXT_DOMAIN),
        ];
        for order in 0..=3u32 {
            let parts = [(0u32, 20u32), (20, 30), (30, 4096)]
                .map(|(lo, hi)| oracle_partition(&stream, order, lo, hi).expect("partition"));
            for (triple_domain, context_domain) in domains {
                for chunk in [1, 7, TRIPLE_BYTES, CONTEXT_BYTES, 1 << 20] {
                    let mut assembler = NgramAssembler::with_domains(
                        order,
                        true,
                        triple_domain,
                        context_domain,
                        chunk,
                    )
                    .expect("assembler");
                    for part in &parts {
                        assembler.push(part).expect("canonical partition");
                    }
                    let (digest, triples, contexts) = assembler.finish();
                    assert!(triples.len() > 10 && !contexts.is_empty(), "order {order}");
                    let mut inline = Sha256::new();
                    inline.update(triple_domain);
                    inline.update([order as u8]);
                    for triple in &triples {
                        inline.update(triple_bytes(triple));
                    }
                    let mut inline_contexts = Sha256::new();
                    inline_contexts.update(context_domain);
                    inline_contexts.update([order as u8]);
                    for context in &contexts {
                        inline_contexts.update(context_bytes(context));
                    }
                    let at = format!("order {order} chunk {chunk}");
                    assert_eq!(
                        digest.triple_sha256,
                        format!("{:x}", inline.finalize()),
                        "{at}"
                    );
                    assert_eq!(
                        digest.context_sha256,
                        format!("{:x}", inline_contexts.finalize()),
                        "{at}"
                    );
                }
            }
        }
    }

    /// claude-427: timing changes no digest; with it on, every sub-phase of a
    /// non-trivial push and both hashers record time, and a channel wait never
    /// exceeds the feed it is part of; with it off, every total stays zero.
    #[test]
    fn ngram_count_push_timing_is_recorded_only_when_enabled() {
        let stream: Vec<u16> = (0..3_000u32)
            .map(|i| {
                if i % 97 == 0 {
                    NGRAM_BOS
                } else {
                    (5 + (i * i * 7 + i) % 41) as u16
                }
            })
            .collect();
        for order in 1..=3u32 {
            let parts = [(0u32, 20u32), (20, 30), (30, 4096)]
                .map(|(lo, hi)| oracle_partition(&stream, order, lo, hi).expect("partition"));
            let run = |timed: bool| {
                let mut assembler = NgramAssembler::with_domains(
                    order,
                    false,
                    ADJUSTED_TRIPLE_DOMAIN,
                    ADJUSTED_CONTEXT_DOMAIN,
                    7,
                )
                .expect("assembler");
                if timed {
                    assembler.enable_timing();
                }
                for part in &parts {
                    assembler.push(part).expect("canonical partition");
                }
                let (digest, _, _, timing) = assembler.finish_with_timing();
                (digest, timing)
            };
            let (plain, off) = run(false);
            let (timed, on) = run(true);
            assert!(plain.grams > 10 && plain.contexts > 1, "order {order}");
            assert_eq!(timed, plain, "order {order}: timing changed the digest");
            assert_eq!(off, AssemblerTiming::default(), "order {order}");
            for (name, ns) in [
                ("triple checks", on.triple_checks_ns),
                ("context checks", on.context_checks_ns),
                ("census", on.census_ns),
                ("triple feed", on.triple_feed_ns),
                ("context feed", on.context_feed_ns),
                ("triple hasher", on.triple_hash_busy_ns),
                ("context hasher", on.context_hash_busy_ns),
            ] {
                assert!(ns > 0, "order {order}: {name} recorded no time");
            }
            assert!(on.triple_send_wait_ns <= on.triple_feed_ns, "order {order}");
            assert!(
                on.context_send_wait_ns <= on.context_feed_ns,
                "order {order}"
            );
        }
    }

    /// SHA-256 of the domain, the order byte and the encoded records.
    fn inline_digests(
        order: u32,
        domains: (&[u8], &[u8]),
        triples: &[NgramTriple],
        contexts: &[NgramContext],
    ) -> (String, String) {
        let mut triple_hash = Sha256::new();
        triple_hash.update(domains.0);
        triple_hash.update([order as u8]);
        for triple in triples {
            triple_hash.update(triple_bytes(triple));
        }
        let mut context_hash = Sha256::new();
        context_hash.update(domains.1);
        context_hash.update([order as u8]);
        for context in contexts {
            context_hash.update(context_bytes(context));
        }
        (
            format!("{:x}", triple_hash.finalize()),
            format!("{:x}", context_hash.finalize()),
        )
    }

    /// claude-428: partitions handed to the hash workers, owned or borrowed,
    /// give the digests and records of the whole stream counted at once. The
    /// cuts include 64 narrow partitions handed faster than a 1-byte-chunk
    /// worker drains them, and order 0's one context straddles every cut.
    #[test]
    fn ngram_count_owned_push_equals_inline_sha() {
        let stream: Vec<u16> = (0..3_000u32)
            .map(|i| {
                if i % 97 == 0 {
                    NGRAM_BOS
                } else {
                    (5 + (i * i * 7 + i) % 41) as u16
                }
            })
            .collect();
        let domains: (&[u8], &[u8]) = (ADJUSTED_TRIPLE_DOMAIN, ADJUSTED_CONTEXT_DOMAIN);
        let wide = vec![(0u32, 20u32), (20, 30), (30, 4096)];
        let mut narrow: Vec<(u32, u32)> = (0..63u32).map(|i| (i, i + 1)).collect();
        narrow.push((63, 4096));
        for order in 0..=3u32 {
            let whole = oracle_partition(&stream, order, 0, 4096).expect("whole stream");
            let want = inline_digests(order, domains, &whole.triples, &whole.contexts);
            for cuts in [&wide, &narrow] {
                let parts = cuts
                    .iter()
                    .map(|&(lo, hi)| oracle_partition(&stream, order, lo, hi).expect("partition"))
                    .collect::<Vec<_>>();
                if order == 0 {
                    let holding = parts
                        .iter()
                        .filter(|part| !part.contexts.is_empty())
                        .count();
                    assert!(
                        holding >= 3,
                        "order 0's context straddles {holding} partitions"
                    );
                }
                for chunk in [1, 7, TRIPLE_BYTES, 1 << 20] {
                    let mut assembler =
                        NgramAssembler::with_domains(order, true, domains.0, domains.1, chunk)
                            .expect("assembler");
                    for (at, part) in parts.iter().enumerate() {
                        match at % 2 {
                            0 => assembler.push_owned(part.clone()),
                            _ => assembler.push(part),
                        }
                        .expect("canonical partition");
                    }
                    let (digest, triples, contexts) = assembler.finish();
                    let at = format!("order {order} cuts {} chunk {chunk}", cuts.len());
                    assert!(whole.triples.len() > 10, "{at}");
                    assert_eq!(triples, whole.triples, "{at}");
                    assert_eq!(contexts, whole.contexts, "{at}");
                    assert_eq!(digest.contexts, whole.contexts.len() as u64, "{at}");
                    assert_eq!(digest.grams, whole.triples.len() as u64, "{at}");
                    assert_eq!(
                        (digest.triple_sha256, digest.context_sha256),
                        want.clone(),
                        "{at}"
                    );
                }
            }
        }
    }

    /// claude-428: a refused partition hands nothing to the workers and
    /// changes no state; the digest is the digest without it.
    #[test]
    fn ngram_count_refused_partition_hands_nothing_on() {
        let stream: Vec<u16> = (0..3_000u32)
            .map(|i| (5 + (i * 13 + i / 7) % 37) as u16)
            .collect();
        for order in 1..=2u32 {
            let parts = [(0u32, 20u32), (20, 4096)]
                .map(|(lo, hi)| oracle_partition(&stream, order, lo, hi).expect("partition"));
            let mut planted = parts[1].clone();
            planted.count_of_counts[0] += 1;
            let mut clean = NgramAssembler::new(order, true).expect("assembler");
            let mut refused = NgramAssembler::new(order, true).expect("assembler");
            clean.push(&parts[0]).expect("first");
            refused.push(&parts[0]).expect("first");
            let census = refused
                .push_owned(planted)
                .expect_err("planted count-of-counts");
            assert!(census.to_string().contains("count-of-counts"), "{census}");
            // Handed again, the partition goes backwards. The triple-order
            // check refuses it; the context-order check behind the census
            // cannot fire on its own, since a partition's first context key
            // equals its first triple key.
            let backwards = refused
                .push(&parts[0])
                .expect_err("a partition handed twice");
            assert!(
                backwards
                    .to_string()
                    .contains("n-gram triples are not in strictly ascending order"),
                "{backwards}"
            );
            clean.push(&parts[1]).expect("second");
            refused.push(&parts[1]).expect("second after the refusals");
            assert_eq!(refused.finish(), clean.finish(), "order {order}");
        }
        // The seam refusal is the one check after the census that can fire:
        // an order-1 context split across two partitions. It must also hand
        // nothing on.
        let key = PrimeRouteKey::from_context(&[7]).unwrap().word();
        let half = |next: u16| NgramPartition {
            triples: vec![NgramTriple {
                key,
                next,
                count: 1,
            }],
            contexts: vec![NgramContext {
                key,
                total: 1,
                followers: 1,
                n1: 1,
                n2: 0,
                n3_plus: 0,
            }],
            count_of_counts: [1, 0, 0, 0],
        };
        let mut clean = NgramAssembler::new(1, true).expect("assembler");
        let mut refused = NgramAssembler::new(1, true).expect("assembler");
        clean.push_owned(half(1)).expect("first half");
        refused.push_owned(half(1)).expect("first half");
        let seam = refused
            .push_owned(half(2))
            .expect_err("a context across a seam");
        assert!(
            seam.to_string().contains("straddles a partition seam"),
            "{seam}"
        );
        assert_eq!(refused.finish(), clean.finish(), "seam");
    }

    fn refusal(order: u32, partitions: &[NgramPartition]) -> String {
        let mut assembler = NgramAssembler::new(order, false).unwrap();
        for partition in partitions {
            if let Err(error) = assembler.push(partition) {
                return error.to_string();
            }
        }
        panic!("the assembler admitted every partition");
    }

    const HAND: [u16; 8] = [1, 2, 1, 2, 1, 3, 1, 2];

    #[test]
    fn ngram_count_assembler_refuses_a_planted_count_of_counts() {
        let mut partition = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        partition.count_of_counts[0] += 1;
        assert!(refusal(1, &[partition]).contains("count-of-counts differ"));
    }

    #[test]
    fn ngram_count_assembler_refuses_a_corrupted_context_n1() {
        let mut partition = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        // Context [1]: followers 2 = n1 1 + n3+ 1. Move one follower from n3+
        // to n1, so followers = n1 + n2 + n3+ still holds and only the
        // derivation from the triples can see it.
        assert_eq!((partition.contexts[0].n1, partition.contexts[0].n3_plus), (1, 1));
        partition.contexts[0].n1 += 1;
        partition.contexts[0].n3_plus -= 1;
        assert!(refusal(1, &[partition]).contains("context records differ"));
    }

    /// A partition with one context record more, or one fewer, than its
    /// triples imply is refused by the streamed comparison.
    #[test]
    fn ngram_count_assembler_refuses_an_extra_or_missing_context() {
        let mut extra = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        let mut last = *extra.contexts.last().unwrap();
        last.key += 1;
        extra.contexts.push(last);
        assert!(refusal(1, &[extra]).contains("context records differ"));
        let mut missing = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        missing.contexts.pop();
        assert!(refusal(1, &[missing]).contains("context records differ"));
    }

    #[test]
    fn ngram_count_assembler_admits_unmodified_oracle_partitions() {
        for order in 0..=3 {
            for (lo, hi) in [(0, 4096), (0, 2), (2, 4096)] {
                let partition = oracle_partition(&HAND, order, lo, hi).unwrap();
                NgramAssembler::new(order, false).unwrap().push(&partition).unwrap();
            }
        }
    }

    /// Two partitions whose triples ascend, each self-consistent, that share an
    /// order-1 context across their seam.
    #[test]
    fn ngram_count_assembler_refuses_an_order_one_context_across_a_seam() {
        let key = PrimeRouteKey::from_context(&[7]).unwrap().word();
        let half = |next: u16| NgramPartition {
            triples: vec![NgramTriple { key, next, count: 1 }],
            contexts: vec![NgramContext { key, total: 1, followers: 1, n1: 1, n2: 0, n3_plus: 0 }],
            count_of_counts: [1, 0, 0, 0],
        };
        assert!(refusal(1, &[half(1), half(2)]).contains("straddles a partition seam"));
        // Order 0's empty context may straddle.
        let empty = PrimeRouteKey::from_context(&[]).unwrap().word();
        let mut assembler = NgramAssembler::new(0, true).unwrap();
        for next in [1u16, 2] {
            let mut part = half(next);
            part.triples[0].key = empty;
            part.contexts[0].key = empty;
            assembler.push(&part).unwrap();
        }
        let (digest, _, contexts) = assembler.finish();
        assert_eq!((digest.contexts, contexts[0].followers, contexts[0].total), (1, 2, 2));
    }

    #[test]
    fn ngram_count_assembler_refuses_a_next_token_outside_the_vocabulary() {
        let mut partition = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        let last = partition.triples.len() - 1;
        partition.triples[last].next = 4096;
        assert!(refusal(1, &[partition]).contains("next token is outside"));
    }

    #[test]
    fn ngram_count_assembler_refuses_a_context_of_another_order() {
        let mut partition = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        partition.contexts[0].key ^= 3u128 << 120;
        assert!(refusal(1, &[partition]).contains("context carries another order"));
    }

    #[test]
    fn ngram_count_assembler_refuses_followers_unequal_to_their_parts() {
        let mut partition = oracle_partition(&HAND, 1, 0, 4096).unwrap();
        partition.contexts[0].followers += 1;
        assert!(refusal(1, &[partition]).contains("followers differ from n1 + n2 + n3+"));
    }

    #[test]
    fn ngram_count_uort_header_overflow_is_an_error() {
        let mut bytes = vec![0u8; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[8..16].copy_from_slice(&(1u64 << 63).to_le_bytes());
        assert!(read_uort_tokens(&bytes).is_err());
        bytes[8..16].copy_from_slice(&1u64.to_le_bytes());
        bytes.extend_from_slice(&7u16.to_le_bytes());
        assert_eq!(read_uort_tokens(&bytes).unwrap(), vec![7]);
    }

    #[test]
    fn ngram_count_device_bounds_refuse_past_the_u32_index_range() {
        let range = |targets| PartitionRange { lo: 0, hi: 1, targets };
        assert_eq!(partition_target_bound(&range(1 << 30)).unwrap(), 1 << 30);
        assert!(partition_target_bound(&range((1 << 30) + 1)).is_err());
        assert_eq!(stream_length_bound(1 << 31).unwrap(), 1 << 31);
        assert!(stream_length_bound((1 << 31) + 1).is_err());
        assert!(stream_length_bound(0).is_err());
    }

    type OrderStreams = (NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>);

    fn admitted_histogram(tokens: &[u16], order: u32) -> Vec<u64> {
        let offset = partition_offset(order);
        let mut histogram = vec![0u64; 4096];
        for j in order as usize..tokens.len() {
            if ngram_admitted(tokens, j, order as usize) {
                histogram[tokens[j - offset] as usize] += 1;
            }
        }
        histogram
    }

    /// The product path: plan order k from its own histogram, adjust each
    /// partition against order k+1 over the same range, assemble. Order 0
    /// sums the order-1 partitions of order 1's plan.
    fn adjusted_by_plan(tokens: &[u16], order: u32, highest: u32, budget: u64) -> Result<OrderStreams> {
        let mut assembler = NgramAssembler::new_adjusted(order, true)?;
        if order == 0 {
            let (_, unigrams, _) = oracle_count(tokens, 0)?;
            let mut continuation = UnigramContinuation::new();
            if highest >= 1 {
                for range in plan_partitions(&admitted_histogram(tokens, 1), budget)? {
                    continuation.add(&oracle_partition(tokens, 1, range.lo, range.hi)?)?;
                }
            }
            assembler.push(&continuation.adjust(highest, &unigrams)?)?;
        } else {
            for range in plan_partitions(&admitted_histogram(tokens, order), budget)? {
                let counted = oracle_partition(tokens, order, range.lo, range.hi)?;
                let extended = match order < highest {
                    true => Some(oracle_partition(tokens, order + 1, range.lo, range.hi)?),
                    false => None,
                };
                let adjusted =
                    adjust_partition(order, highest, range.lo, range.hi, &counted, extended.as_ref())?;
                assembler.push(&adjusted)?;
            }
        }
        Ok(assembler.finish())
    }

    /// Two documents (BOS = 0, EOS = 1), n = 19, N = 2:
    ///   0 5 6 7 5 6 8 3 6 7 1 | 0 5 6 7 4 6 7 1
    const TWO_DOCS: [u16; 19] = [0, 5, 6, 7, 5, 6, 8, 3, 6, 7, 1, 0, 5, 6, 7, 4, 6, 7, 1];

    fn kn_key(context: &[u16]) -> u128 {
        PrimeRouteKey::from_context(context).unwrap().word()
    }

    fn kn_triple(context: &[u16], next: u16, count: u32) -> NgramTriple {
        NgramTriple { key: kn_key(context), next, count }
    }

    fn kn_context(context: &[u16], census: [u32; 5]) -> NgramContext {
        let [total, followers, n1, n2, n3_plus] = census;
        NgramContext { key: kn_key(context), total: u64::from(total), followers, n1, n2, n3_plus }
    }

    /// Order 2 (k = N): a = c, the raw trigrams; three are seen twice
    /// (0 5 6, 5 6 7, 6 7 1) and nine once.
    /// Order 1 (N1+ of the order-2 left tokens):
    ///   a(0 5) = c = 2: the document-start gram (BOS oldest; no order-2 gram
    ///   extends it, since BOS may sit only in the oldest slot);
    ///   a(5 6) = |{0, 7}| = 2 while c(5 6) = 3 (0 5 6 twice, 7 5 6 once);
    ///   a(6 7) = |{5, 3, 4}| = 3 (c = 4); a(7 1) = |{6}| = 1 (c = 2);
    ///   a = c = 1 for the six bigrams seen once: 7 5, 6 8, 8 3, 3 6, 7 4, 4 6.
    /// Order 0 (N1+ of the left tokens of the counted bigrams):
    ///   1:{7} 3:{8} 4:{7} 5:{0,7} 6:{5,3,4} 7:{6} 8:{6} -> 1 1 1 2 3 1 1,
    ///   while the raw counts are 2 1 1 3 5 4 1.
    #[test]
    fn ngram_count_adjusted_hand_fixture_two_documents() {
        let (d0, t0, h0) = adjusted_by_plan(&TWO_DOCS, 0, 2, u64::MAX).unwrap();
        let unigram = |next, count| kn_triple(&[], next, count);
        assert_eq!(
            t0,
            vec![unigram(1, 1), unigram(3, 1), unigram(4, 1), unigram(5, 2), unigram(6, 3), unigram(7, 1), unigram(8, 1)]
        );
        assert_eq!(h0, vec![kn_context(&[], [10, 7, 5, 1, 1])]);
        assert_eq!((d0.targets, d0.count_of_counts), (10, [5, 1, 1, 0]));
        let (_, raw0, _) = oracle_count(&TWO_DOCS, 0).unwrap();
        assert_eq!(raw0.iter().map(|t| t.count).collect::<Vec<_>>(), vec![2, 1, 1, 3, 5, 4, 1]);

        let (d1, t1, h1) = adjusted_by_plan(&TWO_DOCS, 1, 2, u64::MAX).unwrap();
        assert_eq!(
            t1,
            vec![
                kn_triple(&[0], 5, 2),
                kn_triple(&[3], 6, 1),
                kn_triple(&[4], 6, 1),
                kn_triple(&[5], 6, 2),
                kn_triple(&[6], 7, 3),
                kn_triple(&[6], 8, 1),
                kn_triple(&[7], 1, 1),
                kn_triple(&[7], 4, 1),
                kn_triple(&[7], 5, 1),
                kn_triple(&[8], 3, 1),
            ]
        );
        assert_eq!(
            h1,
            vec![
                kn_context(&[0], [2, 1, 0, 1, 0]),
                kn_context(&[3], [1, 1, 1, 0, 0]),
                kn_context(&[4], [1, 1, 1, 0, 0]),
                kn_context(&[5], [2, 1, 0, 1, 0]),
                kn_context(&[6], [4, 2, 1, 0, 1]),
                kn_context(&[7], [3, 3, 3, 0, 0]),
                kn_context(&[8], [1, 1, 1, 0, 0]),
            ]
        );
        assert_eq!((d1.targets, d1.count_of_counts), (14, [7, 2, 1, 0]));
        let (_, raw1, _) = oracle_count(&TWO_DOCS, 1).unwrap();
        assert!(raw1.contains(&kn_triple(&[5], 6, 3)) && raw1.contains(&kn_triple(&[0], 5, 2)));

        let (d2, t2, _) = adjusted_by_plan(&TWO_DOCS, 2, 2, u64::MAX).unwrap();
        let (raw2_digest, raw2, _) = oracle_count(&TWO_DOCS, 2).unwrap();
        assert_eq!(t2, raw2);
        assert_eq!((d2.count_of_counts, raw2_digest.count_of_counts), ([9, 3, 0, 0], [9, 3, 0, 0]));

        for order in 0..=2 {
            let oracle = oracle_adjusted(&TWO_DOCS, order, 2).unwrap();
            assert_eq!(adjusted_by_plan(&TWO_DOCS, order, 2, 1).unwrap(), oracle, "order {order}");
        }
    }

    /// Skewed seeded stream starting with BOS, BOS planted about one position
    /// in twelve. Starting with BOS matters: the gram at j = k has no left
    /// token, so a stream whose first gram is unique and not BOS-led is refused.
    fn kn_stream(seed: u64, len: usize) -> Vec<u16> {
        let mut state = seed;
        let mut stream: Vec<u16> = (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let r = (state >> 20) as u32;
                (if r.is_multiple_of(4) { r % 64 } else { r % 9 }) as u16
            })
            .collect();
        stream[0] = NGRAM_BOS;
        stream
    }

    #[test]
    fn ngram_count_adjusted_assembler_matches_the_oracle_on_seeded_streams() {
        for seed in [0x418u64, 0x9e37_79b9, 0x5eed_c0de] {
            let stream = kn_stream(seed, 6_000);
            assert!(stream.iter().filter(|&&t| t == NGRAM_BOS).count() > 300);
            for order in 0..=4u32 {
                let oracle = oracle_adjusted(&stream, order, 4).unwrap();
                let oracle_digest = &oracle.0;
                let (raw, ..) = oracle_count(&stream, order).unwrap();
                if order < 4 {
                    assert!(oracle_digest.targets < raw.targets, "seed {seed:#x} order {order}");
                } else {
                    assert_eq!(oracle_digest.targets, raw.targets);
                }
                for budget in [u64::MAX, 700, 1] {
                    let assembled = adjusted_by_plan(&stream, order, 4, budget).unwrap();
                    assert_eq!(assembled, oracle, "seed {seed:#x} order {order} budget {budget}");
                }
            }
        }
    }

    #[test]
    fn ngram_count_adjusted_refuses_a_zero_adjusted_count_and_inconsistent_inputs() {
        let counted = oracle_partition(&TWO_DOCS, 1, 0, 4096).unwrap();
        let extended = oracle_partition(&TWO_DOCS, 2, 0, 4096).unwrap();
        adjust_partition(1, 2, 0, 4096, &counted, Some(&extended)).unwrap();
        let refuse = |order, lo, hi, counted: &NgramPartition, extended: Option<&NgramPartition>| {
            adjust_partition(order, 2, lo, hi, counted, extended).unwrap_err().to_string()
        };
        // Rule 4: 6 7 -> 5 is the only left extension of 7 -> 5; drop it.
        let mut tampered = extended.clone();
        let at = tampered.triples.iter().position(|t| *t == kn_triple(&[6, 7], 5, 1)).unwrap();
        tampered.triples.remove(at);
        let message = refuse(1, 0, 4096, &counted, Some(&tampered));
        assert!(message.contains("adjusted count 0 (rule 4)"), "{message}");
        // An order-2 triple whose suffix 9 -> 9 is not an order-1 gram.
        let mut stray = extended.clone();
        stray.triples.push(kn_triple(&[9, 9], 9, 1));
        assert!(refuse(1, 0, 4096, &counted, Some(&stray)).contains("is absent from the order-1"));
        // Mismatched order, a missing or extra order-(k+1) partition, a range.
        assert!(refuse(1, 0, 4096, &counted, Some(&counted)).contains("in an order-2 partition"));
        assert!(refuse(1, 0, 4096, &counted, None).contains("need the order-2 partition"));
        assert!(refuse(2, 0, 4096, &extended, Some(&extended)).contains("take no order-3"));
        assert!(refuse(1, 0, 5, &counted, Some(&extended)).contains("outside 0..5"));
        let low = oracle_partition(&TWO_DOCS, 1, 0, 5).unwrap();
        assert!(refuse(1, 0, 5, &low, Some(&extended)).contains("outside 0..5"));
        // Order 0, rule 4: 6 -> 8 is the only left extension of 8.
        let (_, unigrams, _) = oracle_count(&TWO_DOCS, 0).unwrap();
        let mut order1 = counted.clone();
        let at = order1.triples.iter().position(|t| *t == kn_triple(&[6], 8, 1)).unwrap();
        order1.triples.remove(at);
        let mut continuation = UnigramContinuation::new();
        continuation.add(&order1).unwrap();
        let message = continuation.adjust(2, &unigrams).unwrap_err().to_string();
        assert!(message.contains("target 8") && message.contains("(rule 4)"), "{message}");
        // Order 0: an order-1 target absent from the unigrams.
        let mut continuation = UnigramContinuation::new();
        continuation.add(&counted).unwrap();
        let missing: Vec<NgramTriple> = unigrams.iter().copied().filter(|t| t.next != 4).collect();
        assert!(continuation.adjust(2, &missing).unwrap_err().to_string().contains("absent"));
        // Review MEDIUM: a repeated order-2 triple would count 6 as a second
        // left token of 7 -> 5 (a = 2, not 1); it is refused instead.
        let mut repeated = extended.clone();
        let at = repeated.triples.iter().position(|t| *t == kn_triple(&[6, 7], 5, 1)).unwrap();
        repeated.triples.insert(at, kn_triple(&[6, 7], 5, 1));
        assert!(refuse(1, 0, 4096, &counted, Some(&repeated)).contains("not strictly ascending"));
        // Order 0: the same order-1 partition added twice is refused, and so
        // is an order-1 partition under N = 0.
        let mut continuation = UnigramContinuation::new();
        continuation.add(&counted).unwrap();
        assert!(continuation.add(&counted).unwrap_err().to_string().contains("not strictly ascending"));
        let message = continuation.adjust(0, &unigrams).unwrap_err().to_string();
        assert!(message.contains("N = 0 take no order-1 partition"), "{message}");
        // The oracle refuses a stream whose first gram is unique and not BOS-led.
        assert!(oracle_adjusted(&[9, 5, 6], 1, 2).is_err());
    }

    #[test]
    fn ngram_count_adjusted_sha_is_independent_of_the_budget() {
        let stream = kn_stream(0x418, 6_000);
        for order in 0..=4u32 {
            let plan_order = order.max(1);
            let mut partitions = Vec::new();
            let mut digests = Vec::new();
            for budget in [u64::MAX, 2_000, 300, 1] {
                let plan = plan_partitions(&admitted_histogram(&stream, plan_order), budget).unwrap();
                partitions.push(plan.len());
                let (digest, ..) = adjusted_by_plan(&stream, order, 4, budget).unwrap();
                digests.push((digest.triple_sha256, digest.context_sha256));
            }
            assert!(partitions.windows(2).all(|w| w[0] < w[1]), "order {order}: {partitions:?}");
            assert!(digests.iter().all(|d| *d == digests[0]), "order {order}");
            // Domain separation: at k = N, a = c, yet the adjusted SHA differs.
            let (raw, ..) = oracle_count(&stream, order).unwrap();
            let (adjusted_triples, _) = &digests[0];
            assert_ne!(*adjusted_triples, raw.triple_sha256);
        }
    }
}
