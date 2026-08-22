//! #845 increment 4 — the equal-budget geometry measurement on both A2 axes,
//! per `docs/w33_geometry_qualification_spec_845.md` §5/§5-A/§6 and
//! `docs/compositional_planning_spec_844.md` §12.
//!
//! The full grids are `#[ignore]`d (measurements, not gates); the default
//! tests assert the instrument's own non-vacuity. Run with
//! `-- --ignored --nocapture` and read the MEAS/VERDICT lines.

mod support;

use support::arms;
use support::episode::{self, Packed};
use support::ordering::{GoalDistanceScorer, RefScratch, Scorer, SeamMode};
use support::stats;
use uor_r4_graph_compiler::compositional_planning as cp;
use uor_r4_graph_runtime::plan::{PlanBudget, PlanScratch, PlanStrategy};

/// Frozen effect floor (#844 §2.5).
const DELTA_MIN: f64 = 0.05;
/// Frozen A2(a) work-reduction floor (spec §5).
const RHO_MIN: f64 = 0.10;

/// The 12 separating cells of the #843 record §3 (A2(a), frozen budget).
const A2A_CELLS: [(cp::TaskFamily, usize); 12] = [
    (cp::TaskFamily::GraphNavigation, 1),
    (cp::TaskFamily::GraphNavigation, 2),
    (cp::TaskFamily::GraphNavigation, 4),
    (cp::TaskFamily::GraphNavigation, 8),
    (cp::TaskFamily::SymbolicTransformation, 1),
    (cp::TaskFamily::ConstraintSatisfaction, 1),
    (cp::TaskFamily::ConstraintSatisfaction, 8),
    (cp::TaskFamily::MultiHopEvidence, 1),
    (cp::TaskFamily::MultiHopEvidence, 2),
    (cp::TaskFamily::MultiHopEvidence, 4),
    (cp::TaskFamily::MultiHopEvidence, 8),
    (cp::TaskFamily::CounterfactualIntervention, 1),
];

/// The A2(b) budget variants (#844 spec §12): primary = the frontier ladder
/// at H = 8; secondary = the expansion ladder at H = 8 and frozen H = 16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellBudget {
    Frozen,
    Frontier(u16),
    Expansions(u32),
}

impl CellBudget {
    fn label(self) -> String {
        match self {
            CellBudget::Frozen => "frozen".to_string(),
            CellBudget::Frontier(width) => format!("frontier-{width}"),
            CellBudget::Expansions(cap) => format!("expansions-{cap}"),
        }
    }

    fn budget(self, horizon: usize) -> PlanBudget {
        let frozen = PlanBudget {
            horizon: horizon as u8,
            ..PlanBudget::frozen()
        };
        match self {
            CellBudget::Frozen => frozen,
            CellBudget::Frontier(width) => PlanBudget {
                frontier: width,
                ..frozen
            },
            CellBudget::Expansions(cap) => PlanBudget {
                max_expansions: cap,
                ..frozen
            },
        }
    }
}

const A2B_PRIMARY_BUDGETS: [CellBudget; 3] = [
    CellBudget::Frontier(16),
    CellBudget::Frontier(8),
    CellBudget::Frontier(4),
];
const A2B_SECONDARY_BUDGETS: [CellBudget; 2] =
    [CellBudget::Expansions(64), CellBudget::Expansions(32)];

/// Everything one (family, horizon) pair fits once: the packed artifacts, the
/// replay nulls, and the fitted controls.
struct CellFit {
    packed: Packed,
    shuffled: Packed,
    nulls: episode::Fitted,
    learned: arms::LearnedTable,
    spectral: arms::SpectralEmbedding,
}

fn fit_cell(tables: &arms::W33Tables, family: cp::TaskFamily, horizon: usize) -> Option<CellFit> {
    let set = episode::induce_for(family, horizon)?;
    let packed = episode::pack(&set, 0)?;
    let shuffled = episode::pack(&set, 1)?;
    let nulls = episode::fit_nulls(family, horizon);
    let observations = episode::fitting_observations(family, horizon);
    let learned = arms::LearnedTable::fit(tables, &observations.remaining);
    let spectral = arms::SpectralEmbedding::fit(tables, &observations.transitions);
    Some(CellFit {
        packed,
        shuffled,
        nulls,
        learned,
        spectral,
    })
}

/// Per-arm episode traces over one cell's held-out instances.
#[derive(Debug, Clone)]
struct ArmTrace {
    name: &'static str,
    correct: Vec<bool>,
    expansions: Vec<f64>,
    lookups: u64,
}

impl ArmTrace {
    fn rate(&self) -> f64 {
        if self.correct.is_empty() {
            return 0.0;
        }
        self.correct.iter().filter(|c| **c).count() as f64 / self.correct.len() as f64
    }

    fn perfect(&self) -> bool {
        self.correct.iter().all(|c| *c)
    }

    fn mean_expansions(&self) -> f64 {
        if self.expansions.is_empty() {
            return 0.0;
        }
        self.expansions.iter().sum::<f64>() / self.expansions.len() as f64
    }
}

/// The ordering-arm roster for one episode. The geometry arm is index 0; the
/// rest are the non-geometric ordering controls.
const ORDERING_ARMS: [&str; 9] = [
    "w33-geometry",
    "table-guided-beam",
    "hamming-popcount",
    "learned-table-codes",
    "vsa-binding",
    "spectral-embedding",
    "random-embedding",
    "isomorphic-relabel",
    "phase-permuted",
];

/// One cell's full measurement: the FIFO baseline, the nine ordering arms in
/// arm mode, and the four #843 nulls, all under one budget.
struct CellData {
    family: &'static str,
    horizon: usize,
    budget_label: String,
    n: usize,
    baseline: ArmTrace,
    arms: Vec<ArmTrace>,
    nulls: [ArmTrace; 4],
    gold_floor: Vec<f64>,
}

/// Run one arm-mode episode with a freshly constructed scorer.
#[allow(clippy::too_many_arguments)]
fn run_ordering_arm(
    name: &'static str,
    tables: &arms::W33Tables,
    vsa: &arms::VsaTables,
    random: &arms::RandomTables,
    relabeled: &arms::RelabeledTables,
    fit: &CellFit,
    task: &cp::TaskInstance,
    budget: PlanBudget,
    scratch: &mut RefScratch,
) -> (bool, f64, u64) {
    let goal = episode::goal_center(task);
    let run = |scorer: &mut dyn Scorer, scratch: &mut RefScratch| -> (bool, f64, u64) {
        let (emission, counters) =
            episode::run_reference(&fit.packed, task, budget, SeamMode::Arm, scorer, scratch)
                .expect("arm episode runs");
        (
            episode::outcome_correct(task, &emission),
            f64::from(counters.expansions),
            scorer.lookups(),
        )
    };
    match name {
        "w33-geometry" => run(&mut arms::GeometryScorer::new(tables, goal), scratch),
        "table-guided-beam" => {
            let schema =
                uor_r4_graph_format::plan_sections::PlanSchema::parse(&fit.packed.schema).unwrap();
            let bytes = episode::predicates_for(task).unwrap();
            let predicates =
                uor_r4_graph_format::plan_sections::PredicateSet::parse(&bytes, &schema).unwrap();
            run(&mut GoalDistanceScorer::new(&predicates), scratch)
        }
        "hamming-popcount" => run(&mut arms::HammingScorer::new(goal), scratch),
        "learned-table-codes" => run(
            &mut arms::LearnedScorer::new(tables, &fit.learned, goal),
            scratch,
        ),
        "vsa-binding" => run(&mut arms::VsaScorer::new(vsa, goal), scratch),
        "spectral-embedding" => run(
            &mut arms::SpectralScorer::new(tables, &fit.spectral, goal),
            scratch,
        ),
        "random-embedding" => run(&mut arms::RandomScorer::new(tables, random, goal), scratch),
        "isomorphic-relabel" => run(
            &mut arms::RelabeledScorer::new(tables, relabeled, goal),
            scratch,
        ),
        "phase-permuted" => run(&mut arms::PhasePermutedScorer::new(tables, goal), scratch),
        other => panic!("unknown arm {other}"),
    }
}

/// Measure one cell completely.
#[allow(clippy::too_many_arguments)]
fn run_cell(
    tables: &arms::W33Tables,
    vsa: &arms::VsaTables,
    random: &arms::RandomTables,
    relabeled: &arms::RelabeledTables,
    fit: &CellFit,
    family: cp::TaskFamily,
    horizon: usize,
    cell_budget: CellBudget,
    n: usize,
) -> CellData {
    let budget = cell_budget.budget(horizon);
    let mut deployed_scratch = Box::new(PlanScratch::new());
    let mut scratch = RefScratch::new();
    let trace = |name: &'static str| ArmTrace {
        name,
        correct: Vec::new(),
        expansions: Vec::new(),
        lookups: 0,
    };
    let mut baseline = trace("bounded-breadth-first");
    let mut arm_traces: Vec<ArmTrace> = ORDERING_ARMS.iter().map(|name| trace(name)).collect();
    let mut null_traces = [
        trace("retrieval-only"),
        trace("direct-continuation"),
        trace("memorized-trajectory"),
        trace("shuffled-state"),
    ];
    let mut gold_floor = Vec::new();
    let mut measured = 0usize;
    for seed in episode::seeds(true, n) {
        let Some(task) = episode::try_generate(family, seed, horizon) else {
            continue;
        };
        measured += 1;
        gold_floor.push(if task.gold.decline.is_some() {
            1.0
        } else {
            task.gold.chosen_path.len() as f64
        });
        {
            let schema =
                uor_r4_graph_format::plan_sections::PlanSchema::parse(&fit.packed.schema).unwrap();
            let bytes = episode::predicates_for(&task).unwrap();
            let predicates =
                uor_r4_graph_format::plan_sections::PredicateSet::parse(&bytes, &schema).unwrap();
            let mut fifo = GoalDistanceScorer::new(&predicates);
            let (emission, counters) = episode::run_reference(
                &fit.packed,
                &task,
                budget,
                SeamMode::Parity(false),
                &mut fifo,
                &mut scratch,
            )
            .expect("baseline episode runs");
            baseline
                .correct
                .push(episode::outcome_correct(&task, &emission));
            baseline.expansions.push(f64::from(counters.expansions));
        }
        for (index, name) in ORDERING_ARMS.iter().enumerate() {
            let (correct, expansions, lookups) = run_ordering_arm(
                name,
                tables,
                vsa,
                random,
                relabeled,
                fit,
                &task,
                budget,
                &mut scratch,
            );
            arm_traces[index].correct.push(correct);
            arm_traces[index].expansions.push(expansions);
            arm_traces[index].lookups = lookups;
        }
        let replays: [episode::Emission; 3] = [
            episode::retrieval_only(&fit.nulls, &task),
            episode::direct_continuation(&task, horizon),
            episode::memorized(&fit.nulls, &task),
        ];
        for (index, emission) in replays.iter().enumerate() {
            null_traces[index]
                .correct
                .push(episode::outcome_correct(&task, emission));
            null_traces[index].expansions.push(0.0);
        }
        let shuffled = episode::run_deployed(
            PlanStrategy::BreadthFirst,
            &fit.shuffled,
            &task,
            budget,
            &mut deployed_scratch,
        )
        .map(|(emission, _)| emission)
        .unwrap_or(None);
        null_traces[3]
            .correct
            .push(episode::outcome_correct(&task, &shuffled));
        null_traces[3].expansions.push(0.0);
    }
    CellData {
        family: family.label(),
        horizon,
        budget_label: cell_budget.label(),
        n: measured,
        baseline,
        arms: arm_traces,
        nulls: null_traces,
        gold_floor,
    }
}

/// The A2(a) per-cell reading (spec §5/§5-A).
#[derive(Debug, Clone)]
struct A2aReading {
    family: &'static str,
    horizon: usize,
    class: &'static str,
    bar: &'static str,
    bar_mean: f64,
    geometry_mean: f64,
    headroom: f64,
    mean: f64,
    standard_error: f64,
    lower_bound: f64,
    p: f64,
    geometry_perfect: bool,
    pass: bool,
}

fn read_a2a(cell: &CellData) -> A2aReading {
    let geometry = &cell.arms[0];
    let bar = cell.arms[1..]
        .iter()
        .filter(|arm| arm.perfect())
        .min_by(|a, b| {
            a.mean_expansions()
                .partial_cmp(&b.mean_expansions())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    let Some(bar) = bar else {
        return A2aReading {
            family: cell.family,
            horizon: cell.horizon,
            class: "NOT-TRIGGERED",
            bar: "none-perfect",
            bar_mean: 0.0,
            geometry_mean: geometry.mean_expansions(),
            headroom: 0.0,
            mean: 0.0,
            standard_error: 0.0,
            lower_bound: 0.0,
            p: 1.0,
            geometry_perfect: geometry.perfect(),
            pass: false,
        };
    };
    let headroom_terms: Vec<f64> = bar
        .expansions
        .iter()
        .zip(cell.gold_floor.iter())
        .map(|(b, floor)| (b - floor).max(0.0) / b.max(1.0))
        .collect();
    let headroom = headroom_terms.iter().sum::<f64>() / headroom_terms.len().max(1) as f64;
    let class = if headroom >= RHO_MIN {
        "reduction"
    } else {
        "no-regression"
    };
    let reductions: Vec<f64> = bar
        .expansions
        .iter()
        .zip(geometry.expansions.iter())
        .map(|(b, g)| (b - g) / b.max(1.0))
        .collect();
    let (mean, standard_error, lower_bound) = stats::paired_lower_bound(&reductions);
    let threshold = if class == "reduction" { RHO_MIN } else { 0.0 };
    // The no-regression reading admits the degenerate identical case (spec
    // §5-A): a zero-variance zero-mean difference reads as LB = 0, which
    // passes at threshold 0 and fails any positive threshold.
    let pass = geometry.perfect()
        && if standard_error == 0.0 {
            mean >= threshold
        } else {
            lower_bound >= threshold
        };
    let p = stats::p_value(mean, standard_error, threshold);
    A2aReading {
        family: cell.family,
        horizon: cell.horizon,
        class,
        bar: bar.name,
        bar_mean: bar.mean_expansions(),
        geometry_mean: geometry.mean_expansions(),
        headroom,
        mean,
        standard_error,
        lower_bound,
        p,
        geometry_perfect: geometry.perfect(),
        pass,
    }
}

/// The A2(b) per-cell reading (spec §5; #844 §12).
#[derive(Debug, Clone)]
struct A2bReading {
    family: &'static str,
    horizon: usize,
    budget_label: String,
    primary: bool,
    geometry_rate: f64,
    bar: &'static str,
    bar_rate: f64,
    mean: f64,
    standard_error: f64,
    lower_bound: f64,
    p: f64,
    pass: bool,
}

fn read_a2b(cell: &CellData, primary: bool) -> A2bReading {
    let geometry = &cell.arms[0];
    let mut candidates: Vec<&ArmTrace> = vec![&cell.baseline];
    candidates.extend(cell.arms[1..].iter());
    candidates.extend(cell.nulls.iter());
    let bar = candidates
        .into_iter()
        .max_by(|a, b| {
            a.rate()
                .partial_cmp(&b.rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("bar candidates exist");
    let differences: Vec<f64> = geometry
        .correct
        .iter()
        .zip(bar.correct.iter())
        .map(|(g, b)| f64::from(u8::from(*g)) - f64::from(u8::from(*b)))
        .collect();
    let (mean, standard_error, lower_bound) = stats::paired_lower_bound(&differences);
    let pass = if standard_error == 0.0 {
        mean > DELTA_MIN
    } else {
        lower_bound >= DELTA_MIN
    };
    A2bReading {
        family: cell.family,
        horizon: cell.horizon,
        budget_label: cell.budget_label.clone(),
        primary,
        geometry_rate: geometry.rate(),
        bar: bar.name,
        bar_rate: bar.rate(),
        mean,
        standard_error,
        lower_bound,
        p: stats::p_value(mean, standard_error, DELTA_MIN),
        pass,
    }
}

fn print_arm_rates(cell: &CellData) {
    let rates: Vec<String> = cell
        .arms
        .iter()
        .map(|arm| {
            format!(
                "{}={:.4}/{:.1}",
                arm.name,
                arm.rate(),
                arm.mean_expansions()
            )
        })
        .collect();
    let nulls: Vec<String> = cell
        .nulls
        .iter()
        .map(|null| format!("{}={:.4}", null.name, null.rate()))
        .collect();
    println!(
        "MEAS-CELL | {:<26} | H={:<2} | {:<14} | n={} | baseline={:.4}/{:.1} | {} | {}",
        cell.family,
        cell.horizon,
        cell.budget_label,
        cell.n,
        cell.baseline.rate(),
        cell.baseline.mean_expansions(),
        rates.join(" "),
        nulls.join(" ")
    );
}

/// The full A2 measurement — both axes, every cell, the pre-registered
/// readings, and the axis verdict inputs. `--ignored` because it is the
/// measurement; the record is `docs/w33_geometry_qualification_845.md`.
#[test]
#[ignore = "the #845 measurement: run explicitly with --ignored --nocapture"]
fn geometry_qualification_measurement() {
    let tables = arms::W33Tables::build();
    let vsa = arms::VsaTables::build();
    let random = arms::RandomTables::build();
    let relabeled = arms::RelabeledTables::build(&tables);

    // ---- A2(a): the 12 separating cells at the frozen budget ----
    let mut a2a = Vec::new();
    for (family, horizon) in A2A_CELLS {
        let fit = fit_cell(&tables, family, horizon).expect("cell fits");
        let cell = run_cell(
            &tables,
            &vsa,
            &random,
            &relabeled,
            &fit,
            family,
            horizon,
            CellBudget::Frozen,
            episode::N_PER_CELL,
        );
        print_arm_rates(&cell);
        let reading = read_a2a(&cell);
        println!(
            "MEAS-A2A | {:<26} | H={:<2} | {} | bar={}@{:.1} geom@{:.1} | headroom={:.4} | r mean={:.4} se={:.4} lb={:.4} p={:.4} | geometry_perfect={} | {}",
            reading.family,
            reading.horizon,
            reading.class,
            reading.bar,
            reading.bar_mean,
            reading.geometry_mean,
            reading.headroom,
            reading.mean,
            reading.standard_error,
            reading.lower_bound,
            reading.p,
            reading.geometry_perfect,
            if reading.pass { "PASS" } else { "fail" }
        );
        a2a.push(reading);
    }
    let a2a_pass = a2a.iter().all(|r| r.pass);
    let a2a_holm = stats::holm_pass(&a2a.iter().map(|r| r.p).collect::<Vec<_>>());
    println!(
        "VERDICT-A2A | conjunction={} | cells_pass={}/12 | holm_pass={}/12",
        if a2a_pass { "PASS" } else { "FAIL" },
        a2a.iter().filter(|r| r.pass).count(),
        a2a_holm.iter().filter(|h| **h).count()
    );

    // ---- A2(b): nine primary + nine secondary cells ----
    let mut a2b = Vec::new();
    for family in episode::SEPARATING {
        let fit = fit_cell(&tables, family, 8).expect("cell fits at H=8");
        for budget in A2B_PRIMARY_BUDGETS {
            let cell = run_cell(
                &tables,
                &vsa,
                &random,
                &relabeled,
                &fit,
                family,
                8,
                budget,
                episode::N_PER_CELL,
            );
            print_arm_rates(&cell);
            a2b.push(read_a2b(&cell, true));
        }
        for budget in A2B_SECONDARY_BUDGETS {
            let cell = run_cell(
                &tables,
                &vsa,
                &random,
                &relabeled,
                &fit,
                family,
                8,
                budget,
                episode::N_PER_CELL,
            );
            print_arm_rates(&cell);
            a2b.push(read_a2b(&cell, false));
        }
        let fit16 = fit_cell(&tables, family, 16).expect("cell fits at H=16");
        let cell = run_cell(
            &tables,
            &vsa,
            &random,
            &relabeled,
            &fit16,
            family,
            16,
            CellBudget::Frozen,
            episode::N_PER_CELL,
        );
        print_arm_rates(&cell);
        a2b.push(read_a2b(&cell, false));
    }
    for reading in &a2b {
        println!(
            "MEAS-A2B | {:<26} | H={:<2} | {:<14} | {} | geom={:.4} bar={}@{:.4} | d mean={:.4} se={:.4} lb={:.4} p={:.4} | {}",
            reading.family,
            reading.horizon,
            reading.budget_label,
            if reading.primary { "PRIMARY" } else { "secondary" },
            reading.geometry_rate,
            reading.bar,
            reading.bar_rate,
            reading.mean,
            reading.standard_error,
            reading.lower_bound,
            reading.p,
            if reading.pass { "PASS" } else { "fail" }
        );
    }
    let primary: Vec<&A2bReading> = a2b.iter().filter(|r| r.primary).collect();
    let a2b_pass = primary.iter().all(|r| r.pass);
    let a2b_holm = stats::holm_pass(&primary.iter().map(|r| r.p).collect::<Vec<_>>());
    println!(
        "VERDICT-A2B | conjunction={} | primary_pass={}/9 | holm_pass={}/9 | secondary_pass={}/9",
        if a2b_pass { "PASS" } else { "FAIL" },
        primary.iter().filter(|r| r.pass).count(),
        a2b_holm.iter().filter(|h| **h).count(),
        a2b.iter().filter(|r| !r.primary && r.pass).count()
    );

    println!(
        "VERDICT | A2a={} A2b={} | verdict-space: both=PROMOTE-FOR-LOWERING one=REVISE neither=NO-GEOMETRIC-ADVANTAGE",
        if a2a_pass { "PASS" } else { "FAIL" },
        if a2b_pass { "PASS" } else { "FAIL" }
    );
}

// ---------------------------------------------------------------------------
// Non-vacuity gates — run by default
// ---------------------------------------------------------------------------

/// The A2(a) classifier and both pass rules can fire and can fail.
#[test]
fn the_a2a_reading_fires_in_both_directions() {
    let synthetic = |name: &'static str, expansions: Vec<f64>, correct: bool| ArmTrace {
        name,
        correct: vec![correct; expansions.len()],
        expansions,
        lookups: 0,
    };
    // Reduction cell: bar mean 10 vs floor 2 (headroom 0.8), geometry cuts to 6.
    let cell = CellData {
        family: "synthetic",
        horizon: 8,
        budget_label: "frozen".to_string(),
        n: 4,
        baseline: synthetic("bounded-breadth-first", vec![12.0; 4], true),
        arms: vec![
            synthetic("w33-geometry", vec![6.0, 6.0, 5.0, 7.0], true),
            synthetic("table-guided-beam", vec![10.0; 4], true),
        ],
        nulls: [
            synthetic("retrieval-only", vec![0.0; 4], false),
            synthetic("direct-continuation", vec![0.0; 4], false),
            synthetic("memorized-trajectory", vec![0.0; 4], false),
            synthetic("shuffled-state", vec![0.0; 4], false),
        ],
        gold_floor: vec![2.0; 4],
    };
    let reading = read_a2a(&cell);
    assert_eq!(reading.class, "reduction");
    assert!(reading.pass, "a real reduction must pass");

    // The same cell with geometry doing MORE work must fail.
    let mut regressed = cell;
    regressed.arms[0] = synthetic("w33-geometry", vec![11.0; 4], true);
    let reading = read_a2a(&regressed);
    assert!(!reading.pass, "a regression must fail");

    // No-regression cell: bar at the floor, geometry identical -> degenerate
    // LB = 0 passes; geometry above the bar fails.
    let flat = CellData {
        family: "synthetic",
        horizon: 1,
        budget_label: "frozen".to_string(),
        n: 4,
        baseline: synthetic("bounded-breadth-first", vec![1.0; 4], true),
        arms: vec![
            synthetic("w33-geometry", vec![1.0; 4], true),
            synthetic("table-guided-beam", vec![1.0; 4], true),
        ],
        nulls: [
            synthetic("retrieval-only", vec![0.0; 4], false),
            synthetic("direct-continuation", vec![0.0; 4], false),
            synthetic("memorized-trajectory", vec![0.0; 4], false),
            synthetic("shuffled-state", vec![0.0; 4], false),
        ],
        gold_floor: vec![1.0; 4],
    };
    let reading = read_a2a(&flat);
    assert_eq!(reading.class, "no-regression");
    assert!(reading.pass, "identical work at the floor must pass");
    let mut worse = flat;
    worse.arms[0] = synthetic("w33-geometry", vec![2.0; 4], true);
    assert!(!read_a2a(&worse).pass, "regression at the floor must fail");
}

/// The measurement machinery runs end-to-end on a small live cell and the
/// A2(b) bar can actually be beaten and be unbeaten.
#[test]
fn the_measurement_machinery_is_live() {
    let tables = arms::W33Tables::build();
    let vsa = arms::VsaTables::build();
    let random = arms::RandomTables::build();
    let relabeled = arms::RelabeledTables::build(&tables);
    let family = cp::TaskFamily::GraphNavigation;
    let fit = fit_cell(&tables, family, 4).expect("fit at H=4");
    let cell = run_cell(
        &tables,
        &vsa,
        &random,
        &relabeled,
        &fit,
        family,
        4,
        CellBudget::Frozen,
        64,
    );
    assert_eq!(cell.n, 64, "the smoke cell is not vacuous");
    assert!(
        cell.baseline.perfect(),
        "the frozen-budget baseline must be perfect at H=4 (the #843 record)"
    );
    let reading = read_a2b(&cell, false);
    assert!(reading.bar_rate > 0.0, "the A2(b) bar must be able to fire");
    let reading_a = read_a2a(&cell);
    assert!(
        reading_a.bar != "none-perfect",
        "at the frozen budget a perfect non-geometric ordering control exists"
    );
}
