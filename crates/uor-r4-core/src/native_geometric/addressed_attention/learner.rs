//! Offline four-particle causal credit, plain SGD, and parameter checkpoints.
//! Checkpoints are training state, not promoted or serving model artifacts.
use super::artifact::BoundGeometry;
use super::engine::{EngineError, RuntimeSession};
use super::objects::Symbol;
use super::policy::{Parameters, SampleCounts, SampledPolicy, PARAMETER_COUNT};
use super::training::SerialLoo;
use std::time::Instant;
type Result<T> = std::result::Result<T, EngineError>;
pub const PARTICLES: u64 = 4;
pub const MAX_POSITIONS: usize = 64;
pub const MAX_UPDATES: u64 = 64;
const MAGIC: &[u8; 8] = b"UORAAL01";
pub const CHECKPOINT_BYTES: usize = 8 + 2 + 32 * 4 + 8 * 3 + 4 + 32 + PARAMETER_COUNT * 8 + 32;

#[derive(Debug)]
pub struct Batch {
    pub gradient: Vec<f64>,
    pub mean_ce: f64,
    /// Aggregate across all four independent particle trajectories.
    pub counts: SampleCounts,
    pub elapsed_us: u128,
}

/// Every document symbol, including prompt and terminal EOS, contributes CE.
/// Parameters stay fixed through all four trajectories and LOO reduction.
/// `document_id` is the invocation identity: use the frozen update index when
/// revisiting a corpus row to avoid replaying the same counter RNG stream.
pub fn batch(
    parameters: &Parameters,
    geometry: &BoundGeometry,
    document: &[u16],
    event_seed: u64,
    document_id: u64,
) -> Result<Batch> {
    if document.is_empty()
        || document.len() > MAX_POSITIONS
        || document.last() != Some(&256)
        || document[..document.len() - 1].iter().any(|&s| s > 255)
    {
        return Err(EngineError::Invalid(
            "learner document must contain bytes and one terminal EOS within64 positions",
        ));
    }
    let start = Instant::now();
    let before = parameters.digest();
    let mut accumulator = SerialLoo::new(parameters.values(), PARTICLES as usize)
        .map_err(|_| EngineError::Invalid("learner LOO setup"))?;
    let mut counts = SampleCounts::default();
    let mut mean_ce = 0.0;
    for particle in 0..PARTICLES {
        let mut runtime = RuntimeSession::new(before, geometry, particle + 1)?;
        let mut policy = SampledPolicy::new(parameters, event_seed, particle, document_id);
        for &target in document {
            policy.set_target(usize::from(target), document.len())?;
            let offered = runtime.predict(geometry, &mut policy)?;
            if policy.target_pending() {
                return Err(EngineError::Invalid("learner target not consumed atEMIT"));
            }
            let actual = if target == 256 {
                Symbol::Eos
            } else {
                Symbol::Byte(target as u8)
            };
            let completed = runtime.observe(actual, offered.offer.id, geometry, &mut policy)?;
            if completed.circuit_calls() > 8
                || completed.candidate_scores > 530
                || completed.scalar_decodes > 2
                || completed.selected_operations > 1
            {
                return Err(EngineError::Invalid("learner complete path bound"));
            }
        }
        let current = policy.counts();
        if current.scored_positions != document.len() as u64
            || current.gate_events != current.circuit_calls * super::circuit::GATES as u64
            || current.circuit_calls != runtime.work().circuit_calls
            || !policy.mean_loss().is_finite()
        {
            return Err(EngineError::Invalid("learner score/event accounting"));
        }
        accumulator
            .add_particle(
                parameters.values(),
                policy.mean_loss(),
                policy.score(),
                policy.direct(),
            )
            .map_err(|_| EngineError::Invalid("learner LOO particle"))?;
        mean_ce += policy.mean_loss() / PARTICLES as f64;
        counts.circuit_calls += current.circuit_calls;
        counts.gate_events += current.gate_events;
        counts.head_events += current.head_events;
        counts.scored_positions += current.scored_positions;
    }
    let gradient = accumulator
        .finish(parameters.values())
        .map_err(|_| EngineError::Invalid("learner LOO finish"))?;
    if before != parameters.digest()
        || !mean_ce.is_finite()
        || gradient.iter().any(|v| !v.is_finite())
    {
        return Err(EngineError::Invalid(
            "learner frozen parameters or finite gradient",
        ));
    }
    Ok(Batch {
        gradient,
        mean_ce,
        counts,
        elapsed_us: start.elapsed().as_micros(),
    })
}

/// Unclipped plain SGD: theta' = theta - rate * gradient. This validates the
/// complete result before returning a new value; the parent stays unchanged.
pub fn sgd(parameters: &Parameters, gradient: &[f64], rate: f64) -> Result<Parameters> {
    if gradient.len() != PARAMETER_COUNT
        || !rate.is_finite()
        || rate <= 0.0
        || gradient.iter().any(|g| !g.is_finite())
    {
        return Err(EngineError::Invalid("SGD shape, rate or finite gradient"));
    }
    let updated = parameters
        .values()
        .iter()
        .zip(gradient)
        .map(|(&theta, &g)| theta - rate * g)
        .collect();
    Parameters::from_values(parameters.seed(), updated)
}

/// The old complete-forward implementation identity remains unchanged. This
/// separately named learner source is additionally bound to each checkpoint.
pub fn implementation_digest() -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"uor-r4.addressed-attention-learner/1");
    h.update(&super::policy::implementation_digest());
    let source = include_bytes!("learner.rs");
    h.update(&(source.len() as u64).to_le_bytes());
    h.update(source);
    *h.finalize().as_bytes()
}

#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub parameters: Parameters,
    pub data_digest: [u8; 32],
    pub config_digest: [u8; 32],
    pub event_seed: u64,
    pub completed_updates: u64,
}
impl Checkpoint {
    pub fn new(
        parameters: Parameters,
        data_digest: [u8; 32],
        config_digest: [u8; 32],
        event_seed: u64,
        completed_updates: u64,
    ) -> Result<Self> {
        let result = Self {
            parameters,
            data_digest,
            config_digest,
            event_seed,
            completed_updates,
        };
        result.validate()?;
        Ok(result)
    }
    fn validate(&self) -> Result<()> {
        if self.data_digest == [0; 32]
            || self.config_digest == [0; 32]
            || self.completed_updates > MAX_UPDATES
        {
            return Err(EngineError::Invalid(
                "learner checkpoint provenance/update bound",
            ));
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut out = Vec::with_capacity(CHECKPOINT_BYTES);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&1u16.to_le_bytes());
        for value in [
            super::policy::implementation_digest(),
            implementation_digest(),
            self.data_digest,
            self.config_digest,
        ] {
            out.extend_from_slice(&value);
        }
        for value in [
            self.parameters.seed(),
            self.event_seed,
            self.completed_updates,
        ] {
            out.extend_from_slice(&value.to_le_bytes());
        }
        out.extend_from_slice(&(PARAMETER_COUNT as u32).to_le_bytes());
        out.extend_from_slice(&self.parameters.digest());
        for value in self.parameters.values() {
            out.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        let checksum = wire_digest(&out);
        out.extend_from_slice(&checksum);
        if out.len() != CHECKPOINT_BYTES {
            return Err(EngineError::Invalid("learner checkpoint encoding length"));
        }
        Ok(out)
    }
    /// Exact caller data/config/event-seed matching prevents adopting a valid
    /// checkpoint from a different experiment. Step is returned for scheduling.
    pub fn decode(
        bytes: &[u8],
        expected_data: [u8; 32],
        expected_config: [u8; 32],
        expected_event_seed: u64,
    ) -> Result<Self> {
        if bytes.len() != CHECKPOINT_BYTES {
            return Err(EngineError::Invalid("learner checkpoint fixed length"));
        }
        let body_len = bytes.len() - 32;
        if wire_digest(&bytes[..body_len]) != bytes[body_len..] {
            return Err(EngineError::Invalid("learner checkpoint checksum"));
        }
        let mut reader = Reader(&bytes[..body_len]);
        if reader.take(8)? != MAGIC || u16::from_le_bytes(reader.array()?) != 1 {
            return Err(EngineError::Invalid("learner checkpoint version"));
        }
        if reader.array::<32>()? != super::policy::implementation_digest()
            || reader.array::<32>()? != implementation_digest()
        {
            return Err(EngineError::Invalid("learner checkpoint source identity"));
        }
        let data_digest = reader.array()?;
        let config_digest = reader.array()?;
        let parameter_seed = u64::from_le_bytes(reader.array()?);
        let event_seed = u64::from_le_bytes(reader.array()?);
        let completed_updates = u64::from_le_bytes(reader.array()?);
        if data_digest != expected_data
            || config_digest != expected_config
            || event_seed != expected_event_seed
            || u32::from_le_bytes(reader.array()?) as usize != PARAMETER_COUNT
        {
            return Err(EngineError::Invalid(
                "learner checkpoint experiment identity",
            ));
        }
        let expected_parameters = reader.array::<32>()?;
        let mut values = Vec::with_capacity(PARAMETER_COUNT);
        for _ in 0..PARAMETER_COUNT {
            values.push(f64::from_bits(u64::from_le_bytes(reader.array()?)));
        }
        if !reader.0.is_empty() {
            return Err(EngineError::Invalid("learner checkpoint trailing data"));
        }
        let parameters = Parameters::from_values(parameter_seed, values)?;
        if parameters.digest() != expected_parameters {
            return Err(EngineError::Invalid(
                "learner checkpoint parameter identity",
            ));
        }
        Self::new(
            parameters,
            data_digest,
            config_digest,
            event_seed,
            completed_updates,
        )
    }
}
fn wire_digest(body: &[u8]) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"uor-r4.addressed-attention-checkpoint/1");
    h.update(body);
    *h.finalize().as_bytes()
}
struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if n > self.0.len() {
            return Err(EngineError::Invalid("truncated learner checkpoint"));
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.take(N)?
            .try_into()
            .map_err(|_| EngineError::Invalid("learner checkpoint scalar"))
    }
}

#[cfg(test)]
#[path = "learner_tests.rs"]
mod tests;
