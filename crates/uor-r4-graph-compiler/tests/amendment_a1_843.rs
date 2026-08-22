//! Amendment A1 gate instruments (#843 increment 2).
//!
//! Frozen contract: `docs/bounded_semantic_transitions_spec_843.md` §2, which
//! implements `docs/compositional_planning_spec_844.md` §11 (Amendment A1).
//!
//! These four instruments are the binding gate that #843's run contract places
//! *before* any packed section is written or any arm is fitted. Run against the
//! pre-amendment generator they returned DEGENERATE:
//!
//! - 13 of the 20 frozen (family x horizon) cells were 0/512 solvable;
//! - a structure-keyed memorized-trajectory null saturated at valid-plan rate
//!   1.0000 in every non-vacuous cell, which put the #826 promotion statistic
//!   at or below zero by construction;
//! - five of six split axes had exactly one cell;
//! - the task identity carried the generator seed, so an identity-keyed null
//!   could never fire and read as healthy while the structure-keyed one was
//!   saturated.
//!
//! Certifier-instrument / off-serving-path. Deterministic, teacher-free, and
//! fixture-free: nothing here is deployed-serving evidence.

use std::collections::BTreeMap;

use uor_r4_graph_compiler::compositional_planning as cp;
use uor_r4_graph_compiler::semantic_state::Action;

/// Frozen effect floor (#844 §2.5, D3). Unchanged by Amendment A1.
const DELTA_MIN: f64 = 0.05;
/// Frozen sample size per held-out cell per horizon (#844 §2.5, D4).
const N_PER_CELL: usize = 512;
/// Frozen horizon progression (#844 §2.5, D2).
const FROZEN_HORIZONS: [usize; 4] = [1, 2, 4, 8];
/// A1-b threshold: distinct cells required per split axis per family.
const MIN_AXIS_CARDINALITY: usize = 8;
/// A1-a threshold: solvable fraction required in every frozen cell.
const MIN_SOLVABLE_FRACTION: f64 = 0.5;

const FAMILIES: [cp::TaskFamily; 5] = [
    cp::TaskFamily::GraphNavigation,
    cp::TaskFamily::SymbolicTransformation,
    cp::TaskFamily::ConstraintSatisfaction,
    cp::TaskFamily::MultiHopEvidence,
    cp::TaskFamily::CounterfactualIntervention,
];

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn ints(v: &[f32]) -> Vec<i64> {
    v.iter().map(|x| x.round() as i64).collect()
}

/// The *semantic* content of an instance: goal, forbidden regions, and operator
/// effects. Surface names and the generation seed are deliberately excluded, so
/// a memorizer keyed on this is the strongest one available - it already sees
/// through entity, vocabulary, and template renaming.
fn semantic_key(t: &cp::TaskInstance) -> String {
    let mut forbidden: Vec<Vec<i64>> = t
        .constraints
        .iter()
        .map(|c| ints(&c.forbidden_region.center))
        .collect();
    forbidden.sort();
    let deltas: Vec<Vec<i64>> = t.actions.iter().map(|a| ints(&a.delta_vector)).collect();
    format!(
        "{}|{:?}|{:?}|{:?}",
        t.family.label(),
        ints(&t.goal.target_region.center),
        forbidden,
        deltas
    )
}

/// A plan as a sequence of operator *effects*. This is the strongest form a
/// memorised or retrieved plan can take: it is invariant to operator naming and
/// to operator ordering, so replaying it transfers by semantics rather than by
/// label or slot. It fails only when the target instance does not offer the
/// effect - which is exactly the generalization the topology axis tests.
fn plan_effects(t: &cp::TaskInstance) -> Vec<Vec<i64>> {
    t.gold
        .chosen_path
        .iter()
        .map(|a| ints(&a.delta_vector))
        .collect()
}

fn replay_is_valid(t: &cp::TaskInstance, effects: &[Vec<i64>]) -> bool {
    if effects.is_empty() {
        return false;
    }
    let mut submitted: Vec<Action> = Vec::with_capacity(effects.len());
    for effect in effects {
        match t.actions.iter().find(|a| ints(&a.delta_vector) == *effect) {
            Some(a) => submitted.push(a.clone()),
            None => return false,
        }
    }
    cp::verify_submission(t, &submitted) == cp::WitnessVerdict::Valid
}

/// The outcome statistic the gate reads: a valid plan on a solvable instance,
/// or a correct honest decline on one with no plan inside the horizon. On every
/// cell whose instances are all solvable this is exactly the frozen valid-plan
/// rate; on the horizon-1 honest-decline cell it is what separates planning
/// from a baseline that always answers.
fn outcome_correct(t: &cp::TaskInstance, emitted: &Option<Vec<Vec<i64>>>) -> bool {
    let unsolvable = t.gold.decline.is_some();
    match (emitted, unsolvable) {
        (None, true) => true,
        (None, false) => false,
        (Some(_), true) => false,
        (Some(plan), false) => replay_is_valid(t, plan),
    }
}

/// The topology digit of a seed - the *semantic* split axis. Fitting takes the
/// low half, held-out the high half, so held-out cells never share a dynamics
/// configuration with fitting data.
fn topology_digit(seed: u64) -> u64 {
    let c = cp::AXIS_CARDINALITY;
    (seed / (c * c)) % c
}

fn seeds_where(high_half: bool, n: usize) -> Vec<u64> {
    let mut out = Vec::with_capacity(n);
    let mut seed = 0u64;
    while out.len() < n {
        if (topology_digit(seed) >= cp::AXIS_CARDINALITY / 2) == high_half {
            out.push(seed);
        }
        seed += 1;
    }
    out
}

struct NullRates {
    memorized: f64,
    retrieval: f64,
    trivial_prior: f64,
}

impl NullRates {
    fn strongest(&self) -> f64 {
        self.memorized.max(self.retrieval).max(self.trivial_prior)
    }
}

/// Fit the three computable non-oracle nulls on the fitting half of the
/// *topology* axis - the semantic one - and score them on the held-out half,
/// so a held-out cell never shares an operator effect set or a forbidden
/// configuration with fitting data.
fn null_rates(family: cp::TaskFamily, horizon: usize) -> NullRates {
    let fit_seeds = seeds_where(false, N_PER_CELL);
    let eval_seeds = seeds_where(true, N_PER_CELL);

    type Emission = Option<Vec<Vec<i64>>>;
    let mut by_key: BTreeMap<String, Emission> = BTreeMap::new();
    let mut by_goal: Vec<(Vec<i64>, Emission)> = Vec::new();
    let mut frequency: BTreeMap<Emission, usize> = BTreeMap::new();
    for s in &fit_seeds {
        let t = cp::generate(family, *s, horizon);
        // Declines are fitted too, so the controls are not strawmen: a baseline
        // that has seen honest declines in fitting data can emit them.
        let emission: Emission = if t.gold.decline.is_some() {
            None
        } else {
            Some(plan_effects(&t))
        };
        by_key
            .entry(semantic_key(&t))
            .or_insert_with(|| emission.clone());
        by_goal.push((ints(&t.goal.target_region.center), emission.clone()));
        *frequency.entry(emission).or_default() += 1;
    }
    // Ties break by the canonical plan order, never by hash-iteration order.
    let modal: Emission = frequency
        .iter()
        .max_by_key(|(plan, count)| (**count, std::cmp::Reverse((*plan).clone())))
        .map(|(plan, _)| plan.clone())
        .unwrap_or(None);

    let (mut memorized, mut retrieval, mut trivial, mut total) = (0usize, 0usize, 0usize, 0usize);
    for s in &eval_seeds {
        let t = cp::generate(family, *s, horizon);
        total += 1;

        // N3 memorized-trajectory: emit by semantic key, falling back to the
        // modal fitting emission so the control can still fire off-key.
        let remembered = by_key.get(&semantic_key(&t)).unwrap_or(&modal);
        if outcome_correct(&t, remembered) {
            memorized += 1;
        }

        // N1 retrieval-only: nearest fitting instance by goal displacement.
        let goal = ints(&t.goal.target_region.center);
        let nearest = by_goal
            .iter()
            .min_by_key(|(g, _)| {
                g.iter()
                    .zip(goal.iter())
                    .map(|(a, b)| (a - b).abs())
                    .sum::<i64>()
            })
            .map(|(_, p)| p.clone())
            .unwrap_or(None);
        if outcome_correct(&t, &nearest) {
            retrieval += 1;
        }

        // trivial-prior: always emit the modal fitting emission.
        if outcome_correct(&t, &modal) {
            trivial += 1;
        }
    }

    let rate = |k: usize| {
        if total == 0 {
            1.0
        } else {
            k as f64 / total as f64
        }
    };
    NullRates {
        memorized: rate(memorized),
        retrieval: rate(retrieval),
        trivial_prior: rate(trivial),
    }
}

// ---------------------------------------------------------------------------
// A1 gate instruments
// ---------------------------------------------------------------------------

/// A1-a: every one of the 20 frozen (family x horizon) cells is non-vacuous.
#[test]
fn a1_a_non_vacuity_per_frozen_cell() {
    let mut failures = Vec::new();
    for horizon in FROZEN_HORIZONS {
        for family in FAMILIES {
            let solvable = (0..N_PER_CELL as u64)
                .filter(|s| cp::generate(family, *s, horizon).gold.decline.is_none())
                .count();
            let fraction = solvable as f64 / N_PER_CELL as f64;
            println!(
                "A1-a H={horizon:<2} {:<28} solvable {solvable:>3}/{N_PER_CELL} = {fraction:.4}",
                family.label()
            );
            if fraction < MIN_SOLVABLE_FRACTION {
                failures.push(format!("{} H={horizon}: {fraction:.4}", family.label()));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "vacuous frozen cells (a cell that cannot separate any mechanism): {failures:?}"
    );
}

/// A1-b: every split axis a seed varies has at least eight distinct cells, so a
/// disjoint fitting/held-out partition is constructible rather than vacuous.
#[test]
fn a1_b_axis_cardinality() {
    for family in FAMILIES {
        let mut entity = std::collections::BTreeSet::new();
        let mut vocabulary = std::collections::BTreeSet::new();
        let mut topology = std::collections::BTreeSet::new();
        let mut template = std::collections::BTreeSet::new();
        let mut composition = std::collections::BTreeSet::new();
        for horizon in FROZEN_HORIZONS {
            for seed in 0..cp::AXIS_CARDINALITY.pow(4) {
                let t = cp::generate(family, seed, horizon);
                entity.insert(t.initial_state.id.clone());
                template.insert(t.goal.name.clone());
                vocabulary.insert(
                    t.actions
                        .iter()
                        .map(|a| a.name.clone())
                        .collect::<Vec<_>>()
                        .join(">"),
                );
                let mut forbidden: Vec<Vec<i64>> = t
                    .constraints
                    .iter()
                    .map(|c| ints(&c.forbidden_region.center))
                    .collect();
                forbidden.sort();
                let deltas: Vec<Vec<i64>> =
                    t.actions.iter().map(|a| ints(&a.delta_vector)).collect();
                topology.insert(format!("{forbidden:?}|{deltas:?}"));
                composition.insert(
                    t.gold
                        .chosen_path
                        .iter()
                        .map(|a| a.name.clone())
                        .collect::<Vec<_>>()
                        .join(">"),
                );
            }
        }
        println!(
            "A1-b {:<28} entity={} vocabulary={} topology={} template={} composition={}",
            family.label(),
            entity.len(),
            vocabulary.len(),
            topology.len(),
            template.len(),
            composition.len()
        );
        for (axis, n) in [
            ("by_entity", entity.len()),
            ("by_vocabulary", vocabulary.len()),
            ("by_topology", topology.len()),
            ("by_template", template.len()),
            ("by_operator_composition", composition.len()),
        ] {
            assert!(
                n >= MIN_AXIS_CARDINALITY,
                "{} axis {axis} has {n} cells; a split on it would be vacuous",
                family.label()
            );
        }
    }
}

/// A1-c: the identity is content-derived, so an identity-keyed control can fire.
#[test]
fn a1_c_identity_is_content_derived() {
    let period = cp::AXIS_CARDINALITY.pow(4);
    for family in FAMILIES {
        for horizon in FROZEN_HORIZONS {
            let a = cp::generate(family, 5, horizon);
            let mut matched = false;
            for k in 1..128u64 {
                let b = cp::generate(family, 5 + k * period, horizon);
                if semantic_key(&b) == semantic_key(&a)
                    && b.initial_state.id == a.initial_state.id
                    && b.goal.name == a.goal.name
                    && b.actions.iter().map(|x| x.name.clone()).collect::<Vec<_>>()
                        == a.actions.iter().map(|x| x.name.clone()).collect::<Vec<_>>()
                {
                    assert_ne!(a.seed, b.seed);
                    assert_eq!(
                        a.id(),
                        b.id(),
                        "{} H={horizon}: structurally identical instances must share an id",
                        family.label()
                    );
                    matched = true;
                    break;
                }
            }
            assert!(
                matched,
                "{} H={horizon}: expected a structurally identical instance at another seed",
                family.label()
            );
        }
    }
}

/// A1-d: the strongest non-oracle null leaves at least delta_min of headroom in
/// every frozen cell. Without this the promotion statistic of #826 is at or
/// below zero by construction and no mechanism can ever clear the gate.
#[test]
fn a1_d_strongest_null_headroom() {
    let ceiling = 1.0 - DELTA_MIN;
    let mut failures = Vec::new();
    for horizon in FROZEN_HORIZONS {
        for family in FAMILIES {
            let rates = null_rates(family, horizon);
            let headroom = 1.0 - rates.strongest();
            println!(
                "A1-d H={horizon:<2} {:<28} memorized={:.4} retrieval={:.4} trivial={:.4} strongest={:.4} headroom={headroom:.4}{}",
                family.label(),
                rates.memorized,
                rates.retrieval,
                rates.trivial_prior,
                rates.strongest(),
                if headroom < 2.0 * DELTA_MIN {
                    "  <- TIGHT (under 2x delta_min)"
                } else {
                    ""
                }
            );
            if rates.strongest() > ceiling {
                failures.push(format!(
                    "{} H={horizon}: strongest null {:.4}",
                    family.label(),
                    rates.strongest()
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "saturated non-oracle nulls leave no headroom above delta_min: {failures:?}"
    );
}

/// Amendment A1 repairs the generator; it moves no number the constitution froze.
#[test]
fn a1_does_not_move_a_frozen_number() {
    assert_eq!(cp::H_MAX, 16, "H_max is frozen (#844 D2)");
    assert_eq!(cp::W_MAX, 64, "W_max is frozen (#844 D2)");
    assert_eq!(DELTA_MIN, 0.05, "delta_min is frozen (#844 D3)");
    assert_eq!(N_PER_CELL, 512, "n per cell is frozen (#844 D4)");
    assert_eq!(FROZEN_HORIZONS, [1, 2, 4, 8], "the horizon grid is frozen");
}
