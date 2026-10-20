//! `operad/` namespace — Operad composition.
//!
//! The `operad/` namespace formalizes structural type nesting via operad
//! composition. Governs how types compose: Table(Tuple(Sequence(Symbol(...)))).
//!
//! - **Amendment 70**: 2 classes, 6 properties
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{Class, Individual, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `operad/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "operad",
            iri: NS_OPERAD,
            label: "UOR Operad Composition",
            comment: "Structural type nesting via operad composition. Governs \
                      how types compose: Table(Tuple(Sequence(Symbol(...)))).",
            space: Space::Kernel,
            imports: &[NS_TYPE, NS_CARRY, NS_MORPHISM],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/operad/StructuralOperad",
            label: "StructuralOperad",
            comment: "The composition structure on the eight structural types. \
                      Governs how types nest.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/operad/OperadComposition",
            label: "OperadComposition",
            comment: "A specific nesting: outer type F applied to inner type G.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Amendment 80: typed replacement for operadDescription
        Property {
            id: "https://uor.foundation/operad/operadStructure",
            label: "operadStructure",
            comment: "The structural operad defining this composition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/operad/StructuralOperad"),
            range: "https://uor.foundation/operad/StructuralOperad",
        },
        // OperadComposition properties
        Property {
            id: "https://uor.foundation/operad/outerType",
            label: "outerType",
            comment: "The outer type F in the nesting F(G).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/operad/OperadComposition"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/operad/innerType",
            label: "innerType",
            comment: "The inner type G in the nesting F(G).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/operad/OperadComposition"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/operad/composedType",
            label: "composedType",
            comment: "The resulting composed type F(G).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/operad/OperadComposition"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/operad/composedSiteCount",
            label: "composedSiteCount",
            comment: "Site count of the composed type F(G).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/operad/OperadComposition"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/operad/composedGrounding",
            label: "composedGrounding",
            comment: "Grounding of the composed type F(G).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/operad/OperadComposition"),
            range: "https://uor.foundation/morphism/GroundingMap",
        },
    ]
}

fn individuals() -> Vec<Individual> {
    vec![]
}
