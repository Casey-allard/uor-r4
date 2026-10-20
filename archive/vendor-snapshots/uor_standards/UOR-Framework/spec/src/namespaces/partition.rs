//! `partition/` namespace — Irreducibility partitions of the ring (Amendment 5).
//!
//! The partition map Π : T_n → Part(R_n) is the central function of the UOR
//! Framework. It maps a type declaration to a four-component partition of the
//! ring, classifying every ring element as irreducible, reducible, a unit,
//! or exterior to the carrier.
//!
//! Amendment 9 adds site budget formalization: site coordinates, budget
//! accounting, and site pinning — the completeness criterion for type
//! declarations.
//!
//! **Space classification:** `bridge` — produced by the kernel, consumed by user-space.

use crate::model::iris::*;
use crate::model::{Class, Individual, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `partition/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "partition",
            iri: NS_PARTITION,
            label: "UOR Partitions",
            comment: "Irreducibility partitions produced by type resolution. \
                      A partition divides the ring into four disjoint components: \
                      Irreducible, Reducible, Units, and Exterior.",
            space: Space::Bridge,
            imports: &[NS_SCHEMA, NS_TYPE],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn individuals() -> Vec<Individual> {
    vec![
        // v0.2.2 Phase E — PartitionComponent individuals (closed
        // catalogue of 4 partition classifications).
        Individual {
            id: "https://uor.foundation/partition/Irreducible",
            type_: "https://uor.foundation/partition/PartitionComponent",
            label: "Irreducible",
            comment: "The irreducible component: elements that admit no \
                      non-trivial factorization within the ring.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/partition/Reducible",
            type_: "https://uor.foundation/partition/PartitionComponent",
            label: "Reducible",
            comment: "The reducible component: elements that factor into \
                      non-trivial parts.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/partition/Units",
            type_: "https://uor.foundation/partition/PartitionComponent",
            label: "Units",
            comment: "The unit component: invertible elements of the ring.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/partition/Exterior",
            type_: "https://uor.foundation/partition/PartitionComponent",
            label: "Exterior",
            comment: "The exterior component: elements outside the factorization \
                      domain (e.g., zero or ring-boundary values).",
            properties: &[],
        },
    ]
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/partition/Partition",
            label: "Partition",
            comment: "A four-component partition of R_n produced by resolving a \
                      type declaration. The four components — Irreducible, Reducible, \
                      Units, Exterior — are mutually disjoint and exhaustive over \
                      the carrier.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/partition/Component",
            label: "Component",
            comment: "A single component of a partition: a set of datum values \
                      belonging to one of the four categories.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/partition/SiteIndex",
                "https://uor.foundation/partition/FreeRank",
            ],
        },
        Class {
            id: "https://uor.foundation/partition/IrreducibleSet",
            label: "IrreducibleSet",
            comment: "The set of irreducible elements under the active type: elements \
                      whose only factorizations involve units or themselves. \
                      Analogous to prime elements in a ring.",
            subclass_of: &["https://uor.foundation/partition/Component"],
            disjoint_with: &[
                "https://uor.foundation/partition/ReducibleSet",
                "https://uor.foundation/partition/UnitGroup",
                "https://uor.foundation/partition/Complement",
            ],
        },
        Class {
            id: "https://uor.foundation/partition/ReducibleSet",
            label: "ReducibleSet",
            comment: "The set of reducible non-unit elements: elements that can be \
                      expressed as a product of two or more non-unit elements.",
            subclass_of: &["https://uor.foundation/partition/Component"],
            disjoint_with: &[
                "https://uor.foundation/partition/IrreducibleSet",
                "https://uor.foundation/partition/UnitGroup",
                "https://uor.foundation/partition/Complement",
            ],
        },
        Class {
            id: "https://uor.foundation/partition/UnitGroup",
            label: "UnitGroup",
            comment: "The set of invertible elements (units) in the carrier: elements \
                      with a multiplicative inverse. In Z/(2^n)Z, the units are the \
                      odd integers.",
            subclass_of: &["https://uor.foundation/partition/Component"],
            disjoint_with: &[
                "https://uor.foundation/partition/IrreducibleSet",
                "https://uor.foundation/partition/ReducibleSet",
                "https://uor.foundation/partition/Complement",
            ],
        },
        Class {
            id: "https://uor.foundation/partition/Complement",
            label: "Complement",
            comment: "Elements of R_n that fall outside the active carrier — i.e., \
                      outside the type's domain. These are ring elements that do \
                      not participate in the current type resolution.",
            subclass_of: &["https://uor.foundation/partition/Component"],
            disjoint_with: &[
                "https://uor.foundation/partition/IrreducibleSet",
                "https://uor.foundation/partition/ReducibleSet",
                "https://uor.foundation/partition/UnitGroup",
            ],
        },
        // Amendment 9: Free Rank classes
        Class {
            id: "https://uor.foundation/partition/SiteIndex",
            label: "SiteIndex",
            comment: "A single site coordinate in the iterated Z/2Z fibration. \
                      Each site represents one binary degree of freedom in the \
                      ring's structure. The total number of sites equals the \
                      quantum level n.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/partition/FreeRank",
                "https://uor.foundation/partition/Component",
            ],
        },
        Class {
            id: "https://uor.foundation/partition/FreeRank",
            label: "FreeRank",
            comment: "The site budget for a partition: an accounting of how many \
                      sites are pinned (determined by constraints) versus free \
                      (still available for further refinement). A closed budget \
                      means all sites are pinned and the type is fully resolved.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/partition/SiteIndex",
                "https://uor.foundation/partition/Component",
            ],
        },
        Class {
            id: "https://uor.foundation/partition/SiteBinding",
            label: "SiteBinding",
            comment: "A record of a single site being pinned by a constraint. \
                      Links a specific site coordinate to the constraint that \
                      determined its value.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 37: Partition Tensor Product (Gap 8)
        Class {
            id: "https://uor.foundation/partition/PartitionProduct",
            label: "PartitionProduct",
            comment: "The tensor product of two partitions: partition(A × B) = \
                      partition(A) ⊗ partition(B). The four-component structure \
                      combines component-wise under the product type construction \
                      (PT_2a). Carries leftFactor and rightFactor links to the \
                      operand partitions.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/partition/PartitionCoproduct",
                "https://uor.foundation/partition/CartesianPartitionProduct",
            ],
        },
        Class {
            id: "https://uor.foundation/partition/PartitionCoproduct",
            label: "PartitionCoproduct",
            comment: "The coproduct (disjoint union) of two partitions: \
                      partition(A + B) = partition(A) ⊕ partition(B). The \
                      four-component structure combines via disjoint union under \
                      the sum type construction (PT_2b). Carries leftSummand and \
                      rightSummand links to the operand partitions.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/partition/PartitionProduct",
                "https://uor.foundation/partition/CartesianPartitionProduct",
            ],
        },
        // Product/Coproduct Completion Amendment — Gap 3 (CartesianPartitionProduct)
        Class {
            id: "https://uor.foundation/partition/CartesianPartitionProduct",
            label: "CartesianPartitionProduct",
            comment: "The Cartesian product of partitions. Classifies the nerve \
                      topology of A ⊠ B as a simplicial product (χ \
                      multiplicative per CPT_3, Betti by Künneth per CPT_4) \
                      rather than a site-disjoint union (χ additive — \
                      PartitionProduct). Site budget is |S_A| + |S_B| per \
                      CPT_1 — the bit width of the product state space. \
                      Partition-ness is asserted via leftCartesianFactor / \
                      rightCartesianFactor (both ranged at Partition), matching \
                      the sibling pattern for PartitionProduct and \
                      PartitionCoproduct. Satisfies CPT_1–CPT_6 per this \
                      amendment.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/partition/PartitionProduct",
                "https://uor.foundation/partition/PartitionCoproduct",
            ],
        },
        // Product/Coproduct Completion Amendment — Gap 4 (TagSite)
        Class {
            id: "https://uor.foundation/partition/TagSite",
            label: "TagSite",
            comment: "The distinguishing site in a PartitionCoproduct whose \
                      value (0 or 1) selects the variant. Logically, the tag \
                      is not a data site of either operand (ST_6) and carries \
                      exactly the ln 2 entropy quantum (ST_2). Its physical \
                      placement in a flat constraint layout follows the \
                      foundation layout convention: \
                      layoutTagSite = max(SITE_COUNT(A), SITE_COUNT(B)), so \
                      the tag does not collide with any inherited bookkeeping \
                      sites when operands are themselves coproducts.",
            subclass_of: &["https://uor.foundation/partition/SiteIndex"],
            disjoint_with: &[],
        },
        // v0.2.2 Phase D (Q4) — observable backing the siteConstraintKind
        // BoundConstraint individual.
        Class {
            id: "https://uor.foundation/partition/FreeRankObservable",
            label: "FreeRankObservable",
            comment: "Observes the free-rank of the partition associated with \
                      a Datum's site context, recording the count of unbound \
                      sites at the moment of observation. Used as the bound \
                      observable for the siteConstraintKind BoundConstraint.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        // v0.2.2 Phase E — enum class classifying partition components.
        Class {
            id: "https://uor.foundation/partition/PartitionComponent",
            label: "PartitionComponent",
            comment: "Closed enumeration of partition component kinds: \
                      Irreducible (non-factorizable), Reducible (factorizable \
                      into non-trivial parts), Units (invertible), Exterior \
                      (outside the factorization domain). Codegen treats this \
                      as an enum class with exactly 4 individuals.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/partition/irreducibles",
            label: "irreducibles",
            comment: "The irreducible component of this partition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Partition"),
            range: "https://uor.foundation/partition/IrreducibleSet",
        },
        Property {
            id: "https://uor.foundation/partition/reducibles",
            label: "reducibles",
            comment: "The reducible component of this partition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Partition"),
            range: "https://uor.foundation/partition/ReducibleSet",
        },
        Property {
            id: "https://uor.foundation/partition/units",
            label: "units",
            comment: "The units component of this partition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Partition"),
            range: "https://uor.foundation/partition/UnitGroup",
        },
        Property {
            id: "https://uor.foundation/partition/exterior",
            label: "exterior",
            comment: "The exterior component of this partition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Partition"),
            range: "https://uor.foundation/partition/Complement",
        },
        Property {
            id: "https://uor.foundation/partition/member",
            label: "member",
            comment: "A datum value belonging to this partition component.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/partition/Component"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/partition/cardinality",
            label: "cardinality",
            comment: "The number of elements in this partition component. \
                      The cardinalities of the four components must sum to 2^n.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Component"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/partition/density",
            label: "density",
            comment: "The irreducible density of this partition: |Irr| / |A|, \
                      where A is the active carrier.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Partition"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/partition/sourceType",
            label: "sourceType",
            comment: "The type declaration that was resolved to produce this \
                      partition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Partition"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/partition/wittLength",
            label: "wittLength",
            comment: "The Witt level n at which this partition was computed. \
                      The ring has 2^n elements at this level.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Partition"),
            range: XSD_POSITIVE_INTEGER,
        },
        // Amendment 9: Free Rank properties
        Property {
            id: "https://uor.foundation/partition/sitePosition",
            label: "sitePosition",
            comment: "The zero-based position of this site coordinate within \
                      the iterated fibration. Position 0 is the least significant \
                      bit; position n-1 is the most significant.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/SiteIndex"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/partition/siteState",
            label: "siteState",
            comment: "The current state of this site coordinate: 'pinned' if \
                      determined by a constraint, 'free' if still available for \
                      refinement.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/SiteIndex"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/partition/siteBudget",
            label: "siteBudget",
            comment: "The site budget associated with this partition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Partition"),
            range: "https://uor.foundation/partition/FreeRank",
        },
        Property {
            id: "https://uor.foundation/partition/totalSites",
            label: "totalSites",
            comment: "The total number of site coordinates in this budget, \
                      equal to the quantum level n.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/FreeRank"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/partition/pinnedCount",
            label: "pinnedCount",
            comment: "The number of site coordinates currently pinned by \
                      constraints.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/FreeRank"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/partition/freeRank",
            label: "freeRank",
            comment: "The number of site coordinates still free (not yet \
                      pinned). Equals totalSites - pinnedCount.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/FreeRank"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/partition/isClosed",
            label: "isClosed",
            comment: "Whether all sites in this budget are pinned. A closed \
                      budget means the type is fully resolved and the partition \
                      is complete.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/FreeRank"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/partition/hasSite",
            label: "hasSite",
            comment: "A site coordinate belonging to this budget.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/partition/FreeRank"),
            range: "https://uor.foundation/partition/SiteIndex",
        },
        Property {
            id: "https://uor.foundation/partition/pinnedBy",
            label: "pinnedBy",
            comment: "The constraint that pins this site coordinate.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/SiteBinding"),
            range: "https://uor.foundation/type/Constraint",
        },
        Property {
            id: "https://uor.foundation/partition/pinsCoordinate",
            label: "pinsCoordinate",
            comment: "The site coordinate that this pinning determines.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/SiteBinding"),
            range: "https://uor.foundation/partition/SiteIndex",
        },
        Property {
            id: "https://uor.foundation/partition/hasBinding",
            label: "hasBinding",
            comment: "A site pinning record in this budget.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/partition/FreeRank"),
            range: "https://uor.foundation/partition/SiteBinding",
        },
        // Amendment 31: Reversible computation properties (RC_1–RC_4)
        Property {
            id: "https://uor.foundation/partition/ancillaSite",
            label: "ancillaSite",
            comment: "An ancilla site coordinate paired with this site for \
                      reversible computation (RC_1–RC_4 ancilla model).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/SiteIndex"),
            range: "https://uor.foundation/partition/SiteIndex",
        },
        Property {
            id: "https://uor.foundation/partition/reversibleStrategy",
            label: "reversibleStrategy",
            comment: "True when this site budget uses a reversible computation \
                      strategy preserving information through ancilla sites.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/FreeRank"),
            range: XSD_BOOLEAN,
        },
        // Amendment 37: Complement formal criteria (Gap 2)
        Property {
            id: "https://uor.foundation/partition/exteriorCriteria",
            label: "exteriorCriteria",
            comment: "The formal membership criterion for this Complement: \
                      x ∈ Ext(T) iff x ∉ carrier(T). The Complement is \
                      context-dependent on the active type T (FPM_9).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Complement"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // Amendment 37: Partition exhaustiveness (Gap 3)
        Property {
            id: "https://uor.foundation/partition/isExhaustive",
            label: "isExhaustive",
            comment: "Whether the four components of this partition are exhaustive \
                      over R_n: |Irr| + |Red| + |Unit| + |Ext| = 2^n (FPM_8). \
                      Set by the kernel after verification.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Partition"),
            range: XSD_BOOLEAN,
        },
        // Amendment 37: Partition tensor product properties (Gap 8)
        Property {
            id: "https://uor.foundation/partition/leftFactor",
            label: "leftFactor",
            comment: "The left operand partition of this tensor product.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/PartitionProduct"),
            range: "https://uor.foundation/partition/Partition",
        },
        Property {
            id: "https://uor.foundation/partition/rightFactor",
            label: "rightFactor",
            comment: "The right operand partition of this tensor product.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/PartitionProduct"),
            range: "https://uor.foundation/partition/Partition",
        },
        Property {
            id: "https://uor.foundation/partition/leftSummand",
            label: "leftSummand",
            comment: "The left operand partition of this coproduct.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/PartitionCoproduct"),
            range: "https://uor.foundation/partition/Partition",
        },
        Property {
            id: "https://uor.foundation/partition/rightSummand",
            label: "rightSummand",
            comment: "The right operand partition of this coproduct.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/PartitionCoproduct"),
            range: "https://uor.foundation/partition/Partition",
        },
        // Product/Coproduct Completion Amendment — Gap 3 (CartesianPartitionProduct factors)
        Property {
            id: "https://uor.foundation/partition/leftCartesianFactor",
            label: "leftCartesianFactor",
            comment: "The left operand partition of this Cartesian partition product.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/CartesianPartitionProduct"),
            range: "https://uor.foundation/partition/Partition",
        },
        Property {
            id: "https://uor.foundation/partition/rightCartesianFactor",
            label: "rightCartesianFactor",
            comment: "The right operand partition of this Cartesian partition product.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/CartesianPartitionProduct"),
            range: "https://uor.foundation/partition/Partition",
        },
        // Product/Coproduct Completion Amendment — Gap 4 (TagSite links)
        Property {
            id: "https://uor.foundation/partition/tagSiteOf",
            label: "tagSiteOf",
            comment: "The tag site distinguishing the variants of a \
                      PartitionCoproduct. Logically distinct from every data \
                      site of either operand (ST_6) and carries the ln 2 \
                      entropy quantum of ST_2.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/Partition"),
            range: "https://uor.foundation/partition/TagSite",
        },
        Property {
            id: "https://uor.foundation/partition/tagValue",
            label: "tagValue",
            comment: "The boolean value (false = 0, true = 1) assigned to a \
                      tag site. false selects the left variant of the \
                      PartitionCoproduct; true selects the right variant.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/partition/TagSite"),
            range: XSD_BOOLEAN,
        },
        // Product/Coproduct Completion Amendment — Q4 resolution
        Property {
            id: "https://uor.foundation/partition/productCategoryLevel",
            label: "productCategoryLevel",
            comment: "The categorical level at which this construction is a \
                      product / coproduct. Values: 'partition_classification' \
                      (PartitionProduct, PartitionCoproduct), or \
                      'nerve_topology' (CartesianPartitionProduct). Prevents \
                      misreading the product vs coproduct distinction across \
                      levels.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: true,
            domain: Some("https://uor.foundation/partition/Partition"),
            range: XSD_STRING,
        },
    ]
}
