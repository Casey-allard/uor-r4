//! Free-running trajectory-gap evaluation — frozen instrument + first
//! quantification (#841, item A of S3 tracker #824).
//!
//! Companion contract: `docs/free_running_eval_841.md`. The S3 tracker
//! explicitly sanctions preparing this first offline gap instrument once the
//! S0 evaluation is frozen (it is: #832/#833); S3 STAGE closure remains gated
//! on the #822/#823 stage verdicts, which this issue does not touch.
//!
//! ## What this measures
//!
//! Teacher-forced and recorded-corpus-replay signals do not measure how
//! errors compound under the system's OWN prefixes. This harness drives the
//! normative deployed `R4Engine` (the released #833 `graph/score.r4g1`, the
//! ADR-0001 scorer, the D4 policy) over MATCHED trajectory pairs from frozen
//! prompt-family v1:
//!
//!   * **teacher-prefix (TF)** — at each of `H` matched steps the engine
//!     predicts from the RECORDED in-story window (the classic teacher-forced
//!     read, scored against the recorded teacher argmax);
//!   * **student-prefix (FR)** — from the same prompt the engine consumes its
//!     OWN served tokens (greedy, deterministic; an abstention terminates the
//!     trajectory — the honest deployed behavior);
//!   * controls — **shuffled-prompt** (rotate-by-one derangement of the
//!     prompt window), **repeated-prefix** (last token repeated to fill the
//!     window; the pure-repetition reference), **suffix-only** (the last
//!     2 tokens only; the memoryless floor — if FR rollouts equal
//!     suffix-only rollouts, generation itself is suffix-local, the #874
//!     finding extended from scoring to generation).
//!
//! Primary statistic (frozen before any corrective fitting, spec §6): the
//! median first-divergence step at `H = 32` under greedy — the first step
//! where the student token departs the recorded story text (abstention =
//! termination, recorded separately). Secondary deterministic metrics:
//! matched TF agreement, serve/abstain/widen counts, per-step
//! ExactContext/NGRAM/Graph attribution, cycle/repetition structure,
//! distinct-1, prompt-content overlap. Every step of every trajectory is
//! recorded in a versioned, CID-bound trace and is replayable; a versioned
//! secondary judge is contract-defined but UNAVAILABLE in this run (no judge
//! identity is pinned — recorded truthfully, never a vacuous pass).
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test free_running_eval_841 -- --ignored --nocapture
//! Env: R4_CAUSAL_BUNDLE (bundle dir).

#![allow(clippy::doc_lazy_continuation)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_api::capability_suite::compute_cid;
use uor_r4_api::engine::{EngineParts, PredictDecision, R4Engine};
use uor_r4_api::serving_eval::ServingBundle;
use uor_r4_api::ScoreStatus;
use uor_r4_core::transformerless::compiler;
use uor_r4_graph_compiler::induction;

/// Deployed context-window cap (compiler::WINDOW).
const WINDOW: usize = 8;
/// Frozen horizon ladder of prompt-family v1; run-1 executes both rungs.
const HORIZONS: [usize; 2] = [8, 32];
/// Frozen prompt-family size (one prompt per held-out story, first
/// `N_PROMPTS` stories with a full window and `H_MAX` recorded continuation).
const N_PROMPTS: usize = 100;
const H_MAX: usize = 32;
/// Cycle detector: periods checked and the repeat count that flags a cycle.
const MAX_PERIOD: usize = 4;
const CYCLE_FLAG_REPEATS: u32 = 3;
/// Trace schema version (spec §3).
const TRACE_SCHEMA: u32 = 1;

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

// --- §3: versioned trace schema ----------------------------------------------

/// Per-step attribution label (the #832 `ResolutionPath` served subset plus
/// the typed abstention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum StepPath {
    ExactContext,
    Ngram,
    Graph,
    Decline,
}

/// One generated (or matched teacher-forced) step of a trajectory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TraceStep {
    /// Served token, or `None` on the terminal abstention.
    token: Option<u32>,
    path: StepPath,
    widened: bool,
}

/// One recorded trajectory: mode, prompt id, and its steps.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Trace {
    schema: u32,
    prompt_id: u32,
    mode: &'static str,
    steps: Vec<TraceStep>,
}

impl Trace {
    /// Canonical byte serialization (deterministic; the CID input).
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(16 + self.steps.len() * 8);
        b.extend_from_slice(&self.schema.to_le_bytes());
        b.extend_from_slice(&self.prompt_id.to_le_bytes());
        b.extend_from_slice(&(self.mode.len() as u32).to_le_bytes());
        b.extend_from_slice(self.mode.as_bytes());
        b.extend_from_slice(&(self.steps.len() as u32).to_le_bytes());
        for s in &self.steps {
            b.extend_from_slice(&s.token.map_or(u32::MAX, |t| t).to_le_bytes());
            b.push(match s.path {
                StepPath::ExactContext => 0,
                StepPath::Ngram => 1,
                StepPath::Graph => 2,
                StepPath::Decline => 3,
            });
            b.push(u8::from(s.widened));
        }
        b
    }

    /// Parse canonical bytes back (round-trip check; a truncated or
    /// tag-corrupted buffer is a typed `None`, never a panic).
    fn parse(bytes: &[u8]) -> Option<Trace> {
        let mut at = 0usize;
        let take4 = |at: &mut usize| -> Option<u32> {
            let v = bytes.get(*at..*at + 4)?;
            *at += 4;
            Some(u32::from_le_bytes(v.try_into().ok()?))
        };
        let schema = take4(&mut at)?;
        let prompt_id = take4(&mut at)?;
        let mode_len = take4(&mut at)? as usize;
        let mode_bytes = bytes.get(at..at + mode_len)?;
        at += mode_len;
        let mode: &'static str = match mode_bytes {
            b"teacher-prefix" => "teacher-prefix",
            b"student-prefix" => "student-prefix",
            b"shuffled-prompt" => "shuffled-prompt",
            b"repeated-prefix" => "repeated-prefix",
            b"suffix-only" => "suffix-only",
            b"fixture" => "fixture",
            _ => return None,
        };
        let n = take4(&mut at)? as usize;
        let mut steps = Vec::with_capacity(n);
        for _ in 0..n {
            let tok = take4(&mut at)?;
            let path = match bytes.get(at)? {
                0 => StepPath::ExactContext,
                1 => StepPath::Ngram,
                2 => StepPath::Graph,
                3 => StepPath::Decline,
                _ => return None,
            };
            at += 1;
            let widened = *bytes.get(at)? != 0;
            at += 1;
            steps.push(TraceStep {
                token: (tok != u32::MAX).then_some(tok),
                path,
                widened,
            });
        }
        (at == bytes.len()).then_some(Trace {
            schema,
            prompt_id,
            mode,
            steps,
        })
    }
}

// --- §4: deterministic sequence metrics --------------------------------------

/// Integer metrics of one trajectory against its recorded reference
/// continuation. All fields are counts/steps/permille — no floats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SeqMetrics {
    /// First step whose token departs the recorded text (`None` = never
    /// within the horizon; a terminal abstention before any divergence
    /// reports the abstention step here — termination IS departure).
    first_divergence: Option<u32>,
    /// Steps actually served (before any terminal abstention).
    served: u32,
    /// 1 when the trajectory ended on a typed abstention.
    abstained: bool,
    widened: u32,
    exact_context: u32,
    ngram: u32,
    graph: u32,
    /// Longest trailing-cycle repeat count over periods 1..=MAX_PERIOD.
    max_cycle_repeats: u32,
    /// Distinct served tokens, ‰ of served.
    distinct1_permille: u32,
    /// Served tokens that also occur in the prompt window, ‰ of served
    /// (the deterministic prompt-content overlap proxy of spec §4).
    prompt_overlap_permille: u32,
}

/// Longest number of consecutive repeats of any trailing period-`p` cycle
/// (p ≤ MAX_PERIOD) anywhere in the sequence.
fn max_cycle_repeats(seq: &[u32]) -> u32 {
    let n = seq.len();
    let mut best = 0u32;
    for p in 1..=MAX_PERIOD.min(n) {
        // `aligned` counts the current run of positions matching the value
        // one period back; `aligned / p` whole extra periods have repeated.
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

fn metrics_of(trace: &Trace, prompt: &[u32], recorded: &[u32]) -> SeqMetrics {
    let served_tokens: Vec<u32> = trace.steps.iter().filter_map(|s| s.token).collect();
    let served = served_tokens.len() as u32;
    let abstained = trace.steps.last().is_some_and(|s| s.token.is_none());
    let mut first_divergence = None;
    for (k, s) in trace.steps.iter().enumerate() {
        match s.token {
            None => {
                first_divergence = Some(k as u32);
                break;
            }
            Some(t) => {
                if recorded.get(k).copied() != Some(t) {
                    first_divergence = Some(k as u32);
                    break;
                }
            }
        }
    }
    let mut distinct: Vec<u32> = served_tokens.clone();
    distinct.sort_unstable();
    distinct.dedup();
    let mut prompt_set: Vec<u32> = prompt.to_vec();
    prompt_set.sort_unstable();
    prompt_set.dedup();
    let overlap = served_tokens
        .iter()
        .filter(|t| prompt_set.binary_search(t).is_ok())
        .count() as u32;
    let denom = u64::from(served.max(1));
    SeqMetrics {
        first_divergence,
        served,
        abstained,
        widened: trace.steps.iter().filter(|s| s.widened).count() as u32,
        exact_context: trace
            .steps
            .iter()
            .filter(|s| s.path == StepPath::ExactContext)
            .count() as u32,
        ngram: trace
            .steps
            .iter()
            .filter(|s| s.path == StepPath::Ngram)
            .count() as u32,
        graph: trace
            .steps
            .iter()
            .filter(|s| s.path == StepPath::Graph)
            .count() as u32,
        max_cycle_repeats: max_cycle_repeats(&served_tokens),
        distinct1_permille: ((distinct.len() as u64 * 1000) / denom) as u32,
        prompt_overlap_permille: ((u64::from(overlap) * 1000) / denom) as u32,
    }
}

/// Median of a non-empty integer sample (lower median; deterministic).
fn median(mut xs: Vec<u32>) -> u32 {
    xs.sort_unstable();
    xs[(xs.len() - 1) / 2]
}

/// Order-statistic 95% CI ranks for the median of n samples (normal
/// approximation to Binomial(n, 1/2); integer arithmetic on ranks).
fn median_rank_ci(n: usize) -> (usize, usize) {
    // half-width ≈ 0.98 * sqrt(n); integer square root.
    let mut r = 0usize;
    while (r + 1) * (r + 1) <= n {
        r += 1;
    }
    let half = (98 * r).div_ceil(100);
    let lo = (n / 2).saturating_sub(half);
    let hi = (n / 2 + half).min(n - 1);
    (lo, hi)
}

// --- fixture teeth (non-ignored) ---------------------------------------------

fn fixture_trace(tokens: &[Option<u32>], mode: &'static str) -> Trace {
    Trace {
        schema: TRACE_SCHEMA,
        prompt_id: 7,
        mode,
        steps: tokens
            .iter()
            .map(|&t| TraceStep {
                token: t,
                path: if t.is_some() {
                    StepPath::Graph
                } else {
                    StepPath::Decline
                },
                widened: false,
            })
            .collect(),
    }
}

#[test]
fn trace_schema_round_trips_and_is_deterministic() {
    let t = fixture_trace(&[Some(5), Some(9), None], "fixture");
    let b1 = t.canonical_bytes();
    let b2 = t.canonical_bytes();
    assert_eq!(b1, b2, "canonical bytes deterministic");
    let back = Trace::parse(&b1).expect("round trip");
    assert_eq!(back, t);
    // Tampered tag and truncation are typed failures, not panics.
    let mut bad = b1.clone();
    let last_tag = bad.len() - 2;
    bad[last_tag] = 9;
    assert!(Trace::parse(&bad).is_none(), "corrupt path tag rejected");
    assert!(
        Trace::parse(&b1[..b1.len() - 1]).is_none(),
        "truncation rejected"
    );
    let cid1 = compute_cid(&b1);
    let cid2 = compute_cid(&t.canonical_bytes());
    assert_eq!(cid1, cid2, "trace CID stable");
}

#[test]
fn first_divergence_locator_is_correct_on_planted_cases() {
    let recorded = [10u32, 11, 12, 13, 14, 15];
    let prompt = [1u32, 2, 3];
    // Perfect continuation: no divergence.
    let perfect = fixture_trace(&[Some(10), Some(11), Some(12)], "fixture");
    assert_eq!(
        metrics_of(&perfect, &prompt, &recorded).first_divergence,
        None
    );
    // Planted early-drift model: distinct plausible tokens, wrong from step
    // 0 — must read first_divergence = 0 (distinctness alone must not pass).
    let drift = fixture_trace(&[Some(99), Some(98), Some(97)], "fixture");
    let dm = metrics_of(&drift, &prompt, &recorded);
    assert_eq!(dm.first_divergence, Some(0));
    assert!(dm.distinct1_permille == 1000 && dm.max_cycle_repeats <= 1);
    // Mid-trajectory divergence.
    let mid = fixture_trace(&[Some(10), Some(11), Some(77)], "fixture");
    assert_eq!(
        metrics_of(&mid, &prompt, &recorded).first_divergence,
        Some(2)
    );
    // Terminal abstention counts as departure at its step.
    let abst = fixture_trace(&[Some(10), None], "fixture");
    let am = metrics_of(&abst, &prompt, &recorded);
    assert_eq!(am.first_divergence, Some(1));
    assert!(am.abstained);
}

#[test]
fn repetition_only_model_is_flagged_and_distinctness_alone_cannot_pass() {
    let recorded = [10u32; 12];
    let prompt = [1u32, 2];
    // Planted repetition-only model: a period-2 loop.
    let looper = fixture_trace(
        &[
            Some(4),
            Some(5),
            Some(4),
            Some(5),
            Some(4),
            Some(5),
            Some(4),
            Some(5),
        ],
        "fixture",
    );
    let lm = metrics_of(&looper, &prompt, &recorded);
    assert!(
        lm.max_cycle_repeats >= CYCLE_FLAG_REPEATS,
        "cycle detector flags the loop ({} repeats)",
        lm.max_cycle_repeats
    );
    assert!(
        lm.distinct1_permille <= 250,
        "loop has collapsed distinctness"
    );
    // A period-1 stutter is also flagged.
    let stutter = fixture_trace(&[Some(7), Some(7), Some(7), Some(7)], "fixture");
    assert!(metrics_of(&stutter, &prompt, &recorded).max_cycle_repeats >= CYCLE_FLAG_REPEATS);
    // A non-repeating sequence is not flagged.
    let clean = fixture_trace(&[Some(1), Some(2), Some(3), Some(4)], "fixture");
    assert!(metrics_of(&clean, &prompt, &recorded).max_cycle_repeats < CYCLE_FLAG_REPEATS);
}

#[test]
fn prompt_overlap_and_median_ci_are_deterministic() {
    let prompt = [5u32, 6, 7, 8];
    let recorded = [9u32; 4];
    let t = fixture_trace(&[Some(5), Some(6), Some(1), Some(2)], "fixture");
    let m = metrics_of(&t, &prompt, &recorded);
    assert_eq!(m.prompt_overlap_permille, 500);
    // Median + rank CI: deterministic, integer.
    assert_eq!(median(vec![3, 1, 2]), 2);
    assert_eq!(median(vec![4, 1, 2, 3]), 2, "lower median on even n");
    let (lo, hi) = median_rank_ci(100);
    assert!(lo < 50 && hi > 50 && hi < 100);
    assert_eq!((lo, hi), median_rank_ci(100));
}

#[test]
fn judge_metrics_without_a_pinned_judge_are_unavailable() {
    // The contract's secondary-judge discipline (spec §5): a report may carry
    // judge metrics ONLY under a pinned judge identity; without one the
    // metric is UNAVAILABLE — never a vacuous pass, never silently dropped.
    #[derive(Debug, PartialEq, Eq)]
    enum JudgeMetric {
        Measured(u32),
        Unavailable(&'static str),
    }
    fn judge_metric(judge_id: Option<&str>, value: u32) -> JudgeMetric {
        match judge_id {
            Some(_) => JudgeMetric::Measured(value),
            None => JudgeMetric::Unavailable("no judge identity pinned"),
        }
    }
    assert_eq!(
        judge_metric(None, 900),
        JudgeMetric::Unavailable("no judge identity pinned")
    );
    assert_eq!(
        judge_metric(Some("judge-v1"), 900),
        JudgeMetric::Measured(900)
    );
}

// --- the engine-driven run -----------------------------------------------------

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
/// deterministic); an abstention or an engine error terminates.
fn rollout(
    engine: &mut R4Engine,
    prompt: &[u32],
    h: usize,
    prompt_id: u32,
    mode: &'static str,
) -> Trace {
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
    Trace {
        schema: TRACE_SCHEMA,
        prompt_id,
        mode,
        steps,
    }
}

/// Teacher-prefix pass: predict at each matched RECORDED window.
fn teacher_forced(
    engine: &mut R4Engine,
    corpus: &compiler::Corpus,
    i0: usize,
    h: usize,
    prompt_id: u32,
) -> (Trace, u32) {
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
    (
        Trace {
            schema: TRACE_SCHEMA,
            prompt_id,
            mode: "teacher-prefix",
            steps,
        },
        agree,
    )
}

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
    widened: u64,
    distinct1_sum: u64,
    overlap_sum: u64,
    n: u32,
}

impl Aggregate {
    fn new() -> Aggregate {
        Aggregate {
            diverge_steps: Vec::new(),
            diverged_at0: 0,
            survived_full: 0,
            abstained: 0,
            cycled: 0,
            served: 0,
            exact_context: 0,
            ngram: 0,
            graph: 0,
            widened: 0,
            distinct1_sum: 0,
            overlap_sum: 0,
            n: 0,
        }
    }
    fn push(&mut self, m: &SeqMetrics, h: usize) {
        self.n += 1;
        let d = m.first_divergence.unwrap_or(h as u32);
        self.diverge_steps.push(d);
        if m.first_divergence == Some(0) {
            self.diverged_at0 += 1;
        }
        if m.first_divergence.is_none() {
            self.survived_full += 1;
        }
        self.abstained += u32::from(m.abstained);
        self.cycled += u32::from(m.max_cycle_repeats >= CYCLE_FLAG_REPEATS);
        self.served += u64::from(m.served);
        self.exact_context += u64::from(m.exact_context);
        self.ngram += u64::from(m.ngram);
        self.graph += u64::from(m.graph);
        self.widened += u64::from(m.widened);
        self.distinct1_sum += u64::from(m.distinct1_permille);
        self.overlap_sum += u64::from(m.prompt_overlap_permille);
    }
    fn median_diverge(&self) -> u32 {
        median(self.diverge_steps.clone())
    }
    fn json(&self, h: usize) -> String {
        let mut sorted = self.diverge_steps.clone();
        sorted.sort_unstable();
        let (lo_rank, hi_rank) = median_rank_ci(sorted.len().max(1));
        let n = u64::from(self.n.max(1));
        format!(
            "{{\"n\": {}, \"median_first_divergence\": {}, \"median_ci_steps\": [{}, {}], \"diverged_at_step0_permille\": {}, \"survived_full_horizon_permille\": {}, \"abstained_permille\": {}, \"cycled_permille\": {}, \"mean_distinct1_permille\": {}, \"mean_prompt_overlap_permille\": {}, \"paths\": {{\"exact_context\": {}, \"ngram\": {}, \"graph\": {}, \"widened\": {}, \"served\": {}}}, \"horizon\": {}}}",
            self.n,
            self.median_diverge(),
            sorted.get(lo_rank).copied().unwrap_or(0),
            sorted.get(hi_rank).copied().unwrap_or(0),
            u64::from(self.diverged_at0) * 1000 / n,
            u64::from(self.survived_full) * 1000 / n,
            u64::from(self.abstained) * 1000 / n,
            u64::from(self.cycled) * 1000 / n,
            self.distinct1_sum / n,
            self.overlap_sum / n,
            self.exact_context,
            self.ngram,
            self.graph,
            self.widened,
            self.served,
            h
        )
    }
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle; run with --ignored"]
fn free_running_gap_run_841() {
    let root = bundle_root();
    let Some(bundle) = ServingBundle::discover(&root) else {
        eprintln!("SKIP free_running_gap_run_841: no serving bundle");
        return;
    };
    let started = Instant::now();
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
    let mut engine = R4Engine::load_accepting_quality(EngineParts {
        graph: &graph_bytes,
        signature_artifact: &teacher_bytes,
        tokenizer: tokenizer_bytes.as_deref(),
        score_report: score_report.as_deref(),
    })
    .expect("engine load");

    // --- frozen prompt-family v1: first qualifying position per story ------
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
    let corpus_cid = compute_cid(&meta_bytes);

    println!("=== #841 free-running gap run (greedy, deployed engine) ===");
    println!("corpus_meta_cid  : {corpus_cid}");
    println!("prompt_family    : v1, n={N_PROMPTS}, cid {family_cid}");

    let mut json_blocks: Vec<String> = Vec::new();
    let mut trace_cids: Vec<String> = Vec::new();
    let mut suffix_equal_fr = 0u32;
    let mut shuffle_changed = 0u32;
    let mut tf_agree_total = 0u64;
    let mut fr_median_h32 = 0u32;

    for &h in &HORIZONS {
        let mut agg: BTreeMap<&'static str, Aggregate> = BTreeMap::new();
        for mode in [
            "teacher-prefix",
            "student-prefix",
            "shuffled-prompt",
            "repeated-prefix",
            "suffix-only",
        ] {
            agg.insert(mode, Aggregate::new());
        }
        let mut tf_agree = 0u64;
        for (pid, &i0) in prompts.iter().enumerate() {
            let pid32 = pid as u32;
            let prompt = induction::context_window(&corpus, i0);
            let recorded: Vec<u32> = (1..=h).map(|k| corpus.input[i0 + k]).collect();

            let (tf_trace, agree) = teacher_forced(&mut engine, &corpus, i0, h, pid32);
            tf_agree += u64::from(agree);
            let tf_m = metrics_of(&tf_trace, &prompt, &recorded);
            agg.get_mut("teacher-prefix").expect("mode").push(&tf_m, h);

            let fr = rollout(&mut engine, &prompt, h, pid32, "student-prefix");
            let fr_m = metrics_of(&fr, &prompt, &recorded);
            agg.get_mut("student-prefix").expect("mode").push(&fr_m, h);

            let mut shuffled: Vec<u32> = prompt.clone();
            shuffled.rotate_left(1);
            let sh = rollout(&mut engine, &shuffled, h, pid32, "shuffled-prompt");
            let sh_m = metrics_of(&sh, &prompt, &recorded);
            agg.get_mut("shuffled-prompt").expect("mode").push(&sh_m, h);

            let repeated = vec![*prompt.last().expect("prompt"); WINDOW];
            let rp = rollout(&mut engine, &repeated, h, pid32, "repeated-prefix");
            let rp_m = metrics_of(&rp, &prompt, &recorded);
            agg.get_mut("repeated-prefix").expect("mode").push(&rp_m, h);

            let suffix = &prompt[prompt.len() - 2..];
            let so = rollout(&mut engine, suffix, h, pid32, "suffix-only");
            let so_m = metrics_of(&so, &prompt, &recorded);
            agg.get_mut("suffix-only").expect("mode").push(&so_m, h);

            if h == H_MAX {
                let fr_toks: Vec<Option<u32>> = fr.steps.iter().map(|s| s.token).collect();
                let so_toks: Vec<Option<u32>> = so.steps.iter().map(|s| s.token).collect();
                suffix_equal_fr += u32::from(fr_toks == so_toks);
                shuffle_changed += u32::from(
                    fr.steps.iter().map(|s| s.token).collect::<Vec<_>>()
                        != sh.steps.iter().map(|s| s.token).collect::<Vec<_>>(),
                );
                // Trace CIDs (replay identity) for the primary modes.
                trace_cids.push(compute_cid(&tf_trace.canonical_bytes()));
                trace_cids.push(compute_cid(&fr.canonical_bytes()));
            }
        }
        if h == H_MAX {
            tf_agree_total = tf_agree;
            fr_median_h32 = agg["student-prefix"].median_diverge();
        }
        for (mode, a) in &agg {
            println!(
                "h={h:<3} {mode:<16} median-div {} | at0 {}permille | survived {}permille | abstained {}permille | cycled {}permille",
                a.median_diverge(),
                u64::from(a.diverged_at0) * 1000 / u64::from(a.n.max(1)),
                u64::from(a.survived_full) * 1000 / u64::from(a.n.max(1)),
                u64::from(a.abstained) * 1000 / u64::from(a.n.max(1)),
                u64::from(a.cycled) * 1000 / u64::from(a.n.max(1)),
            );
            json_blocks.push(format!("    {:?}: {}", format!("{mode}@h{h}"), a.json(h)));
        }
    }

    // Double-run determinism: the first 5 prompts reproduce identical FR
    // traces (greedy, reset per rollout).
    for (pid, &i0) in prompts.iter().take(5).enumerate() {
        let prompt = induction::context_window(&corpus, i0);
        let a = rollout(&mut engine, &prompt, H_MAX, pid as u32, "student-prefix");
        let b = rollout(&mut engine, &prompt, H_MAX, pid as u32, "student-prefix");
        assert_eq!(a, b, "greedy rollout deterministic");
    }

    // Control non-degeneracy at run scale.
    assert!(
        shuffle_changed > 0,
        "shuffled-prompt must change at least one rollout"
    );

    let tf_rate_pm = tf_agree_total * 1000 / (N_PROMPTS as u64 * H_MAX as u64);
    println!(
        "tf agreement     : {tf_rate_pm}permille over {} matched steps",
        N_PROMPTS * H_MAX
    );
    println!(
        "suffix-locality  : {suffix_equal_fr}/{N_PROMPTS} FR rollouts identical to suffix-only rollouts"
    );
    let traces_cat: Vec<u8> = trace_cids.iter().flat_map(|c| c.bytes()).collect();
    let traces_cid = compute_cid(&traces_cat);
    let elapsed = started.elapsed();
    println!("elapsed          : {:.1}s", elapsed.as_secs_f64());

    let mut rec = Vec::new();
    for v in [
        N_PROMPTS as u64,
        H_MAX as u64,
        tf_agree_total,
        u64::from(fr_median_h32),
        u64::from(suffix_equal_fr),
        u64::from(shuffle_changed),
    ] {
        rec.extend_from_slice(&v.to_le_bytes());
    }
    rec.extend_from_slice(corpus_cid.as_bytes());
    rec.extend_from_slice(family_cid.as_bytes());
    rec.extend_from_slice(traces_cid.as_bytes());
    let result_cid = compute_cid(&rec);
    println!("result_cid       : {result_cid}");

    let json = format!(
        concat!(
            "{{\n",
            "  \"issue\": 841,\n",
            "  \"study\": \"free-running-trajectory-gap-v1\",\n",
            "  \"mode\": \"greedy\",\n",
            "  \"sampled_mode\": {{\"status\": \"UNAVAILABLE\", \"reason\": \"no seeded-sampling driver pinned in run-1; contract defines it\"}},\n",
            "  \"judge_metrics\": {{\"status\": \"UNAVAILABLE\", \"reason\": \"no judge identity pinned; deterministic primaries only\"}},\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"prompt_family\": {{\"version\": 1, \"n\": {}, \"cid\": \"{}\"}},\n",
            "  \"horizons\": [8, 32],\n",
            "  \"teacher_forced_agreement_permille\": {},\n",
            "  \"suffix_locality\": {{\"fr_equals_suffix_only_rollouts\": {}, \"of\": {}}},\n",
            "  \"controls\": {{\"shuffled_prompt_changed_rollouts\": {}}},\n",
            "  \"aggregates\": {{\n{}\n  }},\n",
            "  \"traces_cid\": \"{}\",\n",
            "  \"result_cid\": \"{}\"\n",
            "}}\n"
        ),
        corpus_cid,
        N_PROMPTS,
        family_cid,
        tf_rate_pm,
        suffix_equal_fr,
        N_PROMPTS,
        shuffle_changed,
        json_blocks.join(",\n"),
        traces_cid,
        result_cid,
    );
    let out = repo_root()
        .join("docs")
        .join("free_running_841_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote            : {}", out.display());
}
