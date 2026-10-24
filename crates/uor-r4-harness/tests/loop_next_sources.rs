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
        self.next_fake(&empty_next_fake())
    }

    /// `loop next` with a scripted GitHub fake.
    fn next_fake(&self, fake: &Value) -> Output {
        let nf = self.tmp.path().join("next-fake.json");
        let cf = self.tmp.path().join("claim-fake.json");
        std::fs::write(&nf, fake.to_string()).unwrap();
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
fn an_open_owner_direction_is_offered_before_repairs() {
    let plan = "# PLAN\n\n## Owner directions\n\n| Direction | Status | Date | Note |\n|---|---|---|---|\n| D1 build the GPU collection record | open | 2026-10-22 | x |\n";
    let r = Repo::new(
        &[("PLAN.md", plan), ("docs/harness/labs.json", &labs(None))],
        &[],
    );
    let o = r.next();
    assert_eq!(
        o.status.code(),
        Some(0),
        "the open owner direction must be offered before repairs: {o:?}"
    );
    assert!(
        decision(&o).starts_with("GO owner-direction D1 build the GPU collection record"),
        "the open owner direction must be offered before repairs: {}",
        decision(&o)
    );
    assert!(
        stdout(&o).contains("owner-direction: PLAN owner direction"),
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
        out.contains("owner-direction: not built — PLAN.md is not on origin/main yet"),
        "{out}"
    );
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

#[test]
fn the_open_owner_direction_is_offered_before_an_own_repair() {
    let plan = "# PLAN\n\n## Owner directions\n\n| Direction | Status | Date | Note |\n|---|---|---|---|\n| D1 build the GPU collection record | open | 2026-10-22 | x |\n";
    let r = Repo::new(
        &[("PLAN.md", plan), ("docs/harness/labs.json", &labs(None))],
        &[],
    );
    let order_path = "docs/loop/tasks/infra-loop-harness/deepseek-99/ORDER.md";
    let order = "# ORDER\n\nLab: deepseek. Item: [it](../../../../roadmap/items/it.md) (#1).\n";
    let fake = json!({
        "prs": [{
            "number": 9,
            "headRefName": "deepseek/repair-me",
            "headRefOid": "aaaa",
            "isDraft": false,
            "mergeable": "MERGEABLE",
            "files": [{"path": order_path}],
            "checks": [{"name": "ci", "conclusion": "FAILURE"}]
        }],
        "comments": {},
        "files": {format!("9:{order_path}"): order},
        "labels": {},
        "failComments": [],
        "allFiles": {}
    });
    let o = r.next_fake(&fake);
    assert_eq!(
        o.status.code(),
        Some(0),
        "the open owner direction must be offered before repairs: {o:?}"
    );
    assert!(
        decision(&o).starts_with("GO owner-direction D1 build the GPU collection record"),
        "the open owner direction must be offered before repairs: {}",
        decision(&o)
    );
}

#[test]
fn a_note_cell_that_says_open_is_not_a_status() {
    let plan = "# PLAN\n\n## Owner directions\n\n| Direction | Status | Date | Note |\n|---|---|---|---|\n| D3 done thing | done | 2026-10-22 | open |\n";
    let r = Repo::new(
        &[("PLAN.md", plan), ("docs/harness/labs.json", &labs(None))],
        &[],
    );
    let o = r.next();
    assert!(!stdout(&o).contains("GO owner-direction"), "{}", stdout(&o));
    assert!(
        stdout(&o).contains("owner-direction: PLAN.md has no open owner direction"),
        "{}",
        stdout(&o)
    );
}

#[test]
fn a_malformed_owner_open_row_is_skipped_and_named() {
    let plan = "# PLAN\n\n## Owner directions\n\n| Direction | Status | Date | Note |\n|---|---|---|---|\n|  | open | 2026-10-22 | x |\n| D2 valid thing | open | 2026-10-22 | y |\n";
    let r = Repo::new(
        &[("PLAN.md", plan), ("docs/harness/labs.json", &labs(None))],
        &[],
    );
    let o = r.next();
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(
        decision(&o).starts_with("GO owner-direction D2 valid thing"),
        "{}",
        decision(&o)
    );
    assert!(
        decision(&o).contains("skipped malformed")
            && decision(&o).contains("Direction cell is empty"),
        "{}",
        decision(&o)
    );
}

#[test]
fn a_malformed_premise_open_row_is_named_not_hidden() {
    let plan = "# PLAN\n\n## Premises\n\n| Premise | Probe | Status | Cancel if |\n|---|---|---|---|\n| P1 no command | not-backticked | open | x |\n";
    let r = Repo::new(
        &[("PLAN.md", plan), ("docs/harness/labs.json", &labs(None))],
        &[],
    );
    let o = r.next();
    assert!(
        stdout(&o).contains("not built — the Premises table has malformed open row(s): table row 2: no probe command in backticks"),
        "{}",
        stdout(&o)
    );
    assert!(
        !stdout(&o).contains("premise-probes: PLAN premise"),
        "{}",
        stdout(&o)
    );
}

#[test]
fn a_missing_plan_table_is_named_not_guessed() {
    let r = Repo::new(
        &[
            ("PLAN.md", "# PLAN\n"),
            ("docs/harness/labs.json", &labs(None)),
        ],
        &[],
    );
    let o = r.next();
    assert!(
        stdout(&o)
            .contains("owner-direction: not built — PLAN.md has no `## Owner directions` table"),
        "{}",
        stdout(&o)
    );
    assert!(
        stdout(&o).contains("premise-probes: not built — PLAN.md has no `## Premises` table"),
        "{}",
        stdout(&o)
    );
}

#[test]
fn a_finished_gpu_collection_is_offered_and_a_collected_one_is_not() {
    use uor_r4_harness::loopdrive::{write_collection, GpuCollection};

    let r = Repo::new(&[("docs/harness/labs.json", &labs(None))], &[]);
    let record = GpuCollection {
        id: "deepseek-run-1".to_string(),
        lab: "deepseek".to_string(),
        renew: vec!["uor-pod".to_string()],
        started_unix: 1,
        ended_unix: Some(2),
        status: "finished".to_string(),
        collected: false,
        artifacts: Vec::new(),
        notes: Vec::new(),
    };
    write_collection(&r.root, "deepseek", &record).expect("write the collection record");
    let o = r.next();
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(
        stdout(&o).contains("gpu-collections: GPU collection deepseek-run-1 finished"),
        "a finished uncollected collection must be named: {}",
        stdout(&o)
    );
    assert!(
        decision(&o).starts_with("GO gpu-collection deepseek-run-1 — "),
        "the finished collection must be the target: {}",
        decision(&o)
    );

    let mut collected = record.clone();
    collected.collected = true;
    write_collection(&r.root, "deepseek", &collected).expect("mark collected");
    let o = r.next();
    assert!(
        !decision(&o).starts_with("GO gpu-collection"),
        "a collected record must not be offered: {}",
        decision(&o)
    );
    assert!(
        stdout(&o).contains("gpu-collections: no GPU collection record is waiting"),
        "{}",
        stdout(&o)
    );
}

#[test]
fn the_collect_command_marks_the_record_so_loop_next_stops_offering_it() {
    use uor_r4_harness::loopdrive::{read_collections, write_collection, GpuCollection};

    let r = Repo::new(&[("docs/harness/labs.json", &labs(None))], &[]);
    let record = GpuCollection {
        id: "deepseek-run-2".to_string(),
        lab: "deepseek".to_string(),
        renew: vec!["uor-pod".to_string()],
        started_unix: 1,
        ended_unix: Some(2),
        status: "finished".to_string(),
        collected: false,
        artifacts: Vec::new(),
        notes: Vec::new(),
    };
    write_collection(&r.root, "deepseek", &record).expect("write the collection record");
    let o = r.next();
    assert!(
        decision(&o).starts_with("GO gpu-collection deepseek-run-2 — "),
        "the finished collection must be offered: {}",
        decision(&o)
    );

    // The CLI path the supervisor uses: only --collect and --artifact, no --client or --model.
    let mut cmd = Command::new(BIN);
    clean(&mut cmd);
    cmd.arg("--repo")
        .arg(&r.root)
        .args([
            "loop",
            "drive",
            "--collect",
            "deepseek-run-2",
            "--artifact",
            "bundle/out.sha256",
        ])
        .env("UOR_LAB", "deepseek")
        .current_dir(&r.root);
    let out = cmd.output().expect("uor-harness runs");
    assert_eq!(
        out.status.code(),
        Some(0),
        "loop drive --collect must be reachable without --client/--model: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let after = read_collections(&r.root, "deepseek").expect("read after collect");
    let collected = after
        .iter()
        .find(|record| record.id == "deepseek-run-2")
        .expect("the record stays");
    assert!(collected.collected);
    assert_eq!(collected.artifacts, vec!["bundle/out.sha256".to_string()]);

    let o = r.next();
    assert!(
        !decision(&o).starts_with("GO gpu-collection"),
        "a collected record must not be offered: {}",
        decision(&o)
    );
    assert!(
        stdout(&o).contains("gpu-collections: no GPU collection record is waiting"),
        "{}",
        stdout(&o)
    );
}
