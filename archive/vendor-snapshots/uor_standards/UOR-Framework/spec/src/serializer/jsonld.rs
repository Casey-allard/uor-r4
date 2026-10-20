//! JSON-LD 1.1 serializer for the UOR Foundation ontology.
//!
//! Produces a single JSON-LD document containing the complete `@context`
//! and `@graph` array with all namespace modules in dependency order.
//! The output matches the structure of the reference `uor.foundation.jsonld`.

use serde_json::{json, Map, Value};

use crate::model::{IndividualValue, Ontology, PropertyKind};
use crate::serializer::prefixes::STANDARD_PREFIXES;

/// Serializes the complete UOR Foundation ontology to a JSON-LD `Value`.
///
/// The returned value can be pretty-printed with [`serde_json::to_string_pretty`].
///
/// # Errors
///
/// This function is infallible; it always returns a valid JSON-LD `Value`.
#[must_use]
pub fn to_json_ld(ontology: &Ontology) -> Value {
    let context = build_context(ontology);
    let graph = build_graph(ontology);
    json!({
        "@context": context,
        "@graph": graph
    })
}

fn build_context(ontology: &Ontology) -> Value {
    let mut ctx = Map::new();
    // JSON-LD 1.1 processing mode
    ctx.insert("@version".to_owned(), json!(1.1));
    // Standard semantic web prefixes
    for (prefix, iri) in STANDARD_PREFIXES {
        ctx.insert((*prefix).to_owned(), json!(*iri));
    }
    // UOR namespace prefixes
    for module in &ontology.namespaces {
        ctx.insert(
            module.namespace.prefix.to_owned(),
            json!(module.namespace.iri),
        );
    }
    Value::Object(ctx)
}

fn build_graph(ontology: &Ontology) -> Value {
    let mut nodes: Vec<Value> = Vec::new();

    // Root ontology declaration
    nodes.push(json!({
        "@id": ontology.base_iri,
        "@type": "owl:Ontology",
        "rdfs:label": "UOR Foundation",
        "rdfs:comment": "Universal Object Reference Foundation Ontology — the complete \
                         vocabulary for the UOR kernel, type system, resolution pipeline, \
                         and state model.",
        "owl:versionInfo": ontology.version
    }));

    // Annotation property (Amendment 8)
    for ap in &ontology.annotation_properties {
        nodes.push(json!({
            "@id": ap.id,
            "@type": "owl:AnnotationProperty",
            "rdfs:label": ap.label,
            "rdfs:comment": ap.comment,
            "rdfs:range": { "@id": ap.range }
        }));
    }

    // Each namespace module
    for module in &ontology.namespaces {
        // Namespace ontology declaration
        let imports: Vec<Value> = module
            .namespace
            .imports
            .iter()
            .map(|iri| json!({ "@id": iri }))
            .collect();
        let mut ns_node = json!({
            "@id": module.namespace.iri,
            "@type": "owl:Ontology",
            "rdfs:label": module.namespace.label,
            "rdfs:comment": module.namespace.comment,
            "uor:space": module.namespace.space.as_str()
        });
        if !imports.is_empty() {
            ns_node["owl:imports"] = Value::Array(imports);
        }
        nodes.push(ns_node);

        // Classes
        for class in &module.classes {
            nodes.push(class_to_json(class));
        }

        // Properties
        for prop in &module.properties {
            nodes.push(property_to_json(prop));
        }

        // Individuals
        for ind in &module.individuals {
            nodes.push(individual_to_json(ind));
        }
    }

    Value::Array(nodes)
}

fn class_to_json(class: &crate::model::Class) -> Value {
    let subclass_of: Vec<Value> = class
        .subclass_of
        .iter()
        .map(|iri| json!({ "@id": iri }))
        .collect();
    let disjoint_with: Vec<Value> = class
        .disjoint_with
        .iter()
        .map(|iri| json!({ "@id": iri }))
        .collect();

    let mut node = json!({
        "@id": class.id,
        "@type": "owl:Class",
        "rdfs:label": class.label,
        "rdfs:comment": class.comment,
        "rdfs:subClassOf": subclass_of
    });
    if !disjoint_with.is_empty() {
        node["owl:disjointWith"] = Value::Array(disjoint_with);
    }
    node
}

fn property_to_json(prop: &crate::model::Property) -> Value {
    let type_ = match prop.kind {
        PropertyKind::Datatype => {
            if prop.functional {
                json!(["owl:DatatypeProperty", "owl:FunctionalProperty"])
            } else {
                json!("owl:DatatypeProperty")
            }
        }
        PropertyKind::Object => {
            if prop.functional {
                json!(["owl:ObjectProperty", "owl:FunctionalProperty"])
            } else {
                json!("owl:ObjectProperty")
            }
        }
        PropertyKind::Annotation => json!("owl:AnnotationProperty"),
    };

    let mut node = json!({
        "@id": prop.id,
        "@type": type_,
        "rdfs:label": prop.label,
        "rdfs:comment": prop.comment,
        "rdfs:range": { "@id": prop.range }
    });
    if let Some(domain) = prop.domain {
        node["rdfs:domain"] = json!({ "@id": domain });
    }
    node
}

fn individual_to_json(ind: &crate::model::Individual) -> Value {
    let mut node = json!({
        "@id": ind.id,
        "@type": ["owl:NamedIndividual", ind.type_],
        "rdfs:label": ind.label,
        "rdfs:comment": ind.comment
    });

    for (prop_iri, value) in ind.properties {
        let json_value = match value {
            IndividualValue::Str(s) => json!(s),
            IndividualValue::Int(i) => json!(i),
            IndividualValue::Bool(b) => json!(b),
            IndividualValue::Float(x) => json!({
                "@value": x.to_string(),
                "@type": "xsd:decimal"
            }),
            IndividualValue::IriRef(iri) => json!({ "@id": iri }),
            IndividualValue::List(items) => {
                // Encode as rdf:List structure
                json!({
                    "@type": "rdf:List",
                    "rdf:first": { "@id": items.first().copied().unwrap_or("") },
                    "rdf:rest": build_rdf_list_rest(&items[1..])
                })
            }
        };
        // Use the prefixed form of the IRI as the JSON-LD key.
        // When the same property key appears more than once (non-functional property),
        // promote to a JSON-LD array rather than overwriting.
        let key = shorten_iri(prop_iri);
        if let Value::Object(ref mut map) = node {
            if let Some(existing) = map.get_mut(&key) {
                let prev = std::mem::replace(existing, Value::Null);
                *existing = match prev {
                    Value::Array(mut arr) => {
                        arr.push(json_value);
                        Value::Array(arr)
                    }
                    other => json!([other, json_value]),
                };
            } else {
                map.insert(key, json_value);
            }
        }
    }

    node
}

/// Shortens a full IRI to a prefixed form using known UOR prefixes.
fn shorten_iri(iri: &str) -> String {
    const PREFIXES: &[(&str, &str)] = &[
        ("https://uor.foundation/op/", "op:"),
        ("https://uor.foundation/schema/", "schema:"),
        ("https://uor.foundation/proof/", "proof:"),
        ("https://uor.foundation/trace/", "trace:"),
        ("https://uor.foundation/cert/", "cert:"),
        ("https://uor.foundation/observable/", "observable:"),
        ("https://uor.foundation/morphism/", "morphism:"),
        ("https://uor.foundation/state/", "state:"),
        ("https://uor.foundation/partition/", "partition:"),
        ("https://uor.foundation/derivation/", "derivation:"),
        ("https://uor.foundation/resolver/", "resolver:"),
        ("https://uor.foundation/type/", "type:"),
        ("https://uor.foundation/query/", "query:"),
        ("https://uor.foundation/u/", "u:"),
        ("https://uor.foundation/homology/", "homology:"),
        ("https://uor.foundation/cohomology/", "cohomology:"),
        ("http://www.w3.org/2002/07/owl#", "owl:"),
        ("http://www.w3.org/1999/02/22-rdf-syntax-ns#", "rdf:"),
        ("http://www.w3.org/2000/01/rdf-schema#", "rdfs:"),
    ];
    for (full, prefix) in PREFIXES {
        if let Some(local) = iri.strip_prefix(full) {
            return format!("{prefix}{local}");
        }
    }
    iri.to_owned()
}

fn build_rdf_list_rest(items: &[&str]) -> Value {
    if items.is_empty() {
        return json!({ "@id": "rdf:nil" });
    }
    json!({
        "@type": "rdf:List",
        "rdf:first": { "@id": items[0] },
        "rdf:rest": build_rdf_list_rest(&items[1..])
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::Ontology;

    #[test]
    fn produces_context_and_graph() {
        let ontology = Ontology::full();
        let json = to_json_ld(ontology);
        assert!(json["@context"].is_object());
        assert!(json["@graph"].is_array());
    }

    #[test]
    fn context_has_all_namespace_prefixes() {
        let ontology = Ontology::full();
        let json = to_json_ld(ontology);
        let ctx = &json["@context"];
        for module in &ontology.namespaces {
            assert!(
                !ctx[module.namespace.prefix].is_null(),
                "Missing prefix '{}' in @context",
                module.namespace.prefix
            );
        }
    }

    #[test]
    fn graph_is_non_empty() {
        let ontology = Ontology::full();
        let json = to_json_ld(ontology);
        let graph = json["@graph"].as_array().expect("@graph must be array");
        // root ontology + annotation property + 16 ns declarations +
        // 123 classes + 229 properties + 269 individuals >= 639
        assert!(graph.len() >= 639, "graph has {} nodes", graph.len());
    }

    #[test]
    fn context_has_version_1_1() {
        let ontology = Ontology::full();
        let json = to_json_ld(ontology);
        let version = json["@context"]["@version"]
            .as_f64()
            .expect("@version must be a number");
        assert!(
            (version - 1.1).abs() < f64::EPSILON,
            "@version should be 1.1, got {}",
            version
        );
    }

    #[test]
    fn all_nodes_have_ids() {
        let ontology = Ontology::full();
        let json = to_json_ld(ontology);
        let graph = json["@graph"].as_array().expect("@graph must be array");
        for (i, node) in graph.iter().enumerate() {
            assert!(!node["@id"].is_null(), "Node at index {i} is missing @id");
        }
    }

    #[test]
    fn contains_amendment_95_terms() {
        let ontology = Ontology::full();
        let json = to_json_ld(ontology);
        let text = json.to_string();
        assert!(text.contains("SurfaceSymbol"), "Missing SurfaceSymbol");
        assert!(text.contains("always"), "Missing predicate:always");
        assert!(
            text.contains("IntegerGroundingMap"),
            "Missing IntegerGroundingMap"
        );
    }
}
