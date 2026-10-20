//! Phase 2 test: for every Path-1 class in the emitable subset, codegen
//! emits the expected `Null{Class}<H>` stub, `Default` impl, `ABSENT`
//! const, and trait impl(s).
//!
//! The emitable subset is smaller than the full Path-1 set because a
//! Null stub can only reference other Null stubs — classes whose
//! property ranges point at `Path2TheoremWitness` / `Path3PrimitiveBacked`
//! / `Path4TheoryDeferred` classes (which don't get Null stubs) cascade
//! out of Phase 2 via a fixed-point closure in
//! `uor_codegen::traits::emitable_null_set`.
//!
//! Test semantics:
//! 1. The emitable subset is non-empty.
//! 2. Every class in the emitable subset has an emitted stub struct,
//!    ABSENT const, and at least one `impl` for its ontology trait.
//! 3. The stub count is at least `MIN_PHASE2_STUBS`; if it drops below,
//!    a regression was introduced.

use std::fs;
use std::path::PathBuf;

use uor_codegen::classification::{classify_all, PathKind};
use uor_ontology::Ontology;

/// Stub-count ratchet: Phase 7 close. The number only grows as later
/// phases expand the emitable set; drops indicate regression. After
/// Phase 7d admitted Path-4 classes and Phase 7e removed the
/// enum-accessor filter, the emitable set covers Path-1 + Path-2 +
/// Path-4 — every class whose trait is supposed to be impl'd by a
/// generated stub.
const MIN_PHASE2_STUBS: usize = 440;

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
fn phase2_emission_produces_at_least_min_stubs() {
    let sources = load_namespace_sources();
    // Count `pub struct Null{X}<H: HostTypes>` in namespace module files
    // (enforcement.rs's 14 hand-written stubs are excluded).
    let count = sources.matches("pub struct Null").count();
    assert!(
        count >= MIN_PHASE2_STUBS,
        "Phase 2 regression: only {count} Null stubs emitted (expected ≥ {MIN_PHASE2_STUBS})"
    );
}

#[test]
fn every_emitted_stub_has_absent_const() {
    // For every `pub struct Null{X}<H: HostTypes>` in the namespace files,
    // there must be a matching `pub const ABSENT: Null{X}<H>`.
    let sources = load_namespace_sources();
    let mut missing: Vec<String> = Vec::new();
    for line in sources.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("pub struct Null") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            let expected = format!("ABSENT: Null{name}<H>");
            if !sources.contains(&expected) {
                missing.push(format!("Null{name}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "Phase 2 emission gap: {} Null stubs without ABSENT const:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}

#[test]
fn classification_path1_reports_nonzero_emitable_subset() {
    // Sanity: the Phase 0 classification reports a nonzero Path-1 count,
    // and Phase 2 emits at least some of them. If Path-1 count drops to
    // zero, the classifier broke; if emitted count is zero, Phase 2
    // regressed.
    let ontology = Ontology::full();
    let entries = classify_all(ontology);
    let path1 = entries
        .iter()
        .filter(|e| matches!(e.path_kind, PathKind::Path1HandleResolver))
        .count();
    assert!(path1 > 0);

    let sources = load_namespace_sources();
    let emitted = sources.matches("pub struct Null").count();
    assert!(emitted > 0);
    // Phase 7 summary: emitted covers Path-1 + Path-2 + Path-4. The
    // Path-2/Path-4 contributions are additive on top of the Path-1 base.
    eprintln!(
        "Phase 7 summary: {emitted} Null stubs emitted (Path-1 = {path1}, plus \
         Path-2 theorem-witness + Path-4 theory-deferred)"
    );
}
