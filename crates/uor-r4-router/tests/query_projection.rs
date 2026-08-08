//! Query-projection shape vs retrieval quality (issue #480).
//!
//! # The asymmetry
//!
//! PR #465 adopted full-width content-bearing storage: `banded_storage`
//! defaults to false and the stored vector keeps all 512 coefficients. The
//! query side did not follow — `retrieve_geometric_resonance` built a
//! band-only projection (zero outside `active_range`) regardless. Zeroed
//! query coordinates contribute nothing to a dot product, so the cosine only
//! ever saw the band no matter how wide the stored vector was, and the
//! adopted de-banding gain (retrieval MRR 0.2348 -> 0.8948, router anchor
//! accuracy 9.3% -> 11.4%, #442) could not reach this path.
//!
//! The fix makes the query projection obey the same rule storage obeys, so
//! the two shapes are symmetric by construction rather than by coincidence.
//!
//! # What this harness can and cannot show
//!
//! Relevance in that function is
//!
//! ```text
//! relevance = shared_count * 100 + sim * slice_norm + scope_boost
//! ```
//!
//! `shared_count` is the number of query primes present in the item, times
//! **100**; `sim` is bounded by 1 and `slice_norm` is a band-slice norm. The
//! ranking is therefore dominated by lexical overlap, and the cosine acts as
//! a **tie-breaker within equal-overlap groups**. A query-shape change can
//! only reorder candidates that are tied on `shared_count`; it cannot rescue
//! a target the lexical term has already placed below others, and it cannot
//! improve recall, which was already 0.9720 at these caps (#479).
//!
//! So the honest expectation is a small MRR gain concentrated in top-1, not a
//! move toward the 0.8948 figure from #442 — that number came from a harness
//! ranking by cosine ALONE, with no lexical term. This harness reports the
//! tie-mass explicitly so the size of the reachable slice is visible next to
//! the result rather than inferred from it.
//!
//! # Arms
//!
//! The projection lever this issue is about — band-only vs full-width QUERY —
//! only exists on the `content_query = false` path. Since #490, `new()` defaults
//! `content_query = true`, which builds the query from the content vector and
//! makes `set_full_width_query` a no-op. The first three arms therefore set
//! `content_query = false` explicitly; without that they would all collapse to
//! the content-vector query and the shipped-vs-symmetric verdict would be a
//! vacuous `+0.0000`.
//!
//! - `banded` — `set_banded_storage(true)`: banded store, banded query. The
//!   pre-#465 world.
//! - `shipped` — full store, band-only query. The PRE-#490 deployed default,
//!   and the asymmetry the issue was originally about.
//! - `symmetric` — full store, full-width query via
//!   `set_full_width_query(true)`: #480's proposed fix.
//! - `content (deployed)` — `content_query = true`: what actually ships since
//!   #490. The query is the content vector, full-width by construction and the
//!   same KIND of object as the stored vector (which the projected query never
//!   was, #486). The other three are read against this one.
//!
//! # What #490 did to this question
//!
//! #480's original verdict was NEGATIVE: making the shapes symmetric was worth
//! only +0.0059 MRR against a +0.05 bar. #486 then showed why — the cosine was
//! at chance because the query (routing path) and the stored vector (content)
//! were different objects, so no query SHAPE could pay. #490 fixed the object by
//! building the query from the content vector, which is full-width by
//! construction. That is the shape #480 was reaching for, and it pays about
//! +0.136 MRR over the band-only query. So "shape does not pay" is SUPERSEDED:
//! the lever was real; it was mis-measured because the query was the wrong kind
//! of object, not the wrong shape. This harness now shows both the flat
//! projection arms and the deployed content arm side by side.
//!
//! # Corpus, probes, null
//!
//! The #422/#423 laws, unchanged, as in `geometry_ablation.rs`: D3 token
//! corpus, synthetic word rendering, eight-token windows deduplicated to
//! first occurrence, probes in the held-out even-offset-reversed form, and a
//! deranged answer key (probe `i` graded against target `i + 1 (mod n)`) as
//! the falsifier. Ground truth is matched on the stored SENTENCE, never on
//! `ResonanceResult::window_index`, which is a routing-window identifier
//! shared by several stored sentences.
//!
//! # Pre-declared exit rule
//!
//! - The deranged-key null must be near zero in both arms, or the harness is
//!   void.
//! - Positive if `full` beats `banded` by >= 0.05 MRR.
//!
//! Recorded outcome: `docs/query_projection_480.md`.

use std::collections::{HashMap, HashSet};

use uor_r4_core::transformerless::compiler;
use uor_r4_router::UorR4Router;

const ID: &str = "user:qproj";
const TOP_N: usize = 20;
const NULL_MRR_CEILING: f64 = 0.02;
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

fn render_query(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens
        .iter()
        .step_by(2)
        .rev()
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

/// Fraction of probes whose returned list contains at least one OTHER result
/// within a hair of the target's relevance — i.e. tied on the lexical term,
/// where the cosine is the only thing that can reorder. This is the slice a
/// query-shape change can act on, and it bounds any gain reported below.
fn tie_mass(router_results: &[(Vec<uor_r4_router::ResonanceResult>, String)]) -> f64 {
    let mut contested = 0usize;
    for (results, target) in router_results {
        let Some(pos) = results.iter().position(|r| &r.sentence == target) else {
            continue;
        };
        let target_relevance = results[pos].relevance;
        // The lexical term moves in steps of 100; anything inside one step of
        // the target is in the same overlap bucket.
        let same_bucket = results
            .iter()
            .enumerate()
            .filter(|(i, r)| *i != pos && (r.relevance - target_relevance).abs() < 100.0)
            .count();
        if same_bucket > 0 {
            contested += 1;
        }
    }
    contested as f64 / router_results.len() as f64
}

#[test]
#[ignore = "measurement harness (issue #480); run explicitly with --ignored"]
fn query_projection_shape() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let Some(c) = compiler::load_corpus_from(&meta_path, &recs_path) else {
        println!("SKIP: corpus fixtures absent ({meta_path} + {recs_path})");
        return;
    };
    println!("#480: query-projection shape vs retrieval quality");
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

    println!("\narm\t\t\tstore/query\ttop-1\tMRR\trecall@{TOP_N}\ttie-mass\tnull MRR");
    let mut summary: HashMap<&str, (f64, f64, f64, f64, f64)> = HashMap::new();
    // The #480 projection lever — band-only vs full-width QUERY — lives only on
    // the `content_query = false` path. Since #490, `new()` defaults
    // `content_query = true`, which builds the query from the content vector and
    // makes `set_full_width_query` a no-op; without setting the flag here all
    // three projection arms would collapse to one measurement and the
    // shipped-vs-symmetric verdict would be a vacuous +0.0000. It is set
    // explicitly so each arm means what its name says.
    //
    // The `content (deployed)` arm is what actually ships after #490: the query
    // is the content vector, full-width by construction AND the same KIND of
    // object as the stored vector (which the projected query never was, #486).
    // The other three should be read against it — it is the realized answer to
    // the asymmetry this issue was about, reached by a better route than
    // reshaping a query that was never comparable in the first place.
    for (arm, banded, full_query, content) in [
        ("banded", true, false, false),
        ("shipped", false, false, false),
        ("symmetric", false, true, false),
        ("content (deployed)", false, false, true),
    ] {
        let mut router = UorR4Router::new(0.5);
        router.set_banded_storage(banded);
        router.set_full_width_query(full_query);
        router.set_content_query_vector(content);
        let corpus_text: String = windows.join(" ");
        let indexed = router.index_corpus(&corpus_text, ID);
        assert_eq!(indexed, windows.len(), "[{arm}] indexed every window");

        let mut ranks = Vec::with_capacity(queries.len());
        let mut null_ranks = Vec::with_capacity(queries.len());
        let mut per_probe = Vec::with_capacity(queries.len());
        for (qi, query) in queries.iter().enumerate() {
            // Identity-scoped retrieval: query under the identity the corpus
            // was indexed with, or the index is empty and every arm scores 0.
            let results = router.get_top_resonances_native(query, ID, TOP_N);
            ranks.push(rank_of(&results, &windows[targets[qi]]));
            null_ranks.push(rank_of(&results, &windows[deranged[qi]]));
            per_probe.push((results, windows[targets[qi]].clone()));
        }

        let (top1, mrr, recall) = metrics(&ranks);
        let (_, null_mrr, _) = metrics(&null_ranks);
        let ties = tie_mass(&per_probe);
        println!(
            "{arm:<19}{}\t{top1:.4}\t{mrr:.4}\t{recall:.4}\t\t{ties:.4}\t\t{null_mrr:.4}",
            match (banded, full_query, content) {
                (_, _, true) => "full/content ",
                (true, _, false) => "banded/banded",
                (false, false, false) => "full/banded  ",
                (false, true, false) => "full/full    ",
            }
        );
        summary.insert(arm, (top1, mrr, recall, ties, null_mrr));
    }

    let (b_top1, b_mrr, b_recall, b_ties, b_null) = summary["shipped"];
    let (f_top1, f_mrr, f_recall, _, f_null) = summary["symmetric"];
    let (c_top1, c_mrr, c_recall, _, c_null) = summary["content (deployed)"];

    println!("\n==== validity ====");
    let null_dead =
        b_null < NULL_MRR_CEILING && f_null < NULL_MRR_CEILING && c_null < NULL_MRR_CEILING;
    println!(
        "deranged-key MRR: banded-query {b_null:.4}, full-query {f_null:.4}, \
         content-query {c_null:.4} (ceiling {NULL_MRR_CEILING}) — {}",
        if null_dead { "PASS" } else { "VOID" }
    );
    assert!(null_dead, "deranged-key control must be near zero");

    println!(
        "\nreachable slice: {:.1}% of probes have another candidate inside the target's \
         lexical bucket, which is where a query-shape change can act at all",
        b_ties * 100.0
    );

    let delta_mrr = f_mrr - b_mrr;
    let delta_top1 = f_top1 - b_top1;
    println!("\n==== verdict against the pre-declared exit rule ====");
    let delta_recall = f_recall - b_recall;
    println!(
        "shipped (full/banded) {b_mrr:.4} MRR, {b_top1:.4} top-1, {b_recall:.4} recall  ->  \
         symmetric (full/full) {f_mrr:.4} MRR, {f_top1:.4} top-1, {f_recall:.4} recall  =  \
         {delta_mrr:+.4} MRR, {delta_top1:+.4} top-1, {delta_recall:+.4} recall \
         [exit rule: >= +{WIN_MARGIN:.2} MRR]"
    );
    if delta_mrr >= WIN_MARGIN {
        println!(
            "POSITIVE: matching the query shape to the storage shape is worth \
             {delta_mrr:+.4} MRR. Re-run the #421 reconnection and memory-lift gates \
             that de-banding was adopted against before treating this as shipped."
        );
    } else {
        println!(
            "NEGATIVE against the exit rule: {delta_mrr:+.4} MRR, and recall moves \
             {delta_recall:+.4}. The asymmetry is real but fixing it does not pay on this \
             path, so the symmetric shape stays behind `set_full_width_query` and the \
             deployed default is unchanged. The 0.8948 de-banding figure was never going \
             to appear here: that harness ranked by cosine ALONE, while this path adds a \
             lexical term worth 100x the cosine, so the vector shape only reorders \
             candidates already tied on word overlap. The premise that a measured gain \
             was going unrealized at serving is REFUTED — not because de-banding failed, \
             but because this ranking is lexical, not geometric."
        );
    }

    // The reconciliation the projection arms cannot show on their own. The #480
    // question — "does matching the query shape to the storage shape pay?" — was
    // answered NO above, but only on the path where the cosine is at chance
    // (#486): reshaping a query vector that is not the same KIND of object as the
    // stored vector cannot pay at any shape. #490 fixed the object, not the
    // shape, by building the query from the content vector. That query is
    // full-width by construction — the shape #480 was reaching for — and it is
    // what ships. This arm is the realized answer.
    let content_gain_mrr = c_mrr - b_mrr;
    let content_gain_recall = c_recall - b_recall;
    println!("\n==== reconciliation: the deployed content-vector query (#490) ====");
    println!(
        "shipped band-query {b_mrr:.4} MRR, {b_top1:.4} top-1, {b_recall:.4} recall  ->  \
         content query (deployed) {c_mrr:.4} MRR, {c_top1:.4} top-1, {c_recall:.4} recall  =  \
         {content_gain_mrr:+.4} MRR, {content_gain_recall:+.4} recall"
    );
    println!(
        "The full-width query DID pay — as the content vector via #490, not via \
         `set_full_width_query`. #480's 'shape does not pay' verdict is SUPERSEDED: the \
         lever was real and was mis-measured because the query was the wrong kind of \
         object, not the wrong shape. The band-vs-full projection arms remain flat \
         ({delta_mrr:+.4} MRR) because on the content-query path they are all overridden \
         by the content vector; they are retained here as the pre-#490 diagnostic that \
         localised the problem to the object, not the shape.",
        delta_mrr = f_mrr - b_mrr
    );
    assert!(
        content_gain_mrr >= WIN_MARGIN,
        "the deployed content-vector query must beat the band-query shipped arm by at least \
         the {WIN_MARGIN:.2} MRR exit margin (it measured about +0.136 at #490); if it does \
         not, #490 has regressed or this corpus has moved"
    );
}
