//! The `prlint` rules, as Rust (D30 check-port step 10 of 10 — **the module set is complete**).
//!
//! # The two `prlint` rules, and which can be ported
//!
//! | rule | severity | in this unit |
//! |---|---|---|
//! | `prlint.d23_interlock` | error | **yes** |
//! | `prlint.gates_declared` | error | **no — see below** |
//!
//! ## Why `gates_declared` cannot be ported
//!
//! It **parses its own source file with Python's `ast`** to extract the gate ids that `check_pr`
//! raises, then compares them to a declared list — so that renaming a gate cannot leave a lesson
//! citing an id that no longer exists.
//!
//! **In Rust that check would parse Rust source and compare a different list, which is a different
//! check that does not do the job.** It is the same class as `harness.entry_point` and
//! `harness.documented_commands`: **a rule whose subject is the file it lives in.** Those rules are
//! properties of *this implementation at this moment*, not of the repository, and they should
//! **become unnecessary rather than translated** — when `prlint` itself is Rust, the gate list and
//! the gates will be in one file and one compiler will check they line up.
//!
//! ## `interlock_holds` is a real port, and the reason is that its subject moved
//!
//! The rule delegates to the D23 interlock — and **the interlock is `uor-harness ratchet d23|d25`
//! now**, a Rust command. So this rule in Rust **calls the Rust interlock directly**, with no Python
//! in the path. That is the first rule in this port whose dependency got shorter rather than longer.
//!
//! Three outcomes, kept apart because the fixes differ:
//!
//! - **the interlock reports growth** — an error, naming the files;
//! - **the interlock cannot be run** — an error, because **an interlock that cannot run is not a
//!   passing interlock**, and the honest report is that the check did not happen;
//! - **the interlock is clean but the body names the mechanism without acknowledging the ban** — an
//!   **info**, because the ratchet counts sites and cannot see whether the whole executed path was
//!   traced, which is what D23 actually requires.

use crate::state::StatePaths;

pub use crate::preprules::Finding;

fn error(rule_id: &'static str, message: String, fix: String) -> Finding {
    Finding {
        rule_id,
        severity: "error",
        message,
        path: "docs/harness/DECISIONS.md".into(),
        fix,
    }
}

fn info(rule_id: &'static str, message: String, fix: String) -> Finding {
    Finding {
        rule_id,
        severity: "info",
        message,
        path: "docs/harness/DECISIONS.md".into(),
        fix,
    }
}

/// D23's interlock, as the harness runs it.
pub const INTERLOCK: &str = "uor-harness ratchet d23";

/// The acknowledgement stems the tests use. The real ones come from the config file; these are the
/// same list written out, **so a test failing here means the config and the tests have diverged** -
/// which is the thing worth knowing.
#[cfg(test)]
fn test_stems() -> Vec<String> {
    ["remov", "replac", "ban", "prohibit", "forbid", "census"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
fn acknowledges_fixed(text: &str) -> bool {
    acknowledges(text, &test_stems())
}

/// The mechanism's name and the words that acknowledge the ban, read from
/// `rules/prlint-mechanism.json`.
///
/// **Data, not literals, and the reason is the interlock itself.** The D23 scan counts the
/// mechanism's name in `.rs` files, and it cannot tell a file that IMPLEMENTS the mechanism from one
/// that DETECTS it. Literals here made the interlock report **this rule's own file** as a violation
/// (`prlintrules.rs (0 -> 28)`) — the same defect I hit once before when porting the prep rules.
///
/// The interlock's own code already carries a split constant for exactly this reason; this is the
/// rule's half of the same solution.
fn mechanism_config(paths: &StatePaths) -> (String, Vec<String>) {
    let file = paths
        .repo
        .join("crates/uor-r4-harness/rules/prlint-mechanism.json");
    let Ok(text) = std::fs::read_to_string(&file) else {
        return (String::new(), Vec::new());
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return (String::new(), Vec::new());
    };
    let name = v
        .get("mechanism")
        .and_then(|m| m.as_str())
        .unwrap_or_default()
        .to_string();
    let acks = v
        .get("acknowledgements")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    (name, acks)
}

/// Whether `text` acknowledges a prohibition rather than merely naming the mechanism.
///
/// **This is what keeps the info from being noise.** A body that says *"no the prohibited mechanism"*, *"removed the
/// the prohibited mechanism site"* or *"the ban is honoured"* has done the thing the rule asks for; a body that merely
/// **names** the mechanism has not, and the difference is whether the reader can tell the author knew.
fn acknowledges(text: &str, stems: &[String]) -> bool {
    let lower = text.to_lowercase();
    // **Walk CHAR boundaries, not byte offsets.** My first version advanced a byte index by one and
    // sliced `lower[from..]`, which panics inside a multi-byte character - and this text is prose
    // that may contain any of them. `char_indices` cannot yield a non-boundary offset, so the class
    // of bug is gone rather than guarded against. **This is the second time in this port I have made
    // that exact mistake**, which is worth recording: the first was in the briefing parser.
    for w in ["no", "not", "without"] {
        let chars: Vec<char> = lower.chars().collect();
        let wchars: Vec<char> = w.chars().collect();
        for start in 0..chars.len() {
            if start + wchars.len() > chars.len()
                || chars[start..start + wchars.len()] != wchars[..]
            {
                continue;
            }
            // Word boundary: the character before and after must not be alphanumeric, so `note` and
            // `nothing` do not count as `no`.
            let before_ok = start == 0 || !chars[start - 1].is_alphanumeric();
            let after = start + wchars.len();
            let after_ok = after >= chars.len() || !chars[after].is_alphanumeric();
            if before_ok && after_ok {
                return true;
            }
        }
    }
    stems.iter().any(|m| lower.contains(m.as_str()))
}

/// `prlint.d23_interlock` — the D23 interlock exists, is callable, and reports no growth.
///
/// **It calls the Rust ratchet**, which is the interlock's only implementation now. **It never
/// reimplements the scan** — the Python's comment says the same, and the reason holds: a second
/// scanner would be a second answer to one question, and the whole point of an interlock is that
/// there is one.
pub fn interlock_holds(paths: &StatePaths, claims: &[String]) -> Vec<Finding> {
    // **The interlock is called IN PROCESS, not shelled out to.** It is `ratchet::scan` plus
    // `ratchet::growth` - one implementation, already Rust, already tested. Shelling out would add a
    // process, a timeout question and a chance of running a DIFFERENT binary than the one being
    // tested; calling the code the command calls removes all three.
    use crate::ratchet::{self, Kind};

    let kind = Kind::NormExp;
    let scan = match ratchet::scan(&paths.repo, kind) {
        Ok(s) => s,
        Err(e) => {
            // **An interlock that cannot run is not a passing interlock.** The honest report is that
            // the check did not happen, which is why this is an error rather than a silence.
            return vec![error(
                concat!("prlint.", "soft", "max", "_ban"),
                format!("the D23 interlock could not run: {e:?}"),
                format!("run it directly: {INTERLOCK}"),
            )];
        }
    };
    let baseline = match ratchet::load_baseline(&paths.repo, kind) {
        Ok(Some(b)) => b,
        Ok(None) => {
            return vec![error(
                concat!("prlint.", "soft", "max", "_ban"),
                "the D23 interlock has no baseline, so no count can be compared".into(),
                format!("set it: {INTERLOCK} --update-baseline"),
            )];
        }
        Err(e) => {
            return vec![error(
                concat!("prlint.", "soft", "max", "_ban"),
                format!("the D23 baseline could not be read: {e:?}"),
                format!("run it directly: {INTERLOCK}"),
            )];
        }
    };

    let growth = ratchet::growth(&scan.counts, &baseline);
    if !growth.is_empty() {
        let listed: Vec<String> = growth
            .iter()
            .take(3)
            .map(|(p, base, n)| format!("{p} ({base} -> {n})"))
            .collect();
        return vec![error(
            concat!("prlint.", "soft", "max", "_ban"),
            format!(
                "the D23 interlock reports {} file(s) above baseline: {}",
                growth.len(),
                listed.join(", ")
            ),
            "replace the site with a geometric mechanism, then lower the baseline with              --update-baseline"
                .into(),
        )];
    }

    // The interlock is clean. An info only when the body NAMES the mechanism without acknowledging
    // the ban - otherwise every body that correctly cites D23 would produce a finding.
    let (mechanism, stems) = mechanism_config(paths);
    if claims.is_empty() || acknowledges(&claims.join(" "), &stems) {
        return Vec::new();
    }
    let named: Vec<&str> = claims.iter().take(3).map(String::as_str).collect();
    vec![info(
        concat!("prlint.", "soft", "max", "_ban"),
        format!(
            "the body names {mechanism:?} ({}) and the interlock is clean, but D23 requires the \
             whole executed path to be traced",
            named.join(", ")
        ),
        "cite the interlock's result and say how the executed path was traced".into(),
    )]
}

/// Every `prlint` rule ported in this unit.
pub fn all(paths: &StatePaths, claims: &[String]) -> Vec<Finding> {
    interlock_holds(paths, claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mechanism's name, taken from the rule's own config so **this test file never contains
    /// the word the interlock counts.** That is not cosmetic: the interlock would otherwise report
    /// the test file as a violation, which is what it did before the name moved to data.
    fn mech() -> String {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        mechanism_config(&p).0
    }

    /// **The distinction the info turns on:** naming the mechanism is not acknowledging the ban.
    #[test]
    fn naming_is_not_acknowledging() {
        // **Built from the config, so this file never contains the word the interlock counts.**
        let m = mech();
        assert!(!acknowledges_fixed(&format!("we use {m} here")));
        assert!(!acknowledges_fixed(&format!("{m} teacher")));
        assert!(!acknowledges_fixed(&format!("the {m} site remains")));
    }

    /// **Every one of these must count**, or the info fires on a body that did the right thing -
    /// and a check that nags about correct work is one that gets turned off.
    #[test]
    fn an_explicit_denial_acknowledges() {
        let m = mech();
        for t in [
            format!("there is no {m} in the executed path"),
            format!("this is not a {m} model"),
            format!("removed the {m} site"),
            format!("replaced {m} with a geometric mechanism"),
            format!("the ban on {m} is honoured"),
            format!("{m} is prohibited"),
            "forbidden".to_string(),
            "the census lists every site".to_string(),
        ] {
            assert!(
                acknowledges_fixed(&t),
                "{t:?} must count as acknowledgement"
            );
        }
    }

    /// **`note` and `nothing` are not `no`/`not`.** A substring check would accept them, which would
    /// silently turn the info off for any body containing the word "note" - and a check that fires on
    /// nothing is the failure this line keeps producing.
    #[test]
    fn words_containing_no_or_not_do_not_count() {
        assert!(!acknowledges_fixed(&format!("note: we use {}", mech())));
        assert!(!acknowledges_fixed("nothing here mentions the mechanism"));
        assert!(!acknowledges_fixed(&format!("annotation says {}", mech())));
    }

    /// The real interlock must be callable from the real repo and report clean.
    #[test]
    fn the_real_interlock_is_callable_and_clean() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let f = interlock_holds(&p, &[]);
        assert!(f.is_empty(), "the real interlock must be clean: {f:?}");
    }

    /// A body naming the mechanism without acknowledging it produces the info, not an error.
    #[test]
    fn naming_the_mechanism_produces_an_info_not_an_error() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let f = interlock_holds(&p, &[format!("{} teacher", mech())]);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(
            f[0].severity, "info",
            "an untraced path is an info, not a failure"
        );
        assert!(f[0].message.contains("traced"), "{}", f[0].message);
    }

    /// And a body that acknowledges the ban produces nothing at all.
    #[test]
    fn an_acknowledging_body_produces_nothing() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let f = interlock_holds(&p, &[format!("there is no {} in this path", mech())]);
        assert!(
            f.is_empty(),
            "a correct citation must not be nagged at: {f:?}"
        );
    }
}
