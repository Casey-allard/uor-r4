//! Typed transition observations and deterministic induction (#843, S4 item B).
//!
//! Frozen contract: `docs/bounded_semantic_transitions_spec_843.md` §3. This is
//! the *offline compiler* half of the issue: it turns typed observations of
//! attempted state transitions into a bounded **operator rule set** the packed
//! sections carry and the deployed planner executes.
//!
//! The induced object is deliberately *schematic*, not grounded. A rule is
//! `(precondition mask and comparison block, typed effect delta, support,
//! ordinal band)`; grounding — computing `T(s, a)` — happens at plan time by
//! saturating integer addition on the packed slot vector. A grounded
//! `(state, action) -> state` table would scale with the reachable state space,
//! while a rule set scales with the operator vocabulary, and that is what makes
//! the frozen capacities and the P-4 hot path achievable at all.
//!
//! Execution scope: compiler / off-serving-path. Owned collections are
//! permitted here. Nothing in this module is deployed-serving evidence, and no
//! result here establishes any planning capability — that is the measured
//! question of the later increments.
//!
//! Boundaries carried in: ordinal confidence bands, never a calibrated
//! probability (S2 #823); teacher-forced scope, no free-running generation
//! (S3 #824).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use uor_r4_graph_format::plan::{
    EffectDelta, PLAN_ACTIONS_MAX, PLAN_RULES_MAX, PLAN_SLOTS_MAX, PreconditionMask, SlotVec,
};

use crate::compositional_planning::{ConfidenceBand, SplitCell, TaskFamily, TaskInstance};
use crate::semantic_state::{Constraint, SemanticState, TransitionEvaluator};

/// How many reachable states one instance is observed from, in canonical
/// breadth-first order. A declared, deterministic bound: the observation pass
/// never walks an unbounded state space, and truncation is at a stated limit
/// rather than wherever a timer expired.
pub const OBSERVED_STATES_PER_INSTANCE: usize = 32;

/// Default content-addressed shard count for the induction partition. The
/// emitted rule set does not depend on this value; the determinism instrument
/// asserts exactly that.
pub const DEFAULT_SHARDS: u32 = 16;

/// Minimum share of negative evidence, in parts per thousand, an observation
/// set must carry **when the observed tasks declared forbidden regions**.
///
/// The floor catches a sampling failure, not a task property: if a boundary
/// existed and nothing was ever observed being refused by it, the observation
/// pass never probed that boundary and any precondition induced from the set
/// would be unfalsifiable. Where the tasks declare no constraints at all the
/// dynamics really are total, there is no negative evidence to demand, and the
/// floor does not apply. It is deliberately low rather than tuned, because the
/// forbidden-region density of a task is a property of the task.
pub const NEGATIVE_FRACTION_FLOOR_MILLI: u32 = 10;

/// What happened when an operator was attempted from a state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Outcome {
    /// The operator applied and produced a successor.
    Applied,
    /// The operator's declared precondition did not hold.
    PreconditionFailed,
    /// The successor would have entered a forbidden region.
    ForbiddenRegion,
    /// The typed outcome could not be determined — resolved by decline, never
    /// by a default.
    Unknown,
}

/// Whether an observation supports a rule, refutes one, or is known to disagree
/// with another observation of the same typed situation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Polarity {
    /// The operator applied.
    Positive,
    /// The operator did not apply. Negative evidence is first class: a
    /// precondition cannot be induced without it.
    Negative,
    /// Assembled from sources known to disagree about the same typed situation.
    Conflicting,
}

/// The polarity an outcome carries.
pub fn polarity_of(outcome: Outcome) -> Polarity {
    match outcome {
        Outcome::Applied => Polarity::Positive,
        _ => Polarity::Negative,
    }
}

/// Typed support with a stable provenance identifier (F4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Provenance {
    /// Stable identifier of the supporting record.
    pub id: String,
    /// Ordinal support strength — never a calibrated probability.
    pub support: ConfidenceBand,
}

/// One observed attempt to apply one operator from one typed state.
///
/// The record describes a single attempted step and nothing else: there is no
/// field, and no combination of fields, from which a gold plan or a gold
/// terminal state is recoverable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionObservation {
    /// Task family the observation came from.
    pub family: TaskFamily,
    /// The split cell this observation belongs to.
    pub split_cell: SplitCell,
    /// Typed valuation before the attempt.
    pub from_slots: SlotVec,
    /// Typed valuation after it, when the operator applied.
    pub to_slots: Option<SlotVec>,
    /// Index of the operator within its instance's vocabulary.
    pub operator: u16,
    /// Surface name of the operator — a label, varying with the vocabulary axis.
    pub operator_name: String,
    /// Declared effect of the operator in this instance.
    pub declared_effect: EffectDelta,
    /// What happened.
    pub outcome: Outcome,
    /// The slots whose values the outcome depended on.
    pub read_mask: u8,
    /// The effect actually observed, when the operator applied.
    pub effect_delta: Option<EffectDelta>,
    /// Identity of the goal predicate in force.
    pub goal_ref: u64,
    /// Identities of the forbidden-region predicates in force.
    pub constraint_refs: Vec<u64>,
    /// Cited support (F4); empty otherwise.
    pub evidence: Vec<Provenance>,
    /// Supporting, refuting, or known-disagreeing.
    pub polarity: Polarity,
}

impl TransitionObservation {
    /// Content-addressed identity, derived from the typed content only — no
    /// generation seed, clock, RNG, or hash-iteration order. Two observations
    /// of the same typed situation share an id and collapse on ingest.
    pub fn sample_id(&self) -> u64 {
        let mut canon = format!(
            "{}|t{}|from={:?}|op={}:{:?}|out={:?}|to={:?}|goal={}",
            self.family.label(),
            self.split_cell.topology,
            self.from_slots.as_slice(),
            self.operator_name,
            self.declared_effect.as_slice(),
            self.outcome,
            self.to_slots.map(|s| s.as_slice().to_vec()),
            self.goal_ref,
        );
        let mut refs = self.constraint_refs.clone();
        refs.sort_unstable();
        for r in &refs {
            canon.push_str(&format!("|c{r}"));
        }
        fnv1a64(&canon)
    }
}

fn fnv1a64(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in s.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn predicate_id(kind: &str, center: &[f32], radius: f32) -> u64 {
    let cells: Vec<i64> = center.iter().map(|v| v.round() as i64).collect();
    fnv1a64(&format!(
        "{kind}|{cells:?}|{}",
        (radius * 1000.0).round() as i64
    ))
}

/// Project a reference semantic state onto the bounded deployed-form slot
/// valuation. `None` when the state carries more slots than the frozen
/// capacity — reported, never truncated.
pub fn slots_of(state: &SemanticState) -> Option<SlotVec> {
    if state.vector.len() > PLAN_SLOTS_MAX {
        return None;
    }
    let values: Vec<i16> = state
        .vector
        .iter()
        .map(|v| v.round().clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16)
        .collect();
    SlotVec::from_slice(&values)
}

/// The canonical breadth-first prefix of states reachable under the instance's
/// own dynamics, bounded by [`OBSERVED_STATES_PER_INSTANCE`].
fn reachable_prefix(task: &TaskInstance, evaluator: &TransitionEvaluator) -> Vec<SemanticState> {
    let mut seen: BTreeSet<Vec<i64>> = BTreeSet::new();
    let key =
        |s: &SemanticState| -> Vec<i64> { s.vector.iter().map(|v| v.round() as i64).collect() };
    let mut queue: VecDeque<SemanticState> = VecDeque::new();
    let mut out = Vec::new();
    seen.insert(key(&task.initial_state));
    queue.push_back(task.initial_state.clone());
    while let Some(state) = queue.pop_front() {
        out.push(state.clone());
        if out.len() >= OBSERVED_STATES_PER_INSTANCE {
            break;
        }
        for action in &task.actions {
            if let Some(next) = evaluator.apply(&state, action)
                && seen.insert(key(&next))
            {
                queue.push_back(next);
            }
        }
    }
    out
}

/// Compile the typed transition observations one task instance exposes.
///
/// Every operator is attempted from every state of the canonical reachable
/// prefix, and the outcome is recorded whether or not it applied — so
/// `ForbiddenRegion` and `PreconditionFailed` observations are emitted
/// alongside `Applied` ones rather than filtered out. The gold plan is never
/// read.
pub fn observe(task: &TaskInstance) -> Vec<TransitionObservation> {
    let mut permitted = TransitionEvaluator::new();
    for c in &task.constraints {
        permitted.add_constraint(c.clone());
    }
    let unconstrained = TransitionEvaluator::new();

    let goal_ref = predicate_id(
        "goal",
        &task.goal.target_region.center,
        task.goal.target_region.radius,
    );
    let constraint_refs: Vec<u64> = task
        .constraints
        .iter()
        .map(|c: &Constraint| {
            predicate_id(
                "forbid",
                &c.forbidden_region.center,
                c.forbidden_region.radius,
            )
        })
        .collect();

    let mut out = Vec::new();
    for state in reachable_prefix(task, &permitted) {
        let Some(from_slots) = slots_of(&state) else {
            continue;
        };
        let full_mask = mask_for(from_slots.len());
        for (index, action) in task.actions.iter().enumerate() {
            let Some(declared_effect) = EffectDelta::from_slice(&round_slots(&action.delta_vector))
            else {
                continue;
            };
            let applicable = unconstrained.apply(&state, action);
            let allowed = permitted.apply(&state, action);
            let (outcome, to_slots, read_mask) = match (&applicable, &allowed) {
                // The operator's own precondition refused it. The predicate is
                // opaque to the observer, so no read mask is claimed and the
                // observation is not used to induce one.
                (None, _) => (Outcome::PreconditionFailed, None, 0u8),
                // The operator applied but the successor is forbidden. The
                // constraint is a predicate over the destination, so every slot
                // determined the outcome.
                (Some(_), None) => (Outcome::ForbiddenRegion, None, full_mask),
                (Some(_), Some(next)) => match slots_of(next) {
                    Some(to) => (Outcome::Applied, Some(to), 0u8),
                    None => (Outcome::Unknown, None, 0u8),
                },
            };
            let effect_delta = to_slots.and_then(|to| EffectDelta::between(&from_slots, &to));
            let evidence = if task.gold.require_evidence_per_step && outcome == Outcome::Applied {
                vec![Provenance {
                    id: format!("prov-{}-{}", task.id(), index),
                    support: ConfidenceBand::High,
                }]
            } else {
                Vec::new()
            };
            out.push(TransitionObservation {
                family: task.family,
                split_cell: task.split_cell(),
                from_slots,
                to_slots,
                operator: index as u16,
                operator_name: action.name.clone(),
                declared_effect,
                outcome,
                read_mask,
                effect_delta,
                goal_ref,
                constraint_refs: constraint_refs.clone(),
                evidence,
                polarity: polarity_of(outcome),
            });
        }
    }
    out
}

fn round_slots(vector: &[f32]) -> Vec<i16> {
    vector
        .iter()
        .map(|v| v.round().clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16)
        .collect()
}

fn mask_for(len: usize) -> u8 {
    if len >= 8 { u8::MAX } else { (1u8 << len) - 1 }
}

// ---------------------------------------------------------------------------
// Split enforcement
// ---------------------------------------------------------------------------

/// Which split cells an induction pass may read. Fitting data and evaluation
/// data never share a cell on the axis being split, and the sealed cells
/// reserved for the final certification are never opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitPolicy {
    /// Topology cells reserved for evaluation.
    pub held_out_topologies: BTreeSet<u8>,
    /// Topology cells sealed for the final untouched-partition verdict.
    pub sealed_topologies: BTreeSet<u8>,
}

impl SplitPolicy {
    /// The frozen protocol: the low half of the semantic topology axis fits,
    /// the high half evaluates. A held-out cell therefore never shares an
    /// operator effect set or a forbidden configuration with fitting data.
    pub fn topology_halves() -> Self {
        Self {
            held_out_topologies: (4..8).collect(),
            sealed_topologies: BTreeSet::new(),
        }
    }

    /// Reserve `cells` as sealed in addition to the held-out half.
    pub fn sealing(mut self, cells: impl IntoIterator<Item = u8>) -> Self {
        self.sealed_topologies.extend(cells);
        self
    }

    /// Whether an observation from `cell` may reach the inducer.
    pub fn admits(&self, cell: &SplitCell) -> bool {
        !self.held_out_topologies.contains(&cell.topology)
            && !self.sealed_topologies.contains(&cell.topology)
    }
}

// ---------------------------------------------------------------------------
// The induced rule set
// ---------------------------------------------------------------------------

/// One induced operator rule: a precondition, a typed effect, and the evidence
/// behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionRule {
    /// When the rule applies.
    pub precondition: PreconditionMask,
    /// What it does, by saturating integer addition.
    pub effect: EffectDelta,
    /// How many distinct observations support it.
    pub support: u32,
    /// Ordinal band derived from `support` — never a calibrated probability.
    pub band: ConfidenceBand,
    /// Surface names seen for this effect. A compiler-side record only: the
    /// deployed table is keyed by the typed effect, because names vary with the
    /// vocabulary axis and carry no semantics.
    pub labels: BTreeSet<String>,
}

/// A declared conflict: within one topology cell, the same operator applied
/// from the same typed state was observed to produce more than one successor.
/// The conflicting effects are recorded and **not** emitted into the rule
/// table, so reaching them at plan time is `Decline(unknown)` rather than a
/// majority vote or a silent default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictRecord {
    /// The topology cell the disagreement was observed in.
    pub topology: u8,
    /// Surface name of the operator that disagreed.
    pub operator_name: String,
    /// The typed state it was applied from.
    pub from_slots: SlotVec,
    /// The distinct effects observed, canonically ordered.
    pub effects: Vec<EffectDelta>,
}

/// A bounded, canonically ordered operator rule set plus everything needed to
/// audit how it was induced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionRuleSet {
    /// Rules, ordered by `(precondition, effect)`. Deterministic.
    pub rules: Vec<TransitionRule>,
    /// Declared conflicts, ordered. Their effects are excluded from `rules`.
    pub conflicts: Vec<ConflictRecord>,
    /// Distinct observations that reached the reduction.
    pub observations: u32,
    /// How many of them were negative evidence.
    pub negatives: u32,
    /// Negative share in parts per thousand — an integer, so the record carries
    /// no floating point.
    pub negative_fraction_milli: u32,
    /// Topology cells that contributed. Auditable against the split policy.
    pub fitting_cells: BTreeSet<u8>,
}

impl TransitionRuleSet {
    /// Canonical bytes of the rule set — the determinism surface. Identical
    /// pinned inputs produce identical bytes.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = String::new();
        for rule in &self.rules {
            out.push_str(&format!(
                "r|{}|{:?}|{:?}|{}|{:?}\n",
                rule.precondition.read_mask(),
                rule.precondition,
                rule.effect.as_slice(),
                rule.support,
                rule.band
            ));
        }
        for conflict in &self.conflicts {
            out.push_str(&format!(
                "x|{}|{}|{:?}|{:?}\n",
                conflict.topology,
                conflict.operator_name,
                conflict.from_slots.as_slice(),
                conflict
                    .effects
                    .iter()
                    .map(|e| e.as_slice().to_vec())
                    .collect::<Vec<_>>()
            ));
        }
        out.into_bytes()
    }

    /// Content-addressed identity of the rule set.
    pub fn content_id(&self) -> u64 {
        fnv1a64(&String::from_utf8_lossy(&self.canonical_bytes()))
    }
}

/// Why an induction pass refused to emit a rule set. Every refusal is typed;
/// none of them is a silent empty result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefusalReason {
    /// The observation set carries too little negative evidence to express a
    /// precondition at all.
    InsufficientNegatives {
        /// Measured negative share, parts per thousand.
        observed_milli: u32,
        /// The frozen floor.
        floor_milli: u32,
    },
    /// An observation from a held-out or sealed cell reached the inducer.
    HeldOutLeakage {
        /// The offending cell.
        cell: SplitCell,
    },
    /// The induced set exceeds a frozen capacity.
    Capacity {
        /// Which capacity.
        what: &'static str,
        /// What the observations needed.
        needed: usize,
        /// The frozen limit.
        limit: usize,
    },
}

/// The outcome of an induction pass: a rule set, or a typed refusal. Modelled
/// as data rather than as an error, so a refusal is as inspectable as a
/// success and cannot be discarded by a `?`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InductionOutcome {
    /// A rule set was induced.
    Induced(TransitionRuleSet),
    /// Nothing was induced, for this stated reason.
    Refused(RefusalReason),
}

impl InductionOutcome {
    /// The rule set, when one was induced.
    pub fn induced(&self) -> Option<&TransitionRuleSet> {
        match self {
            InductionOutcome::Induced(set) => Some(set),
            InductionOutcome::Refused(_) => None,
        }
    }
}

/// Ordinal band for a support count. Frozen thresholds; ordinal only.
fn band_for(support: u32) -> ConfidenceBand {
    match support {
        0 => ConfidenceBand::None,
        1..=3 => ConfidenceBand::Low,
        4..=15 => ConfidenceBand::Medium,
        _ => ConfidenceBand::High,
    }
}

/// Induce an operator rule set from typed observations, at the default shard
/// count. See [`induce_with_shards`].
pub fn induce(observations: &[TransitionObservation], policy: &SplitPolicy) -> InductionOutcome {
    induce_with_shards(observations, policy, DEFAULT_SHARDS)
}

/// Induce an operator rule set from typed observations.
///
/// The pipeline is frozen: content-addressed partitioning, an ordered reduction
/// that is independent of shard count and of input order, deduplication on the
/// canonical rule key, declared-conflict detection, ordinal band assignment,
/// and split enforcement with a leakage scan. No step depends on a clock, an
/// RNG, or hash-iteration order.
pub fn induce_with_shards(
    observations: &[TransitionObservation],
    policy: &SplitPolicy,
    shards: u32,
) -> InductionOutcome {
    // 0. Leakage scan, before anything is read into the reduction.
    for observation in observations {
        if !policy.admits(&observation.split_cell) {
            return InductionOutcome::Refused(RefusalReason::HeldOutLeakage {
                cell: observation.split_cell,
            });
        }
    }

    // 1. Content-addressed partition. Identical typed observations share a
    //    sample id and collapse here, so support counts distinct evidence.
    let shard_count = u64::from(shards.max(1));
    let mut buckets: BTreeMap<u64, BTreeMap<u64, &TransitionObservation>> = BTreeMap::new();
    for observation in observations {
        let id = observation.sample_id();
        buckets
            .entry(id % shard_count)
            .or_default()
            .entry(id)
            .or_insert(observation);
    }

    // 2. Ordered reduction: shards in ascending index, samples in ascending id.
    let mut accum: BTreeMap<(PreconditionMask, EffectDelta), (u32, BTreeSet<String>)> =
        BTreeMap::new();
    let mut successors: BTreeMap<(u8, String, SlotVec), BTreeSet<EffectDelta>> = BTreeMap::new();
    let mut fitting_cells: BTreeSet<u8> = BTreeSet::new();
    let (mut total, mut negatives) = (0u32, 0u32);

    for shard in buckets.values() {
        for observation in shard.values() {
            total += 1;
            fitting_cells.insert(observation.split_cell.topology);
            if observation.polarity != Polarity::Positive {
                negatives += 1;
            }
            let Some(effect) = observation.effect_delta else {
                continue;
            };
            // 3. Deduplicate on the canonical rule key; duplicates raise support.
            let key = (unconditional_for(observation), effect);
            let entry = accum.entry(key).or_insert((0, BTreeSet::new()));
            entry.0 += 1;
            entry.1.insert(observation.operator_name.clone());
            // 4. Record the observed successor so a disagreement is visible.
            successors
                .entry((
                    observation.split_cell.topology,
                    observation.operator_name.clone(),
                    observation.from_slots,
                ))
                .or_default()
                .insert(effect);
        }
    }

    // 4b. Declared conflicts. A conflicted effect is not emitted.
    let mut conflicts = Vec::new();
    let mut conflicted: BTreeSet<EffectDelta> = BTreeSet::new();
    for ((topology, operator_name, from_slots), effects) in &successors {
        if effects.len() <= 1 {
            continue;
        }
        conflicted.extend(effects.iter().copied());
        conflicts.push(ConflictRecord {
            topology: *topology,
            operator_name: operator_name.clone(),
            from_slots: *from_slots,
            effects: effects.iter().copied().collect(),
        });
    }

    // 5. Negative-evidence floor, applied only where a refusal was possible.
    //
    //    If the observed tasks declared forbidden regions but nothing was ever
    //    observed to be refused by one, the observation pass never probed the
    //    boundary and any precondition induced from it would be unfalsifiable.
    //    Where the tasks declare no constraints at all, the dynamics really are
    //    total and there is no negative evidence to demand - the floor is about
    //    a sampling failure, not about a property of the task.
    let negative_fraction_milli = if total == 0 {
        0
    } else {
        (u64::from(negatives) * 1000 / u64::from(total)) as u32
    };
    let boundary_existed = observations
        .iter()
        .any(|observation| !observation.constraint_refs.is_empty());
    if boundary_existed && negative_fraction_milli < NEGATIVE_FRACTION_FLOOR_MILLI {
        return InductionOutcome::Refused(RefusalReason::InsufficientNegatives {
            observed_milli: negative_fraction_milli,
            floor_milli: NEGATIVE_FRACTION_FLOOR_MILLI,
        });
    }

    // 6. Emit, in canonical key order, excluding conflicted effects.
    let mut rules = Vec::new();
    let mut distinct_effects: BTreeSet<EffectDelta> = BTreeSet::new();
    for ((precondition, effect), (support, labels)) in accum {
        if conflicted.contains(&effect) {
            continue;
        }
        distinct_effects.insert(effect);
        rules.push(TransitionRule {
            precondition,
            effect,
            support,
            band: band_for(support),
            labels,
        });
    }

    // 7. Frozen capacities.
    if rules.len() > PLAN_RULES_MAX {
        return InductionOutcome::Refused(RefusalReason::Capacity {
            what: "rules",
            needed: rules.len(),
            limit: PLAN_RULES_MAX,
        });
    }
    if distinct_effects.len() > PLAN_ACTIONS_MAX {
        return InductionOutcome::Refused(RefusalReason::Capacity {
            what: "operators",
            needed: distinct_effects.len(),
            limit: PLAN_ACTIONS_MAX,
        });
    }

    InductionOutcome::Induced(TransitionRuleSet {
        rules,
        conflicts,
        observations: total,
        negatives,
        negative_fraction_milli,
        fitting_cells,
    })
}

/// The precondition an observation supports. The reference model's operators
/// carry no declared precondition, so an applied observation supports the
/// unconditional rule; the constraint that refused a *forbidden* successor is a
/// predicate over the destination and belongs to the query, not to the operator.
/// Keeping that distinction is what stops a forbidden region from being baked
/// into the dynamics.
fn unconditional_for(_observation: &TransitionObservation) -> PreconditionMask {
    PreconditionMask::unconditional()
}
