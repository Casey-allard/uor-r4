//! Opt-in textual cue/query carriers; original bank replay and keys stay intact.
//! Admission expands packed q4 angular tables. Numerical scoring uses exact
//! directed H4 relation lookup and integer addition. No semantic metadata or
//! target is consumed.
use crate::{
    geometric_context::NativeContextState,
    geometric_context_q4::{ContextQ4Config, NativeContextQ4},
    geometric_occurrence_read::{OccurrenceBankSegment, MAX_SEQUENCE},
    geometric_potential::AddressLane,
    geometric_potential_q4::{compile_ordered_pair_q4, unpack_coefficients},
    geometric_source_realizer::{
        NativeArtifactBinding, ObservedCode, SourceRuntimeError, SourceRuntimeResult as Result,
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
pub const SCHEMA: &str = "uor-r4.geometric-text-cue-carrier/1";
pub const POLICY: &str = "local-Context-text-encoder-from-identity;Source-immediate-predecessor-Context-only;empty-or-no-Context-structurally-absent;query-only-encoder-stable-through-prefix;inverse(query)*cue;signed120-angular-q4;observed-both-present-mask;radius-retained-unused-v1;all-Copy-before-global-normalization/1";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CueScoreMode {
    DirectedRelative,
    CueUnary,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueAngularConfig {
    pub heads: usize,
    pub lanes_per_head: usize,
    pub mode: CueScoreMode,
}
impl CueAngularConfig {
    pub fn coefficient_count(self) -> Result<usize> {
        if !(1..=2).contains(&self.heads) || !(1..=4).contains(&self.lanes_per_head) {
            return Err(error("cue angular dimensions out of bounds"));
        }
        Ok(self.heads * self.lanes_per_head * ROOT_COUNT)
    }
}
/// Ordered local lanes within one head. Reversing them changes the operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueJointConfig {
    pub head: usize,
    pub left_lane: usize,
    pub right_lane: usize,
}
#[derive(Clone)]
pub struct CueJointQ4 {
    config: CueJointConfig,
    packed: Vec<u8>,
    table: Vec<i32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueJointMetadata {
    pub schema: String,
    pub policy: String,
    pub config: CueJointConfig,
    pub packed_sha256: String,
    pub table_sha256: String,
    pub basis_sha256: String,
}
impl CueJointQ4 {
    pub fn new(config: CueJointConfig, packed: Vec<u8>) -> Result<Self> {
        if config.head >= 2
            || config.left_lane >= 4
            || config.right_lane >= 4
            || config.left_lane == config.right_lane
        {
            return Err(error("joint cue ordered lanes out of bounds"));
        }
        let table = compile_ordered_pair_q4(&packed).map_err(|e| error(e.to_string()))?;
        Ok(Self {
            config,
            packed,
            table,
        })
    }
    pub fn config(&self) -> CueJointConfig {
        self.config
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        &self.packed
    }
    pub fn metadata(&self) -> CueJointMetadata {
        let table: Vec<_> = self.table.iter().flat_map(|v| v.to_le_bytes()).collect();
        let basis: Vec<_> = crate::geometric_no_read::CANONICAL_BASIS_Q25
            .iter()
            .flatten()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        CueJointMetadata {
            schema: "uor-r4.geometric-cue-joint-q4/1".into(),
            policy: "ordered-directed-relative-roots;both-present;canonical-F32-Q25;16-row-major-quarter-nat;cross-Q24-nearest-ties-away;zero-padded128;lookup-add/1".into(),
            config: self.config, packed_sha256: sha(&self.packed),
            table_sha256: sha(&table), basis_sha256: sha(&basis),
        }
    }
    fn contribution(&self, left: Option<u8>, right: Option<u8>) -> Result<Option<i64>> {
        match (left, right) {
            (Some(a), Some(b)) if usize::from(a) < ROOT_COUNT && usize::from(b) < ROOT_COUNT => {
                Ok(Some(i64::from(
                    self.table[(usize::from(a) << 7) + usize::from(b)],
                )))
            }
            (Some(_), Some(_)) => Err(error("joint cue root out of bounds")),
            _ => Ok(None),
        }
    }
}
#[derive(Clone)]
/// Same120 signed roots perhead/lane in both arms. Admission partitions tables
/// so numerical contribution lookup does not need a runtime coefficient product.
pub struct CueAngularQ4 {
    config: CueAngularConfig,
    packed: Vec<u8>,
    tables: Vec<Vec<[i32; ROOT_COUNT]>>,
    joint: Option<CueJointQ4>,
}
impl CueAngularQ4 {
    pub fn new(config: CueAngularConfig, packed: &[u8]) -> Result<Self> {
        let q = unpack_coefficients(config.coefficient_count()?, packed)
            .map_err(|e| error(e.to_string()))?;
        let mut rows = q.chunks_exact(ROOT_COUNT);
        let mut tables = Vec::new();
        for _ in 0..config.heads {
            let mut head = Vec::new();
            for _ in 0..config.lanes_per_head {
                let row = rows
                    .next()
                    .ok_or_else(|| error("cue angular table row absent"))?;
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
            joint: None,
        })
    }
    pub fn config(&self) -> CueAngularConfig {
        self.config
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        &self.packed
    }
    pub fn with_joint(mut self, joint: CueJointQ4) -> Result<Self> {
        let c = joint.config();
        if self.config.mode != CueScoreMode::DirectedRelative
            || c.head >= self.config.heads
            || c.left_lane >= self.config.lanes_per_head
            || c.right_lane >= self.config.lanes_per_head
        {
            return Err(error("joint cue requires valid directed-relative lanes"));
        }
        self.joint = Some(joint);
        Ok(self)
    }
    pub fn joint(&self) -> Option<&CueJointQ4> {
        self.joint.as_ref()
    }
    fn contribution(
        &self,
        head: usize,
        lane: usize,
        query: AddressLane,
        cue: AddressLane,
        geometry: &HistoricalH4Tables,
    ) -> Result<(i64, Option<u8>, Option<u8>)> {
        let table = self
            .tables
            .get(head)
            .and_then(|h| h.get(lane))
            .ok_or_else(|| error("cue angular head/lane absent"))?;
        if !query.present() || !cue.present() {
            return Ok((0, None, None));
        }
        let q = H4Code::try_from(query.root()).map_err(|e| error(e.to_string()))?;
        let k = H4Code::try_from(cue.root()).map_err(|e| error(e.to_string()))?;
        let relative = geometry.relative(q, k).index();
        let index = match self.config.mode {
            CueScoreMode::DirectedRelative => relative,
            CueScoreMode::CueUnary => cue.root(),
        };
        Ok((
            i64::from(table[usize::from(index)]),
            Some(index),
            Some(relative),
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueCarrierMetadata {
    pub schema: String,
    pub policy: String,
    pub parent_artifact: NativeArtifactBinding,
    pub context: ContextQ4Config,
    pub context_packed_sha256: String,
    pub potential: CueAngularConfig,
    pub potential_packed_sha256: String,
    pub algebra_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joint: Option<CueJointMetadata>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CarrierState {
    pub token_ids: Vec<u32>,
    /// Retained latent roots are distinct from lossy/possibly absent observations.
    pub states: Vec<u8>,
    pub raw_roots: Vec<u8>,
    pub categories: Vec<u8>,
    pub codes: Vec<ObservedCode>,
}
impl CarrierState {
    fn addresses(&self) -> Result<Vec<AddressLane>> {
        self.codes
            .iter()
            .map(|c| {
                AddressLane::new(c.root, c.radius_bin, c.present).map_err(|e| error(e.to_string()))
            })
            .collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CueSegmentTrace {
    pub source_segment_index: usize,
    pub context_segment_index: usize,
    pub state: CarrierState,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CueCarrierCosts {
    pub extra_encoder_tokens: usize,
    pub extra_context_coefficient_reads: usize,
    pub extra_potential_table_reads: usize,
    pub extra_geometry_relative_reads: usize,
    /// Serialized payload-sized owned sidecar data, excluding allocator overhead.
    pub logical_sidecar_payload_bytes: usize,
    pub compiled_table_payload_bytes: usize,
    pub packed_source_bytes: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CueCarrierTrace {
    pub metadata: CueCarrierMetadata,
    pub query: CarrierState,
    pub cues: Vec<CueSegmentTrace>,
    /// Exact bank candidate ordinal -> cue index; None is structural absence.
    pub candidate_cue_indices: Vec<Option<usize>>,
    pub copy_q24: Vec<Vec<i64>>,
    /// Global head-major lane, then candidate. None means structural/masked absence.
    pub angular_indices: Vec<Vec<Option<u8>>>,
    pub relative_roots: Vec<Vec<Option<u8>>>,
    pub costs: CueCarrierCosts,
}
/// An explicit borrowed component boundary: equal shapes are insufficient.
/// The exact compiled encoder and geometry instances must outlive this carrier.
/// Metadata/payload accessors let an outer bundle bind the new component; this
/// constructor is not a filesystem artifact loader or a self-trusted receipt.
pub struct NativeCueCarrier<'a> {
    context: &'a NativeContextQ4,
    geometry: &'a HistoricalH4Tables,
    potential: CueAngularQ4,
    packed: Vec<u8>,
    table_bytes: usize,
    metadata: CueCarrierMetadata,
}
impl<'a> NativeCueCarrier<'a> {
    pub fn compile(
        parent: NativeArtifactBinding,
        context: &'a NativeContextQ4,
        geometry: &'a HistoricalH4Tables,
        potential: CueAngularQ4,
    ) -> Result<Self> {
        parent.identity.validate()?;
        if parent.metadata_sha256.len() != 64
            || !parent
                .metadata_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(error("cue parent digest invalid"));
        }
        let c = context.config();
        let p = potential.config();
        if p.heads != c.heads || p.lanes_per_head != c.lanes_per_head {
            return Err(error("cue encoder/potential dimensions differ"));
        }
        let packed = potential.packed_coefficients().to_vec();
        let metadata = CueCarrierMetadata {
            schema: SCHEMA.into(),
            policy: POLICY.into(),
            parent_artifact: parent,
            context: c,
            context_packed_sha256: sha(context.packed_coefficients()),
            potential: p,
            potential_packed_sha256: sha(&packed),
            algebra_sha256: TRUSTED_MATHEMATICAL_SHA256
                .ok_or_else(|| error("cue algebra trust anchor absent"))?
                .into(),
            joint: potential.joint().map(CueJointQ4::metadata),
        };
        let table_bytes = p.coefficient_count()? * std::mem::size_of::<i32>()
            + potential
                .joint()
                .map_or(0, |j| j.table.len() * std::mem::size_of::<i32>());
        Ok(Self {
            context,
            geometry,
            potential,
            packed,
            table_bytes,
            metadata,
        })
    }
    pub fn metadata(&self) -> &CueCarrierMetadata {
        &self.metadata
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        &self.packed
    }
    pub fn joint(&self) -> Option<&CueJointQ4> {
        self.potential.joint()
    }
    pub fn angular_source(&self) -> CueAngularQ4 {
        self.potential.clone()
    }
    pub(crate) fn validate_execution(
        &self,
        parent: &NativeArtifactBinding,
        context: &NativeContextQ4,
        geometry: &HistoricalH4Tables,
    ) -> Result<()> {
        if parent != &self.metadata.parent_artifact
            || !std::ptr::eq(context, self.context)
            || !std::ptr::eq(geometry, self.geometry)
        {
            return Err(error("foreign cue parent/encoder/geometry"));
        }
        Ok(())
    }
    fn encode(&self, ids: &[u32], costs: &mut CueCarrierCosts) -> Result<CarrierState> {
        if ids.is_empty() || ids.len() > MAX_SEQUENCE {
            return Err(error("cue encoder empty/oversized sequence"));
        }
        let c = self.context.config();
        let width = c.heads * c.lanes_per_head;
        let mut state =
            NativeContextState::new(c.heads, c.lanes_per_head).map_err(|e| error(e.to_string()))?;
        let mut last = None;
        for &token in ids {
            let step = state
                .step(token as usize, self.context.native(), self.geometry)
                .map_err(|e| error(e.to_string()))?;
            costs.extra_context_coefficient_reads += step.coefficient_reads;
            costs.extra_encoder_tokens += 1;
            last = Some(step);
        }
        let step = last.ok_or_else(|| error("cue final state absent"))?;
        Ok(CarrierState {
            token_ids: ids.to_vec(),
            states: step.states[..width].iter().map(|x| x.index()).collect(),
            raw_roots: step.readout_roots[..width]
                .iter()
                .map(|x| x.index())
                .collect(),
            categories: step.categories[..width].to_vec(),
            codes: step.output[..width]
                .iter()
                .copied()
                .map(ObservedCode::from)
                .collect(),
        })
    }
    /// Target-free source score, including sources without Copy occurrences.
    /// The legacy per-occurrence preparation remains unchanged.
    pub(crate) fn score_source_cue(
        &self,
        trace: &CueCarrierTrace,
        source_segment_index: usize,
    ) -> Result<(Vec<i64>, CueCarrierCosts)> {
        if trace.metadata != self.metadata {
            return Err(error("source cue trace metadata differs"));
        }
        let c = self.context.config();
        let mut scores = vec![0i64; c.heads];
        let mut costs = CueCarrierCosts {
            logical_sidecar_payload_bytes: c.heads * 8,
            ..Default::default()
        };
        let matches = trace
            .cues
            .iter()
            .filter(|s| s.source_segment_index == source_segment_index)
            .collect::<Vec<_>>();
        if matches.len() > 1 {
            return Err(error("source cue segment appears more than once"));
        }
        let Some(cue) = matches.first() else {
            return Ok((scores, costs));
        };
        let query = trace.query.addresses()?;
        let source = cue.state.addresses()?;
        let width = c.heads * c.lanes_per_head;
        if query.len() != width || source.len() != width {
            return Err(error("source cue code shape differs"));
        }
        let mut relative = vec![None; width];
        for (h, score) in scores.iter_mut().enumerate() {
            for lane in 0..c.lanes_per_head {
                let global = h * c.lanes_per_head + lane;
                let (value, bin, root) = self.potential.contribution(
                    h,
                    lane,
                    query[global],
                    source[global],
                    self.geometry,
                )?;
                *score = score
                    .checked_add(value)
                    .ok_or_else(|| error("source cue angular sum overflow"))?;
                relative[global] = root;
                if bin.is_some() {
                    costs.extra_potential_table_reads += 1;
                    costs.extra_geometry_relative_reads += 1;
                }
            }
        }
        if let Some(joint) = self.joint() {
            let jc = joint.config();
            let left = jc.head * c.lanes_per_head + jc.left_lane;
            let right = jc.head * c.lanes_per_head + jc.right_lane;
            if let Some(value) = joint.contribution(relative[left], relative[right])? {
                scores[jc.head] = scores[jc.head]
                    .checked_add(value)
                    .ok_or_else(|| error("source cue joint sum overflow"))?;
                costs.extra_potential_table_reads += 1;
            }
        }
        costs.logical_sidecar_payload_bytes =
            scores.len() * 8 + relative.len() * std::mem::size_of::<Option<u8>>();
        Ok((scores, costs))
    }

    pub(crate) fn prepare(
        &self,
        segments: &[OccurrenceBankSegment<'_>],
        query: &[u32],
        candidate_segments: &[usize],
    ) -> Result<CueCarrierTrace> {
        if segments.len() > MAX_SEQUENCE || candidate_segments.len() > MAX_SEQUENCE {
            return Err(error("cue segment/candidate cap exceeds128"));
        }
        let mut costs = CueCarrierCosts {
            compiled_table_payload_bytes: self.table_bytes,
            packed_source_bytes: self.packed.len()
                + self.joint().map_or(0, |j| j.packed_coefficients().len()),
            ..Default::default()
        };
        let query = self.encode(query, &mut costs)?;
        let mut cues = Vec::new();
        let mut source_cues = vec![None; segments.len()];
        for (index, segment) in segments.iter().enumerate() {
            if matches!(segment, OccurrenceBankSegment::Source { .. }) && index > 0 {
                if let OccurrenceBankSegment::Context { token_ids, .. } = segments[index - 1] {
                    if !token_ids.is_empty() {
                        source_cues[index] = Some(cues.len());
                        cues.push(CueSegmentTrace {
                            source_segment_index: index,
                            context_segment_index: index - 1,
                            state: self.encode(token_ids, &mut costs)?,
                        });
                    }
                }
            }
        }
        let mut candidate_cue_indices = Vec::new();
        let mut expected_segments = Vec::new();
        for (index, segment) in segments.iter().enumerate() {
            if let OccurrenceBankSegment::Source { frame, .. } = segment {
                expected_segments.extend(std::iter::repeat_n(index, frame.token_ids.len()));
            }
        }
        if expected_segments != candidate_segments {
            return Err(error("cue candidate/source ordinal binding differs"));
        }
        for &index in candidate_segments {
            candidate_cue_indices.push(source_cues[index]);
        }
        let c = self.context.config();
        let q = query.addresses()?;
        let cue_codes = cues
            .iter()
            .map(|cue| cue.state.addresses())
            .collect::<Result<Vec<_>>>()?;
        let mut copy_q24 = vec![vec![0i64; candidate_segments.len()]; c.heads];
        let mut angular_indices =
            vec![vec![None; candidate_segments.len()]; c.heads * c.lanes_per_head];
        let mut relative_roots = angular_indices.clone();
        for (h, scores) in copy_q24.iter_mut().enumerate() {
            for (j, cue) in candidate_cue_indices.iter().enumerate() {
                if let Some(index) = cue {
                    for lane in 0..c.lanes_per_head {
                        let global_lane = h * c.lanes_per_head + lane;
                        let (score, bin, relative) = self.potential.contribution(
                            h,
                            lane,
                            q[global_lane],
                            cue_codes[*index][global_lane],
                            self.geometry,
                        )?;
                        scores[j] = scores[j]
                            .checked_add(score)
                            .ok_or_else(|| error("cue angular sum overflow"))?;
                        angular_indices[global_lane][j] = bin;
                        relative_roots[global_lane][j] = relative;
                        if bin.is_some() {
                            costs.extra_potential_table_reads += 1;
                            costs.extra_geometry_relative_reads += 1;
                        }
                    }
                }
            }
        }
        if let Some(joint) = self.joint() {
            let jc = joint.config();
            let left = jc.head * c.lanes_per_head + jc.left_lane;
            let right = jc.head * c.lanes_per_head + jc.right_lane;
            for j in 0..candidate_segments.len() {
                if let Some(score) =
                    joint.contribution(relative_roots[left][j], relative_roots[right][j])?
                {
                    copy_q24[jc.head][j] = copy_q24[jc.head][j]
                        .checked_add(score)
                        .ok_or_else(|| error("joint cue sum overflow"))?;
                    costs.extra_potential_table_reads += 1;
                }
            }
        }
        let payload =
            |s: &CarrierState| s.token_ids.len() * 4 + s.states.len() * 3 + s.codes.len() * 3;
        costs.logical_sidecar_payload_bytes = payload(&query)
            + cues
                .iter()
                .map(|x| payload(&x.state) + 2 * std::mem::size_of::<usize>())
                .sum::<usize>()
            + candidate_cue_indices.len() * std::mem::size_of::<Option<usize>>()
            + c.heads * candidate_segments.len() * 8
            + angular_indices.len()
                * candidate_segments.len()
                * 2
                * std::mem::size_of::<Option<u8>>();
        Ok(CueCarrierTrace {
            metadata: self.metadata.clone(),
            query,
            cues,
            candidate_cue_indices,
            copy_q24,
            angular_indices,
            relative_roots,
            costs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn joint_order_mask_and_signed_roots_are_preserved() -> Result<()> {
        use crate::geometric_no_read::CANONICAL_BASIS_Q25;
        use crate::geometric_potential_q4::pack_coefficients;
        let config = CueJointConfig {
            head: 1,
            left_lane: 1,
            right_lane: 3,
        };
        let zero = CueJointQ4::new(config, vec![0; 8])?;
        assert!(zero.table.iter().all(|v| *v == 0));
        let mut q = vec![0; 16];
        q[1] = 1;
        let joint = CueJointQ4::new(
            config,
            pack_coefficients(&q).map_err(|e| error(e.to_string()))?,
        )?;
        let unit = |axis| CANONICAL_BASIS_Q25.iter().position(|r| r[axis] == 1 << 25);
        let left = unit(0).ok_or_else(|| error("basis unit0 absent"))? as u8;
        let right = unit(1).ok_or_else(|| error("basis unit1 absent"))? as u8;
        let negative = CANONICAL_BASIS_Q25
            .iter()
            .position(|r| r[0] == -(1 << 25))
            .ok_or_else(|| error("basis negative unit0 absent"))? as u8;
        assert_eq!(joint.contribution(Some(left), Some(right))?, Some(1 << 22));
        assert_eq!(joint.contribution(Some(right), Some(left))?, Some(0));
        assert_eq!(
            joint.contribution(Some(negative), Some(right))?,
            Some(-(1 << 22))
        );
        assert_eq!(joint.contribution(None, Some(right))?, None);
        assert_eq!(joint.contribution(Some(left), None)?, None);
        assert!(joint.contribution(Some(120), Some(right)).is_err());
        assert!(CueJointQ4::new(config, vec![0; 7]).is_err());
        assert!(CueJointQ4::new(config, vec![0x88; 8]).is_err());
        let directed = CueAngularConfig {
            heads: 2,
            lanes_per_head: 4,
            mode: CueScoreMode::DirectedRelative,
        };
        let potential = CueAngularQ4::new(directed, &vec![0; 480])?.with_joint(joint)?;
        assert_eq!(potential.joint().map(|j| j.table.len()), Some(16384));
        assert!(CueAngularQ4::new(
            CueAngularConfig {
                mode: CueScoreMode::CueUnary,
                ..directed
            },
            &vec![0; 480]
        )?
        .with_joint(zero)
        .is_err());
        Ok(())
    }

    #[test]
    fn angular_lookup_preserves_signed_bins_and_masks_observed_absence() -> Result<()> {
        let geometry = HistoricalH4Tables::from_bytes(include_bytes!(
            "../fixtures/historical-h4-tables-v1.bin"
        ))
        .map_err(|e| error(e.to_string()))?;
        let config = CueAngularConfig {
            heads: 1,
            lanes_per_head: 1,
            mode: CueScoreMode::DirectedRelative,
        };
        let mut q = vec![0; config.coefficient_count()?];
        q[0] = -7;
        q[1] = 7;
        let packed = crate::geometric_potential_q4::pack_coefficients(&q)
            .map_err(|e| error(e.to_string()))?;
        let table = CueAngularQ4::new(config, &packed)?;
        let identity = AddressLane::new(1, 31, true).map_err(|e| error(e.to_string()))?;
        let antipode = AddressLane::new(0, 0, true).map_err(|e| error(e.to_string()))?;
        let absent = AddressLane::new(1, 0, false).map_err(|e| error(e.to_string()))?;
        assert_eq!(
            table.contribution(0, 0, identity, identity, &geometry)?,
            (7i64 << 22, Some(1), Some(1))
        );
        assert_eq!(
            table.contribution(0, 0, identity, antipode, &geometry)?,
            (-(7i64 << 22), Some(0), Some(0))
        );
        assert_eq!(
            table.contribution(0, 0, absent, antipode, &geometry)?,
            (0, None, None)
        );
        assert_eq!(
            table.contribution(0, 0, identity, absent, &geometry)?,
            (0, None, None)
        );
        // v1 explicitly does not score radius; signed root identity is intact.
        let different_radius = AddressLane::new(1, 0, true).map_err(|e| error(e.to_string()))?;
        assert_eq!(
            table.contribution(0, 0, identity, identity, &geometry)?,
            table.contribution(0, 0, different_radius, identity, &geometry)?
        );
        Ok(())
    }
}
