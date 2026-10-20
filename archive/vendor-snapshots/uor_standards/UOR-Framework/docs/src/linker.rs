//! Resolves `{@class}`, `{@prop}`, `{@ind}` IRIs to relative HTML paths.

use crate::extractor::OntologyIndex;

/// Resolves an IRI to an absolute site-relative HTML link for use in docs pages.
///
/// For `{@class https://uor.foundation/schema/Ring}`:
/// - Returns `/docs/namespaces/schema.html#Ring`
///
/// Absolute paths are used so the link is correct regardless of the page's
/// depth within the docs hierarchy.
pub fn resolve_ref(kind: &str, iri: &str, index: &OntologyIndex) -> String {
    let ns = match index.namespace_for_iri(iri) {
        Some(ns) => ns,
        None => return format!("#{}", fragment_from_iri(iri)),
    };

    let prefix = ns.prefix;
    let fragment = fragment_from_iri(iri);

    let _ = kind; // kind used for semantic clarity but path is same for all
    let base_path = std::env::var("PUBLIC_BASE_PATH").unwrap_or_default();
    let base_path = base_path.trim_end_matches('/');
    format!("{}/docs/namespaces/{}.html#{}", base_path, prefix, fragment)
}

/// Extracts the local name from an IRI (the last path segment after `/`).
pub fn fragment_from_iri(iri: &str) -> String {
    iri.rsplit('/')
        .next()
        .unwrap_or(iri)
        .trim_end_matches('#')
        .to_string()
}
