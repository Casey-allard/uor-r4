//! `reduction/` namespace — Euler reduction sequential composition.
//!
//! The `reduction/` namespace formalizes the sequential composition of
//! the foundation's nine inter-algebra maps into a parameterized reduction
//! pipeline, parameterized by the phase angle \u{03a9} = e^{i\u{03c0}/6}. It
//! defines the six-stage pipeline, phase gate attestation, complex conjugate
//! rollback, and epoch-based temporal segmentation. (The internal proof
//! identifiers for the inter-algebra maps live in the `proof/` and `op/`
//! namespaces; consumer-facing names use `inter-algebra map` / `reduction
//! stage` vocabulary.)
//!
//! - **Amendment 63**: 10 classes, 25 properties, reduction core formalization
//! - **Amendment 64**: 10 classes, 20 properties, reduction expansion
//!   (predicates, guards, effects, service windows, transactions,
//!   preflight checks, pipeline termination)
//! - **Amendment 65**: 6 classes, 11 properties, reduction completion
//!   (feasibility results, lease lifecycle, back-pressure, deferred queries)
//! - **Amendment 84**: 1 class, 6 properties, 1 individual (CompileUnit,
//!   reduction admission, preflight ordering, budget solvency)
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `reduction/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "reduction",
            iri: NS_REDUCTION,
            label: "UOR Euler Reduction",
            comment: "Sequential composition of the foundation's nine inter-algebra \
                      maps into a parameterized reduction pipeline. Defines stages, \
                      phase gates, rollback, and epochs.",
            space: Space::Kernel,
            imports: &[
                NS_OP,
                NS_STATE,
                NS_PARTITION,
                NS_RESOLVER,
                NS_MORPHISM,
                NS_OBSERVABLE,
                NS_PREDICATE,
                NS_EFFECT,
                NS_SCHEMA,
                NS_U,
            ],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/reduction/EulerReduction",
            label: "EulerReduction",
            comment: "The composite endofunctor \u{03c8} = \u{03c8}_9 \u{2218} \u{2026} \
                      \u{2218} \u{03c8}_1, parameterized by \u{03a9} = e^{i\u{03c0}/6}.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/PhaseRotationScheduler",
            label: "PhaseRotationScheduler",
            comment: "Schedule \u{03a9}\u{2070}, \u{03a9}\u{00b9}, \u{2026}, \u{03a9}\u{2075} \
                      assigning a phase angle to each stage of the reduction.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/TargetConvergenceAngle",
            label: "TargetConvergenceAngle",
            comment: "The angle at which the reduction terminates \
                      (default: \u{03c0}).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/PhaseGateAttestation",
            label: "PhaseGateAttestation",
            comment: "Validation at each stage boundary checking that the \
                      accumulated phase angle matches the expected \u{03a9}^k.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/ComplexConjugateRollback",
            label: "ComplexConjugateRollback",
            comment: "Recovery operation when a phase gate fails: z \u{2192} z\u{0304}. \
                      Involutory: applying twice yields the original value.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/ReductionStep",
            label: "ReductionStep",
            comment: "A named stage of the reduction. The standard reduction has \
                      six stages (Initialization through Convergence).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/ReductionState",
            label: "ReductionState",
            comment: "State of reduction execution at a specific point, including \
                      the current stage, phase angle, and pinned mask.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/ReductionRule",
            label: "ReductionRule",
            comment: "Guard-effect pair governing stage transitions in the \
                      reduction. The guard must be satisfied before the effect \
                      is applied.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/Epoch",
            label: "Epoch",
            comment: "Temporal segment of reduction execution. Each epoch \
                      represents one complete pass through the reduction stages.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/EpochBoundary",
            label: "EpochBoundary",
            comment: "Transition between epochs. Carries metadata about \
                      the epoch boundary crossing.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 64: Reduction Expansion (10 classes)
        Class {
            id: "https://uor.foundation/reduction/PredicateExpression",
            label: "PredicateExpression",
            comment: "A Boolean expression over the reduction state. \
                      The atomic guard unit.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/GuardExpression",
            label: "GuardExpression",
            comment: "A conjunction of PredicateExpressions that must hold \
                      for a transition to fire.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/TransitionEffect",
            label: "TransitionEffect",
            comment: "State changes applied when a transition fires. \
                      Contains PropertyBind steps.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/PropertyBind",
            label: "PropertyBind",
            comment: "A single site pinning: target site + value.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/ReductionAdvance",
            label: "ReductionAdvance",
            comment: "Advancement from one ReductionStep to the next.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/ServiceWindow",
            label: "ServiceWindow",
            comment: "A sliding window over recent epochs providing \
                      BaseContext.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/ReductionTransaction",
            label: "ReductionTransaction",
            comment: "An atomic group of state changes within the reduction.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/PipelineSuccess",
            label: "PipelineSuccess",
            comment: "Successful termination (FullGrounding).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/PipelineFailureReason",
            label: "PipelineFailureReason",
            comment: "Typed failure: DispatchMiss, GroundingFailure, \
                      ConvergenceStall, etc.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/PreflightCheck",
            label: "PreflightCheck",
            comment: "A pre-execution validation: feasibility, dispatch \
                      coverage, coherence.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 65: Reduction Completion (6 classes)
        Class {
            id: "https://uor.foundation/reduction/FeasibilityResult",
            label: "FeasibilityResult",
            comment: "Result of a preflight check: feasibility witness or \
                      infeasibility witness.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/LeaseState",
            label: "LeaseState",
            comment: "Lifecycle of a partitioned context lease: Pending \
                      \u{2192} Active \u{2192} Released/Expired/Suspended.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/ManagedLease",
            label: "ManagedLease",
            comment: "A context lease with lifecycle tracking.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/LeaseCheckpoint",
            label: "LeaseCheckpoint",
            comment: "Snapshot of lease state at a point in time.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/BackPressureSignal",
            label: "BackPressureSignal",
            comment: "Flow control when reduction produces faster than \
                      downstream can consume.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/DeferredQuerySet",
            label: "DeferredQuerySet",
            comment: "Queries postponed to a future epoch.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 71: SubleaseTransfer
        Class {
            id: "https://uor.foundation/reduction/SubleaseTransfer",
            label: "SubleaseTransfer",
            comment: "Transfer of a lease from one computation to another.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 71: Predicate subclasses (10)
        Class {
            id: "https://uor.foundation/reduction/ComparisonPredicate",
            label: "ComparisonPredicate",
            comment: "Predicate comparing a state field against a value.",
            subclass_of: &["https://uor.foundation/reduction/PredicateExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/ConjunctionPredicate",
            label: "ConjunctionPredicate",
            comment: "Conjunction (AND) of multiple predicates.",
            subclass_of: &["https://uor.foundation/reduction/PredicateExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/DisjunctionPredicate",
            label: "DisjunctionPredicate",
            comment: "Disjunction (OR) of multiple predicates.",
            subclass_of: &["https://uor.foundation/reduction/PredicateExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/NegationPredicate",
            label: "NegationPredicate",
            comment: "Negation (NOT) of a single predicate.",
            subclass_of: &["https://uor.foundation/reduction/PredicateExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/MembershipPredicate",
            label: "MembershipPredicate",
            comment: "Predicate testing membership of an element in a set.",
            subclass_of: &["https://uor.foundation/reduction/PredicateExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/GroundingPredicate",
            label: "GroundingPredicate",
            comment: "Predicate testing whether grounding exceeds a threshold.",
            subclass_of: &["https://uor.foundation/reduction/PredicateExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/SiteCoveragePredicate",
            label: "SiteCoveragePredicate",
            comment: "Predicate testing whether a site coverage target is met.",
            subclass_of: &["https://uor.foundation/reduction/PredicateExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/EqualsPredicate",
            label: "EqualsPredicate",
            comment: "Predicate testing equality of two expressions.",
            subclass_of: &["https://uor.foundation/reduction/PredicateExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/NonNullPredicate",
            label: "NonNullPredicate",
            comment: "Predicate testing that a field is non-null.",
            subclass_of: &["https://uor.foundation/reduction/PredicateExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/reduction/QuerySubtypePredicate",
            label: "QuerySubtypePredicate",
            comment: "Predicate testing whether a query is a subtype of a given type.",
            subclass_of: &["https://uor.foundation/reduction/PredicateExpression"],
            disjoint_with: &[],
        },
        // Amendment 84: CompileUnit
        Class {
            id: "https://uor.foundation/reduction/CompileUnit",
            label: "CompileUnit",
            comment: "The typed input graph submitted to the reduction pipeline. \
                      Packages a root Term, target quantum level, verification \
                      domains, and thermodynamic budget. Stage 0 accepts exactly \
                      one CompileUnit and initializes the reduction state vector \
                      from it.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // v0.2.1: Parametric PipelineFailure variant-field metadata.
        // The Rust codegen reads FailureField individuals to generate one
        // PipelineFailure enum variant per PipelineFailureReason individual,
        // with named Rust fields read from the ofFailure / fieldName / \
        // fieldType properties on each FailureField.
        Class {
            id: "https://uor.foundation/reduction/FailureField",
            label: "FailureField",
            comment: "An ontology fact describing a single field on a \
                      reduction:PipelineFailureReason variant. The v0.2.1 \
                      Rust codegen walks FailureField individuals filtered \
                      by ofFailure to assemble the PipelineFailure enum's \
                      variant fields. Adding a new field to a failure \
                      variant requires only adding a new FailureField \
                      individual.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // v0.2.1 Phase 7a.1: Parametric SAT-decider bounds. The Rust and
        // Lean pipeline driver codegen read SatBound individuals at
        // codegen time to bake the bounded 2-SAT / Horn-SAT decider
        // constants (TWO_SAT_MAX_VARS, HORN_SAT_MAX_VARS) into the
        // emitted source. No hand-written bound in Rust/Lean source.
        Class {
            id: "https://uor.foundation/reduction/SatBound",
            label: "SatBound",
            comment: "Declarative bound for SAT-fragment deciders. The v0.2.1 \
                      pipeline driver reads these individuals at codegen time \
                      to emit the bounded iterative Tarjan SCC (2-SAT) and \
                      unit propagation (Horn-SAT) implementations.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // v0.2.1 Phase 7a.2: Parametric timing bounds for preflight/runtime
        // stages. v0.2.1 ships the stubs returning Ok(()) but the bounds
        // are carried in the ontology so future releases can plug in real
        // timing checks without an API break.
        Class {
            id: "https://uor.foundation/reduction/TimingBound",
            label: "TimingBound",
            comment: "Declarative timing bound for reduction pipeline stages. \
                      Values in nanoseconds at the v0.2.1 reference hardware.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // EulerReduction properties
        Property {
            id: "https://uor.foundation/reduction/phaseParameter",
            label: "phaseParameter",
            comment: "The base phase parameter \u{03a9} for this reduction \
                      (e.g., e^{i\u{03c0}/6}).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/EulerReduction"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/stageCount",
            label: "stageCount",
            comment: "The number of stages in this reduction.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/EulerReduction"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/convergenceAngle",
            label: "convergenceAngle",
            comment: "The cumulative phase angle at which the reduction converges.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/EulerReduction"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/composedOfMaps",
            label: "composedOfMaps",
            comment: "The ordered list of \u{03c8}-maps that compose this reduction.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/reduction/EulerReduction"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // PhaseRotationScheduler properties
        Property {
            id: "https://uor.foundation/reduction/rotationSchedule",
            label: "rotationSchedule",
            comment: "String representation of the rotation schedule \
                      \u{03a9}\u{2070}, \u{03a9}\u{00b9}, \u{2026}, \u{03a9}\u{2075}.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PhaseRotationScheduler"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/baseAngle",
            label: "baseAngle",
            comment: "The base angle \u{03c0}/6 from which the schedule is derived.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PhaseRotationScheduler"),
            range: XSD_STRING,
        },
        // TargetConvergenceAngle properties
        Property {
            id: "https://uor.foundation/reduction/targetAngle",
            label: "targetAngle",
            comment: "The target convergence angle (default: \u{03c0}).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/TargetConvergenceAngle"),
            range: XSD_STRING,
        },
        // PhaseGateAttestation properties
        Property {
            id: "https://uor.foundation/reduction/gateStage",
            label: "gateStage",
            comment: "The reduction stage at which this gate is applied.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PhaseGateAttestation"),
            range: "https://uor.foundation/reduction/ReductionStep",
        },
        Property {
            id: "https://uor.foundation/reduction/gateExpectedPhase",
            label: "gateExpectedPhase",
            comment: "The expected phase angle \u{03a9}^k at this gate.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PhaseGateAttestation"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/gateResult",
            label: "gateResult",
            comment: "Whether the phase gate check passed or failed.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PhaseGateAttestation"),
            range: XSD_BOOLEAN,
        },
        // ComplexConjugateRollback properties
        Property {
            id: "https://uor.foundation/reduction/rollbackTarget",
            label: "rollbackTarget",
            comment: "The reduction stage to which execution rolls back on gate failure.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ComplexConjugateRollback"),
            range: "https://uor.foundation/reduction/ReductionStep",
        },
        // ReductionStep properties
        Property {
            id: "https://uor.foundation/reduction/stageIndex",
            label: "stageIndex",
            comment: "Zero-based index of this stage in the reduction.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionStep"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/stageName",
            label: "stageName",
            comment: "Human-readable name of this reduction stage.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionStep"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/expectedPhase",
            label: "expectedPhase",
            comment: "The expected phase angle \u{03a9}^k at this stage.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionStep"),
            range: XSD_STRING,
        },
        // Amendment 73: typed guards replacing entryCondition/exitCondition
        Property {
            id: "https://uor.foundation/reduction/entryGuard",
            label: "entryGuard",
            comment: "A typed predicate evaluated on the current \
                      ReductionState. Must be satisfied to enter this stage.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionStep"),
            range: "https://uor.foundation/predicate/StatePredicate",
        },
        Property {
            id: "https://uor.foundation/reduction/exitGuard",
            label: "exitGuard",
            comment: "A typed predicate that must be satisfied before the \
                      reduction advances past this stage.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionStep"),
            range: "https://uor.foundation/predicate/StatePredicate",
        },
        Property {
            id: "https://uor.foundation/reduction/stageEffect",
            label: "stageEffect",
            comment: "The effect applied by this stage upon successful exit.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionStep"),
            range: "https://uor.foundation/effect/Effect",
        },
        // ReductionState properties
        Property {
            id: "https://uor.foundation/reduction/currentStage",
            label: "currentStage",
            comment: "The reduction stage at which execution is currently positioned.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionState"),
            range: "https://uor.foundation/reduction/ReductionStep",
        },
        Property {
            id: "https://uor.foundation/reduction/phaseAngle",
            label: "phaseAngle",
            comment: "The accumulated phase angle at the current point.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionState"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/pinnedMask",
            label: "pinnedMask",
            comment: "Bit mask of sites that are pinned (resolved) at this point.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionState"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/freeRank",
            label: "freeRank",
            comment: "The number of free (unresolved) sites at this point.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionState"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // ReductionRule properties
        Property {
            id: "https://uor.foundation/reduction/transitionGuard",
            label: "transitionGuard",
            comment: "A typed GuardedTransition from predicate/ governing \
                      the stage transition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionRule"),
            range: "https://uor.foundation/predicate/GuardedTransition",
        },
        Property {
            id: "https://uor.foundation/reduction/transitionEffect",
            label: "transitionEffect",
            comment: "The effect applied when this transition fires.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionRule"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/transitionAdvance",
            label: "transitionAdvance",
            comment: "Whether this transition advances to the next stage.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionRule"),
            range: XSD_BOOLEAN,
        },
        // Epoch properties
        Property {
            id: "https://uor.foundation/reduction/epochIndex",
            label: "epochIndex",
            comment: "Zero-based index of this epoch in the reduction execution.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/Epoch"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/epochDatum",
            label: "epochDatum",
            comment: "Metadata or summary datum for this epoch.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/Epoch"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // EpochBoundary properties
        Property {
            id: "https://uor.foundation/reduction/epochBoundaryType",
            label: "epochBoundaryType",
            comment: "The type of epoch boundary crossing (e.g., normal, forced, timeout).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/EpochBoundary"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // Amendment 64: PredicateExpression properties
        Property {
            id: "https://uor.foundation/reduction/predicateField",
            label: "predicateField",
            comment: "The state field this predicate tests.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PredicateExpression"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/predicateOperator",
            label: "predicateOperator",
            comment: "The comparison operator (e.g., '=', '<', '>=').",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PredicateExpression"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/predicateValue",
            label: "predicateValue",
            comment: "The value against which the field is compared.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PredicateExpression"),
            range: XSD_STRING,
        },
        // GuardExpression properties
        Property {
            id: "https://uor.foundation/reduction/guardPredicates",
            label: "guardPredicates",
            comment: "The predicate expressions that compose this guard.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/reduction/GuardExpression"),
            range: "https://uor.foundation/reduction/PredicateExpression",
        },
        // TransitionEffect properties
        Property {
            id: "https://uor.foundation/reduction/effectBindings",
            label: "effectBindings",
            comment: "The property bind steps applied by this effect.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/reduction/TransitionEffect"),
            range: "https://uor.foundation/reduction/PropertyBind",
        },
        // PropertyBind properties
        Property {
            id: "https://uor.foundation/reduction/bindTarget",
            label: "bindTarget",
            comment: "The target site identifier for this binding.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PropertyBind"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/bindValue",
            label: "bindValue",
            comment: "The value to pin the target site to.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PropertyBind"),
            range: XSD_STRING,
        },
        // ReductionAdvance properties
        Property {
            id: "https://uor.foundation/reduction/advanceFrom",
            label: "advanceFrom",
            comment: "The source stage of the advancement.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionAdvance"),
            range: "https://uor.foundation/reduction/ReductionStep",
        },
        Property {
            id: "https://uor.foundation/reduction/advanceTo",
            label: "advanceTo",
            comment: "The target stage of the advancement.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionAdvance"),
            range: "https://uor.foundation/reduction/ReductionStep",
        },
        // ServiceWindow properties
        Property {
            id: "https://uor.foundation/reduction/windowSize",
            label: "windowSize",
            comment: "The number of recent epochs in this service window.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ServiceWindow"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/windowOffset",
            label: "windowOffset",
            comment: "The starting epoch offset of this service window.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ServiceWindow"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // ReductionTransaction properties
        Property {
            id: "https://uor.foundation/reduction/transactionPolicy",
            label: "transactionPolicy",
            comment: "The execution policy for this transaction (e.g., AllOrNothing, BestEffort).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionTransaction"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/transactionOutcome",
            label: "transactionOutcome",
            comment: "The outcome of this transaction (e.g., committed, rolled back).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionTransaction"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // PipelineFailureReason properties
        Property {
            id: "https://uor.foundation/reduction/failureKind",
            label: "failureKind",
            comment: "The kind of pipeline failure (e.g., DispatchMiss, ConvergenceStall).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PipelineFailureReason"),
            range: XSD_STRING,
        },
        // Amendment 76: failureDetail removed (replaced by failure:failureReason)
        // PreflightCheck properties
        Property {
            id: "https://uor.foundation/reduction/preflightKind",
            label: "preflightKind",
            comment: "The kind of preflight check (e.g., feasibility, dispatch coverage).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PreflightCheck"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/preflightResult",
            label: "preflightResult",
            comment: "The result of the preflight check (e.g., pass, fail).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PreflightCheck"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // PipelineSuccess properties
        // Amendment 76: successOutcome removed (replaced by failure:resultDatum)
        Property {
            id: "https://uor.foundation/reduction/groundingReached",
            label: "groundingReached",
            comment: "Whether full grounding was achieved.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PipelineSuccess"),
            range: XSD_BOOLEAN,
        },
        // Amendment 65: Reduction Completion (11 properties)
        // FeasibilityResult properties
        Property {
            id: "https://uor.foundation/reduction/feasibilityKind",
            label: "feasibilityKind",
            comment: "The kind of feasibility result (e.g., Feasible, Infeasible).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/FeasibilityResult"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/feasibilityWitness",
            label: "feasibilityWitness",
            comment: "The witness justifying the feasibility or infeasibility result.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/FeasibilityResult"),
            range: XSD_STRING,
        },
        // LeaseState properties
        Property {
            id: "https://uor.foundation/reduction/leasePhase",
            label: "leasePhase",
            comment: "The lifecycle phase of a lease (e.g., Pending, Active, Released).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/LeaseState"),
            range: XSD_STRING,
        },
        // ManagedLease properties
        Property {
            id: "https://uor.foundation/reduction/managedLeaseId",
            label: "managedLeaseId",
            comment: "Unique identifier for this managed lease.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ManagedLease"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/leaseLifecycle",
            label: "leaseLifecycle",
            comment: "The current lifecycle state of this managed lease.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ManagedLease"),
            range: "https://uor.foundation/reduction/LeaseState",
        },
        // LeaseCheckpoint properties
        Property {
            id: "https://uor.foundation/reduction/checkpointEpoch",
            label: "checkpointEpoch",
            comment: "The epoch index at which this checkpoint was taken.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/LeaseCheckpoint"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/checkpointState",
            label: "checkpointState",
            comment: "The reduction state captured at this checkpoint.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/LeaseCheckpoint"),
            range: "https://uor.foundation/reduction/ReductionState",
        },
        // BackPressureSignal properties
        Property {
            id: "https://uor.foundation/reduction/pressureLevel",
            label: "pressureLevel",
            comment: "The current back-pressure level (e.g., Low, Medium, High).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/BackPressureSignal"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/pressureThreshold",
            label: "pressureThreshold",
            comment: "The threshold at which back-pressure activates.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/BackPressureSignal"),
            range: XSD_DECIMAL,
        },
        // DeferredQuerySet properties
        Property {
            id: "https://uor.foundation/reduction/deferredCount",
            label: "deferredCount",
            comment: "The number of queries in this deferred set.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/DeferredQuerySet"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/deferralEpoch",
            label: "deferralEpoch",
            comment: "The epoch in which these queries were deferred.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/DeferredQuerySet"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 71: SubleaseTransfer properties (4)
        Property {
            id: "https://uor.foundation/reduction/sourceLeaseRef",
            label: "sourceLeaseRef",
            comment: "The lease being transferred from.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/SubleaseTransfer"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/targetLeaseRef",
            label: "targetLeaseRef",
            comment: "The lease being transferred to.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/SubleaseTransfer"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/transferredBudget",
            label: "transferredBudget",
            comment: "The site budget transferred between leases.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/SubleaseTransfer"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/transferCompleted",
            label: "transferCompleted",
            comment: "Whether the sublease transfer has been completed.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/SubleaseTransfer"),
            range: XSD_BOOLEAN,
        },
        // Amendment 71: Predicate subclass properties (15)
        Property {
            id: "https://uor.foundation/reduction/comparisonField",
            label: "comparisonField",
            comment: "The state field tested by this comparison predicate.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ComparisonPredicate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/comparisonOperator",
            label: "comparisonOperator",
            comment: "The comparison operator (e.g., '=', '<', '>=').",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ComparisonPredicate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/comparisonValue",
            label: "comparisonValue",
            comment: "The value against which the comparison is made.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ComparisonPredicate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/conjuncts",
            label: "conjuncts",
            comment: "A conjunct predicate in a conjunction.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/reduction/ConjunctionPredicate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/disjuncts",
            label: "disjuncts",
            comment: "A disjunct predicate in a disjunction.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/reduction/DisjunctionPredicate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/negatedPredicate",
            label: "negatedPredicate",
            comment: "The predicate being negated.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/NegationPredicate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/membershipSet",
            label: "membershipSet",
            comment: "The set against which membership is tested.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/MembershipPredicate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/membershipElement",
            label: "membershipElement",
            comment: "The element being tested for set membership.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/MembershipPredicate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/groundingThreshold",
            label: "groundingThreshold",
            comment: "The grounding threshold above which the predicate holds.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/GroundingPredicate"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/coverageTarget",
            label: "coverageTarget",
            comment: "The site coverage target expression.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/SiteCoveragePredicate"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/equalityLeft",
            label: "equalityLeft",
            comment: "The left-hand side of an equality test.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/EqualsPredicate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/equalityRight",
            label: "equalityRight",
            comment: "The right-hand side of an equality test.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/EqualsPredicate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/nonNullField",
            label: "nonNullField",
            comment: "The field that must be non-null.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/NonNullPredicate"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/queryTypeRef",
            label: "queryTypeRef",
            comment: "The query type reference for subtype testing.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/QuerySubtypePredicate"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // Amendment 71: Missing reduction properties (15)
        Property {
            id: "https://uor.foundation/reduction/siteState",
            label: "siteState",
            comment: "The site state descriptor within a reduction state.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionState"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/transactionScope",
            label: "transactionScope",
            comment: "The scope of sites affected by this transaction.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionTransaction"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/transactionStatus",
            label: "transactionStatus",
            comment: "Current status of this transaction (e.g., pending, committed).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ReductionTransaction"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/baseContextRef",
            label: "baseContextRef",
            comment: "Reference to the base context provided by this service window.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ServiceWindow"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/leaseRemainingBudget",
            label: "leaseRemainingBudget",
            comment: "The remaining site budget at this checkpoint.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/LeaseCheckpoint"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/expiryEpoch",
            label: "expiryEpoch",
            comment: "The epoch at which this managed lease expires.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ManagedLease"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/leaseBudget",
            label: "leaseBudget",
            comment: "The total site budget allocated to this managed lease.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/ManagedLease"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/sourceStage",
            label: "sourceStage",
            comment: "The source stage emitting back-pressure.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/BackPressureSignal"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/targetStage",
            label: "targetStage",
            comment: "The target stage receiving back-pressure.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/BackPressureSignal"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/deferralReason",
            label: "deferralReason",
            comment: "The reason for deferring these queries.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/DeferredQuerySet"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/infeasibilityKind",
            label: "infeasibilityKind",
            comment: "The kind of infeasibility detected.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/FeasibilityResult"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // Amendment 76: infeasibilityDetail removed (replaced by failure:failureReason)
        Property {
            id: "https://uor.foundation/reduction/failureStage",
            label: "failureStage",
            comment: "The reduction stage at which the pipeline failure occurred.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PipelineFailureReason"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/finalGrounding",
            label: "finalGrounding",
            comment: "The final grounding level achieved on pipeline success.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PipelineSuccess"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/preservedGrounding",
            label: "preservedGrounding",
            comment: "Whether grounding was preserved across the epoch boundary.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/EpochBoundary"),
            range: XSD_BOOLEAN,
        },
        // Amendment 94: CompileUnit operational properties
        Property {
            id: "https://uor.foundation/reduction/rootTerm",
            label: "rootTerm",
            comment: "The root term expression of a CompileUnit — the top-level \
                      syntactic node from which all sub-expressions descend.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/CompileUnit"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/reduction/unitWittLevel",
            label: "unitWittLevel",
            comment: "The quantum level at which this CompileUnit operates.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/CompileUnit"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/reduction/thermodynamicBudget",
            label: "thermodynamicBudget",
            comment: "The Landauer-bounded energy budget for this CompileUnit's \
                      resolution, measured in k_B T ln 2 units.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/CompileUnit"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/targetDomains",
            label: "targetDomains",
            comment: "The verification domain(s) targeted by this CompileUnit.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/reduction/CompileUnit"),
            range: "https://uor.foundation/op/VerificationDomain",
        },
        Property {
            id: "https://uor.foundation/reduction/unitAddress",
            label: "unitAddress",
            comment: "Content-addressable identifier computed as the u:Element \
                      of the root term\u{2019}s transitive closure. Computed by \
                      stage_initialization, not declared by the submitter. \
                      Excludes budget, domains, and quantum level to enable \
                      memoization.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/CompileUnit"),
            range: "https://uor.foundation/u/Element",
        },
        Property {
            id: "https://uor.foundation/reduction/preflightOrder",
            label: "preflightOrder",
            comment: "Zero-based execution order for preflight checks. Lower \
                      indices execute first. BudgetSolvencyCheck (order 0) must \
                      precede all others.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/PreflightCheck"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // v0.2.1: FailureField properties (parametric PipelineFailure metadata)
        Property {
            id: "https://uor.foundation/reduction/ofFailure",
            label: "ofFailure",
            comment: "The PipelineFailureReason (or class) this field belongs to.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/FailureField"),
            range: OWL_THING,
        },
        Property {
            id: "https://uor.foundation/reduction/fieldName",
            label: "fieldName",
            comment: "Snake_case name of the Rust field on the generated \
                      PipelineFailure variant.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/FailureField"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/reduction/fieldType",
            label: "fieldType",
            comment: "Rust type for the field, expressed as a literal type \
                      string the codegen emits verbatim (e.g., \
                      \"&'static str\", \"usize\", \"ReductionStep\").",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/FailureField"),
            range: XSD_STRING,
        },
        // v0.2.1 Phase 7a.1: SatBound properties.
        Property {
            id: "https://uor.foundation/reduction/maxVarCount",
            label: "maxVarCount",
            comment: "Maximum variable count the SAT decider supports.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/SatBound"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/maxClauseCount",
            label: "maxClauseCount",
            comment: "Maximum clause count the SAT decider supports.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/SatBound"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/maxLiteralsPerClause",
            label: "maxLiteralsPerClause",
            comment: "Maximum literals per clause (2 for 2-SAT, higher for Horn).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/SatBound"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // v0.2.1 Phase 7a.2: TimingBound properties.
        Property {
            id: "https://uor.foundation/reduction/preflightBudgetNs",
            label: "preflightBudgetNs",
            comment: "Preflight-stage time budget in nanoseconds at the v0.2.1 \
                      reference hardware.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/TimingBound"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/reduction/runtimeBudgetNs",
            label: "runtimeBudgetNs",
            comment: "Runtime-stage time budget in nanoseconds at the v0.2.1 \
                      reference hardware.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/reduction/TimingBound"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
    ]
}

fn individuals() -> Vec<Individual> {
    vec![
        // 7 ReductionStep individuals
        Individual {
            id: "https://uor.foundation/reduction/stage_initialization",
            type_: "https://uor.foundation/reduction/ReductionStep",
            label: "Initialization",
            comment: "Stage 0: initialize state vector to identity.",
            properties: &[
                (
                    "https://uor.foundation/reduction/stageIndex",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/reduction/stageName",
                    IndividualValue::Str("Initialization"),
                ),
                (
                    "https://uor.foundation/reduction/expectedPhase",
                    IndividualValue::Str("\u{03a9}\u{2070}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/stage_declare",
            type_: "https://uor.foundation/reduction/ReductionStep",
            label: "Declare",
            comment: "Stage 1: dispatch resolver (\u{03b4} selects).",
            properties: &[
                (
                    "https://uor.foundation/reduction/stageIndex",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/reduction/stageName",
                    IndividualValue::Str("Declare"),
                ),
                (
                    "https://uor.foundation/reduction/expectedPhase",
                    IndividualValue::Str("\u{03a9}\u{00b9}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/stage_factorize",
            type_: "https://uor.foundation/reduction/ReductionStep",
            label: "Factorize",
            comment: "Stage 2: produce valid ring address (G grounds).",
            properties: &[
                (
                    "https://uor.foundation/reduction/stageIndex",
                    IndividualValue::Int(2),
                ),
                (
                    "https://uor.foundation/reduction/stageName",
                    IndividualValue::Str("Factorize"),
                ),
                (
                    "https://uor.foundation/reduction/expectedPhase",
                    IndividualValue::Str("\u{03a9}\u{00b2}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/stage_resolve",
            type_: "https://uor.foundation/reduction/ReductionStep",
            label: "Resolve",
            comment: "Stage 3: resolve constraints (\u{03a0} terminates).",
            properties: &[
                (
                    "https://uor.foundation/reduction/stageIndex",
                    IndividualValue::Int(3),
                ),
                (
                    "https://uor.foundation/reduction/stageName",
                    IndividualValue::Str("Resolve"),
                ),
                (
                    "https://uor.foundation/reduction/expectedPhase",
                    IndividualValue::Str("\u{03a9}\u{00b3}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/stage_attest",
            type_: "https://uor.foundation/reduction/ReductionStep",
            label: "Attest",
            comment: "Stage 4: accumulate without contradiction (\u{03b1} consistent).",
            properties: &[
                (
                    "https://uor.foundation/reduction/stageIndex",
                    IndividualValue::Int(4),
                ),
                (
                    "https://uor.foundation/reduction/stageName",
                    IndividualValue::Str("Attest"),
                ),
                (
                    "https://uor.foundation/reduction/expectedPhase",
                    IndividualValue::Str("\u{03a9}\u{2074}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/stage_extract",
            type_: "https://uor.foundation/reduction/ReductionStep",
            label: "Extract",
            comment: "Stage 5: extract coherent output (P projects).",
            properties: &[
                (
                    "https://uor.foundation/reduction/stageIndex",
                    IndividualValue::Int(5),
                ),
                (
                    "https://uor.foundation/reduction/stageName",
                    IndividualValue::Str("Extract"),
                ),
                (
                    "https://uor.foundation/reduction/expectedPhase",
                    IndividualValue::Str("\u{03a9}\u{2075}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/stage_convergence",
            type_: "https://uor.foundation/reduction/ReductionStep",
            label: "Convergence",
            comment: "Terminal stage: reduction has reached the convergence angle \u{03c0}.",
            properties: &[
                (
                    "https://uor.foundation/reduction/stageIndex",
                    IndividualValue::Int(6),
                ),
                (
                    "https://uor.foundation/reduction/stageName",
                    IndividualValue::Str("Convergence"),
                ),
                (
                    "https://uor.foundation/reduction/expectedPhase",
                    IndividualValue::Str("\u{03c0}"),
                ),
            ],
        },
        // 6 PhaseGateAttestation individuals
        Individual {
            id: "https://uor.foundation/reduction/gate_initialization",
            type_: "https://uor.foundation/reduction/PhaseGateAttestation",
            label: "gate_initialization",
            comment: "Phase gate at stage 0 boundary: checks \u{03a9}\u{2070} = 1.",
            properties: &[
                (
                    "https://uor.foundation/reduction/gateStage",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/stage_initialization",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/gateExpectedPhase",
                    IndividualValue::Str("\u{03a9}\u{2070}"),
                ),
                (
                    "https://uor.foundation/reduction/gateResult",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/gate_declare",
            type_: "https://uor.foundation/reduction/PhaseGateAttestation",
            label: "gate_declare",
            comment: "Phase gate at stage 1 boundary: checks \u{03a9}\u{00b9}.",
            properties: &[
                (
                    "https://uor.foundation/reduction/gateStage",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/stage_declare",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/gateExpectedPhase",
                    IndividualValue::Str("\u{03a9}\u{00b9}"),
                ),
                (
                    "https://uor.foundation/reduction/gateResult",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/gate_factorize",
            type_: "https://uor.foundation/reduction/PhaseGateAttestation",
            label: "gate_factorize",
            comment: "Phase gate at stage 2 boundary: checks \u{03a9}\u{00b2}.",
            properties: &[
                (
                    "https://uor.foundation/reduction/gateStage",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/stage_factorize",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/gateExpectedPhase",
                    IndividualValue::Str("\u{03a9}\u{00b2}"),
                ),
                (
                    "https://uor.foundation/reduction/gateResult",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/gate_resolve",
            type_: "https://uor.foundation/reduction/PhaseGateAttestation",
            label: "gate_resolve",
            comment: "Phase gate at stage 3 boundary: checks \u{03a9}\u{00b3}.",
            properties: &[
                (
                    "https://uor.foundation/reduction/gateStage",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/stage_resolve",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/gateExpectedPhase",
                    IndividualValue::Str("\u{03a9}\u{00b3}"),
                ),
                (
                    "https://uor.foundation/reduction/gateResult",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/gate_attest",
            type_: "https://uor.foundation/reduction/PhaseGateAttestation",
            label: "gate_attest",
            comment: "Phase gate at stage 4 boundary: checks \u{03a9}\u{2074}.",
            properties: &[
                (
                    "https://uor.foundation/reduction/gateStage",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/stage_attest",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/gateExpectedPhase",
                    IndividualValue::Str("\u{03a9}\u{2074}"),
                ),
                (
                    "https://uor.foundation/reduction/gateResult",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/gate_extract",
            type_: "https://uor.foundation/reduction/PhaseGateAttestation",
            label: "gate_extract",
            comment: "Phase gate at stage 5 boundary: checks \u{03a9}\u{2075}.",
            properties: &[
                (
                    "https://uor.foundation/reduction/gateStage",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/stage_extract",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/gateExpectedPhase",
                    IndividualValue::Str("\u{03a9}\u{2075}"),
                ),
                (
                    "https://uor.foundation/reduction/gateResult",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        // Reduction-level individuals
        Individual {
            id: "https://uor.foundation/reduction/euler_reduction_instance",
            type_: "https://uor.foundation/reduction/EulerReduction",
            label: "euler_reduction_instance",
            comment: "The canonical Euler reduction instance with \u{03a9} = e^{i\u{03c0}/6} \
                      and 6 stages.",
            properties: &[
                (
                    "https://uor.foundation/reduction/phaseParameter",
                    IndividualValue::Str("e^{i\u{03c0}/6}"),
                ),
                (
                    "https://uor.foundation/reduction/stageCount",
                    IndividualValue::Int(6),
                ),
                (
                    "https://uor.foundation/reduction/convergenceAngle",
                    IndividualValue::Str("\u{03c0}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/phase_schedule",
            type_: "https://uor.foundation/reduction/PhaseRotationScheduler",
            label: "phase_schedule",
            comment: "The canonical phase rotation schedule for the 6-stage reduction.",
            properties: &[
                (
                    "https://uor.foundation/reduction/rotationSchedule",
                    IndividualValue::Str("\u{03a9}\u{2070}, \u{03a9}\u{00b9}, \u{03a9}\u{00b2}, \u{03a9}\u{00b3}, \u{03a9}\u{2074}, \u{03a9}\u{2075}"),
                ),
                (
                    "https://uor.foundation/reduction/baseAngle",
                    IndividualValue::Str("\u{03c0}/6"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/convergence_target",
            type_: "https://uor.foundation/reduction/TargetConvergenceAngle",
            label: "convergence_target",
            comment: "The default convergence target angle \u{03c0}.",
            properties: &[
                (
                    "https://uor.foundation/reduction/targetAngle",
                    IndividualValue::Str("\u{03c0}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/conjugate_rollback",
            type_: "https://uor.foundation/reduction/ComplexConjugateRollback",
            label: "conjugate_rollback",
            comment: "The canonical complex conjugate rollback operation: z \u{2192} z\u{0304}.",
            properties: &[
                (
                    "https://uor.foundation/reduction/rollbackTarget",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/stage_initialization",
                    ),
                ),
            ],
        },
        // Amendment 64: PipelineFailureReason individuals (5)
        Individual {
            id: "https://uor.foundation/reduction/DispatchMiss",
            type_: "https://uor.foundation/reduction/PipelineFailureReason",
            label: "DispatchMiss",
            comment: "Failure: no resolver matched the dispatch query.",
            properties: &[
                (
                    "https://uor.foundation/reduction/failureKind",
                    IndividualValue::Str("DispatchMiss"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/GroundingFailure",
            type_: "https://uor.foundation/reduction/PipelineFailureReason",
            label: "GroundingFailure",
            comment: "Failure: grounding to a valid ring address failed.",
            properties: &[
                (
                    "https://uor.foundation/reduction/failureKind",
                    IndividualValue::Str("GroundingFailure"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/ConvergenceStall",
            type_: "https://uor.foundation/reduction/PipelineFailureReason",
            label: "ConvergenceStall",
            comment: "Failure: reduction stalled before reaching convergence angle.",
            properties: &[
                (
                    "https://uor.foundation/reduction/failureKind",
                    IndividualValue::Str("ConvergenceStall"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/ContradictionDetected",
            type_: "https://uor.foundation/reduction/PipelineFailureReason",
            label: "ContradictionDetected",
            comment: "Failure: accumulation detected a logical contradiction.",
            properties: &[
                (
                    "https://uor.foundation/reduction/failureKind",
                    IndividualValue::Str("ContradictionDetected"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/CoherenceViolation",
            type_: "https://uor.foundation/reduction/PipelineFailureReason",
            label: "CoherenceViolation",
            comment: "Failure: coherence constraint violated during extraction.",
            properties: &[
                (
                    "https://uor.foundation/reduction/failureKind",
                    IndividualValue::Str("CoherenceViolation"),
                ),
            ],
        },
        // v0.2.2 W14: ShapeMismatch — signals that pipeline::run::<T> produced
        // a value of a shape other than the caller-declared T. Variant fields
        // (expected, got) are added below via FailureField individuals.
        Individual {
            id: "https://uor.foundation/reduction/ShapeMismatch",
            type_: "https://uor.foundation/reduction/PipelineFailureReason",
            label: "ShapeMismatch",
            comment: "Failure: the CompileUnit's root term produced a value of \
                      a shape other than the caller-declared expected shape. \
                      Introduced in v0.2.2 W14 as part of the typed \
                      pipeline::run::<T> entry point.",
            properties: &[
                (
                    "https://uor.foundation/reduction/failureKind",
                    IndividualValue::Str("ShapeMismatch"),
                ),
            ],
        },
        // PipelineSuccess individual (1)
        Individual {
            id: "https://uor.foundation/reduction/FullGroundingSuccess",
            type_: "https://uor.foundation/reduction/PipelineSuccess",
            label: "FullGroundingSuccess",
            comment: "Successful termination: all sites saturated.",
            properties: &[
                (
                    "https://uor.foundation/reduction/groundingReached",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        // PreflightCheck individuals (3)
        Individual {
            id: "https://uor.foundation/reduction/FeasibilityCheck",
            type_: "https://uor.foundation/reduction/PreflightCheck",
            label: "FeasibilityCheck",
            comment: "Preflight: checks that the reduction can reach convergence.",
            properties: &[
                (
                    "https://uor.foundation/reduction/preflightKind",
                    IndividualValue::Str("Feasibility"),
                ),
                (
                    "https://uor.foundation/reduction/preflightOrder",
                    IndividualValue::Int(1),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/DispatchCoverageCheck",
            type_: "https://uor.foundation/reduction/PreflightCheck",
            label: "DispatchCoverageCheck",
            comment: "Preflight: checks that every dispatch query has a resolver.",
            properties: &[
                (
                    "https://uor.foundation/reduction/preflightKind",
                    IndividualValue::Str("DispatchCoverage"),
                ),
                (
                    "https://uor.foundation/reduction/preflightOrder",
                    IndividualValue::Int(2),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/PackageCoherenceCheck",
            type_: "https://uor.foundation/reduction/PreflightCheck",
            label: "PackageCoherenceCheck",
            comment: "Preflight: checks package-level coherence constraints.",
            properties: &[
                (
                    "https://uor.foundation/reduction/preflightKind",
                    IndividualValue::Str("PackageCoherence"),
                ),
                (
                    "https://uor.foundation/reduction/preflightOrder",
                    IndividualValue::Int(3),
                ),
            ],
        },
        // ServiceWindow individual (1)
        Individual {
            id: "https://uor.foundation/reduction/default_service_window",
            type_: "https://uor.foundation/reduction/ServiceWindow",
            label: "default_service_window",
            comment: "The default service window: 3 epochs, zero offset.",
            properties: &[
                (
                    "https://uor.foundation/reduction/windowSize",
                    IndividualValue::Int(3),
                ),
                (
                    "https://uor.foundation/reduction/windowOffset",
                    IndividualValue::Int(0),
                ),
            ],
        },
        // ReductionAdvance individual (1)
        Individual {
            id: "https://uor.foundation/reduction/advance_init_to_declare",
            type_: "https://uor.foundation/reduction/ReductionAdvance",
            label: "advance_init_to_declare",
            comment: "Advancement from Initialization to Declare.",
            properties: &[
                (
                    "https://uor.foundation/reduction/advanceFrom",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/stage_initialization",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/advanceTo",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/stage_declare",
                    ),
                ),
            ],
        },
        // ReductionTransaction individual (1)
        Individual {
            id: "https://uor.foundation/reduction/atomic_transaction",
            type_: "https://uor.foundation/reduction/ReductionTransaction",
            label: "atomic_transaction",
            comment: "An all-or-nothing atomic reduction transaction.",
            properties: &[
                (
                    "https://uor.foundation/reduction/transactionPolicy",
                    IndividualValue::Str("AllOrNothing"),
                ),
            ],
        },
        // GuardExpression individual (1)
        Individual {
            id: "https://uor.foundation/reduction/empty_guard",
            type_: "https://uor.foundation/reduction/GuardExpression",
            label: "empty_guard",
            comment: "A trivially satisfied guard with no predicates.",
            properties: &[],
        },
        // TransitionEffect individual (1)
        Individual {
            id: "https://uor.foundation/reduction/identity_effect",
            type_: "https://uor.foundation/reduction/TransitionEffect",
            label: "identity_effect",
            comment: "The identity effect: no state changes.",
            properties: &[],
        },
        // PredicateExpression individual (1)
        Individual {
            id: "https://uor.foundation/reduction/true_predicate",
            type_: "https://uor.foundation/reduction/PredicateExpression",
            label: "true_predicate",
            comment: "A predicate that always evaluates to true.",
            properties: &[
                (
                    "https://uor.foundation/reduction/predicateField",
                    IndividualValue::Str("*"),
                ),
                (
                    "https://uor.foundation/reduction/predicateOperator",
                    IndividualValue::Str("true"),
                ),
                (
                    "https://uor.foundation/reduction/predicateValue",
                    IndividualValue::Str("*"),
                ),
            ],
        },
        // PropertyBind individual (1)
        Individual {
            id: "https://uor.foundation/reduction/noop_bind",
            type_: "https://uor.foundation/reduction/PropertyBind",
            label: "noop_bind",
            comment: "A no-op property binding that changes nothing.",
            properties: &[
                (
                    "https://uor.foundation/reduction/bindTarget",
                    IndividualValue::Str("none"),
                ),
                (
                    "https://uor.foundation/reduction/bindValue",
                    IndividualValue::Str("unchanged"),
                ),
            ],
        },
        // Amendment 65: LeaseState individuals (5)
        Individual {
            id: "https://uor.foundation/reduction/Pending",
            type_: "https://uor.foundation/reduction/LeaseState",
            label: "Pending",
            comment: "Lease is pending activation.",
            properties: &[
                (
                    "https://uor.foundation/reduction/leasePhase",
                    IndividualValue::Str("Pending"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/Active",
            type_: "https://uor.foundation/reduction/LeaseState",
            label: "Active",
            comment: "Lease is active and resources are allocated.",
            properties: &[
                (
                    "https://uor.foundation/reduction/leasePhase",
                    IndividualValue::Str("Active"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/Released",
            type_: "https://uor.foundation/reduction/LeaseState",
            label: "Released",
            comment: "Lease has been explicitly released.",
            properties: &[
                (
                    "https://uor.foundation/reduction/leasePhase",
                    IndividualValue::Str("Released"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/Expired",
            type_: "https://uor.foundation/reduction/LeaseState",
            label: "Expired",
            comment: "Lease has expired due to timeout.",
            properties: &[
                (
                    "https://uor.foundation/reduction/leasePhase",
                    IndividualValue::Str("Expired"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/Suspended",
            type_: "https://uor.foundation/reduction/LeaseState",
            label: "Suspended",
            comment: "Lease is temporarily suspended.",
            properties: &[
                (
                    "https://uor.foundation/reduction/leasePhase",
                    IndividualValue::Str("Suspended"),
                ),
            ],
        },
        // FeasibilityResult individuals (2)
        Individual {
            id: "https://uor.foundation/reduction/FeasibilityWitness",
            type_: "https://uor.foundation/reduction/FeasibilityResult",
            label: "FeasibilityWitness",
            comment: "Preflight result: reduction is feasible.",
            properties: &[
                (
                    "https://uor.foundation/reduction/feasibilityKind",
                    IndividualValue::Str("Feasible"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/InfeasibilityWitness",
            type_: "https://uor.foundation/reduction/FeasibilityResult",
            label: "InfeasibilityWitness",
            comment: "Preflight result: reduction is infeasible.",
            properties: &[
                (
                    "https://uor.foundation/reduction/feasibilityKind",
                    IndividualValue::Str("Infeasible"),
                ),
                (
                    "https://uor.foundation/reduction/feasibilityWitness",
                    IndividualValue::Str("obstruction detected"),
                ),
            ],
        },
        // Timing / misc individuals (3)
        Individual {
            id: "https://uor.foundation/reduction/PreflightTiming",
            type_: "https://uor.foundation/reduction/PreflightCheck",
            label: "PreflightTiming",
            comment: "Preflight: timing feasibility check.",
            properties: &[
                (
                    "https://uor.foundation/reduction/preflightKind",
                    IndividualValue::Str("Timing"),
                ),
                (
                    "https://uor.foundation/reduction/preflightOrder",
                    IndividualValue::Int(4),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/RuntimeTiming",
            type_: "https://uor.foundation/reduction/PreflightCheck",
            label: "RuntimeTiming",
            comment: "Preflight: runtime timing bounds check.",
            properties: &[
                (
                    "https://uor.foundation/reduction/preflightKind",
                    IndividualValue::Str("RuntimeTiming"),
                ),
                (
                    "https://uor.foundation/reduction/preflightOrder",
                    IndividualValue::Int(5),
                ),
            ],
        },
        // Amendment 84: BudgetSolvencyCheck
        Individual {
            id: "https://uor.foundation/reduction/BudgetSolvencyCheck",
            type_: "https://uor.foundation/reduction/PreflightCheck",
            label: "BudgetSolvencyCheck",
            comment: "Preflight: verifies thermodynamicBudget \u{2265} \
                      bitsWidth(unitWittLevel) \u{00d7} ln 2. Rejects \
                      the CompileUnit if the budget is absent or insufficient. \
                      Must execute before all other preflights (preflightOrder 0). \
                      Cost is O(1) per CS_4.",
            properties: &[
                (
                    "https://uor.foundation/reduction/preflightKind",
                    IndividualValue::Str("BudgetSolvency"),
                ),
                (
                    "https://uor.foundation/reduction/preflightOrder",
                    IndividualValue::Int(0),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/BackPressureDefault",
            type_: "https://uor.foundation/reduction/BackPressureSignal",
            label: "BackPressureDefault",
            comment: "Default back-pressure signal with medium threshold.",
            properties: &[
                (
                    "https://uor.foundation/reduction/pressureLevel",
                    IndividualValue::Str("Medium"),
                ),
                (
                    "https://uor.foundation/reduction/pressureThreshold",
                    IndividualValue::Float(0.75),
                ),
            ],
        },
        // ── v0.2.1: FailureField individuals (parametric Rust PipelineFailure
        // variant fields). One per (variant, field) pair. The codegen reads
        // these to emit the foundation::PipelineFailure enum.
        //
        // DispatchMiss → query_iri, table_iri
        Individual {
            id: "https://uor.foundation/reduction/dispatchMiss_queryIri_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "dispatchMiss_queryIri_field",
            comment: "DispatchMiss field carrying the query IRI that failed \
                      to dispatch.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/DispatchMiss",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("query_iri"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("&'static str"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/dispatchMiss_tableIri_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "dispatchMiss_tableIri_field",
            comment: "DispatchMiss field carrying the dispatch table IRI \
                      that failed to find a matching rule.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/DispatchMiss",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("table_iri"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("&'static str"),
                ),
            ],
        },
        // GroundingFailure → reason_iri
        Individual {
            id: "https://uor.foundation/reduction/groundingFailure_reasonIri_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "groundingFailure_reasonIri_field",
            comment: "GroundingFailure field carrying the morphism:GroundingMap \
                      IRI whose image missed the ring.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/GroundingFailure",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("reason_iri"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("&'static str"),
                ),
            ],
        },
        // ConvergenceStall → stage, angle
        Individual {
            id: "https://uor.foundation/reduction/convergenceStall_stage_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "convergenceStall_stage_field",
            comment: "ConvergenceStall field carrying the IRI of the \
                      ReductionStep at which the phase-rotation scheduler \
                      stopped advancing.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/ConvergenceStall",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("stage_iri"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("&'static str"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/convergenceStall_angle_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "convergenceStall_angle_field",
            comment: "ConvergenceStall field carrying the last \
                      reduction:convergenceAngle reached, encoded as a \
                      milli-radian integer to preserve `Eq` on PipelineFailure.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/ConvergenceStall",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("angle_milliradians"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("i64"),
                ),
            ],
        },
        // ContradictionDetected → at_step, trace_iri
        Individual {
            id: "https://uor.foundation/reduction/contradictionDetected_atStep_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "contradictionDetected_atStep_field",
            comment: "ContradictionDetected field carrying the \
                      derivation:stepIndex of the accumulation step.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/ContradictionDetected",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("at_step"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("usize"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/contradictionDetected_traceIri_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "contradictionDetected_traceIri_field",
            comment: "ContradictionDetected field pointing to the \
                      trace:ComputationTrace record.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/ContradictionDetected",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("trace_iri"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("&'static str"),
                ),
            ],
        },
        // CoherenceViolation → site_position, constraint_iri
        Individual {
            id: "https://uor.foundation/reduction/coherenceViolation_sitePosition_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "coherenceViolation_sitePosition_field",
            comment: "CoherenceViolation field carrying the site position \
                      where the conformance:PackageCoherenceCheck failed.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/CoherenceViolation",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("site_position"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("usize"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/coherenceViolation_constraintIri_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "coherenceViolation_constraintIri_field",
            comment: "CoherenceViolation field carrying the failing \
                      constraint IRI.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/CoherenceViolation",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("constraint_iri"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("&'static str"),
                ),
            ],
        },
        // v0.2.2 W14: ShapeMismatch field carrying the expected shape IRI.
        Individual {
            id: "https://uor.foundation/reduction/shapeMismatch_expected_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "shapeMismatch_expected_field",
            comment: "ShapeMismatch field carrying the expected shape IRI \
                      declared by the caller of pipeline::run::<T>.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/ShapeMismatch",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("expected"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("&'static str"),
                ),
            ],
        },
        // v0.2.2 W14: ShapeMismatch field carrying the actual shape IRI.
        Individual {
            id: "https://uor.foundation/reduction/shapeMismatch_got_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "shapeMismatch_got_field",
            comment: "ShapeMismatch field carrying the actual shape IRI \
                      produced by the CompileUnit's root term.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/ShapeMismatch",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("got"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("&'static str"),
                ),
            ],
        },
        // failure:LiftObstructionFailure → site_position, obstruction_class_iri
        // (Cross-namespace ofFailure pointing into the failure: namespace)
        Individual {
            id: "https://uor.foundation/reduction/liftObstruction_sitePosition_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "liftObstruction_sitePosition_field",
            comment: "LiftObstructionFailure field carrying the site position \
                      from the resolver:LiftRefinementSuggestion.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/failure/LiftObstructionFailure",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("site_position"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("usize"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/liftObstruction_obstructionClassIri_field",
            type_: "https://uor.foundation/reduction/FailureField",
            label: "liftObstruction_obstructionClassIri_field",
            comment: "LiftObstructionFailure field carrying the obstruction \
                      class IRI from the LiftRefinementSuggestion.",
            properties: &[
                (
                    "https://uor.foundation/reduction/ofFailure",
                    IndividualValue::IriRef(
                        "https://uor.foundation/failure/LiftObstructionFailure",
                    ),
                ),
                (
                    "https://uor.foundation/reduction/fieldName",
                    IndividualValue::Str("obstruction_class_iri"),
                ),
                (
                    "https://uor.foundation/reduction/fieldType",
                    IndividualValue::Str("&'static str"),
                ),
            ],
        },
        // v0.2.1 Phase 7a.1: SatBound individuals.
        Individual {
            id: "https://uor.foundation/reduction/TwoSatBound",
            type_: "https://uor.foundation/reduction/SatBound",
            label: "TwoSatBound",
            comment: "v0.2.1 2-SAT decider bounds. Drives TWO_SAT_MAX_VARS and \
                      the corresponding Lean fuel constant.",
            properties: &[
                (
                    "https://uor.foundation/reduction/maxVarCount",
                    IndividualValue::Int(256),
                ),
                (
                    "https://uor.foundation/reduction/maxClauseCount",
                    IndividualValue::Int(512),
                ),
                (
                    "https://uor.foundation/reduction/maxLiteralsPerClause",
                    IndividualValue::Int(2),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/reduction/HornSatBound",
            type_: "https://uor.foundation/reduction/SatBound",
            label: "HornSatBound",
            comment: "v0.2.1 Horn-SAT decider bounds.",
            properties: &[
                (
                    "https://uor.foundation/reduction/maxVarCount",
                    IndividualValue::Int(256),
                ),
                (
                    "https://uor.foundation/reduction/maxClauseCount",
                    IndividualValue::Int(512),
                ),
                (
                    "https://uor.foundation/reduction/maxLiteralsPerClause",
                    IndividualValue::Int(8),
                ),
            ],
        },
        // v0.2.1 Phase 7a.2: TimingBound individuals.
        Individual {
            id: "https://uor.foundation/reduction/PreflightTimingBound",
            type_: "https://uor.foundation/reduction/TimingBound",
            label: "PreflightTimingBound",
            comment: "v0.2.1 preflight-stage time budget.",
            properties: &[(
                "https://uor.foundation/reduction/preflightBudgetNs",
                IndividualValue::Int(10_000_000),
            )],
        },
        Individual {
            id: "https://uor.foundation/reduction/RuntimeTimingBound",
            type_: "https://uor.foundation/reduction/TimingBound",
            label: "RuntimeTimingBound",
            comment: "v0.2.1 runtime-stage time budget.",
            properties: &[(
                "https://uor.foundation/reduction/runtimeBudgetNs",
                IndividualValue::Int(10_000_000),
            )],
        },
    ]
}
