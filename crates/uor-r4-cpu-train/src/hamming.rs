//! Exact CPU execution of the integer artifact's Hamming copy and type reads.
//!
//! This is an independent port of the public `lut-stack/6` semantics. The
//! artifact reader supplies only the validated tables; all code construction,
//! candidate selection, payload caching and hook arithmetic happen here.

use uor_r4_integer::bitcode::popcount_swar;
use uor_r4_integer::stack::{
    copy_weight_q31, HammingImage, HammingReadImage, StackHamming, COPY_MEAN_EXP,
    HAMMING_WEIGHT_EXP,
};

const RESIDUAL_EXP: i32 = -16;
const NORM_FINE_EXP: i32 = -24;
const STREAM_SHIFT: u32 = (RESIDUAL_EXP - HAMMING_WEIGHT_EXP - COPY_MEAN_EXP) as u32;
const NORM_RAISE: u32 = (NORM_FINE_EXP - COPY_MEAN_EXP) as u32;
const READOUT_SHIFT: u32 = (NORM_FINE_EXP - HAMMING_WEIGHT_EXP - COPY_MEAN_EXP) as u32;
const TYPE_MEAN_SHIFT: u32 = 31;
const STREAM_COPY_RAISE: u32 = (COPY_MEAN_EXP - RESIDUAL_EXP + TYPE_MEAN_SHIFT as i32) as u32;
const READOUT_TYPE_DROP: u32 = (COPY_MEAN_EXP - NORM_FINE_EXP + TYPE_MEAN_SHIFT as i32) as u32;
const MAX_READ_CACHE_BYTES: u64 = 1 << 32;

#[derive(Clone)]
struct CopyHeads {
    copy: usize,
    kept: usize,
    width: usize,
    codes: Vec<u64>,
    margins: Vec<i32>,
    /// One mask per copy head, as the integer `CopyHeads` builds it: the low-half mask of a
    /// copy-key-2 head, `None` for a copy-key-1 head (claude-269).
    pair: Vec<Option<u64>>,
}

impl CopyHeads {
    fn new(
        record: &StackHamming,
        read: &HammingReadImage,
        table: &[i16],
        vocab: usize,
        width: usize,
    ) -> Self {
        let copy = record.heads / 2;
        Self {
            copy,
            kept: record.kept,
            width,
            codes: vocab_codes(
                &unit_rotations(&read.key_rotation),
                table,
                vocab,
                width,
                record.heads,
                0..copy,
            ),
            margins: read.margin[..copy].to_vec(),
            pair: (0..copy)
                .map(|h| {
                    (record.head_copy_key(h) == 2).then(|| (1u64 << (width / record.heads / 2)) - 1)
                })
                .collect(),
        }
    }

    fn joined(&self, near: u32, far: u32, head: usize) -> u64 {
        let code = self.codes[near as usize * self.copy + head];
        match self.pair[head] {
            None => code,
            Some(first) => (code & first) | (self.codes[far as usize * self.copy + head] & !first),
        }
    }

    fn query_at(&self, tokens: &[u32], position: usize, head: usize) -> u64 {
        self.joined(tokens[position], tokens[position.saturating_sub(1)], head)
    }

    fn key_at(&self, tokens: &[u32], position: usize, head: usize) -> u64 {
        self.joined(
            tokens[position - 1],
            tokens[position.saturating_sub(2)],
            head,
        )
    }

    fn choose(&self, head: usize, tokens: &[u32]) -> Vec<(usize, u32)> {
        let Some(current) = tokens.len().checked_sub(1) else {
            return Vec::new();
        };
        nearest(
            self.query_at(tokens, current, head),
            current,
            self.kept,
            |position| self.key_at(tokens, position, head),
        )
    }

    fn mean(&self, tokens: &[u32], table: &[i16]) -> Option<Vec<i128>> {
        let mut sum = vec![0i128; self.width];
        let mut found = 0i128;
        for head in 0..self.copy {
            let mut head_sum = vec![0i128; self.width];
            let mut count = 0i128;
            for (position, distance) in self.choose(head, tokens) {
                let weight = copy_weight_q31(self.margins[head], distance);
                if weight == 0 {
                    continue;
                }
                count += 1;
                let row = &table[tokens[position] as usize * self.width..][..self.width];
                for (slot, &point) in head_sum.iter_mut().zip(row) {
                    *slot += i128::from(point) * i128::from(weight);
                }
            }
            if count == 0 {
                continue;
            }
            found += 1;
            for (slot, value) in sum.iter_mut().zip(head_sum) {
                *slot += value / count;
            }
        }
        (found > 0).then(|| sum.into_iter().map(|value| value / found).collect())
    }
}

#[derive(Clone)]
struct TypeHeads {
    copy: usize,
    heads: usize,
    kept: usize,
    width: usize,
    keys: Vec<u64>,
    margins: Vec<i32>,
    query: Vec<[i64; 4]>,
    value: Vec<[i64; 4]>,
}

impl TypeHeads {
    fn new(
        record: &StackHamming,
        read: &HammingReadImage,
        table: &[i16],
        vocab: usize,
        width: usize,
    ) -> Self {
        let (copy, heads) = (record.heads / 2, record.heads);
        Self {
            copy,
            heads,
            kept: record.kept,
            width,
            keys: vocab_codes(
                &unit_rotations(&read.key_rotation),
                table,
                vocab,
                width,
                heads,
                copy..heads,
            ),
            margins: read.margin[copy..].to_vec(),
            query: unit_rotations(&read.query_rotation),
            value: unit_rotations(&read.value_rotation),
        }
    }

    fn queries(&self, point: &[i64]) -> Vec<u64> {
        let head_width = self.width / self.heads;
        (self.copy..self.heads)
            .map(|head| head_code(&self.query, point, head, head_width))
            .collect()
    }

    fn payload(&self, point: &[i64], out: &mut [i64]) {
        for ((lane, &unit), out) in point
            .chunks_exact(4)
            .zip(&self.value)
            .zip(out.chunks_exact_mut(4))
        {
            let rotated = hamilton(unit, [lane[0], lane[1], lane[2], lane[3]]);
            for (slot, value) in out.iter_mut().zip(rotated) {
                *slot =
                    round_shift(value, 31).clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
            }
        }
    }

    fn choose(&self, head: usize, query: u64, tokens: &[u32]) -> Vec<(usize, u32)> {
        if tokens.is_empty() {
            return Vec::new();
        }
        nearest(query, tokens.len() - 1, self.kept, |position| {
            self.keys[tokens[position - 1] as usize * (self.heads - self.copy) + head]
        })
    }

    fn mean(&self, queries: &[u64], tokens: &[u32], payloads: &[i64]) -> Option<Vec<i128>> {
        let mut sum = vec![0i128; self.width];
        let mut found = 0i128;
        for (head, &query) in queries.iter().enumerate() {
            let mut head_sum = vec![0i128; self.width];
            let mut count = 0i128;
            for (position, distance) in self.choose(head, query, tokens) {
                let weight = copy_weight_q31(self.margins[head], distance);
                if weight == 0 {
                    continue;
                }
                count += 1;
                let row = &payloads[position * self.width..][..self.width];
                for (slot, &value) in head_sum.iter_mut().zip(row) {
                    *slot += i128::from(value) * i128::from(weight);
                }
            }
            if count == 0 {
                continue;
            }
            found += 1;
            for (slot, value) in sum.iter_mut().zip(head_sum) {
                *slot += value / count;
            }
        }
        (found > 0).then(|| sum.into_iter().map(|value| value / found).collect())
    }
}

#[derive(Clone)]
struct Read {
    depth: usize,
    copy: CopyHeads,
    copy_weight: Vec<i32>,
    types: Option<TypeHeads>,
    type_weight: Vec<i32>,
}

#[derive(Clone)]
pub(crate) struct CpuHamming {
    reads: Vec<Read>,
    layers: usize,
    width: usize,
}

impl CpuHamming {
    pub(crate) fn new(
        image: &HammingImage,
        table: &[i16],
        vocab: usize,
        width: usize,
        layers: usize,
        context: usize,
    ) -> Result<Self, String> {
        if table.len() != vocab.saturating_mul(width) {
            return Err("Hamming reads require a writer table of vocab * width".to_owned());
        }
        let mut reads = Vec::with_capacity(image.reads.len());
        for read in &image.reads {
            let (copy_weight, type_weight) = read.weight.split_at(width);
            let typed = type_weight.iter().any(|&weight| weight != 0);
            reads.push(Read {
                depth: read.depth,
                copy: CopyHeads::new(&image.record, read, table, vocab, width),
                copy_weight: copy_weight.to_vec(),
                types: typed.then(|| TypeHeads::new(&image.record, read, table, vocab, width)),
                type_weight: type_weight.to_vec(),
            });
        }
        let typed = reads.iter().filter(|read| read.types.is_some()).count();
        check_payload_cache(typed, context, width)?;
        Ok(Self {
            reads,
            layers,
            width,
        })
    }

    pub(crate) fn payload_caches(&self, context: usize) -> Vec<Vec<i64>> {
        self.reads
            .iter()
            .map(|read| {
                if read.types.is_some() {
                    vec![0; context * self.width]
                } else {
                    Vec::new()
                }
            })
            .collect()
    }

    pub(crate) fn payload_lengths(&self, context: usize) -> Vec<usize> {
        self.reads
            .iter()
            .map(|read| read.types.as_ref().map_or(0, |_| context * self.width))
            .collect()
    }

    pub(crate) fn stream(
        &self,
        depth: usize,
        tokens: &[u32],
        table: &[i16],
        payloads: &mut [Vec<i64>],
        x: &mut [i32],
    ) {
        if depth == self.layers {
            return;
        }
        let Some(index) = self.reads.iter().position(|read| read.depth == depth) else {
            return;
        };
        let read = &self.reads[index];
        let copy = read.copy.mean(tokens, table);
        let typed = self.type_mean(
            index,
            &x.iter().map(|&v| i64::from(v)).collect::<Vec<_>>(),
            tokens,
            payloads,
        );
        if copy.is_none() && typed.is_none() {
            return;
        }
        for (channel, value) in x.iter_mut().enumerate() {
            let copy = copy.as_ref().map_or(0, |mean| mean[channel]);
            let typed = typed.as_ref().map_or(0, |mean| mean[channel]);
            *value = stream_sum(
                *value,
                read.copy_weight[channel],
                copy,
                read.type_weight[channel],
                typed,
            );
        }
    }

    pub(crate) fn readout(
        &self,
        tokens: &[u32],
        table: &[i16],
        payloads: &mut [Vec<i64>],
        norm: &mut [i64],
    ) {
        let Some(index) = self.reads.iter().position(|read| read.depth == self.layers) else {
            return;
        };
        let read = &self.reads[index];
        let copy = read.copy.mean(tokens, table);
        let typed = self.type_mean(index, norm, tokens, payloads);
        if copy.is_none() && typed.is_none() {
            return;
        }
        for (channel, value) in norm.iter_mut().enumerate() {
            let copy = copy.as_ref().map(|mean| mean[channel]);
            let typed = typed.as_ref().map_or(0, |mean| mean[channel]);
            *value = readout_sum(
                *value,
                read.copy_weight[channel],
                copy,
                read.type_weight[channel],
                typed,
            );
        }
    }

    fn type_mean(
        &self,
        index: usize,
        point: &[i64],
        tokens: &[u32],
        payloads: &mut [Vec<i64>],
    ) -> Option<Vec<i128>> {
        let types = self.reads[index].types.as_ref()?;
        let row = tokens.len().checked_sub(1)?;
        let cache = &mut payloads[index];
        types.payload(point, &mut cache[row * self.width..][..self.width]);
        let payloads = &cache[..(row + 1) * self.width];
        types.mean(&types.queries(point), tokens, payloads)
    }

    #[cfg(test)]
    pub(crate) fn erase_copy_codes(&mut self) {
        self.reads[0].copy.codes.fill(0);
    }
}

fn units(raw: &[i32]) -> impl Iterator<Item = [i64; 4]> + '_ {
    raw.chunks_exact(4)
        .map(|raw| unit_rotation([raw[0], raw[1], raw[2], raw[3]]))
}

fn unit_rotations(raw: &[i32]) -> Vec<[i64; 4]> {
    units(raw).collect()
}

fn unit_rotation(raw: [i32; 4]) -> [i64; 4] {
    let square = raw.iter().fold(0u128, |sum, &value| {
        let magnitude = u64::from(value.unsigned_abs());
        sum.wrapping_add(u128::from(magnitude.wrapping_mul(magnitude)))
    });
    let norm = isqrt(square << 32);
    std::array::from_fn(|channel| {
        let value = raw[channel];
        let magnitude = (u128::from(value.unsigned_abs()) << 46)
            .checked_div(norm)
            .unwrap_or(0);
        let signed = if value < 0 {
            -(magnitude as i128)
        } else {
            magnitude as i128
        };
        ((i128::from(1u64 << 31) * signed) >> 30) as i64
    })
}

fn head_code(units: &[[i64; 4]], point: &[i64], head: usize, head_width: usize) -> u64 {
    let mut word = 0u64;
    for coordinate in (0..head_width).step_by(4) {
        let at = head * head_width + coordinate;
        let rotated = hamilton(
            units[at / 4],
            [point[at], point[at + 1], point[at + 2], point[at + 3]],
        );
        for (offset, &value) in rotated.iter().enumerate() {
            if value >= 0 {
                word |= 1u64 << (coordinate + offset);
            }
        }
    }
    word
}

fn vocab_codes(
    units: &[[i64; 4]],
    table: &[i16],
    vocab: usize,
    width: usize,
    heads: usize,
    selected: std::ops::Range<usize>,
) -> Vec<u64> {
    let head_width = width / heads;
    let mut codes = Vec::with_capacity(vocab * selected.len());
    let mut point = vec![0i64; width];
    for row in table.chunks_exact(width).take(vocab) {
        for (slot, &value) in point.iter_mut().zip(row) {
            *slot = i64::from(value);
        }
        for head in selected.clone() {
            codes.push(head_code(units, &point, head, head_width));
        }
    }
    codes
}

fn nearest(query: u64, row: usize, kept: usize, key: impl Fn(usize) -> u64) -> Vec<(usize, u32)> {
    let mut candidates: Vec<(u32, usize)> = (1..row)
        .map(|position| (popcount_swar(query ^ key(position)), position))
        .collect();
    candidates.sort_unstable();
    candidates.truncate(kept);
    candidates
        .into_iter()
        .map(|(distance, position)| (position, distance))
        .collect()
}

fn hamilton(a: [i64; 4], b: [i64; 4]) -> [i128; 4] {
    let product =
        |left: usize, right: usize| i128::from(a[left]).wrapping_mul(i128::from(b[right]));
    [
        product(0, 0)
            .wrapping_sub(product(1, 1))
            .wrapping_sub(product(2, 2))
            .wrapping_sub(product(3, 3)),
        product(0, 1)
            .wrapping_add(product(1, 0))
            .wrapping_add(product(2, 3))
            .wrapping_sub(product(3, 2)),
        product(0, 2)
            .wrapping_sub(product(1, 3))
            .wrapping_add(product(2, 0))
            .wrapping_add(product(3, 1)),
        product(0, 3)
            .wrapping_add(product(1, 2))
            .wrapping_sub(product(2, 1))
            .wrapping_add(product(3, 0)),
    ]
}

fn stream_sum(x: i32, copy_weight: i32, copy: i128, type_weight: i32, typed: i128) -> i32 {
    let shift = STREAM_SHIFT + STREAM_COPY_RAISE;
    let sum = (i128::from(x) << shift)
        + ((i128::from(copy_weight) * copy) << STREAM_COPY_RAISE)
        + i128::from(type_weight) * typed;
    round_shift(sum, shift).clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32
}

fn readout_sum(
    norm: i64,
    copy_weight: i32,
    copy: Option<i128>,
    type_weight: i32,
    typed: i128,
) -> i64 {
    let moved = copy.map_or(0, |copy| {
        i128::from(copy_weight) * (copy - (i128::from(norm) << NORM_RAISE))
    });
    let typed = round_shift(i128::from(type_weight) * typed, READOUT_TYPE_DROP);
    let sum = round_shift(
        (i128::from(norm) << READOUT_SHIFT) + moved + typed,
        READOUT_SHIFT,
    );
    sum.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

fn round_shift(value: i128, shift: u32) -> i128 {
    let half = 1i128 << (shift - 1);
    if value >= 0 {
        (value + half) >> shift
    } else {
        -((half - value) >> shift)
    }
}

fn isqrt(value: u128) -> u128 {
    if value == 0 {
        return 0;
    }
    let mut remainder = value;
    let mut root = 0u128;
    let mut bit = 1u128 << ((127 - value.leading_zeros()) & !1);
    while bit != 0 {
        if remainder >= root + bit {
            remainder -= root + bit;
            root = (root >> 1) + bit;
        } else {
            root >>= 1;
        }
        bit >>= 2;
    }
    root
}

fn check_payload_cache(typed: usize, context: usize, width: usize) -> Result<(), String> {
    let bytes = (typed as u64)
        .checked_mul(context as u64)
        .and_then(|bytes| bytes.checked_mul(width as u64))
        .and_then(|bytes| bytes.checked_mul(8));
    match bytes {
        Some(bytes) if bytes <= MAX_READ_CACHE_BYTES => Ok(()),
        _ => Err(
            "Hamming type-head payloads exceed the artifact reader's 4 GiB cache bound".to_owned(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_and_readout_round_the_complete_sum_half_away_from_zero() {
        assert_eq!(stream_sum(3, 1, 1 << 50, 0, 0), 4);
        assert_eq!(stream_sum(3, -1, 1 << 50, 0, 0), 3);
        assert_eq!(stream_sum(-3, 1, 1 << 50, 0, 0), -3);
        assert_eq!(stream_sum(3, 1, 1 << 49, 1, 1 << 53), 4);
        assert_eq!(readout_sum(3, 5, None, 1, 1 << 54), 4);
        assert_eq!(readout_sum(3, 5, None, -1, 1 << 54), 3);
        assert_eq!(readout_sum(-7, 1 << 24, Some(5 << 19), 0, 0), 5);
    }

    #[test]
    fn the_payload_cache_is_bounded_at_four_gibibytes() {
        check_payload_cache(1, 1 << 16, 1 << 13).unwrap();
        assert!(check_payload_cache(1, 1 << 16, (1 << 13) + 1).is_err());
        assert!(check_payload_cache(usize::MAX, 2, 2).is_err());
    }

    #[test]
    fn candidates_include_the_immediate_predecessor_and_exclude_the_current_row() {
        // Five stream tokens put the current row at 4. Valid payload positions are 1..4:
        // 3 is the immediate predecessor, 0 has no predecessor, and 4 is the current row.
        let kept = nearest(0, 4, 16, |_| 0);
        assert_eq!(kept, vec![(1, 0), (2, 0), (3, 0)]);
    }

    #[test]
    fn copy_key_two_tells_two_facts_after_the_same_token_apart() {
        // B X P C X Q: asks at the final X whether the next fact follows (C, X) or (B, X).
        // The one-token key ties both positions; the two-token key keeps (C, X) at distance 0.
        let (b, x, c, p, q) = (1u32, 2, 3, 4, 5);
        let tokens = [10, b, x, p, c, x, q, 70, c, x];
        let mut codes = vec![u64::MAX; 71];
        codes[x as usize] = 0;
        codes[c as usize] = 1;
        codes[b as usize] = 0xff << 8;
        let one = CopyHeads {
            copy: 1,
            kept: 8,
            width: 64,
            codes: codes.clone(),
            margins: vec![4 << 16],
            pair: vec![None],
        };
        let two = CopyHeads {
            pair: vec![Some(0xff)],
            ..one.clone()
        };
        assert_eq!(&one.choose(0, &tokens)[..2], &[(3, 0), (6, 0)]);
        assert_eq!(two.choose(0, &tokens)[0], (6, 0));
        // claude-269: a mixed pair reads each copy head by its own key.
        for pair in [vec![Some(0xff), None], vec![None, Some(0xff)]] {
            let mixed = CopyHeads {
                copy: 2,
                codes: codes.iter().flat_map(|&c| [c, c]).collect(),
                margins: vec![4 << 16; 2],
                pair: pair.clone(),
                ..one.clone()
            };
            for (h, key) in pair.iter().enumerate() {
                let alone = if key.is_some() { &two } else { &one };
                assert_eq!(
                    mixed.choose(h, &tokens),
                    alone.choose(0, &tokens),
                    "{pair:?} head {h}"
                );
            }
        }
    }
}
