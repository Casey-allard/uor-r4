//! `schema/` namespace — Ring substrate, term language, and core value types.
//!
//! The `schema/` namespace defines the fundamental algebraic substrate of the
//! UOR Framework: the ring Z/(2^n)Z (`Datum`), its term language (`Term`,
//! `Literal`, `Application`), and the ring container itself (`Ring`).
//!
//! **Key invariant:** `Term` and `Datum` are `owl:disjointWith` — syntax and
//! semantics are strictly separated. A `Literal` *denotes* a `Datum` via
//! `schema:denotes` without *being* one.
//!
//! Amendment 26 adds `W16Ring` — the concrete ring Z/(2^16)Z at Witt level 16 —
//! with properties `W16bitWidth` (= 16) and `W16capacity` (= 65,536).
//!
//! **Space classification:** `kernel` — compiled into ROM.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `schema/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "schema",
            iri: NS_SCHEMA,
            label: "UOR Schema",
            comment: "Core value types and term language for the UOR ring substrate. \
                      Defines Datum (ring element), Term (syntactic expression), and \
                      the Ring container.",
            space: Space::Kernel,
            imports: &[NS_U],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/schema/Datum",
            label: "Datum",
            comment: "An element of the ring Z/(2^n)Z at a specific Witt level n. \
                      The primary semantic value type. Disjoint from Term: datums are \
                      values, terms are syntactic expressions that evaluate to datums.",
            subclass_of: &[OWL_THING],
            disjoint_with: &["https://uor.foundation/schema/Term"],
        },
        Class {
            id: "https://uor.foundation/schema/Term",
            label: "Term",
            comment: "A syntactic expression in the UOR term language. Terms are \
                      evaluated to produce Datums. Disjoint from Datum.",
            subclass_of: &[OWL_THING],
            disjoint_with: &["https://uor.foundation/schema/Datum"],
        },
        Class {
            id: "https://uor.foundation/schema/Triad",
            label: "Triad",
            comment: "A three-component structure encoding an element's position in \
                      the UOR address space: stratum (ring layer), spectrum (bit \
                      pattern), and address (content-addressable position in the \
                      Braille glyph encoding). The three required functional \
                      properties schema:triadStratum, schema:triadSpectrum, and \
                      schema:triadAddress project a Triad onto its TwoAdicValuation, \
                      WalshHadamardImage, and Address coordinates respectively.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/Literal",
            label: "Literal",
            comment: "A term that directly denotes a datum value. A Literal is a \
                      leaf node in the term language — it refers to a concrete Datum \
                      via schema:denotes without being a Datum itself.",
            subclass_of: &[
                "https://uor.foundation/schema/Term",
                "https://uor.foundation/schema/SurfaceSymbol",
            ],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/Application",
            label: "Application",
            comment: "A term formed by applying an operation to one or more argument \
                      terms. The application's value is the result of evaluating the \
                      operator on the evaluated arguments.",
            subclass_of: &["https://uor.foundation/schema/Term"],
            disjoint_with: &[],
        },
        // Amendment 2: Ring class
        Class {
            id: "https://uor.foundation/schema/Ring",
            label: "Ring",
            comment: "The ambient ring Z/(2^n)Z at a specific Witt level n. \
                      The Ring is the primary data structure of the UOR kernel. \
                      Its two generators (negation and complement) produce the \
                      dihedral group D_{2^n} that governs the invariance frame.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 26: W16Ring — the concrete ring at Witt level 16
        Class {
            id: "https://uor.foundation/schema/W16Ring",
            label: "W16Ring",
            comment: "The concrete ring Z/(2^16)Z at Witt level 16. Subclass of \
                      schema:Ring. Carries 65,536 elements. W16Ring is the first \
                      extension of the default Q0 ring and is the target of Amendment \
                      26's universality proofs.",
            subclass_of: &["https://uor.foundation/schema/Ring"],
            disjoint_with: &[],
        },
        // v3.2: WittLevel class for Q-n generalization
        Class {
            id: "https://uor.foundation/schema/WittLevel",
            label: "WittLevel",
            comment: "A named Witt level Q_k at which the UOR ring operates. \
                      Level Q_k uses 8*(k+1) bits, 2^(8*(k+1)) states, and modulus \
                      2^(8*(k+1)). The named individuals Q0-Q3 are the spec-defined \
                      reference levels. The class is open: Prism implementations \
                      operating at higher levels declare their own WittLevel \
                      individuals. The nextWittLevel property forms an unbounded chain \
                      Q0 -> Q1 -> Q2 -> ...",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 89: AST classes for machine-parsable identity formalization
        Class {
            id: "https://uor.foundation/schema/TermExpression",
            label: "TermExpression",
            comment: "Root AST node for parsed EBNF term expressions. Identity \
                      lhs/rhs values are instances of TermExpression subtypes. \
                      Maps to the `term` production in the EBNF grammar.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/LiteralExpression",
            label: "LiteralExpression",
            comment: "A leaf AST node: an integer literal, variable reference, \
                      or named constant.",
            subclass_of: &["https://uor.foundation/schema/TermExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/ApplicationExpression",
            label: "ApplicationExpression",
            comment: "An AST node representing operator application: an operator \
                      applied to an argument list (e.g., add(x, y)).",
            subclass_of: &["https://uor.foundation/schema/TermExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/InfixExpression",
            label: "InfixExpression",
            comment: "An AST node for infix relations and logical connectives \
                      (e.g., x <= y, P -> Q, a = b).",
            subclass_of: &["https://uor.foundation/schema/TermExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/SetExpression",
            label: "SetExpression",
            comment: "An AST node for set-builder notation (e.g., {x : P(x)}).",
            subclass_of: &["https://uor.foundation/schema/TermExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/CompositionExpression",
            label: "CompositionExpression",
            comment: "An AST node for function composition (f compose g).",
            subclass_of: &["https://uor.foundation/schema/TermExpression"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/ForAllDeclaration",
            label: "ForAllDeclaration",
            comment: "A structured quantifier binding: typed variable declarations \
                      with a domain and quantifier kind (universal or existential). \
                      Replaces the string-valued op:forAll property.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/VariableBinding",
            label: "VariableBinding",
            comment: "A single variable binding: a variable name bound to a domain \
                      type (e.g., x in R_n).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/QuantifierKind",
            label: "QuantifierKind",
            comment: "The kind of quantifier: Universal (forall) or Existential \
                      (exists). Controlled vocabulary with exactly 2 individuals.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 95: Host-value sort (Workstream 5)
        Class {
            id: "https://uor.foundation/schema/SurfaceSymbol",
            label: "SurfaceSymbol",
            comment: "An abstract leaf value that a grounding map can accept as \
                      surface input. Has no direct instances: every SurfaceSymbol \
                      is either a Datum-denoting schema:Literal or an xsd-typed \
                      schema:HostValue, and the two cases are disjoint.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/HostValue",
            label: "HostValue",
            comment: "An xsd-typed value that denotes a host datatype rather than \
                      a ring datum. Used in property-position slots whose range is \
                      xsd and as the host-side input of a grounding map.",
            subclass_of: &["https://uor.foundation/schema/SurfaceSymbol"],
            disjoint_with: &[
                "https://uor.foundation/schema/Term",
                "https://uor.foundation/schema/Datum",
            ],
        },
        Class {
            id: "https://uor.foundation/schema/HostStringLiteral",
            label: "HostStringLiteral",
            comment: "A host string literal carrying an xsd:string value.",
            subclass_of: &["https://uor.foundation/schema/HostValue"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/schema/HostBooleanLiteral",
            label: "HostBooleanLiteral",
            comment: "A host boolean literal carrying an xsd:boolean value.",
            subclass_of: &["https://uor.foundation/schema/HostValue"],
            disjoint_with: &[],
        },
        // v0.2.1: Inhabitance witness carrier
        Class {
            id: "https://uor.foundation/schema/ValueTuple",
            label: "ValueTuple",
            comment: "An ordered tuple of values drawn from a type:ConstrainedType's \
                      carrier. Serves as the witness form for cert:InhabitanceCertificate \
                      when verified is true.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/schema/value",
            label: "value",
            comment: "The integer value of a datum element. For a Datum in Z/(2^n)Z, \
                      this is an integer in [0, 2^n).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Datum"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/schema/wittLength",
            label: "wittLength",
            comment: "The Witt level n of a datum, where the datum's ring is \
                      Z/(2^n)Z. Determines the bit width and modulus of the datum.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Datum"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/schema/stratum",
            label: "stratum",
            comment: "The ring-layer index of a datum, indicating its position in \
                      the stratification of Z/(2^n)Z.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Datum"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/schema/spectrum",
            label: "spectrum",
            comment: "The bit-pattern representation of a datum, encoding its \
                      position in the hypercube geometry of Z/(2^n)Z.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Datum"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/schema/element",
            label: "element",
            comment: "The content-addressable element associated with this datum, \
                      linking the algebraic value to its identifier.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Datum"),
            range: "https://uor.foundation/u/Element",
        },
        Property {
            id: "https://uor.foundation/schema/operator",
            label: "operator",
            comment: "The operation applied in an Application term.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Application"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/schema/argument",
            label: "argument",
            comment: "An argument term in an Application. The ordering of arguments \
                      follows rdf:List semantics.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/schema/Application"),
            range: "https://uor.foundation/schema/Term",
        },
        // Amendment 2: Ring properties
        Property {
            id: "https://uor.foundation/schema/ringWittLength",
            label: "ringWittLength",
            comment: "The bit width n of the ring Z/(2^n)Z. Distinct from \
                      schema:wittLength on Datum — ringWittLength is the container's \
                      bit width; datum wittLength is a membership property.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Ring"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/schema/modulus",
            label: "modulus",
            comment: "The modulus 2^n of the ring. Equals 2 raised to the power \
                      of ringWittLength.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Ring"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/schema/generator",
            label: "generator",
            comment: "The generator element π₁ (value = 1) of the ring. Under \
                      iterated successor application, π₁ generates all ring elements.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Ring"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/schema/negation",
            label: "negation",
            comment: "The ring reflection involution: neg(x) = (-x) mod 2^n. \
                      One of the two generators of the dihedral group D_{2^n}.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Ring"),
            range: "https://uor.foundation/op/Involution",
        },
        Property {
            id: "https://uor.foundation/schema/complement",
            label: "complement",
            comment: "The hypercube reflection involution: bnot(x) = (2^n - 1) ⊕ x. \
                      The second generator of the dihedral group D_{2^n}.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Ring"),
            range: "https://uor.foundation/op/Involution",
        },
        Property {
            id: "https://uor.foundation/schema/denotes",
            label: "denotes",
            comment: "The datum value that a Literal term denotes. Bridges the \
                      Term/Datum disjointness: a Literal refers to a Datum without \
                      being one. Evaluation of a Literal produces its denoted Datum.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Literal"),
            range: "https://uor.foundation/schema/Datum",
        },
        // v3.2: WittLevel properties
        Property {
            id: "https://uor.foundation/schema/bitsWidth",
            label: "bitsWidth",
            comment: "The bit width 8*(k+1) of this Witt level.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/WittLevel"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/schema/cycleSize",
            label: "cycleSize",
            comment: "The number of distinct states 2^(8*(k+1)) at this Witt level.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/WittLevel"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/schema/nextWittLevel",
            label: "nextWittLevel",
            comment: "The next Witt level in the chain: Q_k -> Q_(k+1). The chain \
                      is unbounded; Q3 points to Q4, which is not a named individual \
                      in the spec but may be declared by Prism implementations.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/WittLevel"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        // Amendment 37: Quantum level chain successor (Gap 5)
        Property {
            id: "https://uor.foundation/schema/wittLevelPredecessor",
            label: "wittLevelPredecessor",
            comment: "The predecessor Witt level in the chain: Q_(k+1) -> Q_k. \
                      Inverse of nextWittLevel. If nextWittLevel(Q_k) = Q_(k+1), then \
                      wittLevelPredecessor(Q_(k+1)) = Q_k. Formalizes the chain extension \
                      protocol (QL_8).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/WittLevel"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/schema/atWittLevel",
            label: "atWittLevel",
            comment: "The Witt level at which this Ring instance operates. Links a \
                      concrete Ring individual to its WittLevel.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Ring"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        // Amendment 26: W16Ring properties
        Property {
            id: "https://uor.foundation/schema/W16bitWidth",
            label: "W16bitWidth",
            comment: "Bit width of the Q1 ring: 16.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/W16Ring"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/schema/W16capacity",
            label: "W16capacity",
            comment: "Carrier set size of the Q1 ring: 65,536 elements.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/W16Ring"),
            range: XSD_POSITIVE_INTEGER,
        },
        // Amendment 89: AST properties for identity formalization
        Property {
            id: "https://uor.foundation/schema/boundVariables",
            label: "boundVariables",
            comment: "The variable bindings in a quantifier declaration. \
                      Non-functional: a ForAllDeclaration may bind multiple variables.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/schema/ForAllDeclaration"),
            range: "https://uor.foundation/schema/VariableBinding",
        },
        Property {
            id: "https://uor.foundation/schema/variableDomain",
            label: "variableDomain",
            comment: "The domain type of a variable binding (e.g., schema:Ring, \
                      type:ConstrainedType).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/VariableBinding"),
            range: OWL_CLASS,
        },
        Property {
            id: "https://uor.foundation/schema/variableName",
            label: "variableName",
            comment: "The name of a bound variable (e.g., 'x', 'y', 'n').",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/VariableBinding"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/schema/quantifierKind",
            label: "quantifierKind",
            comment: "The kind of quantifier: Universal or Existential.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/ForAllDeclaration"),
            range: "https://uor.foundation/schema/QuantifierKind",
        },
        Property {
            id: "https://uor.foundation/schema/expressionOperator",
            label: "expressionOperator",
            comment: "The operator in an application expression (e.g., op:add, op:neg).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/ApplicationExpression"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/schema/leftOperand",
            label: "leftOperand",
            comment: "The left operand of an infix expression.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/InfixExpression"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/schema/rightOperand",
            label: "rightOperand",
            comment: "The right operand of an infix expression.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/InfixExpression"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/schema/arguments",
            label: "arguments",
            comment: "The argument list of an application expression. Non-functional: \
                      an application may take multiple arguments.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/schema/ApplicationExpression"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/schema/literalValue",
            label: "literalValue",
            comment: "The string representation of a literal expression value \
                      (e.g., '42', 'x', 'pi1').",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/LiteralExpression"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/schema/infixOperator",
            label: "infixOperator",
            comment: "The operator symbol in an infix expression (e.g., '=', \
                      '\\u{2264}', '\\u{2192}').",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/InfixExpression"),
            range: XSD_STRING,
        },
        // Amendment 95: Host-value sort properties (Workstream 5)
        Property {
            id: "https://uor.foundation/schema/hostString",
            label: "hostString",
            comment: "The string value carried by a HostStringLiteral.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/HostStringLiteral"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/schema/hostBoolean",
            label: "hostBoolean",
            comment: "The boolean value carried by a HostBooleanLiteral.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/HostBooleanLiteral"),
            range: XSD_BOOLEAN,
        },
        // v0.2.2 W8: Triad bundling — functional projection properties.
        // These form the canonical observable triple of a Datum at grounding time:
        // (stratum, spectrum, address). Cardinality-exactly-1 is enforced at the
        // Rust surface by making the Triad<L> struct fields non-Option.
        Property {
            id: "https://uor.foundation/schema/triadStratum",
            label: "triadStratum",
            comment: "The stratum component of a Triad: the datum's two-adic \
                      valuation, indexing its layer in the ring stratification. \
                      Semantically corresponds to query:TwoAdicValuation.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Triad"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/schema/triadSpectrum",
            label: "triadSpectrum",
            comment: "The spectrum component of a Triad: the datum's \
                      Walsh-Hadamard transform image, indexing its position in \
                      the hypercube spectral decomposition. Semantically \
                      corresponds to query:WalshHadamardImage.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Triad"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/schema/triadAddress",
            label: "triadAddress",
            comment: "The address component of a Triad: the datum's \
                      content-addressable position in the ring's Braille glyph \
                      encoding. Semantically corresponds to query:Address \
                      (renamed from RingElement in v0.2.2 W8).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/schema/Triad"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
    ]
}

/// Extracts the local name from an IRI (the part after the last `/`).
fn local_name(iri: &str) -> &str {
    match iri.rfind('/') {
        Some(pos) => &iri[pos + 1..],
        None => iri,
    }
}

/// Generates AST individuals (TermExpression / ForAllDeclaration) for all
/// identity individuals across op, homology, and cohomology namespaces.
///
/// Each identity's `lhs` and `rhs` string values become `LiteralExpression`
/// individuals; each `forAll` string becomes a `ForAllDeclaration` individual.
/// The IRI pattern is `schema/term_{localName}_{lhs|rhs|forAll}`.
///
/// Uses `Box::leak` to produce `&'static str` references from dynamic data.
fn generate_ast_individuals() -> Vec<Individual> {
    let identity_type = "https://uor.foundation/op/Identity";
    let lhs_prop = "https://uor.foundation/op/lhs";
    let rhs_prop = "https://uor.foundation/op/rhs";
    let forall_prop = "https://uor.foundation/op/forAll";

    let op_inds = super::op::raw_individuals();
    let hom_inds = super::homology::raw_individuals();
    let coh_inds = super::cohomology::raw_individuals();

    let mut ast = Vec::new();

    for individuals in &[&op_inds, &hom_inds, &coh_inds] {
        for ind in individuals.iter() {
            if ind.type_ != identity_type {
                continue;
            }

            let name = local_name(ind.id);

            for &(prop, ref val) in ind.properties {
                let is_lhs = prop == lhs_prop;
                let is_rhs = prop == rhs_prop;
                let is_forall = prop == forall_prop;

                if !is_lhs && !is_rhs && !is_forall {
                    continue;
                }

                // Only convert Str values — IriRef/List values (e.g.
                // criticalIdentity lhs/rhs) are already typed references.
                let text = match val {
                    IndividualValue::Str(s) => *s,
                    _ => continue,
                };

                let suffix = if is_lhs {
                    "lhs"
                } else if is_rhs {
                    "rhs"
                } else {
                    "forAll"
                };

                let (type_iri, value_prop) = if is_forall {
                    (
                        "https://uor.foundation/schema/ForAllDeclaration",
                        "https://uor.foundation/schema/variableName",
                    )
                } else {
                    (
                        "https://uor.foundation/schema/LiteralExpression",
                        "https://uor.foundation/schema/literalValue",
                    )
                };

                let id_string = format!("https://uor.foundation/schema/term_{name}_{suffix}");
                let label_string = format!("term_{name}_{suffix}");

                let id: &'static str = Box::leak(id_string.into_boxed_str());
                let label: &'static str = Box::leak(label_string.into_boxed_str());
                let val_str: &'static str = Box::leak(text.to_string().into_boxed_str());
                let props: &'static [(&'static str, IndividualValue)] =
                    Box::leak(vec![(value_prop, IndividualValue::Str(val_str))].into_boxed_slice());

                ast.push(Individual {
                    id,
                    type_: type_iri,
                    label,
                    comment: "",
                    properties: props,
                });
            }
        }
    }

    ast
}

fn individuals() -> Vec<Individual> {
    let mut base = vec![
        // Amendment 89: QuantifierKind vocabulary
        Individual {
            id: "https://uor.foundation/schema/Universal",
            type_: "https://uor.foundation/schema/QuantifierKind",
            label: "Universal",
            comment: "Universal quantification (forall).",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/schema/Existential",
            type_: "https://uor.foundation/schema/QuantifierKind",
            label: "Existential",
            comment: "Existential quantification (exists).",
            properties: &[],
        },
        // Amendment 2: pi1 — the generator (value = 1)
        Individual {
            id: "https://uor.foundation/schema/pi1",
            type_: "https://uor.foundation/schema/Datum",
            label: "π₁",
            comment: "The unique generator of R_n under successor. Value = 1 at every \
                      Witt level. Under iterated application of succ, π₁ generates \
                      every element of the ring.",
            properties: &[(
                "https://uor.foundation/schema/value",
                IndividualValue::Int(1),
            )],
        },
        // Amendment 2: zero — the additive identity
        Individual {
            id: "https://uor.foundation/schema/zero",
            type_: "https://uor.foundation/schema/Datum",
            label: "zero",
            comment: "The additive identity of the ring. Value = 0 at every Witt \
                      level. op:add(x, zero) = x for all x in R_n.",
            properties: &[(
                "https://uor.foundation/schema/value",
                IndividualValue::Int(0),
            )],
        },
        // v3.2: WittLevel individuals W8-W32
        Individual {
            id: "https://uor.foundation/schema/W8",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W8",
            comment: "Witt level 0: 8-bit ring Z/256Z, 256 states. The reference \
                      level for all ComputationCertificate proofs in the spec.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(8),
                ),
                (
                    "https://uor.foundation/schema/cycleSize",
                    IndividualValue::Int(256),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W16"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W16",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W16",
            comment: "Witt level 1: 16-bit ring Z/65536Z, 65,536 states.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(16),
                ),
                (
                    "https://uor.foundation/schema/cycleSize",
                    IndividualValue::Int(65536),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W24"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W24",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W24",
            comment: "Witt level 2: 24-bit ring Z/16777216Z, 16,777,216 states.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(24),
                ),
                (
                    "https://uor.foundation/schema/cycleSize",
                    IndividualValue::Int(16_777_216),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W32"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W16"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W32",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W32",
            comment: "Witt level 3: 32-bit ring Z/4294967296Z, 4,294,967,296 states. \
                      The highest 32-bit-and-below named level in the v0.2.1 spec; v0.2.2 \
                      Phase C extends the tower with the dense and powers-of-two set.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(32),
                ),
                (
                    "https://uor.foundation/schema/cycleSize",
                    IndividualValue::Int(4_294_967_296),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W40"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W24"),
                ),
            ],
        },
        // ─────────────────────────────────────────────────────────────────
        // v0.2.2 Phase C.1 — dense u64-backed Witt levels (W40, W48, W56, W64).
        // bit_width = 8·(k+1), k = 4..7. cycleSize fits in i64 for all four.
        // ─────────────────────────────────────────────────────────────────
        Individual {
            id: "https://uor.foundation/schema/W40",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W40",
            comment: "Witt level 4: 40-bit ring Z/2^40 Z. Backed by u64 with a \
                      40-bit mask at the arithmetic boundary. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(40),
                ),
                (
                    "https://uor.foundation/schema/cycleSize",
                    IndividualValue::Int(1_099_511_627_776),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W48"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W32"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W48",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W48",
            comment: "Witt level 5: 48-bit ring Z/2^48 Z. Backed by u64 with a \
                      48-bit mask at the arithmetic boundary. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(48),
                ),
                (
                    "https://uor.foundation/schema/cycleSize",
                    IndividualValue::Int(281_474_976_710_656),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W56"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W40"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W56",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W56",
            comment: "Witt level 6: 56-bit ring Z/2^56 Z. Backed by u64 with a \
                      56-bit mask at the arithmetic boundary. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(56),
                ),
                (
                    "https://uor.foundation/schema/cycleSize",
                    IndividualValue::Int(72_057_594_037_927_936),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W64"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W48"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W64",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W64",
            comment: "Witt level 7: 64-bit ring Z/2^64 Z. Backed by u64 directly \
                      (exact fit; no mask). v0.2.2 Phase C. cycle_size = 2^64 \
                      exceeds i64 representation and is omitted; codegen derives \
                      it from bit_width.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(64),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W72"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W56"),
                ),
            ],
        },
        // ─────────────────────────────────────────────────────────────────
        // v0.2.2 Phase C.2 — dense u128-backed Witt levels (W72..W128).
        // bit_width = 8·(k+1), k = 8..15. cycleSize is omitted for all
        // (2^bits > i64::MAX). Backing: u128 with bit-width mask at
        // arithmetic boundary; W128 is exact-fit (no mask).
        // ─────────────────────────────────────────────────────────────────
        Individual {
            id: "https://uor.foundation/schema/W72",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W72",
            comment: "Witt level 8: 72-bit ring Z/2^72 Z. Backed by u128 with a \
                      72-bit mask at the arithmetic boundary. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(72),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W80"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W64"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W80",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W80",
            comment: "Witt level 9: 80-bit ring Z/2^80 Z. Backed by u128 with an \
                      80-bit mask at the arithmetic boundary. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(80),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W88"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W72"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W88",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W88",
            comment: "Witt level 10: 88-bit ring Z/2^88 Z. Backed by u128 with an \
                      88-bit mask at the arithmetic boundary. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(88),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W96"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W80"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W96",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W96",
            comment: "Witt level 11: 96-bit ring Z/2^96 Z. Backed by u128 with a \
                      96-bit mask at the arithmetic boundary. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(96),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W104"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W88"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W104",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W104",
            comment: "Witt level 12: 104-bit ring Z/2^104 Z. Backed by u128 with \
                      a 104-bit mask at the arithmetic boundary. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(104),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W112"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W96"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W112",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W112",
            comment: "Witt level 13: 112-bit ring Z/2^112 Z. Backed by u128 with \
                      a 112-bit mask at the arithmetic boundary. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(112),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W120"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W104"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W120",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W120",
            comment: "Witt level 14: 120-bit ring Z/2^120 Z. Backed by u128 with \
                      a 120-bit mask at the arithmetic boundary. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(120),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W128"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W112"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W128",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W128",
            comment: "Witt level 15: 128-bit ring Z/2^128 Z. Backed by u128 \
                      directly (exact fit; no mask). The largest native-backed \
                      Witt level; levels above W128 use the Limbs<N> generic \
                      kernel emitted in Phase C.3. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(128),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W160"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W120"),
                ),
            ],
        },
        // ─────────────────────────────────────────────────────────────────
        // v0.2.2 Phase C.3 — Limbs<N>-backed Witt levels (W160..W32768).
        // Each level bit_width is a multiple of 8; backed by `Limbs<N>`
        // where N = ⌈bit_width / 64⌉. The chain is sorted by bit width:
        //   semantically-meaningful intermediates: W160 (SHA-1), W192,
        //   W224 (SHA-224), W384 (SHA-384, P-384), W448, W520, W528 (P-521),
        //   W12288, W32768.
        //   powers-of-two above native: W256, W512, W1024, W2048, W4096,
        //   W8192, W16384.
        // Total 16 individuals, sorted by bit width below.
        // ─────────────────────────────────────────────────────────────────
        Individual {
            id: "https://uor.foundation/schema/W160",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W160",
            comment: "Witt level: 160-bit ring (SHA-1 digest carrier). Backed \
                      by Limbs<3> with a 160-bit mask. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(160),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W192"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W128"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W192",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W192",
            comment: "Witt level: 192-bit ring (P-192 carrier). Backed by Limbs<3>. \
                      v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(192),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W224"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W160"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W224",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W224",
            comment: "Witt level: 224-bit ring (SHA-224 digest carrier). Backed by \
                      Limbs<4> with a 224-bit mask. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(224),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W256"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W192"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W256",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W256",
            comment: "Witt level: 256-bit ring (SHA-256, blake3, secp256k1, P-256 \
                      carrier). Backed by Limbs<4> directly (exact fit; no mask). \
                      v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(256),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W384"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W224"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W384",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W384",
            comment: "Witt level: 384-bit ring (SHA-384, P-384 carrier). Backed by \
                      Limbs<6> directly (exact fit; no mask). v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(384),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W448"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W256"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W448",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W448",
            comment: "Witt level: 448-bit ring (Curve448 carrier). Backed by \
                      Limbs<7> directly (exact fit; no mask). v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(448),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W512"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W384"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W512",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W512",
            comment: "Witt level: 512-bit ring (SHA-512 carrier). Backed by Limbs<8> \
                      directly (exact fit; no mask). v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(512),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W520"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W448"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W520",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W520",
            comment: "Witt level: 520-bit ring (P-521 prime carrier, lower-bound). \
                      Backed by Limbs<9> with a 520-bit mask. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(520),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W528"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W512"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W528",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W528",
            comment: "Witt level: 528-bit ring (P-521 prime carrier, upper-bound; \
                      P-521 elements are constrained by an additional residue check). \
                      Backed by Limbs<9> with a 528-bit mask. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(528),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W1024"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W520"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W1024",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W1024",
            comment: "Witt level: 1024-bit ring (RSA-1024 carrier). Backed by \
                      Limbs<16> directly (exact fit; no mask). v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(1024),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W2048"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W528"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W2048",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W2048",
            comment: "Witt level: 2048-bit ring (RSA-2048 carrier). Backed by \
                      Limbs<32> directly (exact fit; no mask). v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(2048),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W4096"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W1024"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W4096",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W4096",
            comment: "Witt level: 4096-bit ring (RSA-4096, BFV/CKKS HE ring \
                      dimension carrier). Backed by Limbs<64>. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(4096),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8192"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W2048"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W8192",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W8192",
            comment: "Witt level: 8192-bit ring (lattice-based crypto carrier). \
                      Backed by Limbs<128>. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(8192),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W12288"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W4096"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W12288",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W12288",
            comment: "Witt level: 12288-bit ring (BFV/BGV HE ring dimension at \
                      n=12288). Backed by Limbs<192>. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(12288),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W16384"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W8192"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W16384",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W16384",
            comment: "Witt level: 16384-bit ring (post-quantum lattice parameter). \
                      Backed by Limbs<256>. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(16384),
                ),
                (
                    "https://uor.foundation/schema/nextWittLevel",
                    IndividualValue::IriRef("https://uor.foundation/schema/W32768"),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W12288"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/schema/W32768",
            type_: "https://uor.foundation/schema/WittLevel",
            label: "W32768",
            comment: "Witt level: 32768-bit ring (post-quantum / extreme-precision \
                      arithmetic carrier). Backed by Limbs<512>. The highest \
                      foundation-shipped level in v0.2.2; downstream Prism \
                      implementations may declare higher levels via the \
                      `witt_level` conformance declaration form. v0.2.2 Phase C.",
            properties: &[
                (
                    "https://uor.foundation/schema/bitsWidth",
                    IndividualValue::Int(32768),
                ),
                (
                    "https://uor.foundation/schema/wittLevelPredecessor",
                    IndividualValue::IriRef("https://uor.foundation/schema/W16384"),
                ),
            ],
        },
    ];
    base.extend(generate_ast_individuals());
    base
}
