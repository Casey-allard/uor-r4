//! Evidence-sparsity audit (issue #433).
//!
//! First measurement for the broad-topic ceiling question: on wiki10k-class
//! corpora the substrate caps at roughly 26-27% held-out top-1 at every
//! capacity, vs 36.5% on the redundancy-rich corpus. Is that ceiling
//! EVIDENCE-SPARSITY (held-out contexts are genuinely novel — no
//! construction evidence exists at any key) or REPRESENTATION (evidence
//! exists but the codes/cover cannot reach it)?
//!
//! For each held-out position this harness computes ORACLE evidence
//! availability in the construction split, independent of any code or
//! cover representation:
//!
//! - Exact-context match ladder: for k in {1, 2, 3, 4, 8}, does ANY
//!   construction position share the same last-k input tokens
//!   (story-bounded windows, [`story_bounded_window`] semantics — the same
//!   boundary rule the compiler's trigram keys obey)? If so, does the
//!   majority continuation at that key equal the held-out target?
//!   Reported per k: match-rate (evidence exists) and oracle top-1 (the
//!   upper bound of ANY representation that keys on exactly that context
//!   length). A held-out position whose story prefix is shorter than k has
//!   no length-k context and counts as unmatched at that k.
//! - The unigram null as floor (majority target token of the construction
//!   split), and as the backoff prediction wherever a ladder key is absent.
//! - Longest-match backoff (largest matched k wins, else unigram) and the
//!   any-k oracle (correct if the majority continuation at ANY matched k
//!   equals truth) — the ceiling of a representation free to pick its key
//!   length per position.
//!
//! TARGET DEFINITION: the whole ladder runs twice, under two target
//! streams, because on broad corpora they diverge sharply:
//!
//! - "observed-token target": evidence and truth are `c.next`, the sampled
//!   corpus continuation. On high-entropy text this target is largely
//!   unpredictable and the oracle ceiling is low.
//! - "teacher-argmax target": evidence and truth are `c.t_argmax`, the
//!   teacher's greedy token — the target gate C's measured ~26% agreement
//!   is scored against. This ladder is the ceiling comparable to that
//!   number.
//!
//! The printed agreement rate P(next == `t_argmax`) on each split
//! quantifies the divergence between the two targets.
//!
//! Pre-declared interpretation (teacher-argmax ladder vs gate C's 26%):
//! if oracle top-1 at k <= 4 lands around the measured 26-28% ceiling,
//! the substrate is ALREADY near the evidence ceiling — sparsity is the
//! binding constraint, better representation cannot help much, and more
//! data or longer keys are the lever. If the oracle reaches 35% or more,
//! there is a representation gap: the codes are losing evidence that
//! exact-context lookup can recover.
//!
//! Determinism: ties in every majority argmax break to the lowest token id
//! (the convention of `anchor_infill.rs`). Window keys for k <= 4 are
//! exact u128 packings of the tokens (collision-free); k = 8 keys are
//! 64-bit `DefaultHasher` digests of the token window. `DefaultHasher` is
//! stable within one process run but NOT guaranteed stable across Rust
//! releases — acceptable for a measurement, and at ~2M distinct keys the
//! expected 64-bit collision count is ~1e-7, i.e. zero in practice.
//!
//! Run:
//!   cargo test -p uor-r4-graph-certify --test evidence_sparsity -- --ignored --nocapture
//!
//! Env: R4_CORPUS_META / R4_CORPUS_RECS select the corpus (default:
//! checked-in fixture); R4_STORIES points at an obs pass's stories.jsonl
//! to use its D3 article-hash partition (else sequential 80/20 story cut).

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use uor_r4_core::transformerless::compiler;
use uor_r4_graph_certify::score::story_bounded_window;

/// Continuation-count table: window key -> (target token -> count).
type ContTable = HashMap<u128, HashMap<u32, u32>>;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// Key for a length-k token window. k <= 4: exact packing of the tokens
/// into a u128 behind a length sentinel bit (collision-free). k > 4:
/// 64-bit `DefaultHasher` digest (stable within this run; documented in
/// the module header).
fn window_key(window: &[u32]) -> u128 {
    if window.len() <= 4 {
        let mut key = 1u128;
        for &t in window {
            key = (key << 32) | u128::from(t);
        }
        key
    } else {
        let mut h = DefaultHasher::new();
        window.hash(&mut h);
        u128::from(h.finish())
    }
}

/// Majority token of a count distribution; ties break to the lowest token.
fn majority(dist: &HashMap<u32, u32>) -> Option<u32> {
    dist.iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&tok, _)| tok)
}

/// Construction / held-out story partition, following the
/// `anchor_infill.rs` convention: R4_STORIES selects the D3 article-hash
/// split recorded in an obs pass's stories.jsonl, else a sequential 80/20
/// story cut.
fn construction_split(c: &compiler::Corpus) -> Vec<bool> {
    let cut = (c.stories as f64 * 0.8) as u32;
    match std::env::var("R4_STORIES") {
        Ok(path) => {
            let text = std::fs::read_to_string(&path).expect("stories.jsonl");
            let mut v = vec![true; c.stories as usize];
            for line in text.lines() {
                let Some(story_pos) = line.find("\"story\":") else {
                    continue;
                };
                let story: usize = line[story_pos + 8..]
                    .split(',')
                    .next()
                    .and_then(|x| x.trim().parse().ok())
                    .expect("story id");
                if story < v.len() {
                    v[story] = !line.contains("\"partition\":\"HeldOut\"");
                }
            }
            println!(
                "partition: D3 hash split from {path} ({} construction / {} held-out stories)",
                v.iter().filter(|&&b| b).count(),
                v.iter().filter(|&&b| !b).count()
            );
            v
        }
        Err(_) => (0..c.stories).map(|sid| sid < u64::from(cut)).collect(),
    }
}

/// Per-k audit result over the held-out positions (parallel to the
/// held-out ordinal enumeration).
struct KAudit {
    k: usize,
    /// Held-out position has a full-length window AND its key exists in
    /// the construction table.
    matched: Vec<bool>,
    /// Matched AND the majority continuation equals the held-out truth.
    correct: Vec<bool>,
    distinct_keys: usize,
    construction_positions: u64,
}

/// Build the length-k construction table over `target` continuations,
/// then grade every held-out position's `target` against it. Two passes
/// over the corpus; the table is dropped on return so peak memory is one
/// k at a time.
fn audit_k(
    c: &compiler::Corpus,
    constr: &[bool],
    target: &[u32],
    k: usize,
    held_n: usize,
) -> KAudit {
    let mut table: ContTable = HashMap::new();
    let mut construction_positions = 0u64;
    #[allow(clippy::needless_range_loop)] // index i addresses parallel corpus arrays
    for i in 0..c.n {
        if !constr[c.story[i] as usize] {
            continue;
        }
        let window = story_bounded_window(c, i, k);
        if window.len() < k {
            continue;
        }
        construction_positions += 1;
        *table
            .entry(window_key(window))
            .or_default()
            .entry(target[i])
            .or_default() += 1;
    }
    let distinct_keys = table.len();

    let mut matched = vec![false; held_n];
    let mut correct = vec![false; held_n];
    let mut ord = 0usize;
    #[allow(clippy::needless_range_loop)] // index i addresses parallel corpus arrays
    for i in 0..c.n {
        if constr[c.story[i] as usize] {
            continue;
        }
        let window = story_bounded_window(c, i, k);
        if window.len() == k {
            if let Some(dist) = table.get(&window_key(window)) {
                matched[ord] = true;
                correct[ord] = majority(dist) == Some(target[i]);
            }
        }
        ord += 1;
    }
    assert_eq!(ord, held_n, "held-out enumeration drifted");
    KAudit {
        k,
        matched,
        correct,
        distinct_keys,
        construction_positions,
    }
}

/// Run the full match ladder (unigram floor, per-k table, composite
/// ceilings) under one target stream and print it under `label`.
fn run_ladder(c: &compiler::Corpus, constr: &[bool], target: &[u32], label: &str) {
    // ---- unigram null (construction split, target continuations) ----
    let mut unigram: HashMap<u32, u32> = HashMap::new();
    #[allow(clippy::needless_range_loop)] // index i addresses parallel corpus arrays
    for i in 0..c.n {
        if constr[c.story[i] as usize] {
            *unigram.entry(target[i]).or_default() += 1;
        }
    }
    let unigram_pred = majority(&unigram);

    // ---- held-out enumeration + unigram floor ----
    let mut uni_correct: Vec<bool> = Vec::new();
    #[allow(clippy::needless_range_loop)] // index i addresses parallel corpus arrays
    for i in 0..c.n {
        if !constr[c.story[i] as usize] {
            uni_correct.push(unigram_pred == Some(target[i]));
        }
    }
    let held_n = uni_correct.len();
    assert!(held_n > 0, "no held-out positions under this split");
    let pct = |h: u64, t: u64| 100.0 * h as f64 / t.max(1) as f64;
    let count_true = |v: &[bool]| v.iter().filter(|&&b| b).count() as u64;
    println!(
        "\n==== ladder: {label} ====\nheld-out positions: {held_n} | unigram floor top-1: {:.2}%",
        pct(count_true(&uni_correct), held_n as u64)
    );

    // ---- exact-context match ladder ----
    const KS: [usize; 5] = [1, 2, 3, 4, 8];
    let audits: Vec<KAudit> = KS
        .iter()
        .map(|&k| audit_k(c, constr, target, k, held_n))
        .collect();

    println!(
        "{:>3} | {:>10} | {:>12} | {:>18} | {:>12} | {:>14}",
        "k", "match-rate", "oracle@match", "oracle+uni-backoff", "keys(constr)", "positions"
    );
    for a in &audits {
        let m = count_true(&a.matched);
        let hits = count_true(&a.correct);
        // unmatched positions fall back to the unigram null, so the arm
        // always predicts (comparable with the measured substrate top-1).
        let backoff_hits: u64 = (0..held_n)
            .map(|p| {
                u64::from(if a.matched[p] {
                    a.correct[p]
                } else {
                    uni_correct[p]
                })
            })
            .sum();
        println!(
            "{:>3} | {:>9.2}% | {:>11.2}% | {:>17.2}% | {:>12} | {:>14}",
            a.k,
            pct(m, held_n as u64),
            pct(hits, m),
            pct(backoff_hits, held_n as u64),
            a.distinct_keys,
            a.construction_positions
        );
    }

    // ---- composite ceilings across the ladder ----
    // longest-match backoff: largest matched k wins, else unigram.
    let longest_hits: u64 = (0..held_n)
        .map(|p| {
            for a in audits.iter().rev() {
                if a.matched[p] {
                    return u64::from(a.correct[p]);
                }
            }
            u64::from(uni_correct[p])
        })
        .sum();
    // any-k oracle: correct if ANY matched k's majority equals truth —
    // the ceiling of a representation free to pick its key length.
    let any_k_hits: u64 = (0..held_n)
        .map(|p| {
            let any = audits.iter().any(|a| a.correct[p]);
            let none_matched = !audits.iter().any(|a| a.matched[p]);
            u64::from(any || (none_matched && uni_correct[p]))
        })
        .sum();
    let no_evidence: u64 = (0..held_n)
        .map(|p| u64::from(!audits.iter().any(|a| a.matched[p])))
        .sum();
    println!(
        "longest-match backoff top-1: {:.2}% | any-k oracle top-1: {:.2}%",
        pct(longest_hits, held_n as u64),
        pct(any_k_hits, held_n as u64)
    );
    println!(
        "no-evidence-at-any-k (genuinely novel even at k=1): {} ({:.2}% of held-out)",
        no_evidence,
        pct(no_evidence, held_n as u64)
    );
}

#[test]
#[ignore = "measurement harness (issue #433); run explicitly with --ignored"]
fn evidence_sparsity_audit() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let c = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    println!("corpus: {meta_path} + {recs_path}");
    println!(
        "evidence-sparsity audit (#433): {} records, {} stories",
        c.n, c.stories
    );
    let constr = construction_split(&c);

    // ---- target divergence: P(next == t_argmax) per split ----
    let mut agree = [0u64; 2]; // [construction, held-out]
    let mut total = [0u64; 2];
    #[allow(clippy::needless_range_loop)] // index i addresses parallel corpus arrays
    for i in 0..c.n {
        let s = usize::from(!constr[c.story[i] as usize]);
        total[s] += 1;
        agree[s] += u64::from(c.next[i] == c.t_argmax[i]);
    }
    let pct = |h: u64, t: u64| 100.0 * h as f64 / t.max(1) as f64;
    println!(
        "target agreement P(next == t_argmax): construction {:.2}% ({}/{}) | held-out {:.2}% ({}/{})",
        pct(agree[0], total[0]),
        agree[0],
        total[0],
        pct(agree[1], total[1]),
        agree[1],
        total[1]
    );

    run_ladder(&c, &constr, &c.next, "observed-token target (c.next)");
    run_ladder(
        &c,
        &constr,
        &c.t_argmax,
        "teacher-argmax target (c.t_argmax — gate C's target)",
    );
}
