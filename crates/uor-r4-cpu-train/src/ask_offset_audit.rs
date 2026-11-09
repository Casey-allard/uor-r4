//! Read-only CPU audit of Claude-330's trained ask-offset tables.
//!
//! The audit reads the float checkpoint's shared writer table and Hamming
//! `d{depth}.ask` tensors. It does not export, serve or retrain the model.

use std::collections::BTreeSet;
use std::ops::Range;

use safetensors::{Dtype, SafeTensors};
use serde::{Deserialize, Serialize};
use uor_r4_integer::bitcode::popcount_swar;
use uor_r4_integer::fbs_snapshot::{hamilton_gate_f32, pack_signs_f32};

#[derive(Clone, Debug)]
pub struct TrainedAskOffsets {
    table: Vec<f32>,
    vocab: usize,
    width: usize,
    heads: usize,
    reads: Vec<TrainedAskRead>,
}

#[derive(Clone, Debug)]
struct TrainedAskRead {
    depth: usize,
    key_rotation: Vec<f32>,
    ask: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskOffsetCellReading {
    pub depth: usize,
    pub head: usize,
    pub zero_bits: u32,
    pub trained_bits: u32,
    pub improvement_bits: i32,
    pub zero_target_rank: usize,
    pub trained_target_rank: usize,
    pub zero_target_nearest: bool,
    pub trained_target_nearest: bool,
    pub offset_abs_sum_q16: u64,
    pub offset_changed_coordinates: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskOffsetRelationReading {
    pub relation: u32,
    pub question: u32,
    pub cells: Vec<AskOffsetCellReading>,
}

impl TrainedAskOffsets {
    pub fn from_bytes(model: &[u8], hamming: &[u8]) -> Result<Self, String> {
        let model = SafeTensors::deserialize(model)
            .map_err(|error| format!("model.safetensors: {error}"))?;
        let table = model
            .tensor("embedding.weight")
            .map_err(|error| format!("embedding.weight: {error}"))?;
        let (vocab, width) = match table.shape() {
            [vocab, width] => (*vocab, *width),
            shape => {
                return Err(format!(
                    "embedding.weight is {shape:?}, not a two-dimensional table"
                ))
            }
        };
        let table = f32_values(&table, "embedding.weight", &[vocab, width])?;

        let hamming = SafeTensors::deserialize(hamming)
            .map_err(|error| format!("hamming_read.safetensors: {error}"))?;
        let head_tensor = hamming
            .tensor("heads")
            .map_err(|error| format!("heads: {error}"))?;
        let heads = *u32_values(&head_tensor, "heads", &[1])?
            .first()
            .ok_or_else(|| "heads is empty".to_owned())? as usize;
        if heads < 2 || !heads.is_multiple_of(2) {
            return Err(format!("{heads} heads is not an even positive count"));
        }
        if !width.is_multiple_of(heads) {
            return Err(format!("width {width} is not divisible by {heads} heads"));
        }
        let head_width = width / heads;
        if head_width == 0 || !head_width.is_multiple_of(4) || head_width > 64 {
            return Err(format!(
                "head width {head_width} is not a quaternion multiple within one word"
            ));
        }

        let mut all_depths = BTreeSet::new();
        let mut ask_depths = BTreeSet::new();
        for name in hamming.names() {
            if let Some((depth, part)) = name
                .strip_prefix('d')
                .and_then(|name| name.split_once('.'))
                .and_then(|(depth, part)| depth.parse::<usize>().ok().map(|depth| (depth, part)))
            {
                all_depths.insert(depth);
                if part == "ask" {
                    ask_depths.insert(depth);
                }
            }
        }
        if ask_depths.is_empty() {
            return Err("checkpoint carries no d{depth}.ask tensors".to_owned());
        }
        if ask_depths != all_depths {
            return Err(format!(
                "ask tables cover depths {ask_depths:?}, but the Hamming read covers {all_depths:?}"
            ));
        }
        if hamming
            .names()
            .iter()
            .any(|name| name.ends_with(".step_rotation"))
        {
            return Err("ask heads and the type spin are mutually exclusive".to_owned());
        }

        let mut reads = Vec::with_capacity(ask_depths.len());
        for depth in ask_depths {
            let ask_name = format!("d{depth}.ask");
            let rotation_name = format!("d{depth}.key_rotation");
            let ask = hamming
                .tensor(&ask_name)
                .map_err(|error| format!("{ask_name}: {error}"))?;
            let rotation = hamming
                .tensor(&rotation_name)
                .map_err(|error| format!("{rotation_name}: {error}"))?;
            reads.push(TrainedAskRead {
                depth,
                key_rotation: f32_values(&rotation, &rotation_name, &[width])?,
                ask: f32_values(&ask, &ask_name, &[vocab, width])?,
            });
        }

        Ok(Self {
            table,
            vocab,
            width,
            heads,
            reads,
        })
    }

    pub fn vocab(&self) -> usize {
        self.vocab
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn heads(&self) -> usize {
        self.heads
    }

    pub fn ask_head_count(&self) -> usize {
        self.heads / 2
    }

    pub fn depths(&self) -> impl Iterator<Item = usize> + '_ {
        self.reads.iter().map(|read| read.depth)
    }

    pub fn audit_relations(
        &self,
        relations: Range<u32>,
        query_shift: u32,
    ) -> Result<Vec<AskOffsetRelationReading>, String> {
        if relations.is_empty() {
            return Err("relation token range is empty".to_owned());
        }
        let keys = relations.collect::<Vec<_>>();
        let mut out = Vec::with_capacity(keys.len());
        for &relation in &keys {
            let question = relation
                .checked_add(query_shift)
                .ok_or_else(|| "question token overflow".to_owned())?;
            self.check_token(relation)?;
            self.check_token(question)?;
            let mut cells = Vec::with_capacity(self.reads.len() * self.ask_head_count());
            for read in &self.reads {
                let zero_query = self.table_row(question);
                let offset = &read.ask[question as usize * self.width..][..self.width];
                let trained_query = zero_query
                    .iter()
                    .zip(offset)
                    .map(|(&point, &offset)| point + offset)
                    .collect::<Vec<_>>();
                if let Some(coordinate) = trained_query.iter().position(|value| !value.is_finite())
                {
                    return Err(format!(
                        "token {question} depth {} query coordinate {coordinate} is not finite after adding its ask offset",
                        read.depth
                    ));
                }
                let key_points = keys
                    .iter()
                    .map(|&key| self.table_row(key))
                    .collect::<Vec<_>>();
                for head in self.heads / 2..self.heads {
                    let zero = self.head_code(read, zero_query, head)?;
                    let trained = self.head_code(read, &trained_query, head)?;
                    let target_index = relation as usize - keys[0] as usize;
                    let distances = |query| -> Result<Vec<u64>, String> {
                        key_points
                            .iter()
                            .map(|point| {
                                Ok(u64::from(popcount_swar(
                                    query ^ self.head_code(read, point, head)?,
                                )))
                            })
                            .collect()
                    };
                    let zero_distances = distances(zero)?;
                    let trained_distances = distances(trained)?;
                    let zero_target = zero_distances[target_index];
                    let trained_target = trained_distances[target_index];
                    let rank = |values: &[u64]| {
                        1 + values
                            .iter()
                            .filter(|&&distance| distance < values[target_index])
                            .count()
                    };
                    let zero_nearest = zero_target == *zero_distances.iter().min().unwrap();
                    let trained_nearest =
                        trained_target == *trained_distances.iter().min().unwrap();
                    cells.push(AskOffsetCellReading {
                        depth: read.depth,
                        head,
                        zero_bits: zero_target as u32,
                        trained_bits: trained_target as u32,
                        improvement_bits: zero_target as i32 - trained_target as i32,
                        zero_target_rank: rank(&zero_distances),
                        trained_target_rank: rank(&trained_distances),
                        zero_target_nearest: zero_nearest,
                        trained_target_nearest: trained_nearest,
                        offset_abs_sum_q16: offset.iter().try_fold(0u64, |sum, &value| {
                            let q16 = (f64::from(value).abs() * 65_536.0).round();
                            if !q16.is_finite() || q16 > u64::MAX as f64 {
                                return Err(format!(
                                    "token {question} has an unrepresentable ask offset"
                                ));
                            }
                            sum.checked_add(q16 as u64).ok_or_else(|| {
                                format!("token {question} ask-offset magnitude overflows")
                            })
                        })?,
                        offset_changed_coordinates: offset.iter().filter(|&&v| v != 0.0).count(),
                    });
                }
            }
            out.push(AskOffsetRelationReading {
                relation,
                question,
                cells,
            });
        }
        Ok(out)
    }

    fn head_code(&self, read: &TrainedAskRead, point: &[f32], head: usize) -> Result<u64, String> {
        let head_width = self.width / self.heads;
        let start = head * head_width;
        let rotated = hamilton_gate_f32(
            &read.key_rotation[start..start + head_width],
            &point[start..start + head_width],
        );
        if let Some(coordinate) = rotated.iter().position(|value| !value.is_finite()) {
            return Err(format!(
                "depth {} head {head} rotated coordinate {coordinate} is not finite",
                read.depth
            ));
        }
        Ok(pack_signs_f32(&rotated)[0])
    }

    fn table_row(&self, token: u32) -> &[f32] {
        &self.table[token as usize * self.width..][..self.width]
    }

    fn check_token(&self, token: u32) -> Result<(), String> {
        if token as usize >= self.vocab {
            return Err(format!(
                "token {token} is outside vocabulary {}",
                self.vocab
            ));
        }
        Ok(())
    }
}

fn f32_values(
    tensor: &safetensors::tensor::TensorView<'_>,
    name: &str,
    shape: &[usize],
) -> Result<Vec<f32>, String> {
    if tensor.dtype() != Dtype::F32 || tensor.shape() != shape {
        return Err(format!(
            "{name} is {:?} {:?}, not F32 {shape:?}",
            tensor.dtype(),
            tensor.shape()
        ));
    }
    let values = tensor
        .data()
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().expect("four bytes")))
        .collect::<Vec<_>>();
    if let Some(index) = values.iter().position(|value| !value.is_finite()) {
        return Err(format!("{name}[{index}] is not finite"));
    }
    Ok(values)
}

fn u32_values(
    tensor: &safetensors::tensor::TensorView<'_>,
    name: &str,
    shape: &[usize],
) -> Result<Vec<u32>, String> {
    if tensor.dtype() != Dtype::U32 || tensor.shape() != shape {
        return Err(format!(
            "{name} is {:?} {:?}, not U32 {shape:?}",
            tensor.dtype(),
            tensor.shape()
        ));
    }
    Ok(tensor
        .data()
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four bytes")))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use safetensors::{serialize, tensor::TensorView};

    fn f32_bytes(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
        values.into_iter().flat_map(f32::to_le_bytes).collect()
    }

    fn u32_bytes(values: impl IntoIterator<Item = u32>) -> Vec<u8> {
        values.into_iter().flat_map(u32::to_le_bytes).collect()
    }

    fn file(tensors: Vec<(&str, Dtype, Vec<usize>, Vec<u8>)>) -> Vec<u8> {
        let tensors = tensors
            .into_iter()
            .map(|(name, dtype, shape, data)| (name.to_owned(), dtype, shape, data))
            .collect::<Vec<_>>();
        let views = tensors
            .iter()
            .map(|(name, dtype, shape, data)| {
                (
                    name.clone(),
                    TensorView::new(*dtype, shape.clone(), data).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        serialize(views, None).unwrap()
    }

    fn fixture(offset: f32, relation_18: f32, include_ask: bool) -> (Vec<u8>, Vec<u8>) {
        let (vocab, width, heads) = (64usize, 8usize, 2usize);
        let mut table = vec![1.0f32; vocab * width];
        table[18 * width..19 * width].fill(relation_18);
        let model = file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(table),
        )]);
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        let mut ask = vec![0.0f32; vocab * width];
        ask[33 * width..34 * width].fill(offset);
        let mut tensors = vec![
            ("heads", Dtype::U32, vec![1], u32_bytes([heads as u32])),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(rotation),
            ),
        ];
        if include_ask {
            tensors.push(("d2.ask", Dtype::F32, vec![vocab, width], f32_bytes(ask)));
        }
        (model, file(tensors))
    }

    #[test]
    fn audit_distinguishes_improvement_from_regression() {
        let (model, hamming) = fixture(-2.0, -1.0, true);
        let offsets = TrainedAskOffsets::from_bytes(&model, &hamming).unwrap();
        assert_eq!(
            (
                offsets.vocab(),
                offsets.width(),
                offsets.heads(),
                offsets.depths().collect::<Vec<_>>()
            ),
            (64, 8, 2, vec![2])
        );
        let readings = offsets.audit_relations(16..32, 16).unwrap();
        let unchanged = readings
            .iter()
            .find(|reading| reading.relation == 16)
            .unwrap()
            .cells
            .first()
            .unwrap();
        assert_eq!((unchanged.zero_bits, unchanged.trained_bits), (0, 0));
        assert!(unchanged.trained_target_nearest);

        let changed = readings
            .iter()
            .find(|reading| reading.relation == 17)
            .unwrap()
            .cells
            .first()
            .unwrap();
        assert_eq!((changed.zero_bits, changed.trained_bits), (0, 4));
        assert_eq!(changed.improvement_bits, -4);
        assert!(changed.zero_target_nearest);
        assert!(!changed.trained_target_nearest);
        assert_eq!(changed.offset_changed_coordinates, 8);
        assert_eq!(changed.offset_abs_sum_q16, 8 * 2 * 65_536);
    }

    #[test]
    fn missing_or_malformed_ask_tables_are_refused() {
        let (model, hamming) = fixture(0.0, -1.0, false);
        assert!(TrainedAskOffsets::from_bytes(&model, &hamming)
            .unwrap_err()
            .contains("no d{depth}.ask"));

        let (_, hamming) = fixture(f32::NAN, -1.0, true);
        assert!(TrainedAskOffsets::from_bytes(&model_file(), &hamming)
            .unwrap_err()
            .contains("not finite"));
    }

    #[test]
    fn partial_ask_depths_are_refused() {
        let (model, _) = fixture(0.0, -1.0, true);
        let width = 8;
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        let hamming = file(vec![
            ("heads", Dtype::U32, vec![1], u32_bytes([2])),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(rotation.clone()),
            ),
            (
                "d2.ask",
                Dtype::F32,
                vec![64, width],
                f32_bytes(vec![0.0; 64 * width]),
            ),
            (
                "d6.key_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(rotation),
            ),
        ]);
        assert!(TrainedAskOffsets::from_bytes(&model, &hamming)
            .unwrap_err()
            .contains("ask tables cover depths"));
    }

    #[test]
    fn zero_head_width_and_empty_relations_are_refused() {
        let model = file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![64, 0],
            Vec::new(),
        )]);
        let hamming = file(vec![
            ("heads", Dtype::U32, vec![1], u32_bytes([2])),
            ("d2.key_rotation", Dtype::F32, vec![0], Vec::new()),
            ("d2.ask", Dtype::F32, vec![64, 0], Vec::new()),
        ]);
        assert!(TrainedAskOffsets::from_bytes(&model, &hamming)
            .unwrap_err()
            .contains("head width 0"));

        let (model, hamming) = fixture(0.0, -1.0, true);
        let offsets = TrainedAskOffsets::from_bytes(&model, &hamming).unwrap();
        assert!(offsets
            .audit_relations(16..16, 16)
            .unwrap_err()
            .contains("range is empty"));

        let (vocab, width) = (64usize, 8usize);
        let mut table = vec![1.0f32; vocab * width];
        table[33 * width..34 * width].fill(f32::MAX);
        let model = file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(table),
        )]);
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        let mut ask = vec![0.0f32; vocab * width];
        ask[33 * width..34 * width].fill(f32::MAX);
        let hamming = file(vec![
            ("heads", Dtype::U32, vec![1], u32_bytes([2])),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(rotation),
            ),
            ("d2.ask", Dtype::F32, vec![vocab, width], f32_bytes(ask)),
        ]);
        let offsets = TrainedAskOffsets::from_bytes(&model, &hamming).unwrap();
        assert!(offsets
            .audit_relations(16..32, 16)
            .unwrap_err()
            .contains("not finite after adding"));
    }

    fn model_file() -> Vec<u8> {
        fixture(0.0, -1.0, true).0
    }
}
