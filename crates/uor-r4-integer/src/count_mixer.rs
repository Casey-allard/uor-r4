//! Integer full-support count mixtures. IDs 1..4095 are legal targets.
use crate::discrete_d11::{IntegerDiscreteModel, DISCRETE_TRUNK_SCHEMA, MAGIC};
use crate::{invalid, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SCHEMA: &str = "uor-r4.lut-discrete-count-mixer/1";
pub const CONTEXT_SCHEMA: &str = "uor-r4.lut-discrete-count-mixer/2";
pub const VOCAB: usize = 4096;
pub const CONTEXT_MAX_ORDER: usize = 4;
pub const CONTEXT_FLOOR_WEIGHT: u32 = 8;
pub const CONTEXT_HEADER: usize = (VOCAB + CONTEXT_MAX_ORDER + 1) * 4;
pub const CONTEXT_ROW_BYTES: usize = 12;
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
    context: Option<ContextRows>,
}

#[derive(Clone, Debug)]
struct ContextRows {
    weights: [u32; CONTEXT_MAX_ORDER + 1],
    // Keys and boundaries of rows, followed by one sentinel boundary.
    keys: Vec<u64>,
    starts: Vec<u32>,
    entries: Vec<u64>,
    unigram_top8: [u16; 8],
}

fn context_key<T: Copy + Into<u32>>(prefix: &[T], order: usize) -> Option<u64> {
    if order == 0 || order > CONTEXT_MAX_ORDER || order > prefix.len() {
        return None;
    }
    let mut key = (order as u64) << 48;
    for (slot, &value) in prefix.iter().rev().take(order).enumerate() {
        let id = value.into();
        if id >= VOCAB as u32 || (id <= 1 && (id == 1 || slot + 1 < order)) {
            return None;
        }
        key |= u64::from(id) << (36 - slot * 12);
    }
    Some(key)
}

impl ContextRows {
    fn order(&self) -> usize {
        self.weights
            .iter()
            .rposition(|&weight| weight != 0)
            .unwrap_or(0)
    }

    fn active<T: Copy + Into<u32>>(&self, prefix: &[T]) -> [Option<&[u64]>; CONTEXT_MAX_ORDER] {
        let mut active = [None; CONTEXT_MAX_ORDER];
        let mut fallback = None;
        for (i, entry) in active.iter_mut().enumerate().take(self.order()) {
            if let Some(row) =
                context_key(prefix, i + 1).and_then(|key| self.keys.binary_search(&key).ok())
            {
                fallback =
                    Some(&self.entries[self.starts[row] as usize..self.starts[row + 1] as usize]);
            }
            *entry = fallback;
        }
        active
    }
}

fn read_count(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes[..4].try_into().expect("validated count word"))
}

fn mass_step(cumulative: &mut u64, count: u64, total: u64, available: u64) -> u32 {
    let low = *cumulative * available / total;
    *cumulative += count;
    ((*cumulative * available / total) - low) as u32
}

fn unigram_from_counts(count_bytes: &[u8]) -> Result<Vec<u32>> {
    let counts: Vec<_> = count_bytes.chunks_exact(4).map(read_count).collect();
    let total = counts.iter().map(|&c| u64::from(c)).sum::<u64>();
    if counts[0] != 0 || total == 0 || total > u64::from(u32::MAX) {
        return Err(invalid("invalid unigram count total or BOS count"));
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
    Ok(unigram)
}

fn insert_rank(ranked: &mut [(u16, u64); 8], target: u16, mass: u64) {
    let score = (mass, std::cmp::Reverse(target));
    let at = ranked.partition_point(|&(id, value)| (value, std::cmp::Reverse(id)) > score);
    if at < ranked.len() && ranked[at].0 != target {
        ranked.copy_within(at..7, at + 1);
        ranked[at] = (target, mass);
    }
}

fn context_expert_mass(row: &[u64], target: u16) -> u32 {
    row.binary_search_by_key(&u64::from(target), |&word| word & 0xfff)
        .map_or(0, |index| (row[index] >> 12) as u32)
}

impl CountMixer {
    /// Canonical LE stream: conditional weight, V unigram counts, sorted (previous, target, count).
    pub fn from_words(words: &[u8]) -> Result<Self> {
        if words.len() < HEADER || (words.len() - HEADER) % 8 != 0 {
            return Err(invalid("partial count mixer word stream"));
        }
        let weight = read_count(words);
        if !(1..=255).contains(&weight) {
            return Err(invalid("invalid conditional weight"));
        }
        let unigram = unigram_from_counts(&words[4..HEADER])?;
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
            context: None,
        })
    }

    /// Canonical LE context stream: five Q8 weights, V counts, ascending packed
    /// (order, latest-first context, original ID) u64 keys with exact u32 counts.
    pub fn from_context_words(words: &[u8]) -> Result<Self> {
        if words.len() < CONTEXT_HEADER || (words.len() - CONTEXT_HEADER) % CONTEXT_ROW_BYTES != 0 {
            return Err(invalid("partial context count stream"));
        }
        let weights = std::array::from_fn(|i| read_count(&words[i * 4..]));
        if weights[0] != CONTEXT_FLOOR_WEIGHT
            || weights[1] == 0
            || weights.iter().map(|&x| u64::from(x)).sum::<u64>() != 256
            || weights[1..]
                .windows(2)
                .any(|pair| pair[0] == 0 && pair[1] != 0)
        {
            return Err(invalid("invalid context count weights"));
        }
        let unigram = unigram_from_counts(&words[(CONTEXT_MAX_ORDER + 1) * 4..CONTEXT_HEADER])?;
        let mut ranked = [(0, 0); 8];
        for (id, &mass) in unigram.iter().enumerate().skip(1) {
            insert_rank(&mut ranked, id as u16, u64::from(mass));
        }
        let mut state = ContextRows {
            weights,
            keys: Vec::new(),
            starts: Vec::new(),
            entries: Vec::with_capacity((words.len() - CONTEXT_HEADER) / CONTEXT_ROW_BYTES),
            unigram_top8: ranked.map(|(id, _)| id),
        };
        let raw: fn(&[u8]) -> u64 =
            |row| u64::from_le_bytes(row[..8].try_into().expect("by row width"));
        let mut previous = 0;
        let mut populated = [false; CONTEXT_MAX_ORDER + 1];
        let mut pending = &words[CONTEXT_HEADER..];
        while !pending.is_empty() {
            let key = raw(pending) >> 12;
            let order = (key >> 48) as usize;
            if order == 0 || order > state.order() {
                return Err(invalid("invalid count context order"));
            }
            let mut prefix = [0u16; CONTEXT_MAX_ORDER];
            for slot in 0..order {
                prefix[order - 1 - slot] = ((key >> (36 - 12 * slot)) & 0xfff) as u16;
            }
            if context_key(&prefix[..order], order) != Some(key) {
                return Err(invalid("non-canonical count context key"));
            }
            populated[order] = true;
            let mut row_count = 0;
            let mut total = 0u64;
            for row in pending
                .chunks_exact(CONTEXT_ROW_BYTES)
                .take_while(|row| raw(row) >> 12 == key)
            {
                row_count += 1;
                total += u64::from(read_count(&row[8..]));
            }
            let (rows, rest) = pending.split_at(row_count * CONTEXT_ROW_BYTES);
            pending = rest;
            if total == 0 || total > u64::from(u32::MAX) {
                return Err(invalid("invalid context count row total"));
            }
            state.keys.push(key);
            state.starts.push(
                u32::try_from(state.entries.len())
                    .map_err(|_| invalid("too many context entries"))?,
            );
            let mut cumulative = 0;
            for row in rows.chunks_exact(CONTEXT_ROW_BYTES) {
                let key = raw(row);
                let target = key & 0xfff;
                let count = read_count(&row[8..]);
                if key <= previous || target == 0 || count == 0 {
                    return Err(invalid("invalid or unordered context count entry"));
                }
                previous = key;
                let mass = mass_step(&mut cumulative, u64::from(count), total, Q24);
                state.entries.push((u64::from(mass) << 12) | target);
            }
        }
        if populated[1..=state.order()].contains(&false) {
            return Err(invalid("missing configured count context rows"));
        }
        state.starts.push(
            u32::try_from(state.entries.len()).map_err(|_| invalid("too many context entries"))?,
        );
        Ok(Self {
            weight: 256 - CONTEXT_FLOOR_WEIGHT,
            unigram,
            offsets: Vec::new(),
            conditional: Vec::new(),
            words: words.to_vec(),
            context: Some(state),
        })
    }

    pub fn schema(&self) -> &'static str {
        if self.context.is_some() {
            CONTEXT_SCHEMA
        } else {
            SCHEMA
        }
    }

    pub fn context_order(&self) -> usize {
        self.context.as_ref().map_or(1, ContextRows::order)
    }

    pub fn context_weights(&self) -> Option<[u32; CONTEXT_MAX_ORDER + 1]> {
        self.context.as_ref().map(|state| state.weights)
    }

    /// Q24 probability per occupied depth. A missing row reads the shorter suffix.
    pub fn context_masses<T: Copy + Into<u32>>(
        &self,
        prefix: &[T],
        target: u16,
    ) -> Option<[u32; CONTEXT_MAX_ORDER + 1]> {
        let state = self.context.as_ref()?;
        let uni = self.unigram.get(usize::from(target)).copied().unwrap_or(0);
        let active = state.active(prefix);
        Some(std::array::from_fn(|i| {
            if i == 0 {
                uni
            } else {
                active[i - 1].map_or(uni, |row| context_expert_mass(row, target))
            }
        }))
    }

    pub fn context_routes<T: Copy + Into<u32>>(&self, prefix: &[T]) -> [u64; CONTEXT_MAX_ORDER] {
        let mut fallback = 0;
        std::array::from_fn(|i| {
            if let Some(state) = &self.context {
                if i >= state.order() {
                    return 0;
                }
                if let Some(key) =
                    context_key(prefix, i + 1).filter(|key| state.keys.binary_search(key).is_ok())
                {
                    fallback = key;
                }
            }
            fallback
        })
    }

    pub fn mass_q32_for_prefix<T: Copy + Into<u32>>(&self, prefix: &[T], target: u16) -> u64 {
        match self.context_masses(prefix, target) {
            Some(masses) => self
                .context_weights()
                .unwrap_or_default()
                .into_iter()
                .zip(masses)
                .map(|(weight, mass)| u64::from(weight) * u64::from(mass))
                .sum(),
            None => self.mass_q32(
                prefix
                    .last()
                    .map_or(0, |&id| id.into().min(u16::MAX as u32) as u16),
                target,
            ),
        }
    }

    pub fn top8_for_prefix<T: Copy + Into<u32>>(&self, prefix: &[T]) -> [u16; 8] {
        let Some(state) = &self.context else {
            return self.top8(
                prefix
                    .last()
                    .map_or(0, |&id| id.into().min(u16::MAX as u32) as u16),
            );
        };
        let active = state.active(prefix);
        let mut ranked = [(0, 0); 8];
        let mut consider = |target: u16| {
            let uni = self.unigram[target as usize];
            let mut mass = u64::from(state.weights[0]) * u64::from(uni);
            for (row, &weight) in active.iter().zip(&state.weights[1..]) {
                if weight != 0 {
                    mass += u64::from(weight)
                        * u64::from(row.map_or(uni, |r| context_expert_mass(r, target)));
                }
            }
            insert_rank(&mut ranked, target, mass);
        };
        for target in state.unigram_top8 {
            consider(target);
        }
        for row in active.into_iter().take(state.order()).flatten() {
            for word in row {
                consider((word & 0xfff) as u16);
            }
        }
        ranked.map(|(id, _)| id)
    }

    pub fn top1_for_prefix<T: Copy + Into<u32>>(&self, prefix: &[T]) -> u16 {
        self.top8_for_prefix(prefix)[0]
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
        if let Some(state) = &self.context {
            let conditional =
                state.active(&[previous])[0].map_or(uni, |row| context_expert_mass(row, target));
            return (uni, conditional);
        }
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
    pack_schema(base, words, SCHEMA)
}

pub fn pack_context(base: &[u8], words: &[u8]) -> Result<Vec<u8>> {
    let _ = CountMixer::from_context_words(words)?;
    pack_schema(base, words, CONTEXT_SCHEMA)
}

fn pack_schema(base: &[u8], words: &[u8], schema: &str) -> Result<Vec<u8>> {
    let model = IntegerDiscreteModel::from_bytes(base)?;
    if model.artifact_schema() != DISCRETE_TRUNK_SCHEMA || model.shape().vocabulary != VOCAB as u32
    {
        return Err(invalid("expected V4096 base trunk"));
    }
    let header = serde_json::to_vec(&Envelope {
        schema: schema.to_string(),
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
