//! N-Triples serializer for the UOR Foundation ontology.
//!
//! Produces a valid N-Triples document (one triple per line, absolute IRIs).
//! N-Triples is suitable for streaming, bulk loading, and diff-friendly storage.

use crate::model::{IndividualValue, Ontology, PropertyKind};

const OWL_CLASS: &str = "http://www.w3.org/2002/07/owl#Class";
const OWL_ONTOLOGY: &str = "http://www.w3.org/2002/07/owl#Ontology";
const OWL_DATATYPE_PROPERTY: &str = "http://www.w3.org/2002/07/owl#DatatypeProperty";
const OWL_OBJECT_PROPERTY: &str = "http://www.w3.org/2002/07/owl#ObjectProperty";
const OWL_ANNOTATION_PROPERTY: &str = "http://www.w3.org/2002/07/owl#AnnotationProperty";
const OWL_FUNCTIONAL_PROPERTY: &str = "http://www.w3.org/2002/07/owl#FunctionalProperty";
const OWL_NAMED_INDIVIDUAL: &str = "http://www.w3.org/2002/07/owl#NamedIndividual";
const OWL_DISJOINT_WITH: &str = "http://www.w3.org/2002/07/owl#disjointWith";
const OWL_IMPORTS: &str = "http://www.w3.org/2002/07/owl#imports";
const OWL_VERSION_INFO: &str = "http://www.w3.org/2002/07/owl#versionInfo";
const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
const RDFS_COMMENT: &str = "http://www.w3.org/2000/01/rdf-schema#comment";
const RDFS_SUBCLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const RDFS_DOMAIN: &str = "http://www.w3.org/2000/01/rdf-schema#domain";
const RDFS_RANGE: &str = "http://www.w3.org/2000/01/rdf-schema#range";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";
const XSD_DECIMAL: &str = "http://www.w3.org/2001/XMLSchema#decimal";
const UOR_SPACE: &str = "https://uor.foundation/space";

/// Serializes the complete UOR Foundation ontology to an N-Triples string.
///
/// # Errors
///
/// This function is infallible; it always returns a valid N-Triples string.
#[must_use]
pub fn to_ntriples(ontology: &Ontology) -> String {
    let mut out = String::with_capacity(256 * 1024);
    let mut bnode_counter: usize = 0;

    // Root ontology
    triple(&mut out, ontology.base_iri, RDF_TYPE, &iri(OWL_ONTOLOGY));
    triple(
        &mut out,
        ontology.base_iri,
        OWL_VERSION_INFO,
        &lit(ontology.version, XSD_STRING),
    );

    // Annotation properties
    for ap in &ontology.annotation_properties {
        triple(&mut out, ap.id, RDF_TYPE, &iri(OWL_ANNOTATION_PROPERTY));
        triple(&mut out, ap.id, RDFS_LABEL, &lit(ap.label, XSD_STRING));
        triple(&mut out, ap.id, RDFS_COMMENT, &lit(ap.comment, XSD_STRING));
        triple(&mut out, ap.id, RDFS_RANGE, &iri(ap.range));
    }

    for module in &ontology.namespaces {
        // Namespace ontology declaration
        triple(&mut out, module.namespace.iri, RDF_TYPE, &iri(OWL_ONTOLOGY));
        triple(
            &mut out,
            module.namespace.iri,
            RDFS_LABEL,
            &lit(module.namespace.label, XSD_STRING),
        );
        triple(
            &mut out,
            module.namespace.iri,
            RDFS_COMMENT,
            &lit(module.namespace.comment, XSD_STRING),
        );
        triple(
            &mut out,
            module.namespace.iri,
            UOR_SPACE,
            &lit(module.namespace.space.as_str(), XSD_STRING),
        );
        for import in module.namespace.imports {
            triple(&mut out, module.namespace.iri, OWL_IMPORTS, &iri(import));
        }

        // Classes
        for class in &module.classes {
            triple(&mut out, class.id, RDF_TYPE, &iri(OWL_CLASS));
            triple(
                &mut out,
                class.id,
                RDFS_LABEL,
                &lit(class.label, XSD_STRING),
            );
            triple(
                &mut out,
                class.id,
                RDFS_COMMENT,
                &lit(class.comment, XSD_STRING),
            );
            for parent in class.subclass_of {
                triple(&mut out, class.id, RDFS_SUBCLASS_OF, &iri(parent));
            }
            for other in class.disjoint_with {
                triple(&mut out, class.id, OWL_DISJOINT_WITH, &iri(other));
            }
        }

        // Properties
        for prop in &module.properties {
            let type_iri = match prop.kind {
                PropertyKind::Datatype => OWL_DATATYPE_PROPERTY,
                PropertyKind::Object => OWL_OBJECT_PROPERTY,
                PropertyKind::Annotation => OWL_ANNOTATION_PROPERTY,
            };
            triple(&mut out, prop.id, RDF_TYPE, &iri(type_iri));
            if prop.functional {
                triple(&mut out, prop.id, RDF_TYPE, &iri(OWL_FUNCTIONAL_PROPERTY));
            }
            triple(&mut out, prop.id, RDFS_LABEL, &lit(prop.label, XSD_STRING));
            triple(
                &mut out,
                prop.id,
                RDFS_COMMENT,
                &lit(prop.comment, XSD_STRING),
            );
            if let Some(domain) = prop.domain {
                triple(&mut out, prop.id, RDFS_DOMAIN, &iri(domain));
            }
            triple(&mut out, prop.id, RDFS_RANGE, &iri(prop.range));
        }

        // Individuals
        for ind in &module.individuals {
            triple(&mut out, ind.id, RDF_TYPE, &iri(OWL_NAMED_INDIVIDUAL));
            triple(&mut out, ind.id, RDF_TYPE, &iri(ind.type_));
            triple(&mut out, ind.id, RDFS_LABEL, &lit(ind.label, XSD_STRING));
            triple(
                &mut out,
                ind.id,
                RDFS_COMMENT,
                &lit(ind.comment, XSD_STRING),
            );
            for (prop_iri, value) in ind.properties {
                if let IndividualValue::List(items) = value {
                    emit_rdf_list(&mut out, ind.id, prop_iri, items, &mut bnode_counter);
                } else {
                    let obj = individual_value_to_object(value);
                    triple(&mut out, ind.id, prop_iri, &obj);
                }
            }
        }
    }

    out
}

fn triple(out: &mut String, subj: &str, pred: &str, obj: &str) {
    out.push('<');
    out.push_str(subj);
    out.push_str("> <");
    out.push_str(pred);
    out.push_str("> ");
    out.push_str(obj);
    out.push_str(" .\n");
}

fn iri(s: &str) -> String {
    format!("<{}>", s)
}

fn lit(s: &str, datatype: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{}\"^^<{}>", escaped, datatype)
}

fn individual_value_to_object(value: &IndividualValue) -> String {
    match value {
        IndividualValue::Str(s) => lit(s, XSD_STRING),
        IndividualValue::Int(i) => format!("\"{}\"^^<{}>", i, XSD_INTEGER),
        IndividualValue::Bool(b) => format!("\"{}\"^^<{}>", b, XSD_BOOLEAN),
        IndividualValue::Float(x) => format!("\"{}\"^^<{}>", x, XSD_DECIMAL),
        IndividualValue::IriRef(iri_ref) => iri(iri_ref),
        // Lists are handled by emit_rdf_list at the call site.
        IndividualValue::List(_) => iri(RDF_NIL),
    }
}

/// Emits a proper `rdf:List` blank-node chain in N-Triples format.
///
/// For a list `[A, B]` linked from `<subj> <pred>`, this produces:
/// ```text
/// <subj> <pred> _:list0 .
/// _:list0 <rdf:first> <A> .
/// _:list0 <rdf:rest> _:list1 .
/// _:list1 <rdf:first> <B> .
/// _:list1 <rdf:rest> <rdf:nil> .
/// ```
fn emit_rdf_list(
    out: &mut String,
    subject: &str,
    predicate: &str,
    items: &[&str],
    counter: &mut usize,
) {
    if items.is_empty() {
        triple(out, subject, predicate, &iri(RDF_NIL));
        return;
    }

    let head_label = format!("_:list{}", *counter);
    *counter += 1;
    // Link subject to the head of the list
    triple(out, subject, predicate, &head_label);

    let mut current_label = head_label;
    for (i, item) in items.iter().enumerate() {
        // rdf:first
        bnode_triple(out, &current_label, RDF_FIRST, &iri(item));
        // rdf:rest
        if i + 1 < items.len() {
            let next_label = format!("_:list{}", *counter);
            *counter += 1;
            bnode_triple(out, &current_label, RDF_REST, &next_label);
            current_label = next_label;
        } else {
            bnode_triple(out, &current_label, RDF_REST, &iri(RDF_NIL));
        }
    }
}

/// Writes a triple where the subject is a blank node label.
fn bnode_triple(out: &mut String, bnode: &str, pred: &str, obj: &str) {
    out.push_str(bnode);
    out.push_str(" <");
    out.push_str(pred);
    out.push_str("> ");
    out.push_str(obj);
    out.push_str(" .\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ontology;

    #[test]
    fn produces_non_empty_ntriples() {
        let ontology = Ontology::full();
        let nt = to_ntriples(ontology);
        assert!(!nt.is_empty());
    }

    #[test]
    fn every_line_ends_with_period() {
        let ontology = Ontology::full();
        let nt = to_ntriples(ontology);
        for line in nt.lines() {
            if !line.is_empty() {
                assert!(line.ends_with(" ."), "Line does not end with ' .': {line}");
            }
        }
    }

    #[test]
    fn contains_owl_class_declarations() {
        let ontology = Ontology::full();
        let nt = to_ntriples(ontology);
        assert!(nt.contains(&format!("<{}>", OWL_CLASS)));
    }

    #[test]
    fn lists_produce_rdf_first_rest() {
        let ontology = Ontology::full();
        let nt = to_ntriples(ontology);
        // 2 `op:composedOf` `IndividualValue::List` assertions remain
        // in the ontology (op:succ and op:pred), each listing two
        // operations. Each 2-element list produces 2 rdf:first and
        // 2 rdf:rest triples → 4 of each minimum.
        //
        // `op:criticalIdentity`'s previous `rhs = List` assertion was
        // removed in Amendment 96 (the assertion was
        // type-incompatible with the `schema:TermExpression` field
        // declared on `op:Identity.rhs`; see
        // `model::rewrite_identity_ast_refs`).
        let rdf_first_count = nt.matches(RDF_FIRST).count();
        let rdf_rest_count = nt.matches(RDF_REST).count();
        assert!(
            rdf_first_count >= 4,
            "Expected at least 4 rdf:first triples, got {}",
            rdf_first_count
        );
        assert_eq!(
            rdf_first_count, rdf_rest_count,
            "rdf:first count ({}) should equal rdf:rest count ({})",
            rdf_first_count, rdf_rest_count
        );
    }
}
