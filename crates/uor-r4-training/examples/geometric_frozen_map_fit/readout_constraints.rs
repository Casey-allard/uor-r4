//! One frozen Cue/Prefix adjacent-code pass with donor-aware atomic guards.
use super::native_proposals as np;
use super::*;
use uor_r4_integer::geometric_vocabulary_actions::{NativeVocabularyActions, VocabularyReduction};

const FAMILY_SIZE: usize = 960;
const LANES: usize = 8;
const GUARDS: usize = 377;
const NAMES: [&str; 2] = ["cue.coefficients", "prefix.coefficients"];

#[derive(serde::Serialize)]
pub(super) struct ReadoutProtectedPool {
    pub required_token: u32,
    pub base_copy_q24: Vec<i64>,
    pub copy_ids: Vec<u32>,
    pub frozen_u_q24: Vec<i64>,
    /// Physical occurrence-major lanes; absent Cue contributes no coefficient.
    pub cue_keys: Vec<Option<u16>>,
    pub prefix_keys: Vec<u16>,
    /// Exact BASEGenerate for `donor`, not a fixed vector across donor changes.
    pub base_generate_q24: Vec<i64>,
    pub donor: usize,
}
pub(super) struct ReducedPool {
    pub generate_q24: Vec<i64>,
    pub copy_q24: Vec<i64>,
    pub token_masses: Vec<u64>,
    pub summary: VocabularyReduction,
}
pub(super) fn earliest_base_donor(scores: &[i64]) -> Result<usize> {
    let mut best = 0;
    replay_require(!scores.is_empty(), "readout physical Copy empty")?;
    for i in 1..scores.len() {
        if scores[i] > scores[best] {
            best = i;
        }
    }
    Ok(best)
}
pub(super) fn reduce_pool(
    reducer: &mut NativeVocabularyActions,
    base_generate: &[i64],
    base_copy: &[i64],
    copy_ids: &[u32],
    frozen_u: &[i64],
) -> Result<ReducedPool> {
    replay_require(
        base_generate.len() == frozen_u.len() && base_copy.len() == copy_ids.len(),
        "readout raw pool shape",
    )?;
    let generate_q24 = base_generate
        .iter()
        .zip(frozen_u)
        .map(|(g, u)| {
            g.checked_add(*u)
                .ok_or_else(|| bad("readout Generate U overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    let copy_q24 = base_copy
        .iter()
        .zip(copy_ids)
        .map(|(c, id)| {
            let u = frozen_u
                .get(*id as usize)
                .ok_or_else(|| bad("readout alias outside legal vocabulary"))?;
            c.checked_add(*u)
                .ok_or_else(|| bad("readout Copy U overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut weights = vec![0; reducer.legal_token_ids().len() + copy_ids.len()];
    let mut token_masses = vec![0; base_generate.len()];
    let summary = reducer.reduce_into(
        &generate_q24,
        copy_ids,
        &copy_q24,
        &mut weights,
        &mut token_masses,
    )?;
    Ok(ReducedPool {
        generate_q24,
        copy_q24,
        token_masses,
        summary,
    })
}
#[derive(Clone, Debug, serde::Serialize)]
pub(super) struct DonorTransition {
    pub pool: usize,
    pub before: usize,
    pub after: usize,
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
    pub affected_pool_indices: Vec<usize>,
    pub checked_pools: usize,
    /// Evaluated transitions only; early rejection does not enumerate all effects.
    pub donor_transitions: Vec<DonorTransition>,
    /// First blocker only, not a complete blocker set.
    pub blocking_pool: Option<usize>,
    pub blocking_winner: Option<u32>,
}
#[derive(Default, serde::Serialize)]
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
    pub full_reductions: usize,
    pub changed_reference_reductions: usize,
    pub donor_generate_calls: usize,
    pub evaluated_donor_switches: usize,
    pub accepted_donor_switches: usize,
    pub protected_decisions_preserved: bool,
}
#[derive(serde::Serialize)]
pub(super) struct Construction {
    pub edits: Vec<np::Edit>,
    pub trials: Vec<Trial>,
    pub stats: Stats,
    pub g_dot_actual_delta: f64,
    pub final_base_copy_scores: Vec<Vec<i64>>,
    pub final_base_generate_scores: Vec<Vec<i64>>,
    pub final_generate_scores: Vec<Vec<i64>>,
    pub final_copy_scores: Vec<Vec<i64>>,
    pub final_donors: Vec<usize>,
    pub final_token_masses: Vec<Vec<u64>>,
    pub final_pool_summaries: Vec<VocabularyReduction>,
}
fn native_code(v: f32) -> Result<i8> {
    replay_require(
        v.is_finite() && (-1.75..=1.75).contains(&v),
        "readout master outside legal Q4 range",
    )?;
    Ok((v * 4.).round() as i8)
}
pub(super) fn construct(
    parent: &np::Shadows,
    gradients: &np::Shadows,
    pools: Vec<ReadoutProtectedPool>,
    reducer: &mut NativeVocabularyActions,
    donor_generate: impl FnMut(usize, usize) -> Result<Vec<i64>>,
) -> Result<Construction> {
    replay_require(pools.len() == GUARDS, "readout guards must be377")?;
    replay_require(
        reducer.legal_token_ids().iter().copied().eq(0u32..4096),
        "readout full4096 vocabulary required",
    )?;
    construct_inner(parent, gradients, pools, reducer, donor_generate)
}
fn construct_inner(
    parent: &np::Shadows,
    gradients: &np::Shadows,
    mut pools: Vec<ReadoutProtectedPool>,
    reducer: &mut NativeVocabularyActions,
    mut donor_generate: impl FnMut(usize, usize) -> Result<Vec<i64>>,
) -> Result<Construction> {
    let mut trials = Vec::with_capacity(2 * FAMILY_SIZE);
    for name in NAMES {
        let master = parent
            .get(name)
            .ok_or_else(|| bad("readout master absent"))?;
        let gradient = gradients
            .get(name)
            .ok_or_else(|| bad("readout gradient absent"))?;
        replay_require(
            master.len() == FAMILY_SIZE && gradient.len() == FAMILY_SIZE,
            "readout eligible shape",
        )?;
        for (index, (&original_master, &g)) in master.iter().zip(gradient).enumerate() {
            replay_require(g.is_finite(), "readout nonfinite gradient")?;
            let before = native_code(original_master)?;
            let direction = if g > 0. {
                -1
            } else if g < 0. {
                1
            } else {
                0
            };
            let after = (before + direction).clamp(-7, 7);
            let predicted_delta = if before == after {
                0.
            } else {
                f64::from(g) * (f64::from(after) * 0.25 - f64::from(original_master))
            };
            replay_require(
                before == after || predicted_delta < 0.,
                "readout actual displacement is not descent",
            )?;
            trials.push(Trial {
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
                .into(),
                affected_postings: 0,
                affected_pool_indices: Vec::new(),
                checked_pools: 0,
                donor_transitions: Vec::new(),
                blocking_pool: None,
                blocking_winner: None,
            });
        }
    }
    trials.sort_by(|a, b| {
        a.predicted_delta
            .total_cmp(&b.predicted_delta)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.index.cmp(&b.index))
    });
    let mut postings = vec![Vec::<(u32, u32)>::new(); 2 * FAMILY_SIZE];
    let mut summaries = Vec::with_capacity(pools.len());
    let mut stats = Stats {
        coordinates: trials.len(),
        protected_pools: pools.len(),
        ..Stats::default()
    };
    for (pi, p) in pools.iter().enumerate() {
        let n = p.copy_ids.len();
        replay_require(
            pi <= u32::MAX as usize
                && n <= u32::MAX as usize
                && p.cue_keys.len() == n * LANES
                && p.prefix_keys.len() == n * LANES
                && p.base_generate_q24.len() == reducer.legal_token_ids().len()
                && p.frozen_u_q24.len() == p.base_generate_q24.len(),
            "readout pool/incidence shape",
        )?;
        replay_require(
            p.donor == earliest_base_donor(&p.base_copy_q24)?,
            "readout baseline donor differs",
        )?;
        for occurrence in 0..n {
            for lane in 0..LANES {
                if let Some(key) = p.cue_keys[occurrence * LANES + lane] {
                    replay_require(
                        usize::from(key) / 120 == lane,
                        "readout Cue key lane/domain",
                    )?;
                    postings[usize::from(key)].push((pi as u32, occurrence as u32));
                }
                let key = usize::from(p.prefix_keys[occurrence * LANES + lane]);
                replay_require(key / 120 == lane, "readout Prefix key lane/domain")?;
                postings[FAMILY_SIZE + key].push((pi as u32, occurrence as u32));
            }
        }
        let r = reduce_pool(
            reducer,
            &p.base_generate_q24,
            &p.base_copy_q24,
            &p.copy_ids,
            &p.frozen_u_q24,
        )?;
        stats.full_reductions += 1;
        replay_require(
            r.summary.chosen_token_id == p.required_token,
            "readout baseline protected winner differs",
        )?;
        summaries.push(r.summary);
    }
    stats.incidence_postings = postings.iter().map(Vec::len).sum();
    stats.incidence_bytes = postings
        .iter()
        .map(|p| p.capacity() * std::mem::size_of::<(u32, u32)>())
        .sum::<usize>()
        + postings.capacity() * std::mem::size_of::<Vec<(u32, u32)>>()
        + pools
            .iter()
            .map(|p| {
                p.cue_keys.capacity() * std::mem::size_of::<Option<u16>>()
                    + p.prefix_keys.capacity() * std::mem::size_of::<u16>()
            })
            .sum::<usize>();
    let mut edits = Vec::new();
    let mut g_dot_actual_delta = 0.;
    for t in &mut trials {
        if t.status == "zero_gradient" {
            stats.zero_gradient += 1;
            continue;
        }
        if t.status == "saturated" {
            stats.saturated += 1;
            continue;
        }
        let family = usize::from(t.name == NAMES[1]);
        let ps = &postings[family * FAMILY_SIZE + t.index];
        t.affected_postings = ps.len();
        let mut grouped = BTreeMap::<usize, Vec<usize>>::new();
        for &(pi, oi) in ps {
            grouped.entry(pi as usize).or_default().push(oi as usize);
        }
        t.affected_pool_indices = grouped.keys().copied().collect();
        let delta = i64::from(t.after - t.before) * (1i64 << 22);
        let mut staged = Vec::with_capacity(grouped.len());
        for (pi, occurrences) in grouped {
            let p = &pools[pi];
            let mut copy = p.base_copy_q24.clone();
            for oi in occurrences {
                copy[oi] = copy[oi]
                    .checked_add(delta)
                    .ok_or_else(|| bad("readout BASECopy overflow"))?;
            }
            let donor = earliest_base_donor(&copy)?;
            let generate = if donor == p.donor {
                p.base_generate_q24.clone()
            } else {
                stats.donor_generate_calls += 1;
                stats.evaluated_donor_switches += 1;
                t.donor_transitions.push(DonorTransition {
                    pool: pi,
                    before: p.donor,
                    after: donor,
                });
                donor_generate(pi, donor)?
            };
            let r = reduce_pool(reducer, &generate, &copy, &p.copy_ids, &p.frozen_u_q24)?;
            stats.full_reductions += 1;
            stats.checked_pools += 1;
            t.checked_pools += 1;
            stats.changed_reference_reductions +=
                usize::from(r.summary.max_score_q24 != summaries[pi].max_score_q24);
            if r.summary.chosen_token_id != p.required_token {
                t.blocking_pool = Some(pi);
                t.blocking_winner = Some(r.summary.chosen_token_id);
                break;
            }
            staged.push((pi, copy, generate, donor, r.summary));
        }
        if t.blocking_pool.is_some() {
            t.status = "rejected".into();
            stats.rejected += 1;
            continue;
        }
        // No fallible operation follows the first mutation of accepted row state.
        for (pi, copy, generate, donor, summary) in staged {
            let p = &mut pools[pi];
            stats.accepted_donor_switches += usize::from(p.donor != donor);
            p.base_copy_q24 = copy;
            p.base_generate_q24 = generate;
            p.donor = donor;
            summaries[pi] = summary;
        }
        if ps.is_empty() {
            stats.no_protected_incidence += 1;
        }
        t.status = "accepted".into();
        stats.accepted += 1;
        g_dot_actual_delta += t.predicted_delta;
        edits.push(np::Edit {
            name: t.name.clone(),
            index: t.index,
            before: t.before,
            after: t.after,
        });
    }
    let mut final_generate_scores = Vec::new();
    let mut final_copy_scores = Vec::new();
    let mut final_token_masses = Vec::new();
    for (pi, p) in pools.iter().enumerate() {
        let r = reduce_pool(
            reducer,
            &p.base_generate_q24,
            &p.base_copy_q24,
            &p.copy_ids,
            &p.frozen_u_q24,
        )?;
        stats.full_reductions += 1;
        replay_require(
            r.summary == summaries[pi] && r.summary.chosen_token_id == p.required_token,
            "readout final independent reduction differs",
        )?;
        final_generate_scores.push(r.generate_q24);
        final_copy_scores.push(r.copy_q24);
        final_token_masses.push(r.token_masses);
    }
    stats.protected_decisions_preserved = true;
    Ok(Construction {
        edits,
        trials,
        stats,
        g_dot_actual_delta,
        final_base_copy_scores: pools.iter().map(|p| p.base_copy_q24.clone()).collect(),
        final_base_generate_scores: pools.iter().map(|p| p.base_generate_q24.clone()).collect(),
        final_donors: pools.iter().map(|p| p.donor).collect(),
        final_generate_scores,
        final_copy_scores,
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
    fn fixture(rows: usize) -> (np::Shadows, np::Shadows, Vec<ReadoutProtectedPool>) {
        let parent = NAMES
            .into_iter()
            .map(|n| (n.to_owned(), vec![0.; 960]))
            .collect();
        let gradient = NAMES
            .into_iter()
            .map(|n| (n.to_owned(), vec![0.; 960]))
            .collect();
        let pools = (0..rows)
            .map(|_| {
                let keys = (0..3)
                    .flat_map(|oi| (0..8).map(move |lane| (lane * 120 + oi) as u16))
                    .collect::<Vec<_>>();
                let mut generate = vec![-(4 << 24); 6];
                generate[5] = 4 << 24;
                ReadoutProtectedPool {
                    required_token: 5,
                    base_copy_q24: vec![0, -(1 << 21), -(1 << 24)],
                    copy_ids: vec![3, 3, 5],
                    frozen_u_q24: vec![1 << 22; 6],
                    cue_keys: keys.iter().map(|k| Some(*k)).collect(),
                    prefix_keys: keys,
                    base_generate_q24: generate,
                    donor: 0,
                }
            })
            .collect();
        (parent, gradient, pools)
    }
    #[test]
    fn readout_donor_switch_recomputes_generate_and_duplicate_aliases() -> Result<()> {
        let (parent, mut gradient, pools) = fixture(2);
        gradient.get_mut(NAMES[0]).ok_or_else(|| bad("fixture"))?[1] = -1.;
        let old = parent.clone();
        let mut calls = Vec::new();
        let c = construct_inner(&parent, &gradient, pools, &mut reducer()?, |pi, donor| {
            calls.push((pi, donor));
            let mut g = vec![-(4 << 24); 6];
            g[5] = 5 << 24;
            Ok(g)
        })?;
        assert!(np::same_bits(&parent, &old));
        assert_eq!(calls, vec![(0, 1), (1, 1)]);
        assert_eq!(c.final_donors, vec![1, 1]);
        assert_eq!(c.final_base_generate_scores[0][5], 5 << 24);
        assert_eq!(
            c.final_copy_scores[0],
            vec![1 << 22, (1 << 21) + (1 << 22), -(1 << 24) + (1 << 22)]
        );
        assert_eq!(c.stats.accepted, 1);
        assert_eq!(c.stats.zero_gradient, 1919);
        assert_eq!(c.trials.len(), 1920);
        assert_eq!(c.stats.accepted_donor_switches, 2);
        Ok(())
    }
    #[test]
    fn readout_late_donor_veto_restores_all_rows_then_other_family_can_commit() -> Result<()> {
        let (parent, mut gradient, pools) = fixture(3);
        let original = pools
            .iter()
            .map(|p| p.base_copy_q24.clone())
            .collect::<Vec<_>>();
        gradient.get_mut(NAMES[0]).ok_or_else(|| bad("fixture"))?[1] = -2.;
        gradient.get_mut(NAMES[1]).ok_or_else(|| bad("fixture"))?[2] = 1.;
        let c = construct_inner(&parent, &gradient, pools, &mut reducer()?, |pi, _| {
            let mut g = vec![-(4 << 24); 6];
            g[if pi == 2 { 3 } else { 5 }] = 6 << 24;
            Ok(g)
        })?;
        assert_eq!(c.stats.rejected, 1);
        assert_eq!(c.stats.accepted, 1);
        assert_eq!(c.trials[0].blocking_pool, Some(2));
        assert_eq!(c.trials[0].checked_pools, 3);
        assert_eq!(c.final_donors, vec![0; 3]);
        assert_eq!(c.stats.accepted_donor_switches, 0);
        for (pi, s) in original.iter().enumerate() {
            assert_eq!(c.final_base_copy_scores[pi][..2], s[..2]);
            assert_eq!(c.final_base_copy_scores[pi][2], s[2] - (1 << 22));
            assert_eq!(c.final_base_generate_scores[pi][5], 4 << 24);
        }
        assert_eq!(c.edits[0].name, NAMES[1]);
        Ok(())
    }
    #[test]
    fn readout_signed_scale_and_earliest_physical_ties_are_exact() -> Result<()> {
        assert_eq!(earliest_base_donor(&[7, 7, 6])?, 0);
        assert_eq!(earliest_base_donor(&[6, 7, 7])?, 1);
        assert!(earliest_base_donor(&[]).is_err());
        let mut r = reducer()?;
        let p = reduce_pool(
            &mut r,
            &[0; 6],
            &[-(1 << 22), -(1 << 22), 0],
            &[3, 3, 5],
            &[0, 0, 0, 1 << 22, 0, -(1 << 22)],
        )?;
        assert_eq!(p.copy_q24, vec![0, 0, -(1 << 22)]);
        assert_eq!(p.generate_q24[3], 1 << 22);
        assert_eq!(p.generate_q24[5], -(1 << 22));
        assert_eq!(p.summary.max_score_q24, 1 << 22);
        assert_eq!(p.summary.chosen_token_id, 3);
        Ok(())
    }
    #[test]
    fn readout_once_order_bounds_no_incidence_and_invalid_inputs() -> Result<()> {
        let (mut parent, mut gradient, pools) = fixture(1);
        parent.get_mut(NAMES[0]).ok_or_else(|| bad("fixture"))?[959] = 1.75;
        gradient.get_mut(NAMES[0]).ok_or_else(|| bad("fixture"))?[959] = -1.;
        gradient.get_mut(NAMES[0]).ok_or_else(|| bad("fixture"))?[958] = -1.;
        gradient.get_mut(NAMES[1]).ok_or_else(|| bad("fixture"))?[958] = -1.;
        let c = construct_inner(&parent, &gradient, pools, &mut reducer()?, |_, _| {
            Err(bad("unused callback"))
        })?;
        assert_eq!(c.stats.saturated, 1);
        assert_eq!(c.stats.no_protected_incidence, 2);
        assert_eq!(c.edits[0].name, NAMES[0]);
        assert_eq!(c.edits[1].name, NAMES[1]);
        assert_eq!(
            c.trials
                .iter()
                .map(|t| (&t.name, t.index))
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            1920
        );
        let (_, mut bad_gradient, bad_pools) = fixture(1);
        bad_gradient
            .get_mut(NAMES[0])
            .ok_or_else(|| bad("fixture"))?[0] = f32::NAN;
        assert!(construct_inner(
            &parent,
            &bad_gradient,
            bad_pools,
            &mut reducer()?,
            |_, _| Err(bad("unused"))
        )
        .is_err());
        let (_, gradient, mut bad_pools) = fixture(1);
        bad_pools[0].prefix_keys[0] = 960;
        assert!(
            construct_inner(&parent, &gradient, bad_pools, &mut reducer()?, |_, _| Err(
                bad("unused")
            ))
            .is_err()
        );
        Ok(())
    }
}
