//! Typed transition observations and deterministic induction — the machine-
//! checked record for #843 increment 3.
//!
//! Frozen contract: `docs/bounded_semantic_transitions_spec_843.md` §3. Every
//! guarantee that section states is asserted here. Compiler /
//! off-serving-path: nothing in this file is deployed-serving evidence and no
//! result here establishes any planning capability.

use std::collections::BTreeSet;

use uor_r4_graph_compiler::compositional_planning as cp;
use uor_r4_graph_compiler::semantic_transitions as st;
use uor_r4_graph_format::plan::{EffectDelta, PLAN_RULES_MAX, PreconditionMask};

/// Enough seeds to span the whole fitting half of the topology axis: the
/// topology digit is `(seed / 64) % 8`, so the low four cells need 256 seeds.
const FITTING_SEEDS: u64 = 256;

const FAMILIES: [cp::TaskFamily; 5] = [
    cp::TaskFamily::GraphNavigation,
    cp::TaskFamily::SymbolicTransformation,
    cp::TaskFamily::ConstraintSatisfaction,
    cp::TaskFamily::MultiHopEvidence,
    cp::TaskFamily::CounterfactualIntervention,
];

/// Observations over the fitting half of the topology axis.
fn fitting_observations(family: cp::TaskFamily, seeds: u64) -> Vec<st::TransitionObservation> {
    let policy = st::SplitPolicy::topology_halves();
    let mut out = Vec::new();
    for seed in 0..seeds {
        let task = cp::generate(family, seed, 8);
        if !policy.admits(&task.split_cell()) {
            continue;
        }
        out.extend(st::observe(&task));
    }
    out
}

// ---------------------------------------------------------------------------
// The observation record
// ---------------------------------------------------------------------------

/// Negative and forbidden outcomes are first class: a set of purely positive
/// examples cannot express a precondition, so the observer must emit them.
#[test]
fn negative_and_forbidden_outcomes_are_recorded_alongside_applied_ones() {
    // Graph navigation topology cell 0 keeps a forbidden cell on the lattice.
    let task = cp::generate(cp::TaskFamily::GraphNavigation, 0, 8);
    assert!(
        !task.constraints.is_empty(),
        "the fixture declares a boundary"
    );
    let observations = st::observe(&task);
    assert!(!observations.is_empty());

    let applied = observations
        .iter()
        .filter(|o| o.outcome == st::Outcome::Applied)
        .count();
    let forbidden = observations
        .iter()
        .filter(|o| o.outcome == st::Outcome::ForbiddenRegion)
        .count();
    assert!(applied > 0, "some operator applied");
    assert!(
        forbidden > 0,
        "the observation pass must probe the declared boundary"
    );

    for observation in &observations {
        assert_eq!(
            observation.polarity,
            st::polarity_of(observation.outcome),
            "polarity follows the outcome"
        );
        match observation.outcome {
            st::Outcome::Applied => {
                assert!(observation.to_slots.is_some());
                assert!(observation.effect_delta.is_some());
            }
            _ => {
                assert!(observation.to_slots.is_none());
                assert!(observation.effect_delta.is_none());
            }
        }
    }
}

/// A forbidden outcome records that the destination's slots determined it; an
/// applied one claims no read mask, because the operator is unconditional.
#[test]
fn an_observation_records_the_slots_its_outcome_depended_on() {
    let task = cp::generate(cp::TaskFamily::GraphNavigation, 0, 8);
    for observation in st::observe(&task) {
        match observation.outcome {
            st::Outcome::ForbiddenRegion => assert_ne!(observation.read_mask, 0),
            _ => assert_eq!(observation.read_mask, 0),
        }
    }
}

/// **Guarantee (no future-answer field). Status: Structural.** An observation
/// describes one attempted step. No field, and no combination of fields,
/// reproduces the gold plan or the gold terminal state.
#[test]
fn no_observation_exposes_the_gold_plan_or_terminal_state() {
    for family in FAMILIES {
        let task = cp::generate(family, 3, 8);
        let gold: Vec<String> = task
            .gold
            .chosen_path
            .iter()
            .map(|a| a.name.clone())
            .collect();
        assert_eq!(gold.len(), 8, "the fixture has a non-trivial gold plan");
        let observations = st::observe(&task);

        // No single observation carries more than one step.
        for observation in &observations {
            assert!(
                observation.to_slots.is_none()
                    || observation.effect_delta.is_some_and(|e| e.len() <= 8),
                "an observation records one step"
            );
        }

        // The gold terminal state is not recoverable: the observation set is
        // the same whichever goal cell the instance carries, because the
        // dynamics do not depend on the goal. Two instances that differ only in
        // their goal produce observation sets whose typed transitions agree.
        let mut with_goal = cp::generate(family, 3, 8);
        let other = cp::generate(family, 3 + cp::AXIS_CARDINALITY.pow(4), 8);
        with_goal.goal = other.goal.clone();
        let transitions = |set: &[st::TransitionObservation]| -> BTreeSet<String> {
            set.iter()
                .map(|o| {
                    format!(
                        "{:?}|{}|{:?}",
                        o.from_slots.as_slice(),
                        o.operator,
                        o.effect_delta.map(|e| e.as_slice().to_vec())
                    )
                })
                .collect()
        };
        assert_eq!(
            transitions(&observations),
            transitions(&st::observe(&with_goal)),
            "{}: the observed dynamics must not depend on the goal",
            family.label()
        );
    }
}

/// The content identity is derived from typed content only — no seed, clock,
/// RNG, or hash-iteration order — so identical typed observations collapse.
#[test]
fn observation_identity_is_content_addressed() {
    let task = cp::generate(cp::TaskFamily::GraphNavigation, 0, 8);
    let first = st::observe(&task);
    let second = st::observe(&task);
    let ids = |set: &[st::TransitionObservation]| -> Vec<u64> {
        set.iter().map(|o| o.sample_id()).collect()
    };
    assert_eq!(ids(&first), ids(&second));
    assert!(
        first
            .iter()
            .map(|o| o.sample_id())
            .collect::<BTreeSet<_>>()
            .len()
            > 1,
        "distinct typed situations get distinct ids"
    );
}

// ---------------------------------------------------------------------------
// Induction
// ---------------------------------------------------------------------------

/// **Guarantee (induction determinism). Status: Structural.** The same rule set
/// byte-for-byte from reordered input, from a different shard count, and across
/// repeated runs.
#[test]
fn induction_is_deterministic_across_order_and_shard_count() {
    let policy = st::SplitPolicy::topology_halves();
    for family in FAMILIES {
        let observations = fitting_observations(family, FITTING_SEEDS);
        let baseline = st::induce(&observations, &policy);
        let set = baseline
            .induced()
            .unwrap_or_else(|| panic!("{}: {baseline:?}", family.label()));

        let repeated = st::induce(&observations, &policy);
        assert_eq!(
            set.canonical_bytes(),
            repeated.induced().unwrap().canonical_bytes(),
            "{}: repeated runs differ",
            family.label()
        );

        let mut reversed = observations.clone();
        reversed.reverse();
        assert_eq!(
            set.canonical_bytes(),
            st::induce(&reversed, &policy)
                .induced()
                .unwrap()
                .canonical_bytes(),
            "{}: input order changed the rule set",
            family.label()
        );

        for shards in [1u32, 3, 16, 97] {
            let sharded = st::induce_with_shards(&observations, &policy, shards);
            assert_eq!(
                set.content_id(),
                sharded.induced().unwrap().content_id(),
                "{}: shard count {shards} changed the rule set",
                family.label()
            );
        }
    }
}

/// **Guarantee (no evaluation leakage). Status: Structural.** A held-out
/// observation reaching the inducer is refused by name, and the emitted set
/// attributes support to fitting cells only.
#[test]
fn held_out_observations_are_refused_and_never_contribute() {
    let policy = st::SplitPolicy::topology_halves();
    let observations = fitting_observations(cp::TaskFamily::GraphNavigation, FITTING_SEEDS);
    let set = st::induce(&observations, &policy)
        .induced()
        .unwrap()
        .clone();
    for cell in &set.fitting_cells {
        assert!(
            !policy.held_out_topologies.contains(cell),
            "a held-out cell contributed support"
        );
    }

    // Plant one held-out observation and the scan must fire.
    let held_out = cp::generate(cp::TaskFamily::GraphNavigation, 4 * 64, 8);
    assert!(!policy.admits(&held_out.split_cell()));
    let mut leaky = observations.clone();
    leaky.extend(st::observe(&held_out));
    match st::induce(&leaky, &policy) {
        st::InductionOutcome::Refused(st::RefusalReason::HeldOutLeakage { cell }) => {
            assert!(policy.held_out_topologies.contains(&cell.topology));
        }
        other => panic!("expected a held-out leakage refusal, got {other:?}"),
    }
}

/// A sealed cell is refused exactly like a held-out one, so the partitions
/// reserved for the final verdict are never opened by this issue.
#[test]
fn sealed_cells_are_refused() {
    let policy = st::SplitPolicy::topology_halves().sealing([0u8]);
    let observations = fitting_observations(cp::TaskFamily::GraphNavigation, FITTING_SEEDS);
    assert!(matches!(
        st::induce(&observations, &policy),
        st::InductionOutcome::Refused(st::RefusalReason::HeldOutLeakage { .. })
    ));
}

/// A declared conflict is recorded and its effects are **not** emitted, so
/// reaching one at plan time is a decline rather than a majority vote.
#[test]
fn a_declared_conflict_is_recorded_and_excluded_from_the_rule_table() {
    let policy = st::SplitPolicy::topology_halves();
    let observations = fitting_observations(cp::TaskFamily::GraphNavigation, FITTING_SEEDS);
    let clean = st::induce(&observations, &policy)
        .induced()
        .unwrap()
        .clone();
    assert!(clean.conflicts.is_empty(), "the fixture is consistent");

    // Plant a disagreement: the same operator, from the same typed state, in
    // the same topology cell, observed producing a different successor.
    let mut planted = observations.clone();
    let source = planted
        .iter()
        .find(|o| o.outcome == st::Outcome::Applied)
        .cloned()
        .expect("an applied observation to contradict");
    let contradicted_effect = source.effect_delta.unwrap();
    let mut contradiction = source.clone();
    let bogus = EffectDelta::from_slice(&[99, -99]).unwrap();
    contradiction.effect_delta = Some(bogus);
    contradiction.to_slots = Some(source.from_slots.apply(&bogus).unwrap());
    planted.push(contradiction);

    let outcome = st::induce(&planted, &policy);
    let set = outcome
        .induced()
        .expect("a conflict is recorded, not fatal");
    assert!(
        !set.conflicts.is_empty(),
        "the disagreement must be declared"
    );
    let conflicted = &set.conflicts[0];
    assert_eq!(conflicted.from_slots, source.from_slots);
    assert!(conflicted.effects.contains(&bogus));
    assert!(conflicted.effects.contains(&contradicted_effect));
    for rule in &set.rules {
        assert_ne!(
            rule.effect, bogus,
            "a conflicted effect must not be emitted"
        );
        assert_ne!(
            rule.effect, contradicted_effect,
            "both sides of a conflict are withheld"
        );
    }
}

/// Support raises the ordinal band; the band is never read as a probability.
#[test]
fn support_maps_to_an_ordinal_band() {
    let policy = st::SplitPolicy::topology_halves();
    let observations = fitting_observations(cp::TaskFamily::GraphNavigation, FITTING_SEEDS);
    let set = st::induce(&observations, &policy)
        .induced()
        .unwrap()
        .clone();
    assert!(!set.rules.is_empty());
    let mut sorted = set.rules.clone();
    sorted.sort_by_key(|r| r.support);
    for window in sorted.windows(2) {
        assert!(
            window[0].band <= window[1].band,
            "the band must be monotone in support"
        );
    }
}

/// The forbidden regions of a task are a predicate over the destination, not a
/// property of the operator, so they must not be baked into the dynamics.
#[test]
fn constraints_do_not_leak_into_the_induced_dynamics() {
    let policy = st::SplitPolicy::topology_halves();
    let observations = fitting_observations(cp::TaskFamily::ConstraintSatisfaction, FITTING_SEEDS);
    assert!(
        observations
            .iter()
            .any(|o| o.outcome == st::Outcome::ForbiddenRegion),
        "the fixture probes its declared boundary"
    );
    let outcome = st::induce(&observations, &policy);
    let set = outcome.induced().expect("a probed boundary induces");

    // No rule became conditional on the state it happened to be observed from.
    for rule in &set.rules {
        assert_eq!(
            rule.precondition,
            PreconditionMask::unconditional(),
            "a forbidden region turned into an operator precondition"
        );
    }

    // The induced effect vocabulary is exactly the declared operator effects:
    // the boundary added none and removed none.
    let declared: BTreeSet<Vec<i16>> = observations
        .iter()
        .map(|o| o.declared_effect.as_slice().to_vec())
        .collect();
    let induced: BTreeSet<Vec<i16>> = set
        .rules
        .iter()
        .map(|r| r.effect.as_slice().to_vec())
        .collect();
    assert_eq!(
        induced, declared,
        "the induced dynamics differ from the declared operator effects"
    );
}

/// The negative-evidence floor catches a sampling failure — a declared boundary
/// that was never probed — and does not fire where no boundary exists.
#[test]
fn the_negative_floor_fires_only_where_a_boundary_existed() {
    let policy = st::SplitPolicy::topology_halves();

    // Constrained family, boundary observations stripped: refused.
    let constrained = fitting_observations(cp::TaskFamily::ConstraintSatisfaction, FITTING_SEEDS);
    assert!(constrained.iter().any(|o| !o.constraint_refs.is_empty()));
    let unprobed: Vec<st::TransitionObservation> = constrained
        .iter()
        .filter(|o| o.outcome != st::Outcome::ForbiddenRegion)
        .cloned()
        .collect();
    match st::induce(&unprobed, &policy) {
        st::InductionOutcome::Refused(st::RefusalReason::InsufficientNegatives {
            observed_milli,
            floor_milli,
        }) => {
            assert_eq!(observed_milli, 0);
            assert_eq!(floor_milli, st::NEGATIVE_FRACTION_FLOOR_MILLI);
        }
        other => panic!("expected an insufficient-negatives refusal, got {other:?}"),
    }

    // Unconstrained family: the dynamics really are total, so the floor does
    // not apply and induction proceeds.
    let unconstrained = fitting_observations(cp::TaskFamily::SymbolicTransformation, FITTING_SEEDS);
    assert!(unconstrained.iter().all(|o| o.constraint_refs.is_empty()));
    assert!(st::induce(&unconstrained, &policy).induced().is_some());
}

/// Every refusal is typed and inspectable; none is a silent empty result.
#[test]
fn an_empty_observation_set_induces_an_empty_rule_set_rather_than_guessing() {
    let policy = st::SplitPolicy::topology_halves();
    let set = st::induce(&[], &policy).induced().unwrap().clone();
    assert!(set.rules.is_empty());
    assert!(set.conflicts.is_empty());
    assert_eq!(set.observations, 0);
    assert_eq!(set.negative_fraction_milli, 0);
}

/// The emitted set stays inside the frozen capacities.
#[test]
fn the_induced_set_stays_within_the_frozen_capacities() {
    let policy = st::SplitPolicy::topology_halves();
    for family in FAMILIES {
        let set = st::induce(&fitting_observations(family, FITTING_SEEDS), &policy)
            .induced()
            .unwrap()
            .clone();
        assert!(
            set.rules.len() <= PLAN_RULES_MAX,
            "{}: {} rules",
            family.label(),
            set.rules.len()
        );
        println!(
            "{:<28} rules={:<4} conflicts={:<3} observations={:<6} negatives={:<5} ({}‰) cells={:?}",
            family.label(),
            set.rules.len(),
            set.conflicts.len(),
            set.observations,
            set.negatives,
            set.negative_fraction_milli,
            set.fitting_cells
        );
    }
}
