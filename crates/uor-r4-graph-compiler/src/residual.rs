//! Residual quantization (Phase 4).
//! Quantizes teacher log-probs into fixed-point ScoreQ residuals.

#[cfg(not(target_arch = "wasm32"))]
use crate::executor::RayonExecutor;
use crate::executor::{CompilerExecutor, SequentialExecutor};
use std::collections::BTreeMap;
use uor_r4_graph_format::ScoreQ;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantizedResidual {
    pub token: u32,
    pub score: ScoreQ,
}

/// Quantize a list of f32 log-probabilities into `ScoreQ` fixed-point entries.
pub fn quantize_logprobs(tokens: &[u32], logprobs: &[f32]) -> Vec<QuantizedResidual> {
    tokens
        .iter()
        .copied()
        .zip(logprobs.iter().copied())
        .map(|(token, logprob)| QuantizedResidual {
            token,
            score: ScoreQ::from_logprob(logprob),
        })
        .collect()
}

/// Quantize log-probabilities with bounded parallel workers while preserving
/// positional output determinism.
pub fn quantize_logprobs_with_threads(
    tokens: &[u32],
    logprobs: &[f32],
    threads: usize,
) -> Vec<QuantizedResidual> {
    // Total: quantize over the common prefix, truncating on any length mismatch
    // exactly as the single-threaded `quantize_logprobs` does (a mismatch is a
    // property of the caller's inputs, not a sanctioned condition).
    let n = tokens.len().min(logprobs.len());
    let indices: Vec<usize> = (0..n).collect();
    map_with_threads(&indices, threads, |&idx| QuantizedResidual {
        token: tokens[idx],
        score: ScoreQ::from_logprob(logprobs[idx]),
    })
}

/// Compute residual delta corrections between child region scores and parent region scores:
/// Delta = ScoreQ(child) - ScoreQ(parent).
pub fn compute_residual_deltas(
    child_residuals: &[QuantizedResidual],
    parent_residuals: &[QuantizedResidual],
) -> Vec<QuantizedResidual> {
    compute_residual_deltas_with_threads(child_residuals, parent_residuals, 1)
}

/// Compute residual delta corrections with canonical key ordering and bounded
/// parallel workers for per-token subtraction.
pub fn compute_residual_deltas_with_threads(
    child_residuals: &[QuantizedResidual],
    parent_residuals: &[QuantizedResidual],
    threads: usize,
) -> Vec<QuantizedResidual> {
    let child_canonical = canonicalize_residuals(child_residuals);
    let parent_by_token = canonicalize_residuals(parent_residuals)
        .into_iter()
        .map(|r| (r.token, r.score))
        .collect::<BTreeMap<_, _>>();

    map_with_threads(&child_canonical, threads, |child| {
        let parent_score = parent_by_token
            .get(&child.token)
            .copied()
            .unwrap_or(ScoreQ::ZERO);
        QuantizedResidual {
            token: child.token,
            score: child.score.saturating_sub(parent_score),
        }
    })
}

fn canonicalize_residuals(residuals: &[QuantizedResidual]) -> Vec<QuantizedResidual> {
    let mut canonical = residuals.to_vec();
    canonical.sort_by(|a, b| {
        a.token
            .cmp(&b.token)
            .then_with(|| b.score.raw().cmp(&a.score.raw()))
    });
    canonical.dedup_by_key(|r| r.token);
    canonical
}

fn map_with_threads<I, O, F>(inputs: &[I], threads: usize, map_fn: F) -> Vec<O>
where
    I: Sync,
    O: Send,
    F: Fn(&I) -> O + Sync,
{
    if threads == 1 {
        return SequentialExecutor::new().map(inputs, map_fn);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        RayonExecutor::new(threads).map(inputs, map_fn)
    }
    #[cfg(target_arch = "wasm32")]
    {
        SequentialExecutor::new().map(inputs, map_fn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantize_and_delta() {
        let tokens = vec![1, 2, 3];
        let child_lps = vec![-0.5, -1.2, -2.0];
        let parent_lps = vec![-1.0, -1.2, -3.0];

        let child_q = quantize_logprobs(&tokens, &child_lps);
        let parent_q = quantize_logprobs(&tokens, &parent_lps);
        let deltas = compute_residual_deltas(&child_q, &parent_q);

        assert_eq!(deltas.len(), 3);
        assert!(deltas[0].score.raw() > 0); // child was better than parent
        assert_eq!(deltas[1].score.raw(), 0); // child and parent equal
        assert!(deltas[2].score.raw() > 0); // child better than parent
    }

    #[test]
    fn quantization_is_identical_across_thread_counts() {
        let tokens = vec![10, 11, 12, 13, 14, 15];
        let logprobs = vec![-0.1, -0.7, -1.3, -2.2, -0.4, -9.9];

        let seq = quantize_logprobs_with_threads(&tokens, &logprobs, 1);
        let par2 = quantize_logprobs_with_threads(&tokens, &logprobs, 2);
        let par4 = quantize_logprobs_with_threads(&tokens, &logprobs, 4);

        assert_eq!(seq, par2);
        assert_eq!(seq, par4);
    }

    #[test]
    fn quantize_logprobs_truncates_on_length_mismatch() {
        let tokens = vec![1, 2, 3];
        let logprobs = vec![-0.5, -1.2];

        let quantized = quantize_logprobs(&tokens, &logprobs);
        assert_eq!(quantized.len(), 2);
        assert_eq!(quantized[0].token, 1);
        assert_eq!(quantized[1].token, 2);
    }

    #[test]
    fn quantization_threads_zero_matches_sequential() {
        let tokens = vec![10, 11, 12, 13, 14, 15];
        let logprobs = vec![-0.1, -0.7, -1.3, -2.2, -0.4, -9.9];

        let seq = quantize_logprobs_with_threads(&tokens, &logprobs, 1);
        let auto = quantize_logprobs_with_threads(&tokens, &logprobs, 0);
        assert_eq!(seq, auto);
    }

    #[test]
    fn residual_deltas_are_canonical_and_thread_invariant() {
        let child = vec![
            QuantizedResidual {
                token: 7,
                score: ScoreQ::from_raw(100),
            },
            QuantizedResidual {
                token: 5,
                score: ScoreQ::from_raw(300),
            },
            QuantizedResidual {
                token: 5,
                score: ScoreQ::from_raw(250),
            },
        ];
        let parent = vec![
            QuantizedResidual {
                token: 7,
                score: ScoreQ::from_raw(80),
            },
            QuantizedResidual {
                token: 5,
                score: ScoreQ::from_raw(120),
            },
        ];

        let seq = compute_residual_deltas_with_threads(&child, &parent, 1);
        let par = compute_residual_deltas_with_threads(&child, &parent, 4);
        assert_eq!(seq, par);
        assert_eq!(seq.len(), 2);
        assert_eq!(seq[0].token, 5);
        assert_eq!(seq[1].token, 7);
    }
}
