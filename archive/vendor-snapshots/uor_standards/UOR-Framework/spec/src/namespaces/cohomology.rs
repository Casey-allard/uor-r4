//! `cohomology/` namespace — Cochain complexes, sheaf cohomology, and local-to-global reasoning.
//!
//! The contravariant arm of structural reasoning. Defines cochain complexes,
//! coboundary operators, sheaves, stalks, sections, and gluing obstructions
//! that detect when local solutions fail to globalize.
//!
//! **Space classification:** `bridge` — kernel-computed, user-requested.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `cohomology/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "cohomology",
            iri: NS_COHOMOLOGY,
            label: "UOR Cohomology",
            comment:
                "Cochain complexes, sheaf cohomology, and local-to-global obstruction detection.",
            space: Space::Bridge,
            imports: &[
                NS_SCHEMA,
                NS_HOMOLOGY,
                NS_RESOLVER,
                NS_OBSERVABLE,
                NS_PARTITION,
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
            id: "https://uor.foundation/cohomology/CochainGroup",
            label: "CochainGroup",
            comment: "A cochain group: the dual of a chain group, maps chains to coefficients.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cohomology/CoboundaryOperator",
            label: "CoboundaryOperator",
            comment: "The coboundary operator δ^k: C^k → C^{k+1}. Satisfies δ² = 0.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cohomology/CochainComplex",
            label: "CochainComplex",
            comment: "A cochain complex: a sequence of cochain groups connected by coboundary operators.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cohomology/CohomologyGroup",
            label: "CohomologyGroup",
            comment: "The k-th cohomology group H^k = ker(δ^k) / im(δ^{k-1}). Measures k-dimensional obstructions.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cohomology/Sheaf",
            label: "Sheaf",
            comment: "A sheaf F over a simplicial complex: assigns data to each simplex with restriction maps.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cohomology/Stalk",
            label: "Stalk",
            comment: "A stalk F_σ: the local data of a sheaf at a simplex σ.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cohomology/Section",
            label: "Section",
            comment: "A global section of a sheaf: a consistent choice of local data across all simplices.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cohomology/LocalSection",
            label: "LocalSection",
            comment: "A local section: a consistent choice of data over a subcomplex.",
            subclass_of: &["https://uor.foundation/cohomology/Section"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cohomology/RestrictionMap",
            label: "RestrictionMap",
            comment: "A restriction map ρ_{σ,τ}: maps data from a simplex to a face.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/cohomology/GluingObstruction",
            label: "GluingObstruction",
            comment: "A gluing obstruction: a cohomology class that detects when local sections fail to glue.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Own-domain properties
        Property {
            id: "https://uor.foundation/cohomology/cochainDegree",
            label: "cochainDegree",
            comment: "The degree k of this cochain group C^k.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/CochainGroup"),
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/cohomology/cochainRank",
            label: "cochainRank",
            comment: "The rank (dimension) of this cochain group.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/CochainGroup"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/cohomology/dualOf",
            label: "dualOf",
            comment: "The chain group that this cochain group is dual to.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/CochainGroup"),
            range: "https://uor.foundation/homology/ChainGroup",
        },
        Property {
            id: "https://uor.foundation/cohomology/coboundarySource",
            label: "coboundarySource",
            comment: "The source cochain group of this coboundary operator.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/CoboundaryOperator"),
            range: "https://uor.foundation/cohomology/CochainGroup",
        },
        Property {
            id: "https://uor.foundation/cohomology/coboundaryTarget",
            label: "coboundaryTarget",
            comment: "The target cochain group of this coboundary operator.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/CoboundaryOperator"),
            range: "https://uor.foundation/cohomology/CochainGroup",
        },
        Property {
            id: "https://uor.foundation/cohomology/satisfiesCoboundarySquaredZero",
            label: "satisfiesCoboundarySquaredZero",
            comment: "Whether this coboundary operator satisfies δ² = 0.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/CoboundaryOperator"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/cohomology/hasCochainGroup",
            label: "hasCochainGroup",
            comment: "A cochain group belonging to this cochain complex.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/cohomology/CochainComplex"),
            range: "https://uor.foundation/cohomology/CochainGroup",
        },
        Property {
            id: "https://uor.foundation/cohomology/hasCoboundary",
            label: "hasCoboundary",
            comment: "A coboundary operator belonging to this cochain complex.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/cohomology/CochainComplex"),
            range: "https://uor.foundation/cohomology/CoboundaryOperator",
        },
        Property {
            id: "https://uor.foundation/cohomology/cohomologyDegree",
            label: "cohomologyDegree",
            comment: "The degree k of this cohomology group H^k.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/CohomologyGroup"),
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/cohomology/cohomologyRank",
            label: "cohomologyRank",
            comment: "The rank (dimension) of this cohomology group.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/CohomologyGroup"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/cohomology/sheafOver",
            label: "sheafOver",
            comment: "The simplicial complex that this sheaf is defined over.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/Sheaf"),
            range: "https://uor.foundation/homology/SimplicialComplex",
        },
        Property {
            id: "https://uor.foundation/cohomology/coefficientIn",
            label: "coefficientIn",
            comment: "The coefficient ring of this sheaf.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/Sheaf"),
            range: "https://uor.foundation/schema/Ring",
        },
        Property {
            id: "https://uor.foundation/cohomology/hasStalks",
            label: "hasStalks",
            comment: "A stalk belonging to this sheaf.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/cohomology/Sheaf"),
            range: "https://uor.foundation/cohomology/Stalk",
        },
        Property {
            id: "https://uor.foundation/cohomology/stalkAt",
            label: "stalkAt",
            comment: "The simplex at which this stalk is located.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/Stalk"),
            range: "https://uor.foundation/homology/Simplex",
        },
        Property {
            id: "https://uor.foundation/cohomology/restrictsFrom",
            label: "restrictsFrom",
            comment: "The source simplex of this restriction map.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/RestrictionMap"),
            range: "https://uor.foundation/homology/Simplex",
        },
        Property {
            id: "https://uor.foundation/cohomology/restrictsTo",
            label: "restrictsTo",
            comment: "The target simplex (face) of this restriction map.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/RestrictionMap"),
            range: "https://uor.foundation/homology/Simplex",
        },
        Property {
            id: "https://uor.foundation/cohomology/hasGlobalSection",
            label: "hasGlobalSection",
            comment: "A global section of this sheaf.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/cohomology/Sheaf"),
            range: "https://uor.foundation/cohomology/Section",
        },
        Property {
            id: "https://uor.foundation/cohomology/obstructionClass",
            label: "obstructionClass",
            comment: "The cohomology class that this gluing obstruction represents.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/cohomology/GluingObstruction"),
            range: "https://uor.foundation/cohomology/CohomologyGroup",
        },
        // Gap B: GluingObstruction feedback to resolver
        Property {
            id: "https://uor.foundation/cohomology/addressesSuggestion",
            label: "addressesSuggestion",
            comment: "The refinement suggestion that, if applied, would resolve this gluing \
                      obstruction. Computed by the kernel when ψ₆ detects H^1 ≠ 0: the \
                      obstruction class indexes the site pair that is incompatible, and \
                      the suggestion targets that pair with a new bridging constraint.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/cohomology/GluingObstruction"),
            range: "https://uor.foundation/resolver/RefinementSuggestion",
        },
        // Cross-NS bridge property
        Property {
            id: "https://uor.foundation/cohomology/sheafAnalysis",
            label: "sheafAnalysis",
            comment: "The sheaf analysis associated with a resolution state.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ResolutionState"),
            range: "https://uor.foundation/cohomology/Sheaf",
        },
    ]
}

/// Returns identity individuals with original `Str` values for lhs/rhs/forAll.
pub(crate) fn raw_individuals() -> Vec<Individual> {
    raw_individuals_vec()
}

fn individuals() -> Vec<Individual> {
    crate::model::rewrite_identity_ast_refs(raw_individuals_vec())
}

fn raw_individuals_vec() -> Vec<Individual> {
    vec![
        Individual {
            id: "https://uor.foundation/cohomology/coboundarySquaredZero",
            type_: "https://uor.foundation/op/Identity",
            label: "coboundarySquaredZero",
            comment: "δ² = 0: the coboundary of a coboundary is zero.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("δ^{k+1}(δ^k(f))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("f ∈ C^k")),

                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/cohomology/deRhamDuality",
            type_: "https://uor.foundation/op/Identity",
            label: "deRhamDuality",
            comment: "Discrete de Rham duality: H^k ≅ Hom(H_k, R).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("H^k(K; R)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Hom(H_k(K), R)")),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str("simplicial complex K, ring R"),
                ),

                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/cohomology/sheafCohomologyBridge",
            type_: "https://uor.foundation/op/Identity",
            label: "sheafCohomologyBridge",
            comment: "Sheaf cohomology equals simplicial cohomology for constant sheaves.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("H^k(K; F_R)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("H^k(K; R)")),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str("constant sheaf F_R over K"),
                ),

                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/cohomology/localGlobalPrinciple",
            type_: "https://uor.foundation/op/Identity",
            label: "localGlobalPrinciple",
            comment: "Local-global principle: H^1(K; F) = 0 implies all local sections glue to global sections.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("H^1(K; F) = 0")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("all local sections glue")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("sheaf F over K")),

                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
    ]
}
