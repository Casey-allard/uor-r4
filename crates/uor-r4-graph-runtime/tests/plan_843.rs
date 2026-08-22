//! The deployed bounded planner — the machine-checked record for #843
//! increment 5.
//!
//! Frozen contract: `docs/bounded_semantic_transitions_spec_843.md` §6 and §7.
//! Normative deployed-serving scope: the planner runs on packed sections with
//! caller-owned scratch, executes only P-4 operations, and allocates nothing.
//! Nothing here claims a planning *capability* — that is increment 6's
//! measurement. What is asserted here is that the machinery is total,
//! deterministic, bounded, and honest at its boundaries.

use uor_r4_graph_format::plan::{
    CompareOp, EffectDelta, PLAN_FRONTIER_MAX, PLAN_HORIZON_MAX, PLAN_WITNESS_MAX_BYTES,
    PreconditionMask, SlotVec,
};
use uor_r4_graph_format::plan_sections::{
    PackedDecline, PackedRule, PlanSchema, PlanWitnessBytes, PredicateSet, ReplayVerdict,
    RuleTable, WitnessDraft, WitnessStep, build_predicate_set, build_rule_table, build_schema,
    encode_witness_into,
};
use uor_r4_graph_runtime::plan::{
    PlanBudget, PlanOutcome, PlanQuery, PlanScratch, PlanStrategy, plan,
};

const ARITY: u8 = 2;
const BANDS: (u32, u32, u32) = (1, 4, 16);

fn effect(x: i16, y: i16) -> EffectDelta {
    EffectDelta::from_slice(&[x, y]).unwrap()
}

fn slots(x: i16, y: i16) -> SlotVec {
    SlotVec::from_slice(&[x, y]).unwrap()
}

fn cell_predicate(x: i16, y: i16) -> PreconditionMask {
    PreconditionMask::unconditional()
        .reading(0, CompareOp::Equal, x)
        .unwrap()
        .reading(1, CompareOp::Equal, y)
        .unwrap()
}

/// A four-operator grid artifact: the packed sections a query reads.
struct Fixture {
    schema: Vec<u8>,
    rules: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        let vocabulary = vec![effect(1, 0), effect(0, 1), effect(-1, 0), effect(0, -1)];
        let schema = build_schema(ARITY, &vocabulary, BANDS).unwrap();
        let parsed = PlanSchema::parse(&schema).unwrap();
        let rules: Vec<PackedRule> = (0..parsed.operator_count())
            .map(|index| PackedRule {
                operator: index as u16,
                precondition: PreconditionMask::unconditional(),
                effect: parsed.operator(index).unwrap(),
                support: 8,
                band: 2,
            })
            .collect();
        let rules = build_rule_table(ARITY, parsed.operator_count() as u16, &rules).unwrap();
        Self { schema, rules }
    }
}

fn budget() -> PlanBudget {
    PlanBudget {
        horizon: 8,
        frontier: PLAN_FRONTIER_MAX as u16,
        ..PlanBudget::frozen()
    }
}

/// Run one episode against the standard fixture: reach `(3, 0)` from the
/// origin without entering `(2, 0)`.
fn run(
    strategy: PlanStrategy,
    goal: (i16, i16),
    blocked: &[(i16, i16)],
    available: u64,
    budget: PlanBudget,
    scratch: &mut PlanScratch,
) -> (PlanOutcome, uor_r4_graph_runtime::plan::PlanCounters) {
    let fixture = Fixture::new();
    let schema = PlanSchema::parse(&fixture.schema).unwrap();
    let rules = RuleTable::parse(&fixture.rules, &schema).unwrap();
    let constraints: Vec<PreconditionMask> = blocked
        .iter()
        .map(|(x, y)| cell_predicate(*x, *y))
        .collect();
    let raw = build_predicate_set(ARITY, &[cell_predicate(goal.0, goal.1)], &constraints).unwrap();
    let predicates = PredicateSet::parse(&raw, &schema).unwrap();
    let result = plan(
        &PlanQuery {
            strategy,
            schema: &schema,
            rules: &rules,
            predicates: &predicates,
            initial: slots(0, 0),
            available,
            budget,
        },
        scratch,
    );
    (result.outcome, result.counters)
}

/// Encode the scratch's plan as a witness and replay it independently.
fn replay_plan(scratch: &PlanScratch, goal: (i16, i16), blocked: &[(i16, i16)]) -> ReplayVerdict {
    let steps: Vec<WitnessStep> = (0..scratch.path_len())
        .map(|i| scratch.path_step(i).unwrap())
        .collect();
    let constraints: Vec<PreconditionMask> = blocked
        .iter()
        .map(|(x, y)| cell_predicate(*x, *y))
        .collect();
    let draft = WitnessDraft {
        slot_count: ARITY,
        initial: slots(0, 0),
        goal: cell_predicate(goal.0, goal.1),
        constraints: &constraints,
        steps: &steps,
        considered: scratch.considered(),
        considered_per_step: scratch.considered_per_step() as u8,
        decline: None,
        verdict: (0, 0),
    };
    let mut buffer = [0u8; PLAN_WITNESS_MAX_BYTES];
    let written = encode_witness_into(&draft, &mut buffer).expect("the witness fits the envelope");
    PlanWitnessBytes::parse(&buffer[..written])
        .expect("the emitted witness is a product of its own bytes")
        .replay()
}

// ---------------------------------------------------------------------------

#[test]
fn a_bounded_episode_finds_a_plan_that_replays_valid() {
    let mut scratch = PlanScratch::new();
    let (outcome, counters) = run(
        PlanStrategy::BreadthFirst,
        (3, 0),
        &[(2, 0)],
        0b1111,
        budget(),
        &mut scratch,
    );
    let PlanOutcome::Plan { steps } = outcome else {
        panic!("expected a plan, got {outcome:?}");
    };
    assert_eq!(steps, scratch.path_len());
    assert!(
        steps >= 3,
        "a detour around the block takes at least 3 steps"
    );
    assert!(steps <= usize::from(budget().horizon));
    assert!(counters.expansions > 0 && counters.table_reads > 0);
    assert_eq!(
        replay_plan(&scratch, (3, 0), &[(2, 0)]),
        ReplayVerdict::Valid
    );
}

/// All three arms read the same sections, execute the same operation set, and
/// run under the same budget, so a difference between them is a difference of
/// search order and nothing else.
#[test]
fn every_strategy_finds_a_plan_that_replays_valid_under_the_same_budget() {
    for strategy in [
        PlanStrategy::BreadthFirst,
        PlanStrategy::IterativeDeepening,
        PlanStrategy::BestFirstBeam,
    ] {
        let mut scratch = PlanScratch::new();
        let (outcome, counters) = run(strategy, (3, 0), &[(2, 0)], 0b1111, budget(), &mut scratch);
        let PlanOutcome::Plan { steps } = outcome else {
            panic!("{strategy:?} declined: {outcome:?}");
        };
        assert!(steps > 0, "{strategy:?} returned an empty plan");
        assert_eq!(
            replay_plan(&scratch, (3, 0), &[(2, 0)]),
            ReplayVerdict::Valid,
            "{strategy:?} emitted a witness that does not replay"
        );
        assert!(
            counters.expansions <= budget().max_expansions
                && counters.candidates <= budget().max_candidates
                && counters.table_reads <= budget().max_table_reads,
            "{strategy:?} overspent its declared budget: {counters:?}"
        );
        assert!(
            counters.max_probe <= uor_r4_graph_runtime::plan::VISITED_MAX_PROBE,
            "{strategy:?} exceeded the checked probe bound"
        );
    }
}

#[test]
fn no_step_of_a_plan_enters_a_forbidden_region() {
    let blocked = [(2, 0), (1, 1)];
    let mut scratch = PlanScratch::new();
    let (outcome, _) = run(
        PlanStrategy::BreadthFirst,
        (3, 0),
        &blocked,
        0b1111,
        budget(),
        &mut scratch,
    );
    assert!(matches!(outcome, PlanOutcome::Plan { .. }));
    for index in 0..scratch.path_len() {
        let (_, state, _, _) = scratch.path_step(index).unwrap();
        for (x, y) in blocked {
            assert_ne!(
                state,
                slots(x, y),
                "step {index} entered the forbidden cell ({x}, {y})"
            );
        }
    }
    assert_eq!(
        replay_plan(&scratch, (3, 0), &blocked),
        ReplayVerdict::Valid
    );
}

#[test]
fn an_unreachable_goal_declines_no_plan_rather_than_fabricating_one() {
    let mut scratch = PlanScratch::new();
    let (outcome, counters) = run(
        PlanStrategy::BreadthFirst,
        (100, 0),
        &[],
        0b1111,
        PlanBudget {
            horizon: 4,
            ..budget()
        },
        &mut scratch,
    );
    assert_eq!(outcome, PlanOutcome::Declined(PackedDecline::NoPlan));
    assert_eq!(scratch.path_len(), 0, "a decline emits no plan");
    assert!(
        counters.expansions > 0,
        "the decline is measured, not assumed"
    );
}

#[test]
fn a_budget_beyond_the_frozen_capacity_declines_capacity() {
    let mut scratch = PlanScratch::new();
    let (outcome, _) = run(
        PlanStrategy::BreadthFirst,
        (3, 0),
        &[(2, 0)],
        0b1111,
        PlanBudget {
            horizon: (PLAN_HORIZON_MAX + 1) as u8,
            ..budget()
        },
        &mut scratch,
    );
    assert_eq!(outcome, PlanOutcome::Declined(PackedDecline::Capacity));

    let mut scratch = PlanScratch::new();
    let (outcome, _) = run(
        PlanStrategy::BreadthFirst,
        (3, 0),
        &[(2, 0)],
        0b1111,
        PlanBudget {
            frontier: (PLAN_FRONTIER_MAX + 1) as u16,
            ..budget()
        },
        &mut scratch,
    );
    assert_eq!(outcome, PlanOutcome::Declined(PackedDecline::Capacity));
}

/// An exhausted expansion budget is a capacity decline, not a truncated plan.
#[test]
fn an_exhausted_expansion_budget_declines_rather_than_truncating() {
    let mut scratch = PlanScratch::new();
    let (outcome, counters) = run(
        PlanStrategy::BreadthFirst,
        (100, 0),
        &[],
        0b1111,
        PlanBudget {
            horizon: PLAN_HORIZON_MAX as u8,
            max_expansions: 4,
            ..budget()
        },
        &mut scratch,
    );
    assert_eq!(outcome, PlanOutcome::Declined(PackedDecline::Capacity));
    assert_eq!(scratch.path_len(), 0);
    assert!(counters.expansions >= 4);
}

#[test]
fn planning_is_deterministic() {
    let plan_once = || {
        let mut scratch = PlanScratch::new();
        let (outcome, counters) = run(
            PlanStrategy::BestFirstBeam,
            (3, 0),
            &[(2, 0)],
            0b1111,
            budget(),
            &mut scratch,
        );
        let path: Vec<WitnessStep> = (0..scratch.path_len())
            .map(|i| scratch.path_step(i).unwrap())
            .collect();
        (outcome, counters, path)
    };
    let first = plan_once();
    let second = plan_once();
    assert_eq!(first.0, second.0);
    assert_eq!(first.1, second.1, "the counters must be reproducible too");
    assert_eq!(first.2, second.2);
}

/// The operator vocabulary is artifact-wide; which operators an instance offers
/// is a property of the query. An operator the mask withholds is never used.
#[test]
fn an_unavailable_operator_is_never_used() {
    let fixture = Fixture::new();
    let schema = PlanSchema::parse(&fixture.schema).unwrap();
    // Withhold the +x operator and the goal on the +x axis becomes unreachable.
    let plus_x = (0..schema.operator_count())
        .find(|i| schema.operator(*i).unwrap() == effect(1, 0))
        .unwrap();
    let mask = 0b1111u64 & !(1u64 << plus_x);

    let mut scratch = PlanScratch::new();
    let (outcome, _) = run(
        PlanStrategy::BreadthFirst,
        (3, 0),
        &[],
        mask,
        budget(),
        &mut scratch,
    );
    assert_eq!(
        outcome,
        PlanOutcome::Declined(PackedDecline::NoPlan),
        "withholding the only operator that can reach the goal must decline"
    );

    // With it available the same query succeeds, so the decline was the mask.
    let mut scratch = PlanScratch::new();
    let (outcome, _) = run(
        PlanStrategy::BreadthFirst,
        (3, 0),
        &[],
        0b1111,
        budget(),
        &mut scratch,
    );
    assert!(matches!(outcome, PlanOutcome::Plan { .. }));
}

#[test]
fn an_initial_state_on_the_goal_is_a_zero_step_plan() {
    let mut scratch = PlanScratch::new();
    let (outcome, _) = run(
        PlanStrategy::BreadthFirst,
        (0, 0),
        &[],
        0b1111,
        budget(),
        &mut scratch,
    );
    assert_eq!(outcome, PlanOutcome::Plan { steps: 0 });
    assert_eq!(scratch.path_len(), 0);
}

#[test]
fn a_forbidden_initial_state_declines() {
    let mut scratch = PlanScratch::new();
    let (outcome, _) = run(
        PlanStrategy::BreadthFirst,
        (3, 0),
        &[(0, 0)],
        0b1111,
        budget(),
        &mut scratch,
    );
    assert_eq!(outcome, PlanOutcome::Declined(PackedDecline::NoPlan));
}

/// The scratch is a compile-time function of the frozen capacities. The number
/// is asserted rather than described, so it cannot drift unnoticed.
#[test]
fn the_scratch_size_is_a_compile_time_function_of_the_frozen_capacities() {
    let size = core::mem::size_of::<PlanScratch>();
    assert!(
        size <= 96 * 1024,
        "the caller-owned scratch grew past its declared envelope: {size} bytes"
    );
    assert!(
        size > 32 * 1024,
        "the scratch should hold the frozen capacities"
    );
    println!("PlanScratch = {size} bytes");
}
