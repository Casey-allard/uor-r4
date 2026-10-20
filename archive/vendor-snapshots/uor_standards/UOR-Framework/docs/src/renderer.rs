//! Renders Markdown content with `{@class}`, `{@prop}`, `{@ind}` DSL expansion.

use pulldown_cmark::{html, Options, Parser};

use crate::extractor::OntologyIndex;
use crate::linker::resolve_ref;

/// Renders a Markdown file to HTML, expanding `{@class}`, `{@prop}`, `{@ind}` directives.
///
/// # Errors
///
/// This function is infallible (returns `String`).
pub fn render_markdown(source: &str, index: &OntologyIndex) -> String {
    let expanded = expand_directives(source, index);
    markdown_to_html(&expanded)
}

/// Expands `{@class iri}`, `{@prop iri}`, `{@ind iri}` directives into Markdown links.
pub fn expand_directives(source: &str, index: &OntologyIndex) -> String {
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
            result.push_str(&resolve_count(count_key, index));
            continue;
        }

        let parts: Vec<&str> = directive.splitn(2, ' ').collect();
        if parts.len() != 2 {
            result.push_str(&format!("{{@{}}}", directive));
            continue;
        }

        let kind = parts[0].trim();
        let iri = parts[1].trim();

        let link = match kind {
            "class" => resolve_class_ref(iri, index),
            "prop" => resolve_prop_ref(iri, index),
            "ind" => resolve_ind_ref(iri, index),
            _ => format!("{{@{} {}}}", kind, iri),
        };

        result.push_str(&link);
    }

    result.push_str(remaining);
    result
}

/// Resolves a `{@class iri}` to a Markdown link.
fn resolve_class_ref(iri: &str, index: &OntologyIndex) -> String {
    if let Some(class) = index.classes.iter().find(|c| c.id == iri) {
        let href = resolve_ref("class", iri, index);
        format!("[{}]({})", class.label, href)
    } else {
        format!("`{}`", iri)
    }
}

/// Resolves a `{@prop iri}` to a Markdown link.
fn resolve_prop_ref(iri: &str, index: &OntologyIndex) -> String {
    if let Some(prop) = index.properties.iter().find(|p| p.id == iri) {
        let href = resolve_ref("prop", iri, index);
        format!("[{}]({})", prop.label, href)
    } else {
        format!("`{}`", iri)
    }
}

/// Resolves a `{@ind iri}` to a Markdown link.
fn resolve_ind_ref(iri: &str, index: &OntologyIndex) -> String {
    if let Some(ind) = index.individuals.iter().find(|i| i.id == iri) {
        let href = resolve_ref("ind", iri, index);
        format!("[{}]({})", ind.label, href)
    } else {
        format!("`{}`", iri)
    }
}

/// Resolves a `{@count:KEY}` directive to the current ontology count.
/// All numeric constants come from [`uor_ontology::counts`].
fn resolve_count(key: &str, index: &OntologyIndex) -> String {
    use uor_ontology::counts;
    match key {
        "namespaces" => index.modules.len().to_string(),
        "classes" => index.classes.len().to_string(),
        "properties" => index.properties.len().to_string(),
        "individuals" => index.individuals.len().to_string(),
        "amendments" => counts::AMENDMENTS.to_string(),
        "shacl_tests" => counts::SHACL_TESTS.to_string(),
        "traits" => (index.classes.len() - counts::ENUM_CLASSES).to_string(),
        "shapes" => index.classes.len().to_string(),
        "identities" => counts::IDENTITY_COUNT.to_string(),
        "methods" => counts::METHODS.to_string(),
        "constant_modules" => counts::CONSTANT_MODULES.to_string(),
        "enums" => counts::ENUM_CLASSES.to_string(),
        "kernel_ns" => counts::KERNEL_NAMESPACES.to_string(),
        "bridge_ns" => counts::BRIDGE_NAMESPACES.to_string(),
        "user_ns" => counts::USER_NAMESPACES.to_string(),
        "conformance_checks" => counts::CONFORMANCE_CHECKS.to_string(),
        _ => format!("{{@count:{}}}", key),
    }
}

/// Converts Markdown to HTML using pulldown-cmark.
pub fn markdown_to_html(markdown: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(markdown, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Renders a docs page inside the standard site shell with a sidebar.
///
/// Uses the same `<header class="site-header">` / `<footer class="site-footer">`
/// as the website, with the docs-specific nav tree in an `<aside class="docs-sidebar">`.
pub fn render_docs_page(
    title: &str,
    content_html: &str,
    site_nav_html: &str,
    docs_nav_html: &str,
    breadcrumb: &str,
    base_path: &str,
) -> String {
    let home_url = format!("{}/", base_path);
    let css_url = format!("{}/css/style.css", base_path);
    let js_url = format!("{}/js/search.js", base_path);
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title} — UOR Foundation</title>
<link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-QWTKZyjpPEjISv5WaRU9OFeRpok6YctnYmDr5pNlyT2bRjXh0JMhjY6hW+ALEwIH" crossorigin="anonymous">
<link rel="stylesheet" href="{css_url}">
</head>
<body>
<a href="#main-content" class="skip-link">Skip to main content</a>
<header class="navbar navbar-expand-lg navbar-dark site-header">
<div class="container-fluid">
<a class="navbar-brand site-logo" href="{home_url}">UOR Foundation</a>
<button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#siteNav" aria-controls="siteNav" aria-expanded="false" aria-label="Toggle navigation">
<span class="navbar-toggler-icon"></span>
</button>
<div class="collapse navbar-collapse" id="siteNav">
<nav aria-label="Site navigation" class="site-nav me-auto">
{site_nav_html}
</nav>
</div>
</div>
</header>
<div class="docs-layout">
<aside class="docs-sidebar">
<nav aria-label="Documentation navigation">
{docs_nav_html}
</nav>
</aside>
<main id="main-content">
<nav aria-label="Breadcrumb" class="site-breadcrumb">{breadcrumb}</nav>
<article class="page-content">
{content_html}
</article>
</main>
</div>
<footer class="site-footer">
<p>UOR Foundation — <a href="https://uor.foundation/">uor.foundation</a> — MIT</p>
</footer>
<script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous"></script>
<script src="{js_url}" defer></script>
</body>
</html>"##,
        title = escape_html(title),
        home_url = escape_html(&home_url),
        css_url = escape_html(&css_url),
        site_nav_html = site_nav_html,
        docs_nav_html = docs_nav_html,
        breadcrumb = breadcrumb,
        content_html = content_html,
        js_url = escape_html(&js_url),
    )
}

/// Escapes HTML special characters in a string.
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
