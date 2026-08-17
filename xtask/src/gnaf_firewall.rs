//! GNAF dependency firewall (SPEC 10.1), ported from the vendored
//! `proofs/wasm-gemm-gnaf/Tools/firewall.py` (#653 phase-2 follow-up:
//! `docs/gnaf_integration_653.md` classifies `Tools/*.py` as `defer-open`,
//! recommending r4-native equivalents rather than a vendored Python
//! dependency; this is the first such port).
//!
//! SPEC 10.1: "Foundation, Wasm, Gemm, Cost, and the extensional definitions
//! in Universal/Competitor.lean, Correct.lean and Feasible.lean SHALL NOT
//! import GNAF, Atlas, Artifact, Universal/LowerBound, Universal/Argmin, or
//! Theorems. A source-and-environment gate SHALL reject an artifact-,
//! selector-, or conclusion-dependent scope predicate."
//!
//! The point is that the competitor universe must be defined without
//! reference to the artifact that will be compared against it. If
//! `ProfileValid` could see the selected artifact, `GlobalOptimal` would be a
//! statement about a universe built around its own answer. This check makes
//! that structurally impossible rather than a convention someone remembers.
//!
//! This is a faithful, static-analysis-only port: same `PROTECTED`/
//! `FORBIDDEN` module lists, same "protected module imports a forbidden
//! module" violation shape, same exit-nonzero-on-violation contract as
//! `Tools/firewall.py`. It does not run `lake`/`lean` and has no Lean
//! toolchain dependency, so it can run in this repository's own `cargo
//! xtask` gate ladder without vendoring Python or requiring Lean to be
//! installed.

use std::path::{Path, PathBuf};

use crate::Fail;

/// Modules whose competitor-universe definitions must stay
/// artifact-/selector-/conclusion-blind. Paths are relative to
/// `proofs/wasm-gemm-gnaf/WasmGemmGnaf/`, matching `Tools/firewall.py`'s
/// `PROTECTED` list exactly: a trailing `/` protects a whole directory, a
/// bare `.lean` path protects one file.
const PROTECTED: [&str; 7] = [
    "Foundation/",
    "Wasm/",
    "Gemm/",
    "Cost/",
    "Universal/Competitor.lean",
    "Universal/Correct.lean",
    "Universal/Feasible.lean",
];

/// Modules a protected module may never import, directly or as a
/// sub-namespace (`WasmGemmGnaf.GNAF.Foo` is as forbidden as
/// `WasmGemmGnaf.GNAF` itself). Matches `Tools/firewall.py`'s `FORBIDDEN`
/// list exactly.
const FORBIDDEN: [&str; 6] = [
    "WasmGemmGnaf.GNAF",
    "WasmGemmGnaf.Atlas",
    "WasmGemmGnaf.Artifact",
    "WasmGemmGnaf.Universal.LowerBound",
    "WasmGemmGnaf.Universal.Argmin",
    "WasmGemmGnaf.Theorems",
];

/// SPEC 10.1: no protected module imports a forbidden one.
pub fn gnaf_firewall(root: &Path) -> Result<(), Fail> {
    let base = root.join("proofs/wasm-gemm-gnaf/WasmGemmGnaf");
    if !base.exists() {
        return Err(format!(
            "{} not found; the vendored GNAF proof tree (#742) is expected \
             at this path",
            base.display()
        )
        .into());
    }

    let mut files = Vec::new();
    collect_lean(&base, &mut files)?;
    files.sort();

    let mut checked = 0usize;
    let mut violations = Vec::new();
    for path in &files {
        let rel = path
            .strip_prefix(&base)
            .unwrap_or(path)
            .display()
            .to_string();
        if !PROTECTED.iter().any(|p| rel == *p || rel.starts_with(p)) {
            continue;
        }
        checked += 1;
        let text = std::fs::read_to_string(path)?;
        for (i, line) in text.lines().enumerate() {
            let Some(module) = parse_import(line) else {
                continue;
            };
            for bad in FORBIDDEN {
                if module == bad || module.starts_with(&format!("{bad}.")) {
                    violations.push(format!("{}:{}: imports {module}", path.display(), i + 1));
                }
            }
        }
    }

    if !violations.is_empty() {
        return Err(format!(
            "SPEC 10.1: the competitor universe (Foundation, Wasm, Gemm, Cost, \
             and the extensional Universal/Competitor|Correct|Feasible \
             definitions) must not depend on the artifact, the selector, or \
             any conclusion. Move the definition or invert the dependency.\n\n{}",
            violations.join("\n")
        )
        .into());
    }
    println!(
        "gnaf-firewall: dependency firewall clean: {checked} protected \
         modules, no forbidden import (SPEC 10.1)"
    );
    Ok(())
}

fn collect_lean(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Fail> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_lean(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "lean") {
            out.push(path);
        }
    }
    Ok(())
}

/// Parse a Lean `import Foo.Bar.Baz` line the way `Tools/firewall.py`'s
/// regex (`^\s*import\s+([A-Za-z0-9_.]+)`) does: optional leading
/// whitespace, the literal keyword `import`, at least one whitespace
/// character, then a dotted identifier. Returns `None` for any other line
/// (comments, `open`, `namespace`, blank lines, and so on), matching the
/// regex's `re.match` (line-anchored, no match on the token appearing
/// mid-line).
fn parse_import(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    if parts.next()? != "import" {
        return None;
    }
    let rest = parts.next()?.trim_start();
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
        .unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_import_reads_a_dotted_module_after_leading_whitespace() {
        assert_eq!(
            parse_import("import WasmGemmGnaf.Foundation.Types"),
            Some("WasmGemmGnaf.Foundation.Types".to_string())
        );
        assert_eq!(
            parse_import("  import WasmGemmGnaf.GNAF"),
            Some("WasmGemmGnaf.GNAF".to_string())
        );
    }

    #[test]
    fn parse_import_ignores_non_import_lines() {
        assert_eq!(parse_import("-- import WasmGemmGnaf.GNAF"), None);
        assert_eq!(parse_import("namespace WasmGemmGnaf.Foundation"), None);
        assert_eq!(parse_import(""), None);
        assert_eq!(parse_import("theorem foo : True := trivial"), None);
        // The regex is line-anchored: "import" appearing after other tokens
        // on the same line is not a match.
        assert_eq!(parse_import("open import WasmGemmGnaf.GNAF"), None);
    }

    /// Regression guard for `Tools/mutation.py`'s M9: a protected module with
    /// a planted forbidden import must be rejected, and an otherwise-identical
    /// tree with no such import must pass. Built entirely under
    /// `std::env::temp_dir()` on a synthetic layout -- never against the real
    /// vendored tree -- so this test cannot be affected by, or accidentally
    /// mutate, `proofs/wasm-gemm-gnaf/`.
    #[test]
    fn gnaf_firewall_rejects_a_planted_import_and_accepts_a_clean_tree() {
        let root = std::env::temp_dir().join(format!(
            "xtask-gnaf-firewall-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let universal = root.join("proofs/wasm-gemm-gnaf/WasmGemmGnaf/Universal");
        std::fs::create_dir_all(&universal).unwrap();

        let planted = universal.join("Competitor.lean");
        std::fs::write(&planted, "import WasmGemmGnaf.Artifact.Bytes\n").unwrap();
        assert!(gnaf_firewall(&root).is_err());

        std::fs::write(&planted, "import WasmGemmGnaf.Foundation.Types\n").unwrap();
        assert!(gnaf_firewall(&root).is_ok());

        std::fs::remove_dir_all(&root).unwrap();
    }
}
