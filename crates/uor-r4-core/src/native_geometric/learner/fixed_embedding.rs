//! Continuous Token Embeddings in H4 \oplus \phi H4 and Differentiable Hopf Projections.
//!
//! Maps discrete vocabulary/byte tokens into continuous unit quaternion pairs
//! (q_base, q_companion) \in S3 x S3 and computes differentiable Hopf projections
//! S3 -> S2 onto the Bloch sphere. Supports soft and discrete assignment to the
//! 120 canonical H4 roots of the 600-cell.

use crate::native_geometric::hopf_metric::{
    HopfFiberPointQ30, UnitS2, UnitS2Q30, UnitS3, UnitS3Q30,
};
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
/// Lazy initialization constructs floating canonical roots and converts them to Q1.30.
/// Subsequent calls read the cached fixed-point roots without floating computation.
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
/// Lazy initialization reuses the root cache (whose first construction uses floating geometry).
/// Subsequent calls read cached fixed-point projections without floating computation or allocation.
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
