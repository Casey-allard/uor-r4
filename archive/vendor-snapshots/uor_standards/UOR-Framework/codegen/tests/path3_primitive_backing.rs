//! Phase 11a R13 verification: every `PATH3_ALLOW_LIST` entry names a
//! primitive function that exists in `foundation/src/enforcement.rs` or
//! `foundation/src/pipeline.rs`. Loud failure: a missing primitive
//! breaks the test, blocking allow-list growth without backing code.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use uor_codegen::classification::{classify_all, PathKind, PATH3_ALLOW_LIST};
use uor_ontology::Ontology;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn read_concat(paths: &[&str]) -> String {
    let root = workspace_root();
    let mut out = String::new();
    for p in paths {
        let path = root.join(p);
        let body = std::fs::read_to_string(&path).expect("read foundation source");
        out.push_str(&body);
        out.push('\n');
    }
    out
}

#[test]
fn every_path3_allow_list_primitive_exists() {
    let foundation = read_concat(&[
        "foundation/src/enforcement.rs",
        "foundation/src/pipeline.rs",
    ]);
    for (class_iri, primitive) in PATH3_ALLOW_LIST {
        // Match either `pub fn {primitive}` or `pub(crate) fn {primitive}`.
        let pat_pub = format!("pub fn {primitive}");
        let pat_crate = format!("pub(crate) fn {primitive}");
        let found = foundation.contains(&pat_pub) || foundation.contains(&pat_crate);
        assert!(
            found,
            "PATH3_ALLOW_LIST entry `{class_iri}` references missing primitive `{primitive}` \
             (expected `pub fn {primitive}` or `pub(crate) fn {primitive}` in foundation)"
        );
    }
}

#[test]
fn every_path3_classification_has_primitive_backing() {
    let ontology = Ontology::full();
    for entry in classify_all(ontology) {
        if let PathKind::Path3PrimitiveBacked { primitive_name } = &entry.path_kind {
            // Verify the primitive_name matches one in the allow-list.
            let in_list = PATH3_ALLOW_LIST
                .iter()
                .any(|(iri, prim)| iri == &entry.class_iri && prim == &primitive_name.as_str());
            assert!(
                in_list,
                "Path-3 classification for `{}` names primitive `{primitive_name}` not in PATH3_ALLOW_LIST",
                entry.class_iri
            );
        }
    }
}
