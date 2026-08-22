//! #842 -- trajectory-state gate + S3 free-running generation verdict (item C
//! of S3 tracker #824, programme #820). BINDING CHEAP INSTRUMENT for the #842
//! run contract (AGENTS.md long-run discipline): it decides, from the frozen
//! sibling-A (#841) and sibling-B (#840) evidence, whether a bounded
//! trajectory-state mechanism is TRIGGERED before any runtime code is written.
//!
//! ## The diagnostic question
//!
//! #842 must distinguish STATE STARVATION (a failure a bounded topic/entity/
//! anti-cycle memory could fix) from EVIDENCE / CANDIDATE / DECODER failures
//! (which added state cannot fix). It classifies every free-running failure on
//! the frozen prompt-family v1 by pairing the deployed student-prefix rollout
//! against the teacher-prefix rollout and probing the deployed candidate list.
//!
//! The five causes: `Survived`: the student
//! stayed on-text the full horizon. `SingleStepAt0`: it diverged at step 0,
//! where there is no prior trajectory (student and teacher share the prompt
//! window), so trajectory state is definitionally irrelevant. `CandidateGap`: it
//! diverged at step d>=1 but the recorded token was not among the deployed
//! candidate list, so re-ranking cannot reach it. `RankLimit`: it diverged at
//! d>=1 with the recorded token a candidate, but the teacher-prefix diverges at
//! the same-or-earlier step -- a context-independent rank/decoder limit the
//! perfect context does not fix either. `StateStarvation`: it diverged at d>=1
//! with the recorded token a candidate AND the teacher-prefix survives strictly
//! longer -- the ONLY failure a bounded state mechanism could address (the
//! student's own drifted context caused the early loss).
//!
//! ## The reachability argument (why this is teacher-free and cheap)
//!
//! The frozen #841 §6 bar requires, to count as an improvement: median
//! first-divergence +>=2 steps AND diverged-at-step-0 -100permille. Two
//! representation-independent bounds cap what ANY trajectory-state mechanism can
//! do against them:
//!   1. STEP-0 INVARIANCE. `diverged-at-step-0` counts step-0 failures; at step
//!      0 the student and teacher share the exact prompt window, so no state
//!      mechanism changes step 0. The reachable at0-drop is 0permille. And if
//!      the step-0 fraction exceeds 500permille the median first-divergence is
//!      pinned at 0 regardless of any post-step-0 improvement.
//!   2. TEACHER-PREFIX UPPER BOUND. The drift-free reference state -- perfect
//!      context reconstruction -- is exactly the teacher-prefix trajectory. Its
//!      median and at0 are the ceiling of any state mechanism on the student
//!      side; if teacher-prefix itself does not clear the §6 bar, nothing does.
//!
//! Sibling B (#840) already showed the only representation change that adds
//! cross-step signal (the RF-31 skip-mix lane) REGRESSES free-running
//! (exposure-bias amplification) with a ~1permille correctable footprint. This
//! instrument confirms the bounds directly and issues the S3 verdict.
//!
//! ## What it does NOT do
//!
//! It fits nothing, launches no corrective round, adds no runtime/format/
//! compiler code, and moves no gate on its own -- identical treatment to #840
//! and #841 (RF-14 / RF-21 / RF-22 / RF-29 evidence language; no new built
//! capability, no `model/ids.toml` row, no `CONFORMANCE.md` regeneration).
//!
//! Run:
//!   R4_CAUSAL_BUNDLE=<bundle> cargo test -p uor-r4-api --release \
//!     --test state_trajectory_gate_842 -- --ignored --nocapture

#![allow(clippy::doc_lazy_continuation)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_api::capability_suite::compute_cid;
use uor_r4_api::engine::{EngineParts, PredictDecision, R4Engine};
use uor_r4_api::serving_eval::ServingBundle;
use uor_r4_api::ScoreStatus;
use uor_r4_core::transformerless::compiler;
use uor_r4_graph_certify::StepCandidates;
use uor_r4_graph_compiler::induction;

/// Deployed context-window cap (compiler::WINDOW).
const WINDOW: usize = 8;
/// Frozen horizon ladder of prompt-family v1 (identical to #841/#840).
const HORIZONS: [usize; 2] = [8, 32];
/// Frozen prompt-family size.
const N_PROMPTS: usize = 100;
const H_MAX: usize = 32;
/// Cycle detector: periods checked and the repeat count that flags a cycle.
const MAX_PERIOD: usize = 4;
const CYCLE_FLAG_REPEATS: u32 = 3;
/// The attested #833 bundle's `corpus.meta` CID prefix -- the run is only valid
/// on that bundle (identical guard to #841/#840/#908).
const ATTESTED_CORPUS_CID_PREFIX: &str = "blake3:aa9d1767";
/// Frozen #841 §6 corrective bar: a mechanism clears it only if the median
/// first-divergence rises by >= this many steps AND diverged-at-step-0 falls by
/// >= `SEC6_AT0_DROP_PERMILLE` (no TF regression > 10permille). #842 asks
/// whether ANY bounded trajectory-state mechanism can clear it.
const SEC6_MEDIAN_RISE: u32 = 2;
const SEC6_AT0_DROP_PERMILLE: u64 = 100;

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
/// typed abstention), identical semantics to #841/#840.
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
/// continuation (verbatim semantics from #841/#840; no floats).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SeqMetrics {
    first_divergence: Option<u32>,
    served: u32,
    abstained: bool,
    exact_context: u32,
    ngram: u32,
    graph: u32,
    max_cycle_repeats: u32,
    distinct1_permille: u32,
}

/// Longest number of consecutive repeats of any trailing period-`p` cycle
/// (p <= MAX_PERIOD) -- verbatim from #841/#840.
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
    for (k, s) in steps.iter().enumerate() {
        let departs = match s.token {
            None => true,
            Some(t) => recorded.get(k).copied() != Some(t),
        };
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
        exact_context: steps
            .iter()
            .filter(|s| s.path == StepPath::ExactContext)
            .count() as u32,
        ngram: steps.iter().filter(|s| s.path == StepPath::Ngram).count() as u32,
        graph: steps.iter().filter(|s| s.path == StepPath::Graph).count() as u32,
        max_cycle_repeats: max_cycle_repeats(&served_tokens),
        distinct1_permille: ((distinct.len() as u64 * 1000) / denom) as u32,
    }
}

/// Lower median of a non-empty integer sample (deterministic; verbatim #841).
fn median(mut xs: Vec<u32>) -> u32 {
    xs.sort_unstable();
    xs[(xs.len() - 1) / 2]
}

/// Map one deployed decision to a trace step (verbatim from #841/#840).
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

/// The diagnosed cause of a free-running trajectory's failure. `StateStarvation`
/// is the ONLY cause a bounded trajectory-state mechanism could address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Cause {
    Survived,
    SingleStepAt0,
    CandidateGap,
    RankLimit,
    StateStarvation,
}

/// The predeclared, non-vacuous state-starvation classifier (pure). Inputs are
/// the student and teacher first-divergence steps and whether the recorded
/// reference token was among the deployed candidate list at the student's
/// divergence step. The confusion fixture proves it distinguishes every cause.
///
/// - `d_student == None` -> the student survived the full horizon on-text.
/// - `d == 0` -> step-0 failure: no trajectory exists yet, state-irrelevant.
/// - recorded token not a candidate -> candidate/evidence gap; re-ranking cannot
///   reach a token the deployed scorer never proposed.
/// - teacher survives strictly longer (or the full horizon) with the token
///   rankable -> STATE STARVATION: the student's own drifted context caused the
///   early loss, and the perfect context would have avoided it.
/// - otherwise (teacher diverges at the same or an earlier step) -> a
///   context-independent rank/decoder limit that the perfect context does not
///   fix either.
fn classify(d_student: Option<u32>, d_teacher: Option<u32>, recorded_in_candidates: bool) -> Cause {
    let Some(d) = d_student else {
        return Cause::Survived;
    };
    if d == 0 {
        return Cause::SingleStepAt0;
    }
    if !recorded_in_candidates {
        return Cause::CandidateGap;
    }
    match d_teacher {
        None => Cause::StateStarvation,
        Some(dt) if dt > d => Cause::StateStarvation,
        _ => Cause::RankLimit,
    }
}

/// The reachable diverged-at-step-0 drop (permille) any trajectory-state
/// mechanism can achieve, upper-bounded by the drift-free reference
/// (teacher-prefix). At step 0 the student and teacher share the prompt window,
/// so a well-formed instrument reports `student_at0 == teacher_at0` and this is
/// 0 -- the STEP-0 INVARIANCE bound. Saturating (never underflows below 0).
fn at0_reachable_drop_pm(student_at0_pm: u64, teacher_at0_pm: u64) -> u64 {
    student_at0_pm.saturating_sub(teacher_at0_pm)
}

/// Whether the reachable ceiling (the drift-free teacher-prefix reference)
/// clears the frozen #841 §6 bar: median first-divergence up by
/// `SEC6_MEDIAN_RISE` AND diverged-at-step-0 down by `SEC6_AT0_DROP_PERMILLE`.
/// BOTH prongs must hold. Pure; the fixture proves it can report either verdict
/// (it is not vacuously false).
fn ceiling_clears_bar(
    student_median: u32,
    teacher_median: u32,
    student_at0_pm: u64,
    teacher_at0_pm: u64,
) -> bool {
    let median_ok = teacher_median >= student_median.saturating_add(SEC6_MEDIAN_RISE);
    let at0_ok = at0_reachable_drop_pm(student_at0_pm, teacher_at0_pm) >= SEC6_AT0_DROP_PERMILLE;
    median_ok && at0_ok
}

/// Free-running student-prefix rollout (greedy, deterministic) that ALSO records,
/// per step k, whether the recorded reference token at k was among the deployed
/// candidate list at k. Uses the deployed decision path
/// (`predict_decision_candidates_with_skipmix`) so the served tokens are
/// byte-identical to #841's `predict_decision`; on the SKMX-absent broad-clean
/// bundle the candidate list is the base ranked list. An abstention or engine
/// error terminates.
fn rollout_diag(
    engine: &mut R4Engine,
    prompt: &[u32],
    recorded: &[u32],
    h: usize,
) -> (Vec<TraceStep>, Vec<bool>) {
    engine.reset();
    let mut seq: Vec<u32> = prompt.to_vec();
    let mut steps = Vec::with_capacity(h);
    let mut rec_in_cand = Vec::with_capacity(h);
    let mut cands = StepCandidates::default();
    for k in 0..h {
        let start = seq.len().saturating_sub(WINDOW);
        match engine.predict_decision_candidates_with_skipmix(&seq[start..], &mut cands) {
            Ok(d) => {
                let s = step_of(&d);
                let in_cand = recorded
                    .get(k)
                    .is_some_and(|rt| cands.ranked().iter().any(|(t, _)| t == rt));
                rec_in_cand.push(in_cand);
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
                rec_in_cand.push(false);
                break;
            }
        }
    }
    (steps, rec_in_cand)
}

/// Teacher-prefix rollout: predict at each matched RECORDED window (the perfect,
/// drift-free context). Returns the trace steps and the matched teacher-forced
/// agreement count against the recorded teacher argmax. This side is BOTH the
/// #841 teacher-forced control AND the reachability upper bound: it is the best
/// any bounded trajectory-state mechanism could reconstruct.
fn teacher_prefix(
    engine: &mut R4Engine,
    corpus: &compiler::Corpus,
    i0: usize,
    h: usize,
) -> (Vec<TraceStep>, u32) {
    engine.reset();
    let mut steps = Vec::with_capacity(h);
    let mut agree = 0u32;
    for k in 0..h {
        let w = induction::context_window(corpus, i0 + k);
        match engine.predict_decision(&w) {
            Ok(d) => {
                let s = step_of(&d);
                if s.token == Some(corpus.t_argmax[i0 + k]) {
                    agree += 1;
                }
                steps.push(s);
            }
            Err(_) => steps.push(TraceStep {
                token: None,
                path: StepPath::Decline,
                widened: false,
            }),
        }
    }
    (steps, agree)
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
        self.distinct1_sum += u64::from(m.distinct1_permille);
    }
    fn median_diverge(&self) -> u32 {
        median(self.diverge_steps.clone())
    }
    fn permille(v: u64, n: u64) -> u64 {
        v * 1000 / n.max(1)
    }
    fn at0_permille(&self) -> u64 {
        Self::permille(u64::from(self.diverged_at0), u64::from(self.n))
    }
}

/// Compact per-horizon aggregate JSON block.
fn agg_json(a: &Aggregate, h: usize) -> String {
    let n = u64::from(a.n.max(1));
    format!(
        "{{\"n\": {}, \"median_first_divergence\": {}, \"diverged_at_step0_permille\": {}, \"survived_full_horizon_permille\": {}, \"abstained_permille\": {}, \"cycled_permille\": {}, \"mean_distinct1_permille\": {}, \"paths\": {{\"exact_context\": {}, \"ngram\": {}, \"graph\": {}, \"served\": {}}}, \"horizon\": {}}}",
        a.n,
        a.median_diverge(),
        a.at0_permille(),
        Aggregate::permille(u64::from(a.survived_full), n),
        Aggregate::permille(u64::from(a.abstained), n),
        Aggregate::permille(u64::from(a.cycled), n),
        a.distinct1_sum / n,
        a.exact_context,
        a.ngram,
        a.graph,
        a.served,
        h,
    )
}

// ---- fixture teeth (non-ignored; the instrument can fail without the bundle) --

fn step(token: Option<u32>, path: StepPath) -> TraceStep {
    TraceStep {
        token,
        path,
        widened: false,
    }
}

#[test]
fn classifier_distinguishes_every_cause() {
    // Survived: student never diverged.
    assert_eq!(classify(None, Some(0), false), Cause::Survived);
    // Step-0 failure: no trajectory yet -> state-irrelevant (teacher also 0).
    assert_eq!(classify(Some(0), Some(0), true), Cause::SingleStepAt0);
    // Candidate gap: diverged late but the recorded token was never a candidate.
    assert_eq!(classify(Some(3), Some(10), false), Cause::CandidateGap);
    // State starvation: token rankable AND the perfect context survives longer.
    assert_eq!(classify(Some(3), Some(10), true), Cause::StateStarvation);
    // State starvation: teacher survived the whole horizon; student drifted out.
    assert_eq!(classify(Some(3), None, true), Cause::StateStarvation);
    // Rank limit: token rankable but the perfect context fails at the same step.
    assert_eq!(classify(Some(4), Some(4), true), Cause::RankLimit);
    // Rank limit: teacher fails even earlier -> context-independent.
    assert_eq!(classify(Some(4), Some(2), true), Cause::RankLimit);
}

#[test]
fn reachability_bar_can_report_both_verdicts() {
    // The observed shape (median 0->0, at0 590->590) must NOT clear the bar:
    // step-0 invariance pins at0 and the median.
    assert!(
        !ceiling_clears_bar(0, 0, 590, 590),
        "step-0-invariant shape cannot clear the §6 bar"
    );
    // A planted clearing shape MUST report TRIGGERED, proving the instrument is
    // not vacuously negative: median 0->3 (>= +2) and at0 590->400 (>= -100).
    assert!(
        ceiling_clears_bar(0, 3, 590, 400),
        "a genuinely clearing ceiling must report TRIGGERED"
    );
    // Only one prong met is still NOT cleared (both are required).
    assert!(
        !ceiling_clears_bar(0, 3, 590, 560),
        "median only -> not cleared"
    );
    assert!(
        !ceiling_clears_bar(0, 0, 590, 400),
        "at0 only -> not cleared"
    );
}

#[test]
fn at0_reachable_drop_is_zero_under_step0_invariance() {
    // A well-formed instrument reports student_at0 == teacher_at0 (step 0 shares
    // the prompt window), so the reachable at0 drop is exactly 0.
    assert_eq!(at0_reachable_drop_pm(590, 590), 0);
    // Saturating: a teacher that (impossibly) diverged MORE never yields a
    // negative "drop".
    assert_eq!(at0_reachable_drop_pm(590, 620), 0);
    // A genuine drop is reported when it exists.
    assert_eq!(at0_reachable_drop_pm(590, 400), 190);
}

#[test]
fn metrics_locate_first_divergence_and_paths() {
    let recorded = [10u32, 11, 12, 13];
    let steps = [
        step(Some(10), StepPath::Ngram),
        step(Some(11), StepPath::ExactContext),
        step(Some(99), StepPath::Graph), // departs at step 2
        step(Some(98), StepPath::Ngram),
    ];
    let m = metrics_of(&steps, &recorded);
    assert_eq!(m.first_divergence, Some(2));
    assert_eq!(m.graph, 1);
    assert_eq!(m.exact_context, 1);
    assert_eq!(m.ngram, 2);
    assert_eq!(m.served, 4);
    assert!(!m.abstained);
}

#[test]
fn metrics_flag_cycles_and_terminal_abstention() {
    let recorded = [1u32, 2, 3, 4, 5, 6];
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
    assert_eq!(m.first_divergence, Some(0));
    assert!(m.abstained);
    assert!(m.max_cycle_repeats >= CYCLE_FLAG_REPEATS);
}

#[test]
fn median_is_lower_median_deterministic() {
    assert_eq!(median(vec![0, 0, 5, 9]), 0);
    assert_eq!(median(vec![3, 1, 2]), 2);
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle; run with --ignored (see module docs)"]
fn state_trajectory_gate_run_842() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!(
            "SKIP state_trajectory_gate_run_842: no serving bundle at {}",
            root.display()
        );
        return;
    };
    let started = Instant::now();

    // === load the deployed engine directly (verbatim #841: the released #833
    //     graph/score.r4g1 + teacher artifacts + tokenizer + score_report) =====
    let graph_bytes = std::fs::read(&bundle.graph).expect("graph bytes");
    let teacher_bytes = std::fs::read(&bundle.teacher).expect("teacher bytes");
    let score_report = bundle
        .graph
        .parent()
        .and_then(|p| std::fs::read(p.join("score_report.json")).ok())
        .filter(|b| serde_json::from_slice::<serde_json::Value>(b).is_ok());
    let meta_bytes = std::fs::read(&bundle.corpus_meta).expect("corpus meta");
    let tokenizer_bytes = std::fs::read(bundle.root.join("tokenizer.bin")).ok();
    let corpus = compiler::load_corpus_from(
        bundle.corpus_meta.to_str().expect("meta utf8"),
        bundle.corpus_records.to_str().expect("recs utf8"),
    )
    .expect("load corpus");
    let (_, held_out) = induction::split_positions(&corpus);
    let corpus_cid = compute_cid(&meta_bytes);
    assert!(
        corpus_cid.starts_with(ATTESTED_CORPUS_CID_PREFIX),
        "run is only valid on the attested #833 bundle (got {corpus_cid})"
    );
    let mut engine = R4Engine::load_accepting_quality(EngineParts {
        graph: &graph_bytes,
        signature_artifact: &teacher_bytes,
        tokenizer: tokenizer_bytes.as_deref(),
        score_report: score_report.as_deref(),
    })
    .expect("engine load");
    // The broad-clean bundle carries NO SKMX/PSIB, so the deployed decision is
    // the base decision -- the same conditions #841 measured (validation below).
    assert_eq!(
        engine.skipmix_tables_present(),
        (false, false),
        "the attested broad-clean bundle must carry no skip-mix lane sections"
    );

    // === frozen prompt-family v1 (verbatim #841/#840 selection) ==============
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

    // === measure student-prefix + teacher-prefix on the frozen family =======
    let mut student = [Aggregate::default(), Aggregate::default()];
    let mut teacher = [Aggregate::default(), Aggregate::default()];
    let mut cause_counts: BTreeMap<Cause, u32> = BTreeMap::new();
    let mut tf_agree_total = 0u64;
    let mut suffix_equal_fr_h32 = 0u32;

    for (hi, &h) in HORIZONS.iter().enumerate() {
        for &i0 in &prompts {
            let prompt = induction::context_window(&corpus, i0);
            let recorded: Vec<u32> = (1..=h).map(|k| corpus.input[i0 + k]).collect();

            let (st_steps, rec_in_cand) = rollout_diag(&mut engine, &prompt, &recorded, h);
            let st_m = metrics_of(&st_steps, &recorded);
            student[hi].push(&st_m, h);

            let (tc_steps, agree) = teacher_prefix(&mut engine, &corpus, i0, h);
            let tc_m = metrics_of(&tc_steps, &recorded);
            teacher[hi].push(&tc_m, h);

            if h == H_MAX {
                tf_agree_total += u64::from(agree);

                // suffix-only control -> reproduce #841's suffix-locality stat.
                let suffix = &prompt[prompt.len() - 2..];
                let (so_steps, _) = rollout_diag(&mut engine, suffix, &recorded, h);
                let st_tok: Vec<Option<u32>> = st_steps.iter().map(|s| s.token).collect();
                let so_tok: Vec<Option<u32>> = so_steps.iter().map(|s| s.token).collect();
                suffix_equal_fr_h32 += u32::from(st_tok == so_tok);

                // classify the failure cause for this prompt.
                let d_s = st_m.first_divergence;
                let d_t = tc_m.first_divergence;
                let in_cand_at_div = d_s
                    .and_then(|d| rec_in_cand.get(d as usize).copied())
                    .unwrap_or(false);
                let cause = classify(d_s, d_t, in_cand_at_div);
                *cause_counts.entry(cause).or_default() += 1;

                // Reference-state consistency (the binding instrument check): the
                // drift-free reference (teacher-prefix) MUST survive longer on
                // state-diagnosed prompts and MUST NOT help step-0 prompts.
                match cause {
                    Cause::StateStarvation => assert!(
                        d_t.is_none_or(|dt| d_s.is_some_and(|d| dt > d)),
                        "state-starvation requires the reference prefix to survive longer"
                    ),
                    Cause::SingleStepAt0 => assert_eq!(
                        d_t,
                        Some(0),
                        "a step-0 failure shares the prompt window, so the reference also diverges at 0"
                    ),
                    _ => {}
                }
            }
        }
    }

    // determinism tooth: the first 5 student rollouts reproduce identically.
    for &i0 in prompts.iter().take(5) {
        let prompt = induction::context_window(&corpus, i0);
        let recorded: Vec<u32> = (1..=H_MAX).map(|k| corpus.input[i0 + k]).collect();
        assert_eq!(
            rollout_diag(&mut engine, &prompt, &recorded, H_MAX).0,
            rollout_diag(&mut engine, &prompt, &recorded, H_MAX).0,
            "greedy rollout deterministic"
        );
    }

    // === reachability arithmetic (the fork numbers) =========================
    let ss = &student[1];
    let ts = &teacher[1];
    let student_median = ss.median_diverge();
    let teacher_median = ts.median_diverge();
    let student_at0 = ss.at0_permille();
    let teacher_at0 = ts.at0_permille();
    let get = |c: Cause| *cause_counts.get(&c).unwrap_or(&0);
    let (n_survived, n_ssl0, n_cand, n_rank, n_state) = (
        get(Cause::Survived),
        get(Cause::SingleStepAt0),
        get(Cause::CandidateGap),
        get(Cause::RankLimit),
        get(Cause::StateStarvation),
    );
    let pm = |v: u32| u64::from(v) * 1000 / N_PROMPTS as u64;
    let state_addressable_pm = pm(n_state);
    let at0_reachable = at0_reachable_drop_pm(student_at0, teacher_at0);
    let clears = ceiling_clears_bar(student_median, teacher_median, student_at0, teacher_at0);
    let verdict_code = if clears { "TRIGGERED" } else { "NOT-TRIGGERED" };
    let generation_verdict = if clears {
        "IMPLEMENT-BOUNDED-STATE"
    } else {
        "GENERATION-NOT-ESTABLISHED"
    };
    let tf_pm = tf_agree_total * 1000 / (N_PROMPTS as u64 * H_MAX as u64);

    // === console report =====================================================
    println!("=== #842 trajectory-state gate (greedy, teacher-free) ===");
    println!("bundle           : {}", bundle.root.display());
    println!("corpus_meta_cid  : {corpus_cid}");
    println!("prompt_family    : v1 n={N_PROMPTS} cid {family_cid}");
    println!(
        "TF agreement     : {tf_pm}permille (reproduces #841 base) over {} matched steps",
        N_PROMPTS * H_MAX
    );
    println!(
        "suffix-locality  : {suffix_equal_fr_h32}/{N_PROMPTS} FR rollouts identical to suffix-only"
    );
    for (name, r) in [("student", &student), ("teacher", &teacher)] {
        for (hi, &h) in HORIZONS.iter().enumerate() {
            let a = &r[hi];
            println!(
                "{name:<8} h={h:<3} median-div {} | at0 {}permille | survived {}permille | cycled {}permille | paths exct/ngram/graph {}/{}/{}",
                a.median_diverge(),
                a.at0_permille(),
                Aggregate::permille(u64::from(a.survived_full), u64::from(a.n)),
                Aggregate::permille(u64::from(a.cycled), u64::from(a.n)),
                a.exact_context,
                a.ngram,
                a.graph,
            );
        }
    }
    println!(
        "diagnosis h32    : survived {n_survived} | single-step@0 {n_ssl0} | candidate-gap {n_cand} | rank-limit {n_rank} | STATE-STARVATION {n_state} (of {N_PROMPTS})"
    );
    println!(
        "reachability     : state-addressable {state_addressable_pm}permille | at0 reachable-drop {at0_reachable}permille (step-0 invariant) vs §6 bar {SEC6_AT0_DROP_PERMILLE}permille"
    );
    println!(
        "ceiling (teacher): median {student_median}->{teacher_median} (need +{SEC6_MEDIAN_RISE}) | at0 {student_at0}->{teacher_at0}permille (need -{SEC6_AT0_DROP_PERMILLE})"
    );
    println!("VERDICT          : {verdict_code} / {generation_verdict}");

    // === CID-bound result record ============================================
    let graph_cid = compute_cid(&graph_bytes);
    let result_cid = compute_cid(
        format!(
            "{tf_pm}:{suffix_equal_fr_h32}:{student_at0}:{teacher_at0}:{student_median}:{teacher_median}:{n_survived}:{n_ssl0}:{n_cand}:{n_rank}:{n_state}:{verdict_code}:{graph_cid}:{corpus_cid}"
        )
        .as_bytes(),
    );
    let json = format!(
        concat!(
            "{{\n",
            "  \"issue\": 842,\n",
            "  \"study\": \"trajectory-state-gate-v1\",\n",
            "  \"execution_scope\": \"deployed R4Engine generation path, teacher-free (recorded t_argmax labels)\",\n",
            "  \"mode\": \"greedy\",\n",
            "  \"bundle\": \"{}\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"prompt_family\": {{\"version\": 1, \"n\": {}, \"cid\": \"{}\"}},\n",
            "  \"graph_cid\": \"{}\",\n",
            "  \"tf_agreement_permille\": {},\n",
            "  \"suffix_locality_of_100\": {},\n",
            "  \"student\": {{\"h8\": {}, \"h32\": {}}},\n",
            "  \"teacher_prefix\": {{\"h8\": {}, \"h32\": {}}},\n",
            "  \"diagnosis_h32\": {{\"survived\": {}, \"single_step_at0\": {}, \"candidate_gap\": {}, \"rank_limit\": {}, \"state_starvation\": {}}},\n",
            "  \"reachability\": {{\"state_addressable_permille\": {}, \"at0_reachable_drop_permille\": {}, \"student_median\": {}, \"teacher_median\": {}, \"student_at0_permille\": {}, \"teacher_at0_permille\": {}, \"sec6_median_rise_required\": {}, \"sec6_at0_drop_required_permille\": {}, \"ceiling_clears_bar\": {}}},\n",
            "  \"verdict\": \"{}\",\n",
            "  \"generation_verdict\": \"{}\",\n",
            "  \"result_cid\": \"{}\"\n",
            "}}\n"
        ),
        bundle.root.display(),
        corpus_cid,
        N_PROMPTS,
        family_cid,
        graph_cid,
        tf_pm,
        suffix_equal_fr_h32,
        agg_json(&student[0], 8),
        agg_json(&student[1], 32),
        agg_json(&teacher[0], 8),
        agg_json(&teacher[1], 32),
        n_survived,
        n_ssl0,
        n_cand,
        n_rank,
        n_state,
        state_addressable_pm,
        at0_reachable,
        student_median,
        teacher_median,
        student_at0,
        teacher_at0,
        SEC6_MEDIAN_RISE,
        SEC6_AT0_DROP_PERMILLE,
        clears,
        verdict_code,
        generation_verdict,
        result_cid,
    );
    let out = repo_root()
        .join("docs")
        .join("state_trajectory_gate_842_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("result_cid       : {result_cid}");
    println!("wrote            : {}", out.display());
    println!("elapsed          : {:.1}s", started.elapsed().as_secs_f64());

    // === validation: the instrument reproduces #841's frozen base numbers ===
    // (loading the deployed graph directly, these must match the sibling-A
    // record exactly; a drift here invalidates the diagnostic.)
    assert_eq!(tf_pm, 304, "reproduce #841 teacher-forced agreement");
    assert_eq!(
        student_at0, 590,
        "reproduce #841 student-prefix diverged-at-0"
    );
    assert_eq!(student_median, 0, "reproduce #841 student-prefix median");
    assert_eq!(
        suffix_equal_fr_h32, 99,
        "reproduce #841 suffix-locality (99/100 FR == suffix-only)"
    );
    // === the S3 gate: the reachable ceiling does not clear the §6 bar =======
    assert!(
        !clears,
        "the drift-free reference (teacher-prefix) does not clear the §6 bar; \
         trajectory state is NOT TRIGGERED at this representation"
    );
    assert_eq!(
        at0_reachable, 0,
        "step-0 invariance: no trajectory-state mechanism can reduce diverged-at-0"
    );
}
