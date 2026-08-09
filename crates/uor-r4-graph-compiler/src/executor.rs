//! Deterministic Compiler Executor Abstraction (#165).
//!
//! Provides a backend-neutral abstraction (`CompilerExecutor`) with:
//! - `SequentialExecutor`: Normative reference implementation (single-threaded loop).
//! - `RayonExecutor`: Multicore CPU executor owning an explicit Rayon `ThreadPool`.
//!
//! Enforces:
//! - Positional output mapping (`map` result index matches input index).
//! - Worker panic propagation: `map` is total over an infallible worker
//!   closure, so the only way a worker can fail is to panic, and a panic is a
//!   defect that propagates (re-raised in the calling thread), never a
//!   sanctioned reportable condition (R5). There is no error surface to
//!   aggregate.

/// Abstract compiler task executor.
pub trait CompilerExecutor: Send + Sync {
    /// Execute function `f` over each item in `inputs`, returning mapped outputs
    /// in positional order. Total: `f` is infallible and the mapping cannot
    /// report an error; a worker panic propagates.
    fn map<I, O, F>(&self, inputs: &[I], f: F) -> Vec<O>
    where
        I: Sync,
        O: Send,
        F: Fn(&I) -> O + Sync;
}

/// Normative sequential reference executor (zero concurrency).
#[derive(Debug, Default, Clone)]
pub struct SequentialExecutor;

impl SequentialExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl CompilerExecutor for SequentialExecutor {
    fn map<I, O, F>(&self, inputs: &[I], f: F) -> Vec<O>
    where
        I: Sync,
        O: Send,
        F: Fn(&I) -> O + Sync,
    {
        inputs.iter().map(&f).collect()
    }
}

/// Multicore CPU executor backed by an owned Rayon `ThreadPool`.
#[cfg(not(target_arch = "wasm32"))]
pub struct RayonExecutor {
    pool: rayon::ThreadPool,
    num_threads: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl RayonExecutor {
    /// Build an executor over a dedicated `ThreadPool` of `num_threads`
    /// (0 = the host's available parallelism). Total: a thread-pool that will
    /// not build is a host defect (cannot spawn threads), so it propagates as a
    /// panic rather than a sanctioned condition (R5).
    pub fn new(num_threads: usize) -> Self {
        let threads = if num_threads == 0 {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        } else {
            num_threads
        };
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .thread_name(|i| format!("uor-r4-compiler-{i}"))
            .build()
            .unwrap_or_else(|e| panic!("uor-r4 compiler thread pool construction failed: {e}"));
        Self {
            pool,
            num_threads: threads,
        }
    }

    pub fn num_threads(&self) -> usize {
        self.num_threads
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl CompilerExecutor for RayonExecutor {
    fn map<I, O, F>(&self, inputs: &[I], f: F) -> Vec<O>
    where
        I: Sync,
        O: Send,
        F: Fn(&I) -> O + Sync,
    {
        if inputs.is_empty() {
            return Vec::new();
        }
        // `par_iter().map(..).collect()` preserves positional order, and rayon
        // re-raises a worker panic in this thread on `collect` — the total
        // panic-propagation contract above.
        self.pool.install(|| {
            use rayon::prelude::*;
            inputs.par_iter().map(&f).collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_executor_mapping_and_panic_propagation() {
        let exec = SequentialExecutor::new();
        let inputs = vec![1, 2, 3, 4, 5];
        let res = exec.map(&inputs, |&x| x * 10);
        assert_eq!(res, vec![10, 20, 30, 40, 50]);

        // A worker panic propagates (total map): catch it to assert it aborts
        // the whole mapping rather than being folded into a reported error.
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            exec.map(
                &inputs,
                |&x| if x == 2 { panic!("simulated panic") } else { x },
            )
        }));
        assert!(panicked.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_rayon_executor_mapping_equivalence_and_panic_propagation() {
        let seq_exec = SequentialExecutor::new();
        let par_exec = RayonExecutor::new(4);
        assert_eq!(par_exec.num_threads(), 4);

        let inputs: Vec<u32> = (1..=100).collect();
        let seq_out = seq_exec.map(&inputs, |&x| x * x + 7);
        let par_out = par_exec.map(&inputs, |&x| x * x + 7);
        assert_eq!(seq_out, par_out);

        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            par_exec.map(&inputs, |&x| {
                if x == 42 {
                    panic!("rayon worker panic")
                } else {
                    x
                }
            })
        }));
        assert!(panicked.is_err());
    }
}
