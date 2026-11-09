//! CPU oracle for Claude-330's question-keyed ask-head geometry.
//!
//! This module implements only the stable integer read: a token point plus a
//! learned ask offset, rotated through the artifact's Hamming key rotation,
//! compared with earlier copy-key codes. It does not implement the float trunk
//! or define an artifact schema.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use uor_r4_integer::bitcode::popcount_swar;
use uor_r4_integer::stack::{served_copy_codes_uorlut01, HammingImage, ServedCopyCodes};

use crate::hamming::{head_code, unit_rotations};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskHeadKept {
    pub depth: usize,
    pub head: usize,
    pub ask_margin_q16: i32,
    pub artifact_type_margin_q16: i32,
    pub query: u64,
    pub key: u64,
    pub kept: Vec<AskKeptPosition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskKeptPosition {
    pub position: u32,
    pub distance: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskOffsetFit {
    pub start_bits: u64,
    pub end_bits: u64,
    pub evaluations: u64,
    pub changed_coordinates: usize,
    pub budget_exhausted: bool,
    pub offset: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskCellBits {
    pub depth: usize,
    pub head: usize,
    pub bits: u32,
}

#[derive(Clone)]
struct AskRead {
    depth: usize,
    units: Vec<[i64; 4]>,
    heads: Vec<AskHead>,
}

#[derive(Clone, Copy)]
struct AskHead {
    head: usize,
    artifact_type_margin_q16: i32,
}

pub struct AskHeadOracle {
    table: Vec<i16>,
    vocab: usize,
    width: usize,
    heads: usize,
    kept: usize,
    head_width: usize,
    reads: Vec<AskRead>,
}

impl AskHeadOracle {
    pub fn from_artifact(bytes: &[u8]) -> Result<Self, String> {
        let ServedCopyCodes {
            image,
            table,
            vocab,
            width,
            ..
        } = served_copy_codes_uorlut01(bytes)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "artifact carries no Hamming reads".to_owned())?;
        Self::from_image(&image, table, vocab, width)
    }

    fn from_image(
        image: &HammingImage,
        table: Vec<i16>,
        vocab: usize,
        width: usize,
    ) -> Result<Self, String> {
        let heads = image.record.heads;
        if heads == 0 || !heads.is_multiple_of(2) {
            return Err("ask-head oracle needs an even positive head count".to_owned());
        }
        let head_width = width / heads;
        if head_width == 0 || !head_width.is_multiple_of(4) || head_width > 64 {
            return Err(
                "ask-head width is not a positive quaternion multiple within one word".to_owned(),
            );
        }
        let copy = heads / 2;
        let mut reads = Vec::with_capacity(image.reads.len());
        for read in &image.reads {
            reads.push(AskRead {
                depth: read.depth,
                units: unit_rotations(&read.key_rotation),
                heads: (copy..heads)
                    .map(|head| AskHead {
                        head,
                        artifact_type_margin_q16: read.margin[head],
                    })
                    .collect(),
            });
        }
        Ok(Self {
            table,
            vocab,
            width,
            heads,
            kept: image.record.kept,
            head_width,
            reads,
        })
    }

    pub fn vocab(&self) -> usize {
        self.vocab
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn head_count(&self) -> usize {
        self.heads
    }

    pub fn ask_head_count(&self) -> usize {
        self.heads / 2
    }

    pub fn kept_count(&self) -> usize {
        self.kept
    }

    pub fn zero_offset(&self) -> Vec<i32> {
        vec![0; self.width]
    }

    pub fn exact_offset_to_key(&self, question: u32, key: u32) -> Result<Vec<i32>, String> {
        let question = self.point(question)?;
        let key = self.point(key)?;
        Ok(question
            .iter()
            .zip(key)
            .map(|(&question, &key)| i32::from(key) - i32::from(question))
            .collect())
    }

    pub fn total_bits_to_key(
        &self,
        question: u32,
        key: u32,
        offset: &[i32],
    ) -> Result<u64, String> {
        if offset.len() != self.width {
            return Err("ask offset width differs".to_owned());
        }
        let (shifted, key) = self.prepared_points(question, key, offset)?;
        Ok(self.total_bits_to_points(&shifted, &key))
    }

    pub fn cell_bits_to_key(
        &self,
        question: u32,
        key: u32,
        offset: &[i32],
    ) -> Result<Vec<AskCellBits>, String> {
        if offset.len() != self.width {
            return Err("ask offset width differs".to_owned());
        }
        let (shifted, key) = self.prepared_points(question, key, offset)?;
        let mut cells = Vec::with_capacity(self.reads.iter().map(|read| read.heads.len()).sum());
        for read in &self.reads {
            for head in &read.heads {
                let query = head_code(&read.units, &shifted, head.head, self.head_width);
                let target = head_code(&read.units, &key, head.head, self.head_width);
                cells.push(AskCellBits {
                    depth: read.depth,
                    head: head.head,
                    bits: popcount_swar(query ^ target),
                });
            }
        }
        Ok(cells)
    }

    fn prepared_points(
        &self,
        question: u32,
        key: u32,
        offset: &[i32],
    ) -> Result<(Vec<i64>, Vec<i64>), String> {
        let question_point = self.point(question)?;
        let key_point = self.point(key)?;
        let mut shifted = vec![0i64; self.width];
        let mut key_i64 = vec![0i64; self.width];
        for ((slot, &point), &offset) in shifted.iter_mut().zip(question_point).zip(offset) {
            *slot = i64::from(point) + i64::from(offset);
        }
        for (slot, &point) in key_i64.iter_mut().zip(key_point) {
            *slot = i64::from(point);
        }
        Ok((shifted, key_i64))
    }

    /// Fit one shared token offset toward a target key with deterministic
    /// bounded coordinate search. The objective is the total Hamming bits over
    /// every ask head and depth, matching the found-copy pull.
    pub fn fit_offset_to_key(
        &self,
        question: u32,
        key: u32,
        max_evaluations: u64,
    ) -> Result<AskOffsetFit, String> {
        let mut offset = self.zero_offset();
        let question_point = self.point(question)?;
        let key_point = self.point(key)?;
        let mut shifted = question_point
            .iter()
            .map(|&point| i64::from(point))
            .collect::<Vec<_>>();
        let key_i64 = key_point
            .iter()
            .map(|&point| i64::from(point))
            .collect::<Vec<_>>();
        let start_bits = self.total_bits_to_points(&shifted, &key_i64);
        let mut best = start_bits;
        let mut evaluations = 0u64;
        let mut changed_coordinates = BTreeSet::new();
        for step in [16384i32, 4096, 1024, 256, 64, 16, 4, 1] {
            loop {
                let mut improved = false;
                for coordinate in 0..self.width {
                    for delta in [step, -step] {
                        if evaluations >= max_evaluations {
                            return Ok(AskOffsetFit {
                                start_bits,
                                end_bits: best,
                                evaluations,
                                changed_coordinates: changed_coordinates.len(),
                                budget_exhausted: true,
                                offset,
                            });
                        }
                        let previous = offset[coordinate];
                        let candidate = previous.saturating_add(delta);
                        if candidate == previous {
                            continue;
                        }
                        offset[coordinate] = candidate;
                        shifted[coordinate] = i64::from(candidate);
                        evaluations += 1;
                        let bits = self.total_bits_to_points(&shifted, &key_i64);
                        if bits < best {
                            best = bits;
                            changed_coordinates.insert(coordinate);
                            improved = true;
                        } else {
                            offset[coordinate] = previous;
                            shifted[coordinate] = i64::from(previous);
                        }
                    }
                }
                if !improved {
                    break;
                }
            }
        }
        Ok(AskOffsetFit {
            start_bits,
            end_bits: best,
            evaluations,
            changed_coordinates: changed_coordinates.len(),
            budget_exhausted: false,
            offset,
        })
    }

    fn total_bits_to_points(&self, shifted: &[i64], key: &[i64]) -> u64 {
        let mut bits = 0u64;
        for read in &self.reads {
            for head in &read.heads {
                let query = head_code(&read.units, shifted, head.head, self.head_width);
                let key = head_code(&read.units, key, head.head, self.head_width);
                bits += u64::from(popcount_swar(query ^ key));
            }
        }
        bits
    }

    pub fn kept_at_question(
        &self,
        tokens: &[u32],
        offset: &[i32],
    ) -> Result<Vec<AskHeadKept>, String> {
        if offset.len() != self.width {
            return Err("ask offset width differs".to_owned());
        }
        let (question, prefix) = tokens
            .split_last()
            .ok_or_else(|| "ask-head question row is empty".to_owned())?;
        let mut kept = Vec::with_capacity(self.reads.iter().map(|r| r.heads.len()).sum());
        for read in &self.reads {
            for head in &read.heads {
                let query = self.ask_query_code(read, *head, *question, offset)?;
                let mut candidates = (1..tokens.len())
                    .map(|position| {
                        let key = self.key_code(read, *head, prefix[position - 1])?;
                        Ok((popcount_swar(query ^ key), position))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                candidates.sort_unstable();
                candidates.truncate(self.kept);
                kept.push(AskHeadKept {
                    depth: read.depth,
                    head: head.head,
                    ask_margin_q16: ((self.head_width / 4) << 16) as i32,
                    artifact_type_margin_q16: head.artifact_type_margin_q16,
                    query,
                    key: self.key_code(read, *head, *question)?,
                    kept: candidates
                        .into_iter()
                        .map(|(distance, position)| AskKeptPosition {
                            position: position as u32,
                            distance,
                        })
                        .collect(),
                });
            }
        }
        Ok(kept)
    }

    fn point(&self, token: u32) -> Result<&[i16], String> {
        if token as usize >= self.vocab {
            return Err(format!(
                "token {token} is outside vocabulary {}",
                self.vocab
            ));
        }
        Ok(&self.table[token as usize * self.width..][..self.width])
    }

    fn ask_query_code(
        &self,
        read: &AskRead,
        head: AskHead,
        token: u32,
        offset: &[i32],
    ) -> Result<u64, String> {
        let point = self.point(token)?;
        let shifted = point
            .iter()
            .zip(offset)
            .map(|(&point, &offset)| i64::from(point) + i64::from(offset))
            .collect::<Vec<_>>();
        Ok(head_code(&read.units, &shifted, head.head, self.head_width))
    }

    fn key_code(&self, read: &AskRead, head: AskHead, token: u32) -> Result<u64, String> {
        let point = self.point(token)?;
        let point = point
            .iter()
            .map(|&value| i64::from(value))
            .collect::<Vec<_>>();
        Ok(head_code(&read.units, &point, head.head, self.head_width))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth};

    fn artifact() -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            244,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 2,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = true;
        snapshot.hamming = Some(synthetic_hamming(snapshot.width, 2, 4, 244));
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn identity_hamming_artifact() -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            244,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 2,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = true;
        let mut hamming = synthetic_hamming(snapshot.width, 2, 4, 244);
        for read in &mut hamming.reads {
            for unit in read.key_rotation.chunks_exact_mut(4) {
                unit.copy_from_slice(&[1.0, 0.0, 0.0, 0.0]);
            }
        }
        snapshot.hamming = Some(hamming);
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    #[test]
    fn malformed_and_read_less_artifacts_are_refused() {
        assert!(AskHeadOracle::from_artifact(&[]).is_err());

        let mut snapshot = synthetic_fbs_trunk_with(
            244,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 2,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = true;
        snapshot.hamming = None;
        let bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        assert!(AskHeadOracle::from_artifact(&bytes).is_err());
    }

    #[test]
    fn zero_offset_is_the_copy_key_geometry_and_exact_offset_zeroes_bits() {
        let oracle = AskHeadOracle::from_artifact(&artifact()).unwrap();
        let zero = oracle.zero_offset();
        assert_eq!(
            oracle.total_bits_to_key(41, 41, &zero).unwrap(),
            0,
            "a token is its own zero-offset key"
        );
        let exact = oracle.exact_offset_to_key(41, 17).unwrap();
        assert_eq!(oracle.total_bits_to_key(41, 17, &exact).unwrap(), 0);
        assert!(oracle
            .cell_bits_to_key(41, 17, &exact)
            .unwrap()
            .iter()
            .all(|cell| cell.bits == 0));
    }

    #[test]
    fn zero_offset_matches_an_independently_signed_copy_query() {
        let artifact = identity_hamming_artifact();
        let served = served_copy_codes_uorlut01(&artifact).unwrap().unwrap();
        let oracle = AskHeadOracle::from_artifact(&artifact).unwrap();
        let zero = oracle.zero_offset();
        let point = &served.table[41 * oracle.width()..][..oracle.width()];
        let head_width = oracle.width() / oracle.head_count();
        let kept = oracle.kept_at_question(&[3, 41], &zero).unwrap();
        for codes in &served.codes {
            assert_eq!(codes.len(), served.vocab * (oracle.head_count() / 2));
        }
        for read in &oracle.reads {
            for head in &read.heads {
                let query = oracle.ask_query_code(read, *head, 41, &zero).unwrap();
                let mut copy = 0u64;
                for coordinate in 0..head_width {
                    if point[head.head * head_width + coordinate] >= 0 {
                        copy |= 1u64 << coordinate;
                    }
                }
                assert_eq!(
                    query, copy,
                    "depth {} head {} differs from the copy query",
                    read.depth, head.head
                );
            }
            let first = kept.iter().find(|head| head.depth == read.depth).unwrap();
            assert_eq!(first.ask_margin_q16, ((head_width / 4) << 16) as i32);
        }
    }

    #[test]
    fn offset_fit_is_deterministic_bounded_and_non_improving() {
        let oracle = AskHeadOracle::from_artifact(&artifact()).unwrap();
        let first = oracle.fit_offset_to_key(41, 17, 50_000).unwrap();
        let second = oracle.fit_offset_to_key(41, 17, 50_000).unwrap();
        assert_eq!(first, second);
        assert!(first.evaluations <= 50_000);
        assert!(first.end_bits <= first.start_bits);
        assert!(first.changed_coordinates > 0);
        assert!(!first.budget_exhausted);
    }

    #[test]
    fn exact_offset_retains_the_value_position_after_its_key() {
        let oracle = AskHeadOracle::from_artifact(&artifact()).unwrap();
        let (question, key, value) = (41u32, 17u32, 300u32);
        let mut tokens = vec![3; 12];
        tokens[7] = key;
        tokens[8] = value;
        tokens.push(question);
        let exact = oracle.exact_offset_to_key(question, key).unwrap();
        let kept = oracle.kept_at_question(&tokens, &exact).unwrap();
        assert!(!kept.is_empty());
        assert!(kept.iter().all(|head| head
            .kept
            .iter()
            .any(|position| position.position == 8 && position.distance == 0)));
    }
}
