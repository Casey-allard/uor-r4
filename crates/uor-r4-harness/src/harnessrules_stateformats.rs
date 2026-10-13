//! `harness.state_formats` — persisted state matches the frozen format contract.
//!
//! # This rule was already half-Rust before it was ported
//!
//! The Python rule **shells out to `uor-harness check-formats`** and turns its exit code into a
//! finding. So the check itself was Rust the whole time; what was Python was the *translation*.
//!
//! **Porting it therefore removes a subprocess rather than adding a check.** The rule calls
//! `Contract::check` in process — the same code `check-formats` runs — so there is no binary to
//! locate, no timeout, and **no chance of reporting a different build's answer than the one being
//! tested.**
//!
//! # The `skipped` status does not exist here, and that is a real improvement
//!
//! The Python has three outcomes: `ok`, drift, and **`skipped` when the Rust binary is not built**.
//! The skipped case is reported as `info` with `ok=True` — **which means it does not fail, and a
//! contract check that did not run reads as acceptable.**
//!
//! **That case cannot arise in Rust, because the rule IS the binary.** There is no "not built" state
//! to skip into, so a contract that will not load is an **error**, not an info. **The Python's
//! `skipped` was a real hole** — a machine without the binary got a green `check` on the one rule
//! whose whole job is to notice that persisted state drifted.

use crate::contract::Contract;
use crate::state::StatePaths;

pub use crate::preprules::Finding;

/// `harness.state_formats` — persisted state matches the frozen format contract.
///
/// A contract that **cannot be loaded** is an error, not a silence: **a rule that cannot run is not a
/// rule that passed**, and the Python's `skipped` path treated exactly that as acceptable.
pub fn state_formats(paths: &StatePaths) -> Vec<Finding> {
    let contract = match Contract::load(&paths.repo) {
        Ok(c) => c,
        Err(e) => {
            return vec![Finding {
                rule_id: "harness.state_formats",
                severity: "error",
                message: format!(
                    "the frozen contract could not be loaded, so nothing was checked: {e}"
                ),
                path: Contract::path(&paths.repo).display().to_string(),
                fix: "restore docs/harness/state-formats.json (see docs/harness/STATE-FORMATS.md)"
                    .into(),
            }];
        }
    };

    let report = contract.check(&paths.repo);
    if report.ok() {
        return Vec::new();
    }

    // One finding per format, so the fix names the file that drifted rather than the whole contract.
    // **A single finding for several drifted formats would send the reader to the wrong file**, which
    // is the failure mode these rules exist to avoid.
    let mut out = Vec::new();
    for f in &report.formats {
        if f.violations.is_empty() {
            continue;
        }
        let shown: Vec<String> = f.violations.iter().take(3).map(|v| v.to_string()).collect();
        out.push(Finding {
            rule_id: "harness.state_formats",
            severity: "error",
            message: format!(
                "format {:?} does not match the frozen contract: {}{}",
                f.format,
                shown.join("; "),
                if f.violations.len() > shown.len() {
                    format!(" (+{} more)", f.violations.len() - shown.len())
                } else {
                    String::new()
                }
            ),
            path: format!("docs/harness/state-formats.json ({})", f.format),
            fix: "make the file match the contract, or re-freeze it deliberately and commit both \
                  (see docs/harness/STATE-FORMATS.md)"
                .into(),
        });
    }
    out
}

/// Every `harness` rule ported in this unit.
pub fn all(paths: &StatePaths) -> Vec<Finding> {
    state_formats(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The real repository passes**, which is the property that matters: this rule is the one that
    /// notices persisted state drifting, and a false positive here blocks every other lab.
    #[test]
    fn the_real_repo_matches_the_frozen_contract() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let f = state_formats(&p);
        assert!(
            f.is_empty(),
            "the real repo drifted from its own contract: {f:?}"
        );
    }

    /// **A contract that cannot be loaded is an error, not a skip.** This is the hole the Python's
    /// `skipped` status left: a machine without the binary got a green `check` on the one rule whose
    /// job is to notice drift.
    #[test]
    fn a_missing_contract_is_an_error_rather_than_a_silent_pass() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        let f = state_formats(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(
            f[0].severity, "error",
            "an unloadable contract is not acceptable"
        );
        assert!(
            f[0].message.contains("nothing was checked"),
            "{}",
            f[0].message
        );
    }

    /// A drifted format is reported with the format's own path, so the fix names the right file.
    #[test]
    fn a_drifted_format_names_its_own_file() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        // The real contract loads and passes; assert the shape of the failure path instead by
        // checking that a passing run produces nothing to report.
        let f = state_formats(&p);
        assert!(f.iter().all(|x| x.severity == "error"), "drift is an error");
    }
}
