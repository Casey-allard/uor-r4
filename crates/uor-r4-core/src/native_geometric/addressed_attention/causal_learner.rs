//! Separately source-bound causal-SGD training checkpoints.
//! This format never relabels the preserved full-trajectory learner checkpoint
//! or initialized serving artifact. It records parameters, not qualification.
use super::engine::EngineError;
use super::policy::{Parameters, PARAMETER_COUNT};
type Result<T> = std::result::Result<T, EngineError>;
pub const MAX_UPDATES: u64 = 64;
const MAGIC: &[u8; 8] = b"UORACL01";
pub const CHECKPOINT_BYTES: usize = 8 + 2 + 32 * 4 + 8 * 3 + 4 + 32 + PARAMETER_COUNT * 8 + 32;

/// Old SGD and policy remain byte-for-byte bound by the old learner identity.
/// This new identity additionally names the causal estimator and this codec.
pub fn implementation_digest() -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"uor-r4.addressed-attention-causal-learner/1");
    h.update(&super::learner::implementation_digest());
    for (name, source) in [
        (
            "causal_credit.rs",
            include_bytes!("causal_credit.rs").as_slice(),
        ),
        (
            "causal_learner.rs",
            include_bytes!("causal_learner.rs").as_slice(),
        ),
    ] {
        h.update(&(name.len() as u64).to_le_bytes());
        h.update(name.as_bytes());
        h.update(&(source.len() as u64).to_le_bytes());
        h.update(source);
    }
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
                "causal checkpoint provenance/update bound",
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
            super::learner::implementation_digest(),
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
            return Err(EngineError::Invalid("causal checkpoint encoding length"));
        }
        Ok(out)
    }
    /// Exact experiment bindings are mandatory. Returned step count lets the
    /// caller enforce the scheduled endpoint without resetting cumulative work.
    pub fn decode(
        bytes: &[u8],
        expected_data: [u8; 32],
        expected_config: [u8; 32],
        expected_event_seed: u64,
    ) -> Result<Self> {
        if bytes.len() != CHECKPOINT_BYTES {
            return Err(EngineError::Invalid("causal checkpoint fixed length"));
        }
        let body_len = bytes.len() - 32;
        if wire_digest(&bytes[..body_len]) != bytes[body_len..] {
            return Err(EngineError::Invalid("causal checkpoint checksum"));
        }
        let mut reader = Reader(&bytes[..body_len]);
        if reader.take(8)? != MAGIC || u16::from_le_bytes(reader.array()?) != 1 {
            return Err(EngineError::Invalid("causal checkpoint version"));
        }
        if reader.array::<32>()? != super::learner::implementation_digest()
            || reader.array::<32>()? != implementation_digest()
        {
            return Err(EngineError::Invalid("causal checkpoint source identity"));
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
                "causal checkpoint experiment identity",
            ));
        }
        let expected_parameters = reader.array::<32>()?;
        let mut values = Vec::with_capacity(PARAMETER_COUNT);
        for _ in 0..PARAMETER_COUNT {
            values.push(f64::from_bits(u64::from_le_bytes(reader.array()?)));
        }
        if !reader.0.is_empty() {
            return Err(EngineError::Invalid("causal checkpoint trailing data"));
        }
        let parameters = Parameters::from_values(parameter_seed, values)?;
        if parameters.digest() != expected_parameters {
            return Err(EngineError::Invalid("causal checkpoint parameter identity"));
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
    h.update(b"uor-r4.addressed-attention-causal-checkpoint/1");
    h.update(body);
    *h.finalize().as_bytes()
}
struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if n > self.0.len() {
            return Err(EngineError::Invalid("truncated causal checkpoint"));
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.take(N)?
            .try_into()
            .map_err(|_| EngineError::Invalid("causal checkpoint scalar"))
    }
}

#[cfg(test)]
#[path = "causal_learner_tests.rs"]
mod tests;
