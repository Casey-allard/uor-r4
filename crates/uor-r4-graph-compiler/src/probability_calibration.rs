//! Compiler-side calibration summaries for probability-aware evaluations.
//!
//! This module is deliberately outside the deployed graph runtime. It uses
//! teacher probability metadata to condition evaluation results on teacher
//! uncertainty; it does not participate in artifact execution or serving.

use crate::observation::{self, ProbabilityMetadata};

/// One deterministic teacher-entropy quartile of an A/B evaluation.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct EntropyBucket {
    pub samples: u64,
    pub entropy_bits: f64,
    pub context_hits: u64,
    pub session_hits: u64,
    pub context_teacher_hits: u64,
    pub session_teacher_hits: u64,
    pub session_token_changes: u64,
    pub session_corrections: u64,
    pub session_regressions: u64,
}

impl EntropyBucket {
    pub fn record(
        &mut self,
        entropy_bits: f32,
        context_token: u32,
        session_token: u32,
        target: u32,
        teacher_argmax: u32,
    ) {
        self.samples += 1;
        self.entropy_bits += f64::from(entropy_bits);
        self.context_hits += u64::from(context_token == target);
        self.session_hits += u64::from(session_token == target);
        self.context_teacher_hits += u64::from(context_token == teacher_argmax);
        self.session_teacher_hits += u64::from(session_token == teacher_argmax);
        self.session_token_changes += u64::from(session_token != context_token);
        self.session_corrections += u64::from(context_token != target && session_token == target);
        self.session_regressions += u64::from(context_token == target && session_token != target);
    }

    pub fn mean_entropy_bits(self) -> f64 {
        if self.samples == 0 {
            0.0
        } else {
            self.entropy_bits / self.samples as f64
        }
    }
}

/// Return the inclusive quartile bucket for a teacher entropy value.
pub fn entropy_bucket(entropy_bits: f32, quartiles: &[f32; 3]) -> usize {
    quartiles
        .iter()
        .position(|threshold| entropy_bits <= *threshold)
        .unwrap_or(3)
}

/// Compute deterministic quartile thresholds over sampled positions.
pub fn entropy_quartiles(
    positions: &[usize],
    metadata: Option<&[ProbabilityMetadata]>,
) -> Option<[f32; 3]> {
    let metadata = metadata?;
    let mut values: Vec<f32> = positions
        .iter()
        .map(|&position| metadata[position].entropy_bits)
        .collect();
    values.sort_by(f32::total_cmp);
    if values.is_empty() {
        return None;
    }
    Some([
        values[values.len() / 4],
        values[values.len() / 2],
        values[values.len() * 3 / 4],
    ])
}

/// Compute teacher-forced information for sampled held-out positions.
pub fn sampled_teacher_bits_per_token(
    positions: &[usize],
    metadata: &[ProbabilityMetadata],
) -> Option<f64> {
    let sampled: Vec<_> = positions
        .iter()
        .map(|&position| metadata[position])
        .collect();
    observation::message_bits_per_token(&sampled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropy_buckets_are_monotonic_and_inclusive() {
        assert_eq!(entropy_bucket(1.0, &[1.0, 2.0, 3.0]), 0);
        assert_eq!(entropy_bucket(2.0, &[1.0, 2.0, 3.0]), 1);
        assert_eq!(entropy_bucket(3.0, &[1.0, 2.0, 3.0]), 2);
        assert_eq!(entropy_bucket(4.0, &[1.0, 2.0, 3.0]), 3);
    }

    #[test]
    fn bucket_records_corrections_and_regressions_separately() {
        let mut bucket = EntropyBucket::default();
        bucket.record(2.0, 10, 11, 11, 11);
        bucket.record(2.0, 10, 12, 10, 10);
        bucket.record(2.0, 10, 10, 10, 10);

        assert_eq!(bucket.samples, 3);
        assert_eq!(bucket.session_corrections, 1);
        assert_eq!(bucket.session_regressions, 1);
        assert_eq!(bucket.session_token_changes, 2);
        assert_eq!(bucket.mean_entropy_bits(), 2.0);
    }

    #[test]
    fn quartiles_use_stable_sorted_positions() {
        let metadata = (0..8)
            .map(|index| ProbabilityMetadata {
                target_logprob_nats: -1.0,
                entropy_bits: index as f32,
                top8_mass: 0.5,
                target_rank: u16::MAX,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            entropy_quartiles(&[7, 0, 6, 1, 5, 2, 4, 3], Some(&metadata)),
            Some([2.0, 4.0, 6.0])
        );
    }
}
