//! `resolver/` namespace — Type resolution strategies.
//!
//! Resolvers implement the map Π : T_n → Part(R_n), transforming a type
//! declaration into a partition of the ring. Resolution requests come from
//! user-space; execution occurs in the kernel.
//!
//! Amendment 11 adds iterative resolution: resolution state tracking,
//! refinement suggestions, and convergence metrics for the resolution-as-learning
//! loop.
//!
//! Amendment 25 adds `CompletenessResolver` — a specialised resolver that drives
//! the completeness certification loop for `CompletenessCandidate` instances.
//!
//! Amendment 26 adds `WittLevelResolver` — a resolver parameterised by quantum
//! level, allowing the same strategy to run at Q0, Q1, or any future level.
//!
//! Amendment 27 adds `SessionResolver` — a resolver that maintains a
//! `BindingAccumulator` across multiple `RelationQuery` evaluations for
//! multi-turn Prism deployments.
//!
//! Amendment 48 adds `ExecutionPolicy` and `ExecutionPolicyKind` — a typed
//! controlled vocabulary for scheduling strategies (FifoPolicy,
//! MinFreeCountFirst, MaxFreeCountFirst, DisjointFirst) bound to a
//! SessionResolver.
//!
//! **Space classification:** `bridge` — user-requested, kernel-executed.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `resolver/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "resolver",
            iri: NS_RESOLVER,
            label: "UOR Resolvers",
            comment: "Type resolution strategies implementing the partition map \
                      Π : T_n → Part(R_n). Resolvers transform type declarations \
                      into ring partitions.",
            space: Space::Bridge,
            imports: &[NS_SCHEMA, NS_QUERY, NS_PARTITION, NS_TYPE, NS_PREDICATE],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/resolver/Resolver",
            label: "Resolver",
            comment: "A strategy for resolving a type declaration into a partition \
                      of the ring. The kernel dispatches to a specific resolver \
                      based on the type's structure.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/resolver/ResolutionState",
                "https://uor.foundation/resolver/RefinementSuggestion",
            ],
        },
        Class {
            id: "https://uor.foundation/resolver/DihedralFactorizationResolver",
            label: "DihedralFactorizationResolver",
            comment: "Resolves types by factoring the ring under dihedral group \
                      action. Identifies orbits under D_{2^n} to determine \
                      irreducibility boundaries.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/resolver/CanonicalFormResolver",
            label: "CanonicalFormResolver",
            comment: "Resolves types by computing canonical forms via term rewriting. \
                      Applies the critical identity and normalization rules to \
                      reduce terms to unique canonical representatives.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/resolver/EvaluationResolver",
            label: "EvaluationResolver",
            comment: "Resolves types by direct evaluation: applies operations to \
                      enumerate ring elements and classify them as irreducible, \
                      reducible, unit, or exterior.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // Amendment 11: Iterative Resolution classes
        Class {
            id: "https://uor.foundation/resolver/ResolutionState",
            label: "ResolutionState",
            comment: "The current state of an iterative resolution: tracks how many \
                      iterations have been performed, whether the resolution is \
                      complete, and the current site deficit.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/resolver/Resolver",
                "https://uor.foundation/resolver/RefinementSuggestion",
            ],
        },
        Class {
            id: "https://uor.foundation/resolver/RefinementSuggestion",
            label: "RefinementSuggestion",
            comment: "A suggestion from the resolver for how to refine an incomplete \
                      resolution: which metric axis to explore, which class to narrow \
                      to, and which sites to target.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/resolver/Resolver",
                "https://uor.foundation/resolver/ResolutionState",
            ],
        },
        // Amendment 23: Typed controlled vocabulary class
        Class {
            id: "https://uor.foundation/resolver/ComplexityClass",
            label: "ComplexityClass",
            comment: "A computational complexity classification for resolvers. \
                      Each resolver's asymptotic runtime is typed as a named \
                      ComplexityClass individual rather than a free string.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 18: Constraint Nerve
        Class {
            id: "https://uor.foundation/resolver/CechNerve",
            label: "CechNerve",
            comment: "The simplicial complex whose vertices are constraints and \
                      where a k-simplex exists iff the corresponding k+1 constraints \
                      have nonempty intersection. The nerve's topology governs \
                      resolution convergence: trivial homology ↔ smooth convergence, \
                      non-trivial homology ↔ potential stalls.",
            subclass_of: &[
                OWL_THING,
                "https://uor.foundation/homology/SimplicialComplex",
                "https://uor.foundation/homology/KanComplex",
            ],
            disjoint_with: &[],
        },
        // Amendment 25: Completeness Certification Pathway
        Class {
            id: "https://uor.foundation/resolver/CompletenessResolver",
            label: "CompletenessResolver",
            comment: "A specialisation of Resolver driving the completeness certification \
                      loop. Accepts a CompletenessCandidate, runs the ψ-pipeline (reading \
                      nerveEulerCharacteristic from ResolutionState), and either issues a \
                      CompletenessCertificate or produces a RefinementSuggestion.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // Amendment 26: Quantum Level Scaling
        Class {
            id: "https://uor.foundation/resolver/WittLevelResolver",
            label: "WittLevelResolver",
            comment: "A Resolver parameterised by quantum level. The same resolver \
                      strategy runs at any quantum level n ≥ 1 by substituting the \
                      appropriate R_n ring.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // Amendment 27: Session-Scoped Resolution
        Class {
            id: "https://uor.foundation/resolver/SessionResolver",
            label: "SessionResolver",
            comment: "A Resolver that maintains a BindingAccumulator across multiple \
                      RelationQuery evaluations. The top-level resolver for multi-turn \
                      Prism deployments.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // Amendment 28: ψ-Pipeline Inversion (Type Synthesis)
        Class {
            id: "https://uor.foundation/resolver/TypeSynthesisResolver",
            label: "TypeSynthesisResolver",
            comment: "A Resolver that runs the ψ-pipeline in inverse mode. Accepts a \
                      TypeSynthesisGoal and returns a TypeSynthesisResult. Internally \
                      maintains a ConstraintSearchState tracking which constraint \
                      combinations have been explored and which Betti profiles they realise.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/resolver/ConstraintSearchState",
            label: "ConstraintSearchState",
            comment: "Internal resolver state tracking the boundary of explored constraint \
                      combinations during synthesis. Carries exploredCount, currentCandidate, \
                      and a link to the best SynthesisSignature achieved so far.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 29: Quantum Level Spectral Sequence
        Class {
            id: "https://uor.foundation/resolver/IncrementalCompletenessResolver",
            label: "IncrementalCompletenessResolver",
            comment: "A Resolver that determines whether a CompleteType T at Q_n lifts to a \
                      CompleteType at Q_{n+1} without re-running the full ψ-pipeline from \
                      scratch. It computes the SpectralSequencePage sequence, reads the \
                      LiftObstruction, and either confirms the lift or returns a \
                      LiftRefinementSuggestion.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/resolver/LiftRefinementSuggestion",
            label: "LiftRefinementSuggestion",
            comment: "A RefinementSuggestion produced when a WittLift has a non-trivial \
                      LiftObstruction. Specialises RefinementSuggestion with liftSitePosition \
                      (the new bit position n+1) and obstructionClass.",
            subclass_of: &["https://uor.foundation/resolver/RefinementSuggestion"],
            disjoint_with: &[],
        },
        // Amendment 30: Monodromy Observables
        Class {
            id: "https://uor.foundation/resolver/MonodromyResolver",
            label: "MonodromyResolver",
            comment: "A Resolver that computes the HolonomyGroup of a ConstrainedType by \
                      enumerating closed paths in the constraint nerve and accumulating \
                      DihedralElement values. Outputs a MonodromyClass and classifies the \
                      type as FlatType or TwistedType.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // Amendment 31: Jacobian-guided resolver (DC_10)
        Class {
            id: "https://uor.foundation/resolver/JacobianGuidedResolver",
            label: "JacobianGuidedResolver",
            comment: "A resolver that uses the Jacobian matrix to guide \
                      constraint selection, implementing DC_10: select the \
                      constraint that maximises total curvature reduction.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // Amendment 32: Superposition resolver (RC_5)
        Class {
            id: "https://uor.foundation/resolver/SuperpositionResolver",
            label: "SuperpositionResolver",
            comment: "A resolver that handles superposed site states, computing \
                      amplitudes and determining when superposition collapses to \
                      a classical site assignment (Amendment 32).",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // Amendment 33: Grounding-Aware Resolution
        Class {
            id: "https://uor.foundation/resolver/GroundingAwareResolver",
            label: "GroundingAwareResolver",
            comment: "A resolver that exploits accumulated session bindings at full \
                      saturation (σ = 1) to provide O(1) resolution via direct \
                      coordinate reads (SC_5).",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // Amendment 35: Geodesic Validation
        Class {
            id: "https://uor.foundation/resolver/GeodesicValidator",
            label: "GeodesicValidator",
            comment: "A resolver that validates whether a ComputationTrace satisfies \
                      the dual geodesic condition (AR_1-ordered and DC_10-selected). \
                      Produces GeodesicViolation individuals on failure.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // Amendment 36: Measurement Resolution
        Class {
            id: "https://uor.foundation/resolver/MeasurementResolver",
            label: "MeasurementResolver",
            comment: "A resolver that handles projective collapse of \
                      SuperposedSiteState components. Issues \
                      MeasurementCertificate upon successful collapse with \
                      QM_1 verification.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // Amendment 41: Tower Completeness Resolver
        Class {
            id: "https://uor.foundation/resolver/TowerCompletenessResolver",
            label: "TowerCompletenessResolver",
            comment: "A Resolver that constructs a LiftChain from liftSourceLevel \
                      to an arbitrary liftTargetLevel Q_k by iterating \
                      IncrementalCompletenessResolver step by step.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // v0.2.1: Inhabitance Verdict Instantiation resolvers
        Class {
            id: "https://uor.foundation/resolver/InhabitanceResolver",
            label: "InhabitanceResolver",
            comment: "A Resolver whose dispatch is governed by a new \
                      predicate:InhabitanceDispatchTable. Returns either a \
                      cert:InhabitanceCertificate (verified true with witness) \
                      or a proof:InhabitanceImpossibilityWitness (verified \
                      false with contradiction proof). Inherits the \
                      dual-output termination discipline from \
                      resolver:TypeSynthesisResolver.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/resolver/TwoSatDecider",
            label: "TwoSatDecider",
            comment: "A Resolver target that decides carrier non-emptiness on \
                      ConstrainedType instances whose constraint nerve \
                      contains only disjunctions of width \u{2264} 2, via \
                      classical 2-SAT in O(n+m). Dispatch rule 1 of the \
                      InhabitanceDispatchTable.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/resolver/HornSatDecider",
            label: "HornSatDecider",
            comment: "A Resolver target that decides carrier non-emptiness on \
                      ConstrainedType instances whose disjunctions each \
                      contain at most one positive literal, via classical \
                      Horn-SAT unit propagation in O(n+m). Dispatch rule 2 \
                      of the InhabitanceDispatchTable.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/resolver/ResidualVerdictResolver",
            label: "ResidualVerdictResolver",
            comment: "A Resolver target for the catch-all default dispatch \
                      rule. Returns the residual-hard verdict without \
                      promising a polynomial bound; the verdict is \
                      well-formed but the cost identity is unbounded. \
                      Dispatch rule 3 of the InhabitanceDispatchTable \
                      ensuring total coverage (reduction:DispatchMiss is \
                      unreachable for this table).",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // v0.2.2 Phase C.4 — MultiplicationResolver.
        Class {
            id: "https://uor.foundation/resolver/MultiplicationResolver",
            label: "MultiplicationResolver",
            comment: "A Resolver target that decides the cost-optimal Toom-Cook \
                      splitting factor R for a Datum<L> \u{00d7} Datum<L> \
                      multiplication at a given call-site context (stack budget \
                      linear:stackBudgetBytes, const-eval regime). The decision \
                      procedure is a pure derivation over a closed-form Landauer \
                      cost function grounded in op:OA_5: for each admissible R, \
                      the cost is (2R - 1) \u{00b7} (N/R)\u{00b2} \u{00b7} \
                      64 \u{00b7} ln 2 nats (R > 1) or N\u{00b2} \u{00b7} 64 \
                      \u{00b7} ln 2 nats (R = 1). The resolver picks the \
                      cost-minimum R subject to stack-budget and const-eval \
                      constraints and returns a cert:MultiplicationCertificate \
                      recording the choice.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // v0.2.1: parametric Certify-mapping metadata.
        // One CertifyMapping individual per resolver class encodes the
        // (resolver -> certificate, witness) triple. Codegen reads these
        // individuals to drive Certify trait impl emission, so adding a
        // new resolver requires only an ontology edit.
        Class {
            id: "https://uor.foundation/resolver/CertifyMapping",
            label: "CertifyMapping",
            comment: "An ontology fact recording that a resolver:Resolver \
                      subclass produces a specific cert:Certificate \
                      subclass on success and a specific \
                      proof:ImpossibilityWitness subclass on failure. The \
                      v0.2.1 Rust codegen reads CertifyMapping individuals \
                      to emit foundation::Certify trait impls for each \
                      resolver class, keeping the mapping data-driven \
                      rather than hand-tabulated in source.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 48: Multi-Session Coordination classes
        Class {
            id: "https://uor.foundation/resolver/ExecutionPolicy",
            label: "ExecutionPolicy",
            comment: "A strategy class that defines how a SessionResolver orders \
                      pending RelationQuery instances for dispatch. The policy \
                      reads the targetSite.freeRank of each pending query and \
                      applies an ordering function.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[
                "https://uor.foundation/resolver/Resolver",
                "https://uor.foundation/resolver/ResolutionState",
                "https://uor.foundation/resolver/RefinementSuggestion",
            ],
        },
        Class {
            id: "https://uor.foundation/resolver/ExecutionPolicyKind",
            label: "ExecutionPolicyKind",
            comment: "A typed controlled vocabulary for ExecutionPolicy \
                      individuals. Follows the SessionBoundaryType pattern: \
                      a single class with named individuals rather than a \
                      subclass hierarchy.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 55: Homotopy Pipeline
        Class {
            id: "https://uor.foundation/resolver/HomotopyResolver",
            label: "HomotopyResolver",
            comment: "A resolver that runs the extended \u{03c8}-pipeline \
                      (\u{03c8}_7\u{2013}\u{03c8}_9) to compute the full homotopy \
                      type of a CechNerve. Returns HomotopyGroup observables \
                      and PostnikovTruncation records.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
        // Amendment 57: Moduli Resolver
        Class {
            id: "https://uor.foundation/resolver/ModuliResolver",
            label: "ModuliResolver",
            comment: "A resolver that computes the local structure of the moduli \
                      space at a given CompleteType: constructs the \
                      DeformationComplex, determines the HolonomyStratum, and \
                      computes tangent/obstruction dimensions.",
            subclass_of: &["https://uor.foundation/resolver/Resolver"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/resolver/inputType",
            label: "inputType",
            comment: "The type of input this resolver accepts.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/Resolver"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/resolver/outputType",
            label: "outputType",
            comment: "The type of output this resolver produces. For all UOR \
                      resolvers, the output is a partition:Partition.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/Resolver"),
            range: "https://uor.foundation/partition/Partition",
        },
        Property {
            id: "https://uor.foundation/resolver/strategy",
            label: "strategy",
            comment: "A human-readable description of the resolution strategy \
                      this resolver implements.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/Resolver"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // complexity property removed by Amendment 23 (replaced by hasComplexityClass)
        // Amendment 11: Iterative Resolution properties
        Property {
            id: "https://uor.foundation/resolver/resolutionState",
            label: "resolutionState",
            comment: "The current resolution state of this resolver.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/Resolver"),
            range: "https://uor.foundation/resolver/ResolutionState",
        },
        Property {
            id: "https://uor.foundation/resolver/isComplete",
            label: "isComplete",
            comment: "Whether this resolution is complete: all sites are pinned \
                      and the partition is fully determined.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ResolutionState"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/resolver/iterationCount",
            label: "iterationCount",
            comment: "The number of refinement iterations performed so far.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ResolutionState"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/resolver/siteDeficit",
            label: "siteDeficit",
            comment: "The site budget showing the remaining unpinned sites. \
                      When all sites are pinned, the deficit is zero and \
                      resolution is complete.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ResolutionState"),
            range: "https://uor.foundation/partition/FreeRank",
        },
        Property {
            id: "https://uor.foundation/resolver/suggestion",
            label: "suggestion",
            comment: "A refinement suggestion for advancing this resolution.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/resolver/ResolutionState"),
            range: "https://uor.foundation/resolver/RefinementSuggestion",
        },
        Property {
            id: "https://uor.foundation/resolver/suggestedAxis",
            label: "suggestedAxis",
            comment: "The metric axis this suggestion recommends exploring.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/RefinementSuggestion"),
            range: "https://uor.foundation/type/MetricAxis",
        },
        Property {
            id: "https://uor.foundation/resolver/suggestedClass",
            label: "suggestedClass",
            comment: "The constraint class this suggestion recommends applying.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/RefinementSuggestion"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/resolver/targetSites",
            label: "targetSites",
            comment: "The site coordinates this suggestion targets for pinning.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/resolver/RefinementSuggestion"),
            range: "https://uor.foundation/partition/SiteIndex",
        },
        Property {
            id: "https://uor.foundation/resolver/convergenceRate",
            label: "convergenceRate",
            comment: "The rate at which sites are being pinned per iteration. \
                      A higher rate indicates faster convergence toward a \
                      complete resolution.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ResolutionState"),
            range: XSD_DECIMAL,
        },
        // Amendment 23: Typed controlled vocabulary property
        Property {
            id: "https://uor.foundation/resolver/hasComplexityClass",
            label: "hasComplexityClass",
            comment: "The computational complexity class of this resolver. \
                      Replaces the string-valued resolver:complexity property.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/Resolver"),
            range: "https://uor.foundation/resolver/ComplexityClass",
        },
        // Amendment 18: Analytical resolver properties
        Property {
            id: "https://uor.foundation/resolver/cechNerve",
            label: "cechNerve",
            comment: "The constraint nerve associated with this resolution state.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ResolutionState"),
            range: "https://uor.foundation/resolver/CechNerve",
        },
        Property {
            id: "https://uor.foundation/resolver/residualEntropy",
            label: "residualEntropy",
            comment: "The residual Shannon entropy of the resolution state: \
                      S = freeRank × ln 2. Measures remaining uncertainty.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ResolutionState"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/resolver/topologicallyComplete",
            label: "topologicallyComplete",
            comment: "Whether all Betti numbers of the constraint nerve are zero, \
                      indicating no topological obstructions to resolution.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ResolutionState"),
            range: XSD_BOOLEAN,
        },
        // Gap B: Euler characteristic cache
        Property {
            id: "https://uor.foundation/resolver/nerveEulerCharacteristic",
            label: "nerveEulerCharacteristic",
            comment: "The Euler characteristic χ(N(C)) of the active constraint nerve at this \
                      resolution state. IT_7d requires this value to equal n (the quantum level) \
                      for resolution to be complete. Cached here to avoid recomputing the full \
                      ψ pipeline on each iteration check.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ResolutionState"),
            range: XSD_INTEGER,
        },
        // Amendment 25: Completeness Certification Pathway property
        Property {
            id: "https://uor.foundation/resolver/completenessTarget",
            label: "completenessTarget",
            comment: "The CompletenessCandidate this resolver is certifying.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/CompletenessResolver"),
            range: "https://uor.foundation/type/CompletenessCandidate",
        },
        // Amendment 26: Quantum Level Scaling property
        Property {
            id: "https://uor.foundation/resolver/quantumLevel",
            label: "quantumLevel",
            comment: "The quantum level this resolver instance is configured for.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/WittLevelResolver"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        // Amendment 27: Session-Scoped Resolution property
        Property {
            id: "https://uor.foundation/resolver/sessionAccumulator",
            label: "sessionAccumulator",
            comment: "The BindingAccumulator this session resolver maintains across \
                      multiple RelationQuery evaluations.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/SessionResolver"),
            range: "https://uor.foundation/state/BindingAccumulator",
        },
        // Amendment 28: TypeSynthesisResolver and ConstraintSearchState properties
        Property {
            id: "https://uor.foundation/resolver/synthesisGoal",
            label: "synthesisGoal",
            comment: "The goal this type synthesis resolver is working to achieve.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/TypeSynthesisResolver"),
            range: "https://uor.foundation/type/TypeSynthesisGoal",
        },
        Property {
            id: "https://uor.foundation/resolver/exploredCount",
            label: "exploredCount",
            comment: "Number of constraint combinations evaluated so far during synthesis.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ConstraintSearchState"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/resolver/currentCandidate",
            label: "currentCandidate",
            comment: "The type candidate currently being evaluated during synthesis.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ConstraintSearchState"),
            range: "https://uor.foundation/type/ConstrainedType",
        },
        // Amendment 29: IncrementalCompletenessResolver and LiftRefinementSuggestion properties
        Property {
            id: "https://uor.foundation/resolver/liftTarget",
            label: "liftTarget",
            comment: "The WittLift this incremental completeness resolver is evaluating.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/IncrementalCompletenessResolver"),
            range: "https://uor.foundation/type/WittLift",
        },
        Property {
            id: "https://uor.foundation/resolver/liftSitePosition",
            label: "liftSitePosition",
            comment: "The new site position at Q_{n+1} that the lift refinement suggestion \
                      targets.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/LiftRefinementSuggestion"),
            range: "https://uor.foundation/partition/SiteIndex",
        },
        Property {
            id: "https://uor.foundation/resolver/obstructionClass",
            label: "obstructionClass",
            comment: "The obstruction class this lift refinement suggestion is designed to kill.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/LiftRefinementSuggestion"),
            range: "https://uor.foundation/observable/LiftObstructionClass",
        },
        // Amendment 30: MonodromyResolver properties
        Property {
            id: "https://uor.foundation/resolver/monodromyTarget",
            label: "monodromyTarget",
            comment: "The type whose holonomy this monodromy resolver is computing.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/MonodromyResolver"),
            range: "https://uor.foundation/type/ConstrainedType",
        },
        Property {
            id: "https://uor.foundation/resolver/holonomyResult",
            label: "holonomyResult",
            comment: "The HolonomyGroup produced by this monodromy resolver run.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/MonodromyResolver"),
            range: "https://uor.foundation/observable/HolonomyGroup",
        },
        // Amendment 31: Jacobian-guided resolver property (DC_10)
        Property {
            id: "https://uor.foundation/resolver/guidingJacobian",
            label: "guidingJacobian",
            comment: "The Jacobian observable guiding constraint selection \
                      at this resolution state (DC_10).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ResolutionState"),
            range: "https://uor.foundation/observable/Jacobian",
        },
        // Amendment 33: Grounding-Aware Resolution
        Property {
            id: "https://uor.foundation/resolver/usedGrounding",
            label: "usedGrounding",
            comment: "Whether this resolver used the saturation shortcut (SC_5) \
                      to bypass the ψ-pipeline and return a direct coordinate read.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/GroundingAwareResolver"),
            range: XSD_BOOLEAN,
        },
        // Amendment 35: Geodesic Validation
        Property {
            id: "https://uor.foundation/resolver/validateGeodesic",
            label: "validateGeodesic",
            comment: "The GeodesicTrace being validated by this GeodesicValidator.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/GeodesicValidator"),
            range: "https://uor.foundation/trace/GeodesicTrace",
        },
        // Amendment 36: Measurement Resolution
        Property {
            id: "https://uor.foundation/resolver/collapseAmplitude",
            label: "collapseAmplitude",
            comment: "The amplitude of the SuperposedSiteState prior to projective \
                      collapse by this MeasurementResolver.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/MeasurementResolver"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/resolver/collapsedSite",
            label: "collapsedSite",
            comment: "The site index that was collapsed (pinned to a classical \
                      value) by the projective measurement.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/MeasurementResolver"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/resolver/measurementOutcome",
            label: "measurementOutcome",
            comment: "The classical value obtained from the projective collapse. \
                      Either 0 or 1 for a single-site measurement.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/MeasurementResolver"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 37: Amplitude vector for SuperpositionResolver (Gap 1)
        Property {
            id: "https://uor.foundation/resolver/amplitudeVector",
            label: "amplitudeVector",
            comment: "The amplitude vector of all branches maintained by this \
                      SuperpositionResolver during ψ-pipeline traversal. \
                      Encoded as a comma-separated list of decimal amplitudes. \
                      Must satisfy Σ|αᵢ|² = 1 (QM_5) after normalization.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/SuperpositionResolver"),
            range: XSD_DECIMAL,
        },
        // Amendment 37: Prior amplitude vector for MeasurementResolver (Gap 10)
        Property {
            id: "https://uor.foundation/resolver/priorAmplitudeVector",
            label: "priorAmplitudeVector",
            comment: "The full vector of all branch amplitudes before projective \
                      collapse. Recorded by the MeasurementResolver to enable \
                      Born rule verification (QM_5): P(outcome k) = |α_k|².",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/MeasurementResolver"),
            range: XSD_DECIMAL,
        },
        // Amendment 41: TowerCompletenessResolver properties
        Property {
            id: "https://uor.foundation/resolver/towerSourceLevel",
            label: "towerSourceLevel",
            comment: "The level at which the tower starts.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/TowerCompletenessResolver"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/resolver/towerTargetLevel",
            label: "towerTargetLevel",
            comment: "The level to which the tower is being built.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/TowerCompletenessResolver"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/resolver/currentChain",
            label: "currentChain",
            comment: "The LiftChain under construction.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/TowerCompletenessResolver"),
            range: "https://uor.foundation/type/LiftChain",
        },
        Property {
            id: "https://uor.foundation/resolver/towerStepResolver",
            label: "towerStepResolver",
            comment: "The IncrementalCompletenessResolver used for each single-step \
                      lift.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/TowerCompletenessResolver"),
            range: "https://uor.foundation/resolver/IncrementalCompletenessResolver",
        },
        // Amendment 48: Multi-Session Coordination
        Property {
            id: "https://uor.foundation/resolver/executionPolicy",
            label: "executionPolicy",
            comment: "The ordering strategy this resolver applies to pending \
                      queries. Defaults to FifoPolicy if unset.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/SessionResolver"),
            range: "https://uor.foundation/resolver/ExecutionPolicy",
        },
        // Amendment 55: HomotopyResolver properties
        Property {
            id: "https://uor.foundation/resolver/homotopyTarget",
            label: "homotopyTarget",
            comment: "The CechNerve whose homotopy type this resolver computes.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/HomotopyResolver"),
            range: "https://uor.foundation/resolver/CechNerve",
        },
        Property {
            id: "https://uor.foundation/resolver/homotopyResult",
            label: "homotopyResult",
            comment: "A HomotopyGroup observable produced by this resolver.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/resolver/HomotopyResolver"),
            range: "https://uor.foundation/observable/HomotopyGroup",
        },
        // Amendment 57: ModuliResolver properties
        Property {
            id: "https://uor.foundation/resolver/moduliTarget",
            label: "moduliTarget",
            comment: "The CompleteType whose local moduli structure this resolver computes.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ModuliResolver"),
            range: "https://uor.foundation/type/CompleteType",
        },
        Property {
            id: "https://uor.foundation/resolver/moduliDeformation",
            label: "moduliDeformation",
            comment: "The DeformationComplex constructed by this resolver.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/ModuliResolver"),
            range: "https://uor.foundation/homology/DeformationComplex",
        },
        // Amendment 73: Dispatch formalization
        Property {
            id: "https://uor.foundation/resolver/dispatchTable",
            label: "dispatchTable",
            comment: "The dispatch table governing resolver selection for \
                      this resolver class.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/Resolver"),
            range: "https://uor.foundation/predicate/DispatchTable",
        },
        Property {
            id: "https://uor.foundation/resolver/resolverPredicate",
            label: "resolverPredicate",
            comment: "The predicate that selects this specific resolver. \
                      When the predicate evaluates to true on an input type, \
                      this resolver is chosen.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/Resolver"),
            range: "https://uor.foundation/predicate/TypePredicate",
        },
        // v0.2.1: CertifyMapping properties (parametric Certify metadata)
        Property {
            id: "https://uor.foundation/resolver/forResolver",
            label: "forResolver",
            comment: "The resolver:Resolver subclass this CertifyMapping \
                      describes.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/CertifyMapping"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/resolver/producesCertificate",
            label: "producesCertificate",
            comment: "The cert:Certificate (or proof:ComputationCertificate) \
                      subclass this resolver produces on success.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/CertifyMapping"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/resolver/producesWitness",
            label: "producesWitness",
            comment: "The proof:ImpossibilityWitness subclass this resolver \
                      produces on failure.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/resolver/CertifyMapping"),
            range: OWL_CLASS,
        },
    ]
}

// Amendment 23: Typed controlled vocabulary individuals
fn individuals() -> Vec<Individual> {
    vec![
        Individual {
            id: "https://uor.foundation/resolver/ConstantTime",
            type_: "https://uor.foundation/resolver/ComplexityClass",
            label: "ConstantTime",
            comment: "O(1) complexity — the resolver runs in constant time \
                      regardless of ring size.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/resolver/LogarithmicTime",
            type_: "https://uor.foundation/resolver/ComplexityClass",
            label: "LogarithmicTime",
            comment: "O(log n) complexity — the resolver runs in logarithmic time \
                      in the quantum level.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/resolver/LinearTime",
            type_: "https://uor.foundation/resolver/ComplexityClass",
            label: "LinearTime",
            comment: "O(n) complexity — the resolver runs in time linear in the \
                      quantum level.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/resolver/ExponentialTime",
            type_: "https://uor.foundation/resolver/ComplexityClass",
            label: "ExponentialTime",
            comment: "O(2^n) complexity — the resolver runs in time exponential \
                      in the quantum level.",
            properties: &[],
        },
        // Amendment 48: ExecutionPolicyKind vocabulary
        Individual {
            id: "https://uor.foundation/resolver/FifoPolicy",
            type_: "https://uor.foundation/resolver/ExecutionPolicyKind",
            label: "FifoPolicy",
            comment: "Process queries in arrival order. The implicit \
                      pre-Amendment 48 behavior.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/resolver/MinFreeCountFirst",
            type_: "https://uor.foundation/resolver/ExecutionPolicyKind",
            label: "MinFreeCountFirst",
            comment: "Process the query with the smallest targetSite.freeRank \
                      first. Favors cheapest resolutions, accelerating early \
                      grounding gain.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/resolver/MaxFreeCountFirst",
            type_: "https://uor.foundation/resolver/ExecutionPolicyKind",
            label: "MaxFreeCountFirst",
            comment: "Process the query with the largest targetSite.freeRank \
                      first. Favors hardest resolutions, maximizing information \
                      gain per step.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/resolver/DisjointFirst",
            type_: "https://uor.foundation/resolver/ExecutionPolicyKind",
            label: "DisjointFirst",
            comment: "Process queries whose targetSite is disjoint from all \
                      other pending queries' site sets first. Minimizes \
                      contention when operating against a SharedContext.",
            properties: &[],
        },
        // v0.2.1: CertifyMapping facts (parametric Certify metadata)
        Individual {
            id: "https://uor.foundation/resolver/towerCertifyMapping",
            type_: "https://uor.foundation/resolver/CertifyMapping",
            label: "towerCertifyMapping",
            comment: "TowerCompletenessResolver produces LiftChainCertificate \
                      on success and ImpossibilityWitness on failure.",
            properties: &[
                (
                    "https://uor.foundation/resolver/forResolver",
                    IndividualValue::IriRef(
                        "https://uor.foundation/resolver/TowerCompletenessResolver",
                    ),
                ),
                (
                    "https://uor.foundation/resolver/producesCertificate",
                    IndividualValue::IriRef("https://uor.foundation/cert/LiftChainCertificate"),
                ),
                (
                    "https://uor.foundation/resolver/producesWitness",
                    IndividualValue::IriRef("https://uor.foundation/proof/ImpossibilityWitness"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/resolver/incrementalCertifyMapping",
            type_: "https://uor.foundation/resolver/CertifyMapping",
            label: "incrementalCertifyMapping",
            comment: "IncrementalCompletenessResolver produces \
                      LiftChainCertificate (single-step) on success and \
                      ImpossibilityWitness on failure.",
            properties: &[
                (
                    "https://uor.foundation/resolver/forResolver",
                    IndividualValue::IriRef(
                        "https://uor.foundation/resolver/IncrementalCompletenessResolver",
                    ),
                ),
                (
                    "https://uor.foundation/resolver/producesCertificate",
                    IndividualValue::IriRef("https://uor.foundation/cert/LiftChainCertificate"),
                ),
                (
                    "https://uor.foundation/resolver/producesWitness",
                    IndividualValue::IriRef("https://uor.foundation/proof/ImpossibilityWitness"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/resolver/groundingAwareCertifyMapping",
            type_: "https://uor.foundation/resolver/CertifyMapping",
            label: "groundingAwareCertifyMapping",
            comment: "GroundingAwareResolver produces GroundingCertificate \
                      on success and ImpossibilityWitness on failure.",
            properties: &[
                (
                    "https://uor.foundation/resolver/forResolver",
                    IndividualValue::IriRef(
                        "https://uor.foundation/resolver/GroundingAwareResolver",
                    ),
                ),
                (
                    "https://uor.foundation/resolver/producesCertificate",
                    IndividualValue::IriRef("https://uor.foundation/cert/GroundingCertificate"),
                ),
                (
                    "https://uor.foundation/resolver/producesWitness",
                    IndividualValue::IriRef("https://uor.foundation/proof/ImpossibilityWitness"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/resolver/inhabitanceCertifyMapping",
            type_: "https://uor.foundation/resolver/CertifyMapping",
            label: "inhabitanceCertifyMapping",
            comment: "InhabitanceResolver produces InhabitanceCertificate \
                      on success and InhabitanceImpossibilityWitness on \
                      failure.",
            properties: &[
                (
                    "https://uor.foundation/resolver/forResolver",
                    IndividualValue::IriRef("https://uor.foundation/resolver/InhabitanceResolver"),
                ),
                (
                    "https://uor.foundation/resolver/producesCertificate",
                    IndividualValue::IriRef("https://uor.foundation/cert/InhabitanceCertificate"),
                ),
                (
                    "https://uor.foundation/resolver/producesWitness",
                    IndividualValue::IriRef(
                        "https://uor.foundation/proof/InhabitanceImpossibilityWitness",
                    ),
                ),
            ],
        },
        // v0.2.2 Phase C.4 — MultiplicationResolver CertifyMapping.
        Individual {
            id: "https://uor.foundation/resolver/multiplicationCertifyMapping",
            type_: "https://uor.foundation/resolver/CertifyMapping",
            label: "multiplicationCertifyMapping",
            comment: "MultiplicationResolver produces MultiplicationCertificate \
                      on success and ImpossibilityWitness on failure. The \
                      resolver is total over admissible call-site contexts \
                      (stack_budget_bytes > 0), so failure is unreachable for \
                      well-formed inputs.",
            properties: &[
                (
                    "https://uor.foundation/resolver/forResolver",
                    IndividualValue::IriRef(
                        "https://uor.foundation/resolver/MultiplicationResolver",
                    ),
                ),
                (
                    "https://uor.foundation/resolver/producesCertificate",
                    IndividualValue::IriRef(
                        "https://uor.foundation/cert/MultiplicationCertificate",
                    ),
                ),
                (
                    "https://uor.foundation/resolver/producesWitness",
                    IndividualValue::IriRef("https://uor.foundation/proof/ImpossibilityWitness"),
                ),
            ],
        },
    ]
}
