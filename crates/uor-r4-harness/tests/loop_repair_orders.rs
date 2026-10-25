//! A repair chain (loop step 11) keeps one item, lab and setup row: `loop pr-check`'s `order`
//! rule compares every ORDER of the chain with the active one. Each case runs against a
//! temporary git repository; every child process has every `GIT_*` variable removed, and the
//! repository is checked to sit inside its temporary directory.

use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const LABS: &str = r#"{"labs":{"claude":{"vendor":"a","client":"b","headless":"c","open_pr_limit":2},"codex":{"vendor":"d","client":"b","headless":"c","open_pr_limit":2}}}"#;
const ORDER_1: &str = "docs/loop/tasks/item-a/claude-1/ORDER.md";
const ORDER_2: &str = "docs/loop/tasks/item-a/claude-2/ORDER.md";

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

fn order(lab: &str, id: &str, extra: &str) -> String {
    format!("# ORDER\n\nLab: {lab}. Item: [{id}](../../../../roadmap/items/{id}.md) (#1).\n{extra}")
}

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    base: String,
}

impl Repo {
    fn new() -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap().join("repo");
        assert!(root.starts_with(tmp.path().canonicalize().unwrap()));
        std::fs::create_dir_all(&root).unwrap();
        let mut r = Repo {
            _tmp: tmp,
            root,
            base: String::new(),
        };
        r.git(&["init", "-q", "-b", "main"]);
        r.write("docs/harness/labs.json", LABS);
        for id in ["item-a", "item-b"] {
            r.write(
                &format!("docs/roadmap/items/{id}.md"),
                &format!("---\nid: {id}\nstatus: ready\nscope: [crates/a/**]\n---\n"),
            );
        }
        r.write("crates/a/src/lib.rs", "pub fn a() {}\n");
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "base"]);
        r.base = r.out(&["rev-parse", "HEAD"]);
        r
    }

    fn cmd_git(&self, args: &[&str]) -> Output {
        clean(&mut Command::new("git"))
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs")
    }

    fn git(&self, args: &[&str]) {
        let st = self.cmd_git(args);
        assert!(st.status.success(), "git {args:?}: {st:?}");
    }

    fn out(&self, args: &[&str]) -> String {
        let st = self.cmd_git(args);
        assert!(st.status.success(), "git {args:?}: {st:?}");
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

    /// A repair PR: `first` as `claude-1` alone, work, then `second` as `claude-2` alone.
    fn repair(&self, first: &str, second: &str) -> String {
        self.git(&["checkout", "-q", "--detach", &self.base]);
        for (path, text) in [
            (ORDER_1, first),
            ("crates/a/src/lib.rs", "//\n"),
            (ORDER_2, second),
        ] {
            self.write(path, text);
            self.git(&["add", "-A"]);
            self.git(&["commit", "-q", "-m", "step"]);
        }
        self.out(&["rev-parse", "HEAD"])
    }

    /// The `order` rule's line and the whole output of `loop pr-check --local` at `head`.
    fn order_rule(&self, head: &str) -> (String, String) {
        let o = clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop", "pr-check"])
            .args(["--base", &self.base, "--head", head, "--local"])
            .env("UOR_LAB", "claude")
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs");
        let out = String::from_utf8_lossy(&o.stdout).into_owned();
        let line = out
            .lines()
            .find(|l| l.starts_with("PASS order ") || l.starts_with("FAIL order "))
            .unwrap_or_else(|| panic!("no order line:\n{out}"))
            .to_string();
        (line, out)
    }
}

#[test]
fn a_clean_chain_passes() {
    let r = Repo::new();
    let row = "Setup row: 7a.\n";
    let h = r.repair(
        &order("claude", "item-a", row),
        &order("claude", "item-a", row),
    );
    let (line, out) = r.order_rule(&h);
    assert!(line.starts_with("PASS order"), "{out}");
}

#[test]
fn a_repair_order_naming_another_item_fails() {
    let r = Repo::new();
    let h = r.repair(
        &order("claude", "item-a", ""),
        &order("claude", "item-b", ""),
    );
    let (line, out) = r.order_rule(&h);
    assert!(line.starts_with("FAIL order"), "{out}");
    assert!(
        out.contains(&format!(
            "{ORDER_1} has `Item: item-a` but the active ORDER {ORDER_2} has `Item: item-b`"
        )),
        "{out}"
    );
}

#[test]
fn an_earlier_order_naming_another_lab_fails() {
    let r = Repo::new();
    let h = r.repair(
        &order("codex", "item-a", ""),
        &order("claude", "item-a", ""),
    );
    let (line, out) = r.order_rule(&h);
    assert!(line.starts_with("FAIL order"), "{out}");
    assert!(
        out.contains(&format!(
            "{ORDER_1} says `Lab: codex` but its folder is lab claude"
        )),
        "{out}"
    );
}

#[test]
fn a_repair_order_dropping_the_setup_row_fails() {
    let r = Repo::new();
    let h = r.repair(
        &order("claude", "item-a", "Setup row: 7a.\n"),
        &order("claude", "item-a", ""),
    );
    let (line, out) = r.order_rule(&h);
    assert!(line.starts_with("FAIL order"), "{out}");
    assert!(
        out.contains(&format!(
            "{ORDER_1} has `Setup row: 7a` but the active ORDER {ORDER_2} has `Setup row: (none)`"
        )),
        "{out}"
    );
}

/// The repair-chain cases of `loop pr-check`'s `commit-order`, `order`, `report`, `scope` and
/// `review-record` rules. The fixtures are copied from `loop_prcheck.rs` (a test file cannot
/// import another); the same environment stripping and temp-dir check apply.
mod chain {
    use std::path::PathBuf;
    use std::process::{Command, Output};

    const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");

    /// One change applied to the working tree before a head is committed.
    type Edit = Box<dyn FnOnce(&Repo)>;
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

    const ITEM_A: &str = "---\nid: item-a\nstatus: ready\nscope: [crates/a/**, docs/a.md, .github/**, CHARTER.md]\nnotebook: docs/research/notebook/a.md\n---\n\n## Why\n\nBecause.\n";
    const NOTEBOOK: &str = "# A notebook\n\n| row | result |\n|---|---|\n| 1 | first |\n";
    const FILEMAP: &str = "# FILEMAP\n\n| path | what |\n|---|---|\n| crates/a | the a crate |\n| `crates/` | crates |\n| `crates/a` | a |\n| `docs` | docs |\n| `docs/harness` | h |\n| `docs/loop` | loop |\n| `docs/research` | r |\n| `docs/roadmap` | rm |\n| `docs/start` | s |\n";
    const OWNER_PATHS: &str = r#"{"schema": 1, "paths": [
  {"path": "CHARTER.md", "why": "rules"},
  {"path": ".github/**", "why": "CI"},
  {"path": "docs/harness/owner-paths.json", "why": "this list"}
]}"#;
    /// A valid REPORT, headings worded as in a real one.
    const REPORT: &str = "# REPORT\n\n## What did not work, first\n\nNothing.\n\n## What changed\n\nA.\n\n## Measurements\n\nNone.\n\n## Checks run at the head\n\n- tests\n\n## Skills used\n\nNone.\n\n## Still open\n\nNothing.\n\n## Plain summary\n\nDone.\n";
    const ORDER_A: &str = "docs/loop/tasks/item-a/claude-1/ORDER.md";
    struct Repo {
        _tmp: tempfile::TempDir,
        root: PathBuf,
        base: String,
    }

    /// A lab registry naming `names`, with the fields the binary's own lab check needs; each lab is
    /// its own vendor.
    fn labs(names: &[&str]) -> String {
        let labs: serde_json::Map<String, serde_json::Value> = names
        .iter()
        .map(|n| {
            let v = serde_json::json!({"vendor": n, "client": "b", "headless": "c", "open_pr_limit": 2});
            (n.to_string(), v)
        })
        .collect();
        serde_json::json!({ "labs": labs }).to_string()
    }

    fn order(lab: &str, id: &str, extra: &str) -> String {
        format!(
            "# ORDER\n\nLab: {lab}. Item: [{id}](../../../../roadmap/items/{id}.md) (#1).\n{extra}"
        )
    }
    impl Repo {
        fn new() -> Repo {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path().canonicalize().unwrap().join("repo");
            assert!(root.starts_with(tmp.path().canonicalize().unwrap()));
            std::fs::create_dir_all(&root).unwrap();
            let mut r = Repo {
                _tmp: tmp,
                root,
                base: String::new(),
            };
            r.git(&["init", "-q", "-b", "main"]);
            r.write("docs/harness/labs.json", &labs(&["claude", "codex"]));
            r.write("docs/roadmap/items/item-a.md", ITEM_A);
            r.write(
                "docs/roadmap/items/item-noscope.md",
                "---\nid: item-noscope\nstatus: ready\n---\n\n## Why\n",
            );
            r.write(
                "docs/roadmap/items/infra-loop-pr9.md",
                "---\nid: infra-loop-pr9\nstatus: ready\nscope: [crates/a/**]\n---\n",
            );
            r.write(
                "docs/roadmap/items/infra-loop-pr99-x.md",
                "---\nid: infra-loop-pr99-x\nstatus: ready\nscope: [crates/a/**]\n---\n",
            );
            r.write("docs/research/notebook/a.md", NOTEBOOK);
            r.write("docs/start/FILEMAP.md", FILEMAP);
            r.write("crates/a/src/lib.rs", "pub fn a() {}\n");
            r.write("docs/a.md", "# a\n");
            r.write("docs/harness/owner-paths.json", OWNER_PATHS);
            r.write(
                "docs/loop/README.md",
                "# Loop\n\n## Build plan\n\n| PR | What it builds |\n|---|---|\n| 99 | x |\n",
            );
            r.write("CHARTER.md", "# Charter\n");
            r.git(&["add", "-A"]);
            r.git(&["commit", "-q", "-m", "base"]);
            r.base = r.out(&["rev-parse", "HEAD"]);
            r
        }

        fn cmd_git(&self, args: &[&str]) -> Output {
            clean(&mut Command::new("git"))
                .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
                .args(args)
                .current_dir(&self.root)
                .output()
                .expect("git runs")
        }

        fn git(&self, args: &[&str]) {
            let st = self.cmd_git(args);
            assert!(st.status.success(), "git {args:?}: {st:?}");
        }

        fn out(&self, args: &[&str]) -> String {
            let st = self.cmd_git(args);
            assert!(st.status.success(), "git {args:?}: {st:?}");
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

        /// The ORDER for `item-a` by claude at `docs/loop/tasks/item-a/claude-1/ORDER.md`.
        fn order_a(&self) {
            self.write(
                "docs/loop/tasks/item-a/claude-1/ORDER.md",
                &order("claude", "item-a", ""),
            );
        }

        /// A head of one commit per step, built from the base; returns its sha.
        fn commits(&self, steps: Vec<Edit>) -> String {
            self.git(&["checkout", "-q", "--detach", &self.base]);
            for step in steps {
                step(self);
                self.git(&["add", "-A"]);
                self.git(&["commit", "-q", "--allow-empty", "-m", "step"]);
            }
            self.out(&["rev-parse", "HEAD"])
        }

        fn check(&self, head: &str, extra: &[&str]) -> Output {
            self.check_against(&self.base, head, extra)
        }

        /// The check with `extra`; `--local` is added unless `extra` names a GitHub mode.
        fn check_against(&self, base: &str, head: &str, extra: &[&str]) -> Output {
            let mode = ["--pr", "--local", "--ci"];
            let mut args = extra.to_vec();
            if !extra.iter().any(|a| mode.contains(a)) {
                args.push("--local");
            }
            self.check_raw(base, head, &args, None)
        }

        /// The check with exactly `extra`, and `UOR_PRCHECK_GH_FAKE` holding `fake` when given.
        fn check_raw(&self, base: &str, head: &str, extra: &[&str], fake: Option<&str>) -> Output {
            let mut cmd = Command::new(BIN);
            clean(&mut cmd);
            if let Some(json) = fake {
                let p = self._tmp.path().join("gh-fake.json");
                std::fs::write(&p, json).unwrap();
                cmd.env("UOR_PRCHECK_GH_FAKE", &p);
            }
            cmd.args(["--repo", self.root.to_str().unwrap(), "loop", "pr-check"])
                .args(["--base", base, "--head", head])
                .args(extra)
                .env("UOR_LAB", "claude")
                .current_dir(&self.root)
                .output()
                .expect("uor-harness runs")
        }
    }

    fn stdout(o: &Output) -> String {
        String::from_utf8_lossy(&o.stdout).into_owned()
    }

    /// Exit `code`, a `<verdict> <rule>` line, and `text` somewhere in the output.
    fn expect(o: &Output, code: i32, line: &str, text: &str) {
        let out = stdout(o);
        assert_eq!(o.status.code(), Some(code), "{out}\n{o:?}");
        assert!(
            out.lines().any(|l| l.starts_with(&format!("{line} "))),
            "no `{line}` line:\n{out}"
        );
        assert!(out.contains(text), "no `{text}`:\n{out}");
    }
    /// [`record`] for the ORDER at `order`.
    fn record_on(order: &str, lab: &str, sha: &str, verdict: &str) -> serde_json::Value {
        use uor_r4_harness::loopreview::{render, DoneWhen, Record, Verdict};
        let body = render(&Record {
            lab: lab.to_string(),
            pr: 5,
            head: sha.to_string(),
            model: "m".to_string(),
            launch: "l".to_string(),
            verdict: if verdict == "APPROVE" {
                Verdict::Approve
            } else {
                Verdict::Changes
            },
            order: order.to_string(),
            fresh_context: true,
            done_when: DoneWhen::Host("cargo test: ok".to_string()),
        });
        serde_json::json!({"author": "someone", "body": body})
    }

    fn fake(comments: Vec<serde_json::Value>) -> String {
        serde_json::json!({"comments": comments, "checkRuns": [], "statuses": []}).to_string()
    }

    impl Repo {
        fn check_gh(&self, head: &str, fake: &str) -> Output {
            self.check_raw(&self.base, head, &["--pr", "5"], Some(fake))
        }
    }
    const ORDER_A2: &str = "docs/loop/tasks/item-a/claude-2/ORDER.md";
    const REPORT_A2: &str = "docs/loop/tasks/item-a/claude-2/REPORT.md";

    /// A repair PR (loop step 11): `claude-1` alone first, then work, then `claude-2` added by
    /// `repair`, then the REPORT in `claude-2`'s folder alone.
    fn repair_pr(r: &Repo, repair: impl FnOnce(&Repo) + 'static) -> String {
        r.commits(vec![
            Box::new(|r: &Repo| r.order_a()),
            Box::new(|r: &Repo| r.write("crates/a/src/lib.rs", "//\n")),
            Box::new(repair),
            Box::new(|r: &Repo| r.write(REPORT_A2, REPORT)),
        ])
    }

    /// The highest `<lab>-<n>` of one lab's ORDERs in one task folder is the active ORDER: `order`,
    /// `commit-order` and `report` judge `claude-2`, and `scope` allows the history folder.
    #[test]
    fn a_repair_pr_resolves_to_the_highest_order() {
        let r = Repo::new();
        let h = repair_pr(&r, |r| r.write(ORDER_A2, &order("claude", "item-a", "")));
        let o = r.check(&h, &[]);
        expect(&o, 0, "PASS order", "verdict: PASS");
        expect(&o, 0, "PASS commit-order", "1 repair ORDERs is added alone");
        expect(&o, 0, "PASS report", REPORT_A2);
        expect(&o, 0, "PASS scope", "verdict: PASS");
    }

    /// A repair ORDER added with a code file fails; so does a PR whose first commit is not the
    /// lowest ORDER.
    #[test]
    fn commit_order_needs_each_repair_order_alone_and_the_lowest_first() {
        let r = Repo::new();
        let h = repair_pr(&r, |r| {
            r.write(ORDER_A2, &order("claude", "item-a", ""));
            r.write("crates/a/x.rs", "\n");
        });
        expect(
        &r.check(&h, &[]),
        0,
        "WARN commit-order",
        "adds the repair ORDER docs/loop/tasks/item-a/claude-2/ORDER.md but also changes crates/a/x.rs",
    );

        let h = r.commits(vec![
            Box::new(|r: &Repo| r.write(ORDER_A2, &order("claude", "item-a", ""))),
            Box::new(|r: &Repo| r.order_a()),
            Box::new(|r: &Repo| r.write(REPORT_A2, REPORT)),
        ]);
        expect(
        &r.check(&h, &[]),
        0,
        "WARN commit-order",
        "changes docs/loop/tasks/item-a/claude-2/ORDER.md before the commit that adds docs/loop/tasks/item-a/claude-1/ORDER.md",
    );
    }

    /// The planted fault: ORDERs of two labs, or in two task folders, are refused.
    #[test]
    fn orders_of_two_labs_or_two_folders_are_refused() {
        let r = Repo::new();
        let h = repair_pr(&r, |r| {
            r.write(
                "docs/loop/tasks/item-a/codex-2/ORDER.md",
                &order("codex", "item-a", ""),
            )
        });
        let o = r.check(&h, &[]);
        expect(&o, 1, "FAIL order", "labs: claude, codex");
        expect(
            &o,
            1,
            "WARN commit-order",
            "not one lab's repairs in one task folder",
        );

        let h = repair_pr(&r, |r| {
            r.write(
                "docs/loop/tasks/item-b/claude-2/ORDER.md",
                &order("claude", "item-a", ""),
            )
        });
        expect(
            &r.check(&h, &[]),
            1,
            "FAIL order",
            "folders: item-a, item-b",
        );
    }

    /// A counted APPROVE for the superseded `claude-1` does not count once `claude-2` is the
    /// active ORDER; one for `claude-2` counts.
    #[test]
    fn a_review_record_for_a_superseded_order_does_not_count() {
        let r = Repo::new();
        let h = repair_pr(&r, |r| r.write(ORDER_A2, &order("claude", "item-a", "")));
        let o = r.check_gh(&h, &fake(vec![record_on(ORDER_A, "codex", &h, "APPROVE")]));
        expect(&o, 1, "FAIL review-record", "is not this PR's ORDER");
        let o = r.check_gh(&h, &fake(vec![record_on(ORDER_A2, "codex", &h, "APPROVE")]));
        expect(&o, 1, "PASS review-record", "counted");
    }
}
