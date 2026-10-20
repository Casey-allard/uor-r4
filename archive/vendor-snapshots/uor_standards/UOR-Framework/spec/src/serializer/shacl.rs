//! SHACL shapes serializer for the UOR Foundation ontology.
//!
//! Produces a valid Turtle document containing one `sh:NodeShape` per class,
//! with property constraints derived from the ontology model. Functional
//! properties receive `sh:minCount 1 ; sh:maxCount 1`; datatype properties
//! receive `sh:datatype`; object properties receive `sh:class`.
//!
//! This is a generated baseline. The hand-written shapes in
//! `conformance/shapes/uor-shapes.ttl` contain richer constraints (e.g.,
//! `sh:hasValue`, custom cardinality) and remain the conformance reference.

use crate::model::{Ontology, PropertyKind};

/// Prefix entries used in the shapes output: (prefix, IRI).
const STANDARD_PREFIXES: &[(&str, &str)] = &[
    ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
    ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
    ("owl", "http://www.w3.org/2002/07/owl#"),
    ("xsd", "http://www.w3.org/2001/XMLSchema#"),
    ("sh", "http://www.w3.org/ns/shacl#"),
];

/// Serializes SHACL validation shapes for all classes in the ontology.
///
/// # Errors
///
/// This function is infallible; it always returns a valid Turtle string.
#[must_use]
pub fn to_shacl(ontology: &Ontology) -> String {
    let mut out = String::with_capacity(64 * 1024);

    // Build prefix map for IRI shortening.
    let prefix_map: Vec<(&str, &str)> = build_prefix_map(ontology);

    // Header
    out.push_str(&format!(
        "# UOR Foundation SHACL Shapes\n\
         # Generated from ontology v{} \u{2014} {} NodeShapes.\n\n",
        ontology.version,
        ontology.class_count()
    ));

    // Prefix declarations
    for (prefix, iri) in &prefix_map {
        out.push_str(&format!(
            "@prefix {:<12}<{}> .\n",
            format!("{}:", prefix),
            iri
        ));
    }
    out.push_str(&format!(
        "@prefix {:<12}<https://uor.foundation/shapes/> .\n",
        "uor-sh:"
    ));
    out.push('\n');

    // One NodeShape per class, grouped by namespace
    for module in &ontology.namespaces {
        let class_count = module.classes.len();
        // Namespace header comment
        out.push_str(&format!(
            "# \u{2500}\u{2500} {}/ namespace ({}) {}\n\n",
            module.namespace.prefix,
            class_count,
            "\u{2500}".repeat(50_usize.saturating_sub(module.namespace.prefix.len() + 20)),
        ));

        for class in &module.classes {
            // Collect non-annotation properties in this namespace with
            // domain matching this class.
            let props: Vec<_> = module
                .properties
                .iter()
                .filter(|p| {
                    p.domain == Some(class.id) && !matches!(p.kind, PropertyKind::Annotation)
                })
                .collect();

            let target = shorten_iri(class.id, &prefix_map);

            if props.is_empty() {
                // Minimal shape (no property constraints)
                out.push_str(&format!(
                    "uor-sh:{}Shape\n\
                     \x20   a               sh:NodeShape ;\n\
                     \x20   sh:targetClass  {} ;\n\
                     \x20   rdfs:label      \"{}Shape\" .\n\n",
                    class.label, target, class.label,
                ));
            } else {
                out.push_str(&format!(
                    "uor-sh:{}Shape\n\
                     \x20   a               sh:NodeShape ;\n\
                     \x20   sh:targetClass  {} ;\n\
                     \x20   rdfs:label      \"{}Shape\"",
                    class.label, target, class.label,
                ));

                for prop in &props {
                    let path = shorten_iri(prop.id, &prefix_map);
                    out.push_str(" ;\n    sh:property [\n");
                    out.push_str(&format!("        sh:path {} ;\n", path));

                    if prop.functional {
                        out.push_str(
                            "        sh:minCount 1 ;\n\
                             \x20       sh:maxCount 1 ;\n",
                        );
                    }

                    match prop.kind {
                        PropertyKind::Datatype => {
                            let dt = shorten_iri(prop.range, &prefix_map);
                            out.push_str(&format!("        sh:datatype {} ;\n", dt));
                        }
                        PropertyKind::Object => {
                            let cls = shorten_iri(prop.range, &prefix_map);
                            out.push_str(&format!("        sh:class {} ;\n", cls));
                        }
                        PropertyKind::Annotation => {}
                    }

                    out.push_str("    ]");
                }

                out.push_str(" .\n\n");
            }
        }
    }

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

/// Shortens a full IRI to a prefixed Turtle form using the given prefix map.
fn shorten_iri(iri: &str, prefix_map: &[(&str, &str)]) -> String {
    for (prefix, ns_iri) in prefix_map {
        if let Some(local) = iri.strip_prefix(ns_iri) {
            return format!("{prefix}:{local}");
        }
    }
    // Fallback: angle-bracket IRI.
    format!("<{iri}>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ontology;

    #[test]
    fn produces_non_empty_shacl() {
        let ontology = Ontology::full();
        let shacl = to_shacl(ontology);
        assert!(!shacl.is_empty());
        assert!(shacl.contains("@prefix sh:"));
        assert!(shacl.contains("sh:NodeShape"));
    }

    #[test]
    fn one_shape_per_class() {
        let ontology = Ontology::full();
        let shacl = to_shacl(ontology);
        let count = shacl.matches("sh:NodeShape").count();
        assert_eq!(
            count,
            ontology.class_count(),
            "Expected {} NodeShapes, found {}",
            ontology.class_count(),
            count
        );
    }

    #[test]
    fn contains_all_namespace_prefixes() {
        let ontology = Ontology::full();
        let shacl = to_shacl(ontology);
        for module in &ontology.namespaces {
            assert!(
                shacl.contains(&format!("@prefix {}:", module.namespace.prefix)),
                "Missing prefix declaration for '{}'",
                module.namespace.prefix
            );
        }
    }

    #[test]
    fn functional_property_has_max_count() {
        let ontology = Ontology::full();
        let shacl = to_shacl(ontology);
        // schema:ringWittLength is functional — should have maxCount 1
        assert!(
            shacl.contains("sh:path schema:ringWittLength"),
            "Missing schema:ringWittLength path"
        );
        // Check maxCount is near the ringWittLength path
        let idx = shacl.find("sh:path schema:ringWittLength").unwrap_or(0);
        let snippet = &shacl[idx..idx + 120];
        assert!(
            snippet.contains("sh:maxCount 1"),
            "ringWittLength should have sh:maxCount 1"
        );
    }

    #[test]
    fn contains_shapes_prefix() {
        let ontology = Ontology::full();
        let shacl = to_shacl(ontology);
        assert!(shacl.contains("@prefix uor-sh:"), "Missing uor-sh: prefix");
    }
}
