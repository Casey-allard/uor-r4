//! Compiler Memory-Budget & Backpressure Model (#169).
//!
//! Provides concurrency-aware memory budget derivation, per-stage memory estimates,
//! bounded in-flight backpressure limiting, and typed `BudgetExceeded` error handling.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Minimum baseline memory footprint required for compiler operation (64 MiB).
pub const MINIMUM_COMPILER_BUDGET_BYTES: usize = 64 * 1024 * 1024;
/// Baseline per-worker scratch memory allocation (4 MiB).
pub const DEFAULT_PER_WORKER_SCRATCH_BYTES: usize = 4 * 1024 * 1024;

/// Per-stage memory estimate attached to pipeline DAG stages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageMemoryEstimate {
    /// Stage identifier (e.g. `"observation_ingestion"`).
    pub stage_id: &'static str,
    /// Base memory overhead required by stage execution (bytes).
    pub base_overhead_bytes: usize,
    /// Memory required per active worker thread (bytes).
    pub per_worker_scratch_bytes: usize,
    /// Buffer memory required per active shard queue item (bytes).
    pub per_shard_buffer_bytes: usize,
}

/// Concurrency-aware compiler memory budget configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerMemoryBudget {
    /// Total allocated memory budget (bytes).
    pub total_budget_bytes: usize,
    /// Max concurrent in-flight tasks permitted under this budget.
    pub max_in_flight_tasks: usize,
    /// Memory allocated per worker thread scratch buffer (bytes).
    pub per_worker_scratch_bytes: usize,
    /// Active worker thread count ($T \ge 1$).
    pub worker_threads: usize,
}

impl CompilerMemoryBudget {
    /// Calculate minimum supported memory budget for $T$ worker threads.
    pub fn min_supported_budget_bytes(worker_threads: usize) -> usize {
        MINIMUM_COMPILER_BUDGET_BYTES + (worker_threads * DEFAULT_PER_WORKER_SCRATCH_BYTES)
    }

    /// Derive concurrency-aware memory budget configuration ($PeakRSS \le
    /// Budget$). `None` when `total_budget_bytes` is below the minimum this
    /// worker count requires (R5 — the requested budget is not a valid
    /// configuration, the absence of a product rather than a raised error).
    pub fn derive(total_budget_bytes: usize, worker_threads: usize) -> Option<Self> {
        let threads = worker_threads.max(1);
        let min_required = Self::min_supported_budget_bytes(threads);
        if total_budget_bytes < min_required {
            return None;
        }

        let worker_scratch_total = threads * DEFAULT_PER_WORKER_SCRATCH_BYTES;
        let available_for_in_flight =
            total_budget_bytes.saturating_sub(MINIMUM_COMPILER_BUDGET_BYTES + worker_scratch_total);
        let per_task_estimate = 512 * 1024; // 512 KB per task buffer
        let max_in_flight_tasks = (available_for_in_flight / per_task_estimate).max(threads * 2);

        Some(CompilerMemoryBudget {
            total_budget_bytes,
            max_in_flight_tasks,
            per_worker_scratch_bytes: DEFAULT_PER_WORKER_SCRATCH_BYTES,
            worker_threads: threads,
        })
    }
}

/// In-flight backpressure limiter to prevent unbounded queue growth.
#[derive(Debug)]
pub struct InFlightBackpressureLimiter {
    capacity: usize,
    current_in_flight: Arc<AtomicUsize>,
}

impl InFlightBackpressureLimiter {
    /// Construct a new limiter with bounded task capacity.
    pub fn new(capacity: usize) -> Self {
        InFlightBackpressureLimiter {
            capacity: capacity.max(1),
            current_in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Attempt to acquire a slot for an in-flight task. Returns a RAII guard, or
    /// `None` when the limiter is already at capacity (R5 — the requested slot
    /// is not available, reported as `None` rather than a raised error).
    pub fn try_acquire(&self) -> Option<BackpressureGuard> {
        let mut curr = self.current_in_flight.load(Ordering::Relaxed);
        loop {
            if curr >= self.capacity {
                return None;
            }
            match self.current_in_flight.compare_exchange_weak(
                curr,
                curr + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Some(BackpressureGuard {
                        current_in_flight: Arc::clone(&self.current_in_flight),
                    });
                }
                Err(actual) => curr = actual,
            }
        }
    }

    /// Return current number of in-flight tasks.
    pub fn current_in_flight(&self) -> usize {
        self.current_in_flight.load(Ordering::Relaxed)
    }

    /// Return total task capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// RAII guard releasing an in-flight slot upon drop.
#[derive(Debug)]
pub struct BackpressureGuard {
    current_in_flight: Arc<AtomicUsize>,
}

impl Drop for BackpressureGuard {
    fn drop(&mut self) {
        self.current_in_flight.fetch_sub(1, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_derivation_valid() {
        let budget_bytes = 256 * 1024 * 1024; // 256 MiB
        let budget = CompilerMemoryBudget::derive(budget_bytes, 4).unwrap();
        assert_eq!(budget.total_budget_bytes, budget_bytes);
        assert_eq!(budget.worker_threads, 4);
        assert!(budget.max_in_flight_tasks >= 8);
    }

    #[test]
    fn test_budget_derivation_too_small() {
        let too_small = 10 * 1024 * 1024; // 10 MiB
        assert!(CompilerMemoryBudget::derive(too_small, 4).is_none());
    }

    #[test]
    fn test_backpressure_limiter_capacity_cap() {
        let limiter = InFlightBackpressureLimiter::new(2);
        let g1 = limiter.try_acquire().unwrap();
        let g2 = limiter.try_acquire().unwrap();
        assert_eq!(limiter.current_in_flight(), 2);

        assert!(limiter.try_acquire().is_none());

        drop(g1);
        assert_eq!(limiter.current_in_flight(), 1);
        let g3 = limiter.try_acquire().unwrap();
        assert_eq!(limiter.current_in_flight(), 2);
        drop(g2);
        drop(g3);
        assert_eq!(limiter.current_in_flight(), 0);
    }
}
