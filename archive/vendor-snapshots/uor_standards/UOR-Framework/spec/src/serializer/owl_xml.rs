//! OWL 2 RDF/XML serializer for the UOR Foundation ontology.
//!
//! Produces a valid RDF/XML document containing all namespace declarations,
//! class definitions, property definitions, and named individuals in OWL 2
//! RDF/XML syntax — the standard interchange format for Protégé, OWL
//! reasoners, and TopBraid.

use crate::model::{IndividualValue, Ontology, PropertyKind};

/// Standard prefix entries: (prefix, namespace IRI).
const STANDARD_PREFIXES: &[(&str, &str)] = &[
    ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
    ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
    ("owl", "http://www.w3.org/2002/07/owl#"),
    ("xsd", "http://www.w3.org/2001/XMLSchema#"),
    ("sh", "http://www.w3.org/ns/shacl#"),
    ("uor", "https://uor.foundation/"),
];

/// Serializes the complete UOR Foundation ontology to an OWL 2 RDF/XML string.
///
/// # Errors
///
/// This function is infallible; it always returns a valid RDF/XML string.
#[must_use]
pub fn to_owl_xml(ontology: &Ontology) -> String {
    let mut out = String::with_capacity(256 * 1024);

    // Build the prefix map for IRI shortening (namespace prefix → IRI).
    let prefix_map: Vec<(&str, &str)> = build_prefix_map(ontology);

    // XML declaration
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

    // rdf:RDF root with namespace declarations
    out.push_str("<rdf:RDF\n");
    for (prefix, iri) in &prefix_map {
        out.push_str(&format!("    xmlns:{}=\"{}\"\n", prefix, iri));
    }
    out.push_str(">\n\n");

    // Root ontology declaration
    out.push_str(&format!(
        "  <owl:Ontology rdf:about=\"{}\">\n\
         \x20   <rdfs:label>UOR Foundation</rdfs:label>\n\
         \x20   <owl:versionInfo>{}</owl:versionInfo>\n\
         \x20 </owl:Ontology>\n\n",
        xml_escape(ontology.base_iri),
        xml_escape(ontology.version)
    ));

    // Annotation properties
    for ap in &ontology.annotation_properties {
        out.push_str(&format!(
            "  <owl:AnnotationProperty rdf:about=\"{}\">\n\
             \x20   <rdfs:label>{}</rdfs:label>\n\
             \x20   <rdfs:comment>{}</rdfs:comment>\n\
             \x20   <rdfs:range rdf:resource=\"{}\"/>\n\
             \x20 </owl:AnnotationProperty>\n\n",
            xml_escape(ap.id),
            xml_escape(ap.label),
            xml_escape(ap.comment),
            xml_escape(ap.range),
        ));
    }

    // Namespace modules
    for module in &ontology.namespaces {
        out.push_str(&format!(
            "  <!-- Namespace: {} -->\n",
            module.namespace.prefix
        ));

        // Namespace ontology declaration
        out.push_str(&format!(
            "  <owl:Ontology rdf:about=\"{}\">\n\
             \x20   <rdfs:label>{}</rdfs:label>\n\
             \x20   <rdfs:comment>{}</rdfs:comment>\n\
             \x20   <uor:space>{}</uor:space>\n",
            xml_escape(module.namespace.iri),
            xml_escape(module.namespace.label),
            xml_escape(module.namespace.comment),
            module.namespace.space.as_str(),
        ));
        for import in module.namespace.imports {
            out.push_str(&format!(
                "    <owl:imports rdf:resource=\"{}\"/>\n",
                xml_escape(import)
            ));
        }
        out.push_str("  </owl:Ontology>\n\n");

        // Classes
        for class in &module.classes {
            out.push_str(&format!(
                "  <owl:Class rdf:about=\"{}\">\n\
                 \x20   <rdfs:label>{}</rdfs:label>\n\
                 \x20   <rdfs:comment>{}</rdfs:comment>\n",
                xml_escape(class.id),
                xml_escape(class.label),
                xml_escape(class.comment),
            ));
            for parent in class.subclass_of {
                out.push_str(&format!(
                    "    <rdfs:subClassOf rdf:resource=\"{}\"/>\n",
                    xml_escape(parent)
                ));
            }
            for other in class.disjoint_with {
                out.push_str(&format!(
                    "    <owl:disjointWith rdf:resource=\"{}\"/>\n",
                    xml_escape(other)
                ));
            }
            out.push_str("  </owl:Class>\n\n");
        }

        // Properties
        for prop in &module.properties {
            let elem = match prop.kind {
                PropertyKind::Datatype => "owl:DatatypeProperty",
                PropertyKind::Object => "owl:ObjectProperty",
                PropertyKind::Annotation => "owl:AnnotationProperty",
            };
            out.push_str(&format!(
                "  <{} rdf:about=\"{}\">\n",
                elem,
                xml_escape(prop.id)
            ));
            if prop.functional && !matches!(prop.kind, PropertyKind::Annotation) {
                out.push_str(
                    "    <rdf:type rdf:resource=\"http://www.w3.org/2002/07/owl#FunctionalProperty\"/>\n",
                );
            }
            out.push_str(&format!(
                "    <rdfs:label>{}</rdfs:label>\n\
                 \x20   <rdfs:comment>{}</rdfs:comment>\n",
                xml_escape(prop.label),
                xml_escape(prop.comment),
            ));
            if let Some(domain) = prop.domain {
                out.push_str(&format!(
                    "    <rdfs:domain rdf:resource=\"{}\"/>\n",
                    xml_escape(domain)
                ));
            }
            out.push_str(&format!(
                "    <rdfs:range rdf:resource=\"{}\"/>\n",
                xml_escape(prop.range)
            ));
            out.push_str(&format!("  </{}>\n\n", elem));
        }

        // Individuals
        for ind in &module.individuals {
            out.push_str(&format!(
                "  <owl:NamedIndividual rdf:about=\"{}\">\n\
                 \x20   <rdf:type rdf:resource=\"{}\"/>\n\
                 \x20   <rdfs:label>{}</rdfs:label>\n\
                 \x20   <rdfs:comment>{}</rdfs:comment>\n",
                xml_escape(ind.id),
                xml_escape(ind.type_),
                xml_escape(ind.label),
                xml_escape(ind.comment),
            ));
            for (prop_iri, value) in ind.properties {
                let elem_name = shorten_iri(prop_iri, &prefix_map);
                out.push_str(&individual_value_to_xml(&elem_name, value));
            }
            out.push_str("  </owl:NamedIndividual>\n\n");
        }
    }

    out.push_str("</rdf:RDF>\n");
    out
}

/// Builds the full prefix map from standard prefixes + ontology namespaces.
fn build_prefix_map(ontology: &Ontology) -> Vec<(&str, &str)> {
    let mut map: Vec<(&str, &str)> = STANDARD_PREFIXES.to_vec();
    for module in &ontology.namespaces {
        map.push((module.namespace.prefix, module.namespace.iri));
    }
    map
}

/// Shortens a full IRI to a prefixed form using the given prefix map.
fn shorten_iri(iri: &str, prefix_map: &[(&str, &str)]) -> String {
    for (prefix, ns_iri) in prefix_map {
        if let Some(local) = iri.strip_prefix(ns_iri) {
            return format!("{prefix}:{local}");
        }
    }
    // Fallback: use the full IRI (should not happen in practice).
    iri.to_owned()
}

/// Escapes special XML characters in a string.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Formats an individual property assertion as an XML child element.
fn individual_value_to_xml(elem_name: &str, value: &IndividualValue) -> String {
    match value {
        IndividualValue::Str(s) => {
            format!("    <{}>{}</{}>\n", elem_name, xml_escape(s), elem_name)
        }
        IndividualValue::Int(i) => {
            format!(
                "    <{} rdf:datatype=\"http://www.w3.org/2001/XMLSchema#integer\"\
                 >{}</{}>\n",
                elem_name, i, elem_name
            )
        }
        IndividualValue::Bool(b) => {
            format!(
                "    <{} rdf:datatype=\"http://www.w3.org/2001/XMLSchema#boolean\"\
                 >{}</{}>\n",
                elem_name, b, elem_name
            )
        }
        IndividualValue::Float(x) => {
            format!(
                "    <{} rdf:datatype=\"http://www.w3.org/2001/XMLSchema#decimal\"\
                 >{}</{}>\n",
                elem_name, x, elem_name
            )
        }
        IndividualValue::IriRef(iri) => {
            format!(
                "    <{} rdf:resource=\"{}\"/>\n",
                elem_name,
                xml_escape(iri)
            )
        }
        IndividualValue::List(items) => {
            let mut out = format!("    <{} rdf:parseType=\"Collection\">\n", elem_name);
            for item in *items {
                out.push_str(&format!(
                    "      <rdf:Description rdf:about=\"{}\"/>\n",
                    xml_escape(item)
                ));
            }
            out.push_str(&format!("    </{}>\n", elem_name));
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ontology;

    #[test]
    fn produces_valid_xml_structure() {
        let ontology = Ontology::full();
        let xml = to_owl_xml(ontology);
        assert!(xml.starts_with("<?xml version=\"1.0\""));
        assert!(xml.contains("<rdf:RDF"));
        assert!(xml.ends_with("</rdf:RDF>\n"));
    }

    #[test]
    fn contains_all_namespace_declarations() {
        let ontology = Ontology::full();
        let xml = to_owl_xml(ontology);
        for module in &ontology.namespaces {
            assert!(
                xml.contains(&format!(
                    "xmlns:{}=\"{}\"",
                    module.namespace.prefix, module.namespace.iri
                )),
                "Missing xmlns declaration for '{}'",
                module.namespace.prefix
            );
        }
    }

    #[test]
    fn contains_owl_class_declarations() {
        let ontology = Ontology::full();
        let xml = to_owl_xml(ontology);
        assert!(xml.contains("<owl:Class"), "Missing owl:Class declarations");
    }

    #[test]
    fn contains_named_individuals() {
        let ontology = Ontology::full();
        let xml = to_owl_xml(ontology);
        assert!(
            xml.contains("<owl:NamedIndividual"),
            "Missing owl:NamedIndividual declarations"
        );
    }

    #[test]
    fn contains_version_info() {
        let ontology = Ontology::full();
        let xml = to_owl_xml(ontology);
        assert!(
            xml.contains(&format!(
                "<owl:versionInfo>{}</owl:versionInfo>",
                ontology.version
            )),
            "Missing version info"
        );
    }

    #[test]
    fn xml_escape_handles_special_chars() {
        assert_eq!(
            xml_escape("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }
}
