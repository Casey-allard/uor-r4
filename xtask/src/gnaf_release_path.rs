//! GNAF release-path computability check (SPEC 19 / 6.3), ported from the
//! vendored `proofs/wasm-gemm-gnaf/Tools/releasepath.py` (#653 phase-2
//! follow-up, fourth `Tools/*.py` port after `gnaf_firewall.rs`,
//! `gnaf_root.rs`, and `gnaf_scan.rs`).
//!
//! SPEC 19 excludes `noncomputable` definitions from the product/proof
//! path, and SPEC 6.3 requires executable proof-producing functions to be
//! computable. `gnaf-scan` deliberately ignores comments and so cannot see
//! a `noncomputable def` sitting in real code either -- this check exists
//! precisely because that scan's blind spot for declaration *keywords*
//! (as opposed to banned words) is a different check with a different
//! target: the modules that constitute the release path specifically, not
//! the whole tree. A classically-chosen evaluator decodes, validates,
//! enumerates and executes nothing; it cannot stand in for the implemented
//! explorer.
//!
//! Reuses `gnaf_scan::strip_comments_and_strings`, mirroring
//! `Tools/releasepath.py`'s own `from scan import strip`.
//!
//! Unlike `gnaf-firewall`, `gnaf-root`, and `gnaf-scan`, this check
//! currently and expectedly FAILS against the real vendored tree: two
//! `noncomputable def`s in `WasmGemmGnaf/Artifact/Release.lean`
//! (`evaluateClassically`, `decider`), gated on the still-outstanding
//! WGG-GO-1 release theorem -- the exact condition `Tools/gate.py` itself
//! documents as "expected to fail... that failure is the conforming
//! behavior" while it's outstanding. For that reason `cargo xtask
//! validate` deliberately does NOT run this check (see the comment on
//! `validate` in `main.rs`); it's available on its own as `cargo xtask
//! gnaf-release-path` for whoever is tracking WGG-GO-1's progress.

use std::path::{Path, PathBuf};

use crate::gnaf_scan::strip_comments_and_strings;
use crate::Fail;

/// The modules that constitute the release path (SPEC 19's own list),
/// relative to `proofs/wasm-gemm-gnaf/WasmGemmGnaf/`.
const RELEASE_PATH: [&str; 3] = ["Artifact", "Theorems", "Universal"];

/// SPEC 19 / 6.3: no `noncomputable def`/`abbrev`/`instance` on the
/// release path.
pub fn gnaf_release_path(root: &Path) -> Result<(), Fail> {
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
    for layer in RELEASE_PATH {
        let dir = base.join(layer);
        if dir.exists() {
            collect_lean(&dir, &mut files)?;
        }
    }
    files.sort();

    let mut hits = Vec::new();
    for path in &files {
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8_lossy(&bytes);
        let stripped = strip_comments_and_strings(&text);
        for (i, line) in stripped.lines().enumerate() {
            if let Some((decl, name)) = match_noncomputable(line) {
                hits.push(format!(
                    "{}:{}: noncomputable {decl} {name}",
                    path.display(),
                    i + 1
                ));
            }
        }
    }

    if !hits.is_empty() {
        return Err(format!(
            "NONCOMPUTABLE ON THE RELEASE PATH (SPEC 19 / 6.3):\n{}\n\n\
             SPEC 19 excludes noncomputable definitions from the product/proof \
             path. A classically-chosen evaluator decodes, validates, enumerates \
             and executes nothing; it cannot stand in for the implemented \
             explorer.",
            hits.iter()
                .map(|h| format!("  {h}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
        .into());
    }
    println!(
        "gnaf-release-path: release path computable: {} modules, no \
         noncomputable definition (SPEC 19 / 6.3)",
        files.len()
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

/// `^\s*noncomputable\s+(def|abbrev|instance)\s+(\S+)`, matched by hand
/// (no regex dependency, matching this port series' existing discipline).
/// Returns the declaration keyword and the declared name on a match.
fn match_noncomputable(line: &str) -> Option<(&'static str, String)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("noncomputable")?;
    let after_kw = rest.trim_start();
    if after_kw.len() == rest.len() {
        return None; // \s+ needs at least one whitespace char
    }
    for decl in ["def", "abbrev", "instance"] {
        let Some(rest2) = after_kw.strip_prefix(decl) else {
            continue;
        };
        let after_decl = rest2.trim_start();
        if after_decl.len() == rest2.len() {
            continue; // \s+ needs at least one whitespace char before the name
        }
        let name: String = after_decl
            .chars()
            .take_while(|c| !c.is_whitespace())
            .collect();
        if name.is_empty() {
            continue; // \S+ needs at least one non-whitespace character
        }
        return Some((decl, name));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_noncomputable_captures_the_declaration_kind_and_name() {
        assert_eq!(
            match_noncomputable("noncomputable def globalOptimal : Prop := trivial"),
            Some(("def", "globalOptimal".to_string()))
        );
        assert_eq!(
            match_noncomputable("  noncomputable instance Foo : Bar where"),
            Some(("instance", "Foo".to_string()))
        );
        assert_eq!(
            match_noncomputable("noncomputable abbrev X := Y"),
            Some(("abbrev", "X".to_string()))
        );
    }

    #[test]
    fn match_noncomputable_rejects_non_matching_lines() {
        // computable (no "noncomputable" keyword at all).
        assert_eq!(
            match_noncomputable("def globalOptimal : Prop := trivial"),
            None
        );
        // "noncomputable" as part of a longer identifier is not the keyword.
        assert_eq!(match_noncomputable("noncomputableFoo bar"), None);
        // a declaration kind this check doesn't cover.
        assert_eq!(
            match_noncomputable("noncomputable theorem foo : True"),
            None
        );
        // no whitespace between the keyword and the declaration kind.
        assert_eq!(match_noncomputable("noncomputabledef foo"), None);
    }

    fn scratch_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "xtask-gnaf-release-path-test-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }

    /// A `noncomputable def` inside a release-path layer (Artifact/
    /// Theorems/Universal) must be rejected; the identical text in a layer
    /// outside the release path (e.g. Foundation/) must not be, since this
    /// check's whole point is scope, not a tree-wide ban. Built entirely
    /// under `std::env::temp_dir()`, never against the real vendored tree.
    #[test]
    fn gnaf_release_path_rejects_noncomputable_only_inside_the_release_path() {
        let root = scratch_dir("scope");
        let base = root.join("proofs/wasm-gemm-gnaf/WasmGemmGnaf");
        std::fs::create_dir_all(base.join("Artifact")).unwrap();
        std::fs::create_dir_all(base.join("Foundation")).unwrap();
        std::fs::write(
            base.join("Foundation/Scratch.lean"),
            "noncomputable def notOnTheReleasePath : Prop := trivial\n",
        )
        .unwrap();
        assert!(gnaf_release_path(&root).is_ok());

        std::fs::write(
            base.join("Artifact/Bytes.lean"),
            "noncomputable def onTheReleasePath : Prop := trivial\n",
        )
        .unwrap();
        assert!(gnaf_release_path(&root).is_err());

        std::fs::remove_dir_all(&root).unwrap();
    }

    /// A `noncomputable` mentioned only inside a comment or string must not
    /// trip the check -- it reuses `gnaf_scan`'s stripper for exactly this
    /// reason.
    #[test]
    fn gnaf_release_path_ignores_a_commented_mention() {
        let root = scratch_dir("comment");
        let base = root.join("proofs/wasm-gemm-gnaf/WasmGemmGnaf/Theorems");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(
            base.join("Doc.lean"),
            "-- noncomputable def mentioned in a comment, not real code\n\
             def real : Prop := trivial\n",
        )
        .unwrap();
        assert!(gnaf_release_path(&root).is_ok());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
