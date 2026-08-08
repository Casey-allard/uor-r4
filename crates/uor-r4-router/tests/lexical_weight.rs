//! What the `shared_count * 100` lexical weight is actually worth (issue #484).
//!
//! # The question
//!
//! `retrieve_geometric_resonance` ranks by
//!
//! ```text
//! relevance = shared_count * W + sim * slice_norm + scope_boost
//! ```
//!
//! with `W` a hard-coded `100.0` that had never been measured against any
//! alternative. #480 established what that costs: at `W = 100`, against a
//! cosine bounded by one and a `scope_boost` of 15, the ranking is LEXICAL
//! and the geometry is a tie-break inside equal-overlap groups. Making the
//! query projection symmetric with storage — a real wiring fix — was worth
//! +0.0059 MRR for exactly that reason, and the adopted de-banding gain
//! (retrieval MRR 0.2348 -> 0.8948, #442) cannot reach this path at all,
//! because that figure came from a harness ranking by cosine ALONE.
//!
//! `W` is not a flag, it is a continuum. At `W = 0` the ranking is pure
//! cosine; as `W` grows it approaches strict lexicographic order with the
//! cosine as tie-break. The shipped value is one arbitrary point on it. This
//! harness looks at the others.
//!
//! # Why the two known numbers do NOT settle it
//!
//! 0.8948 (cosine-only, #442) against 0.7179 (lexical, #480) is the reason
//! to run this, not the result of it: different harnesses, different
//! corpora, different probe forms. Putting both rankings on ONE harness over
//! ONE corpus is the whole point. Quoting the cross-harness gap as if it
//! were a measured delta would repeat the error #480 was filed on.
//!
//! # Two baselines since #490
//!
//! When #484 was first run, the deployed default was the routing-path query and
//! the cosine was at chance (#486). "The weight is inert" was measured there:
//! with no geometric signal for `W` to trade against, any `W > ~0.4` gives the
//! same lexicographic order. #490 then made the deployed query the CONTENT
//! vector, so the cosine now carries signal — and the inertness claim has to be
//! reassessed on that path. This harness runs in two modes:
//!
//! - **default (`content_query = false`)** — reproduces the PRE-#490 lexical
//!   baseline. Retained as the regression tell: the 0.6240 / 0.7179 / 0.9720
//!   triple is the dead-cosine measurement that #434, #480 and #484 each
//!   re-derived without noticing it was one number three times, so it is pinned
//!   and asserted. This is NOT what ships any more.
//! - **deployed (`R4_LEXW_CONTENT_QUERY=1`, `content_query = true`)** — the path
//!   that ships since #490. Here the weight is NOT inert: dropping the lexical
//!   term (`W = 0`, pure `sim * slice_norm` geometry) is worth about +0.022 MRR
//!   and lifts recall 0.9720 -> 0.9900. Since #502 that `W = 0` row IS the
//!   deployed default on this path (`UorR4Router::default_lexical_weight`); the
//!   `W = 100` arm is retained as a reproducibility anchor. Both are pinned
//!   below (`CONTENT_W0_*` deployed, `CONTENT_*` the retired W=100 row).
//!
//! # Arms
//!
//! `W` in {0, 1, 10, 100 (shipped), 1000, 100000 (effectively strict
//! lexicographic)}. Storage and query shapes are held fixed within a mode, so
//! `W` is the only thing that moves; the query KIND (routing vs content) is the
//! mode switch above.
//!
//! # Corpus, probes, null — identical to `query_projection.rs`
//!
//! Deliberately verbatim, so these numbers are COMPARABLE to the #480
//! record rather than merely similar to it: D3 token corpus, synthetic word
//! rendering (`t00042`, the standing #422/#423 law for this stack),
//! eight-token windows deduplicated to first occurrence, probes in the
//! held-out even-offset-reversed form, ground truth matched on the stored
//! SENTENCE (never on `window_index`, which is a routing-window id shared by
//! several sentences), and a deranged answer key (probe `i` graded against
//! target `i + 1 mod n`) as the falsifier.
//!
//! # Validity checks, both binding
//!
//! 1. **The `W = 100` arm must reproduce the mode's pinned row.** In default
//!    mode that is #480's PRE-#490 dead-cosine row (0.6240 top-1 / 0.7179 MRR
//!    / 0.9720 recall@20, `SHIPPED_*`); in content-query mode it is the
//!    post-#490 deployed row (`CONTENT_*`). If the `W = 100` arm does not
//!    reproduce its mode's pin, the corpus or probes moved and the sweep says
//!    nothing about the weight. Checked against pinned constants, not by eye.
//!    The two pins also guard the DISTINCTNESS of the baselines: the deployed
//!    content row must sit well above the dead-cosine 0.7179, or #490 has
//!    regressed.
//! 2. **The deranged-key null must be near zero in EVERY arm.** A weight
//!    that lifts the null is not measuring retrieval quality. This matters
//!    more here than in #480: `W = 0` is a genuinely different ranking, and
//!    it gets no benefit of the doubt from the other arms passing.
//!
//! # Pre-declared exit rule (issue #484)
//!
//! POSITIVE if some `W` beats the shipped arm by >= 0.05 MRR AND costs no
//! more than 0.02 of recall@20, with the null dead everywhere. Same bar
//! #480 was held to — a weight change is an ordering change and gets no
//! discount for being a one-line diff.
//!
//! A positive result does NOT flip the default here: an ordering change
//! moves the pinned #421 anchor-accuracy rows and needs `router_reconnect`
//! against a scored R4G1 artifact first (the gate #480 recorded).
//!
//! # Running it
//!
//! ```text
//! cargo test --release -p uor-r4-router --test lexical_weight -- --ignored --nocapture
//! ```
//!
//! About four minutes per arm; no teacher, no checkpoint. Knobs:
//! `R4_HOPF_CONSTR_STORIES` (2,000), `R4_HOPF_PROBES` (500),
//! `R4_CORPUS_META` / `R4_CORPUS_RECS`.

use std::collections::{HashMap, HashSet};

use uor_r4_core::transformerless::compiler;
use uor_r4_router::{UorR4Router, DEFAULT_LEXICAL_WEIGHT};

const ID: &str = "user:lexw";
const TOP_N: usize = 20;
const NULL_MRR_CEILING: f64 = 0.02;
/// Pre-declared #484 exit margin, in MRR over the shipped arm.
const WIN_MARGIN: f64 = 0.05;
/// How much recall@20 a winning arm may give up.
const RECALL_GIVEBACK: f64 = 0.02;

/// #480's PRE-#490 row on this corpus and these caps — the dead-cosine
/// baseline, back when the deployed query was the routing path. Pinned in
/// DEFAULT mode (`content_query = false`): the `W = 100` arm IS that
/// configuration, so anything else means the corpus, the probes or the ranking
/// changed under us and the sweep is void. This is the triple #434/#480/#484
/// each re-derived without noticing it was one measurement three times, so it
/// earns a tight pin as the canonical dead-cosine tell.
const SHIPPED_TOP1: f64 = 0.6240;
const SHIPPED_MRR: f64 = 0.7179;
const SHIPPED_RECALL: f64 = 0.9720;
const REPRODUCTION_TOLERANCE: f64 = 0.0005;

/// The post-#490 content-query row at the EXPLICIT `W = 100` weight, pinned in
/// content-query mode as a reproducibility anchor. Since #502 this is no longer
/// the deployed default (the deployed default dropped to `W = 0`, `CONTENT_W0_*`
/// below); the `W = 100` arm sets the weight explicitly, so this pin still
/// reproduces and guards against the 0.7179 dead-cosine baseline reappearing on
/// the wrong path. Wider tolerance than the dead-cosine pin: this anchor catches
/// a path regression, not a fourth decimal. Re-confirmed on current main in #500.
const CONTENT_MRR: f64 = 0.8542;
const CONTENT_RECALL_MIN: f64 = 0.97;
const CONTENT_TOLERANCE: f64 = 0.02;

/// The DEPLOYED content-query row since #502: dropping the lexical term
/// (`W = 0`, pure `sim * slice_norm` geometry) is worth +0.022 MRR / +0.032
/// top-1 over the `W = 100` row at equal recall@20 (0.99). This is what
/// `get_top_resonances_native` now ranks by at the default weight on the
/// content path (`UorR4Router::default_lexical_weight`). Pinned so a regression
/// in the deployed serving order is caught here, not in production.
const CONTENT_W0_MRR: f64 = 0.8763;
const CONTENT_W0_TOP1: f64 = 0.8160;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// #486/#490: run the sweep with the query vector built from the query text's
/// CONTENT state instead of the routing state — which is the DEPLOYED path
/// since #490. Off by default here only so the default run keeps reproducing
/// the pre-#490 dead-cosine row (`SHIPPED_*`) as the regression tell; the
/// deployed row (`CONTENT_*`) is measured with `R4_LEXW_CONTENT_QUERY=1`.
///
/// This exists because #484's flat sweep was measured with a cosine that
/// #486 then showed to be noise. "The weight does not matter" is CONDITIONAL
/// on that: with a working geometric term the balance between the two terms
/// is a live question again, and the same sweep answers it.
fn content_query_mode() -> bool {
    std::env::var("R4_LEXW_CONTENT_QUERY")
        .map(|v| v.trim() == "1")
        .unwrap_or(false)
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

fn render_query(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens
        .iter()
        .step_by(2)
        .rev()
        .map(|&token| token_word(token))
        .collect();
    words.join(" ")
}

/// Every other token in ORIGINAL order — the shipped probe form with the
/// reversal removed, so subsampling and reordering can be told apart.
fn render_query_ordered(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens
        .iter()
        .step_by(2)
        .map(|&token| token_word(token))
        .collect();
    words.join(" ")
}

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

fn rank_of(results: &[uor_r4_router::ResonanceResult], target_text: &str) -> usize {
    results
        .iter()
        .position(|r| r.sentence == target_text)
        .map(|p| p + 1)
        .unwrap_or(0)
}

/// Fraction of probes whose top-`TOP_N` list holds another candidate in the
/// target's own lexical bucket. Bucket width is `W`, so unlike #480 (where
/// `W` was fixed at 100) this has to be computed per arm — at `W = 0` every
/// candidate is in one bucket and the number is meaningless, which is itself
/// the point: tie-mass measures how much of the ordering the lexical term is
/// deciding, and it should collapse as `W` goes to zero.
fn tie_mass(router_results: &[(Vec<uor_r4_router::ResonanceResult>, String)], weight: f64) -> f64 {
    if weight <= 0.0 {
        return f64::NAN;
    }
    let mut contested = 0usize;
    for (results, target) in router_results {
        let Some(pos) = results.iter().position(|r| &r.sentence == target) else {
            continue;
        };
        let target_relevance = results[pos].relevance;
        let same_bucket = results
            .iter()
            .enumerate()
            .filter(|(i, r)| *i != pos && (r.relevance - target_relevance).abs() < weight)
            .count();
        if same_bucket > 0 {
            contested += 1;
        }
    }
    contested as f64 / router_results.len() as f64
}

/// The spread of the non-lexical part of relevance, over every returned
/// candidate of every probe.
///
/// This is the quantity that makes `W` interpretable and that nobody had
/// measured. "The lexical term is 100x the cosine" was an inference from
/// three observed relevance values and from `sim` being bounded by one — but
/// the cosine is multiplied by `slice_norm`, a per-window scalar whose scale
/// nobody had looked at. `W` dominates only relative to THIS spread, so a
/// sweep without it is uninterpretable.
fn geometric_term_spread(
    results: &[Vec<uor_r4_router::ResonanceResult>],
    weight: f64,
) -> (f64, f64) {
    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for list in results {
        for r in list {
            // Strip the lexical steps: whatever remains is the cosine term
            // plus the 0-or-15 scope boost.
            let residue = if weight > 0.0 {
                r.relevance - (r.relevance / weight).floor() * weight
            } else {
                r.relevance
            };
            low = low.min(residue);
            high = high.max(residue);
        }
    }
    if low.is_finite() {
        (low, high)
    } else {
        (0.0, 0.0)
    }
}

#[test]
#[ignore = "measurement harness (issue #484); run explicitly with --ignored"]
fn lexical_weight_sweep() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let Some(c) = compiler::load_corpus_from(&meta_path, &recs_path) else {
        println!("SKIP: corpus fixtures absent ({meta_path} + {recs_path})");
        return;
    };
    println!("#484: what the lexical weight is worth");
    println!("corpus: {meta_path} + {recs_path} ({} records)", c.n);

    let cut = (c.stories as f64 * 0.8) as u32;
    let constr: Vec<bool> = (0..c.stories).map(|sid| sid < u64::from(cut)).collect();

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
    println!(
        "CAPS: {} stories, {} distinct windows, up to {probe_cap} probes, top_n {TOP_N}",
        capped.len(),
        windows.len()
    );
    assert!(windows.len() > probe_cap, "more windows than probes");

    let stride = (windows.len() / probe_cap).max(1);
    let targets: Vec<usize> = (0..windows.len()).step_by(stride).take(probe_cap).collect();
    let queries: Vec<String> = targets
        .iter()
        .map(|&t| render_query(&window_tokens[t]))
        .collect();
    let deranged: Vec<usize> = (0..targets.len())
        .map(|i| targets[(i + 1) % targets.len()])
        .collect();

    // FULL-LIST DIAGNOSTIC, run before the sweep because it is what makes a
    // zero row readable.
    //
    // A truncated `recall@20` of 0.0000 is ambiguous: it can mean the ranking
    // carries no signal, or that the harness is broken, and this repository's
    // standing rule is that an all-zero arm is a harness bug until proven
    // otherwise. Asking for the WHOLE ranked list resolves it — the target's
    // median position among 46k candidates separates "ranked near random"
    // from "ranked well but outside twenty" from "never retrieved at all".
    //
    // It also gives the ceiling: ranking cannot place a target the candidate
    // set never contained, so containment bounds recall@20 for every arm, and
    // `W` reorders the set without changing what is in it.
    println!(
        "\nFULL-LIST DIAGNOSTIC (all {} candidates ranked)",
        windows.len()
    );
    // The probe forms exist to rule out the alternative explanation. If the
    // cosine ranks the target near random on the shipped probe, that could
    // mean the geometry carries no signal — or it could mean THIS PROBE FORM
    // defeats a geometry that works, since the shipped probe both subsamples
    // (every other token) and reverses. Those have different consequences, so
    // they are separated: `identity` is the stored sentence itself and upper
    // bounds what the geometry can do; `ordered` subsamples without
    // reversing; `shipped` does both.
    let identity_probes: Vec<String> = targets
        .iter()
        .map(|&t| render_window(&window_tokens[t]))
        .collect();
    let ordered_probes: Vec<String> = targets
        .iter()
        .map(|&t| render_query_ordered(&window_tokens[t]))
        .collect();
    println!("ranking\t\t\t\tprobe\t\tcontainment\tmedian rank\tfull-list MRR");
    for (label, weight, unscaled, probes) in [
        (
            "shipped (W=100, ×norm)",
            DEFAULT_LEXICAL_WEIGHT,
            false,
            &queries,
        ),
        ("bare cosine (W=0)", 0.0, true, &queries),
        ("bare cosine (W=0)", 0.0, true, &ordered_probes),
        ("bare cosine (W=0)", 0.0, true, &identity_probes),
    ] {
        let probe_label = if std::ptr::eq(probes, &queries) {
            "shipped"
        } else if std::ptr::eq(probes, &ordered_probes) {
            "ordered"
        } else {
            "identity"
        };
        let mut router = UorR4Router::new(0.5);
        router.set_lexical_weight(weight);
        router.set_unscaled_geometric_term(unscaled);
        router.set_content_query_vector(content_query_mode());
        let corpus_text: String = windows.join(" ");
        let indexed = router.index_corpus(&corpus_text, ID);
        assert_eq!(indexed, windows.len(), "[{label}] indexed every window");
        let mut ranks = Vec::with_capacity(probes.len());
        for (qi, query) in probes.iter().enumerate() {
            let results = router.get_top_resonances_native(query, ID, windows.len());
            ranks.push(rank_of(&results, &windows[targets[qi]]));
        }
        let found: Vec<usize> = ranks.iter().copied().filter(|&r| r > 0).collect();
        let contained = found.len() as f64 / ranks.len() as f64;
        let mut sorted = found.clone();
        sorted.sort_unstable();
        let median = sorted.get(sorted.len() / 2).copied().unwrap_or(0);
        let (_, full_mrr, _) = metrics(&ranks);
        println!("{label:<32}{probe_label:<16}{contained:.4}\t\t{median}\t\t{full_mrr:.4}");
    }
    println!(
        "Random over this candidate set is a median rank near {}. A cosine median near that \
         on the IDENTITY probe would mean the geometry itself carries no retrievable signal; \
         a small median on identity and a random one on shipped would mean the probe form, \
         not the geometry, is what fails. The zero rows below are not interpreted without \
         these lines.",
        windows.len() / 2
    );

    // The sweep. Every arm but the last holds the deployed geometric scaling
    // (`sim * slice_norm`) and moves only `W`.
    //
    // The last arm exists because the first one is CONFOUNDED, and the first
    // run of this harness is what exposed it. `W = 0` scored 0.0000 on every
    // metric — but that is not "cosine ranking loses", because `slice_norm` is
    // a per-window-BUCKET scalar. With the lexical term gone, `sim *
    // slice_norm` compares candidates from different buckets on different
    // scales, so the order it produces is driven by which bucket has the
    // largest slice norm, not by similarity. Reporting that row as the
    // cosine-only comparator — the arm that speaks to #442's 0.8948 — would
    // have been a measurement of the wrong quantity presented as the right
    // one. `cosine` drops the scaling and is the honest comparator.
    let arms: [(f64, bool); 7] = [
        (0.0, false),
        (1.0, false),
        (10.0, false),
        (DEFAULT_LEXICAL_WEIGHT, false),
        (1_000.0, false),
        (100_000.0, false),
        (0.0, true),
    ];
    println!("\nW\t\tgeom\ttop-1\tMRR\trecall@{TOP_N}\ttie-mass\tnull MRR\tgeom-term range");
    let mut summary: HashMap<u64, (f64, f64, f64, f64)> = HashMap::new();
    let mut cosine_row = (0f64, 0f64, 0f64, 0f64);
    for (weight, unscaled) in arms {
        let mut router = UorR4Router::new(0.5);
        router.set_lexical_weight(weight);
        router.set_unscaled_geometric_term(unscaled);
        router.set_content_query_vector(content_query_mode());
        let corpus_text: String = windows.join(" ");
        let indexed = router.index_corpus(&corpus_text, ID);
        assert_eq!(indexed, windows.len(), "[W={weight}] indexed every window");

        let mut ranks = Vec::with_capacity(queries.len());
        let mut null_ranks = Vec::with_capacity(queries.len());
        let mut per_probe = Vec::with_capacity(queries.len());
        let mut raw = Vec::with_capacity(queries.len());
        for (qi, query) in queries.iter().enumerate() {
            let results = router.get_top_resonances_native(query, ID, TOP_N);
            ranks.push(rank_of(&results, &windows[targets[qi]]));
            null_ranks.push(rank_of(&results, &windows[deranged[qi]]));
            raw.push(results.clone());
            per_probe.push((results, windows[targets[qi]].clone()));
        }

        let (top1, mrr, recall) = metrics(&ranks);
        let (_, null_mrr, _) = metrics(&null_ranks);
        let ties = tie_mass(&per_probe, weight);
        let (low, high) = geometric_term_spread(&raw, weight);
        println!(
            "{weight:<10.0}\t{}\t{top1:.4}\t{mrr:.4}\t{recall:.4}\t\t{ties:.4}\t\t{null_mrr:.4}\t\t[{low:.3}, {high:.3}]",
            if unscaled { "bare " } else { "×norm" }
        );
        if unscaled {
            cosine_row = (top1, mrr, recall, null_mrr);
        } else {
            summary.insert(weight.to_bits(), (top1, mrr, recall, null_mrr));
        }
    }

    println!("\n==== validity ====");

    // 1. The null must be dead in every arm, including the ones that are a
    //    different ranking rather than a retuned one.
    let mut worst_null = cosine_row.3;
    for (weight, unscaled) in arms {
        if unscaled {
            continue;
        }
        let (_, _, _, null_mrr) = summary[&weight.to_bits()];
        worst_null = worst_null.max(null_mrr);
    }
    println!(
        "deranged-key MRR, worst arm: {worst_null:.4} (ceiling {NULL_MRR_CEILING}) — {}",
        if worst_null < NULL_MRR_CEILING {
            "PASS"
        } else {
            "VOID"
        }
    );
    assert!(
        worst_null < NULL_MRR_CEILING,
        "deranged-key control must be near zero in EVERY arm; a weight that lifts the \
         null is not measuring retrieval quality"
    );

    // 2. The W=100 arm must BE its mode's pinned configuration, AND the two
    //    baselines must stay distinct — the whole point of #490 is that the
    //    deployed content row sits well above the dead-cosine 0.7179.
    let (s_top1, s_mrr, s_recall, _) = summary[&DEFAULT_LEXICAL_WEIGHT.to_bits()];
    if content_query_mode() {
        println!(
            "W={DEFAULT_LEXICAL_WEIGHT:.0} on the DEPLOYED content-query path: \
             top-1 {s_top1:.4}, MRR {s_mrr:.4} vs pinned {CONTENT_MRR:.4}, \
             recall {s_recall:.4} (>= {CONTENT_RECALL_MIN:.4}); dead-cosine baseline is \
             {SHIPPED_MRR:.4}"
        );
        assert!(
            (s_mrr - CONTENT_MRR).abs() <= CONTENT_TOLERANCE,
            "W={DEFAULT_LEXICAL_WEIGHT:.0} on the content path must reproduce the deployed \
             MRR {CONTENT_MRR:.4} +/- {CONTENT_TOLERANCE:.2}, got {s_mrr:.4}. Either #490 \
             regressed or the corpus/probes moved — either way this sweep says nothing."
        );
        assert!(
            s_recall >= CONTENT_RECALL_MIN,
            "deployed content-path recall must be >= {CONTENT_RECALL_MIN:.4}, got {s_recall:.4}"
        );
        assert!(
            s_mrr >= SHIPPED_MRR + 0.05,
            "the content row ({s_mrr:.4}) must sit clearly above the dead-cosine \
             baseline ({SHIPPED_MRR:.4}); if it does not, the content-vector query has \
             stopped paying and #490 has regressed"
        );
        // #502: the DEPLOYED default on this path is now W=0, not W=100. Pin
        // the row it actually ships (`default_lexical_weight` returns 0 here),
        // and hold it distinct from — and above — the W=100 row it replaced.
        let (w0_top1, w0_mrr, w0_recall, _) = summary[&0.0f64.to_bits()];
        println!(
            "W=0 (DEPLOYED default since #502): top-1 {w0_top1:.4}, MRR {w0_mrr:.4}, \
             recall {w0_recall:.4} vs pinned {CONTENT_W0_MRR:.4}/{CONTENT_W0_TOP1:.4}; \
             W=100 was {s_mrr:.4} MRR ({:+.4})",
            w0_mrr - s_mrr
        );
        assert!(
            (w0_mrr - CONTENT_W0_MRR).abs() <= CONTENT_TOLERANCE,
            "the deployed W=0 content row must reproduce MRR {CONTENT_W0_MRR:.4} +/- \
             {CONTENT_TOLERANCE:.2}, got {w0_mrr:.4}; the #502 serving order regressed"
        );
        assert!(
            w0_mrr >= s_mrr,
            "dropping the lexical term (W=0, #502) must not rank BELOW W=100 on the \
             content path: W=0 {w0_mrr:.4} vs W=100 {s_mrr:.4}"
        );
        assert!(
            w0_recall >= CONTENT_RECALL_MIN,
            "deployed W=0 recall must be >= {CONTENT_RECALL_MIN:.4}, got {w0_recall:.4}"
        );
    } else {
        println!(
            "W={DEFAULT_LEXICAL_WEIGHT:.0} reproduces #480's PRE-#490 dead-cosine row: \
             top-1 {s_top1:.4} vs {SHIPPED_TOP1:.4}, MRR {s_mrr:.4} vs {SHIPPED_MRR:.4}, \
             recall {s_recall:.4} vs {SHIPPED_RECALL:.4}"
        );
        for (label, got, want) in [
            ("top-1", s_top1, SHIPPED_TOP1),
            ("MRR", s_mrr, SHIPPED_MRR),
            ("recall@20", s_recall, SHIPPED_RECALL),
        ] {
            assert!(
                (got - want).abs() <= REPRODUCTION_TOLERANCE,
                "W={DEFAULT_LEXICAL_WEIGHT:.0} must reproduce #480's pre-#490 {label} \
                 ({want:.4}), got {got:.4}. Either the routing-path default moved or the \
                 corpus/probes moved — either way this sweep says nothing about the weight."
            );
        }
    }

    // 3. The cosine-only comparator, reported before the verdict because it is
    //    what decides whether the verdict is interesting. If bare-cosine
    //    ranking is competitive, a flat weight sweep means the lexical term is
    //    redundant; if it collapses, the flat sweep means the geometry has
    //    nothing to contribute on this path at this scale, which is a
    //    different conclusion with a different next action.
    let (c_top1, c_mrr, c_recall, _) = cosine_row;
    println!(
        "cosine-only comparator (W=0, bare `sim`, no slice_norm): \
         top-1 {c_top1:.4}, MRR {c_mrr:.4}, recall@{TOP_N} {c_recall:.4}. \
         This is the arm that speaks to #442's 0.8948 — the W=0 row above does NOT, \
         because `slice_norm` is per-bucket and makes the scaled term incomparable \
         across buckets."
    );

    println!("\n==== verdict against the pre-declared exit rule ====");
    let mut best: Option<(f64, f64, f64, f64)> = None;
    for (weight, unscaled) in arms {
        if unscaled || weight == DEFAULT_LEXICAL_WEIGHT {
            continue;
        }
        let (top1, mrr, recall, _) = summary[&weight.to_bits()];
        if best.is_none_or(|(_, best_mrr, _, _)| mrr > best_mrr) {
            best = Some((weight, mrr, recall, top1));
        }
    }
    let (best_w, best_mrr, best_recall, best_top1) =
        best.expect("the sweep has at least one non-shipped arm");
    let delta_mrr = best_mrr - s_mrr;
    let delta_recall = best_recall - s_recall;
    println!(
        "best non-shipped arm W={best_w:.0}: {best_mrr:.4} MRR, {best_top1:.4} top-1, \
         {best_recall:.4} recall  vs  shipped {s_mrr:.4} / {s_top1:.4} / {s_recall:.4}  =  \
         {delta_mrr:+.4} MRR, {delta_recall:+.4} recall \
         [exit rule: >= +{WIN_MARGIN:.2} MRR and >= -{RECALL_GIVEBACK:.2} recall]"
    );

    if delta_mrr >= WIN_MARGIN && delta_recall >= -RECALL_GIVEBACK {
        println!(
            "POSITIVE: the shipped weight is not the right one — W={best_w:.0} is worth \
             {delta_mrr:+.4} MRR. DO NOT flip DEFAULT_LEXICAL_WEIGHT on this result alone: \
             changing the weight changes retrieval ORDERING, which moves the pinned #421 \
             anchor-accuracy rows, and `router_reconnect` against a scored R4G1 artifact is \
             the gate #480 recorded for exactly this. File the adoption issue."
        );
    } else {
        println!(
            "NEGATIVE against the exit rule: the best alternative weight moves MRR \
             {delta_mrr:+.4} and recall {delta_recall:+.4}. The ranking is INSENSITIVE to a \
             weight spanning decades, which retires 'the lexical term is suppressing the \
             geometry' as a live explanation rather than leaving it as folklore. The \
             cross-harness gap between #442's 0.8948 (cosine-only) and this path's \
             {s_mrr:.4} is therefore NOT the lexical term's doing, and vector-shape work on \
             this path should not be re-proposed on that premise."
        );
    }
}
