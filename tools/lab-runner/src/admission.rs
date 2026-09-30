//! Pure admission-control arithmetic over the union of running jobs.

use crate::spec::JobSpec;

pub const MAX_THREADS: u32 = 8;
pub const MAX_RSS_GIB: f64 = 11.0;
pub const MAX_GPU_JOBS: u32 = 1;

/// Aggregate load of the jobs currently in `running/`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Load {
    pub count: usize,
    pub threads: u64,
    /// Declared RSS; a gpu job's `rss_gib` counts as GPU unified memory.
    pub rss_gib: f64,
    pub gpu_jobs: u32,
    pub exclusive_running: bool,
}

impl Load {
    pub fn of(specs: &[JobSpec]) -> Self {
        let mut load = Load::default();
        for spec in specs {
            load.count += 1;
            load.threads += u64::from(spec.threads);
            load.rss_gib += spec.rss_gib;
            if spec.gpu {
                load.gpu_jobs += 1;
            }
            if spec.exclusive {
                load.exclusive_running = true;
            }
        }
        load
    }
}

/// Returns `Ok(())` when `spec` may be admitted alongside `load`, or the
/// human-readable rejection reason.
pub fn admit(load: &Load, spec: &JobSpec) -> std::result::Result<(), String> {
    if spec.exclusive && load.count > 0 {
        return Err("exclusive job requires an empty runner".to_string());
    }
    if load.exclusive_running {
        return Err("an exclusive job is running".to_string());
    }
    if load.threads + u64::from(spec.threads) > u64::from(MAX_THREADS) {
        return Err(format!(
            "threads {} + {} would exceed {MAX_THREADS}",
            load.threads, spec.threads
        ));
    }
    if load.rss_gib + spec.rss_gib > MAX_RSS_GIB {
        return Err(format!(
            "rss {:.2} + {:.2} GiB would exceed {MAX_RSS_GIB} GiB",
            load.rss_gib, spec.rss_gib
        ));
    }
    if spec.gpu && load.gpu_jobs >= MAX_GPU_JOBS {
        return Err("a gpu job is already running".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{JobSpec, KillCriterion, KillKind};
    use std::path::PathBuf;

    fn spec(threads: u32, rss_gib: f64, gpu: bool, exclusive: bool) -> JobSpec {
        JobSpec {
            schema: None,
            id: format!("t{threads}-r{rss_gib}-g{gpu}-e{exclusive}"),
            lab: "lab".to_string(),
            cwd: PathBuf::from("/tmp"),
            argv: vec!["true".to_string()],
            env: None,
            threads,
            rss_gib,
            gpu,
            wall_s: 60,
            stop_grace_ms: 250,
            kill_criterion: KillCriterion {
                kind: KillKind::Wall,
                value: String::new(),
            },
            exclusive,
            cargo: false,
            storage: vec![],
            coordination: None,
            provenance: None,
        }
    }

    #[test]
    fn empty_runner_admits_anything_valid() {
        let load = Load::default();
        assert!(admit(&load, &spec(8, 11.0, true, false)).is_ok());
        assert!(admit(&load, &spec(1, 0.5, false, true)).is_ok());
    }

    #[test]
    fn thread_limit_is_enforced() {
        let load = Load::of(&[spec(6, 1.0, false, false)]);
        assert!(admit(&load, &spec(2, 1.0, false, false)).is_ok());
        assert!(admit(&load, &spec(3, 1.0, false, false)).is_err());
    }

    #[test]
    fn rss_limit_is_enforced() {
        let load = Load::of(&[spec(1, 6.0, false, false)]);
        assert!(admit(&load, &spec(1, 5.0, false, false)).is_ok());
        assert!(admit(&load, &spec(1, 5.1, false, false)).is_err());
    }

    #[test]
    fn at_most_one_gpu_job() {
        let load = Load::of(&[spec(1, 2.0, true, false)]);
        assert!(admit(&load, &spec(1, 2.0, true, false)).is_err());
        assert!(admit(&load, &spec(1, 2.0, false, false)).is_ok());
    }

    #[test]
    fn exclusive_blocks_and_is_blocked() {
        let running = Load::of(&[spec(1, 1.0, false, false)]);
        assert!(admit(&running, &spec(1, 1.0, false, true)).is_err());

        let exclusive_running = Load::of(&[spec(1, 1.0, false, true)]);
        assert!(admit(&exclusive_running, &spec(1, 1.0, false, false)).is_err());
    }

    #[test]
    fn gpu_memory_counts_toward_rss_budget() {
        let load = Load::of(&[spec(1, 8.0, true, false)]);
        assert!(admit(&load, &spec(1, 4.0, false, false)).is_err());
    }
}
