//! EBNF serializer for the UOR Term Language grammar (Amendment 95).
//!
//! Generates an ISO/IEC 14977 EBNF grammar from the ontology, output to
//! `public/uor.term.ebnf`. The grammar describes the free term tree;
//! equational structure is enforced by the resolver, not by the grammar.

use crate::model::Ontology;

/// Target line width for section header comments (matches the target EBNF).
const HEADER_WIDTH: usize = 80;

/// Generates a section header comment line in the EBNF style.
///
/// Single-line headers (no body text) produce `(* \u{2500}\u{2500} Name \u{2500}...\u{2500} *)`.
/// Multi-line headers (with body text) produce `(* \u{2500}\u{2500} Name \u{2500}...\n`.
fn section_header(title: &str, close: bool) -> String {
    let prefix = format!("(* \u{2500}\u{2500} {} ", title);
    let prefix_chars = prefix.chars().count();
    if close {
        let remaining = HEADER_WIDTH.saturating_sub(prefix_chars);
        let dashes = "\u{2500}".repeat(remaining);
        format!("{prefix}{dashes} *)")
    } else {
        let remaining = (HEADER_WIDTH + 1).saturating_sub(prefix_chars);
        let dashes = "\u{2500}".repeat(remaining);
        format!("{prefix}{dashes}")
    }
}

/// Serializes the UOR Term Language grammar as ISO/IEC 14977 EBNF.
///
/// The grammar is a static template parameterised only by the ontology
/// version string.
///
/// # Errors
///
/// This function is infallible; it always returns a valid EBNF string.
#[must_use]
pub fn to_ebnf(ontology: &Ontology) -> String {
    let mut out = String::with_capacity(8 * 1024);

    emit_header(&mut out, ontology);
    emit_top_level(&mut out);
    emit_terms(&mut out);
    emit_applications(&mut out);
    emit_variables(&mut out);
    emit_type_expr(&mut out);
    emit_type_decl(&mut out);
    emit_bindings(&mut out);
    emit_assertions(&mut out);
    emit_match_expr(&mut out);
    emit_stream_expr(&mut out);
    emit_set_expr(&mut out);
    emit_value_productions(&mut out);
    emit_effect_decl(&mut out);
    emit_try_expr(&mut out);
    emit_recurse_expr(&mut out);
    emit_boundary_decl(&mut out);
    emit_whitespace(&mut out);

    out
}

// ── Section emitters ────────────────────────────────────────────────────────

/// Emits the header block with version and ISO/IEC 14977 notation guide.
fn emit_header(out: &mut String, ontology: &Ontology) {
    let _ = std::fmt::Write::write_fmt(
        out,
        format_args!(
            "(* ============================================================================\n\
             \x20  UOR Term Language \u{2014} EBNF\n\
             \x20  Specification version: v{version}\n\
             \x20  Authoritative source: https://uor.foundation/\n\
             \x20  Notation: ISO/IEC 14977.\n\
             \n\
             \x20  The grammar describes the free term tree. Equational structure\n\
             \x20  (operad axioms, rewrite rules, canonical forms) is enforced by the\n\
             \x20  resolver, not by the grammar.\n\
             \x20  ============================================================================ *)\n\n",
            version = ontology.version
        ),
    );
}

/// Emits the top-level entry point rules.
fn emit_top_level(out: &mut String) {
    out.push_str(
        "program          ::= { statement } ;\n\
         \n\
         statement        ::= type-decl\n\
         \x20                  | binding\n\
         \x20                  | assertion\n\
         \x20                  | effect-decl\n\
         \x20                  | boundary-decl\n\
         \x20                  | expression , \";\" ;\n\
         \n\
         expression       ::= term ;\n\n",
    );
}

/// Emits the term, literal, and related rules.
fn emit_terms(out: &mut String) {
    out.push_str(&section_header(
        "Terms (schema:Term, disjoint from schema:Datum)",
        true,
    ));
    out.push_str(
        "\n\n\
         term             ::= literal\n\
         \x20                  | application\n\
         \x20                  | variable\n\
         \x20                  | match-expr\n\
         \x20                  | try-expr\n\
         \x20                  | recurse-expr\n\
         \x20                  | stream-expr\n\
         \x20                  | set-expr ;\n\
         \n\
         literal          ::= integer-literal\n\
         \x20                  | element-literal\n\
         \x20                  | witt-literal ;\n\
         \n\
         integer-literal  ::= digit , { digit } ;\n\
         \n\
         element-literal  ::= byte-element , { byte-element } ;\n\
         byte-element     ::= \"\\u2800\" .. \"\\u28FF\" ;\n\
         \n\
         witt-literal     ::= integer-literal , \"@\" , name ;\n\
         \x20                  (* name \u{2192} individual of schema:WittLevel. *)\n\n",
    );
}

/// Emits the application rule (n-ary).
fn emit_applications(out: &mut String) {
    out.push_str(&section_header("Application (schema:Application)", false));
    out.push_str(
        "\n\
         \x20  The n-ary form realises operadic substitution \u{03B8} \u{2218} \
         (\u{03B8}\u{2081},\u{2026},\u{03B8}\u{2096}).\n\
         \x20  Arity is checked semantically against op:arity for the resolved name.\n\
         \x20  The \u{03A3}\u{2099} action and associative reassociation are quotients applied by the\n\
         \x20  canonical-form resolver, not constraints of this grammar.\n\
         \x20  Asymmetry with type-app: application admits \u{2265} 0 args (so op:arity = 0\n\
         \x20  is expressible); type-app requires \u{2265} 1 because a nullary type\n\
         \x20  constructor is already covered by the name alternative of type-expr. *)\n\
         \n\
         application      ::= name , \"(\" , [ term , { \",\" , term } ] , \")\" ;\n\
         \x20                  (* name \u{2192} individual of op:Operation (a UnaryOp, BinaryOp,\n\
         \x20                     or Involution member). op:ComposedOperation individuals\n\
         \x20                     are synthesised by the resolver from nested applications\n\
         \x20                     and have no introduction form in this grammar. *)\n\n",
    );
}

/// Emits variable and identifier rules.
fn emit_variables(out: &mut String) {
    out.push_str(
        "variable         ::= identifier ;\n\
         \n\
         (* An identifier whose meaning is resolved through the ontology rather\n\
         \x20  than the local binding context. Each use site comments which\n\
         \x20  ontology class the name must resolve to. *)\n\
         name             ::= identifier ;\n\
         \n\
         identifier       ::= alpha , { alpha | digit | \"_\" } ;\n\
         alpha            ::= \"a\" .. \"z\" | \"A\" .. \"Z\" ;\n\
         digit            ::= \"0\" .. \"9\" ;\n\n",
    );
}

/// Emits type expression rules.
fn emit_type_expr(out: &mut String) {
    out.push_str(&section_header(
        "Type expressions (operad:StructuralOperad)",
        false,
    ));
    out.push_str(
        "\n\
         \x20  type-app is an operadic composition over the structural operad and\n\
         \x20  materialises an operad:OperadComposition individual whose outerType,\n\
         \x20  innerType, composedType, composedSiteCount and composedGrounding are\n\
         \x20  populated from the AST and the carry layer. *)\n\
         \n\
         type-expr        ::= name\n\
         \x20                  | type-app ;\n\
         \x20                  (* name \u{2192} owl:NamedIndividual of type:TypeDefinition\n\
         \x20                     (transitively, via any subclass). Category classes\n\
         \x20                     (PrimitiveType, ProductType, SumType, ConstrainedType)\n\
         \x20                     and state classes (CompleteType, SuperposedSiteState,\n\
         \x20                     CompletenessCandidate, SynthesizedType, TwistedType,\n\
         \x20                     FlatType) are not valid targets at this position. *)\n\
         \n\
         type-app         ::= name , \"(\" , type-expr , { \",\" , type-expr } , \")\" ;\n\
         \x20                  (* name \u{2192} owl:NamedIndividual of type:TypeDefinition\n\
         \x20                     (a parameterised one). The application materialises\n\
         \x20                     a fresh operad:OperadComposition individual whose\n\
         \x20                     outerType, innerType, composedType, composedSiteCount,\n\
         \x20                     and composedGrounding are populated from the AST and\n\
         \x20                     the carry layer, and a fresh type:TypeDefinition\n\
         \x20                     individual representing the composed type. *)\n\n",
    );
}

/// Emits type declaration rules.
fn emit_type_decl(out: &mut String) {
    out.push_str(&section_header(
        "Type declarations (type:TypeDefinition)",
        false,
    ));
    out.push_str(
        "\n\
         \x20  Parameterised declarations carry the colored structural operad.\n\
         \x20  Per-position variance (type:positionVariance, range type:VarianceAnnotation)\n\
         \x20  is asserted via constraint-decl in the body, not encoded as syntax. *)\n\
         \n\
         type-decl        ::= \"type\" , identifier , [ type-params ] ,\n\
         \x20                    \"{\" , { constraint-decl } , \"}\" ;\n\
         \n\
         type-params      ::= \"(\" , type-param , { \",\" , type-param } , \")\" ;\n\
         type-param       ::= identifier ;\n\
         \n\
         constraint-decl  ::= name , \"(\" , [ constraint-arg , { \",\" , constraint-arg } ] , \")\" , \";\" ;\n\
         constraint-arg   ::= name , \":\" , term ;\n\
         \x20                  (* outer name \u{2192} subclass of type:Constraint. The currently\n\
         \x20                     declared subclasses are ResidueConstraint,\n\
         \x20                     CarryConstraint, DepthConstraint, HammingConstraint,\n\
         \x20                     SiteConstraint, AffineConstraint. CompositeConstraint\n\
         \x20                     is the implicit shape of any type-decl body containing\n\
         \x20                     more than one constraint-decl and has no syntactic\n\
         \x20                     constructor. inner name (in constraint-arg) \u{2192} property\n\
         \x20                     whose rdfs:domain is the outer subclass; the trailing\n\
         \x20                     term is the value, type-checked against the property's\n\
         \x20                     rdfs:range and required to be Datum-denoting. Functional\n\
         \x20                     properties accept exactly one matching constraint-arg;\n\
         \x20                     non-functional properties accept either repeated\n\
         \x20                     constraint-args with the same field name or a single\n\
         \x20                     set-expr value that unpacks. *)\n\n",
    );
}

/// Emits binding rules.
fn emit_bindings(out: &mut String) {
    out.push_str(&section_header("Bindings", true));
    out.push_str(
        "\n\n\
         binding          ::= \"let\" , identifier , \":\" , type-expr , \"=\" , term , \";\" ;\n\n",
    );
}

/// Emits assertion rules.
fn emit_assertions(out: &mut String) {
    out.push_str(&section_header("Assertions", false));
    out.push_str(
        "\n\
         \x20  The asserted term must resolve to a verdict at the predicate layer.\n\
         \x20  Equality flavours (ring equality, canonical-form equivalence, etc.) are\n\
         \x20  selected by the predicate identifier appearing in the term, not by syntax. *)\n\
         \n\
         assertion        ::= \"assert\" , term , \";\" ;\n\n",
    );
}

/// Emits match expression rules.
fn emit_match_expr(out: &mut String) {
    out.push_str(&section_header("Match", true));
    out.push_str(
        "\n\n\
         match-expr       ::= \"match\" , term , \"{\" , { match-arm } , \"}\" ;\n\
         match-arm        ::= name , \"=>\" , term , \";\" ;\n\
         \x20                  (* name \u{2192} individual of predicate:Predicate (a TypePredicate,\n\
         \x20                     StatePredicate, or SitePredicate member). *)\n\n",
    );
}

/// Emits stream constructor rules.
fn emit_stream_expr(out: &mut String) {
    out.push_str(&section_header("Streams", true));
    out.push_str(
        "\n\n\
         stream-expr      ::= \"unfold\" , identifier , \":\" , type-expr , \"from\" , term ;\n\n",
    );
}

/// Emits set expression rules.
fn emit_set_expr(out: &mut String) {
    out.push_str(&section_header("Sets", false));
    out.push_str(
        "\n\
         \x20  A constructor form, not a literal: elements may be arbitrary terms.\n\
         \x20  Element order in the source is parser-arbitrary; the resolver\n\
         \x20  canonicalises by content address (the same Normalization quotient\n\
         \x20  applied to commutative applications). *)\n\
         \n\
         set-expr         ::= \"{\" , [ term , { \",\" , term } ] , \"}\" ;\n\n",
    );
}

/// Emits value, host-literal, string-literal, boolean-literal productions.
fn emit_value_productions(out: &mut String) {
    out.push_str(&section_header("Property values", false));
    out.push_str(
        "\n\
         \x20  A value is what fills a property-position slot whose ontology range may\n\
         \x20  be a Datum (term) or an xsd-typed host datum (host-literal). The two\n\
         \x20  sorts are syntactically disjoint. Operations are mono-sorted over Datum\n\
         \x20  terms \u{2014} host literals can never appear inside an application \u{2014} so the\n\
         \x20  bi-sortedness is enforced by the grammar, not deferred to type checking. *)\n\
         \n\
         value            ::= term | host-literal ;\n\
         \n\
         host-literal     ::= string-literal | boolean-literal ;\n\
         \x20                  (* No host-integer-literal: integer-literal in term\n\
         \x20                     position already projects to xsd:integer through the\n\
         \x20                     Datum integer projection, and the resolver coerces\n\
         \x20                     when the property range is xsd:integer. *)\n\
         \n\
         string-literal   ::= '\"' , { string-char } , '\"' ;\n\
         string-char      ::= any-char-except-quote-or-backslash\n\
         \x20                  | \"\\\\\" , escape-char ;\n\
         escape-char      ::= '\"' | \"\\\\\" | \"n\" | \"r\" | \"t\" | \"u\" , hex , hex , hex , hex ;\n\
         hex              ::= digit | \"a\" .. \"f\" | \"A\" .. \"F\" ;\n\
         \n\
         boolean-literal  ::= \"true\" | \"false\" ;\n\n",
    );
}

/// Emits effect declaration rules.
fn emit_effect_decl(out: &mut String) {
    out.push_str(&section_header("Effects", false));
    out.push_str(
        "\n\
         \x20  Property keys must resolve to ontology-declared properties on the\n\
         \x20  target effect class; the property's range constrains the value. *)\n\
         \n\
         effect-decl      ::= \"effect\" , identifier , \"{\" , { effect-prop } , \"}\" ;\n\
         effect-prop      ::= name , \":\" , value , \";\" ;\n\
         \x20                  (* name \u{2192} property IRI (an owl:DatatypeProperty or\n\
         \x20                     owl:ObjectProperty whose domain is the effect class).\n\
         \x20                     The \"effect\" identifier is a fresh binding occurrence\n\
         \x20                     introducing a new effect:Effect individual. *)\n\n",
    );
}

/// Emits failure and recovery rules.
fn emit_try_expr(out: &mut String) {
    out.push_str(&section_header("Failure and recovery", true));
    out.push_str(
        "\n\n\
         try-expr         ::= \"try\" , term , \"{\" , { recover-arm } , \"}\" ;\n\
         recover-arm      ::= \"recover\" , name , \"=>\" , term , \";\" ;\n\
         \x20                  (* name \u{2192} subclass of failure:FailureReason. The currently\n\
         \x20                     declared subclasses are GuardFailure,\n\
         \x20                     ConstraintContradiction, SiteExhaustion,\n\
         \x20                     LiftObstructionFailure. *)\n\n",
    );
}

/// Emits bounded recursion rules.
fn emit_recurse_expr(out: &mut String) {
    out.push_str(&section_header("Bounded recursion", true));
    out.push_str(
        "\n\n\
         recurse-expr     ::= \"recurse\" , identifier , \"(\" , term , \")\" ,\n\
         \x20                    \"measure\" , term ,\n\
         \x20                    base-arm , { base-arm } ,\n\
         \x20                    \"step\" , \"=>\" , term ;\n\
         base-arm         ::= \"base\" , name , \"=>\" , term ;\n\
         \x20                  (* name \u{2192} individual of predicate:Predicate. *)\n\n",
    );
}

/// Emits IO boundary declaration rules.
fn emit_boundary_decl(out: &mut String) {
    out.push_str(&section_header("IO boundary declarations", true));
    out.push_str(
        "\n\n\
         boundary-decl    ::= source-decl | sink-decl ;\n\
         source-decl      ::= \"source\" , identifier , \":\" , type-expr , \"via\" , name , \";\" ;\n\
         \x20                  (* \"via\" name \u{2192} individual of morphism:GroundingMap.\n\
         \x20                     The leading identifier is a fresh binding occurrence\n\
         \x20                     introducing a new boundary:Source individual. *)\n\
         sink-decl        ::= \"sink\"   , identifier , \":\" , type-expr , \"via\" , name , \";\" ;\n\
         \x20                  (* \"via\" name \u{2192} individual of morphism:ProjectionMap.\n\
         \x20                     The leading identifier is a fresh binding occurrence\n\
         \x20                     introducing a new boundary:Sink individual. *)\n\n",
    );
}

/// Emits whitespace and comment rules.
fn emit_whitespace(out: &mut String) {
    out.push_str(&section_header("Lexical", true));
    out.push_str(
        "\n\n\
         whitespace       ::= \" \" | \"\\t\" | \"\\n\" | \"\\r\" ;\n\
         line-comment     ::= \"--\" , { any-char-except-newline } , \"\\n\" ;\n\
         block-comment    ::= \"(*\" , { any-char } , \"*)\" ;\n",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ontology;

    #[test]
    fn produces_non_empty_ebnf() {
        let ontology = Ontology::full();
        let ebnf = to_ebnf(ontology);
        assert!(!ebnf.is_empty());
        assert!(ebnf.contains("::="));
    }

    #[test]
    fn contains_version() {
        let ontology = Ontology::full();
        let ebnf = to_ebnf(ontology);
        assert!(ebnf.contains(ontology.version));
    }

    #[test]
    fn balanced_comments() {
        let ontology = Ontology::full();
        let ebnf = to_ebnf(ontology);
        let opens = ebnf.matches("(*").count();
        let closes = ebnf.matches("*)").count();
        assert_eq!(
            opens, closes,
            "Unbalanced EBNF comments: {opens} opens vs {closes} closes"
        );
    }

    #[test]
    fn contains_constraint_arg() {
        let ontology = Ontology::full();
        let ebnf = to_ebnf(ontology);
        assert!(
            ebnf.contains("constraint-arg"),
            "Missing constraint-arg production"
        );
    }

    #[test]
    fn contains_value_production() {
        let ontology = Ontology::full();
        let ebnf = to_ebnf(ontology);
        assert!(ebnf.contains("value"), "Missing value production");
        assert!(
            ebnf.contains("string-literal"),
            "Missing string-literal production"
        );
        assert!(
            ebnf.contains("boolean-literal"),
            "Missing boolean-literal production"
        );
    }

    #[test]
    fn contains_type_expr() {
        let ontology = Ontology::full();
        let ebnf = to_ebnf(ontology);
        assert!(ebnf.contains("type-expr"), "Missing type-expr production");
        assert!(ebnf.contains("type-app"), "Missing type-app production");
    }
}
