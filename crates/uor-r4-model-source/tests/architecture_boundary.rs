//! #607 boundary leak test — the point of the issue.
//!
//! The architecture-neutral claim is that GPT-2-specific knowledge lives
//! ONLY in `uor-r4-model-source` (the adapter crate); every other crate
//! consumes the neutral `TeacherOracle` two-surface trait and never a
//! GPT-2 tensor name, fused-QKV detail, or family label. This test scans
//! the workspace source and fails if any GPT-2 family token appears outside
//! this crate, so a future change that leaks family logic downstream is
//! caught mechanically rather than by review.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

/// Unambiguous GPT-2 family identifiers: fused-QKV tensor, learned-position
/// and embedding tensors, the HF architecture label, and the prefixed block
/// naming. None of these has a non-GPT-2 meaning, so any occurrence outside
/// the adapter crate is a real boundary leak.
const GPT2_TOKENS: &[&str] = &[
    "c_attn",
    "wte.weight",
    "wpe.weight",
    "GPT2LMHead",
    "transformer.h.",
    "gelu_new",
    "huggingface-gpt2",
];

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = <root>/crates/uor-r4-model-source
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize workspace root")
}

/// The adapter crate that is ALLOWED to contain GPT-2 tokens.
fn adapter_crate() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .expect("canonicalize adapter crate")
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
            // Skip build output and VCS; never descend into the adapter crate.
            if name == "target" || name == ".git" {
                continue;
            }
            rust_sources(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn gpt2_family_tokens_stay_inside_the_adapter_crate() {
    let root = workspace_root();
    let adapter = adapter_crate();

    // Scan every crate's src/ plus the root binary's src/, excluding the
    // adapter crate itself.
    let mut roots = vec![root.join("src")];
    if let Ok(entries) = std::fs::read_dir(root.join("crates")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.canonicalize().ok().as_deref() != Some(adapter.as_path()) {
                roots.push(path.join("src"));
            }
        }
    }

    let mut files = Vec::new();
    for r in &roots {
        rust_sources(r, &mut files);
    }
    assert!(
        files.len() > 20,
        "leak scan found only {} files under {roots:?}; path resolution is wrong",
        files.len()
    );

    let mut leaks: Vec<String> = Vec::new();
    for file in &files {
        // Defensive: never flag the adapter crate even if path logic drifts.
        if file.starts_with(&adapter) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        for (lineno, line) in text.lines().enumerate() {
            for token in GPT2_TOKENS {
                if line.contains(token) {
                    leaks.push(format!(
                        "{}:{}: leaks `{token}`",
                        file.display(),
                        lineno + 1
                    ));
                }
            }
        }
    }

    assert!(
        leaks.is_empty(),
        "GPT-2 family tokens leaked outside uor-r4-model-source ({} occurrences):\n{}",
        leaks.len(),
        leaks.join("\n")
    );
    eprintln!(
        "boundary leak scan: {} source files outside the adapter crate, 0 GPT-2 tokens",
        files.len()
    );
}
