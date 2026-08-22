//! Shared #845 episode machinery: benchmark instance -> packed sections ->
//! one deployed or reference planning episode, judged by the frozen #844
//! verifier. The construction mirrors the #843 increment-6 harness protocol
//! (and the frozen probe instrument `geometry_probe_845.rs`).
#![allow(dead_code)]

use uor_r4_graph_compiler::compositional_planning as cp;
use uor_r4_graph_compiler::semantic_state::Action;
use uor_r4_graph_compiler::semantic_transitions as st;
use uor_r4_graph_format::plan::{CompareOp, EffectDelta, PreconditionMask, SlotVec};
use uor_r4_graph_format::plan_sections::{
    build_predicate_set, build_rule_table, build_schema, PackedRule, PlanSchema, PredicateSet,
    RuleTable,
};
use uor_r4_graph_runtime::plan::{
    plan, PlanBudget, PlanCounters, PlanOutcome, PlanQuery, PlanScratch, PlanStrategy,
};

use super::arms::{RemainingObservation, Transition};
use super::ordering::{plan_reference, RefOutcome, RefQuery, RefScratch, Scorer, SeamMode};

/// Frozen sample size per held-out cell per horizon (#844 §2.5).
pub const N_PER_CELL: usize = 512;

pub const FAMILIES: [cp::TaskFamily; 5] = [
    cp::TaskFamily::GraphNavigation,
    cp::TaskFamily::SymbolicTransformation,
    cp::TaskFamily::ConstraintSatisfaction,
    cp::TaskFamily::MultiHopEvidence,
    cp::TaskFamily::CounterfactualIntervention,
];

/// The three A2(b) separating families.
pub const SEPARATING: [cp::TaskFamily; 3] = [
    cp::TaskFamily::GraphNavigation,
    cp::TaskFamily::ConstraintSatisfaction,
    cp::TaskFamily::MultiHopEvidence,
];

fn axis_digits(seed: u64) -> [u64; 4] {
    let c = cp::AXIS_CARDINALITY;
    [
        seed % c,
        (seed / c) % c,
        (seed / (c * c)) % c,
        (seed / (c * c * c)) % c,
    ]
}

/// Joint-split seed walk (#844 §2.2): held-out = high half of every axis,
/// fitting = low half of every axis.
pub fn seeds(held_out: bool, count: usize) -> Vec<u64> {
    let half = cp::AXIS_CARDINALITY / 2;
    let mut out = Vec::with_capacity(count);
    let mut seed = 0u64;
    while out.len() < count {
        let keep = if held_out {
            axis_digits(seed).iter().all(|d| *d >= half)
        } else {
            axis_digits(seed).iter().all(|d| *d < half)
        };
        if keep {
            out.push(seed);
        }
        seed += 1;
    }
    out
}

/// Generator viability guard (`cp::generate` panics when a requested BFS layer
/// is empty); unavailable instances are recorded, never crashed on.
pub fn try_generate(family: cp::TaskFamily, seed: u64, horizon: usize) -> Option<cp::TaskInstance> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cp::generate(family, seed, horizon)
    }))
    .ok()
}

pub fn ints(values: &[f32]) -> Vec<i16> {
    values.iter().map(|v| v.round() as i16).collect()
}

fn band_code(band: cp::ConfidenceBand) -> u8 {
    match band {
        cp::ConfidenceBand::None => 0,
        cp::ConfidenceBand::Low => 1,
        cp::ConfidenceBand::Medium => 2,
        cp::ConfidenceBand::High => 3,
    }
}

fn cell_predicate(values: &[i16]) -> PreconditionMask {
    let mut predicate = PreconditionMask::unconditional();
    for (slot, value) in values.iter().enumerate() {
        predicate = predicate.reading(slot, CompareOp::Equal, *value).unwrap();
    }
    predicate
}

/// The artifact side of an episode.
pub struct Packed {
    pub schema: Vec<u8>,
    pub rules: Vec<u8>,
    pub effects: Vec<Vec<i16>>,
}

/// Pack an induced rule set; `rotate` is the shuffled-state null (0 = faithful).
pub fn pack(set: &st::TransitionRuleSet, rotate: usize) -> Option<Packed> {
    let mut effects: Vec<Vec<i16>> = set
        .rules
        .iter()
        .map(|r| r.effect.as_slice().to_vec())
        .collect();
    effects.sort();
    effects.dedup();
    if effects.is_empty() {
        return None;
    }
    let vocabulary: Vec<EffectDelta> = effects
        .iter()
        .map(|e| EffectDelta::from_slice(e).unwrap())
        .collect();
    let schema = build_schema(2, &vocabulary, (1, 4, 16))?;
    let parsed = PlanSchema::parse(&schema).ok()?;
    let mut rules = Vec::new();
    for rule in &set.rules {
        let effect = rule.effect.as_slice().to_vec();
        let operator = effects.iter().position(|e| *e == effect)?;
        let carried = (operator + rotate) % effects.len();
        rules.push(PackedRule {
            operator: operator as u16,
            precondition: rule.precondition,
            effect: parsed.operator(carried)?,
            support: rule.support,
            band: band_code(rule.band),
        });
    }
    rules.sort_by_key(|r| (r.operator, r.effect.as_slice().to_vec()));
    rules.dedup_by_key(|r| (r.operator, r.effect.as_slice().to_vec()));
    let rules = build_rule_table(2, effects.len() as u16, &rules)?;
    Some(Packed {
        schema,
        rules,
        effects,
    })
}

/// Induce the artifact-side rule set from the fitting half (joint split).
pub fn induce_for(family: cp::TaskFamily, horizon: usize) -> Option<st::TransitionRuleSet> {
    let mut observations = Vec::new();
    for seed in seeds(false, N_PER_CELL / 4) {
        let task = try_generate(family, seed, horizon)?;
        observations.extend(st::observe(&task));
    }
    let held_out: std::collections::BTreeSet<u8> = (0..cp::AXIS_CARDINALITY)
        .filter(|d| *d >= cp::AXIS_CARDINALITY / 2)
        .map(|d| d as u8)
        .collect();
    let policy = st::SplitPolicy {
        held_out_topologies: held_out,
        sealed_topologies: std::collections::BTreeSet::new(),
    };
    st::induce(&observations, &policy).induced().cloned()
}

pub fn predicates_for(task: &cp::TaskInstance) -> Option<Vec<u8>> {
    let goal = cell_predicate(&ints(&task.goal.target_region.center));
    let mut constraints: Vec<PreconditionMask> = task
        .constraints
        .iter()
        .map(|c| cell_predicate(&ints(&c.forbidden_region.center)))
        .collect();
    constraints.sort_by_key(|c| (0..2).map(|s| c.bound(s)).collect::<Vec<_>>());
    constraints.dedup_by_key(|c| (0..2).map(|s| c.bound(s)).collect::<Vec<_>>());
    build_predicate_set(2, &[goal], &constraints)
}

pub fn initial_of(task: &cp::TaskInstance) -> SlotVec {
    SlotVec::from_slice(&ints(&task.initial_state.vector)).unwrap()
}

pub fn goal_center(task: &cp::TaskInstance) -> (i16, i16) {
    let center = ints(&task.goal.target_region.center);
    (
        center.first().copied().unwrap_or(0),
        center.get(1).copied().unwrap_or(0),
    )
}

pub fn mask_for(packed: &Packed, task: &cp::TaskInstance) -> u64 {
    let mut mask = 0u64;
    for action in &task.actions {
        let effect = ints(&action.delta_vector);
        if let Some(index) = packed.effects.iter().position(|e| *e == effect) {
            if index < 64 {
                mask |= 1u64 << index;
            }
        }
    }
    mask
}

/// A plan as operator-effect steps, or an honest decline.
pub type Emission = Option<Vec<Vec<i16>>>;

pub fn as_actions(task: &cp::TaskInstance, effects: &[Vec<i16>]) -> Option<Vec<Action>> {
    effects
        .iter()
        .map(|effect| {
            task.actions
                .iter()
                .find(|a| ints(&a.delta_vector) == *effect)
                .cloned()
        })
        .collect()
}

/// Judge an emission against the frozen #844 verifier (correct-outcome rate,
/// the #844 §11.6 reading).
pub fn outcome_correct(task: &cp::TaskInstance, emitted: &Emission) -> bool {
    let unsolvable = task.gold.decline.is_some();
    match (emitted, unsolvable) {
        (None, true) => true,
        (None, false) => false,
        (Some(_), true) => false,
        (Some(effects), false) => match as_actions(task, effects) {
            Some(actions) => cp::verify_submission(task, &actions) == cp::WitnessVerdict::Valid,
            None => false,
        },
    }
}

/// One deployed episode under an explicit budget.
pub fn run_deployed(
    strategy: PlanStrategy,
    packed: &Packed,
    task: &cp::TaskInstance,
    budget: PlanBudget,
    scratch: &mut PlanScratch,
) -> Option<(Emission, PlanCounters)> {
    let schema = PlanSchema::parse(&packed.schema).ok()?;
    let rules = RuleTable::parse(&packed.rules, &schema).ok()?;
    let predicate_bytes = predicates_for(task)?;
    let predicates = PredicateSet::parse(&predicate_bytes, &schema).ok()?;
    let result = plan(
        &PlanQuery {
            strategy,
            schema: &schema,
            rules: &rules,
            predicates: &predicates,
            initial: initial_of(task),
            available: mask_for(packed, task),
            budget,
        },
        scratch,
    );
    let emitted = match result.outcome {
        PlanOutcome::Plan { steps } => Some(
            (0..steps)
                .filter_map(|i| {
                    scratch
                        .path_step(i)
                        .map(|(e, _, _, _)| e.as_slice().to_vec())
                })
                .collect(),
        ),
        PlanOutcome::Declined(_) => None,
    };
    Some((emitted, result.counters))
}

/// One reference episode under the same budget and artifact, with the
/// retention seam opened to `scorer`.
pub fn run_reference(
    packed: &Packed,
    task: &cp::TaskInstance,
    budget: PlanBudget,
    mode: SeamMode,
    scorer: &mut dyn Scorer,
    scratch: &mut RefScratch,
) -> Option<(Emission, PlanCounters)> {
    let schema = PlanSchema::parse(&packed.schema).ok()?;
    let rules = RuleTable::parse(&packed.rules, &schema).ok()?;
    let predicate_bytes = predicates_for(task)?;
    let predicates = PredicateSet::parse(&predicate_bytes, &schema).ok()?;
    let query = RefQuery {
        schema: &schema,
        rules: &rules,
        predicates: &predicates,
        initial: initial_of(task),
        available: mask_for(packed, task),
        horizon: budget.horizon,
        frontier: budget.frontier,
        max_expansions: budget.max_expansions,
        max_candidates: budget.max_candidates,
        max_table_reads: budget.max_table_reads,
    };
    let result = plan_reference(&query, scratch, mode, scorer);
    let emitted = match result.outcome {
        RefOutcome::Plan(path) => Some(
            path.iter()
                .map(|(effect, _, _, _)| effect.as_slice().to_vec())
                .collect(),
        ),
        RefOutcome::Declined(_) => None,
    };
    Some((emitted, result.counters))
}

/// Gold-path observations for the fitted controls: (state, goal, remaining
/// steps) triples and (state, successor) transitions from the fitting half.
pub struct FittingObservations {
    pub remaining: Vec<RemainingObservation>,
    pub transitions: Vec<Transition>,
}

pub fn fitting_observations(family: cp::TaskFamily, horizon: usize) -> FittingObservations {
    let mut remaining = Vec::new();
    let mut transitions = Vec::new();
    for seed in seeds(false, N_PER_CELL / 4) {
        let Some(task) = try_generate(family, seed, horizon) else {
            continue;
        };
        if task.gold.decline.is_some() {
            continue;
        }
        let goal = goal_center(&task);
        let mut state = {
            let v = ints(&task.initial_state.vector);
            (
                v.first().copied().unwrap_or(0),
                v.get(1).copied().unwrap_or(0),
            )
        };
        let steps = task.gold.chosen_path.len();
        for (index, action) in task.gold.chosen_path.iter().enumerate() {
            let left = (steps - index).min(254) as u8;
            remaining.push((state, goal, left));
            let delta = ints(&action.delta_vector);
            let next = (
                state.0.saturating_add(delta.first().copied().unwrap_or(0)),
                state.1.saturating_add(delta.get(1).copied().unwrap_or(0)),
            );
            transitions.push((state, next));
            state = next;
        }
        remaining.push((state, goal, 0));
    }
    FittingObservations {
        remaining,
        transitions,
    }
}

// ---------------------------------------------------------------------------
// The four #843 nulls (harness protocol, ported unchanged)
// ---------------------------------------------------------------------------

use std::collections::BTreeMap;

/// What the replay nulls learned from the fitting half.
pub struct Fitted {
    by_structure: BTreeMap<String, Emission>,
    by_goal: Vec<(Vec<i16>, Emission)>,
    modal: Emission,
}

fn structure_key(task: &cp::TaskInstance) -> String {
    let mut forbidden: Vec<Vec<i16>> = task
        .constraints
        .iter()
        .map(|c| ints(&c.forbidden_region.center))
        .collect();
    forbidden.sort();
    let effects: Vec<Vec<i16>> = task.actions.iter().map(|a| ints(&a.delta_vector)).collect();
    format!(
        "{}|{:?}|{:?}|{:?}",
        task.family.label(),
        ints(&task.goal.target_region.center),
        forbidden,
        effects
    )
}

pub fn gold_effects(task: &cp::TaskInstance) -> Emission {
    if task.gold.decline.is_some() {
        return None;
    }
    Some(
        task.gold
            .chosen_path
            .iter()
            .map(|a| ints(&a.delta_vector))
            .collect(),
    )
}

pub fn fit_nulls(family: cp::TaskFamily, horizon: usize) -> Fitted {
    let mut by_structure = BTreeMap::new();
    let mut by_goal = Vec::new();
    let mut frequency: BTreeMap<Emission, usize> = BTreeMap::new();
    for seed in seeds(false, N_PER_CELL) {
        let Some(task) = try_generate(family, seed, horizon) else {
            continue;
        };
        let emission = gold_effects(&task);
        by_structure
            .entry(structure_key(&task))
            .or_insert_with(|| emission.clone());
        by_goal.push((ints(&task.goal.target_region.center), emission.clone()));
        *frequency.entry(emission).or_default() += 1;
    }
    let modal = frequency
        .iter()
        .max_by_key(|(plan, count)| (**count, std::cmp::Reverse((*plan).clone())))
        .map(|(plan, _)| plan.clone())
        .unwrap_or(None);
    Fitted {
        by_structure,
        by_goal,
        modal,
    }
}

/// N1 retrieval-only: nearest fitting instance by goal displacement.
pub fn retrieval_only(fitted: &Fitted, task: &cp::TaskInstance) -> Emission {
    let goal = ints(&task.goal.target_region.center);
    fitted
        .by_goal
        .iter()
        .min_by_key(|(g, _)| {
            g.iter()
                .zip(goal.iter())
                .map(|(a, b)| i64::from(*a - *b).abs())
                .sum::<i64>()
        })
        .and_then(|(_, plan)| plan.clone())
}

/// N2 direct-continuation: greedy descent on goal distance, no lookahead.
pub fn direct_continuation(task: &cp::TaskInstance, horizon: usize) -> Emission {
    let goal = ints(&task.goal.target_region.center);
    let mut state = ints(&task.initial_state.vector);
    let mut out = Vec::new();
    let distance = |s: &[i16]| -> i64 {
        s.iter()
            .zip(goal.iter())
            .map(|(a, b)| i64::from(*a - *b).abs())
            .sum()
    };
    for _ in 0..horizon {
        if distance(&state) == 0 {
            return Some(out);
        }
        let mut best: Option<(i64, Vec<i16>, Vec<i16>)> = None;
        for action in &task.actions {
            let effect = ints(&action.delta_vector);
            let next: Vec<i16> = state
                .iter()
                .zip(effect.iter())
                .map(|(s, d)| s.saturating_add(*d))
                .collect();
            let score = distance(&next);
            let better = match &best {
                None => true,
                Some((current, _, current_effect)) => {
                    score < *current || (score == *current && effect < *current_effect)
                }
            };
            if better {
                best = Some((score, next, effect));
            }
        }
        let (_, next, effect) = best?;
        state = next;
        out.push(effect);
    }
    Some(out)
}

/// N3 memorized-trajectory: structural-key replay with a modal fallback.
pub fn memorized(fitted: &Fitted, task: &cp::TaskInstance) -> Emission {
    match fitted.by_structure.get(&structure_key(task)) {
        Some(plan) => plan.clone(),
        None => fitted.modal.clone(),
    }
}
