//! Bounded exact-matmul execution for the offline teacher.
//!
//! Parallel work is partitioned only across complete output rows. Every output
//! cell still owns its full `k`-term exact accumulator and is rounded once by
//! `uor-matmul`; worker count and completion order therefore cannot change a
//! result bit. Hosted builds use one persistent, dedicated Rayon pool. Wasm
//! keeps the same public configuration and counters but executes sequentially.

#![cfg_attr(
    all(feature = "observation-blas-exception", target_os = "macos"),
    allow(dead_code)
)]

use std::num::NonZeroUsize;
use std::sync::Arc;

/// Git revision of the exact arithmetic dependency bound into probe evidence.
pub const UOR_MATMUL_REVISION: &str = "b13c98449948174f590e337c4dc25dfc394a07d0";

/// A point-in-time view of exact teacher execution.
///
/// Counters are monotonic for one executor. Replacing an oracle's execution
/// configuration installs a fresh executor and resets them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TeacherExecutionSnapshot {
    /// Monotonic sequence assigned to observer callbacks from this executor.
    /// Consumers must ignore a callback older than the newest epoch seen.
    pub observer_epoch: u64,
    /// Worker bound requested after resolving explicit host discovery.
    pub requested_workers: usize,
    /// Dedicated workers actually installed (one on the portable fallback).
    pub effective_workers: usize,
    /// Output-row tile tasks currently inside exact GEMM.
    pub active_workers: usize,
    /// Largest observed simultaneous exact-GEMM task count.
    pub max_active_workers: usize,
    /// Largest simultaneous exact-GEMM task count in the current or most
    /// recently completed physical forward. Reset at every forward boundary.
    pub forward_max_active_workers: usize,
    /// Physical forwards that observed at least two simultaneous row tasks.
    /// This monotonic counter proves genuine multiworker execution without
    /// requiring every configured worker to be scheduled at the same instant.
    pub multiworker_forward_calls: u64,
    /// Single-stream or batched forward invocations begun.
    pub forward_calls: u64,
    /// Independent sequence steps begun across all forward invocations.
    pub streams_started: u64,
    /// Independent sequence steps completed across all forward invocations.
    pub streams_completed: u64,
    /// Independent sequence states currently inside one or more forwards.
    pub active_streams: usize,
    /// Largest independent-stream cohort observed in flight.
    pub max_active_streams: usize,
    /// Exact matrix operations begun.
    pub matrix_calls: u64,
    /// Exact matrix operations executed as one shared-weight multi-stream GEMM.
    pub batched_matrix_calls: u64,
    /// Largest matrix batch width passed to one shared-weight exact GEMM.
    pub max_matrix_batch_width: usize,
    /// Disjoint output-row tiles completed.
    pub tiles_completed: u64,
    /// Exact output cells completed.
    pub output_cells_completed: u64,
    /// Scalar product terms absorbed into complete accumulators.
    pub scalar_terms_completed: u64,
    /// Retained workspace buffers whose capacity grew during this measurement.
    pub workspace_growth_events: u64,
    /// Actual `Vec` capacity bytes added by those workspace growth events.
    pub workspace_growth_bytes: u64,
}

/// Thread-safe progress callback used by long-running teacher harnesses.
///
/// It may be invoked by a dedicated executor worker and should return quickly.
pub type TeacherExecutionObserver = Arc<dyn Fn(TeacherExecutionSnapshot) + Send + Sync + 'static>;

/// Excluded, bounded preparation of retained exact-forward workspaces.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TeacherExecutionPreparation {
    /// Wall time spent waking the pool, reserving real-model workspaces, and
    /// exercising a tiny known exact product.
    pub elapsed_seconds: f64,
    /// Dedicated workers observed through the pool-wide barrier.
    pub workers_observed: usize,
    /// Shared-weight batch width prepared.
    pub batch_width: usize,
    /// Whether the known exact product returned every expected output bit.
    pub backend_exercised: bool,
    /// Total retained model/executor workspace capacity after preparation.
    pub workspace_capacity_bytes: u64,
    /// Retained buffers whose capacity grew during preparation.
    pub workspace_growth_events: u64,
    /// Actual `Vec` capacity bytes added during preparation.
    pub workspace_growth_bytes: u64,
}

/// Nonblocking publication bridge for heartbeat-only execution progress.
///
/// Executor workers never wait for a consumer: one publisher claims the
/// atomic slot, and a concurrent coarse update may be dropped. Forward-end
/// publication and the executor's own final snapshot remain exact. Readers use
/// the version latch to avoid observing a partially published snapshot.
#[cfg(all(test, not(target_arch = "wasm32")))]
#[derive(Default)]
pub(crate) struct AtomicTeacherExecutionProgress {
    writer_claimed: std::sync::atomic::AtomicBool,
    version: std::sync::atomic::AtomicU64,
    observer_epoch: std::sync::atomic::AtomicU64,
    requested_workers: std::sync::atomic::AtomicUsize,
    effective_workers: std::sync::atomic::AtomicUsize,
    active_workers: std::sync::atomic::AtomicUsize,
    max_active_workers: std::sync::atomic::AtomicUsize,
    forward_max_active_workers: std::sync::atomic::AtomicUsize,
    multiworker_forward_calls: std::sync::atomic::AtomicU64,
    forward_calls: std::sync::atomic::AtomicU64,
    streams_started: std::sync::atomic::AtomicU64,
    streams_completed: std::sync::atomic::AtomicU64,
    active_streams: std::sync::atomic::AtomicUsize,
    max_active_streams: std::sync::atomic::AtomicUsize,
    matrix_calls: std::sync::atomic::AtomicU64,
    batched_matrix_calls: std::sync::atomic::AtomicU64,
    max_matrix_batch_width: std::sync::atomic::AtomicUsize,
    tiles_completed: std::sync::atomic::AtomicU64,
    output_cells_completed: std::sync::atomic::AtomicU64,
    scalar_terms_completed: std::sync::atomic::AtomicU64,
    workspace_growth_events: std::sync::atomic::AtomicU64,
    workspace_growth_bytes: std::sync::atomic::AtomicU64,
}

#[cfg(all(test, not(target_arch = "wasm32")))]
impl AtomicTeacherExecutionProgress {
    /// Publish a newer coarse snapshot without blocking an executor worker.
    pub(crate) fn publish(&self, snapshot: TeacherExecutionSnapshot) {
        use std::sync::atomic::Ordering;

        if self
            .writer_claimed
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        if snapshot.observer_epoch > self.observer_epoch.load(Ordering::Acquire) {
            self.version.fetch_add(1, Ordering::AcqRel);
            self.requested_workers
                .store(snapshot.requested_workers, Ordering::Relaxed);
            self.effective_workers
                .store(snapshot.effective_workers, Ordering::Relaxed);
            self.active_workers
                .store(snapshot.active_workers, Ordering::Relaxed);
            self.max_active_workers
                .store(snapshot.max_active_workers, Ordering::Relaxed);
            self.forward_max_active_workers
                .store(snapshot.forward_max_active_workers, Ordering::Relaxed);
            self.multiworker_forward_calls
                .store(snapshot.multiworker_forward_calls, Ordering::Relaxed);
            self.forward_calls
                .store(snapshot.forward_calls, Ordering::Relaxed);
            self.streams_started
                .store(snapshot.streams_started, Ordering::Relaxed);
            self.streams_completed
                .store(snapshot.streams_completed, Ordering::Relaxed);
            self.active_streams
                .store(snapshot.active_streams, Ordering::Relaxed);
            self.max_active_streams
                .store(snapshot.max_active_streams, Ordering::Relaxed);
            self.matrix_calls
                .store(snapshot.matrix_calls, Ordering::Relaxed);
            self.batched_matrix_calls
                .store(snapshot.batched_matrix_calls, Ordering::Relaxed);
            self.max_matrix_batch_width
                .store(snapshot.max_matrix_batch_width, Ordering::Relaxed);
            self.tiles_completed
                .store(snapshot.tiles_completed, Ordering::Relaxed);
            self.output_cells_completed
                .store(snapshot.output_cells_completed, Ordering::Relaxed);
            self.scalar_terms_completed
                .store(snapshot.scalar_terms_completed, Ordering::Relaxed);
            self.workspace_growth_events
                .store(snapshot.workspace_growth_events, Ordering::Relaxed);
            self.workspace_growth_bytes
                .store(snapshot.workspace_growth_bytes, Ordering::Relaxed);
            self.observer_epoch
                .store(snapshot.observer_epoch, Ordering::Relaxed);
            self.version.fetch_add(1, Ordering::Release);
        }
        self.writer_claimed.store(false, Ordering::Release);
    }

    /// Read one internally consistent published snapshot.
    pub(crate) fn snapshot(&self) -> TeacherExecutionSnapshot {
        use std::sync::atomic::Ordering;

        loop {
            let before = self.version.load(Ordering::Acquire);
            if !before.is_multiple_of(2) {
                std::hint::spin_loop();
                continue;
            }
            let snapshot = TeacherExecutionSnapshot {
                observer_epoch: self.observer_epoch.load(Ordering::Relaxed),
                requested_workers: self.requested_workers.load(Ordering::Relaxed),
                effective_workers: self.effective_workers.load(Ordering::Relaxed),
                active_workers: self.active_workers.load(Ordering::Relaxed),
                max_active_workers: self.max_active_workers.load(Ordering::Relaxed),
                forward_max_active_workers: self.forward_max_active_workers.load(Ordering::Relaxed),
                multiworker_forward_calls: self.multiworker_forward_calls.load(Ordering::Relaxed),
                forward_calls: self.forward_calls.load(Ordering::Relaxed),
                streams_started: self.streams_started.load(Ordering::Relaxed),
                streams_completed: self.streams_completed.load(Ordering::Relaxed),
                active_streams: self.active_streams.load(Ordering::Relaxed),
                max_active_streams: self.max_active_streams.load(Ordering::Relaxed),
                matrix_calls: self.matrix_calls.load(Ordering::Relaxed),
                batched_matrix_calls: self.batched_matrix_calls.load(Ordering::Relaxed),
                max_matrix_batch_width: self.max_matrix_batch_width.load(Ordering::Relaxed),
                tiles_completed: self.tiles_completed.load(Ordering::Relaxed),
                output_cells_completed: self.output_cells_completed.load(Ordering::Relaxed),
                scalar_terms_completed: self.scalar_terms_completed.load(Ordering::Relaxed),
                workspace_growth_events: self.workspace_growth_events.load(Ordering::Relaxed),
                workspace_growth_bytes: self.workspace_growth_bytes.load(Ordering::Relaxed),
            };
            let after = self.version.load(Ordering::Acquire);
            if before == after {
                return snapshot;
            }
        }
    }
}

/// Truthful provenance for the exact teacher arithmetic backend.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ExactBackendReport {
    /// Numerical owner used by the default teacher projection path.
    pub arithmetic_owner: String,
    /// Whether uor-matmul was built with hosted runtime CPU detection.
    pub std_runtime_detection_enabled: bool,
    /// Rust compilation target architecture.
    pub target_arch: String,
    /// Rust compilation target operating system.
    pub target_os: String,
    /// Pinned git revision of the exact arithmetic implementation.
    pub uor_matmul_revision: String,
    /// Backends the dependency reports available for the Atlas reduction
    /// family consumed by exact float GEMM, in registry order without repeats.
    pub available_backends: Vec<String>,
    /// Actual private auto-selection, when externally observable.
    ///
    /// The pinned dependency does not expose its float auto-selection cache,
    /// so this is currently `None`; callers must not infer it from availability.
    pub selected_backend: Option<String>,
    /// Machine-readable reason for [`ExactBackendReport::selected_backend`].
    pub selection_status: String,
}

/// Report arithmetic ownership for a selected source-backed forward mode.
pub(crate) fn backend_report_for_fast_matmul(fast_matmul: bool) -> ExactBackendReport {
    let mut available_backends = Vec::new();
    for spec in uor_matmul::kernels::cached::available_reduce_i8().filter(|spec| spec.k_group == 1)
    {
        let name = spec.backend.as_str().to_owned();
        if !available_backends.contains(&name) {
            available_backends.push(name);
        }
    }
    #[cfg(all(feature = "observation-blas-exception", target_os = "macos"))]
    let (arithmetic_owner, selected_backend, selection_status) = if fast_matmul {
        (
            "Apple Accelerate CPU BLAS".to_owned(),
            Some("Apple Accelerate".to_owned()),
            "AVAILABLE: explicitly selected by observation-blas-exception".to_owned(),
        )
    } else {
        (
            "uor-matmul exact GEMM".to_owned(),
            None,
            "AVAILABLE_BUT_DISABLED: exact/canonical runtime selected".to_owned(),
        )
    };
    #[cfg(not(all(feature = "observation-blas-exception", target_os = "macos")))]
    let (arithmetic_owner, selected_backend, selection_status) = {
        let _ = fast_matmul;
        (
            "uor-matmul exact GEMM".to_owned(),
            None,
            "UNAVAILABLE: uor-matmul does not expose the private float auto-selection cache"
                .to_owned(),
        )
    };

    ExactBackendReport {
        arithmetic_owner,
        std_runtime_detection_enabled: !cfg!(target_arch = "wasm32"),
        target_arch: std::env::consts::ARCH.to_owned(),
        target_os: std::env::consts::OS.to_owned(),
        uor_matmul_revision: UOR_MATMUL_REVISION.to_owned(),
        available_backends,
        selected_backend,
        selection_status,
    }
}

/// Report the arithmetic owner selected by the current process environment.
///
/// The historical function name is retained for API compatibility. A macOS
/// build with the Accelerate feature reports BLAS in normal fast mode, while
/// either exact override reports the exact owner that will actually execute.
pub fn exact_backend_report() -> ExactBackendReport {
    #[cfg(all(feature = "observation-blas-exception", target_os = "macos"))]
    let fast_matmul = {
        let canonical_math =
            std::env::var("TLESS_CANONICAL_DETERMINISTIC").is_ok_and(|value| value != "0");
        !canonical_math && std::env::var("TLESS_EXACT_SCALAR").is_err()
    };
    #[cfg(not(all(feature = "observation-blas-exception", target_os = "macos")))]
    let fast_matmul = false;

    backend_report_for_fast_matmul(fast_matmul)
}

#[derive(Clone, Copy)]
enum WorkerRequest {
    Fixed(NonZeroUsize),
    AvailableParallelism,
}

/// Bounded execution policy for the exact offline teacher.
///
/// [`Default`] and [`TeacherExecutionConfig::sequential`] never inspect the
/// host and install one worker. Host-core discovery occurs only when
/// [`TeacherExecutionConfig::available_parallelism`] is explicitly selected.
#[derive(Clone)]
pub struct TeacherExecutionConfig {
    workers: WorkerRequest,
    tiles_per_worker: NonZeroUsize,
    observer: Option<TeacherExecutionObserver>,
}

impl Default for TeacherExecutionConfig {
    fn default() -> Self {
        Self::sequential()
    }
}

impl TeacherExecutionConfig {
    /// A one-worker deterministic executor without host-core discovery.
    pub fn sequential() -> Self {
        Self {
            workers: WorkerRequest::Fixed(NonZeroUsize::MIN),
            tiles_per_worker: NonZeroUsize::new(4).expect("four is nonzero"),
            observer: None,
        }
    }

    /// A fixed, nonzero dedicated worker bound.
    pub fn fixed_workers(workers: NonZeroUsize) -> Self {
        Self {
            workers: WorkerRequest::Fixed(workers),
            ..Self::sequential()
        }
    }

    /// Size the dedicated pool from `std::thread::available_parallelism()`.
    ///
    /// This is the sole policy that discovers host capacity. A failed query
    /// conservatively resolves to one worker.
    pub fn available_parallelism() -> Self {
        Self {
            workers: WorkerRequest::AvailableParallelism,
            ..Self::sequential()
        }
    }

    /// Bound scheduler granularity per worker (default: four row tiles).
    pub fn with_tiles_per_worker(mut self, tiles_per_worker: NonZeroUsize) -> Self {
        self.tiles_per_worker = tiles_per_worker;
        self
    }

    /// Observe forward boundaries, concurrency high-water marks, and coarse
    /// output-row progress. Exact per-tile totals remain in [`TeacherExecutionSnapshot`].
    pub fn with_observer(mut self, observer: TeacherExecutionObserver) -> Self {
        self.observer = Some(observer);
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn resolved_workers(&self) -> usize {
        match self.workers {
            WorkerRequest::Fixed(workers) => workers.get(),
            WorkerRequest::AvailableParallelism => {
                std::thread::available_parallelism().map_or(1, NonZeroUsize::get)
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn resolved_workers(&self) -> usize {
        // No threaded exact executor is installed for this target. Preserve
        // the configuration API while reporting the truthful effective bound.
        match self.workers {
            WorkerRequest::Fixed(requested) => {
                let _ = requested;
                1
            }
            WorkerRequest::AvailableParallelism => 1,
        }
    }
}

pub(crate) fn exact_tile_rows_for(
    rows: usize,
    effective_workers: usize,
    tiles_per_worker: usize,
) -> usize {
    if effective_workers <= 1 {
        return rows.max(1);
    }
    let target_tiles = effective_workers.saturating_mul(tiles_per_worker).max(1);
    rows.div_ceil(target_tiles).max(1)
}

pub(crate) fn exact_row_tiles_for(
    rows: usize,
    effective_workers: usize,
    tiles_per_worker: usize,
) -> usize {
    if rows == 0 {
        0
    } else {
        rows.div_ceil(exact_tile_rows_for(
            rows,
            effective_workers,
            tiles_per_worker,
        ))
    }
}

/// Portable shared-weight batched exact product used by the sequential wasm
/// executor and its host-side bit-identity test.
#[cfg(any(test, target_arch = "wasm32"))]
fn portable_shared_weight_matmul_batched(
    output: &mut [f32],
    x: &[f32],
    w: &[f32],
    k: usize,
    batch: usize,
) -> usize {
    assert!(batch > 0, "teacher batch must be nonzero");
    assert_eq!(output.len() % batch, 0);
    let rows = output.len() / batch;
    assert_eq!(x.len(), batch.saturating_mul(k));
    assert!(w.len() >= rows.saturating_mul(k));
    if rows == 0 {
        return 0;
    }

    let input_words = k
        .checked_mul(batch)
        .expect("portable exact input shape must fit usize");
    let output_words = rows
        .checked_mul(batch)
        .expect("portable exact output shape must fit usize");
    let mut x_transposed = vec![0.0f32; input_words];
    let mut output_transposed = vec![0.0f32; output_words];
    for stream in 0..batch {
        for depth in 0..k {
            x_transposed[depth * batch + stream] = x[stream * k + depth];
        }
    }

    let mut pa = vec![uor_matmul::PackedCode::default(); k];
    let mut pb = vec![uor_matmul::PackedCode::default(); input_words];
    uor_matmul::slice::gemm_float(
        rows,
        k,
        batch,
        &w[..rows * k],
        &x_transposed,
        &mut output_transposed,
        &mut pa,
        &mut pb,
    )
    .expect("portable exact shared-weight batch product is valid");

    for row in 0..rows {
        for stream in 0..batch {
            output[stream * rows + row] = output_transposed[row * batch + stream];
        }
    }
    rows
}

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    use super::{
        TeacherExecutionConfig, TeacherExecutionObserver, TeacherExecutionPreparation,
        TeacherExecutionSnapshot,
    };
    use crate::SourceUnavailable;
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    use std::sync::Mutex;
    use uor_matmul::{PackedCode, Partition, Shape};

    #[derive(Default)]
    struct Counters {
        observer_epoch: AtomicU64,
        active_workers: AtomicUsize,
        max_active_workers: AtomicUsize,
        forward_max_active_workers: AtomicUsize,
        multiworker_forward_calls: AtomicU64,
        multiworker_observed_forward: AtomicU64,
        forward_calls: AtomicU64,
        streams_started: AtomicU64,
        streams_completed: AtomicU64,
        active_streams: AtomicUsize,
        max_active_streams: AtomicUsize,
        matrix_calls: AtomicU64,
        batched_matrix_calls: AtomicU64,
        max_matrix_batch_width: AtomicUsize,
        tiles_completed: AtomicU64,
        output_cells_completed: AtomicU64,
        scalar_terms_completed: AtomicU64,
        workspace_growth_events: AtomicU64,
        workspace_growth_bytes: AtomicU64,
    }

    /// Persistent owner of the dedicated exact-teacher worker pool.
    pub(crate) struct ExactExecutor {
        requested_workers: usize,
        effective_workers: usize,
        tiles_per_worker: usize,
        pool: Option<rayon::ThreadPool>,
        counters: Counters,
        observer: Option<TeacherExecutionObserver>,
        batch_workspace: Mutex<Box<ExactBatchWorkspace>>,
        worker_scratch: Vec<Mutex<ExactScratch>>,
    }

    #[derive(Default)]
    struct ExactScratch {
        pa: Vec<PackedCode>,
        pb: Vec<PackedCode>,
    }

    #[derive(Default)]
    struct ExactBatchWorkspace {
        x_transposed: Vec<f32>,
        output_transposed: Vec<f32>,
    }

    struct ActiveTask<'a> {
        executor: &'a ExactExecutor,
    }

    impl Drop for ActiveTask<'_> {
        fn drop(&mut self) {
            self.executor
                .counters
                .active_workers
                .fetch_sub(1, Ordering::AcqRel);
        }
    }

    impl ExactExecutor {
        pub(crate) fn new(config: TeacherExecutionConfig) -> Result<Self, SourceUnavailable> {
            let workers = config.resolved_workers();
            let pool = if workers == 1 {
                None
            } else {
                Some(
                    rayon::ThreadPoolBuilder::new()
                        .num_threads(workers)
                        .thread_name(|index| format!("r4-exact-teacher-{index}"))
                        .build()
                        .map_err(|error| SourceUnavailable::new(error.to_string()))?,
                )
            };
            Ok(Self {
                requested_workers: workers,
                effective_workers: workers,
                tiles_per_worker: config.tiles_per_worker.get(),
                pool,
                counters: Counters::default(),
                observer: config.observer,
                batch_workspace: Mutex::new(Box::new(ExactBatchWorkspace::default())),
                worker_scratch: (0..workers)
                    .map(|_| Mutex::new(ExactScratch::default()))
                    .collect(),
            })
        }

        pub(crate) fn snapshot(&self) -> TeacherExecutionSnapshot {
            TeacherExecutionSnapshot {
                observer_epoch: self.counters.observer_epoch.load(Ordering::Acquire),
                requested_workers: self.requested_workers,
                effective_workers: self.effective_workers,
                active_workers: self.counters.active_workers.load(Ordering::Acquire),
                max_active_workers: self.counters.max_active_workers.load(Ordering::Acquire),
                forward_max_active_workers: self
                    .counters
                    .forward_max_active_workers
                    .load(Ordering::Acquire),
                multiworker_forward_calls: self
                    .counters
                    .multiworker_forward_calls
                    .load(Ordering::Acquire),
                forward_calls: self.counters.forward_calls.load(Ordering::Acquire),
                streams_started: self.counters.streams_started.load(Ordering::Acquire),
                streams_completed: self.counters.streams_completed.load(Ordering::Acquire),
                active_streams: self.counters.active_streams.load(Ordering::Acquire),
                max_active_streams: self.counters.max_active_streams.load(Ordering::Acquire),
                matrix_calls: self.counters.matrix_calls.load(Ordering::Acquire),
                batched_matrix_calls: self.counters.batched_matrix_calls.load(Ordering::Acquire),
                max_matrix_batch_width: self
                    .counters
                    .max_matrix_batch_width
                    .load(Ordering::Acquire),
                tiles_completed: self.counters.tiles_completed.load(Ordering::Acquire),
                output_cells_completed: self
                    .counters
                    .output_cells_completed
                    .load(Ordering::Acquire),
                scalar_terms_completed: self
                    .counters
                    .scalar_terms_completed
                    .load(Ordering::Acquire),
                workspace_growth_events: self
                    .counters
                    .workspace_growth_events
                    .load(Ordering::Acquire),
                workspace_growth_bytes: self
                    .counters
                    .workspace_growth_bytes
                    .load(Ordering::Acquire),
            }
        }

        pub(crate) fn begin_measured_execution(&mut self, observer: TeacherExecutionObserver) {
            self.counters = Counters::default();
            self.observer = Some(observer);
        }

        /// Account one retained workspace capacity increase. The byte count is
        /// the actual `Vec::capacity` delta, not the requested logical length.
        pub(crate) fn record_workspace_growth_bytes(&self, bytes: usize) {
            if bytes == 0 {
                return;
            }
            saturating_add(&self.counters.workspace_growth_events, 1);
            saturating_add(&self.counters.workspace_growth_bytes, as_u64(bytes));
        }

        fn grow_vec<T: Clone>(&self, values: &mut Vec<T>, length: usize, fill: T) {
            if values.len() >= length {
                return;
            }
            let before = values.capacity();
            values.resize(length, fill);
            let added_elements = values.capacity().saturating_sub(before);
            self.record_workspace_growth_bytes(
                added_elements.saturating_mul(std::mem::size_of::<T>()),
            );
        }

        fn ensure_scratch(&self, scratch: &mut ExactScratch, k: usize, columns: usize) {
            let packed_columns = k
                .checked_mul(columns)
                .expect("exact scratch shape must fit usize");
            self.grow_vec(&mut scratch.pa, k, PackedCode::default());
            self.grow_vec(&mut scratch.pb, packed_columns, PackedCode::default());
        }

        pub(crate) fn prepare_workspace(&self, k: usize, rows: usize, batch: usize) {
            let transposed_input = k
                .checked_mul(batch)
                .expect("exact input workspace shape must fit usize");
            let transposed_output = rows
                .checked_mul(batch)
                .expect("exact output workspace shape must fit usize");
            {
                let mut workspace = self
                    .batch_workspace
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                self.grow_vec(&mut workspace.x_transposed, transposed_input, 0.0f32);
                self.grow_vec(&mut workspace.output_transposed, transposed_output, 0.0f32);
            }
            for scratch in &self.worker_scratch {
                let mut scratch = scratch
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                self.ensure_scratch(&mut scratch, k, batch);
            }
        }

        fn workspace_capacity_bytes(&self) -> usize {
            let workspace = self
                .batch_workspace
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut bytes = workspace
                .x_transposed
                .capacity()
                .saturating_add(workspace.output_transposed.capacity())
                .saturating_mul(std::mem::size_of::<f32>());
            drop(workspace);
            for scratch in &self.worker_scratch {
                let scratch = scratch
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                bytes = bytes.saturating_add(
                    scratch
                        .pa
                        .capacity()
                        .saturating_add(scratch.pb.capacity())
                        .saturating_mul(std::mem::size_of::<PackedCode>()),
                );
            }
            bytes
        }

        fn current_worker_scratch(&self) -> std::sync::MutexGuard<'_, ExactScratch> {
            let index = rayon::current_thread_index()
                .unwrap_or(0)
                .min(self.worker_scratch.len().saturating_sub(1));
            self.worker_scratch[index]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        }

        /// Wake every dedicated worker and exercise the exact GEMM dispatcher
        /// without running a model forward.
        ///
        /// The tiny product has one-hot weight rows and exactly representable
        /// inputs, so its expected bits are known independently of worker
        /// ordering. Callers reset measurement counters after this excluded
        /// prestart.
        pub(crate) fn prestart(
            &self,
            batch_width: usize,
            maximum_k: usize,
            maximum_rows: usize,
        ) -> Result<TeacherExecutionPreparation, SourceUnavailable> {
            if batch_width == 0 {
                return Err(SourceUnavailable::new(
                    "exact executor prestart batch width must be nonzero",
                ));
            }
            let started = std::time::Instant::now();
            let workers_observed = if let Some(pool) = &self.pool {
                let mut indices = pool.broadcast(|context| context.index());
                indices.sort_unstable();
                indices.dedup();
                indices.len()
            } else {
                1
            };
            if workers_observed != self.effective_workers {
                return Err(SourceUnavailable::new(format!(
                    "exact executor prestart observed {workers_observed} of {} workers",
                    self.effective_workers
                )));
            }

            self.prepare_workspace(maximum_k, maximum_rows, batch_width);

            const K: usize = 8;
            // The synthetic fan-out never exceeds a real model matrix. An
            // operator-selected tile count can refine scheduling but cannot
            // turn excluded preparation into an unbounded allocation.
            let rows = self
                .effective_workers
                .saturating_mul(self.tiles_per_worker)
                .min(maximum_rows)
                .max(1);
            let mut weights = vec![0.0f32; rows.saturating_mul(K)];
            for row in 0..rows {
                weights[row * K + row % K] = 1.0;
            }
            let mut input = vec![0.0f32; batch_width.saturating_mul(K)];
            for stream in 0..batch_width {
                for depth in 0..K {
                    input[stream * K + depth] = (1 + stream + depth) as f32;
                }
            }
            let mut output = vec![f32::NAN; rows.saturating_mul(batch_width)];
            self.matmul_batched(&mut output, &input, &weights, K, batch_width);
            let backend_exercised = (0..batch_width).all(|stream| {
                (0..rows).all(|row| {
                    output[stream * rows + row].to_bits() == input[stream * K + row % K].to_bits()
                })
            });
            if !backend_exercised {
                return Err(SourceUnavailable::new(
                    "exact executor prestart produced unexpected output bits",
                ));
            }
            Ok(TeacherExecutionPreparation {
                elapsed_seconds: started.elapsed().as_secs_f64(),
                workers_observed,
                batch_width,
                backend_exercised,
                workspace_capacity_bytes: as_u64(self.workspace_capacity_bytes()),
                workspace_growth_events: self
                    .counters
                    .workspace_growth_events
                    .load(Ordering::Acquire),
                workspace_growth_bytes: self
                    .counters
                    .workspace_growth_bytes
                    .load(Ordering::Acquire),
            })
        }

        pub(crate) fn begin_forward(&self, streams: usize) {
            // The model owns one bounded physical-forward lane at a time. Reset
            // only the per-forward high-water; lifetime diagnostics remain
            // monotonic across the measured execution.
            self.counters
                .forward_max_active_workers
                .store(0, Ordering::Release);
            saturating_add(&self.counters.forward_calls, 1);
            saturating_add(&self.counters.streams_started, as_u64(streams));
            let active = self
                .counters
                .active_streams
                .fetch_add(streams, Ordering::AcqRel)
                .saturating_add(streams);
            self.counters
                .max_active_streams
                .fetch_max(active, Ordering::AcqRel);
            self.emit();
        }

        pub(crate) fn complete_forward(&self, streams: usize) {
            saturating_add(&self.counters.streams_completed, as_u64(streams));
            self.counters
                .active_streams
                .fetch_sub(streams, Ordering::AcqRel);
            self.emit();
        }

        fn emit(&self) {
            if let Some(observer) = &self.observer {
                let epoch = self
                    .counters
                    .observer_epoch
                    .fetch_add(1, Ordering::AcqRel)
                    .saturating_add(1);
                let mut snapshot = self.snapshot();
                snapshot.observer_epoch = epoch;
                observer(snapshot);
            }
        }

        fn enter_task(&self) -> ActiveTask<'_> {
            let previous_active = self.counters.active_workers.fetch_add(1, Ordering::AcqRel);
            let active = previous_active.saturating_add(1);
            self.counters
                .max_active_workers
                .fetch_max(active, Ordering::AcqRel);
            let previous_forward_max = self
                .counters
                .forward_max_active_workers
                .fetch_max(active, Ordering::AcqRel);
            if active > 1 {
                let forward = self.counters.forward_calls.load(Ordering::Acquire);
                let newly_multiworker = self
                    .counters
                    .multiworker_observed_forward
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |observed| {
                        (observed < forward).then_some(forward)
                    })
                    .is_ok();
                if newly_multiworker {
                    saturating_add(&self.counters.multiworker_forward_calls, 1);
                }
            }
            // Long exact tiles can take far longer than a heartbeat interval.
            // Publish entry after the active/peak counters move so an observer
            // can distinguish healthy in-flight work from a stalled black box.
            // Bound callback overhead: only a newly observed *per-forward*
            // concurrency high-water emits (at most W callbacks per physical
            // forward). A lifetime-only high-water would make later forwards
            // invisible after their phase-local observer state resets.
            if active > previous_forward_max {
                self.emit();
            }
            ActiveTask { executor: self }
        }

        fn tile_rows(&self, rows: usize) -> usize {
            super::exact_tile_rows_for(rows, self.effective_workers, self.tiles_per_worker)
        }

        pub(crate) fn row_tiles(&self, rows: usize) -> usize {
            super::exact_row_tiles_for(rows, self.effective_workers, self.tiles_per_worker)
        }

        fn record_tile(&self, output_cells: usize, k: usize) {
            let tiles_completed = saturating_add(&self.counters.tiles_completed, 1);
            saturating_add(&self.counters.output_cells_completed, as_u64(output_cells));
            saturating_add(
                &self.counters.scalar_terms_completed,
                as_u64(output_cells).saturating_mul(as_u64(k)),
            );
            // Exact final counters advance for every tile. Progress callbacks
            // are deliberately coarse: the first tile and then one actual
            // worker-wave quantum. A matrix with at most W*T row tiles emits
            // at most T+1 tile-progress callbacks, so large matrices visibly
            // advance without putting observer synchronization in every tile.
            let progress_quantum = as_u64(self.effective_workers.max(1));
            if tiles_completed == 1 || tiles_completed.is_multiple_of(progress_quantum) {
                self.emit();
            }
        }

        /// Compute `W[rows,k] * x[k]`, partitioning only complete output rows.
        pub(crate) fn matmul(&self, output: &mut [f32], x: &[f32], w: &[f32], k: usize) {
            let rows = output.len();
            assert_eq!(x.len(), k, "teacher input vector must have k elements");
            assert!(
                w.len() >= rows.saturating_mul(k),
                "teacher weight matrix does not contain every output row"
            );
            saturating_add(&self.counters.matrix_calls, 1);
            if rows == 0 {
                return;
            }
            let w = &w[..rows * k];
            let tile_rows = self.tile_rows(rows);
            let partition = Partition::new(Shape { m: rows, k, n: 1 }, tile_rows, 1);

            if let Some(pool) = &self.pool {
                pool.install(|| {
                    output
                        .par_chunks_mut(tile_rows)
                        .zip(w.par_chunks(tile_rows * k))
                        .enumerate()
                        .for_each(|(tile_index, (tile_output, tile_weights))| {
                            let tile = partition
                                .tile(tile_index)
                                .expect("parallel row chunk belongs to the partition");
                            debug_assert_eq!(tile.col, 0);
                            debug_assert_eq!(tile.cols, 1);
                            debug_assert_eq!(tile.rows, tile_output.len());
                            let mut scratch = self.current_worker_scratch();
                            self.ensure_scratch(&mut scratch, k, 1);
                            let ExactScratch { pa, pb } = &mut *scratch;
                            let task = self.enter_task();
                            uor_matmul::slice::gemm_float(
                                tile.rows,
                                k,
                                1,
                                tile_weights,
                                x,
                                tile_output,
                                &mut pa[..k],
                                &mut pb[..k],
                            )
                            .expect("exact output-row tile is a valid product");
                            drop(task);
                            self.record_tile(tile.rows, k);
                        });
                });
            } else {
                let mut scratch = self.current_worker_scratch();
                self.ensure_scratch(&mut scratch, k, 1);
                let ExactScratch { pa, pb } = &mut *scratch;
                let task = self.enter_task();
                uor_matmul::slice::gemm_float(rows, k, 1, w, x, output, &mut pa[..k], &mut pb[..k])
                    .expect("exact teacher matrix-vector product is a valid product");
                drop(task);
                self.record_tile(rows, k);
            }
        }

        /// Compute independent `W[rows,k] * x_b[k]` products with shared
        /// immutable weights and private per-worker exact scratch.
        pub(crate) fn matmul_batched(
            &self,
            output: &mut [f32],
            x: &[f32],
            w: &[f32],
            k: usize,
            batch: usize,
        ) {
            assert!(batch > 0, "teacher batch must be nonzero");
            assert_eq!(output.len() % batch, 0);
            let rows = output.len() / batch;
            assert_eq!(x.len(), batch.saturating_mul(k));
            assert!(w.len() >= rows.saturating_mul(k));
            saturating_add(&self.counters.matrix_calls, 1);
            saturating_add(&self.counters.batched_matrix_calls, 1);
            self.counters
                .max_matrix_batch_width
                .fetch_max(batch, Ordering::AcqRel);
            if rows == 0 {
                return;
            }
            let w = &w[..rows * k];

            // The caller stores X sequence-major (`batch x k`). One canonical
            // transpose makes every stream a column, so each output-row tile
            // is ONE exact GEMM over shared weights:
            //
            //   W_tile[tr,k] * X^T[k,batch] -> C_tile[tr,batch].
            //
            // K is never partitioned. A weight row is decoded once per tile,
            // then reused across all stream columns by the exact driver.
            let input_words = k
                .checked_mul(batch)
                .expect("exact input workspace shape must fit usize");
            let output_words = rows
                .checked_mul(batch)
                .expect("exact output workspace shape must fit usize");
            let mut workspace = self
                .batch_workspace
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.grow_vec(&mut workspace.x_transposed, input_words, 0.0f32);
            self.grow_vec(&mut workspace.output_transposed, output_words, 0.0f32);
            let ExactBatchWorkspace {
                x_transposed,
                output_transposed,
            } = &mut **workspace;
            let x_transposed = &mut x_transposed[..input_words];
            let output_transposed = &mut output_transposed[..output_words];
            for stream in 0..batch {
                for depth in 0..k {
                    x_transposed[depth * batch + stream] = x[stream * k + depth];
                }
            }

            let tile_rows = self.tile_rows(rows);
            let partition = Partition::new(
                Shape {
                    m: rows,
                    k,
                    n: batch,
                },
                tile_rows,
                0,
            );

            if let Some(pool) = &self.pool {
                pool.install(|| {
                    output_transposed
                        .par_chunks_mut(tile_rows * batch)
                        .zip(w.par_chunks(tile_rows * k))
                        .enumerate()
                        .for_each(|(tile_index, (tile_output, tile_weights))| {
                            let tile = partition
                                .tile(tile_index)
                                .expect("parallel row tile belongs to the partition");
                            debug_assert_eq!(tile.col, 0);
                            debug_assert_eq!(tile.cols, batch);
                            debug_assert_eq!(tile_output.len(), tile.rows * batch);
                            let mut scratch = self.current_worker_scratch();
                            self.ensure_scratch(&mut scratch, k, batch);
                            let ExactScratch { pa, pb } = &mut *scratch;
                            let task = self.enter_task();
                            uor_matmul::slice::gemm_float(
                                tile.rows,
                                k,
                                batch,
                                tile_weights,
                                x_transposed,
                                tile_output,
                                &mut pa[..k],
                                &mut pb[..input_words],
                            )
                            .expect("exact shared-weight batch tile is a valid product");
                            drop(task);
                            self.record_tile(tile.rows * batch, k);
                        });
                });
            } else {
                let mut scratch = self.current_worker_scratch();
                self.ensure_scratch(&mut scratch, k, batch);
                let ExactScratch { pa, pb } = &mut *scratch;
                let task = self.enter_task();
                uor_matmul::slice::gemm_float(
                    rows,
                    k,
                    batch,
                    w,
                    x_transposed,
                    output_transposed,
                    &mut pa[..k],
                    &mut pb[..input_words],
                )
                .expect("exact shared-weight batch product is a valid product");
                drop(task);
                self.record_tile(rows * batch, k);
            }

            // Canonical ordered scatter back to the public sequence-major
            // layout. Completion order cannot affect bytes or write ownership.
            for row in 0..rows {
                for stream in 0..batch {
                    output[stream * rows + row] = output_transposed[row * batch + stream];
                }
            }
        }
    }

    fn as_u64(value: usize) -> u64 {
        u64::try_from(value).unwrap_or(u64::MAX)
    }

    fn saturating_add(counter: &AtomicU64, value: u64) -> u64 {
        counter
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                Some(current.saturating_add(value))
            })
            .unwrap_or_else(|current| current)
            .saturating_add(value)
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    use super::{TeacherExecutionConfig, TeacherExecutionObserver, TeacherExecutionSnapshot};
    use crate::SourceUnavailable;
    use std::cell::Cell;
    use uor_matmul::PackedCode;

    /// Sequential portable counterpart; no thread or host-feature dependency.
    pub(crate) struct ExactExecutor {
        counters: Cell<TeacherExecutionSnapshot>,
        observer: Option<TeacherExecutionObserver>,
    }

    impl ExactExecutor {
        pub(crate) fn new(config: TeacherExecutionConfig) -> Result<Self, SourceUnavailable> {
            let workers = config.resolved_workers();
            Ok(Self {
                counters: Cell::new(TeacherExecutionSnapshot {
                    requested_workers: workers,
                    effective_workers: 1,
                    ..TeacherExecutionSnapshot::default()
                }),
                observer: config.observer,
            })
        }

        pub(crate) fn snapshot(&self) -> TeacherExecutionSnapshot {
            self.counters.get()
        }

        pub(crate) fn begin_measured_execution(&mut self, observer: TeacherExecutionObserver) {
            let snapshot = self.snapshot();
            self.counters.set(TeacherExecutionSnapshot {
                requested_workers: snapshot.requested_workers,
                effective_workers: snapshot.effective_workers,
                ..TeacherExecutionSnapshot::default()
            });
            self.observer = Some(observer);
        }

        pub(crate) fn record_workspace_growth_bytes(&self, bytes: usize) {
            if bytes == 0 {
                return;
            }
            self.update(|snapshot| {
                snapshot.workspace_growth_events =
                    snapshot.workspace_growth_events.saturating_add(1);
                snapshot.workspace_growth_bytes = snapshot
                    .workspace_growth_bytes
                    .saturating_add(u64::try_from(bytes).unwrap_or(u64::MAX));
            });
        }

        pub(crate) fn prepare_workspace(&self, _k: usize, _rows: usize, _batch: usize) {
            // The wasm32 fallback remains sequential and portable. Model-level
            // stacked buffers are retained; uor-matmul scratch stays local to
            // the portable call because there is no hosted worker pool.
        }

        pub(crate) fn row_tiles(&self, rows: usize) -> usize {
            usize::from(rows > 0)
        }

        fn update(&self, change: impl FnOnce(&mut TeacherExecutionSnapshot)) {
            let mut snapshot = self.snapshot();
            change(&mut snapshot);
            if self.observer.is_some() {
                snapshot.observer_epoch = snapshot.observer_epoch.saturating_add(1);
            }
            self.counters.set(snapshot);
            if let Some(observer) = &self.observer {
                observer(snapshot);
            }
        }

        pub(crate) fn begin_forward(&self, streams: usize) {
            self.update(|snapshot| {
                snapshot.forward_max_active_workers = 0;
                snapshot.forward_calls = snapshot.forward_calls.saturating_add(1);
                snapshot.streams_started = snapshot
                    .streams_started
                    .saturating_add(u64::try_from(streams).unwrap_or(u64::MAX));
                snapshot.active_streams = snapshot.active_streams.saturating_add(streams);
                snapshot.max_active_streams =
                    snapshot.max_active_streams.max(snapshot.active_streams);
            });
        }

        pub(crate) fn complete_forward(&self, streams: usize) {
            self.update(|snapshot| {
                snapshot.streams_completed = snapshot
                    .streams_completed
                    .saturating_add(u64::try_from(streams).unwrap_or(u64::MAX));
                snapshot.active_streams = snapshot.active_streams.saturating_sub(streams);
            });
        }

        pub(crate) fn matmul(&self, output: &mut [f32], x: &[f32], w: &[f32], k: usize) {
            let rows = output.len();
            let mut pa = vec![PackedCode::default(); k];
            let mut pb = vec![PackedCode::default(); k];
            uor_matmul::slice::gemm_float(rows, k, 1, &w[..rows * k], x, output, &mut pa, &mut pb)
                .expect("portable exact teacher product is valid");
            self.update(|snapshot| {
                snapshot.matrix_calls = snapshot.matrix_calls.saturating_add(1);
                snapshot.tiles_completed = snapshot.tiles_completed.saturating_add(1);
                snapshot.output_cells_completed = snapshot
                    .output_cells_completed
                    .saturating_add(u64::try_from(rows).unwrap_or(u64::MAX));
                snapshot.scalar_terms_completed = snapshot.scalar_terms_completed.saturating_add(
                    u64::try_from(rows)
                        .unwrap_or(u64::MAX)
                        .saturating_mul(u64::try_from(k).unwrap_or(u64::MAX)),
                );
                snapshot.max_active_workers = 1;
                snapshot.forward_max_active_workers = 1;
            });
        }

        pub(crate) fn matmul_batched(
            &self,
            output: &mut [f32],
            x: &[f32],
            w: &[f32],
            k: usize,
            batch: usize,
        ) {
            let rows = super::portable_shared_weight_matmul_batched(output, x, w, k, batch);
            self.update(|snapshot| {
                snapshot.matrix_calls = snapshot.matrix_calls.saturating_add(1);
                snapshot.batched_matrix_calls = snapshot.batched_matrix_calls.saturating_add(1);
                snapshot.max_matrix_batch_width = snapshot.max_matrix_batch_width.max(batch);
                if rows > 0 {
                    let output_cells = u64::try_from(rows)
                        .unwrap_or(u64::MAX)
                        .saturating_mul(u64::try_from(batch).unwrap_or(u64::MAX));
                    snapshot.tiles_completed = snapshot.tiles_completed.saturating_add(1);
                    snapshot.output_cells_completed =
                        snapshot.output_cells_completed.saturating_add(output_cells);
                    snapshot.scalar_terms_completed =
                        snapshot.scalar_terms_completed.saturating_add(
                            output_cells.saturating_mul(u64::try_from(k).unwrap_or(u64::MAX)),
                        );
                    snapshot.max_active_workers = 1;
                    snapshot.forward_max_active_workers = 1;
                }
            });
        }
    }
}

pub(crate) use platform::ExactExecutor;

#[cfg(test)]
mod tests {
    #[test]
    fn portable_shared_weight_batch_matches_serial_exact_bits() {
        const ROWS: usize = 11;
        const K: usize = 13;
        const BATCH: usize = 5;
        let weights = (0..ROWS * K)
            .map(|index| ((index * 29 % 43) as f32 - 21.0) / 32.0)
            .collect::<Vec<_>>();
        let input = (0..BATCH * K)
            .map(|index| ((index * 17 % 31) as f32 - 15.0) / 16.0)
            .collect::<Vec<_>>();
        let mut expected = vec![0.0f32; BATCH * ROWS];
        for (stream, stream_input) in input.chunks_exact(K).enumerate() {
            let mut pa = vec![uor_matmul::PackedCode::default(); K];
            let mut pb = vec![uor_matmul::PackedCode::default(); K];
            uor_matmul::slice::gemm_float(
                ROWS,
                K,
                1,
                &weights,
                stream_input,
                &mut expected[stream * ROWS..(stream + 1) * ROWS],
                &mut pa,
                &mut pb,
            )
            .expect("serial exact product");
        }

        let mut actual = vec![0.0f32; BATCH * ROWS];
        let rows =
            super::portable_shared_weight_matmul_batched(&mut actual, &input, &weights, K, BATCH);
        assert_eq!(rows, ROWS);
        assert_eq!(
            actual
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
    }
}
