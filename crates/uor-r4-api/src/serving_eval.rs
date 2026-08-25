//! Serving-surface held-out evaluation — the certify C row (issue #280).
//!
//! Measures the configuration that actually serves: token-free D4 policy
//! resolution composed with [`uor_r4_graph_runtime::R4G1Runtime`] candidate
//! selection, fed the same token windows as the HTTP server. The evaluator is
//! teacher-free: it consumes recorded `t_argmax` labels and the same-position
//! plain TLA comparator.
//!
//! The previous C row measured the `convert_r4g1` certify scaffold,
//! which was never a functional prediction path (issue #280 diagnosis:
//! no per-node emissions, kind=2 transition walks structurally absent,
//! stride-8 EXCT decode over a variable-stride container). That row is
//! retired; the recorded 0.0% stands in issue #280 as the reason.
//!
//! Discipline carried over from the retired row (#279/#282/#232):
//! deterministic nested story-distributed sample, wall-clock budgets that
//! turn silent stalls into recorded skips, and a readiness probe extended with an
//! accuracy spot-check — scored + non-constant demonstrably does not
//! imply functional, so the probe now requires at least one served
//! prediction that matches the recorded corpus/teacher continuation
//! before the full sample is spent.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde::Serialize;
use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_compiler::induction;
use uor_r4_model_source::SourceUnavailable;

use crate::deployed_quality::{
    deployed_quality_positions_cid, derive_deployed_quality_bindings, deterministic_story_sample,
    ComparatorIdentity, DeployedQualityBindingMaterial, DeployedQualityReport, EvaluationEvidence,
    EvaluationMode, ExactRate, ExactSignedRate, NegativeControlEvidence, NegativeControlVerdict,
    PairedComparison, PairedCounts as QualityPairedCounts, PairedInterval, QualityMeasurements,
    QualityProfileIdentity, QualityVerdict, WitnessReplayEvidence, DEPLOYED_QUALITY_PROFILE_ID,
    DEPLOYED_QUALITY_PROFILE_VERSION, DEPLOYED_QUALITY_REPORT_SCHEMA,
    DETERMINISTIC_SAMPLE_SELECTION_ALGORITHM, FULL_POPULATION_SELECTION_ALGORITHM,
    LABEL_SHUFFLED_CONTROL_ID, NORMATIVE_EXECUTION_SCOPE, RF31_MIN_LANE_DELTA_PPM,
    SECTIONS_ABSENT_COMPARATOR_ID, SECTIONS_ABSENT_COMPARATOR_VERSION, TLA_COMPARATOR_ID,
    TLA_COMPARATOR_VERSION,
};
use crate::engine::{EngineParts, PolicyStatus};
use crate::serving::{
    CrossSurfaceParityEvidence, NormativeServingDecision, NormativeServingEngine,
};
use crate::witness_replay::{
    parse_and_validate_normative_witness_replay, NormativeWitnessReplayArtifact,
    NormativeWitnessReplayMaterial, NormativeWitnessReplaySpec, DEFAULT_NORMATIVE_WITNESS_SAMPLE,
};
use uor_r4_graph_runtime::{ServedCandidateSource, ServedCandidates};

/// Default size of the predeclared nested story-distributed instrument.
pub const SAMPLE_TARGET: usize = 6000;

/// Predeclared non-census extension used when the 6,000-position screen is
/// statistically inconclusive but neither bound rules the mechanism out.
pub const EXTENDED_SAMPLE_TARGET: usize = 18_000;

/// Probe positions spent on the readiness/accuracy spot-check before
/// the full sample runs (#232 and its #280 extension).
pub const PROBE_POSITIONS: usize = 64;

const PROBE_PROGRESS_INTERVAL: usize = 16;
const EVAL_PROGRESS_INTERVAL: usize = 256;
const LABEL_SHUFFLED_CONTROL_VERSION: &str = "train-target-rotation-half-plus-one/1";
const REPORT_TERMINAL_SCHEMA: &str = "uor-r4-deployed-quality-terminal/2";
const REPORT_PROGRESS_SCHEMA: &str = "uor-r4-deployed-quality-progress/1";

/// Deterministic evaluation extent. A sample can guide whether a census is
/// worth running but can never authorize production admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ServingEvalMode {
    Sample { positions: usize },
    FullCensus,
}

/// What to evaluate: a compiled bundle directory holding the serving
/// artifacts. `graph/score.r4g1` is preferred, root `score.r4g1`
/// accepted; `score_report.json` is read from the graph's directory;
/// the teacher artifact and corpus files sit at the bundle root.
#[derive(Debug, Clone)]
pub struct ServingBundle {
    pub root: PathBuf,
    pub graph: PathBuf,
    /// Diagnostic-only pre-re-emission canonical graph. When present, it is
    /// evaluated beside the newly emitted sections-absent graph to expose
    /// compiler/artifact/selector drift. It never enters production admission.
    pub canonical_base_graph: Option<PathBuf>,
    /// Planted causal control: the same graph generation with SKMX/PSIB
    /// omitted. Absence keeps research evaluation available but makes
    /// production quality evidence UNAVAILABLE.
    pub sections_absent_graph: Option<PathBuf>,
    /// Planted negative control fitted after rotating TRAIN targets by
    /// `n/2 + 1`, then evaluated against pristine held-out labels.
    pub label_shuffled_graph: Option<PathBuf>,
    pub teacher: PathBuf,
    pub store: PathBuf,
    pub corpus_meta: PathBuf,
    pub corpus_records: PathBuf,
    pub tokenizer: Option<PathBuf>,
    pub tokenizer_adapter: Option<PathBuf>,
    pub score_report: Option<PathBuf>,
    pub compile_report: Option<PathBuf>,
}

/// One immutable, content-addressed capture of every bundle component used by
/// deployed-quality measurement or report construction.
///
/// Capturing once prevents a filesystem generation change from making the
/// measured row describe one set of bytes while the report binds another.
/// Wall-clock paths are retained only for diagnostics; [`Self::generation_cid`]
/// commits to component names, presence, and bytes, not their locations.
pub struct ServingBundleSnapshot {
    bundle: ServingBundle,
    graph: Vec<u8>,
    canonical_base_graph: Option<Vec<u8>>,
    sections_absent_graph: Option<Vec<u8>>,
    label_shuffled_graph: Option<Vec<u8>>,
    teacher: Vec<u8>,
    store: Vec<u8>,
    corpus_meta: Vec<u8>,
    corpus_records: Vec<u8>,
    tokenizer: Option<Vec<u8>>,
    tokenizer_adapter: Option<Vec<u8>>,
    score_report: Option<Vec<u8>>,
    compile_report: Option<Vec<u8>>,
    generation_cid: String,
}

impl ServingBundleSnapshot {
    /// Capture a complete generation exactly once. A path discovered as
    /// present but unreadable is UNAVAILABLE; it is never silently downgraded
    /// to an absent optional component.
    pub fn capture(bundle: &ServingBundle) -> Result<Self, SourceUnavailable> {
        let read = |field: &str, path: &Path| {
            std::fs::read(path).map_err(|error| {
                SourceUnavailable::new(format!(
                    "capture deployed-quality {field} {}: {error}",
                    path.display()
                ))
            })
        };
        let read_optional =
            |field: &str, path: Option<&Path>| path.map(|path| read(field, path)).transpose();

        let graph = read("graph", &bundle.graph)?;
        let canonical_base_graph = read_optional(
            "pre-re-emission canonical diagnostic",
            bundle.canonical_base_graph.as_deref(),
        )?;
        let sections_absent_graph = read_optional(
            "sections-absent control",
            bundle.sections_absent_graph.as_deref(),
        )?;
        let label_shuffled_graph = read_optional(
            "label-shuffled control",
            bundle.label_shuffled_graph.as_deref(),
        )?;
        let teacher = read("teacher artifact", &bundle.teacher)?;
        let store = read("TLA store", &bundle.store)?;
        let corpus_meta = read("corpus metadata", &bundle.corpus_meta)?;
        let corpus_records = read("corpus records", &bundle.corpus_records)?;
        let tokenizer = read_optional("tokenizer", bundle.tokenizer.as_deref())?;
        let tokenizer_adapter =
            read_optional("tokenizer adapter", bundle.tokenizer_adapter.as_deref())?;
        let score_report = read_optional("score report", bundle.score_report.as_deref())?;
        let compile_report = read_optional("compile report", bundle.compile_report.as_deref())?;
        let generation_cid = serving_generation_cid(&[
            ("graph", Some(graph.as_slice())),
            ("canonical_base_graph", canonical_base_graph.as_deref()),
            ("sections_absent_graph", sections_absent_graph.as_deref()),
            ("label_shuffled_graph", label_shuffled_graph.as_deref()),
            ("teacher", Some(teacher.as_slice())),
            ("store", Some(store.as_slice())),
            ("corpus_meta", Some(corpus_meta.as_slice())),
            ("corpus_records", Some(corpus_records.as_slice())),
            ("tokenizer", tokenizer.as_deref()),
            ("tokenizer_adapter", tokenizer_adapter.as_deref()),
            ("score_report", score_report.as_deref()),
            ("compile_report", compile_report.as_deref()),
        ]);

        Ok(Self {
            bundle: bundle.clone(),
            graph,
            canonical_base_graph,
            sections_absent_graph,
            label_shuffled_graph,
            teacher,
            store,
            corpus_meta,
            corpus_records,
            tokenizer,
            tokenizer_adapter,
            score_report,
            compile_report,
            generation_cid,
        })
    }

    pub fn generation_cid(&self) -> &str {
        &self.generation_cid
    }

    /// Refuse to continue an expensive phase when its fresh capture no longer
    /// matches the generation which passed the binding sample.
    pub fn require_generation(&self, expected: &str) -> Result<(), SourceUnavailable> {
        if self.generation_cid == expected {
            Ok(())
        } else {
            Err(SourceUnavailable::new(format!(
                "binding sample measured generation {expected}, but the fresh full-census capture is {}; full census was not launched",
                self.generation_cid
            )))
        }
    }

    pub fn graph(&self) -> &[u8] {
        &self.graph
    }

    pub fn signature_artifact(&self) -> &[u8] {
        &self.teacher
    }

    pub fn corpus_meta(&self) -> &[u8] {
        &self.corpus_meta
    }

    pub fn corpus_records(&self) -> &[u8] {
        &self.corpus_records
    }

    pub fn tokenizer(&self) -> Option<&[u8]> {
        self.tokenizer.as_deref()
    }

    pub fn score_report(&self) -> Option<&[u8]> {
        self.score_report.as_deref()
    }
}

/// Wall-clock budgets. Defaults match the retired C row's env contract
/// (`R4_CERTIFY_R4G1_BUDGET_SECS` / `R4_CERTIFY_R4G1_EVAL_BUDGET_SECS`
/// still override, so existing run scripts keep working).
#[derive(Debug, Clone, Copy)]
pub struct ServingEvalBudgets {
    pub probe: Duration,
    pub eval: Duration,
    pub mode: ServingEvalMode,
    pub workers: usize,
}

impl ServingEvalBudgets {
    /// Defaults (probe 120s, eval 600s) with the historical env overrides.
    pub fn from_env() -> Self {
        let secs = |name: &str, default: u64| {
            std::env::var(name)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };
        Self {
            probe: Duration::from_secs(secs("R4_CERTIFY_R4G1_BUDGET_SECS", 120)),
            eval: Duration::from_secs(secs("R4_CERTIFY_R4G1_EVAL_BUDGET_SECS", 600)),
            mode: match std::env::var("R4_DEPLOYED_QUALITY_MODE").as_deref() {
                Ok("full") => ServingEvalMode::FullCensus,
                _ => ServingEvalMode::Sample {
                    positions: std::env::var("R4_DEPLOYED_QUALITY_POSITIONS")
                        .ok()
                        .and_then(|value| value.parse().ok())
                        .filter(|&value| value > 0)
                        .unwrap_or(SAMPLE_TARGET),
                },
            },
            workers: std::env::var("R4_DEPLOYED_QUALITY_WORKERS")
                .ok()
                .and_then(|value| value.parse().ok())
                .filter(|&value| value > 0)
                .unwrap_or_else(|| {
                    std::thread::available_parallelism()
                        .map(usize::from)
                        .unwrap_or(1)
                }),
        }
    }
}

/// Per-status decision counts (D4 policy vocabulary). Used for both
/// served and abstained positions — issue #234 item 3: every
/// evaluation run reports the count of held-out probes resolved at
/// each `ResolutionStatus` level, so blended headline numbers cannot
/// hide an exact-context-only distribution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct StatusBreakdown {
    pub exact_context: u64,
    pub graph: u64,
    pub novel: u64,
    pub contradictory: u64,
    /// Of `exact_context`, how many resolved via an explicit NGRAM
    /// context row rather than the EXCT probe (#362 attribution — the
    /// two mechanisms share the `ExactContext` status since e77b1d4,
    /// so era comparisons need the split).
    pub exact_context_ngram: u64,
}

/// Exact paired 2x2 counts against one same-position comparator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct PairedCounts {
    pub both: u64,
    pub normative_only: u64,
    pub comparator_only: u64,
    pub neither: u64,
}

impl PairedCounts {
    fn record(&mut self, normative_hit: bool, comparator_hit: bool) {
        match (normative_hit, comparator_hit) {
            (true, true) => self.both += 1,
            (true, false) => self.normative_only += 1,
            (false, true) => self.comparator_only += 1,
            (false, false) => self.neither += 1,
        }
    }

    pub fn total(&self) -> u64 {
        self.both + self.normative_only + self.comparator_only + self.neither
    }
}

/// Post-selection policy disposition used by the diagnostic attribution
/// table. Candidate selection has already completed before this value or any
/// teacher target is inspected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttributionDisposition {
    Served,
    Abstained,
    Declined,
}

/// Stable wire vocabulary for D4 status attribution. `Unavailable` is used
/// only when D4 declined before a policy status existed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttributionStatus {
    ExactContext,
    Graph,
    Novel,
    Contradictory,
    Unavailable,
}

impl From<Option<PolicyStatus>> for AttributionStatus {
    fn from(status: Option<PolicyStatus>) -> Self {
        match status {
            Some(PolicyStatus::ExactContext) => Self::ExactContext,
            Some(PolicyStatus::Graph) => Self::Graph,
            Some(PolicyStatus::Novel) => Self::Novel,
            Some(PolicyStatus::Contradictory) => Self::Contradictory,
            None => Self::Unavailable,
        }
    }
}

/// Same-position normative/TLA correctness cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NormativeTlaCell {
    BothCorrect,
    NormativeOnlyCorrect,
    TlaOnlyCorrect,
    NeitherCorrect,
}

/// Runtime-owned source of the recorded teacher target when it is present in
/// the post-lane normative shortlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttributionCandidateSource {
    Base,
    Skipmix,
}

impl From<ServedCandidateSource> for AttributionCandidateSource {
    fn from(source: ServedCandidateSource) -> Self {
        match source {
            ServedCandidateSource::Base => Self::Base,
            ServedCandidateSource::Skipmix => Self::Skipmix,
        }
    }
}

/// Exact effect of the learned lane on teacher-target correctness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LaneTransition {
    Unchanged,
    TowardTarget,
    AwayFromTarget,
    ChangedOther,
}

/// Cross-tab dimensions for one evaluated position. `*_target_rank` is
/// one-based and `None` means absent only when the corresponding
/// `*_candidates_evaluated` flag is true. This distinguishes absence from an
/// abstained/declined or unavailable-control decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DecisionAttributionDimensions {
    pub status: AttributionStatus,
    pub disposition: AttributionDisposition,
    pub normative_tla_cell: NormativeTlaCell,
    /// Candidate rank under the pre-re-emission canonical graph. This arm is
    /// diagnostic only and cannot contribute to an admission comparison.
    pub canonical_base_candidates_evaluated: bool,
    pub canonical_base_target_rank: Option<u8>,
    pub normative_candidates_evaluated: bool,
    pub normative_target_rank: Option<u8>,
    pub sections_absent_candidates_evaluated: bool,
    pub sections_absent_target_rank: Option<u8>,
    pub target_source: Option<AttributionCandidateSource>,
    pub target_skmx_contributed: bool,
    pub target_psib_contributed: bool,
    pub lane_transition: LaneTransition,
}

/// One post-hoc position record. The target is used only to construct this
/// record after main and control predictions have returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PositionDecisionAttribution {
    pub position: u64,
    pub dimensions: DecisionAttributionDimensions,
}

/// Canonically ordered run-length reduction of identical attribution
/// dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DecisionAttributionCell {
    pub dimensions: DecisionAttributionDimensions,
    pub count: u64,
}

/// Deterministic per-position evidence and its independently reproducible
/// aggregate. Both are emitted because aggregates guide the next experiment,
/// while position rows make every bucket falsifiable without rerunning.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DecisionAttributionEvidence {
    pub positions: Vec<PositionDecisionAttribution>,
    pub cells: Vec<DecisionAttributionCell>,
}

impl DecisionAttributionEvidence {
    fn record(&mut self, position: PositionDecisionAttribution) {
        self.positions.push(position);
    }

    fn finalize(&mut self) {
        let mut dimensions: Vec<_> = self
            .positions
            .iter()
            .map(|position| position.dimensions)
            .collect();
        dimensions.sort_unstable();
        self.cells.clear();
        for dimensions in dimensions {
            if let Some(last) = self
                .cells
                .last_mut()
                .filter(|last| last.dimensions == dimensions)
            {
                last.count += 1;
            } else {
                self.cells.push(DecisionAttributionCell {
                    dimensions,
                    count: 1,
                });
            }
        }
    }

    /// Stable bytes retained inside the atomic terminal artifact and bound by
    /// [`Self::cid`].
    pub fn deterministic_json_bytes(&self) -> Result<Vec<u8>, SourceUnavailable> {
        serde_json::to_vec(self).map_err(|error| {
            SourceUnavailable::new(format!("serialize decision attribution: {error}"))
        })
    }

    /// Content identity of the complete typed attribution evidence, its
    /// immutable bundle generation, and exact evaluated-position population.
    pub fn cid(
        &self,
        generation_cid: &str,
        evaluated_positions_cid: &str,
    ) -> Result<String, SourceUnavailable> {
        let bytes = self.deterministic_json_bytes()?;
        Ok(report_tagged_cid(
            b"r4-serving-decision-attribution/1",
            &[
                generation_cid.as_bytes(),
                evaluated_positions_cid.as_bytes(),
                &bytes,
            ],
        ))
    }

    fn validate(
        &self,
        expected_positions: &[usize],
        normative_vs_tla: PairedCounts,
        lane_changed: u64,
        lane_toward: u64,
        lane_away: u64,
    ) -> Result<(), SourceUnavailable> {
        if self.positions.len() != expected_positions.len() {
            return Err(SourceUnavailable::new(format!(
                "attribution has {} positions, expected {}",
                self.positions.len(),
                expected_positions.len()
            )));
        }
        for (record, &expected) in self.positions.iter().zip(expected_positions) {
            if record.position != expected as u64 {
                return Err(SourceUnavailable::new(format!(
                    "attribution position {} does not equal selected position {expected}",
                    record.position
                )));
            }
            let dimensions = record.dimensions;
            for (name, evaluated, rank) in [
                (
                    "pre-re-emission-canonical",
                    dimensions.canonical_base_candidates_evaluated,
                    dimensions.canonical_base_target_rank,
                ),
                (
                    "normative",
                    dimensions.normative_candidates_evaluated,
                    dimensions.normative_target_rank,
                ),
                (
                    "sections-absent",
                    dimensions.sections_absent_candidates_evaluated,
                    dimensions.sections_absent_target_rank,
                ),
            ] {
                if !evaluated && rank.is_some() {
                    return Err(SourceUnavailable::new(format!(
                        "{name} target rank is present without an evaluated candidate list"
                    )));
                }
                if rank.is_some_and(|rank| rank == 0 || usize::from(rank) > 8) {
                    return Err(SourceUnavailable::new(format!(
                        "{name} target rank is outside 1..=8"
                    )));
                }
            }
            if dimensions.target_source.is_some() != dimensions.normative_target_rank.is_some() {
                return Err(SourceUnavailable::new(
                    "target source and normative target presence disagree",
                ));
            }
            if dimensions.target_source != Some(AttributionCandidateSource::Skipmix)
                && (dimensions.target_skmx_contributed || dimensions.target_psib_contributed)
            {
                return Err(SourceUnavailable::new(
                    "base/absent target claims SKMX or PSIB contribution",
                ));
            }
            if dimensions.target_source == Some(AttributionCandidateSource::Skipmix)
                && !dimensions.target_skmx_contributed
                && !dimensions.target_psib_contributed
            {
                return Err(SourceUnavailable::new(
                    "skipmix target has neither SKMX nor PSIB contribution",
                ));
            }
        }

        let mut expected_cells = self.clone();
        expected_cells.finalize();
        if self.cells != expected_cells.cells {
            return Err(SourceUnavailable::new(
                "aggregate attribution cells do not reproduce from positions",
            ));
        }
        let total = self.cells.iter().try_fold(0u64, |total, cell| {
            total
                .checked_add(cell.count)
                .ok_or_else(|| SourceUnavailable::new("attribution cell count overflow"))
        })?;
        if total != expected_positions.len() as u64 {
            return Err(SourceUnavailable::new(format!(
                "aggregate attribution total {total} does not equal {}",
                expected_positions.len()
            )));
        }

        let mut paired = PairedCounts::default();
        let mut changed = 0u64;
        let mut toward = 0u64;
        let mut away = 0u64;
        for position in &self.positions {
            match position.dimensions.normative_tla_cell {
                NormativeTlaCell::BothCorrect => paired.both += 1,
                NormativeTlaCell::NormativeOnlyCorrect => paired.normative_only += 1,
                NormativeTlaCell::TlaOnlyCorrect => paired.comparator_only += 1,
                NormativeTlaCell::NeitherCorrect => paired.neither += 1,
            }
            match position.dimensions.lane_transition {
                LaneTransition::Unchanged => {}
                LaneTransition::TowardTarget => {
                    changed += 1;
                    toward += 1;
                }
                LaneTransition::AwayFromTarget => {
                    changed += 1;
                    away += 1;
                }
                LaneTransition::ChangedOther => changed += 1,
            }
        }
        if paired != normative_vs_tla {
            return Err(SourceUnavailable::new(
                "attribution normative/TLA cells disagree with paired counts",
            ));
        }
        if (changed, toward, away) != (lane_changed, lane_toward, lane_away) {
            return Err(SourceUnavailable::new(format!(
                "attribution lane transitions {changed}/{toward}/{away} disagree with row {lane_changed}/{lane_toward}/{lane_away}"
            )));
        }
        Ok(())
    }
}

/// Live, monotonic evaluation counters. The callback receives this after the
/// probe and at least every 256 completed positions, so a run always exposes
/// rate, ETA, reachability, and both decision branches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ServingProgress {
    pub phase: &'static str,
    pub processed: usize,
    pub total: usize,
    pub served: u64,
    pub abstained: u64,
    pub declined: u64,
    pub normative_hits: u64,
    pub tla_hits: u64,
    pub lane_reachable: u64,
    pub lane_changed: u64,
    pub lane_toward: u64,
    pub lane_away: u64,
    pub sections_absent_hits: u64,
    pub label_shuffled_hits: u64,
    pub internal_base_control_checks: u64,
    pub internal_base_control_mismatches: u64,
    pub planted_controls_available: bool,
    pub elapsed_millis: u64,
    pub positions_per_second_milli: u64,
    pub eta_seconds: Option<u64>,
    pub workers: usize,
}

impl StatusBreakdown {
    fn record(&mut self, status: PolicyStatus, ngram_hit: bool) {
        match status {
            PolicyStatus::ExactContext => {
                self.exact_context += 1;
                if ngram_hit {
                    self.exact_context_ngram += 1;
                }
            }
            PolicyStatus::Graph => self.graph += 1,
            PolicyStatus::Novel => self.novel += 1,
            PolicyStatus::Contradictory => self.contradictory += 1,
        }
    }
    pub fn total(&self) -> u64 {
        self.exact_context + self.graph + self.novel + self.contradictory
    }
}

/// The measured serving-surface row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServingEvalRow {
    /// Bundle root the row measured.
    pub bundle: PathBuf,
    /// Exact component generation captured before any probe or measurement.
    /// The report builder rejects a row if it is not given this same snapshot.
    pub generation_cid: String,
    /// Canonical nested-sample size actually evaluated.
    pub sample_n: usize,
    /// Full held-out population from which this deterministic evaluation was
    /// selected.
    pub population_n: usize,
    pub mode: ServingEvalMode,
    /// Versioned position-selection semantics. Report binding independently
    /// reproduces this value and the selected position CID.
    pub position_selection_algorithm: String,
    pub workers: usize,
    pub elapsed_millis: u64,
    /// Positions where the policy served a token.
    pub served: u64,
    /// Served predictions that ran the widened re-probe first.
    pub served_widened: u64,
    /// Served predictions by resolving status (#234 item 3).
    pub served_by: StatusBreakdown,
    /// Abstentions by resolving status.
    pub abstained: StatusBreakdown,
    /// Served predictions matching the recorded corpus continuation.
    pub top1_served: u64,
    /// Served predictions matching the recorded teacher argmax.
    pub agree_served: u64,
    /// Same-position teacher-argmax hits by the plain TLA comparator.
    pub tla_hits: u64,
    /// Same-position teacher-argmax hits by the normative runtime with the
    /// SKMX/PSIB lane disabled (the causal base control).
    pub base_hits: u64,
    pub normative_vs_tla: PairedCounts,
    /// Main graph versus its in-runtime sections-absent candidate. Retained as
    /// an implementation cross-check, not used for RF-31 promotion.
    pub normative_vs_base: PairedCounts,
    /// Diagnostic-only comparison between the pre-re-emission canonical graph
    /// and the newly emitted sections-absent graph. Excluded from
    /// `QualityMeasurements`, promotion failures, and production admission.
    pub canonical_base_vs_sections_absent: Option<PairedCounts>,
    pub canonical_base_hits: u64,
    /// Main graph versus the independently emitted sections-absent artifact.
    pub normative_vs_sections_absent: Option<PairedCounts>,
    /// Label-shuffled planted graph versus the independently emitted
    /// sections-absent artifact, both evaluated on pristine held-out labels.
    pub label_shuffled_vs_sections_absent: Option<PairedCounts>,
    pub sections_absent_hits: u64,
    pub label_shuffled_hits: u64,
    pub internal_base_control_checks: u64,
    pub internal_base_control_mismatches: u64,
    pub lane_reachable: u64,
    pub lane_changed: u64,
    pub lane_toward: u64,
    pub lane_away: u64,
    /// Post-hoc decision evidence. Teacher targets are inspected only after
    /// the runtime and both planted controls have selected their candidates.
    pub decision_attribution: DecisionAttributionEvidence,
    /// D4 permitted, but the normative runtime could not select a candidate.
    pub declined: u64,
    /// Probe evidence: positions probed / served / hits.
    pub probe_positions: usize,
    pub probe_served: u64,
    pub probe_hits: u64,
    /// Exact absolute-position identities (wall time and worker count are
    /// deliberately excluded).
    pub evaluated_positions_cid: String,
    pub population_positions_cid: String,
}

/// External evidence hooks whose raw counts cannot be inferred inside this
/// evaluator. The report builder validates every witness position against the
/// exact evaluated population and derives its CID itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServingReportEvidence {
    pub compiler_revision: String,
    /// Durable, deterministic parity artifact bytes. The report builder hashes
    /// and parses these bytes itself, recomputes their counts, and binds their
    /// graph/signature/tokenizer CIDs to the bundle. Bare claimed counts
    /// cannot enter.
    pub cross_surface_evidence: Vec<u8>,
    /// Canonical replay artifact bytes. Positions, runtime winner claims,
    /// lane attribution, replay verdicts, and counts are parsed and
    /// independently recomputed; no caller-provided counter is accepted.
    pub witness_replay_evidence: Vec<u8>,
}

impl ServingReportEvidence {
    /// Validate and decode the external parity hook against the exact serving
    /// artifacts. Certifiers should use this seam before accepting raw parity
    /// bytes; checks and mismatches are properties of the validated artifact.
    pub fn validated_cross_surface_evidence(
        &self,
        graph: &[u8],
        signature_artifact: &[u8],
        tokenizer: &[u8],
        score_report: &[u8],
    ) -> Result<CrossSurfaceParityEvidence, SourceUnavailable> {
        CrossSurfaceParityEvidence::parse_and_validate_for_production_bundle(
            &self.cross_surface_evidence,
            graph,
            signature_artifact,
            tokenizer,
            score_report,
        )
    }

    /// Replay the raw witness artifact against the exact serving generation
    /// and evaluator population before any count enters a quality report.
    pub fn validated_witness_replay_evidence(
        &self,
        spec: NormativeWitnessReplaySpec<'_>,
    ) -> Result<NormativeWitnessReplayArtifact, SourceUnavailable> {
        parse_and_validate_normative_witness_replay(&self.witness_replay_evidence, spec)
    }
}

/// Durable paths for one run. Callers should place these in a staging bundle;
/// a failed run never overwrites a production-admissible report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServingReportPaths {
    pub progress_jsonl: PathBuf,
    pub terminal_json: PathBuf,
    pub deployed_quality_json: PathBuf,
}

impl ServingReportPaths {
    pub fn in_bundle(bundle: &ServingBundle) -> Self {
        let graph_dir = bundle.graph.parent().unwrap_or(bundle.root.as_path());
        Self {
            progress_jsonl: graph_dir.join("deployed_quality_progress.jsonl"),
            terminal_json: graph_dir.join("deployed_quality_terminal.json"),
            deployed_quality_json: graph_dir.join("deployed_quality_report.json"),
        }
    }
}

/// High-level producer result. `report` is absent only for a skipped or
/// unavailable evaluation; sample reports are present but remain `Estimate`
/// and cannot pass production validation.
#[derive(Debug, Clone)]
pub struct RecordedServingEval {
    pub outcome: ServingEvalOutcome,
    pub report: Option<DeployedQualityReport>,
    pub report_cid: Option<String>,
}

/// A recorded skip: the row was not measured, and the reason is data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServingEvalSkip {
    /// The probe could not finish inside its budget.
    ProbeBudgetExceeded { probed: usize, elapsed: Duration },
    /// The probe served predictions but none matched the corpus
    /// continuation or teacher argmax (#280 functional spot-check).
    ProbeFunctionalCheckFailed { served: u64, probed: usize },
    /// The subsampled evaluation exceeded its budget; partial counts are
    /// discarded because they do not equal the selected position population.
    EvalBudgetExceeded {
        done: usize,
        sample_n: usize,
        elapsed: Duration,
    },
}

/// Outcome of one serving-surface evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServingEvalOutcome {
    Row(Box<ServingEvalRow>),
    Skipped(ServingEvalSkip),
}

impl ServingBundle {
    /// Resolve a bundle directory into its serving artifact paths.
    /// Returns `None` when any required file is absent — callers treat
    /// that as "this directory is not a compiled serving bundle".
    pub fn discover(root: &Path) -> Option<Self> {
        let graph_nested = root.join("graph").join("score.r4g1");
        let graph_flat = root.join("score.r4g1");
        let graph = if graph_nested.is_file() {
            graph_nested
        } else if graph_flat.is_file() {
            graph_flat
        } else {
            return None;
        };
        let graph_dir = graph.parent()?.to_path_buf();
        let optional_file = |path: PathBuf| path.is_file().then_some(path);
        let canonical_base_graph = optional_file(graph_dir.join("score_canonical_base.r4g1"));
        let sections_absent_graph = optional_file(graph_dir.join("score_sections_absent.r4g1"));
        let label_shuffled_graph = optional_file(graph_dir.join("score_label_shuffled.r4g1"));
        let teacher = root.join("tless_artifacts.bin");
        let store = root.join("tless_store.bin");
        let corpus_meta = root.join("corpus.meta");
        let corpus_records = root.join("corpus.records");
        if !teacher.is_file()
            || !store.is_file()
            || !corpus_meta.is_file()
            || !corpus_records.is_file()
        {
            return None;
        }
        Some(Self {
            root: root.to_path_buf(),
            graph,
            canonical_base_graph,
            sections_absent_graph,
            label_shuffled_graph,
            teacher,
            store,
            corpus_meta,
            corpus_records,
            tokenizer: optional_file(root.join("tokenizer.bin")),
            tokenizer_adapter: optional_file(root.join("tokenizer_adapter.json")),
            score_report: optional_file(graph_dir.join("score_report.json")),
            compile_report: optional_file(root.join("graph-cover").join("cover_report.json")),
        })
    }

    /// Scan `.uor-models/compiled/*` under `base` for serving bundles,
    /// in deterministic (sorted) directory order.
    pub fn scan(base: &Path) -> Vec<Self> {
        let compiled = base.join(".uor-models").join("compiled");
        let Ok(entries) = std::fs::read_dir(&compiled) else {
            return Vec::new();
        };
        let mut roots: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        roots.sort();
        roots.iter().filter_map(|r| Self::discover(r)).collect()
    }
}

#[derive(Debug, Clone, Copy)]
struct PositionRow {
    position: usize,
    token: Option<u32>,
    internal_base_token: Option<u32>,
    canonical_base_token: Option<u32>,
    sections_absent_token: Option<u32>,
    label_shuffled_token: Option<u32>,
    status: Option<PolicyStatus>,
    ngram_hit: bool,
    widened: bool,
    abstained: bool,
    declined: bool,
    lane_reachable: bool,
    lane_changed: bool,
    next: u32,
    teacher_argmax: u32,
    tla_token: u32,
    attribution_dimensions: DecisionAttributionDimensions,
}

#[derive(Debug, Clone, Copy, Default)]
struct ControlPositionDecision {
    token: Option<u32>,
    candidates: Option<ServedCandidates>,
}

#[derive(Clone, Copy)]
struct EvaluationContext<'a> {
    corpus: &'a compiler::Corpus,
    artifacts: &'a compiler::Compiled,
    store: &'a runtime::Store,
    tables: &'a runtime::AssignTables,
    rotations: &'a [usize; compiler::WINDOW + 1],
}

struct ControlEngines<'borrow, 'graph> {
    canonical_base: Option<&'borrow mut NormativeServingEngine<'graph>>,
    sections_absent: Option<&'borrow mut NormativeServingEngine<'graph>>,
    label_shuffled: Option<&'borrow mut NormativeServingEngine<'graph>>,
}

fn evaluate_position(
    engine: &mut NormativeServingEngine<'_>,
    controls: ControlEngines<'_, '_>,
    context: EvaluationContext<'_>,
    position: usize,
) -> Result<PositionRow, SourceUnavailable> {
    // Each recorded position is an isolated teacher-forced decision. D4's
    // bounded session memory therefore resets between positions; no worker or
    // sharding order can change a verdict.
    engine.reset_policy_state();
    let window = induction::context_window(context.corpus, position);
    let code = runtime::code_plain_with(
        context.tables,
        context.artifacts,
        context.rotations,
        context.corpus,
        position,
    );
    let tla_token = runtime::predict_witness_plain(context.store, &code).token;
    let next = context.corpus.next[position];
    let mut row = PositionRow {
        position,
        token: None,
        internal_base_token: None,
        canonical_base_token: None,
        sections_absent_token: None,
        label_shuffled_token: None,
        status: None,
        ngram_hit: false,
        widened: false,
        abstained: false,
        declined: false,
        lane_reachable: false,
        lane_changed: false,
        next,
        // The recorded target is deliberately not read until every selector
        // below has returned.
        teacher_argmax: 0,
        tla_token,
        // Replaced only after main and both control predictions finish.
        attribution_dimensions: DecisionAttributionDimensions {
            status: AttributionStatus::Unavailable,
            disposition: AttributionDisposition::Declined,
            normative_tla_cell: NormativeTlaCell::NeitherCorrect,
            canonical_base_candidates_evaluated: false,
            canonical_base_target_rank: None,
            normative_candidates_evaluated: false,
            normative_target_rank: None,
            sections_absent_candidates_evaluated: false,
            sections_absent_target_rank: None,
            target_source: None,
            target_skmx_contributed: false,
            target_psib_contributed: false,
            lane_transition: LaneTransition::Unchanged,
        },
    };
    let mut normative_candidates = None;
    match engine
        .predict(&window)
        .map_err(|error| SourceUnavailable::new(format!("serving decision: {error}")))?
    {
        NormativeServingDecision::Serve(outcome) => {
            row.token = Some(outcome.token);
            row.internal_base_token = Some(outcome.base_token);
            row.status = Some(outcome.status.into());
            row.ngram_hit = outcome.ngram_hit;
            row.widened = outcome.widened;
            row.lane_reachable = outcome.lane_reachable;
            row.lane_changed = outcome.token != outcome.base_token;
            normative_candidates = Some(outcome.candidates);
        }
        NormativeServingDecision::Abstain(outcome) => {
            row.status = Some(outcome.status.into());
            row.ngram_hit = outcome.ngram_hit;
            row.widened = outcome.widened;
            row.abstained = true;
        }
        NormativeServingDecision::Decline(_) => row.declined = true,
    }
    let canonical_base = evaluate_control_decision(controls.canonical_base, &window)?;
    let sections_absent = evaluate_control_decision(controls.sections_absent, &window)?;
    let label_shuffled = evaluate_control_decision(controls.label_shuffled, &window)?;
    row.canonical_base_token = canonical_base.token;
    row.sections_absent_token = sections_absent.token;
    row.label_shuffled_token = label_shuffled.token;
    row.teacher_argmax = context.corpus.t_argmax[position];

    // Teacher targets are diagnostic labels, never selector inputs. All
    // runtime decisions above are complete before this post-hoc cross-tab is
    // constructed.
    row.attribution_dimensions = posthoc_attribution_dimensions(
        &row,
        canonical_base.candidates.as_ref(),
        normative_candidates.as_ref(),
        sections_absent.candidates.as_ref(),
    );
    Ok(row)
}

fn evaluate_control_decision(
    engine: Option<&mut NormativeServingEngine<'_>>,
    window: &[u32],
) -> Result<ControlPositionDecision, SourceUnavailable> {
    let Some(engine) = engine else {
        return Ok(ControlPositionDecision::default());
    };
    engine.reset_policy_state();
    match engine
        .predict(window)
        .map_err(|error| SourceUnavailable::new(format!("control serving decision: {error}")))?
    {
        NormativeServingDecision::Serve(outcome) => Ok(ControlPositionDecision {
            token: Some(outcome.token),
            candidates: Some(outcome.candidates),
        }),
        NormativeServingDecision::Abstain(_) | NormativeServingDecision::Decline(_) => {
            Ok(ControlPositionDecision::default())
        }
    }
}

fn posthoc_attribution_dimensions(
    row: &PositionRow,
    canonical_base_candidates: Option<&ServedCandidates>,
    normative_candidates: Option<&ServedCandidates>,
    sections_absent_candidates: Option<&ServedCandidates>,
) -> DecisionAttributionDimensions {
    let normative_hit = row.token == Some(row.teacher_argmax);
    let tla_hit = row.tla_token == row.teacher_argmax;
    let base_hit = row.internal_base_token == Some(row.teacher_argmax);
    let normative_target = normative_candidates.and_then(|candidates| {
        candidates
            .ranked()
            .iter()
            .enumerate()
            .find(|(_, candidate)| candidate.token == row.teacher_argmax)
    });
    let canonical_base_target_rank = canonical_base_candidates.and_then(|candidates| {
        candidates
            .ranked()
            .iter()
            .position(|candidate| candidate.token == row.teacher_argmax)
            .map(|rank| (rank + 1) as u8)
    });
    let sections_absent_target_rank = sections_absent_candidates.and_then(|candidates| {
        candidates
            .ranked()
            .iter()
            .position(|candidate| candidate.token == row.teacher_argmax)
            .map(|rank| (rank + 1) as u8)
    });
    let disposition = if row.declined {
        AttributionDisposition::Declined
    } else if row.abstained {
        AttributionDisposition::Abstained
    } else {
        AttributionDisposition::Served
    };
    let lane_transition = if !row.lane_changed {
        LaneTransition::Unchanged
    } else if normative_hit && !base_hit {
        LaneTransition::TowardTarget
    } else if !normative_hit && base_hit {
        LaneTransition::AwayFromTarget
    } else {
        LaneTransition::ChangedOther
    };
    DecisionAttributionDimensions {
        status: row.status.into(),
        disposition,
        normative_tla_cell: match (normative_hit, tla_hit) {
            (true, true) => NormativeTlaCell::BothCorrect,
            (true, false) => NormativeTlaCell::NormativeOnlyCorrect,
            (false, true) => NormativeTlaCell::TlaOnlyCorrect,
            (false, false) => NormativeTlaCell::NeitherCorrect,
        },
        canonical_base_candidates_evaluated: canonical_base_candidates.is_some(),
        canonical_base_target_rank,
        normative_candidates_evaluated: normative_candidates.is_some(),
        normative_target_rank: normative_target.map(|(rank, _)| (rank + 1) as u8),
        sections_absent_candidates_evaluated: sections_absent_candidates.is_some(),
        sections_absent_target_rank,
        target_source: normative_target.map(|(_, candidate)| candidate.source.into()),
        target_skmx_contributed: normative_target
            .is_some_and(|(_, candidate)| candidate.skmx_contributed),
        target_psib_contributed: normative_target
            .is_some_and(|(_, candidate)| candidate.psib_contributed),
        lane_transition,
    }
}

fn record_position(row: &mut ServingEvalRow, position: PositionRow) {
    let normative_hit = position.token == Some(position.teacher_argmax);
    let base_hit = position.internal_base_token == Some(position.teacher_argmax);
    let tla_hit = position.tla_token == position.teacher_argmax;
    row.normative_vs_tla.record(normative_hit, tla_hit);
    row.normative_vs_base.record(normative_hit, base_hit);
    row.tla_hits += u64::from(tla_hit);
    row.base_hits += u64::from(base_hit);
    if let Some(counts) = row.canonical_base_vs_sections_absent.as_mut() {
        let canonical_hit = position.canonical_base_token == Some(position.teacher_argmax);
        let absent_hit = position.sections_absent_token == Some(position.teacher_argmax);
        counts.record(canonical_hit, absent_hit);
        row.canonical_base_hits += u64::from(canonical_hit);
    }
    if let Some(counts) = row.normative_vs_sections_absent.as_mut() {
        let absent_hit = position.sections_absent_token == Some(position.teacher_argmax);
        counts.record(normative_hit, absent_hit);
        row.sections_absent_hits += u64::from(absent_hit);
        row.internal_base_control_checks += 1;
        row.internal_base_control_mismatches +=
            u64::from(position.internal_base_token != position.sections_absent_token);
    }
    if let Some(counts) = row.label_shuffled_vs_sections_absent.as_mut() {
        let shuffled_hit = position.label_shuffled_token == Some(position.teacher_argmax);
        let absent_hit = position.sections_absent_token == Some(position.teacher_argmax);
        counts.record(shuffled_hit, absent_hit);
        row.label_shuffled_hits += u64::from(shuffled_hit);
    }
    row.lane_reachable += u64::from(position.lane_reachable);
    row.lane_changed += u64::from(position.lane_changed);
    if position.lane_changed {
        row.lane_toward += u64::from(normative_hit && !base_hit);
        row.lane_away += u64::from(!normative_hit && base_hit);
    }
    row.decision_attribution
        .record(PositionDecisionAttribution {
            position: position.position as u64,
            dimensions: position.attribution_dimensions,
        });
    if position.declined {
        row.declined += 1;
    } else if position.abstained {
        if let Some(status) = position.status {
            row.abstained.record(status, position.ngram_hit);
        }
    } else if let Some(token) = position.token {
        row.served += 1;
        if let Some(status) = position.status {
            row.served_by.record(status, position.ngram_hit);
        }
        row.served_widened += u64::from(position.widened);
        row.top1_served += u64::from(token == position.next);
        row.agree_served += u64::from(normative_hit);
    }
}

fn progress_snapshot(
    phase: &'static str,
    row: &ServingEvalRow,
    processed: usize,
    total: usize,
    started: Instant,
) -> ServingProgress {
    let elapsed_millis = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let positions_per_second_milli = (processed as u64)
        .saturating_mul(1_000_000)
        .checked_div(elapsed_millis)
        .unwrap_or(0);
    let eta_seconds = (processed > 0 && processed < total).then(|| {
        elapsed_millis.saturating_mul((total - processed) as u64) / processed as u64 / 1000
    });
    ServingProgress {
        phase,
        processed,
        total,
        served: row.served,
        abstained: row.abstained.total(),
        declined: row.declined,
        normative_hits: row.agree_served,
        tla_hits: row.tla_hits,
        lane_reachable: row.lane_reachable,
        lane_changed: row.lane_changed,
        lane_toward: row.lane_toward,
        lane_away: row.lane_away,
        sections_absent_hits: row.sections_absent_hits,
        label_shuffled_hits: row.label_shuffled_hits,
        internal_base_control_checks: row.internal_base_control_checks,
        internal_base_control_mismatches: row.internal_base_control_mismatches,
        planted_controls_available: row.normative_vs_sections_absent.is_some()
            && row.label_shuffled_vs_sections_absent.is_some(),
        elapsed_millis,
        positions_per_second_milli,
        eta_seconds,
        workers: row.workers,
    }
}

fn probe_progress_snapshot(
    processed: usize,
    total: usize,
    served: u64,
    hits: u64,
    started: Instant,
    workers: usize,
    planted_controls_available: bool,
) -> ServingProgress {
    let elapsed_millis = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let positions_per_second_milli = (processed as u64)
        .saturating_mul(1_000_000)
        .checked_div(elapsed_millis)
        .unwrap_or(0);
    let eta_seconds = (processed > 0 && processed < total).then(|| {
        elapsed_millis.saturating_mul((total - processed) as u64) / processed as u64 / 1000
    });
    ServingProgress {
        phase: "probe",
        processed,
        total,
        served,
        abstained: processed as u64 - served,
        declined: 0,
        normative_hits: hits,
        tla_hits: 0,
        lane_reachable: 0,
        lane_changed: 0,
        lane_toward: 0,
        lane_away: 0,
        sections_absent_hits: 0,
        label_shuffled_hits: 0,
        internal_base_control_checks: 0,
        internal_base_control_mismatches: 0,
        planted_controls_available,
        elapsed_millis,
        positions_per_second_milli,
        eta_seconds,
        workers,
    }
}

fn positions_cid_usize(positions: &[usize]) -> String {
    let positions: Vec<u64> = positions.iter().map(|&position| position as u64).collect();
    deployed_quality_positions_cid(&positions)
}

/// One canonical, content-bound serving-evaluation selection. Callers which
/// generate witness evidence must consume this exact list and CID rather than
/// independently slicing the certification partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServingEvalSelection {
    pub positions: Vec<usize>,
    pub evaluated_positions_cid: String,
    pub population_positions_cid: String,
    pub algorithm: &'static str,
}

/// Select a full census or nested story-distributed sample from the canonical
/// certification population. This is the sole public evaluator/witness seam;
/// it delegates sample semantics to
/// [`crate::deployed_quality::deterministic_story_sample`], which the report
/// binding layer independently invokes as well.
pub fn select_serving_eval_positions(
    story_by_position: &[u32],
    held_out: &[usize],
    mode: ServingEvalMode,
) -> Result<ServingEvalSelection, SourceUnavailable> {
    let population: Vec<u64> = held_out
        .iter()
        .copied()
        .map(|position| {
            u64::try_from(position)
                .map_err(|_| SourceUnavailable::new("corpus position does not fit u64"))
        })
        .collect::<Result<Vec<_>, SourceUnavailable>>()?;
    let (positions, algorithm) = match mode {
        ServingEvalMode::FullCensus => (population.clone(), FULL_POPULATION_SELECTION_ALGORITHM),
        ServingEvalMode::Sample { positions } => (
            deterministic_story_sample(story_by_position, &population, positions)
                .map_err(|error| SourceUnavailable::new(error.to_string()))?,
            DETERMINISTIC_SAMPLE_SELECTION_ALGORITHM,
        ),
    };
    let positions: Vec<usize> = positions
        .into_iter()
        .map(|position| {
            usize::try_from(position)
                .map_err(|_| SourceUnavailable::new("selected corpus position does not fit usize"))
        })
        .collect::<Result<Vec<_>, SourceUnavailable>>()?;
    Ok(ServingEvalSelection {
        evaluated_positions_cid: positions_cid_usize(&positions),
        population_positions_cid: deployed_quality_positions_cid(&population),
        positions,
        algorithm,
    })
}

/// Evaluate one bundle's exact normative serving surface on its held-out
/// partition. Work is deterministically sharded over all configured workers;
/// results reduce in original position order, so worker count cannot change
/// artifact bytes or verdicts.
pub fn evaluate_serving_bundle(
    bundle: &ServingBundle,
    budgets: ServingEvalBudgets,
    progress: &mut dyn FnMut(ServingProgress),
) -> Result<ServingEvalOutcome, SourceUnavailable> {
    let snapshot = ServingBundleSnapshot::capture(bundle)?;
    evaluate_serving_snapshot(&snapshot, budgets, progress)
}

/// Evaluate a previously captured immutable generation. Measurement, controls,
/// comparator, and the resulting row all borrow this single byte snapshot.
pub fn evaluate_serving_snapshot(
    snapshot: &ServingBundleSnapshot,
    budgets: ServingEvalBudgets,
    progress: &mut dyn FnMut(ServingProgress),
) -> Result<ServingEvalOutcome, SourceUnavailable> {
    progress(ServingProgress {
        phase: "load",
        processed: 0,
        total: 0,
        served: 0,
        abstained: 0,
        declined: 0,
        normative_hits: 0,
        tla_hits: 0,
        lane_reachable: 0,
        lane_changed: 0,
        lane_toward: 0,
        lane_away: 0,
        sections_absent_hits: 0,
        label_shuffled_hits: 0,
        internal_base_control_checks: 0,
        internal_base_control_mismatches: 0,
        planted_controls_available: snapshot.sections_absent_graph.is_some()
            && snapshot.label_shuffled_graph.is_some(),
        elapsed_millis: 0,
        positions_per_second_milli: 0,
        eta_seconds: None,
        workers: budgets.workers.max(1),
    });
    let graph_bytes = snapshot.graph.as_slice();
    let canonical_base_bytes = snapshot.canonical_base_graph.as_deref();
    let sections_absent_bytes = snapshot.sections_absent_graph.as_deref();
    let label_shuffled_bytes = snapshot.label_shuffled_graph.as_deref();
    validate_planted_control_graphs(
        graph_bytes,
        canonical_base_bytes,
        sections_absent_bytes,
        label_shuffled_bytes,
    )?;
    let teacher_bytes = snapshot.teacher.as_slice();
    let store_bytes = snapshot.store.as_slice();
    let score_report = snapshot
        .score_report
        .as_deref()
        .filter(|bytes| serde_json::from_slice::<serde_json::Value>(bytes).is_ok());
    let artifacts = compiler::parse_artifacts(teacher_bytes)
        .ok_or_else(|| SourceUnavailable::new("not a supported plain-TLA artifact container"))?;
    let store = runtime::parse_store(store_bytes)
        .ok_or_else(|| SourceUnavailable::new("not a valid TLS1 TLA store"))?;
    let tables = runtime::AssignTables::new(&artifacts);
    let rotations = compiler::derive_rotations();

    let corpus = compiler::load_corpus_bytes(&snapshot.corpus_meta, &snapshot.corpus_records, None)
        .ok_or_else(|| {
            SourceUnavailable::new("corpus: captured bytes are incomplete or invalid")
        })?;
    let (_, held_out) = induction::split_positions(&corpus);
    if held_out.is_empty() {
        return Err(SourceUnavailable::new(
            "the bundle's held-out partition is empty",
        ));
    }
    let ServingEvalSelection {
        positions,
        evaluated_positions_cid,
        population_positions_cid,
        algorithm: position_selection_algorithm,
    } = select_serving_eval_positions(&corpus.story, &held_out, budgets.mode)?;
    if positions.is_empty() {
        return Err(SourceUnavailable::new(
            "the configured serving evaluation selects zero positions",
        ));
    }
    if matches!(budgets.mode, ServingEvalMode::Sample { .. }) && positions.len() == held_out.len() {
        return Err(SourceUnavailable::new(
            "sample mode reaches the full population; select full-census mode explicitly",
        ));
    }
    let workers = budgets.workers.max(1).min(positions.len().max(1));
    let parts = serving_engine_parts(snapshot, graph_bytes, score_report);
    let mut probe_engine = NormativeServingEngine::load_for_research(parts)?;
    let mut probe_canonical_base = canonical_base_bytes
        .map(|graph| NormativeServingEngine::load_for_research(EngineParts { graph, ..parts }))
        .transpose()?;
    let mut probe_sections_absent = sections_absent_bytes
        .map(|graph| NormativeServingEngine::load_for_research(EngineParts { graph, ..parts }))
        .transpose()?;
    let mut probe_label_shuffled = label_shuffled_bytes
        .map(|graph| NormativeServingEngine::load_for_research(EngineParts { graph, ..parts }))
        .transpose()?;
    let probe_n = positions.len().min(PROBE_POSITIONS);
    let probe_start = Instant::now();
    let (mut probe_served, mut probe_hits) = (0u64, 0u64);
    for (done, &position) in positions[..probe_n].iter().enumerate() {
        if probe_start.elapsed() >= budgets.probe {
            return Ok(ServingEvalOutcome::Skipped(
                ServingEvalSkip::ProbeBudgetExceeded {
                    probed: done,
                    elapsed: probe_start.elapsed(),
                },
            ));
        }
        let measured = evaluate_position(
            &mut probe_engine,
            ControlEngines {
                canonical_base: probe_canonical_base.as_mut(),
                sections_absent: probe_sections_absent.as_mut(),
                label_shuffled: probe_label_shuffled.as_mut(),
            },
            EvaluationContext {
                corpus: &corpus,
                artifacts: &artifacts,
                store: &store,
                tables: &tables,
                rotations: &rotations,
            },
            position,
        )
        .map_err(SourceUnavailable::new)?;
        if let Some(token) = measured.token {
            probe_served += 1;
            probe_hits += u64::from(token == measured.next || token == measured.teacher_argmax);
        }
        let processed = done + 1;
        if processed.is_multiple_of(PROBE_PROGRESS_INTERVAL) || processed == probe_n {
            progress(probe_progress_snapshot(
                processed,
                probe_n,
                probe_served,
                probe_hits,
                probe_start,
                1,
                sections_absent_bytes.is_some() && label_shuffled_bytes.is_some(),
            ));
        }
    }
    if probe_served == 0 || probe_hits == 0 {
        return Ok(ServingEvalOutcome::Skipped(
            ServingEvalSkip::ProbeFunctionalCheckFailed {
                served: probe_served,
                probed: probe_n,
            },
        ));
    }

    let eval_start = Instant::now();
    let mut aggregate = ServingEvalRow {
        bundle: snapshot.bundle.root.clone(),
        generation_cid: snapshot.generation_cid.clone(),
        sample_n: positions.len(),
        population_n: held_out.len(),
        mode: budgets.mode,
        position_selection_algorithm: position_selection_algorithm.to_string(),
        workers,
        elapsed_millis: 0,
        served: 0,
        served_widened: 0,
        served_by: StatusBreakdown::default(),
        abstained: StatusBreakdown::default(),
        top1_served: 0,
        agree_served: 0,
        tla_hits: 0,
        base_hits: 0,
        normative_vs_tla: PairedCounts::default(),
        normative_vs_base: PairedCounts::default(),
        canonical_base_vs_sections_absent: (canonical_base_bytes.is_some()
            && sections_absent_bytes.is_some())
        .then(PairedCounts::default),
        canonical_base_hits: 0,
        normative_vs_sections_absent: sections_absent_bytes.is_some().then(PairedCounts::default),
        label_shuffled_vs_sections_absent: (sections_absent_bytes.is_some()
            && label_shuffled_bytes.is_some())
        .then(PairedCounts::default),
        sections_absent_hits: 0,
        label_shuffled_hits: 0,
        internal_base_control_checks: 0,
        internal_base_control_mismatches: 0,
        lane_reachable: 0,
        lane_changed: 0,
        lane_toward: 0,
        lane_away: 0,
        decision_attribution: DecisionAttributionEvidence::default(),
        declined: 0,
        probe_positions: probe_n,
        probe_served,
        probe_hits,
        evaluated_positions_cid: evaluated_positions_cid.clone(),
        population_positions_cid: population_positions_cid.clone(),
    };
    let cancelled = AtomicBool::new(false);
    let (sender, receiver) = mpsc::channel::<(usize, Result<PositionRow, SourceUnavailable>)>();
    let mut ordered = vec![None; positions.len()];
    let mut done = 0usize;
    let mut worker_error = None;
    let mut budget_exceeded = false;

    std::thread::scope(|scope| {
        for worker in 0..workers {
            let sender = sender.clone();
            let positions = &positions;
            let corpus = &corpus;
            let artifacts = &artifacts;
            let store = &store;
            let tables = &tables;
            let rotations = &rotations;
            let cancelled = &cancelled;
            scope.spawn(move || {
                let mut engine = match NormativeServingEngine::load_for_research(parts) {
                    Ok(engine) => engine,
                    Err(error) => {
                        let _ = sender.send((usize::MAX, Err(error)));
                        return;
                    }
                };
                let mut canonical_base_engine = match canonical_base_bytes
                    .map(|graph| {
                        NormativeServingEngine::load_for_research(EngineParts { graph, ..parts })
                    })
                    .transpose()
                {
                    Ok(engine) => engine,
                    Err(error) => {
                        let _ = sender.send((usize::MAX, Err(error)));
                        return;
                    }
                };
                let mut sections_absent_engine = match sections_absent_bytes
                    .map(|graph| {
                        NormativeServingEngine::load_for_research(EngineParts { graph, ..parts })
                    })
                    .transpose()
                {
                    Ok(engine) => engine,
                    Err(error) => {
                        let _ = sender.send((usize::MAX, Err(error)));
                        return;
                    }
                };
                let mut label_shuffled_engine = match label_shuffled_bytes
                    .map(|graph| {
                        NormativeServingEngine::load_for_research(EngineParts { graph, ..parts })
                    })
                    .transpose()
                {
                    Ok(engine) => engine,
                    Err(error) => {
                        let _ = sender.send((usize::MAX, Err(error)));
                        return;
                    }
                };
                for ordinal in (worker..positions.len()).step_by(workers) {
                    if cancelled.load(Ordering::Relaxed) {
                        break;
                    }
                    let result = evaluate_position(
                        &mut engine,
                        ControlEngines {
                            canonical_base: canonical_base_engine.as_mut(),
                            sections_absent: sections_absent_engine.as_mut(),
                            label_shuffled: label_shuffled_engine.as_mut(),
                        },
                        EvaluationContext {
                            corpus,
                            artifacts,
                            store,
                            tables,
                            rotations,
                        },
                        positions[ordinal],
                    );
                    if sender.send((ordinal, result)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(sender);
        while done < positions.len() {
            if eval_start.elapsed() >= budgets.eval {
                budget_exceeded = true;
                cancelled.store(true, Ordering::Relaxed);
                break;
            }
            match receiver.recv_timeout(Duration::from_millis(200)) {
                Ok((ordinal, Ok(position))) if ordinal < ordered.len() => {
                    ordered[ordinal] = Some(position);
                    record_position(&mut aggregate, position);
                    done += 1;
                    if done.is_multiple_of(EVAL_PROGRESS_INTERVAL) || done == positions.len() {
                        progress(progress_snapshot(
                            "evaluate",
                            &aggregate,
                            done,
                            positions.len(),
                            eval_start,
                        ));
                    }
                }
                Ok((_, Err(error))) => {
                    worker_error = Some(error);
                    cancelled.store(true, Ordering::Relaxed);
                    break;
                }
                Ok((ordinal, Ok(_))) => {
                    worker_error = Some(SourceUnavailable::new(format!(
                        "worker returned out-of-range ordinal {ordinal}"
                    )));
                    cancelled.store(true, Ordering::Relaxed);
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
    if let Some(error) = worker_error {
        return Err(error);
    }
    if budget_exceeded || done != positions.len() {
        return Ok(ServingEvalOutcome::Skipped(
            ServingEvalSkip::EvalBudgetExceeded {
                done,
                sample_n: positions.len(),
                elapsed: eval_start.elapsed(),
            },
        ));
    }

    // Re-reduce in canonical position order. The live accumulator above is
    // instrumentation only; this ordered reduction is the evidence value and
    // is invariant to worker count and completion order.
    let mut row = ServingEvalRow {
        bundle: snapshot.bundle.root.clone(),
        generation_cid: snapshot.generation_cid.clone(),
        sample_n: positions.len(),
        population_n: held_out.len(),
        mode: budgets.mode,
        position_selection_algorithm: position_selection_algorithm.to_string(),
        workers,
        elapsed_millis: eval_start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
        served: 0,
        served_widened: 0,
        served_by: StatusBreakdown::default(),
        abstained: StatusBreakdown::default(),
        top1_served: 0,
        agree_served: 0,
        tla_hits: 0,
        base_hits: 0,
        normative_vs_tla: PairedCounts::default(),
        normative_vs_base: PairedCounts::default(),
        canonical_base_vs_sections_absent: (canonical_base_bytes.is_some()
            && sections_absent_bytes.is_some())
        .then(PairedCounts::default),
        canonical_base_hits: 0,
        normative_vs_sections_absent: sections_absent_bytes.is_some().then(PairedCounts::default),
        label_shuffled_vs_sections_absent: (sections_absent_bytes.is_some()
            && label_shuffled_bytes.is_some())
        .then(PairedCounts::default),
        sections_absent_hits: 0,
        label_shuffled_hits: 0,
        internal_base_control_checks: 0,
        internal_base_control_mismatches: 0,
        lane_reachable: 0,
        lane_changed: 0,
        lane_toward: 0,
        lane_away: 0,
        decision_attribution: DecisionAttributionEvidence::default(),
        declined: 0,
        probe_positions: probe_n,
        probe_served,
        probe_hits,
        evaluated_positions_cid,
        population_positions_cid,
    };
    for position in ordered {
        record_position(
            &mut row,
            position.expect("every completed ordinal has a measured row"),
        );
    }
    row.decision_attribution.finalize();
    debug_assert_eq!(row.normative_vs_tla.total(), positions.len() as u64);
    progress(progress_snapshot(
        "complete",
        &row,
        positions.len(),
        positions.len(),
        eval_start,
    ));
    Ok(ServingEvalOutcome::Row(Box::new(row)))
}

fn serving_engine_parts<'a>(
    snapshot: &'a ServingBundleSnapshot,
    graph: &'a [u8],
    score_report: Option<&'a [u8]>,
) -> EngineParts<'a> {
    EngineParts {
        graph,
        signature_artifact: snapshot.teacher.as_slice(),
        tokenizer: snapshot.tokenizer.as_deref(),
        score_report,
    }
}

/// Build the deterministic, content-bound quality report for one completed
/// row. Every byte-derived identity is recomputed here through
/// [`derive_deployed_quality_bindings`]; only the compiler source revision and
/// separately executed parity/witness observations enter through `evidence`.
pub fn build_deployed_quality_report(
    bundle: &ServingBundle,
    row: &ServingEvalRow,
    evidence: &ServingReportEvidence,
) -> Result<DeployedQualityReport, SourceUnavailable> {
    let snapshot = ServingBundleSnapshot::capture(bundle)?;
    build_deployed_quality_report_from_snapshot(&snapshot, row, evidence)
}

fn build_deployed_quality_report_from_snapshot(
    snapshot: &ServingBundleSnapshot,
    row: &ServingEvalRow,
    evidence: &ServingReportEvidence,
) -> Result<DeployedQualityReport, SourceUnavailable> {
    if row.bundle != snapshot.bundle.root {
        return Err(SourceUnavailable::new(format!(
            "serving row names bundle {}, report builder received {}",
            row.bundle.display(),
            snapshot.bundle.root.display()
        )));
    }
    if row.generation_cid != snapshot.generation_cid {
        return Err(SourceUnavailable::new(format!(
            "serving row generation {} does not equal captured report generation {}",
            row.generation_cid, snapshot.generation_cid
        )));
    }
    let graph = snapshot.graph.as_slice();
    let teacher = snapshot.teacher.as_slice();
    let store = snapshot.store.as_slice();
    let corpus_meta = snapshot.corpus_meta.as_slice();
    let corpus_records = snapshot.corpus_records.as_slice();
    let tokenizer = required_snapshot_component("tokenizer", snapshot.tokenizer.as_deref())?;
    let tokenizer_adapter =
        required_snapshot_component("tokenizer adapter", snapshot.tokenizer_adapter.as_deref())?;
    let score_report =
        required_snapshot_component("score report", snapshot.score_report.as_deref())?;
    let cross_surface_evidence =
        evidence.validated_cross_surface_evidence(graph, teacher, tokenizer, score_report)?;
    let compile_report =
        required_snapshot_component("compile report", snapshot.compile_report.as_deref())?;

    let corpus = compiler::load_corpus_bytes(corpus_meta, corpus_records, None)
        .ok_or_else(|| SourceUnavailable::new("deployed-quality corpus parse failed"))?;
    let (_, held_out) = induction::split_positions(&corpus);
    let selection = select_serving_eval_positions(&corpus.story, &held_out, row.mode)?;
    let selected = selection.positions;
    if selected.len() != row.sample_n || held_out.len() != row.population_n {
        return Err(SourceUnavailable::new(format!(
            "serving row population {}/{}, recomputed {}/{}",
            row.sample_n,
            row.population_n,
            selected.len(),
            held_out.len()
        )));
    }
    if row.position_selection_algorithm != selection.algorithm {
        return Err(SourceUnavailable::new(format!(
            "serving row selection algorithm {:?} does not equal canonical {:?}",
            row.position_selection_algorithm, selection.algorithm
        )));
    }
    let full_positions = usize_positions_to_u64(&held_out)?;
    let evaluated_positions = usize_positions_to_u64(&selected)?;
    if selection.population_positions_cid != row.population_positions_cid
        || selection.evaluated_positions_cid != row.evaluated_positions_cid
    {
        return Err(SourceUnavailable::new(
            "serving row position identities do not reproduce from the loaded corpus",
        ));
    }
    row.decision_attribution
        .validate(
            &selected,
            row.normative_vs_tla,
            row.lane_changed,
            row.lane_toward,
            row.lane_away,
        )
        .map_err(|reason| {
            SourceUnavailable::new(format!("serving row decision attribution: {reason}"))
        })?;
    let witness_artifact =
        evidence.validated_witness_replay_evidence(NormativeWitnessReplaySpec {
            material: NormativeWitnessReplayMaterial {
                graph,
                signature_artifact: teacher,
                tokenizer,
                score_report: Some(score_report),
                corpus_meta,
                corpus_records,
            },
            evaluated_positions: &evaluated_positions,
            sample_size: DEFAULT_NORMATIVE_WITNESS_SAMPLE,
        })?;

    let bindings = derive_deployed_quality_bindings(DeployedQualityBindingMaterial {
        graph,
        teacher_artifact: teacher,
        corpus_meta,
        corpus_records,
        tokenizer,
        tokenizer_adapter,
        score_report,
        compile_report,
        compiler_revision: &evidence.compiler_revision,
        full_population_positions: &full_positions,
        evaluated_positions: &evaluated_positions,
    })
    .map_err(|error| SourceUnavailable::new(error.to_string()))?;

    let tla_definition_cid = report_tagged_cid(
        b"r4-deployed-quality-tla-comparator/1",
        &[TLA_COMPARATOR_VERSION.as_bytes(), store],
    );
    let versus_tla = paired_comparison(
        TLA_COMPARATOR_ID,
        TLA_COMPARATOR_VERSION,
        tla_definition_cid,
        &bindings.partition.evaluated_positions_cid,
        row.normative_vs_tla,
    )?;

    let sections_absent_bytes = snapshot.sections_absent_graph.as_deref();
    let label_shuffled_bytes = snapshot.label_shuffled_graph.as_deref();
    let planted_controls_available = sections_absent_bytes.is_some()
        && label_shuffled_bytes.is_some()
        && row.normative_vs_sections_absent.is_some()
        && row.label_shuffled_vs_sections_absent.is_some();
    validate_planted_control_graphs(
        graph,
        snapshot.canonical_base_graph.as_deref(),
        sections_absent_bytes,
        label_shuffled_bytes,
    )?;

    let (versus_sections_absent, label_control) = if planted_controls_available {
        let sections_absent_bytes = sections_absent_bytes.ok_or_else(|| {
            SourceUnavailable::new("sections-absent control disappeared during report build")
        })?;
        let label_shuffled_bytes = label_shuffled_bytes.ok_or_else(|| {
            SourceUnavailable::new("label-shuffled control disappeared during report build")
        })?;
        let absent_definition_cid = report_tagged_cid(
            b"r4-deployed-quality-sections-absent-comparator/1",
            &[
                SECTIONS_ABSENT_COMPARATOR_VERSION.as_bytes(),
                sections_absent_bytes,
            ],
        );
        let comparison = paired_comparison(
            SECTIONS_ABSENT_COMPARATOR_ID,
            SECTIONS_ABSENT_COMPARATOR_VERSION,
            absent_definition_cid.clone(),
            &bindings.partition.evaluated_positions_cid,
            row.normative_vs_sections_absent
                .ok_or_else(|| SourceUnavailable::new("missing sections-absent paired counts"))?,
        )?;
        let shuffled_counts = row
            .label_shuffled_vs_sections_absent
            .ok_or_else(|| SourceUnavailable::new("missing label-shuffled paired counts"))?;
        let shuffled_comparison = paired_comparison(
            SECTIONS_ABSENT_COMPARATOR_ID,
            SECTIONS_ABSENT_COMPARATOR_VERSION,
            absent_definition_cid,
            &bindings.partition.evaluated_positions_cid,
            shuffled_counts,
        )?;
        let identity_cid = report_tagged_cid(
            b"r4-deployed-quality-label-shuffled-control/1",
            &[
                LABEL_SHUFFLED_CONTROL_VERSION.as_bytes(),
                sections_absent_bytes,
                label_shuffled_bytes,
                bindings.partition.manifest_cid.as_bytes(),
            ],
        );
        let verdict = if shuffled_comparison.delta.numerator <= 0 {
            NegativeControlVerdict::Passed
        } else {
            NegativeControlVerdict::Failed
        };
        (
            comparison,
            NegativeControlEvidence {
                id: LABEL_SHUFFLED_CONTROL_ID.to_string(),
                identity_cid,
                verdict,
                comparison: Some(shuffled_comparison),
            },
        )
    } else {
        // Research estimates retain a same-position comparison against the
        // runtime's internal lane-absent candidate, but the absent planted
        // artifacts are explicitly UNAVAILABLE and can never produce PASS.
        let comparison = paired_comparison(
            "R4G1Runtime-internal-sections-absent",
            "internal-base-candidate/1",
            report_tagged_cid(b"r4-deployed-quality-internal-base-comparator/1", &[graph]),
            &bindings.partition.evaluated_positions_cid,
            row.normative_vs_base,
        )?;
        (
            comparison,
            NegativeControlEvidence {
                id: LABEL_SHUFFLED_CONTROL_ID.to_string(),
                identity_cid: report_tagged_cid(
                    b"r4-deployed-quality-label-shuffled-control-unavailable/1",
                    &[bindings.partition.manifest_cid.as_bytes()],
                ),
                verdict: NegativeControlVerdict::Unavailable,
                comparison: None,
            },
        )
    };

    let cross_surface_checks = cross_surface_evidence
        .checks
        .checked_add(row.internal_base_control_checks)
        .ok_or_else(|| SourceUnavailable::new("cross-surface check count overflow"))?;
    let cross_surface_mismatches = cross_surface_evidence
        .mismatches
        .checked_add(row.internal_base_control_mismatches)
        .ok_or_else(|| SourceUnavailable::new("cross-surface mismatch count overflow"))?;
    let measurements = QualityMeasurements {
        versus_tla,
        versus_sections_absent,
        internal_base_control_checks: row.internal_base_control_checks,
        internal_base_control_mismatches: row.internal_base_control_mismatches,
        cross_surface_checks,
        cross_surface_mismatches,
        cross_surface_evidence_cid: report_tagged_cid(
            b"r4-deployed-quality-cross-surface-evidence/1",
            &[
                &cross_surface_evidence.checks.to_le_bytes(),
                &cross_surface_evidence.mismatches.to_le_bytes(),
                &row.internal_base_control_checks.to_le_bytes(),
                &row.internal_base_control_mismatches.to_le_bytes(),
                cross_surface_evidence.graph_cid.as_bytes(),
                cross_surface_evidence.signature_artifact_cid.as_bytes(),
                &evidence.cross_surface_evidence,
            ],
        ),
    };
    let witness_replay = WitnessReplayEvidence {
        sample_cid: witness_artifact.sample_positions_cid.clone(),
        requested: witness_artifact.requested,
        replayed: witness_artifact.replayed,
        failures: witness_artifact.failures,
    };

    let gate_inputs = PromotionGateInputs {
        row,
        measurements: &measurements,
        label_control: &label_control,
        planted_controls_available,
        external_cross_surface_checks: cross_surface_evidence.checks,
        combined_cross_surface_mismatches: cross_surface_mismatches,
        witness_replayed: witness_artifact.replayed,
        witness_failures: witness_artifact.failures,
    };
    let promotion_failures = promotion_failures(gate_inputs);
    let (mode, verdict, measurements) = match row.mode {
        ServingEvalMode::Sample { .. } => (
            EvaluationMode::Sample,
            QualityVerdict::Estimate {
                decision: staged_sample_decision(gate_inputs).render(),
            },
            Some(measurements),
        ),
        ServingEvalMode::FullCensus if !planted_controls_available => (
            EvaluationMode::FullCensus,
            QualityVerdict::Unavailable {
                reason:
                    "required sections-absent and label-shuffled planted controls are unavailable"
                        .to_string(),
            },
            None,
        ),
        ServingEvalMode::FullCensus => {
            let verdict = if promotion_failures.is_empty() {
                QualityVerdict::Pass
            } else {
                QualityVerdict::Fail {
                    reason: promotion_failures.join("; "),
                }
            };
            (EvaluationMode::FullCensus, verdict, Some(measurements))
        }
    };
    let report = DeployedQualityReport {
        schema: DEPLOYED_QUALITY_REPORT_SCHEMA,
        profile: QualityProfileIdentity {
            id: DEPLOYED_QUALITY_PROFILE_ID.to_string(),
            version: DEPLOYED_QUALITY_PROFILE_VERSION,
            execution_scope: NORMATIVE_EXECUTION_SCOPE.to_string(),
        },
        bindings,
        evaluation: EvaluationEvidence {
            mode,
            population_size: row.population_n as u64,
            evaluated_positions: row.sample_n as u64,
            verdict,
            measurements,
        },
        witness_replay,
        negative_controls: vec![label_control],
    };
    if let Some(error) = report.validate_for_research() {
        return Err(SourceUnavailable::new(format!(
            "constructed deployed-quality report failed validation: {error}"
        )));
    }
    Ok(report)
}

#[derive(Clone, Copy)]
struct PromotionGateInputs<'a> {
    row: &'a ServingEvalRow,
    measurements: &'a QualityMeasurements,
    label_control: &'a NegativeControlEvidence,
    planted_controls_available: bool,
    external_cross_surface_checks: u64,
    combined_cross_surface_mismatches: u64,
    witness_replayed: u64,
    witness_failures: u64,
}

/// Typed outcome of the pre-registered sample funnel. It never represents a
/// production verdict: `Proceed` authorizes only the full census on the same
/// captured generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServingSampleDecision {
    Proceed,
    Stop {
        reasons: Vec<String>,
    },
    Inconclusive {
        reasons: Vec<String>,
        /// `Some(n)` requests the one predeclared non-census extension.
        /// `None` means that extension already ran; only a separately
        /// authorized full census can resolve the overlap.
        next_positions: Option<usize>,
    },
}

impl ServingSampleDecision {
    fn render(&self) -> String {
        match self {
            Self::Proceed => "PROCEED: binding sample satisfies every structural and paired lower-bound gate; run the full census on this exact evidence generation".to_string(),
            Self::Stop { reasons } => format!("STOP: {}", reasons.join("; ")),
            Self::Inconclusive {
                reasons,
                next_positions: Some(next_positions),
            } => format!(
                "INCONCLUSIVE: {}; extend this exact generation to {next_positions} positions with {DETERMINISTIC_SAMPLE_SELECTION_ALGORITHM}",
                reasons.join("; ")
            ),
            Self::Inconclusive {
                reasons,
                next_positions: None,
            } => format!(
                "INCONCLUSIVE: {}; the predeclared extension is complete, so only a separately authorized full census on this exact generation can resolve the overlapping interval",
                reasons.join("; ")
            ),
        }
    }
}

fn structural_failures(inputs: PromotionGateInputs<'_>) -> Vec<String> {
    let PromotionGateInputs {
        row,
        measurements: _,
        label_control,
        planted_controls_available,
        external_cross_surface_checks,
        combined_cross_surface_mismatches,
        witness_replayed,
        witness_failures,
    } = inputs;
    let mut failures = Vec::new();
    if row.lane_reachable == 0 {
        failures.push("SKMX/PSIB lane is unreachable on the evaluated positions".to_string());
    }
    if !planted_controls_available {
        failures.push(
            "required sections-absent and label-shuffled planted controls are unavailable"
                .to_string(),
        );
    } else {
        let expected_internal_checks = u64::try_from(row.sample_n).ok();
        if expected_internal_checks != Some(row.internal_base_control_checks)
            || row.internal_base_control_mismatches != 0
        {
            failures.push(format!(
                "internal sections-absent identity covered {}/{} evaluated positions with {} mismatches",
                row.internal_base_control_checks,
                row.sample_n,
                row.internal_base_control_mismatches
            ));
        }
        if label_control.verdict != NegativeControlVerdict::Passed
            || label_control
                .comparison
                .as_ref()
                .is_none_or(|comparison| comparison.delta.numerator > 0)
        {
            failures.push("label-shuffled planted control did not pass".to_string());
        }
    }
    if external_cross_surface_checks == 0 || combined_cross_surface_mismatches != 0 {
        failures.push(format!(
            "external cross-surface evidence has {external_cross_surface_checks} checks; combined evidence has {combined_cross_surface_mismatches} mismatches"
        ));
    }
    if witness_replayed == 0 || witness_failures != 0 {
        failures.push(format!(
            "witness replay has {witness_replayed} replayed and {witness_failures} failures"
        ));
    }
    failures
}

fn promotion_failures(inputs: PromotionGateInputs<'_>) -> Vec<String> {
    let mut failures = structural_failures(inputs);
    if inputs.measurements.versus_tla.interval.lower_delta_ppm < 0 {
        failures.push("paired lower bound versus TLA is below zero".to_string());
    }
    if inputs
        .measurements
        .versus_sections_absent
        .interval
        .lower_delta_ppm
        < RF31_MIN_LANE_DELTA_PPM
    {
        failures.push(format!(
            "paired lane lower bound is below {RF31_MIN_LANE_DELTA_PPM} ppm"
        ));
    }
    failures
}

fn staged_sample_decision(inputs: PromotionGateInputs<'_>) -> ServingSampleDecision {
    let structural = structural_failures(inputs);
    if !structural.is_empty() {
        return ServingSampleDecision::Stop {
            reasons: structural,
        };
    }

    let tla = &inputs.measurements.versus_tla.interval;
    let lane = &inputs.measurements.versus_sections_absent.interval;
    if tla.lower_delta_ppm >= 0 && lane.lower_delta_ppm >= RF31_MIN_LANE_DELTA_PPM {
        return ServingSampleDecision::Proceed;
    }

    let mut impossible = Vec::new();
    if tla.upper_delta_ppm < 0 {
        impossible.push(format!(
            "paired TLA upper bound {} ppm is below zero",
            tla.upper_delta_ppm
        ));
    }
    if lane.upper_delta_ppm < RF31_MIN_LANE_DELTA_PPM {
        impossible.push(format!(
            "paired lane upper bound {} ppm is below {RF31_MIN_LANE_DELTA_PPM} ppm",
            lane.upper_delta_ppm
        ));
    }
    let reachability_ceiling_ppm = if inputs.row.sample_n == 0 {
        0
    } else {
        ((u128::from(inputs.row.lane_reachable) * 1_000_000) / inputs.row.sample_n as u128) as i64
    };
    if reachability_ceiling_ppm < RF31_MIN_LANE_DELTA_PPM {
        impossible.push(format!(
            "lane reachability ceiling {reachability_ceiling_ppm} ppm is below {RF31_MIN_LANE_DELTA_PPM} ppm"
        ));
    }
    if !impossible.is_empty() {
        return ServingSampleDecision::Stop {
            reasons: impossible,
        };
    }

    let max_non_census = inputs.row.population_n.saturating_sub(1);
    let extension_target = EXTENDED_SAMPLE_TARGET.min(max_non_census);
    let next_positions = (inputs.row.sample_n < extension_target).then_some(extension_target);
    let mut reasons = Vec::new();
    if tla.lower_delta_ppm < 0 {
        reasons.push(format!(
            "paired TLA interval [{}, {}] ppm crosses zero",
            tla.lower_delta_ppm, tla.upper_delta_ppm
        ));
    }
    if lane.lower_delta_ppm < RF31_MIN_LANE_DELTA_PPM {
        reasons.push(format!(
            "paired lane interval [{}, {}] ppm crosses the {} ppm floor",
            lane.lower_delta_ppm, lane.upper_delta_ppm, RF31_MIN_LANE_DELTA_PPM
        ));
    }
    ServingSampleDecision::Inconclusive {
        reasons,
        next_positions,
    }
}

/// Run the evaluator with a durable append-only progress stream, then write a
/// deterministic report and a terminal summary. Errors and budget skips still
/// receive a terminal record; they never emit a production-admissible PASS.
pub fn evaluate_serving_bundle_recorded(
    bundle: &ServingBundle,
    budgets: ServingEvalBudgets,
    evidence: &ServingReportEvidence,
    paths: &ServingReportPaths,
    progress: &mut dyn FnMut(ServingProgress),
) -> Result<RecordedServingEval, SourceUnavailable> {
    let snapshot = ServingBundleSnapshot::capture(bundle)?;
    evaluate_serving_snapshot_recorded(&snapshot, budgets, evidence, paths, progress)
}

/// Recorded evaluation over one caller-captured immutable generation. This is
/// the production orchestration seam: witness creation, measurement, report
/// construction, and terminal evidence can all share the same snapshot.
pub fn evaluate_serving_snapshot_recorded(
    snapshot: &ServingBundleSnapshot,
    budgets: ServingEvalBudgets,
    evidence: &ServingReportEvidence,
    paths: &ServingReportPaths,
    progress: &mut dyn FnMut(ServingProgress),
) -> Result<RecordedServingEval, SourceUnavailable> {
    let mut progress_file = create_progress_file(&paths.progress_jsonl)?;
    let cross_surface_evidence = match validate_snapshot_cross_surface_evidence(snapshot, evidence)
    {
        Ok(evidence) => evidence,
        Err(error) => {
            write_terminal(
                &paths.terminal_json,
                serde_json::json!({
                    "schema": REPORT_TERMINAL_SCHEMA,
                    "status": "unavailable",
                    "reason": error.to_string(),
                    "generation_cid": snapshot.generation_cid,
                    "evidence_hooks": terminal_evidence_hooks(evidence, None, None, None),
                    "report_emitted": false,
                }),
            )?;
            return Err(error);
        }
    };
    let evidence_hooks =
        terminal_evidence_hooks(evidence, Some(&cross_surface_evidence), None, None);
    let mut progress_error: Option<String> = None;
    let outcome = {
        let mut recorded_progress = |state: ServingProgress| {
            progress(state);
            if progress_error.is_none() {
                if let Err(error) = append_progress(&mut progress_file, &state) {
                    progress_error = Some(error.to_string());
                }
            }
        };
        evaluate_serving_snapshot(snapshot, budgets, &mut recorded_progress)
    };
    if let Some(error) = progress_error {
        write_terminal(
            &paths.terminal_json,
            serde_json::json!({
                "schema": REPORT_TERMINAL_SCHEMA,
                "status": "unavailable",
                "reason": format!("durable progress write failed: {error}"),
                "generation_cid": snapshot.generation_cid,
                "evidence_hooks": evidence_hooks,
                "report_emitted": false,
            }),
        )?;
        return Err(SourceUnavailable::new(format!(
            "durable progress write failed: {error}"
        )));
    }
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(error) => {
            write_terminal(
                &paths.terminal_json,
                serde_json::json!({
                    "schema": REPORT_TERMINAL_SCHEMA,
                    "status": "unavailable",
                    "reason": error.to_string(),
                    "generation_cid": snapshot.generation_cid,
                    "evidence_hooks": evidence_hooks,
                    "report_emitted": false,
                }),
            )?;
            return Err(error);
        }
    };
    match &outcome {
        ServingEvalOutcome::Skipped(skip) => {
            write_terminal(
                &paths.terminal_json,
                serde_json::json!({
                    "schema": REPORT_TERMINAL_SCHEMA,
                    "status": "skipped",
                    "reason": skip_reason(skip),
                    "generation_cid": snapshot.generation_cid,
                    "evidence_hooks": evidence_hooks,
                    "report_emitted": false,
                }),
            )?;
            Ok(RecordedServingEval {
                outcome,
                report: None,
                report_cid: None,
            })
        }
        ServingEvalOutcome::Row(row) => {
            let decision_attribution_cid = row
                .decision_attribution
                .cid(&row.generation_cid, &row.evaluated_positions_cid)
                .map_err(|error| {
                    SourceUnavailable::new(format!("serialize decision attribution: {error}"))
                })?;
            let report = match build_deployed_quality_report_from_snapshot(snapshot, row, evidence)
            {
                Ok(report) => report,
                Err(error) => {
                    write_terminal(
                        &paths.terminal_json,
                        serde_json::json!({
                            "schema": REPORT_TERMINAL_SCHEMA,
                            "status": "unavailable",
                            "reason": error.to_string(),
                            "generation_cid": snapshot.generation_cid,
                            "evaluation": row,
                            "decision_attribution_cid": decision_attribution_cid,
                            "evidence_hooks": evidence_hooks,
                            "report_emitted": false,
                        }),
                    )?;
                    return Err(error);
                }
            };
            let report_bytes = report
                .deterministic_json_bytes()
                .map_err(|error| SourceUnavailable::new(format!("serialize report: {error}")))?;
            write_atomic(&paths.deployed_quality_json, &report_bytes)?;
            let report_cid = format!("blake3:{}", blake3::hash(&report_bytes).to_hex());
            let report_cross_surface_evidence_cid = report
                .evaluation
                .measurements
                .as_ref()
                .map(|measurements| measurements.cross_surface_evidence_cid.as_str());
            let evidence_hooks = terminal_evidence_hooks(
                evidence,
                Some(&cross_surface_evidence),
                report_cross_surface_evidence_cid,
                Some(&report.witness_replay),
            );
            let terminal_status = if matches!(
                report.evaluation.verdict,
                QualityVerdict::Unavailable { .. }
            ) {
                "unavailable"
            } else {
                "complete"
            };
            write_terminal(
                &paths.terminal_json,
                serde_json::json!({
                    "schema": REPORT_TERMINAL_SCHEMA,
                    "status": terminal_status,
                    "generation_cid": snapshot.generation_cid,
                    "evaluation": row,
                    "decision_attribution_cid": decision_attribution_cid,
                    "quality_verdict": &report.evaluation.verdict,
                    "report_path": paths.deployed_quality_json,
                    "report_cid": report_cid,
                    "evidence_hooks": evidence_hooks,
                    "report_emitted": true,
                }),
            )?;
            Ok(RecordedServingEval {
                outcome,
                report: Some(report),
                report_cid: Some(report_cid),
            })
        }
    }
}

fn terminal_evidence_hooks(
    evidence: &ServingReportEvidence,
    cross_surface_evidence: Option<&CrossSurfaceParityEvidence>,
    report_cross_surface_evidence_cid: Option<&str>,
    witness_replay: Option<&WitnessReplayEvidence>,
) -> serde_json::Value {
    let external_cross_surface_evidence_cid = report_tagged_cid(
        b"r4-deployed-quality-external-cross-surface-evidence/1",
        &[&evidence.cross_surface_evidence],
    );
    let external_witness_replay_evidence_cid = report_tagged_cid(
        b"r4-deployed-quality-external-witness-replay-evidence/1",
        &[&evidence.witness_replay_evidence],
    );
    serde_json::json!({
        "compiler_revision": evidence.compiler_revision,
        "external_cross_surface_evidence_cid": external_cross_surface_evidence_cid,
        "report_cross_surface_evidence_cid": report_cross_surface_evidence_cid,
        "cross_surface_validated": cross_surface_evidence.is_some(),
        "cross_surface_graph_cid": cross_surface_evidence.map(|artifact| artifact.graph_cid.as_str()),
        "cross_surface_signature_artifact_cid": cross_surface_evidence.map(|artifact| artifact.signature_artifact_cid.as_str()),
        "cross_surface_tokenizer_cid": cross_surface_evidence.and_then(|artifact| artifact.tokenizer_cid.as_deref()),
        "cross_surface_checks": cross_surface_evidence.map(|artifact| artifact.checks),
        "cross_surface_mismatches": cross_surface_evidence.map(|artifact| artifact.mismatches),
        "external_witness_replay_evidence_cid": external_witness_replay_evidence_cid,
        "witness_validated": witness_replay.is_some(),
        "witness_sample_cid": witness_replay.map(|artifact| artifact.sample_cid.as_str()),
        "witness_requested": witness_replay.map(|artifact| artifact.requested),
        "witness_replayed": witness_replay.map(|artifact| artifact.replayed),
        "witness_failures": witness_replay.map(|artifact| artifact.failures),
    })
}

fn validate_snapshot_cross_surface_evidence(
    snapshot: &ServingBundleSnapshot,
    evidence: &ServingReportEvidence,
) -> Result<CrossSurfaceParityEvidence, SourceUnavailable> {
    let tokenizer = required_snapshot_component("tokenizer", snapshot.tokenizer.as_deref())?;
    let score_report =
        required_snapshot_component("score report", snapshot.score_report.as_deref())?;
    evidence.validated_cross_surface_evidence(
        &snapshot.graph,
        &snapshot.teacher,
        tokenizer,
        score_report,
    )
}

fn paired_comparison(
    id: &str,
    version: &str,
    definition_cid: String,
    positions_cid: &str,
    counts: PairedCounts,
) -> Result<PairedComparison, SourceUnavailable> {
    let counts = QualityPairedCounts {
        both_correct: counts.both,
        selector_only_correct: counts.normative_only,
        comparator_only_correct: counts.comparator_only,
        neither_correct: counts.neither,
    };
    let denominator = counts
        .both_correct
        .checked_add(counts.selector_only_correct)
        .and_then(|value| value.checked_add(counts.comparator_only_correct))
        .and_then(|value| value.checked_add(counts.neither_correct))
        .ok_or_else(|| SourceUnavailable::new("paired counts overflow"))?;
    if denominator == 0 {
        return Err(SourceUnavailable::new("paired comparison is empty"));
    }
    let selector_hits = counts
        .both_correct
        .checked_add(counts.selector_only_correct)
        .ok_or_else(|| SourceUnavailable::new("selector hit count overflow"))?;
    let comparator_hits = counts
        .both_correct
        .checked_add(counts.comparator_only_correct)
        .ok_or_else(|| SourceUnavailable::new("comparator hit count overflow"))?;
    let delta_numerator = i64::try_from(counts.selector_only_correct)
        .ok()
        .and_then(|selector| {
            i64::try_from(counts.comparator_only_correct)
                .ok()
                .and_then(|comparator| selector.checked_sub(comparator))
        })
        .ok_or_else(|| SourceUnavailable::new("paired delta exceeds i64"))?;
    let exact_rate = |numerator: u64| ExactRate {
        numerator,
        denominator,
        ppm: ((u128::from(numerator) * 1_000_000) / u128::from(denominator)) as u32,
    };
    let delta = ExactSignedRate {
        numerator: delta_numerator,
        denominator,
        ppm: ((i128::from(delta_numerator) * 1_000_000) / i128::from(denominator)) as i64,
    };
    let interval = PairedInterval::from_counts(counts).ok_or_else(|| {
        SourceUnavailable::new("paired counts cannot produce the fixed-point interval")
    })?;
    Ok(PairedComparison {
        comparator: ComparatorIdentity {
            id: id.to_string(),
            version: version.to_string(),
            definition_cid,
            positions_cid: positions_cid.to_string(),
        },
        counts,
        selector_rate: exact_rate(selector_hits),
        comparator_rate: exact_rate(comparator_hits),
        delta,
        interval,
    })
}

fn usize_positions_to_u64(positions: &[usize]) -> Result<Vec<u64>, SourceUnavailable> {
    positions
        .iter()
        .map(|&position| {
            u64::try_from(position)
                .map_err(|_| SourceUnavailable::new("corpus position exceeds u64"))
        })
        .collect()
}

fn required_snapshot_component<'a>(
    field: &str,
    bytes: Option<&'a [u8]>,
) -> Result<&'a [u8], SourceUnavailable> {
    bytes.ok_or_else(|| {
        SourceUnavailable::new(format!(
            "deployed-quality {field} is UNAVAILABLE in captured generation"
        ))
    })
}

fn serving_generation_cid(components: &[(&str, Option<&[u8]>)]) -> String {
    let mut hasher = blake3::Hasher::new();
    let tag = b"r4-deployed-quality-serving-generation/1";
    hasher.update(&(tag.len() as u64).to_le_bytes());
    hasher.update(tag);
    for (name, bytes) in components {
        hasher.update(&(name.len() as u64).to_le_bytes());
        hasher.update(name.as_bytes());
        match bytes {
            Some(bytes) => {
                hasher.update(&[1]);
                hasher.update(&(bytes.len() as u64).to_le_bytes());
                hasher.update(bytes);
            }
            None => {
                hasher.update(&[0]);
            }
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn report_tagged_cid(tag: &[u8], parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(tag.len() as u64).to_le_bytes());
    hasher.update(tag);
    for part in parts {
        hasher.update(&(part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn validate_planted_control_graphs(
    main: &[u8],
    canonical_base: Option<&[u8]>,
    sections_absent: Option<&[u8]>,
    label_shuffled: Option<&[u8]>,
) -> Result<(), SourceUnavailable> {
    use uor_r4_graph_format::SectionId;

    let main_view = parse_control_graph("main", main)?;
    if main_view.section(SectionId::SKMX).is_none() || main_view.section(SectionId::PSIB).is_none()
    {
        return Err(SourceUnavailable::new(
            "main graph lacks required SKMX/PSIB sections",
        ));
    }
    if let Some(bytes) = canonical_base {
        let canonical_view = parse_control_graph("pre-re-emission canonical", bytes)?;
        if canonical_view.section(SectionId::SKMX).is_some()
            || canonical_view.section(SectionId::PSIB).is_some()
        {
            return Err(SourceUnavailable::new(
                "pre-re-emission canonical diagnostic contains SKMX or PSIB",
            ));
        }
    }
    if let Some(bytes) = sections_absent {
        let absent_view = parse_control_graph("sections-absent", bytes)?;
        if absent_view.section(SectionId::SKMX).is_some()
            || absent_view.section(SectionId::PSIB).is_some()
        {
            return Err(SourceUnavailable::new(
                "sections-absent control still contains SKMX or PSIB",
            ));
        }
        require_non_lane_identity(&main_view, &absent_view, "sections-absent")?;
    }
    if let Some(bytes) = label_shuffled {
        let shuffled_view = parse_control_graph("label-shuffled", bytes)?;
        if shuffled_view.section(SectionId::SKMX).is_none()
            || shuffled_view.section(SectionId::PSIB).is_none()
        {
            return Err(SourceUnavailable::new(
                "label-shuffled control lacks SKMX or PSIB",
            ));
        }
        require_non_lane_identity(&main_view, &shuffled_view, "label-shuffled")?;
        let lane_differs = [SectionId::SKMX, SectionId::PSIB]
            .iter()
            .any(|&id| main_view.section(id) != shuffled_view.section(id));
        if !lane_differs {
            return Err(SourceUnavailable::new(
                "label-shuffled control is byte-identical to the main lane",
            ));
        }
    }
    Ok(())
}

fn parse_control_graph<'a>(
    label: &str,
    bytes: &'a [u8],
) -> Result<uor_r4_graph_format::GraphView<'a>, SourceUnavailable> {
    let view = uor_r4_graph_format::GraphView::parse(bytes)
        .map_err(|error| SourceUnavailable::new(format!("{label} graph parse failed: {error}")))?;
    view.verify_cids().map_err(|error| {
        SourceUnavailable::new(format!("{label} graph CID check failed: {error}"))
    })?;
    Ok(view)
}

fn require_non_lane_identity(
    main: &uor_r4_graph_format::GraphView<'_>,
    control: &uor_r4_graph_format::GraphView<'_>,
    label: &str,
) -> Result<(), SourceUnavailable> {
    use uor_r4_graph_format::SectionId;

    for section in main.sections() {
        if section.id == SectionId::SKMX || section.id == SectionId::PSIB {
            continue;
        }
        let Some(control_section) = control.sections().find(|row| row.id == section.id) else {
            return Err(SourceUnavailable::new(format!(
                "{label} control is missing non-lane section 0x{:08x}",
                section.id.raw()
            )));
        };
        if control_section.flags != section.flags || control_section.payload != section.payload {
            return Err(SourceUnavailable::new(format!(
                "{label} control changes non-lane section 0x{:08x}",
                section.id.raw()
            )));
        }
    }
    for section in control.sections() {
        if section.id == SectionId::SKMX || section.id == SectionId::PSIB {
            continue;
        }
        if main.section(section.id).is_none() {
            return Err(SourceUnavailable::new(format!(
                "{label} control adds non-lane section 0x{:08x}",
                section.id.raw()
            )));
        }
    }
    Ok(())
}

fn create_progress_file(path: &Path) -> Result<File, SourceUnavailable> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            SourceUnavailable::new(format!(
                "create progress directory {}: {error}",
                parent.display()
            ))
        })?;
    }
    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "open progress artifact {}: {error}",
                path.display()
            ))
        })
}

fn append_progress(file: &mut File, state: &ServingProgress) -> Result<(), SourceUnavailable> {
    let record = serde_json::json!({
        "schema": REPORT_PROGRESS_SCHEMA,
        "progress": state,
    });
    serde_json::to_writer(&mut *file, &record)
        .map_err(|error| SourceUnavailable::new(format!("serialize progress record: {error}")))?;
    file.write_all(b"\n")
        .map_err(|error| SourceUnavailable::new(format!("append progress record: {error}")))?;
    file.flush()
        .map_err(|error| SourceUnavailable::new(format!("flush progress record: {error}")))?;
    file.sync_data()
        .map_err(|error| SourceUnavailable::new(format!("sync progress record: {error}")))
}

fn write_terminal(path: &Path, value: serde_json::Value) -> Result<(), SourceUnavailable> {
    let mut bytes = serde_json::to_vec_pretty(&value)
        .map_err(|error| SourceUnavailable::new(format!("serialize terminal record: {error}")))?;
    bytes.push(b'\n');
    write_atomic(path, &bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), SourceUnavailable> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            SourceUnavailable::new(format!(
                "create artifact directory {}: {error}",
                parent.display()
            ))
        })?;
    }
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("artifact");
    let temporary = path.with_extension(format!("{extension}.tmp-{}", std::process::id()));
    let write_result = (|| {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.flush()?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)
    })();
    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&temporary);
        return Err(SourceUnavailable::new(format!(
            "write durable artifact {}: {error}",
            path.display()
        )));
    }
    Ok(())
}

fn skip_reason(skip: &ServingEvalSkip) -> String {
    match skip {
        ServingEvalSkip::ProbeBudgetExceeded { probed, elapsed } => format!(
            "probe budget exceeded after {probed} positions and {} ms",
            elapsed.as_millis()
        ),
        ServingEvalSkip::ProbeFunctionalCheckFailed { served, probed } => format!(
            "probe functional check failed: {served} served across {probed} positions with zero hits"
        ),
        ServingEvalSkip::EvalBudgetExceeded {
            done,
            sample_n,
            elapsed,
        } => format!(
            "evaluation budget exceeded after {done}/{sample_n} positions and {} ms; partial counts discarded",
            elapsed.as_millis()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control_graph_with_lane(candidate: Option<u32>) -> Vec<u8> {
        use uor_r4_graph_format::{
            build_psi_bag_table, build_skipmix_table, ArtifactBuilder, SectionId,
        };

        let mut head = Vec::with_capacity(uor_r4_graph_format::HEAD_PAYLOAD_LEN);
        head.extend_from_slice(&[0x11; 32]);
        head.extend_from_slice(&[0x22; 32]);
        head.extend_from_slice(&[0x33; 32]);
        head.extend_from_slice(&[0x44; 32]);
        head.extend_from_slice(b"0123456789abcdef0123");
        head.extend_from_slice(&[0x55; 32]);
        head.extend_from_slice(&32u16.to_le_bytes());
        head.extend_from_slice(&16u16.to_le_bytes());
        head.extend_from_slice(&8u16.to_le_bytes());
        head.extend_from_slice(&8u16.to_le_bytes());
        head.extend_from_slice(&64u32.to_le_bytes());
        head.extend_from_slice(&64u32.to_le_bytes());
        head.extend_from_slice(&0u32.to_le_bytes());
        head.extend_from_slice(&0u32.to_le_bytes());
        head.push(1);
        head.extend_from_slice(&[0; 7]);
        head.extend_from_slice(&64u16.to_le_bytes());
        head.extend_from_slice(&1u16.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.extend_from_slice(&100u32.to_le_bytes());
        assert_eq!(head.len(), uor_r4_graph_format::HEAD_PAYLOAD_LEN);

        let mut builder = ArtifactBuilder::new(3);
        builder.add_section(SectionId::HEAD, 0, &head);
        if let Some(candidate) = candidate {
            let skmx =
                build_skipmix_table(&[(1, 2, vec![(candidate, 3)])]).expect("test SKMX table");
            let psib = build_psi_bag_table(&[(1, vec![(candidate, 4)])]).expect("test PSIB table");
            builder.add_section(SectionId::SKMX, 0, &skmx);
            builder.add_section(SectionId::PSIB, 0, &psib);
        }
        builder.build().expect("test control graph")
    }

    #[test]
    fn pre_reemission_diagnostic_must_really_be_sections_absent() {
        let main = control_graph_with_lane(Some(3));
        let absent = control_graph_with_lane(None);
        let shuffled = control_graph_with_lane(Some(4));
        validate_planted_control_graphs(&main, Some(&absent), Some(&absent), Some(&shuffled))
            .expect("sections-absent diagnostic is valid");

        let error =
            validate_planted_control_graphs(&main, Some(&main), Some(&absent), Some(&shuffled))
                .expect_err("lane-bearing graph cannot be labelled pre-re-emission absent");
        assert!(error.reason.contains("canonical diagnostic contains SKMX"));
    }

    #[test]
    fn discover_requires_all_bundle_files() {
        let root = std::env::temp_dir().join(format!("r4-serving-eval-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("graph")).unwrap();
        assert!(ServingBundle::discover(&root).is_none());
        for name in [
            "tless_artifacts.bin",
            "tless_store.bin",
            "corpus.meta",
            "corpus.records",
        ] {
            std::fs::write(root.join(name), b"x").unwrap();
        }
        assert!(
            ServingBundle::discover(&root).is_none(),
            "no graph artifact yet"
        );
        std::fs::write(root.join("graph").join("score.r4g1"), b"x").unwrap();
        let bundle = ServingBundle::discover(&root).expect("bundle");
        assert_eq!(bundle.graph, root.join("graph").join("score.r4g1"));
        assert_eq!(bundle.teacher, root.join("tless_artifacts.bin"));
        assert_eq!(bundle.store, root.join("tless_store.bin"));
        assert!(bundle.canonical_base_graph.is_none());
        assert!(bundle.sections_absent_graph.is_none());
        assert!(bundle.label_shuffled_graph.is_none());
        std::fs::write(
            root.join("graph").join("score_canonical_base.r4g1"),
            b"canonical-diagnostic",
        )
        .unwrap();
        std::fs::write(
            root.join("graph").join("score_sections_absent.r4g1"),
            b"control",
        )
        .unwrap();
        std::fs::write(
            root.join("graph").join("score_label_shuffled.r4g1"),
            b"control",
        )
        .unwrap();
        std::fs::write(root.join("tokenizer.bin"), b"captured-tokenizer").unwrap();
        let with_controls = ServingBundle::discover(&root).expect("bundle with controls");
        assert!(with_controls.canonical_base_graph.is_some());
        assert!(with_controls.sections_absent_graph.is_some());
        assert!(with_controls.label_shuffled_graph.is_some());
        let first = ServingBundleSnapshot::capture(&with_controls).expect("capture generation");
        assert_eq!(first.generation_cid().len(), "blake3:".len() + 64);
        let parts = serving_engine_parts(&first, first.graph.as_slice(), None);
        assert_eq!(
            parts.tokenizer,
            Some(b"captured-tokenizer".as_slice()),
            "every probe, worker, and control engine must receive the tokenizer captured in the immutable generation"
        );
        std::fs::write(
            root.join("graph").join("score_canonical_base.r4g1"),
            b"changed-canonical-diagnostic",
        )
        .unwrap();
        let diagnostic_changed =
            ServingBundleSnapshot::capture(&with_controls).expect("recapture diagnostic");
        assert_ne!(
            first.generation_cid(),
            diagnostic_changed.generation_cid(),
            "a diagnostic-arm byte change must create a different generation"
        );
        std::fs::write(root.join("tless_store.bin"), b"changed").unwrap();
        let second = ServingBundleSnapshot::capture(&with_controls).expect("recapture generation");
        assert_ne!(
            first.generation_cid(),
            second.generation_cid(),
            "a comparator-byte change must create a different generation"
        );
        let error = second
            .require_generation(first.generation_cid())
            .expect_err("a changed full-census capture must be refused");
        assert!(error.to_string().contains("full census was not launched"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn abstain_breakdown_records_by_status() {
        let mut b = StatusBreakdown::default();
        b.record(PolicyStatus::Novel, false);
        b.record(PolicyStatus::Novel, false);
        b.record(PolicyStatus::Graph, false);
        assert_eq!(b.novel, 2);
        assert_eq!(b.graph, 1);
        assert_eq!(b.total(), 3);
    }

    /// #362 attribution: the NGRAM subcount tracks only exact-context
    /// records, and the flag is ignored on other statuses.
    #[test]
    fn breakdown_splits_exact_context_by_ngram() {
        let mut b = StatusBreakdown::default();
        b.record(PolicyStatus::ExactContext, true);
        b.record(PolicyStatus::ExactContext, false);
        b.record(PolicyStatus::Graph, true);
        assert_eq!(b.exact_context, 2);
        assert_eq!(b.exact_context_ngram, 1);
        assert_eq!(b.graph, 1);
        assert_eq!(b.total(), 3, "the ngram split is not a fourth bucket");
    }

    #[test]
    fn sample_is_story_distributed_and_canonically_sorted() {
        let held_out: Vec<usize> = (100..1_100).collect();
        let story: Vec<u32> = (0..1_100).map(|position| position / 100).collect();
        let selection = select_serving_eval_positions(
            &story,
            &held_out,
            ServingEvalMode::Sample { positions: 7 },
        )
        .expect("selection");
        assert_eq!(selection.positions.len(), 7);
        assert!(selection.positions.windows(2).all(|pair| pair[0] < pair[1]));
        let selected_stories: Vec<_> = selection
            .positions
            .iter()
            .map(|&position| story[position])
            .collect();
        assert_eq!(selected_stories, vec![1, 2, 3, 5, 6, 8, 9]);
        assert_eq!(
            selection.algorithm,
            DETERMINISTIC_SAMPLE_SELECTION_ALGORITHM
        );
    }

    #[test]
    fn six_thousand_position_sample_is_nested_in_eighteen_thousand_extension() {
        const POPULATION: usize = 24_000;
        const STORY_LEN: usize = 6_000;
        let held_out: Vec<usize> = (0..POPULATION).collect();
        let story: Vec<u32> = (0..POPULATION)
            .map(|position| (position / STORY_LEN) as u32)
            .collect();
        let screen = select_serving_eval_positions(
            &story,
            &held_out,
            ServingEvalMode::Sample {
                positions: SAMPLE_TARGET,
            },
        )
        .expect("screen");
        let extension = select_serving_eval_positions(
            &story,
            &held_out,
            ServingEvalMode::Sample {
                positions: EXTENDED_SAMPLE_TARGET,
            },
        )
        .expect("extension");
        assert_eq!(screen.positions.len(), SAMPLE_TARGET);
        assert_eq!(extension.positions.len(), EXTENDED_SAMPLE_TARGET);
        assert!(screen
            .positions
            .iter()
            .all(|position| extension.positions.binary_search(position).is_ok()));
        for story_id in 0..4 {
            assert_eq!(
                screen
                    .positions
                    .iter()
                    .filter(|&&position| story[position] == story_id)
                    .count(),
                SAMPLE_TARGET / 4
            );
            assert_eq!(
                extension
                    .positions
                    .iter()
                    .filter(|&&position| story[position] == story_id)
                    .count(),
                EXTENDED_SAMPLE_TARGET / 4
            );
        }
        assert_eq!(
            screen,
            select_serving_eval_positions(
                &story,
                &held_out,
                ServingEvalMode::Sample {
                    positions: SAMPLE_TARGET,
                },
            )
            .expect("deterministic screen")
        );
    }

    #[test]
    fn attribution_rows_reproduce_aggregate_and_bind_generation_and_positions() {
        let base = DecisionAttributionDimensions {
            status: AttributionStatus::Graph,
            disposition: AttributionDisposition::Served,
            normative_tla_cell: NormativeTlaCell::NormativeOnlyCorrect,
            canonical_base_candidates_evaluated: true,
            canonical_base_target_rank: None,
            normative_candidates_evaluated: true,
            normative_target_rank: Some(1),
            sections_absent_candidates_evaluated: true,
            sections_absent_target_rank: None,
            target_source: Some(AttributionCandidateSource::Skipmix),
            target_skmx_contributed: true,
            target_psib_contributed: false,
            lane_transition: LaneTransition::TowardTarget,
        };
        let mut evidence = DecisionAttributionEvidence::default();
        evidence.record(PositionDecisionAttribution {
            position: 10,
            dimensions: base,
        });
        evidence.record(PositionDecisionAttribution {
            position: 20,
            dimensions: DecisionAttributionDimensions {
                normative_tla_cell: NormativeTlaCell::TlaOnlyCorrect,
                normative_target_rank: None,
                target_source: None,
                target_skmx_contributed: false,
                lane_transition: LaneTransition::AwayFromTarget,
                ..base
            },
        });
        evidence.finalize();
        assert_eq!(evidence.cells.len(), 2);
        assert!(evidence
            .validate(
                &[10, 20],
                PairedCounts {
                    both: 0,
                    normative_only: 1,
                    comparator_only: 1,
                    neither: 0,
                },
                2,
                1,
                1,
            )
            .is_ok());
        let first = evidence
            .cid("blake3:generation-a", "blake3:positions")
            .expect("cid");
        let second = evidence
            .cid("blake3:generation-b", "blake3:positions")
            .expect("cid");
        assert_ne!(first, second);

        evidence.cells[0].count += 1;
        assert!(evidence
            .validate(
                &[10, 20],
                PairedCounts {
                    both: 0,
                    normative_only: 1,
                    comparator_only: 1,
                    neither: 0,
                },
                2,
                1,
                1,
            )
            .expect_err("tampered aggregate rejected")
            .reason
            .contains("do not reproduce"));
    }

    #[test]
    fn durable_progress_is_newline_delimited_and_parseable() {
        let root = std::env::temp_dir().join(format!("r4-serving-progress-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let path = root.join("progress.jsonl");
        let mut file = create_progress_file(&path).expect("progress file");
        let progress = ServingProgress {
            phase: "evaluate",
            processed: 256,
            total: 1_000,
            served: 200,
            abstained: 50,
            declined: 6,
            normative_hits: 75,
            tla_hits: 70,
            lane_reachable: 40,
            lane_changed: 20,
            lane_toward: 15,
            lane_away: 5,
            sections_absent_hits: 60,
            label_shuffled_hits: 55,
            internal_base_control_checks: 256,
            internal_base_control_mismatches: 0,
            planted_controls_available: true,
            elapsed_millis: 1_000,
            positions_per_second_milli: 256_000,
            eta_seconds: Some(3),
            workers: 8,
        };
        append_progress(&mut file, &progress).expect("append");
        drop(file);
        let bytes = std::fs::read(&path).expect("read");
        assert_eq!(bytes.last(), Some(&b'\n'));
        let line = bytes.strip_suffix(b"\n").expect("newline");
        let value: serde_json::Value = serde_json::from_slice(line).expect("JSON line");
        assert_eq!(value["schema"], REPORT_PROGRESS_SCHEMA);
        assert_eq!(value["progress"]["workers"], 8);
        let _ = std::fs::remove_dir_all(&root);
    }

    fn gate_comparison(lower_delta_ppm: i64, delta_numerator: i64) -> PairedComparison {
        let counts = QualityPairedCounts {
            both_correct: 70,
            selector_only_correct: u64::try_from(delta_numerator.max(0)).unwrap(),
            comparator_only_correct: u64::try_from((-delta_numerator).max(0)).unwrap(),
            neither_correct: 30,
        };
        let denominator = counts.both_correct
            + counts.selector_only_correct
            + counts.comparator_only_correct
            + counts.neither_correct;
        PairedComparison {
            comparator: ComparatorIdentity {
                id: "fixture".to_owned(),
                version: "1".to_owned(),
                definition_cid: "blake3:fixture".to_owned(),
                positions_cid: "blake3:positions".to_owned(),
            },
            counts,
            selector_rate: ExactRate {
                numerator: 70 + counts.selector_only_correct,
                denominator,
                ppm: 0,
            },
            comparator_rate: ExactRate {
                numerator: 70 + counts.comparator_only_correct,
                denominator,
                ppm: 0,
            },
            delta: ExactSignedRate {
                numerator: delta_numerator,
                denominator,
                ppm: 0,
            },
            interval: PairedInterval {
                method: "fixture".to_owned(),
                confidence_ppm: 950_000,
                lower_delta_ppm,
                estimate_delta_ppm: delta_numerator,
                upper_delta_ppm: lower_delta_ppm.saturating_add(1),
            },
        }
    }

    fn gate_row() -> ServingEvalRow {
        ServingEvalRow {
            bundle: PathBuf::from("fixture"),
            generation_cid: "blake3:generation".to_owned(),
            sample_n: SAMPLE_TARGET,
            population_n: 72_130,
            mode: ServingEvalMode::Sample {
                positions: SAMPLE_TARGET,
            },
            position_selection_algorithm: DETERMINISTIC_SAMPLE_SELECTION_ALGORITHM.to_string(),
            workers: 8,
            elapsed_millis: 1,
            served: SAMPLE_TARGET as u64,
            served_widened: 0,
            served_by: StatusBreakdown::default(),
            abstained: StatusBreakdown::default(),
            top1_served: 0,
            agree_served: 0,
            tla_hits: 0,
            base_hits: 0,
            normative_vs_tla: PairedCounts::default(),
            normative_vs_base: PairedCounts::default(),
            canonical_base_vs_sections_absent: None,
            canonical_base_hits: 0,
            normative_vs_sections_absent: Some(PairedCounts::default()),
            label_shuffled_vs_sections_absent: Some(PairedCounts::default()),
            sections_absent_hits: 0,
            label_shuffled_hits: 0,
            internal_base_control_checks: SAMPLE_TARGET as u64,
            internal_base_control_mismatches: 0,
            lane_reachable: 1_000,
            lane_changed: 100,
            lane_toward: 100,
            lane_away: 0,
            decision_attribution: DecisionAttributionEvidence::default(),
            declined: 0,
            probe_positions: 256,
            probe_served: 256,
            probe_hits: 1,
            evaluated_positions_cid: "blake3:evaluated".to_owned(),
            population_positions_cid: "blake3:population".to_owned(),
        }
    }

    #[test]
    fn binding_sample_gate_proceeds_only_when_every_falsifier_passes() {
        let row = gate_row();
        let measurements = QualityMeasurements {
            versus_tla: gate_comparison(0, 1),
            versus_sections_absent: gate_comparison(RF31_MIN_LANE_DELTA_PPM, 1),
            internal_base_control_checks: SAMPLE_TARGET as u64,
            internal_base_control_mismatches: 0,
            cross_surface_checks: SAMPLE_TARGET as u64 + 6,
            cross_surface_mismatches: 0,
            cross_surface_evidence_cid: "blake3:evidence".to_owned(),
        };
        let label_control = NegativeControlEvidence {
            id: LABEL_SHUFFLED_CONTROL_ID.to_owned(),
            identity_cid: "blake3:control".to_owned(),
            verdict: NegativeControlVerdict::Passed,
            comparison: Some(gate_comparison(-1, 0)),
        };
        let inputs = PromotionGateInputs {
            row: &row,
            measurements: &measurements,
            label_control: &label_control,
            planted_controls_available: true,
            external_cross_surface_checks: 6,
            combined_cross_surface_mismatches: 0,
            witness_replayed: 64,
            witness_failures: 0,
        };
        assert!(promotion_failures(inputs).is_empty());
        assert_eq!(
            staged_sample_decision(inputs),
            ServingSampleDecision::Proceed
        );
    }

    #[test]
    fn pre_reemission_canonical_arm_is_diagnostic_only() {
        let mut row = gate_row();
        row.canonical_base_vs_sections_absent = Some(PairedCounts {
            both: 0,
            normative_only: SAMPLE_TARGET as u64,
            comparator_only: 0,
            neither: 0,
        });
        row.canonical_base_hits = SAMPLE_TARGET as u64;
        let measurements = QualityMeasurements {
            versus_tla: gate_comparison(0, 1),
            versus_sections_absent: gate_comparison(RF31_MIN_LANE_DELTA_PPM, 1),
            internal_base_control_checks: SAMPLE_TARGET as u64,
            internal_base_control_mismatches: 0,
            cross_surface_checks: SAMPLE_TARGET as u64 + 6,
            cross_surface_mismatches: 0,
            cross_surface_evidence_cid: "blake3:evidence".to_owned(),
        };
        let label_control = NegativeControlEvidence {
            id: LABEL_SHUFFLED_CONTROL_ID.to_owned(),
            identity_cid: "blake3:control".to_owned(),
            verdict: NegativeControlVerdict::Passed,
            comparison: Some(gate_comparison(-1, 0)),
        };
        assert_eq!(
            staged_sample_decision(PromotionGateInputs {
                row: &row,
                measurements: &measurements,
                label_control: &label_control,
                planted_controls_available: true,
                external_cross_surface_checks: 6,
                combined_cross_surface_mismatches: 0,
                witness_replayed: 64,
                witness_failures: 0,
            }),
            ServingSampleDecision::Proceed
        );
    }

    #[test]
    fn binding_sample_gate_stops_on_reachability_quality_and_replay_failures() {
        let mut row = gate_row();
        row.lane_reachable = 0;
        row.internal_base_control_checks -= 1;
        row.internal_base_control_mismatches = 1;
        let measurements = QualityMeasurements {
            versus_tla: gate_comparison(-1, 0),
            versus_sections_absent: gate_comparison(RF31_MIN_LANE_DELTA_PPM - 1, 0),
            internal_base_control_checks: row.internal_base_control_checks,
            internal_base_control_mismatches: row.internal_base_control_mismatches,
            cross_surface_checks: row.internal_base_control_checks,
            cross_surface_mismatches: row.internal_base_control_mismatches,
            cross_surface_evidence_cid: "blake3:evidence".to_owned(),
        };
        let label_control = NegativeControlEvidence {
            id: LABEL_SHUFFLED_CONTROL_ID.to_owned(),
            identity_cid: "blake3:control".to_owned(),
            verdict: NegativeControlVerdict::Failed,
            comparison: Some(gate_comparison(1, 1)),
        };
        let failures = promotion_failures(PromotionGateInputs {
            row: &row,
            measurements: &measurements,
            label_control: &label_control,
            planted_controls_available: true,
            external_cross_surface_checks: 0,
            combined_cross_surface_mismatches: 1,
            witness_replayed: 0,
            witness_failures: 1,
        });
        for expected in [
            "unreachable",
            "versus TLA",
            "paired lane lower bound",
            "sections-absent identity",
            "label-shuffled",
            "cross-surface",
            "witness replay",
        ] {
            assert!(
                failures.iter().any(|failure| failure.contains(expected)),
                "missing {expected:?} from {failures:?}"
            );
        }
        assert!(matches!(
            staged_sample_decision(PromotionGateInputs {
                row: &row,
                measurements: &measurements,
                label_control: &label_control,
                planted_controls_available: true,
                external_cross_surface_checks: 0,
                combined_cross_surface_mismatches: 1,
                witness_replayed: 0,
                witness_failures: 1,
            }),
            ServingSampleDecision::Stop { .. }
        ));
    }

    #[test]
    fn sample_gate_distinguishes_upper_bound_stop_from_extendable_inconclusive() {
        let row = gate_row();
        let label_control = NegativeControlEvidence {
            id: LABEL_SHUFFLED_CONTROL_ID.to_owned(),
            identity_cid: "blake3:control".to_owned(),
            verdict: NegativeControlVerdict::Passed,
            comparison: Some(gate_comparison(-1, 0)),
        };
        fn make_inputs<'a>(
            row: &'a ServingEvalRow,
            measurements: &'a QualityMeasurements,
            label_control: &'a NegativeControlEvidence,
        ) -> PromotionGateInputs<'a> {
            PromotionGateInputs {
                row,
                measurements,
                label_control,
                planted_controls_available: true,
                external_cross_surface_checks: 6,
                combined_cross_surface_mismatches: 0,
                witness_replayed: 64,
                witness_failures: 0,
            }
        }

        let mut upper_miss_tla = gate_comparison(-10, 0);
        upper_miss_tla.interval.upper_delta_ppm = -1;
        let upper_miss = QualityMeasurements {
            versus_tla: upper_miss_tla,
            versus_sections_absent: gate_comparison(RF31_MIN_LANE_DELTA_PPM, 1),
            internal_base_control_checks: SAMPLE_TARGET as u64,
            internal_base_control_mismatches: 0,
            cross_surface_checks: SAMPLE_TARGET as u64 + 6,
            cross_surface_mismatches: 0,
            cross_surface_evidence_cid: "blake3:evidence".to_owned(),
        };
        assert!(matches!(
            staged_sample_decision(make_inputs(&row, &upper_miss, &label_control)),
            ServingSampleDecision::Stop { reasons }
                if reasons.iter().any(|reason| reason.contains("upper bound"))
        ));

        let mut crossing_tla = gate_comparison(-1, 0);
        crossing_tla.interval.upper_delta_ppm = 1;
        let mut crossing_lane = gate_comparison(RF31_MIN_LANE_DELTA_PPM - 1, 0);
        crossing_lane.interval.upper_delta_ppm = RF31_MIN_LANE_DELTA_PPM + 1;
        let inconclusive = QualityMeasurements {
            versus_tla: crossing_tla,
            versus_sections_absent: crossing_lane,
            internal_base_control_checks: SAMPLE_TARGET as u64,
            internal_base_control_mismatches: 0,
            cross_surface_checks: SAMPLE_TARGET as u64 + 6,
            cross_surface_mismatches: 0,
            cross_surface_evidence_cid: "blake3:evidence".to_owned(),
        };
        assert_eq!(
            staged_sample_decision(make_inputs(&row, &inconclusive, &label_control)),
            ServingSampleDecision::Inconclusive {
                reasons: vec![
                    "paired TLA interval [-1, 1] ppm crosses zero".to_string(),
                    format!(
                        "paired lane interval [{}, {}] ppm crosses the {} ppm floor",
                        RF31_MIN_LANE_DELTA_PPM - 1,
                        RF31_MIN_LANE_DELTA_PPM + 1,
                        RF31_MIN_LANE_DELTA_PPM
                    ),
                ],
                next_positions: Some(EXTENDED_SAMPLE_TARGET),
            }
        );

        let mut small_population_row = row.clone();
        small_population_row.population_n = 10_000;
        small_population_row.sample_n = 9_999;
        small_population_row.mode = ServingEvalMode::Sample { positions: 9_999 };
        small_population_row.internal_base_control_checks = 9_999;
        let mut small_population = inconclusive.clone();
        small_population.internal_base_control_checks = 9_999;
        small_population.cross_surface_checks = 10_005;
        assert!(matches!(
            staged_sample_decision(make_inputs(
                &small_population_row,
                &small_population,
                &label_control,
            )),
            ServingSampleDecision::Inconclusive {
                next_positions: None,
                ..
            }
        ));

        let mut extended_row = row.clone();
        extended_row.sample_n = EXTENDED_SAMPLE_TARGET;
        extended_row.mode = ServingEvalMode::Sample {
            positions: EXTENDED_SAMPLE_TARGET,
        };
        extended_row.internal_base_control_checks = EXTENDED_SAMPLE_TARGET as u64;
        let mut extended = inconclusive;
        extended.internal_base_control_checks = EXTENDED_SAMPLE_TARGET as u64;
        extended.cross_surface_checks = EXTENDED_SAMPLE_TARGET as u64 + 6;
        assert!(matches!(
            staged_sample_decision(make_inputs(&extended_row, &extended, &label_control)),
            ServingSampleDecision::Inconclusive {
                next_positions: None,
                ..
            }
        ));
    }
}
