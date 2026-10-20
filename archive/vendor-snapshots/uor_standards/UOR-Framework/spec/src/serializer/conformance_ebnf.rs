//! Conformance grammar EBNF serializer (v0.2.1).
//!
//! Emits `public/uor.conformance.ebnf` from the ontology by walking
//! `conformance:Shape` individuals annotated with the v0.2.1 surface-grammar
//! metadata properties:
//!
//! - `conformance:surfaceForm` on each Shape names its top-level production.
//! - `conformance:requiredProperty` links to the constraint individuals.
//! - On each `conformance:PropertyConstraint`:
//!   - `conformance:surfaceKeyword` is the literal keyword in the grammar.
//!   - `conformance:surfaceProduction` is the value-slot non-terminal.
//!   - `conformance:minCount` / `conformance:maxCount` drive cardinality.
//!
//! The grammar is the v0.2.1 companion to `uor.term.ebnf`. Adding a new
//! conformance shape (or a new property constraint on an existing shape)
//! requires only an ontology edit; this serializer regenerates the grammar
//! file from the new metadata with no Rust changes.

use crate::model::{IndividualValue, Ontology};

/// Target line width for section header comments.
const HEADER_WIDTH: usize = 80;

/// Render a section header in the same style as the term grammar emitter.
fn section_header(title: &str) -> String {
    let prefix = format!("(* \u{2500}\u{2500} {} ", title);
    let prefix_chars = prefix.chars().count();
    let remaining = HEADER_WIDTH.saturating_sub(prefix_chars + 3);
    let dashes = "\u{2500}".repeat(remaining);
    format!("{prefix}{dashes} *)")
}

/// Read a property string value off an individual (IriRef or Str).
fn ind_prop_str<'a>(ind: &'a crate::model::Individual, prop_iri: &str) -> Option<&'a str> {
    for (k, v) in ind.properties {
        if *k == prop_iri {
            return match v {
                IndividualValue::IriRef(s) | IndividualValue::Str(s) => Some(s),
                _ => None,
            };
        }
    }
    None
}

/// Read all matching properties as IriRefs (for non-functional properties).
fn ind_prop_iris<'a>(ind: &'a crate::model::Individual, prop_iri: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    for (k, v) in ind.properties {
        if *k == prop_iri {
            if let IndividualValue::IriRef(s) = v {
                out.push(*s);
            }
        }
    }
    out
}

/// Read an integer property.
fn ind_prop_int(ind: &crate::model::Individual, prop_iri: &str) -> Option<i64> {
    for (k, v) in ind.properties {
        if *k == prop_iri {
            if let IndividualValue::Int(n) = v {
                return Some(*n);
            }
        }
    }
    None
}

/// Find an individual by IRI.
fn find_individual<'a>(ontology: &'a Ontology, iri: &str) -> Option<&'a crate::model::Individual> {
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            if ind.id == iri {
                return Some(ind);
            }
        }
    }
    None
}

/// All individuals of a given type.
fn individuals_of_type<'a>(
    ontology: &'a Ontology,
    type_iri: &str,
) -> Vec<&'a crate::model::Individual> {
    let mut out = Vec::new();
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            if ind.type_ == type_iri {
                out.push(ind);
            }
        }
    }
    out
}

/// Serialize the conformance grammar to EBNF.
///
/// The output is a complete ISO/IEC 14977 grammar covering the seven
/// `conformance:*Declaration` shapes (compile-unit, dispatch-rule,
/// witt-level, predicate, parallel, stream, lease).
///
/// # Errors
///
/// This function is infallible.
#[must_use]
pub fn to_conformance_ebnf(ontology: &Ontology) -> String {
    let mut out = String::with_capacity(8 * 1024);

    emit_header(&mut out, ontology);
    emit_top_level(&mut out, ontology);
    emit_shape_productions(&mut out, ontology);
    emit_phase_d_constraint_productions(&mut out);
    emit_lexical(&mut out);

    out
}

/// v0.2.2 Phase D / T2.3 (cleanup): emits the parametric constraint
/// declaration grammar.
///
/// Emits `constraint-decl`, `conjunction-decl`, and the 6 legacy-sugar
/// productions for the closed (observable, bound_shape) catalogue. The
/// hand-coded preamble lands after the Shape walker. Adding a new pair
/// requires editing both this emitter and the ontology individuals.
fn emit_phase_d_constraint_productions(out: &mut String) {
    out.push_str("(* ");
    out.push_str(&"=".repeat(76));
    out.push('\n');
    out.push_str("   v0.2.2 Phase D / T2.3 — parametric constraint declaration grammar.\n");
    out.push_str("   ");
    out.push_str(&"=".repeat(76));
    out.push_str(" *)\n\n");

    out.push_str("(* Parametric constraint declaration. Selects an observable + bound\n");
    out.push_str("   shape from the closed Phase D catalogue and supplies the\n");
    out.push_str("   kind-specific arguments. *)\n");
    out.push_str("constraint-decl        ::= \"constraint\" , identifier , \"{\" ,\n");
    out.push_str(
        "                             \"observable\" , \":\" , observable-iri , \";\" ,\n",
    );
    out.push_str("                             \"shape\" , \":\" , bound-shape-iri , \";\" ,\n");
    out.push_str(
        "                             \"args\" , \":\" , \"{\" , arg-list , \"}\" , \";\" ,\n",
    );
    out.push_str("                           \"}\" ;\n\n");

    out.push_str("observable-iri         ::= \"observable:\" , (\n");
    out.push_str("                             \"ValueModObservable\"\n");
    out.push_str("                           | \"HammingMetric\" )\n");
    out.push_str("                         | \"derivation:\" , \"DerivationDepthObservable\"\n");
    out.push_str("                         | \"carry:\" , \"CarryDepthObservable\"\n");
    out.push_str("                         | \"partition:\" , \"FreeRankObservable\" ;\n\n");

    out.push_str("bound-shape-iri        ::= \"type:\" , (\n");
    out.push_str("                             \"EqualBound\"\n");
    out.push_str("                           | \"LessEqBound\"\n");
    out.push_str("                           | \"GreaterEqBound\"\n");
    out.push_str("                           | \"RangeContainBound\"\n");
    out.push_str("                           | \"ResidueClassBound\"\n");
    out.push_str("                           | \"AffineEqualBound\" ) ;\n\n");

    out.push_str("arg-list               ::= arg , { \",\" , arg } ;\n");
    out.push_str(
        "arg                    ::= identifier , \":\" , ( integer-literal | datum-literal ) ;\n\n",
    );

    out.push_str("(* Conjunction of BoundConstraint instances, ordered. *)\n");
    out.push_str("conjunction-decl       ::= \"conjunction\" , identifier , \"{\" ,\n");
    out.push_str("                             \"conjuncts\" , \":\" , \"[\" ,\n");
    out.push_str("                             identifier , { \",\" , identifier } ,\n");
    out.push_str("                             \"]\" , \";\" ,\n");
    out.push_str("                           \"}\" ;\n\n");

    out.push_str("(* Legacy sugar forms (v0.2.1 compatibility — desugar to constraint-decl). *)\n");
    out.push_str("residue-sugar          ::= \"residue\" , \"{\" ,\n");
    out.push_str("                             \"modulus\" , \":\" , integer-literal , \";\" ,\n");
    out.push_str("                             \"residue\" , \":\" , integer-literal , \";\" ,\n");
    out.push_str("                           \"}\" ;\n");
    out.push_str("hamming-sugar          ::= \"hamming\" , \"{\" ,\n");
    out.push_str("                             \"bound\" , \":\" , integer-literal , \";\" ,\n");
    out.push_str("                           \"}\" ;\n");
    out.push_str("depth-sugar            ::= \"depth\" , \"{\" ,\n");
    out.push_str(
        "                             \"min_depth\" , \":\" , integer-literal , \";\" ,\n",
    );
    out.push_str(
        "                             \"max_depth\" , \":\" , integer-literal , \";\" ,\n",
    );
    out.push_str("                           \"}\" ;\n");
    out.push_str("carry-sugar            ::= \"carry\" , \"{\" ,\n");
    out.push_str("                             \"bound\" , \":\" , integer-literal , \";\" ,\n");
    out.push_str("                           \"}\" ;\n");
    out.push_str("site-sugar             ::= \"site\" , \"{\" ,\n");
    out.push_str(
        "                             \"site_index\" , \":\" , integer-literal , \";\" ,\n",
    );
    out.push_str("                           \"}\" ;\n");
    out.push_str("affine-sugar           ::= \"affine\" , \"{\" ,\n");
    out.push_str("                             \"offset\" , \":\" , integer-literal , \";\" ,\n");
    out.push_str("                           \"}\" ;\n\n");
}

fn emit_header(out: &mut String, ontology: &Ontology) {
    out.push_str("(* ");
    out.push_str(&"=".repeat(76));
    out.push('\n');
    out.push_str("   UOR Conformance Declaration Grammar — EBNF\n");
    out.push_str(&format!(
        "   Specification version: v{}\n",
        ontology.version
    ));
    out.push_str("   Authoritative source: https://uor.foundation/\n");
    out.push_str("   Notation: ISO/IEC 14977.\n");
    out.push('\n');
    out.push_str("   This grammar is machine-generated from the ontology's\n");
    out.push_str("   conformance:Shape and conformance:PropertyConstraint individuals\n");
    out.push_str("   via spec/src/serializer/conformance_ebnf.rs. Adding a new\n");
    out.push_str("   declaration shape requires only an ontology edit.\n");
    out.push('\n');
    out.push_str("   This grammar is a companion to uor_term.ebnf. The term grammar\n");
    out.push_str("   describes the free term tree; this grammar describes the\n");
    out.push_str("   conformance-shape envelope around terms.\n");
    out.push('\n');
    out.push_str("   Layering. This grammar imports from uor_term.ebnf:\n");
    out.push_str("     - `program`     (rootTerm slot of compile-unit-decl)\n");
    out.push_str("     - `term`        (wherever a value is expected)\n");
    out.push_str("     - `type-expr`   (TypeDefinition slots)\n");
    out.push_str("     - `name`        (ontology-resolved identifier)\n");
    out.push_str("     - `identifier`  (fresh binding occurrence)\n");
    out.push_str("     - `integer-literal`, `string-literal`, `boolean-literal`\n");
    out.push_str("     - whitespace and comment lexicals\n");
    out.push_str("   ");
    out.push_str(&"=".repeat(76));
    out.push_str(" *)\n\n");
}

fn emit_top_level(out: &mut String, ontology: &Ontology) {
    out.push_str("conformance-program    ::= { conformance-decl } ;\n\n");

    // Collect surface-form names from all conformance:Shape individuals.
    let mut shape_specs: Vec<(String, String, &crate::model::Individual)> = Vec::new();
    let shapes = individuals_of_type(ontology, "https://uor.foundation/conformance/Shape");
    for s in shapes {
        if let Some(form) = ind_prop_str(s, "https://uor.foundation/conformance/surfaceForm") {
            // The surface-form keyword is the production's leading literal,
            // recovered as everything before the first hyphen-decl suffix.
            let keyword = form.strip_suffix("-decl").unwrap_or(form).replace('-', "_");
            shape_specs.push((form.to_string(), keyword, s));
        }
    }
    shape_specs.sort_by(|a, b| a.0.cmp(&b.0));

    out.push_str("conformance-decl       ::= ");
    let mut first = true;
    for (form, _kw, _ind) in &shape_specs {
        if first {
            out.push_str(form);
            first = false;
        } else {
            out.push_str("\n                         | ");
            out.push_str(form);
        }
    }
    out.push_str(" ;\n\n");
}

fn emit_shape_productions(out: &mut String, ontology: &Ontology) {
    let mut shape_specs: Vec<(String, &crate::model::Individual)> = Vec::new();
    let shapes = individuals_of_type(ontology, "https://uor.foundation/conformance/Shape");
    for s in shapes {
        if let Some(form) = ind_prop_str(s, "https://uor.foundation/conformance/surfaceForm") {
            shape_specs.push((form.to_string(), s));
        }
    }
    shape_specs.sort_by(|a, b| a.0.cmp(&b.0));

    for (form, shape_ind) in &shape_specs {
        let label = shape_ind.label;
        out.push_str(&section_header(label));
        out.push('\n');

        // The decl-keyword is everything before "-decl"
        let keyword = form.strip_suffix("-decl").unwrap_or(form).replace('-', "_");
        let prop_form = format!("{}-prop", form.strip_suffix("-decl").unwrap_or(form));

        // Top decl production: `compile_unit identifier { { compile-unit-prop } }`
        out.push_str(&format!(
            "{form: <22} ::= \"{keyword}\" , identifier , \"{{\" ,\n",
            form = form
        ));
        out.push_str(&format!("{:<22}     {{ {prop_form} }} ,\n", ""));
        out.push_str(&format!("{:<22}     \"}}\" ;\n\n", ""));

        // Collect required-property constraint individuals.
        let constraint_iris = ind_prop_iris(
            shape_ind,
            "https://uor.foundation/conformance/requiredProperty",
        );
        let mut constraint_inds: Vec<&crate::model::Individual> = constraint_iris
            .iter()
            .filter_map(|iri| find_individual(ontology, iri))
            .collect();
        // Stable order: by surfaceKeyword.
        constraint_inds.sort_by_key(|c| {
            ind_prop_str(c, "https://uor.foundation/conformance/surfaceKeyword")
                .unwrap_or("")
                .to_string()
        });

        // Disjunction body for prop production
        out.push_str(&format!("{prop_form: <22} ::= ", prop_form = prop_form));
        let mut first = true;
        for c in &constraint_inds {
            let kw =
                ind_prop_str(c, "https://uor.foundation/conformance/surfaceKeyword").unwrap_or("?");
            let body_name = format!("{kw}-prop");
            if first {
                out.push_str(&body_name);
                first = false;
            } else {
                out.push_str(&format!("\n{:<22}   | {body_name}", ""));
            }
        }
        out.push_str(" ;\n\n");

        // Body productions for each constraint
        for c in &constraint_inds {
            let kw =
                ind_prop_str(c, "https://uor.foundation/conformance/surfaceKeyword").unwrap_or("?");
            let prod = ind_prop_str(c, "https://uor.foundation/conformance/surfaceProduction")
                .unwrap_or("term");
            let body_name = format!("{kw}-prop");
            let max_count = ind_prop_int(c, "https://uor.foundation/conformance/maxCount");
            // maxCount == 0 means unbounded; emit a set form whose name
            // doesn't double-suffix when prod already ends in `-set`.
            let value_part = if max_count == Some(0) {
                if prod.ends_with("-set") {
                    prod.to_string()
                } else {
                    format!("{prod}-set")
                }
            } else if prod == "program" {
                "\"{\" , program , \"}\"".to_string()
            } else {
                prod.to_string()
            };
            out.push_str(&format!(
                "{body_name: <22} ::= \"{kw}\" , \":\" , {value_part} , \";\" ;\n",
                body_name = body_name
            ));
        }
        out.push('\n');

        // For unbounded slots, emit the corresponding set production once
        // per shape (e.g. domain-set ::= "{" , name , { "," , name } , "}").
        let mut emitted_sets: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        for c in &constraint_inds {
            let max_count = ind_prop_int(c, "https://uor.foundation/conformance/maxCount");
            if max_count != Some(0) {
                continue;
            }
            let prod = ind_prop_str(c, "https://uor.foundation/conformance/surfaceProduction")
                .unwrap_or("name");
            let (set_name, base_prod) = if prod.ends_with("-set") {
                (prod.to_string(), prod.trim_end_matches("-set").to_string())
            } else {
                (format!("{prod}-set"), prod.to_string())
            };
            if emitted_sets.insert(set_name.clone()) {
                out.push_str(&format!(
                    "{set_name: <22} ::= \"{{\" , {base_prod} , {{ \",\" , {base_prod} }} , \"}}\" ;\n",
                    set_name = set_name
                ));
            }
        }
        if !emitted_sets.is_empty() {
            out.push('\n');
        }
    }
}

fn emit_lexical(out: &mut String) {
    out.push_str(&section_header("Lexical additions"));
    out.push('\n');
    out.push_str("(* This grammar introduces one lexical production not present in\n");
    out.push_str("   uor_term.ebnf: decimal-literal, needed by thermodynamic-budget-prop.\n");
    out.push_str("   All other lexicals (integer-literal, string-literal, boolean-literal,\n");
    out.push_str("   identifier, name, whitespace, line-comment, block-comment) are\n");
    out.push_str("   inherited from uor_term.ebnf without redefinition. *)\n\n");
    out.push_str(
        "decimal-literal        ::= [ \"-\" ] , integer-literal ,\n\
         \x20                          [ \".\" , digit , { digit } ] ,\n\
         \x20                          [ ( \"e\" | \"E\" ) , [ \"+\" | \"-\" ] , integer-literal ] ;\n\n"
    );

    out.push_str("(* ");
    out.push_str(&"=".repeat(74));
    out.push_str(" *)\n");
    out.push_str("(* End of grammar *)\n");
}
