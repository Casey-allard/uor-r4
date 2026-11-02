//! Deterministic frozen-query rank-margin training of the shared token table.
//!
//! One accepted update moves the target row toward the query and the hardest
//! admitted rival away from it. The complete patched artifact is parsed and
//! rebuilt immediately, so table-derived Hamming keys cannot remain stale.

use uor_r4_integer::stack::IntegerStackModel;

use crate::{CpuFbsForward, CpuFbsHead, CpuTrainError, CELL_COUNT, TOP_K, VOCABULARY};

const MAGIC: &[u8; 16] = b"UOR-R4-CPU-TRAIN";
const CHECKPOINT_VERSION: u32 = 1;

type Result<T> = std::result::Result<T, CpuTrainError>;

fn invalid(message: impl Into<String>) -> CpuTrainError {
    CpuTrainError::Invalid(message.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RankMarginConfig {
    /// Required per-coordinate separation between the target and its hardest wrong rival.
    pub margin_q12: i32,
    /// A coordinate moves by this many Q12 units.
    pub step_q12: i16,
    /// Bidirectional absolute bound for a trained table entry.
    pub parameter_limit_q12: i16,
}

impl Default for RankMarginConfig {
    fn default() -> Self {
        Self {
            margin_q12: 1,
            step_q12: 1,
            parameter_limit_q12: 8_192,
        }
    }
}

impl RankMarginConfig {
    fn validate(self) -> Result<()> {
        if self.margin_q12 <= 0 {
            return Err(invalid("rank margin must be positive"));
        }
        if self.step_q12 <= 0 || self.parameter_limit_q12 <= 0 {
            return Err(invalid(
                "rank-margin step and parameter limit must be positive",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RankMarginSample {
    /// Causal prefix; the final token's query predicts `target`.
    pub input: Vec<u32>,
    pub target: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankMarginReading {
    pub target_distance: i64,
    pub target_rank: usize,
    pub margin_loss: i64,
    pub top1: u32,
    pub top8: Vec<u32>,
    pub target_admitted: bool,
    pub target_cell: u16,
    pub admitted_cells: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankMarginStep {
    pub sample: usize,
    pub target: u32,
    pub updated: bool,
    pub coordinate: Option<usize>,
    pub rival: Option<u32>,
    pub before: RankMarginReading,
    pub after: RankMarginReading,
}

#[derive(Clone)]
pub struct CpuRankMarginTrainer {
    artifact: Vec<u8>,
    forward: CpuFbsForward,
    samples: Vec<RankMarginSample>,
    sample_fingerprint: [u8; 32],
    config: RankMarginConfig,
    seed: u64,
    step: u64,
    epoch: u64,
    cursor: usize,
    order: Vec<usize>,
}

impl CpuRankMarginTrainer {
    pub fn from_artifact(
        artifact: &[u8],
        samples: Vec<RankMarginSample>,
        config: RankMarginConfig,
        seed: u64,
    ) -> Result<Self> {
        config.validate()?;
        if samples.is_empty() {
            return Err(invalid("rank-margin training needs at least one sample"));
        }
        validate_training_model(artifact)?;
        let forward = CpuFbsForward::from_artifact(artifact)
            .map_err(|error| invalid(format!("rank-margin artifact does not load: {error}")))?;
        validate_samples(&samples, &forward)?;
        let fingerprint = sample_fingerprint(&samples);
        let order = shuffled_order(samples.len(), seed, 0);
        Ok(Self {
            artifact: artifact.to_vec(),
            forward,
            samples,
            sample_fingerprint: fingerprint,
            config,
            seed,
            step: 0,
            epoch: 0,
            cursor: 0,
            order,
        })
    }

    pub fn config(&self) -> RankMarginConfig {
        self.config
    }

    pub fn step_count(&self) -> u64 {
        self.step
    }

    pub fn artifact(&self) -> &[u8] {
        &self.artifact
    }

    pub fn table(&self) -> &[i16] {
        self.forward.token_table()
    }

    pub fn train_steps(&mut self, count: usize) -> Result<Vec<RankMarginStep>> {
        let mut steps = Vec::with_capacity(count);
        for _ in 0..count {
            steps.push(self.train_step()?);
        }
        Ok(steps)
    }

    /// Exact writer readings for one causal sample, without changing the table or clocks.
    pub fn evaluate_sample(&mut self, sample: &RankMarginSample) -> Result<RankMarginReading> {
        validate_samples(std::slice::from_ref(sample), &self.forward)?;
        let query = self.query_for(sample)?;
        self.reading(&query, sample.target)
    }

    pub fn train_step(&mut self) -> Result<RankMarginStep> {
        if self.cursor == self.order.len() {
            self.epoch = self
                .epoch
                .checked_add(1)
                .ok_or_else(|| invalid("training epoch counter exhausted"))?;
            self.cursor = 0;
            self.order = shuffled_order(self.samples.len(), self.seed, self.epoch);
        }
        let sample_index = self.order[self.cursor];
        let sample = self.samples[sample_index].clone();
        let query = self.query_for(&sample)?;
        let step = self.apply_query(sample_index, &query, sample.target)?;
        self.step = self
            .step
            .checked_add(1)
            .ok_or_else(|| invalid("training step counter exhausted"))?;
        self.cursor += 1;
        Ok(step)
    }

    pub fn checkpoint(&self) -> Result<Vec<u8>> {
        let order_len =
            u32::try_from(self.order.len()).map_err(|_| invalid("sample order exceeds u32"))?;
        let artifact_len = u64::try_from(self.artifact.len())
            .map_err(|_| invalid("artifact length exceeds u64"))?;
        let mut out = Vec::with_capacity(128 + self.order.len() * 4 + self.artifact.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&CHECKPOINT_VERSION.to_le_bytes());
        out.extend_from_slice(&self.config.margin_q12.to_le_bytes());
        out.extend_from_slice(&self.config.step_q12.to_le_bytes());
        out.extend_from_slice(&self.config.parameter_limit_q12.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&self.seed.to_le_bytes());
        out.extend_from_slice(&self.step.to_le_bytes());
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&(self.cursor as u64).to_le_bytes());
        out.extend_from_slice(&self.sample_fingerprint);
        out.extend_from_slice(&order_len.to_le_bytes());
        for &index in &self.order {
            out.extend_from_slice(
                &u32::try_from(index)
                    .map_err(|_| invalid("sample index exceeds u32"))?
                    .to_le_bytes(),
            );
        }
        out.extend_from_slice(&artifact_len.to_le_bytes());
        out.extend_from_slice(&self.artifact);
        Ok(out)
    }

    pub fn resume(checkpoint: &[u8], samples: Vec<RankMarginSample>) -> Result<Self> {
        let mut input = Reader::new(checkpoint);
        if input.take(MAGIC.len())? != MAGIC {
            return Err(invalid("rank-margin checkpoint magic differs"));
        }
        if input.u32()? != CHECKPOINT_VERSION {
            return Err(invalid("rank-margin checkpoint version differs"));
        }
        let config = RankMarginConfig {
            margin_q12: input.i32()?,
            step_q12: input.i16()?,
            parameter_limit_q12: input.i16()?,
        };
        config.validate()?;
        if input.u32()? != 0 {
            return Err(invalid("rank-margin checkpoint reserved field is nonzero"));
        }
        let seed = input.u64()?;
        let step = input.u64()?;
        let epoch = input.u64()?;
        let cursor = usize::try_from(input.u64()?).map_err(|_| invalid("cursor exceeds usize"))?;
        let fingerprint: [u8; 32] = input
            .take(32)?
            .try_into()
            .map_err(|_| invalid("sample fingerprint length differs"))?;
        if fingerprint != sample_fingerprint(&samples) {
            return Err(invalid("rank-margin checkpoint sample set differs"));
        }
        let order_len =
            usize::try_from(input.u32()?).map_err(|_| invalid("order length exceeds usize"))?;
        if order_len != samples.len() {
            return Err(invalid("rank-margin checkpoint order length differs"));
        }
        let mut order = Vec::with_capacity(order_len);
        let mut seen = vec![false; samples.len()];
        for _ in 0..order_len {
            let index =
                usize::try_from(input.u32()?).map_err(|_| invalid("order index exceeds usize"))?;
            if index >= samples.len() {
                return Err(invalid("rank-margin checkpoint order is out of range"));
            }
            if std::mem::replace(&mut seen[index], true) {
                return Err(invalid("rank-margin checkpoint order repeats a sample"));
            }
            order.push(index);
        }
        let artifact_len =
            usize::try_from(input.u64()?).map_err(|_| invalid("artifact length exceeds usize"))?;
        let artifact = input.take(artifact_len)?.to_vec();
        if !input.done() {
            return Err(invalid("rank-margin checkpoint has trailing bytes"));
        }
        if order.is_empty() {
            return Err(invalid("rank-margin checkpoint has no sample order"));
        }
        if cursor > order.len() {
            return Err(invalid("rank-margin checkpoint cursor is out of range"));
        }
        validate_training_model(&artifact)?;
        let forward = CpuFbsForward::from_artifact(&artifact)
            .map_err(|error| invalid(format!("rank-margin checkpoint artifact: {error}")))?;
        validate_samples(&samples, &forward)?;
        if forward.vocabulary() != VOCABULARY {
            return Err(invalid("rank-margin checkpoint table is empty"));
        }
        Ok(Self {
            artifact,
            forward,
            samples,
            sample_fingerprint: fingerprint,
            config,
            seed,
            step,
            epoch,
            cursor,
            order,
        })
    }

    pub(crate) fn query_for(&mut self, sample: &RankMarginSample) -> Result<Vec<i32>> {
        self.forward.reset();
        let mut step = None;
        for &token in &sample.input {
            step = Some(
                self.forward
                    .step(token, CpuFbsHead::Writer)
                    .map_err(|error| invalid(format!("rank-margin forward: {error}")))?,
            );
        }
        step.map(|step| step.query_q12)
            .ok_or_else(|| invalid("training sample input is empty"))
    }

    fn apply_query(
        &mut self,
        sample_index: usize,
        query: &[i32],
        target: u32,
    ) -> Result<RankMarginStep> {
        let before = self.reading(query, target)?;
        let Some(rival) = before.top8.iter().copied().find(|&token| token != target) else {
            return Ok(RankMarginStep {
                sample: sample_index,
                target,
                updated: false,
                coordinate: None,
                rival: None,
                before: before.clone(),
                after: before,
            });
        };
        if before.margin_loss == 0 || !before.target_admitted {
            return Ok(RankMarginStep {
                sample: sample_index,
                target,
                updated: false,
                coordinate: None,
                rival: Some(rival),
                before: before.clone(),
                after: before,
            });
        }

        let width = self.forward.width();
        let table = self.forward.token_table();
        let target_row = &table[target as usize * width..][..width];
        let rival_row = &table[rival as usize * width..][..width];
        let interior_start = 8;
        let interior_end = width.saturating_sub(8);
        if interior_start >= interior_end {
            return Err(invalid("table has no mutable interior coordinate"));
        }
        let coordinate = (interior_start..interior_end)
            .map(|coordinate| {
                (
                    i64::from(query[coordinate].abs_diff(i32::from(target_row[coordinate])))
                        + i64::from(query[coordinate].abs_diff(i32::from(rival_row[coordinate]))),
                    coordinate,
                )
            })
            .max_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)));
        let Some((_, coordinate)) = coordinate else {
            return Err(invalid("table has no mutable interior coordinate"));
        };
        if query[coordinate] == i32::from(target_row[coordinate])
            && query[coordinate] == i32::from(rival_row[coordinate])
        {
            return Ok(RankMarginStep {
                sample: sample_index,
                target,
                updated: false,
                coordinate: Some(coordinate),
                rival: Some(rival),
                before: before.clone(),
                after: before,
            });
        }

        let mut proposed = table.to_vec();
        proposed[target as usize * width + coordinate] = move_toward(
            target_row[coordinate],
            query[coordinate],
            self.config.step_q12,
            self.config.parameter_limit_q12,
        );
        proposed[rival as usize * width + coordinate] = move_away(
            rival_row[coordinate],
            query[coordinate],
            self.config.step_q12,
            self.config.parameter_limit_q12,
        );
        let patched = patch_writer_table(&self.artifact, &proposed)?;
        let forward = CpuFbsForward::from_artifact(&patched)
            .map_err(|error| invalid(format!("patched rank-margin artifact: {error}")))?;
        let mut probe = Self {
            artifact: patched.clone(),
            forward,
            samples: self.samples.clone(),
            sample_fingerprint: self.sample_fingerprint,
            config: self.config,
            seed: self.seed,
            step: self.step,
            epoch: self.epoch,
            cursor: self.cursor,
            order: self.order.clone(),
        };
        let after_query = probe.query_for(&probe.samples[sample_index].clone())?;
        let after = probe.reading(&after_query, target)?;
        let accepted = after.margin_loss < before.margin_loss
            && (!before.target_admitted || after.target_admitted);
        if accepted {
            self.artifact = patched;
            self.forward = probe.forward;
        }
        Ok(RankMarginStep {
            sample: sample_index,
            target,
            updated: accepted,
            coordinate: Some(coordinate),
            rival: Some(rival),
            before,
            after,
        })
    }

    pub(crate) fn reading(&self, query: &[i32], target: u32) -> Result<RankMarginReading> {
        if query.len() != self.forward.width() {
            return Err(invalid("rank-margin query width differs from the table"));
        }
        if target as usize >= self.forward.vocabulary() {
            return Err(invalid("rank-margin target is outside the vocabulary"));
        }
        let width = query.len();
        let table = self.forward.token_table();
        let vocab = self.forward.vocabulary();
        let mut distances = Vec::with_capacity(vocab);
        for token in 0..vocab {
            let row = &table[token * width..][..width];
            let distance = query.iter().zip(row).fold(0i64, |sum, (&value, &point)| {
                sum.wrapping_add(i64::from(value.abs_diff(i32::from(point))))
            });
            distances.push(distance);
        }
        let target_distance = distances[target as usize];
        let target_rank = 1 + distances
            .iter()
            .enumerate()
            .filter(|&(token, &distance)| {
                distance < target_distance
                    || (distance == target_distance && token < target as usize)
            })
            .count();

        let mut cells = Vec::with_capacity(CELL_COUNT);
        for cell in 0..CELL_COUNT {
            let first = ((query[..8]
                .iter()
                .zip(&self.forward.writer_roots()[cell / 16]))
            .fold(0i64, |sum, (&value, &point)| {
                sum.wrapping_add(i64::from(value.abs_diff(point)))
            }))
            .wrapping_neg();
            let last = ((query[width - 8..]
                .iter()
                .zip(&self.forward.writer_roots()[cell % 16]))
            .fold(0i64, |sum, (&value, &point)| {
                sum.wrapping_add(i64::from(value.abs_diff(point)))
            }))
            .wrapping_neg();
            cells.push((first.wrapping_add(last), cell));
        }
        cells.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
        let admitted: Vec<u32> = cells
            .iter()
            .take(self.forward.kept_cell_count())
            .map(|&(_, cell)| cell as u32)
            .collect();
        let target_cell = (target % CELL_COUNT as u32) as u16;
        let target_admitted = admitted.contains(&u32::from(target_cell));

        let mut ranked: Vec<(i64, u32)> = (0..vocab)
            .filter(|&token| admitted.contains(&((token % CELL_COUNT) as u32)))
            .map(|token| (distances[token], token as u32))
            .collect();
        ranked.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        let top8: Vec<u32> = ranked.iter().take(TOP_K).map(|&(_, token)| token).collect();
        let top1 = top8
            .first()
            .copied()
            .ok_or_else(|| invalid("writer admitted no token"))?;
        let rival_distance = ranked
            .iter()
            .find(|&&(_, token)| token != target)
            .map_or(target_distance, |&(distance, _)| distance);
        let margin = i64::from(self.config.margin_q12)
            .checked_mul(width as i64)
            .ok_or_else(|| invalid("rank margin overflows i64"))?;
        Ok(RankMarginReading {
            target_distance,
            target_rank,
            margin_loss: (margin + target_distance - rival_distance).max(0),
            top1,
            top8,
            target_admitted,
            target_cell,
            admitted_cells: admitted,
        })
    }
}

pub(crate) fn validate_training_model(artifact: &[u8]) -> Result<()> {
    let model = IntegerStackModel::parse(artifact)
        .map_err(|error| invalid(format!("rank-margin model does not parse: {error}")))?;
    if model.shape().vocab != VOCABULARY {
        return Err(invalid("rank-margin training requires vocabulary 4096"));
    }
    if !model.shape().head_native_vector() || model.shape().head_nearest() {
        return Err(invalid(
            "rank-margin training requires the native cell-writer head",
        ));
    }
    Ok(())
}

fn validate_samples(samples: &[RankMarginSample], forward: &CpuFbsForward) -> Result<()> {
    for (index, sample) in samples.iter().enumerate() {
        if sample.input.is_empty()
            || sample.input.len() >= forward.context_length()
            || sample
                .input
                .iter()
                .any(|&token| token as usize >= forward.vocabulary())
            || sample.target as usize >= forward.vocabulary()
        {
            return Err(invalid(format!(
                "rank-margin sample {index} is empty, outside context or outside vocabulary"
            )));
        }
    }
    Ok(())
}

pub(crate) fn sample_fingerprint(samples: &[RankMarginSample]) -> [u8; 32] {
    let mut hash = blake3::Hasher::new();
    hash.update(b"uor-r4.cpu-rank-margin-samples/1\0");
    hash.update(&(samples.len() as u64).to_le_bytes());
    for sample in samples {
        hash.update(&(sample.input.len() as u64).to_le_bytes());
        for &token in &sample.input {
            hash.update(&token.to_le_bytes());
        }
        hash.update(&sample.target.to_le_bytes());
    }
    *hash.finalize().as_bytes()
}

pub(crate) fn shuffled_order(len: usize, seed: u64, epoch: u64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..len).collect();
    let mut state = seed
        .wrapping_mul(0xD6E8_FEB8_6659_FD93)
        .wrapping_add(epoch.wrapping_mul(0xA076_1D64_78BD_642F))
        | 1;
    for end in (1..len).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let draw = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        order.swap(end, (draw % (end as u64 + 1)) as usize);
    }
    order
}

fn move_toward(value: i16, query: i32, step: i16, limit: i16) -> i16 {
    if value < -limit || value > limit || query == i32::from(value) {
        return value;
    }
    let next = if query > i32::from(value) {
        i32::from(value) + i32::from(step)
    } else if query < i32::from(value) {
        i32::from(value) - i32::from(step)
    } else {
        i32::from(value)
    };
    next.clamp(-i32::from(limit), i32::from(limit)) as i16
}

fn move_away(value: i16, query: i32, step: i16, limit: i16) -> i16 {
    if value < -limit || value > limit {
        return value;
    }
    let next = if query >= i32::from(value) {
        i32::from(value) - i32::from(step)
    } else {
        i32::from(value) + i32::from(step)
    };
    next.clamp(-i32::from(limit), i32::from(limit)) as i16
}

fn patch_writer_table(artifact: &[u8], table: &[i16]) -> Result<Vec<u8>> {
    if artifact.len() < 16 || &artifact[..8] != b"UORLUT01" {
        return Err(invalid("artifact magic differs"));
    }
    let header_len = u64::from_le_bytes(
        artifact[8..16]
            .try_into()
            .map_err(|_| invalid("artifact header length differs"))?,
    );
    let header_end = 16usize
        .checked_add(
            usize::try_from(header_len).map_err(|_| invalid("artifact header is too large"))?,
        )
        .filter(|&end| end <= artifact.len())
        .ok_or_else(|| invalid("artifact header is outside the file"))?;
    let data_start = header_end.div_ceil(64) * 64;
    if data_start > artifact.len() {
        return Err(invalid("artifact data start is outside the file"));
    }
    let header: serde_json::Value = serde_json::from_slice(&artifact[16..header_end])
        .map_err(|error| invalid(format!("artifact header is invalid JSON: {error}")))?;
    let entry = header["tables"]
        .as_array()
        .and_then(|tables| {
            tables
                .iter()
                .find(|entry| entry["name"].as_str() == Some("writer.table"))
        })
        .ok_or_else(|| invalid("artifact has no writer.table"))?;
    if entry["kind"].as_str() != Some("i16") || entry["len"].as_u64() != Some(table.len() as u64) {
        return Err(invalid("artifact writer.table shape differs"));
    }
    let offset = usize::try_from(
        entry["span"]["offset"]
            .as_u64()
            .ok_or_else(|| invalid("writer.table offset is missing"))?,
    )
    .map_err(|_| invalid("writer.table offset exceeds usize"))?;
    let bytes = usize::try_from(
        entry["span"]["bytes"]
            .as_u64()
            .ok_or_else(|| invalid("writer.table byte length is missing"))?,
    )
    .map_err(|_| invalid("writer.table byte length exceeds usize"))?;
    if bytes != table.len() * 2 {
        return Err(invalid("writer.table byte length differs"));
    }
    let start = data_start
        .checked_add(offset)
        .ok_or_else(|| invalid("writer.table offset overflows"))?;
    let end = start
        .checked_add(bytes)
        .filter(|&end| end <= artifact.len())
        .ok_or_else(|| invalid("writer.table is outside the file"))?;
    let mut out = artifact.to_vec();
    for (slot, &value) in out[start..end].chunks_exact_mut(2).zip(table) {
        slot.copy_from_slice(&value.to_le_bytes());
    }
    IntegerStackModel::parse(&out)
        .map_err(|error| invalid(format!("patched artifact does not parse: {error}")))?;
    Ok(out)
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(len)
            .filter(|&end| end <= self.bytes.len())
            .ok_or_else(|| invalid("rank-margin checkpoint is truncated"))?;
        let slice = &self.bytes[self.at..end];
        self.at = end;
        Ok(slice)
    }

    fn i16(&mut self) -> Result<i16> {
        Ok(i16::from_le_bytes(
            self.take(2)?.try_into().map_err(|_| invalid("i16"))?,
        ))
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| invalid("u32"))?,
        ))
    }

    fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| invalid("i32"))?,
        ))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| invalid("u64"))?,
        ))
    }

    fn done(&self) -> bool {
        self.at == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_discrete::normalization_mechanism::orbit_root_coords;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth};
    use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, FbsTrunkSnapshot};

    fn canonical_snapshot(hamming: bool) -> FbsTrunkSnapshot {
        let mut snapshot = synthetic_fbs_trunk_with(
            151,
            FbsSynth {
                vocab: VOCABULARY,
                d: 64,
                mlp: 64,
                layers: if hamming { 2 } else { 1 },
                context: 8,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.25,
                decay_step: 0.0625,
            },
        );
        snapshot.embed.clear();
        for token in 0..snapshot.vocab {
            for block in 0..snapshot.width / 8 {
                let root = match block {
                    0 => token & 15,
                    7 => ((token >> 4) ^ (token >> 8)) & 15,
                    _ => {
                        ((token >> 8) * 7 + ((token >> 4) & 15) * 3 + (token & 15) * 5 + block) & 15
                    }
                };
                snapshot.embed.extend(
                    orbit_root_coords(root as u8)
                        .unwrap()
                        .map(|value| value as f32),
                );
            }
        }
        if hamming {
            let mut reads = synthetic_hamming(snapshot.width, snapshot.layers.len(), 4, 152);
            reads
                .reads
                .iter_mut()
                .for_each(|read| read.weight.fill(1.0));
            snapshot.hamming = Some(reads);
        }
        snapshot
    }

    fn training_case(
        hamming: bool,
    ) -> (Vec<u8>, RankMarginConfig, RankMarginSample, Vec<i32>, u32) {
        let snapshot = canonical_snapshot(hamming);
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let config = RankMarginConfig {
            margin_q12: 1,
            step_q12: 1,
            parameter_limit_q12: 8_192,
        };
        let mut probe = CpuRankMarginTrainer::from_artifact(
            &artifact,
            vec![RankMarginSample {
                input: vec![0],
                target: 0,
            }],
            config,
            7,
        )
        .unwrap();
        let sample = admitted_samples(&mut probe, 1).remove(0);
        let query = probe.query_for(&sample).unwrap();
        let before = probe.reading(&query, sample.target).unwrap();
        let rival = before
            .top8
            .iter()
            .copied()
            .find(|&token| token != sample.target)
            .unwrap();
        (artifact, config, sample, query, rival)
    }

    fn admitted_samples(probe: &mut CpuRankMarginTrainer, count: usize) -> Vec<RankMarginSample> {
        let mut samples = Vec::new();
        for input in 0u32..VOCABULARY as u32 {
            let scan = RankMarginSample {
                input: vec![input],
                target: 0,
            };
            let reading = probe.evaluate_sample(&scan).unwrap();
            if let Some(target) = reading
                .top8
                .iter()
                .copied()
                .find(|&token| token != reading.top1)
            {
                samples.push(RankMarginSample {
                    input: vec![input],
                    target,
                });
                if samples.len() == count {
                    break;
                }
            }
        }
        assert_eq!(samples.len(), count);
        samples
    }

    fn writer_span(artifact: &[u8]) -> std::ops::Range<usize> {
        let header_len = u64::from_le_bytes(artifact[8..16].try_into().unwrap()) as usize;
        let header_end = 16 + header_len;
        let data_start = header_end.div_ceil(64) * 64;
        let header: serde_json::Value = serde_json::from_slice(&artifact[16..header_end]).unwrap();
        let entry = header["tables"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["name"].as_str() == Some("writer.table"))
            .unwrap();
        let start = data_start + entry["span"]["offset"].as_u64().unwrap() as usize;
        start..start + entry["span"]["bytes"].as_u64().unwrap() as usize
    }

    #[test]
    fn a_rank_margin_step_moves_target_and_rival_without_changing_cells() {
        let (artifact, config, sample, query, rival) = training_case(false);
        let mut trainer =
            CpuRankMarginTrainer::from_artifact(&artifact, vec![sample.clone()], config, 7)
                .unwrap();
        let before = trainer.reading(&query, sample.target).unwrap();
        assert!(before.target_admitted, "{before:?}");
        assert!(before.margin_loss > 0);
        let width = trainer.forward.width();
        let target_cell = before.target_cell;
        let step = trainer.train_steps(1).unwrap().remove(0);
        assert!(step.updated, "{step:?}");
        assert_eq!(step.rival, Some(rival));
        let coordinate = step.coordinate.unwrap();
        assert!((8..width - 8).contains(&coordinate));
        assert!(step.after.margin_loss < step.before.margin_loss);
        assert_eq!(step.after.target_cell, target_cell);
        assert!(step.after.target_admitted);
        assert!(step.after.top8.len() <= TOP_K);
        assert!(IntegerStackModel::parse(trainer.artifact()).is_ok());
        let span = writer_span(trainer.artifact());
        assert_eq!(artifact.len(), trainer.artifact().len());
        for (index, (&before, &after)) in artifact.iter().zip(trainer.artifact()).enumerate() {
            if index < span.start || index >= span.end {
                assert_eq!(before, after, "non-writer byte {index} changed");
            }
        }
    }

    #[test]
    fn admission_guard_refuses_an_unadmitted_target() {
        let (artifact, config, sample, query, _) = training_case(false);
        let mut trainer =
            CpuRankMarginTrainer::from_artifact(&artifact, vec![sample], config, 7).unwrap();
        let admitted = trainer.reading(&query, 0).unwrap().admitted_cells;
        let target = (1..VOCABULARY)
            .find(|&token| !admitted.contains(&((token % CELL_COUNT) as u32)))
            .unwrap() as u32;
        let before = trainer.artifact().to_vec();
        let step = trainer.apply_query(0, &query, target).unwrap();
        assert!(!step.updated);
        assert_eq!(trainer.artifact(), before);
    }

    #[test]
    fn an_update_refreshes_hamming_codes_and_matches_the_served_artifact() {
        let (artifact, config, sample, _, _) = training_case(true);
        let mut trainer =
            CpuRankMarginTrainer::from_artifact(&artifact, vec![sample], config, 7).unwrap();
        let step = trainer.train_steps(1).unwrap().remove(0);
        assert!(step.updated);
        let model = IntegerStackModel::parse(trainer.artifact()).unwrap();
        let mut session = model.session();
        let mut cpu = CpuFbsForward::from_artifact(trainer.artifact()).unwrap();
        for token in [0, 5, 151, 272] {
            let expected = session.step_native(token).unwrap();
            let actual = cpu.step(token, CpuFbsHead::Writer).unwrap();
            assert_eq!(actual.query_q12, expected.query_q12);
            assert_eq!(actual.token, expected.token);
            assert_eq!(actual.kept_cells, expected.kept_cells);
        }
    }

    #[test]
    fn checkpoint_resume_reproduces_bytes_and_next_readings() {
        let snapshot = canonical_snapshot(true);
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let mut probe = CpuRankMarginTrainer::from_artifact(
            &artifact,
            vec![RankMarginSample {
                input: vec![0],
                target: 0,
            }],
            RankMarginConfig::default(),
            17,
        )
        .unwrap();
        let samples = admitted_samples(&mut probe, 3);
        let mut original =
            CpuRankMarginTrainer::from_artifact(&artifact, samples.clone(), probe.config, 17)
                .unwrap();
        original.train_steps(2).unwrap();
        let checkpoint = original.checkpoint().unwrap();
        let mut resumed = CpuRankMarginTrainer::resume(&checkpoint, samples.clone()).unwrap();
        let left = original.train_steps(5).unwrap();
        let right = resumed.train_steps(5).unwrap();
        assert_eq!(left, right);
        assert_eq!(
            original.checkpoint().unwrap(),
            resumed.checkpoint().unwrap()
        );

        let mut short = checkpoint.clone();
        short.truncate(short.len() - 1);
        assert!(CpuRankMarginTrainer::resume(&short, samples.clone()).is_err());
        let mut oversized = checkpoint.clone();
        oversized[96..100].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(CpuRankMarginTrainer::resume(&oversized, samples.clone()).is_err());
        let mut duplicate = checkpoint.clone();
        let first: [u8; 4] = duplicate[100..104].try_into().unwrap();
        duplicate[104..108].copy_from_slice(&first);
        assert!(CpuRankMarginTrainer::resume(&duplicate, samples.clone()).is_err());
        let mut other = samples;
        other[0].target = 0;
        assert!(CpuRankMarginTrainer::resume(&checkpoint, other).is_err());
    }

    #[test]
    fn the_trainer_requires_the_native_cell_writer_head() {
        let mut snapshot = canonical_snapshot(false);
        snapshot.readout_nearest = true;
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        assert!(CpuRankMarginTrainer::from_artifact(
            &artifact,
            vec![RankMarginSample {
                input: vec![0],
                target: 0,
            }],
            RankMarginConfig::default(),
            7,
        )
        .is_err());
    }

    #[test]
    fn parameter_bounds_and_frozen_query_training_are_deterministic() {
        assert_eq!(move_toward(10, 0, 4, 8), 10);
        assert_eq!(move_toward(6, 0, 4, 8), 2);
        assert_eq!(move_toward(0, 10, 4, 8), 4);
        assert_eq!(move_away(10, 0, 4, 8), 10);
        assert_eq!(move_away(6, 0, 4, 8), 8);
        assert_eq!(move_away(0, 10, 4, 8), -4);
        assert_eq!(move_toward(20, 0, 1, 8), 20);
        assert_eq!(move_away(-20, 0, 1, 8), -20);
        let (artifact, config, sample, _, _) = training_case(false);
        let mut left =
            CpuRankMarginTrainer::from_artifact(&artifact, vec![sample.clone()], config, 9)
                .unwrap();
        let mut right =
            CpuRankMarginTrainer::from_artifact(&artifact, vec![sample], config, 9).unwrap();
        assert_eq!(left.train_steps(3).unwrap(), right.train_steps(3).unwrap());
        assert_eq!(left.checkpoint().unwrap(), right.checkpoint().unwrap());
    }

    #[test]
    fn a_satisfied_margin_leaves_the_artifact_and_clocks_unchanged() {
        let snapshot = canonical_snapshot(false);
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let config = RankMarginConfig::default();
        let mut probe = CpuRankMarginTrainer::from_artifact(
            &artifact,
            vec![RankMarginSample {
                input: vec![0],
                target: 0,
            }],
            config,
            13,
        )
        .unwrap();
        let sample = (0u32..VOCABULARY as u32)
            .find_map(|input| {
                let scan = RankMarginSample {
                    input: vec![input],
                    target: 0,
                };
                let reading = probe.evaluate_sample(&scan).unwrap();
                (reading.margin_loss == 0).then_some(RankMarginSample {
                    input: vec![input],
                    target: reading.top1,
                })
            })
            .expect("an admitted sample with satisfied margin");
        let before_artifact = artifact.clone();
        let mut trainer =
            CpuRankMarginTrainer::from_artifact(&artifact, vec![sample], config, 13).unwrap();
        let step = trainer.train_steps(1).unwrap().remove(0);
        assert!(!step.updated);
        assert_eq!(trainer.artifact(), before_artifact);
        assert_eq!(trainer.step_count(), 1);
        assert_eq!(step.before, step.after);
    }
}
