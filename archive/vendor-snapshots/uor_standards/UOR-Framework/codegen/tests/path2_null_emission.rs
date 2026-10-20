//! Phase 3 test: at least some `Path2TheoremWitness` classes have
//! emitted Null stubs. The exact subset depends on the fixed-point
//! reference closure in `emitable_null_set`; this test ratchets the
//! current lower bound.

use std::fs;
use std::path::PathBuf;

use uor_codegen::classification::{classify_all, PathKind};
use uor_ontology::Ontology;

/// Phase 3 + Phase 7 closure ratchet. Phase 7 unblocked the remaining
/// cascade drops by admitting Path-4 references; every Path-2 class now
/// has a Null stub.
const MIN_PHASE3_PATH2_STUBS: usize = 10;

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
fn phase3_path2_coverage() {
    let ontology = Ontology::full();
    let entries = classify_all(ontology);
    let sources = load_namespace_sources();

    let path2_classes: Vec<&str> = entries
        .iter()
        .filter(|e| matches!(e.path_kind, PathKind::Path2TheoremWitness { .. }))
        .map(|e| e.class_local)
        .collect();

    let mut emitted: Vec<&str> = Vec::new();
    let mut missing: Vec<&str> = Vec::new();
    for name in &path2_classes {
        let needle = format!("pub struct Null{name}<H: HostTypes>");
        if sources.contains(&needle) {
            emitted.push(name);
        } else {
            missing.push(name);
        }
    }

    eprintln!("Phase 3 Path-2 closure:");
    eprintln!("  emitted ({}): {:?}", emitted.len(), emitted);
    eprintln!(
        "  cascaded out of emission set ({}): {:?}",
        missing.len(),
        missing
    );

    assert!(
        emitted.len() >= MIN_PHASE3_PATH2_STUBS,
        "Phase 3 regression: only {} Path-2 Null stubs emitted (expected ≥ {MIN_PHASE3_PATH2_STUBS}):\n  \
         emitted: {:?}\n  missing: {:?}",
        emitted.len(),
        emitted,
        missing
    );
}
