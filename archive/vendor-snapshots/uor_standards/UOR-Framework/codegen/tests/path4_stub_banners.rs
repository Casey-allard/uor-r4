//! Phase 7d test: every Path-4 theory-deferred class has a Null stub in
//! the generated source carrying the exact `#[doc(hidden)]` +
//! THEORY-DEFERRED banner combination. Missing or drifted banner fails.

use std::fs;
use std::path::PathBuf;

use uor_codegen::classification::{classify_all, PathKind};
use uor_ontology::Ontology;

const BANNER_MARKER: &str =
    "THEORY-DEFERRED \\u{2014} not a valid implementation; see [docs/theory_deferred.md].";

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

fn load_foundation_source() -> String {
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
fn every_path4_class_has_null_stub_with_banner() {
    let ontology = Ontology::full();
    let entries = classify_all(ontology);
    let source = load_foundation_source();

    let path4: Vec<&str> = entries
        .iter()
        .filter(|e| matches!(e.path_kind, PathKind::Path4TheoryDeferred))
        .map(|e| e.class_local)
        .collect();

    assert!(
        !path4.is_empty(),
        "Path-4 class list is empty — classifier regression"
    );

    let mut missing_stub: Vec<&str> = Vec::new();
    let mut missing_banner: Vec<&str> = Vec::new();
    for name in &path4 {
        let stub_decl = format!("pub struct Null{name}<H: HostTypes>");
        let stub_pos = match source.find(&stub_decl) {
            Some(p) => p,
            None => {
                missing_stub.push(name);
                continue;
            }
        };
        // The 400-character window preceding the declaration must contain
        // both `#[doc(hidden)]` and the THEORY-DEFERRED marker string.
        let lookback = stub_pos.saturating_sub(400);
        let window = &source[lookback..stub_pos];
        if !window.contains("#[doc(hidden)]") || !window.contains(BANNER_MARKER) {
            missing_banner.push(name);
        }
    }

    assert!(
        missing_stub.is_empty(),
        "Path-4 classes with no generated Null stub: {missing_stub:?}"
    );
    assert!(
        missing_banner.is_empty(),
        "Path-4 Null stubs missing the `#[doc(hidden)]` / THEORY-DEFERRED \
         banner: {missing_banner:?}"
    );
}
