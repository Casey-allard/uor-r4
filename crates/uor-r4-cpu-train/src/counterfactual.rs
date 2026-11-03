//! Exact CPU counterfactual updates for lawful FBS parameter tables.
//!
//! A step mirrors the discrete BVG-K rule: score one bounded block of distinct
//! coordinates, keep each coordinate's best strict single-move improvement,
//! then commit the whole eligible block only when its exact joint loss beats
//! the best member. Every proposal is patched into the artifact, parsed, and
//! evaluated through the changed full forward.

use uor_r4_integer::stack::{IntegerStackModel, StackShape};

use crate::train::{
    sample_fingerprint, shuffled_order, validate_training_model, CpuRankMarginTrainer,
    RankMarginConfig, RankMarginReading, RankMarginSample,
};
use crate::CpuTrainError;

const MAGIC: &[u8; 16] = b"UORR4-CPARAM-TRN";
const CHECKPOINT_VERSION: u32 = 2;
const BVGK_MAX_COORDINATES: usize = 64;

type Result<T> = std::result::Result<T, CpuTrainError>;

fn invalid(message: impl Into<String>) -> CpuTrainError {
    CpuTrainError::Invalid(message.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterfactualGroup {
    Normalization,
    Bias,
    Decay,
    Route,
}

impl CounterfactualGroup {
    fn code(self) -> u32 {
        match self {
            Self::Normalization => 1,
            Self::Bias => 2,
            Self::Decay => 3,
            Self::Route => 4,
        }
    }

    fn from_code(code: u32) -> Result<Self> {
        match code {
            1 => Ok(Self::Normalization),
            2 => Ok(Self::Bias),
            3 => Ok(Self::Decay),
            4 => Ok(Self::Route),
            _ => Err(invalid("counterfactual group code is unknown")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CounterfactualConfig {
    pub group: CounterfactualGroup,
    pub margin_q12: i32,
    /// Integer delta for table units or route frame/atom movement.
    pub delta: i32,
    /// Distinct coordinates scored as one BVG-K block.
    pub coordinates_per_step: usize,
}

impl Default for CounterfactualConfig {
    fn default() -> Self {
        Self {
            group: CounterfactualGroup::Normalization,
            margin_q12: 1,
            delta: 1,
            coordinates_per_step: 16,
        }
    }
}

impl CounterfactualConfig {
    fn validate(self) -> Result<()> {
        if self.margin_q12 <= 0 || self.delta <= 0 {
            return Err(invalid("counterfactual margin and delta must be positive"));
        }
        if !(1..=BVGK_MAX_COORDINATES).contains(&self.coordinates_per_step) {
            return Err(invalid(format!(
                "counterfactual coordinate block must be in 1..={BVGK_MAX_COORDINATES}"
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterfactualField {
    Value,
    RouteFrame,
    RouteAtom,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CounterfactualCandidate {
    pub table: String,
    pub index: usize,
    pub field: CounterfactualField,
    /// Semantic delta: table units or route frame steps.
    pub delta: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CounterfactualBundle {
    pub members: Vec<CounterfactualCandidate>,
    pub best: CounterfactualCandidate,
    /// `true` when the exact joint block beat the best single member.
    pub joint: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CounterfactualStep {
    pub sample: usize,
    pub target: u32,
    pub coordinates: usize,
    pub tried: usize,
    pub eligible: usize,
    pub accepted: Option<CounterfactualBundle>,
    pub before: RankMarginReading,
    pub after: RankMarginReading,
}

#[derive(Clone)]
pub struct CpuCounterfactualTrainer {
    artifact: Vec<u8>,
    evaluator: CpuRankMarginTrainer,
    samples: Vec<RankMarginSample>,
    fingerprint: [u8; 32],
    config: CounterfactualConfig,
    shape: StackShape,
    seed: u64,
    step: u64,
    epoch: u64,
    cursor: usize,
    order: Vec<usize>,
}

impl CpuCounterfactualTrainer {
    pub fn from_artifact(
        artifact: &[u8],
        samples: Vec<RankMarginSample>,
        config: CounterfactualConfig,
        seed: u64,
    ) -> Result<Self> {
        config.validate()?;
        validate_training_model(artifact)?;
        if samples.is_empty() {
            return Err(invalid("counterfactual training needs samples"));
        }
        let model = IntegerStackModel::parse(artifact)
            .map_err(|error| invalid(format!("counterfactual model: {error}")))?;
        let evaluator = rank_evaluator(artifact, &samples, config.margin_q12, seed)?;
        let fingerprint = sample_fingerprint(&samples);
        let order = shuffled_order(samples.len(), seed, 0);
        Ok(Self {
            artifact: artifact.to_vec(),
            evaluator,
            samples,
            fingerprint,
            config,
            shape: model.shape().clone(),
            seed,
            step: 0,
            epoch: 0,
            cursor: 0,
            order,
        })
    }

    pub fn artifact(&self) -> &[u8] {
        &self.artifact
    }

    pub fn step_count(&self) -> u64 {
        self.step
    }

    pub fn evaluate_sample(&mut self, sample: &RankMarginSample) -> Result<RankMarginReading> {
        self.evaluator.evaluate_sample(sample)
    }

    pub fn train_steps(&mut self, count: usize) -> Result<Vec<CounterfactualStep>> {
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            out.push(self.train_step()?);
        }
        Ok(out)
    }

    pub fn train_step(&mut self) -> Result<CounterfactualStep> {
        if self.cursor == self.order.len() {
            self.epoch = self
                .epoch
                .checked_add(1)
                .ok_or_else(|| invalid("counterfactual epoch exhausted"))?;
            self.cursor = 0;
            self.order = shuffled_order(self.samples.len(), self.seed, self.epoch);
        }
        let sample_index = self.order[self.cursor];
        let sample = self.samples[sample_index].clone();
        let before = self.evaluator.evaluate_sample(&sample)?;
        let coordinates = if before.margin_loss == 0 || !before.target_admitted {
            Vec::new()
        } else {
            self.coordinates()?
        };
        let coordinate_count = coordinates.len();
        let mut tried = 0usize;
        let mut eligible = Vec::new();
        for coordinate in coordinates {
            let mut best: Option<(
                CounterfactualCandidate,
                CpuRankMarginTrainer,
                RankMarginReading,
            )> = None;
            for candidate in coordinate.lanes(self.config.delta, &self.artifact)? {
                tried += 1;
                let patched = match patch_candidates(
                    &self.artifact,
                    std::slice::from_ref(&candidate),
                    self.config,
                ) {
                    Ok(patched) => patched,
                    Err(_) => continue,
                };
                let mut evaluator = match rank_evaluator(
                    &patched,
                    &self.samples,
                    self.config.margin_q12,
                    self.seed,
                ) {
                    Ok(evaluator) => evaluator,
                    Err(_) => continue,
                };
                let after = evaluator.evaluate_sample(&sample)?;
                if !candidate_improves(&after, &before) {
                    continue;
                }
                let replace = best
                    .as_ref()
                    .is_none_or(|(_, _, current)| after.margin_loss < current.margin_loss);
                if replace {
                    best = Some((candidate, evaluator, after));
                }
            }
            if let Some((candidate, evaluator, after)) = best {
                eligible.push((candidate, evaluator, after));
            }
        }
        let eligible_count = eligible.len();
        let accepted = self.accept_eligible(&sample, &before, eligible)?;
        self.step = self
            .step
            .checked_add(1)
            .ok_or_else(|| invalid("counterfactual step exhausted"))?;
        self.cursor += 1;
        Ok(CounterfactualStep {
            sample: sample_index,
            target: sample.target,
            coordinates: coordinate_count,
            tried,
            eligible: eligible_count,
            accepted: accepted.as_ref().map(|(bundle, _)| bundle.clone()),
            before: before.clone(),
            after: accepted.map_or(before, |(_, reading)| reading),
        })
    }

    #[allow(clippy::type_complexity)]
    fn accept_eligible(
        &mut self,
        sample: &RankMarginSample,
        before: &RankMarginReading,
        mut eligible: Vec<(
            CounterfactualCandidate,
            CpuRankMarginTrainer,
            RankMarginReading,
        )>,
    ) -> Result<Option<(CounterfactualBundle, RankMarginReading)>> {
        let Some(best_at) = eligible
            .iter()
            .enumerate()
            .min_by_key(|(_, (_, _, reading))| reading.margin_loss)
            .map(|(index, _)| index)
        else {
            return Ok(None);
        };
        let (best, best_evaluator, best_reading) = eligible.swap_remove(best_at);
        if eligible.is_empty() {
            self.artifact = best_evaluator.artifact().to_vec();
            self.evaluator = best_evaluator;
            return Ok(Some((
                CounterfactualBundle {
                    members: vec![best.clone()],
                    best,
                    joint: false,
                },
                best_reading,
            )));
        }

        let mut members = vec![best.clone()];
        let mut keys = vec![(best.table.clone(), best.index)];
        for (candidate, _, _) in eligible {
            let key = (candidate.table.clone(), candidate.index);
            if !keys.contains(&key) {
                keys.push(key);
                members.push(candidate);
            }
        }
        let joint_patch = patch_candidates(&self.artifact, &members, self.config)?;
        let mut joint_evaluator = rank_evaluator(
            &joint_patch,
            &self.samples,
            self.config.margin_q12,
            self.seed,
        )?;
        let joint_reading = joint_evaluator.evaluate_sample(sample)?;
        let joint = joint_reading.margin_loss < best_reading.margin_loss
            && joint_reading.target_admitted
            && joint_reading.margin_loss < before.margin_loss;
        if joint {
            self.artifact = joint_patch;
            self.evaluator = joint_evaluator;
            Ok(Some((
                CounterfactualBundle {
                    members,
                    best,
                    joint: true,
                },
                joint_reading,
            )))
        } else {
            self.artifact = best_evaluator.artifact().to_vec();
            self.evaluator = best_evaluator;
            Ok(Some((
                CounterfactualBundle {
                    members: vec![best.clone()],
                    best,
                    joint: false,
                },
                best_reading,
            )))
        }
    }

    pub fn checkpoint(&self) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(128 + self.order.len() * 4 + self.artifact.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&CHECKPOINT_VERSION.to_le_bytes());
        out.extend_from_slice(&self.config.group.code().to_le_bytes());
        out.extend_from_slice(&self.config.margin_q12.to_le_bytes());
        out.extend_from_slice(&self.config.delta.to_le_bytes());
        out.extend_from_slice(
            &u32::try_from(self.config.coordinates_per_step)
                .map_err(|_| invalid("counterfactual coordinate block exceeds u32"))?
                .to_le_bytes(),
        );
        out.extend_from_slice(&self.seed.to_le_bytes());
        out.extend_from_slice(&self.step.to_le_bytes());
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&(self.cursor as u64).to_le_bytes());
        out.extend_from_slice(&self.fingerprint);
        out.extend_from_slice(
            &u32::try_from(self.order.len())
                .map_err(|_| invalid("counterfactual order length exceeds u32"))?
                .to_le_bytes(),
        );
        for &index in &self.order {
            out.extend_from_slice(
                &u32::try_from(index)
                    .map_err(|_| invalid("counterfactual sample index exceeds u32"))?
                    .to_le_bytes(),
            );
        }
        out.extend_from_slice(
            &u64::try_from(self.artifact.len())
                .map_err(|_| invalid("counterfactual artifact length exceeds u64"))?
                .to_le_bytes(),
        );
        out.extend_from_slice(&self.artifact);
        Ok(out)
    }

    pub fn resume(checkpoint: &[u8], samples: Vec<RankMarginSample>) -> Result<Self> {
        let mut input = Reader::new(checkpoint);
        if input.take(MAGIC.len())? != MAGIC || input.u32()? != CHECKPOINT_VERSION {
            return Err(invalid("counterfactual checkpoint identity differs"));
        }
        let config = CounterfactualConfig {
            group: CounterfactualGroup::from_code(input.u32()?)?,
            margin_q12: input.i32()?,
            delta: input.i32()?,
            coordinates_per_step: usize::try_from(input.u32()?)
                .map_err(|_| invalid("counterfactual coordinate block exceeds usize"))?,
        };
        config.validate()?;
        let seed = input.u64()?;
        let step = input.u64()?;
        let epoch = input.u64()?;
        let cursor = usize::try_from(input.u64()?).map_err(|_| invalid("cursor"))?;
        let fingerprint: [u8; 32] = input.take(32)?.try_into().unwrap();
        if fingerprint != sample_fingerprint(&samples) {
            return Err(invalid("counterfactual checkpoint samples differ"));
        }
        let order_len = usize::try_from(input.u32()?)
            .map_err(|_| invalid("counterfactual order length exceeds usize"))?;
        if order_len != samples.len() {
            return Err(invalid("counterfactual checkpoint order length differs"));
        }
        let mut order = Vec::with_capacity(order_len);
        let mut seen = vec![false; samples.len()];
        for _ in 0..order_len {
            let index = usize::try_from(input.u32()?)
                .map_err(|_| invalid("counterfactual order index exceeds usize"))?;
            if index >= samples.len() || std::mem::replace(&mut seen[index], true) {
                return Err(invalid("counterfactual checkpoint order is invalid"));
            }
            order.push(index);
        }
        let artifact_len = usize::try_from(input.u64()?).map_err(|_| invalid("artifact length"))?;
        let artifact = input.take(artifact_len)?.to_vec();
        if !input.done() || cursor > order.len() {
            return Err(invalid(
                "counterfactual checkpoint cursor/trailing data differs",
            ));
        }
        validate_training_model(&artifact)?;
        let model = IntegerStackModel::parse(&artifact)
            .map_err(|error| invalid(format!("counterfactual checkpoint model: {error}")))?;
        let evaluator = rank_evaluator(&artifact, &samples, config.margin_q12, seed)?;
        Ok(Self {
            artifact,
            evaluator,
            samples,
            fingerprint,
            config,
            shape: model.shape().clone(),
            seed,
            step,
            epoch,
            cursor,
            order,
        })
    }

    fn coordinates(&self) -> Result<Vec<CandidateCoordinate>> {
        let specs = table_specs(self.config.group, &self.shape);
        let mut all = Vec::new();
        for spec in specs {
            let (len, _) = table_section(&self.artifact, &spec.name, spec.kind)?;
            for index in 0..len {
                match self.config.group {
                    CounterfactualGroup::Route => {
                        all.push(CandidateCoordinate {
                            group: self.config.group,
                            table: spec.name.clone(),
                            index,
                            field: CounterfactualField::RouteFrame,
                            route_input_atoms: spec.input_atoms,
                        });
                    }
                    _ => all.push(CandidateCoordinate {
                        group: self.config.group,
                        table: spec.name.clone(),
                        index,
                        field: CounterfactualField::Value,
                        route_input_atoms: None,
                    }),
                }
            }
        }
        if all.is_empty() {
            return Ok(all);
        }
        let start = (self.step as usize) % all.len();
        let count = self.config.coordinates_per_step.min(all.len());
        Ok((0..count)
            .map(|offset| all[(start + offset) % all.len()].clone())
            .collect())
    }
}

#[derive(Clone)]
struct CandidateCoordinate {
    group: CounterfactualGroup,
    table: String,
    index: usize,
    field: CounterfactualField,
    route_input_atoms: Option<usize>,
}

impl CandidateCoordinate {
    fn lanes(&self, delta: i32, artifact: &[u8]) -> Result<Vec<CounterfactualCandidate>> {
        let kind = table_kind(self.group, &self.table)
            .ok_or_else(|| invalid("candidate coordinate table has no legal group"))?;
        let (_, bytes) = table_section(artifact, &self.table, kind)?;
        let at = self.index * kind.width();
        let mut out = Vec::with_capacity(if self.group == CounterfactualGroup::Route {
            4
        } else {
            2
        });
        let fields: &[CounterfactualField] = if self.group == CounterfactualGroup::Route {
            &[
                CounterfactualField::RouteFrame,
                CounterfactualField::RouteAtom,
            ]
        } else {
            std::slice::from_ref(&self.field)
        };
        for &field in fields {
            for candidate_delta in [delta, -delta] {
                if self.legal_lane(field, candidate_delta, kind, bytes, at)? {
                    out.push(CounterfactualCandidate {
                        table: self.table.clone(),
                        index: self.index,
                        field,
                        delta: candidate_delta,
                    });
                }
            }
        }
        Ok(out)
    }

    fn legal_lane(
        &self,
        field: CounterfactualField,
        delta: i32,
        kind: TableKind,
        bytes: &[u8],
        at: usize,
    ) -> Result<bool> {
        match (kind, field) {
            (TableKind::I16, CounterfactualField::Value) => {
                let old = i16::from_le_bytes(
                    bytes
                        .get(at..at + 2)
                        .ok_or_else(|| invalid("gain candidate is outside its table"))?
                        .try_into()
                        .unwrap(),
                );
                let Some(next) = i32::from(old).checked_add(delta) else {
                    return Ok(false);
                };
                Ok(
                    next != i32::from(old)
                        && i16::try_from(next).is_ok()
                        && grid_valid(next as i16),
                )
            }
            (TableKind::I32, CounterfactualField::Value) => {
                let old = i32::from_le_bytes(
                    bytes
                        .get(at..at + 4)
                        .ok_or_else(|| invalid("integer candidate is outside its table"))?
                        .try_into()
                        .unwrap(),
                );
                Ok(old.checked_add(delta).is_some_and(|next| next != old))
            }
            (TableKind::U32, CounterfactualField::RouteFrame) => Ok(delta.rem_euclid(64) != 0),
            (TableKind::U32, CounterfactualField::RouteAtom) => {
                let old = u32::from_le_bytes(
                    bytes
                        .get(at..at + 4)
                        .ok_or_else(|| invalid("route candidate is outside its table"))?
                        .try_into()
                        .unwrap(),
                );
                let atom = old & 0xffff;
                let Some(atom) = i64::from(atom).checked_add(i64::from(delta)) else {
                    return Ok(false);
                };
                Ok(atom >= 0
                    && atom < self.route_input_atoms.unwrap_or(0) as i64
                    && atom != i64::from(old & 0xffff))
            }
            _ => Err(invalid("counterfactual table kind and field differ")),
        }
    }
}

fn rank_evaluator(
    artifact: &[u8],
    samples: &[RankMarginSample],
    margin_q12: i32,
    seed: u64,
) -> Result<CpuRankMarginTrainer> {
    CpuRankMarginTrainer::from_artifact(
        artifact,
        samples.to_vec(),
        RankMarginConfig {
            margin_q12,
            step_q12: 1,
            parameter_limit_q12: 32_767,
            coordinates_per_step: 1,
        },
        seed,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TableKind {
    I16,
    I32,
    U32,
}

#[derive(Clone)]
struct TableSpec {
    name: String,
    kind: TableKind,
    input_atoms: Option<usize>,
}

fn table_specs(group: CounterfactualGroup, shape: &StackShape) -> Vec<TableSpec> {
    let mut out = Vec::new();
    for layer in 0..shape.layers() {
        let mut push = |name: String, kind, input_atoms| {
            out.push(TableSpec {
                name,
                kind,
                input_atoms,
            })
        };
        match group {
            CounterfactualGroup::Normalization => {
                push(format!("l{layer}.rec_norm_gain"), TableKind::I16, None);
                push(format!("l{layer}.mlp_norm_gain"), TableKind::I16, None);
            }
            CounterfactualGroup::Bias => {
                push(format!("l{layer}.conv_bias"), TableKind::I32, None);
                push(format!("l{layer}.gate_bias"), TableKind::I32, None);
            }
            CounterfactualGroup::Decay => {
                push(format!("l{layer}.decay_offset"), TableKind::I32, None);
            }
            CounterfactualGroup::Route => {
                for suffix in [
                    "rec_in.fbs",
                    "rec_gate.fbs",
                    "rec_out.fbs",
                    "mlp_gate.fbs",
                    "mlp_up.fbs",
                    "mlp_down.fbs",
                ] {
                    push(
                        format!("l{layer}.{suffix}"),
                        TableKind::U32,
                        Some(route_input_atoms(suffix, shape)),
                    );
                }
            }
        }
    }
    if group == CounterfactualGroup::Normalization {
        out.push(TableSpec {
            name: "final_norm_gain".to_owned(),
            kind: TableKind::I16,
            input_atoms: None,
        });
    }
    out
}

fn route_input_atoms(suffix: &str, shape: &StackShape) -> usize {
    match suffix {
        "mlp_down.fbs" => shape.mlp / 8,
        _ => shape.width / 8,
    }
}

fn strictly_better(after: &RankMarginReading, before: &RankMarginReading) -> bool {
    after.margin_loss < before.margin_loss
}

fn candidate_improves(after: &RankMarginReading, before: &RankMarginReading) -> bool {
    strictly_better(after, before) && after.target_admitted
}

fn grid_valid(code: i16) -> bool {
    code == 0 || (1..=127).contains(&(code.unsigned_abs() >> 4))
}

fn patch_candidates(
    artifact: &[u8],
    candidates: &[CounterfactualCandidate],
    config: CounterfactualConfig,
) -> Result<Vec<u8>> {
    if candidates.is_empty() || candidates.len() > BVGK_MAX_COORDINATES {
        return Err(invalid(
            "counterfactual bundle must contain 1..=64 candidates",
        ));
    }
    let mut patched = artifact.to_vec();
    let mut seen = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if seen
            .iter()
            .any(|&(table, index)| table == candidate.table && index == candidate.index)
        {
            return Err(invalid("counterfactual bundle repeats one table entry"));
        }
        seen.push((candidate.table.as_str(), candidate.index));
        let kind = table_kind(config.group, &candidate.table)
            .ok_or_else(|| invalid("counterfactual candidate table is not in its group"))?;
        let legal_field = match kind {
            TableKind::I16 | TableKind::I32 => candidate.field == CounterfactualField::Value,
            TableKind::U32 => matches!(
                candidate.field,
                CounterfactualField::RouteFrame | CounterfactualField::RouteAtom
            ),
        };
        if !legal_field {
            return Err(invalid("counterfactual table kind and field differ"));
        }
        let (len, table_at) = table_location(artifact, &candidate.table, kind)?;
        if candidate.index >= len {
            return Err(invalid(
                "counterfactual candidate index is outside its table",
            ));
        }
        let at = table_at + candidate.index * kind.width();
        match kind {
            TableKind::I16 => {
                let old = i16::from_le_bytes(patched[at..at + 2].try_into().unwrap());
                let next = i32::from(old)
                    .checked_add(candidate.delta)
                    .and_then(|value| i16::try_from(value).ok())
                    .ok_or_else(|| invalid("counterfactual gain delta overflows"))?;
                if next == old {
                    return Err(invalid("counterfactual gain move is a no-op"));
                }
                patched[at..at + 2].copy_from_slice(&next.to_le_bytes());
            }
            TableKind::I32 => {
                let old = i32::from_le_bytes(patched[at..at + 4].try_into().unwrap());
                let next = old
                    .checked_add(candidate.delta)
                    .ok_or_else(|| invalid("counterfactual integer delta overflows"))?;
                if next == old {
                    return Err(invalid("counterfactual integer move is a no-op"));
                }
                patched[at..at + 4].copy_from_slice(&next.to_le_bytes());
            }
            TableKind::U32 => {
                let old = u32::from_le_bytes(patched[at..at + 4].try_into().unwrap());
                let next = match candidate.field {
                    CounterfactualField::RouteFrame => {
                        let next_state =
                            ((((old >> 16) & 63) as i32 + candidate.delta).rem_euclid(64)) as u32;
                        (old & 0xffff) | (next_state << 16)
                    }
                    CounterfactualField::RouteAtom => {
                        let next_atom = i64::from(old & 0xffff) + i64::from(candidate.delta);
                        if !(0..=u16::MAX as i64).contains(&next_atom) {
                            return Err(invalid("counterfactual route atom delta overflows"));
                        }
                        (old & 0xffff_0000) | next_atom as u32
                    }
                    CounterfactualField::Value => unreachable!("field checked above"),
                };
                if next == old {
                    return Err(invalid("counterfactual route move is a no-op"));
                }
                patched[at..at + 4].copy_from_slice(&next.to_le_bytes());
            }
        }
    }
    IntegerStackModel::parse(&patched)
        .map_err(|error| invalid(format!("counterfactual artifact does not parse: {error}")))?;
    Ok(patched)
}

fn table_section<'a>(artifact: &'a [u8], name: &str, kind: TableKind) -> Result<(usize, &'a [u8])> {
    let (len, start) = table_location(artifact, name, kind)?;
    let bytes = artifact
        .get(start..start + len * kind.width())
        .ok_or_else(|| invalid(format!("artifact table {name} is outside the file")))?;
    Ok((len, bytes))
}

fn table_location(artifact: &[u8], name: &str, kind: TableKind) -> Result<(usize, usize)> {
    let header = header(artifact)?;
    let entry = header["tables"]
        .as_array()
        .and_then(|tables| {
            tables
                .iter()
                .find(|entry| entry["name"].as_str() == Some(name))
        })
        .ok_or_else(|| invalid(format!("artifact table {name} is absent")))?;
    let expected = match kind {
        TableKind::I16 => "i16",
        TableKind::I32 => "i32",
        TableKind::U32 => "u32",
    };
    if entry["kind"].as_str() != Some(expected) {
        return Err(invalid(format!("artifact table {name} kind differs")));
    }
    let len = usize::try_from(
        entry["len"]
            .as_u64()
            .ok_or_else(|| invalid(format!("artifact table {name} length is absent")))?,
    )
    .map_err(|_| invalid(format!("artifact table {name} length exceeds usize")))?;
    let offset = usize::try_from(
        entry["span"]["offset"]
            .as_u64()
            .ok_or_else(|| invalid("artifact span offset is absent"))?,
    )
    .map_err(|_| invalid("artifact span offset exceeds usize"))?;
    Ok((len, data_start(artifact)? + offset))
}

impl TableKind {
    fn width(self) -> usize {
        match self {
            Self::I16 => 2,
            Self::I32 | Self::U32 => 4,
        }
    }
}

fn table_kind(group: CounterfactualGroup, name: &str) -> Option<TableKind> {
    let suffix = name.split_once('.').map_or(name, |(_, suffix)| suffix);
    match group {
        CounterfactualGroup::Normalization
            if matches!(suffix, "rec_norm_gain" | "mlp_norm_gain") =>
        {
            Some(TableKind::I16)
        }
        CounterfactualGroup::Bias if matches!(suffix, "conv_bias" | "gate_bias") => {
            Some(TableKind::I32)
        }
        CounterfactualGroup::Decay if suffix == "decay_offset" => Some(TableKind::I32),
        CounterfactualGroup::Route
            if matches!(
                suffix,
                "rec_in.fbs"
                    | "rec_gate.fbs"
                    | "rec_out.fbs"
                    | "mlp_gate.fbs"
                    | "mlp_up.fbs"
                    | "mlp_down.fbs"
            ) =>
        {
            Some(TableKind::U32)
        }
        _ => None,
    }
}

fn header(artifact: &[u8]) -> Result<serde_json::Value> {
    if artifact.len() < 16 || &artifact[..8] != b"UORLUT01" {
        return Err(invalid("counterfactual artifact magic differs"));
    }
    let len = u64::from_le_bytes(artifact[8..16].try_into().unwrap());
    let end = 16 + usize::try_from(len).map_err(|_| invalid("header length"))?;
    serde_json::from_slice(
        artifact
            .get(16..end)
            .ok_or_else(|| invalid("counterfactual header is truncated"))?,
    )
    .map_err(|error| invalid(format!("counterfactual header: {error}")))
}

fn data_start(artifact: &[u8]) -> Result<usize> {
    let len = u64::from_le_bytes(
        artifact
            .get(8..16)
            .ok_or_else(|| invalid("counterfactual artifact is truncated"))?
            .try_into()
            .unwrap(),
    );
    let end = 16 + usize::try_from(len).map_err(|_| invalid("header length"))?;
    Ok(end.div_ceil(64) * 64)
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
            .ok_or_else(|| invalid("counterfactual checkpoint is truncated"))?;
        let out = &self.bytes[self.at..end];
        self.at = end;
        Ok(out)
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn done(&self) -> bool {
        self.at == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VOCABULARY;
    use uor_r4_discrete::normalization_mechanism::orbit_root_coords;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    fn artifact() -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            211,
            FbsSynth {
                vocab: VOCABULARY,
                d: 64,
                mlp: 64,
                layers: 1,
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
                let root =
                    ((token >> 8) * 7 + ((token >> 4) & 15) * 3 + (token & 15) * 5 + block) & 15;
                snapshot.embed.extend(
                    orbit_root_coords(root as u8)
                        .unwrap()
                        .map(|value| value as f32),
                );
            }
        }
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn sample_at(artifact: &[u8], input: u32) -> Option<RankMarginSample> {
        let mut probe = rank_evaluator(
            artifact,
            &[RankMarginSample {
                input: vec![0],
                target: 0,
            }],
            1,
            19,
        )
        .unwrap();
        let scan = RankMarginSample {
            input: vec![input],
            target: 0,
        };
        let reading = probe.evaluate_sample(&scan).unwrap();
        reading.top8.iter().rev().copied().find_map(|target| {
            if target == reading.top1 {
                return None;
            }
            let sample = RankMarginSample {
                input: vec![input],
                target,
            };
            (probe.evaluate_sample(&sample).unwrap().margin_loss > 0).then_some(sample)
        })
    }

    fn trainer(
        artifact: &[u8],
        sample: RankMarginSample,
        group: CounterfactualGroup,
        delta: i32,
        coordinates_per_step: usize,
    ) -> CpuCounterfactualTrainer {
        CpuCounterfactualTrainer::from_artifact(
            artifact,
            vec![sample],
            CounterfactualConfig {
                group,
                margin_q12: 1,
                delta,
                coordinates_per_step,
            },
            23,
        )
        .unwrap()
    }

    fn fixture_step(
        group: CounterfactualGroup,
        delta: i32,
    ) -> Option<(CpuCounterfactualTrainer, CounterfactualStep)> {
        let artifact = artifact();
        let dummy = RankMarginSample {
            input: vec![0],
            target: 0,
        };
        let probe = trainer(&artifact, dummy.clone(), group, delta, 64);
        let coordinate = probe.coordinates().unwrap().into_iter().next()?;
        let candidate = coordinate
            .lanes(delta, &artifact)
            .unwrap()
            .into_iter()
            .next()?;
        let patched = patch_candidates(&artifact, &[candidate], probe.config).ok()?;
        let mut patched_eval =
            rank_evaluator(&patched, std::slice::from_ref(&dummy), 1, 23).unwrap();
        let query = patched_eval.query_for(&dummy).unwrap();
        let patched_reading = patched_eval.reading(&query, 0).unwrap();
        let mut baseline = rank_evaluator(&artifact, std::slice::from_ref(&dummy), 1, 23).unwrap();
        for target in (0..VOCABULARY as u32)
            .filter(|&token| patched_reading.admitted_cells.contains(&(token % 256)))
        {
            let sample = RankMarginSample {
                input: vec![0],
                target,
            };
            let before = baseline.evaluate_sample(&sample).unwrap();
            if !before.target_admitted || before.margin_loss == 0 {
                continue;
            }
            let mut trainer = trainer(&artifact, sample, group, delta, 64);
            let step = trainer.train_steps(1).unwrap().remove(0);
            if step.accepted.is_some() {
                return Some((trainer, step));
            }
        }
        None
    }

    #[test]
    fn normalization_candidates_change_only_a_legal_grid_code() {
        let artifact = artifact();
        let (_, bytes) = table_section(&artifact, "l0.rec_norm_gain", TableKind::I16).unwrap();
        let index = bytes
            .chunks_exact(2)
            .position(|code| {
                let code = i16::from_le_bytes(code.try_into().unwrap());
                grid_valid(code.saturating_add(16))
            })
            .expect("a legal gain shift");
        let candidate = CounterfactualCandidate {
            table: "l0.rec_norm_gain".to_owned(),
            index,
            field: CounterfactualField::Value,
            delta: 16,
        };
        let patched = patch_candidates(
            &artifact,
            &[candidate],
            CounterfactualConfig {
                group: CounterfactualGroup::Normalization,
                ..CounterfactualConfig::default()
            },
        )
        .unwrap();
        let changed: Vec<usize> = artifact
            .iter()
            .zip(&patched)
            .enumerate()
            .filter_map(|(at, (&old, &new))| (old != new).then_some(at))
            .collect();
        let (_, start) = table_location(&artifact, "l0.rec_norm_gain", TableKind::I16).unwrap();
        assert!(changed
            .iter()
            .all(|&at| { (start + index * 2..start + index * 2 + 2).contains(&at) }));
        assert!(IntegerStackModel::parse(&patched).is_ok());
    }

    #[test]
    fn malformed_spans_and_grid_boundaries_are_refused() {
        let mut artifact = artifact();
        let (_, start) = table_location(&artifact, "l0.rec_norm_gain", TableKind::I16).unwrap();
        artifact[start..start + 2].copy_from_slice(&2047i16.to_le_bytes());
        assert!(IntegerStackModel::parse(&artifact).is_ok());
        let overflow_code = CounterfactualCandidate {
            table: "l0.rec_norm_gain".to_owned(),
            index: 0,
            field: CounterfactualField::Value,
            delta: 1,
        };
        assert!(patch_candidates(
            &artifact,
            &[overflow_code],
            CounterfactualConfig {
                group: CounterfactualGroup::Normalization,
                ..CounterfactualConfig::default()
            },
        )
        .is_err());

        let mut truncated = artifact.clone();
        truncated.truncate(truncated.len() - 1);
        let candidate = CounterfactualCandidate {
            table: "l0.rec_norm_gain".to_owned(),
            index: 0,
            field: CounterfactualField::Value,
            delta: 16,
        };
        assert!(patch_candidates(
            &truncated,
            &[candidate],
            CounterfactualConfig {
                group: CounterfactualGroup::Normalization,
                ..CounterfactualConfig::default()
            },
        )
        .is_err());
    }

    #[test]
    fn route_frame_and_atom_moves_keep_legal_fields() {
        let mut artifact = artifact();
        let name = "l0.rec_in.fbs";
        let (len, _) = table_section(&artifact, name, TableKind::U32).unwrap();
        assert!(len > 0);
        let index = 0;
        let (_, at) = table_location(&artifact, name, TableKind::U32).unwrap();
        let initial = u32::from_le_bytes(
            artifact[at + index * 4..at + index * 4 + 4]
                .try_into()
                .unwrap(),
        );
        artifact[at + index * 4..at + index * 4 + 4]
            .copy_from_slice(&(initial & 0xffff).to_le_bytes());
        assert!(IntegerStackModel::parse(&artifact).is_ok());
        let frame = CounterfactualCandidate {
            table: name.to_owned(),
            index,
            field: CounterfactualField::RouteFrame,
            delta: -1,
        };
        let old = u32::from_le_bytes(
            artifact[at + index * 4..at + index * 4 + 4]
                .try_into()
                .unwrap(),
        );
        let atom = old & 0xffff;
        let atom_delta = if atom + 1 < 8 { 1 } else { -1 };
        let atom = CounterfactualCandidate {
            table: name.to_owned(),
            index,
            field: CounterfactualField::RouteAtom,
            delta: atom_delta,
        };
        let frame_patch = patch_candidates(
            &artifact,
            &[frame],
            CounterfactualConfig {
                group: CounterfactualGroup::Route,
                ..CounterfactualConfig::default()
            },
        )
        .unwrap();
        let new = u32::from_le_bytes(
            frame_patch[at + index * 4..at + index * 4 + 4]
                .try_into()
                .unwrap(),
        );
        assert_eq!(old & 0xffff, new & 0xffff);
        assert_eq!((new >> 16) & 63, 63);

        let atom_patch = patch_candidates(
            &artifact,
            &[atom],
            CounterfactualConfig {
                group: CounterfactualGroup::Route,
                ..CounterfactualConfig::default()
            },
        )
        .unwrap();
        let new = u32::from_le_bytes(
            atom_patch[at + index * 4..at + index * 4 + 4]
                .try_into()
                .unwrap(),
        );
        assert_eq!(new & 0xffff, (old & 0xffff).wrapping_add_signed(atom_delta));
        assert_eq!(new & 0xffff_0000, old & 0xffff_0000);

        let outside = CounterfactualCandidate {
            table: name.to_owned(),
            index,
            field: CounterfactualField::RouteAtom,
            delta: -(i64::from(old & 0xffff) as i32) - 1,
        };
        assert!(patch_candidates(
            &artifact,
            &[outside],
            CounterfactualConfig {
                group: CounterfactualGroup::Route,
                ..CounterfactualConfig::default()
            },
        )
        .is_err());
    }

    #[test]
    fn coordinated_bvgk_steps_are_bounded_and_deterministic() {
        let artifact = artifact();
        let sample = sample_at(&artifact, 0).expect("admitted non-top-1 sample");
        let mut left = trainer(
            &artifact,
            sample.clone(),
            CounterfactualGroup::Normalization,
            16,
            16,
        );
        let mut right = trainer(
            &artifact,
            sample,
            CounterfactualGroup::Normalization,
            16,
            16,
        );
        let left_steps = left.train_steps(3).unwrap();
        let right_steps = right.train_steps(3).unwrap();
        assert_eq!(left_steps, right_steps);
        assert_eq!(left.checkpoint().unwrap(), right.checkpoint().unwrap());
        assert!(left_steps
            .iter()
            .all(|step| step.coordinates <= 16 && step.tried <= 32));
    }

    #[test]
    fn admission_and_satisfied_margin_are_no_ops() {
        let artifact = artifact();
        let sample = sample_at(&artifact, 0).unwrap();
        let mut probe = rank_evaluator(&artifact, std::slice::from_ref(&sample), 1, 29).unwrap();
        let before = probe.evaluate_sample(&sample).unwrap();
        let target_cell = (0u32..256)
            .find(|cell| !before.admitted_cells.contains(cell))
            .unwrap();
        let target = (0u32..VOCABULARY as u32)
            .find(|&token| token % 256 == target_cell)
            .unwrap();
        let mut trainer = CpuCounterfactualTrainer::from_artifact(
            &artifact,
            vec![RankMarginSample {
                input: sample.input,
                target,
            }],
            CounterfactualConfig {
                group: CounterfactualGroup::Bias,
                margin_q12: 1,
                delta: 16,
                coordinates_per_step: 16,
            },
            29,
        )
        .unwrap();
        let unchanged = artifact.clone();
        let step = trainer.train_steps(1).unwrap().remove(0);
        assert_eq!(step.coordinates, 0);
        assert_eq!(trainer.artifact(), unchanged);
    }

    #[test]
    fn admission_loss_is_never_an_improving_candidate() {
        let reading = |margin_loss, target_rank, target_admitted| RankMarginReading {
            target_distance: 10,
            target_rank,
            margin_loss,
            top1: 1,
            top8: vec![1, 2],
            target_admitted,
            target_cell: 3,
            admitted_cells: vec![3, 4],
        };
        let before = reading(20, 8, true);
        assert!(candidate_improves(&reading(10, 7, true), &before));
        assert!(!candidate_improves(&reading(10, 7, false), &before));
        assert!(!candidate_improves(&reading(20, 8, true), &before));
        assert!(!candidate_improves(&reading(20, 7, true), &before));
    }

    #[test]
    fn full_forward_rejections_leave_every_artifact_unchanged() {
        let artifact = artifact();
        let sample = sample_at(&artifact, 0).unwrap();
        for (group, delta) in [
            (CounterfactualGroup::Normalization, 16),
            (CounterfactualGroup::Bias, 1_000_000),
            (CounterfactualGroup::Decay, 1_000_000),
            (CounterfactualGroup::Route, 1),
        ] {
            let mut trainer = trainer(&artifact, sample.clone(), group, delta, 16);
            let step = trainer.train_steps(1).unwrap().remove(0);
            assert!(step.tried > 0, "{group:?} evaluated no candidates");
            assert!(step.accepted.is_none(), "{group:?} unexpectedly changed");
            assert_eq!(step.after, step.before);
            assert_eq!(trainer.artifact(), artifact);
        }
    }

    #[test]
    fn a_patched_full_forward_candidate_that_loses_admission_is_rejected() {
        let artifact = artifact();
        for (group, delta) in [
            (CounterfactualGroup::Normalization, 32_767),
            (CounterfactualGroup::Bias, i32::MAX / 2),
            (CounterfactualGroup::Decay, i32::MAX / 2),
            (CounterfactualGroup::Route, 1),
            (CounterfactualGroup::Route, 7),
        ] {
            for input in 0u32..64 {
                let dummy = RankMarginSample {
                    input: vec![input],
                    target: 0,
                };
                let trainer = trainer(&artifact, dummy.clone(), group, delta, 64);
                let mut baseline =
                    rank_evaluator(&artifact, std::slice::from_ref(&dummy), 1, 23).unwrap();
                let query = baseline.query_for(&dummy).unwrap();
                let baseline_reading = baseline.reading(&query, 0).unwrap();
                for coordinate in trainer.coordinates().unwrap() {
                    for candidate in coordinate.lanes(delta, &artifact).unwrap() {
                        let Ok(patched) = patch_candidates(
                            &artifact,
                            std::slice::from_ref(&candidate),
                            trainer.config,
                        ) else {
                            continue;
                        };
                        let mut changed =
                            rank_evaluator(&patched, std::slice::from_ref(&dummy), 1, 23).unwrap();
                        let changed_query = changed.query_for(&dummy).unwrap();
                        let changed_reading = changed.reading(&changed_query, 0).unwrap();
                        let Some(target) = (0..VOCABULARY as u32).find(|&token| {
                            baseline_reading.admitted_cells.contains(&(token % 256))
                                && !changed_reading.admitted_cells.contains(&(token % 256))
                        }) else {
                            continue;
                        };
                        let sample = RankMarginSample {
                            input: vec![input],
                            target,
                        };
                        let before = baseline.evaluate_sample(&sample).unwrap();
                        let after = changed.evaluate_sample(&sample).unwrap();
                        if before.target_admitted && !after.target_admitted {
                            assert!(!candidate_improves(&after, &before));
                            let mut baseline_panel =
                                crate::CpuPanelEvaluator::from_artifact(&artifact, vec![sample], 1)
                                    .unwrap();
                            let mut changed_panel = crate::CpuPanelEvaluator::from_artifact(
                                &patched,
                                vec![RankMarginSample {
                                    input: vec![input],
                                    target,
                                }],
                                1,
                            )
                            .unwrap();
                            let before_panel = baseline_panel.evaluate().unwrap();
                            let after_panel = changed_panel.evaluate().unwrap();
                            assert!(matches!(
                                crate::panel_admission_gate(&before_panel, &after_panel).unwrap(),
                                crate::PanelGateDecision::RejectAdmission(_)
                            ));
                            return;
                        }
                    }
                }
            }
        }
        panic!("no patched candidate lost target admission");
    }

    #[test]
    fn checkpoint_and_malformed_candidate_controls_hold() {
        let artifact = artifact();
        let sample = sample_at(&artifact, 0).unwrap();
        let mut trainer = trainer(
            &artifact,
            sample,
            CounterfactualGroup::Normalization,
            16,
            16,
        );
        trainer.train_steps(2).unwrap();
        let checkpoint = trainer.checkpoint().unwrap();
        let mut resumed =
            CpuCounterfactualTrainer::resume(&checkpoint, trainer.samples.clone()).unwrap();
        let left = trainer.train_steps(2).unwrap();
        let right = resumed.train_steps(2).unwrap();
        assert_eq!(left, right);
        assert_eq!(trainer.checkpoint().unwrap(), resumed.checkpoint().unwrap());

        let invalid = CounterfactualCandidate {
            table: "l0.not_a_table".to_owned(),
            index: 0,
            field: CounterfactualField::Value,
            delta: 1,
        };
        assert!(patch_candidates(&trainer.artifact, &[invalid], trainer.config).is_err());
    }

    #[test]
    fn every_parameter_group_can_accept_a_coordinated_step() {
        let mut missing = Vec::new();
        let mut multi_member = false;
        for (group, delta) in [
            (CounterfactualGroup::Normalization, 16),
            (CounterfactualGroup::Bias, 1_000_000),
            (CounterfactualGroup::Decay, 1_000_000),
            (CounterfactualGroup::Route, 1),
        ] {
            if let Some((trainer, step)) = fixture_step(group, delta) {
                assert!(step.after.margin_loss < step.before.margin_loss);
                assert!(step.after.target_admitted);
                assert!(step
                    .accepted
                    .as_ref()
                    .is_some_and(|bundle| !bundle.members.is_empty()));
                if let Some(bundle) = &step.accepted {
                    assert!(bundle.members.contains(&bundle.best));
                    multi_member |= bundle.members.len() > 1;
                }
                assert!(IntegerStackModel::parse(trainer.artifact()).is_ok());
            } else {
                missing.push(group);
            }
        }
        assert!(
            missing.is_empty(),
            "no accepted coordinated {missing:?} step"
        );
        assert!(multi_member, "no accepted bundle exercised >1 coordinate");
    }
}
