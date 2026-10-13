//! The `harness` rules, as Rust (D30 check-port step 2 of 10).
//!
//! # The six `harness` rules
//!
//! | rule | severity | what it catches |
//! |---|---|---|
//! | `harness.milestones.present` | error | the milestone registry is missing m1–m4 |
//! | `harness.skills.nonempty` | error | no skill is registered |
//! | `harness.mistakes.have_guards` | warn | an active lesson with no guard, or a guard naming no rule |
//! | `harness.state_formats` | error | persisted state drifts from the frozen format contract |
//! | `harness.entry_point` | error | normative docs invoke the Python entry point |
//! | `harness.documented_commands` | warn | a documented command does not exist in the parser |
//!
//! **The last three are text analysis over docs and source**, and two of them read the Python parser
//! itself. They are not in this unit: `harness.entry_point` and `harness.documented_commands` are only
//! meaningful while both implementations exist, and porting them to Rust would mean a Rust program
//! that reads a Python argument parser to check documentation — **a check that dies with the thing it
//! checks.** They belong with the retirement of `cli.py`, not before it.

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
    out
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

    /// The three ported rules must all pass on the real repository.
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
}
