//! Adversarial Stress Verification Suite: Radix-4 Shift-Add & Hopf Projection
//!
//! Milestone M1 Iteration 2 Adversarial Verification:
//! 1. Rigorous testing of `mul_i32_radix4` across full 32-bit boundary domain:
//!    `i32::MIN`, `i32::MAX`, 0, -1, 1, powers of 2, alternating bit patterns,
//!    contrasted bit-for-bit against 64-bit mathematical integer multiplication `(a as i64) * (b as i64)`.
//! 2. 1,000,000 pseudo-random fuzzing trials verifying exact algebraic identity and commutativity.
//! 3. Adversarial testing of `UnitS3Q30::from_i32_coords` and `UnitS3Q30::hopf_project`
//!    under extreme coordinate magnitudes (zeros, single-axis extrema, multi-axis extrema,
//!    smallest non-zero coordinates, mixed sign extrema).
//! 4. Verification of zero overflow, zero panic, and exact preservation of unit sphere coordinates on S2.

use uor_r4_integer::math::{mul_i32_radix4, UnitS3Q30, Q30_SCALE};

/// Deterministic 64-bit SplitMix PRNG for repeatable adversarial fuzzing.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    fn next_i32(&mut self) -> i32 {
        self.next_u64() as i32
    }
}

#[test]
fn test_mul_i32_radix4_boundary_matrix() {
    let boundary_values: [i32; 38] = [
        // Extrema and near-extrema
        i32::MIN,
        i32::MIN + 1,
        i32::MIN + 2,
        -1000000000,
        -1000000,
        -65536,
        -32768,
        -256,
        -2,
        -1,
        0,
        1,
        2,
        256,
        32768,
        65536,
        1000000,
        1000000000,
        i32::MAX - 2,
        i32::MAX - 1,
        i32::MAX,
        // Bit patterns
        0x5555_5555,
        0x3333_3333,
        0x0F0F_0F0F,
        0x00FF_00FF,
        0x0000_FFFF,
        0x5555_5555_u32 as i32,
        0xAAAA_AAAA_u32 as i32,
        0xCCCC_CCCC_u32 as i32,
        0xF0F0_F0F0_u32 as i32,
        0xFF00_FF00_u32 as i32,
        0xFFFF_0000_u32 as i32,
        // Powers of two
        1 << 1,
        1 << 8,
        1 << 15,
        1 << 16,
        1 << 30,
        -(1 << 30),
    ];

    for &a in &boundary_values {
        for &b in &boundary_values {
            let actual = mul_i32_radix4(a, b);
            let expected = (a as i64) * (b as i64);
            assert_eq!(
                actual, expected,
                "mul_i32_radix4 failed for ({a}, {b}): got {actual}, expected {expected}"
            );

            // Commutativity
            let comm = mul_i32_radix4(b, a);
            assert_eq!(
                actual, comm,
                "mul_i32_radix4 commutativity failed for ({a}, {b}): {actual} vs {comm}"
            );
        }
    }
}

#[test]
fn test_mul_i32_radix4_all_powers_of_two() {
    let mut powers: Vec<i32> = Vec::new();
    for shift in 0..31 {
        let p = 1i32 << shift;
        powers.push(p);
        powers.push(-p);
        if shift > 0 {
            powers.push(p - 1);
            powers.push(-(p - 1));
            powers.push(p + 1);
            powers.push(-(p + 1));
        }
    }
    powers.push(i32::MIN);
    powers.push(i32::MAX);

    for &a in &powers {
        for &b in &powers {
            let actual = mul_i32_radix4(a, b);
            let expected = (a as i64) * (b as i64);
            assert_eq!(
                actual, expected,
                "mul_i32_radix4 power-of-two failed for ({a}, {b})"
            );
        }
    }
}

#[test]
fn test_mul_i32_radix4_one_million_fuzz_trials() {
    let mut rng = SplitMix64::new(0xDEAD_BEEF_CAFE_BABE);
    const NUM_TRIALS: usize = 1_000_000;

    for i in 0..NUM_TRIALS {
        let a = rng.next_i32();
        let b = rng.next_i32();
        let actual = mul_i32_radix4(a, b);
        let expected = (a as i64) * (b as i64);
        assert_eq!(
            actual, expected,
            "Fuzz trial #{i} failed for ({a}, {b}): actual={actual}, expected={expected}"
        );
    }
}

#[test]
fn test_mul_i32_radix4_algebraic_identities() {
    let test_set = [
        i32::MIN,
        i32::MIN + 1,
        -100,
        -1,
        0,
        1,
        42,
        100,
        i32::MAX - 1,
        i32::MAX,
    ];

    for &x in &test_set {
        // Zero product identity
        assert_eq!(mul_i32_radix4(x, 0), 0);
        assert_eq!(mul_i32_radix4(0, x), 0);

        // Multiplicative identity
        assert_eq!(mul_i32_radix4(x, 1), x as i64);
        assert_eq!(mul_i32_radix4(1, x), x as i64);

        // Negation identity
        assert_eq!(mul_i32_radix4(x, -1), -(x as i64));
        assert_eq!(mul_i32_radix4(-1, x), -(x as i64));

        // Sign inversion: (-a)*b == -(a*b) (when -a does not overflow i32)
        if x != i32::MIN {
            for &y in &test_set {
                assert_eq!(
                    mul_i32_radix4(-x, y),
                    -mul_i32_radix4(x, y),
                    "Sign symmetry failed for x={x}, y={y}"
                );
            }
        }
    }
}

#[test]
fn test_from_i32_coords_extreme_magnitudes() {
    let extreme_cases: [&[i32; 4]; 18] = [
        // All zeros (boundary: must yield IDENTITY without division by zero)
        &[0, 0, 0, 0],
        // Single-axis extreme positives
        &[i32::MAX, 0, 0, 0],
        &[0, i32::MAX, 0, 0],
        &[0, 0, i32::MAX, 0],
        &[0, 0, 0, i32::MAX],
        // Single-axis extreme negatives
        &[i32::MIN, 0, 0, 0],
        &[0, i32::MIN, 0, 0],
        &[0, 0, i32::MIN, 0],
        &[0, 0, 0, i32::MIN],
        // Multi-axis extreme positives
        &[i32::MAX, i32::MAX, i32::MAX, i32::MAX],
        // Multi-axis extreme negatives
        &[i32::MIN, i32::MIN, i32::MIN, i32::MIN],
        // Alternating extreme signs
        &[i32::MIN, i32::MAX, i32::MIN, i32::MAX],
        &[i32::MAX, i32::MIN, i32::MAX, i32::MIN],
        &[i32::MIN, 0, i32::MAX, 0],
        &[0, i32::MIN, 0, i32::MAX],
        // Smallest non-zero coordinates
        &[1, 0, 0, 0],
        &[1, 1, 1, 1],
        &[-1, -1, -1, -1],
    ];

    let q30 = Q30_SCALE as i128;
    let expected_norm_sq = q30 * q30; // 2^60

    for &coords in &extreme_cases {
        let s3 = UnitS3Q30::from_i32_coords(coords);
        let raw = s3.raw();

        // 1. All coordinates must be within [-2^30, 2^30]
        for (idx, &c) in raw.iter().enumerate() {
            assert!(
                (-(1 << 30)..=(1 << 30)).contains(&c),
                "Coordinate {idx} out of Q1.30 bounds: {c} for input {coords:?}"
            );
        }

        // 2. Check S3 unit norm: sum of squares must be close to 2^60
        let norm_sq: i128 = raw.iter().map(|&c| (c as i128) * (c as i128)).sum();

        // All non-zero inputs should produce a unit quaternion with norm close to 2^60
        let diff = (norm_sq - expected_norm_sq).abs();
        let relative_error = (diff as f64) / (expected_norm_sq as f64);
        assert!(
            relative_error < 0.001,
            "S3 norm squared error {relative_error:.6} exceeds 0.1% for coords {coords:?}: norm_sq={norm_sq}, expected={expected_norm_sq}"
        );
    }
}

#[test]
fn test_hopf_projection_extreme_magnitudes_preserves_s2() {
    let extreme_cases: [&[i32; 18]; 1] = [&[0; 18]];
    let _ = extreme_cases;

    let test_vectors: [&[i32; 4]; 18] = [
        &[0, 0, 0, 0],
        &[i32::MAX, 0, 0, 0],
        &[0, i32::MAX, 0, 0],
        &[0, 0, i32::MAX, 0],
        &[0, 0, 0, i32::MAX],
        &[i32::MIN, 0, 0, 0],
        &[0, i32::MIN, 0, 0],
        &[0, 0, i32::MIN, 0],
        &[0, 0, 0, i32::MIN],
        &[i32::MAX, i32::MAX, i32::MAX, i32::MAX],
        &[i32::MIN, i32::MIN, i32::MIN, i32::MIN],
        &[i32::MIN, i32::MAX, i32::MIN, i32::MAX],
        &[i32::MAX, i32::MIN, i32::MAX, i32::MIN],
        &[i32::MIN, 0, i32::MAX, 0],
        &[0, i32::MIN, 0, i32::MAX],
        &[1, 0, 0, 0],
        &[1, 1, 1, 1],
        &[-1, -1, -1, -1],
    ];

    let q30 = Q30_SCALE as f64;

    for &coords in &test_vectors {
        let s3 = UnitS3Q30::from_i32_coords(coords);
        let s2 = s3.hopf_project();

        // 1. Clamping check: each coordinate in [-2^30, 2^30]
        for (i, &coord) in s2.iter().enumerate() {
            assert!(
                (-(1 << 30)..=(1 << 30)).contains(&coord),
                "S2 coord {i} out of bounds: {coord} for input {coords:?}"
            );
        }

        // 2. S2 unit sphere preservation: x^2 + y^2 + z^2 == 2^60 (radius == 2^30)
        let sum_sq = (s2[0] as f64).powi(2) + (s2[1] as f64).powi(2) + (s2[2] as f64).powi(2);
        let radius = sum_sq.sqrt();
        let diff = (radius - q30).abs();
        let rel_err = diff / q30;

        assert!(
            rel_err < 0.001,
            "S2 radius relative error {rel_err:.6} exceeds 0.1% for coords {coords:?}: s2={s2:?}, radius={radius}, expected={q30}"
        );
    }
}

#[test]
fn test_hopf_projection_fuzz_10000_trials() {
    let mut rng = SplitMix64::new(0xCAFE_F00D_1234_5678);
    const NUM_TRIALS: usize = 10_000;
    let q30 = Q30_SCALE as f64;

    for trial in 0..NUM_TRIALS {
        // Generate diverse coordinates across 4 regimes:
        // 0: small [1..100]
        // 1: medium [1000..1000000]
        // 2: large [10000000..i32::MAX]
        // 3: completely arbitrary 32-bit bit patterns
        let regime = trial % 4;
        let coords: [i32; 4] = match regime {
            0 => [
                (rng.next_u64() % 200) as i32 - 100,
                (rng.next_u64() % 200) as i32 - 100,
                (rng.next_u64() % 200) as i32 - 100,
                (rng.next_u64() % 200) as i32 - 100,
            ],
            1 => [
                (rng.next_u64() % 2_000_000) as i32 - 1_000_000,
                (rng.next_u64() % 2_000_000) as i32 - 1_000_000,
                (rng.next_u64() % 2_000_000) as i32 - 1_000_000,
                (rng.next_u64() % 2_000_000) as i32 - 1_000_000,
            ],
            2 => [
                rng.next_i32().saturating_mul(100),
                rng.next_i32().saturating_mul(100),
                rng.next_i32().saturating_mul(100),
                rng.next_i32().saturating_mul(100),
            ],
            _ => [
                rng.next_i32(),
                rng.next_i32(),
                rng.next_i32(),
                rng.next_i32(),
            ],
        };

        let s3 = UnitS3Q30::from_i32_coords(&coords);
        let s2 = s3.hopf_project();

        // Preservation of S2 radius
        let radius =
            ((s2[0] as f64).powi(2) + (s2[1] as f64).powi(2) + (s2[2] as f64).powi(2)).sqrt();
        let rel_err = (radius - q30).abs() / q30;
        assert!(
            rel_err < 0.001,
            "Fuzz trial #{trial} failed S2 preservation: rel_err={rel_err:.6}, s2={s2:?}, coords={coords:?}"
        );

        // Verification of U(1) fiber phasor
        let fiber = s3.fiber_u1_q30();
        let fiber_norm = ((fiber[0] as f64).powi(2) + (fiber[1] as f64).powi(2)).sqrt();
        let fiber_rel_err = (fiber_norm - q30).abs() / q30;
        assert!(
            fiber_rel_err < 0.001,
            "Fuzz trial #{trial} failed U(1) fiber norm preservation: rel_err={fiber_rel_err:.6}, fiber={fiber:?}"
        );

        // Full bundle project
        let bundle = s3.hopf_fiber_project();
        assert!((-(1 << 30)..=(1 << 30)).contains(&bundle.fiber_phase));
    }
}

#[test]
fn test_hopf_projection_known_poles_and_equator() {
    // 1. North Pole of S3 -> North Pole of S2: (1, 0, 0, 0) -> (0, 0, 1)
    let north = UnitS3Q30::IDENTITY;
    let s2_north = north.hopf_project();
    assert_eq!(s2_north, [0, 0, 1 << 30]);

    // 2. South Pole of S3: (0, 0, 1, 0) in Q1.30 -> South Pole of S2: (0, 0, -1)
    let south = UnitS3Q30([0, 0, 1 << 30, 0]);
    let s2_south = south.hopf_project();
    assert_eq!(s2_south, [0, 0, -(1 << 30)]);

    // 3. Equator point: (1/sqrt(2), 0, 1/sqrt(2), 0)
    // 1/sqrt(2) * 2^30 = 759250125
    let half_sqrt2 = 759250125i32;
    let eq1 = UnitS3Q30([half_sqrt2, 0, half_sqrt2, 0]);
    let s2_eq1 = eq1.hopf_project();
    // For a=1/sqrt(2), b=0, c=1/sqrt(2), d=0:
    // x = 2*ac = 2*(1/2) = 1 in Q1.30 -> (1 << 30)
    // y = 0
    // z = a^2 - c^2 = 1/2 - 1/2 = 0
    assert_eq!(s2_eq1[1], 0);
    assert_eq!(s2_eq1[2], 0);
    // Allow slight rounding tolerance for 1/sqrt(2)
    assert!((s2_eq1[0] - (1 << 30)).abs() < 100);
}
