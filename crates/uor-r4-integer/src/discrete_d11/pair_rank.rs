//! Signed differences of the existing within-assertion D26 source coordinates.
use super::{
    contextual_source::{SourceCandidate, SourceOutcome, SourceSelection, DIMENSIONS},
    read_role_digest, IntegerDiscreteModel, ALIGN, MAGIC as D11_MAGIC, MAX_HEADER,
};
use crate::{invalid, paired_role::RoleSpan, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SCHEMA: &str = "uor-r4.lut-discrete-pair-rank/1";
pub const CANDIDATE_SCHEMA: &str = "uor-r4.lut-discrete-pair-rank/2";
const MAGIC: &[u8; 8] = b"D35PAIR1";
const MAX_ROWS: usize = 8192;
const RECORD_BYTES: usize = DIMENSIONS * (4 + 8);
const MAX_BANK_BYTES: usize = 12 + MAX_ROWS * RECORD_BYTES;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Geometry {
    FactPrefix,
    CandidatePrefix,
}

impl Geometry {
    pub(super) fn schema(self) -> &'static str {
        match self {
            Self::FactPrefix => SCHEMA,
            Self::CandidatePrefix => CANDIDATE_SCHEMA,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    #[default]
    Forward,
    Reversed,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct AssertionOracle {
    pub query: RoleSpan,
    pub fact: RoleSpan,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Replay {
    pub orientation: Orientation,
    pub oracle: Option<AssertionOracle>,
    pub project: Option<Geometry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Difference {
    pub local: [i32; DIMENSIONS],
    pub contextual: [i64; DIMENSIONS],
}

impl Difference {
    pub fn between(candidate: &SourceCandidate, rival: &SourceCandidate) -> Self {
        Self {
            local: core::array::from_fn(|i| {
                i32::from(candidate.local[i]) - i32::from(rival.local[i])
            }),
            contextual: core::array::from_fn(|i| {
                i64::from(candidate.contextual[i]) - i64::from(rival.contextual[i])
            }),
        }
    }

    fn valid(&self) -> bool {
        self.local
            .iter()
            .all(|v| v.unsigned_abs() <= u32::from(u16::MAX))
            && self
                .contextual
                .iter()
                .all(|v| v.unsigned_abs() <= u64::from(u32::MAX))
    }

    fn distances(&self, row: &Self) -> (u64, u64) {
        // Each term is <= 2*u32::MAX in i64. The total across both blocks
        // is <= 32 * (2*u16::MAX + 2*u32::MAX), far below i64::MAX.
        self.local
            .iter()
            .map(|&x| i64::from(x))
            .chain(self.contextual)
            .zip(
                row.local
                    .iter()
                    .map(|&x| i64::from(x))
                    .chain(row.contextual),
            )
            .fold((0, 0), |(forward, reverse), (a, b)| {
                (forward + a.abs_diff(b), reverse + (a + b).unsigned_abs())
            })
    }
}

#[derive(Clone, Debug)]
pub struct Bank(Vec<Difference>);

impl Bank {
    pub fn from_bytes(raw: &[u8]) -> Result<Self> {
        if raw.len() < 12 || raw.len() > MAX_BANK_BYTES || &raw[..8] != MAGIC {
            return Err(invalid("invalid pair-rank bank magic or size"));
        }
        let count = u32::from_le_bytes(read_word(&raw[8..12])?) as usize;
        if !(1..=MAX_ROWS).contains(&count) || raw.len() != 12 + count * RECORD_BYTES {
            return Err(invalid("invalid pair-rank bank row count"));
        }
        let mut rows = Vec::with_capacity(count);
        for record in raw[12..].chunks_exact(RECORD_BYTES) {
            let mut row = Difference {
                local: [0; DIMENSIONS],
                contextual: [0; DIMENSIONS],
            };
            let (local, contextual) = record.split_at(DIMENSIONS * 4);
            for (word, value) in local.chunks_exact(4).zip(&mut row.local) {
                *value = i32::from_le_bytes(read_word(word)?);
            }
            for (word, value) in contextual.chunks_exact(8).zip(&mut row.contextual) {
                *value = i64::from_le_bytes(read_word(word)?);
            }
            if !row.valid() {
                return Err(invalid(
                    "pair-rank difference exceeds native coordinate bounds",
                ));
            }
            rows.push(row);
        }
        Ok(Self(rows))
    }

    pub fn encode(rows: &[Difference]) -> Result<Vec<u8>> {
        if !(1..=MAX_ROWS).contains(&rows.len()) || !rows.iter().all(Difference::valid) {
            return Err(invalid("invalid pair-rank differences"));
        }
        let mut raw = Vec::with_capacity(12 + rows.len() * RECORD_BYTES);
        raw.extend_from_slice(MAGIC);
        raw.extend_from_slice(&(rows.len() as u32).to_le_bytes());
        for row in rows {
            raw.extend(row.local.iter().flat_map(|value| value.to_le_bytes()));
            raw.extend(row.contextual.iter().flat_map(|value| value.to_le_bytes()));
        }
        Ok(raw)
    }

    fn nearest(&self, difference: &Difference) -> (u64, u64) {
        self.0
            .iter()
            .map(|row| difference.distances(row))
            .fold((u64::MAX, u64::MAX), |(forward, reverse), (f, r)| {
                (forward.min(f), reverse.min(r))
            })
    }

    pub(super) fn rank<'a>(
        &self,
        fact: RoleSpan,
        baseline: SourceSelection,
        candidates: impl IntoIterator<Item = &'a SourceCandidate>,
        orientation: Orientation,
    ) -> AssertionRank {
        let eligible: Vec<_> = candidates
            .into_iter()
            .filter(|candidate| baseline.arm.includes(candidate))
            .collect();
        let eligible_sources = eligible.iter().map(|c| c.span.start as u32).collect();
        let mut selection = baseline;
        let mut pairs = Vec::new();
        if eligible.len() > 1 {
            selection.source = None;
            let mut any_tie = false;
            for (i, candidate) in eligible.iter().enumerate() {
                let mut strict = true;
                for (j, rival) in eligible.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let (nearest_forward, nearest_reverse) =
                        self.nearest(&Difference::between(candidate, rival));
                    let forward_margin = nearest_reverse as i64 - nearest_forward as i64;
                    let margin = match orientation {
                        Orientation::Forward => forward_margin,
                        Orientation::Reversed => -forward_margin,
                    };
                    any_tie |= margin == 0;
                    strict &= margin > 0;
                    pairs.push(PairScore {
                        candidate: candidate.span.start as u32,
                        rival: rival.span.start as u32,
                        nearest_forward,
                        nearest_reverse,
                        margin,
                    });
                }
                if strict {
                    // Antisymmetry makes two strict winners impossible.
                    selection.source = Some(candidate.span.start as u32);
                }
            }
            selection.outcome = match selection.source {
                Some(_) => SourceOutcome::Pick,
                None if any_tie => SourceOutcome::Tie,
                None => SourceOutcome::Nonpositive,
            };
        }
        AssertionRank {
            fact_span: [fact.start, fact.end],
            selection,
            eligible_sources,
            orientation,
            pairs,
        }
    }
}

fn read_word<const N: usize>(word: &[u8]) -> Result<[u8; N]> {
    word.try_into()
        .map_err(|_| invalid("truncated pair-rank word"))
}

#[derive(Clone, Debug, Serialize)]
pub struct PairScore {
    pub candidate: u32,
    pub rival: u32,
    pub nearest_forward: u64,
    pub nearest_reverse: u64,
    pub margin: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssertionRank {
    pub fact_span: [usize; 2],
    pub selection: SourceSelection,
    pub eligible_sources: Vec<u32>,
    pub orientation: Orientation,
    pub pairs: Vec<PairScore>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssertionCandidates {
    pub fact_span: [usize; 2],
    pub geometry: Geometry,
    pub candidates: Vec<SourceCandidate>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Bindings {
    pub geometry: Geometry,
    pub parent_sha256: String,
    pub train_sha256: String,
    pub bank_sha256: String,
}

#[derive(Clone, Debug)]
pub(super) struct Policy {
    pub bank: Bank,
    pub bindings: Bindings,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: String,
    geometry: Geometry,
    bank_bytes: usize,
    parent_sha256: String,
    train_sha256: String,
    bank_sha256: String,
    policy_sha256: String,
}

fn policy_digest(
    geometry: Geometry,
    parent: [u8; 32],
    train: [u8; 32],
    bank: [u8; 32],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"uor-r4.pair-rank/1;full-detached;D26-wide-recurrence;D35PAIR1;candidate-minus-rival;ordered-L1-nearest;strict-all-rivals;frozen-singleton");
    hash.update(geometry.schema().as_bytes());
    hash.update(parent);
    hash.update(train);
    hash.update(bank);
    hash.finalize().into()
}

pub fn pack_artifact(
    parent: &[u8],
    bank: &[u8],
    train_digest: [u8; 32],
    geometry: Geometry,
) -> Result<Vec<u8>> {
    let parent_digest = Sha256::digest(parent).into();
    let bank_digest = Sha256::digest(bank).into();
    let header = serde_json::to_vec(&Envelope {
        schema: geometry.schema().to_owned(),
        geometry,
        bank_bytes: bank.len(),
        parent_sha256: hex::encode(parent_digest),
        train_sha256: hex::encode(train_digest),
        bank_sha256: hex::encode(bank_digest),
        policy_sha256: hex::encode(policy_digest(
            geometry,
            parent_digest,
            train_digest,
            bank_digest,
        )),
    })?;
    let mut bytes = D11_MAGIC.to_vec();
    bytes.extend_from_slice(&(header.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&header);
    bytes.resize(bytes.len().next_multiple_of(ALIGN), 0);
    bytes.extend_from_slice(bank);
    bytes.extend_from_slice(parent);
    IntegerDiscreteModel::from_bytes(&bytes)?;
    Ok(bytes)
}

pub(super) fn read(
    bytes: &[u8],
    header: &[u8],
    end_header: usize,
    allow_role: bool,
) -> Result<IntegerDiscreteModel> {
    if !allow_role {
        return Err(invalid("nested pair-rank carrier"));
    }
    let envelope: Envelope = serde_json::from_slice(header)?;
    if envelope.geometry.schema() != envelope.schema {
        return Err(invalid("pair-rank schema and geometry disagree"));
    }
    let start = end_header.next_multiple_of(ALIGN);
    let padding = bytes
        .get(end_header..start)
        .filter(|padding| padding.iter().all(|&word| word == 0))
        .ok_or_else(|| invalid("invalid pair-rank alignment padding"))?;
    let payload = &bytes[end_header + padding.len()..];
    let (bank, parent) = payload
        .split_at_checked(envelope.bank_bytes)
        .filter(|(bank, _)| bank.len() <= MAX_BANK_BYTES)
        .ok_or_else(|| invalid("invalid pair-rank bank bounds"))?;
    let parent_digest = read_role_digest(&envelope.parent_sha256)?;
    let train_digest = read_role_digest(&envelope.train_sha256)?;
    let bank_digest = read_role_digest(&envelope.bank_sha256)?;
    if Sha256::digest(parent).as_slice() != parent_digest
        || Sha256::digest(bank).as_slice() != bank_digest
        || policy_digest(envelope.geometry, parent_digest, train_digest, bank_digest)
            != read_role_digest(&envelope.policy_sha256)?
    {
        return Err(invalid("pair-rank carrier binding mismatch"));
    }
    // Recognize the only legal parent before opening it to bound nesting.
    let prefix = parent
        .get(..16)
        .filter(|prefix| &prefix[..8] == D11_MAGIC)
        .ok_or_else(|| invalid("missing pair-rank D26 parent"))?;
    let length = u64::from_le_bytes(read_word(&prefix[8..])?);
    if length > MAX_HEADER {
        return Err(invalid("pair-rank parent header length out of bounds"));
    }
    let header = parent
        .get(16..16 + length as usize)
        .ok_or_else(|| invalid("truncated pair-rank parent header"))?;
    #[derive(Deserialize)]
    struct Schema {
        schema: String,
    }
    if serde_json::from_slice::<Schema>(header)?.schema
        != super::contextual_source::RECURRENCE_SCHEMA
    {
        return Err(invalid(
            "pair-rank requires the frozen D26 recurrence parent",
        ));
    }
    let mut model = IntegerDiscreteModel::from_bytes(parent)?;
    let bank = Bank::from_bytes(bank)?;
    if envelope.geometry == Geometry::FactPrefix
        && bank
            .0
            .iter()
            .any(|row| row.contextual.iter().any(|&x| x != 0))
    {
        return Err(invalid(
            "fact-prefix pair bank must have zero contextual differences",
        ));
    }
    model.pair_rank = Some(Policy {
        bank,
        bindings: Bindings {
            geometry: envelope.geometry,
            parent_sha256: envelope.parent_sha256,
            train_sha256: envelope.train_sha256,
            bank_sha256: envelope.bank_sha256,
        },
    });
    model.artifact_sha256 = hex::encode(Sha256::digest(bytes));
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discrete_d11::contextual_source::SourceArm;

    fn candidate(source: u32, local: i16, contextual: i32) -> SourceCandidate {
        SourceCandidate {
            span: RoleSpan {
                start: source as usize,
                end: source as usize + 1,
            },
            narrow: false,
            local: [local; DIMENSIONS],
            contextual: [contextual; DIMENSIONS],
            completed_source_range: [source, 20],
        }
    }

    #[test]
    fn signed_distance_preserves_full_difference_width() -> Result<()> {
        let low = candidate(3, i16::MIN, i32::MIN);
        let high = candidate(4, i16::MAX, i32::MAX);
        let forward = Difference::between(&high, &low);
        let reverse = Difference::between(&low, &high);
        let encoded = Bank::encode(core::slice::from_ref(&forward))?;
        let bank = Bank::from_bytes(&encoded)?;
        let bound = DIMENSIONS as u64 * (2 * u64::from(u16::MAX) + 2 * u64::from(u32::MAX));
        assert_eq!(bank.0, [forward.clone()]);
        assert_eq!(bank.nearest(&forward), (0, bound));
        assert_eq!(bank.nearest(&reverse), (bound, 0));
        assert!(Bank::from_bytes(&encoded[..encoded.len() - 1]).is_err());
        Ok(())
    }

    #[test]
    fn rank_is_antisymmetric_with_exact_frozen_singleton_fallback() -> Result<()> {
        let fact = RoleSpan { start: 2, end: 3 };
        let candidates = [
            candidate(5, 1, 91),
            candidate(7, 2, 91),
            candidate(9, 3, 91),
        ];
        let direction = Difference::between(&candidates[2], &candidates[0]);
        assert_eq!(direction.contextual, [0; DIMENSIONS]);
        let bank = Bank::from_bytes(&Bank::encode(&[direction])?)?;
        let baseline = SourceSelection {
            arm: SourceArm::WideRecurrence,
            outcome: SourceOutcome::Nonpositive,
            source: None,
        };
        let forward = bank.rank(fact, baseline, &candidates, Orientation::Forward);
        let reverse = bank.rank(fact, baseline, &candidates, Orientation::Reversed);
        assert_eq!(forward.selection.source, Some(9));
        assert_eq!(reverse.selection.source, Some(5));
        assert_eq!(forward.pairs.len(), 6);
        for (f, r) in forward.pairs.iter().zip(&reverse.pairs) {
            assert_eq!(f.nearest_forward, r.nearest_forward);
            assert_eq!(f.nearest_reverse, r.nearest_reverse);
            assert_eq!(f.margin, -r.margin);
        }
        assert_eq!(
            bank.rank(fact, baseline, &candidates[..1], Orientation::Forward)
                .selection,
            baseline,
        );
        let picked = SourceSelection {
            outcome: SourceOutcome::Pick,
            source: Some(5),
            ..baseline
        };
        assert_eq!(
            bank.rank(fact, picked, &candidates[..1], Orientation::Reversed)
                .selection,
            picked,
        );
        let tied = [candidate(5, 1, 91), candidate(7, 1, 91)];
        assert_eq!(
            bank.rank(fact, picked, &tied, Orientation::Forward)
                .selection
                .outcome,
            SourceOutcome::Tie
        );
        Ok(())
    }
}
