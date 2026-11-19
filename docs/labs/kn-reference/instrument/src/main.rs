//! `kn-reference` (claude-421): the bits instrument of the sealed reference R*.
//!
//! It counts TRAIN's modified Kneser-Ney adjusted orders 0..=N on the GPU, keeps only the
//! records the query's targets read (`KnQuery`), scores the query on the GPU (`KnScorer`,
//! equal to the host `KnTables`), takes `lawful_likelihood::bits_q12` of every Q48 mass, and
//! sums bits and bytes over all targets and over the novel class. Everything is an integer.
//!
//! Usage (needs `--features cuda`):
//! `kn-reference TRAIN.u16 QUERY.u16 ID-BYTES.u8 CLASSES.u8 N BUDGET OUT.json
//!  [--per-mille P] [--check-host] [--profile-phases] [--extension-census] [--count-only]
//!  [--ceiling]`
//! `--ceiling` (claude-433) also counts raw orders 0..N, looks the query up in them and
//! scores the expert-mixture ceiling modes; it refuses `--count-only`, `--extension-census`
//! and `--sealed-run`.
//! claude-425: every order is counted once, by `NgramCounter::count_adjusted_sweep`.
//! `--count-only` writes each order's records and discounts to OUT.json and stops before the
//! scorer; it refuses `--check-host` and `--sealed-run`. `--extension-census` implies it.
//! `--per-mille P` keeps the first P per mille of TRAIN and of the query, each cut at a record
//! start. `--profile-phases` adds synchronized wall-time diagnostics on stderr.
//! `--check-host` also scores on the host and refuses at the first mass that differs.
//! `--extension-census` (claude-424) tallies, for each order k < N, the order-k+1 grams whose
//! suffix the query's order-k keys read, writes that census to OUT.json and stops after the
//! count: nothing is scored. It refuses `--check-host` and `--sealed-run`.

// Without `cuda` only the argument parser runs; the tally is still built and tested.
#![cfg_attr(not(feature = "cuda"), allow(dead_code))]

use sha2::{Digest, Sha256};
use uor_r4_discrete::kn_score::PROBABILITY_ONE as KN_ONE;
use uor_r4_discrete::ngram_count::NGRAM_BOS;
use uor_r4_integer::lawful_likelihood::{bits_q12, LOG2_SCALE, PROBABILITY_ONE as BITS_ONE};

// The scorer's masses and `bits_q12` must share one unit: 2^48.
const _: () = assert!(KN_ONE == BITS_ONE && KN_ONE == 1 << 48);

/// The novel class of claude-420's `S.novel.u8`.
const CLASS_NOVEL: u8 = 1;
/// The BOS class: never scored.
const CLASS_BOS: u8 = 3;
/// The record end id: a whole-record prefix ends on it.
const EOS: u16 = 1;

/// The sha256 of claude-420's sealed `dev-test.tokens.u16`. It is scored only with
/// `--sealed-run`, which claude-422 passes after its sheet and FIT.
const DEV_TEST_TOKENS_SHA256: &str =
    "023784b74c98a796401a064d50bf2f83f2f8eb95ab8bbead1cbe2aec6e7a176b";

/// Refuses the sealed DEV-test stream unless the run is the sealed run.
fn seal_check(query_sha256: &str, sealed_run: bool) -> Result<(), Failure> {
    if query_sha256 == DEV_TEST_TOKENS_SHA256 && !sealed_run {
        return Err(fail(
            "the query is the sealed DEV-test stream; only the sealed run (--sealed-run) scores it",
        ));
    }
    Ok(())
}

type Failure = Box<dyn std::error::Error>;

fn fail(message: impl Into<String>) -> Failure {
    message.into().into()
}

/// The length of the prefix holding the first `per_mille` per mille of `tokens`, cut at a
/// record start: the last BOS at or before the mark, so the prefix ends before it. A prefix
/// with no whole record, an empty stream, and a cut that does not follow an EOS are refused.
fn record_prefix(tokens: &[u16], per_mille: u64) -> Result<usize, Failure> {
    if per_mille == 0 || per_mille > 1000 {
        return Err(fail(format!("--per-mille takes 1..=1000, got {per_mille}")));
    }
    if tokens.is_empty() {
        return Err(fail("the stream is empty"));
    }
    if per_mille == 1000 {
        return Ok(tokens.len());
    }
    let mark = (tokens.len() as u128 * u128::from(per_mille) / 1000) as usize;
    let cut = tokens[..=mark.min(tokens.len() - 1)]
        .iter()
        .rposition(|&t| t == NGRAM_BOS)
        .unwrap_or(0);
    if cut == 0 {
        return Err(fail(format!(
            "the first {per_mille} per mille holds no whole record"
        )));
    }
    if tokens[cut - 1] != EOS {
        return Err(fail(format!(
            "the record before position {cut} does not end on EOS"
        )));
    }
    Ok(cut)
}

/// The classes of the query prefix. The class file must cover the whole query stream, so a
/// class file made for another stream that only agrees on the prefix is refused.
fn classes_prefix(classes: &[u8], stream_len: usize, prefix: usize) -> Result<&[u8], Failure> {
    if classes.len() != stream_len {
        return Err(fail(format!(
            "the class file holds {} positions, the query stream {stream_len}",
            classes.len()
        )));
    }
    Ok(&classes[..prefix])
}

/// Bits and bytes over a set of targets.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Tally {
    targets: u64,
    bits_q12: u128,
    bytes: u64,
}

impl Tally {
    fn add(&mut self, bits: u64, bytes: u8) {
        self.targets += 1;
        self.bits_q12 += u128::from(bits);
        self.bytes += u64::from(bytes);
    }

    /// Bits per byte in micro-bits, floored: bits_q12 * 10^6 / (4096 * bytes).
    fn bpb_micro(&self) -> Option<u128> {
        let denominator = u128::from(LOG2_SCALE) * u128::from(self.bytes);
        (denominator > 0).then(|| self.bits_q12 * 1_000_000 / denominator)
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "targets": self.targets,
            "bits_q12": self.bits_q12.to_string(),
            "bytes": self.bytes,
            "bpb_micro": self.bpb_micro().map(|v| v.to_string()),
        })
    }
}

/// Sums `bits_q12` of each mass and the bytes of its target, over every non-BOS target of
/// `query` (`masses` in stream order) and over the novel class. Refuses a length mismatch, a
/// class file that disagrees with the stream on BOS, and a mass `bits_q12` refuses.
fn tally(
    query: &[u16],
    masses: &[u64],
    id_bytes: &[u8],
    classes: &[u8],
) -> Result<(Tally, Tally), Failure> {
    if classes.len() != query.len() {
        return Err(fail(format!(
            "classes hold {} positions, the query {}",
            classes.len(),
            query.len()
        )));
    }
    let mut all = Tally::default();
    let mut novel = Tally::default();
    let mut next = masses.iter();
    for (j, (&token, &class)) in query.iter().zip(classes).enumerate() {
        if class > CLASS_BOS {
            return Err(fail(format!("position {j}: class {class} is not 0..=3")));
        }
        if (token == NGRAM_BOS) != (class == CLASS_BOS) {
            return Err(fail(format!(
                "position {j}: token {token} has class {class}"
            )));
        }
        if token == NGRAM_BOS {
            continue;
        }
        let mass = *next
            .next()
            .ok_or_else(|| fail(format!("no mass for target {j}")))?;
        let bits = bits_q12(mass).map_err(|e| fail(format!("target {j}: {e:?}")))?;
        let bytes = id_bytes[usize::from(token)];
        all.add(bits, bytes);
        if class == CLASS_NOVEL {
            novel.add(bits, bytes);
        }
    }
    if next.next().is_some() {
        return Err(fail("more masses than scored targets"));
    }
    Ok((all, novel))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("kn-reference: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

struct Args {
    paths: [String; 4],
    highest: u32,
    budget: u64,
    out: String,
    per_mille: u64,
    check_host: bool,
    profile_phases: bool,
    sealed_run: bool,
    extension_census: bool,
    count_only: bool,
    ceiling: bool,
}

fn parse(args: &[String]) -> Result<Args, Failure> {
    let usage = "usage: kn-reference TRAIN.u16 QUERY.u16 ID-BYTES.u8 CLASSES.u8 N BUDGET OUT.json [--per-mille P] [--check-host] [--sealed-run] [--profile-phases] [--extension-census] [--count-only] [--ceiling]";
    let mut positional = Vec::new();
    let mut per_mille = 1000;
    let mut check_host = false;
    let mut profile_phases = false;
    let mut sealed_run = false;
    let mut extension_census = false;
    let mut count_only = false;
    let mut ceiling = false;
    let mut at = 0;
    while at < args.len() {
        match args[at].as_str() {
            "--per-mille" => {
                at += 1;
                per_mille = args.get(at).ok_or_else(|| fail(usage))?.parse()?;
            }
            "--check-host" => check_host = true,
            "--profile-phases" => profile_phases = true,
            "--sealed-run" => sealed_run = true,
            "--extension-census" => extension_census = true,
            "--count-only" => count_only = true,
            "--ceiling" => ceiling = true,
            other => positional.push(other.to_string()),
        }
        at += 1;
    }
    // claude-425: the extension census is a count-only run.
    count_only |= extension_census;
    if count_only && (check_host || sealed_run) {
        return Err(fail(
            "--count-only and --extension-census score nothing: they refuse --check-host and --sealed-run",
        ));
    }
    // claude-433: the ceiling scores DEV only; the sealed DEV-test stays sealed.
    if ceiling && (count_only || sealed_run) {
        return Err(fail(
            "--ceiling scores every mode: it refuses --count-only, --extension-census and --sealed-run",
        ));
    }
    let [train, query, id_bytes, classes, highest, budget, out]: [String; 7] =
        positional.try_into().map_err(|_| fail(usage))?;
    Ok(Args {
        paths: [train, query, id_bytes, classes],
        highest: highest.parse()?,
        budget: budget.parse()?,
        out,
        per_mille,
        check_host,
        profile_phases,
        sealed_run,
        extension_census,
        count_only,
        ceiling,
    })
}

#[cfg(not(feature = "cuda"))]
fn run(args: &[String]) -> Result<(), Failure> {
    parse(args)?;
    Err(fail(
        "kn-reference counts and scores on the GPU: build with --features cuda",
    ))
}

#[cfg(feature = "cuda")]
fn run(args: &[String]) -> Result<(), Failure> {
    use uor_r4_discrete::kn_filter::{KnExtensionCensus, KnQuery};
    use uor_r4_discrete::kn_score::KnTables;
    use uor_r4_discrete::kn_score_device::KnScorer;
    use uor_r4_discrete::ngram_count::{
        read_uort_tokens, AssemblerTiming, NgramAssembler, NgramContext, NgramOrderDigest,
        NgramTriple,
    };
    use uor_r4_discrete::ngram_count_device::NgramCounter;

    let args = parse(args)?;
    let [train_path, query_path, id_bytes_path, classes_path] = &args.paths;
    let started = std::time::Instant::now();
    let train_bytes = std::fs::read(train_path)?;
    let query_bytes = std::fs::read(query_path)?;
    let query_sha256 = sha256_hex(&query_bytes);
    seal_check(&query_sha256, args.sealed_run)?;
    let id_bytes = std::fs::read(id_bytes_path)?;
    let classes_bytes = std::fs::read(classes_path)?;
    if id_bytes.len() != 4096 {
        return Err(fail(format!(
            "id-bytes holds {} entries, not 4096",
            id_bytes.len()
        )));
    }
    let train_all = read_uort_tokens(&train_bytes)?;
    // claude-422: TRAIN's file bytes are needed only for their hash. Freed
    // here, they no longer hold 4 GB of host memory through the counting.
    let train_sha256 = sha256_hex(&train_bytes);
    drop(train_bytes);
    let query_all = read_uort_tokens(&query_bytes)?;
    let train = &train_all[..record_prefix(&train_all, args.per_mille)?];
    let query_len = record_prefix(&query_all, args.per_mille)?;
    let query = &query_all[..query_len];
    let classes = classes_prefix(&classes_bytes, query_all.len(), query_len)?;
    eprintln!(
        "read: train {} of {} tokens, query {} of {}, {} ms",
        train.len(),
        train_all.len(),
        query.len(),
        query_all.len(),
        started.elapsed().as_millis()
    );

    let wanted = KnQuery::from_stream(query, args.highest)?;
    let mut orders: Vec<(NgramOrderDigest, _, _)> = Vec::new();
    let mut order_json = Vec::new();
    let train_tokens = train.len();
    let mut counter = NgramCounter::new(0, train)?;
    if args.profile_phases {
        counter.enable_phase_timing();
    }
    // claude-422: the counter holds TRAIN on the device and the host copy is
    // never read again, so its 4 GB are freed before the counting.
    drop(train_all);
    // claude-425: one sweep counts every order once. Its sink calls for
    // different orders interleave, so each order keeps its own assembler,
    // filter and sink timings; each order's partitions still ascend.
    let width = args.highest as usize + 1;
    let mut assemblers = (0..=args.highest)
        .map(|k| NgramAssembler::new_adjusted(k, false))
        .collect::<Result<Vec<_>, _>>()?;
    // claude-427: split each push into its sub-phases.
    if args.profile_phases {
        assemblers
            .iter_mut()
            .for_each(NgramAssembler::enable_timing);
    }
    let mut filters = (0..=args.highest)
        .map(|k| wanted.filter(k))
        .collect::<Result<Vec<_>, _>>()?;
    // claude-424: census k is fed by order k+1's partitions; there is none
    // for the top order.
    let mut censuses: Vec<KnExtensionCensus> = if args.extension_census {
        (0..args.highest)
            .map(|k| wanted.extension_census(k))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    // The partitions each order was handed, and claude-423's sink host time.
    let mut handed = vec![0usize; width];
    let mut push_ns = vec![0u64; width];
    let mut keep_ns = vec![0u64; width];
    let nanos =
        |since: std::time::Instant| u64::try_from(since.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let start_phase = || args.profile_phases.then(std::time::Instant::now);
    let sweep_started = start_phase();
    let shared = counter.count_adjusted_sweep(args.highest, args.budget, |order, partition| {
        let k = order as usize;
        let (Some(assembler), Some(filter)) = (assemblers.get_mut(k), filters.get_mut(k)) else {
            return Err(uor_r4_discrete::Error::Invalid(format!(
                "the sweep handed order {order} above N = {}",
                args.highest
            )));
        };
        handed[k] += 1;
        // claude-428: the filter and the census borrow the partition, then
        // the assembler takes it for its hash workers. Their results are held
        // so a refusal keeps its old precedence: push, census, keep.
        let at = start_phase();
        let kept = filter.keep(&partition);
        if let Some(at) = at {
            keep_ns[k] += nanos(at);
        }
        let observed = match k.checked_sub(1).and_then(|below| censuses.get_mut(below)) {
            Some(census) => census.observe(&partition),
            None => Ok(()),
        };
        let at = start_phase();
        assembler.push_owned(partition)?;
        if let Some(at) = at {
            push_ns[k] += nanos(at);
        }
        observed?;
        kept
    })?;
    eprintln!("sweep: {} shared ranges for orders 1..={}", shared.len(), args.highest);
    if let Some(sweep_started) = sweep_started {
        let sweep_ns = nanos(sweep_started);
        let p = counter.take_phases();
        let ms = |ns: u64| ns / 1_000_000;
        eprintln!(
            "sweep phases ms: wall {} | own count {} extension count {} | histogram {} select {} sort {} emit {} download {} records {} | runs {} adjust {} | push {} keep {} | {} partitions counted",
            ms(sweep_ns),
            ms(p.own_count_ns),
            ms(p.extension_count_ns),
            ms(p.histogram_ns),
            ms(p.select_ns),
            ms(p.sort_ns),
            ms(p.emit_ns),
            ms(p.download_ns),
            ms(p.records_ns),
            ms(p.runs_ns),
            ms(p.adjust_ns),
            ms(push_ns.iter().sum()),
            ms(keep_ns.iter().sum()),
            p.partitions
        );
        for k in 0..width {
            eprintln!("order {k} sink ms: push {} keep {}", ms(push_ns[k]), ms(keep_ns[k]));
        }
    }
    // claude-433: the ceiling's RAW counts of orders 0..N-1, kept to the
    // query's keys, before the counter (the only copy of TRAIN) is dropped.
    // Order N's raw table is the KN table's own and is added after the scorer.
    let raw_started = std::time::Instant::now();
    let mut raw_orders: Vec<(NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>)> = Vec::new();
    if args.ceiling {
        for k in 0..args.highest {
            let mut assembler = NgramAssembler::new(k, false)?;
            let mut filter = wanted.filter(k)?;
            for range in counter.plan(k, args.budget)? {
                let partition = counter.count_partition(k, &range)?;
                filter.keep(&partition)?;
                assembler.push_owned(partition)?;
            }
            let (digest, _, _) = assembler.finish();
            let (triples, contexts) = filter.finish();
            eprintln!(
                "raw order {k}: grams {} contexts {}, kept triples {} contexts {}, {} ms",
                digest.grams,
                digest.contexts,
                triples.len(),
                contexts.len(),
                started.elapsed().as_millis()
            );
            raw_orders.push((digest, triples, contexts));
        }
    }
    let raw_count_ms = raw_started.elapsed().as_millis();
    drop(counter);
    let mut census_json = Vec::new();
    for (k, census) in censuses.into_iter().enumerate() {
        let tally = census.finish();
        eprintln!(
            "order {k} extensions: order-{} grams {}, context extensions {}, pair extensions {}",
            k + 1,
            tally.grams,
            tally.context_extensions,
            tally.pair_extensions
        );
        census_json.push(serde_json::json!({
            "order": k,
            "extension_grams": tally.grams,
            "context_extensions": tally.context_extensions,
            "pair_extensions": tally.pair_extensions,
        }));
    }
    let mut push_parts = AssemblerTiming::default();
    let ms = |ns: u64| ns / 1_000_000;
    let mut top_digest: Option<NgramOrderDigest> = None;
    for (k, (assembler, filter)) in assemblers.into_iter().zip(filters).enumerate() {
        let (digest, _, _, timing) = assembler.finish_with_timing();
        if args.profile_phases {
            eprintln!(
                "order {k} push parts ms: triple checks {} context checks {} census {} triple feed {} (send wait {}) context feed {} (send wait {}) | hasher busy triple {} context {}",
                ms(timing.triple_checks_ns),
                ms(timing.context_checks_ns),
                ms(timing.census_ns),
                ms(timing.triple_feed_ns),
                ms(timing.triple_send_wait_ns),
                ms(timing.context_feed_ns),
                ms(timing.context_send_wait_ns),
                ms(timing.triple_hash_busy_ns),
                ms(timing.context_hash_busy_ns)
            );
            push_parts.triple_checks_ns += timing.triple_checks_ns;
            push_parts.context_checks_ns += timing.context_checks_ns;
            push_parts.census_ns += timing.census_ns;
            push_parts.triple_feed_ns += timing.triple_feed_ns;
            push_parts.triple_send_wait_ns += timing.triple_send_wait_ns;
            push_parts.context_feed_ns += timing.context_feed_ns;
            push_parts.context_send_wait_ns += timing.context_send_wait_ns;
            push_parts.triple_hash_busy_ns += timing.triple_hash_busy_ns;
            push_parts.context_hash_busy_ns += timing.context_hash_busy_ns;
        }
        let (triples, contexts) = filter.finish();
        eprintln!(
            "order {k}: {} partitions, grams {} contexts {}, kept triples {} contexts {}, {} ms",
            handed[k],
            digest.grams,
            digest.contexts,
            triples.len(),
            contexts.len(),
            started.elapsed().as_millis()
        );
        order_json.push(serde_json::json!({
            "order": digest.order,
            "targets": digest.targets,
            "grams": digest.grams,
            "contexts": digest.contexts,
            "count_of_counts": digest.count_of_counts,
            "triple_sha256": digest.triple_sha256,
            "context_sha256": digest.context_sha256,
            "partitions": handed[k],
            "kept_triples": triples.len(),
            "kept_contexts": contexts.len(),
        }));
        if args.ceiling && k as u32 == args.highest {
            top_digest = Some(digest.clone());
        }
        orders.push((digest, triples, contexts));
    }
    if args.profile_phases {
        let p = push_parts;
        eprintln!(
            "push parts ms, all orders: triple checks {} context checks {} census {} triple feed {} (send wait {}) context feed {} (send wait {}) | hasher busy triple {} context {}",
            ms(p.triple_checks_ns),
            ms(p.context_checks_ns),
            ms(p.census_ns),
            ms(p.triple_feed_ns),
            ms(p.triple_send_wait_ns),
            ms(p.context_feed_ns),
            ms(p.context_send_wait_ns),
            ms(p.triple_hash_busy_ns),
            ms(p.context_hash_busy_ns)
        );
    }
    let tables = KnTables::new(args.highest, orders)?;
    for (k, order) in order_json.iter_mut().enumerate() {
        let k = k as u32;
        let (discounts, _, _) = tables
            .order_tables(k)
            .ok_or_else(|| fail(format!("no order-{k} table")))?;
        let fallback = tables.discount_fallback(k) == Some(true);
        order["discounts_q48"] = serde_json::json!(discounts.dq);
        order["discount_fallback"] = serde_json::json!(fallback);
        if fallback {
            eprintln!("order {k}: KenLM fallback discounts (1/2, 1, 3/2)");
        }
    }
    // claude-425: a count-only run stops here, with the discounts but before
    // the scorer reads a single target; the extension census is count-only.
    if args.count_only {
        let mut summary = serde_json::json!({
            "unit": "claude-431",
            "mode": if args.extension_census { "extension-census" } else { "count-only" },
            "highest_order": args.highest,
            "budget": args.budget,
            "per_mille": args.per_mille,
            "shared_ranges": shared.len(),
            "inputs": {
                "train_sha256": train_sha256,
                "query_sha256": query_sha256,
                "train_tokens": train_tokens,
                "query_tokens": query.len(),
            },
            "orders": order_json,
        });
        if args.extension_census {
            summary["extensions"] = serde_json::json!(census_json);
        }
        std::fs::write(&args.out, serde_json::to_string_pretty(&summary)? + "\n")?;
        eprintln!(
            "wrote the counts {}, {} ms; nothing was scored",
            args.out,
            started.elapsed().as_millis()
        );
        return Ok(());
    }
    let masses = KnScorer::new(0, &tables)?.score(query)?;
    eprintln!(
        "gpu scored {} targets, {} ms",
        masses.len(),
        started.elapsed().as_millis()
    );
    if args.check_host {
        let host = tables.scored_masses(query)?;
        if host.len() != masses.len() {
            return Err(fail(format!(
                "host scored {} targets, GPU {}",
                host.len(),
                masses.len()
            )));
        }
        if let Some(i) = (0..host.len()).find(|&i| host[i] != masses[i]) {
            return Err(fail(format!(
                "scored target {i}: GPU mass {} differs from host {}",
                masses[i], host[i]
            )));
        }
        eprintln!(
            "host = GPU at all {} targets, {} ms",
            host.len(),
            started.elapsed().as_millis()
        );
    }
    let (all, novel) = tally(query, &masses, &id_bytes, classes)?;
    // claude-422: a mass of one Q48 unit is the scorer's floor under an
    // underflowed gamma chain; the record counts the targets that sit on it.
    let masses_at_floor = masses.iter().filter(|&&m| m == 1).count();
    eprintln!("masses at the one-unit floor: {masses_at_floor}");
    let mass_bytes: Vec<u8> = masses.iter().flat_map(|m| m.to_le_bytes()).collect();
    // claude-433: the ceiling modes, from order N's raw table (the KN table's
    // own), the GPU lookups and the host EM over them.
    let ceiling_json = if args.ceiling {
        use uor_r4_discrete::kn_ceiling::RawTables;
        use uor_r4_discrete::kn_lookup_device::KnLookup;
        let digest = top_digest.ok_or_else(|| fail("no order-N digest for the ceiling"))?;
        let (_, top_triples, top_contexts) = tables
            .order_tables(args.highest)
            .ok_or_else(|| fail(format!("no order-{} table", args.highest)))?;
        raw_orders.push((digest, top_triples.to_vec(), top_contexts.to_vec()));
        let raw = RawTables::new(args.highest, raw_orders)?;
        let lookup_started = std::time::Instant::now();
        let (counts, totals) = KnLookup::new(0, &raw)?.lookups(query)?;
        let lookup_ms = lookup_started.elapsed().as_millis();
        eprintln!(
            "gpu looked up {} targets, {} ms",
            masses.len(),
            started.elapsed().as_millis()
        );
        if args.check_host {
            let (host_counts, host_totals) = raw.lookups(query)?;
            let width = args.highest as usize + 1;
            let differs = |i: usize| {
                host_counts.get(i) != counts.get(i) || host_totals.get(i) != totals.get(i)
            };
            let length = host_counts
                .len()
                .max(counts.len())
                .max(host_totals.len())
                .max(totals.len());
            if let Some(i) = (0..length).find(|&i| differs(i)) {
                return Err(fail(format!(
                    "lookup of target {} order {}: GPU differs from host",
                    i / width,
                    i % width
                )));
            }
            eprintln!("host lookups = GPU at all {} targets", masses.len());
        }
        let em_started = std::time::Instant::now();
        let mut json = ceiling_modes(
            args.highest,
            query,
            &masses,
            &counts,
            &totals,
            &id_bytes,
            classes,
        )?;
        let em_ms = em_started.elapsed().as_millis();
        if args.profile_phases {
            eprintln!("ceiling phases ms: raw-count {raw_count_ms} lookup {lookup_ms} em {em_ms}");
            json["phases_ms"] = serde_json::json!({
                "raw_count": raw_count_ms.to_string(),
                "lookup": lookup_ms.to_string(),
                "em": em_ms.to_string(),
            });
        }
        Some(json)
    } else {
        None
    };
    let mut summary = serde_json::json!({
        "unit": if args.ceiling { "claude-433" } else { "claude-421" },
        "highest_order": args.highest,
        "budget": args.budget,
        "per_mille": args.per_mille,
        "check_host": args.check_host,
        "sealed_run": args.sealed_run,
        "inputs": {
            "train_sha256": train_sha256,
            "query_sha256": query_sha256,
            "id_bytes_sha256": sha256_hex(&id_bytes),
            "classes_sha256": sha256_hex(&classes_bytes),
            "train_tokens": train_tokens,
            "query_tokens": query.len(),
        },
        "orders": order_json,
        "masses_sha256": sha256_hex(&mass_bytes),
        "masses_at_floor": masses_at_floor,
        "all": all.json(),
        "novel": novel.json(),
    });
    if let Some(json) = ceiling_json {
        summary["ceiling"] = json;
    }
    std::fs::write(&args.out, serde_json::to_string_pretty(&summary)? + "\n")?;
    eprintln!("wrote {}, {} ms", args.out, started.elapsed().as_millis());
    Ok(())
}

/// claude-433: the EM passes of every ceiling mode.
#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
const EM_ITERATIONS: u32 = 24;

/// claude-433: the ceiling modes over the lookups (`RawTables::lookups`'s layout) and the
/// KN masses, each tallied as the KN run is: `u_count` fit on every target and scored on
/// every target; `u_split` fit on one record parity and scored on the other; `kn_u` with
/// only the KN and uniform experts; `kn_only`, the KN masses themselves.
#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
fn ceiling_modes(
    highest: u32,
    query: &[u16],
    kn_masses: &[u64],
    counts: &[u32],
    totals: &[u64],
    id_bytes: &[u8],
    classes: &[u8],
) -> Result<serde_json::Value, Failure> {
    use uor_r4_discrete::kn_ceiling::{bucket_count, expert_count, CeilingFit, CeilingTable};

    let table = CeilingTable::from_lookups(highest, kn_masses, counts, totals)?;
    let experts = expert_count(highest);
    let targets = table.targets();
    let fit_bits =
        |fit: &CeilingFit| -> Vec<String> { fit.fit_bits.iter().map(|b| b.to_string()).collect() };
    let scored = |masses: &[u64]| -> Result<serde_json::Value, Failure> {
        let (all, novel) = tally(query, masses, id_bytes, classes)?;
        Ok(serde_json::json!({ "all": all.json(), "novel": novel.json() }))
    };

    let everyone = vec![true; targets];
    let u_count = table.fit_em(&everyone, EM_ITERATIONS, bits_q12)?;
    let u_count_masses = table.mixed_masses(&u_count.weights)?;
    eprintln!("ceiling u_count fitted");

    let parity = record_parity(query);
    let even: Vec<bool> = parity.iter().map(|&odd| !odd).collect();
    let even_fit = table.fit_em(&even, EM_ITERATIONS, bits_q12)?;
    let odd_fit = table.fit_em(&parity, EM_ITERATIONS, bits_q12)?;
    let split_masses = combine_by_parity(
        &parity,
        &table.mixed_masses(&even_fit.weights)?,
        &table.mixed_masses(&odd_fit.weights)?,
    )?;
    eprintln!("ceiling u_split fitted");

    let mut kn_and_uniform = vec![false; experts];
    kn_and_uniform[0] = true;
    kn_and_uniform[experts - 1] = true;
    let kn_u = table.fit_em_with(&everyone, &kn_and_uniform, EM_ITERATIONS, bits_q12)?;
    let kn_u_masses = table.mixed_masses(&kn_u.weights)?;
    eprintln!("ceiling kn_u fitted");

    let mut in_bucket = vec![0u64; bucket_count(highest)];
    for &b in table.buckets() {
        in_bucket[usize::from(b)] += 1;
    }
    let buckets: Vec<serde_json::Value> = in_bucket
        .iter()
        .enumerate()
        .filter(|&(_, &n)| n > 0)
        .map(|(b, &n)| {
            let weights: Vec<String> = u_count.weights[b * experts..(b + 1) * experts]
                .iter()
                .map(|w| w.to_string())
                .collect();
            serde_json::json!({ "bucket": b, "targets": n, "weights_q32": weights })
        })
        .collect();

    Ok(serde_json::json!({
        "em_iterations": EM_ITERATIONS,
        "experts": experts,
        "expert_order": "KN, ML_0..ML_N, uniform",
        "u_count": {
            "scored": scored(&u_count_masses)?,
            "fit_bits_q12": fit_bits(&u_count),
            "buckets": buckets,
        },
        "u_split": {
            "scored": scored(&split_masses)?,
            "fit_bits_q12_even": fit_bits(&even_fit),
            "fit_bits_q12_odd": fit_bits(&odd_fit),
        },
        "kn_u": {
            "scored": scored(&kn_u_masses)?,
            "fit_bits_q12": fit_bits(&kn_u),
        },
        "kn_only": { "scored": scored(kn_masses)? },
    }))
}

/// claude-433: the record parity of each non-BOS target, in stream order. A
/// target's record index is the number of BOS tokens before it; true is odd.
#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
fn record_parity(query: &[u16]) -> Vec<bool> {
    let mut odd = false;
    let mut parity = Vec::new();
    for &token in query {
        if token == NGRAM_BOS {
            odd = !odd;
        } else {
            parity.push(odd);
        }
    }
    parity
}

/// claude-433: the split mode's masses in stream order: an odd target takes
/// the mass of the fit on the even records, an even target that of the fit
/// on the odd records.
#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
fn combine_by_parity(
    parity: &[bool],
    from_even_fit: &[u64],
    from_odd_fit: &[u64],
) -> Result<Vec<u64>, Failure> {
    if from_even_fit.len() != parity.len() || from_odd_fit.len() != parity.len() {
        return Err(fail(format!(
            "{} targets, but the even fit mixed {} masses and the odd fit {}",
            parity.len(),
            from_even_fit.len(),
            from_odd_fit.len()
        )));
    }
    Ok(parity
        .iter()
        .zip(from_even_fit.iter().zip(from_odd_fit))
        .map(|(&odd, (&even_mass, &odd_mass))| if odd { even_mass } else { odd_mass })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ceiling_refuses_count_only_census_and_sealed_runs() {
        let base: Vec<String> = ["train", "query", "bytes", "classes", "4", "64", "out"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let with = |flags: &[&str]| {
            let mut args = base.clone();
            args.extend(flags.iter().map(|f| f.to_string()));
            parse(&args)
        };
        assert!(!with(&[]).unwrap().ceiling);
        let ceiling = with(&["--ceiling", "--check-host", "--profile-phases"]).unwrap();
        assert!(ceiling.ceiling && ceiling.check_host && !ceiling.count_only);
        for flag in ["--count-only", "--extension-census", "--sealed-run"] {
            assert!(with(&["--ceiling", flag]).is_err(), "{flag}");
            assert!(with(&[flag, "--ceiling"]).is_err(), "{flag}");
        }
        assert!(with(&["--sealed-run"]).is_ok());
    }

    #[test]
    fn a_record_parity_counts_the_bos_tokens_before_each_target() {
        let b = NGRAM_BOS;
        assert!(![5u16, 6, 7, 8, 9].contains(&b));
        // A target before any BOS, then records of length 0, 1 and 3.
        let stream = [9, b, b, 5, b, 6, 7, 8];
        assert_eq!(record_parity(&stream), [false, false, true, true, true]);
        assert!(record_parity(&[b, b]).is_empty());
    }

    #[test]
    fn the_split_fits_combine_back_in_stream_order() {
        let parity = [false, true, true, false, true];
        let from_even_fit = [10, 11, 12, 13, 14];
        let from_odd_fit = [20, 21, 22, 23, 24];
        assert_eq!(
            combine_by_parity(&parity, &from_even_fit, &from_odd_fit).unwrap(),
            [20, 11, 12, 23, 14]
        );
        assert!(combine_by_parity(&parity, &from_even_fit[..4], &from_odd_fit).is_err());
        assert!(combine_by_parity(&parity, &from_even_fit, &from_odd_fit[..4]).is_err());
    }

    #[test]
    fn synchronized_phase_profiling_requires_its_flag() {
        let mut args: Vec<String> = ["train", "query", "bytes", "classes", "4", "64", "out"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert!(!parse(&args).unwrap().profile_phases);
        args.push("--profile-phases".into());
        assert!(parse(&args).unwrap().profile_phases);
    }

    #[test]
    fn the_extension_census_scores_nothing() {
        let base: Vec<String> = ["train", "query", "bytes", "classes", "4", "64", "out"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert!(!parse(&base).unwrap().extension_census);
        let with = |flags: &[&str]| {
            let mut args = base.clone();
            args.extend(flags.iter().map(|f| f.to_string()));
            parse(&args)
        };
        assert!(with(&["--extension-census"]).unwrap().extension_census);
        assert!(with(&["--extension-census"]).unwrap().count_only);
        assert!(with(&["--extension-census", "--check-host"]).is_err());
        assert!(with(&["--extension-census", "--sealed-run"]).is_err());
    }

    #[test]
    fn a_count_only_run_scores_nothing() {
        let base: Vec<String> = ["train", "query", "bytes", "classes", "10", "64", "out"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let with = |flags: &[&str]| {
            let mut args = base.clone();
            args.extend(flags.iter().map(|f| f.to_string()));
            parse(&args)
        };
        let plain = with(&[]).unwrap();
        assert!(!plain.count_only && !plain.extension_census);
        let counted = with(&["--count-only", "--profile-phases"]).unwrap();
        assert!(counted.count_only && counted.profile_phases && !counted.extension_census);
        assert!(with(&["--count-only", "--check-host"]).is_err());
        assert!(with(&["--count-only", "--sealed-run"]).is_err());
        assert!(with(&["--check-host"]).is_ok());
    }

    #[test]
    fn a_prefix_ends_before_a_record_start() {
        // Records: [0 5 6 1] [0 7 1] [0 8 9 1].
        let tokens = [0u16, 5, 6, 1, 0, 7, 1, 0, 8, 9, 1];
        // The mark of 500 per mille is position 5, inside the second record.
        assert_eq!(record_prefix(&tokens, 500).unwrap(), 4);
        assert_eq!(record_prefix(&tokens, 1000).unwrap(), tokens.len());
        // 100 per mille marks position 1, inside the first record: no whole record.
        assert!(record_prefix(&tokens, 100).is_err());
        assert!(record_prefix(&tokens, 0).is_err());
        assert!(record_prefix(&tokens, 1001).is_err());
        assert!(record_prefix(&[], 500).is_err());
        // [0 5] [0 6 1]: the cut at the second BOS follows 5, not EOS.
        assert!(record_prefix(&[0, 5, 0, 6, 1], 500).is_err());
    }

    #[test]
    fn a_class_file_must_cover_the_whole_query_stream() {
        let classes = [3u8, 1, 0, 1, 3, 1];
        assert_eq!(classes_prefix(&classes, 6, 4).unwrap(), &classes[..4]);
        // A longer class file (another stream) agreeing on the prefix is refused.
        assert!(classes_prefix(&classes, 5, 4).is_err());
    }

    #[test]
    fn the_sealed_stream_is_refused_outside_the_sealed_run() {
        assert!(seal_check(DEV_TEST_TOKENS_SHA256, false).is_err());
        assert!(seal_check(DEV_TEST_TOKENS_SHA256, true).is_ok());
        assert!(seal_check(&"0".repeat(64), false).is_ok());
    }

    #[test]
    fn the_tally_sums_bits_of_each_mass_and_bytes_of_each_target() {
        let half = PROBABILITY_ONE_TEST / 2;
        let quarter = PROBABILITY_ONE_TEST / 4;
        // Query [BOS 5 6 EOS]; classes [3 1 0 1].
        let query = [0u16, 5, 6, 1];
        let classes = [3u8, 1, 0, 1];
        let mut id_bytes = vec![0u8; 4096];
        id_bytes[5] = 2;
        id_bytes[6] = 3;
        let (all, novel) = tally(&query, &[half, quarter, half], &id_bytes, &classes).unwrap();
        // -log2(1/2) = 1 bit, -log2(1/4) = 2 bits; Q12 units.
        assert_eq!(
            all,
            Tally {
                targets: 3,
                bits_q12: 4 * 4096,
                bytes: 5
            }
        );
        assert_eq!(
            novel,
            Tally {
                targets: 2,
                bits_q12: 2 * 4096,
                bytes: 2
            }
        );
        // 4 bits over 5 bytes.
        assert_eq!(all.bpb_micro(), Some(800_000));
        // A mass of 0, a missing mass, a spare mass and a BOS/class mismatch are refused.
        assert!(tally(&query, &[half, 0, half], &id_bytes, &classes).is_err());
        assert!(tally(&query, &[half, quarter], &id_bytes, &classes).is_err());
        assert!(tally(&query, &[half, quarter, half, half], &id_bytes, &classes).is_err());
        assert!(tally(&query, &[half, quarter, half], &id_bytes, &[0, 1, 0, 1]).is_err());
        assert!(tally(&query, &[half, quarter, half], &id_bytes, &[3, 4, 0, 1]).is_err());
    }

    const PROBABILITY_ONE_TEST: u64 = 1 << 48;
}
