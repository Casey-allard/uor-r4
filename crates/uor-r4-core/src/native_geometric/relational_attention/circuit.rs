//! Hard Boolean DAGs with an explicitly biased offline straight-through learner.
//! Serving uses only Boolean operations, integer indexing and table reads.
//! The backward pass differentiates the multilinear extension of each learned
//! LUT at its actual hard inputs, treating output thresholding as identity.
//! This is a surrogate, not the derivative of the discontinuous hard circuit.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// Unified indices: external inputs, followed by earlier nodes.
    pub inputs: [usize; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Circuit {
    pub input_count: usize,
    pub nodes: Vec<Node>,
    pub outputs: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    /// Row index is input[0] + 2 * input[1]. Values threshold at 0.5.
    /// These are offline training values, absent from exported serving tables.
    pub cells: Vec<[f64; 4]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tape {
    /// External inputs and every hard node output, in unified wire order.
    pub values: Vec<bool>,
    pub outputs: Vec<bool>,
}

/// Offline-only smooth surrogate; never used to produce a serving response.
#[derive(Clone, Debug, PartialEq)]
pub struct SoftTape {
    pub values: Vec<f64>,
    pub outputs: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Shape,
    Topology,
    TableDomain,
    NonFinite,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

pub const FALSE: u8 = 0b0000;
pub const AND: u8 = 0b1000;
pub const OR: u8 = 0b1110;
pub const XOR: u8 = 0b0110;
pub const PASS_A: u8 = 0b1010;
pub const PASS_B: u8 = 0b1100;

impl Circuit {
    pub fn validate(&self) -> Result<()> {
        let total = self
            .input_count
            .checked_add(self.nodes.len())
            .ok_or(Error::Shape)?;
        for (i, node) in self.nodes.iter().enumerate() {
            if node.inputs.iter().any(|&wire| wire >= self.input_count + i) {
                return Err(Error::Topology);
            }
        }
        if self.outputs.iter().any(|&wire| wire >= total) {
            return Err(Error::Topology);
        }
        Ok(())
    }
}

impl Parameters {
    pub fn validate(&self, circuit: &Circuit) -> Result<()> {
        circuit.validate()?;
        if self.cells.len() != circuit.nodes.len() {
            return Err(Error::Shape);
        }
        if self.cells.iter().flatten().any(|cell| !cell.is_finite()) {
            return Err(Error::NonFinite);
        }
        Ok(())
    }

    pub fn export(&self) -> Result<Vec<u8>> {
        if self.cells.iter().flatten().any(|cell| !cell.is_finite()) {
            return Err(Error::NonFinite);
        }
        Ok(self
            .cells
            .iter()
            .map(|cells| {
                let mut table = 0;
                for (row, cell) in cells.iter().enumerate() {
                    table |= u8::from(*cell >= 0.5) << row;
                }
                table
            })
            .collect())
    }
}

/// Explicit topology builders can initialize carry and combining nodes directly.
/// Values 0.1/0.9 retain a nonzero input surrogate at pass/AND/OR gates.
pub fn parameters_from_tables(tables: &[u8]) -> Result<Parameters> {
    if tables.iter().any(|table| table & 0xf0 != 0) {
        return Err(Error::TableDomain);
    }
    Ok(Parameters {
        cells: tables
            .iter()
            .map(|table| std::array::from_fn(|row| if table & (1 << row) != 0 { 0.9 } else { 0.1 }))
            .collect(),
    })
}

/// Deterministic offline initializer; topology and carry policy remain explicit.
/// Its seed is a parameter initialization seed, not a learned semantic mapping.
pub fn seeded_parameters(circuit: &Circuit, seed: u64) -> Result<Parameters> {
    circuit.validate()?;
    let mut state = seed ^ 0x72656c6174696f6e;
    let kinds = [AND, OR, XOR, PASS_A, PASS_B];
    let tables: Vec<_> = circuit
        .nodes
        .iter()
        .map(|_| {
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^= z >> 31;
            kinds[(z % kinds.len() as u64) as usize]
        })
        .collect();
    parameters_from_tables(&tables)
}

fn row(a: bool, b: bool) -> usize {
    usize::from(a) | (usize::from(b) << 1)
}

/// Bounded host allocation is deliberate; this is not the frozen R4G1 kernel.
/// Validate an exported artifact once upstream when using a hot serving loop.
pub fn hard(circuit: &Circuit, tables: &[u8], inputs: &[bool]) -> Result<Tape> {
    circuit.validate()?;
    if inputs.len() != circuit.input_count || tables.len() != circuit.nodes.len() {
        return Err(Error::Shape);
    }
    if tables.iter().any(|table| table & 0xf0 != 0) {
        return Err(Error::TableDomain);
    }
    let mut values = Vec::with_capacity(circuit.input_count + circuit.nodes.len());
    values.extend_from_slice(inputs);
    for (node, table) in circuit.nodes.iter().zip(tables) {
        let index = row(values[node.inputs[0]], values[node.inputs[1]]);
        values.push(table & (1 << index) != 0);
    }
    let outputs = circuit.outputs.iter().map(|&wire| values[wire]).collect();
    Ok(Tape { values, outputs })
}

pub fn forward(circuit: &Circuit, parameters: &Parameters, inputs: &[bool]) -> Result<Tape> {
    parameters.validate(circuit)?;
    hard(circuit, &parameters.export()?, inputs)
}

/// Accumulate all output paths, including repeated/shared inputs and direct carries.
/// A cell receives the adjoint only when its hard row was visited. Input adjoints
/// use the two neighboring learned cell values at the other actual hard input.
/// Training updates/clipping and recurrent unrolling belong to the caller.
pub fn backward(
    circuit: &Circuit,
    parameters: &Parameters,
    tape: &Tape,
    output_adjoints: &[f64],
) -> Result<(Vec<[f64; 4]>, Vec<f64>)> {
    parameters.validate(circuit)?;
    if tape.values.len() != circuit.input_count + circuit.nodes.len()
        || tape.outputs.len() != circuit.outputs.len()
        || output_adjoints.len() != circuit.outputs.len()
    {
        return Err(Error::Shape);
    }
    if output_adjoints.iter().any(|x| !x.is_finite()) {
        return Err(Error::NonFinite);
    }
    let mut adjoints = vec![0.0; tape.values.len()];
    for (&wire, &adjoint) in circuit.outputs.iter().zip(output_adjoints) {
        adjoints[wire] += adjoint;
    }
    let mut cells = vec![[0.0; 4]; circuit.nodes.len()];
    for i in (0..circuit.nodes.len()).rev() {
        let [a, b] = circuit.nodes[i].inputs;
        let index = row(tape.values[a], tape.values[b]);
        let gradient = adjoints[circuit.input_count + i];
        cells[i][index] += gradient;
        let table = parameters.cells[i];
        adjoints[a] += gradient * (table[index | 1] - table[index & !1]);
        adjoints[b] += gradient * (table[index | 2] - table[index & !2]);
    }
    if adjoints.iter().any(|x| !x.is_finite()) || cells.iter().flatten().any(|x| !x.is_finite()) {
        return Err(Error::NonFinite);
    }
    adjoints.truncate(circuit.input_count);
    Ok((cells, adjoints))
}

/// Evaluate the multilinear relaxation without intermediate hard thresholds.
/// The caller still evaluates and trains against its actual hard trajectory.
/// These fractional values are an optimizer approximation, never legal H4 states.
pub fn soft_forward(
    circuit: &Circuit,
    parameters: &Parameters,
    inputs: &[f64],
) -> Result<SoftTape> {
    parameters.validate(circuit)?;
    if inputs.len() != circuit.input_count {
        return Err(Error::Shape);
    }
    if inputs.iter().any(|x| !x.is_finite()) {
        return Err(Error::NonFinite);
    }
    let mut values = Vec::with_capacity(circuit.input_count + circuit.nodes.len());
    values.extend_from_slice(inputs);
    for (node, t) in circuit.nodes.iter().zip(&parameters.cells) {
        let a = values[node.inputs[0]];
        let b = values[node.inputs[1]];
        values.push((1.0 - b) * ((1.0 - a) * t[0] + a * t[1]) + b * ((1.0 - a) * t[2] + a * t[3]));
    }
    if values.iter().any(|x| !x.is_finite()) {
        return Err(Error::NonFinite);
    }
    let outputs = circuit.outputs.iter().map(|&wire| values[wire]).collect();
    Ok(SoftTape { values, outputs })
}

/// Exact derivative of `soft_forward`, used only as a biased hard-path surrogate.
pub fn soft_backward(
    circuit: &Circuit,
    parameters: &Parameters,
    tape: &SoftTape,
    output_adjoints: &[f64],
) -> Result<(Vec<[f64; 4]>, Vec<f64>)> {
    parameters.validate(circuit)?;
    if tape.values.len() != circuit.input_count + circuit.nodes.len()
        || tape.outputs.len() != circuit.outputs.len()
        || output_adjoints.len() != circuit.outputs.len()
    {
        return Err(Error::Shape);
    }
    if tape
        .values
        .iter()
        .chain(output_adjoints)
        .any(|x| !x.is_finite())
    {
        return Err(Error::NonFinite);
    }
    let mut adjoints = vec![0.0; tape.values.len()];
    for (&wire, &gradient) in circuit.outputs.iter().zip(output_adjoints) {
        adjoints[wire] += gradient;
    }
    let mut cells = vec![[0.0; 4]; circuit.nodes.len()];
    for i in (0..circuit.nodes.len()).rev() {
        let [a_wire, b_wire] = circuit.nodes[i].inputs;
        let a = tape.values[a_wire];
        let b = tape.values[b_wire];
        let gradient = adjoints[circuit.input_count + i];
        cells[i] = [
            gradient * (1.0 - a) * (1.0 - b),
            gradient * a * (1.0 - b),
            gradient * (1.0 - a) * b,
            gradient * a * b,
        ];
        let t = parameters.cells[i];
        adjoints[a_wire] += gradient * ((1.0 - b) * (t[1] - t[0]) + b * (t[3] - t[2]));
        adjoints[b_wire] += gradient * ((1.0 - a) * (t[2] - t[0]) + a * (t[3] - t[1]));
    }
    if adjoints.iter().any(|x| !x.is_finite()) || cells.iter().flatten().any(|x| !x.is_finite()) {
        return Err(Error::NonFinite);
    }
    adjoints.truncate(circuit.input_count);
    Ok((cells, adjoints))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_truth_tables_match_hard_training_path() -> Result<()> {
        let circuit = Circuit {
            input_count: 2,
            nodes: vec![Node { inputs: [0, 1] }],
            outputs: vec![2],
        };
        for table in 0..16 {
            let parameters = parameters_from_tables(&[table])?;
            for index in 0..4 {
                let inputs = [index & 1 != 0, index & 2 != 0];
                let trained = forward(&circuit, &parameters, &inputs)?;
                assert_eq!(trained, hard(&circuit, &parameters.export()?, &inputs)?);
                assert_eq!(trained.outputs, vec![table & (1 << index) != 0]);
            }
        }
        Ok(())
    }

    #[test]
    fn surrogate_accumulates_shared_paths_and_direct_carries() -> Result<()> {
        let circuit = Circuit {
            input_count: 2,
            nodes: vec![Node { inputs: [0, 1] }, Node { inputs: [2, 2] }],
            outputs: vec![3, 2, 0],
        };
        let parameters = parameters_from_tables(&[XOR, OR])?;
        let tape = forward(&circuit, &parameters, &[true, true])?;
        let (cells, inputs) = backward(&circuit, &parameters, &tape, &[1.0, 2.0, 3.0])?;
        // XOR hard output is false, so both OR input derivatives are +0.8.
        assert!((cells[0][3] - 3.6).abs() < 1e-12);
        assert_eq!(cells[1], [1.0, 0.0, 0.0, 0.0]);
        assert!((inputs[0] - 0.12).abs() < 1e-12);
        assert!((inputs[1] + 2.88).abs() < 1e-12);
        Ok(())
    }

    #[test]
    fn malformed_topologies_and_nonfinite_parameters_are_rejected() {
        let circuit = Circuit {
            input_count: 1,
            nodes: vec![Node { inputs: [0, 1] }],
            outputs: vec![1],
        };
        assert_eq!(circuit.validate(), Err(Error::Topology));
        assert_eq!(parameters_from_tables(&[16]), Err(Error::TableDomain));
        assert_eq!(
            Parameters {
                cells: vec![[f64::NAN; 4]]
            }
            .export(),
            Err(Error::NonFinite)
        );
    }

    #[test]
    fn soft_backward_matches_finite_differences_with_shared_wires() -> Result<()> {
        let circuit = Circuit {
            input_count: 2,
            nodes: vec![Node { inputs: [0, 1] }, Node { inputs: [2, 2] }],
            outputs: vec![3, 2],
        };
        let parameters = parameters_from_tables(&[XOR, AND])?;
        let inputs = [0.23, 0.61];
        let tape = soft_forward(&circuit, &parameters, &inputs)?;
        let (cells, input_gradients) = soft_backward(&circuit, &parameters, &tape, &[0.7, -0.3])?;
        let objective = |p: &Parameters, x: &[f64]| -> Result<f64> {
            let outputs = soft_forward(&circuit, p, x)?.outputs;
            Ok(0.7 * outputs[0] - 0.3 * outputs[1])
        };
        let step = 1e-6;
        for (node, cell_gradients) in cells.iter().enumerate() {
            for (row, &gradient) in cell_gradients.iter().enumerate() {
                let mut plus = parameters.clone();
                let mut minus = parameters.clone();
                plus.cells[node][row] += step;
                minus.cells[node][row] -= step;
                let finite =
                    (objective(&plus, &inputs)? - objective(&minus, &inputs)?) / (2.0 * step);
                assert!((gradient - finite).abs() < 1e-8);
            }
        }
        for (i, &gradient) in input_gradients.iter().enumerate() {
            let mut plus = inputs;
            let mut minus = inputs;
            plus[i] += step;
            minus[i] -= step;
            let finite =
                (objective(&parameters, &plus)? - objective(&parameters, &minus)?) / (2.0 * step);
            assert!((gradient - finite).abs() < 1e-8);
        }
        Ok(())
    }
}
