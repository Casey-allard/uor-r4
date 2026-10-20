//! `foundation/` namespace — Foundation-level layout invariants (Product/Coproduct Completion Amendment).
//!
//! Carries invariants that complement op-namespace theorems. Where
//! op-namespace identities (PT_*, ST_*, CPT_*) quantify over
//! ontology-level quantities like `siteBudget`, this namespace
//! quantifies over foundation-defined quantities — `SITE_COUNT`
//! arithmetic, the byte-level encoding of `ConstraintRef::Affine`
//! tag-pinners — that the foundation's `validate_const()` pass and
//! the sealed mint primitives enforce. Violations surface as typed
//! `GenericImpossibilityWitness` citations against the IRIs defined
//! here rather than against op-namespace theorems, so downstream
//! consumers can distinguish a layout-level failure from a
//! theorem-level failure.
//!
//! **Space classification:** `bridge` — produced by the foundation
//! layer, consumed by downstream consumers auditing witness failures.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `foundation/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "foundation",
            iri: NS_FOUNDATION,
            label: "UOR Foundation Layout Invariants",
            comment: "Foundation-level layout invariants complementing \
                      op-namespace theorems. Quantifies over foundation-\
                      defined SITE_COUNT arithmetic and ConstraintRef byte \
                      patterns, not over ontology-level siteBudget.",
            space: Space::Bridge,
            imports: &[NS_PARTITION, NS_TYPE],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![Class {
        id: "https://uor.foundation/foundation/LayoutInvariant",
        label: "LayoutInvariant",
        comment: "A foundation-level layout invariant. Each instance \
                  describes an arithmetic or encoding identity that the \
                  foundation's mint primitives and validate_const() pass \
                  enforce at compile time, distinct from the \
                  ontology-level theorem individuals carried by the op \
                  namespace. Violations produce GenericImpossibilityWitness \
                  citations against the specific LayoutInvariant IRI, \
                  letting consumers distinguish a layout-level failure \
                  from a theorem-level failure.",
        subclass_of: &[OWL_THING],
        disjoint_with: &[],
    }]
}

fn properties() -> Vec<Property> {
    vec![Property {
        id: "https://uor.foundation/foundation/layoutRule",
        label: "layoutRule",
        comment: "The arithmetic or encoding identity this LayoutInvariant \
                  asserts, expressed as a human-readable string for \
                  inspection in documentation and debugging output.",
        kind: PropertyKind::Datatype,
        functional: true,
        required: true,
        domain: Some("https://uor.foundation/foundation/LayoutInvariant"),
        range: XSD_STRING,
    }]
}

fn individuals() -> Vec<Individual> {
    vec![
        Individual {
            id: "https://uor.foundation/foundation/ProductLayoutWidth",
            type_: "https://uor.foundation/foundation/LayoutInvariant",
            label: "ProductLayoutWidth",
            comment: "PartitionProduct layout-width invariant: products \
                      introduce no bookkeeping of their own, so layout \
                      widths add. Cited by primitive_partition_product when \
                      the caller-supplied combined SITE_COUNT differs from \
                      the sum of operand SITE_COUNTs.",
            properties: &[(
                "https://uor.foundation/foundation/layoutRule",
                IndividualValue::Str("SITE_COUNT(A × B) = SITE_COUNT(A) + SITE_COUNT(B)"),
            )],
        },
        Individual {
            id: "https://uor.foundation/foundation/CartesianLayoutWidth",
            type_: "https://uor.foundation/foundation/LayoutInvariant",
            label: "CartesianLayoutWidth",
            comment: "CartesianPartitionProduct layout-width invariant: \
                      cartesian products introduce no bookkeeping either, \
                      so layout widths add the same way PartitionProduct \
                      does. The distinction between these two constructions \
                      lives at the nerve-topology level (χ multiplicative \
                      vs additive), not the layout level.",
            properties: &[(
                "https://uor.foundation/foundation/layoutRule",
                IndividualValue::Str("SITE_COUNT(A ⊠ B) = SITE_COUNT(A) + SITE_COUNT(B)"),
            )],
        },
        Individual {
            id: "https://uor.foundation/foundation/CoproductLayoutWidth",
            type_: "https://uor.foundation/foundation/LayoutInvariant",
            label: "CoproductLayoutWidth",
            comment: "PartitionCoproduct layout-width invariant: coproducts \
                      add exactly one tag site beyond the widest operand's \
                      full layout. Uses SITE_COUNT (not siteBudget) so \
                      nested coproducts whose operands carry inherited \
                      bookkeeping do not collide their outer tag with an \
                      inner tag site.",
            properties: &[(
                "https://uor.foundation/foundation/layoutRule",
                IndividualValue::Str("SITE_COUNT(A + B) = max(SITE_COUNT(A), SITE_COUNT(B)) + 1"),
            )],
        },
        Individual {
            id: "https://uor.foundation/foundation/CoproductTagEncoding",
            type_: "https://uor.foundation/foundation/LayoutInvariant",
            label: "CoproductTagEncoding",
            comment: "PartitionCoproduct canonical tag-pinner encoding: \
                      each variant's tag-pinning constraint is the \
                      canonical Affine form with all-zero coefficients \
                      except a single 1 at tag_site, with bias 0 for the \
                      left variant and bias −1 for the right. Semantically \
                      equivalent but non-normalized encodings (coefficient \
                      ≠ 1, or alternative biases, etc.) are rejected at \
                      mint time because content-addressing depends on the \
                      normalized byte pattern, not the semantic equivalence \
                      class.",
            properties: &[(
                "https://uor.foundation/foundation/layoutRule",
                IndividualValue::Str(
                    "Affine { coefficients: [0,…,0, 1 at tag_site], bias: 0 (left) | −1 (right) }",
                ),
            )],
        },
    ]
}
