//! The `harness` rules, as Rust (D30 check-port step 2 of 10).
//!
//! # The seven `harness` rules
//!
//! | rule | severity | what it catches |
//! |---|---|---|
//! | `harness.milestones.present` | error | the milestone registry is missing m1–m4 |
//! | `harness.skills.nonempty` | error | no skill is registered |
//! | `harness.mistakes.have_guards` | warn | an active lesson with no guard, or a guard naming no rule |
//! | `harness.state_formats` | error | persisted state drifts from the frozen format contract |
//! | `harness.owner_rules_applied` | warn | an `all`-scoped owner rule is absent from `APPLICABLE_RULES` |
//! | `harness.entry_point` | error | normative docs invoke the Python entry point |
//! | `harness.documented_commands` | warn | a documented command does not exist in the parser |

use crate::mistakes;
use crate::registries::Registries;
use crate::skills;
use crate::state::StatePaths;

pub use crate::preprules::Finding;

/// `harness.milestones.present` — the registry exists and covers m1–m4.
///
/// **The harness cannot brief a session without the live contract**, so a missing milestone is an
/// error rather than a warning: every other command degrades silently against a hole in this table.
pub fn milestones_present(regs: &Registries) -> Vec<Finding> {
    let missing: Vec<&str> = ["m1", "m2", "m3", "m4"]
        .into_iter()
        .filter(|k| regs.milestone(k).is_none())
        .collect();
    if missing.is_empty() {
        return Vec::new();
    }
    vec![Finding {
        rule_id: "harness.milestones.present",
        severity: "error",
        message: format!("milestone registry is missing {}", missing.join(", ")),
        path: "docs/harness/milestones.json".into(),
        fix: "add the missing milestone entries".into(),
    }]
}

/// `harness.skills.nonempty` — at least one skill is registered.
///
/// **An empty skill registry means no reusable protocol is reachable**, and the failure is silent:
/// `skill use` would report an unknown id rather than an empty registry.
pub fn skills_nonempty(paths: &StatePaths) -> Vec<Finding> {
    if !skills::load(paths).is_empty() {
        return Vec::new();
    }
    vec![Finding {
        rule_id: "harness.skills.nonempty",
        severity: "error",
        message: "no skills found under docs/harness/skills/*/SKILL.md".into(),
        path: "docs/harness/skills".into(),
        fix: "add at least one skill directory with SKILL.md front matter".into(),
    }]
}

/// `harness.mistakes.have_guards` — every **active** lesson names a guard, or explains why it has none.
///
/// Two distinct failures, and keeping them apart is the point:
///
/// - **no guard at all** — the lesson is a note that will be forgotten again;
/// - **a guard that names no rule** — `guard: be careful` reads as a control and enforces nothing.
///
/// **`none: <reason>` is accepted on purpose.** Some lessons genuinely have no mechanical guard, and
/// forcing one would produce a fake rule — so the escape is explicit and has to be written down.
pub fn mistakes_have_guards(paths: &StatePaths) -> Vec<Finding> {
    let mut out = Vec::new();
    for m in mistakes::load(paths) {
        // A closed lesson is history: it already happened and the guard's absence is recorded.
        if m.status != "active" {
            continue;
        }
        if m.guard.trim().is_empty() {
            out.push(Finding {
                rule_id: "harness.mistakes.have_guards",
                severity: "warn",
                message: format!("{} has no guard", m.id),
                path: m.path.display().to_string(),
                fix: "add a guard: line naming a rule id, or 'none: <reason>'".into(),
            });
        } else if !m.guard.starts_with("none") && !m.guard.contains('.') {
            out.push(Finding {
                rule_id: "harness.mistakes.have_guards",
                severity: "warn",
                message: format!("{} guard {:?} does not name a rule id", m.id, m.guard),
                path: m.path.display().to_string(),
                fix: "use a dotted rule id such as prlint.record, or 'none: <reason>'".into(),
            });
        }
    }
    out
}

/// Every `harness` rule ported in this unit.
pub fn all(paths: &StatePaths, regs: &Registries) -> Vec<Finding> {
    let mut out = milestones_present(regs);
    out.extend(skills_nonempty(paths));
    out.extend(mistakes_have_guards(paths));
    // **`state_formats` was in the dispatch table and NOTHING EMITTED IT.** The rule existed as part
    // of the format contract, which `check` ran separately, so **the dispatcher reported it as an `ok`
    // record while it was never evaluated as a rule** - and a lab comparing the two implementations
    // saw Python report drift where Rust reported a pass. **Found by diffing the two outputs.**
    out.extend(state_formats(paths));
    out.extend(owner_rules_applied(regs));
    out.extend(entry_point(paths));
    out.extend(documented_commands(paths));
    out
}

/// `harness.owner_rules_applied` — every owner rule scoped to `all` (excluding `OWNER-*`
/// sub-clauses) must appear in [`crate::prep::APPLICABLE_RULES`].
pub fn owner_rules_applied(regs: &Registries) -> Vec<Finding> {
    let mut owed: Vec<&str> = regs
        .owner_rules
        .iter()
        .filter(|r| r.get("scope").and_then(|s| s.as_str()) == Some("all"))
        .filter_map(|r| r.get("id").and_then(|s| s.as_str()))
        .filter(|id| !id.is_empty() && !id.starts_with("OWNER-"))
        .collect();
    owed.sort_unstable();
    owed.dedup();
    let missing: Vec<&str> = owed
        .into_iter()
        .filter(|rid| !crate::prep::APPLICABLE_RULES.contains(rid))
        .collect();
    if missing.is_empty() {
        return Vec::new();
    }
    vec![Finding {
        rule_id: "harness.owner_rules_applied",
        severity: "warn",
        message: format!(
            "owner rule(s) scoped to everything are absent from a pre-registration sheet: {} - the sheet a lab follows does not state them",
            missing.join(", ")
        ),
        path: "crates/uor-r4-harness/src/prep.rs".into(),
        fix: "add them to APPLICABLE_RULES in BOTH scripts/harness/prep.py and crates/uor-r4-harness/src/prep.rs, and give each text in docs/harness/milestones.json's owner_rules".into(),
    }]
}

/// `harness.entry_point` — normative docs must invoke `uor-harness`, not `python3 scripts/harness/cli.py`.
///
/// [`crate::doccheck::entry_point`] returns a `warn` finding with `fix: ""` when clean to match
/// Python's per-rule output shape; `checkdispatch::run_scoped` synthesizes `ok` when no finding is
/// returned and treats any emitted finding with `message != "ok"` as a failure, so this dispatcher
/// adapter forwards only the `error` findings.
pub fn entry_point(paths: &StatePaths) -> Vec<Finding> {
    crate::doccheck::entry_point(&paths.repo)
        .into_iter()
        .filter(|f| f.severity == "error")
        .collect()
}

/// `harness.documented_commands` — every `uor-harness <subcommand> [flags]` named in normative docs
/// resolves against [`crate::doccheck`]'s static `ACCEPTED`, `GROUPS`, and `FLAGS` tables.
///
/// [`crate::doccheck::documented_commands_rule`] returns a `warn` finding with `fix: ""` when all
/// documented commands resolve; `checkdispatch::run_scoped` synthesizes `ok` when no finding is
/// returned, so this dispatcher adapter forwards only findings with a non-empty `fix`.
pub fn documented_commands(paths: &StatePaths) -> Vec<Finding> {
    crate::doccheck::documented_commands_rule(&paths.repo, &[])
        .into_iter()
        .filter(|f| !f.fix.is_empty())
        .collect()
}

/// `harness.state_formats` — the persisted state against the frozen contract.
///
/// **This is the rule the whole `check` command is named for**, and it was the one missing from the
/// dispatch path.
///
/// **Python's own rule shells out to the Rust binary** (`stateformats.check_contract` runs
/// `uor-harness check-formats --json`) and reads `PROBLEM` lines out of stdout. **The Rust rule calls
/// the checker directly**, which removes the subprocess and the string parsing between them — **so
/// the two are the same check without the round trip, and this one cannot disagree with itself.**
///
/// **The three outcomes are kept distinct**, which is the distinction the Python comment argues for:
/// * the contract is clean → no finding;
/// * the contract reports drift → `error`, naming the first three problems;
/// * the contract cannot be read → `error`, because **a check that cannot run is not a check that
///   passed.**
pub fn state_formats(paths: &StatePaths) -> Vec<Finding> {
    use crate::contract::Contract;
    let contract = match Contract::load(&paths.repo) {
        Ok(c) => c,
        Err(e) => {
            return vec![Finding {
                rule_id: "harness.state_formats",
                severity: "error",
                message: format!("the frozen contract could not be read: {e}"),
                path: "docs/harness/state-formats.json".into(),
                fix: "restore docs/harness/state-formats.json from main".into(),
            }]
        }
    };
    let report = contract.check(&paths.repo);
    if report.ok() {
        return Vec::new();
    }
    let mut problems: Vec<String> = Vec::new();
    for f in report.formats.iter() {
        for v in f.violations.iter() {
            problems.push(format!("{}: {v}", f.format));
            if problems.len() == 3 {
                break;
            }
        }
        if problems.len() == 3 {
            break;
        }
    }
    let detail = if problems.is_empty() {
        "state format drift".to_string()
    } else {
        problems.join("; ")
    };
    vec![Finding {
        rule_id: "harness.state_formats",
        severity: "error",
        message: detail,
        path: "docs/harness/state-formats.json".into(),
        fix: "make the file match the contract, or re-freeze it deliberately and commit both (see docs/harness/STATE-FORMATS.md)".into(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StatePaths;
    use std::path::{Path, PathBuf};

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn paths_of(root: &Path) -> StatePaths {
        StatePaths::resolve(root, None, Some("/nonexistent-live"))
    }

    fn regs_of(p: &StatePaths) -> Registries {
        Registries::load(p).expect("registry")
    }

    /// All seven ported `harness.*` rules must pass on the real repository.
    #[test]
    fn the_real_repo_passes_the_ported_rules() {
        let r = repo();
        let p = paths_of(&r);
        let f = all(&p, &regs_of(&p));
        assert!(f.is_empty(), "the real repo fails a harness rule: {f:?}");
    }

    #[test]
    fn a_registry_missing_a_milestone_is_an_error_naming_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("docs/harness")).expect("mkdir");
        std::fs::write(
            root.join("docs/harness/milestones.json"),
            r#"{"milestones":{"m1":{"id":"M1","issue":2029}}}"#,
        )
        .expect("write");
        let p = paths_of(root);
        let regs = Registries::load(&p).expect("registry");
        let f = milestones_present(&regs);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("m2"), "{}", f[0].message);
        assert!(f[0].message.contains("m4"), "{}", f[0].message);
        assert!(
            !f[0].message.contains("m1,"),
            "m1 is present and must not be named: {}",
            f[0].message
        );
    }

    #[test]
    fn a_complete_registry_passes() {
        let r = repo();
        let p = paths_of(&r);
        assert!(
            milestones_present(&regs_of(&p)).is_empty(),
            "m1-m4 must all be registered"
        );
    }

    /// An empty registry is an error, because the failure is otherwise invisible.
    #[test]
    fn an_empty_skill_registry_is_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = paths_of(dir.path());
        let f = skills_nonempty(&p);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, "error");
        assert!(f[0].message.contains("no skills"), "{}", f[0].message);
    }

    #[test]
    fn the_real_skill_registry_is_not_empty() {
        let p = paths_of(&repo());
        assert!(skills_nonempty(&p).is_empty());
        assert!(
            !skills::load(&p).is_empty(),
            "the real repo registers skills"
        );
    }

    /// **The two failures are different and must stay distinguishable.** A guard naming no rule reads
    /// as a control and enforces nothing, which is not the same as having no guard.
    #[test]
    fn a_guard_that_names_no_rule_is_reported_differently_from_no_guard() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let d = root.join("docs/harness/mistakes");
        std::fs::create_dir_all(&d).expect("mkdir");
        std::fs::write(
            d.join("no-guard.md"),
            "---\nid: no-guard\ntitle: t\nseverity: block\nstatus: active\n---\n\nlesson\n",
        )
        .expect("write");
        std::fs::write(
            d.join("vague.md"),
            "---\nid: vague\ntitle: t\nseverity: block\nstatus: active\nguard: be careful\n---\n\nlesson\n",
        )
        .expect("write");
        let f = mistakes_have_guards(&paths_of(root));
        assert_eq!(f.len(), 2, "{f:?}");
        let by_id: std::collections::HashMap<&str, &str> = f
            .iter()
            .map(|x| (x.message.as_str(), x.fix.as_str()))
            .collect();
        assert!(
            by_id.keys().any(|k| k.contains("no-guard has no guard")),
            "the no-guard case must say so: {by_id:?}"
        );
        assert!(
            by_id.keys().any(|k| k.contains("does not name a rule id")),
            "the vague-guard case must say so: {by_id:?}"
        );
    }

    /// `none: <reason>` is an explicit escape, not a violation - forcing a guard would produce a fake
    /// rule, which is worse than a recorded absence.
    #[test]
    fn none_with_a_reason_is_accepted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let d = root.join("docs/harness/mistakes");
        std::fs::create_dir_all(&d).expect("mkdir");
        std::fs::write(
            d.join("n.md"),
            "---\nid: n\ntitle: t\nseverity: block\nstatus: active\nguard: none: no mechanical check exists\n---\n\nlesson\n",
        )
        .expect("write");
        assert!(mistakes_have_guards(&paths_of(root)).is_empty());
    }

    /// A closed lesson is history: its guard's absence is already recorded.
    #[test]
    fn a_closed_lesson_is_not_reported() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let d = root.join("docs/harness/mistakes");
        std::fs::create_dir_all(&d).expect("mkdir");
        std::fs::write(
            d.join("c.md"),
            "---\nid: c\ntitle: t\nseverity: block\nstatus: closed\n---\n\nlesson\n",
        )
        .expect("write");
        assert!(mistakes_have_guards(&paths_of(root)).is_empty());
    }

    /// The real registry's active lessons all carry guards that name a rule or an explicit `none`.
    #[test]
    fn the_real_lessons_all_carry_a_guard() {
        let p = paths_of(&repo());
        let f = mistakes_have_guards(&p);
        assert!(
            f.is_empty(),
            "a real active lesson has no usable guard: {f:?}"
        );
    }

    /// An `all`-scoped owner rule absent from `APPLICABLE_RULES` is reported, while `m1`-scoped and
    /// `OWNER-*` sub-clause entries are exempt.
    #[test]
    fn an_unapplied_all_scoped_owner_rule_is_reported() {
        let regs = Registries {
            owner_rules: vec![
                serde_json::json!({"id": "D21", "scope": "all", "rule": "ok"}),
                serde_json::json!({"id": "D99", "scope": "all", "rule": "missing from sheet"}),
                serde_json::json!({"id": "D98", "scope": "m1", "rule": "milestone-scoped"}),
                serde_json::json!({"id": "OWNER-2026-10-12-X", "scope": "all", "rule": "sub-clause"}),
            ],
            ..Default::default()
        };
        let f = owner_rules_applied(&regs);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule_id, "harness.owner_rules_applied");
        assert_eq!(f[0].severity, "warn");
        assert!(f[0].message.contains("D99"), "{}", f[0].message);
        assert!(!f[0].message.contains("D98"), "{}", f[0].message);
        assert!(!f[0].message.contains("OWNER-"), "{}", f[0].message);
    }

    /// `entry_point` emits an `error` finding when a normative doc invokes `python3 scripts/harness/cli.py`
    /// and is silent when clean.
    #[test]
    fn entry_point_reports_stale_cli_and_stays_silent_when_clean() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::write(root.join("README.md"), "Run `uor-harness status`.\n").expect("write clean");
        assert!(entry_point(&paths_of(root)).is_empty());

        std::fs::write(
            root.join("README.md"),
            "Run `python3 scripts/harness/cli.py status`.\n",
        )
        .expect("write stale");
        let f = entry_point(&paths_of(root));
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule_id, "harness.entry_point");
        assert_eq!(f[0].severity, "error");
        assert!(f[0].message.contains("README.md:1"), "{}", f[0].message);
    }

    /// `documented_commands` emits a `warn` finding when a normative doc names an unknown subcommand
    /// or flag and is silent when all documented commands exist.
    #[test]
    fn documented_commands_reports_unknown_command_and_stays_silent_when_clean() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::write(
            root.join("README.md"),
            "Run `uor-harness status --refresh`.\n",
        )
        .expect("write clean");
        assert!(documented_commands(&paths_of(root)).is_empty());

        std::fs::write(
            root.join("README.md"),
            "Run `uor-harness nonexistent-subcommand` and `uor-harness status --refreshh`.\n",
        )
        .expect("write unknown");
        let f = documented_commands(&paths_of(root));
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule_id, "harness.documented_commands");
        assert_eq!(f[0].severity, "warn");
        assert!(
            f[0].message.contains("nonexistent-subcommand"),
            "{}",
            f[0].message
        );
        assert!(f[0].message.contains("--refreshh"), "{}", f[0].message);
    }

    /// `harnessrules::all` and `checkdispatch::run_scoped` must actually dispatch
    /// `owner_rules_applied`, `entry_point`, and `documented_commands` so removing any `out.extend`
    /// call in `all` fails the suite.
    #[test]
    fn all_and_checkdispatch_surface_owner_rules_entry_point_and_documented_commands() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::write(
            root.join("README.md"),
            "Run `python3 scripts/harness/cli.py status`, `uor-harness nonexistent-subcommand`, and `uor-harness status --refreshh`.\n",
        )
        .expect("write violating doc");
        let p = paths_of(root);
        let regs = Registries {
            owner_rules: vec![serde_json::json!({
                "id": "D99",
                "scope": "all",
                "rule": "missing from APPLICABLE_RULES"
            })],
            ..Default::default()
        };

        let direct = all(&p, &regs);
        for expected in [
            "harness.owner_rules_applied",
            "harness.entry_point",
            "harness.documented_commands",
        ] {
            assert!(
                direct.iter().any(|f| f.rule_id == expected),
                "harnessrules::all must emit {expected}: {direct:?}"
            );
        }

        let dispatched = crate::checkdispatch::run_scoped(
            &p,
            &regs,
            &["harness.".to_string()],
            crate::checkdispatch::Scope::Repo,
            0,
        );
        for (expected_id, expected_substr) in [
            ("harness.owner_rules_applied", "D99"),
            ("harness.entry_point", "README.md:1"),
            ("harness.documented_commands", "--refreshh"),
        ] {
            let hit = dispatched
                .iter()
                .find(|f| f.rule_id == expected_id)
                .unwrap_or_else(|| panic!("missing {expected_id} in {dispatched:?}"));
            assert!(
                hit.message.contains(expected_substr),
                "{expected_id} should contain {expected_substr:?}, got: {}",
                hit.message
            );
        }
    }
}
