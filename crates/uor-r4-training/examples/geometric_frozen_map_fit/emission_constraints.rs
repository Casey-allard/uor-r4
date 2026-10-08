//! One frozen, adjacent-code shared Generate construction with exact pooled guards.
use super::native_proposals as np;
use super::*;
use uor_r4_integer::geometric_vocabulary_actions::{GeneratePatchSummary, NativeVocabularyActions};

const UNARY: usize = 960;
const PAIR: usize = 57_600;
const COORDINATES: usize = UNARY + PAIR;
const FACTORS: usize = 12;
const PROTECTED: usize = 84;
const TOKEN_STRIDE: usize = 4096;

/// Target-free native capture plus the offline protected-winner predicate.
/// Factor keys are token-major, eight unary lanes then four declared edges.
pub(super) struct ProtectedPool {
    pub required_token: u32,
    pub generate_q24: Vec<i64>,
    pub copy_ids: Vec<u32>,
    pub copy_q24: Vec<i64>,
    pub factor_keys: Vec<u32>,
}
#[derive(Clone, Debug, serde::Serialize)]
pub(super) struct Trial {
    pub name: String,
    pub index: usize,
    pub before: i8,
    pub after: i8,
    pub gradient: f32,
    pub original_master: f32,
    pub predicted_delta: f64,
    pub status: String,
    pub affected_postings: usize,
    pub affected_pools: usize,
    pub checked_pools: usize,
    pub blocking_pool: Option<usize>,
    pub blocking_winner: Option<u32>,
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub(super) struct Stats {
    pub coordinates: usize,
    pub protected_pools: usize,
    pub incidence_postings: usize,
    pub incidence_bytes: usize,
    pub accepted: usize,
    pub rejected: usize,
    pub zero_gradient: usize,
    pub saturated: usize,
    pub no_protected_incidence: usize,
    pub checked_pools: usize,
    pub changed_reference_fallbacks: usize,
    pub maximum_scans: usize,
    pub winner_scans: usize,
    pub checked_postings: usize,
    pub accepted_postings: usize,
    pub protected_decisions_preserved: bool,
}
#[derive(serde::Serialize)]
pub(super) struct Construction {
    pub edits: Vec<np::Edit>,
    pub trials: Vec<Trial>,
    pub stats: Stats,
    pub g_dot_actual_delta: f64,
    pub final_generate_scores: Vec<Vec<i64>>,
    pub final_token_masses: Vec<Vec<u64>>,
    pub final_pool_summaries: Vec<GeneratePatchSummary>,
}
fn code(value: f32) -> Result<i8> {
    replay_require(
        value.is_finite() && (-1.75..=1.75).contains(&value),
        "emission construction master outside strict Q4 range",
    )?;
    Ok((4.0 * value).round() as i8)
}

/// No caller masters mutate. All row trials stage first; batch commit validates
/// every pending revision before any row advances. A dropped trial changes none.
pub(super) fn construct(
    parent: &np::Shadows,
    gradients: &np::Shadows,
    pools: Vec<ProtectedPool>,
    reducer: &mut NativeVocabularyActions,
) -> Result<Construction> {
    replay_require(
        pools.len() == PROTECTED,
        "emission requires all84 protected pools",
    )?;
    let mut ordered = Vec::with_capacity(COORDINATES);
    for (name, count, start) in [("generate.unary", UNARY, 0), ("generate.pair", PAIR, UNARY)] {
        let master = parent
            .get(name)
            .ok_or_else(|| bad("emission master absent"))?;
        let gradient = gradients
            .get(name)
            .ok_or_else(|| bad("emission gradient absent"))?;
        replay_require(
            master.len() == count && gradient.len() == count,
            "emission eligible tensor shape differs",
        )?;
        for (index, (&original_master, &g)) in master.iter().zip(gradient).enumerate() {
            replay_require(g.is_finite(), "nonfinite emission gradient")?;
            let before = code(original_master)?;
            let direction = if g > 0.0 {
                -1
            } else if g < 0.0 {
                1
            } else {
                0
            };
            let after = (before + direction).clamp(-7, 7);
            let predicted_delta = if before == after {
                0.0
            } else {
                f64::from(g) * (f64::from(after) * 0.25 - f64::from(original_master))
            };
            replay_require(
                before == after || predicted_delta < 0.0,
                "emission adjacent move fails actual linear descent",
            )?;
            ordered.push((
                start + index,
                Trial {
                    name: name.to_owned(),
                    index,
                    before,
                    after,
                    gradient: g,
                    original_master,
                    predicted_delta,
                    status: if direction == 0 {
                        "zero_gradient"
                    } else if before == after {
                        "saturated"
                    } else {
                        "pending"
                    }
                    .to_owned(),
                    affected_postings: 0,
                    affected_pools: 0,
                    checked_pools: 0,
                    blocking_pool: None,
                    blocking_winner: None,
                },
            ));
        }
    }
    ordered.sort_by(|(_, a), (_, b)| {
        a.predicted_delta
            .total_cmp(&b.predicted_delta)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.index.cmp(&b.index))
    });
    // Compact CSR. Fixed token stride lets every posting identify its row and
    // vocabulary token without storing a pair or assuming equal pool lengths.
    let mut offsets = vec![0u32; COORDINATES + 1];
    for pool in &pools {
        replay_require(
            !pool.generate_q24.is_empty()
                && pool.generate_q24.len() <= TOKEN_STRIDE
                && pool.factor_keys.len() == pool.generate_q24.len() * FACTORS,
            "protected emission incidence shape differs",
        )?;
        for keys in pool.factor_keys.chunks_exact(FACTORS) {
            for (factor, &key) in keys.iter().enumerate() {
                let key = key as usize;
                let (lo, hi) = if factor < 8 {
                    (factor * 120, (factor + 1) * 120)
                } else {
                    (UNARY + (factor - 8) * 14_400, UNARY + (factor - 7) * 14_400)
                };
                replay_require(
                    (lo..hi).contains(&key),
                    "invalid logical emission factor key/order",
                )?;
                offsets[key + 1] = offsets[key + 1]
                    .checked_add(1)
                    .ok_or_else(|| bad("emission incidence count overflow"))?;
            }
        }
    }
    for i in 1..offsets.len() {
        offsets[i] = offsets[i]
            .checked_add(offsets[i - 1])
            .ok_or_else(|| bad("emission incidence offset overflow"))?;
    }
    let count = offsets[COORDINATES] as usize;
    let mut postings = vec![0u32; count];
    let mut cursors = offsets[..COORDINATES].to_vec();
    for (row, pool) in pools.iter().enumerate() {
        for (token, keys) in pool.factor_keys.chunks_exact(FACTORS).enumerate() {
            for &key in keys {
                let index = key as usize;
                postings[cursors[index] as usize] = (row * TOKEN_STRIDE + token) as u32;
                cursors[index] += 1;
            }
        }
    }
    drop(cursors);
    let mut required = Vec::with_capacity(PROTECTED);
    let mut caches = Vec::with_capacity(PROTECTED);
    for pool in pools {
        let cache = reducer.prepare_generate_patch_cache(
            pool.generate_q24,
            pool.copy_ids,
            pool.copy_q24,
        )?;
        replay_require(
            cache.summary().chosen_token_id == pool.required_token,
            "protected emission parent does not choose required token",
        )?;
        required.push(pool.required_token);
        caches.push(cache);
    }
    let mut stats = Stats {
        coordinates: COORDINATES,
        protected_pools: PROTECTED,
        incidence_postings: count,
        incidence_bytes: 4 * (postings.len() + offsets.len()),
        ..Stats::default()
    };
    let mut edits = Vec::new();
    let mut trials = Vec::with_capacity(COORDINATES);
    let mut g_dot_actual_delta = 0.0;
    for (key, mut trial) in ordered {
        if trial.before == trial.after {
            if trial.status == "zero_gradient" {
                stats.zero_gradient += 1;
            } else {
                stats.saturated += 1;
            }
            trials.push(trial);
            continue;
        }
        let incident = &postings[offsets[key] as usize..offsets[key + 1] as usize];
        trial.affected_postings = incident.len();
        trial.affected_pools = usize::from(!incident.is_empty())
            + incident
                .windows(2)
                .filter(|pair| pair[0] as usize / TOKEN_STRIDE != pair[1] as usize / TOKEN_STRIDE)
                .count();
        let delta = i64::from(trial.after - trial.before) << 20;
        let mut pending = Vec::with_capacity(trial.affected_pools);
        let mut cursor = 0;
        let mut changes = Vec::new();
        while cursor < incident.len() {
            let row = incident[cursor] as usize / TOKEN_STRIDE;
            changes.clear();
            while cursor < incident.len() && incident[cursor] as usize / TOKEN_STRIDE == row {
                let token = incident[cursor] as usize % TOKEN_STRIDE;
                let score = caches[row].generate_scores()[token]
                    .checked_add(delta)
                    .ok_or_else(|| bad("emission native atom overflow"))?;
                changes.push((token as u32, score));
                cursor += 1;
            }
            let patch = reducer.evaluate_generate_patch(&caches[row], &changes)?;
            let summary = patch.summary();
            stats.checked_pools += 1;
            stats.checked_postings += changes.len();
            stats.changed_reference_fallbacks += usize::from(summary.used_full_reduction);
            stats.maximum_scans += usize::from(summary.scanned_maximum);
            stats.winner_scans += usize::from(summary.scanned_winner);
            trial.checked_pools += 1;
            if summary.chosen_token_id != required[row] {
                trial.blocking_pool = Some(row);
                trial.blocking_winner = Some(summary.chosen_token_id);
                break;
            }
            pending.push((row, patch));
        }
        if trial.blocking_pool.is_some() {
            stats.rejected += 1;
            trial.status = "rejected".to_owned();
        } else {
            reducer.commit_generate_patch_batch(&mut caches, pending)?;
            stats.accepted += 1;
            stats.accepted_postings += incident.len();
            stats.no_protected_incidence += usize::from(incident.is_empty());
            g_dot_actual_delta += trial.predicted_delta;
            edits.push(np::Edit {
                name: trial.name.clone(),
                index: trial.index,
                before: trial.before,
                after: trial.after,
            });
            trial.status = "accepted".to_owned();
        }
        trials.push(trial);
    }
    stats.protected_decisions_preserved = caches
        .iter()
        .zip(&required)
        .all(|(cache, token)| cache.summary().chosen_token_id == *token);
    replay_require(
        stats.protected_decisions_preserved,
        "emission protected winners changed",
    )?;
    Ok(Construction {
        edits,
        trials,
        stats,
        g_dot_actual_delta,
        final_generate_scores: caches
            .iter()
            .map(|c| c.generate_scores().to_vec())
            .collect(),
        final_token_masses: caches.iter().map(|c| c.token_masses().to_vec()).collect(),
        final_pool_summaries: caches.iter().map(|c| c.summary().clone()).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::geometric_source_actions::SourceActionBinding;
    const TOKENIZER: &[u8] = br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    fn reducer() -> Result<NativeVocabularyActions> {
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        Ok(NativeVocabularyActions::new(
            SourceActionBinding::new(TOKENIZER)?,
            &exp,
        )?)
    }
    fn fixture() -> (np::Shadows, np::Shadows, Vec<ProtectedPool>) {
        let parent = [
            ("generate.unary".to_owned(), vec![0.; UNARY]),
            ("generate.pair".to_owned(), vec![0.; PAIR]),
            ("generate.bias".to_owned(), vec![-0.0; 6]),
            ("generate.prototype_choices".to_owned(), vec![0.37; 12]),
        ]
        .into_iter()
        .collect::<np::Shadows>();
        let gradient = [
            ("generate.unary".to_owned(), vec![0.; UNARY]),
            ("generate.pair".to_owned(), vec![0.; PAIR]),
        ]
        .into_iter()
        .collect();
        let pools = (0..PROTECTED)
            .map(|_| {
                let mut keys = Vec::new();
                for token in 0..6 {
                    keys.extend((0..8).map(|lane| (lane * 120 + token) as u32));
                    keys.extend((0..4).map(|edge| (UNARY + edge * 14_400 + token * 121) as u32));
                }
                let mut generate_q24 = vec![0; 6];
                generate_q24[5] = 4 << 24;
                ProtectedPool {
                    required_token: 5,
                    generate_q24,
                    copy_ids: vec![3, 3],
                    copy_q24: vec![-(4 << 24); 2],
                    factor_keys: keys,
                }
            })
            .collect();
        (parent, gradient, pools)
    }
    #[test]
    fn emission_once_only_shared_patch_late_rejection_and_offgrid_bits() -> Result<()> {
        let (mut parent, mut g, mut pools) = fixture();
        // Rank the unsafe rival before the accepted winner increase. The last
        // row rejects it, so all earlier staged rows must remain unchanged.
        pools[PROTECTED - 1].generate_q24.fill(-(4 << 24));
        pools[PROTECTED - 1].generate_q24[5] = 0;
        pools[PROTECTED - 1].generate_q24[4] = -(1 << 19);
        g.get_mut("generate.unary")
            .ok_or_else(|| bad("fixture unary"))?[4] = -3.;
        g.get_mut("generate.unary")
            .ok_or_else(|| bad("fixture unary"))?[5] = -1.;
        parent
            .get_mut("generate.unary")
            .ok_or_else(|| bad("fixture unary"))?[100] = 0.07;
        g.get_mut("generate.unary")
            .ok_or_else(|| bad("fixture unary"))?[100] = -0.5;
        parent
            .get_mut("generate.unary")
            .ok_or_else(|| bad("fixture unary"))?[101] = 1.75;
        g.get_mut("generate.unary")
            .ok_or_else(|| bad("fixture unary"))?[101] = -1.;
        g.get_mut("generate.pair")
            .ok_or_else(|| bad("fixture pair"))?[3 * 121] = -0.25;
        g.get_mut("generate.pair")
            .ok_or_else(|| bad("fixture pair"))?[5 * 121] = -0.125;
        let before = parent.clone();
        let baseline = pools
            .iter()
            .map(|p| p.generate_q24.clone())
            .collect::<Vec<_>>();
        let result = construct(&parent, &g, pools, &mut reducer()?)?;
        assert!(np::same_bits(&parent, &before));
        assert_eq!(result.stats.rejected, 1);
        assert_eq!(result.stats.accepted, 4);
        assert_eq!(result.stats.no_protected_incidence, 1);
        assert_eq!(result.stats.saturated, 1);
        assert_eq!(result.stats.coordinates, result.trials.len());
        let blocked = result
            .trials
            .iter()
            .find(|t| t.name == "generate.unary" && t.index == 4)
            .ok_or_else(|| bad("fixture rejected trial absent"))?;
        assert_eq!(blocked.blocking_pool, Some(PROTECTED - 1));
        assert_eq!(blocked.checked_pools, PROTECTED);
        for (row, scores) in result.final_generate_scores.iter().enumerate() {
            for token in 0..6 {
                let expected_delta = match token {
                    5 => 2 << 20,
                    3 => 1 << 20,
                    _ => 0,
                };
                assert_eq!(scores[token], baseline[row][token] + expected_delta);
            }
        }
        let edited = np::edited(&parent, &result.edits)?;
        assert_eq!(edited["generate.unary"][100], 0.25);
        assert_eq!(edited["generate.unary"][4], parent["generate.unary"][4]);
        assert_eq!(
            edited["generate.bias"]
                .iter()
                .map(|x| x.to_bits())
                .collect::<Vec<_>>(),
            parent["generate.bias"]
                .iter()
                .map(|x| x.to_bits())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            edited["generate.prototype_choices"],
            parent["generate.prototype_choices"]
        );
        assert!(result.stats.protected_decisions_preserved);
        Ok(())
    }
    #[test]
    fn emission_frozen_rank_ties_and_invalid_incidence_fail() -> Result<()> {
        let (parent, mut g, pools) = fixture();
        g.get_mut("generate.unary")
            .ok_or_else(|| bad("fixture unary"))?[98] = -1.;
        g.get_mut("generate.unary")
            .ok_or_else(|| bad("fixture unary"))?[99] = -1.;
        g.get_mut("generate.pair")
            .ok_or_else(|| bad("fixture pair"))?[1000] = -1.;
        let result = construct(&parent, &g, pools, &mut reducer()?)?;
        assert_eq!(result.trials[0].name, "generate.pair");
        assert_eq!(result.trials[0].index, 1000);
        assert_eq!(result.trials[1].index, 98);
        assert_eq!(result.trials[2].index, 99);
        let (parent, g, mut pools) = fixture();
        pools[PROTECTED - 1].factor_keys[0] = UNARY as u32;
        assert!(construct(&parent, &g, pools, &mut reducer()?).is_err());
        let (parent, mut g, pools) = fixture();
        g.get_mut("generate.unary")
            .ok_or_else(|| bad("fixture unary"))?[0] = f32::NAN;
        assert!(construct(&parent, &g, pools, &mut reducer()?).is_err());
        Ok(())
    }
}
