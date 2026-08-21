//! #886 — bounded minimal-pairs spot-check of the DEPLOYED segment lane on the
//! real #833 canonical bundle (S1 follow-up A of #836; parent programme #822).
//!
//! ## What this is (and is not)
//!
//! #836 lowered the #835 segment lane end-to-end onto the deployed `R4Engine`
//! serving path and closed with verdict **REVISE** (dormant): the whole-prompt
//! content effect the #834 §6.2 reference arm measured (+17.5‰, CI [15.9, 19.0])
//! sits below the pre-registered 20‰ causal floor, and the deployed lane is
//! strictly weaker still (integer-`ScoreQ` quantization + the top-8 candidate
//! bound + the bounded content ring). That verdict is by reachability arithmetic
//! — no full causal run was launched, and the #836 tests exercise the lane on
//! SYNTHETIC fixtures only.
//!
//! This harness is the cheap, bounded check #886 asks for: does the lane, when
//! **actually lowered onto the real bundle** (fit → emit → consume), realize the
//! whole-prompt-content signal on its own best cases — the exact minimal pairs
//! the §6.2 reference arm resolved — within its known ceiling? It is NOT the
//! full N≈24k causal run (that outcome is already fixed by the ceiling < floor
//! arithmetic); it replays only the tiny favorable subset.
//!
//! ## Emission-path honesty
//!
//! The segment lane's ONLY emission path is
//! `convert_r4g1::convert_with_segment_table` (the transformerless TLS→R4G1
//! converter). The RELEASED bundle's serving graph `graph/score.r4g1` is emitted
//! by a DIFFERENT path (the graph-compiler cover/score emitter, `score::
//! emit_scored_r4g1`), so it carries no PSTATE section and cannot drive the lane.
//! Therefore "the deployed lane on the real bundle" = the fitted table emitted by
//! `convert_with_segment_table` over the real store/artifacts and consumed by
//! `R4Engine` — which is exactly what #836 lowered. This harness reports whether
//! that path serves the favorable held-out windows at all (a re-emit that cannot
//! reproduce the released graph's held-out serving behavior is itself a
//! lowering-fidelity finding), and, where it serves, whether it follows.
//!
//! ## Construction (all deterministic, off no RNG)
//!
//! 1. **Reproduce the §6.2 favorable pairs.** Rebuild the reference arm's
//!    suffix/content tables from the corpus TRAIN split, mine the minimal pairs
//!    (same 2-token suffix, different story, different teacher argmax, one pair
//!    per suffix key — the §6.2 recipe), and collect exactly the pairs the
//!    reference Ψ scorer FOLLOWED. This reproduces the recorded
//!    `psi_follow = 10 / 4722` (`docs/psi_arm_834_result.json`); the count is
//!    asserted — a reproduction control.
//! 2. **Fit + emit the deployed table.** `segment_fit::fit_segment_table`
//!    (top-`CAP`=64/key, the §6.2 cap, quantized to integer ScoreQ) over the same
//!    corpus, emitted into a real R4G1 bundle with
//!    `convert_r4g1::convert_with_segment_table`, loaded into `R4Engine` — the
//!    engine consuming every fitted row is asserted (`segment_learned_rows`).
//! 3. **Replay the deployed lane.** For each favorable position, on a freshly
//!    reset serving session, prime an active `SegmentSession` with the
//!    whole-prompt window and take the served token from
//!    `predict_decision_candidates_with_segment` (the P-4 quantized top-8
//!    re-rank). Each position is classified served / abstained / UNSERVABLE (the
//!    deployed policy path faulting on a window is caught, not fatal), and a
//!    favorable pair "follows" when the lane serves each side's own teacher
//!    target and the two differ.
//!
//! ## Predeclared read (frozen before the run; § Definition of done)
//!
//!   * `< 6/10` ⇒ the deployed path does not track its own ceiling even on its
//!     best cases — a lowering-fidelity problem to fix before anything else.
//!   * `>= 6/10` ⇒ the lowering is faithful and the lane sits at its arithmetic
//!     ceiling as expected (still sub-floor; the lane stays dormant regardless).
//!
//! The lane stays dormant whatever the outcome: no `model/ids.toml` change, no
//! `CONFORMANCE.md` change. This is measurement, not deployment. RF-21/22/27/28.
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test segment_lane_spotcheck_886 \
//!       -- --ignored --nocapture
//! Env: R4_CAUSAL_BUNDLE (bundle dir; corpus + store + artifacts read from it).

#![allow(clippy::doc_lazy_continuation)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_api::capability_suite::compute_cid;
use uor_r4_api::engine::{EngineParts, PredictDecision, R4Engine};
use uor_r4_api::serving_eval::ServingBundle;
use uor_r4_core::transformerless::{compiler, convert_r4g1, runtime};
use uor_r4_graph_certify::StepCandidates;
use uor_r4_graph_compiler::{induction, segment_fit};
use uor_r4_graph_format::SegmentLaneDescriptor;
use uor_r4_graph_runtime::runtime_state::SEGMENT_STATE_CAPACITY;

// --- pre-registered constants (frozen before the run) ---------------------
/// Suffix length that defines the minimal-pair key and the suffix-local floor.
const SUFFIX_K: usize = 2;
/// Per-key cap on retained argmax (bounded segment-lane residual table, §6.2).
const CAP: usize = 64;
/// Candidate-set widths from the suffix baseline and the content evidence
/// (the reference arm's `CAND_SUFFIX` / `CAND_CONTENT`).
const CAND_SUFFIX: usize = 32;
const CAND_CONTENT: usize = 32;
/// Pre-registered content weight λ = LAMBDA_NUM / LAMBDA_DEN (as §6.2).
const LAMBDA_NUM: f64 = 1.0;
const LAMBDA_DEN: f64 = 1.0;
/// Retain all fitted content keys (the §6.2 lane bounds keys only by top-K per
/// key, not by key count); the vocab is ~49k so this is effectively unbounded.
const FIT_MAX_KEYS: usize = 1 << 20;
/// The §6.2 reference arm's recorded favorable-pair count
/// (`docs/psi_arm_834_result.json` minimal_pairs.psi_follow) — a reproduction
/// control: the mined favorable set must match it exactly.
const REFERENCE_FAVORABLE_EXPECTED: u64 = 10;
/// Predeclared read threshold: the deployed lane must follow at least this many
/// of the reference arm's favorable pairs for the lowering to be judged faithful.
const FOLLOW_MIN: u64 = 6;

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = <repo>/crates/uor-r4-api
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

/// The selected segment-lane descriptor the deployed lane consumes (the #836
/// `selected_descriptor`): bounded ring, one-boost content weight, no decay.
fn selected_descriptor() -> SegmentLaneDescriptor {
    SegmentLaneDescriptor {
        ring_capacity: SEGMENT_STATE_CAPACITY as u32,
        decay_shift: 0,
        base_w: 1 << 12,
        boost: 1 << 20,
        key_quant_id: 0,
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

/// Counts of teacher argmax under one key, capped to the top `CAP` by count
/// (ties broken by the smaller token id) — the reference arm's `Counter`.
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

/// Argmax by score, canonical tie-break (score desc, id asc).
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

/// A reproduced §6.2 favorable minimal pair (positions + teacher targets).
struct FavPair {
    ia: usize,
    ib: usize,
    ta: u32,
    tb: u32,
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle (corpus+store+artifacts); run with --ignored"]
fn segment_lane_spotcheck_886() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP segment_lane_spotcheck_886: no serving bundle at {}",
            root.display()
        );
        return;
    };
    let meta_bytes = std::fs::read(&bundle.corpus_meta).expect("corpus meta");
    let corpus = compiler::load_corpus_from(
        bundle.corpus_meta.to_str().expect("meta utf8"),
        bundle.corpus_records.to_str().expect("recs utf8"),
    )
    .expect("load corpus");
    let (train, held_out) = induction::split_positions(&corpus);
    assert!(!train.is_empty() && !held_out.is_empty(), "non-empty split");

    let started = Instant::now();

    // === 1. reproduce the §6.2 reference favorable pairs =====================
    // Build suffix + whole-prompt-content tables from TRAIN (the reference arm).
    let mut suffix_next: HashMap<(u32, u32), Counter> = HashMap::new();
    let mut content_next: HashMap<u32, Counter> = HashMap::new();
    let mut marginal = Counter::default();
    for &i in &train {
        let w = induction::context_window(&corpus, i);
        let target = corpus.t_argmax[i];
        marginal.bump(target);
        suffix_next.entry(suffix_key(&w)).or_default().bump(target);
        let mut seen: Vec<u32> = w.clone();
        seen.sort_unstable();
        seen.dedup();
        for t in seen {
            content_next.entry(t).or_default().bump(target);
        }
    }
    for c in suffix_next.values_mut() {
        c.cap_to_top(CAP);
    }
    for c in content_next.values_mut() {
        c.cap_to_top(CAP);
    }
    let marginal_tok = marginal
        .map
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
        .unwrap_or(0);

    let lambda = LAMBDA_NUM / LAMBDA_DEN;

    // Reference scorer: (baseline_pred, psi_pred) for one held-out window —
    // byte-for-byte the §6.2 `score` closure.
    let score = |w: &[u32]| -> (u32, u32) {
        let sk = suffix_key(w);
        let sfx = suffix_next.get(&sk);
        let sfx_total = sfx.map(|c| c.total()).unwrap_or(0).max(1) as f64;

        let mut content_rate: HashMap<u32, f64> = HashMap::new();
        let mut uniq: Vec<u32> = w.to_vec();
        uniq.sort_unstable();
        uniq.dedup();
        let ncontent = uniq.len().max(1) as f64;
        for t in &uniq {
            if let Some(cn) = content_next.get(t) {
                let tot = cn.total().max(1) as f64;
                for (&c, &cnt) in &cn.map {
                    *content_rate.entry(c).or_insert(0.0) += (cnt as f64 / tot) / ncontent;
                }
            }
        }

        let mut suffix_cands: Vec<u32> = Vec::new();
        if let Some(c) = sfx {
            let mut v: Vec<(u32, u32)> = c.map.iter().map(|(&k, &n)| (k, n)).collect();
            v.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            suffix_cands.extend(v.into_iter().take(CAND_SUFFIX).map(|(k, _)| k));
        }
        suffix_cands.push(marginal_tok);
        suffix_cands.sort_unstable();
        suffix_cands.dedup();

        let mut base_scores: HashMap<u32, f64> = HashMap::new();
        for &c in &suffix_cands {
            let base = sfx.and_then(|s| s.map.get(&c)).copied().unwrap_or(0) as f64 / sfx_total;
            base_scores.insert(c, base);
        }

        let mut cr: Vec<(u32, f64)> = content_rate.iter().map(|(&k, &v)| (k, v)).collect();
        cr.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
        let mut all_cands = suffix_cands.clone();
        all_cands.extend(cr.into_iter().take(CAND_CONTENT).map(|(k, _)| k));
        all_cands.sort_unstable();
        all_cands.dedup();
        let mut psi_scores: HashMap<u32, f64> = HashMap::new();
        for &c in &all_cands {
            let base = sfx.and_then(|s| s.map.get(&c)).copied().unwrap_or(0) as f64 / sfx_total;
            let cont = content_rate.get(&c).copied().unwrap_or(0.0);
            psi_scores.insert(c, base + lambda * cont);
        }
        (argmax(&base_scores), argmax(&psi_scores))
    };

    // Mine the minimal pairs (one per suffix key) and keep the reference-followed
    // ones — the §6.2 `by_suffix` loop, recording the favorable pairs' positions.
    let mut windows: Vec<Vec<u32>> = Vec::with_capacity(held_out.len());
    for &i in &held_out {
        windows.push(induction::context_window(&corpus, i));
    }
    let mut by_suffix: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (idx, w) in windows.iter().enumerate() {
        if w.len() >= SUFFIX_K {
            by_suffix.entry(suffix_key(w)).or_default().push(idx);
        }
    }
    let mut mp_total = 0u64;
    let mut favorable: Vec<FavPair> = Vec::new();
    let mut base_follow = 0u64;
    for group in by_suffix.values() {
        let mut made = 0usize;
        'outer: for a in 0..group.len() {
            for b in (a + 1)..group.len() {
                let (xa, xb) = (group[a], group[b]);
                let (ia, ib) = (held_out[xa], held_out[xb]);
                if corpus.story[ia] == corpus.story[ib] {
                    continue;
                }
                let (ta, tb) = (corpus.t_argmax[ia], corpus.t_argmax[ib]);
                if ta == tb {
                    continue;
                }
                mp_total += 1;
                let (ba, pa) = score(&windows[xa]);
                let (bb, pb) = score(&windows[xb]);
                if pa == ta && pb == tb && pa != pb {
                    favorable.push(FavPair { ia, ib, ta, tb });
                }
                if ba == ta && bb == tb && ba != bb {
                    base_follow += 1;
                }
                made += 1;
                if made >= 1 {
                    break 'outer;
                }
            }
        }
    }
    let reference_favorable = favorable.len() as u64;

    // === 2. fit + emit the deployed table into a real bundle ================
    let rows = segment_fit::fit_segment_table(&corpus, CAP, FIT_MAX_KEYS);
    assert!(!rows.is_empty(), "the fit must learn content keys");

    let art_bytes = std::fs::read(&bundle.teacher).expect("teacher artifacts");
    let artifacts = compiler::parse_artifacts(&art_bytes).expect("parse artifacts");
    let store_bytes = std::fs::read(bundle.root.join("tless_store.bin")).expect("store bytes");
    let store = runtime::parse_store(&store_bytes).expect("parse store (u32)");
    let calibration: Option<compiler::HammingCalibrationReport> =
        std::fs::read(bundle.root.join("hamming_calibration.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok());
    assert!(
        calibration.is_some(),
        "the deployed bundle carries hamming_calibration.json; the base graph needs it"
    );
    let descriptor = selected_descriptor();
    let (graph_bytes, _report) = convert_r4g1::convert_with_segment_table(
        &art_bytes,
        &artifacts,
        &store,
        &store_bytes,
        calibration.as_ref(),
        &descriptor,
        &rows,
    )
    .expect("convert real bundle with fitted segment table");

    let tokenizer_bytes = std::fs::read(bundle.root.join("tokenizer.bin")).ok();
    let score_report = bundle
        .graph
        .parent()
        .and_then(|p| std::fs::read(p.join("score_report.json")).ok())
        .filter(|b| serde_json::from_slice::<serde_json::Value>(b).is_ok());
    // Load once to assert the fit reached the engine intact (the "consume" leg).
    let probe = R4Engine::load_accepting_quality(EngineParts {
        graph: &graph_bytes,
        signature_artifact: &art_bytes,
        tokenizer: tokenizer_bytes.as_deref(),
        score_report: score_report.as_deref(),
    })
    .expect("re-emitted engine load");
    assert!(
        probe.segment_session().is_active(),
        "the re-emitted bundle must activate the deployed segment lane"
    );
    assert_eq!(
        probe.segment_learned_rows(),
        Some(rows.len()),
        "the engine must consume every fitted learned row"
    );
    drop(probe);

    // === 3. replay the deployed lane on the favorable pairs (robust) ========
    // Each position runs on its own freshly reset serving session (reset()
    // rebuilds the step scratch at the widened width and clears the widen-once
    // memory — a clean prompt). A serving-policy fault on one window (e.g. a
    // widen re-probe the re-emitted graph cannot satisfy) is caught and
    // classified UNSERVABLE rather than aborting the whole measurement.
    let mut engine = R4Engine::load_accepting_quality(EngineParts {
        graph: &graph_bytes,
        signature_artifact: &art_bytes,
        tokenizer: tokenizer_bytes.as_deref(),
        score_report: score_report.as_deref(),
    })
    .expect("re-emitted engine load");

    // Silence the per-window panic backtrace while we classify unservable ones.
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    // Deployed-lane outcome for one position:
    //   Some(Some(tok)) served; Some(None) abstained/no-serve; None UNSERVABLE.
    let eval = |engine: &mut R4Engine, i: usize| -> Option<Option<u32>> {
        engine.reset();
        let w = induction::context_window(&corpus, i);
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut session = engine.segment_session();
            session.fold_prompt(&w);
            let mut cands = StepCandidates::default();
            match engine.predict_decision_candidates_with_segment(&w, &mut cands, &session) {
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

    // === 4. verdict + record ===============================================
    let faithful = deployed_follow >= FOLLOW_MIN;
    let read = if positions > 0 && unservable == positions {
        "UNSERVABLE-AT-EMISSION — the re-emitted (convert_with_segment_table) serving graph cannot serve the reference arm's favorable held-out windows under the deployed policy, so the deployed lane could not be exercised on the real bundle at all. A lowering-fidelity gap at the EMISSION level: the segment lane's only emitter does not reproduce the released cover/score serving graph's held-out serving behavior (the released graph/score.r4g1 comes from a different emitter and carries no PSTATE)."
    } else if faithful {
        "FAITHFUL — the deployed quantized top-8 lane follows >= 6/10 of the reference arm's favorable pairs; the fit->emit->consume lowering tracks the §6.2 ceiling (lane still sub-floor and dormant)"
    } else {
        "LOWERING-FIDELITY GAP — the deployed lane follows < 6/10 of the reference arm's favorable pairs; the quantized top-8 serving path does not track its own ceiling on its best cases"
    };

    let elapsed = started.elapsed();
    let artifact_cid = compute_cid(&graph_bytes);
    let corpus_cid = compute_cid(&meta_bytes);

    println!("=== #886 deployed segment-lane bounded spot-check ===");
    println!("bundle              : {}", bundle.root.display());
    println!("reemit_artifact_cid : {artifact_cid}");
    println!("corpus_meta_cid     : {corpus_cid}");
    println!("train / held_out    : {} / {}", train.len(), held_out.len());
    println!("fitted learned rows : {}", rows.len());
    println!("minimal pairs mined : {mp_total}");
    println!(
        "reference favorable : {reference_favorable} (expected {REFERENCE_FAVORABLE_EXPECTED}); reference baseline-follow {base_follow}"
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
        rows.len() as u64,
        train.len() as u64,
        held_out.len() as u64,
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
            "  \"issue\": 886,\n",
            "  \"follows\": 836,\n",
            "  \"check\": \"deployed-segment-lane-bounded-minimal-pairs-spot-check\",\n",
            "  \"decision_kind\": \"bounded spot-check (no full causal run launched)\",\n",
            "  \"bundle\": \"{}\",\n",
            "  \"reemit_artifact_cid\": \"{}\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"emission_path\": \"convert_with_segment_table (transformerless emitter; the released graph/score.r4g1 uses a different emitter and carries no PSTATE)\",\n",
            "  \"train\": {},\n",
            "  \"held_out\": {},\n",
            "  \"fitted_learned_rows\": {},\n",
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
        train.len(),
        held_out.len(),
        rows.len(),
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
        .join("segment_lane_886_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote               : {}", out.display());

    // === structural guards (non-vacuous machinery, not a vacuous green) =====
    // Reproduction control: the mined favorable set matches the recorded §6.2
    // psi_follow exactly.
    assert_eq!(
        reference_favorable, REFERENCE_FAVORABLE_EXPECTED,
        "the reproduced §6.2 favorable-pair count must match docs/psi_arm_834_result.json"
    );
    // Control degeneracy: the suffix baseline cannot follow identical-suffix
    // pairs (§6.2 baseline_follow = 0).
    assert_eq!(
        base_follow, 0,
        "the suffix baseline cannot follow identical-suffix minimal pairs"
    );
    assert!(positions > 0, "no favorable positions evaluated");
}
