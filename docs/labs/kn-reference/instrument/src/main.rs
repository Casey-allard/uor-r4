//! `kn-reference` (claude-421): the bits instrument of the sealed reference R*.
//!
//! It counts TRAIN's modified Kneser-Ney adjusted orders 0..=N on the GPU, keeps only the
//! records the query's targets read (`KnQuery`), scores the query on the GPU (`KnScorer`,
//! equal to the host `KnTables`), takes `lawful_likelihood::bits_q12` of every Q48 mass, and
//! sums bits and bytes over all targets and over the novel class. Everything is an integer.
//!
//! Usage (needs `--features cuda`):
//! `kn-reference TRAIN.u16 QUERY.u16 ID-BYTES.u8 CLASSES.u8 N BUDGET OUT.json
//!  [--per-mille P] [--check-host] [--profile-phases]`
//! `--per-mille P` keeps the first P per mille of TRAIN and of the query, each cut at a record
//! start. `--profile-phases` adds synchronized wall-time diagnostics on stderr.
//! `--check-host` also scores on the host and refuses at the first mass that differs.

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
}

fn parse(args: &[String]) -> Result<Args, Failure> {
    let usage = "usage: kn-reference TRAIN.u16 QUERY.u16 ID-BYTES.u8 CLASSES.u8 N BUDGET OUT.json [--per-mille P] [--check-host] [--sealed-run] [--profile-phases]";
    let mut positional = Vec::new();
    let mut per_mille = 1000;
    let mut check_host = false;
    let mut profile_phases = false;
    let mut sealed_run = false;
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
            other => positional.push(other.to_string()),
        }
        at += 1;
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
    use uor_r4_discrete::kn_filter::KnQuery;
    use uor_r4_discrete::kn_score::KnTables;
    use uor_r4_discrete::kn_score_device::KnScorer;
    use uor_r4_discrete::ngram_count::{read_uort_tokens, NgramAssembler, NgramOrderDigest};
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
    {
        for k in 0..=args.highest {
            let mut assembler = NgramAssembler::new_adjusted(k, false)?;
            let mut filter = wanted.filter(k)?;
            // claude-423: the sink's host time, apart from the counter's phases.
            let (mut push_ns, mut keep_ns) = (0u64, 0u64);
            let nanos = |since: std::time::Instant| {
                u64::try_from(since.elapsed().as_nanos()).unwrap_or(u64::MAX)
            };
            let start_phase = || args.profile_phases.then(std::time::Instant::now);
            let order_started = start_phase();
            let plan = counter.count_order_adjusted_with(
                k,
                args.highest,
                args.budget,
                &mut |partition| {
                    let at = start_phase();
                    assembler.push(partition)?;
                    if let Some(at) = at {
                        push_ns += nanos(at);
                    }
                    let at = start_phase();
                    let kept = filter.keep(partition);
                    if let Some(at) = at {
                        keep_ns += nanos(at);
                    }
                    kept
                },
            )?;
            if let Some(order_started) = order_started {
                let order_ns = nanos(order_started);
                let p = counter.take_phases();
                let ms = |ns: u64| ns / 1_000_000;
                eprintln!(
                    "order {k} phases ms: wall {} | own count {} extension count {} | histogram {} select {} sort {} emit {} download {} records {} | runs {} adjust {} | push {} keep {} | {} partitions counted",
                    ms(order_ns),
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
                    ms(push_ns),
                    ms(keep_ns),
                    p.partitions
                );
            }
            let (digest, _, _) = assembler.finish();
            let (triples, contexts) = filter.finish();
            eprintln!(
                "order {k}: {} partitions, grams {} contexts {}, kept triples {} contexts {}, {} ms",
                plan.len(),
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
                "partitions": plan.len(),
                "kept_triples": triples.len(),
                "kept_contexts": contexts.len(),
            }));
            orders.push((digest, triples, contexts));
        }
    }
    drop(counter);
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
    let summary = serde_json::json!({
        "unit": "claude-421",
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
    std::fs::write(&args.out, serde_json::to_string_pretty(&summary)? + "\n")?;
    eprintln!("wrote {}, {} ms", args.out, started.elapsed().as_millis());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
