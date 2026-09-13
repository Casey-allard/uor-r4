//! Compiled serving decisions and explicit offline parameter interpreters.
use super::circuit::{
    CompiledCircuit, CompiledHeads, PrimitiveExport, Topology, GATE_LOGITS, HEAD_LOGITS, INPUT_BITS,
};
use super::engine::{EngineError, Head, Policy};
use super::inputs::Phase;
use super::training::{bernoulli, categorical, cross_entropy, EventKey, EventKind};
type Result<T> = std::result::Result<T, EngineError>;
pub const WIRING_SEED: u64 = 0x973;
pub const PARAMETER_COUNT: usize = GATE_LOGITS + HEAD_LOGITS;

/// Actual implementation identity, with explicit name/length framing.
pub fn implementation_digest() -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"uor-r4.addressed-attention-implementation/1");
    for (name,bytes) in [
        ("engine",include_bytes!("engine.rs").as_slice()),
        ("artifact",include_bytes!("artifact.rs").as_slice()),
        ("policy",include_bytes!("policy.rs").as_slice()),
        ("offline",include_bytes!("offline.rs").as_slice()),
        ("circuit",include_bytes!("circuit.rs").as_slice()),
        ("inputs",include_bytes!("inputs.rs").as_slice()),
        ("objects",include_bytes!("objects.rs").as_slice()),
        ("training",include_bytes!("training.rs").as_slice()),
        ("native_geometry",include_bytes!("../training.rs").as_slice()),
        ("anchors",include_bytes!("../anchors.rs").as_slice()),
        ("contract",include_bytes!("../../../../../docs/evidence/native_geometric_addressed_attention_contract_973.json").as_slice()),
    ] { h.update(&(name.len() as u64).to_le_bytes());h.update(name.as_bytes());h.update(&(bytes.len() as u64).to_le_bytes());h.update(bytes); }
    *h.finalize().as_bytes()
}

#[derive(Clone, Debug)]
pub struct Parameters {
    values: Vec<f64>,
    topology: Topology,
    seed: u64,
}
impl Parameters {
    /// Offline symmetric uniform [-.25,.25) initialization, not a fit.
    pub fn seeded(seed: u64) -> Result<Self> {
        let mut counter = seed;
        let mut values = Vec::with_capacity(PARAMETER_COUNT);
        for _ in 0..PARAMETER_COUNT {
            counter = counter.wrapping_add(0x9e3779b97f4a7c15);
            let mut x = counter;
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
            x ^= x >> 31;
            values.push(((x >> 11) as f64 / 9_007_199_254_740_992.0 - 0.5) * 0.5);
        }
        Self::from_values(seed, values)
    }
    pub fn from_values(seed: u64, values: Vec<f64>) -> Result<Self> {
        if values.len() != PARAMETER_COUNT || values.iter().any(|x| !x.is_finite()) {
            return Err(EngineError::Invalid("parameter shape or finite domain"));
        }
        Ok(Self {
            values,
            topology: Topology::seeded(WIRING_SEED),
            seed,
        })
    }
    pub fn values(&self) -> &[f64] {
        &self.values
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn digest(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"uor-r4.addressed-attention-parameters/1");
        h.update(&self.seed.to_le_bytes());
        for v in &self.values {
            h.update(&v.to_bits().to_le_bytes());
        }
        *h.finalize().as_bytes()
    }
    pub fn compile(&self) -> Result<PrimitiveExport> {
        Ok(PrimitiveExport {
            circuit: CompiledCircuit::compile(self.topology.clone(), &self.values[..GATE_LOGITS])?,
            heads: CompiledHeads::compile(&self.values[GATE_LOGITS..])?,
        })
    }
}

/// Shared parameter rows, with absolute offsets into the 665216-vector.
pub fn head_range(head: Head, context: u16) -> Result<(usize, usize)> {
    if context >= 512 {
        return Err(EngineError::Invalid("head context"));
    }
    let c = usize::from(context);
    let extent = GATE_LOGITS + 8 * 512 * 120;
    let control = extent + 512 * 64;
    let emission = control + 512 * 7;
    match head {
        Head::Root(h) if h < 8 => Ok((GATE_LOGITS + usize::from(h) * 512 * 120 + c * 120, 120)),
        Head::Extent => Ok((extent + c * 64, 64)),
        Head::Control => Ok((control + c * 7, 7)),
        Head::Emission => Ok((emission + c * 257, 257)),
        Head::Null(h) if h < 2 => Ok((emission + 512 * 257 + usize::from(h) * 120, 120)),
        _ => Err(EngineError::Invalid("head kind")),
    }
}
fn validate_choice(choice: usize, legal: &[bool], expected: usize) -> Result<usize> {
    if legal.len() != expected || choice >= legal.len() || !legal[choice] {
        Err(EngineError::Invalid("head legal choice"))
    } else {
        Ok(choice)
    }
}

pub struct CompiledPolicy<'a> {
    pub compiled: &'a PrimitiveExport,
}
impl Policy for CompiledPolicy<'_> {
    fn context(&mut self, _phase: Phase, input: &[bool; INPUT_BITS]) -> Result<u16> {
        Ok(self.compiled.circuit.evaluate(input)?)
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> Result<usize> {
        let (_, n) = head_range(head, context)?;
        let choice = match head {
            Head::Root(h) => usize::from(self.compiled.heads.roots(context)?[usize::from(h)]),
            Head::Null(h) => usize::from(self.compiled.heads.null_roots()[usize::from(h)]),
            Head::Emission => usize::from(self.compiled.heads.emission(context)?),
            Head::Extent => usize::from(
                self.compiled.heads.extent(
                    context,
                    legal
                        .try_into()
                        .map_err(|_| EngineError::Invalid("extent mask"))?,
                )? - 1,
            ),
            Head::Control => usize::from(
                self.compiled.heads.control(
                    context,
                    legal
                        .try_into()
                        .map_err(|_| EngineError::Invalid("control mask"))?,
                )?,
            ),
        };
        validate_choice(choice, legal, n)
    }
}

/// Host deterministic interpreter of frozen parameters. It does not read any
/// compiled truth table, mode or preference order; it shares only fixed wiring.
pub struct InterpretedPolicy<'a> {
    pub parameters: &'a Parameters,
}
impl Policy for InterpretedPolicy<'_> {
    fn context(&mut self, _phase: Phase, input: &[bool; INPUT_BITS]) -> Result<u16> {
        let mut previous = input.to_vec();
        let mut start = 0;
        for width in super::circuit::WIDTHS {
            let mut next = Vec::with_capacity(width);
            for gate in start..start + width {
                let pins = self.parameters.topology.wires()[gate];
                let mut row = 0;
                for (bit, pin) in pins.into_iter().enumerate() {
                    if previous[usize::from(pin)] {
                        row |= 1 << bit;
                    }
                }
                next.push(self.parameters.values[gate * 16 + row] > 0.0);
            }
            previous = next;
            start += width;
        }
        Ok(previous
            .iter()
            .rev()
            .fold(0, |a, &b| (a << 1) | u16::from(b)))
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> Result<usize> {
        let (offset, n) = head_range(head, context)?;
        if legal.len() != n {
            return Err(EngineError::Invalid("interpreter mask"));
        }
        let mut best = None;
        for (i, &allowed) in legal.iter().enumerate() {
            if allowed
                && best.is_none_or(|b| {
                    self.parameters.values[offset + i] > self.parameters.values[offset + b]
                })
            {
                best = Some(i);
            }
        }
        best.ok_or(EngineError::Invalid("empty legal interpreter support"))
    }
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct SampleCounts {
    pub circuit_calls: u64,
    pub gate_events: u64,
    pub head_events: u64,
    pub scored_positions: u64,
}
/// Explicit offline path. Target is read exclusively in the EMIT choice hook,
/// after context has been fixed and before its target-independent random draw.
pub struct SampledPolicy<'a> {
    parameters: &'a Parameters,
    key: EventKey,
    phase: Phase,
    target: Option<(usize, usize)>,
    score: Vec<f64>,
    direct: Vec<f64>,
    loss: f64,
    counts: SampleCounts,
}
impl<'a> SampledPolicy<'a> {
    pub fn new(parameters: &'a Parameters, seed: u64, particle: u64, document: u64) -> Self {
        Self {
            parameters,
            key: EventKey {
                seed,
                batch: 0,
                particle,
                document,
                event: 0,
                phase: 0,
                kind: EventKind::Gate,
                slot: 0,
            },
            phase: Phase::QueryA,
            target: None,
            score: vec![0.0; PARAMETER_COUNT],
            direct: vec![0.0; PARAMETER_COUNT],
            loss: 0.0,
            counts: SampleCounts::default(),
        }
    }
    pub fn set_target(&mut self, target: usize, normalizer: usize) -> Result<()> {
        if target > 256 || normalizer == 0 || normalizer > 512 || self.target.is_some() {
            return Err(EngineError::Invalid(
                "target/normalizer or unconsumed target",
            ));
        }
        self.target = Some((target, normalizer));
        Ok(())
    }
    pub fn counts(&self) -> SampleCounts {
        self.counts
    }
    pub fn mean_loss(&self) -> f64 {
        self.loss
    }
    pub fn score(&self) -> &[f64] {
        &self.score
    }
    pub fn direct(&self) -> &[f64] {
        &self.direct
    }
    pub fn target_pending(&self) -> bool {
        self.target.is_some()
    }
    pub fn scratch_bytes(&self) -> usize {
        (self.score.capacity() + self.direct.capacity()) * std::mem::size_of::<f64>()
    }
}
impl Policy for SampledPolicy<'_> {
    fn context(&mut self, phase: Phase, input: &[bool; INPUT_BITS]) -> Result<u16> {
        self.phase = phase;
        self.counts.circuit_calls += 1;
        let parameters = self.parameters;
        let score = &mut self.score;
        let key = &mut self.key;
        let counts = &mut self.counts;
        let result = parameters.topology.evaluate_with(input, |row| {
            key.phase = phase as u8;
            key.kind = EventKind::Gate;
            key.slot = (row >> 4) as u32;
            let draw = bernoulli(parameters.values[row], *key)
                .map_err(|_| super::circuit::CircuitError::NonFinite)?;
            key.event = key
                .event
                .checked_add(1)
                .ok_or(super::circuit::CircuitError::Domain)?;
            score[row] += draw.score;
            counts.gate_events += 1;
            Ok(draw.outcome)
        })?;
        Ok(result)
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> Result<usize> {
        let (offset, n) = head_range(head, context)?;
        if legal.len() != n {
            return Err(EngineError::Invalid("sample mask"));
        }
        let logits = &self.parameters.values[offset..offset + n];
        if matches!(head, Head::Emission) {
            if let Some((target, normalizer)) = self.target.take() {
                let ce = cross_entropy(logits, legal, target, normalizer)
                    .map_err(|_| EngineError::Invalid("conditional CE"))?;
                self.loss += ce.loss;
                for (i, v) in ce.direct.into_iter().enumerate() {
                    self.direct[offset + i] += v;
                }
                self.counts.scored_positions += 1;
            }
        }
        self.key.phase = self.phase as u8;
        self.key.kind = EventKind::Head;
        self.key.slot = offset as u32;
        let draw = categorical(logits, legal, self.key)
            .map_err(|_| EngineError::Invalid("categorical sample"))?;
        self.key.event = self
            .key
            .event
            .checked_add(1)
            .ok_or(EngineError::Invalid("event overflow"))?;
        self.counts.head_events += 1;
        for (i, v) in draw.score.into_iter().enumerate() {
            self.score[offset + i] += v;
        }
        Ok(draw.outcome)
    }
}
