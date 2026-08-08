//! Spectral vs VSA geometry on the production retrieval surface (#434 item 2).
//!
//! # What this replaces
//!
//! `benchmark::run_ablation_benchmark` plus `tests/ablation_benchmarks.rs`
//! were the only coverage `SpectralGeometry` and `VsaGeometry` had, and they
//! could not measure anything: `migration_agreement` was a hard-coded `0.98`
//! that the test then asserted equal to `0.98`; `unlearning_time_ns` timed
//! `ground()` under a comment claiming it measured route deletion;
//! `recall_at_3` and `hits_at_3` were the same quantity computed twice; and
//! the remaining assertions were `>= 0.0` on ratios of non-negative counts.
//! Two synthetic queries, no corpus, no ground truth, no null.
//!
//! The underlying question is real and live. `GeometryType` is a router
//! config switch (`Spectral` by default, `Vsa` via `set_geometry_type`), and
//! `get_top_resonances_native` dispatches on it into genuinely different
//! retrieval implementations — `retrieve_vsa_multi_facet_resonance` versus
//! route-then-`retrieve_geometric_resonance`. Which one the router should use
//! is a decision with no evidence behind it.
//!
//! # Method — the #422/#423 laws, unchanged
//!
//! Same natural stack as `zeta_state_retrieval.rs`: the D3 token corpus via
//! `R4_CORPUS_META` / `R4_CORPUS_RECS`, token ids rendered as synthetic words
//! (`t00042`), the D3 hash partition when `R4_STORIES` is set (else the
//! sequential 80/20 story cut), consecutive non-overlapping eight-token
//! windows deduplicated to first occurrence, and probe queries in the #423
//! held-out form (a target window's even-offset tokens in reverse order; the
//! query string is never ingested). Caps are the same env knobs and print.
//!
//! **Storage shape is the shipped default (full-width).** `zeta_state_retrieval`
//! explicitly re-enables banding because its arms are defined against the
//! banded shape; this harness measures the production surface, so it takes
//! the post-#465 default.
//!
//! # Arms
//!
//! Both arms are the SAME store, the SAME probes and the SAME ground truth —
//! only `set_geometry_type` differs, so the comparison isolates the switch.
//! Retrieval goes through `get_top_resonances_native`, the surface a caller
//! actually uses, and a probe's rank is the position of its ground-truth
//! `window_index` in the returned list.
//!
//! # Null arm (falsifier)
//!
//! Each arm is re-scored against a DERANGED ground truth: probe `i` is graded
//! against target `i + 1 (mod n)`. Same store, same queries, same retrieval
//! calls — only the answer key moves. A geometry that scores on the deranged
//! key is matching on something other than content, and its real number does
//! not count.
//!
//! # Pre-declared exit rule
//!
//! - The null must be near zero for both arms (MRR below 0.02), or the
//!   harness is void.
//! - A geometry WINS if its MRR exceeds the other's by >= 0.05.
//! - Otherwise the switch is MEASURED INDIFFERENT, and that is the result:
//!   recorded with numbers so the choice is not re-litigated from intuition.
//!
//! Reference points on this stack, for scale — but NOT commensurable with this
//! harness's numbers: content-full-width 0.8948 MRR, banded 0.2348, shuffled
//! control 0.0001 (#440). Those are COSINE-ranked. This harness's Spectral row
//! ranks by `shared_count * 100 + sim * slice_norm`, and #484/#486 measured the
//! cosine term (`sim`) at chance on this path (the query vector is the routing
//! vector, not the content vector). So this arm's score is word overlap, not
//! spectral geometry, and it cannot be read on the same scale as 0.8948. See
//! `docs/geometry_ablation_434.md` (correction block) and #487.
//!
//! # Two harness traps, recorded because both look exactly like a finding
//!
//! **Retrieval is identity-scoped.** `retrieve_geometric_resonance` reads
//! `corpus_index_by_identity[identity_key(identity)]`, so a probe must query
//! under the identity the corpus was indexed with. Querying fresh per-probe
//! identities returns an empty index and scores zero for BOTH arms, which
//! reads as "neither geometry works" rather than "the harness asked the wrong
//! store".
//!
//! **`ResonanceResult::window_index` is not a store index.** It is a
//! routing-window identifier; several distinct stored sentences share one
//! value. Ranking ground truth by it also scores zero for both arms. Ground
//! truth here is matched on the stored SENTENCE, which is what
//! `aligned_vectors` in `zeta_state_retrieval.rs` does and for the same
//! reason.
//!
//! Both were hit while building this, and both produced a clean, plausible,
//! entirely false "measured indifferent" verdict over two zeros. An all-zero
//! result across every arm is a harness bug until proven otherwise.
//!
//! Recorded outcome: `docs/geometry_ablation_434.md`.

use std::collections::HashSet;

use uor_r4_core::transformerless::compiler;
use uor_r4_router::UorR4Router;

const ID: &str = "user:geomab";
/// Depth of the returned resonance list. A probe whose target is not in the
/// list scores rank 0 (no reciprocal credit), which is the honest treatment:
/// the production caller sees only this list.
const TOP_N: usize = 20;
/// Pre-declared: null MRR must sit below this or the harness is void.
const NULL_MRR_CEILING: f64 = 0.02;
/// Pre-declared: MRR margin that constitutes a win.
const WIN_MARGIN: f64 = 0.05;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn token_word(token: u32) -> String {
    format!("t{token:05}")
}

fn render_window(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens.iter().map(|&token| token_word(token)).collect();
    format!("{}.", words.join(" "))
}

/// The #423 held-out probe form: even-offset tokens in reverse order.
fn render_query(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens
        .iter()
        .step_by(2)
        .rev()
        .map(|&token| token_word(token))
        .collect();
    words.join(" ")
}

/// (top-1 rate, MRR, recall@TOP_N) over ranks; rank 0 means "not returned".
fn metrics(ranks: &[usize]) -> (f64, f64, f64) {
    let n = ranks.len() as f64;
    let hits = ranks.iter().filter(|&&r| r == 1).count() as f64;
    let found = ranks.iter().filter(|&&r| r > 0).count() as f64;
    let mrr: f64 = ranks
        .iter()
        .map(|&r| if r == 0 { 0.0 } else { 1.0 / r as f64 })
        .sum();
    (hits / n, mrr / n, found / n)
}

/// Rank of the window whose text is `target_text`, or 0 if absent.
///
/// Ground truth is matched on the stored SENTENCE, not on
/// `ResonanceResult::window_index`. `window_index` is a routing-window
/// identifier, not a per-item store index — several distinct stored
/// sentences share one value — so ranking by it scores zero for every arm
/// and looks exactly like "both geometries fail". `aligned_vectors` in
/// `zeta_state_retrieval.rs` matches by sentence for the same reason.
fn rank_of(results: &[uor_r4_router::ResonanceResult], target_text: &str) -> usize {
    results
        .iter()
        .position(|r| r.sentence == target_text)
        .map(|p| p + 1)
        .unwrap_or(0)
}

#[test]
#[ignore = "measurement harness (issue #434 item 2); run explicitly with --ignored"]
fn spectral_vs_vsa_retrieval() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let Some(c) = compiler::load_corpus_from(&meta_path, &recs_path) else {
        println!("SKIP: corpus fixtures absent ({meta_path} + {recs_path})");
        return;
    };
    println!("#434 item 2: Spectral vs VSA on the production retrieval surface");
    println!("corpus: {meta_path} + {recs_path} ({} records)", c.n);

    let cut = (c.stories as f64 * 0.8) as u32;
    let constr: Vec<bool> = (0..c.stories).map(|sid| sid < u64::from(cut)).collect();

    // Per-story token streams (stream index zero = first input).
    let mut streams: Vec<(u32, Vec<u32>)> = Vec::new();
    for ((&sid, &input), &next) in c.story.iter().zip(&c.input).zip(&c.next).take(c.n) {
        if streams.last().map(|(last, _)| *last) != Some(sid) {
            streams.push((sid, vec![input]));
        }
        let (_, stream) = streams.last_mut().expect("just pushed");
        stream.push(next);
    }

    let story_cap = env_usize("R4_HOPF_CONSTR_STORIES", 2_000);
    let probe_cap = env_usize("R4_HOPF_PROBES", 500).max(1);
    let capped: Vec<&(u32, Vec<u32>)> = streams
        .iter()
        .filter(|(sid, _)| constr[*sid as usize])
        .take(story_cap)
        .collect();
    println!(
        "CAPS: ingesting {} construction stories; up to {probe_cap} probes; top_n {TOP_N}",
        capped.len()
    );

    let mut windows: Vec<String> = Vec::new();
    let mut window_tokens: Vec<Vec<u32>> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (_, stream) in &capped {
        for chunk in stream.chunks_exact(compiler::WINDOW) {
            let sentence = render_window(chunk);
            if seen.insert(sentence.clone()) {
                windows.push(sentence);
                window_tokens.push(chunk.to_vec());
            }
        }
    }
    println!("windows: {} distinct eight-token windows", windows.len());
    assert!(
        windows.len() > probe_cap,
        "corpus-scale premise: more windows than probes"
    );

    let stride = (windows.len() / probe_cap).max(1);
    let targets: Vec<usize> = (0..windows.len()).step_by(stride).take(probe_cap).collect();
    let queries: Vec<String> = targets
        .iter()
        .map(|&t| render_query(&window_tokens[t]))
        .collect();
    println!(
        "probes: {} targets, stride {stride}, query = even-offset tokens reversed",
        targets.len()
    );

    // Deranged answer key: probe i graded against target i+1 (mod n). A true
    // derangement for n > 1, so no probe is ever graded against its own
    // target.
    let deranged: Vec<usize> = (0..targets.len())
        .map(|i| targets[(i + 1) % targets.len()])
        .collect();

    println!("\narm\t\ttop-1\tMRR\trecall@{TOP_N}\tnull top-1\tnull MRR");
    let mut measured: Vec<(&str, f64, f64, f64, f64)> = Vec::new();
    for geometry in ["spectral", "vsa"] {
        // One fresh store per arm so neither inherits the other's session
        // state; storage shape is the shipped post-#465 default.
        let mut router = UorR4Router::new(0.5);
        router.set_geometry_type(geometry);
        let corpus_text: String = windows.join(" ");
        let indexed = router.index_corpus(&corpus_text, ID);
        assert_eq!(
            indexed,
            windows.len(),
            "[{geometry}] production bulk surface indexed every distinct window"
        );

        let mut ranks = Vec::with_capacity(queries.len());
        let mut null_ranks = Vec::with_capacity(queries.len());
        for (qi, query) in queries.iter().enumerate() {
            // Retrieval is identity-scoped: `retrieve_geometric_resonance`
            // looks the store up under `corpus_index_by_identity[key]`, so a
            // probe must query under the identity the corpus was indexed
            // with. Querying fresh identities returns an empty index and
            // scores zero for every arm — which is what a first pass of this
            // harness did, and why the all-zero result was a harness bug and
            // not a finding.
            let results = router.get_top_resonances_native(query, ID, TOP_N);
            ranks.push(rank_of(&results, &windows[targets[qi]]));
            null_ranks.push(rank_of(&results, &windows[deranged[qi]]));
        }

        let (top1, mrr, recall) = metrics(&ranks);
        let (null_top1, null_mrr, _) = metrics(&null_ranks);
        println!(
            "{geometry:<12}\t{top1:.4}\t{mrr:.4}\t{recall:.4}\t\t{null_top1:.4}\t\t{null_mrr:.4}"
        );
        measured.push((
            if geometry == "spectral" {
                "spectral"
            } else {
                "vsa"
            },
            top1,
            mrr,
            null_mrr,
            recall,
        ));
    }

    let (spectral_name, _, spectral_mrr, spectral_null, spectral_recall) = measured[0];
    let (vsa_name, _, vsa_mrr, vsa_null, vsa_recall) = measured[1];

    println!("\n==== validity ====");
    let null_dead = spectral_null < NULL_MRR_CEILING && vsa_null < NULL_MRR_CEILING;
    println!(
        "deranged-key MRR: {spectral_name} {spectral_null:.4}, {vsa_name} {vsa_null:.4} \
         (ceiling {NULL_MRR_CEILING}) — {}",
        if null_dead {
            "PASS"
        } else {
            "VOID: an arm scores on the wrong answer key, so it is not matching on content"
        }
    );
    assert!(
        null_dead,
        "deranged-key control must be near zero; {spectral_name} {spectral_null:.4}, \
         {vsa_name} {vsa_null:.4}"
    );

    println!("\n==== verdict against the pre-declared exit rule ====");
    let margin = (spectral_mrr - vsa_mrr).abs();
    if margin >= WIN_MARGIN {
        let (winner, loser) = if spectral_mrr > vsa_mrr {
            (spectral_name, vsa_name)
        } else {
            (vsa_name, spectral_name)
        };
        println!(
            "{winner} WINS by {margin:.4} MRR (>= {WIN_MARGIN}). {loser} becomes a removal \
             candidate and {winner} becomes a documented default with a reason rather than \
             a historical accident."
        );
    } else {
        println!(
            "MEASURED INDIFFERENT: MRR margin {margin:.4} < {WIN_MARGIN} \
             ({spectral_name} {spectral_mrr:.4} vs {vsa_name} {vsa_mrr:.4}; recall \
             {spectral_recall:.4} vs {vsa_recall:.4}). The geometry switch does not change \
             retrieval quality on this corpus. The default stands on other grounds, and \
             this is now a number rather than an intuition."
        );
    }
}
