use super::*;

fn parity_parameters() -> Vec<f64> {
    (0..GATE_LOGITS)
        .map(|row| {
            if (row & 15).count_ones() & 1 == 1 {
                1.0
            } else {
                -1.0
            }
        })
        .collect()
}
fn independent(topology: &Topology, logits: &[f64], input: &[bool; INPUT_BITS]) -> u16 {
    let mut values = input.to_vec();
    let mut first = 0;
    for width in WIDTHS {
        let mut layer = Vec::with_capacity(width);
        for gate in first..first + width {
            let pins = topology.wires[gate];
            let address = usize::from(values[pins[0] as usize])
                + 2 * usize::from(values[pins[1] as usize])
                + 4 * usize::from(values[pins[2] as usize])
                + 8 * usize::from(values[pins[3] as usize]);
            layer.push(logits[gate * 16 + address] > 0.0);
        }
        values = layer;
        first += width;
    }
    values.iter().rev().fold(0, |a, &b| a * 2 + u16::from(b))
}
#[test]
fn addressed_circuit_every_input_reaches_output_and_lowering_matches() {
    let topology = Topology::seeded(7341);
    topology.validate().unwrap();
    assert_eq!(topology, Topology::seeded(7341));
    assert_ne!(topology, Topology::seeded(7342));
    let logits = parity_parameters();
    let compiled = CompiledCircuit::compile(topology.clone(), &logits).unwrap();
    let mut input = [false; INPUT_BITS];
    let zero = compiled.evaluate(&input).unwrap();
    for bit in 0..INPUT_BITS {
        input[bit] = true;
        let output = compiled.evaluate(&input).unwrap();
        assert_ne!(zero, output, "unreachable input {bit}");
        assert_eq!(output, independent(&topology, &logits, &input));
        input[bit] = false;
    }
    for row in 0..16 {
        // Each local address is interpreted least-significant pin first.
        for gate in 0..GATES {
            assert_eq!(
                (compiled.tables[gate] >> row) & 1,
                (row as u16).count_ones() as u16 & 1
            );
        }
    }
    let mut bad = topology;
    bad.wires[256][0] = 256;
    assert_eq!(bad.validate(), Err(CircuitError::Topology));
}
#[test]
fn addressed_circuit_reused_rows_request_new_decisions() {
    let topology = Topology::seeded(19);
    let input = [false; INPUT_BITS];
    let mut visits = vec![0_u8; GATE_LOGITS];
    let first = topology
        .evaluate_with(&input, |row| {
            visits[row] += 1;
            Ok(false)
        })
        .unwrap();
    let second = topology
        .evaluate_with(&input, |row| {
            visits[row] += 1;
            Ok(true)
        })
        .unwrap();
    assert_eq!(first, 0);
    assert_eq!(second, 511);
    // The entire first layer reused precisely the same rows but reevaluated them.
    for gate in 0..256 {
        assert_eq!(visits[gate * 16], 2);
    }
}
#[test]
fn addressed_export_modes_masks_ties_roundtrip_and_refusals() {
    assert_eq!(order(&[-0.0, 0.0, -0.0]), vec![0, 1, 2]);
    let mut logits = vec![0.0; HEAD_LOGITS];
    // Non-default ties test lowest-index mode; full orders must retain fallback.
    for head in 0..8 {
        for context in 0..CONTEXTS {
            let offset = (head * CONTEXTS + context) * 120;
            logits[offset + 3] = 2.0;
            logits[offset + 5] = 2.0;
        }
    }
    let extent_start = 8 * CONTEXTS * 120;
    let control_start = extent_start + CONTEXTS * 64;
    let emission_start = control_start + CONTEXTS * 7;
    for context in 0..CONTEXTS {
        logits[extent_start + context * 64 + 63] = 3.0;
        logits[control_start + context * 7 + 4] = 3.0;
        logits[emission_start + context * 257 + 256] = 2.0;
    }
    let heads = CompiledHeads::compile(&logits).unwrap();
    for context in 0..CONTEXTS as u16 {
        assert_eq!(heads.roots(context).unwrap(), [3; 8]);
        assert_eq!(heads.emission(context).unwrap(), 256);
        let mut legal = [true; 64];
        assert_eq!(heads.extent(context, &legal).unwrap(), 64);
        legal[63] = false;
        assert_eq!(heads.extent(context, &legal).unwrap(), 1);
        for length in 0..64 {
            let mut sole = [false; 64];
            sole[length] = true;
            assert_eq!(heads.extent(context, &sole).unwrap(), length as u8 + 1);
        }
        assert_eq!(
            heads.extent(context, &[false; 64]),
            Err(CircuitError::EmptyLegalSet)
        );
        let mut controls = [true; 7];
        assert_eq!(heads.control(context, &controls).unwrap(), 4);
        controls[4] = false;
        assert_eq!(heads.control(context, &controls).unwrap(), 0);
        assert_eq!(
            heads.control(context, &[false; 7]),
            Err(CircuitError::EmptyLegalSet)
        );
    }
    assert_eq!(heads.emission(512), Err(CircuitError::Domain));
    let lowered = PrimitiveExport {
        circuit: CompiledCircuit::compile(Topology::seeded(7341), &parity_parameters()).unwrap(),
        heads,
    };
    let wire = lowered.encode();
    assert_eq!(wire.len(), COMPILED_BYTES + 8);
    assert_eq!(PrimitiveExport::decode(&wire).unwrap(), lowered);
    let mut duplicate = wire.clone();
    let extent_offset = 8 + GATES * 10 + CONTEXTS * 8 * 2;
    duplicate[extent_offset + 1] = duplicate[extent_offset];
    assert_eq!(
        PrimitiveExport::decode(&duplicate),
        Err(CircuitError::Domain)
    );
    let mut invalid_root = wire.clone();
    invalid_root[8 + GATES * 10] = 120;
    assert_eq!(
        PrimitiveExport::decode(&invalid_root),
        Err(CircuitError::Domain)
    );
    let mut trailing = wire.clone();
    trailing.push(0);
    assert_eq!(PrimitiveExport::decode(&trailing), Err(CircuitError::Wire));
    for length in [0, 7, 8, 100, wire.len() - 1] {
        assert!(PrimitiveExport::decode(&wire[..length]).is_err());
    }
    logits[0] = f64::NAN;
    assert_eq!(
        CompiledHeads::compile(&logits),
        Err(CircuitError::NonFinite)
    );
    assert_eq!(
        CompiledCircuit::compile(Topology::seeded(0), &[0.0]),
        Err(CircuitError::Shape)
    );
    let mut gates = vec![0.0; GATE_LOGITS];
    gates[1] = f64::INFINITY;
    assert_eq!(
        CompiledCircuit::compile(Topology::seeded(0), &gates),
        Err(CircuitError::NonFinite)
    );
}

#[test]
fn addressed_circuit_offline_sampler_visits_actual_rows_and_replays() {
    use crate::native_geometric::addressed_attention::training::{bernoulli, EventKey, EventKind};
    let topology = Topology::seeded(37);
    let input = [false; INPUT_BITS];
    let execute = |invocation: u64| {
        let mut events = Vec::new();
        let output = topology
            .evaluate_with(&input, |row| {
                let event = EventKey {
                    seed: 973,
                    batch: 0,
                    particle: 0,
                    document: 0,
                    event: invocation * GATES as u64 + events.len() as u64,
                    phase: 0,
                    kind: EventKind::Gate,
                    slot: (row >> 4) as u32,
                };
                let draw = bernoulli(0.0, event).map_err(|_| CircuitError::Domain)?;
                events.push((row, draw.outcome, draw.score));
                Ok(draw.outcome)
            })
            .unwrap();
        (output, events)
    };
    let first = execute(0);
    assert_eq!(first, execute(0));
    let second = execute(1);
    assert_eq!(first.1.len(), GATES);
    assert_eq!(second.1.len(), GATES);
    let mut score = vec![0.0; GATE_LOGITS];
    for &(row, outcome, s) in first.1.iter().chain(&second.1) {
        assert_eq!(s, f64::from(u8::from(outcome)) - 0.5);
        score[row] += s;
    }
    for gate in 0..256 {
        assert_eq!(first.1[gate].0, second.1[gate].0);
        assert_eq!(score[gate * 16], first.1[gate].2 + second.1[gate].2);
    }
    assert!(first.1[..256]
        .iter()
        .zip(&second.1)
        .any(|(a, b)| a.1 != b.1));
}
