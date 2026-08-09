//! Semantic State Space ($S$) and Typed Dynamics ($T: S \times A \to S$)
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §§2, 3, 12;
//! `docs/formal_vocabulary.md` §3; GitHub Issue #124.
//!
//! This module provides the compiler/reference representation for:
//! - Semantic states ($s \in S$) with latent vector projections and bit-signatures
//! - Semantic regions ($R_i \subseteq S$) defined as hyper-ball/bounding predicates
//! - Action/Semantic Operators ($A$) with explicit preconditions and postconditions
//! - Typed transitions ($T: S \times A \to S$) returning structured outcomes
//! - Beliefs, goals, and constraints evaluated over state space $S$
//! - Deterministic trajectory execution and replay with bounded termination limits

use std::collections::HashMap;
use std::fmt;

/// Classification of state fields into compiler-only vs lowered runtime fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldLoweringTier {
    /// Reference/compiler-only field (floating-point vectors, arbitrary closures).
    CompilerReferenceOnly,
    /// Fixed-capacity lowered runtime representation (bit-signatures, integer indices).
    LoweredRuntime,
}

/// A semantic state $s \in S$ in continuous latent space and integer bit-signature space.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticState {
    /// Unique identifier for the state.
    pub id: String,
    /// Continuous latent state vector projection (Compiler/Reference-only).
    pub vector: Vec<f32>,
    /// Bit-packed Boolean semantic signature (Lowered Runtime).
    pub boolean_signature: Vec<u64>,
    /// State confidence / probability measure in $[0.0, 1.0]$.
    pub confidence: f32,
    /// Structured state metadata (labels, properties).
    pub attributes: HashMap<String, String>,
}

impl SemanticState {
    /// Create a new semantic state with vector and Boolean signature.
    pub fn new(
        id: impl Into<String>,
        vector: Vec<f32>,
        boolean_signature: Vec<u64>,
        confidence: f32,
    ) -> Self {
        Self {
            id: id.into(),
            vector,
            boolean_signature,
            confidence: confidence.clamp(0.0, 1.0),
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute metadata key-value pair.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Compute Euclidean distance to another state vector. `None` when the two
    /// vectors have different dimensions: no distance is a product of
    /// incomparable states (R5 — the absence of a product rather than a raised
    /// error).
    pub fn distance_to(&self, other: &Self) -> Option<f32> {
        if self.vector.len() != other.vector.len() {
            return None;
        }
        let sum_sq: f32 = self
            .vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        Some(sum_sq.sqrt())
    }

    /// Compute Hamming distance between bit signatures. `None` when the two
    /// signatures have different lengths (incomparable states — R5).
    pub fn hamming_distance(&self, other: &Self) -> Option<u32> {
        if self.boolean_signature.len() != other.boolean_signature.len() {
            return None;
        }
        let dist: u32 = self
            .boolean_signature
            .iter()
            .zip(other.boolean_signature.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum();
        Some(dist)
    }

    /// Record lowering tier specifications for state fields.
    pub fn field_lowering_tiers() -> HashMap<&'static str, FieldLoweringTier> {
        let mut map = HashMap::new();
        map.insert("vector", FieldLoweringTier::CompilerReferenceOnly);
        map.insert("attributes", FieldLoweringTier::CompilerReferenceOnly);
        map.insert("boolean_signature", FieldLoweringTier::LoweredRuntime);
        map.insert("id", FieldLoweringTier::LoweredRuntime);
        map.insert("confidence", FieldLoweringTier::LoweredRuntime);
        map
    }
}

/// A semantic region $R_i \subseteq S$ in state space.
#[derive(Debug, Clone)]
pub struct Region {
    /// Region identifier.
    pub id: String,
    /// Center of the region in latent space.
    pub center: Vec<f32>,
    /// Radius bound of the hyper-ball.
    pub radius: f32,
    /// Region label or category description.
    pub label: String,
}

impl Region {
    /// Create a new hyper-ball region centered at `center` with radius `radius`.
    pub fn new(
        id: impl Into<String>,
        center: Vec<f32>,
        radius: f32,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            center,
            radius,
            label: label.into(),
        }
    }

    /// Check if a semantic state lies within this region's boundary.
    pub fn contains(&self, state: &SemanticState) -> bool {
        if self.center.len() != state.vector.len() {
            return false;
        }
        let dist_sq: f32 = self
            .center
            .iter()
            .zip(state.vector.iter())
            .map(|(c, v)| (c - v).powi(2))
            .sum();
        dist_sq.sqrt() <= self.radius
    }
}

/// A belief predicate / evaluation function over state space $S$.
#[derive(Debug, Clone)]
pub struct Belief {
    /// Name / description of the belief.
    pub name: String,
    /// Target region or state predicate.
    pub target_region: Region,
    /// Prior probability weight $[0.0, 1.0]$.
    pub prior: f32,
}

impl Belief {
    pub fn new(name: impl Into<String>, target_region: Region, prior: f32) -> Self {
        Self {
            name: name.into(),
            target_region,
            prior: prior.clamp(0.0, 1.0),
        }
    }

    /// Evaluate belief likelihood given a state.
    pub fn evaluate(&self, state: &SemanticState) -> f32 {
        if self.target_region.contains(state) {
            (state.confidence * 0.8 + self.prior * 0.2).min(1.0)
        } else {
            (self.prior * 0.1).max(0.0)
        }
    }
}

/// A goal defining desired target subsets of $S$.
#[derive(Debug, Clone)]
pub struct Goal {
    /// Goal name.
    pub name: String,
    /// Target region requirements.
    pub target_region: Region,
    /// Minimum required state confidence.
    pub min_confidence: f32,
}

impl Goal {
    pub fn new(name: impl Into<String>, target_region: Region, min_confidence: f32) -> Self {
        Self {
            name: name.into(),
            target_region,
            min_confidence,
        }
    }

    /// Check if a state satisfies the goal criteria.
    pub fn is_satisfied_by(&self, state: &SemanticState) -> bool {
        state.confidence >= self.min_confidence && self.target_region.contains(state)
    }
}

/// A invariant constraint defining forbidden state conditions in $S$.
#[derive(Debug, Clone)]
pub struct Constraint {
    /// Constraint name.
    pub name: String,
    /// Forbidden region (state inside this region is invalid/forbidden).
    pub forbidden_region: Region,
}

impl Constraint {
    pub fn new(name: impl Into<String>, forbidden_region: Region) -> Self {
        Self {
            name: name.into(),
            forbidden_region,
        }
    }

    /// Check if a state violates this constraint.
    pub fn is_violated_by(&self, state: &SemanticState) -> bool {
        self.forbidden_region.contains(state)
    }
}

/// Type alias for thread-safe state predicates.
pub type StatePredicate = std::sync::Arc<dyn Fn(&SemanticState) -> bool + Send + Sync>;

/// A semantic action/operator $A$ acting on state space $S$.
#[derive(Clone)]
pub struct Action {
    /// Action name.
    pub name: String,
    /// Latent vector displacement / transformation applied by the action.
    pub delta_vector: Vec<f32>,
    /// Bit-flip mask applied to boolean signatures.
    pub mask_flip: Vec<u64>,
    /// Confidence multiplier (scaling state confidence post-action).
    pub confidence_scale: f32,
    /// Precondition predicate on state before applying action.
    pub precondition: Option<StatePredicate>,
    /// Postcondition predicate on state after applying action.
    pub postcondition: Option<StatePredicate>,
}

impl Action {
    pub fn new(name: impl Into<String>, delta_vector: Vec<f32>, mask_flip: Vec<u64>) -> Self {
        Self {
            name: name.into(),
            delta_vector,
            mask_flip,
            confidence_scale: 1.0,
            precondition: None,
            postcondition: None,
        }
    }

    pub fn with_precondition<F>(mut self, pred: F) -> Self
    where
        F: Fn(&SemanticState) -> bool + Send + Sync + 'static,
    {
        self.precondition = Some(std::sync::Arc::new(pred));
        self
    }

    pub fn with_postcondition<F>(mut self, pred: F) -> Self
    where
        F: Fn(&SemanticState) -> bool + Send + Sync + 'static,
    {
        self.postcondition = Some(std::sync::Arc::new(pred));
        self
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("name", &self.name)
            .field("delta_vector", &self.delta_vector)
            .field("mask_flip", &self.mask_flip)
            .field("confidence_scale", &self.confidence_scale)
            .finish()
    }
}

/// Deterministic transition evaluator $T: S \times A \to S$.
#[derive(Debug)]
pub struct TransitionEvaluator {
    /// Registered constraints enforced on all state transitions.
    pub constraints: Vec<Constraint>,
}

impl TransitionEvaluator {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Evaluate transition $T(s, a) \to S$ with precondition and constraint
    /// enforcement. `None` when the transition produces no next state: the
    /// precondition fails, the action's dimensions do not match the state, the
    /// result violates a constraint (forbidden state), or the postcondition
    /// fails. All are properties of the requested (state, action) pair, so no
    /// next state is a product of them (R5 — the absence of a product rather
    /// than a raised error).
    pub fn apply(&self, state: &SemanticState, action: &Action) -> Option<SemanticState> {
        // 1. Check Preconditions
        if matches!(action.precondition.as_ref(), Some(prec) if !prec(state)) {
            return None;
        }

        // 2. Compute Target Vector & Bit Signature
        if state.vector.len() != action.delta_vector.len() {
            return None;
        }
        if state.boolean_signature.len() != action.mask_flip.len() {
            return None;
        }
        let new_vector: Vec<f32> = state
            .vector
            .iter()
            .zip(action.delta_vector.iter())
            .map(|(v, d)| v + d)
            .collect();

        let new_sig: Vec<u64> = state
            .boolean_signature
            .iter()
            .zip(action.mask_flip.iter())
            .map(|(s, m)| s ^ m)
            .collect();

        let new_confidence = (state.confidence * action.confidence_scale).clamp(0.0, 1.0);
        let next_id = format!("{}_{}", state.id, action.name);

        let next_state = SemanticState::new(next_id, new_vector, new_sig, new_confidence);

        // 3. Check Constraints (Forbidden States)
        for constraint in &self.constraints {
            if constraint.is_violated_by(&next_state) {
                return None;
            }
        }

        // 4. Check Postconditions
        if matches!(action.postcondition.as_ref(), Some(postc) if !postc(&next_state)) {
            return None;
        }

        Some(next_state)
    }
}

impl Default for TransitionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// A bounded trajectory of states and actions with step limit enforcement.
#[derive(Debug, Clone)]
pub struct Trajectory {
    /// Maximum allowed steps in trajectory.
    pub max_steps: usize,
    /// Sequence of states along trajectory.
    pub states: Vec<SemanticState>,
    /// Actions applied at each step.
    pub actions: Vec<String>,
}

impl Trajectory {
    pub fn new(initial_state: SemanticState, max_steps: usize) -> Self {
        Self {
            max_steps,
            states: vec![initial_state],
            actions: Vec::new(),
        }
    }

    /// Current head state of trajectory.
    pub fn current_state(&self) -> &SemanticState {
        self.states
            .last()
            .expect("trajectory must have initial state")
    }

    /// Step trajectory forward using an action and transition evaluator. `None`
    /// when the trajectory cannot advance: the step limit is reached, or the
    /// transition produces no next state (R5 — the absence of a product rather
    /// than a raised error).
    pub fn step(
        &mut self,
        action: &Action,
        evaluator: &TransitionEvaluator,
    ) -> Option<&SemanticState> {
        if self.actions.len() >= self.max_steps {
            return None;
        }
        let current = self.current_state();
        let next_state = evaluator.apply(current, action)?;
        self.actions.push(action.name.clone());
        self.states.push(next_state);
        Some(self.current_state())
    }

    /// Deterministic trajectory replay from initial state across actions. `None`
    /// when any step cannot advance (R5).
    pub fn replay(
        initial_state: SemanticState,
        actions: &[Action],
        evaluator: &TransitionEvaluator,
        max_steps: usize,
    ) -> Option<Self> {
        let mut traj = Self::new(initial_state, max_steps);
        for action in actions {
            traj.step(action, evaluator)?;
        }
        Some(traj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_state_distance_and_hamming() {
        let s1 = SemanticState::new("s1", vec![0.0, 1.0, 0.0], vec![0b1010], 0.9);
        let s2 = SemanticState::new("s2", vec![3.0, 5.0, 0.0], vec![0b1100], 0.8);

        let euclidean = s1.distance_to(&s2).unwrap();
        assert_eq!(euclidean, 5.0); // sqrt(3^2 + 4^2) = 5

        let hamming = s1.hamming_distance(&s2).unwrap();
        assert_eq!(hamming, 2); // 0b1010 ^ 0b1100 = 0b0110 -> 2 bits set
    }

    #[test]
    fn test_typed_transition_and_preconditions() {
        let s0 = SemanticState::new("s0", vec![0.0, 0.0], vec![0b00], 1.0);
        let action_ok = Action::new("move_up", vec![0.0, 1.0], vec![0b01])
            .with_precondition(|s| s.vector[1] >= 0.0);

        let evaluator = TransitionEvaluator::new();
        let s1 = evaluator.apply(&s0, &action_ok).unwrap();
        assert_eq!(s1.id, "s0_move_up");
        assert_eq!(s1.vector, vec![0.0, 1.0]);
        assert_eq!(s1.boolean_signature, vec![0b01]);

        // Test precondition failure
        let invalid_s = SemanticState::new("invalid", vec![0.0, -1.0], vec![0b00], 1.0);
        assert!(evaluator.apply(&invalid_s, &action_ok).is_none());
    }

    #[test]
    fn test_constraint_enforcement_forbidden_state() {
        let s0 = SemanticState::new("s0", vec![0.0, 0.0], vec![0b00], 1.0);
        let action = Action::new("enter_danger", vec![5.0, 5.0], vec![0b11]);

        let danger_zone = Region::new("danger", vec![5.0, 5.0], 1.0, "Hazard");
        let constraint = Constraint::new("no_danger", danger_zone);

        let mut evaluator = TransitionEvaluator::new();
        evaluator.add_constraint(constraint);

        assert!(evaluator.apply(&s0, &action).is_none());
    }

    #[test]
    fn test_goal_satisfaction_and_belief_evaluation() {
        let target_region = Region::new("target", vec![10.0, 10.0], 2.0, "Goal Zone");
        let goal = Goal::new("reach_target", target_region.clone(), 0.8);
        let belief = Belief::new("target_belief", target_region, 0.5);

        let s_out = SemanticState::new("s_out", vec![0.0, 0.0], vec![0b00], 0.9);
        let s_in = SemanticState::new("s_in", vec![10.0, 11.0], vec![0b01], 0.9);

        assert!(!goal.is_satisfied_by(&s_out));
        assert!(goal.is_satisfied_by(&s_in));

        assert!(belief.evaluate(&s_in) > belief.evaluate(&s_out));
    }

    #[test]
    fn test_bounded_trajectory_and_max_steps() {
        let s0 = SemanticState::new("s0", vec![0.0], vec![0], 1.0);
        let step_action = Action::new("inc", vec![1.0], vec![0]);
        let evaluator = TransitionEvaluator::new();

        let mut traj = Trajectory::new(s0, 2);
        assert!(traj.step(&step_action, &evaluator).is_some());
        assert!(traj.step(&step_action, &evaluator).is_some());

        // 3rd step should exceed max_steps = 2
        assert!(traj.step(&step_action, &evaluator).is_none());
    }
}
