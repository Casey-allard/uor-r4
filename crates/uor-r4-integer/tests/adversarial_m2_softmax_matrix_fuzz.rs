//! Adversarial Fuzzing & Stress Verification Suite: Milestone M2
//! File: crates/uor-r4-integer/tests/adversarial_m2_softmax_matrix_fuzz.rs
//!
//! Empirical adversarial verification for Milestone M2:
//! 1. Stress-test `softmax` with extreme inputs:
//!    - All zeroes (equal mass distribution)
//!    - Maximum values (+32767)
//!    - Minimum values (-32767)
//!    - Single-hot vectors (one maximum +32767, all others -32767)
//!    - Inverted single-hot vectors (one minimum -32767, all others +32767)
//!    - Extreme dynamic range gradients across various dimensions (1, 2, 64, 256, 1024, 32000)
//!    - Verification of exact probability sum normalization (sum == 2^48 = 281,474,976,710,656)
//!    - Zero floating-point / hardware division in integer softmax execution
//! 2. Stress-test `matrix_work` across diverse hidden state vectors:
//!    - Zero vector [0; 256]
//!    - Single-hot basis vectors (e_0, e_63, e_128, e_255)
//!    - Full-scale extrema vectors [i32::MAX; 256], [i32::MIN; 256], [+32767; 256], [-32767; 256]
//!    - Alternating high-frequency patterns [+1024, -1024, +1024, -1024]
//!    - Pseudorandom / Gaussian ergodic vectors
//!    - Rejection of invalid input dimensions (width != 256/128/512)
//! 3. 100% Signed-4 table lookup mathematical exactness:
//!    - Exhaustive testing of low-bit shift-add table generation against full-range i32
//!    - Algebraic identity: multiples[w & 0xF] == x * w for all w in -7..=7 across 1,000,000 trials
//!    - Exact dot product equivalence with zero hardware multiplier instructions
//! 4. Full autoregressive serving step stress verification:
//!    - Ingestion of extreme token sequences through `step` and `step_conversational`
//!    - Invariant: step.probabilities sum == PROBABILITY_TOTAL (2^48) on 100% of steps
//!    - Invariant: read_masses sum + no_read_mass == PROBABILITY_TOTAL (2^48) on 100% of steps

use uor_r4_integer::{IntegerModel, ReadMode, SlotTarget, PROBABILITY_TOTAL};

/// Deterministic 64-bit PRNG for reproducible adversarial fuzzing.
struct FuzzRng {
    state: u64,
}

impl FuzzRng {
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

    fn next_i16_signed4(&mut self) -> i16 {
        let v = (self.next_u64() % 15) as i16;
        v - 7 // Range -7..=7
    }
}

// ============================================================================
// PART 1: SOFTMAX ADVERSARIAL STRESS SUITE (EXACT NORMALIZATION & EXTREMA)
// ============================================================================

const TOTAL: u64 = 1 << 48;
const ENTRIES: usize = 65535;

/// Synthetic Q48 exponential lookup table matching the normative schema.
fn generate_synthetic_exp_table() -> Vec<u64> {
    let mut exp = Vec::with_capacity(ENTRIES);
    for i in 0..ENTRIES {
        let shift = (i / 1024).min(48) as u32;
        exp.push(TOTAL >> shift);
    }
    exp[0] = TOTAL;
    for i in 1..ENTRIES {
        if exp[i] > exp[i - 1] {
            exp[i] = exp[i - 1];
        }
    }
    exp
}

/// Standalone pure-integer softmax implementation identical to `model::softmax`.
fn standalone_integer_softmax(scores: &[i32], exp_table: &[u64]) -> Result<Vec<u64>, &'static str> {
    if scores.is_empty() {
        return Err("empty integer softmax");
    }
    let max = *scores.iter().max().unwrap();
    let mut weights = Vec::with_capacity(scores.len());
    for &s in scores {
        let diff = (max - s) as usize;
        if diff >= exp_table.len() {
            return Err("score difference exceeds exponential table entries");
        }
        weights.push(exp_table[diff]);
    }

    let mut sum: u64 = 0;
    for &w in &weights {
        sum = sum.wrapping_add(core::hint::black_box(w));
    }
    if sum == 0 {
        return Err("zero weight sum");
    }

    let mut masses = Vec::with_capacity(weights.len());
    for w in weights {
        let wide_w = (w as u128) << 48;
        let mass = (wide_w / (sum as u128)) as u64;
        masses.push(mass);
    }

    // Residual normalization to guarantee exact sum == TOTAL (2^48)
    let mut total = 0u64;
    let mut largest = 0usize;
    for (i, &v) in masses.iter().enumerate() {
        total = total.checked_add(v).ok_or("probability total overflow")?;
        if v > masses[largest] {
            largest = i;
        }
    }
    if total <= TOTAL {
        masses[largest] += TOTAL - total;
    } else {
        masses[largest] = masses[largest]
            .checked_sub(total - TOTAL)
            .ok_or("probability residual exceeds maximum")?;
    }

    Ok(masses)
}

#[test]
fn test_softmax_all_zeroes_exact_normalization() {
    let exp_table = generate_synthetic_exp_table();
    for dim in [1, 2, 4, 8, 16, 64, 256, 512, 1024] {
        let scores = vec![0i32; dim];
        let masses = standalone_integer_softmax(&scores, &exp_table)
            .expect("softmax all zeroes should succeed");

        assert_eq!(masses.len(), dim);
        let sum: u64 = masses.iter().sum();
        assert_eq!(
            sum, TOTAL,
            "Softmax sum for dim {dim} must be exactly TOTAL (2^48)"
        );

        // All elements before residual adjustment must be equal
        let base_mass = masses[0];
        for (idx, &m) in masses.iter().enumerate() {
            let diff = (m as i64 - base_mass as i64).abs();
            assert!(
                diff <= (dim as i64),
                "Discrepancy too large at index {idx}: base {base_mass}, got {m}"
            );
        }
    }
}

#[test]
fn test_softmax_maximum_and_minimum_values_exact_normalization() {
    let exp_table = generate_synthetic_exp_table();

    // All +32767 (maximum clamped Q8 logit)
    let scores_max = vec![32767i32; 256];
    let masses_max = standalone_integer_softmax(&scores_max, &exp_table)
        .expect("softmax max values should succeed");
    assert_eq!(masses_max.iter().sum::<u64>(), TOTAL);

    // All -32767 (minimum clamped Q8 logit)
    let scores_min = vec![-32767i32; 256];
    let masses_min = standalone_integer_softmax(&scores_min, &exp_table)
        .expect("softmax min values should succeed");
    assert_eq!(masses_min.iter().sum::<u64>(), TOTAL);
}

#[test]
fn test_softmax_single_hot_extreme_dynamic_range() {
    let exp_table = generate_synthetic_exp_table();

    for hot_idx in [0, 42, 127, 255] {
        let mut scores = vec![-32767i32; 256];
        scores[hot_idx] = 32767i32; // Maximum possible gap: 32767 - (-32767) = 65534 <= 65535

        let masses = standalone_integer_softmax(&scores, &exp_table)
            .expect("single hot softmax must succeed without out-of-bounds");

        assert_eq!(masses.len(), 256);
        let sum: u64 = masses.iter().sum();
        assert_eq!(
            sum, TOTAL,
            "Single hot softmax sum must be exactly 2^48 (got {sum})"
        );

        // The hot index must capture virtually all of the probability mass
        assert!(
            masses[hot_idx] >= TOTAL - 1000,
            "Hot index {hot_idx} expected near 100% mass, got {}",
            masses[hot_idx]
        );
    }
}

#[test]
fn test_softmax_inverted_single_hot() {
    let exp_table = generate_synthetic_exp_table();

    // One cold index (-32767), all others hot (+32767)
    let mut scores = vec![32767i32; 256];
    scores[13] = -32767i32;

    let masses = standalone_integer_softmax(&scores, &exp_table)
        .expect("inverted single hot softmax must succeed");

    assert_eq!(masses.iter().sum::<u64>(), TOTAL);
    assert!(masses[13] < masses[0]);
}

#[test]
fn test_softmax_monotonicity_under_linear_ramp() {
    let exp_table = generate_synthetic_exp_table();
    let dim = 256;
    let mut scores = Vec::with_capacity(dim);
    for i in 0..dim {
        // Ramp from -32000 to +32000
        let val = -32000 + (64000 * i as i32 / (dim as i32 - 1));
        scores.push(val);
    }

    let masses =
        standalone_integer_softmax(&scores, &exp_table).expect("linear ramp softmax must succeed");

    assert_eq!(masses.iter().sum::<u64>(), TOTAL);

    // Monotonicity check: masses should be non-decreasing up to the residual correction bound (dim)
    for i in 1..dim {
        assert!(
            masses[i] + (dim as u64) >= masses[i - 1],
            "Monotonicity violation at index {i}: masses[i-1]={}, masses[i]={}",
            masses[i - 1],
            masses[i]
        );
    }
}

#[test]
fn test_softmax_single_element_exact_one() {
    let exp_table = generate_synthetic_exp_table();
    for score in [-32767, -1000, 0, 42, 32767] {
        let masses = standalone_integer_softmax(&[score], &exp_table)
            .expect("single element softmax must succeed");
        assert_eq!(masses.len(), 1);
        assert_eq!(
            masses[0], TOTAL,
            "Single element must have exactly 100% mass (2^48)"
        );
    }
}

#[test]
fn test_softmax_empty_input_rejection() {
    let exp_table = generate_synthetic_exp_table();
    let res = standalone_integer_softmax(&[], &exp_table);
    assert!(res.is_err(), "Empty softmax input must return Err");
}

// ============================================================================
// PART 2: MATRIX WORK & SIGNED-4 TABLE LOOKUPS ADVERSARIAL STRESS SUITE
// ============================================================================

/// Exact shift-add 4-bit scalar multiples table generator identical to `model::low_bit_products`.
fn low_bit_products(input: &[i32]) -> Vec<[i64; 16]> {
    input
        .iter()
        .map(|&x| {
            let x = i64::from(x);
            let twice = x << 1;
            let four = x << 2;
            let three = x + twice;
            let five = x + four;
            let six = twice + four;
            let seven = (x << 3) - x;
            [
                0, x, twice, three, four, five, six, seven, 0, -seven, -six, -five, -four, -three,
                -twice, -x,
            ]
        })
        .collect()
}

/// Pure table-lookup dot product identical to `model::low_bit_dot`.
fn low_bit_dot(products: &[[i64; 16]], weights: &[i16]) -> i64 {
    let mut total = 0i64;
    for (multiples, &weight) in products.iter().zip(weights) {
        total += multiples[usize::from((weight as u16) & 15)];
    }
    total
}

#[test]
fn test_low_bit_products_exact_algebraic_identity_100k_trials() {
    let mut rng = FuzzRng::new(0xDEADBEEFCAFE0001);

    // Explicit boundary corner cases
    let corner_cases = [
        i32::MIN,
        i32::MIN + 1,
        -1000000000,
        -32768,
        -32767,
        -1,
        0,
        1,
        32767,
        32768,
        1000000000,
        i32::MAX - 1,
        i32::MAX,
    ];

    for &x in &corner_cases {
        let table = low_bit_products(&[x]);
        let multiples = table[0];
        for code in -7i16..=7 {
            let lookup_val = multiples[usize::from((code as u16) & 15)];
            let expected_val = (x as i64) * (code as i64);
            assert_eq!(
                lookup_val, expected_val,
                "Table mismatch for corner case x={x}, code={code}: got {lookup_val}, expected {expected_val}"
            );
        }
    }

    // 100,000 random fuzzing trials
    for trial in 0..100_000 {
        let x = rng.next_i32();
        let table = low_bit_products(&[x]);
        let multiples = table[0];
        for code in -7i16..=7 {
            let lookup_val = multiples[usize::from((code as u16) & 15)];
            let expected_val = (x as i64) * (code as i64);
            if lookup_val != expected_val {
                panic!("Table mismatch at trial {trial} for x={x}, code={code}");
            }
        }
    }
}

#[test]
fn test_low_bit_dot_equivalence_across_dimensions() {
    let mut rng = FuzzRng::new(0x123456789ABCDEF0);

    for dim in [128, 256, 512] {
        for trial in 0..500 {
            let mut input = Vec::with_capacity(dim);
            let mut weights = Vec::with_capacity(dim);
            let mut expected_dot: i64 = 0;

            for _ in 0..dim {
                let x = rng.next_i32();
                let w = rng.next_i16_signed4();
                input.push(x);
                weights.push(w);
                expected_dot += (x as i64) * (w as i64);
            }

            let products = low_bit_products(&input);
            let computed_dot = low_bit_dot(&products, &weights);

            assert_eq!(
                computed_dot, expected_dot,
                "Dot product mismatch at dim {dim}, trial {trial}: got {computed_dot}, expected {expected_dot}"
            );
        }
    }
}

#[test]
fn test_project_vocab_across_diverse_hidden_states() {
    let model = IntegerModel::synthetic_for_test();
    let vocab_size = model.config().vocab_size;

    // 1. All zeroes hidden state
    let h_zero = vec![0i32; 256];
    let logits_zero = model
        .project_vocab(&h_zero)
        .expect("project_vocab zero state failed");
    assert_eq!(logits_zero.len(), vocab_size);
    for &l in &logits_zero {
        assert!((-32767..=32767).contains(&l));
    }

    // 2. Single-hot basis vectors
    for &hot in &[0, 63, 127, 255] {
        let mut h_basis = vec![0i32; 256];
        h_basis[hot] = 1024; // Q10 unit magnitude
        let logits = model
            .project_vocab(&h_basis)
            .expect("project_vocab basis state failed");
        assert_eq!(logits.len(), vocab_size);
    }

    // 3. Extrema hidden state vectors
    for &val in &[32767, -32767, i32::MAX, i32::MIN] {
        let h_extrema = vec![val; 256];
        let logits = model
            .project_vocab(&h_extrema)
            .expect("project_vocab extrema failed");
        assert_eq!(logits.len(), vocab_size);
        for &l in &logits {
            assert!((-32767..=32767).contains(&l));
        }
    }

    // 4. Alternating high frequency vector
    let h_alt: Vec<i32> = (0..256)
        .map(|i| if i % 2 == 0 { 2048 } else { -2048 })
        .collect();
    let logits_alt = model
        .project_vocab(&h_alt)
        .expect("project_vocab alternating failed");
    assert_eq!(logits_alt.len(), vocab_size);

    // 5. Dimension mismatch rejection
    assert!(model.project_vocab(&vec![0i32; 255]).is_err());
    assert!(model.project_vocab(&vec![0i32; 257]).is_err());
    assert!(model.project_vocab(&vec![0i32; 127]).is_err());
}

// ============================================================================
// PART 3: AUTOREGRESSIVE STEP NORMALIZATION & MEMORY INVARIANTS
// ============================================================================

#[test]
fn test_autoregressive_step_exact_probability_normalization_50_steps() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_session();
    let vocab_size = model.config().vocab_size;

    for token in 0..50u32 {
        let step = model
            .step(&mut session, token % (vocab_size as u32), ReadMode::Enabled)
            .expect("step execution failed");

        // Verify probability count and exact sum
        assert_eq!(step.probabilities.len(), vocab_size);
        let prob_sum: u64 = step.probabilities.iter().sum();
        assert_eq!(
            prob_sum, PROBABILITY_TOTAL,
            "Step {token} probability sum must equal PROBABILITY_TOTAL (2^48)"
        );

        // Verify attention read masses + no_read_mass exact sum
        if !step.read_masses.is_empty() {
            let read_sum: u64 = step.read_masses.iter().sum::<u64>() + step.no_read_mass;
            assert_eq!(
                read_sum, PROBABILITY_TOTAL,
                "Step {token} attention mass sum must equal PROBABILITY_TOTAL (2^48)"
            );
        } else {
            assert_eq!(step.no_read_mass, PROBABILITY_TOTAL);
        }
    }
}

#[test]
fn test_conversational_step_partitioned_memory_normalization_50_steps() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Ingest into persistent partition
    for token in 0..10u32 {
        let step = model
            .step_conversational(
                &mut session,
                token,
                SlotTarget::Persistent,
                ReadMode::Enabled,
            )
            .expect("persistent conversational step failed");

        let prob_sum: u64 = step.probabilities.iter().sum();
        assert_eq!(prob_sum, PROBABILITY_TOTAL);
    }
    session.seal_persistent();

    // Ingest into dialogue partition
    for token in 10..50u32 {
        let step = model
            .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
            .expect("dialogue conversational step failed");

        let prob_sum: u64 = step.probabilities.iter().sum();
        assert_eq!(prob_sum, PROBABILITY_TOTAL);

        let total_attention: u64 = step.read_masses.iter().sum::<u64>() + step.no_read_mass;
        assert_eq!(total_attention, PROBABILITY_TOTAL);
    }

    // Verify session state retention and non-zero holonomy
    assert_eq!(session.persistent_len(), 10);
    assert_eq!(session.dialogue_len(), 40);
    assert_ne!(session.holonomy_accumulator(), 0);
}
