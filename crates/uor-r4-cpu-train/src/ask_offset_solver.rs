//! Offline CPU solver for Claude-330's per-token ask offsets.
//!
//! The found-copy pull depends on token ids, the shared writer table and key
//! rotation alone. This module optimizes checkpoint-compatible ask rows against
//! supplied `(question, target key)` pulls without running the trunk.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use rayon::prelude::*;
use safetensors::tensor::TensorView;
use safetensors::{serialize, Dtype, SafeTensors};
use serde::{Deserialize, Serialize};
use uor_r4_integer::bitcode::popcount_swar;
use uor_r4_integer::fbs_snapshot::{hamilton_gate_f32, pack_signs_f32};

pub const PULL_CORPUS_SCHEMA: &str = "uor-r4.ask-pull-corpus/1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskPull {
    pub question: u32,
    pub key: u32,
    #[serde(default = "unit_weight")]
    pub weight: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskPullCorpus {
    pub schema: String,
    pub pulls: Vec<AskPull>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskOffsetSolveReading {
    pub question: u32,
    pub depth: usize,
    pub pulls: usize,
    pub pull_weight: u64,
    pub target_keys: usize,
    pub zero_weighted_bits: u64,
    pub solved_weighted_bits: u64,
    pub improvement_bits: u64,
    pub candidate_evaluations: u64,
    pub coordinate_evaluations: u64,
    pub total_evaluations: u64,
    pub search_starts: u64,
    pub budget_exhausted: bool,
    pub offset_changed_coordinates: usize,
    pub offset_abs_sum_q16: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskCorpusAuditReading {
    pub question: u32,
    pub depth: usize,
    pub pulls: usize,
    pub pull_weight: u64,
    pub target_keys: usize,
    pub zero_weighted_bits: u64,
    pub candidate_weighted_bits: u64,
    pub improvement_bits: u64,
}

pub struct SolvedAskOffsets {
    pub hamming_bytes: Vec<u8>,
    pub readings: Vec<AskOffsetSolveReading>,
}

struct SolvedQuestion {
    offset: Vec<f32>,
    zero_bits: u64,
    solved_bits: u64,
    candidate_evaluations: u64,
    coordinate_evaluations: u64,
    search_starts: u64,
    budget_exhausted: bool,
}

#[derive(Clone)]
struct NamedTensor {
    name: String,
    dtype: Dtype,
    shape: Vec<usize>,
    data: Vec<u8>,
}

#[derive(Clone)]
struct SolveRead {
    depth: usize,
    key_rotation: Vec<f32>,
}

pub struct AskOffsetSolveBasis {
    table: Vec<f32>,
    vocab: usize,
    width: usize,
    heads: usize,
    reads: Vec<SolveRead>,
    original_tensors: Vec<NamedTensor>,
    metadata: Option<std::collections::HashMap<String, String>>,
}

impl AskPullCorpus {
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        let corpus: Self =
            serde_json::from_slice(bytes).map_err(|error| format!("pull corpus: {error}"))?;
        if corpus.schema != PULL_CORPUS_SCHEMA {
            return Err(format!(
                "pull corpus schema is {:?}, not {PULL_CORPUS_SCHEMA:?}",
                corpus.schema
            ));
        }
        Ok(corpus)
    }

    pub fn b2_relations(relations: Range<u32>, query_shift: u32) -> Result<Self, String> {
        let pulls = relations
            .map(|key| {
                let question = key
                    .checked_add(query_shift)
                    .ok_or_else(|| format!("question token for key {key} overflows"))?;
                Ok(AskPull {
                    question,
                    key,
                    weight: 1,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            schema: PULL_CORPUS_SCHEMA.to_owned(),
            pulls,
        })
    }

    fn validate(&self, vocab: usize) -> Result<(), String> {
        if self.pulls.is_empty() {
            return Err("pull corpus is empty".to_owned());
        }
        for (index, pull) in self.pulls.iter().enumerate() {
            if pull.weight == 0 {
                return Err(format!("pull {index} has zero weight"));
            }
            for (role, token) in [("question", pull.question), ("key", pull.key)] {
                if token as usize >= vocab {
                    return Err(format!(
                        "pull {index} {role} token {token} is outside vocabulary {vocab}"
                    ));
                }
            }
        }
        Ok(())
    }
}

impl AskOffsetSolveBasis {
    pub fn from_bytes(model: &[u8], hamming: &[u8]) -> Result<Self, String> {
        let model_tensors = SafeTensors::deserialize(model)
            .map_err(|error| format!("model.safetensors: {error}"))?;
        let table_tensor = model_tensors
            .tensor("embedding.weight")
            .map_err(|error| format!("embedding.weight: {error}"))?;
        let (vocab, width) = match table_tensor.shape() {
            [vocab, width] => (*vocab, *width),
            shape => {
                return Err(format!(
                    "embedding.weight is {shape:?}, not a two-dimensional table"
                ))
            }
        };
        let table = f32_values(&table_tensor, "embedding.weight", &[vocab, width])?;

        let hamming_tensors =
            SafeTensors::deserialize(hamming).map_err(|error| format!("hamming_read: {error}"))?;
        let metadata = safetensors_metadata(hamming)?;
        let head_tensor = hamming_tensors
            .tensor("heads")
            .map_err(|error| format!("heads: {error}"))?;
        let heads = *u32_values(&head_tensor, "heads", &[1])?
            .first()
            .ok_or_else(|| "heads is empty".to_owned())? as usize;
        if heads < 2 || !heads.is_multiple_of(2) || !width.is_multiple_of(heads) {
            return Err(format!("{heads} heads over width {width} is invalid"));
        }
        let head_width = width / heads;
        if head_width == 0 || !head_width.is_multiple_of(4) || head_width > 64 {
            return Err(format!(
                "head width {head_width} is not a quaternion multiple within one word"
            ));
        }
        if hamming_tensors
            .names()
            .iter()
            .any(|name| name.ends_with(".step_rotation"))
        {
            return Err("ask heads and the type spin are mutually exclusive".to_owned());
        }
        for name in hamming_tensors.names() {
            if matches!(name, "heads" | "copy_key" | "copy_keys") {
                continue;
            }
            let recognized = name
                .strip_prefix('d')
                .and_then(|name| name.split_once('.'))
                .is_some_and(|(depth, part)| {
                    canonical_depth(depth).is_some()
                        && matches!(
                            part,
                            "query_rotation"
                                | "key_rotation"
                                | "value_rotation"
                                | "margin"
                                | "weight"
                                | "ask"
                        )
                });
            if !recognized {
                return Err(format!("unknown Claude-330 Hamming tensor {name}"));
            }
        }

        let mut depths = BTreeSet::new();
        for name in hamming_tensors.names() {
            if let Some((depth, _)) = name
                .strip_prefix('d')
                .and_then(|name| name.split_once('.'))
                .and_then(|(depth, part)| canonical_depth(depth).map(|depth| (depth, part)))
            {
                depths.insert(depth);
            }
        }
        if depths.is_empty() {
            return Err("checkpoint carries no Hamming depths".to_owned());
        }
        let depths = depths.into_iter().collect::<Vec<_>>();
        if depths != [2, 6] {
            return Err(format!(
                "Claude-330 Hamming depths are [2, 6], not {depths:?}"
            ));
        }

        let mut reads = Vec::with_capacity(depths.len());
        let mut ask_depths = BTreeSet::new();
        for &depth in &depths {
            let quaternion_shape = vec![width / 4, 4];
            for (part, shape) in [
                ("query_rotation", quaternion_shape.clone()),
                ("key_rotation", quaternion_shape.clone()),
                ("value_rotation", quaternion_shape.clone()),
                ("margin", vec![heads]),
                ("weight", vec![2, width]),
            ] {
                let name = format!("d{depth}.{part}");
                let tensor = hamming_tensors
                    .tensor(&name)
                    .map_err(|error| format!("{name}: {error}"))?;
                f32_values(&tensor, &name, &shape)?;
            }
            let ask_name = format!("d{depth}.ask");
            if hamming_tensors.names().contains(&ask_name.as_str()) {
                let ask = hamming_tensors
                    .tensor(&ask_name)
                    .map_err(|error| format!("{ask_name}: {error}"))?;
                f32_values(&ask, &ask_name, &[vocab, width])?;
                ask_depths.insert(depth);
            }
            let key_name = format!("d{depth}.key_rotation");
            let key_rotation = hamming_tensors
                .tensor(&key_name)
                .map_err(|error| format!("{key_name}: {error}"))?;
            reads.push(SolveRead {
                depth,
                key_rotation: f32_values(&key_rotation, &key_name, &quaternion_shape)?,
            });
        }
        if !ask_depths.is_empty() && ask_depths.len() != depths.len() {
            return Err(format!(
                "ask tables cover depths {ask_depths:?}, but the Hamming read covers {depths:?}"
            ));
        }
        let has_copy_key = hamming_tensors.names().contains(&"copy_key");
        let has_copy_keys = hamming_tensors.names().contains(&"copy_keys");
        if has_copy_key && has_copy_keys {
            return Err("checkpoint records both copy_key and copy_keys".to_owned());
        }
        if has_copy_key {
            let value = *u32_values(
                &hamming_tensors
                    .tensor("copy_key")
                    .map_err(|error| format!("copy_key: {error}"))?,
                "copy_key",
                &[1],
            )?
            .first()
            .ok_or_else(|| "copy_key is empty".to_owned())?;
            if !matches!(value, 1 | 2) {
                return Err(format!("copy_key {value} is not 1 or 2"));
            }
            if value == 2 && !head_width.is_multiple_of(8) {
                return Err("copy_key 2 requires heads of whole 8-coordinate blocks".to_owned());
            }
        }
        if has_copy_keys {
            let values = u32_values(
                &hamming_tensors
                    .tensor("copy_keys")
                    .map_err(|error| format!("copy_keys: {error}"))?,
                "copy_keys",
                &[heads / 2],
            )?;
            if values.iter().any(|&value| !matches!(value, 1 | 2)) {
                return Err("copy_keys carries a value other than 1 or 2".to_owned());
            }
            if values.contains(&2) && !head_width.is_multiple_of(8) {
                return Err(
                    "copy_keys naming 2 requires heads of whole 8-coordinate blocks".to_owned(),
                );
            }
            if values.windows(2).all(|pair| pair[0] == pair[1]) {
                return Err("copy_keys is uniform; record a scalar copy_key".to_owned());
            }
        }

        let mut names = hamming_tensors.names().to_vec();
        names.sort_unstable();
        let original_tensors = names
            .into_iter()
            .filter(|name| !name.ends_with(".ask"))
            .map(|name| {
                let tensor = hamming_tensors
                    .tensor(name)
                    .map_err(|error| format!("{name}: {error}"))?;
                Ok(NamedTensor {
                    name: name.to_owned(),
                    dtype: tensor.dtype(),
                    shape: tensor.shape().to_vec(),
                    data: tensor.data().to_vec(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Self {
            table,
            vocab,
            width,
            heads,
            reads,
            original_tensors,
            metadata,
        })
    }

    pub fn solve(
        &self,
        corpus: &AskPullCorpus,
        max_evaluations_per_cell: u64,
    ) -> Result<SolvedAskOffsets, String> {
        corpus.validate(self.vocab)?;
        if max_evaluations_per_cell == 0 {
            return Err("max evaluations must be positive".to_owned());
        }
        let mut groups: BTreeMap<u32, Vec<&AskPull>> = BTreeMap::new();
        for pull in &corpus.pulls {
            groups.entry(pull.question).or_default().push(pull);
        }
        let tasks = self
            .reads
            .iter()
            .enumerate()
            .flat_map(|(read_index, read)| {
                groups
                    .iter()
                    .map(move |(&question, pulls)| (read_index, read, question, pulls))
            })
            .collect::<Vec<_>>();
        let cells = tasks
            .into_par_iter()
            .map(|(read_index, read, question, pulls)| {
                let solved =
                    self.solve_question(read, question, pulls, max_evaluations_per_cell)?;
                let offset_changed_coordinates =
                    solved.offset.iter().filter(|&&value| value != 0.0).count();
                let offset_abs_sum_q16 = offset_abs_sum_q16(&solved.offset)?;
                Ok((
                    read_index,
                    question,
                    solved.offset,
                    AskOffsetSolveReading {
                        question,
                        depth: read.depth,
                        pulls: pulls.len(),
                        pull_weight: pull_weight(pulls)?,
                        target_keys: pulls
                            .iter()
                            .map(|pull| pull.key)
                            .collect::<BTreeSet<_>>()
                            .len(),
                        zero_weighted_bits: solved.zero_bits,
                        solved_weighted_bits: solved.solved_bits,
                        improvement_bits: solved.zero_bits.saturating_sub(solved.solved_bits),
                        candidate_evaluations: solved.candidate_evaluations,
                        coordinate_evaluations: solved.coordinate_evaluations,
                        total_evaluations: solved
                            .candidate_evaluations
                            .checked_add(solved.coordinate_evaluations)
                            .ok_or_else(|| "evaluation total overflows".to_owned())?,
                        search_starts: solved.search_starts,
                        budget_exhausted: solved.budget_exhausted,
                        offset_changed_coordinates,
                        offset_abs_sum_q16,
                    },
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut tables = self
            .reads
            .iter()
            .map(|read| (read.depth, vec![0.0f32; self.vocab * self.width]))
            .collect::<BTreeMap<_, _>>();
        let mut readings = Vec::with_capacity(cells.len());
        for (read_index, question, offset, reading) in cells {
            let depth = self.reads[read_index].depth;
            tables
                .get_mut(&depth)
                .ok_or_else(|| format!("depth {depth} table disappeared"))?
                [question as usize * self.width..][..self.width]
                .copy_from_slice(&offset);
            readings.push(reading);
        }
        readings.sort_by_key(|reading| (reading.question, reading.depth));
        Ok(SolvedAskOffsets {
            hamming_bytes: self.serialize_hamming(&tables)?,
            readings,
        })
    }

    pub fn audit_candidate(
        &self,
        hamming: &[u8],
        corpus: &AskPullCorpus,
    ) -> Result<Vec<AskCorpusAuditReading>, String> {
        corpus.validate(self.vocab)?;
        let tensors =
            SafeTensors::deserialize(hamming).map_err(|error| format!("hamming_read: {error}"))?;
        let heads = *u32_values(
            &tensors
                .tensor("heads")
                .map_err(|error| format!("heads: {error}"))?,
            "heads",
            &[1],
        )?
        .first()
        .ok_or_else(|| "heads is empty".to_owned())? as usize;
        if heads != self.heads {
            return Err(format!(
                "candidate heads {heads} differs from source heads {}",
                self.heads
            ));
        }
        let asks = self
            .reads
            .iter()
            .map(|read| {
                let name = format!("d{}.ask", read.depth);
                let tensor = tensors
                    .tensor(&name)
                    .map_err(|error| format!("{name}: {error}"))?;
                Ok((
                    read.depth,
                    f32_values(&tensor, &name, &[self.vocab, self.width])?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        let candidate_metadata = safetensors_metadata(hamming)?;
        if candidate_metadata != self.metadata {
            return Err("candidate Hamming metadata differs from the source".to_owned());
        }
        let expected_non_ask = self
            .original_tensors
            .iter()
            .map(|tensor| tensor.name.as_str())
            .collect::<BTreeSet<_>>();
        let candidate_non_ask = tensors
            .names()
            .into_iter()
            .filter(|name| !name.ends_with(".ask"))
            .collect::<BTreeSet<_>>();
        if candidate_non_ask != expected_non_ask {
            return Err("candidate Hamming tensor inventory differs from the source".to_owned());
        }
        for original in &self.original_tensors {
            let candidate = tensors
                .tensor(&original.name)
                .map_err(|error| format!("{}: {error}", original.name))?;
            if candidate.dtype() != original.dtype
                || candidate.shape() != original.shape
                || candidate.data() != original.data
            {
                return Err(format!(
                    "candidate tensor {} differs from the source",
                    original.name
                ));
            }
        }
        let expected_asks = self
            .reads
            .iter()
            .map(|read| format!("d{}.ask", read.depth))
            .collect::<BTreeSet<_>>();
        let candidate_asks = tensors
            .names()
            .into_iter()
            .filter(|name| name.ends_with(".ask"))
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        if candidate_asks != expected_asks {
            return Err("candidate ask tensor inventory differs from the source depths".to_owned());
        }
        let mut groups: BTreeMap<u32, Vec<&AskPull>> = BTreeMap::new();
        for pull in &corpus.pulls {
            groups.entry(pull.question).or_default().push(pull);
        }
        let tasks = self
            .reads
            .iter()
            .flat_map(|read| {
                groups
                    .iter()
                    .map(move |(&question, pulls)| (read, question, pulls))
            })
            .collect::<Vec<_>>();
        let mut readings = tasks
            .into_par_iter()
            .map(|(read, question, pulls)| {
                let ask = asks
                    .get(&read.depth)
                    .ok_or_else(|| format!("depth {} ask table disappeared", read.depth))?;
                let offset = &ask[question as usize * self.width..][..self.width];
                let zero = vec![0.0f32; self.width];
                let zero_bits = self.objective(read, question, &zero, pulls)?;
                let candidate_bits = self.objective(read, question, offset, pulls)?;
                Ok(AskCorpusAuditReading {
                    question,
                    depth: read.depth,
                    pulls: pulls.len(),
                    pull_weight: pull_weight(pulls)?,
                    target_keys: pulls
                        .iter()
                        .map(|pull| pull.key)
                        .collect::<BTreeSet<_>>()
                        .len(),
                    zero_weighted_bits: zero_bits,
                    candidate_weighted_bits: candidate_bits,
                    improvement_bits: zero_bits.saturating_sub(candidate_bits),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        readings.sort_by_key(|reading| (reading.question, reading.depth));
        Ok(readings)
    }

    fn solve_question(
        &self,
        read: &SolveRead,
        question: u32,
        pulls: &[&AskPull],
        max_evaluations: u64,
    ) -> Result<SolvedQuestion, String> {
        let zero = vec![0.0f32; self.width];
        let mut candidate_evaluations = 1;
        let zero_bits = self.objective(read, question, &zero, pulls)?;
        let question_point = self.table_row(question);
        let targets = pulls.iter().map(|pull| pull.key).collect::<BTreeSet<_>>();
        let mut candidates = vec![(zero.clone(), zero_bits)];
        let mut budget_exhausted = false;
        for key in &targets {
            let key_point = self.table_row(*key);
            let offset = question_point
                .iter()
                .zip(key_point)
                .map(|(&question, &key)| key - question)
                .collect::<Vec<_>>();
            ensure_finite(&offset, "exact candidate")?;
            if candidate_evaluations >= max_evaluations {
                budget_exhausted = true;
                break;
            }
            let bits = self.objective(read, question, &offset, pulls)?;
            candidate_evaluations += 1;
            candidates.push((offset, bits));
        }
        if !budget_exhausted {
            let offset = self.weighted_median_offset(question, pulls)?;
            if candidate_evaluations >= max_evaluations {
                budget_exhausted = true;
            } else {
                let bits = self.objective(read, question, &offset, pulls)?;
                candidate_evaluations += 1;
                candidates.push((offset, bits));
            }
        }
        candidates
            .sort_by(|(a, a_bits), (b, b_bits)| a_bits.cmp(b_bits).then_with(|| offset_cmp(a, b)));
        candidates.dedup_by(|(a, _), (b, _)| a == b);

        let mut coordinate_evaluations = 0u64;
        let mut search_starts = 0u64;
        let (mut best, mut best_bits) = candidates
            .first()
            .cloned()
            .ok_or_else(|| "candidate set is empty".to_owned())?;
        for (start, start_bits) in candidates.into_iter().take(4) {
            if budget_exhausted
                || candidate_evaluations
                    .checked_add(coordinate_evaluations)
                    .ok_or_else(|| "evaluation total overflows".to_owned())?
                    >= max_evaluations
            {
                budget_exhausted = true;
                break;
            }
            let mut offset = start;
            let mut bits = start_bits;
            search_starts += 1;
            if bits == 0 {
                best = offset;
                best_bits = 0;
                break;
            }
            for step in [16384.0f32, 4096.0, 1024.0, 256.0, 64.0, 16.0, 4.0, 1.0] {
                loop {
                    let mut improved = false;
                    for coordinate in 0..self.width {
                        for delta in [step, -step] {
                            if candidate_evaluations
                                .checked_add(coordinate_evaluations)
                                .ok_or_else(|| "evaluation total overflows".to_owned())?
                                >= max_evaluations
                            {
                                budget_exhausted = true;
                                break;
                            }
                            let previous = offset[coordinate];
                            let candidate = previous + delta;
                            if !candidate.is_finite() || candidate == previous {
                                continue;
                            }
                            offset[coordinate] = candidate;
                            let candidate_bits = self.objective(read, question, &offset, pulls)?;
                            coordinate_evaluations += 1;
                            if candidate_bits < bits {
                                bits = candidate_bits;
                                improved = true;
                            } else {
                                offset[coordinate] = previous;
                            }
                        }
                    }
                    if !improved || budget_exhausted {
                        break;
                    }
                }
            }
            if bits < best_bits || (bits == best_bits && offset_cmp(&offset, &best).is_lt()) {
                best = offset;
                best_bits = bits;
            }
        }
        Ok(SolvedQuestion {
            offset: best,
            zero_bits,
            solved_bits: best_bits,
            candidate_evaluations,
            coordinate_evaluations,
            search_starts,
            budget_exhausted,
        })
    }

    fn objective(
        &self,
        read: &SolveRead,
        question: u32,
        offset: &[f32],
        pulls: &[&AskPull],
    ) -> Result<u64, String> {
        if offset.len() != self.width {
            return Err("ask offset width differs".to_owned());
        }
        let query = self
            .table_row(question)
            .iter()
            .zip(offset)
            .map(|(&point, &offset)| point + offset)
            .collect::<Vec<_>>();
        ensure_finite(&query, "question point plus offset")?;
        let mut total = 0u128;
        for pull in pulls {
            let key = self.table_row(pull.key);
            let mut bits = 0u64;
            for head in self.heads / 2..self.heads {
                bits += u64::from(popcount_swar(
                    self.head_code(read, &query, head)? ^ self.head_code(read, key, head)?,
                ));
            }
            total = total
                .checked_add(u128::from(bits) * u128::from(pull.weight))
                .ok_or_else(|| "pull objective overflows".to_owned())?;
        }
        u64::try_from(total).map_err(|_| "pull objective exceeds u64".to_owned())
    }

    fn weighted_median_offset(
        &self,
        question: u32,
        pulls: &[&AskPull],
    ) -> Result<Vec<f32>, String> {
        let question_point = self.table_row(question);
        let mut offset = Vec::with_capacity(self.width);
        for (coordinate, &question_value) in question_point.iter().enumerate() {
            let mut values = pulls
                .iter()
                .map(|pull| (self.table_row(pull.key)[coordinate], pull.weight))
                .collect::<Vec<_>>();
            values.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
            let total = values
                .iter()
                .try_fold(0u64, |sum, (_, weight)| sum.checked_add(u64::from(*weight)))
                .ok_or_else(|| "weighted median overflows".to_owned())?;
            let halfway = total / 2 + total % 2;
            let mut cumulative = 0u64;
            let median = values
                .iter()
                .find_map(|(value, weight)| {
                    cumulative += u64::from(*weight);
                    (cumulative >= halfway).then_some(*value)
                })
                .ok_or_else(|| "weighted median is empty".to_owned())?;
            let value = median - question_value;
            if !value.is_finite() {
                return Err("weighted median offset is not finite".to_owned());
            }
            offset.push(value);
        }
        Ok(offset)
    }

    fn head_code(&self, read: &SolveRead, point: &[f32], head: usize) -> Result<u64, String> {
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

    fn serialize_hamming(&self, tables: &BTreeMap<usize, Vec<f32>>) -> Result<Vec<u8>, String> {
        let mut tensors = self.original_tensors.clone();
        for (&depth, ask) in tables {
            tensors.push(NamedTensor {
                name: format!("d{depth}.ask"),
                dtype: Dtype::F32,
                shape: vec![self.vocab, self.width],
                data: f32_bytes(ask),
            });
        }
        tensors.sort_by(|a, b| a.name.cmp(&b.name));
        let views = tensors
            .iter()
            .map(|tensor| {
                Ok((
                    tensor.name.clone(),
                    TensorView::new(tensor.dtype, tensor.shape.clone(), &tensor.data)
                        .map_err(|error| format!("{}: {error}", tensor.name))?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let bytes = serialize(views, self.metadata.clone()).map_err(|error| error.to_string())?;
        canonicalize_safetensors_header(bytes)
    }
}

fn unit_weight() -> u32 {
    1
}

fn canonical_depth(depth: &str) -> Option<usize> {
    if depth.is_empty()
        || !depth.bytes().all(|byte| byte.is_ascii_digit())
        || (depth.len() > 1 && depth.starts_with('0'))
    {
        return None;
    }
    depth.parse().ok()
}

fn pull_weight(pulls: &[&AskPull]) -> Result<u64, String> {
    pulls
        .iter()
        .try_fold(0u64, |total, pull| {
            total.checked_add(u64::from(pull.weight))
        })
        .ok_or_else(|| "pull weight total overflows".to_owned())
}

fn f32_values(tensor: &TensorView<'_>, name: &str, shape: &[usize]) -> Result<Vec<f32>, String> {
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
    ensure_finite(&values, name)?;
    Ok(values)
}

fn u32_values(tensor: &TensorView<'_>, name: &str, shape: &[usize]) -> Result<Vec<u32>, String> {
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

fn f32_bytes(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn ensure_finite(values: &[f32], what: &str) -> Result<(), String> {
    if let Some(index) = values.iter().position(|value| !value.is_finite()) {
        return Err(format!("{what}[{index}] is not finite"));
    }
    Ok(())
}

fn safetensors_metadata(
    bytes: &[u8],
) -> Result<Option<std::collections::HashMap<String, String>>, String> {
    let len = bytes
        .get(..8)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or_else(|| "safetensors header has no length".to_owned())?;
    let len = usize::try_from(len).map_err(|_| "safetensors header is too large".to_owned())?;
    let header = bytes
        .get(8..8usize.saturating_add(len))
        .ok_or_else(|| "safetensors header is truncated".to_owned())?;
    let value: serde_json::Value =
        serde_json::from_slice(header).map_err(|error| format!("safetensors header: {error}"))?;
    value
        .get("__metadata__")
        .map(|metadata| {
            serde_json::from_value(metadata.clone())
                .map_err(|error| format!("safetensors metadata: {error}"))
        })
        .transpose()
}

fn canonicalize_safetensors_header(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    let len = bytes
        .get(..8)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or_else(|| "safetensors header has no length".to_owned())?;
    let len = usize::try_from(len).map_err(|_| "safetensors header is too large".to_owned())?;
    let data_start = 8usize
        .checked_add(len)
        .filter(|&end| end <= bytes.len())
        .ok_or_else(|| "safetensors header is truncated".to_owned())?;
    let value: serde_json::Value = serde_json::from_slice(&bytes[8..data_start])
        .map_err(|error| format!("safetensors header: {error}"))?;
    let mut header =
        serde_json::to_vec(&value).map_err(|error| format!("safetensors header: {error}"))?;
    let aligned_len = header.len().next_multiple_of(8);
    header.resize(aligned_len, b' ');
    let aligned_len =
        u64::try_from(aligned_len).map_err(|_| "safetensors header is too large".to_owned())?;
    let mut canonical = Vec::with_capacity(8 + header.len() + bytes.len() - data_start);
    canonical.extend_from_slice(&aligned_len.to_le_bytes());
    canonical.extend_from_slice(&header);
    canonical.extend_from_slice(&bytes[data_start..]);
    Ok(canonical)
}

fn offset_cmp(a: &[f32], b: &[f32]) -> std::cmp::Ordering {
    a.iter()
        .map(|value| value.to_bits())
        .cmp(b.iter().map(|value| value.to_bits()))
}

fn offset_abs_sum_q16(offset: &[f32]) -> Result<u64, String> {
    offset.iter().try_fold(0u64, |sum, &value| {
        let q16 = (f64::from(value).abs() * 65_536.0).round();
        // `u64::MAX as f64` rounds up to 2^64, which is already exclusive.
        if !q16.is_finite() || q16 >= 18_446_744_073_709_551_616.0 {
            return Err("ask offset magnitude is unrepresentable".to_owned());
        }
        sum.checked_add(q16 as u64)
            .ok_or_else(|| "ask offset magnitude overflows".to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ask_offset_audit::TrainedAskOffsets;

    fn u32_bytes(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn tensor_file(tensors: Vec<(&str, Dtype, Vec<usize>, Vec<u8>)>) -> Vec<u8> {
        tensor_file_owned(
            tensors
                .into_iter()
                .map(|(name, dtype, shape, data)| (name.to_owned(), dtype, shape, data))
                .collect(),
            None,
        )
    }

    fn tensor_file_with_metadata(
        tensors: Vec<(&str, Dtype, Vec<usize>, Vec<u8>)>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Vec<u8> {
        tensor_file_owned(
            tensors
                .into_iter()
                .map(|(name, dtype, shape, data)| (name.to_owned(), dtype, shape, data))
                .collect(),
            metadata,
        )
    }

    fn tensor_file_owned(
        tensors: Vec<(String, Dtype, Vec<usize>, Vec<u8>)>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Vec<u8> {
        let views = tensors
            .iter()
            .map(|(name, dtype, shape, data)| {
                (
                    name.clone(),
                    TensorView::new(*dtype, shape.clone(), data).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        serialize(views, metadata).unwrap()
    }

    fn complete_read_tensors(
        width: usize,
        heads: usize,
        depths: &[usize],
        vocab: usize,
    ) -> Vec<(String, Dtype, Vec<usize>, Vec<u8>)> {
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        depths
            .iter()
            .flat_map(|depth| {
                [
                    (
                        format!("d{depth}.query_rotation"),
                        Dtype::F32,
                        vec![width / 4, 4],
                        f32_bytes(&rotation),
                    ),
                    (
                        format!("d{depth}.key_rotation"),
                        Dtype::F32,
                        vec![width / 4, 4],
                        f32_bytes(&rotation),
                    ),
                    (
                        format!("d{depth}.value_rotation"),
                        Dtype::F32,
                        vec![width / 4, 4],
                        f32_bytes(&rotation),
                    ),
                    (
                        format!("d{depth}.margin"),
                        Dtype::F32,
                        vec![heads],
                        f32_bytes(&vec![2.0; heads]),
                    ),
                    (
                        format!("d{depth}.weight"),
                        Dtype::F32,
                        vec![2, width],
                        f32_bytes(&vec![0.0; 2 * width]),
                    ),
                    (
                        format!("d{depth}.ask"),
                        Dtype::F32,
                        vec![vocab, width],
                        f32_bytes(&vec![0.0; vocab * width]),
                    ),
                ]
            })
            .collect()
    }

    fn fixture() -> (Vec<u8>, Vec<u8>, AskPullCorpus) {
        let (vocab, width) = (64usize, 8usize);
        let mut table = vec![1.0f32; vocab * width];
        table[17 * width..18 * width].fill(-1.0);
        let model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&table),
        )]);
        let mut tensors = vec![(
            "heads".to_owned(),
            Dtype::U32,
            vec![1],
            [2u32].into_iter().flat_map(u32::to_le_bytes).collect(),
        )];
        tensors.extend(complete_read_tensors(width, 2, &[2, 6], vocab));
        let hamming = tensor_file_owned(tensors, None);
        let corpus = AskPullCorpus {
            schema: PULL_CORPUS_SCHEMA.to_owned(),
            pulls: vec![AskPull {
                question: 33,
                key: 17,
                weight: 1,
            }],
        };
        (model, hamming, corpus)
    }

    #[test]
    fn solves_b2_style_pull_and_preserves_hamming_tensors() {
        let (model, hamming, corpus) = fixture();
        let basis = AskOffsetSolveBasis::from_bytes(&model, &hamming).unwrap();
        let solved = basis.solve(&corpus, 100_000).unwrap();
        assert_eq!(solved.readings.len(), 2);
        for reading in &solved.readings {
            assert_eq!(reading.zero_weighted_bits, 4);
            assert_eq!(reading.solved_weighted_bits, 0);
            assert_eq!(reading.improvement_bits, 4);
            assert!(!reading.budget_exhausted);
            assert!(reading.candidate_evaluations >= 3);
            assert_eq!(
                reading.total_evaluations,
                reading.candidate_evaluations + reading.coordinate_evaluations
            );
        }
        let corpus_audit = basis
            .audit_candidate(&solved.hamming_bytes, &corpus)
            .unwrap();
        assert_eq!(corpus_audit.len(), 2);
        for reading in &corpus_audit {
            assert_eq!(reading.zero_weighted_bits, 4);
            assert_eq!(reading.candidate_weighted_bits, 0);
            assert_eq!(reading.improvement_bits, 4);
        }
        let parsed = TrainedAskOffsets::from_bytes(&model, &solved.hamming_bytes).unwrap();
        let audit = parsed.audit_relations(16..32, 16).unwrap();
        let target = audit.iter().find(|reading| reading.relation == 17).unwrap();
        assert!(target.cells.iter().all(|cell| cell.trained_bits == 0));
    }

    #[test]
    fn repeated_solve_is_byte_identical_and_multiple_targets_do_not_regress() {
        let (model, hamming, mut corpus) = fixture();
        corpus.pulls[0].weight = 3;
        corpus.pulls.push(AskPull {
            question: 33,
            key: 18,
            weight: 1,
        });
        let basis = AskOffsetSolveBasis::from_bytes(&model, &hamming).unwrap();
        let first = basis.solve(&corpus, 20_000).unwrap();
        let second = basis.solve(&corpus, 20_000).unwrap();
        assert_eq!(first.hamming_bytes, second.hamming_bytes);
        let limited = basis.solve(&corpus, 1).unwrap();
        assert!(limited
            .readings
            .iter()
            .all(|reading| reading.total_evaluations == 1 && reading.budget_exhausted));
        let candidate_cutoff = basis.solve(&corpus, 4).unwrap();
        assert!(candidate_cutoff.readings.iter().all(|reading| {
            reading.candidate_evaluations == 4
                && reading.coordinate_evaluations == 0
                && reading.total_evaluations == 4
                && reading.budget_exhausted
                && reading.solved_weighted_bits == 4
        }));
        let coordinate_cutoff = basis.solve(&corpus, 5).unwrap();
        assert!(coordinate_cutoff.readings.iter().all(|reading| {
            reading.candidate_evaluations == 4
                && reading.coordinate_evaluations == 1
                && reading.total_evaluations == 5
                && reading.budget_exhausted
        }));
        assert!(basis.solve(&corpus, 0).is_err());
        assert!(first
            .readings
            .iter()
            .all(|reading| reading.solved_weighted_bits <= reading.zero_weighted_bits));
        assert!(first
            .readings
            .iter()
            .any(|reading| reading.target_keys == 2));
        assert!(first
            .readings
            .iter()
            .all(|reading| reading.zero_weighted_bits == 12));
        assert!(first
            .readings
            .iter()
            .all(|reading| reading.solved_weighted_bits == 4));
        let corpus_audit = basis
            .audit_candidate(&first.hamming_bytes, &corpus)
            .unwrap();
        assert!(corpus_audit.iter().all(|reading| {
            reading.zero_weighted_bits == 12
                && reading.candidate_weighted_bits == 4
                && reading.improvement_bits == 8
        }));
    }

    #[test]
    fn malformed_corpora_are_refused() {
        let (model, hamming, _) = fixture();
        let basis = AskOffsetSolveBasis::from_bytes(&model, &hamming).unwrap();
        for pull in [
            AskPull {
                question: 64,
                key: 17,
                weight: 1,
            },
            AskPull {
                question: 33,
                key: 17,
                weight: 0,
            },
        ] {
            let corpus = AskPullCorpus {
                schema: PULL_CORPUS_SCHEMA.to_owned(),
                pulls: vec![pull],
            };
            assert!(basis.solve(&corpus, 10).is_err());
        }
        assert!(basis
            .solve(
                &AskPullCorpus {
                    schema: PULL_CORPUS_SCHEMA.to_owned(),
                    pulls: vec![],
                },
                10,
            )
            .is_err());
    }

    #[test]
    fn relation_mapping_and_q16_overflow_are_refused() {
        assert!(AskPullCorpus::b2_relations(u32::MAX - 1..u32::MAX, 2)
            .unwrap_err()
            .contains("overflows"));
        assert_eq!(
            AskPullCorpus::b2_relations(16..18, 16)
                .unwrap()
                .pulls
                .iter()
                .map(|pull| pull.question)
                .collect::<Vec<_>>(),
            vec![32, 33]
        );
        assert!(offset_abs_sum_q16(&[281_474_976_710_656.0]).is_err());
        assert!(offset_abs_sum_q16(&[281_474_943_156_224.0]).is_ok());
    }

    #[test]
    fn malformed_and_nonfinite_checkpoints_are_refused() {
        let (vocab, width, heads) = (4usize, 8usize, 2usize);
        let table = vec![0.5f32; vocab * width];
        let model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&table),
        )]);
        let valid_hamming = |head_count: u32| {
            let mut tensors = vec![(
                "heads".to_owned(),
                Dtype::U32,
                vec![1],
                u32_bytes(&[head_count]),
            )];
            tensors.extend(complete_read_tensors(
                width,
                head_count as usize,
                &[2, 6],
                vocab,
            ));
            tensor_file_owned(tensors, None)
        };
        assert!(AskOffsetSolveBasis::from_bytes(&model, &valid_hamming(heads as u32)).is_ok());

        let mut nonfinite_model_values = table.clone();
        nonfinite_model_values[0] = f32::NAN;
        let nonfinite_model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&nonfinite_model_values),
        )]);
        assert!(
            AskOffsetSolveBasis::from_bytes(&nonfinite_model, &valid_hamming(heads as u32))
                .err()
                .expect("non-finite table")
                .contains("embedding.weight[0] is not finite")
        );

        assert!(AskOffsetSolveBasis::from_bytes(&model, &valid_hamming(1))
            .err()
            .expect("one head")
            .contains("1 heads over width 8 is invalid"));

        let mut nonfinite_tensors = vec![(
            "heads".to_owned(),
            Dtype::U32,
            vec![1],
            u32_bytes(&[heads as u32]),
        )];
        nonfinite_tensors.extend(complete_read_tensors(width, heads, &[2, 6], vocab));
        let rotation = nonfinite_tensors
            .iter_mut()
            .find(|(name, ..)| name == "d2.key_rotation")
            .unwrap();
        rotation.3 = f32_bytes(&vec![f32::INFINITY; width]);
        let nonfinite_hamming = tensor_file_owned(nonfinite_tensors, None);
        assert!(AskOffsetSolveBasis::from_bytes(&model, &nonfinite_hamming)
            .err()
            .expect("non-finite rotation")
            .contains("d2.key_rotation[0] is not finite"));

        let mut partial_tensors = vec![(
            "heads".to_owned(),
            Dtype::U32,
            vec![1],
            u32_bytes(&[heads as u32]),
        )];
        partial_tensors.extend(complete_read_tensors(width, heads, &[2, 6], vocab));
        partial_tensors.push((
            "d8.query_rotation".to_owned(),
            Dtype::F32,
            vec![width / 4, 4],
            f32_bytes(&vec![1.0; width]),
        ));
        let partial_hamming = tensor_file_owned(partial_tensors, None);
        assert!(AskOffsetSolveBasis::from_bytes(&model, &partial_hamming)
            .err()
            .expect("unexpected depth")
            .contains("Claude-330 Hamming depths are [2, 6]"));

        let mut partial_ask_tensors = vec![(
            "heads".to_owned(),
            Dtype::U32,
            vec![1],
            u32_bytes(&[heads as u32]),
        )];
        partial_ask_tensors.extend(complete_read_tensors(width, heads, &[2, 6], vocab));
        partial_ask_tensors.retain(|(name, ..)| name != "d6.ask");
        let partial_ask_hamming = tensor_file_owned(partial_ask_tensors, None);
        assert!(
            AskOffsetSolveBasis::from_bytes(&model, &partial_ask_hamming)
                .err()
                .expect("partial ask depths")
                .contains("ask tables cover depths")
        );

        let mut aliased_tensors = vec![(
            "heads".to_owned(),
            Dtype::U32,
            vec![1],
            u32_bytes(&[heads as u32]),
        )];
        aliased_tensors.extend(complete_read_tensors(width, heads, &[2, 6], vocab));
        aliased_tensors.push((
            "d02.key_rotation".to_owned(),
            Dtype::F32,
            vec![width / 4, 4],
            f32_bytes(&vec![1.0; width]),
        ));
        assert!(
            AskOffsetSolveBasis::from_bytes(&model, &tensor_file_owned(aliased_tensors, None))
                .err()
                .expect("noncanonical depth")
                .contains("unknown Claude-330 Hamming tensor d02.key_rotation")
        );

        let mut type_spin_tensors = vec![(
            "heads".to_owned(),
            Dtype::U32,
            vec![1],
            u32_bytes(&[heads as u32]),
        )];
        type_spin_tensors.extend(complete_read_tensors(width, heads, &[2, 6], vocab));
        type_spin_tensors.push((
            "d2.step_rotation".to_owned(),
            Dtype::F32,
            vec![width / 4, 4],
            f32_bytes(&vec![1.0; width]),
        ));
        let type_spin_hamming = tensor_file_owned(type_spin_tensors, None);
        assert!(AskOffsetSolveBasis::from_bytes(&model, &type_spin_hamming)
            .err()
            .expect("type spin")
            .contains("mutually exclusive"));
    }

    #[test]
    fn required_hamming_inventory_copy_keys_and_candidate_asks_are_validated() {
        let (vocab, width, heads) = (4usize, 8usize, 2usize);
        let model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&vec![0.5; vocab * width]),
        )]);
        let complete = || {
            let mut tensors = vec![(
                "heads".to_owned(),
                Dtype::U32,
                vec![1],
                u32_bytes(&[heads as u32]),
            )];
            tensors.extend(complete_read_tensors(width, heads, &[2, 6], vocab));
            tensors
        };

        let mut missing_query = complete();
        missing_query.retain(|(name, ..)| name != "d2.query_rotation");
        assert!(
            AskOffsetSolveBasis::from_bytes(&model, &tensor_file_owned(missing_query, None))
                .err()
                .expect("missing query rotation")
                .contains("d2.query_rotation")
        );

        let mut unknown_tensor = complete();
        unknown_tensor.push((
            "d2.unknown".to_owned(),
            Dtype::U32,
            vec![1],
            u32_bytes(&[1]),
        ));
        assert!(
            AskOffsetSolveBasis::from_bytes(&model, &tensor_file_owned(unknown_tensor, None))
                .err()
                .expect("unknown Hamming tensor")
                .contains("unknown Claude-330 Hamming tensor d2.unknown")
        );

        let mut nonfinite_margin = complete();
        let margin = nonfinite_margin
            .iter_mut()
            .find(|(name, ..)| name == "d2.margin")
            .unwrap();
        margin.3 = f32_bytes(&vec![f32::NAN; heads]);
        assert!(AskOffsetSolveBasis::from_bytes(
            &model,
            &tensor_file_owned(nonfinite_margin, None)
        )
        .err()
        .expect("non-finite margin")
        .contains("d2.margin[0] is not finite"));

        let mut invalid_copy_key = complete();
        invalid_copy_key.push(("copy_key".to_owned(), Dtype::U32, vec![1], u32_bytes(&[3])));
        assert!(AskOffsetSolveBasis::from_bytes(
            &model,
            &tensor_file_owned(invalid_copy_key, None)
        )
        .err()
        .expect("invalid copy key")
        .contains("copy_key 3 is not 1 or 2"));

        let mut narrow_copy_key = complete();
        narrow_copy_key.push(("copy_key".to_owned(), Dtype::U32, vec![1], u32_bytes(&[2])));
        assert!(
            AskOffsetSolveBasis::from_bytes(&model, &tensor_file_owned(narrow_copy_key, None))
                .err()
                .expect("copy key 2 on a four-coordinate head")
                .contains("whole 8-coordinate blocks")
        );

        let basis =
            AskOffsetSolveBasis::from_bytes(&model, &tensor_file_owned(complete(), None)).unwrap();
        let corpus = AskPullCorpus {
            schema: PULL_CORPUS_SCHEMA.to_owned(),
            pulls: vec![AskPull {
                question: 3,
                key: 1,
                weight: 1,
            }],
        };
        let mut changed_margin_candidate = complete();
        let margin = changed_margin_candidate
            .iter_mut()
            .find(|(name, ..)| name == "d2.margin")
            .unwrap();
        margin.3 = f32_bytes(&vec![3.0; heads]);
        assert!(basis
            .audit_candidate(&tensor_file_owned(changed_margin_candidate, None), &corpus)
            .unwrap_err()
            .contains("candidate tensor d2.margin differs from the source"));
        let mut nonfinite_candidate = complete();
        let ask = nonfinite_candidate
            .iter_mut()
            .find(|(name, ..)| name == "d2.ask")
            .unwrap();
        ask.3 = f32_bytes(&vec![f32::NAN; vocab * width]);
        assert!(basis
            .audit_candidate(&tensor_file_owned(nonfinite_candidate, None), &corpus)
            .unwrap_err()
            .contains("d2.ask[0] is not finite"));
    }

    #[test]
    fn metadata_is_preserved_and_serialized_canonically() {
        let (model, hamming, corpus) = fixture();
        let original = SafeTensors::deserialize(&hamming).unwrap();
        let tensors = original.names().into_iter().map(|name| {
            let tensor = original.tensor(name).unwrap();
            (
                name,
                tensor.dtype(),
                tensor.shape().to_vec(),
                tensor.data().to_vec(),
            )
        });
        let tensors = tensors.collect::<Vec<_>>();
        let mut metadata_a = std::collections::HashMap::new();
        metadata_a.insert("z-last".to_owned(), "2".to_owned());
        metadata_a.insert("a-first".to_owned(), "1".to_owned());
        let mut metadata_b = std::collections::HashMap::new();
        metadata_b.insert("a-first".to_owned(), "1".to_owned());
        metadata_b.insert("z-last".to_owned(), "2".to_owned());
        let hamming_a = tensor_file_with_metadata(tensors.clone(), Some(metadata_a));
        let hamming_b = tensor_file_with_metadata(tensors, Some(metadata_b));
        let basis_a = AskOffsetSolveBasis::from_bytes(&model, &hamming_a).unwrap();
        let basis_b = AskOffsetSolveBasis::from_bytes(&model, &hamming_b).unwrap();
        let solved_a = basis_a.solve(&corpus, 10_000).unwrap();
        let solved_b = basis_b.solve(&corpus, 10_000).unwrap();
        assert_eq!(solved_a.hamming_bytes, solved_b.hamming_bytes);
        assert_eq!(
            safetensors_metadata(&solved_a.hamming_bytes).unwrap(),
            Some(std::collections::HashMap::from([
                ("z-last".to_owned(), "2".to_owned()),
                ("a-first".to_owned(), "1".to_owned()),
            ]))
        );
        assert_eq!(
            canonicalize_safetensors_header(solved_a.hamming_bytes.clone()).unwrap(),
            solved_a.hamming_bytes
        );
    }

    #[test]
    #[ignore = "release benchmark"]
    fn release_width_256_two_depth_solver_benchmark() {
        let (vocab, width, heads) = (4_096usize, 256usize, 4usize);
        let table = (0..vocab * width)
            .map(|index| ((index.wrapping_mul(37).wrapping_add(11) % 257) as f32 - 128.0) / 64.0)
            .collect::<Vec<_>>();
        let model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&table),
        )]);
        let mut tensors = vec![(
            "heads".to_owned(),
            Dtype::U32,
            vec![1],
            [heads as u32]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect(),
        )];
        tensors.extend(complete_read_tensors(width, heads, &[2, 6], vocab));
        let hamming = tensor_file_owned(tensors, None);
        let basis = AskOffsetSolveBasis::from_bytes(&model, &hamming).unwrap();
        let mut corpus = AskPullCorpus::b2_relations(16..32, 16).unwrap();
        let second_targets = corpus
            .pulls
            .iter()
            .map(|pull| AskPull {
                question: pull.question,
                key: pull.key + 16,
                weight: 1,
            })
            .collect::<Vec<_>>();
        corpus.pulls.extend(second_targets);
        let started = std::time::Instant::now();
        let solved = basis.solve(&corpus, 200_000).unwrap();
        let elapsed = started.elapsed();
        assert!(solved
            .readings
            .iter()
            .all(|reading| reading.solved_weighted_bits <= reading.zero_weighted_bits));
        eprintln!(
            "width=256 depths=2 questions=16 evaluations={} exhausted={} wall_ms={}",
            solved
                .readings
                .iter()
                .map(|reading| reading.total_evaluations)
                .sum::<u64>(),
            solved
                .readings
                .iter()
                .filter(|reading| reading.budget_exhausted)
                .count(),
            elapsed.as_millis()
        );
    }
}
