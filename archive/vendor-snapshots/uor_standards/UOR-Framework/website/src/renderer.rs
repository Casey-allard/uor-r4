//! HTML rendering for the website (no external template engine required).
//!
//! All HTML is generated directly in Rust for determinism and zero dependencies.

pub use uor_docs::renderer::escape_html;
use uor_ontology::{IndividualValue, NamespaceModule, Ontology, PropertyKind};

use crate::model::{BreadcrumbItem, ConceptPage, NamespaceSummary};

/// Renders a complete HTML page using the site layout.
pub fn render_page(
    title: &str,
    body: &str,
    nav_html: &str,
    breadcrumbs: &[BreadcrumbItem],
    base_path: &str,
) -> String {
    let crumb_html = render_breadcrumbs(breadcrumbs);
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
{nav_html}
</nav>
<a href="https://github.com/UOR-Foundation/UOR-Framework" class="github-link" target="_blank" rel="noopener">&#9733; Star on GitHub</a>
</div>
</div>
</header>
<main id="main-content">
<nav aria-label="Breadcrumb" class="site-breadcrumb">
{crumb_html}
</nav>
<article class="page-content">
{body}
</article>
</main>
<footer class="site-footer">
<p>UOR Foundation — <a href="https://uor.foundation/">uor.foundation</a> — <a href="https://github.com/UOR-Foundation/UOR-Framework">GitHub</a> — MIT</p>
</footer>
<script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous"></script>
<script src="{js_url}" defer></script>
</body>
</html>"##,
        title = escape_html(title),
        css_url = escape_html(&css_url),
        home_url = escape_html(&home_url),
        nav_html = nav_html,
        crumb_html = crumb_html,
        body = body,
        js_url = escape_html(&js_url),
    )
}

/// Renders breadcrumb navigation as an ordered list.
pub fn render_breadcrumbs(crumbs: &[BreadcrumbItem]) -> String {
    let mut html = String::from("<ol>\n");
    for (i, crumb) in crumbs.iter().enumerate() {
        if i + 1 == crumbs.len() {
            html.push_str(&format!(
                "<li aria-current=\"page\">{}</li>\n",
                escape_html(&crumb.label)
            ));
        } else {
            html.push_str(&format!(
                "<li><a href=\"{}\">{}</a></li>\n",
                escape_html(&crumb.url),
                escape_html(&crumb.label)
            ));
        }
    }
    html.push_str("</ol>");
    html
}

/// Renders the homepage body with namespace grid.
/// Featured namespace prefixes for the homepage (representative sample).
const FEATURED_PREFIXES: &[&str] = &["u", "schema", "op", "partition", "proof", "cert"];

/// Renders the homepage body with hero, inventory, pipeline steps, and featured namespaces.
pub fn render_homepage(summaries: &[NamespaceSummary], base_path: &str) -> String {
    let total_ns = summaries.len();
    let total_classes: usize = summaries.iter().map(|s| s.class_count).sum();
    let total_props: usize = summaries.iter().map(|s| s.property_count).sum();
    let total_inds: usize = summaries.iter().map(|s| s.individual_count).sum();

    let mut grid = String::new();
    for ns in summaries
        .iter()
        .filter(|ns| FEATURED_PREFIXES.contains(&ns.prefix.as_str()))
    {
        grid.push_str(&format!(
            r#"<article class="ns-card ns-space-{space}">
<h3><a href="{url}">{label}</a></h3>
<p class="ns-iri"><code>{iri}</code></p>
<p>{comment}</p>
<dl class="ns-counts">
<dt>Classes</dt><dd>{classes}</dd>
<dt>Properties</dt><dd>{props}</dd>
<dt>Individuals</dt><dd>{inds}</dd>
</dl>
</article>
"#,
            space = escape_html(&ns.space),
            url = escape_html(&ns.url),
            label = escape_html(&ns.label),
            iri = escape_html(&ns.iri),
            comment = escape_html(&ns.comment),
            classes = ns.class_count,
            props = ns.property_count,
            inds = ns.individual_count,
        ));
    }

    let learn_url = format!("{}/learn/", base_path);
    let ns_url = format!("{}/namespaces/", base_path);
    let search_url = format!("{}/search.html", base_path);
    let concepts_url = format!("{}/concepts", base_path);
    let pipeline_url = format!("{}/pipeline/", base_path);
    let explore_url = format!("{}/explore/", base_path);
    let identities_url = format!("{}/identities/", base_path);
    let download_url = format!("{}/download/", base_path);

    format!(
        r#"<section class="hero">
<h1>UOR Foundation</h1>
<p class="tagline">Universal Object Reference — a formal ontology for content-addressed, algebraically-structured object spaces.</p>
<p>
<a href="{learn_url}" class="cta-primary">Start Learning</a>
<a href="{ns_url}" class="cta-secondary">Browse Namespaces</a>
<a href="{search_url}" class="cta-secondary">Search</a>
</p>
</section>

<section class="inventory">
<h2>Ontology Inventory</h2>
<p>{total_ns} namespaces · {total_classes} classes · {total_props} properties · {total_inds} named individuals</p>
</section>

<section class="pipeline">
<h2>The Resolution Pipeline</h2>
<div class="pipeline-steps">
<div class="pipeline-step">
<span class="step-number">1</span>
<h3>Define</h3>
<p>Declare types with constraints that pin sites of the Z/2Z fibration.</p>
</div>
<div class="pipeline-arrow" aria-hidden="true"></div>
<div class="pipeline-step">
<span class="step-number">2</span>
<h3>Resolve</h3>
<p>Factorize under D_{{2^n}}, classify into partition components, measure observables.</p>
</div>
<div class="pipeline-arrow" aria-hidden="true"></div>
<div class="pipeline-step">
<span class="step-number">3</span>
<h3>Certify</h3>
<p>Attest the result with a verification hash and replayable computation trace.</p>
</div>
</div>
</section>

<section class="reading-guide" aria-labelledby="reading-guide-heading">
<h2 id="reading-guide-heading">Where to Start</h2>
<p>The UOR Foundation ontology is a formal mathematical framework. Choose a path based on what you want to learn:</p>
<div class="pathway-grid">
<article class="pathway pathway-kernel">
<h3>Understand the Mathematics</h3>
<p>Start with the algebraic substrate and build up to the full type system.</p>
<ol class="pathway-steps">
<li><a href="{concepts_url}/ring.html">The Ring Substrate</a></li>
<li><a href="{concepts_url}/witt-levels.html">Witt Levels</a></li>
<li><a href="{concepts_url}/content-addressing.html">Content Addressing</a></li>
<li><a href="{concepts_url}/site.html">Site Bundles</a></li>
</ol>
</article>
<article class="pathway pathway-bridge">
<h3>See the Pipeline</h3>
<p>Understand how the Define \u{{2192}} Resolve \u{{2192}} Certify pipeline works end to end.</p>
<ol class="pathway-steps">
<li><a href="{pipeline_url}">The PRISM Pipeline</a></li>
<li><a href="{explore_url}">Explore Dependencies</a></li>
</ol>
</article>
<article class="pathway pathway-user">
<h3>Browse the Reference</h3>
<p>Jump straight into the formal ontology artifacts and namespace documentation.</p>
<ol class="pathway-steps">
<li><a href="{ns_url}">All Namespaces</a></li>
<li><a href="{identities_url}">Algebraic Identities</a></li>
<li><a href="{download_url}">Download Artifacts</a></li>
</ol>
</article>
</div>
</section>

<section class="namespace-grid">
<h2>Featured Namespaces</h2>
<div class="grid">
{grid}
</div>
<p><a href="{ns_url}">View all {total_ns} namespaces &rarr;</a></p>
</section>"#,
        learn_url = escape_html(&learn_url),
        ns_url = escape_html(&ns_url),
        search_url = escape_html(&search_url),
        concepts_url = escape_html(&concepts_url),
        pipeline_url = escape_html(&pipeline_url),
        explore_url = escape_html(&explore_url),
        identities_url = escape_html(&identities_url),
        download_url = escape_html(&download_url),
        total_ns = total_ns,
        total_classes = total_classes,
        total_props = total_props,
        total_inds = total_inds,
        grid = grid,
    )
}

/// Renders the namespaces index page (Reference section landing).
pub fn render_namespaces_index(summaries: &[NamespaceSummary], base_path: &str) -> String {
    let kernel_count = summaries.iter().filter(|s| s.space == "kernel").count();
    let bridge_count = summaries.iter().filter(|s| s.space == "bridge").count();
    let user_count = summaries.iter().filter(|s| s.space == "user").count();
    let identity_count = uor_ontology::counts::IDENTITY_COUNT;
    let identities_url = format!("{base_path}/identities/");
    let explore_url = format!("{base_path}/explore/");

    let mut rows = String::new();
    for ns in summaries {
        rows.push_str(&format!(
            r#"<tr class="ns-space-{space}">
<td><a href="{url}"><code>{prefix}</code></a></td>
<td>{label}</td>
<td>{classes}</td>
<td>{props}</td>
<td>{inds}</td>
<td>{space}</td>
</tr>
"#,
            space = escape_html(&ns.space),
            url = escape_html(&ns.url),
            prefix = escape_html(&ns.prefix),
            label = escape_html(&ns.label),
            classes = ns.class_count,
            props = ns.property_count,
            inds = ns.individual_count,
        ));
    }

    format!(
        r#"<h1>Ontology Reference</h1>
<p class="reference-summary">{ns_count} namespaces in 3 spaces:
<span class="badge badge-kernel">Kernel ({kernel_count})</span>
<span class="badge badge-bridge">Bridge ({bridge_count})</span>
<span class="badge badge-user">User ({user_count})</span></p>

<div class="reference-quick-links">
<a href="{identities_url}" class="quick-link-card">
<h3>Algebraic Identities</h3>
<p>Browse {identity_count} verified identities across all verification domains.</p>
</a>
<a href="{explore_url}" class="quick-link-card">
<h3>Dependency Explorer</h3>
<p>Interactive graph of namespace import relationships.</p>
</a>
</div>

<h2>All Namespaces</h2>
<table>
<thead>
<tr><th>Prefix</th><th>Label</th><th>Classes</th><th>Properties</th><th>Individuals</th><th>Space</th></tr>
</thead>
<tbody>
{rows}
</tbody>
</table>"#,
        ns_count = summaries.len(),
    )
}

/// Renders a namespace detail page from the spec module.
pub fn render_namespace_page(module: &NamespaceModule, base_path: Option<&str>) -> String {
    let ns = &module.namespace;
    let mut body = format!(
        r#"<h1>{label}</h1>
<dl class="ns-meta">
<dt>IRI</dt><dd><code>{iri}</code></dd>
<dt>Prefix</dt><dd><code>{prefix}:</code></dd>
<dt>Space</dt><dd class="space-{space}">{space}</dd>
<dt>Comment</dt><dd>{comment}</dd>
</dl>
"#,
        label = escape_html(ns.label),
        iri = escape_html(ns.iri),
        prefix = escape_html(ns.prefix),
        space = format!("{:?}", ns.space).to_lowercase(),
        comment = escape_html(ns.comment),
    );

    // Documentation link
    let bp = base_path.unwrap_or("");
    body.push_str(&format!(
        "<p class=\"ns-docs-link\"><a href=\"{bp}/docs/namespaces/{prefix}.html\">View documentation reference</a></p>\n",
        prefix = escape_html(ns.prefix),
    ));

    // Space context paragraph
    body.push_str(&space_context_html(&ns.space, ns.prefix, bp));

    // Class hierarchy SVG (after meta block)
    let hierarchy_svg = crate::svg::render_class_hierarchy_svg(module);
    if !hierarchy_svg.is_empty() {
        body.push_str(&format!(
            "<figure class=\"diagram-container\" \
             aria-label=\"Class hierarchy for {} namespace\">\n\
             <figcaption>Class hierarchy</figcaption>\n\
             {hierarchy_svg}\n\
             </figure>\n",
            escape_html(ns.label),
        ));
    }

    // Imports
    if !ns.imports.is_empty() {
        body.push_str("<h2>Imports</h2><ul>\n");
        for imp in ns.imports {
            body.push_str(&format!("<li><code>{}</code></li>\n", escape_html(imp)));
        }
        body.push_str("</ul>\n");
    }

    // Classes
    if !module.classes.is_empty() {
        body.push_str("<h2>Classes</h2>\n<table>\n");
        body.push_str("<thead><tr><th>Name</th><th>Subclass Of</th><th>Disjoint With</th><th>Comment</th></tr></thead>\n<tbody>\n");
        for class in &module.classes {
            let local = local_name(class.id);
            body.push_str(&format!(
                "<tr id=\"class-{local}\"><td><code>{label}</code></td><td>{parents}</td><td>{disjoints}</td><td>{comment}</td></tr>\n",
                local = escape_html(local),
                label = escape_html(class.label),
                parents = class.subclass_of.iter().map(|p| format!("<code>{}</code>", escape_html(local_name(p)))).collect::<Vec<_>>().join(", "),
                disjoints = class.disjoint_with.iter().map(|d| format!("<code>{}</code>", escape_html(local_name(d)))).collect::<Vec<_>>().join(", "),
                comment = escape_html(class.comment),
            ));
        }
        body.push_str("</tbody>\n</table>\n");
    }

    // Properties
    if !module.properties.is_empty() {
        body.push_str("<h2>Properties</h2>\n<table>\n");
        body.push_str("<thead><tr><th>Name</th><th>Kind</th><th>Functional</th><th>Domain</th><th>Range</th><th>Comment</th></tr></thead>\n<tbody>\n");
        for prop in &module.properties {
            let local = local_name(prop.id);
            let kind = match prop.kind {
                PropertyKind::Datatype => "Datatype",
                PropertyKind::Object => "Object",
                PropertyKind::Annotation => "Annotation",
            };
            body.push_str(&format!(
                "<tr id=\"prop-{local}\"><td><code>{label}</code></td><td>{kind}</td><td>{functional}</td><td><code>{domain}</code></td><td><code>{range}</code></td><td>{comment}</td></tr>\n",
                local = escape_html(local),
                label = escape_html(prop.label),
                kind = kind,
                functional = prop.functional,
                domain = escape_html(prop.domain.map(local_name).unwrap_or("—")),
                range = escape_html(local_name(prop.range)),
                comment = escape_html(prop.comment),
            ));
        }
        body.push_str("</tbody>\n</table>\n");
    }

    // Individuals
    if !module.individuals.is_empty() {
        body.push_str("<h2>Named Individuals</h2>\n<table>\n");
        body.push_str(
            "<thead><tr><th>Name</th><th>Type</th><th>Comment</th></tr></thead>\n<tbody>\n",
        );
        for ind in &module.individuals {
            let local = local_name(ind.id);
            body.push_str(&format!(
                "<tr id=\"ind-{local}\"><td><code>{label}</code></td><td><code>{type_}</code></td><td>{comment}</td></tr>\n",
                local = escape_html(local),
                label = escape_html(ind.label),
                type_ = escape_html(local_name(ind.type_)),
                comment = escape_html(ind.comment),
            ));
            // Individual properties as a nested list
            if !ind.properties.is_empty() {
                body.push_str("<tr><td colspan=\"3\"><ul>");
                for (prop_iri, value) in ind.properties {
                    let value_html = render_individual_value(value);
                    body.push_str(&format!(
                        "<li><code>{}</code>: {}</li>",
                        escape_html(local_name(prop_iri)),
                        value_html
                    ));
                }
                body.push_str("</ul></td></tr>\n");
            }
        }
        body.push_str("</tbody>\n</table>\n");
    }

    // Related concepts (derived from the inverse of CONCEPT_RELATIONS)
    let related_slugs = crate::concepts::concepts_for_namespace(ns.prefix);
    if !related_slugs.is_empty() {
        body.push_str("<section class=\"related-section\">\n<h2>Related Concepts</h2>\n<ul class=\"related-list\">\n");
        for slug in &related_slugs {
            let title = concept_title(slug);
            body.push_str(&format!(
                "<li><a href=\"{bp}/concepts/{slug}.html\">{title}</a></li>\n",
            ));
        }
        body.push_str("</ul>\n</section>\n");
    }

    body
}

/// Renders an individual property value as HTML.
fn render_individual_value(value: &IndividualValue) -> String {
    match value {
        IndividualValue::Str(s) => escape_html(s),
        IndividualValue::Int(i) => i.to_string(),
        IndividualValue::Bool(b) => b.to_string(),
        IndividualValue::Float(x) => x.to_string(),
        IndividualValue::IriRef(iri) => {
            format!("<code>{}</code>", escape_html(local_name(iri)))
        }
        IndividualValue::List(items) => {
            let parts: Vec<String> = items
                .iter()
                .map(|i| format!("<code>{}</code>", escape_html(local_name(i))))
                .collect();
            format!("[{}]", parts.join(", "))
        }
    }
}

/// Renders the search page body with facet filters and kind badge documentation.
pub fn render_search_page(base_path: &str) -> String {
    let action = format!("{}/search.html", base_path);
    format!(
        "<h1>Search</h1>\n\
         <form role=\"search\" action=\"{action}\" method=\"get\">\n\
         <label for=\"search-input\">Search ontology terms</label>\n\
         <input type=\"search\" id=\"search-input\" name=\"q\" \
         placeholder=\"e.g. Ring, criticalIdentity, partition\u{2026}\" autocomplete=\"off\">\n\
         </form>\n\
         <details class=\"search-facets\">\n\
         <summary>Filter results</summary>\n\
         <fieldset>\n\
         <legend>Space</legend>\n\
         <label><input type=\"checkbox\" class=\"facet-space\" value=\"kernel\"> \
         <span class=\"badge badge-kernel\">kernel</span></label>\n\
         <label><input type=\"checkbox\" class=\"facet-space\" value=\"bridge\"> \
         <span class=\"badge badge-bridge\">bridge</span></label>\n\
         <label><input type=\"checkbox\" class=\"facet-space\" value=\"user\"> \
         <span class=\"badge badge-user\">user</span></label>\n\
         </fieldset>\n\
         <fieldset>\n\
         <legend>Kind</legend>\n\
         <label><input type=\"checkbox\" class=\"facet-kind\" value=\"class\"> \
         <span class=\"badge badge-class\">class</span></label>\n\
         <label><input type=\"checkbox\" class=\"facet-kind\" value=\"property\"> \
         <span class=\"badge badge-property\">property</span></label>\n\
         <label><input type=\"checkbox\" class=\"facet-kind\" value=\"individual\"> \
         <span class=\"badge badge-individual\">individual</span></label>\n\
         <label><input type=\"checkbox\" class=\"facet-kind\" value=\"namespace\"> \
         <span class=\"badge badge-namespace\">namespace</span></label>\n\
         </fieldset>\n\
         </details>\n\
         <ul id=\"search-results\" aria-live=\"polite\"></ul>",
        action = escape_html(&action),
    )
}

/// Renders the sitemap.xml content.
pub fn render_sitemap(base_url: &str, paths: &[String]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for path in paths {
        xml.push_str(&format!("  <url><loc>{}{}</loc></url>\n", base_url, path));
    }
    xml.push_str("</urlset>\n");
    xml
}

/// Renders the download page body.
pub fn render_download_page(base_path: &str) -> String {
    let json_url = format!("{base_path}/uor.foundation.jsonld");
    let ttl_url = format!("{base_path}/uor.foundation.ttl");
    let nt_url = format!("{base_path}/uor.foundation.nt");
    let owl_url = format!("{base_path}/uor.foundation.owl");
    let schema_url = format!("{base_path}/uor.foundation.schema.json");
    let shapes_url = format!("{base_path}/uor.shapes.ttl");
    let ebnf_url = format!("{base_path}/uor.term.ebnf");
    format!(
        "<h1>Download</h1>\n\
         <p>The complete UOR Foundation ontology is available in seven serialization formats. \
         All files are generated from the same authoritative source in \
         <code>spec/</code>.</p>\n\
         <table class=\"download-table\">\n\
         <thead><tr><th>Format</th><th>Description</th><th>Download</th></tr></thead>\n\
         <tbody>\n\
         <tr>\n\
         <td><span class=\"format-badge\">.jsonld</span></td>\n\
         <td>JSON-LD 1.1 — linked data with <code>@context</code> and <code>@graph</code></td>\n\
         <td><a href=\"{json_url}\">uor.foundation.jsonld</a></td>\n\
         </tr>\n\
         <tr>\n\
         <td><span class=\"format-badge\">.ttl</span></td>\n\
         <td>Turtle 1.1 — human-readable RDF serialization</td>\n\
         <td><a href=\"{ttl_url}\">uor.foundation.ttl</a></td>\n\
         </tr>\n\
         <tr>\n\
         <td><span class=\"format-badge\">.nt</span></td>\n\
         <td>N-Triples — line-oriented RDF for streaming and tooling</td>\n\
         <td><a href=\"{nt_url}\">uor.foundation.nt</a></td>\n\
         </tr>\n\
         <tr>\n\
         <td><span class=\"format-badge\">.owl</span></td>\n\
         <td>OWL 2 RDF/XML — ontology interchange for Protege and OWL reasoners</td>\n\
         <td><a href=\"{owl_url}\">uor.foundation.owl</a></td>\n\
         </tr>\n\
         <tr>\n\
         <td><span class=\"format-badge\">.schema.json</span></td>\n\
         <td>JSON Schema (Draft 2020-12) — typed code generation in 30+ languages</td>\n\
         <td><a href=\"{schema_url}\">uor.foundation.schema.json</a></td>\n\
         </tr>\n\
         <tr>\n\
         <td><span class=\"format-badge\">.shapes.ttl</span></td>\n\
         <td>SHACL Shapes — W3C validation constraints for RDF data</td>\n\
         <td><a href=\"{shapes_url}\">uor.shapes.ttl</a></td>\n\
         </tr>\n\
         <tr>\n\
         <td><span class=\"format-badge\">.ebnf</span></td>\n\
         <td>EBNF Grammar (ISO/IEC 14977) — UOR Term Language formal grammar</td>\n\
         <td><a href=\"{ebnf_url}\">uor.term.ebnf</a></td>\n\
         </tr>\n\
         </tbody>\n\
         </table>\n\
         <h2>Rust Crate</h2>\n\
         <p>The <code>uor-foundation</code> crate provides the ontology as typed Rust \
         traits and constants, suitable for <code>#![no_std]</code> environments.</p>\n\
         <p><a href=\"https://crates.io/crates/uor-foundation\" class=\"cta-primary\">View on crates.io</a></p>\n\
         <h2>Lean 4 Package</h2>\n\
         <p>The <code>uor</code> Lean 4 package provides the ontology as typed \
         structures and constants for formal verification and theorem proving.</p>\n\
         <p><a href=\"https://github.com/UOR-Foundation/UOR-Framework/tree/main/lean4\" class=\"cta-primary\">View on GitHub</a></p>"
    )
}

/// Renders the about page body from `content/about.md`.
///
/// Expands `{@count:}` and other directives in the markdown before rendering.
///
/// # Errors
///
/// Returns an error if `about.md` cannot be read.
pub fn render_about_page(
    content_dir: &std::path::Path,
    ontology: &Ontology,
    concept_list: &[crate::model::ConceptPage],
    base_path: &str,
) -> anyhow::Result<String> {
    let about_path = content_dir.join("about.md");
    let raw = std::fs::read_to_string(&about_path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", about_path.display(), e))?;
    let expanded = crate::concepts::expand_directives(&raw, ontology, concept_list, base_path);
    let content_html = crate::concepts::markdown_to_html(&expanded);
    let ns_count = uor_ontology::counts::NAMESPACES;
    Ok(format!(
        "{content_html}\n\
         <section class=\"about-links\">\n\
         <h2>Quick Links</h2>\n\
         <ul>\n\
         <li><a href=\"{base_path}/concepts/\">Concepts</a> \u{2014} \
         deep-dive explanations of core ideas</li>\n\
         <li><a href=\"{base_path}/pipeline/\">Pipeline</a> \u{2014} \
         the Define \u{2192} Resolve \u{2192} Certify stages</li>\n\
         <li><a href=\"{base_path}/namespaces/\">Namespaces</a> \u{2014} \
         browse the {ns_count} ontology namespaces</li>\n\
         <li><a href=\"{base_path}/download/\">Download</a> \u{2014} \
         serialization artifacts in 7 formats</li>\n\
         <li><a href=\"{base_path}/docs/overview.html\">Documentation</a> \u{2014} \
         full technical reference</li>\n\
         </ul>\n\
         </section>"
    ))
}

/// Renders the citation page body.
pub fn render_citation_page() -> String {
    "<h1>Citation</h1>\n\
     <p>If you use the UOR Framework in academic work, please cite it using the \
     metadata in <a href=\"https://github.com/UOR-Foundation/UOR-Framework/blob/main/CITATION.cff\">\
     <code>CITATION.cff</code></a> at the root of the repository.</p>\n\
     <h2>BibTeX</h2>\n\
     <pre><code>@software{uor-framework,\n\
     \x20 author       = {{The UOR Foundation}},\n\
     \x20 title        = {{UOR Framework}},\n\
     \x20 url          = {https://github.com/UOR-Foundation/UOR-Framework},\n\
     \x20 license      = {MIT},\n\
     \x20 abstract     = {A Rust workspace implementing the UOR Foundation\n\
     \x20                  ontology --- a mathematical framework for\n\
     \x20                  content-addressed, algebraically-structured\n\
     \x20                  object spaces.}\n\
     }</code></pre>\n\
     <h2>Zenodo</h2>\n\
     <p>Each tagged release is automatically archived by \
     <a href=\"https://zenodo.org/\">Zenodo</a> with a persistent DOI:</p>\n\
     <p><a href=\"https://doi.org/10.5281/zenodo.19068826\">\
     <img src=\"https://zenodo.org/badge/DOI/10.5281/zenodo.19068826.svg\" \
     alt=\"DOI\"></a></p>\n\
     <h2>GitHub</h2>\n\
     <p>GitHub renders a <strong>Cite this repository</strong> button on the \
     repository page using the <code>CITATION.cff</code> file.</p>"
        .to_string()
}

/// Renders the pipeline page body with stage sections and inline SVG.
/// Renders the pipeline page body.
///
/// If `narrative_html` is non-empty, it is injected between the heading and the
/// pipeline SVG diagram to provide narrative context from the merged PRISM concept.
pub fn render_pipeline_page(
    summaries: &[NamespaceSummary],
    base_path: &str,
    narrative_html: &str,
) -> String {
    use crate::pipeline::PRISM_STAGES;

    let pipeline_svg = crate::svg::render_prism_pipeline_svg(summaries);

    let narrative_block = if narrative_html.is_empty() {
        String::new()
    } else {
        format!(
            "<section class=\"pipeline-narrative\">\n{}\n</section>\n",
            narrative_html
        )
    };

    let mut body = format!(
        "<h1>The PRISM Pipeline</h1>\n\
         {narrative_block}\
         <p>The UOR Foundation resolution pipeline transforms a content-addressed reference \
         through three stages: <strong>Define</strong> (kernel-space declarations), \
         <strong>Resolve</strong> (bridge-space factorization), and \
         <strong>Certify</strong> (attestation). Each namespace belongs to exactly one stage.</p>\n\
         <figure class=\"diagram-container\" aria-label=\"PRISM pipeline diagram\">\n\
         <figcaption>PRISM pipeline — {n} stages across {ns} namespaces</figcaption>\n\
         {pipeline_svg}\n\
         </figure>\n",
        n = PRISM_STAGES.len(),
        ns = summaries.len(),
    );

    for (name, section_id, match_key, is_prefix) in PRISM_STAGES {
        let ns_in_stage: Vec<&NamespaceSummary> = summaries
            .iter()
            .filter(|s| {
                if *is_prefix {
                    s.prefix == *match_key
                } else {
                    s.space == *match_key
                }
            })
            .collect();

        let space_key = if *is_prefix { "cert" } else { *match_key };

        body.push_str(&format!(
            "<section class=\"stage-block stage-{name_lower}\" id=\"{section_id}\" \
             data-space=\"{space_key}\">\n\
             <span class=\"stage-label\">{name}</span>\n\
             <h2>{name}</h2>\n",
            name_lower = name.to_lowercase(),
        ));

        match *section_id {
            "stage-define" => body.push_str(
                "<p>The Define stage comprises kernel-space namespaces — the immutable \
                 algebraic substrate compiled into ROM. These namespaces declare the \
                 ring structure, schema vocabulary, and operation algebra that underpin \
                 all content addressing.</p>\n",
            ),
            "stage-resolve" => body.push_str(
                "<p>The Resolve stage comprises bridge-space namespaces — kernel-computed, \
                 user-consumed transformations. These namespaces apply queries, partitions, \
                 observables, homological analysis, derivation rules, and trace recording \
                 to produce certified results from kernel inputs.</p>\n",
            ),
            "stage-certify" => body.push_str(
                "<p>The Certify stage attests resolution results with verification hashes \
                 and replayable computation traces. The <code>cert</code> namespace issues \
                 certificates that bind identity proofs to specific Witt-level \
                 computations.</p>\n",
            ),
            _ => {}
        }

        // Concept links for this stage
        body.push_str(&stage_concept_links(section_id, base_path));

        if !ns_in_stage.is_empty() {
            body.push_str("<ul>\n");
            for ns in &ns_in_stage {
                let ns_url = format!("{base_path}/namespaces/{}/", ns.prefix);
                body.push_str(&format!(
                    "<li><a href=\"{ns_url}\"><code>{prefix}</code></a> — {label}: {comment}</li>\n",
                    prefix = escape_html(&ns.prefix),
                    label = escape_html(&ns.label),
                    comment = escape_html(&ns.comment),
                ));
            }
            body.push_str("</ul>\n");
        }

        body.push_str("</section>\n");
    }

    body
}

/// Renders the identities browser page body.
pub fn render_identities_page(ontology: &Ontology, base_path: &str) -> String {
    let identity_count = uor_ontology::counts::IDENTITY_COUNT;

    let dist_svg = crate::svg::render_identity_distribution_svg(ontology);

    let ring_url = format!("{base_path}/concepts/ring.html");
    let proof_url = format!("{base_path}/concepts/proof-system.html");

    let mut body = format!(
        "<h1>Algebraic Identities</h1>\n\
         <div class=\"identities-intro\">\n\
         <p>An <strong>algebraic identity</strong> is a named equation that holds over the ring \
         substrate Z/(2^n)Z. Each identity is an <code>op:Identity</code> individual in the \
         ontology, paired with a <code>proof:AxiomaticDerivation</code> or \
         <code>proof:ComputationCertificate</code> that certifies it.</p>\n\
         <p>Identities are classified by <strong>verification domain</strong> \u{2014} the mathematical \
         discipline used to prove them. Domains include Enumerative (exhaustive check at Q0), \
         Algebraic (ring-theoretic derivation), Geometric (metric-space arguments), and others. \
         The chart below shows the distribution across all {identity_count} identities.</p>\n\
         <p>Each identity also has a <strong>validity scope</strong>: Universal (valid at all \
         Witt levels), ParametricLower (valid at level \u{2265} k), ParametricRange (valid in \
         [k_min, k_max]), or LevelSpecific (valid at exactly one level). \
         See <a href=\"{ring_url}\">The Ring Substrate</a> for the \
         ring foundation and <a href=\"{proof_url}\">Proofs, \
         Derivations &amp; Traces</a> for the certification pipeline.</p>\n\
         </div>\n\
         <figure class=\"diagram-container identity-distribution\" \
         aria-label=\"Identity distribution by verification domain\">\n\
         <figcaption>Identity count by verification domain</figcaption>\n\
         {dist_svg}\n\
         </figure>\n\
         <table id=\"identity-table\">\n\
         <thead>\n\
         <tr>\n\
         <th>ID</th><th>Label</th><th>Domain</th><th>Comment</th>\n\
         </tr>\n\
         </thead>\n\
         <tbody>\n"
    );

    // Find verification domain IRIs → labels for lookup
    let domain_map: std::collections::HashMap<&str, &str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|i| i.type_.ends_with("VerificationDomain"))
        .map(|i| (i.id, i.label))
        .collect();

    for ind in ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|i| i.type_.ends_with("Identity"))
    {
        let local_id = ind.id.rsplit('/').next().unwrap_or(ind.id);
        let ns_prefix = ind
            .id
            .trim_start_matches("https://uor.foundation/")
            .split('/')
            .next()
            .unwrap_or("");
        let ns_url = format!("{base_path}/namespaces/{ns_prefix}/");

        let domains: Vec<&str> = ind
            .properties
            .iter()
            .filter(|(k, _)| k.ends_with("verificationDomain"))
            .filter_map(|(_, v)| {
                if let IndividualValue::IriRef(iri) = v {
                    domain_map.get(*iri).copied()
                } else {
                    None
                }
            })
            .collect();

        let domain_str = if domains.is_empty() {
            "—".to_string()
        } else {
            domains
                .iter()
                .map(|d| escape_html(d))
                .collect::<Vec<_>>()
                .join(", ")
        };

        body.push_str(&format!(
            "<tr class=\"identity-row\" id=\"id-{local}\">\n\
             <td><a href=\"{ns_url}#ind-{local}\"><code>{local}</code></a></td>\n\
             <td>{label}</td>\n\
             <td>{domain_str}</td>\n\
             <td>{comment}</td>\n\
             </tr>\n",
            local = escape_html(local_id),
            label = escape_html(ind.label),
            comment = escape_html(ind.comment),
        ));
    }

    body.push_str(
        "</tbody>\n</table>\n\
         <script>\n\
         (function(){\n\
           var table=document.getElementById('identity-table');\n\
           if(!table)return;\n\
           var headers=table.querySelectorAll('thead th');\n\
           headers.forEach(function(th,col){\n\
             th.style.cursor='pointer';\n\
             th.addEventListener('click',function(){\n\
               var tbody=table.querySelector('tbody');\n\
               var rows=Array.from(tbody.querySelectorAll('tr'));\n\
               var asc=th.dataset.sort!=='asc';\n\
               rows.sort(function(a,b){\n\
                 var x=a.cells[col]?a.cells[col].textContent:'';\n\
                 var y=b.cells[col]?b.cells[col].textContent:'';\n\
                 return asc?x.localeCompare(y):y.localeCompare(x);\n\
               });\n\
               rows.forEach(function(r){tbody.appendChild(r);});\n\
               headers.forEach(function(h){delete h.dataset.sort;});\n\
               th.dataset.sort=asc?'asc':'desc';\n\
             });\n\
           });\n\
         })();\n\
         </script>",
    );

    body
}

/// Renders the explore page body with namespace dependency graph and interactive explorer.
pub fn render_explore(
    ontology: &Ontology,
    summaries: &[NamespaceSummary],
    base_path: &str,
) -> String {
    use crate::svg::space_display_name;
    use uor_ontology::model::Space;

    let dep_svg = crate::svg::render_namespace_dependency_graph_svg(ontology, base_path);
    let explore_data = crate::extractor::generate_explore_data(summaries);

    let pipeline_url = format!("{base_path}/pipeline/");
    let ns_list_url = format!("{base_path}/namespaces/");

    let mut body = format!(
        "<h1>Explore the Ontology</h1>\n\
         <p>The UOR Foundation ontology comprises {ns} namespaces organized into three \
         space classifications: Kernel (immutable substrate), Bridge (computed transforms), \
         and User (runtime parameterization). The diagram below shows import dependencies.</p>\n\
         <p>Each namespace imports other namespaces for its domain and range declarations, \
         forming a directed acyclic graph. The assembly order (u \u{2192} schema \u{2192} op \
         \u{2192} \u{2026} \u{2192} state) ensures no circular dependencies. Expand a \
         namespace below for a summary, or visit its \
         <a href=\"{ns_list_url}\">full reference page</a>. \
         For a conceptual overview, start with \
         <a href=\"{pipeline_url}\">The PRISM Pipeline</a>.</p>\n\
         <figure class=\"diagram-container\" aria-label=\"Namespace dependency graph\">\n\
         <figcaption>Namespace import dependencies (assembly order, left to right)</figcaption>\n\
         {dep_svg}\n\
         </figure>\n\
         <div id=\"ontology-explorer\">\n",
        ns = summaries.len(),
    );

    for space in &[Space::Kernel, Space::Bridge, Space::User] {
        let space_str = format!("{:?}", space).to_lowercase();
        let space_name = space_display_name(space);
        let ns_in_space: Vec<&NamespaceSummary> =
            summaries.iter().filter(|s| s.space == space_str).collect();

        body.push_str(&format!(
            "<section class=\"space-section\">\n\
             <h2 data-space=\"{space_str}\">{space_name} Space</h2>\n"
        ));

        for ns in &ns_in_space {
            let ns_url = format!("{base_path}/namespaces/{}/", ns.prefix);
            body.push_str(&format!(
                "<details class=\"ns-detail\">\n\
                 <summary><code>{prefix}</code> — {label}</summary>\n\
                 <div class=\"ns-detail-body\">\n\
                 <p>{comment}</p>\n\
                 <dl>\n\
                 <dt>Classes</dt><dd>{classes}</dd>\n\
                 <dt>Properties</dt><dd>{props}</dd>\n\
                 <dt>Individuals</dt><dd>{inds}</dd>\n\
                 </dl>\n\
                 <p><a href=\"{ns_url}\">View full namespace documentation</a></p>\n\
                 </div>\n\
                 </details>\n",
                prefix = escape_html(&ns.prefix),
                label = escape_html(&ns.label),
                comment = escape_html(&ns.comment),
                classes = ns.class_count,
                props = ns.property_count,
                inds = ns.individual_count,
            ));
        }

        body.push_str("</section>\n");
    }

    body.push_str("</div>\n");

    // Inline JS with explore data for progressive enhancement
    body.push_str(&format!("<script>\n{explore_data}\n</script>\n"));

    body
}

/// Suggested reading order: `(slug, title, space)`.
/// Renders the Learn landing page.
///
/// Provides an orientation page for first-time visitors with persona-aware
/// reading paths, a concept overview, and links to the pipeline and reference.
pub fn render_learn_landing(
    concepts: &[ConceptPage],
    summaries: &[NamespaceSummary],
    base_path: &str,
) -> String {
    let pipeline_svg = crate::svg::render_prism_pipeline_svg(summaries);
    let concepts_url = format!("{base_path}/concepts");
    let pipeline_url = format!("{base_path}/pipeline/");
    let ns_url = format!("{base_path}/namespaces/");
    let identities_url = format!("{base_path}/identities/");
    let explore_url = format!("{base_path}/explore/");
    let overview_url = format!("{base_path}/docs/overview.html");
    let arch_url = format!("{base_path}/docs/architecture.html");
    let download_url = format!("{base_path}/download/");

    let mut concept_cards = String::new();
    for c in concepts {
        concept_cards.push_str(&format!(
            "<li><a href=\"{url}\"><span class=\"badge badge-{space}\">{space}</span> \
             {title}</a> — {desc}</li>\n",
            url = escape_html(&c.url),
            space = escape_html(&c.space),
            title = escape_html(&c.title),
            desc = escape_html(&c.description),
        ));
    }

    format!(
        r#"<section class="learn-hero">
<h1>Understanding UOR</h1>
<p>The Universal Object Reference is a formal ontology built on ring theory.
Every computable object lives in the ring Z/(2<sup>n</sup>)Z &mdash; integers modulo a
power of two. The framework specifies how objects are uniquely identified
(content-addressed), decomposed (factorized under the dihedral group), and
verified (certified by algebraic proofs).</p>
</section>

<section class="reading-guide" aria-labelledby="paths-heading">
<h2 id="paths-heading">Choose Your Path</h2>
<p>The ontology spans algebraic foundations, computational pipelines, and formal
verification. Start with the path closest to your background:</p>
<div class="persona-grid">
<article class="persona-card pathway-kernel">
<h3>For Mathematicians</h3>
<p>Begin with the algebraic substrate and build up to the full proof system.</p>
<ol>
<li><a href="{concepts_url}/ring.html">The Ring Substrate</a></li>
<li><a href="{concepts_url}/witt-levels.html">Witt Levels</a></li>
<li><a href="{identities_url}">Algebraic Identities</a></li>
<li><a href="{concepts_url}/homology.html">Homological Analysis</a></li>
<li><a href="{pipeline_url}">The PRISM Pipeline</a></li>
</ol>
</article>
<article class="persona-card pathway-bridge">
<h3>For Engineers</h3>
<p>Understand the pipeline, then dive into serialization formats and tooling.</p>
<ol>
<li><a href="{pipeline_url}">The PRISM Pipeline</a></li>
<li><a href="{concepts_url}/content-addressing.html">Content Addressing</a></li>
<li><a href="{download_url}">Download Artifacts</a></li>
<li><a href="{ns_url}">Browse Namespaces</a></li>
</ol>
</article>
<article class="persona-card pathway-user">
<h3>For Ontologists</h3>
<p>Start with the specification, then explore the formal structure.</p>
<ol>
<li><a href="{overview_url}">Overview</a></li>
<li><a href="{arch_url}">Architecture</a></li>
<li><a href="{ns_url}">All Namespaces</a></li>
<li><a href="{concepts_url}/site.html">Site Bundles</a></li>
<li><a href="{concepts_url}/partition.html">Partitions</a></li>
</ol>
</article>
</div>
</section>

<section class="pipeline-overview">
<h2>The Resolution Pipeline</h2>
<figure class="diagram-container" aria-label="PRISM pipeline overview">
{pipeline_svg}
</figure>
<p>Every resolution passes through three stages:
<strong>Define</strong> (kernel-space declarations),
<strong>Resolve</strong> (bridge-space factorization), and
<strong>Certify</strong> (attestation).
<a href="{pipeline_url}">Read the full pipeline guide &rarr;</a></p>
</section>

<section class="concept-overview">
<h2>Core Concepts</h2>
<figure class="diagram-container" aria-label="Concept relationship map">
{concept_map_svg}
<figcaption>Concept relationships — kernel (purple), bridge (green), user (cyan)</figcaption>
</figure>
<p>Deep-dive explanations connecting formal ontology definitions to the
intuitions behind the framework:</p>
<ul class="concept-list">
{concept_cards}
</ul>
<p><a href="{concepts_url}/">View all concepts &rarr;</a></p>
</section>

<section class="reference-links">
<h2>Reference</h2>
<ul>
<li><a href="{ns_url}">Namespaces</a> &mdash; auto-generated reference for every namespace</li>
<li><a href="{identities_url}">Identities</a> &mdash; searchable browser for algebraic identities</li>
<li><a href="{explore_url}">Dependency Graph</a> &mdash; interactive namespace dependency explorer</li>
</ul>
</section>"#,
        pipeline_svg = pipeline_svg,
        concepts_url = escape_html(&concepts_url),
        pipeline_url = escape_html(&pipeline_url),
        ns_url = escape_html(&ns_url),
        identities_url = escape_html(&identities_url),
        explore_url = escape_html(&explore_url),
        overview_url = escape_html(&overview_url),
        arch_url = escape_html(&arch_url),
        download_url = escape_html(&download_url),
        concept_cards = concept_cards,
        concept_map_svg = crate::svg::render_concept_map_svg(concepts, base_path),
    )
}

const READING_ORDER: &[(&str, &str, &str)] = &[
    ("ring", "The Ring Substrate", "kernel"),
    ("witt-levels", "Witt Levels", "kernel"),
    ("content-addressing", "Content Addressing", "kernel"),
    ("partition", "The Partition Decomposition", "bridge"),
    ("site", "Site Bundle Semantics", "bridge"),
    ("resolution", "Resolution &amp; Queries", "bridge"),
    ("observables", "Observables &amp; Measurement", "bridge"),
    ("proof-system", "Proofs, Derivations &amp; Traces", "bridge"),
    ("homology", "Homological Analysis", "bridge"),
    ("certification", "Certification and Verification", "bridge"),
    ("morphisms", "Morphisms and Transformations", "user"),
    (
        "state-management",
        "State, Sessions, and Accumulation",
        "user",
    ),
];

/// Renders the concepts index page.
pub fn render_concepts_index(concepts: &[ConceptPage], base_path: &str) -> String {
    let concept_count = concepts.len();
    let pipeline_url = format!("{base_path}/pipeline/");

    // Reading pathway
    let mut path_items = String::new();
    for (i, (slug, title, space)) in READING_ORDER.iter().enumerate() {
        path_items.push_str(&format!(
            "<li><a href=\"{base_path}/concepts/{slug}.html\">\
             <span class=\"rp-number\">{num}</span>\
             <span class=\"rp-title\">{title}</span>\
             <span class=\"badge badge-{space} rp-badge\">{space}</span>\
             </a></li>\n",
            num = i + 1,
        ));
    }

    let mut cards = String::new();
    for concept in concepts {
        cards.push_str(&format!(
            "<article class=\"concept-card\">\n\
             <h3><a href=\"{url}\">{title}</a></h3>\n\
             <p>{desc}</p>\n\
             </article>\n",
            url = escape_html(&concept.url),
            title = escape_html(&concept.title),
            desc = escape_html(&concept.description),
        ));
    }

    format!(
        "<h1>Concepts</h1>\n\
         <p>Deep-dive explanations of the core mathematical and architectural concepts \
         in the UOR Foundation. Each page connects the formal ontology definitions to \
         the intuitions behind the <a href=\"{pipeline_url}\">PRISM pipeline</a>.</p>\n\
         <section class=\"reading-path\" aria-labelledby=\"reading-path-heading\">\n\
         <h2 id=\"reading-path-heading\">Suggested Reading Order</h2>\n\
         <p>New to UOR? Follow this path through the core concepts, from the algebraic \
         foundation to the full certification pipeline:</p>\n\
         <ol class=\"reading-path-list\">\n\
         {path_items}\
         </ol>\n\
         </section>\n\
         <h2>All Concepts</h2>\n\
         <div class=\"concept-grid\">\n\
         {cards}\n\
         </div>\n\
         <p>Found {concept_count} concept page{pl}.</p>",
        pl = if concept_count == 1 { "" } else { "s" },
    )
}

/// Renders a concept page body wrapping rendered markdown content.
///
/// If `extra_svg` is provided, it is injected after the first `<h1>` heading.
/// If related namespaces or concepts are provided, a "Related" section is appended.
pub fn render_concept_page_body(
    title: &str,
    content_html: &str,
    extra_svg: Option<&str>,
    related_ns: &[(&str, &str)],
    related_concepts: &[(&str, &str)],
    base_path: &str,
) -> String {
    let svg_block = extra_svg
        .map(|svg| {
            format!(
                "<figure class=\"diagram-container\" aria-label=\"{title} diagram\">\n\
                 {svg}\n\
                 </figure>\n",
                title = escape_html(title),
            )
        })
        .unwrap_or_default();

    // Inject SVG after the first <h1> if present, otherwise prepend
    let mut result = if content_html.contains("</h1>") {
        let split_pos = content_html.find("</h1>").map(|p| p + 5).unwrap_or(0);
        let (before, after) = content_html.split_at(split_pos);
        format!("{before}\n{svg_block}{after}")
    } else {
        format!("{svg_block}{content_html}")
    };

    // Related section
    if !related_ns.is_empty() || !related_concepts.is_empty() {
        result.push_str(
            "<section class=\"related-section\" aria-labelledby=\"related-heading\">\n\
             <h2 id=\"related-heading\">Related</h2>\n\
             <div class=\"related-columns\">\n",
        );

        if !related_ns.is_empty() {
            result.push_str(
                "<div class=\"related-col\">\n<h3>Namespaces</h3>\n<ul class=\"related-list\">\n",
            );
            for (prefix, label) in related_ns {
                result.push_str(&format!(
                    "<li><a href=\"{base_path}/namespaces/{prefix}/\"><code>{prefix}</code> \u{2014} {label}</a></li>\n",
                    prefix = escape_html(prefix),
                    label = escape_html(label),
                ));
            }
            result.push_str("</ul>\n</div>\n");
        }

        if !related_concepts.is_empty() {
            result.push_str(
                "<div class=\"related-col\">\n<h3>Concepts</h3>\n<ul class=\"related-list\">\n",
            );
            for (slug, c_title) in related_concepts {
                result.push_str(&format!(
                    "<li><a href=\"{base_path}/concepts/{slug}.html\">{title}</a></li>\n",
                    slug = escape_html(slug),
                    title = escape_html(c_title),
                ));
            }
            result.push_str("</ul>\n</div>\n");
        }

        result.push_str("</div>\n</section>\n");
    }

    result
}

/// Returns a contextual paragraph for a namespace page based on its space and prefix.
fn space_context_html(space: &uor_ontology::model::Space, prefix: &str, base_path: &str) -> String {
    use uor_ontology::model::Space;
    let (stage, stage_id, stage_desc) = match space {
        Space::Kernel => (
            "Define",
            "stage-define",
            "the immutable algebraic substrate \u{2014} ring structure, schema vocabulary, \
             and operation algebra",
        ),
        Space::Bridge => (
            "Resolve",
            "stage-resolve",
            "the resolution infrastructure \u{2014} queries, partitions, observables, proofs, \
             derivations, and traces that transform inputs into certified results",
        ),
        Space::User => (
            "Apply",
            "stage-resolve",
            "the application layer \u{2014} types, morphisms, and state that parameterize \
             the resolution pipeline",
        ),
    };

    let space_str = format!("{:?}", space).to_lowercase();

    let concept_link = match prefix {
        "u" => Some(("content-addressing", "Content Addressing")),
        "schema" => Some(("ring", "The Ring Substrate")),
        "op" => Some(("witt-levels", "Witt Levels")),
        "query" | "resolver" => Some(("resolution", "Resolution &amp; Queries")),
        "partition" => Some(("partition", "The Partition Decomposition")),
        "observable" => Some(("observables", "Observables &amp; Measurement")),
        "homology" | "cohomology" => Some(("homology", "Homological Analysis")),
        "proof" | "derivation" | "trace" => {
            Some(("proof-system", "Proofs, Derivations &amp; Traces"))
        }
        "cert" => Some(("proof-system", "Proofs, Derivations &amp; Traces")),
        "type" | "morphism" | "state" => Some(("site", "Site Bundle Semantics")),
        _ => None,
    };

    let concept_html = concept_link
        .map(|(slug, title)| {
            format!(" \u{00b7} <a href=\"{base_path}/concepts/{slug}.html\">{title}</a>")
        })
        .unwrap_or_default();

    format!(
        "<div class=\"ns-context ns-context-{space_str}\">\n\
         <p>This is a <strong>{space_str}-space</strong> namespace in the \
         <a href=\"{base_path}/pipeline/#{stage_id}\">{stage}</a> stage of the \
         PRISM pipeline. It provides {stage_desc}.</p>\n\
         <p>Learn more: <a href=\"{base_path}/pipeline/\">Pipeline Overview</a>{concept_html}</p>\n\
         </div>\n"
    )
}

/// Returns concept link HTML for a pipeline stage.
fn stage_concept_links(section_id: &str, base_path: &str) -> String {
    let links: &[(&str, &str)] = match section_id {
        "stage-define" => &[
            ("ring", "The Ring Substrate"),
            ("witt-levels", "Witt Levels"),
            ("content-addressing", "Content Addressing"),
        ],
        "stage-resolve" => &[
            ("resolution", "Resolution &amp; Queries"),
            ("partition", "The Partition Decomposition"),
            ("observables", "Observables &amp; Measurement"),
            ("homology", "Homological Analysis"),
            ("site", "Site Bundle Semantics"),
        ],
        "stage-certify" => &[("proof-system", "Proofs, Derivations &amp; Traces")],
        _ => &[],
    };

    if links.is_empty() {
        return String::new();
    }

    let items: Vec<String> = links
        .iter()
        .map(|(slug, title)| format!("<a href=\"{base_path}/concepts/{slug}.html\">{title}</a>"))
        .collect();

    format!(
        "<p class=\"stage-concepts\">Related concepts: {}</p>\n",
        items.join(" \u{00b7} ")
    )
}

/// Looks up the proper display title for a concept page slug.
///
/// Falls back to a title-cased version of the slug if not found in `READING_ORDER`.
fn concept_title(slug: &str) -> String {
    READING_ORDER
        .iter()
        .find(|(s, _, _)| *s == slug)
        .map(|(_, title, _)| (*title).to_string())
        .unwrap_or_else(|| slug.replace('-', " "))
}

/// Extracts the local name from an IRI.
///
/// Handles both `/`-separated paths and `#`-fragment IRIs (OWL, XSD, RDF).
fn local_name(iri: &str) -> &str {
    let after_slash = iri.rsplit('/').next().unwrap_or(iri);
    after_slash.rsplit('#').next().unwrap_or(after_slash)
}
