//! Experimental shared geometric byte learner. No retained-model dispatch uses it.
//!
//! Host construction and training live separately from the discrete kernel.
//! State parameters are canonical H4 roots; calibrated emission also learns
//! bounded integer thresholds and intersection/union choices. Composition reads
//! the exact finite group table, never a tabulated linear contraction. Four
//! independent state slots are not four independent Galois companions.
mod block_calibration;
mod calibration;
use block_calibration::BlockCalibrationConfig;
pub use block_calibration::BlockCalibrationReport;
mod training;
use super::Geometry;
pub use calibration::CalibrationReport;
use serde::{Deserialize, Serialize};
pub use training::{FitConfig, FitReport, Metrics};

pub const EOS: u16 = 256;
pub const CONTEXT: usize = 32;
const LANES: usize = 4;
const ROOTS: usize = 120;
const SCHEMA: &str = "uor-r4.shared-geometric-core/1";
const CALIBRATED_SCHEMA: &str = "uor-r4.shared-geometric-core/2";
// The preserved v1 source/binary binding. Its runtime path remains supported;
// this is not permission to load arbitrary implementation identities.
const LEGACY_IMPLEMENTATION: &str =
    "blake3:b6d1b6fb5c6af3be5fe31dc44dfdf762aad084e3100b5484e374c6a99a6b756d";
const CALIBRATION_V1_IMPLEMENTATION: &str =
    "blake3:e719c2d0b3372e8e39495cc2990fbc8afd37787fa8c6f31a0d7f09818891cec1";
const EMBED: usize = 0;
const TRANSITION: usize = EMBED + LANES * 256;
const QUERY: usize = TRANSITION + LANES * ROOTS;
const KEY: usize = QUERY + ROOTS;
const READ: usize = KEY + ROOTS;
const PHASE: usize = READ + LANES * ROOTS;
const OUTPUT: usize = PHASE + LANES * 16;
const OUTPUT_MIX: usize = OUTPUT + 512;
const NULL: usize = OUTPUT_MIX + 9;
const PARAMETERS: usize = NULL + 1;

#[derive(Debug)]
pub enum CoreError {
    InvalidArtifact(&'static str),
    InvalidSnapshot(&'static str),
    InvalidInput(&'static str),
    Host(String),
}
impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CoreError {}
type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CapBranch {
    landmarks: [u16; 2],
    thresholds: [i16; 2],
    union: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CalibratedEmission {
    branches: Vec<CapBranch>,
    parent: String,
    calibration_data: Option<String>,
    calibration_passes: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    block_config: Option<BlockCalibrationConfig>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    schema: String,
    implementation: String,
    geometry: Geometry,
    angular: Vec<i16>,
    parameters: Vec<u16>,
    seed: u64,
    training_digest: Option<String>,
    fit_config: Option<FitConfig>,
    training_parent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    calibrated: Option<CalibratedEmission>,
}

/// A separately versioned experimental artifact in the native model module.
/// Loading it does not load or call the old additive predictor.
#[derive(Clone)]
pub struct SharedCore {
    artifact: Artifact,
    cid: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intervention {
    #[default]
    Full,
    ContextDisabled,
    StateDisabled,
    ZetaDisabled,
    /// Replace only recurrent finite products by their right operand.
    TransportDisabled,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Work {
    pub products: u64,
    pub table_reads: u64,
    pub candidates: u64,
    pub output_decisions: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    key: u16,
    values: [u16; LANES],
    byte: u8,
    position: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    roots: [u16; LANES],
    phases: [u16; LANES],
    ring: [Entry; CONTEXT],
    seen: u64,
    last_source: Option<u64>,
}

/// Fixed-size causal state. Query/output prediction is observational.
pub struct CoreSession<'a> {
    model: &'a SharedCore,
    state: State,
    control: Intervention,
    work: Work,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema: String,
    artifact: String,
    control: Intervention,
    state: State,
}

impl SharedCore {
    fn implementation_digest() -> String {
        let mut hash = blake3::Hasher::new();
        hash.update(include_bytes!("shared_core.rs"));
        hash.update(include_bytes!("shared_core/training.rs"));
        hash.update(include_bytes!("shared_core/calibration.rs"));
        hash.update(include_bytes!("shared_core/block_calibration.rs"));
        hash.update(include_bytes!("training.rs"));
        hash.update(include_bytes!("anchors.rs"));
        format!("blake3:{}", hash.finalize())
    }
    pub fn artifact_cid(&self) -> &str {
        &self.cid
    }
    pub fn session(&self, control: Intervention) -> CoreSession<'_> {
        CoreSession {
            model: self,
            state: State {
                roots: [self.artifact.geometry.identity; LANES],
                phases: [0; LANES],
                ring: [Entry::default(); CONTEXT],
                seen: 0,
                last_source: None,
            },
            control,
            work: Work::default(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self.artifact).map_err(|e| CoreError::Host(e.to_string()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(CoreError::InvalidArtifact("artifact byte bound"));
        }
        let artifact: Artifact =
            serde_json::from_slice(bytes).map_err(|e| CoreError::Host(e.to_string()))?;
        let expected =
            super::training::geometry(258, CONTEXT).map_err(|e| CoreError::Host(e.to_string()))?;
        let version_valid = match &artifact.calibrated {
            None => {
                artifact.schema == SCHEMA
                    && (artifact.implementation == Self::implementation_digest()
                        || artifact.implementation == LEGACY_IMPLEMENTATION)
            }
            Some(c) => {
                artifact.schema == CALIBRATED_SCHEMA
                    && (artifact.implementation == Self::implementation_digest()
                        || artifact.implementation == CALIBRATION_V1_IMPLEMENTATION)
                    && c.branches.len() == 512
                    && c.parent.starts_with("blake3:")
                    && c.parent.len() == 71
                    && match &c.block_config {
                        None => {
                            (c.calibration_data.is_none() && c.calibration_passes == 0)
                                || (c.calibration_data.is_some() && c.calibration_passes == 3)
                        }
                        Some(config) => {
                            artifact.implementation == Self::implementation_digest()
                                && c.calibration_data.is_some()
                                && c.calibration_passes == 1
                                && config.max_seconds > 0
                                && config.max_seconds <= 120
                        }
                    }
                    && c.branches.iter().all(|b| {
                        b.landmarks.iter().all(|&r| r < 120)
                            && b.thresholds.iter().all(|&t| (-5..=5).contains(&t))
                    })
            }
        };
        if !version_valid
            || artifact.seed == 0
            || artifact.training_digest.is_some() != artifact.fit_config.is_some()
            || artifact.training_digest.is_some() != artifact.training_parent.is_some()
            || artifact.geometry != expected
            || artifact.angular != training::angular(&expected)?
            || artifact.parameters.len() != PARAMETERS
            || artifact.parameters.iter().any(|&x| usize::from(x) >= ROOTS)
        {
            return Err(CoreError::InvalidArtifact(
                "schema, geometry or parameter contract",
            ));
        }
        let mut model = Self {
            artifact,
            cid: String::new(),
        };
        model.refresh_identity()?;
        Ok(model)
    }

    fn refresh_identity(&mut self) -> Result<()> {
        self.cid = format!("blake3:{}", blake3::hash(&self.to_bytes()?));
        Ok(())
    }

    pub fn restore(&self, bytes: &[u8]) -> Result<CoreSession<'_>> {
        if bytes.len() > 32768 {
            return Err(CoreError::InvalidSnapshot("snapshot byte bound"));
        }
        let snapshot: Snapshot =
            serde_json::from_slice(bytes).map_err(|e| CoreError::Host(e.to_string()))?;
        let state = &snapshot.state;
        if snapshot.schema != self.artifact.schema
            || snapshot.artifact != self.cid
            || state.roots.iter().any(|&r| usize::from(r) >= ROOTS)
            || state.last_source.is_some_and(|p| {
                state.seen < 2 || p >= state.seen - 1 || p < state.seen.saturating_sub(33)
            })
        {
            return Err(CoreError::InvalidSnapshot("artifact, state or source"));
        }
        let lower = state.seen.saturating_sub(CONTEXT as u64);
        for position in lower..state.seen {
            let entry = state.ring[(position & 31) as usize];
            if entry.position != position
                || usize::from(entry.key) >= ROOTS
                || entry.values.iter().any(|&r| usize::from(r) >= ROOTS)
            {
                return Err(CoreError::InvalidSnapshot("resident occurrence"));
            }
        }
        Ok(CoreSession {
            model: self,
            state: snapshot.state,
            control: snapshot.control,
            work: Work::default(),
        })
    }
}

// SHARED_CORE_INTEGER_KERNEL_BEGIN
impl SharedCore {
    fn cap_score(
        &self,
        roots: [u16; LANES],
        branch: CapBranch,
        depth: usize,
        work: &mut Work,
    ) -> i16 {
        let lane = depth & 3;
        work.table_reads = work.table_reads.saturating_add(5);
        let first =
            self.relative_score(roots[lane], branch.landmarks[0], work) - branch.thresholds[0];
        let second = self.relative_score(roots[(lane + 1) & 3], branch.landmarks[1], work)
            - branch.thresholds[1];
        if branch.union {
            first.max(second)
        } else {
            first.min(second)
        }
    }

    fn product(&self, a: u16, b: u16, work: &mut Work) -> u16 {
        work.products = work.products.saturating_add(1);
        work.table_reads = work.table_reads.saturating_add(2);
        let base = self.artifact.geometry.row_bases[usize::from(a)];
        self.artifact.geometry.products[base + usize::from(b)]
    }
    fn parameter(&self, at: usize, work: &mut Work) -> u16 {
        work.table_reads = work.table_reads.saturating_add(1);
        self.artifact.parameters[at]
    }
    fn relative_score(&self, query: u16, key: u16, work: &mut Work) -> i16 {
        let inverse = self.artifact.geometry.inverses[usize::from(key)];
        work.table_reads = work.table_reads.saturating_add(2);
        let relative = self.product(query, inverse, work);
        self.artifact.angular[usize::from(relative)]
    }
}

impl CoreSession<'_> {
    pub fn work(&self) -> Work {
        self.work
    }
    pub fn state_roots(&self) -> [u16; LANES] {
        self.state.roots
    }
    pub fn last_source(&self) -> Option<u64> {
        self.state.last_source
    }
    pub fn seen(&self) -> u64 {
        self.state.seen
    }

    /// Commit one exact byte. No future byte or target is accessible here.
    pub fn observe(&mut self, byte: u8) -> Result<()> {
        if self.state.seen == u64::MAX {
            return Err(CoreError::InvalidInput("position exhausted"));
        }
        let m = self.model;
        let identity = m.artifact.geometry.identity;
        let mut injected = [identity; LANES];
        for (lane, target) in injected.iter_mut().enumerate() {
            let previous = if self.control == Intervention::StateDisabled {
                identity
            } else {
                self.state.roots[lane]
            };
            let code = m.parameter(EMBED + (lane << 8) + usize::from(byte), &mut self.work);
            *target = if self.control == Intervention::TransportDisabled {
                code
            } else {
                m.product(previous, code, &mut self.work)
            };
            // Byte b uses canonical token address b+2, leaving BOS/EOS distinct.
            self.state.phases[lane] = self.state.phases[lane]
                .wrapping_add(m.artifact.geometry.tokens[usize::from(byte) + 2].phases[lane]);
            self.work.table_reads = self.work.table_reads.saturating_add(1);
            if self.control != Intervention::ZetaDisabled {
                let phase = m.parameter(
                    PHASE + (lane << 4) + usize::from(self.state.phases[lane] >> 12),
                    &mut self.work,
                );
                *target = m.product(*target, phase, &mut self.work);
            }
        }
        let query_code = m.parameter(QUERY + usize::from(injected[1]), &mut self.work);
        let query = m.product(injected[0], query_code, &mut self.work);
        let null = m.parameter(NULL, &mut self.work);
        let mut score = m.relative_score(query, null, &mut self.work);
        let mut selected = None;
        if self.control != Intervention::ContextDisabled {
            let lower = self.state.seen.saturating_sub(CONTEXT as u64);
            for position in lower..self.state.seen {
                let entry = self.state.ring[(position & 31) as usize];
                self.work.candidates = self.work.candidates.saturating_add(1);
                let candidate = m.relative_score(query, entry.key, &mut self.work);
                if candidate > score {
                    score = candidate;
                    selected = Some(entry);
                }
            }
        }
        self.state.last_source = selected.map(|entry| entry.position);
        for lane in 0..LANES {
            let inverse = m.artifact.geometry.inverses[usize::from(injected[(lane + 1) & 3])];
            self.work.table_reads = self.work.table_reads.saturating_add(1);
            let relative = m.product(injected[lane], inverse, &mut self.work);
            let lane_base = m.artifact.geometry.row_bases[lane];
            self.work.table_reads = self.work.table_reads.saturating_add(1);
            let transition = m.parameter(
                TRANSITION + lane_base + usize::from(relative),
                &mut self.work,
            );
            let mut next = m.product(injected[lane], transition, &mut self.work);
            if let Some(entry) = selected {
                let read = m.parameter(
                    READ + lane_base + usize::from(entry.values[lane]),
                    &mut self.work,
                );
                next = m.product(next, read, &mut self.work);
            }
            self.state.roots[lane] = next;
        }
        let key = m.parameter(KEY + usize::from(self.state.roots[0]), &mut self.work);
        self.state.ring[(self.state.seen & 31) as usize] = Entry {
            key,
            values: self.state.roots,
            byte,
            position: self.state.seen,
        };
        self.state.seen += 1;
        Ok(())
    }

    fn branch_score(&mut self, node: usize, depth: usize) -> i16 {
        let m = self.model;
        if let Some(c) = &m.artifact.calibrated {
            self.work.output_decisions = self.work.output_decisions.saturating_add(1);
            return m.cap_score(self.state.roots, c.branches[node], depth, &mut self.work);
        }
        let lane = depth & 3;
        let mix = m.parameter(OUTPUT_MIX + depth, &mut self.work);
        let code = m.product(self.state.roots[lane], mix, &mut self.work);
        let code = m.product(code, self.state.roots[(lane + 1) & 3], &mut self.work);
        let landmark = m.parameter(OUTPUT + node, &mut self.work);
        self.work.output_decisions = self.work.output_decisions.saturating_add(1);
        m.relative_score(code, landmark, &mut self.work)
    }

    /// A hard walk over the 257 byte/EOS leaves. No vocabulary score sum,
    /// softmax, dot product, allocation, or baseline predictor is called.
    pub fn predict(&mut self) -> u16 {
        let (mut lo, mut hi, mut node, mut depth) = (0_u16, EOS + 1, 0, 0);
        while hi - lo > 1 {
            let mid = (lo + hi) >> 1;
            let right = self.branch_score(node, depth) > 0;
            if right {
                lo = mid;
            } else {
                hi = mid;
            }
            node = (node << 1) + 1 + usize::from(right);
            depth += 1;
        }
        lo
    }
}
// SHARED_CORE_INTEGER_KERNEL_END

impl CoreSession<'_> {
    pub fn checkpoint(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&Snapshot {
            schema: self.model.artifact.schema.clone(),
            artifact: self.model.cid.clone(),
            control: self.control,
            state: self.state.clone(),
        })
        .map_err(|e| CoreError::Host(e.to_string()))
    }
}

#[cfg(test)]
mod tests;
