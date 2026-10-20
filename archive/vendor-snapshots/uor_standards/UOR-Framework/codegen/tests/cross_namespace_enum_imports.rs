//! Phase 7c test: enum imports for parent-trait properties are added to the
//! current namespace module so Null-stub impls compile.
//!
//! Strategy: render every generated namespace module, scan for
//! `{AnyEnum}::` references in the body, and require a matching
//! `use crate::enums::{AnyEnum};` line at the top.

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

#[test]
fn every_namespace_module_imports_every_referenced_enum() {
    let ontology = Ontology::full();
    let ns_map = namespace_mappings();
    let all = props_by_domain(ontology);
    let enum_names: HashSet<&'static str> = Ontology::enum_class_names().iter().copied().collect();

    for module in &ontology.namespaces {
        let src = generate_namespace_module(module, &ns_map, &all);
        let used_enums: HashSet<&str> = enum_names
            .iter()
            .copied()
            .filter(|name| {
                // Skip self-references inside the enum's own definition file.
                let needle_fn = format!(") -> {name}");
                let needle_param = format!("-> {name};");
                let needle_return = format!(" -> {name} ");
                let needle_path = format!("crate::enums::{name}");
                src.contains(needle_fn.as_str())
                    || src.contains(needle_param.as_str())
                    || src.contains(needle_return.as_str())
                    || src.contains(needle_path.as_str())
            })
            .collect();

        for name in used_enums {
            // Either `use crate::enums::{name};` is present at file top, or
            // the body references it via the `crate::enums::{name}` path.
            let use_line = format!("use crate::enums::{name};");
            let qualified = format!("crate::enums::{name}");
            assert!(
                src.contains(&use_line) || src.contains(&qualified),
                "namespace `{}` references enum `{name}` without a \
                 `use crate::enums::{name};` line at the top of the module",
                module.namespace.prefix
            );
        }
    }
}
