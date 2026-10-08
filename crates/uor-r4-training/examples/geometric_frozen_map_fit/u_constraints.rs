//! One frozen adjacent-code U construction with transactional full alias pools.
//! Shared-action v2 changes Generate and every physical Copy alias before the
//! single authoritative clip/reduction. No Generate-only patch cache is used.
use super::native_proposals as np;
use super::*;
use uor_r4_integer::geometric_vocabulary_actions::{NativeVocabularyActions, VocabularyReduction};

const COORDINATES: usize = 960;
const FACTORS: usize = 8;
const PROTECTED: usize = 86;
const TOKEN_STRIDE: usize = 4096;
const SCORE_SHIFT: u32 = 22;

/// Token-major directed inv(local-state)*fixed-prototype unary incidence.
pub(super) struct UProtectedPool {
    pub required_token: u32,
    pub generate_q24: Vec<i64>,
    pub copy_ids: Vec<u32>,
    pub copy_q24: Vec<i64>,
    pub factor_keys: Vec<u16>,
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
    pub affected_copy_postings: usize,
    pub affected_pools: usize,
    pub affected_pool_indices: Vec<usize>,
    pub checked_pools: usize,
    /// First blocker only; later rows are not evaluated after rejection.
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
    /// Includes baseline and final fidelity reductions as well as trial rows.
    pub full_reductions: usize,
    pub changed_reference_reductions: usize,
    pub checked_generate_postings: usize,
    pub checked_copy_postings: usize,
    pub accepted_generate_postings: usize,
    pub accepted_copy_postings: usize,
    pub protected_decisions_preserved: bool,
}
#[derive(serde::Serialize)]
pub(super) struct Construction {
    pub edits: Vec<np::Edit>,
    pub trials: Vec<Trial>,
    pub stats: Stats,
    pub g_dot_actual_delta: f64,
    pub final_generate_scores: Vec<Vec<i64>>,
    pub final_copy_scores: Vec<Vec<i64>>,
    pub final_token_masses: Vec<Vec<u64>>,
    pub final_pool_summaries: Vec<VocabularyReduction>,
}
fn code(value: f32) -> Result<i8> {
    replay_require(
        value.is_finite() && (-1.75..=1.75).contains(&value),
        "U construction master outside strict Q4 range",
    )?;
    Ok((4.0 * value).round() as i8)
}
fn full_reduce(
    reducer: &mut NativeVocabularyActions,
    generate: &[i64],
    ids: &[u32],
    copy: &[i64],
    weights: &mut Vec<u64>,
    masses: &mut Vec<u64>,
) -> Result<VocabularyReduction> {
    weights.resize(reducer.legal_token_ids().len() + ids.len(), 0);
    masses.resize(generate.len(), 0);
    Ok(reducer.reduce_into(generate, ids, copy, weights, masses)?)
}

/// Caller masters/gradients never mutate. Every changed row is reduced in full
/// and staged before any row commits. Errors or a late blocker discard the
/// entire pending coordinate. The fixed gradient/order never refreshes.
pub(super) fn construct(
    parent: &np::Shadows,
    gradients: &np::Shadows,
    mut pools: Vec<UProtectedPool>,
    reducer: &mut NativeVocabularyActions,
) -> Result<Construction> {
    replay_require(pools.len() == PROTECTED, "U protected population must be86")?;
    let name = "continuation.unary";
    let master = parent.get(name).ok_or_else(|| bad("U master absent"))?;
    let gradient = gradients
        .get(name)
        .ok_or_else(|| bad("U gradient absent"))?;
    replay_require(
        master.len() == COORDINATES && gradient.len() == COORDINATES,
        "U eligible tensor shape differs",
    )?;
    let mut ordered = Vec::with_capacity(COORDINATES);
    for (index, (&original_master, &g)) in master.iter().zip(gradient).enumerate() {
        replay_require(g.is_finite(), "nonfinite U gradient")?;
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
            "U actual adjacent displacement fails linear descent",
        )?;
        ordered.push(Trial {
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
            affected_copy_postings: 0,
            affected_pools: 0,
            affected_pool_indices: Vec::new(),
            checked_pools: 0,
            blocking_pool: None,
            blocking_winner: None,
        });
    }
    ordered.sort_by(|a, b| {
        a.predicted_delta
            .total_cmp(&b.predicted_delta)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.index.cmp(&b.index))
    });
    // CSR stores Generate token postings. Physical aliases use the exact same
    // token's lane key, so all duplicates receive the same delta independently.
    let mut offsets = vec![0u32; COORDINATES + 1];
    for pool in &pools {
        replay_require(
            !pool.generate_q24.is_empty()
                && pool.generate_q24.len() <= TOKEN_STRIDE
                && pool.factor_keys.len() == pool.generate_q24.len() * FACTORS
                && pool.copy_ids.len() == pool.copy_q24.len(),
            "U protected incidence shape differs",
        )?;
        for &token in &pool.copy_ids {
            replay_require(
                (token as usize) < pool.generate_q24.len(),
                "U physical Copy token outside vocabulary",
            )?;
        }
        for keys in pool.factor_keys.chunks_exact(FACTORS) {
            for (lane, &key) in keys.iter().enumerate() {
                replay_require(
                    (lane * 120..(lane + 1) * 120).contains(&(key as usize)),
                    "U logical lane key/order differs",
                )?;
                offsets[key as usize + 1] = offsets[key as usize + 1]
                    .checked_add(1)
                    .ok_or_else(|| bad("U incidence count overflow"))?;
            }
        }
    }
    for i in 1..offsets.len() {
        offsets[i] = offsets[i]
            .checked_add(offsets[i - 1])
            .ok_or_else(|| bad("U incidence offset overflow"))?;
    }
    let mut postings = vec![0u32; offsets[COORDINATES] as usize];
    let mut cursors = offsets[..COORDINATES].to_vec();
    for (row, pool) in pools.iter().enumerate() {
        for (token, keys) in pool.factor_keys.chunks_exact(FACTORS).enumerate() {
            for &key in keys {
                postings[cursors[key as usize] as usize] = (row * TOKEN_STRIDE + token) as u32;
                cursors[key as usize] += 1;
            }
        }
    }
    drop(cursors);
    let mut weights = Vec::new();
    let mut masses = Vec::new();
    let mut summaries = Vec::with_capacity(PROTECTED);
    for pool in &pools {
        let summary = full_reduce(
            reducer,
            &pool.generate_q24,
            &pool.copy_ids,
            &pool.copy_q24,
            &mut weights,
            &mut masses,
        )?;
        replay_require(
            summary.chosen_token_id == pool.required_token,
            "U protected baseline winner differs",
        )?;
        summaries.push(summary);
    }
    let mut stats = Stats {
        coordinates: COORDINATES,
        protected_pools: PROTECTED,
        incidence_postings: postings.len(),
        incidence_bytes: 4 * (postings.len() + offsets.len())
            + 2 * pools.iter().map(|p| p.factor_keys.len()).sum::<usize>(),
        full_reductions: PROTECTED,
        ..Stats::default()
    };
    let mut edits = Vec::new();
    let mut trials = Vec::with_capacity(COORDINATES);
    let mut g_dot_actual_delta = 0.0;
    for mut trial in ordered {
        if trial.before == trial.after {
            if trial.status == "zero_gradient" {
                stats.zero_gradient += 1;
            } else {
                stats.saturated += 1;
            }
            trials.push(trial);
            continue;
        }
        let key = trial.index;
        let lane = key / 120;
        let incident = &postings[offsets[key] as usize..offsets[key + 1] as usize];
        trial.affected_postings = incident.len();
        trial.affected_pools = usize::from(!incident.is_empty())
            + incident
                .windows(2)
                .filter(|w| w[0] as usize / TOKEN_STRIDE != w[1] as usize / TOKEN_STRIDE)
                .count();
        for &posting in incident {
            let row = posting as usize / TOKEN_STRIDE;
            if trial.affected_pool_indices.last() != Some(&row) {
                trial.affected_pool_indices.push(row);
            }
        }
        // Count complete physical alias support before first-blocker traversal.
        for pool in &pools {
            trial.affected_copy_postings += pool
                .copy_ids
                .iter()
                .filter(|&&t| pool.factor_keys[t as usize * FACTORS + lane] as usize == key)
                .count();
        }
        let delta = i64::from(trial.after - trial.before) << SCORE_SHIFT;
        let mut pending = Vec::with_capacity(trial.affected_pools);
        let mut cursor = 0;
        while cursor < incident.len() {
            let row = incident[cursor] as usize / TOKEN_STRIDE;
            let pool = &pools[row];
            let mut generate = pool.generate_q24.clone();
            let mut copy = pool.copy_q24.clone();
            let start = cursor;
            while cursor < incident.len() && incident[cursor] as usize / TOKEN_STRIDE == row {
                let token = incident[cursor] as usize % TOKEN_STRIDE;
                generate[token] = generate[token]
                    .checked_add(delta)
                    .ok_or_else(|| bad("U Generate atom overflow"))?;
                cursor += 1;
            }
            let mut copy_count = 0;
            for (i, &token) in pool.copy_ids.iter().enumerate() {
                if pool.factor_keys[token as usize * FACTORS + lane] as usize == key {
                    copy[i] = copy[i]
                        .checked_add(delta)
                        .ok_or_else(|| bad("U physical Copy atom overflow"))?;
                    copy_count += 1;
                }
            }
            let summary = full_reduce(
                reducer,
                &generate,
                &pool.copy_ids,
                &copy,
                &mut weights,
                &mut masses,
            )?;
            stats.checked_pools += 1;
            stats.full_reductions += 1;
            stats.checked_generate_postings += cursor - start;
            stats.checked_copy_postings += copy_count;
            stats.changed_reference_reductions +=
                usize::from(summary.max_score_q24 != summaries[row].max_score_q24);
            trial.checked_pools += 1;
            if summary.chosen_token_id != pool.required_token {
                trial.blocking_pool = Some(row);
                trial.blocking_winner = Some(summary.chosen_token_id);
                break;
            }
            pending.push((row, generate, copy, summary));
        }
        if trial.blocking_pool.is_some() {
            stats.rejected += 1;
            trial.status = "rejected".to_owned();
        } else {
            // All fallible arithmetic/reductions and all winner predicates are
            // complete. Only infallible owned-buffer replacement remains.
            for (row, generate, copy, summary) in pending {
                pools[row].generate_q24 = generate;
                pools[row].copy_q24 = copy;
                summaries[row] = summary;
            }
            stats.accepted += 1;
            stats.accepted_generate_postings += incident.len();
            stats.accepted_copy_postings += trial.affected_copy_postings;
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
    stats.protected_decisions_preserved = pools
        .iter()
        .zip(&summaries)
        .all(|(p, s)| p.required_token == s.chosen_token_id);
    replay_require(
        stats.protected_decisions_preserved,
        "U protected winner changed",
    )?;
    let mut final_token_masses = Vec::with_capacity(PROTECTED);
    for (pool, expected) in pools.iter().zip(&summaries) {
        let actual = full_reduce(
            reducer,
            &pool.generate_q24,
            &pool.copy_ids,
            &pool.copy_q24,
            &mut weights,
            &mut masses,
        )?;
        stats.full_reductions += 1;
        replay_require(
            &actual == expected,
            "U final independent full reduction differs",
        )?;
        final_token_masses.push(masses.clone());
    }
    Ok(Construction {
        edits,
        trials,
        stats,
        g_dot_actual_delta,
        final_generate_scores: pools.iter().map(|p| p.generate_q24.clone()).collect(),
        final_copy_scores: pools.iter().map(|p| p.copy_q24.clone()).collect(),
        final_token_masses,
        final_pool_summaries: summaries,
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
    fn fixture() -> (np::Shadows, np::Shadows, Vec<UProtectedPool>) {
        let parent = [
            ("continuation.unary".to_owned(), vec![0.; COORDINATES]),
            ("generate.bias".to_owned(), vec![-0.0; 6]),
        ]
        .into_iter()
        .collect();
        let gradient = [("continuation.unary".to_owned(), vec![0.; COORDINATES])]
            .into_iter()
            .collect();
        let pools = (0..PROTECTED)
            .map(|_| {
                let mut factor_keys = Vec::new();
                for token in 0..6 {
                    factor_keys.extend((0..8).map(|lane| (lane * 120 + token) as u16));
                }
                let mut generate_q24 = vec![0; 6];
                generate_q24[5] = 4 << 24;
                UProtectedPool {
                    required_token: 5,
                    generate_q24,
                    copy_ids: vec![3, 3, 5],
                    copy_q24: vec![-(4 << 24); 3],
                    factor_keys,
                }
            })
            .collect();
        (parent, gradient, pools)
    }
    #[test]
    fn u_signed_v2_scale_updates_every_duplicate_physical_alias() -> Result<()> {
        let (parent, mut g, pools) = fixture();
        let before = parent.clone();
        g.get_mut("continuation.unary")
            .ok_or_else(|| bad("fixture gradient"))?[3] = 1.;
        g.get_mut("continuation.unary")
            .ok_or_else(|| bad("fixture gradient"))?[5] = -1.;
        let c = construct(&parent, &g, pools, &mut reducer()?)?;
        assert!(np::same_bits(&parent, &before));
        assert_eq!(c.stats.accepted, 2);
        for row in 0..PROTECTED {
            assert_eq!(c.final_generate_scores[row][3], -(1 << 22));
            assert_eq!(c.final_generate_scores[row][5], (4 << 24) + (1 << 22));
            assert_eq!(
                c.final_copy_scores[row],
                vec![
                    -(4 << 24) - (1 << 22),
                    -(4 << 24) - (1 << 22),
                    -(4 << 24) + (1 << 22)
                ]
            );
        }
        assert_eq!(c.stats.accepted_copy_postings, 3 * PROTECTED);
        assert_eq!(c.trials.len(), COORDINATES);
        Ok(())
    }
    #[test]
    fn u_late_blocker_discards_all_staged_generate_and_copy_buffers() -> Result<()> {
        let (parent, mut g, mut pools) = fixture();
        pools[PROTECTED - 1].generate_q24.fill(-(4 << 24));
        pools[PROTECTED - 1].generate_q24[5] = 0;
        pools[PROTECTED - 1].generate_q24[3] = -(1 << 21);
        let gen = pools
            .iter()
            .map(|p| p.generate_q24.clone())
            .collect::<Vec<_>>();
        let copy = pools.iter().map(|p| p.copy_q24.clone()).collect::<Vec<_>>();
        g.get_mut("continuation.unary")
            .ok_or_else(|| bad("fixture gradient"))?[3] = -1.;
        let c = construct(&parent, &g, pools, &mut reducer()?)?;
        assert_eq!(c.stats.accepted, 0);
        assert_eq!(c.stats.rejected, 1);
        assert_eq!(c.trials[0].blocking_pool, Some(PROTECTED - 1));
        assert_eq!(c.trials[0].checked_pools, PROTECTED);
        assert_eq!(c.final_generate_scores, gen);
        assert_eq!(c.final_copy_scores, copy);
        Ok(())
    }
    #[test]
    fn u_full_reducer_handles_changed_reference_and_legal_token_ties() -> Result<()> {
        let (parent, mut g, mut pools) = fixture();
        for p in &mut pools {
            p.required_token = 4;
            p.generate_q24.fill(-(4 << 24));
            p.generate_q24[4] = 0;
            p.generate_q24[5] = -(1 << 22);
            p.copy_ids = vec![3, 3];
            p.copy_q24 = vec![-(4 << 24); 2];
        }
        g.get_mut("continuation.unary")
            .ok_or_else(|| bad("fixture gradient"))?[5] = -2.;
        g.get_mut("continuation.unary")
            .ok_or_else(|| bad("fixture gradient"))?[4] = -1.;
        let c = construct(&parent, &g, pools, &mut reducer()?)?;
        assert_eq!(c.stats.accepted, 2);
        assert_eq!(c.stats.rejected, 0);
        assert!(c.stats.changed_reference_reductions >= PROTECTED);
        assert!(c
            .final_pool_summaries
            .iter()
            .all(|s| s.chosen_token_id == 4 && s.max_score_q24 == 1 << 22));
        Ok(())
    }
    #[test]
    fn u_frozen_actual_displacement_rank_once_noops_and_invalid_incidence() -> Result<()> {
        let (mut parent, mut g, pools) = fixture();
        parent
            .get_mut("continuation.unary")
            .ok_or_else(|| bad("fixture master"))?[99] = 0.07;
        parent
            .get_mut("continuation.unary")
            .ok_or_else(|| bad("fixture master"))?[101] = 1.75;
        for i in [98, 99, 101] {
            g.get_mut("continuation.unary")
                .ok_or_else(|| bad("fixture gradient"))?[i] = -1.;
        }
        let c = construct(&parent, &g, pools, &mut reducer()?)?;
        assert_eq!(c.trials[0].index, 98);
        assert_eq!(c.trials[1].index, 99);
        assert_eq!(c.stats.accepted, 2);
        assert_eq!(c.stats.saturated, 1);
        assert_eq!(c.stats.zero_gradient, 957);
        assert_eq!(
            c.trials
                .iter()
                .map(|t| t.index)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            COORDINATES
        );
        let (parent, g, mut pools) = fixture();
        pools[0].factor_keys[0] = 120;
        assert!(construct(&parent, &g, pools, &mut reducer()?).is_err());
        let (parent, mut g, pools) = fixture();
        g.get_mut("continuation.unary")
            .ok_or_else(|| bad("fixture gradient"))?[0] = f32::NAN;
        assert!(construct(&parent, &g, pools, &mut reducer()?).is_err());
        Ok(())
    }
}
