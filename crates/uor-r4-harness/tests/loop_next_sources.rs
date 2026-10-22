//! `loop next` source tests: the lane and premise-probe sources read PLAN.md and the registry
//! `lane` key plus item `lane:`/`rank:` keys, and name what is missing instead of guessing.

use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::{json, Value};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB")
        .env_remove("UOR_LOOP_GH_FAKE")
        .env_remove("UOR_LOOP_NEXT_GH_FAKE")
}

/// The `loop next` GitHub fake: no open PRs, so the first source that decides is a local one.
fn empty_next_fake() -> Value {
    json!({"prs": [], "comments": {}, "files": {}, "labels": {}, "failComments": [], "allFiles": {}})
}

struct Repo {
    tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
    /// A repository with the given files and item frontmatter on `origin/main`.
    fn new(files: &[(&str, &str)], items: &[(&str, &str)]) -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap().join("repo");
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { tmp, root };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "t@example.invalid"]);
        r.git(&["config", "user.name", "t"]);
        r.write(".gitignore", "/local/\n/.worktrees/\n");
        r.write("docs/roadmap/items/.keep", "");
        r.write(
            "docs/loop/README.md",
            "# Loop\n\n## Build plan\n\n| PR | What it builds |\n|---|---|\n| 1 | x |\n",
        );
        for (path, text) in files {
            r.write(path, text);
        }
        for (id, frontmatter) in items {
            r.write(&format!("docs/roadmap/items/{id}.md"), frontmatter);
        }
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "init"]);
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

    fn git(&self, args: &[&str]) {
        let st = clean(&mut Command::new("git"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .unwrap();
        assert!(st.status.success(), "git {args:?}: {st:?}");
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

    fn next(&self) -> Output {
        let nf = self.tmp.path().join("next-fake.json");
        let cf = self.tmp.path().join("claim-fake.json");
        std::fs::write(&nf, empty_next_fake().to_string()).unwrap();
        std::fs::write(&cf, "[]").unwrap();
        let mut cmd = Command::new(BIN);
        clean(&mut cmd);
        cmd.arg("--repo")
            .arg(&self.root)
            .args(["loop", "next"])
            .env("UOR_LAB", "deepseek")
            .env("UOR_LOOP_NEXT_GH_FAKE", &nf)
            .env("UOR_LOOP_GH_FAKE", &cf)
            .current_dir(&self.root);
        cmd.output().expect("uor-harness runs")
    }
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn decision(o: &Output) -> String {
    stdout(o).lines().last().unwrap_or("").to_string()
}

fn item(status: &str, extra: &str) -> String {
    format!("---\nid: x\nstatus: {status}\nscope: [docs/]\n{extra}---\n\n## Why\n")
}

fn labs(lane: Option<&str>) -> String {
    let lane = lane
        .map(|l| format!(",\"lane\":\"{l}\""))
        .unwrap_or_default();
    format!(
        "{{\"labs\":{{\"deepseek\":{{\"vendor\":\"DeepSeek\",\"client\":\"d\",\"headless\":\"dsh\",\"open_pr_limit\":3{lane}}}}}}}"
    )
}

#[test]
fn the_top_of_the_lab_lane_is_offered_before_any() {
    let r = Repo::new(
        &[("docs/harness/labs.json", &labs(Some("builder")))],
        &[
            ("lane-a", &item("ready", "lane: builder\nrank: 2\n")),
            ("lane-b", &item("ready", "lane: builder\nrank: 1\n")),
            ("other", &item("ready", "lane: infra\nrank: 0\n")),
        ],
    );
    let o = r.next();
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(
        decision(&o).starts_with("GO item lane-b — "),
        "the top of the lab lane must be offered before any: {}",
        decision(&o)
    );
    assert!(decision(&o).contains("builder lane"), "{}", decision(&o));
    assert!(
        stdout(&o).contains("lane: top of deepseek"),
        "{}",
        stdout(&o)
    );
}

#[test]
fn an_open_premise_probe_is_offered_from_plan() {
    let plan = "# PLAN\n\n## Premises\n\n| Premise | Probe | Status | Cancel if |\n|---|---|---|---|\n| P1 memory beats search | `cargo test -p x` | open | if y |\n| P2 already probed | `cargo test -p y` | probed | z |\n";
    let r = Repo::new(
        &[("PLAN.md", plan), ("docs/harness/labs.json", &labs(None))],
        &[],
    );
    let o = r.next();
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(
        stdout(&o).contains("premise-probes: PLAN premise"),
        "{}",
        stdout(&o)
    );
    assert!(
        decision(&o).contains("GO premise-probe cargo test -p x"),
        "the open premise probe must be the target: {}",
        decision(&o)
    );
}

#[test]
fn missing_plan_and_missing_lane_are_named_not_guessed() {
    let r = Repo::new(&[("docs/harness/labs.json", &labs(None))], &[]);
    let o = r.next();
    let out = stdout(&o);
    // An empty queue waits with a re-check time; only the kill switch stops (claude-31).
    assert_eq!(o.status.code(), Some(3), "{o:?}");
    assert!(
        out.contains("not built — PLAN.md is not on origin/main yet"),
        "{out}"
    );
    assert!(
        out.contains("no lane is assigned to deepseek in docs/harness/labs.json"),
        "{out}"
    );
    assert!(decision(&o).starts_with("WAIT empty queue"), "{o:?}");
}
