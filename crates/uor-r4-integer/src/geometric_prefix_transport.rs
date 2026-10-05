//! Ordered prefix compatibility from explicitly typed retained H4 latent states.
//! Source state is before the candidate token; response uses actual emitted IDs.
//! No numeric offset reward, target, semantic metadata, cursor or candidate gate.
use crate::{
    geometric_context::NativeContextState,
    geometric_context_q4::{ContextQ4Config, NativeContextQ4},
    geometric_cue_carrier::CueCarrierMetadata,
    geometric_occurrence_read::{OccurrenceBankSegment, MAX_SEQUENCE},
    geometric_potential_q4::unpack_coefficients,
    geometric_source_realizer::{
        NativeArtifactBinding, SourceRuntimeError, SourceRuntimeResult as Result,
    },
    h4_tables::{H4Code, HistoricalH4Tables, ROOT_COUNT, TRUSTED_MATHEMATICAL_SHA256},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
fn error(s: impl Into<String>) -> SourceRuntimeError {
    SourceRuntimeError::Invalid(s.into())
}
fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub const SCHEMA: &str = "uor-r4.geometric-ordered-prefix-transport/1";
pub const POLICY:&str="Source-emission-view-local-prefix-before-candidate;actual-ownprefix-only;empty-identity;retained-latent-not-observed-presence;inverse(response)*source-before;all-candidates;signed120-q4-quarter;cue-frozen;before-global-Copy-Period-Stop/1";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrefixScoreMode {
    DirectedRelative,
    SourcePrefixUnary,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrefixAngularConfig {
    pub heads: usize,
    pub lanes_per_head: usize,
    pub mode: PrefixScoreMode,
}
impl PrefixAngularConfig {
    pub fn coefficient_count(self) -> Result<usize> {
        if !(1..=2).contains(&self.heads) || !(1..=4).contains(&self.lanes_per_head) {
            return Err(error("prefix angular dimensions out of bounds"));
        }
        Ok(self.heads * self.lanes_per_head * ROOT_COUNT)
    }
}
/// Tables are split at admission. Per-lane scoring performs table reads/adds;
/// whole-path compiler opcode/allocation evidence is a separate measurement.
pub struct PrefixAngularQ4 {
    config: PrefixAngularConfig,
    packed: Vec<u8>,
    tables: Vec<Vec<[i32; ROOT_COUNT]>>,
}
impl PrefixAngularQ4 {
    pub fn new(config: PrefixAngularConfig, packed: &[u8]) -> Result<Self> {
        let q = unpack_coefficients(config.coefficient_count()?, packed)
            .map_err(|e| error(e.to_string()))?;
        let mut rows = q.chunks_exact(ROOT_COUNT);
        let mut tables = Vec::new();
        for _ in 0..config.heads {
            let mut head = Vec::new();
            for _ in 0..config.lanes_per_head {
                let row = rows
                    .next()
                    .ok_or_else(|| error("prefix table row absent"))?;
                let mut table = [0; ROOT_COUNT];
                for (dst, &q) in table.iter_mut().zip(row) {
                    *dst = i32::from(q) << 22;
                }
                head.push(table);
            }
            tables.push(head);
        }
        Ok(Self {
            config,
            packed: packed.to_vec(),
            tables,
        })
    }
    pub fn config(&self) -> PrefixAngularConfig {
        self.config
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        &self.packed
    }
    fn contribution(
        &self,
        head: usize,
        lane: usize,
        response: H4Code,
        source: H4Code,
        geometry: &HistoricalH4Tables,
    ) -> Result<(i64, u8, u8)> {
        let table = self
            .tables
            .get(head)
            .and_then(|h| h.get(lane))
            .ok_or_else(|| error("prefix head/lane absent"))?;
        let relative = geometry.relative(response, source).index();
        let bin = match self.config.mode {
            PrefixScoreMode::DirectedRelative => relative,
            PrefixScoreMode::SourcePrefixUnary => source.index(),
        };
        Ok((i64::from(table[usize::from(bin)]), bin, relative))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrefixTransportMetadata {
    pub schema: String,
    pub policy: String,
    pub parent_artifact: NativeArtifactBinding,
    pub frozen_cue: CueCarrierMetadata,
    pub context: ContextQ4Config,
    pub context_packed_sha256: String,
    pub algebra_sha256: String,
    pub potential: PrefixAngularConfig,
    pub potential_packed_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PrefixState {
    pub token_ids: Vec<u32>,
    /// Empty text has the declared group identity in each retained latent lane.
    /// These states are not observed roots and have no fabricated presence mask.
    pub states: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourcePrefixTrace {
    pub source_segment_index: usize,
    pub token_ids: Vec<u32>,
    /// Row j consumes source token IDs [0,j), never token j or a future token.
    pub states_before: Vec<Vec<u8>>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PrefixTransportCosts {
    pub extra_source_encoder_tokens: usize,
    pub extra_response_encoder_tokens: usize,
    pub extra_context_coefficient_reads: usize,
    pub extra_angular_table_reads: usize,
    pub extra_geometry_relative_operations: usize,
    pub logical_sidecar_payload_bytes: usize,
    pub compiled_table_payload_bytes: usize,
    pub packed_source_bytes: usize,
    /// Logical Vec containers used during preparation, not allocator calls or
    /// reallocation counts. Excludes metadata strings and the bank/cue wrapper.
    pub preparation_vec_containers: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PrefixTransportTrace {
    pub metadata: PrefixTransportMetadata,
    pub response: PrefixState,
    pub sources: Vec<SourcePrefixTrace>,
    pub candidate_source_indices: Vec<usize>,
    pub candidate_offsets: Vec<usize>,
    pub angular_indices: Vec<Vec<u8>>,
    pub relative_roots: Vec<Vec<u8>>,
    pub copy_q24: Vec<Vec<i64>>,
    pub costs: PrefixTransportCosts,
}
pub struct NativePrefixTransport<'a> {
    context: &'a NativeContextQ4,
    geometry: &'a HistoricalH4Tables,
    potential: PrefixAngularQ4,
    metadata: PrefixTransportMetadata,
}
impl<'a> NativePrefixTransport<'a> {
    pub fn compile(
        parent: NativeArtifactBinding,
        context: &'a NativeContextQ4,
        geometry: &'a HistoricalH4Tables,
        frozen_cue: CueCarrierMetadata,
        potential: PrefixAngularQ4,
    ) -> Result<Self> {
        parent.identity.validate()?;
        let c = context.config();
        let p = potential.config();
        let context_sha = sha(context.packed_coefficients());
        let algebra = TRUSTED_MATHEMATICAL_SHA256
            .ok_or_else(|| error("prefix algebra trust anchor absent"))?;
        if parent != frozen_cue.parent_artifact
            || frozen_cue.context != c
            || frozen_cue.context_packed_sha256 != context_sha
            || frozen_cue.algebra_sha256 != algebra
            || p.heads != c.heads
            || p.lanes_per_head != c.lanes_per_head
        {
            return Err(error(
                "prefix parent/frozen cue/encoder/geometry dimensions differ",
            ));
        }
        let metadata = PrefixTransportMetadata {
            schema: SCHEMA.into(),
            policy: POLICY.into(),
            parent_artifact: parent,
            frozen_cue,
            context: c,
            context_packed_sha256: context_sha,
            algebra_sha256: algebra.into(),
            potential: p,
            potential_packed_sha256: sha(potential.packed_coefficients()),
        };
        Ok(Self {
            context,
            geometry,
            potential,
            metadata,
        })
    }
    pub fn metadata(&self) -> &PrefixTransportMetadata {
        &self.metadata
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        self.potential.packed_coefficients()
    }
    pub(crate) fn validate_execution(
        &self,
        parent: &NativeArtifactBinding,
        context: &NativeContextQ4,
        geometry: &HistoricalH4Tables,
        cue: &CueCarrierMetadata,
    ) -> Result<()> {
        if parent != &self.metadata.parent_artifact
            || cue != &self.metadata.frozen_cue
            || !std::ptr::eq(context, self.context)
            || !std::ptr::eq(geometry, self.geometry)
        {
            return Err(error("foreign prefix parent/cue/encoder/geometry"));
        }
        Ok(())
    }
    pub(crate) fn prepare(
        &self,
        segments: &[OccurrenceBankSegment<'_>],
        response: &[u32],
        candidate_segments: &[usize],
    ) -> Result<PrefixTransportTrace> {
        if segments.len() > MAX_SEQUENCE
            || response.len() > MAX_SEQUENCE
            || candidate_segments.len() > MAX_SEQUENCE
        {
            return Err(error("prefix preparation cap exceeds128"));
        }
        let c = self.context.config();
        let width = c.heads * c.lanes_per_head;
        let mut costs = PrefixTransportCosts {
            compiled_table_payload_bytes: self.potential.config().coefficient_count()? * 4,
            packed_source_bytes: self.packed_coefficients().len(),
            ..Default::default()
        };
        let mut state =
            NativeContextState::new(c.heads, c.lanes_per_head).map_err(|e| error(e.to_string()))?;
        for &token in response {
            let step = state
                .step(token as usize, self.context.native(), self.geometry)
                .map_err(|e| error(e.to_string()))?;
            costs.extra_context_coefficient_reads += step.coefficient_reads;
            costs.extra_response_encoder_tokens += 1;
        }
        let response = PrefixState {
            token_ids: response.to_vec(),
            states: state.states().iter().map(|x| x.index()).collect(),
        };
        let response_roots = response
            .states
            .iter()
            .copied()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| error(e.to_string()))?;
        let mut sources = Vec::new();
        let mut expected_segments = Vec::new();
        let mut candidate_source_indices = Vec::new();
        let mut candidate_offsets = Vec::new();
        for (segment_index, segment) in segments.iter().enumerate() {
            if let OccurrenceBankSegment::Source { frame, .. } = segment {
                let ids = frame.token_ids;
                if ids.len() > MAX_SEQUENCE {
                    return Err(error("prefix source cap exceeds128"));
                }
                let source_index = sources.len();
                let mut before = Vec::with_capacity(ids.len());
                state.reset();
                for (offset, &token) in ids.iter().enumerate() {
                    before.push(state.states().iter().map(|x| x.index()).collect::<Vec<_>>());
                    expected_segments.push(segment_index);
                    candidate_source_indices.push(source_index);
                    candidate_offsets.push(offset);
                    // The last source token has no following candidate state; no
                    // unnecessary transition/observation is performed for it.
                    if offset + 1 < ids.len() {
                        let step = state
                            .step(token as usize, self.context.native(), self.geometry)
                            .map_err(|e| error(e.to_string()))?;
                        costs.extra_context_coefficient_reads += step.coefficient_reads;
                        costs.extra_source_encoder_tokens += 1;
                    }
                }
                sources.push(SourcePrefixTrace {
                    source_segment_index: segment_index,
                    token_ids: ids.to_vec(),
                    states_before: before,
                });
            }
        }
        if expected_segments != candidate_segments {
            return Err(error("prefix source/candidate binding differs"));
        }
        let mut angular_indices = vec![vec![0u8; candidate_segments.len()]; width];
        let mut relative_roots = angular_indices.clone();
        let mut copy_q24 = vec![vec![0i64; candidate_segments.len()]; c.heads];
        for (candidate, (&source, &offset)) in candidate_source_indices
            .iter()
            .zip(&candidate_offsets)
            .enumerate()
        {
            let before = &sources[source].states_before[offset];
            for (h, head) in copy_q24.iter_mut().enumerate() {
                for lane in 0..c.lanes_per_head {
                    let global = h * c.lanes_per_head + lane;
                    let source =
                        H4Code::try_from(before[global]).map_err(|e| error(e.to_string()))?;
                    let (score, bin, relative) = self.potential.contribution(
                        h,
                        lane,
                        response_roots[global],
                        source,
                        self.geometry,
                    )?;
                    head[candidate] = head[candidate]
                        .checked_add(score)
                        .ok_or_else(|| error("prefix score sum overflow"))?;
                    angular_indices[global][candidate] = bin;
                    relative_roots[global][candidate] = relative;
                    costs.extra_angular_table_reads += 1;
                    costs.extra_geometry_relative_operations += 1;
                }
            }
        }
        costs.logical_sidecar_payload_bytes = response.token_ids.len() * 4
            + response.states.len()
            + sources
                .iter()
                .map(|s| {
                    std::mem::size_of::<usize>()
                        + s.token_ids.len() * 4
                        + s.states_before.len() * width
                })
                .sum::<usize>()
            + candidate_segments.len()
                * (2 * std::mem::size_of::<usize>() + 2 * width + c.heads * 8);
        costs.preparation_vec_containers = 2
            + 1
            + 4
            + sources.len() * 2
            + candidate_segments.len()
            + 2 * (1 + width)
            + (1 + c.heads);
        Ok(PrefixTransportTrace {
            metadata: self.metadata.clone(),
            response,
            sources,
            candidate_source_indices,
            candidate_offsets,
            angular_indices,
            relative_roots,
            copy_q24,
            costs,
        })
    }
}
