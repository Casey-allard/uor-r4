//! Focused tests for the opt-in relaxed/straight-through gate estimator.
//!
//! They cover three risks: the hard forward used by the estimator is exactly the
//! production cascade; the analytically derived gradients match finite differences
//! of the relaxed objective; and the estimator is additive, i.e. it changes no
//! default-path bytes and touches only the two blocks it declares.
use super::super::circuit::{CompiledCircuit, Topology, GATE_LOGITS, INPUT_BITS};
use super::super::engine::Head;
use super::super::learner;
use super::super::policy::{head_range, Parameters, PARAMETER_COUNT};
use super::*;

const SEED: u64 = 7341;

fn topology() -> Topology {
    Topology::seeded(super::super::policy::WIRING_SEED)
}

fn input_with(seed: u64, set: &[(usize, u8)]) -> [bool; INPUT_BITS] {
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    let mut input = [false; INPUT_BITS];
    for bit in input.iter_mut() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *bit = state >> 63 != 0;
    }
    for &(offset, value) in set {
        for i in 0..8 {
            input[offset + i] = (value >> i) & 1 != 0;
        }
    }
    input
}

fn relaxed_loss(
    parameters: &Parameters,
    input: &[bool; INPUT_BITS],
    target: usize,
    temperature: f64,
) -> f64 {
    let forward = soft_forward(&topology(), parameters, input, temperature).expect("soft forward");
    soft_mixture(parameters, &forward.bits, target)
        .expect("soft mixture")
        .loss
}

fn with_perturbed(values: &[f64], index: usize, delta: f64) -> Parameters {
    let mut changed = values.to_vec();
    changed[index] += delta;
    Parameters::from_values(SEED, changed).expect("perturbed parameters")
}

/// The estimator's forward is the production cascade, bit for bit.
#[test]
fn hard_context_matches_compiled_circuit() {
    let parameters = Parameters::seeded(SEED).expect("seeded");
    let topology = topology();
    let compiled = parameters.compile().expect("compile");
    for seed in 0..8u64 {
        let input = input_with(
            seed,
            &[
                (792, (seed.wrapping_mul(37) & 0xff) as u8),
                (776, 9),
                (784, 4),
            ],
        );
        let hard = hard_context(&topology, &parameters, &input).expect("hard");
        assert_eq!(hard, compiled.circuit.evaluate(&input).expect("evaluate"));
        assert_eq!(
            hard,
            CompiledCircuit::compile(topology.clone(), &parameters.values()[..GATE_LOGITS])
                .expect("primitive compile")
                .evaluate(&input)
                .expect("primitive evaluate")
        );
    }
}

/// Lowering the temperature moves the relaxed bits toward the hard table: the
/// rounding of each relaxed bit agrees with the hard bit it relaxes wherever the
/// relaxed bit is not on the decision boundary, and the largest gap shrinks. A
/// gate whose selected logit sits on the boundary keeps a soft value at any
/// temperature, so equality is not asserted.
#[test]
fn soft_forward_converges_to_hard() {
    let parameters = Parameters::seeded(SEED).expect("seeded");
    let topology = topology();
    for seed in 0..4u64 {
        let input = input_with(seed, &[(792, (seed.wrapping_mul(11) & 0xff) as u8)]);
        let hard = hard_context(&topology, &parameters, &input).expect("hard");
        let coarse = soft_forward(&topology, &parameters, &input, 0.5).expect("coarse");
        let fine = soft_forward(&topology, &parameters, &input, 1e-3).expect("fine");
        let mut coarse_gap = 0.0f64;
        let mut fine_gap = 0.0f64;
        for i in 0..CONTEXT_BITS {
            let want = f64::from(u8::from((hard >> i) & 1 != 0));
            coarse_gap = coarse_gap.max((coarse.bits[i] - want).abs());
            fine_gap = fine_gap.max((fine.bits[i] - want).abs());
            if (fine.bits[i] - 0.5).abs() > 0.05 {
                assert_eq!(
                    fine.bits[i] > 0.5,
                    want > 0.5,
                    "bit {i} rounded the wrong way: soft {} hard {want}",
                    fine.bits[i]
                );
            }
        }
        assert!(
            fine_gap < coarse_gap,
            "no convergence: coarse {coarse_gap} fine {fine_gap}"
        );
    }
}

/// d(loss)/d(relaxed context bit) matches finite differences of the mixture loss.
#[test]
fn mixture_bit_gradient_matches_finite_difference() {
    let parameters = Parameters::seeded(SEED).expect("seeded");
    let input = input_with(3, &[(792, 118)]);
    for target in [0usize, 51, 256] {
        let forward = soft_forward(&topology(), &parameters, &input, 0.4).expect("soft");
        let mixture = soft_mixture(&parameters, &forward.bits, target).expect("mixture");
        let step = 1e-5;
        for i in 0..CONTEXT_BITS {
            let mut plus = forward.bits;
            let mut minus = forward.bits;
            plus[i] = (plus[i] + step).min(1.0);
            minus[i] = (minus[i] - step).max(0.0);
            let numeric = (soft_mixture(&parameters, &plus, target).expect("plus").loss
                - soft_mixture(&parameters, &minus, target)
                    .expect("minus")
                    .loss)
                / (2.0 * step);
            let analytic = mixture.d_bits[i];
            assert!(
                (analytic - numeric).abs() <= 1e-6 * analytic.abs().max(1e-3),
                "bit {i} analytic {analytic} numeric {numeric}"
            );
        }
    }
}

/// The relaxed cascade backward matches a finite-difference Jacobian-vector
/// product for the actual upstream gradient.
#[test]
fn cascade_backward_matches_finite_difference() {
    let parameters = Parameters::seeded(SEED).expect("seeded");
    let input = input_with(9, &[(792, 200), (784, 7)]);
    let temperature = 0.5;
    let forward = soft_forward(&topology(), &parameters, &input, temperature).expect("soft");
    let mixture = soft_mixture(&parameters, &forward.bits, 200).expect("mixture");
    let analytic = soft_backward(
        &topology(),
        &parameters,
        &forward,
        &mixture.d_bits,
        temperature,
    )
    .expect("backward");
    let values = parameters.values().to_vec();
    // Check the entries the relaxed objective actually depends on.
    let mut ranked: Vec<usize> = (0..GATE_LOGITS).collect();
    ranked.sort_by(|&a, &b| analytic[b].abs().total_cmp(&analytic[a].abs()));
    let step = 1e-4;
    for &index in ranked.iter().take(12) {
        let plus = soft_forward(
            &topology(),
            &with_perturbed(&values, index, step),
            &input,
            temperature,
        )
        .expect("plus");
        let minus = soft_forward(
            &topology(),
            &with_perturbed(&values, index, -step),
            &input,
            temperature,
        )
        .expect("minus");
        let numeric = mixture
            .d_bits
            .iter()
            .zip(plus.bits.iter().zip(minus.bits.iter()))
            .map(|(d, (p, m))| d * (p - m) / (2.0 * step))
            .sum::<f64>();
        let exact = analytic[index];
        assert!(
            (exact - numeric).abs() <= 1e-6 * exact.abs().max(1e-3),
            "logit {index} analytic {exact} numeric {numeric}"
        );
    }
}

/// End-to-end: the gradient the fit applies is the gradient of the relaxed loss.
#[test]
fn gate_gradient_matches_finite_difference_of_relaxed_loss() {
    let parameters = Parameters::seeded(SEED).expect("seeded");
    let input = input_with(21, &[(792, 45)]);
    let target = 45usize;
    let temperature = 0.35;
    let forward = soft_forward(&topology(), &parameters, &input, temperature).expect("soft");
    let mixture = soft_mixture(&parameters, &forward.bits, target).expect("mixture");
    let analytic = soft_backward(
        &topology(),
        &parameters,
        &forward,
        &mixture.d_bits,
        temperature,
    )
    .expect("backward");
    let values = parameters.values().to_vec();
    let mut ranked: Vec<usize> = (0..GATE_LOGITS).collect();
    ranked.sort_by(|&a, &b| analytic[b].abs().total_cmp(&analytic[a].abs()));
    let step = 1e-4;
    for &index in ranked.iter().take(8) {
        let numeric = (relaxed_loss(
            &with_perturbed(&values, index, step),
            &input,
            target,
            temperature,
        ) - relaxed_loss(
            &with_perturbed(&values, index, -step),
            &input,
            target,
            temperature,
        )) / (2.0 * step);
        let exact = analytic[index];
        assert!(
            (exact - numeric).abs() <= 1e-5 * exact.abs().max(1e-3),
            "logit {index} analytic {exact} numeric {numeric}"
        );
    }
}

/// A bounded fit changes only the declared blocks and the emission rows it visits,
/// and it is deterministic for the same inputs.
#[test]
fn fit_touches_only_declared_blocks_and_is_deterministic() {
    let parameters = Parameters::seeded(SEED).expect("seeded");
    let pairs: Vec<StePair> = (0..3)
        .map(|i| StePair {
            input: input_with(40 + i, &[(792, 30 + i as u8)]),
            target: 30 + i as u16,
        })
        .collect();
    let config = SteConfig {
        updates: 6,
        rate: 0.05,
        temperature_start: 1.0,
        temperature_end: 0.5,
        gate_rms: SCORE_FUNCTION_GATE_RMS,
    };
    let first = fit(&parameters, &pairs, &config, &mut |_, _| true).expect("fit");
    let second = fit(&parameters, &pairs, &config, &mut |_, _| true).expect("fit");
    assert_eq!(first.updates_completed, 6);
    assert_eq!(first.parameters.digest(), second.parameters.digest());
    let (emission_offset, _) = head_range(Head::Emission, 0).expect("emission range");
    for (index, (&after, &before)) in first
        .parameters
        .values()
        .iter()
        .zip(parameters.values().iter())
        .enumerate()
    {
        if index < GATE_LOGITS || (emission_offset..PARAMETER_COUNT).contains(&index) {
            continue;
        }
        assert_eq!(after, before, "untrained block changed at {index}");
    }
    let gate_moved = (0..GATE_LOGITS)
        .filter(|&i| first.parameters.values()[i] != parameters.values()[i])
        .count();
    assert!(gate_moved > 0, "relaxed gate block did not move");
    let head_moved = (emission_offset..PARAMETER_COUNT)
        .filter(|&i| first.parameters.values()[i] != parameters.values()[i])
        .count();
    assert!(head_moved > 0, "emission rows did not move");
    assert!(gate_moved < GATE_LOGITS, "gate block moved everywhere");
}

/// Bound check: a fit that hits the observer's stop returns the completed count.
#[test]
fn fit_honours_observer_stop() {
    let parameters = Parameters::seeded(SEED).expect("seeded");
    let pairs = [StePair {
        input: input_with(5, &[(792, 1)]),
        target: 1,
    }];
    let config = SteConfig {
        updates: 100,
        rate: 0.05,
        temperature_start: 1.0,
        temperature_end: 0.02,
        gate_rms: SCORE_FUNCTION_GATE_RMS,
    };
    let stopped = fit(&parameters, &pairs, &config, &mut |_, record| {
        record.update < 3
    })
    .expect("fit");
    assert_eq!(stopped.updates_completed, 3);
    assert!(stopped.stopped_early);
}

/// `learner::sgd` is the only optimizer, so the fit's update rule is the crate's.
#[test]
fn fit_uses_the_crate_optimizer() {
    let parameters = Parameters::seeded(SEED).expect("seeded");
    let gradient = vec![0.0; PARAMETER_COUNT];
    let same = learner::sgd(&parameters, &gradient, 0.05).expect("sgd");
    assert_eq!(same.digest(), parameters.digest());
}

/// Rejects malformed configuration and pairs instead of running.
#[test]
fn fit_rejects_invalid_inputs() {
    let parameters = Parameters::seeded(SEED).expect("seeded");
    let pairs = [StePair {
        input: [false; INPUT_BITS],
        target: 256,
    }];
    let bad_updates = SteConfig {
        updates: 0,
        rate: 0.05,
        temperature_start: 1.0,
        temperature_end: 0.02,
        gate_rms: 0.02,
    };
    assert!(fit(&parameters, &pairs, &bad_updates, &mut |_, _| true).is_err());
    let bad_target = SteConfig {
        updates: 1,
        ..bad_updates
    };
    let out_of_range = [StePair {
        input: [false; INPUT_BITS],
        target: 257,
    }];
    assert!(fit(&parameters, &out_of_range, &bad_target, &mut |_, _| true).is_err());
}

/// Regression lock: the default serving export of the frozen seed is unchanged by
/// this module. The digest was taken from the unmodified `origin/main` export
/// (`aa-arith-emit-fit-20261005-210113/run/initial-compiled.bin`, 49,030 bytes).
#[test]
fn default_compiled_export_is_unchanged() {
    let parameters = Parameters::seeded(SEED).expect("seeded");
    let encoded = parameters.compile().expect("compile").encode();
    assert_eq!(encoded.len(), 49_030);
    assert_eq!(
        blake3::hash(&encoded).to_hex().to_string(),
        "3948334fa414a13fc74a514ce8f8079087aba3204e768affb94652b708d6a1ca"
    );
}
