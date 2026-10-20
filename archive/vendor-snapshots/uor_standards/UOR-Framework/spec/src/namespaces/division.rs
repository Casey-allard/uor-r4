//! `division/` namespace — Division algebras.
//!
//! The `division/` namespace formalizes the four normed division algebras
//! R, C, H, O and the Cayley-Dickson construction that builds each from
//! the previous. Hurwitz's theorem proves these are the only normed
//! division algebras over the reals.
//!
//! - **Amendment 67**: 5 classes, 11 properties, 7 individuals
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `division/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "division",
            iri: NS_DIVISION,
            label: "UOR Division Algebras",
            comment: "The four normed division algebras R, C, H, O and the \
                      Cayley-Dickson construction.",
            space: Space::Kernel,
            imports: &[NS_OP, NS_CONVERGENCE],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/division/NormedDivisionAlgebra",
            label: "NormedDivisionAlgebra",
            comment: "An algebra over R that is a division ring with \
                      multiplicative norm. Exactly four exist (Hurwitz \
                      theorem): R, C, H, O.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/division/CayleyDicksonConstruction",
            label: "CayleyDicksonConstruction",
            comment: "The doubling construction that builds each division \
                      algebra from the previous: R \u{2192} C \u{2192} H \
                      \u{2192} O.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/division/MultiplicationTable",
            label: "MultiplicationTable",
            comment: "The explicit product rules for a division algebra\u{2019}s \
                      basis elements.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/division/AlgebraCommutator",
            label: "AlgebraCommutator",
            comment: "The commutator \u{005b}a,b\u{005d} = ab \u{2212} ba. \
                      Zero for R and C; non-zero for H and O.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/division/AlgebraAssociator",
            label: "AlgebraAssociator",
            comment: "The associator \u{005b}a,b,c\u{005d} = (ab)c \u{2212} \
                      a(bc). Zero for R, C, H; non-zero for O.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // NormedDivisionAlgebra properties
        Property {
            id: "https://uor.foundation/division/algebraDimension",
            label: "algebraDimension",
            comment: "The dimension of this division algebra (1, 2, 4, or 8).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/division/NormedDivisionAlgebra"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/division/isCommutative",
            label: "isCommutative",
            comment: "Whether multiplication in this algebra is commutative.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/division/NormedDivisionAlgebra"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/division/isAssociative",
            label: "isAssociative",
            comment: "Whether multiplication in this algebra is associative.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/division/NormedDivisionAlgebra"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/division/basisElements",
            label: "basisElements",
            comment: "The basis elements of this division algebra.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/division/NormedDivisionAlgebra"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/division/algebraMultiplicationTable",
            label: "algebraMultiplicationTable",
            comment: "The multiplication table for this algebra.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/division/NormedDivisionAlgebra"),
            range: "https://uor.foundation/division/MultiplicationTable",
        },
        // CayleyDicksonConstruction properties
        Property {
            id: "https://uor.foundation/division/cayleyDicksonSource",
            label: "cayleyDicksonSource",
            comment: "The source algebra of the Cayley-Dickson doubling.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/division/CayleyDicksonConstruction"),
            range: "https://uor.foundation/division/NormedDivisionAlgebra",
        },
        Property {
            id: "https://uor.foundation/division/cayleyDicksonTarget",
            label: "cayleyDicksonTarget",
            comment: "The target algebra of the Cayley-Dickson doubling.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/division/CayleyDicksonConstruction"),
            range: "https://uor.foundation/division/NormedDivisionAlgebra",
        },
        Property {
            id: "https://uor.foundation/division/adjoinedElement",
            label: "adjoinedElement",
            comment: "The new basis element adjoined by this doubling step.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/division/CayleyDicksonConstruction"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/division/conjugationRule",
            label: "conjugationRule",
            comment: "The conjugation and multiplication rule for the adjoined element.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/division/CayleyDicksonConstruction"),
            range: XSD_STRING,
        },
        // Amendment 80: commutatorFormula/associatorFormula removed
        // (typed replacements in convergence/ as commutatorRef/associatorRef)
    ]
}

fn individuals() -> Vec<Individual> {
    vec![
        Individual {
            id: "https://uor.foundation/division/RealAlgebra",
            type_: "https://uor.foundation/division/NormedDivisionAlgebra",
            label: "RealAlgebra",
            comment: "The real numbers R: dimension 1, commutative, associative.",
            properties: &[
                (
                    "https://uor.foundation/division/algebraDimension",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/division/isCommutative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/division/isAssociative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/division/basisElements",
                    IndividualValue::Str("{1}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/division/ComplexAlgebra",
            type_: "https://uor.foundation/division/NormedDivisionAlgebra",
            label: "ComplexAlgebra",
            comment: "The complex numbers C: dimension 2, commutative, associative.",
            properties: &[
                (
                    "https://uor.foundation/division/algebraDimension",
                    IndividualValue::Int(2),
                ),
                (
                    "https://uor.foundation/division/isCommutative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/division/isAssociative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/division/basisElements",
                    IndividualValue::Str("{1, i}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/division/QuaternionAlgebra",
            type_: "https://uor.foundation/division/NormedDivisionAlgebra",
            label: "QuaternionAlgebra",
            comment: "The quaternions H: dimension 4, non-commutative, associative.",
            properties: &[
                (
                    "https://uor.foundation/division/algebraDimension",
                    IndividualValue::Int(4),
                ),
                (
                    "https://uor.foundation/division/isCommutative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/division/isAssociative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/division/basisElements",
                    IndividualValue::Str("{1, i, j, k}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/division/OctonionAlgebra",
            type_: "https://uor.foundation/division/NormedDivisionAlgebra",
            label: "OctonionAlgebra",
            comment: "The octonions O: dimension 8, non-commutative, non-associative.",
            properties: &[
                (
                    "https://uor.foundation/division/algebraDimension",
                    IndividualValue::Int(8),
                ),
                (
                    "https://uor.foundation/division/isCommutative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/division/isAssociative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/division/basisElements",
                    IndividualValue::Str("{1, i, j, k, l, il, jl, kl}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/division/cayleyDickson_R_to_C",
            type_: "https://uor.foundation/division/CayleyDicksonConstruction",
            label: "cayleyDickson_R_to_C",
            comment: "Cayley-Dickson doubling R \u{2192} C: adjoin i with \
                      i\u{00b2} = \u{2212}1.",
            properties: &[
                (
                    "https://uor.foundation/division/cayleyDicksonSource",
                    IndividualValue::IriRef("https://uor.foundation/division/RealAlgebra"),
                ),
                (
                    "https://uor.foundation/division/cayleyDicksonTarget",
                    IndividualValue::IriRef("https://uor.foundation/division/ComplexAlgebra"),
                ),
                (
                    "https://uor.foundation/division/adjoinedElement",
                    IndividualValue::Str("i"),
                ),
                (
                    "https://uor.foundation/division/conjugationRule",
                    IndividualValue::Str("i\u{00b2} = \u{2212}1"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/division/cayleyDickson_C_to_H",
            type_: "https://uor.foundation/division/CayleyDicksonConstruction",
            label: "cayleyDickson_C_to_H",
            comment: "Cayley-Dickson doubling C \u{2192} H: adjoin j with \
                      j\u{00b2} = \u{2212}1, ij = k, ji = \u{2212}k.",
            properties: &[
                (
                    "https://uor.foundation/division/cayleyDicksonSource",
                    IndividualValue::IriRef("https://uor.foundation/division/ComplexAlgebra"),
                ),
                (
                    "https://uor.foundation/division/cayleyDicksonTarget",
                    IndividualValue::IriRef("https://uor.foundation/division/QuaternionAlgebra"),
                ),
                (
                    "https://uor.foundation/division/adjoinedElement",
                    IndividualValue::Str("j"),
                ),
                (
                    "https://uor.foundation/division/conjugationRule",
                    IndividualValue::Str("ij = k, ji = \u{2212}k"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/division/cayleyDickson_H_to_O",
            type_: "https://uor.foundation/division/CayleyDicksonConstruction",
            label: "cayleyDickson_H_to_O",
            comment: "Cayley-Dickson doubling H \u{2192} O: adjoin l, \
                      non-associative Fano plane products.",
            properties: &[
                (
                    "https://uor.foundation/division/cayleyDicksonSource",
                    IndividualValue::IriRef("https://uor.foundation/division/QuaternionAlgebra"),
                ),
                (
                    "https://uor.foundation/division/cayleyDicksonTarget",
                    IndividualValue::IriRef("https://uor.foundation/division/OctonionAlgebra"),
                ),
                (
                    "https://uor.foundation/division/adjoinedElement",
                    IndividualValue::Str("l"),
                ),
                (
                    "https://uor.foundation/division/conjugationRule",
                    IndividualValue::Str("non-associative Fano plane products"),
                ),
            ],
        },
    ]
}
