//! Compositional-planning reference semantics (#844, S4 item A).
//!
//! Frozen contract: `docs/compositional_planning_spec_844.md`. This module
//! extends the RF-27 semantic-state reference model ([`crate::semantic_state`])
//! and the RF-08 future-state planner with the typed objects the S4 benchmark
//! and #843 planner consume: a replayable, independently-verifiable plan
//! witness; typed evidence/provenance; an ordinal (not calibrated) confidence
//! band; a typed decline; and deterministic generators plus verifiers for the
//! five frozen task families.
//!
//! Execution scope: reference-only / off-serving-path (RF-27/RF-28 sense).
//! f32 and owned collections are permitted here; the deployed integer/table
//! planner is #843. Scoring is teacher-forced only (S3 #824 LIMIT boundary);
//! confidence is an ordinal, decline-oriented signal, not a calibrated
//! probability (S2 #823 REVISE). This establishes a falsifiable target and
//! byte-level meaning, not reasoning performance.

use std::collections::VecDeque;

use crate::semantic_state::{Action, Constraint, Goal, Region, SemanticState, TransitionEvaluator};

/// Ordinal, decline-oriented confidence band (S2 boundary: NOT a calibrated
/// probability). Ordered low to high; used only for decline ordering and
/// evidence support, never as a numeric probability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfidenceBand {
    None,
    Low,
    Medium,
    High,
}

/// Why a planning episode declined to emit a plan (honest abstention). Every
/// non-answer is exactly one of these; a fabricated plan is never emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclineReason {
    /// No valid plan exists within the representational/reachability ceiling.
    NoPlan,
    /// A fixed capacity (horizon, frontier, state/action count) was exceeded.
    Capacity,
    /// An unknown slot/state was encountered; resolved by decline, not default.
    Unknown,
    /// Confidence fell below the decline threshold.
    LowConfidence,
}

/// Typed evidence with provenance (F4 multi-hop evidence composition).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    /// Stable provenance identifier of the supporting record.
    pub provenance_id: String,
    /// The claim this evidence supports.
    pub claim: String,
    /// Ordinal support strength.
    pub support: ConfidenceBand,
}

/// One considered action at a planning step, with its transition evidence and
/// deterministic score/tie rank. Informational: the witness carries what the
/// reference planner considered; [`PlanWitness::verify`] does not depend on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRecord {
    /// Considered action name.
    pub action: String,
    /// State the action was considered from.
    pub from_state: String,
    /// Resulting state id, or `None` when the action was inapplicable/forbidden.
    pub to_state: Option<String>,
    /// Deterministic integer score (no f32 in the witness ordering).
    pub score: i64,
    /// Canonical deterministic tie-break rank (lower wins).
    pub tie_rank: u32,
}

/// The five frozen task families (spec section 2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskFamily {
    GraphNavigation,
    SymbolicTransformation,
    ConstraintSatisfaction,
    MultiHopEvidence,
    CounterfactualIntervention,
}

impl TaskFamily {
    /// Stable kebab-case label.
    pub fn label(self) -> &'static str {
        match self {
            TaskFamily::GraphNavigation => "graph-navigation",
            TaskFamily::SymbolicTransformation => "symbolic-transformation",
            TaskFamily::ConstraintSatisfaction => "constraint-satisfaction",
            TaskFamily::MultiHopEvidence => "multi-hop-evidence",
            TaskFamily::CounterfactualIntervention => "counterfactual-intervention",
        }
    }
}

/// The verdict of independently verifying a witness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WitnessVerdict {
    /// Terminal outcome and every intermediate transition are valid.
    Valid,
    /// An intermediate transition (or the terminal goal, at `step ==
    /// chosen_path.len()`) is invalid.
    Invalid { step: usize, reason: String },
    /// The witness is an honest decline.
    Declined(DeclineReason),
}

/// A replayable, independently-verifiable record of a planning episode.
///
/// The witness is verified from itself alone: replaying `chosen_path` from
/// `initial_state` under `constraints` reproduces the terminal state, which
/// must satisfy `goal` with no intermediate state entering a forbidden region.
#[derive(Debug, Clone)]
pub struct PlanWitness {
    /// Task family this witness answers.
    pub family: TaskFamily,
    /// Initial state.
    pub initial_state: SemanticState,
    /// Goal predicate (desired future-state subset).
    pub goal: Goal,
    /// Forbidden regions the plan must avoid at every step.
    pub constraints: Vec<Constraint>,
    /// What the reference planner considered (informational).
    pub considered: Vec<StepRecord>,
    /// The chosen action sequence.
    pub chosen_path: Vec<Action>,
    /// Cited evidence per chosen step (F4). Parallel to `chosen_path` when
    /// `require_evidence_per_step` is set.
    pub step_evidence: Vec<Vec<Evidence>>,
    /// When set (F4), every chosen step must carry at least one evidence item.
    pub require_evidence_per_step: bool,
    /// Set when the episode is an honest decline rather than a plan.
    pub decline: Option<DeclineReason>,
}

impl PlanWitness {
    /// Independently re-verify this witness from the witness alone. Total over
    /// valid inputs; deterministic.
    pub fn verify(&self) -> WitnessVerdict {
        if let Some(reason) = self.decline {
            return WitnessVerdict::Declined(reason);
        }
        let mut evaluator = TransitionEvaluator::new();
        for c in &self.constraints {
            evaluator.add_constraint(c.clone());
        }
        let mut state = self.initial_state.clone();
        for (i, action) in self.chosen_path.iter().enumerate() {
            match evaluator.apply(&state, action) {
                Some(next) => state = next,
                None => {
                    return WitnessVerdict::Invalid {
                        step: i,
                        reason: format!("transition `{}` inapplicable or forbidden", action.name),
                    };
                }
            }
        }
        if !self.goal.is_satisfied_by(&state) {
            return WitnessVerdict::Invalid {
                step: self.chosen_path.len(),
                reason: "terminal state does not satisfy the goal".to_string(),
            };
        }
        if self.require_evidence_per_step {
            if self.step_evidence.len() != self.chosen_path.len() {
                return WitnessVerdict::Invalid {
                    step: self.chosen_path.len(),
                    reason: "evidence is not cited for every step".to_string(),
                };
            }
            for (i, ev) in self.step_evidence.iter().enumerate() {
                if ev.is_empty() {
                    return WitnessVerdict::Invalid {
                        step: i,
                        reason: "step cites no supporting evidence".to_string(),
                    };
                }
            }
        }
        WitnessVerdict::Valid
    }
}

/// Deterministic FNV-1a/64 over a canonical string — the content-addressed
/// sample identity (no clock, RNG, or hash-iteration-order dependence).
fn fnv1a64(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Frozen default capacities (spec section 2.5): maximum planning horizon and
/// frontier width. Instances exceeding them decline at the capacity boundary.
pub const H_MAX: usize = 16;
/// Frozen maximum frontier width (spec section 2.5).
pub const W_MAX: usize = 64;

/// A generated task instance with a replayable gold plan.
#[derive(Debug, Clone)]
pub struct TaskInstance {
    /// Task family.
    pub family: TaskFamily,
    /// Deterministic generation seed (sample identity input).
    pub seed: u64,
    /// Planning horizon (bounded).
    pub horizon: usize,
    /// Initial state.
    pub initial_state: SemanticState,
    /// Goal predicate.
    pub goal: Goal,
    /// Forbidden regions.
    pub constraints: Vec<Constraint>,
    /// Available actions/operators.
    pub actions: Vec<Action>,
    /// Replayable gold plan (or an honest decline when unsolvable).
    pub gold: PlanWitness,
}

impl TaskInstance {
    /// Content-addressed identity derived from the frozen generation inputs.
    /// Identical inputs share an id (deterministic; no clock/RNG/order).
    pub fn id(&self) -> u64 {
        let mut canon = format!(
            "{}|{}|{}|init={:?}|goal={:?}:{}",
            self.family.label(),
            self.seed,
            self.horizon,
            round_vec(&self.initial_state.vector),
            round_vec(&self.goal.target_region.center),
            (self.goal.target_region.radius * 1000.0).round() as i64,
        );
        for c in &self.constraints {
            canon.push_str(&format!("|f={:?}", round_vec(&c.forbidden_region.center)));
        }
        fnv1a64(&canon)
    }
}

fn round_vec(v: &[f32]) -> Vec<i64> {
    v.iter().map(|x| x.round() as i64).collect()
}

fn state_key(s: &SemanticState) -> String {
    format!("{:?}|{:?}", round_vec(&s.vector), s.boolean_signature)
}

fn cell(id: &str, x: i64, y: i64) -> SemanticState {
    SemanticState::new(id, vec![x as f32, y as f32], vec![0], 1.0)
}

fn goal_at(x: i64, y: i64) -> Goal {
    Goal::new(
        "reach",
        Region::new("goal", vec![x as f32, y as f32], 0.5, "goal-cell"),
        0.0,
    )
}

fn forbid(id: &str, x: i64, y: i64) -> Constraint {
    Constraint::new(
        id,
        Region::new(id, vec![x as f32, y as f32], 0.5, "forbidden"),
    )
}

fn grid_actions() -> Vec<Action> {
    vec![
        Action::new("east", vec![1.0, 0.0], vec![0]),
        Action::new("north", vec![0.0, 1.0], vec![0]),
        Action::new("west", vec![-1.0, 0.0], vec![0]),
        Action::new("south", vec![0.0, -1.0], vec![0]),
    ]
}

/// Deterministic breadth-first reference solver: the shortest action sequence
/// from `initial` to a goal-satisfying state that never enters a forbidden
/// region, exploring `actions` in fixed order. `None` when no plan exists
/// within `max_steps` (the reachability ceiling).
fn bfs_plan(
    initial: &SemanticState,
    goal: &Goal,
    constraints: &[Constraint],
    actions: &[Action],
    max_steps: usize,
) -> Option<Vec<Action>> {
    let mut evaluator = TransitionEvaluator::new();
    for c in constraints {
        evaluator.add_constraint(c.clone());
    }
    if goal.is_satisfied_by(initial) {
        return Some(Vec::new());
    }
    let mut visited = std::collections::BTreeSet::new();
    visited.insert(state_key(initial));
    let mut queue: VecDeque<(SemanticState, Vec<Action>)> = VecDeque::new();
    queue.push_back((initial.clone(), Vec::new()));
    while let Some((state, path)) = queue.pop_front() {
        if path.len() >= max_steps {
            continue;
        }
        for action in actions {
            if let Some(next) = evaluator.apply(&state, action) {
                let key = state_key(&next);
                if visited.contains(&key) {
                    continue;
                }
                let mut next_path = path.clone();
                next_path.push(action.clone());
                if goal.is_satisfied_by(&next) {
                    return Some(next_path);
                }
                visited.insert(key);
                queue.push_back((next, next_path));
            }
        }
    }
    None
}

/// The problem components of a task instance (grouped to keep `finish` within
/// the argument-count lint and to make the generators read declaratively).
struct Problem {
    initial: SemanticState,
    goal: Goal,
    constraints: Vec<Constraint>,
    actions: Vec<Action>,
}

fn finish(
    family: TaskFamily,
    seed: u64,
    horizon: usize,
    problem: Problem,
    require_evidence: bool,
) -> TaskInstance {
    let Problem {
        initial,
        goal,
        constraints,
        actions,
    } = problem;
    let solved = bfs_plan(&initial, &goal, &constraints, &actions, horizon);
    let (chosen, decline): (Vec<Action>, Option<DeclineReason>) = match solved {
        Some(p) => (p, None),
        None => (Vec::new(), Some(DeclineReason::NoPlan)),
    };
    let step_evidence: Vec<Vec<Evidence>> = if require_evidence && decline.is_none() {
        chosen
            .iter()
            .enumerate()
            .map(|(i, _)| {
                vec![Evidence {
                    provenance_id: format!("prov-{seed}-{i}"),
                    claim: "hop supported by cited provenance".to_string(),
                    support: ConfidenceBand::High,
                }]
            })
            .collect()
    } else {
        Vec::new()
    };
    let considered: Vec<StepRecord> = chosen
        .iter()
        .enumerate()
        .map(|(i, a)| StepRecord {
            action: a.name.clone(),
            from_state: format!("s{i}"),
            to_state: Some(format!("s{}", i + 1)),
            score: (chosen.len() - i) as i64,
            tie_rank: 0,
        })
        .collect();
    let gold = PlanWitness {
        family,
        initial_state: initial.clone(),
        goal: goal.clone(),
        constraints: constraints.clone(),
        considered,
        chosen_path: chosen,
        step_evidence,
        require_evidence_per_step: require_evidence,
        decline,
    };
    TaskInstance {
        family,
        seed,
        horizon,
        initial_state: initial,
        goal,
        constraints,
        actions,
        gold,
    }
}

/// Generate a deterministic task instance for `family` from `seed`. Instances
/// are constructed to be solvable within `horizon`; the gold plan is the BFS
/// reference solver's shortest valid plan. Teacher-forced scope (S3 boundary).
pub fn generate(family: TaskFamily, seed: u64, horizon: usize) -> TaskInstance {
    match family {
        TaskFamily::GraphNavigation => {
            let tx = 3 + (seed % 3) as i64;
            finish(
                family,
                seed,
                horizon,
                Problem {
                    initial: cell("start", 0, 0),
                    goal: goal_at(tx, 0),
                    constraints: vec![forbid("block", 2, 0)],
                    actions: grid_actions(),
                },
                false,
            )
        }
        TaskFamily::ConstraintSatisfaction => {
            let gap = 1 + (seed % 2) as i64;
            let mut constraints = Vec::new();
            for y in -1..=3 {
                if y != gap {
                    constraints.push(forbid(&format!("wall-{y}"), 2, y));
                }
            }
            finish(
                family,
                seed,
                horizon,
                Problem {
                    initial: cell("start", 0, 0),
                    goal: goal_at(4, 0),
                    constraints,
                    actions: grid_actions(),
                },
                false,
            )
        }
        TaskFamily::SymbolicTransformation => {
            let tx = 4 + (seed % 3) as i64;
            let actions = vec![
                Action::new("op-add2x", vec![2.0, 0.0], vec![0]),
                Action::new("op-inc-y", vec![0.0, 1.0], vec![0]),
                Action::new("op-dec-x", vec![-1.0, 0.0], vec![0]),
            ];
            finish(
                family,
                seed,
                horizon,
                Problem {
                    initial: cell("term0", 0, 0),
                    goal: goal_at(tx, 2),
                    constraints: Vec::new(),
                    actions,
                },
                false,
            )
        }
        TaskFamily::MultiHopEvidence => {
            let tx = 3 + (seed % 3) as i64;
            finish(
                family,
                seed,
                horizon,
                Problem {
                    initial: cell("q", 0, 0),
                    goal: goal_at(tx, 0),
                    constraints: Vec::new(),
                    actions: grid_actions(),
                },
                true,
            )
        }
        TaskFamily::CounterfactualIntervention => {
            let tx = 3 + (seed % 3) as i64;
            let block_x = 1 + (seed % (tx as u64 - 1)) as i64;
            finish(
                family,
                seed,
                horizon,
                Problem {
                    initial: cell("start", 0, 0),
                    goal: goal_at(tx, 0),
                    constraints: vec![forbid("intervened", block_x, 0)],
                    actions: grid_actions(),
                },
                false,
            )
        }
    }
}

/// Verify a submitted action sequence against a task's frozen goal and
/// constraints, independent of how it was produced. Deterministic and total.
pub fn verify_submission(task: &TaskInstance, submitted: &[Action]) -> WitnessVerdict {
    let step_evidence: Vec<Vec<Evidence>> = if task.gold.require_evidence_per_step {
        submitted
            .iter()
            .map(|_| {
                vec![Evidence {
                    provenance_id: "submitted".to_string(),
                    claim: "hop".to_string(),
                    support: ConfidenceBand::Medium,
                }]
            })
            .collect()
    } else {
        Vec::new()
    };
    let witness = PlanWitness {
        family: task.family,
        initial_state: task.initial_state.clone(),
        goal: task.goal.clone(),
        constraints: task.constraints.clone(),
        considered: Vec::new(),
        chosen_path: submitted.to_vec(),
        step_evidence,
        require_evidence_per_step: task.gold.require_evidence_per_step,
        decline: None,
    };
    witness.verify()
}

/// Metamorphic relabeling: translate the whole task by `(dx, dy)`. The gold
/// action sequence stays valid (coordinate/label invariance), so a memorizer
/// keyed on absolute positions breaks while a real planner does not.
pub fn relabel(task: &TaskInstance, dx: i64, dy: i64) -> TaskInstance {
    let shift_state = |s: &SemanticState| {
        SemanticState::new(
            s.id.clone(),
            vec![s.vector[0] + dx as f32, s.vector[1] + dy as f32],
            s.boolean_signature.clone(),
            s.confidence,
        )
    };
    let shift_region = |r: &Region| {
        Region::new(
            r.id.clone(),
            vec![r.center[0] + dx as f32, r.center[1] + dy as f32],
            r.radius,
            r.label.clone(),
        )
    };
    let initial = shift_state(&task.initial_state);
    let goal = Goal::new(
        task.goal.name.clone(),
        shift_region(&task.goal.target_region),
        task.goal.min_confidence,
    );
    let constraints: Vec<Constraint> = task
        .constraints
        .iter()
        .map(|c| Constraint::new(c.name.clone(), shift_region(&c.forbidden_region)))
        .collect();
    let gold = PlanWitness {
        family: task.family,
        initial_state: initial.clone(),
        goal: goal.clone(),
        constraints: constraints.clone(),
        considered: task.gold.considered.clone(),
        chosen_path: task.gold.chosen_path.clone(),
        step_evidence: task.gold.step_evidence.clone(),
        require_evidence_per_step: task.gold.require_evidence_per_step,
        decline: task.gold.decline,
    };
    TaskInstance {
        family: task.family,
        seed: task.seed,
        horizon: task.horizon,
        initial_state: initial,
        goal,
        constraints,
        actions: task.actions.clone(),
        gold,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAMILIES: [TaskFamily; 5] = [
        TaskFamily::GraphNavigation,
        TaskFamily::SymbolicTransformation,
        TaskFamily::ConstraintSatisfaction,
        TaskFamily::MultiHopEvidence,
        TaskFamily::CounterfactualIntervention,
    ];

    #[test]
    fn every_family_generates_a_valid_replayable_gold() {
        for family in FAMILIES {
            for seed in 0..6u64 {
                let t = generate(family, seed, H_MAX);
                assert_eq!(
                    t.gold.verify(),
                    WitnessVerdict::Valid,
                    "{} seed {seed} gold must verify",
                    family.label()
                );
                assert!(
                    !t.gold.chosen_path.is_empty(),
                    "{} seed {seed} gold path is non-trivial",
                    family.label()
                );
                assert!(t.gold.chosen_path.len() <= t.horizon);
            }
        }
    }

    #[test]
    fn verifier_rejects_a_step_into_a_forbidden_region() {
        // GraphNavigation seed 0: target (3,0), forbidden (2,0).
        let t = generate(TaskFamily::GraphNavigation, 0, H_MAX);
        let acts = grid_actions();
        let east = acts[0].clone();
        let two_easts = vec![east.clone(), east];
        match verify_submission(&t, &two_easts) {
            WitnessVerdict::Invalid { step, .. } => assert_eq!(step, 1),
            other => panic!("expected Invalid at the forbidden step, got {other:?}"),
        }
    }

    #[test]
    fn verifier_rejects_a_terminal_that_misses_the_goal() {
        let t = generate(TaskFamily::GraphNavigation, 1, H_MAX);
        let mut short = t.gold.chosen_path.clone();
        short.pop();
        assert!(matches!(
            verify_submission(&t, &short),
            WitnessVerdict::Invalid { .. }
        ));
    }

    #[test]
    fn a_declining_witness_reports_the_typed_decline() {
        let t = generate(TaskFamily::GraphNavigation, 0, H_MAX);
        let mut w = t.gold.clone();
        w.decline = Some(DeclineReason::NoPlan);
        assert_eq!(w.verify(), WitnessVerdict::Declined(DeclineReason::NoPlan));
    }

    #[test]
    fn multihop_gold_requires_cited_evidence_per_step() {
        let t = generate(TaskFamily::MultiHopEvidence, 2, H_MAX);
        assert!(t.gold.require_evidence_per_step);
        assert_eq!(t.gold.verify(), WitnessVerdict::Valid);
        // stripping the cited evidence makes an otherwise-valid path invalid.
        let mut stripped = t.gold.clone();
        stripped.step_evidence = Vec::new();
        assert!(matches!(stripped.verify(), WitnessVerdict::Invalid { .. }));
    }

    #[test]
    fn counterfactual_intervention_invalidates_the_base_straight_path() {
        // seed 0: target (3,0), intervention blocks a cell on the straight path.
        let t = generate(TaskFamily::CounterfactualIntervention, 0, H_MAX);
        assert_eq!(t.gold.verify(), WitnessVerdict::Valid); // the detour gold is valid
        let acts = grid_actions();
        let east = acts[0].clone();
        let tx = t.goal.target_region.center[0].round() as usize;
        let straight = vec![east; tx];
        assert!(
            matches!(
                verify_submission(&t, &straight),
                WitnessVerdict::Invalid { .. }
            ),
            "the pre-intervention straight path must fail under the intervened dynamics"
        );
    }

    #[test]
    fn generation_is_deterministic() {
        let a = generate(TaskFamily::GraphNavigation, 2, H_MAX);
        let b = generate(TaskFamily::GraphNavigation, 2, H_MAX);
        assert_eq!(a.id(), b.id());
        let names = |t: &TaskInstance| -> Vec<String> {
            t.gold.chosen_path.iter().map(|x| x.name.clone()).collect()
        };
        assert_eq!(names(&a), names(&b));
    }

    #[test]
    fn relabeling_preserves_validity_and_the_action_sequence() {
        let t = generate(TaskFamily::GraphNavigation, 1, H_MAX);
        let r = relabel(&t, 7, -3);
        assert_eq!(r.gold.verify(), WitnessVerdict::Valid);
        let names = |t: &TaskInstance| -> Vec<String> {
            t.gold.chosen_path.iter().map(|x| x.name.clone()).collect()
        };
        assert_eq!(
            names(&t),
            names(&r),
            "relabeling keeps the relative action sequence (a memorizer would break)"
        );
        assert_ne!(t.id(), r.id(), "relabeling changes the content id");
    }

    #[test]
    fn content_id_distinguishes_the_families() {
        let ids: Vec<u64> = FAMILIES
            .iter()
            .map(|&f| generate(f, 5, H_MAX).id())
            .collect();
        for (i, &a) in ids.iter().enumerate() {
            for &b in &ids[i + 1..] {
                assert_ne!(a, b, "two families share a content id (from index {i})");
            }
        }
    }

    #[test]
    fn confidence_band_is_ordinal_not_calibrated() {
        let ordered = [
            ConfidenceBand::None,
            ConfidenceBand::Low,
            ConfidenceBand::Medium,
            ConfidenceBand::High,
        ];
        let mut sorted = ordered;
        sorted.sort();
        assert_eq!(ordered, sorted, "confidence bands are ordinal, low to high");
    }

    #[test]
    fn verification_is_deterministic() {
        let t = generate(TaskFamily::ConstraintSatisfaction, 3, H_MAX);
        assert_eq!(t.gold.verify(), t.gold.verify());
    }
}
