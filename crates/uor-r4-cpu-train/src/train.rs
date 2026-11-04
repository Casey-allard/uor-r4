//! Deterministic frozen-query rank-margin training of the shared token table.
//!
//! One accepted update moves the target row toward the query and the hardest
//! admitted rival away from it. The complete patched artifact is parsed and
//! rebuilt immediately, so table-derived Hamming keys cannot remain stale.

use uor_r4_integer::stack::IntegerStackModel;

use crate::{CpuFbsForward, CpuFbsHead, CpuTrainError, CELL_COUNT, TOP_K, VOCABULARY};

const MAGIC: &[u8; 16] = b"UOR-R4-CPU-TRAIN";
const CHECKPOINT_VERSION: u32 = 2;
const TABLE_BVGK_MAX_COORDINATES: usize = 64;

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
    /// Distinct target/rival coordinates committed as one BVG-K table block.
    pub coordinates_per_step: usize,
}

impl Default for RankMarginConfig {
    fn default() -> Self {
        Self {
            margin_q12: 1,
            step_q12: 1,
            parameter_limit_q12: 8_192,
            coordinates_per_step: 1,
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
        if !(1..=TABLE_BVGK_MAX_COORDINATES).contains(&self.coordinates_per_step) {
            return Err(invalid(format!(
                "table BVG-K coordinate block must be in 1..={TABLE_BVGK_MAX_COORDINATES}"
            )));
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
    /// Every coordinate in the committed or refused BVG-K block.
    pub coordinates: Vec<usize>,
    pub rival: Option<u32>,
    pub before: RankMarginReading,
    pub after: RankMarginReading,
}

pub(crate) struct WriterUpdateProposal {
    pub(crate) artifact: Vec<u8>,
    pub(crate) forward: CpuFbsForward,
    pub(crate) coordinates: Vec<usize>,
    pub(crate) rival: u32,
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

#[derive(Clone)]
pub(crate) struct CpuRankMarginEvaluator {
    forward: CpuFbsForward,
    config: RankMarginConfig,
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

    pub fn epoch(&self) -> u64 {
        self.epoch
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
        out.extend_from_slice(
            &u32::try_from(self.config.coordinates_per_step)
                .map_err(|_| invalid("table BVG-K coordinate block exceeds u32"))?
                .to_le_bytes(),
        );
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
        let version = input.u32()?;
        if !(1..=CHECKPOINT_VERSION).contains(&version) {
            return Err(invalid("rank-margin checkpoint version differs"));
        }
        let config = RankMarginConfig {
            margin_q12: input.i32()?,
            step_q12: input.i16()?,
            parameter_limit_q12: input.i16()?,
            coordinates_per_step: if version >= 2 {
                usize::try_from(input.u32()?)
                    .map_err(|_| invalid("rank-margin checkpoint coordinate block differs"))?
            } else {
                1
            },
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
        query_for_forward(&mut self.forward, sample)
    }

    pub(crate) fn evaluator(&self) -> CpuRankMarginEvaluator {
        CpuRankMarginEvaluator {
            forward: self.forward.clone(),
            config: self.config,
        }
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
                coordinates: Vec::new(),
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
                coordinates: Vec::new(),
                rival: Some(rival),
                before: before.clone(),
                after: before,
            });
        }

        let Some(proposal) =
            propose_writer_update(&self.artifact, &self.forward, self.config, query, target)?
        else {
            return Ok(RankMarginStep {
                sample: sample_index,
                target,
                updated: false,
                coordinate: None,
                coordinates: Vec::new(),
                rival: Some(rival),
                before: before.clone(),
                after: before,
            });
        };
        let selected = proposal.coordinates;
        let rival = proposal.rival;
        let patched = proposal.artifact;
        let mut probe = Self {
            artifact: patched.clone(),
            forward: proposal.forward,
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
            coordinate: selected.first().copied(),
            coordinates: selected,
            rival: Some(rival),
            before,
            after,
        })
    }

    pub(crate) fn reading(&self, query: &[i32], target: u32) -> Result<RankMarginReading> {
        reading_from_forward(&self.forward, self.config, query, target)
    }
}

pub(crate) fn propose_writer_update(
    artifact: &[u8],
    forward: &CpuFbsForward,
    config: RankMarginConfig,
    query: &[i32],
    target: u32,
) -> Result<Option<WriterUpdateProposal>> {
    let before = reading_from_forward(forward, config, query, target)?;
    let Some(rival) = before.top8.iter().copied().find(|&token| token != target) else {
        return Ok(None);
    };
    if before.margin_loss == 0 || !before.target_admitted {
        return Ok(None);
    }
    let width = forward.width();
    let table = forward.token_table();
    let target_row = &table[target as usize * width..][..width];
    let rival_row = &table[rival as usize * width..][..width];
    let interior_start = 8;
    let interior_end = width.saturating_sub(8);
    if interior_start >= interior_end {
        return Err(invalid("table has no mutable interior coordinate"));
    }
    let selected = if config.coordinates_per_step == 1 {
        let coordinate = (interior_start..interior_end)
            .map(|coordinate| {
                (
                    i64::from(query[coordinate].abs_diff(i32::from(target_row[coordinate])))
                        + i64::from(query[coordinate].abs_diff(i32::from(rival_row[coordinate]))),
                    coordinate,
                )
            })
            .max_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)))
            .map(|(_, coordinate)| coordinate);
        let Some(coordinate) = coordinate else {
            return Err(invalid("table has no mutable interior coordinate"));
        };
        if query[coordinate] == i32::from(target_row[coordinate])
            && query[coordinate] == i32::from(rival_row[coordinate])
        {
            return Ok(None);
        }
        vec![coordinate]
    } else {
        let mut coordinates = (interior_start..interior_end)
            .map(|coordinate| {
                (
                    i64::from(query[coordinate].abs_diff(i32::from(target_row[coordinate])))
                        + i64::from(query[coordinate].abs_diff(i32::from(rival_row[coordinate]))),
                    coordinate,
                )
            })
            .filter(|&(_, coordinate)| {
                query[coordinate] != i32::from(target_row[coordinate])
                    || query[coordinate] != i32::from(rival_row[coordinate])
            })
            .collect::<Vec<_>>();
        coordinates.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
        coordinates.truncate(config.coordinates_per_step);
        coordinates
            .iter()
            .map(|&(_, coordinate)| coordinate)
            .collect::<Vec<_>>()
    };
    if selected.is_empty() {
        return Ok(None);
    }
    let mut proposed = table.to_vec();
    for &coordinate in &selected {
        proposed[target as usize * width + coordinate] = move_toward(
            target_row[coordinate],
            query[coordinate],
            config.step_q12,
            config.parameter_limit_q12,
        );
        proposed[rival as usize * width + coordinate] = move_away(
            rival_row[coordinate],
            query[coordinate],
            config.step_q12,
            config.parameter_limit_q12,
        );
    }
    let artifact = patch_writer_table(artifact, &proposed)?;
    let patched_forward = CpuFbsForward::from_artifact(&artifact)
        .map_err(|error| invalid(format!("patched rank-margin artifact: {error}")))?;
    Ok(Some(WriterUpdateProposal {
        artifact,
        forward: patched_forward,
        coordinates: selected,
        rival,
    }))
}

impl CpuRankMarginEvaluator {
    pub(crate) fn config(&self) -> RankMarginConfig {
        self.config
    }

    pub(crate) fn query_for(&mut self, sample: &RankMarginSample) -> Result<Vec<i32>> {
        query_for_forward(&mut self.forward, sample)
    }

    pub(crate) fn reading(&self, query: &[i32], target: u32) -> Result<RankMarginReading> {
        reading_from_forward(&self.forward, self.config, query, target)
    }
}

fn query_for_forward(forward: &mut CpuFbsForward, sample: &RankMarginSample) -> Result<Vec<i32>> {
    forward.reset();
    let mut step = None;
    for &token in &sample.input {
        step = Some(
            forward
                .step(token, CpuFbsHead::Writer)
                .map_err(|error| invalid(format!("rank-margin forward: {error}")))?,
        );
    }
    step.map(|step| step.query_q12)
        .ok_or_else(|| invalid("training sample input is empty"))
}

pub(crate) fn reading_from_forward(
    forward: &CpuFbsForward,
    config: RankMarginConfig,
    query: &[i32],
    target: u32,
) -> Result<RankMarginReading> {
    reading_from_forward_at_kept(forward, config, query, target, forward.kept_cell_count())
}

pub(crate) fn reading_from_forward_at_kept(
    forward: &CpuFbsForward,
    config: RankMarginConfig,
    query: &[i32],
    target: u32,
    kept_cells: usize,
) -> Result<RankMarginReading> {
    if query.len() != forward.width() {
        return Err(invalid("rank-margin query width differs from the table"));
    }
    if target as usize >= forward.vocabulary() {
        return Err(invalid("rank-margin target is outside the vocabulary"));
    }
    if !(1..=16).contains(&kept_cells) {
        return Err(invalid("rank-margin kept-cell override is out of bounds"));
    }
    let admitted = forward
        .writer_cell_order(query)
        .map_err(|error| invalid(format!("rank-margin writer cells: {error}")))?
        .into_iter()
        .take(kept_cells)
        .collect::<Vec<_>>();
    reading_from_admitted_cells(forward, config, query, target, &admitted)
}

pub(crate) fn reading_from_forward_cells(
    forward: &CpuFbsForward,
    config: RankMarginConfig,
    query: &[i32],
    target: u32,
    admitted: &[u32],
) -> Result<RankMarginReading> {
    if admitted.is_empty()
        || admitted.len() > 16
        || admitted.iter().any(|cell| *cell >= CELL_COUNT as u32)
        || admitted
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != admitted.len()
    {
        return Err(invalid("rank-margin admitted-cell override differs"));
    }
    reading_from_admitted_cells(forward, config, query, target, admitted)
}

fn reading_from_admitted_cells(
    forward: &CpuFbsForward,
    config: RankMarginConfig,
    query: &[i32],
    target: u32,
    admitted: &[u32],
) -> Result<RankMarginReading> {
    if query.len() != forward.width() {
        return Err(invalid("rank-margin query width differs from the table"));
    }
    if target as usize >= forward.vocabulary() {
        return Err(invalid("rank-margin target is outside the vocabulary"));
    }
    let width = query.len();
    let table = forward.token_table();
    let vocab = forward.vocabulary();
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
            distance < target_distance || (distance == target_distance && token < target as usize)
        })
        .count();

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
    let margin = i64::from(config.margin_q12)
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
        admitted_cells: admitted.to_vec(),
    })
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
            || sample.input.len() > forward.context_length()
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
    use std::collections::BTreeMap;
    use uor_r4_discrete::normalization_mechanism::orbit_root_coords;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth};
    use uor_r4_integer::stack::{
        add_point_key_uorlut01, export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot,
        FbsReplyTable, FbsTrunkSnapshot, PointKeyImage, MEMORY_PAD,
    };

    use crate::{audit_served_forward, ServedForwardAuditConfig};

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
            coordinates_per_step: 1,
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

    fn full_path_artifact() -> Vec<u8> {
        let mut snapshot = canonical_snapshot(true);
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 4,
            key_window: snapshot.context,
            opening: vec![FbsMemoryEntry {
                key: vec![0, 7],
                value: 42,
            }],
            opening_weight: vec![1.0; snapshot.width],
            reply: Some(FbsReplyTable {
                turn: 3,
                entries: vec![
                    FbsMemoryEntry {
                        key: vec![42, 77, MEMORY_PAD, 1, MEMORY_PAD, 11],
                        value: 42,
                    },
                    FbsMemoryEntry {
                        key: vec![42, 77, MEMORY_PAD, 2, MEMORY_PAD, 22],
                        value: 77,
                    },
                ],
                weight: vec![1.0; snapshot.width],
            }),
        });
        let bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        add_point_key_uorlut01(
            &bytes,
            &PointKeyImage {
                width: snapshot.width,
                points: vec![0; snapshot.width],
                targets: vec![99],
                vote_k: 1,
            },
        )
        .unwrap()
    }

    #[derive(Clone)]
    struct ConversationExample {
        prompt: Vec<u32>,
        reply: Vec<u32>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct FullPathParts {
        initial: Vec<u8>,
        after_writer: Vec<u8>,
        after_hamming: Vec<u8>,
        with_memory: Vec<u8>,
        hamming_off: Vec<u8>,
        stripped: Vec<u8>,
        final_artifact: Vec<u8>,
    }

    fn commonest_target(counts: &BTreeMap<u32, usize>) -> u32 {
        counts
            .iter()
            .min_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)))
            .map(|(&target, _)| target)
            .unwrap()
    }

    #[test]
    fn a_tied_memory_target_uses_the_smaller_token() {
        assert_eq!(
            commonest_target(&BTreeMap::from([(20, 1), (19, 1), (21, 1)])),
            19
        );
    }

    fn memory_entries(counts: BTreeMap<Vec<u32>, BTreeMap<u32, usize>>) -> Vec<FbsMemoryEntry> {
        counts
            .into_iter()
            .map(|(key, targets)| FbsMemoryEntry {
                key,
                value: commonest_target(&targets),
            })
            .collect()
    }

    fn train_turn_memory(
        examples: &[ConversationExample],
        width: usize,
        k: usize,
        key_window: usize,
        reply_turn: usize,
    ) -> FbsMemorySnapshot {
        let mut opening = BTreeMap::<Vec<u32>, BTreeMap<u32, usize>>::new();
        let mut reply = BTreeMap::<Vec<u32>, BTreeMap<u32, usize>>::new();
        for example in examples {
            let prompt = &example.prompt;
            let reply_tokens = &example.reply;
            assert!(!reply_tokens.is_empty());
            let prompt_start = prompt
                .iter()
                .rposition(|&token| token == 0 || token == 1)
                .map_or(0, |at| at + 1);
            let opening_key = prompt[prompt_start..]
                .iter()
                .copied()
                .take(k)
                .collect::<Vec<_>>();
            assert!(!opening_key.is_empty());
            *opening
                .entry(opening_key)
                .or_default()
                .entry(reply_tokens[0])
                .or_default() += 1;

            let turn = prompt[prompt_start..]
                .iter()
                .copied()
                .take(reply_turn)
                .collect::<Vec<_>>();
            let tokens = prompt
                .iter()
                .chain(reply_tokens)
                .copied()
                .collect::<Vec<_>>();
            let reply_start = prompt.len();
            for offset in 1..reply_tokens.len().saturating_sub(1) {
                let row = reply_start + offset;
                let block_start = row / key_window * key_window;
                let visible = block_start.max(reply_start);
                let length = row + 1 - visible;
                let mut key = turn.clone();
                key.extend([
                    MEMORY_PAD,
                    u32::try_from(length).unwrap(),
                    MEMORY_PAD,
                    tokens[row],
                ]);
                *reply
                    .entry(key)
                    .or_default()
                    .entry(*reply_tokens.get(offset + 1).unwrap())
                    .or_default() += 1;
            }
        }
        FbsMemorySnapshot {
            k,
            key_window,
            opening: memory_entries(opening),
            opening_weight: vec![1.0; width],
            reply: Some(FbsReplyTable {
                turn: reply_turn,
                entries: memory_entries(reply),
                weight: vec![1.0; width],
            }),
        }
    }

    fn trained_full_path_parts() -> FullPathParts {
        let mut snapshot = canonical_snapshot(true);
        let initial = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        assert!(IntegerStackModel::parse(&initial)
            .unwrap()
            .turn_memory()
            .is_none());
        let writer_samples = admitted_samples(
            &mut CpuRankMarginTrainer::from_artifact(
                &initial,
                vec![RankMarginSample {
                    input: vec![0],
                    target: 0,
                }],
                RankMarginConfig::default(),
                911,
            )
            .unwrap(),
            8,
        );
        let mut writer = CpuRankMarginTrainer::from_artifact(
            &initial,
            writer_samples,
            RankMarginConfig::default(),
            911,
        )
        .unwrap();
        let old_table = writer.table().to_vec();
        let steps = writer.train_steps(8).unwrap();
        assert!(steps.iter().all(|step| step.updated));
        for ((source, &old), &trained) in snapshot
            .embed
            .iter_mut()
            .zip(&old_table)
            .zip(writer.table())
        {
            *source += f64::from(i32::from(trained) - i32::from(old)) as f32 / 4096.0;
        }
        let after_writer = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let parsed = IntegerStackModel::parse(&after_writer).unwrap();
        assert_eq!(
            writer.table(),
            parsed.fbs_trunk_image().unwrap().writer_table
        );

        let rank_samples = [
            RankMarginSample {
                input: vec![10, 11, 12],
                target: 20,
            },
            RankMarginSample {
                input: vec![30, 31, 32],
                target: 40,
            },
        ];

        let score =
            |candidate: &FbsTrunkSnapshot, samples: &[RankMarginSample]| -> (u64, u64, i64, i64) {
                let artifact = export_fbs_trunk_uorlut01(candidate).unwrap();
                let mut trainer = CpuRankMarginTrainer::from_artifact(
                    &artifact,
                    samples.to_vec(),
                    RankMarginConfig::default(),
                    912,
                )
                .unwrap();
                let mut score = (0u64, 0u64, 0i64, 0i64);
                for sample in samples {
                    let reading = trainer.evaluate_sample(sample).unwrap();
                    score.0 += u64::from(!reading.target_admitted);
                    score.1 += u64::try_from(reading.target_rank).unwrap_or(u64::MAX);
                    score.2 += reading.margin_loss;
                    score.3 = score.3.saturating_add(reading.target_distance);
                }
                score
            };

        let mut margins_changed = 0usize;
        let candidates = [0.5f32, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0];
        let read_count = snapshot.hamming.as_ref().unwrap().reads.len();
        let head_count = snapshot.hamming.as_ref().unwrap().heads;
        for read in 0..read_count {
            for head in 0..head_count {
                let original = snapshot.hamming.as_ref().unwrap().reads[read].margin[head];
                let mut best = (score(&snapshot, &rank_samples), original);
                for candidate in candidates {
                    let mut trial = snapshot.clone();
                    trial.hamming.as_mut().unwrap().reads[read].margin[head] = candidate;
                    let candidate_score = score(&trial, &rank_samples);
                    if candidate_score < best.0 {
                        best = (candidate_score, candidate);
                    }
                }
                if best.1 != original {
                    snapshot.hamming.as_mut().unwrap().reads[read].margin[head] = best.1;
                    margins_changed += 1;
                }
            }
        }
        assert!(margins_changed > 0, "Hamming margins did not train");
        let after_hamming = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        assert_ne!(after_hamming, after_writer);

        let memory_examples = [
            ConversationExample {
                prompt: vec![10, 11, 12],
                reply: vec![20, 21, 22, 23],
            },
            ConversationExample {
                prompt: vec![10, 11, 12],
                reply: vec![19, 24],
            },
        ];
        let memory = train_turn_memory(&memory_examples, snapshot.width, 3, snapshot.context, 3);
        assert_eq!(
            memory.opening[0].value, 19,
            "the tied target did not break to 19"
        );
        snapshot.memory = Some(memory);
        let with_memory = export_fbs_trunk_uorlut01(&snapshot).unwrap();

        let point_prompt = [30, 31, 32];
        let mut point_forward = CpuFbsForward::from_artifact(&with_memory).unwrap();
        let mut point = None;
        for (index, &token) in point_prompt.iter().enumerate() {
            if index + 1 == point_prompt.len() {
                point_forward.mark_reply_start();
            }
            point = Some(point_forward.step(token, CpuFbsHead::Writer).unwrap());
        }
        let point = point.unwrap();
        let point_values = point
            .query_q12
            .iter()
            .map(|&value| i16::try_from(value).unwrap())
            .collect::<Vec<_>>();
        let mut no_hamming = snapshot.clone();
        no_hamming.hamming = None;
        let no_hamming_memory = export_fbs_trunk_uorlut01(&no_hamming).unwrap();
        let hamming_off = add_point_key_uorlut01(
            &no_hamming_memory,
            &PointKeyImage {
                width: snapshot.width,
                points: point_values.clone(),
                targets: vec![40],
                vote_k: 1,
            },
        )
        .unwrap();
        no_hamming.memory = None;
        let stripped = export_fbs_trunk_uorlut01(&no_hamming).unwrap();
        let final_artifact = add_point_key_uorlut01(
            &with_memory,
            &PointKeyImage {
                width: snapshot.width,
                points: point_values.clone(),
                targets: vec![40],
                vote_k: 1,
            },
        )
        .unwrap();
        let memory = uor_r4_integer::stack::read_memory_uorlut01(&final_artifact)
            .unwrap()
            .unwrap();
        assert_eq!(memory.record.opening, 1);
        assert_eq!(memory.record.reply, Some(2));
        assert_eq!(memory.opening.keys, vec![vec![10, 11, 12]]);
        assert_eq!(memory.opening.values, vec![19]);
        assert!(memory.opening.keys.windows(2).all(|pair| pair[0] < pair[1]));
        let reply = memory.reply.as_ref().unwrap();
        assert!(reply.keys.windows(2).all(|pair| pair[0] < pair[1]));
        let point = memory.point.as_ref().unwrap();
        assert_eq!(point.points, point_values);
        assert_eq!(point.targets, vec![40]);
        eprintln!(
            "{}",
            serde_json::json!({
                "initial_sha256": uor_r4_discrete::corpus::sha256(&initial),
                "writer_sha256": uor_r4_discrete::corpus::sha256(&after_writer),
                "hamming_sha256": uor_r4_discrete::corpus::sha256(&after_hamming),
                "hamming_off_sha256": uor_r4_discrete::corpus::sha256(&hamming_off),
                "stripped_sha256": uor_r4_discrete::corpus::sha256(&stripped),
                "final_sha256": uor_r4_discrete::corpus::sha256(&final_artifact),
                "margins_changed": margins_changed,
                "opening_rows": memory.record.opening,
                "reply_rows": memory.record.reply,
                "final_bytes": final_artifact.len(),
            })
        );
        FullPathParts {
            initial,
            after_writer,
            after_hamming,
            with_memory,
            hamming_off,
            stripped,
            final_artifact,
        }
    }

    fn trace(
        artifact: &[u8],
        tokens: &[u32],
        reply_start: usize,
    ) -> Result<Vec<crate::CpuFbsStep>> {
        let mut forward =
            CpuFbsForward::from_artifact(artifact).map_err(|error| invalid(error.to_string()))?;
        let mut out = Vec::with_capacity(tokens.len());
        for (index, &token) in tokens.iter().enumerate() {
            if index == reply_start {
                forward.mark_reply_start();
            }
            out.push(
                forward
                    .step(token, CpuFbsHead::Writer)
                    .map_err(|error| invalid(error.to_string()))?,
            );
        }
        Ok(out)
    }

    fn first_query_divergence(
        left: &[crate::CpuFbsStep],
        right: &[crate::CpuFbsStep],
    ) -> Option<usize> {
        left.iter()
            .zip(right)
            .position(|(left, right)| left.query_q12 != right.query_q12)
    }

    #[test]
    fn data_derived_records_train_and_audit_the_full_path() -> Result<()> {
        let parts = trained_full_path_parts();
        assert_eq!(
            parts,
            trained_full_path_parts(),
            "the full-path construction is not deterministic"
        );
        assert_ne!(
            parts.after_writer, parts.initial,
            "the writer did not train"
        );
        assert_ne!(
            parts.after_hamming, parts.after_writer,
            "Hamming margins did not train"
        );
        assert_ne!(
            parts.final_artifact, parts.after_hamming,
            "records did not attach"
        );

        let base = trace(&parts.after_writer, &[10, 11, 12], 2)?;
        let hamming = trace(&parts.after_hamming, &[10, 11, 12], 2)?;
        assert_ne!(
            base.last().unwrap().query_q12,
            hamming.last().unwrap().query_q12,
            "trained Hamming margins did not change the served query"
        );

        let full_reply = trace(&parts.final_artifact, &[10, 11, 12, 20, 21, 22, 23], 2)?;
        let without_memory = trace(&parts.after_hamming, &[10, 11, 12, 20, 21, 22, 23], 2)?;
        for (row, name) in [
            (2, "opening"),
            (4, "first reply row"),
            (5, "second reply row"),
        ] {
            assert_ne!(
                full_reply[row].query_q12, without_memory[row].query_q12,
                "the {name} memory move did not change the served query"
            );
        }
        assert_eq!(
            full_reply[..2]
                .iter()
                .map(|step| &step.query_q12)
                .collect::<Vec<_>>(),
            without_memory[..2]
                .iter()
                .map(|step| &step.query_q12)
                .collect::<Vec<_>>(),
            "the opening memory changed a row before its lookup"
        );

        let point_reply = trace(&parts.final_artifact, &[30, 31, 32, 40], 2)?;
        let no_point = trace(&parts.after_hamming, &[30, 31, 32, 40], 2)?;
        assert_eq!(
            point_reply[2].token, 40,
            "the point key did not name its target"
        );
        assert_ne!(
            point_reply[2].token, no_point[2].token,
            "the point key changed no complete decision"
        );

        let mut audited = (0u64, 0u64, 0u64);
        for (tokens, reply_start) in [
            (vec![10, 11, 12, 20, 21, 22, 23], Some(2)),
            (vec![30, 31, 32, 40], Some(2)),
        ] {
            let audit = audit_served_forward(
                &parts.final_artifact,
                &ServedForwardAuditConfig {
                    tokens,
                    token_source: None,
                    expected_artifact_sha256: None,
                    reply_start,
                    measure_timing: false,
                },
            )
            .unwrap();
            assert!(!audit.parity.instrument_fault);
            assert_eq!(audit.parity.query_coordinate_mismatches, 0);
            assert_eq!(audit.parity.token_mismatches, 0);
            assert_eq!(audit.parity.kept_cell_mismatches, 0);
            audited.0 += audit.parity.query_coordinates_compared;
            audited.1 += audit.parity.token_decisions_compared;
            audited.2 += audit.parity.kept_cells_compared;
            let manifest = &audit.manifest;
            assert!(manifest.hamming.is_some());
            assert!(manifest.memory.is_some());
            assert!(manifest.memory.as_ref().unwrap().point_key.is_some());
        }
        eprintln!(
            "{}",
            serde_json::json!({
                "query_coordinates": audited.0,
                "token_decisions": audited.1,
                "kept_cells": audited.2,
                "point_decision": point_reply[2].token,
                "no_point_decision": no_point[2].token,
            })
        );
        let root =
            std::env::temp_dir().join(format!("uor-full-path-records-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("trained-full-path.uorlut");
        std::fs::write(&path, &parts.final_artifact).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), parts.final_artifact);
        std::fs::remove_dir_all(root).unwrap();
        if let Some(out) = std::env::var_os("UOR_FULL_PATH_ARTIFACT") {
            std::fs::write(out, &parts.final_artifact).unwrap();
        }
        Ok(())
    }

    #[test]
    fn full_path_record_controls_are_matched_and_exact() -> Result<()> {
        let parts = trained_full_path_parts();
        let first = [10, 11, 12, 20, 21, 22, 23];
        let point = [30, 31, 32, 40];

        let full_first = trace(&parts.final_artifact, &first, 2)?;
        let memory_off = trace(&parts.after_hamming, &first, 2)?;
        let hamming_off = trace(&parts.hamming_off, &first, 2)?;
        let stripped = trace(&parts.stripped, &first, 2)?;
        for (row, name) in [(2, "opening"), (4, "reply 1"), (5, "reply 2")] {
            assert_ne!(
                full_first[row].query_q12, memory_off[row].query_q12,
                "memory-off did not remove the {name} move"
            );
        }
        assert_ne!(
            memory_off[2].query_q12, stripped[2].query_q12,
            "Hamming-off did not remove the Hamming query contribution"
        );
        assert_ne!(
            full_first[2].query_q12, stripped[2].query_q12,
            "the fully stripped arm retained the full query"
        );

        let point_full = trace(&parts.final_artifact, &point, 2)?;
        let point_off = trace(&parts.with_memory, &point, 2)?;
        let hamming_off_point = trace(&parts.hamming_off, &point, 2)?;
        let memory_off_point = trace(&parts.after_hamming, &point, 2)?;
        let stripped_point = trace(&parts.stripped, &point, 2)?;
        assert_eq!(point_full[2].token, 40);
        assert_ne!(
            point_full[2].token, point_off[2].token,
            "point-off did not remove the point fallback"
        );

        assert!(
            uor_r4_integer::stack::read_hamming_uorlut01(&parts.final_artifact)
                .unwrap()
                .is_some()
        );
        let full_memory = uor_r4_integer::stack::read_memory_uorlut01(&parts.final_artifact)
            .unwrap()
            .unwrap();
        assert!(full_memory.point.is_some());
        assert!(
            uor_r4_integer::stack::read_hamming_uorlut01(&parts.after_hamming)
                .unwrap()
                .is_some()
        );
        assert!(
            uor_r4_integer::stack::read_memory_uorlut01(&parts.after_hamming)
                .unwrap()
                .is_none()
        );
        assert!(
            uor_r4_integer::stack::read_hamming_uorlut01(&parts.with_memory)
                .unwrap()
                .is_some()
        );
        let point_off_memory = uor_r4_integer::stack::read_memory_uorlut01(&parts.with_memory)
            .unwrap()
            .unwrap();
        assert!(point_off_memory.point.is_none());
        let stripped_model = IntegerStackModel::parse(&parts.stripped).unwrap();
        assert!(stripped_model.turn_memory().is_none());
        assert!(
            uor_r4_integer::stack::read_hamming_uorlut01(&parts.stripped)
                .unwrap()
                .is_none()
        );
        let hamming_off_memory = uor_r4_integer::stack::read_memory_uorlut01(&parts.hamming_off)
            .unwrap()
            .unwrap();
        assert_eq!(hamming_off_memory.record.opening, 1);
        assert_eq!(hamming_off_memory.record.reply, Some(2));
        assert_eq!(hamming_off_memory.point.as_ref().unwrap().targets, [40]);
        assert!(
            uor_r4_integer::stack::read_hamming_uorlut01(&parts.hamming_off)
                .unwrap()
                .is_none()
        );

        let arms = [
            ("full", parts.final_artifact.as_slice()),
            ("memory_off", parts.after_hamming.as_slice()),
            ("point_off", parts.with_memory.as_slice()),
            ("hamming_off", parts.hamming_off.as_slice()),
            ("stripped", parts.stripped.as_slice()),
        ];
        for (name, artifact) in arms {
            for (tokens, reply_start) in [(first.as_slice(), Some(2)), (point.as_slice(), Some(2))]
            {
                let audit = audit_served_forward(
                    artifact,
                    &ServedForwardAuditConfig {
                        tokens: tokens.to_vec(),
                        token_source: None,
                        expected_artifact_sha256: None,
                        reply_start,
                        measure_timing: false,
                    },
                )
                .unwrap();
                assert!(!audit.parity.instrument_fault, "{name}");
                assert_eq!(audit.parity.query_coordinate_mismatches, 0, "{name}");
                assert_eq!(audit.parity.token_mismatches, 0, "{name}");
                assert_eq!(audit.parity.kept_cell_mismatches, 0, "{name}");
            }
        }
        assert_eq!(
            parts,
            trained_full_path_parts(),
            "the control arms are not deterministic"
        );
        eprintln!(
            "{}",
            serde_json::json!({
                "full_sha256": uor_r4_discrete::corpus::sha256(&parts.final_artifact),
                "memory_off_sha256": uor_r4_discrete::corpus::sha256(&parts.after_hamming),
                "point_off_sha256": uor_r4_discrete::corpus::sha256(&parts.with_memory),
                "hamming_off_sha256": uor_r4_discrete::corpus::sha256(&parts.hamming_off),
                "stripped_sha256": uor_r4_discrete::corpus::sha256(&parts.stripped),
                "full_point_decision": point_full[2].token,
                "point_off_decision": point_off[2].token,
                "memory_off_point_decision": memory_off_point[2].token,
                "hamming_off_point_decision": hamming_off_point[2].token,
                "stripped_point_decision": stripped_point[2].token,
                "first_memory_divergence": first_query_divergence(&full_first, &memory_off),
                "first_hamming_off_divergence": first_query_divergence(&full_first, &hamming_off),
                "first_stripped_divergence": first_query_divergence(&full_first, &stripped),
                "point_hamming_off_divergence": first_query_divergence(&point_full, &hamming_off_point),
            })
        );
        Ok(())
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
        assert_eq!(step.coordinates.len(), 1);
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
    fn rank_margin_k1_preserves_the_pre_bvgk_artifact_hash() {
        let (artifact, config, sample, query, rival) = training_case(false);
        assert_eq!(config.coordinates_per_step, 1);
        let mut trainer =
            CpuRankMarginTrainer::from_artifact(&artifact, vec![sample.clone()], config, 7)
                .unwrap();
        let width = trainer.forward.width();
        let table = trainer.forward.token_table();
        let target_row = &table[sample.target as usize * width..][..width];
        let rival_row = &table[rival as usize * width..][..width];
        let coordinate = (8..width.saturating_sub(8))
            .map(|coordinate| {
                (
                    i64::from(query[coordinate].abs_diff(i32::from(target_row[coordinate])))
                        + i64::from(query[coordinate].abs_diff(i32::from(rival_row[coordinate]))),
                    coordinate,
                )
            })
            .max_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)))
            .map(|(_, coordinate)| coordinate)
            .unwrap();
        let mut legacy = table.to_vec();
        legacy[sample.target as usize * width + coordinate] = move_toward(
            target_row[coordinate],
            query[coordinate],
            config.step_q12,
            config.parameter_limit_q12,
        );
        legacy[rival as usize * width + coordinate] = move_away(
            rival_row[coordinate],
            query[coordinate],
            config.step_q12,
            config.parameter_limit_q12,
        );
        let expected = patch_writer_table(&artifact, &legacy).unwrap();
        assert!(trainer.train_steps(1).unwrap()[0].updated);
        assert_eq!(trainer.artifact(), expected);
        let actual = uor_r4_discrete::corpus::sha256(trainer.artifact());
        assert_eq!(
            actual,
            "cddc9b4f646098e8b7140c73b6114963c2188b6d8d85701348a81c755b974816"
        );
    }

    #[test]
    fn a_table_bvgk_block_commits_multiple_coordinates_or_changes_nothing() {
        let (artifact, mut config, _, _, _) = training_case(false);
        config.coordinates_per_step = 4;
        let refused = CpuRankMarginTrainer::from_artifact(&artifact, vec![], config, 7);
        assert!(match refused {
            Ok(_) => panic!("empty samples were accepted before the block ran"),
            Err(error) => error.to_string().contains("at least one sample"),
        });

        let mut seed_probe = CpuRankMarginTrainer::from_artifact(
            &artifact,
            vec![RankMarginSample {
                input: vec![0],
                target: 0,
            }],
            config,
            7,
        )
        .unwrap();
        let samples = admitted_samples(&mut seed_probe, 16);
        let checkpoint_samples = samples.clone();
        let mut trainer =
            CpuRankMarginTrainer::from_artifact(&artifact, samples, config, 7).unwrap();
        let width = trainer.forward.width();
        let steps = trainer.train_steps(16).unwrap();
        for step in steps.iter().filter(|step| step.updated) {
            let mut distinct = step.coordinates.clone();
            distinct.sort_unstable();
            distinct.dedup();
            assert!(!step.coordinates.is_empty());
            assert!(step.coordinates.len() <= 4);
            assert_eq!(distinct.len(), step.coordinates.len());
            assert!(step
                .coordinates
                .iter()
                .all(|coordinate| (8..width.saturating_sub(8)).contains(coordinate)));
        }
        let block = steps
            .iter()
            .find(|step| step.updated && step.coordinates.len() > 1)
            .expect("no accepted multi-coordinate table block");
        assert!(block.coordinates.len() <= 4);
        assert_eq!(block.coordinate, block.coordinates.first().copied());
        assert!(block.after.margin_loss < block.before.margin_loss);
        assert!(block.after.target_admitted);
        IntegerStackModel::parse(trainer.artifact()).unwrap();
        let span = writer_span(trainer.artifact());
        for (index, (&before, &after)) in artifact.iter().zip(trainer.artifact()).enumerate() {
            if index < span.start || index >= span.end {
                assert_eq!(before, after, "non-writer byte {index} changed");
            }
        }
        let resumed =
            CpuRankMarginTrainer::resume(&trainer.checkpoint().unwrap(), checkpoint_samples)
                .unwrap();
        assert_eq!(resumed.config().coordinates_per_step, 4);
        assert_eq!(resumed.artifact(), trainer.artifact());
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
    fn a_trained_writer_preserves_the_full_served_path_and_exact_audit() {
        let artifact = full_path_artifact();
        let config = RankMarginConfig::default();
        let mut probe = CpuRankMarginTrainer::from_artifact(
            &artifact,
            vec![RankMarginSample {
                input: vec![0],
                target: 0,
            }],
            config,
            701,
        )
        .unwrap();
        let samples = admitted_samples(&mut probe, 8);
        let mut trainer =
            CpuRankMarginTrainer::from_artifact(&artifact, samples.clone(), config, 701).unwrap();
        let steps = trainer.train_steps(samples.len()).unwrap();
        let accepted = steps.iter().filter(|step| step.updated).count();
        assert_eq!(
            accepted,
            steps.len(),
            "every admitted full-path sample must accept its update"
        );
        assert_ne!(trainer.artifact(), artifact, "training wrote no artifact");

        let span = writer_span(trainer.artifact());
        for (index, (&before, &after)) in artifact.iter().zip(trainer.artifact()).enumerate() {
            if index < span.start || index >= span.end {
                assert_eq!(before, after, "non-writer byte {index} changed");
            }
        }

        let audit_config = ServedForwardAuditConfig {
            tokens: vec![0, 42, 77, 11, 22],
            token_source: None,
            expected_artifact_sha256: None,
            reply_start: Some(3),
            measure_timing: false,
        };
        let untrained = audit_served_forward(&artifact, &audit_config).unwrap();
        let trained = audit_served_forward(trainer.artifact(), &audit_config).unwrap();
        let repeated = audit_served_forward(trainer.artifact(), &audit_config).unwrap();
        assert_eq!(
            serde_json::to_vec(&trained).unwrap(),
            serde_json::to_vec(&repeated).unwrap(),
            "the trained audit is not deterministic"
        );
        for report in [&untrained, &trained] {
            assert!(!report.parity.instrument_fault);
            assert_eq!(report.head, "native_vector");
            assert_eq!(report.parity.query_coordinate_mismatches, 0);
            assert_eq!(report.parity.token_mismatches, 0);
            assert_eq!(report.parity.kept_cell_mismatches, 0);
            assert_eq!(
                report.parity.cpu_decision_sha256,
                report.parity.serving_decision_sha256
            );

            let hamming = report.manifest.hamming.as_ref().unwrap();
            assert_eq!(hamming.heads, 4);
            assert!(hamming.copy_heads > 0);
            assert!(hamming.type_heads > 0);
            assert!(
                hamming.reads.iter().any(|read| read.depth == 2),
                "the readout-depth Hamming read is not active"
            );
            let memory = report.manifest.memory.as_ref().unwrap();
            assert_eq!(memory.opening_rows, 1);
            assert_eq!(memory.reply_rows, 2);
            assert_eq!(memory.reply_turn, Some(3));
            assert_eq!(memory.point_key.as_ref().unwrap().rows, 1);
            assert!(report
                .modelled_sequence_cost
                .iter()
                .any(|stage| { stage.stage == "turn_memory" && stage.memory_key_lookups > 0 }));
            assert!(report
                .modelled_sequence_cost
                .iter()
                .any(|stage| stage.hamming_candidate_comparisons > 0));
            assert!(report
                .modelled_sequence_cost
                .iter()
                .any(|stage| stage.stage == "readout" && stage.readout_candidate_tokens > 0));
        }
        assert_ne!(
            trained.parity.cpu_decision_sha256, untrained.parity.cpu_decision_sha256,
            "the accepted writer update changed no complete served decision"
        );
        let model = IntegerStackModel::parse(trainer.artifact()).unwrap();
        let mut session = model.session();
        let mut served = Vec::new();
        for (row, &token) in audit_config.tokens.iter().enumerate() {
            if row + 1 == 3 {
                session.mark_reply_start();
            }
            served.push(session.step_native(token).unwrap());
        }
        assert_eq!(
            served[2].token, 99,
            "the trained artifact did not take the opening point-key fallback"
        );

        let checkpoint = trainer.checkpoint().unwrap();
        let resumed = CpuRankMarginTrainer::resume(&checkpoint, samples).unwrap();
        assert_eq!(resumed.artifact(), trainer.artifact());
        assert_eq!(resumed.checkpoint().unwrap(), checkpoint);
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

        let mut legacy = checkpoint[..28].to_vec();
        legacy[16..20].copy_from_slice(&1u32.to_le_bytes());
        legacy.extend_from_slice(&checkpoint[32..]);
        let legacy = CpuRankMarginTrainer::resume(&legacy, samples.clone()).unwrap();
        assert_eq!(legacy.config().coordinates_per_step, 1);

        let mut short = checkpoint.clone();
        short.truncate(short.len() - 1);
        assert!(CpuRankMarginTrainer::resume(&short, samples.clone()).is_err());
        let mut oversized = checkpoint.clone();
        oversized[104..108].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(CpuRankMarginTrainer::resume(&oversized, samples.clone()).is_err());
        let mut duplicate = checkpoint.clone();
        let first: [u8; 4] = duplicate[108..112].try_into().unwrap();
        duplicate[112..116].copy_from_slice(&first);
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
