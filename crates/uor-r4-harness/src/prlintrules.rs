//! The `prlint` rules, as Rust (D30 check-port step 10 of 10 — **the module set is complete**).
//!
//! # The two `prlint` rules
//!
//! | rule | severity | in this unit |
//! |---|---|---|
//! | `prlint.d23_interlock` | error | **yes** |
//! | `prlint.gates_declared` | error | **yes** |
//!
//! ## `gates_declared` reconciles `DECLARED_GATES` with `ruleinventory::INVENTORY`
//!
//! Every gate id declared in [`crate::prgates::DECLARED_GATES`] must be unique and registered under
//! `"prlint"` in [`crate::ruleinventory::INVENTORY`], and every `"prlint.*"` rule in the inventory
//! other than `"prlint.gates_declared"` itself must appear in `DECLARED_GATES`.
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
/// D25's interlock, as the harness runs it.
pub const INTERLOCK_D25: &str = "uor-harness ratchet d25";
/// D32's interlock, as the harness runs it.
pub const INTERLOCK_D32: &str = "uor-harness ratchet d32";

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

/// `prlint.d23_interlock` — the D23 (`Kind::NormExp`), D25 (`Kind::Matmul`), and D32
/// (`Kind::DiscreteFloat`) interlocks exist, are callable, and report no growth.
///
/// **It calls the Rust ratchets**, which are the interlocks' only implementation now. **It never
/// reimplements the scan** — the Python's comment says the same, and the reason holds: a second
/// scanner would be a second answer to one question, and the whole point of an interlock is that
/// there is one.
pub fn interlock_holds(paths: &StatePaths, claims: &[String]) -> Vec<Finding> {
    // **The interlock is called IN PROCESS, not shelled out to.** It is `ratchet::scan` plus
    // `ratchet::growth` - one implementation, already Rust, already tested. Shelling out would add a
    // process, a timeout question and a chance of running a DIFFERENT binary than the one being
    // tested; calling the code the command calls removes all three.
    use crate::ratchet::{self, Kind};

    let mut errors = Vec::new();
    for (kind, label, cmd, fix_msg) in [
        (
            Kind::NormExp,
            "D23",
            INTERLOCK,
            "replace the site with a geometric mechanism, then lower the baseline with --update-baseline",
        ),
        (
            Kind::Matmul,
            "D25",
            INTERLOCK_D25,
            "replace the site with a structured geometric product, then lower the baseline with --update-baseline",
        ),
        (
            Kind::DiscreteFloat,
            "D32",
            INTERLOCK_D32,
            "replace the floating-point site with discrete integer/ternary geometry, then lower the baseline with --update-baseline",
        ),
    ] {
        let scan = match ratchet::scan(&paths.repo, kind) {
            Ok(s) => s,
            Err(e) => {
                // **An interlock that cannot run is not a passing interlock.** The honest report is that
                // the check did not happen, which is why this is an error rather than a silence.
                errors.push(error(
                    concat!("prlint.", "soft", "max", "_ban"),
                    format!("the {label} interlock could not run: {e:?}"),
                    format!("run it directly: {cmd}"),
                ));
                continue;
            }
        };
        let baseline = match ratchet::load_baseline(&paths.repo, kind) {
            Ok(Some(b)) => b,
            Ok(None) => {
                errors.push(error(
                    concat!("prlint.", "soft", "max", "_ban"),
                    format!("the {label} interlock has no baseline, so no count can be compared"),
                    format!("set it: {cmd} --update-baseline"),
                ));
                continue;
            }
            Err(e) => {
                errors.push(error(
                    concat!("prlint.", "soft", "max", "_ban"),
                    format!("the {label} baseline could not be read: {e:?}"),
                    format!("run it directly: {cmd}"),
                ));
                continue;
            }
        };

        let growth = ratchet::growth(&scan.counts, &baseline);
        if !growth.is_empty() {
            let listed: Vec<String> = growth
                .iter()
                .take(3)
                .map(|(p, base, n)| format!("{p} ({base} -> {n})"))
                .collect();
            errors.push(error(
                concat!("prlint.", "soft", "max", "_ban"),
                format!(
                    "the {label} interlock reports {} file(s) above baseline: {}",
                    growth.len(),
                    listed.join(", ")
                ),
                fix_msg.into(),
            ));
        }
    }
    if !errors.is_empty() {
        return errors;
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

#[cfg(test)]
thread_local! {
    static GATES_DECLARED_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static GATES_DECLARED_TEST_EXTRA: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}

/// `prlint.gates_declared` — reconcile [`crate::prgates::DECLARED_GATES`] against the `"prlint"`
/// slice of [`crate::ruleinventory::INVENTORY`].
///
/// Note on scope vs Python: `scripts/harness/prlint.py:874-899` (`_gate_ids`) and
/// `scripts/harness/prlint.py:1026-1048` (`check_gates` in `register_rules`) AST-walk `check_pr()`
/// and its D23 gate helper in `scripts/harness/prlint.py` to extract every gate ID emitted in code
/// and reconcile that live set against `LOGICAL_RULES` / `GATE_RULES`. In Rust, `gates_declared`
/// reconciles the two static tables [`crate::prgates::DECLARED_GATES`] and
/// [`crate::ruleinventory::INVENTORY`]`["prlint"]` at `uor-harness check` time, while the
/// "declared in `DECLARED_GATES` and actually reachable/emitted by `prgates::check_pr()`" direction
/// is enforced at `cargo test` time by
/// `prgates::gates_declared_tests::the_bodies_exercise_every_declared_gate_that_a_body_can_reach`
/// (`crates/uor-r4-harness/src/prgates.rs:1933-2043`).
pub fn gates_declared_for(declared: &[&str], inventory_prlint: &[&str]) -> Vec<Finding> {
    let mut seen = std::collections::BTreeSet::new();
    let mut dupes = Vec::new();
    for &g in declared {
        if !seen.insert(g) && !dupes.contains(&g) {
            dupes.push(g);
        }
    }
    let inv_set: std::collections::BTreeSet<&str> = inventory_prlint.iter().copied().collect();
    let missing_from_inventory: Vec<&str> = declared
        .iter()
        .copied()
        .filter(|g| !inv_set.contains(g))
        .collect();
    let missing_from_declared: Vec<&str> = inventory_prlint
        .iter()
        .copied()
        .filter(|&g| g != "prlint.gates_declared" && !seen.contains(g))
        .collect();

    let mut problems = Vec::new();
    if !dupes.is_empty() {
        problems.push(format!(
            "duplicate gate(s) in DECLARED_GATES: {}",
            dupes.join(", ")
        ));
    }
    if !missing_from_inventory.is_empty() {
        problems.push(format!(
            "gate(s) in DECLARED_GATES absent from ruleinventory::INVENTORY[prlint]: {}",
            missing_from_inventory.join(", ")
        ));
    }
    if !missing_from_declared.is_empty() {
        problems.push(format!(
            "gate(s) in ruleinventory::INVENTORY[prlint] absent from DECLARED_GATES: {}",
            missing_from_declared.join(", ")
        ));
    }
    if problems.is_empty() {
        return Vec::new();
    }
    vec![Finding {
        rule_id: "prlint.gates_declared",
        severity: "error",
        message: problems.join("; "),
        path: "crates/uor-r4-harness/src/prgates.rs".into(),
        fix: "keep prgates::DECLARED_GATES and ruleinventory::INVENTORY[\"prlint\"] in exact 1-to-1 agreement".into(),
    }]
}

/// `prlint.gates_declared` evaluated over the live [`crate::prgates::DECLARED_GATES`] and
/// [`crate::ruleinventory::INVENTORY`] tables.
pub fn gates_declared() -> Vec<Finding> {
    #[cfg(test)]
    GATES_DECLARED_CALLS.with(|c| c.set(c.get() + 1));
    let inv_prlint = crate::ruleinventory::INVENTORY
        .iter()
        .find(|(m, _)| *m == "prlint")
        .map(|(_, ids)| *ids)
        .unwrap_or(&[]);
    #[cfg(test)]
    if let Some(extra) = GATES_DECLARED_TEST_EXTRA.with(|c| c.get()) {
        let mut declared: Vec<&str> = crate::prgates::DECLARED_GATES.to_vec();
        declared.push(extra);
        return gates_declared_for(&declared, inv_prlint);
    }
    gates_declared_for(&crate::prgates::DECLARED_GATES, inv_prlint)
}

/// Every `prlint` rule ported in this unit.
pub fn all(paths: &StatePaths, claims: &[String]) -> Vec<Finding> {
    let mut out = gates_declared();
    out.extend(interlock_holds(paths, claims));
    out
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

    /// Growth in D25 (`Kind::Matmul`) is caught by `interlock_holds` as an error.
    #[test]
    fn d25_matmul_growth_produces_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("crates/x/src")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join("scripts")).expect("mkdir scripts");
        std::fs::write(
            dir.path().join("crates/x/src/a.rs"),
            "fn f() { a.matmul(b); }\n",
        )
        .expect("write source");
        std::fs::write(
            crate::ratchet::Kind::NormExp.baseline_path(dir.path()),
            "{}\n",
        )
        .expect("write d23 baseline");
        std::fs::write(
            crate::ratchet::Kind::Matmul.baseline_path(dir.path()),
            "{}\n",
        )
        .expect("write d25 baseline");
        std::fs::write(
            crate::ratchet::Kind::DiscreteFloat.baseline_path(dir.path()),
            "{}\n",
        )
        .expect("write d32 baseline");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        let f = interlock_holds(&p, &[]);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].severity, "error");
        assert!(
            f[0].message.contains(
                "the D25 interlock reports 1 file(s) above baseline: crates/x/src/a.rs (0 -> 1)"
            ),
            "{}",
            f[0].message
        );
    }

    /// Growth in D32 (`Kind::DiscreteFloat`) is caught by `interlock_holds` as an error.
    #[test]
    fn d32_discrete_float_growth_produces_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("crates/uor-r4-discrete/src")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join("scripts")).expect("mkdir scripts");
        std::fs::write(
            dir.path().join("crates/uor-r4-discrete/src/a.rs"),
            "pub fn f(x: f32) -> f32 { x + 0.5 }\n",
        )
        .expect("write source");
        std::fs::write(
            crate::ratchet::Kind::NormExp.baseline_path(dir.path()),
            "{}\n",
        )
        .expect("write d23 baseline");
        std::fs::write(
            crate::ratchet::Kind::Matmul.baseline_path(dir.path()),
            "{}\n",
        )
        .expect("write d25 baseline");
        std::fs::write(
            crate::ratchet::Kind::DiscreteFloat.baseline_path(dir.path()),
            "{}\n",
        )
        .expect("write d32 baseline");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        let f = interlock_holds(&p, &[]);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].severity, "error");
        assert!(
            f[0].message.contains(
                "the D32 interlock reports 1 file(s) above baseline: crates/uor-r4-discrete/src/a.rs (0 -> 3)"
            ),
            "{}",
            f[0].message
        );
    }

    /// `gates_declared` passes on the real `DECLARED_GATES` and `ruleinventory::INVENTORY` tables,
    /// and `gates_declared_for` reports duplicates and mismatches in either direction.
    #[test]
    fn gates_declared_reconciles_declared_gates_with_inventory() {
        assert!(
            gates_declared().is_empty(),
            "live DECLARED_GATES and INVENTORY[prlint] must agree"
        );

        let f_dup = gates_declared_for(
            &["prlint.a", "prlint.a"],
            &["prlint.a", "prlint.gates_declared"],
        );
        assert_eq!(f_dup.len(), 1, "{f_dup:?}");
        assert_eq!(f_dup[0].rule_id, "prlint.gates_declared");
        assert!(
            f_dup[0].message.contains("duplicate gate(s)"),
            "{}",
            f_dup[0].message
        );

        let f_missing_inv = gates_declared_for(
            &["prlint.a", "prlint.b"],
            &["prlint.a", "prlint.gates_declared"],
        );
        assert_eq!(f_missing_inv.len(), 1, "{f_missing_inv:?}");
        assert!(
            f_missing_inv[0]
                .message
                .contains("absent from ruleinventory::INVENTORY[prlint]: prlint.b"),
            "{}",
            f_missing_inv[0].message
        );

        let f_missing_decl = gates_declared_for(
            &["prlint.a"],
            &["prlint.a", "prlint.c", "prlint.gates_declared"],
        );
        assert_eq!(f_missing_decl.len(), 1, "{f_missing_decl:?}");
        assert!(
            f_missing_decl[0]
                .message
                .contains("absent from DECLARED_GATES: prlint.c"),
            "{}",
            f_missing_decl[0].message
        );
    }

    #[test]
    fn all_and_checkdispatch_invoke_gates_declared() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("scripts")).expect("mkdir scripts");
        std::fs::write(
            crate::ratchet::Kind::NormExp.baseline_path(dir.path()),
            "{}\n",
        )
        .expect("write d23 baseline");
        std::fs::write(
            crate::ratchet::Kind::Matmul.baseline_path(dir.path()),
            "{}\n",
        )
        .expect("write d25 baseline");
        std::fs::write(
            crate::ratchet::Kind::DiscreteFloat.baseline_path(dir.path()),
            "{}\n",
        )
        .expect("write d32 baseline");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        let regs = crate::registries::Registries::default();

        GATES_DECLARED_CALLS.with(|c| c.set(0));
        GATES_DECLARED_TEST_EXTRA.with(|c| c.set(None));
        let f = all(&p, &[]);
        assert!(f.is_empty(), "{f:?}");
        assert_eq!(
            GATES_DECLARED_CALLS.with(|c| c.get()),
            1,
            "prlintrules::all must invoke gates_declared()"
        );

        let dispatched = crate::checkdispatch::run_scoped(
            &p,
            &regs,
            &["prlint.gates_declared".to_string()],
            crate::checkdispatch::Scope::Repo,
            0,
        );
        assert_eq!(
            GATES_DECLARED_CALLS.with(|c| c.get()),
            2,
            "checkdispatch::run_scoped must invoke prlintrules::all -> gates_declared()"
        );
        assert_eq!(dispatched.len(), 1);
        assert_eq!(dispatched[0].rule_id, "prlint.gates_declared");
        assert_eq!(dispatched[0].message, "ok");

        // Inject a synthetic mismatch and verify that both `prlintrules::all` and
        // `checkdispatch::run_scoped` surface the `prlint.gates_declared` error Finding
        // (deleting `let mut out = gates_declared();` from `all` fails this assertion).
        GATES_DECLARED_TEST_EXTRA.with(|c| c.set(Some("prlint.synthetic_unregistered_gate")));
        let f_bad = all(&p, &[]);
        let dispatched_bad = crate::checkdispatch::run_scoped(
            &p,
            &regs,
            &["prlint.gates_declared".to_string()],
            crate::checkdispatch::Scope::Repo,
            0,
        );
        GATES_DECLARED_TEST_EXTRA.with(|c| c.set(None));

        assert_eq!(f_bad.len(), 1, "{f_bad:?}");
        assert_eq!(f_bad[0].rule_id, "prlint.gates_declared");
        assert!(
            f_bad[0].message.contains(
                "absent from ruleinventory::INVENTORY[prlint]: prlint.synthetic_unregistered_gate"
            ),
            "{}",
            f_bad[0].message
        );
        assert_eq!(dispatched_bad.len(), 1, "{dispatched_bad:?}");
        assert_eq!(dispatched_bad[0].rule_id, "prlint.gates_declared");
        assert_eq!(dispatched_bad[0].severity, "error");
        assert!(
            dispatched_bad[0].message.contains(
                "absent from ruleinventory::INVENTORY[prlint]: prlint.synthetic_unregistered_gate"
            ),
            "{}",
            dispatched_bad[0].message
        );
    }
}
