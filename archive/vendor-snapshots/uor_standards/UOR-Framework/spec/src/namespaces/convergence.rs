//! `convergence/` namespace — Hopf convergence tower.
//!
//! The `convergence/` namespace formalizes the four-level convergence tower
//! R \u{2192} C \u{2192} H \u{2192} O corresponding to the four normed division
//! algebras. Each level carries a Hopf fibration fiber, Betti signature,
//! and characteristic identity.
//!
//! - **Amendment 66**: 5 classes, 13 properties, 8 individuals
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `convergence/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "convergence",
            iri: NS_CONVERGENCE,
            label: "UOR Convergence Tower",
            comment: "Hopf convergence tower: four levels R, C, H, O \
                      corresponding to the four normed division algebras \
                      of dimensions 1, 2, 4, 8. Each level carries a Hopf \
                      fibration fiber and Betti signature.",
            space: Space::Kernel,
            imports: &[NS_OP, NS_REDUCTION, NS_OBSERVABLE, NS_HOMOLOGY],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/convergence/ConvergenceLevel",
            label: "ConvergenceLevel",
            comment: "A level in the convergence tower. Four instances: \
                      R (dim 1), C (dim 2), H (dim 4), O (dim 8).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/convergence/HopfFiber",
            label: "HopfFiber",
            comment: "The fiber of the Hopf fibration at a convergence \
                      level. Four instances: S\u{2070}, S\u{00b9}, \
                      S\u{00b3}, S\u{2077}.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/convergence/ConvergenceResidual",
            label: "ConvergenceResidual",
            comment: "The unresolved structure at a convergence level. \
                      The \u{03b2}_{2^k\u{2212}1} = 1 Betti number that \
                      persists.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/convergence/CommutativeSubspace",
            label: "CommutativeSubspace",
            comment: "The subspace U(1) \u{2282} SU(2) selected when \
                      pairwise interaction converges.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/convergence/AssociativeSubalgebra",
            label: "AssociativeSubalgebra",
            comment: "The subspace H \u{2282} O selected when triple \
                      interaction converges.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // ConvergenceLevel properties
        Property {
            id: "https://uor.foundation/convergence/algebraDimension",
            label: "algebraDimension",
            comment: "The dimension of the division algebra at this level \
                      (1, 2, 4, or 8).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/ConvergenceLevel"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/convergence/bettiSignature",
            label: "bettiSignature",
            comment: "The Betti number signature at this convergence level.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/ConvergenceLevel"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/convergence/fiberType",
            label: "fiberType",
            comment: "The Hopf fiber associated with this convergence level.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/ConvergenceLevel"),
            range: "https://uor.foundation/convergence/HopfFiber",
        },
        Property {
            id: "https://uor.foundation/convergence/characteristicIdentity",
            label: "characteristicIdentity",
            comment: "The characteristic identity at this convergence level \
                      (existence, feedback, choice, self-reference).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/ConvergenceLevel"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/convergence/levelName",
            label: "levelName",
            comment: "Human-readable name of this convergence level.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/ConvergenceLevel"),
            range: XSD_STRING,
        },
        // HopfFiber properties
        Property {
            id: "https://uor.foundation/convergence/fiberDimension",
            label: "fiberDimension",
            comment: "The dimension of the Hopf fiber sphere.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/HopfFiber"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/convergence/totalSpace",
            label: "totalSpace",
            comment: "The total space of the Hopf fibration.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/HopfFiber"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/convergence/baseSpace",
            label: "baseSpace",
            comment: "The base space of the Hopf fibration.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/HopfFiber"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/convergence/fiberSphere",
            label: "fiberSphere",
            comment: "The fiber sphere designation (e.g. S\u{2070}, S\u{00b9}).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/HopfFiber"),
            range: XSD_STRING,
        },
        // ConvergenceResidual properties
        Property {
            id: "https://uor.foundation/convergence/residualBetti",
            label: "residualBetti",
            comment: "The persistent Betti number at this residual.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/ConvergenceResidual"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/convergence/residualDimension",
            label: "residualDimension",
            comment: "The dimension at which the residual persists.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/ConvergenceResidual"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 80: typed replacements for string properties
        Property {
            id: "https://uor.foundation/convergence/subspaceRef",
            label: "subspaceRef",
            comment: "The commutative subspace selected by pairwise \
                      convergence.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/CommutativeSubspace"),
            range: "https://uor.foundation/convergence/CommutativeSubspace",
        },
        Property {
            id: "https://uor.foundation/convergence/subalgebraRef",
            label: "subalgebraRef",
            comment: "The associative subalgebra selected by triple \
                      convergence.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/AssociativeSubalgebra"),
            range: "https://uor.foundation/convergence/AssociativeSubalgebra",
        },
        Property {
            id: "https://uor.foundation/convergence/commutatorRef",
            label: "commutatorRef",
            comment: "Reference to the commutator pair for this convergence.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/CommutativeSubspace"),
            range: "https://uor.foundation/observable/Commutator",
        },
        Property {
            id: "https://uor.foundation/convergence/associatorRef",
            label: "associatorRef",
            comment: "Reference to the associator triple for this convergence.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/convergence/AssociativeSubalgebra"),
            range: "https://uor.foundation/interaction/AssociatorTriple",
        },
    ]
}

fn individuals() -> Vec<Individual> {
    vec![
        // ConvergenceLevel individuals
        Individual {
            id: "https://uor.foundation/convergence/L0_State",
            type_: "https://uor.foundation/convergence/ConvergenceLevel",
            label: "L0_State",
            comment: "Level 0: R (reals), dimension 1, existence.",
            properties: &[
                (
                    "https://uor.foundation/convergence/algebraDimension",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/convergence/bettiSignature",
                    IndividualValue::Str("[1]"),
                ),
                (
                    "https://uor.foundation/convergence/fiberType",
                    IndividualValue::IriRef("https://uor.foundation/convergence/hopf_S0"),
                ),
                (
                    "https://uor.foundation/convergence/characteristicIdentity",
                    IndividualValue::Str("existence"),
                ),
                (
                    "https://uor.foundation/convergence/levelName",
                    IndividualValue::Str("R"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/convergence/L1_Memory",
            type_: "https://uor.foundation/convergence/ConvergenceLevel",
            label: "L1_Memory",
            comment: "Level 1: C (complex), dimension 2, feedback.",
            properties: &[
                (
                    "https://uor.foundation/convergence/algebraDimension",
                    IndividualValue::Int(2),
                ),
                (
                    "https://uor.foundation/convergence/bettiSignature",
                    IndividualValue::Str("[1,1]"),
                ),
                (
                    "https://uor.foundation/convergence/fiberType",
                    IndividualValue::IriRef("https://uor.foundation/convergence/hopf_S1"),
                ),
                (
                    "https://uor.foundation/convergence/characteristicIdentity",
                    IndividualValue::Str("feedback"),
                ),
                (
                    "https://uor.foundation/convergence/levelName",
                    IndividualValue::Str("C"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/convergence/L2_Agency",
            type_: "https://uor.foundation/convergence/ConvergenceLevel",
            label: "L2_Agency",
            comment: "Level 2: H (quaternions), dimension 4, choice.",
            properties: &[
                (
                    "https://uor.foundation/convergence/algebraDimension",
                    IndividualValue::Int(4),
                ),
                (
                    "https://uor.foundation/convergence/bettiSignature",
                    IndividualValue::Str("[1,0,0,1]"),
                ),
                (
                    "https://uor.foundation/convergence/fiberType",
                    IndividualValue::IriRef("https://uor.foundation/convergence/hopf_S3"),
                ),
                (
                    "https://uor.foundation/convergence/characteristicIdentity",
                    IndividualValue::Str("choice"),
                ),
                (
                    "https://uor.foundation/convergence/levelName",
                    IndividualValue::Str("H"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/convergence/L3_Self",
            type_: "https://uor.foundation/convergence/ConvergenceLevel",
            label: "L3_Self",
            comment: "Level 3: O (octonions), dimension 8, self-reference.",
            properties: &[
                (
                    "https://uor.foundation/convergence/algebraDimension",
                    IndividualValue::Int(8),
                ),
                (
                    "https://uor.foundation/convergence/bettiSignature",
                    IndividualValue::Str("[1,0,0,0,0,0,0,1]"),
                ),
                (
                    "https://uor.foundation/convergence/fiberType",
                    IndividualValue::IriRef("https://uor.foundation/convergence/hopf_S7"),
                ),
                (
                    "https://uor.foundation/convergence/characteristicIdentity",
                    IndividualValue::Str("self-reference"),
                ),
                (
                    "https://uor.foundation/convergence/levelName",
                    IndividualValue::Str("O"),
                ),
            ],
        },
        // HopfFiber individuals
        Individual {
            id: "https://uor.foundation/convergence/hopf_S0",
            type_: "https://uor.foundation/convergence/HopfFiber",
            label: "hopf_S0",
            comment: "Hopf fiber S\u{2070}: dimension 0, total space S\u{00b9}, base pt.",
            properties: &[
                (
                    "https://uor.foundation/convergence/fiberDimension",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/convergence/totalSpace",
                    IndividualValue::Str("S\u{00b9}"),
                ),
                (
                    "https://uor.foundation/convergence/baseSpace",
                    IndividualValue::Str("pt"),
                ),
                (
                    "https://uor.foundation/convergence/fiberSphere",
                    IndividualValue::Str("S\u{2070}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/convergence/hopf_S1",
            type_: "https://uor.foundation/convergence/HopfFiber",
            label: "hopf_S1",
            comment: "Hopf fiber S\u{00b9}: dimension 1, total space S\u{00b3}, base S\u{00b2}.",
            properties: &[
                (
                    "https://uor.foundation/convergence/fiberDimension",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/convergence/totalSpace",
                    IndividualValue::Str("S\u{00b3}"),
                ),
                (
                    "https://uor.foundation/convergence/baseSpace",
                    IndividualValue::Str("S\u{00b2}"),
                ),
                (
                    "https://uor.foundation/convergence/fiberSphere",
                    IndividualValue::Str("S\u{00b9}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/convergence/hopf_S3",
            type_: "https://uor.foundation/convergence/HopfFiber",
            label: "hopf_S3",
            comment: "Hopf fiber S\u{00b3}: dimension 3, total space S\u{2077}, base S\u{2074}.",
            properties: &[
                (
                    "https://uor.foundation/convergence/fiberDimension",
                    IndividualValue::Int(3),
                ),
                (
                    "https://uor.foundation/convergence/totalSpace",
                    IndividualValue::Str("S\u{2077}"),
                ),
                (
                    "https://uor.foundation/convergence/baseSpace",
                    IndividualValue::Str("S\u{2074}"),
                ),
                (
                    "https://uor.foundation/convergence/fiberSphere",
                    IndividualValue::Str("S\u{00b3}"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/convergence/hopf_S7",
            type_: "https://uor.foundation/convergence/HopfFiber",
            label: "hopf_S7",
            comment:
                "Hopf fiber S\u{2077}: dimension 7, total space S\u{00b9}\u{2075}, base S\u{2078}.",
            properties: &[
                (
                    "https://uor.foundation/convergence/fiberDimension",
                    IndividualValue::Int(7),
                ),
                (
                    "https://uor.foundation/convergence/totalSpace",
                    IndividualValue::Str("S\u{00b9}\u{2075}"),
                ),
                (
                    "https://uor.foundation/convergence/baseSpace",
                    IndividualValue::Str("S\u{2078}"),
                ),
                (
                    "https://uor.foundation/convergence/fiberSphere",
                    IndividualValue::Str("S\u{2077}"),
                ),
            ],
        },
    ]
}
