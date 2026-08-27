use std::collections::BTreeSet;

use uor_r4_core::prime_route_attention::{PrimeRegistry, SemanticAtom};
use uor_r4_core::spiralcore_operator::{
    cl06_bivectors, cl06_finite_group, cl06_killing_form, cl06_left_generators, cl06_left_volume,
    cl06_right_generators, cl06_right_volume, octonion_basis_associator,
    octonion_left_basis_matrix, octonion_right_basis_matrix, spiralcore_operator_kappa,
    validate_spiralcore_v63_operator, SignedMatrix8, SixPrimeOperatorChart,
    SpiralCoreOperatorError, CANONICAL_SIX_PRIME_VALUES, CHART_TRANSPORT_STATUS,
    OCTONION_FANO_CYCLES, OPERATOR_SEMANTIC_STATUS, SPIRALCORE_OPERATOR_KAPPA_REFERENCE,
    SPIRALCORE_V63_REFERENCE_SHA256,
};

const FIXTURE_CHART_KAPPA_REFERENCE: &str =
    "blake3:4924602ccdee712fda8de26163fc74d1f014adac377d10d6be6cd57ee34df8a8";

fn deterministic_label(seed: &str) -> String {
    format!("blake3:{}", blake3::hash(seed.as_bytes()).to_hex())
}

fn fixture_registry(extra: bool) -> PrimeRegistry {
    let mut ids = vec!["alpha", "beta", "delta", "epsilon", "gamma", "zeta"];
    if extra {
        ids.push("zz-extra");
    }
    PrimeRegistry::compile(
        &ids.into_iter()
            .map(|id| SemanticAtom {
                semantic_atom_id: id.to_owned(),
                payload_cid: deterministic_label(&format!("payload-{id}")),
            })
            .collect::<Vec<_>>(),
    )
    .expect("canonical registry")
}

fn carrier_values(chart: &SixPrimeOperatorChart) -> [u32; 6] {
    std::array::from_fn(|index| chart.carriers()[index].atom.value())
}

#[test]
fn exact_fano_convention_retains_non_associativity() {
    assert_eq!(
        OCTONION_FANO_CYCLES,
        [
            [1, 2, 4],
            [2, 3, 5],
            [3, 4, 6],
            [4, 5, 7],
            [5, 6, 1],
            [6, 7, 2],
            [7, 1, 3],
        ]
    );
    let mut expected = [0i8; 8];
    expected[6] = -2;
    assert_eq!(octonion_basis_associator(1, 2, 3).unwrap(), expected);
    assert_eq!(SPIRALCORE_V63_REFERENCE_SHA256.len(), 64);
}

#[test]
fn left_and_right_cl06_representations_match_v63_exactly() {
    let identity = SignedMatrix8::identity();
    let negative_identity = identity.negated().unwrap();
    for generators in [
        cl06_left_generators().unwrap(),
        cl06_right_generators().unwrap(),
    ] {
        for first in 0..6 {
            for second in 0..6 {
                let forward = generators[first].checked_mul(generators[second]).unwrap();
                let reverse = generators[second].checked_mul(generators[first]).unwrap();
                for row in 0..8 {
                    for column in 0..8 {
                        let observed = i16::from(forward.entries()[row][column])
                            + i16::from(reverse.entries()[row][column]);
                        let expected = if first == second && row == column {
                            -2
                        } else {
                            0
                        };
                        assert_eq!(observed, expected, "generator pair ({first},{second})");
                    }
                }
            }
        }
    }
    assert_eq!(
        cl06_left_volume().unwrap(),
        octonion_left_basis_matrix(7).unwrap()
    );
    assert_eq!(
        cl06_right_volume().unwrap(),
        octonion_right_basis_matrix(7).unwrap().negated().unwrap()
    );
    assert_ne!(negative_identity, identity);
}

#[test]
fn bivectors_close_with_exact_killing_form_and_generate_64_states() {
    let identity = SignedMatrix8::identity();
    let negative_identity = identity.negated().unwrap();
    let bivectors = cl06_bivectors().unwrap();
    assert_eq!(bivectors.len(), 15);
    for (index, bivector) in bivectors.iter().enumerate() {
        assert_eq!(bivector.index as usize, index);
        assert_eq!(bivector.matrix.checked_power(2).unwrap(), negative_identity);
        assert_eq!(bivector.matrix.checked_power(4).unwrap(), identity);
    }
    assert_eq!(cl06_finite_group().unwrap().len(), 64);
    let killing = cl06_killing_form().unwrap();
    for (row, values) in killing.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            assert_eq!(*value, if row == column { -32 } else { 0 });
        }
    }
}

#[test]
fn six_prime_chart_is_j62_ordered_and_kappa_bound() {
    let registry = fixture_registry(false);
    let chart = SixPrimeOperatorChart::from_registry(&registry).unwrap();
    chart.validate().unwrap();
    assert_eq!(carrier_values(&chart), CANONICAL_SIX_PRIME_VALUES);
    assert_eq!(chart.slots().len(), 15);
    let mut products = BTreeSet::new();
    for (index, slot) in chart.slots().iter().enumerate() {
        assert_eq!(slot.index as usize, index);
        assert_eq!(slot.carrier_slots, slot.bivector.pair);
        assert!(products.insert(slot.expert.product()));
    }
    assert_eq!(products.len(), 15);
    for (left_index, left) in chart.slots().iter().enumerate() {
        for (right_index, right) in chart.slots().iter().enumerate() {
            if left_index == right_index {
                continue;
            }
            let shared_slots = left
                .carrier_slots
                .iter()
                .filter(|slot| right.carrier_slots.contains(slot))
                .count();
            assert_eq!(
                left.expert.handoff(right.expert).is_some(),
                shared_slots == 1,
                "J(6,2) adjacency mismatch for slots {left_index},{right_index}"
            );
        }
    }
    assert_eq!(chart.operator_kappa(), spiralcore_operator_kappa().unwrap());
    assert_eq!(chart.chart_kappa(), chart.reproduce_kappa().unwrap());
    assert_eq!(chart.chart_kappa(), FIXTURE_CHART_KAPPA_REFERENCE);
    assert_eq!(CHART_TRANSPORT_STATUS, "NOT_ESTABLISHED");
    assert_eq!(OPERATOR_SEMANTIC_STATUS, "OPTIONAL_CONTROL_PENDING");

    let extended = SixPrimeOperatorChart::from_registry(&fixture_registry(true)).unwrap();
    assert_eq!(carrier_values(&chart), carrier_values(&extended));
    assert_ne!(
        chart.source_registry_kappa(),
        extended.source_registry_kappa()
    );
    assert_ne!(chart.chart_kappa(), extended.chart_kappa());
}

#[test]
fn chart_rejects_a_forged_canonical_looking_registry_kappa() {
    let mut registry = fixture_registry(false);
    registry.registry_kappa = deterministic_label("forged-registry-kappa");
    let error = SixPrimeOperatorChart::from_registry(&registry).unwrap_err();
    assert!(
        matches!(
            &error,
            SpiralCoreOperatorError::InvalidRegistry(reason)
                if reason.contains("prime registry kappa does not reproduce")
        ),
        "unexpected registry rejection: {error}"
    );
}

#[test]
fn chart_rejects_malformed_or_unbound_registry_tail_bindings() {
    let registry = fixture_registry(true);

    let mut unbound_tail = registry.clone();
    unbound_tail.bindings[6].payload_cid = deterministic_label("tampered-tail-payload");
    let error = SixPrimeOperatorChart::from_registry(&unbound_tail).unwrap_err();
    assert!(
        matches!(
            &error,
            SpiralCoreOperatorError::InvalidRegistry(reason)
                if reason.contains("prime registry kappa does not reproduce")
        ),
        "unexpected tail-kappa rejection: {error}"
    );

    let mut malformed_tail = registry;
    malformed_tail.bindings[6].atom = malformed_tail.bindings[5].atom;
    let error = SixPrimeOperatorChart::from_registry(&malformed_tail).unwrap_err();
    assert!(
        matches!(
            &error,
            SpiralCoreOperatorError::InvalidRegistry(reason)
                if reason.contains("prime registry repeats a prime")
        ),
        "unexpected malformed-tail rejection: {error}"
    );
}

#[test]
fn aggregate_validator_reports_every_exact_fixture() {
    let report = validate_spiralcore_v63_operator().unwrap();
    assert_eq!(report.left_anticommutators, 36);
    assert_eq!(report.right_anticommutators, 36);
    assert_eq!(report.bivectors, 15);
    assert_eq!(report.commutators, 225);
    assert_eq!(report.finite_group_size, 64);
    assert_eq!(report.killing_diagonal, -32);
    assert_eq!(report.operator_kappa, spiralcore_operator_kappa().unwrap());
    assert_eq!(report.operator_kappa, SPIRALCORE_OPERATOR_KAPPA_REFERENCE);
}
