//! Phase 8 test: every Path-1 class has the four-type Resolved wrapper
//! emitted: `{Foo}Handle<H>`, `{Foo}Resolver<H>`, `{Foo}Record<H>`, and
//! `Resolved{Foo}<'r, R, H>`.

use std::fs;
use std::path::PathBuf;

use uor_codegen::classification::{classify_all, PathKind};
use uor_ontology::Ontology;

fn find_workspace_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("cwd");
    loop {
        if dir.join("foundation/src/enforcement.rs").exists() {
            return dir;
        }
        dir = match dir.parent() {
            Some(p) => p.to_path_buf(),
            None => panic!("no workspace root"),
        };
    }
}

fn load_namespace_sources() -> String {
    let root = find_workspace_root();
    let mut out = String::new();
    for subdir in ["bridge", "kernel", "user"] {
        let dir = root.join("foundation/src").join(subdir);
        if let Ok(entries) = fs::read_dir(&dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "rs") {
                    if let Ok(content) = fs::read_to_string(&p) {
                        out.push_str(&content);
                        out.push('\n');
                    }
                }
            }
        }
    }
    out
}

#[test]
fn every_path1_class_has_handle_resolver_record_resolved() {
    let ontology = Ontology::full();
    let entries = classify_all(ontology);
    let source = load_namespace_sources();

    let path1: Vec<&str> = entries
        .iter()
        .filter(|e| matches!(e.path_kind, PathKind::Path1HandleResolver))
        .map(|e| e.class_local)
        .collect();

    assert!(
        path1.len() >= 400,
        "expected ≥ 400 Path-1 classes, got {}",
        path1.len()
    );

    let mut missing_handle = Vec::new();
    let mut missing_resolver = Vec::new();
    let mut missing_record = Vec::new();
    let mut missing_resolved = Vec::new();
    for name in &path1 {
        if !source.contains(&format!("pub struct {name}Handle<H: HostTypes>")) {
            missing_handle.push(*name);
        }
        if !source.contains(&format!("pub trait {name}Resolver<H: HostTypes>")) {
            missing_resolver.push(*name);
        }
        if !source.contains(&format!("pub struct {name}Record<H: HostTypes>")) {
            missing_record.push(*name);
        }
        if !source.contains(&format!("pub struct Resolved{name}<")) {
            missing_resolved.push(*name);
        }
    }

    let report = |label: &str, list: &[&str]| {
        if !list.is_empty() {
            let preview: Vec<&&str> = list.iter().take(10).collect();
            panic!("{} {label} missing — first 10: {preview:?}", list.len());
        }
    };
    report("`{Name}Handle<H>`", &missing_handle);
    report("`{Name}Resolver<H>`", &missing_resolver);
    report("`{Name}Record<H>`", &missing_record);
    report("`Resolved{Name}<...>`", &missing_resolved);
}

#[test]
fn resolved_wrapper_has_new_handle_resolver_record() {
    // Spot-check: `Resolved{Foo}::new(handle, resolver)` is generated for at
    // least one well-known Path-1 class — `IOBoundary`.
    let source = load_namespace_sources();
    assert!(
        source.contains("ResolvedIOBoundary"),
        "ResolvedIOBoundary missing from generated source"
    );
    assert!(
        source.contains("pub fn new(handle: IOBoundaryHandle<H>, resolver: &'r R)"),
        "ResolvedIOBoundary::new constructor missing"
    );
}
