//! Phase 7b test: `type Foo = Null{Range}<H>;` in a Null-stub impl block for
//! a child trait is skipped if a parent trait already introduced `Foo`.
//!
//! Strategy: pick a Null stub emitted for a class with multi-level
//! inheritance and assert each associated-type name appears at most once
//! across all `impl ... for Null{X}<H>` blocks for that class.

use std::collections::{HashMap, HashSet};

use uor_codegen::mapping::namespace_mappings;
use uor_codegen::traits::generate_namespace_module;
use uor_ontology::model::{Property, PropertyKind};
use uor_ontology::Ontology;

fn props_by_domain<'a>(ontology: &'a Ontology) -> HashMap<&'a str, Vec<&'a Property>> {
    let mut map: HashMap<&'a str, Vec<&'a Property>> = HashMap::new();
    for module in &ontology.namespaces {
        for prop in &module.properties {
            if let Some(domain) = prop.domain {
                if prop.kind != PropertyKind::Annotation {
                    map.entry(domain).or_default().push(prop);
                }
            }
        }
    }
    map
}

/// Returns every generated namespace module's Rust source, keyed by
/// namespace prefix.
fn render_all_modules(ontology: &Ontology) -> HashMap<String, String> {
    let ns_map = namespace_mappings();
    let all = props_by_domain(ontology);
    let mut out = HashMap::new();
    for module in &ontology.namespaces {
        let src = generate_namespace_module(module, &ns_map, &all);
        out.insert(module.namespace.prefix.to_string(), src);
    }
    out
}

/// For every generated module, assert each `Null{X}<H>`'s impl blocks —
/// class trait + transitive supertrait impls — together emit each `type` key
/// at most once.
#[test]
fn null_stub_assoc_types_unique_across_impls() {
    let ontology = Ontology::full();
    let rendered = render_all_modules(ontology);

    for (ns, source) in &rendered {
        // For each `pub struct Null{X}<H: HostTypes>` block, collect the
        // following impl blocks (until the next `pub struct` / end-of-file)
        // and parse their `type Foo = ...;` lines.
        let mut i = 0usize;
        let lines: Vec<&str> = source.lines().collect();
        while i < lines.len() {
            let line = lines[i];
            if let Some(rest) = line.strip_prefix("pub struct Null") {
                let struct_name = rest.split(['<', ' ']).next().unwrap_or_default();
                let null_type = format!("Null{struct_name}");
                // Scan forward, collecting assoc declarations until the next
                // `pub struct Null` or `pub trait` line.
                let mut j = i + 1;
                let mut assocs: Vec<String> = Vec::new();
                while j < lines.len() {
                    let l = lines[j];
                    if l.starts_with("pub struct Null") || l.starts_with("pub trait ") {
                        break;
                    }
                    if let Some(rest) = l.strip_prefix("    type ") {
                        // `type Foo = NullBar<H>;` — capture the first word.
                        if let Some(eq_pos) = rest.find(" =") {
                            let assoc_name = rest[..eq_pos].trim().to_string();
                            assocs.push(assoc_name);
                        }
                    }
                    j += 1;
                }
                let mut seen: HashSet<String> = HashSet::new();
                let mut duplicates: Vec<String> = Vec::new();
                for a in &assocs {
                    if !seen.insert(a.clone()) {
                        duplicates.push(a.clone());
                    }
                }
                assert!(
                    duplicates.is_empty(),
                    "namespace `{ns}` / `{null_type}`: assoc types \
                     emitted multiple times across impl blocks — {duplicates:?}. \
                     Phase 7b dedup via `inherited_assocs` must skip the \
                     child-trait impl's redundant `type = ...` lines."
                );
                i = j;
                continue;
            }
            i += 1;
        }
    }
}
