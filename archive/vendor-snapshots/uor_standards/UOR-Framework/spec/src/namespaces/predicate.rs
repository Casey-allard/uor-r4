//! `predicate/` namespace — Predicates and dispatch.
//!
//! The `predicate/` namespace formalizes boolean-valued functions on kernel
//! objects: resolver dispatch, reduction guard evaluation, and conditional
//! resolution paths. Every predicate is total (evaluation terminates for
//! all inputs) and pure (no side effects).
//!
//! - **Amendment 95**: 9 classes, 15 properties, 12 individuals (predicate registry)
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `predicate/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "predicate",
            iri: NS_PREDICATE,
            label: "UOR Predicates and Dispatch",
            comment: "Boolean-valued functions on kernel objects. Formalizes \
                      resolver dispatch, reduction guard evaluation, and \
                      conditional resolution paths.",
            space: Space::Kernel,
            imports: &[NS_OP, NS_SCHEMA, NS_TYPE, NS_STATE, NS_EFFECT, NS_PARTITION],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/predicate/Predicate",
            label: "Predicate",
            comment: "A total, pure, boolean-valued function on a kernel \
                      object. Evaluation terminates for all inputs and \
                      produces no side effects.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/predicate/TypePredicate",
            label: "TypePredicate",
            comment: "A predicate over type:TypeDefinition. Used for \
                      resolver dispatch.",
            subclass_of: &["https://uor.foundation/predicate/Predicate"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/predicate/StatePredicate",
            label: "StatePredicate",
            comment: "A predicate over state:Context or \
                      reduction:ReductionState. Used for reduction step guards.",
            subclass_of: &["https://uor.foundation/predicate/Predicate"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/predicate/SitePredicate",
            label: "SitePredicate",
            comment: "A predicate over partition:SiteIndex. Used for \
                      site-level selection in geodesic resolution.",
            subclass_of: &["https://uor.foundation/predicate/Predicate"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/predicate/DispatchRule",
            label: "DispatchRule",
            comment: "A pair (Predicate, Target) where Target is a \
                      resolver:Resolver class. The kernel evaluates the \
                      predicate; if true, the target resolver is selected.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/predicate/DispatchTable",
            label: "DispatchTable",
            comment: "An ordered set of DispatchRules for a single dispatch \
                      point. Must satisfy exhaustiveness and mutual exclusion.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/predicate/GuardedTransition",
            label: "GuardedTransition",
            comment: "A triple (StatePredicate, effect:Effect, \
                      reduction:ReductionStep). The guard is a StatePredicate; \
                      if true, the effect is applied and the reduction advances \
                      to the target step.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/predicate/MatchArm",
            label: "MatchArm",
            comment: "A single case in a pattern match: a Predicate and a \
                      result Term. The match evaluates predicates in order \
                      and returns the result of the first matching arm.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/predicate/MatchExpression",
            label: "MatchExpression",
            comment: "A term formed by evaluating a sequence of MatchArms. \
                      Extends the term language with deterministic conditional \
                      evaluation.",
            subclass_of: &["https://uor.foundation/schema/Term"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Object properties
        Property {
            id: "https://uor.foundation/predicate/evaluatesOver",
            label: "evaluatesOver",
            comment: "The OWL class of objects this predicate accepts as input.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/Predicate"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/predicate/dispatchPredicate",
            label: "dispatchPredicate",
            comment: "The predicate that triggers this dispatch rule.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/DispatchRule"),
            range: "https://uor.foundation/predicate/Predicate",
        },
        Property {
            id: "https://uor.foundation/predicate/dispatchTarget",
            label: "dispatchTarget",
            comment: "The resolver class selected when the predicate is \
                      satisfied. Range is the OWL class IRI of a \
                      resolver:Resolver subclass; v0.2.1 uses class IRIs \
                      so the codegen can construct façade structs.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/DispatchRule"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/predicate/dispatchRules",
            label: "dispatchRules",
            comment: "The ordered set of rules in this table.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/predicate/DispatchTable"),
            range: "https://uor.foundation/predicate/DispatchRule",
        },
        // v0.2.1: Dispatch-rule priority for deterministic evaluation order
        Property {
            id: "https://uor.foundation/predicate/dispatchPriority",
            label: "dispatchPriority",
            comment: "Non-negative integer priority. Lower priority values \
                      are evaluated first; ties within a DispatchTable are \
                      resolved by declaration order.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/DispatchRule"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // v0.2.1: Conformance PredicateShape backing properties for
        // user-declared predicate:Predicate individuals.
        Property {
            id: "https://uor.foundation/predicate/evaluatorTerm",
            label: "evaluatorTerm",
            comment: "The evaluator term that must reduce to a boolean-shaped \
                      datum on every input of the declared input type.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/Predicate"),
            range: "https://uor.foundation/schema/Term",
        },
        Property {
            id: "https://uor.foundation/predicate/terminationWitness",
            label: "terminationWitness",
            comment: "An IRI or identifier of the proof:Proof / \
                      proof:ComputationCertificate attesting that the \
                      evaluator halts on all inputs.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/Predicate"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/predicate/guardPredicate",
            label: "guardPredicate",
            comment: "The guard predicate for this transition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/GuardedTransition"),
            range: "https://uor.foundation/predicate/StatePredicate",
        },
        Property {
            id: "https://uor.foundation/predicate/guardEffect",
            label: "guardEffect",
            comment: "The effect applied when the guard is satisfied.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/GuardedTransition"),
            range: "https://uor.foundation/effect/Effect",
        },
        Property {
            id: "https://uor.foundation/predicate/guardTarget",
            label: "guardTarget",
            comment: "The reduction step to advance to.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/GuardedTransition"),
            // Full IRI string: predicate/ cannot import reduction/
            // because reduction/ will import predicate/ in Phase 3
            range: "https://uor.foundation/reduction/ReductionStep",
        },
        Property {
            id: "https://uor.foundation/predicate/matchArms",
            label: "matchArms",
            comment: "The ordered arms of this match expression.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/predicate/MatchExpression"),
            range: "https://uor.foundation/predicate/MatchArm",
        },
        Property {
            id: "https://uor.foundation/predicate/armPredicate",
            label: "armPredicate",
            comment: "The predicate guarding this arm.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/MatchArm"),
            range: "https://uor.foundation/predicate/Predicate",
        },
        Property {
            id: "https://uor.foundation/predicate/armResult",
            label: "armResult",
            comment: "The result term if this arm matches.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/MatchArm"),
            range: "https://uor.foundation/schema/Term",
        },
        Property {
            id: "https://uor.foundation/predicate/boundedEvaluator",
            label: "boundedEvaluator",
            comment: "A termination witness for user-declared predicates. \
                      Kernel predicates are total by construction; \
                      user-declared predicates must carry a descent measure \
                      certifying termination.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/predicate/Predicate"),
            range: "https://uor.foundation/recursion/DescentMeasure",
        },
        // Datatype properties
        Property {
            id: "https://uor.foundation/predicate/dispatchIndex",
            label: "dispatchIndex",
            comment: "Position in the dispatch table (evaluation order).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/DispatchRule"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/predicate/isExhaustive",
            label: "isExhaustive",
            comment: "True iff the disjunction of all dispatch predicates is \
                      a tautology over the input class.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/DispatchTable"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/predicate/isMutuallyExclusive",
            label: "isMutuallyExclusive",
            comment: "True iff no two dispatch predicates can be \
                      simultaneously true for any input.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/DispatchTable"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/predicate/armIndex",
            label: "armIndex",
            comment: "Position in the match expression (evaluation order).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/predicate/MatchArm"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
    ]
}

const EVALUATES_OVER: &str = "https://uor.foundation/predicate/evaluatesOver";

/// Amendment 95: Predicate registry individuals (Workstream 1).
fn individuals() -> Vec<Individual> {
    vec![
        Individual {
            id: "https://uor.foundation/predicate/always",
            type_: "https://uor.foundation/predicate/Predicate",
            label: "always",
            comment: "True on every Datum. Match-arm default catch-all.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/schema/Datum"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/never",
            type_: "https://uor.foundation/predicate/Predicate",
            label: "never",
            comment: "False on every Datum. Disabled-arm marker.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/schema/Datum"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/isZero",
            type_: "https://uor.foundation/predicate/TypePredicate",
            label: "isZero",
            comment: "True iff the Datum is the additive identity of its ring.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/schema/Datum"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/isUnit",
            type_: "https://uor.foundation/predicate/TypePredicate",
            label: "isUnit",
            comment: "True iff the Datum is the multiplicative identity.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/schema/Datum"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/isOdd",
            type_: "https://uor.foundation/predicate/TypePredicate",
            label: "isOdd",
            comment: "True iff the Datum parity bit is 1.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/schema/Datum"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/isEven",
            type_: "https://uor.foundation/predicate/TypePredicate",
            label: "isEven",
            comment: "True iff the Datum parity bit is 0.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/schema/Datum"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/isInvolution",
            type_: "https://uor.foundation/predicate/TypePredicate",
            label: "isInvolution",
            comment: "True iff op(op(x)) = x for the bound op.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/schema/Datum"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/sitePinned",
            type_: "https://uor.foundation/predicate/SitePredicate",
            label: "sitePinned",
            comment: "True iff the named site coordinate is currently pinned.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/partition/SiteIndex"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/siteFree",
            type_: "https://uor.foundation/predicate/SitePredicate",
            label: "siteFree",
            comment: "True iff the named site coordinate is currently free.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/partition/SiteIndex"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/contradictionReached",
            type_: "https://uor.foundation/predicate/StatePredicate",
            label: "contradictionReached",
            comment: "True iff the resolver has entered a ContradictionBoundary.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/state/ContradictionBoundary"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/budgetExhausted",
            type_: "https://uor.foundation/predicate/StatePredicate",
            label: "budgetExhausted",
            comment: "True iff the FreeRank deficit is zero.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/partition/FreeRank"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/reductionConverged",
            type_: "https://uor.foundation/predicate/StatePredicate",
            label: "reductionConverged",
            comment: "True iff the reduction fixpoint has been reached.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/reduction/ReductionState"),
            )],
        },
        // v0.2.1: Inhabitance fragment classifier predicates
        Individual {
            id: "https://uor.foundation/predicate/Is2SatShape",
            type_: "https://uor.foundation/predicate/TypePredicate",
            label: "Is2SatShape",
            comment: "True on ConstrainedType instances whose constraint \
                      nerve contains only disjunctions of width \u{2264} 2.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/type/ConstrainedType"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/IsHornShape",
            type_: "https://uor.foundation/predicate/TypePredicate",
            label: "IsHornShape",
            comment: "True on ConstrainedType instances whose disjunctions \
                      each contain at most one positive literal.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/type/ConstrainedType"),
            )],
        },
        Individual {
            id: "https://uor.foundation/predicate/IsResidualFragment",
            type_: "https://uor.foundation/predicate/TypePredicate",
            label: "IsResidualFragment",
            comment: "Default (catch-all) predicate. True on ConstrainedType \
                      instances not classified by Is2SatShape or IsHornShape.",
            properties: &[(
                EVALUATES_OVER,
                IndividualValue::IriRef("https://uor.foundation/type/ConstrainedType"),
            )],
        },
        // v0.2.1: Inhabitance dispatch table and its three rules
        Individual {
            id: "https://uor.foundation/predicate/InhabitanceDispatchTable",
            type_: "https://uor.foundation/predicate/DispatchTable",
            label: "InhabitanceDispatchTable",
            comment: "The predicate:DispatchTable governing \
                      resolver:InhabitanceResolver. Three rules form a \
                      partition of type:ConstrainedType: Is2SatShape \
                      \u{2192} TwoSatDecider (priority 0), IsHornShape \
                      \u{2192} HornSatDecider (priority 1), \
                      IsResidualFragment \u{2192} ResidualVerdictResolver \
                      (priority 2, catch-all). Total coverage is enforced \
                      by reduction:DispatchCoverageCheck; DispatchMiss is \
                      unreachable for this table.",
            properties: &[
                (
                    "https://uor.foundation/predicate/isExhaustive",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/predicate/isMutuallyExclusive",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/predicate/inhabitance_rule_2sat",
            type_: "https://uor.foundation/predicate/DispatchRule",
            label: "inhabitance_rule_2sat",
            comment: "Dispatch rule 1 of InhabitanceDispatchTable: \
                      Is2SatShape \u{2192} TwoSatDecider at priority 0.",
            properties: &[
                (
                    "https://uor.foundation/predicate/dispatchPredicate",
                    IndividualValue::IriRef("https://uor.foundation/predicate/Is2SatShape"),
                ),
                (
                    "https://uor.foundation/predicate/dispatchTarget",
                    IndividualValue::IriRef("https://uor.foundation/resolver/TwoSatDecider"),
                ),
                (
                    "https://uor.foundation/predicate/dispatchPriority",
                    IndividualValue::Int(0),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/predicate/inhabitance_rule_horn",
            type_: "https://uor.foundation/predicate/DispatchRule",
            label: "inhabitance_rule_horn",
            comment: "Dispatch rule 2 of InhabitanceDispatchTable: \
                      IsHornShape \u{2192} HornSatDecider at priority 1.",
            properties: &[
                (
                    "https://uor.foundation/predicate/dispatchPredicate",
                    IndividualValue::IriRef("https://uor.foundation/predicate/IsHornShape"),
                ),
                (
                    "https://uor.foundation/predicate/dispatchTarget",
                    IndividualValue::IriRef("https://uor.foundation/resolver/HornSatDecider"),
                ),
                (
                    "https://uor.foundation/predicate/dispatchPriority",
                    IndividualValue::Int(1),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/predicate/inhabitance_rule_residual",
            type_: "https://uor.foundation/predicate/DispatchRule",
            label: "inhabitance_rule_residual",
            comment: "Dispatch rule 3 (catch-all) of InhabitanceDispatchTable: \
                      IsResidualFragment \u{2192} ResidualVerdictResolver at \
                      priority 2. Ensures total coverage.",
            properties: &[
                (
                    "https://uor.foundation/predicate/dispatchPredicate",
                    IndividualValue::IriRef("https://uor.foundation/predicate/IsResidualFragment"),
                ),
                (
                    "https://uor.foundation/predicate/dispatchTarget",
                    IndividualValue::IriRef(
                        "https://uor.foundation/resolver/ResidualVerdictResolver",
                    ),
                ),
                (
                    "https://uor.foundation/predicate/dispatchPriority",
                    IndividualValue::Int(2),
                ),
            ],
        },
    ]
}
