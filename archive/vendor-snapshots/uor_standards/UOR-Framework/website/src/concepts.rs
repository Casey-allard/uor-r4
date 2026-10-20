//! Concept page discovery, rendering, and cross-reference directive expansion.
//!
//! Concept pages are discovered dynamically from `content/concepts/*.md` files.
//! Adding a new `.md` file automatically produces a new HTML page — no hard-coded
//! slug list is maintained here.
//!
//! The directive system supports `{@class IRI}`, `{@prop IRI}`, `{@ind IRI}`,
//! `{@ns prefix}`, `{@concept slug}`, and `{@count:KEY}` directives that expand
//! to Markdown links pointing at website namespace pages.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use uor_ontology::Ontology;

use crate::model::ConceptPage;

// ── Concept relations ───────────────────────────────────────────────────────

/// Maps concept page slugs to related namespace prefixes and concept slugs.
///
/// Format: `(slug, related_namespace_prefixes, related_concept_slugs)`
pub const CONCEPT_RELATIONS: &[(&str, &[&str], &[&str])] = &[
    (
        "ring",
        &["schema", "op"],
        &["witt-levels", "content-addressing", "partition"],
    ),
    (
        "witt-levels",
        &["schema", "op"],
        &["ring", "proof-system", "observables"],
    ),
    (
        "site",
        &["type", "partition", "morphism"],
        &["partition", "resolution", "observables"],
    ),
    ("content-addressing", &["u"], &["ring", "site", "partition"]),
    ("partition", &["partition"], &["ring", "site", "resolution"]),
    (
        "resolution",
        &["resolver", "query", "state"],
        &["proof-system", "partition", "observables"],
    ),
    (
        "observables",
        &["observable"],
        &["witt-levels", "resolution", "homology"],
    ),
    (
        "homology",
        &["homology", "cohomology"],
        &["site", "resolution", "observables"],
    ),
    (
        "proof-system",
        &["proof", "derivation", "trace"],
        &["ring", "witt-levels", "resolution"],
    ),
    (
        "certification",
        &["cert", "proof", "derivation"],
        &["proof-system", "resolution"],
    ),
    ("morphisms", &["morphism", "type"], &["site", "partition"]),
    (
        "state-management",
        &["state", "resolver"],
        &["resolution", "certification"],
    ),
];

/// Returns the related namespace prefixes and concept slugs for a given concept.
pub fn concept_relations(slug: &str) -> (&[&str], &[&str]) {
    CONCEPT_RELATIONS
        .iter()
        .find(|(s, _, _)| *s == slug)
        .map(|(_, ns, cs)| (*ns, *cs))
        .unwrap_or((&[], &[]))
}

/// Returns concept slugs that list the given namespace prefix as related.
pub fn concepts_for_namespace(prefix: &str) -> Vec<&'static str> {
    CONCEPT_RELATIONS
        .iter()
        .filter(|(_, ns_list, _)| ns_list.contains(&prefix))
        .map(|(slug, _, _)| *slug)
        .collect()
}

// ── Page discovery ──────────────────────────────────────────────────────────

/// Discovers and returns all concept pages from the `content/concepts/` directory.
///
/// Pages are sorted alphabetically by slug for deterministic output order.
///
/// # Errors
///
/// Returns an error if the concepts directory cannot be read.
pub fn concept_page_list(concept_dirs: &[PathBuf], base_path: &str) -> Result<Vec<ConceptPage>> {
    // Merge every concept-content root into one catalog. Earlier roots win on
    // slug conflict, so the website's own curated page is preferred over the
    // docs reference page for the six overlapping concepts. Every concept the
    // project authors — in any root — therefore gets a `/concepts/<slug>.html`
    // page and a card in the concepts index.
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut pages: Vec<ConceptPage> = Vec::new();

    for dir in concept_dirs {
        if !dir.exists() {
            continue;
        }
        let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read {}", dir.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
            // prism.md is merged into the pipeline page — skip it as a standalone concept
            .filter(|e| {
                e.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s != "prism")
                    .unwrap_or(true)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in &entries {
            let path = entry.path();
            let slug = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            if !seen.insert(slug.clone()) {
                continue; // already provided by an earlier (higher-priority) root
            }
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let title = first_h1(&raw).unwrap_or_else(|| slug.replace('-', " "));
            let description = first_paragraph(&raw)
                .map(|d| strip_directives_to_plain_text(&d))
                .unwrap_or_default();
            let space = infer_space(&slug);
            pages.push(ConceptPage {
                url: format!("{base_path}/concepts/{slug}.html"),
                slug,
                title,
                description,
                space,
                source: path,
            });
        }
    }

    // Deterministic, alphabetical catalog order for the concept grid.
    pages.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(pages)
}

// ── Rendering ───────────────────────────────────────────────────────────────

/// Reads and renders a concept markdown file to an HTML body string.
///
/// Expands `{@class}`, `{@prop}`, `{@ind}`, `{@ns}`, `{@concept}`, and
/// `{@count:KEY}` directives to Markdown links before converting to HTML.
///
/// # Errors
///
/// Returns an error if the content file cannot be read.
pub fn render_concept_from_file(
    path: &Path,
    ontology: &Ontology,
    concept_list: &[ConceptPage],
    base_path: &str,
) -> Result<String> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read concept file {}", path.display()))?;
    let expanded = expand_directives(&raw, ontology, concept_list, base_path);
    let html = markdown_to_html(&expanded);
    Ok(retarget_docs_relative_links(&html, base_path))
}

/// Retargets the docs-relative links some concept sources author
/// (`../namespaces/<x>.html`, `../guides/<x>.html`) to absolute docs URLs.
///
/// Concept pages are rendered into `/concepts/` on the website, but a concept
/// imported from `docs/content/concepts/` may link to its docs siblings using
/// docs-relative paths that only resolve under `/docs/concepts/`. Those targets
/// (per-namespace reference pages, how-to guides) live only in the docs tree,
/// so the links are rewritten to point there. The website's own concepts use
/// `{@ns}`/`{@class}` directives and `../pipeline/` (a real website path), none
/// of which match these patterns, so they are left untouched.
fn retarget_docs_relative_links(html: &str, base_path: &str) -> String {
    html.replace(
        "href=\"../namespaces/",
        &format!("href=\"{base_path}/docs/namespaces/"),
    )
    .replace(
        "href=\"../guides/",
        &format!("href=\"{base_path}/docs/guides/"),
    )
}

/// Converts CommonMark markdown to HTML.
pub fn markdown_to_html(source: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES;
    let parser = Parser::new_ext(source, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

// ── Directive expansion ─────────────────────────────────────────────────────

/// Expands website directives in concept markdown before HTML conversion.
///
/// Supports: `{@class IRI}`, `{@prop IRI}`, `{@ind IRI}`, `{@ns prefix}`,
/// `{@concept slug}`, `{@count:KEY}`.
///
/// Directives resolve to Markdown links pointing at website namespace pages
/// (not docs pages). Unknown directives pass through unchanged.
pub(crate) fn expand_directives(
    source: &str,
    ontology: &Ontology,
    concept_list: &[ConceptPage],
    base_path: &str,
) -> String {
    let mut result = String::with_capacity(source.len());
    let mut remaining = source;

    while let Some(start) = remaining.find("{@") {
        result.push_str(&remaining[..start]);
        remaining = &remaining[start..];

        let end = match remaining.find('}') {
            Some(e) => e,
            None => {
                result.push_str(remaining);
                return result;
            }
        };

        let directive = &remaining[2..end];
        remaining = &remaining[end + 1..];

        // Handle {@count:KEY} directives (colon syntax, no space argument)
        if let Some(count_key) = directive.strip_prefix("count:") {
            result.push_str(&resolve_count(count_key));
            continue;
        }

        let parts: Vec<&str> = directive.splitn(2, ' ').collect();
        if parts.len() != 2 {
            result.push_str(&format!("{{@{directive}}}"));
            continue;
        }

        let kind = parts[0].trim();
        let arg = parts[1].trim();

        let link = match kind {
            "class" => resolve_class(arg, ontology, base_path),
            "prop" => resolve_prop(arg, ontology, base_path),
            "ind" => resolve_ind(arg, ontology, base_path),
            "ns" => resolve_ns(arg, ontology, base_path),
            "concept" => resolve_concept(arg, concept_list, base_path),
            _ => format!("{{@{kind} {arg}}}"),
        };

        result.push_str(&link);
    }

    result.push_str(remaining);
    result
}

/// Resolves `{@class IRI}` to a Markdown link to the namespace page.
fn resolve_class(iri: &str, ontology: &Ontology, base_path: &str) -> String {
    for module in &ontology.namespaces {
        if let Some(class) = module.classes.iter().find(|c| c.id == iri) {
            let prefix = module.namespace.prefix;
            let local = local_name(iri);
            return format!(
                "[{}]({}/namespaces/{}/#class-{})",
                class.label, base_path, prefix, local
            );
        }
    }
    format!("`{}`", local_name(iri))
}

/// Resolves `{@prop IRI}` to a Markdown link to the namespace page.
fn resolve_prop(iri: &str, ontology: &Ontology, base_path: &str) -> String {
    for module in &ontology.namespaces {
        if let Some(prop) = module.properties.iter().find(|p| p.id == iri) {
            let prefix = module.namespace.prefix;
            let local = local_name(iri);
            return format!(
                "[{}]({}/namespaces/{}/#prop-{})",
                prop.label, base_path, prefix, local
            );
        }
    }
    format!("`{}`", local_name(iri))
}

/// Resolves `{@ind IRI}` to a Markdown link to the namespace page.
fn resolve_ind(iri: &str, ontology: &Ontology, base_path: &str) -> String {
    for module in &ontology.namespaces {
        if let Some(ind) = module.individuals.iter().find(|i| i.id == iri) {
            let prefix = module.namespace.prefix;
            let local = local_name(iri);
            return format!(
                "[{}]({}/namespaces/{}/#ind-{})",
                ind.label, base_path, prefix, local
            );
        }
    }
    format!("`{}`", local_name(iri))
}

/// Resolves `{@ns prefix}` to a Markdown link to the namespace page.
fn resolve_ns(prefix: &str, ontology: &Ontology, base_path: &str) -> String {
    if let Some(module) = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == prefix)
    {
        format!(
            "[{}]({}/namespaces/{}/)",
            module.namespace.label, base_path, prefix
        )
    } else {
        format!("`{}`", prefix)
    }
}

/// Resolves `{@concept slug}` to a Markdown link to the concept page.
fn resolve_concept(slug: &str, concept_list: &[ConceptPage], base_path: &str) -> String {
    if let Some(concept) = concept_list.iter().find(|c| c.slug == slug) {
        format!("[{}]({}/concepts/{}.html)", concept.title, base_path, slug)
    } else {
        format!("`{}`", slug)
    }
}

/// Resolves `{@count:KEY}` to the current ontology count value.
fn resolve_count(key: &str) -> String {
    use uor_ontology::counts;
    match key {
        "namespaces" => counts::NAMESPACES.to_string(),
        "classes" => counts::CLASSES.to_string(),
        "properties" => counts::PROPERTIES.to_string(),
        "individuals" => counts::INDIVIDUALS.to_string(),
        "amendments" => counts::AMENDMENTS.to_string(),
        "shacl_tests" => counts::SHACL_TESTS.to_string(),
        "traits" => (counts::CLASSES - counts::ENUM_CLASSES).to_string(),
        "shapes" => counts::CLASSES.to_string(),
        "identities" => counts::IDENTITY_COUNT.to_string(),
        "methods" => counts::METHODS.to_string(),
        "constant_modules" => counts::CONSTANT_MODULES.to_string(),
        "enums" => counts::ENUM_CLASSES.to_string(),
        "kernel_ns" => counts::KERNEL_NAMESPACES.to_string(),
        "bridge_ns" => counts::BRIDGE_NAMESPACES.to_string(),
        "user_ns" => counts::USER_NAMESPACES.to_string(),
        "conformance_checks" => counts::CONFORMANCE_CHECKS.to_string(),
        _ => format!("{{@count:{key}}}"),
    }
}

/// Extracts the local name from an IRI (the last path segment after `/`).
fn local_name(iri: &str) -> &str {
    iri.rsplit('/').next().unwrap_or(iri)
}

// ── Plain-text directive stripping ───────────────────────────────────────────

/// Strips `{@...}` directives from a raw markdown string, replacing them
/// with plain-text equivalents suitable for concept card descriptions.
///
/// Unlike `expand_directives` (which produces Markdown links), this produces
/// bare text so that descriptions render cleanly in `escape_html()` contexts.
fn strip_directives_to_plain_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut remaining = text;

    while let Some(start) = remaining.find("{@") {
        result.push_str(&remaining[..start]);
        remaining = &remaining[start..];

        let end = match remaining.find('}') {
            Some(e) => e,
            None => {
                result.push_str(remaining);
                return result;
            }
        };

        let directive = &remaining[2..end];
        remaining = &remaining[end + 1..];

        // {@count:KEY} -> numeric value
        if let Some(count_key) = directive.strip_prefix("count:") {
            result.push_str(&resolve_count(count_key));
            continue;
        }

        let parts: Vec<&str> = directive.splitn(2, ' ').collect();
        if parts.len() != 2 {
            continue;
        }

        let kind = parts[0].trim();
        let arg = parts[1].trim();

        match kind {
            "class" | "prop" | "ind" => {
                // Extract local name from IRI
                result.push_str(local_name(arg));
            }
            "ns" => {
                // Use the prefix as-is
                result.push_str(arg);
            }
            "concept" => {
                // Convert slug to readable text
                let title: String = arg
                    .split('-')
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => {
                                let upper: String = f.to_uppercase().collect();
                                upper + c.as_str()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                result.push_str(&title);
            }
            _ => {}
        }
    }

    result.push_str(remaining);
    result
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Extracts the first ATX heading (`# Title`) from raw markdown.
fn first_h1(md: &str) -> Option<String> {
    md.lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim().to_string())
}

/// Extracts the first non-heading, non-empty paragraph from raw markdown.
fn first_paragraph(md: &str) -> Option<String> {
    md.lines()
        .skip_while(|l| l.starts_with('#') || l.trim().is_empty())
        .take_while(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .into()
}

/// Infers the space classification for a concept page from its slug.
///
/// This is editorial metadata for color-coding cards — not a strict ontology rule.
fn infer_space(slug: &str) -> String {
    match slug {
        "ring" | "witt-levels" | "content-addressing" => "kernel".to_string(),
        "site" | "partition" | "resolution" | "observables" | "homology" | "proof-system"
        | "certification" => "bridge".to_string(),
        "morphisms" | "state-management" => "user".to_string(),
        _ => "kernel".to_string(),
    }
}
