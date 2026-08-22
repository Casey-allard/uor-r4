//! #845 binding cheap instrument — the `bounded-breadth-first` failure-surface
//! probe authorized by the maintainer decision of 2026-08-22 on #845.
//!
//! Maps where the lowered non-geometric baseline (RF-33, `bounded-breadth-first`)
//! actually degrades: five families × H ∈ {1, 2, 4, 8, 12, 16} at the frozen
//! `PlanBudget`, plus a tightened frontier/expansion ladder at H = 8. All four
//! #843 nulls are measured alongside, so an Amendment A2(b) correctness cell is
//! admitted only where the strongest of {baseline, nulls} sits at or below
//! 1 − δ_min — otherwise the cell would recreate the saturated-baseline trap
//! this probe exists to prevent.
//!
//! Certifier-instrument / off-serving-path. Teacher-free, fixture-free,
//! deterministic. The full grid is `#[ignore]`d because it is a measurement;
//! the default test asserts the probe itself is non-vacuous.
//!
//! Helper construction mirrors `compositional_planning_measurement_843.rs`
//! (the #843 increment-6 harness), which is the protocol A2 inherits.

use std::collections::BTreeMap;

use uor_r4_graph_compiler::compositional_planning as cp;
use uor_r4_graph_compiler::semantic_state::Action;
use uor_r4_graph_compiler::semantic_transitions as st;
use uor_r4_graph_format::plan::{CompareOp, EffectDelta, PreconditionMask, SlotVec};
use uor_r4_graph_format::plan_sections::{
    build_predicate_set, build_rule_table, build_schema, PackedRule, PlanSchema, PredicateSet,
    RuleTable,
};
use uor_r4_graph_runtime::plan::{
    plan, PlanBudget, PlanOutcome, PlanQuery, PlanScratch, PlanStrategy,
};

/// Frozen effect floor (#844 §2.5).
const DELTA_MIN: f64 = 0.05;
/// Frozen sample size per held-out cell per horizon (#844 §2.5).
const N_PER_CELL: usize = 512;
/// The probed horizons: the frozen grid plus the extended-capacity points.
const PROBE_HORIZONS: [usize; 6] = [1, 2, 4, 8, 12, 16];

const FAMILIES: [cp::TaskFamily; 5] = [
    cp::TaskFamily::GraphNavigation,
    cp::TaskFamily::SymbolicTransformation,
    cp::TaskFamily::ConstraintSatisfaction,
    cp::TaskFamily::MultiHopEvidence,
    cp::TaskFamily::CounterfactualIntervention,
];

// ---------------------------------------------------------------------------
// The joint split (#844 §2.2): a held-out instance is in the high half of
// EVERY seed-varied axis, a fitting instance in the low half of every axis.
// ---------------------------------------------------------------------------

fn axis_digits(seed: u64) -> [u64; 4] {
    let c = cp::AXIS_CARDINALITY;
    [
        seed % c,
        (seed / c) % c,
        (seed / (c * c)) % c,
        (seed / (c * c * c)) % c,
    ]
}

fn joint_held_out(seed: u64) -> bool {
    axis_digits(seed)
        .iter()
        .all(|d| *d >= cp::AXIS_CARDINALITY / 2)
}

fn joint_fitting(seed: u64) -> bool {
    axis_digits(seed)
        .iter()
        .all(|d| *d < cp::AXIS_CARDINALITY / 2)
}

fn seeds(held_out: bool, count: usize) -> Vec<u64> {
    let mut out = Vec::with_capacity(count);
    let mut seed = 0u64;
    while out.len() < count {
        let keep = if held_out {
            joint_held_out(seed)
        } else {
            joint_fitting(seed)
        };
        if keep {
            out.push(seed);
        }
        seed += 1;
    }
    out
}

// ---------------------------------------------------------------------------
// Packing: reference instance -> packed sections (the #843 harness protocol)
// ---------------------------------------------------------------------------

fn ints(values: &[f32]) -> Vec<i16> {
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

/// The artifact side of an episode: schema and rule table from the induced
/// rule set alone.
struct Packed {
    schema: Vec<u8>,
    rules: Vec<u8>,
    effects: Vec<Vec<i16>>,
}

/// Pack an induced rule set. `rotate` canonically rotates each operator's
/// effect (the shuffled-state null); `rotate = 0` is the faithful artifact.
fn pack(set: &st::TransitionRuleSet, rotate: usize) -> Option<Packed> {
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

fn predicates_for(task: &cp::TaskInstance) -> Option<Vec<u8>> {
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

fn initial_of(task: &cp::TaskInstance) -> SlotVec {
    SlotVec::from_slice(&ints(&task.initial_state.vector)).unwrap()
}

fn mask_for(packed: &Packed, task: &cp::TaskInstance) -> u64 {
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

/// A plan as a sequence of operator effects, or an honest decline.
type Emission = Option<Vec<Vec<i16>>>;

fn as_actions(task: &cp::TaskInstance, effects: &[Vec<i16>]) -> Option<Vec<Action>> {
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

/// Judge an emitted outcome against the frozen benchmark verifier (correct =
/// valid plan on a solvable instance, or a correct decline on an unsolvable
/// one — the #844 §11.6 reading; on all-solvable cells this is exactly the
/// frozen valid-plan rate).
fn outcome_correct(task: &cp::TaskInstance, emitted: &Emission) -> bool {
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

// ---------------------------------------------------------------------------
// The deployed arm under an explicit budget, with counters
// ---------------------------------------------------------------------------

type Episode = (Emission, uor_r4_graph_runtime::plan::PlanCounters);

fn production_measured(
    strategy: PlanStrategy,
    packed: &Packed,
    task: &cp::TaskInstance,
    budget: PlanBudget,
    scratch: &mut PlanScratch,
) -> Option<Episode> {
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

// ---------------------------------------------------------------------------
// The four #843 nulls (budget-independent replays plus the shuffled artifact)
// ---------------------------------------------------------------------------

struct Fitted {
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

fn gold_effects(task: &cp::TaskInstance) -> Emission {
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

fn fit_nulls(family: cp::TaskFamily, horizon: usize) -> Fitted {
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
fn retrieval_only(fitted: &Fitted, task: &cp::TaskInstance) -> Emission {
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

/// N2 direct-continuation: greedy one-step descent on goal distance, no
/// lookahead, no backtracking, no honest decline.
fn direct_continuation(task: &cp::TaskInstance, horizon: usize) -> Emission {
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

/// N3 memorized-trajectory: replay by structural key with a modal fallback.
fn memorized(fitted: &Fitted, task: &cp::TaskInstance) -> Emission {
    match fitted.by_structure.get(&structure_key(task)) {
        Some(plan) => plan.clone(),
        None => fitted.modal.clone(),
    }
}

// ---------------------------------------------------------------------------
// Generator viability and induction
// ---------------------------------------------------------------------------

/// `cp::generate` places the goal on the BFS layer at exactly `depth`; a
/// (family, topology, horizon) whose layer is empty panics. The probe records
/// that as generator-unavailable rather than crashing: an extended-horizon
/// cell the generator cannot populate is excluded by evidence.
fn try_generate(family: cp::TaskFamily, seed: u64, horizon: usize) -> Option<cp::TaskInstance> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cp::generate(family, seed, horizon)
    }))
    .ok()
}

/// Induce the artifact-side rule set from the fitting half of the joint split
/// (the #843 harness protocol: N/4 fitting seeds, held-out topologies armed).
fn induce_for(family: cp::TaskFamily, horizon: usize) -> Option<st::TransitionRuleSet> {
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

// ---------------------------------------------------------------------------
// The probe grid
// ---------------------------------------------------------------------------

/// One probed (family × horizon × budget) cell.
#[derive(Debug, Clone)]
struct ProbeCell {
    family: &'static str,
    horizon: usize,
    budget_label: String,
    n: usize,
    generator_unavailable: usize,
    arm_correct: usize,
    null_correct: [usize; 4],
    mean_expansions: f64,
    max_expansions: u32,
    mean_candidates: f64,
    mean_table_reads: f64,
}

const NULL_NAMES: [&str; 4] = [
    "retrieval-only",
    "direct-continuation",
    "memorized-trajectory",
    "shuffled-state",
];

impl ProbeCell {
    fn arm_rate(&self) -> f64 {
        if self.n == 0 {
            return 0.0;
        }
        self.arm_correct as f64 / self.n as f64
    }

    fn null_rate(&self, index: usize) -> f64 {
        if self.n == 0 {
            return 0.0;
        }
        self.null_correct[index] as f64 / self.n as f64
    }

    fn strongest(&self) -> (usize, f64) {
        let index = (0..4).max_by_key(|i| self.null_correct[*i]).unwrap();
        (index, self.null_rate(index))
    }

    /// A2(b) admissibility: the strongest of {baseline, nulls} must sit at or
    /// below 1 − δ_min, or geometry has nothing measurable to beat.
    fn admissible(&self) -> bool {
        if self.n == 0 {
            return false;
        }
        let (_, strongest_null) = self.strongest();
        self.arm_rate().max(strongest_null) <= 1.0 - DELTA_MIN
    }
}

fn frozen_with(horizon: usize) -> PlanBudget {
    PlanBudget {
        horizon: horizon as u8,
        ..PlanBudget::frozen()
    }
}

/// The tightened ladder at the anchor horizon: frontier alone, then
/// expansions alone, each stepped down from the frozen value.
fn ladder(horizon: usize) -> Vec<(String, PlanBudget)> {
    let mut out = Vec::new();
    for frontier in [32u16, 16, 8, 4] {
        out.push((
            format!("frontier-{frontier}"),
            PlanBudget {
                frontier,
                ..frozen_with(horizon)
            },
        ));
    }
    for expansions in [256u32, 128, 64, 32] {
        out.push((
            format!("expansions-{expansions}"),
            PlanBudget {
                max_expansions: expansions,
                ..frozen_with(horizon)
            },
        ));
    }
    out
}

/// Everything a (family, horizon) pair fits once and every budget reuses:
/// the packed faithful artifact, the shuffled artifact, and the fitted nulls.
struct FittedCell {
    packed: Packed,
    shuffled: Packed,
    fitted: Fitted,
}

fn fit_cell(family: cp::TaskFamily, horizon: usize) -> Option<FittedCell> {
    let set = induce_for(family, horizon)?;
    let packed = pack(&set, 0)?;
    let shuffled = pack(&set, 1)?;
    let fitted = fit_nulls(family, horizon);
    Some(FittedCell {
        packed,
        shuffled,
        fitted,
    })
}

fn probe_cell(
    family: cp::TaskFamily,
    horizon: usize,
    budget_label: &str,
    budget: PlanBudget,
    cellfit: &FittedCell,
    scratch: &mut PlanScratch,
) -> ProbeCell {
    let mut cell = ProbeCell {
        family: family.label(),
        horizon,
        budget_label: budget_label.to_string(),
        n: 0,
        generator_unavailable: 0,
        arm_correct: 0,
        null_correct: [0; 4],
        mean_expansions: 0.0,
        max_expansions: 0,
        mean_candidates: 0.0,
        mean_table_reads: 0.0,
    };
    let mut expansions = 0u64;
    let mut candidates = 0u64;
    let mut table_reads = 0u64;
    for seed in seeds(true, N_PER_CELL) {
        let Some(task) = try_generate(family, seed, horizon) else {
            cell.generator_unavailable += 1;
            continue;
        };
        cell.n += 1;
        let (emitted, counters) = production_measured(
            PlanStrategy::BreadthFirst,
            &cellfit.packed,
            &task,
            budget,
            scratch,
        )
        .unwrap_or_default();
        if outcome_correct(&task, &emitted) {
            cell.arm_correct += 1;
        }
        expansions += u64::from(counters.expansions);
        candidates += u64::from(counters.candidates);
        table_reads += u64::from(counters.table_reads);
        cell.max_expansions = cell.max_expansions.max(counters.expansions);
        let shuffled_emission = production_measured(
            PlanStrategy::BreadthFirst,
            &cellfit.shuffled,
            &task,
            budget,
            scratch,
        )
        .and_then(|(plan, _)| plan);
        let nulls: [Emission; 4] = [
            retrieval_only(&cellfit.fitted, &task),
            direct_continuation(&task, horizon),
            memorized(&cellfit.fitted, &task),
            shuffled_emission,
        ];
        for (index, null) in nulls.iter().enumerate() {
            if outcome_correct(&task, null) {
                cell.null_correct[index] += 1;
            }
        }
    }
    if cell.n > 0 {
        cell.mean_expansions = expansions as f64 / cell.n as f64;
        cell.mean_candidates = candidates as f64 / cell.n as f64;
        cell.mean_table_reads = table_reads as f64 / cell.n as f64;
    }
    cell
}

fn print_cell(cell: &ProbeCell) {
    let (strongest_index, strongest_rate) = cell.strongest();
    println!(
        "PROBE | {:<26} | H={:<2} | {:<14} | n={} unavail={} | arm={:.4} | strongest={}@{:.4} | nulls r={:.4} d={:.4} m={:.4} s={:.4} | exp mean={:.1} max={} | cand mean={:.1} | reads mean={:.1} | {}",
        cell.family,
        cell.horizon,
        cell.budget_label,
        cell.n,
        cell.generator_unavailable,
        cell.arm_rate(),
        NULL_NAMES[strongest_index],
        strongest_rate,
        cell.null_rate(0),
        cell.null_rate(1),
        cell.null_rate(2),
        cell.null_rate(3),
        cell.mean_expansions,
        cell.max_expansions,
        cell.mean_candidates,
        cell.mean_table_reads,
        if cell.admissible() { "ADMISSIBLE" } else { "saturated" },
    );
}

// ---------------------------------------------------------------------------
// The probe itself
// ---------------------------------------------------------------------------

/// The full failure surface. `--ignored` because it is a measurement, not a
/// gate; run with `-- --ignored --nocapture` and read the PROBE lines.
#[test]
#[ignore = "measurement probe: run explicitly with --ignored --nocapture"]
fn breadth_first_failure_surface() {
    let quiet = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut scratch = Box::new(PlanScratch::new());
    let mut cells: Vec<ProbeCell> = Vec::new();
    for family in FAMILIES {
        for horizon in PROBE_HORIZONS {
            let Some(cellfit) = fit_cell(family, horizon) else {
                println!(
                    "PROBE | {:<26} | H={:<2} | GENERATOR-UNAVAILABLE (fitting half)",
                    family.label(),
                    horizon
                );
                continue;
            };
            let cell = probe_cell(
                family,
                horizon,
                "frozen",
                frozen_with(horizon),
                &cellfit,
                &mut scratch,
            );
            print_cell(&cell);
            cells.push(cell);
            if horizon == 8 {
                for (label, budget) in ladder(horizon) {
                    let cell = probe_cell(family, horizon, &label, budget, &cellfit, &mut scratch);
                    print_cell(&cell);
                    cells.push(cell);
                }
            }
        }
    }
    std::panic::set_hook(quiet);
    let admissible: Vec<&ProbeCell> = cells.iter().filter(|c| c.admissible()).collect();
    println!(
        "PROBE-SUMMARY | cells={} admissible={} | delta_min={} n_per_cell={}",
        cells.len(),
        admissible.len(),
        DELTA_MIN,
        N_PER_CELL
    );
    for cell in &admissible {
        println!(
            "PROBE-ADMISSIBLE | {} H={} {} arm={:.4} strongest_null={:.4}",
            cell.family,
            cell.horizon,
            cell.budget_label,
            cell.arm_rate(),
            cell.strongest().1
        );
    }
}

// ---------------------------------------------------------------------------
// Non-vacuity gates — run by default, so the probe cannot silently rot
// ---------------------------------------------------------------------------

/// The probe's machinery is real: induction packs, the deployed planner plans
/// on the packed artifact, and the verifier accepts the outcome.
#[test]
fn the_probe_packs_and_plans() {
    let mut scratch = Box::new(PlanScratch::new());
    let cellfit = fit_cell(cp::TaskFamily::GraphNavigation, 8).expect("fit succeeds at H=8");
    let task = cp::generate(cp::TaskFamily::GraphNavigation, 4, 8);
    let (emitted, counters) = production_measured(
        PlanStrategy::BreadthFirst,
        &cellfit.packed,
        &task,
        frozen_with(8),
        &mut scratch,
    )
    .expect("the packed artifact parses and plans");
    assert!(
        outcome_correct(&task, &emitted),
        "the deployed planner did not produce an accepted outcome"
    );
    assert!(counters.expansions > 0, "the planner did no work");
}

/// The admissibility judge can fire in both directions: a trap family at H=8
/// has direct-continuation failures (so headroom can exist), and a saturated
/// null is judged inadmissible (so the trap this probe exists to prevent is
/// actually prevented).
#[test]
fn the_admissibility_judge_can_fire_and_can_fail() {
    let mut dc_failures = 0usize;
    for seed in seeds(true, 64) {
        let task = cp::generate(cp::TaskFamily::GraphNavigation, seed, 8);
        let emission = direct_continuation(&task, 8);
        if !outcome_correct(&task, &emission) {
            dc_failures += 1;
        }
    }
    assert!(
        dc_failures > 0,
        "direct-continuation never fails on the trap family — the probe cannot separate anything"
    );
    let saturated = ProbeCell {
        family: "synthetic",
        horizon: 8,
        budget_label: "synthetic".to_string(),
        n: 512,
        generator_unavailable: 0,
        arm_correct: 512,
        null_correct: [512, 0, 0, 0],
        mean_expansions: 0.0,
        max_expansions: 0,
        mean_candidates: 0.0,
        mean_table_reads: 0.0,
    };
    assert!(!saturated.admissible(), "a saturated cell must be excluded");
    let open = ProbeCell {
        arm_correct: 384,
        null_correct: [256, 128, 64, 32],
        ..saturated
    };
    assert!(
        open.admissible(),
        "a cell with real headroom must be admitted"
    );
}
