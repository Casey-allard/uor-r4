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

    /// Project to fiber-preserving Hopf state (base point in S2 + U(1) fiber phase).
    pub fn hopf_fiber_point(&self) -> HopfFiberPoint {
        let base = self.hopf_map();
        let phase = self.fiber_phase();
        HopfFiberPoint::new(base, phase)
    }

    /// Exact reconstruction of unit quaternion from S2 base point and S1 fiber phase.
    pub fn from_hopf_fiber(base: &UnitS2, phase: f64) -> Result<Self, HopfError> {
        let z = base.z.clamp(-1.0, 1.0);
        let r1_sq = ((1.0 + z) * 0.5).max(0.0);
        let r1 = libm::sqrt(r1_sq);
        let cos_p = libm::cos(phase);
        let sin_p = libm::sin(phase);

        if r1 > EPSILON {
            let a = r1 * cos_p;
            let b = r1 * sin_p;
            let inv_2r1 = 1.0 / (2.0 * r1);
            let c = (base.x * cos_p + base.y * sin_p) * inv_2r1;
            let d = (base.x * sin_p - base.y * cos_p) * inv_2r1;
            Self::new(a, b, c, d)
        } else {
            // At south pole (z = -1, r1 = 0, r2 = 1)
            Self::new(0.0, 0.0, cos_p, sin_p)
        }
    }

    /// Euclidean dot product between unit quaternions on S3: q1 . q2 = a1*a2 + b1*b2 + c1*c2 + d1*d2.
    /// Clamped to [-1.0, 1.0].
    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        let d = self.a * other.a + self.b * other.b + self.c * other.c + self.d * other.d;
        d.clamp(-1.0, 1.0)
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

/// Continuous representation of a state in the Hopf fibration S1 -> S3 -> S2.
/// Combines the S2 base projection with the U(1) S1 fiber phase to prevent
/// topological memory collapse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HopfFiberPoint {
    /// Base point on S2: pi(q) in R3.
    pub base: UnitS2,
    /// U(1) fiber phase angle psi in [-pi, pi).
    pub fiber_phase: f64,
    /// U(1) unit complex phasor: [cos(psi), sin(psi)].
    pub fiber_u1: [f64; 2],
}

impl HopfFiberPoint {
    pub fn new(base: UnitS2, fiber_phase: f64) -> Self {
        let mut p = fiber_phase;
        while p >= core::f64::consts::PI {
            p -= 2.0 * core::f64::consts::PI;
        }
        while p < -core::f64::consts::PI {
            p += 2.0 * core::f64::consts::PI;
        }
        let cos_p = libm::cos(p);
        let sin_p = libm::sin(p);
        Self {
            base,
            fiber_phase: p,
            fiber_u1: [cos_p, sin_p],
        }
    }

    /// Reconstruct unit quaternion in S3 from S2 base projection and S1 fiber phase.
    pub fn to_unit_s3(&self) -> Result<UnitS3, HopfError> {
        UnitS3::from_hopf_fiber(&self.base, self.fiber_phase)
    }

    /// Combined metric distance on S3 taking into account both S2 geodesic base distance
    /// and U(1) fiber holonomy phase displacement:
    ///   d^2 = d_FS(base_1, base_2)^2 + (Delta psi / 2)^2
    pub fn metric_distance(&self, other: &Self) -> f64 {
        let base_d = self.base.fubini_study_distance(&other.base);
        let mut d_phase = (self.fiber_phase - other.fiber_phase).abs();
        if d_phase > core::f64::consts::PI {
            d_phase = 2.0 * core::f64::consts::PI - d_phase;
        }
        libm::sqrt(base_d * base_d + 0.25 * d_phase * d_phase)
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

/// Continuous-state trajectory tracker in S3 with canonical projection to S2
/// and fiber-preserving holonomy phase tracking.
/// Tracks cumulative Fubini-Study geodesic displacement on S2 and holonomy phase rotation on S1.
/// Stack-allocated struct with zero dynamic allocations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HopfStateTrajectory {
    pub current_s3: UnitS3,
    pub current_s2: UnitS2,
    pub current_fiber_phase: f64,
    pub cumulative_fubini_study: f64,
    pub cumulative_holonomy: f64,
    pub step_count: usize,
}

impl HopfStateTrajectory {
    pub fn new(initial_s3: UnitS3) -> Self {
        let initial_s2 = initial_s3.hopf_map();
        let initial_phase = initial_s3.fiber_phase();
        Self {
            current_s3: initial_s3,
            current_s2: initial_s2,
            current_fiber_phase: initial_phase,
            cumulative_fubini_study: 0.0,
            cumulative_holonomy: 0.0,
            step_count: 0,
        }
    }

    pub fn reset(&mut self, initial_s3: UnitS3) {
        self.current_s3 = initial_s3;
        self.current_s2 = initial_s3.hopf_map();
        self.current_fiber_phase = initial_s3.fiber_phase();
        self.cumulative_fubini_study = 0.0;
        self.cumulative_holonomy = 0.0;
        self.step_count = 0;
    }

    /// Transition to next state via group action: q_{t+1} = q_t * delta.
    /// Updates S2 Hopf observation, tracks U(1) holonomy phase, and accumulates geodesic Fubini-Study distance.
    /// Returns step Fubini-Study distance d_FS(S2_t, S2_{t+1}).
    pub fn step(&mut self, delta_quaternion: &UnitS3) -> f64 {
        let next_s3 = self.current_s3.mul(delta_quaternion);
        let next_s2 = next_s3.hopf_map();
        let next_phase = next_s3.fiber_phase();
        let step_dist = self.current_s2.fubini_study_distance(&next_s2);

        let mut d_phase = next_phase - self.current_fiber_phase;
        if d_phase > core::f64::consts::PI {
            d_phase -= 2.0 * core::f64::consts::PI;
        } else if d_phase < -core::f64::consts::PI {
            d_phase += 2.0 * core::f64::consts::PI;
        }

        self.cumulative_holonomy += d_phase;
        self.cumulative_fubini_study += step_dist;
        self.current_s3 = next_s3;
        self.current_s2 = next_s2;
        self.current_fiber_phase = next_phase;
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

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN

/// Pure integer square root for u64 using Newton-Raphson.
/// Exact, deterministic, zero-allocation, zero-float.
pub const fn isqrt_u64(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut x = 1u64 << ((64 - n.leading_zeros() + 1) / 2);
    loop {
        let y = (x + n / x) >> 1;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// Pure integer square root for u128 using Newton-Raphson.
/// Exact, deterministic, zero-allocation, zero-float.
pub const fn isqrt_u128(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = 1u128 << ((128 - n.leading_zeros() + 1) / 2);
    loop {
        let y = (x + n / x) >> 1;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// 16-step CORDIC arc-tangent table in Q1.30: atan(2^-i) / pi * 2^30.
/// Exact integer values normalized such that pi corresponds to 2^30.
pub const CORDIC_ATAN_TABLE_Q30: [i32; 16] = [
    268435456, // atan(2^0) = pi/4 -> 2^28
    158467776, // atan(2^-1) / pi * 2^30
    83730694,  // atan(2^-2) / pi * 2^30
    42502690,  // atan(2^-3) / pi * 2^30
    21334812,  // atan(2^-4) / pi * 2^30
    10677271,  // atan(2^-5) / pi * 2^30
    5339870,   // atan(2^-6) / pi * 2^30
    2670177,   // atan(2^-7) / pi * 2^30
    1335119,   // atan(2^-8) / pi * 2^30
    667563,    // atan(2^-9) / pi * 2^30
    333782,    // atan(2^-10) / pi * 2^30
    166891,    // atan(2^-11) / pi * 2^30
    83446,     // atan(2^-12) / pi * 2^30
    41723,     // atan(2^-13) / pi * 2^30
    20861,     // atan(2^-14) / pi * 2^30
    10431,     // atan(2^-15) / pi * 2^30
];

/// Fixed-point CORDIC atan2 in Q1.30 with zero floats.
/// Returns angle theta / pi in Q1.30 format: range [-2^30, 2^30].
/// That is, -pi -> -2^30, 0 -> 0, +pi -> 2^30.
pub fn atan2_q30(y: i32, x: i32) -> i32 {
    if x == 0 && y == 0 {
        return 0;
    }
    if x == 0 {
        return if y > 0 { 1 << 29 } else { -(1 << 29) };
    }
    if y == 0 {
        return if x > 0 { 0 } else { 1 << 30 };
    }

    let mut x_curr = (x as i64).abs();
    let mut y_curr = y as i64;
    let mut angle: i64 = 0;

    for i in 0..16 {
        let x_shift = x_curr >> i;
        let y_shift = y_curr >> i;
        if y_curr > 0 {
            x_curr += y_shift;
            y_curr -= x_shift;
            angle += CORDIC_ATAN_TABLE_Q30[i] as i64;
        } else {
            x_curr -= y_shift;
            y_curr += x_shift;
            angle -= CORDIC_ATAN_TABLE_Q30[i] as i64;
        }
    }

    if x < 0 {
        if y >= 0 {
            angle = (1 << 30) - angle;
        } else {
            angle = -(1 << 30) - angle;
        }
    }

    angle.clamp(-Q30_SCALE, Q30_SCALE) as i32
}

/// Fixed-point Q1.30 representation of the Hopf bundle state S1 -> S3 -> S2.
/// Zero runtime floats, zero heap allocations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HopfFiberPointQ30 {
    /// Base point on S2 in Q1.30.
    pub base: UnitS2Q30,
    /// U(1) fiber phasor: [cos(psi), sin(psi)] in Q1.30.
    pub fiber_u1: [i32; 2],
    /// U(1) fiber phase angle normalized by pi: psi / pi in Q1.30 (range [-2^30, 2^30]).
    pub fiber_phase: i32,
}

impl HopfFiberPointQ30 {
    #[inline]
    pub fn dot_fiber_q30(&self, other: &[i32; 2]) -> i32 {
        let u1 = self.fiber_u1[0] as i64;
        let v1 = self.fiber_u1[1] as i64;
        let u2 = other[0] as i64;
        let v2 = other[1] as i64;
        let dot = (u1 * u2 + v1 * v2) >> 30;
        dot.clamp(-Q30_SCALE, Q30_SCALE) as i32
    }

    /// Reconstruct unit quaternion in Q1.30 from S2 base and U(1) fiber phasor.
    #[inline]
    pub fn to_unit_s3_q30(&self) -> UnitS3Q30 {
        UnitS3Q30::from_hopf_fiber_q30(&self.base, &self.fiber_u1)
    }

    /// Combined metric distance in Q1.30 taking into account base chordal distance
    /// and fiber phase displacement. Zero runtime floats, zero heap allocations.
    pub fn metric_distance_q30(&self, other: &Self) -> i32 {
        let dot_base = self.base.dot_q30(&other.base) as i64;
        let norm1 = self.base.dot_q30(&self.base) as i64;
        let norm2 = other.base.dot_q30(&other.base) as i64;
        let base_dist_sq = (norm1 + norm2 - 2 * dot_base).max(0) as u64;

        let mut d_phase = (self.fiber_phase as i64 - other.fiber_phase as i64).abs();
        if d_phase > Q30_SCALE {
            d_phase = 2 * Q30_SCALE - d_phase;
        }
        let phase_dist_sq = ((d_phase * d_phase) >> 30) as u64;

        let total_sq = base_dist_sq + (phase_dist_sq >> 2);
        isqrt_u64(total_sq << 30).min(Q30_SCALE as u64) as i32
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

    /// Exact fixed-point reconstruction of unit quaternion from S2 base vector and U(1) fiber phasor.
    /// Zero runtime floats, zero dynamic heap allocations.
    pub fn from_hopf_fiber_q30(base: &UnitS2Q30, fiber_u1: &[i32; 2]) -> Self {
        let z = base.0[2] as i64;
        let u = fiber_u1[0] as i64;
        let v = fiber_u1[1] as i64;

        // r1_sq = (1 + z) / 2 in Q1.30
        let r1_sq = ((Q30_SCALE + z).max(0) >> 1) as u64;
        // r1 in Q1.30: sqrt(r1_sq * 2^30)
        let r1 = isqrt_u64(r1_sq << 30) as i64;

        let clamp_q30 = |val: i64| -> i32 { val.clamp(-Q30_SCALE, Q30_SCALE) as i32 };

        // Threshold in Q1.30 for south pole singularity: r1 <= 1024 (~1e-6)
        if r1 > 1024 {
            let a = (r1 * u) >> 30;
            let b = (r1 * v) >> 30;
            let x = base.0[0] as i128;
            let y = base.0[1] as i128;
            let u128 = u as i128;
            let v128 = v as i128;
            let denom = 2 * (r1 as i128);
            let c = ((x * u128 + y * v128) / denom) as i64;
            let d = ((x * v128 - y * u128) / denom) as i64;
            Self([clamp_q30(a), clamp_q30(b), clamp_q30(c), clamp_q30(d)])
        } else {
            // At south pole (z = -1, r1 = 0, r2 = 1)
            Self([0, 0, clamp_q30(u), clamp_q30(v)])
        }
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

    /// Extract U(1) fiber unit complex phasor in Q1.30: [cos(psi), sin(psi)].
    /// Uses integer square root with zero floats and zero heap allocations.
    pub fn fiber_u1_q30(&self) -> [i32; 2] {
        let a = self.0[0] as i64;
        let b = self.0[1] as i64;
        let a_sq = (a.unsigned_abs() as u128) * (a.unsigned_abs() as u128);
        let b_sq = (b.unsigned_abs() as u128) * (b.unsigned_abs() as u128);
        let norm_ab_sq = a_sq + b_sq;
        // Guard against small numerical noise near south pole (r1^2 <= 1 << 20)
        if norm_ab_sq > (1 << 20) {
            let s = isqrt_u64(norm_ab_sq as u64);
            if s > 0 {
                let u = (((a as i128) << 30) / s as i128) as i64;
                let v = (((b as i128) << 30) / s as i128) as i64;
                let clamp_q30 = |val: i64| -> i32 { val.clamp(-Q30_SCALE, Q30_SCALE) as i32 };
                return [clamp_q30(u), clamp_q30(v)];
            }
        }
        let c = self.0[2] as i64;
        let d = self.0[3] as i64;
        let c_sq = (c.unsigned_abs() as u128) * (c.unsigned_abs() as u128);
        let d_sq = (d.unsigned_abs() as u128) * (d.unsigned_abs() as u128);
        let norm_cd_sq = c_sq + d_sq;
        if norm_cd_sq > 0 {
            let s = isqrt_u64(norm_cd_sq as u64);
            if s > 0 {
                let u = (((c as i128) << 30) / s as i128) as i64;
                let v = (((d as i128) << 30) / s as i128) as i64;
                let clamp_q30 = |val: i64| -> i32 { val.clamp(-Q30_SCALE, Q30_SCALE) as i32 };
                return [clamp_q30(u), clamp_q30(v)];
            }
        }
        [1 << 30, 0]
    }

    /// Compute U(1) fiber phase angle normalized by pi in Q1.30 via integer CORDIC.
    pub fn fiber_phase_q30(&self) -> i32 {
        let [u, v] = self.fiber_u1_q30();
        atan2_q30(v, u)
    }

    /// Full fiber-preserving Hopf projection in Q1.30: base point on S2 + U(1) fiber phase.
    pub fn hopf_fiber_project(&self) -> HopfFiberPointQ30 {
        let base = self.hopf_project();
        let fiber_u1 = self.fiber_u1_q30();
        let fiber_phase = atan2_q30(fiber_u1[1], fiber_u1[0]);
        HopfFiberPointQ30 {
            base,
            fiber_u1,
            fiber_phase,
        }
    }

    /// Fixed-point Hamilton product in Q1.30: q1 * q2.
    /// Preserves unit norm approximately with zero floating-point operations.
    pub fn mul_q30(&self, rhs: &Self) -> Self {
        let a1 = self.0[0] as i64;
        let b1 = self.0[1] as i64;
        let c1 = self.0[2] as i64;
        let d1 = self.0[3] as i64;

        let a2 = rhs.0[0] as i64;
        let b2 = rhs.0[1] as i64;
        let c2 = rhs.0[2] as i64;
        let d2 = rhs.0[3] as i64;

        let a = (a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2) >> 30;
        let b = (a1 * b2 + b1 * a2 + c1 * d2 - d1 * c2) >> 30;
        let c = (a1 * c2 - b1 * d2 + c1 * a2 + d1 * b2) >> 30;
        let d = (a1 * d2 + b1 * c2 - c1 * b2 + d1 * a2) >> 30;

        let clamp_q30 = |v: i64| -> i32 { v.clamp(-Q30_SCALE, Q30_SCALE) as i32 };
        UnitS3Q30([clamp_q30(a), clamp_q30(b), clamp_q30(c), clamp_q30(d)])
    }

    /// Exact fixed-point Euclidean norm in Q1.30: sqrt(a^2 + b^2 + c^2 + d^2).
    /// Uses integer Newton-Raphson square root with zero floats and zero heap allocations.
    #[inline]
    pub fn norm_q30(&self) -> u32 {
        let a_sq = (self.0[0] as i128) * (self.0[0] as i128);
        let b_sq = (self.0[1] as i128) * (self.0[1] as i128);
        let c_sq = (self.0[2] as i128) * (self.0[2] as i128);
        let d_sq = (self.0[3] as i128) * (self.0[3] as i128);
        let norm_sq = (a_sq + b_sq + c_sq + d_sq) as u128;
        isqrt_u128(norm_sq) as u32
    }

    /// Project and normalize any 4D quaternion in Q1.30 to a unit quaternion on S3.
    /// Uses pure integer Newton-Raphson square root with zero floats and zero heap allocations.
    pub fn normalize(raw: [i64; 4]) -> Self {
        let a_sq = (raw[0].unsigned_abs() as u128) * (raw[0].unsigned_abs() as u128);
        let b_sq = (raw[1].unsigned_abs() as u128) * (raw[1].unsigned_abs() as u128);
        let c_sq = (raw[2].unsigned_abs() as u128) * (raw[2].unsigned_abs() as u128);
        let d_sq = (raw[3].unsigned_abs() as u128) * (raw[3].unsigned_abs() as u128);
        let norm_sq = a_sq
            .saturating_add(b_sq)
            .saturating_add(c_sq)
            .saturating_add(d_sq);
        if norm_sq == 0 {
            return Self::IDENTITY;
        }
        let r = isqrt_u128(norm_sq) as i128;
        if r == 0 {
            return Self::IDENTITY;
        }
        let round_div = |num: i128, den: i128| -> i64 {
            if num >= 0 {
                ((num + (den >> 1)) / den) as i64
            } else {
                ((num - (den >> 1)) / den) as i64
            }
        };
        let clamp_q30 = |val: i64| -> i32 { val.clamp(-Q30_SCALE, Q30_SCALE) as i32 };
        let na = clamp_q30(round_div(raw[0] as i128 * (Q30_SCALE as i128), r));
        let nb = clamp_q30(round_div(raw[1] as i128 * (Q30_SCALE as i128), r));
        let nc = clamp_q30(round_div(raw[2] as i128 * (Q30_SCALE as i128), r));
        let nd = clamp_q30(round_div(raw[3] as i128 * (Q30_SCALE as i128), r));
        let mut res = Self([na, nb, nc, nd]);

        let mut cur_norm = res.norm_q30();
        if cur_norm > Q30_SCALE as u32 {
            for i in 0..4 {
                res.0[i] = clamp_q30(round_div(
                    res.0[i] as i128 * (Q30_SCALE as i128),
                    cur_norm as i128,
                ));
            }
            cur_norm = res.norm_q30();
        }

        for _ in 0..16 {
            if cur_norm <= Q30_SCALE as u32 && cur_norm >= (Q30_SCALE - 1) as u32 {
                break;
            }
            if cur_norm > Q30_SCALE as u32 {
                let mut max_idx = 0;
                let mut max_val = res.0[0].abs();
                for i in 1..4 {
                    let v = res.0[i].abs();
                    if v > max_val {
                        max_val = v;
                        max_idx = i;
                    }
                }
                if max_val == 0 {
                    return Self::IDENTITY;
                }
                if res.0[max_idx] > 0 {
                    res.0[max_idx] -= 1;
                } else {
                    res.0[max_idx] += 1;
                }
                cur_norm = res.norm_q30();
            } else if cur_norm < (Q30_SCALE - 1) as u32 {
                let mut max_idx = 0;
                let mut max_val = res.0[0].abs();
                for i in 1..4 {
                    let v = res.0[i].abs();
                    if v > max_val {
                        max_val = v;
                        max_idx = i;
                    }
                }
                if max_val == 0 {
                    return Self::IDENTITY;
                }
                if res.0[max_idx] >= 0 {
                    res.0[max_idx] += 1;
                } else {
                    res.0[max_idx] -= 1;
                }
                cur_norm = res.norm_q30();
            }
        }
        res
    }

    /// Convenience method calling UnitS3Q30::normalize on self coordinates.
    #[inline]
    pub fn normalized(&self) -> Self {
        Self::normalize(self.0.map(|x| x as i64))
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

    /// Project and normalize any 3D vector in Q1.30 to a unit vector on S2.
    /// Uses pure integer square root: zero floats, zero heap allocations.
    pub fn normalize(raw: [i64; 3]) -> Self {
        let x_sq = (raw[0] as i128) * (raw[0] as i128);
        let y_sq = (raw[1] as i128) * (raw[1] as i128);
        let z_sq = (raw[2] as i128) * (raw[2] as i128);
        let norm_sq = (x_sq + y_sq + z_sq) as u128;
        if norm_sq == 0 {
            return Self::NORTH_POLE;
        }
        let r = isqrt_u128(norm_sq) as i128;
        if r == 0 {
            return Self::NORTH_POLE;
        }
        let round_div = |num: i128, den: i128| -> i64 {
            if num >= 0 {
                ((num + (den >> 1)) / den) as i64
            } else {
                ((num - (den >> 1)) / den) as i64
            }
        };
        let clamp_q30 = |val: i64| -> i32 { val.clamp(-Q30_SCALE, Q30_SCALE) as i32 };
        let nx = round_div(raw[0] as i128 * (Q30_SCALE as i128), r);
        let ny = round_div(raw[1] as i128 * (Q30_SCALE as i128), r);
        let nz = round_div(raw[2] as i128 * (Q30_SCALE as i128), r);
        Self([clamp_q30(nx), clamp_q30(ny), clamp_q30(nz)])
    }

    /// Project and normalize any 2D vector in Q1.30 to a unit complex phasor on U(1).
    /// Uses pure integer square root: zero floats, zero heap allocations.
    pub fn normalize_u1(raw: [i64; 2]) -> [i32; 2] {
        let x_sq = (raw[0] as i128) * (raw[0] as i128);
        let y_sq = (raw[1] as i128) * (raw[1] as i128);
        let norm_sq = (x_sq + y_sq) as u128;
        if norm_sq == 0 {
            return [1 << 30, 0];
        }
        let r = isqrt_u128(norm_sq) as i128;
        if r == 0 {
            return [1 << 30, 0];
        }
        let round_div = |num: i128, den: i128| -> i64 {
            if num >= 0 {
                ((num + (den >> 1)) / den) as i64
            } else {
                ((num - (den >> 1)) / den) as i64
            }
        };
        let clamp_q30 = |val: i64| -> i32 { val.clamp(-Q30_SCALE, Q30_SCALE) as i32 };
        let nx = round_div(raw[0] as i128 * (Q30_SCALE as i128), r);
        let ny = round_div(raw[1] as i128 * (Q30_SCALE as i128), r);
        [clamp_q30(nx), clamp_q30(ny)]
    }
}

/// Fixed-point trajectory tracker with holonomy tracking in Q1.30.
/// Zero runtime floats, zero dynamic allocations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HopfStateTrajectoryQ30 {
    pub current_s3: UnitS3Q30,
    pub current_base: UnitS2Q30,
    pub current_fiber_u1: [i32; 2],
    pub current_fiber_phase: i32,
    pub cumulative_holonomy_q30: i64,
    pub step_count: usize,
}

impl HopfStateTrajectoryQ30 {
    pub fn new(initial_s3: UnitS3Q30) -> Self {
        let pt = initial_s3.hopf_fiber_project();
        Self {
            current_s3: initial_s3,
            current_base: pt.base,
            current_fiber_u1: pt.fiber_u1,
            current_fiber_phase: pt.fiber_phase,
            cumulative_holonomy_q30: 0,
            step_count: 0,
        }
    }

    pub fn reset(&mut self, initial_s3: UnitS3Q30) {
        let pt = initial_s3.hopf_fiber_project();
        self.current_s3 = initial_s3;
        self.current_base = pt.base;
        self.current_fiber_u1 = pt.fiber_u1;
        self.current_fiber_phase = pt.fiber_phase;
        self.cumulative_holonomy_q30 = 0;
        self.step_count = 0;
    }

    pub fn step_q30(&mut self, delta_quaternion: &UnitS3Q30) {
        let next_s3 = self.current_s3.mul_q30(delta_quaternion);
        let pt = next_s3.hopf_fiber_project();
        let mut d_phase = (pt.fiber_phase as i64) - (self.current_fiber_phase as i64);
        if d_phase > (1 << 30) {
            d_phase -= 2 * (1 << 30);
        } else if d_phase < -(1 << 30) {
            d_phase += 2 * (1 << 30);
        }
        self.cumulative_holonomy_q30 += d_phase;
        self.current_s3 = next_s3;
        self.current_base = pt.base;
        self.current_fiber_u1 = pt.fiber_u1;
        self.current_fiber_phase = pt.fiber_phase;
        self.step_count += 1;
    }
}

impl Default for HopfStateTrajectoryQ30 {
    fn default() -> Self {
        Self::new(UnitS3Q30::IDENTITY)
    }
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

/// Non-serving floating-point bridge for offline training and evaluation.
impl UnitS3Q30 {
    /// Convert fixed-point Q1.30 unit quaternion to continuous UnitS3.
    #[inline]
    pub fn to_unit_s3(&self) -> UnitS3 {
        let inv = 1.0 / (Q30_SCALE as f64);
        UnitS3 {
            a: self.0[0] as f64 * inv,
            b: self.0[1] as f64 * inv,
            c: self.0[2] as f64 * inv,
            d: self.0[3] as f64 * inv,
        }
    }
}

impl HopfFiberPointQ30 {
    /// Reconstruct continuous unit quaternion on S3 from S2 base and U(1) fiber phasor.
    #[inline]
    pub fn to_unit_s3(&self) -> UnitS3 {
        self.to_unit_s3_q30().to_unit_s3()
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
    fn test_fixed_point_q30_hamilton_product() {
        let id = UnitS3Q30::IDENTITY;
        let q = UnitS3::new(1.0, 2.0, 3.0, 4.0).unwrap().to_q30();
        let prod = q.mul_q30(&id);
        for i in 0..4 {
            assert!((prod.0[i] - q.0[i]).abs() <= 1);
        }

        // Test i * j = k in Q1.30
        let qi = UnitS3::I.to_q30();
        let qj = UnitS3::J.to_q30();
        let qk = UnitS3::K.to_q30();
        let ij = qi.mul_q30(&qj);
        for i in 0..4 {
            assert!((ij.0[i] - qk.0[i]).abs() <= 2);
        }
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

    #[test]
    fn test_hopf_fiber_reconstruction() {
        let test_quats = [
            UnitS3::IDENTITY,
            UnitS3::I,
            UnitS3::J,
            UnitS3::K,
            UnitS3::new(1.0, 2.0, 3.0, 4.0).unwrap(),
            UnitS3::new(-0.5, 0.5, -0.5, 0.5).unwrap(),
            UnitS3::new(0.0, 0.0, 1.0, 0.0).unwrap(),
            UnitS3::new(0.0, 0.0, 0.0, 1.0).unwrap(),
        ];

        for q in test_quats {
            let pt = q.hopf_fiber_point();
            // Verify S2 base lies on sphere
            let s2_norm = pt.base.x * pt.base.x + pt.base.y * pt.base.y + pt.base.z * pt.base.z;
            assert!((s2_norm - 1.0).abs() < 1e-10);

            // Verify U1 phasor lies on circle
            let u1_norm = pt.fiber_u1[0] * pt.fiber_u1[0] + pt.fiber_u1[1] * pt.fiber_u1[1];
            assert!((u1_norm - 1.0).abs() < 1e-10);

            // Reconstruct and verify S2 base projection matches exactly
            let recon_q = pt.to_unit_s3().expect("valid reconstruction");
            let recon_pt = recon_q.hopf_fiber_point();
            assert!((recon_pt.base.x - pt.base.x).abs() < 1e-8);
            assert!((recon_pt.base.y - pt.base.y).abs() < 1e-8);
            assert!((recon_pt.base.z - pt.base.z).abs() < 1e-8);

            // Metric distance to self is 0
            assert!(pt.metric_distance(&pt) < 1e-12);
        }
    }

    #[test]
    fn test_isqrt_u64_and_cordic_atan2() {
        // Test integer square root
        assert_eq!(isqrt_u64(0), 0);
        assert_eq!(isqrt_u64(1), 1);
        assert_eq!(isqrt_u64(4), 2);
        assert_eq!(isqrt_u64(9), 3);
        assert_eq!(isqrt_u64(16), 4);
        assert_eq!(isqrt_u64(100), 10);
        assert_eq!(isqrt_u64(1 << 30), 1 << 15);
        assert_eq!(isqrt_u64(1 << 60), 1 << 30);

        // Test CORDIC atan2 against float atan2
        let scale = Q30_SCALE as f64;
        let test_cases = [
            (1000, 0, 0.0),                  // 0
            (0, 1000, PI / 2.0),             // pi/2
            (0, -1000, -PI / 2.0),           // -pi/2
            (1000, 1000, PI / 4.0),          // pi/4
            (-1000, 1000, 3.0 * PI / 4.0),   // 3pi/4
            (-1000, -1000, -3.0 * PI / 4.0), // -3pi/4
            (1000, -1000, -PI / 4.0),        // -pi/4
        ];

        for (x, y, expected_rad) in test_cases {
            let q30_angle = atan2_q30(y, x);
            let computed_rad = (q30_angle as f64 / scale) * PI;
            assert!(
                (computed_rad - expected_rad).abs() < 0.02,
                "atan2_q30({y}, {x}) expected ~{expected_rad:.4}, got {computed_rad:.4}"
            );
        }
    }

    #[test]
    fn test_fixed_point_q30_fiber_holonomy() {
        let q = UnitS3::new(1.0, 2.0, 3.0, 4.0).unwrap();
        let q30 = q.to_q30();

        let pt = q30.hopf_fiber_project();
        // Base is on S2
        let dot_self = pt.base.dot_q30(&pt.base);
        assert!((dot_self - (Q30_SCALE as i32)).abs() <= 10);

        // Fiber phasor norm is approx 1 in Q30
        let u = pt.fiber_u1[0] as i64;
        let v = pt.fiber_u1[1] as i64;
        let norm_sq = (u * u + v * v) >> 30;
        assert!((norm_sq - Q30_SCALE).abs() <= 2000);

        // Trajectory with holonomy in Q30
        let mut traj = HopfStateTrajectoryQ30::new(q30);
        let delta = UnitS3::new(0.99, 0.01, 0.05, 0.02).unwrap().to_q30();
        for _ in 0..10 {
            traj.step_q30(&delta);
        }
        assert_eq!(traj.step_count, 10);
    }

    #[test]
    fn test_fixed_point_q30_fiber_reconstruction_roundtrip() {
        let test_quats = [
            UnitS3::IDENTITY,
            UnitS3::I,
            UnitS3::J,
            UnitS3::K,
            UnitS3::new(1.0, 2.0, 3.0, 4.0).unwrap(),
            UnitS3::new(-0.5, 0.5, -0.5, 0.5).unwrap(),
            UnitS3::new(0.0, 0.0, 1.0, 0.0).unwrap(), // south pole (z = -1)
            UnitS3::new(0.0, 0.0, 0.0, 1.0).unwrap(), // south pole with phase pi/2
            UnitS3::new(0.99, 0.01, 0.05, 0.02).unwrap(),
        ];

        for q in test_quats {
            let q30 = q.to_q30();
            let pt = q30.hopf_fiber_project();

            // Reconstruct in Q1.30
            let recon = pt.to_unit_s3_q30();
            let recon_pt = recon.hopf_fiber_project();
            let recon_f64 = pt.to_unit_s3();
            assert!((recon_f64.dot(&q) - 1.0).abs() < 1e-4);

            // Verify base S2 projection matches closely
            for i in 0..3 {
                assert!(
                    (recon_pt.base.0[i] - pt.base.0[i]).abs() <= 2000,
                    "Base mismatch on coordinate {i}: original {}, recon {}",
                    pt.base.0[i],
                    recon_pt.base.0[i]
                );
            }

            // Verify metric distance to self is 0 and to reconstruction is minimal
            assert_eq!(pt.metric_distance_q30(&pt), 0);
            assert!(
                pt.metric_distance_q30(&recon_pt) <= 35_000,
                "Reconstruction metric distance too high: {}",
                pt.metric_distance_q30(&recon_pt)
            );
        }
    }

    #[test]
    fn test_fixed_point_q30_south_pole_noise_stability() {
        // Construct state near south pole with tiny noise in a, b
        // q = (1, 0, 0, 1 << 30): near south pole, fiber phasor should be [0, 1 << 30] (phase ~ pi/2)
        let q_noisy = UnitS3Q30([1, 0, 0, 1 << 30]);
        let pt = q_noisy.hopf_fiber_project();

        // Must not snap to [1 << 30, 0] (phase 0)
        assert!(
            pt.fiber_u1[1].abs() > (1 << 28),
            "Phasor y component must reflect south pole phase, got {:?}",
            pt.fiber_u1
        );
        assert!(
            pt.fiber_u1[0].abs() < (1 << 20),
            "Phasor x component must be near 0, got {:?}",
            pt.fiber_u1
        );
    }

    #[test]
    fn test_unit_s3_q30_normalization() {
        assert_eq!(UnitS3Q30::IDENTITY.norm_q30(), Q30_SCALE as u32);
        assert_eq!(UnitS3Q30::normalize([0, 0, 0, 0]), UnitS3Q30::IDENTITY);

        let test_cases = [
            [1, 0, 0, 0],
            [0, 1, 0, 0],
            [0, 0, 1, 0],
            [0, 0, 0, 1],
            [1, 1, 0, 0],
            [1, 1, 1, 0],
            [1, 1, 1, 1],
            [1, 2, 3, 4],
            [10, 20, 30, 40],
            [-500, 1200, -3400, 900],
            [1 << 28, -(1 << 28), 1 << 28, -(1 << 28)],
            [1 << 30, 1 << 30, 0, 0],
            [1234567, -9876543, 5555555, -2222222],
            [i64::MIN, 0, 0, 0],
            [0, i64::MIN, 0, 0],
            [i64::MIN, i64::MIN, i64::MIN, i64::MIN],
            [i64::MAX, i64::MAX, i64::MAX, i64::MAX],
            [i64::MAX, 0, -1, 42],
        ];

        for raw in test_cases {
            let s3 = UnitS3Q30::normalize(raw);
            let n = s3.norm_q30();
            assert!(
                n <= Q30_SCALE as u32 && n >= (Q30_SCALE - 1) as u32,
                "Normalized vector must have norm in [Q30_SCALE - 1, Q30_SCALE], got {n} for {raw:?}"
            );
            for c in s3.0 {
                assert!(
                    c >= -(Q30_SCALE as i32) && c <= Q30_SCALE as i32,
                    "Coordinate must be clamped in [-Q30_SCALE, Q30_SCALE], got {c}"
                );
            }

            let s3_renorm = s3.normalized();
            let renorm_n = s3_renorm.norm_q30();
            assert!(
                renorm_n <= Q30_SCALE as u32 && renorm_n >= (Q30_SCALE - 1) as u32,
                "Renormalized vector must have norm in [Q30_SCALE - 1, Q30_SCALE], got {renorm_n}"
            );
        }

        // Stress test across 1,000 pseudo-random 4D vectors of various magnitudes
        let mut seed = 0xdead_beef_cafe_babe_u64;
        for _ in 0..1000 {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let a = (seed as i32) as i64;
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let b = (seed as i32) as i64;
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let c = (seed as i32) as i64;
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let d = (seed as i32) as i64;

            let v = [a, b, c, d];
            let s3 = UnitS3Q30::normalize(v);
            let n = s3.norm_q30();
            assert!(
                n <= Q30_SCALE as u32 && n >= (Q30_SCALE - 1) as u32,
                "Stress test vector norm must be in [Q30_SCALE - 1, Q30_SCALE], got {n} for {v:?}"
            );
        }
    }
}
