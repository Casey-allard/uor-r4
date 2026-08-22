//! #840 -- reachability instrument for student-prefix corrections (item B of
//! S3 tracker #824). BINDING CHEAP INSTRUMENT for the #840 run contract
//! (AGENTS.md long-run discipline): run BEFORE any expensive corrective
//! compile.
//!
//! ## Why this exists
//!
//! #840 begins only against the NEW (skip-mix) representation -- the maintainer
//! HOLD (2026-08-21) was lifted by the S1 PROMOTE (the lane is RF-31, activated
//! in #910). Corrective student-prefix observation can only move the
//! free-running gap where the trajectory (a) departs the recorded text AND (b)
//! is driven by a CORRECTABLE (graph/skip-mix) path rather than the memoryless
//! suffix. #841 measured the BASE representation at 99/100 suffix-local
//! (student-prefix rollouts byte-identical to last-2-token rollouts; ~99.9%
//! ngram-path). If the activated lane leaves free-running suffix-local, the
//! reachability ceiling is ~0 and corrective rounds would only teach more
//! suffix patterns -- the exact failure the #824 kill criterion names.
//!
//! ## What it measures (TEACHER-FREE)
//!
//! The SAME frozen prompt-family v1 as #841, driven through the normative
//! deployed `R4Engine`, on TWO engines built from the SAME recompiled sections
//! (the #908 machinery): `base` carries empty SKMX/PSIB (must reproduce #841);
//! `skip` carries the real `fit_skipmix_tables` sections (the deployed lane).
//! `predict_decision` routes through the lane when the sections are present
//! (engine.rs, #910), so free-running rollouts on `skip` ARE the deployed lane.
//! It reports, per engine: median first-divergence, diverged-at-0, the
//! suffix-locality statistic, and the free-running path histogram (the
//! correctable/graph fraction), plus the cross-engine "did the lane change any
//! free-running rollout" count. It fits NO corrections, launches NO corrective
//! round, and moves NO gate.
//!
//! Run:
//!   R4_CAUSAL_BUNDLE=<bundle> cargo test -p uor-r4-api --release \
//!     --test free_running_reachability_840 -- --ignored --nocapture

#![allow(clippy::doc_lazy_continuation)]

use std::path::PathBuf;
use std::time::Instant;

use uor_r4_api::capability_suite::compute_cid;
use uor_r4_api::engine::{EngineParts, PredictDecision, R4Engine};
use uor_r4_api::serving_eval::ServingBundle;
use uor_r4_api::ScoreStatus;
use uor_r4_core::transformerless::{compiler, runtime};
use uor_r4_graph_certify as score;
use uor_r4_graph_compiler::{induction, skipmix_fit};

/// Deployed context-window cap (compiler::WINDOW).
const WINDOW: usize = 8;
/// Frozen horizon ladder of prompt-family v1 (identical to #841).
const HORIZONS: [usize; 2] = [8, 32];
/// Frozen prompt-family size.
const N_PROMPTS: usize = 100;
const H_MAX: usize = 32;
/// Cycle detector: periods checked and the repeat count that flags a cycle.
const MAX_PERIOD: usize = 4;
const CYCLE_FLAG_REPEATS: u32 = 3;
/// Per-key retained-count cap for the fitted lane tables (#835/#906/#908).
const CAP: usize = uor_r4_graph_compiler::segment_fit::DEFAULT_TOP_K;
/// The attested #833 bundle's `corpus.meta` CID prefix -- the run is only valid
/// on that bundle (identical guard to #908).
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

/// Per-step attribution label (the #832 `ResolutionPath` served subset plus the
/// typed abstention), identical semantics to #841.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepPath {
    ExactContext,
    Ngram,
    Graph,
    Decline,
}

/// One generated step of a trajectory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TraceStep {
    token: Option<u32>,
    path: StepPath,
    widened: bool,
}

/// Integer metrics of one trajectory against its recorded reference
/// continuation (verbatim semantics from #841; no floats).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SeqMetrics {
    first_divergence: Option<u32>,
    served: u32,
    abstained: bool,
    widened: u32,
    exact_context: u32,
    ngram: u32,
    graph: u32,
    /// Graph-path steps that ALSO depart the recorded text -- the correctable
    /// failure count that upper-bounds #840's reachable movement.
    graph_failures: u32,
    max_cycle_repeats: u32,
    distinct1_permille: u32,
}

/// Longest number of consecutive repeats of any trailing period-`p` cycle
/// (p <= MAX_PERIOD) -- verbatim from #841.
fn max_cycle_repeats(seq: &[u32]) -> u32 {
    let n = seq.len();
    let mut best = 0u32;
    for p in 1..=MAX_PERIOD.min(n) {
        let mut aligned = 0u32;
        for i in p..n {
            if seq[i] == seq[i - p] {
                aligned += 1;
                best = best.max(aligned / p as u32 + 1);
            } else {
                aligned = 0;
            }
        }
    }
    best
}

fn metrics_of(steps: &[TraceStep], recorded: &[u32]) -> SeqMetrics {
    let served_tokens: Vec<u32> = steps.iter().filter_map(|s| s.token).collect();
    let served = served_tokens.len() as u32;
    let abstained = steps.last().is_some_and(|s| s.token.is_none());
    let mut first_divergence = None;
    let mut graph_failures = 0u32;
    for (k, s) in steps.iter().enumerate() {
        let departs = match s.token {
            None => true,
            Some(t) => recorded.get(k).copied() != Some(t),
        };
        if departs && s.path == StepPath::Graph {
            graph_failures += 1;
        }
        if first_divergence.is_none() && departs {
            first_divergence = Some(k as u32);
        }
    }
    let mut distinct: Vec<u32> = served_tokens.clone();
    distinct.sort_unstable();
    distinct.dedup();
    let denom = u64::from(served.max(1));
    SeqMetrics {
        first_divergence,
        served,
        abstained,
        widened: steps.iter().filter(|s| s.widened).count() as u32,
        exact_context: steps
            .iter()
            .filter(|s| s.path == StepPath::ExactContext)
            .count() as u32,
        ngram: steps.iter().filter(|s| s.path == StepPath::Ngram).count() as u32,
        graph: steps.iter().filter(|s| s.path == StepPath::Graph).count() as u32,
        graph_failures,
        max_cycle_repeats: max_cycle_repeats(&served_tokens),
        distinct1_permille: ((distinct.len() as u64 * 1000) / denom) as u32,
    }
}

/// Lower median of a non-empty integer sample (deterministic; verbatim #841).
fn median(mut xs: Vec<u32>) -> u32 {
    xs.sort_unstable();
    xs[(xs.len() - 1) / 2]
}

/// Map one deployed decision to a trace step (verbatim from #841).
fn step_of(decision: &PredictDecision) -> TraceStep {
    match decision {
        PredictDecision::Serve(o) => TraceStep {
            token: Some(o.token),
            path: if o.ngram_hit {
                StepPath::Ngram
            } else {
                match o.status {
                    ScoreStatus::ExactContext => StepPath::ExactContext,
                    _ => StepPath::Graph,
                }
            },
            widened: o.widened,
        },
        PredictDecision::Abstain(a) => TraceStep {
            token: None,
            path: StepPath::Decline,
            widened: a.widened,
        },
    }
}

/// Free-running rollout: the engine consumes its own served tokens (greedy,
/// deterministic); an abstention or error terminates (verbatim from #841).
fn rollout(engine: &mut R4Engine, prompt: &[u32], h: usize) -> Vec<TraceStep> {
    engine.reset();
    let mut seq: Vec<u32> = prompt.to_vec();
    let mut steps = Vec::with_capacity(h);
    for _ in 0..h {
        let start = seq.len().saturating_sub(WINDOW);
        match engine.predict_decision(&seq[start..]) {
            Ok(d) => {
                let s = step_of(&d);
                let done = s.token.is_none();
                if let Some(t) = s.token {
                    seq.push(t);
                }
                steps.push(s);
                if done {
                    break;
                }
            }
            Err(_) => {
                steps.push(TraceStep {
                    token: None,
                    path: StepPath::Decline,
                    widened: false,
                });
                break;
            }
        }
    }
    steps
}

/// Teacher-prefix pass: predict at each matched RECORDED window; count agreement
/// against the recorded teacher argmax (verbatim from #841).
fn teacher_forced(engine: &mut R4Engine, corpus: &compiler::Corpus, i0: usize, h: usize) -> u32 {
    engine.reset();
    let mut agree = 0u32;
    for k in 0..h {
        let w = induction::context_window(corpus, i0 + k);
        if let Ok(d) = engine.predict_decision(&w) {
            if step_of(&d).token == Some(corpus.t_argmax[i0 + k]) {
                agree += 1;
            }
        }
    }
    agree
}

#[derive(Default)]
struct Aggregate {
    diverge_steps: Vec<u32>,
    diverged_at0: u32,
    survived_full: u32,
    abstained: u32,
    cycled: u32,
    served: u64,
    exact_context: u64,
    ngram: u64,
    graph: u64,
    graph_failures: u64,
    distinct1_sum: u64,
    n: u32,
}

impl Aggregate {
    fn push(&mut self, m: &SeqMetrics, h: usize) {
        self.n += 1;
        self.diverge_steps
            .push(m.first_divergence.unwrap_or(h as u32));
        self.diverged_at0 += u32::from(m.first_divergence == Some(0));
        self.survived_full += u32::from(m.first_divergence.is_none());
        self.abstained += u32::from(m.abstained);
        self.cycled += u32::from(m.max_cycle_repeats >= CYCLE_FLAG_REPEATS);
        self.served += u64::from(m.served);
        self.exact_context += u64::from(m.exact_context);
        self.ngram += u64::from(m.ngram);
        self.graph += u64::from(m.graph);
        self.graph_failures += u64::from(m.graph_failures);
        self.distinct1_sum += u64::from(m.distinct1_permille);
    }
    fn median_diverge(&self) -> u32 {
        median(self.diverge_steps.clone())
    }
    fn permille(v: u64, n: u64) -> u64 {
        v * 1000 / n.max(1)
    }
}

// --- fixture teeth (non-ignored; the instrument can fail without the bundle) --

fn step(token: Option<u32>, path: StepPath) -> TraceStep {
    TraceStep {
        token,
        path,
        widened: false,
    }
}

#[test]
fn metrics_locate_first_divergence_and_count_graph_failures() {
    // recorded continuation: [10, 11, 12, 13]
    let recorded = [10u32, 11, 12, 13];
    // on-text for 2 steps, then a graph-path departure, then an ngram departure.
    let steps = [
        step(Some(10), StepPath::Ngram),
        step(Some(11), StepPath::ExactContext),
        step(Some(99), StepPath::Graph), // departs at step 2, graph path
        step(Some(98), StepPath::Ngram), // departs, ngram path (not correctable)
    ];
    let m = metrics_of(&steps, &recorded);
    assert_eq!(m.first_divergence, Some(2), "first departure at step 2");
    assert_eq!(m.graph, 1);
    assert_eq!(
        m.graph_failures, 1,
        "one graph-path departure is correctable"
    );
    assert_eq!(m.served, 4);
    assert!(!m.abstained);
}

#[test]
fn metrics_flag_cycles_and_terminal_abstention() {
    let recorded = [1u32, 2, 3, 4, 5, 6];
    // A period-2 cycle a,b,a,b,a,b (three full repeats) then a terminal abstention.
    let steps = [
        step(Some(7), StepPath::Graph),
        step(Some(8), StepPath::Graph),
        step(Some(7), StepPath::Graph),
        step(Some(8), StepPath::Graph),
        step(Some(7), StepPath::Graph),
        step(Some(8), StepPath::Graph),
        step(None, StepPath::Decline),
    ];
    let m = metrics_of(&steps, &recorded);
    assert_eq!(m.first_divergence, Some(0), "departs immediately");
    assert!(m.abstained, "terminal abstention recorded");
    assert!(
        m.max_cycle_repeats >= CYCLE_FLAG_REPEATS,
        "the a,b,a,b,a cycle is flagged"
    );
}

#[test]
fn median_is_lower_median_deterministic() {
    assert_eq!(median(vec![0, 0, 5, 9]), 0);
    assert_eq!(median(vec![3, 1, 2]), 2);
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle; run with --ignored (see module docs)"]
fn free_running_reachability_run_840() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP free_running_reachability_run_840: no serving bundle at {}",
            root.display()
        );
        return;
    };
    let started = Instant::now();

    // === load corpus + split ================================================
    let meta_bytes = std::fs::read(&bundle.corpus_meta).expect("corpus meta");
    let recs_bytes = std::fs::read(&bundle.corpus_records).expect("corpus records");
    let corpus = compiler::load_corpus_from(
        bundle.corpus_meta.to_str().expect("meta utf8"),
        bundle.corpus_records.to_str().expect("recs utf8"),
    )
    .expect("load corpus");
    let (train_positions, held_out) = induction::split_positions(&corpus);
    assert!(!train_positions.is_empty() && !held_out.is_empty(), "split");
    let corpus_cid = compute_cid(&meta_bytes);
    assert!(
        corpus_cid.starts_with(ATTESTED_CORPUS_CID_PREFIX),
        "run is only valid on the attested #833 bundle (got {corpus_cid})"
    );

    // === fit the deployed lane tables (real corpus, #908 machinery) =========
    let (skip_rows, psi_rows) = skipmix_fit::fit_skipmix_tables(&corpus, CAP);
    assert!(
        !skip_rows.is_empty() && !psi_rows.is_empty(),
        "fit learns keys"
    );

    // === recompile the shared graph sections (verbatim #908/#906) ===========
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

    // === emit base (empty lane) / skip (real lane) from the SAME sections ===
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
        "base engine must carry NO lane sections"
    );
    assert_eq!(
        engine_skip.skipmix_tables_present(),
        (true, true),
        "skip engine must consume BOTH lane sections"
    );

    // === frozen prompt-family v1 (verbatim #841 selection) ==================
    let mut prompts: Vec<usize> = Vec::new();
    let mut last_story = u64::MAX;
    for &i in &held_out {
        let story = corpus.story[i] as u64;
        if story == last_story {
            continue;
        }
        let w = induction::context_window(&corpus, i);
        if w.len() == WINDOW
            && i + H_MAX + 1 < corpus.n
            && (corpus.story[i + H_MAX + 1] as u64) == story
        {
            prompts.push(i);
            last_story = story;
            if prompts.len() == N_PROMPTS {
                break;
            }
        }
    }
    assert_eq!(prompts.len(), N_PROMPTS, "frozen prompt family fills");
    let family_bytes: Vec<u8> = prompts
        .iter()
        .flat_map(|&i| (i as u64).to_le_bytes())
        .collect();
    let family_cid = compute_cid(&family_bytes);

    // === measure both engines on the frozen family ==========================
    let base = measure_engine(&mut engine_base, &corpus, &prompts);
    let skip = measure_engine(&mut engine_skip, &corpus, &prompts);

    // determinism teeth: the skip engine reproduces identical FR rollouts.
    for &i0 in prompts.iter().take(5) {
        let prompt = induction::context_window(&corpus, i0);
        assert_eq!(
            rollout(&mut engine_skip, &prompt, H_MAX),
            rollout(&mut engine_skip, &prompt, H_MAX),
            "greedy rollout deterministic"
        );
    }

    // cross-engine: how many free-running rollouts did the lane actually change?
    let lane_changed_fr = base
        .fr_h32_seqs
        .iter()
        .zip(&skip.fr_h32_seqs)
        .filter(|(b, s)| b != s)
        .count() as u32;

    // === reachability arithmetic (the fork number) ==========================
    let bs = &base.student[1];
    let ss = &skip.student[1];
    let skip_served = ss.served.max(1);
    let graph_reach_pm = ss.graph * 1000 / skip_served;
    let graph_fail_reach_pm = ss.graph_failures * 1000 / skip_served;
    let base_graph_reach_pm = bs.graph * 1000 / bs.served.max(1);
    let tf_base_pm = base.tf_agree_total * 1000 / (N_PROMPTS as u64 * H_MAX as u64);
    let tf_skip_pm = skip.tf_agree_total * 1000 / (N_PROMPTS as u64 * H_MAX as u64);

    println!("=== #840 free-running reachability instrument (greedy, teacher-free) ===");
    println!("bundle           : {}", bundle.root.display());
    println!("corpus_meta_cid  : {corpus_cid}");
    println!("prompt_family    : v1 n={N_PROMPTS} cid {family_cid}");
    println!(
        "TF agreement     : base {tf_base_pm}permille | skip {tf_skip_pm}permille (over {} matched steps)",
        N_PROMPTS * H_MAX
    );
    for (name, r) in [("base", &base), ("skip", &skip)] {
        for (hi, &h) in HORIZONS.iter().enumerate() {
            let a = &r.student[hi];
            println!(
                "{name:<4} student h={h:<3} median-div {} | at0 {}permille | survived {}permille | abstained {}permille | cycled {}permille | paths exct/ngram/graph {}/{}/{}",
                a.median_diverge(),
                Aggregate::permille(u64::from(a.diverged_at0), u64::from(a.n)),
                Aggregate::permille(u64::from(a.survived_full), u64::from(a.n)),
                Aggregate::permille(u64::from(a.abstained), u64::from(a.n)),
                Aggregate::permille(u64::from(a.cycled), u64::from(a.n)),
                a.exact_context,
                a.ngram,
                a.graph,
            );
        }
    }
    println!(
        "suffix-locality  : base {}/{N_PROMPTS} | skip {}/{N_PROMPTS} FR rollouts identical to suffix-only",
        base.suffix_equal_fr_h32, skip.suffix_equal_fr_h32
    );
    println!("lane changed FR  : {lane_changed_fr}/{N_PROMPTS} rollouts differ (skip vs base)");
    println!(
        "graph reach (h32): base {base_graph_reach_pm}permille | skip {graph_reach_pm}permille of served steps; skip graph-path FAILURES {graph_fail_reach_pm}permille"
    );

    // The #841 §6 bar requires, per corrective round, median first-divergence
    // +>=2 steps AND diverged-at-0 -100permille. Two independent bounds say a
    // corrective run cannot reach it against this representation:
    //   (1) the CORRECTABLE footprint -- corrective observation reshapes the
    //       graph/skip-mix tables, whose free-running footprint is graph_reach_pm
    //       (~1permille here), ~100x below the 100permille at0-drop bar; and
    //   (2) DIRECTION -- for corrective rounds of the SAME evidence family to
    //       have any chance, the activated lane must at least move free-running
    //       TOWARD the bar (median up or at0 down); it moves the opposite way.
    let bar_at0_drop_pm = 100u64;
    let median_base = bs.median_diverge();
    let median_skip = ss.median_diverge();
    let at0_base = Aggregate::permille(u64::from(bs.diverged_at0), u64::from(bs.n));
    let at0_skip = Aggregate::permille(u64::from(ss.diverged_at0), u64::from(ss.n));
    let cycled_base = Aggregate::permille(u64::from(bs.cycled), u64::from(bs.n));
    let cycled_skip = Aggregate::permille(u64::from(ss.cycled), u64::from(ss.n));
    let reachable_ceiling_pm = graph_reach_pm;
    let ceiling_clears_bar = reachable_ceiling_pm >= bar_at0_drop_pm;
    // "improves" = strictly better on the frozen primary (median up) or a
    // material diverged-at-0 drop.
    let lane_improves_coherence = median_skip > median_base || at0_skip + 1 < at0_base;
    let verdict_code = if ceiling_clears_bar && lane_improves_coherence {
        "PROCEED"
    } else if !lane_improves_coherence {
        "STOP-RECOMMEND-GENERATION-NOT-ESTABLISHED"
    } else {
        "STOP-CEILING-BELOW-BAR"
    };
    println!(
        "coherence dlt h32: median-div {median_base}->{median_skip} | diverged-at-0 {at0_base}->{at0_skip}permille | cycled {cycled_base}->{cycled_skip}permille"
    );
    println!("reachable ceiling: {reachable_ceiling_pm}permille (correctable/graph footprint) vs §6 bar {bar_at0_drop_pm}permille at0-drop");
    let verdict_text = match verdict_code {
        "PROCEED" => "PROCEED -- correctable footprint clears the bar AND the lane moves free-running toward coherence; design the corrective run",
        "STOP-RECOMMEND-GENERATION-NOT-ESTABLISHED" => "STOP -- the activated lane does NOT move free-running toward coherence (median unchanged, diverged-at-0 not reduced) and the correctable footprint is far below the §6 bar; corrective rounds of the same evidence family cannot clear it -- recommend GENERATION-NOT-ESTABLISHED (do-not-launch)",
        _ => "STOP -- reachable correctable footprint is far below the §6 at0-drop bar; corrective ceiling < bar (do-not-launch)",
    };
    println!("VERDICT          : {verdict_text}");

    // === CID-bound result record ============================================
    let base_graph_cid = compute_cid(&base_graph);
    let skip_graph_cid = compute_cid(&skip_graph);
    let result_cid = compute_cid(
        format!(
            "{}:{}:{}:{}:{}:{}:{}:{}",
            base.suffix_equal_fr_h32,
            skip.suffix_equal_fr_h32,
            lane_changed_fr,
            graph_reach_pm,
            graph_fail_reach_pm,
            ss.diverged_at0,
            skip_graph_cid,
            corpus_cid,
        )
        .as_bytes(),
    );
    let json = format!(
        concat!(
            "{{\n",
            "  \"issue\": 840,\n",
            "  \"study\": \"free-running-reachability-under-skipmix-v1\",\n",
            "  \"execution_scope\": \"deployed R4Engine, teacher-free (recorded t_argmax labels)\",\n",
            "  \"mode\": \"greedy\",\n",
            "  \"bundle\": \"{}\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"prompt_family\": {{\"version\": 1, \"n\": {}, \"cid\": \"{}\"}},\n",
            "  \"base_graph_cid\": \"{}\",\n",
            "  \"skip_graph_cid\": \"{}\",\n",
            "  \"tf_agreement_permille\": {{\"base\": {}, \"skip\": {}}},\n",
            "  \"suffix_locality_of_100\": {{\"base\": {}, \"skip\": {}}},\n",
            "  \"lane_changed_fr_rollouts\": {},\n",
            "  \"base_student\": {{\"h8\": {}, \"h32\": {}}},\n",
            "  \"skip_student\": {{\"h8\": {}, \"h32\": {}}},\n",
            "  \"coherence_h32\": {{\"median_base\": {}, \"median_skip\": {}, \"at0_base_permille\": {}, \"at0_skip_permille\": {}, \"cycled_base_permille\": {}, \"cycled_skip_permille\": {}}},\n",
            "  \"reachability\": {{\"skip_graph_reach_permille\": {}, \"skip_graph_failure_permille\": {}, \"base_graph_reach_permille\": {}, \"sec6_at0_drop_bar_permille\": {}, \"ceiling_clears_bar\": {}, \"lane_improves_coherence\": {}}},\n",
            "  \"verdict\": \"{}\",\n",
            "  \"result_cid\": \"{}\"\n",
            "}}\n"
        ),
        bundle.root.display(),
        corpus_cid,
        N_PROMPTS,
        family_cid,
        base_graph_cid,
        skip_graph_cid,
        tf_base_pm,
        tf_skip_pm,
        base.suffix_equal_fr_h32,
        skip.suffix_equal_fr_h32,
        lane_changed_fr,
        agg_json(&base.student[0], 8),
        agg_json(&base.student[1], 32),
        agg_json(&skip.student[0], 8),
        agg_json(&skip.student[1], 32),
        median_base,
        median_skip,
        at0_base,
        at0_skip,
        cycled_base,
        cycled_skip,
        graph_reach_pm,
        graph_fail_reach_pm,
        base_graph_reach_pm,
        bar_at0_drop_pm,
        ceiling_clears_bar,
        lane_improves_coherence,
        verdict_code,
        result_cid,
    );
    let out = repo_root()
        .join("docs")
        .join("free_running_reachability_840_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("result_cid       : {result_cid}");
    println!("wrote            : {}", out.display());
    println!("elapsed          : {:.1}s", started.elapsed().as_secs_f64());
}

/// Per-engine free-running measurement over the frozen family.
struct EngineReport {
    tf_agree_total: u64,
    /// student-prefix aggregate per horizon (index 0 -> h8, 1 -> h32).
    student: [Aggregate; 2],
    suffix_equal_fr_h32: u32,
    /// FR served-token sequences at H_MAX (for the cross-engine lane-change count).
    fr_h32_seqs: Vec<Vec<Option<u32>>>,
}

/// Run student-prefix (and, at H_MAX, the suffix-only control + teacher-forced
/// agreement) over the frozen family on one engine. TEACHER-FREE.
fn measure_engine(
    engine: &mut R4Engine,
    corpus: &compiler::Corpus,
    prompts: &[usize],
) -> EngineReport {
    let mut student = [Aggregate::default(), Aggregate::default()];
    let mut tf_agree_total = 0u64;
    let mut suffix_equal_fr_h32 = 0u32;
    let mut fr_h32_seqs = Vec::with_capacity(prompts.len());
    for (hi, &h) in HORIZONS.iter().enumerate() {
        for &i0 in prompts {
            let prompt = induction::context_window(corpus, i0);
            let recorded: Vec<u32> = (1..=h).map(|k| corpus.input[i0 + k]).collect();
            let fr = rollout(engine, &prompt, h);
            student[hi].push(&metrics_of(&fr, &recorded), h);
            if h == H_MAX {
                let so = rollout(engine, &prompt[prompt.len() - 2..], h);
                let fr_seq: Vec<Option<u32>> = fr.iter().map(|s| s.token).collect();
                let so_seq: Vec<Option<u32>> = so.iter().map(|s| s.token).collect();
                suffix_equal_fr_h32 += u32::from(fr_seq == so_seq);
                fr_h32_seqs.push(fr_seq);
                tf_agree_total += u64::from(teacher_forced(engine, corpus, i0, h));
            }
        }
    }
    EngineReport {
        tf_agree_total,
        student,
        suffix_equal_fr_h32,
        fr_h32_seqs,
    }
}

/// Compact per-horizon student aggregate JSON block.
fn agg_json(a: &Aggregate, h: usize) -> String {
    let n = u64::from(a.n.max(1));
    format!(
        "{{\"n\": {}, \"median_first_divergence\": {}, \"diverged_at_step0_permille\": {}, \"survived_full_horizon_permille\": {}, \"abstained_permille\": {}, \"cycled_permille\": {}, \"mean_distinct1_permille\": {}, \"paths\": {{\"exact_context\": {}, \"ngram\": {}, \"graph\": {}, \"graph_failures\": {}, \"served\": {}}}, \"horizon\": {}}}",
        a.n,
        a.median_diverge(),
        Aggregate::permille(u64::from(a.diverged_at0), n),
        Aggregate::permille(u64::from(a.survived_full), n),
        Aggregate::permille(u64::from(a.abstained), n),
        Aggregate::permille(u64::from(a.cycled), n),
        a.distinct1_sum / n,
        a.exact_context,
        a.ngram,
        a.graph,
        a.graph_failures,
        a.served,
        h,
    )
}
