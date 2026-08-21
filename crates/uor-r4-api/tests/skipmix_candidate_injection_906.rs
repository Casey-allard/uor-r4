//! #906 -- confirms the candidate-injection fix (follow-up to #897/#904)
//! against the REAL production serving path, not a simulated harness.
//!
//! #904 diagnosed the #897 LOWERING-FIDELITY GAP
//! (`docs/skipmix_lane_897_result.json`: deployed-lane follow 41/87, below
//! the predeclared 53/87 = 60% bar) as breadth-bound: 35/87 favorable
//! pairs have at least one side's teacher target absent from the base
//! engine's own decided candidate list (`StepCandidates::ranked()`,
//! capped at `STEP_TOP_CANDIDATES = 8`), before any skip-mix scoring runs.
//! Every width/selection lever available to the base engine's own
//! candidate generation was then tested (off-serving) and found not to
//! close it (#906 issue body); separately, 45/45 (100%) of the missing
//! pair-sides were found to already have their teacher target recorded as
//! a co-occurrence partner in the SKMX/PSIB tables -- the deployed lane
//! simply never had a code path to use that knowledge, since it only
//! re-ranked the base engine's already-decided list. `crates/uor-r4-api/
//! src/engine.rs`'s `skipmix_injected_argmax`/`skipmix_injected_lane_
//! attribution` (this issue) fix that: they extend the candidate space to
//! every SKMX/PSIB-known token, combined under the non-additive
//! "unit-safe" rule #904 arm 3 validated.
//!
//! A prototype-measured simulation (off-serving, same mining/fit/emit
//! machinery as this test) projected 58/87 (66.7%) under that combine
//! rule -- clearing the 60% bar. This test reuses the exact same
//! mining/fit/emit machinery as `skipmix_gap_diagnostic_904.rs`
//! (same corpus, same fit, same real bundle, same 87 reference-favorable
//! pairs), but replays them through the REAL, now-patched
//! `predict_decision_candidates_with_skipmix` rather than a simulated
//! combine, to confirm the production code path actually delivers the
//! projected improvement.
//!
//! Off-serving; gated behind `--ignored`, same discipline as
//! `skipmix_gap_diagnostic_904.rs`. Result written to
//! `docs/skipmix_candidate_injection_906_result.json`.
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test skipmix_candidate_injection_906 \
//!       -- --ignored --nocapture
//! Env: R4_CAUSAL_BUNDLE (bundle dir), same default as #897/#904's harness.

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

// --- pre-registered constants (identical to skipmix_gap_diagnostic_904.rs) -
const SUFFIX_K: usize = 2;
const CAP: usize = uor_r4_graph_compiler::segment_fit::DEFAULT_TOP_K;
const CAND_SUFFIX: usize = 32;
const CAND_CONTENT: usize = 32;
const CAND_JOINT: usize = 32;
const LAMBDA_NUM: f64 = 1.0;
const LAMBDA_DEN: f64 = 1.0;
const REFERENCE_FAVORABLE_EXPECTED: u64 = 87;
/// The #897/#904 recorded deployed-lane follow count under the OLD
/// re-rank-only combine -- reproduced here only as a before/after anchor,
/// not asserted (this test exercises the NEW combine).
const OLD_DEPLOYED_FOLLOW: u64 = 41;
/// The predeclared #897 fidelity bar (60% of 87).
const FIDELITY_BAR: u64 = 53;
/// The #906 confirmed deployed-lane follow count under the new
/// candidate-injection combine, against the real production code path
/// (measured by this test's own first clean run; see
/// `docs/skipmix_candidate_injection_906_result.json`).
const NEW_DEPLOYED_FOLLOW_EXPECTED: u64 = 58;
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
/// `skipmix_gap_diagnostic_904.rs`.
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

/// The reference tables needed to mine the same 87 favorable pairs --
/// verbatim from `skipmix_gap_diagnostic_904.rs::Tables` (trimmed to only
/// what mining needs; this test does not re-run arms 1-3).
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
    /// `skipmix_gap_diagnostic_904.rs::Tables::skipmix_scores` -- used only
    /// to mine the same 87 favorable pairs here, not to re-score anything.
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

#[test]
#[ignore = "heavy: needs the compiled #833 bundle (corpus+store+cover+artifacts); run with --ignored"]
fn skipmix_candidate_injection_906() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP skipmix_candidate_injection_906: no serving bundle at {}",
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
        "confirmation is only valid on the attested #833 bundle (corpus.meta CID \
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
        "the reproduced skipmix favorable-pair count must match #904's \
         (same mining logic, same bundle)"
    );

    // === 2. fit + emit the deployed tables via the REAL release emitter ====
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

    // === 3. replay the REAL (patched) deployed lane on the favorable pairs =
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
        let ra = eval(&mut engine, p.ia);
        let rb = eval(&mut engine, p.ib);
        match ra {
            Some(Some(_)) => served += 1,
            Some(None) => abstained += 1,
            None => unservable += 1,
        }
        match rb {
            Some(Some(_)) => served += 1,
            Some(None) => abstained += 1,
            None => unservable += 1,
        }
        let (Some(served_a), Some(served_b)) = (ra, rb) else {
            pair_lines.push(format!("  pair {idx}: UNSERVABLE-AT-REPLAY"));
            continue;
        };
        let follow = served_a == Some(p.ta) && served_b == Some(p.tb) && served_a != served_b;
        if follow {
            deployed_follow += 1;
        }
        pair_lines.push(format!(
            "  pair {idx}: served=({served_a:?},{served_b:?}) follow={follow}"
        ));
    }

    std::panic::set_hook(prev_hook);
    let positions = reference_favorable * 2;
    assert!(positions > 0, "no favorable positions evaluated");

    let elapsed = started.elapsed();
    let artifact_cid = compute_cid(&graph_bytes);

    println!("=== #906 skip-mix candidate-injection confirmation (follow-up to #897/#904) ===");
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
    println!(
        "deployed follow (NEW, candidate-injection combine): {deployed_follow}/{reference_favorable}"
    );
    println!(
        "deployed follow (OLD, re-rank-only combine, #897/#904 record): {OLD_DEPLOYED_FOLLOW}/{reference_favorable}"
    );
    println!("predeclared 60% fidelity bar: {FIDELITY_BAR}/{reference_favorable}");
    println!("elapsed             : {:.1}s", elapsed.as_secs_f64());

    assert_eq!(
        deployed_follow, NEW_DEPLOYED_FOLLOW_EXPECTED,
        "the real, patched predict_decision_candidates_with_skipmix must \
         reproduce this test's own confirmed measurement exactly (same fit, \
         same emitter, same engine, same replay) -- a mismatch means \
         something about the bundle or the candidate-injection code changed \
         since this measurement was taken"
    );
    assert!(
        deployed_follow >= FIDELITY_BAR,
        "the candidate-injection fix must clear the predeclared 60% \
         fidelity bar ({FIDELITY_BAR}/{reference_favorable}); got \
         {deployed_follow}/{reference_favorable}"
    );
    assert!(
        deployed_follow > OLD_DEPLOYED_FOLLOW,
        "the candidate-injection fix must strictly improve on the #897/#904 \
         recorded re-rank-only follow count ({OLD_DEPLOYED_FOLLOW}/{reference_favorable}); \
         got {deployed_follow}/{reference_favorable}"
    );

    let result_cid = compute_cid(
        format!("{deployed_follow}:{reference_favorable}:{artifact_cid}:{corpus_cid}").as_bytes(),
    );
    println!("result_cid          : {result_cid}");

    let json = format!(
        concat!(
            "{{\n",
            "  \"issue\": 906,\n",
            "  \"parent_issues\": [897, 904],\n",
            "  \"check\": \"skipmix-candidate-injection-confirmation\",\n",
            "  \"decision_kind\": \"real production code path (predict_decision_candidates_with_skipmix, patched)\",\n",
            "  \"bundle\": \"{}\",\n",
            "  \"reemit_artifact_cid\": \"{}\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"reference_favorable\": {},\n",
            "  \"positions\": {},\n",
            "  \"served\": {},\n",
            "  \"abstained\": {},\n",
            "  \"unservable\": {},\n",
            "  \"deployed_follow_new\": {},\n",
            "  \"deployed_follow_old_897_904\": {},\n",
            "  \"fidelity_bar\": {},\n",
            "  \"clears_fidelity_bar\": {},\n",
            "  \"result_cid\": \"{}\"\n",
            "}}\n"
        ),
        bundle.root.display(),
        artifact_cid,
        corpus_cid,
        reference_favorable,
        positions,
        served,
        abstained,
        unservable,
        deployed_follow,
        OLD_DEPLOYED_FOLLOW,
        FIDELITY_BAR,
        deployed_follow >= FIDELITY_BAR,
        result_cid,
    );
    let out = repo_root()
        .join("docs")
        .join("skipmix_candidate_injection_906_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote               : {}", out.display());
}
