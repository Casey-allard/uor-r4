//! #845 increment 3 — the reference ordering skeleton, its deployed-parity
//! proof-by-test, the arm/control roster, and the equal-bytes audit, per
//! `docs/w33_geometry_qualification_spec_845.md` §4 and §4-A.
//!
//! The load-bearing property is counter-exact parity: the reference skeleton
//! with the incumbent scorer must reproduce the deployed planner — outcome,
//! plan, and every `PlanCounters` field — episode by episode, for both the
//! breadth-first and the beam retention rules. Everything increment 4
//! measures rests on that equivalence.

mod support;

use support::arms;
use support::episode::{self, Packed};
use support::ordering::{GoalDistanceScorer, RefScratch, Scorer, SeamMode};
use support::w33;
use uor_r4_graph_compiler::compositional_planning as cp;
use uor_r4_graph_runtime::plan::{PlanBudget, PlanScratch, PlanStrategy};

const PARITY_HORIZONS: [usize; 3] = [2, 4, 8];
const PARITY_SEEDS: usize = 32;

fn frozen_with(horizon: usize) -> PlanBudget {
    PlanBudget {
        horizon: horizon as u8,
        ..PlanBudget::frozen()
    }
}

fn parity_budgets(horizon: usize) -> Vec<(String, PlanBudget)> {
    vec![
        ("frozen".to_string(), frozen_with(horizon)),
        (
            "frontier-16".to_string(),
            PlanBudget {
                frontier: 16,
                ..frozen_with(horizon)
            },
        ),
        (
            "frontier-8".to_string(),
            PlanBudget {
                frontier: 8,
                ..frozen_with(horizon)
            },
        ),
        (
            "expansions-64".to_string(),
            PlanBudget {
                max_expansions: 64,
                ..frozen_with(horizon)
            },
        ),
    ]
}

fn fitted_cell(family: cp::TaskFamily, horizon: usize) -> Option<Packed> {
    let set = episode::induce_for(family, horizon)?;
    episode::pack(&set, 0)
}

/// Reference FIFO retention == deployed breadth-first: same emission, same
/// counters, every episode of the parity grid.
#[test]
fn reference_fifo_reproduces_deployed_breadth_first() {
    parity_grid(PlanStrategy::BreadthFirst, false);
}

/// Reference goal-distance retention == deployed table-guided beam.
#[test]
fn reference_goal_distance_reproduces_deployed_beam() {
    parity_grid(PlanStrategy::BestFirstBeam, true);
}

fn parity_grid(strategy: PlanStrategy, ordered: bool) {
    let mut deployed_scratch = Box::new(PlanScratch::new());
    let mut reference_scratch = RefScratch::new();
    let mut episodes = 0usize;
    for family in episode::SEPARATING {
        for horizon in PARITY_HORIZONS {
            let Some(packed) = fitted_cell(family, horizon) else {
                panic!("{}: induction failed at H={horizon}", family.label());
            };
            for (label, budget) in parity_budgets(horizon) {
                for seed in episode::seeds(true, PARITY_SEEDS) {
                    let Some(task) = episode::try_generate(family, seed, horizon) else {
                        continue;
                    };
                    let (deployed_emission, deployed_counters) = episode::run_deployed(
                        strategy,
                        &packed,
                        &task,
                        budget,
                        &mut deployed_scratch,
                    )
                    .expect("deployed episode runs");
                    let predicate_bytes = episode::predicates_for(&task).unwrap();
                    let schema =
                        uor_r4_graph_format::plan_sections::PlanSchema::parse(&packed.schema)
                            .unwrap();
                    let predicates = uor_r4_graph_format::plan_sections::PredicateSet::parse(
                        &predicate_bytes,
                        &schema,
                    )
                    .unwrap();
                    let mut scorer = GoalDistanceScorer::new(&predicates);
                    let (reference_emission, reference_counters) = episode::run_reference(
                        &packed,
                        &task,
                        budget,
                        SeamMode::Parity(ordered),
                        &mut scorer,
                        &mut reference_scratch,
                    )
                    .expect("reference episode runs");
                    assert_eq!(
                        deployed_emission,
                        reference_emission,
                        "{} H={horizon} {label} seed={seed}: emission diverged",
                        family.label()
                    );
                    assert_eq!(
                        deployed_counters,
                        reference_counters,
                        "{} H={horizon} {label} seed={seed}: counters diverged",
                        family.label()
                    );
                    episodes += 1;
                }
            }
        }
    }
    assert!(episodes >= 1000, "the parity grid must not be vacuous");
}

/// Every arm is deterministic across independent constructions and can
/// separate states (at least two distinct scores over a successor sample) —
/// a control unable to fire or to fail voids a cell, never passes it.
#[test]
fn every_arm_is_deterministic_and_can_separate() {
    let tables = arms::W33Tables::build();
    let tables_again = arms::W33Tables::build();
    let observations = episode::fitting_observations(cp::TaskFamily::GraphNavigation, 8);
    let learned = arms::LearnedTable::fit(&tables, &observations.remaining);
    let learned_again = arms::LearnedTable::fit(&tables_again, &observations.remaining);
    let spectral = arms::SpectralEmbedding::fit(&tables, &observations.transitions);
    let spectral_again = arms::SpectralEmbedding::fit(&tables_again, &observations.transitions);
    let vsa = arms::VsaTables::build();
    let random = arms::RandomTables::build();
    let relabeled = arms::RelabeledTables::build(&tables);

    let goal = (6i16, 4i16);
    let sample: Vec<uor_r4_graph_format::plan::SlotVec> = (0..24)
        .map(|i| {
            uor_r4_graph_format::plan::SlotVec::from_slice(&[i % 9, (i * 5 + 2) % 11]).unwrap()
        })
        .collect();

    let mut roster: Vec<Box<dyn Scorer>> = vec![
        Box::new(arms::GeometryScorer::new(&tables, goal)),
        Box::new(arms::HammingScorer::new(goal)),
        Box::new(arms::LearnedScorer::new(&tables, &learned, goal)),
        Box::new(arms::VsaScorer::new(&vsa, goal)),
        Box::new(arms::SpectralScorer::new(&tables, &spectral, goal)),
        Box::new(arms::RandomScorer::new(&tables, &random, goal)),
        Box::new(arms::RelabeledScorer::new(&tables, &relabeled, goal)),
        Box::new(arms::PhasePermutedScorer::new(&tables, goal)),
    ];
    let mut roster_again: Vec<Box<dyn Scorer>> = vec![
        Box::new(arms::GeometryScorer::new(&tables_again, goal)),
        Box::new(arms::HammingScorer::new(goal)),
        Box::new(arms::LearnedScorer::new(
            &tables_again,
            &learned_again,
            goal,
        )),
        Box::new(arms::VsaScorer::new(&vsa, goal)),
        Box::new(arms::SpectralScorer::new(
            &tables_again,
            &spectral_again,
            goal,
        )),
        Box::new(arms::RandomScorer::new(&tables_again, &random, goal)),
        Box::new(arms::RelabeledScorer::new(&tables_again, &relabeled, goal)),
        Box::new(arms::PhasePermutedScorer::new(&tables_again, goal)),
    ];

    for (arm, twin) in roster.iter_mut().zip(roster_again.iter_mut()) {
        let scores: Vec<i32> = sample.iter().map(|s| arm.score(s)).collect();
        let twin_scores: Vec<i32> = sample.iter().map(|s| twin.score(s)).collect();
        assert_eq!(scores, twin_scores, "{}: not deterministic", arm.name());
        let mut distinct = scores.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert!(
            distinct.len() >= 2,
            "{}: cannot separate the successor sample",
            arm.name()
        );
    }
}

/// The relabel scramble must NOT be a collinearity automorphism, or the
/// control is vacuous by construction.
#[test]
fn the_relabel_scramble_is_not_an_automorphism() {
    let tables = arms::W33Tables::build();
    let relabeled = arms::RelabeledTables::build(&tables);
    assert!(
        !relabeled.is_automorphism(&tables),
        "the pinned scramble preserves d_W everywhere — pick a different seed"
    );
}

/// Equal-bytes audit: no control consults more auxiliary table bytes than the
/// geometry arm's budget; the geometry arm consumes its budget exactly.
#[test]
fn the_byte_parity_audit_holds() {
    let tables = arms::W33Tables::build();
    let observations = episode::fitting_observations(cp::TaskFamily::GraphNavigation, 8);
    let learned = arms::LearnedTable::fit(&tables, &observations.remaining);
    let spectral = arms::SpectralEmbedding::fit(&tables, &observations.transitions);
    let vsa = arms::VsaTables::build();
    let random = arms::RandomTables::build();
    let relabeled = arms::RelabeledTables::build(&tables);
    let goal = (0i16, 0i16);
    let roster: Vec<Box<dyn Scorer>> = vec![
        Box::new(arms::GeometryScorer::new(&tables, goal)),
        Box::new(arms::HammingScorer::new(goal)),
        Box::new(arms::LearnedScorer::new(&tables, &learned, goal)),
        Box::new(arms::VsaScorer::new(&vsa, goal)),
        Box::new(arms::SpectralScorer::new(&tables, &spectral, goal)),
        Box::new(arms::RandomScorer::new(&tables, &random, goal)),
        Box::new(arms::RelabeledScorer::new(&tables, &relabeled, goal)),
        Box::new(arms::PhasePermutedScorer::new(&tables, goal)),
    ];
    for arm in &roster {
        assert!(
            arm.table_bytes() <= arms::GEOMETRY_TABLE_BYTES,
            "{}: {} auxiliary bytes exceed the geometry budget {}",
            arm.name(),
            arm.table_bytes(),
            arms::GEOMETRY_TABLE_BYTES
        );
    }
    assert_eq!(
        roster[0].table_bytes(),
        arms::GEOMETRY_TABLE_BYTES,
        "the geometry arm defines the budget exactly"
    );
    assert_eq!(w33::POINTS, 40, "the budget is over the 40-point tables");
}

/// The retention seam is live: in a tightened-frontier cell, geometry-ordered
/// retention and FIFO retention must actually diverge somewhere — otherwise
/// increment 4 would be comparing an arm against itself.
#[test]
fn the_ordering_seam_is_live() {
    let family = cp::TaskFamily::GraphNavigation;
    let horizon = 8usize;
    let packed = fitted_cell(family, horizon).expect("induction at H=8");
    let tables = arms::W33Tables::build();
    let budget = PlanBudget {
        frontier: 8,
        ..frozen_with(horizon)
    };
    let mut reference_scratch = RefScratch::new();
    let mut diverged = 0usize;
    let mut episodes = 0usize;
    for seed in episode::seeds(true, 64) {
        let Some(task) = episode::try_generate(family, seed, horizon) else {
            continue;
        };
        episodes += 1;
        let goal = episode::goal_center(&task);
        let mut geometry = arms::GeometryScorer::new(&tables, goal);
        let (geometry_emission, geometry_counters) = episode::run_reference(
            &packed,
            &task,
            budget,
            SeamMode::Arm,
            &mut geometry,
            &mut reference_scratch,
        )
        .expect("geometry episode runs");
        let predicate_bytes = episode::predicates_for(&task).unwrap();
        let schema = uor_r4_graph_format::plan_sections::PlanSchema::parse(&packed.schema).unwrap();
        let predicates =
            uor_r4_graph_format::plan_sections::PredicateSet::parse(&predicate_bytes, &schema)
                .unwrap();
        let mut fifo = GoalDistanceScorer::new(&predicates);
        let (fifo_emission, fifo_counters) = episode::run_reference(
            &packed,
            &task,
            budget,
            SeamMode::Parity(false),
            &mut fifo,
            &mut reference_scratch,
        )
        .expect("fifo episode runs");
        if geometry_emission != fifo_emission || geometry_counters != fifo_counters {
            diverged += 1;
        }
    }
    assert!(episodes >= 32, "the seam check must not be vacuous");
    assert!(
        diverged > 0,
        "geometry-ordered retention never diverged from FIFO at frontier-8"
    );
}
