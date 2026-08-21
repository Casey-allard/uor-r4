//! #904 -- cheap, off-serving diagnostic decomposing the #897
//! LOWERING-FIDELITY GAP (`docs/skipmix_lane_897_result.json`:
//! deployed-lane follow 41/87, below the predeclared 53/87 bar) into its
//! candidate causes, before any fix is chosen (Casey's "diagnose the gap
//! first" sign-off on #897/#903's follow-up).
//!
//! ## Correction to the #897 record
//!
//! `docs/skipmix_lane_897_spotcheck.md` named "the per-key top-64 cap" as a
//! suspect. It is not: the reference `Tables` here and the deployed
//! `skipmix_fit::fit_skipmix_tables` both cap at the identical
//! `CAP = DEFAULT_TOP_K = 64`. This harness does not re-test the cap.
//!
//! ## What this measures (three arms, decomposed)
//!
//! Reuses the exact fit -> emit(real emitter) -> consume machinery of
//! `skipmix_lane_spotcheck_897.rs` (same corpus, same fit, same real
//! bundle), then replays the same 87 reference-favorable pairs with added
//! instrumentation:
//!
//! 1. **Candidate coverage ceiling.** The deployed re-rank
//!    (`predict_decision_candidates_with_skipmix`) only ever chooses among
//!    `candidates.ranked()`, capped at
//!    `uor_r4_graph_certify::score_runtime::StepCandidates::STEP_TOP_CANDIDATES
//!    = 8`. The reference arm's `skipmix_scores` is evaluated over `d1_cands`,
//!    a suffix/content/joint-widened union up to ~97 tokens. For each pair,
//!    is the teacher target present at all in the engine's real top-8 list on
//!    each side? This is the maximum ANY re-ranking policy bound to this
//!    candidate list could ever follow -- independent of scoring.
//! 2. **Reference math, restricted to the narrow (real, top-8) candidate
//!    list.** The validated, unquantized, wide-candidate `skipmix_scores`
//!    formula, evaluated only over each position's actual top-8 tokens.
//!    Isolates the pure candidate-breadth effect on an otherwise-idealized
//!    scorer.
//! 3. **Real base `ScoreQ` + a unit-safe (non-additive) combine, over the
//!    same narrow candidate list.** The deployed lane adds
//!    `segment_fit::quantize_rate`'s linear, `2^20`-scaled rate directly onto
//!    `ScoreQ.raw()`, a log-probability scaled by `2^16`
//!    (`skipmix_scale_by_lambda_and_support` is the identity function -- no
//!    calibration reconciles the two scales). This arm instead ranks
//!    candidates by (has skip-mix support, contribution magnitude, base
//!    `ScoreQ`, token id ascending) -- trusting the skip-mix signal whenever
//!    present, never summing across incompatible scales. Isolates whether
//!    the additive scale mismatch specifically is costing follow-throughs
//!    among reachable candidates.
//!
//! A fourth quantity, `deployed_follow`, is recomputed here as a
//! reproduction control: it must equal the recorded 41/87
//! (`docs/skipmix_lane_897_result.json`), and a per-position manual
//! re-derivation (base `ScoreQ.raw()` + this harness's own contribution
//! replica, additive, same as production) must equal the engine's actual
//! served token on every one of the 174 positions -- an instrument-fidelity
//! control: if this harness's contribution replica ever disagreed with the
//! engine, the other three arms would not be trustworthy.
//!
//! ## Predeclared reads (frozen before the run)
//!
//! * `coverage_ok` close to 87 (say >= 70) => breadth is NOT the bottleneck;
//!   look to quantization/combine (arm 3 vs deployed).
//! * `coverage_ok` well below 87 => breadth is a hard ceiling; no combine-rule
//!   fix alone can close the gap past `coverage_ok`, only widening
//!   `STEP_TOP_CANDIDATES` (or the base engine's own candidate generation)
//!   could.
//! * `arm3_follow` >> `deployed_follow` (both bounded by `coverage_ok`) =>
//!   the additive scale mismatch is a real, fixable defect independent of
//!   breadth -- a bounded, low-risk fix (change the combine rule).
//! * `arm3_follow` ~= `deployed_follow` => the additive scale mismatch is not
//!   the dominant factor; the gap is chiefly explained by `coverage_ok`
//!   (arm 1/2), which needs a different (larger) fix or the lane stays
//!   dormant.
//!
//! No production code changes. Off-serving; gated behind `--ignored`, same
//! discipline as `skipmix_lane_spotcheck_897.rs` and `segment_lane_
//! spotcheck_886.rs`. Result written to
//! `docs/skipmix_gap_diagnostic_904_result.json`.
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test skipmix_gap_diagnostic_904 \
//!       -- --ignored --nocapture
//! Env: R4_CAUSAL_BUNDLE (bundle dir), same default as #897's harness.

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
use uor_r4_graph_format::ScoreQ;

// --- pre-registered constants (identical to skipmix_lane_spotcheck_897.rs) -
const SUFFIX_K: usize = 2;
const CAP: usize = uor_r4_graph_compiler::segment_fit::DEFAULT_TOP_K;
const CAND_SUFFIX: usize = 32;
const CAND_CONTENT: usize = 32;
const CAND_JOINT: usize = 32;
const LAMBDA_NUM: f64 = 1.0;
const LAMBDA_DEN: f64 = 1.0;
const REFERENCE_FAVORABLE_EXPECTED: u64 = 87;
/// The #897 recorded deployed-lane follow count -- this harness's own
/// recompute must reproduce it exactly (same fit, same emitter, same
/// engine, same replay).
const DEPLOYED_FOLLOW_EXPECTED: u64 = 41;
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
/// (ties broken by the smaller token id) -- verbatim from
/// `skipmix_lane_spotcheck_897.rs` / `skipmix_confirm_897.rs`.
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
/// favorable pairs (and, here, arm 2's restricted re-score) needs --
/// verbatim from `skipmix_lane_spotcheck_897.rs::Tables`.
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

    /// The confirmed PRIMARY `skipmix` arm, verbatim from
    /// `skipmix_confirm_897.rs::Tables::skipmix_scores` -- generic over
    /// whatever candidate list `cands` is (arm 2 passes the narrow, real,
    /// top-8 list instead of `d1_cands`).
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

/// One held-out position's reference predictions (full-width `d1_cands`).
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

/// Additive skip-mix contribution replica: for each unique window token,
/// either the joint-table row (if the composite key is present) or the
/// Ψ-bag fallback row (if not) contributes its raw `i32` weight to
/// `candidate`, summed via saturating add -- bit-identical to
/// `uor_r4_api::engine`'s private `skipmix_token_contribution`, replicated
/// here because that function is not `pub` outside the crate.
fn skmx_contribution(
    skmx: &HashMap<(u32, u32), HashMap<u32, i32>>,
    psib: &HashMap<u32, HashMap<u32, i32>>,
    unique_tokens: &[u32],
    last_token: u32,
    candidate: u32,
) -> i32 {
    let mut acc: i32 = 0;
    for &t in unique_tokens {
        if let Some(row) = skmx.get(&(t, last_token)) {
            acc = acc.saturating_add(row.get(&candidate).copied().unwrap_or(0));
        } else if let Some(row) = psib.get(&t) {
            acc = acc.saturating_add(row.get(&candidate).copied().unwrap_or(0));
        }
    }
    acc
}

/// Arm 3's unit-safe combine key: candidates with ANY skip-mix support
/// (`contribution > 0`, guaranteed by `quantize_rate`'s `clamp(1, ..)`
/// floor) always outrank candidates with none, ranked among themselves by
/// contribution magnitude; candidates with no support fall back to the
/// real base `ScoreQ`. Never sums across the two incompatible scales.
fn arm3_key(contribution: i32, base_raw: i32) -> (i32, i32) {
    if contribution > 0 {
        (1, contribution)
    } else {
        (0, base_raw)
    }
}

/// Arm 3's pick: argmax of `arm3_key` over the real narrow `ranked` list,
/// tie-broken by smaller token id (the canonical tie-break).
fn arm3_pick(
    ranked: &[(u32, ScoreQ)],
    skmx: &HashMap<(u32, u32), HashMap<u32, i32>>,
    psib: &HashMap<u32, HashMap<u32, i32>>,
    unique_tokens: &[u32],
    last_token: u32,
) -> Option<u32> {
    let mut best: Option<(u32, (i32, i32))> = None;
    for &(tok, score) in ranked {
        let key = arm3_key(
            skmx_contribution(skmx, psib, unique_tokens, last_token, tok),
            score.raw(),
        );
        best = match best {
            None => Some((tok, key)),
            Some((bt, bk)) if key > bk || (key == bk && tok < bt) => Some((tok, key)),
            some => some,
        };
    }
    best.map(|(t, _)| t)
}

/// Manual additive re-derivation of the deployed engine's own combine rule
/// (`segment_argmax`: `score.raw().saturating_add(contribution)`), used only
/// as an instrument-fidelity control -- it must equal the engine's actual
/// served token on every replayed position, or this harness's `skmx`/`psib`
/// map replicas do not faithfully mirror the real SKMX/PSIB tables.
fn additive_pick(
    ranked: &[(u32, ScoreQ)],
    skmx: &HashMap<(u32, u32), HashMap<u32, i32>>,
    psib: &HashMap<u32, HashMap<u32, i32>>,
    unique_tokens: &[u32],
    last_token: u32,
) -> Option<u32> {
    let mut best: Option<(u32, i32)> = None;
    for &(tok, score) in ranked {
        let adjusted = score.raw().saturating_add(skmx_contribution(
            skmx,
            psib,
            unique_tokens,
            last_token,
            tok,
        ));
        best = match best {
            None => Some((tok, adjusted)),
            Some((bt, ba)) if adjusted > ba || (adjusted == ba && tok < bt) => {
                Some((tok, adjusted))
            }
            some => some,
        };
    }
    best.map(|(t, _)| t)
}

/// Predeclared "coverage is not the bottleneck" bar: `>= 70/87` (~0.8),
/// chosen before the run as "close to 87" per the doc comment's predeclared
/// reads.
const COVERAGE_HIGH_BAR: u64 = 70;
/// Predeclared "arm 3 recovers significant follow-through over the deployed
/// additive combine" margin: `arm3_follow >= deployed_follow + 10`.
const ARM3_SIGNIFICANT_MARGIN: u64 = 10;

#[test]
#[ignore = "heavy: needs the compiled #833 bundle (corpus+store+cover+artifacts); run with --ignored"]
fn skipmix_gap_diagnostic_904() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP skipmix_gap_diagnostic_904: no serving bundle at {}",
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
        "diagnostic is only valid on the attested #833 bundle (corpus.meta CID \
         prefix {ATTESTED_CORPUS_CID_PREFIX}); got {corpus_cid}"
    );

    let started = Instant::now();

    // === 1. reproduce the phase-0 `skipmix` favorable pairs (verbatim) =====
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
    assert_eq!(
        reference_favorable, REFERENCE_FAVORABLE_EXPECTED,
        "the reproduced skipmix favorable-pair count must match \
         docs/skipmix_confirm_897_result.json's minimal_pairs.skipmix_follow"
    );
    assert_eq!(
        base_follow, 0,
        "the suffix baseline cannot follow identical-suffix minimal pairs"
    );

    // === 2. fit + emit the deployed tables via the REAL release emitter ====
    let rows_top_k = CAP;
    let (skipmix_rows, psi_bag_rows) = skipmix_fit::fit_skipmix_tables(&corpus, rows_top_k);
    assert!(!skipmix_rows.is_empty(), "the fit must learn joint keys");
    assert!(!psi_bag_rows.is_empty(), "the fit must learn psi-bag keys");

    let mut skmx_map: HashMap<(u32, u32), HashMap<u32, i32>> = HashMap::new();
    for (c, l, entries) in &skipmix_rows {
        skmx_map.insert((*c, *l), entries.iter().copied().collect());
    }
    let mut psib_map: HashMap<u32, HashMap<u32, i32>> = HashMap::new();
    for (c, entries) in &psi_bag_rows {
        psib_map.insert(*c, entries.iter().copied().collect());
    }

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
    let mut engine = R4Engine::load_accepting_quality(EngineParts {
        graph: &graph_bytes,
        signature_artifact: &artifact_container,
        tokenizer: tokenizer_bytes.as_deref(),
        score_report: None,
    })
    .expect("re-emitted engine load");
    assert_eq!(
        engine.skipmix_tables_present(),
        (true, true),
        "the engine must consume both the SKMX and PSIB sections"
    );

    // === 3. replay the deployed lane on the favorable pairs (instrumented) =
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    #[allow(clippy::type_complexity)]
    let eval = |engine: &mut R4Engine, i: usize| -> Option<(Option<u32>, Vec<(u32, ScoreQ)>)> {
        engine.reset();
        let w = induction::context_window(&corpus, i);
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut cands = StepCandidates::default();
            let served = match engine.predict_decision_candidates_with_skipmix(&w, &mut cands) {
                Ok(PredictDecision::Serve(outcome)) => Some(outcome.token),
                _ => None,
            };
            (served, cands.ranked().to_vec())
        }))
        .ok()
    };

    let mut served = 0u64;
    let mut abstained = 0u64;
    let mut unservable = 0u64;
    let mut deployed_follow = 0u64;
    let mut coverage_pairs = 0u64;
    let mut arm2_follow = 0u64;
    let mut arm3_follow = 0u64;
    let mut instrument_mismatches = 0u64;
    let mut pair_lines: Vec<String> = Vec::new();

    for (idx, p) in favorable.iter().enumerate() {
        let ra = eval(&mut engine, p.ia);
        let rb = eval(&mut engine, p.ib);
        match &ra {
            Some((Some(_), _)) => served += 1,
            Some((None, _)) => abstained += 1,
            None => unservable += 1,
        }
        match &rb {
            Some((Some(_), _)) => served += 1,
            Some((None, _)) => abstained += 1,
            None => unservable += 1,
        }
        let (Some((served_a, ranked_a)), Some((served_b, ranked_b))) = (&ra, &rb) else {
            pair_lines.push(format!("  pair {idx}: UNSERVABLE-AT-REPLAY"));
            continue;
        };
        let served_a = *served_a;
        let served_b = *served_b;

        let w_a = induction::context_window(&corpus, p.ia);
        let w_b = induction::context_window(&corpus, p.ib);
        let uniq_a = uniq_tokens(&w_a);
        let uniq_b = uniq_tokens(&w_b);
        let last_a = w_a.last().copied().unwrap_or(u32::MAX);
        let last_b = w_b.last().copied().unwrap_or(u32::MAX);

        // instrument-fidelity control: this harness's own map-based
        // contribution replica, combined additively exactly as the engine
        // does, must reproduce the engine's actual served token.
        let manual_a = additive_pick(ranked_a, &skmx_map, &psib_map, &uniq_a, last_a);
        let manual_b = additive_pick(ranked_b, &skmx_map, &psib_map, &uniq_b, last_b);
        if manual_a != served_a || manual_b != served_b {
            instrument_mismatches += 1;
        }

        // arm 0 (ground truth, recomputed): the actual deployed lane.
        let follow0 = served_a == Some(p.ta) && served_b == Some(p.tb) && served_a != served_b;
        if follow0 {
            deployed_follow += 1;
        }

        // arm 1: candidate coverage ceiling.
        let cov_a = ranked_a.iter().any(|&(t, _)| t == p.ta);
        let cov_b = ranked_b.iter().any(|&(t, _)| t == p.tb);
        if cov_a && cov_b {
            coverage_pairs += 1;
        }

        // arm 2: reference math restricted to the real narrow candidate list.
        let narrow_a: Vec<u32> = ranked_a.iter().map(|&(t, _)| t).collect();
        let narrow_b: Vec<u32> = ranked_b.iter().map(|&(t, _)| t).collect();
        let pick2_a = argmax(&tables.skipmix_scores(&w_a, last_a, &narrow_a, lambda));
        let pick2_b = argmax(&tables.skipmix_scores(&w_b, last_b, &narrow_b, lambda));
        let follow2 = pick2_a == p.ta && pick2_b == p.tb && pick2_a != pick2_b;
        if follow2 {
            arm2_follow += 1;
        }

        // arm 3: real base ScoreQ + unit-safe (non-additive) combine, over
        // the same narrow candidate list.
        let pick3_a = arm3_pick(ranked_a, &skmx_map, &psib_map, &uniq_a, last_a);
        let pick3_b = arm3_pick(ranked_b, &skmx_map, &psib_map, &uniq_b, last_b);
        let follow3 = pick3_a == Some(p.ta) && pick3_b == Some(p.tb) && pick3_a != pick3_b;
        if follow3 {
            arm3_follow += 1;
        }

        pair_lines.push(format!(
            "  pair {idx}: cov=({cov_a},{cov_b}) served=({served_a:?},{served_b:?}) \
             arm2=({pick2_a},{pick2_b}) arm3=({pick3_a:?},{pick3_b:?}) \
             follow0={follow0} follow2={follow2} follow3={follow3}"
        ));
    }

    std::panic::set_hook(prev_hook);
    let positions = reference_favorable * 2;

    assert_eq!(
        deployed_follow, DEPLOYED_FOLLOW_EXPECTED,
        "this harness's recomputed deployed-lane follow count must match \
         docs/skipmix_lane_897_result.json's deployed_lane_follow exactly \
         (same fit, same emitter, same engine, same replay) -- a mismatch \
         means something about the bundle or code changed since #897/#903"
    );
    assert_eq!(
        instrument_mismatches, 0,
        "this harness's skmx/psib map replica must reproduce the engine's \
         own additive combine exactly on every replayed position, or arms \
         2/3 above are not measuring what they claim to"
    );
    assert!(positions > 0, "no favorable positions evaluated");

    // === 4. verdict + record ================================================
    let breadth_is_bottleneck = coverage_pairs < COVERAGE_HIGH_BAR;
    let scale_is_significant = arm3_follow >= deployed_follow + ARM3_SIGNIFICANT_MARGIN;
    let read: String = if breadth_is_bottleneck {
        format!(
            "BREADTH-BOUND: coverage ceiling {coverage_pairs}/{reference_favorable} is well \
             below the reference count -- StepCandidates::STEP_TOP_CANDIDATES=8 structurally \
             excludes at least one pair's teacher target from the served candidate list on \
             {}/{reference_favorable} pairs, before any scoring runs. No combine-rule fix alone \
             can exceed this ceiling; closing the gap needs a wider served candidate list.",
            reference_favorable - coverage_pairs
        )
    } else if scale_is_significant {
        format!(
            "SCALE-BOUND: coverage ceiling {coverage_pairs}/{reference_favorable} is high (breadth \
             is not the bottleneck), and the unit-safe combine (arm 3, {arm3_follow}/{reference_favorable}) \
             recovers materially more follow-through than the deployed additive combine \
             (arm 0, {deployed_follow}/{reference_favorable}). The additive scale mismatch between \
             quantize_rate's linear 2^20-scaled rate and ScoreQ's log-probability 2^16 scale is a \
             real, fixable defect: a bounded, low-risk fix (change the combine rule) is worth \
             attempting."
        )
    } else {
        format!(
            "INCONCLUSIVE: coverage ceiling {coverage_pairs}/{reference_favorable} is high, but \
             arm 3 ({arm3_follow}/{reference_favorable}) does not materially exceed the deployed \
             additive combine (arm 0, {deployed_follow}/{reference_favorable}). Neither breadth \
             nor the additive combine rule alone explains most of the gap; arm 2 \
             ({arm2_follow}/{reference_favorable}, the idealized formula boxed to the narrow list) \
             bounds what candidate breadth allows a perfect scorer to achieve. Further \
             investigation (e.g. the fit itself, or a genuine calibration/lambda term) would be \
             needed before attempting a fix."
        )
    };

    let elapsed = started.elapsed();
    let artifact_cid = compute_cid(&graph_bytes);

    println!("=== #904 skip-mix lane gap diagnostic (follow-up to #897) ===");
    println!("bundle              : {}", bundle.root.display());
    println!("reemit_artifact_cid : {artifact_cid}");
    println!("corpus_meta_cid     : {corpus_cid}");
    println!(
        "train / held_out    : {} / {}",
        train_positions.len(),
        held_out_positions.len()
    );
    println!(
        "reference favorable : {reference_favorable} (expected {REFERENCE_FAVORABLE_EXPECTED})"
    );
    for line in &pair_lines {
        println!("{line}");
    }
    println!(
        "servability         : served {served}, abstained {abstained}, unservable {unservable} of {positions} positions"
    );
    println!("instrument mismatches: {instrument_mismatches} (must be 0)");
    println!("arm0 deployed follow: {deployed_follow}/{reference_favorable} (expected {DEPLOYED_FOLLOW_EXPECTED})");
    println!("arm1 coverage ceiling: {coverage_pairs}/{reference_favorable}");
    println!("arm2 narrow-ref follow: {arm2_follow}/{reference_favorable}");
    println!("arm3 unit-safe follow: {arm3_follow}/{reference_favorable}");
    println!("elapsed             : {:.1}s", elapsed.as_secs_f64());
    println!("READ                : {read}");

    let mut rec = Vec::new();
    for v in [
        reference_favorable,
        deployed_follow,
        coverage_pairs,
        arm2_follow,
        arm3_follow,
        instrument_mismatches,
        positions,
        served,
        abstained,
        unservable,
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
            "  \"issue\": 904,\n",
            "  \"parent_issue\": 897,\n",
            "  \"check\": \"skipmix-lane-gap-diagnostic\",\n",
            "  \"decision_kind\": \"off-serving diagnostic (no production code changes)\",\n",
            "  \"bundle\": \"{}\",\n",
            "  \"reemit_artifact_cid\": \"{}\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"reference_favorable\": {},\n",
            "  \"reference_favorable_expected\": {},\n",
            "  \"positions\": {},\n",
            "  \"served\": {},\n",
            "  \"abstained\": {},\n",
            "  \"unservable\": {},\n",
            "  \"instrument_mismatches\": {},\n",
            "  \"arm0_deployed_follow\": {},\n",
            "  \"arm0_deployed_follow_expected\": {},\n",
            "  \"arm1_coverage_ceiling\": {},\n",
            "  \"arm2_narrow_reference_follow\": {},\n",
            "  \"arm3_unit_safe_follow\": {},\n",
            "  \"breadth_is_bottleneck\": {},\n",
            "  \"scale_is_significant\": {},\n",
            "  \"result_cid\": \"{}\",\n",
            "  \"read\": \"{}\"\n",
            "}}\n"
        ),
        bundle.root.display(),
        artifact_cid,
        corpus_cid,
        reference_favorable,
        REFERENCE_FAVORABLE_EXPECTED,
        positions,
        served,
        abstained,
        unservable,
        instrument_mismatches,
        deployed_follow,
        DEPLOYED_FOLLOW_EXPECTED,
        coverage_pairs,
        arm2_follow,
        arm3_follow,
        breadth_is_bottleneck,
        scale_is_significant,
        result_cid,
        read,
    );
    let out = repo_root()
        .join("docs")
        .join("skipmix_gap_diagnostic_904_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote               : {}", out.display());
}
