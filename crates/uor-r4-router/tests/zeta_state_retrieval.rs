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

    // ---- stores in window order + norms ----
    let content_store = aligned_vectors(&router, &windows);
    let content_norms: Vec<f64> = content_store.iter().map(|v| norm(v)).collect();
    let zeta_store: Vec<&[f64]> = store_z.iter().map(|v| v.as_slice()).collect();
    let zeta_norms: Vec<f64> = zeta_store.iter().map(|v| norm(v)).collect();

    // ---- CONTROL: arm Z's stored states rotated across window
    // positions by the fixed half-length rotation (the #423/#422
    // control construction); query states unrotated ----
    let rotation = windows.len() / 2;
    let shuffled_store: Vec<&[f64]> = (0..windows.len())
        .map(|position| zeta_store[(position + rotation) % windows.len()])
        .collect();
    let shuffled_norms: Vec<f64> = (0..windows.len())
        .map(|position| zeta_norms[(position + rotation) % windows.len()])
        .collect();

    // ---- retrieval: identical ranking law per arm (module docs) ----
    let mut ranks_base: Vec<usize> = Vec::with_capacity(targets.len());
    let mut ranks_z: Vec<usize> = Vec::with_capacity(targets.len());
    let mut ranks_ctrl: Vec<usize> = Vec::with_capacity(targets.len());
    for (probe, &t) in targets.iter().enumerate() {
        let sims_base = sims_against(&qv[probe], &content_store, &content_norms);
        ranks_base.push(base_rank(&sims_base, t));
        let sims_z = sims_against(&query_z[probe], &zeta_store, &zeta_norms);
        ranks_z.push(base_rank(&sims_z, t));
        let sims_ctrl = sims_against(&query_z[probe], &shuffled_store, &shuffled_norms);
        ranks_ctrl.push(base_rank(&sims_ctrl, t));
    }

    let (top1_base, mrr_base) = rank_metrics(&ranks_base);
    let (top1_z, mrr_z) = rank_metrics(&ranks_z);
    let (top1_ctrl, mrr_ctrl) = rank_metrics(&ranks_ctrl);

    println!(
        "zeta-grid state retrieval (issue #434): {} windows, {} probes",
        windows.len(),
        targets.len()
    );
    println!(
        "  baseline content vectors (#423 law, ref MRR 0.2348): top1 {top1_base:.3} | \
         MRR {mrr_base:.4}"
    );
    println!(
        "  arm Z zeta-grid state:                               top1 {top1_z:.3} | MRR {mrr_z:.4}"
    );
    println!(
        "  control shuffled zeta store (half-length rotation):  top1 {top1_ctrl:.3} | \
         MRR {mrr_ctrl:.4}"
    );

    // Structural invariants gate; direction prints (module docs).
    for (name, m) in [
        ("baseline", mrr_base),
        ("arm Z", mrr_z),
        ("control", mrr_ctrl),
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
}
