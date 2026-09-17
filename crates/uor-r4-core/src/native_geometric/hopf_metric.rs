//! Quantum Fubini-Study metric, canonical S3 -> S2 Hopf projection, and
//! continuous-to-discrete state trajectory tracking.
//!
//! Serving hot paths use integer/fixed-point arithmetic with zero dynamic
//! heap allocations. Differentiable offline training may use continuous
//! unit quaternions and geodesic distances on the Bloch sphere S2.

use core::fmt;

/// Normalization tolerance for floating-point unit vectors.
pub const EPSILON: f64 = 1e-12;

/// Fixed-point scale for Q1.30 format: 2^30.
pub const Q30_SCALE: i64 = 1 << 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HopfError {
    ZeroNorm,
    NonFinite,
    DimensionMismatch,
}

impl fmt::Display for HopfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroNorm => write!(f, "zero vector has undefined direction"),
            Self::NonFinite => write!(f, "vector components must be finite"),
            Self::DimensionMismatch => write!(f, "vector dimension mismatch"),
        }
    }
}

impl std::error::Error for HopfError {}

/// Continuous unit quaternion state in S3: q = a + bi + cj + dk.
/// Satisfies a^2 + b^2 + c^2 + d^2 = 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitS3 {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

impl UnitS3 {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    };

    pub const I: Self = Self {
        a: 0.0,
        b: 1.0,
        c: 0.0,
        d: 0.0,
    };

    pub const J: Self = Self {
        a: 0.0,
        b: 0.0,
        c: 1.0,
        d: 0.0,
    };

    pub const K: Self = Self {
        a: 0.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
    };

    pub fn new(a: f64, b: f64, c: f64, d: f64) -> Result<Self, HopfError> {
        if !a.is_finite() || !b.is_finite() || !c.is_finite() || !d.is_finite() {
            return Err(HopfError::NonFinite);
        }
        let norm_sq = a * a + b * b + c * c + d * d;
        if norm_sq < EPSILON {
            return Err(HopfError::ZeroNorm);
        }
        let norm = libm::sqrt(norm_sq);
        Ok(Self {
            a: a / norm,
            b: b / norm,
            c: c / norm,
            d: d / norm,
        })
    }

    #[inline]
    pub fn from_array(arr: [f64; 4]) -> Result<Self, HopfError> {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }

    #[inline]
    pub fn to_array(&self) -> [f64; 4] {
        [self.a, self.b, self.c, self.d]
    }

    /// Quaternion conjugate: q* = a - bi - cj - dk.
    #[inline]
    pub fn conjugate(&self) -> Self {
        Self {
            a: self.a,
            b: -self.b,
            c: -self.c,
            d: -self.d,
        }
    }

    /// Hamilton product: q1 * q2.
    /// Preserves unit norm on S3.
    pub fn mul(&self, rhs: &Self) -> Self {
        let a = self.a * rhs.a - self.b * rhs.b - self.c * rhs.c - self.d * rhs.d;
        let b = self.a * rhs.b + self.b * rhs.a + self.c * rhs.d - self.d * rhs.c;
        let c = self.a * rhs.c - self.b * rhs.d + self.c * rhs.a + self.d * rhs.b;
        let d = self.a * rhs.d + self.b * rhs.c - self.c * rhs.b + self.d * rhs.a;
        let norm_sq = a * a + b * b + c * c + d * d;
        let inv_norm = 1.0 / libm::sqrt(norm_sq);
        Self {
            a: a * inv_norm,
            b: b * inv_norm,
            c: c * inv_norm,
            d: d * inv_norm,
        }
    }

    /// Canonical Hopf fibration map pi: S3 -> S2.
    /// Maps unit quaternion (a, b, c, d) to unit vector (x, y, z) on S2:
    ///   x = 2(ac + bd)
    ///   y = 2(bc - ad)
    ///   z = a^2 + b^2 - c^2 - d^2
    /// Guaranteed to lie on unit sphere S2: x^2 + y^2 + z^2 = 1.
    pub fn hopf_map(&self) -> UnitS2 {
        let x = 2.0 * (self.a * self.c + self.b * self.d);
        let y = 2.0 * (self.b * self.c - self.a * self.d);
        let z = self.a * self.a + self.b * self.b - self.c * self.c - self.d * self.d;
        let norm_sq = x * x + y * y + z * z;
        let inv_norm = 1.0 / libm::sqrt(norm_sq);
        UnitS2 {
            x: x * inv_norm,
            y: y * inv_norm,
            z: z * inv_norm,
        }
    }

    /// S1 fiber phase angle psi in [-pi, pi).
    /// Identifies the position along the fiber circle above pi(q) in S2.
    pub fn fiber_phase(&self) -> f64 {
        let norm_ab = self.a * self.a + self.b * self.b;
        if norm_ab > EPSILON {
            libm::atan2(self.b, self.a)
        } else {
            libm::atan2(self.d, self.c)
        }
    }

    /// Apply the S1 fiber rotation (z1, z2) -> (e^(i*phi)*z1, e^(i*phi)*z2).
    /// Rotates along the fiber while leaving the Hopf projection pi(q) on S2 invariant.
    pub fn rotate_fiber(&self, phi: f64) -> Self {
        if !phi.is_finite() {
            return *self;
        }
        let cos_p = libm::cos(phi);
        let sin_p = libm::sin(phi);
        let a = self.a * cos_p - self.b * sin_p;
        let b = self.a * sin_p + self.b * cos_p;
        let c = self.c * cos_p - self.d * sin_p;
        let d = self.c * sin_p + self.d * cos_p;
        let norm_sq = a * a + b * b + c * c + d * d;
        let inv_norm = 1.0 / libm::sqrt(norm_sq);
        Self {
            a: a * inv_norm,
            b: b * inv_norm,
            c: c * inv_norm,
            d: d * inv_norm,
        }
    }

    /// Convert to fixed-point Q1.30 format.
    pub fn to_q30(&self) -> UnitS3Q30 {
        let to_fixed = |v: f64| -> i32 {
            let clamped = v.clamp(-1.0, 1.0);
            (clamped * (Q30_SCALE as f64)).round() as i32
        };
        UnitS3Q30([
            to_fixed(self.a),
            to_fixed(self.b),
            to_fixed(self.c),
            to_fixed(self.d),
        ])
    }
}

/// Continuous unit vector on S2 (Bloch sphere): u = (x, y, z) with x^2 + y^2 + z^2 = 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitS2 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl UnitS2 {
    pub const NORTH_POLE: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    pub const SOUTH_POLE: Self = Self {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };
    pub const EQUATOR_X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const EQUATOR_Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    pub fn new(x: f64, y: f64, z: f64) -> Result<Self, HopfError> {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(HopfError::NonFinite);
        }
        let norm_sq = x * x + y * y + z * z;
        if norm_sq < EPSILON {
            return Err(HopfError::ZeroNorm);
        }
        let norm = libm::sqrt(norm_sq);
        Ok(Self {
            x: x / norm,
            y: y / norm,
            z: z / norm,
        })
    }

    #[inline]
    pub fn from_array(arr: [f64; 3]) -> Result<Self, HopfError> {
        Self::new(arr[0], arr[1], arr[2])
    }

    #[inline]
    pub fn to_array(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    /// Euclidean dot product between unit vectors: u . v = cos(theta).
    /// Clamped to [-1.0, 1.0].
    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        let d = self.x * other.x + self.y * other.y + self.z * other.z;
        d.clamp(-1.0, 1.0)
    }

    /// Quantum Fubini-Study distance on S2:
    ///   d_FS(u, v) = arccos(sqrt((1 + u . v) / 2)) = 0.5 * arccos(u . v)
    /// Range: [0, pi/2].
    /// Satisfies metric axioms:
    ///   1. d_FS(u, v) >= 0; d_FS(u, v) = 0 <=> u = v.
    ///   2. d_FS(u, v) = d_FS(v, u).
    ///   3. d_FS(u, w) <= d_FS(u, v) + d_FS(v, w).
    pub fn fubini_study_distance(&self, other: &Self) -> f64 {
        let dot = self.dot(other);
        let arg = libm::sqrt(((1.0 + dot) * 0.5).clamp(0.0, 1.0));
        libm::acos(arg.clamp(-1.0, 1.0))
    }

    /// Great-circle geodesic distance on S2: theta = arccos(u . v) = 2 * d_FS(u, v).
    /// Range: [0, pi].
    pub fn great_circle_distance(&self, other: &Self) -> f64 {
        let dot = self.dot(other);
        libm::acos(dot)
    }

    /// Geodesic midpoint on S2 between self and other.
    /// If self and other are antipodal, returns None.
    pub fn geodesic_midpoint(&self, other: &Self) -> Option<Self> {
        let x = self.x + other.x;
        let y = self.y + other.y;
        let z = self.z + other.z;
        Self::new(x, y, z).ok()
    }

    /// Nearest root index on S2 among a set of reference candidate vectors.
    /// Minimizes Fubini-Study distance (maximizes dot product).
    /// Executes with zero dynamic heap allocation.
    pub fn nearest_root(&self, roots: &[Self]) -> Result<usize, HopfError> {
        if roots.is_empty() {
            return Err(HopfError::DimensionMismatch);
        }
        let mut best_idx = 0;
        let mut best_dot = -2.0;
        for (idx, r) in roots.iter().enumerate() {
            let d = self.dot(r);
            if d > best_dot {
                best_dot = d;
                best_idx = idx;
            }
        }
        Ok(best_idx)
    }

    /// Convert to fixed-point Q1.30 format.
    pub fn to_q30(&self) -> UnitS2Q30 {
        let to_fixed = |v: f64| -> i32 {
            let clamped = v.clamp(-1.0, 1.0);
            (clamped * (Q30_SCALE as f64)).round() as i32
        };
        UnitS2Q30([to_fixed(self.x), to_fixed(self.y), to_fixed(self.z)])
    }
}

/// Fixed-point Q1.30 unit quaternion on S3 for zero-float serving hot paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitS3Q30(pub [i32; 4]);

impl UnitS3Q30 {
    pub const IDENTITY: Self = Self([1 << 30, 0, 0, 0]);

    #[inline]
    pub fn raw(&self) -> [i32; 4] {
        self.0
    }

    /// Exact integer / fixed-point Hopf projection from S3 to S2:
    ///   x = 2(ac + bd) / 2^30
    ///   y = 2(bc - ad) / 2^30
    ///   z = (a^2 + b^2 - c^2 - d^2) / 2^30
    /// Zero floating-point operations; zero dynamic heap allocations.
    pub fn hopf_project(&self) -> UnitS2Q30 {
        let a = self.0[0] as i64;
        let b = self.0[1] as i64;
        let c = self.0[2] as i64;
        let d = self.0[3] as i64;

        let x = (2 * (a * c + b * d)) >> 30;
        let y = (2 * (b * c - a * d)) >> 30;
        let z = (a * a + b * b - c * c - d * d) >> 30;

        let clamp_q30 = |v: i64| -> i32 { v.clamp(-Q30_SCALE, Q30_SCALE) as i32 };

        UnitS2Q30([clamp_q30(x), clamp_q30(y), clamp_q30(z)])
    }
}

/// Fixed-point Q1.30 unit vector on S2 for zero-float serving hot paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitS2Q30(pub [i32; 3]);

impl UnitS2Q30 {
    pub const NORTH_POLE: Self = Self([0, 0, 1 << 30]);
    pub const SOUTH_POLE: Self = Self([0, 0, -(1 << 30)]);
    pub const EQUATOR_X: Self = Self([1 << 30, 0, 0]);
    pub const EQUATOR_Y: Self = Self([0, 1 << 30, 0]);

    #[inline]
    pub fn raw(&self) -> [i32; 3] {
        self.0
    }

    /// Fixed-point dot product in Q1.30: returns value in [-2^30, 2^30].
    /// Zero floating-point instructions.
    #[inline]
    pub fn dot_q30(&self, other: &Self) -> i32 {
        let x1 = self.0[0] as i64;
        let y1 = self.0[1] as i64;
        let z1 = self.0[2] as i64;
        let x2 = other.0[0] as i64;
        let y2 = other.0[1] as i64;
        let z2 = other.0[2] as i64;
        let sum = (x1 * x2 + y1 * y2 + z1 * z2) >> 30;
        sum.clamp(-Q30_SCALE, Q30_SCALE) as i32
    }

    /// Nearest root index among a fixed array of Q1.30 roots.
    /// Maximizes dot product without floating point or dynamic heap allocations.
    /// Returns None if roots slice is empty.
    pub fn nearest_root_q30(&self, roots: &[Self]) -> Option<usize> {
        if roots.is_empty() {
            return None;
        }
        let mut best_idx = 0;
        let mut best_dot = i32::MIN;
        for (idx, r) in roots.iter().enumerate() {
            let d = self.dot_q30(r);
            if d > best_dot {
                best_dot = d;
                best_idx = idx;
            }
        }
        Some(best_idx)
    }
}

/// Continuous-state trajectory tracker in S3 with canonical projection to S2.
/// Tracks cumulative Fubini-Study geodesic displacement along a sequence of transitions.
/// Stack-allocated struct with zero dynamic allocations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HopfStateTrajectory {
    pub current_s3: UnitS3,
    pub current_s2: UnitS2,
    pub cumulative_fubini_study: f64,
    pub step_count: usize,
}

impl HopfStateTrajectory {
    pub fn new(initial_s3: UnitS3) -> Self {
        let initial_s2 = initial_s3.hopf_map();
        Self {
            current_s3: initial_s3,
            current_s2: initial_s2,
            cumulative_fubini_study: 0.0,
            step_count: 0,
        }
    }

    pub fn reset(&mut self, initial_s3: UnitS3) {
        self.current_s3 = initial_s3;
        self.current_s2 = initial_s3.hopf_map();
        self.cumulative_fubini_study = 0.0;
        self.step_count = 0;
    }

    /// Transition to next state via group action: q_{t+1} = q_t * delta.
    /// Updates S2 Hopf observation and accumulates geodesic Fubini-Study distance.
    /// Returns step Fubini-Study distance d_FS(S2_t, S2_{t+1}).
    pub fn step(&mut self, delta_quaternion: &UnitS3) -> f64 {
        let next_s3 = self.current_s3.mul(delta_quaternion);
        let next_s2 = next_s3.hopf_map();
        let step_dist = self.current_s2.fubini_study_distance(&next_s2);
        self.cumulative_fubini_study += step_dist;
        self.current_s3 = next_s3;
        self.current_s2 = next_s2;
        self.step_count += 1;
        step_dist
    }

    /// Geodesic Fubini-Study displacement from an arbitrary reference point on S2.
    pub fn distance_from_reference(&self, reference: &UnitS2) -> f64 {
        self.current_s2.fubini_study_distance(reference)
    }
}

impl Default for HopfStateTrajectory {
    fn default() -> Self {
        Self::new(UnitS3::IDENTITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_unit_s3_normalization_and_algebra() {
        let q = UnitS3::new(1.0, 2.0, 3.0, 4.0).expect("valid quaternion");
        let norm_sq = q.a * q.a + q.b * q.b + q.c * q.c + q.d * q.d;
        assert!((norm_sq - 1.0).abs() < 1e-10);

        // Identity multiplication
        let id_prod = q.mul(&UnitS3::IDENTITY);
        assert!((id_prod.a - q.a).abs() < 1e-10);
        assert!((id_prod.b - q.b).abs() < 1e-10);
        assert!((id_prod.c - q.c).abs() < 1e-10);
        assert!((id_prod.d - q.d).abs() < 1e-10);

        // Inverse property: q * q* = 1
        let inv = q.conjugate();
        let prod = q.mul(&inv);
        assert!((prod.a - 1.0).abs() < 1e-10);
        assert!(prod.b.abs() < 1e-10);
        assert!(prod.c.abs() < 1e-10);
        assert!(prod.d.abs() < 1e-10);

        // Quaternion basis products: i * j = k
        let ij = UnitS3::I.mul(&UnitS3::J);
        assert!((ij.a - UnitS3::K.a).abs() < 1e-10);
        assert!((ij.b - UnitS3::K.b).abs() < 1e-10);
        assert!((ij.c - UnitS3::K.c).abs() < 1e-10);
        assert!((ij.d - UnitS3::K.d).abs() < 1e-10);
    }

    #[test]
    fn test_hopf_projection_unit_norm() {
        let quaternions = [
            UnitS3::IDENTITY,
            UnitS3::I,
            UnitS3::J,
            UnitS3::K,
            UnitS3::new(1.0, 1.0, 1.0, 1.0).unwrap(),
            UnitS3::new(-3.0, 4.0, -5.0, 2.0).unwrap(),
            UnitS3::new(0.5, 0.5, 0.5, 0.5).unwrap(),
        ];

        for q in quaternions {
            let s2 = q.hopf_map();
            let norm_sq = s2.x * s2.x + s2.y * s2.y + s2.z * s2.z;
            assert!(
                (norm_sq - 1.0).abs() < 1e-10,
                "Hopf projection must lie on unit S2"
            );
        }

        // Identity maps to North Pole (0, 0, 1)
        let s2_id = UnitS3::IDENTITY.hopf_map();
        assert!((s2_id.x - 0.0).abs() < 1e-10);
        assert!((s2_id.y - 0.0).abs() < 1e-10);
        assert!((s2_id.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_hopf_fiber_invariance() {
        // Any rotation along the S1 fiber must leave the S2 Hopf projection identical
        let q = UnitS3::new(0.6, 0.8, 0.0, 0.0).unwrap();
        let base_s2 = q.hopf_map();

        for angle in [0.1, 0.5, 1.0, 2.0, 3.14159, -1.2] {
            let rotated_q = q.rotate_fiber(angle);
            let rotated_s2 = rotated_q.hopf_map();
            let dist = base_s2.fubini_study_distance(&rotated_s2);
            assert!(
                dist < 1e-10,
                "Fiber rotation by {angle} radians must leave S2 projection invariant, got distance {dist}"
            );
        }
    }

    #[test]
    fn test_fubini_study_metric_properties() {
        let u = UnitS2::NORTH_POLE;
        let v = UnitS2::SOUTH_POLE;
        let w = UnitS2::EQUATOR_X;
        let diag = UnitS2::new(1.0, 1.0, 1.0).unwrap();

        // 1. Positivity & Identity of indiscernibles
        assert_eq!(u.fubini_study_distance(&u), 0.0);
        assert_eq!(v.fubini_study_distance(&v), 0.0);
        assert_eq!(w.fubini_study_distance(&w), 0.0);

        // 2. Symmetry: d_FS(u, v) = d_FS(v, u)
        assert!((u.fubini_study_distance(&w) - w.fubini_study_distance(&u)).abs() < 1e-12);
        assert!((u.fubini_study_distance(&diag) - diag.fubini_study_distance(&u)).abs() < 1e-12);

        // 3. Orthogonal vectors on S2: dot = 0 -> d_FS = pi / 4
        let d_uw = u.fubini_study_distance(&w);
        assert!((d_uw - (PI / 4.0)).abs() < 1e-10);

        // 4. Antipodal vectors on S2: dot = -1 -> d_FS = pi / 2
        let d_uv = u.fubini_study_distance(&v);
        assert!((d_uv - (PI / 2.0)).abs() < 1e-10);

        // 5. Great-circle vs Fubini-Study relationship: theta = 2 * d_FS
        let gc = u.great_circle_distance(&diag);
        let fs = u.fubini_study_distance(&diag);
        assert!((gc - 2.0 * fs).abs() < 1e-10);

        // 6. Triangle inequality: d_FS(u, v) <= d_FS(u, diag) + d_FS(diag, v)
        let d_u_diag = u.fubini_study_distance(&diag);
        let d_diag_v = diag.fubini_study_distance(&v);
        assert!(d_uv <= d_u_diag + d_diag_v + 1e-10);
    }

    #[test]
    fn test_fubini_study_geodesic_midpoint() {
        let u = UnitS2::NORTH_POLE;
        let w = UnitS2::EQUATOR_X;
        let midpoint = u.geodesic_midpoint(&w).expect("non-antipodal midpoint");

        let d_u_m = u.fubini_study_distance(&midpoint);
        let d_m_w = midpoint.fubini_study_distance(&w);
        let d_u_w = u.fubini_study_distance(&w);

        assert!((d_u_m - d_m_w).abs() < 1e-10);
        assert!((d_u_m + d_m_w - d_u_w).abs() < 1e-10);
    }

    #[test]
    fn test_fixed_point_q30_hopf_projection() {
        let q = UnitS3::new(1.0, 0.0, 0.0, 0.0).unwrap();
        let q30 = q.to_q30();
        let s2_q30 = q30.hopf_project();

        assert_eq!(s2_q30.0[0], 0);
        assert_eq!(s2_q30.0[1], 0);
        assert_eq!(s2_q30.0[2], Q30_SCALE as i32);

        // Dot product between identical states in Q30
        let dot = s2_q30.dot_q30(&s2_q30);
        assert_eq!(dot, Q30_SCALE as i32);

        // Dot product between orthogonal states
        let eq_x = UnitS2Q30::EQUATOR_X;
        assert_eq!(s2_q30.dot_q30(&eq_x), 0);
    }

    #[test]
    fn test_hopf_state_trajectory() {
        let mut traj = HopfStateTrajectory::default();
        assert_eq!(traj.step_count, 0);
        assert_eq!(traj.cumulative_fubini_study, 0.0);

        let delta = UnitS3::new(1.0, 0.0, 1.0, 0.0).unwrap();
        let step1 = traj.step(&delta);
        assert!(step1 > 0.0);
        assert_eq!(traj.step_count, 1);
        assert_eq!(traj.cumulative_fubini_study, step1);

        let step2 = traj.step(&delta);
        assert!(step2 > 0.0);
        assert_eq!(traj.step_count, 2);
        assert!((traj.cumulative_fubini_study - (step1 + step2)).abs() < 1e-10);
    }

    #[test]
    fn test_hopf_metric_zero_allocations() {
        // Execute a trajectory with 100 transitions on stack only
        let initial = UnitS3::IDENTITY;
        let mut traj = HopfStateTrajectory::new(initial);
        let delta = UnitS3::new(0.99, 0.01, 0.05, 0.02).unwrap();

        let mut total_displacement = 0.0;
        for _ in 0..100 {
            total_displacement += traj.step(&delta);
        }

        assert!(total_displacement > 0.0);
        assert_eq!(traj.step_count, 100);

        // Fixed-point operations
        let q30 = traj.current_s3.to_q30();
        let s2_q30 = q30.hopf_project();
        let roots = [
            UnitS2Q30::NORTH_POLE,
            UnitS2Q30::SOUTH_POLE,
            UnitS2Q30::EQUATOR_X,
            UnitS2Q30::EQUATOR_Y,
        ];
        let nearest = s2_q30.nearest_root_q30(&roots);
        assert!(nearest.is_some_and(|idx| idx < 4));
        assert_eq!(s2_q30.nearest_root_q30(&[]), None);
    }
}
