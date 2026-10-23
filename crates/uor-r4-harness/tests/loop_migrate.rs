//! `loop migrate --check` integration tests: the notes-migration invariants (loop PR 8).
//! Each negative test removes or breaks exactly one invariant and must fail naming it.

use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
    fn git(&self, args: &[&str]) -> String {
        let o = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(o.status.success(), "git {args:?}: {o:?}");
        String::from_utf8(o.stdout).unwrap().trim().to_string()
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn commit(&self, msg: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", msg]);
        self.git(&["rev-parse", "HEAD"])
    }

    fn migrate(&self) -> Output {
        Command::new(BIN)
            .args([
                "--repo",
                self.root.to_str().unwrap(),
                "loop",
                "migrate",
                "--check",
            ])
            .env("UOR_LAB", "kimi")
            .output()
            .unwrap()
    }
}

const AGENTS: &str = "# AGENTS\n\n## Lessons the project paid for\n\n<!-- loop:lessons-top-ten:start -->\n<!-- loop:lessons-top-ten:end -->\n\nSee [the plan](PLAN.md).\n";
const PLAN: &str = "# PLAN\n\nThe goal.\n\n## Premises\n\n| Premise | Probe | Status |\n|---|---|---|\n| p | `cargo test -p x` | open |\n\n## Owner directions\n\n| Direction | Status | Date | Note |\n|---|---|---|---|\n| d | done | 2026-10-22 | n |\n\nSee [AGENTS.md](AGENTS.md) and [the registry](docs/harness/pm-rules.md).\n";
const STATE: &str = "<!-- rendered by uor-harness loop render from 0000000000000000000000000000000000000000; do not edit -->\n# STATE\n";
const HANDOFF: &str = "<!-- rendered by uor-harness loop render from 0000000000000000000000000000000000000000; do not edit -->\n# HANDOFF\n";
const PM_RULES: &str = "# PM rules\n\nOne list.\n";
const DECISIONS: &str = "# DECISIONS\n\n## Decisions after the charter (from D43)\n\n| Id | Date | Decision | Reason | Alternatives turned down | Cost to undo |\n|---|---|---|---|---|---|\n| D43 | 2026-10-20 | x | y | z | w |\n";
const LESSON: &str = "---\nid: L-001\ntitle: Records drifted\ndate: 2026-10-21\nseverity: block\nclass: process\ntrigger: [records drifted]\npaths: [docs/**]\nmilestones: [m1]\nmechanism:\nguard: \"none: the loop itself\"\nevidence: [docs/loop/README.md]\nstatus: active\n---\n\n## What happened\n\nx\n\n## Symptom\n\ny\n\n## Lesson\n\nz\n\n## How to apply\n\nw\n";

/// A complete migrated fixture: every invariant holds.
fn migrated_repo() -> Repo {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("repo");
    std::fs::create_dir_all(&root).unwrap();
    let r = Repo { _tmp: tmp, root };
    r.git(&["init", "-q", "-b", "main"]);
    r.write(
        "docs/harness/labs.json",
        r#"{"schema":1,"labs":{"kimi":{"vendor":"Moonshot","client":"Kimi","headless":"kimi -p","open_pr_limit":3}}}"#,
    );
    r.write(
        "STATUS.md",
        "# Status\n\nThe old hand-written page. [CHARTER.md](CHARTER.md)\n",
    );
    r.write(
        "docs/integration/NOW.md",
        "# NOW\n\nThe old current-work page.\n",
    );
    r.commit("live notes");
    r.write("CHARTER.md", "# Charter\n");
    r.write("AGENTS.md", AGENTS);
    r.write("PLAN.md", PLAN);
    r.write("STATE.md", STATE);
    r.write("HANDOFF.md", HANDOFF);
    r.write("docs/harness/pm-rules.md", PM_RULES);
    r.write("docs/integration/DECISIONS.md", DECISIONS);
    r.write("docs/harness/mistakes/L-001.md", LESSON);
    let status = std::fs::read_to_string(r.root.join("STATUS.md")).unwrap();
    let now = std::fs::read_to_string(r.root.join("docs/integration/NOW.md")).unwrap();
    r.write("archive/STATUS.md", &status);
    r.write("archive/integration/NOW-2026-10-22.md", &now);
    r.write(
        "STATUS.md",
        "# STATUS.md is retired\n\nMoved byte-identical to [archive/STATUS.md](archive/STATUS.md).\n",
    );
    r.write(
        "docs/integration/NOW.md",
        "# NOW.md is retired\n\nMoved byte-identical to [archive/integration/NOW-2026-10-22.md](../../../archive/integration/NOW-2026-10-22.md).\n",
    );
    r.write(
        "docs/start/FILEMAP.md",
        "# FILEMAP\n\n| Path | What it holds |\n|---|---|\n| `archive` | retired |\n| `archive/integration` | retired integration notes |\n| `docs` | docs |\n| `docs/harness` | harness docs |\n| `docs/harness/mistakes` | the mistakes store |\n| `docs/integration` | integration notes |\n| `docs/start` | start docs |\n",
    );
    r.commit("the migration");
    r
}

#[test]
fn complete_migrated_state_passes() {
    let r = migrated_repo();
    let o = r.migrate();
    assert!(o.status.success(), "complete fixture must pass: {o:?}");
}

#[test]
fn tampered_archive_copy_fails_byte_identity() {
    let r = migrated_repo();
    r.write("archive/STATUS.md", "# Status\n\nedited after the move\n");
    let o = r.migrate();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("byte-identical"),
        "name the byte-identity failure: {err}"
    );
}

#[test]
fn missing_lessons_markers_fail() {
    let r = migrated_repo();
    r.write(
        "AGENTS.md",
        "# AGENTS\n\nNo markers here. [the plan](PLAN.md)\n",
    );
    let o = r.migrate();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("lessons-top-ten"), "name the markers: {err}");
}

#[test]
fn unkeyed_lesson_fails() {
    let r = migrated_repo();
    r.write(
        "docs/harness/mistakes/old-kebab.md",
        &LESSON.replace("id: L-001", "id: old-kebab"),
    );
    let o = r.migrate();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("old-kebab.md"),
        "name the unkeyed lesson: {err}"
    );
}

#[test]
fn unmapped_directory_fails_filemap() {
    let r = migrated_repo();
    r.write("docs/unmapped/page.md", "# x\n");
    r.commit("an unmapped directory lands");
    let o = r.migrate();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("docs/unmapped"),
        "name the unmapped directory: {err}"
    );
}

#[test]
fn broken_link_in_a_migrated_view_fails() {
    let r = migrated_repo();
    r.write(
        "PLAN.md",
        &PLAN.replace("docs/harness/pm-rules.md", "docs/harness/pm-rules-GONE.md"),
    );
    let o = r.migrate();
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("broken link"), "name the broken link: {err}");
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn pointer_stub_not_naming_the_archive_copy_fails() {
    let r = migrated_repo();
    r.write("STATUS.md", "# STATUS.md is retired\n\nMoved away.\n");
    let o = r.migrate();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("STATUS.md: the pointer does not name archive/STATUS.md"),
        "name the pointer failure: {err}"
    );
}

#[test]
fn lesson_id_not_matching_the_file_stem_fails() {
    let r = migrated_repo();
    r.write(
        "docs/harness/mistakes/L-002.md",
        &LESSON.replace("id: L-001", "id: L-003"),
    );
    let o = r.migrate();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("docs/harness/mistakes/L-002.md: its `id:` is not `L-002`"),
        "name the id mismatch: {err}"
    );
}

#[test]
fn missing_l001_fails() {
    let r = migrated_repo();
    // Keep the mistakes store present (another keyed lesson) so only the L-001 rule fires.
    r.write(
        "docs/harness/mistakes/L-002.md",
        &LESSON.replace("id: L-001", "id: L-002"),
    );
    r.git(&["rm", "-q", "docs/harness/mistakes/L-001.md"]);
    let o = r.migrate();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("L-001.md (records drifted) is missing"),
        "name the missing L-001: {err}"
    );
}

#[test]
fn decisions_table_without_the_new_columns_fails() {
    let r = migrated_repo();
    r.write(
        "docs/integration/DECISIONS.md",
        "# DECISIONS\n\n## Decisions after the charter (from D43)\n\n| Id | Date | Decision | Reason |\n|---|---|---|---|\n| D43 | 2026-10-20 | x | y |\n",
    );
    let o = r.migrate();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("lacks the `Alternatives turned down` column")
            && err.contains("lacks the `Cost to undo` column"),
        "name both missing columns: {err}"
    );
}

#[test]
fn agents_over_the_80_line_budget_fails() {
    let r = migrated_repo();
    let padded = format!(
        "# AGENTS\n\n## Lessons the project paid for\n\n<!-- loop:lessons-top-ten:start -->\n<!-- loop:lessons-top-ten:end -->\n\nSee [the plan](PLAN.md).\n{}",
        "pad\n".repeat(80)
    );
    r.write("AGENTS.md", &padded);
    let o = r.migrate();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("over the 80-line gate"),
        "name the read-path budget: {err}"
    );
}

#[test]
fn filemap_row_naming_a_gone_path_fails() {
    let r = migrated_repo();
    let text = std::fs::read_to_string(r.root.join("docs/start/FILEMAP.md")).unwrap();
    r.write(
        "docs/start/FILEMAP.md",
        &format!("{text}| `docs/gone` | not there |\n"),
    );
    let o = r.migrate();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("FILEMAP: row `docs/gone` names a path that is gone"),
        "name the gone-path row: {err}"
    );
}

#[test]
fn a_missing_view_is_named_not_skipped() {
    let r = migrated_repo();
    r.git(&["rm", "-q", "PLAN.md"]);
    let o = r.migrate();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("PLAN.md: the migrated view is missing"),
        "name the missing view: {err}"
    );
}
