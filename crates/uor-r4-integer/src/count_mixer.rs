//! Integer, full-support order-one count mixture. IDs 1..4095 are legal targets.
use crate::discrete_d11::{IntegerDiscreteModel, DISCRETE_TRUNK_SCHEMA, MAGIC};
use crate::{invalid, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SCHEMA: &str = "uor-r4.lut-discrete-count-mixer/1";
pub const VOCAB: usize = 4096;
const Q24: u64 = 1 << 24;
const HEADER: usize = (VOCAB + 1) * 4;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Envelope {
    pub schema: String,
    pub base_sha256: String,
    pub tokenizer_sha256: String,
    pub state_bytes: usize,
    pub state_sha256: String,
}

#[derive(Clone, Debug)]
pub struct CountMixer {
    weight: u32,
    unigram: Vec<u32>,
    offsets: Vec<usize>,
    conditional: Vec<(u16, u32)>,
    words: Vec<u8>,
}

fn read_count(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes[..4].try_into().expect("validated count word"))
}

fn mass_step(cumulative: &mut u64, count: u64, total: u64, available: u64) -> u32 {
    let low = *cumulative * available / total;
    *cumulative += count;
    ((*cumulative * available / total) - low) as u32
}

impl CountMixer {
    /// Canonical LE stream: conditional weight, V unigram counts, sorted (previous, target, count).
    pub fn from_words(words: &[u8]) -> Result<Self> {
        if words.len() < HEADER || (words.len() - HEADER) % 8 != 0 {
            return Err(invalid("partial count mixer word stream"));
        }
        let weight = read_count(words);
        if !(1..=255).contains(&weight) || read_count(&words[4..]) != 0 {
            return Err(invalid("invalid conditional weight or BOS target count"));
        }
        let counts: Vec<_> = words[4..HEADER].chunks_exact(4).map(read_count).collect();
        let total = counts.iter().map(|&c| u64::from(c)).sum::<u64>();
        if total == 0 || total > u64::from(u32::MAX) {
            return Err(invalid("invalid unigram count total"));
        }
        let mut cumulative = 0;
        let prior = (VOCAB - 1) as u64;
        let mut unigram = vec![0];
        for &count in &counts[1..] {
            unigram.push(
                1 + mass_step(
                    &mut cumulative,
                    u64::from(count) + 1,
                    total + prior,
                    Q24 - prior,
                ),
            );
        }
        let mut offsets = vec![0usize; VOCAB + 1];
        let mut totals = vec![0u64; VOCAB];
        let mut conditional = Vec::with_capacity((words.len() - HEADER) / 8);
        let mut last = None;
        for row in words[HEADER..].chunks_exact(8) {
            let p = usize::from(u16::from_le_bytes([row[0], row[1]]));
            let t = u16::from_le_bytes([row[2], row[3]]);
            let count = read_count(&row[4..]);
            if p >= VOCAB
                || t == 0
                || usize::from(t) >= VOCAB
                || count == 0
                || last.is_some_and(|v| v >= (p, t))
            {
                return Err(invalid("invalid or unordered conditional count row"));
            }
            last = Some((p, t));
            offsets[p + 1] += 1;
            totals[p] += u64::from(count);
            if totals[p] > u64::from(u32::MAX) {
                return Err(invalid("conditional count total overflows"));
            }
            conditional.push((t, count));
        }
        for p in 0..VOCAB {
            offsets[p + 1] += offsets[p];
            let mut cumulative = 0;
            for (_, value) in &mut conditional[offsets[p]..offsets[p + 1]] {
                *value = mass_step(&mut cumulative, u64::from(*value), totals[p], Q24);
            }
        }
        Ok(Self {
            weight,
            unigram,
            offsets,
            conditional,
            words: words.to_vec(),
        })
    }

    pub fn words(&self) -> &[u8] {
        &self.words
    }
    pub fn conditional_weight(&self) -> u32 {
        self.weight
    }

    /// The two normalized Q24 expert masses. A missing context reads the unigram twice.
    pub fn expert_masses(&self, previous: u16, target: u16) -> (u32, u32) {
        let uni = self.unigram.get(usize::from(target)).copied().unwrap_or(0);
        let p = usize::from(previous);
        if p >= VOCAB || self.offsets[p] == self.offsets[p + 1] {
            return (uni, uni);
        }
        let row = &self.conditional[self.offsets[p]..self.offsets[p + 1]];
        let conditional = row
            .binary_search_by_key(&target, |r| r.0)
            .map_or(0, |i| row[i].1);
        (uni, conditional)
    }

    pub fn mass_q32(&self, previous: u16, target: u16) -> u64 {
        let (uni, conditional) = self.expert_masses(previous, target);
        u64::from(256 - self.weight) * u64::from(uni)
            + u64::from(self.weight) * u64::from(conditional)
    }

    /// Eight highest-mass original IDs, lowest ID first on equal mass.
    pub fn top8(&self, previous: u16) -> [u16; 8] {
        let mut ranked = [(0u16, 0u64); 8];
        for t in 1..VOCAB as u16 {
            let mass = self.mass_q32(previous, t);
            if mass > ranked[7].1 {
                let at = ranked.partition_point(|&(_, old)| old >= mass);
                ranked.copy_within(at..7, at + 1);
                ranked[at] = (t, mass);
            }
        }
        ranked.map(|pair| pair.0)
    }

    pub fn top1(&self, previous: u16) -> u16 {
        self.top8(previous)[0]
    }
}

/// Bind the canonical count state and a validated D11 trunk in one ordinary-loadable file.
pub fn pack(base: &[u8], words: &[u8]) -> Result<Vec<u8>> {
    let _ = CountMixer::from_words(words)?;
    let model = IntegerDiscreteModel::from_bytes(base)?;
    if model.artifact_schema() != DISCRETE_TRUNK_SCHEMA || model.shape().vocabulary != VOCAB as u32
    {
        return Err(invalid("expected V4096 base trunk"));
    }
    let header = serde_json::to_vec(&Envelope {
        schema: SCHEMA.to_string(),
        base_sha256: model.artifact_sha256().to_string(),
        tokenizer_sha256: model
            .tokenizer_sha256()
            .ok_or_else(|| invalid("missing tokenizer identity"))?
            .to_string(),
        state_bytes: words.len(),
        state_sha256: hex::encode(Sha256::digest(words)),
    })?;
    let mut out = MAGIC.to_vec();
    out.extend_from_slice(&(header.len() as u64).to_le_bytes());
    out.extend(header);
    out.resize(out.len().next_multiple_of(64), 0);
    out.extend_from_slice(words);
    out.extend_from_slice(base);
    Ok(out)
}
