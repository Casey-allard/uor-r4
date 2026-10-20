//! `morphism/` namespace — Transforms, morphisms, and the Surface Grounding Layer
//! (Amendments 6, 12, 24).
//!
//! The morphism namespace defines the abstractions for maps between UOR objects.
//! These are the things that `cert/` certificates attest to, that `trace/`
//! traces record the execution of, and that `state/` transitions capture the
//! cumulative effect of.
//!
//! Amendment 12 adds the composition primitive: the categorical backbone that
//! turns transforms into a category with identity and associative composition.
//!
//! Amendment 24 adds the **Surface Grounding Layer**: `GroundingMap` (surface symbol
//! to ring address), `ProjectionMap` (ring address neighbourhood to surface symbol
//! sequence), and `GroundingCertificate` (round-trip attestation). Together these
//! form the universal I/O boundary through which any typed surface representation
//! enters or exits the ring address space.
//!
//! **Space classification:** `user` — transforms are instantiated by applications.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `morphism/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "morphism",
            iri: NS_MORPHISM,
            label: "UOR Transforms and Morphisms",
            comment: "Runtime abstractions for maps between UOR objects: transforms, \
                      isometries, embeddings, and group actions. The foundation \
                      provides the vocabulary; Prism writes the sentences.",
            space: Space::User,
            imports: &[
                NS_SCHEMA,
                NS_TYPE,
                NS_OP,
                NS_OBSERVABLE,
                NS_PARTITION,
                NS_TRACE,
                NS_HOMOLOGY,
                NS_U,
                NS_DERIVATION,
                NS_CERT,
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
            id: "https://uor.foundation/morphism/Transform",
            label: "Transform",
            comment: "A map between UOR objects. The root abstraction: source, target, \
                      and optionally what structure (if any) is preserved. This is \
                      what cert:TransformCertificate certifies.",
            subclass_of: &[OWL_THING],
            disjoint_with: &["https://uor.foundation/morphism/CompositionLaw"],
        },
        Class {
            id: "https://uor.foundation/morphism/Isometry",
            label: "Isometry",
            comment: "A transform that preserves metric structure with respect to a \
                      specified metric. In UOR, isometry is metric-relative: neg is a \
                      ring isometry, bnot is a Hamming isometry. A transform can be \
                      an isometry with respect to one metric but not the other. This \
                      is what cert:IsometryCertificate certifies.",
            subclass_of: &["https://uor.foundation/morphism/Transform"],
            disjoint_with: &["https://uor.foundation/morphism/Composition"],
        },
        Class {
            id: "https://uor.foundation/morphism/Embedding",
            label: "Embedding",
            comment: "An injective, structure-preserving transform across quantum \
                      levels. The canonical instance is the level embedding \
                      ι : R_n → R_{n'} (n < n'), which preserves addition, \
                      multiplication, and content addressing.",
            subclass_of: &["https://uor.foundation/morphism/Transform"],
            disjoint_with: &["https://uor.foundation/morphism/Composition"],
        },
        Class {
            id: "https://uor.foundation/morphism/Action",
            label: "Action",
            comment: "The mechanism by which a group applies transforms systematically \
                      to a set. Each group element induces a transform of the set. \
                      The dihedral action on type space is an action by isometries — \
                      every element of D_{2^n} produces an isometric transform of 𝒯_n.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 12: Composition Primitive classes
        Class {
            id: "https://uor.foundation/morphism/Composition",
            label: "Composition",
            comment: "A transform formed by composing two or more transforms \
                      sequentially. The categorical composition operation that \
                      turns transforms into a category.",
            subclass_of: &["https://uor.foundation/morphism/Transform"],
            disjoint_with: &[
                "https://uor.foundation/morphism/Isometry",
                "https://uor.foundation/morphism/Embedding",
                "https://uor.foundation/morphism/Identity",
            ],
        },
        Class {
            id: "https://uor.foundation/morphism/Identity",
            label: "Identity",
            comment: "The identity transform on a type: maps every element to itself. \
                      The categorical identity morphism.",
            subclass_of: &["https://uor.foundation/morphism/Transform"],
            disjoint_with: &["https://uor.foundation/morphism/Composition"],
        },
        Class {
            id: "https://uor.foundation/morphism/CompositionLaw",
            label: "CompositionLaw",
            comment: "A law governing how operations compose. Records whether the \
                      composition is associative, commutative, and what the result \
                      operation is. The critical composition law (neg ∘ bnot = succ) \
                      is the foundational instance.",
            subclass_of: &[OWL_THING],
            disjoint_with: &["https://uor.foundation/morphism/Transform"],
        },
        // Amendment 24: Surface Grounding Layer
        Class {
            id: "https://uor.foundation/morphism/GroundingMap",
            label: "GroundingMap",
            comment: "A Transform mapping a surface symbol (any schema:Literal) to its ring \
                      datum (a schema:Datum) via the content-addressing bijection. Typed, \
                      derivation-witnessed, constraint-preserving map from surface to coordinate. \
                      Applies identically across NLP tokens, ARC-AGI grid cells, MIDI notes, \
                      pixels, sensor readings, and logical propositions.",
            subclass_of: &["https://uor.foundation/morphism/Transform"],
            disjoint_with: &["https://uor.foundation/morphism/ProjectionMap"],
        },
        Class {
            id: "https://uor.foundation/morphism/ProjectionMap",
            label: "ProjectionMap",
            comment: "A Transform mapping a resolved partition:Partition (address neighbourhood) \
                      to an ordered sequence of surface symbols. Output ordering determined by \
                      the active state:Frame — the same constraint frame that decomposed the \
                      input symbol sequence.",
            subclass_of: &["https://uor.foundation/morphism/Transform"],
            disjoint_with: &["https://uor.foundation/morphism/GroundingMap"],
        },
        // Gap G: GroundingCertificate
        Class {
            id: "https://uor.foundation/morphism/GroundingCertificate",
            label: "GroundingCertificate",
            comment: "A certificate attesting that a specific grounding round-trip (P∘Π∘G) \
                      satisfied the shared-frame condition and landed in the type-equivalent \
                      neighbourhood of the source symbol. Witnesses the Surface Symmetry Theorem \
                      (op:surfaceSymmetry) for one specific symbol instance.",
            subclass_of: &["https://uor.foundation/cert/Certificate"],
            disjoint_with: &[],
        },
        // Amendment 22: TopologicalDelta
        Class {
            id: "https://uor.foundation/morphism/TopologicalDelta",
            label: "TopologicalDelta",
            comment:
                "A topological delta: records changes in topological invariants (Betti numbers, \
                      Euler characteristic, nerve structure) before and after a morphism.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 75: Higher-Order Computation classes
        Class {
            id: "https://uor.foundation/morphism/ComputationDatum",
            label: "ComputationDatum",
            comment: "A datum whose ring value is the content address of a \
                      cert:TransformCertificate. Represents a certified \
                      computation as a first-class value within the ring.",
            subclass_of: &["https://uor.foundation/schema/Datum"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/morphism/ApplicationMorphism",
            label: "ApplicationMorphism",
            comment: "A transform that applies a ComputationDatum to an \
                      input datum, producing an output datum. The output \
                      inherits the certificate of the ComputationDatum.",
            subclass_of: &["https://uor.foundation/morphism/Transform"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/morphism/PartialApplication",
            label: "PartialApplication",
            comment: "A ComputationDatum formed by fixing some but not all \
                      inputs of a multi-argument transform.",
            subclass_of: &["https://uor.foundation/morphism/ComputationDatum"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/morphism/TransformComposition",
            label: "TransformComposition",
            comment: "A ComputationDatum representing the composition f \
                      \u{2218} g of two ComputationDatums. Certified iff \
                      both components are certified and range(g) = domain(f).",
            subclass_of: &["https://uor.foundation/morphism/ComputationDatum"],
            disjoint_with: &[],
        },
        // Amendment 95: Boundary map registry (Workstream 4)
        Class {
            id: "https://uor.foundation/morphism/Witness",
            label: "Witness",
            comment: "Abstract supertype for one specific input/output pair \
                      witnessing a Transform.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/morphism/GroundingWitness",
            label: "GroundingWitness",
            comment: "One specific surface symbol mapped to one specific \
                      grounded address by some GroundingMap.",
            subclass_of: &["https://uor.foundation/morphism/Witness"],
            disjoint_with: &["https://uor.foundation/morphism/ProjectionWitness"],
        },
        Class {
            id: "https://uor.foundation/morphism/ProjectionWitness",
            label: "ProjectionWitness",
            comment: "One specific input partition mapped to one specific \
                      surface symbol sequence by some ProjectionMap.",
            subclass_of: &["https://uor.foundation/morphism/Witness"],
            disjoint_with: &["https://uor.foundation/morphism/GroundingWitness"],
        },
        Class {
            id: "https://uor.foundation/morphism/SymbolSequence",
            label: "SymbolSequence",
            comment: "An ordered sequence of surface symbols. Elements are \
                      reified as SequenceElement individuals with explicit \
                      position indices.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/morphism/SequenceElement",
            label: "SequenceElement",
            comment: "One position in a SymbolSequence: a reified \
                      (value, index) pair.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/morphism/source",
            label: "source",
            comment: "The domain of the transform.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Transform"),
            range: OWL_THING,
        },
        Property {
            id: "https://uor.foundation/morphism/target",
            label: "target",
            comment: "The codomain of the transform.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Transform"),
            range: OWL_THING,
        },
        Property {
            id: "https://uor.foundation/morphism/preserves",
            label: "preserves",
            comment: "The structure preserved by this transform (if any). \
                      E.g., a ring homomorphism preserves addition and multiplication.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/morphism/Transform"),
            range: OWL_THING,
        },
        Property {
            id: "https://uor.foundation/morphism/preservesMetric",
            label: "preservesMetric",
            comment: "The specific metric this isometry preserves. Points to \
                      observable:RingMetric or observable:HammingMetric. A transform \
                      that preserves both is an isometry of the full UOR geometry. \
                      A transform that preserves one but not the other has nontrivial \
                      curvature — observable:CurvatureObservable measures this gap.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/morphism/Isometry"),
            range: "https://uor.foundation/observable/MetricObservable",
        },
        Property {
            id: "https://uor.foundation/morphism/sourceQuantum",
            label: "sourceQuantum",
            comment: "The quantum level n of the source ring for an embedding.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Embedding"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/morphism/targetQuantum",
            label: "targetQuantum",
            comment: "The quantum level n' of the target ring for an embedding. \
                      Must satisfy n' > n (embeddings go to larger rings).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Embedding"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/morphism/group",
            label: "group",
            comment: "The group acting in this group action.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Action"),
            range: "https://uor.foundation/op/Group",
        },
        Property {
            id: "https://uor.foundation/morphism/actingOn",
            label: "actingOn",
            comment: "The set being acted upon by this group action.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Action"),
            range: OWL_THING,
        },
        Property {
            id: "https://uor.foundation/morphism/actionIsometry",
            label: "actionIsometry",
            comment: "Whether every transform induced by this action is an isometry. \
                      True for the dihedral action on 𝒯_n (Frame Theorem).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Action"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/morphism/trace",
            label: "trace",
            comment: "The computation trace that realized this transform at runtime. \
                      A Transform is an abstraction; a trace is the kernel's record \
                      of how it was executed via concrete operations.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Transform"),
            range: "https://uor.foundation/trace/ComputationTrace",
        },
        // Amendment 12: Composition Primitive properties
        Property {
            id: "https://uor.foundation/morphism/composesWith",
            label: "composesWith",
            comment: "A transform that this transform can be composed with. \
                      The target of this transform must match the source of \
                      the composed transform.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/morphism/Transform"),
            range: "https://uor.foundation/morphism/Transform",
        },
        Property {
            id: "https://uor.foundation/morphism/compositionResult",
            label: "compositionResult",
            comment: "The transform that results from this composition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Composition"),
            range: "https://uor.foundation/morphism/Transform",
        },
        Property {
            id: "https://uor.foundation/morphism/compositionComponents",
            label: "compositionComponents",
            comment: "A component transform of this composition.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/morphism/Composition"),
            range: "https://uor.foundation/morphism/Transform",
        },
        Property {
            id: "https://uor.foundation/morphism/identityOn",
            label: "identityOn",
            comment: "The type on which this identity transform acts.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Identity"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/morphism/compositionOrder",
            label: "compositionOrder",
            comment: "The number of component transforms in a composition.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: None,
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/morphism/isAssociative",
            label: "isAssociative",
            comment: "Whether this composition law is associative.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/CompositionLaw"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/morphism/isCommutative",
            label: "isCommutative",
            comment: "Whether this composition law is commutative.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/CompositionLaw"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/morphism/lawComponents",
            label: "lawComponents",
            comment: "An operation that is a component of this composition law.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/morphism/CompositionLaw"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/morphism/lawResult",
            label: "lawResult",
            comment: "The operation that results from this composition law.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/CompositionLaw"),
            range: "https://uor.foundation/op/Operation",
        },
        // Amendment 75: preservesStructure replaced by preservedInvariant
        Property {
            id: "https://uor.foundation/morphism/preservedInvariant",
            label: "preservedInvariant",
            comment: "The identity preserved by this transform (reference \
                      to the op:Identity that the transform commutes with).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Transform"),
            range: "https://uor.foundation/op/Identity",
        },
        // Amendment 13: Address Resolution
        Property {
            id: "https://uor.foundation/morphism/addressCoherence",
            label: "addressCoherence",
            comment: "Certificate that this embedding's addressing diagram commutes: \
                      glyph ∘ ι ∘ addresses is well-defined and injective.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Embedding"),
            range: "https://uor.foundation/op/Identity",
        },
        // Amendment 22: TopologicalDelta properties
        Property {
            id: "https://uor.foundation/morphism/bettisBefore",
            label: "bettisBefore",
            comment: "Betti numbers before the morphism.",
            domain: Some("https://uor.foundation/morphism/TopologicalDelta"),
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            range: "https://uor.foundation/observable/BettiNumber",
        },
        Property {
            id: "https://uor.foundation/morphism/bettisAfter",
            label: "bettisAfter",
            comment: "Betti numbers after the morphism.",
            domain: Some("https://uor.foundation/morphism/TopologicalDelta"),
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            range: "https://uor.foundation/observable/BettiNumber",
        },
        Property {
            id: "https://uor.foundation/morphism/eulerBefore",
            label: "eulerBefore",
            comment: "Euler characteristic before the morphism.",
            domain: Some("https://uor.foundation/morphism/TopologicalDelta"),
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/morphism/eulerAfter",
            label: "eulerAfter",
            comment: "Euler characteristic after the morphism.",
            domain: Some("https://uor.foundation/morphism/TopologicalDelta"),
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/morphism/nerveBefore",
            label: "nerveBefore",
            comment: "Constraint nerve (simplicial complex) before the morphism.",
            domain: Some("https://uor.foundation/morphism/TopologicalDelta"),
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            range: "https://uor.foundation/homology/SimplicialComplex",
        },
        Property {
            id: "https://uor.foundation/morphism/nerveAfter",
            label: "nerveAfter",
            comment: "Constraint nerve (simplicial complex) after the morphism.",
            domain: Some("https://uor.foundation/morphism/TopologicalDelta"),
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            range: "https://uor.foundation/homology/SimplicialComplex",
        },
        // Amendment 24 (re-domained in Amendment 95): GroundingWitness properties
        Property {
            id: "https://uor.foundation/morphism/surfaceSymbol",
            label: "surfaceSymbol",
            comment: "The surface symbol that is the source of this grounding witness.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/GroundingWitness"),
            range: "https://uor.foundation/schema/SurfaceSymbol",
        },
        Property {
            id: "https://uor.foundation/morphism/groundedAddress",
            label: "groundedAddress",
            comment: "The resolved ring address that is the target of this grounding witness.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/GroundingWitness"),
            range: "https://uor.foundation/u/Element",
        },
        Property {
            id: "https://uor.foundation/morphism/groundingDerivation",
            label: "groundingDerivation",
            comment: "The derivation witnessing the content-addressing computation \
                      that produced the grounded address from the surface symbol.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/GroundingMap"),
            range: "https://uor.foundation/derivation/Derivation",
        },
        Property {
            id: "https://uor.foundation/morphism/symbolConstraints",
            label: "symbolConstraints",
            comment: "A typed attribute preserved by this grounding. Non-functional: \
                      one assertion per active constraint axis (vertical, horizontal, diagonal).",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/morphism/GroundingMap"),
            range: "https://uor.foundation/type/Constraint",
        },
        // Amendment 24: ProjectionMap properties
        Property {
            id: "https://uor.foundation/morphism/projectionFrame",
            label: "projectionFrame",
            comment: "The active frame — shared with the grounding that produced the query. \
                      The shared-frame condition (Surface Symmetry Theorem) requires G and P \
                      to reference the same frame.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/ProjectionMap"),
            range: "https://uor.foundation/state/Frame",
        },
        Property {
            id: "https://uor.foundation/morphism/projectionSource",
            label: "projectionSource",
            comment: "The resolved partition (address neighbourhood) that this projection \
                      witness projects back to surface symbols.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/ProjectionWitness"),
            range: "https://uor.foundation/partition/Partition",
        },
        Property {
            id: "https://uor.foundation/morphism/projectionOrder",
            label: "projectionOrder",
            comment: "Ordering constraint determining the output symbol sequence. \
                      Domain-specific: syntactic position (NLP), row-major scan (ARC), \
                      temporal sequence (music).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/ProjectionMap"),
            range: "https://uor.foundation/type/Conjunction",
        },
        Property {
            id: "https://uor.foundation/morphism/roundTripCoherence",
            label: "roundTripCoherence",
            comment: "Completeness criterion: does projecting the grounded source address \
                      recover a symbol in the same type class as the input? True iff the \
                      shared-frame condition holds.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/ProjectionMap"),
            range: XSD_BOOLEAN,
        },
        // Gap G: GroundingCertificate properties
        Property {
            id: "https://uor.foundation/morphism/groundingCertMap",
            label: "groundingCertMap",
            comment: "The GroundingMap used in this certified round-trip.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/GroundingCertificate"),
            range: "https://uor.foundation/morphism/GroundingMap",
        },
        Property {
            id: "https://uor.foundation/morphism/groundingCertProjection",
            label: "groundingCertProjection",
            comment: "The ProjectionMap used in this certified round-trip.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/GroundingCertificate"),
            range: "https://uor.foundation/morphism/ProjectionMap",
        },
        Property {
            id: "https://uor.foundation/morphism/groundingCertSourceSymbol",
            label: "groundingCertSourceSymbol",
            comment: "The surface symbol that entered the grounding boundary.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/GroundingCertificate"),
            range: "https://uor.foundation/schema/Literal",
        },
        Property {
            id: "https://uor.foundation/morphism/groundingCertAddress",
            label: "groundingCertAddress",
            comment: "The ring address the symbol was grounded to.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/GroundingCertificate"),
            range: "https://uor.foundation/u/Element",
        },
        // Amendment 75: Higher-Order Computation properties
        Property {
            id: "https://uor.foundation/morphism/referencedCertificate",
            label: "referencedCertificate",
            comment: "The certificate this computation datum encodes.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/ComputationDatum"),
            range: "https://uor.foundation/cert/TransformCertificate",
        },
        Property {
            id: "https://uor.foundation/morphism/computationAddress",
            label: "computationAddress",
            comment: "The content address of the referenced certificate.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/ComputationDatum"),
            range: "https://uor.foundation/u/Element",
        },
        Property {
            id: "https://uor.foundation/morphism/applicationTarget",
            label: "applicationTarget",
            comment: "The computation being applied.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/ApplicationMorphism"),
            range: "https://uor.foundation/morphism/ComputationDatum",
        },
        Property {
            id: "https://uor.foundation/morphism/applicationInput",
            label: "applicationInput",
            comment: "The input datum to the application.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/ApplicationMorphism"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/morphism/partialBase",
            label: "partialBase",
            comment: "The base computation being partially applied.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/PartialApplication"),
            range: "https://uor.foundation/morphism/ComputationDatum",
        },
        Property {
            id: "https://uor.foundation/morphism/boundArguments",
            label: "boundArguments",
            comment: "The arguments already bound.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/morphism/PartialApplication"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/morphism/compositionLeft",
            label: "compositionLeft",
            comment: "The outer function f in f \u{2218} g.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/TransformComposition"),
            range: "https://uor.foundation/morphism/ComputationDatum",
        },
        Property {
            id: "https://uor.foundation/morphism/compositionRight",
            label: "compositionRight",
            comment: "The inner function g in f \u{2218} g.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/TransformComposition"),
            range: "https://uor.foundation/morphism/ComputationDatum",
        },
        Property {
            id: "https://uor.foundation/morphism/remainingArity",
            label: "remainingArity",
            comment: "Number of unbound arguments remaining.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/PartialApplication"),
            range: XSD_POSITIVE_INTEGER,
        },
        // Amendment 95: Boundary map registry properties (Workstream 4)
        Property {
            id: "https://uor.foundation/morphism/inputClass",
            label: "inputClass",
            comment: "The OWL class of inputs this transform accepts. Uses \
                      OWL2 punning so the value is a class IRI treated as \
                      an individual at this position.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Transform"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/morphism/outputClass",
            label: "outputClass",
            comment: "The OWL class of outputs this transform produces. Uses \
                      OWL2 punning.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/Transform"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/morphism/outputElementClass",
            label: "outputElementClass",
            comment: "When outputClass is a sequence type, the OWL class of \
                      individual sequence elements. Uses OWL2 punning.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/ProjectionMap"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/morphism/hasWitness",
            label: "hasWitness",
            comment: "Zero or more witness pairs documenting specific \
                      input/output bindings of this transform.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/morphism/Transform"),
            range: "https://uor.foundation/morphism/Witness",
        },
        Property {
            id: "https://uor.foundation/morphism/hasElement",
            label: "hasElement",
            comment: "Membership of a SymbolSequence. The sequence is \
                      reconstructed by sorting elements by elementIndex.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/morphism/SymbolSequence"),
            range: "https://uor.foundation/morphism/SequenceElement",
        },
        Property {
            id: "https://uor.foundation/morphism/elementValue",
            label: "elementValue",
            comment: "The surface symbol value of this sequence element.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/SequenceElement"),
            range: "https://uor.foundation/schema/SurfaceSymbol",
        },
        Property {
            id: "https://uor.foundation/morphism/elementIndex",
            label: "elementIndex",
            comment: "The zero-based position of this element in the sequence.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/SequenceElement"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/morphism/projectionOutput",
            label: "projectionOutput",
            comment: "The single SymbolSequence produced by this projection \
                      witness.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/morphism/ProjectionWitness"),
            range: "https://uor.foundation/morphism/SymbolSequence",
        },
    ]
}

fn individuals() -> Vec<Individual> {
    vec![
        Individual {
            id: "https://uor.foundation/morphism/criticalComposition",
            type_: "https://uor.foundation/morphism/CompositionLaw",
            label: "criticalComposition",
            comment: "The critical composition law: neg ∘ bnot = succ. This is the \
                      operational form of the critical identity theorem. The \
                      composition of the two involutions (neg, bnot) yields the \
                      successor operation. Non-associative and non-commutative.",
            properties: &[
                (
                    "https://uor.foundation/morphism/lawComponents",
                    IndividualValue::IriRef("https://uor.foundation/op/neg"),
                ),
                (
                    "https://uor.foundation/morphism/lawComponents",
                    IndividualValue::IriRef("https://uor.foundation/op/bnot"),
                ),
                (
                    "https://uor.foundation/morphism/lawResult",
                    IndividualValue::IriRef("https://uor.foundation/op/succ"),
                ),
                (
                    "https://uor.foundation/morphism/isAssociative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/morphism/isCommutative",
                    IndividualValue::Bool(false),
                ),
            ],
        },
        // Amendment 95: Boundary map individuals (Workstream 4)
        Individual {
            id: "https://uor.foundation/morphism/IntegerGroundingMap",
            type_: "https://uor.foundation/morphism/GroundingMap",
            label: "IntegerGroundingMap",
            comment: "Grounds integer surface symbols to ring addresses.",
            properties: &[
                (
                    "https://uor.foundation/morphism/inputClass",
                    IndividualValue::IriRef("https://uor.foundation/schema/Literal"),
                ),
                (
                    "https://uor.foundation/morphism/outputClass",
                    IndividualValue::IriRef("https://uor.foundation/u/Element"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/morphism/Utf8GroundingMap",
            type_: "https://uor.foundation/morphism/GroundingMap",
            label: "Utf8GroundingMap",
            comment: "Grounds UTF-8 host strings to ring addresses.",
            properties: &[
                (
                    "https://uor.foundation/morphism/inputClass",
                    IndividualValue::IriRef("https://uor.foundation/schema/HostStringLiteral"),
                ),
                (
                    "https://uor.foundation/morphism/outputClass",
                    IndividualValue::IriRef("https://uor.foundation/u/Element"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/morphism/JsonGroundingMap",
            type_: "https://uor.foundation/morphism/GroundingMap",
            label: "JsonGroundingMap",
            comment: "Grounds JSON host strings to ring addresses.",
            properties: &[
                (
                    "https://uor.foundation/morphism/inputClass",
                    IndividualValue::IriRef("https://uor.foundation/schema/HostStringLiteral"),
                ),
                (
                    "https://uor.foundation/morphism/outputClass",
                    IndividualValue::IriRef("https://uor.foundation/u/Element"),
                ),
            ],
        },
        // v0.2.2 W4: GroundingMap kind discriminator — two new individuals
        // exposing kinds that the v0.2.2 Rust enforcement layer marks with
        // specific PreservesStructure / Invertible / Total marker traits.
        Individual {
            id: "https://uor.foundation/morphism/DigestGroundingMap",
            type_: "https://uor.foundation/morphism/GroundingMap",
            label: "DigestGroundingMap",
            comment: "A GroundingMap that maps fixed-size external byte \
                      strings to ring datums via a one-way digest function \
                      (e.g., SHA-256). Total but not invertible: every input \
                      grounds, but distinct inputs may collide; the inverse \
                      is computationally unbounded. Preserves no algebraic \
                      structure.",
            properties: &[
                (
                    "https://uor.foundation/morphism/inputClass",
                    IndividualValue::IriRef("https://uor.foundation/schema/HostStringLiteral"),
                ),
                (
                    "https://uor.foundation/morphism/outputClass",
                    IndividualValue::IriRef("https://uor.foundation/u/Element"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/morphism/BinaryGroundingMap",
            type_: "https://uor.foundation/morphism/GroundingMap",
            label: "BinaryGroundingMap",
            comment: "A GroundingMap that maps host-supplied byte sequences \
                      to ring datums via a fixed bit-pattern interpretation \
                      (e.g., little-endian unsigned integer reading). Total \
                      and invertible on its declared bit-width; preserves no \
                      algebraic structure beyond bit identity.",
            properties: &[
                (
                    "https://uor.foundation/morphism/inputClass",
                    IndividualValue::IriRef("https://uor.foundation/schema/HostStringLiteral"),
                ),
                (
                    "https://uor.foundation/morphism/outputClass",
                    IndividualValue::IriRef("https://uor.foundation/u/Element"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/morphism/IntegerProjectionMap",
            type_: "https://uor.foundation/morphism/ProjectionMap",
            label: "IntegerProjectionMap",
            comment: "Projects partitions to integer symbol sequences.",
            properties: &[
                (
                    "https://uor.foundation/morphism/inputClass",
                    IndividualValue::IriRef("https://uor.foundation/partition/Partition"),
                ),
                (
                    "https://uor.foundation/morphism/outputClass",
                    IndividualValue::IriRef("https://uor.foundation/morphism/SymbolSequence"),
                ),
                (
                    "https://uor.foundation/morphism/outputElementClass",
                    IndividualValue::IriRef("https://uor.foundation/schema/Literal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/morphism/Utf8ProjectionMap",
            type_: "https://uor.foundation/morphism/ProjectionMap",
            label: "Utf8ProjectionMap",
            comment: "Projects partitions to UTF-8 symbol sequences.",
            properties: &[
                (
                    "https://uor.foundation/morphism/inputClass",
                    IndividualValue::IriRef("https://uor.foundation/partition/Partition"),
                ),
                (
                    "https://uor.foundation/morphism/outputClass",
                    IndividualValue::IriRef("https://uor.foundation/morphism/SymbolSequence"),
                ),
                (
                    "https://uor.foundation/morphism/outputElementClass",
                    IndividualValue::IriRef("https://uor.foundation/schema/HostStringLiteral"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/morphism/JsonProjectionMap",
            type_: "https://uor.foundation/morphism/ProjectionMap",
            label: "JsonProjectionMap",
            comment: "Projects partitions to JSON symbol sequences.",
            properties: &[
                (
                    "https://uor.foundation/morphism/inputClass",
                    IndividualValue::IriRef("https://uor.foundation/partition/Partition"),
                ),
                (
                    "https://uor.foundation/morphism/outputClass",
                    IndividualValue::IriRef("https://uor.foundation/morphism/SymbolSequence"),
                ),
                (
                    "https://uor.foundation/morphism/outputElementClass",
                    IndividualValue::IriRef("https://uor.foundation/schema/HostStringLiteral"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/morphism/DigestProjectionMap",
            type_: "https://uor.foundation/morphism/ProjectionMap",
            label: "DigestProjectionMap",
            comment: "Projects partitions to fixed-size digest symbol \
                      sequences. Dual of DigestGroundingMap.",
            properties: &[
                (
                    "https://uor.foundation/morphism/inputClass",
                    IndividualValue::IriRef("https://uor.foundation/partition/Partition"),
                ),
                (
                    "https://uor.foundation/morphism/outputClass",
                    IndividualValue::IriRef("https://uor.foundation/morphism/SymbolSequence"),
                ),
                (
                    "https://uor.foundation/morphism/outputElementClass",
                    IndividualValue::IriRef("https://uor.foundation/schema/Literal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/morphism/BinaryProjectionMap",
            type_: "https://uor.foundation/morphism/ProjectionMap",
            label: "BinaryProjectionMap",
            comment: "Projects partitions to raw byte symbol sequences. \
                      Dual of BinaryGroundingMap.",
            properties: &[
                (
                    "https://uor.foundation/morphism/inputClass",
                    IndividualValue::IriRef("https://uor.foundation/partition/Partition"),
                ),
                (
                    "https://uor.foundation/morphism/outputClass",
                    IndividualValue::IriRef("https://uor.foundation/morphism/SymbolSequence"),
                ),
                (
                    "https://uor.foundation/morphism/outputElementClass",
                    IndividualValue::IriRef("https://uor.foundation/schema/Literal"),
                ),
            ],
        },
    ]
}
