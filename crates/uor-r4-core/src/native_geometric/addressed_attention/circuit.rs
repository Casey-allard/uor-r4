//! Fixed seeded LUT4 topology and deterministic primitive lowering.
//! This wire format is a primitive test format, not a loadable language artifact.

pub const INPUT_BITS: usize = 1024;
pub const WIDTHS: [usize; 4] = [256, 64, 16, 9];
pub const GATES: usize = 345;
pub const GATE_LOGITS: usize = GATES * 16;
pub const CONTEXTS: usize = 512;
pub const HEAD_LOGITS: usize = CONTEXTS * (8 * 120 + 64 + 7 + 257) + 240;
pub const COMPILED_BYTES: usize = 49_022;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitError {
    Shape,
    NonFinite,
    Topology,
    Domain,
    EmptyLegalSet,
    Wire,
}
impl std::fmt::Display for CircuitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CircuitError {}
pub type Result<T> = std::result::Result<T, CircuitError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Topology {
    wires: [[u16; 4]; GATES],
}
impl Topology {
    /// Host-only seeded wiring. Every predecessor reaches the next layer.
    pub fn seeded(seed: u64) -> Self {
        let mut state = seed ^ 0x9e3779b97f4a7c15;
        let mut wires = [[0; 4]; GATES];
        let mut start = 0;
        let mut previous = INPUT_BITS;
        for width in WIDTHS {
            let mut permutation: Vec<_> = (0..previous as u16).collect();
            for i in (1..previous).rev() {
                state = state.wrapping_add(0x9e3779b97f4a7c15);
                let mut z = state;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
                let j = ((z ^ (z >> 31)) % (i as u64 + 1)) as usize;
                permutation.swap(i, j);
            }
            for gate in 0..width {
                for pin in 0..4 {
                    wires[start + gate][pin] = permutation[(gate * 4 + pin) % previous];
                }
            }
            start += width;
            previous = width;
        }
        Self { wires }
    }

    pub fn validate(&self) -> Result<()> {
        let mut start = 0;
        let mut previous = INPUT_BITS;
        for width in WIDTHS {
            let mut seen = [false; INPUT_BITS];
            for pins in &self.wires[start..start + width] {
                for (i, &pin) in pins.iter().enumerate() {
                    if usize::from(pin) >= previous || pins[..i].contains(&pin) {
                        return Err(CircuitError::Topology);
                    }
                    seen[usize::from(pin)] = true;
                }
            }
            if !seen[..previous].iter().all(|&x| x) {
                return Err(CircuitError::Topology);
            }
            start += width;
            previous = width;
        }
        Ok(())
    }

    /// Each callback receives the actual visited Bernoulli parameter row.
    /// A caller samples anew on every invocation, including a repeated row.
    pub fn evaluate_with<F>(&self, input: &[bool; INPUT_BITS], mut bit: F) -> Result<u16>
    where
        F: FnMut(usize) -> Result<bool>,
    {
        let mut prior = *input;
        let mut next = [false; INPUT_BITS];
        let mut start = 0;
        for width in WIDTHS {
            for (local, pins) in self.wires[start..start + width].iter().enumerate() {
                let mut address = 0;
                for (pin, &source) in pins.iter().enumerate() {
                    address |= usize::from(prior[usize::from(source)]) << pin;
                }
                next[local] = bit(((start + local) << 4) | address)?;
            }
            std::mem::swap(&mut prior, &mut next);
            start += width;
        }
        Ok(prior[..9]
            .iter()
            .enumerate()
            .fold(0, |address, (i, &b)| address | (u16::from(b) << i)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledCircuit {
    topology: Topology,
    tables: [u16; GATES],
}
impl CompiledCircuit {
    /// Host lowering; zero ties compile to zero. No logits remain in this value.
    pub fn compile(topology: Topology, logits: &[f64]) -> Result<Self> {
        topology.validate()?;
        if logits.len() != GATE_LOGITS {
            return Err(CircuitError::Shape);
        }
        if logits.iter().any(|x| !x.is_finite()) {
            return Err(CircuitError::NonFinite);
        }
        let mut tables = [0; GATES];
        for (gate, row) in logits.chunks_exact(16).enumerate() {
            for (address, &logit) in row.iter().enumerate() {
                tables[gate] |= u16::from(logit > 0.0) << address;
            }
        }
        Ok(Self { topology, tables })
    }

    pub fn evaluate(&self, input: &[bool; INPUT_BITS]) -> Result<u16> {
        self.topology.evaluate_with(input, |row| {
            Ok((self.tables[row >> 4] >> (row & 15)) & 1 != 0)
        })
    }
}

/// Head-major, then context-major rows: 8 roots, extent, control, emission, 2 Null.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledHeads {
    roots: [[u16; 8]; CONTEXTS],
    extent: [[u8; 64]; CONTEXTS],
    control: [[u8; 7]; CONTEXTS],
    emission: [u16; CONTEXTS],
    null: [u16; 2],
}
fn order(row: &[f64]) -> Vec<usize> {
    let mut indices: Vec<_> = (0..row.len()).collect();
    indices.sort_by(|&a, &b| {
        if row[a] == row[b] {
            a.cmp(&b)
        } else {
            row[b].total_cmp(&row[a])
        }
    });
    indices
}
impl CompiledHeads {
    pub fn compile(logits: &[f64]) -> Result<Self> {
        if logits.len() != HEAD_LOGITS {
            return Err(CircuitError::Shape);
        }
        if logits.iter().any(|x| !x.is_finite()) {
            return Err(CircuitError::NonFinite);
        }
        let mut heads = Self {
            roots: [[0; 8]; CONTEXTS],
            extent: [[0; 64]; CONTEXTS],
            control: [[0; 7]; CONTEXTS],
            emission: [0; CONTEXTS],
            null: [0; 2],
        };
        let mut pos = 0;
        for head in 0..8 {
            for row in &mut heads.roots {
                row[head] = order(&logits[pos..pos + 120])[0] as u16;
                pos += 120;
            }
        }
        for row in &mut heads.extent {
            for (r, i) in row.iter_mut().zip(order(&logits[pos..pos + 64])) {
                *r = i as u8;
            }
            pos += 64;
        }
        for row in &mut heads.control {
            for (r, i) in row.iter_mut().zip(order(&logits[pos..pos + 7])) {
                *r = i as u8;
            }
            pos += 7;
        }
        for row in &mut heads.emission {
            *row = order(&logits[pos..pos + 257])[0] as u16;
            pos += 257;
        }
        for root in &mut heads.null {
            *root = order(&logits[pos..pos + 120])[0] as u16;
            pos += 120;
        }
        heads.validate()?;
        Ok(heads)
    }
    fn validate(&self) -> Result<()> {
        for row in &self.roots {
            if row.iter().any(|&r| r >= 120) {
                return Err(CircuitError::Domain);
            }
        }
        if self.emission.iter().any(|&v| v > 256) || self.null.iter().any(|&v| v >= 120) {
            return Err(CircuitError::Domain);
        }
        for row in &self.extent {
            permutation(row)?;
        }
        for row in &self.control {
            permutation(row)?;
        }
        Ok(())
    }
    pub fn roots(&self, context: u16) -> Result<[u16; 8]> {
        self.roots
            .get(usize::from(context))
            .copied()
            .ok_or(CircuitError::Domain)
    }
    pub fn null_roots(&self) -> [u16; 2] {
        self.null
    }
    pub fn emission(&self, context: u16) -> Result<u16> {
        self.emission
            .get(usize::from(context))
            .copied()
            .ok_or(CircuitError::Domain)
    }
    pub fn extent(&self, context: u16, legal: &[bool; 64]) -> Result<u8> {
        let row = self
            .extent
            .get(usize::from(context))
            .ok_or(CircuitError::Domain)?;
        Ok(first_legal(row, legal)? + 1)
    }
    pub fn control(&self, context: u16, legal: &[bool; 7]) -> Result<u8> {
        first_legal(
            self.control
                .get(usize::from(context))
                .ok_or(CircuitError::Domain)?,
            legal,
        )
    }
}
fn permutation<const N: usize>(row: &[u8; N]) -> Result<()> {
    let mut seen = [false; N];
    for &v in row {
        let cell = seen.get_mut(usize::from(v)).ok_or(CircuitError::Domain)?;
        if *cell {
            return Err(CircuitError::Domain);
        }
        *cell = true;
    }
    Ok(())
}
fn first_legal<const N: usize>(row: &[u8; N], legal: &[bool; N]) -> Result<u8> {
    row.iter()
        .copied()
        .find(|&v| legal[usize::from(v)])
        .ok_or(CircuitError::EmptyLegalSet)
}

/// Fixed-length, little-endian primitive payload. Normal artifact/data/geometry
/// binding belongs to the future full model envelope and is not implied here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrimitiveExport {
    pub circuit: CompiledCircuit,
    pub heads: CompiledHeads,
}
impl PrimitiveExport {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(COMPILED_BYTES + 8);
        out.extend_from_slice(b"UORLUT01");
        for pins in &self.circuit.topology.wires {
            for &v in pins {
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
        for v in self.circuit.tables {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for head in 0..8 {
            for row in &self.heads.roots {
                out.extend_from_slice(&row[head].to_le_bytes());
            }
        }
        for row in &self.heads.extent {
            out.extend_from_slice(row);
        }
        for row in &self.heads.control {
            out.extend_from_slice(row);
        }
        for v in self.heads.emission {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for v in self.heads.null {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != COMPILED_BYTES + 8 || bytes.get(..8) != Some(b"UORLUT01") {
            return Err(CircuitError::Wire);
        }
        let mut rest = &bytes[8..];
        fn u16le(rest: &mut &[u8]) -> u16 {
            let v = u16::from_le_bytes([rest[0], rest[1]]);
            *rest = &rest[2..];
            v
        }
        let mut topology = Topology {
            wires: [[0; 4]; GATES],
        };
        for pins in &mut topology.wires {
            for v in pins {
                *v = u16le(&mut rest);
            }
        }
        topology.validate()?;
        let mut tables = [0; GATES];
        for v in &mut tables {
            *v = u16le(&mut rest);
        }
        let mut heads = CompiledHeads {
            roots: [[0; 8]; CONTEXTS],
            extent: [[0; 64]; CONTEXTS],
            control: [[0; 7]; CONTEXTS],
            emission: [0; CONTEXTS],
            null: [0; 2],
        };
        for head in 0..8 {
            for row in &mut heads.roots {
                row[head] = u16le(&mut rest);
            }
        }
        for row in &mut heads.extent {
            row.copy_from_slice(&rest[..64]);
            rest = &rest[64..];
        }
        for row in &mut heads.control {
            row.copy_from_slice(&rest[..7]);
            rest = &rest[7..];
        }
        for v in &mut heads.emission {
            *v = u16le(&mut rest);
        }
        for v in &mut heads.null {
            *v = u16le(&mut rest);
        }
        heads.validate()?;
        Ok(Self {
            circuit: CompiledCircuit { topology, tables },
            heads,
        })
    }
}

#[cfg(test)]
mod tests;
