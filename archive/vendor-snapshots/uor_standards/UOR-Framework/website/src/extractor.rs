//! Builds the site model from `uor_ontology::Ontology` and `uor_docs` content.

use uor_ontology::{NamespaceModule, Ontology};

use crate::model::{BreadcrumbItem, NamespaceSummary, SearchEntry};

/// Builds the list of namespace summaries for the homepage grid.
pub fn namespace_summaries(base_path: &str) -> Vec<NamespaceSummary> {
    let ontology = Ontology::full();
    ontology
        .namespaces
        .iter()
        .map(|m| namespace_summary_from_module(m, base_path))
        .collect()
}

/// Converts a namespace module to a `NamespaceSummary`.
fn namespace_summary_from_module(module: &NamespaceModule, base_path: &str) -> NamespaceSummary {
    let ns = &module.namespace;
    NamespaceSummary {
        prefix: ns.prefix.to_string(),
        iri: ns.iri.to_string(),
        label: ns.label.to_string(),
        comment: ns.comment.to_string(),
        space: format!("{:?}", ns.space).to_lowercase(),
        url: format!("{}/namespaces/{}/", base_path, ns.prefix),
        class_count: module.classes.len(),
        property_count: module.properties.len(),
        individual_count: module.individuals.len(),
    }
}

/// Builds the full search index from all ontology terms.
pub fn build_search_index(base_path: &str) -> Vec<SearchEntry> {
    let ontology = Ontology::full();
    let enum_names = Ontology::enum_class_names();
    let mut entries = Vec::new();

    for module in &ontology.namespaces {
        let prefix = module.namespace.prefix;
        let space_str = format!("{:?}", module.namespace.space).to_lowercase();

        // Namespace itself
        entries.push(SearchEntry {
            label: module.namespace.label.to_string(),
            description: module.namespace.comment.to_string(),
            url: format!("{}/namespaces/{}/", base_path, prefix),
            kind: "namespace".to_string(),
            space: space_str.clone(),
            namespace: String::new(),
            subkind: String::new(),
        });

        // Classes
        for class in &module.classes {
            let subkind = if enum_names.contains(&class.label) {
                "enum".to_string()
            } else {
                String::new()
            };
            entries.push(SearchEntry {
                label: class.label.to_string(),
                description: class.comment.to_string(),
                url: format!(
                    "{}/namespaces/{}/#class-{}",
                    base_path,
                    prefix,
                    local_name(class.id)
                ),
                kind: "class".to_string(),
                space: space_str.clone(),
                namespace: prefix.to_string(),
                subkind,
            });
        }

        // Properties
        for prop in &module.properties {
            entries.push(SearchEntry {
                label: prop.label.to_string(),
                description: prop.comment.to_string(),
                url: format!(
                    "{}/namespaces/{}/#prop-{}",
                    base_path,
                    prefix,
                    local_name(prop.id)
                ),
                kind: "property".to_string(),
                space: space_str.clone(),
                namespace: prefix.to_string(),
                subkind: String::new(),
            });
        }

        // Individuals
        for ind in &module.individuals {
            let subkind = if ind.type_.ends_with("Identity") {
                "identity".to_string()
            } else {
                String::new()
            };
            entries.push(SearchEntry {
                label: ind.label.to_string(),
                description: ind.comment.to_string(),
                url: format!(
                    "{}/namespaces/{}/#ind-{}",
                    base_path,
                    prefix,
                    local_name(ind.id)
                ),
                kind: "individual".to_string(),
                space: space_str.clone(),
                namespace: prefix.to_string(),
                subkind,
            });
        }
    }

    entries
}

/// Serializes namespace summaries into a JavaScript `const exploreData = {...}` block.
///
/// Embedded as an inline `<script>` in `explore/index.html`.
pub fn generate_explore_data(summaries: &[NamespaceSummary]) -> String {
    match serde_json::to_string(summaries) {
        Ok(json) => format!("const exploreData = {json};"),
        Err(_) => "const exploreData = [];".to_string(),
    }
}

/// Extracts the local name from an IRI.
fn local_name(iri: &str) -> &str {
    iri.rsplit('/').next().unwrap_or(iri)
}

/// Builds standard breadcrumbs for a namespace page.
pub fn namespace_breadcrumbs(label: &str, base_path: &str) -> Vec<BreadcrumbItem> {
    vec![
        BreadcrumbItem {
            label: "Home".to_string(),
            url: format!("{}/", base_path),
        },
        BreadcrumbItem {
            label: "Reference".to_string(),
            url: format!("{}/namespaces/", base_path),
        },
        BreadcrumbItem {
            label: label.to_string(),
            url: String::new(),
        },
    ]
}

/// Builds standard breadcrumbs for the homepage.
pub fn home_breadcrumbs(base_path: &str) -> Vec<BreadcrumbItem> {
    vec![BreadcrumbItem {
        label: "Home".to_string(),
        url: format!("{}/", base_path),
    }]
}

/// Builds breadcrumbs for the namespaces index page.
pub fn namespaces_index_breadcrumbs(base_path: &str) -> Vec<BreadcrumbItem> {
    vec![
        BreadcrumbItem {
            label: "Home".to_string(),
            url: format!("{}/", base_path),
        },
        BreadcrumbItem {
            label: "Reference".to_string(),
            url: String::new(),
        },
    ]
}

/// Builds standard breadcrumbs for a simple top-level page.
pub fn simple_breadcrumbs(label: &str, base_path: &str) -> Vec<BreadcrumbItem> {
    vec![
        BreadcrumbItem {
            label: "Home".to_string(),
            url: format!("{}/", base_path),
        },
        BreadcrumbItem {
            label: label.to_string(),
            url: String::new(),
        },
    ]
}

/// Builds breadcrumbs for pages in the Learn section.
pub fn learn_breadcrumbs(label: &str, base_path: &str) -> Vec<BreadcrumbItem> {
    vec![
        BreadcrumbItem {
            label: "Home".to_string(),
            url: format!("{}/", base_path),
        },
        BreadcrumbItem {
            label: "Learn".to_string(),
            url: format!("{}/learn/", base_path),
        },
        BreadcrumbItem {
            label: label.to_string(),
            url: String::new(),
        },
    ]
}

/// Builds breadcrumbs for pages in the Reference section.
pub fn reference_breadcrumbs(label: &str, base_path: &str) -> Vec<BreadcrumbItem> {
    vec![
        BreadcrumbItem {
            label: "Home".to_string(),
            url: format!("{}/", base_path),
        },
        BreadcrumbItem {
            label: "Reference".to_string(),
            url: format!("{}/namespaces/", base_path),
        },
        BreadcrumbItem {
            label: label.to_string(),
            url: String::new(),
        },
    ]
}

/// Builds breadcrumbs for a concept detail page.
pub fn concept_breadcrumbs(title: &str, base_path: &str) -> Vec<BreadcrumbItem> {
    vec![
        BreadcrumbItem {
            label: "Home".to_string(),
            url: format!("{}/", base_path),
        },
        BreadcrumbItem {
            label: "Learn".to_string(),
            url: format!("{}/learn/", base_path),
        },
        BreadcrumbItem {
            label: "Concepts".to_string(),
            url: format!("{}/concepts/", base_path),
        },
        BreadcrumbItem {
            label: title.to_string(),
            url: String::new(),
        },
    ]
}
