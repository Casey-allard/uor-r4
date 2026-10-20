//! `conformance/` namespace — Conformance shapes.
//!
//! The `conformance/` namespace defines SHACL-equivalent constraint shapes
//! specifying what a Prism implementation must provide at each extension
//! point. Machine-verifiable contracts.
//!
//! - **Amendment 82**: 11 classes, 9 properties, 0 individuals
//! - **Amendment 84**: 0 classes, 0 properties, 5 individuals
//!   (CompileUnitShape + 4 PropertyConstraint)
//! - **Amendment 95**: 19 classes, 40 properties, 5 individuals
//!   (Declarative enforcement: opaque witnesses, builders, violation kinds)
//!
//! **Space classification:** `bridge` — kernel-computed, user-consumed.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `conformance/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "conformance",
            iri: NS_CONFORMANCE,
            label: "UOR Conformance Shapes",
            comment: "SHACL-equivalent constraint shapes defining what a \
                      Prism implementation must provide at each extension \
                      point. Machine-verifiable contracts.",
            space: Space::Bridge,
            imports: &[
                NS_SCHEMA,
                NS_TYPE,
                NS_OP,
                NS_EFFECT,
                NS_PREDICATE,
                NS_PARALLEL,
                NS_STREAM,
                NS_LINEAR,
                NS_REGION,
                NS_FAILURE,
                NS_RECURSION,
                NS_BOUNDARY,
                NS_REDUCTION,
                NS_CERT,
                NS_TRACE,
                NS_STATE,
                NS_MORPHISM,
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
            id: "https://uor.foundation/conformance/Shape",
            label: "Shape",
            comment: "A constraint shape that a Prism-declared extension \
                      must satisfy. Analogous to sh:NodeShape in SHACL.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/PropertyConstraint",
            label: "PropertyConstraint",
            comment: "A single required property within a shape: the \
                      property URI, its expected range, minimum and maximum \
                      cardinality.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/WittLevelShape",
            label: "WittLevelShape",
            comment: "Shape for declaring a new WittLevel beyond Q3.",
            subclass_of: &["https://uor.foundation/conformance/Shape"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/EffectShape",
            label: "EffectShape",
            comment: "Shape for declaring an ExternalEffect.",
            subclass_of: &["https://uor.foundation/conformance/Shape"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/ParallelShape",
            label: "ParallelShape",
            comment: "Shape for declaring a ParallelProduct.",
            subclass_of: &["https://uor.foundation/conformance/Shape"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/StreamShape",
            label: "StreamShape",
            comment: "Shape for declaring a ProductiveStream (targets \
                      stream:Unfold, the coinductive constructor).",
            subclass_of: &["https://uor.foundation/conformance/Shape"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/DispatchShape",
            label: "DispatchShape",
            comment: "Shape for declaring a new DispatchRule in a \
                      DispatchTable.",
            subclass_of: &["https://uor.foundation/conformance/Shape"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/LeaseShape",
            label: "LeaseShape",
            comment: "Shape for declaring a Lease with LinearSite \
                      allocation.",
            subclass_of: &["https://uor.foundation/conformance/Shape"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/GroundingShape",
            label: "GroundingShape",
            comment: "Shape for declaring a GroundingMap from surface data \
                      to the ring.",
            subclass_of: &["https://uor.foundation/conformance/Shape"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/ValidationResult",
            label: "ValidationResult",
            comment: "The result of validating an extension against a shape: \
                      conforms (boolean), and violation details if \
                      non-conformant.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/PredicateShape",
            label: "PredicateShape",
            comment: "Shape for user-declared predicates. Requires a \
                      bounded evaluator (termination witness) and input \
                      type declaration.",
            subclass_of: &["https://uor.foundation/conformance/Shape"],
            disjoint_with: &[],
        },
        // v0.2.2 T1.2: Shape backing the InteractionDeclarationBuilder
        // validate path (originally planned for Phase E but not shipped).
        Class {
            id: "https://uor.foundation/conformance/InteractionShape",
            label: "InteractionShape",
            comment: "Shape describing the required surface of an \
                      InteractionDeclaration consumed by the foundation's \
                      InteractionDeclarationBuilder: peer protocol, \
                      convergence predicate, and commutator state class. \
                      Rejects builders missing any of the three.",
            subclass_of: &["https://uor.foundation/conformance/Shape"],
            disjoint_with: &[],
        },
        // ── Amendment 95: Declarative enforcement types ──
        Class {
            id: "https://uor.foundation/conformance/WitnessDatum",
            label: "WitnessDatum",
            comment: "Opaque ring element witness. Cannot be constructed \
                      outside the foundation crate \u{2014} only produced by \
                      reduction evaluation or the two-phase minting boundary.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/GroundedCoordinate",
            label: "GroundedCoordinate",
            comment: "Boundary crossing intermediate for a single grounded \
                      coordinate value. Not a WitnessDatum \u{2014} must be \
                      validated and minted by the foundation.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/GroundedTuple",
            label: "GroundedTuple",
            comment: "Boundary crossing intermediate for a fixed-size array \
                      of GroundedCoordinate values. Stack-resident, no heap \
                      allocation.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/GroundedValueMarker",
            label: "GroundedValueMarker",
            comment: "Sealed marker trait class. Implemented only for \
                      GroundedCoordinate and GroundedTuple. Prevents \
                      downstream crates from substituting arbitrary types.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/ValidatedWrapper",
            label: "ValidatedWrapper",
            comment: "Generic validation-proof wrapper. Proves that the \
                      inner value was produced by the conformance checker, \
                      not fabricated by Prism code.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/WitnessDerivation",
            label: "WitnessDerivation",
            comment: "Opaque derivation trace that can only be extended \
                      by the rewrite engine. Records rewrite step count \
                      and root term content address.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/WitnessSiteBudget",
            label: "WitnessSiteBudget",
            comment: "Opaque site budget that can only be decremented by \
                      PinningEffect and incremented by UnbindingEffect \
                      \u{2014} never by direct mutation.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/ShapeViolationReport",
            label: "ShapeViolationReport",
            comment: "Structured violation diagnostic carrying the shape \
                      IRI, constraint IRI, property IRI, expected range, \
                      cardinality bounds, and violation kind.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/ViolationKind",
            label: "ViolationKind",
            comment: "The kind of shape violation: Missing, TypeMismatch, \
                      CardinalityViolation, ValueCheck, or LevelMismatch.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/CompileUnitBuilder",
            label: "CompileUnitBuilder",
            comment: "Builder for CompileUnit admission. Collects rootTerm, \
                      quantumLevelCeiling, thermodynamicBudget, and \
                      targetDomains. Validates against CompileUnitShape.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/EffectDeclaration",
            label: "EffectDeclaration",
            comment: "Builder for EffectShape. Collects effect name, target \
                      sites, budget delta, and commutation flag.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/GroundingDeclaration",
            label: "GroundingDeclaration",
            comment: "Builder for GroundingShape. Collects source type, \
                      ring mapping, and invertibility contract.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/DispatchDeclaration",
            label: "DispatchDeclaration",
            comment: "Builder for DispatchShape. Collects predicate, target \
                      resolver, and dispatch priority.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/LeaseDeclaration",
            label: "LeaseDeclaration",
            comment: "Builder for LeaseShape. Collects linear site and \
                      lease scope.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/StreamDeclaration",
            label: "StreamDeclaration",
            comment: "Builder for StreamShape. Collects unfold seed, step \
                      term, and productivity witness.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/PredicateDeclaration",
            label: "PredicateDeclaration",
            comment: "Builder for PredicateShape. Collects input type, \
                      evaluator term, and termination witness.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/ParallelDeclaration",
            label: "ParallelDeclaration",
            comment: "Builder for ParallelShape. Collects site partition \
                      and disjointness witness.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/WittLevelDeclaration",
            label: "WittLevelDeclaration",
            comment: "Builder for WittLevelShape. Collects declared bit \
                      width, cycle size, and predecessor level.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/conformance/MintingSession",
            label: "MintingSession",
            comment: "Boundary session state tracker. Records crossing count \
                      and idempotency flag for the two-phase minting \
                      boundary.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // v0.2.1: Parametric prelude membership metadata. The Rust codegen
        // walks PreludeExport individuals to emit the foundation::prelude
        // module's `pub use` re-exports. Adding a new class to the prelude
        // requires only a new PreludeExport individual referencing that
        // class IRI.
        Class {
            id: "https://uor.foundation/conformance/PreludeExport",
            label: "PreludeExport",
            comment: "An ontology fact recording that a particular OWL class \
                      should appear in the foundation crate's `prelude` \
                      module re-exports. The v0.2.1 Rust codegen walks \
                      PreludeExport individuals filtered by exportsClass to \
                      assemble the prelude membership list.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Object properties
        Property {
            id: "https://uor.foundation/conformance/targetClass",
            label: "targetClass",
            comment: "The OWL class that instances of this shape must \
                      belong to.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/Shape"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/conformance/requiredProperty",
            label: "requiredProperty",
            comment: "A required property in this shape.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/conformance/Shape"),
            range: "https://uor.foundation/conformance/PropertyConstraint",
        },
        Property {
            id: "https://uor.foundation/conformance/constraintProperty",
            label: "constraintProperty",
            comment: "The property URI that must be present.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/PropertyConstraint"),
            range: OWL_THING,
        },
        Property {
            id: "https://uor.foundation/conformance/constraintRange",
            label: "constraintRange",
            comment: "The expected range of the required property.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/PropertyConstraint"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/conformance/validationShape",
            label: "validationShape",
            comment: "The shape that was validated against.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ValidationResult"),
            range: "https://uor.foundation/conformance/Shape",
        },
        Property {
            id: "https://uor.foundation/conformance/validationTarget",
            label: "validationTarget",
            comment: "The instance that was validated.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ValidationResult"),
            range: OWL_THING,
        },
        // Datatype properties
        Property {
            id: "https://uor.foundation/conformance/minCount",
            label: "minCount",
            comment: "Minimum cardinality of the required property.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/PropertyConstraint"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/conformance/maxCount",
            label: "maxCount",
            comment: "Maximum cardinality (0 = unbounded).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/PropertyConstraint"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/conformance/conforms",
            label: "conforms",
            comment: "True iff the target satisfies all constraints of the \
                      shape.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ValidationResult"),
            range: XSD_BOOLEAN,
        },
        // ── Amendment 95: Witness type properties (11) ──
        Property {
            id: "https://uor.foundation/conformance/witnessLevel",
            label: "witnessLevel",
            comment: "The quantum level at which this witness datum was minted.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/WitnessDatum"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/conformance/witnessBytes",
            label: "witnessBytes",
            comment: "The raw byte representation of this witness datum.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/WitnessDatum"),
            range: XSD_HEX_BINARY,
        },
        Property {
            id: "https://uor.foundation/conformance/coordinateLevel",
            label: "coordinateLevel",
            comment: "The quantum level tag of this grounded coordinate.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/GroundedCoordinate"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/conformance/validatedInner",
            label: "validatedInner",
            comment: "The validated inner value wrapped by this proof.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ValidatedWrapper"),
            range: OWL_THING,
        },
        Property {
            id: "https://uor.foundation/conformance/shapeIri",
            label: "shapeIri",
            comment: "IRI of the conformance:Shape that was validated against.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ShapeViolationReport"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/conformance/constraintIri",
            label: "constraintIri",
            comment: "IRI of the specific PropertyConstraint that failed.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ShapeViolationReport"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/conformance/propertyIri",
            label: "propertyIri",
            comment: "IRI of the property that was missing or invalid.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ShapeViolationReport"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/conformance/expectedRange",
            label: "expectedRange",
            comment: "The expected range class IRI for the violated property.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ShapeViolationReport"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/conformance/violationMinCount",
            label: "violationMinCount",
            comment: "The minimum cardinality from the violated constraint.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ShapeViolationReport"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/conformance/violationMaxCount",
            label: "violationMaxCount",
            comment: "The maximum cardinality from the violated constraint \
                      (0 = unbounded).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ShapeViolationReport"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/conformance/violationKind",
            label: "violationKind",
            comment: "The kind of violation that occurred.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ShapeViolationReport"),
            range: "https://uor.foundation/conformance/ViolationKind",
        },
        // ── Amendment 95: Builder properties (27) ──
        // CompileUnitBuilder (4)
        Property {
            id: "https://uor.foundation/conformance/builderRootTerm",
            label: "builderRootTerm",
            comment: "The root term expression for the CompileUnit.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/CompileUnitBuilder"),
            range: "https://uor.foundation/schema/Term",
        },
        Property {
            id: "https://uor.foundation/conformance/builderWittLevelCeiling",
            label: "builderWittLevelCeiling",
            comment: "The widest quantum level the computation may reference.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/CompileUnitBuilder"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/conformance/builderThermodynamicBudget",
            label: "builderThermodynamicBudget",
            comment: "Landauer-bounded energy budget in kBT ln 2 units.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/CompileUnitBuilder"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/conformance/builderTargetDomains",
            label: "builderTargetDomains",
            comment: "Verification domains targeted by the CompileUnit.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/conformance/CompileUnitBuilder"),
            range: "https://uor.foundation/op/VerificationDomain",
        },
        // EffectDeclaration (4)
        Property {
            id: "https://uor.foundation/conformance/effectName",
            label: "effectName",
            comment: "The name of the declared effect.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/EffectDeclaration"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/conformance/targetSites",
            label: "targetSites",
            comment: "Site coordinates this effect reads or writes.",
            kind: PropertyKind::Datatype,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/conformance/EffectDeclaration"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/conformance/budgetDelta",
            label: "budgetDelta",
            comment: "The site budget delta (positive = increment, \
                      negative = decrement).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/EffectDeclaration"),
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/conformance/commutationFlag",
            label: "commutationFlag",
            comment: "Whether this effect commutes with effects on \
                      disjoint sites.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/EffectDeclaration"),
            range: XSD_BOOLEAN,
        },
        // GroundingDeclaration (3)
        Property {
            id: "https://uor.foundation/conformance/groundingSourceType",
            label: "groundingSourceType",
            comment: "The source type of incoming external data.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/GroundingDeclaration"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/conformance/ringMapping",
            label: "ringMapping",
            comment: "Description of the mapping from surface data to ring.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/GroundingDeclaration"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/conformance/invertibilityContract",
            label: "invertibilityContract",
            comment: "Whether the grounding map is invertible.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/GroundingDeclaration"),
            range: XSD_BOOLEAN,
        },
        // DispatchDeclaration (3)
        Property {
            id: "https://uor.foundation/conformance/dispatchPredicate",
            label: "dispatchPredicate",
            comment: "The predicate expression guarding this dispatch rule.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/DispatchDeclaration"),
            range: "https://uor.foundation/reduction/PredicateExpression",
        },
        Property {
            id: "https://uor.foundation/conformance/targetResolver",
            label: "targetResolver",
            comment: "The resolver to dispatch to when the predicate holds.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/DispatchDeclaration"),
            range: "https://uor.foundation/resolver/Resolver",
        },
        Property {
            id: "https://uor.foundation/conformance/dispatchPriority",
            label: "dispatchPriority",
            comment: "Priority ordering for this dispatch rule (lower = first).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/DispatchDeclaration"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // LeaseDeclaration (2)
        Property {
            id: "https://uor.foundation/conformance/linearSite",
            label: "linearSite",
            comment: "The site coordinate allocated linearly by this lease.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/LeaseDeclaration"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/conformance/leaseScope",
            label: "leaseScope",
            comment: "The scope within which this lease is valid.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/LeaseDeclaration"),
            range: XSD_STRING,
        },
        // StreamDeclaration (3)
        Property {
            id: "https://uor.foundation/conformance/unfoldSeed",
            label: "unfoldSeed",
            comment: "The seed term for the stream unfold constructor.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/StreamDeclaration"),
            range: "https://uor.foundation/schema/Term",
        },
        Property {
            id: "https://uor.foundation/conformance/stepTerm",
            label: "stepTerm",
            comment: "The step function term for the stream unfold.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/StreamDeclaration"),
            range: "https://uor.foundation/schema/Term",
        },
        Property {
            id: "https://uor.foundation/conformance/productivityWitness",
            label: "productivityWitness",
            comment: "Evidence that the stream is productive (always \
                      produces a next element).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/StreamDeclaration"),
            range: XSD_STRING,
        },
        // PredicateDeclaration (3)
        Property {
            id: "https://uor.foundation/conformance/predicateInputType",
            label: "predicateInputType",
            comment: "The input type for the declared predicate.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/PredicateDeclaration"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/conformance/evaluatorTerm",
            label: "evaluatorTerm",
            comment: "The evaluator term for the declared predicate.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/PredicateDeclaration"),
            range: "https://uor.foundation/schema/Term",
        },
        Property {
            id: "https://uor.foundation/conformance/terminationWitness",
            label: "terminationWitness",
            comment: "Evidence that the predicate evaluator terminates on \
                      all inputs.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/PredicateDeclaration"),
            range: XSD_STRING,
        },
        // ParallelDeclaration (2)
        Property {
            id: "https://uor.foundation/conformance/sitePartition",
            label: "sitePartition",
            comment: "The site partition for the parallel composition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ParallelDeclaration"),
            range: "https://uor.foundation/partition/Partition",
        },
        Property {
            id: "https://uor.foundation/conformance/disjointnessWitness",
            label: "disjointnessWitness",
            comment: "Evidence that the site partition components are \
                      pairwise disjoint.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/ParallelDeclaration"),
            range: XSD_STRING,
        },
        // WittLevelDeclaration (3)
        Property {
            id: "https://uor.foundation/conformance/declaredBitWidth",
            label: "declaredBitWidth",
            comment: "The declared bit width for this quantum level.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/WittLevelDeclaration"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/conformance/declaredCycleSize",
            label: "declaredCycleSize",
            comment: "The declared number of ring states at this level.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/WittLevelDeclaration"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/conformance/predecessorLevel",
            label: "predecessorLevel",
            comment: "The predecessor quantum level in the chain.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/WittLevelDeclaration"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        // MintingSession (2)
        Property {
            id: "https://uor.foundation/conformance/sessionCrossingCount",
            label: "sessionCrossingCount",
            comment: "Total boundary crossings in this minting session.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/MintingSession"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/conformance/sessionIsIdempotent",
            label: "sessionIsIdempotent",
            comment: "Whether applying this session's boundary effect \
                      twice equals applying it once.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/MintingSession"),
            range: XSD_BOOLEAN,
        },
        // v0.2.1: Surface-grammar metadata for parametric uor.conformance.ebnf
        // emission (the conformance ebnf serializer reads these to drive
        // production names, keyword literals, and value-type slots).
        Property {
            id: "https://uor.foundation/conformance/surfaceForm",
            label: "surfaceForm",
            comment: "Top-level EBNF non-terminal name this Shape generates \
                      (e.g., \"compile-unit-decl\" for CompileUnitShape).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/Shape"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/conformance/surfaceKeyword",
            label: "surfaceKeyword",
            comment: "Literal surface keyword used in the conformance grammar \
                      for this property constraint (e.g., \"root_term\", \
                      \"witt_level_ceiling\").",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/PropertyConstraint"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/conformance/surfaceProduction",
            label: "surfaceProduction",
            comment: "EBNF non-terminal that the value at this constraint \
                      slot must match (e.g., \"program\" for Term ranges, \
                      \"name\" for WittLevel ranges, \"decimal-literal\" for \
                      xsd:decimal, \"domain-set\" for non-functional IRI lists).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/PropertyConstraint"),
            range: XSD_STRING,
        },
        // v0.2.1: PreludeExport metadata
        Property {
            id: "https://uor.foundation/conformance/exportsClass",
            label: "exportsClass",
            comment: "The OWL class IRI that the foundation crate's prelude \
                      module should re-export.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/PreludeExport"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/conformance/exportRustName",
            label: "exportRustName",
            comment: "The Rust identifier under which the prelude exposes \
                      this symbol. Codegen uses this when the class's \
                      generated Rust name differs from a desired prelude \
                      alias.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/conformance/PreludeExport"),
            range: XSD_STRING,
        },
    ]
}

fn individuals() -> Vec<Individual> {
    vec![
        // Amendment 84: CompileUnit admission shape
        Individual {
            id: "https://uor.foundation/conformance/CompileUnitShape",
            type_: "https://uor.foundation/conformance/Shape",
            label: "CompileUnitShape",
            comment: "Shape validating that a CompileUnit carries all required \
                      properties before reduction admission. The unitAddress \
                      property is NOT required \u{2014} it is computed by \
                      stage_initialization after shape validation passes.",
            properties: &[
                (
                    "https://uor.foundation/conformance/targetClass",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/CompileUnit",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/surfaceForm",
                    IndividualValue::Str("compile-unit-decl"),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/compileUnit_rootTerm_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/compileUnit_unitWittLevel_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/compileUnit_thermodynamicBudget_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/compileUnit_targetDomains_constraint",
                    ),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/compileUnit_rootTerm_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "compileUnit_rootTerm_constraint",
            comment: "Exactly one root term is required. Range is schema:Term.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/rootTerm",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "https://uor.foundation/schema/Term",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("root_term"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("program"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/compileUnit_unitWittLevel_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "compileUnit_unitWittLevel_constraint",
            comment: "Exactly one quantum level is required. Range is \
                      schema:WittLevel.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/unitWittLevel",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "https://uor.foundation/schema/WittLevel",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("witt_level_ceiling"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("name"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/compileUnit_thermodynamicBudget_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "compileUnit_thermodynamicBudget_constraint",
            comment: "Exactly one thermodynamic budget is required. Shape \
                      validates presence and type; the BudgetSolvencyCheck \
                      preflight validates the value against the Landauer bound.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/thermodynamicBudget",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "http://www.w3.org/2001/XMLSchema#decimal",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("thermodynamic_budget"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("decimal-literal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/compileUnit_targetDomains_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "compileUnit_targetDomains_constraint",
            comment: "At least one target verification domain is required. \
                      maxCount 0 means unbounded.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/reduction/targetDomains",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "https://uor.foundation/op/VerificationDomain",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("target_domains"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("domain-set"),
                ),
            ],
        },
        // ── Amendment 95: ViolationKind individuals (5) ──
        Individual {
            id: "https://uor.foundation/conformance/Missing",
            type_: "https://uor.foundation/conformance/ViolationKind",
            label: "Missing",
            comment: "Required property was not set on the builder.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/conformance/TypeMismatch",
            type_: "https://uor.foundation/conformance/ViolationKind",
            label: "TypeMismatch",
            comment: "Property was set but its value is not an instance \
                      of the constraintRange.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/conformance/CardinalityViolation",
            type_: "https://uor.foundation/conformance/ViolationKind",
            label: "CardinalityViolation",
            comment: "Cardinality violated: too few or too many values \
                      provided.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/conformance/ValueCheck",
            type_: "https://uor.foundation/conformance/ViolationKind",
            label: "ValueCheck",
            comment: "Value-dependent check failed (Tier 2). For example, \
                      thermodynamic budget insufficient for Landauer bound.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/conformance/LevelMismatch",
            type_: "https://uor.foundation/conformance/ViolationKind",
            label: "LevelMismatch",
            comment: "A term's quantum level annotation exceeds the \
                      CompileUnit ceiling, or binary operation operands \
                      are at different levels without an intervening \
                      lift or project.",
            properties: &[],
        },
        // ── v0.2.1: Surface-grammar metadata for the 6 conformance shapes
        // not previously decomposed into PropertyConstraint individuals.
        // Each shape carries a `surfaceForm` for the EBNF emitter and
        // requiredProperty links to its constraints. Each constraint
        // carries `surfaceKeyword` (the grammar literal) and
        // `surfaceProduction` (the value-slot non-terminal).
        //
        // ── DispatchShape ──
        Individual {
            id: "https://uor.foundation/conformance/DispatchShapeInstance",
            type_: "https://uor.foundation/conformance/Shape",
            label: "DispatchShapeInstance",
            comment: "Shape instance validating predicate:DispatchRule \
                      declarations against the dispatch-rule-decl grammar.",
            properties: &[
                (
                    "https://uor.foundation/conformance/targetClass",
                    IndividualValue::IriRef(
                        "https://uor.foundation/predicate/DispatchRule",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/surfaceForm",
                    IndividualValue::Str("dispatch-rule-decl"),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/dispatch_predicate_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/dispatch_target_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/dispatch_priority_constraint",
                    ),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/dispatch_predicate_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "dispatch_predicate_constraint",
            comment: "Exactly one predicate term selecting this dispatch \
                      rule.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/predicate/dispatchPredicate",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "https://uor.foundation/schema/Term",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("predicate"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("term"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/dispatch_target_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "dispatch_target_constraint",
            comment: "The resolver class invoked when the predicate holds.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/predicate/dispatchTarget",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "https://uor.foundation/resolver/Resolver",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("target_resolver"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("name"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/dispatch_priority_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "dispatch_priority_constraint",
            comment: "Non-negative integer evaluation order; lower values \
                      evaluate first.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/predicate/dispatchPriority",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("priority"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("integer-literal"),
                ),
            ],
        },
        // ── WittLevelShape ──
        Individual {
            id: "https://uor.foundation/conformance/WittLevelShapeInstance",
            type_: "https://uor.foundation/conformance/Shape",
            label: "WittLevelShapeInstance",
            comment: "Shape instance validating schema:WittLevel declarations \
                      against the witt-level-decl grammar.",
            properties: &[
                (
                    "https://uor.foundation/conformance/targetClass",
                    IndividualValue::IriRef(
                        "https://uor.foundation/schema/WittLevel",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/surfaceForm",
                    IndividualValue::Str("witt-level-decl"),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/wittLevel_bitWidth_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/wittLevel_cycleSize_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/wittLevel_predecessorLevel_constraint",
                    ),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/wittLevel_bitWidth_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "wittLevel_bitWidth_constraint",
            comment: "Bit width must equal 8\u{00b7}(k+1) for some \
                      non-negative integer k.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/schema/bitsWidth",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "http://www.w3.org/2001/XMLSchema#positiveInteger",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("bit_width"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("integer-literal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/wittLevel_cycleSize_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "wittLevel_cycleSize_constraint",
            comment: "Cycle size must equal 2^bit_width.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/schema/cycleSize",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("cycle_size"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("integer-literal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/wittLevel_predecessorLevel_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "wittLevel_predecessorLevel_constraint",
            comment: "The predecessor WittLevel individual whose nextWittLevel \
                      will be updated to point at this new level.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/schema/wittLevelPredecessor",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "https://uor.foundation/schema/WittLevel",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("predecessor_level"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("name"),
                ),
            ],
        },
        // ── PredicateShape ──
        Individual {
            id: "https://uor.foundation/conformance/PredicateShapeInstance",
            type_: "https://uor.foundation/conformance/Shape",
            label: "PredicateShapeInstance",
            comment: "Shape instance for predicate:Predicate declarations.",
            properties: &[
                (
                    "https://uor.foundation/conformance/targetClass",
                    IndividualValue::IriRef(
                        "https://uor.foundation/predicate/Predicate",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/surfaceForm",
                    IndividualValue::Str("predicate-decl"),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/predicate_inputType_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/predicate_evaluator_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/predicate_terminationWitness_constraint",
                    ),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/predicate_inputType_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "predicate_inputType_constraint",
            comment: "Input type the predicate evaluates over.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/predicate/evaluatesOver",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "https://uor.foundation/type/TypeDefinition",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("input_type"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("type-expr"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/predicate_evaluator_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "predicate_evaluator_constraint",
            comment: "The evaluator term producing a boolean.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/predicate/evaluatorTerm",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "https://uor.foundation/schema/Term",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("evaluator"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("term"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/predicate_terminationWitness_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "predicate_terminationWitness_constraint",
            comment: "IRI of a proof:Proof attesting that the evaluator \
                      halts on all inputs.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/predicate/terminationWitness",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "http://www.w3.org/2001/XMLSchema#string",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("termination_witness"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("string-literal"),
                ),
            ],
        },
        // ── ParallelShape ──
        Individual {
            id: "https://uor.foundation/conformance/ParallelShapeInstance",
            type_: "https://uor.foundation/conformance/Shape",
            label: "ParallelShapeInstance",
            comment: "Shape instance for parallel:ParallelProduct declarations.",
            properties: &[
                (
                    "https://uor.foundation/conformance/targetClass",
                    IndividualValue::IriRef(
                        "https://uor.foundation/parallel/ParallelProduct",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/surfaceForm",
                    IndividualValue::Str("parallel-decl"),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/parallel_sitePartition_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/parallel_disjointnessWitness_constraint",
                    ),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/parallel_sitePartition_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "parallel_sitePartition_constraint",
            comment: "The site partition this parallel product is over.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/parallel/sitePartition",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "https://uor.foundation/partition/Partition",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("site_partition"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("name"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/parallel_disjointnessWitness_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "parallel_disjointnessWitness_constraint",
            comment: "IRI of a proof of pairwise disjointness of the \
                      partition components.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/parallel/disjointnessWitness",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "http://www.w3.org/2001/XMLSchema#string",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("disjointness_witness"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("string-literal"),
                ),
            ],
        },
        // ── StreamShape ──
        Individual {
            id: "https://uor.foundation/conformance/StreamShapeInstance",
            type_: "https://uor.foundation/conformance/Shape",
            label: "StreamShapeInstance",
            comment: "Shape instance for stream:ProductiveStream declarations.",
            properties: &[
                (
                    "https://uor.foundation/conformance/targetClass",
                    IndividualValue::IriRef(
                        "https://uor.foundation/stream/ProductiveStream",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/surfaceForm",
                    IndividualValue::Str("stream-decl"),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/stream_unfoldSeed_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/stream_step_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/stream_productivityWitness_constraint",
                    ),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/stream_unfoldSeed_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "stream_unfoldSeed_constraint",
            comment: "Initial seed value from which the stream unfolds.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/stream/unfoldSeed",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "https://uor.foundation/schema/Term",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("unfold_seed"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("term"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/stream_step_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "stream_step_constraint",
            comment: "Function from current seed to (head, next_seed).",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/stream/stepTerm",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "https://uor.foundation/schema/Term",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("step"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("term"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/stream_productivityWitness_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "stream_productivityWitness_constraint",
            comment: "IRI of a proof of stream productivity (coinductive \
                      well-foundedness).",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/stream/productivityWitness",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "http://www.w3.org/2001/XMLSchema#string",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("productivity_witness"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("string-literal"),
                ),
            ],
        },
        // ── LeaseShape ──
        Individual {
            id: "https://uor.foundation/conformance/LeaseShapeInstance",
            type_: "https://uor.foundation/conformance/Shape",
            label: "LeaseShapeInstance",
            comment: "Shape instance for state:ContextLease declarations.",
            properties: &[
                (
                    "https://uor.foundation/conformance/targetClass",
                    IndividualValue::IriRef(
                        "https://uor.foundation/state/ContextLease",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/surfaceForm",
                    IndividualValue::Str("lease-decl"),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/lease_linearSite_constraint",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/requiredProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/lease_leaseScope_constraint",
                    ),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/lease_linearSite_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "lease_linearSite_constraint",
            comment: "Site coordinate allocated linearly by this lease.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/state/linearSite",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("linear_site"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("integer-literal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/lease_leaseScope_constraint",
            type_: "https://uor.foundation/conformance/PropertyConstraint",
            label: "lease_leaseScope_constraint",
            comment: "Lexical or session scope within which the lease is valid.",
            properties: &[
                (
                    "https://uor.foundation/conformance/constraintProperty",
                    IndividualValue::IriRef(
                        "https://uor.foundation/state/leaseScope",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/constraintRange",
                    IndividualValue::IriRef(
                        "http://www.w3.org/2001/XMLSchema#string",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/minCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/maxCount",
                    IndividualValue::Int(1),
                ),
                (
                    "https://uor.foundation/conformance/surfaceKeyword",
                    IndividualValue::Str("lease_scope"),
                ),
                (
                    "https://uor.foundation/conformance/surfaceProduction",
                    IndividualValue::Str("string-literal"),
                ),
            ],
        },
        // ── v0.2.1: PreludeExport facts. The Rust codegen reads these to
        // assemble foundation::prelude. Each individual carries the OWL
        // class IRI and (optionally) the Rust alias name. The macros uor!
        // and uor_ground! plus the Primitives trait are added by the
        // codegen as static line entries — they are not OWL classes.
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_Datum",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_Datum",
            comment: "Prelude re-export for schema:Datum.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef("https://uor.foundation/schema/Datum"),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_Term",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_Term",
            comment: "Prelude re-export for schema:Term.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef("https://uor.foundation/schema/Term"),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_WittLevel",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_WittLevel",
            comment: "Prelude re-export for schema:WittLevel.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef("https://uor.foundation/schema/WittLevel"),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_CompileUnit",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_CompileUnit",
            comment: "Prelude re-export for reduction:CompileUnit.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef("https://uor.foundation/reduction/CompileUnit"),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_CompileUnitBuilder",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_CompileUnitBuilder",
            comment: "Prelude re-export for conformance:CompileUnitBuilder.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef(
                    "https://uor.foundation/conformance/CompileUnitBuilder",
                ),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_ValidatedWrapper",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_ValidatedWrapper",
            comment: "Prelude re-export for conformance:ValidatedWrapper \
                      (exposed in Rust as `Validated`).",
            properties: &[
                (
                    "https://uor.foundation/conformance/exportsClass",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/ValidatedWrapper",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/exportRustName",
                    IndividualValue::Str("Validated"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_ShapeViolationReport",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_ShapeViolationReport",
            comment: "Prelude re-export for conformance:ShapeViolationReport.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef(
                    "https://uor.foundation/conformance/ShapeViolationReport",
                ),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_ValidationResult",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_ValidationResult",
            comment: "Prelude re-export for conformance:ValidationResult.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef(
                    "https://uor.foundation/conformance/ValidationResult",
                ),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_GroundingCertificate",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_GroundingCertificate",
            comment: "Prelude re-export for cert:GroundingCertificate.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef(
                    "https://uor.foundation/cert/GroundingCertificate",
                ),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_LiftChainCertificate",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_LiftChainCertificate",
            comment: "Prelude re-export for cert:LiftChainCertificate.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef(
                    "https://uor.foundation/cert/LiftChainCertificate",
                ),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_InhabitanceCertificate",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_InhabitanceCertificate",
            comment: "Prelude re-export for cert:InhabitanceCertificate (v0.2.1).",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef(
                    "https://uor.foundation/cert/InhabitanceCertificate",
                ),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_CompletenessCertificate",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_CompletenessCertificate",
            comment: "Prelude re-export for cert:CompletenessCertificate.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef(
                    "https://uor.foundation/cert/CompletenessCertificate",
                ),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_ConstrainedType",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_ConstrainedType",
            comment: "Prelude re-export for type:ConstrainedType.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef(
                    "https://uor.foundation/type/ConstrainedType",
                ),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_CompleteType",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_CompleteType",
            comment: "Prelude re-export for type:CompleteType.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef(
                    "https://uor.foundation/type/CompleteType",
                ),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_GroundedContext",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_GroundedContext",
            comment: "Prelude re-export for state:GroundedContext.",
            properties: &[(
                "https://uor.foundation/conformance/exportsClass",
                IndividualValue::IriRef(
                    "https://uor.foundation/state/GroundedContext",
                ),
            )],
        },
        Individual {
            id: "https://uor.foundation/conformance/preludeExport_TermArena",
            type_: "https://uor.foundation/conformance/PreludeExport",
            label: "preludeExport_TermArena",
            comment: "Prelude re-export for the foundation enforcement \
                      TermArena type. Backed by conformance:WitnessDatum \
                      since TermArena has no direct OWL class but is the \
                      term-storage container.",
            properties: &[
                (
                    "https://uor.foundation/conformance/exportsClass",
                    IndividualValue::IriRef(
                        "https://uor.foundation/conformance/WitnessDatum",
                    ),
                ),
                (
                    "https://uor.foundation/conformance/exportRustName",
                    IndividualValue::Str("TermArena"),
                ),
            ],
        },
    ]
}
