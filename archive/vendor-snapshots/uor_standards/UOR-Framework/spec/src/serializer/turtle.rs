//! Turtle 1.1 serializer for the UOR Foundation ontology.
//!
//! Produces a valid Turtle document containing all namespace declarations,
//! class definitions, property definitions, and named individuals.

use crate::model::{IndividualValue, Ontology, PropertyKind};
use crate::serializer::prefixes::STANDARD_PREFIXES;

/// Serializes the complete UOR Foundation ontology to a Turtle string.
///
/// # Errors
///
/// This function is infallible; it always returns a valid Turtle string.
#[must_use]
pub fn to_turtle(ontology: &Ontology) -> String {
    let mut out = String::with_capacity(128 * 1024);

    // Prefix declarations (aligned to a 6-character column for readability).
    for (prefix, iri) in STANDARD_PREFIXES {
        let head = format!("{prefix}:");
        out.push_str(&format!("@prefix {head:<6}<{iri}> .\n"));
    }

    for module in &ontology.namespaces {
        out.push_str(&format!(
            "@prefix {}:  <{}> .\n",
            module.namespace.prefix, module.namespace.iri
        ));
    }
    out.push('\n');

    // Root ontology
    out.push_str(&format!(
        "<{}>\n  a owl:Ontology ;\n  rdfs:label \"UOR Foundation\" ;\n  owl:versionInfo \"{}\" .\n\n",
        ontology.base_iri, ontology.version
    ));

    // Annotation properties
    for ap in &ontology.annotation_properties {
        out.push_str(&format!(
            "<{}>\n  a owl:AnnotationProperty ;\n  rdfs:label {} ;\n  rdfs:comment {} ;\n  rdfs:range <{}> .\n\n",
            ap.id,
            turtle_string(ap.label),
            turtle_string(ap.comment),
            ap.range
        ));
    }

    // Namespace modules
    for module in &ontology.namespaces {
        out.push_str(&format!("# Namespace: {}\n", module.namespace.prefix));

        // Namespace ontology declaration
        let imports_str: String = module
            .namespace
            .imports
            .iter()
            .map(|iri| format!("  owl:imports <{}> ;\n", iri))
            .collect();
        out.push_str(&format!(
            "<{}>\n  a owl:Ontology ;\n  rdfs:label {} ;\n  rdfs:comment {} ;\n  uor:space \"{}\" ;\n{}.\n\n",
            module.namespace.iri,
            turtle_string(module.namespace.label),
            turtle_string(module.namespace.comment),
            module.namespace.space.as_str(),
            imports_str
        ));

        // Classes
        for class in &module.classes {
            let subclasses: String = class
                .subclass_of
                .iter()
                .map(|iri| format!("  rdfs:subClassOf <{}> ;\n", iri))
                .collect();
            let disjoints: String = class
                .disjoint_with
                .iter()
                .map(|iri| format!("  owl:disjointWith <{}> ;\n", iri))
                .collect();
            out.push_str(&format!(
                "<{}>\n  a owl:Class ;\n  rdfs:label {} ;\n  rdfs:comment {} ;\n{}{}.\n\n",
                class.id,
                turtle_string(class.label),
                turtle_string(class.comment),
                subclasses,
                disjoints
            ));
        }

        // Properties
        for prop in &module.properties {
            let type_str = match prop.kind {
                PropertyKind::Datatype if prop.functional => {
                    "owl:DatatypeProperty , owl:FunctionalProperty"
                }
                PropertyKind::Datatype => "owl:DatatypeProperty",
                PropertyKind::Object if prop.functional => {
                    "owl:ObjectProperty , owl:FunctionalProperty"
                }
                PropertyKind::Object => "owl:ObjectProperty",
                PropertyKind::Annotation => "owl:AnnotationProperty",
            };
            let domain_str = prop
                .domain
                .map(|d| format!("  rdfs:domain <{}> ;\n", d))
                .unwrap_or_default();
            out.push_str(&format!(
                "<{}>\n  a {} ;\n  rdfs:label {} ;\n  rdfs:comment {} ;\n{}  rdfs:range <{}> .\n\n",
                prop.id,
                type_str,
                turtle_string(prop.label),
                turtle_string(prop.comment),
                domain_str,
                prop.range
            ));
        }

        // Individuals
        for ind in &module.individuals {
            let mut ind_str = format!(
                "<{}>\n  a owl:NamedIndividual , <{}> ;\n  rdfs:label {} ;\n  rdfs:comment {}",
                ind.id,
                ind.type_,
                turtle_string(ind.label),
                turtle_string(ind.comment)
            );
            for (prop_iri, value) in ind.properties {
                ind_str.push_str(&format!(
                    " ;\n  <{}> {}",
                    prop_iri,
                    individual_value_to_turtle(value)
                ));
            }
            ind_str.push_str(" .\n\n");
            out.push_str(&ind_str);
        }
    }

    out
}

fn turtle_string(s: &str) -> String {
    // Use triple-quoted strings for multi-line content; escape quotes otherwise.
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

fn individual_value_to_turtle(value: &IndividualValue) -> String {
    match value {
        IndividualValue::Str(s) => turtle_string(s),
        IndividualValue::Int(i) => format!("\"{}\"^^xsd:integer", i),
        IndividualValue::Bool(b) => format!("\"{}\"^^xsd:boolean", b),
        IndividualValue::Float(x) => format!("\"{}\"^^xsd:decimal", x),
        IndividualValue::IriRef(iri) => format!("<{}>", iri),
        IndividualValue::List(items) => {
            // Encode as rdf:List
            let mut result = "( ".to_owned();
            for item in *items {
                result.push_str(&format!("<{}> ", item));
            }
            result.push(')');
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ontology;

    #[test]
    fn produces_non_empty_turtle() {
        let ontology = Ontology::full();
        let turtle = to_turtle(ontology);
        assert!(!turtle.is_empty());
        assert!(turtle.contains("@prefix owl:"));
        assert!(turtle.contains("owl:Ontology"));
    }

    #[test]
    fn contains_all_namespace_prefixes() {
        let ontology = Ontology::full();
        let turtle = to_turtle(ontology);
        for module in &ontology.namespaces {
            assert!(
                turtle.contains(&format!("@prefix {}:", module.namespace.prefix)),
                "Missing prefix declaration for '{}'",
                module.namespace.prefix
            );
        }
    }

    #[test]
    fn contains_critical_identity_individual() {
        let ontology = Ontology::full();
        let turtle = to_turtle(ontology);
        assert!(
            turtle.contains("https://uor.foundation/op/criticalIdentity"),
            "Missing criticalIdentity individual in Turtle output"
        );
    }

    #[test]
    fn contains_amendment_95_terms() {
        let ontology = Ontology::full();
        let turtle = to_turtle(ontology);
        assert!(
            turtle.contains("SurfaceSymbol"),
            "Missing SurfaceSymbol class"
        );
        assert!(
            turtle.contains("HammingConstraint"),
            "Missing HammingConstraint class"
        );
        assert!(
            turtle.contains("IntegerGroundingMap"),
            "Missing IntegerGroundingMap individual"
        );
        assert!(
            turtle.contains("predicate/always"),
            "Missing predicate:always individual"
        );
    }
}
