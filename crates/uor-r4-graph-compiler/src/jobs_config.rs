//! Compiler Thread-Pool, Jobs Configuration & Oversubscription Policy (#168).
//!
//! Provides explicit thread concurrency resolution (`CLI > env > default`),
//! typed error validation, and dedicated custom Rayon thread pool construction.

use serde::{Deserialize, Serialize};

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

    /// Resolve configuration following precedence: CLI (`--jobs`) > Env
    /// (`R4_COMPILER_THREADS`) > Default. `None` when the requested count is not
    /// a valid configuration — a CLI/env value of 0, or an env string that is
    /// not a decimal integer (R5 — the absence of a valid config, not a raised
    /// error). An empty/absent env value falls through to the default policy.
    pub fn resolve(cli_jobs: Option<usize>, env_val: Option<&str>) -> Option<Self> {
        // Tier 1: CLI argument
        if let Some(jobs) = cli_jobs {
            if jobs == 0 {
                return None;
            }
            return Some(CompilerJobsConfig {
                jobs,
                thread_name_prefix: "r4-compile".to_string(),
                source: JobsConfigSource::CliArg,
            });
        }

        // Tier 2: Environment variable
        if let Some(env_str) = env_val {
            let trimmed = env_str.trim();
            if !trimmed.is_empty() {
                let parsed: usize = trimmed.parse().ok()?;
                if parsed == 0 {
                    return None;
                }
                return Some(CompilerJobsConfig {
                    jobs: parsed,
                    thread_name_prefix: "r4-compile".to_string(),
                    source: JobsConfigSource::EnvVar,
                });
            }
        }

        // Tier 3: Default policy
        Some(CompilerJobsConfig::default())
    }

    /// Build a dedicated named Rayon thread pool owned by the compiler context.
    /// Total: a thread pool that will not build is a host defect (cannot spawn
    /// threads), so it propagates as a panic rather than a sanctioned condition
    /// (R5), matching [`crate::executor::RayonExecutor::new`].
    #[cfg(not(target_arch = "wasm32"))]
    pub fn build_dedicated_thread_pool(&self) -> rayon::ThreadPool {
        let prefix = self.thread_name_prefix.clone();
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.jobs)
            .thread_name(move |idx| format!("{prefix}-{idx}"))
            .build()
            .unwrap_or_else(|e| panic!("compiler dedicated thread pool construction failed: {e:?}"))
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
        assert_eq!(CompilerJobsConfig::resolve(Some(0), None), None);
        assert_eq!(CompilerJobsConfig::resolve(None, Some("0")), None);
        assert_eq!(CompilerJobsConfig::resolve(None, Some("invalid_num")), None);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_build_dedicated_thread_pool_naming() {
        let config = CompilerJobsConfig {
            jobs: 2,
            thread_name_prefix: "test-compile".to_string(),
            source: JobsConfigSource::CliArg,
        };
        let pool = config.build_dedicated_thread_pool();
        assert_eq!(pool.current_num_threads(), 2);
    }
}
