//! Zeta-grid state as retrieval vector (issue #434, arm Z): the first
//! natural-corpus measurement of whether the router's 512-dimensional
//! zeta-zero-grid SESSION STATE — not the content-derived index vector —
//! carries retrieval-relevant structure. Every prior retrieval number on
//! this stack (the #423 base MRR of 0.2348, the #422 filtered-arm
//! frontier of 0.0743) ranks stored windows by cosine over
//! content-derived vectors; the evolved state the router actually routes
//! with has never been used as the retrieval key. This harness measures
//! exactly that.
//!
//! # Corpus, mapping, split, probes (the #422/#423 laws, unchanged)
//!
//! Same natural stack as `hopf_retrieval_quality.rs`: the D3 token
//! corpus via `R4_CORPUS_META` / `R4_CORPUS_RECS`, token ids rendered as
//! synthetic words (`t00042`), the D3 hash partition when `R4_STORIES`
//! is set (else the sequential eighty/twenty story cut), consecutive
//! non-overlapping eight-token windows deduplicated to first occurrence,
//! probe queries in the held-out #423 form (a target window's
//! even-offset tokens in reverse order; the query string is never
//! ingested). Caps are the SAME env-overridable knobs as that harness —
//! `R4_HOPF_CONSTR_STORIES` (default two thousand construction stories)
//! and `R4_HOPF_PROBES` (default five hundred probes) — and both print.
//!
//! # The zeta-grid state and the API that exposes it
//!
//! The router seeds every vocabulary word on the zeta-zero grid
//! (`uor_r4_core::get_word_vector`: coordinate `i` is
//! `sin(ln(prime) * ZETA_ZEROS[i])` over the first 512 non-trivial
//! Riemann zeta zeros, L2-normalized), and a session's state evolves
//! through `evolve_state(identity, text, gamma)` — the public wrapper
//! over `evolve_brain_state`, which blends the normalized zeta word-sum
//! of the text into the identity's stored session state
//! (`gamma * state + (1 - gamma) * content`, renormalized) and returns
//! the new 512-d state. That stored session state is PRECISELY the
//! vector `route_query_to_manifold_native_with_hopf_input` consumes as
//! `active_state` when no override is supplied (the
//! `session_brain_states` entry for the identity), so `evolve_state`'s
//! return value is the most direct public surface onto the routed
//! zeta-grid state; the harness additionally asserts it equals
//! `get_brain_state_native` for the same identity, tying the measured
//! vector to the consumed one.
//!
//! # Arm Z construction (deterministic; no session carry-over)
//!
//! For each stored window AND each probe query, the zeta-grid state is
//! computed by driving the router's actual state evolution over that
//! text under a FRESH per-text identity: one `evolve_state(identity,
//! text, GAMMA)` call from the default uniform start — the same
//! evolve-then-route sequence as `src/server.rs` POST /api/chat and the
//! \#303 occupancy harness, at that harness's fixed `GAMMA` of 0.85
//! (the server autotunes gamma per request, which would make runs
//! non-comparable). A retrieval key must be a stable function of
//! content, so no state is shared across texts. Determinism: word-prime
//! assignment is fixed by ingestion order (`index_corpus` runs before
//! any evolution, and probe queries contain no novel words by the #423
//! query law), `get_word_vector` is a pure function of the prime, and
//! the single-step evolution touches no RNG and no HashMap iteration
//! order. Under this law the state has the closed form
//! `normalize(gamma * u0 + (1 - gamma) * content)` with `u0` the
//! uniform default — a contraction of the content vector toward the
//! shared default direction. Cosine RANKINGS are not preserved under
//! adding a common vector, so arm Z is a real measurement of how much
//! retrieval structure survives the gamma-contraction the production
//! state applies, not a tautology; the closed form is stated so the
//! result is interpretable either way.
//!
//! # Arms and metrics (identical to the #422 harness's base metrics)
//!
//! - BASELINE — content vectors: query vectors through the router's own
//!   indexing surface (scratch identities), cosine over stored
//!   content-derived state vectors, stable descending sort, ties by
//!   ascending window index. This is the #423 base retrieval (reference
//!   MRR 0.2348 at the default caps on the D3 stack).
//! - ARM Z — zeta-grid state: identical ranking law, but query AND
//!   stored vectors are the evolved zeta-grid states above.
//! - CONTROL — shuffled: arm Z's STORED states rotated across window
//!   positions by the fixed half-length rotation (the #423/#422 control
//!   construction); query states unrotated. Same vector population,
//!   state-content correspondence destroyed.
//!
//! Per arm: MRR and top-1. Structural invariants gate; direction prints
//! (the #423 convention).
//!
//! # Pre-declared exit rule
//!
//! The zeta-grid state carries retrieval-relevant structure if and only
//! if arm Z's MRR beats the shuffled control AND reaches at least HALF
//! the content-vector baseline MRR measured in the same run. The result
//! is reported regardless of direction.
//!
//! # Band-matched controls (issue #434 follow-up; the caveat-killer)
//!
//! The full-cap arm Z result (MRR of 0.8932 against the content
//! baseline's 0.2348) has a confound: the stored content vectors the
//! baseline ranks over are NOT full-width. The storage surface
//! (`index_sentence_routed`, the #245 path) routes each sentence
//! through the spectral zeta QR projection and stores only the winning
//! window's coefficient magnitudes inside that window's channel range
//! (`active_range`), zeros elsewhere — so the baseline compares
//! band-sparse vectors while arm Z compares full 512-d states. Two
//! arms make the comparison like-for-like:
//!
//! - ARM ZB — Z-banded: each zeta-grid state zeroed outside the SAME
//!   `active_range` its text's stored content vector received, then
//!   renormalized. The band is recovered through the public surface:
//!   `evolve_state(fresh identity, text, gamma 0.0)` sets the session
//!   state to exactly the normalized zeta word-sum — the very content
//!   vector the #245 override supplies at index time — so routing
//!   under that identity reproduces the index-time window choice.
//!   Parity with the storage surface is asserted per text: the derived
//!   band is one of the sixteen zeta window channel ranges, the stored
//!   vector's support lies inside it, and the re-routed in-band
//!   coefficients match the stored ones. Note the stored baseline
//!   vectors additionally carry the QR coefficient transform inside
//!   the band; arm ZB matches band SUPPORT — the sparsity confound
//!   under test — not the transform (arm CF covers the other
//!   direction).
//! - ARM CF — content-full: the content-derived vector WITHOUT band
//!   slicing, for stores and queries alike, constructed through the
//!   same public surface (`evolve_state` at gamma zero IS the
//!   full-width normalized zeta word-sum). If CF matches arm Z, the
//!   arm-Z-versus-baseline gap was vector fullness, not anything the
//!   gamma-contracted state adds.
//!
//! Every arm (baseline included) reports its own shuffled control —
//! that arm's stored set under the fixed half-length rotation, queries
//! unrotated. Pre-declared band-matched rule: the zeta claim SURVIVES
//! iff arm ZB's MRR is at least HALF of arm Z's same-run MRR (at the
//! merged full-cap reference, half of 0.8932 is 0.4466) AND still
//! exceeds the content baseline's same-run MRR; otherwise the arm Z
//! win is re-attributed to banding. Reported regardless.
//!
//! Gate: the measurement runs only under `R4_ZETA_ARM=1` (else the test
//! skips vacuously with a printed notice, the `R4_HOPF_REDESIGN`
//! pattern). Run (natural stack):
//!   R4_ZETA_ARM=1 \
//!   R4_CORPUS_META=/tmp/c_meta.bin R4_CORPUS_RECS=/tmp/c_recs.bin \
//!   R4_STORIES=/tmp/wiki-obs/stories.jsonl \
//!   cargo test --release -p uor-r4-router --test zeta_state_retrieval -- \
//!   --ignored --nocapture

use std::collections::{HashMap, HashSet};

use uor_r4_core::transformerless::compiler;
use uor_r4_core::zeta_projection::window_ranges;
use uor_r4_router::UorR4Router;

/// Identity scope of the corpus store (the #422 harness convention).
const ID: &str = "user:zetarq";
/// Fixed session-evolution gain (the #303 occupancy-harness value; the
/// server autotunes gamma, which would make runs non-comparable).
const GAMMA: f64 = 0.85;

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

/// Token id rendered as the synthetic router word (module docs).
fn token_word(token: u32) -> String {
    format!("t{token:05}")
}

/// An eight-token window rendered as the stored sentence form.
fn render_window(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens.iter().map(|&token| token_word(token)).collect();
    format!("{}.", words.join(" "))
}

/// The held-out probe query for a target window: even-offset tokens in
/// reverse order (the #423 query law).
fn render_query(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens
        .iter()
        .step_by(2)
        .rev()
        .map(|&token| token_word(token))
        .collect();
    words.join(" ")
}

fn cosine(a: &[f64], b: &[f64], norm_a: f64, norm_b: f64) -> f64 {
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    dot / (norm_a * norm_b)
}

fn norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

/// Unfiltered rank of `target` (the #423 base retrieval: stable
/// descending cosine, ties by ascending window index).
fn base_rank(sims: &[f64], target: usize) -> usize {
    let target_sim = sims[target];
    let mut rank = 1usize;
    for (position, &sim) in sims.iter().enumerate() {
        if sim > target_sim || (sim == target_sim && position < target) {
            rank += 1;
        }
    }
    rank
}

/// (top-1 hit rate, MRR) over unfiltered ranks.
fn rank_metrics(ranks: &[usize]) -> (f64, f64) {
    let n = ranks.len() as f64;
    let hits = ranks.iter().filter(|&&rank| rank == 1).count() as f64;
    let mrr: f64 = ranks.iter().map(|&rank| 1.0 / rank as f64).sum();
    (hits / n, mrr / n)
}

/// Similarities of one query vector against an ordered store.
fn sims_against(query: &[f64], store: &[&[f64]], store_norms: &[f64]) -> Vec<f64> {
    let query_norm = norm(query);
    store
        .iter()
        .zip(store_norms)
        .map(|(v, &vn)| cosine(query, v, query_norm, vn))
        .collect()
}

/// Content-derived query vectors through the router's own indexing
/// surface (scratch identities — the #255/#423 pattern).
fn query_vectors(router: &mut UorR4Router, queries: &[String]) -> Vec<Vec<f64>> {
    queries
        .iter()
        .enumerate()
        .map(|(qi, q)| {
            let scratch = format!("user:zq{qi}");
            router.index_sentence(q, &scratch);
            let items = router.corpus_items_for(&scratch);
            assert_eq!(items.len(), 1, "one stored item per probe query");
            items[0].state_vector.clone()
        })
        .collect()
}

/// Stored content state vectors re-ordered into window order (sentence
/// text is the key; store iteration order is HashMap-dependent).
fn aligned_vectors<'a>(router: &'a UorR4Router, windows: &[String]) -> Vec<&'a [f64]> {
    let items = router.corpus_items_for(ID);
    assert_eq!(
        items.len(),
        windows.len(),
        "store holds every deduplicated window (no silent truncation)"
    );
    let by_sentence: HashMap<&str, &[f64]> = items
        .iter()
        .map(|item| (item.sentence.as_str(), item.state_vector.as_slice()))
        .collect();
    windows
        .iter()
        .map(|w| {
            *by_sentence
                .get(w.as_str())
                .expect("every window is stored under its own text")
        })
        .collect()
}

/// The zeta-grid state for one text: the router's actual state
/// evolution driven over the text under a fresh identity (module docs).
/// Asserts the returned vector is the stored session state — the exact
/// vector the routing surface would consume for this identity — and
/// that it moved off the default uniform start (every word is known).
fn zeta_state(router: &mut UorR4Router, identity: &str, text: &str) -> Vec<f64> {
    let state = router.evolve_state(identity, text, GAMMA);
    assert_eq!(state.len(), 512, "zeta-grid state is 512-dimensional");
    assert_eq!(
        router.get_brain_state_native(identity),
        state,
        "evolve_state returns the stored session state (the routing input)"
    );
    let default_component = 1.0 / (512.0f64).sqrt();
    assert!(
        state
            .iter()
            .any(|&value| (value - default_component).abs() > 1e-12),
        "state must move off the default start: no word of {text:?} is in vocabulary"
    );
    state
}

/// A text's storage band: the channel range (start, end) of the zeta
/// window its stored content vector occupies.
type Band = (usize, usize);

/// The full-width content vector and storage band for one text (module
/// docs, band-matched controls). `evolve_state` at gamma zero sets the
/// fresh identity's session state to exactly the normalized zeta
/// word-sum — the content vector the #245 override supplies at index
/// time — so routing under that identity reproduces the index-time
/// window choice, and `active_range` is the band the stored vector
/// received. Parity with the storage surface is asserted: the band is
/// one of the sixteen zeta window channel ranges, the stored vector is
/// zero outside it, and the re-routed in-band coefficients match the
/// stored ones (loose float tolerance: the index-time content vector
/// and the gamma-zero evolved state differ only by a second
/// renormalization, and grounding casts through f32 either way).
fn content_full_and_band(
    router: &mut UorR4Router,
    identity: &str,
    text: &str,
    stored: &[f64],
) -> (Vec<f64>, Band) {
    let content_full = router.evolve_state(identity, text, 0.0);
    assert_eq!(content_full.len(), 512, "content vector is 512-dimensional");
    let routing = router.route_query_to_manifold_native(text, identity);
    let start = routing.routed.active_range[0] as usize;
    let end = routing.routed.active_range[1] as usize;
    assert!(start < end && end <= 512, "well-formed active_range");
    assert!(
        window_ranges().contains(&(start, end)),
        "active_range must be one of the sixteen zeta window channel ranges"
    );
    for (index, &value) in stored.iter().enumerate() {
        assert!(
            (start..end).contains(&index) || value == 0.0,
            "stored content vector must be zero outside its band \
             (index {index}, band {start}..{end})"
        );
    }
    let in_band = &routing.routed.state_vector;
    assert_eq!(in_band.len(), end - start, "routed slice spans the band");
    for (routed_value, stored_value) in in_band.iter().zip(&stored[start..end]) {
        assert!(
            (routed_value - stored_value).abs() < 1e-5,
            "re-routed banded coefficients match the storage surface \
             ({routed_value} vs {stored_value})"
        );
    }
    (content_full, (start, end))
}

/// Arm ZB law (module docs): zero the state outside the band, then
/// renormalize.
fn banded(state: &[f64], band: Band) -> Vec<f64> {
    let (start, end) = band;
    let mut v = vec![0.0; state.len()];
    v[start..end].copy_from_slice(&state[start..end]);
    let band_norm = norm(&v);
    assert!(
        band_norm > 1e-12,
        "banded state must be nonzero inside its band"
    );
    for value in &mut v[start..end] {
        *value /= band_norm;
    }
    v
}

/// Base and shuffled-control ranks for one arm: cosine of each probe
/// query vector against the ordered store (base) and against the store
/// rotated across window positions by `rotation` (the #423/#422
/// control construction; queries unrotated).
fn arm_ranks(
    query_vecs: &[Vec<f64>],
    store: &[Vec<f64>],
    targets: &[usize],
    rotation: usize,
) -> (Vec<usize>, Vec<usize>) {
    let n = store.len();
    let store_refs: Vec<&[f64]> = store.iter().map(|v| v.as_slice()).collect();
    let norms: Vec<f64> = store_refs.iter().map(|v| norm(v)).collect();
    let rotated: Vec<&[f64]> = (0..n).map(|p| store_refs[(p + rotation) % n]).collect();
    let rotated_norms: Vec<f64> = (0..n).map(|p| norms[(p + rotation) % n]).collect();
    let mut base = Vec::with_capacity(targets.len());
    let mut ctrl = Vec::with_capacity(targets.len());
    for (probe, &t) in targets.iter().enumerate() {
        let sims = sims_against(&query_vecs[probe], &store_refs, &norms);
        base.push(base_rank(&sims, t));
        let sims_ctrl = sims_against(&query_vecs[probe], &rotated, &rotated_norms);
        ctrl.push(base_rank(&sims_ctrl, t));
    }
    (base, ctrl)
}

#[test]
#[ignore = "issue #434 measurement harness; run explicitly with --ignored"]
fn zeta_grid_state_as_retrieval_vector() {
    // ---- gate (module docs) ----
    if !std::env::var("R4_ZETA_ARM").is_ok_and(|value| value == "1") {
        println!("SKIP: zeta-grid state measurement runs only under R4_ZETA_ARM=1");
        return;
    }

    // ---- natural stack ----
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let c = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    println!("corpus: {meta_path} + {recs_path} ({} records)", c.n);

    // ---- split: D3 hash partition when R4_STORIES is set, else the
    // sequential eighty/twenty story cut (anchor_infill.rs law) ----
    let cut = (c.stories as f64 * 0.8) as u32;
    let constr: Vec<bool> = match std::env::var("R4_STORIES") {
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
    };

    // ---- per-story token streams (stream index zero = first input) ----
    let mut streams: Vec<(u32, Vec<u32>)> = Vec::new();
    for ((&sid, &input), &next) in c.story.iter().zip(&c.input).zip(&c.next).take(c.n) {
        if streams.last().map(|(last, _)| *last) != Some(sid) {
            streams.push((sid, vec![input]));
        }
        let (_, stream) = streams.last_mut().expect("just pushed");
        stream.push(next);
    }

    // ---- CAPS (module docs; the #422 knobs; printed, never silent) ----
    let story_cap = env_usize("R4_HOPF_CONSTR_STORIES", 2_000);
    let probe_cap = env_usize("R4_HOPF_PROBES", 500).max(1);
    let constr_total = streams.iter().filter(|(s, _)| constr[*s as usize]).count();
    let capped: Vec<&(u32, Vec<u32>)> = streams
        .iter()
        .filter(|(sid, _)| constr[*sid as usize])
        .take(story_cap)
        .collect();
    println!(
        "CAPS: ingesting {} of {} construction stories; up to {} probes \
         (R4_HOPF_CONSTR_STORIES / R4_HOPF_PROBES override)",
        capped.len(),
        constr_total,
        probe_cap
    );

    // ---- eight-token windows, deduplicated to first occurrence ----
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

    // ---- probes: evenly strided targets, held-out query form ----
    let stride = (windows.len() / probe_cap).max(1);
    let targets: Vec<usize> = (0..windows.len()).step_by(stride).take(probe_cap).collect();
    let queries: Vec<String> = targets
        .iter()
        .map(|&t| render_query(&window_tokens[t]))
        .collect();
    println!(
        "probes: {} targets, window stride {stride}, query = even-offset tokens reversed",
        targets.len()
    );

    // ---- one store, production bulk ingestion (#423 pattern); the
    // vocabulary (word-prime order, zeta word vectors) is fixed HERE,
    // before any state evolution (determinism, module docs) ----
    let mut router = UorR4Router::new(0.5);
    let corpus_text: String = windows.join(" ");
    let indexed = router.index_corpus(&corpus_text, ID);
    assert_eq!(
        indexed,
        windows.len(),
        "production bulk surface indexed every distinct window"
    );
    let qv = query_vectors(&mut router, &queries);

    // ---- zeta-grid states: the router's actual state evolution over
    // every stored window and every probe query, fresh identity per
    // text (module docs) ----
    println!("arm Z state law: evolve_state(fresh identity, text, gamma {GAMMA}) from default");
    let store_z: Vec<Vec<f64>> = windows
        .iter()
        .enumerate()
        .map(|(wi, w)| zeta_state(&mut router, &format!("user:zw{wi}"), w))
        .collect();
    let query_z: Vec<Vec<f64>> = queries
        .iter()
        .enumerate()
        .map(|(qi, q)| zeta_state(&mut router, &format!("user:zqz{qi}"), q))
        .collect();

    // ---- stored content vectors in window order (owned: the band
    // derivation below needs the router mutably) ----
    let content_store: Vec<Vec<f64>> = aligned_vectors(&router, &windows)
        .iter()
        .map(|v| v.to_vec())
        .collect();

    // ---- band-matched arms (module docs): storage band + full-width
    // content vector per stored window and per probe query ----
    println!(
        "band-matched arms: band = active_range of the stored content vector \
         (parity-asserted); content-full = evolve_state(fresh identity, text, gamma 0.0)"
    );
    let (store_cf, store_band): (Vec<Vec<f64>>, Vec<Band>) = windows
        .iter()
        .enumerate()
        .map(|(wi, w)| {
            content_full_and_band(&mut router, &format!("user:zcw{wi}"), w, &content_store[wi])
        })
        .unzip();
    let (query_cf, query_band): (Vec<Vec<f64>>, Vec<Band>) = queries
        .iter()
        .enumerate()
        .map(|(qi, q)| content_full_and_band(&mut router, &format!("user:zcq{qi}"), q, &qv[qi]))
        .unzip();
    let store_zb: Vec<Vec<f64>> = store_z
        .iter()
        .zip(&store_band)
        .map(|(state, &band)| banded(state, band))
        .collect();
    let query_zb: Vec<Vec<f64>> = query_z
        .iter()
        .zip(&query_band)
        .map(|(state, &band)| banded(state, band))
        .collect();

    // ---- retrieval: identical ranking law per arm; each arm's
    // shuffled control is its own stored set under the fixed
    // half-length rotation, queries unrotated (module docs) ----
    let rotation = windows.len() / 2;
    let (ranks_base, ranks_base_ctrl) = arm_ranks(&qv, &content_store, &targets, rotation);
    let (ranks_z, ranks_ctrl) = arm_ranks(&query_z, &store_z, &targets, rotation);
    let (ranks_zb, ranks_zb_ctrl) = arm_ranks(&query_zb, &store_zb, &targets, rotation);
    let (ranks_cf, ranks_cf_ctrl) = arm_ranks(&query_cf, &store_cf, &targets, rotation);

    let (top1_base, mrr_base) = rank_metrics(&ranks_base);
    let (_, mrr_base_ctrl) = rank_metrics(&ranks_base_ctrl);
    let (top1_z, mrr_z) = rank_metrics(&ranks_z);
    let (top1_ctrl, mrr_ctrl) = rank_metrics(&ranks_ctrl);
    let (top1_zb, mrr_zb) = rank_metrics(&ranks_zb);
    let (_, mrr_zb_ctrl) = rank_metrics(&ranks_zb_ctrl);
    let (top1_cf, mrr_cf) = rank_metrics(&ranks_cf);
    let (_, mrr_cf_ctrl) = rank_metrics(&ranks_cf_ctrl);

    println!(
        "zeta-grid state retrieval (issue #434): {} windows, {} probes",
        windows.len(),
        targets.len()
    );
    println!(
        "  baseline content vectors (#423 law, ref MRR 0.2348): top1 {top1_base:.3} | \
         MRR {mrr_base:.4} | ctrl MRR {mrr_base_ctrl:.4}"
    );
    println!(
        "  arm Z zeta-grid state:                               top1 {top1_z:.3} | MRR {mrr_z:.4}"
    );
    println!(
        "  control shuffled zeta store (half-length rotation):  top1 {top1_ctrl:.3} | \
         MRR {mrr_ctrl:.4}"
    );
    println!(
        "  arm ZB zeta-grid state, band-matched:                top1 {top1_zb:.3} | \
         MRR {mrr_zb:.4} | ctrl MRR {mrr_zb_ctrl:.4}"
    );
    println!(
        "  arm CF content vectors, full-width:                  top1 {top1_cf:.3} | \
         MRR {mrr_cf:.4} | ctrl MRR {mrr_cf_ctrl:.4}"
    );

    // Structural invariants gate; direction prints (module docs).
    for (name, m) in [
        ("baseline", mrr_base),
        ("baseline control", mrr_base_ctrl),
        ("arm Z", mrr_z),
        ("control", mrr_ctrl),
        ("arm ZB", mrr_zb),
        ("arm ZB control", mrr_zb_ctrl),
        ("arm CF", mrr_cf),
        ("arm CF control", mrr_cf_ctrl),
    ] {
        assert!((0.0..=1.0).contains(&m), "{name} MRR out of range: {m:.4}");
    }
    assert!(
        mrr_base > 0.0,
        "baseline premise: content retrieval must be nonzero for the ratio to mean anything"
    );

    // ---- pre-declared exit rule (module docs) ----
    let structure = mrr_z > mrr_ctrl && mrr_z >= 0.5 * mrr_base;
    println!(
        "exit rule (#434): arm Z MRR {mrr_z:.4} vs control {mrr_ctrl:.4} ({:+.4}, need \
         positive) and vs half the baseline {:.4} ({:+.4}, need at least zero) -> {}",
        mrr_z - mrr_ctrl,
        0.5 * mrr_base,
        mrr_z - 0.5 * mrr_base,
        if structure {
            "CONFIRMED (zeta-grid state carries retrieval-relevant structure)"
        } else {
            "recorded NEGATIVE (state does not function as a retrieval key at this bar)"
        }
    );

    // ---- pre-declared band-matched rule (module docs) ----
    let survives = mrr_zb >= 0.5 * mrr_z && mrr_zb > mrr_base;
    println!(
        "band-matched rule (#434 follow-up): arm ZB MRR {mrr_zb:.4} vs half arm Z {:.4} \
         ({:+.4}, need at least zero) and vs content baseline {mrr_base:.4} ({:+.4}, need \
         positive) -> {}",
        0.5 * mrr_z,
        mrr_zb - 0.5 * mrr_z,
        mrr_zb - mrr_base,
        if survives {
            "SURVIVES (the win is the zeta-state geometry, not vector fullness)"
        } else {
            "re-attributed to BANDING (arm Z's advantage does not survive band matching)"
        }
    );
}
