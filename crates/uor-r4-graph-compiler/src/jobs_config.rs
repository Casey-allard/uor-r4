//! Compiler Thread-Pool, Jobs Configuration & Oversubscription Policy (#168).
//!
//! Provides explicit thread concurrency resolution (`CLI > env > default`),
//! typed error validation, and dedicated custom Rayon thread pool construction.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Precedence source of resolved compiler jobs configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobsConfigSource {
    /// `--jobs N` passed explicitly on the CLI command line.
    CliArg,
    /// `R4_COMPILER_THREADS` environment variable set.
    EnvVar,
    /// Default policy (`min(logical_cpus, 8)`).
    DefaultPolicy,
}

/// Errors returned when parsing or validating jobs configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobsConfigError {
    /// Passing 0 worker threads is forbidden.
    ZeroJobsForbidden,
    /// Thread count string failed decimal integer parsing.
    InvalidJobCount { value: String },
    /// Custom thread pool construction failed.
    ThreadPoolBuildFailed { reason: String },
}

impl fmt::Display for JobsConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobsConfigError::ZeroJobsForbidden => {
                write!(f, "Jobs configuration error: thread count 0 is forbidden")
            }
            JobsConfigError::InvalidJobCount { value } => write!(
                f,
                "Jobs configuration error: invalid thread count '{value}'"
            ),
            JobsConfigError::ThreadPoolBuildFailed { reason } => write!(
                f,
                "Jobs configuration error: failed to build dedicated thread pool: {reason}"
            ),
        }
    }
}

impl std::error::Error for JobsConfigError {}

/// Compiler jobs and thread-pool configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerJobsConfig {
    /// Resolved worker thread count ($jobs \ge 1$).
    pub jobs: usize,
    /// Prefix for custom worker thread naming (default `"r4-compile"`).
    pub thread_name_prefix: String,
    /// Precedence source used to resolve thread count.
    pub source: JobsConfigSource,
}

impl Default for CompilerJobsConfig {
    fn default() -> Self {
        let jobs = Self::default_job_count();
        CompilerJobsConfig {
            jobs,
            thread_name_prefix: "r4-compile".to_string(),
            source: JobsConfigSource::DefaultPolicy,
        }
    }
}

impl CompilerJobsConfig {
    /// Calculate default thread count (`min(logical_cpus, 8)`).
    pub fn default_job_count() -> usize {
        let cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        cpus.min(8)
    }

    /// Resolve configuration following precedence: CLI (`--jobs`) > Env (`R4_COMPILER_THREADS`) > Default.
    pub fn resolve(
        cli_jobs: Option<usize>,
        env_val: Option<&str>,
    ) -> Result<Self, JobsConfigError> {
        // Tier 1: CLI argument
        if let Some(jobs) = cli_jobs {
            if jobs == 0 {
                return Err(JobsConfigError::ZeroJobsForbidden);
            }
            return Ok(CompilerJobsConfig {
                jobs,
                thread_name_prefix: "r4-compile".to_string(),
                source: JobsConfigSource::CliArg,
            });
        }

        // Tier 2: Environment variable
        if let Some(env_str) = env_val {
            let trimmed = env_str.trim();
            if !trimmed.is_empty() {
                let parsed: usize =
                    trimmed
                        .parse()
                        .map_err(|_| JobsConfigError::InvalidJobCount {
                            value: trimmed.to_string(),
                        })?;
                if parsed == 0 {
                    return Err(JobsConfigError::ZeroJobsForbidden);
                }
                return Ok(CompilerJobsConfig {
                    jobs: parsed,
                    thread_name_prefix: "r4-compile".to_string(),
                    source: JobsConfigSource::EnvVar,
                });
            }
        }

        // Tier 3: Default policy
        Ok(CompilerJobsConfig::default())
    }

    /// Build a dedicated named Rayon thread pool owned by the compiler context.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn build_dedicated_thread_pool(&self) -> Result<rayon::ThreadPool, JobsConfigError> {
        let prefix = self.thread_name_prefix.clone();
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.jobs)
            .thread_name(move |idx| format!("{prefix}-{idx}"))
            .build()
            .map_err(|e| JobsConfigError::ThreadPoolBuildFailed {
                reason: format!("{e:?}"),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jobs_config_precedence_cli_over_env_and_default() {
        let config = CompilerJobsConfig::resolve(Some(4), Some("16")).unwrap();
        assert_eq!(config.jobs, 4);
        assert_eq!(config.source, JobsConfigSource::CliArg);
    }

    #[test]
    fn test_jobs_config_precedence_env_over_default() {
        let config = CompilerJobsConfig::resolve(None, Some("6")).unwrap();
        assert_eq!(config.jobs, 6);
        assert_eq!(config.source, JobsConfigSource::EnvVar);
    }

    #[test]
    fn test_jobs_config_default_policy() {
        let config = CompilerJobsConfig::resolve(None, None).unwrap();
        assert!(config.jobs >= 1 && config.jobs <= 8);
        assert_eq!(config.source, JobsConfigSource::DefaultPolicy);
    }

    #[test]
    fn test_jobs_config_rejects_zero_and_invalid() {
        assert_eq!(
            CompilerJobsConfig::resolve(Some(0), None),
            Err(JobsConfigError::ZeroJobsForbidden)
        );
        assert_eq!(
            CompilerJobsConfig::resolve(None, Some("0")),
            Err(JobsConfigError::ZeroJobsForbidden)
        );
        assert_eq!(
            CompilerJobsConfig::resolve(None, Some("invalid_num")),
            Err(JobsConfigError::InvalidJobCount {
                value: "invalid_num".to_string()
            })
        );
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_build_dedicated_thread_pool_naming() {
        let config = CompilerJobsConfig {
            jobs: 2,
            thread_name_prefix: "test-compile".to_string(),
            source: JobsConfigSource::CliArg,
        };
        let pool = config.build_dedicated_thread_pool().unwrap();
        assert_eq!(pool.current_num_threads(), 2);
    }
}
