//! Packed bounded-planning sections — the machine-checked record for #843
//! increment 4.
//!
//! Frozen contract: `docs/bounded_semantic_transitions_spec_843.md` §5 and §7.
//! Every guarantee those sections state is asserted here, including the
//! planted-mutation table: each mutation is planted and the named detector must
//! fire, and a mutation that fires no detector fails the suite.

use uor_r4_graph_format::plan::{
    CompareOp, EffectDelta, PreconditionMask, SlotVec, PLAN_ACTIONS_MAX, PLAN_HORIZON_MAX,
    PLAN_RULES_MAX, PLAN_SLOTS_MAX,
};
use uor_r4_graph_format::plan_sections::{
    build_predicate_set, build_rule_table, build_schema, build_witness, ConsideredCandidate,
    PackedDecline, PackedRule, PlanSchema, PlanWitnessBytes, PredicateSet, ReplayFault,
    ReplayVerdict, RuleTable, WitnessDraft, WitnessStep, PGOL_HEADER_LEN, PSCH_HEADER_LEN,
    PTRN_HEADER_LEN, PTRN_ROW_LEN, PWIT_HEADER_LEN,
};
use uor_r4_graph_format::SectionId;

const ARITY: u8 = 2;
const BANDS: (u32, u32, u32) = (1, 4, 16);

fn effect(x: i16, y: i16) -> EffectDelta {
    EffectDelta::from_slice(&[x, y]).unwrap()
}

fn slots(x: i16, y: i16) -> SlotVec {
    SlotVec::from_slice(&[x, y]).unwrap()
}

/// The four axis effects, canonically ordered by the builder.
fn vocabulary() -> Vec<EffectDelta> {
    vec![effect(1, 0), effect(0, 1), effect(-1, 0), effect(0, -1)]
}

fn schema_bytes() -> Vec<u8> {
    build_schema(ARITY, &vocabulary(), BANDS).unwrap()
}

fn rules_for(schema: &PlanSchema<'_>) -> Vec<PackedRule> {
    (0..schema.operator_count())
        .map(|index| PackedRule {
            operator: index as u16,
            precondition: PreconditionMask::unconditional(),
            effect: schema.operator(index).unwrap(),
            support: 8 + index as u32,
            band: 2,
        })
        .collect()
}

/// A goal predicate satisfied exactly at `(x, y)`.
fn goal_at(x: i16, y: i16) -> PreconditionMask {
    PreconditionMask::unconditional()
        .reading(0, CompareOp::Equal, x)
        .unwrap()
        .reading(1, CompareOp::Equal, y)
        .unwrap()
}

// ---------------------------------------------------------------------------
// PSCH
// ---------------------------------------------------------------------------

#[test]
fn a_schema_round_trips_and_orders_its_vocabulary_canonically() {
    let bytes = schema_bytes();
    let schema = PlanSchema::parse(&bytes).unwrap();
    assert_eq!(schema.slot_count(), usize::from(ARITY));
    assert_eq!(schema.operator_count(), 4);
    assert_eq!(schema.band_thresholds(), BANDS);

    let mut previous = None;
    for index in 0..schema.operator_count() {
        let current = schema.operator(index).unwrap();
        assert!(
            previous.is_none_or(|p| p < current),
            "the vocabulary must be strictly ascending"
        );
        previous = Some(current);
    }
    assert!(schema.operator(schema.operator_count()).is_none());

    // The builder sorts, so declaration order cannot produce a second encoding.
    let shuffled = build_schema(
        ARITY,
        &[effect(0, -1), effect(1, 0), effect(0, 1), effect(-1, 0)],
        BANDS,
    )
    .unwrap();
    assert_eq!(shuffled, bytes, "the section has one canonical form");
}

#[test]
fn a_schema_refuses_a_duplicate_operator_and_a_bad_band_order() {
    assert!(build_schema(ARITY, &[effect(1, 0), effect(1, 0)], BANDS).is_none());
    assert!(build_schema(ARITY, &vocabulary(), (4, 4, 16)).is_none());
    assert!(build_schema(0, &vocabulary(), BANDS).is_none());
}

#[test]
fn a_schema_fails_closed_on_every_planted_header_mutation() {
    let good = schema_bytes();
    assert!(PlanSchema::parse(&good).is_ok());

    // truncation
    assert!(PlanSchema::parse(&good[..PSCH_HEADER_LEN - 1]).is_err());
    // a whole operator entry missing
    assert!(PlanSchema::parse(&good[..good.len() - 2]).is_err());
    // trailing bytes
    let mut trailing = good.clone();
    trailing.push(0);
    assert!(PlanSchema::parse(&trailing).is_err());
    // bad magic
    let mut magic = good.clone();
    magic[0] ^= 0xff;
    assert!(PlanSchema::parse(&magic).is_err());
    // unsupported version
    let mut version = good.clone();
    version[4] = 9;
    assert!(PlanSchema::parse(&version).is_err());
    // non-zero reserved
    let mut reserved = good.clone();
    reserved[10] = 1;
    assert!(PlanSchema::parse(&reserved).is_err());
    // a capacity this build does not enforce
    let mut capacity = good.clone();
    capacity[12] = (PLAN_HORIZON_MAX as u8) + 1;
    assert!(PlanSchema::parse(&capacity).is_err());
    // a non-canonical vocabulary: swap the first two entries
    let mut order = good.clone();
    for byte in 0..4 {
        order.swap(PSCH_HEADER_LEN + byte, PSCH_HEADER_LEN + 16 + byte);
    }
    assert!(PlanSchema::parse(&order).is_err());
}

// ---------------------------------------------------------------------------
// PTRN
// ---------------------------------------------------------------------------

#[test]
fn a_rule_table_round_trips_and_its_index_tiles_the_rows() {
    let schema_raw = schema_bytes();
    let schema = PlanSchema::parse(&schema_raw).unwrap();
    let rules = rules_for(&schema);
    let raw = build_rule_table(ARITY, schema.operator_count() as u16, &rules).unwrap();
    let table = RuleTable::parse(&raw, &schema).unwrap();

    assert_eq!(table.rule_count(), rules.len());
    assert_eq!(table.operator_count(), schema.operator_count());

    let mut covered = 0usize;
    for operator in 0..table.operator_count() {
        let (first, end) = table.rules_for(operator).unwrap();
        assert_eq!(first, covered, "the index must tile the rows in order");
        for row in first..end {
            let rule = table.rule(row).unwrap();
            assert_eq!(usize::from(rule.operator), operator);
            assert_eq!(rule.effect, schema.operator(operator).unwrap());
            assert_eq!(rule.precondition, PreconditionMask::unconditional());
        }
        covered = end;
    }
    assert_eq!(covered, table.rule_count(), "every rule is reachable");
    assert!(table.rules_for(table.operator_count()).is_none());
}

#[test]
fn a_rule_table_refuses_a_duplicate_key_and_an_unknown_operator() {
    let schema_raw = schema_bytes();
    let schema = PlanSchema::parse(&schema_raw).unwrap();
    let mut rules = rules_for(&schema);

    // Same operator, same precondition, same effect: one canonical rule, so a
    // second copy is a duplicate key rather than extra support.
    let duplicate = rules[0];
    rules.push(duplicate);
    assert!(build_rule_table(ARITY, schema.operator_count() as u16, &rules).is_none());

    let mut unknown = rules_for(&schema);
    unknown[0].operator = 99;
    assert!(build_rule_table(ARITY, schema.operator_count() as u16, &unknown).is_none());

    let mut bad_band = rules_for(&schema);
    bad_band[0].band = 4;
    assert!(build_rule_table(ARITY, schema.operator_count() as u16, &bad_band).is_none());
}

#[test]
fn a_rule_table_fails_closed_on_every_planted_mutation() {
    let schema_raw = schema_bytes();
    let schema = PlanSchema::parse(&schema_raw).unwrap();
    let good =
        build_rule_table(ARITY, schema.operator_count() as u16, &rules_for(&schema)).unwrap();
    assert!(RuleTable::parse(&good, &schema).is_ok());

    // truncated section
    assert!(RuleTable::parse(&good[..good.len() - 1], &schema).is_err());
    // trailing bytes
    let mut trailing = good.clone();
    trailing.push(0);
    assert!(RuleTable::parse(&trailing, &schema).is_err());
    // bad magic
    let mut magic = good.clone();
    magic[1] ^= 0xff;
    assert!(RuleTable::parse(&magic, &schema).is_err());
    // unsupported version
    let mut version = good.clone();
    version[4] = 7;
    assert!(RuleTable::parse(&version, &schema).is_err());
    // non-zero reserved in the header
    let mut reserved = good.clone();
    reserved[11] = 1;
    assert!(RuleTable::parse(&reserved, &schema).is_err());
    // an out-of-range operator id in a row
    let mut operator = good.clone();
    operator[PTRN_HEADER_LEN] = 200;
    assert!(RuleTable::parse(&operator, &schema).is_err());
    // a read mask that disagrees with the row's per-slot operations
    let mut mask = good.clone();
    mask[PTRN_HEADER_LEN + 2] = 0b0000_0001;
    assert!(RuleTable::parse(&mask, &schema).is_err());
    // a band outside the ordinal range
    let mut band = good.clone();
    band[PTRN_HEADER_LEN + 3] = 9;
    assert!(RuleTable::parse(&band, &schema).is_err());
    // non-canonical rows: swap the first two
    let mut order = good.clone();
    for byte in 0..PTRN_ROW_LEN {
        order.swap(
            PTRN_HEADER_LEN + byte,
            PTRN_HEADER_LEN + PTRN_ROW_LEN + byte,
        );
    }
    assert!(RuleTable::parse(&order, &schema).is_err());
    // an index entry that does not tile the rows
    let mut index = good.clone();
    let index_at = PTRN_HEADER_LEN + 4 * PTRN_ROW_LEN;
    index[index_at] = 3;
    assert!(RuleTable::parse(&index, &schema).is_err());
}

#[test]
fn a_rule_table_must_agree_with_its_schema() {
    let schema_raw = schema_bytes();
    let schema = PlanSchema::parse(&schema_raw).unwrap();
    let raw = build_rule_table(ARITY, 3, &rules_for(&schema)[..3]).unwrap();
    assert!(
        RuleTable::parse(&raw, &schema).is_err(),
        "a table declaring a different operator count is not a product of this schema"
    );
}

// ---------------------------------------------------------------------------
// PGOL
// ---------------------------------------------------------------------------

#[test]
fn a_predicate_set_round_trips_and_evaluates_goals_and_constraints() {
    let schema_raw = schema_bytes();
    let schema = PlanSchema::parse(&schema_raw).unwrap();
    let goal = goal_at(3, 0);
    let blocked = goal_at(2, 0);
    let raw = build_predicate_set(ARITY, &[goal], &[blocked]).unwrap();
    let set = PredicateSet::parse(&raw, &schema).unwrap();

    assert_eq!(set.goal_count(), 1);
    assert_eq!(set.constraint_count(), 1);
    assert_eq!(set.goal(0).unwrap(), goal);
    assert_eq!(set.constraint(0).unwrap(), blocked);
    assert!(set.goal(1).is_none());
    assert!(set.constraint(1).is_none());

    assert!(set.satisfies_goal(&slots(3, 0)));
    assert!(!set.satisfies_goal(&slots(3, 1)));
    assert!(set.is_forbidden(&slots(2, 0)));
    assert!(!set.is_forbidden(&slots(1, 0)));
}

#[test]
fn a_predicate_set_fails_closed_on_every_planted_mutation() {
    let schema_raw = schema_bytes();
    let schema = PlanSchema::parse(&schema_raw).unwrap();
    let good =
        build_predicate_set(ARITY, &[goal_at(3, 0)], &[goal_at(2, 0), goal_at(2, 1)]).unwrap();
    assert!(PredicateSet::parse(&good, &schema).is_ok());

    assert!(PredicateSet::parse(&good[..PGOL_HEADER_LEN - 1], &schema).is_err());
    let mut trailing = good.clone();
    trailing.push(0);
    assert!(PredicateSet::parse(&trailing, &schema).is_err());
    let mut magic = good.clone();
    magic[2] ^= 0xff;
    assert!(PredicateSet::parse(&magic, &schema).is_err());
    let mut version = good.clone();
    version[4] = 3;
    assert!(PredicateSet::parse(&version, &schema).is_err());
    let mut reserved = good.clone();
    reserved[9] = 1;
    assert!(PredicateSet::parse(&reserved, &schema).is_err());
    // a slot count that disagrees with the schema
    let mut arity = good.clone();
    arity[8] = ARITY + 1;
    assert!(PredicateSet::parse(&arity, &schema).is_err());
    // an unknown comparison code
    let mut op = good.clone();
    op[PGOL_HEADER_LEN + 2] = 200;
    assert!(PredicateSet::parse(&op, &schema).is_err());
    // duplicate constraints are refused at build time
    assert!(
        build_predicate_set(ARITY, &[goal_at(3, 0)], &[goal_at(2, 0), goal_at(2, 0)]).is_none()
    );
}

// ---------------------------------------------------------------------------
// PWIT
// ---------------------------------------------------------------------------

/// A three-step plan from the origin to `(3, 0)` avoiding `(2, 0)`.
fn witness_draft_parts() -> (Vec<WitnessStep>, Vec<ConsideredCandidate>) {
    let steps = vec![
        (effect(1, 0), slots(1, 0), 0, 0),
        (effect(0, 1), slots(1, 1), 0, 1),
        (effect(1, 0), slots(2, 1), 0, 0),
        (effect(1, 0), slots(3, 1), 0, 0),
        (effect(0, -1), slots(3, 0), 0, 3),
    ];
    let considered = steps
        .iter()
        .enumerate()
        .map(|(index, (_, _, _, rule_row))| ConsideredCandidate {
            operator: *rule_row,
            rule_row: *rule_row,
            score: 10 - index as i32,
            tie_rank: 0,
            support: 8,
            band: 2,
            flags: 1,
        })
        .collect();
    (steps, considered)
}

fn good_witness() -> Vec<u8> {
    let (steps, considered) = witness_draft_parts();
    build_witness(&WitnessDraft {
        slot_count: ARITY,
        initial: slots(0, 0),
        goal: goal_at(3, 0),
        constraints: &[goal_at(2, 0)],
        steps: &steps,
        considered: &considered,
        considered_per_step: 1,
        decline: None,
        verdict: (0, 0),
    })
    .unwrap()
}

#[test]
fn a_witness_round_trips_and_replays_valid_from_its_own_bytes() {
    let raw = good_witness();
    let witness = PlanWitnessBytes::parse(&raw).unwrap();
    assert_eq!(witness.slot_count(), usize::from(ARITY));
    assert_eq!(witness.step_count(), 5);
    assert_eq!(witness.decline(), None);
    assert_eq!(witness.initial_state().unwrap(), slots(0, 0));
    assert_eq!(witness.goal().unwrap(), goal_at(3, 0));
    assert_eq!(witness.constraint(0).unwrap(), goal_at(2, 0));
    assert_eq!(witness.replay(), ReplayVerdict::Valid);
    assert_eq!(witness.recorded_verdict(), (0, 0));
    // The considered records are informational and do not drive replay.
    assert!(witness.considered(0).is_some());
    assert!(witness.considered(5).is_none());
}

#[test]
fn a_declining_witness_replays_as_an_honest_decline() {
    let raw = build_witness(&WitnessDraft {
        slot_count: ARITY,
        initial: slots(0, 0),
        goal: goal_at(3, 0),
        constraints: &[],
        steps: &[],
        considered: &[],
        considered_per_step: 0,
        decline: Some(PackedDecline::NoPlan),
        verdict: (2, 0),
    })
    .unwrap();
    let witness = PlanWitnessBytes::parse(&raw).unwrap();
    assert_eq!(witness.decline(), Some(PackedDecline::NoPlan));
    assert_eq!(
        witness.replay(),
        ReplayVerdict::Declined(PackedDecline::NoPlan)
    );
}

/// The #846 rule: a right answer reached through an invalid intermediate step
/// is not a valid plan.
#[test]
fn a_right_answer_through_an_invalid_intermediate_step_is_rejected() {
    // Step 2 walks straight through the forbidden cell `(2, 0)` but the plan
    // still terminates on the goal.
    let steps = vec![
        (effect(1, 0), slots(1, 0), 0, 0),
        (effect(1, 0), slots(2, 0), 0, 0),
        (effect(1, 0), slots(3, 0), 0, 0),
    ];
    let considered: Vec<ConsideredCandidate> = steps
        .iter()
        .map(|_| ConsideredCandidate {
            operator: 0,
            rule_row: 0,
            score: 1,
            tie_rank: 0,
            support: 8,
            band: 2,
            flags: 1,
        })
        .collect();
    let raw = build_witness(&WitnessDraft {
        slot_count: ARITY,
        initial: slots(0, 0),
        goal: goal_at(3, 0),
        constraints: &[goal_at(2, 0)],
        steps: &steps,
        considered: &considered,
        considered_per_step: 1,
        decline: None,
        verdict: (0, 0),
    })
    .unwrap();
    let witness = PlanWitnessBytes::parse(&raw).unwrap();
    assert_eq!(
        witness.replay(),
        ReplayVerdict::Invalid {
            step: 1,
            reason: ReplayFault::EntersForbiddenRegion
        },
        "the terminal state is right but the path is not"
    );
}

#[test]
fn a_witness_fails_closed_or_replays_invalid_on_every_planted_mutation() {
    let good = good_witness();
    assert!(PlanWitnessBytes::parse(&good).is_ok());

    // structural mutations: the parser must refuse
    assert!(PlanWitnessBytes::parse(&good[..PWIT_HEADER_LEN - 1]).is_err());
    let mut trailing = good.clone();
    trailing.push(0);
    assert!(PlanWitnessBytes::parse(&trailing).is_err());
    let mut magic = good.clone();
    magic[3] ^= 0xff;
    assert!(PlanWitnessBytes::parse(&magic).is_err());
    let mut version = good.clone();
    version[4] = 5;
    assert!(PlanWitnessBytes::parse(&version).is_err());
    let mut reserved = good.clone();
    reserved[14] = 1;
    assert!(PlanWitnessBytes::parse(&reserved).is_err());
    let mut decline = good.clone();
    decline[10] = 9;
    assert!(PlanWitnessBytes::parse(&decline).is_err());

    // semantic mutations: the parser accepts, and replay must reject
    let steps_at = PWIT_HEADER_LEN + PLAN_SLOTS_MAX * 2 + 28 + 28;

    // a corrupted step: the recorded effect no longer produces the recorded
    // successor
    let mut step = good.clone();
    step[steps_at] = step[steps_at].wrapping_add(1);
    match PlanWitnessBytes::parse(&step).unwrap().replay() {
        ReplayVerdict::Invalid { step, reason } => {
            assert_eq!(step, 0);
            assert_eq!(reason, ReplayFault::EffectDoesNotProduceState);
        }
        other => panic!("expected a corrupted step to replay Invalid, got {other:?}"),
    }

    // a corrupted terminal state: the last step no longer lands on the goal
    let mut terminal = good.clone();
    let last = steps_at + 4 * 36;
    terminal[last] = 0;
    terminal[last + 1] = 0;
    terminal[last + 16] = 9;
    match PlanWitnessBytes::parse(&terminal).unwrap().replay() {
        ReplayVerdict::Invalid { reason, .. } => assert_ne!(reason, ReplayFault::GoalNotSatisfied),
        other => panic!("expected a corrupted terminal to replay Invalid, got {other:?}"),
    }
}

#[test]
fn a_witness_beyond_the_frozen_envelope_is_refused_rather_than_truncated() {
    let steps: Vec<WitnessStep> = (0..PLAN_HORIZON_MAX)
        .map(|i| (effect(1, 0), slots(i as i16 + 1, 0), 0, 0))
        .collect();
    let considered: Vec<ConsideredCandidate> = (0..PLAN_HORIZON_MAX * PLAN_ACTIONS_MAX)
        .map(|_| ConsideredCandidate {
            operator: 0,
            rule_row: 0,
            score: 0,
            tie_rank: 0,
            support: 0,
            band: 0,
            flags: 0,
        })
        .collect();
    assert!(
        build_witness(&WitnessDraft {
            slot_count: ARITY,
            initial: slots(0, 0),
            goal: goal_at(PLAN_HORIZON_MAX as i16, 0),
            constraints: &[],
            steps: &steps,
            considered: &considered,
            considered_per_step: PLAN_ACTIONS_MAX as u8,
            decline: None,
            verdict: (0, 0),
        })
        .is_none(),
        "a witness past the frozen envelope must be a capacity decline, not a truncated record"
    );
}

// ---------------------------------------------------------------------------
// Absent-section identity
// ---------------------------------------------------------------------------

#[test]
fn every_new_section_is_optional_and_known() {
    for id in [
        SectionId::PSCH,
        SectionId::PTRN,
        SectionId::PGOL,
        SectionId::PWIT,
    ] {
        assert!(id.is_known(), "the reader must know {id:?}");
        assert!(
            !id.mandatory(),
            "{id:?} must be optional so an older reader skips it"
        );
        assert_ne!(
            id.raw() & SectionId::OPTIONAL_BIT,
            0,
            "{id:?} must carry the optional bit on the wire"
        );
    }
    // The four ids are distinct and do not collide with the existing ones.
    let ids = [
        SectionId::PSTATE.raw(),
        SectionId::SKMX.raw(),
        SectionId::PSIB.raw(),
        SectionId::PSCH.raw(),
        SectionId::PTRN.raw(),
        SectionId::PGOL.raw(),
        SectionId::PWIT.raw(),
    ];
    let mut sorted = ids.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "section ids must be distinct");
}

#[test]
fn the_frozen_capacities_are_recorded_in_the_schema_and_enforced_on_read() {
    let raw = schema_bytes();
    assert!(PlanSchema::parse(&raw).is_ok());
    // Every capacity slot in the header is checked, not just the first.
    for (offset, width) in [
        (12usize, 2usize),
        (14, 2),
        (16, 2),
        (18, 2),
        (20, 2),
        (22, 2),
        (24, 4),
        (28, 4),
    ] {
        let mut mutated = raw.clone();
        mutated[offset] = mutated[offset].wrapping_add(1);
        assert!(
            PlanSchema::parse(&mutated).is_err(),
            "a capacity at offset {offset} (width {width}) was not enforced"
        );
    }
    assert_eq!(PLAN_RULES_MAX, 256);
    assert_eq!(PLAN_SLOTS_MAX, 8);
}
