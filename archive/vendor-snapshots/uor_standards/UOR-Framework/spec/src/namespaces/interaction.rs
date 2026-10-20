//! `interaction/` namespace — Interaction algebra.
//!
//! The `interaction/` namespace formalizes multi-entity interaction:
//! commutator states, associator triples, negotiation convergence,
//! and interaction nerve topology.
//!
//! - **Amendment 68**: 9 classes, 20 properties
//!
//! **Space classification:** `bridge` — kernel-computed, user-consumed.

use crate::model::iris::*;
use crate::model::{Class, Individual, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `interaction/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "interaction",
            iri: NS_INTERACTION,
            label: "UOR Interaction Algebra",
            comment: "Multi-entity interaction: commutator states, associator \
                      triples, negotiation convergence.",
            space: Space::Bridge,
            imports: &[NS_OP, NS_CONVERGENCE, NS_STATE, NS_OBSERVABLE, NS_SCHEMA],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/interaction/InteractionContext",
            label: "InteractionContext",
            comment: "Two entities sharing sites through composed operations. \
                      Properties: entityA, entityB, sharedSiteMask, commutatorNorm.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/interaction/CommutatorState",
            label: "CommutatorState",
            comment: "The norm \u{2016}[\u{03b4}_A, \u{03b9}_B]\u{2016} on \
                      shared sites. Zero iff operators commute on the shared domain.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/interaction/AssociatorState",
            label: "AssociatorState",
            comment: "The norm of the three-way associator on shared sites.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/interaction/AssociatorTriple",
            label: "AssociatorTriple",
            comment: "Three entities whose interaction exhibits non-associativity \
                      due to read-write interleaving.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/interaction/ThreeWaySite",
            label: "ThreeWaySite",
            comment: "A site shared by all three entities in an AssociatorTriple.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/interaction/NegotiationTrace",
            label: "NegotiationTrace",
            comment: "Sequence of CommutatorStates across interaction steps.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/interaction/MutualModelTrace",
            label: "MutualModelTrace",
            comment: "Sequence of AssociatorStates across interaction steps.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/interaction/InteractionNerve",
            label: "InteractionNerve",
            comment: "Simplicial complex of N-entity coupling.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/interaction/InteractionComposition",
            label: "InteractionComposition",
            comment: "IC(A,B) = \u{03ba}(session(A,B)). Combined \
                      interaction-composition operator.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // InteractionContext properties
        Property {
            id: "https://uor.foundation/interaction/entityA",
            label: "entityA",
            comment: "First entity in the interaction context.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/InteractionContext"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/interaction/entityB",
            label: "entityB",
            comment: "Second entity in the interaction context.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/InteractionContext"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/interaction/sharedSiteMask",
            label: "sharedSiteMask",
            comment: "Bitmask identifying which sites are shared between \
                      the two entities.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/InteractionContext"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/interaction/commutatorNorm",
            label: "commutatorNorm",
            comment: "The norm of the commutator on shared sites. \
                      Zero iff the operators commute.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/InteractionContext"),
            range: XSD_DECIMAL,
        },
        // CommutatorState property
        Property {
            id: "https://uor.foundation/interaction/commutatorValue",
            label: "commutatorValue",
            comment: "The computed commutator norm value.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/CommutatorState"),
            range: XSD_DECIMAL,
        },
        // AssociatorState property
        Property {
            id: "https://uor.foundation/interaction/associatorNorm",
            label: "associatorNorm",
            comment: "The norm of the three-way associator on shared sites.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/AssociatorState"),
            range: XSD_DECIMAL,
        },
        // Amendment 80: typed replacements for AssociatorTriple string props
        Property {
            id: "https://uor.foundation/interaction/tripleComponentA",
            label: "tripleComponentA",
            comment: "First component datum in the associator triple.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/AssociatorTriple"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/interaction/tripleComponentB",
            label: "tripleComponentB",
            comment: "Second component datum in the associator triple.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/AssociatorTriple"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/interaction/tripleComponentC",
            label: "tripleComponentC",
            comment: "Third component datum in the associator triple.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/AssociatorTriple"),
            range: "https://uor.foundation/schema/Datum",
        },
        // ThreeWaySite properties
        Property {
            id: "https://uor.foundation/interaction/sitePosition",
            label: "sitePosition",
            comment: "The position index of the shared site.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/ThreeWaySite"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/interaction/leftGroupingValue",
            label: "leftGroupingValue",
            comment: "Value under left-associative grouping (AB)C.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/ThreeWaySite"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/interaction/rightGroupingValue",
            label: "rightGroupingValue",
            comment: "Value under right-associative grouping A(BC).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/ThreeWaySite"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/interaction/isPinned",
            label: "isPinned",
            comment: "Whether this site is pinned by a lease constraint.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/ThreeWaySite"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/interaction/pinningPair",
            label: "pinningPair",
            comment: "Identifier of the entity pair that pins this site.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/ThreeWaySite"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // NegotiationTrace properties
        Property {
            id: "https://uor.foundation/interaction/negotiationSteps",
            label: "negotiationSteps",
            comment: "Number of steps in the negotiation trace.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/NegotiationTrace"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/interaction/isConvergent",
            label: "isConvergent",
            comment: "Whether the trace converges to zero commutator \
                      or zero associator.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/NegotiationTrace"),
            range: XSD_BOOLEAN,
        },
        // MutualModelTrace property
        Property {
            id: "https://uor.foundation/interaction/modelConvergent",
            label: "modelConvergent",
            comment: "Whether the mutual model trace converges.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/MutualModelTrace"),
            range: XSD_BOOLEAN,
        },
        // InteractionNerve properties
        Property {
            id: "https://uor.foundation/interaction/nerveDimension",
            label: "nerveDimension",
            comment: "Maximum dimension of the interaction nerve \
                      simplicial complex.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/InteractionNerve"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/interaction/nerveBettiNumbers",
            label: "nerveBettiNumbers",
            comment: "Betti number sequence of the interaction nerve.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/InteractionNerve"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // InteractionComposition properties
        Property {
            id: "https://uor.foundation/interaction/reificationDepth",
            label: "reificationDepth",
            comment: "Depth of the interaction composition reification.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/InteractionComposition"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 71: Missing interaction properties (5)
        Property {
            id: "https://uor.foundation/interaction/traceLength",
            label: "traceLength",
            comment: "The number of steps in a negotiation trace.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/NegotiationTrace"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/interaction/convergenceRate",
            label: "convergenceRate",
            comment: "The rate at which the negotiation trace converges.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/NegotiationTrace"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/interaction/terminalValue",
            label: "terminalValue",
            comment: "The terminal value of the negotiation trace.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/NegotiationTrace"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // Amendment 80: typed replacements
        Property {
            id: "https://uor.foundation/interaction/associatorProfileRef",
            label: "associatorProfileRef",
            comment: "Reference to the observable describing the \
                      associativity profile.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/AssociatorTriple"),
            range: "https://uor.foundation/observable/Observable",
        },
        Property {
            id: "https://uor.foundation/interaction/associatorNormValue",
            label: "associatorNormValue",
            comment: "The maximum norm of the associator for this triple.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/interaction/AssociatorTriple"),
            range: XSD_DECIMAL,
        },
    ]
}

fn individuals() -> Vec<Individual> {
    vec![]
}
