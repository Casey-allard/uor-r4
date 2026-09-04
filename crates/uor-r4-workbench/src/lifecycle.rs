//! Pure serialized lifecycle for the one-worker, one-job workbench.
//!
//! The controller/effect layer owns clocks, threads, pipes, and the process
//! handle.  This module owns the public state machine and its immutable
//! snapshots.  In particular, a spawned worker is never considered removed
//! merely because a stop was requested: only `confirm_reaped` clears its
//! identity and releases the single admission slot.

use crate::wire::{
    is_hex, ArtifactIdentity, HostIdentity, JobKind, JobSnapshot, JobState, ModelSnapshot,
    ModelState, NativeTextResult, Progress, ProgressStage, ServiceError, ServiceErrorTag,
    StopReason, Work, WorkerReady, JOB_SCHEMA, MODEL_SCHEMA, UINT53_MAX,
};
use std::collections::VecDeque;
use std::fmt;

pub const TERMINAL_JOB_LIMIT: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    InvalidInitialState(&'static str),
    InvalidWireValue(String),
    Busy,
    NotReady,
    AlreadyLoaded,
    AlreadyUnloaded,
    StaleModel { expected: u64, actual: u64 },
    JobNotFound,
    AlreadyTerminal,
    NotCancellable,
    WrongActiveJob,
    InvalidTransition(&'static str),
    CounterExhausted(&'static str),
    WorkerAlreadySpawned,
    WorkerNotSpawned,
    WorkerReadyMismatch(&'static str),
    ResultIdentityMismatch(&'static str),
    InvalidProgress(&'static str),
    MissingUnloadAcknowledgement,
}

impl LifecycleError {
    /// Stable outer tag for a rejected public admission or an internal event.
    /// The caller still owns the bounded user-facing message.
    pub fn service_tag(&self) -> ServiceErrorTag {
        match self {
            Self::Busy => ServiceErrorTag::Busy,
            Self::NotReady => ServiceErrorTag::NotReady,
            Self::AlreadyLoaded => ServiceErrorTag::AlreadyLoaded,
            Self::AlreadyUnloaded => ServiceErrorTag::AlreadyUnloaded,
            Self::StaleModel { .. } => ServiceErrorTag::StaleModel,
            Self::JobNotFound => ServiceErrorTag::JobNotFound,
            Self::AlreadyTerminal => ServiceErrorTag::AlreadyTerminal,
            Self::NotCancellable => ServiceErrorTag::NotCancellable,
            Self::MissingUnloadAcknowledgement => ServiceErrorTag::WorkerFailure,
            Self::WorkerReadyMismatch(_) | Self::ResultIdentityMismatch(_) => {
                ServiceErrorTag::WorkerProtocolFailure
            }
            Self::InvalidInitialState(_)
            | Self::InvalidWireValue(_)
            | Self::WrongActiveJob
            | Self::InvalidTransition(_)
            | Self::CounterExhausted(_)
            | Self::WorkerAlreadySpawned
            | Self::WorkerNotSpawned
            | Self::InvalidProgress(_) => ServiceErrorTag::WorkerProtocolFailure,
        }
    }
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInitialState(message) => {
                write!(formatter, "invalid initial state: {message}")
            }
            Self::InvalidWireValue(message) => write!(formatter, "invalid wire value: {message}"),
            Self::Busy => formatter.write_str("the single lifecycle slot is busy"),
            Self::NotReady => formatter.write_str("the model is not ready"),
            Self::AlreadyLoaded => formatter.write_str("the model is already loaded"),
            Self::AlreadyUnloaded => formatter.write_str("the model is already unloaded"),
            Self::StaleModel { expected, actual } => {
                write!(
                    formatter,
                    "stale model generation {expected}; current is {actual}"
                )
            }
            Self::JobNotFound => formatter.write_str("job not found"),
            Self::AlreadyTerminal => formatter.write_str("job is already terminal"),
            Self::NotCancellable => formatter.write_str("job kind is not cancellable"),
            Self::WrongActiveJob => formatter.write_str("event does not match the active job"),
            Self::InvalidTransition(message) => {
                write!(formatter, "invalid lifecycle transition: {message}")
            }
            Self::CounterExhausted(counter) => write!(formatter, "{counter} exhausted uint53"),
            Self::WorkerAlreadySpawned => formatter.write_str("a worker is already spawned"),
            Self::WorkerNotSpawned => formatter.write_str("no worker has been spawned"),
            Self::WorkerReadyMismatch(axis) => {
                write!(formatter, "worker readiness mismatches {axis}")
            }
            Self::ResultIdentityMismatch(axis) => {
                write!(formatter, "worker result mismatches {axis}")
            }
            Self::InvalidProgress(message) => write!(formatter, "invalid progress: {message}"),
            Self::MissingUnloadAcknowledgement => {
                formatter.write_str("worker was reaped without the required unload acknowledgment")
            }
        }
    }
}

impl std::error::Error for LifecycleError {}

#[derive(Debug, Clone)]
struct StopWinner {
    reason: StopReason,
    cause: Option<ServiceError>,
}

#[derive(Debug, Clone)]
struct ActiveJob {
    snapshot: JobSnapshot,
    launch_pending: bool,
    child_spawned: bool,
    dispatched: bool,
    generation_invalidated: bool,
    unload_acknowledged: bool,
    stop: Option<StopWinner>,
    stop_progress_rank: u8,
}

#[derive(Debug, Clone)]
struct IdleStop {
    winner: StopWinner,
    progress_rank: u8,
}

#[derive(Debug, Clone)]
pub struct Lifecycle {
    instance_id: String,
    host: HostIdentity,
    artifact: ArtifactIdentity,
    revision: u64,
    next_job_id: Option<u64>,
    worker_generation_counter: u64,
    current_worker_generation: Option<u64>,
    model: ModelSnapshot,
    active: Option<ActiveJob>,
    terminal: VecDeque<JobSnapshot>,
    idle_stop: Option<IdleStop>,
}

impl Lifecycle {
    /// Start in discovery-only mode.  No accepted host identity is inferred
    /// from historical evidence or from the configured artifact.
    pub fn new_unavailable(
        instance_id: String,
        host: HostIdentity,
        artifact: ArtifactIdentity,
        error: ServiceError,
    ) -> Result<Self, LifecycleError> {
        validate_common_initial(&instance_id, &host, &artifact)?;
        error
            .validate()
            .map_err(|error| LifecycleError::InvalidWireValue(error.to_string()))?;
        if host.runtime_receipt_sha256.is_some()
            || host.host_acceptance_sha256.is_some()
            || host.qualification_receipt_sha256.is_some()
        {
            return Err(LifecycleError::InvalidInitialState(
                "unavailable host must not publish acceptance receipts",
            ));
        }
        Ok(Self::new(
            instance_id,
            host,
            artifact,
            ModelState::Unavailable,
            ProgressStage::Error,
            Some(error),
        ))
    }

    /// Start with an independently adopted host bundle but no worker.
    pub fn new_unloaded(
        instance_id: String,
        host: HostIdentity,
        artifact: ArtifactIdentity,
    ) -> Result<Self, LifecycleError> {
        validate_common_initial(&instance_id, &host, &artifact)?;
        if host.runtime_receipt_sha256.is_none()
            || host.host_acceptance_sha256.is_none()
            || host.qualification_receipt_sha256.is_none()
        {
            return Err(LifecycleError::InvalidInitialState(
                "unloaded host requires adopted runtime, qualification, and host acceptance",
            ));
        }
        Ok(Self::new(
            instance_id,
            host,
            artifact,
            ModelState::Unloaded,
            ProgressStage::Idle,
            None,
        ))
    }

    fn new(
        instance_id: String,
        host: HostIdentity,
        artifact: ArtifactIdentity,
        state: ModelState,
        stage: ProgressStage,
        error: Option<ServiceError>,
    ) -> Self {
        let model = ModelSnapshot {
            schema: MODEL_SCHEMA.to_owned(),
            instance_id: instance_id.clone(),
            revision: 0,
            model_id: artifact.model_id.clone(),
            model_generation: 0,
            state,
            verified_artifact: None,
            qualification_receipt_sha256: None,
            active_job_id: None,
            last_job_id: None,
            progress: progress(stage),
            error,
        };
        Self {
            instance_id,
            host,
            artifact,
            revision: 0,
            next_job_id: Some(1),
            worker_generation_counter: 0,
            current_worker_generation: None,
            model,
            active: None,
            terminal: VecDeque::with_capacity(TERMINAL_JOB_LIMIT),
            idle_stop: None,
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn host_identity(&self) -> &HostIdentity {
        &self.host
    }

    pub fn configured_artifact(&self) -> &ArtifactIdentity {
        &self.artifact
    }

    pub fn model_snapshot(&self) -> ModelSnapshot {
        self.model.clone()
    }

    pub fn active_job(&self) -> Option<JobSnapshot> {
        self.active.as_ref().map(|active| active.snapshot.clone())
    }

    pub fn active_job_id(&self) -> Option<&str> {
        self.active
            .as_ref()
            .map(|active| active.snapshot.job_id.as_str())
    }

    pub fn active_job_kind(&self) -> Option<JobKind> {
        self.active.as_ref().map(|active| active.snapshot.kind)
    }

    pub fn current_worker_generation(&self) -> Option<u64> {
        self.current_worker_generation
    }

    pub fn worker_spawned_for_active_job(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|active| active.child_spawned)
    }

    pub fn terminal_job_count(&self) -> usize {
        self.terminal.len()
    }

    /// Polling is a pure clone.  It cannot execute or mutate work.
    pub fn job(&self, job_id: &str) -> Option<JobSnapshot> {
        self.active
            .as_ref()
            .filter(|active| active.snapshot.job_id == job_id)
            .map(|active| active.snapshot.clone())
            .or_else(|| {
                self.terminal
                    .iter()
                    .find(|job| job.job_id == job_id)
                    .cloned()
            })
    }

    pub fn admit_load(&mut self) -> Result<JobSnapshot, LifecycleError> {
        self.require_free_slot()?;
        match self.model.state {
            ModelState::Unloaded | ModelState::Error => {}
            ModelState::Ready => return Err(LifecycleError::AlreadyLoaded),
            _ => return Err(LifecycleError::NotReady),
        }
        if self.current_worker_generation.is_some() || self.idle_stop.is_some() {
            return Err(LifecycleError::Busy);
        }
        self.reserve_job(JobKind::Load, None, 0, ModelState::Loading)
    }

    pub fn admit_answer(
        &mut self,
        expected_generation: u64,
        raw_text_sha256: String,
    ) -> Result<JobSnapshot, LifecycleError> {
        self.require_free_slot()?;
        if self.model.state != ModelState::Ready {
            return Err(LifecycleError::NotReady);
        }
        if expected_generation != self.model.model_generation {
            return Err(LifecycleError::StaleModel {
                expected: expected_generation,
                actual: self.model.model_generation,
            });
        }
        if !is_hex(&raw_text_sha256, 64) {
            return Err(LifecycleError::InvalidWireValue(
                "answer input digest must be lowercase hex64".to_owned(),
            ));
        }
        if self.current_worker_generation.is_none() {
            return Err(LifecycleError::InvalidTransition(
                "ready model has no current worker",
            ));
        }
        let snapshot = self.reserve_job(
            JobKind::Answer,
            Some(raw_text_sha256),
            1,
            ModelState::Running,
        )?;
        let active = self.active.as_mut().ok_or(LifecycleError::WrongActiveJob)?;
        active.child_spawned = true;
        Ok(snapshot)
    }

    pub fn admit_unload(
        &mut self,
        expected_generation: u64,
    ) -> Result<JobSnapshot, LifecycleError> {
        self.require_free_slot()?;
        if self.model.state == ModelState::Unloaded {
            return Err(LifecycleError::AlreadyUnloaded);
        }
        if self.model.state != ModelState::Ready {
            return Err(LifecycleError::NotReady);
        }
        if expected_generation != self.model.model_generation {
            return Err(LifecycleError::StaleModel {
                expected: expected_generation,
                actual: self.model.model_generation,
            });
        }
        if self.current_worker_generation.is_none() {
            return Err(LifecycleError::InvalidTransition(
                "ready model has no current worker",
            ));
        }
        let next_generation = checked_increment(self.model.model_generation, "model generation")?;
        let snapshot = self.reserve_job(JobKind::Unload, None, 0, ModelState::Unloading)?;
        self.model.model_generation = next_generation;
        let active = self.active.as_mut().ok_or(LifecycleError::WrongActiveJob)?;
        active.child_spawned = true;
        active.generation_invalidated = true;
        Ok(snapshot)
    }

    fn reserve_job(
        &mut self,
        kind: JobKind,
        raw_text_sha256: Option<String>,
        forward_upper_bound: u64,
        model_state: ModelState,
    ) -> Result<JobSnapshot, LifecycleError> {
        let numeric_job_id = self
            .next_job_id
            .ok_or(LifecycleError::CounterExhausted("job ID"))?;
        let revision = checked_increment(self.revision, "revision")?;
        let next_job_id = if numeric_job_id == UINT53_MAX {
            None
        } else {
            Some(numeric_job_id + 1)
        };
        let job_id = numeric_job_id.to_string();
        let snapshot = JobSnapshot {
            schema: JOB_SCHEMA.to_owned(),
            instance_id: self.instance_id.clone(),
            revision,
            job_id: job_id.clone(),
            kind,
            state: JobState::Accepted,
            model_id: self.artifact.model_id.clone(),
            admitted_generation: self.model.model_generation,
            raw_text_sha256,
            progress: progress(ProgressStage::Idle),
            stop_reason: None,
            result: None,
            error: None,
            work: Work {
                forward_count: Some(0),
                forward_upper_bound,
                elapsed_ms: None,
            },
            host: self.host.clone(),
            artifact: self.artifact.clone(),
        };
        self.revision = revision;
        self.next_job_id = next_job_id;
        self.model.revision = revision;
        self.model.state = model_state;
        self.model.active_job_id = Some(job_id.clone());
        self.model.last_job_id = Some(job_id);
        self.model.progress = snapshot.progress.clone();
        self.model.error = None;
        self.active = Some(ActiveJob {
            snapshot: snapshot.clone(),
            launch_pending: false,
            child_spawned: false,
            dispatched: false,
            generation_invalidated: false,
            unload_acknowledged: false,
            stop: None,
            stop_progress_rank: 0,
        });
        Ok(snapshot)
    }

    /// Mark that the controller owns one outstanding launcher thread.  This
    /// private reservation keeps cancel/deadline from freeing the public slot
    /// while that thread could still return a newly created child.
    pub fn note_load_launch_pending(&mut self, job_id: &str) -> Result<(), LifecycleError> {
        self.require_active(job_id, JobKind::Load)?;
        let active = self.active.as_ref().ok_or(LifecycleError::WrongActiveJob)?;
        if active.launch_pending || active.child_spawned || self.current_worker_generation.is_some()
        {
            return Err(LifecycleError::WorkerAlreadySpawned);
        }
        // Prove now, before an operating-system effect starts, that adopting a
        // returned child cannot fail because the private counter is exhausted.
        checked_increment(self.worker_generation_counter, "private worker generation")?;
        self.active
            .as_mut()
            .ok_or(LifecycleError::WrongActiveJob)?
            .launch_pending = true;
        Ok(())
    }

    /// Run the caller's one child-adoption effect only after every lifecycle
    /// and counter precondition has been checked, then publish the returned
    /// child and its private generation without another fallible transition.
    ///
    /// The nested result keeps an operating-system spawn error distinct from
    /// a lifecycle rejection. A failed effect leaves the lifecycle in the
    /// exact pre-spawn state, while an accepted effect cannot escape without
    /// `child_spawned` and `current_worker_generation` being installed.
    pub fn spawn_load_worker<T, E>(
        &mut self,
        job_id: &str,
        spawn: impl FnOnce() -> Result<T, E>,
    ) -> Result<Result<(u64, T), E>, LifecycleError> {
        self.require_active(job_id, JobKind::Load)?;
        if self.current_worker_generation.is_some()
            || self
                .active
                .as_ref()
                .is_some_and(|active| active.child_spawned)
        {
            return Err(LifecycleError::WorkerAlreadySpawned);
        }
        let generation =
            checked_increment(self.worker_generation_counter, "private worker generation")?;
        let stopped_while_launching = self
            .active
            .as_ref()
            .is_some_and(|active| active.launch_pending && active.stop.is_some());
        let stopped_model_generation = if stopped_while_launching {
            Some(checked_increment(
                self.model.model_generation,
                "model generation",
            )?)
        } else {
            None
        };
        let stopped_revision = if stopped_while_launching {
            Some(checked_increment(self.revision, "revision")?)
        } else {
            None
        };
        let child = match spawn() {
            Ok(child) => child,
            Err(error) => return Ok(Err(error)),
        };
        self.worker_generation_counter = generation;
        self.current_worker_generation = Some(generation);
        let active = self.active.as_mut().ok_or(LifecycleError::WrongActiveJob)?;
        active.launch_pending = false;
        active.child_spawned = true;
        if let (Some(model_generation), Some(revision)) =
            (stopped_model_generation, stopped_revision)
        {
            self.revision = revision;
            self.model.revision = revision;
            self.model.model_generation = model_generation;
            active.snapshot.revision = revision;
            active.generation_invalidated = true;
        }
        Ok(Ok((generation, child)))
    }

    /// Resolve a launcher which proved that no child was created.  An earlier
    /// cancellation/deadline remains the winner; otherwise the supplied spawn
    /// failure becomes the terminal pre-spawn cause.
    pub fn finish_load_launch_without_child(
        &mut self,
        job_id: &str,
        cause: ServiceError,
    ) -> Result<JobSnapshot, LifecycleError> {
        cause
            .validate()
            .map_err(|error| LifecycleError::InvalidWireValue(error.to_string()))?;
        self.require_active(job_id, JobKind::Load)?;
        let active = self.active.as_mut().ok_or(LifecycleError::WrongActiveJob)?;
        if !active.launch_pending || active.child_spawned {
            return Err(LifecycleError::InvalidTransition(
                "load launcher is not pending",
            ));
        }
        active.launch_pending = false;
        let winner = active.stop.clone();
        match winner {
            Some(winner) => self.commit_pre_spawn_stop(winner.reason, winner.cause),
            None => self.commit_pre_spawn_stop(StopReason::WorkerFailure, Some(cause)),
        }
    }

    #[cfg(test)]
    pub fn note_load_worker_spawned(&mut self, job_id: &str) -> Result<u64, LifecycleError> {
        self.spawn_load_worker(job_id, || Ok::<(), ()>(()))?
            .map(|(generation, ())| generation)
            .map_err(|()| LifecycleError::InvalidTransition("infallible test spawn failed"))
    }

    pub fn mark_dispatched(&mut self, job_id: &str) -> Result<JobSnapshot, LifecycleError> {
        let kind = self.require_active_any(job_id)?.snapshot.kind;
        let active = self.active.as_ref().ok_or(LifecycleError::WrongActiveJob)?;
        if active.stop.is_some() {
            return Err(LifecycleError::InvalidTransition("stop already won"));
        }
        if active.dispatched {
            return Err(LifecycleError::InvalidTransition("job already dispatched"));
        }
        if !active.child_spawned {
            return Err(LifecycleError::WorkerNotSpawned);
        }
        let revision = checked_increment(self.revision, "revision")?;
        self.revision = revision;
        let active = self.active.as_mut().ok_or(LifecycleError::WrongActiveJob)?;
        active.dispatched = true;
        active.snapshot.revision = revision;
        active.snapshot.state = working_state(kind);
        if kind == JobKind::Answer {
            active.snapshot.work.forward_count = None;
        }
        self.model.revision = revision;
        Ok(active.snapshot.clone())
    }

    /// Record only an actually reported stage.  The method permits skipped
    /// stages but never backward movement or synthesized percentages.
    pub fn report_progress(
        &mut self,
        job_id: &str,
        next: Progress,
    ) -> Result<JobSnapshot, LifecycleError> {
        next.validate()
            .map_err(|error| LifecycleError::InvalidWireValue(error.to_string()))?;
        let active = self.require_active_any(job_id)?;
        if active.stop.is_some() {
            return self.report_stop_progress(job_id, next);
        }
        if !active.dispatched {
            return Err(LifecycleError::InvalidTransition(
                "progress preceded command dispatch",
            ));
        }
        validate_work_progress(active.snapshot.kind, &active.snapshot.progress, &next)?;
        let revision = checked_increment(self.revision, "revision")?;
        self.revision = revision;
        let active = self.active.as_mut().ok_or(LifecycleError::WrongActiveJob)?;
        active.snapshot.revision = revision;
        active.snapshot.progress = next.clone();
        self.model.revision = revision;
        self.model.progress = next;
        Ok(active.snapshot.clone())
    }

    fn report_stop_progress(
        &mut self,
        job_id: &str,
        next: Progress,
    ) -> Result<JobSnapshot, LifecycleError> {
        self.require_active_any(job_id)?;
        let rank = match next.stage {
            ProgressStage::Terminating => 1,
            ProgressStage::Reaping => 2,
            _ => {
                return Err(LifecycleError::InvalidProgress(
                    "stopping permits only actual terminating or reaping stages",
                ))
            }
        };
        let prior = self
            .active
            .as_ref()
            .ok_or(LifecycleError::WrongActiveJob)?
            .stop_progress_rank;
        if rank < prior {
            return Err(LifecycleError::InvalidProgress(
                "stopping progress moved backwards",
            ));
        }
        let revision = checked_increment(self.revision, "revision")?;
        self.revision = revision;
        let active = self.active.as_mut().ok_or(LifecycleError::WrongActiveJob)?;
        active.stop_progress_rank = rank;
        active.snapshot.revision = revision;
        active.snapshot.progress = next.clone();
        self.model.revision = revision;
        self.model.progress = next;
        Ok(active.snapshot.clone())
    }

    pub fn accept_worker_ready(
        &mut self,
        job_id: &str,
        ready: WorkerReady,
        elapsed_ms: Option<u64>,
    ) -> Result<JobSnapshot, LifecycleError> {
        validate_elapsed(elapsed_ms)?;
        self.require_active(job_id, JobKind::Load)?;
        let active = self.active.as_ref().ok_or(LifecycleError::WrongActiveJob)?;
        if active.stop.is_some() {
            return Err(LifecycleError::InvalidTransition("stop already won"));
        }
        if !active.child_spawned || !active.dispatched {
            return Err(LifecycleError::InvalidTransition(
                "worker readiness preceded spawn or dispatch",
            ));
        }
        ready
            .host
            .validate()
            .and_then(|_| ready.artifact.validate())
            .map_err(|error| LifecycleError::InvalidWireValue(error.to_string()))?;
        if ready.host != self.host {
            return Err(LifecycleError::WorkerReadyMismatch("host identity"));
        }
        if ready.artifact != self.artifact {
            return Err(LifecycleError::WorkerReadyMismatch("artifact identity"));
        }
        let qualification = ready.host.qualification_receipt_sha256.clone().ok_or(
            LifecycleError::WorkerReadyMismatch("installed qualification receipt"),
        )?;
        let ready_generation = checked_increment(self.model.model_generation, "model generation")?;
        let terminal = self.commit_normal(
            JobState::Completed,
            ModelState::Ready,
            progress(ProgressStage::Complete),
            progress(ProgressStage::Ready),
            None,
            elapsed_ms,
            None,
        )?;
        self.model.model_generation = ready_generation;
        self.model.verified_artifact = Some(ready.artifact);
        self.model.qualification_receipt_sha256 = Some(qualification);
        Ok(terminal)
    }

    pub fn accept_answer_result(
        &mut self,
        job_id: &str,
        result: NativeTextResult,
        elapsed_ms: Option<u64>,
    ) -> Result<JobSnapshot, LifecycleError> {
        validate_elapsed(elapsed_ms)?;
        self.require_active(job_id, JobKind::Answer)?;
        let active = self.active.as_ref().ok_or(LifecycleError::WrongActiveJob)?;
        if active.stop.is_some() {
            return Err(LifecycleError::InvalidTransition("stop already won"));
        }
        if !active.dispatched {
            return Err(LifecycleError::InvalidTransition(
                "answer result preceded dispatch",
            ));
        }
        result
            .validate()
            .map_err(|error| LifecycleError::InvalidWireValue(error.to_string()))?;
        validate_result_identity(&active.snapshot, &self.model, &result)?;
        let (forward_count, forward_upper_bound) = match &result {
            NativeTextResult::Model(_) => (Some(1), 1),
            NativeTextResult::Refusal(_) => (Some(0), 0),
        };
        self.commit_normal(
            JobState::Completed,
            ModelState::Ready,
            progress(ProgressStage::Complete),
            progress(ProgressStage::Ready),
            Some(result),
            elapsed_ms,
            Some((forward_count, forward_upper_bound)),
        )
    }

    fn commit_normal(
        &mut self,
        terminal_state: JobState,
        model_state: ModelState,
        job_progress: Progress,
        model_progress: Progress,
        result: Option<NativeTextResult>,
        elapsed_ms: Option<u64>,
        final_forward_counts: Option<(Option<u64>, u64)>,
    ) -> Result<JobSnapshot, LifecycleError> {
        let revision = checked_increment(self.revision, "revision")?;
        let mut active = self.active.take().ok_or(LifecycleError::WrongActiveJob)?;
        active.snapshot.revision = revision;
        active.snapshot.state = terminal_state;
        active.snapshot.progress = job_progress;
        active.snapshot.stop_reason = None;
        active.snapshot.result = result;
        active.snapshot.error = None;
        active.snapshot.work.elapsed_ms = elapsed_ms;
        if let Some((forward_count, forward_upper_bound)) = final_forward_counts {
            active.snapshot.work.forward_count = forward_count;
            active.snapshot.work.forward_upper_bound = forward_upper_bound;
        }
        self.revision = revision;
        self.model.revision = revision;
        self.model.state = model_state;
        self.model.active_job_id = None;
        self.model.progress = model_progress;
        self.model.error = None;
        self.retain_terminal(active.snapshot.clone());
        Ok(active.snapshot)
    }

    /// The unload acknowledgment is necessary but not sufficient.  It is a
    /// private fact until the same owned child is confirmed reaped.
    pub fn acknowledge_unloaded(&mut self, job_id: &str) -> Result<(), LifecycleError> {
        self.require_active(job_id, JobKind::Unload)?;
        let active = self.active.as_ref().ok_or(LifecycleError::WrongActiveJob)?;
        if active.stop.is_some() {
            return Err(LifecycleError::InvalidTransition("stop already won"));
        }
        if !active.dispatched {
            return Err(LifecycleError::InvalidTransition(
                "unload acknowledgment preceded dispatch",
            ));
        }
        self.active
            .as_mut()
            .ok_or(LifecycleError::WrongActiveJob)?
            .unload_acknowledged = true;
        Ok(())
    }

    /// Request user cancellation.  Repeated cancellation while stopping is
    /// idempotent and cannot replace the first stop winner.
    pub fn request_cancel(&mut self, job_id: &str) -> Result<JobSnapshot, LifecycleError> {
        if self.terminal.iter().any(|job| job.job_id == job_id) {
            return Err(LifecycleError::AlreadyTerminal);
        }
        let active = self.require_active_any(job_id)?;
        if active.snapshot.kind == JobKind::Unload {
            return Err(LifecycleError::NotCancellable);
        }
        if active.stop.is_some() {
            return Ok(active.snapshot.clone());
        }
        if active.snapshot.kind == JobKind::Load && !active.child_spawned {
            if active.launch_pending {
                return self.begin_pending_launch_stop(StopReason::UserCancel, None);
            }
            return self.commit_pre_spawn_stop(StopReason::UserCancel, None);
        }
        self.begin_spawned_stop(StopReason::UserCancel, None)
    }

    pub fn deadline(&mut self, job_id: &str) -> Result<JobSnapshot, LifecycleError> {
        let active = self.require_active_any(job_id)?;
        if active.stop.is_some() {
            return Ok(active.snapshot.clone());
        }
        let cause = deadline_error();
        if active.snapshot.kind == JobKind::Load && !active.child_spawned {
            if active.launch_pending {
                return self.begin_pending_launch_stop(StopReason::Deadline, Some(cause));
            }
            return self.commit_pre_spawn_stop(StopReason::Deadline, Some(cause));
        }
        self.begin_spawned_stop(StopReason::Deadline, Some(cause))
    }

    /// Preserve the exact mapped cause.  A failure reply does not imply that
    /// the worker exited, so spawned failures remain stopping until reap.
    pub fn worker_failure(
        &mut self,
        job_id: &str,
        cause: ServiceError,
    ) -> Result<JobSnapshot, LifecycleError> {
        cause
            .validate()
            .map_err(|error| LifecycleError::InvalidWireValue(error.to_string()))?;
        let active = self.require_active_any(job_id)?;
        if active.stop.is_some() {
            return Ok(active.snapshot.clone());
        }
        if active.snapshot.kind == JobKind::Load && !active.child_spawned {
            if active.launch_pending {
                return self.begin_pending_launch_stop(StopReason::WorkerFailure, Some(cause));
            }
            return self.commit_pre_spawn_stop(StopReason::WorkerFailure, Some(cause));
        }
        self.begin_spawned_stop(StopReason::WorkerFailure, Some(cause))
    }

    fn begin_pending_launch_stop(
        &mut self,
        reason: StopReason,
        cause: Option<ServiceError>,
    ) -> Result<JobSnapshot, LifecycleError> {
        let active = self.active.as_ref().ok_or(LifecycleError::WrongActiveJob)?;
        if active.snapshot.kind != JobKind::Load || !active.launch_pending || active.child_spawned {
            return Err(LifecycleError::InvalidTransition(
                "pending-launch stop requires an unresolved load launcher",
            ));
        }
        // Reserve arithmetic for either possible launcher outcome. No counter
        // changes until a returned child is actually adopted.
        checked_increment(self.model.model_generation, "model generation")?;
        let revision = checked_increment(self.revision, "revision")?;
        checked_increment(revision, "revision")?;
        self.revision = revision;
        self.model.revision = revision;
        self.model.state = ModelState::Stopping;
        self.model.error = cause.clone();
        let active = self.active.as_mut().ok_or(LifecycleError::WrongActiveJob)?;
        active.stop = Some(StopWinner {
            reason,
            cause: cause.clone(),
        });
        active.snapshot.revision = revision;
        active.snapshot.state = JobState::Stopping;
        active.snapshot.stop_reason = Some(reason);
        active.snapshot.result = None;
        active.snapshot.error = cause;
        Ok(active.snapshot.clone())
    }

    fn begin_spawned_stop(
        &mut self,
        reason: StopReason,
        cause: Option<ServiceError>,
    ) -> Result<JobSnapshot, LifecycleError> {
        if !self
            .active
            .as_ref()
            .is_some_and(|active| active.child_spawned)
        {
            return Err(LifecycleError::WorkerNotSpawned);
        }
        let already_invalidated = self
            .active
            .as_ref()
            .ok_or(LifecycleError::WrongActiveJob)?
            .generation_invalidated;
        let next_generation = if already_invalidated {
            self.model.model_generation
        } else {
            checked_increment(self.model.model_generation, "model generation")?
        };
        let revision = checked_increment(self.revision, "revision")?;
        self.revision = revision;
        self.model.revision = revision;
        self.model.model_generation = next_generation;
        self.model.state = ModelState::Stopping;
        self.model.error = cause.clone();
        let active = self.active.as_mut().ok_or(LifecycleError::WrongActiveJob)?;
        active.generation_invalidated = true;
        active.stop = Some(StopWinner {
            reason,
            cause: cause.clone(),
        });
        active.snapshot.revision = revision;
        active.snapshot.state = JobState::Stopping;
        active.snapshot.stop_reason = Some(reason);
        active.snapshot.result = None;
        active.snapshot.error = cause;
        Ok(active.snapshot.clone())
    }

    fn commit_pre_spawn_stop(
        &mut self,
        reason: StopReason,
        cause: Option<ServiceError>,
    ) -> Result<JobSnapshot, LifecycleError> {
        let revision = checked_increment(self.revision, "revision")?;
        let mut active = self.active.take().ok_or(LifecycleError::WrongActiveJob)?;
        if active.snapshot.kind != JobKind::Load || active.child_spawned {
            self.active = Some(active);
            return Err(LifecycleError::InvalidTransition(
                "direct terminal stop requires pre-spawn load",
            ));
        }
        let user_cancel = reason == StopReason::UserCancel;
        active.snapshot.revision = revision;
        active.snapshot.state = if user_cancel {
            JobState::Cancelled
        } else {
            JobState::Failed
        };
        active.snapshot.progress = progress(if user_cancel {
            ProgressStage::Complete
        } else {
            ProgressStage::Error
        });
        active.snapshot.stop_reason = Some(reason);
        active.snapshot.error = if user_cancel { None } else { cause.clone() };
        active.snapshot.result = None;
        active.snapshot.work.forward_count = Some(0);
        active.snapshot.work.forward_upper_bound = 0;
        self.revision = revision;
        self.model.revision = revision;
        self.model.state = if user_cancel {
            ModelState::Unloaded
        } else {
            ModelState::Error
        };
        self.model.active_job_id = None;
        self.model.progress = progress(if user_cancel {
            ProgressStage::Idle
        } else {
            ProgressStage::Error
        });
        self.model.error = if user_cancel { None } else { cause };
        self.clear_installed_worker();
        self.retain_terminal(active.snapshot.clone());
        Ok(active.snapshot)
    }

    /// An elapsed grace/force window is not evidence of exit.  Keep the slot
    /// occupied and the original stop winner private for later resolution.
    pub fn note_termination_unconfirmed(
        &mut self,
        job_id: &str,
    ) -> Result<JobSnapshot, LifecycleError> {
        let active = self.require_active_any(job_id)?;
        if active.stop.is_none() || self.model.state != ModelState::Stopping {
            return Err(LifecycleError::InvalidTransition(
                "termination uncertainty requires a stopping worker",
            ));
        }
        let error = termination_unconfirmed_error();
        let revision = checked_increment(self.revision, "revision")?;
        self.revision = revision;
        self.model.revision = revision;
        self.model.error = Some(error.clone());
        let active = self.active.as_mut().ok_or(LifecycleError::WrongActiveJob)?;
        active.snapshot.revision = revision;
        active.snapshot.error = Some(error);
        Ok(active.snapshot.clone())
    }

    /// Confirm removal of the exact owned worker.  For an ordinary unload,
    /// the caller must supply the causal WORKER_FAILURE used if the child
    /// exited without its required acknowledgment.
    pub fn confirm_reaped(
        &mut self,
        job_id: &str,
        missing_unload_ack_error: ServiceError,
    ) -> Result<JobSnapshot, LifecycleError> {
        self.require_active_any(job_id)?;
        if self.current_worker_generation.is_none() {
            return Err(LifecycleError::WorkerNotSpawned);
        }
        let (kind, has_stop, unload_acknowledged) = {
            let active = self.active.as_ref().ok_or(LifecycleError::WrongActiveJob)?;
            (
                active.snapshot.kind,
                active.stop.is_some(),
                active.unload_acknowledged,
            )
        };
        if kind == JobKind::Unload && !has_stop {
            if unload_acknowledged {
                return self.commit_reaped_unload_success();
            }
            missing_unload_ack_error
                .validate()
                .map_err(|error| LifecycleError::InvalidWireValue(error.to_string()))?;
            if missing_unload_ack_error.tag != ServiceErrorTag::WorkerFailure {
                return Err(LifecycleError::MissingUnloadAcknowledgement);
            }
            self.begin_spawned_stop(StopReason::WorkerFailure, Some(missing_unload_ack_error))?;
            let winner = self
                .active
                .as_ref()
                .and_then(|active| active.stop.clone())
                .ok_or(LifecycleError::InvalidTransition(
                    "missing unload acknowledgment did not install a stop winner",
                ))?;
            let cause = winner.cause.ok_or(LifecycleError::InvalidTransition(
                "missing unload acknowledgment lost its cause",
            ))?;
            return self.commit_reaped_failure(cause);
        }
        let winner = self
            .active
            .as_ref()
            .and_then(|active| active.stop.clone())
            .ok_or(LifecycleError::InvalidTransition(
                "reap preceded unload acknowledgment or a stop winner",
            ))?;
        match winner.reason {
            StopReason::UserCancel => self.commit_reaped_cancel(),
            StopReason::Deadline | StopReason::WorkerFailure => {
                let cause = winner.cause.ok_or(LifecycleError::InvalidTransition(
                    "failure stop lost its causal error",
                ))?;
                self.commit_reaped_failure(cause)
            }
            StopReason::Shutdown => Err(LifecycleError::InvalidTransition(
                "shutdown does not promise a durable public terminal snapshot",
            )),
        }
    }

    fn commit_reaped_unload_success(&mut self) -> Result<JobSnapshot, LifecycleError> {
        let terminal = self.commit_reaped(
            JobState::Completed,
            ModelState::Unloaded,
            ProgressStage::Complete,
            ProgressStage::Idle,
            None,
        )?;
        Ok(terminal)
    }

    fn commit_reaped_cancel(&mut self) -> Result<JobSnapshot, LifecycleError> {
        self.commit_reaped(
            JobState::Cancelled,
            ModelState::Unloaded,
            ProgressStage::Complete,
            ProgressStage::Idle,
            None,
        )
    }

    fn commit_reaped_failure(
        &mut self,
        cause: ServiceError,
    ) -> Result<JobSnapshot, LifecycleError> {
        self.commit_reaped(
            JobState::Failed,
            ModelState::Error,
            ProgressStage::Error,
            ProgressStage::Error,
            Some(cause),
        )
    }

    fn commit_reaped(
        &mut self,
        job_state: JobState,
        model_state: ModelState,
        job_stage: ProgressStage,
        model_stage: ProgressStage,
        error: Option<ServiceError>,
    ) -> Result<JobSnapshot, LifecycleError> {
        let revision = checked_increment(self.revision, "revision")?;
        let mut active = self.active.take().ok_or(LifecycleError::WrongActiveJob)?;
        active.snapshot.revision = revision;
        active.snapshot.state = job_state;
        active.snapshot.progress = progress(job_stage);
        active.snapshot.result = None;
        active.snapshot.error = error.clone();
        if job_state == JobState::Cancelled
            && active.snapshot.kind == JobKind::Answer
            && active.dispatched
        {
            active.snapshot.work.forward_count = None;
            active.snapshot.work.forward_upper_bound = 1;
        } else if active.snapshot.kind != JobKind::Answer || !active.dispatched {
            active.snapshot.work.forward_count = Some(0);
        }
        self.revision = revision;
        self.model.revision = revision;
        self.model.state = model_state;
        self.model.active_job_id = None;
        self.model.progress = progress(model_stage);
        self.model.error = error;
        self.clear_installed_worker();
        self.retain_terminal(active.snapshot.clone());
        Ok(active.snapshot)
    }

    /// Enter the model-only ready -> stopping path for an unexpected idle
    /// worker failure.  No job is created or changed.
    pub fn idle_worker_failure(
        &mut self,
        cause: ServiceError,
    ) -> Result<ModelSnapshot, LifecycleError> {
        cause
            .validate()
            .map_err(|error| LifecycleError::InvalidWireValue(error.to_string()))?;
        self.require_free_slot()?;
        if self.model.state != ModelState::Ready || self.current_worker_generation.is_none() {
            return Err(LifecycleError::InvalidTransition(
                "idle worker failure requires a ready owned worker",
            ));
        }
        let generation = checked_increment(self.model.model_generation, "model generation")?;
        let revision = checked_increment(self.revision, "revision")?;
        self.revision = revision;
        self.model.revision = revision;
        self.model.model_generation = generation;
        self.model.state = ModelState::Stopping;
        self.model.error = Some(cause.clone());
        self.idle_stop = Some(IdleStop {
            winner: StopWinner {
                reason: StopReason::WorkerFailure,
                cause: Some(cause),
            },
            progress_rank: 0,
        });
        Ok(self.model.clone())
    }

    pub fn report_idle_stop_progress(
        &mut self,
        stage: ProgressStage,
    ) -> Result<ModelSnapshot, LifecycleError> {
        let rank = match stage {
            ProgressStage::Terminating => 1,
            ProgressStage::Reaping => 2,
            _ => {
                return Err(LifecycleError::InvalidProgress(
                    "idle stop permits only terminating or reaping",
                ))
            }
        };
        let idle = self
            .idle_stop
            .as_ref()
            .ok_or(LifecycleError::InvalidTransition(
                "no idle worker stop is active",
            ))?;
        if rank < idle.progress_rank {
            return Err(LifecycleError::InvalidProgress(
                "idle stop progress moved backwards",
            ));
        }
        let revision = checked_increment(self.revision, "revision")?;
        self.revision = revision;
        self.model.revision = revision;
        self.model.progress = progress(stage);
        self.idle_stop
            .as_mut()
            .ok_or(LifecycleError::InvalidTransition(
                "idle stop disappeared during progress",
            ))?
            .progress_rank = rank;
        Ok(self.model.clone())
    }

    pub fn note_idle_termination_unconfirmed(&mut self) -> Result<ModelSnapshot, LifecycleError> {
        if self.idle_stop.is_none() {
            return Err(LifecycleError::InvalidTransition(
                "no idle worker stop is active",
            ));
        }
        let revision = checked_increment(self.revision, "revision")?;
        self.revision = revision;
        self.model.revision = revision;
        self.model.error = Some(termination_unconfirmed_error());
        Ok(self.model.clone())
    }

    pub fn confirm_idle_worker_reaped(&mut self) -> Result<ModelSnapshot, LifecycleError> {
        if self.current_worker_generation.is_none() {
            return Err(LifecycleError::WorkerNotSpawned);
        }
        let stop = self
            .idle_stop
            .as_ref()
            .ok_or(LifecycleError::InvalidTransition(
                "no idle worker stop is active",
            ))?;
        if stop.winner.reason != StopReason::WorkerFailure {
            return Err(LifecycleError::InvalidTransition(
                "idle stop has a non-failure winner",
            ));
        }
        let cause = stop
            .winner
            .cause
            .clone()
            .ok_or(LifecycleError::InvalidTransition(
                "idle worker failure lost its cause",
            ))?;
        let revision = checked_increment(self.revision, "revision")?;
        self.idle_stop = None;
        self.revision = revision;
        self.model.revision = revision;
        self.model.state = ModelState::Error;
        self.model.progress = progress(ProgressStage::Error);
        self.model.error = Some(cause);
        self.clear_installed_worker();
        Ok(self.model.clone())
    }

    fn clear_installed_worker(&mut self) {
        self.current_worker_generation = None;
        self.model.verified_artifact = None;
        self.model.qualification_receipt_sha256 = None;
    }

    fn retain_terminal(&mut self, job: JobSnapshot) {
        if self.terminal.len() == TERMINAL_JOB_LIMIT {
            self.terminal.pop_front();
        }
        self.terminal.push_back(job);
    }

    fn require_free_slot(&self) -> Result<(), LifecycleError> {
        if self.active.is_some()
            || self.idle_stop.is_some()
            || self.model.state == ModelState::Stopping
        {
            Err(LifecycleError::Busy)
        } else {
            Ok(())
        }
    }

    fn require_active_any(&self, job_id: &str) -> Result<&ActiveJob, LifecycleError> {
        if let Some(active) = self
            .active
            .as_ref()
            .filter(|active| active.snapshot.job_id == job_id)
        {
            return Ok(active);
        }
        if self.terminal.iter().any(|job| job.job_id == job_id) {
            Err(LifecycleError::AlreadyTerminal)
        } else {
            Err(LifecycleError::JobNotFound)
        }
    }

    fn require_active(&self, job_id: &str, kind: JobKind) -> Result<&ActiveJob, LifecycleError> {
        let active = self.require_active_any(job_id)?;
        if active.snapshot.kind != kind {
            return Err(LifecycleError::InvalidTransition(
                "event does not apply to this job kind",
            ));
        }
        Ok(active)
    }
}

fn validate_common_initial(
    instance_id: &str,
    host: &HostIdentity,
    artifact: &ArtifactIdentity,
) -> Result<(), LifecycleError> {
    if !is_hex(instance_id, 32) {
        return Err(LifecycleError::InvalidInitialState(
            "instance ID must be lowercase hex32",
        ));
    }
    host.validate()
        .and_then(|_| artifact.validate())
        .map_err(|error| LifecycleError::InvalidWireValue(error.to_string()))
}

fn progress(stage: ProgressStage) -> Progress {
    Progress {
        stage,
        completed: None,
        total: None,
        unit: None,
        fraction: None,
        eta_ms: None,
    }
}

fn working_state(kind: JobKind) -> JobState {
    match kind {
        JobKind::Load => JobState::Loading,
        JobKind::Answer => JobState::Running,
        JobKind::Unload => JobState::Unloading,
    }
}

fn checked_increment(value: u64, name: &'static str) -> Result<u64, LifecycleError> {
    if value >= UINT53_MAX {
        Err(LifecycleError::CounterExhausted(name))
    } else {
        Ok(value + 1)
    }
}

fn validate_elapsed(value: Option<u64>) -> Result<(), LifecycleError> {
    if value.is_some_and(|elapsed| elapsed > UINT53_MAX) {
        Err(LifecycleError::InvalidWireValue(
            "elapsed milliseconds exceed uint53".to_owned(),
        ))
    } else {
        Ok(())
    }
}

fn validate_work_progress(
    kind: JobKind,
    previous: &Progress,
    next: &Progress,
) -> Result<(), LifecycleError> {
    let previous_rank = work_progress_rank(kind, previous.stage).ok_or(
        LifecycleError::InvalidProgress("existing stage is invalid for job kind"),
    )?;
    let next_rank = work_progress_rank(kind, next.stage).ok_or(LifecycleError::InvalidProgress(
        "stage is invalid for job kind",
    ))?;
    if next_rank < previous_rank {
        return Err(LifecycleError::InvalidProgress("stage moved backwards"));
    }
    if next.stage == ProgressStage::ReadingArtifact
        && previous.stage == ProgressStage::ReadingArtifact
        && next.completed < previous.completed
    {
        return Err(LifecycleError::InvalidProgress(
            "artifact byte count moved backwards",
        ));
    }
    Ok(())
}

fn work_progress_rank(kind: JobKind, stage: ProgressStage) -> Option<u8> {
    match (kind, stage) {
        (_, ProgressStage::Idle) => Some(0),
        (JobKind::Load, ProgressStage::ReadingArtifact) => Some(1),
        (JobKind::Load, ProgressStage::Validating) => Some(2),
        (JobKind::Load, ProgressStage::Qualifying) => Some(3),
        (JobKind::Answer, ProgressStage::Inference) => Some(1),
        (JobKind::Unload, ProgressStage::Terminating) => Some(1),
        (JobKind::Unload, ProgressStage::Reaping) => Some(2),
        (_, ProgressStage::Complete) => Some(4),
        _ => None,
    }
}

fn validate_result_identity(
    job: &JobSnapshot,
    model: &ModelSnapshot,
    result: &NativeTextResult,
) -> Result<(), LifecycleError> {
    if model.verified_artifact.as_ref() != Some(&job.artifact) {
        return Err(LifecycleError::ResultIdentityMismatch(
            "current verified artifact",
        ));
    }
    if let NativeTextResult::Model(token) = result {
        if job.raw_text_sha256.as_deref() != Some(token.raw_text_sha256.as_str()) {
            return Err(LifecycleError::ResultIdentityMismatch("raw input digest"));
        }
        for (matches, axis) in [
            (
                token.policy_sha256 == job.artifact.policy_sha256,
                "policy digest",
            ),
            (
                token.reader_file_cid == job.artifact.reader_file_cid,
                "reader file CID",
            ),
            (
                token.core_file_cid == job.artifact.core_file_cid,
                "core file CID",
            ),
            (
                token.frame_tree_cid == job.artifact.frame_tree_cid,
                "frame tree CID",
            ),
        ] {
            if !matches {
                return Err(LifecycleError::ResultIdentityMismatch(axis));
            }
        }
    }
    Ok(())
}

fn deadline_error() -> ServiceError {
    ServiceError {
        tag: ServiceErrorTag::DeadlineExceeded,
        message: "The admitted job exceeded its fixed deadline.".to_owned(),
        native: None,
    }
}

fn termination_unconfirmed_error() -> ServiceError {
    ServiceError {
        tag: ServiceErrorTag::TerminationUnconfirmed,
        message: "Owned worker termination could not be confirmed.".to_owned(),
        native: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{
        ModelToken, WorkerReady, ARTIFACT_BYTES, ARTIFACT_SHA256, CONFIGURED_MODEL_ID,
        FIRST_TARGET, NATIVE_STATE_SHA256, OPERATOR_PROFILE, ORIGINAL_EXPORT_RELEASE_SHA256,
    };

    const INSTANCE: &str = "00000000000000000000000000000001";

    fn host() -> HostIdentity {
        HostIdentity {
            native_binary_sha256: "1".repeat(64),
            runtime_receipt_sha256: Some("2".repeat(64)),
            target: FIRST_TARGET.to_owned(),
            operator_profile: OPERATOR_PROFILE.to_owned(),
            service_contract_sha256: "3".repeat(64),
            asset_manifest_sha256: "4".repeat(64),
            host_acceptance_sha256: Some("5".repeat(64)),
            qualification_receipt_sha256: Some("6".repeat(64)),
        }
    }

    fn artifact() -> ArtifactIdentity {
        ArtifactIdentity {
            model_id: CONFIGURED_MODEL_ID.to_owned(),
            artifact_sha256: ARTIFACT_SHA256.to_owned(),
            artifact_bytes: ARTIFACT_BYTES,
            native_state_sha256: NATIVE_STATE_SHA256.to_owned(),
            codec_cid: format!("blake3:{}", "7".repeat(64)),
            policy_sha256: "8".repeat(64),
            reader_file_cid: format!("blake3:{}", "9".repeat(64)),
            core_file_cid: format!("blake3:{}", "a".repeat(64)),
            frame_tree_cid: format!("blake3:{}", "b".repeat(64)),
            original_export_release_sha256: ORIGINAL_EXPORT_RELEASE_SHA256.to_owned(),
        }
    }

    fn lifecycle() -> Lifecycle {
        Lifecycle::new_unloaded(INSTANCE.to_owned(), host(), artifact()).expect("lifecycle")
    }

    fn worker_error() -> ServiceError {
        ServiceError {
            tag: ServiceErrorTag::WorkerFailure,
            message: "Synthetic worker exited without its required reply.".to_owned(),
            native: None,
        }
    }

    fn ready() -> WorkerReady {
        WorkerReady {
            host: host(),
            artifact: artifact(),
        }
    }

    fn load_ready(state: &mut Lifecycle) -> JobSnapshot {
        let load = state.admit_load().expect("admit load");
        state
            .note_load_worker_spawned(&load.job_id)
            .expect("spawn worker");
        state.mark_dispatched(&load.job_id).expect("dispatch load");
        state
            .accept_worker_ready(&load.job_id, ready(), Some(1))
            .expect("ready")
    }

    fn model_token(raw_text_sha256: &str) -> NativeTextResult {
        let artifact = artifact();
        NativeTextResult::Model(ModelToken {
            schema: "uor-r4.text-binding-result/1".to_owned(),
            status: "MODEL_TOKEN".to_owned(),
            policy_sha256: artifact.policy_sha256,
            raw_text_sha256: raw_text_sha256.to_owned(),
            derived_input_sha256: "c".repeat(64),
            reader_file_cid: artifact.reader_file_cid,
            core_file_cid: artifact.core_file_cid,
            frame_tree_cid: artifact.frame_tree_cid,
            token_id: 1,
            token: "synthetic".to_owned(),
        })
    }

    #[test]
    fn one_slot_has_no_queue_and_successful_readiness_advances_generation_once() {
        let mut state = lifecycle();
        let load = state.admit_load().expect("load admitted");
        assert_eq!(state.admit_load(), Err(LifecycleError::Busy));
        assert_eq!(state.model_snapshot().model_generation, 0);
        assert_eq!(state.note_load_worker_spawned(&load.job_id).unwrap(), 1);
        assert_eq!(state.model_snapshot().model_generation, 0);
        state.mark_dispatched(&load.job_id).unwrap();
        let completed = state
            .accept_worker_ready(&load.job_id, ready(), Some(7))
            .unwrap();
        assert_eq!(completed.state, JobState::Completed);
        assert_eq!(completed.admitted_generation, 0);
        assert_eq!(state.model_snapshot().model_generation, 1);
        assert_eq!(state.model_snapshot().state, ModelState::Ready);
        assert_eq!(state.current_worker_generation(), Some(1));
    }

    #[test]
    fn answer_result_validates_all_published_identity_axes() {
        let mut state = lifecycle();
        load_ready(&mut state);
        let digest = "d".repeat(64);
        let answer = state.admit_answer(1, digest.clone()).unwrap();
        state.mark_dispatched(&answer.job_id).unwrap();
        let before = state.active_job().unwrap();
        let mut mismatched = model_token(&digest);
        let NativeTextResult::Model(token) = &mut mismatched else {
            unreachable!()
        };
        token.core_file_cid = format!("blake3:{}", "e".repeat(64));
        assert_eq!(
            state.accept_answer_result(&answer.job_id, mismatched, Some(1)),
            Err(LifecycleError::ResultIdentityMismatch("core file CID"))
        );
        assert_eq!(state.active_job(), Some(before));

        let completed = state
            .accept_answer_result(&answer.job_id, model_token(&digest), Some(2))
            .unwrap();
        assert_eq!(completed.work.forward_count, Some(1));
        assert_eq!(completed.result, Some(model_token(&digest)));
        assert_eq!(state.model_snapshot().model_generation, 1);
    }

    #[test]
    fn spawned_cancel_stays_stopping_until_confirmed_reap() {
        let mut state = lifecycle();
        load_ready(&mut state);
        let answer = state.admit_answer(1, "d".repeat(64)).unwrap();
        state.mark_dispatched(&answer.job_id).unwrap();
        let stopping = state.request_cancel(&answer.job_id).unwrap();
        assert_eq!(stopping.state, JobState::Stopping);
        assert_eq!(stopping.stop_reason, Some(StopReason::UserCancel));
        assert_eq!(state.model_snapshot().model_generation, 2);
        assert_eq!(state.request_cancel(&answer.job_id).unwrap(), stopping);

        let uncertain = state.note_termination_unconfirmed(&answer.job_id).unwrap();
        assert_eq!(
            uncertain.error.as_ref().map(|error| error.tag),
            Some(ServiceErrorTag::TerminationUnconfirmed)
        );
        assert_eq!(state.admit_load(), Err(LifecycleError::Busy));

        let cancelled = state
            .confirm_reaped(&answer.job_id, worker_error())
            .unwrap();
        assert_eq!(cancelled.state, JobState::Cancelled);
        assert_eq!(cancelled.error, None);
        assert_eq!(cancelled.work.forward_count, None);
        assert_eq!(state.model_snapshot().state, ModelState::Unloaded);
        assert_eq!(state.model_snapshot().model_generation, 2);
        assert_eq!(state.current_worker_generation(), None);
    }

    #[test]
    fn pre_spawn_load_stop_is_direct_and_does_not_advance_generation() {
        let mut state = lifecycle();
        let load = state.admit_load().unwrap();
        let cancelled = state.request_cancel(&load.job_id).unwrap();
        assert_eq!(cancelled.state, JobState::Cancelled);
        assert_eq!(cancelled.work.forward_count, Some(0));
        assert_eq!(state.model_snapshot().model_generation, 0);
        assert_eq!(state.model_snapshot().state, ModelState::Unloaded);

        let load = state.admit_load().unwrap();
        let failed = state.deadline(&load.job_id).unwrap();
        assert_eq!(failed.state, JobState::Failed);
        assert_eq!(
            failed.error.as_ref().map(|error| error.tag),
            Some(ServiceErrorTag::DeadlineExceeded)
        );
        assert_eq!(state.model_snapshot().model_generation, 0);
    }

    #[test]
    fn unresolved_launcher_keeps_cancel_nonterminal_until_no_child_is_proved() {
        let mut state = lifecycle();
        let load = state.admit_load().unwrap();
        state.note_load_launch_pending(&load.job_id).unwrap();
        let stopping = state.request_cancel(&load.job_id).unwrap();
        assert_eq!(stopping.state, JobState::Stopping);
        assert_eq!(stopping.stop_reason, Some(StopReason::UserCancel));
        assert_eq!(state.model_snapshot().model_generation, 0);
        assert_eq!(state.admit_load(), Err(LifecycleError::Busy));

        let cancelled = state
            .finish_load_launch_without_child(&load.job_id, worker_error())
            .unwrap();
        assert_eq!(cancelled.state, JobState::Cancelled);
        assert_eq!(cancelled.error, None);
        assert_eq!(state.model_snapshot().state, ModelState::Unloaded);
        assert_eq!(state.model_snapshot().model_generation, 0);
    }

    #[test]
    fn child_returned_after_launch_deadline_is_owned_invalidated_and_reaped() {
        let mut state = lifecycle();
        let load = state.admit_load().unwrap();
        state.note_load_launch_pending(&load.job_id).unwrap();
        let stopping = state.deadline(&load.job_id).unwrap();
        assert_eq!(stopping.state, JobState::Stopping);
        assert_eq!(state.model_snapshot().model_generation, 0);

        let (worker_generation, ()) = state
            .spawn_load_worker(&load.job_id, || Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(worker_generation, 1);
        assert_eq!(state.current_worker_generation(), Some(1));
        assert_eq!(state.model_snapshot().model_generation, 1);

        let failed = state.confirm_reaped(&load.job_id, worker_error()).unwrap();
        assert_eq!(failed.state, JobState::Failed);
        assert_eq!(failed.stop_reason, Some(StopReason::Deadline));
        assert_eq!(
            failed.error.as_ref().map(|error| error.tag),
            Some(ServiceErrorTag::DeadlineExceeded)
        );
        assert_eq!(state.model_snapshot().model_generation, 1);
    }

    #[test]
    fn unload_requires_both_acknowledgment_and_reap() {
        let mut state = lifecycle();
        load_ready(&mut state);
        let unload = state.admit_unload(1).unwrap();
        assert_eq!(state.model_snapshot().model_generation, 2);
        state.mark_dispatched(&unload.job_id).unwrap();
        state.acknowledge_unloaded(&unload.job_id).unwrap();
        assert_eq!(state.active_job().unwrap().state, JobState::Unloading);
        let completed = state
            .confirm_reaped(&unload.job_id, worker_error())
            .unwrap();
        assert_eq!(completed.state, JobState::Completed);
        assert_eq!(state.model_snapshot().state, ModelState::Unloaded);
        assert_eq!(state.model_snapshot().model_generation, 2);

        load_ready(&mut state);
        let unload = state.admit_unload(3).unwrap();
        state.mark_dispatched(&unload.job_id).unwrap();
        let failed = state
            .confirm_reaped(&unload.job_id, worker_error())
            .unwrap();
        assert_eq!(failed.state, JobState::Failed);
        assert_eq!(failed.stop_reason, Some(StopReason::WorkerFailure));
        assert_eq!(state.model_snapshot().state, ModelState::Error);
    }

    #[test]
    fn unload_is_not_cancellable_and_deadline_does_not_double_invalidate_generation() {
        let mut state = lifecycle();
        load_ready(&mut state);
        let unload = state.admit_unload(1).unwrap();
        assert_eq!(state.model_snapshot().model_generation, 2);
        assert_eq!(
            state.request_cancel(&unload.job_id),
            Err(LifecycleError::NotCancellable)
        );
        state.mark_dispatched(&unload.job_id).unwrap();
        let stopping = state.deadline(&unload.job_id).unwrap();
        assert_eq!(stopping.stop_reason, Some(StopReason::Deadline));
        assert_eq!(state.model_snapshot().model_generation, 2);
        let failed = state
            .confirm_reaped(&unload.job_id, worker_error())
            .unwrap();
        assert_eq!(failed.state, JobState::Failed);
        assert_eq!(state.model_snapshot().model_generation, 2);
    }

    #[test]
    fn spawned_load_failure_advances_each_counter_on_its_distinct_event() {
        let mut state = lifecycle();
        let first = state.admit_load().unwrap();
        assert_eq!(first.job_id, "1");
        assert_eq!(first.revision, 1);
        assert_eq!(state.note_load_worker_spawned(&first.job_id).unwrap(), 1);
        state.mark_dispatched(&first.job_id).unwrap();
        state.worker_failure(&first.job_id, worker_error()).unwrap();
        assert_eq!(state.model_snapshot().model_generation, 1);
        state.confirm_reaped(&first.job_id, worker_error()).unwrap();

        let second = state.admit_load().unwrap();
        assert_eq!(second.job_id, "2");
        assert!(second.revision > first.revision);
        assert_eq!(state.note_load_worker_spawned(&second.job_id).unwrap(), 2);
        assert_eq!(state.model_snapshot().model_generation, 1);
    }

    #[test]
    fn terminal_fifo_retains_only_the_latest_sixty_four_immutable_jobs() {
        let mut state = lifecycle();
        let mut first_job_id = None;
        for _ in 0..33 {
            let load = state.admit_load().unwrap();
            first_job_id.get_or_insert_with(|| load.job_id.clone());
            state.note_load_worker_spawned(&load.job_id).unwrap();
            state.mark_dispatched(&load.job_id).unwrap();
            state
                .accept_worker_ready(&load.job_id, ready(), Some(0))
                .unwrap();
            let generation = state.model_snapshot().model_generation;
            let unload = state.admit_unload(generation).unwrap();
            state.mark_dispatched(&unload.job_id).unwrap();
            state.acknowledge_unloaded(&unload.job_id).unwrap();
            state
                .confirm_reaped(&unload.job_id, worker_error())
                .unwrap();
        }
        assert_eq!(state.terminal_job_count(), TERMINAL_JOB_LIMIT);
        assert_eq!(state.job(first_job_id.as_deref().unwrap()), None);
        let newest = state.model_snapshot().last_job_id.unwrap();
        let frozen = state.job(&newest).unwrap();
        assert_eq!(state.job(&newest), Some(frozen));
    }

    #[test]
    fn first_stop_winner_survives_unconfirmed_termination() {
        let mut state = lifecycle();
        load_ready(&mut state);
        let answer = state.admit_answer(1, "d".repeat(64)).unwrap();
        state.mark_dispatched(&answer.job_id).unwrap();
        state.deadline(&answer.job_id).unwrap();
        state.note_termination_unconfirmed(&answer.job_id).unwrap();
        let failed = state
            .confirm_reaped(&answer.job_id, worker_error())
            .unwrap();
        assert_eq!(failed.stop_reason, Some(StopReason::Deadline));
        assert_eq!(
            failed.error.as_ref().map(|error| error.tag),
            Some(ServiceErrorTag::DeadlineExceeded)
        );
    }

    #[test]
    fn idle_worker_failure_creates_no_job_and_waits_for_reap() {
        let mut state = lifecycle();
        let completed_load = load_ready(&mut state);
        let terminal_count = state.terminal_job_count();
        state.idle_worker_failure(worker_error()).unwrap();
        assert_eq!(state.active_job(), None);
        assert_eq!(state.model_snapshot().state, ModelState::Stopping);
        state.note_idle_termination_unconfirmed().unwrap();
        let model = state.confirm_idle_worker_reaped().unwrap();
        assert_eq!(model.state, ModelState::Error);
        assert_eq!(state.terminal_job_count(), terminal_count);
        assert_eq!(state.job(&completed_load.job_id), Some(completed_load));
    }
}
