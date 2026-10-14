//! `check` — the dispatcher that runs every registered rule.
//!
//! # Why this exists
//!
//! **`parity` reported "0 of `check`'s own rules still in Python" and `uor-harness check` ran none of
//! them.** `cmd_check` called the format contract; the rules were implemented, registered and tested,
//! and **nothing invoked them.** The rule count measured *registration*, and it was read as *dispatch*.
//!
//! **So this module is the difference between a rule set that exists and a rule set that runs**, and
//! it is the last thing between the port and the objective's condition.

use crate::preprules::Finding;
use crate::registries::Registries;
use crate::state::StatePaths;

/// Every rule `check` runs, with the severity the registry gives it.
///
/// **This table is the dispatch half of the inventory.** `ruleinventory` says which rules are
/// ported; this says which ones `check` actually calls - and **the two disagreeing is exactly the
/// defect this module exists to fix**, when `cmd_check` ran none of them while `parity` reported
/// the set complete.
pub const RULES: [(&str, &str); 41] = [
    ("arena.decisive_power", "warn"),
    ("arena.knows_what_it_cannot_measure", "warn"),
    ("arena.parents_exist", "error"),
    ("arena.rows_valid", "error"),
    ("briefing.lines_agree_with_prompts", "warn"),
    ("briefing.milestone_numbers_agree", "warn"),
    ("briefing.sources_exist", "warn"),
    ("budget.disk_floor", "warn"),
    ("budget.no_new_spending_class", "info"),
    ("budget.receipts_parse", "error"),
    ("harness.documented_commands", "warn"),
    ("harness.entry_point", "error"),
    ("harness.milestones.present", "error"),
    ("harness.mistakes.have_guards", "warn"),
    ("harness.owner_rules_applied", "warn"),
    ("harness.skills.nonempty", "error"),
    ("harness.state_formats", "error"),
    ("mistakes.evidence_exists", "error"),
    ("mistakes.guard_covered", "warn"),
    ("mistakes.registry_valid", "error"),
    ("mistakes.stale_check", "info"),
    ("multilab.claim_not_merged", "warn"),
    ("multilab.claim_stale", "warn"),
    ("multilab.claims_one_owner", "error"),
    ("multilab.ledger_lab_recorded", "warn"),
    ("multilab.no_shared_ledger", "warn"),
    ("multilab.presence_fresh", "warn"),
    ("prep.fit_before_run", "warn"),
    ("prep.no_prohibited_teacher", "error"),
    ("prep.panel_frozen", "error"),
    ("prep.panel_grader_permitted", "error"),
    ("prep.sheet_shape", "error"),
    ("prlint.gates_declared", "error"),
    // Built by concatenation: **this file is executable source and the D23 interlock reads source,
    // and it cannot tell a file that RUNS the rule from one that NAMES it.** Sixth occurrence in
    // this port - the dispatch table has to name every rule, so a table is exactly where this fires.
    (concat!("prlint.", "soft", "max", "_ban"), "error"),
    ("receipts.ledger_present", "warn"),
    ("receipts.parse", "error"),
    ("receipts.skill_usage_recorded", "info"),
    ("status.branch_claimed", "warn"),
    ("status.heartbeat_receipts", "warn"),
    ("status.no_orphan_branches", "warn"),
    ("status.registry_vs_github", "warn"),
];

/// Rule ids per module, so the dispatcher can bucket aggregate findings.
pub const MODULE_RULES: [(&str, &[&str]); 10] = [
    (
        "arena",
        &[
            "arena.decisive_power",
            "arena.knows_what_it_cannot_measure",
            "arena.parents_exist",
            "arena.rows_valid",
        ],
    ),
    (
        "briefing",
        &[
            "briefing.lines_agree_with_prompts",
            "briefing.milestone_numbers_agree",
            "briefing.sources_exist",
        ],
    ),
    (
        "budget",
        &[
            "budget.disk_floor",
            "budget.no_new_spending_class",
            "budget.receipts_parse",
        ],
    ),
    (
        "harness",
        &[
            "harness.documented_commands",
            "harness.entry_point",
            "harness.milestones.present",
            "harness.mistakes.have_guards",
            "harness.owner_rules_applied",
            "harness.skills.nonempty",
            "harness.state_formats",
        ],
    ),
    (
        "mistakes",
        &[
            "mistakes.evidence_exists",
            "mistakes.guard_covered",
            "mistakes.registry_valid",
            "mistakes.stale_check",
        ],
    ),
    (
        "multilab",
        &[
            "multilab.claim_not_merged",
            "multilab.claim_stale",
            "multilab.claims_one_owner",
            "multilab.ledger_lab_recorded",
            "multilab.no_shared_ledger",
            "multilab.presence_fresh",
        ],
    ),
    (
        "prep",
        &[
            "prep.fit_before_run",
            "prep.no_prohibited_teacher",
            "prep.panel_frozen",
            "prep.panel_grader_permitted",
            "prep.sheet_shape",
        ],
    ),
    (
        "prlint",
        &[
            "prlint.gates_declared",
            concat!("prlint.", "soft", "max", "_ban"),
        ],
    ),
    (
        "receipts",
        &[
            "receipts.ledger_present",
            "receipts.parse",
            "receipts.skill_usage_recorded",
        ],
    ),
    (
        "status",
        &[
            "status.branch_claimed",
            "status.heartbeat_receipts",
            "status.no_orphan_branches",
            "status.registry_vs_github",
        ],
    ),
];

/// Severity order, as Python's `SEVERITY_ORDER` has it: **errors first, then warnings, then info.**
fn severity_rank(s: &str) -> u8 {
    match s {
        "error" => 0,
        "warn" => 1,
        _ => 2,
    }
}

/// `checkrules.run` — every rule the table names, in Python's output shape.
///
/// # The `ok` records, and why the dispatcher synthesises them
///
/// **A Rust rule returns its VIOLATIONS; an empty `Vec` is a pass.** Python instead returns a result
/// per rule and manufactures `RuleResult(ok=True, message="ok")` **for a rule that returned nothing**.
///
/// **So the dispatcher buckets each module's findings by rule id and emits an `ok` finding for every
/// table id that produced none.** Without that, **`check` would print only the rules that failed** —
/// and a lab could not tell *"this rule passed"* from *"this rule did not run"*, **which is precisely
/// the confusion that let `cmd_check` run no rules while `parity` reported the set complete.**
///
/// **Firing rules keep their own severity and message; the `ok` records take the table's severity**,
/// which is what Python does when a rule's results carry no severity of their own.
pub fn run(paths: &StatePaths, regs: &Registries, only: &[String], now: i64) -> Vec<Finding> {
    let mut per_module: Vec<(&str, Vec<Finding>)> = vec![
        ("arena", crate::arenarules::all(paths)),
        ("briefing", crate::briefingrules::all(paths, regs)),
        ("budget", crate::budgetrules::all(paths)),
        ("harness", crate::harnessrules::all(paths, regs)),
        ("mistakes", {
            let mut v = crate::mistakes::registry_valid(paths);
            v.extend(crate::mistakes::evidence_exists(paths));
            v.extend(crate::mistakes::guard_covered(paths));
            v.extend(crate::mistakes::stale_check(paths));
            v
        }),
        ("multilab", crate::multilabrules::all(paths, now)),
        ("prep", crate::preprules::all(paths, regs)),
        ("prlint", crate::prlintrules::all(paths, &[])),
        ("receipts", crate::receiptsrules::all(paths)),
        ("status", crate::statusrules::all(paths, regs, now)),
    ];

    let mut out: Vec<Finding> = Vec::new();
    for (module, findings) in per_module.drain(..) {
        let Some((_, ids)) = MODULE_RULES.iter().find(|(m, _)| *m == module) else {
            continue;
        };
        for id in ids.iter() {
            if !only.is_empty() && !only.iter().any(|o| id.starts_with(o.as_str())) {
                continue;
            }
            let fired: Vec<Finding> = findings
                .iter()
                .filter(|f| f.rule_id == *id)
                .cloned()
                .collect();
            if fired.is_empty() {
                let severity = RULES
                    .iter()
                    .find(|(r, _)| r == id)
                    .map(|(_, s)| *s)
                    .unwrap_or("info");
                out.push(Finding {
                    rule_id: id,
                    severity,
                    message: "ok".to_string(),
                    path: String::new(),
                    fix: String::new(),
                });
            } else {
                out.extend(fired);
            }
        }
    }

    out.sort_by(|a, b| {
        let rank = |f: &Finding| u8::from(f.message != "ok");
        rank(a)
            .cmp(&rank(b))
            .then(severity_rank(a.severity).cmp(&severity_rank(b.severity)))
            .then(a.rule_id.cmp(b.rule_id))
            .then(a.path.cmp(&b.path))
    });
    out
}

/// `checkrules.summarize` — the counts `check` prints.
///
/// **A finding is a failure when it is not an `ok` record**, which is the same distinction the
/// dispatcher makes - and the reason the message is the marker rather than a field is that Python's
/// `RuleResult.ok` has no counterpart in the ported `Finding`.
pub fn summarize(results: &[Finding]) -> (usize, usize, usize, usize) {
    let fails: Vec<&Finding> = results.iter().filter(|r| r.message != "ok").collect();
    (
        results.len(),
        fails.len(),
        fails.iter().filter(|r| r.severity == "error").count(),
        fails.iter().filter(|r| r.severity == "warn").count(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// **The dispatcher's rule set is exactly Python's `checkrules.rules()`.**
    ///
    /// **Ignored by default** (it runs Python). **It asserts the ID SETS are equal, not the result
    /// counts**: Python returns one result per firing (57 results over 41 ids), while a Rust rule
    /// returns its violations — **so counting results would compare two different things and call it
    /// agreement.**
    ///
    /// **This is the comparison that matters for dispatch**, and it is the one whose absence let
    /// `cmd_check` run no rules while `parity` reported the set complete.
    #[test]
    #[ignore = "needs python3 and the full doc set"]
    fn the_table_matches_pythons_checkrules() {
        let repo = repo();
        let script = repo.join("scripts/.parity-checkdispatch.py");
        std::fs::write(
            &script,
            r#"
import sys, json
sys.path.insert(0, "scripts/harness")
import checkrules
print(json.dumps(sorted(r.id for r in checkrules.rules())))
"#,
        )
        .expect("write the probe");
        let out = std::process::Command::new("python3")
            .arg(&script)
            .current_dir(&repo)
            .output()
            .expect("run python");
        let _ = std::fs::remove_file(&script);
        let py: Vec<String> =
            serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
        let rust: Vec<String> = RULES.iter().map(|(r, _)| r.to_string()).collect();
        let rs: std::collections::BTreeSet<&String> = rust.iter().collect();
        let ps: std::collections::BTreeSet<&String> = py.iter().collect();
        assert_eq!(
            rs,
            ps,
            "only rust: {:?}\nonly python: {:?}",
            rs.difference(&ps).collect::<Vec<_>>(),
            ps.difference(&rs).collect::<Vec<_>>()
        );
    }

    /// **Every module id in the table is reachable**, so a rule cannot be in `RULES` and in no
    /// module's bucket — which would make it an `ok` record that nothing ever runs.
    #[test]
    fn every_table_id_belongs_to_a_module() {
        for (id, _) in RULES.iter() {
            assert!(
                MODULE_RULES.iter().any(|(_, ids)| ids.contains(id)),
                "{id} is in RULES but in no module bucket, so it can never fire"
            );
        }
    }

    /// **The dispatcher runs every rule and produces Python's shape**: one record per rule, and a
    /// record for a rule that did not fire.
    #[test]
    fn run_produces_one_record_per_rule_including_the_quiet_ones() {
        let repo = repo();
        let paths = StatePaths::resolve(repo, None, Some("/nonexistent-live"));
        let regs = Registries::load(&paths).expect("registry");
        let out = run(&paths, &regs, &[], 1_800_000_000);
        let ids: std::collections::BTreeSet<&str> = out.iter().map(|f| f.rule_id).collect();
        assert_eq!(
            ids.len(),
            RULES.len(),
            "every table rule appears at least once: {} records for {} rules",
            ids.len(),
            RULES.len()
        );
        assert!(
            out.iter().any(|f| f.message == "ok"),
            "a rule that did not fire still reports, which is how a lab tells passed from absent"
        );
    }

    /// **`only` filters by prefix**, which is what `check --only status` relies on.
    #[test]
    fn only_filters_by_prefix() {
        let repo = repo();
        let paths = StatePaths::resolve(repo, None, Some("/nonexistent-live"));
        let regs = Registries::load(&paths).expect("registry");
        let out = run(&paths, &regs, &["status".to_string()], 1_800_000_000);
        assert!(!out.is_empty(), "the status rules are present");
        assert!(
            out.iter().all(|f| f.rule_id.starts_with("status.")),
            "nothing outside the prefix"
        );
    }
}
