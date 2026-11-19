//! Bounded integer onset scoring from the same native bank used by CUDA.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    assertion_value::AssertionValue,
    discrete_d11::spiralcore_orbit_roots,
    invalid,
    span_selector::{CONT, START},
    Result,
};

const RAW: u32 = 0x31425355;
const SEALED: u32 = 0x32425355;
const CAP: usize = 256;
const RECORD: usize = 8;
const BINDINGS: usize = 208;
pub const BANK_BYTES: usize = BINDINGS + 16 + CAP * RECORD * 4;
const NIL: u32 = u32::MAX;
const NO_RADIUS: u32 = 65535;
pub const CANDIDATE_WORDS: usize = 35;
pub const DECISION_WORDS: usize = 68;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(u32)]
pub enum OnsetMode {
    Ordered = 0,
    Neutral = 1,
    Position = 2,
    Ranked = 3,
    Veto = 4,
    Both = 5,
    /// External supplied-query evaluation only. Not a serialized onset profile.
    #[serde(skip)]
    EarlierConnectorV1 = 6,
}

impl OnsetMode {
    pub const fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::Ordered),
            1 => Some(Self::Neutral),
            2 => Some(Self::Position),
            3 => Some(Self::Ranked),
            4 => Some(Self::Veto),
            5 => Some(Self::Both),
            6 => Some(Self::EarlierConnectorV1),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Ordered => "ordered",
            Self::Neutral => "neutral",
            Self::Position => "position",
            Self::Ranked => "ranked",
            Self::Veto => "veto",
            Self::Both => "both",
            Self::EarlierConnectorV1 => "earlier-connector-v1",
        }
    }

    /// New policies select over the same losslessly loaded ordered bank.
    pub const fn fitted_mode(self) -> Self {
        match self {
            Self::Ranked | Self::Veto | Self::Both | Self::EarlierConnectorV1 => Self::Ordered,
            _ => self,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OnsetProfile {
    pub mode: OnsetMode,
    pub train_sha256: String,
    pub labels_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct OnsetDecision {
    /// 0 selected, 1 no candidate, 2 unseen, 3 tie, 4 nonpositive.
    pub status: u32,
    pub source: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct Bank {
    words: Box<[u32]>,
    mode: OnsetMode,
}
impl Bank {
    pub fn from_profile(
        bytes: &[u8],
        base: [u8; 32],
        tokenizer: [u8; 32],
        sites: [u8; 32],
        profile: &OnsetProfile,
    ) -> Result<Self> {
        if bytes.len() != BANK_BYTES
            || word(bytes, 0) != SEALED
            || word(bytes, 1) != 6
            || word(bytes, 2) as usize != BANK_BYTES - BINDINGS
            || word(bytes, 3) != 0
        {
            return Err(invalid("invalid onset bank envelope"));
        }
        let expected = [
            base,
            tokenizer,
            sites,
            digest_field(&profile.train_sha256)?,
            digest_field(&profile.labels_sha256)?,
            Sha256::digest(&bytes[BINDINGS..]).into(),
        ];
        if !bytes[16..BINDINGS]
            .chunks_exact(32)
            .zip(&expected)
            .all(|(actual, expected)| actual == expected)
        {
            return Err(invalid(
                "onset model, site, profile or payload binding mismatch",
            ));
        }
        let raw = &bytes[BINDINGS..];
        let words: Box<[u32]> = raw
            .chunks_exact(4)
            .map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
            .collect();
        if words[0] != RAW
            || words[1] != profile.mode.fitted_mode() as u32
            || !(1..=CAP as u32).contains(&words[2])
            || words[3] != CAP as u32
        {
            return Err(invalid("invalid onset mode or capacity"));
        }
        let mode = words[1];
        let used = words[2] as usize;
        let mut classes = [false; 2];
        for (i, r) in words[4..].chunks_exact(RECORD).enumerate() {
            if i >= used {
                if r.iter().any(|&word| word != 0) {
                    return Err(invalid("nonzero onset bank padding"));
                }
                continue;
            }
            let label = r[0] >> 16;
            let depth = (r[0] & 255) as usize;
            let length = ((r[0] >> 8) & 255) as usize;
            if label > 1
                || !(1..=3).contains(&depth)
                || !(1..=7).contains(&length)
                || (r[2] > 1280 && r[2] != NO_RADIUS)
                || (mode == 2 && r[2] != NO_RADIUS)
            {
                return Err(invalid("invalid onset class, shape or radius"));
            }
            classes[label as usize] = true;
            let slots = depth + length;
            if (r[1] >> (2 * slots)) != 0
                || (mode != 2 && (r[1] >> (2 * depth)) & 3 != START)
                || (mode == 2 && (r[1] != 0 || r[3..].iter().any(|&x| x != 0)))
            {
                return Err(invalid("invalid onset shape or position vocabulary"));
            }
            let roots: [u16; 10] = core::array::from_fn(|j| root_at(&r[3..], j));
            for (j, &root) in roots.iter().enumerate() {
                let class = (r[1] >> (2 * j)) & 3;
                if (j < slots && mode != 2 && (class > CONT || (j > depth && class != CONT)))
                    || (j >= slots && root != 0)
                {
                    return Err(invalid("invalid onset class or root padding"));
                }
            }
            if mode == 1 {
                let mut ordered = roots;
                canonical(&mut ordered[..slots], depth, r[1]);
                if ordered != roots {
                    return Err(invalid("noncanonical neutral onset geometry"));
                }
            }
            for previous in words[4..4 + i * RECORD].chunks_exact(RECORD) {
                if previous[0] & 65535 == r[0] & 65535
                    && previous[1] == r[1]
                    && (previous[2] != r[2]
                        || (mode != 2 && previous[0] >> 16 != label && previous[3..] == r[3..]))
                {
                    return Err(invalid("conflicting onset radii or geometric classes"));
                }
            }
        }
        if classes != [true, true] {
            return Err(invalid("onset bank lacks one class"));
        }
        Ok(Self {
            words,
            mode: profile.mode,
        })
    }

    pub fn mode(&self) -> OnsetMode {
        self.mode
    }

    pub(crate) fn choose(
        &self,
        ids: &[u32],
        roots: &[u32],
        classes: &[u32],
        last: usize,
        query: usize,
        frozen: AssertionValue,
    ) -> Result<OnsetDecision> {
        if self.mode == OnsetMode::EarlierConnectorV1 {
            return Err(invalid(
                "earlier connector policy requires an external supplied-query row",
            ));
        }
        let Some(fence) = frozen.fence.map(|at| at as usize) else {
            return Ok(OnsetDecision {
                status: 1,
                source: None,
            });
        };
        if last >= fence
            || fence >= ids.len()
            || (frozen.value.eligible_mask | frozen.value.word_start_mask) > 7
        {
            return Err(invalid("invalid frozen onset fence or mask"));
        }
        let mask = frozen.value.eligible_mask & frozen.value.word_start_mask;
        let mut row = [0u32; CANDIDATE_WORDS];
        row[2..8].copy_from_slice(&[
            ids.len() as u32,
            last as u32,
            query as u32,
            fence as u32,
            frozen.value.eligible_mask,
            frozen.value.word_start_mask,
        ]);
        for offset in 0..3 {
            let candidate = &mut row[8 + offset * 9..17 + offset * 9];
            candidate[0] = NIL;
            if mask & (1 << offset) == 0 {
                continue;
            }
            let source = last + offset + 2;
            if source >= fence || classes[ids[source] as usize] != START {
                return Err(invalid("legacy onset mask points outside a complete word"));
            }
            let end = (source + 1..fence)
                .find(|&at| classes[ids[at] as usize] != CONT)
                .unwrap_or(fence);
            candidate[0] = source as u32;
            candidate[1] = end as u32;
            if fence - source >= 8 {
                continue;
            }
            candidate[2] = 1;
            for (i, &id) in ids[last + 1..end].iter().enumerate() {
                candidate[3] |= classes[id as usize] << (2 * i);
                candidate[4 + i / 2] |= roots[id as usize] << (16 * (i % 2));
            }
        }
        let t = self.score_legacy_row(&row)?;
        Ok(OnsetDecision {
            status: t[2],
            source: (t[3] != NIL).then_some(t[3]),
        })
    }

    fn records(&self) -> impl Iterator<Item = &[u32]> {
        self.words[4..].chunks_exact(8).take(self.words[2] as usize)
    }

    /// Serve the canonical blind packet row also consumed by the CUDA backend.
    /// No fit annotations or output IDs enter this path.
    pub fn score_legacy_row(&self, row: &[u32; CANDIDATE_WORDS]) -> Result<[u32; DECISION_WORDS]> {
        use crate::paired_role::ROLE_CONTEXT;
        let [length, fact, query, fence, eligible, start] =
            [row[2], row[3], row[4], row[5], row[6], row[7]];
        let mask = eligible & start;
        if !(1..=ROLE_CONTEXT as u32).contains(&length)
            || (eligible | start) > 7
            || (fence != NIL
                && !(fact > 0 && fact < fence && fence < query && query < length && row[1] != NIL))
            || (fence == NIL && (eligible | start) != 0)
        {
            return Err(invalid("invalid original onset packet bounds"));
        }
        let slots = [&row[8..17], &row[17..26], &row[26..35]];
        for (offset, record) in slots.into_iter().enumerate() {
            if mask & (1 << offset) == 0 {
                if record != [NIL, 0, 0, 0, 0, 0, 0, 0, 0] {
                    return Err(invalid("nonempty absent onset packet slot"));
                }
                continue;
            }
            let source = fact + 2 + offset as u32;
            if record[0] != source
                || !(source < record[1] && record[1] <= fence)
                || record[2] != u32::from(fence - source < 8)
            {
                return Err(invalid("invalid onset packet source or cap word"));
            }
            if record[2] == 0 {
                if record[3..].iter().any(|&word| word != 0) {
                    return Err(invalid("unreachable onset carries geometry"));
                }
                continue;
            }
            let depth = offset + 1;
            let used = depth + (record[1] - source) as usize;
            if used > 10 || record[3] >> (2 * used) != 0 || (record[3] >> (2 * depth)) & 3 != START
            {
                return Err(invalid("invalid onset packet shape"));
            }
            for i in 0..10 {
                let class = (record[3] >> (2 * i)) & 3;
                if (i < used && (class > CONT || i > depth && class != CONT))
                    || (i >= used && root_at(&record[4..], i) != 0)
                {
                    return Err(invalid("invalid onset packet word classes or padding"));
                }
            }
            for other in &slots[..offset] {
                if other[2] != 1 {
                    continue;
                }
                let shared = (record[1].min(other[1]) - fact - 1) as usize;
                for i in 0..shared {
                    if (record[3] >> (2 * i)) & 3 != (other[3] >> (2 * i)) & 3
                        || root_at(&record[4..], i) != root_at(&other[4..], i)
                    {
                        return Err(invalid("overlapping onset packet slots disagree"));
                    }
                }
            }
        }
        Ok(self.score_row(row))
    }

    fn score_row(&self, row: &[u32; CANDIDATE_WORDS]) -> [u32; DECISION_WORDS] {
        let mut trace = [NIL; DECISION_WORDS];
        trace[..2].copy_from_slice(&row[..2]);
        let mut margins = [Margin::ABSENT; 3];
        let mut unseen = false;
        let mut outside_radius = false;
        let sum_denominator = matches!(self.mode, OnsetMode::Ranked | OnsetMode::Both);
        let skip_outside = matches!(self.mode, OnsetMode::Veto | OnsetMode::Both);
        for (i, margin) in margins.iter_mut().enumerate() {
            let c = &row[8 + i * 9..17 + i * 9];
            if c[0] == NIL {
                continue;
            }
            let out = &mut trace[8 + 20 * i..28 + 20 * i];
            out[..3].copy_from_slice(&[c[0], c[1], 1]);
            if c[2] == 0 {
                continue;
            }
            out[2] = 2;
            let depth = i + 1;
            let word_len = if self.mode() == OnsetMode::Position {
                row[5] - c[0]
            } else {
                c[1] - c[0]
            };
            let slots = depth + word_len as usize;
            let shape = depth as u32 | (word_len << 8);
            let sig = if self.mode() == OnsetMode::Position {
                0
            } else {
                c[3]
            };
            let mut roots = [0u16; 10];
            if self.mode() != OnsetMode::Position {
                for (slot, to) in roots[..slots].iter_mut().enumerate() {
                    *to = root_at(&c[4..], slot);
                }
                if self.mode() == OnsetMode::Neutral {
                    canonical(&mut roots[..slots], depth, sig);
                }
            }
            out[3..5].copy_from_slice(&[shape, sig]);
            for (to, &value) in out[10..10 + slots].iter_mut().zip(&roots) {
                *to = u32::from(value);
            }
            let hits = self
                .records()
                .filter(|record| record[0] & 65535 == shape && record[1] == sig);
            let score = if self.mode() == OnsetMode::Position {
                let mut votes = [0u32; 2];
                for record in hits {
                    votes[(record[0] >> 16) as usize] += 1;
                }
                out[5..8].copy_from_slice(&[votes[0], votes[1], NO_RADIUS]);
                (votes != [0, 0]).then_some((votes[0] as i32 - votes[1] as i32, 1))
            } else {
                let mut closest = [NIL; 2];
                let mut radius = NO_RADIUS;
                for record in hits {
                    let d = roots[..slots]
                        .iter()
                        .enumerate()
                        .map(|(j, &r)| l1(r, root_at(&record[3..], j)))
                        .sum();
                    let index = (record[0] >> 16) as usize;
                    closest[index] = closest[index].min(d);
                    radius = record[2];
                }
                out[5..8].copy_from_slice(&[closest[0], closest[1], radius]);
                if closest.contains(&NIL) || radius == NO_RADIUS {
                    None
                } else if closest[0] > radius && skip_outside {
                    // The rival is known but lacks positive admission. It is
                    // excluded from the rank without masking a supported source.
                    out[2] = 4;
                    outside_radius = true;
                    continue;
                } else if closest[0] > radius {
                    None
                } else {
                    let denominator = if sum_denominator {
                        (closest[0] + closest[1]).max(1) as i32
                    } else {
                        slots as i32
                    };
                    Some((closest[1] as i32 - closest[0] as i32, denominator))
                }
            };
            if let Some((num, den)) = score {
                out[2] = 3;
                out[8..10].copy_from_slice(&[num as u32, den as u32]);
                *margin = Margin {
                    num,
                    den,
                    source: c[0],
                };
            } else {
                unseen = true;
            }
        }
        margins.sort_by(|a, b| b.compare(*a));
        let [best, next, _] = margins;
        for (score, dest) in [best, next]
            .into_iter()
            .zip(trace[4..8].chunks_exact_mut(2))
        {
            if score.den != 0 {
                dest.copy_from_slice(&[score.num as u32, score.den as u32]);
            }
        }
        if self.mode == OnsetMode::EarlierConnectorV1 && earlier_connector_run(&trace) {
            trace[10] = 4;
            unseen = false;
        }
        trace[2] = if unseen || (outside_radius && best.den == 0) {
            2
        } else if best.den == 0 {
            1
        } else if best.num <= 0 {
            4
        } else if next.den != 0 && best.compare(next).is_eq() {
            3
        } else {
            0
        };
        if trace[2] == 0 {
            trace[3] = best.source;
        }
        trace
    }
}

// The admitted middle word must independently look negative. With a gap,
// missing exemplar, or competing positive, the ordered veto still applies.
fn earlier_connector_run(trace: &[u32; DECISION_WORDS]) -> bool {
    let first = &trace[8..28];
    let middle = &trace[28..48];
    let last = &trace[48..68];
    first[2] == 2
        && first[5] != NIL
        && first[6] != NIL
        && first[7] != NO_RADIUS
        && first[5] > first[7]
        && first[6] < first[5]
        && middle[2] == 3
        && (middle[8] as i32) < 0
        && last[2] == 3
        && (last[8] as i32) > 0
        && first[1] == middle[0]
        && middle[1] == last[0]
}

#[derive(Clone, Copy)]
struct Margin {
    num: i32,
    den: i32,
    source: u32,
}
impl Margin {
    const ABSENT: Self = Self {
        num: 0,
        den: 0,
        source: NIL,
    };
    fn compare(self, other: Self) -> std::cmp::Ordering {
        match (self.den == 0, other.den == 0) {
            (false, false) => (self.num * other.den).cmp(&(other.num * self.den)),
            (a, b) => b.cmp(&a),
        }
    }
}

fn root_at(data: &[u32], slot: usize) -> u16 {
    (data[slot / 2] >> (16 * (slot % 2))) as u16
}

fn l1(a: u16, b: u16) -> u32 {
    let orbit = spiralcore_orbit_roots();
    (0..4)
        .map(|i| {
            let x = &orbit[((a >> (4 * i)) & 15) as usize];
            let y = &orbit[((b >> (4 * i)) & 15) as usize];
            x.iter()
                .zip(y)
                .map(|(&x, &y)| u32::from(x.abs_diff(y)))
                .sum::<u32>()
        })
        .sum()
}

fn canonical(geometry: &mut [u16], depth: usize, signature: u32) {
    for range in [0..depth, depth..geometry.len()] {
        for class in [1, 2] {
            let mut values = [0u16; 10];
            let mut count = 0;
            let positions = range
                .clone()
                .filter(|&i| (signature >> (2 * i)) & 3 == class);
            for i in positions.clone() {
                values[count] = geometry[i];
                count += 1;
            }
            values[..count].sort_unstable();
            for (i, word) in positions.zip(values) {
                geometry[i] = word;
            }
        }
    }
}

fn word(bytes: &[u8], i: usize) -> u32 {
    let start = i * 4;
    u32::from_le_bytes([
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
    ])
}

fn digest_field(value: &str) -> Result<[u8; 32]> {
    let mut hash = [0; 32];
    hex::decode_to_slice(value, &mut hash).map_err(|_| invalid("invalid onset profile digest"))?;
    if hex::encode(hash) != value {
        return Err(invalid("noncanonical onset profile digest"));
    }
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_bank_binding_and_original_onset_packet() {
        let profile = OnsetProfile {
            mode: OnsetMode::Ordered,
            train_sha256: "03".repeat(32),
            labels_sha256: "04".repeat(32),
        };
        let mut payload: Vec<u8> = [
            RAW, 0, 2, CAP as u32, 0x0101, 5, 0, 0, 0, 0, 0, 0, 0x10101, 5, 0, 2, 0, 0, 0, 0,
        ]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect();
        payload.resize(BANK_BYTES - BINDINGS, 0);
        let mut bytes: Vec<u8> = [SEALED, 6, payload.len() as u32, 0]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect();
        for digest in [
            [0; 32],
            [1; 32],
            [2; 32],
            [3; 32],
            [4; 32],
            Sha256::digest(&payload).into(),
        ] {
            bytes.extend(digest);
        }
        bytes.extend(payload);
        let bank = Bank::from_profile(&bytes, [0; 32], [1; 32], [2; 32], &profile).unwrap();
        let mut row = [0u32; CANDIDATE_WORDS];
        row[..8].copy_from_slice(&[0, 0, 14, 2, 12, 9, 1, 1]);
        row[8..17].copy_from_slice(&[4, 5, 1, 5, 0, 0, 0, 0, 0]);
        row[17] = NIL;
        row[26] = NIL;
        let decision = bank.score_legacy_row(&row).unwrap();
        assert_eq!(&decision[2..6], &[0, 4, 4, 2]);
        row[8] = 7;
        assert!(bank.score_legacy_row(&row).is_err());
        assert!(Bank::from_profile(&bytes, [0; 32], [1; 32], [5; 32], &profile).is_err());
        bytes[BANK_BYTES - 1] ^= 1;
        assert!(Bank::from_profile(&bytes, [0; 32], [1; 32], [2; 32], &profile).is_err());
    }
}
