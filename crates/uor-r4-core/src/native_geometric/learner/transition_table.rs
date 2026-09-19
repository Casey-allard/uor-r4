//! Differentiable 120x120 Geometric Compatibility Tables and Discrete Quantization.
//!
//! Replaces symmetric group distance and discrete Hamming angle collapse with
//! learned pairwise compatibility matrices (14,400 entries per lane).
//! At runtime, query-key interaction is an exact O(1) table lookup:
//! table[query_root * 120 + key_root] with zero matrix multiplications and zero floats.

use super::embedding::H4_ROOT_COUNT;
use serde::{Deserialize, Serialize};

/// Number of entries in a 120x120 compatibility matrix: 14,400.
pub const ENTRIES_PER_TABLE: usize = H4_ROOT_COUNT * H4_ROOT_COUNT;

/// Continuous compatibility table for a single attention/routing lane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousLaneTable {
    /// Continuous compatibility logits M[q, k], row-major order: index = q * 120 + k.
    pub logits: Vec<f64>,
    /// Accumulated gradients during offline training.
    #[serde(skip)]
    pub grad: Vec<f64>,
}

impl ContinuousLaneTable {
    /// Initialize with zero or small random logits.
    pub fn new(seed: u64) -> Self {
        let mut logits = Vec::with_capacity(ENTRIES_PER_TABLE);
        let mut rng = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);

        for _ in 0..ENTRIES_PER_TABLE {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = ((rng >> 32) as i32 as f64) / (i32::MAX as f64) * 0.02;
            logits.push(val);
        }

        Self {
            logits,
            grad: vec![0.0; ENTRIES_PER_TABLE],
        }
    }

    /// Read continuous score for discrete query and key roots.
    #[inline]
    pub fn score_discrete(&self, query_root: usize, key_root: usize) -> f64 {
        let q = query_root.min(H4_ROOT_COUNT - 1);
        let k = key_root.min(H4_ROOT_COUNT - 1);
        self.logits[q * H4_ROOT_COUNT + k]
    }

    /// Compute expected score under soft distributions over roots.
    pub fn score_soft(
        &self,
        query_dist: &[f64; H4_ROOT_COUNT],
        key_dist: &[f64; H4_ROOT_COUNT],
    ) -> f64 {
        let mut total = 0.0;
        for q in 0..H4_ROOT_COUNT {
            let pq = query_dist[q];
            if pq < 1e-6 {
                continue;
            }
            let row_offset = q * H4_ROOT_COUNT;
            for k in 0..H4_ROOT_COUNT {
                total += pq * key_dist[k] * self.logits[row_offset + k];
            }
        }
        total
    }

    /// Backward pass for discrete query and key roots.
    #[inline]
    pub fn backward_discrete(&mut self, query_root: usize, key_root: usize, grad_loss: f64) {
        let q = query_root.min(H4_ROOT_COUNT - 1);
        let k = key_root.min(H4_ROOT_COUNT - 1);
        self.grad[q * H4_ROOT_COUNT + k] += grad_loss;
    }

    /// Backward pass for soft distributions.
    pub fn backward_soft(
        &mut self,
        query_dist: &[f64; H4_ROOT_COUNT],
        key_dist: &[f64; H4_ROOT_COUNT],
        grad_loss: f64,
    ) {
        for q in 0..H4_ROOT_COUNT {
            let pq = query_dist[q];
            if pq < 1e-6 {
                continue;
            }
            let row_offset = q * H4_ROOT_COUNT;
            for k in 0..H4_ROOT_COUNT {
                self.grad[row_offset + k] += pq * key_dist[k] * grad_loss;
            }
        }
    }

    /// Quantize continuous logits to discrete integer / Q1.15 table for serving.
    /// Uses fixed scale 819.2 (0.1 continuous lane weight * 8192.0) matching Q1.13 fixed-point logit scale.
    pub fn quantize(&self) -> DiscreteServingTable {
        let scale = 819.2;
        let mut scores = Vec::with_capacity(ENTRIES_PER_TABLE);

        for &val in &self.logits {
            let quantized = (val * scale).round().clamp(-32767.0, 32767.0) as i16;
            scores.push(quantized);
        }

        DiscreteServingTable {
            scores,
            scale_bits: scale.to_bits(),
        }
    }

    /// Reset accumulated gradients to zero.
    pub fn zero_grad(&mut self) {
        for g in self.grad.iter_mut() {
            *g = 0.0;
        }
    }
}

/// Discrete serving table for zero-runtime-float, zero-matmul O(1) query-key compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscreteServingTable {
    /// 14,400 integer compatibility entries indexed by `query_root * 120 + key_root`.
    pub scores: Vec<i16>,
    /// Quantization scale factor bits (used for verification/testing only; not required at runtime).
    pub scale_bits: u64,
}

impl DiscreteServingTable {
    /// Quantization scale factor.
    #[inline]
    pub fn scale(&self) -> f64 {
        f64::from_bits(self.scale_bits)
    }

    /// O(1) table lookup with zero matrix multiplications and zero heap allocations.
    #[inline]
    pub fn score(&self, query_root: usize, key_root: usize) -> i16 {
        let q = query_root.min(H4_ROOT_COUNT - 1);
        let k = key_root.min(H4_ROOT_COUNT - 1);
        self.scores[q * H4_ROOT_COUNT + k]
    }
}

/// Multi-lane continuous transition tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiLaneTransitionTables {
    pub num_lanes: usize,
    pub lanes: Vec<ContinuousLaneTable>,
}

impl MultiLaneTransitionTables {
    pub fn new(num_lanes: usize, seed: u64) -> Self {
        let mut lanes = Vec::with_capacity(num_lanes);
        for i in 0..num_lanes {
            lanes.push(ContinuousLaneTable::new(seed.wrapping_add((i as u64) * 31)));
        }
        Self { num_lanes, lanes }
    }

    pub fn quantize(&self) -> Vec<DiscreteServingTable> {
        self.lanes.iter().map(|lane| lane.quantize()).collect()
    }

    pub fn zero_grad(&mut self) {
        for lane in self.lanes.iter_mut() {
            lane.zero_grad();
        }
    }
}
