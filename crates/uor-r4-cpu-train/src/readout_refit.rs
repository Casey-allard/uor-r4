//! Exact refitting of a nearest-token output table with a fixed input path.
//!
//! The exported `UORLUT01` artifact carries one shared token table. This
//! development module scores unsaved output-table candidates against queries
//! captured from the unchanged artifact forward. It does not export a hybrid
//! artifact or change serving.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::{invalid, CpuTrainError};

type Result<T> = std::result::Result<T, CpuTrainError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadoutRefitMethod {
    Median,
    MeanCentroid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReadoutObservation {
    pub query_q12: Vec<i32>,
    pub target: u32,
    pub kept_cells: Vec<u32>,
    pub original_token: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefittedOutputTable {
    pub table: Vec<i16>,
    pub fitted_tokens: Vec<u32>,
    pub observations: u64,
    pub changed_coordinates: u64,
    pub displacement_l1: u64,
    pub maximum_row_displacement_l1: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShuffledAssignment {
    pub observations: Vec<ReadoutObservation>,
    pub fitted_tokens: Vec<u32>,
    pub changed_targets: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReadoutDecision {
    pub chosen_token: u32,
    pub target_rank: Option<usize>,
    pub candidate_count: usize,
    pub target_distance: i64,
    pub chosen_distance: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReadoutEvaluation {
    pub samples: u64,
    pub admitted: u64,
    pub top1_correct: u64,
    pub top5_correct: u64,
    pub top8_correct: u64,
    pub target_distance_sum: u64,
    pub target_rank_sum: u64,
    pub ranked_targets: u64,
    pub changed_decisions: u64,
    pub decision_sha256: String,
    #[serde(skip)]
    pub decision_tokens: Vec<u32>,
    #[serde(skip)]
    pub top1_rows: Vec<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationCensus {
    pub observations: u64,
    pub distinct_targets: u64,
    pub minimum_observations: usize,
    pub eligible_tokens: u64,
    pub eligible_observations: u64,
}

pub fn observe_refit_thresholds(
    observations: &[ReadoutObservation],
    thresholds: &[usize],
) -> Result<BTreeMap<usize, ObservationCensus>> {
    if observations.is_empty() {
        return Err(invalid("readout refit has no observations"));
    }
    let mut counts = vec![0u64; observations.len().max(1)];
    let maximum_target = observations
        .iter()
        .map(|observation| observation.target as usize)
        .max()
        .ok_or_else(|| invalid("readout refit target census is empty"))?;
    if counts.len() <= maximum_target {
        counts.resize(maximum_target + 1, 0);
    }
    for observation in observations {
        counts[observation.target as usize] = counts[observation.target as usize]
            .checked_add(1)
            .ok_or_else(|| invalid("readout refit observations overflow"))?;
    }
    let distinct_targets = counts.iter().filter(|&&count| count != 0).count() as u64;
    thresholds
        .iter()
        .map(|&minimum| {
            if minimum == 0 {
                return Err(invalid("readout refit minimum observations is zero"));
            }
            let eligible = counts
                .iter()
                .copied()
                .filter(|&count| count >= minimum as u64)
                .collect::<Vec<_>>();
            Ok((
                minimum,
                ObservationCensus {
                    observations: observations.len() as u64,
                    distinct_targets,
                    minimum_observations: minimum,
                    eligible_tokens: eligible.len() as u64,
                    eligible_observations: eligible.iter().sum(),
                },
            ))
        })
        .collect()
}

pub fn refit_output_table(
    original: &[i16],
    vocab: usize,
    width: usize,
    observations: &[ReadoutObservation],
    method: ReadoutRefitMethod,
    minimum_observations: usize,
) -> Result<RefittedOutputTable> {
    validate_shape(original, vocab, width)?;
    if minimum_observations == 0 {
        return Err(invalid("readout refit minimum observations is zero"));
    }
    let mut groups = vec![Vec::<usize>::new(); vocab];
    for (index, observation) in observations.iter().enumerate() {
        validate_observation(observation, vocab, width)?;
        groups[observation.target as usize].push(index);
    }
    let fitted_tokens = groups
        .iter()
        .enumerate()
        .filter_map(|(token, rows)| (rows.len() >= minimum_observations).then_some(token as u32))
        .collect::<Vec<_>>();
    if fitted_tokens.is_empty() {
        return Err(invalid("readout refit has no rows above the power floor"));
    }

    let mut table = original.to_vec();
    let fitted_observations = fitted_tokens
        .iter()
        .map(|&token| groups[token as usize].len() as u64)
        .sum();
    let mut changed_coordinates = 0u64;
    let mut displacement_l1 = 0u64;
    let mut maximum_row_displacement_l1 = 0u32;
    let mut values = Vec::new();
    for &token in &fitted_tokens {
        let rows = &groups[token as usize];
        let row_start = token as usize * width;
        let mut row_displacement = 0u32;
        for coordinate in 0..width {
            values.clear();
            values.extend(
                rows.iter()
                    .map(|&index| observations[index].query_q12[coordinate]),
            );
            values.sort_unstable();
            let value = match method {
                ReadoutRefitMethod::Median => median(&values),
                ReadoutRefitMethod::MeanCentroid => mean_centroid(&values)?,
            };
            let value =
                i16::try_from(value).map_err(|_| invalid("readout refit value is outside i16"))?;
            let previous = table[row_start + coordinate];
            changed_coordinates += u64::from(previous != value);
            let displacement = u32::from(previous.abs_diff(value));
            row_displacement = row_displacement
                .checked_add(displacement)
                .ok_or_else(|| invalid("readout row displacement overflows"))?;
            displacement_l1 = displacement_l1
                .checked_add(u64::from(displacement))
                .ok_or_else(|| invalid("readout displacement overflows"))?;
            table[row_start + coordinate] = value;
        }
        maximum_row_displacement_l1 = maximum_row_displacement_l1.max(row_displacement);
    }
    Ok(RefittedOutputTable {
        table,
        fitted_tokens,
        observations: fitted_observations,
        changed_coordinates,
        displacement_l1,
        maximum_row_displacement_l1,
    })
}

/// Permute target labels among observations belonging to rows that meet the
/// refit floor. Per-target counts and the global query multiset stay fixed.
pub fn shuffle_refit_targets(
    observations: &[ReadoutObservation],
    vocab: usize,
    width: usize,
    minimum_observations: usize,
) -> Result<ShuffledAssignment> {
    if minimum_observations == 0 {
        return Err(invalid("readout shuffle minimum observations is zero"));
    }
    let mut counts = vec![0usize; vocab];
    for observation in observations {
        validate_observation(observation, vocab, width)?;
        counts[observation.target as usize] += 1;
    }
    let eligible = counts
        .iter()
        .enumerate()
        .filter_map(|(token, &count)| (count >= minimum_observations).then_some(token as u32))
        .collect::<Vec<_>>();
    let eligible_set = eligible
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let positions = observations
        .iter()
        .enumerate()
        .filter_map(|(index, observation)| {
            eligible_set.contains(&observation.target).then_some(index)
        })
        .collect::<Vec<_>>();
    if positions.len() < 2 || eligible.len() < 2 {
        return Err(invalid(
            "readout shuffle has fewer than two eligible targets",
        ));
    }
    let labels = positions
        .iter()
        .map(|&index| observations[index].target)
        .collect::<Vec<_>>();
    let (offset, changed) = (1..positions.len())
        .map(|offset| {
            let changed = positions
                .iter()
                .enumerate()
                .filter(|(index, _)| labels[*index] != labels[(index + offset) % labels.len()])
                .count() as u64;
            (offset, changed)
        })
        .max_by_key(|&(offset, changed)| (changed, std::cmp::Reverse(offset)))
        .ok_or_else(|| invalid("readout shuffle found no permutation"))?;
    if changed == 0 {
        return Err(invalid("readout target shuffle is vacuous"));
    }
    let mut shuffled = observations.to_vec();
    for (slot, &index) in positions.iter().enumerate() {
        shuffled[index].target = labels[(slot + offset) % labels.len()];
    }
    Ok(ShuffledAssignment {
        observations: shuffled,
        fitted_tokens: eligible,
        changed_targets: changed,
    })
}

pub fn score_readout(
    query: &[i32],
    target: u32,
    kept_cells: &[u32],
    table: &[i16],
    vocab: usize,
    width: usize,
) -> Result<ReadoutDecision> {
    if query.len() != width || target as usize >= vocab {
        return Err(invalid("readout score query or target differs"));
    }
    validate_shape(table, vocab, width)?;
    if kept_cells.is_empty() || kept_cells.iter().any(|&cell| cell >= 256) {
        return Err(invalid("readout score admitted cells differ"));
    }
    let mut candidates = Vec::new();
    for &cell in kept_cells {
        let mut token = cell as usize;
        while token < vocab {
            let row = &table[token * width..(token + 1) * width];
            let distance = query.iter().zip(row).fold(0i64, |sum, (&left, &right)| {
                sum.wrapping_add(i64::from(left.abs_diff(i32::from(right))))
            });
            candidates.push((token as u32, distance));
            token += 256;
        }
    }
    if candidates.is_empty() {
        return Err(invalid("readout score found no candidates"));
    }
    candidates.sort_unstable_by(|left, right| left.1.cmp(&right.1).then(left.0.cmp(&right.0)));
    let target_row = &table[target as usize * width..(target as usize + 1) * width];
    let target_distance = query
        .iter()
        .zip(target_row)
        .fold(0i64, |sum, (&left, &right)| {
            sum.wrapping_add(i64::from(left.abs_diff(i32::from(right))))
        });
    Ok(ReadoutDecision {
        chosen_token: candidates[0].0,
        target_rank: candidates
            .iter()
            .position(|&(token, _)| token == target)
            .map(|rank| rank + 1),
        candidate_count: candidates.len(),
        target_distance,
        chosen_distance: candidates[0].1,
    })
}

pub fn evaluate_output_table(
    table: &[i16],
    vocab: usize,
    width: usize,
    observations: &[ReadoutObservation],
    original_tokens: &[u32],
) -> Result<ReadoutEvaluation> {
    if observations.len() != original_tokens.len() {
        return Err(invalid("readout evaluation decisions differ"));
    }
    let mut evaluation = ReadoutEvaluation {
        samples: observations.len() as u64,
        admitted: 0,
        top1_correct: 0,
        top5_correct: 0,
        top8_correct: 0,
        target_distance_sum: 0,
        target_rank_sum: 0,
        ranked_targets: 0,
        changed_decisions: 0,
        decision_sha256: String::new(),
        decision_tokens: Vec::with_capacity(observations.len()),
        top1_rows: Vec::with_capacity(observations.len()),
    };
    let mut decisions = Vec::with_capacity(observations.len());
    for (observation, &original_token) in observations.iter().zip(original_tokens) {
        let decision = score_readout(
            &observation.query_q12,
            observation.target,
            &observation.kept_cells,
            table,
            vocab,
            width,
        )?;
        if let Some(rank) = decision.target_rank {
            evaluation.admitted += 1;
            evaluation.top5_correct += u64::from(rank <= 5);
            evaluation.top8_correct += u64::from(rank <= 8);
            evaluation.ranked_targets += 1;
            evaluation.target_rank_sum = evaluation
                .target_rank_sum
                .checked_add(rank as u64)
                .ok_or_else(|| invalid("readout target rank sum overflows"))?;
        }
        let top1 = decision.chosen_token == observation.target;
        evaluation.top1_correct += u64::from(top1);
        evaluation.target_distance_sum = evaluation
            .target_distance_sum
            .checked_add(
                u64::try_from(decision.target_distance)
                    .map_err(|_| invalid("readout target distance is negative"))?,
            )
            .ok_or_else(|| invalid("readout target distance sum overflows"))?;
        evaluation.changed_decisions += u64::from(decision.chosen_token != original_token);
        evaluation.decision_tokens.push(decision.chosen_token);
        evaluation.top1_rows.push(top1);
        decisions.extend_from_slice(&decision.chosen_token.to_le_bytes());
    }
    evaluation.decision_sha256 = uor_r4_discrete::corpus::sha256(&decisions);
    Ok(evaluation)
}

pub fn paired_two_sigma(candidate: &[bool], control: &[bool]) -> Result<serde_json::Value> {
    if candidate.len() != control.len() {
        return Err(invalid("paired readout decisions differ"));
    }
    let candidate_only = candidate
        .iter()
        .zip(control)
        .filter(|(candidate, control)| **candidate && !**control)
        .count() as u64;
    let control_only = candidate
        .iter()
        .zip(control)
        .filter(|(candidate, control)| !**candidate && **control)
        .count() as u64;
    let candidate_sum = candidate.iter().filter(|&&value| value).count() as u64;
    let control_sum = control.iter().filter(|&&value| value).count() as u64;
    let discordant = candidate_only
        .checked_add(control_only)
        .ok_or_else(|| invalid("paired readout discordance overflows"))?;
    let threshold = ceil_sqrt(
        discordant
            .checked_mul(4)
            .ok_or_else(|| invalid("paired readout threshold overflows"))?,
    )
    .max(2);
    Ok(serde_json::json!({
        "candidate_only": candidate_only,
        "control_only": control_only,
        "candidate_sum": candidate_sum,
        "control_sum": control_sum,
        "discordant": discordant,
        "threshold": threshold,
        "passes": candidate_sum > control_sum && candidate_only >= threshold,
    }))
}

fn median(sorted: &[i32]) -> i32 {
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[middle]
    } else {
        let lower = sorted[middle - 1];
        lower.saturating_add((sorted[middle] - lower).div_euclid(2))
    }
}

fn mean_centroid(values: &[i32]) -> Result<i32> {
    let sum = values.iter().try_fold(0i64, |sum, &value| {
        sum.checked_add(i64::from(value))
            .ok_or_else(|| invalid("readout centroid sum overflows"))
    })?;
    let count =
        i64::try_from(values.len()).map_err(|_| invalid("readout centroid count exceeds i64"))?;
    if count == 0 {
        return Err(invalid("readout centroid is empty"));
    }
    let magnitude = sum.unsigned_abs();
    let denominator = count as u64;
    let quotient = magnitude / denominator;
    let remainder = magnitude % denominator;
    let rounded = quotient + u64::from(remainder * 2 >= denominator);
    let signed =
        i64::try_from(rounded).map_err(|_| invalid("readout rounded centroid exceeds i64"))?;
    i32::try_from(if sum < 0 { -signed } else { signed })
        .map_err(|_| invalid("readout rounded centroid exceeds i32"))
}

fn validate_shape(table: &[i16], vocab: usize, width: usize) -> Result<()> {
    if vocab == 0 || width == 0 || table.len() != vocab.saturating_mul(width) {
        return Err(invalid("readout table shape differs"));
    }
    Ok(())
}

fn validate_observation(
    observation: &ReadoutObservation,
    vocab: usize,
    width: usize,
) -> Result<()> {
    if observation.query_q12.len() != width || observation.target as usize >= vocab {
        return Err(invalid("readout observation query or target differs"));
    }
    Ok(())
}

fn ceil_sqrt(value: u64) -> u64 {
    if value <= 1 {
        return value;
    }
    let mut low = 1u64;
    let mut high = value.min(1u64 << 32);
    while low < high {
        let middle = low + (high - low) / 2;
        if middle
            .checked_mul(middle)
            .is_some_and(|square| square >= value)
        {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(query: &[i32], target: u32, kept: &[u32]) -> ReadoutObservation {
        ReadoutObservation {
            query_q12: query.to_vec(),
            target,
            kept_cells: kept.to_vec(),
            original_token: target,
        }
    }

    #[test]
    fn median_is_an_integer_minimizer_for_odd_and_even_counts() {
        assert_eq!(median(&[1, 2, 9]), 2);
        assert_eq!(median(&[-5, -2]), -4);
        assert_eq!(median(&[-3, 2]), -1);
        assert_eq!(median(&[4, 4]), 4);
    }

    #[test]
    fn mean_centroid_rounds_half_away_from_zero_without_float() {
        assert_eq!(mean_centroid(&[1, 2]).unwrap(), 2);
        assert_eq!(mean_centroid(&[-1, -2]).unwrap(), -2);
        assert_eq!(mean_centroid(&[-2, 3]).unwrap(), 1);
        assert_eq!(mean_centroid(&[4, 4, 5]).unwrap(), 4);
    }

    #[test]
    fn refit_changes_only_rows_at_the_observation_floor() {
        let original = vec![0i16; 4 * 2];
        let observations = vec![
            observation(&[10, 20], 1, &[1]),
            observation(&[12, 24], 1, &[1]),
            observation(&[30, 40], 2, &[2]),
        ];
        let refit = refit_output_table(
            &original,
            4,
            2,
            &observations,
            ReadoutRefitMethod::Median,
            2,
        )
        .unwrap();
        assert_eq!(refit.fitted_tokens, vec![1]);
        assert_eq!(&refit.table[2..4], &[11, 22]);
        assert_eq!(&refit.table[0..2], &[0, 0]);
        assert_eq!(&refit.table[4..6], &[0, 0]);
    }

    #[test]
    fn matched_shuffle_preserves_counts_and_row_set() {
        let observations = vec![
            observation(&[1, 0], 1, &[1]),
            observation(&[2, 0], 1, &[1]),
            observation(&[3, 0], 2, &[2]),
            observation(&[4, 0], 2, &[2]),
            observation(&[5, 0], 3, &[3]),
        ];
        let shuffled = shuffle_refit_targets(&observations, 4, 2, 2).unwrap();
        assert_eq!(shuffled.fitted_tokens, vec![1, 2]);
        assert!(shuffled.changed_targets > 0);
        let before = observations
            .iter()
            .filter(|observation| observation.target == 1 || observation.target == 2)
            .map(|observation| observation.target)
            .collect::<Vec<_>>();
        let after = shuffled
            .observations
            .iter()
            .filter(|observation| observation.target == 1 || observation.target == 2)
            .map(|observation| observation.target)
            .collect::<Vec<_>>();
        assert_ne!(before, after);
        assert_eq!(
            shuffled
                .observations
                .iter()
                .filter(|observation| observation.target == 1)
                .count(),
            2
        );
        assert_eq!(
            shuffled
                .observations
                .iter()
                .filter(|observation| observation.target == 2)
                .count(),
            2
        );
    }

    #[test]
    fn decoupled_identical_table_reproduces_every_decision() {
        let table = vec![0i16, 0, 10, 10, -10, -10, 40, 40];
        let observations = vec![
            observation(&[9, 9], 1, &[1, 2, 3]),
            observation(&[-8, -8], 2, &[1, 2, 3]),
            observation(&[38, 38], 3, &[1, 2, 3]),
        ];
        let original = observations
            .iter()
            .map(|observation| {
                score_readout(
                    &observation.query_q12,
                    observation.target,
                    &observation.kept_cells,
                    &table,
                    4,
                    2,
                )
                .unwrap()
                .chosen_token
            })
            .collect::<Vec<_>>();
        let evaluation = evaluate_output_table(&table, 4, 2, &observations, &original).unwrap();
        assert_eq!(evaluation.changed_decisions, 0);
        assert_eq!(evaluation.decision_sha256.len(), 64);
    }

    #[test]
    fn score_breaks_ties_by_lower_token_and_reports_rank() {
        let table = vec![0i16, 0, 2, 2, -2, -2, 0, 0];
        let decision = score_readout(&[0, 0], 2, &[1, 2, 3], &table, 4, 2).unwrap();
        assert_eq!(decision.chosen_token, 3);
        assert_eq!(decision.target_rank, Some(3));
        assert_eq!(decision.candidate_count, 3);
    }

    #[test]
    fn paired_threshold_uses_both_discordant_arms() {
        let candidate = [true, true, true, true, false, false, false];
        let control = [false, false, false, false, true, true, false];
        let result = paired_two_sigma(&candidate, &control).unwrap();
        assert_eq!(result["candidate_only"], 4);
        assert_eq!(result["control_only"], 2);
        assert_eq!(result["threshold"], 5);
        assert_eq!(result["passes"], false);
    }

    #[test]
    fn ceil_sqrt_is_exact() {
        assert_eq!(ceil_sqrt(0), 0);
        assert_eq!(ceil_sqrt(1), 1);
        assert_eq!(ceil_sqrt(2), 2);
        assert_eq!(ceil_sqrt(4), 2);
        assert_eq!(ceil_sqrt(5), 3);
        assert_eq!(ceil_sqrt(u64::MAX), 1u64 << 32);
    }
}
