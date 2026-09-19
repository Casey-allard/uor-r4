//! Continuous Token Embeddings in H4 \oplus \phi H4 and Differentiable Hopf Projections.
//!
//! Maps discrete vocabulary/byte tokens into continuous unit quaternion pairs
//! (q_base, q_companion) \in S3 x S3 and computes differentiable Hopf projections
//! S3 -> S2 onto the Bloch sphere. Supports soft and discrete assignment to the
//! 120 canonical H4 roots of the 600-cell.

use crate::native_geometric::hopf_metric::{
    HopfFiberPointQ30, UnitS2, UnitS2Q30, UnitS3, UnitS3Q30, EPSILON,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Number of canonical roots in the H4 600-cell.
pub const H4_ROOT_COUNT: usize = 120;

/// The golden ratio phi = (1 + sqrt(5)) / 2.
pub const PHI: f64 = 1.6180339887498948482;

/// Get the 120 canonical roots of the 600-cell on S3.
/// Norm is guaranteed to be 1.0.
pub fn canonical_h4_roots() -> &'static [UnitS3; H4_ROOT_COUNT] {
    static ROOTS: OnceLock<[UnitS3; H4_ROOT_COUNT]> = OnceLock::new();
    ROOTS.get_or_init(|| {
        let mut list = Vec::with_capacity(H4_ROOT_COUNT);

        // 1. 8 permutations of (+-1, 0, 0, 0)
        for axis in 0..4 {
            for sign in [-1.0, 1.0] {
                let mut q = [0.0; 4];
                q[axis] = sign;
                list.push(UnitS3::from_array(q).expect("canonical 1-element root"));
            }
        }

        // 2. 16 combinations of (+-1/2, +-1/2, +-1/2, +-1/2)
        for i in 0..16 {
            let s0 = if (i & 1) != 0 { 0.5 } else { -0.5 };
            let s1 = if (i & 2) != 0 { 0.5 } else { -0.5 };
            let s2 = if (i & 4) != 0 { 0.5 } else { -0.5 };
            let s3 = if (i & 8) != 0 { 0.5 } else { -0.5 };
            list.push(UnitS3::from_array([s0, s1, s2, s3]).expect("canonical 1/2 root"));
        }

        // 3. 96 even permutations of (0, +-1/2, +-phi/2, +-(phi-1)/2)
        // 12 even permutations of 4 coordinate slots:
        let even_perms: [[usize; 4]; 12] = [
            [0, 1, 2, 3],
            [0, 2, 3, 1],
            [0, 3, 1, 2],
            [1, 0, 3, 2],
            [1, 2, 0, 3],
            [1, 3, 2, 0],
            [2, 0, 1, 3],
            [2, 1, 3, 0],
            [2, 3, 0, 1],
            [3, 0, 2, 1],
            [3, 1, 0, 2],
            [3, 2, 1, 0],
        ];

        let base_vals = [0.0, 0.5, PHI * 0.5, (PHI - 1.0) * 0.5];

        for perm in &even_perms {
            for s1 in [-1.0, 1.0] {
                for s2 in [-1.0, 1.0] {
                    for s3 in [-1.0, 1.0] {
                        let signed = [
                            base_vals[0],
                            base_vals[1] * s1,
                            base_vals[2] * s2,
                            base_vals[3] * s3,
                        ];
                        let q = [
                            signed[perm[0]],
                            signed[perm[1]],
                            signed[perm[2]],
                            signed[perm[3]],
                        ];
                        list.push(UnitS3::from_array(q).expect("canonical golden root"));
                    }
                }
            }
        }

        assert_eq!(list.len(), H4_ROOT_COUNT);
        let mut arr = [UnitS3::IDENTITY; H4_ROOT_COUNT];
        arr.copy_from_slice(&list);
        arr
    })
}

/// Get the 120 canonical roots of the 600-cell on S3 in Q1.30 fixed-point format.
/// Norm is guaranteed to be 1.0 (Q1.30). Zero runtime floats.
pub fn canonical_h4_roots_q30() -> &'static [UnitS3Q30; H4_ROOT_COUNT] {
    static ROOTS_Q30: OnceLock<[UnitS3Q30; H4_ROOT_COUNT]> = OnceLock::new();
    ROOTS_Q30.get_or_init(|| {
        let roots = canonical_h4_roots();
        let mut list = [UnitS3Q30::IDENTITY; H4_ROOT_COUNT];
        for (i, r) in roots.iter().enumerate() {
            list[i] = r.to_q30();
        }
        list
    })
}

/// Get the fiber-preserving Hopf S3 -> S2 projections of the 120 canonical H4 roots in Q1.30.
/// Zero runtime floats, zero heap allocations.
pub fn canonical_h4_fiber_roots_q30() -> &'static [HopfFiberPointQ30; H4_ROOT_COUNT] {
    static FIBER_ROOTS_Q30: OnceLock<[HopfFiberPointQ30; H4_ROOT_COUNT]> = OnceLock::new();
    FIBER_ROOTS_Q30.get_or_init(|| {
        let roots = canonical_h4_roots_q30();
        let mut list = [HopfFiberPointQ30 {
            base: UnitS2Q30::NORTH_POLE,
            fiber_u1: [1 << 30, 0],
            fiber_phase: 0,
        }; H4_ROOT_COUNT];
        for (i, r) in roots.iter().enumerate() {
            list[i] = r.hopf_fiber_project();
        }
        list
    })
}

/// Get the Hopf S3 -> S2 projections of the 120 canonical H4 roots.
pub fn canonical_h4_hopf_s2() -> &'static [UnitS2; H4_ROOT_COUNT] {
    static S2_ROOTS: OnceLock<[UnitS2; H4_ROOT_COUNT]> = OnceLock::new();
    S2_ROOTS.get_or_init(|| {
        let roots = canonical_h4_roots();
        let mut arr = [UnitS2::NORTH_POLE; H4_ROOT_COUNT];
        for (i, r) in roots.iter().enumerate() {
            arr[i] = r.hopf_map();
        }
        arr
    })
}

/// Continuous Token Embedding layer mapping discrete vocabulary tokens
/// into continuous unit quaternion pairs (base, companion) in H4 \oplus \phi H4.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousEmbedding {
    pub vocab_size: usize,
    /// Unnormalized continuous parameters for base quaternion [a, b, c, d].
    pub weights_base: Vec<[f64; 4]>,
    /// Unnormalized continuous parameters for golden companion quaternion [a, b, c, d].
    pub weights_companion: Vec<[f64; 4]>,
    /// Gradients for base quaternion parameters.
    #[serde(skip)]
    pub grad_base: Vec<[f64; 4]>,
    /// Gradients for companion quaternion parameters.
    #[serde(skip)]
    pub grad_companion: Vec<[f64; 4]>,
    /// Temperature parameter for soft root assignment.
    pub temperature: f64,
}

impl ContinuousEmbedding {
    /// Initialize a continuous embedding table with deterministic pseudo-random initialization.
    pub fn new(vocab_size: usize, seed: u64) -> Self {
        let mut weights_base = Vec::with_capacity(vocab_size);
        let mut weights_companion = Vec::with_capacity(vocab_size);
        let roots = canonical_h4_roots();

        // Use a lightweight LCG PRNG for initialization reproducibility
        let mut rng = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);

        for token in 0..vocab_size {
            // Seed near the canonical root (token % 120) with small continuous perturbation
            let root_idx = token % H4_ROOT_COUNT;
            let root_base = roots[root_idx].to_array();
            let root_comp = roots[(root_idx.wrapping_mul(7) + 13) % H4_ROOT_COUNT].to_array();

            let mut perturb = |base: [f64; 4]| -> [f64; 4] {
                let mut out = base;
                for item in out.iter_mut() {
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let noise = ((rng >> 32) as i32 as f64) / (i32::MAX as f64) * 0.05;
                    *item += noise;
                }
                out
            };

            weights_base.push(perturb(root_base));
            weights_companion.push(perturb(root_comp));
        }

        Self {
            vocab_size,
            weights_base,
            weights_companion,
            grad_base: vec![[0.0; 4]; vocab_size],
            grad_companion: vec![[0.0; 4]; vocab_size],
            temperature: 10.0,
        }
    }

    /// Forward pass: returns normalized unit quaternions (q_base, q_companion) in S3.
    #[inline]
    pub fn forward_quaternions(&self, token: usize) -> (UnitS3, UnitS3) {
        let t = token.min(self.vocab_size.saturating_sub(1));
        let q_base = UnitS3::from_array(self.weights_base[t]).unwrap_or(UnitS3::IDENTITY);
        let q_comp = UnitS3::from_array(self.weights_companion[t]).unwrap_or(UnitS3::IDENTITY);
        (q_base, q_comp)
    }

    /// Forward pass: computes Hopf projections pi(q_base) and pi(q_companion) onto S2.
    #[inline]
    pub fn forward_hopf(&self, token: usize) -> (UnitS2, UnitS2) {
        let (q_base, q_comp) = self.forward_quaternions(token);
        (q_base.hopf_map(), q_comp.hopf_map())
    }

    /// Compute soft distribution p \in \Delta^{120} over canonical H4 roots using S3 cosine similarity.
    pub fn soft_h4_distribution(&self, token: usize) -> [f64; H4_ROOT_COUNT] {
        let (q_base, _) = self.forward_quaternions(token);
        let roots = canonical_h4_roots();

        let mut logits = [0.0; H4_ROOT_COUNT];
        let mut max_logit = f64::NEG_INFINITY;

        for (i, r) in roots.iter().enumerate() {
            // S3 inner product <q_base, r> \in [-1, 1]
            let dot = q_base.a * r.a + q_base.b * r.b + q_base.c * r.c + q_base.d * r.d;
            let logit = dot * self.temperature;
            logits[i] = logit;
            if logit > max_logit {
                max_logit = logit;
            }
        }

        // Numerically stable softmax
        let mut sum_exp = 0.0;
        let mut dist = [0.0; H4_ROOT_COUNT];
        for i in 0..H4_ROOT_COUNT {
            let exp_val = libm::exp(logits[i] - max_logit);
            dist[i] = exp_val;
            sum_exp += exp_val;
        }

        let inv_sum = 1.0 / (sum_exp + EPSILON);
        for item in dist.iter_mut() {
            *item *= inv_sum;
        }

        dist
    }

    /// Find the nearest discrete H4 root index \in 0..120 on S3.
    pub fn nearest_h4_root(&self, token: usize) -> usize {
        let (q_base, _) = self.forward_quaternions(token);
        let roots = canonical_h4_roots();

        let mut best_idx = 0;
        let mut best_dot = f64::NEG_INFINITY;

        for (i, r) in roots.iter().enumerate() {
            let dot = q_base.a * r.a + q_base.b * r.b + q_base.c * r.c + q_base.d * r.d;
            if dot > best_dot {
                best_dot = dot;
                best_idx = i;
            }
        }

        best_idx
    }

    /// Backward pass from unit quaternion gradients to unnormalized parameters.
    pub fn backward_quaternions(
        &mut self,
        token: usize,
        grad_base_s3: [f64; 4],
        grad_comp_s3: [f64; 4],
    ) {
        let t = token.min(self.vocab_size.saturating_sub(1));

        let project_grad = |w: [f64; 4], g: [f64; 4]| -> [f64; 4] {
            let norm_sq = w[0] * w[0] + w[1] * w[1] + w[2] * w[2] + w[3] * w[3];
            if norm_sq < EPSILON {
                return [0.0; 4];
            }
            let norm = libm::sqrt(norm_sq);
            let u = [w[0] / norm, w[1] / norm, w[2] / norm, w[3] / norm];
            let dot_gu = g[0] * u[0] + g[1] * u[1] + g[2] * u[2] + g[3] * u[3];
            [
                (g[0] - dot_gu * u[0]) / norm,
                (g[1] - dot_gu * u[1]) / norm,
                (g[2] - dot_gu * u[2]) / norm,
                (g[3] - dot_gu * u[3]) / norm,
            ]
        };

        let dw_base = project_grad(self.weights_base[t], grad_base_s3);
        let dw_comp = project_grad(self.weights_companion[t], grad_comp_s3);

        for c in 0..4 {
            self.grad_base[t][c] += dw_base[c];
            self.grad_companion[t][c] += dw_comp[c];
        }
    }

    /// Backward pass from Hopf S2 vector gradients to S3 unit quaternion gradients,
    /// then to unnormalized parameters.
    pub fn backward_hopf(&mut self, token: usize, grad_base_s2: [f64; 3], grad_comp_s2: [f64; 3]) {
        let (q_base, q_comp) = self.forward_quaternions(token);

        // Analytical Jacobian adjoint for Hopf map S3 -> S2
        let hopf_adjoint = |q: UnitS3, g: [f64; 3]| -> [f64; 4] {
            let (a, b, c, d) = (q.a, q.b, q.c, q.d);
            let (gx, gy, gz) = (g[0], g[1], g[2]);
            [
                2.0 * (gx * c - gy * d + gz * a),
                2.0 * (gx * d + gy * c + gz * b),
                2.0 * (gx * a + gy * b - gz * c),
                2.0 * (gx * b - gy * a - gz * d),
            ]
        };

        let grad_base_s3 = hopf_adjoint(q_base, grad_base_s2);
        let grad_comp_s3 = hopf_adjoint(q_comp, grad_comp_s2);

        self.backward_quaternions(token, grad_base_s3, grad_comp_s3);
    }

    /// Reset accumulated gradients to zero.
    pub fn zero_grad(&mut self) {
        for g in self.grad_base.iter_mut() {
            *g = [0.0; 4];
        }
        for g in self.grad_companion.iter_mut() {
            *g = [0.0; 4];
        }
    }
}
