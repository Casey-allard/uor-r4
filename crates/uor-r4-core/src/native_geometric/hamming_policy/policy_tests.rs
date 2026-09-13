use super::*;
use crate::native_geometric::addressed_attention::objects::{Action, ObjectSession};

fn fields(g: &BoundGeometry) -> Fields {
    Fields {
        initial_roots: [g.identity(); 4],
        roots: [g.identity(); 4],
        selected: None,
        previous: None,
        active: None,
        last_observed: None,
        zeta_bins: [0; 4],
        position: 0,
        round: 0,
        numeric_valid: [false; 2],
    }
}
fn environment(g: &BoundGeometry) -> Environment {
    Environment {
        last_observed: None,
        zeta_bins: [0; 4],
        position: 0,
        initial_roots: [g.identity(); 4],
    }
}
fn lease(g: &BoundGeometry, prefix: usize, bytes: &[u8], roots: [u16; 4]) -> Lease {
    let mut objects = ObjectSession::new([17; 32], g.identity_digest(), 7);
    for _ in 0..prefix {
        objects
            .observe_input(b'_', [g.identity(); 2], [g.identity(); 4])
            .unwrap();
    }
    for &byte in bytes {
        objects.observe_input(byte, [1, 2], roots).unwrap();
    }
    objects
        .acquire_occurrence(7, prefix as u64, bytes.len() as u8)
        .unwrap()
}

#[test]
fn hamming_policy_topology_carries_every_input_to_geometric_and_emission_outputs() {
    let topology = Topology::seeded(973);
    topology.validate().unwrap();
    for range in [0..480, 768..1796] {
        let mut reached = [false; INPUT_BITS];
        for output in range {
            for &middle in &topology.wires[1280 + output] {
                for &first in &topology.wires[1024 + usize::from(middle)] {
                    for &input in &topology.wires[usize::from(first)] {
                        reached[usize::from(input)] = true;
                    }
                }
            }
        }
        assert!(reached.iter().all(|v| *v));
    }
    let mut invalid = topology.clone();
    invalid.wires[0][0] = 4096;
    assert_eq!(invalid.validate(), Err(Error::Domain));
}

#[test]
fn hamming_policy_packing_retains_fourth_roots_all_payload_bytes_and_excludes_ids() {
    let g = BoundGeometry::canonical().unwrap();
    let bytes: Vec<_> = (0..64).collect();
    let original = lease(&g, 0, &bytes, [3, 4, 5, 6]);
    let relocated = lease(&g, 5, &bytes, [3, 4, 5, 6]);
    assert_ne!(original.reference(), relocated.reference());
    let mut f = fields(&g);
    f.selected = Some(original);
    f.previous = Some(original);
    let packed = pack_fields(&f, Phase::Delta, &g).unwrap();
    let mut relocated_fields = f;
    relocated_fields.selected = Some(relocated);
    relocated_fields.previous = Some(relocated);
    assert_eq!(
        packed,
        pack_fields(&relocated_fields, Phase::Delta, &g).unwrap()
    );
    for slot in 0..2 {
        for position in 0..64 {
            let mut changed_bytes = bytes.clone();
            changed_bytes[position] ^= 0x80;
            let changed = lease(&g, 0, &changed_bytes, [3, 4, 5, 6]);
            let mut changed_fields = f;
            if slot == 0 {
                changed_fields.selected = Some(changed);
            } else {
                changed_fields.previous = Some(changed);
            }
            let new = pack_fields(&changed_fields, Phase::Delta, &g).unwrap();
            let changed_positions: Vec<_> = packed
                .iter()
                .zip(new)
                .enumerate()
                .filter_map(|(i, (a, b))| (*a != b).then_some(i))
                .collect();
            assert_eq!(
                changed_positions,
                vec![1920 + (slot << 9) + (position << 3) + 7]
            );
        }
        for lane in 0..4 {
            let mut roots = [3, 4, 5, 6];
            roots[lane] = 7;
            let changed = lease(&g, 0, &bytes, roots);
            let mut changed_fields = f;
            if slot == 0 {
                changed_fields.selected = Some(changed);
            } else {
                changed_fields.previous = Some(changed);
            }
            assert_ne!(
                packed,
                pack_fields(&changed_fields, Phase::Delta, &g).unwrap()
            );
        }
    }
    let mut invalid = f;
    invalid.roots[3] = 120;
    assert_eq!(pack_fields(&invalid, Phase::Emit, &g), Err(Error::Geometry));
}

#[test]
fn hamming_policy_direct_integer_scores_repeated_cells_and_geometric_decode() {
    let g = BoundGeometry::canonical().unwrap();
    let mut params = Params {
        seed: 973,
        tables: vec![0; GATES],
    };
    // Constant gate tables are an authored interface fixture, not fitted behavior.
    for (lane, root) in [4u16, 13, 27, 89].into_iter().enumerate() {
        let signature = g.signature(root).unwrap();
        for bit in 0..120 {
            params.tables[1280 + lane * 120 + bit] = if (signature[bit >> 6] >> (bit & 63)) & 1 != 0
            {
                u16::MAX
            } else {
                0
            };
        }
    }
    let mut policy = Policy::new(params.clone(), &g, environment(&g), Intervention::Full).unwrap();
    let first = policy.evaluate(&fields(&g), Phase::Emit).unwrap();
    assert_eq!(first.roots, [4, 13, 27, 89]);
    assert_eq!(first.emission(), 0);
    let second = policy.evaluate(&fields(&g), Phase::Emit).unwrap();
    assert_eq!(first, second);
    assert_eq!(policy.audit().calls, 2);
    assert_eq!(policy.audit().gates, (GATES << 1) as u64);
    assert_eq!(
        policy
            .audit()
            .cells
            .iter()
            .map(|&v| u64::from(v))
            .sum::<u64>(),
        (GATES << 1) as u64
    );
    assert!(policy.audit().cells.iter().all(|&v| v == 0 || v == 2));
    // Final EOS score bit receives address zero from the all-zero middle layer.
    params.flip(GATES - 1, 0).unwrap();
    let mut changed = Policy::new(params, &g, environment(&g), Intervention::Full).unwrap();
    let output = changed.evaluate(&fields(&g), Phase::Emit).unwrap();
    assert_eq!(output.emission_scores[256], 8);
    assert_eq!(output.emission(), 256);
    assert_eq!(output.roots, first.roots);
    assert_eq!(
        &output.emission_scores[..256],
        &first.emission_scores[..256]
    );
}

#[test]
fn hamming_policy_legal_argmax_parameter_validation_and_interventions() {
    let g = BoundGeometry::canonical().unwrap();
    let params = Params::seeded(7341);
    assert_eq!(params, Params::seeded(7341));
    assert_ne!(params.digest(), Params::seeded(7342).digest());
    let mut malformed = params.clone();
    malformed.tables.pop();
    assert_eq!(malformed.validate(), Err(Error::Shape));
    assert_eq!(params.clone().flip(GATES, 0), Err(Error::Domain));
    assert_eq!(params.clone().flip(0, 16), Err(Error::Domain));
    let mut decision = Decision {
        roots: [0; 4],
        extent_scores: [15; 64],
        control_scores: [15; 7],
        emission_scores: [0; 257],
    };
    assert_eq!(decision.extent(&[false; 64]), Err(Error::EmptyLegalSet));
    let mut legal = [false; 64];
    legal[4] = true;
    legal[17] = true;
    assert_eq!(decision.extent(&legal), Ok(5));
    decision.extent_scores[17] = 15;
    decision.extent_scores[4] = 14;
    assert_eq!(decision.extent(&legal), Ok(18));
    let mut f = fields(&g);
    f.selected = Some(lease(&g, 0, b"payload", [1, 2, 3, 4]));
    let masked = pack(&f, Phase::Delta, &g, Intervention::PayloadMasked).unwrap();
    assert!(masked[1920..2944].iter().all(|v| !v));
    assert!(masked[2944..2951].iter().all(|v| *v));
    let full = pack_fields(&f, Phase::Delta, &g).unwrap();
    assert_eq!(&masked[..1920], &full[..1920]);
    assert_eq!(&masked[2944..], &full[2944..]);
    let disabled = pack(&f, Phase::Delta, &g, Intervention::ReadDisabled).unwrap();
    assert!(disabled[960..3552].iter().all(|v| !v));
    assert_ne!(pack_fields(&f, Phase::Emit, &g).unwrap(), full);
}

#[test]
fn hamming_policy_active_length_is_visible_and_unknown_parameters_are_rejected() {
    let g = BoundGeometry::canonical().unwrap();
    let mut objects = ObjectSession::new([18; 32], g.identity_digest(), 8);
    for byte in b"xy" {
        objects.observe_input(*byte, [1, 2], [3, 4, 5, 6]).unwrap();
    }
    let mut frames = Vec::new();
    for length in [1, 2] {
        let lease = objects.acquire_occurrence(8, 0, length).unwrap();
        let operands = objects.prepare_operands(Some(lease), None).unwrap();
        let action = objects.prepare_action(operands, Action::AcquireA).unwrap();
        let mut f = fields(&g);
        f.active = action.active().copied();
        let full = pack_fields(&f, Phase::Emit, &g).unwrap();
        let masked = pack(&f, Phase::Emit, &g, Intervention::PayloadMasked).unwrap();
        assert_ne!(full, masked);
        assert!(masked[3635..3643].iter().all(|b| !b));
        assert_eq!(&masked[..3635], &full[..3635]);
        assert_eq!(&masked[3643..], &full[3643..]);
        frames.push(full);
    }
    let differing: Vec<_> = frames[0]
        .iter()
        .zip(frames[1])
        .enumerate()
        .filter_map(|(i, (a, b))| (*a != b).then_some(i))
        .collect();
    assert_eq!(differing, vec![3651, 3652]);
    let mut wire = serde_json::to_value(Params::seeded(973)).unwrap();
    wire["unbound_extra"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Params>(wire).is_err());
}
