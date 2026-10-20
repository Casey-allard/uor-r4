//! v0.2.2 W5: ψ-leakage validator.
//!
//! Scans the consumer-facing documentation surface for unauthorized
//! references to the ψ vocabulary. ψ_1..ψ_9 are mathematically correct
//! identifiers in the proof and op namespaces; the validator excludes
//! those scopes. Any other surface (rendered rustdoc, README, in-tree
//! markdown) carrying ψ glyphs or the `psi_*` / `psi-*` substrings fails
//! the v0.2.2 "ψ vocabulary reservation" gate.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

/// Forbidden patterns. Case-insensitive substring search.
const FORBIDDEN_GLYPHS: &[&str] = &["\u{03C8}", "\u{03A8}"];
const FORBIDDEN_SUBSTRINGS_CI: &[&str] =
    &["psi-map", "psi map", "ψ-map", "ψ-pipeline", "psi-pipeline"];
/// Word-boundary patterns that need adjacency checks.
const FORBIDDEN_TOKEN_PREFIXES: &[&str] = &["psi_", "psi-"];

/// File extensions to scan.
const SCAN_EXTENSIONS: &[&str] = &["md", "rs"];

/// Path substrings that exclude a file from the scan (mathematically
/// correct internal use of ψ in the proof and op namespaces is allowed).
const EXCLUDED_PATH_SUBSTRINGS: &[&str] = &[
    "/spec/src/namespaces/proof.rs",
    "/spec/src/namespaces/op.rs",
    "/spec/src/namespaces/homology.rs",
    "/spec/src/namespaces/cohomology.rs",
    "/spec/src/namespaces/derivation.rs",
    "/lean4/",
    "/foundation/src/bridge/proof.rs",
    "/foundation/src/bridge/cohomology.rs",
    "/foundation/src/bridge/homology.rs",
    "/foundation/src/bridge/derivation.rs",
    "/foundation/src/kernel/op.rs",
    "/conformance/src/validators/docs/psi_leakage.rs",
    "/conformance/standards/owl.md",
    "/external/",
    "/target/",
    // The plan file documents the gate itself.
    "/external/v0.2.2-plan.md",
];

/// Targets the v0.2.2 ψ-leakage gate scans. The plan scopes the v0.2.2 gate
/// to the consumer-facing *crate* surface — the workspace `README.md`,
/// `foundation/README.md`, and any `foundation/docs/*.md` — which is the
/// surface the published `uor-foundation` crate exposes on docs.rs and via
/// `cargo doc`. The website concept pages under `docs/content/concepts/`
/// are addressed by the W18 documentation refresh (a separate editorial
/// sweep), and the proof/op/homology/cohomology namespace files are
/// excluded because ψ_1..ψ_9 are mathematically correct identifiers there.
const SCAN_TARGETS: &[&str] = &["README.md", "foundation/README.md", "foundation/docs"];

/// Validates the v0.2.2 W5 ψ-leakage gate.
///
/// # Errors
///
/// Returns an error if a target cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();
    let validator = "docs/psi_leakage";

    let mut violations: Vec<String> = Vec::new();
    for target in SCAN_TARGETS {
        let path = workspace.join(target);
        if path.is_dir() {
            walk(&path, &mut violations)?;
        } else if path.is_file() {
            scan_file(&path, &mut violations);
        }
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            validator,
            "No ψ vocabulary leaks in consumer-facing crate surface (README, foundation/README, foundation/docs)",
        ));
    } else {
        let preview: Vec<String> = violations.iter().take(10).cloned().collect();
        let summary = format!(
            "ψ vocabulary leak in {} location(s):\n       {}",
            violations.len(),
            preview.join("\n       ")
        );
        report.push(TestResult::fail(validator, summary));
    }

    Ok(report)
}

fn scan_file(path: &Path, violations: &mut Vec<String>) {
    let path_str = path.to_string_lossy().into_owned();
    if EXCLUDED_PATH_SUBSTRINGS
        .iter()
        .any(|excl| path_str.contains(excl))
    {
        return;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    if !SCAN_EXTENSIONS.contains(&ext) {
        return;
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    for (line_no, line) in content.lines().enumerate() {
        if line_violates(line) {
            violations.push(format!("{path_str}:{}", line_no + 1));
        }
    }
}

fn walk(dir: &Path, violations: &mut Vec<String>) -> Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, violations)?;
        } else if path.is_file() {
            scan_file(&path, violations);
        }
    }
    Ok(())
}

fn line_violates(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    for glyph in FORBIDDEN_GLYPHS {
        if line.contains(glyph) {
            return true;
        }
    }
    for needle in FORBIDDEN_SUBSTRINGS_CI {
        if lower.contains(needle) {
            return true;
        }
    }
    for prefix in FORBIDDEN_TOKEN_PREFIXES {
        if let Some(idx) = lower.find(prefix) {
            // Check word boundary on the preceding char (must not be alphanumeric).
            let before_ok = idx == 0
                || !lower
                    .as_bytes()
                    .get(idx - 1)
                    .copied()
                    .map(|b| (b as char).is_ascii_alphanumeric())
                    .unwrap_or(false);
            if before_ok {
                return true;
            }
        }
    }
    false
}
