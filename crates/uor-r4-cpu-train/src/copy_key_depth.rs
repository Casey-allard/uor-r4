//! Exact hierarchical copy-key depth screening over BOS-delimited token streams.

use serde::Serialize;

use crate::{invalid, CpuTrainError};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const COPY_KEY_MAX_DEPTH: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CopyKeyDepthArm {
    pub name: String,
    pub positions_with_match: u64,
    pub right: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CopyKeyComparison {
    pub candidate_only: u64,
    pub control_only: u64,
    pub candidate_sum: u64,
    pub control_sum: u64,
    pub threshold: u64,
    pub passes: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CopyKeyDepthReading {
    pub windows: u64,
    pub eligible_positions: u64,
    pub copyable_positions: u64,
    pub fixed: Vec<CopyKeyDepthArm>,
    pub longest: CopyKeyDepthArm,
    pub k2_backoff_k1: CopyKeyDepthArm,
    pub random_depth: CopyKeyDepthArm,
    pub random_changed_from_depth1: u64,
    pub random_changed_from_depth1_all: u64,
    pub random_changed_from_longest: u64,
    pub longest_vs_k2_backoff: CopyKeyComparison,
    pub longest_vs_random: CopyKeyComparison,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Outcome {
    copyable: bool,
    fixed: [Option<u32>; COPY_KEY_MAX_DEPTH],
    random: Option<u32>,
}

pub fn evaluate_copy_key_depth(
    tokens: &[u32],
    mask: &[u8],
    context: usize,
    bos: u32,
    seed: u64,
) -> Result<CopyKeyDepthReading> {
    if tokens.len() != mask.len() || tokens.is_empty() {
        return Err(invalid("copy-key tokens and mask differ in length"));
    }
    if context < COPY_KEY_MAX_DEPTH + 2 {
        return Err(invalid(
            "copy-key context is too short for the maximum depth",
        ));
    }
    if tokens[0] != bos || mask.iter().any(|&value| value > 1) {
        return Err(invalid("copy-key input lacks its initial BOS or mask bit"));
    }

    let starts = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, &token)| (token == bos).then_some(index))
        .collect::<Vec<_>>();
    let mut windows = 0u64;
    let mut eligible_positions = 0u64;
    let mut copyable_positions = 0u64;
    let mut outcomes = Vec::new();
    let mut targets = Vec::new();

    for (doc_id, &doc_start) in starts.iter().enumerate() {
        let doc_end = starts.get(doc_id + 1).copied().unwrap_or(tokens.len());
        if doc_end <= doc_start + 1 {
            continue;
        }
        let mut target_cursor = doc_start + 1;
        while target_cursor < doc_end {
            let target_end = (target_cursor + context).min(doc_end);
            let win_start = (target_end - 1).saturating_sub(context).max(doc_start);
            windows += 1;
            let scored = (target_cursor..target_end)
                .filter(|&position| position > 0 && tokens[position] != bos && mask[position] == 1)
                .collect::<Vec<_>>();
            let Some(&last) = scored.last() else {
                target_cursor = target_end;
                continue;
            };
            let x = &tokens[win_start..=last];
            for position in scored {
                eligible_positions += 1;
                let t = position - 1 - win_start;
                let next = x[t + 1];
                let copyable = x[..=t].contains(&next);
                copyable_positions += u64::from(copyable);

                let mut predictions = [None; COPY_KEY_MAX_DEPTH];
                for depth in 1..=COPY_KEY_MAX_DEPTH {
                    let query = (0..depth)
                        .map(|back| x[t.saturating_sub(back)])
                        .collect::<Vec<_>>();
                    predictions[depth - 1] = (1..t).rev().find_map(|candidate| {
                        let key = (0..depth)
                            .map(|back| x[candidate.saturating_sub(back + 1)])
                            .collect::<Vec<_>>();
                        (key == query).then_some(x[candidate])
                    });
                }
                let available = predictions
                    .iter()
                    .filter_map(|token| *token)
                    .collect::<Vec<_>>();
                let random = if available.is_empty() {
                    None
                } else {
                    let draw =
                        splitmix64(seed ^ (position as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
                            as usize
                            % available.len();
                    Some(available[draw])
                };
                targets.push(next);
                outcomes.push(Outcome {
                    copyable,
                    fixed: predictions,
                    random,
                });
            }
            target_cursor = target_end;
        }
    }

    if outcomes.is_empty() {
        return Err(invalid("copy-key input has no eligible target positions"));
    }

    let fixed = (1..=COPY_KEY_MAX_DEPTH)
        .map(|depth| {
            arm(
                format!("fixed_depth_{depth}"),
                &targets,
                outcomes.iter().map(|outcome| outcome.copyable),
                outcomes.iter().map(|outcome| outcome.fixed[depth - 1]),
            )
        })
        .collect::<Vec<_>>();
    let longest = arm(
        "longest_4_to_1",
        &targets,
        outcomes.iter().map(|outcome| outcome.copyable),
        outcomes
            .iter()
            .map(|outcome| outcome.fixed.iter().rev().flatten().copied().next()),
    );
    let k2_backoff_k1 = arm(
        "k2_backoff_k1",
        &targets,
        outcomes.iter().map(|outcome| outcome.copyable),
        outcomes
            .iter()
            .map(|outcome| outcome.fixed[1].or(outcome.fixed[0])),
    );
    let random_depth = arm(
        "random_available_depth",
        &targets,
        outcomes.iter().map(|outcome| outcome.copyable),
        outcomes.iter().map(|outcome| outcome.random),
    );
    let random_changed_from_depth1_all = outcomes
        .iter()
        .filter(|outcome| outcome.random != outcome.fixed[0])
        .count() as u64;
    let random_changed_from_depth1 = outcomes
        .iter()
        .filter(|outcome| outcome.copyable && outcome.random != outcome.fixed[0])
        .count() as u64;
    let random_changed_from_longest = outcomes
        .iter()
        .filter(|outcome| {
            outcome.copyable
                && outcome.random != outcome.fixed.iter().rev().flatten().copied().next()
        })
        .count() as u64;

    let longest_predictions = outcomes
        .iter()
        .map(|outcome| outcome.fixed.iter().rev().flatten().copied().next())
        .collect::<Vec<_>>();
    let backoff_predictions = outcomes
        .iter()
        .map(|outcome| outcome.fixed[1].or(outcome.fixed[0]))
        .collect::<Vec<_>>();
    let random_predictions = outcomes
        .iter()
        .map(|outcome| outcome.random)
        .collect::<Vec<_>>();

    Ok(CopyKeyDepthReading {
        windows,
        eligible_positions,
        copyable_positions,
        fixed,
        longest,
        k2_backoff_k1,
        random_depth,
        random_changed_from_depth1,
        random_changed_from_depth1_all,
        random_changed_from_longest,
        longest_vs_k2_backoff: compare(
            tokens,
            mask,
            bos,
            &longest_predictions,
            &backoff_predictions,
        )?,
        longest_vs_random: compare(tokens, mask, bos, &longest_predictions, &random_predictions)?,
    })
}

fn arm(
    name: impl Into<String>,
    targets: &[u32],
    copyable: impl IntoIterator<Item = bool>,
    predictions: impl IntoIterator<Item = Option<u32>>,
) -> CopyKeyDepthArm {
    let (mut positions_with_match, mut right) = (0u64, 0u64);
    for ((target, copyable), prediction) in targets.iter().zip(copyable).zip(predictions) {
        if !copyable {
            continue;
        }
        if let Some(prediction) = prediction {
            positions_with_match += 1;
            right += u64::from(prediction == *target);
        }
    }
    CopyKeyDepthArm {
        name: name.into(),
        positions_with_match,
        right,
    }
}

fn compare(
    tokens: &[u32],
    mask: &[u8],
    bos: u32,
    candidate: &[Option<u32>],
    control: &[Option<u32>],
) -> Result<CopyKeyComparison> {
    if candidate.len() != control.len() {
        return Err(invalid("copy-key comparison lengths differ"));
    }
    let targets = eligible_targets(tokens, mask, bos)?;
    if targets.len() != candidate.len() {
        return Err(invalid("copy-key comparison rows differ"));
    }
    let mut candidate_only = 0u64;
    let mut control_only = 0u64;
    let mut candidate_sum = 0u64;
    let mut control_sum = 0u64;
    for ((target, candidate), control) in targets.into_iter().zip(candidate).zip(control) {
        let candidate = candidate.is_some_and(|token| token == target);
        let control = control.is_some_and(|token| token == target);
        candidate_sum += u64::from(candidate);
        control_sum += u64::from(control);
        candidate_only += u64::from(candidate && !control);
        control_only += u64::from(control && !candidate);
    }
    let threshold = ceil_two_sqrt(control_only).max(2);
    Ok(CopyKeyComparison {
        candidate_only,
        control_only,
        candidate_sum,
        control_sum,
        threshold,
        passes: candidate_sum > control_sum && candidate_only >= threshold,
    })
}

fn eligible_targets(tokens: &[u32], mask: &[u8], bos: u32) -> Result<Vec<u32>> {
    if tokens.len() != mask.len() {
        return Err(invalid("copy-key target mask differs"));
    }
    Ok(tokens
        .iter()
        .zip(mask)
        .enumerate()
        .filter_map(|(index, (&token, &mask))| {
            (index > 0 && token != bos && mask == 1).then_some(token)
        })
        .collect())
}

fn ceil_two_sqrt(value: u64) -> u64 {
    let mut root = 0u64;
    while root.saturating_mul(root) < value.saturating_mul(4) {
        root += 1;
    }
    root
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_backoff_prefers_a_deeper_exact_copy() {
        let tokens = [
            0, 10, 20, 30, 40, 99, 50, 20, 30, 40, 60, 10, 20, 30, 40, 99,
        ];
        let mut mask = [0u8; 16];
        mask[15] = 1;
        let reading = evaluate_copy_key_depth(&tokens, &mask, 16, 0, 149).unwrap();
        assert_eq!(reading.eligible_positions, 1);
        assert_eq!(reading.copyable_positions, 1);
        assert_eq!(reading.fixed[3].positions_with_match, 1);
        assert_eq!(reading.fixed[3].right, 1);
        assert_eq!(reading.fixed[2].positions_with_match, 1);
        assert_eq!(reading.fixed[2].right, 0);
        assert_eq!(reading.fixed[1].positions_with_match, 1);
        assert_eq!(reading.fixed[1].right, 0);
        assert_eq!(reading.longest.right, 1);
        assert_eq!(reading.k2_backoff_k1.right, 0);
        assert_eq!(reading.longest_vs_k2_backoff.candidate_sum, 1);
        assert_eq!(reading.longest_vs_k2_backoff.control_sum, 0);
    }

    #[test]
    fn bos_boundaries_prevent_cross_document_copy_candidates() {
        let tokens = [0, 10, 20, 30, 40, 0, 20, 30, 50];
        let mut mask = [0u8; 9];
        mask[8] = 1;
        let reading = evaluate_copy_key_depth(&tokens, &mask, 8, 0, 149).unwrap();
        assert_eq!(reading.eligible_positions, 1);
        assert_eq!(reading.copyable_positions, 0);
        assert!(reading
            .fixed
            .iter()
            .all(|arm| arm.positions_with_match == 0));
    }

    #[test]
    fn the_current_query_position_is_not_a_candidate() {
        let tokens = [0, 1, 2, 3, 3, 3];
        let mut mask = [0u8; 6];
        mask[5] = 1;
        let reading = evaluate_copy_key_depth(&tokens, &mask, 6, 0, 149).unwrap();
        assert_eq!(reading.copyable_positions, 1);
        assert_eq!(reading.fixed[0].positions_with_match, 0);
    }

    #[test]
    fn random_novelty_is_gated_on_copyable_positions() {
        let tokens = [0, 1, 2, 3, 4, 5, 3, 6, 2, 3, 9];
        let mut mask = [0u8; 11];
        mask[10] = 1;
        let changed = (0..100)
            .map(|seed| evaluate_copy_key_depth(&tokens, &mask, 12, 0, seed).unwrap())
            .find(|reading| reading.random_changed_from_depth1_all > 0)
            .expect("some seed changes a random depth on a noncopyable position");
        assert_eq!(changed.copyable_positions, 0);
        assert_eq!(changed.random_changed_from_depth1, 0);
        assert_eq!(changed.random_changed_from_longest, 0);
    }

    #[test]
    fn random_depth_is_deterministic_for_one_seed() {
        let tokens = [0, 1, 2, 3, 1, 2, 4, 1, 2, 5];
        let mask = [0, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let first = evaluate_copy_key_depth(&tokens, &mask, 10, 0, 9).unwrap();
        let second = evaluate_copy_key_depth(&tokens, &mask, 10, 0, 9).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first.random_depth.positions_with_match,
            first.fixed[0].positions_with_match
        );
    }

    #[test]
    fn malformed_inputs_are_refused() {
        assert!(evaluate_copy_key_depth(&[1, 2], &[0, 1], 8, 0, 1).is_err());
        assert!(evaluate_copy_key_depth(&[0, 1], &[0, 2], 8, 0, 1).is_err());
        assert!(evaluate_copy_key_depth(&[0, 1], &[0, 1], 5, 0, 1).is_err());
    }
}
