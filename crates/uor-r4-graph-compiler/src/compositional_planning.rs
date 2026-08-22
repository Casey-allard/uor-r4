//! Compositional-planning reference semantics (#844, S4 item A), carrying
//! Amendment A1 (#843 increment 2).
//!
//! Frozen contract: `docs/compositional_planning_spec_844.md`, including its
//! appended section 11 (Amendment A1). A1 repairs the *generator*, not the
//! constitution: no frozen number moves. It makes instance difficulty scale
//! with the requested horizon so every frozen cell is non-vacuous; gives each
//! family real entity, vocabulary, topology, template, and operator-composition
//! variation so a split is a partition rather than a one-element set; derives
//! the content identity from problem content with the generation seed excluded,
//! so an identity-keyed control can actually fire; and leaves the strongest
//! non-oracle control headroom above the effect floor. Before it, a
//! structure-keyed memorization control saturated at a valid-plan rate of
//! 1.0000 and the S4 promotion statistic was at or below zero by construction.
//! The horizon-1 cell is gated on honest decline rather than valid-plan rate
//! (section 11.6): a one-step answer is a function of the observable state,
//! goal, and operator set, so retrieval is optimal there whatever the
//! generator does. This module
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

/// Number of distinct cells on each surface/structure split axis (Amendment
/// A1-b, #843). Every §2.2 axis a seed can vary carries this many cells, so a
/// disjoint fitting/held-out partition is constructible rather than vacuous.
pub const AXIS_CARDINALITY: u64 = 8;

/// The split-axis cell an instance belongs to (#844 §2.2, repaired by A1-b).
/// Fitting and evaluation data never share a cell on an axis being split.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SplitCell {
    /// Entity-naming scheme (surface; a semantic no-op).
    pub entity: u8,
    /// Operator surface vocabulary (surface; a semantic no-op).
    pub vocabulary: u8,
    /// Topology / dynamics configuration (semantic).
    pub topology: u8,
    /// Goal/prompt template naming (surface; a semantic no-op).
    pub template: u8,
    /// Reasoning horizon (semantic).
    pub horizon: usize,
}

impl SplitCell {
    /// The cell a `(seed, horizon)` pair lands in. Deterministic; the four
    /// surface/structure axes are independent base-`AXIS_CARDINALITY` digits of
    /// the seed, so seeds `0..AXIS_CARDINALITY.pow(4)` cover every combination.
    pub fn of(seed: u64, horizon: usize) -> Self {
        let c = AXIS_CARDINALITY;
        Self {
            entity: (seed % c) as u8,
            vocabulary: ((seed / c) % c) as u8,
            topology: ((seed / (c * c)) % c) as u8,
            template: ((seed / (c * c * c)) % c) as u8,
            horizon,
        }
    }

    /// A deterministic in-cell variant index. Mixed from the whole seed rather
    /// than read off its high digits, so the goal varies rapidly *within* a
    /// cell and is distributed identically across both halves of every axis:
    /// it is a nuisance parameter, not a fifth split axis. Taking it from the
    /// high digits instead left every seed in a sampling window sharing one
    /// goal, which by itself made a constant-plan control score 1.0000.
    fn variant(seed: u64) -> u64 {
        fnv1a64(&format!("in-cell-variant-{seed}"))
    }
}

/// A generated task instance with a replayable gold plan.
#[derive(Debug, Clone)]
pub struct TaskInstance {
    /// Task family.
    pub family: TaskFamily,
    /// Deterministic generation seed. Selects the split cell and the in-cell
    /// variant; it is **not** part of the content identity (A1-c).
    pub seed: u64,
    /// Planning horizon (bounded). The gold plan is exactly this long.
    pub horizon: usize,
    /// Initial state.
    pub initial_state: SemanticState,
    /// Goal predicate.
    pub goal: Goal,
    /// Forbidden regions.
    pub constraints: Vec<Constraint>,
    /// F5 only: the action *names* of the plan that is optimal under the
    /// *pre-intervention* (base) dynamics. Replaying it under this instance's
    /// intervened dynamics must be `Invalid` — that is what makes the family a
    /// counterfactual rather than a re-run. Empty for F1-F4.
    pub counterfactual_base: Vec<String>,
    /// Available actions/operators.
    pub actions: Vec<Action>,
    /// Replayable gold plan (or an honest decline when unsolvable).
    pub gold: PlanWitness,
}

impl TaskInstance {
    /// Content-addressed identity derived from the typed problem content alone.
    ///
    /// **Amendment A1-c (#843).** The generation seed is *excluded*: it is a
    /// generator input, not problem content, and mixing it in gave every
    /// instance a unique id while only a handful of structurally distinct
    /// problems existed — which made an identity-keyed memorization control
    /// unable to fire and therefore vacuous. Structurally identical instances
    /// now share an id. Deterministic; no clock, RNG, or hash-iteration order.
    pub fn id(&self) -> u64 {
        let mut canon = format!(
            "{}|entity={}|init={:?}|tmpl={}|goal={:?}:{}",
            self.family.label(),
            self.initial_state.id,
            round_vec(&self.initial_state.vector),
            self.goal.name,
            round_vec(&self.goal.target_region.center),
            (self.goal.target_region.radius * 1000.0).round() as i64,
        );
        let mut forbidden: Vec<String> = self
            .constraints
            .iter()
            .map(|c| format!("{:?}", round_vec(&c.forbidden_region.center)))
            .collect();
        forbidden.sort();
        for f in &forbidden {
            canon.push_str("|f=");
            canon.push_str(f);
        }
        for a in &self.actions {
            canon.push_str(&format!("|a={}:{:?}", a.name, round_vec(&a.delta_vector)));
        }
        fnv1a64(&canon)
    }

    /// The A1-b split cell this instance belongs to.
    pub fn split_cell(&self) -> SplitCell {
        SplitCell::of(self.seed, self.horizon)
    }
}

/// Resolve action names against a task's own action set, so a submitted plan is
/// evaluated under the task's dynamics rather than under whatever deltas the
/// caller happened to hold. `None` when a name is not in the task's vocabulary.
pub fn resolve_actions(task: &TaskInstance, names: &[String]) -> Option<Vec<Action>> {
    names
        .iter()
        .map(|n| task.actions.iter().find(|a| &a.name == n).cloned())
        .collect()
}

fn round_vec(v: &[f32]) -> Vec<i64> {
    v.iter().map(|x| x.round() as i64).collect()
}

fn state_key(s: &SemanticState) -> String {
    format!("{:?}|{:?}", round_vec(&s.vector), s.boolean_signature)
}

// ---------------------------------------------------------------------------
// Amendment A1-b (#843): split-axis vocabularies, effect sets, and topologies.
//
// Two kinds of axis, deliberately separated:
//
// * **Surface axes** - entity naming, operator vocabulary, goal template. These
//   are semantic no-ops, so splitting on them isolates exactly one failure
//   mode: a mechanism keyed on labels rather than on the typed dynamics.
// * **Semantic axes** - topology (the operator *effect set* plus the forbidden
//   configuration) and horizon. Splitting on these is what separates planning
//   from retrieval: under a held-out effect set, a plan memorised or retrieved
//   from fitting data no longer applies, while a planner re-plans with the
//   operators the instance actually offers.
//
// The effect sets are drawn from a **shared pool**, and the low and high halves
// of the topology axis each cover the whole pool. That is deliberate: an
// inducer fitted on the low half sees every effect it will need on the high
// half, so a held-out cell is a novel *composition*, never a novel primitive.
// A benchmark whose held-out cells needed unseen primitives would be
// unsolvable rather than hard.
// ---------------------------------------------------------------------------

/// Entity-naming schemes (surface axis).
const ENTITY_NAMES: [&str; 8] = [
    "start", "origin", "home", "base", "root", "anchor", "source", "depot",
];

/// Goal/prompt template names (surface axis).
const GOAL_TEMPLATES: [&str; 8] = [
    "reach", "arrive", "attain", "achieve", "satisfy", "fulfil", "obtain", "complete",
];

/// Operator surface vocabularies (surface axis). Vocabulary 0 keeps the
/// historical compass names, which pair with topology 0's axis-aligned effect
/// set and so keep the pinned seed-0 fixtures readable; the rest are abstract,
/// because an operator's name is a label and its effect comes from the topology.
const MOVE_VOCAB: [[&str; 4]; 8] = [
    ["east", "north", "west", "south"],
    ["alpha", "beta", "gamma", "delta"],
    ["push", "pull", "lift", "drop"],
    ["p0", "p1", "p2", "p3"],
    ["rho", "sigma", "tau", "upsilon"],
    ["step-1", "step-2", "step-3", "step-4"],
    ["mv-a", "mv-b", "mv-c", "mv-d"],
    ["op-w", "op-x", "op-y", "op-z"],
];

/// Operator *effect* sets for the grid families (semantic topology axis). Drawn
/// from the eight-effect pool {(+-1,0), (0,+-1), (+-1,+-1)}; topologies 0-3 and
/// topologies 4-7 each cover the whole pool.
const TOPOLOGY_EFFECTS: [[(i64, i64); 4]; 8] = [
    [(1, 0), (0, 1), (-1, 0), (0, -1)],
    [(1, 1), (1, -1), (-1, 1), (-1, -1)],
    [(1, 0), (0, 1), (1, 1), (-1, -1)],
    [(0, -1), (-1, 0), (1, -1), (-1, 1)],
    [(1, 0), (0, -1), (-1, 1), (1, 1)],
    [(0, 1), (-1, 0), (1, -1), (-1, -1)],
    [(1, 0), (-1, 0), (1, 1), (-1, -1)],
    [(0, 1), (0, -1), (1, -1), (-1, 1)],
];

/// Symbolic-operator surface vocabularies (surface axis).
const SYMBOL_VOCAB: [[&str; 3]; 8] = [
    ["op-a", "op-b", "op-c"],
    ["rewrite-a", "rewrite-b", "rewrite-c"],
    ["t1", "t2", "t3"],
    ["apply-alpha", "apply-beta", "apply-gamma"],
    ["reduce", "expand", "shift"],
    ["sigma", "tau", "rho"],
    ["f", "g", "h"],
    ["norm-a", "norm-b", "norm-c"],
];

/// Symbolic-operator effect sets (semantic topology axis for F2), drawn from the
/// six-effect pool {(2,0), (0,1), (-1,0), (1,1), (0,-1), (1,0)}. As above, the
/// low and high halves of the axis each cover the whole pool.
const SYMBOL_EFFECTS: [[(i64, i64); 3]; 8] = [
    [(2, 0), (0, 1), (-1, 0)],
    [(1, 1), (0, -1), (1, 0)],
    [(2, 0), (1, 1), (0, -1)],
    [(0, 1), (-1, 0), (1, 0)],
    [(1, 0), (0, 1), (1, 1)],
    [(2, 0), (0, -1), (-1, 0)],
    [(0, 1), (1, 1), (0, -1)],
    [(2, 0), (1, 0), (-1, 0)],
];

/// Surface names for the F5 twin operator (surface axis).
const TWIN_NAMES: [&str; 8] = [
    "twin-east",
    "alt-alpha",
    "mirror-push",
    "p0-prime",
    "rho-alt",
    "step-1b",
    "mv-a2",
    "op-w-alt",
];

fn cell(id: &str, x: i64, y: i64) -> SemanticState {
    SemanticState::new(id, vec![x as f32, y as f32], vec![0], 1.0)
}

fn goal_at(template: &str, x: i64, y: i64) -> Goal {
    Goal::new(
        template,
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

/// Forbidden-cell configuration for a family's topology cell. Index 0 keeps the
/// historical `(2, 0)` block for the grid families, so the pinned seed-0
/// fixtures keep their meaning. Every configuration is finite and the lattice is
/// unbounded, so no configuration can disconnect the state space.
fn obstacles(family: TaskFamily, topology: u8) -> Vec<(i64, i64)> {
    match family {
        // F2 and F5 carry their whole topology in the operator effect set.
        TaskFamily::SymbolicTransformation | TaskFamily::CounterfactualIntervention => Vec::new(),
        TaskFamily::ConstraintSatisfaction => {
            // A wall with exactly one gap: the classic typed-constraint shape.
            let col = 2 + i64::from(topology / 4);
            let gap = -1 + i64::from(topology % 4);
            (-2..=3).filter(|y| *y != gap).map(|y| (col, y)).collect()
        }
        _ => {
            const PATTERNS: [&[(i64, i64)]; 8] = [
                &[(2, 0)],
                &[],
                &[(1, 0)],
                &[(1, 0), (1, 1)],
                &[(2, 0), (2, 1)],
                &[(1, -1), (2, 0)],
                &[(1, 0), (2, 1), (3, -1)],
                &[(2, -1), (2, 0), (2, 1)],
            ];
            PATTERNS[usize::from(topology) % 8].to_vec()
        }
    }
}

fn named_actions(names: &[&str], deltas: &[(i64, i64)]) -> Vec<Action> {
    names
        .iter()
        .zip(deltas.iter())
        .map(|(n, (dx, dy))| Action::new(*n, vec![*dx as f32, *dy as f32], vec![0]))
        .collect()
}

/// The operator set for a family's (vocabulary, topology) cell. For F5 the twin
/// operator is listed first and carries its *intervened* effect; `base` selects
/// the pre-intervention effect instead.
fn family_actions(family: TaskFamily, vocabulary: u8, topology: u8, base: bool) -> Vec<Action> {
    let v = usize::from(vocabulary) % 8;
    let t = usize::from(topology) % 8;
    match family {
        TaskFamily::SymbolicTransformation => named_actions(&SYMBOL_VOCAB[v], &SYMBOL_EFFECTS[t]),
        TaskFamily::CounterfactualIntervention => {
            // Pre-intervention the twin duplicates the first operator's effect;
            // the declared intervention changes it to the second operator's.
            let effect = if base {
                TOPOLOGY_EFFECTS[t][0]
            } else {
                TOPOLOGY_EFFECTS[t][1]
            };
            let mut acts = vec![Action::new(
                TWIN_NAMES[v],
                vec![effect.0 as f32, effect.1 as f32],
                vec![0],
            )];
            acts.extend(named_actions(&MOVE_VOCAB[v], &TOPOLOGY_EFFECTS[t]));
            acts
        }
        _ => named_actions(&MOVE_VOCAB[v], &TOPOLOGY_EFFECTS[t]),
    }
}

/// Every cell first reached at exactly `depth` steps, in canonical order.
///
/// **Amendment A1-a (#843).** Placing the goal on this layer makes the shortest
/// valid plan exactly `depth` long, so an instance generated for horizon `H` is
/// a genuine `H`-step task. Before the amendment the goal was fixed and the
/// horizon merely truncated the search, which made every H = 1 and H = 2 cell
/// unsolvable and therefore unable to separate any mechanism. Falls back to the
/// deepest reachable layer, so the function is total.
fn layer_at_depth(
    initial: &SemanticState,
    constraints: &[Constraint],
    actions: &[Action],
    depth: usize,
) -> Vec<(i64, i64)> {
    let mut evaluator = TransitionEvaluator::new();
    for c in constraints {
        evaluator.add_constraint(c.clone());
    }
    let mut visited = std::collections::BTreeSet::new();
    visited.insert(state_key(initial));
    let mut frontier = vec![initial.clone()];
    let mut deepest = frontier.clone();
    for _ in 0..depth {
        let mut next: Vec<SemanticState> = Vec::new();
        for state in &frontier {
            for action in actions {
                if let Some(successor) = evaluator.apply(state, action)
                    && visited.insert(state_key(&successor))
                {
                    next.push(successor);
                }
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
        deepest = frontier.clone();
    }
    let mut cells: Vec<(i64, i64)> = deepest
        .iter()
        .map(|s| (s.vector[0].round() as i64, s.vector[1].round() as i64))
        .collect();
    cells.sort_unstable();
    cells.dedup();
    cells
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

/// Does `plan` reach `goal` from `initial` without entering a forbidden region?
fn plan_reaches(
    initial: &SemanticState,
    goal: &Goal,
    constraints: &[Constraint],
    plan: &[Action],
) -> bool {
    let mut evaluator = TransitionEvaluator::new();
    for c in constraints {
        evaluator.add_constraint(c.clone());
    }
    let mut state = initial.clone();
    for action in plan {
        match evaluator.apply(&state, action) {
            Some(next) => state = next,
            None => return false,
        }
    }
    goal.is_satisfied_by(&state)
}

/// The problem components of a task instance (grouped to keep `finish` within
/// the argument-count lint and to make the generators read declaratively).
struct Problem {
    initial: SemanticState,
    goal: Goal,
    constraints: Vec<Constraint>,
    actions: Vec<Action>,
    /// F5 only: action names of the plan optimal under the pre-intervention
    /// dynamics. Empty for F1-F4.
    counterfactual_base: Vec<String>,
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
        counterfactual_base,
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
        counterfactual_base,
        actions,
        gold,
    }
}

/// Pick F5's goal and its pre-intervention plan: the first layer cell, scanning
/// canonically from the in-cell variant, whose base-dynamics optimal plan uses
/// the twin operator and no longer reaches the goal once the twin's declared
/// effect changes. That is what makes the family a counterfactual rather than a
/// re-run, at every horizon including H = 1.
struct CounterfactualSearch<'a> {
    initial: &'a SemanticState,
    constraints: &'a [Constraint],
    actions: &'a [Action],
    base_actions: &'a [Action],
    layer: &'a [(i64, i64)],
    template: &'a str,
    depth: usize,
    variant: u64,
}

fn counterfactual_goal(search: &CounterfactualSearch<'_>) -> ((i64, i64), Vec<String>) {
    let CounterfactualSearch {
        initial,
        constraints,
        actions,
        base_actions,
        layer,
        template,
        depth,
        variant,
    } = *search;
    let start = (variant as usize) % layer.len();
    let twin = actions[0].name.clone();
    let mut fallback = None;
    for k in 0..layer.len() {
        let (gx, gy) = layer[(start + k) % layer.len()];
        let goal = goal_at(template, gx, gy);
        let Some(base_plan) = bfs_plan(initial, &goal, constraints, base_actions, depth) else {
            continue;
        };
        let names: Vec<String> = base_plan.iter().map(|a| a.name.clone()).collect();
        if fallback.is_none() {
            fallback = Some(((gx, gy), names.clone()));
        }
        if !names.contains(&twin) {
            continue;
        }
        let replay: Vec<Action> = names
            .iter()
            .filter_map(|n| actions.iter().find(|a| &a.name == n).cloned())
            .collect();
        if replay.len() == names.len() && !plan_reaches(initial, &goal, constraints, &replay) {
            return ((gx, gy), names);
        }
    }
    fallback.unwrap_or((layer[start], Vec::new()))
}

/// Generate a deterministic task instance for `family` from `seed` at `horizon`.
///
/// The seed selects the A1-b split cell - entity naming, operator vocabulary,
/// topology, goal template - plus an in-cell variant; the horizon sets the
/// task's *difficulty*, so the gold plan is exactly `horizon` steps long
/// (A1-a). Teacher-forced scope (S3 boundary).
pub fn generate(family: TaskFamily, seed: u64, horizon: usize) -> TaskInstance {
    let split = SplitCell::of(seed, horizon);
    let variant = SplitCell::variant(seed);
    let depth = horizon.clamp(1, H_MAX);
    let entity = ENTITY_NAMES[usize::from(split.entity) % 8];
    let template = GOAL_TEMPLATES[usize::from(split.template) % 8];
    let initial = cell(entity, 0, 0);
    let actions = family_actions(family, split.vocabulary, split.topology, false);
    let constraints: Vec<Constraint> = obstacles(family, split.topology)
        .into_iter()
        .enumerate()
        .map(|(i, (x, y))| forbid(&format!("blocked-{i}"), x, y))
        .collect();

    // The horizon-1 cell is the honest-decline cell (#843, maintainer sign-off
    // 2026-08-22). A one-step task's answer is a deterministic function of the
    // observable state, goal, and operator set, and the fitting split must
    // cover the whole operator pool or induction has nothing to learn from - so
    // a displacement-indexed retrieval baseline is optimal at horizon 1 by
    // construction and valid-plan rate cannot separate it from planning there.
    // A quarter of horizon-1 instances therefore place the goal one step beyond
    // the horizon: they are genuinely unsolvable within it and the correct
    // outcome is Decline(no_plan), which a replaying baseline cannot produce.
    // Deciding correctly requires evaluating reachability, which is planning.
    let beyond_horizon = depth == 1 && variant.is_multiple_of(4);
    let layer = layer_at_depth(
        &initial,
        &constraints,
        &actions,
        if beyond_horizon { depth + 1 } else { depth },
    );

    if family == TaskFamily::CounterfactualIntervention && !beyond_horizon {
        let base_actions = family_actions(family, split.vocabulary, split.topology, true);
        let ((gx, gy), base) = counterfactual_goal(&CounterfactualSearch {
            initial: &initial,
            constraints: &constraints,
            actions: &actions,
            base_actions: &base_actions,
            layer: &layer,
            template,
            depth,
            variant,
        });
        return finish(
            family,
            seed,
            depth,
            Problem {
                initial,
                goal: goal_at(template, gx, gy),
                constraints,
                actions,
                counterfactual_base: base,
            },
            false,
        );
    }

    let (gx, gy) = layer[(variant as usize) % layer.len()];
    finish(
        family,
        seed,
        depth,
        Problem {
            initial,
            goal: goal_at(template, gx, gy),
            constraints,
            actions,
            counterfactual_base: Vec::new(),
        },
        family == TaskFamily::MultiHopEvidence,
    )
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
        counterfactual_base: task.counterfactual_base.clone(),
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

    /// Amendment A1-a: an instance generated for horizon `H` is a genuine
    /// `H`-step task, at every frozen horizon including the low ones that were
    /// entirely unsolvable before the repair.
    #[test]
    fn gold_length_equals_the_requested_horizon_at_every_frozen_horizon() {
        for family in FAMILIES {
            for horizon in [1usize, 2, 4, 8] {
                for seed in 0..8u64 {
                    let t = generate(family, seed, horizon);
                    if t.gold.decline.is_some() {
                        // Horizon-1 decline instances are deliberate (the
                        // honest-decline cell); every other cell is solvable.
                        assert_eq!(horizon, 1, "{} seed {seed}", family.label());
                        continue;
                    }
                    assert_eq!(
                        t.gold.chosen_path.len(),
                        horizon,
                        "{} seed {seed} H={horizon} gold length",
                        family.label()
                    );
                    assert_eq!(t.gold.verify(), WitnessVerdict::Valid);
                }
            }
        }
    }

    #[test]
    fn verifier_rejects_a_step_into_a_forbidden_region() {
        // GraphNavigation seed 0 is topology cell 0, which keeps the historical
        // forbidden cell at (2, 0); vocabulary cell 0 names +x "east" first.
        let t = generate(TaskFamily::GraphNavigation, 0, H_MAX);
        let east = t.actions[0].clone();
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

    /// F5 is a counterfactual, not a re-run: the plan that was optimal under the
    /// pre-intervention dynamics must fail under the declared intervened
    /// dynamics, at every horizon.
    #[test]
    fn counterfactual_base_plan_is_invalid_under_the_intervened_dynamics() {
        for horizon in [1usize, 2, 4, 8, H_MAX] {
            for seed in 0..16u64 {
                let t = generate(TaskFamily::CounterfactualIntervention, seed, horizon);
                if t.gold.decline.is_some() {
                    // A horizon-1 decline instance has no counterfactual to
                    // express: there is no plan under either dynamics.
                    assert!(t.counterfactual_base.is_empty());
                    continue;
                }
                assert_eq!(t.gold.verify(), WitnessVerdict::Valid);
                assert_eq!(t.counterfactual_base.len(), horizon);
                let base = resolve_actions(&t, &t.counterfactual_base)
                    .expect("the base plan names the task's own operators");
                assert!(
                    matches!(verify_submission(&t, &base), WitnessVerdict::Invalid { .. }),
                    "seed {seed} H={horizon}: the pre-intervention plan must fail here"
                );
            }
        }
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

    /// Amendment A1-c: the identity is derived from problem content, so
    /// structurally identical instances at different seeds share it. Before the
    /// repair the seed was mixed in and they never did, which left an
    /// identity-keyed memorization control unable to fire.
    #[test]
    fn content_id_excludes_the_generation_seed() {
        let period = AXIS_CARDINALITY.pow(4);
        let a = generate(TaskFamily::GraphNavigation, 3, 8);
        let mut matched = false;
        for k in 1..64u64 {
            let b = generate(TaskFamily::GraphNavigation, 3 + k * period, 8);
            if round_vec(&b.goal.target_region.center) == round_vec(&a.goal.target_region.center) {
                assert_ne!(a.seed, b.seed, "the two instances differ by seed");
                assert_eq!(
                    a.id(),
                    b.id(),
                    "structurally identical instances must share a content id"
                );
                matched = true;
                break;
            }
        }
        assert!(matched, "expected a structurally identical instance");
    }

    /// Amendment A1-b: the four seed-varied axes are independent digits, so
    /// seeds `0..AXIS_CARDINALITY^4` cover every combination exactly once.
    #[test]
    fn split_cell_axes_are_independent_digits_of_the_seed() {
        let c = AXIS_CARDINALITY;
        let mut seen = std::collections::BTreeSet::new();
        for seed in 0..c.pow(4) {
            let cell = SplitCell::of(seed, 8);
            assert!(seen.insert((cell.entity, cell.vocabulary, cell.topology, cell.template)));
        }
        assert_eq!(seen.len() as u64, c.pow(4));
    }

    /// A1-a holds across surface cells too: the plan length is the horizon
    /// whichever entity, vocabulary, or template cell the seed lands in.
    #[test]
    fn plan_length_is_the_horizon_across_surface_cells() {
        let c = AXIS_CARDINALITY;
        for seed in [0u64, 1, 2, c, c + 1, c * c * c, c * c * c + 1] {
            let t = generate(TaskFamily::GraphNavigation, seed, 8);
            assert_eq!(
                t.gold.chosen_path.len(),
                8,
                "seed {seed} in cell {:?}",
                t.split_cell()
            );
            assert_eq!(t.gold.verify(), WitnessVerdict::Valid);
        }
    }

    /// The topology axis is semantic: two topology cells offer different
    /// operator effect sets, so a plan is not transferable between them by
    /// operator index alone. This is what separates planning from retrieval.
    #[test]
    fn topology_cells_offer_different_operator_effects() {
        let c = AXIS_CARDINALITY;
        let effects = |seed: u64| -> Vec<Vec<i64>> {
            generate(TaskFamily::GraphNavigation, seed, 8)
                .actions
                .iter()
                .map(|a| round_vec(&a.delta_vector))
                .collect()
        };
        // seeds 0 and 4 * c * c differ only in the topology digit.
        assert_ne!(
            effects(0),
            effects(4 * c * c),
            "two topology cells must not share an effect set"
        );
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
