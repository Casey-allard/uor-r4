//! Exact hierarchical copy-key depth screening over BOS-delimited token streams.

use serde::{Deserialize, Serialize};

use crate::{invalid, CpuTrainError};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const COPY_KEY_MAX_DEPTH: usize = 4;
pub const COPY_KEY_AGE_BINS: [u16; 9] = [1, 2, 4, 8, 16, 32, 64, 128, 192];

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyKeyRouteStat {
    pub depth: usize,
    pub age_upper: u16,
    pub matches: u64,
    pub right: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyKeyRouter {
    pub age_bins: Vec<u16>,
    pub stats: Vec<CopyKeyRouteStat>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CopyKeyRouterReading {
    pub windows: u64,
    pub eligible_positions: u64,
    pub copyable_positions: u64,
    pub router: CopyKeyDepthArm,
    pub longest: CopyKeyDepthArm,
    pub k2_backoff_k1: CopyKeyDepthArm,
    pub random_depth: CopyKeyDepthArm,
    pub random_changed_from_longest: u64,
    pub router_vs_longest: CopyKeyComparison,
    pub router_vs_k2_backoff: CopyKeyComparison,
    pub router_vs_random: CopyKeyComparison,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CopyCandidate {
    token: u32,
    age: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Outcome {
    copyable: bool,
    fixed: [Option<CopyCandidate>; COPY_KEY_MAX_DEPTH],
    random: Option<CopyCandidate>,
}

struct CopyKeyRows {
    windows: u64,
    eligible_positions: u64,
    copyable_positions: u64,
    targets: Vec<u32>,
    outcomes: Vec<Outcome>,
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
                        (key == query).then_some(CopyCandidate {
                            token: x[candidate],
                            age: t - candidate,
                        })
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
                outcomes
                    .iter()
                    .map(|outcome| outcome.fixed[depth - 1].map(|candidate| candidate.token)),
            )
        })
        .collect::<Vec<_>>();
    let longest = arm(
        "longest_4_to_1",
        &targets,
        outcomes.iter().map(|outcome| outcome.copyable),
        outcomes.iter().map(|outcome| {
            outcome
                .fixed
                .iter()
                .rev()
                .flatten()
                .next()
                .map(|candidate| candidate.token)
        }),
    );
    let k2_backoff_k1 = arm(
        "k2_backoff_k1",
        &targets,
        outcomes.iter().map(|outcome| outcome.copyable),
        outcomes.iter().map(|outcome| {
            outcome
                .fixed
                .get(1)
                .copied()
                .flatten()
                .or(outcome.fixed[0])
                .map(|candidate| candidate.token)
        }),
    );
    let random_depth = arm(
        "random_available_depth",
        &targets,
        outcomes.iter().map(|outcome| outcome.copyable),
        outcomes
            .iter()
            .map(|outcome| outcome.random.map(|candidate| candidate.token)),
    );
    let random_changed_from_depth1_all = outcomes
        .iter()
        .filter(|outcome| {
            outcome.random.map(|candidate| candidate.token)
                != outcome.fixed[0].map(|candidate| candidate.token)
        })
        .count() as u64;
    let random_changed_from_depth1 = outcomes
        .iter()
        .filter(|outcome| {
            outcome.copyable
                && outcome.random.map(|candidate| candidate.token)
                    != outcome.fixed[0].map(|candidate| candidate.token)
        })
        .count() as u64;
    let random_changed_from_longest = outcomes
        .iter()
        .filter(|outcome| {
            outcome.copyable
                && outcome.random.map(|candidate| candidate.token)
                    != outcome
                        .fixed
                        .iter()
                        .rev()
                        .flatten()
                        .next()
                        .map(|candidate| candidate.token)
        })
        .count() as u64;

    let longest_predictions = outcomes
        .iter()
        .map(|outcome| {
            outcome
                .fixed
                .iter()
                .rev()
                .flatten()
                .next()
                .map(|candidate| candidate.token)
        })
        .collect::<Vec<_>>();
    let backoff_predictions = outcomes
        .iter()
        .map(|outcome| {
            outcome
                .fixed
                .get(1)
                .copied()
                .flatten()
                .or(outcome.fixed[0])
                .map(|candidate| candidate.token)
        })
        .collect::<Vec<_>>();
    let random_predictions = outcomes
        .iter()
        .map(|outcome| outcome.random.map(|candidate| candidate.token))
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

pub fn fit_copy_key_router(
    tokens: &[u32],
    mask: &[u8],
    context: usize,
    bos: u32,
    seed: u64,
) -> Result<CopyKeyRouter> {
    let rows = scan_copy_key_rows(tokens, mask, context, bos, seed)?;
    let mut stats = Vec::with_capacity(COPY_KEY_MAX_DEPTH * COPY_KEY_AGE_BINS.len());
    for &age_upper in &COPY_KEY_AGE_BINS {
        for depth in 1..=COPY_KEY_MAX_DEPTH {
            stats.push(CopyKeyRouteStat {
                depth,
                age_upper,
                matches: 0,
                right: 0,
            });
        }
    }
    for (target, outcome) in rows.targets.iter().zip(&rows.outcomes) {
        if !outcome.copyable {
            continue;
        }
        for (depth, candidate) in outcome.fixed.iter().enumerate() {
            let Some(candidate) = candidate else {
                continue;
            };
            let bucket = age_bucket(candidate.age);
            let stat = &mut stats[bucket * COPY_KEY_MAX_DEPTH + depth];
            stat.matches += 1;
            stat.right += u64::from(candidate.token == *target);
        }
    }
    let router = CopyKeyRouter {
        age_bins: COPY_KEY_AGE_BINS.to_vec(),
        stats,
    };
    validate_copy_key_router(&router)?;
    Ok(router)
}

pub fn evaluate_copy_key_router(
    tokens: &[u32],
    mask: &[u8],
    context: usize,
    bos: u32,
    router: &CopyKeyRouter,
    seed: u64,
) -> Result<CopyKeyRouterReading> {
    validate_copy_key_router(router)?;
    let rows = scan_copy_key_rows(tokens, mask, context, bos, seed)?;
    let router_predictions = rows
        .outcomes
        .iter()
        .map(|outcome| route_copy_key(router, outcome))
        .collect::<Vec<_>>();
    let longest_predictions = rows
        .outcomes
        .iter()
        .map(|outcome| {
            outcome
                .fixed
                .iter()
                .rev()
                .flatten()
                .next()
                .map(|candidate| candidate.token)
        })
        .collect::<Vec<_>>();
    let backoff_predictions = rows
        .outcomes
        .iter()
        .map(|outcome| {
            outcome
                .fixed
                .get(1)
                .copied()
                .flatten()
                .or(outcome.fixed[0])
                .map(|candidate| candidate.token)
        })
        .collect::<Vec<_>>();
    let random_predictions = rows
        .outcomes
        .iter()
        .map(|outcome| outcome.random.map(|candidate| candidate.token))
        .collect::<Vec<_>>();
    let random_changed_from_longest = rows
        .outcomes
        .iter()
        .filter(|outcome| {
            outcome.copyable
                && outcome.random.map(|candidate| candidate.token)
                    != outcome
                        .fixed
                        .iter()
                        .rev()
                        .flatten()
                        .next()
                        .map(|candidate| candidate.token)
        })
        .count() as u64;
    Ok(CopyKeyRouterReading {
        windows: rows.windows,
        eligible_positions: rows.eligible_positions,
        copyable_positions: rows.copyable_positions,
        router: arm(
            "trained_depth_age_router",
            &rows.targets,
            rows.outcomes.iter().map(|outcome| outcome.copyable),
            router_predictions.iter().copied(),
        ),
        longest: arm(
            "longest_4_to_1",
            &rows.targets,
            rows.outcomes.iter().map(|outcome| outcome.copyable),
            longest_predictions.iter().copied(),
        ),
        k2_backoff_k1: arm(
            "k2_backoff_k1",
            &rows.targets,
            rows.outcomes.iter().map(|outcome| outcome.copyable),
            backoff_predictions.iter().copied(),
        ),
        random_depth: arm(
            "random_available_depth",
            &rows.targets,
            rows.outcomes.iter().map(|outcome| outcome.copyable),
            random_predictions.iter().copied(),
        ),
        random_changed_from_longest,
        router_vs_longest: compare(tokens, mask, bos, &router_predictions, &longest_predictions)?,
        router_vs_k2_backoff: compare(
            tokens,
            mask,
            bos,
            &router_predictions,
            &backoff_predictions,
        )?,
        router_vs_random: compare(tokens, mask, bos, &router_predictions, &random_predictions)?,
    })
}

fn route_copy_key(router: &CopyKeyRouter, outcome: &Outcome) -> Option<u32> {
    let mut best: Option<(usize, &CopyKeyRouteStat)> = None;
    for (depth, candidate) in outcome.fixed.iter().enumerate() {
        let Some(candidate) = candidate else {
            continue;
        };
        let bucket = age_bucket(candidate.age);
        let stat = &router.stats[bucket * COPY_KEY_MAX_DEPTH + depth];
        if stat.matches == 0 {
            continue;
        }
        let better = best.is_none_or(|(current_index, current)| {
            let stat_cross = u128::from(stat.right) * u128::from(current.matches);
            let current_cross = u128::from(current.right) * u128::from(stat.matches);
            stat_cross > current_cross || (stat_cross == current_cross && depth > current_index)
        });
        if better {
            best = Some((depth, stat));
        }
    }
    best.map(|(depth, _)| outcome.fixed[depth].unwrap().token)
        .or_else(|| {
            outcome
                .fixed
                .iter()
                .rev()
                .flatten()
                .next()
                .map(|candidate| candidate.token)
        })
}

fn age_bucket(age: usize) -> usize {
    COPY_KEY_AGE_BINS
        .iter()
        .position(|&upper| age <= usize::from(upper))
        .unwrap_or(COPY_KEY_AGE_BINS.len() - 1)
}

fn validate_copy_key_router(router: &CopyKeyRouter) -> Result<()> {
    if router.age_bins.as_slice() != COPY_KEY_AGE_BINS.as_slice()
        || router.stats.len() != router.age_bins.len() * COPY_KEY_MAX_DEPTH
    {
        return Err(invalid("copy-key router shape differs"));
    }
    for (bucket, &age_upper) in router.age_bins.iter().enumerate() {
        for depth in 1..=COPY_KEY_MAX_DEPTH {
            let stat = &router.stats[bucket * COPY_KEY_MAX_DEPTH + depth - 1];
            if stat.depth != depth || stat.age_upper != age_upper || stat.right > stat.matches {
                return Err(invalid("copy-key router statistic differs"));
            }
        }
    }
    Ok(())
}

fn scan_copy_key_rows(
    tokens: &[u32],
    mask: &[u8],
    context: usize,
    bos: u32,
    seed: u64,
) -> Result<CopyKeyRows> {
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
                let mut candidates = [None; COPY_KEY_MAX_DEPTH];
                for depth in 1..=COPY_KEY_MAX_DEPTH {
                    let query = (0..depth)
                        .map(|back| x[t.saturating_sub(back)])
                        .collect::<Vec<_>>();
                    candidates[depth - 1] = (1..t).rev().find_map(|candidate| {
                        let key = (0..depth)
                            .map(|back| x[candidate.saturating_sub(back + 1)])
                            .collect::<Vec<_>>();
                        (key == query).then_some(CopyCandidate {
                            token: x[candidate],
                            age: t - candidate,
                        })
                    });
                }
                let available = candidates
                    .iter()
                    .filter_map(|item| *item)
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
                    fixed: candidates,
                    random,
                });
            }
            target_cursor = target_end;
        }
    }
    if outcomes.is_empty() {
        return Err(invalid("copy-key input has no eligible target positions"));
    }
    Ok(CopyKeyRows {
        windows,
        eligible_positions,
        copyable_positions,
        targets,
        outcomes,
    })
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
    let target = u128::from(value) * 4;
    let (mut low, mut high) = (0u128, (1u128 << 33) + 1);
    while low + 1 < high {
        let middle = (low + high) / 2;
        if middle * middle >= target {
            high = middle;
        } else {
            low = middle;
        }
    }
    high as u64
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
    fn random_novelty_counts_the_emitted_token_not_candidate_age() {
        let tokens = [0, 10, 20, 30, 40, 50, 40, 50, 10, 20, 30, 40, 50];
        let mut mask = [0u8; 13];
        mask[12] = 1;
        for seed in 0..100 {
            let reading = evaluate_copy_key_depth(&tokens, &mask, 13, 0, seed).unwrap();
            assert_eq!(reading.copyable_positions, 1);
            assert_eq!(reading.fixed[0].right, 1);
            assert_eq!(reading.fixed[3].right, 1);
            assert_eq!(reading.random_depth.right, 1);
            assert_eq!(reading.random_changed_from_longest, 0, "seed {seed}");
        }
    }

    #[test]
    fn ceil_two_sqrt_handles_the_u64_limit() {
        assert_eq!(ceil_two_sqrt((1u64 << 62) + 1), (1u64 << 32) + 1);
        assert_eq!(ceil_two_sqrt(u64::MAX), 1u64 << 33);
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
    fn router_learns_a_deeper_candidate_and_round_trips() {
        let tokens = [
            0, 10, 20, 30, 40, 99, 50, 20, 30, 40, 60, 10, 20, 30, 40, 99,
        ];
        let mut mask = [0u8; 16];
        mask[15] = 1;
        let router = fit_copy_key_router(&tokens, &mask, 16, 0, 151).unwrap();
        let encoded = serde_json::to_vec(&router).unwrap();
        let decoded = serde_json::from_slice::<CopyKeyRouter>(&encoded).unwrap();
        assert_eq!(decoded, router);
        let reading = evaluate_copy_key_router(&tokens, &mask, 16, 0, &router, 151).unwrap();
        assert_eq!(reading.router.right, 1);
        assert_eq!(reading.longest.right, 1);
        assert_eq!(reading.k2_backoff_k1.right, 0);
        assert_eq!(reading.router_vs_k2_backoff.candidate_sum, 1);
        assert_eq!(reading.router_vs_k2_backoff.control_sum, 0);
    }

    #[test]
    fn router_uses_learned_precision_instead_of_depth_preference() {
        let mut router = CopyKeyRouter {
            age_bins: COPY_KEY_AGE_BINS.to_vec(),
            stats: (0..COPY_KEY_AGE_BINS.len())
                .flat_map(|bucket| {
                    (1..=COPY_KEY_MAX_DEPTH).map(move |depth| CopyKeyRouteStat {
                        depth,
                        age_upper: COPY_KEY_AGE_BINS[bucket],
                        matches: 0,
                        right: 0,
                    })
                })
                .collect(),
        };
        router.stats[0] = CopyKeyRouteStat {
            depth: 1,
            age_upper: 1,
            matches: 4,
            right: 4,
        };
        router.stats[1] = CopyKeyRouteStat {
            depth: 2,
            age_upper: 1,
            matches: 4,
            right: 0,
        };
        let outcome = Outcome {
            copyable: true,
            fixed: [
                Some(CopyCandidate { token: 7, age: 1 }),
                Some(CopyCandidate { token: 9, age: 1 }),
                None,
                None,
            ],
            random: None,
        };
        assert_eq!(route_copy_key(&router, &outcome), Some(7));
    }

    #[test]
    fn router_equal_precision_prefers_the_deeper_key() {
        let mut router = CopyKeyRouter {
            age_bins: COPY_KEY_AGE_BINS.to_vec(),
            stats: (0..COPY_KEY_AGE_BINS.len())
                .flat_map(|bucket| {
                    (1..=COPY_KEY_MAX_DEPTH).map(move |depth| CopyKeyRouteStat {
                        depth,
                        age_upper: COPY_KEY_AGE_BINS[bucket],
                        matches: 0,
                        right: 0,
                    })
                })
                .collect(),
        };
        router.stats[0] = CopyKeyRouteStat {
            depth: 1,
            age_upper: 1,
            matches: 2,
            right: 1,
        };
        router.stats[1] = CopyKeyRouteStat {
            depth: 2,
            age_upper: 1,
            matches: 4,
            right: 2,
        };
        let outcome = Outcome {
            copyable: true,
            fixed: [
                Some(CopyCandidate { token: 7, age: 1 }),
                Some(CopyCandidate { token: 9, age: 1 }),
                None,
                None,
            ],
            random: None,
        };
        assert_eq!(route_copy_key(&router, &outcome), Some(9));
    }

    #[test]
    fn router_with_no_learned_cell_falls_back_to_longest() {
        let router = CopyKeyRouter {
            age_bins: COPY_KEY_AGE_BINS.to_vec(),
            stats: (0..COPY_KEY_AGE_BINS.len())
                .flat_map(|bucket| {
                    (1..=COPY_KEY_MAX_DEPTH).map(move |depth| CopyKeyRouteStat {
                        depth,
                        age_upper: COPY_KEY_AGE_BINS[bucket],
                        matches: 0,
                        right: 0,
                    })
                })
                .collect(),
        };
        let outcome = Outcome {
            copyable: true,
            fixed: [
                Some(CopyCandidate { token: 7, age: 1 }),
                Some(CopyCandidate { token: 9, age: 2 }),
                None,
                None,
            ],
            random: None,
        };
        assert_eq!(route_copy_key(&router, &outcome), Some(9));
    }

    #[test]
    fn router_precision_comparison_cannot_overflow() {
        let mut router = CopyKeyRouter {
            age_bins: COPY_KEY_AGE_BINS.to_vec(),
            stats: (0..COPY_KEY_AGE_BINS.len())
                .flat_map(|bucket| {
                    (1..=COPY_KEY_MAX_DEPTH).map(move |depth| CopyKeyRouteStat {
                        depth,
                        age_upper: COPY_KEY_AGE_BINS[bucket],
                        matches: 0,
                        right: 0,
                    })
                })
                .collect(),
        };
        router.stats[0] = CopyKeyRouteStat {
            depth: 1,
            age_upper: 1,
            matches: u64::MAX,
            right: u64::MAX - 1,
        };
        router.stats[1] = CopyKeyRouteStat {
            depth: 2,
            age_upper: 1,
            matches: u64::MAX,
            right: u64::MAX,
        };
        let outcome = Outcome {
            copyable: true,
            fixed: [
                Some(CopyCandidate { token: 7, age: 1 }),
                Some(CopyCandidate { token: 9, age: 1 }),
                None,
                None,
            ],
            random: None,
        };
        assert_eq!(route_copy_key(&router, &outcome), Some(9));
    }

    #[test]
    fn malformed_router_tables_are_refused() {
        let mut router = CopyKeyRouter {
            age_bins: COPY_KEY_AGE_BINS.to_vec(),
            stats: (0..COPY_KEY_AGE_BINS.len())
                .flat_map(|bucket| {
                    (1..=COPY_KEY_MAX_DEPTH).map(move |depth| CopyKeyRouteStat {
                        depth,
                        age_upper: COPY_KEY_AGE_BINS[bucket],
                        matches: 0,
                        right: 0,
                    })
                })
                .collect(),
        };
        validate_copy_key_router(&router).unwrap();
        router.stats[0].right = 1;
        assert!(validate_copy_key_router(&router).is_err());

        let mut custom_bins = CopyKeyRouter {
            age_bins: vec![192],
            stats: (1..=COPY_KEY_MAX_DEPTH)
                .map(|depth| CopyKeyRouteStat {
                    depth,
                    age_upper: 192,
                    matches: 0,
                    right: 0,
                })
                .collect(),
        };
        assert!(validate_copy_key_router(&custom_bins).is_err());
        custom_bins.age_bins = COPY_KEY_AGE_BINS.to_vec();
        custom_bins.stats = (0..COPY_KEY_AGE_BINS.len())
            .flat_map(|bucket| {
                (1..=COPY_KEY_MAX_DEPTH).map(move |depth| CopyKeyRouteStat {
                    depth,
                    age_upper: COPY_KEY_AGE_BINS[bucket],
                    matches: 0,
                    right: 0,
                })
            })
            .collect();
        validate_copy_key_router(&custom_bins).unwrap();
    }

    #[test]
    fn malformed_inputs_are_refused() {
        assert!(evaluate_copy_key_depth(&[1, 2], &[0, 1], 8, 0, 1).is_err());
        assert!(evaluate_copy_key_depth(&[0, 1], &[0, 2], 8, 0, 1).is_err());
        assert!(evaluate_copy_key_depth(&[0, 1], &[0, 1], 5, 0, 1).is_err());
    }
}
