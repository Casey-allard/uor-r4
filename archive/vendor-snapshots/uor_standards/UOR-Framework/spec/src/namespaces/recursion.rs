//! `recursion/` namespace — Bounded recursion.
//!
//! The `recursion/` namespace formalizes self-referential computations with
//! well-founded descent measures guaranteeing termination. Distinct from
//! `stream:ProductiveStream` which models unbounded coinductive sequences;
//! recursion/ models finite inductive decomposition.
//!
//! - **Amendment 78**: 7 classes, 10 properties, 0 individuals (identities in op/)
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{Class, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `recursion/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "recursion",
            iri: NS_RECURSION,
            label: "UOR Bounded Recursion",
            comment: "Self-referential computations with well-founded \
                      descent measures guaranteeing termination.",
            space: Space::Kernel,
            imports: &[
                NS_OP,
                NS_SCHEMA,
                NS_PREDICATE,
                NS_MORPHISM,
                NS_EFFECT,
                NS_FAILURE,
                NS_TRACE,
            ],
        },
        classes: classes(),
        properties: properties(),
        individuals: vec![],
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/recursion/BoundedRecursion",
            label: "BoundedRecursion",
            comment: "A self-referential computation parameterized by a \
                      descent measure. Each recursive step strictly \
                      decreases the measure; the computation terminates \
                      when the base case predicate is satisfied.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/recursion/DescentMeasure",
            label: "DescentMeasure",
            comment: "A well-founded function from computation state to \
                      xsd:nonNegativeInteger. Strictly decreases on every \
                      recursive step.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/recursion/BaseCase",
            label: "BaseCase",
            comment: "The match arm selected when the descent measure \
                      reaches zero or the base predicate is satisfied. \
                      Produces a direct result without further recursion.",
            subclass_of: &["https://uor.foundation/predicate/MatchArm"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/recursion/RecursiveCase",
            label: "RecursiveCase",
            comment: "The match arm selected when the descent measure is \
                      positive. Applies a decomposition step, decreases \
                      the measure, and invokes the recursion.",
            subclass_of: &["https://uor.foundation/predicate/MatchArm"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/recursion/RecursiveStep",
            label: "RecursiveStep",
            comment: "A single step in a bounded recursion: the \
                      decomposition applied before the recursive invocation. \
                      Must strictly decrease the DescentMeasure.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/recursion/RecursionTrace",
            label: "RecursionTrace",
            comment: "A computation trace recording the sequence of \
                      recursive steps, measure values, and the base case \
                      result. The trace length is bounded by the initial \
                      measure value.",
            subclass_of: &["https://uor.foundation/trace/ComputationTrace"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/recursion/StructuralRecursion",
            label: "StructuralRecursion",
            comment: "A bounded recursion where the descent measure is the \
                      structural size of the input type (site count, \
                      constraint count, or operad nesting depth).",
            subclass_of: &["https://uor.foundation/recursion/BoundedRecursion"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Object properties
        Property {
            id: "https://uor.foundation/recursion/measure",
            label: "measure",
            comment: "The descent measure guaranteeing termination.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/recursion/BoundedRecursion"),
            range: "https://uor.foundation/recursion/DescentMeasure",
        },
        Property {
            id: "https://uor.foundation/recursion/baseCase",
            label: "baseCase",
            comment: "The base case match arm.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/recursion/BoundedRecursion"),
            range: "https://uor.foundation/recursion/BaseCase",
        },
        Property {
            id: "https://uor.foundation/recursion/recursiveCase",
            label: "recursiveCase",
            comment: "The recursive case match arm.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/recursion/BoundedRecursion"),
            range: "https://uor.foundation/recursion/RecursiveCase",
        },
        Property {
            id: "https://uor.foundation/recursion/stepDecomposition",
            label: "stepDecomposition",
            comment: "The transform applied to decompose the input before \
                      recursion.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/recursion/RecursiveStep"),
            range: "https://uor.foundation/morphism/Transform",
        },
        Property {
            id: "https://uor.foundation/recursion/stepMeasurePre",
            label: "stepMeasurePre",
            comment: "The measure value before this step.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/recursion/RecursiveStep"),
            range: "https://uor.foundation/recursion/DescentMeasure",
        },
        Property {
            id: "https://uor.foundation/recursion/stepMeasurePost",
            label: "stepMeasurePost",
            comment: "The measure value after this step.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/recursion/RecursiveStep"),
            range: "https://uor.foundation/recursion/DescentMeasure",
        },
        Property {
            id: "https://uor.foundation/recursion/basePredicate",
            label: "basePredicate",
            comment: "The predicate that identifies the base case.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/recursion/BoundedRecursion"),
            range: "https://uor.foundation/predicate/Predicate",
        },
        Property {
            id: "https://uor.foundation/recursion/recursionBody",
            label: "recursionBody",
            comment: "The computation applied at each recursive step (may \
                      reference itself via its content address).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/recursion/BoundedRecursion"),
            range: "https://uor.foundation/morphism/ComputationDatum",
        },
        // Datatype properties
        Property {
            id: "https://uor.foundation/recursion/initialMeasure",
            label: "initialMeasure",
            comment: "The measure value at the start of recursion. Bounds \
                      the maximum recursion depth.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/recursion/BoundedRecursion"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/recursion/measureValue",
            label: "measureValue",
            comment: "The current value of the descent measure.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/recursion/DescentMeasure"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
    ]
}
