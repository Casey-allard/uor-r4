//! `monoidal/` namespace — Monoidal composition.
//!
//! The `monoidal/` namespace formalizes sequential composition of computations
//! via the monoidal product A \u{2297} B: the output of A feeds the input of B.
//! Includes the identity computation I and the associativity isomorphism.
//!
//! - **Amendment 69**: 3 classes, 8 properties
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{Class, Individual, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `monoidal/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "monoidal",
            iri: NS_MONOIDAL,
            label: "UOR Monoidal Composition",
            comment: "Sequential composition of computations via monoidal \
                      product A \u{2297} B.",
            space: Space::Kernel,
            imports: &[NS_OP, NS_REDUCTION, NS_CERT, NS_MORPHISM, NS_SCHEMA],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/monoidal/MonoidalProduct",
            label: "MonoidalProduct",
            comment: "A \u{2297} B: the sequential composition of two \
                      computations. Output of A feeds input of B.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/monoidal/MonoidalUnit",
            label: "MonoidalUnit",
            comment: "The identity computation I: passes input through \
                      unchanged. I \u{2297} A \u{2245} A \u{2245} A \
                      \u{2297} I.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/monoidal/MonoidalAssociator",
            label: "MonoidalAssociator",
            comment: "The witness that (A\u{2297}B)\u{2297}C \u{2245} \
                      A\u{2297}(B\u{2297}C). The associativity isomorphism.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Amendment 80: typed replacements for MonoidalProduct string props
        Property {
            id: "https://uor.foundation/monoidal/leftOperand",
            label: "leftOperand",
            comment: "The left operand in the monoidal product A \u{2297} B.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/monoidal/MonoidalProduct"),
            range: "https://uor.foundation/morphism/ComputationDatum",
        },
        Property {
            id: "https://uor.foundation/monoidal/rightOperand",
            label: "rightOperand",
            comment: "The right operand in the monoidal product A \u{2297} B.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/monoidal/MonoidalProduct"),
            range: "https://uor.foundation/morphism/ComputationDatum",
        },
        Property {
            id: "https://uor.foundation/monoidal/composedResult",
            label: "composedResult",
            comment: "The result datum of the composed computation A \u{2297} B.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/monoidal/MonoidalProduct"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/monoidal/saturationValue",
            label: "saturationValue",
            comment: "\u{03c3}(A\u{2297}B) relationship: saturation of the \
                      sequential composition.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/monoidal/MonoidalProduct"),
            range: XSD_DECIMAL,
        },
        // MonoidalUnit property (typed replacement)
        Property {
            id: "https://uor.foundation/monoidal/unitWitnessRef",
            label: "unitWitnessRef",
            comment: "Certificate witnessing I \u{2297} A \u{2245} A \u{2245} A \
                      \u{2297} I.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/monoidal/MonoidalUnit"),
            range: "https://uor.foundation/cert/Certificate",
        },
        // MonoidalAssociator properties (typed replacements)
        Property {
            id: "https://uor.foundation/monoidal/associatorLeft",
            label: "associatorLeft",
            comment: "The left-grouped product (A\u{2297}B)\u{2297}C.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/monoidal/MonoidalAssociator"),
            range: "https://uor.foundation/monoidal/MonoidalProduct",
        },
        Property {
            id: "https://uor.foundation/monoidal/associatorRight",
            label: "associatorRight",
            comment: "The right-grouped product A\u{2297}(B\u{2297}C).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/monoidal/MonoidalAssociator"),
            range: "https://uor.foundation/monoidal/MonoidalProduct",
        },
        Property {
            id: "https://uor.foundation/monoidal/associatorWitnessRef",
            label: "associatorWitnessRef",
            comment: "Certificate witnessing the associativity isomorphism \
                      (A\u{2297}B)\u{2297}C \u{2245} A\u{2297}(B\u{2297}C).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/monoidal/MonoidalAssociator"),
            range: "https://uor.foundation/cert/Certificate",
        },
    ]
}

fn individuals() -> Vec<Individual> {
    vec![]
}
