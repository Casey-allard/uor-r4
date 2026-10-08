//! One deterministic, feasibility-preserving pass over shared Context Q4 coefficients.
//! This is a greedy feasible-prefix construction, not a global discrete optimizer.
use super::native_proposals as np;
use super::*;
use sha2::Digest;
use uor_r4_integer::geometric_context::{ContextDecisionEvent, ContextDecisionFamily};
use uor_r4_integer::geometric_context_q4::basis_score_q24;

const LANES: usize = 8;
const COORDINATES: usize = 17_472;

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
    pub affected_events: usize,
    pub checked_events: usize,
    pub blocking_event: Option<usize>,
    pub blocking_winner: Option<u8>,
    pub blocking_competitor: Option<u8>,
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub(super) struct Stats {
    pub coordinates: usize,
    pub protected_events: usize,
    pub accepted: usize,
    pub rejected: usize,
    pub zero_gradient: usize,
    pub saturated: usize,
    pub constraint_checks: usize,
    pub protected_decisions_preserved: bool,
}
#[derive(Clone, Debug, serde::Serialize)]
pub(super) struct Construction {
    pub edits: Vec<np::Edit>,
    pub trials: Vec<Trial>,
    pub stats: Stats,
    pub g_dot_actual_delta: f64,
    pub final_scores_sha256: String,
}

fn classes(basis: usize) -> usize {
    if basis >= 4 {
        33
    } else {
        120
    }
}
fn event_family(family: &ContextDecisionFamily) -> usize {
    match family {
        ContextDecisionFamily::Transition => 0,
        ContextDecisionFamily::Root => 1,
        ContextDecisionFamily::Category => 2,
    }
}
fn legal_code(value: f32) -> Result<i8> {
    replay_require(
        value.is_finite() && (-1.75..=1.75).contains(&value),
        "Context constrained construction master outside Q4 range",
    )?;
    Ok((4.0 * value).round() as i8)
}

// Max tree stores the exact earliest-choice constraint: a lower-index rival needs
// one integer score unit less than the incumbent; a higher rival may tie it.
struct Protected {
    own: u8,
    neighbor: u8,
    winner: usize,
    scores: Vec<i64>,
    leaf: usize,
    rivals: Vec<(i64, usize)>,
}
impl Protected {
    fn new(event: &ContextDecisionEvent) -> Result<Self> {
        let count = if event_family(&event.family) == 2 {
            33
        } else {
            120
        };
        replay_require(
            event.scores_q24.len() == count
                && usize::from(event.winner) < count
                && event.own < 120
                && event.neighbor < 120
                && event.flat_lane < LANES,
            "invalid protected Context decision event",
        )?;
        let winner = usize::from(event.winner);
        replay_require(
            event.scores_q24.iter().enumerate().all(|(a, s)| {
                a == winner
                    || if a < winner {
                        event.scores_q24[winner] > *s
                    } else {
                        event.scores_q24[winner] >= *s
                    }
            }),
            "protected Context event winner violates exact native tie order",
        )?;
        let leaf = count.next_power_of_two();
        let mut p = Self {
            own: event.own,
            neighbor: event.neighbor,
            winner,
            scores: event.scores_q24.clone(),
            leaf,
            rivals: vec![(i64::MIN, usize::MAX); 2 * leaf],
        };
        for a in 0..count {
            p.update_rival(a)?;
        }
        Ok(p)
    }
    fn update_rival(&mut self, a: usize) -> Result<()> {
        let value = if a == self.winner {
            (i64::MIN, usize::MAX)
        } else {
            (
                self.scores[a]
                    .checked_add(i64::from(a < self.winner))
                    .ok_or_else(|| bad("Context constraint score overflow"))?,
                a,
            )
        };
        let mut i = self.leaf + a;
        self.rivals[i] = value;
        while i > 1 {
            i /= 2;
            let (l, r) = (self.rivals[2 * i], self.rivals[2 * i + 1]);
            self.rivals[i] = if l.0 > r.0 || (l.0 == r.0 && l.1 < r.1) {
                l
            } else {
                r
            };
        }
        Ok(())
    }
    fn blocker(&self, class: usize, delta: i64) -> Result<Option<usize>> {
        let changed = self.scores[class]
            .checked_add(delta)
            .ok_or_else(|| bad("Context candidate score overflow"))?;
        if class == self.winner {
            Ok((changed < self.rivals[1].0).then_some(self.rivals[1].1))
        } else {
            let required = changed
                .checked_add(i64::from(class < self.winner))
                .ok_or_else(|| bad("Context candidate tie margin overflow"))?;
            Ok((self.scores[self.winner] < required).then_some(class))
        }
    }
    fn apply(&mut self, class: usize, delta: i64) -> Result<()> {
        self.scores[class] = self.scores[class]
            .checked_add(delta)
            .ok_or_else(|| bad("Context accepted score overflow"))?;
        self.update_rival(class)
    }
}

/// Does not mutate either caller snapshot. Every nonselected master, including
/// off-grid values and signed zero, stays untouched when `np::edited` applies edits.
pub(super) fn construct(
    parent: &np::Shadows,
    gradients: &np::Shadows,
    events: &[ContextDecisionEvent],
) -> Result<Construction> {
    replay_require(
        !events.is_empty(),
        "Context construction requires protected decisions",
    )?;
    let mut codes = Vec::<Vec<i8>>::new();
    let mut ordered = Vec::<(usize, Trial)>::new();
    for (basis, suffix) in np::BASIS.iter().enumerate() {
        let name = format!("consumer.context.{suffix}");
        let master = parent
            .get(&name)
            .ok_or_else(|| bad("Context basis masters absent"))?;
        let gradient = gradients
            .get(&name)
            .ok_or_else(|| bad("Context basis gradients absent"))?;
        replay_require(
            master.len() == LANES * classes(basis) * 4 && gradient.len() == master.len(),
            "Context constrained construction basis shape mismatch",
        )?;
        let native = master
            .iter()
            .map(|x| legal_code(*x))
            .collect::<Result<Vec<_>>>()?;
        for (index, (&original_master, &g)) in master.iter().zip(gradient).enumerate() {
            replay_require(g.is_finite(), "nonfinite Context construction gradient")?;
            let before = native[index];
            let direction = if g > 0.0 {
                -1
            } else if g < 0.0 {
                1
            } else {
                0
            };
            let after = (before + direction).clamp(-7, 7);
            let status = if direction == 0 {
                "zero_gradient"
            } else if before == after {
                "saturated"
            } else {
                "pending"
            };
            let predicted_delta = if before == after {
                0.0
            } else {
                f64::from(g) * (f64::from(after) * 0.25 - f64::from(original_master))
            };
            replay_require(
                before == after || predicted_delta < 0.0,
                "signed Context code move does not decrease actual linear surrogate",
            )?;
            ordered.push((
                basis,
                Trial {
                    name: name.clone(),
                    index,
                    before,
                    after,
                    gradient: g,
                    original_master,
                    predicted_delta,
                    status: status.into(),
                    affected_events: 0,
                    checked_events: 0,
                    blocking_event: None,
                    blocking_winner: None,
                    blocking_competitor: None,
                },
            ));
        }
        codes.push(native);
    }
    replay_require(
        ordered.len() == COORDINATES,
        "Context coordinate inventory changed",
    )?;
    ordered.sort_by(|(_, a), (_, b)| {
        a.predicted_delta
            .total_cmp(&b.predicted_delta)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.index.cmp(&b.index))
    });
    let mut protected = events
        .iter()
        .map(Protected::new)
        .collect::<Result<Vec<_>>>()?;
    let mut incidence = vec![Vec::<usize>::new(); 3 * LANES];
    for (i, e) in events.iter().enumerate() {
        incidence[event_family(&e.family) * LANES + e.flat_lane].push(i);
    }
    let mut result = Construction {
        edits: vec![],
        trials: vec![],
        stats: Stats {
            coordinates: COORDINATES,
            protected_events: events.len(),
            ..Stats::default()
        },
        g_dot_actual_delta: 0.0,
        final_scores_sha256: String::new(),
    };
    for (basis, mut trial) in ordered {
        if trial.status == "zero_gradient" {
            result.stats.zero_gradient += 1;
            result.trials.push(trial);
            continue;
        }
        if trial.status == "saturated" {
            result.stats.saturated += 1;
            result.trials.push(trial);
            continue;
        }
        let count = classes(basis);
        let row = trial.index / 4;
        let lane = row / count;
        let class = row % count;
        let offset = row * 4;
        let current: [i8; 4] = codes[basis][offset..offset + 4]
            .try_into()
            .map_err(|_| bad("Context basis row missing"))?;
        let mut candidate = current;
        candidate[trial.index % 4] = trial.after;
        // Re-round the entire factor after earlier accepted coordinates in this row.
        // Rounding individual scalar contributions would give a different compiler.
        let mut deltas = [0i64; 120];
        for (root, delta) in deltas.iter_mut().enumerate() {
            *delta = i64::from(basis_score_q24(candidate, root as u8)?)
                - i64::from(basis_score_q24(current, root as u8)?);
        }
        let affected = &incidence[(basis / 2) * LANES + lane];
        trial.affected_events = affected.len();
        for &event_index in affected {
            let p = &protected[event_index];
            let root = if basis % 2 == 0 { p.own } else { p.neighbor };
            trial.checked_events += 1;
            result.stats.constraint_checks += 1;
            if let Some(rival) = p.blocker(class, deltas[usize::from(root)])? {
                trial.blocking_event = Some(event_index);
                trial.blocking_winner = Some(p.winner as u8);
                trial.blocking_competitor = Some(rival as u8);
                break;
            }
        }
        if trial.blocking_event.is_some() {
            trial.status = "rejected_protected_decision".into();
            result.stats.rejected += 1;
        } else {
            // Apply only after every affected event passes: rejected trials leave no
            // score-cache or coefficient residue, even when the last event blocks.
            for &event_index in affected {
                let p = &mut protected[event_index];
                let root = if basis % 2 == 0 { p.own } else { p.neighbor };
                p.apply(class, deltas[usize::from(root)])?;
            }
            codes[basis][trial.index] = trial.after;
            result.edits.push(np::Edit {
                name: trial.name.clone(),
                index: trial.index,
                before: trial.before,
                after: trial.after,
            });
            result.g_dot_actual_delta += trial.predicted_delta;
            trial.status = "accepted".into();
            result.stats.accepted += 1;
        }
        result.trials.push(trial);
    }
    replay_require(
        protected
            .iter()
            .all(|p| p.scores[p.winner] >= p.rivals[1].0),
        "Context composite construction violated protected decision",
    )?;
    let mut digest = sha2::Sha256::new();
    for event in &protected {
        for score in &event.scores {
            digest.update(score.to_le_bytes());
        }
    }
    result.final_scores_sha256 = format!("{:x}", digest.finalize());
    result.stats.protected_decisions_preserved = true;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::geometric_context::ContextInvocation;
    fn fixture() -> (np::Shadows, np::Shadows) {
        let p = np::BASIS
            .iter()
            .enumerate()
            .map(|(b, s)| {
                (
                    format!("consumer.context.{s}"),
                    vec![0.0; LANES * classes(b) * 4],
                )
            })
            .collect::<np::Shadows>();
        (p.clone(), p)
    }
    fn event(winner: u8, scores: Vec<i64>) -> ContextDecisionEvent {
        ContextDecisionEvent {
            call_index: 0,
            invocation: ContextInvocation::Step,
            family: ContextDecisionFamily::Transition,
            token_id: 0,
            head: 0,
            lane: 0,
            flat_lane: 0,
            own: 1,
            neighbor: 1,
            winner,
            scores_q24: scores,
        }
    }
    #[test]
    fn context_constraint_exact_earliest_ties_and_nonmutating_rejection() -> Result<()> {
        let mut scores = vec![0; 120];
        scores[1] = 5;
        scores[0] = 4;
        let mut p = Protected::new(&event(1, scores))?;
        assert_eq!(p.blocker(0, 1)?, Some(0)); // Equal lower ID wins.
        assert_eq!(p.blocker(2, 5)?, None); // Equal higher ID loses.
        assert_eq!(p.scores[0], 4); // Failed consideration cannot alter a later trial.
        p.apply(2, 5)?;
        assert_eq!(p.blocker(1, -1)?, Some(0));
        assert!(Protected::new(&event(1, vec![0; 120])).is_err());
        Ok(())
    }
    #[test]
    fn context_constraint_shared_self_neighbor_collision_and_last_visit_rollback() -> Result<()> {
        let (parent, mut g) = fixture();
        g.get_mut("consumer.context.self_transition").unwrap()[4] = -2.0;
        g.get_mut("consumer.context.neighbor_transition").unwrap()[4] = -1.0;
        let quantum = 1i64 << 22;
        let mut first = vec![0; 120];
        first[0] = 3 * quantum;
        let mut last = vec![0; 120];
        last[0] = quantum + quantum / 2;
        let r = construct(&parent, &g, &[event(0, first), event(0, last)])?;
        assert_eq!(r.edits.len(), 1);
        assert_eq!(r.edits[0].name, "consumer.context.self_transition");
        let rejected = r
            .trials
            .iter()
            .find(|t| t.status == "rejected_protected_decision")
            .unwrap();
        assert_eq!(rejected.blocking_event, Some(1));
        assert_eq!(rejected.checked_events, 2);
        assert_eq!(r.stats.accepted, 1);
        assert_eq!(r.stats.rejected, 1);
        assert!(r.stats.protected_decisions_preserved);
        // Repeating this pure construction cannot inherit tentative state.
        assert_eq!(
            construct(
                &parent,
                &g,
                &[event(0, {
                    let mut scores = vec![0; 120];
                    scores[0] = 10 * quantum;
                    scores
                })]
            )?
            .edits
            .len(),
            2
        );
        assert!(parent.values().flatten().all(|x| x.to_bits() == 0));
        Ok(())
    }
    #[test]
    fn context_constraint_actual_offgrid_cost_saturation_and_all_tensor_scope() -> Result<()> {
        let (mut p, mut g) = fixture();
        // A less steep gradient ranks first because its actual off-grid displacement
        // is larger. Untouched off-grid/signed-zero coordinates retain their bits.
        p.get_mut("consumer.context.self_root").unwrap()[0] = 0.124;
        g.get_mut("consumer.context.self_root").unwrap()[0] = 1.0;
        p.get_mut("consumer.context.neighbor_root").unwrap()[0] = -0.124;
        g.get_mut("consumer.context.neighbor_root").unwrap()[0] = 2.0;
        p.get_mut("consumer.context.self_category").unwrap()[0] = 1.75;
        g.get_mut("consumer.context.self_category").unwrap()[0] = -100.0;
        p.get_mut("consumer.context.neighbor_category").unwrap()[0] = -0.0;
        p.get_mut("consumer.context.neighbor_category").unwrap()[1] = 0.123;
        let r = construct(&p, &g, &[event(0, vec![0; 120])])?;
        assert_eq!(r.trials[0].name, "consumer.context.self_root");
        assert_eq!(r.edits.len(), 2);
        assert_eq!(r.stats.saturated, 1);
        assert_eq!(r.trials.len(), COORDINATES);
        let edited = np::edited(&p, &r.edits)?;
        assert_eq!(
            edited["consumer.context.neighbor_category"][0].to_bits(),
            (-0.0f32).to_bits()
        );
        assert_eq!(
            edited["consumer.context.neighbor_category"][1].to_bits(),
            0.123f32.to_bits()
        );
        let independently_summed: f64 = r
            .edits
            .iter()
            .map(|e| {
                f64::from(g[&e.name][e.index])
                    * (f64::from(edited[&e.name][e.index]) - f64::from(p[&e.name][e.index]))
            })
            .sum();
        assert_eq!(r.g_dot_actual_delta, independently_summed);
        Ok(())
    }
    #[test]
    fn context_constraint_global_ties_are_name_then_coordinate_and_invalid_inputs_fail(
    ) -> Result<()> {
        let (p, mut g) = fixture();
        g.get_mut("consumer.context.self_root").unwrap()[0] = -1.0;
        g.get_mut("consumer.context.neighbor_root").unwrap()[1] = -1.0;
        g.get_mut("consumer.context.neighbor_root").unwrap()[0] = -1.0;
        let events = [event(0, vec![0; 120])];
        let result = construct(&p, &g, &events)?;
        assert_eq!(
            result
                .edits
                .iter()
                .map(|e| (e.name.as_str(), e.index))
                .collect::<Vec<_>>(),
            vec![
                ("consumer.context.neighbor_root", 0),
                ("consumer.context.neighbor_root", 1),
                ("consumer.context.self_root", 0)
            ]
        );
        assert!(construct(&p, &g, &[]).is_err());
        g.get_mut("consumer.context.self_root").unwrap()[0] = f32::NAN;
        assert!(construct(&p, &g, &events).is_err());
        Ok(())
    }
    #[test]
    fn context_constraint_rounds_whole_updated_factor_not_scalar_summands() -> Result<()> {
        // Find an actual canonical Q25 root where per-coordinate rounding differs
        // from the compiler's whole-factor rule; force the one-unit distinction to
        // decide preservation. This catches a plausible incremental-cache shortcut.
        let mut witness = None;
        'roots: for root in 0u8..120 {
            for d in 0..4 {
                for e in (d + 1)..4 {
                    for sign in [-1, 1] {
                        let mut a = [0i8; 4];
                        a[d] = 1;
                        let mut b = [0i8; 4];
                        b[e] = sign;
                        let mut ab = a;
                        ab[e] = sign;
                        let (sa, sb, sab) = (
                            basis_score_q24(a, root)?,
                            basis_score_q24(b, root)?,
                            basis_score_q24(ab, root)?,
                        );
                        if sa + sb != sab && sab - sa > 0 && sb >= 0 {
                            witness = Some((root, d, e, sign, sa, sb, sab));
                            break 'roots;
                        }
                    }
                }
            }
        }
        let (root, d, e, sign, sa, sb, sab) =
            witness.ok_or_else(|| bad("canonical rounding witness absent"))?;
        let (mut p, mut g) = fixture();
        p.get_mut("consumer.context.self_transition").unwrap()[4 + d] = 0.25;
        g.get_mut("consumer.context.self_transition").unwrap()[4 + e] = -f32::from(sign);
        let exact = i64::from(sab - sa);
        let rounded_scalar = i64::from(sb);
        let threshold = exact.min(rounded_scalar);
        let mut scores = vec![-100_000_000; 120];
        scores[0] = 0;
        scores[1] = -threshold;
        let mut ev = event(0, scores);
        ev.own = root;
        let result = construct(&p, &g, &[ev])?;
        assert_eq!(result.edits.len(), usize::from(exact <= threshold));
        assert_ne!(exact <= threshold, rounded_scalar <= threshold);
        Ok(())
    }
}
