//! Ontology → Lean 4 mapping tables.
//!
//! Deterministic mappings from OWL constructs to Lean 4 identifiers, modules,
//! and types.

use std::collections::HashMap;

use uor_ontology::model::iris::*;
use uor_ontology::model::Space;

/// Mapping from a namespace IRI to its Lean 4 module path.
pub struct LeanNamespaceMapping {
    /// The space classification (Kernel, Bridge, User).
    pub space: Space,
    /// e.g. "Kernel", "Bridge", "User"
    pub space_module: &'static str,
    /// e.g. "Address", "Schema", "Op"
    pub file_module: &'static str,
}

/// Returns the namespace → Lean module mapping (33 entries).
pub fn lean_namespace_mappings() -> HashMap<&'static str, LeanNamespaceMapping> {
    let mut m = HashMap::new();
    // Kernel space (17 namespaces)
    ins(&mut m, NS_U, Space::Kernel, "Kernel", "Address");
    ins(&mut m, NS_SCHEMA, Space::Kernel, "Kernel", "Schema");
    ins(&mut m, NS_OP, Space::Kernel, "Kernel", "Op");
    ins(&mut m, NS_CARRY, Space::Kernel, "Kernel", "Carry");
    ins(&mut m, NS_REDUCTION, Space::Kernel, "Kernel", "Reduction");
    ins(
        &mut m,
        NS_CONVERGENCE,
        Space::Kernel,
        "Kernel",
        "Convergence",
    );
    ins(&mut m, NS_DIVISION, Space::Kernel, "Kernel", "Division");
    ins(&mut m, NS_MONOIDAL, Space::Kernel, "Kernel", "Monoidal");
    ins(&mut m, NS_OPERAD, Space::Kernel, "Kernel", "Operad");
    ins(&mut m, NS_EFFECT, Space::Kernel, "Kernel", "Effect");
    ins(&mut m, NS_PREDICATE, Space::Kernel, "Kernel", "Predicate");
    ins(&mut m, NS_PARALLEL, Space::Kernel, "Kernel", "Parallel");
    ins(&mut m, NS_STREAM, Space::Kernel, "Kernel", "Stream_");
    ins(&mut m, NS_FAILURE, Space::Kernel, "Kernel", "Failure");
    ins(&mut m, NS_LINEAR, Space::Kernel, "Kernel", "Linear");
    ins(&mut m, NS_RECURSION, Space::Kernel, "Kernel", "Recursion");
    ins(&mut m, NS_REGION, Space::Kernel, "Kernel", "Region");
    // Bridge space (13 namespaces)
    ins(&mut m, NS_QUERY, Space::Bridge, "Bridge", "Query");
    ins(&mut m, NS_RESOLVER, Space::Bridge, "Bridge", "Resolver");
    ins(&mut m, NS_PARTITION, Space::Bridge, "Bridge", "Partition");
    ins(&mut m, NS_OBSERVABLE, Space::Bridge, "Bridge", "Observable");
    ins(&mut m, NS_HOMOLOGY, Space::Bridge, "Bridge", "Homology");
    ins(&mut m, NS_COHOMOLOGY, Space::Bridge, "Bridge", "Cohomology");
    ins(&mut m, NS_PROOF, Space::Bridge, "Bridge", "Proof");
    ins(&mut m, NS_DERIVATION, Space::Bridge, "Bridge", "Derivation");
    ins(&mut m, NS_TRACE, Space::Bridge, "Bridge", "Trace");
    ins(&mut m, NS_CERT, Space::Bridge, "Bridge", "Cert");
    ins(
        &mut m,
        NS_INTERACTION,
        Space::Bridge,
        "Bridge",
        "Interaction",
    );
    ins(&mut m, NS_BOUNDARY, Space::Bridge, "Bridge", "Boundary");
    ins(
        &mut m,
        NS_CONFORMANCE,
        Space::Bridge,
        "Bridge",
        "Conformance_",
    );
    // Product/Coproduct Completion Amendment — foundation namespace.
    ins(&mut m, NS_FOUNDATION, Space::Bridge, "Bridge", "Foundation");
    // User space (3 namespaces)
    ins(&mut m, NS_TYPE, Space::User, "User", "Type_");
    ins(&mut m, NS_MORPHISM, Space::User, "User", "Morphism");
    ins(&mut m, NS_STATE, Space::User, "User", "State");
    m
}

/// Helper to insert a mapping entry.
fn ins(
    m: &mut HashMap<&'static str, LeanNamespaceMapping>,
    ns: &'static str,
    space: Space,
    space_module: &'static str,
    file_module: &'static str,
) {
    m.insert(
        ns,
        LeanNamespaceMapping {
            space,
            space_module,
            file_module,
        },
    );
}

/// Maps an XSD IRI to the corresponding Lean `P.` field expression.
pub fn xsd_to_lean_type(xsd_iri: &str) -> Option<&'static str> {
    match xsd_iri {
        XSD_STRING => Some("P.String"),
        XSD_INTEGER => Some("P.Integer"),
        XSD_NON_NEGATIVE_INTEGER => Some("P.NonNegativeInteger"),
        XSD_POSITIVE_INTEGER => Some("P.PositiveInteger"),
        XSD_BOOLEAN => Some("P.Boolean"),
        XSD_DECIMAL => Some("P.Decimal"),
        XSD_DATETIME => Some("P.String"),
        XSD_HEX_BINARY => Some("P.String"),
        _ => None,
    }
}

/// Lean 4 reserved words that require guillemet escaping.
const LEAN_KEYWORDS: &[&str] = &[
    "type",
    "def",
    "where",
    "structure",
    "class",
    "theorem",
    "let",
    "do",
    "return",
    "match",
    "if",
    "else",
    "for",
    "in",
    "open",
    "namespace",
    "end",
    "import",
    "mutual",
    "variable",
    "instance",
    "deriving",
    "extends",
    "with",
    "fun",
    "have",
    "show",
    "calc",
    "by",
    "sorry",
    "set_option",
    "section",
    "true",
    "false",
    "mod",
    "protected",
    "private",
    "noncomputable",
    "unsafe",
    "partial",
    "macro",
    "syntax",
    "axiom",
    "opaque",
    "abbrev",
    "inductive",
    "example",
    "at",
    "from",
    "to",
    "then",
    "as",
    "is",
    "of",
    "nomatch",
    "rec",
    "lemma",
    "attribute",
];

/// Converts a camelCase label to a Lean field name, escaping keywords with guillemets.
pub fn to_lean_field_name(s: &str) -> String {
    let name = to_camel_case(s);
    if LEAN_KEYWORDS.contains(&name.as_str()) {
        format!("\u{ab}{name}\u{bb}")
    } else {
        name
    }
}

/// Converts a PascalCase or camelCase label into camelCase.
///
/// The ontology uses camelCase for property labels already, so this typically
/// just returns the input unchanged. Handles the edge case where the first
/// character is uppercase.
fn to_camel_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let mut result: String = c.to_lowercase().collect();
            result.extend(chars);
            result
        }
    }
}

/// Extracts the local name from a full IRI (after the last `/` or `#`).
pub fn local_name(iri: &str) -> &str {
    let after_slash = iri.rsplit('/').next().unwrap_or(iri);
    after_slash.rsplit('#').next().unwrap_or(after_slash)
}

/// Returns the fully-qualified Lean structure path for a class IRI.
///
/// E.g. `"https://uor.foundation/partition/Partition"` → `"UOR.Bridge.Partition.Partition"`
pub fn lean_qualified_name(
    class_iri: &str,
    ns_map: &HashMap<&str, LeanNamespaceMapping>,
) -> Option<String> {
    for (ns_iri, mapping) in ns_map {
        if class_iri.starts_with(ns_iri) {
            let name = local_name(class_iri);
            return Some(format!(
                "UOR.{}.{}.{name}",
                mapping.space_module, mapping.file_module
            ));
        }
    }
    None
}

/// Returns the Lean module import path for a namespace IRI.
///
/// E.g. `"https://uor.foundation/op/"` → `"UOR.Kernel.Op"`
pub fn lean_module_import(
    ns_iri: &str,
    ns_map: &HashMap<&str, LeanNamespaceMapping>,
) -> Option<String> {
    ns_map
        .get(ns_iri)
        .map(|m| format!("UOR.{}.{}", m.space_module, m.file_module))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camel_case_conversion() {
        assert_eq!(to_camel_case("FreeRank"), "freeRank");
        assert_eq!(to_camel_case("arity"), "arity");
        assert_eq!(to_camel_case("IsClosed"), "isClosed");
    }

    #[test]
    fn keyword_escaping() {
        assert_eq!(to_lean_field_name("type"), "\u{ab}type\u{bb}");
        assert_eq!(to_lean_field_name("arity"), "arity");
        assert_eq!(to_lean_field_name("match"), "\u{ab}match\u{bb}");
    }

    #[test]
    fn local_name_extraction() {
        assert_eq!(
            local_name("https://uor.foundation/partition/Partition"),
            "Partition"
        );
        assert_eq!(
            local_name("http://www.w3.org/2001/XMLSchema#string"),
            "string"
        );
    }

    #[test]
    fn xsd_mapping_complete() {
        // All 8 XSD IRIs must map to Lean types
        assert!(xsd_to_lean_type(XSD_STRING).is_some());
        assert!(xsd_to_lean_type(XSD_INTEGER).is_some());
        assert!(xsd_to_lean_type(XSD_NON_NEGATIVE_INTEGER).is_some());
        assert!(xsd_to_lean_type(XSD_POSITIVE_INTEGER).is_some());
        assert!(xsd_to_lean_type(XSD_BOOLEAN).is_some());
        assert!(xsd_to_lean_type(XSD_DECIMAL).is_some());
        assert!(xsd_to_lean_type(XSD_DATETIME).is_some());
        assert!(xsd_to_lean_type(XSD_HEX_BINARY).is_some());
    }
}
