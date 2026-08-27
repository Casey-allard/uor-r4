//! Release-only one-worker versus four-worker canary for the fixed-zeta
//! prime-route compiler (#958).
//!
//! This module owns the deterministic synthetic workload, measurement order,
//! semantic and worker-evidence checks, and the predeclared verdict. It does
//! not own terminal I/O, watchdog policy, or CLI presentation. Host timings
//! and worker observations are operational evidence and never enter the
//! compiled manifest's canonical bytes or kappa.

use std::num::{NonZeroU16, NonZeroUsize};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use uor_r4_core::prime_route_attention::{
    compile_spin_manifest, CompiledSpinManifest, GeometricAddress, ManifestProvenance, PhaseQ29,
    PrimeRegistry, PrimeRouteCompilation, PrimeRouteCompileMetadata, PrimeRouteError,
    RouteSentence, SemanticAtom, SpinTorsionState, UnitS3Q30, ZPhi, ZeroPowerBridge,
    TINY_CANARY_MAX_OCCURRENCES, TINY_CANARY_MAX_ROUTES_PER_SENTENCE, TINY_CANARY_MAX_SENTENCES,
    TINY_CANARY_MAX_TOTAL_ROUTES, TINY_CANARY_MAX_TRANSITIONS,
};

pub const PRIME_ROUTE_WORKER_CANARY_SCHEMA: u32 = 2;
pub const PRIME_ROUTE_WORKER_CANARY_DOMAIN: &str = "uor-r4.prime-route-worker-canary/2";
pub const PRIME_ROUTE_WORKLOAD_DOMAIN: &str = "uor-r4.prime-route-worker-canary-workload/1";
/// Immutable semantic provenance for the compiled manifest. Report schemas may
/// advance without changing these compiler/cost-profile inputs.
pub const PRIME_ROUTE_MANIFEST_PROVENANCE_DOMAIN: &str = "uor-r4.prime-route-worker-canary/1";
/// Re-pinning any reference is an explicit review decision, never an
/// automatic consequence of changing the measurement report.
pub const CANARY_REFERENCE_WORKLOAD_CID: &str =
    "blake3:ce3d96826ffd7134495536d439795ad0e4b035122b41329afd2a6ec4a96cacc6";
pub const CANARY_REFERENCE_CANONICAL_BYTES_CID: &str =
    "blake3:973acbe598b15aa152532910ac593ab70ebd723a5e76ee16de4ef030a0285422";
pub const CANARY_REFERENCE_MANIFEST_KAPPA: &str =
    "blake3:e8f1ed27755b36cfd8e3161b8c6cf46bcef3a2afeeafdd00c8a398b40e14aa4f";

pub const CANARY_SEMANTIC_ATOMS: usize = 32;
pub const CANARY_SPIN_VARIANTS: usize = 4;
pub const CANARY_LONG_SENTENCES: usize = 15;
pub const CANARY_SHORT_SENTENCES: usize = 1;
pub const CANARY_LONG_ROUTES: usize = TINY_CANARY_MAX_ROUTES_PER_SENTENCE;
pub const CANARY_SHORT_ROUTES: usize = 34;
pub const CANARY_MAXIMUM_CANDIDATES: u16 = 8;
pub const CANARY_WARMUPS_PER_WORKER_COUNT: usize = 1;
pub const CANARY_REPETITIONS_PER_WORKER_COUNT: usize = 3;
pub const CANARY_EXECUTIONS: usize = 8;
pub const CANARY_COMPILATIONS_PER_EXECUTION: usize = 4;
pub const CANARY_TOTAL_COMPILATIONS: usize = CANARY_EXECUTIONS * CANARY_COMPILATIONS_PER_EXECUTION;
pub const CANARY_SPEEDUP_MILLI_FLOOR: u64 = 1_200;
pub const CANARY_MIN_ONE_WORKER_MEDIAN_NS: u64 = 500_000_000;
pub const CANARY_MAX_SAMPLE_DEVIATION_MILLI: u64 = 150;
pub const CANARY_MAX_MANIFEST_BYTES: usize = 32 * 1024 * 1024;
pub const CANARY_HARD_WALL_MILLIS: u64 = 90_000;
pub const CANARY_WATCHDOG_KILL_MILLIS: u64 = 85_000;

const ONE_WORKER: usize = 1;
const FOUR_WORKERS: usize = 4;
const RELEASE_BUILD_PROFILE: &str = "release";
const Q30_ONE: i32 = 1 << 30;
const Q30_HALF: i32 = 1 << 29;
const SPIN_FIXTURES: [[i32; 4]; CANARY_SPIN_VARIANTS] = [
    [Q30_ONE, 0, 0, 0],
    [0, Q30_ONE, 0, 0],
    [Q30_HALF, Q30_HALF, Q30_HALF, Q30_HALF],
    [Q30_HALF, -Q30_HALF, Q30_HALF, -Q30_HALF],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrimeRouteCanaryPhase {
    Warmup,
    Measured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrimeRouteCanaryProgressState {
    Started,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrimeRouteCanaryCanonicalEvidence {
    StrictRoundtripVerified,
    ExactStrictBaselineMatch,
    ExactStrictBaselineMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrimeRouteCanaryVerdict {
    Pass,
    OptimizeBeforeLongRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrimeRouteCanaryFailure {
    RunScheduleInvalid,
    CompilationBatchAccountingInvalid,
    CanonicalEvidenceInvalid,
    ReferenceArtifactMismatch,
    SemanticBytesMismatch,
    ManifestKappaMismatch,
    WorkerTransitionAccountingInvalid,
    PeakActiveInsufficient,
    TimingResolutionInsufficient,
    TimingUnstable,
    SpeedupBelowFloor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeRouteCanaryWorkloadSpec {
    pub domain: String,
    pub manifest_provenance_domain: String,
    pub compiler_cid: String,
    pub cost_profile_cid: String,
    pub workload_cid: String,
    pub semantic_atoms: usize,
    pub address_pool: usize,
    pub sentences: usize,
    pub total_routes: usize,
    pub maximum_routes_per_sentence: usize,
    pub causal_transitions: usize,
    pub index_occurrences: usize,
    pub maximum_candidates: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeRouteCanaryContract {
    pub worker_counts: [usize; 2],
    pub warmups_per_worker_count: usize,
    pub repetitions_per_worker_count: usize,
    pub compilations_per_execution: usize,
    pub reference_workload_cid: String,
    pub reference_canonical_bytes_cid: String,
    pub reference_manifest_kappa: String,
    pub speedup_milli_floor: u64,
    pub minimum_one_worker_median_ns: u64,
    pub maximum_sample_deviation_milli: u64,
    pub maximum_manifest_bytes: usize,
    /// The CLI parent must terminate the canary before this wall-clock bound.
    pub hard_wall_millis: u64,
    /// The CLI parent should kill the child at this earlier watchdog bound so
    /// it has time to atomically persist a terminal report.
    pub watchdog_kill_millis: u64,
}

impl PrimeRouteCanaryContract {
    pub fn frozen() -> Self {
        Self {
            worker_counts: [ONE_WORKER, FOUR_WORKERS],
            warmups_per_worker_count: CANARY_WARMUPS_PER_WORKER_COUNT,
            repetitions_per_worker_count: CANARY_REPETITIONS_PER_WORKER_COUNT,
            compilations_per_execution: CANARY_COMPILATIONS_PER_EXECUTION,
            reference_workload_cid: CANARY_REFERENCE_WORKLOAD_CID.to_owned(),
            reference_canonical_bytes_cid: CANARY_REFERENCE_CANONICAL_BYTES_CID.to_owned(),
            reference_manifest_kappa: CANARY_REFERENCE_MANIFEST_KAPPA.to_owned(),
            speedup_milli_floor: CANARY_SPEEDUP_MILLI_FLOOR,
            minimum_one_worker_median_ns: CANARY_MIN_ONE_WORKER_MEDIAN_NS,
            maximum_sample_deviation_milli: CANARY_MAX_SAMPLE_DEVIATION_MILLI,
            maximum_manifest_bytes: CANARY_MAX_MANIFEST_BYTES,
            hard_wall_millis: CANARY_HARD_WALL_MILLIS,
            watchdog_kill_millis: CANARY_WATCHDOG_KILL_MILLIS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeRouteCanaryProgress {
    pub state: PrimeRouteCanaryProgressState,
    pub phase: PrimeRouteCanaryPhase,
    pub repetition: usize,
    pub execution_index: usize,
    pub total_executions: usize,
    pub requested_workers: usize,
    pub completed_executions: usize,
    pub completed_compilations: usize,
    pub total_compilations: usize,
    pub completed_transition_work: usize,
    pub total_transition_work: usize,
    pub execution_elapsed_ns: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeRouteCanaryWorkerEvidence {
    pub partition_id: usize,
    pub sentence_count: usize,
    pub assigned_transitions: usize,
    pub completed_transitions: usize,
    pub elapsed_ns: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeRouteCanaryCompileEvidence {
    pub requested_workers: usize,
    pub used_workers: usize,
    pub sentences: usize,
    pub route_steps: usize,
    pub causal_transitions: usize,
    pub index_occurrences: usize,
    pub peak_active_workers: usize,
    pub worker_reports: Vec<PrimeRouteCanaryWorkerEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeRouteCanaryCompilation {
    pub compilation_index: usize,
    pub elapsed_ns: u64,
    pub canonical_bytes_len: usize,
    pub canonical_bytes_cid: String,
    pub manifest_kappa: String,
    pub canonical_bytes_equal_to_baseline: bool,
    pub manifest_kappa_equal_to_baseline: bool,
    pub canonical_evidence: PrimeRouteCanaryCanonicalEvidence,
    pub compile: PrimeRouteCanaryCompileEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeRouteCanaryRun {
    pub execution_index: usize,
    pub phase: PrimeRouteCanaryPhase,
    pub repetition: usize,
    pub requested_workers: usize,
    pub compilations_per_execution: usize,
    /// Checked sum of the four complete `compile_spin_manifest` call timings.
    pub elapsed_ns: u64,
    pub compilations: Vec<PrimeRouteCanaryCompilation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeRouteCanaryDecision {
    pub verdict: PrimeRouteCanaryVerdict,
    pub failures: Vec<PrimeRouteCanaryFailure>,
    pub one_worker_median_ns: Option<u64>,
    pub four_worker_median_ns: Option<u64>,
    pub one_worker_max_deviation_milli: Option<u64>,
    pub four_worker_max_deviation_milli: Option<u64>,
    pub speedup_milli: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeRouteWorkerCanaryReport {
    pub schema: u32,
    pub domain: String,
    pub build_profile: String,
    pub target_arch: String,
    pub available_parallelism: usize,
    pub total_elapsed_ns: u64,
    pub workload: PrimeRouteCanaryWorkloadSpec,
    pub contract: PrimeRouteCanaryContract,
    pub runs: Vec<PrimeRouteCanaryRun>,
    pub decision: PrimeRouteCanaryDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimeRouteWorkerCanaryReportValidationError {
    SchemaMismatch {
        observed: u32,
    },
    DomainMismatch {
        observed: String,
    },
    BuildProfileMismatch {
        observed: String,
    },
    TargetArchMismatch {
        expected: &'static str,
        observed: String,
    },
    InsufficientParallelism {
        observed: usize,
        required: usize,
    },
    ZeroTotalElapsed,
    FrozenWorkloadUnavailable {
        reason: String,
    },
    WorkloadMismatch,
    ContractMismatch,
    DecisionMismatch,
}

impl std::fmt::Display for PrimeRouteWorkerCanaryReportValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SchemaMismatch { observed } => write!(
                formatter,
                "prime-route canary report schema {observed} does not match {PRIME_ROUTE_WORKER_CANARY_SCHEMA}"
            ),
            Self::DomainMismatch { observed } => write!(
                formatter,
                "prime-route canary report domain {observed:?} does not match {PRIME_ROUTE_WORKER_CANARY_DOMAIN:?}"
            ),
            Self::BuildProfileMismatch { observed } => write!(
                formatter,
                "prime-route canary report build profile {observed:?} is not {RELEASE_BUILD_PROFILE:?}"
            ),
            Self::TargetArchMismatch { expected, observed } => write!(
                formatter,
                "prime-route canary report target architecture {observed:?} does not match current target {expected:?}"
            ),
            Self::InsufficientParallelism { observed, required } => write!(
                formatter,
                "prime-route canary report records {observed} available workers; at least {required} are required"
            ),
            Self::ZeroTotalElapsed => {
                formatter.write_str("prime-route canary report records zero total elapsed time")
            }
            Self::FrozenWorkloadUnavailable { reason } => write!(
                formatter,
                "could not rebuild the frozen prime-route canary workload: {reason}"
            ),
            Self::WorkloadMismatch => formatter.write_str(
                "prime-route canary report workload does not match the exact frozen workload",
            ),
            Self::ContractMismatch => formatter.write_str(
                "prime-route canary report contract does not match the exact frozen contract",
            ),
            Self::DecisionMismatch => formatter.write_str(
                "prime-route canary report decision does not match a fresh evaluation of its runs",
            ),
        }
    }
}

impl std::error::Error for PrimeRouteWorkerCanaryReportValidationError {}

#[derive(Debug)]
pub enum PrimeRouteWorkerCanaryError {
    ReleaseBuildRequired,
    InsufficientParallelism { available: usize, required: usize },
    InvalidWorkload(String),
    ArithmeticOverflow,
    ManifestTooLarge { observed: usize, maximum: usize },
    Compile(PrimeRouteError),
}

impl std::fmt::Display for PrimeRouteWorkerCanaryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReleaseBuildRequired => {
                formatter.write_str("prime-route worker canary requires a release build")
            }
            Self::InsufficientParallelism {
                available,
                required,
            } => write!(
                formatter,
                "prime-route worker canary requires {required} available workers; host reports {available}"
            ),
            Self::InvalidWorkload(reason) => {
                write!(formatter, "invalid prime-route canary workload: {reason}")
            }
            Self::ArithmeticOverflow => {
                formatter.write_str("prime-route canary arithmetic overflow")
            }
            Self::ManifestTooLarge { observed, maximum } => write!(
                formatter,
                "prime-route canary manifest has {observed} bytes, above the {maximum}-byte ceiling"
            ),
            Self::Compile(error) => write!(formatter, "prime-route canary compile failed: {error}"),
        }
    }
}

impl std::error::Error for PrimeRouteWorkerCanaryError {}

impl From<PrimeRouteError> for PrimeRouteWorkerCanaryError {
    fn from(error: PrimeRouteError) -> Self {
        Self::Compile(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrozenWorkload {
    spec: PrimeRouteCanaryWorkloadSpec,
    sentences: Vec<RouteSentence>,
    registry: PrimeRegistry,
    provenance: ManifestProvenance,
    maximum_candidates: NonZeroU16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WorkloadCounts {
    total_routes: usize,
    maximum_routes_per_sentence: usize,
    causal_transitions: usize,
    index_occurrences: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScheduleEntry {
    phase: PrimeRouteCanaryPhase,
    repetition: usize,
    workers: usize,
}

const SCHEDULE: [ScheduleEntry; CANARY_EXECUTIONS] = [
    ScheduleEntry {
        phase: PrimeRouteCanaryPhase::Warmup,
        repetition: 0,
        workers: ONE_WORKER,
    },
    ScheduleEntry {
        phase: PrimeRouteCanaryPhase::Warmup,
        repetition: 0,
        workers: FOUR_WORKERS,
    },
    ScheduleEntry {
        phase: PrimeRouteCanaryPhase::Measured,
        repetition: 0,
        workers: ONE_WORKER,
    },
    ScheduleEntry {
        phase: PrimeRouteCanaryPhase::Measured,
        repetition: 0,
        workers: FOUR_WORKERS,
    },
    ScheduleEntry {
        phase: PrimeRouteCanaryPhase::Measured,
        repetition: 1,
        workers: FOUR_WORKERS,
    },
    ScheduleEntry {
        phase: PrimeRouteCanaryPhase::Measured,
        repetition: 1,
        workers: ONE_WORKER,
    },
    ScheduleEntry {
        phase: PrimeRouteCanaryPhase::Measured,
        repetition: 2,
        workers: ONE_WORKER,
    },
    ScheduleEntry {
        phase: PrimeRouteCanaryPhase::Measured,
        repetition: 2,
        workers: FOUR_WORKERS,
    },
];

/// Run the frozen release canary. The callback receives deterministic phase
/// boundaries; a CLI may mirror them into an atomic progress state and emit a
/// heartbeat without introducing I/O into this certifier.
pub fn run_prime_route_worker_canary<F>(
    mut progress: F,
) -> Result<PrimeRouteWorkerCanaryReport, PrimeRouteWorkerCanaryError>
where
    F: FnMut(&PrimeRouteCanaryProgress),
{
    if cfg!(debug_assertions) {
        return Err(PrimeRouteWorkerCanaryError::ReleaseBuildRequired);
    }
    let available_parallelism = std::thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(ONE_WORKER);
    if available_parallelism < FOUR_WORKERS {
        return Err(PrimeRouteWorkerCanaryError::InsufficientParallelism {
            available: available_parallelism,
            required: FOUR_WORKERS,
        });
    }

    let canary_started = Instant::now();
    let workload = build_frozen_workload()?;
    let total_transition_work =
        checked_transition_work(workload.spec.causal_transitions, CANARY_TOTAL_COMPILATIONS)?;
    let mut runs = Vec::with_capacity(CANARY_EXECUTIONS);
    let mut baseline_bytes: Option<Vec<u8>> = None;
    let mut baseline_kappa: Option<String> = None;

    for (execution_index, entry) in SCHEDULE.iter().enumerate() {
        progress(&PrimeRouteCanaryProgress {
            state: PrimeRouteCanaryProgressState::Started,
            phase: entry.phase,
            repetition: entry.repetition,
            execution_index,
            total_executions: CANARY_EXECUTIONS,
            requested_workers: entry.workers,
            completed_executions: execution_index,
            completed_compilations: execution_index
                .checked_mul(CANARY_COMPILATIONS_PER_EXECUTION)
                .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)?,
            total_compilations: CANARY_TOTAL_COMPILATIONS,
            completed_transition_work: checked_transition_work(
                workload.spec.causal_transitions,
                execution_index
                    .checked_mul(CANARY_COMPILATIONS_PER_EXECUTION)
                    .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)?,
            )?,
            total_transition_work,
            execution_elapsed_ns: None,
        });

        let workers = NonZeroUsize::new(entry.workers).ok_or_else(|| {
            PrimeRouteWorkerCanaryError::InvalidWorkload(
                "schedule requested zero workers".to_owned(),
            )
        })?;
        let mut compilations = Vec::with_capacity(CANARY_COMPILATIONS_PER_EXECUTION);
        let mut elapsed_ns = 0u64;
        for compilation_index in 0..CANARY_COMPILATIONS_PER_EXECUTION {
            let registry = workload.registry.clone();
            let provenance = workload.provenance.clone();
            let started = Instant::now();
            let compilation = compile_spin_manifest(
                &workload.sentences,
                registry,
                ZeroPowerBridge::ContinuousNull,
                provenance,
                workload.maximum_candidates,
                workers,
            )?;
            let compilation_elapsed_ns = duration_ns(started.elapsed());
            elapsed_ns = elapsed_ns
                .checked_add(compilation_elapsed_ns)
                .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)?;
            let (compilation, canonical_bytes) = record_compilation(
                compilation_index,
                compilation_elapsed_ns,
                compilation,
                baseline_bytes.as_deref(),
                baseline_kappa.as_deref(),
            )?;
            if baseline_bytes.is_none() {
                baseline_kappa = Some(compilation.manifest_kappa.clone());
                baseline_bytes = Some(canonical_bytes);
            }
            compilations.push(compilation);
        }
        let run = PrimeRouteCanaryRun {
            execution_index,
            phase: entry.phase,
            repetition: entry.repetition,
            requested_workers: entry.workers,
            compilations_per_execution: CANARY_COMPILATIONS_PER_EXECUTION,
            elapsed_ns,
            compilations,
        };
        runs.push(run);

        progress(&PrimeRouteCanaryProgress {
            state: PrimeRouteCanaryProgressState::Completed,
            phase: entry.phase,
            repetition: entry.repetition,
            execution_index,
            total_executions: CANARY_EXECUTIONS,
            requested_workers: entry.workers,
            completed_executions: execution_index + 1,
            completed_compilations: (execution_index + 1)
                .checked_mul(CANARY_COMPILATIONS_PER_EXECUTION)
                .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)?,
            total_compilations: CANARY_TOTAL_COMPILATIONS,
            completed_transition_work: checked_transition_work(
                workload.spec.causal_transitions,
                (execution_index + 1)
                    .checked_mul(CANARY_COMPILATIONS_PER_EXECUTION)
                    .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)?,
            )?,
            total_transition_work,
            execution_elapsed_ns: Some(elapsed_ns),
        });
    }

    let decision = evaluate_prime_route_worker_canary(&workload.spec, &runs);
    Ok(PrimeRouteWorkerCanaryReport {
        schema: PRIME_ROUTE_WORKER_CANARY_SCHEMA,
        domain: PRIME_ROUTE_WORKER_CANARY_DOMAIN.to_owned(),
        build_profile: RELEASE_BUILD_PROFILE.to_owned(),
        target_arch: std::env::consts::ARCH.to_owned(),
        available_parallelism,
        total_elapsed_ns: duration_ns(canary_started.elapsed()),
        workload: workload.spec,
        contract: PrimeRouteCanaryContract::frozen(),
        runs,
        decision,
    })
}

/// Convenience entry point for callers that do not need phase-boundary
/// progress. Terminal persistence still belongs to the CLI.
pub fn run_prime_route_worker_canary_silent(
) -> Result<PrimeRouteWorkerCanaryReport, PrimeRouteWorkerCanaryError> {
    run_prime_route_worker_canary(|_| {})
}

fn record_compilation(
    compilation_index: usize,
    elapsed_ns: u64,
    compilation: PrimeRouteCompilation,
    baseline_bytes: Option<&[u8]>,
    baseline_kappa: Option<&str>,
) -> Result<(PrimeRouteCanaryCompilation, Vec<u8>), PrimeRouteWorkerCanaryError> {
    let canonical_bytes = compilation.manifest.canonical_bytes()?;
    if canonical_bytes.len() > CANARY_MAX_MANIFEST_BYTES {
        return Err(PrimeRouteWorkerCanaryError::ManifestTooLarge {
            observed: canonical_bytes.len(),
            maximum: CANARY_MAX_MANIFEST_BYTES,
        });
    }
    let canonical_bytes_cid = blake3_cid(&canonical_bytes);
    let manifest_kappa = compilation.manifest.manifest_kappa.clone();
    let canonical_bytes_equal_to_baseline = baseline_bytes
        .map(|baseline| baseline == canonical_bytes.as_slice())
        .unwrap_or(true);
    let manifest_kappa_equal_to_baseline = baseline_kappa
        .map(|baseline| baseline == manifest_kappa)
        .unwrap_or(true);
    let canonical_evidence = if baseline_bytes.is_none() {
        let decoded = CompiledSpinManifest::decode_canonical(&canonical_bytes)?;
        let roundtrip = decoded.canonical_bytes()?;
        if roundtrip != canonical_bytes {
            return Err(PrimeRouteWorkerCanaryError::InvalidWorkload(
                "baseline canonical decode/re-encode changed the manifest bytes".to_owned(),
            ));
        }
        PrimeRouteCanaryCanonicalEvidence::StrictRoundtripVerified
    } else if canonical_bytes_equal_to_baseline {
        PrimeRouteCanaryCanonicalEvidence::ExactStrictBaselineMatch
    } else {
        PrimeRouteCanaryCanonicalEvidence::ExactStrictBaselineMismatch
    };

    Ok((
        PrimeRouteCanaryCompilation {
            compilation_index,
            elapsed_ns,
            canonical_bytes_len: canonical_bytes.len(),
            canonical_bytes_cid,
            manifest_kappa,
            canonical_bytes_equal_to_baseline,
            manifest_kappa_equal_to_baseline,
            canonical_evidence,
            compile: compile_evidence(compilation.metadata),
        },
        canonical_bytes,
    ))
}

fn compile_evidence(metadata: PrimeRouteCompileMetadata) -> PrimeRouteCanaryCompileEvidence {
    PrimeRouteCanaryCompileEvidence {
        requested_workers: metadata.requested_workers,
        used_workers: metadata.used_workers,
        sentences: metadata.sentences,
        route_steps: metadata.route_steps,
        causal_transitions: metadata.causal_transitions,
        index_occurrences: metadata.index_occurrences,
        peak_active_workers: metadata.peak_active_workers,
        worker_reports: metadata
            .worker_reports
            .into_iter()
            .map(|worker| PrimeRouteCanaryWorkerEvidence {
                partition_id: worker.partition_id,
                sentence_count: worker.sentence_count,
                assigned_transitions: worker.assigned_transitions,
                completed_transitions: worker.completed_transitions,
                elapsed_ns: duration_ns(worker.elapsed),
            })
            .collect(),
    }
}

/// Apply the frozen decision contract to completed run records. This function
/// is timing-free so every falsifier and the exact 1199/1200 boundary can be
/// covered by ordinary unit tests.
pub fn evaluate_prime_route_worker_canary(
    workload: &PrimeRouteCanaryWorkloadSpec,
    runs: &[PrimeRouteCanaryRun],
) -> PrimeRouteCanaryDecision {
    let mut failures = Vec::new();
    if !schedule_is_exact(runs) {
        push_failure(&mut failures, PrimeRouteCanaryFailure::RunScheduleInvalid);
    }

    if runs.iter().any(|run| !compilation_batch_is_valid(run)) {
        push_failure(
            &mut failures,
            PrimeRouteCanaryFailure::CompilationBatchAccountingInvalid,
        );
    }
    if !canonical_evidence_is_exact(runs) {
        push_failure(
            &mut failures,
            PrimeRouteCanaryFailure::CanonicalEvidenceInvalid,
        );
    }
    if !reference_artifact_matches(workload, runs) {
        push_failure(
            &mut failures,
            PrimeRouteCanaryFailure::ReferenceArtifactMismatch,
        );
    }

    if let Some(baseline) = runs.first().and_then(|run| run.compilations.first()) {
        if runs
            .iter()
            .flat_map(|run| &run.compilations)
            .any(|compilation| {
                !compilation.canonical_bytes_equal_to_baseline
                    || compilation.canonical_bytes_len != baseline.canonical_bytes_len
                    || compilation.canonical_bytes_cid != baseline.canonical_bytes_cid
            })
        {
            push_failure(
                &mut failures,
                PrimeRouteCanaryFailure::SemanticBytesMismatch,
            );
        }
        if runs
            .iter()
            .flat_map(|run| &run.compilations)
            .any(|compilation| {
                !compilation.manifest_kappa_equal_to_baseline
                    || compilation.manifest_kappa != baseline.manifest_kappa
            })
        {
            push_failure(
                &mut failures,
                PrimeRouteCanaryFailure::ManifestKappaMismatch,
            );
        }
    }

    for run in runs {
        for compilation in &run.compilations {
            if !worker_transition_accounting_is_valid(workload, run.requested_workers, compilation)
            {
                push_failure(
                    &mut failures,
                    PrimeRouteCanaryFailure::WorkerTransitionAccountingInvalid,
                );
            }
            if !peak_active_is_valid(run.requested_workers, compilation) {
                push_failure(
                    &mut failures,
                    PrimeRouteCanaryFailure::PeakActiveInsufficient,
                );
            }
        }
    }

    let one_samples = measured_samples(runs, ONE_WORKER);
    let four_samples = measured_samples(runs, FOUR_WORKERS);
    let one_worker_median_ns = median_ns(&one_samples);
    let four_worker_median_ns = median_ns(&four_samples);
    let one_worker_max_deviation_milli =
        one_worker_median_ns.map(|median| maximum_deviation_milli(&one_samples, median));
    let four_worker_max_deviation_milli =
        four_worker_median_ns.map(|median| maximum_deviation_milli(&four_samples, median));
    let speedup_milli = match (one_worker_median_ns, four_worker_median_ns) {
        (Some(one), Some(four)) => speedup_milli(one, four),
        _ => None,
    };

    if one_worker_median_ns.is_none_or(|median| median < CANARY_MIN_ONE_WORKER_MEDIAN_NS) {
        push_failure(
            &mut failures,
            PrimeRouteCanaryFailure::TimingResolutionInsufficient,
        );
    }
    if one_worker_max_deviation_milli
        .is_none_or(|deviation| deviation > CANARY_MAX_SAMPLE_DEVIATION_MILLI)
        || four_worker_max_deviation_milli
            .is_none_or(|deviation| deviation > CANARY_MAX_SAMPLE_DEVIATION_MILLI)
    {
        push_failure(&mut failures, PrimeRouteCanaryFailure::TimingUnstable);
    }
    if speedup_milli.is_none_or(|speedup| speedup < CANARY_SPEEDUP_MILLI_FLOOR) {
        push_failure(&mut failures, PrimeRouteCanaryFailure::SpeedupBelowFloor);
    }

    PrimeRouteCanaryDecision {
        verdict: if failures.is_empty() {
            PrimeRouteCanaryVerdict::Pass
        } else {
            PrimeRouteCanaryVerdict::OptimizeBeforeLongRun
        },
        failures,
        one_worker_median_ns,
        four_worker_median_ns,
        one_worker_max_deviation_milli,
        four_worker_max_deviation_milli,
        speedup_milli,
    }
}

/// Validate a completed report against the current frozen canary and recompute
/// its decision from the recorded runs. This is the fail-closed entry point for
/// consumers loading report evidence after the live run has completed.
pub fn validate_prime_route_worker_canary_report(
    report: &PrimeRouteWorkerCanaryReport,
) -> Result<(), PrimeRouteWorkerCanaryReportValidationError> {
    if report.schema != PRIME_ROUTE_WORKER_CANARY_SCHEMA {
        return Err(
            PrimeRouteWorkerCanaryReportValidationError::SchemaMismatch {
                observed: report.schema,
            },
        );
    }
    if report.domain != PRIME_ROUTE_WORKER_CANARY_DOMAIN {
        return Err(
            PrimeRouteWorkerCanaryReportValidationError::DomainMismatch {
                observed: report.domain.clone(),
            },
        );
    }
    if report.build_profile != RELEASE_BUILD_PROFILE {
        return Err(
            PrimeRouteWorkerCanaryReportValidationError::BuildProfileMismatch {
                observed: report.build_profile.clone(),
            },
        );
    }
    if report.target_arch != std::env::consts::ARCH {
        return Err(
            PrimeRouteWorkerCanaryReportValidationError::TargetArchMismatch {
                expected: std::env::consts::ARCH,
                observed: report.target_arch.clone(),
            },
        );
    }
    if report.available_parallelism < FOUR_WORKERS {
        return Err(
            PrimeRouteWorkerCanaryReportValidationError::InsufficientParallelism {
                observed: report.available_parallelism,
                required: FOUR_WORKERS,
            },
        );
    }
    if report.total_elapsed_ns == 0 {
        return Err(PrimeRouteWorkerCanaryReportValidationError::ZeroTotalElapsed);
    }

    let frozen_workload = build_frozen_workload().map_err(|error| {
        PrimeRouteWorkerCanaryReportValidationError::FrozenWorkloadUnavailable {
            reason: error.to_string(),
        }
    })?;
    if report.workload != frozen_workload.spec {
        return Err(PrimeRouteWorkerCanaryReportValidationError::WorkloadMismatch);
    }
    if report.contract != PrimeRouteCanaryContract::frozen() {
        return Err(PrimeRouteWorkerCanaryReportValidationError::ContractMismatch);
    }

    let recomputed = evaluate_prime_route_worker_canary(&report.workload, &report.runs);
    if report.decision != recomputed {
        return Err(PrimeRouteWorkerCanaryReportValidationError::DecisionMismatch);
    }
    Ok(())
}

fn schedule_is_exact(runs: &[PrimeRouteCanaryRun]) -> bool {
    runs.len() == SCHEDULE.len()
        && runs
            .iter()
            .zip(SCHEDULE)
            .enumerate()
            .all(|(execution_index, (run, expected))| {
                run.execution_index == execution_index
                    && run.phase == expected.phase
                    && run.repetition == expected.repetition
                    && run.requested_workers == expected.workers
            })
}

fn compilation_batch_is_valid(run: &PrimeRouteCanaryRun) -> bool {
    let elapsed = run
        .compilations
        .iter()
        .try_fold(0u64, |total, compilation| {
            total.checked_add(compilation.elapsed_ns)
        });
    run.compilations_per_execution == CANARY_COMPILATIONS_PER_EXECUTION
        && run.compilations.len() == CANARY_COMPILATIONS_PER_EXECUTION
        && run
            .compilations
            .iter()
            .enumerate()
            .all(|(expected, compilation)| compilation.compilation_index == expected)
        && elapsed == Some(run.elapsed_ns)
}

fn canonical_evidence_is_exact(runs: &[PrimeRouteCanaryRun]) -> bool {
    let mut compilations = runs.iter().flat_map(|run| &run.compilations);
    let Some(baseline) = compilations.next() else {
        return false;
    };
    baseline.canonical_evidence == PrimeRouteCanaryCanonicalEvidence::StrictRoundtripVerified
        && compilations.all(|compilation| {
            compilation.canonical_evidence
                == if compilation.canonical_bytes_equal_to_baseline {
                    PrimeRouteCanaryCanonicalEvidence::ExactStrictBaselineMatch
                } else {
                    PrimeRouteCanaryCanonicalEvidence::ExactStrictBaselineMismatch
                }
        })
}

fn reference_artifact_matches(
    workload: &PrimeRouteCanaryWorkloadSpec,
    runs: &[PrimeRouteCanaryRun],
) -> bool {
    let mut compilations = runs.iter().flat_map(|run| &run.compilations).peekable();
    workload.workload_cid == CANARY_REFERENCE_WORKLOAD_CID
        && compilations.peek().is_some()
        && compilations.all(|compilation| {
            compilation.canonical_bytes_cid == CANARY_REFERENCE_CANONICAL_BYTES_CID
                && compilation.manifest_kappa == CANARY_REFERENCE_MANIFEST_KAPPA
        })
}

fn worker_transition_accounting_is_valid(
    workload: &PrimeRouteCanaryWorkloadSpec,
    requested_workers: usize,
    compilation: &PrimeRouteCanaryCompilation,
) -> bool {
    if compilation.compile.requested_workers != requested_workers
        || compilation.compile.used_workers != requested_workers
        || compilation.compile.sentences != workload.sentences
        || compilation.compile.route_steps != workload.total_routes
        || compilation.compile.causal_transitions != workload.causal_transitions
        || compilation.compile.index_occurrences != workload.index_occurrences
        || compilation.compile.worker_reports.len() != requested_workers
    {
        return false;
    }

    let mut partition_ids = compilation
        .compile
        .worker_reports
        .iter()
        .map(|worker| worker.partition_id)
        .collect::<Vec<_>>();
    partition_ids.sort_unstable();
    if partition_ids != (0..requested_workers).collect::<Vec<_>>() {
        return false;
    }

    let sentence_count = compilation
        .compile
        .worker_reports
        .iter()
        .try_fold(0usize, |total, worker| {
            total.checked_add(worker.sentence_count)
        });
    let assigned = compilation
        .compile
        .worker_reports
        .iter()
        .try_fold(0usize, |total, worker| {
            total.checked_add(worker.assigned_transitions)
        });
    let completed = compilation
        .compile
        .worker_reports
        .iter()
        .try_fold(0usize, |total, worker| {
            total.checked_add(worker.completed_transitions)
        });
    compilation.compile.worker_reports.iter().all(|worker| {
        worker.sentence_count > 0
            && worker.assigned_transitions > 0
            && worker.completed_transitions == worker.assigned_transitions
    }) && sentence_count == Some(workload.sentences)
        && assigned == Some(workload.causal_transitions)
        && completed == Some(workload.causal_transitions)
}

fn peak_active_is_valid(
    requested_workers: usize,
    compilation: &PrimeRouteCanaryCompilation,
) -> bool {
    match requested_workers {
        ONE_WORKER => compilation.compile.peak_active_workers == ONE_WORKER,
        FOUR_WORKERS => compilation.compile.peak_active_workers == FOUR_WORKERS,
        _ => false,
    }
}

fn measured_samples(runs: &[PrimeRouteCanaryRun], workers: usize) -> Vec<u64> {
    runs.iter()
        .filter(|run| {
            run.phase == PrimeRouteCanaryPhase::Measured && run.requested_workers == workers
        })
        .map(|run| run.elapsed_ns)
        .collect()
}

pub fn median_ns(samples: &[u64]) -> Option<u64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        Some(sorted[middle])
    } else {
        let sum = u128::from(sorted[middle - 1]) + u128::from(sorted[middle]);
        Some(u64::try_from(sum / 2).unwrap_or(u64::MAX))
    }
}

pub fn maximum_deviation_milli(samples: &[u64], median: u64) -> u64 {
    if samples.is_empty() || median == 0 {
        return u64::MAX;
    }
    let maximum = samples
        .iter()
        .map(|sample| sample.abs_diff(median))
        .max()
        .unwrap_or(0);
    let scaled = u128::from(maximum).saturating_mul(1_000) / u128::from(median);
    u64::try_from(scaled).unwrap_or(u64::MAX)
}

pub fn speedup_milli(one_worker_ns: u64, four_worker_ns: u64) -> Option<u64> {
    if four_worker_ns == 0 {
        return None;
    }
    let scaled = u128::from(one_worker_ns).saturating_mul(1_000) / u128::from(four_worker_ns);
    Some(u64::try_from(scaled).unwrap_or(u64::MAX))
}

fn push_failure(failures: &mut Vec<PrimeRouteCanaryFailure>, failure: PrimeRouteCanaryFailure) {
    if !failures.contains(&failure) {
        failures.push(failure);
    }
}

fn build_frozen_workload() -> Result<FrozenWorkload, PrimeRouteWorkerCanaryError> {
    let atoms = (0..CANARY_SEMANTIC_ATOMS)
        .map(|index| {
            let semantic_atom_id = format!("atom-{index:02}");
            SemanticAtom {
                payload_cid: blake3_cid(
                    format!("{PRIME_ROUTE_WORKLOAD_DOMAIN}/payload/{semantic_atom_id}").as_bytes(),
                ),
                semantic_atom_id,
            }
        })
        .collect::<Vec<_>>();
    let registry = PrimeRegistry::compile(&atoms)?;
    let mut address_pool = Vec::with_capacity(CANARY_SEMANTIC_ATOMS * CANARY_SPIN_VARIANTS);
    for atom_index in 0..CANARY_SEMANTIC_ATOMS {
        let semantic_atom_id = format!("atom-{atom_index:02}");
        let binding = registry.binding_for_id(&semantic_atom_id).ok_or_else(|| {
            PrimeRouteWorkerCanaryError::InvalidWorkload(format!(
                "compiled registry omitted {semantic_atom_id}"
            ))
        })?;
        for (variant, raw_spin) in SPIN_FIXTURES.iter().copied().enumerate() {
            let fiber_raw = i32::try_from(atom_index * 4_096 + variant * 257)
                .map_err(|_| PrimeRouteWorkerCanaryError::ArithmeticOverflow)?;
            let torsion_raw = fiber_raw
                .checked_neg()
                .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)?;
            let spin = SpinTorsionState::new(
                UnitS3Q30::from_raw(raw_spin)?,
                PhaseQ29::from_raw(fiber_raw)?,
                PhaseQ29::from_raw(torsion_raw)?,
            )?;
            address_pool.push(GeometricAddress {
                atom: binding.atom,
                spin,
                radial: ZPhi::new(
                    i64::try_from(atom_index + 1)
                        .map_err(|_| PrimeRouteWorkerCanaryError::ArithmeticOverflow)?,
                    i64::try_from(variant)
                        .map_err(|_| PrimeRouteWorkerCanaryError::ArithmeticOverflow)?,
                ),
                payload_cid: binding.payload_cid.clone(),
            });
        }
    }

    let sentence_lengths = std::iter::repeat_n(CANARY_LONG_ROUTES, CANARY_LONG_SENTENCES)
        .chain(std::iter::repeat_n(
            CANARY_SHORT_ROUTES,
            CANARY_SHORT_SENTENCES,
        ))
        .collect::<Vec<_>>();
    let mut sentences = Vec::with_capacity(sentence_lengths.len());
    for (sentence_index, route_count) in sentence_lengths.into_iter().enumerate() {
        let mut routes = Vec::with_capacity(route_count);
        for position in 0..route_count {
            let pool_index = sentence_index
                .checked_mul(47)
                .and_then(|value| value.checked_add(position.checked_mul(29)?))
                .and_then(|value| {
                    position
                        .checked_mul(position)?
                        .checked_mul(3)?
                        .checked_add(value)
                })
                .and_then(|value| {
                    sentence_index
                        .checked_mul(position)?
                        .checked_mul(5)?
                        .checked_add(value)
                })
                .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)?
                % address_pool.len();
            routes.push(address_pool[pool_index].clone());
        }
        sentences.push(RouteSentence {
            sentence_id: format!("canary-sentence-{sentence_index:02}"),
            routes,
        });
    }

    let counts = derive_workload_counts(&sentences)?;
    validate_workload_limits(&sentences, counts)?;
    let maximum_candidates = NonZeroU16::new(CANARY_MAXIMUM_CANDIDATES).ok_or_else(|| {
        PrimeRouteWorkerCanaryError::InvalidWorkload("candidate bound must be nonzero".to_owned())
    })?;
    let workload_cid = workload_cid(&sentences, maximum_candidates.get())?;
    let compiler_cid =
        blake3_cid(format!("{PRIME_ROUTE_MANIFEST_PROVENANCE_DOMAIN}/compiler").as_bytes());
    let cost_profile_cid =
        blake3_cid(format!("{PRIME_ROUTE_MANIFEST_PROVENANCE_DOMAIN}/cost-profile").as_bytes());
    let spec = PrimeRouteCanaryWorkloadSpec {
        domain: PRIME_ROUTE_WORKLOAD_DOMAIN.to_owned(),
        manifest_provenance_domain: PRIME_ROUTE_MANIFEST_PROVENANCE_DOMAIN.to_owned(),
        compiler_cid: compiler_cid.clone(),
        cost_profile_cid: cost_profile_cid.clone(),
        workload_cid,
        semantic_atoms: atoms.len(),
        address_pool: address_pool.len(),
        sentences: sentences.len(),
        total_routes: counts.total_routes,
        maximum_routes_per_sentence: counts.maximum_routes_per_sentence,
        causal_transitions: counts.causal_transitions,
        index_occurrences: counts.index_occurrences,
        maximum_candidates: maximum_candidates.get(),
    };
    let provenance = ManifestProvenance {
        tokenizer_cid: blake3_cid(format!("{PRIME_ROUTE_WORKLOAD_DOMAIN}/tokenizer").as_bytes()),
        corpus_cid: spec.workload_cid.clone(),
        compiler_cid,
        cost_profile_cid,
    };
    Ok(FrozenWorkload {
        spec,
        sentences,
        registry,
        provenance,
        maximum_candidates,
    })
}

fn derive_workload_counts(
    sentences: &[RouteSentence],
) -> Result<WorkloadCounts, PrimeRouteWorkerCanaryError> {
    let mut counts = WorkloadCounts {
        total_routes: 0,
        maximum_routes_per_sentence: 0,
        causal_transitions: 0,
        index_occurrences: 0,
    };
    for sentence in sentences {
        let routes = sentence.routes.len();
        let transitions = routes.saturating_sub(1);
        let last_two = routes.saturating_sub(2);
        let occurrences = transitions
            .checked_mul(2)
            .and_then(|value| value.checked_add(last_two))
            .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)?;
        counts.total_routes = counts
            .total_routes
            .checked_add(routes)
            .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)?;
        counts.maximum_routes_per_sentence = counts.maximum_routes_per_sentence.max(routes);
        counts.causal_transitions = counts
            .causal_transitions
            .checked_add(transitions)
            .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)?;
        counts.index_occurrences = counts
            .index_occurrences
            .checked_add(occurrences)
            .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)?;
    }
    Ok(counts)
}

fn validate_workload_limits(
    sentences: &[RouteSentence],
    counts: WorkloadCounts,
) -> Result<(), PrimeRouteWorkerCanaryError> {
    let failures = [
        ("sentences", sentences.len(), TINY_CANARY_MAX_SENTENCES),
        (
            "routes per sentence",
            counts.maximum_routes_per_sentence,
            TINY_CANARY_MAX_ROUTES_PER_SENTENCE,
        ),
        (
            "total routes",
            counts.total_routes,
            TINY_CANARY_MAX_TOTAL_ROUTES,
        ),
        (
            "causal transitions",
            counts.causal_transitions,
            TINY_CANARY_MAX_TRANSITIONS,
        ),
        (
            "index occurrences",
            counts.index_occurrences,
            TINY_CANARY_MAX_OCCURRENCES,
        ),
    ];
    if let Some((label, observed, maximum)) = failures
        .into_iter()
        .find(|(_, observed, maximum)| observed > maximum)
    {
        return Err(PrimeRouteWorkerCanaryError::InvalidWorkload(format!(
            "{label} {observed} exceeds core ceiling {maximum}"
        )));
    }
    if sentences.len() < FOUR_WORKERS || sentences.iter().any(|sentence| sentence.routes.len() < 2)
    {
        return Err(PrimeRouteWorkerCanaryError::InvalidWorkload(
            "four-worker canary requires at least four sentences with positive transitions"
                .to_owned(),
        ));
    }
    Ok(())
}

fn workload_cid(
    sentences: &[RouteSentence],
    maximum_candidates: u16,
) -> Result<String, PrimeRouteWorkerCanaryError> {
    let mut hasher = blake3::Hasher::new();
    hash_bytes(&mut hasher, PRIME_ROUTE_WORKLOAD_DOMAIN.as_bytes())?;
    hasher.update(&maximum_candidates.to_le_bytes());
    hash_usize(&mut hasher, sentences.len())?;
    for sentence in sentences {
        hash_bytes(&mut hasher, sentence.sentence_id.as_bytes())?;
        hash_usize(&mut hasher, sentence.routes.len())?;
        for address in &sentence.routes {
            hasher.update(&address.atom.value().to_le_bytes());
            for coordinate in address.spin.s3.raw() {
                hasher.update(&coordinate.to_le_bytes());
            }
            for coordinate in address.spin.hopf.raw() {
                hasher.update(&coordinate.to_le_bytes());
            }
            hasher.update(&address.spin.fiber.raw().to_le_bytes());
            hasher.update(&address.spin.torsion.raw().to_le_bytes());
            hasher.update(&address.radial.a.to_le_bytes());
            hasher.update(&address.radial.b.to_le_bytes());
            hash_bytes(&mut hasher, address.payload_cid.as_bytes())?;
        }
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn hash_usize(
    hasher: &mut blake3::Hasher,
    value: usize,
) -> Result<(), PrimeRouteWorkerCanaryError> {
    let value =
        u64::try_from(value).map_err(|_| PrimeRouteWorkerCanaryError::ArithmeticOverflow)?;
    hasher.update(&value.to_le_bytes());
    Ok(())
}

fn hash_bytes(
    hasher: &mut blake3::Hasher,
    bytes: &[u8],
) -> Result<(), PrimeRouteWorkerCanaryError> {
    hash_usize(hasher, bytes.len())?;
    hasher.update(bytes);
    Ok(())
}

fn blake3_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn duration_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn checked_transition_work(
    transitions_per_compilation: usize,
    completed_compilations: usize,
) -> Result<usize, PrimeRouteWorkerCanaryError> {
    transitions_per_compilation
        .checked_mul(completed_compilations)
        .ok_or(PrimeRouteWorkerCanaryError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_workload_is_deterministic_and_derived_counts_fit_core_limits() {
        let first = build_frozen_workload().expect("first workload");
        let second = build_frozen_workload().expect("second workload");
        assert_eq!(first.spec, second.spec);
        assert_eq!(first.sentences, second.sentences);
        assert_eq!(first.registry, second.registry);
        assert_eq!(first.spec.sentences, 16);
        assert_eq!(first.spec.total_routes, 1_954);
        assert_eq!(first.spec.maximum_routes_per_sentence, 128);
        assert_eq!(first.spec.causal_transitions, 1_938);
        assert_eq!(first.spec.index_occurrences, 5_798);
        assert_eq!(first.spec.address_pool, 128);
        assert_eq!(PRIME_ROUTE_WORKER_CANARY_SCHEMA, 2);
        assert_eq!(
            first.spec.manifest_provenance_domain,
            PRIME_ROUTE_MANIFEST_PROVENANCE_DOMAIN
        );
        assert_ne!(
            first.spec.manifest_provenance_domain,
            PRIME_ROUTE_WORKER_CANARY_DOMAIN
        );
        assert_eq!(first.spec.compiler_cid, first.provenance.compiler_cid);
        assert_eq!(
            first.spec.cost_profile_cid,
            first.provenance.cost_profile_cid
        );
        assert_eq!(CANARY_TOTAL_COMPILATIONS, 32);
        assert_eq!(
            checked_transition_work(first.spec.causal_transitions, CANARY_TOTAL_COMPILATIONS)
                .expect("total transition work"),
            62_016
        );
        assert_eq!(
            checked_transition_work(
                first.spec.causal_transitions,
                CANARY_COMPILATIONS_PER_EXECUTION
            )
            .expect("one execution transition work"),
            7_752
        );
        assert_eq!(
            PrimeRouteCanaryContract::frozen().compilations_per_execution,
            4
        );
        assert_eq!(first.spec.workload_cid, CANARY_REFERENCE_WORKLOAD_CID);
        assert_eq!(
            PrimeRouteCanaryContract::frozen().reference_workload_cid,
            CANARY_REFERENCE_WORKLOAD_CID
        );
        assert_eq!(
            PrimeRouteCanaryContract::frozen().reference_canonical_bytes_cid,
            CANARY_REFERENCE_CANONICAL_BYTES_CID
        );
        assert_eq!(
            PrimeRouteCanaryContract::frozen().reference_manifest_kappa,
            CANARY_REFERENCE_MANIFEST_KAPPA
        );
        assert_eq!(PrimeRouteCanaryContract::frozen().hard_wall_millis, 90_000);
        assert_eq!(
            PrimeRouteCanaryContract::frozen().watchdog_kill_millis,
            85_000
        );
        assert!(
            PrimeRouteCanaryContract::frozen().watchdog_kill_millis
                < PrimeRouteCanaryContract::frozen().hard_wall_millis
        );
        assert!(first.spec.sentences <= TINY_CANARY_MAX_SENTENCES);
        assert!(first.spec.total_routes <= TINY_CANARY_MAX_TOTAL_ROUTES);
        assert!(first.spec.causal_transitions <= TINY_CANARY_MAX_TRANSITIONS);
        assert!(first.spec.index_occurrences <= TINY_CANARY_MAX_OCCURRENCES);
        assert!(first
            .sentences
            .iter()
            .all(|sentence| sentence.routes.len() >= 2));
        assert!(first.spec.workload_cid.starts_with("blake3:"));
        assert_eq!(first.spec.workload_cid.len(), "blake3:".len() + 64);
    }

    #[test]
    fn report_validator_accepts_the_exact_frozen_report() {
        let report = fake_valid_report();
        assert_eq!(validate_prime_route_worker_canary_report(&report), Ok(()));
    }

    #[test]
    fn report_validator_rejects_stored_decision_tampering() {
        let mut report = fake_valid_report();
        report.decision.speedup_milli = None;
        assert_eq!(
            validate_prime_route_worker_canary_report(&report),
            Err(PrimeRouteWorkerCanaryReportValidationError::DecisionMismatch)
        );
    }

    #[test]
    fn report_validator_recomputes_the_decision_from_run_evidence() {
        let mut report = fake_valid_report();
        report.runs[3].compilations[0].compile.peak_active_workers = 3;
        assert_eq!(
            validate_prime_route_worker_canary_report(&report),
            Err(PrimeRouteWorkerCanaryReportValidationError::DecisionMismatch)
        );
    }

    #[test]
    fn report_validator_rejects_contract_tampering() {
        let mut report = fake_valid_report();
        report.contract.speedup_milli_floor = CANARY_SPEEDUP_MILLI_FLOOR - 1;
        assert_eq!(
            evaluate_prime_route_worker_canary(&report.workload, &report.runs),
            report.decision
        );
        assert_eq!(
            validate_prime_route_worker_canary_report(&report),
            Err(PrimeRouteWorkerCanaryReportValidationError::ContractMismatch)
        );
    }

    #[test]
    fn report_validator_rejects_workload_field_tampering() {
        let mut report = fake_valid_report();
        report.workload.compiler_cid = blake3_cid(b"tampered-report-compiler");
        assert_eq!(
            evaluate_prime_route_worker_canary(&report.workload, &report.runs),
            report.decision
        );
        assert_eq!(
            validate_prime_route_worker_canary_report(&report),
            Err(PrimeRouteWorkerCanaryReportValidationError::WorkloadMismatch)
        );
    }

    #[test]
    fn report_validator_rejects_non_frozen_envelope_fields() {
        let mut report = fake_valid_report();
        report.schema += 1;
        assert_eq!(
            validate_prime_route_worker_canary_report(&report),
            Err(
                PrimeRouteWorkerCanaryReportValidationError::SchemaMismatch {
                    observed: PRIME_ROUTE_WORKER_CANARY_SCHEMA + 1,
                }
            )
        );

        let mut report = fake_valid_report();
        report.domain = "uor-r4.tampered/1".to_owned();
        assert_eq!(
            validate_prime_route_worker_canary_report(&report),
            Err(
                PrimeRouteWorkerCanaryReportValidationError::DomainMismatch {
                    observed: "uor-r4.tampered/1".to_owned(),
                }
            )
        );

        let mut report = fake_valid_report();
        report.build_profile = "debug".to_owned();
        assert_eq!(
            validate_prime_route_worker_canary_report(&report),
            Err(
                PrimeRouteWorkerCanaryReportValidationError::BuildProfileMismatch {
                    observed: "debug".to_owned(),
                }
            )
        );

        let mut report = fake_valid_report();
        report.target_arch = "tampered-arch".to_owned();
        assert_eq!(
            validate_prime_route_worker_canary_report(&report),
            Err(
                PrimeRouteWorkerCanaryReportValidationError::TargetArchMismatch {
                    expected: std::env::consts::ARCH,
                    observed: "tampered-arch".to_owned(),
                }
            )
        );

        let mut report = fake_valid_report();
        report.available_parallelism = FOUR_WORKERS - 1;
        assert_eq!(
            validate_prime_route_worker_canary_report(&report),
            Err(
                PrimeRouteWorkerCanaryReportValidationError::InsufficientParallelism {
                    observed: FOUR_WORKERS - 1,
                    required: FOUR_WORKERS,
                }
            )
        );

        let mut report = fake_valid_report();
        report.total_elapsed_ns = 0;
        assert_eq!(
            validate_prime_route_worker_canary_report(&report),
            Err(PrimeRouteWorkerCanaryReportValidationError::ZeroTotalElapsed)
        );
    }

    #[test]
    fn batch_accounting_and_strict_baseline_evidence_are_exact() {
        let workload = build_frozen_workload().expect("workload").spec;
        let runs = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        assert_eq!(runs.len(), CANARY_EXECUTIONS);
        assert!(runs.iter().all(compilation_batch_is_valid));
        assert!(canonical_evidence_is_exact(&runs));
        assert_eq!(
            runs.iter().map(|run| run.compilations.len()).sum::<usize>(),
            CANARY_TOTAL_COMPILATIONS
        );

        let mut missing = runs.clone();
        missing[7].compilations.pop();
        let decision = evaluate_prime_route_worker_canary(&workload, &missing);
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::CompilationBatchAccountingInvalid));

        let mut wrong_elapsed = runs.clone();
        wrong_elapsed[7].elapsed_ns += 1;
        let decision = evaluate_prime_route_worker_canary(&workload, &wrong_elapsed);
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::CompilationBatchAccountingInvalid));

        let mut missing_strict_roundtrip = runs;
        missing_strict_roundtrip[0].compilations[0].canonical_evidence =
            PrimeRouteCanaryCanonicalEvidence::ExactStrictBaselineMatch;
        let decision = evaluate_prime_route_worker_canary(&workload, &missing_strict_roundtrip);
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::CanonicalEvidenceInvalid));
    }

    #[test]
    fn homogeneous_reference_artifact_drift_fails_closed() {
        let workload = build_frozen_workload().expect("workload").spec;
        let runs = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        let mut workload_drift = workload.clone();
        workload_drift.workload_cid = blake3_cid(b"homogeneous-workload-drift");
        let decision = evaluate_prime_route_worker_canary(&workload_drift, &runs);
        assert_eq!(
            decision.verdict,
            PrimeRouteCanaryVerdict::OptimizeBeforeLongRun
        );
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::ReferenceArtifactMismatch));

        let mut byte_drift = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        for compilation in byte_drift.iter_mut().flat_map(|run| &mut run.compilations) {
            compilation.canonical_bytes_cid = blake3_cid(b"homogeneous-byte-drift");
        }
        let decision = evaluate_prime_route_worker_canary(&workload, &byte_drift);
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::ReferenceArtifactMismatch));
        assert!(!decision
            .failures
            .contains(&PrimeRouteCanaryFailure::SemanticBytesMismatch));

        let mut kappa_drift = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        for compilation in kappa_drift.iter_mut().flat_map(|run| &mut run.compilations) {
            compilation.manifest_kappa = blake3_cid(b"homogeneous-kappa-drift");
        }
        let decision = evaluate_prime_route_worker_canary(&workload, &kappa_drift);
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::ReferenceArtifactMismatch));
        assert!(!decision
            .failures
            .contains(&PrimeRouteCanaryFailure::ManifestKappaMismatch));
    }

    #[test]
    fn median_dispersion_and_speedup_use_exact_integer_math() {
        assert_eq!(median_ns(&[]), None);
        assert_eq!(median_ns(&[300, 100, 200]), Some(200));
        assert_eq!(median_ns(&[100, 200]), Some(150));
        assert_eq!(maximum_deviation_milli(&[190, 200, 210], 200), 50);
        assert_eq!(speedup_milli(1_200, 1_000), Some(1_200));
        assert_eq!(speedup_milli(1_199, 1_000), Some(1_199));
        assert_eq!(speedup_milli(1_000, 0), None);
    }

    #[test]
    fn speedup_boundary_is_pass_at_1200_and_optimize_at_1199() {
        let workload = build_frozen_workload().expect("workload").spec;
        let passing = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        let decision = evaluate_prime_route_worker_canary(&workload, &passing);
        assert_eq!(decision.speedup_milli, Some(1_200));
        assert_eq!(decision.verdict, PrimeRouteCanaryVerdict::Pass);

        let sub_floor = fake_runs(&workload, 1_199_000_000, 1_000_000_000);
        let decision = evaluate_prime_route_worker_canary(&workload, &sub_floor);
        assert_eq!(decision.speedup_milli, Some(1_199));
        assert_eq!(
            decision.verdict,
            PrimeRouteCanaryVerdict::OptimizeBeforeLongRun
        );
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::SpeedupBelowFloor));
    }

    #[test]
    fn timing_resolution_and_dispersion_fail_closed() {
        let workload = build_frozen_workload().expect("workload").spec;
        let too_short = fake_runs(&workload, 499_000_000, 400_000_000);
        let decision = evaluate_prime_route_worker_canary(&workload, &too_short);
        assert_eq!(
            decision.verdict,
            PrimeRouteCanaryVerdict::OptimizeBeforeLongRun
        );
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::TimingResolutionInsufficient));

        let mut unstable = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        let mut measured_one = unstable
            .iter_mut()
            .filter(|run| {
                run.phase == PrimeRouteCanaryPhase::Measured && run.requested_workers == ONE_WORKER
            })
            .collect::<Vec<_>>();
        set_run_elapsed(measured_one[0], 1_500_000_000);
        let decision = evaluate_prime_route_worker_canary(&workload, &unstable);
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::TimingUnstable));
    }

    #[test]
    fn semantic_and_worker_falsifiers_block_promotion() {
        let workload = build_frozen_workload().expect("workload").spec;

        let mut bytes_changed = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        bytes_changed[7].compilations[3].canonical_bytes_equal_to_baseline = false;
        bytes_changed[7].compilations[3].canonical_bytes_cid = blake3_cid(b"changed");
        bytes_changed[7].compilations[3].canonical_evidence =
            PrimeRouteCanaryCanonicalEvidence::ExactStrictBaselineMismatch;
        let decision = evaluate_prime_route_worker_canary(&workload, &bytes_changed);
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::SemanticBytesMismatch));

        let mut kappa_changed = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        kappa_changed[7].compilations[3].manifest_kappa_equal_to_baseline = false;
        kappa_changed[7].compilations[3].manifest_kappa = "blake3:changed".to_owned();
        let decision = evaluate_prime_route_worker_canary(&workload, &kappa_changed);
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::ManifestKappaMismatch));

        let mut incomplete = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        incomplete[3].compilations[3].compile.worker_reports[0].completed_transitions -= 1;
        let decision = evaluate_prime_route_worker_canary(&workload, &incomplete);
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::WorkerTransitionAccountingInvalid));

        let mut empty_worker = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        let moved = empty_worker[3].compilations[3].compile.worker_reports[0].assigned_transitions;
        empty_worker[3].compilations[3].compile.worker_reports[0].assigned_transitions = 0;
        empty_worker[3].compilations[3].compile.worker_reports[0].completed_transitions = 0;
        empty_worker[3].compilations[3].compile.worker_reports[1].assigned_transitions += moved;
        empty_worker[3].compilations[3].compile.worker_reports[1].completed_transitions += moved;
        let decision = evaluate_prime_route_worker_canary(&workload, &empty_worker);
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::WorkerTransitionAccountingInvalid));

        let mut inactive = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        inactive[3].compilations[3].compile.peak_active_workers = 1;
        let decision = evaluate_prime_route_worker_canary(&workload, &inactive);
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::PeakActiveInsufficient));

        let mut only_three_concurrent = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        only_three_concurrent[3].compilations[3]
            .compile
            .peak_active_workers = 3;
        let decision = evaluate_prime_route_worker_canary(&workload, &only_three_concurrent);
        assert_eq!(
            decision.verdict,
            PrimeRouteCanaryVerdict::OptimizeBeforeLongRun
        );
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::PeakActiveInsufficient));

        let mut wrong_sentence_total = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        wrong_sentence_total[3].compilations[3]
            .compile
            .worker_reports[0]
            .sentence_count += 1;
        let decision = evaluate_prime_route_worker_canary(&workload, &wrong_sentence_total);
        assert_eq!(
            decision.verdict,
            PrimeRouteCanaryVerdict::OptimizeBeforeLongRun
        );
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::WorkerTransitionAccountingInvalid));

        let mut overflowing_sentence_total = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        overflowing_sentence_total[3].compilations[3]
            .compile
            .worker_reports[0]
            .sentence_count = usize::MAX;
        let decision = evaluate_prime_route_worker_canary(&workload, &overflowing_sentence_total);
        assert_eq!(
            decision.verdict,
            PrimeRouteCanaryVerdict::OptimizeBeforeLongRun
        );
        assert!(decision
            .failures
            .contains(&PrimeRouteCanaryFailure::WorkerTransitionAccountingInvalid));
    }

    fn fake_valid_report() -> PrimeRouteWorkerCanaryReport {
        let workload = build_frozen_workload().expect("frozen workload").spec;
        let runs = fake_runs(&workload, 1_200_000_000, 1_000_000_000);
        let total_elapsed_ns = runs
            .iter()
            .try_fold(0u64, |total, run| total.checked_add(run.elapsed_ns))
            .expect("small fake report timing sum");
        let decision = evaluate_prime_route_worker_canary(&workload, &runs);
        PrimeRouteWorkerCanaryReport {
            schema: PRIME_ROUTE_WORKER_CANARY_SCHEMA,
            domain: PRIME_ROUTE_WORKER_CANARY_DOMAIN.to_owned(),
            build_profile: RELEASE_BUILD_PROFILE.to_owned(),
            target_arch: std::env::consts::ARCH.to_owned(),
            available_parallelism: FOUR_WORKERS,
            total_elapsed_ns,
            workload,
            contract: PrimeRouteCanaryContract::frozen(),
            runs,
            decision,
        }
    }

    fn fake_runs(
        workload: &PrimeRouteCanaryWorkloadSpec,
        one_worker_ns: u64,
        four_worker_ns: u64,
    ) -> Vec<PrimeRouteCanaryRun> {
        SCHEDULE
            .iter()
            .enumerate()
            .map(|(execution_index, entry)| {
                let elapsed_ns = if entry.workers == ONE_WORKER {
                    one_worker_ns
                } else {
                    four_worker_ns
                };
                let compilations = (0..CANARY_COMPILATIONS_PER_EXECUTION)
                    .map(|compilation_index| PrimeRouteCanaryCompilation {
                        compilation_index,
                        elapsed_ns: split_batch_elapsed(elapsed_ns, compilation_index),
                        canonical_bytes_len: 4_096,
                        canonical_bytes_cid: CANARY_REFERENCE_CANONICAL_BYTES_CID.to_owned(),
                        manifest_kappa: CANARY_REFERENCE_MANIFEST_KAPPA.to_owned(),
                        canonical_bytes_equal_to_baseline: true,
                        manifest_kappa_equal_to_baseline: true,
                        canonical_evidence: if execution_index == 0 && compilation_index == 0 {
                            PrimeRouteCanaryCanonicalEvidence::StrictRoundtripVerified
                        } else {
                            PrimeRouteCanaryCanonicalEvidence::ExactStrictBaselineMatch
                        },
                        compile: fake_compile_evidence(workload, entry.workers),
                    })
                    .collect();
                PrimeRouteCanaryRun {
                    execution_index,
                    phase: entry.phase,
                    repetition: entry.repetition,
                    requested_workers: entry.workers,
                    compilations_per_execution: CANARY_COMPILATIONS_PER_EXECUTION,
                    elapsed_ns,
                    compilations,
                }
            })
            .collect()
    }

    fn split_batch_elapsed(total: u64, compilation_index: usize) -> u64 {
        let divisor = u64::try_from(CANARY_COMPILATIONS_PER_EXECUTION).expect("small divisor");
        let base = total / divisor;
        let remainder = total % divisor;
        base + u64::from(compilation_index < usize::try_from(remainder).expect("small remainder"))
    }

    fn set_run_elapsed(run: &mut PrimeRouteCanaryRun, elapsed_ns: u64) {
        run.elapsed_ns = elapsed_ns;
        for compilation in &mut run.compilations {
            compilation.elapsed_ns = split_batch_elapsed(elapsed_ns, compilation.compilation_index);
        }
    }

    fn fake_compile_evidence(
        workload: &PrimeRouteCanaryWorkloadSpec,
        workers: usize,
    ) -> PrimeRouteCanaryCompileEvidence {
        let base = workload.causal_transitions / workers;
        let remainder = workload.causal_transitions % workers;
        let sentence_base = workload.sentences / workers;
        let sentence_remainder = workload.sentences % workers;
        let worker_reports = (0..workers)
            .map(|partition_id| {
                let transitions = base + usize::from(partition_id < remainder);
                PrimeRouteCanaryWorkerEvidence {
                    partition_id,
                    sentence_count: sentence_base + usize::from(partition_id < sentence_remainder),
                    assigned_transitions: transitions,
                    completed_transitions: transitions,
                    elapsed_ns: 100,
                }
            })
            .collect();
        PrimeRouteCanaryCompileEvidence {
            requested_workers: workers,
            used_workers: workers,
            sentences: workload.sentences,
            route_steps: workload.total_routes,
            causal_transitions: workload.causal_transitions,
            index_occurrences: workload.index_occurrences,
            peak_active_workers: workers,
            worker_reports,
        }
    }
}
