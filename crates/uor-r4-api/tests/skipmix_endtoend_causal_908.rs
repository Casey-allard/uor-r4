//! #908 -- end-to-end DEPLOYED causal run for the injected-candidate skip-mix
//! lane (follow-up to #897/#904/#906; child of #822 / S1).
//!
//! #906 lifted the deployed lane's FIDELITY spot-check to 58/87 (clears the
//! 60% bar). Fidelity is not promotion: per the #822 §6-4 re-entry decision
//! the frozen **20permille end-to-end causal floor** is the S1 promotion gate.
//! This harness measures the lane's causal effect on deployed held-out top-1
//! against that floor -- TEACHER-FREE (deployed base vs deployed skip-mix
//! served tokens, both scored against the corpus's recorded `t_argmax`).
//!
//! Design (apples-to-apples, isolates exactly the lane's contribution):
//!   * one-time: fit + recompile the graph sections from the bundle's TRAIN
//!     split, exactly as `skipmix_candidate_injection_906.rs` does;
//!   * emit THREE graphs from the SAME recompiled sections, differing only in
//!     the two optional lane sections. `base` carries empty SKMX/PSIB, so
//!     `predict_decision_candidates_with_skipmix` is byte-identical to plain
//!     base (absent-section identity). `skip` carries the real fitted
//!     SKMX/PSIB (`fit_skipmix_tables`). `null` carries SKMX/PSIB fitted on a
//!     corpus whose TRAIN targets are deterministically rotated (window and
//!     target association broken, target multiset preserved) -- the
//!     conditioning-specificity null.
//!   * replay held-out positions through each engine's
//!     `predict_decision_candidates_with_skipmix` and compare served tokens
//!     to `corpus.t_argmax`.
//!
//! Two phases via `R4_SKIPMIX_PHASE` (default `probe`):
//!   probe -- the AGENTS.md binding cheap instrument: a subsample
//!            (`R4_PROBE_N`, default 6000) reporting the reachability ceiling
//!            (changed/n) and the subsample net delta; no null, no record.
//!   full  -- all held-out positions + the label-shuffle null + paired 95%
//!            CIs; writes `docs/skipmix_endtoend_causal_908.md` and
//!            `docs/skipmix_endtoend_causal_908_result.json`.
//!
//! Off-serving harness; gated behind `--ignored` (same discipline as
//! #897/#904/#906). Needs the attested #833 bundle.
//!
//! Run:
//!   R4_SKIPMIX_PHASE=probe cargo test -p uor-r4-api --release \
//!       --test skipmix_endtoend_causal_908 -- --ignored --nocapture
//!   R4_SKIPMIX_PHASE=full  cargo test -p uor-r4-api --release \
//!       --test skipmix_endtoend_causal_908 -- --ignored --nocapture

#![allow(clippy::doc_lazy_continuation)]

use std::path::PathBuf;
use std::time::Instant;

use uor_r4_api::capability_suite::compute_cid;
use uor_r4_api::engine::{EngineParts, PredictDecision, R4Engine};
use uor_r4_api::serving_eval::ServingBundle;
use uor_r4_core::transformerless::{compiler, runtime};
use uor_r4_graph_certify as score;
use uor_r4_graph_certify::StepCandidates;
use uor_r4_graph_compiler::{induction, skipmix_fit};

/// Per-key retained-count cap for the fitted lane tables (#835), identical to
/// the deployed `#906` fit.
const CAP: usize = uor_r4_graph_compiler::segment_fit::DEFAULT_TOP_K;
/// The frozen S1 end-to-end promotion floor (permille), recorded on #822.
const CAUSAL_FLOOR_PERMILLE: f64 = 20.0;
/// The attested #833 bundle's `corpus.meta` CID prefix -- the run is only
/// valid on that bundle.
const ATTESTED_CORPUS_CID_PREFIX: &str = "blake3:aa9d1767";
/// Default reachability-probe subsample size.
const PROBE_DEFAULT_N: usize = 6000;

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

/// Normal-approx 95% CI for a rate, in permille -- verbatim convention from
/// `skipmix_confirm_897.rs`.
fn ci95_permille(hits: u64, n: u64) -> (f64, f64, f64) {
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let p = hits as f64 / n as f64;
    let half = 1.96 * (p * (1.0 - p) / n as f64).sqrt();
    (
        p * 1000.0,
        (p - half).max(0.0) * 1000.0,
        (p + half).min(1.0) * 1000.0,
    )
}

/// Paired-difference mean + normal-approx 95% CI, in permille, over per-item
/// deltas in {-1,0,1} -- verbatim convention from `skipmix_confirm_897.rs`.
fn paired_delta_permille(d: &[i8]) -> (f64, f64, f64) {
    let n = d.len();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let mean = d.iter().map(|&x| x as f64).sum::<f64>() / n as f64;
    let var = d
        .iter()
        .map(|&x| {
            let dx = x as f64 - mean;
            dx * dx
        })
        .sum::<f64>()
        / n as f64;
    let half = 1.96 * (var / n as f64).sqrt();
    (
        mean * 1000.0,
        (mean - half) * 1000.0,
        (mean + half) * 1000.0,
    )
}

fn phase() -> String {
    std::env::var("R4_SKIPMIX_PHASE").unwrap_or_else(|_| "probe".to_string())
}

fn probe_n() -> usize {
    std::env::var("R4_PROBE_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(PROBE_DEFAULT_N)
}

/// One deployed prediction: the served top-1 token, or `None` for
/// abstain / unservable / panic (all count as a top-1 miss). Panics are
/// caught so a single pathological window cannot abort the whole replay.
fn served_token(engine: &mut R4Engine, w: &[u32]) -> Option<u32> {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        engine.reset();
        let mut cands = StepCandidates::default();
        match engine.predict_decision_candidates_with_skipmix(w, &mut cands) {
            Ok(PredictDecision::Serve(outcome)) => Some(outcome.token),
            _ => None,
        }
    }));
    r.unwrap_or(None)
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle; run with --ignored (see module docs)"]
fn skipmix_endtoend_causal_908() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP skipmix_endtoend_causal_908: no serving bundle at {}",
            root.display()
        );
        return;
    };
    let is_full = phase() == "full";
    let started = Instant::now();

    // === load corpus + split =================================================
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
        "run is only valid on the attested #833 bundle (corpus.meta CID prefix \
         {ATTESTED_CORPUS_CID_PREFIX}); got {corpus_cid}"
    );

    // === fit the deployed lane tables (real corpus) ==========================
    let (skip_rows, psi_rows) = skipmix_fit::fit_skipmix_tables(&corpus, CAP);
    assert!(!skip_rows.is_empty(), "the fit must learn joint keys");
    assert!(!psi_rows.is_empty(), "the fit must learn psi-bag keys");

    // === recompile the shared graph sections (verbatim #906 machinery) =======
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

    // === label-shuffle null tables (full phase only) =========================
    // Rotate TRAIN targets by a large fixed offset: window<->target broken,
    // target multiset preserved, fully deterministic (no RNG). The held-out
    // evaluation always scores against the pristine `corpus.t_argmax`.
    let (null_skip_rows, null_psi_rows) = if is_full {
        let mut corpus_null = corpus.clone();
        corpus_null.hidden = None;
        let n = corpus_null.t_argmax.len();
        let off = n / 2 + 1;
        let orig = corpus.t_argmax.clone();
        for (i, slot) in corpus_null.t_argmax.iter_mut().enumerate() {
            *slot = orig[(i + off) % n];
        }
        skipmix_fit::fit_skipmix_tables(&corpus_null, CAP)
    } else {
        (Vec::new(), Vec::new())
    };

    // === emit base / skip / null graphs from the SAME sections ===============
    let empty_skip: Vec<uor_r4_graph_format::SkipmixRowInput> = Vec::new();
    let empty_psi: Vec<(u32, Vec<(u32, i32)>)> = Vec::new();
    let emit = |sk: &[uor_r4_graph_format::SkipmixRowInput], pb: &[(u32, Vec<(u32, i32)>)]| {
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
            skipmix_rows: sk,
            psi_bag_rows: pb,
        };
        let (graph_bytes, _info) = score::emit_scored_r4g1(
            &artifact_container,
            (&meta_bytes, &recs_bytes),
            vocab,
            &sections,
        );
        graph_bytes
    };
    let base_graph = emit(&empty_skip, &empty_psi);
    let skip_graph = emit(&skip_rows, &psi_rows);
    let null_graph = if is_full {
        emit(&null_skip_rows, &null_psi_rows)
    } else {
        Vec::new()
    };

    let tokenizer_bytes = std::fs::read(bundle.root.join("tokenizer.bin")).ok();
    let load = |graph: &[u8]| {
        R4Engine::load_accepting_quality(EngineParts {
            graph,
            signature_artifact: &artifact_container,
            tokenizer: tokenizer_bytes.as_deref(),
            score_report: None,
        })
        .expect("engine load")
    };
    let mut engine_base = load(&base_graph);
    let mut engine_skip = load(&skip_graph);
    assert_eq!(
        engine_base.skipmix_tables_present(),
        (false, false),
        "the base engine must carry NO skip-mix sections (absent-section base)"
    );
    assert_eq!(
        engine_skip.skipmix_tables_present(),
        (true, true),
        "the skip engine must consume both the SKMX and PSIB sections"
    );
    let mut engine_null = if is_full {
        let e = load(&null_graph);
        assert_eq!(e.skipmix_tables_present(), (true, true));
        Some(e)
    } else {
        None
    };

    // === replay ==============================================================
    let eval_positions: &[usize] = if is_full {
        &held_out_positions
    } else {
        let k = probe_n().min(held_out_positions.len());
        &held_out_positions[..k]
    };
    let n = eval_positions.len() as u64;

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    let mut base_hits = 0u64;
    let mut skip_hits = 0u64;
    let mut null_hits = 0u64;
    let mut base_served = 0u64;
    let mut skip_served = 0u64;
    let mut changed = 0u64;
    let mut toward = 0u64; // base wrong -> skip right (the lane helps)
    let mut away = 0u64; // base right -> skip wrong (the lane hurts)
    let mut neutral_changed = 0u64; // changed, but both miss the target
    let mut d_skip: Vec<i8> = Vec::with_capacity(eval_positions.len());
    let mut d_null: Vec<i8> = Vec::with_capacity(if is_full { eval_positions.len() } else { 0 });

    for &i in eval_positions {
        let w = induction::context_window(&corpus, i);
        let target = corpus.t_argmax[i];
        let bt = served_token(&mut engine_base, &w);
        let st = served_token(&mut engine_skip, &w);
        let bh = bt == Some(target);
        let sh = st == Some(target);
        base_hits += u64::from(bh);
        skip_hits += u64::from(sh);
        base_served += u64::from(bt.is_some());
        skip_served += u64::from(st.is_some());
        d_skip.push(i8::from(sh) - i8::from(bh));
        if bt != st {
            changed += 1;
            if sh && !bh {
                toward += 1;
            } else if bh && !sh {
                away += 1;
            } else {
                neutral_changed += 1;
            }
        }
        if let Some(engine_null) = engine_null.as_mut() {
            let nt = served_token(engine_null, &w);
            let nh = nt == Some(target);
            null_hits += u64::from(nh);
            d_null.push(i8::from(nh) - i8::from(bh));
        }
    }
    std::panic::set_hook(prev_hook);

    // === statistics ==========================================================
    let (base_r, base_lo, base_hi) = ci95_permille(base_hits, n);
    let (skip_r, skip_lo, skip_hi) = ci95_permille(skip_hits, n);
    let (skip_delta, skip_delta_lo, skip_delta_hi) = paired_delta_permille(&d_skip);
    let ceiling_permille = changed as f64 / n as f64 * 1000.0;
    let ceiling_ok = ceiling_permille >= CAUSAL_FLOOR_PERMILLE;
    let net_ok = skip_delta > 0.0;
    let elapsed = started.elapsed();

    println!(
        "=== #908 skip-mix end-to-end DEPLOYED causal run ({}) ===",
        phase()
    );
    println!("bundle              : {}", bundle.root.display());
    println!("corpus_meta_cid     : {corpus_cid}");
    println!(
        "train / held_out    : {} / {}",
        train_positions.len(),
        held_out_positions.len()
    );
    println!("positions evaluated : {n}");
    println!(
        "base served/top1    : {base_served}/{n} served, {base_hits} top1 = {base_r:.2}permille [{base_lo:.2},{base_hi:.2}]"
    );
    println!(
        "skip served/top1    : {skip_served}/{n} served, {skip_hits} top1 = {skip_r:.2}permille [{skip_lo:.2},{skip_hi:.2}]"
    );
    println!(
        "paired skip-vs-base : {skip_delta:.2}permille [{skip_delta_lo:.2}, {skip_delta_hi:.2}]"
    );
    println!(
        "reachability        : changed {changed}/{n} = {ceiling_permille:.2}permille (ceiling); toward {toward}, away {away}, neutral {neutral_changed}"
    );
    println!("frozen floor        : {CAUSAL_FLOOR_PERMILLE:.1}permille");
    println!("elapsed             : {:.1}s", elapsed.as_secs_f64());

    // sanity: the null-collapse machinery requires the arm to read positive
    // before its separation binds; but the reproduction/structural asserts
    // above always hold. The floor comparison is a REPORTED verdict, never a
    // test failure -- a sub-floor deployed delta is legitimate S1 evidence.
    if !is_full {
        println!(
            "PROBE GATE          : ceiling_ok={ceiling_ok} (>= {CAUSAL_FLOOR_PERMILLE:.0}permille) net_ok={net_ok} (>0) => proceed_to_full={}",
            ceiling_ok && net_ok
        );
        println!(
            "probe verdict       : {}",
            if ceiling_ok && net_ok {
                "PROCEED -- reachability ceiling clears the floor and net delta is positive; launch the full run"
            } else if !ceiling_ok {
                "STOP -- reachability ceiling below the 20permille floor; the lane cannot clear it at any scale (record REVISE, do not launch full)"
            } else {
                "STOP -- net delta not positive on the subsample; do not launch full"
            }
        );
        return;
    }

    // === full-phase null + verdict ==========================================
    let (null_r, null_lo, null_hi) = ci95_permille(null_hits, n);
    let (null_delta, null_delta_lo, null_delta_hi) = paired_delta_permille(&d_null);
    let floor_cleared = skip_delta_lo >= CAUSAL_FLOOR_PERMILLE;
    let null_collapses = null_delta <= 0.0;
    let promote_recommended = floor_cleared && null_collapses;

    println!(
        "null served/top1    : {null_hits} top1 = {null_r:.2}permille [{null_lo:.2},{null_hi:.2}]"
    );
    println!(
        "paired null-vs-base : {null_delta:.2}permille [{null_delta_lo:.2}, {null_delta_hi:.2}]"
    );
    println!("floor cleared (lo>=20): {floor_cleared}");
    println!("null collapses (<=0): {null_collapses}");
    println!(
        "RECOMMENDATION      : {}",
        if promote_recommended {
            "PROMOTE (evidence clears the frozen 20permille floor and the null collapses) -- maintainer verdict required (#822)"
        } else if !floor_cleared {
            "REVISE / retain-dormant -- deployed causal delta does not clear the frozen 20permille floor"
        } else {
            "REVISE -- floor cleared but the conditioning-specificity null did NOT collapse; signal not lane-specific"
        }
    );

    let artifact_cid = compute_cid(&skip_graph);
    let base_artifact_cid = compute_cid(&base_graph);
    let result_cid = compute_cid(
        format!(
            "{base_hits}:{skip_hits}:{null_hits}:{n}:{changed}:{toward}:{away}:{artifact_cid}:{corpus_cid}"
        )
        .as_bytes(),
    );
    println!("base_artifact_cid   : {base_artifact_cid}");
    println!("skip_artifact_cid   : {artifact_cid}");
    println!("result_cid          : {result_cid}");

    let json = format!(
        concat!(
            "{{\n",
            "  \"issue\": 908,\n",
            "  \"parent_issues\": [897, 904, 906, 822],\n",
            "  \"check\": \"skipmix-endtoend-deployed-causal\",\n",
            "  \"execution_scope\": \"deployed R4Engine, teacher-free (recorded t_argmax labels)\",\n",
            "  \"bundle\": \"{}\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"base_artifact_cid\": \"{}\",\n",
            "  \"skip_artifact_cid\": \"{}\",\n",
            "  \"positions\": {},\n",
            "  \"base_top1\": {},\n",
            "  \"skip_top1\": {},\n",
            "  \"null_top1\": {},\n",
            "  \"base_rate_permille\": {:.3},\n",
            "  \"skip_rate_permille\": {:.3},\n",
            "  \"paired_skip_vs_base_permille\": {:.3},\n",
            "  \"paired_skip_vs_base_lo\": {:.3},\n",
            "  \"paired_skip_vs_base_hi\": {:.3},\n",
            "  \"paired_null_vs_base_permille\": {:.3},\n",
            "  \"paired_null_vs_base_lo\": {:.3},\n",
            "  \"reachability_changed\": {},\n",
            "  \"reachability_ceiling_permille\": {:.3},\n",
            "  \"toward\": {},\n",
            "  \"away\": {},\n",
            "  \"neutral_changed\": {},\n",
            "  \"frozen_floor_permille\": {:.1},\n",
            "  \"floor_cleared\": {},\n",
            "  \"null_collapses\": {},\n",
            "  \"promote_recommended\": {},\n",
            "  \"result_cid\": \"{}\"\n",
            "}}\n"
        ),
        bundle.root.display(),
        corpus_cid,
        base_artifact_cid,
        artifact_cid,
        n,
        base_hits,
        skip_hits,
        null_hits,
        base_r,
        skip_r,
        skip_delta,
        skip_delta_lo,
        skip_delta_hi,
        null_delta,
        null_delta_lo,
        changed,
        ceiling_permille,
        toward,
        away,
        neutral_changed,
        CAUSAL_FLOOR_PERMILLE,
        floor_cleared,
        null_collapses,
        promote_recommended,
        result_cid,
    );
    let out = repo_root()
        .join("docs")
        .join("skipmix_endtoend_causal_908_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote               : {}", out.display());

    // === regression pins (full phase) =======================================
    // These counts reproduce exactly on the attested #833 bundle: the emit is
    // deterministic and the deployed engine has no RNG/clock/HashMap-order
    // dependence. A mismatch means the bundle or the lowering code changed.
    // First measured 2026-08-21 (result_cid blake3:e32e4e33…).
    assert_eq!(base_hits, 19_372, "base top-1 count drift");
    assert_eq!(skip_hits, 21_424, "skip top-1 count drift");
    assert_eq!(null_hits, 2_281, "label-shuffle null top-1 count drift");
    assert_eq!(changed, 39_360, "reachability changed-count drift");
    assert_eq!(toward, 6_651, "toward-count drift");
    assert_eq!(away, 4_599, "away-count drift");
    assert!(
        floor_cleared,
        "deployed causal delta paired 95% lower bound must clear the frozen \
         20permille floor (measured +28.45permille [25.57, 31.32])"
    );
    assert!(
        null_collapses,
        "the conditioning-specificity (label-shuffle) null must collapse (<= 0); \
         measured -236.95permille"
    );
}
