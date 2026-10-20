//! JSON Schema (Draft 2020-12) serializer for the UOR Foundation ontology.
//!
//! Produces a single JSON Schema document with `$defs` containing one
//! definition per class. Vocabulary enum classes are emitted as `"enum"`
//! schemas; regular classes as `"object"` schemas with typed properties.
//! Subclass relationships map to `allOf` with `$ref`.
//!
//! Class keys in `$defs` use the format `"prefix/LocalName"` (e.g.,
//! `"op/Identity"`) to avoid collisions when different namespaces share
//! the same local name.

use serde_json::{json, Map, Value};

use crate::model::iris::{
    OWL_CLASS, OWL_THING, RDF_LIST, XSD_BOOLEAN, XSD_DATETIME, XSD_DECIMAL, XSD_HEX_BINARY,
    XSD_INTEGER, XSD_NON_NEGATIVE_INTEGER, XSD_POSITIVE_INTEGER, XSD_STRING,
};
use crate::model::{Ontology, PropertyKind};

/// Serializes the complete UOR Foundation ontology to a JSON Schema `Value`.
///
/// The returned value can be pretty-printed with [`serde_json::to_string_pretty`].
///
/// # Errors
///
/// This function is infallible; it always returns a valid JSON Schema `Value`.
#[must_use]
pub fn to_json_schema(ontology: &Ontology) -> Value {
    let mut defs = Map::new();

    let enum_names = Ontology::enum_class_names();

    for module in &ontology.namespaces {
        let prefix = module.namespace.prefix;
        for class in &module.classes {
            let key = qualified_name(prefix, class.id);
            let is_enum = enum_names.contains(&class.label);

            let def = if is_enum {
                build_enum_def(class, ontology)
            } else {
                build_class_def(class, ontology)
            };
            defs.insert(key, def);
        }
    }

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://uor.foundation/schema.json",
        "title": "UOR Foundation Ontology",
        "description": format!(
            "JSON Schema for UOR Foundation v{}. {} classes, {} properties, {} individuals.",
            ontology.version,
            ontology.class_count(),
            ontology.property_count(),
            ontology.individual_count()
        ),
        "$defs": Value::Object(defs)
    })
}

/// Builds a JSON Schema definition for a vocabulary enum class.
fn build_enum_def(class: &crate::model::Class, ontology: &Ontology) -> Value {
    let values = collect_enum_values(class.id, ontology);
    let mut def = json!({
        "title": class.label,
        "description": class.comment,
    });
    if !values.is_empty() {
        def["enum"] = Value::Array(values.iter().map(|v| json!(v)).collect());
    }
    def
}

/// Builds a JSON Schema definition for a regular (non-enum) class.
fn build_class_def(class: &crate::model::Class, ontology: &Ontology) -> Value {
    // Collect all non-annotation properties with domain == this class.
    let props: Vec<_> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.properties.iter())
        .filter(|p| p.domain == Some(class.id) && !matches!(p.kind, PropertyKind::Annotation))
        .collect();

    let mut properties = Map::new();
    let mut required: Vec<Value> = Vec::new();

    for prop in &props {
        let prop_name = local_name(prop.id);
        let base_schema = range_to_schema(prop.range, ontology);
        let schema = if prop.functional {
            required.push(json!(prop_name));
            base_schema
        } else {
            json!({"type": "array", "items": base_schema})
        };
        properties.insert(prop_name.to_owned(), schema);
    }

    let obj_def = {
        let mut obj = json!({"type": "object"});
        if !properties.is_empty() {
            obj["properties"] = Value::Object(properties);
        }
        if !required.is_empty() {
            obj["required"] = Value::Array(required);
        }
        obj
    };

    // Subclass → allOf composition (skip owl:Thing parents).
    let parents: Vec<&str> = class
        .subclass_of
        .iter()
        .filter(|iri| **iri != OWL_THING)
        .copied()
        .collect();

    let core = if parents.is_empty() {
        obj_def
    } else {
        let mut all_of: Vec<Value> = parents
            .iter()
            .map(|iri| {
                json!({"$ref": format!(
                    "#/$defs/{}",
                    ref_escape(&iri_to_qualified_name(iri, ontology))
                )})
            })
            .collect();
        all_of.push(obj_def);
        json!({"allOf": Value::Array(all_of)})
    };

    let mut def = json!({
        "title": class.label,
        "description": class.comment,
    });
    // Merge core into def.
    if let (Value::Object(ref mut def_map), Value::Object(core_map)) = (&mut def, core) {
        for (k, v) in core_map {
            def_map.insert(k, v);
        }
    }
    def
}

/// Maps an OWL/XSD range IRI to a JSON Schema type definition.
fn range_to_schema(range: &str, ontology: &Ontology) -> Value {
    match range {
        XSD_STRING => json!({"type": "string"}),
        XSD_INTEGER => json!({"type": "integer"}),
        XSD_POSITIVE_INTEGER => {
            json!({"type": "integer", "minimum": 1})
        }
        XSD_NON_NEGATIVE_INTEGER => {
            json!({"type": "integer", "minimum": 0})
        }
        XSD_BOOLEAN => json!({"type": "boolean"}),
        XSD_DECIMAL => json!({"type": "number"}),
        XSD_DATETIME => {
            json!({"type": "string", "format": "date-time"})
        }
        XSD_HEX_BINARY => {
            json!({"type": "string", "pattern": "^[0-9a-fA-F]*$"})
        }
        OWL_THING | OWL_CLASS => {
            json!({"type": "string", "format": "iri"})
        }
        RDF_LIST => {
            json!({
                "type": "array",
                "items": {"type": "string", "format": "iri"}
            })
        }
        _ => {
            // UOR class IRI → $ref with qualified name
            json!({"$ref": format!(
                "#/$defs/{}",
                ref_escape(&iri_to_qualified_name(range, ontology))
            )})
        }
    }
}

/// Extracts individual labels for a vocabulary enum class.
fn collect_enum_values<'a>(class_iri: &str, ontology: &'a Ontology) -> Vec<&'a str> {
    ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|ind| ind.type_ == class_iri)
        .map(|ind| ind.label)
        .collect()
}

/// Creates a qualified `$defs` key: `"prefix/LocalName"`.
fn qualified_name(prefix: &str, iri: &str) -> String {
    format!("{}/{}", prefix, local_name(iri))
}

/// Converts a full class IRI to a qualified `$defs` key by finding its
/// namespace prefix.
fn iri_to_qualified_name(iri: &str, ontology: &Ontology) -> String {
    for module in &ontology.namespaces {
        if let Some(local) = iri.strip_prefix(module.namespace.iri) {
            return format!("{}/{}", module.namespace.prefix, local);
        }
    }
    // Fallback: just use local name.
    local_name(iri).to_owned()
}

/// Escapes a `$defs` key for use inside a JSON Pointer `$ref`.
///
/// Per RFC 6901, `/` is the token delimiter and must be escaped as `~1`.
/// The key `"op/Identity"` becomes `"op~1Identity"` in the `$ref` path
/// `"#/$defs/op~1Identity"`.
fn ref_escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// Extracts the local name (part after the last `/`) from a full IRI.
fn local_name(iri: &str) -> &str {
    iri.rsplit('/').next().unwrap_or(iri)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::Ontology;

    #[test]
    fn produces_valid_schema_structure() {
        let ontology = Ontology::full();
        let schema = to_json_schema(ontology);
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(!schema["$id"].is_null());
        assert!(!schema["$defs"].is_null());
        assert!(!schema["title"].is_null());
    }

    #[test]
    fn all_classes_in_defs() {
        let ontology = Ontology::full();
        let schema = to_json_schema(ontology);
        let defs = schema["$defs"].as_object().expect("$defs must be object");
        assert_eq!(
            defs.len(),
            ontology.class_count(),
            "Expected {} $defs entries, found {}",
            ontology.class_count(),
            defs.len()
        );
    }

    #[test]
    fn enum_classes_have_enum_keyword() {
        let ontology = Ontology::full();
        let schema = to_json_schema(ontology);
        let defs = schema["$defs"].as_object().expect("$defs must be object");
        // Enum classes use qualified keys, so look for the right
        // prefix. All enum classes have unique local names, so
        // just search by suffix.
        for name in Ontology::enum_class_names() {
            let found = defs
                .iter()
                .find(|(k, _)| k.ends_with(&format!("/{}", name)));
            let (key, entry) =
                found.unwrap_or_else(|| panic!("Missing $defs entry for enum class '{}'", name));
            assert!(
                entry.get("enum").is_some(),
                "Enum class '{}' (key '{}') missing 'enum' keyword",
                name,
                key
            );
        }
    }

    #[test]
    fn subclass_produces_all_of() {
        let ontology = Ontology::full();
        let schema = to_json_schema(ontology);
        // TransformCertificate subclasses cert:Certificate (non-Thing)
        let tc = &schema["$defs"]["cert/TransformCertificate"];
        assert!(!tc.is_null(), "Missing cert/TransformCertificate");
        assert!(
            tc.get("allOf").is_some(),
            "TransformCertificate should have allOf (subclasses Certificate)"
        );
    }

    #[test]
    fn qualified_keys_avoid_collisions() {
        let ontology = Ontology::full();
        let schema = to_json_schema(ontology);
        let defs = schema["$defs"].as_object().expect("$defs must be object");
        // Both op/Identity and morphism/Identity should exist.
        assert!(defs.contains_key("op/Identity"), "Missing op/Identity");
        assert!(
            defs.contains_key("morphism/Identity"),
            "Missing morphism/Identity"
        );
    }

    #[test]
    fn all_refs_resolve() {
        let ontology = Ontology::full();
        let schema = to_json_schema(ontology);
        let defs = schema["$defs"].as_object().expect("$defs must be object");
        let mut refs: Vec<String> = Vec::new();
        collect_refs(&schema, &mut refs);
        assert!(!refs.is_empty(), "Schema should contain at least one $ref");
        for r in &refs {
            let escaped_key = r
                .strip_prefix("#/$defs/")
                .unwrap_or_else(|| panic!("Unexpected $ref format: {}", r));
            // Unescape JSON Pointer per RFC 6901: ~1 -> /, ~0 -> ~
            let key = escaped_key.replace("~1", "/").replace("~0", "~");
            assert!(
                defs.contains_key(&key),
                "$ref '{}' resolves to key '{}' which does not exist in $defs",
                r,
                key
            );
        }
    }

    fn collect_refs(value: &Value, refs: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                if let Some(Value::String(r)) = map.get("$ref") {
                    refs.push(r.clone());
                }
                for v in map.values() {
                    collect_refs(v, refs);
                }
            }
            Value::Array(arr) => {
                for v in arr {
                    collect_refs(v, refs);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn version_in_description() {
        let ontology = Ontology::full();
        let schema = to_json_schema(ontology);
        let desc = schema["description"]
            .as_str()
            .expect("description must be string");
        assert!(
            desc.contains(ontology.version),
            "Version '{}' not found in description",
            ontology.version
        );
    }
}
