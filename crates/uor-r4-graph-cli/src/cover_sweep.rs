//! Cover fineness sweep under the fixed scorer (graph-compiler plan §5,
//! issue #70): a rate–distortion sweep of the induced cover with the
//! issue-#64 scorer held fixed, producing the regions × bytes × agreement
//! table and a recorded operating-point recommendation.
//!
//! # What varies and what is fixed
//!
//! Only the cover fineness knobs vary. The grid is
//! `k0 ∈ {8, 16}` × split threshold `entropy_gain_bits ∈ {0.25, 0.10}` bits
//! × `regions_budget ∈ {128, 512}`, all at the default depth cap 3 — 8
//! points — plus the current default operating point (`k0 = 8`, threshold
//! 0.25 bits, budget [`cover::DEFAULT_REGIONS_BUDGET`], the 42-region
//! baseline row) for 9 points total. Everything else is pinned: the
//! scorer is [`score::ScoreConfig::default`] (the fixed #64 scorer with
//! the #66 ΔT-ablation decision deployed), the observation lane, the
//! train/held-out story cut, and the corpus/artifact inputs are shared
//! across all points.
//!
//! # Per-point pipeline
//!
//! Each point re-runs the exact `cover` → `score` compiler pipeline on
//! the shared inputs: [`cover::induce_cover`], the frozen
//! [`cover::ReferenceClassifier`], [`cover::evaluate_held_out`] for the
//! per-depth reference top-1/top-M recall and frontier width, the
//! structural edges, [`score::compile_transitions`] /
//! [`score::compile_emissions`], [`score::emit_scored_r4g1`] (the scored
//! artifact whose byte length is the rate axis), and
//! [`score::evaluate_gate_c`] on the held-out partition for the Rule 1+2
//! top-1 teacher-argmax agreement and bits/token (the distortion axis).
//!
//! # Recommendation rule (the agreement-per-byte knee)
//!
//! [`recommend`] applies the documented rule, deterministically:
//!
//! 1. Sort the sweep rows by ascending artifact bytes; ties break by
//!    descending Rule 1+2 top-1 agreement, then ascending label.
//! 2. Reduce to the rate–distortion frontier: walking the sorted rows,
//!    keep a row only when its agreement is strictly greater than every
//!    cheaper row's (dominated rows — no fidelity gain for their bytes —
//!    are dropped). Frontier steps therefore have strictly increasing
//!    bytes and strictly increasing agreement.
//! 3. Walk the frontier from the cheapest point, advancing while the
//!    marginal slope `Δagreement / Δbytes` of the step is at least
//!    [`KNEE_SLOPE_FLOOR`] (agreement per byte; `1e-7` = 10 percentage
//!    points of top-1 agreement per added megabyte). The recommendation
//!    is where the walk stops: the first step below the floor is the
//!    knee — fidelity beyond it is not earning its bytes. If no step
//!    clears the floor the cheapest frontier point is recommended; if
//!    every step does, the most expensive one is. An empty grid yields
//!    no recommendation.
//! 4. The recorded justification compares the recommendation against
//!    the baseline row (`Δbytes`, `Δagreement`). The rule never changes
//!    the default [`cover::CoverConfig`] — adoption is an explicit
//!    maintainer decision; this module only writes the recommendation
//!    into the report.
//!
//! # Report schema (`cover_sweep.json`, `schema = 5`)
//!
//! Schema 5 (#610): the sweep builds its cover-independent scoring
//! preparation — the vocabulary and the whole-corpus context n-gram rows
//! and forward-anchor rows, all functions of the corpus, teacher artifact,
//! and fixed scorer and NOT of the induced cover — exactly once per run
//! instead of once per point, handing shared references to every point. The
//! `timing` block gains `preparation_ms` (the one-time shared build), and
//! each point's `context_and_forward_rows_ms` now measures only the
//! per-point reference binding (≈0), so the moved cost stays visible rather
//! than hidden as a false measured-zero: nine repeated whole-corpus passes
//! collapse to one. A [`PreparedScoring`] fingerprint of the inputs and
//! scorer config guards every reuse, so a shared context can never be
//! silently applied to incompatible inputs. Emitted artifact bytes and
//! every score metric are byte-for-byte identical to schema 4 — the caching
//! changes only where the invariant work runs, never its result.
//!
//! Schema 4 (#609): the report additionally records a `timing` block that
//! attributes wall-clock time across the nine-point compile. Each point
//! reports the elapsed milliseconds of its seven pipeline stages (cover
//! induction, recall/edges, transitions, context/forward rows, emissions,
//! R4G1 emission, Gate C), the point total, and the `dominant_stage` — the
//! stage that consumed the most wall time for that point (ties break to the
//! earliest stage in pipeline order). The block also records the run total,
//! the thread count (currently one — the sweep is sequential), and the
//! observation counts, so a future parallel sweep can be compared against
//! this sequential baseline. The instrumentation is pure measurement: it
//! wraps each existing stage in an [`std::time::Instant`] read and changes
//! no artifact bytes and no score values (the schema-3 row fields are
//! byte-for-byte identical with and without the timing block).
//!
//! Schema 3 (#456): every sweep row additionally records `reconstruction`
//! — the EXCT-disabled reconstruction metric (held-out top-1 agreement
//! and bits/token of the graph-only Rule 1 chain,
//! [`score::evaluate_gate_c`]'s `rule1_chain`) — alongside the existing
//! with-EXCT `gate_c_rule12` agreement block, promoting the
//! graph-only reconstructability of the cover from a post-hoc Gate C
//! observation to a per-frontier-point report column. The recommendation
//! block records the recommended point's reconstruction bits/token; the
//! knee rule itself still runs on with-EXCT agreement (reporting only).
//!
//! Schema 2 (#364): the scorer block records `emission_selection`,
//! `emission_shrinkage`, `context_order`, and `context_entries`, and the
//! command accepts `--emission-selection` / `--emission-shrinkage` so the
//! granularity grid can be swept under the contrast-weighted scorer —
//! the previously unmeasured axis behind the #364 cover-ceiling verdict.
//! Everything else in the sweep is unchanged; the default invocation
//! reproduces schema-1 rows exactly (modulo the added config fields).
//! The optional `--distinctiveness-weight` applies the induction objective's
//! between-region distinctiveness reward to every cover point; its default is
//! zero, preserving the byte-exact default cover.
//!
//! ```text
//! schema:          5
//! inputs:          {artifact_kappa, corpus_kappa,
//!                   train_observations, held_out_observations}
//! scorer:          {transition_out_degree, emission_entries, root_top_b,
//!                   exct_top_x, witness_sample, smoothing,
//!                   emission_selection, emission_shrinkage,
//!                   context_order, context_entries}
//!                   — the fixed #64 scorer (with the #67 smoothing knob
//!                   and the #364/#362 emission/context axes)
//! tla3_baseline:   {positions, top1_agreement, bits_per_token}
//!                   (cover-independent store baseline, recorded once)
//! recommendation:  {label, bytes, agreement, slope_floor, frontier,
//!                   delta_bytes_vs_baseline, delta_agreement_vs_baseline,
//!                   reconstruction_bits_per_token, rationale} | null
//! points:          per point, grid order then the baseline row:
//!   {label, baseline, config: {k0, depths, entropy_gain_bits,
//!     regions_budget, min_support, memory_budget_bytes},
//!    regions: {total, per_depth, splits, max_depth},
//!    recall: per depth {depth, evaluated, reference_top1, reference_topm,
//!      frontier_mean, frontier_max},
//!    artifact_bytes, graph_kappa,
//!    gate_c_rule12: {positions, top1_agreement, bits_per_token},
//!    reconstruction: {positions, top1_agreement, bits_per_token}}
//!                   — #456: EXCT-disabled graph-only held-out score
//! timing:          {thread_count, gate_c_sample, preparation_ms,
//!                   train_observations,
//!                   held_out_observations, total_ms, points: per point
//!                   {label, cover_induction_ms, recall_and_edges_ms,
//!                    transitions_ms, context_and_forward_rows_ms,
//!                    emissions_ms, r4g1_emission_ms, gate_c_ms, total_ms,
//!                    dominant_stage}}
//!                   — #609: wall-time attribution (pure measurement);
//!                   #610: `preparation_ms` = one-time cover-independent
//!                   scoring build (context/forward rows + vocab)
//! determinism:     note string
//! ```
//!
//! # Determinism
//!
//! Every consumed compiler is deterministic by construction
//! (content-addressed seeds, ordered reductions, canonical sorts), so any
//! single point run twice produces byte-identical scored artifacts and
//! identical metrics (a property of the consumed compilers; the
//! historical `tests/cover_sweep.rs` reference predates the crate
//! restructuring — a dedicated sweep double-run test is future work). The f64
//! entropy/`ln` sites inherit the macOS-pinned, libm-sensitive status of
//! the cover and score compilers (their module docs); same-machine
//! double-runs are byte-exact, cross-platform byte equality awaits the D2
//! canonical deterministic compile mode.

use serde::Serialize;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use uor_r4_core::transformerless::compiler::{self, Corpus};
use uor_r4_core::transformerless::runtime::{self, Store};
use uor_r4_graph_certify::{self as score, GateCMetrics, ScoreConfig};
use uor_r4_graph_compiler::induction::{self as cover, Observation};
use uor_r4_graph_compiler::reproducibility as repro;
use uor_r4_model_source::SourceUnavailable;

/// The `cover_sweep.json` schema version (module docs). v4 adds the #609
/// per-stage wall-clock `timing` section; v5 (#610) adds `preparation_ms`
/// as the one-time cover-independent scoring build moves out of the loop.
pub const SWEEP_REPORT_SCHEMA: u32 = 5;

/// Grid axis: the broad depth-1 region counts under test.
pub const SWEEP_K0: [usize; 2] = [8, 16];
/// Grid axis: the split entropy-gain floors under test, in bits/token.
pub const SWEEP_ENTROPY_GAIN_BITS: [f64; 2] = [0.25, 0.10];
/// Grid axis: the total-region budgets under test.
pub const SWEEP_REGIONS_BUDGET: [usize; 2] = [128, 512];

/// Marginal agreement-per-byte floor of the recommendation rule
/// (module docs): `1e-7` = 10 percentage points of top-1 agreement per
/// added megabyte of scored artifact.
pub const KNEE_SLOPE_FLOOR: f64 = 1e-7;

/// One sweep point: the cover configuration plus its report label.
#[derive(Debug, Clone, PartialEq)]
pub struct SweepPoint {
    /// Human/JSON label (`k0=8/gain=0.25/budget=128`).
    pub label: String,
    /// True on the default-operating-point row (the 42-region baseline).
    pub baseline: bool,
    /// The cover configuration induced at this point.
    pub config: cover::CoverConfig,
}

/// The 9-point sweep grid (module docs): 8 grid points in
/// (k0, gain, budget) nested order, then the default operating point.
pub fn sweep_grid() -> Vec<SweepPoint> {
    let mut points = Vec::with_capacity(9);
    for &k0 in &SWEEP_K0 {
        for &gain in &SWEEP_ENTROPY_GAIN_BITS {
            for &budget in &SWEEP_REGIONS_BUDGET {
                points.push(SweepPoint {
                    label: format!("k0={k0}/gain={gain}/budget={budget}"),
                    baseline: false,
                    config: cover::CoverConfig {
                        k0,
                        entropy_gain_bits: gain,
                        regions_budget: budget,
                        ..cover::CoverConfig::default()
                    },
                });
            }
        }
    }
    let config = cover::CoverConfig::default();
    points.push(SweepPoint {
        label: format!(
            "k0={}/gain={}/budget={} (default)",
            config.k0, config.entropy_gain_bits, config.regions_budget
        ),
        baseline: true,
        config,
    });
    points
}

/// The inputs shared by every sweep point, loaded/built once (data
/// bundle, the `cover::ReportData` pattern).
pub struct SweepInputs {
    /// TLA container bytes (the teacher artifact).
    pub artifact_container: Vec<u8>,
    /// Parsed teacher artifact.
    pub artifacts: compiler::Compiled,
    /// The labeled corpus stream.
    pub corpus: Corpus,
    /// Corpus metadata bytes (CID material).
    pub meta_bytes: Vec<u8>,
    /// Corpus record bytes (CID material).
    pub recs_bytes: Vec<u8>,
    /// Train observations (stories below the 80/20 cut).
    pub train: Vec<Observation>,
    /// Held-out observations (stories at/above the cut).
    pub held_out: Vec<Observation>,
    /// The graded store (EXCT compiler input + TLA3 baseline).
    pub store: Store,
    /// TLS1 container bytes of the store.
    pub tls1: Vec<u8>,
    /// κ of the artifact container.
    pub artifact_kappa: String,
    /// κ of the corpus stream (meta then records).
    pub corpus_kappa: String,
}

/// Load the shared sweep inputs from disk, mirroring the `score` CLI's
/// loading exactly (same corpus cut, same κs, same store).
pub fn load_inputs(
    corpus_meta: &Path,
    corpus_recs: &Path,
    artifacts_path: &Path,
) -> Result<SweepInputs, SourceUnavailable> {
    let meta_str = corpus_meta
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus metadata path is not UTF-8"))?;
    let recs_str = corpus_recs
        .to_str()
        .ok_or_else(|| SourceUnavailable::new("corpus records path is not UTF-8"))?;
    // #450: announce the resolved containers before the sweep's long work.
    let artifact_container = std::fs::read(artifacts_path).map_err(|error| {
        SourceUnavailable::new(format!("{}: {error}", artifacts_path.display()))
    })?;
    let artifacts = compiler::parse_artifacts(&artifact_container).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{}: not a TLA3/TLA4/TLA5 artifact container",
            artifacts_path.display()
        ))
    })?;
    let artifact_kappa = repro::container_kappa(&artifact_container);
    repro::announce_teacher_container(artifacts_path, &artifact_kappa);
    let meta_bytes = std::fs::read(corpus_meta)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", corpus_meta.display())))?;
    let recs_bytes = std::fs::read(corpus_recs)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", corpus_recs.display())))?;
    let corpus_kappa = repro::corpus_stream_kappa(&meta_bytes, &recs_bytes);
    repro::announce_corpus(corpus_meta, corpus_recs, &corpus_kappa);
    let corpus = compiler::load_corpus_from(meta_str, recs_str).ok_or_else(|| {
        SourceUnavailable::new(format!(
            "corpus is incomplete at {}/{}; run compile until it is complete",
            corpus_meta.display(),
            corpus_recs.display()
        ))
    })?;
    let (train_positions, held_out_positions) = cover::split_positions(&corpus);
    let train = cover::build_observations(&artifacts, &corpus, &train_positions);
    let held_out = cover::build_observations(&artifacts, &corpus, &held_out_positions);
    let (store, _) = runtime::build_store(&artifacts, &corpus);
    let tls1 = runtime::store_bytes(&store);
    Ok(SweepInputs {
        artifact_container,
        artifacts,
        corpus,
        meta_bytes,
        recs_bytes,
        train,
        held_out,
        store,
        tls1,
        artifact_kappa,
        corpus_kappa,
    })
}

/// Per-depth routing numbers of one sweep point (the rate–distortion
/// table's recall columns; a focused subset of [`cover::DepthRecall`]).
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SweepDepthRecall {
    /// Multiresolution depth.
    pub depth: usize,
    /// Held-out positions evaluated at this depth.
    pub evaluated: usize,
    /// P(shipped binary top-1 == exact reference top-1).
    pub reference_top1: f64,
    /// P(exact reference top-1 ∈ binary top-M membership).
    pub reference_topm: f64,
    /// Mean active-region count (frontier width) at this depth.
    pub frontier_mean: f64,
    /// Max active-region count at this depth.
    pub frontier_max: u32,
}

/// Region-count summary of one sweep point.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SweepRegions {
    /// Total induced regions.
    pub total: usize,
    /// Region count per depth (`per_depth[d - 1]`, ascending depth).
    pub per_depth: Vec<u32>,
    /// Regions with an accepted split.
    pub splits: usize,
    /// Deepest depth with at least one region.
    pub max_depth: usize,
}

/// The cover configuration columns of one report row.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SweepRowConfig {
    pub k0: usize,
    pub depths: usize,
    /// Split entropy-gain floor in bits/token.
    pub entropy_gain_bits: f64,
    pub regions_budget: usize,
    pub min_support: usize,
    pub memory_budget_bytes: u64,
    /// Induction-time reward weight against the global next-token prior.
    pub distinctiveness_weight: f64,
}

/// One rate–distortion table row: one sweep point's regions × bytes ×
/// agreement plus the routing-recall detail.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SweepRow {
    /// Sweep-point label.
    pub label: String,
    /// True on the default-operating-point (baseline) row.
    pub baseline: bool,
    /// The cover configuration induced at this point.
    pub config: SweepRowConfig,
    /// Region counts by depth.
    pub regions: SweepRegions,
    /// Per-depth held-out routing recall and frontier width.
    pub recall: Vec<SweepDepthRecall>,
    /// Scored R4G1 artifact size — the rate axis.
    pub artifact_bytes: usize,
    /// κ of the scored artifact bytes.
    pub graph_kappa: String,
    /// Gate C Rule 1+2 (chain + D4 EXCT precedence) on held-out — the
    /// distortion axis.
    pub gate_c_rule12: GateCMetrics,
    /// Issue #456: Gate C Rule 1 chain (EXCT disabled) on held-out — the
    /// graph-only reconstruction score of this cover point, recorded
    /// alongside the with-EXCT agreement.
    pub reconstruction: GateCMetrics,
}

/// The recorded operating-point recommendation (module docs for the
/// rule). `None` deltas mean the sweep carried no baseline row.
#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    /// Label of the recommended point.
    pub label: String,
    /// Its scored-artifact bytes.
    pub bytes: usize,
    /// Its Rule 1+2 top-1 agreement.
    pub agreement: f64,
    /// The slope floor applied ([`KNEE_SLOPE_FLOOR`]).
    pub slope_floor: f64,
    /// Labels of the frontier points in walk order (cheapest first).
    pub frontier: Vec<String>,
    /// `recommended − baseline` artifact bytes, when a baseline row exists.
    pub delta_bytes_vs_baseline: Option<i64>,
    /// `recommended − baseline` Rule 1+2 top-1 agreement.
    pub delta_agreement_vs_baseline: Option<f64>,
    /// The recommended point's EXCT-disabled reconstruction bits/token
    /// (#456; reporting only — the knee rule still runs on agreement).
    pub reconstruction_bits_per_token: f64,
    /// The written justification (numbers + the rule's application).
    pub rationale: String,
}

/// The fixed scorer configuration, recorded for report honesty.
#[derive(Debug, Clone, Serialize)]
pub struct SweepReportScorer {
    pub transition_out_degree: usize,
    pub emission_entries: usize,
    pub root_top_b: usize,
    pub exct_top_x: usize,
    pub witness_sample: usize,
    /// Emission smoothing rule label (`score::Smoothing::label`; the
    /// #67 knob — add-one, byte-exact with the pre-#67 compiler).
    pub smoothing: String,
    /// Emission list selection rule label (#364; schema 2).
    pub emission_selection: String,
    /// Per-region residual shrinkage label (#364; schema 2). The sweep
    /// had only ever run under the default scorer, so granularity ×
    /// contrast-shrinkage was unmeasured — the #364 ceiling question.
    pub emission_shrinkage: String,
    /// Highest compiled lexical context order (#362 knob; schema 2).
    pub context_order: u8,
    /// Per-context candidate bound for NGRAM rows (schema 2).
    pub context_entries: usize,
}

/// The shared-input provenance of the sweep.
#[derive(Debug, Clone, Serialize)]
pub struct SweepReportInputs {
    pub artifact_kappa: String,
    pub corpus_kappa: String,
    pub train_observations: usize,
    pub held_out_observations: usize,
}

/// Milliseconds elapsed since `start` (saturating into `u64`; a sweep
/// stage never runs long enough to overflow).
fn elapsed_ms(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Per-stage wall-clock timing for one sweep point (#609). This is pure
/// measurement: wrapping each stage in an `Instant` never changes the
/// artifact bytes or the scores, so a timed run is comparable to an
/// untimed one and to a future parallel run.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct PointTiming {
    /// The point label (`k0=8/gain=0.25/budget=128`).
    pub label: String,
    pub cover_induction_ms: u64,
    /// Held-out routing recall plus edge/region/structural construction.
    pub recall_and_edges_ms: u64,
    pub transitions_ms: u64,
    pub context_and_forward_rows_ms: u64,
    pub emissions_ms: u64,
    /// Scored R4G1 artifact emission.
    pub r4g1_emission_ms: u64,
    pub gate_c_ms: u64,
    /// Sum of the stages above.
    pub total_ms: u64,
    /// The single stage with the largest wall time — the dominant cost
    /// this instrumentation exists to identify for every point.
    pub dominant_stage: String,
}

impl PointTiming {
    #[allow(clippy::too_many_arguments)]
    fn new(
        label: String,
        cover_induction_ms: u64,
        recall_and_edges_ms: u64,
        transitions_ms: u64,
        context_and_forward_rows_ms: u64,
        emissions_ms: u64,
        r4g1_emission_ms: u64,
        gate_c_ms: u64,
    ) -> Self {
        let stages = [
            ("cover_induction", cover_induction_ms),
            ("recall_and_edges", recall_and_edges_ms),
            ("transitions", transitions_ms),
            ("context_and_forward_rows", context_and_forward_rows_ms),
            ("emissions", emissions_ms),
            ("r4g1_emission", r4g1_emission_ms),
            ("gate_c", gate_c_ms),
        ];
        let total_ms = stages.iter().map(|(_, ms)| *ms).sum();
        // Strictly-greater replacement keeps the FIRST stage on ties (in the
        // fixed order above), so the dominant label is deterministic given
        // the timings — unlike `max_by_key`, which keeps the last.
        let mut dominant = stages[0];
        for &entry in &stages[1..] {
            if entry.1 > dominant.1 {
                dominant = entry;
            }
        }
        let dominant_stage = dominant.0.to_owned();
        Self {
            label,
            cover_induction_ms,
            recall_and_edges_ms,
            transitions_ms,
            context_and_forward_rows_ms,
            emissions_ms,
            r4g1_emission_ms,
            gate_c_ms,
            total_ms,
            dominant_stage,
        }
    }
}

/// Sweep-level timing context plus the per-point stage timings (#609):
/// enough to attribute wall time at every point and to compare a
/// sequential run against a future parallel one (#612).
#[derive(Debug, Clone, Serialize)]
pub struct SweepTiming {
    /// Points evaluated concurrently (`1` = serial; #612 raises this). Kept
    /// in the report so a parallel run is interpretable against this one.
    pub thread_count: usize,
    /// `R4_GATE_C_SAMPLE` at run time (Gate C held-out sampling cap; absent
    /// means the full held-out slice was scored). Recorded because it moves
    /// the Gate C stage cost.
    pub gate_c_sample: Option<String>,
    /// #610: wall time of the one-time cover-independent scoring build
    /// ([`PreparedScoring::prepare`] — vocab + context/forward rows) that
    /// used to run once per point. Each point's `context_and_forward_rows_ms`
    /// now measures only the per-point reference binding (≈0); this is where
    /// that whole-corpus cost moved to.
    pub preparation_ms: u64,
    pub train_observations: usize,
    pub held_out_observations: usize,
    /// Wall time of the whole point loop. On a serial run this is about the
    /// sum of the point totals; a parallel run drives it well below that.
    pub total_ms: u64,
    /// Per-point stage timings, in sweep-grid order.
    pub points: Vec<PointTiming>,
}

/// The `cover_sweep.json` document (schema in the module docs).
#[derive(Debug, Clone, Serialize)]
pub struct SweepReport {
    pub schema: u32,
    pub inputs: SweepReportInputs,
    /// The fixed #64 scorer configuration used at every point.
    pub scorer: SweepReportScorer,
    /// The cover-independent TLA3 store baseline, recorded once.
    pub tla3_baseline: GateCMetrics,
    /// The operating-point recommendation (the documented knee rule).
    pub recommendation: Option<Recommendation>,
    /// The rate–distortion rows: grid points then the baseline row.
    pub points: Vec<SweepRow>,
    /// #609 per-stage wall-clock attribution (does not affect any scored
    /// value; wall time is non-deterministic and excluded from the
    /// byte-equality determinism claim).
    pub timing: SweepTiming,
    /// Determinism status note.
    pub determinism: String,
}

/// Cover-independent scoring preparation, built once per sweep (#610).
///
/// The vocabulary, the context n-gram rows ([`score::compile_context_rows`]),
/// and the forward-anchor rows ([`score::compile_forward_anchor_rows`]) are
/// functions of the corpus, the teacher artifact, and the fixed scorer
/// config only — never of the induced cover — so the nine-point sweep can
/// build them once and hand shared references to every point instead of
/// repeating three whole-corpus passes nine times. Cover-dependent work
/// (induction, regions, transitions, emissions, artifact emission, and Gate
/// C, whose scorers are rebuilt from each point's own artifact bytes) stays
/// per point.
///
/// The stored [`PreparedScoring::fingerprint`] pins the exact inputs and
/// scorer this context was built for; [`PreparedScoring::validate`] refuses
/// reuse against any mismatch, so a shared, immutable context can never be
/// silently applied to incompatible inputs. (The single-point
/// [`reconstruction_null`] path does not loop over covers and so keeps its
/// own inline preparation; only the sweep loop repeated the work.)
pub struct PreparedScoring {
    /// Vocabulary size (teacher token codes / compiler stages).
    vocab: u32,
    /// Context bigram/trigram rows, invariant across cover points.
    context_rows: Vec<score::ContextRow>,
    /// Forward-anchor rows, invariant across cover points.
    fwd_rows: Vec<score::ForwardAnchorRow>,
    /// Identity of the inputs + scorer this context was prepared for; a
    /// reuse whose fingerprint differs is rejected by [`Self::validate`].
    fingerprint: String,
}

impl PreparedScoring {
    /// Build the cover-independent scoring context once from the shared
    /// inputs and the fixed scorer config.
    pub fn prepare(inputs: &SweepInputs, score_config: &ScoreConfig) -> Self {
        let vocab = u32::try_from(inputs.artifacts.token_codes.len() / compiler::STAGES)
            .expect("vocabulary exceeds u32 token ids");
        let context_rows =
            score::compile_context_rows(&inputs.corpus, &inputs.train, vocab, score_config);
        let fwd_rows = score::compile_forward_anchor_rows(&inputs.corpus, &inputs.train);
        Self {
            vocab,
            context_rows,
            fwd_rows,
            fingerprint: scoring_fingerprint(inputs, score_config),
        }
    }

    /// Panic if this context was not prepared for `inputs`/`score_config`.
    /// Reuse across incompatible inputs would silently score a cover point
    /// against another run's corpus/scorer tables — a correctness fault, so
    /// it aborts rather than degrading quietly.
    fn validate(&self, inputs: &SweepInputs, score_config: &ScoreConfig) {
        let expected = scoring_fingerprint(inputs, score_config);
        assert_eq!(
            self.fingerprint, expected,
            "prepared scoring context does not match the sweep inputs/scorer it is applied to"
        );
    }
}

/// Identity string for the inputs + scorer a [`PreparedScoring`] context is
/// valid for. The two κs pin the corpus stream and teacher artifact (and
/// therefore the derived train observations and vocab); the scorer fields
/// pin every knob that feeds the context/forward-row builders (and more,
/// conservatively), so any incompatible reuse is caught.
fn scoring_fingerprint(inputs: &SweepInputs, score_config: &ScoreConfig) -> String {
    fingerprint_string(
        &inputs.artifact_kappa,
        &inputs.corpus_kappa,
        inputs.train.len(),
        score_config,
    )
}

/// Pure fingerprint builder (testable without a full [`SweepInputs`]).
fn fingerprint_string(
    artifact_kappa: &str,
    corpus_kappa: &str,
    train_len: usize,
    score_config: &ScoreConfig,
) -> String {
    format!(
        "artifact={}|corpus={}|train={}|order={}|entries={}|smoothing={}|selection={}|\
         shrinkage={}|out_degree={}|emission_entries={}|root_top_b={}|exct_top_x={}|witness={}",
        artifact_kappa,
        corpus_kappa,
        train_len,
        score_config.context_order,
        score_config.context_entries,
        score_config.smoothing.label(),
        score_config.emission_selection.label(),
        score_config.emission_shrinkage.label(),
        score_config.transition_out_degree,
        score_config.emission_entries,
        score_config.root_top_b,
        score_config.exct_top_x,
        score_config.witness_sample,
    )
}

/// Run one sweep point: induce the cover, measure held-out routing
/// recall, emit the scored R4G1, and run Gate C. Returns the report row,
/// the cover-independent TLA3 baseline metrics (identical at every
/// point), the scored artifact bytes (for the determinism double-run
/// assertion; the sweep itself keeps only their length and κ), and the
/// #609 per-stage wall-clock timing. The `Instant` wrappers are pure
/// measurement — they never change the artifact bytes or the scores.
pub fn run_point(
    inputs: &SweepInputs,
    point: &SweepPoint,
    score_config: &ScoreConfig,
    prepared: &PreparedScoring,
) -> Option<(SweepRow, GateCMetrics, Vec<u8>, PointTiming)> {
    // #610: refuse a prepared context that was not built for these inputs.
    prepared.validate(inputs, score_config);
    let stage = Instant::now();
    let induced = cover::induce_cover(
        &inputs.train,
        &point.config,
        &inputs.artifact_kappa,
        &inputs.corpus_kappa,
    )?;
    let cover_induction_ms = elapsed_ms(stage);

    let stage = Instant::now();
    let reference = cover::ReferenceClassifier::freeze(&induced.cover);
    let recall = cover::evaluate_held_out(
        &inputs.artifacts,
        &induced.cover,
        &reference,
        &inputs.train,
        &inputs.held_out,
    );
    let edges = cover::build_edges(
        &induced.cover,
        &reference,
        &inputs.train,
        &inputs.corpus.story,
    );
    let regions = score::regions_from_cover(&induced.cover);
    let structural = score::structural_from_cover(&edges);
    let max_depth = induced.cover.max_depth;
    let recall_and_edges_ms = elapsed_ms(stage);

    let stage = Instant::now();
    let (transitions, transition_quantization) = score::compile_transitions_with_quantization(
        &inputs.corpus,
        &regions,
        &inputs.train,
        max_depth,
        score_config.transition_out_degree,
    );
    let transitions_ms = elapsed_ms(stage);

    let vocab = prepared.vocab;

    let stage = Instant::now();
    // #610: the context and forward-anchor rows are cover-independent and
    // were built once per sweep in `PreparedScoring`; here we only bind the
    // shared references, so this stage now measures the per-point binding
    // cost (≈0). The one-time build is recorded in the report's
    // `preparation_ms` — the moved cost stays visible, not a false zero.
    let context_rows: &[score::ContextRow] = &prepared.context_rows;
    let fwd_rows: &[score::ForwardAnchorRow] = &prepared.fwd_rows;
    let context_and_forward_rows_ms = elapsed_ms(stage);

    let stage = Instant::now();
    let emissions = score::compile_emissions(
        &inputs.corpus,
        &inputs.store,
        &regions,
        &inputs.train,
        max_depth,
        vocab,
        score_config,
    );
    let emissions_ms = elapsed_ms(stage);

    let stage = Instant::now();
    let (artifact_bytes, _info) = score::emit_scored_r4g1(
        &inputs.artifact_container,
        (&inputs.meta_bytes, &inputs.recs_bytes),
        vocab,
        &score::ScoredGraphSections {
            regions: &regions,
            structural: &structural,
            transitions: &transitions,
            transition_quantization,
            emissions: &emissions,
            context_rows,
            exct_tls1: &inputs.tls1,
            exct_top_x: score_config.exct_top_x,
            fwd_rows,
            // #897: not yet wired into this sweep's fitting stage; empty
            // means the SKMX/PSIB sections are not emitted (absent-section
            // identity, unchanged behavior).
            skipmix_rows: &[],
            psi_bag_rows: &[],
        },
    );
    let r4g1_emission_ms = elapsed_ms(stage);

    let stage = Instant::now();
    // #611: the slim sweep evaluator computes exactly the three metrics this
    // row reads (Rule 1+2, Rule 1 reconstruction, TLA3 baseline) — two scorers
    // and one lean pass — instead of the full evaluator's ~thirty arms.
    let gate_c = score::evaluate_gate_c_sweep(
        &artifact_bytes,
        &inputs.artifact_container,
        &inputs.artifacts,
        &inputs.store,
        &inputs.corpus,
        &inputs.held_out,
        score_config,
    )?;
    let gate_c_ms = elapsed_ms(stage);

    let timing = PointTiming::new(
        point.label.clone(),
        cover_induction_ms,
        recall_and_edges_ms,
        transitions_ms,
        context_and_forward_rows_ms,
        emissions_ms,
        r4g1_emission_ms,
        gate_c_ms,
    );

    let cover = &induced.cover;
    let mut per_depth = vec![0u32; cover.max_depth];
    for region in &cover.regions {
        per_depth[region.depth as usize - 1] += 1;
    }
    let splits = cover
        .regions
        .iter()
        .filter(|r| !r.children.is_empty())
        .count();
    let graph_kappa = format!("blake3:{}", blake3::hash(&artifact_bytes).to_hex());
    let row = SweepRow {
        label: point.label.clone(),
        baseline: point.baseline,
        config: SweepRowConfig {
            k0: point.config.k0,
            depths: point.config.depths,
            entropy_gain_bits: point.config.entropy_gain_bits,
            regions_budget: point.config.regions_budget,
            min_support: point.config.min_support,
            memory_budget_bytes: point.config.memory_budget_bytes,
            distinctiveness_weight: point
                .config
                .objective
                .weights
                .between_region_distinctiveness,
        },
        regions: SweepRegions {
            total: cover.regions.len(),
            per_depth,
            splits,
            max_depth,
        },
        recall: recall
            .iter()
            .map(|d| SweepDepthRecall {
                depth: d.depth,
                evaluated: d.evaluated,
                reference_top1: d.reference_top1_recall,
                reference_topm: d.reference_topm_recall,
                frontier_mean: d.frontier_width_mean,
                frontier_max: d.frontier_width_max,
            })
            .collect(),
        artifact_bytes: artifact_bytes.len(),
        graph_kappa,
        gate_c_rule12: gate_c.rule12_precedence.clone(),
        reconstruction: gate_c.rule1_chain.clone(),
    };
    Some((row, gate_c.tla3_baseline.clone(), artifact_bytes, timing))
}

/// #456 null arm (K-2 mutation discipline). The EXCT-disabled reconstruction
/// metric ([`SweepRow::reconstruction`]) is trusted to reflect a cover's actual
/// residual STRUCTURE, not merely its shape or byte budget. This re-scores one
/// point twice on the SAME held-out slice — once with the compiled emission
/// tables, once with the per-region ΔE lists DERANGED (every region reads a
/// different region's residuals; the root prior is untouched) — and reports both
/// against the unigram floor. A certificate that still beats the floor on the
/// deranged tables would be measuring an artifact, not reconstructability.
///
/// `held_cap` truncates the held-out slice so the two Gate C passes stay cheap
/// (this is a validity check, not a precision measurement); pass `usize::MAX`
/// for the full split. Deterministic given `seed`.
pub struct ReconstructionNull {
    /// The sweep point measured.
    pub label: String,
    /// Held-out positions scored in each arm (after `held_cap`).
    pub held_out_scored: usize,
    /// EXCT-disabled reconstruction with the real emission tables.
    pub real: GateCMetrics,
    /// EXCT-disabled reconstruction with the deranged emission tables.
    pub null: GateCMetrics,
    /// Train-unigram null top-1 on the same held-out slice (the floor).
    pub unigram_top1: f64,
    /// Train-unigram null bits/token on the same held-out slice (the floor).
    pub unigram_bits: f64,
}

pub fn reconstruction_null(
    inputs: &SweepInputs,
    point: &SweepPoint,
    score_config: &ScoreConfig,
    held_cap: usize,
    seed: u64,
) -> Option<ReconstructionNull> {
    let induced = cover::induce_cover(
        &inputs.train,
        &point.config,
        &inputs.artifact_kappa,
        &inputs.corpus_kappa,
    )?;
    let reference = cover::ReferenceClassifier::freeze(&induced.cover);
    let edges = cover::build_edges(
        &induced.cover,
        &reference,
        &inputs.train,
        &inputs.corpus.story,
    );
    let regions = score::regions_from_cover(&induced.cover);
    let structural = score::structural_from_cover(&edges);
    let max_depth = induced.cover.max_depth;
    let (transitions, transition_quantization) = score::compile_transitions_with_quantization(
        &inputs.corpus,
        &regions,
        &inputs.train,
        max_depth,
        score_config.transition_out_degree,
    );
    let vocab = u32::try_from(inputs.artifacts.token_codes.len() / compiler::STAGES)
        .expect("vocabulary exceeds u32 token ids");
    let context_rows =
        score::compile_context_rows(&inputs.corpus, &inputs.train, vocab, score_config);
    let fwd_rows = score::compile_forward_anchor_rows(&inputs.corpus, &inputs.train);
    let emissions = score::compile_emissions(
        &inputs.corpus,
        &inputs.store,
        &regions,
        &inputs.train,
        max_depth,
        vocab,
        score_config,
    );

    let held_len = held_cap.min(inputs.held_out.len());
    let held = &inputs.held_out[..held_len];

    let score_with = |tables: &score::EmissionTables| -> Option<score::SweepGateC> {
        let (artifact_bytes, _info) = score::emit_scored_r4g1(
            &inputs.artifact_container,
            (&inputs.meta_bytes, &inputs.recs_bytes),
            vocab,
            &score::ScoredGraphSections {
                regions: &regions,
                structural: &structural,
                transitions: &transitions,
                transition_quantization,
                emissions: tables,
                context_rows: &context_rows,
                exct_tls1: &inputs.tls1,
                exct_top_x: score_config.exct_top_x,
                fwd_rows: &fwd_rows,
                // #897: not yet wired into this sweep's fitting stage; empty
                // means the SKMX/PSIB sections are not emitted
                // (absent-section identity, unchanged behavior).
                skipmix_rows: &[],
                psi_bag_rows: &[],
            },
        );
        // #611: reconstruction_null reads only rule1_chain + the analytic
        // nulls, both produced by the slim sweep evaluator.
        score::evaluate_gate_c_sweep(
            &artifact_bytes,
            &inputs.artifact_container,
            &inputs.artifacts,
            &inputs.store,
            &inputs.corpus,
            held,
            score_config,
        )
    };

    let real_gate = score_with(&emissions)?;
    let mut shuffled = emissions.clone();
    derange_region_lists(&mut shuffled.region_lists, seed);
    let null_gate = score_with(&shuffled)?;

    Some(ReconstructionNull {
        label: point.label.clone(),
        held_out_scored: held_len,
        real: real_gate.rule1_chain.clone(),
        null: null_gate.rule1_chain.clone(),
        unigram_top1: real_gate.nulls.unigram_null_top1_all,
        unigram_bits: real_gate.nulls.unigram_null_bits_all,
    })
}

/// Seeded derangement of the per-region ΔE lists (no region keeps its own list,
/// so the mutation is fair). xorshift64* — no `rand`, deterministic per `seed`,
/// so the null arm is reproducible.
fn derange_region_lists<T: Clone>(lists: &mut [Vec<T>], seed: u64) {
    let n = lists.len();
    if n < 2 {
        return;
    }
    let mut state = seed | 1;
    let mut next = || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    let mut perm: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (next() % (i as u64 + 1)) as usize;
        perm.swap(i, j);
    }
    // Eliminate any fixed points so every region genuinely reads another's list.
    for i in 0..n {
        if perm[i] == i {
            let swap_with = (i + 1) % n;
            perm.swap(i, swap_with);
        }
    }
    let old = lists.to_vec();
    for (i, slot) in lists.iter_mut().enumerate() {
        *slot = old[perm[i]].clone();
    }
}

/// The agreement-per-byte knee rule (module docs). Deterministic: the
/// sort keys are total, so equal (bytes, agreement) rows resolve by
/// label. `None` on an empty grid.
pub fn recommend(rows: &[SweepRow]) -> Option<Recommendation> {
    if rows.is_empty() {
        return None;
    }
    let mut sorted: Vec<&SweepRow> = rows.iter().collect();
    sorted.sort_by(|a, b| {
        a.artifact_bytes
            .cmp(&b.artifact_bytes)
            .then_with(|| {
                b.gate_c_rule12
                    .top1_agreement
                    .partial_cmp(&a.gate_c_rule12.top1_agreement)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.label.cmp(&b.label))
    });
    // Rate–distortion frontier: strictly increasing agreement as bytes
    // grow (dominated rows dropped).
    let mut frontier: Vec<&SweepRow> = Vec::new();
    for row in sorted {
        let dominated = frontier
            .last()
            .is_some_and(|f| row.gate_c_rule12.top1_agreement <= f.gate_c_rule12.top1_agreement);
        if !dominated {
            frontier.push(row);
        }
    }
    // Walk while the marginal slope clears the floor; stop at the knee.
    let mut chosen = frontier[0];
    let mut stopped_at_knee = false;
    for pair in frontier.windows(2) {
        let (prev, cur) = (pair[0], pair[1]);
        let d_bytes = (cur.artifact_bytes - prev.artifact_bytes) as f64;
        let d_agreement = cur.gate_c_rule12.top1_agreement - prev.gate_c_rule12.top1_agreement;
        if d_agreement / d_bytes >= KNEE_SLOPE_FLOOR {
            chosen = cur;
        } else {
            stopped_at_knee = true;
            break;
        }
    }
    let baseline = rows.iter().find(|r| r.baseline);
    let delta_bytes = baseline.map(|b| chosen.artifact_bytes as i64 - b.artifact_bytes as i64);
    let delta_agreement =
        baseline.map(|b| chosen.gate_c_rule12.top1_agreement - b.gate_c_rule12.top1_agreement);
    let mut rationale = String::new();
    let walk_note = if frontier.len() == 1 {
        "single-point frontier (every other point is dominated)".to_owned()
    } else if stopped_at_knee {
        format!(
            "the next frontier step's marginal slope falls below the {:.0e} agreement/byte floor",
            KNEE_SLOPE_FLOOR
        )
    } else if chosen.label == frontier[0].label {
        format!(
            "no frontier step clears the {:.0e} agreement/byte floor",
            KNEE_SLOPE_FLOOR
        )
    } else {
        format!(
            "every walked frontier step clears the {:.0e} agreement/byte floor",
            KNEE_SLOPE_FLOOR
        )
    };
    let _ = write!(
        rationale,
        "knee rule over {} frontier point(s) ({}): recommended {}",
        frontier.len(),
        walk_note,
        chosen.label
    );
    if let (Some(db), Some(da), Some(base)) = (delta_bytes, delta_agreement, baseline) {
        let verdict = if chosen.label == base.label {
            "the default operating point is itself the recommendation — the sweep finds no \
             fineness change worth its bytes under the fixed scorer"
        } else if db == 0 && da == 0.0 {
            "the recommended point ties the baseline exactly (identical bytes and fidelity — the \
             swept knob is inert between them); the 42-region default is confirmed adequate \
             under the fixed scorer: no grid point buys agreement with bytes"
        } else if da > 0.0 {
            "the 42-region default is too coarse under the fixed scorer: the recommended point \
             buys agreement at a marginal rate the floor accepts"
        } else if da == 0.0 {
            "the baseline's exact fidelity is available cheaper: the 42-region default carries \
             bytes the fixed scorer does not spend"
        } else {
            "the baseline's extra fidelity costs more per byte than the floor allows under the \
             fixed scorer"
        };
        let _ = write!(
            rationale,
            "; vs the baseline row ({} bytes, {:.4} agreement): {:+} bytes, {:+.4} agreement — {}",
            base.artifact_bytes, base.gate_c_rule12.top1_agreement, db, da, verdict
        );
    } else {
        rationale.push_str("; no baseline row in the grid, so no default comparison");
    }
    Some(Recommendation {
        label: chosen.label.clone(),
        bytes: chosen.artifact_bytes,
        agreement: chosen.gate_c_rule12.top1_agreement,
        slope_floor: KNEE_SLOPE_FLOOR,
        frontier: frontier.iter().map(|r| r.label.clone()).collect(),
        delta_bytes_vs_baseline: delta_bytes,
        delta_agreement_vs_baseline: delta_agreement,
        reconstruction_bits_per_token: chosen.reconstruction.bits_per_token,
        rationale,
    })
}

/// One point's report contributions: its row, the cover-independent TLA3
/// baseline it observed (identical at every point), and its stage timings.
type PointOutput = (SweepRow, GateCMetrics, PointTiming);

/// Worker count for the point loop (#612), from `R4_SWEEP_JOBS` (default `1`
/// = serial, for reproducibility/debugging). Clamped to the point count —
/// more workers than points is wasted — and to at least one. The bound is a
/// memory bound: at most this many points hold their large intermediate
/// tables (cover, transitions, emissions, the ~15 MB artifact) at once.
fn sweep_jobs(point_count: usize) -> usize {
    let requested = std::env::var("R4_SWEEP_JOBS")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok());
    clamp_jobs(requested, point_count)
}

/// Pure worker-count policy (testable without the process-global env): default
/// `1` (serial), then clamp to `[1, point_count]`.
fn clamp_jobs(requested: Option<usize>, point_count: usize) -> usize {
    requested.unwrap_or(1).clamp(1, point_count.max(1))
}

// --------------------------------------------------------- checkpoints ---
// #613: persist each completed point atomically so a long (or parallel)
// sweep can resume after an interruption without recomputing valid points.
// Opt-in: unset `R4_SWEEP_CHECKPOINT_DIR` = today's behavior exactly (no
// files written, no reuse). The final report is still assembled only once
// every point is present (`run_sweep` needs all rows), so a half-finished
// checkpoint directory never yields a report.

/// The compatibility manifest a stored point is reused under. Any change to
/// the inputs, scorer, sampling, skipped arms, schema, or source revision
/// makes a checkpoint ineligible — it is recomputed rather than trusted.
#[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
pub struct CheckpointKey {
    pub schema: u32,
    pub artifact_kappa: String,
    pub corpus_kappa: String,
    /// The fixed #64 scorer knobs that shape every metric.
    pub scorer: String,
    /// Optional campaign/source revision tag (`R4_SWEEP_SOURCE_REV`).
    pub source_revision: String,
    /// Gate C held-out sampling cap (`R4_GATE_C_SAMPLE`); empty = full census.
    pub gate_c_sample: String,
    /// Skipped Gate C arm groups (`R4_GATE_C_SKIP_ARMS`); empty = none.
    pub gate_c_skip_arms: String,
}

/// One point's persisted result plus the manifest and cover identity it was
/// produced under (both re-checked before reuse).
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
struct PointCheckpoint {
    key: CheckpointKey,
    label: String,
    /// The point's cover configuration identity (its `Debug` form; stable per
    /// build and covering every knob, not only the label's k0/gain/budget).
    cover: String,
    row: SweepRow,
    baseline: GateCMetrics,
    timing: PointTiming,
}

/// The checkpoint directory, if resume/persist is enabled for this run.
fn checkpoint_dir() -> Option<PathBuf> {
    std::env::var("R4_SWEEP_CHECKPOINT_DIR")
        .ok()
        .map(|value| PathBuf::from(value.trim()))
        .filter(|path| !path.as_os_str().is_empty())
}

/// The manifest this run's checkpoints are valid under.
fn checkpoint_key(inputs: &SweepInputs, score_config: &ScoreConfig) -> CheckpointKey {
    CheckpointKey {
        schema: SWEEP_REPORT_SCHEMA,
        artifact_kappa: inputs.artifact_kappa.clone(),
        corpus_kappa: inputs.corpus_kappa.clone(),
        scorer: fingerprint_string(
            &inputs.artifact_kappa,
            &inputs.corpus_kappa,
            inputs.train.len(),
            score_config,
        ),
        source_revision: std::env::var("R4_SWEEP_SOURCE_REV").unwrap_or_default(),
        gate_c_sample: std::env::var("R4_GATE_C_SAMPLE").unwrap_or_default(),
        gate_c_skip_arms: std::env::var("R4_GATE_C_SKIP_ARMS").unwrap_or_default(),
    }
}

/// A point's cover-configuration identity (its full `Debug` form).
fn point_cover_identity(point: &SweepPoint) -> String {
    format!("{:?}", point.config)
}

fn checkpoint_path(dir: &Path, index: usize) -> PathBuf {
    dir.join(format!("point-{index:02}.json"))
}

/// Load a compatible checkpoint for `index`, or `None` to (re)compute it.
///
/// Returns `None` on a missing file, an unreadable or partial/corrupt file, or
/// ANY manifest/label/cover mismatch. A stored point is trusted only when it
/// fully deserializes AND its key, label, and cover identity all match the
/// current run — so a partial write (which only ever exists under the `.tmp`
/// name, never the final name) or a stale point can never be read as valid.
fn load_point_checkpoint(
    dir: &Path,
    index: usize,
    key: &CheckpointKey,
    label: &str,
    cover: &str,
) -> Option<PointOutput> {
    let bytes = std::fs::read(checkpoint_path(dir, index)).ok()?;
    let checkpoint: PointCheckpoint = serde_json::from_slice(&bytes).ok()?;
    if &checkpoint.key == key && checkpoint.label == label && checkpoint.cover == cover {
        Some((checkpoint.row, checkpoint.baseline, checkpoint.timing))
    } else {
        None
    }
}

/// Persist a completed point atomically: write the JSON to a `.tmp` sibling,
/// then rename it into place. The rename is atomic on the same filesystem, so
/// a reader only ever sees a complete file — an interrupted write leaves a
/// stray `.tmp` that resume ignores. Best-effort: a persistence failure warns
/// and does not fail the sweep (the point itself was computed correctly).
fn save_point_checkpoint(
    dir: &Path,
    index: usize,
    key: &CheckpointKey,
    point: &SweepPoint,
    cover: &str,
    output: &PointOutput,
) {
    let checkpoint = PointCheckpoint {
        key: key.clone(),
        label: point.label.clone(),
        cover: cover.to_owned(),
        row: output.0.clone(),
        baseline: output.1.clone(),
        timing: output.2.clone(),
    };
    // Best-effort persistence, handled inline: each fallible step warns and
    // returns rather than propagating an error. (The R5 audit forbids a
    // shipped signature that returns an unsanctioned `Result`, so this does
    // not wrap the I/O in a `Result`-typed helper — a persistence failure is
    // not a limitation the model reports; the point was computed correctly.)
    let warn = |what: &str, error: &dyn std::fmt::Display| {
        eprintln!(
            "cover-sweep: WARNING could not persist checkpoint for point {index} ({}): \
             {what}: {error}",
            point.label
        );
    };
    let json = match serde_json::to_vec_pretty(&checkpoint) {
        Ok(json) => json,
        Err(error) => return warn("serialize", &error),
    };
    if let Err(error) = std::fs::create_dir_all(dir) {
        return warn("create dir", &error);
    }
    let tmp_path = dir.join(format!("point-{index:02}.json.tmp"));
    if let Err(error) = std::fs::write(&tmp_path, &json) {
        return warn("write tmp", &error);
    }
    // Atomic rename into place: a reader sees a complete file or nothing.
    if let Err(error) = std::fs::rename(&tmp_path, checkpoint_path(dir, index)) {
        warn("rename", &error);
    }
}

/// Run the sweep points and return their outputs in canonical grid order.
///
/// With `jobs == 1` this is the serial loop (the reproducible default). With
/// `jobs > 1`, `jobs` OS worker threads pull points from a shared cursor and
/// write each result into the point's fixed slot, so the returned vector is
/// always in grid order regardless of completion order — the report and every
/// artifact byte are identical to serial.
///
/// Determinism: each point's INTERNAL parallelism (the compile and Gate C
/// Rayon passes) uses the global Rayon pool exactly as in serial mode, and
/// those passes already reduce in input order, so a point computes the same
/// bytes whether it runs alone or beside others — only the wall-clock overlap
/// changes.
///
/// Oversubscription: the outer workers are OS threads, so their inner Rayon
/// passes share the one global Rayon pool; with `jobs > 1` that pool is
/// oversubscribed. This is bounded and deliberate — the per-point Gate C cost
/// is small since #611, `jobs` is small, and the OS scheduler multiplexes the
/// overlap. Keep `jobs` at or below the core count for the best wall-clock;
/// peak memory is `jobs` concurrent points.
fn run_points_bounded(
    inputs: &SweepInputs,
    configs: &[SweepPoint],
    score_config: &ScoreConfig,
    prepared: &PreparedScoring,
    jobs: usize,
) -> Vec<Option<PointOutput>> {
    let n = configs.len();
    // #613: resume/persist is enabled only when the checkpoint dir is set.
    let checkpoint = checkpoint_dir();
    let key = checkpoint
        .as_ref()
        .map(|_| checkpoint_key(inputs, score_config));
    let run_one = |index: usize| -> Option<PointOutput> {
        let point = &configs[index];
        // #613: reuse a compatible persisted point instead of recomputing.
        if let (Some(dir), Some(key)) = (checkpoint.as_ref(), key.as_ref()) {
            let cover = point_cover_identity(point);
            if let Some(output) = load_point_checkpoint(dir, index, key, &point.label, &cover) {
                eprintln!(
                    "cover-sweep: point {}/{} ({}) resumed from checkpoint",
                    index + 1,
                    n,
                    point.label
                );
                return Some(output);
            }
        }
        eprintln!(
            "cover-sweep: point {}/{} ({})...",
            index + 1,
            n,
            point.label
        );
        let (row, baseline, _bytes, timing) = run_point(inputs, point, score_config, prepared)?;
        eprintln!(
            "cover-sweep: {} regions, {} bytes, Rule 1+2 top-1 {:.4}, {:.4} bits/token; \
             dominant stage {}, point total {} ms",
            row.regions.total,
            row.artifact_bytes,
            row.gate_c_rule12.top1_agreement,
            row.gate_c_rule12.bits_per_token,
            timing.dominant_stage,
            timing.total_ms
        );
        let output = (row, baseline, timing);
        if let (Some(dir), Some(key)) = (checkpoint.as_ref(), key.as_ref()) {
            let cover = point_cover_identity(point);
            save_point_checkpoint(dir, index, key, point, &cover, &output);
        }
        Some(output)
    };

    if jobs <= 1 {
        return (0..n).map(run_one).collect();
    }

    // `jobs` OS workers pull the next point index from a shared cursor and
    // drop the result into that index's slot; grid order is by construction.
    let cursor = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<Option<PointOutput>>>> = (0..n).map(|_| Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..jobs {
            scope.spawn(|| {
                loop {
                    let index = cursor.fetch_add(1, Ordering::Relaxed);
                    if index >= n {
                        break;
                    }
                    let output = run_one(index);
                    *slots[index].lock().expect("sweep slot mutex poisoned") = Some(output);
                }
            });
        }
    });
    slots
        .into_iter()
        .map(|slot| {
            slot.into_inner()
                .expect("sweep slot mutex poisoned")
                .expect("every point slot is filled exactly once")
        })
        .collect()
}

/// Run the full 9-point sweep over the shared inputs with the fixed
/// scorer and assemble the report.
pub fn run_sweep(
    inputs: &SweepInputs,
    score_config: &ScoreConfig,
    distinctiveness_weight: f64,
) -> Option<SweepReport> {
    let points = sweep_grid();
    // #610: build the cover-independent scoring context once, before the
    // point loop, then hand shared references to every point. `sweep_start`
    // begins after this so `total_ms` stays the point-loop wall time and the
    // one-time build is reported separately as `preparation_ms`.
    let prepare_start = Instant::now();
    let prepared = PreparedScoring::prepare(inputs, score_config);
    let preparation_ms = elapsed_ms(prepare_start);
    eprintln!(
        "cover-sweep: cover-independent scoring prepared once in {preparation_ms} ms \
         (context/forward rows + vocab; shared across all {} points)",
        points.len()
    );
    // The per-point configs in canonical grid order (the only per-point
    // mutation is the #364 distinctiveness weight on the COVER config).
    let configs: Vec<SweepPoint> = points
        .iter()
        .map(|point| {
            let mut point = point.clone();
            point
                .config
                .objective
                .weights
                .between_region_distinctiveness = distinctiveness_weight;
            point
        })
        .collect();
    // #612: run independent points with bounded parallelism (default serial).
    // Results come back in grid order and are byte-identical to serial.
    let jobs = sweep_jobs(configs.len());
    if jobs > 1 {
        eprintln!(
            "cover-sweep: evaluating {} points with {jobs} bounded workers \
             (peak memory ~{jobs} concurrent points; set R4_SWEEP_JOBS=1 for serial)",
            configs.len()
        );
    }
    let sweep_start = Instant::now();
    let outputs = run_points_bounded(inputs, &configs, score_config, &prepared, jobs);
    let sweep_total_ms = elapsed_ms(sweep_start);

    // Reduce in grid order: the TLA3 baseline is taken from the first point
    // (identical at every point), exactly as the serial `get_or_insert` did.
    let mut rows = Vec::with_capacity(configs.len());
    let mut timings = Vec::with_capacity(configs.len());
    let mut tla3_baseline: Option<GateCMetrics> = None;
    for output in outputs {
        let (row, baseline_metrics, timing) = output?;
        tla3_baseline.get_or_insert(baseline_metrics);
        rows.push(row);
        timings.push(timing);
    }
    let tla3_baseline = tla3_baseline?;
    let recommendation = recommend(&rows);
    Some(SweepReport {
        schema: SWEEP_REPORT_SCHEMA,
        inputs: SweepReportInputs {
            artifact_kappa: inputs.artifact_kappa.clone(),
            corpus_kappa: inputs.corpus_kappa.clone(),
            train_observations: inputs.train.len(),
            held_out_observations: inputs.held_out.len(),
        },
        scorer: SweepReportScorer {
            transition_out_degree: score_config.transition_out_degree,
            emission_entries: score_config.emission_entries,
            root_top_b: score_config.root_top_b,
            exct_top_x: score_config.exct_top_x,
            witness_sample: score_config.witness_sample,
            smoothing: score_config.smoothing.label(),
            emission_selection: score_config.emission_selection.label(),
            emission_shrinkage: score_config.emission_shrinkage.label(),
            context_order: score_config.context_order,
            context_entries: score_config.context_entries,
        },
        tla3_baseline,
        recommendation,
        points: rows,
        timing: SweepTiming {
            thread_count: jobs,
            gate_c_sample: std::env::var("R4_GATE_C_SAMPLE").ok(),
            preparation_ms,
            train_observations: inputs.train.len(),
            held_out_observations: inputs.held_out.len(),
            total_ms: sweep_total_ms,
            points: timings,
        },
        determinism: "every consumed compiler is deterministic by construction (content-\
                      addressed seeds, ordered reductions, canonical sorts): any single point \
                      run twice produces byte-identical scored artifacts and identical metrics \
                      (a property of the consumed compilers); f64 entropy/ln sites are macOS-pinned \
                      and libm-sensitive cross-platform, the inherited status of the cover and \
                      score compilers (D2 resolves cross-platform byte equality later)"
            .to_owned(),
    })
}

/// The console rate–distortion table, rows ordered by artifact bytes:
/// regions × bytes × Rule 1+2 agreement, with the deepest-depth routing
/// recall and frontier width. The baseline row is marked `*`, the
/// recommended point `<- recommended`.
pub fn render_table(report: &SweepReport) -> String {
    let mut rows: Vec<&SweepRow> = report.points.iter().collect();
    rows.sort_by(|a, b| {
        a.artifact_bytes
            .cmp(&b.artifact_bytes)
            .then_with(|| a.label.cmp(&b.label))
    });
    let recommended = report.recommendation.as_ref().map(|r| r.label.as_str());
    let mut out = String::new();
    let _ = writeln!(
        out,
        "rate-distortion sweep ({} points, fixed #64 scorer, ordered by bytes):",
        rows.len()
    );
    let _ = writeln!(
        out,
        "  {:<34} {:>7} {:>10} {:>10} {:>10} {:>9} {:>9} {:>11}",
        "point", "regions", "bytes", "R1+2 top1", "bits/token", "ref-top1", "ref-topM", "frontier"
    );
    for row in rows {
        let marker = if row.baseline {
            "*"
        } else if Some(row.label.as_str()) == recommended {
            "<"
        } else {
            " "
        };
        let deepest = row.recall.last();
        let (ref1, refm, frontier) = match deepest {
            Some(d) => (
                format!("{:.1}%", 100.0 * d.reference_top1),
                format!("{:.1}%", 100.0 * d.reference_topm),
                format!("{:.2}/{}", d.frontier_mean, d.frontier_max),
            ),
            None => ("-".to_owned(), "-".to_owned(), "-".to_owned()),
        };
        let _ = writeln!(
            out,
            "{} {:<34} {:>7} {:>10} {:>9.1}% {:>10.4} {:>9} {:>9} {:>11}",
            marker,
            row.label,
            row.regions.total,
            row.artifact_bytes,
            100.0 * row.gate_c_rule12.top1_agreement,
            row.gate_c_rule12.bits_per_token,
            ref1,
            refm,
            frontier
        );
    }
    let _ = writeln!(
        out,
        "  (* = default operating point; ref/frontier columns at the deepest depth; \
         TLA3 store baseline: {:.1}% top-1, {:.4} bits/token)",
        100.0 * report.tla3_baseline.top1_agreement,
        report.tla3_baseline.bits_per_token
    );
    if let Some(rec) = &report.recommendation {
        let _ = writeln!(out, "recommendation: {}", rec.rationale);
    }
    out
}

// ------------------------------------------------------------ CLI --------

#[derive(Debug, PartialEq)]
struct CoverSweepOptions {
    corpus_meta: PathBuf,
    corpus_recs: PathBuf,
    artifacts: PathBuf,
    output: PathBuf,
    emission_selection: score::EmissionSelection,
    emission_shrinkage: score::EmissionShrinkage,
    distinctiveness_weight: f64,
}

fn parse_cover_sweep_options(args: &[String]) -> Result<CoverSweepOptions, SourceUnavailable> {
    let (default_meta, default_recs) = compiler::corpus_paths();
    let mut options = CoverSweepOptions {
        corpus_meta: PathBuf::from(default_meta),
        corpus_recs: PathBuf::from(default_recs),
        artifacts: PathBuf::from(compiler::ART_PATH),
        output: PathBuf::from("cover_sweep"),
        emission_selection: score::EmissionSelection::default(),
        emission_shrinkage: score::EmissionShrinkage::default(),
        distinctiveness_weight: 0.0,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| SourceUnavailable::new(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--corpus-meta" => options.corpus_meta = PathBuf::from(value),
            "--corpus-recs" => options.corpus_recs = PathBuf::from(value),
            "--artifacts" => options.artifacts = PathBuf::from(value),
            "--out" => options.output = PathBuf::from(value),
            "--emission-selection" => {
                options.emission_selection = match value.as_str() {
                    "ratio" => score::EmissionSelection::Ratio,
                    "probability" => score::EmissionSelection::Probability,
                    other => {
                        return Err(SourceUnavailable::new(format!(
                            "invalid --emission-selection value: {other} \
                             (expected ratio|probability)"
                        )));
                    }
                };
            }
            "--emission-shrinkage" => {
                options.emission_shrinkage = match value.as_str() {
                    "none" => score::EmissionShrinkage::None,
                    "witten-bell" => score::EmissionShrinkage::WittenBell,
                    "contrast" => score::EmissionShrinkage::Contrast,
                    other => {
                        return Err(SourceUnavailable::new(format!(
                            "invalid --emission-shrinkage value: {other} \
                             (expected none|witten-bell|contrast)"
                        )));
                    }
                };
            }
            "--distinctiveness-weight" => {
                let weight = value.parse::<f64>().map_err(|error| {
                    SourceUnavailable::new(format!(
                        "invalid --distinctiveness-weight value {value}: {error}"
                    ))
                })?;
                if !weight.is_finite() || weight < 0.0 {
                    return Err(SourceUnavailable::new(format!(
                        "invalid --distinctiveness-weight value {value} (expected finite non-negative number)"
                    )));
                }
                options.distinctiveness_weight = weight;
            }
            _ => {
                return Err(SourceUnavailable::new(format!(
                    "unknown cover-sweep option: {flag}"
                )));
            }
        }
        index += 2;
    }
    Ok(options)
}

/// Cover fineness sweep (issue #70, module docs): run the 9-point
/// rate–distortion grid under the fixed scorer and write
/// `cover_sweep.json` plus the console table. `--distinctiveness-weight`
/// sets the optional induction-time between-region contrast reward.
/// Release-mode workload on the fixture corpus.
pub fn cover_sweep_command(args: &[String]) -> Result<(), SourceUnavailable> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make the sweep much slower; use `cargo run --release -- transformerless cover-sweep ...`"
    );
    let options = parse_cover_sweep_options(args)?;
    let inputs = load_inputs(
        &options.corpus_meta,
        &options.corpus_recs,
        &options.artifacts,
    )?;
    let score_config = ScoreConfig {
        emission_selection: options.emission_selection,
        emission_shrinkage: options.emission_shrinkage,
        ..ScoreConfig::default()
    };
    eprintln!(
        "cover-sweep: {} train / {} held-out observations; running the 9-point grid (fixed scorer, \
         emission {}/{}, cover distinctiveness {})...",
        inputs.train.len(),
        inputs.held_out.len(),
        score_config.emission_selection.label(),
        score_config.emission_shrinkage.label(),
        options.distinctiveness_weight
    );
    let report =
        run_sweep(&inputs, &score_config, options.distinctiveness_weight).ok_or_else(|| {
            SourceUnavailable::new(
                "cover sweep produced no report: degenerate corpus \
                 (empty train/held-out split or empty grid)",
            )
        })?;

    std::fs::create_dir_all(&options.output)?;
    let report_json = serde_json::to_string_pretty(&report).map_err(SourceUnavailable::from)?;
    let report_path = options.output.join("cover_sweep.json");
    std::fs::write(&report_path, &report_json)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", report_path.display())))?;

    print!("{}", render_table(&report));
    println!("  report: {}", report_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults_and_overrides() {
        let options = parse_cover_sweep_options(&[]).expect("defaults");
        let (default_meta, default_recs) = compiler::corpus_paths();
        assert_eq!(options.corpus_meta, PathBuf::from(default_meta));
        assert_eq!(options.corpus_recs, PathBuf::from(default_recs));
        assert_eq!(options.artifacts, PathBuf::from(compiler::ART_PATH));
        assert_eq!(options.output, PathBuf::from("cover_sweep"));
        assert_eq!(
            options.emission_selection,
            score::EmissionSelection::default()
        );
        assert_eq!(
            options.emission_shrinkage,
            score::EmissionShrinkage::default()
        );
        assert_eq!(options.distinctiveness_weight, 0.0);

        let args = [
            "--corpus-meta",
            "/tmp/m.bin",
            "--corpus-recs",
            "/tmp/r.bin",
            "--artifacts",
            "/tmp/a.bin",
            "--out",
            "/tmp/sweep",
            "--emission-selection",
            "probability",
            "--emission-shrinkage",
            "contrast",
            "--distinctiveness-weight",
            "1.5",
        ]
        .map(str::to_owned);
        let options = parse_cover_sweep_options(&args).expect("valid options");
        assert_eq!(options.corpus_meta, PathBuf::from("/tmp/m.bin"));
        assert_eq!(options.corpus_recs, PathBuf::from("/tmp/r.bin"));
        assert_eq!(options.artifacts, PathBuf::from("/tmp/a.bin"));
        assert_eq!(options.output, PathBuf::from("/tmp/sweep"));
        assert_eq!(
            options.emission_selection,
            score::EmissionSelection::Probability
        );
        assert_eq!(
            options.emission_shrinkage,
            score::EmissionShrinkage::Contrast
        );
        assert_eq!(options.distinctiveness_weight, 1.5);

        let bad_shrinkage = ["--emission-shrinkage", "sideways"].map(str::to_owned);
        assert!(parse_cover_sweep_options(&bad_shrinkage).is_err());
        let bad_distinctiveness = ["--distinctiveness-weight", "-1"].map(str::to_owned);
        assert!(parse_cover_sweep_options(&bad_distinctiveness).is_err());

        let bad = ["--k0", "16"].map(str::to_owned);
        assert!(parse_cover_sweep_options(&bad).is_err());
        let missing = ["--out"].map(str::to_owned);
        assert!(parse_cover_sweep_options(&missing).is_err());
    }

    #[test]
    fn sweep_row_serializes_reconstruction_block() {
        let metrics = |bits_per_token: f64| GateCMetrics {
            positions: 10,
            top1_agreement: 0.5,
            bits_per_token,
            ..GateCMetrics::default()
        };
        let row = SweepRow {
            label: "k0=8/gain=0.25/budget=128".to_owned(),
            baseline: false,
            config: SweepRowConfig {
                k0: 8,
                depths: 3,
                entropy_gain_bits: 0.25,
                regions_budget: 128,
                min_support: 4,
                memory_budget_bytes: 64 * 1024 * 1024,
                distinctiveness_weight: 0.0,
            },
            regions: SweepRegions {
                total: 1,
                per_depth: vec![1],
                splits: 0,
                max_depth: 1,
            },
            recall: Vec::new(),
            artifact_bytes: 1024,
            graph_kappa: "blake3:test".to_owned(),
            gate_c_rule12: metrics(1.0),
            reconstruction: metrics(16.03),
        };
        let json = serde_json::to_value(&row).expect("row serializes");
        assert_eq!(json["gate_c_rule12"]["positions"], 10);
        assert_eq!(json["gate_c_rule12"]["bits_per_token"], 1.0);
        let reconstruction = &json["reconstruction"];
        assert_eq!(reconstruction["positions"], 10);
        assert_eq!(reconstruction["top1_agreement"], 0.5);
        assert_eq!(reconstruction["bits_per_token"], 16.03);

        let recommendation = Recommendation {
            label: row.label.clone(),
            bytes: row.artifact_bytes,
            agreement: 0.5,
            slope_floor: KNEE_SLOPE_FLOOR,
            frontier: vec![row.label.clone()],
            delta_bytes_vs_baseline: Some(0),
            delta_agreement_vs_baseline: Some(0.0),
            reconstruction_bits_per_token: 16.03,
            rationale: "test".to_owned(),
        };
        let json = serde_json::to_value(&recommendation).expect("recommendation serializes");
        assert_eq!(json["reconstruction_bits_per_token"], 16.03);
        assert_eq!(SWEEP_REPORT_SCHEMA, 5);
    }

    #[test]
    fn point_timing_reports_dominant_stage_and_serializes() {
        // gate_c (100 ms) dominates; total is the sum of the stage times.
        let timing = PointTiming::new(
            "k0=8/gain=0.25/budget=128".to_owned(),
            10,
            20,
            5,
            3,
            40,
            7,
            100,
        );
        assert_eq!(timing.dominant_stage, "gate_c");
        assert_eq!(timing.total_ms, 10 + 20 + 5 + 3 + 40 + 7 + 100);

        let json = serde_json::to_value(&timing).expect("point timing serializes");
        assert_eq!(json["label"], "k0=8/gain=0.25/budget=128");
        assert_eq!(json["cover_induction_ms"], 10);
        assert_eq!(json["recall_and_edges_ms"], 20);
        assert_eq!(json["transitions_ms"], 5);
        assert_eq!(json["context_and_forward_rows_ms"], 3);
        assert_eq!(json["emissions_ms"], 40);
        assert_eq!(json["r4g1_emission_ms"], 7);
        assert_eq!(json["gate_c_ms"], 100);
        assert_eq!(json["total_ms"], 185);
        assert_eq!(json["dominant_stage"], "gate_c");

        let sweep = SweepTiming {
            thread_count: 1,
            gate_c_sample: Some("2000".to_owned()),
            preparation_ms: 12,
            train_observations: 500,
            held_out_observations: 100,
            total_ms: 185,
            points: vec![timing],
        };
        let json = serde_json::to_value(&sweep).expect("sweep timing serializes");
        assert_eq!(json["thread_count"], 1);
        assert_eq!(json["gate_c_sample"], "2000");
        assert_eq!(json["preparation_ms"], 12);
        assert_eq!(json["train_observations"], 500);
        assert_eq!(json["held_out_observations"], 100);
        assert_eq!(json["points"][0]["dominant_stage"], "gate_c");
    }

    #[test]
    fn point_timing_dominant_stage_breaks_ties_to_the_first_stage() {
        // Equal maxima resolve to the first stage in the fixed order, so
        // the dominant label is deterministic given the timings.
        let timing = PointTiming::new("t".to_owned(), 5, 5, 0, 0, 0, 0, 0);
        assert_eq!(timing.dominant_stage, "cover_induction");
    }

    // #610: the PreparedScoring reuse guard. The fingerprint must change
    // whenever any input the cover-independent context depends on changes —
    // otherwise a shared context could be silently applied to an
    // incompatible corpus/artifact/scorer and score a point against the
    // wrong tables. This runs in CI without fixtures (pure string logic).
    #[test]
    fn fingerprint_distinguishes_inputs_and_scorer() {
        let cfg = ScoreConfig::default();
        let base = fingerprint_string("art-A", "corpus-B", 100, &cfg);

        // Stable: identical inputs yield an identical fingerprint.
        assert_eq!(base, fingerprint_string("art-A", "corpus-B", 100, &cfg));

        // Any input-identity change must move the fingerprint.
        assert_ne!(base, fingerprint_string("art-X", "corpus-B", 100, &cfg));
        assert_ne!(base, fingerprint_string("art-A", "corpus-X", 100, &cfg));
        assert_ne!(base, fingerprint_string("art-A", "corpus-B", 101, &cfg));

        // A scorer knob that feeds the context rows must move it too.
        let other = ScoreConfig {
            context_order: cfg.context_order + 1,
            ..ScoreConfig::default()
        };
        assert_ne!(base, fingerprint_string("art-A", "corpus-B", 100, &other));
    }

    // #612: the worker-count policy. Default is serial; a request is clamped to
    // [1, point_count] so a point never has more workers than there are points
    // and an out-of-range request cannot drive unbounded parallelism.
    #[test]
    fn clamp_jobs_defaults_to_serial_and_bounds_the_request() {
        // Unset / unparseable -> serial.
        assert_eq!(clamp_jobs(None, 9), 1);
        // In range -> honored.
        assert_eq!(clamp_jobs(Some(2), 9), 2);
        assert_eq!(clamp_jobs(Some(9), 9), 9);
        // Zero or above the point count -> clamped into [1, point_count].
        assert_eq!(clamp_jobs(Some(0), 9), 1);
        assert_eq!(clamp_jobs(Some(64), 9), 9);
        // Degenerate point counts stay at one worker.
        assert_eq!(clamp_jobs(Some(4), 0), 1);
        assert_eq!(clamp_jobs(Some(4), 1), 1);
    }

    fn sample_row() -> SweepRow {
        let metrics = |bits: f64| GateCMetrics {
            positions: 10,
            top1_agreement: 0.5,
            bits_per_token: bits,
            ..GateCMetrics::default()
        };
        SweepRow {
            label: "k0=8/gain=0.25/budget=128".to_owned(),
            baseline: false,
            config: SweepRowConfig {
                k0: 8,
                depths: 3,
                entropy_gain_bits: 0.25,
                regions_budget: 128,
                min_support: 4,
                memory_budget_bytes: 64 * 1024 * 1024,
                distinctiveness_weight: 0.0,
            },
            regions: SweepRegions {
                total: 1,
                per_depth: vec![1],
                splits: 0,
                max_depth: 1,
            },
            recall: Vec::new(),
            artifact_bytes: 1024,
            graph_kappa: "blake3:test".to_owned(),
            gate_c_rule12: metrics(1.0),
            reconstruction: metrics(16.03),
        }
    }

    // #613: a persisted point is reused ONLY under a fully matching manifest,
    // label, and cover identity, and a partial/corrupt/interrupted write is
    // never mistaken for a completed result — the guarantees the resume path
    // rests on. Runs in CI (no fixtures): a temp dir plus round-trip.
    #[test]
    fn checkpoint_reused_only_under_a_matching_manifest() {
        let dir = std::env::temp_dir().join("uor_r4_i613_ckpt_test");
        let _ = std::fs::remove_dir_all(&dir);

        let key = CheckpointKey {
            schema: SWEEP_REPORT_SCHEMA,
            artifact_kappa: "art-A".to_owned(),
            corpus_kappa: "corpus-B".to_owned(),
            scorer: "scorer-1".to_owned(),
            source_revision: "rev-1".to_owned(),
            gate_c_sample: String::new(),
            gate_c_skip_arms: String::new(),
        };
        let point = sweep_grid().into_iter().next().expect("grid has points");
        let cover = point_cover_identity(&point);
        let output: PointOutput = (
            sample_row(),
            GateCMetrics::default(),
            PointTiming::new(point.label.clone(), 1, 2, 3, 4, 5, 6, 7),
        );

        // No file yet -> recompute.
        assert!(load_point_checkpoint(&dir, 0, &key, &point.label, &cover).is_none());

        // Persist atomically, then a fully-matching load returns it.
        save_point_checkpoint(&dir, 0, &key, &point, &cover, &output);
        let loaded = load_point_checkpoint(&dir, 0, &key, &point.label, &cover)
            .expect("matching checkpoint is reused");
        // Bit-exact round-trip: a reused row serializes byte-for-byte the same
        // as the persisted one, so a resumed report equals a clean one. This
        // relies on serde_json's `float_roundtrip` feature — without it the
        // default parser lands ~1 ULP off and the assertion below fails.
        assert_eq!(
            serde_json::to_string(&loaded.0).unwrap(),
            serde_json::to_string(&output.0).unwrap(),
            "reused row must serialize identically to the persisted row"
        );
        assert_eq!(
            loaded.0.reconstruction.bits_per_token.to_bits(),
            output.0.reconstruction.bits_per_token.to_bits(),
            "a metric must survive the checkpoint round-trip bit-for-bit"
        );

        // Any manifest / label / cover mismatch -> recompute (not reused).
        let mismatched_key = CheckpointKey {
            scorer: "scorer-2".to_owned(),
            ..key.clone()
        };
        assert!(load_point_checkpoint(&dir, 0, &mismatched_key, &point.label, &cover).is_none());
        assert!(load_point_checkpoint(&dir, 0, &key, "other-label", &cover).is_none());
        assert!(load_point_checkpoint(&dir, 0, &key, &point.label, "other-cover").is_none());

        // A corrupt/partial FINAL file is never read as a result.
        std::fs::write(checkpoint_path(&dir, 1), b"{ not complete json").unwrap();
        assert!(load_point_checkpoint(&dir, 1, &key, &point.label, &cover).is_none());
        // A stray `.tmp` from an interrupted write is ignored: no final file.
        std::fs::write(dir.join("point-02.json.tmp"), b"{}").unwrap();
        assert!(load_point_checkpoint(&dir, 2, &key, &point.label, &cover).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
