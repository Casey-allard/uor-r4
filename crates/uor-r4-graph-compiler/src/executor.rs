//! Deterministic Compiler Executor Abstraction (#165).
//!
//! Provides a backend-neutral abstraction (`CompilerExecutor`) with:
//! - `SequentialExecutor`: Normative reference implementation (single-threaded loop).
//! - `RayonExecutor`: Multicore CPU executor owning an explicit Rayon `ThreadPool`.
//!
//! Enforces:
//! - Positional output mapping (`map` result index matches input index).
//! - Deterministic error aggregation (returns the error from the lowest input index).
//! - Worker panic containment (`std::panic::catch_unwind` converted to `CompileError::ExecutionPanic`).

use std::any::Any;
use std::fmt;

/// Typed compilation error taxonomy for compiler executors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// Worker closure returned an explicit error message.
    WorkerError { input_index: usize, message: String },
    /// Worker closure panicked during execution.
    ExecutionPanic {
        input_index: usize,
        panic_message: String,
    },
    /// Executor configuration error.
    InvalidConfiguration(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkerError {
                input_index,
                message,
            } => {
                write!(f, "Worker error at index {input_index}: {message}")
            }
            Self::ExecutionPanic {
                input_index,
                panic_message,
            } => {
                write!(f, "Worker panic at index {input_index}: {panic_message}")
            }
            Self::InvalidConfiguration(msg) => write!(f, "Executor configuration error: {msg}"),
        }
    }
}

impl std::error::Error for CompileError {}

/// Abstract compiler task executor.
pub trait CompilerExecutor: Send + Sync {
    /// Execute function `f` over each item in `inputs`, returning mapped outputs in positional order.
    fn map<I, O, F>(&self, inputs: &[I], f: F) -> Result<Vec<O>, CompileError>
    where
        I: Sync,
        O: Send,
        F: Fn(&I) -> Result<O, String> + Sync;
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
    fn map<I, O, F>(&self, inputs: &[I], f: F) -> Result<Vec<O>, CompileError>
    where
        I: Sync,
        O: Send,
        F: Fn(&I) -> Result<O, String> + Sync,
    {
        let mut results = Vec::with_capacity(inputs.len());
        for (idx, item) in inputs.iter().enumerate() {
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(item)));
            match res {
                Ok(Ok(out)) => results.push(out),
                Ok(Err(msg)) => {
                    return Err(CompileError::WorkerError {
                        input_index: idx,
                        message: msg,
                    });
                }
                Err(panic_err) => {
                    let msg = extract_panic_message(&*panic_err);
                    return Err(CompileError::ExecutionPanic {
                        input_index: idx,
                        panic_message: msg,
                    });
                }
            }
        }
        Ok(results)
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
    pub fn new(num_threads: usize) -> Result<Self, CompileError> {
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
            .map_err(|e| CompileError::InvalidConfiguration(e.to_string()))?;
        Ok(Self {
            pool,
            num_threads: threads,
        })
    }

    pub fn num_threads(&self) -> usize {
        self.num_threads
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl CompilerExecutor for RayonExecutor {
    fn map<I, O, F>(&self, inputs: &[I], f: F) -> Result<Vec<O>, CompileError>
    where
        I: Sync,
        O: Send,
        F: Fn(&I) -> Result<O, String> + Sync,
    {
        if inputs.is_empty() {
            return Ok(Vec::new());
        }

        let raw_results: Vec<Result<Result<O, String>, Box<dyn Any + Send>>> =
            self.pool.install(|| {
                use rayon::prelude::*;
                inputs
                    .par_iter()
                    .map(|item| std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(item))))
                    .collect()
            });

        let mut outputs = Vec::with_capacity(inputs.len());
        for (idx, res) in raw_results.into_iter().enumerate() {
            match res {
                Ok(Ok(val)) => outputs.push(val),
                Ok(Err(msg)) => {
                    return Err(CompileError::WorkerError {
                        input_index: idx,
                        message: msg,
                    });
                }
                Err(panic_err) => {
                    let msg = extract_panic_message(&*panic_err);
                    return Err(CompileError::ExecutionPanic {
                        input_index: idx,
                        panic_message: msg,
                    });
                }
            }
        }

        Ok(outputs)
    }
}

fn extract_panic_message(err: &(dyn Any + Send)) -> String {
    if let Some(s) = err.downcast_ref::<&'static str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown worker panic".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_executor_mapping_and_panic_containment() {
        let exec = SequentialExecutor::new();
        let inputs = vec![1, 2, 3, 4, 5];
        let res = exec.map(&inputs, |&x| Ok(x * 10)).unwrap();
        assert_eq!(res, vec![10, 20, 30, 40, 50]);

        let err = exec
            .map(&inputs, |&x| {
                if x == 3 {
                    Err("x is 3".to_string())
                } else {
                    Ok(x)
                }
            })
            .unwrap_err();
        assert_eq!(
            err,
            CompileError::WorkerError {
                input_index: 2,
                message: "x is 3".to_string()
            }
        );

        let panic_err = exec
            .map(&inputs, |&x| {
                if x == 2 {
                    panic!("simulated panic");
                } else {
                    Ok(x)
                }
            })
            .unwrap_err();
        assert_eq!(
            panic_err,
            CompileError::ExecutionPanic {
                input_index: 1,
                panic_message: "simulated panic".to_string()
            }
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_rayon_executor_mapping_equivalence_and_panic_containment() {
        let seq_exec = SequentialExecutor::new();
        let par_exec = RayonExecutor::new(4).unwrap();
        assert_eq!(par_exec.num_threads(), 4);

        let inputs: Vec<u32> = (1..=100).collect();
        let seq_out = seq_exec.map(&inputs, |&x| Ok(x * x + 7)).unwrap();
        let par_out = par_exec.map(&inputs, |&x| Ok(x * x + 7)).unwrap();
        assert_eq!(seq_out, par_out);

        let panic_err = par_exec
            .map(&inputs, |&x| {
                if x == 42 {
                    panic!("rayon worker panic");
                } else {
                    Ok(x)
                }
            })
            .unwrap_err();
        assert_eq!(
            panic_err,
            CompileError::ExecutionPanic {
                input_index: 41,
                panic_message: "rayon worker panic".to_string()
            }
        );
    }
}
