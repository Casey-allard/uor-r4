//! `op/` namespace — Ring operations, involutions, identities, and the dihedral group.
//!
//! The `op/` namespace defines the primitive operations of the UOR kernel:
//! - **Amendments 1**: 10 named operation individuals
//! - **Amendment 3**: `op:Identity` class and `op:criticalIdentity` individual
//! - **Amendment 4**: `op:Group`, `op:DihedralGroup` classes and `op:D2n` individual
//! - **Amendment 25**: 5 `CC_` identity individuals (completeness certification algebra)
//! - **Amendment 26**: `WittLevelBinding` class, 3 universality properties, 7 `QL_` identity individuals
//! - **Amendment 27**: 5 `SR_` identity individuals (session-scoped resolution algebra)
//! - **Amendments 28–30**: 20 identity families (TS_, QLS_, MN_) for type synthesis,
//!   quantum spectral sequence, and monodromy observables
//! - **Amendments 31–32**: Product/sum type identities (PT_, ST_) and superposition vocabulary
//! - **Amendment 33**: 7 `SC_` identity individuals (saturated context limit algebra)
//! - **Amendment 34**: 5 `MS_` identity individuals (morphospace achievability algebra)
//! - **Amendment 35**: 5 `GD_` identity individuals (computational geodesic algebra)
//! - **Amendment 36**: `QuantumThermodynamicDomain` class, `QuantumThermodynamic` domain individual,
//!   4 `QM_` identity individuals (measurement boundary algebra)
//! - **Amendment 37**: `enumVariant` annotation property, 14 gap-closure identity individuals
//!   (QM_5, RC_6, FPM_8–9, MN_8, QL_8, D_7, SP_1–4, PT_2a–2b, GD_6)
//! - **Amendment 41**: `ValidityScopeKind` class and 4 scope individuals,
//!   `validityKind`/`validKMin`/`validKMax` properties, 7 `QT_` tower identity individuals
//! - **Amendment 48**: 3 `SR_` + 8 `MC_` identity individuals (multi-session coordination algebra)
//! - **Amendment 53**: 12 `WC_` Witt\u{2013}carry bridge identities, 5 `OA_`
//!   Ostrowski\u{2013}Archimedean bridge identities, `ArithmeticValuation`
//!   verification domain, CA_1\u{2013}CA_6 reclassified Enumerative \u{2192} Algebraic
//! - **Amendment 54**: 8 `HT_` homotopy nerve identities (Kan condition,
//!   Hurewicz, Whitehead, Postnikov truncation, spectral collapse)
//! - **Amendment 55**: 3 `psi_` pipeline stages (\u{03c8}_7\u{2013}\u{03c8}_9) +
//!   4 `HP_` homotopy pipeline composition identities
//! - **Amendment 56**: 10 `MD_` moduli space / deformation complex identities
//! - **Amendment 57**: 4 `MR_` moduli resolver identities
//! - **Amendment 58**: 7 `CY_` carry algebra identities (carry
//!   generate/propagate/kill, d_\u{0394} discrepancy, optimal encoding,
//!   site ordering, carry lookahead)
//! - **Amendment 59**: 6 `BM_` named base metrics identities (saturation,
//!   Euler characteristic, index theorem, Jacobian vanishing, Witt defect,
//!   metric composition tower)
//! - **Amendment 60**: 4 `GL_` Galois connection identities (lower/upper
//!   adjoint, fixpoint completeness, order reversal) + 4 `NV_` nerve
//!   operations identities (nerve additivity, Mayer\u{2013}Vietoris,
//!   Betti bounded change, accumulation monotonicity)
//! - **Amendment 61**: 8 `SD_` structural type identities (scalar/symbol
//!   grounding, sequence/tuple site count, graph/set/tree constraints,
//!   table functorial decomposition)
//! - **Amendment 62**: 6 composed operation classes (`ComposedOperation`,
//!   `DispatchOperation`, `InferenceOperation`, `AccumulationOperation`,
//!   `LeasePartitionOperation`, `SessionCompositionOperation`),
//!   `ComposedAlgebraic` verification domain, 5 geometric characters,
//!   5 operator individuals, 18 identity individuals (DD_, PI_, PA_,
//!   PL_, PK_, PP_)
//! - **Amendment 63**: 16 reduction core identities: 7 `PE_` pipeline
//!   evaluation semantics, 7 `PM_` machine execution model, 2 `ER_`
//!   execution rules
//! - **Amendment 64**: 16 reduction expansion identities: 2 `ER_` additional
//!   execution rules, 4 `EA_` epoch admission, 3 `OE_` optimization
//!   equivalences, 3 `OE_4` sub-lemmas, 4 `CS_` cost semantics
//! - **Amendment 65**: 16 reduction completion identities: 1 `CS_5` (total cost
//!   bound), 3 `FA_` (scheduler fairness), 4 `SW_` (service window),
//!   4 `LS_` (lease suspension), 3 `TJ_` (transaction), 1 `AP_1`\u{2013}3
//!   (approximation)
//! - **Amendment 67**: 7 `DA_` division algebra identities (Cayley-Dickson
//!   doublings R\u{2192}C\u{2192}H\u{2192}O, Adams dimension restriction,
//!   convergence\u{2013}algebra correspondence, commutator/associator
//!   characterisation)
//! - **Amendment 68**: 9 `IN_` interaction algebra identities (interaction cost,
//!   commutator disjointness/sharing, negotiation convergence, subspace selection,
//!   outcome space, mutual modeling, nerve Betti bounds) + 4 `AS_` associator
//!   identities (non-associativity from read-write interleaving)
//! - **Amendment 69**: 5 `MO_` monoidal composition identities (unit law,
//!   associativity, certificate composition, saturation and residual monotonicity)
//! - **Amendment 84**: 2 `CS_` compile unit identities (CS_6 budget
//!   solvency rejection, CS_7 unit address computation)
//!
//! **Critical identity:** `neg(bnot(x)) = succ(x)` for all x in R_n.
//!
//! **Space classification:** `kernel` — compiled into ROM.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `op/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "op",
            iri: NS_OP,
            label: "UOR Operations",
            comment: "Ring operations, involutions, algebraic identities, and the \
                      dihedral symmetry group D_{2^n} generated by neg and bnot.",
            space: Space::Kernel,
            imports: &[NS_SCHEMA],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/op/Operation",
            label: "Operation",
            comment: "An operation on the ring Z/(2^n)Z. The root class for all \
                      UOR kernel operations.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/op/UnaryOp",
            label: "UnaryOp",
            comment: "A unary operation on the ring: takes one datum and produces \
                      one datum.",
            subclass_of: &["https://uor.foundation/op/Operation"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/op/BinaryOp",
            label: "BinaryOp",
            comment: "A binary operation on the ring: takes two datums and produces \
                      one datum.",
            subclass_of: &["https://uor.foundation/op/Operation"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/op/Involution",
            label: "Involution",
            comment: "A unary operation f such that f(f(x)) = x for all x in R_n. \
                      The two UOR involutions are neg (ring reflection) and bnot \
                      (hypercube reflection).",
            subclass_of: &["https://uor.foundation/op/UnaryOp"],
            disjoint_with: &[],
        },
        // Amendment 3: Identity class
        Class {
            id: "https://uor.foundation/op/Identity",
            label: "Identity",
            comment: "An algebraic identity: a statement that two expressions are \
                      equal for all inputs. The critical identity is \
                      neg(bnot(x)) = succ(x) for all x in R_n.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 4: Group classes
        Class {
            id: "https://uor.foundation/op/Group",
            label: "Group",
            comment: "A group: a set with an associative binary operation, an \
                      identity element, and inverses for every element.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/op/DihedralGroup",
            label: "DihedralGroup",
            comment: "The dihedral group D_{2^n} of order 2^(n+1), generated by the \
                      ring reflection (neg) and the hypercube reflection (bnot). \
                      This group governs the symmetry of the UOR type space.",
            subclass_of: &["https://uor.foundation/op/Group"],
            disjoint_with: &[],
        },
        // Amendment 23: Typed controlled vocabulary classes
        Class {
            id: "https://uor.foundation/op/VerificationDomain",
            label: "VerificationDomain",
            comment: "A named mathematical discipline through which an algebraic \
                      identity is established and grounded. Every op:Identity \
                      individual references at least one VerificationDomain via \
                      op:verificationDomain. The nine canonical domain individuals \
                      are kernel-level constants defined in op/.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/op/GeometricCharacter",
            label: "GeometricCharacter",
            comment: "The geometric role of a ring operation in the UOR dual-geometry \
                      (ring + hypercube). Every op:Operation individual references \
                      exactly one GeometricCharacter via op:hasGeometricCharacter. \
                      The nine canonical individuals correspond to the action types \
                      of the dihedral group D_{2^n}.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 26: Quantum Level Scaling
        Class {
            id: "https://uor.foundation/op/WittLevelBinding",
            label: "WittLevelBinding",
            comment: "A record linking an op:Identity individual to a specific quantum \
                      level at which it has been verified. Non-functional: one \
                      WittLevelBinding per (Identity, WittLevel) pair verified.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 36: Quantum Thermodynamic Domain
        Class {
            id: "https://uor.foundation/op/QuantumThermodynamicDomain",
            label: "QuantumThermodynamicDomain",
            comment: "A verification domain at the intersection of quantum \
                      superposition and classical thermodynamics. Identities in \
                      this domain require both SuperpositionDomain and \
                      Thermodynamic reasoning simultaneously.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 41: ValidityScopeKind
        Class {
            id: "https://uor.foundation/op/ValidityScopeKind",
            label: "ValidityScopeKind",
            comment: "Root class for validity scope individuals. Instances are the \
                      four named scope kinds: Universal, ParametricLower, \
                      ParametricRange, and LevelSpecific.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 62: Composed Operations
        Class {
            id: "https://uor.foundation/op/ComposedOperation",
            label: "ComposedOperation",
            comment: "An operation formed by composing ring operations, witnessed by \
                      op:composedOf and morphism/CompositionLaw.",
            subclass_of: &[
                "https://uor.foundation/op/Operation",
                "https://uor.foundation/morphism/Composition",
            ],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/op/DispatchOperation",
            label: "DispatchOperation",
            comment: "\u{03b4}: Query \u{00d7} ResolverRegistry \u{2192} Resolver. \
                      Non-commutative, non-associative, arity 2.",
            subclass_of: &["https://uor.foundation/op/ComposedOperation"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/op/InferenceOperation",
            label: "InferenceOperation",
            comment: "\u{03b9} = P \u{2218} \u{03a0} \u{2218} G (the \u{03c6}-pipeline \
                      composed). Non-commutative, non-associative, arity 2.",
            subclass_of: &["https://uor.foundation/op/ComposedOperation"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/op/AccumulationOperation",
            label: "AccumulationOperation",
            comment: "\u{03b1}: Binding \u{00d7} Context \u{2192} Context. \
                      Non-commutative, associative at convergence (SR_10), arity 2.",
            subclass_of: &["https://uor.foundation/op/ComposedOperation"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/op/LeasePartitionOperation",
            label: "LeasePartitionOperation",
            comment: "\u{03bb}: SharedContext \u{00d7} \u{2115} \u{2192} \
                      ContextLease^k. Non-commutative, non-associative, arity 2.",
            subclass_of: &["https://uor.foundation/op/ComposedOperation"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/op/SessionCompositionOperation",
            label: "SessionCompositionOperation",
            comment: "\u{03ba}: Session \u{00d7} Session \u{2192} Session. \
                      Commutative (disjoint leases), associative (SR_8), arity 2.",
            subclass_of: &["https://uor.foundation/op/ComposedOperation"],
            disjoint_with: &[],
        },
        // Amendment 89: Structured group presentation class
        Class {
            id: "https://uor.foundation/op/GroupPresentation",
            label: "GroupPresentation",
            comment: "A structured group presentation: generators and relations \
                      as typed data rather than prose strings.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/op/arity",
            label: "arity",
            comment: "The number of arguments this operation takes. 1 for unary \
                      operations, 2 for binary operations.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Operation"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 23: hasGeometricCharacter (Object, replaces geometricCharacter)
        Property {
            id: "https://uor.foundation/op/hasGeometricCharacter",
            label: "hasGeometricCharacter",
            comment: "The geometric role of this operation in the UOR ring and \
                      hypercube geometry. Functional: each operation has exactly \
                      one geometric character.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Operation"),
            range: "https://uor.foundation/op/GeometricCharacter",
        },
        Property {
            id: "https://uor.foundation/op/commutative",
            label: "commutative",
            comment: "Whether this binary operation satisfies op(x,y) = op(y,x) \
                      for all x, y in R_n.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/BinaryOp"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/op/associative",
            label: "associative",
            comment: "Whether this binary operation satisfies \
                      op(op(x,y),z) = op(x,op(y,z)) for all x, y, z in R_n.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/BinaryOp"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/op/identity",
            label: "identity",
            comment: "The identity element of this binary operation: the value e \
                      such that op(x, e) = op(e, x) = x for all x in R_n.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/BinaryOp"),
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/op/inverse",
            label: "inverse",
            comment: "The inverse operation: the operation inv_op such that \
                      op(x, inv_op(x)) = e for all x, where e is the identity.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Operation"),
            range: "https://uor.foundation/op/Operation",
        },
        // Amendment 1: composedOf property
        Property {
            id: "https://uor.foundation/op/composedOf",
            label: "composedOf",
            comment: "Ordered list of operations this operation is composed from. \
                      Uses rdf:List to preserve application order (first element \
                      applied innermost). E.g., succ = neg ∘ bnot is encoded as \
                      [op:neg, op:bnot] meaning neg applied to the result of bnot.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Operation"),
            range: RDF_LIST,
        },
        // Amendment 3: Identity properties
        Property {
            id: "https://uor.foundation/op/lhs",
            label: "lhs",
            comment: "The left-hand side of an algebraic identity as a typed \
                      AST node (schema:TermExpression).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Identity"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/op/rhs",
            label: "rhs",
            comment: "The right-hand side of an algebraic identity as a typed \
                      AST node (schema:TermExpression).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Identity"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/op/forAll",
            label: "forAll",
            comment: "The quantifier scope: a typed declaration of the \
                      variable(s) over which this identity holds.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Identity"),
            range: "https://uor.foundation/schema/ForAllDeclaration",
        },
        // Amendment 4: Group properties
        Property {
            id: "https://uor.foundation/op/generatedBy",
            label: "generatedBy",
            comment: "An operation that generates this group. The dihedral group \
                      D_{2^n} is generated by op:neg and op:bnot.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/op/Group"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/op/order",
            label: "order",
            comment: "The number of elements in the group. For D_{2^n}, the order \
                      is 2^(n+1).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Group"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/op/presentation",
            label: "presentation",
            comment: "The group presentation (generators and relations). \
                      Example: ⟨r, s | r^{2^n} = s² = e, srs = r⁻¹⟩",
            kind: PropertyKind::Annotation,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Group"),
            range: XSD_STRING,
        },
        // Amendment 23: Typed verification properties (replaces Amendment 21 string properties)
        Property {
            id: "https://uor.foundation/op/verificationDomain",
            label: "verificationDomain",
            comment: "The mathematical discipline(s) through which this identity is \
                      established. Range is op:VerificationDomain. Non-functional: \
                      composite identities (e.g. IT_7a–IT_7d) reference multiple \
                      domain individuals.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/op/Identity"),
            range: "https://uor.foundation/op/VerificationDomain",
        },
        // Amendment 26: Quantum Level Scaling properties
        Property {
            id: "https://uor.foundation/op/verifiedAtLevel",
            label: "verifiedAtLevel",
            comment: "Links an Identity individual to a WittLevelBinding attesting \
                      verification at a specific quantum level. Non-functional: one \
                      binding per (Identity, WittLevel) pair.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/op/Identity"),
            range: "https://uor.foundation/op/WittLevelBinding",
        },
        Property {
            id: "https://uor.foundation/op/bindingLevel",
            label: "bindingLevel",
            comment: "The quantum level at which this WittLevelBinding was verified.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/WittLevelBinding"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/op/universallyValid",
            label: "universallyValid",
            comment: "True iff this identity holds for all n ≥ 1 (proved symbolically \
                      by induction on the ring axioms, not just exhaustively at Q0). \
                      Identities that reference 8-bit-specific constants receive \
                      universallyValid = false.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Identity"),
            range: XSD_BOOLEAN,
        },
        // Amendment 37: Enum variant alignment (Gap 11)
        Property {
            id: "https://uor.foundation/op/enumVariant",
            label: "enumVariant",
            comment: "The canonical Rust enum variant name corresponding to this \
                      VerificationDomain individual. Provides formal alignment \
                      between the OWL individual and the generated Rust enum.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/VerificationDomain"),
            range: "https://uor.foundation/op/VerificationDomain",
        },
        // Amendment 41: Validity scope properties
        Property {
            id: "https://uor.foundation/op/validityKind",
            label: "validityKind",
            comment: "The structured validity scope of this identity, replacing the \
                      binary universallyValid flag. Required on all new Identity \
                      individuals.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Identity"),
            range: "https://uor.foundation/op/ValidityScopeKind",
        },
        Property {
            id: "https://uor.foundation/op/validKMin",
            label: "validKMin",
            comment: "Minimum quantum level index k for ParametricLower and \
                      ParametricRange scopes.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Identity"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/op/validKMax",
            label: "validKMax",
            comment: "Maximum quantum level index k (inclusive) for ParametricRange \
                      scope.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Identity"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 62: Composed Operations properties
        Property {
            id: "https://uor.foundation/op/composedOfOps",
            label: "composedOfOps",
            comment: "References a constituent operation of a ComposedOperation. \
                      Non-functional: a composed operation may reference multiple \
                      constituent operations.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/op/ComposedOperation"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/op/operatorSignature",
            label: "operatorSignature",
            comment: "Human-readable type signature describing the domain and codomain \
                      of a composed operation (e.g., 'Query \u{00d7} Registry \u{2192} \
                      Resolver').",
            kind: PropertyKind::Annotation,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/ComposedOperation"),
            range: XSD_STRING,
        },
        // Amendment 71: Missing composed operation properties (20)
        Property {
            id: "https://uor.foundation/op/dispatchSource",
            label: "dispatchSource",
            comment: "The source selector for a dispatch operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/DispatchOperation"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/op/dispatchTarget",
            label: "dispatchTarget",
            comment: "The target resolver for a dispatch operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/DispatchOperation"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/op/inferenceSource",
            label: "inferenceSource",
            comment: "The source data for an inference operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/InferenceOperation"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/op/inferenceTarget",
            label: "inferenceTarget",
            comment: "The target type for an inference operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/InferenceOperation"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/op/inferencePipeline",
            label: "inferencePipeline",
            comment: "The pipeline through which inference is performed.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/InferenceOperation"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/op/accumulationBase",
            label: "accumulationBase",
            comment: "The base value for an accumulation operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/AccumulationOperation"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/op/accumulationBinding",
            label: "accumulationBinding",
            comment: "The binding accumulator for an accumulation operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/AccumulationOperation"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/op/leaseSource",
            label: "leaseSource",
            comment: "The source context for a lease partition operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/LeasePartitionOperation"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/op/leaseFactor",
            label: "leaseFactor",
            comment: "The partition factor for a lease partition operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/LeasePartitionOperation"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/op/leasePartitionCount",
            label: "leasePartitionCount",
            comment: "The number of partitions in a lease partition operation.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/LeasePartitionOperation"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/op/compositionLeftSession",
            label: "compositionLeftSession",
            comment: "The left session in a session composition operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/SessionCompositionOperation"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/op/compositionRightSession",
            label: "compositionRightSession",
            comment: "The right session in a session composition operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/SessionCompositionOperation"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/op/operatorDomainType",
            label: "operatorDomainType",
            comment: "The domain type of a composed operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/ComposedOperation"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/op/operatorRangeType",
            label: "operatorRangeType",
            comment: "The range type of a composed operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/ComposedOperation"),
            range: "https://uor.foundation/type/TypeDefinition",
        },
        Property {
            id: "https://uor.foundation/op/operatorComplexity",
            label: "operatorComplexity",
            comment: "The computational complexity class of a composed operation.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/ComposedOperation"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/op/operatorIdempotent",
            label: "operatorIdempotent",
            comment: "Whether this composed operation is idempotent.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/ComposedOperation"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/op/composedOperatorCount",
            label: "composedOperatorCount",
            comment: "The number of constituent operations in a composed operation.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/ComposedOperation"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/op/isInvolutory",
            label: "isInvolutory",
            comment: "Whether applying this operation twice yields the identity.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/ComposedOperation"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/op/convergenceGuarantee",
            label: "convergenceGuarantee",
            comment: "Description of the convergence guarantee for this operation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/ComposedOperation"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // v0.2.1 Phase 7a.9: Parametric ring-op metadata. Lean codegen
        // walks op:PrimitiveOp individuals filtered by op:isRingOp to
        // emit one typeclass per op. Arity comes from the existing
        // op:arity integer property (1=unary, 2=binary).
        Property {
            id: "https://uor.foundation/op/isRingOp",
            label: "isRingOp",
            comment: "True iff this Operation participates in the \
                      Z/(2^n)Z ring-arithmetic vocabulary. Annotation drives \
                      the Lean RingOp class generation in UOR/Enforcement.lean.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/op/Operation"),
            range: XSD_BOOLEAN,
        },
    ]
}

/// Returns identity individuals with original `Str` values for lhs/rhs/forAll.
/// Used by `schema::generate_ast_individuals()` to read the source expressions.
pub(crate) fn raw_individuals() -> Vec<Individual> {
    raw_individuals_vec()
}

fn individuals() -> Vec<Individual> {
    crate::model::rewrite_identity_ast_refs(raw_individuals_vec())
}

fn raw_individuals_vec() -> Vec<Individual> {
    vec![
        // Amendment 23: VerificationDomain individuals (8)
        Individual {
            id: "https://uor.foundation/op/Enumerative",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "Enumerative",
            comment: "Established by exhaustive traversal of R_n. Valid for all \
                      identities where the ring is finite.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/Enumerative")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/Algebraic",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "Algebraic",
            comment: "Established by equational reasoning from ring or group axioms. \
                      Covers derivations via associativity, commutativity, inverse \
                      laws, and group presentations.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/Geometric",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "Geometric",
            comment: "Established by isometry, symmetry, or GeometricCharacter \
                      arguments. Covers dihedral actions, fixed-point analysis, \
                      automorphism groups, and affine embeddings.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/Analytical",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "Analytical",
            comment: "Established via discrete differential calculus or metric \
                      analysis. Covers ring/Hamming derivatives (DC_), metric \
                      divergence (AM_), and adiabatic scheduling (AR_).",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/Thermodynamic",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "Thermodynamic",
            comment: "Established via entropy, Landauer bounds, or Boltzmann \
                      distributions. Covers site entropy (TH_), reversible \
                      computation (RC_), and phase transitions.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/Topological",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "Topological",
            comment: "Established via simplicial homology, cohomology, or \
                      constraint nerve analysis. Covers homological algebra (HA_) \
                      and the ψ-pipeline base chain ψ_1..ψ_6 (constraint nerve \
                      construction, chain functor, homology, Betti extraction, \
                      dualization, cohomology).",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/Pipeline",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "Pipeline",
            comment: "Established by the inter-algebra map structure of the \
                      resolution pipeline. Covers φ-maps (phi_1–phi_6) and the \
                      ψ-pipeline tower ψ_7..ψ_9 (Postnikov truncation, homotopy \
                      group extraction, k-invariant computation). The earlier \
                      ψ_1..ψ_6 chain (constraint nerve → simplicial homology) \
                      is established under op:Topological.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IndexTheoretic",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "IndexTheoretic",
            comment: "Established by the composition of Analytical and Topological \
                      reasoning. The only domain requiring multiple op:verificationDomain \
                      assertions. Covers the UOR Index Theorem (IT_7a–IT_7d).",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic")),
            ],
        },
        // Amendment 32: SuperpositionDomain
        Individual {
            id: "https://uor.foundation/op/SuperpositionDomain",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "SuperpositionDomain",
            comment: "Established by superposition analysis of site states. \
                      Covers identities involving superposed (non-classical) \
                      site assignments where sites carry complex amplitudes.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/SuperpositionDomain")),
            ],
        },
        // Amendment 36: QuantumThermodynamic verification domain
        Individual {
            id: "https://uor.foundation/op/QuantumThermodynamic",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "QuantumThermodynamic",
            comment: "Established by the intersection of quantum superposition \
                      analysis and classical thermodynamic reasoning. Covers \
                      identities relating von Neumann entropy of superposed \
                      states to Landauer costs of projective collapse (QM_).",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/QuantumThermodynamic")),
            ],
        },
        // Amendment 53: ArithmeticValuation verification domain
        Individual {
            id: "https://uor.foundation/op/ArithmeticValuation",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "ArithmeticValuation",
            comment: "Established by number-theoretic valuation arguments including \
                      p-adic absolute values, the Ostrowski product formula, and \
                      the arithmetic of global fields. Covers identities grounded \
                      in the product formula |x|_p \u{00b7} |x|_\u{221e} = 1 and \
                      the Witt\u{2013}Ostrowski derivation chain.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/ArithmeticValuation")),
            ],
        },
        // Amendment 62: ComposedAlgebraic verification domain
        Individual {
            id: "https://uor.foundation/op/ComposedAlgebraic",
            type_: "https://uor.foundation/op/VerificationDomain",
            label: "ComposedAlgebraic",
            comment: "Verification domain for composed operation identities \u{2014} \
                      algebraic properties of operator compositions including \
                      dispatch, inference, accumulation, lease, and session \
                      composition operations.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
            ],
        },
        // Amendment 41: ValidityScopeKind individuals (4)
        Individual {
            id: "https://uor.foundation/op/Universal",
            type_: "https://uor.foundation/op/ValidityScopeKind",
            label: "Universal",
            comment: "Holds for all k in N. No minimum k constraint.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ParametricLower",
            type_: "https://uor.foundation/op/ValidityScopeKind",
            label: "ParametricLower",
            comment: "Holds for all k >= k_min, where k_min is given by validKMin.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/ParametricLower")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ParametricRange",
            type_: "https://uor.foundation/op/ValidityScopeKind",
            label: "ParametricRange",
            comment: "Holds for k_min <= k <= k_max. Both validKMin and validKMax \
                      required.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/ParametricRange")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/LevelSpecific",
            type_: "https://uor.foundation/op/ValidityScopeKind",
            label: "LevelSpecific",
            comment: "Holds only at exactly one level, given by a WittLevelBinding.",
            properties: &[
                ("https://uor.foundation/op/enumVariant", IndividualValue::IriRef("https://uor.foundation/op/LevelSpecific")),
            ],
        },
        // Amendment 23: GeometricCharacter individuals (9)
        Individual {
            id: "https://uor.foundation/op/RingReflection",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "RingReflection",
            comment: "Reflection through the origin of the additive ring: \
                      neg(x) = -x mod 2^n. One of the two generators of D_{2^n}.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/HypercubeReflection",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "HypercubeReflection",
            comment: "Reflection through the centre of the hypercube: \
                      bnot(x) = (2^n-1) ⊕ x. The second generator of D_{2^n}.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/Rotation",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "Rotation",
            comment: "Rotation by one step: succ(x) = (x+1) mod 2^n. \
                      The composition of the two reflections.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/RotationInverse",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "RotationInverse",
            comment: "Rotation by one step in the reverse direction: \
                      pred(x) = (x-1) mod 2^n.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/Translation",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "Translation",
            comment: "Translation along the ring axis: add(x,y), sub(x,y). \
                      Preserves Hamming distance locally.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/Scaling",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "Scaling",
            comment: "Scaling along the ring axis: mul(x,y) = (x×y) mod 2^n.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/HypercubeTranslation",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "HypercubeTranslation",
            comment: "Translation along the hypercube axis: xor(x,y) = x ⊕ y. \
                      Preserves ring distance locally.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/HypercubeProjection",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "HypercubeProjection",
            comment: "Projection onto a hypercube face: and(x,y) = x ∧ y. \
                      Idempotent; collapses dimensions.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/HypercubeJoin",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "HypercubeJoin",
            comment: "Join on the hypercube lattice: or(x,y) = x ∨ y. \
                      Idempotent; dual to projection.",
            properties: &[],
        },
        // Wiki ADR-053: three ring-axis arithmetic geometric characters
        // for Div/Mod/Pow per the catalog extension. The fold-rule
        // dual-naming preserves the dihedral-group action structure
        // (`Quotient` ⊣ `Scaling`, `Remainder` complements `Quotient`,
        // `IteratedScaling` extends the multiplicative `Scaling` axis).
        Individual {
            id: "https://uor.foundation/op/Quotient",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "Quotient",
            comment: "Euclidean quotient along the ring axis: div(a,b) — the \
                      structural dual of Scaling. Geometric character of \
                      `op:div` per ADR-053.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/Remainder",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "Remainder",
            comment: "Euclidean remainder along the ring axis: mod(a,b). \
                      Complement of Quotient — together they realize the \
                      divmod fold-rule. Geometric character of `op:mod` \
                      per ADR-053.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/IteratedScaling",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "IteratedScaling",
            comment: "Iterated multiplicative scaling along the ring axis: \
                      pow(base, exp) = base^exp mod 2^n. Extends the \
                      multiplicative Scaling axis via square-and-multiply \
                      iteration. Geometric character of `op:pow` per ADR-053.",
            properties: &[],
        },
        // Amendment 62: Composed-operation geometric characters (5)
        Individual {
            id: "https://uor.foundation/op/ConstraintSelection",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "ConstraintSelection",
            comment: "Geometric character of dispatch: constraint-guided selection \
                      over the resolver registry lattice.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/ResolutionTraversal",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "ResolutionTraversal",
            comment: "Geometric character of inference: traversal through the \
                      \u{03c6}-pipeline resolution graph P \u{2218} \u{03a0} \
                      \u{2218} G.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/SiteBinding",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "SiteBinding",
            comment: "Geometric character of accumulation: progressive pinning of \
                      site states in the context lattice.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/SitePartition",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "SitePartition",
            comment: "Geometric character of lease partition: splitting a shared \
                      context into disjoint site-set leases.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/op/SessionMerge",
            type_: "https://uor.foundation/op/GeometricCharacter",
            label: "SessionMerge",
            comment: "Geometric character of session composition: merging disjoint \
                      lease sessions into a unified resolution context.",
            properties: &[],
        },
        // Amendment 1: 10 primitive operation individuals
        Individual {
            id: "https://uor.foundation/op/neg",
            type_: "https://uor.foundation/op/Involution",
            label: "neg",
            comment: "Ring reflection: neg(x) = (-x) mod 2^n. One of the two \
                      generators of the dihedral group D_{2^n}. neg(neg(x)) = x \
                      (involution property).",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(1)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/RingReflection"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/bnot",
            type_: "https://uor.foundation/op/Involution",
            label: "bnot",
            comment: "Hypercube reflection: bnot(x) = (2^n - 1) ⊕ x (bitwise complement). \
                      The second generator of D_{2^n}. bnot(bnot(x)) = x.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(1)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/HypercubeReflection"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/succ",
            type_: "https://uor.foundation/op/UnaryOp",
            label: "succ",
            comment: "Successor: succ(x) = neg(bnot(x)) = (x + 1) mod 2^n. \
                      The critical identity: succ is the composition neg ∘ bnot.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(1)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/Rotation"),
                ),
                (
                    "https://uor.foundation/op/composedOf",
                    IndividualValue::List(&[
                        "https://uor.foundation/op/neg",
                        "https://uor.foundation/op/bnot",
                    ]),
                ),
                (
                    "https://uor.foundation/op/inverse",
                    IndividualValue::IriRef("https://uor.foundation/op/pred"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/pred",
            type_: "https://uor.foundation/op/UnaryOp",
            label: "pred",
            comment: "Predecessor: pred(x) = bnot(neg(x)) = (x - 1) mod 2^n. \
                      The inverse of succ. pred is the composition bnot ∘ neg.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(1)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/RotationInverse"),
                ),
                (
                    "https://uor.foundation/op/composedOf",
                    IndividualValue::List(&[
                        "https://uor.foundation/op/bnot",
                        "https://uor.foundation/op/neg",
                    ]),
                ),
                (
                    "https://uor.foundation/op/inverse",
                    IndividualValue::IriRef("https://uor.foundation/op/succ"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/add",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "add",
            comment: "Ring addition: add(x, y) = (x + y) mod 2^n. \
                      Commutative, associative; identity element is 0.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/Translation"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/identity",
                    IndividualValue::Int(0),
                ),
                (
                    "https://uor.foundation/op/inverse",
                    IndividualValue::IriRef("https://uor.foundation/op/sub"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/sub",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "sub",
            comment: "Ring subtraction: sub(x, y) = (x - y) mod 2^n. \
                      Not commutative, not associative.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/Translation"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(false),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/mul",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "mul",
            comment: "Ring multiplication: mul(x, y) = (x × y) mod 2^n. \
                      Commutative, associative; identity element is 1.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/Scaling"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/identity",
                    IndividualValue::Int(1),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/xor",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "xor",
            comment: "Bitwise exclusive or: xor(x, y) = x ⊕ y. \
                      Commutative, associative; identity element is 0.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/HypercubeTranslation"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/identity",
                    IndividualValue::Int(0),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/and",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "and",
            comment: "Bitwise and: and(x, y) = x ∧ y. \
                      Commutative, associative.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/HypercubeProjection"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/or",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "or",
            comment: "Bitwise or: or(x, y) = x ∨ y. \
                      Commutative, associative.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/HypercubeJoin"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        // ADR-013/TR-08 substrate amendment: byte-level comparison + concat
        // primitives admitting <=, <, >=, > closure-body operators (G3) and
        // byte-packing serialization (concat). Each is a binary op whose
        // catamorphism fold-rule lives in apply_primitive_op.
        Individual {
            id: "https://uor.foundation/op/le",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "le",
            comment: "Byte-level less-than-or-equal: le(x, y) = 1 if x ≤ y else 0. \
                      Operands compared as big-endian unsigned byte sequences. \
                      The catamorphism fold-rule emits Literal(1) on true, \
                      Literal(0) on false.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(false)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/ConstraintSelection"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/lt",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "lt",
            comment: "Byte-level less-than: lt(x, y) = 1 if x < y else 0. \
                      Operands compared as big-endian unsigned byte sequences.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(false)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/ConstraintSelection"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ge",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "ge",
            comment: "Byte-level greater-than-or-equal: ge(x, y) = 1 if x ≥ y else 0. \
                      Operands compared as big-endian unsigned byte sequences.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(false)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/ConstraintSelection"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/gt",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "gt",
            comment: "Byte-level greater-than: gt(x, y) = 1 if x > y else 0. \
                      Operands compared as big-endian unsigned byte sequences.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(false)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/ConstraintSelection"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/concat",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "concat",
            comment: "Byte-sequence concatenation: concat(x, y) = x ⧺ y. \
                      The substrate's byte-packing primitive — admits header \
                      serialization and other byte-array construction patterns. \
                      Result length is len(x) + len(y), carried by the \
                      source-polymorphic TermValue (inline up to the application's \
                      carrier width, else borrowed or streamed — no fixed ceiling).",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(false)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/HypercubeJoin"),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        // Wiki ADR-053: three ring-axis arithmetic primitives extending the
        // closed PrimitiveOp catalog. Each is realized as a
        // folding-transformation fold-rule per ADR-050's width-parametric
        // evaluation. Discriminant ordering preserves the prior 15-variant
        // layout (Neg=0..Concat=14); Div/Mod/Pow append at 15/16/17.
        Individual {
            id: "https://uor.foundation/op/div",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "div",
            comment: "Euclidean quotient: div(a, b) = q where a = q·b + r, \
                      0 ⇐ r < b. Total on the ring for b > 0; b = 0 emits a \
                      ShapeViolation. Operands read as unsigned big-endian \
                      integers at the operand width.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/Quotient"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(false),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/mod",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "mod",
            comment: "Euclidean remainder: mod(a, b) = r where a = q·b + r, \
                      0 ⇐ r < b. Total on the ring for b > 0; b = 0 emits a \
                      ShapeViolation. Operands read as unsigned big-endian \
                      integers at the operand width.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/Remainder"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(false),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/pow",
            type_: "https://uor.foundation/op/BinaryOp",
            label: "pow",
            comment: "Modular exponentiation: pow(base, exp) = base^exp mod 2^n. \
                      Fold-rule: square-and-multiply over exp bits. \
                      pow(_, 0) = 1; pow(0, b > 0) = 0.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                ("https://uor.foundation/op/isRingOp", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/IteratedScaling"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(false),
                ),
            ],
        },
        // Amendment 62: 5 composed operation individuals
        Individual {
            id: "https://uor.foundation/op/dispatch",
            type_: "https://uor.foundation/op/DispatchOperation",
            label: "dispatch",
            comment: "\u{03b4}(q, R) = R(q): dispatches a query to the matching \
                      resolver in the registry. Non-commutative, non-associative.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/ConstraintSelection"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/op/operatorSignature",
                    IndividualValue::Str("Query \u{00d7} ResolverRegistry \u{2192} Resolver"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/infer",
            type_: "https://uor.foundation/op/InferenceOperation",
            label: "infer",
            comment: "\u{03b9}(s, C) = P(\u{03a0}(G(s, C))): the \u{03c6}-pipeline \
                      composed into a single inference step. Non-commutative, \
                      non-associative.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/ResolutionTraversal"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/op/operatorSignature",
                    IndividualValue::Str("Symbol \u{00d7} Context \u{2192} ResolvedType"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/accumulate",
            type_: "https://uor.foundation/op/AccumulationOperation",
            label: "accumulate",
            comment: "\u{03b1}(b, C) = C': accumulates a binding into a resolution \
                      context, pinning a site. Non-commutative, associative at \
                      convergence (SR_10).",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/SiteBinding"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/operatorSignature",
                    IndividualValue::Str("Binding \u{00d7} Context \u{2192} Context"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/partition_op",
            type_: "https://uor.foundation/op/LeasePartitionOperation",
            label: "partition_op",
            comment: "\u{03bb}(S, k) = (L\u{2081}, \u{2026}, L\u{2096}): partitions \
                      a shared context into k disjoint leases. Non-commutative, \
                      non-associative.",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/SitePartition"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(false),
                ),
                (
                    "https://uor.foundation/op/operatorSignature",
                    IndividualValue::Str("SharedContext \u{00d7} \u{2115} \u{2192} ContextLease^k"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/compose_op",
            type_: "https://uor.foundation/op/SessionCompositionOperation",
            label: "compose_op",
            comment: "\u{03ba}(S\u{2081}, S\u{2082}) = S\u{2081} \u{222a} S\u{2082}: \
                      composes two sessions with disjoint leases into one. \
                      Commutative, associative (SR_8).",
            properties: &[
                ("https://uor.foundation/op/arity", IndividualValue::Int(2)),
                (
                    "https://uor.foundation/op/hasGeometricCharacter",
                    IndividualValue::IriRef("https://uor.foundation/op/SessionMerge"),
                ),
                (
                    "https://uor.foundation/op/commutative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/associative",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/operatorSignature",
                    IndividualValue::Str("Session \u{00d7} Session \u{2192} Session"),
                ),
            ],
        },
        // Amendment 3: criticalIdentity individual
        Individual {
            id: "https://uor.foundation/op/criticalIdentity",
            type_: "https://uor.foundation/op/Identity",
            label: "Critical Identity",
            comment: "The foundational theorem of the UOR kernel: \
                      neg(bnot(x)) = succ(x) for all x in R_n. \
                      This identity links the two involutions (neg and bnot) to the \
                      successor operation, making succ derivable from neg and bnot.",
            properties: &[
                // Amendment 96 (2026-04-12): `lhs`/`rhs` are now `Str`
                // values describing the expressions. The AST auto-
                // generator (`schema::generate_ast_individuals`) reads
                // these strings and emits matching
                // `schema:LiteralExpression` individuals at
                // `schema/term_criticalIdentity_{lhs,rhs}`; the
                // rewrite pipeline (`model::rewrite_identity_ast_refs`)
                // then replaces these source strings with IriRefs to
                // those term individuals so the Lean Identity
                // structure's `lhs`/`rhs : TermExpression` fields
                // type-check. The prior encoding stored a Str for lhs
                // and a List for rhs, violating the functional
                // property's cardinality.
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("neg(bnot(x))"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("succ(x)"),
                ),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str("x ∈ R_n"),
                ),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                // Amendment 26: universally valid across all quantum levels
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 13: Address Resolution identities
        Individual {
            id: "https://uor.foundation/op/AD_1",
            type_: "https://uor.foundation/op/Identity",
            label: "AD_1",
            comment: "Addressing bijection: addresses(glyph(d)) = d. \
                      Round-trip from datum through glyph and back is identity.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("addresses(glyph(d))"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("d"),
                ),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str("d ∈ R_n"),
                ),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AD_2",
            type_: "https://uor.foundation/op/Identity",
            label: "AD_2",
            comment: "Embedding coherence: glyph(ι(addresses(a))) = ι_addr(a). \
                      The addressing diagram commutes through embeddings.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("glyph(ι(addresses(a)))"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("ι_addr(a)"),
                ),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str("a ∈ Addr(R_n), ι : R_n → R_{n'}"),
                ),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 14: Ring Algebra Laws — Additive group (6)
        Individual {
            id: "https://uor.foundation/op/R_A1",
            type_: "https://uor.foundation/op/Identity",
            label: "R_A1",
            comment: "Additive associativity: add(x, add(y, z)) = add(add(x, y), z).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("add(x, add(y, z))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("add(add(x, y), z)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y, z ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/R_A2",
            type_: "https://uor.foundation/op/Identity",
            label: "R_A2",
            comment: "Additive identity: add(x, 0) = x.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("add(x, 0)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/R_A3",
            type_: "https://uor.foundation/op/Identity",
            label: "R_A3",
            comment: "Additive inverse: add(x, neg(x)) = 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("add(x, neg(x))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/R_A4",
            type_: "https://uor.foundation/op/Identity",
            label: "R_A4",
            comment: "Additive commutativity: add(x, y) = add(y, x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("add(x, y)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("add(y, x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/R_A5",
            type_: "https://uor.foundation/op/Identity",
            label: "R_A5",
            comment: "Subtraction definition: sub(x, y) = add(x, neg(y)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("sub(x, y)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("add(x, neg(y))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/R_A6",
            type_: "https://uor.foundation/op/Identity",
            label: "R_A6",
            comment: "Negation involution: neg(neg(x)) = x.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("neg(neg(x))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 14: Multiplicative monoid (5)
        Individual {
            id: "https://uor.foundation/op/R_M1",
            type_: "https://uor.foundation/op/Identity",
            label: "R_M1",
            comment: "Multiplicative associativity: mul(x, mul(y, z)) = mul(mul(x, y), z).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("mul(x, mul(y, z))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("mul(mul(x, y), z)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y, z ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/R_M2",
            type_: "https://uor.foundation/op/Identity",
            label: "R_M2",
            comment: "Multiplicative identity: mul(x, 1) = x.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("mul(x, 1)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/R_M3",
            type_: "https://uor.foundation/op/Identity",
            label: "R_M3",
            comment: "Multiplicative commutativity: mul(x, y) = mul(y, x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("mul(x, y)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("mul(y, x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/R_M4",
            type_: "https://uor.foundation/op/Identity",
            label: "R_M4",
            comment: "Distributivity: mul(x, add(y, z)) = add(mul(x, y), mul(x, z)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("mul(x, add(y, z))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("add(mul(x, y), mul(x, z))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y, z ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/R_M5",
            type_: "https://uor.foundation/op/Identity",
            label: "R_M5",
            comment: "Annihilation: mul(x, 0) = 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("mul(x, 0)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // ADR-053 Ring-axis completion: Euclidean division identities (DV_1..4)
        // and modular-exponentiation identities (PW_1..3). Each governs the
        // corresponding fold-rule in pipeline::apply_primitive_op and the
        // const-eval `const_ring_eval_w{n}` helpers per ADR-050's width-
        // parametric kernel.
        Individual {
            id: "https://uor.foundation/op/DV_1",
            type_: "https://uor.foundation/op/Identity",
            label: "DV_1",
            comment: "Division right-identity: div(a, 1) = a.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("div(a, 1)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("a")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("a ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DV_2",
            type_: "https://uor.foundation/op/Identity",
            label: "DV_2",
            comment: "Division left-absorbing: div(0, b) = 0 for b ≠ 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("div(0, b)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("b ∈ R_n, b ≠ 0")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DV_3",
            type_: "https://uor.foundation/op/Identity",
            label: "DV_3",
            comment: "Division-of-multiplication recovery: div(mul(a, b), b) = a for b ≠ 0 in the unit cone.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("div(mul(a, b), b)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("a")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("a, b ∈ R_n, b ≠ 0, mul(a, b) does not overflow")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DV_4",
            type_: "https://uor.foundation/op/Identity",
            label: "DV_4",
            comment: "Euclidean compatibility: a = add(mul(div(a, b), b), mod(a, b)) for b ≠ 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("a")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("add(mul(div(a, b), b), mod(a, b))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("a, b ∈ R_n, b ≠ 0")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PW_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PW_1",
            comment: "Zero exponent identity: pow(a, 0) = 1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pow(a, 0)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("a ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PW_2",
            type_: "https://uor.foundation/op/Identity",
            label: "PW_2",
            comment: "Unit exponent identity: pow(a, 1) = a.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pow(a, 1)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("a")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("a ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PW_3",
            type_: "https://uor.foundation/op/Identity",
            label: "PW_3",
            comment: "Additive exponent decomposition: pow(a, add(b, c)) = mul(pow(a, b), pow(a, c)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pow(a, add(b, c))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("mul(pow(a, b), pow(a, c))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("a, b, c ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 14: Boolean algebra (13)
        Individual {
            id: "https://uor.foundation/op/B_1",
            type_: "https://uor.foundation/op/Identity",
            label: "B_1",
            comment: "XOR associativity: xor(x, xor(y, z)) = xor(xor(x, y), z).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("xor(x, xor(y, z))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("xor(xor(x, y), z)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y, z ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_2",
            type_: "https://uor.foundation/op/Identity",
            label: "B_2",
            comment: "XOR identity: xor(x, 0) = x.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("xor(x, 0)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_3",
            type_: "https://uor.foundation/op/Identity",
            label: "B_3",
            comment: "XOR self-inverse: xor(x, x) = 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("xor(x, x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_4",
            type_: "https://uor.foundation/op/Identity",
            label: "B_4",
            comment: "AND associativity: and(x, and(y, z)) = and(and(x, y), z).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("and(x, and(y, z))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("and(and(x, y), z)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y, z ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_5",
            type_: "https://uor.foundation/op/Identity",
            label: "B_5",
            comment: "AND identity: and(x, 2^n - 1) = x.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("and(x, 2^n - 1)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_6",
            type_: "https://uor.foundation/op/Identity",
            label: "B_6",
            comment: "AND annihilation: and(x, 0) = 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("and(x, 0)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_7",
            type_: "https://uor.foundation/op/Identity",
            label: "B_7",
            comment: "OR associativity: or(x, or(y, z)) = or(or(x, y), z).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("or(x, or(y, z))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("or(or(x, y), z)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y, z ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_8",
            type_: "https://uor.foundation/op/Identity",
            label: "B_8",
            comment: "OR identity: or(x, 0) = x.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("or(x, 0)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_9",
            type_: "https://uor.foundation/op/Identity",
            label: "B_9",
            comment: "Absorption: and(x, or(x, y)) = x.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("and(x, or(x, y))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_10",
            type_: "https://uor.foundation/op/Identity",
            label: "B_10",
            comment: "AND distributes over OR: and(x, or(y, z)) = or(and(x, y), and(x, z)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("and(x, or(y, z))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("or(and(x, y), and(x, z))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y, z ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_11",
            type_: "https://uor.foundation/op/Identity",
            label: "B_11",
            comment: "De Morgan 1: bnot(and(x, y)) = or(bnot(x), bnot(y)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("bnot(and(x, y))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("or(bnot(x), bnot(y))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_12",
            type_: "https://uor.foundation/op/Identity",
            label: "B_12",
            comment: "De Morgan 2: bnot(or(x, y)) = and(bnot(x), bnot(y)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("bnot(or(x, y))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("and(bnot(x), bnot(y))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/B_13",
            type_: "https://uor.foundation/op/Identity",
            label: "B_13",
            comment: "Bnot involution: bnot(bnot(x)) = x.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("bnot(bnot(x))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 14: Cross-structure (7)
        Individual {
            id: "https://uor.foundation/op/X_1",
            type_: "https://uor.foundation/op/Identity",
            label: "X_1",
            comment: "Neg via subtraction: neg(x) = sub(0, x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("neg(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("sub(0, x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/X_2",
            type_: "https://uor.foundation/op/Identity",
            label: "X_2",
            comment: "Complement via XOR: bnot(x) = xor(x, 2^n - 1).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("bnot(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("xor(x, 2^n - 1)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/X_3",
            type_: "https://uor.foundation/op/Identity",
            label: "X_3",
            comment: "Succ via addition: succ(x) = add(x, 1).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("succ(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("add(x, 1)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/X_4",
            type_: "https://uor.foundation/op/Identity",
            label: "X_4",
            comment: "Pred via subtraction: pred(x) = sub(x, 1).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pred(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("sub(x, 1)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/X_5",
            type_: "https://uor.foundation/op/Identity",
            label: "X_5",
            comment: "Neg-bnot bridge: neg(x) = add(bnot(x), 1).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("neg(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("add(bnot(x), 1)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/X_6",
            type_: "https://uor.foundation/op/Identity",
            label: "X_6",
            comment: "Complement predecessor: bnot(x) = pred(neg(x)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("bnot(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("pred(neg(x))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/X_7",
            type_: "https://uor.foundation/op/Identity",
            label: "X_7",
            comment: "XOR-add bridge: xor(x, y) = add(x, y) - 2 * and(x, y) (in Z before mod).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("xor(x, y)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("add(x, y) - 2 * and(x, y)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ Z (before mod)")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 14: Dihedral group (4, D-2 omitted as duplicate of R-A6)
        Individual {
            id: "https://uor.foundation/op/D_1",
            type_: "https://uor.foundation/op/Identity",
            label: "D_1",
            comment: "Rotation order: succ^[2^n](x) = x.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("succ^{2^n}(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/D_3",
            type_: "https://uor.foundation/op/Identity",
            label: "D_3",
            comment: "Conjugation: neg(succ(neg(x))) = pred(x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("neg(succ(neg(x)))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("pred(x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/D_4",
            type_: "https://uor.foundation/op/Identity",
            label: "D_4",
            comment: "Reverse composition: bnot(neg(x)) = pred(x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("bnot(neg(x))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("pred(x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/D_5",
            type_: "https://uor.foundation/op/Identity",
            label: "D_5",
            comment: "Group closure: D_[2^n] = [succ^k, neg ∘ succ^k : 0 ≤ k < 2^n].",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("D_{2^n}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("{succ^k, neg ∘ succ^k : 0 ≤ k < 2^n}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 14: Unit group (5)
        Individual {
            id: "https://uor.foundation/op/U_1",
            type_: "https://uor.foundation/op/Identity",
            label: "U_1",
            comment: "Unit group decomposition: R_n× ≅ Z/2 × Z/2^[n-2] for n ≥ 3.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("R_n×")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Z/2 × Z/2^{n-2}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 3")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/U_2",
            type_: "https://uor.foundation/op/Identity",
            label: "U_2",
            comment: "Unit group special cases: R_1× ≅ [1]; R_2× ≅ Z/2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("R_1× ≅ {1}; R_2× ≅ Z/2")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("special cases for small n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ∈ {1, 2}")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/U_3",
            type_: "https://uor.foundation/op/Identity",
            label: "U_3",
            comment: "Multiplicative order: ord(u) = lcm(ord((-1)^a), ord(3^b)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ord(u)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("lcm(ord((-1)^a), ord(3^b))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("u = (-1)^a · 3^b ∈ R_n×")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/U_4",
            type_: "https://uor.foundation/op/Identity",
            label: "U_4",
            comment: "Resonance period: ord_g(2) divides φ(g).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ord_g(2)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("divides φ(g)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("g odd")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/U_5",
            type_: "https://uor.foundation/op/Identity",
            label: "U_5",
            comment: "Step formula derivation: step_g = 2 * ((g - (2^n mod g)) mod g) + 1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("step_g")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("2 * ((g - (2^n mod g)) mod g) + 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("g odd, g > 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 14: Affine group (4)
        Individual {
            id: "https://uor.foundation/op/AG_1",
            type_: "https://uor.foundation/op/Identity",
            label: "AG_1",
            comment: "Scaling not dihedral: μ_u ∉ D_[2^n] for u ≠ ±1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("μ_u")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("∉ D_{2^n}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("u ∈ R_n×, u ≠ ±1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AG_2",
            type_: "https://uor.foundation/op/Identity",
            label: "AG_2",
            comment: "Affine group: Aff(R_n) = R_n× ⋉ R_n.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Aff(R_n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("R_n× ⋉ R_n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AG_3",
            type_: "https://uor.foundation/op/Identity",
            label: "AG_3",
            comment: "Affine group order: |Aff(R_n)| = 2^[2n-1].",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("|Aff(R_n)|")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("2^{2n-1}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AG_4",
            type_: "https://uor.foundation/op/Identity",
            label: "AG_4",
            comment: "Subgroup inclusion: D_[2^n] ⊂ Aff(R_n) with u ∈ [±1].",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("D_{2^n}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("⊂ Aff(R_n), u ∈ {±1}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 14: Carry algebra (6)
        Individual {
            id: "https://uor.foundation/op/CA_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CA_1",
            comment: "Addition decomposition: add(x,y)_k = xor(x_k, xor(y_k, c_k(x,y))).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("add(x,y)_k")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("xor(x_k, xor(y_k, c_k(x,y)))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n, 0 ≤ k < n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CA_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CA_2",
            comment: "Carry recurrence: c_[k+1](x,y) = or(and(x_k,y_k), and(xor(x_k,y_k), c_k)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("c_{k+1}(x,y)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("or(and(x_k,y_k), and(xor(x_k,y_k), c_k))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CA_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CA_3",
            comment: "Carry commutativity: c(x, y) = c(y, x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("c(x, y)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("c(y, x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CA_4",
            type_: "https://uor.foundation/op/Identity",
            label: "CA_4",
            comment: "Zero carry: c(x, 0) = 0 at all positions.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("c(x, 0)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n, all positions")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CA_5",
            type_: "https://uor.foundation/op/Identity",
            label: "CA_5",
            comment: "Negation carry: c(x, neg(x))_k = 1 for k > v_2(x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("c(x, neg(x))_k")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n, k > v_2(x)")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CA_6",
            type_: "https://uor.foundation/op/Identity",
            label: "CA_6",
            comment: "Carry-incompatibility link: d_Δ(x, y) > 0 iff ∃ k : c_k(x,y) = 1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_Δ(x, y) > 0")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("∃ k : c_k(x,y) = 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 15: Constraint, Site & Partition Laws — Constraint composition (6)
        Individual {
            id: "https://uor.foundation/op/C_1",
            type_: "https://uor.foundation/op/Identity",
            label: "C_1",
            comment: "Constraint pin union: pins of a composite constraint equal \
                      the union of component pins.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pins(compose(A, B))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("pins(A) ∪ pins(B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraints A, B")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/C_2",
            type_: "https://uor.foundation/op/Identity",
            label: "C_2",
            comment: "Constraint composition commutativity.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("compose(A, B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("compose(B, A)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraints A, B")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/C_3",
            type_: "https://uor.foundation/op/Identity",
            label: "C_3",
            comment: "Constraint composition associativity.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("compose(compose(A,B), C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("compose(A, compose(B,C))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraints A, B, C")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/C_4",
            type_: "https://uor.foundation/op/Identity",
            label: "C_4",
            comment: "Constraint composition idempotence.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("compose(A, A)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("A")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint A")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/C_5",
            type_: "https://uor.foundation/op/Identity",
            label: "C_5",
            comment: "Constraint composition identity element.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("compose(A, ε)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("A")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint A, identity ε")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/C_6",
            type_: "https://uor.foundation/op/Identity",
            label: "C_6",
            comment: "Constraint composition annihilator.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("compose(A, Ω)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Ω")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint A, annihilator Ω")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 15: Constraint dependence (1)
        Individual {
            id: "https://uor.foundation/op/CDI",
            type_: "https://uor.foundation/op/Identity",
            label: "CDI",
            comment: "Constraint-depth invariant: carry complexity of the residue \
                      representation equals the type depth.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("carry(residue(T))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("depth(T)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T ∈ T_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 15: Constraint lattice (5)
        Individual {
            id: "https://uor.foundation/op/CL_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CL_1",
            comment: "Constraint quotient lattice isomorphism to power set.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Constraint/≡")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("2^{[n]}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint equivalence classes")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CL_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CL_2",
            comment: "Lattice join equals constraint composition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("A ∨ B")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("compose(A, B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraints A, B")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CL_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CL_3",
            comment: "Lattice meet pins the intersection of component pins.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pins(A ∧ B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("pins(A) ∩ pins(B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraints A, B")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CL_4",
            type_: "https://uor.foundation/op/Identity",
            label: "CL_4",
            comment: "Constraint lattice distributivity.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("(A ∨ B) ∧ C")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("(A ∧ C) ∨ (B ∧ C)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraints A, B, C")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CL_5",
            type_: "https://uor.foundation/op/Identity",
            label: "CL_5",
            comment: "Constraint lattice complement existence.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("A ∧ A̅ = ε, A ∨ A̅ = Ω")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("∃ A̅ (complement)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint A")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 15: Constraint minimization (3)
        Individual {
            id: "https://uor.foundation/op/CM_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CM_1",
            comment: "Constraint redundancy criterion.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("C_i redundant in {C_1,...,C_k}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("pins(C_i) ⊆ ∪_{j≠i} pins(C_j)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint set {C_1,...,C_k}")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CM_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CM_2",
            comment: "Minimal cover via greedy irredundant removal.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("minimal cover")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("irredundant sub-collection (greedy removal)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("CompositeConstraint")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CM_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CM_3",
            comment: "Minimum constraint count to cover n sites.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("min constraints to cover n sites")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("⌈n / max_k pins_per_constraint_k⌉")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n sites, constraint set")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 15: Constraint cost (5)
        Individual {
            id: "https://uor.foundation/op/CR_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CR_1",
            comment: "Residue constraint cost is the step formula.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cost(ResidueConstraint(m, r))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("step_m = 2 × ((−2^n) mod m) + 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ResidueConstraint")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CR_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CR_2",
            comment: "Carry constraint cost is the popcount of the pattern.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cost(CarryConstraint(p))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("popcount(p)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("CarryConstraint")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CR_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CR_3",
            comment: "Depth constraint cost is sum of residue and carry costs.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cost(DepthConstraint(d_min, d_max))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("cost(residue) + cost(carry)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("DepthConstraint")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CR_4",
            type_: "https://uor.foundation/op/Identity",
            label: "CR_4",
            comment: "Composite constraint cost is subadditive.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cost(compose(A, B))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ cost(A) + cost(B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraints A, B")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CR_5",
            type_: "https://uor.foundation/op/Identity",
            label: "CR_5",
            comment: "Optimal resolution order is increasing cost.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("optimal resolution order")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("increasing cost order")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint set")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 15: Site monotone (4)
        Individual {
            id: "https://uor.foundation/op/F_1",
            type_: "https://uor.foundation/op/Identity",
            label: "F_1",
            comment: "Site monotonicity: a pinned site cannot be unpinned.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pinned site")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("cannot be unpinned")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("SiteIndex")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/F_2",
            type_: "https://uor.foundation/op/Identity",
            label: "F_2",
            comment: "Site budget upper bound: at most n pin operations to close.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pin operations to close")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("FreeRank")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/F_3",
            type_: "https://uor.foundation/op/Identity",
            label: "F_3",
            comment: "Site budget conservation: pinned + free = total sites.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pinnedCount + freeRank")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("totalSites = n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("FreeRank")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/F_4",
            type_: "https://uor.foundation/op/Identity",
            label: "F_4",
            comment: "Site budget closure: closed iff all sites pinned.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("isClosed")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("freeRank = 0 ⇔ pinnedCount = n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("FreeRank")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 15: Site lattice (4)
        Individual {
            id: "https://uor.foundation/op/FL_1",
            type_: "https://uor.foundation/op/Identity",
            label: "FL_1",
            comment: "Site lattice bottom: all sites free.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("⊥")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("all sites free (freeRank = n)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("FreeRank lattice")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FL_2",
            type_: "https://uor.foundation/op/Identity",
            label: "FL_2",
            comment: "Site lattice top: all sites pinned.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("⊤")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("all sites pinned (pinnedCount = n)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("FreeRank lattice")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FL_3",
            type_: "https://uor.foundation/op/Identity",
            label: "FL_3",
            comment: "Site lattice join is union of pinnings.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("join(S₁, S₂)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("union of pinnings from S₁ and S₂")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("FreeRank states S₁, S₂")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FL_4",
            type_: "https://uor.foundation/op/Identity",
            label: "FL_4",
            comment: "Site lattice height equals n.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("lattice height")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("FreeRank lattice")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 15: Site-partition map (7)
        Individual {
            id: "https://uor.foundation/op/FPM_1",
            type_: "https://uor.foundation/op/Identity",
            label: "FPM_1",
            comment: "Unit partition membership: x is a unit iff site_0(x) = 1 (x is odd).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("x ∈ Unit")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("site_0(x) = 1 (x is odd)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FPM_2",
            type_: "https://uor.foundation/op/Identity",
            label: "FPM_2",
            comment: "Exterior partition membership: x is exterior iff x is not in the carrier of T.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("x ∈ Exterior")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x ∉ carrier(T)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n, type T")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FPM_3",
            type_: "https://uor.foundation/op/Identity",
            label: "FPM_3",
            comment: "Irreducible partition membership: x is irreducible iff \
                      x is not a unit, exterior, or reducible.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("x ∈ Irreducible")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x ∉ Unit ∪ Exterior AND no non-trivial factorization")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FPM_4",
            type_: "https://uor.foundation/op/Identity",
            label: "FPM_4",
            comment: "Reducible partition membership: x is reducible iff x is \
                      not a unit, exterior, or irreducible.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("x ∈ Reducible")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x ∉ Unit ∪ Exterior ∪ Irreducible")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FPM_5",
            type_: "https://uor.foundation/op/Identity",
            label: "FPM_5",
            comment: "2-adic decomposition: every element factors as 2^{v(x)} times an odd unit.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("x = 2^{v(x)} ⋅ u")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("u odd, v(x) = min position of 1-bit")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FPM_6",
            type_: "https://uor.foundation/op/Identity",
            label: "FPM_6",
            comment: "Stratum size formula.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("|{x: v(x) = k}|")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("2^{n−k−1} for 0 < k < n; 1 for k = n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FPM_7",
            type_: "https://uor.foundation/op/Identity",
            label: "FPM_7",
            comment: "Base type partition cardinalities.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("base type partition")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("|Unit| = 2^{n−1}; |Irr| = 2^{n−2}; |Red| = 2^{n−2}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("R_n, n ≥ 3")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 15: Site position semantics (7)
        Individual {
            id: "https://uor.foundation/op/FS_1",
            type_: "https://uor.foundation/op/Identity",
            label: "FS_1",
            comment: "Site extraction: site_k(x) is the k-th bit of x.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("site_k(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("(x >> k) AND 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n, 0 ≤ k < n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FS_2",
            type_: "https://uor.foundation/op/Identity",
            label: "FS_2",
            comment: "Site 0 is parity.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("site_0(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x mod 2 (parity)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FS_3",
            type_: "https://uor.foundation/op/Identity",
            label: "FS_3",
            comment: "Progressive site determination: site_k given lower sites \
                      determines x mod 2^{k+1}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("site_k(x) given sites 0..k−1")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("determines x mod 2^{k+1}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FS_4",
            type_: "https://uor.foundation/op/Identity",
            label: "FS_4",
            comment: "Cumulative site determination: sites 0..k together determine x mod 2^{k+1}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("sites 0..k together")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("determine x mod 2^{k+1}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FS_5",
            type_: "https://uor.foundation/op/Identity",
            label: "FS_5",
            comment: "Complete site determination: all n sites determine x uniquely.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("all n sites")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("determine x uniquely")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FS_6",
            type_: "https://uor.foundation/op/Identity",
            label: "FS_6",
            comment: "Stratum from sites: v_2(x) is the minimum k where site_k(x) = 1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("stratum(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("v_2(x) = min{k : site_k(x) = 1}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FS_7",
            type_: "https://uor.foundation/op/Identity",
            label: "FS_7",
            comment: "Depth bound: depth(x) ≤ v_2(x) for irreducible elements.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("depth(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ v_2(x) for irreducible elements")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n, base type")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
            ],
        },
        // Amendment 16: Resolution & Observable Laws — Strategy equivalence (1)
        Individual {
            id: "https://uor.foundation/op/RE_1",
            type_: "https://uor.foundation/op/Identity",
            label: "RE_1",
            comment: "Resolution strategy equivalence: dihedral, canonical-form, and \
                      evaluation resolvers all compute the same partition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Π_D(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Π_C(T) = Π_E(T)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T ∈ T_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        // Amendment 16: Iterative resolution (4)
        Individual {
            id: "https://uor.foundation/op/IR_1",
            type_: "https://uor.foundation/op/Identity",
            label: "IR_1",
            comment: "Resolution monotonicity: pinned count never decreases.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pinnedCount(state_{i+1})")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≥ pinnedCount(state_i)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("resolution states")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IR_2",
            type_: "https://uor.foundation/op/Identity",
            label: "IR_2",
            comment: "Resolution convergence bound: at most n iterations.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("iterations to converge")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("resolution loop")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IR_3",
            type_: "https://uor.foundation/op/Identity",
            label: "IR_3",
            comment: "Convergence rate definition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("convergenceRate")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("pinnedCount / iterationCount")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ResolutionState")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IR_4",
            type_: "https://uor.foundation/op/Identity",
            label: "IR_4",
            comment: "Resolution termination: loop terminates if constraint set spans all sites.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("constraint set spans all sites")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("loop terminates")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("complete constraint set")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        // Amendment 16: Step formula (2)
        Individual {
            id: "https://uor.foundation/op/SF_1",
            type_: "https://uor.foundation/op/Identity",
            label: "SF_1",
            comment: "Optimal resolution level for a factor: n ≡ 0 (mod ord_g(2)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("n ≡ 0 (mod ord_g(2))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("factor g has optimal resolution at level n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("factor g, quantum n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SF_2",
            type_: "https://uor.foundation/op/Identity",
            label: "SF_2",
            comment: "Constraint ordering by step cost: smaller step_g first.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("constraints with smaller step_g")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("are more constraining, apply first")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint ordering")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        // Amendment 16: Determinism (2)
        Individual {
            id: "https://uor.foundation/op/RD_1",
            type_: "https://uor.foundation/op/Identity",
            label: "RD_1",
            comment: "Resolution determinism: same type and constraint sequence yield unique partition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("same type T and constraint sequence")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("unique resolved partition")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T ∈ T_n, [C₁,...,Cₖ]")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/RD_2",
            type_: "https://uor.foundation/op/Identity",
            label: "RD_2",
            comment: "Order independence: complete constraint sets yield the same partition regardless of order.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("complete constraint set, any order")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("same final partition")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("closing constraint set")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        // Amendment 16: Strategy proofs (4)
        Individual {
            id: "https://uor.foundation/op/SE_1",
            type_: "https://uor.foundation/op/Identity",
            label: "SE_1",
            comment: "Evaluation resolver directly computes the set-theoretic partition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("EvaluationResolver")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("directly computes set-theoretic partition")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T ∈ T_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SE_2",
            type_: "https://uor.foundation/op/Identity",
            label: "SE_2",
            comment: "Dihedral factorization resolver yields the same partition via orbit decomposition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("DihedralFactorizationResolver")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("orbit decomposition yields same partition")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T ∈ T_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SE_3",
            type_: "https://uor.foundation/op/Identity",
            label: "SE_3",
            comment: "Canonical form resolver yields the same partition via confluent rewrite.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("CanonicalFormResolver")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("confluent rewrite → same partition")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T ∈ T_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SE_4",
            type_: "https://uor.foundation/op/Identity",
            label: "SE_4",
            comment: "All three strategies compute the same set-theoretic partition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Π_D(T) = Π_C(T) = Π_E(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("all compute same set-theoretic partition")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T ∈ T_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        // Amendment 16: Optimal ordering (5)
        Individual {
            id: "https://uor.foundation/op/OO_1",
            type_: "https://uor.foundation/op/Identity",
            label: "OO_1",
            comment: "Benefit of a constraint is the number of new pins it provides.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("benefit(C_i, S)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("|pins(C_i) ∖ S|")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint C_i, pinned set S")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OO_2",
            type_: "https://uor.foundation/op/Identity",
            label: "OO_2",
            comment: "Constraint cost is step or popcount depending on type.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cost(C_i)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("step_{m_i} or popcount(p_i)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ResidueConstraint or CarryConstraint")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OO_3",
            type_: "https://uor.foundation/op/Identity",
            label: "OO_3",
            comment: "Greedy selection maximizes benefit-to-cost ratio.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("greedy selection")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("argmax benefit(C_i, S) / cost(C_i)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("each resolution step")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OO_4",
            type_: "https://uor.foundation/op/Identity",
            label: "OO_4",
            comment: "Greedy approximation achieves (1 − 1/e) ≈ 63% of optimal.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("greedy approximation")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("(1 − 1/e) ≈ 63% of optimal")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("weighted set cover")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OO_5",
            type_: "https://uor.foundation/op/Identity",
            label: "OO_5",
            comment: "Tiebreaker: prefer vertical (residue) before horizontal (carry).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("equal cost tiebreaker")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("prefer vertical (residue) before horizontal (carry)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("cost-tied constraints")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        // Amendment 16: Convergence bounds (6)
        Individual {
            id: "https://uor.foundation/op/CB_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CB_1",
            comment: "Minimum convergence rate: 1 site per iteration (worst case).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("min convergenceRate")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("1 site per iteration")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("worst case")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CB_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CB_2",
            comment: "Maximum convergence rate: n sites in 1 iteration (best case).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("max convergenceRate")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("n sites in 1 iteration")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("best case")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CB_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CB_3",
            comment: "Expected residue constraint rate: ⌊log_2(m)⌋ sites per constraint.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("expected rate (residue)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("⌊log_2(m)⌋ sites per constraint")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ResidueConstraint(m, r)")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CB_4",
            type_: "https://uor.foundation/op/Identity",
            label: "CB_4",
            comment: "Stall detection: convergenceRate < 1 for 2 iterations suggests insufficiency.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("convergenceRate < 1 for 2 iterations")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("constraint set may be insufficient")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("stall detection")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CB_5",
            type_: "https://uor.foundation/op/Identity",
            label: "CB_5",
            comment: "Sufficiency criterion: pin union covers all site positions.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("∪_i pins(C_i) = {0,...,n−1}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("constraint set closes budget")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("sufficiency criterion")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CB_6",
            type_: "https://uor.foundation/op/Identity",
            label: "CB_6",
            comment: "Iteration bound for k constraints: at most min(k, n) iterations.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("iterations for k constraints")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ min(k, n)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("well-formed model")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        // Amendment 16: Metric identities (6)
        Individual {
            id: "https://uor.foundation/op/OB_M1",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_M1",
            comment: "Ring metric triangle inequality.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_R(x, z)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ d_R(x, y) + d_R(y, z)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y, z ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OB_M2",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_M2",
            comment: "Hamming metric triangle inequality.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_H(x, z)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ d_H(x, y) + d_H(y, z)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y, z ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OB_M3",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_M3",
            comment: "Incompatibility metric is the absolute difference of ring and Hamming metrics.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_Δ(x, y)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("|d_R(x, y) − d_H(x, y)|")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OB_M4",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_M4",
            comment: "Negation preserves ring metric.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_R(neg(x), neg(y))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("d_R(x, y)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OB_M5",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_M5",
            comment: "Bitwise complement preserves Hamming metric.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_H(bnot(x), bnot(y))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("d_H(x, y)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OB_M6",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_M6",
            comment: "Successor preserves ring metric but may change Hamming metric.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_R(succ(x), succ(y))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("d_R(x, y) but d_H may differ")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 16: Commutator identities (3)
        Individual {
            id: "https://uor.foundation/op/OB_C1",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_C1",
            comment: "Negation-complement commutator is constant 2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("[neg, bnot](x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OB_C2",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_C2",
            comment: "Negation-translation commutator.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("[neg, add(•,k)](x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("−2k mod 2^n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n, constant k")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OB_C3",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_C3",
            comment: "Complement-XOR commutator is trivial.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("[bnot, xor(•,k)](x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n, constant k")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 16: Holonomy (3)
        Individual {
            id: "https://uor.foundation/op/OB_H1",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_H1",
            comment: "Additive paths have trivial monodromy (abelian ⇒ path-independent).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("closed additive path monodromy")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("trivial (abelian ⇒ path-independent)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("additive group")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OB_H2",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_H2",
            comment: "Dihedral generator paths have monodromy in D_{2^n}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("closed {neg,bnot} path monodromy")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("∈ D_{2^n}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("dihedral generators")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OB_H3",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_H3",
            comment: "Successor-only closed path winding number.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("succ-only path WindingNumber")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("path length / 2^n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("closed succ path")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 16: Observable composition (3)
        Individual {
            id: "https://uor.foundation/op/OB_P1",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_P1",
            comment: "Path length is additive under concatenation.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("PathLength(p₁ ⋅ p₂)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("PathLength(p₁) + PathLength(p₂)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("paths p₁, p₂")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OB_P2",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_P2",
            comment: "Total variation is subadditive under concatenation.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("TotalVariation(p₁ ⋅ p₂)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ TotalVariation(p₁) + TotalVariation(p₂)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("paths p₁, p₂")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OB_P3",
            type_: "https://uor.foundation/op/Identity",
            label: "OB_P3",
            comment: "Reduction length is additive under sequential composition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ReductionLength(c₁ ; c₂)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ReductionLength(c₁) + ReductionLength(c₂)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("reductions c₁, c₂")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 16: Catastrophe thresholds (4)
        Individual {
            id: "https://uor.foundation/op/CT_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CT_1",
            comment: "Catastrophe boundaries are powers of 2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("catastrophe boundaries")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("g = 2^k for 1 ≤ k ≤ n−1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("stratum transitions")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CT_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CT_2",
            comment: "Odd prime catastrophe transitions visibility via residue constraints.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("odd prime catastrophe")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ResidueConstraint(p, •) transitions visibility")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("odd prime p")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CT_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CT_3",
            comment: "Catastrophe threshold is normalized step cost.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("CatastropheThreshold(g)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("step_g / n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("factor g")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CT_4",
            type_: "https://uor.foundation/op/Identity",
            label: "CT_4",
            comment: "Composite catastrophe threshold is max of component thresholds.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("composite catastrophe g = p⋅q")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("max(step_p, step_q) / n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("composite g")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 16: Curvature-cost (4)
        Individual {
            id: "https://uor.foundation/op/CF_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CF_1",
            comment: "Curvature flux is the sum of incompatibility along a path.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("CurvatureFlux(γ)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Σ |d_R(x_i, x_{i+1}) − d_H(x_i, x_{i+1})|")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("path γ")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CF_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CF_2",
            comment: "Resolution cost is bounded below by curvature flux of optimal path.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ResolutionCost(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≥ CurvatureFlux(γ_opt)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("type T")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CF_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CF_3",
            comment: "Successor curvature flux: trailing_ones(x) for most x, n−1 at maximum.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("CurvatureFlux(x, succ(x))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("trailing_ones(x) for t < n; n−1 for x = 2^n−1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CF_4",
            type_: "https://uor.foundation/op/Identity",
            label: "CF_4",
            comment: "Total successor curvature flux over R_n equals 2^n − 2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Σ_{x ∈ R_n} CurvatureFlux(x, succ(x))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("2^n − 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 16: Complete holonomy (5)
        Individual {
            id: "https://uor.foundation/op/HG_1",
            type_: "https://uor.foundation/op/Identity",
            label: "HG_1",
            comment: "Additive holonomy is trivial (abelian group).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("additive holonomy")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("trivial (abelian ⇒ path-independent)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("additive group")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HG_2",
            type_: "https://uor.foundation/op/Identity",
            label: "HG_2",
            comment: "Dihedral generator holonomy group is D_{2^n}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("{neg, bnot, succ, pred} holonomy")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("D_{2^n}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("dihedral generators")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HG_3",
            type_: "https://uor.foundation/op/Identity",
            label: "HG_3",
            comment: "Unit multiplication holonomy equals the unit group.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("{mul(•, u) : u ∈ R_n×} holonomy")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("R_n× ≅ Z/2 × Z/2^{n−2}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("unit group")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HG_4",
            type_: "https://uor.foundation/op/Identity",
            label: "HG_4",
            comment: "Full holonomy group is the affine group.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Hol(R_n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Aff(R_n) = R_n× ⋉ R_n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HG_5",
            type_: "https://uor.foundation/op/Identity",
            label: "HG_5",
            comment: "Holonomy group decomposition into dihedral and non-trivial units.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Hol(R_n) decomposition")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("D_{2^n} ⋅ {mul(•,u) : u ∈ R_n×, u ≠ ±1}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 17: Transform, Addressing & Thermodynamic Laws — Category laws (4)
        Individual {
            id: "https://uor.foundation/op/T_C1",
            type_: "https://uor.foundation/op/Identity",
            label: "T_C1",
            comment: "Category left identity: compose(id, f) = f.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("compose(id, f)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("f")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("f ∈ Transform")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_C2",
            type_: "https://uor.foundation/op/Identity",
            label: "T_C2",
            comment: "Category right identity: compose(f, id) = f.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("compose(f, id)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("f")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("f ∈ Transform")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_C3",
            type_: "https://uor.foundation/op/Identity",
            label: "T_C3",
            comment: "Category associativity of transform composition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("compose(f, compose(g, h))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("compose(compose(f, g), h)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("f, g, h ∈ Transform")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_C4",
            type_: "https://uor.foundation/op/Identity",
            label: "T_C4",
            comment: "Composability condition: f composesWith g iff target(f) = source(g).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("f composesWith g")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("target(f) = source(g)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("f, g ∈ Transform")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        // Amendment 17: Isometry laws (5)
        Individual {
            id: "https://uor.foundation/op/T_I1",
            type_: "https://uor.foundation/op/Identity",
            label: "T_I1",
            comment: "Negation is a ring-metric isometry.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_R(neg(x), neg(y))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("d_R(x, y)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_I2",
            type_: "https://uor.foundation/op/Identity",
            label: "T_I2",
            comment: "Bitwise complement is a Hamming-metric isometry.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_H(bnot(x), bnot(y))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("d_H(x, y)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_I3",
            type_: "https://uor.foundation/op/Identity",
            label: "T_I3",
            comment: "Successor as composed isometries: succ = neg ∘ bnot preserves d_R but not d_H.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("succ = neg ∘ bnot")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("preserves d_R but not d_H")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_I4",
            type_: "https://uor.foundation/op/Identity",
            label: "T_I4",
            comment: "Ring isometries form a group under composition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ring isometries")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("form a group under composition")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Isometry")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_I5",
            type_: "https://uor.foundation/op/Identity",
            label: "T_I5",
            comment: "CurvatureObservable measures failure of ring isometry to be Hamming isometry.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("CurvatureObservable")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("measures failure of ring isometry to be Hamming isometry")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Isometry")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        // Amendment 17: Embedding laws (4)
        Individual {
            id: "https://uor.foundation/op/T_E1",
            type_: "https://uor.foundation/op/Identity",
            label: "T_E1",
            comment: "Embedding injectivity.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ι(x) = ι(y)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x = y")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n (injectivity)")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_E2",
            type_: "https://uor.foundation/op/Identity",
            label: "T_E2",
            comment: "Embedding is a ring homomorphism.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ι(add(x,y))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("add(ι(x), ι(y)); ι(mul(x,y)) = mul(ι(x), ι(y))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_E3",
            type_: "https://uor.foundation/op/Identity",
            label: "T_E3",
            comment: "Embedding transitivity: composition of embeddings is an embedding.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ι₂ ∘ ι₁ : R_n → R_k")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("is an embedding (transitivity)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ι₁: R_n → R_m, ι₂: R_m → R_k")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_E4",
            type_: "https://uor.foundation/op/Identity",
            label: "T_E4",
            comment: "Embedding address coherence: glyph ∘ ι ∘ addresses is well-defined.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("glyph ∘ ι ∘ addresses")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("well-defined")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("embedding ι")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        // Amendment 17: Group action (4)
        Individual {
            id: "https://uor.foundation/op/T_A1",
            type_: "https://uor.foundation/op/Identity",
            label: "T_A1",
            comment: "Dihedral group acts on constraints by transforming them.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("g ∈ D_{2^n} on Constraint C")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("g⋅C (transformed constraint)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("g ∈ D_{2^n}, C ∈ Constraint")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_A2",
            type_: "https://uor.foundation/op/Identity",
            label: "T_A2",
            comment: "Dihedral group action on partitions permutes components.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("g ∈ D_{2^n} on Partition")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("permutes components")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("g ∈ D_{2^n}")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_A3",
            type_: "https://uor.foundation/op/Identity",
            label: "T_A3",
            comment: "Dihedral orbits determine irreducibility boundaries.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("D_{2^n} orbits on R_n")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("determine irreducibility boundaries")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("DihedralFactorizationResolver")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/T_A4",
            type_: "https://uor.foundation/op/Identity",
            label: "T_A4",
            comment: "Fixed points of negation are {0, 2^{n−1}}; bnot has no fixed points.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("fixed points of neg")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("{0, 2^{n−1}}; bnot has none (n > 0)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        // Amendment 17: Automorphism group (5)
        Individual {
            id: "https://uor.foundation/op/AU_1",
            type_: "https://uor.foundation/op/Identity",
            label: "AU_1",
            comment: "Automorphism group consists of unit multiplications.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Aut(R_n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("{μ_u : x ↦ mul(u, x) | u ∈ R_n×}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AU_2",
            type_: "https://uor.foundation/op/Identity",
            label: "AU_2",
            comment: "Automorphism group is isomorphic to the unit group.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Aut(R_n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≅ R_n× ≅ Z/2 × Z/2^{n−2}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 3")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AU_3",
            type_: "https://uor.foundation/op/Identity",
            label: "AU_3",
            comment: "Automorphism group order is 2^{n−1}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("|Aut(R_n)|")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("2^{n−1}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AU_4",
            type_: "https://uor.foundation/op/Identity",
            label: "AU_4",
            comment: "Intersection of automorphism group with dihedral group is {id, neg}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Aut(R_n) ∩ D_{2^n}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("{id, neg}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AU_5",
            type_: "https://uor.foundation/op/Identity",
            label: "AU_5",
            comment: "Affine group is generated by D_{2^n} and μ_3.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Aff(R_n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("⟨D_{2^n}, μ_3⟩")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        // Amendment 17: Embedding functors (7)
        Individual {
            id: "https://uor.foundation/op/EF_1",
            type_: "https://uor.foundation/op/Identity",
            label: "EF_1",
            comment: "Embedding functor action on morphisms.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("F_ι(f)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ι ∘ f ∘ ι⁻¹ on Im(ι)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ι: R_n → R_m, f ∈ Cat(R_n)")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EF_2",
            type_: "https://uor.foundation/op/Identity",
            label: "EF_2",
            comment: "Embedding functor preserves composition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("F_ι(f ∘ g)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("F_ι(f) ∘ F_ι(g)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ι: R_n → R_m")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EF_3",
            type_: "https://uor.foundation/op/Identity",
            label: "EF_3",
            comment: "Embedding functor preserves identities.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("F_ι(id_{R_n})")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("id_{Im(ι)}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ι: R_n → R_m")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EF_4",
            type_: "https://uor.foundation/op/Identity",
            label: "EF_4",
            comment: "Embedding functor composition is functorial.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("F_{ι₂ ∘ ι₁}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("F_{ι₂} ∘ F_{ι₁}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ι₁: R_n → R_m, ι₂: R_m → R_k")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EF_5",
            type_: "https://uor.foundation/op/Identity",
            label: "EF_5",
            comment: "Embedding functor preserves ring isometries.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("F_ι(ring isometry)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ring isometry at level m")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ι: R_n → R_m")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EF_6",
            type_: "https://uor.foundation/op/Identity",
            label: "EF_6",
            comment: "Embedding functor maps dihedral subgroup into target dihedral group.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("F_ι(D_{2^n})")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("⊆ D_{2^m} as subgroup")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ι: R_n → R_m")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EF_7",
            type_: "https://uor.foundation/op/Identity",
            label: "EF_7",
            comment: "Embedding functor maps unit group into target unit group.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("F_ι(R_n×)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("⊆ R_m× as subgroup")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ι: R_n → R_m")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Geometric")),
            ],
        },
        // Amendment 17: Address arithmetic (6)
        Individual {
            id: "https://uor.foundation/op/AA_1",
            type_: "https://uor.foundation/op/Identity",
            label: "AA_1",
            comment: "Braille glyph encoding: 6-bit blocks to Braille characters.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("glyph(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("[braille(x[0:5]), braille(x[6:11]), ...]")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n (6-bit blocks)")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AA_2",
            type_: "https://uor.foundation/op/Identity",
            label: "AA_2",
            comment: "Braille XOR homomorphism: Braille encoding commutes with XOR.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("braille(a ⊕ b)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("braille(a) ⊕ braille(b)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("a, b ∈ {0,1}^6")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AA_3",
            type_: "https://uor.foundation/op/Identity",
            label: "AA_3",
            comment: "Braille complement: glyph of bnot(x) is character-wise complement of glyph(x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("glyph(bnot(x))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("complement each Braille character of glyph(x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AA_4",
            type_: "https://uor.foundation/op/Identity",
            label: "AA_4",
            comment: "Addition does not lift to address space: no glyph homomorphism for add.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("glyph(add(x, y))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≠ f(glyph(x), glyph(y)) in general")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AA_5",
            type_: "https://uor.foundation/op/Identity",
            label: "AA_5",
            comment: "Liftable operations are exactly the Boolean operations.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("liftable operations")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("{xor, and, or, bnot}; NOT {add, sub, mul, neg, succ, pred}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("operations on R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AA_6",
            type_: "https://uor.foundation/op/Identity",
            label: "AA_6",
            comment: "Negation decomposes into liftable bnot and non-liftable succ.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("neg(x) = succ(bnot(x))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("bnot lifts, succ does not")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 17: Address metric (4)
        Individual {
            id: "https://uor.foundation/op/AM_1",
            type_: "https://uor.foundation/op/Identity",
            label: "AM_1",
            comment: "Address metric is sum of per-character Hamming distances.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_addr(a, b)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Σ_i popcount(braille_i(a) ⊕ braille_i(b))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("addresses a, b")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AM_2",
            type_: "https://uor.foundation/op/Identity",
            label: "AM_2",
            comment: "Address metric equals Hamming metric on ring elements.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_addr(glyph(x), glyph(y))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("d_H(x, y)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AM_3",
            type_: "https://uor.foundation/op/Identity",
            label: "AM_3",
            comment: "Address metric does not preserve ring metric in general.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_addr")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("does NOT preserve d_R in general")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("addresses")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AM_4",
            type_: "https://uor.foundation/op/Identity",
            label: "AM_4",
            comment: "Address incompatibility metric.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_Δ(x, y)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("|d_R(x,y) − d_addr(glyph(x), glyph(y))|")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 17: Thermodynamic (10)
        Individual {
            id: "https://uor.foundation/op/TH_1",
            type_: "https://uor.foundation/op/Identity",
            label: "TH_1",
            comment: "Entropy of a site budget state.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("S(state)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("freeRank × ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("state ∈ FreeRank")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TH_2",
            type_: "https://uor.foundation/op/Identity",
            label: "TH_2",
            comment: "Maximum entropy: unconstrained state has S = n × ln 2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("S(⊥)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("n × ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("unconstrained type")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TH_3",
            type_: "https://uor.foundation/op/Identity",
            label: "TH_3",
            comment: "Zero entropy: fully resolved state has S = 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("S(⊤)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("fully resolved type")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TH_4",
            type_: "https://uor.foundation/op/Identity",
            label: "TH_4",
            comment: "Landauer bound on total resolution cost.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("total resolution cost")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("n × k_B T × ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Landauer bound")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TH_5",
            type_: "https://uor.foundation/op/Identity",
            label: "TH_5",
            comment: "Critical inverse temperature for UOR site system.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("β*")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("UOR site system")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TH_6",
            type_: "https://uor.foundation/op/Identity",
            label: "TH_6",
            comment: "Constraint application removes entropy; convergence rate equals cooling rate.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("constraint application")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("removes entropy; convergenceRate = cooling rate")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("resolution loop")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TH_7",
            type_: "https://uor.foundation/op/Identity",
            label: "TH_7",
            comment: "CatastropheThreshold is the temperature of a partition phase transition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("CatastropheThreshold")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("temperature of partition phase transition")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("partition bifurcation")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TH_8",
            type_: "https://uor.foundation/op/Identity",
            label: "TH_8",
            comment: "Step formula as free-energy cost of a constraint boundary.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("step_g")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("free-energy cost of constraint boundary")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint boundary g")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TH_9",
            type_: "https://uor.foundation/op/Identity",
            label: "TH_9",
            comment: "Computational hardness maps to type incompleteness (high temperature).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("computational hardness")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("type incompleteness (high temperature)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("type specification")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TH_10",
            type_: "https://uor.foundation/op/Identity",
            label: "TH_10",
            comment: "Type resolution is measurement; cost ≥ entropy removed.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("type resolution")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("measurement; cost ≥ entropy removed")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("resolution process")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        // Amendment 17: Adiabatic (5)
        Individual {
            id: "https://uor.foundation/op/AR_1",
            type_: "https://uor.foundation/op/Identity",
            label: "AR_1",
            comment: "Adiabatic schedule: decreasing freeRank, cost-per-site ordering.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("adiabatic schedule")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("decreasing freeRank × cost-per-site order")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint ordering")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AR_2",
            type_: "https://uor.foundation/op/Identity",
            label: "AR_2",
            comment: "Adiabatic cost is sum of constraint costs in optimal order.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Cost_adiabatic")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Σ_i cost(C_{σ(i)}) where σ is optimal")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("optimal ordering")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AR_3",
            type_: "https://uor.foundation/op/Identity",
            label: "AR_3",
            comment: "Adiabatic cost satisfies Landauer bound.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Cost_adiabatic")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≥ n × k_B T × ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Landauer bound")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AR_4",
            type_: "https://uor.foundation/op/Identity",
            label: "AR_4",
            comment: "Adiabatic efficiency η ≤ 1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("η = (n × ln 2) / Cost_adiabatic")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("adiabatic efficiency")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AR_5",
            type_: "https://uor.foundation/op/Identity",
            label: "AR_5",
            comment: "Greedy vs adiabatic cost difference: ≤ 5% for n ≤ 16.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("greedy vs adiabatic difference")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ 5% for n ≤ 16")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("empirical, Q0–Q4")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 17: Phase diagram (5)
        Individual {
            id: "https://uor.foundation/op/PD_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PD_1",
            comment: "Phase space definition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("phase space")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("{(n, g) : n ∈ Z₊, g constraint boundary}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("UOR phase diagram")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PD_2",
            type_: "https://uor.foundation/op/Identity",
            label: "PD_2",
            comment: "Phase boundaries occur where g divides 2^n − 1 or g is a power of 2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("phase boundaries")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("g | (2^n − 1) or g = 2^k")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("(n, g) plane")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PD_3",
            type_: "https://uor.foundation/op/Identity",
            label: "PD_3",
            comment: "Parity boundary divides R_n into equal halves.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("parity boundary")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("|Unit| = 2^{n−1}, |non-Unit| = 2^{n−1}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("g = 2")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PD_4",
            type_: "https://uor.foundation/op/Identity",
            label: "PD_4",
            comment: "Resonance lines in the phase diagram.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resonance lines")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("n = k ⋅ ord_g(2)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("(n, g) plane")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PD_5",
            type_: "https://uor.foundation/op/Identity",
            label: "PD_5",
            comment: "Phase count at level n is at most 2^n (typically O(n)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("phase count at level n")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ 2^n (typical O(n))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("quantum level n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        // Amendment 17: Reversible computation (5)
        Individual {
            id: "https://uor.foundation/op/RC_1",
            type_: "https://uor.foundation/op/Identity",
            label: "RC_1",
            comment: "Reversible pinning stores prior state in ancilla site.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("reversible pinning of site k")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("store prior state in ancilla site k'")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("SiteIndex k")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/RC_2",
            type_: "https://uor.foundation/op/Identity",
            label: "RC_2",
            comment: "Reversible pinning has zero total entropy change.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("reversible pinning entropy")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ΔS_total = 0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("reversible strategy")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/RC_3",
            type_: "https://uor.foundation/op/Identity",
            label: "RC_3",
            comment: "Deferred Landauer cost at ancilla erasure.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("deferred Landauer cost")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("n × k_B T × ln 2 at ancilla erasure")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ancilla cleanup")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/RC_4",
            type_: "https://uor.foundation/op/Identity",
            label: "RC_4",
            comment: "Reversible total cost equals irreversible total cost (redistributed).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("reversible total cost")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("= irreversible total cost (redistributed)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("reversible strategy")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/RC_5",
            type_: "https://uor.foundation/op/Identity",
            label: "RC_5",
            comment: "Quantum UOR: superposed sites, cost proportional to winning path.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("quantum UOR")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("superposed sites, cost ∝ winning path")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("hypothetical quantum")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic")),
            ],
        },
        // Amendment 19: Analytical Identities — Differential calculus (11)
        Individual {
            id: "https://uor.foundation/op/DC_1",
            type_: "https://uor.foundation/op/Identity",
            label: "DC_1",
            comment: "Ring derivative: ∂_R f(x) = f(succ(x)) - f(x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("∂_R f(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("f(succ(x)) - f(x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("f : R_n → R_n, x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DC_2",
            type_: "https://uor.foundation/op/Identity",
            label: "DC_2",
            comment: "Hamming derivative: ∂_H f(x) = f(bnot(x)) - f(x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("∂_H f(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("f(bnot(x)) - f(x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("f : R_n → R_n, x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DC_3",
            type_: "https://uor.foundation/op/Identity",
            label: "DC_3",
            comment: "Hamming derivative of identity: ∂_H id(x) = -(2x + 1) mod 2^n.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("∂_H id(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("bnot(x) - x = -(2x + 1) mod 2^n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DC_4",
            type_: "https://uor.foundation/op/Identity",
            label: "DC_4",
            comment: "Commutator from derivatives: [neg, bnot](x) = ∂_R neg(x) - ∂_H neg(x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("[neg, bnot](x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("∂_R neg(x) - ∂_H neg(x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DC_5",
            type_: "https://uor.foundation/op/Identity",
            label: "DC_5",
            comment: "Carry dependence: the difference ∂_R f - ∂_H f decomposes into carry contributions.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("∂_R f - ∂_H f")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Σ carry contributions")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("f : R_n → R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DC_6",
            type_: "https://uor.foundation/op/Identity",
            label: "DC_6",
            comment: "Jacobian definition: J_k(x) = ∂_R f_k(x) where f_k = site_k.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("J_k(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("∂_R f_k(x) where f_k = site_k")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n, 0 ≤ k < n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DC_7",
            type_: "https://uor.foundation/op/Identity",
            label: "DC_7",
            comment: "Top-site anomaly: J_{n-1}(x) may differ from lower sites.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("J_{n-1}(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("may differ from lower sites")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DC_8",
            type_: "https://uor.foundation/op/Identity",
            label: "DC_8",
            comment: "Rank-curvature identity: rank(J(x)) = d_H(x, succ(x)) - 1 for generic x.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("rank(J(x))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("= d_H(x, succ(x)) - 1 for generic x")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DC_9",
            type_: "https://uor.foundation/op/Identity",
            label: "DC_9",
            comment: "Total curvature from Jacobian: κ(x) = Σ_k J_k(x).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("κ(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Σ_k J_k(x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DC_10",
            type_: "https://uor.foundation/op/Identity",
            label: "DC_10",
            comment: "Curvature-weighted ordering: optimal next constraint maximizes J_k over free sites.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("optimal next constraint")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("argmax J_k over free sites")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("resolution step")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DC_11",
            type_: "https://uor.foundation/op/Identity",
            label: "DC_11",
            comment: "Curvature equipartition: Σ_{x} J_k(x) ≈ (2^n - 2)/n for each k.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Σ_{x} J_k(x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≈ (2^n - 2)/n for each k")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("0 ≤ k < n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
            ],
        },
        // Amendment 19: Homological algebra (3)
        Individual {
            id: "https://uor.foundation/op/HA_1",
            type_: "https://uor.foundation/op/Identity",
            label: "HA_1",
            comment: "Constraint nerve: N(C) is the simplicial complex on constraints.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("N(C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("simplicial complex on constraints")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint set C")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HA_2",
            type_: "https://uor.foundation/op/Identity",
            label: "HA_2",
            comment: "Stall iff non-trivial homology: resolution stalls ⟺ H_k(N(C)) ≠ 0 for some k > 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resolution stalls")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("⟺ H_k(N(C)) ≠ 0 for some k > 0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint set C")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HA_3",
            type_: "https://uor.foundation/op/Identity",
            label: "HA_3",
            comment: "Betti-entropy theorem: S_residual ≥ Σ_k β_k × ln 2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("S_residual")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≥ Σ_k β_k × ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint configuration C")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        // Amendment 19: Index theorem (7)
        Individual {
            id: "https://uor.foundation/op/IT_2",
            type_: "https://uor.foundation/op/Identity",
            label: "IT_2",
            comment: "Euler-Poincaré formula for constraint nerve.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("χ(N(C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Σ_k (-1)^k β_k")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint nerve N(C)")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IT_3",
            type_: "https://uor.foundation/op/Identity",
            label: "IT_3",
            comment: "Spectral Euler characteristic.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("χ(N(C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Σ_k (-1)^k dim(H_k)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint nerve N(C)")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IT_6",
            type_: "https://uor.foundation/op/Identity",
            label: "IT_6",
            comment: "Spectral gap bounds convergence rate from below.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("λ_1(N(C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("lower bounds convergence rate")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint nerve N(C)")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IT_7a",
            type_: "https://uor.foundation/op/Identity",
            label: "IT_7a",
            comment: "UOR index theorem (topological form): total curvature minus Euler \
                      characteristic equals residual entropy in bits.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Σ κ_k - χ(N(C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("= S_residual / ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint configuration C")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IT_7b",
            type_: "https://uor.foundation/op/Identity",
            label: "IT_7b",
            comment: "UOR index theorem (entropy-topology duality).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("S_residual")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("= (Σ κ_k - χ) × ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint configuration C")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IT_7c",
            type_: "https://uor.foundation/op/Identity",
            label: "IT_7c",
            comment: "UOR index theorem (spectral cost bound): resolution cost ≥ n - χ(N(C)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resolution cost")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≥ n - χ(N(C))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint configuration C")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IT_7d",
            type_: "https://uor.foundation/op/Identity",
            label: "IT_7d",
            comment: "UOR index theorem (completeness criterion): resolution is complete \
                      iff χ(N(C)) = n and all Betti numbers vanish.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resolution complete")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("⟺ χ(N(C)) = n and all β_k = 0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint nerve N(C)")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        // Amendment 20: Inter-algebra Map Formalization (6)
        Individual {
            id: "https://uor.foundation/op/phi_1",
            type_: "https://uor.foundation/op/Identity",
            label: "phi_1",
            comment: "Ring → Constraints map: negation transforms a residue constraint.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("φ₁(neg, ResidueConstraint(m,r))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ResidueConstraint(m, m-r)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ring op, constraint")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/phi_2",
            type_: "https://uor.foundation/op/Identity",
            label: "phi_2",
            comment: "Constraints → Sites map: composition maps to union of site pinnings.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("φ₂(compose(A,B))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("φ₂(A) ∪ φ₂(B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraints A, B")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/phi_3",
            type_: "https://uor.foundation/op/Identity",
            label: "phi_3",
            comment: "Sites → Partition map: a closed site state determines a unique 4-component partition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("φ₃(closed site state)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("unique 4-component partition")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("closed FreeRank")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/phi_4",
            type_: "https://uor.foundation/op/Identity",
            label: "phi_4",
            comment: "Resolution pipeline: φ₄ = φ₃ ∘ φ₂ ∘ φ₁ is the composite resolution map.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("φ₄(T, x)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("φ₃(φ₂(φ₁(T, x)))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T ∈ T_n, x ∈ R_n")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/phi_5",
            type_: "https://uor.foundation/op/Identity",
            label: "phi_5",
            comment: "Operations → Observables map: negation preserves d_R, may change d_H.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("φ₅(neg)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("preserves d_R, may change d_H")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("op ∈ Operation")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/phi_6",
            type_: "https://uor.foundation/op/Identity",
            label: "phi_6",
            comment: "Observables → Refinement map: observables on a state yield a refinement suggestion.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("φ₆(state, observables)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("RefinementSuggestion")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ResolutionState")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
            ],
        },
        // Amendment 21: ψ-pipeline individuals (homological algebra)
        Individual {
            id: "https://uor.foundation/op/psi_1",
            type_: "https://uor.foundation/op/Identity",
            label: "psi_1",
            comment: "ψ_1: Constraints → SimplicialComplex (nerve construction). \
                      Maps a set of constraints to its nerve simplicial complex.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("N(constraints)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("SimplicialComplex")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint set")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/psi_2",
            type_: "https://uor.foundation/op/Identity",
            label: "psi_2",
            comment: "ψ_2: SimplicialComplex → ChainComplex (chain functor). \
                      Maps a simplicial complex to its chain complex.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("C(K)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ChainComplex")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("simplicial complex K")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/psi_3",
            type_: "https://uor.foundation/op/Identity",
            label: "psi_3",
            comment: "ψ_3: ChainComplex → HomologyGroups (homology functor). \
                      Computes homology groups from a chain complex.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("H_k(C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ker(∂_k) / im(∂_{k+1})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("chain complex C")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/psi_5",
            type_: "https://uor.foundation/op/Identity",
            label: "psi_5",
            comment: "ψ_5: ChainComplex → CochainComplex (dualization functor). \
                      Dualizes a chain complex to obtain a cochain complex.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("C^k")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Hom(C_k, R)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("chain complex C, ring R")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/psi_6",
            type_: "https://uor.foundation/op/Identity",
            label: "psi_6",
            comment: "ψ_6: CochainComplex → CohomologyGroups (cohomology functor). \
                      Computes cohomology groups from a cochain complex.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("H^k(C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ker(δ^k) / im(δ^{k-1})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("cochain complex C")),
                ("https://uor.foundation/op/verificationDomain", IndividualValue::IriRef("https://uor.foundation/op/Topological")),
            ],
        },
        // Amendment 4: D2n individual
        Individual {
            id: "https://uor.foundation/op/D2n",
            type_: "https://uor.foundation/op/DihedralGroup",
            label: "D_{2^n}",
            comment: "The dihedral group of order 2^(n+1), generated by neg (ring \
                      reflection) and bnot (hypercube reflection). Every element of \
                      this group acts as an isometry on the type space 𝒯_n.",
            properties: &[
                (
                    "https://uor.foundation/op/generatedBy",
                    IndividualValue::IriRef("https://uor.foundation/op/neg"),
                ),
                (
                    "https://uor.foundation/op/generatedBy",
                    IndividualValue::IriRef("https://uor.foundation/op/bnot"),
                ),
                (
                    "https://uor.foundation/op/presentation",
                    IndividualValue::Str("⟨r, s | r^{2^n} = s² = e, srs = r⁻¹⟩"),
                ),
            ],
        },
        // Gap C: Surface Symmetry Theorem
        Individual {
            id: "https://uor.foundation/op/surfaceSymmetry",
            type_: "https://uor.foundation/op/Identity",
            label: "Surface Symmetry",
            comment: "The Surface Symmetry Theorem: the composite P∘Π∘G is a well-typed \
                      morphism iff G and P share the same state:Frame F. When the \
                      shared-frame condition holds, the output lands in the type-equivalent \
                      neighbourhood of the source symbol.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("P(Π(G(s)))"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("s' where type(s') ≡ type(s) under F.constraint"),
                ),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str(
                        "G: GroundingMap, P: ProjectionMap, F: Frame, s: Literal, \
                         G.symbolConstraints = P.projectionOrder",
                    ),
                ),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        // Amendment 25: Completeness Certification algebra (CC_1–CC_5)
        Individual {
            id: "https://uor.foundation/op/CC_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CC_1",
            comment: "Completeness implies O(1) resolution: a CompleteType T satisfies \
                      ∀ x ∈ R_n, resolution(x, T) terminates in O(1).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resolution(x, T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("O(1) for CompleteType T")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n, T: CompleteType")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CC_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CC_2",
            comment: "Completeness is monotone: if T ⊆ T' (T' has more constraints), \
                      then completeness(T) implies completeness(T').",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("completeness(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("implies completeness(T')")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T, T': ConstrainedType, T ⊆ T'")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CC_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CC_3",
            comment: "Witness composition: concat(W₁, W₂) is a valid audit trail iff \
                      W₁.sitesClosed + W₂.sitesClosed = n.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("sitesClosed(W₁) + sitesClosed(W₂)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("= n for valid concat(W₁, W₂)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("W₁, W₂: CompletenessWitness")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CC_4",
            type_: "https://uor.foundation/op/Identity",
            label: "CC_4",
            comment: "The CompletenessResolver is the unique fixed point of the \
                      ψ-pipeline applied to a CompletenessCandidate.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("CompletenessResolver")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("fix(ψ-pipeline, CompletenessCandidate)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("CompletenessCandidate")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CC_5",
            type_: "https://uor.foundation/op/Identity",
            label: "CC_5",
            comment: "CompletenessCertificate soundness: cert.verified = true implies \
                      χ(N(C)) = n and for all k: β_k = 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cert.verified = true")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("implies χ(N(C)) = n ∧ ∀k: β_k = 0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("cert: CompletenessCertificate")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        // Amendment 26: Quantum Level Scaling identities (QL_1–QL_7)
        Individual {
            id: "https://uor.foundation/op/QL_1",
            type_: "https://uor.foundation/op/Identity",
            label: "QL_1",
            comment: "neg(bnot(x)) = succ(x) holds in Z/(2ⁿ)Z for all n ≥ 1. \
                      Universal form of the Critical Identity across all quantum levels.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("neg(bnot(x))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("succ(x) in Z/(2ⁿ)Z")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n, n ≥ 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/QL_2",
            type_: "https://uor.foundation/op/Identity",
            label: "QL_2",
            comment: "The dihedral group D_{2ⁿ} generated by neg and bnot has order \
                      2ⁿ⁺¹ for all n ≥ 1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("|D_{2ⁿ}|")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("2ⁿ⁺¹ for all n ≥ 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("n ≥ 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/QL_3",
            type_: "https://uor.foundation/op/Identity",
            label: "QL_3",
            comment: "The reduction distribution P(j) = 2^{-j} is the Boltzmann \
                      distribution at β* = ln 2 for all n ≥ 1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("P(j) = 2^{-j}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Boltzmann distribution at β* = ln 2, all n ≥ 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("j ∈ R_n, n ≥ 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/QL_4",
            type_: "https://uor.foundation/op/Identity",
            label: "QL_4",
            comment: "The site budget of a PrimitiveType at quantum level n is \
                      exactly n (one site per bit).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("siteBudget(PrimitiveType, n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("= n (one site per bit)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("PrimitiveType, n ≥ 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/QL_5",
            type_: "https://uor.foundation/op/Identity",
            label: "QL_5",
            comment: "Resolution complexity for a CompleteType is O(1) for all n ≥ 1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resolution(CompleteType, n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("O(1) for all n ≥ 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("CompleteType, n ≥ 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/QL_6",
            type_: "https://uor.foundation/op/Identity",
            label: "QL_6",
            comment: "Content addressing is a bijection for all n ≥ 1 \
                      (AD_1/AD_2 universality).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("contentAddress(x, n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("bijection for all n ≥ 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x ∈ R_n, n ≥ 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/QL_7",
            type_: "https://uor.foundation/op/Identity",
            label: "QL_7",
            comment: "The ψ-pipeline produces a valid ChainComplex for any \
                      ConstrainedType at any quantum level n ≥ 1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ψ-pipeline(ConstrainedType, n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("valid ChainComplex for all n ≥ 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ConstrainedType, n ≥ 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 27: Session-Scoped Resolution algebra (SR_1–SR_5)
        Individual {
            id: "https://uor.foundation/op/GR_1",
            type_: "https://uor.foundation/op/Identity",
            label: "GR_1",
            comment: "Binding monotonicity: freeRank(B_{i+1}) ≤ freeRank(B_i) \
                      for all i in a Session.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("freeRank(B_{i+1})")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ freeRank(B_i)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("i in Session S")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GR_2",
            type_: "https://uor.foundation/op/Identity",
            label: "GR_2",
            comment: "Binding soundness: a Binding b is sound iff b.datum resolves \
                      under b.constraint in O(1).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("b.datum resolves under b.constraint")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("in O(1) iff Binding b is sound")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("b: Binding")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GR_3",
            type_: "https://uor.foundation/op/Identity",
            label: "GR_3",
            comment: "Session convergence: a Session S converges iff there exists i \
                      such that freeRank(B_i) = 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("∃ i: freeRank(B_i) = 0")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Session S converges")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Session S")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GR_4",
            type_: "https://uor.foundation/op/Identity",
            label: "GR_4",
            comment: "Context reset isolation: bindings in C_fresh are independent of \
                      bindings in C_prior after a SessionBoundary.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("bindings(C_fresh) ∩ bindings(C_prior)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("= ∅ after SessionBoundary")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("C_prior, C_fresh: Context, SessionBoundary event")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GR_5",
            type_: "https://uor.foundation/op/Identity",
            label: "GR_5",
            comment: "Contradiction detection: ContradictionBoundary fires iff there \
                      exist bindings b, b' with the same address, different datum, \
                      under the same constraint.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ContradictionBoundary")),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("iff ∃ b, b': same address, different datum, same constraint"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("b, b': Binding in same Context")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        // Amendment 28: ψ-Pipeline Inversion (Type Synthesis) identities
        Individual {
            id: "https://uor.foundation/op/TS_1",
            type_: "https://uor.foundation/op/Identity",
            label: "TS_1",
            comment: "Nerve realisability: for any target (χ*, β₀* = 1, β_k* = 0 for k ≥ 1) \
                      with χ* ≤ n, there exists a ConstrainedType T over R_n whose constraint \
                      nerve realises the target.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("nerve(T, target)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("∃ ConstrainedType T over R_n realising target")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("target: χ* ≤ n, β₀* = 1, β_k* = 0 for k ≥ 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TS_2",
            type_: "https://uor.foundation/op/Identity",
            label: "TS_2",
            comment: "Minimal basis bound: for the IT_7d target (χ* = n, all β* = 0), the \
                      MinimalConstraintBasis has size exactly n (one constraint per site \
                      position). No redundant constraints exist in the minimal basis.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("basisSize(T, IT_7d target)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("IT_7d target, n-site types")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TS_3",
            type_: "https://uor.foundation/op/Identity",
            label: "TS_3",
            comment: "Synthesis monotonicity: adding a constraint to a synthesis candidate \
                      never decreases the Euler characteristic of the resulting nerve \
                      (χ is monotone non-decreasing under constraint addition).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("χ(N(C + constraint))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≥ χ(N(C))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("C: synthesis candidate constraint set")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TS_4",
            type_: "https://uor.foundation/op/Identity",
            label: "TS_4",
            comment: "Synthesis convergence: the TypeSynthesisResolver terminates for any \
                      realisable target in at most n constraint additions (for n-site types).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("steps(TypeSynthesisResolver, target)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("target: realisable n-site type synthesis goal")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TS_5",
            type_: "https://uor.foundation/op/Identity",
            label: "TS_5",
            comment: "Synthesis–certification duality: a SynthesizedType T achieves the \
                      IT_7d target if and only if the CompletenessResolver certifies T as a \
                      CompleteType. The forward ψ-pipeline and the inverse TypeSynthesisResolver \
                      are dual computations.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("SynthesizedType achieves IT_7d")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("iff CompletenessResolver certifies CompleteType")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: SynthesizedType")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TS_6",
            type_: "https://uor.foundation/op/Identity",
            label: "TS_6",
            comment: "Jacobian-guided synthesis efficiency: using the Jacobian (DC_10) to \
                      select the next constraint reduces the expected number of synthesis \
                      steps from O(n²) (uninformed) to O(n log n) (Jacobian-guided).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("E[steps, Jacobian-guided synthesis]")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("O(n log n) vs O(n²) uninformed")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: n-site type synthesis goal")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TS_7",
            type_: "https://uor.foundation/op/Identity",
            label: "TS_7",
            comment: "Unreachable signatures: a Betti profile with β₀ = 0 is unreachable by \
                      any non-empty ConstrainedType — the nerve of a non-empty constraint set \
                      is always connected (β₀ ≥ 1).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("β₀(N(C)) for non-empty C")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≥ 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("C: non-empty constraint set")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        // Amendment 29: Quantum Level Spectral Sequence identities
        Individual {
            id: "https://uor.foundation/op/WLS_1",
            type_: "https://uor.foundation/op/Identity",
            label: "WLS_1",
            comment: "Lift unobstructedness criterion: WittLift T' is a CompleteType iff the \
                      spectral sequence E_r^{p,q} collapses at E_2 (d_2 = 0 and all higher \
                      differentials zero).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("WittLift T' is CompleteType")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("iff spectral sequence collapses at E_2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: CompleteType at Q_n, T': WittLift to Q_{n+1}")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WLS_2",
            type_: "https://uor.foundation/op/Identity",
            label: "WLS_2",
            comment: "Obstruction localisation: a non-trivial LiftObstruction is localised to \
                      a specific site at bit position n+1. The obstruction class lives in \
                      H²(N(C(T))) and is killed by adding one constraint involving the new site.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("non-trivial LiftObstruction location")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("specific site at bit position n+1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("non-trivial LiftObstruction")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WLS_3",
            type_: "https://uor.foundation/op/Identity",
            label: "WLS_3",
            comment: "Monotone lifting for trivially obstructed types: if T is a CompleteType \
                      at Q_n and its constraint set is closed under the Q_{n+1} extension map, \
                      then T' is a CompleteType at Q_{n+1} with basisSize(T') = basisSize(T) + 1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("basisSize(T') for trivial lift")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("basisSize(T) + 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: CompleteType at Q_n with closed constraint set")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WLS_4",
            type_: "https://uor.foundation/op/Identity",
            label: "WLS_4",
            comment: "Spectral sequence convergence bound: for constraint configurations of \
                      homological depth d (H_k(N(C(T))) = 0 for k > d), the spectral sequence \
                      converges by page E_{d+2}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("spectral sequence convergence page")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ E_{d+2}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("depth-d constraint configuration")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WLS_5",
            type_: "https://uor.foundation/op/Identity",
            label: "WLS_5",
            comment: "Universal identity preservation: every op:universallyValid identity holds \
                      in ℤ/(2^{n+1})ℤ with the lifted constraint set. The lift does not \
                      invalidate any certified universal identity.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("universallyValid identity in R_{n+1}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("holds with lifted constraint set")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("every op:universallyValid identity, WittLift T'")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WLS_6",
            type_: "https://uor.foundation/op/Identity",
            label: "WLS_6",
            comment: "ψ-pipeline universality for quantum lifts: the ψ-pipeline produces a \
                      valid ChainComplex for any WittLift T' — the chain complex of T' \
                      restricts to the chain complex of T on the base nerve, and the extension \
                      is well-formed by construction.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ψ-pipeline ChainComplex(T')")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("valid and restricts to ChainComplex(T) on base nerve")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T': any WittLift of a CompleteType T")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 30: Monodromy Observables identities
        Individual {
            id: "https://uor.foundation/op/MN_1",
            type_: "https://uor.foundation/op/Identity",
            label: "MN_1",
            comment: "Holonomy group containment: HolonomyGroup(T) ≤ D_{2^n} for all \
                      ConstrainedTypes T over R_n. The holonomy group is always a subgroup \
                      of the full dihedral group.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("HolonomyGroup(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ D_{2^n}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: ConstrainedType over R_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MN_2",
            type_: "https://uor.foundation/op/Identity",
            label: "MN_2",
            comment: "Additive flatness (extends OB_H1): if all constraints in T are additive \
                      (ResidueConstraint or DepthConstraint type), then HolonomyGroup(T) = {id} \
                      — T is a FlatType.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("HolonomyGroup(T) for additive constraints")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("{id} (trivial: T is FlatType)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: all ResidueConstraint or DepthConstraint")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MN_3",
            type_: "https://uor.foundation/op/Identity",
            label: "MN_3",
            comment: "Dihedral generation: if T contains both a neg-related and a bnot-related \
                      constraint in a common closed path, then HolonomyGroup(T) = D_{2^n} — T \
                      has full dihedral holonomy.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("HolonomyGroup(T) with neg + bnot in closed path")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("D_{2^n} (full dihedral holonomy)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: ConstrainedType with neg-related and bnot-related constraints in closed path")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MN_4",
            type_: "https://uor.foundation/op/Identity",
            label: "MN_4",
            comment: "Holonomy-Betti implication: HolonomyGroup(T) ≠ {id} ⟹ β₁(N(C(T))) ≥ 1. \
                      Non-trivial monodromy requires a topological loop. (Converse is false: \
                      β₁ ≥ 1 does not imply non-trivial holonomy.)",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("HolonomyGroup(T) ≠ {id}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("⟹ β₁(N(C(T))) ≥ 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: ConstrainedType")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MN_5",
            type_: "https://uor.foundation/op/Identity",
            label: "MN_5",
            comment: "CompleteType holonomy: a CompleteType (IT_7d: χ = n, all β = 0) has \
                      trivial holonomy. IT_7d implies FlatType because IT_7d requires β₁ = 0, \
                      which by MN_4 implies trivial monodromy.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("CompleteType (IT_7d) ⟹ β₁ = 0 ⟹ holonomy")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("trivial ⟹ FlatType")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: CompleteType")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MN_6",
            type_: "https://uor.foundation/op/Identity",
            label: "MN_6",
            comment: "Monodromy composition: if p₁ and p₂ are closed constraint paths, then \
                      monodromy(p₁ · p₂) = monodromy(p₁) · monodromy(p₂) in D_{2^n} (group \
                      homomorphism from loops to dihedral elements).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("monodromy(p₁ · p₂)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("monodromy(p₁) · monodromy(p₂) in D_{2^n}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("p₁, p₂: ClosedConstraintPath")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MN_7",
            type_: "https://uor.foundation/op/Identity",
            label: "MN_7",
            comment: "TwistedType obstruction class: the monodromy of a TwistedType contributes \
                      a non-zero class to H²(N(C(T')); ℤ/2ℤ) where T' is any WittLift of T. \
                      TwistedTypes always have non-trivial lift obstructions.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("TwistedType T ⟹ H²(N(C(T')); ℤ/2ℤ)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("non-zero class (non-trivial LiftObstruction)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T': any WittLift of TwistedType T")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        // Amendment 31: Product Type identities
        Individual {
            id: "https://uor.foundation/op/PT_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PT_1",
            comment: "Product type site additivity: siteBudget(A × B) = \
                      siteBudget(A) + siteBudget(B).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("siteBudget(A × B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("siteBudget(A) + siteBudget(B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("A, B: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PT_2",
            type_: "https://uor.foundation/op/Identity",
            label: "PT_2",
            comment: "Product type partition product: partition(A × B) = \
                      partition(A) ⊗ partition(B). v0.2.2 T1.3: grounds \
                      cert:PartitionCertificate — every PartitionCertificate \
                      attests the PartitionComponent classification \
                      (Irreducible / Reducible / Units / Exterior) derived \
                      from this partition-map identity.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("partition(A × B); grounds PartitionCertificate")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("partition(A) ⊗ partition(B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("A, B: TypeDefinition; \u{2200} cert:PartitionCertificate c: c.partitionComponent \u{2208} {Irreducible, Reducible, Units, Exterior}")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PT_3",
            type_: "https://uor.foundation/op/Identity",
            label: "PT_3",
            comment: "Product type Euler additivity: χ(N(C(A × B))) = \
                      χ(N(C(A))) + χ(N(C(B))).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("χ(N(C(A × B)))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("χ(N(C(A))) + χ(N(C(B)))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("A, B: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PT_4",
            type_: "https://uor.foundation/op/Identity",
            label: "PT_4",
            comment: "Product type entropy additivity: S(A × B) = S(A) + S(B).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("S(A × B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("S(A) + S(B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("A, B: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic"),
                ),
            ],
        },
        // Amendment 31: Sum Type identities
        Individual {
            id: "https://uor.foundation/op/ST_1",
            type_: "https://uor.foundation/op/Identity",
            label: "ST_1",
            comment: "Sum type site budget: siteBudget(A + B) = \
                      max(siteBudget(A), siteBudget(B)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("siteBudget(A + B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("max(siteBudget(A), siteBudget(B))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("A, B: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ST_2",
            type_: "https://uor.foundation/op/Identity",
            label: "ST_2",
            comment: "Sum type entropy: S(A + B) = ln 2 + max(S(A), S(B)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("S(A + B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ln 2 + max(S(A), S(B))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("A, B: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic"),
                ),
            ],
        },
        // Amendment 33: Grounded Context Limit algebra (SC_1–SC_7)
        Individual {
            id: "https://uor.foundation/op/GS_1",
            type_: "https://uor.foundation/op/Identity",
            label: "GS_1",
            comment: "Context temperature: T_ctx(C) = freeRank(C) × ln 2 / n.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("T_ctx(C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("freeRank(C) × ln 2 / n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("C: Context, n = siteBudget")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GS_2",
            type_: "https://uor.foundation/op/Identity",
            label: "GS_2",
            comment: "Grounding degree: σ(C) = (n − freeRank(C)) / n.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("σ(C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("(n − freeRank(C)) / n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("C: Context, n = siteBudget")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GS_3",
            type_: "https://uor.foundation/op/Identity",
            label: "GS_3",
            comment: "Grounding monotonicity: σ(B_{i+1}) ≥ σ(B_i) for all i in a Session.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("σ(B_{i+1})")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≥ σ(B_i)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("i in Session S")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GS_4",
            type_: "https://uor.foundation/op/Identity",
            label: "GS_4",
            comment: "Ground state equivalence: σ(C) = 1 ↔ freeRank(C) = 0 \
                      ↔ S(C) = 0 ↔ T_ctx(C) = 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("σ(C) = 1")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("freeRank(C) = 0 ↔ S(C) = 0 ↔ T_ctx(C) = 0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("C: Context")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GS_5",
            type_: "https://uor.foundation/op/Identity",
            label: "GS_5",
            comment: "O(1) resolution guarantee: freeRank(C) = 0 ∧ q.address ∈ \
                      bindings(C) → stepCount(q, C) = 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("stepCount(q, C) at freeRank(C) = 0")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("q: Query, C: GroundedContext")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GS_6",
            type_: "https://uor.foundation/op/Identity",
            label: "GS_6",
            comment: "Pre-reduction of effective budget: effectiveBudget(q, C) = \
                      max(0, siteBudget(q.type) − |pinnedSites(C) ∩ q.siteSet|).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("effectiveBudget(q, C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("max(0, siteBudget(q.type) − |pinnedSites(C) ∩ q.siteSet|)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("q: Query, C: Context")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GS_7",
            type_: "https://uor.foundation/op/Identity",
            label: "GS_7",
            comment: "Thermodynamic cooling cost: Cost_saturation(C) = n × k_B T × ln 2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Cost_saturation(C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("n × k_B T × ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("C: GroundedContext, n = siteBudget")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic"),
                ),
            ],
        },
        // Amendment 34: Morphospace Achievability algebra (MS_1–MS_5)
        Individual {
            id: "https://uor.foundation/op/MS_1",
            type_: "https://uor.foundation/op/Identity",
            label: "MS_1",
            comment: "Connectivity lower bound: β₀(N(C)) ≥ 1 for all non-empty C.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("β₀(N(C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≥ 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("C: non-empty ConstrainedType")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MS_2",
            type_: "https://uor.foundation/op/Identity",
            label: "MS_2",
            comment: "Euler capacity ceiling: χ(N(C)) ≤ n for all C at quantum level n.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("χ(N(C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≤ n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("C: ConstrainedType at quantum level n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MS_3",
            type_: "https://uor.foundation/op/Identity",
            label: "MS_3",
            comment: "Betti monotonicity under addition: χ(N(C + c)) ≥ χ(N(C)) \
                      for any constraint c added to C.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("χ(N(C + c))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("≥ χ(N(C))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("C: ConstrainedType, c: Constraint")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MS_4",
            type_: "https://uor.foundation/op/Identity",
            label: "MS_4",
            comment: "Level-relative achievability: a signature achievable at quantum \
                      level n remains achievable at level n+1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("achievable(χ*, β_k*, n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("→ achievable(χ*, β_k*, n+1)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("(χ*, β_k*) achievable at level n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MS_5",
            type_: "https://uor.foundation/op/Identity",
            label: "MS_5",
            comment: "Empirical completeness convergence: verified SynthesisSignature \
                      individuals converge to the exact morphospace boundary.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("verified SynthesisSignature set")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("→ exact morphospace boundary in the limit")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("all quantum levels")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        // Amendment 35: Computational Geodesic algebra (GD_1–GD_5)
        Individual {
            id: "https://uor.foundation/op/GD_1",
            type_: "https://uor.foundation/op/Identity",
            label: "GD_1",
            comment: "Geodesic condition: a ComputationTrace is a geodesic iff its \
                      steps are AR_1-ordered and each step selects the constraint \
                      maximising J_k over free sites (DC_10).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("isGeodesic(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("AR_1-ordered(T) ∧ DC_10-selected(T)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: ComputationTrace")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Analytical"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GD_2",
            type_: "https://uor.foundation/op/Identity",
            label: "GD_2",
            comment: "Geodesic entropy bound: ΔS_step(i) = ln 2 for every step i \
                      of a geodesic trace.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("ΔS_step(i) on geodesic")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("step i of GeodesicTrace T")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GD_3",
            type_: "https://uor.foundation/op/Identity",
            label: "GD_3",
            comment: "Total geodesic cost: Cost_geodesic(T) = freeRank_initial × \
                      k_B T ln 2 = TH_4.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Cost_geodesic(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("freeRank_initial × k_B T × ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: GeodesicTrace")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GD_4",
            type_: "https://uor.foundation/op/Identity",
            label: "GD_4",
            comment: "Geodesic uniqueness up to step-order equivalence: all geodesics \
                      for the same ConstrainedType share stepCount and constraint set.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Cost(T) for geodesic T")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("= Cost(T') for any geodesic T' on same type")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T, T': GeodesicTrace on same ConstrainedType")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Analytical"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GD_5",
            type_: "https://uor.foundation/op/Identity",
            label: "GD_5",
            comment: "Subgeodesic detectability: a trace is sub-geodesic iff ∃ step i \
                      where J_k(step_i) < max_{free sites} J_k(state_i).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("isSubgeodesic(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("∃ i: J_k(step_i) < max_{free} J_k(state_i)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T: ComputationTrace")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
            ],
        },
        // Amendment 36: Measurement Boundary algebra (QM_1–QM_4)
        Individual {
            id: "https://uor.foundation/op/QM_1",
            type_: "https://uor.foundation/op/Identity",
            label: "QM_1",
            comment: "Von Neumann–Landauer bridge: S_vonNeumann(ψ) = \
                      Cost_Landauer(collapse(ψ)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("S_vonNeumann(ψ)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Cost_Landauer(collapse(ψ))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ψ: SuperposedSiteState")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/QuantumThermodynamic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/QM_2",
            type_: "https://uor.foundation/op/Identity",
            label: "QM_2",
            comment: "Measurement as site topology change: projective collapse on a \
                      SuperposedSiteState is topologically equivalent to applying a \
                      ResidueConstraint that pins the collapsed site.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("collapse(ψ)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("apply(ResidueConstraint, collapsed_site)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ψ: SuperposedSiteState")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/QM_3",
            type_: "https://uor.foundation/op/Identity",
            label: "QM_3",
            comment: "Superposition entropy bound: 0 ≤ S_vN(ψ) ≤ ln 2 for any \
                      single-site SuperposedSiteState.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("S_vN(ψ)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("∈ [0, ln 2]")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ψ: single-site SuperposedSiteState")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/QuantumThermodynamic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/QM_4",
            type_: "https://uor.foundation/op/Identity",
            label: "QM_4",
            comment: "Collapse idempotence: collapse(collapse(ψ)) = collapse(ψ). \
                      Measurement on an already-collapsed state is a no-op.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("collapse(collapse(ψ))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("collapse(ψ)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ψ: SuperposedSiteState")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        // Amendment 37: Gap Closure identities (QM_5, RC_6, FPM_8–9, MN_8, QL_8, D_7, SP_1–4, PT_2a–2b, GD_6)
        Individual {
            id: "https://uor.foundation/op/QM_5",
            type_: "https://uor.foundation/op/Identity",
            label: "QM_5",
            comment: "Amplitude normalization (Born rule): the sum of squared \
                      amplitudes equals 1 for any well-formed SuperposedSiteState.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Σᵢ |αᵢ|²")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("SuperposedSiteState ψ")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/QuantumThermodynamic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/RC_6",
            type_: "https://uor.foundation/op/Identity",
            label: "RC_6",
            comment: "Amplitude renormalization: a SuperposedSiteState can always \
                      be normalized to satisfy QM_5.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("normalize(ψ)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ψ / sqrt(Σ |αᵢ|²)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("SuperposedSiteState ψ")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/SuperpositionDomain"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FPM_8",
            type_: "https://uor.foundation/op/Identity",
            label: "FPM_8",
            comment: "Partition exhaustiveness: the four component cardinalities \
                      sum to the ring size.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("|Irr| + |Red| + |Unit| + |Ext|")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("2ⁿ")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Partition P over R_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Enumerative"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FPM_9",
            type_: "https://uor.foundation/op/Identity",
            label: "FPM_9",
            comment: "Exterior membership criterion: x is exterior iff x is not \
                      in the carrier of T.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("x ∈ Ext(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("x ∉ carrier(T)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("TypeDefinition T, Datum x")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MN_8",
            type_: "https://uor.foundation/op/Identity",
            label: "MN_8",
            comment: "Holonomy classification covering: every ConstrainedType with \
                      a computed holonomy group is either flat or twisted, not both.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("holonomyClassified(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("isFlatType(T) xor isTwistedType(T)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ConstrainedType T with holonomyGroup")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/QL_8",
            type_: "https://uor.foundation/op/Identity",
            label: "QL_8",
            comment: "Witt level chain inverse: wittLevelPredecessor is the left \
                      inverse of nextWittLevel.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("wittLevelPredecessor(nextWittLevel(Q_k))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Q_k")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("WittLevel W_n with nextWittLevel defined")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/D_7",
            type_: "https://uor.foundation/op/Identity",
            label: "D_7",
            comment: "Dihedral composition rule: (rᵃ sᵖ)(rᵇ sᵠ) = \
                      r^(a + (-1)ᵖ b mod 2ⁿ) s^(p xor q).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("compose(rᵃ sᵖ, rᵇ sᵠ)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("r^((a + (-1)ᵖ b) mod 2ⁿ) s^(p xor q)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("DihedralElement rᵃ sᵖ, rᵇ sᵠ in D_{2ⁿ}")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Geometric"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SP_1",
            type_: "https://uor.foundation/op/Identity",
            label: "SP_1",
            comment: "Classical embedding: superposition resolution of a classical \
                      (non-superposed) datum reduces to classical resolution.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resolve_superposition(classical(x))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("resolve_classical(x)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Datum x")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/SuperpositionDomain"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SP_2",
            type_: "https://uor.foundation/op/Identity",
            label: "SP_2",
            comment: "Collapse–resolve commutativity: collapsing then resolving \
                      classically equals resolving in superposition then collapsing.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("collapse(resolve_superposition(ψ))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("resolve_classical(collapse(ψ))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("SuperposedSiteState ψ")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/QuantumThermodynamic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SP_3",
            type_: "https://uor.foundation/op/Identity",
            label: "SP_3",
            comment: "Amplitude preservation: the SuperpositionResolver preserves \
                      the normalized amplitude vector during ψ-pipeline traversal.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("amplitudeVector(resolve_superposition(ψ))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("normalized")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("SuperposedSiteState ψ")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/SuperpositionDomain"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SP_4",
            type_: "https://uor.foundation/op/Identity",
            label: "SP_4",
            comment: "Born rule outcome probability: the probability of collapsing \
                      to site k equals the squared amplitude of that site.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("P(collapse to site k)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("|α_k|²")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("SuperposedSiteState ψ, site index k")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/QuantumThermodynamic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PT_2a",
            type_: "https://uor.foundation/op/Identity",
            label: "PT_2a",
            comment: "Product type partition tensor: the partition of a product \
                      type is the tensor product of the component partitions.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Π(A × B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("PartitionProduct(Π(A), Π(B))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ProductType A × B")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PT_2b",
            type_: "https://uor.foundation/op/Identity",
            label: "PT_2b",
            comment: "Sum type partition coproduct: the partition of a sum type \
                      is the coproduct of the variant partitions.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Π(A + B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("PartitionCoproduct(Π(A), Π(B))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("SumType A + B")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GD_6",
            type_: "https://uor.foundation/op/Identity",
            label: "GD_6",
            comment: "Geodesic predicate decomposition: a trace is geodesic iff \
                      it is both AR_1-ordered and DC_10-selected.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("isGeodesic(trace)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("isAR1Ordered(trace) ∧ isDC10Selected(trace)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ComputationTrace trace")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Analytical"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 41: Tower identities (QT_ series)
        Individual {
            id: "https://uor.foundation/op/WT_1",
            type_: "https://uor.foundation/op/Identity",
            label: "WT_1",
            comment: "LiftChain(Q_j, Q_k) is valid CompleteType tower iff every \
                      chainStep WittLift has trivial or resolved LiftObstruction.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("LiftChain(Q_j, Q_k) valid")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("every chainStep has trivial or resolved LiftObstruction")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("LiftChain from Q_j to Q_k")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WT_2",
            type_: "https://uor.foundation/op/Identity",
            label: "WT_2",
            comment: "Obstruction count bound: the number of non-trivial \
                      LiftObstructions in a chain is at most the chain length.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("obstructionCount(chain)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("<= chainLength(chain)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("LiftChain")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WT_3",
            type_: "https://uor.foundation/op/Identity",
            label: "WT_3",
            comment: "Resolved basis size formula: the basis size at Q_k equals \
                      basisSize(Q_j) + chainLength + obstructionResolutionCost.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resolvedBasisSize(Q_k)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("basisSize(Q_j) + chainLength + obstructionResolutionCost")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("LiftChain with source Q_j, target Q_k")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WT_4",
            type_: "https://uor.foundation/op/Identity",
            label: "WT_4",
            comment: "Flat tower characterization: isFlat(chain) iff \
                      obstructionCount = 0 iff HolonomyGroup trivial at every step.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("isFlat(chain)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("obstructionCount = 0 iff HolonomyGroup trivial at every step")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("LiftChain")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WT_5",
            type_: "https://uor.foundation/op/Identity",
            label: "WT_5",
            comment: "LiftChainCertificate existence: a CompleteType at Q_k \
                      satisfies IT_7d with a witness chain.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("LiftChainCertificate exists")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("CompleteType at Q_k satisfies IT_7d with witness chain")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Q_k for arbitrary k")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WT_6",
            type_: "https://uor.foundation/op/Identity",
            label: "WT_6",
            comment: "Single-step reduction: QT_3 with chainLength=1 and cost=0 \
                      reduces to QLS_3.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("QT_3 with chainLength=1, cost=0")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("reduces to QLS_3")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Single-step chains")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WT_7",
            type_: "https://uor.foundation/op/Identity",
            label: "WT_7",
            comment: "Flat chain basis size: for flat chains, resolvedBasisSize(Q_k) = \
                      basisSize(Q_j) + (k - j).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("flat chain resolvedBasisSize(Q_k)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("basisSize(Q_j) + (k - j)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("LiftChain with isFlat = true")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind", IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 44: Structural Gap Closures (G1--G11)
        // G7: CarryConstraint site-pinning map
        Individual {
            id: "https://uor.foundation/op/CC_PINS",
            type_: "https://uor.foundation/op/Identity",
            label: "CC_PINS",
            comment: "Carry-constraint site-pinning map: \
                      pinsSites(CarryConstraint(p)) equals the set of bit \
                      positions where p has a 1 plus the first-zero stopping \
                      position.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("pinsSites(CarryConstraint(p))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("{k : p(k)=1} union {first-zero(p)}")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("bit-pattern p in CarryConstraint")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CC_COST_SITE",
            type_: "https://uor.foundation/op/Identity",
            label: "CC_COST_SITE",
            comment: "Carry-constraint cost-to-site count: the number of \
                      sites pinned by a CarryConstraint equals popcount plus \
                      one for the stopping position.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("|pinsSites(CarryConstraint(p))|")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("popcount(p) + 1")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("bit-pattern p in CarryConstraint")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Enumerative"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // G1: Nerve joint satisfiability decision procedure
        Individual {
            id: "https://uor.foundation/op/jsat_RR",
            type_: "https://uor.foundation/op/Identity",
            label: "jsat_RR",
            comment: "CRT joint satisfiability: two ResidueConstraints are \
                      jointly satisfiable iff the gcd of their moduli divides \
                      the difference of their residues.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("jointSat(Res(m1,r1), Res(m2,r2))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("gcd(m1,m2) | (r1 - r2)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("ResidueConstraint pairs over R_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/jsat_CR",
            type_: "https://uor.foundation/op/Identity",
            label: "jsat_CR",
            comment: "Carry-residue joint satisfiability: a CarryConstraint \
                      and ResidueConstraint are jointly satisfiable iff the \
                      carry-pinned sites are compatible with the residue \
                      class.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("jointSat(Carry(p), Res(m,r))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("pin-site intersection residue-class compatible")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("CarryConstraint, ResidueConstraint pairs")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/jsat_CC",
            type_: "https://uor.foundation/op/Identity",
            label: "jsat_CC",
            comment: "Carry-carry joint satisfiability: two CarryConstraints \
                      are jointly satisfiable iff their bit-patterns have no \
                      conflicting positions.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("jointSat(Carry(p1), Carry(p2))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("p1 AND p2 conflict-free")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("CarryConstraint pairs over R_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Enumerative"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // G2: DihedralElement inverse and order
        Individual {
            id: "https://uor.foundation/op/D_8",
            type_: "https://uor.foundation/op/Identity",
            label: "D_8",
            comment: "Dihedral inverse formula: the inverse of r^a s^p in \
                      D_(2^n) is r^(-(−1)^p a mod 2^n) s^p.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("(r^a s^p)^(-1)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("r^(-(−1)^p a mod 2^n) s^p")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("a in 0..2^n, p in {0,1}")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/D_9",
            type_: "https://uor.foundation/op/Identity",
            label: "D_9",
            comment: "Dihedral reflection order: every reflection element \
                      r^k s^1 in D_(2^n) has order 2.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("ord(r^k s^1)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("2")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("k in Z/(2^n)Z")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // G5: Constraint language expressiveness boundary
        Individual {
            id: "https://uor.foundation/op/EXP_1",
            type_: "https://uor.foundation/op/Identity",
            label: "EXP_1",
            comment: "Monotone carrier characterization: a ConstrainedType \
                      has an upward-closed carrier iff every \
                      ResidueConstraint has residue = modulus - 1 and no \
                      CarryConstraint or DepthConstraint appears.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("carrier(C) is monotone")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("all residues of C = modulus - 1, no Carry/Depth")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("ConstrainedType C over R_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EXP_2",
            type_: "https://uor.foundation/op/Identity",
            label: "EXP_2",
            comment: "Monotone constraint count: the number of expressible \
                      monotone ConstrainedTypes at quantum level Q_n is 2^n, \
                      corresponding to the principal filter count.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("count of monotone ConstrainedTypes at Q_n")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("2^n")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("WittLevel W_n, n >= 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Enumerative"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EXP_3",
            type_: "https://uor.foundation/op/Identity",
            label: "EXP_3",
            comment: "SumType carrier semantics: the carrier of a SumType is \
                      the coproduct (disjoint union) of component carriers, \
                      not the set-theoretic union.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("carrier(SumType(A,B))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("coproduct(carrier(A), carrier(B))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("SumType A + B")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // G4: SumType topological identity algebra
        Individual {
            id: "https://uor.foundation/op/ST_3",
            type_: "https://uor.foundation/op/Identity",
            label: "ST_3",
            comment: "SumType Euler characteristic additivity: for a SumType \
                      with topologically disjoint component nerves, the Euler \
                      characteristic is additive.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("chi(N(C(A+B)))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("chi(N(C(A))) + chi(N(C(B)))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("disjoint SumType A + B")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ST_4",
            type_: "https://uor.foundation/op/Identity",
            label: "ST_4",
            comment: "SumType Betti number additivity: for disjoint component \
                      nerves, all Betti numbers are additive.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("beta_k(N(C(A+B)))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("beta_k(N(C(A))) + beta_k(N(C(B)))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("disjoint SumType A + B, k >= 0")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ST_5",
            type_: "https://uor.foundation/op/Identity",
            label: "ST_5",
            comment: "SumType completeness transfer: a SumType A+B is \
                      CompleteType iff both A and B are CompleteType and they \
                      have equal quantum levels.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("CompleteType(A + B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("CompleteType(A) and CompleteType(B) and Q(A)=Q(B)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("SumType A + B")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Product/Coproduct Completion Amendment — Gap 4 (ST_6..ST_10)
        Individual {
            id: "https://uor.foundation/op/ST_6",
            type_: "https://uor.foundation/op/Identity",
            label: "ST_6",
            comment: "SumType tag site unique existence: every PartitionCoproduct \
                      A + B has exactly one tag site, logically distinct from \
                      every data site of A and B, carrying the ln 2 entropy \
                      quantum of ST_2.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("∃! tagSite(A + B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("uniqueSite ∉ dataSites(A) ∪ dataSites(B) ∧ carries ln 2 entropy (ST_2)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A, B: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ST_7",
            type_: "https://uor.foundation/op/Identity",
            label: "ST_7",
            comment: "SumType variant tagging: the constraints of a \
                      PartitionCoproduct decompose into the operands' \
                      constraints plus one tag-pinning constraint per variant \
                      (tag = 0 for the left variant, tag = 1 for the right).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("constraints(A + B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("constraints(A) ∪ {tag=0} ∪ constraints(B) ∪ {tag=1}")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A, B: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ST_8",
            type_: "https://uor.foundation/op/Identity",
            label: "ST_8",
            comment: "SumType variant nerve disjointness bridge: a \
                      PartitionCoproduct constructed via ST_6 + ST_7 + the \
                      foundation layout convention produces topologically \
                      disjoint component nerves. Bridges the existing \
                      conditional ST_3 and ST_4 to PartitionCoproduct \
                      constructions concretely.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("disjoint(N(C(A)), N(C(B)))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("true")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A + B constructed via ST_6 ∧ ST_7 ∧ layoutTagSite")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ST_9",
            type_: "https://uor.foundation/op/Identity",
            label: "ST_9",
            comment: "SumType Euler corollary for PartitionCoproduct \
                      constructions: combining ST_8's disjointness guarantee \
                      with the universal ST_3 yields Euler additivity for any \
                      A + B assembled via the PartitionCoproduct construction.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("χ(N(C(A + B)))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("χ(N(C(A))) + χ(N(C(B)))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A + B constructed via PartitionCoproduct")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ST_10",
            type_: "https://uor.foundation/op/Identity",
            label: "ST_10",
            comment: "SumType Betti corollary for PartitionCoproduct \
                      constructions: combining ST_8's disjointness guarantee \
                      with the universal ST_4 yields Betti additivity at every \
                      dimension for any A + B assembled via the \
                      PartitionCoproduct construction.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("β_k(A + B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("β_k(A) + β_k(B)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A + B constructed via PartitionCoproduct, k ≥ 0")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
            ],
        },
        // Product/Coproduct Completion Amendment — Gap 3 (CPT_1..CPT_6)
        Individual {
            id: "https://uor.foundation/op/CPT_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CPT_1",
            comment: "Cartesian partition product site additivity: \
                      siteBudget(A ⊠ B) = siteBudget(A) + siteBudget(B). \
                      UOR sites are bit widths; |A ⊠ B| = 2^{n_A} · 2^{n_B} = \
                      2^{n_A + n_B}, so sites add even though cardinalities \
                      multiply.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("siteBudget(A ⊠ B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("siteBudget(A) + siteBudget(B)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A, B: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CPT_2a",
            type_: "https://uor.foundation/op/Identity",
            label: "CPT_2a",
            comment: "Cartesian partition product partition map: \
                      Π(A ⊠ B) = CartesianPartitionProduct(Π(A), Π(B)).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("Π(A ⊠ B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("CartesianPartitionProduct(Π(A), Π(B))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A, B: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CPT_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CPT_3",
            comment: "Cartesian partition product Euler multiplicativity: \
                      χ(N(C(A ⊠ B))) = χ(N(C(A))) · χ(N(C(B))). Distinguishes \
                      CartesianPartitionProduct (multiplicative χ, Künneth) \
                      from PartitionProduct (additive χ, site-disjoint union).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("χ(N(C(A ⊠ B)))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("χ(N(C(A))) · χ(N(C(B)))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A, B: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CPT_4",
            type_: "https://uor.foundation/op/Identity",
            label: "CPT_4",
            comment: "Cartesian partition product Künneth formula: Betti \
                      numbers compose via the Künneth convolution — \
                      β_k(A ⊠ B) = Σ_{i+j=k} β_i(A) · β_j(B). Routed \
                      through primitive_cartesian_nerve_betti when a \
                      CartesianProductShape marker is declared on the \
                      combined shape.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("β_k(A ⊠ B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("Σ_{i+j=k} β_i(A) · β_j(B)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A, B: TypeDefinition, k ≥ 0")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CPT_5",
            type_: "https://uor.foundation/op/Identity",
            label: "CPT_5",
            comment: "Cartesian partition product entropy additivity: \
                      S(A ⊠ B) = S(A) + S(B). Matches PT_4's additive \
                      Shannon convention for independent subsystems and \
                      is consistent pointwise with IT_7a/b on the combined \
                      system (no cross-factor bilinearity required).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("S(A ⊠ B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("S(A) + S(B)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A, B: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CPT_6",
            type_: "https://uor.foundation/op/Identity",
            label: "CPT_6",
            comment: "Cartesian partition product distributes over \
                      PartitionCoproduct: A ⊠ (B + C) ≡ (A ⊠ B) + (A ⊠ C) \
                      with equality at the siteBudget, SITE_COUNT, Euler, \
                      and entropy levels simultaneously. Does NOT extend \
                      to PartitionProduct — distributing ⊠ over × fails \
                      at the site-budget level because it duplicates the \
                      outer factor.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("A ⊠ (B + C)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("(A ⊠ B) + (A ⊠ C)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A, B, C: TypeDefinition")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
            ],
        },
        // G3: TypeSynthesis reachability domain completeness
        Individual {
            id: "https://uor.foundation/op/TS_8",
            type_: "https://uor.foundation/op/Identity",
            label: "TS_8",
            comment: "Betti-1 minimum constraint count: the minimum number \
                      of constraints needed to achieve first Betti number \
                      beta_1 = k in the constraint nerve is 2k + 1.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("min constraints for beta_1 = k")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("2k + 1")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("first Betti number k >= 1, n-site type")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TS_9",
            type_: "https://uor.foundation/op/Identity",
            label: "TS_9",
            comment: "TypeSynthesisResolver termination: the resolver \
                      terminates in at most 2^n steps for any target \
                      signature at quantum level Q_n, returning either a \
                      ConstrainedType or a ForbiddenSignature certificate.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("TypeSynthesisResolver terminates")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("within 2^n steps")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("WittLevel W_n, any target signature")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TS_10",
            type_: "https://uor.foundation/op/Identity",
            label: "TS_10",
            comment: "ForbiddenSignature membership criterion: a topological \
                      signature is a ForbiddenSignature iff no \
                      ConstrainedType with at most n constraints realises it \
                      at quantum level Q_n.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("ForbiddenSignature(sigma)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("no ConstrainedType with <= n constraints realises sigma")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("topological signature sigma at Q_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // G6: ObstructionChain termination guarantee
        Individual {
            id: "https://uor.foundation/op/WT_8",
            type_: "https://uor.foundation/op/Identity",
            label: "WT_8",
            comment: "ObstructionChain length bound: the length of the \
                      ObstructionChain from Q_j to Q_k is at most \
                      (k-j) times C(basisSize(Q_j), 3).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("ObstructionChain length from Q_j to Q_k")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("<= (k-j) * C(basisSize(Q_j), 3)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("LiftChain from Q_j to Q_k")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WT_9",
            type_: "https://uor.foundation/op/Identity",
            label: "WT_9",
            comment: "TowerCompletenessResolver termination: the resolver \
                      terminates for any finite LiftChain within the QT_8 \
                      bound, producing a CompleteType certificate or a \
                      bounded ObstructionChain.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("TowerCompletenessResolver terminates")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("within QT_8 bound")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("LiftChain of finite length")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // G11: Sheaf coefficient ring grounding
        Individual {
            id: "https://uor.foundation/op/COEFF_1",
            type_: "https://uor.foundation/op/Identity",
            label: "COEFF_1",
            comment: "Standard coefficient ring: the coefficient ring for \
                      all psi-pipeline cohomology computations in \
                      uor.foundation is Z/2Z, consistent with MN_7.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("standard coefficient ring for psi-pipeline")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("Z/2Z")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("CechNerve N(C) at any quantum level")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // G9: GluingObstruction resolver feedback
        Individual {
            id: "https://uor.foundation/op/GO_1",
            type_: "https://uor.foundation/op/Identity",
            label: "GO_1",
            comment: "GluingObstruction feedback: given a \
                      GluingObstruction class in H^1(N(C)), the killing \
                      RefinementSuggestion adds a constraint whose pinned \
                      sites contain the intersection of the \
                      cycle-generating pair.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("pinsSites(killing constraint for obstruction c)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("superset of pinsSites(C_i) cap pinsSites(C_j)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("GluingObstruction c, cycle pair (C_i, C_j)")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // G8: Session saturation lifecycle bridge
        Individual {
            id: "https://uor.foundation/op/GR_6",
            type_: "https://uor.foundation/op/Identity",
            label: "GR_6",
            comment: "Grounding re-entry free rank: for a session at full \
                      grounding, a new query q has freeRank equal to the \
                      number of q's site coordinates not already bound.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("freeRank(q) after grounding")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("sites of q not in BindingAccumulator")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("grounded Session, new RelationQuery q")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GR_7",
            type_: "https://uor.foundation/op/Identity",
            label: "GR_7",
            comment: "Grounding degree degradation: after re-entry with \
                      query q, the grounding degree becomes \
                      min(current sigma, 1 - freeRank(q)/n).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("sigma after re-entry with query q")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("min(sigma, 1 - freeRank(q)/n)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("SessionResolver, new query q")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // G10: SuperposedSiteState amplitude index set
        Individual {
            id: "https://uor.foundation/op/QM_6",
            type_: "https://uor.foundation/op/Identity",
            label: "QM_6",
            comment: "Amplitude index set characterization: the amplitude \
                      index set of a SuperposedSiteState over \
                      ConstrainedType T at Q_n is the set of monotone \
                      pinning trajectories consistent with T's constraints.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("amplitude index set of SuperposedSiteState over T")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("monotone pinning trajectories consistent with T")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("SuperposedSiteState over ConstrainedType T at Q_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/SuperpositionDomain"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 46: Certificate Issuance Coverage identities
        Individual {
            id: "https://uor.foundation/op/CIC_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CIC_1",
            comment: "Certificate issuance: every valid Transform admits a \
                      TransformCertificate attesting correct source-to-target \
                      mapping.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("valid(T) \u{2227} T: Transform")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2203} c: TransformCertificate. certifies(c, T)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Transform T")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CIC_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CIC_2",
            comment: "Certificate issuance: every metric-preserving Transform \
                      admits an IsometryCertificate attesting distance \
                      preservation.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("isometry(T) \u{2227} metric-preserving(T)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2203} c: IsometryCertificate. certifies(c, T)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Isometry T")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Geometric"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CIC_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CIC_3",
            comment: "Certificate issuance: every involutive operation f where \
                      f(f(x)) = x admits an InvolutionCertificate.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("f(f(x)) = x \u{2200} x \u{2208} R_n")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2203} c: InvolutionCertificate. certifies(c, f)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Involution f")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CIC_4",
            type_: "https://uor.foundation/op/Identity",
            label: "CIC_4",
            comment: "Certificate issuance: full saturation (\u{03c3} = 1, \
                      freeRank = 0) admits a GroundingCertificate.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03c3}(C) = 1 \u{2227} freeRank = 0")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2203} c: GroundingCertificate. certifies(c, C)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("GroundedContext C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Thermodynamic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CIC_5",
            type_: "https://uor.foundation/op/Identity",
            label: "CIC_5",
            comment: "Certificate issuance: an AR_1-ordered and DC_10-selected \
                      trace admits a GeodesicCertificate.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("AR_1-ordered \u{2227} DC_10-selected trace")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2203} c: GeodesicCertificate. certifies(c, trace)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("GeodesicTrace")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CIC_6",
            type_: "https://uor.foundation/op/Identity",
            label: "CIC_6",
            comment: "Certificate issuance: a MeasurementEvent verifying the \
                      von Neumann\u{2013}Landauer bridge at \u{03b2}* = ln 2 admits \
                      a MeasurementCertificate.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("S_vN = L_cost at \u{03b2}* = ln 2")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2203} c: MeasurementCertificate. certifies(c, event)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("MeasurementEvent")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/QuantumThermodynamic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CIC_7",
            type_: "https://uor.foundation/op/Identity",
            label: "CIC_7",
            comment: "Certificate issuance: a MeasurementEvent verifying \
                      P(outcome k) = |\u{03b1}_k|\u{00b2} admits a \
                      BornRuleVerification certificate.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("P(k) = |\u{03b1}_k|\u{00b2} verified")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2203} c: BornRuleVerification. certifies(c, event)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("MeasurementEvent with amplitude vector")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/QuantumThermodynamic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GC_1",
            type_: "https://uor.foundation/op/Identity",
            label: "GC_1",
            comment: "Certificate issuance: shared-frame grounding that lands \
                      in the type-equivalent neighbourhood admits a \
                      GroundingCertificate.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("shared-frame grounding of symbol s")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2203} c: GroundingCertificate. certifies(c, map)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("GroundingMap with valid ProjectionMap")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 48: Multi-Session Coordination — axiomatic identities
        Individual {
            id: "https://uor.foundation/op/GR_8",
            type_: "https://uor.foundation/op/Identity",
            label: "GR_8",
            comment: "Session composition validity: compose(S_A, S_B) is valid at \
                      Q_k iff all pinned-site intersections agree at every tower \
                      level Q_0 through Q_k.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("compose(S_A, S_B) valid at Q_k")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2200} j \u{2264} k: \u{2200} a \u{2208} pinnedSites(S_A, Q_j) \u{2229} pinnedSites(S_B, Q_j): datum(S_A, a, Q_j) = datum(S_B, a, Q_j)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("S_A, S_B: Session at quantum level Q_k (k \u{2265} 0)")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(false)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/ParametricLower")),
                ("https://uor.foundation/op/validKMin", IndividualValue::Int(0)),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GR_9",
            type_: "https://uor.foundation/op/Identity",
            label: "GR_9",
            comment: "ContextLease disjointness: two distinct leases on the same \
                      SharedContext have non-overlapping site sets.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("leasedSites(L_A) \u{2229} leasedSites(L_B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("= \u{2205}")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("L_A, L_B: ContextLease on SharedContext C, L_A \u{2260} L_B")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GR_10",
            type_: "https://uor.foundation/op/Identity",
            label: "GR_10",
            comment: "ExecutionPolicy confluence: different execution policies on \
                      the same pending query set produce the same final resolved \
                      state (Church-Rosser for session resolution).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("finalState(R, P_1, Q)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("= finalState(R, P_2, Q) for any P_1, P_2: ExecutionPolicy")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("SessionResolver R with ExecutionPolicy P, pending query set Q")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 48: Multi-Session Coordination — derivational identities
        Individual {
            id: "https://uor.foundation/op/MC_1",
            type_: "https://uor.foundation/op/Identity",
            label: "MC_1",
            comment: "Lease partition conserves total budget: the sum of \
                      freeRank over all leases equals the SharedContext freeRank.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03a3}\u{1d62} freeRank(leasedSites(L_i))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("= freeRank(C)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("SharedContext C; leaseSet {L_1, \u{2026}, L_k} covering all sites of C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MC_2",
            type_: "https://uor.foundation/op/Identity",
            label: "MC_2",
            comment: "Per-lease binding monotonicity: within a leased sub-domain, \
                      freeRank decreases monotonically (SR_1 restricted to lease).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("freeRank(B_{i+1} |_L)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2264} freeRank(B_i |_L)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("ContextLease L held by Session S; binding step i within S restricted to leasedSites(L)")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MC_3",
            type_: "https://uor.foundation/op/Identity",
            label: "MC_3",
            comment: "General composition freeRank via inclusion-exclusion.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("freeRank(compose(S_A, S_B))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("freeRank(S_A) + freeRank(S_B) \u{2212} |pinnedSites(S_A) \u{2229} pinnedSites(S_B)|")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("S_A, S_B: Session; compose(S_A, S_B) valid (SR_8 satisfied)")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MC_4",
            type_: "https://uor.foundation/op/Identity",
            label: "MC_4",
            comment: "Disjoint-lease composition is additive: the intersection \
                      term vanishes when leases are site-disjoint (SR_9).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("freeRank(compose(S_A, S_B))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("= freeRank(S_A) + freeRank(S_B)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("S_A, S_B on ContextLeases L_A, L_B within SharedContext C; SR_9 holds")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MC_5",
            type_: "https://uor.foundation/op/Identity",
            label: "MC_5",
            comment: "Policy-invariant final binding set: different execution \
                      policies produce identical SiteBinding records.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("finalBindings(R, P_1, Q)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("= finalBindings(R, P_2, Q)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("SessionResolver R; pending query set Q; ExecutionPolicy P_1, P_2")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MC_6",
            type_: "https://uor.foundation/op/Identity",
            label: "MC_6",
            comment: "Full lease coverage implies composed saturation: k sessions \
                      on disjoint covering leases, each locally converged, produce \
                      a GroundedContext via composition.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03c3}(compose(S_1, \u{2026}, S_k))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("= 1 (FullGrounding)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("SharedContext C; leases {L_1, \u{2026}, L_k} pairwise disjoint (SR_9) and fully covering C; each S_i with freeRank = 0 within L_i")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MC_7",
            type_: "https://uor.foundation/op/Identity",
            label: "MC_7",
            comment: "Distributed O(1) resolution: a query against a composed \
                      GroundedContext resolves in zero steps.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("stepCount(q, C*)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("= 0")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("q: RelationQuery; C* = compose(S_1, \u{2026}, S_k) with \u{03c3}(C*) = 1 by MC_6")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MC_8",
            type_: "https://uor.foundation/op/Identity",
            label: "MC_8",
            comment: "Parallelism bound: per-session resolution work is bounded \
                      by lease size, not by total site count n.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("max_i stepCount(S_i to convergence within L_i)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2264} \u{2308}n/k\u{2309}")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("SharedContext C with totalSites = n; uniform partition into k leases")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 53: Witt\u{2013}Carry Formalization \u{2014} WC_ series (12)
        Individual {
            id: "https://uor.foundation/op/WC_1",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_1",
            comment: "Witt coordinate identification: the bit coordinates \
                      (x_0, \u{2026}, x_[n\u{2212}1]) of x \u{2208} \
                      Z/(2\u{207f})Z are exactly its Witt coordinates under \
                      the canonical isomorphism W_n(F_2) \u{2245} Z/(2\u{207f})Z.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("a_k(x)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("x_k (k-th bit of x)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x \u{2208} R_n, 0 \u{2264} k < n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WC_2",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_2",
            comment: "Witt sum correction equals carry: the k-th Witt \
                      addition polynomial correction term S_k \u{2212} x_k \
                      \u{2212} y_k (mod 2) is exactly the carry c_k(x,y).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("S_k \u{2212} x_k \u{2212} y_k (mod 2)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("c_k(x,y)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x, y \u{2208} R_n, 0 \u{2264} k < n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WC_3",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_3",
            comment: "Carry recurrence is the Witt polynomial recurrence: \
                      CA_2 implements the ghost equation for S_[k+1] at p=2.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("CA_2 recurrence")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("S_{k+1} ghost equation at p=2")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x, y \u{2208} R_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WC_4",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_4",
            comment: "The \u{03b4}-correction at level k equals the \
                      single-level carry c_[k+1](x,y). Each application of \
                      \u{03b4} divides by 2, consuming one unit of 2-adic \
                      valuation.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03b4}_k(x+y) correction")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("c_{k+1}(x,y)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x, y \u{2208} R_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WC_5",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_5",
            comment: "LiftObstruction is equivalent to \u{03b4}-nonvanishing: \
                      a nontrivial LiftObstruction at Q_[k+1] means \
                      \u{03b4}_k \u{2260} 0 for some element pair.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("obstruction_trivial = false")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{03b4}_k \u{2260} 0 for some pair")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Q_k, k \u{2265} 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WC_6",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_6",
            comment: "Metric discrepancy equals Witt defect: \
                      d_\u{0394}(x,y) > 0 iff the ghost map correction \
                      (carry) is nonzero.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("d_\u{0394}(x,y) > 0")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("ghost defect nonzero")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x, y \u{2208} R_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Analytical"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WC_7",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_7",
            comment: "D_1 is the Witt truncation order relation: \
                      succ\u{005e}[2\u{207f}](x) = x is the group relation \
                      r\u{005e}[2\u{207f}] = 1 in the Witt-Burnside ring of \
                      D_[2\u{221e}].",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("succ^{2\u{207f}}(x) = x")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("r^{2\u{207f}} = 1 in Witt-Burnside ring")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x \u{2208} R_n, n \u{2265} 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WC_8",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_8",
            comment: "D_3 is the Witt-Burnside conjugation relation: \
                      neg(succ(neg(x))) = pred(x) is srs = \
                      r\u{207b}\u{00b9} in the pro-dihedral group.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("neg(succ(neg(x)))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("srs = r\u{207b}\u{00b9} relation")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x \u{2208} R_n, n \u{2265} 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WC_9",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_9",
            comment: "D_4 is a Witt-Burnside reflection composition: \
                      bnot(neg(x)) = pred(x) is the product of two \
                      reflections yielding inverse rotation.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("bnot(neg(x)) = pred(x)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("Product of Witt-Burnside reflections")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x \u{2208} R_n, n \u{2265} 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WC_10",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_10",
            comment: "The \u{03b4}-ring Frobenius lift on W_n(F_2) is the \
                      identity map because F_2 is a perfect field of \
                      characteristic 2 (a\u{00b2} = a for a \u{2208} F_2).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03c6}(x) on W_n(F_2)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("x (identity, F_2 perfect)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x \u{2208} R_n, n \u{2265} 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WC_11",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_11",
            comment: "The Verschiebung on W_n(F_2) is multiplication by 2: \
                      V(x) = 2x = add(x,x). This is a coordinate shift with \
                      zero Witt defect.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("V(x) on W_n(F_2)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("add(x, x) in Z/(2\u{207f})Z")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x \u{2208} R_n, n \u{2265} 1")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/WC_12",
            type_: "https://uor.foundation/op/Identity",
            label: "WC_12",
            comment: "The \u{03b4}-operator on W_n(F_2) is the squaring \
                      defect divided by 2: \u{03b4}(x) = (x \u{2212} \
                      mul(x,x)) / 2. Expressible entirely in existing op/ \
                      primitives (sub, mul, arithmetic right shift).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03b4}(x) on W_n(F_2)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("(x \u{2212} mul(x,x)) / 2")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x \u{2208} R_n, n \u{2265} 2")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 53: Ostrowski\u{2013}Archimedean bridge \u{2014} OA_ series (5)
        Individual {
            id: "https://uor.foundation/op/OA_1",
            type_: "https://uor.foundation/op/Identity",
            label: "OA_1",
            comment: "Ostrowski product formula at p=2: |2|_2 \u{00b7} \
                      |2|_\u{221e} = 1. The 2-adic and Archimedean absolute \
                      values of 2 are multiplicative inverses.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("|2|_2 \u{00b7} |2|_\u{221e}")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("1 in Q\u{00d7}")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("p = 2")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/ArithmeticValuation"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OA_2",
            type_: "https://uor.foundation/op/Identity",
            label: "OA_2",
            comment: "Crossing cost equals ln 2: the Archimedean image of \
                      one unit of 2-adic valuation, under the product formula, \
                      is ln 2 nats.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("CrossingCost(p=2)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("ln 2 = \u{2212}ln|2|_2")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("p = 2")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/ArithmeticValuation"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OA_3",
            type_: "https://uor.foundation/op/Identity",
            label: "OA_3",
            comment: "QM_1 grounding: the Landauer cost \u{03b2}* = ln 2 is \
                      the crossing cost from OA_2, derived from the prime p=2 \
                      that structures the Witt tower.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03b2}* in Cost_Landauer")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("CrossingCost(p=2)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("p = 2")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/ArithmeticValuation"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OA_4",
            type_: "https://uor.foundation/op/Identity",
            label: "OA_4",
            comment: "Born rule bridge (conditional on amplitude rationality): \
                      P(outcome k) = |\u{03b1}_k|_\u{221e}\u{00b2}, where \
                      |\u{00b7}|_\u{221e} is the Archimedean image of the \
                      2-adic amplitude via the product formula.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("P(outcome k) = |\u{03b1}_k|_\u{221e}\u{00b2}")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("Archimedean image of 2-adic amplitude")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("rational amplitudes")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/ArithmeticValuation"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OA_5",
            type_: "https://uor.foundation/op/Identity",
            label: "OA_5",
            comment: "Entropy per delta-level equals the crossing cost: each \
                      application of the delta-operator (division by 2) costs \
                      ln 2 nats in the Archimedean completion, which is the \
                      per-bit Landauer cost. v0.2.2 Phase C.4: grounds \
                      MultiplicationCertificate.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("Information cost of delta (division by 2); grounds MultiplicationCertificate")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("ln 2 nats per MultiplicationCertificate sub-multiplication")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("p = 2; every MultiplicationCertificate accumulates OA_5 cost")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/ArithmeticValuation"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 54: Homotopy Nerve
        Individual {
            id: "https://uor.foundation/op/HT_1",
            type_: "https://uor.foundation/op/Identity",
            label: "HT_1",
            comment: "KanComplex(N(C)) \u{2014} the constraint nerve satisfies the \
                      Kan extension condition for all horns of dimension \u{2264} d \
                      where d is the maximum simplex dimension of N(C).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("N(C)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("KanComplex")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("constraint configuration C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HT_2",
            type_: "https://uor.foundation/op/Identity",
            label: "HT_2",
            comment: "Path components of nerve recover \u{03b2}\u{2080}: \
                      \u{03c0}\u{2080}(N(C)) \u{2245} Z^{\u{03b2}\u{2080}} \
                      counts the connected components of the constraint \
                      configuration.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03c0}\u{2080}(N(C))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("Z^{\u{03b2}\u{2080}}")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("constraint configuration C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HT_3",
            type_: "https://uor.foundation/op/Identity",
            label: "HT_3",
            comment: "MN_6 monodromy is abelianisation of full \u{03c0}\u{2081}: \
                      the fundamental group \u{03c0}\u{2081}(N(C)) surjects onto \
                      the HolonomyGroup D_{2^n} via abelianisation.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03c0}\u{2081}(N(C)) \u{2192} D_{2^n}")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("HolonomyGroup factorization")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("constraint configuration C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HT_4",
            type_: "https://uor.foundation/op/Identity",
            label: "HT_4",
            comment: "Higher homotopy groups vanish above nerve dimension: \
                      \u{03c0}_k(N(C)) = 0 for all k > dim(N(C)), because the \
                      nerve is a finite CW-complex.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03c0}_k(N(C)) for k > dim(N(C))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("constraint configuration C, k > dim(N(C))")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HT_5",
            type_: "https://uor.foundation/op/Identity",
            label: "HT_5",
            comment: "1-truncation determines flat/twisted classification: \
                      \u{03c4}_{\u{2264}1}(N(C)) captures the holonomy action \
                      that distinguishes FlatType from TwistedType.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03c4}_{\u{2264}1}(N(C))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("FlatType/TwistedType classification")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("constraint configuration C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HT_6",
            type_: "https://uor.foundation/op/Identity",
            label: "HT_6",
            comment: "Trivial k-invariants beyond depth d imply spectral collapse: \
                      if \u{03ba}_k is trivial for all k > d then the spectral \
                      sequence collapses at E_{d+2}.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03ba}_k trivial for all k > d")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("spectral sequence collapses at E_{d+2}")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("constraint configuration C, d = max simplex dim")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HT_7",
            type_: "https://uor.foundation/op/Identity",
            label: "HT_7",
            comment: "Non-trivial Whitehead product implies lift obstruction: \
                      [\u{03b1}, \u{03b2}] \u{2260} 0 in \u{03c0}_{p+q\u{2212}1} \
                      implies a non-trivial LiftObstruction that Betti numbers \
                      alone cannot detect.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("[\u{03b1}, \u{03b2}] \u{2260} 0 in \u{03c0}_{p+q\u{2212}1}")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("LiftObstruction non-trivial")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("\u{03b1} \u{2208} \u{03c0}_p, \u{03b2} \u{2208} \u{03c0}_q")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HT_8",
            type_: "https://uor.foundation/op/Identity",
            label: "HT_8",
            comment: "Hurewicz isomorphism for first non-vanishing group: \
                      \u{03c0}_k(N(C)) \u{2297} Z \u{2245} H_k(N(C); Z) for the \
                      smallest k with \u{03c0}_k \u{2260} 0, linking homotopy \
                      invariants to homology.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03c0}_k(N(C)) \u{2297} Z")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("H_k(N(C); Z) for smallest k with \u{03c0}_k \u{2260} 0")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("constraint configuration C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 55: Homotopy Pipeline stages
        Individual {
            id: "https://uor.foundation/op/psi_7",
            type_: "https://uor.foundation/op/Identity",
            label: "psi_7",
            comment: "\u{03c8}_7: KanComplex \u{2192} PostnikovTower \u{2014} compute \
                      the Postnikov truncations \u{03c4}_{\u{2264}k} for \
                      k = 0, 1, \u{2026}, dim(N(C)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("KanComplex(N(C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("PostnikovTower")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint configuration C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/psi_8",
            type_: "https://uor.foundation/op/Identity",
            label: "psi_8",
            comment: "\u{03c8}_8: PostnikovTower \u{2192} HomotopyGroups \u{2014} extract \
                      the homotopy groups \u{03c0}_k from each truncation stage.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("PostnikovTower(\u{03c4}\u{2264}k)"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("HomotopyGroups(\u{03c0}_k)"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint configuration C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/psi_9",
            type_: "https://uor.foundation/op/Identity",
            label: "psi_9",
            comment: "\u{03c8}_9: HomotopyGroups \u{2192} KInvariants \u{2014} compute \
                      the k-invariants \u{03ba}_k classifying the Postnikov tower.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("HomotopyGroups(\u{03c0}_k)"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("KInvariants(\u{03ba}_k)"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint configuration C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HP_1",
            type_: "https://uor.foundation/op/Identity",
            label: "HP_1",
            comment: "Pipeline composition: nerve construction + Kan promotion = \
                      \u{03c8}_7 \u{2218} \u{03c8}_1.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("\u{03c8}_7 \u{2218} \u{03c8}_1"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("Kan promotion of nerve"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint configuration C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HP_2",
            type_: "https://uor.foundation/op/Identity",
            label: "HP_2",
            comment: "Homotopy extraction agrees with homology on k-skeleton.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("\u{03c8}_8(\u{03c4}\u{2264}k) restricted"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("\u{03c8}_3(C\u{2264}k)"),
                ),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str("constraint configuration C, truncation level k"),
                ),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HP_3",
            type_: "https://uor.foundation/op/Identity",
            label: "HP_3",
            comment: "k-invariant computation detects QLS_4 convergence.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("\u{03c8}_9 detects convergence"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("spectral sequence converges at E_{d+2}"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint configuration C")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HP_4",
            type_: "https://uor.foundation/op/Identity",
            label: "HP_4",
            comment: "Complexity bound for homotopy type computation.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("HomotopyResolver time"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("O(n^{d+1})"),
                ),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str("d = max simplex dimension"),
                ),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Analytical"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        // Amendment 56: Moduli Space identities
        Individual {
            id: "https://uor.foundation/op/MD_1",
            type_: "https://uor.foundation/op/Identity",
            label: "MD_1",
            comment: "Moduli space dimension equals basis size of any contained type.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("dim(M_n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("basisSize(T)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("T in M_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MD_2",
            type_: "https://uor.foundation/op/Identity",
            label: "MD_2",
            comment: "Zeroth deformation cohomology = automorphism group \
                      intersected with dihedral group.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("H^0(Def(T))"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("Aut(T) \u{2229} D_{2^n}"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("CompleteType T")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MD_3",
            type_: "https://uor.foundation/op/Identity",
            label: "MD_3",
            comment: "First deformation cohomology = tangent space to the \
                      moduli space at T.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("H^1(Def(T))"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("T_T(M_n)"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("CompleteType T")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MD_4",
            type_: "https://uor.foundation/op/Identity",
            label: "MD_4",
            comment: "Second deformation cohomology = LiftObstruction space.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("H^2(Def(T))"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("LiftObstruction space"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("CompleteType T")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MD_5",
            type_: "https://uor.foundation/op/Identity",
            label: "MD_5",
            comment: "FlatType stratum has codimension zero in the moduli space.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("FlatType stratum codimension"),
                ),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("M_n at any quantum level")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MD_6",
            type_: "https://uor.foundation/op/Identity",
            label: "MD_6",
            comment: "TwistedType stratum has codimension at least 1.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("TwistedType stratum codimension"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("\u{2265} 1"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("M_n at any quantum level")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MD_7",
            type_: "https://uor.foundation/op/Identity",
            label: "MD_7",
            comment: "VersalDeformation existence is guaranteed when the \
                      obstruction space H\u{00b2} vanishes.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("VersalDeformation existence"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("guaranteed when H^2 = 0"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("CompleteType T")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MD_8",
            type_: "https://uor.foundation/op/Identity",
            label: "MD_8",
            comment: "A deformation family preserves completeness iff \
                      H\u{00b2}(Def(T_t)) = 0 along the entire path.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("familyPreservesCompleteness"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("H^2(Def(T_t)) = 0 along path"),
                ),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str("DeformationFamily {C_t}"),
                ),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MD_9",
            type_: "https://uor.foundation/op/Identity",
            label: "MD_9",
            comment: "The site of a ModuliTowerMap at T has dimension 1 \
                      when the obstruction is trivial.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("site(ModuliTowerMap, T) dimension"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("1 when obstructionTrivial"),
                ),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str("CompleteType T, obstruction = 0"),
                ),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MD_10",
            type_: "https://uor.foundation/op/Identity",
            label: "MD_10",
            comment: "The site of a ModuliTowerMap at T is empty iff T is \
                      a TwistedType at every level.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("site(ModuliTowerMap, T)"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("empty iff TwistedType at every level"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("CompleteType T")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        // Amendment 57: Moduli Resolver identities
        Individual {
            id: "https://uor.foundation/op/MR_1",
            type_: "https://uor.foundation/op/Identity",
            label: "MR_1",
            comment: "ModuliResolver boundary agrees with MorphospaceBoundary.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("ModuliResolver boundary"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("MorphospaceBoundary"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("M_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MR_2",
            type_: "https://uor.foundation/op/Identity",
            label: "MR_2",
            comment: "StratificationRecord covers every CompleteType in exactly \
                      one stratum.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("StratificationRecord coverage"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("every CompleteType in exactly one stratum"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("M_n")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Topological"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MR_3",
            type_: "https://uor.foundation/op/Identity",
            label: "MR_3",
            comment: "ModuliResolver complexity bound.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("ModuliResolver complexity"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("O(n \u{00d7} basisSize\u{00b2})"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("CompleteType T")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Analytical"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MR_4",
            type_: "https://uor.foundation/op/Identity",
            label: "MR_4",
            comment: "Achievable signatures correspond to membership in some \
                      HolonomyStratum.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("achievabilityStatus = Achievable"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("signature in some HolonomyStratum"),
                ),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("MorphospaceRecord")),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        // ── Amendment 58: Carry Algebra identities ──────────────────────
        Individual {
            id: "https://uor.foundation/op/CY_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CY_1",
            comment: "Carry generates at site k iff and(x_k, y_k) = 1. \
                      Extends CA_1 (addition decomposition) and WC_2 \
                      (Witt sum correction).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("generate(k)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("and(x_k, y_k) = 1")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x, y \u{2208} R_n, 0 \u{2264} k < n")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CY_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CY_2",
            comment: "Carry propagates at site k iff xor(x_k, y_k) = 1 and \
                      c_k = 1. Extends CA_2 (carry recurrence) and WC_3 \
                      (Witt polynomial recurrence).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("propagate(k)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("xor(x_k, y_k) = 1 \u{2227} c_k = 1")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x, y \u{2208} R_n, 0 \u{2264} k < n")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CY_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CY_3",
            comment: "Carry kills at site k iff and(x_k, y_k) = 0 and \
                      xor(x_k, y_k) = 0. Complement of CY_1 and CY_2.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("kill(k)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("and(x_k, y_k) = 0 \u{2227} xor(x_k, y_k) = 0")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x, y \u{2208} R_n, 0 \u{2264} k < n")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CY_4",
            type_: "https://uor.foundation/op/Identity",
            label: "CY_4",
            comment: "d_\u{0394}(x,y) = |carryCount(x+y) \u{2212} \
                      hammingDistance(x,y)|. The metric incompatibility IS \
                      the discrepancy between carry count and Hamming distance. \
                      Strengthens WC_6.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("d_\u{0394}(x, y)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("|carryCount(x+y) \u{2212} hammingDistance(x, y)|")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("x, y \u{2208} R_n")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CY_5",
            type_: "https://uor.foundation/op/Identity",
            label: "CY_5",
            comment: "Optimal encoding theorem: the encoding that minimizes \
                      \u{03a3} d_\u{0394} over observed pairs is the one where \
                      the carry chain\u{2019}s significance hierarchy matches the \
                      domain\u{2019}s dependency structure.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("argmin_enc \u{03a3} d_\u{0394}(enc(s_i), enc(s_j))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("enc where carry significance \u{2245} domain dependency")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("finite symbol set S, observed pairs (s_i, s_j)")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CY_6",
            type_: "https://uor.foundation/op/Identity",
            label: "CY_6",
            comment: "Site ordering theorem: d_\u{0394} is minimized when \
                      high-significance sites (upstream in the carry chain) \
                      encode the most structurally informative observables.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("min d_\u{0394} site ordering")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("high-significance sites \u{2192} most informative observables")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("EncodingConfiguration over ordered domain")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CY_7",
            type_: "https://uor.foundation/op/Identity",
            label: "CY_7",
            comment: "Carry lookahead: the carry chain for n sites is \
                      computable in O(log n) using prefix computation on \
                      generate/propagate pairs.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("T(carry_chain(n))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("O(log n) via prefix computation on (g_k, p_k) pairs")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("CarryChain of length n")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid",
                 IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 59: Named Base Metrics identities ─────────────────
        Individual {
            id: "https://uor.foundation/op/BM_1",
            type_: "https://uor.foundation/op/Identity",
            label: "BM_1",
            comment: "\u{03c3}(C) = (n \u{2212} freeRank(C)) / n. The saturation \
                      metric is the complement of free site ratio. Derives from SC_2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03c3}(C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("(n \u{2212} freeRank(C)) / n")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Context C with n sites")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/BM_2",
            type_: "https://uor.foundation/op/Identity",
            label: "BM_2",
            comment: "\u{03c7} = \u{03a3}(\u{2212}1)^k \u{03b2}_k. The Euler \
                      characteristic of the constraint nerve. Derives from IT_2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03c7}(nerve(C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03a3}(\u{2212}1)^k \u{03b2}_k(nerve(C))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint set C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/BM_3",
            type_: "https://uor.foundation/op/Identity",
            label: "BM_3",
            comment: "Index theorem: \u{03a3}\u{03ba}_k \u{2212} \u{03c7} = \
                      S_residual / ln 2. Links all six metrics. Derives from IT_7a.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03a3}\u{03ba}_k \u{2212} \u{03c7}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("S_residual / ln 2")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("computation state")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/BM_4",
            type_: "https://uor.foundation/op/Identity",
            label: "BM_4",
            comment: "J_k = 0 for pinned sites. The Jacobian vanishes on resolved sites.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("J_k (pinned site k)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("pinned site k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/BM_5",
            type_: "https://uor.foundation/op/Identity",
            label: "BM_5",
            comment: "d_\u{0394} > 0 iff carry \u{2260} 0. The metric discrepancy \
                      equals the Witt defect. Derives from WC_6.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_\u{0394}(x, y) > 0")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("carry(x + y) \u{2260} 0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("x, y \u{2208} R_n")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/BM_6",
            type_: "https://uor.foundation/op/Identity",
            label: "BM_6",
            comment: "Metric composition tower: d_\u{0394} \u{2192} {\u{03c3}, J_k} \
                      \u{2192} \u{03b2}_k \u{2192} \u{03c7} \u{2192} r. Each metric \
                      derives from previous ones.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("metric tower")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("d_\u{0394} \u{2192} {\u{03c3}, J_k} \u{2192} \u{03b2}_k \u{2192} \u{03c7} \u{2192} r")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("computation state")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/IndexTheoretic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 60: Galois Connection identities ───────────────────
        Individual {
            id: "https://uor.foundation/op/GL_1",
            type_: "https://uor.foundation/op/Identity",
            label: "GL_1",
            comment: "\u{03c3} = lower adjoint evaluated at current type. The saturation \
                      metric is the lower adjoint of the Galois connection. Derives \
                      from SC_2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03c3}(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("lower_adjoint(T)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("type T in type lattice")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GL_2",
            type_: "https://uor.foundation/op/Identity",
            label: "GL_2",
            comment: "r = complement of upper adjoint image. The residual freedom is \
                      what the type closure does not reach.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("r(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("1 \u{2212} upper_adjoint(T)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("type T in type lattice")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GL_3",
            type_: "https://uor.foundation/op/Identity",
            label: "GL_3",
            comment: "CompleteType = fixpoint of Galois connection, \u{03c3}=1, r=0. \
                      Derives from IT_7d.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("upper(lower(T))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("T (fixpoint: \u{03c3}=1, r=0)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("CompleteType T")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/GL_4",
            type_: "https://uor.foundation/op/Identity",
            label: "GL_4",
            comment: "Type refinement = ascending in type lattice = descending in \
                      site freedom. The Galois connection reverses order.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("T\u{2081} \u{2264} T\u{2082} in type lattice")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("site(T\u{2082}) \u{2286} site(T\u{2081})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("types T\u{2081}, T\u{2082}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 60: Nerve Operations identities ────────────────────
        Individual {
            id: "https://uor.foundation/op/NV_1",
            type_: "https://uor.foundation/op/Identity",
            label: "NV_1",
            comment: "nerve(C\u{2081} \u{222a} C\u{2082}) = nerve(C\u{2081}) \u{222a} \
                      nerve(C\u{2082}) for disjoint constraint domains.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("nerve(C\u{2081} \u{222a} C\u{2082})")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("nerve(C\u{2081}) \u{222a} nerve(C\u{2082})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("disjoint constraint domains C\u{2081}, C\u{2082}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/NV_2",
            type_: "https://uor.foundation/op/Identity",
            label: "NV_2",
            comment: "Mayer\u{2013}Vietoris: \u{03b2}_k(C\u{2081} \u{222a} C\u{2082}) \
                      computable from parts and intersection.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03b2}_k(C\u{2081} \u{222a} C\u{2082})")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03b2}_k(C\u{2081}) + \u{03b2}_k(C\u{2082}) \u{2212} \u{03b2}_k(C\u{2081} \u{2229} C\u{2082})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint sets C\u{2081}, C\u{2082}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/NV_3",
            type_: "https://uor.foundation/op/Identity",
            label: "NV_3",
            comment: "Single constraint addition: \u{0394}\u{03b2}_k \u{2208} \
                      {\u{2212}1, 0, +1} per dimension.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{0394}\u{03b2}_k")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{2208} {\u{2212}1, 0, +1}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("single constraint addition, dimension k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/NV_4",
            type_: "https://uor.foundation/op/Identity",
            label: "NV_4",
            comment: "Constraint accumulation monotonicity: \u{03b2}_k non-increasing \
                      under SR_1. Derives from SR_1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03b2}_k(C \u{222a} {c})")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{2264} \u{03b2}_k(C)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint set C, new constraint c")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 61: Structural Type identities ─────────────────────
        Individual {
            id: "https://uor.foundation/op/SD_1",
            type_: "https://uor.foundation/op/Identity",
            label: "SD_1",
            comment: "ScalarType grounding: quantize(value, range, bits) produces \
                      ring element where d_R reflects value proximity.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("quantize(value, range, bits)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("ring element with d_R \u{221d} |v\u{2081} \u{2212} v\u{2082}|")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("values v in ordered domain")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SD_2",
            type_: "https://uor.foundation/op/Identity",
            label: "SD_2",
            comment: "SymbolType grounding: argmin_{encoding} \u{03a3} d_\u{0394} over \
                      observed pairs (CY_5).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("encoding(alphabet)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("argmin_{e} \u{03a3} d_\u{0394}(e(a), e(b))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("symbol pairs (a,b) in alphabet")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SD_3",
            type_: "https://uor.foundation/op/Identity",
            label: "SD_3",
            comment: "SequenceType = free monoid on element type with backbone \
                      constraint.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Seq(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Free(T) with backbone ordering")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("element type T")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SD_4",
            type_: "https://uor.foundation/op/Identity",
            label: "SD_4",
            comment: "TupleType site count = \u{03a3} field site counts, site \
                      ordering follows CY_6.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("sites(Tuple(f\u{2081},...,f\u{2096}))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03a3}\u{1d62} sites(f\u{1d62})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("fields f_1,...,f_k of tuple type")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SD_5",
            type_: "https://uor.foundation/op/Identity",
            label: "SD_5",
            comment: "GraphType constraint nerve = graph nerve, \u{03b2}_k equality.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("nerve(Graph(V,E))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("constraint_nerve(Graph(V,E))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("graph (V,E) with typed nodes")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SD_6",
            type_: "https://uor.foundation/op/Identity",
            label: "SD_6",
            comment: "SetType d_\u{0394} invariant under element permutation, \
                      D_{2n} symmetry.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_\u{0394}(Set(a,b,c))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("d_\u{0394}(Set(\u{03c3}(a,b,c)))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("permutation \u{03c3} of set elements")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SD_7",
            type_: "https://uor.foundation/op/Identity",
            label: "SD_7",
            comment: "TreeType \u{03b2}_1=0, \u{03b2}_0=1 topological constraints.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03b2}_1(Tree(V,E))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("tree (V,E) with beta_0=1")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SD_8",
            type_: "https://uor.foundation/op/Identity",
            label: "SD_8",
            comment: "TableType = SequenceType(TupleType(S)), functorial decomposition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Table(S)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Seq(Tuple(S))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("schema S defining tuple fields")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 62: Composed Operations identities (18)
        // DD_ — Dispatch determinism (2)
        Individual {
            id: "https://uor.foundation/op/DD_1",
            type_: "https://uor.foundation/op/Identity",
            label: "DD_1",
            comment: "Dispatch determinism: same query and same registry always yield \
                      the same resolver.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03b4}(q, R)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03b4}(q, R)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("query q, registry R")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DD_2",
            type_: "https://uor.foundation/op/Identity",
            label: "DD_2",
            comment: "Dispatch coverage: every query in the registry domain has a \
                      matching resolver.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("dom(R)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("{q | \u{2203}r. \u{03b4}(q,R)=r}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("registry R")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // PI_ — Pipeline inference (5)
        Individual {
            id: "https://uor.foundation/op/PI_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PI_1",
            comment: "Inference idempotence: \u{03b9}(\u{03b9}(s,C),C) = \u{03b9}(s,C) \
                      on GroundedContext.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03b9}(\u{03b9}(s,C),C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03b9}(s,C)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("symbol s, GroundedContext C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PI_2",
            type_: "https://uor.foundation/op/Identity",
            label: "PI_2",
            comment: "Inference soundness: \u{03b9}(s,C) resolves to a type consistent \
                      with C.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("type(\u{03b9}(s,C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("consistent(C)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("symbol s, context C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PI_3",
            type_: "https://uor.foundation/op/Identity",
            label: "PI_3",
            comment: "Inference composition: \u{03b9} = P \u{2218} \u{03a0} \u{2218} G \
                      (references phi_4).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03b9}(s,C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("P(\u{03a0}(G(s,C)))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("symbol s, context C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PI_4",
            type_: "https://uor.foundation/op/Identity",
            label: "PI_4",
            comment: "Inference complexity: O(n) worst case, O(1) on CompleteType.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("complexity(\u{03b9}(s,C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("O(|C|)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("symbol s, context C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PI_5",
            type_: "https://uor.foundation/op/Identity",
            label: "PI_5",
            comment: "Inference coherence: roundTrip(P(\u{03a0}(G(s)))) = s.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("roundTrip(P(\u{03a0}(G(s))))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("s")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("symbol s")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // PA_ — Accumulation algebra (5)
        Individual {
            id: "https://uor.foundation/op/PA_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PA_1",
            comment: "Accumulation permutation invariance: accumulating bindings in \
                      any order yields the same saturated context (derives from SR_10).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03b1}(b\u{2081},\u{03b1}(b\u{2082},C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03b1}(b\u{2082},\u{03b1}(b\u{2081},C))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("bindings b\u{2081},b\u{2082}, context C at saturation")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PA_2",
            type_: "https://uor.foundation/op/Identity",
            label: "PA_2",
            comment: "Accumulation monotonicity: \u{03b1}(b,C) \u{2287} C (the context \
                      only grows, never loses bindings). Derives from SR_1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("sites(\u{03b1}(b,C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("sites(C) \u{222a} {b.site}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("binding b, context C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PA_3",
            type_: "https://uor.foundation/op/Identity",
            label: "PA_3",
            comment: "Accumulation soundness: \u{03b1}(b,C) preserves all previously \
                      satisfied constraints. Derives from SR_2.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("constraints(\u{03b1}(b,C))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("constraints(C) \u{222a} constraints(b)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("binding b, context C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PA_4",
            type_: "https://uor.foundation/op/Identity",
            label: "PA_4",
            comment: "Accumulation base preservation: \u{03b1} does not modify previously \
                      pinned sites.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03b1}(b,C)|_{pinned(C)}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("C|_{pinned(C)}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("binding b, context C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PA_5",
            type_: "https://uor.foundation/op/Identity",
            label: "PA_5",
            comment: "Accumulation identity: \u{03b1}(\u{2205}, C) = C.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03b1}(\u{2205}, C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("C")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("context C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // PL_ — Lease partition (3)
        Individual {
            id: "https://uor.foundation/op/PL_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PL_1",
            comment: "Lease disjointness: partitioned leases have pairwise disjoint \
                      site sets (derives from SR_9).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("L\u{1d62} \u{2229} L\u{2c7c}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{2205}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("leases L\u{1d62}, L\u{2c7c} with i \u{2260} j")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PL_2",
            type_: "https://uor.foundation/op/Identity",
            label: "PL_2",
            comment: "Lease conservation: union of all leases equals the original \
                      shared context (derives from MC_1).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{22c3}\u{1d62} L\u{1d62}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("S")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("shared context S, leases L\u{1d62}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PL_3",
            type_: "https://uor.foundation/op/Identity",
            label: "PL_3",
            comment: "Lease coverage: every site in the shared context appears in \
                      exactly one lease (derives from MC_6).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("|{i | f \u{2208} L\u{1d62}}|")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("site f in shared context S")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // PK_ — Session composition (2)
        Individual {
            id: "https://uor.foundation/op/PK_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PK_1",
            comment: "Composition validity: \u{03ba}(S\u{2081},S\u{2082}) is a valid \
                      session if S\u{2081},S\u{2082} have disjoint leases (derives \
                      from SR_8).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("valid(\u{03ba}(S\u{2081},S\u{2082}))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("disjoint(S\u{2081},S\u{2082})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("sessions S\u{2081},S\u{2082}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PK_2",
            type_: "https://uor.foundation/op/Identity",
            label: "PK_2",
            comment: "Distributed resolution: resolving in \u{03ba}(S\u{2081},S\u{2082}) \
                      equals resolving in S\u{2081} or S\u{2082} independently \
                      (derives from MC_7).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resolve(s, \u{03ba}(S\u{2081},S\u{2082}))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("resolve(s, S\u{2081}) \u{2228} resolve(s, S\u{2082})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("symbol s, sessions S\u{2081},S\u{2082}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // PP_ — Pipeline unification (1)
        Individual {
            id: "https://uor.foundation/op/PP_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PP_1",
            comment: "Pipeline unification: \u{03ba}(\u{03bb}\u{2096}(\u{03b1}*(\
                      \u{03b9}(s,\u{00b7}))),C) = resolve(s,C). The full composed \
                      pipeline equals the top-level resolution function.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03ba}(\u{03bb}\u{2096}(\u{03b1}*(\u{03b9}(s,\u{00b7}))), C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("resolve(s, C)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("symbol s, context C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // Amendment 63: Reduction Core identities (16)
        // PE_ — Pipeline evaluation semantics (7)
        Individual {
            id: "https://uor.foundation/op/PE_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PE_1",
            comment: "Stage 0 initializes state vector to 1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("state(\u{03c8}, 0)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("reduction \u{03c8}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PE_2",
            type_: "https://uor.foundation/op/Identity",
            label: "PE_2",
            comment: "Stage 1 dispatches resolver (\u{03b4} selects).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03c8}_1(q)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03b4}(q, R)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("query q, registry R")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PE_3",
            type_: "https://uor.foundation/op/Identity",
            label: "PE_3",
            comment: "Stage 2 produces valid ring address (G grounds).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03c8}_2(r)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("G(r)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("resolver r")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PE_4",
            type_: "https://uor.foundation/op/Identity",
            label: "PE_4",
            comment: "Stage 3 resolves constraints (\u{03a0} terminates).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03c8}_3(a)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03a0}(a)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("address a")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PE_5",
            type_: "https://uor.foundation/op/Identity",
            label: "PE_5",
            comment: "Stage 4 accumulates without contradiction (\u{03b1} consistent).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03c8}_4(c)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03b1}*(c)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("constraint set c")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PE_6",
            type_: "https://uor.foundation/op/Identity",
            label: "PE_6",
            comment: "Stage 5 extracts coherent output (P projects).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03c8}_5(b)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("P(b)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("binding b")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PE_7",
            type_: "https://uor.foundation/op/Identity",
            label: "PE_7",
            comment: "Full pipeline is the composition PE_6 \u{2218} \u{2026} \u{2218} PE_1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03c8}_5 \u{2218} \u{03c8}_4 \u{2218} \u{03c8}_3 \u{2218} \u{03c8}_2 \u{2218} \u{03c8}_1 \u{2218} \u{03c8}_0")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03c8}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("reduction \u{03c8}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // PM_ — Pipeline machine execution model (7)
        Individual {
            id: "https://uor.foundation/op/PM_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PM_1",
            comment: "Phase rotation \u{03a9}^k at stage k.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("phase(stage_k)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03a9}^k")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("stage index k in 0..5")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PM_2",
            type_: "https://uor.foundation/op/Identity",
            label: "PM_2",
            comment: "Phase gate checks \u{03a9}^k at boundary.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("gate(k)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("phase(k) == \u{03a9}^k")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("stage boundary k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PM_3",
            type_: "https://uor.foundation/op/Identity",
            label: "PM_3",
            comment: "Gate failure triggers complex conjugate rollback.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("fail(gate(k))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("rollback(z \u{2192} z\u{0304})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("stage k with gate failure")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PM_4",
            type_: "https://uor.foundation/op/Identity",
            label: "PM_4",
            comment: "Rollback is involutory: (z\u{0304})\u{0304} = z.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("conj(conj(z))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("z")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("complex value z")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PM_5",
            type_: "https://uor.foundation/op/Identity",
            label: "PM_5",
            comment: "Epoch boundary preserves saturation.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("sat(epoch_n)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("sat(epoch_{n+1})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("consecutive epochs n, n+1")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PM_6",
            type_: "https://uor.foundation/op/Identity",
            label: "PM_6",
            comment: "Service window provides base context.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("context(window)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("base_context")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("service window")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PM_7",
            type_: "https://uor.foundation/op/Identity",
            label: "PM_7",
            comment: "Machine is deterministic given initial state.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03c8}(s\u{2080})")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03c8}(s\u{2080})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("initial state s\u{2080}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ER_ — Execution rules (2)
        Individual {
            id: "https://uor.foundation/op/ER_1",
            type_: "https://uor.foundation/op/Identity",
            label: "ER_1",
            comment: "Stage transition requires guard satisfaction.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("advance(k, k+1)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("guard(k) = true")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("stage transition k to k+1")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ER_2",
            type_: "https://uor.foundation/op/Identity",
            label: "ER_2",
            comment: "Effect application is atomic.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("apply(effect(k))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("atomic(effect(k))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("transition effect at stage k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ER_3–ER_4 — Additional execution rules (Amendment 64)
        Individual {
            id: "https://uor.foundation/op/ER_3",
            type_: "https://uor.foundation/op/Identity",
            label: "ER_3",
            comment: "Guard evaluation is side-effect-free.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("eval(guard(k), s)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("s (state unchanged)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("guard evaluation at stage k with state s")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/ER_4",
            type_: "https://uor.foundation/op/Identity",
            label: "ER_4",
            comment: "Effect composition is order-independent within a stage.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("apply(e1; e2)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("apply(e2; e1)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("effects e1, e2 within same stage")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // EA_ — Epoch admission (4)
        Individual {
            id: "https://uor.foundation/op/EA_1",
            type_: "https://uor.foundation/op/Identity",
            label: "EA_1",
            comment: "Epoch boundary resets free sites.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("free(epoch(n+1))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("free(epoch(0))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("epoch boundary n to n+1")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EA_2",
            type_: "https://uor.foundation/op/Identity",
            label: "EA_2",
            comment: "Grounding carries across epochs.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("grounded(epoch(n))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("grounded(epoch(n+1))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("grounded sites across epoch boundary")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EA_3",
            type_: "https://uor.foundation/op/Identity",
            label: "EA_3",
            comment: "Service window bounds context size.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("|context(w)|")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("windowSize(w)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("service window w")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EA_4",
            type_: "https://uor.foundation/op/Identity",
            label: "EA_4",
            comment: "Epoch admits one datum or one refinement pass.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("admit(epoch(n))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("datum | refinement")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("epoch admission at epoch n")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // OE_ — Optimization equivalences (3)
        Individual {
            id: "https://uor.foundation/op/OE_1",
            type_: "https://uor.foundation/op/Identity",
            label: "OE_1",
            comment: "Adjacent stages with compatible guards may fuse.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("stage(k); stage(k+1)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("fused(k, k+1)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("adjacent stages k, k+1 with compatible guards")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OE_2",
            type_: "https://uor.foundation/op/Identity",
            label: "OE_2",
            comment: "Independent effects commute.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("effect(a); effect(b)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("effect(b); effect(a)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("independent effects a, b")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OE_3",
            type_: "https://uor.foundation/op/Identity",
            label: "OE_3",
            comment: "Disjoint leases parallelize.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("lease(A); lease(B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("lease(A) || lease(B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("disjoint lease sets A, B")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // OE_4a–OE_4c — Sub-lemmas (3)
        Individual {
            id: "https://uor.foundation/op/OE_4a",
            type_: "https://uor.foundation/op/Identity",
            label: "OE_4a",
            comment: "Stage fusion preserves semantics.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("sem(fused(k, k+1))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("sem(stage(k)); sem(stage(k+1))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("fused stages k, k+1")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OE_4b",
            type_: "https://uor.foundation/op/Identity",
            label: "OE_4b",
            comment: "Effect commutation preserves outcome.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("outcome(a; b)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("outcome(b; a)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("commuting effects a, b")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OE_4c",
            type_: "https://uor.foundation/op/Identity",
            label: "OE_4c",
            comment: "Lease parallelism preserves coverage.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("coverage(A || B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("coverage(A) + coverage(B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("parallel leases A, B")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // CS_ — Cost semantics (4)
        Individual {
            id: "https://uor.foundation/op/CS_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CS_1",
            comment: "Each stage has bounded cost.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cost(stage(k))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("O(1)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("reduction step k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CS_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CS_2",
            comment: "Pipeline cost = sum of stage costs.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cost(pipeline)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("sum(cost(stage(k)))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("reduction pipeline")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CS_3",
            type_: "https://uor.foundation/op/Identity",
            label: "CS_3",
            comment: "Rollback cost is at most forward cost.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cost(rollback)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("cost(forward)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("reduction rollback operation")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CS_4",
            type_: "https://uor.foundation/op/Identity",
            label: "CS_4",
            comment: "Preflight cost is O(1).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cost(preflight)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("O(1)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("preflight check")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // CS_5 — Cost semantics (Amendment 65)
        Individual {
            id: "https://uor.foundation/op/CS_5",
            type_: "https://uor.foundation/op/Identity",
            label: "CS_5",
            comment: "Total reduction cost bounded by n \u{00d7} stage_max_cost.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cost(reduction)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("n * max(cost(stage(k)))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("reduction with n stages")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // CS_6 — Budget solvency rejection (Amendment 84)
        Individual {
            id: "https://uor.foundation/op/CS_6",
            type_: "https://uor.foundation/op/Identity",
            label: "CS_6",
            comment: "Budget solvency rejection: a CompileUnit whose declared \
                      thermodynamicBudget is strictly less than the Landauer \
                      minimum (bitsWidth(Q_k) \u{00d7} ln 2) is rejected at \
                      the BudgetSolvencyCheck preflight.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str(
                        "thermodynamicBudget(U) < bitsWidth(unitWittLevel(U)) \u{00d7} ln 2",
                    ),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str("reject(U) at BudgetSolvencyCheck"),
                ),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str("CompileUnit U"),
                ),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Pipeline"),
                ),
                (
                    "https://uor.foundation/op/universallyValid",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        // CS_7 — Unit address computation (Amendment 84)
        Individual {
            id: "https://uor.foundation/op/CS_7",
            type_: "https://uor.foundation/op/Identity",
            label: "CS_7",
            comment: "Unit address identity: the unitAddress of a CompileUnit \
                      is the u:Element computed by hashing the canonical byte \
                      serialization of the root term\u{2019}s transitive closure.",
            properties: &[
                (
                    "https://uor.foundation/op/lhs",
                    IndividualValue::Str("unitAddress(U)"),
                ),
                (
                    "https://uor.foundation/op/rhs",
                    IndividualValue::Str(
                        "address(canonicalBytes(transitiveClosure(rootTerm(U))))",
                    ),
                ),
                (
                    "https://uor.foundation/op/forAll",
                    IndividualValue::Str("CompileUnit U"),
                ),
                (
                    "https://uor.foundation/op/verificationDomain",
                    IndividualValue::IriRef("https://uor.foundation/op/Algebraic"),
                ),
                (
                    "https://uor.foundation/op/universallyValid",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/op/validityKind",
                    IndividualValue::IriRef("https://uor.foundation/op/Universal"),
                ),
            ],
        },
        // FA_ — Scheduler fairness (3, Amendment 65)
        Individual {
            id: "https://uor.foundation/op/FA_1",
            type_: "https://uor.foundation/op/Identity",
            label: "FA_1",
            comment: "Every pending query eventually reaches a stage gate.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pending(q)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("reaches_gate(q)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("query q in reduction")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FA_2",
            type_: "https://uor.foundation/op/Identity",
            label: "FA_2",
            comment: "No starvation under bounded epoch admission.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("admitted(q, epoch)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("served(q, epoch + k)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("query q, bounded k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FA_3",
            type_: "https://uor.foundation/op/Identity",
            label: "FA_3",
            comment: "Fair lease allocation under disjoint composition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("lease_alloc(p1 + p2)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("lease_alloc(p1) + lease_alloc(p2)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("disjoint partitions p1, p2")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // SW_ — Service window (4, Amendment 65)
        Individual {
            id: "https://uor.foundation/op/SW_1",
            type_: "https://uor.foundation/op/Identity",
            label: "SW_1",
            comment: "Service window bounds context memory.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("memory(window(w))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("O(w)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("service window of size w")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SW_2",
            type_: "https://uor.foundation/op/Identity",
            label: "SW_2",
            comment: "Window slide preserves saturation invariant.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("saturated(slide(w))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("saturated(w)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("window slide operation")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SW_3",
            type_: "https://uor.foundation/op/Identity",
            label: "SW_3",
            comment: "Evicted epochs release lease resources.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resources(evict(e))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("evicted epoch e")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SW_4",
            type_: "https://uor.foundation/op/Identity",
            label: "SW_4",
            comment: "Window size \u{2265} 1 (non-empty).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("size(window)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str(">= 1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("service window")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // LS_ — Lease suspension (4, Amendment 65)
        Individual {
            id: "https://uor.foundation/op/LS_1",
            type_: "https://uor.foundation/op/Identity",
            label: "LS_1",
            comment: "Suspended lease preserves pinned state.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("pinned(suspend(lease))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("pinned(lease)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("lease suspension")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/LS_2",
            type_: "https://uor.foundation/op/Identity",
            label: "LS_2",
            comment: "Lease expiry triggers resource release.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resources(expire(lease))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("expired lease")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/LS_3",
            type_: "https://uor.foundation/op/Identity",
            label: "LS_3",
            comment: "Checkpoint restore is idempotent.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("restore(restore(checkpoint))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("restore(checkpoint)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("checkpoint restore")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/LS_4",
            type_: "https://uor.foundation/op/Identity",
            label: "LS_4",
            comment: "Active \u{2192} Suspended \u{2192} Active round-trip preserves state.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("resume(suspend(lease))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("lease")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("active lease")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // TJ_ — Transaction (3, Amendment 65)
        Individual {
            id: "https://uor.foundation/op/TJ_1",
            type_: "https://uor.foundation/op/Identity",
            label: "TJ_1",
            comment: "AllOrNothing transaction rolls back on any failure.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("all_or_nothing(fail)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("rollback")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("AllOrNothing transaction with failure")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TJ_2",
            type_: "https://uor.foundation/op/Identity",
            label: "TJ_2",
            comment: "BestEffort transaction commits partial results.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("best_effort(partial_fail)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("commit(succeeded)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("BestEffort transaction")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/TJ_3",
            type_: "https://uor.foundation/op/Identity",
            label: "TJ_3",
            comment: "Transaction atomicity within a single epoch.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("scope(transaction)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("single_epoch")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("reduction transaction")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // AP_ — Approximation (3, Amendment 65)
        Individual {
            id: "https://uor.foundation/op/AP_1",
            type_: "https://uor.foundation/op/Identity",
            label: "AP_1",
            comment: "Partial saturation is monotonically non-decreasing across stages.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("sat(stage(k+1))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str(">= sat(stage(k))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("consecutive stages k, k+1")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AP_2",
            type_: "https://uor.foundation/op/Identity",
            label: "AP_2",
            comment: "Approximation quality improves with additional epochs.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("quality(epoch(n+1))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str(">= quality(epoch(n))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("consecutive epochs n, n+1")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AP_3",
            type_: "https://uor.foundation/op/Identity",
            label: "AP_3",
            comment: "Deferred queries are eventually processed or explicitly dropped.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("deferred(q)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("processed(q) | dropped(q)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("deferred query q")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // EC_ — Euler convergence tower (8, Amendment 66)
        Individual {
            id: "https://uor.foundation/op/EC_1",
            type_: "https://uor.foundation/op/Identity",
            label: "EC_1",
            comment: "\u{03a9}\u{2076} = \u{2212}1: reduction converges in 6 stages (phase half-turn).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03a9}\u{2076}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{2212}1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("reduction phase angle \u{03a9} = e^{i\u{03c0}/6}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EC_2",
            type_: "https://uor.foundation/op/Identity",
            label: "EC_2",
            comment: "Complex conjugate rollback involutory: (z\u{0304})\u{0304} = z.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("conj(conj(z))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("z")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("complex z in reduction")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EC_3",
            type_: "https://uor.foundation/op/Identity",
            label: "EC_3",
            comment: "Pairwise convergence: commutator converges to identity.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("[q_A, q_B]^inf")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("1")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("quaternion pair q_A, q_B")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EC_4",
            type_: "https://uor.foundation/op/Identity",
            label: "EC_4",
            comment: "Triple convergence: associator converges to zero.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("[q_A, q_B, q_C]^inf")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("octonion triple q_A, q_B, q_C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EC_4a",
            type_: "https://uor.foundation/op/Identity",
            label: "EC_4a",
            comment: "Associator monotonicity: associator norm non-increasing.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("||[a,b,c]_{n+1}||")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("<= ||[a,b,c]_n||")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("successive associator iterates")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EC_4b",
            type_: "https://uor.foundation/op/Identity",
            label: "EC_4b",
            comment: "Associator finiteness: reaches 0 in bounded steps.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("steps_to_zero([a,b,c])")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("<= |three_way_sites|")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("octonion triple a, b, c")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EC_4c",
            type_: "https://uor.foundation/op/Identity",
            label: "EC_4c",
            comment: "Associator vanishing implies associativity on resolved site space.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("[a,b,c] = 0")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("associative(resolved_site_space)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("resolved site space")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/EC_5",
            type_: "https://uor.foundation/op/Identity",
            label: "EC_5",
            comment: "Adams termination: no convergence level beyond L3_Self.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("max_level(convergence_tower)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("L3_Self")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("convergence tower")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 67: Division Algebras (DA_) ───────────────────────
        Individual {
            id: "https://uor.foundation/op/DA_1",
            type_: "https://uor.foundation/op/Identity",
            label: "DA_1",
            comment: "Cayley-Dickson R\u{2192}C: adjoin i with i\u{00b2}=\u{2212}1, \
                      conjugation (a+bi)* = a\u{2212}bi.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("CD(R, i)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("C")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Cayley-Dickson doubling")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DA_2",
            type_: "https://uor.foundation/op/Identity",
            label: "DA_2",
            comment: "Cayley-Dickson C\u{2192}H: adjoin j with j\u{00b2}=\u{2212}1, \
                      ij=k, ji=\u{2212}k, k\u{00b2}=\u{2212}1.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("CD(C, j)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("H")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Cayley-Dickson doubling")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DA_3",
            type_: "https://uor.foundation/op/Identity",
            label: "DA_3",
            comment: "Cayley-Dickson H\u{2192}O: adjoin l, Fano plane products, \
                      associativity fails.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("CD(H, l)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("O")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("Cayley-Dickson doubling")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DA_4",
            type_: "https://uor.foundation/op/Identity",
            label: "DA_4",
            comment: "Adams theorem: no normed division algebra of dimension 16 exists.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("dim(normed_div_alg)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{2208} {1, 2, 4, 8}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("normed division algebras over R")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DA_5",
            type_: "https://uor.foundation/op/Identity",
            label: "DA_5",
            comment: "Convergence level k lives in k-th division algebra: \
                      L0 in R, L1 in C, L2 in H, L3 in O.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("algebra(L_k)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("division_algebra[k]")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("convergence level L_k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DA_6",
            type_: "https://uor.foundation/op/Identity",
            label: "DA_6",
            comment: "Commutator vanishes iff algebra at that level is commutative.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("[a,b] = 0")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("isCommutative(algebra(L_k))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("elements a, b in division algebra at level k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DA_7",
            type_: "https://uor.foundation/op/Identity",
            label: "DA_7",
            comment: "Associator vanishes iff algebra at that level is associative.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("[a,b,c] = 0")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("isAssociative(algebra(L_k))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("elements a, b, c in division algebra at level k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 68: Interaction Algebra (IN_, AS_) ─────────────────
        Individual {
            id: "https://uor.foundation/op/IN_1",
            type_: "https://uor.foundation/op/Identity",
            label: "IN_1",
            comment: "d_\u{0394} as interaction cost between entities.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_\u{0394}(A,B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("interaction_cost(A,B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("entity pairs A, B")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IN_2",
            type_: "https://uor.foundation/op/Identity",
            label: "IN_2",
            comment: "Disjoint leases imply commutator = 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("disjoint_leases(A,B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("commutator(A,B) = 0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("entity pairs with disjoint leases")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IN_3",
            type_: "https://uor.foundation/op/Identity",
            label: "IN_3",
            comment: "Shared sites imply commutator > 0.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("shared_sites(A,B) \u{2260} \u{2205}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("commutator(A,B) > 0")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("entity pairs with shared sites")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IN_4",
            type_: "https://uor.foundation/op/Identity",
            label: "IN_4",
            comment: "SR_8 implies negotiation converges (commutator decreases monotonically).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("SR_8_session(A,B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("commutator(A,B,t+1) \u{2264} commutator(A,B,t)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("entity pairs in session")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IN_5",
            type_: "https://uor.foundation/op/Identity",
            label: "IN_5",
            comment: "Convergent negotiation selects U(1) \u{2282} SU(2).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("converged_negotiation(A,B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("U(1) \u{2282} SU(2)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("converged pairwise interactions")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IN_6",
            type_: "https://uor.foundation/op/Identity",
            label: "IN_6",
            comment: "Outcome space of pairwise negotiation is S\u{00b2}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("outcome_space(pairwise_negotiation)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("S\u{00b2}")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("pairwise negotiations")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IN_7",
            type_: "https://uor.foundation/op/Identity",
            label: "IN_7",
            comment: "Mutual modeling selects H \u{2282} O.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("converged_mutual_model(A,B,C)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("H \u{2282} O")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("converged triple interactions")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IN_8",
            type_: "https://uor.foundation/op/Identity",
            label: "IN_8",
            comment: "Interaction nerve \u{03b2}_k bounds coupling complexity at dimension k.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03b2}_k(interaction_nerve)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("coupling_complexity(k)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("interaction nerve at dimension k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IN_9",
            type_: "https://uor.foundation/op/Identity",
            label: "IN_9",
            comment: "\u{03b2}_2(nerve) \u{00d7} max_disagreement bounds associator norm.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03b2}_2(nerve) \u{00d7} max_disagreement")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("upper_bound(associator_norm)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("interaction nerves with \u{03b2}_2 > 0")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 68: Associator identities (AS_) ────────────────────
        Individual {
            id: "https://uor.foundation/op/AS_1",
            type_: "https://uor.foundation/op/Identity",
            label: "AS_1",
            comment: "\u{03b4}-\u{03b9}-\u{03ba} non-associativity: \u{03b4} reads registry written by \u{03ba}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("(\u{03b4} \u{2218} \u{03b9}) \u{2218} \u{03ba}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03b4} \u{2218} (\u{03b9} \u{2218} \u{03ba})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("triple \u{03b4}, \u{03b9}, \u{03ba} with shared registry")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AS_2",
            type_: "https://uor.foundation/op/Identity",
            label: "AS_2",
            comment: "\u{03b9}-\u{03b1}-\u{03bb} non-associativity: \u{03bb} reads lease state written by \u{03b1}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("(\u{03b9} \u{2218} \u{03b1}) \u{2218} \u{03bb}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03b9} \u{2218} (\u{03b1} \u{2218} \u{03bb})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("triple \u{03b9}, \u{03b1}, \u{03bb} with shared lease")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AS_3",
            type_: "https://uor.foundation/op/Identity",
            label: "AS_3",
            comment: "\u{03bb}-\u{03ba}-\u{03b4} non-associativity: \u{03b4} reads state written by \u{03ba}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("(\u{03bb} \u{2218} \u{03ba}) \u{2218} \u{03b4}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{03bb} \u{2218} (\u{03ba} \u{2218} \u{03b4})")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("triple \u{03bb}, \u{03ba}, \u{03b4} with shared state")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/AS_4",
            type_: "https://uor.foundation/op/Identity",
            label: "AS_4",
            comment: "Root cause: non-associativity from read-write interleaving \
                      through mediating entity.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("associator(A,B,C) \u{2260} 0")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("\u{2203} mediating read-write interleaving")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("triples with non-zero associator")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/ComposedAlgebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 69: Monoidal Composition (MO_) ─────────────────────
        Individual {
            id: "https://uor.foundation/op/MO_1",
            type_: "https://uor.foundation/op/Identity",
            label: "MO_1",
            comment: "Unit law: I \u{2297} A \u{2245} A \u{2245} A \u{2297} I.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("I \u{2297} A")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("A")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("computations A")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MO_2",
            type_: "https://uor.foundation/op/Identity",
            label: "MO_2",
            comment: "Associativity: (A\u{2297}B)\u{2297}C \u{2245} A\u{2297}(B\u{2297}C).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("(A\u{2297}B)\u{2297}C")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("A\u{2297}(B\u{2297}C)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("computations A, B, C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MO_3",
            type_: "https://uor.foundation/op/Identity",
            label: "MO_3",
            comment: "Certificate composition: cert(A\u{2297}B) contains cert(A) and cert(B).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cert(A\u{2297}B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("cert(A) \u{2227} cert(B)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("certified computations A, B")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MO_4",
            type_: "https://uor.foundation/op/Identity",
            label: "MO_4",
            comment: "\u{03c3}(A\u{2297}B) \u{2265} max(\u{03c3}(A), \u{03c3}(B)): \
                      sequential composition does not lose saturation.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("\u{03c3}(A\u{2297}B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("max(\u{03c3}(A), \u{03c3}(B))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("computations A, B")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/MO_5",
            type_: "https://uor.foundation/op/Identity",
            label: "MO_5",
            comment: "r(A\u{2297}B) \u{2264} min(r(A), r(B)): residual can \
                      only shrink under sequential composition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("r(A\u{2297}B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("min(r(A), r(B))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("computations A, B")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 70: Operad Composition (OP_) ──────────────────────────
        Individual {
            id: "https://uor.foundation/op/OP_1",
            type_: "https://uor.foundation/op/Identity",
            label: "OP_1",
            comment: "Site additivity: siteCount(F(G)) = F.sites + \
                      \u{03a3}_i G_i.sites.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("siteCount(F(G))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("F.sites + \u{03a3}_i G_i.sites")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("structural types F, G")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OP_2",
            type_: "https://uor.foundation/op/Identity",
            label: "OP_2",
            comment: "Grounding distributivity: grounding(F(G(x))) = \
                      F.ground(G.ground(x)).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("grounding(F(G(x)))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("F.ground(G.ground(x))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("structural types F, G, element x")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OP_3",
            type_: "https://uor.foundation/op/Identity",
            label: "OP_3",
            comment: "d_\u{0394} decomposition: d_\u{0394}(F(G)) decomposes \
                      into outer + inner d_\u{0394}.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("d_\u{0394}(F(G))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("d_\u{0394}(F) \u{2218} G + F \u{2218} d_\u{0394}(G)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("structural types F, G")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OP_4",
            type_: "https://uor.foundation/op/Identity",
            label: "OP_4",
            comment: "Table(Tuple(fields)): standard tabular data structure \
                      decomposition.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Table(Tuple(fields))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Sequence(Tuple(fields))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("tabular data")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/OP_5",
            type_: "https://uor.foundation/op/Identity",
            label: "OP_5",
            comment: "Tree(leaves): standard hierarchical data structure \
                      (AST, XML, JSON).",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("Tree(Symbol(leaves))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Graph(Symbol(leaves), acyclic)")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("hierarchical data")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 71: Effect Algebra identities (FX_1–FX_7) ─────────
        Individual {
            id: "https://uor.foundation/op/FX_1",
            type_: "https://uor.foundation/op/Identity",
            label: "FX_1",
            comment: "Pinning decrements free count by exactly 1.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("freeRank(postContext(e))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("freeRank(preContext(e)) \u{2212} 1")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("PinningEffect e")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FX_2",
            type_: "https://uor.foundation/op/Identity",
            label: "FX_2",
            comment: "Unbinding increments free count by exactly 1.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("freeRank(postContext(e))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("freeRank(preContext(e)) + 1")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("UnbindingEffect e")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FX_3",
            type_: "https://uor.foundation/op/Identity",
            label: "FX_3",
            comment: "Phase effects preserve site budget.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("freeRank(postContext(e))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("freeRank(preContext(e))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("PhaseEffect e")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FX_4",
            type_: "https://uor.foundation/op/Identity",
            label: "FX_4",
            comment: "Disjoint effects commute.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("apply(A ; B, ctx)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("apply(B ; A, ctx)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Effects A, B with DisjointnessWitness(target(A), target(B))")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FX_5",
            type_: "https://uor.foundation/op/Identity",
            label: "FX_5",
            comment: "Composite free-count delta is additive.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("freeRankDelta(E\u{2081} ; E\u{2082})")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("freeRankDelta(E\u{2081}) + freeRankDelta(E\u{2082})")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("CompositeEffect (E\u{2081} ; E\u{2082})")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FX_6",
            type_: "https://uor.foundation/op/Identity",
            label: "FX_6",
            comment: "Every ReversibleEffect has an inverse \
                      (PinningEffect\u{207b}\u{00b9} = UnbindingEffect \
                      on same site, PhaseEffect\u{207b}\u{00b9} = \
                      conjugate phase).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("apply(e, apply(e\u{207b}\u{00b9}, ctx))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("ctx")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("ReversibleEffect e")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FX_7",
            type_: "https://uor.foundation/op/Identity",
            label: "FX_7",
            comment: "External effects must match their declared shape.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("freeRankDelta(e)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("declared freeRankDelta in EffectShape")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("ExternalEffect e satisfying conformance:EffectShape")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 72: Predicate & Dispatch identities (PR_1–PR_5) ───
        Individual {
            id: "https://uor.foundation/op/PR_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PR_1",
            comment: "Every predicate is total: evaluation terminates for \
                      all inputs.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("eval(p, x)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2208} {true, false}")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Predicate p, input x")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PR_2",
            type_: "https://uor.foundation/op/Identity",
            label: "PR_2",
            comment: "Every predicate is pure: evaluation does not modify \
                      state.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("state(eval(p, x, s))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("s")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Predicate p, input x, state s")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PR_3",
            type_: "https://uor.foundation/op/Identity",
            label: "PR_3",
            comment: "Exhaustive + mutually exclusive dispatch is \
                      deterministic.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("dispatch(D, x)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("exactly one DispatchRule")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("DispatchTable D with isExhaustive=true, isMutuallyExclusive=true")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PR_4",
            type_: "https://uor.foundation/op/Identity",
            label: "PR_4",
            comment: "Match evaluation is deterministic given exhaustive, \
                      ordered arms.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("eval(M)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("armResult(first matching arm)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("MatchExpression M with exhaustive arms")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PR_5",
            type_: "https://uor.foundation/op/Identity",
            label: "PR_5",
            comment: "Stage transition requires typed guard satisfaction.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("advance(k, guardTarget(g))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("requires guardPredicate(g) = true")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("GuardedTransition g at reduction step k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 73: Reduction guard + Resolver dispatch identities ─
        Individual {
            id: "https://uor.foundation/op/CG_1",
            type_: "https://uor.foundation/op/Identity",
            label: "CG_1",
            comment: "Entry guard must be satisfied to enter a stage.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("advance_to(s)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("requires eval(g, currentState) = true")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("ReductionStep s with entryGuard g")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/CG_2",
            type_: "https://uor.foundation/op/Identity",
            label: "CG_2",
            comment: "Exit guard must be satisfied, then the stage effect \
                      is applied.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("advance_from(s)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("requires eval(g, currentState) = true, then apply(e)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("ReductionStep s with exitGuard g and stageEffect e")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DIS_1",
            type_: "https://uor.foundation/op/Identity",
            label: "DIS_1",
            comment: "The root resolver dispatch table is exhaustive and \
                      mutually exclusive over all TypeDefinitions.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("isExhaustive(D) \u{2227} isMutuallyExclusive(D)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("true")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Root DispatchTable D")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/DIS_2",
            type_: "https://uor.foundation/op/Identity",
            label: "DIS_2",
            comment: "Resolver dispatch is deterministic for every type.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("dispatch(D, T)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("exactly one Resolver")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("TypeDefinition T, DispatchTable D")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 74: Parallel Composition identities (PAR_1–PAR_5) ─
        Individual {
            id: "https://uor.foundation/op/PAR_1",
            type_: "https://uor.foundation/op/Identity",
            label: "PAR_1",
            comment: "Disjoint parallel computations commute: A \u{2297} B = \
                      B \u{2297} A when site targets are disjoint.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("apply(A \u{2297} B, ctx)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("apply(B \u{2297} A, ctx)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("ParallelProduct A \u{2225} B with DisjointnessCertificate")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PAR_2",
            type_: "https://uor.foundation/op/Identity",
            label: "PAR_2",
            comment: "Parallel free-count deltas are additive.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("freeRankDelta(A \u{2225} B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("freeRankDelta(A) + freeRankDelta(B)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("ParallelProduct A \u{2225} B")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PAR_3",
            type_: "https://uor.foundation/op/Identity",
            label: "PAR_3",
            comment: "Partitioning is exhaustive: component cardinalities \
                      sum to total site budget.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03a3} |component_i|")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("n")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("SitePartitioning P over n sites")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PAR_4",
            type_: "https://uor.foundation/op/Identity",
            label: "PAR_4",
            comment: "All interleavings of disjoint parallel computations \
                      yield the same final context.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("finalContext(\u{03c3}(A, B))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("finalContext(A \u{2297} B)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("ParallelProduct A \u{2225} B, any interleaving \u{03c3}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/PAR_5",
            type_: "https://uor.foundation/op/Identity",
            label: "PAR_5",
            comment: "Parallel certificate is the conjunction of component \
                      certificates plus disjointness.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("cert(A \u{2225} B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("cert(A) \u{2227} cert(B) \u{2227} DisjointnessCertificate(A, B)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("cert(A \u{2225} B)")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 75: Higher-Order identities (HO_1–HO_4) ──────────
        Individual {
            id: "https://uor.foundation/op/HO_1",
            type_: "https://uor.foundation/op/Identity",
            label: "HO_1",
            comment: "A ComputationDatum\u{2019}s ring value is the content \
                      hash of its certificate.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("value(c)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("contentHash(referencedCertificate(c))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ComputationDatum c")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HO_2",
            type_: "https://uor.foundation/op/Identity",
            label: "HO_2",
            comment: "Application preserves certification.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cert(output(app))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("cert(applicationTarget(app))")),
                ("https://uor.foundation/op/forAll", IndividualValue::Str("ApplicationMorphism app")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HO_3",
            type_: "https://uor.foundation/op/Identity",
            label: "HO_3",
            comment: "Composition certification requires both components \
                      certified and type-compatible.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("cert(f \u{2218} g)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("cert(f) \u{2227} cert(g) \u{2227} range(g) = domain(f)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("TransformComposition f \u{2218} g")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/HO_4",
            type_: "https://uor.foundation/op/Identity",
            label: "HO_4",
            comment: "Fully saturated partial application equals direct \
                      application.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("p")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("ApplicationMorphism(partialBase(p), boundArguments(p))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("PartialApplication p with remainingArity = 0")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 75: Stream identities (STR_1–STR_6) ──────────────
        Individual {
            id: "https://uor.foundation/op/STR_1",
            type_: "https://uor.foundation/op/Identity",
            label: "STR_1",
            comment: "Every epoch terminates: the reduction within each epoch \
                      reaches convergence angle \u{03c0}.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("reduction(e_k) converges to \u{03c0}")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("true")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Epoch e_k in ProductiveStream")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/STR_2",
            type_: "https://uor.foundation/op/Identity",
            label: "STR_2",
            comment: "Grounding preservation across epoch boundaries.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("saturation(continuationContext(b))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("saturation(postContext(e_k))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("EpochBoundary b between e_k and e_{k+1}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/STR_3",
            type_: "https://uor.foundation/op/Identity",
            label: "STR_3",
            comment: "Every finite prefix computes in finite time.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("computationTime(P)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{03a3}_{i=0}^{k\u{2212}1} computationTime(epoch_i)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("StreamPrefix P of length k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/STR_4",
            type_: "https://uor.foundation/op/Identity",
            label: "STR_4",
            comment: "The first epoch starts from the unfold seed context.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("epoch_0.context")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("seed")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Unfold(seed, step)")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/STR_5",
            type_: "https://uor.foundation/op/Identity",
            label: "STR_5",
            comment: "Each subsequent epoch starts from the previous \
                      boundary\u{2019}s continuation context.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("epoch_{k+1}.context")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("continuationContext(boundary(e_k))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Unfold(seed, step), epoch e_k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/STR_6",
            type_: "https://uor.foundation/op/Identity",
            label: "STR_6",
            comment: "Lease expiry at an epoch boundary returns claimed \
                      sites to the next epoch\u{2019}s linear budget.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("linearBudget(epoch_{k+1})")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("linearBudget(epoch_k) + leaseCardinality(L)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("EpochBoundary b with LeaseAllocation L expiring at b")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 76: Failure Algebra identities (FLR_1–FLR_6) ──────
        Individual {
            id: "https://uor.foundation/op/FLR_1",
            type_: "https://uor.foundation/op/Identity",
            label: "FLR_1",
            comment: "Every partial computation produces exactly one of \
                      Success or Failure.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("result(P)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2208} {Success, Failure}")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("PartialComputation P")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FLR_2",
            type_: "https://uor.foundation/op/Identity",
            label: "FLR_2",
            comment: "A total computation always succeeds.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("result(T)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Success")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("TotalComputation T")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FLR_3",
            type_: "https://uor.foundation/op/Identity",
            label: "FLR_3",
            comment: "Sequential failure propagation: if A fails, B is not \
                      evaluated.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("result(A \u{2297} B)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("Failure(A)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A \u{2297} B where result(A) = Failure")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FLR_4",
            type_: "https://uor.foundation/op/Identity",
            label: "FLR_4",
            comment: "Parallel failure independence: one component\u{2019}s \
                      failure does not prevent the other\u{2019}s success.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("result(A \u{2225} B)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("Failure(A) (left component)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("A \u{2225} B where result(A) = Failure, result(B) = Success")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FLR_5",
            type_: "https://uor.foundation/op/Identity",
            label: "FLR_5",
            comment: "Recovery produces a new ComputationResult.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("result(apply(recoveryEffect(r), failureState(f)))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("ComputationResult")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Recovery r on Failure f")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/FLR_6",
            type_: "https://uor.foundation/op/Identity",
            label: "FLR_6",
            comment: "The reduction\u{2019}s existing rollback mechanism is a \
                      Recovery whose effect is the conjugate phase rotation.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("recoveryEffect(rollback(f))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("PhaseEffect(conjugate)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("ComplexConjugateRollback on Failure f")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 77: Linear Resource identities (LN_1–LN_6) ────────
        Individual {
            id: "https://uor.foundation/op/LN_1",
            type_: "https://uor.foundation/op/Identity",
            label: "LN_1",
            comment: "In a linear trace, every site is targeted exactly \
                      once. Total effect count equals site budget.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03a3} targetCount(site_i)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("n")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("LinearTrace T over n-bit type")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/LN_2",
            type_: "https://uor.foundation/op/Identity",
            label: "LN_2",
            comment: "After a LinearEffect, the target site is pinned.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("status(f, postContext(e))")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("pinned")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("LinearEffect e on site f")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/LN_3",
            type_: "https://uor.foundation/op/Identity",
            label: "LN_3",
            comment: "A consumed LinearSite cannot be targeted again.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("target(e\u{2032}) = f")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("forbidden")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("LinearEffect e on site f, any subsequent effect e\u{2032}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/LN_4",
            type_: "https://uor.foundation/op/Identity",
            label: "LN_4",
            comment: "Lease allocation decrements the linear budget by the \
                      lease cardinality.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("remainingCount(budget after L)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("remainingCount(budget before L) \u{2212} k")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("LeaseAllocation L with leaseCardinality k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/LN_5",
            type_: "https://uor.foundation/op/Identity",
            label: "LN_5",
            comment: "Lease expiry returns claimed sites to the budget.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("remainingCount(budget after expiry)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("remainingCount(budget before expiry) + leaseCardinality(L)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Lease expiry on LeaseAllocation L")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/LN_6",
            type_: "https://uor.foundation/op/Identity",
            label: "LN_6",
            comment: "Every geodesic trace is a linear trace.",
            properties: &[
                ("https://uor.foundation/op/lhs", IndividualValue::Str("G")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("LinearTrace")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("GeodesicTrace G")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 77: Subtyping identities (SB_1–SB_6) ─────────────
        Individual {
            id: "https://uor.foundation/op/SB_1",
            type_: "https://uor.foundation/op/Identity",
            label: "SB_1",
            comment: "Subtyping is constraint superset: more constraints = \
                      more specific.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("constraints(T\u{2081})")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2287} constraints(T\u{2082})")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("TypeInclusion T\u{2081} \u{2264} T\u{2082}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SB_2",
            type_: "https://uor.foundation/op/Identity",
            label: "SB_2",
            comment: "Subtype has fewer valid resolutions.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("resolutions(T\u{2081})")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2286} resolutions(T\u{2082})")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("TypeInclusion T\u{2081} \u{2264} T\u{2082}, resolution R")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SB_3",
            type_: "https://uor.foundation/op/Identity",
            label: "SB_3",
            comment: "The constraint nerve of the supertype is a sub-complex \
                      of the subtype\u{2019}s nerve.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("N(C(T\u{2082}))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("sub-complex of N(C(T\u{2081}))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("TypeInclusion T\u{2081} \u{2264} T\u{2082}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SB_4",
            type_: "https://uor.foundation/op/Identity",
            label: "SB_4",
            comment: "Covariance preserves inclusion.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("F(T\u{2081})")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2264} F(T\u{2082})")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Covariant position F(_), T\u{2081} \u{2264} T\u{2082}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SB_5",
            type_: "https://uor.foundation/op/Identity",
            label: "SB_5",
            comment: "Contravariance reverses inclusion.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("F(T\u{2082})")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2264} F(T\u{2081})")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Contravariant position F(_), T\u{2081} \u{2264} T\u{2082}")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/SB_6",
            type_: "https://uor.foundation/op/Identity",
            label: "SB_6",
            comment: "Lattice depth equals site budget.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("latticeDepth")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("n")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("SubtypingLattice at quantum level n")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 78: Bounded Recursion identities (BR_1–BR_5) ──────
        Individual {
            id: "https://uor.foundation/op/BR_1",
            type_: "https://uor.foundation/op/Identity",
            label: "BR_1",
            comment: "Every recursive step strictly decreases the descent \
                      measure.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("measureValue(stepMeasurePost(s))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("< measureValue(stepMeasurePre(s))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("RecursiveStep s")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/BR_2",
            type_: "https://uor.foundation/op/Identity",
            label: "BR_2",
            comment: "Recursion depth is bounded by the initial measure value.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("depth(RecursionTrace(R))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2264} m")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("BoundedRecursion R with initialMeasure m")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/BR_3",
            type_: "https://uor.foundation/op/Identity",
            label: "BR_3",
            comment: "Every bounded recursion terminates.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("terminates(R)")),
                ("https://uor.foundation/op/rhs", IndividualValue::Str("true")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("BoundedRecursion R")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/BR_4",
            type_: "https://uor.foundation/op/Identity",
            label: "BR_4",
            comment: "Structural recursion\u{2019}s measure is the input \
                      type\u{2019}s structural size.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("initialMeasure(R)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("structuralSize(T)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("StructuralRecursion R on type T")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/BR_5",
            type_: "https://uor.foundation/op/Identity",
            label: "BR_5",
            comment: "The base predicate is satisfied exactly when the \
                      measure reaches zero.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("eval(p, state) = true")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("measureValue = 0")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("BoundedRecursion R with basePredicate p")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 79: Address Region identities (RG_1–RG_4) ─────────
        Individual {
            id: "https://uor.foundation/op/RG_1",
            type_: "https://uor.foundation/op/Identity",
            label: "RG_1",
            comment: "The working set is determined by the constraint nerve \
                      and the stage\u{2019}s site targets.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("workingSetRegions(W)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("computable from N(C(T)) and stage k site targets")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("WorkingSet W for type T at stage k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Topological")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/RG_2",
            type_: "https://uor.foundation/op/Identity",
            label: "RG_2",
            comment: "All addresses within a region are within the \
                      region\u{2019}s diameter under the chosen metric.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{2200} a, b \u{2208} R: d_R(a, b)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2264} diameter(R)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("AddressRegion R with LocalityMetric d_R")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/RG_3",
            type_: "https://uor.foundation/op/Identity",
            label: "RG_3",
            comment: "Total working set size is bounded by the addressable \
                      space at the quantum level.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("\u{03a3} workingSetSize(stage_k)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2264} totalAddressableSpace(quantumLevel)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("RegionAllocation A for computation C")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/RG_4",
            type_: "https://uor.foundation/op/Identity",
            label: "RG_4",
            comment: "The resolver at stage k accesses only addresses \
                      within its working set.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("addresses accessed by resolver at stage k")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("\u{2286} addresses(W_k)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Reduction step k with WorkingSet W_k")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // ── Amendment 81: IO Boundary identities (IO_1–IO_5) ────────────
        Individual {
            id: "https://uor.foundation/op/IO_1",
            type_: "https://uor.foundation/op/Identity",
            label: "IO_1",
            comment: "Ingested data conforms to the source\u{2019}s declared type.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("type(resultDatum(e))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("conformsTo(sourceType(s))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("IngestEffect e from Source s")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IO_2",
            type_: "https://uor.foundation/op/Identity",
            label: "IO_2",
            comment: "Emitted data conforms to the sink\u{2019}s declared type.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("type(emittedDatum(e))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("conformsTo(sinkType(s))")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("EmitEffect e to Sink s")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IO_3",
            type_: "https://uor.foundation/op/Identity",
            label: "IO_3",
            comment: "Every ingestion through a source produces a valid \
                      ring datum via grounding.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("apply(g, ingest(s))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("Datum in R_n")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Source s with GroundingMap g")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IO_4",
            type_: "https://uor.foundation/op/Identity",
            label: "IO_4",
            comment: "Every emission through a sink produces a valid \
                      surface symbol via projection.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("apply(p, d)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("surface symbol conforming to sinkType(s)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("Sink s with ProjectionMap p, Datum d")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Pipeline")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IO_5",
            type_: "https://uor.foundation/op/Identity",
            label: "IO_5",
            comment: "Every boundary effect touches at least one site.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("effect:effectTarget(e)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("non-empty EffectTarget")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("BoundaryEffect e")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        // v0.2.1: Inhabitance Verdict identities (IH_1, IH_2a, IH_2b, IH_3)
        Individual {
            id: "https://uor.foundation/op/IH_1",
            type_: "https://uor.foundation/op/Identity",
            label: "IH_1",
            comment: "Inhabitance soundness (bidirectional). For any \
                      type:ConstrainedType T, cert:InhabitanceCertificate(T) \
                      .verified is true iff the carrier of T is non-empty. \
                      Discharge: the verdict reified by cert:InhabitanceCertificate \
                      agrees with the denotational carrier-non-emptiness \
                      predicate, by direct unfolding of cert:verified \
                      against the predicate semantics.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("InhabitanceCertificate(T).verified")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("carrier(T) \u{2260} \u{2205}")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("T : type:ConstrainedType")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IH_2a",
            type_: "https://uor.foundation/op/Identity",
            label: "IH_2a",
            comment: "Inhabitance cost (2-SAT restriction). For instances \
                      T satisfying predicate:Is2SatShape, the cost of \
                      resolver:TwoSatDecider on T is inherited from the \
                      classical 2-SAT decision procedure: O(n+m) where n is \
                      the variable count and m the clause count. Unrestricted \
                      IH_2 would be equivalent to P = NP and is not adopted.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("cost(TwoSatDecider, T)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("O(n + m)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("T : type:ConstrainedType | Is2SatShape(T)")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IH_2b",
            type_: "https://uor.foundation/op/Identity",
            label: "IH_2b",
            comment: "Inhabitance cost (Horn-SAT restriction). For instances \
                      T satisfying predicate:IsHornShape, the cost of \
                      resolver:HornSatDecider on T is inherited from the \
                      classical Horn-SAT decision procedure via unit \
                      propagation: O(n+m).",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("cost(HornSatDecider, T)")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("O(n + m)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("T : type:ConstrainedType | IsHornShape(T)")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Analytical")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
        Individual {
            id: "https://uor.foundation/op/IH_3",
            type_: "https://uor.foundation/op/Identity",
            label: "IH_3",
            comment: "Carrier preservation under basis reduction. The \
                      constraint-nerve reduction performed during inhabitance \
                      search preserves the carrier of the input \
                      ConstrainedType. Discharged against \
                      resolver:JacobianGuidedResolver and \
                      resolver:guidingJacobian vocabulary.",
            properties: &[
                ("https://uor.foundation/op/lhs",
                 IndividualValue::Str("carrier(reduce(T))")),
                ("https://uor.foundation/op/rhs",
                 IndividualValue::Str("carrier(T)")),
                ("https://uor.foundation/op/forAll",
                 IndividualValue::Str("T : type:ConstrainedType")),
                ("https://uor.foundation/op/verificationDomain",
                 IndividualValue::IriRef("https://uor.foundation/op/Algebraic")),
                ("https://uor.foundation/op/universallyValid", IndividualValue::Bool(true)),
                ("https://uor.foundation/op/validityKind",
                 IndividualValue::IriRef("https://uor.foundation/op/Universal")),
            ],
        },
    ]
}
