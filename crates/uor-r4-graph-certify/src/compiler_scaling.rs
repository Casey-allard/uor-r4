//! Compiler Parallelism Benchmarks and Scaling Certificate Engine
//!
//! Specification: `docs/compiler_scaling_certificate.md` (Issue #175).
//!
//! Measures, classifies, and certifies compiler stage parallelism across multicore thread sweeps
//! ($T=1, 2, 4, N$) on CPU-only benchmark environments.

use serde::{Deserialize, Serialize};

/// 5-way bottleneck classification taxonomy for compiler stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageScalingClassification {
    /// Stage scales efficiently with increasing worker thread counts ($S(T) \ge 0.7 \times T$).
    Scaling,
    /// Memory bandwidth saturation caps parallel speedup.
    BandwidthLimited,
    /// External teacher probe batching latency dominates runtime.
    TeacherLimited,
    /// Inherent sequential spine (Issue #166 canonical packing and spill).
    SequentialFinalization,
    /// Thread count exceeds physical cores, degrading efficiency.
    OversubscriptionPenalty,
}

/// Execution metrics recorded for a specific compiler stage under thread count $T$.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompilerStageMetrics {
    /// Name of the compiler stage.
    pub stage_name: String,
    /// Number of worker threads ($T$).
    pub thread_count: usize,
    /// Wall-clock execution time in nanoseconds.
    pub wall_clock_ns: u64,
    /// CPU utilization percentage ($0.0$ to $100.0 \times T$).
    pub cpu_utilization_pct: f64,
    /// Peak resident set size (RSS) in bytes.
    pub peak_rss_bytes: usize,
    /// Throughput in items per second.
    pub throughput_items_per_sec: f64,
    /// Confirmation that compiled output bytes match sequential baseline 100%.
    pub byte_equality_verified: bool,
    /// Bottleneck classification.
    pub classification: StageScalingClassification,
}

/// Hardware environment metadata for CPU-only certification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareMetadata {
    pub cpu_model: String,
    pub physical_cores: usize,
    pub logical_threads: usize,
    pub total_memory_bytes: usize,
    pub is_gpu_free: bool,
}

/// Empirical scaling report produced by `CompilerScalingEngine`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompilerScalingReport {
    /// Hardware environment metadata.
    pub hardware: HardwareMetadata,
    /// Name of the evaluated dataset fixture.
    pub dataset_name: String,
    /// Measured metrics per stage across thread sweep.
    pub stage_metrics: Vec<CompilerStageMetrics>,
    /// End-to-end wall-clock time in nanoseconds under max threads.
    pub overall_wall_clock_ns: u64,
    /// Overall speedup $S(T) = T_{\text{wall}}(1) / T_{\text{wall}}(T)$.
    pub overall_speedup: f64,
    /// Overall parallel efficiency $E(T) = S(T) / T$.
    pub overall_efficiency: f64,
    /// Certification status.
    pub is_certified: bool,
}

/// Engine for evaluating and generating compiler scaling certificates.
pub struct CompilerScalingEngine;

impl CompilerScalingEngine {
    /// Compute speedup ratio $S(T) = T_1 / T_N$.
    pub fn compute_speedup(t1_wall_ns: u64, tn_wall_ns: u64) -> f64 {
        if tn_wall_ns == 0 {
            return 1.0;
        }
        (t1_wall_ns as f64) / (tn_wall_ns as f64)
    }

    /// Compute parallel efficiency $E(T) = S(T) / T$.
    pub fn compute_efficiency(speedup: f64, thread_count: usize) -> f64 {
        if thread_count == 0 {
            return 0.0;
        }
        speedup / (thread_count as f64)
    }

    /// Classify stage scaling bottleneck based on speedup, thread count, and sequential flag.
    pub fn classify_stage(
        speedup: f64,
        thread_count: usize,
        physical_cores: usize,
        is_sequential_spine: bool,
    ) -> StageScalingClassification {
        if is_sequential_spine {
            return StageScalingClassification::SequentialFinalization;
        }
        if thread_count > physical_cores {
            return StageScalingClassification::OversubscriptionPenalty;
        }
        let efficiency = Self::compute_efficiency(speedup, thread_count);
        if efficiency >= 0.7 {
            StageScalingClassification::Scaling
        } else {
            StageScalingClassification::BandwidthLimited
        }
    }

    /// Generate a complete scaling report for a benchmark sweep.
    pub fn generate_report(
        hardware: HardwareMetadata,
        dataset_name: impl Into<String>,
        stage_metrics: Vec<CompilerStageMetrics>,
        t1_total_ns: u64,
        tn_total_ns: u64,
        max_threads: usize,
    ) -> CompilerScalingReport {
        let overall_speedup = Self::compute_speedup(t1_total_ns, tn_total_ns);
        let overall_efficiency = Self::compute_efficiency(overall_speedup, max_threads);
        let is_certified =
            hardware.is_gpu_free && stage_metrics.iter().all(|m| m.byte_equality_verified);

        CompilerScalingReport {
            hardware,
            dataset_name: dataset_name.into(),
            stage_metrics,
            overall_wall_clock_ns: tn_total_ns,
            overall_speedup,
            overall_efficiency,
            is_certified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speedup_and_efficiency_computation() {
        let t1 = 1000;
        let t4 = 250;
        let speedup = CompilerScalingEngine::compute_speedup(t1, t4);
        assert!((speedup - 4.0).abs() < 1e-6);

        let efficiency = CompilerScalingEngine::compute_efficiency(speedup, 4);
        assert!((efficiency - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_stage_classification() {
        assert_eq!(
            CompilerScalingEngine::classify_stage(3.2, 4, 8, false),
            StageScalingClassification::Scaling
        );
        assert_eq!(
            CompilerScalingEngine::classify_stage(1.5, 4, 8, false),
            StageScalingClassification::BandwidthLimited
        );
        assert_eq!(
            CompilerScalingEngine::classify_stage(1.0, 4, 8, true),
            StageScalingClassification::SequentialFinalization
        );
        assert_eq!(
            CompilerScalingEngine::classify_stage(2.0, 16, 8, false),
            StageScalingClassification::OversubscriptionPenalty
        );
    }

    #[test]
    fn test_report_generation_and_certification() {
        let hardware = HardwareMetadata {
            cpu_model: "Apple M2 Pro".to_string(),
            physical_cores: 10,
            logical_threads: 10,
            total_memory_bytes: 32 * 1024 * 1024 * 1024,
            is_gpu_free: true,
        };

        let metrics = vec![CompilerStageMetrics {
            stage_name: "cover_induction".to_string(),
            thread_count: 4,
            wall_clock_ns: 250_000_000,
            cpu_utilization_pct: 380.0,
            peak_rss_bytes: 512 * 1024 * 1024,
            throughput_items_per_sec: 4000.0,
            byte_equality_verified: true,
            classification: StageScalingClassification::Scaling,
        }];

        let report = CompilerScalingEngine::generate_report(
            hardware,
            "pinned_mini_corpus",
            metrics,
            1_000_000_000,
            250_000_000,
            4,
        );

        assert!(report.is_certified);
        assert!((report.overall_speedup - 4.0).abs() < 1e-6);
        assert!((report.overall_efficiency - 1.0).abs() < 1e-6);
    }
}
