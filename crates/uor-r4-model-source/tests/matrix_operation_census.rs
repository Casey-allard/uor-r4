//! #655 — matrix-operation census guard.
//!
//! The production chain must contain no project-owned conventional library-BLAS
//! matrix operation. Crate/manifest audits
//! (`uor-r4-graph-compiler::dependency_audit`, `uor-r4-proof-model::inference_audit`)
//! catch forbidden BLAS/GPU *crate dependencies*, but a `cblas_*` reached through
//! `#[link(name = "Accelerate")]` + `extern "C"` is an FFI *symbol*, not a crate
//! dependency, so those audits cannot see it. This source scan closes that gap:
//! it fails CI if a library-BLAS matrix token appears in any production-chain
//! crate `src/` (only the two audit files, which list such names as denylist
//! data, are exempt).
//!
//! Since #655-B2 the teacher's `cblas_sgemv`/`cblas_sgemm` are gone — its weight
//! matmuls are the pinned `uor-matmul` exact GEMM — so the guard now enforces
//! zero library-BLAS use anywhere in the production chain.
//!
//! The full classified inventory (including the remaining hand-rolled GPT-2
//! `conv1d` / attention accumulations this mechanical guard does not
//! keyword-match, tracked for a follow-up) lives in
//! `docs/matrix_operation_census.md`.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

/// Library-BLAS / external matrix-multiply markers. These name a conventional
/// matrix backend by symbol or crate; none has a non-matrix meaning, so any
/// occurrence in production `src/` is a real conventional-GEMM site. Bare
/// `sgemm`/`sgemv` are intentionally excluded — they appear as backticked words
/// in doc comments; the `cblas_` FFI prefix is the load-bearing token.
const BLAS_MATRIX_MARKERS: &[&str] =
    &["cblas_", "matrixmultiply", "openblas", "dgemm", "intel_mkl"];

/// Files allowed to contain a library-BLAS matrix marker today, by path suffix.
/// Since #655-B2 the teacher's Accelerate `cblas_*` FFI is gone — every teacher
/// weight matmul is the pinned `uor-matmul` exact GEMM — so no production *use*
/// site remains sanctioned. Only the two dependency/manifest audits are listed:
/// they enumerate forbidden BLAS crate names as denylist *data* so those crates
/// can never enter the tree — the opposite of performing a matrix operation.
const SANCTIONED_SUFFIXES: &[&str] = &[
    "uor-r4-graph-compiler/src/dependency_audit.rs",
    "uor-r4-proof-model/src/inference_audit.rs",
    // #804 Apple Accelerate path (maintainer-approved 2026-08-18): the ONE
    // sanctioned library-BLAS use site, compiled only under the opt-in
    // `observation-blas-exception` feature on macOS, for local source-backed
    // inference and teacher-forced observation. Its gating and its
    // single dispatch site are pinned by
    // `observation_blas_exception_is_opt_in_and_never_default` below —
    // adding this suffix does NOT exempt default builds from anything,
    // because the pin test fails if the cfg gates ever loosen.
    "uor-r4-model-source/src/observation_blas_exception.rs",
];

/// Workspace root: `CARGO_MANIFEST_DIR` = `<root>/crates/uor-r4-model-source`.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize workspace root")
}

/// Every production-chain crate `src/` plus the root serving crate's `src/`.
/// Only `src/` trees are scanned, so this test file (under `tests/`) and its
/// own marker literals are never self-flagged.
fn production_src_roots(root: &Path) -> Vec<PathBuf> {
    let mut roots = vec![root.join("src")];
    if let Ok(entries) = std::fs::read_dir(root.join("crates")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                roots.push(path.join("src"));
            }
        }
    }
    roots
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if name == "target" || name == ".git" {
                continue;
            }
            rust_sources(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn is_sanctioned(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    SANCTIONED_SUFFIXES
        .iter()
        .any(|suffix| normalized.ends_with(suffix))
}

#[test]
fn no_conventional_blas_matrix_op_outside_the_sanctioned_teacher_site() {
    let root = workspace_root();
    let mut files = Vec::new();
    for r in production_src_roots(&root) {
        rust_sources(&r, &mut files);
    }
    assert!(
        files.len() > 20,
        "census scan found only {} files; path resolution is wrong",
        files.len()
    );

    let mut leaks: Vec<String> = Vec::new();
    for file in &files {
        if is_sanctioned(file) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        for (lineno, line) in text.lines().enumerate() {
            for marker in BLAS_MATRIX_MARKERS {
                if line.contains(marker) {
                    leaks.push(format!(
                        "{}:{}: conventional library-BLAS matrix marker `{marker}` — migrate to \
                         uor-matmul (#655-B) or record + classify it in docs/matrix_operation_census.md",
                        file.display(),
                        lineno + 1
                    ));
                }
            }
        }
    }

    assert!(
        leaks.is_empty(),
        "conventional library-BLAS matrix operations leaked into the production chain \
         ({} occurrences):\n{}",
        leaks.len(),
        leaks.join("\n")
    );
    eprintln!(
        "matrix-operation census: {} production source files scanned, 0 unsanctioned library-BLAS matrix ops",
        files.len()
    );
}

/// Since #655-B2, the teacher executor must own NO library-BLAS matrix FFI: its
/// weight matmuls are the pinned `uor-matmul` exact GEMM. If a `cblas_*` symbol
/// reappears here it is a regression to a conventional matrix backend, which the
/// main scan above would also catch (the file is no longer sanctioned) — this
/// asserts it directly at the teacher site for a clearer failure.
#[test]
fn teacher_site_owns_no_blas_matrix_ffi() {
    let root = workspace_root();
    let lib = root.join("crates/uor-r4-model-source/src/lib.rs");
    let text = std::fs::read_to_string(&lib).expect("read the teacher executor source");
    assert!(
        !text.contains("cblas_"),
        "the teacher site {} reintroduced a `cblas_` matrix FFI — teacher matmuls must stay \
         uor-matmul-owned (#655-B2)",
        lib.display()
    );
}

/// #804: the Apple Accelerate path stays opt-in, macOS-only, limited to the
/// local source-backed model, and can never silently widen:
///
/// - the exception module and every dispatch to it in `lib.rs` sit behind
///   `cfg(all(feature = "observation-blas-exception", target_os = "macos"))`
///   (pinned by exact gate-string counting against the dispatch count);
/// - no Cargo manifest in the workspace enables the feature by default
///   (`default = [...]` never names it, and no dependency edge turns it
///   on unconditionally);
/// - the `cblas_` FFI itself lives ONLY in the sanctioned exception file.
#[test]
fn observation_blas_exception_is_opt_in_and_never_default() {
    let root = workspace_root();
    const GATE: &str =
        r#"#[cfg(all(feature = "observation-blas-exception", target_os = "macos"))]"#;
    const ANTI_GATE: &str =
        r#"#[cfg(not(all(feature = "observation-blas-exception", target_os = "macos")))]"#;

    let lib = root.join("crates/uor-r4-model-source/src/lib.rs");
    let lib_text = std::fs::read_to_string(&lib).expect("read the teacher executor source");
    let dispatches = lib_text.matches("observation_blas_exception::").count();
    assert_eq!(
        dispatches, 2,
        "the exception has exactly two dispatch sites (matmul, matmul_batched); \
         a new one needs its own maintainer sign-off and this pin updated"
    );
    let gates = lib_text.matches(GATE).count();
    let anti_gates = lib_text.matches(ANTI_GATE).count();
    assert_eq!(
        gates, 4,
        "lib.rs carries exactly four exception gates (module decl, two \
         dispatches, backend label); found {gates}"
    );
    assert_eq!(
        anti_gates, 3,
        "every gated dispatch keeps its default-path twin (two matmuls + \
         backend label); found {anti_gates}"
    );

    let exception = root.join("crates/uor-r4-model-source/src/observation_blas_exception.rs");
    let exception_text =
        std::fs::read_to_string(&exception).expect("read the exception module source");
    assert!(
        exception_text.contains("#[link(name = \"Accelerate\", kind = \"framework\")]"),
        "the exception file is where the Accelerate FFI lives"
    );

    // No manifest may default-enable the feature: a `default = [...]` list
    // naming it, or a plain dependency edge activating it outside the
    // named passthrough features, would make the exception silent.
    let mut manifests = vec![root.join("Cargo.toml")];
    if let Ok(entries) = std::fs::read_dir(root.join("crates")) {
        for entry in entries.flatten() {
            let manifest = entry.path().join("Cargo.toml");
            if manifest.is_file() {
                manifests.push(manifest);
            }
        }
    }
    for manifest in manifests {
        let Ok(text) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("default") && trimmed.contains("observation-blas-exception") {
                panic!(
                    "{} default-enables the #804 exception feature — it must stay opt-in",
                    manifest.display()
                );
            }
            // A dependency edge like `features = ["observation-blas-exception"]`
            // outside a `[features]` passthrough would hard-enable it.
            if trimmed.contains("path = ") && trimmed.contains("observation-blas-exception") {
                panic!(
                    "{} hard-enables the #804 exception feature on a dependency edge",
                    manifest.display()
                );
            }
        }
    }
}
