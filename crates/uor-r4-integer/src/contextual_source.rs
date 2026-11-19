//! Source-start exemplars and the native integer decision shared by D25 and D26.
#![forbid(unsafe_code)]

use super::{
    orbit_root_coords, paired_copy, read_role_digest, root_id, IntegerDiscreteModel, ALIGN,
    DISCRETE_REQUESTED_OCCURRENCE_SCHEMA, MAGIC as D11_MAGIC, ROLE_COPY_STOPS,
};
use crate::{
    assertion_value::assertion_fence,
    invalid,
    ordered_role::{RoleMode, RoleSites, ROLE_LIMIT},
    paired_role::RoleSpan,
    Result,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAGIC: &[u8; 8] = b"D25BANK1";
pub const SCHEMA: &str = "uor-r4.lut-discrete-contextual-source/1";
pub const RECURRENCE_SCHEMA: &str = "uor-r4.lut-discrete-contextual-source/2";
pub const DIMENSIONS: usize = 32;
pub const RECORD_BYTES: usize = 1 + 2 * DIMENSIONS + 4 * DIMENSIONS;
pub const MAX_BANK_BYTES: usize = 12 + 8192 * RECORD_BYTES;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceArm {
    NarrowLocal,
    WideLocal,
    NarrowContextual,
    WideContextual,
    /// D26 opt-in recurrence read for engineering replay. The parent policy
    /// remains the default. A fit's qualification is external to this schema.
    WideRecurrence,
}

impl SourceArm {
    pub fn includes(self, candidate: &SourceCandidate) -> bool {
        candidate.narrow
            || matches!(
                self,
                Self::WideLocal | Self::WideContextual | Self::WideRecurrence
            )
    }

    fn uses_context(self) -> bool {
        matches!(
            self,
            Self::NarrowContextual | Self::WideContextual | Self::WideRecurrence
        )
    }

    fn uses_recurrence(self) -> bool {
        self == Self::WideRecurrence
    }

    pub(super) fn schema(self) -> &'static str {
        if self.uses_recurrence() {
            RECURRENCE_SCHEMA
        } else {
            SCHEMA
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceCandidate {
    pub span: RoleSpan,
    pub narrow: bool,
    pub local: [i16; DIMENSIONS],
    pub contextual: [i32; DIMENSIONS],
    pub completed_source_range: [u32; 2],
}

#[derive(Clone, Debug)]
struct Exemplar {
    label: i8,
    local: [i16; DIMENSIONS],
    contextual: [i32; DIMENSIONS],
}

#[derive(Clone, Debug)]
pub struct SourceBank(Vec<Exemplar>);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SourceScore {
    pub source: u32,
    pub eligible: bool,
    pub nearest_positive: Option<i64>,
    pub nearest_negative: Option<i64>,
    pub margin: Option<i64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceOutcome {
    Pick,
    Empty,
    Nonpositive,
    Tie,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SourceSelection {
    pub arm: SourceArm,
    pub outcome: SourceOutcome,
    pub source: Option<u32>,
}

impl SourceBank {
    pub fn from_bytes(raw: &[u8]) -> Result<Self> {
        if raw.len() < 12 || raw.len() > MAX_BANK_BYTES || &raw[..8] != MAGIC {
            return Err(invalid("invalid contextual source bank magic or size"));
        }
        let count = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
        if count > 8192 || raw.len() != 12 + count * RECORD_BYTES {
            return Err(invalid("invalid contextual source bank record count"));
        }
        let mut exemplars = Vec::with_capacity(count);
        for record in raw[12..].chunks_exact(RECORD_BYTES) {
            let label = record[0] as i8;
            let local = core::array::from_fn(|i| {
                let j = 1 + 2 * i;
                i16::from_le_bytes([record[j], record[j + 1]])
            });
            let contextual = core::array::from_fn(|i| {
                let j = 1 + 2 * DIMENSIONS + 4 * i;
                i32::from_le_bytes([record[j], record[j + 1], record[j + 2], record[j + 3]])
            });
            if !matches!(label, -1 | 1)
                || local
                    .iter()
                    .any(|x| !(-ROLE_LIMIT..=ROLE_LIMIT).contains(x))
            {
                return Err(invalid("invalid contextual source exemplar"));
            }
            exemplars.push(Exemplar {
                label,
                local,
                contextual,
            });
        }
        if ![-1, 1]
            .iter()
            .all(|label| exemplars.iter().any(|row| row.label == *label))
        {
            return Err(invalid("contextual source bank requires both polarities"));
        }
        Ok(Self(exemplars))
    }

    pub fn score(&self, arm: SourceArm, candidate: &SourceCandidate) -> SourceScore {
        let mut result = SourceScore {
            source: candidate.span.start as u32,
            eligible: arm.includes(candidate),
            nearest_positive: None,
            nearest_negative: None,
            margin: None,
        };
        for row in &self.0 {
            let mut distance = row
                .local
                .iter()
                .zip(candidate.local)
                .map(|(&a, b)| i64::from(a.abs_diff(b)))
                .sum::<i64>();
            if arm.uses_context() {
                distance += row
                    .contextual
                    .iter()
                    .zip(candidate.contextual)
                    .map(|(&a, b)| i64::from(a.abs_diff(b)))
                    .sum::<i64>();
            }
            let nearest = if row.label > 0 {
                &mut result.nearest_positive
            } else {
                &mut result.nearest_negative
            };
            *nearest = Some(nearest.map_or(distance, |old| old.min(distance)));
        }
        result.margin = result
            .nearest_negative
            .zip(result.nearest_positive)
            .map(|(negative, positive)| negative - positive);
        result
    }

    pub fn select(
        &self,
        arm: SourceArm,
        candidates: impl IntoIterator<Item = Result<SourceCandidate>>,
        mut observe: impl FnMut(SourceCandidate, SourceScore),
    ) -> Result<SourceSelection> {
        let mut best = None;
        let mut outcome = SourceOutcome::Empty;
        for candidate in candidates {
            let candidate = candidate?;
            let score = self.score(arm, &candidate);
            observe(candidate, score);
            if !score.eligible {
                continue;
            }
            let Some(margin) = score.margin else { continue };
            if margin <= 0 {
                if outcome == SourceOutcome::Empty {
                    outcome = SourceOutcome::Nonpositive;
                }
                continue;
            }
            if best.is_none_or(|(previous, _)| margin > previous) {
                best = Some((margin, score.source));
                outcome = SourceOutcome::Pick;
            } else if best.is_some_and(|(previous, _)| margin == previous) {
                outcome = SourceOutcome::Tie;
            }
        }
        Ok(SourceSelection {
            arm,
            outcome,
            source: best
                .filter(|_| outcome == SourceOutcome::Pick)
                .map(|(_, source)| source),
        })
    }
}

#[derive(Clone, Debug)]
pub(super) struct SourcePolicy {
    arm: SourceArm,
    bank: SourceBank,
    bindings: SourceCarrierBindings,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceCarrierBindings {
    pub parent_sha256: String,
    pub bank_sha256: String,
    pub train_sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: String,
    arm: SourceArm,
    bank_bytes: usize,
    parent_sha256: String,
    train_sha256: String,
    bank_sha256: String,
    policy_sha256: String,
}

impl SourceArm {
    fn code(self) -> u8 {
        match self {
            Self::NarrowLocal => 0,
            Self::WideLocal => 1,
            Self::NarrowContextual => 2,
            Self::WideContextual => 3,
            Self::WideRecurrence => 4,
        }
    }
}

fn policy_digest(arm: SourceArm, parent: [u8; 32], train: [u8; 32], bank: [u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    if arm.uses_recurrence() {
        hash.update(b"uor-r4.contextual-source/2;D21-frozen-assertion-agreement;assertion-local-positional;D11-active-holonomy-scale-matched-snap-before-fact;4x8-low-nibble-first;first-stop-writer-EOS;wide;L1-signed-unique-positive");
    } else {
        hash.update(b"uor-r4.contextual-source/1;D21-frozen-assertion-agreement;assertion-local-positional;D11-before-fact;first-stop-writer-EOS;2..4;L1-signed-unique-positive");
    }
    hash.update(crate::ordered_role::orbit_action_digest());
    hash.update([arm.code()]);
    hash.update(parent);
    hash.update(train);
    hash.update(bank);
    hash.finalize().into()
}

/// Bind the supplied bank and TRAIN digest to the chosen read and exact parent.
/// The caller verifies fit provenance. Post-gate and recurrence records can
/// share a byte layout, and an orbit membership check cannot identify their fit.
pub fn pack_contextual_source_artifact(
    parent: &[u8],
    bank: &[u8],
    train_digest: [u8; 32],
    arm: SourceArm,
) -> Result<Vec<u8>> {
    let parent_digest = Sha256::digest(parent).into();
    let bank_digest = Sha256::digest(bank).into();
    let encoded = serde_json::to_vec(&Envelope {
        schema: arm.schema().to_owned(),
        arm,
        bank_bytes: bank.len(),
        parent_sha256: hex::encode(parent_digest),
        train_sha256: hex::encode(train_digest),
        bank_sha256: hex::encode(bank_digest),
        policy_sha256: hex::encode(policy_digest(arm, parent_digest, train_digest, bank_digest)),
    })?;
    let mut file = D11_MAGIC.to_vec();
    file.extend_from_slice(&(encoded.len() as u64).to_le_bytes());
    file.extend_from_slice(&encoded);
    file.resize(file.len().next_multiple_of(ALIGN), 0);
    file.extend_from_slice(bank);
    file.extend_from_slice(parent);
    IntegerDiscreteModel::from_bytes(&file)?;
    Ok(file)
}

pub(super) fn read_carrier(
    bytes: &[u8],
    encoded_header: &[u8],
    end_header: usize,
    allow_role: bool,
) -> Result<IntegerDiscreteModel> {
    if !allow_role {
        return Err(invalid("nested contextual source carrier"));
    }
    let envelope: Envelope = serde_json::from_slice(encoded_header)?;
    if envelope.schema != envelope.arm.schema() {
        return Err(invalid("contextual source schema and read arm disagree"));
    }
    let start = end_header.next_multiple_of(ALIGN);
    let payload = bytes
        .get(start..)
        .ok_or_else(|| invalid("missing contextual source payload"))?;
    if envelope.bank_bytes > MAX_BANK_BYTES || bytes[end_header..start].iter().any(|&v| v != 0) {
        return Err(invalid("invalid contextual source bounds or padding"));
    }
    let (bank, parent) = payload
        .split_at_checked(envelope.bank_bytes)
        .ok_or_else(|| invalid("truncated contextual source bank"))?;
    let parent_digest = read_role_digest(&envelope.parent_sha256)?;
    let train_digest = read_role_digest(&envelope.train_sha256)?;
    let bank_digest = read_role_digest(&envelope.bank_sha256)?;
    if Sha256::digest(parent).as_slice() != parent_digest
        || Sha256::digest(bank).as_slice() != bank_digest
        || policy_digest(envelope.arm, parent_digest, train_digest, bank_digest)
            != read_role_digest(&envelope.policy_sha256)?
    {
        return Err(invalid("contextual source carrier binding mismatch"));
    }
    let bank = SourceBank::from_bytes(bank)?;
    if envelope.arm.uses_recurrence()
        && bank.0.iter().any(|row| {
            row.contextual
                .chunks_exact(8)
                .any(|block| root_id(&core::array::from_fn(|i| block[i])).is_none())
        })
    {
        return Err(invalid(
            "contextual source recurrence exemplar is outside the D11 orbit",
        ));
    }
    // The existing D21 carrier is the only legal parent. Inspect its envelope
    // before parsing so recursive contextual carriers cannot reach this path.
    let parent_header = parent
        .get(..16)
        .filter(|prefix| &prefix[..8] == D11_MAGIC)
        .ok_or_else(|| invalid("missing contextual source D21 parent"))?;
    let length = u64::from_le_bytes(
        parent_header[8..]
            .try_into()
            .map_err(|_| invalid("invalid contextual source parent length"))?,
    );
    if length > super::MAX_HEADER {
        return Err(invalid("contextual source parent header too large"));
    }
    let header = parent
        .get(16..16 + length as usize)
        .ok_or_else(|| invalid("truncated contextual source D21 parent"))?;
    #[derive(Deserialize)]
    struct Schema {
        schema: String,
    }
    if serde_json::from_slice::<Schema>(header)?.schema != DISCRETE_REQUESTED_OCCURRENCE_SCHEMA {
        return Err(invalid(
            "contextual source requires its D21 assertion-agreement parent",
        ));
    }
    let mut model = IntegerDiscreteModel::from_bytes(parent)?;
    if model.shape.blocks != 4
        || model.active_s2().is_some()
        || model.count_mixer().is_some()
        || (envelope.arm.uses_recurrence() && !model.is_trunk_d11())
    {
        return Err(invalid(
            "contextual source parent requires the original 4x8 D11 read",
        ));
    }
    model.contextual_source = Some(SourcePolicy {
        arm: envelope.arm,
        bank,
        bindings: SourceCarrierBindings {
            parent_sha256: envelope.parent_sha256,
            bank_sha256: envelope.bank_sha256,
            train_sha256: envelope.train_sha256,
        },
    });
    model.artifact_sha256 = hex::encode(Sha256::digest(bytes));
    Ok(model)
}

impl IntegerDiscreteModel {
    pub fn contextual_source_arm(&self) -> Option<SourceArm> {
        Some(self.contextual_source.as_ref()?.arm)
    }

    pub fn contextual_source_bindings(&self) -> Option<&SourceCarrierBindings> {
        Some(&self.contextual_source.as_ref()?.bindings)
    }

    /// Diagnostic extraction on a factual occurrence already admitted by the
    /// original binder. A caller may supply the query endpoints for an oracle,
    /// but the writer source comes only from the projected original candidates.
    pub fn contextual_source_candidates(
        &self,
        ids: &[u32],
        fact: RoleSpan,
        query: RoleSpan,
    ) -> Result<Vec<SourceCandidate>> {
        self.validate_contextual_assertion(ids, fact, query)?;
        self.with_contextual_candidates(ids, fact, query, |_, candidates| candidates.collect())
    }

    pub fn contextual_source_trace(
        &self,
        ids: &[u32],
        fact: RoleSpan,
        query: RoleSpan,
    ) -> Result<(SourceSelection, Vec<(SourceCandidate, SourceScore)>)> {
        self.validate_contextual_assertion(ids, fact, query)?;
        let policy = self
            .contextual_source
            .as_ref()
            .ok_or_else(|| invalid("missing contextual source policy"))?;
        self.with_contextual_candidates(ids, fact, query, |_, candidates| {
            let mut trace = Vec::new();
            let selected = policy
                .bank
                .select(policy.arm, candidates, |candidate, score| {
                    trace.push((candidate, score));
                })?;
            Ok((selected, trace))
        })
    }

    fn validate_contextual_assertion(
        &self,
        ids: &[u32],
        fact: RoleSpan,
        query: RoleSpan,
    ) -> Result<()> {
        paired_copy::Policy::validate_ids(ids)?;
        let (policy, paired) = self
            .paired_policy()
            .ok_or_else(|| invalid("contextual source requires its D21 binder"))?;
        if self.requested_role_framing().is_none()
            || (self.contextual_source.is_none()
                && self.artifact_schema() != DISCRETE_REQUESTED_OCCURRENCE_SCHEMA)
            || !self
                .ordered_role_sites()?
                .prompt(ids, 1..ids.len())?
                .candidates()
                .any(|span| span == query)
            || !paired
                .assertion_sites(policy, ids, query)
                .any(|(start, last)| fact.start == start && fact.end == last + 1)
        {
            return Err(invalid(
                "contextual source requires an original D21 bound assertion",
            ));
        }
        Ok(())
    }

    pub(super) fn contextual_selection(
        &self,
        ids: &[u32],
        fact: RoleSpan,
        query: RoleSpan,
    ) -> Result<Option<(Option<u32>, SourceSelection)>> {
        let Some(policy) = &self.contextual_source else {
            return Ok(None);
        };
        self.with_contextual_candidates(ids, fact, query, |fence, candidates| {
            Ok(Some((
                fence,
                policy.bank.select(policy.arm, candidates, |_, _| {})?,
            )))
        })
    }

    fn with_contextual_candidates<T>(
        &self,
        ids: &[u32],
        fact: RoleSpan,
        query: RoleSpan,
        consume: impl FnOnce(
            Option<u32>,
            &mut dyn Iterator<Item = Result<SourceCandidate>>,
        ) -> Result<T>,
    ) -> Result<T> {
        let fence = assertion_fence(ids, fact.end, query.start, ROLE_COPY_STOPS);
        let Some(fence_position) = fence.map(|p| p as usize) else {
            return consume(None, &mut core::iter::empty());
        };
        let Some((policy, _)) = self.paired_policy() else {
            return Err(invalid("missing contextual source span classes"));
        };
        if fact.start == 0
            || fact.start > self.shape.context as usize
            || self.shape.blocks != 4
            || self.active_s2().is_some()
            || self.count_mixer().is_some()
        {
            return Err(invalid(
                "contextual source requires a bounded causal 4x8 D11 read",
            ));
        }
        let contextual =
            self.contextual_source_coordinates(ids, fact.start, self.contextual_source_arm())?;
        let projected = RoleSites::from_span_classes(&self.codebook_roots, &policy.span.classes)
            .prompt(ids, 1..ids.len())?;
        let region = fact.start..fence_position;
        let mut candidates = projected
            .candidates()
            .filter(|span| span.start >= fact.end && span.end <= fence_position)
            .filter_map(|span| {
                let source = span.start as u32;
                match paired_copy::completed_sequence(ids, source) {
                    Ok(Some(sequence))
                        if source + sequence.len() as u32 == fence_position as u32 =>
                    {
                        let offset = span.start - (fact.end - 1);
                        Some(
                            projected
                                .feature(RoleMode::Positional, span, region.clone())
                                .map(|local| SourceCandidate {
                                    span,
                                    narrow: (2..=4).contains(&offset),
                                    local,
                                    contextual,
                                    completed_source_range: [source, fence_position as u32],
                                }),
                        )
                    }
                    Ok(_) => None,
                    Err(error) => Some(Err(error)),
                }
            });
        consume(fence, &mut candidates)
    }

    pub(super) fn contextual_source_coordinates(
        &self,
        ids: &[u32],
        fact_start: usize,
        arm: Option<SourceArm>,
    ) -> Result<[i32; DIMENSIONS]> {
        let prefix = ids
            .get(..fact_start)
            .filter(|prefix| !prefix.is_empty() && prefix.len() <= self.shape.context as usize)
            .ok_or_else(|| {
                invalid("contextual source requires a nonempty bounded causal prefix")
            })?;
        if arm.is_some_and(SourceArm::uses_recurrence) {
            let trunk = self
                .trunk
                .as_ref()
                .ok_or_else(|| invalid("contextual recurrence requires the D11 trunk"))?;
            let code = trunk.compute_rec_code(prefix, &self.codebook_roots)?;
            return Ok(expand_recurrence_code(code));
        }
        let (blocks, _, _) = self.causal_geometric_read(prefix, None)?;
        let blocks: [[i32; 8]; 4] = blocks
            .try_into()
            .map_err(|_| invalid("contextual source D11 read width differs from 32"))?;
        Ok(core::array::from_fn(|i| blocks[i / 8][i % 8]))
    }
}

fn expand_recurrence_code(code: u16) -> [i32; DIMENSIONS] {
    let blocks: [[i32; 8]; 4] =
        core::array::from_fn(|b| orbit_root_coords(usize::from((code >> (b * 4)) & 15)));
    core::array::from_fn(|i| blocks[i / 8][i % 8])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(bytes: &mut Vec<u8>, label: i8, local: i16, context: i32) {
        bytes.push(label as u8);
        for i in 0..DIMENSIONS {
            bytes.extend_from_slice(&(if i == 0 { local } else { 0 }).to_le_bytes());
        }
        for i in 0..DIMENSIONS {
            bytes.extend_from_slice(&(if i == 0 { context } else { 0 }).to_le_bytes());
        }
    }

    fn candidate(start: usize, narrow: bool, local: i16, context: i32) -> SourceCandidate {
        let mut feature = SourceCandidate {
            span: RoleSpan {
                start,
                end: start + 1,
            },
            narrow,
            local: [0; DIMENSIONS],
            contextual: [0; DIMENSIONS],
            completed_source_range: [start as u32, 14],
        };
        feature.local[0] = local;
        feature.contextual[0] = context;
        feature
    }

    fn select(
        bank: &SourceBank,
        arm: SourceArm,
        rows: impl IntoIterator<Item = SourceCandidate>,
    ) -> Result<SourceSelection> {
        bank.select(arm, rows.into_iter().map(Ok), |_, _| {})
    }

    #[test]
    fn contextual_switch_and_narrow_mask_are_independent() -> Result<()> {
        let mut raw = MAGIC.to_vec();
        raw.extend_from_slice(&4u32.to_le_bytes());
        record(&mut raw, 1, 0, 0);
        record(&mut raw, 1, 20, 30);
        record(&mut raw, -1, 20, 0);
        record(&mut raw, -1, 0, 30);
        let bank = SourceBank::from_bytes(&raw)?;
        let first = [candidate(8, true, 0, 0), candidate(11, false, 20, 0)];
        let second = [candidate(8, true, 0, 30), candidate(11, false, 20, 30)];
        let masked = bank.score(SourceArm::NarrowContextual, &second[1]);
        assert!(!masked.eligible);
        assert_eq!(masked.nearest_positive, Some(0));
        assert_eq!(masked.nearest_negative, Some(20));
        assert_eq!(masked.margin, Some(20));
        for rows in [first.clone(), second.clone()] {
            for arm in [SourceArm::WideLocal, SourceArm::NarrowLocal] {
                assert_eq!(
                    select(&bank, arm, rows.clone())?.outcome,
                    SourceOutcome::Nonpositive
                );
            }
        }
        assert_eq!(
            select(&bank, SourceArm::WideContextual, first.clone())?.source,
            Some(8)
        );
        assert_eq!(
            select(&bank, SourceArm::NarrowContextual, first)?.source,
            Some(8)
        );
        assert_eq!(
            select(&bank, SourceArm::WideContextual, second.clone())?.source,
            Some(11)
        );
        assert_eq!(
            select(&bank, SourceArm::NarrowContextual, second)?.outcome,
            SourceOutcome::Nonpositive
        );
        assert_eq!(
            select(
                &bank,
                SourceArm::WideContextual,
                [candidate(8, true, 0, 0), candidate(9, true, 0, 0)]
            )?
            .outcome,
            SourceOutcome::Tie,
        );
        assert_eq!(
            select(
                &bank,
                SourceArm::NarrowContextual,
                [candidate(11, false, 20, 30)]
            )?
            .outcome,
            SourceOutcome::Empty,
        );
        assert!(SourceBank::from_bytes(&raw[..raw.len() - 1]).is_err());
        raw[12] = 0;
        assert!(SourceBank::from_bytes(&raw).is_err());
        Ok(())
    }

    #[test]
    fn signed_distance_preserves_full_width() -> Result<()> {
        let mut raw = MAGIC.to_vec();
        raw.extend_from_slice(&2u32.to_le_bytes());
        record(&mut raw, 1, 0, i32::MIN);
        record(&mut raw, -1, 0, i32::MAX);
        let bank = SourceBank::from_bytes(&raw)?;
        let sample = candidate(8, true, 0, i32::MIN);
        let trace = bank.score(SourceArm::WideContextual, &sample);
        assert_eq!(trace.nearest_positive, Some(0));
        assert_eq!(trace.nearest_negative, Some(i64::from(u32::MAX)));
        assert_eq!(trace.margin, trace.nearest_negative);
        assert_eq!(bank.score(SourceArm::WideLocal, &sample).margin, Some(0));
        Ok(())
    }

    #[test]
    fn recurrence_expands_signed_orbit_words_in_cuda_block_order() {
        let expected = [0, 3, 2, 1].map(orbit_root_coords).concat();
        assert_eq!(expand_recurrence_code(0x1230).as_slice(), expected);
    }

    #[test]
    fn carrier_rejects_crossed_read_versions_and_malformed_recurrence_state() -> Result<()> {
        fn rejected(envelope: &Envelope, bank: &[u8], expected: &str) -> Result<()> {
            let header = serde_json::to_vec(envelope)?;
            let mut file = D11_MAGIC.to_vec();
            file.extend_from_slice(&(header.len() as u64).to_le_bytes());
            file.extend_from_slice(&header);
            file.resize(file.len().next_multiple_of(ALIGN), 0);
            file.extend_from_slice(bank);
            let error = IntegerDiscreteModel::from_bytes(&file)
                .err()
                .ok_or_else(|| invalid("invalid carrier loaded"))?;
            assert!(error.to_string().contains(expected), "{error}");
            Ok(())
        }
        let mut raw = MAGIC.to_vec();
        raw.extend_from_slice(&2u32.to_le_bytes());
        record(&mut raw, 1, 0, i32::MAX);
        record(&mut raw, -1, 0, i32::MIN);
        let parent = Sha256::digest([]).into();
        let train = [1; 32];
        let bank = Sha256::digest(&raw).into();
        let arm = SourceArm::WideRecurrence;
        let mut envelope = Envelope {
            schema: SCHEMA.to_owned(),
            arm,
            bank_bytes: raw.len(),
            parent_sha256: hex::encode(parent),
            train_sha256: hex::encode(train),
            bank_sha256: hex::encode(bank),
            policy_sha256: hex::encode(policy_digest(arm, parent, train, bank)),
        };
        rejected(&envelope, &raw, "schema and read arm disagree")?;
        envelope.schema = RECURRENCE_SCHEMA.to_owned();
        envelope.arm = SourceArm::WideContextual;
        rejected(&envelope, &raw, "schema and read arm disagree")?;
        envelope.arm = arm;
        envelope.train_sha256 = hex::encode([2; 32]);
        rejected(&envelope, &raw, "binding mismatch")?;
        envelope.train_sha256 = hex::encode(train);
        let mut changed = raw.clone();
        changed[13] ^= 1;
        rejected(&envelope, &changed, "binding mismatch")?;
        rejected(
            &envelope,
            &raw,
            "recurrence exemplar is outside the D11 orbit",
        )
    }
}
