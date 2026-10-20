//! Reusable, dependency-light observability for the live teacher-parity harness.
//!
//! This module deliberately keeps deterministic evidence separate from empirical
//! run telemetry. Timings, resource measurements, ETA estimates, and scheduling
//! observations may vary without changing the evidence bytes supplied by the
//! harness.

#![allow(
    dead_code,
    reason = "this support API is compiled independently by multiple integration-test crates"
)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::num::{NonZeroU64, NonZeroUsize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uor_r4_model_source::TeacherExecutionSnapshot;

pub const EVENT_SCHEMA: &str = "uor-r4.teacher-parity-progress/2";
pub const RUN_REPORT_SCHEMA: &str = "uor-r4.teacher-parity-report/2";
pub const EVIDENCE_SCHEMA: &str = "uor-r4.teacher-parity-evidence/2";

const DEFAULT_PROGRESS_SECONDS: u64 = 10;
const DEFAULT_MAX_WALL_SECONDS: u64 = 28_800;
const MINIMUM_DEFAULT_STALL_SECONDS: u64 = 120;
pub const CANONICAL_PARITY_STREAMS: usize = 8;
pub const MAXIMUM_ADAPTIVE_GENERATION_TOKENS: usize = 8;
pub const ADAPTIVE_EARLY_STOP_RATIO: f64 = 1.10;
pub const ADAPTIVE_ACCEPTANCE_RATIO: f64 = 1.0;
pub const DEFAULT_PREFLIGHT_REPORT_PATH: &str = "target/teacher-parity/teacher-free-preflight.json";
pub const DEFAULT_EXACT_PROBE_REPORT_PATH: &str =
    "target/teacher-parity/exact-multicore-probe.json";

/// Resolve one fixture directory from a fail-closed environment lookup.
///
/// The caller supplies the lookup result so focused tests can exercise the
/// non-Unicode/error branch without mutating the process environment. Relative
/// overrides are intentionally preserved here; the BDD harness resolves them
/// against its startup working directory before recording the selected path.
pub fn configured_fixture_dir(
    name: &str,
    configured: Result<Option<String>, String>,
    default: PathBuf,
) -> Result<PathBuf, String> {
    let configured = configured.map_err(|reason| format!("{name} {reason}"))?;
    match configured {
        Some(path) if path.trim().is_empty() => Err(format!("{name} is empty")),
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(default),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdaptiveDecodeDecision {
    ExtendConservativeMarginNotCleared,
    StopEarlyConservativeMarginCleared,
    StopAtMaximumAcceptanceCleared,
    StopAtMaximumNotEstablished,
}

impl AdaptiveDecodeDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExtendConservativeMarginNotCleared => "EXTEND_CONSERVATIVE_MARGIN_NOT_CLEARED",
            Self::StopEarlyConservativeMarginCleared => "STOP_EARLY_CONSERVATIVE_MARGIN_CLEARED",
            Self::StopAtMaximumAcceptanceCleared => "STOP_AT_MAXIMUM_ACCEPTANCE_CLEARED",
            Self::StopAtMaximumNotEstablished => "STOP_AT_MAXIMUM_NOT_ESTABLISHED",
        }
    }

    pub fn is_terminal(self) -> bool {
        self != Self::ExtendConservativeMarginNotCleared
    }
}

pub fn adaptive_decode_decision(
    current_steps: usize,
    maximum_steps: usize,
    legacy_ratio: f64,
    graph_ratio: f64,
) -> AdaptiveDecodeDecision {
    let at_maximum = current_steps >= maximum_steps;
    if !at_maximum
        && legacy_ratio > ADAPTIVE_EARLY_STOP_RATIO
        && graph_ratio > ADAPTIVE_EARLY_STOP_RATIO
    {
        AdaptiveDecodeDecision::StopEarlyConservativeMarginCleared
    } else if at_maximum
        && legacy_ratio > ADAPTIVE_ACCEPTANCE_RATIO
        && graph_ratio > ADAPTIVE_ACCEPTANCE_RATIO
    {
        AdaptiveDecodeDecision::StopAtMaximumAcceptanceCleared
    } else if at_maximum {
        AdaptiveDecodeDecision::StopAtMaximumNotEstablished
    } else {
        AdaptiveDecodeDecision::ExtendConservativeMarginNotCleared
    }
}

pub fn adaptive_decode_checkpoints(maximum_steps: usize) -> Vec<usize> {
    let mut checkpoints = Vec::new();
    let mut steps = 1usize;
    while steps <= maximum_steps {
        checkpoints.push(steps);
        if steps == maximum_steps {
            break;
        }
        steps = steps.saturating_mul(2).min(maximum_steps);
    }
    checkpoints
}

enum StartGateState {
    Waiting,
    Started(Instant),
    Cancelled(String),
}

/// A start rendezvous that can release already-created workers when a later
/// OS thread creation fails. Unlike a fixed-party barrier, cancellation never
/// strands the partial worker cohort.
pub struct CancellableStartGate {
    state: Mutex<StartGateState>,
    changed: Condvar,
}

impl CancellableStartGate {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(StartGateState::Waiting),
            changed: Condvar::new(),
        }
    }

    pub fn wait(&self) -> Result<Instant, String> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        loop {
            match &*state {
                StartGateState::Waiting => {
                    state = self
                        .changed
                        .wait(state)
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                }
                StartGateState::Started(origin) => return Ok(*origin),
                StartGateState::Cancelled(reason) => return Err(reason.clone()),
            }
        }
    }

    pub fn start(&self, origin: Instant) {
        *self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = StartGateState::Started(origin);
        self.changed.notify_all();
    }

    pub fn cancel(&self, reason: impl Into<String>) {
        *self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            StartGateState::Cancelled(reason.into());
        self.changed.notify_all();
    }
}

impl Default for CancellableStartGate {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObservabilityMode {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParityConfig {
    pub workers: NonZeroUsize,
    pub streams: NonZeroUsize,
    pub batch_per_worker: NonZeroUsize,
    /// Human and JSONL heartbeat cadence, in seconds.
    pub progress_every: NonZeroU64,
    /// Age of the last completed work item at which ETA changes to STALL.
    pub stall_after: NonZeroU64,
    /// Maximum admitted run wall time, in seconds.
    pub max_wall: NonZeroU64,
    pub positions: NonZeroUsize,
    /// Maximum cumulative decode steps per canonical lane (1/2/4/8).
    pub gen_tokens: NonZeroUsize,
    /// Diagnostic parse surface retained for compatibility; a fixture-present
    /// binding run requires exactly one causal cohort.
    pub runs: NonZeroUsize,
    pub corpus_positions: NonZeroUsize,
    pub fmm_positions: NonZeroUsize,
    pub probe_positions: NonZeroUsize,
    pub mode: ObservabilityMode,
    /// Empirical final report. Evidence and JSONL event paths are siblings.
    pub report_path: PathBuf,
}

impl ParityConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_env_lookup(std::env::var)
    }

    /// Read every parity control exactly once while preserving the distinction
    /// between an absent variable and a present non-Unicode value. The lookup
    /// seam keeps the fail-closed branch directly testable without mutating the
    /// process-wide environment.
    pub fn from_env_lookup(
        mut lookup: impl FnMut(&'static str) -> Result<String, std::env::VarError>,
    ) -> Result<Self, ConfigError> {
        const NAMES: [&str; 13] = [
            "R4_PARITY_WORKERS",
            "R4_PARITY_STREAMS",
            "R4_PARITY_BATCH_PER_WORKER",
            "R4_PARITY_PROGRESS_EVERY_SECS",
            "R4_PARITY_MAX_WALL_SECS",
            "R4_PARITY_POSITIONS",
            "R4_PARITY_GEN_TOKENS",
            "R4_PARITY_RUNS",
            "R4_PARITY_CORPUS_POSITIONS",
            "R4_FMM_POSITIONS",
            "R4_EXACT_PROBE_POSITIONS",
            "R4_PARITY_TELEMETRY",
            "R4_PARITY_REPORT",
        ];
        let mut values = BTreeMap::new();
        for name in NAMES {
            match lookup(name) {
                Ok(value) => {
                    values.insert(name, value);
                }
                Err(std::env::VarError::NotPresent) => {}
                Err(std::env::VarError::NotUnicode(_)) => {
                    return Err(ConfigError::NonUnicode { name });
                }
            }
        }
        Self::from_lookup(|name| values.get(name).cloned())
    }

    pub fn from_lookup(
        mut lookup: impl FnMut(&str) -> Option<String>,
    ) -> Result<Self, ConfigError> {
        let available = std::thread::available_parallelism()
            .unwrap_or(NonZeroUsize::MIN)
            .get();
        let workers =
            parse_nonzero_usize("R4_PARITY_WORKERS", lookup("R4_PARITY_WORKERS"), available)?;
        if workers.get() > available {
            return Err(ConfigError::WorkersAboveAvailable {
                requested: workers.get(),
                available,
            });
        }
        // Stream width is semantic coverage, not a CPU-worker bound. Keep the
        // eight canonical prompt trajectories even on hosts with a different
        // available row-worker count.
        let streams = parse_nonzero_usize(
            "R4_PARITY_STREAMS",
            lookup("R4_PARITY_STREAMS"),
            CANONICAL_PARITY_STREAMS,
        )?;
        let batch_per_worker = parse_nonzero_usize(
            "R4_PARITY_BATCH_PER_WORKER",
            lookup("R4_PARITY_BATCH_PER_WORKER"),
            4,
        )?;
        let progress_every = parse_nonzero_u64(
            "R4_PARITY_PROGRESS_EVERY_SECS",
            lookup("R4_PARITY_PROGRESS_EVERY_SECS"),
            DEFAULT_PROGRESS_SECONDS,
        )?;
        let default_stall = progress_every
            .get()
            .saturating_mul(4)
            .max(MINIMUM_DEFAULT_STALL_SECONDS);
        let stall_after =
            NonZeroU64::new(default_stall).expect("derived stall interval is nonzero");
        let max_wall = parse_nonzero_u64(
            "R4_PARITY_MAX_WALL_SECS",
            lookup("R4_PARITY_MAX_WALL_SECS"),
            DEFAULT_MAX_WALL_SECONDS,
        )?;
        if max_wall.get() > DEFAULT_MAX_WALL_SECONDS {
            return Err(ConfigError::OutOfRange {
                name: "R4_PARITY_MAX_WALL_SECS",
                value: usize::try_from(max_wall.get()).unwrap_or(usize::MAX),
                minimum: 1,
                maximum: usize::try_from(DEFAULT_MAX_WALL_SECONDS).unwrap_or(usize::MAX),
            });
        }
        let positions =
            parse_nonzero_usize("R4_PARITY_POSITIONS", lookup("R4_PARITY_POSITIONS"), 256)?;
        let gen_tokens = parse_nonzero_usize(
            "R4_PARITY_GEN_TOKENS",
            lookup("R4_PARITY_GEN_TOKENS"),
            MAXIMUM_ADAPTIVE_GENERATION_TOKENS,
        )?;
        if gen_tokens.get() > MAXIMUM_ADAPTIVE_GENERATION_TOKENS {
            return Err(ConfigError::OutOfRange {
                name: "R4_PARITY_GEN_TOKENS",
                value: gen_tokens.get(),
                minimum: 1,
                maximum: MAXIMUM_ADAPTIVE_GENERATION_TOKENS,
            });
        }
        if !gen_tokens.get().is_power_of_two() {
            return Err(ConfigError::InvalidAdaptiveMaximum {
                value: gen_tokens.get(),
            });
        }
        let runs = parse_nonzero_usize("R4_PARITY_RUNS", lookup("R4_PARITY_RUNS"), 1)?;
        let corpus_positions = parse_nonzero_usize(
            "R4_PARITY_CORPUS_POSITIONS",
            lookup("R4_PARITY_CORPUS_POSITIONS"),
            1_000,
        )?;
        let fmm_positions =
            parse_nonzero_usize("R4_FMM_POSITIONS", lookup("R4_FMM_POSITIONS"), 256)?;
        let probe_positions = parse_nonzero_usize(
            "R4_EXACT_PROBE_POSITIONS",
            lookup("R4_EXACT_PROBE_POSITIONS"),
            1,
        )?;
        if probe_positions.get() > 8 {
            return Err(ConfigError::OutOfRange {
                name: "R4_EXACT_PROBE_POSITIONS",
                value: probe_positions.get(),
                minimum: 1,
                maximum: 8,
            });
        }
        if gen_tokens
            .get()
            .checked_mul(runs.get())
            .and_then(|measured| measured.checked_add(positions.get()))
            .and_then(|per_stream| per_stream.checked_mul(streams.get()))
            .is_none()
        {
            return Err(ConfigError::BudgetOverflow);
        }
        let mode = parse_mode(lookup("R4_PARITY_TELEMETRY"))?;
        let report_path = match lookup("R4_PARITY_REPORT") {
            None => PathBuf::from("target/teacher-parity/parity-report.json"),
            Some(value) if value.trim().is_empty() => {
                return Err(ConfigError::InvalidPath {
                    name: "R4_PARITY_REPORT",
                    value,
                });
            }
            Some(value) => PathBuf::from(value),
        };
        Ok(Self {
            workers,
            streams,
            batch_per_worker,
            progress_every,
            stall_after,
            max_wall,
            positions,
            gen_tokens,
            runs,
            corpus_positions,
            fmm_positions,
            probe_positions,
            mode,
            report_path,
        })
    }
}

pub fn configured_preflight_report_path(
    configured: Option<String>,
) -> Result<PathBuf, ConfigError> {
    match configured {
        Some(path) if path.trim().is_empty() => Err(ConfigError::InvalidPath {
            name: "R4_PARITY_PREFLIGHT_REPORT",
            value: path,
        }),
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(PathBuf::from(DEFAULT_PREFLIGHT_REPORT_PATH)),
    }
}

pub fn configured_exact_probe_report_path(
    configured: Option<String>,
) -> Result<PathBuf, ConfigError> {
    match configured {
        Some(path) if path.trim().is_empty() => Err(ConfigError::InvalidPath {
            name: "R4_EXACT_PROBE_REPORT",
            value: path,
        }),
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(PathBuf::from(DEFAULT_EXACT_PROBE_REPORT_PATH)),
    }
}

fn parse_nonzero_usize(
    name: &'static str,
    raw: Option<String>,
    default: usize,
) -> Result<NonZeroUsize, ConfigError> {
    let Some(raw) = raw else {
        return NonZeroUsize::new(default).ok_or_else(|| ConfigError::NonPositive {
            name,
            value: default.to_string(),
        });
    };
    let trimmed = raw.trim();
    let value = trimmed
        .parse::<usize>()
        .map_err(|_| ConfigError::InvalidInteger {
            name,
            value: raw.clone(),
        })?;
    NonZeroUsize::new(value).ok_or(ConfigError::NonPositive { name, value: raw })
}

fn parse_nonzero_u64(
    name: &'static str,
    raw: Option<String>,
    default: u64,
) -> Result<NonZeroU64, ConfigError> {
    let Some(raw) = raw else {
        return NonZeroU64::new(default).ok_or_else(|| ConfigError::NonPositive {
            name,
            value: default.to_string(),
        });
    };
    let trimmed = raw.trim();
    let value = trimmed
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidInteger {
            name,
            value: raw.clone(),
        })?;
    NonZeroU64::new(value).ok_or(ConfigError::NonPositive { name, value: raw })
}

fn parse_mode(raw: Option<String>) -> Result<ObservabilityMode, ConfigError> {
    let Some(raw) = raw else {
        return Ok(ObservabilityMode::Enabled);
    };
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(ObservabilityMode::Enabled),
        "0" | "false" | "no" | "off" => Ok(ObservabilityMode::Disabled),
        _ => Err(ConfigError::InvalidBoolean {
            name: "R4_PARITY_TELEMETRY",
            value: raw,
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    NonUnicode {
        name: &'static str,
    },
    InvalidInteger {
        name: &'static str,
        value: String,
    },
    NonPositive {
        name: &'static str,
        value: String,
    },
    InvalidBoolean {
        name: &'static str,
        value: String,
    },
    WorkersAboveAvailable {
        requested: usize,
        available: usize,
    },
    InvalidPath {
        name: &'static str,
        value: String,
    },
    OutOfRange {
        name: &'static str,
        value: usize,
        minimum: usize,
        maximum: usize,
    },
    InvalidAdaptiveMaximum {
        value: usize,
    },
    BudgetOverflow,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonUnicode { name } => {
                write!(formatter, "{name} is present but is not valid Unicode")
            }
            Self::InvalidInteger { name, value } => {
                write!(formatter, "{name} must be a decimal integer, got {value:?}")
            }
            Self::NonPositive { name, value } => {
                write!(formatter, "{name} must be greater than zero, got {value:?}")
            }
            Self::InvalidBoolean { name, value } => write!(
                formatter,
                "{name} must be 1/0, true/false, yes/no, or on/off, got {value:?}"
            ),
            Self::WorkersAboveAvailable {
                requested,
                available,
            } => write!(
                formatter,
                "R4_PARITY_WORKERS ({requested}) exceeds available_parallelism() ({available})"
            ),
            Self::InvalidPath { name, value } => {
                write!(formatter, "{name} must be a non-empty path, got {value:?}")
            }
            Self::OutOfRange {
                name,
                value,
                minimum,
                maximum,
            } => write!(
                formatter,
                "{name} must be in {minimum}..={maximum}, got {value}"
            ),
            Self::InvalidAdaptiveMaximum { value } => write!(
                formatter,
                "R4_PARITY_GEN_TOKENS must be one of 1, 2, 4, or 8 for cumulative adaptive checkpoints, got {value}"
            ),
            Self::BudgetOverflow => write!(
                formatter,
                "teacher-parity budgets overflow host addressable work arithmetic"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingShapeError {
    NoHostCapacity,
    CanonicalStreamsRequired { streams: usize },
    SingleRunRequired { runs: usize },
}

impl fmt::Display for BindingShapeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHostCapacity => write!(formatter, "available_parallelism reported zero"),
            Self::CanonicalStreamsRequired { streams } => write!(
                formatter,
                "binding parity requires the {CANONICAL_PARITY_STREAMS} canonical private streams, got {streams}"
            ),
            Self::SingleRunRequired { runs } => write!(
                formatter,
                "fixture-present adaptive parity requires exactly one causal run, got {runs}"
            ),
        }
    }
}

impl std::error::Error for BindingShapeError {}

/// A binding live run retains the canonical semantic cohort. The exact
/// output-row worker count is independently selected by the bounded probe.
pub fn validate_binding_host_shape(
    config: &ParityConfig,
    available: usize,
) -> Result<(), BindingShapeError> {
    if available == 0 {
        return Err(BindingShapeError::NoHostCapacity);
    }
    if config.streams.get() != CANONICAL_PARITY_STREAMS {
        return Err(BindingShapeError::CanonicalStreamsRequired {
            streams: config.streams.get(),
        });
    }
    if config.runs.get() != 1 {
        return Err(BindingShapeError::SingleRunRequired {
            runs: config.runs.get(),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivateStreamEvidenceError {
    SingleStream,
    SeedCount { actual: usize, expected: usize },
    OutputCount { actual: usize, expected: usize },
    DuplicateSeedIdentity { first: usize, duplicate: usize },
}

impl fmt::Display for PrivateStreamEvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SingleStream => write!(formatter, "private multistream evidence requires S > 1"),
            Self::SeedCount { actual, expected } => {
                write!(formatter, "lane seed identities {actual}/{expected}")
            }
            Self::OutputCount { actual, expected } => {
                write!(formatter, "lane output identities {actual}/{expected}")
            }
            Self::DuplicateSeedIdentity { first, duplicate } => write!(
                formatter,
                "lane {duplicate} duplicates lane {first} seed identity (fan-out evidence is not private multistream evidence)"
            ),
        }
    }
}

impl std::error::Error for PrivateStreamEvidenceError {}

/// Require complete ordered lane records and distinct seed identities. Output
/// identities may coincide by model behavior; the caller separately proves S
/// private states were instantiated and completed.
pub fn validate_private_multistream_evidence(
    seed_cids: &[String],
    output_cids: &[String],
    expected_streams: usize,
) -> Result<(), PrivateStreamEvidenceError> {
    if expected_streams <= 1 {
        return Err(PrivateStreamEvidenceError::SingleStream);
    }
    if seed_cids.len() != expected_streams {
        return Err(PrivateStreamEvidenceError::SeedCount {
            actual: seed_cids.len(),
            expected: expected_streams,
        });
    }
    if output_cids.len() != expected_streams {
        return Err(PrivateStreamEvidenceError::OutputCount {
            actual: output_cids.len(),
            expected: expected_streams,
        });
    }
    for duplicate in 0..seed_cids.len() {
        if let Some(first) = seed_cids[..duplicate]
            .iter()
            .position(|identity| identity == &seed_cids[duplicate])
        {
            return Err(PrivateStreamEvidenceError::DuplicateSeedIdentity { first, duplicate });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunStatus {
    Pass,
    Fail,
    Unavailable,
    Aborted,
    NotRun,
}

impl RunStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Unavailable => "UNAVAILABLE",
            Self::Aborted => "ABORTED",
            Self::NotRun => "NOT_RUN",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventKind {
    SuiteStarted,
    FixtureStatus,
    PhaseStarted,
    PhaseCompleted,
    WorkStarted,
    WorkCompleted,
    Heartbeat,
    WorkFailed,
    SuiteCompleted,
    SuiteAborted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FixtureVerdict {
    Available,
    Unavailable,
    Failed,
    NotRun,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureStatus {
    pub verdict: FixtureVerdict,
    pub cid: Option<String>,
    pub reason: Option<String>,
}

impl FixtureStatus {
    pub fn available(cid: impl Into<String>) -> Self {
        Self {
            verdict: FixtureVerdict::Available,
            cid: Some(cid.into()),
            reason: None,
        }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            verdict: FixtureVerdict::Unavailable,
            cid: None,
            reason: Some(reason.into()),
        }
    }

    pub fn unavailable_with_cid(cid: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            verdict: FixtureVerdict::Unavailable,
            cid: Some(cid.into()),
            reason: Some(reason.into()),
        }
    }

    pub fn failed(reason: impl Into<String>) -> Self {
        Self {
            verdict: FixtureVerdict::Failed,
            cid: None,
            reason: Some(reason.into()),
        }
    }

    pub fn failed_with_cid(cid: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            verdict: FixtureVerdict::Failed,
            cid: Some(cid.into()),
            reason: Some(reason.into()),
        }
    }

    pub fn not_run(reason: impl Into<String>) -> Self {
        Self {
            verdict: FixtureVerdict::NotRun,
            cid: None,
            reason: Some(reason.into()),
        }
    }

    pub fn not_run_with_cid(cid: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            verdict: FixtureVerdict::NotRun,
            cid: Some(cid.into()),
            reason: Some(reason.into()),
        }
    }
}

/// Discriminated shape of an exact-probe artifact before full-report decoding.
#[derive(Debug, Clone, PartialEq)]
pub enum ExactProbeArtifact {
    /// Candidate full report; the model-source schema owner still performs its
    /// complete typed deserialization and admission validation.
    QualifiedCandidate(Value),
    /// Truthful state/refusal written before a full qualified report existed.
    NonQualified(ExactProbeNonQualifiedState),
}

/// Minimal fail-closed projection of `EXACT_MULTICORE_PROBE_STATE`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactProbeNonQualifiedState {
    pub event: String,
    pub probe_status: String,
    pub run_status: RunStatus,
    pub reason: String,
}

impl ExactProbeNonQualifiedState {
    /// Stable run-level reason returned by the BDD fixture loader.
    pub fn outcome_reason(&self) -> String {
        let detail = format!(
            "exact probe state {} / {}: {}",
            self.event, self.probe_status, self.reason
        );
        match self.run_status {
            RunStatus::NotRun => format!("NOT_RUN / REFUSED: {detail}"),
            RunStatus::Unavailable => format!("UNAVAILABLE: {detail}"),
            RunStatus::Fail => format!("FAILED: {detail}"),
            RunStatus::Aborted => format!("ABORTED: {detail}"),
            RunStatus::Pass => format!("FAILED: non-qualified probe state claimed PASS: {detail}"),
        }
    }

    /// Preserve the state artifact's exact bytes while keeping fixture and run
    /// verdicts distinct. A probe abort is non-qualified/NOT_RUN as a fixture,
    /// while the enclosing run retains its ABORTED status.
    pub fn fixture_status(&self, cid: impl Into<String>) -> FixtureStatus {
        let cid = cid.into();
        match self.run_status {
            RunStatus::Unavailable => FixtureStatus::unavailable_with_cid(cid, self.reason.clone()),
            RunStatus::Fail | RunStatus::Pass => {
                FixtureStatus::failed_with_cid(cid, self.reason.clone())
            }
            RunStatus::NotRun | RunStatus::Aborted => {
                FixtureStatus::not_run_with_cid(cid, self.reason.clone())
            }
        }
    }
}

/// Bind a present non-qualified probe artifact to the final run metadata.
///
/// A state/refusal is legitimate evidence for a non-PASS outcome, but it may
/// never be reinterpreted as admission. Its exact persisted bytes, projected
/// fixture verdict, and reason must be the same values recorded when the BDD
/// first consumed the artifact.
pub fn validate_nonqualified_probe_prepublication(
    overall_status: RunStatus,
    fixture: Option<&FixtureStatus>,
    state: &ExactProbeNonQualifiedState,
    artifact_cid: &str,
) -> Result<(), String> {
    if overall_status == RunStatus::Pass {
        return Err("FAIL: a PASS may not publish a non-qualified exact probe state".to_owned());
    }
    let fixture = fixture.ok_or_else(|| {
        "FAIL: present non-qualified exact probe has no explicit fixture status".to_owned()
    })?;
    let expected = state.fixture_status(artifact_cid);
    if fixture.cid.as_deref() != Some(artifact_cid) {
        return Err(format!(
            "FAIL: non-qualified exact probe fixture CID {:?} does not bind persisted artifact {artifact_cid}",
            fixture.cid
        ));
    }
    if fixture.verdict != expected.verdict {
        return Err(format!(
            "FAIL: non-qualified exact probe fixture verdict {:?} does not match state {} / {}",
            fixture.verdict, state.event, state.probe_status
        ));
    }
    if fixture.reason.as_deref() != Some(state.reason.as_str()) {
        return Err(
            "FAIL: non-qualified exact probe fixture reason does not match persisted state"
                .to_owned(),
        );
    }
    Ok(())
}

/// Parse only enough exact-probe structure to distinguish a candidate full
/// report from the versioned, explicitly non-qualified state record. Unknown
/// records, malformed JSON, contradictory status/event pairs, and any state
/// that claims qualification fail closed before typed report deserialization.
pub fn classify_exact_probe_artifact(
    bytes: &[u8],
    expected_schema: &str,
) -> Result<ExactProbeArtifact, String> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("FAILED: exact probe JSON is malformed: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "FAILED: exact probe JSON root must be an object".to_owned())?;
    let schema = object
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| "FAILED: exact probe JSON omitted string schema".to_owned())?;
    if schema != expected_schema {
        return Err(format!(
            "FAILED: exact probe schema {schema:?} does not match {expected_schema:?}"
        ));
    }

    let Some(record) = object.get("record") else {
        return Ok(ExactProbeArtifact::QualifiedCandidate(value));
    };
    let record = record
        .as_str()
        .ok_or_else(|| "FAILED: exact probe record discriminator must be a string".to_owned())?;
    if record != "EXACT_MULTICORE_PROBE_STATE" {
        return Err(format!(
            "FAILED: unknown exact probe record discriminator {record:?}"
        ));
    }
    if object.get("qualifies_full_run").and_then(Value::as_bool) != Some(false) {
        return Err(
            "FAILED: exact probe state must explicitly set qualifies_full_run=false".to_owned(),
        );
    }
    let event = object
        .get("event")
        .and_then(Value::as_str)
        .filter(|event| !event.trim().is_empty())
        .ok_or_else(|| "FAILED: exact probe state omitted nonempty string event".to_owned())?;
    let probe_status = object
        .get("status")
        .and_then(Value::as_str)
        .filter(|status| !status.trim().is_empty())
        .ok_or_else(|| "FAILED: exact probe state omitted nonempty string status".to_owned())?;
    let run_status = match (event, probe_status) {
        ("NOT_RUN", "REFUSE_FULL_RUN") | ("RUNNING", "NOT_QUALIFIED") => RunStatus::NotRun,
        ("UNAVAILABLE", "UNAVAILABLE") => RunStatus::Unavailable,
        ("FAIL", "FAIL") => RunStatus::Fail,
        ("ABORTED", "ABORTED") => RunStatus::Aborted,
        _ => {
            return Err(format!(
                "FAILED: unknown or contradictory exact probe state event/status {event:?}/{probe_status:?}"
            ))
        }
    };
    let reason = object
        .get("reason")
        .and_then(Value::as_str)
        .filter(|reason| !reason.trim().is_empty())
        .map(str::to_owned)
        .or_else(|| {
            (event == "RUNNING" && probe_status == "NOT_QUALIFIED").then(|| {
                "exact probe is still running and has not qualified the full run".to_owned()
            })
        })
        .ok_or_else(|| "FAILED: terminal exact probe state omitted nonempty reason".to_owned())?;

    Ok(ExactProbeArtifact::NonQualified(
        ExactProbeNonQualifiedState {
            event: event.to_owned(),
            probe_status: probe_status.to_owned(),
            run_status,
            reason,
        },
    ))
}

/// Furthest graph-bundle stage reached by a failed teacher-free preflight.
///
/// The distinction prevents a content hash from being promoted into parser or
/// runtime evidence. Every variant is non-PASS; it only identifies which
/// fixture owned the refusal and which later fixture remained unexecuted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeacherFreeGraphFailureStage {
    /// Graph/report bytes may have been presence-checked or hashed, but the
    /// report was not accepted.
    NotReached,
    /// Reading, parsing, or validating the graph report itself failed.
    ReportFailed,
    /// The graph report was accepted, but graph loading was not attempted.
    ReportAccepted,
    /// The report was accepted and graph loading or graph execution was tried.
    GraphLoadAttempted,
}

/// Project a durable teacher-free graph-stage refusal into the final run
/// metadata without promoting unopened teacher source files to AVAILABLE.
/// `compiled_inputs_parsed` is supplied by the ordered preflight owner; a CID
/// alone proves bytes were hashed, not that their parser accepted them.
/// `graph_stage` distinguishes report read/parse/schema refusal, accepted
/// provenance, and graph load/execution so later fixtures remain truthfully
/// NOT_RUN rather than inheriting an earlier fixture's failure.
pub fn apply_teacher_free_preflight_failure_metadata(
    metadata: &mut RunMetadata,
    report_path: &Path,
    report_cid: Option<&str>,
    report: &Value,
    mut preflight_status: FixtureStatus,
    compiled_inputs_parsed: bool,
    graph_stage: TeacherFreeGraphFailureStage,
) {
    let refusal_reason = preflight_status
        .reason
        .clone()
        .unwrap_or_else(|| "teacher-free preflight refused without a reason".to_owned());
    metadata
        .paths
        .entry("teacher_free_preflight_report".to_owned())
        .or_insert_with(|| report_path.display().to_string());
    metadata.fixtures.insert(
        "teacher_weights".to_owned(),
        FixtureStatus::not_run(
            "teacher source presence was inspected by metadata only; weights were not opened",
        ),
    );
    metadata.fixtures.insert(
        "teacher_config".to_owned(),
        FixtureStatus::not_run(
            "teacher source presence was inspected by metadata only; config was not opened",
        ),
    );
    if let Some(report_cid) = report_cid {
        preflight_status.cid = Some(report_cid.to_owned());
        metadata.identities.insert(
            "teacher_free_s4_preflight".to_owned(),
            report_cid.to_owned(),
        );
    }
    metadata
        .fixtures
        .insert("teacher_free_s4_preflight".to_owned(), preflight_status);

    let input_cid = |name: &str| {
        report
            .pointer(&format!("/inputs/{name}/cid"))
            .and_then(Value::as_str)
            .filter(|cid| cid.starts_with("blake3:"))
    };
    if compiled_inputs_parsed {
        for (input, fixture) in [
            ("tokenizer", "tokenizer"),
            ("legacy_artifact", "tla_artifact"),
            ("legacy_store", "tls_store"),
        ] {
            if let Some(cid) = input_cid(input) {
                metadata
                    .fixtures
                    .insert(fixture.to_owned(), FixtureStatus::available(cid));
                metadata
                    .identities
                    .insert(fixture.to_owned(), cid.to_owned());
            }
        }
    }
    if let Some(cid) = input_cid("graph") {
        metadata.fixtures.insert(
            "r4g1_graph".to_owned(),
            if graph_stage == TeacherFreeGraphFailureStage::GraphLoadAttempted {
                FixtureStatus::failed_with_cid(cid, refusal_reason.clone())
            } else {
                FixtureStatus::not_run_with_cid(
                    cid,
                    format!(
                        "graph bytes were present and hashed, but graph load was not attempted: {refusal_reason}"
                    ),
                )
            },
        );
        metadata
            .identities
            .insert("r4g1_graph".to_owned(), cid.to_owned());
    }
    if let Some(cid) = input_cid("graph_report") {
        metadata.fixtures.insert(
            "r4g1_graph_report".to_owned(),
            match graph_stage {
                TeacherFreeGraphFailureStage::ReportFailed => {
                    FixtureStatus::failed_with_cid(cid, refusal_reason.clone())
                }
                TeacherFreeGraphFailureStage::ReportAccepted
                | TeacherFreeGraphFailureStage::GraphLoadAttempted => {
                    FixtureStatus::available(cid)
                }
                TeacherFreeGraphFailureStage::NotReached => FixtureStatus::not_run_with_cid(
                    cid,
                    format!(
                        "graph-report bytes were present and hashed, but report validation was not attempted: {refusal_reason}"
                    ),
                ),
            },
        );
        metadata
            .identities
            .insert("r4g1_graph_report".to_owned(), cid.to_owned());
    } else if graph_stage == TeacherFreeGraphFailureStage::ReportFailed {
        metadata.fixtures.insert(
            "r4g1_graph_report".to_owned(),
            FixtureStatus::failed(refusal_reason),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerSnapshot {
    /// Backward-compatible generic worker bound; interpreted as the bounded
    /// trajectory pool outside exact teacher matrix execution.
    pub requested_workers: NonZeroUsize,
    pub effective_workers: NonZeroUsize,
    pub requested_streams: NonZeroUsize,
    pub effective_streams: NonZeroUsize,
    pub batch_per_worker: NonZeroUsize,
    pub configured_trajectory_workers: NonZeroUsize,
    pub effective_trajectory_workers: NonZeroUsize,
    pub configured_row_workers: NonZeroUsize,
    pub effective_row_workers: NonZeroUsize,
}

impl SchedulerSnapshot {
    pub fn from_config(config: &ParityConfig) -> Self {
        Self {
            requested_workers: config.workers,
            effective_workers: config.workers,
            requested_streams: config.streams,
            effective_streams: config.streams,
            batch_per_worker: config.batch_per_worker,
            configured_trajectory_workers: config.workers,
            effective_trajectory_workers: config.workers,
            configured_row_workers: config.workers,
            effective_row_workers: config.workers,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunMetadata {
    pub scheduler: SchedulerSnapshot,
    pub backend: String,
    pub kernel: String,
    pub isa: String,
    pub identities: BTreeMap<String, String>,
    pub model_geometry: BTreeMap<String, u64>,
    pub budgets: BTreeMap<String, u64>,
    pub fixtures: BTreeMap<String, FixtureStatus>,
    /// Absolute durable artifact paths for the current run.
    pub paths: BTreeMap<String, String>,
}

impl RunMetadata {
    pub fn new(
        scheduler: SchedulerSnapshot,
        backend: impl Into<String>,
        kernel: impl Into<String>,
        isa: impl Into<String>,
    ) -> Self {
        Self {
            scheduler,
            backend: backend.into(),
            kernel: kernel.into(),
            isa: isa.into(),
            identities: BTreeMap::new(),
            model_geometry: BTreeMap::new(),
            budgets: BTreeMap::new(),
            fixtures: BTreeMap::new(),
            paths: BTreeMap::new(),
        }
    }

    pub fn with_identity(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.identities.insert(name.into(), value.into());
        self
    }

    pub fn with_model_geometry(mut self, name: impl Into<String>, value: u64) -> Self {
        self.model_geometry.insert(name.into(), value);
        self
    }

    pub fn with_budget(mut self, name: impl Into<String>, value: u64) -> Self {
        self.budgets.insert(name.into(), value);
        self
    }

    pub fn with_fixture(mut self, name: impl Into<String>, status: FixtureStatus) -> Self {
        self.fixtures.insert(name.into(), status);
        self
    }

    pub fn with_path(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.paths.insert(name.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamState {
    Queued,
    Active,
    Completed,
    Failed,
    Aborted,
    NotRun,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamProgress {
    pub stream_id: String,
    pub phase: String,
    pub state: StreamState,
    pub logical_forwards_completed: u64,
    pub logical_forwards_total: u64,
    pub tokens_completed: u64,
    pub tokens_total: u64,
    pub active_forward_age_millis: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueueSnapshot {
    pub queue_depth: u64,
    pub active_streams: u64,
    pub peak_active_streams: u64,
    pub active_row_workers: u64,
    pub peak_active_row_workers: u64,
    pub completed_streams: u64,
    pub failed_streams: u64,
    pub active_worker_tasks: u64,
    pub completed_worker_tasks: u64,
    pub failed_worker_tasks: u64,
    pub longest_active_millis: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProgressUnit {
    #[default]
    LogicalForwards,
    WorkerTasks,
    ScalarTerms,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RateSnapshot {
    pub rolling_forwards_per_second: Option<f64>,
    pub cumulative_forwards_per_second: Option<f64>,
    pub seconds_per_forward: Option<f64>,
    pub tokens_per_second: Option<f64>,
    pub rolling_worker_tasks_per_second: Option<f64>,
    pub rolling_scalar_terms_per_second: Option<f64>,
    pub cumulative_scalar_terms_per_second: Option<f64>,
    pub eta_progress_unit: ProgressUnit,
    pub eta_progress_completed: u64,
    pub eta_progress_total: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkPlan {
    pub logical_forwards: u64,
    pub tokens: u64,
    pub physical_batches: u64,
    pub matrix_calls: u64,
    pub batched_matrix_calls: u64,
    pub max_matrix_batch_width: u64,
    pub padded_forwards: u64,
    pub cache_hits: u64,
    pub streams: u64,
    pub worker_tasks: u64,
    pub row_tiles: u64,
    pub output_cells: u64,
    pub scalar_terms: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkSnapshot {
    pub plan: WorkPlan,
    pub logical_forwards: u64,
    pub tokens: u64,
    pub physical_batches: u64,
    pub matrix_calls: u64,
    pub batched_matrix_calls: u64,
    pub max_matrix_batch_width: u64,
    pub padded_forwards: u64,
    pub cache_hits: u64,
    pub streams: u64,
    pub worker_tasks: u64,
    pub row_tiles: u64,
    pub output_cells: u64,
    pub scalar_terms: u64,
    pub failed_streams: u64,
    pub failed_worker_tasks: u64,
}

/// Select the cheapest monotonic unit that advances inside a physical exact
/// forward. Scalar terms give the finest ETA signal; worker tasks are the
/// fallback for producers without scalar accounting, and logical forwards
/// remain the truthful final fallback.
pub fn heartbeat_progress_units(work: &WorkSnapshot) -> (ProgressUnit, u64, u64) {
    if work.plan.scalar_terms != 0 {
        (
            ProgressUnit::ScalarTerms,
            work.scalar_terms,
            work.plan.scalar_terms,
        )
    } else if work.plan.worker_tasks != 0 {
        (
            ProgressUnit::WorkerTasks,
            work.worker_tasks,
            work.plan.worker_tasks,
        )
    } else {
        (
            ProgressUnit::LogicalForwards,
            work.logical_forwards,
            work.plan.logical_forwards,
        )
    }
}

pub struct WorkCounters {
    plan: Mutex<WorkPlan>,
    plan_state: AtomicU8,
    work_started: AtomicBool,
    plan_violation: AtomicBool,
    logical_forwards: AtomicU64,
    tokens: AtomicU64,
    physical_batches: AtomicU64,
    matrix_calls: AtomicU64,
    batched_matrix_calls: AtomicU64,
    max_matrix_batch_width: AtomicU64,
    padded_forwards: AtomicU64,
    cache_hits: AtomicU64,
    streams: AtomicU64,
    worker_tasks: AtomicU64,
    row_tiles: AtomicU64,
    output_cells: AtomicU64,
    scalar_terms: AtomicU64,
    failed_streams: AtomicU64,
    failed_worker_tasks: AtomicU64,
}

impl WorkCounters {
    pub fn new(plan: WorkPlan) -> Self {
        Self {
            plan: Mutex::new(plan),
            plan_state: AtomicU8::new(2),
            work_started: AtomicBool::new(false),
            plan_violation: AtomicBool::new(false),
            logical_forwards: AtomicU64::new(0),
            tokens: AtomicU64::new(0),
            physical_batches: AtomicU64::new(0),
            matrix_calls: AtomicU64::new(0),
            batched_matrix_calls: AtomicU64::new(0),
            max_matrix_batch_width: AtomicU64::new(0),
            padded_forwards: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            streams: AtomicU64::new(0),
            worker_tasks: AtomicU64::new(0),
            row_tiles: AtomicU64::new(0),
            output_cells: AtomicU64::new(0),
            scalar_terms: AtomicU64::new(0),
            failed_streams: AtomicU64::new(0),
            failed_worker_tasks: AtomicU64::new(0),
        }
    }

    /// Construct counters whose exact work totals are not known yet. The
    /// caller must install them exactly once, before recording any work.
    pub fn unplanned() -> Self {
        Self {
            plan: Mutex::new(WorkPlan::default()),
            plan_state: AtomicU8::new(0),
            work_started: AtomicBool::new(false),
            plan_violation: AtomicBool::new(false),
            logical_forwards: AtomicU64::new(0),
            tokens: AtomicU64::new(0),
            physical_batches: AtomicU64::new(0),
            matrix_calls: AtomicU64::new(0),
            batched_matrix_calls: AtomicU64::new(0),
            max_matrix_batch_width: AtomicU64::new(0),
            padded_forwards: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            streams: AtomicU64::new(0),
            worker_tasks: AtomicU64::new(0),
            row_tiles: AtomicU64::new(0),
            output_cells: AtomicU64::new(0),
            scalar_terms: AtomicU64::new(0),
            failed_streams: AtomicU64::new(0),
            failed_worker_tasks: AtomicU64::new(0),
        }
    }

    /// Install the precomputed suite plan before any live forward is admitted.
    pub fn set_plan(&self, plan: WorkPlan) -> Result<(), PlanInstallError> {
        self.plan_state
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| PlanInstallError::AlreadyInstalled)?;
        if self.work_started.load(Ordering::Acquire) {
            self.plan_state.store(0, Ordering::Release);
            return Err(PlanInstallError::WorkAlreadyStarted);
        }
        *self
            .plan
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = plan;
        self.plan_state.store(2, Ordering::Release);
        if self.work_started.load(Ordering::Acquire) {
            self.plan_violation.store(true, Ordering::Release);
            return Err(PlanInstallError::WorkAlreadyStarted);
        }
        Ok(())
    }

    /// Close an adaptive upper-bound plan to the work actually selected. Every
    /// field must move monotonically downward without falling below already
    /// observed work. Later non-adaptive phases may continue recording against
    /// the retained totals.
    pub fn reduce_plan(&self, reduced: WorkPlan) -> Result<(), PlanInstallError> {
        if self.plan_state.load(Ordering::Acquire) != 2 {
            return Err(PlanInstallError::AlreadyInstalled);
        }
        let mut plan = self
            .plan
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let observed = WorkPlan {
            logical_forwards: self.logical_forwards.load(Ordering::Acquire),
            tokens: self.tokens.load(Ordering::Acquire),
            physical_batches: self.physical_batches.load(Ordering::Acquire),
            matrix_calls: self.matrix_calls.load(Ordering::Acquire),
            batched_matrix_calls: self.batched_matrix_calls.load(Ordering::Acquire),
            max_matrix_batch_width: self.max_matrix_batch_width.load(Ordering::Acquire),
            padded_forwards: self.padded_forwards.load(Ordering::Acquire),
            cache_hits: self.cache_hits.load(Ordering::Acquire),
            streams: self.streams.load(Ordering::Acquire),
            worker_tasks: self.worker_tasks.load(Ordering::Acquire),
            row_tiles: self.row_tiles.load(Ordering::Acquire),
            output_cells: self.output_cells.load(Ordering::Acquire),
            scalar_terms: self.scalar_terms.load(Ordering::Acquire),
        };
        macro_rules! check_field {
            ($field:ident) => {
                if reduced.$field > plan.$field {
                    return Err(PlanInstallError::ReductionWouldIncrease);
                }
                if reduced.$field < observed.$field {
                    return Err(PlanInstallError::ReductionBelowObserved);
                }
            };
        }
        check_field!(logical_forwards);
        check_field!(tokens);
        check_field!(physical_batches);
        check_field!(matrix_calls);
        check_field!(batched_matrix_calls);
        check_field!(max_matrix_batch_width);
        check_field!(padded_forwards);
        check_field!(cache_hits);
        check_field!(streams);
        check_field!(worker_tasks);
        check_field!(row_tiles);
        check_field!(output_cells);
        check_field!(scalar_terms);
        *plan = reduced;
        Ok(())
    }

    fn begin_recording(&self) {
        self.work_started.store(true, Ordering::Release);
        if self.plan_state.load(Ordering::Acquire) != 2 {
            self.plan_violation.store(true, Ordering::Release);
        }
    }

    pub fn record_batch(
        &self,
        logical_forwards: u64,
        padded_forwards: u64,
        cache_hits: u64,
        worker_tasks: u64,
        row_tiles: u64,
    ) {
        self.begin_recording();
        self.physical_batches.fetch_add(1, Ordering::Relaxed);
        self.logical_forwards
            .fetch_add(logical_forwards, Ordering::Relaxed);
        self.padded_forwards
            .fetch_add(padded_forwards, Ordering::Relaxed);
        self.cache_hits.fetch_add(cache_hits, Ordering::Relaxed);
        self.worker_tasks.fetch_add(worker_tasks, Ordering::Relaxed);
        self.row_tiles.fetch_add(row_tiles, Ordering::Relaxed);
    }

    pub fn record_logical_forwards(&self, count: u64) {
        self.begin_recording();
        self.logical_forwards.fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_tokens(&self, count: u64) {
        if count == 0 {
            return;
        }
        self.begin_recording();
        self.tokens.fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_physical_batch(&self) {
        self.begin_recording();
        self.physical_batches.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_physical_batches(&self, count: u64) {
        if count == 0 {
            return;
        }
        self.begin_recording();
        self.physical_batches.fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_matrix_calls(&self, count: u64) {
        if count == 0 {
            return;
        }
        self.begin_recording();
        self.matrix_calls.fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_batched_matrix_calls(&self, count: u64) {
        if count == 0 {
            return;
        }
        self.begin_recording();
        self.batched_matrix_calls
            .fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_max_matrix_batch_width(&self, width: usize) {
        if width == 0 {
            return;
        }
        self.begin_recording();
        self.max_matrix_batch_width
            .fetch_max(width as u64, Ordering::AcqRel);
    }

    pub fn record_padded_forwards(&self, count: u64) {
        self.begin_recording();
        self.padded_forwards.fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_cache_hits(&self, count: u64) {
        self.begin_recording();
        self.cache_hits.fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_stream_completed(&self) {
        self.begin_recording();
        self.streams.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_stream_failed(&self) {
        self.begin_recording();
        self.failed_streams.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_worker_tasks(&self, count: u64) {
        self.begin_recording();
        self.worker_tasks.fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_worker_task_failed(&self) {
        self.begin_recording();
        self.failed_worker_tasks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_row_tiles(&self, count: u64) {
        if count == 0 {
            return;
        }
        self.begin_recording();
        self.row_tiles.fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_output_cells(&self, count: u64) {
        if count == 0 {
            return;
        }
        self.begin_recording();
        self.output_cells.fetch_add(count, Ordering::Relaxed);
    }

    pub fn record_scalar_terms(&self, count: u64) {
        if count == 0 {
            return;
        }
        self.begin_recording();
        self.scalar_terms.fetch_add(count, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> WorkSnapshot {
        WorkSnapshot {
            plan: *self
                .plan
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            logical_forwards: self.logical_forwards.load(Ordering::Relaxed),
            tokens: self.tokens.load(Ordering::Relaxed),
            physical_batches: self.physical_batches.load(Ordering::Relaxed),
            matrix_calls: self.matrix_calls.load(Ordering::Relaxed),
            batched_matrix_calls: self.batched_matrix_calls.load(Ordering::Relaxed),
            max_matrix_batch_width: self.max_matrix_batch_width.load(Ordering::Relaxed),
            padded_forwards: self.padded_forwards.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            streams: self.streams.load(Ordering::Relaxed),
            worker_tasks: self.worker_tasks.load(Ordering::Relaxed),
            row_tiles: self.row_tiles.load(Ordering::Relaxed),
            output_cells: self.output_cells.load(Ordering::Relaxed),
            scalar_terms: self.scalar_terms.load(Ordering::Relaxed),
            failed_streams: self.failed_streams.load(Ordering::Relaxed),
            failed_worker_tasks: self.failed_worker_tasks.load(Ordering::Relaxed),
        }
    }

    /// A requested PASS is accepted only when every exact counter closes and
    /// no worker or stream reported failure. Other truthful terminal states are
    /// preserved without manufacturing a pass/fail interpretation.
    pub fn completion_status(&self, requested: RunStatus) -> CompletionStatus {
        if requested != RunStatus::Pass {
            return CompletionStatus {
                status: requested,
                detail: None,
            };
        }
        let snapshot = self.snapshot();
        let mut mismatches = Vec::new();
        if self.plan_state.load(Ordering::Acquire) != 2 {
            mismatches.push("work plan was not installed".to_owned());
        }
        if self.plan_violation.load(Ordering::Acquire) {
            mismatches.push("work was recorded before the plan was installed".to_owned());
        }
        push_mismatch(
            &mut mismatches,
            "logical_forwards",
            snapshot.logical_forwards,
            snapshot.plan.logical_forwards,
        );
        push_mismatch(
            &mut mismatches,
            "tokens",
            snapshot.tokens,
            snapshot.plan.tokens,
        );
        push_mismatch(
            &mut mismatches,
            "physical_batches",
            snapshot.physical_batches,
            snapshot.plan.physical_batches,
        );
        push_mismatch(
            &mut mismatches,
            "matrix_calls",
            snapshot.matrix_calls,
            snapshot.plan.matrix_calls,
        );
        push_mismatch(
            &mut mismatches,
            "batched_matrix_calls",
            snapshot.batched_matrix_calls,
            snapshot.plan.batched_matrix_calls,
        );
        push_mismatch(
            &mut mismatches,
            "max_matrix_batch_width",
            snapshot.max_matrix_batch_width,
            snapshot.plan.max_matrix_batch_width,
        );
        push_mismatch(
            &mut mismatches,
            "padded_forwards",
            snapshot.padded_forwards,
            snapshot.plan.padded_forwards,
        );
        push_mismatch(
            &mut mismatches,
            "cache_hits",
            snapshot.cache_hits,
            snapshot.plan.cache_hits,
        );
        push_mismatch(
            &mut mismatches,
            "streams",
            snapshot.streams,
            snapshot.plan.streams,
        );
        push_mismatch(
            &mut mismatches,
            "worker_tasks",
            snapshot.worker_tasks,
            snapshot.plan.worker_tasks,
        );
        push_mismatch(
            &mut mismatches,
            "row_tiles",
            snapshot.row_tiles,
            snapshot.plan.row_tiles,
        );
        push_mismatch(
            &mut mismatches,
            "output_cells",
            snapshot.output_cells,
            snapshot.plan.output_cells,
        );
        push_mismatch(
            &mut mismatches,
            "scalar_terms",
            snapshot.scalar_terms,
            snapshot.plan.scalar_terms,
        );
        if snapshot.failed_streams != 0 {
            mismatches.push(format!("failed_streams {}", snapshot.failed_streams));
        }
        if snapshot.failed_worker_tasks != 0 {
            mismatches.push(format!(
                "failed_worker_tasks {}",
                snapshot.failed_worker_tasks
            ));
        }
        if mismatches.is_empty() {
            CompletionStatus {
                status: RunStatus::Pass,
                detail: None,
            }
        } else {
            CompletionStatus {
                status: RunStatus::Fail,
                detail: Some(format!(
                    "incomplete or failed parity work: {}",
                    mismatches.join(", ")
                )),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanInstallError {
    AlreadyInstalled,
    WorkAlreadyStarted,
    ReductionWouldIncrease,
    ReductionBelowObserved,
}

impl fmt::Display for PlanInstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyInstalled => write!(formatter, "parity work plan is already installed"),
            Self::WorkAlreadyStarted => {
                write!(formatter, "parity work began before its plan was installed")
            }
            Self::ReductionWouldIncrease => {
                write!(
                    formatter,
                    "adaptive plan reduction would increase a planned field"
                )
            }
            Self::ReductionBelowObserved => {
                write!(
                    formatter,
                    "adaptive plan reduction would fall below observed work"
                )
            }
        }
    }
}

impl std::error::Error for PlanInstallError {}

fn push_mismatch(out: &mut Vec<String>, label: &str, actual: u64, planned: u64) {
    if actual != planned {
        out.push(format!("{label} {actual}/{planned}"));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionStatus {
    pub status: RunStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EtaStatus {
    WarmingUp,
    Estimated,
    Unavailable,
    Stall,
}

#[derive(Debug, Clone, Copy)]
pub struct EtaInput {
    pub completed: u64,
    pub total: u64,
    pub elapsed: Duration,
    pub last_progress_age: Duration,
    pub stall_after: Duration,
    pub minimum_samples: u64,
}

impl Default for EtaInput {
    fn default() -> Self {
        Self {
            completed: 0,
            total: 0,
            elapsed: Duration::ZERO,
            last_progress_age: Duration::ZERO,
            stall_after: Duration::from_secs(MINIMUM_DEFAULT_STALL_SECONDS),
            minimum_samples: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EtaSnapshot {
    pub status: EtaStatus,
    pub completed: u64,
    pub total: u64,
    pub elapsed_seconds: u64,
    pub last_progress_age_seconds: u64,
    pub remaining_seconds: Option<u64>,
}

pub fn estimate_eta(input: EtaInput) -> EtaSnapshot {
    let mut result = EtaSnapshot {
        status: EtaStatus::Unavailable,
        completed: input.completed,
        total: input.total,
        elapsed_seconds: input.elapsed.as_secs(),
        last_progress_age_seconds: input.last_progress_age.as_secs(),
        remaining_seconds: None,
    };
    if input.total == 0 || input.completed > input.total {
        return result;
    }
    if input.completed < input.total && input.last_progress_age >= input.stall_after {
        result.status = EtaStatus::Stall;
        return result;
    }
    if input.completed < input.minimum_samples || input.elapsed.is_zero() {
        result.status = EtaStatus::WarmingUp;
        return result;
    }
    let elapsed = input.elapsed.as_secs_f64();
    let rate = input.completed as f64 / elapsed;
    if !rate.is_finite() || rate <= 0.0 {
        return result;
    }
    let remaining = input.total - input.completed;
    let estimate = (remaining as f64 / rate).ceil();
    if estimate.is_finite() && estimate >= 0.0 && estimate <= u64::MAX as f64 {
        result.status = EtaStatus::Estimated;
        result.remaining_seconds = Some(estimate as u64);
    }
    result
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "availability", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Measurement<T> {
    Available { value: T },
    Unavailable { reason: String },
}

impl<T> Measurement<T> {
    pub fn available(value: T) -> Self {
        Self::Available { value }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self::Unavailable {
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "availability", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceAvailability {
    Available(Box<HostResourceSample>),
    Unavailable { reason: String },
}

impl ResourceAvailability {
    fn resident_set_bytes(&self) -> Option<NonZeroU64> {
        match self {
            Self::Available(sample) => match &sample.resident_set_bytes {
                Measurement::Available { value } => Some(*value),
                Measurement::Unavailable { .. } => None,
            },
            Self::Unavailable { .. } => None,
        }
    }

    pub fn set_max_sampled_resident_set_bytes(&mut self, value: Option<NonZeroU64>) {
        if let Self::Available(sample) = self {
            sample.max_sampled_resident_set_bytes = value.map_or_else(
                || Measurement::unavailable("no successful RSS sample was recorded"),
                Measurement::available,
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostResourceSample {
    pub sampled_unix_millis: u64,
    pub architecture: String,
    pub operating_system: String,
    pub operating_system_version: Measurement<String>,
    pub cpu_model: Measurement<String>,
    pub physical_cores: Measurement<NonZeroUsize>,
    pub logical_cores: Measurement<NonZeroUsize>,
    pub performance_cores: Measurement<NonZeroUsize>,
    pub efficiency_cores: Measurement<NonZeroUsize>,
    pub total_memory_bytes: Measurement<NonZeroU64>,
    pub resident_set_bytes: Measurement<NonZeroU64>,
    /// Highest successful periodic RSS sample observed by this run.
    pub max_sampled_resident_set_bytes: Measurement<NonZeroU64>,
    pub virtual_memory_bytes: Measurement<NonZeroU64>,
    /// Process CPU utilization; 800% means eight fully occupied logical CPUs.
    pub cpu_percent: Measurement<f64>,
    pub process_cpu_time_seconds: Measurement<f64>,
    pub thread_count: Measurement<NonZeroUsize>,
    /// macOS ps/sysctl do not expose a safe per-process peak-RSS sample.
    pub peak_resident_set_bytes: Measurement<NonZeroU64>,
}

#[cfg(target_os = "macos")]
pub fn sample_host_resources() -> ResourceAvailability {
    ResourceAvailability::Available(Box::new(sample_macos_resources()))
}

#[cfg(not(target_os = "macos"))]
pub fn sample_host_resources() -> ResourceAvailability {
    ResourceAvailability::Unavailable {
        reason: format!(
            "safe process RSS/CPU sampler is not implemented for {}",
            std::env::consts::OS
        ),
    }
}

#[cfg(target_os = "macos")]
fn sample_macos_resources() -> HostResourceSample {
    fn sysctl(name: &str) -> Result<String, String> {
        let output = std::process::Command::new("/usr/sbin/sysctl")
            .args(["-n", name])
            .output()
            .map_err(|error| format!("sysctl {name}: {error}"))?;
        if !output.status.success() {
            return Err(format!("sysctl {name} exited {}", output.status));
        }
        String::from_utf8(output.stdout)
            .map(|value| value.trim().to_owned())
            .map_err(|error| format!("sysctl {name} returned non-UTF-8 output: {error}"))
    }

    fn parse_nonzero<T>(name: &str, raw: String) -> Result<T, String>
    where
        T: std::str::FromStr,
        T::Err: fmt::Display,
    {
        raw.parse::<T>()
            .map_err(|error| format!("{name} returned {raw:?}: {error}"))
    }

    fn measured_nonempty(result: Result<String, String>, name: &str) -> Measurement<String> {
        match result {
            Ok(value) if !value.trim().is_empty() => Measurement::available(value),
            Ok(_) => Measurement::unavailable(format!("{name} returned an empty value")),
            Err(reason) => Measurement::unavailable(reason),
        }
    }

    fn measured_nonzero_usize(
        result: Result<String, String>,
        name: &str,
    ) -> Measurement<NonZeroUsize> {
        match result.and_then(|raw| parse_nonzero::<usize>(name, raw)) {
            Ok(value) => NonZeroUsize::new(value)
                .map(Measurement::available)
                .unwrap_or_else(|| Measurement::unavailable(format!("{name} reported zero"))),
            Err(reason) => Measurement::unavailable(reason),
        }
    }

    fn measured_nonzero_u64(result: Result<String, String>, name: &str) -> Measurement<NonZeroU64> {
        match result.and_then(|raw| parse_nonzero::<u64>(name, raw)) {
            Ok(value) => NonZeroU64::new(value)
                .map(Measurement::available)
                .unwrap_or_else(|| Measurement::unavailable(format!("{name} reported zero"))),
            Err(reason) => Measurement::unavailable(reason),
        }
    }

    #[derive(Debug)]
    struct ProcessSample {
        resident_set_bytes: NonZeroU64,
        virtual_memory_bytes: NonZeroU64,
        cpu_percent: f64,
        cpu_time_seconds: f64,
    }

    fn parse_cpu_time(raw: &str) -> Result<f64, String> {
        let (days, clock) = match raw.split_once('-') {
            Some((days, clock)) => (
                days.parse::<u64>()
                    .map_err(|error| format!("ps TIME days {days:?}: {error}"))?,
                clock,
            ),
            None => (0, raw),
        };
        let fields: Vec<&str> = clock.split(':').collect();
        let (hours, minutes, seconds) = match fields.as_slice() {
            [minutes, seconds] => (0, *minutes, *seconds),
            [hours, minutes, seconds] => (
                hours
                    .parse::<u64>()
                    .map_err(|error| format!("ps TIME hours {hours:?}: {error}"))?,
                *minutes,
                *seconds,
            ),
            _ => return Err(format!("ps TIME had unexpected shape {raw:?}")),
        };
        let minutes = minutes
            .parse::<u64>()
            .map_err(|error| format!("ps TIME minutes {minutes:?}: {error}"))?;
        let seconds = seconds
            .parse::<f64>()
            .map_err(|error| format!("ps TIME seconds {seconds:?}: {error}"))?;
        let total =
            days as f64 * 86_400.0 + hours as f64 * 3_600.0 + minutes as f64 * 60.0 + seconds;
        if total.is_finite() && total >= 0.0 {
            Ok(total)
        } else {
            Err(format!("ps TIME was not finite and nonnegative: {raw:?}"))
        }
    }

    fn process_sample() -> Result<ProcessSample, String> {
        let process_id = std::process::id().to_string();
        let output = std::process::Command::new("/bin/ps")
            .args([
                "-o",
                "rss=",
                "-o",
                "vsz=",
                "-o",
                "%cpu=",
                "-o",
                "time=",
                "-p",
                &process_id,
            ])
            .output()
            .map_err(|error| format!("ps process sample: {error}"))?;
        if !output.status.success() {
            return Err(format!("ps process sample exited {}", output.status));
        }
        let text = String::from_utf8(output.stdout)
            .map_err(|error| format!("ps returned non-UTF-8 output: {error}"))?;
        let mut fields = text.split_whitespace();
        let rss_kib = fields
            .next()
            .ok_or_else(|| "ps omitted RSS".to_owned())?
            .parse::<u64>()
            .map_err(|error| format!("ps RSS parse: {error}"))?;
        let virtual_kib = fields
            .next()
            .ok_or_else(|| "ps omitted VSZ".to_owned())?
            .parse::<u64>()
            .map_err(|error| format!("ps VSZ parse: {error}"))?;
        let cpu_percent = fields
            .next()
            .ok_or_else(|| "ps omitted CPU percentage".to_owned())?
            .parse::<f64>()
            .map_err(|error| format!("ps CPU parse: {error}"))?;
        let cpu_time_seconds = parse_cpu_time(
            fields
                .next()
                .ok_or_else(|| "ps omitted process CPU time".to_owned())?,
        )?;
        if fields.next().is_some() {
            return Err(format!("ps returned unexpected fields: {text:?}"));
        }
        if !cpu_percent.is_finite() || cpu_percent < 0.0 {
            return Err(format!("ps returned invalid CPU percentage {cpu_percent}"));
        }
        let resident_set_bytes = rss_kib
            .checked_mul(1024)
            .and_then(NonZeroU64::new)
            .ok_or_else(|| format!("ps returned unusable RSS {rss_kib} KiB"))?;
        let virtual_memory_bytes = virtual_kib
            .checked_mul(1024)
            .and_then(NonZeroU64::new)
            .ok_or_else(|| format!("ps returned unusable VSZ {virtual_kib} KiB"))?;
        Ok(ProcessSample {
            resident_set_bytes,
            virtual_memory_bytes,
            cpu_percent,
            cpu_time_seconds,
        })
    }

    let process = process_sample();
    let process_field = |select: fn(&ProcessSample) -> MeasurementField| match &process {
        Ok(sample) => select(sample),
        Err(reason) => MeasurementField::Unavailable(reason.clone()),
    };

    enum MeasurementField {
        U64(NonZeroU64),
        Float(f64),
        Unavailable(String),
    }

    let measured_u64 = |field: MeasurementField| match field {
        MeasurementField::U64(value) => Measurement::available(value),
        MeasurementField::Unavailable(reason) => Measurement::unavailable(reason),
        _ => Measurement::unavailable("internal resource field type mismatch"),
    };
    let measured_float = |field: MeasurementField| match field {
        MeasurementField::Float(value) => Measurement::available(value),
        MeasurementField::Unavailable(reason) => Measurement::unavailable(reason),
        _ => Measurement::unavailable("internal resource field type mismatch"),
    };

    let resident_set_bytes = measured_u64(process_field(|sample| {
        MeasurementField::U64(sample.resident_set_bytes)
    }));
    HostResourceSample {
        sampled_unix_millis: unix_millis_now(),
        architecture: std::env::consts::ARCH.to_owned(),
        operating_system: std::env::consts::OS.to_owned(),
        operating_system_version: measured_nonempty(
            sysctl("kern.osproductversion"),
            "kern.osproductversion",
        ),
        cpu_model: measured_nonempty(
            sysctl("machdep.cpu.brand_string"),
            "machdep.cpu.brand_string",
        ),
        physical_cores: measured_nonzero_usize(sysctl("hw.physicalcpu"), "hw.physicalcpu"),
        logical_cores: measured_nonzero_usize(sysctl("hw.logicalcpu"), "hw.logicalcpu"),
        performance_cores: measured_nonzero_usize(
            sysctl("hw.perflevel0.logicalcpu"),
            "hw.perflevel0.logicalcpu",
        ),
        efficiency_cores: measured_nonzero_usize(
            sysctl("hw.perflevel1.logicalcpu"),
            "hw.perflevel1.logicalcpu",
        ),
        total_memory_bytes: measured_nonzero_u64(sysctl("hw.memsize"), "hw.memsize"),
        resident_set_bytes: resident_set_bytes.clone(),
        max_sampled_resident_set_bytes: resident_set_bytes,
        virtual_memory_bytes: measured_u64(process_field(|sample| {
            MeasurementField::U64(sample.virtual_memory_bytes)
        })),
        cpu_percent: measured_float(process_field(|sample| {
            MeasurementField::Float(sample.cpu_percent)
        })),
        process_cpu_time_seconds: measured_float(process_field(|sample| {
            MeasurementField::Float(sample.cpu_time_seconds)
        })),
        thread_count: Measurement::unavailable(
            "macOS /bin/ps does not expose a supported process thread-count keyword (thcount is rejected)",
        ),
        peak_resident_set_bytes: Measurement::unavailable(
            "safe macOS ps/sysctl sampling does not expose per-process peak RSS",
        ),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeartbeatEvent {
    pub schema: String,
    pub sequence: u64,
    pub run_id: String,
    pub event_kind: EventKind,
    pub timestamp_unix_millis: u64,
    pub phase: String,
    pub phase_elapsed_millis: u64,
    pub elapsed_millis: u64,
    pub status: RunStatus,
    pub metadata: Option<RunMetadata>,
    pub work: WorkSnapshot,
    pub queue: QueueSnapshot,
    pub streams: Vec<StreamProgress>,
    pub rates: RateSnapshot,
    pub eta: EtaSnapshot,
    pub resources: ResourceAvailability,
}

impl HeartbeatEvent {
    pub fn new(
        run_id: impl Into<String>,
        status: RunStatus,
        work: WorkSnapshot,
        eta: EtaSnapshot,
        resources: ResourceAvailability,
    ) -> Self {
        Self {
            schema: EVENT_SCHEMA.to_owned(),
            sequence: 0,
            run_id: run_id.into(),
            event_kind: EventKind::Heartbeat,
            timestamp_unix_millis: unix_millis_now(),
            phase: "not-set".to_owned(),
            phase_elapsed_millis: 0,
            elapsed_millis: 0,
            status,
            metadata: None,
            work,
            queue: QueueSnapshot::default(),
            streams: Vec::new(),
            rates: RateSnapshot::default(),
            eta,
            resources,
        }
    }

    pub fn with_event_kind(mut self, event_kind: EventKind) -> Self {
        self.event_kind = event_kind;
        self
    }

    pub fn with_phase(mut self, phase: impl Into<String>) -> Self {
        self.phase = phase.into();
        self
    }

    pub fn with_elapsed(mut self, elapsed: Duration) -> Self {
        self.elapsed_millis = duration_millis(elapsed);
        self
    }

    pub fn with_phase_elapsed(mut self, elapsed: Duration) -> Self {
        self.phase_elapsed_millis = duration_millis(elapsed);
        self
    }

    pub fn with_metadata(mut self, metadata: RunMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_queue(mut self, queue: QueueSnapshot) -> Self {
        self.queue = queue;
        self
    }

    pub fn with_streams(mut self, streams: Vec<StreamProgress>) -> Self {
        self.streams = streams;
        self
    }

    pub fn with_rates(mut self, rates: RateSnapshot) -> Self {
        self.rates = rates;
        self
    }

    pub fn human_summary(&self) -> String {
        let rate = self
            .rates
            .rolling_forwards_per_second
            .or(self.rates.cumulative_forwards_per_second)
            .map(|value| format!("{value:.4}"))
            .unwrap_or_else(|| "UNAVAILABLE".to_owned());
        let worker_rate = self
            .rates
            .rolling_worker_tasks_per_second
            .map(|value| format!("{value:.4}"))
            .unwrap_or_else(|| "UNAVAILABLE".to_owned());
        let scalar_rate = self
            .rates
            .rolling_scalar_terms_per_second
            .or(self.rates.cumulative_scalar_terms_per_second)
            .map(|value| format!("{value:.4}"))
            .unwrap_or_else(|| "UNAVAILABLE".to_owned());
        let eta = match (self.eta.status, self.eta.remaining_seconds) {
            (EtaStatus::Estimated, Some(seconds)) => format!("{seconds}s"),
            (status, _) => format!("{status:?}").to_ascii_uppercase(),
        };
        let (cpu, rss) = resource_summary(&self.resources);
        format!(
            "teacher-parity event={:?} phase={} status={} forwards={}/{} tokens={}/{} batches={}/{} matrix_calls={}/{} batched_matrix_calls={}/{} max_matrix_batch_width={}/{} worker_tasks={}/{} row_tiles={}/{} output_cells={}/{} scalar_terms={}/{} exact_live_worker_tasks={}/{} queue={} active_streams={} peak_streams={} active_row_workers={} peak_row_workers={} failed_streams={} elapsed={}ms rate_fwd_s={} rate_worker_task_s={} rate_scalar_term_s={} eta_progress={:?}:{}/{} eta={} cpu={} rss={}",
            self.event_kind,
            self.phase,
            self.status.as_str(),
            self.work.logical_forwards,
            self.work.plan.logical_forwards,
            self.work.tokens,
            self.work.plan.tokens,
            self.work.physical_batches,
            self.work.plan.physical_batches,
            self.work.matrix_calls,
            self.work.plan.matrix_calls,
            self.work.batched_matrix_calls,
            self.work.plan.batched_matrix_calls,
            self.work.max_matrix_batch_width,
            self.work.plan.max_matrix_batch_width,
            self.work.worker_tasks,
            self.work.plan.worker_tasks,
            self.work.row_tiles,
            self.work.plan.row_tiles,
            self.work.output_cells,
            self.work.plan.output_cells,
            self.work.scalar_terms,
            self.work.plan.scalar_terms,
            self.queue.completed_worker_tasks,
            self.work.plan.worker_tasks,
            self.queue.queue_depth,
            self.queue.active_streams,
            self.queue.peak_active_streams,
            self.queue.active_row_workers,
            self.queue.peak_active_row_workers,
            self.queue.failed_streams,
            self.elapsed_millis,
            rate,
            worker_rate,
            scalar_rate,
            self.rates.eta_progress_unit,
            self.rates.eta_progress_completed,
            self.rates.eta_progress_total,
            eta,
            cpu,
            rss,
        )
    }
}

/// Require repeated periodic evidence for one still-active unit of work.
///
/// Lifecycle rows, idle/loading rows, and heartbeats from different phases do
/// not establish that the independent writer remained alive while one exact
/// forward was in flight. The two accepted rows must therefore both be
/// heartbeats, retain the same full-width ordered private-stream state, expose
/// bounded nonzero exact-row activity, remain in the same phase with the same
/// completed forward counter, and arrive within two configured cadence
/// intervals. Exact worker saturation is diagnostic, not a binding verdict.
pub fn validate_in_flight_heartbeat_cadence(
    events: &[HeartbeatEvent],
    cadence: Duration,
    expected_streams: usize,
    expected_row_workers: usize,
) -> Result<(), String> {
    if cadence.is_zero() {
        return Err("heartbeat cadence is zero".to_owned());
    }
    if expected_streams == 0 || expected_row_workers == 0 {
        return Err("heartbeat occupancy bounds must be nonzero".to_owned());
    }
    let maximum_delta_millis = duration_millis(cadence).saturating_mul(2);
    let heartbeats = events
        .iter()
        .filter(|event| event.event_kind == EventKind::Heartbeat)
        .collect::<Vec<_>>();
    if heartbeats.len() < 2 {
        return Err(format!(
            "expected at least two periodic HEARTBEAT rows, found {}",
            heartbeats.len()
        ));
    }
    let witnessed = heartbeats.windows(2).any(|pair| {
        let stable_streams = pair[0].streams.len() == expected_streams
            && pair[1].streams.len() == expected_streams
            && pair[0].streams.iter().zip(&pair[1].streams).all(|(a, b)| {
                a.stream_id == b.stream_id
                    && a.phase == b.phase
                    && a.state == StreamState::Active
                    && b.state == StreamState::Active
                    && a.logical_forwards_completed == b.logical_forwards_completed
                    && a.logical_forwards_total == b.logical_forwards_total
                    && a.tokens_completed == b.tokens_completed
                    && a.tokens_total == b.tokens_total
            });
        let distinct_streams = pair[0]
            .streams
            .iter()
            .map(|stream| stream.stream_id.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == expected_streams;
        let delta = pair[1]
            .timestamp_unix_millis
            .saturating_sub(pair[0].timestamp_unix_millis);
        pair[0].queue.active_streams == expected_streams as u64
            && pair[1].queue.active_streams == expected_streams as u64
            && pair[0].queue.active_row_workers > 0
            && pair[1].queue.active_row_workers > 0
            && pair[0].queue.active_row_workers <= expected_row_workers as u64
            && pair[1].queue.active_row_workers <= expected_row_workers as u64
            && stable_streams
            && distinct_streams
            && pair[0].phase == pair[1].phase
            && pair[0].work.logical_forwards == pair[1].work.logical_forwards
            && delta > 0
            && delta <= maximum_delta_millis
    });
    if witnessed {
        Ok(())
    } else {
        Err(format!(
            "no same-phase HEARTBEAT pair retained {expected_streams} stable private streams, bounded nonzero exact-row activity (limit {expected_row_workers}), and an unchanged forward counter within {maximum_delta_millis} ms"
        ))
    }
}

/// Require one durable periodic sample that witnesses the entire private
/// stream cohort and bounded nonzero exact-row activity simultaneously.
/// Distinct stream rows bind the occupancy counters to independently named
/// trajectories instead of permitting one trajectory to be fanned out. The
/// exact worker count remains a diagnostic bounded by the selected executor.
pub fn validate_full_width_exact_heartbeat(
    events: &[HeartbeatEvent],
    expected_streams: usize,
    expected_row_workers: usize,
) -> Result<(), String> {
    if expected_streams <= 1 || expected_row_workers <= 1 {
        return Err(format!(
            "binding heartbeat requires multiple streams and row workers, got streams={expected_streams}, row_workers={expected_row_workers}"
        ));
    }
    let witnessed = events.iter().any(|event| {
        if event.event_kind != EventKind::Heartbeat
            || event.queue.active_streams != expected_streams as u64
            || event.queue.active_row_workers == 0
            || event.queue.active_row_workers > expected_row_workers as u64
            || event.streams.len() != expected_streams
        {
            return false;
        }
        let identities = event
            .streams
            .iter()
            .map(|stream| stream.stream_id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        identities.len() == expected_streams
    });
    if witnessed {
        Ok(())
    } else {
        Err(format!(
            "no HEARTBEAT row simultaneously witnessed {expected_streams} distinct private streams and bounded nonzero exact-row activity (limit {expected_row_workers})"
        ))
    }
}

fn resource_summary(resources: &ResourceAvailability) -> (String, String) {
    match resources {
        ResourceAvailability::Available(sample) => {
            let cpu = match &sample.cpu_percent {
                Measurement::Available { value } => format!("{value:.1}%"),
                Measurement::Unavailable { .. } => "UNAVAILABLE".to_owned(),
            };
            let rss = match &sample.resident_set_bytes {
                Measurement::Available { value } => value.get().to_string(),
                Measurement::Unavailable { .. } => "UNAVAILABLE".to_owned(),
            };
            (cpu, rss)
        }
        ResourceAvailability::Unavailable { .. } => {
            ("UNAVAILABLE".to_owned(), "UNAVAILABLE".to_owned())
        }
    }
}

pub struct HeartbeatLog {
    file: File,
    next_sequence: u64,
}

impl HeartbeatLog {
    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        Ok(Self {
            file,
            next_sequence: 0,
        })
    }

    /// Append one JSON object, then flush userspace buffers and request a data
    /// sync before returning. A reader can therefore inspect every completed
    /// heartbeat while the run is still active.
    pub fn append(&mut self, mut event: HeartbeatEvent) -> Result<(), ReportError> {
        event.sequence = self.next_sequence;
        let mut bytes = serde_json::to_vec(&event)?;
        bytes.push(b'\n');
        self.file.write_all(&bytes)?;
        self.file.flush()?;
        self.file.sync_data()?;
        self.next_sequence = self.next_sequence.saturating_add(1);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LiveProgress {
    pub run_id: String,
    pub status: RunStatus,
    pub phase: String,
    pub metadata: RunMetadata,
    pub queue: QueueSnapshot,
    pub streams: Vec<StreamProgress>,
}

struct TrackedProgress {
    live: LiveProgress,
    phase_started: Instant,
    updated_at: Instant,
}

/// One monotonic callback observation from the exact row executor. Grouping
/// these fields prevents positional call sites from swapping stream and row
/// worker counters as the owner snapshot evolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExactProgressObservation {
    pub observer_epoch: u64,
    pub streams_started: u64,
    pub streams_completed: u64,
    pub active_streams: usize,
    pub peak_active_streams: usize,
    pub active_row_workers: usize,
    pub peak_active_row_workers: usize,
    pub matrix_calls: u64,
    pub batched_matrix_calls: u64,
    pub max_matrix_batch_width: usize,
    pub completed_worker_tasks: u64,
    pub output_cells_completed: u64,
    pub scalar_terms_completed: u64,
    pub effective_workers: usize,
}

/// Scheduler-independent logical exact-work counters retained in deterministic
/// S2 evidence. Observer epochs, worker/tile geometry, current/peak occupancy,
/// workspace behavior, and the count of forwards that happened to witness
/// concurrent workers are empirical scheduling observations and deliberately
/// do not cross this boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeterministicTeacherExecution {
    pub forward_calls: u64,
    pub streams_started: u64,
    pub streams_completed: u64,
    pub matrix_calls: u64,
    pub batched_matrix_calls: u64,
    pub max_matrix_batch_width: usize,
    pub output_cells_completed: u64,
    pub scalar_terms_completed: u64,
}

pub fn deterministic_teacher_execution(
    snapshot: TeacherExecutionSnapshot,
) -> DeterministicTeacherExecution {
    DeterministicTeacherExecution {
        forward_calls: snapshot.forward_calls,
        streams_started: snapshot.streams_started,
        streams_completed: snapshot.streams_completed,
        matrix_calls: snapshot.matrix_calls,
        batched_matrix_calls: snapshot.batched_matrix_calls,
        max_matrix_batch_width: snapshot.max_matrix_batch_width,
        output_cells_completed: snapshot.output_cells_completed,
        scalar_terms_completed: snapshot.scalar_terms_completed,
    }
}

#[derive(Clone)]
pub struct SharedProgress {
    inner: Arc<Mutex<TrackedProgress>>,
    exact: Arc<ExactLiveProgress>,
}

#[derive(Default)]
struct ExactLiveProgress {
    streams_started: AtomicU64,
    streams_completed: AtomicU64,
    current: Mutex<ExactCurrentProgress>,
    peak_active_streams: AtomicU64,
    peak_active_row_workers: AtomicU64,
    matrix_calls: AtomicU64,
    batched_matrix_calls: AtomicU64,
    max_matrix_batch_width: AtomicU64,
    completed_worker_tasks: AtomicU64,
    output_cells_completed: AtomicU64,
    scalar_terms_completed: AtomicU64,
    effective_workers: AtomicU64,
}

#[derive(Default)]
struct ExactCurrentProgress {
    observer_epoch: Option<u64>,
    active_streams: u64,
    active_row_workers: u64,
}

#[derive(Debug, Clone)]
pub struct LiveProgressSnapshot {
    pub live: LiveProgress,
    pub phase_elapsed: Duration,
    pub state_age: Duration,
}

impl SharedProgress {
    pub fn new(run_id: impl Into<String>, metadata: RunMetadata) -> Self {
        let now = Instant::now();
        Self {
            inner: Arc::new(Mutex::new(TrackedProgress {
                live: LiveProgress {
                    run_id: run_id.into(),
                    status: RunStatus::NotRun,
                    phase: "not-set".to_owned(),
                    metadata,
                    queue: QueueSnapshot::default(),
                    streams: Vec::new(),
                },
                phase_started: now,
                updated_at: now,
            })),
            exact: Arc::new(ExactLiveProgress::default()),
        }
    }

    /// Publish exact absolute counters without taking the rich progress lock.
    /// A tiny independent lock makes current occupancy epoch-safe: callbacks
    /// may arrive out of order, but an older callback cannot regress live
    /// state or resurrect a completed forward.
    pub fn publish_exact(&self, observation: ExactProgressObservation) {
        let ExactProgressObservation {
            observer_epoch,
            streams_started,
            streams_completed,
            active_streams,
            peak_active_streams,
            active_row_workers,
            peak_active_row_workers,
            matrix_calls,
            batched_matrix_calls,
            max_matrix_batch_width,
            completed_worker_tasks,
            output_cells_completed,
            scalar_terms_completed,
            effective_workers,
        } = observation;
        let accepted = {
            let mut current = self
                .exact
                .current
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if current
                .observer_epoch
                .is_some_and(|prior| observer_epoch <= prior)
            {
                false
            } else {
                current.observer_epoch = Some(observer_epoch);
                current.active_streams = active_streams as u64;
                current.active_row_workers = active_row_workers as u64;
                true
            }
        };
        if !accepted {
            return;
        }
        self.exact
            .streams_started
            .fetch_max(streams_started, Ordering::AcqRel);
        self.exact
            .streams_completed
            .fetch_max(streams_completed, Ordering::AcqRel);
        self.exact
            .peak_active_streams
            .fetch_max(peak_active_streams as u64, Ordering::AcqRel);
        self.exact
            .peak_active_row_workers
            .fetch_max(peak_active_row_workers as u64, Ordering::AcqRel);
        self.exact
            .matrix_calls
            .fetch_max(matrix_calls, Ordering::AcqRel);
        self.exact
            .batched_matrix_calls
            .fetch_max(batched_matrix_calls, Ordering::AcqRel);
        self.exact.max_matrix_batch_width.fetch_max(
            u64::try_from(max_matrix_batch_width).unwrap_or(u64::MAX),
            Ordering::AcqRel,
        );
        self.exact
            .completed_worker_tasks
            .fetch_max(completed_worker_tasks, Ordering::AcqRel);
        self.exact
            .output_cells_completed
            .fetch_max(output_cells_completed, Ordering::AcqRel);
        self.exact
            .scalar_terms_completed
            .fetch_max(scalar_terms_completed, Ordering::AcqRel);
        self.exact
            .effective_workers
            .fetch_max(effective_workers as u64, Ordering::AcqRel);
    }

    /// A synchronous forward boundary: all worker callbacks have returned, so
    /// current in-flight occupancy may be cleared without a stale callback
    /// resurrecting it.
    pub fn finish_exact_forward(&self) {
        let mut current = self
            .exact
            .current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        current.active_streams = 0;
        current.active_row_workers = 0;
    }

    pub fn update(&self, update: impl FnOnce(&mut LiveProgress)) -> Result<(), ProgressError> {
        let mut tracked = self.inner.lock().map_err(|_| ProgressError::Poisoned)?;
        let prior_phase = tracked.live.phase.clone();
        update(&mut tracked.live);
        tracked.live.queue.peak_active_streams = tracked
            .live
            .queue
            .peak_active_streams
            .max(tracked.live.queue.active_streams);
        tracked.live.queue.peak_active_row_workers = tracked
            .live
            .queue
            .peak_active_row_workers
            .max(tracked.live.queue.active_row_workers);
        let now = Instant::now();
        if tracked.live.phase != prior_phase {
            tracked.phase_started = now;
        }
        tracked.updated_at = now;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<LiveProgressSnapshot, ProgressError> {
        let tracked = self.inner.lock().map_err(|_| ProgressError::Poisoned)?;
        let now = Instant::now();
        let mut live = tracked.live.clone();
        let current = self
            .exact
            .current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let exact_active_streams = current.active_streams;
        let newly_observed_streams = exact_active_streams.saturating_sub(live.queue.active_streams);
        live.queue.queue_depth = live
            .queue
            .queue_depth
            .saturating_sub(newly_observed_streams);
        live.queue.active_streams = live.queue.active_streams.max(exact_active_streams);
        live.queue.active_row_workers = live
            .queue
            .active_row_workers
            .max(current.active_row_workers);
        if exact_active_streams == 0 {
            live.queue.active_row_workers = 0;
            live.queue.active_worker_tasks = 0;
        } else {
            live.queue.active_worker_tasks = live
                .queue
                .active_worker_tasks
                .max(live.queue.active_row_workers);
        }
        live.queue.peak_active_row_workers = live
            .queue
            .peak_active_row_workers
            .max(self.exact.peak_active_row_workers.load(Ordering::Acquire));
        live.queue.peak_active_streams = live
            .queue
            .peak_active_streams
            .max(self.exact.peak_active_streams.load(Ordering::Acquire));
        live.queue.completed_worker_tasks = live
            .queue
            .completed_worker_tasks
            .max(self.exact.completed_worker_tasks.load(Ordering::Acquire));
        if let Some(effective_workers) = NonZeroUsize::new(
            usize::try_from(self.exact.effective_workers.load(Ordering::Acquire))
                .unwrap_or(usize::MAX),
        ) {
            live.metadata.scheduler.effective_row_workers = effective_workers;
        }
        Ok(LiveProgressSnapshot {
            live,
            phase_elapsed: now.saturating_duration_since(tracked.phase_started),
            state_age: now.saturating_duration_since(tracked.updated_at),
        })
    }

    /// Merge monotonic executor callbacks into the durable work view while a
    /// physical forward is still running. Forward-boundary accounting later
    /// records the same absolute totals, so `max` avoids double counting.
    fn overlay_exact_work(&self, work: &mut WorkSnapshot) {
        work.matrix_calls = work
            .matrix_calls
            .max(self.exact.matrix_calls.load(Ordering::Acquire));
        work.batched_matrix_calls = work
            .batched_matrix_calls
            .max(self.exact.batched_matrix_calls.load(Ordering::Acquire));
        work.max_matrix_batch_width = work
            .max_matrix_batch_width
            .max(self.exact.max_matrix_batch_width.load(Ordering::Acquire));
        let completed_worker_tasks = self.exact.completed_worker_tasks.load(Ordering::Acquire);
        work.worker_tasks = work.worker_tasks.max(completed_worker_tasks);
        work.row_tiles = work.row_tiles.max(completed_worker_tasks);
        work.output_cells = work
            .output_cells
            .max(self.exact.output_cells_completed.load(Ordering::Acquire));
        work.scalar_terms = work
            .scalar_terms
            .max(self.exact.scalar_terms_completed.load(Ordering::Acquire));
    }
}

/// Downgrade the in-memory terminal view when terminal emission or canonical
/// report commitment fails. A retry/readback must never return a stale PASS.
pub fn mark_finalization_failed(
    progress: &SharedProgress,
    phase: impl Into<String>,
) -> Result<(), ProgressError> {
    let phase = phase.into();
    progress.update(|live| {
        live.phase = phase;
        live.status = RunStatus::Fail;
        live.queue.active_streams = 0;
        live.queue.active_row_workers = 0;
        live.queue.active_worker_tasks = 0;
        live.streams.clear();
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressError {
    Poisoned,
}

impl fmt::Display for ProgressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poisoned => write!(formatter, "teacher-parity progress state is poisoned"),
        }
    }
}

impl std::error::Error for ProgressError {}

enum HeartbeatCommand {
    Emit {
        kind: EventKind,
        acknowledgement: mpsc::SyncSender<Result<(), String>>,
    },
    EmitAndStop {
        kind: EventKind,
        acknowledgement: mpsc::SyncSender<Result<(), String>>,
    },
    Stop,
}

pub struct HeartbeatWorker {
    commands: mpsc::Sender<HeartbeatCommand>,
    join: Option<JoinHandle<Result<(), HeartbeatWorkerError>>>,
    max_sampled_rss_bytes: Arc<AtomicU64>,
}

impl HeartbeatWorker {
    pub fn spawn(
        path: impl AsRef<Path>,
        cadence: Duration,
        counters: Arc<WorkCounters>,
        progress: SharedProgress,
    ) -> Result<Self, HeartbeatWorkerError> {
        let stall_after = cadence
            .checked_mul(4)
            .unwrap_or(Duration::MAX)
            .max(Duration::from_secs(MINIMUM_DEFAULT_STALL_SECONDS));
        Self::spawn_with_stall_after(path, cadence, stall_after, counters, progress)
    }

    pub fn spawn_with_stall_after(
        path: impl AsRef<Path>,
        cadence: Duration,
        stall_after: Duration,
        counters: Arc<WorkCounters>,
        progress: SharedProgress,
    ) -> Result<Self, HeartbeatWorkerError> {
        if cadence.is_zero() || stall_after.is_zero() {
            return Err(HeartbeatWorkerError::InvalidCadence);
        }
        let log = HeartbeatLog::create(path).map_err(ReportError::from)?;
        let (commands, receiver) = mpsc::channel();
        let max_sampled_rss_bytes = Arc::new(AtomicU64::new(0));
        let worker_max_sampled_rss = Arc::clone(&max_sampled_rss_bytes);
        let join = thread::Builder::new()
            .name("parity-heartbeat".to_owned())
            .spawn(move || {
                heartbeat_loop(
                    log,
                    cadence,
                    stall_after,
                    counters,
                    progress,
                    receiver,
                    worker_max_sampled_rss,
                )
            })
            .map_err(ReportError::from)?;
        Ok(Self {
            commands,
            join: Some(join),
            max_sampled_rss_bytes,
        })
    }

    /// Emit and durably flush a non-periodic lifecycle event before returning.
    pub fn emit(&self, kind: EventKind) -> Result<(), HeartbeatWorkerError> {
        let (acknowledgement, received) = mpsc::sync_channel(0);
        self.commands
            .send(HeartbeatCommand::Emit {
                kind,
                acknowledgement,
            })
            .map_err(|_| HeartbeatWorkerError::WorkerStopped)?;
        match received.recv() {
            Ok(Ok(())) => Ok(()),
            Ok(Err(reason)) => Err(HeartbeatWorkerError::Writer(reason)),
            Err(_) => Err(HeartbeatWorkerError::WorkerStopped),
        }
    }

    pub fn stop(mut self) -> Result<(), HeartbeatWorkerError> {
        self.stop_inner()
    }

    /// Durably append the terminal event and stop the writer as one ordered
    /// worker command. No periodic heartbeat can be appended after `kind`.
    pub fn emit_and_stop(mut self, kind: EventKind) -> Result<(), HeartbeatWorkerError> {
        let (acknowledgement, received) = mpsc::sync_channel(0);
        let sent = self
            .commands
            .send(HeartbeatCommand::EmitAndStop {
                kind,
                acknowledgement,
            })
            .map_err(|_| HeartbeatWorkerError::WorkerStopped);
        let emitted = match sent {
            Ok(()) => match received.recv() {
                Ok(Ok(())) => Ok(()),
                Ok(Err(reason)) => Err(HeartbeatWorkerError::Writer(reason)),
                Err(_) => Err(HeartbeatWorkerError::WorkerStopped),
            },
            Err(error) => Err(error),
        };
        let joined = match self.join.take() {
            Some(join) => join.join().map_err(|_| HeartbeatWorkerError::Panicked)?,
            None => Ok(()),
        };
        emitted?;
        joined
    }

    pub fn max_sampled_rss_bytes(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.max_sampled_rss_bytes.load(Ordering::Acquire))
    }

    fn stop_inner(&mut self) -> Result<(), HeartbeatWorkerError> {
        let _ = self.commands.send(HeartbeatCommand::Stop);
        let Some(join) = self.join.take() else {
            return Ok(());
        };
        join.join().map_err(|_| HeartbeatWorkerError::Panicked)?
    }
}

impl Drop for HeartbeatWorker {
    fn drop(&mut self) {
        let _ = self.stop_inner();
    }
}

struct HeartbeatTiming {
    started: Instant,
    last_progress: Instant,
    prior_sample: Instant,
    prior_forwards: u64,
    prior_tokens: u64,
    prior_worker_tasks: u64,
    prior_scalar_terms: u64,
    prior_progress_unit: ProgressUnit,
    prior_progress_completed: u64,
    stall_after: Duration,
}

fn heartbeat_loop(
    mut log: HeartbeatLog,
    cadence: Duration,
    stall_after: Duration,
    counters: Arc<WorkCounters>,
    progress: SharedProgress,
    receiver: mpsc::Receiver<HeartbeatCommand>,
    max_sampled_rss_bytes: Arc<AtomicU64>,
) -> Result<(), HeartbeatWorkerError> {
    let now = Instant::now();
    let mut timing = HeartbeatTiming {
        started: now,
        last_progress: now,
        prior_sample: now,
        prior_forwards: 0,
        prior_tokens: 0,
        prior_worker_tasks: 0,
        prior_scalar_terms: 0,
        prior_progress_unit: ProgressUnit::LogicalForwards,
        prior_progress_completed: 0,
        stall_after,
    };
    emit_live_event(
        &mut log,
        EventKind::Heartbeat,
        &counters,
        &progress,
        &mut timing,
        &max_sampled_rss_bytes,
    )?;
    loop {
        match receiver.recv_timeout(cadence) {
            Ok(HeartbeatCommand::Emit {
                kind,
                acknowledgement,
            }) => {
                let result = emit_live_event(
                    &mut log,
                    kind,
                    &counters,
                    &progress,
                    &mut timing,
                    &max_sampled_rss_bytes,
                );
                let acknowledgement_result =
                    result.as_ref().map(|_| ()).map_err(ToString::to_string);
                let _ = acknowledgement.send(acknowledgement_result);
                result?;
            }
            Ok(HeartbeatCommand::EmitAndStop {
                kind,
                acknowledgement,
            }) => {
                let result = emit_live_event(
                    &mut log,
                    kind,
                    &counters,
                    &progress,
                    &mut timing,
                    &max_sampled_rss_bytes,
                );
                let acknowledgement_result =
                    result.as_ref().map(|_| ()).map_err(ToString::to_string);
                let _ = acknowledgement.send(acknowledgement_result);
                result?;
                return Ok(());
            }
            Ok(HeartbeatCommand::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
            Err(mpsc::RecvTimeoutError::Timeout) => emit_live_event(
                &mut log,
                EventKind::Heartbeat,
                &counters,
                &progress,
                &mut timing,
                &max_sampled_rss_bytes,
            )?,
        }
    }
}

fn emit_live_event(
    log: &mut HeartbeatLog,
    kind: EventKind,
    counters: &WorkCounters,
    progress: &SharedProgress,
    timing: &mut HeartbeatTiming,
    max_sampled_rss_bytes: &AtomicU64,
) -> Result<(), HeartbeatWorkerError> {
    let now = Instant::now();
    let mut work = counters.snapshot();
    progress.overlay_exact_work(&mut work);
    let (progress_unit, progress_completed, progress_total) = heartbeat_progress_units(&work);
    if progress_unit != timing.prior_progress_unit
        || progress_completed != timing.prior_progress_completed
    {
        timing.last_progress = now;
    }
    let progress = progress.snapshot()?;
    let LiveProgressSnapshot {
        live,
        phase_elapsed,
        state_age,
    } = progress;
    let LiveProgress {
        run_id,
        status,
        phase,
        metadata,
        mut queue,
        mut streams,
    } = live;
    let elapsed = now.saturating_duration_since(timing.started);
    let tokens = work.tokens;
    let sample_elapsed = now
        .saturating_duration_since(timing.prior_sample)
        .as_secs_f64();
    let elapsed_seconds = elapsed.as_secs_f64();
    let forward_delta = work.logical_forwards.saturating_sub(timing.prior_forwards);
    let token_delta = tokens.saturating_sub(timing.prior_tokens);
    let worker_task_delta = work.worker_tasks.saturating_sub(timing.prior_worker_tasks);
    let scalar_term_delta = work.scalar_terms.saturating_sub(timing.prior_scalar_terms);
    let rolling_forwards_per_second = positive_rate(forward_delta, sample_elapsed);
    let cumulative_forwards_per_second = positive_rate(work.logical_forwards, elapsed_seconds);
    let rates = RateSnapshot {
        rolling_forwards_per_second,
        cumulative_forwards_per_second,
        seconds_per_forward: seconds_per_forward_from_rate(cumulative_forwards_per_second),
        tokens_per_second: positive_rate(token_delta, sample_elapsed)
            .or_else(|| positive_rate(tokens, elapsed_seconds)),
        rolling_worker_tasks_per_second: positive_rate(worker_task_delta, sample_elapsed),
        rolling_scalar_terms_per_second: positive_rate(scalar_term_delta, sample_elapsed),
        cumulative_scalar_terms_per_second: positive_rate(work.scalar_terms, elapsed_seconds),
        eta_progress_unit: progress_unit,
        eta_progress_completed: progress_completed,
        eta_progress_total: progress_total,
    };
    let eta = estimate_eta(EtaInput {
        completed: progress_completed,
        total: progress_total,
        elapsed,
        last_progress_age: now.saturating_duration_since(timing.last_progress),
        stall_after: timing.stall_after,
        minimum_samples: 3,
    });
    let state_age_millis = duration_millis(state_age);
    if queue.active_streams != 0 {
        queue.longest_active_millis = queue.longest_active_millis.max(state_age_millis);
    }
    for stream in &mut streams {
        if stream.state == StreamState::Active {
            stream.active_forward_age_millis =
                stream.active_forward_age_millis.max(state_age_millis);
        }
    }
    let mut resources = sample_host_resources();
    if let Some(rss) = resources.resident_set_bytes() {
        max_sampled_rss_bytes.fetch_max(rss.get(), Ordering::AcqRel);
    }
    resources.set_max_sampled_resident_set_bytes(NonZeroU64::new(
        max_sampled_rss_bytes.load(Ordering::Acquire),
    ));
    let event = HeartbeatEvent::new(run_id, status, work, eta, resources)
        .with_event_kind(kind)
        .with_phase(phase)
        .with_phase_elapsed(phase_elapsed)
        .with_elapsed(elapsed)
        .with_metadata(metadata)
        .with_queue(queue)
        .with_streams(streams)
        .with_rates(rates);
    eprintln!("{}", event.human_summary());
    log.append(event)?;
    timing.prior_sample = now;
    timing.prior_forwards = work.logical_forwards;
    timing.prior_tokens = tokens;
    timing.prior_worker_tasks = work.worker_tasks;
    timing.prior_scalar_terms = work.scalar_terms;
    timing.prior_progress_unit = progress_unit;
    timing.prior_progress_completed = progress_completed;
    Ok(())
}

fn positive_rate(count: u64, seconds: f64) -> Option<f64> {
    if count == 0 || !seconds.is_finite() || seconds <= 0.0 {
        return None;
    }
    let rate = count as f64 / seconds;
    rate.is_finite().then_some(rate)
}

/// Derive seconds per forward only when the measured forward rate has a
/// finite, strictly positive reciprocal. A completed zero-work refusal has a
/// valid `0.0` cumulative rate but no meaningful seconds-per-forward value;
/// emitting `+inf` would serialize as JSON `null` and fail semantic readback.
pub fn seconds_per_forward_from_rate(rate: Option<f64>) -> Option<f64> {
    let rate = rate.filter(|rate| rate.is_finite() && *rate > 0.0)?;
    let seconds = 1.0 / rate;
    (seconds.is_finite() && seconds > 0.0).then_some(seconds)
}

#[derive(Debug)]
pub enum HeartbeatWorkerError {
    InvalidCadence,
    Progress(ProgressError),
    Report(ReportError),
    WorkerStopped,
    Writer(String),
    Panicked,
}

impl fmt::Display for HeartbeatWorkerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCadence => write!(formatter, "heartbeat cadence must be nonzero"),
            Self::Progress(error) => write!(formatter, "heartbeat progress: {error}"),
            Self::Report(error) => write!(formatter, "heartbeat report: {error}"),
            Self::WorkerStopped => write!(formatter, "heartbeat worker stopped unexpectedly"),
            Self::Writer(reason) => write!(formatter, "heartbeat writer: {reason}"),
            Self::Panicked => write!(formatter, "heartbeat worker panicked"),
        }
    }
}

impl std::error::Error for HeartbeatWorkerError {}

impl From<ProgressError> for HeartbeatWorkerError {
    fn from(error: ProgressError) -> Self {
        Self::Progress(error)
    }
}

impl From<ReportError> for HeartbeatWorkerError {
    fn from(error: ReportError) -> Self {
        Self::Report(error)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeterministicEvidence {
    pub schema: String,
    pub status: RunStatus,
    pub identity: BTreeMap<String, String>,
    /// Caller-owned exact observations. Observability never transforms this.
    pub output: Value,
}

impl DeterministicEvidence {
    pub fn new(status: RunStatus, identity: BTreeMap<String, String>, output: Value) -> Self {
        Self {
            schema: EVIDENCE_SCHEMA.to_owned(),
            status,
            identity,
            output,
        }
    }
}

/// Project empirical run identities into the timing- and path-free companion.
/// The standalone preflight CID covers absolute operator paths, while the
/// selected executor configuration is timing-derived; both remain in the
/// RunReport. The deterministic sidecar replaces the former with the CID of
/// its already-scrubbed S0 projection and omits the latter.
pub fn deterministic_evidence_identities(
    metadata: &RunMetadata,
    output: &Value,
) -> Result<BTreeMap<String, String>, ReportError> {
    let mut identities = metadata.identities.clone();
    identities.remove("teacher_free_s4_preflight");
    identities.remove("probe_selected_execution");
    if let Some(preflight) = output.get("S0_teacher_free_preflight") {
        let bytes = serde_json::to_vec(preflight)?;
        identities.insert(
            "teacher_free_s4_preflight".to_owned(),
            format!("blake3:{}", blake3::hash(&bytes).to_hex()),
        );
    }
    Ok(identities)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunReport {
    pub schema: String,
    pub run_id: String,
    pub mode: ObservabilityMode,
    pub status: RunStatus,
    pub started_unix_millis: u64,
    pub elapsed_millis: u64,
    pub metadata: Option<RunMetadata>,
    pub work: WorkSnapshot,
    pub queue: QueueSnapshot,
    pub streams: Vec<StreamProgress>,
    pub rates: RateSnapshot,
    pub eta: EtaSnapshot,
    pub resources: ResourceAvailability,
    /// Empirical, run-variant measurements such as per-wave wall times and
    /// throughput. These bytes are deliberately excluded from deterministic
    /// evidence.
    pub measurements: BTreeMap<String, Value>,
    pub detail: Option<String>,
}

impl RunReport {
    pub fn new(
        run_id: impl Into<String>,
        mode: ObservabilityMode,
        status: RunStatus,
        work: WorkSnapshot,
        eta: EtaSnapshot,
        resources: ResourceAvailability,
    ) -> Self {
        let started_unix_millis = unix_millis_now();
        Self {
            schema: RUN_REPORT_SCHEMA.to_owned(),
            run_id: run_id.into(),
            mode,
            status,
            started_unix_millis,
            elapsed_millis: eta.elapsed_seconds.saturating_mul(1000),
            metadata: None,
            work,
            queue: QueueSnapshot::default(),
            streams: Vec::new(),
            rates: RateSnapshot::default(),
            eta,
            resources,
            measurements: BTreeMap::new(),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_metadata(mut self, metadata: RunMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_queue(mut self, queue: QueueSnapshot) -> Self {
        self.queue = queue;
        self
    }

    pub fn with_streams(mut self, streams: Vec<StreamProgress>) -> Self {
        self.streams = streams;
        self
    }

    pub fn with_rates(mut self, rates: RateSnapshot) -> Self {
        self.rates = rates;
        self
    }

    pub fn with_measurements(mut self, measurements: BTreeMap<String, Value>) -> Self {
        self.measurements = measurements;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportPaths {
    pub run: PathBuf,
    pub evidence: PathBuf,
}

/// Fully serialized, synced, and read-back-validated final companions. The
/// caller may prepare this value while the heartbeat writer is still alive,
/// then emit and stop the terminal event, and finally perform only the two
/// atomic renames in [`PreparedReports::commit`].
pub struct PreparedReports {
    paths: ReportPaths,
    run_temp: PathBuf,
    evidence_temp: PathBuf,
}

impl PreparedReports {
    pub fn commit(self) -> Result<ReportPaths, ReportError> {
        let publish = (|| -> Result<(), ReportError> {
            // Evidence is published first; the run report is the commit
            // marker and therefore always lands last.
            std::fs::rename(&self.evidence_temp, &self.paths.evidence)?;
            std::fs::rename(&self.run_temp, &self.paths.run)?;
            Ok(())
        })();
        if let Err(error) = publish {
            if let Err(invalidation) = invalidate_final_reports_checked(&self.paths.run) {
                return Err(ReportError::InvalidCompanion(format!(
                    "publish failed ({error}); canonical companion invalidation also failed ({invalidation})"
                )));
            }
            return Err(error);
        }
        Ok(self.paths.clone())
    }
}

impl Drop for PreparedReports {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.run_temp);
        let _ = std::fs::remove_file(&self.evidence_temp);
    }
}

/// Serialize, sync, and read back both final companions without publishing
/// either canonical path.
pub fn prepare_final_reports(
    report_path: impl AsRef<Path>,
    run: &RunReport,
    evidence: &DeterministicEvidence,
) -> Result<PreparedReports, ReportError> {
    let report_path = report_path.as_ref();
    // This run owns the configured canonical paths from this point onward.
    // Remove any prior pair before staging so a later terminal/preparation
    // failure cannot leave a stale PASS masquerading as the current run.
    invalidate_final_reports_checked(report_path)?;
    let result = (|| -> Result<PreparedReports, ReportError> {
        if run.schema != RUN_REPORT_SCHEMA {
            return Err(ReportError::InvalidCompanion(format!(
                "run schema {:?} does not match {RUN_REPORT_SCHEMA}",
                run.schema
            )));
        }
        if evidence.schema != EVIDENCE_SCHEMA {
            return Err(ReportError::InvalidCompanion(format!(
                "evidence schema {:?} does not match {EVIDENCE_SCHEMA}",
                evidence.schema
            )));
        }
        if run.status != evidence.status {
            return Err(ReportError::InvalidCompanion(format!(
                "run status {} does not match evidence status {}",
                run.status.as_str(),
                evidence.status.as_str()
            )));
        }
        let paths = ReportPaths {
            run: report_path.to_owned(),
            evidence: evidence_path_for_report(report_path)?,
        };
        if let Some(directory) = report_path.parent() {
            std::fs::create_dir_all(directory)?;
        }
        let mut run_bytes = serde_json::to_vec_pretty(run)?;
        run_bytes.push(b'\n');
        let mut evidence_bytes = serde_json::to_vec(evidence)?;
        evidence_bytes.push(b'\n');
        let prepared = PreparedReports {
            run_temp: temporary_sibling(&paths.run, "run")?,
            evidence_temp: temporary_sibling(&paths.evidence, "evidence")?,
            paths,
        };
        write_synced_new(&prepared.run_temp, &run_bytes)?;
        write_synced_new(&prepared.evidence_temp, &evidence_bytes)?;
        let decoded_run: RunReport = serde_json::from_slice(&std::fs::read(&prepared.run_temp)?)?;
        let decoded_evidence: DeterministicEvidence =
            serde_json::from_slice(&std::fs::read(&prepared.evidence_temp)?)?;
        if decoded_run != *run || decoded_evidence != *evidence {
            return Err(ReportError::InvalidCompanion(
                "temporary report readback changed serialized content".to_owned(),
            ));
        }
        Ok(prepared)
    })();
    match result {
        Ok(prepared) => Ok(prepared),
        Err(error) => match invalidate_final_reports_checked(report_path) {
            Ok(()) => Err(error),
            Err(invalidation) => Err(ReportError::InvalidCompanion(format!(
                "preparation failed ({error}); canonical companion invalidation also failed ({invalidation})"
            ))),
        },
    }
}

pub fn write_final_reports(
    report_path: impl AsRef<Path>,
    run: &RunReport,
    evidence: &DeterministicEvidence,
) -> Result<ReportPaths, ReportError> {
    prepare_final_reports(report_path, run, evidence)?.commit()
}

/// Remove both canonical companions after any failed preparation or commit.
/// A stale report or evidence sidecar must not masquerade as the current run.
/// Missing paths are already invalidated; every other removal failure is
/// surfaced so callers cannot continue beside a prior canonical PASS.
pub fn invalidate_final_reports_checked(report_path: impl AsRef<Path>) -> Result<(), ReportError> {
    let report_path = report_path.as_ref();
    let evidence_path = evidence_path_for_report(report_path)?;
    // Evidence is published first and the run report is the commit marker, so
    // invalidate in the same order. A non-removable run path must not prevent
    // cleanup of a partially published evidence companion.
    let mut removal_failure = None;
    for path in [evidence_path.as_path(), report_path] {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                if removal_failure.is_none() {
                    removal_failure = Some(error);
                }
            }
        }
    }
    removal_failure.map_or(Ok(()), |error| Err(ReportError::Io(error)))
}

/// Resolve and invalidate the canonical report companions before any other
/// fallible harness configuration. A later probe or fixture rejection cannot
/// then leave a prior PASS visible under this invocation's selected path.
pub fn take_run_report_ownership(
    current_dir: impl AsRef<Path>,
    configured: Option<String>,
) -> Result<PathBuf, ReportError> {
    let selected = match configured {
        Some(path) if path.trim().is_empty() => {
            return Err(ReportError::InvalidReportPath(PathBuf::from(path)));
        }
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("target/teacher-parity/parity-report.json"),
    };
    let report_path = if selected.is_absolute() {
        selected
    } else {
        current_dir.as_ref().join(selected)
    };
    invalidate_final_reports_checked(&report_path)?;
    Ok(report_path)
}

pub fn events_path_for_report(report_path: impl AsRef<Path>) -> Result<PathBuf, ReportError> {
    sibling_path(report_path.as_ref(), "events.jsonl")
}

pub fn evidence_path_for_report(report_path: impl AsRef<Path>) -> Result<PathBuf, ReportError> {
    sibling_path(report_path.as_ref(), "evidence.json")
}

fn sibling_path(report_path: &Path, suffix: &str) -> Result<PathBuf, ReportError> {
    let stem = report_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| ReportError::InvalidReportPath(report_path.to_owned()))?;
    Ok(report_path.with_file_name(format!("{stem}.{suffix}")))
}

fn temporary_sibling(path: &Path, label: &str) -> Result<PathBuf, ReportError> {
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ReportError::InvalidReportPath(path.to_owned()))?;
    Ok(path.with_file_name(format!(
        ".{filename}.{label}-{}-{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    )))
}

fn write_synced_new(path: &Path, bytes: &[u8]) -> Result<(), ReportError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

/// Atomically publish one standalone JSON artifact after synced temporary
/// write and semantic readback. This is used by cheap preflight-only entry
/// points that deliberately do not initialize the heartbeat/run machinery.
pub fn write_atomic_json(path: impl AsRef<Path>, value: &Value) -> Result<PathBuf, ReportError> {
    let path = path.as_ref();
    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory)?;
    }
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    let temporary = temporary_sibling(path, "standalone")?;
    let result = (|| -> Result<(), ReportError> {
        write_synced_new(&temporary, &bytes)?;
        let decoded: Value = serde_json::from_slice(&std::fs::read(&temporary)?)?;
        if decoded != *value {
            return Err(ReportError::InvalidCompanion(
                "standalone JSON readback changed serialized content".to_owned(),
            ));
        }
        std::fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result.map(|()| path.to_owned())
}

/// Publish a standalone teacher-free preflight outcome before returning it.
/// A non-PASS outcome receives a caller-built evidence object so the exact
/// refusal survives process failure. If publication itself fails, the write
/// failure and the original refusal are both retained in the returned reason.
pub fn publish_atomic_preflight_outcome(
    path: impl AsRef<Path>,
    outcome: Result<Value, String>,
    failure_report: impl FnOnce(&str) -> Value,
) -> Result<Value, String> {
    let path = path.as_ref();
    // A preflight artifact is an admission token for expensive teacher work.
    // Once a new attempt owns this path, an older AVAILABLE token must not
    // survive a later publication failure and masquerade as the current run.
    if let Err(error) = std::fs::remove_file(path) {
        if error.kind() != io::ErrorKind::NotFound {
            return Err(match &outcome {
                Ok(_) => format!(
                    "FAILED: invalidate stale teacher-free preflight report {}: {error}",
                    path.display()
                ),
                Err(reason) => format!(
                    "FAILED: teacher-free preflight refused ({reason}); invalidate stale report {}: {error}",
                    path.display()
                ),
            });
        }
    }
    match outcome {
        Ok(report) => {
            write_atomic_json(path, &report).map_err(|error| {
                format!(
                    "FAILED: write teacher-free preflight report {}: {error}",
                    path.display()
                )
            })?;
            Ok(report)
        }
        Err(reason) => {
            let report = failure_report(&reason);
            write_atomic_json(path, &report).map_err(|error| {
                format!(
                    "FAILED: teacher-free preflight refused ({reason}); write durable refusal report {}: {error}",
                    path.display()
                )
            })?;
            Err(reason)
        }
    }
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

pub fn unix_millis_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(duration_millis)
        .unwrap_or(0)
}

#[derive(Debug)]
pub enum ReportError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidReportPath(PathBuf),
    InvalidCompanion(String),
}

impl fmt::Display for ReportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "telemetry I/O: {error}"),
            Self::Json(error) => write!(formatter, "telemetry JSON: {error}"),
            Self::InvalidReportPath(path) => {
                write!(formatter, "invalid report path {}", path.display())
            }
            Self::InvalidCompanion(reason) => {
                write!(formatter, "invalid report companion: {reason}")
            }
        }
    }
}

impl std::error::Error for ReportError {}

impl From<io::Error> for ReportError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ReportError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
