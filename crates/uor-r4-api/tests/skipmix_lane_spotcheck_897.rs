//! #897 — bounded minimal-pairs spot-check of the DEPLOYED 1-token skip-mix
//! lane on the real #833 canonical bundle (S1 redesign follow-up; parent
//! programme #822).
//!
//! ## What this is (and is not)
//!
//! The #897 phase-0 confirmation (`skipmix_confirm_897.rs`,
//! `docs/skipmix_confirm_897_result.json`) recorded verdict `SELECT-1-token`:
//! the `skipmix` arm clears the 25‰ opening bar (+56.2‰ vs base, CI
//! [53.6, 58.7]) and beats the D1-selected 2-token `mix` arm (paired lower
//! bound +23.1‰). That is an off-serving measurement over the reference
//! Rust closures in `skipmix_confirm_897.rs::Tables`, not the deployed
//! `R4Engine` path.
//!
//! This harness is the cheap, bounded check the lowering (tasks 2-5 of the
//! implementation plan: SKMX/PSIB wire format, the compiler fit, the runtime
//! contribution+argmax integration, the `ScoredGraphSections`/
//! `emit_scored_r4g1` extension) asks before any end-to-end causal run: does
//! the DEPLOYED lane, actually lowered onto the real bundle (fit → emit →
//! consume), realize the reference arm's signal on its own best cases — the
//! exact minimal pairs the `skipmix` arm resolved — within its known ceiling
//! (integer `ScoreQ` quantization, the decided candidate list, the bounded
//! per-key cap)? It is NOT the full causal run (#833); it replays only the
//! favorable subset the phase-0 confirmation already mined.
//!
//! ## Emission-path honesty (contrast with the #886 precedent)
//!
//! The #836 segment lane's spot-check (`segment_lane_spotcheck_886.rs`) had
//! to caveat its result: that lane's only emitter
//! (`convert_r4g1::convert_with_segment_table`) is NOT the emitter that
//! produced the released `graph/score.r4g1`, so re-emitting could not (and
//! did not) reproduce the released graph's held-out serving behavior — 6/20
//! favorable positions came back UNSERVABLE-AT-EMISSION. Per Casey's #897
//! decision point B, this lowering instead extends the REAL release emitter
//! (`uor_r4_graph_certify::score::{ScoredGraphSections, emit_scored_r4g1}`) —
//! the exact function the production `score` CLI command calls. This
//! harness's re-emitted graph is therefore built with the SAME pipeline
//! (cover recovery, the real store, the real `compile_transitions_with_
//! quantization` / `compile_context_rows` / `compile_forward_anchor_rows` /
//! `compile_emissions`) the released graph came from, plus the new SKMX/PSIB
//! sections — so #886's emission-path caveat does not apply here by
//! construction, and an UNSERVABLE-AT-EMISSION outcome here would itself be
//! a new finding (a serving-path defect, not an emitter mismatch).
//!
//! ## Construction (all deterministic, off no RNG)
//!
//! 1. **Reproduce the phase-0 `skipmix` favorable pairs.** Rebuild the
//!    reference tables (`suffix_next`/`content_next`/`joint_next`/
//!    `d4skip_next`) from the corpus TRAIN split exactly as
//!    `skipmix_confirm_897.rs::Tables` does, mine the minimal pairs (same
//!    2-token suffix, different story, different teacher target, one pair
//!    per suffix key), and keep exactly the pairs the `skipmix` arm
//!    FOLLOWED. This reproduces the recorded `skipmix_follow = 87 / 4722`
//!    (`docs/skipmix_confirm_897_result.json`); the count is asserted — a
//!    reproduction control.
//! 2. **Fit + emit the deployed tables into a real bundle.**
//!    `skipmix_fit::fit_skipmix_tables` (top-64/key, the phase-0 `CAP`,
//!    quantized to integer `ScoreQ`) over the same corpus, emitted with the
//!    production `score::emit_scored_r4g1` over the real teacher artifacts,
//!    the real recovered cover (regions/structural), the real store, and the
//!    real compiled transitions/context-rows/forward-anchor-rows/emissions —
//!    loaded into `R4Engine` — the engine consuming both sections is asserted
//!    (`skipmix_tables_present()`).
//! 3. **Replay the deployed lane.** For each favorable position, on a
//!    freshly reset engine, take the served token from
//!    `predict_decision_candidates_with_skipmix` (no persistent session
//!    object is needed — unlike #836, the skip-mix evidence is entirely the
//!    current window). Each position is classified served / abstained /
//!    UNSERVABLE (a serving-policy fault is caught, not fatal), and a
//!    favorable pair "follows" when the lane serves each side's own teacher
//!    target and the two differ.
//!
//! ## Predeclared read (frozen before the run)
//!
//! Mirrors the #886 60% bar, applied to this arm's larger favorable set:
//! `FOLLOW_MIN = ceil(0.6 * 87) = 53`.
//!   * `< 53/87` ⇒ the deployed path does not track its own ceiling even on
//!     its best cases — a lowering-fidelity problem to fix before the
//!     reachability-gate probe or any end-to-end run.
//!   * `>= 53/87` ⇒ the lowering is faithful; the lane proceeds to the
//!     reachability-gate probe (design spec §7 step 1) next.
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test skipmix_lane_spotcheck_897 \
//!       -- --ignored --nocapture
//! Env: R4_CAUSAL_BUNDLE (bundle dir; corpus + store + cover + artifacts are
//! all read from it). Defaults to the attested #833 bundle
//! `.uor-models/compiled/smollm2-360m-broad-clean`.

#![allow(clippy::doc_lazy_continuation)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_api::capability_suite::compute_cid;
use uor_r4_api::engine::{EngineParts, PredictDecision, R4Engine};
use uor_r4_api::serving_eval::ServingBundle;
use uor_r4_core::transformerless::{compiler, runtime};
use uor_r4_graph_certify as score;
use uor_r4_graph_certify::StepCandidates;
use uor_r4_graph_compiler::{induction, skipmix_fit};

// --- pre-registered constants (frozen before the run) ----------------------
/// Suffix length that defines the minimal-pair key (the phase-0 recipe).
const SUFFIX_K: usize = 2;
/// Per-key cap on retained argmax counts (bounded lane tables, #835);
/// matches `skipmix_fit::fit_skipmix_tables`'s `top_k` and the phase-0
/// harness's `CAP`.
const CAP: usize = uor_r4_graph_compiler::segment_fit::DEFAULT_TOP_K;
/// Candidate-set widths: suffix baseline, content widening, and the D1
/// joint-evidence widening — identical to the phase-0 harness's candidate
/// construction, so this replay sees the same decided candidate list.
const CAND_SUFFIX: usize = 32;
const CAND_CONTENT: usize = 32;
const CAND_JOINT: usize = 32;
/// Pre-registered residual weight lambda = 1.0, fixed before evaluation.
const LAMBDA_NUM: f64 = 1.0;
const LAMBDA_DEN: f64 = 1.0;
/// The phase-0 `skipmix` arm's recorded favorable-pair count
/// (`docs/skipmix_confirm_897_result.json` minimal_pairs.skipmix_follow) — a
/// reproduction control: the mined favorable set must match it exactly.
const REFERENCE_FAVORABLE_EXPECTED: u64 = 87;
/// Predeclared read threshold: ceil(0.6 * 87) — the #886 60% bar, scaled to
/// this arm's larger favorable set.
const FOLLOW_MIN: u64 = 53;
/// The attested #833 bundle's corpus.meta CID prefix (same corpus the phase-0
/// confirmation used) — guards against a wrong bundle at R4_CAUSAL_BUNDLE.
const ATTESTED_CORPUS_CID_PREFIX: &str = "blake3:aa9d1767";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

fn bundle_root() -> PathBuf {
    match std::env::var_os("R4_CAUSAL_BUNDLE") {
        Some(v) => PathBuf::from(v),
        None => repo_root()
            .join(".uor-models")
            .join("compiled")
            .join("smollm2-360m-broad-clean"),
    }
}

fn suffix_key(window: &[u32]) -> (u32, u32) {
    let n = window.len();
    if n >= 2 {
        (window[n - 2], window[n - 1])
    } else if n == 1 {
        (u32::MAX, window[0])
    } else {
        (u32::MAX, u32::MAX)
    }
}

fn uniq_tokens(w: &[u32]) -> Vec<u32> {
    let mut uniq: Vec<u32> = w.to_vec();
    uniq.sort_unstable();
    uniq.dedup();
    uniq
}

/// Argmax by score, canonical tie-break (score desc, id asc) — order
/// independent, so mining is deterministic despite HashMap iteration.
fn argmax(scores: &HashMap<u32, f64>) -> u32 {
    let mut best_id = u32::MAX;
    let mut best = f64::NEG_INFINITY;
    for (&id, &s) in scores {
        if s > best || (s == best && id < best_id) {
            best = s;
            best_id = id;
        }
    }
    best_id
}

/// Counts of teacher argmax under one key, capped to the top `CAP` by count
/// (ties broken by the smaller token id) — the phase-0 harness's `Counter`.
#[derive(Default, Clone)]
struct Counter {
    map: HashMap<u32, u32>,
}

impl Counter {
    fn bump(&mut self, tok: u32) {
        *self.map.entry(tok).or_insert(0) += 1;
    }
    fn cap_to_top(&mut self, cap: usize) {
        if self.map.len() <= cap {
            return;
        }
        let mut v: Vec<(u32, u32)> = self.map.drain().collect();
        v.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v.truncate(cap);
        self.map = v.into_iter().collect();
    }
    fn total(&self) -> u64 {
        self.map.values().map(|&c| c as u64).sum()
    }
}

/// The reference tables, trimmed to exactly what mining the `skipmix` arm's
/// favorable pairs needs (`skipmix_confirm_897.rs::Tables`, subset).
struct Tables {
    suffix_next: HashMap<(u32, u32), Counter>,
    content_next: HashMap<u32, Counter>,
    joint_next: HashMap<(u32, (u32, u32)), Counter>,
    d4skip_next: HashMap<(u32, u32), Counter>,
    marginal_tok: u32,
}

impl Tables {
    fn content_rates(&self, w: &[u32]) -> HashMap<u32, f64> {
        let mut content_rate: HashMap<u32, f64> = HashMap::new();
        let uniq = uniq_tokens(w);
        let ncontent = uniq.len().max(1) as f64;
        for t in &uniq {
            if let Some(cn) = self.content_next.get(t) {
                let tot = cn.total().max(1) as f64;
                for (&c, &cnt) in &cn.map {
                    *content_rate.entry(c).or_insert(0.0) += (cnt as f64 / tot) / ncontent;
                }
            }
        }
        content_rate
    }

    fn suffix_cands(&self, w: &[u32]) -> Vec<u32> {
        let mut suffix_cands: Vec<u32> = Vec::new();
        if let Some(c) = self.suffix_next.get(&suffix_key(w)) {
            let mut v: Vec<(u32, u32)> = c.map.iter().map(|(&k, &n)| (k, n)).collect();
            v.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            suffix_cands.extend(v.into_iter().take(CAND_SUFFIX).map(|(k, _)| k));
        }
        suffix_cands.push(self.marginal_tok);
        suffix_cands.sort_unstable();
        suffix_cands.dedup();
        suffix_cands
    }

    fn widened_cands(&self, suffix_cands: &[u32], content_rate: &HashMap<u32, f64>) -> Vec<u32> {
        let mut cr: Vec<(u32, f64)> = content_rate.iter().map(|(&k, &v)| (k, v)).collect();
        cr.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        let mut all_cands = suffix_cands.to_vec();
        all_cands.extend(cr.into_iter().take(CAND_CONTENT).map(|(k, _)| k));
        all_cands.sort_unstable();
        all_cands.dedup();
        all_cands
    }

    /// Summed supported-key joint rates per candidate, over `w`'s unique
    /// tokens conditioned on suffix `sfx` — used only to widen the shared D1
    /// candidate set (the `skipmix` arm itself scores from `d4skip_next`).
    fn joint_acc(&self, tokens: &[u32], sfx: (u32, u32)) -> HashMap<u32, f64> {
        let mut acc: HashMap<u32, f64> = HashMap::new();
        for t in uniq_tokens(tokens) {
            if let Some(cn) = self.joint_next.get(&(t, sfx)) {
                let tot = cn.total().max(1) as f64;
                for (&c, &cnt) in &cn.map {
                    *acc.entry(c).or_insert(0.0) += cnt as f64 / tot;
                }
            }
        }
        acc
    }

    fn joint_widened(&self, legacy: &[u32], joint_acc: &HashMap<u32, f64>) -> Vec<u32> {
        let mut jr: Vec<(u32, f64)> = joint_acc.iter().map(|(&k, &v)| (k, v)).collect();
        jr.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        let mut all = legacy.to_vec();
        all.extend(jr.into_iter().take(CAND_JOINT).map(|(k, _)| k));
        all.sort_unstable();
        all.dedup();
        all
    }

    fn suffix_rate(&self, w: &[u32], c: u32) -> f64 {
        let sfx = self.suffix_next.get(&suffix_key(w));
        let sfx_total = sfx.map(|c| c.total()).unwrap_or(0).max(1) as f64;
        sfx.and_then(|s| s.map.get(&c)).copied().unwrap_or(0) as f64 / sfx_total
    }

    /// The confirmed PRIMARY `skipmix` arm: joint tables keyed by
    /// `(t, cond_last)` where supported, verbatim content-token (Ψ-bag)
    /// contribution where not, normalized over all unique window tokens,
    /// residual against the full 2-token `suffix_rate`
    /// (`skipmix_confirm_897.rs::Tables::skipmix_scores`, verbatim).
    fn skipmix_scores(
        &self,
        w: &[u32],
        cond_last: u32,
        cands: &[u32],
        lam: f64,
    ) -> HashMap<u32, f64> {
        let uniq = uniq_tokens(w);
        let ntot = uniq.len().max(1) as f64;
        let mut contrib: HashMap<u32, f64> = HashMap::new();
        let mut sup_ts = 0usize;
        for t in &uniq {
            if let Some(cn) = self.d4skip_next.get(&(*t, cond_last)) {
                sup_ts += 1;
                let tot = cn.total().max(1) as f64;
                for (&c, &cnt) in &cn.map {
                    *contrib.entry(c).or_insert(0.0) += cnt as f64 / tot;
                }
            } else if let Some(cn) = self.content_next.get(t) {
                let tot = cn.total().max(1) as f64;
                for (&c, &cnt) in &cn.map {
                    *contrib.entry(c).or_insert(0.0) += cnt as f64 / tot;
                }
            }
        }
        let mut m = HashMap::new();
        for &c in cands {
            let base = self.suffix_rate(w, c);
            let resid = (contrib.get(&c).copied().unwrap_or(0.0) - sup_ts as f64 * base) / ntot;
            m.insert(c, base + lam * resid);
        }
        m
    }
}

/// One held-out position's reference predictions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Preds {
    base: u32,
    skipmix: u32,
}

fn score_position(t: &Tables, w: &[u32], lam: f64) -> Preds {
    let sfx = suffix_key(w);
    let content_rate = t.content_rates(w);
    let suffix_cands = t.suffix_cands(w);
    let legacy_cands = t.widened_cands(&suffix_cands, &content_rate);
    let joint_acc = t.joint_acc(w, sfx);
    let d1_cands = t.joint_widened(&legacy_cands, &joint_acc);

    let mut base_scores: HashMap<u32, f64> = HashMap::new();
    for &c in &suffix_cands {
        base_scores.insert(c, t.suffix_rate(w, c));
    }
    let own_last = w.last().copied().unwrap_or(u32::MAX);
    let skipmix_scores = t.skipmix_scores(w, own_last, &d1_cands, lam);

    Preds {
        base: argmax(&base_scores),
        skipmix: argmax(&skipmix_scores),
    }
}

/// A reproduced phase-0 favorable minimal pair (positions + teacher targets).
struct FavPair {
    ia: usize,
    ib: usize,
    ta: u32,
    tb: u32,
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle (corpus+store+cover+artifacts); run with --ignored"]
fn skipmix_lane_spotcheck_897() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP skipmix_lane_spotcheck_897: no serving bundle at {}",
            root.display()
        );
        return;
    };
    let meta_bytes = std::fs::read(&bundle.corpus_meta).expect("corpus meta");
    let recs_bytes = std::fs::read(&bundle.corpus_records).expect("corpus records");
    let corpus = compiler::load_corpus_from(
        bundle.corpus_meta.to_str().expect("meta utf8"),
        bundle.corpus_records.to_str().expect("recs utf8"),
    )
    .expect("load corpus");
    let (train_positions, held_out_positions) = induction::split_positions(&corpus);
    assert!(
        !train_positions.is_empty() && !held_out_positions.is_empty(),
        "non-empty split"
    );

    let corpus_cid = compute_cid(&meta_bytes);
    assert!(
        corpus_cid.starts_with(ATTESTED_CORPUS_CID_PREFIX),
        "spot-check is only valid on the attested #833 bundle (corpus.meta CID \
         prefix {ATTESTED_CORPUS_CID_PREFIX}); got {corpus_cid}"
    );

    let started = Instant::now();

    // === 1. reproduce the phase-0 `skipmix` favorable pairs =================
    let mut suffix_next: HashMap<(u32, u32), Counter> = HashMap::new();
    let mut content_next: HashMap<u32, Counter> = HashMap::new();
    let mut joint_next: HashMap<(u32, (u32, u32)), Counter> = HashMap::new();
    let mut d4skip_next: HashMap<(u32, u32), Counter> = HashMap::new();
    let mut marginal = Counter::default();
    for &i in &train_positions {
        let w = induction::context_window(&corpus, i);
        let target = corpus.t_argmax[i];
        let sfx = suffix_key(&w);
        marginal.bump(target);
        suffix_next.entry(sfx).or_default().bump(target);
        for t in uniq_tokens(&w) {
            content_next.entry(t).or_default().bump(target);
            joint_next.entry((t, sfx)).or_default().bump(target);
        }
        let n = w.len();
        if n >= 2 {
            let last = w[n - 1];
            let mut skip_pairs: Vec<(u32, u32)> = w[..n - 1].iter().map(|&t| (t, last)).collect();
            skip_pairs.sort_unstable();
            skip_pairs.dedup();
            for key in skip_pairs {
                d4skip_next.entry(key).or_default().bump(target);
            }
        }
    }
    for c in suffix_next.values_mut() {
        c.cap_to_top(CAP);
    }
    for c in content_next.values_mut() {
        c.cap_to_top(CAP);
    }
    for c in joint_next.values_mut() {
        c.cap_to_top(CAP);
    }
    for c in d4skip_next.values_mut() {
        c.cap_to_top(CAP);
    }
    let marginal_tok = marginal
        .map
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
        .unwrap_or(0);

    let tables = Tables {
        suffix_next,
        content_next,
        joint_next,
        d4skip_next,
        marginal_tok,
    };
    let lambda = LAMBDA_NUM / LAMBDA_DEN;

    let windows: Vec<Vec<u32>> = held_out_positions
        .iter()
        .map(|&i| induction::context_window(&corpus, i))
        .collect();
    let preds: Vec<Preds> = windows
        .iter()
        .map(|w| score_position(&tables, w, lambda))
        .collect();

    let mut by_suffix: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (idx, w) in windows.iter().enumerate() {
        if w.len() >= SUFFIX_K {
            by_suffix.entry(suffix_key(w)).or_default().push(idx);
        }
    }
    let mut mp_total = 0u64;
    let mut base_follow = 0u64;
    let mut favorable: Vec<FavPair> = Vec::new();
    for group in by_suffix.values() {
        let mut made = 0usize;
        'outer: for a in 0..group.len() {
            for b in (a + 1)..group.len() {
                let (xa, xb) = (group[a], group[b]);
                let (ia, ib) = (held_out_positions[xa], held_out_positions[xb]);
                if corpus.story[ia] == corpus.story[ib] {
                    continue;
                }
                let (ta, tb) = (corpus.t_argmax[ia], corpus.t_argmax[ib]);
                if ta == tb {
                    continue;
                }
                mp_total += 1;
                let (pa, pb) = (preds[xa], preds[xb]);
                if pa.base == ta && pb.base == tb && pa.base != pb.base {
                    base_follow += 1;
                }
                if pa.skipmix == ta && pb.skipmix == tb && pa.skipmix != pb.skipmix {
                    favorable.push(FavPair { ia, ib, ta, tb });
                }
                made += 1;
                if made >= 1 {
                    break 'outer;
                }
            }
        }
    }
    let reference_favorable = favorable.len() as u64;

    // === 2. fit + emit the deployed tables via the REAL release emitter =====
    let rows_top_k = CAP;
    let (skipmix_rows, psi_bag_rows) = skipmix_fit::fit_skipmix_tables(&corpus, rows_top_k);
    assert!(!skipmix_rows.is_empty(), "the fit must learn joint keys");
    assert!(!psi_bag_rows.is_empty(), "the fit must learn psi-bag keys");

    let artifact_container = std::fs::read(&bundle.teacher).expect("teacher artifacts");
    let artifacts = compiler::parse_artifacts(&artifact_container).expect("parse artifacts");
    let threads = std::thread::available_parallelism()
        .map(|c| c.get().min(8))
        .unwrap_or(1);
    let train_obs =
        induction::build_observations_with_threads(&artifacts, &corpus, &train_positions, threads);

    let cover_bytes = std::fs::read(bundle.root.join("graph-cover").join("cover.r4g1"))
        .expect("cached cover artifact");
    let (regions, structural) =
        score::recover_from_artifact(&cover_bytes).expect("recover regions/structural");
    let max_depth = regions.iter().map(|r| r.depth as usize).max().unwrap_or(1);

    let store_file_bytes = std::fs::read(bundle.root.join("tless_store.bin")).expect("store bytes");
    let store = runtime::parse_store(&store_file_bytes).expect("parse store (u32)");

    let config = score::ScoreConfig::default();
    let (transitions, transition_quantization) = score::compile_transitions_with_quantization(
        &corpus,
        &regions,
        &train_obs,
        max_depth,
        config.transition_out_degree,
    );
    let vocab =
        u32::try_from(artifacts.token_codes.len() / compiler::STAGES).expect("vocab fits u32");
    let context_rows = score::compile_context_rows(&corpus, &train_obs, vocab, &config);
    let fwd_rows = score::compile_forward_anchor_rows(&corpus, &train_obs);
    let emissions = score::compile_emissions(
        &corpus, &store, &regions, &train_obs, max_depth, vocab, &config,
    );
    let tls1 = runtime::store_bytes(&store);

    let sections = score::ScoredGraphSections {
        regions: &regions,
        structural: &structural,
        transitions: &transitions,
        transition_quantization,
        emissions: &emissions,
        context_rows: &context_rows,
        exct_tls1: &tls1,
        exct_top_x: config.exct_top_x,
        fwd_rows: &fwd_rows,
        skipmix_rows: &skipmix_rows,
        psi_bag_rows: &psi_bag_rows,
    };
    let (graph_bytes, info) = score::emit_scored_r4g1(
        &artifact_container,
        (&meta_bytes, &recs_bytes),
        vocab,
        &sections,
    );
    assert_eq!(info.skipmix_row_count as usize, skipmix_rows.len());
    assert_eq!(info.psi_bag_row_count as usize, psi_bag_rows.len());

    let tokenizer_bytes = std::fs::read(bundle.root.join("tokenizer.bin")).ok();
    let probe = R4Engine::load_accepting_quality(EngineParts {
        graph: &graph_bytes,
        signature_artifact: &artifact_container,
        tokenizer: tokenizer_bytes.as_deref(),
        score_report: None,
    })
    .expect("re-emitted engine load");
    assert_eq!(
        probe.skipmix_tables_present(),
        (true, true),
        "the engine must consume both the SKMX and PSIB sections"
    );
    drop(probe);

    // === 3. replay the deployed lane on the favorable pairs (robust) =======
    let mut engine = R4Engine::load_accepting_quality(EngineParts {
        graph: &graph_bytes,
        signature_artifact: &artifact_container,
        tokenizer: tokenizer_bytes.as_deref(),
        score_report: None,
    })
    .expect("re-emitted engine load");

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    let eval = |engine: &mut R4Engine, i: usize| -> Option<Option<u32>> {
        engine.reset();
        let w = induction::context_window(&corpus, i);
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut cands = StepCandidates::default();
            match engine.predict_decision_candidates_with_skipmix(&w, &mut cands) {
                Ok(PredictDecision::Serve(outcome)) => Some(outcome.token),
                _ => None,
            }
        }))
        .ok()
    };

    let mut served = 0u64;
    let mut abstained = 0u64;
    let mut unservable = 0u64;
    let mut deployed_follow = 0u64;
    let mut pair_lines: Vec<String> = Vec::new();
    for (idx, p) in favorable.iter().enumerate() {
        let la = eval(&mut engine, p.ia);
        let lb = eval(&mut engine, p.ib);
        for r in [la, lb] {
            match r {
                Some(Some(_)) => served += 1,
                Some(None) => abstained += 1,
                None => unservable += 1,
            }
        }
        let pa = la.flatten();
        let pb = lb.flatten();
        let follow = pa == Some(p.ta) && pb == Some(p.tb) && pa != pb;
        if follow {
            deployed_follow += 1;
        }
        pair_lines.push(format!(
            "  pair {idx}: a[pos {} t {}]->{:?}  b[pos {} t {}]->{:?}  follow={follow}",
            p.ia, p.ta, la, p.ib, p.tb, lb
        ));
    }

    std::panic::set_hook(prev_hook);
    let positions = reference_favorable * 2;

    // === 4. verdict + record =================================================
    let faithful = deployed_follow >= FOLLOW_MIN;
    let read = if positions > 0 && unservable == positions {
        "UNSERVABLE-AT-EMISSION — the re-emitted (emit_scored_r4g1, the REAL \
         release emitter) serving graph cannot serve the reference arm's \
         favorable held-out windows under the deployed policy. Unlike #886, \
         this is not an emission-path mismatch (this IS the production \
         emitter) — a lowering or serving-path defect to investigate."
    } else if faithful {
        "FAITHFUL — the deployed skip-mix lane follows >= 53/87 of the \
         reference arm's favorable pairs; the fit->emit(real emitter)->consume \
         lowering tracks the phase-0 skipmix arm's signal"
    } else {
        "LOWERING-FIDELITY GAP — the deployed lane follows < 53/87 of the \
         reference arm's favorable pairs; the quantized deployed path does \
         not track its own reference ceiling on its best cases"
    };

    let elapsed = started.elapsed();
    let artifact_cid = compute_cid(&graph_bytes);

    println!("=== #897 deployed skip-mix lane bounded spot-check ===");
    println!("bundle              : {}", bundle.root.display());
    println!("reemit_artifact_cid : {artifact_cid}");
    println!("corpus_meta_cid     : {corpus_cid}");
    println!(
        "train / held_out    : {} / {}",
        train_positions.len(),
        held_out_positions.len()
    );
    println!(
        "fitted rows         : skmx {} / psib {}",
        skipmix_rows.len(),
        psi_bag_rows.len()
    );
    println!("minimal pairs mined : {mp_total}");
    println!(
        "reference favorable : {reference_favorable} (expected {REFERENCE_FAVORABLE_EXPECTED}); \
         reference baseline-follow {base_follow}"
    );
    for line in &pair_lines {
        println!("{line}");
    }
    println!(
        "servability         : served {served}, abstained {abstained}, unservable {unservable} of {positions} positions"
    );
    println!("deployed lane follow: {deployed_follow}/{reference_favorable}");
    println!("elapsed             : {:.1}s", elapsed.as_secs_f64());
    println!("READ                : {read}");

    let mut rec = Vec::new();
    for v in [
        reference_favorable,
        deployed_follow,
        mp_total,
        base_follow,
        positions,
        served,
        abstained,
        unservable,
        skipmix_rows.len() as u64,
        psi_bag_rows.len() as u64,
        train_positions.len() as u64,
        held_out_positions.len() as u64,
    ] {
        rec.extend_from_slice(&v.to_le_bytes());
    }
    rec.extend_from_slice(artifact_cid.as_bytes());
    rec.extend_from_slice(corpus_cid.as_bytes());
    let result_cid = compute_cid(&rec);
    println!("result_cid          : {result_cid}");

    let json = format!(
        concat!(
            "{{\n",
            "  \"issue\": 897,\n",
            "  \"check\": \"deployed-skipmix-lane-bounded-minimal-pairs-spot-check\",\n",
            "  \"decision_kind\": \"bounded spot-check (no full causal run launched)\",\n",
            "  \"bundle\": \"{}\",\n",
            "  \"reemit_artifact_cid\": \"{}\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"emission_path\": \"emit_scored_r4g1 (the REAL release emitter; unlike #886 this is not a side converter)\",\n",
            "  \"train\": {},\n",
            "  \"held_out\": {},\n",
            "  \"fitted_rows\": {{\"skmx\": {}, \"psib\": {}}},\n",
            "  \"minimal_pairs_mined\": {},\n",
            "  \"reference_favorable\": {},\n",
            "  \"reference_favorable_expected\": {},\n",
            "  \"reference_baseline_follow\": {},\n",
            "  \"positions\": {},\n",
            "  \"served\": {},\n",
            "  \"abstained\": {},\n",
            "  \"unservable\": {},\n",
            "  \"deployed_lane_follow\": {},\n",
            "  \"follow_min_for_faithful\": {},\n",
            "  \"faithful\": {},\n",
            "  \"lane_status\": \"dormant\",\n",
            "  \"ids_toml_serving_row\": false,\n",
            "  \"conformance_changed\": false,\n",
            "  \"result_cid\": \"{}\",\n",
            "  \"read\": \"{}\"\n",
            "}}\n"
        ),
        bundle.root.display(),
        artifact_cid,
        corpus_cid,
        train_positions.len(),
        held_out_positions.len(),
        skipmix_rows.len(),
        psi_bag_rows.len(),
        mp_total,
        reference_favorable,
        REFERENCE_FAVORABLE_EXPECTED,
        base_follow,
        positions,
        served,
        abstained,
        unservable,
        deployed_follow,
        FOLLOW_MIN,
        faithful,
        result_cid,
        read,
    );
    let out = repo_root()
        .join("docs")
        .join("skipmix_lane_897_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote               : {}", out.display());

    // === structural guards (non-vacuous machinery, not a vacuous green) =====
    assert_eq!(
        reference_favorable, REFERENCE_FAVORABLE_EXPECTED,
        "the reproduced skipmix favorable-pair count must match \
         docs/skipmix_confirm_897_result.json's minimal_pairs.skipmix_follow"
    );
    assert_eq!(
        base_follow, 0,
        "the suffix baseline cannot follow identical-suffix minimal pairs"
    );
    assert!(positions > 0, "no favorable positions evaluated");
}
