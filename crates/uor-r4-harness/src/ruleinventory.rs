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
        &["mistakes.registry_valid", "mistakes.evidence_exists"],
    ),
    (
        "status",
        &["status.branch_claimed", "status.no_orphan_branches"],
    ),
    (
        "harness",
        &[
            "harness.milestones.present",
            "harness.skills.nonempty",
            "harness.mistakes.have_guards",
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
