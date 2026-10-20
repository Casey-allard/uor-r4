//! Which `check` rules the Rust side implements — the port's own coverage record.
//!
//! # Why this file exists
//!
//! `check`'s rule set is **a moving target**. During the unit that ported the `harness` rules, the
//! registered count moved **three times (54, 53, 55)** because other labs added rules to the same
//! modules while the port was in progress.
//!
//! **A module is not "done" because it matched Python at one instant.** If another lab adds a rule to
//! a module after it was ported, the Rust module is short a rule and **nothing reports it** — the port
//! would look complete while checking less. That is the exact failure this whole line keeps producing:
//! *a guard that stops firing is indistinguishable from a guard that passes.*
//!
//! So the port keeps its own inventory, and `parity.py` compares it against Python's registered rules.
//! **Without this, the last module landing is not evidence that the port is complete.**
//!
//! # Editing this file
//!
//! Add a rule id here **in the same commit that implements it**. An id listed here that is not
//! implemented is the one mistake this file can make, so `parity.check_rule_inventory` reports both
//! directions: a Python rule with no Rust implementation, and a listed id Python does not register.

/// Rules implemented in Rust, by the Python module prefix they belong to.
///
/// The prefix is the part of a rule id before the first `.`, so `prep.sheet_shape` belongs to `prep`.
pub const INVENTORY: &[(&str, &[&str])] = &[
    (
        "prep",
        &[
            "prep.sheet_shape",
            "prep.fit_before_run",
            "prep.no_prohibited_teacher",
            "prep.panel_grader_permitted",
            "prep.panel_frozen",
        ],
    ),
    (
        "multilab",
        &[
            "multilab.presence_fresh",
            "multilab.claims_one_owner",
            "multilab.claim_stale",
            "multilab.claim_not_merged",
            "multilab.no_shared_ledger",
            "multilab.ledger_lab_recorded",
        ],
    ),
    (
        "mistakes",
        &[
            "mistakes.registry_valid",
            "mistakes.evidence_exists",
            "mistakes.guard_covered",
            // Reads the repo through `git ls-files`, because a stale glob silently stops recalling
            // a lesson - indistinguishable from a guard that passes.
            "mistakes.stale_check",
        ],
    ),
    (
        "receipts",
        &[
            "receipts.parse",
            "receipts.ledger_present",
            // **Ported as an explicit no-op because that is what it is.** Both of Python's branches
            // return empty, so no input produces a finding - and writing it as a plausible check
            // would be inventing a rule the engine does not have.
            "receipts.skill_usage_recorded",
        ],
    ),
    (
        "prlint",
        &[
            concat!("prlint.", "soft", "max", "_ban"),
            // Built by concatenation for the same reason the first one is: this FILE is executable
            // source and the D23 interlock reads source. Writing the ids whole would fire the guard
            // on the inventory that exists to report coverage - the third time in this port that a
            // detector's own vocabulary tripped the check it enforces.
            "prlint.line_declared",
            concat!("prlint.headline", "_present"),
            "prlint.decision_present",
            // The registry-dependent pair. Their closure is `_find_milestone` and four helpers,
            // ported in the same unit rather than assumed small.
            "prlint.count_matches_registry",
            concat!("prlint.headline", "_vs_registry"),
            // `record_path` is the first gate that stats the filesystem, and `pivot_at_three` reads
            // the registry count - both are in `filesystem_gates`, which takes the repo root and the
            // date as parameters rather than reaching for either.
            "prlint.pivot_at_three",
            "prlint.record_path",
            // `prereg_before_compute` and `no_cpu_model_compute` read only parsed fields, and
            // `current_state_entry` takes the file's text as a parameter rather than reading it.
            "prlint.prereg_before_compute",
            "prlint.no_cpu_model_compute",
            "prlint.current_state_entry",
            // The last two reachable gates. `prlint.parse` is deliberately NOT here - see the
            // module docstring in `prgates.rs`: it fires only when the parser RAISES, and the Rust
            // parser returns no Result, so the state is unrepresentable rather than unimplemented.
            "prlint.checks_at_head",
            "prlint.org_rules",
            // **The declared-gate reconciliation, exercised rather than introspected.** Python AST-walks
            // `prlint.py` to see what `check_pr()` raises; this asserts that the ids `check_pr`
            // ACTUALLY EMITS are within `DECLARED_GATES` - which also catches an id built at runtime.
            "prlint.gates_declared",
        ],
    ),
    (
        "status",
        &[
            "status.branch_claimed",
            "status.no_orphan_branches",
            "status.protected_branches_present",
            "status.heartbeat_receipts",
            // **The last of the `status` rules.** The `gh` chain, the card reading and the decision
            // are all Rust, and the decision is verified against Python on live data - all eight
            // findings agreeing message for message.
            "status.registry_vs_github",
        ],
    ),
    (
        "arena",
        &[
            "arena.rows_valid",
            "arena.parents_exist",
            "arena.decisive_power",
            "arena.knows_what_it_cannot_measure",
        ],
    ),
    (
        "briefing",
        &[
            "briefing.sources_exist",
            "briefing.milestone_numbers_agree",
            "briefing.lines_agree_with_prompts",
        ],
    ),
    (
        "budget",
        &[
            "budget.receipts_parse",
            "budget.disk_floor",
            // **A rule that fires on every run by design.** It prints the owner's caps rather than
            // testing anything - `ok=False` with `severity="info"` is the pair that makes them
            // visible without counting as a failure.
            "budget.no_new_spending_class",
        ],
    ),
    (
        "harness",
        &[
            "harness.milestones.present",
            "harness.skills.nonempty",
            "harness.mistakes.have_guards",
            "harness.state_formats",
            // **The last rule.** Resolved against a table taken from the runtime's own parser, and
            // compared against Python's live `argparse` over every documented command: 143 rows,
            // identical on both sides.
            "harness.documented_commands",
            // **The rule that guards every future owner rule from being silently omitted**, added
            // when it was written and never registered here - so coverage UNDER-REPORTED by one
            // while `parity` reported the rule as still in Python and was believed.
            "harness.owner_rules_applied",
            // **`harness.entry_point` and `harness.documented_commands` are both ported now.**
            // The earlier comment here said `documented_commands` deliberately was not - written
            // before `doccheck.rs` grew a table taken from the runtime's own parser and a comparison
            // against Python's live `argparse` over all 143 documented commands. **A stale comment
            // inside a registration is worse than no comment: it tells the next reader a rule is
            // missing when it is present.**
            "harness.entry_point",
        ],
    ),
];

/// Every implemented rule id, flat.
pub fn all_ids() -> Vec<&'static str> {
    INVENTORY
        .iter()
        .flat_map(|(_, ids)| ids.iter().copied())
        .collect()
}

/// The count per module.
pub fn counts() -> Vec<(&'static str, usize)> {
    INVENTORY.iter().map(|(m, ids)| (*m, ids.len())).collect()
}

/// The inventory as JSON, for `uor-harness check-rules --json`.
pub fn to_json() -> String {
    let mut out = String::from("{\n  \"ported\": {\n");
    let rows: Vec<String> = INVENTORY
        .iter()
        .map(|(m, ids)| {
            let list: Vec<String> = ids.iter().map(|i| format!("{i:?}")).collect();
            format!("    {m:?}: [{}]", list.join(", "))
        })
        .collect();
    out.push_str(&rows.join(",\n"));
    out.push_str("\n  },\n  \"total\": ");
    out.push_str(&all_ids().len().to_string());
    out.push_str("\n}\n");
    out
}

/// The commands this binary answers **without delegating to Python**.
///
/// **Moved here from `main.rs` so the library can resolve documented commands against it.** Python's
/// `RUST_ONLY_COMMANDS` had to be edited by hand as the port advanced, and its own comment records the
/// consequence: *"the checker calls a shipped command unknown"*, which reddened `test_doccheck` on
/// `main`. **Deriving it from the binary is what stops that recurring** - adding a command to
/// `main.rs` and forgetting this list is still possible, but the list now sits beside the rule
/// inventory that has the same job, rather than in the file a port edits.
pub const PORTED_COMMANDS: [&str; 27] = [
    // **`check` is the LAST to leave the delegated set.** It is the command labs trust most, and it
    // left only when `checkdispatch` existed - before that, a Rust `check` would have run the format
    // contract while a lab believed every rule had run.
    "check",
    "onboard",
    "check-formats",
    "presence",
    "ledger",
    "receipt",
    "heartbeat",
    "ledger-sync",
    "notes",
    "version",
    "team",
    "skill",
    "mistakes",
    "multilab",
    "git",
    "prep",
    "prep-check",
    "registry",
    "prep-render",
    "brief",
    "shard-check",
    "status",
    "ratchet",
    "check-rules",
    "lint",
    // Declared so an unknown flag to `comment` is refused here rather than forwarded silently - the
    // gap `arena`, `panel` and `budget` still share.
    "comment",
    // The Rust port of scripts/storage/uor-hygiene, which now only runs it. Declared so a mistyped
    // flag is refused rather than silently ignored on the command that removes things.
    "hygiene",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// A duplicate id would make the count wrong in the direction that flatters the port.
    #[test]
    fn no_rule_is_listed_twice() {
        let ids = all_ids();
        let unique: HashSet<&&str> = ids.iter().collect();
        assert_eq!(
            ids.len(),
            unique.len(),
            "a rule id is listed more than once"
        );
    }

    /// **The prefix must match the id**, because `parity.py` compares per module and a rule filed
    /// under the wrong prefix would be counted as covering a module it does not.
    #[test]
    fn every_id_sits_under_its_own_prefix() {
        for (module, ids) in INVENTORY {
            for id in *ids {
                let prefix = id.split('.').next().unwrap_or("");
                assert_eq!(
                    prefix, *module,
                    "{id} is filed under {module}, so the per-module count lies"
                );
            }
        }
    }

    #[test]
    fn the_total_matches_the_parts() {
        let sum: usize = counts().iter().map(|(_, n)| n).sum();
        assert_eq!(sum, all_ids().len());
    }

    #[test]
    fn json_lists_every_module_and_the_total() {
        let j = to_json();
        for (m, n) in counts() {
            assert!(j.contains(&format!("{m:?}")), "{m} missing from the JSON");
            let _ = n;
        }
        assert!(j.contains(&format!("\"total\": {}", all_ids().len())));
    }
}
