//! Equal-budget arm/null measurement on the repaired S4 benchmark — the
//! machine-checked record for #843 increment 6.
//!
//! Frozen contract: `docs/bounded_semantic_transitions_spec_843.md` §8, §9 and
//! §10. Certifier-instrument / off-serving-path: this harness measures the
//! deployed planner, it is not part of it.
//!
//! Three bounded planning arms and four falsifying nulls run over the frozen
//! #844 constitution — held-out valid-plan rate, n = 512 per cell per horizon,
//! H ∈ {1, 2, 4, 8}, δ_min = 0.05, Holm–Bonferroni across the grid — with the
//! horizon-1 cell read as correct-outcome rate per §11.6 of the constitution.
//!
//! The full grid is `#[ignore]`d because it is a measurement, not a gate; the
//! default tests assert the harness itself is non-vacuous, which is the part
//! that must never silently rot.

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
/// Frozen horizon progression (#844 §2.5).
const FROZEN_HORIZONS: [usize; 4] = [1, 2, 4, 8];
/// One-sided 95% normal quantile.
const Z95: f64 = 1.645;

const FAMILIES: [cp::TaskFamily; 5] = [
    cp::TaskFamily::GraphNavigation,
    cp::TaskFamily::SymbolicTransformation,
    cp::TaskFamily::ConstraintSatisfaction,
    cp::TaskFamily::MultiHopEvidence,
    cp::TaskFamily::CounterfactualIntervention,
];

/// The generalization axes a split is taken on. Entity, vocabulary and template
/// are *surface* axes; topology is semantic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Entity,
    Vocabulary,
    Topology,
    Template,
}

impl Axis {
    fn label(self) -> &'static str {
        match self {
            Axis::Entity => "by_entity",
            Axis::Vocabulary => "by_vocabulary",
            Axis::Topology => "by_topology",
            Axis::Template => "by_template",
        }
    }

    /// The axis digit of a seed. The four seed-varied axes are independent
    /// base-`AXIS_CARDINALITY` digits.
    fn digit(self, seed: u64) -> u64 {
        let c = cp::AXIS_CARDINALITY;
        match self {
            Axis::Entity => seed % c,
            Axis::Vocabulary => (seed / c) % c,
            Axis::Topology => (seed / (c * c)) % c,
            Axis::Template => (seed / (c * c * c)) % c,
        }
    }

    fn is_held_out(self, seed: u64) -> bool {
        self.digit(seed) >= cp::AXIS_CARDINALITY / 2
    }
}

/// How fitting and evaluation data are partitioned.
///
/// **`Joint` is the constitution-conformant protocol** (#844 §2.2: "fitting
/// data and evaluation data never share a cell on *any* axis"): a held-out
/// instance is in the high half of **every** seed-varied axis, so it shares no
/// entity, vocabulary, topology or template cell with anything fitted.
///
/// `Isolating(axis)` splits on one axis only and lets the rest range freely, so
/// fitting and held-out data *do* share cells on the untested axes. It is a
/// diagnostic, not the gate: it answers "is the mechanism sensitive to this
/// axis?", and on a purely surface axis the informative answer is that a
/// semantics-keyed control transfers perfectly, because such an axis
/// introduces no semantic novelty for anything to generalize over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Split {
    Joint,
    Isolating(Axis),
}

impl Split {
    fn label(self) -> &'static str {
        match self {
            Split::Joint => "joint-all-axes",
            Split::Isolating(axis) => axis.label(),
        }
    }

    fn is_held_out(self, seed: u64) -> bool {
        match self {
            Split::Joint => [
                Axis::Entity,
                Axis::Vocabulary,
                Axis::Topology,
                Axis::Template,
            ]
            .iter()
            .all(|a| a.is_held_out(seed)),
            Split::Isolating(axis) => axis.is_held_out(seed),
        }
    }

    /// A fitting instance is in the low half of every axis the split governs.
    fn is_fitting(self, seed: u64) -> bool {
        match self {
            Split::Joint => [
                Axis::Entity,
                Axis::Vocabulary,
                Axis::Topology,
                Axis::Template,
            ]
            .iter()
            .all(|a| !a.is_held_out(seed)),
            Split::Isolating(axis) => !axis.is_held_out(seed),
        }
    }
}

fn seeds(split: Split, held_out: bool, count: usize) -> Vec<u64> {
    let mut out = Vec::with_capacity(count);
    let mut seed = 0u64;
    while out.len() < count {
        let keep = if held_out {
            split.is_held_out(seed)
        } else {
            split.is_fitting(seed)
        };
        if keep {
            out.push(seed);
        }
        seed += 1;
    }
    out
}

// ---------------------------------------------------------------------------
// Packing: reference instance -> packed sections
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

/// The artifact side of an episode: the schema and the rule table, both built
/// from the induced rule set alone.
struct Packed {
    schema: Vec<u8>,
    rules: Vec<u8>,
    effects: Vec<Vec<i16>>,
}

/// Pack an induced rule set. `rotate` canonically rotates the effect each
/// operator carries, which is the shuffled-state null: the vocabulary and the
/// task are unchanged, only the correspondence between them is broken.
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
        // The rotation moves each operator's *effect*, so a plan expressed as
        // operator indices no longer means what it meant.
        let carried = (operator + rotate) % effects.len();
        rules.push(PackedRule {
            operator: operator as u16,
            precondition: rule.precondition,
            effect: parsed.operator(carried)?,
            support: rule.support,
            band: band_code(rule.band),
        });
    }
    // A rotation can collide two rules onto one canonical key; drop the
    // duplicates deterministically rather than failing the null.
    rules.sort_by_key(|r| (r.operator, r.effect.as_slice().to_vec()));
    rules.dedup_by_key(|r| (r.operator, r.effect.as_slice().to_vec()));
    let rules = build_rule_table(2, effects.len() as u16, &rules)?;
    Some(Packed {
        schema,
        rules,
        effects,
    })
}

/// The query side of an episode, built from the instance alone.
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

/// Which packed operators this instance offers. The vocabulary is
/// artifact-wide; availability is a property of the query.
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

/// Resolve an effect sequence back into the instance's own operators, so a plan
/// is judged by the #844 verifier rather than by whatever produced it.
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

/// Judge an emitted outcome against the frozen benchmark verifier.
///
/// Correct means a valid plan on a solvable instance, or a correct decline on
/// one with no plan inside the horizon (the §11.6 horizon-1 reading). On every
/// cell whose instances are all solvable this is exactly the frozen valid-plan
/// rate.
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
// Arms and nulls, all under one budget
// ---------------------------------------------------------------------------

/// The frozen equal budget every arm and every null runs under, so a comparison
/// between them is not a comparison of effort.
fn shared_budget(horizon: usize) -> PlanBudget {
    PlanBudget {
        horizon: horizon as u8,
        ..PlanBudget::frozen()
    }
}

/// Run the deployed planner and return its plan as an effect sequence, or
/// `None` for an honest decline.
type Episode = (
    Option<Vec<Vec<i16>>>,
    uor_r4_graph_runtime::plan::PlanCounters,
);

fn production_measured(
    strategy: PlanStrategy,
    packed: &Packed,
    task: &cp::TaskInstance,
    horizon: usize,
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
            budget: shared_budget(horizon),
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

/// The emitted plan alone, for the callers that do not read counters.
fn production(
    strategy: PlanStrategy,
    packed: &Packed,
    task: &cp::TaskInstance,
    horizon: usize,
    scratch: &mut PlanScratch,
) -> Option<Vec<Vec<i16>>> {
    production_measured(strategy, packed, task, horizon, scratch).and_then(|(plan, _)| plan)
}

/// A plan as a sequence of operator effects, or an honest decline.
type Emission = Option<Vec<Vec<i16>>>;

/// What the nulls learned from the fitting half.
struct Fitted {
    by_structure: BTreeMap<String, Emission>,
    by_goal: Vec<(Vec<i16>, Emission)>,
    modal: Emission,
}

/// The semantic content a memorizer can key on: goal, forbidden set and
/// operator effects, with surface names and the generation seed excluded. This
/// is the strongest key available, so the control already sees through entity,
/// vocabulary and template renaming before the split is applied.
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

fn fit_nulls(family: cp::TaskFamily, split: Split, horizon: usize) -> Fitted {
    let mut by_structure = BTreeMap::new();
    let mut by_goal = Vec::new();
    let mut frequency: BTreeMap<Emission, usize> = BTreeMap::new();
    for seed in seeds(split, false, N_PER_CELL) {
        let task = cp::generate(family, seed, horizon);
        // Declines are fitted too, so a control that has seen honest declines
        // can emit them: the nulls are not strawmen.
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

/// N1 retrieval-only: the nearest fitting instance by goal displacement.
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

/// N2 direct-continuation: the next operator the emission path would produce,
/// repeatedly, with no lookahead and no backtracking. Greedy on goal distance.
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
            // Canonical tie-break: lower distance, then lower effect.
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
    if distance(&state) == 0 {
        Some(out)
    } else {
        // A greedy continuation that has not arrived does not decline honestly;
        // it emits what it has, which is what makes it a null.
        Some(out)
    }
}

/// N3 memorized-trajectory: replay by structural key, falling back to the modal
/// fitting emission so the control can still fire off-key.
fn memorized(fitted: &Fitted, task: &cp::TaskInstance) -> Emission {
    match fitted.by_structure.get(&structure_key(task)) {
        Some(plan) => plan.clone(),
        None => fitted.modal.clone(),
    }
}

// ---------------------------------------------------------------------------
// The measurement grid
// ---------------------------------------------------------------------------

/// One (arm × axis × family × horizon) cell.
#[derive(Debug, Clone)]
struct Cell {
    arm: &'static str,
    axis: &'static str,
    family: &'static str,
    horizon: usize,
    n: usize,
    arm_correct: usize,
    strongest_null: &'static str,
    null_correct: usize,
    /// Paired differences, one per instance: arm correct minus null correct.
    mean_difference: f64,
    standard_error: f64,
    lower_bound: f64,
    counters_within_budget: bool,
}

/// One-sided 95% lower confidence bound on a paired difference, by the normal
/// approximation. Deterministic; no bootstrap, no RNG.
fn paired_lower_bound(differences: &[i8]) -> (f64, f64, f64) {
    let n = differences.len();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let sum: f64 = differences.iter().map(|d| f64::from(*d)).sum();
    let mean = sum / n as f64;
    let variance: f64 = differences
        .iter()
        .map(|d| {
            let centred = f64::from(*d) - mean;
            centred * centred
        })
        .sum::<f64>()
        / n as f64;
    let standard_error = (variance / n as f64).sqrt();
    (mean, standard_error, mean - Z95 * standard_error)
}

/// Standard normal upper tail, for the one-sided test of `d <= DELTA_MIN`.
fn upper_tail(z: f64) -> f64 {
    0.5 * libm::erfc(z / core::f64::consts::SQRT_2)
}

/// The p-value for `H0: mean difference <= DELTA_MIN`. A zero standard error
/// means the difference is unanimous, so the p-value is degenerate: 0 when the
/// point estimate clears the floor and 1 when it does not.
fn p_value(cell: &Cell) -> f64 {
    if cell.standard_error == 0.0 {
        return if cell.mean_difference > DELTA_MIN {
            0.0
        } else {
            1.0
        };
    }
    upper_tail((cell.mean_difference - DELTA_MIN) / cell.standard_error)
}

/// Induce the artifact-side rule set from the fitting half of `axis`.
fn induce_for(
    family: cp::TaskFamily,
    split: Split,
    horizon: usize,
) -> Option<st::TransitionRuleSet> {
    let mut observations = Vec::new();
    for seed in seeds(split, false, N_PER_CELL / 4) {
        observations.extend(st::observe(&cp::generate(family, seed, horizon)));
    }
    // The split policy is the axis being measured: an observation from the
    // held-out half must never reach the inducer.
    let held_out: std::collections::BTreeSet<u8> = (0..cp::AXIS_CARDINALITY)
        .filter(|d| *d >= cp::AXIS_CARDINALITY / 2)
        .map(|d| d as u8)
        .collect();
    // The joint split and the topology split both hold out the high half of
    // the topology axis, so the leakage scan is armed for exactly the splits
    // where a topology cell is reserved.
    let arms_topology = matches!(split, Split::Joint | Split::Isolating(Axis::Topology));
    let policy = st::SplitPolicy {
        held_out_topologies: if arms_topology {
            held_out
        } else {
            std::collections::BTreeSet::new()
        },
        sealed_topologies: std::collections::BTreeSet::new(),
    };
    st::induce(&observations, &policy).induced().cloned()
}

/// Measure one arm against the strongest non-oracle null in one cell.
fn measure_cell(
    arm: &'static str,
    strategy: Option<PlanStrategy>,
    family: cp::TaskFamily,
    split: Split,
    horizon: usize,
    scratch: &mut PlanScratch,
) -> Option<Cell> {
    let set = induce_for(family, split, horizon)?;
    let packed = pack(&set, 0)?;
    let shuffled = pack(&set, 1)?;
    let fitted = fit_nulls(family, split, horizon);

    let mut arm_correct = 0usize;
    let mut null_correct = [0usize; 4];
    let mut differences: Vec<Vec<i8>> = vec![Vec::new(); 4];
    let mut within_budget = true;
    let budget = shared_budget(horizon);

    let evaluation = seeds(split, true, N_PER_CELL);
    for seed in &evaluation {
        let task = cp::generate(family, *seed, horizon);
        let emitted = match strategy {
            Some(strategy) => {
                let (plan, counters) =
                    production_measured(strategy, &packed, &task, horizon, scratch)?;
                // Budget parity is checked, not assumed: an arm that overspent
                // the shared ceiling is reported as invalid rather than
                // compared against one that did not.
                if counters.expansions > budget.max_expansions
                    || counters.candidates > budget.max_candidates
                    || counters.table_reads > budget.max_table_reads
                {
                    within_budget = false;
                }
                plan
            }
            None => None,
        };
        let correct = outcome_correct(&task, &emitted);
        if correct {
            arm_correct += 1;
        }
        let nulls: [Emission; 4] = [
            retrieval_only(&fitted, &task),
            direct_continuation(&task, horizon),
            memorized(&fitted, &task),
            production(
                PlanStrategy::BreadthFirst,
                &shuffled,
                &task,
                horizon,
                scratch,
            ),
        ];
        for (index, null) in nulls.iter().enumerate() {
            let null_ok = outcome_correct(&task, null);
            if null_ok {
                null_correct[index] += 1;
            }
            differences[index].push(i8::from(correct) - i8::from(null_ok));
        }
    }
    let names = [
        "retrieval-only",
        "direct-continuation",
        "memorized-trajectory",
        "shuffled-state",
    ];
    // The promotion statistic is taken against the STRONGEST non-oracle null.
    let strongest = (0..4).max_by_key(|i| null_correct[*i]).unwrap();
    let (mean, standard_error, lower) = paired_lower_bound(&differences[strongest]);
    Some(Cell {
        arm,
        axis: split.label(),
        family: family.label(),
        horizon,
        n: evaluation.len(),
        arm_correct,
        strongest_null: names[strongest],
        null_correct: null_correct[strongest],
        mean_difference: mean,
        standard_error,
        lower_bound: lower,
        counters_within_budget: within_budget,
    })
}

/// **Intersection-union reading (the one the #826 gate actually needs).**
///
/// The promotion gate is a *conjunction*: the bound must clear δ_min on every
/// required cell. For a conjunction the intersection-union principle applies
/// and each cell is tested at the full level α — a Bonferroni or Holm inflation
/// would be answering a different question, because those control false
/// rejections across many claims, while here a single failing cell sinks the
/// claim on its own and no inflation is needed to keep the family-wise error at
/// α.
fn intersection_union_pass(cells: &[Cell]) -> Vec<bool> {
    cells.iter().map(|c| c.lower_bound >= DELTA_MIN).collect()
}

/// **Holm–Bonferroni, as the constitution freezes it, in its standard
/// direction.** Sort the p-values ascending and step down: the strongest
/// evidence faces `α / m`, the next `α / (m - 1)`, and so on, stopping at the
/// first failure. This is the correct reading if the claim were "some cell
/// shows an effect"; it is reported alongside the conjunction reading rather
/// than instead of it, because the two answer different questions and the
/// constitution does not say which one the gate is.
fn holm_pass(cells: &[Cell]) -> Vec<bool> {
    let m = cells.len();
    let mut order: Vec<usize> = (0..m).collect();
    order.sort_by(|a, b| {
        p_value(&cells[*a])
            .partial_cmp(&p_value(&cells[*b]))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut pass = vec![false; m];
    for (rank, index) in order.iter().enumerate() {
        let level = 0.05 / (m - rank) as f64;
        if p_value(&cells[*index]) <= level {
            pass[*index] = true;
        } else {
            // Step-down: once a cell fails, every weaker cell fails too.
            break;
        }
    }
    pass
}

// ---------------------------------------------------------------------------
// Non-vacuity gates — these run by default, because a harness that has rotted
// into always-passing is worse than no harness
// ---------------------------------------------------------------------------

const ARMS: [(&str, PlanStrategy); 3] = [
    ("bounded-breadth-first", PlanStrategy::BreadthFirst),
    (
        "bounded-iterative-deepening",
        PlanStrategy::IterativeDeepening,
    ),
    ("table-guided-beam", PlanStrategy::BestFirstBeam),
];

/// The packing path is real: an induced rule set becomes sections the deployed
/// planner can actually plan on, and the plan the #844 verifier accepts.
#[test]
fn the_harness_packs_an_induced_rule_set_and_plans_on_it() {
    let mut scratch = PlanScratch::new();
    for family in FAMILIES {
        let set = induce_for(family, Split::Joint, 8).expect("induction produces a rule set");
        assert!(!set.rules.is_empty(), "{}: empty rule set", family.label());
        let packed = pack(&set, 0).expect("the rule set packs");
        let task = cp::generate(family, 4, 8);
        let mask = mask_for(&packed, &task);
        assert_ne!(
            mask,
            0,
            "{}: the instance offers no operator the artifact knows",
            family.label()
        );
        let emitted = production(PlanStrategy::BreadthFirst, &packed, &task, 8, &mut scratch);
        assert!(
            outcome_correct(&task, &emitted),
            "{}: the deployed planner did not produce an accepted outcome",
            family.label()
        );
    }
}

/// Every null can fire and can fail. A control stuck at zero is as useless as
/// one saturated at one, and both read as healthy from a single number.
#[test]
fn every_null_can_fire_and_can_fail() {
    for family in [
        cp::TaskFamily::GraphNavigation,
        cp::TaskFamily::ConstraintSatisfaction,
    ] {
        let fitted = fit_nulls(family, Split::Joint, 8);
        let mut scratch = PlanScratch::new();
        let set = induce_for(family, Split::Joint, 8).unwrap();
        let shuffled = pack(&set, 1).unwrap();
        let mut fired = [false; 4];
        let mut failed = [false; 4];
        for seed in seeds(Split::Joint, true, 128) {
            let task = cp::generate(family, seed, 8);
            let nulls = [
                retrieval_only(&fitted, &task),
                direct_continuation(&task, 8),
                memorized(&fitted, &task),
                production(
                    PlanStrategy::BreadthFirst,
                    &shuffled,
                    &task,
                    8,
                    &mut scratch,
                ),
            ];
            for (index, null) in nulls.iter().enumerate() {
                if outcome_correct(&task, null) {
                    fired[index] = true;
                } else {
                    failed[index] = true;
                }
            }
        }
        let names = [
            "retrieval-only",
            "direct-continuation",
            "memorized-trajectory",
            "shuffled-state",
        ];
        for index in 0..4 {
            assert!(
                fired[index],
                "{} / {}: the control never fired, so it cannot be beaten meaningfully",
                family.label(),
                names[index]
            );
            assert!(
                failed[index],
                "{} / {}: the control never failed, so it cannot separate anything",
                family.label(),
                names[index]
            );
        }
    }
}

/// The shuffled-state null must genuinely break: a mechanism that survives a
/// broken correspondence between operators and effects was not using the
/// semantics. Its rate must be strictly below the real arm's.
#[test]
fn the_shuffled_state_control_collapses_relative_to_the_arm() {
    let family = cp::TaskFamily::GraphNavigation;
    let set = induce_for(family, Split::Joint, 8).unwrap();
    let packed = pack(&set, 0).unwrap();
    let shuffled = pack(&set, 1).unwrap();
    let mut scratch = PlanScratch::new();
    let (mut real, mut broken, mut total) = (0usize, 0usize, 0usize);
    for seed in seeds(Split::Joint, true, 128) {
        let task = cp::generate(family, seed, 8);
        total += 1;
        if outcome_correct(
            &task,
            &production(PlanStrategy::BreadthFirst, &packed, &task, 8, &mut scratch),
        ) {
            real += 1;
        }
        if outcome_correct(
            &task,
            &production(
                PlanStrategy::BreadthFirst,
                &shuffled,
                &task,
                8,
                &mut scratch,
            ),
        ) {
            broken += 1;
        }
    }
    println!("real {real}/{total}, shuffled {broken}/{total}");
    assert!(
        broken < real,
        "shuffling the operator/effect correspondence did not degrade the arm: \
         real {real}/{total}, shuffled {broken}/{total}"
    );
}

/// The lower-bound arithmetic is the promotion statistic, so it is asserted
/// rather than trusted.
#[test]
fn the_paired_lower_bound_is_conservative_and_signed() {
    let (mean, _, lower) = paired_lower_bound(&[1; 512]);
    assert_eq!(mean, 1.0);
    assert_eq!(lower, 1.0, "a unanimous difference has no sampling spread");
    let (mean, _, lower) = paired_lower_bound(&[0; 512]);
    assert_eq!(mean, 0.0);
    assert_eq!(lower, 0.0);
    let mut mixed = vec![0i8; 512];
    for slot in mixed.iter_mut().take(33) {
        *slot = 1;
    }
    let (mean, _, lower) = paired_lower_bound(&mixed);
    assert!((mean - 33.0 / 512.0).abs() < 1e-12);
    assert!(lower < mean, "the bound must sit below the point estimate");
    assert!(
        lower < DELTA_MIN,
        "a 6.4% point estimate must not clear the floor at n=512: {lower}"
    );
    // A negative difference stays negative: the bound never launders a loss.
    let (mean, _, lower) = paired_lower_bound(&[-1; 128]);
    assert_eq!(mean, -1.0);
    assert!(lower <= -1.0);
}

// ---------------------------------------------------------------------------
// The full grid — a measurement, not a gate
// ---------------------------------------------------------------------------

/// The frozen §10 measurement: three arms and four nulls over
/// (axis × family × horizon), n = 512 per cell, Holm–Bonferroni across the grid.
///
/// Run with:
/// `cargo test -p uor-r4-graph-certify --test compositional_planning_measurement_843 -- --ignored --nocapture`
#[test]
#[ignore]
fn frozen_grid_measurement() {
    let splits = [
        Split::Joint,
        Split::Isolating(Axis::Entity),
        Split::Isolating(Axis::Vocabulary),
        Split::Isolating(Axis::Topology),
        Split::Isolating(Axis::Template),
    ];
    let mut scratch = PlanScratch::new();
    let mut all: Vec<Cell> = Vec::new();

    for (arm, strategy) in ARMS {
        for split in splits {
            for horizon in FROZEN_HORIZONS {
                for family in FAMILIES {
                    let Some(cell) =
                        measure_cell(arm, Some(strategy), family, split, horizon, &mut scratch)
                    else {
                        println!(
                            "UNAVAILABLE {arm} {} {} H={horizon}",
                            split.label(),
                            family.label()
                        );
                        continue;
                    };
                    println!(
                        "{:<28} {:<24} {:<28} H={:<2} n={:<4} arm={:.4} null[{:<20}]={:.4} d={:+.4} lb={:+.4}",
                        cell.arm,
                        cell.axis,
                        cell.family,
                        cell.horizon,
                        cell.n,
                        cell.arm_correct as f64 / cell.n as f64,
                        cell.strongest_null,
                        cell.null_correct as f64 / cell.n as f64,
                        cell.mean_difference,
                        cell.lower_bound
                    );
                    all.push(cell);
                }
            }
        }
    }

    println!("\n=== PER-ARM VERDICT (delta_min = {DELTA_MIN}) ===");
    println!(
        "Two readings are reported. The #826 gate is a CONJUNCTION - the bound must clear the\n\
         floor on every required cell - so the intersection-union reading tests each cell at the\n\
         full level and needs no inflation. Holm-Bonferroni, which the constitution freezes, is\n\
         the correct reading of a DISJUNCTION (some cell shows an effect) and is reported too.\n"
    );
    for (arm, _) in ARMS {
        let cells: Vec<Cell> = all.iter().filter(|c| c.arm == arm).cloned().collect();
        if cells.is_empty() {
            continue;
        }
        let holm = holm_pass(&cells);
        println!(
            "{arm}: Holm-Bonferroni rejects the null in {}/{} cells",
            holm.iter().filter(|ok| **ok).count(),
            cells.len()
        );
        let pass = intersection_union_pass(&cells);
        let failing: Vec<String> = cells
            .iter()
            .zip(pass.iter())
            .filter(|(_, ok)| !**ok)
            .map(|(c, _)| {
                format!(
                    "{} {} H={} (lb {:+.4}, strongest null {} at {:.4})",
                    c.axis,
                    c.family,
                    c.horizon,
                    c.lower_bound,
                    c.strongest_null,
                    c.null_correct as f64 / c.n as f64
                )
            })
            .collect();
        let worst = cells
            .iter()
            .map(|c| c.lower_bound)
            .fold(f64::INFINITY, f64::min);
        println!(
            "{arm}: conjunction reading - {}/{} cells clear the floor; weakest lower bound {:+.4}",
            cells.len() - failing.len(),
            cells.len(),
            worst
        );
        for line in failing.iter().take(12) {
            println!("    FAILS: {line}");
        }
        if failing.len() > 12 {
            println!("    ... and {} more failing cells", failing.len() - 12);
        }
        assert!(
            cells.iter().all(|c| c.counters_within_budget),
            "{arm} overspent the shared budget, so its reading is invalid rather than comparable"
        );
    }
}

// ---------------------------------------------------------------------------
// §9 three-way differential: reference / packed / production
// ---------------------------------------------------------------------------

/// A breadth-first reference planner over the **reference** semantic model:
/// owned states, f32 vectors, the `semantic_state` evaluator. It shares no code
/// with the packed or production paths.
///
/// Operators are explored in ascending *effect* order so all three arms carry
/// the same canonical expansion order — a differential that compared three
/// different orderings would be measuring the ordering, not the semantics.
fn reference_plan(task: &cp::TaskInstance, horizon: usize) -> Emission {
    use uor_r4_graph_compiler::semantic_state::TransitionEvaluator;
    let mut evaluator = TransitionEvaluator::new();
    for constraint in &task.constraints {
        evaluator.add_constraint(constraint.clone());
    }
    let mut ordered: Vec<Action> = task.actions.clone();
    ordered.sort_by_key(|a| ints(&a.delta_vector));

    if task.goal.is_satisfied_by(&task.initial_state) {
        return Some(Vec::new());
    }
    let key = |s: &uor_r4_graph_compiler::semantic_state::SemanticState| ints(&s.vector);
    let mut seen = std::collections::BTreeSet::new();
    seen.insert(key(&task.initial_state));
    let mut frontier = vec![(task.initial_state.clone(), Vec::<Vec<i16>>::new())];
    for _ in 0..horizon {
        let mut next = Vec::new();
        for (state, path) in &frontier {
            for action in &ordered {
                let Some(successor) = evaluator.apply(state, action) else {
                    continue;
                };
                if !seen.insert(key(&successor)) {
                    continue;
                }
                let mut extended = path.clone();
                extended.push(ints(&action.delta_vector));
                if task.goal.is_satisfied_by(&successor) {
                    return Some(extended);
                }
                next.push((successor, extended));
            }
        }
        if next.is_empty() {
            return None;
        }
        frontier = next;
    }
    None
}

/// A breadth-first planner reading the **packed** sections directly, written
/// independently of `uor_r4_graph_runtime::plan`. Owned collections and a plain
/// `BTreeSet` closed set: it is the offline reading of the same bytes, so a
/// disagreement with the deployed planner isolates the deployed *implementation*
/// rather than the format or the algorithm.
fn packed_plan(packed: &Packed, task: &cp::TaskInstance, horizon: usize) -> Emission {
    let schema = PlanSchema::parse(&packed.schema).ok()?;
    let rules = RuleTable::parse(&packed.rules, &schema).ok()?;
    let predicate_bytes = predicates_for(task)?;
    let predicates = PredicateSet::parse(&predicate_bytes, &schema).ok()?;
    let available = mask_for(packed, task);
    let initial = initial_of(task);

    if predicates.satisfies_goal(&initial) {
        return Some(Vec::new());
    }
    if predicates.is_forbidden(&initial) {
        return None;
    }
    let mut seen = std::collections::BTreeSet::new();
    seen.insert(initial);
    let mut frontier = vec![(initial, Vec::<Vec<i16>>::new())];
    for _ in 0..horizon {
        let mut next = Vec::new();
        for (state, path) in &frontier {
            for operator in 0..schema.operator_count() {
                if available & (1u64 << operator) == 0 {
                    continue;
                }
                let Some((first, end)) = rules.rules_for(operator) else {
                    continue;
                };
                for row in first..end {
                    let Some(rule) = rules.rule(row) else {
                        continue;
                    };
                    if !rule.precondition.holds(state) {
                        continue;
                    }
                    let Some(successor) = state.apply(&rule.effect) else {
                        continue;
                    };
                    if predicates.is_forbidden(&successor) || !seen.insert(successor) {
                        continue;
                    }
                    let mut extended = path.clone();
                    extended.push(rule.effect.as_slice().to_vec());
                    if predicates.satisfies_goal(&successor) {
                        return Some(extended);
                    }
                    next.push((successor, extended));
                }
            }
        }
        if next.is_empty() {
            return None;
        }
        frontier = next;
    }
    None
}

/// **Guarantee (reference, packed and production agree on valid fixtures; all
/// three fail closed on corrupt or incompatible data). Status: Structural.**
///
/// Disagreement fails the gate rather than selecting a winner: three
/// implementations that disagree tell you one of them is wrong, not which.
#[test]
fn reference_packed_and_production_agree_on_every_fixture() {
    let mut scratch = PlanScratch::new();
    let mut compared = 0usize;
    for family in FAMILIES {
        let set = induce_for(family, Split::Joint, 8).expect("induction produces a rule set");
        let packed = pack(&set, 0).expect("the rule set packs");
        for horizon in FROZEN_HORIZONS {
            for seed in seeds(Split::Joint, true, 24) {
                let task = cp::generate(family, seed, horizon);
                let reference = reference_plan(&task, horizon);
                let offline = packed_plan(&packed, &task, horizon);
                let deployed = production(
                    PlanStrategy::BreadthFirst,
                    &packed,
                    &task,
                    horizon,
                    &mut scratch,
                );
                compared += 1;

                assert_eq!(
                    reference.is_some(),
                    offline.is_some(),
                    "{} seed {seed} H={horizon}: reference and packed disagree on whether a plan exists",
                    family.label()
                );
                assert_eq!(
                    offline, deployed,
                    "{} seed {seed} H={horizon}: the packed and deployed readings of the same bytes differ",
                    family.label()
                );
                assert_eq!(
                    reference.as_ref().map(|p| p.len()),
                    deployed.as_ref().map(|p| p.len()),
                    "{} seed {seed} H={horizon}: reference and production plan lengths differ",
                    family.label()
                );
                // Whatever each produced, the frozen #844 verifier is the judge.
                for candidate in [&reference, &deployed] {
                    assert!(
                        outcome_correct(&task, candidate),
                        "{} seed {seed} H={horizon}: an arm produced an outcome the benchmark verifier rejects",
                        family.label()
                    );
                }
            }
        }
    }
    println!("three-way differential: {compared} fixtures, all three paths agree");
    assert!(compared >= 400, "the differential must be non-vacuous");
}

/// All three fail closed together on incompatible data: a rule table built for
/// a different schema is not a product of these bytes for any of them.
#[test]
fn all_three_paths_fail_closed_on_incompatible_data() {
    let set = induce_for(cp::TaskFamily::GraphNavigation, Split::Joint, 8).unwrap();
    let packed = pack(&set, 0).unwrap();
    let mut corrupt = packed.rules.clone();
    // Flip the declared operator count so the table no longer matches PSCH.
    corrupt[8] = corrupt[8].wrapping_add(1);
    let broken = Packed {
        schema: packed.schema.clone(),
        rules: corrupt,
        effects: packed.effects.clone(),
    };
    let task = cp::generate(cp::TaskFamily::GraphNavigation, 4, 8);
    let mut scratch = PlanScratch::new();
    assert!(
        packed_plan(&broken, &task, 8).is_none(),
        "the offline reading must fail closed on an incompatible table"
    );
    assert!(
        production(PlanStrategy::BreadthFirst, &broken, &task, 8, &mut scratch).is_none(),
        "the deployed reading must fail closed on an incompatible table"
    );
    // The reference path does not read the artifact at all, so it is unaffected
    // — which is exactly why it is the independent third opinion.
    assert!(reference_plan(&task, 8).is_some());
}
