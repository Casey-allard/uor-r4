//! `effect/` namespace — Effect algebra.
//!
//! The `effect/` namespace formalizes typed endomorphisms on `state:Context`
//! classified by site target. Effects are the atomic unit of state mutation
//! in the kernel: each effect maps one site-budget configuration to another.
//!
//! - **Amendment 71**: 9 classes, 14 properties, 0 individuals (identities in op/)
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{Class, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `effect/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "effect",
            iri: NS_EFFECT,
            label: "UOR Effect Algebra",
            comment: "Typed endomorphisms on state:Context classified by site \
                      target. Formalizes what reduction guard-effect pairs do to \
                      the site budget.",
            space: Space::Kernel,
            imports: &[NS_OP, NS_STATE, NS_PARTITION, NS_TYPE, NS_CERT],
        },
        classes: classes(),
        properties: properties(),
        individuals: vec![],
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/effect/Effect",
            label: "Effect",
            comment: "A typed endomorphism on state:Context. Maps one \
                      site-budget configuration to another. The atomic unit \
                      of state mutation in the kernel.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/effect/ReversibleEffect",
            label: "ReversibleEffect",
            comment: "An effect that has a well-defined inverse. \
                      PinningEffect and PhaseEffect are reversible; \
                      ExternalEffect and CompositeEffect are not in general.",
            subclass_of: &["https://uor.foundation/effect/Effect"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/effect/PinningEffect",
            label: "PinningEffect",
            comment: "Pins a single free site to a definite value. \
                      Decrements freeRank by exactly 1. The effect produced \
                      by constraint resolution.",
            subclass_of: &["https://uor.foundation/effect/ReversibleEffect"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/effect/UnbindingEffect",
            label: "UnbindingEffect",
            comment: "Releases a pinned site back to free state. Increments \
                      freeRank by exactly 1. The effect produced by session \
                      boundary reset.",
            subclass_of: &["https://uor.foundation/effect/Effect"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/effect/PhaseEffect",
            label: "PhaseEffect",
            comment: "Rotates the reduction phase angle by \u{03a9}^k. Does \
                      not alter the site budget. The effect produced by \
                      reduction step transitions.",
            subclass_of: &["https://uor.foundation/effect/ReversibleEffect"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/effect/CompositeEffect",
            label: "CompositeEffect",
            comment: "An ordered sequence of effects applied atomically. \
                      The composition E\u{2081} ; E\u{2082} applies E\u{2081} \
                      then E\u{2082}.",
            subclass_of: &["https://uor.foundation/effect/Effect"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/effect/ExternalEffect",
            label: "ExternalEffect",
            comment: "An opaque effect declared by a Prism implementation. \
                      The kernel treats it as a site-budget transformation \
                      satisfying the declared commutation contract. Must \
                      carry an effect:externalEffectShape linking to a \
                      conformance:EffectShape.",
            subclass_of: &["https://uor.foundation/effect/Effect"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/effect/EffectTarget",
            label: "EffectTarget",
            comment: "The set of site coordinates that an effect reads or \
                      writes. Determines commutation.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/effect/DisjointnessWitness",
            label: "DisjointnessWitness",
            comment: "A certificate that two EffectTargets have empty \
                      intersection, enabling commutative reordering.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Object properties
        Property {
            id: "https://uor.foundation/effect/effectTarget",
            label: "effectTarget",
            comment: "The site coordinates this effect touches.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/Effect"),
            range: "https://uor.foundation/effect/EffectTarget",
        },
        Property {
            id: "https://uor.foundation/effect/targetSites",
            label: "targetSites",
            comment: "The individual site coordinates in this target set.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/effect/EffectTarget"),
            range: "https://uor.foundation/partition/SiteIndex",
        },
        Property {
            id: "https://uor.foundation/effect/compositeHead",
            label: "compositeHead",
            comment: "The first effect in the composite sequence.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/CompositeEffect"),
            range: "https://uor.foundation/effect/Effect",
        },
        Property {
            id: "https://uor.foundation/effect/compositeTail",
            label: "compositeTail",
            comment: "Subsequent effects in the composite sequence (ordered \
                      by effect:compositeIndex).",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/effect/CompositeEffect"),
            range: "https://uor.foundation/effect/Effect",
        },
        Property {
            id: "https://uor.foundation/effect/disjointnessLeft",
            label: "disjointnessLeft",
            comment: "The left target in the disjointness claim.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/DisjointnessWitness"),
            range: "https://uor.foundation/effect/EffectTarget",
        },
        Property {
            id: "https://uor.foundation/effect/disjointnessRight",
            label: "disjointnessRight",
            comment: "The right target in the disjointness claim.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/DisjointnessWitness"),
            range: "https://uor.foundation/effect/EffectTarget",
        },
        Property {
            id: "https://uor.foundation/effect/preContext",
            label: "preContext",
            comment: "The context before effect application.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/Effect"),
            range: "https://uor.foundation/state/Context",
        },
        Property {
            id: "https://uor.foundation/effect/postContext",
            label: "postContext",
            comment: "The context after effect application.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/Effect"),
            range: "https://uor.foundation/state/Context",
        },
        Property {
            id: "https://uor.foundation/effect/externalEffectShape",
            label: "externalEffectShape",
            comment: "The conformance shape that this external effect \
                      satisfies.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/ExternalEffect"),
            // Full IRI string: effect/ cannot import conformance/
            range: "https://uor.foundation/conformance/EffectShape",
        },
        // Datatype properties
        Property {
            id: "https://uor.foundation/effect/freeRankDelta",
            label: "freeRankDelta",
            comment: "Change in freeRank: \u{2212}1 for PinningEffect, +1 \
                      for UnbindingEffect, 0 for PhaseEffect.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/Effect"),
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/effect/phaseAngleDelta",
            label: "phaseAngleDelta",
            comment: "The phase rotation applied, expressed as \u{03a9}^k.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/PhaseEffect"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/effect/compositeIndex",
            label: "compositeIndex",
            comment: "Position within a CompositeEffect sequence.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/Effect"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/effect/isCommutativeWith",
            label: "isCommutativeWith",
            comment: "Computed: true iff effectTargets are disjoint with the \
                      compared effect. Derived from DisjointnessWitness \
                      existence.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/Effect"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/effect/targetCardinality",
            label: "targetCardinality",
            comment: "Number of site coordinates in this target set.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/effect/EffectTarget"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
    ]
}
