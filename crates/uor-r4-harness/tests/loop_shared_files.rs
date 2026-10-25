//! The nine shared registration files (Casey, 21-22 Oct): `loop pr-check`'s `scope` rule always
//! allows them, and `loop claim` refuses an item whose scope covers one. Each case runs against a
//! temporary git repository; every child process has every `GIT_*` variable removed, and each
//! repository is checked to sit inside its temporary directory.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB")
        .env_remove("UOR_LOOP_GH_FAKE")
        .env_remove("UOR_PRCHECK_GH_FAKE")
}

const LABS: &str =
    r#"{"labs":{"claude":{"vendor":"a","client":"b","headless":"c","open_pr_limit":2}}}"#;
const WATCH: &str = "[crates/uor-r4-harness/src/loopwatch*.rs]";
const REPORT: &str = "# REPORT\n\n## What did not work, first\n\nNothing.\n\n## What changed\n\nA.\n\n## Measurements\n\nNone.\n\n## Checks run at the head\n\n- tests\n\n## Skills used\n\nNone.\n\n## Still open\n\nNothing.\n\n## Plain summary\n\nDone.\n";

fn item(id: &str, scope: Option<&str>) -> String {
    let scope = scope.map(|s| format!("scope: {s}\n")).unwrap_or_default();
    format!("---\nid: {id}\nstatus: ready\n{scope}---\n\n## Why\n")
}

fn order(id: &str) -> String {
    format!("# ORDER\n\nLab: claude. Item: [{id}](../../../../roadmap/items/{id}.md) (#1).\n")
}

struct Repo {
    tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
    /// A repository whose `main` holds the lab registry, `items` and the shared files, pushed to a
    /// bare `origin` in the same temporary directory.
    fn new(items: &[(&str, Option<&str>)]) -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap().join("repo");
        assert!(root.starts_with(tmp.path().canonicalize().unwrap()));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { tmp, root };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "t@example.invalid"]);
        r.git(&["config", "user.name", "t"]);
        r.write(".gitignore", "/local/\n/.worktrees/\n");
        r.write("docs/harness/labs.json", LABS);
        for (id, scope) in items {
            r.write(&format!("docs/roadmap/items/{id}.md"), &item(id, *scope));
        }
        for f in uor_r4_harness::loopclaim::SHARED_FILES {
            r.write(f, "base\n");
        }
        r.write("crates/uor-r4-harness/src/looparrive.rs", "base\n");
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "base"]);
        let remote = r.tmp.path().canonicalize().unwrap().join("remote.git");
        let st = clean(&mut Command::new("git"))
            .args(["init", "-q", "--bare", "-b", "main"])
            .arg(&remote)
            .output()
            .unwrap();
        assert!(st.status.success(), "{st:?}");
        r.git(&["remote", "add", "origin", remote.to_str().unwrap()]);
        r.git(&["push", "-q", "origin", "main"]);
        r.git(&["fetch", "-q", "origin"]);
        r
    }

    fn cmd_git(&self, args: &[&str]) -> Output {
        clean(&mut Command::new("git"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs")
    }

    fn git(&self, args: &[&str]) {
        let st = self.cmd_git(args);
        assert!(st.status.success(), "git {args:?}: {st:?}");
    }

    fn rev(&self, r: &str) -> String {
        let st = self.cmd_git(&["rev-parse", r]);
        assert!(st.status.success(), "{st:?}");
        String::from_utf8_lossy(&st.stdout).trim().to_string()
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.root.join(rel);
        assert!(
            p.starts_with(&self.root),
            "write stays inside the temp repo"
        );
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    /// A PR for `id`: the ORDER alone in the first commit, then a REPORT and every path of
    /// `changed` rewritten. Returns the head sha; `main` is left where it was.
    fn pr(&self, id: &str, changed: &[&str]) -> String {
        let base = self.rev("main");
        let dir = format!("docs/loop/tasks/{id}/claude-1");
        self.git(&["checkout", "-q", "--detach", &base]);
        self.write(&format!("{dir}/ORDER.md"), &order(id));
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", "order"]);
        self.write(&format!("{dir}/REPORT.md"), REPORT);
        for p in changed {
            self.write(p, "changed\n");
        }
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", "work"]);
        let head = self.rev("HEAD");
        self.git(&["checkout", "-q", "main"]);
        head
    }

    fn pr_check(&self, head: &str) -> Output {
        let base = self.rev("main");
        clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop", "pr-check"])
            .args(["--base", &base, "--head", head, "--local"])
            .env("UOR_LAB", "claude")
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs")
    }

    fn claim(&self, id: &str) -> Output {
        self.claim_with(id, "[]")
    }

    /// `loop claim <id>` with the open PRs `prs` (the `UOR_LOOP_GH_FAKE` JSON).
    fn claim_with(&self, id: &str, prs: &str) -> Output {
        let fake = self.tmp.path().join("gh-fake.json");
        assert!(fake.starts_with(self.tmp.path()));
        std::fs::write(&fake, prs).unwrap();
        clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop", "claim", id])
            .env("UOR_LAB", "claude")
            .env("UOR_LOOP_GH_FAKE", &fake)
            .current_dir(Path::new(&self.root))
            .output()
            .expect("uor-harness runs")
    }
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// The `scope` rule's line in the pr-check output.
fn scope_line(o: &Output) -> String {
    let out = stdout(o);
    out.lines()
        .find(|l| {
            ["PASS scope", "FAIL scope", "WARN scope"]
                .iter()
                .any(|p| l.starts_with(p))
        })
        .unwrap_or_else(|| panic!("no scope line:\n{out}"))
        .to_string()
}

const PR_FILES: [&str; 4] = [
    "crates/uor-r4-harness/src/loopwatch.rs",
    "crates/uor-r4-harness/src/lib.rs",
    "crates/uor-r4-harness/src/main.rs",
    "docs/roadmap/README.md",
];

#[test]
fn shared_files_pass_scope_beside_the_items_own_files() {
    let r = Repo::new(&[("watch", Some(WATCH))]);
    let head = r.pr("watch", &PR_FILES);
    let o = r.pr_check(&head);
    assert!(
        scope_line(&o).starts_with("PASS scope"),
        "{}\n{o:?}",
        stdout(&o)
    );
}

#[test]
fn a_file_neither_shared_nor_in_scope_still_warns_scope() {
    let r = Repo::new(&[("watch", Some(WATCH))]);
    let mut files = PR_FILES.to_vec();
    files.push("crates/uor-r4-harness/src/looparrive.rs");
    let o = r.pr_check(&r.pr("watch", &files));
    let out = stdout(&o);
    assert!(scope_line(&o).starts_with("WARN scope"), "{out}");
    assert!(
        out.contains("crates/uor-r4-harness/src/looparrive.rs is outside item watch's scope"),
        "{out}"
    );
    assert!(!out.contains("lib.rs is outside"), "{out}");
    assert!(!out.contains("README.md is outside"), "{out}");
}

#[test]
fn an_item_without_scope_still_warns_scope() {
    let r = Repo::new(&[("noscope", None)]);
    let o = r.pr_check(&r.pr("noscope", &["crates/uor-r4-harness/src/lib.rs"]));
    let out = stdout(&o);
    assert!(scope_line(&o).starts_with("WARN scope"), "{out}");
    assert!(out.contains("has no scope: at the base"), "{out}");
}

fn refused(o: &Output, why: &str) {
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(err.contains("refused") && err.contains(why), "{err}");
}

#[test]
fn claim_refuses_a_scope_glob_covering_a_shared_file() {
    let r = Repo::new(&[("wide", Some("[crates/uor-r4-harness/**]"))]);
    refused(
        &r.claim("wide"),
        "scope glob `crates/uor-r4-harness/**` covers the shared file crates/uor-r4-harness/src/lib.rs",
    );
}

#[test]
fn claim_refuses_a_scope_listing_a_shared_file() {
    let r = Repo::new(&[("gates", Some("[crates/x/**, docs/harness/gates.json]"))]);
    refused(
        &r.claim("gates"),
        "scope glob `docs/harness/gates.json` covers the shared file docs/harness/gates.json",
    );
}

#[test]
fn claim_accepts_a_scope_clear_of_the_shared_files() {
    let r = Repo::new(&[("watch", Some(WATCH))]);
    let o = r.claim("watch");
    assert_eq!(o.status.code(), Some(0), "{o:?}");
}

/// An older item listing a shared file exactly does not make that file collide with a new
/// item: `docs/harness/*.toml` overlaps `docs/harness/gates.json` by prefix alone, and the
/// claim is not refused for overlap.
#[test]
fn an_open_orders_exact_shared_file_glob_never_counts_for_overlap() {
    let r = Repo::new(&[
        ("old", Some("[docs/harness/gates.json, crates/old/**]")),
        ("toml", Some("[docs/harness/*.toml]")),
    ]);
    let path = "docs/loop/tasks/old/codex-1/ORDER.md";
    let open = serde_json::json!([{
        "number": 1, "headRefName": "codex/old", "isDraft": true,
        "files": [{"path": path}],
        "orderTexts": {path: order("old")}
    }]);
    let o = r.claim_with("toml", &open.to_string());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(!err.contains("overlap"), "{err}");
    assert_eq!(o.status.code(), Some(0), "{o:?}");
}
