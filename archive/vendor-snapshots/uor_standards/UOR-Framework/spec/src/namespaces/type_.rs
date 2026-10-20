//! `type/` namespace — Runtime type declarations.
//!
//! Types are declared at runtime by Prism applications and parameterize the
//! resolution pipeline. A type declaration tells the resolver how to partition
//! the ring into irreducible, reducible, unit, and exterior elements.
//!
//! Amendment 10 adds constraint algebra: composable predicates that refine types
//! by pinning sites, plus metric axes that classify constraints by their
//! geometric effect.
//!
//! Amendment 25 adds the completeness certification pathway: `CompletenessCandidate`
//! (a ConstrainedType undergoing certification) and `CompletenessWitness` (a single
//! site-closing event), with four new properties supporting the certification loop.
//!
//! Amendment 61 adds 8 structural type individuals (ScalarType, SymbolType,
//! SequenceType, TupleType, GraphType, SetType, TreeType, TableType) as
//! instances of existing type classes with annotation properties describing
//! site count formulas, grounding rules, and structural constraints.
//!
//! **Space classification:** `user` — parameterizable at runtime.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `type/` namespace module.
///
/// Note: the module is named `type_` because `type` is a reserved keyword in Rust.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "type",
            iri: NS_TYPE,
            label: "UOR Type System",
            comment: "Runtime type declarations that parameterize the resolution \
                      pipeline. Types are declared by Prism applications and \
                      resolved to partitions of the ring.",
            space: Space::User,
            imports: &[NS_SCHEMA, NS_U, NS_OP],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/type/TypeDefinition",
            label: "TypeDefinition",
            comment: "A runtime type declaration. The root class for all UOR types. \
                      Each TypeDefinition, when resolved, produces a partition of \
                      the ring at the specified quantum level.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/type/Constraint",
                "https://uor.foundation/type/MetricAxis",
            ],
        },
        Class {
            id: "https://uor.foundation/type/PrimitiveType",
            label: "PrimitiveType",
            comment: "A primitive type defined by a fixed bit width. The carrier is \
                      the entire ring Z/(2^n)Z at the specified quantum level.",
            subclass_of: &["https://uor.foundation/type/TypeDefinition"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/ProductType",
            label: "ProductType",
            comment: "A product (Cartesian) type formed from multiple component \
                      types. The carrier is the product of the component carriers.",
            subclass_of: &["https://uor.foundation/type/TypeDefinition"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/SumType",
            label: "SumType",
            comment: "A sum (disjoint union) type formed from multiple variant \
                      types. The carrier is the disjoint union of the variant \
                      carriers.",
            subclass_of: &["https://uor.foundation/type/TypeDefinition"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/ConstrainedType",
            label: "ConstrainedType",
            comment: "A type formed by constraining a base type with a predicate. \
                      The carrier is the subset of the base carrier satisfying the \
                      constraint.",
            subclass_of: &["https://uor.foundation/type/TypeDefinition"],
            disjoint_with: &[],
        },
        // Amendment 10: Constraint Algebra classes
        Class {
            id: "https://uor.foundation/type/Constraint",
            label: "Constraint",
            comment: "A composable predicate that refines a type by pinning one or \
                      more site coordinates. Constraints are the parameterization \
                      mechanism for ConstrainedType.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/type/TypeDefinition",
                "https://uor.foundation/type/MetricAxis",
            ],
        },
        // v0.2.2 Phase D: the seven enumerated Constraint subclasses
        // (ResidueConstraint, CarryConstraint, DepthConstraint,
        // CompositeConstraint, HammingConstraint, SiteConstraint,
        // AffineConstraint) have been deleted. Their semantics are preserved
        // by the parametric `BoundConstraint<O: Observable, B: BoundShape>`
        // + `Conjunction<N>` surface declared below; the Rust codegen emits
        // the legacy names as type aliases over the parametric carrier.
        Class {
            id: "https://uor.foundation/type/MetricAxis",
            label: "MetricAxis",
            comment: "A classification axis for constraints by their geometric \
                      effect. The three axes — vertical (ring/additive), \
                      horizontal (Hamming/bitwise), diagonal (incompatibility) — \
                      form the tri-metric coordinate system of UOR.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/type/TypeDefinition",
                "https://uor.foundation/type/Constraint",
            ],
        },
        // Gap D: CompleteType
        Class {
            id: "https://uor.foundation/type/CompleteType",
            label: "CompleteType",
            comment: "A TypeDefinition certified to satisfy the UOR completeness criterion \
                      (IT_7d): its constraint nerve N(C) has Euler characteristic χ = n and \
                      all Betti numbers β_k = 0. A CompleteType guarantees that resolution \
                      closes the site budget in O(1) — no iterative refinement is required. \
                      Completeness is attested by a cert:CompletenessCertificate linked via \
                      cert:certifiedType. This class is not addressable from a type-expr \
                      position in the term language; references from term-language positions \
                      are rejected by the resolver.",
            subclass_of: &["https://uor.foundation/type/TypeDefinition"],
            disjoint_with: &[],
        },
        // Amendment 25: Completeness Certification Pathway
        Class {
            id: "https://uor.foundation/type/CompletenessCandidate",
            label: "CompletenessCandidate",
            comment: "A ConstrainedType actively undergoing the completeness certification \
                      pipeline. Links to the resolver:ResolutionState tracking the current \
                      iteration and to the resolver:CechNerve being computed. \
                      Disjoint from CompleteType (which is already certified). \
                      This class is not addressable from a type-expr position in \
                      the term language; references from term-language positions \
                      are rejected by the resolver.",
            subclass_of: &["https://uor.foundation/type/ConstrainedType"],
            disjoint_with: &["https://uor.foundation/type/CompleteType"],
        },
        Class {
            id: "https://uor.foundation/type/CompletenessWitness",
            label: "CompletenessWitness",
            comment: "A record of a single site-closing event: one constraint application \
                      that reduced the FreeRank deficit. Carries the applied constraint \
                      and the sitesClosed count. Forms the ordered audit trail between \
                      ConstrainedType and CompleteType.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 28: ψ-Pipeline Inversion (Type Synthesis)
        Class {
            id: "https://uor.foundation/type/TypeSynthesisGoal",
            label: "TypeSynthesisGoal",
            comment: "A specification of the desired topological properties of a type to be \
                      synthesised. Carries a target Euler characteristic \
                      (targetEulerCharacteristic) and a target Betti profile (zero or more \
                      targetBettiNumber assertions). The minimal goal for O(1) resolution is: \
                      targetEulerCharacteristic = n and all targetBettiNumber = 0 — the IT_7d \
                      profile.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/TypeSynthesisResult",
            label: "TypeSynthesisResult",
            comment: "The output of a TypeSynthesisResolver run. Contains the SynthesizedType, \
                      the realised topological signature (as a SynthesisSignature), and the \
                      SynthesisTrace recording the construction steps.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/SynthesizedType",
            label: "SynthesizedType",
            comment: "A ConstrainedType produced by the TypeSynthesisResolver. Distinguished \
                      from a hand-authored ConstrainedType by the presence of a \
                      type:synthesisResult link. May or may not be a CompleteType, depending \
                      on the synthesis goal. This class is not addressable from a type-expr \
                      position in the term language; references from term-language positions \
                      are rejected by the resolver.",
            subclass_of: &["https://uor.foundation/type/ConstrainedType"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/MinimalConstraintBasis",
            label: "MinimalConstraintBasis",
            comment: "The minimal set of constraints in the SynthesizedType's constraint set \
                      that is sufficient to realise the target topological signature. The \
                      minimality criterion is that removing any single member changes the \
                      realised signature.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 29: Quantum Level Spectral Sequence
        Class {
            id: "https://uor.foundation/type/WittLift",
            label: "WittLift",
            comment: "A ConstrainedType T' over R_{n+1} obtained by extending a \
                      ConstrainedType T over R_n. Carries a link to the base type (liftBase), \
                      the quantum level it lifts to (liftTargetLevel), and the LiftObstruction \
                      (if the lift fails to transfer completeness). A WittLift is a \
                      CompleteType iff its LiftObstruction is trivial.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/LiftObstruction",
            label: "LiftObstruction",
            comment: "The algebraic obstruction to a WittLift inheriting the completeness \
                      of its base type. Computed as the image of the spectral sequence \
                      differential d_2. If trivial (zero), the base type's completeness lifts. \
                      If non-trivial, at least one additional constraint is needed at the new \
                      quantum level.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 30: Monodromy Observables
        Class {
            id: "https://uor.foundation/type/TwistedType",
            label: "TwistedType",
            comment: "A ConstrainedType whose HolonomyGroup is non-trivial — at least one \
                      closed constraint path produces a non-identity dihedral element. A \
                      TwistedType may still be a CompleteType (IT_7d is a homological, not \
                      holonomic, criterion), but its resolution paths require tracking dihedral \
                      accumulation. This class is not addressable from a type-expr position \
                      in the term language; references from term-language positions are \
                      rejected by the resolver.",
            subclass_of: &["https://uor.foundation/type/ConstrainedType"],
            disjoint_with: &["https://uor.foundation/type/FlatType"],
        },
        Class {
            id: "https://uor.foundation/type/FlatType",
            label: "FlatType",
            comment: "A ConstrainedType whose HolonomyGroup is trivial — all closed constraint \
                      paths have identity monodromy. The constraint configuration is \
                      topologically flat: resolution is path-independent. This class is \
                      not addressable from a type-expr position in the term language; \
                      references from term-language positions are rejected by the resolver.",
            subclass_of: &["https://uor.foundation/type/ConstrainedType"],
            disjoint_with: &["https://uor.foundation/type/TwistedType"],
        },
        // Amendment 32: Superposed site state (RC_5)
        Class {
            id: "https://uor.foundation/type/SuperposedSiteState",
            label: "SuperposedSiteState",
            comment: "A type representing a superposition of site states where sites \
                      carry complex amplitudes rather than binary pinned/free \
                      assignments. Ontological realisation of RC_5 (Amendment 32). \
                      This class is not addressable from a type-expr position in \
                      the term language; references from term-language positions \
                      are rejected by the resolver.",
            subclass_of: &["https://uor.foundation/type/TypeDefinition"],
            disjoint_with: &[],
        },
        // Amendment 34: Morphospace Achievability
        Class {
            id: "https://uor.foundation/type/ForbiddenSignature",
            label: "ForbiddenSignature",
            comment: "A topological signature (χ, β_k) that is formally impossible \
                      to achieve for any ConstrainedType. Witnessed by an \
                      ImpossibilityWitness in proof/.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 36: Measurement Boundary
        Class {
            id: "https://uor.foundation/type/CollapsedSiteState",
            label: "CollapsedSiteState",
            comment: "A site state that has undergone projective collapse from a \
                      SuperposedSiteState to a definitive classical value. \
                      Topologically equivalent to a classically pinned site (QM_2).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 41: LiftChain and ObstructionChain
        Class {
            id: "https://uor.foundation/type/LiftChain",
            label: "LiftChain",
            comment: "An ordered composition of WittLift steps from liftSourceLevel \
                      (Q_j) to liftTargetLevel (Q_k) for any j < k. The canonical \
                      object certifying type completeness at arbitrary Q_k.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/ObstructionChain",
            label: "ObstructionChain",
            comment: "The complete ordered sequence of LiftObstruction records \
                      encountered and resolved along a LiftChain. An empty \
                      ObstructionChain means the tower is flat.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 56: Moduli Space and Deformation Complex
        Class {
            id: "https://uor.foundation/type/ModuliSpace",
            label: "ModuliSpace",
            comment: "The space of all CompleteTypes over R_n at a given quantum level.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/HolonomyStratum",
            label: "HolonomyStratum",
            comment: "A stratum indexed by a conjugacy class of subgroups of D_{2^n}.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/DeformationFamily",
            label: "DeformationFamily",
            comment: "A one-parameter family of constraint configurations \
                      parameterizing a path.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/VersalDeformation",
            label: "VersalDeformation",
            comment: "The versal deformation of a CompleteType T.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/ModuliTowerMap",
            label: "ModuliTowerMap",
            comment: "The map induced by WittLift from one moduli space to the next.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 60: Galois Connection
        Class {
            id: "https://uor.foundation/type/GaloisConnection",
            label: "GaloisConnection",
            comment: "The adjunction between the type lattice and the site lattice. \
                      The upper adjoint is type closure; the lower adjoint is site \
                      interior. \u{03c3} = lower adjoint evaluation; r = complement \
                      of upper adjoint image.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 77: Subtyping and Variance classes
        Class {
            id: "https://uor.foundation/type/TypeInclusion",
            label: "TypeInclusion",
            comment: "A morphism T\u{2081} \u{2192} T\u{2082} witnessing \
                      that T\u{2081} is a subtype of T\u{2082}: the \
                      constraint set of T\u{2081} is a superset of the \
                      constraint set of T\u{2082}.",
            // Full IRI string: type/ cannot import morphism/
            subclass_of: &["https://uor.foundation/morphism/Transform"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/VarianceAnnotation",
            label: "VarianceAnnotation",
            comment: "The variance of a structural type position under \
                      operad composition. One of Covariant, Contravariant, \
                      Invariant, or Bivariant.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/SubtypingLattice",
            label: "SubtypingLattice",
            comment: "The partial order on types induced by TypeInclusion. \
                      The top element is PrimitiveType (no constraints); \
                      the bottom elements are CompleteTypes (all sites \
                      pinned).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // v0.2.1 Phase 7a.6: Parametric defaults for type:Constraint
        // subclasses, consumed by #[derive(ConstrainedType)] at macro-crate
        // build time via the keywords.rs/defaults.rs artifacts generated
        // by uor-foundation-macros/build.rs.
        Class {
            id: "https://uor.foundation/type/ConstraintDefaults",
            label: "ConstraintDefaults",
            comment: "Declarative defaults for type:Constraint subclasses, \
                      consumed by #[derive(ConstrainedType)] at macro-crate \
                      build time.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // v0.2.2 Phase D (Q4) — parametric constraint surface.
        Class {
            id: "https://uor.foundation/type/BoundConstraint",
            label: "BoundConstraint",
            comment: "Parametric constraint carrier consumed by the codegen to \
                      emit `BoundConstraint<O: Observable, B: BoundShape>` in \
                      the Rust foundation. The (observable, bound_shape) pair \
                      picks the predicate form; the typed datatype properties \
                      (modulus, residue, hammingBound, ...) carry the \
                      parameters for that specific kind. Replaces the seven \
                      v0.2.1 enumerated Constraint subclasses with a \
                      parametric form; the legacy names survive as Rust type \
                      aliases (ResidueConstraint, HammingConstraint, ...).",
            subclass_of: &["https://uor.foundation/type/Constraint"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/BoundShape",
            label: "BoundShape",
            comment: "The predicate form a BoundConstraint imposes on its \
                      bound observable. Closed enumeration with exactly six \
                      named individuals (EqualBound, LessEqBound, \
                      GreaterEqBound, RangeContainBound, ResidueClassBound, \
                      AffineEqualBound). Adding a new bound shape is an \
                      ontology+grammar+codegen edit on the same protocol as \
                      adding a Witt level.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/type/Conjunction",
            label: "Conjunction",
            comment: "A conjunction of BoundConstraint instances. The \
                      conjuncts property is ordered. Replaces the v0.2.1 \
                      CompositeConstraint enumeration; the legacy name \
                      survives as a Rust type alias \
                      (CompositeConstraint<const N: usize> = Conjunction<N>).",
            subclass_of: &["https://uor.foundation/type/Constraint"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/type/bitWidth",
            label: "bitWidth",
            comment: "The bit width of a primitive type (the quantum level n). \
                      The carrier is Z/(2^n)Z.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/PrimitiveType"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/component",
            label: "component",
            comment: "A component type in a product type.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/ProductType"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        // Product/Coproduct Completion Amendment — Gap 4 (SumType extension)
        Property {
            id: "https://uor.foundation/type/variant",
            label: "variant",
            comment: "A variant type in a sum type. Ordering contract: \
                      variants are returned in canonical content-fingerprint \
                      order (ascending), matching the canonicalization used \
                      by coproduct_shape! and the content-address hashing \
                      pipeline, so introspection via variant() and the \
                      algebra_id derived from the same type agree on which \
                      variant is 'left' and which is 'right'. Author-order \
                      is not preserved.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/SumType"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/type/tagSite",
            label: "tagSite",
            comment: "The tag site distinguishing variants of this sum type \
                      at runtime (ST_6). Stored at the physical layout \
                      position max(SITE_COUNT(A), SITE_COUNT(B)) so the \
                      tag cannot collide with any inherited bookkeeping \
                      site when either operand is itself a coproduct.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/SumType"),
            range: "https://uor.foundation/partition/TagSite",
        },
        Property {
            id: "https://uor.foundation/type/baseType",
            label: "baseType",
            comment: "The base type that a constrained type restricts.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/ConstrainedType"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/type/contentAddress",
            label: "contentAddress",
            comment: "The content-derived address of this type definition, \
                      uniquely identifying the type in the UOR address space.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/TypeDefinition"),
            range: "https://uor.foundation/u/Element",
        },
        // Amendment 10: Constraint Algebra properties
        Property {
            id: "https://uor.foundation/type/hasConstraint",
            label: "hasConstraint",
            comment: "A typed constraint object applied to this constrained type. \
                      Replaces the deprecated string-based type:constraint property.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/ConstrainedType"),
            range: "https://uor.foundation/type/Constraint",
        },
        Property {
            id: "https://uor.foundation/type/modulus",
            label: "modulus",
            comment: "The modulus m of a residue constraint: x ≡ r (mod m).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/residue",
            label: "residue",
            comment: "The residue value r of a residue constraint: x ≡ r (mod m).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/carryPattern",
            label: "carryPattern",
            comment: "The carry propagation pattern of a carry constraint, \
                      expressed as a Datum at the appropriate quantum level.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/type/minDepth",
            label: "minDepth",
            comment: "The minimum factorization depth required by a depth \
                      constraint.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/maxDepth",
            label: "maxDepth",
            comment: "The maximum factorization depth allowed by a depth \
                      constraint.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/composedFrom",
            label: "composedFrom",
            comment: "A component constraint of this composite constraint.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/Conjunction"),
            range: "https://uor.foundation/type/Constraint",
        },
        // Amendment 95: Constraint completion properties (Workstream 3)
        Property {
            id: "https://uor.foundation/type/hammingBound",
            label: "hammingBound",
            comment: "Upper bound on the Hamming weight of the Datum.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/siteIndex",
            label: "siteIndex",
            comment: "Zero-based index of the pinned site coordinate.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/siteValue",
            label: "siteValue",
            comment: "The value the pinned site coordinate must equal \
                      (a Datum in the set {0, 1}).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/type/affineOffset",
            label: "affineOffset",
            comment: "Constant offset defining the affine subspace.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/type/affineGenerator",
            label: "affineGenerator",
            comment: "A generator of the affine subspace. Non-functional: \
                      multiple generators span the subspace.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/type/metricAxis",
            label: "metricAxis",
            comment: "The metric axis along which this constraint operates: \
                      vertical (ring), horizontal (Hamming), or diagonal \
                      (incompatibility).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/Constraint"),
            range: "https://uor.foundation/type/MetricAxis",
        },
        Property {
            id: "https://uor.foundation/type/pinsSites",
            label: "pinsSites",
            comment: "A site coordinate that this constraint pins when applied.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/Constraint"),
            range: "https://uor.foundation/partition/SiteIndex",
        },
        // type:axisSignature property removed by Amendment 23 (replaced by affectsAxis + axisSignatureNote)
        // Amendment 23: Typed controlled vocabulary properties
        Property {
            id: "https://uor.foundation/type/affectsAxis",
            label: "affectsAxis",
            comment: "The metric axis that this operation affects. Replaces \
                      the string-valued type:axisSignature property with a typed \
                      reference to MetricAxis individuals. Non-functional: an \
                      operation may affect multiple axes.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/op/Operation"),
            range: "https://uor.foundation/type/MetricAxis",
        },
        Property {
            id: "https://uor.foundation/type/crossingCost",
            label: "crossingCost",
            comment: "The cost of applying this constraint in terms of axis \
                      crossings: the number of metric boundaries that must be \
                      traversed.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/Constraint"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 25: Completeness Certification Pathway properties
        Property {
            id: "https://uor.foundation/type/completenessCandidate",
            label: "completenessCandidate",
            comment: "The ConstrainedType being evaluated for completeness \
                      by this CompletenessCandidate.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/CompletenessCandidate"),
            range: "https://uor.foundation/type/ConstrainedType",
        },
        Property {
            id: "https://uor.foundation/type/candidateNerve",
            label: "candidateNerve",
            comment: "The constraint nerve being computed for this candidate. \
                      The CompletenessResolver reads χ(N(C)) from this nerve \
                      at each iteration via resolver:nerveEulerCharacteristic.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/CompletenessCandidate"),
            range: "https://uor.foundation/resolver/CechNerve",
        },
        Property {
            id: "https://uor.foundation/type/witnessConstraint",
            label: "witnessConstraint",
            comment: "The constraint applied in this witness step.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/CompletenessWitness"),
            range: "https://uor.foundation/type/Constraint",
        },
        Property {
            id: "https://uor.foundation/type/sitesClosed",
            label: "sitesClosed",
            comment: "Number of sites closed by this witness step.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/CompletenessWitness"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 28: TypeSynthesisGoal properties
        Property {
            id: "https://uor.foundation/type/targetEulerCharacteristic",
            label: "targetEulerCharacteristic",
            comment: "The target χ(N(C)) value. For O(1) resolution: set equal to n (the \
                      quantum level).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/TypeSynthesisGoal"),
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/targetBettiNumber",
            label: "targetBettiNumber",
            comment: "Non-functional. Each assertion specifies a target Betti number value \
                      for a given homological degree. Multiple assertions permitted, one per \
                      degree.",
            kind: PropertyKind::Datatype,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/TypeSynthesisGoal"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 28: SynthesizedType property
        Property {
            id: "https://uor.foundation/type/synthesisResult",
            label: "synthesisResult",
            comment: "Links a SynthesizedType back to the synthesis run that produced it.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/SynthesizedType"),
            range: "https://uor.foundation/type/TypeSynthesisResult",
        },
        // Amendment 28: MinimalConstraintBasis properties
        Property {
            id: "https://uor.foundation/type/basisConstraint",
            label: "basisConstraint",
            comment: "Non-functional. One assertion per constraint in the minimal basis.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/MinimalConstraintBasis"),
            range: "https://uor.foundation/type/Constraint",
        },
        Property {
            id: "https://uor.foundation/type/basisSize",
            label: "basisSize",
            comment: "The cardinality of the minimal basis. The theoretical lower bound is \
                      n (one constraint per site).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/MinimalConstraintBasis"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 29: WittLift properties
        Property {
            id: "https://uor.foundation/type/liftBase",
            label: "liftBase",
            comment: "The base type being lifted to the next quantum level.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/WittLift"),
            range: "https://uor.foundation/type/ConstrainedType",
        },
        Property {
            id: "https://uor.foundation/type/liftTargetLevel",
            label: "liftTargetLevel",
            comment: "The quantum level this lift targets.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/WittLift"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/type/liftObstruction",
            label: "liftObstruction",
            comment: "The LiftObstruction for this lift. Trivial (zero class) iff the lift \
                      inherits completeness.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/WittLift"),
            range: "https://uor.foundation/type/LiftObstruction",
        },
        // Amendment 29: LiftObstruction properties
        Property {
            id: "https://uor.foundation/type/obstructionTrivial",
            label: "obstructionTrivial",
            comment: "True iff the obstruction class is zero — the base type's completeness \
                      transfers to the lifted quantum level without additional constraints.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/LiftObstruction"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/type/obstructionSite",
            label: "obstructionSite",
            comment: "The site at the new quantum level where the obstruction is located. \
                      Ranges over the new bit position introduced at Q_{n+1}.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/LiftObstruction"),
            range: "https://uor.foundation/partition/SiteIndex",
        },
        // Amendment 30: ConstrainedType extensions
        Property {
            id: "https://uor.foundation/type/holonomyGroup",
            label: "holonomyGroup",
            comment: "The HolonomyGroup of this type. Computed by the MonodromyResolver.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/ConstrainedType"),
            range: "https://uor.foundation/observable/HolonomyGroup",
        },
        Property {
            id: "https://uor.foundation/type/monodromyClass",
            label: "monodromyClass",
            comment: "The MonodromyClass classifying this type as flat or twisted.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/ConstrainedType"),
            range: "https://uor.foundation/observable/MonodromyClass",
        },
        // Amendment 32: Superposed site state amplitude (RC_5)
        Property {
            id: "https://uor.foundation/type/amplitude",
            label: "amplitude",
            comment: "The amplitude coefficient for this superposed site state.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/SuperposedSiteState"),
            range: XSD_DECIMAL,
        },
        // Amendment 34: Morphospace Achievability
        Property {
            id: "https://uor.foundation/type/targetForbidden",
            label: "targetForbidden",
            comment: "Whether the target signature of this TypeSynthesisGoal is a \
                      ForbiddenSignature. If true, synthesis is provably impossible.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/TypeSynthesisGoal"),
            range: XSD_BOOLEAN,
        },
        // Amendment 36: Measurement Boundary
        Property {
            id: "https://uor.foundation/type/collapsedFrom",
            label: "collapsedFrom",
            comment: "The SuperposedSiteState from which this CollapsedSiteState \
                      was produced by projective measurement.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/CollapsedSiteState"),
            range: "https://uor.foundation/type/SuperposedSiteState",
        },
        Property {
            id: "https://uor.foundation/type/survivingAmplitude",
            label: "survivingAmplitude",
            comment: "The amplitude of the surviving branch after projective \
                      collapse. |α|² is the probability of this outcome under \
                      the Born rule.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/CollapsedSiteState"),
            range: XSD_DECIMAL,
        },
        // Amendment 37: Amplitude normalization verification (Gap 1)
        Property {
            id: "https://uor.foundation/type/normalizationVerified",
            label: "normalizationVerified",
            comment: "Whether the amplitude vector of this SuperposedSiteState \
                      satisfies the normalization condition Σ|αᵢ|² = 1 (QM_5). \
                      Set by the SuperpositionResolver after verification.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/SuperposedSiteState"),
            range: XSD_BOOLEAN,
        },
        // Amendment 37: Holonomy classification flag (Gap 4)
        Property {
            id: "https://uor.foundation/type/holonomyClassified",
            label: "holonomyClassified",
            comment: "Whether this ConstrainedType has been classified as FlatType \
                      or TwistedType by the MonodromyResolver. The MN_8 identity \
                      guarantees this is a bivalent classification: \
                      holonomyClassified(T) iff isFlatType(T) xor isTwistedType(T).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/ConstrainedType"),
            range: XSD_BOOLEAN,
        },
        // Amendment 41: LiftChain properties
        Property {
            id: "https://uor.foundation/type/liftSourceLevel",
            label: "liftSourceLevel",
            comment: "The quantum level at the base of the chain.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/LiftChain"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/type/chainLength",
            label: "chainLength",
            comment: "The number of WittLift steps in the chain (k - j).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/LiftChain"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/chainStep",
            label: "chainStep",
            comment: "A WittLift step in this chain. Non-functional: one per step.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/LiftChain"),
            range: "https://uor.foundation/type/WittLift",
        },
        Property {
            id: "https://uor.foundation/type/chainObstructionProfile",
            label: "chainObstructionProfile",
            comment: "The full obstruction history of this chain.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/LiftChain"),
            range: "https://uor.foundation/type/ObstructionChain",
        },
        Property {
            id: "https://uor.foundation/type/resolvedBasisSize",
            label: "resolvedBasisSize",
            comment: "The basis size of the CompleteType at the chain target level.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/LiftChain"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 41: ObstructionChain properties
        Property {
            id: "https://uor.foundation/type/obstructionAt",
            label: "obstructionAt",
            comment: "A non-trivial LiftObstruction in this chain. Non-functional.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/ObstructionChain"),
            range: "https://uor.foundation/type/LiftObstruction",
        },
        Property {
            id: "https://uor.foundation/type/obstructionCount",
            label: "obstructionCount",
            comment: "Total number of non-trivial LiftObstruction records.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/ObstructionChain"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/isFlat",
            label: "isFlat",
            comment: "True iff obstructionCount = 0 (flat tower).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/ObstructionChain"),
            range: XSD_BOOLEAN,
        },
        // Amendment 56: ModuliSpace properties
        Property {
            id: "https://uor.foundation/type/moduliWittLevel",
            label: "moduliWittLevel",
            comment: "The quantum level at which this moduli space is defined.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/ModuliSpace"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/type/moduliPoint",
            label: "moduliPoint",
            comment: "A CompleteType that is a point of this moduli space.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/ModuliSpace"),
            range: "https://uor.foundation/type/CompleteType",
        },
        Property {
            id: "https://uor.foundation/type/moduliDimension",
            label: "moduliDimension",
            comment: "The dimension of this moduli space.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/ModuliSpace"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 56: HolonomyStratum properties
        Property {
            id: "https://uor.foundation/type/stratumHolonomyClass",
            label: "stratumHolonomyClass",
            comment: "The MonodromyClass indexing this stratum.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/HolonomyStratum"),
            range: "https://uor.foundation/observable/MonodromyClass",
        },
        Property {
            id: "https://uor.foundation/type/stratumCodimension",
            label: "stratumCodimension",
            comment: "The codimension of this stratum within the moduli space.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/HolonomyStratum"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/stratumModuli",
            label: "stratumModuli",
            comment: "The moduli space containing this stratum.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/HolonomyStratum"),
            range: "https://uor.foundation/type/ModuliSpace",
        },
        // Amendment 56: DeformationFamily properties
        Property {
            id: "https://uor.foundation/type/familyParameter",
            label: "familyParameter",
            comment: "A CompleteType along the deformation path.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/DeformationFamily"),
            range: "https://uor.foundation/type/CompleteType",
        },
        Property {
            id: "https://uor.foundation/type/familyPreservesCompleteness",
            label: "familyPreservesCompleteness",
            comment: "Whether every member of this deformation family remains a \
                      CompleteType.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/DeformationFamily"),
            range: XSD_BOOLEAN,
        },
        // Amendment 56: VersalDeformation properties
        Property {
            id: "https://uor.foundation/type/versalBase",
            label: "versalBase",
            comment: "The CompleteType at the base of this versal deformation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/VersalDeformation"),
            range: "https://uor.foundation/type/CompleteType",
        },
        Property {
            id: "https://uor.foundation/type/versalDimension",
            label: "versalDimension",
            comment: "The dimension of the versal deformation space.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/VersalDeformation"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 56: ModuliTowerMap property
        Property {
            id: "https://uor.foundation/type/towerMapSource",
            label: "towerMapSource",
            comment: "The source moduli space of this tower map.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/ModuliTowerMap"),
            range: "https://uor.foundation/type/ModuliSpace",
        },
        // Amendment 60: GaloisConnection properties
        Property {
            id: "https://uor.foundation/type/upperAdjoint",
            label: "upperAdjoint",
            comment: "The upper adjoint (type closure) of the Galois connection, \
                      expressed as a symbolic formula string.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/GaloisConnection"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/type/lowerAdjoint",
            label: "lowerAdjoint",
            comment: "The lower adjoint (site interior) of the Galois connection, \
                      expressed as a symbolic formula string.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/GaloisConnection"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/type/fixpointCondition",
            label: "fixpointCondition",
            comment: "The fixpoint condition for the Galois connection: when \
                      upper(lower(T)) = T, the type is complete.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/GaloisConnection"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/type/refinementDirection",
            label: "refinementDirection",
            comment: "Description of the refinement direction: ascending in type \
                      lattice corresponds to descending in site freedom.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/GaloisConnection"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 61: Structural Type annotation properties
        Property {
            id: "https://uor.foundation/type/structuralSiteCount",
            label: "structuralSiteCount",
            comment: "Formula or description of the site count for a structural \
                      type individual.",
            kind: PropertyKind::Annotation,
            functional: true,
            required: false,
            domain: None,
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/type/structuralGrounding",
            label: "structuralGrounding",
            comment: "Description of the grounding rule that maps values of this \
                      structural type onto the UOR ring.",
            kind: PropertyKind::Annotation,
            functional: true,
            required: false,
            domain: None,
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/type/structuralConstraint",
            label: "structuralConstraint",
            comment: "Description of the constraint characterizing this structural \
                      type, if applicable.",
            kind: PropertyKind::Annotation,
            functional: true,
            required: false,
            domain: None,
            range: XSD_STRING,
        },
        // Amendment 71: Missing type properties (14)
        Property {
            id: "https://uor.foundation/type/galoisClosureProperty",
            label: "galoisClosureProperty",
            comment: "The closure property of a Galois connection.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/GaloisConnection"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/type/galoisInteriorProperty",
            label: "galoisInteriorProperty",
            comment: "The interior property of a Galois connection.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/GaloisConnection"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/type/groundingMapRef",
            label: "groundingMapRef",
            comment: "Reference to the grounding map used for this type.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: None,
            range: "https://uor.foundation/morphism/GroundingMap",
        },
        Property {
            id: "https://uor.foundation/type/compositionLaw",
            label: "compositionLaw",
            comment: "The algebraic composition law governing this type.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: None,
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/type/siteOrderingConstraint",
            label: "siteOrderingConstraint",
            comment: "Constraint on the ordering of sites for this type.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: None,
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/type/backboneThreshold",
            label: "backboneThreshold",
            comment: "The threshold for backbone inclusion in the type lattice.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: None,
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/permutationGroup",
            label: "permutationGroup",
            comment: "The permutation group acting on sites of this type.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: None,
            range: "https://uor.foundation/op/Group",
        },
        Property {
            id: "https://uor.foundation/type/acyclicityWitness",
            label: "acyclicityWitness",
            comment: "Witness certifying acyclicity of the type dependency graph.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: None,
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/type/connectivityWitness",
            label: "connectivityWitness",
            comment: "Witness certifying connectivity of the type graph.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: None,
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/type/schemaConstraint",
            label: "schemaConstraint",
            comment: "Constraint imposed by the schema on this type.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: None,
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/type/alphabetSize",
            label: "alphabetSize",
            comment: "The size of the alphabet for symbol-based types.",
            kind: PropertyKind::Annotation,
            functional: true,
            required: false,
            domain: None,
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/type/quantizationRange",
            label: "quantizationRange",
            comment: "The range specification for quantized types.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: None,
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/type/elementTypeRef",
            label: "elementTypeRef",
            comment: "Reference to the element type within a composite type.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: None,
            range: "https://uor.foundation/type/TypeDefinition",
        },
        // Amendment 77: Subtyping and Variance properties
        Property {
            id: "https://uor.foundation/type/inclusionSource",
            label: "inclusionSource",
            comment: "The subtype (more constraints).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/TypeInclusion"),
            range: "https://uor.foundation/type/ConstrainedType",
        },
        Property {
            id: "https://uor.foundation/type/inclusionTarget",
            label: "inclusionTarget",
            comment: "The supertype (fewer constraints).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/TypeInclusion"),
            range: "https://uor.foundation/type/ConstrainedType",
        },
        Property {
            id: "https://uor.foundation/type/positionVariance",
            label: "positionVariance",
            comment: "The variance of each operand position.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            // Cross-namespace domain: operad:StructuralOperad
            domain: Some("https://uor.foundation/operad/StructuralOperad"),
            range: "https://uor.foundation/type/VarianceAnnotation",
        },
        Property {
            id: "https://uor.foundation/type/inclusionWitness",
            label: "inclusionWitness",
            comment: "True iff constraints(source) \u{2287} constraints(target). \
                      Computed, not asserted.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/TypeInclusion"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/type/latticeDepth",
            label: "latticeDepth",
            comment: "Maximum chain length from PrimitiveType (top) to any \
                      CompleteType (bottom). Equals the site budget n at \
                      quantum level Q_k.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/SubtypingLattice"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // v0.2.1 Phase 7a.6: ConstraintDefaults single-value slot.
        Property {
            id: "https://uor.foundation/type/defaultValue",
            label: "defaultValue",
            comment: "Default integer value for the named constraint default.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/ConstraintDefaults"),
            range: XSD_INTEGER,
        },
        // v0.2.2 Phase D (Q4) — parametric constraint surface.
        Property {
            id: "https://uor.foundation/type/boundObservable",
            label: "boundObservable",
            comment: "The observable:Observable whose values this BoundConstraint \
                      bounds. Picks which datum projection is being constrained.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: "https://uor.foundation/observable/Observable",
        },
        Property {
            id: "https://uor.foundation/type/boundShape",
            label: "boundShape",
            comment: "The predicate form this BoundConstraint imposes on its \
                      bound observable. Closed enumeration with exactly six \
                      named individuals (EqualBound, LessEqBound, \
                      GreaterEqBound, RangeContainBound, ResidueClassBound, \
                      AffineEqualBound).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: "https://uor.foundation/type/BoundShape",
        },
        Property {
            id: "https://uor.foundation/type/boundArguments",
            label: "boundArguments",
            comment: "The BoundConstraint's parameters in canonical string form \
                      (e.g., 'modulus=256;residue=255' for a residue class bound). \
                      Non-functional — the string-form encoding is the canonical \
                      serialization; typed accessors on the Rust side unpack it \
                      via the per-type-alias constructors.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/type/BoundConstraint"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/type/conjuncts",
            label: "conjuncts",
            comment: "An ordered list of BoundConstraint individuals that this \
                      Conjunction composes. Non-functional — multiple conjuncts \
                      span the conjunction.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/type/Conjunction"),
            range: "https://uor.foundation/type/BoundConstraint",
        },
    ]
}

fn individuals() -> Vec<Individual> {
    vec![
        Individual {
            id: "https://uor.foundation/type/verticalAxis",
            type_: "https://uor.foundation/type/MetricAxis",
            label: "verticalAxis",
            comment: "The vertical (ring/additive) metric axis. Constraints on \
                      this axis operate through ring arithmetic: residue classes, \
                      divisibility, and additive structure.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/type/horizontalAxis",
            type_: "https://uor.foundation/type/MetricAxis",
            label: "horizontalAxis",
            comment: "The horizontal (Hamming/bitwise) metric axis. Constraints on \
                      this axis operate through bitwise structure: carry patterns, \
                      bit positions, and Hamming distance.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/type/diagonalAxis",
            type_: "https://uor.foundation/type/MetricAxis",
            label: "diagonalAxis",
            comment: "The diagonal (incompatibility) metric axis. Constraints on \
                      this axis measure the gap between ring and Hamming metrics \u{2014} \
                      the curvature of UOR geometry.",
            properties: &[],
        },
        // ── Amendment 61: Structural Type individuals ────────────────────
        Individual {
            id: "https://uor.foundation/type/ScalarType",
            type_: "https://uor.foundation/type/PrimitiveType",
            label: "ScalarType",
            comment: "A single value from an ordered domain. siteCount = n \
                      (quantization bits).",
            properties: &[
                (
                    "https://uor.foundation/type/structuralSiteCount",
                    IndividualValue::Str("n (quantization bits)"),
                ),
                (
                    "https://uor.foundation/type/structuralGrounding",
                    IndividualValue::Str("quantize(value, range, bits) produces ring element where d_R reflects value proximity"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/SymbolType",
            type_: "https://uor.foundation/type/PrimitiveType",
            label: "SymbolType",
            comment: "A value from a finite unordered set. siteCount = \
                      ceil(log2(|alphabet|)).",
            properties: &[
                (
                    "https://uor.foundation/type/structuralSiteCount",
                    IndividualValue::Str("ceil(log2(|alphabet|))"),
                ),
                (
                    "https://uor.foundation/type/structuralGrounding",
                    IndividualValue::Str("argmin_{encoding} sum d_delta over observed pairs (CY_5)"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/SequenceType",
            type_: "https://uor.foundation/type/ProductType",
            label: "SequenceType",
            comment: "An ordered list of elements with backbone constraint. The \
                      free monoid on the element type.",
            properties: &[
                (
                    "https://uor.foundation/type/structuralSiteCount",
                    IndividualValue::Str("sum of element site counts"),
                ),
                (
                    "https://uor.foundation/type/structuralGrounding",
                    IndividualValue::Str("free monoid on element type, backbone constraint"),
                ),
                (
                    "https://uor.foundation/type/structuralConstraint",
                    IndividualValue::Str("backbone ordering: elements indexed by position"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/TupleType",
            type_: "https://uor.foundation/type/ProductType",
            label: "TupleType",
            comment: "A fixed collection of named fields.",
            properties: &[
                (
                    "https://uor.foundation/type/structuralSiteCount",
                    IndividualValue::Str("sum of field site counts"),
                ),
                (
                    "https://uor.foundation/type/structuralGrounding",
                    IndividualValue::Str("site ordering follows CY_6 (optimal site ordering)"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/GraphType",
            type_: "https://uor.foundation/type/ConstrainedType",
            label: "GraphType",
            comment: "Nodes with edge constraints. Constraint nerve = graph topology.",
            properties: &[
                (
                    "https://uor.foundation/type/structuralSiteCount",
                    IndividualValue::Str("sum of node site counts + edge overhead"),
                ),
                (
                    "https://uor.foundation/type/structuralGrounding",
                    IndividualValue::Str("constraint nerve = graph nerve, beta_k equality"),
                ),
                (
                    "https://uor.foundation/type/structuralConstraint",
                    IndividualValue::Str("edge constraints: adjacency preserved under grounding"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/SetType",
            type_: "https://uor.foundation/type/ConstrainedType",
            label: "SetType",
            comment: "Unordered collection. d_delta is permutation-invariant.",
            properties: &[
                (
                    "https://uor.foundation/type/structuralSiteCount",
                    IndividualValue::Str("sum of element site counts"),
                ),
                (
                    "https://uor.foundation/type/structuralGrounding",
                    IndividualValue::Str("d_delta invariant under element permutation, D_{2n} symmetry"),
                ),
                (
                    "https://uor.foundation/type/structuralConstraint",
                    IndividualValue::Str("permutation invariance: encoding is order-independent"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/TreeType",
            type_: "https://uor.foundation/type/ConstrainedType",
            label: "TreeType",
            comment: "Hierarchical structure. beta_1=0 (acyclic), beta_0=1 (connected).",
            properties: &[
                (
                    "https://uor.foundation/type/structuralSiteCount",
                    IndividualValue::Str("sum of node site counts"),
                ),
                (
                    "https://uor.foundation/type/structuralGrounding",
                    IndividualValue::Str("parent-child encoding with acyclicity constraint"),
                ),
                (
                    "https://uor.foundation/type/structuralConstraint",
                    IndividualValue::Str("beta_1=0 (acyclic), beta_0=1 (connected)"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/TableType",
            type_: "https://uor.foundation/type/ProductType",
            label: "TableType",
            comment: "Collection of tuples sharing a schema = Sequence(Tuple(S)). \
                      Functorial decomposition.",
            properties: &[
                (
                    "https://uor.foundation/type/structuralSiteCount",
                    IndividualValue::Str("row_count * tuple_site_count"),
                ),
                (
                    "https://uor.foundation/type/structuralGrounding",
                    IndividualValue::Str("Sequence(Tuple(S)), functorial decomposition"),
                ),
                (
                    "https://uor.foundation/type/structuralConstraint",
                    IndividualValue::Str("shared schema: all rows conform to tuple type S"),
                ),
            ],
        },
        // Amendment 77: VarianceAnnotation individuals
        Individual {
            id: "https://uor.foundation/type/Covariant",
            type_: "https://uor.foundation/type/VarianceAnnotation",
            label: "Covariant",
            comment: "The structural position preserves TypeInclusion: \
                      if T\u{2081} \u{2264} T\u{2082}, then \
                      F(T\u{2081}) \u{2264} F(T\u{2082}).",
            properties: &[(
                "https://uor.foundation/op/enumVariant",
                IndividualValue::IriRef("https://uor.foundation/type/Covariant"),
            )],
        },
        Individual {
            id: "https://uor.foundation/type/Contravariant",
            type_: "https://uor.foundation/type/VarianceAnnotation",
            label: "Contravariant",
            comment: "The structural position reverses TypeInclusion: \
                      if T\u{2081} \u{2264} T\u{2082}, then \
                      F(T\u{2082}) \u{2264} F(T\u{2081}).",
            properties: &[(
                "https://uor.foundation/op/enumVariant",
                IndividualValue::IriRef("https://uor.foundation/type/Contravariant"),
            )],
        },
        Individual {
            id: "https://uor.foundation/type/Invariant",
            type_: "https://uor.foundation/type/VarianceAnnotation",
            label: "Invariant",
            comment: "The structural position requires exact type equality: \
                      F(T\u{2081}) \u{2264} F(T\u{2082}) only if \
                      T\u{2081} = T\u{2082}.",
            properties: &[(
                "https://uor.foundation/op/enumVariant",
                IndividualValue::IriRef("https://uor.foundation/type/Invariant"),
            )],
        },
        Individual {
            id: "https://uor.foundation/type/Bivariant",
            type_: "https://uor.foundation/type/VarianceAnnotation",
            label: "Bivariant",
            comment: "The structural position ignores the type parameter: \
                      F(T\u{2081}) \u{2264} F(T\u{2082}) for all \
                      T\u{2081}, T\u{2082}.",
            properties: &[(
                "https://uor.foundation/op/enumVariant",
                IndividualValue::IriRef("https://uor.foundation/type/Bivariant"),
            )],
        },
        // Amendment 95: TypeDefinition individual coverage (Workstream 8)
        Individual {
            id: "https://uor.foundation/type/EitherType",
            type_: "https://uor.foundation/type/SumType",
            label: "EitherType",
            comment: "The canonical binary disjoint union type whose carrier is \
                      L + R.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/type/OptionType",
            type_: "https://uor.foundation/type/SumType",
            label: "OptionType",
            comment: "The canonical A + Unit idiom for optional values.",
            properties: &[],
        },
        // v0.2.1 Phase 7a.6: ConstraintDefaults individual. Macros crate
        // build.rs reads this to bake RESIDUE_DEFAULT_MODULUS into the
        // derive macro's expansion, so no literal 256 appears in Rust src.
        Individual {
            id: "https://uor.foundation/type/ResidueDefaultModulus",
            type_: "https://uor.foundation/type/ConstraintDefaults",
            label: "ResidueDefaultModulus",
            comment: "Default modulus for ResidueConstraint when the \
                      #[uor(residue = X)] attribute omits the explicit \
                      modulus. 256 matches the v0.2.1 reference ring Z/(2^8)Z.",
            properties: &[(
                "https://uor.foundation/type/defaultValue",
                IndividualValue::Int(256),
            )],
        },
        // ─────────────────────────────────────────────────────────────────
        // v0.2.2 Phase D — BoundShape individuals (closed catalogue of 6).
        // ─────────────────────────────────────────────────────────────────
        Individual {
            id: "https://uor.foundation/type/EqualBound",
            type_: "https://uor.foundation/type/BoundShape",
            label: "EqualBound",
            comment: "Predicate form: `observable(datum) == target`. Used by \
                      BoundConstraint instances asserting strict equality of a \
                      datum's observable projection to a target value.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/type/LessEqBound",
            type_: "https://uor.foundation/type/BoundShape",
            label: "LessEqBound",
            comment: "Predicate form: `observable(datum) <= bound`. Used by \
                      hamming, depth, carry, and site-rank bound constraints.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/type/GreaterEqBound",
            type_: "https://uor.foundation/type/BoundShape",
            label: "GreaterEqBound",
            comment: "Predicate form: `observable(datum) >= bound`.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/type/RangeContainBound",
            type_: "https://uor.foundation/type/BoundShape",
            label: "RangeContainBound",
            comment: "Predicate form: `lo <= observable(datum) <= hi`. The \
                      datum's observable projection must lie within the \
                      inclusive range `[lo, hi]`.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/type/ResidueClassBound",
            type_: "https://uor.foundation/type/BoundShape",
            label: "ResidueClassBound",
            comment: "Predicate form: `observable(datum) ≡ residue (mod modulus)`. \
                      Used by BoundConstraint instances asserting residue-class \
                      membership.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/type/AffineEqualBound",
            type_: "https://uor.foundation/type/BoundShape",
            label: "AffineEqualBound",
            comment: "Predicate form: `observable(datum) == offset + Σ αᵢ·generatorᵢ`. \
                      The datum's observable projection must equal an affine \
                      combination of the BoundConstraint's affine generators.",
            properties: &[],
        },
        // Wiki ADR-040: 7th BoundShape individual — the byte-sequence-valued
        // comparison primitive. Catalog declaration alone determines the
        // comparison semantics; resolvers do not dispatch on observable type.
        Individual {
            id: "https://uor.foundation/type/LexicographicLessEqBound",
            type_: "https://uor.foundation/type/BoundShape",
            label: "LexicographicLessEqBound",
            comment: "Predicate form: `observable(datum) <= bound` over \
                      byte-sequence-valued observables under canonical \
                      big-endian unsigned ordering. Fixed-length: lexicographic \
                      byte-wise comparison. Variable-length: lexicographic with \
                      shorter-prefix-rule (the shorter sequence is less than the \
                      longer when bytes match through the shorter's length). \
                      Wiki ADR-040 + ADR-038: this is the BoundShape primitive \
                      for byte-sequence-valued AxisProjectionObservable instances \
                      (e.g. axis-realized hash projections) — LessEqBound is the \
                      integer-valued primitive (HammingMetric, \
                      DerivationDepthObservable, etc.).",
            properties: &[],
        },
        // ─────────────────────────────────────────────────────────────────
        // v0.2.2 Phase D — BoundConstraint kind individuals (6 catalogued
        // (observable, shape) pairs replacing the v0.2.1 enumerated
        // subclasses).
        // ─────────────────────────────────────────────────────────────────
        Individual {
            id: "https://uor.foundation/type/residueConstraintKind",
            type_: "https://uor.foundation/type/BoundConstraint",
            label: "residueConstraintKind",
            comment: "Parametric replacement for the v0.2.1 ResidueConstraint \
                      class: (observable:ValueModObservable, \
                      type:ResidueClassBound). The Rust foundation exposes \
                      this kind via the `ResidueConstraint` type alias with \
                      `pub const fn new(modulus, residue)`.",
            properties: &[
                (
                    "https://uor.foundation/type/boundObservable",
                    IndividualValue::IriRef(
                        "https://uor.foundation/observable/ValueModObservable",
                    ),
                ),
                (
                    "https://uor.foundation/type/boundShape",
                    IndividualValue::IriRef("https://uor.foundation/type/ResidueClassBound"),
                ),
                (
                    "https://uor.foundation/type/boundArguments",
                    IndividualValue::Str("modulus=256;residue=0"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/hammingConstraintKind",
            type_: "https://uor.foundation/type/BoundConstraint",
            label: "hammingConstraintKind",
            comment: "Parametric replacement for the v0.2.1 HammingConstraint \
                      class: (observable:HammingMetric, type:LessEqBound). The \
                      Rust foundation exposes this kind via the \
                      `HammingConstraint` type alias with \
                      `pub const fn new(bound)`.",
            properties: &[
                (
                    "https://uor.foundation/type/boundObservable",
                    IndividualValue::IriRef("https://uor.foundation/observable/HammingMetric"),
                ),
                (
                    "https://uor.foundation/type/boundShape",
                    IndividualValue::IriRef("https://uor.foundation/type/LessEqBound"),
                ),
                (
                    "https://uor.foundation/type/boundArguments",
                    IndividualValue::Str("bound=0"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/depthConstraintKind",
            type_: "https://uor.foundation/type/BoundConstraint",
            label: "depthConstraintKind",
            comment: "Parametric replacement for the v0.2.1 DepthConstraint \
                      class: (derivation:DerivationDepthObservable, \
                      type:LessEqBound).",
            properties: &[
                (
                    "https://uor.foundation/type/boundObservable",
                    IndividualValue::IriRef(
                        "https://uor.foundation/derivation/DerivationDepthObservable",
                    ),
                ),
                (
                    "https://uor.foundation/type/boundShape",
                    IndividualValue::IriRef("https://uor.foundation/type/LessEqBound"),
                ),
                (
                    "https://uor.foundation/type/boundArguments",
                    IndividualValue::Str("min_depth=0;max_depth=0"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/carryConstraintKind",
            type_: "https://uor.foundation/type/BoundConstraint",
            label: "carryConstraintKind",
            comment: "Parametric replacement for the v0.2.1 CarryConstraint \
                      class: (carry:CarryDepthObservable, type:LessEqBound).",
            properties: &[
                (
                    "https://uor.foundation/type/boundObservable",
                    IndividualValue::IriRef("https://uor.foundation/carry/CarryDepthObservable"),
                ),
                (
                    "https://uor.foundation/type/boundShape",
                    IndividualValue::IriRef("https://uor.foundation/type/LessEqBound"),
                ),
                (
                    "https://uor.foundation/type/boundArguments",
                    IndividualValue::Str("bound=0"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/siteConstraintKind",
            type_: "https://uor.foundation/type/BoundConstraint",
            label: "siteConstraintKind",
            comment: "Parametric replacement for the v0.2.1 SiteConstraint \
                      class: (partition:FreeRankObservable, type:LessEqBound).",
            properties: &[
                (
                    "https://uor.foundation/type/boundObservable",
                    IndividualValue::IriRef(
                        "https://uor.foundation/partition/FreeRankObservable",
                    ),
                ),
                (
                    "https://uor.foundation/type/boundShape",
                    IndividualValue::IriRef("https://uor.foundation/type/LessEqBound"),
                ),
                (
                    "https://uor.foundation/type/boundArguments",
                    IndividualValue::Str("site_index=0"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/type/affineConstraintKind",
            type_: "https://uor.foundation/type/BoundConstraint",
            label: "affineConstraintKind",
            comment: "Parametric replacement for the v0.2.1 AffineConstraint \
                      class: (observable:ValueModObservable, \
                      type:AffineEqualBound).",
            properties: &[
                (
                    "https://uor.foundation/type/boundObservable",
                    IndividualValue::IriRef(
                        "https://uor.foundation/observable/ValueModObservable",
                    ),
                ),
                (
                    "https://uor.foundation/type/boundShape",
                    IndividualValue::IriRef("https://uor.foundation/type/AffineEqualBound"),
                ),
                (
                    "https://uor.foundation/type/boundArguments",
                    IndividualValue::Str("offset=0"),
                ),
            ],
        },
    ]
}
