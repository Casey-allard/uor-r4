//! CUDA driver of the exact n-gram counter (claude-412). Loads the admitted
//! `ngram-count.cubin` (no PTX fallback, no NVRTC), keeps the token stream
//! resident, and counts one order at a time in key-range partitions that it
//! hands, in canonical order, to an [`NgramAssembler`].

use crate::ngram_count::{
    adjust_partition_from_runs, bucket_starts, check_tokens, extension_runs, partition_offset,
    partition_target_bound, plan_partitions,
    stream_length_bound, NgramAssembler, NgramContext, UnigramContinuation,
    NgramPartition, NgramTriple, PartitionRange, NGRAM_MAX_ORDER, NGRAM_VOCABULARY,
};
use crate::{Error, Result};
use cudarc::driver::{
    CudaContext, CudaFunction, CudaSlice, CudaStream, DeviceRepr, LaunchConfig, Ptx,
    PushKernelArg, ValidAsZeroBits,
};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

/// Integer nanosecond totals of the counter's phases since the last
/// [`NgramCounter::take_phases`] when profiling is enabled. These are wall
/// times around driver routines, including host work and device waits.
/// `own_count_ns` and
/// `extension_count_ns` split [`NgramCounter::count_partition`]'s wall time
/// in the adjusted counter between order k itself and the order k+1 it
/// counts for its extension runs; the six inner phases cover both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CountPhases {
    pub histogram_ns: u64,
    pub select_ns: u64,
    pub sort_ns: u64,
    pub emit_ns: u64,
    pub download_ns: u64,
    pub records_ns: u64,
    pub own_count_ns: u64,
    pub extension_count_ns: u64,
    pub runs_ns: u64,
    pub adjust_ns: u64,
    pub partitions: u64,
}

const PHASES: usize = 11;
const HISTOGRAM: usize = 0;
const SELECT: usize = 1;
const SORT: usize = 2;
const EMIT: usize = 3;
const DOWNLOAD: usize = 4;
const RECORDS: usize = 5;
const OWN_COUNT: usize = 6;
const EXTENSION_COUNT: usize = 7;
const RUNS: usize = 8;
const ADJUST: usize = 9;
const PARTITIONS: usize = 10;

fn nanos(since: Instant) -> u64 {
    u64::try_from(since.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

const SOURCE: &str = include_str!("ngram_count.cu");
const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/ngram-count.ptx"));
const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ngram-count.cubin"));
const TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));

/// The kernel inventory admitted by `build.rs`.
pub const NGRAM_COUNT_ENTRIES: [&str; 20] = [
    "ngram_token_hist",
    "ngram_token_hist_reduce",
    "ngram_select_flags",
    "ngram_select_scatter",
    "ngram_bucket_hist",
    "ngram_bucket_scatter",
    "ngram_slice_flags",
    "ngram_slice_scatter",
    "ngram_radix_hist",
    "ngram_radix_scatter",
    "ngram_scan32_reduce",
    "ngram_scan32_sums",
    "ngram_scan32_apply",
    "ngram_scan64_reduce",
    "ngram_scan64_sums",
    "ngram_scan64_apply",
    "ngram_boundaries",
    "ngram_run_starts",
    "ngram_emit_grams",
    "ngram_emit_contexts",
];
const RADIX_TILE: u32 = 4096;
const RADIX_WARPS: u32 = 8;
const SCAN_THREADS: u32 = 256;
const SCAN_BLOCK: u32 = 2048;
const THREADS: u32 = 256;
const HIST_BLOCKS: u32 = 256;
/// Targets examined per selection window: bounds the flag and scan buffers.
const SELECT_WINDOW: u32 = 1 << 26;
/// Positions per warp tile of the slot-1 bucket sort (`BUCKET_TILE` in the .cu).
const BUCKET_TILE: u32 = 1 << 20;

fn cuda(error: impl std::fmt::Display) -> Error {
    Error::Cuda(error.to_string())
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

fn grid(items: u32) -> LaunchConfig {
    LaunchConfig {
        grid_dim: (items.div_ceil(THREADS).max(1), 1, 1),
        block_dim: (THREADS, 1, 1),
        shared_mem_bytes: 0,
    }
}

/// Source, PTX and cubin SHA-256 of the embedded image.
pub fn compiled_ngram_count_image() -> [String; 3] {
    [
        format!("{:x}", Sha256::digest(SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(PTX.as_bytes())),
        format!("{:x}", Sha256::digest(CUBIN)),
    ]
}

/// claude-425: every position j in 1..n stably sorted by its slot-1 token
/// t[j-1], built once per counter on the first order k >= 1 selection.
/// Bucket v is `positions[starts[v]..starts[v + 1]]`, ascending in j.
struct Buckets {
    positions: CudaSlice<u32>,
    starts: Vec<u32>,
}

/// One CUDA context holding the resident token stream.
pub struct NgramCounter {
    _context: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    functions: Vec<CudaFunction>,
    tokens: CudaSlice<u16>,
    n: u32,
    clock: Option<[AtomicU64; PHASES]>,
    buckets: OnceLock<Buckets>,
}

impl NgramCounter {
    /// Validate the stream (12-bit ids, 1..=2^31 tokens) and upload it.
    pub fn new(ordinal: usize, tokens: &[u16]) -> Result<Self> {
        check_tokens(tokens)?;
        let n = stream_length_bound(tokens.len())?;
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        let (major, minor) = context.compute_capability().map_err(cuda)?;
        if TARGET.trim() != format!("sm_{major}{minor}") {
            return Err(invalid(format!(
                "ngram-count cubin target {} differs from device {major}.{minor}",
                TARGET.trim()
            )));
        }
        let stream = context.default_stream();
        let module = context
            .load_module(Ptx::from_binary(CUBIN.to_vec()))
            .map_err(cuda)?;
        let functions = NGRAM_COUNT_ENTRIES
            .iter()
            .map(|entry| module.load_function(entry).map_err(cuda))
            .collect::<Result<Vec<_>>>()?;
        let device_tokens = stream.clone_htod(tokens).map_err(cuda)?;
        Ok(Self {
            _context: context,
            stream,
            functions,
            tokens: device_tokens,
            n,
            clock: None,
            buckets: OnceLock::new(),
        })
    }

    /// Enable wall-clock phase accounting and its three extra stream fences per
    /// nonempty partition. Ordinary counters keep their original scheduling.
    pub fn enable_phase_timing(&mut self) {
        self.clock
            .get_or_insert_with(|| std::array::from_fn(|_| AtomicU64::new(0)));
    }

    fn start_phase(&self) -> Option<Instant> {
        self.clock.as_ref().map(|_| Instant::now())
    }

    fn tick(&self, phase: usize, since: Option<Instant>) {
        if let (Some(clock), Some(started)) = (&self.clock, since) {
            clock[phase].fetch_add(nanos(started), Ordering::Relaxed);
        }
    }

    fn finish_device_phase(&self, phase: usize, since: Option<Instant>) -> Result<()> {
        if since.is_some() {
            self.stream.synchronize().map_err(cuda)?;
            self.tick(phase, since);
        }
        Ok(())
    }

    /// Read and reset the totals. A counter without phase timing returns zeroes.
    pub fn take_phases(&self) -> CountPhases {
        let Some(clock) = &self.clock else {
            return CountPhases::default();
        };
        let take = |phase: usize| clock[phase].swap(0, Ordering::Relaxed);
        CountPhases {
            histogram_ns: take(HISTOGRAM),
            select_ns: take(SELECT),
            sort_ns: take(SORT),
            emit_ns: take(EMIT),
            download_ns: take(DOWNLOAD),
            records_ns: take(RECORDS),
            own_count_ns: take(OWN_COUNT),
            extension_count_ns: take(EXTENSION_COUNT),
            runs_ns: take(RUNS),
            adjust_ns: take(ADJUST),
            partitions: take(PARTITIONS),
        }
    }

    pub fn len(&self) -> u32 {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    fn function(&self, name: &str) -> &CudaFunction {
        let at = NGRAM_COUNT_ENTRIES
            .iter()
            .position(|entry| *entry == name)
            .expect("admitted entry");
        &self.functions[at]
    }

    fn zeros<T: DeviceRepr + ValidAsZeroBits>(&self, len: usize) -> Result<CudaSlice<T>> {
        self.stream.alloc_zeros::<T>(len.max(1)).map_err(cuda)
    }

    fn read_one<T: DeviceRepr + Clone>(&self, slice: &CudaSlice<T>, at: usize) -> Result<T> {
        let words = self.stream.clone_dtoh(&slice.slice(at..at + 1)).map_err(cuda)?;
        Ok(words[0].clone())
    }

    /// The histogram of the partition token over order k's admitted targets
    /// (the document-boundary rule of `ngram_admitted`).
    pub fn partition_histogram(&self, order: u32) -> Result<Vec<u64>> {
        if self.n <= order {
            return Ok(vec![0; NGRAM_VOCABULARY as usize]);
        }
        let mut block_hists = self.zeros::<u32>((HIST_BLOCKS * 4096) as usize)?;
        let mut hist = self.zeros::<u64>(4096)?;
        let (begin, end, blocks) = (order, self.n, HIST_BLOCKS);
        let off = partition_offset(order) as u32;
        let mut launch = self.stream.launch_builder(self.function("ngram_token_hist"));
        launch.arg(&self.tokens).arg(&begin).arg(&end).arg(&order).arg(&off).arg(&mut block_hists);
        // SAFETY: j - off and j - (k-1) >= 0 since j >= order >= off; block_hists HIST_BLOCKS x 4096.
        unsafe { launch.launch(LaunchConfig { grid_dim: (HIST_BLOCKS, 1, 1), block_dim: (THREADS, 1, 1), shared_mem_bytes: 0 }) }
            .map_err(cuda)?;
        let mut launch = self.stream.launch_builder(self.function("ngram_token_hist_reduce"));
        launch.arg(&block_hists).arg(&blocks).arg(&mut hist);
        // SAFETY: 4096 outputs, HIST_BLOCKS rows read.
        unsafe { launch.launch(grid(4096)) }.map_err(cuda)?;
        self.stream.clone_dtoh(&hist).map_err(cuda)
    }

    /// [`Self::partition_histogram`], timed as the histogram phase.
    fn timed_histogram(&self, order: u32) -> Result<Vec<u64>> {
        let started = self.start_phase();
        let histogram = self.partition_histogram(order)?;
        self.tick(HISTOGRAM, started);
        Ok(histogram)
    }

    /// The plan of key-range partitions for `order` under `budget` targets.
    pub fn plan(&self, order: u32, budget: u64) -> Result<Vec<PartitionRange>> {
        plan_partitions(&self.timed_histogram(order)?, budget)
    }

    /// Exclusive scan of `len` u32 words in place; `data` holds len + 1 words.
    fn scan32(&self, data: &mut CudaSlice<u32>, len: u32) -> Result<u32> {
        let blocks = len.div_ceil(SCAN_BLOCK).max(1);
        let mut sums = self.zeros::<u32>(blocks as usize)?;
        let mut total = self.zeros::<u32>(1)?;
        let config = LaunchConfig { grid_dim: (blocks, 1, 1), block_dim: (SCAN_THREADS, 1, 1), shared_mem_bytes: 0 };
        let one = LaunchConfig { grid_dim: (1, 1, 1), block_dim: (SCAN_THREADS, 1, 1), shared_mem_bytes: 0 };
        let mut launch = self.stream.launch_builder(self.function("ngram_scan32_reduce"));
        launch.arg(&*data).arg(&len).arg(&mut sums);
        // SAFETY: data holds len + 1 words, sums `blocks`.
        unsafe { launch.launch(config) }.map_err(cuda)?;
        let mut launch = self.stream.launch_builder(self.function("ngram_scan32_sums"));
        launch.arg(&mut sums).arg(&blocks).arg(&mut total);
        // SAFETY: one block over `blocks` sums.
        unsafe { launch.launch(one) }.map_err(cuda)?;
        let mut launch = self.stream.launch_builder(self.function("ngram_scan32_apply"));
        launch.arg(&mut *data).arg(&len).arg(&sums).arg(&total);
        // SAFETY: in-place apply reads each word before the same thread writes it.
        unsafe { launch.launch(config) }.map_err(cuda)?;
        self.read_one(&total, 0)
    }

    fn scan64(&self, data: &mut CudaSlice<u64>, len: u32) -> Result<u64> {
        let blocks = len.div_ceil(SCAN_BLOCK).max(1);
        let mut sums = self.zeros::<u64>(blocks as usize)?;
        let mut total = self.zeros::<u64>(1)?;
        let config = LaunchConfig { grid_dim: (blocks, 1, 1), block_dim: (SCAN_THREADS, 1, 1), shared_mem_bytes: 0 };
        let one = LaunchConfig { grid_dim: (1, 1, 1), block_dim: (SCAN_THREADS, 1, 1), shared_mem_bytes: 0 };
        let mut launch = self.stream.launch_builder(self.function("ngram_scan64_reduce"));
        launch.arg(&*data).arg(&len).arg(&mut sums);
        // SAFETY: data holds len + 1 words, sums `blocks`.
        unsafe { launch.launch(config) }.map_err(cuda)?;
        let mut launch = self.stream.launch_builder(self.function("ngram_scan64_sums"));
        launch.arg(&mut sums).arg(&blocks).arg(&mut total);
        // SAFETY: one block over `blocks` sums.
        unsafe { launch.launch(one) }.map_err(cuda)?;
        let mut launch = self.stream.launch_builder(self.function("ngram_scan64_apply"));
        launch.arg(&mut *data).arg(&len).arg(&sums).arg(&total);
        // SAFETY: in-place apply reads each word before the same thread writes it.
        unsafe { launch.launch(config) }.map_err(cuda)?;
        self.read_one(&total, 0)
    }

    /// The targets of `order` whose partition token is in lo..hi. Order 0
    /// scans the stream; every order k >= 1 compacts only the slot-1 buckets
    /// lo..hi, so its positions come bucket by bucket (ascending within each)
    /// rather than in stream order. The sorted result is the same: `sort` is a
    /// stable LSD sort whose key includes slot 1, so positions with equal keys
    /// share a bucket and keep their ascending order either way.
    fn select(&self, order: u32, range: &PartitionRange) -> Result<CudaSlice<u32>> {
        if partition_offset(order) == 0 {
            return self.select_stream(order, range);
        }
        self.select_slice(order, range, self.buckets()?)
    }

    /// The cached slot-1 buckets, sorted on the first call.
    fn buckets(&self) -> Result<&Buckets> {
        if let Some(buckets) = self.buckets.get() {
            return Ok(buckets);
        }
        let built = self.build_buckets()?;
        Ok(self.buckets.get_or_init(|| built))
    }

    /// Stable counting sort of positions 1..n by t[j-1]: bin-major per-tile
    /// histograms, one exclusive scan, a stable scatter.
    fn build_buckets(&self) -> Result<Buckets> {
        let positions = self.n.saturating_sub(1);
        let tiles = positions.div_ceil(BUCKET_TILE).max(1);
        let words = u64::from(NGRAM_VOCABULARY) * u64::from(tiles);
        let len = u32::try_from(words)
            .ok()
            .filter(|&len| len < u32::MAX)
            .ok_or_else(|| invalid("bucket histogram exceeds the u32 scan"))?;
        let mut hist = self.zeros::<u32>(len as usize + 1)?;
        let mut sorted = self.zeros::<u32>(positions as usize)?;
        let config = LaunchConfig { grid_dim: (tiles, 1, 1), block_dim: (32, 1, 1), shared_mem_bytes: 0 };
        if positions > 0 {
            let mut launch = self.stream.launch_builder(self.function("ngram_bucket_hist"));
            launch.arg(&self.tokens).arg(&self.n).arg(&tiles).arg(&mut hist);
            // SAFETY: one warp per tile; positions j in 1..n read t[j-1]; hist holds 4096 x tiles + 1 words.
            unsafe { launch.launch(config) }.map_err(cuda)?;
        }
        let total = self.scan32(&mut hist, len)?;
        if total != positions {
            return Err(invalid(format!("bucket sort placed {total} of {positions} positions")));
        }
        if positions > 0 {
            let mut launch = self.stream.launch_builder(self.function("ngram_bucket_scatter"));
            launch.arg(&self.tokens).arg(&self.n).arg(&tiles).arg(&hist).arg(&mut sorted);
            // SAFETY: the scanned offsets partition 0..positions; each slot is written once.
            unsafe { launch.launch(config) }.map_err(cuda)?;
        }
        let scanned = self.stream.clone_dtoh(&hist).map_err(cuda)?;
        let starts = bucket_starts(&scanned, tiles as usize, positions)?;
        Ok(Buckets { positions: sorted, starts })
    }

    /// Compact the order-k targets of buckets lo..hi; the count must equal
    /// the plan's.
    fn select_slice(&self, order: u32, range: &PartitionRange, buckets: &Buckets) -> Result<CudaSlice<u32>> {
        let size = u32::try_from(range.targets).map_err(|_| invalid("partition exceeds u32"))?;
        let mut pos = self.zeros::<u32>(size as usize)?;
        let (first, last) = (buckets.starts[range.lo as usize], buckets.starts[range.hi as usize]);
        let slice = last
            .checked_sub(first)
            .ok_or_else(|| invalid("bucket slice runs backwards"))?;
        let mut flags = self.zeros::<u32>(slice.min(SELECT_WINDOW) as usize + 1)?;
        let mut out_base = 0u32;
        let mut begin = first;
        while begin < last {
            let count = (last - begin).min(SELECT_WINDOW);
            let mut launch = self.stream.launch_builder(self.function("ngram_slice_flags"));
            launch.arg(&self.tokens).arg(&buckets.positions).arg(&begin).arg(&count).arg(&order).arg(&mut flags);
            // SAFETY: begin + count <= last <= positions; ngram_admitted reads only after j >= order.
            unsafe { launch.launch(grid(count)) }.map_err(cuda)?;
            let selected = self.scan32(&mut flags, count)?;
            // Checked before the scatter, so a miscounted slice never writes past `pos`.
            let next_base = out_base
                .checked_add(selected)
                .filter(|&total| total <= size)
                .ok_or_else(|| invalid("partition selection exceeded its planned size"))?;
            let mut launch = self.stream.launch_builder(self.function("ngram_slice_scatter"));
            launch.arg(&self.tokens).arg(&buckets.positions).arg(&begin).arg(&count).arg(&order).arg(&flags).arg(&out_base).arg(&mut pos);
            // SAFETY: out_base + scan < next_base <= size, checked above.
            unsafe { launch.launch(grid(count)) }.map_err(cuda)?;
            out_base = next_base;
            begin += count;
        }
        if out_base != size {
            return Err(invalid(format!("partition selected {out_base} targets, planned {size}")));
        }
        Ok(pos)
    }

    /// The targets of `order` whose partition token is in lo..hi, in stream
    /// order, from a scan of the whole stream: order 0's selection, and the
    /// reference the bucket slices are tested against.
    fn select_stream(&self, order: u32, range: &PartitionRange) -> Result<CudaSlice<u32>> {
        let size = u32::try_from(range.targets).map_err(|_| invalid("partition exceeds u32"))?;
        let mut pos = self.zeros::<u32>(size as usize)?;
        let off = partition_offset(order) as u32;
        let mut flags = self.zeros::<u32>(SELECT_WINDOW as usize + 1)?;
        let mut out_base = 0u32;
        let mut begin = order;
        while begin < self.n {
            let count = (self.n - begin).min(SELECT_WINDOW);
            let mut launch = self.stream.launch_builder(self.function("ngram_select_flags"));
            launch.arg(&self.tokens).arg(&begin).arg(&count).arg(&order).arg(&off).arg(&range.lo).arg(&range.hi).arg(&mut flags);
            // SAFETY: j - off >= 0 since begin >= order >= off; count flags written.
            unsafe { launch.launch(grid(count)) }.map_err(cuda)?;
            let selected = self.scan32(&mut flags, count)?;
            let mut launch = self.stream.launch_builder(self.function("ngram_select_scatter"));
            launch.arg(&self.tokens).arg(&begin).arg(&count).arg(&order).arg(&off).arg(&range.lo).arg(&range.hi).arg(&flags).arg(&out_base).arg(&mut pos);
            // SAFETY: out_base + scan < out_base + selected <= size, checked below.
            unsafe { launch.launch(grid(count)) }.map_err(cuda)?;
            out_base = out_base
                .checked_add(selected)
                .filter(|&total| total <= size)
                .ok_or_else(|| invalid("partition selection exceeded its planned size"))?;
            begin += count;
        }
        if out_base != size {
            return Err(invalid(format!("partition selected {out_base} targets, planned {size}")));
        }
        Ok(pos)
    }

    /// Stable LSD radix sort of positions by gram: next token first, then
    /// slot k .. slot 1, low 6 bits before high 6 bits of each.
    fn sort(&self, order: u32, mut pos: CudaSlice<u32>, n: u32) -> Result<CudaSlice<u32>> {
        let tiles = n.div_ceil(RADIX_TILE);
        let blocks = tiles.div_ceil(RADIX_WARPS);
        let config = LaunchConfig { grid_dim: (blocks, 1, 1), block_dim: (RADIX_WARPS * 32, 1, 1), shared_mem_bytes: 0 };
        let mut other = self.zeros::<u32>(n as usize)?;
        let mut hist = self.zeros::<u32>((64 * tiles + 1) as usize)?;
        let mut passes = Vec::new();
        for back in std::iter::once(0).chain((1..=order).rev()) {
            passes.push((back, 0u32));
            passes.push((back, 6u32));
        }
        for (back, shift) in passes {
            let mut launch = self.stream.launch_builder(self.function("ngram_radix_hist"));
            launch.arg(&self.tokens).arg(&pos).arg(&n).arg(&back).arg(&shift).arg(&tiles).arg(&mut hist);
            // SAFETY: hist holds 64 x tiles + 1 words; positions j >= back.
            unsafe { launch.launch(config) }.map_err(cuda)?;
            self.scan32(&mut hist, 64 * tiles)?;
            let mut launch = self.stream.launch_builder(self.function("ngram_radix_scatter"));
            launch.arg(&self.tokens).arg(&pos).arg(&mut other).arg(&n).arg(&back).arg(&shift).arg(&tiles).arg(&hist);
            // SAFETY: offsets partition 0..n; each slot is written once.
            unsafe { launch.launch(config) }.map_err(cuda)?;
            std::mem::swap(&mut pos, &mut other);
        }
        Ok(pos)
    }

    /// Count one partition on the device and copy out its canonical records.
    pub fn count_partition(&self, order: u32, range: &PartitionRange) -> Result<NgramPartition> {
        if order > NGRAM_MAX_ORDER || range.lo >= range.hi || range.hi > NGRAM_VOCABULARY {
            return Err(invalid("n-gram partition needs order <= 10 and 0 <= lo < hi <= 4096"));
        }
        let n = partition_target_bound(range)?;
        if n == 0 {
            return Ok(NgramPartition::default());
        }
        if let Some(clock) = &self.clock {
            clock[PARTITIONS].fetch_add(1, Ordering::Relaxed);
        }
        let started = self.start_phase();
        let selected = self.select(order, range)?;
        self.finish_device_phase(SELECT, started)?;
        let started = self.start_phase();
        let sorted = self.sort(order, selected, n)?;
        self.finish_device_phase(SORT, started)?;
        let started = self.start_phase();
        let mut gram_flag = self.zeros::<u32>(n as usize + 1)?;
        let mut context_flag = self.zeros::<u32>(n as usize)?;
        let mut launch = self.stream.launch_builder(self.function("ngram_boundaries"));
        launch.arg(&self.tokens).arg(&sorted).arg(&n).arg(&order).arg(&mut gram_flag).arg(&mut context_flag);
        // SAFETY: n flags each; sorted positions j >= order.
        unsafe { launch.launch(grid(n)) }.map_err(cuda)?;
        let flags = gram_flag.clone();
        let grams = self.scan32(&mut gram_flag, n)?;
        let mut gram_start = self.zeros::<u32>(grams as usize + 1)?;
        let mut launch = self.stream.launch_builder(self.function("ngram_run_starts"));
        launch.arg(&flags).arg(&gram_flag).arg(&n).arg(&mut gram_start);
        // SAFETY: index values are < grams; starts holds grams + 1 words.
        unsafe { launch.launch(grid(n)) }.map_err(cuda)?;
        drop(flags);
        drop(gram_flag);
        let mut keys = self.zeros::<u64>(2 * grams as usize)?;
        let mut next = self.zeros::<u16>(grams as usize)?;
        let mut counts = self.zeros::<u32>(grams as usize)?;
        let mut gram_context = self.zeros::<u32>(grams as usize + 1)?;
        let mut ind12 = self.zeros::<u64>(grams as usize + 1)?;
        let mut ind34 = self.zeros::<u64>(grams as usize + 1)?;
        let mut launch = self.stream.launch_builder(self.function("ngram_emit_grams"));
        launch
            .arg(&self.tokens).arg(&sorted).arg(&gram_start).arg(&context_flag).arg(&grams).arg(&order)
            .arg(&mut keys).arg(&mut next).arg(&mut counts).arg(&mut gram_context).arg(&mut ind12).arg(&mut ind34);
        // SAFETY: one thread per gram; every output holds at least `grams` words.
        unsafe { launch.launch(grid(grams)) }.map_err(cuda)?;
        drop(context_flag);
        drop(sorted);
        let context_flags = gram_context.clone();
        let contexts = self.scan32(&mut gram_context, grams)?;
        let mut context_start = self.zeros::<u32>(contexts as usize + 1)?;
        let mut launch = self.stream.launch_builder(self.function("ngram_run_starts"));
        launch.arg(&context_flags).arg(&gram_context).arg(&grams).arg(&mut context_start);
        // SAFETY: index values are < contexts; starts holds contexts + 1 words.
        unsafe { launch.launch(grid(grams)) }.map_err(cuda)?;
        drop(context_flags);
        let t12 = self.scan64(&mut ind12, grams)?;
        let t34 = self.scan64(&mut ind34, grams)?;
        let mut records = self.zeros::<u32>(4 * contexts as usize)?;
        let mut launch = self.stream.launch_builder(self.function("ngram_emit_contexts"));
        launch.arg(&context_start).arg(&gram_start).arg(&ind12).arg(&contexts).arg(&mut records);
        // SAFETY: one thread per context; records hold 4 x contexts words.
        unsafe { launch.launch(grid(contexts)) }.map_err(cuda)?;
        self.finish_device_phase(EMIT, started)?;
        let started = self.start_phase();
        let keys = self.stream.clone_dtoh(&keys).map_err(cuda)?;
        let next = self.stream.clone_dtoh(&next).map_err(cuda)?;
        let counts = self.stream.clone_dtoh(&counts).map_err(cuda)?;
        let records = self.stream.clone_dtoh(&records).map_err(cuda)?;
        self.tick(DOWNLOAD, started);
        let started = self.start_phase();
        let mut partition = NgramPartition {
            count_of_counts: [t12 & 0xffff_ffff, t12 >> 32, t34 & 0xffff_ffff, t34 >> 32],
            ..NgramPartition::default()
        };
        partition.triples = (0..grams as usize)
            .map(|g| NgramTriple {
                key: u128::from(keys[2 * g]) | (u128::from(keys[2 * g + 1]) << 64),
                next: next[g],
                count: counts[g],
            })
            .collect();
        // claude-422: the host copies of the gram words are freed before the
        // contexts are built; only `records` is read from here on.
        drop(keys);
        drop(next);
        drop(counts);
        partition.contexts = (0..contexts as usize)
            .map(|c| {
                let g0 = records[4 * c];
                let g1 = if c + 1 < contexts as usize { records[4 * c + 4] } else { grams };
                let followers = g1 - g0;
                let (n1, n2) = (records[4 * c + 2], records[4 * c + 3]);
                NgramContext {
                    key: partition.triples[g0 as usize].key,
                    total: u64::from(records[4 * c + 1]),
                    followers,
                    n1,
                    n2,
                    n3_plus: followers - n1 - n2,
                }
            })
            .collect();
        self.tick(RECORDS, started);
        Ok(partition)
    }

    /// Count `order` under `budget` targets per partition into `assembler`.
    /// Returns the plan used.
    pub fn count_order(
        &self,
        order: u32,
        budget: u64,
        assembler: &mut NgramAssembler,
    ) -> Result<Vec<PartitionRange>> {
        let plan = self.plan(order, budget)?;
        for range in &plan {
            assembler.push(&self.count_partition(order, range)?)?;
        }
        Ok(plan)
    }

    /// Count the modified Kneser-Ney adjusted counts of `order` under highest
    /// order `highest` (claude-418) into an assembler made by
    /// [`NgramAssembler::new_adjusted`]. For 1 <= k < N each partition of
    /// order k's plan is adjusted against order k+1 over the identical lo..hi;
    /// that range's target count comes from order k+1's own histogram, since
    /// `select` checks the exact planned count and order k+1 admits no more
    /// targets per bin than order k, so order k's budget bounds it. At k = N
    /// the counts are kept. Order 0 counts its whole stream, then adds every
    /// partition of order 1's plan once to the unigram continuation. Returns
    /// the plan whose partitions were counted: order max(k, 1)'s, or order
    /// 0's when N = 0.
    pub fn count_order_adjusted(
        &self,
        order: u32,
        highest: u32,
        budget: u64,
        assembler: &mut NgramAssembler,
    ) -> Result<Vec<PartitionRange>> {
        self.count_order_adjusted_with(order, highest, budget, &mut |p| assembler.push(p))
    }

    /// [`Self::count_order_adjusted`] with each adjusted partition handed, in
    /// canonical order, to `sink` instead of one assembler (claude-421): the
    /// caller can push it to an assembler and also to a
    /// [`crate::kn_filter::KnFilter`].
    pub fn count_order_adjusted_with(
        &self,
        order: u32,
        highest: u32,
        budget: u64,
        sink: &mut dyn FnMut(&NgramPartition) -> Result<()>,
    ) -> Result<Vec<PartitionRange>> {
        if highest > NGRAM_MAX_ORDER || order > highest {
            return Err(invalid(format!(
                "adjusted counts need order <= highest <= {NGRAM_MAX_ORDER}, got order {order} under N = {highest}"
            )));
        }
        if order == 0 {
            let mut raw = NgramAssembler::new(0, true)?;
            let started = self.start_phase();
            let unigram_plan = self.count_order(0, budget, &mut raw)?;
            self.tick(OWN_COUNT, started);
            let (_, unigrams, _) = raw.finish();
            let mut continuation = UnigramContinuation::new();
            let plan = if highest >= 1 {
                let plan = self.plan(1, budget)?;
                for range in &plan {
                    let started = self.start_phase();
                    let extended = self.count_partition(1, range)?;
                    self.tick(EXTENSION_COUNT, started);
                    continuation.add(&extended)?;
                }
                plan
            } else {
                unigram_plan
            };
            sink(&continuation.adjust(highest, &unigrams)?)?;
            return Ok(plan);
        }
        let plan = self.plan(order, budget)?;
        let extended_histogram = match order < highest {
            true => Some(self.timed_histogram(order + 1)?),
            false => None,
        };
        for range in &plan {
            // claude-422: order k+1 is counted first and reduced to its runs,
            // then freed, so it and order k are never held at once. The
            // adjusted partition equals `adjust_partition`'s.
            let runs = {
                let extended = match &extended_histogram {
                    Some(histogram) => {
                        let targets = histogram[range.lo as usize..range.hi as usize].iter().sum();
                        let same_tokens = PartitionRange { lo: range.lo, hi: range.hi, targets };
                        let started = self.start_phase();
                        let extended = self.count_partition(order + 1, &same_tokens)?;
                        self.tick(EXTENSION_COUNT, started);
                        Some(extended)
                    }
                    None => None,
                };
                let started = self.start_phase();
                let runs = extension_runs(order, highest, range.lo, range.hi, extended.as_ref())?;
                self.tick(RUNS, started);
                runs
            };
            let started = self.start_phase();
            let counted = self.count_partition(order, range)?;
            self.tick(OWN_COUNT, started);
            let started = self.start_phase();
            let adjusted =
                adjust_partition_from_runs(order, highest, range.lo, range.hi, &counted, &runs)?;
            self.tick(ADJUST, started);
            drop(runs);
            drop(counted);
            sink(&adjusted)?;
        }
        Ok(plan)
    }

    /// The adjusted counts of every order 0..=`highest` with each order
    /// counted once (claude-425). [`Self::count_order_adjusted_with`] counts
    /// order k+1 again for order k's runs; here one plan, cut from order 1's
    /// histogram, serves every k >= 1 (order k+1's bins never exceed order
    /// k's, since both key on t[j-1]), and per range the orders run from
    /// `highest` down to 1: order k's raw partition is adjusted against the
    /// runs of order k+1's raw partition over the same tokens, then reduced to
    /// order k-1's runs, or at k = 1 added to the unigram continuation. Order
    /// 0 is counted last on its own plan. `sink(k, partition)` receives each
    /// order's adjusted partitions in ascending key order (the ranges ascend
    /// in slot 1); calls of different orders interleave, so the caller keeps
    /// one assembler per order. Returns the shared plan (order 0's when N = 0).
    pub fn count_adjusted_sweep(
        &self,
        highest: u32,
        budget: u64,
        mut sink: impl FnMut(u32, &NgramPartition) -> Result<()>,
    ) -> Result<Vec<PartitionRange>> {
        self.sweep(highest, budget, &mut |_, _| {}, &mut sink)
    }

    /// [`Self::count_adjusted_sweep`] with `tamper(k, raw)` applied to each
    /// raw partition before it is adjusted or reduced to runs: the identity
    /// outside tests.
    fn sweep(
        &self,
        highest: u32,
        budget: u64,
        tamper: &mut dyn FnMut(u32, &mut NgramPartition),
        sink: &mut dyn FnMut(u32, &NgramPartition) -> Result<()>,
    ) -> Result<Vec<PartitionRange>> {
        if highest > NGRAM_MAX_ORDER {
            return Err(invalid(format!("adjusted counts need N <= {NGRAM_MAX_ORDER}, got {highest}")));
        }
        let mut continuation = UnigramContinuation::new();
        let mut plan = Vec::new();
        if highest >= 1 {
            let histograms = (1..=highest)
                .map(|order| self.timed_histogram(order))
                .collect::<Result<Vec<_>>>()?;
            plan = plan_partitions(&histograms[0], budget)?;
            for range in &plan {
                let mut runs = Vec::new();
                for order in (1..=highest).rev() {
                    let histogram = &histograms[order as usize - 1];
                    let targets = histogram[range.lo as usize..range.hi as usize].iter().sum();
                    let same_tokens = PartitionRange { lo: range.lo, hi: range.hi, targets };
                    let started = self.start_phase();
                    let mut counted = self.count_partition(order, &same_tokens)?;
                    self.tick(OWN_COUNT, started);
                    tamper(order, &mut counted);
                    let started = self.start_phase();
                    let adjusted =
                        adjust_partition_from_runs(order, highest, range.lo, range.hi, &counted, &runs)?;
                    self.tick(ADJUST, started);
                    drop(std::mem::take(&mut runs));
                    sink(order, &adjusted)?;
                    drop(adjusted);
                    if order == 1 {
                        continuation.add(&counted)?;
                    } else {
                        let started = self.start_phase();
                        runs = extension_runs(order - 1, highest, range.lo, range.hi, Some(&counted))?;
                        self.tick(RUNS, started);
                    }
                }
            }
        }
        let mut raw = NgramAssembler::new(0, true)?;
        let started = self.start_phase();
        let unigram_plan = self.count_order(0, budget, &mut raw)?;
        self.tick(OWN_COUNT, started);
        let (_, unigrams, _) = raw.finish();
        sink(0, &continuation.adjust(highest, &unigrams)?)?;
        Ok(if highest >= 1 { plan } else { unigram_plan })
    }
}

/// Test-only views of the selection (claude-425).
#[cfg(test)]
impl NgramCounter {
    /// The selected positions of one partition, from the whole-stream scan
    /// (`stream`) or from the path `count_partition` takes.
    pub(crate) fn selected_positions(&self, order: u32, range: &PartitionRange, stream: bool) -> Result<Vec<u32>> {
        let pos = match stream {
            true => self.select_stream(order, range)?,
            false => self.select(order, range)?,
        };
        let mut words = self.stream.clone_dtoh(&pos).map_err(cuda)?;
        words.truncate(range.targets as usize);
        Ok(words)
    }

    /// [`Self::count_adjusted_sweep`] with `tamper(k, raw)` applied to each
    /// raw order-k partition (k >= 1) before it is adjusted or handed on.
    pub(crate) fn count_adjusted_sweep_tampered(
        &self,
        highest: u32,
        budget: u64,
        tamper: &mut dyn FnMut(u32, &mut NgramPartition),
        sink: &mut dyn FnMut(u32, &NgramPartition) -> Result<()>,
    ) -> Result<Vec<PartitionRange>> {
        self.sweep(highest, budget, tamper, sink)
    }

    /// `positions` after the counter's gram sort of `order`.
    pub(crate) fn sorted_positions(&self, order: u32, positions: &[u32]) -> Result<Vec<u32>> {
        let n = u32::try_from(positions.len()).map_err(|_| invalid("positions exceed u32"))?;
        let pos = self.stream.clone_htod(positions).map_err(cuda)?;
        let sorted = self.sort(order, pos, n)?;
        self.stream.clone_dtoh(&sorted.slice(0..positions.len())).map_err(cuda)
    }

    /// Plant a bucket-offset miscount: move bucket `bin`'s start `down`
    /// positions earlier, into bucket bin-1, after building the buckets.
    pub(crate) fn plant_bucket_start_shift(&mut self, bin: usize, down: u32) -> Result<()> {
        self.buckets()?;
        let buckets = self.buckets.get_mut().expect("buckets were just built");
        buckets.starts[bin] = buckets.starts[bin]
            .checked_sub(down)
            .ok_or_else(|| invalid("planted shift runs below zero"))?;
        Ok(())
    }
}
