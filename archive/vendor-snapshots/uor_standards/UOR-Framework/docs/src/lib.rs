//! UOR Foundation documentation generator.
//!
//! Generates verified HTML documentation from the UOR spec and content files.
//! Every ontology reference in prose (`{@class}`, `{@prop}`, `{@ind}`) is
//! validated against `uor_ontology::Ontology::full()` at build time.
//!
//! # Entry Points
//!
//! ```no_run
//! use std::path::PathBuf;
//! use uor_docs::generate;
//!
//! let out = PathBuf::from("public/docs");
//! let readme = PathBuf::from("README.md");
//! generate(&out, &readme).expect("Documentation generation failed");
//! ```
//!
//! # Structure
//!
//! ```text
//! public/docs/
//!   index.html              ← Ontology inventory table
//!   namespaces/
//!     u.html                ← Auto-generated from spec (100% accurate)
//!     schema.html
//!     ... (33 total)
//!   concepts/
//!     ring.html
//!     content-addressing.html
//!     ... (30 pages)
//!   guides/
//!     implementing-prism.html
//!     conformance.html
//!     contributing.html
//! ```

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

pub mod extractor;
pub mod linker;
pub mod nav;
pub mod renderer;
pub mod verifier;
pub mod writer;

use std::path::Path;

use anyhow::Result;

use extractor::OntologyIndex;
use renderer::{escape_html, render_docs_page};
use uor_ontology::{Individual, IndividualValue, NamespaceModule, Ontology, PropertyKind};

/// Generates all documentation artifacts.
///
/// Writes to `out_dir` (HTML docs) and `readme_path` (machine-generated README).
///
/// # Errors
///
/// Returns an error if content verification fails or any file cannot be written.
pub fn generate(out_dir: &Path, readme_path: &Path) -> Result<()> {
    let index = OntologyIndex::from_spec();

    let base_path = std::env::var("PUBLIC_BASE_PATH").unwrap_or_default();
    let base_path = base_path.trim_end_matches('/');

    // Verify prose content references (if content/ dir exists alongside this crate)
    let content_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("content");
    verifier::verify_content(&content_dir)?;

    // Discover every authored concept page so the index + sidebar are generated
    // from the source of truth (never hand-maintained, never out of sync).
    let concept_links = collect_concept_links(&content_dir.join("concepts"));

    // Every namespace (assembly order) so the sidebar lists all of them — the
    // previous hardcoded sidebar exposed only 16 of the 34.
    let ontology = Ontology::full();
    let namespace_prefixes: Vec<&str> = ontology
        .namespaces
        .iter()
        .map(|m| m.namespace.prefix)
        .collect();

    let docs_current = format!("{base_path}/docs/");
    let site_nav_html = nav::render_nav_bootstrap(&nav::build_nav(base_path), &docs_current);
    let docs_nav_html = build_docs_sidebar_html(base_path, &namespace_prefixes, &concept_links);

    // Generate index page
    let index_html = generate_index_page(&index, &site_nav_html, &docs_nav_html, base_path);
    writer::write_html(&out_dir.join("index.html"), &index_html)?;

    // Generate per-namespace reference pages (100% from spec)
    for module in &ontology.namespaces {
        let html = generate_namespace_page(module, &site_nav_html, &docs_nav_html, base_path);
        let path = out_dir
            .join("namespaces")
            .join(format!("{}.html", module.namespace.prefix));
        writer::write_html(&path, &html)?;
    }

    // Generate concept pages from content/
    generate_content_pages(
        &content_dir.join("concepts"),
        &out_dir.join("concepts"),
        &index,
        &site_nav_html,
        &docs_nav_html,
        base_path,
    )?;

    // Generate concepts index page — listing EVERY authored concept page,
    // built from the discovered `content/concepts/*.md` files (ADR: no
    // hand-maintained list, so a new concept is exposed the moment it is added).
    let concepts_index = render_docs_page(
        "Concepts",
        &render_concept_index_body(&concept_links),
        &site_nav_html,
        &docs_nav_html,
        &format!(
            r#"<a href="{base_path}/">Home</a> › <a href="{base_path}/docs/index.html">Docs</a> › Concepts"#
        ),
        base_path,
    );
    writer::write_html(
        &out_dir.join("concepts").join("index.html"),
        &concepts_index,
    )?;

    // Generate guide pages from content/
    generate_content_pages(
        &content_dir.join("guides"),
        &out_dir.join("guides"),
        &index,
        &site_nav_html,
        &docs_nav_html,
        base_path,
    )?;

    // Generate guides index page
    let guides_index = render_docs_page(
        "Guides",
        r#"<h1>Guides</h1>
<p>How-to guides for working with and implementing the UOR Framework.</p>
<ul>
<li><a href="implementing-prism.html">Implementing PRISM — Build a UOR-compliant resolver</a></li>
<li><a href="conformance.html">Conformance — Validate your implementation</a></li>
<li><a href="contributing.html">Contributing — How to contribute to UOR</a></li>
<li><a href="psi-pipeline.html">ψ Pipeline — Structural reasoning via homology and cohomology</a></li>
<li><a href="type-synthesis.html">Type Synthesis — Construct CompleteTypes from topological goals</a></li>
<li><a href="incremental-completeness.html">Incremental Completeness — Lift CompleteTypes across Witt levels</a></li>
<li><a href="homotopy-pipeline.html">Homotopy Pipeline — Extended psi pipeline: psi_7 through psi_9</a></li>
<li><a href="deformation-analysis.html">Deformation Analysis — ModuliResolver and stratification records</a></li>
</ul>"#,
        &site_nav_html,
        &docs_nav_html,
        &format!(
            r#"<a href="{base_path}/">Home</a> › <a href="{base_path}/docs/index.html">Docs</a> › Guides"#
        ),
        base_path,
    );
    writer::write_html(&out_dir.join("guides").join("index.html"), &guides_index)?;

    // Generate overview and architecture pages
    generate_single_page(
        &content_dir.join("overview.md"),
        &out_dir.join("overview.html"),
        "Overview",
        &index,
        &site_nav_html,
        &docs_nav_html,
        base_path,
    )?;
    generate_single_page(
        &content_dir.join("architecture.md"),
        &out_dir.join("architecture.html"),
        "Architecture",
        &index,
        &site_nav_html,
        &docs_nav_html,
        base_path,
    )?;

    // Generate machine-generated README.md
    let readme_content = generate_readme(ontology);
    writer::write_text(readme_path, &readme_content)?;

    Ok(())
}

/// Generates the docs-specific sidebar navigation tree. The Namespaces and
/// Concepts groups are built from the ontology and the discovered concept
/// pages, so every namespace and every authored concept appears (the previous
/// hand-maintained lists silently omitted entries).
fn build_docs_sidebar_html(
    base_path: &str,
    namespace_prefixes: &[&str],
    concepts: &[ConceptLink],
) -> String {
    let namespaces_html = namespace_prefixes
        .iter()
        .map(|prefix| {
            format!(
                r#"<li><a href="{base_path}/docs/namespaces/{prefix}.html">{prefix}</a></li>"#,
                prefix = escape_html(prefix),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let concepts_html = concepts
        .iter()
        .map(|c| {
            format!(
                r#"<li><a href="{base_path}/docs/concepts/{slug}.html">{title}</a></li>"#,
                slug = escape_html(&c.slug),
                title = escape_html(&c.title),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<ul>
<li><a href="{base_path}/docs/index.html">Documentation</a></li>
<li><a href="{base_path}/docs/overview.html">Overview</a></li>
<li><a href="{base_path}/docs/architecture.html">Architecture</a></li>
<li class="nav-group"><span>Namespaces</span>
<ul>
{namespaces_html}
</ul>
</li>
<li class="nav-group"><span>Concepts</span>
<ul>
{concepts_html}
</ul>
</li>
<li class="nav-group"><span>Guides</span>
<ul>
<li><a href="{base_path}/docs/guides/implementing-prism.html">Implementing PRISM</a></li>
<li><a href="{base_path}/docs/guides/conformance.html">Conformance</a></li>
<li><a href="{base_path}/docs/guides/contributing.html">Contributing</a></li>
<li><a href="{base_path}/docs/guides/psi-pipeline.html">ψ Pipeline</a></li>
<li><a href="{base_path}/docs/guides/type-synthesis.html">Type Synthesis</a></li>
<li><a href="{base_path}/docs/guides/incremental-completeness.html">Incremental Completeness</a></li>
<li><a href="{base_path}/docs/guides/homotopy-pipeline.html">Homotopy Pipeline</a></li>
<li><a href="{base_path}/docs/guides/deformation-analysis.html">Deformation Analysis</a></li>
</ul>
</li>
<li><a href="{base_path}/search.html">Search</a></li>
</ul>"#
    )
}

/// Generates the ontology inventory index page.
fn generate_index_page(
    index: &OntologyIndex,
    site_nav_html: &str,
    docs_nav_html: &str,
    base_path: &str,
) -> String {
    let mut rows = String::new();
    for module in &index.modules {
        let ns = &module.namespace;
        rows.push_str(&format!(
            "<tr><td><a href=\"namespaces/{prefix}.html\">{prefix}</a></td><td>{label}</td><td>{classes}</td><td>{props}</td><td>{inds}</td><td>{space}</td></tr>\n",
            prefix = escape_html(ns.prefix),
            label = escape_html(ns.label),
            classes = module.classes.len(),
            props = module.properties.len(),
            inds = module.individuals.len(),
            space = format!("{:?}", ns.space).to_lowercase(),
        ));
    }

    let content = format!(
        r#"<h1>UOR Foundation Ontology</h1>
<p>Version {version} — {ns} namespaces, {classes} classes, {props} properties, {inds} named individuals.</p>
<table>
<thead>
<tr><th>Prefix</th><th>Label</th><th>Classes</th><th>Properties</th><th>Individuals</th><th>Space</th></tr>
</thead>
<tbody>
{rows}
</tbody>
</table>"#,
        version = index.version,
        ns = index.modules.len(),
        classes = index.classes.len(),
        props = index.properties.len(),
        inds = index.individuals.len(),
    );

    render_docs_page(
        "Documentation Index",
        &content,
        site_nav_html,
        docs_nav_html,
        &format!(r#"<a href="{base_path}/">Home</a> › Documentation"#),
        base_path,
    )
}

/// Generates a namespace reference page from the spec (100% auto-generated).
fn generate_namespace_page(
    module: &NamespaceModule,
    site_nav_html: &str,
    docs_nav_html: &str,
    base_path: &str,
) -> String {
    let ns = &module.namespace;
    let mut content = format!(
        r#"<h1>{label}</h1>
<dl>
<dt>IRI</dt><dd><code>{iri}</code></dd>
<dt>Prefix</dt><dd><code>{prefix}:</code></dd>
<dt>Space</dt><dd>{space}</dd>
<dt>Comment</dt><dd>{comment}</dd>
</dl>
"#,
        label = escape_html(ns.label),
        iri = escape_html(ns.iri),
        prefix = escape_html(ns.prefix),
        space = format!("{:?}", ns.space).to_lowercase(),
        comment = escape_html(ns.comment),
    );

    // Imports
    if !ns.imports.is_empty() {
        content.push_str("<h2>Imports</h2><ul>\n");
        for imp in ns.imports {
            content.push_str(&format!("<li><code>{}</code></li>\n", escape_html(imp)));
        }
        content.push_str("</ul>\n");
    }

    // Classes
    if !module.classes.is_empty() {
        content.push_str("<h2>Classes</h2>\n<table>\n<thead><tr><th>Name</th><th>IRI</th><th>Subclass Of</th><th>Disjoint With</th><th>Comment</th></tr></thead>\n<tbody>\n");
        for class in &module.classes {
            content.push_str(&format!(
                "<tr id=\"{id_fragment}\"><td>{label}</td><td><code>{id}</code></td><td>{parents}</td><td>{disjoint}</td><td>{comment}</td></tr>\n",
                id_fragment = escape_html(&linker::fragment_from_iri(class.id)),
                label = escape_html(class.label),
                id = escape_html(class.id),
                parents = class.subclass_of.iter().map(|p| format!("<code>{}</code>", escape_html(p))).collect::<Vec<_>>().join(", "),
                disjoint = class.disjoint_with.iter().map(|d| format!("<code>{}</code>", escape_html(d))).collect::<Vec<_>>().join(", "),
                comment = escape_html(class.comment),
            ));
        }
        content.push_str("</tbody>\n</table>\n");
    }

    // Properties
    if !module.properties.is_empty() {
        content.push_str("<h2>Properties</h2>\n<table>\n<thead><tr><th>Name</th><th>Kind</th><th>Functional</th><th>Domain</th><th>Range</th><th>Comment</th></tr></thead>\n<tbody>\n");
        for prop in &module.properties {
            let kind = match prop.kind {
                PropertyKind::Datatype => "Datatype",
                PropertyKind::Object => "Object",
                PropertyKind::Annotation => "Annotation",
            };
            content.push_str(&format!(
                "<tr id=\"{id_fragment}\"><td>{label}</td><td>{kind}</td><td>{functional}</td><td><code>{domain}</code></td><td><code>{range}</code></td><td>{comment}</td></tr>\n",
                id_fragment = escape_html(&linker::fragment_from_iri(prop.id)),
                label = escape_html(prop.label),
                kind = kind,
                functional = prop.functional,
                domain = escape_html(prop.domain.unwrap_or("—")),
                range = escape_html(prop.range),
                comment = escape_html(prop.comment),
            ));
        }
        content.push_str("</tbody>\n</table>\n");
    }

    // Named individuals
    if !module.individuals.is_empty() {
        content.push_str("<h2>Named Individuals</h2>\n<table>\n<thead><tr><th>Name</th><th>Type</th><th>Properties</th><th>Comment</th></tr></thead>\n<tbody>\n");
        for ind in &module.individuals {
            let props_html = format_individual_properties(ind);
            content.push_str(&format!(
                "<tr id=\"{id_fragment}\"><td>{label}</td><td><code>{type_}</code></td><td>{props}</td><td>{comment}</td></tr>\n",
                id_fragment = escape_html(&linker::fragment_from_iri(ind.id)),
                label = escape_html(ind.label),
                type_ = escape_html(ind.type_),
                props = props_html,
                comment = escape_html(ind.comment),
            ));
        }
        content.push_str("</tbody>\n</table>\n");
    }

    render_docs_page(
        ns.label,
        &content,
        site_nav_html,
        docs_nav_html,
        &format!(
            r#"<a href="{base_path}/">Home</a> › <a href="{base_path}/docs/index.html">Docs</a> › {}"#,
            escape_html(ns.label)
        ),
        base_path,
    )
}

/// Formats an individual's property values as an HTML list.
fn format_individual_properties(ind: &Individual) -> String {
    if ind.properties.is_empty() {
        return String::from("—");
    }
    let mut items = String::from("<ul>");
    for (prop_iri, value) in ind.properties {
        let value_str = match value {
            IndividualValue::Str(s) => escape_html(s),
            IndividualValue::Int(i) => i.to_string(),
            IndividualValue::Bool(b) => b.to_string(),
            IndividualValue::Float(x) => x.to_string(),
            IndividualValue::IriRef(iri) => format!("<code>{}</code>", escape_html(iri)),
            IndividualValue::List(items) => {
                let joined = items
                    .iter()
                    .map(|i| format!("<code>{}</code>", escape_html(i)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", joined)
            }
        };
        let prop_local = linker::fragment_from_iri(prop_iri);
        items.push_str(&format!(
            "<li><code>{}</code>: {}</li>",
            escape_html(&prop_local),
            value_str
        ));
    }
    items.push_str("</ul>");
    items
}

/// Generates HTML pages from Markdown files in `src_dir`, writing to `out_dir`.
///
/// # Errors
///
/// Returns an error if files cannot be read or written.
fn generate_content_pages(
    src_dir: &Path,
    out_dir: &Path,
    index: &OntologyIndex,
    site_nav_html: &str,
    docs_nav_html: &str,
    base_path: &str,
) -> Result<()> {
    if !src_dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(src_dir)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", src_dir.display(), e))?
    {
        let entry = entry.map_err(|e| anyhow::anyhow!("Directory entry error: {}", e))?;
        let path = entry.path();
        if path.extension().map(|x| x == "md").unwrap_or(false) {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("page");
            let out_path = out_dir.join(format!("{}.html", stem));
            generate_single_page(
                &path,
                &out_path,
                stem,
                index,
                site_nav_html,
                docs_nav_html,
                base_path,
            )?;
        }
    }

    Ok(())
}

/// Generates a single HTML page from a Markdown file.
///
/// # Errors
///
/// Returns an error if the source file cannot be read or the output cannot be written.
fn generate_single_page(
    src: &Path,
    out: &Path,
    title: &str,
    index: &OntologyIndex,
    site_nav_html: &str,
    docs_nav_html: &str,
    base_path: &str,
) -> Result<()> {
    let markdown = std::fs::read_to_string(src)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", src.display(), e))?;

    let content_html = renderer::render_markdown(&markdown, index);
    let page = render_docs_page(
        title,
        &content_html,
        site_nav_html,
        docs_nav_html,
        &format!(
            r#"<a href="{base_path}/">Home</a> › <a href="{base_path}/docs/index.html">Docs</a> › {}"#,
            escape_html(title)
        ),
        base_path,
    );

    writer::write_html(out, &page)
}

/// A discovered concept page: its slug, human-readable title, and one-line
/// summary. Built from the `content/concepts/*.md` files so the concept index
/// and sidebar are generated from the source of truth and never drift (the
/// previous hand-maintained lists silently omitted authored concept pages).
struct ConceptLink {
    slug: String,
    title: String,
    summary: String,
}

/// Collects every concept page under `concepts_dir`, sorted alphabetically by
/// title for deterministic output. Title comes from YAML frontmatter `title:`
/// when present, else the first `# H1`, else a humanized slug; the summary is
/// the first prose sentence.
fn collect_concept_links(concepts_dir: &Path) -> Vec<ConceptLink> {
    let mut links: Vec<ConceptLink> = Vec::new();
    let read_dir = match std::fs::read_dir(concepts_dir) {
        Ok(rd) => rd,
        Err(_) => return links,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let slug = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let raw = std::fs::read_to_string(&path).unwrap_or_default();
        let (front_title, body) = strip_frontmatter(&raw);
        let title = front_title
            .or_else(|| first_h1(body))
            .unwrap_or_else(|| humanize_slug(&slug));
        let summary = concept_summary(body);
        links.push(ConceptLink {
            slug,
            title,
            summary,
        });
    }
    links.sort_by_key(|c| c.title.to_lowercase());
    links
}

/// Splits a leading `---`-delimited YAML frontmatter block from a markdown
/// document. Returns `(frontmatter title if any, body after frontmatter)`.
fn strip_frontmatter(raw: &str) -> (Option<String>, &str) {
    let trimmed = raw.trim_start_matches('\u{feff}');
    if let Some(rest) = trimmed.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---") {
            let front = &rest[..end];
            let body = rest[end + 4..].trim_start_matches('\n');
            let title = front.lines().find_map(|line| {
                line.trim()
                    .strip_prefix("title:")
                    .map(|v| v.trim().trim_matches(['"', '\'']).to_string())
                    .filter(|s| !s.is_empty())
            });
            return (title, body);
        }
    }
    (None, raw)
}

/// Returns the text of the first `# H1` heading, stripped of inline markdown
/// emphasis and code ticks.
fn first_h1(body: &str) -> Option<String> {
    body.lines()
        .find_map(|line| line.trim().strip_prefix("# "))
        .map(strip_inline_markdown)
        .filter(|s| !s.is_empty())
}

/// Returns the first prose sentence of `body` (skipping headings, fenced code
/// blocks, tables, lists, blockquotes, and HTML), stripped of inline markdown
/// and capped at 160 chars. A line that *begins* with inline code (a single
/// backtick) is prose and is kept; only ```-fenced blocks are skipped.
fn concept_summary(body: &str) -> String {
    let mut in_fence = false;
    let mut para = "";
    for line in body.lines().map(str::trim) {
        if line.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || line.is_empty() {
            continue;
        }
        let is_block = line.starts_with('#')
            || line.starts_with('|')
            || line.starts_with("- ")
            || line.starts_with("* ")
            || line.starts_with('>')
            || line.starts_with('<');
        if !is_block {
            para = line;
            break;
        }
    }
    let clean = strip_inline_markdown(para);
    let sentence = match clean.find(". ") {
        Some(i) => clean[..=i].trim().to_string(),
        None => clean,
    };
    if sentence.chars().count() > 160 {
        let mut s: String = sentence.chars().take(157).collect();
        s.push('…');
        s
    } else {
        sentence
    }
}

/// Strips markdown emphasis (`*`), code ticks, and link syntax to plain text,
/// for use in titles and summaries. `_` is preserved so snake_case identifiers
/// (e.g. `uor_foundation`) survive — `_`-delimited emphasis is rare in this
/// corpus and `*` is the conventional marker.
fn strip_inline_markdown(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '`' | '*' => {}
            '[' => {
                // `[text](url)` → keep `text`, drop `](url)`.
                for inner in chars.by_ref() {
                    if inner == ']' {
                        break;
                    }
                    out.push(inner);
                }
                if matches!(chars.peek(), Some('(')) {
                    for inner in chars.by_ref() {
                        if inner == ')' {
                            break;
                        }
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out.trim().to_string()
}

/// Humanizes a slug (`free-rank` → `Free Rank`) as a last-resort title.
fn humanize_slug(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Renders the concept index page body (`<h1>` + intro + `<ul>` of every
/// concept) from the discovered concept links.
fn render_concept_index_body(concepts: &[ConceptLink]) -> String {
    let mut body = String::from(
        "<h1>Concepts</h1>\n<p>Explanatory documentation for the key ideas in the \
         UOR Foundation ontology.</p>\n<ul class=\"concept-index\">\n",
    );
    for c in concepts {
        if c.summary.is_empty() {
            body.push_str(&format!(
                "<li><a href=\"{slug}.html\">{title}</a></li>\n",
                slug = escape_html(&c.slug),
                title = escape_html(&c.title),
            ));
        } else {
            body.push_str(&format!(
                "<li><a href=\"{slug}.html\">{title}</a> — {summary}</li>\n",
                slug = escape_html(&c.slug),
                title = escape_html(&c.title),
                summary = escape_html(&c.summary),
            ));
        }
    }
    body.push_str("</ul>");
    body
}

/// Generates the machine-generated README.md content.
fn generate_readme(ontology: &Ontology) -> String {
    format!(
        r#"# UOR Framework

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19068826.svg)](https://doi.org/10.5281/zenodo.19068826)

The Universal Object Reference (UOR) Framework is a Rust workspace implementing
the [UOR Foundation](https://uor.foundation/) ontology — a mathematical framework
for content-addressed, symmetric, multi-metric object spaces with algebraic
structure based on Z/(2^n)Z.

## Ontology

Version {version}: {ns} namespaces · {classes} classes · {props} properties · {inds} named individuals

All terms are encoded as typed Rust data in `spec/` (`uor-ontology`) and exported as:
- `foundation/` (`uor-foundation`) — typed Rust traits (published to crates.io)
- `public/uor.foundation.jsonld` — JSON-LD 1.1
- `public/uor.foundation.ttl` — Turtle 1.1
- `public/uor.foundation.nt` — N-Triples
- `public/uor.foundation.owl` — OWL 2 RDF/XML
- `public/uor.foundation.schema.json` — JSON Schema (Draft 2020-12)
- `public/uor.shapes.ttl` — SHACL validation shapes
- `public/uor.term.ebnf` — EBNF grammar

## Repository Structure

| Directory | Crate | Role |
|-----------|-------|------|
| `spec/` | `uor-ontology` | Ontology source of truth: typed static data + serializers |
| `codegen/` | `uor-codegen` | Code generation logic for the Rust trait crate |
| `foundation/` | `uor-foundation` | Generated Rust trait crate (published to crates.io) |
| `conformance/` | `uor-conformance` | Workspace-wide conformance validators |
| `docs/` | `uor-docs` | Documentation generator |
| `website/` | `uor-website` | Static site generator |
| `clients/` | `uor-clients` | Binaries: build, conformance, docs, website, crate |

## Building

```sh
# Build ontology artifacts
cargo run --bin uor-build

# Generate Rust trait crate
cargo run --bin uor-crate

# Generate documentation
cargo run --bin uor-docs

# Generate website
cargo run --bin uor-website

# Run full conformance suite
cargo run --bin uor-conformance
```

## CI

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Citation

If you use the UOR Framework in academic work, please cite it using the
metadata in [`CITATION.cff`](CITATION.cff).

## License

MIT — see [LICENSE](LICENSE).

---

*This README is machine-generated by `uor-docs`. Do not edit by hand.*
"#,
        version = ontology.version,
        ns = ontology.namespaces.len(),
        classes = ontology.class_count(),
        props = ontology.property_count(),
        inds = ontology.individual_count(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_has_all_terms() {
        let index = OntologyIndex::from_spec();
        assert_eq!(index.modules.len(), uor_ontology::counts::NAMESPACES);
        assert_eq!(index.classes.len(), uor_ontology::counts::CLASSES);
        assert_eq!(
            index.properties.len(),
            uor_ontology::counts::NAMESPACE_PROPERTIES
        );
        assert_eq!(index.individuals.len(), uor_ontology::counts::INDIVIDUALS);
    }

    #[test]
    fn class_ref_resolves() {
        let index = OntologyIndex::from_spec();
        assert!(index.is_class("https://uor.foundation/schema/Ring"));
        assert!(index.is_class("https://uor.foundation/op/Identity"));
        assert!(!index.is_class("https://uor.foundation/nonexistent/Foo"));
    }

    #[test]
    fn directive_expansion_known_class() {
        let index = OntologyIndex::from_spec();
        let src = "See {@class https://uor.foundation/schema/Ring} for details.";
        let expanded = renderer::expand_directives(src, &index);
        assert!(
            expanded.contains("[Ring]"),
            "Expected [Ring] in: {}",
            expanded
        );
    }

    #[test]
    fn directive_expansion_unknown_ref() {
        let index = OntologyIndex::from_spec();
        let src = "{@class https://example.com/Unknown}";
        let expanded = renderer::expand_directives(src, &index);
        // Unknown references render as code spans
        assert!(expanded.contains('`'));
    }

    #[test]
    fn directive_expansion_count() {
        let index = OntologyIndex::from_spec();
        let src = "The ontology has {@count:classes} classes.";
        let expanded = renderer::expand_directives(src, &index);
        assert!(
            expanded.contains(&uor_ontology::counts::CLASSES.to_string()),
            "Count directive should expand to {}, got: {}",
            uor_ontology::counts::CLASSES,
            expanded
        );
    }
}
