//! `uor-harness loop pr-check` against temporary git repositories: a base commit holding the
//! registries and items, and one head per case built from it. Every child process has every
//! `GIT_*` variable removed, and each repository is checked to sit inside its temporary directory.

use std::path::PathBuf;
use std::process::{Command, Output};
use uor_r4_harness::loopprcheck::glob_match;

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
/// Like the real pages, prose follows the results table, so a new row lands mid-file.
const NOTEBOOK: &str =
    "# A notebook\n\n| row | result |\n|---|---|\n| 1 | first |\n\n## What the rows say\n\nProse.\n";
const FILEMAP: &str = "# FILEMAP\n\n| path | what |\n|---|---|\n| crates/a | the a crate |\n| `crates/` | crates |\n| `crates/a` | a |\n| `crates/uor-r4-discrete` | d |\n| `docs` | docs |\n| `docs/harness` | h |\n| `docs/loop` | loop |\n| `docs/research` | r |\n| `docs/roadmap` | rm |\n| `docs/start` | s |\n";
const OWNER_PATHS: &str = r#"{"schema": 1, "paths": [
  {"path": "CHARTER.md", "why": "rules"},
  {"path": ".github/**", "why": "CI"},
  {"path": "docs/harness/owner-paths.json", "why": "this list"}
]}"#;
/// A base file: one gated D23 item behind a block comment, a benign gated item, an item whose
/// identifier is only the English word properties, a gated function whose body has char-literal
/// braces, and a trailing function. The banned word is assembled with concat! so this test file
/// adds no ratchet site itself.
const GATED_BASE: &str = concat!(
    "#[cfg(any())]\n",
    "/* retired classifier, kept for history */\n",
    "pub struct Soft",
    "maxHead {\n",
    "    pub logits: Vec<f64>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct RetiredHelper {\n",
    "    pub n: u32,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct ConfigProperties {\n",
    "    pub properties: Vec<String>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub fn parser() -> bool {\n",
    "    let open = '{';\n",
    "    open == '{'\n",
    "}\n\n",
    "pub fn a() {}\n"
);

/// The same file with only the D23 item un-gated.
const GATED_HEAD_UNGATED: &str = concat!(
    "pub struct Soft",
    "maxHead {\n",
    "    pub logits: Vec<f64>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct RetiredHelper {\n",
    "    pub n: u32,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct ConfigProperties {\n",
    "    pub properties: Vec<String>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub fn parser() -> bool {\n",
    "    let open = '{';\n",
    "    open == '{'\n",
    "}\n\n",
    "pub fn a() {}\n"
);

/// The D23 item moved to a test-only gate.
const GATED_HEAD_TEST_ONLY: &str = concat!(
    "#[cfg(test)]\n",
    "pub struct Soft",
    "maxHead {\n",
    "    pub logits: Vec<f64>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct RetiredHelper {\n",
    "    pub n: u32,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct ConfigProperties {\n",
    "    pub properties: Vec<String>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub fn parser() -> bool {\n",
    "    let open = '{';\n",
    "    open == '{'\n",
    "}\n\n",
    "pub fn a() {}\n"
);

/// Only the benign item un-gated.
const GATED_HEAD_BENIGN: &str = concat!(
    "#[cfg(any())]\n",
    "pub struct Soft",
    "maxHead {\n",
    "    pub logits: Vec<f64>,\n",
    "}\n\n",
    "pub struct RetiredHelper {\n",
    "    pub n: u32,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct ConfigProperties {\n",
    "    pub properties: Vec<String>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub fn parser() -> bool {\n",
    "    let open = '{';\n",
    "    open == '{'\n",
    "}\n\n",
    "pub fn a() {}\n"
);

/// Only the properties-named item un-gated: the identifier is English, not a D23 family.
const GATED_HEAD_PLAIN_WORD: &str = concat!(
    "#[cfg(any())]\n",
    "pub struct Soft",
    "maxHead {\n",
    "    pub logits: Vec<f64>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct RetiredHelper {\n",
    "    pub n: u32,\n",
    "}\n\n",
    "pub struct ConfigProperties {\n",
    "    pub properties: Vec<String>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub fn parser() -> bool {\n",
    "    let open = '{';\n",
    "    open == '{'\n",
    "}\n\n",
    "pub fn a() {}\n"
);

/// Only the char-literal-brace function un-gated.
const GATED_HEAD_CHARLIT: &str = concat!(
    "#[cfg(any())]\n",
    "pub struct Soft",
    "maxHead {\n",
    "    pub logits: Vec<f64>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct RetiredHelper {\n",
    "    pub n: u32,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct ConfigProperties {\n",
    "    pub properties: Vec<String>,\n",
    "}\n\n",
    "pub fn parser() -> bool {\n",
    "    let open = '{';\n",
    "    open == '{'\n",
    "}\n\n",
    "pub fn a() {}\n"
);

/// The D23 item renamed and un-gated: the same banned code still compiles, so the site delta
/// catches it even though no item key survives.
const GATED_HEAD_RENAMED: &str = concat!(
    "pub struct Soft",
    "maxHeadRenamed {\n",
    "    pub logits: Vec<f64>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct RetiredHelper {\n",
    "    pub n: u32,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct ConfigProperties {\n",
    "    pub properties: Vec<String>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub fn parser() -> bool {\n",
    "    let open = '{';\n",
    "    open == '{'\n",
    "}\n\n",
    "pub fn a() {}\n"
);

/// The D23 item deleted outright: nothing compiles, so this stays lawful.
const GATED_HEAD_DELETED: &str = concat!(
    "#[cfg(any())]\n",
    "pub struct RetiredHelper {\n",
    "    pub n: u32,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct ConfigProperties {\n",
    "    pub properties: Vec<String>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub fn parser() -> bool {\n",
    "    let open = '{';\n",
    "    open == '{'\n",
    "}\n\n",
    "pub fn a() {}\n"
);

/// The gate moved from the item to its enclosing module: nothing compiles, so this stays lawful.
const GATED_HEAD_PARENT_GATE: &str = concat!(
    "#[cfg(any())]\n",
    "pub mod retired {\n",
    "    pub struct Soft",
    "maxHead {\n",
    "        pub logits: Vec<f64>,\n",
    "    }\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct RetiredHelper {\n",
    "    pub n: u32,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub struct ConfigProperties {\n",
    "    pub properties: Vec<String>,\n",
    "}\n\n",
    "#[cfg(any())]\n",
    "pub fn parser() -> bool {\n",
    "    let open = '{';\n",
    "    open == '{'\n",
    "}\n\n",
    "pub fn a() {}\n"
);

/// A second item's base file in the D32 scan scope: a gated float function.
const GATED_FLOAT_BASE: &str =
    "#[cfg(any())]\npub fn scale(x: f64) -> f64 {\n    0.5 * x\n}\n\npub fn b() {}\n";

/// The same file with the float function un-gated.
const GATED_FLOAT_HEAD: &str = "pub fn scale(x: f64) -> f64 {\n    0.5 * x\n}\n\npub fn b() {}\n";

const ORDER_B: &str = "docs/loop/tasks/item-b/claude-1/ORDER.md";

/// A valid REPORT, headings worded as in a real one.
const REPORT: &str = "# REPORT\n\n## What did not work, first\n\nNothing.\n\n## What changed\n\nA.\n\n## Measurements\n\nNone.\n\n## Checks run at the head\n\n- tests\n\n## Skills used\n\nNone.\n\n## Still open\n\nNothing.\n\n## Plain summary\n\nDone.\n";
const APPROVAL: &str = "## Owner approval for `.github/**` (an owner path)\n\nRecorded at 2026-10-21T15:25:16Z.\n- \"Approve?\" Casey: \"Approve (Recommended)\".\n";
const ORDER_A: &str = "docs/loop/tasks/item-a/claude-1/ORDER.md";
const REPORT_A: &str = "docs/loop/tasks/item-a/claude-1/REPORT.md";

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
    format!("# ORDER\n\nLab: {lab}. Item: [{id}](../../../../roadmap/items/{id}.md) (#1).\n{extra}")
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
        r.write("crates/a/src/gated.rs", GATED_BASE);
        r.write(
            "docs/roadmap/items/item-b.md",
            "---\nid: item-b\nstatus: ready\nscope: [crates/uor-r4-discrete/**]\n---\n\n## Why\n",
        );
        r.write(
            "crates/uor-r4-discrete/src/gated_float.rs",
            GATED_FLOAT_BASE,
        );
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

    /// The fixture with `edit` committed on top of the base, which becomes the new base.
    fn with_base(mut self, edit: impl FnOnce(&Repo)) -> Repo {
        self.git(&["checkout", "-q", "--detach", &self.base]);
        edit(&self);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", "base edit"]);
        self.base = self.out(&["rev-parse", "HEAD"]);
        self
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

    /// A head commit built from the base by `edit`; returns its sha.
    fn head(&self, edit: impl FnOnce(&Repo)) -> String {
        self.git(&["checkout", "-q", "--detach", &self.base]);
        edit(self);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", "head"]);
        self.out(&["rev-parse", "HEAD"])
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

    /// A green PR: the ORDER at `path` alone in the first commit, then a valid REPORT and `work`.
    fn green(&self, path: &str, text: &str, work: impl FnOnce(&Repo) + 'static) -> String {
        let (path, text) = (path.to_string(), text.to_string());
        let report = path.replace("ORDER.md", "REPORT.md");
        self.commits(vec![
            Box::new(move |r: &Repo| r.write(&path, &text)),
            Box::new(move |r: &Repo| {
                r.write(&report, REPORT);
                work(r);
            }),
        ])
    }

    /// A green PR for `item-a` by claude, with `work` after the ORDER commit.
    fn green_a(&self, work: impl FnOnce(&Repo) + 'static) -> String {
        self.green(ORDER_A, &order("claude", "item-a", ""), work)
    }

    /// A green PR for `item-b` by claude (the D32 scan scope item).
    fn green_b(&self, work: impl FnOnce(&Repo) + 'static) -> String {
        self.green(ORDER_B, &order("claude", "item-b", ""), work)
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

#[test]
fn a_green_pr_passes_order_and_scope() {
    let r = Repo::new();
    let h = r.green_a(|r| {
        r.write("crates/a/src/lib.rs", "pub fn a() { let _ = 1; }\n");
        r.write("crates/a/src/new.rs", "\n");
    });
    let o = r.check(&h, &[]);
    expect(&o, 0, "PASS order", "verdict: PASS");
    expect(&o, 0, "PASS scope", "size: tests 0 file(s)");
    expect(&o, 0, "PASS commit-order", "verdict: PASS");
    expect(&o, 0, "PASS report", REPORT_A);
    expect(&o, 0, "PASS owner-paths", "no owner path changed");
}

#[test]
fn commit_order_needs_the_order_commit_first_and_alone() {
    let r = Repo::new();
    let h = r.commits(vec![
        Box::new(|r: &Repo| r.write("crates/a/src/lib.rs", "//\n")),
        Box::new(|r: &Repo| {
            r.order_a();
            r.write(REPORT_A, REPORT);
        }),
    ]);
    expect(
        &r.check(&h, &[]),
        0,
        "WARN commit-order",
        "changes crates/a/src/lib.rs before the commit that adds",
    );

    let h = r.commits(vec![Box::new(|r: &Repo| {
        r.order_a();
        r.write(REPORT_A, REPORT);
        r.write("crates/a/x.rs", "\n");
    })]);
    expect(
        &r.check(&h, &[]),
        0,
        "WARN commit-order",
        "also changes crates/a/x.rs, outside docs/loop/tasks/item-a/claude-1/",
    );
}

#[test]
fn commit_order_skips_a_merge_of_main_after_the_order_commit() {
    let r = Repo::new();
    // main moves on after the PR branched.
    let main = r.commits(vec![Box::new(|r: &Repo| r.write("docs/b.md", "# b\n"))]);
    let order_commit = r.green_a(|r| r.write("crates/a/src/new.rs", "\n"));
    r.git(&["checkout", "-q", "--detach", &order_commit]);
    r.git(&["merge", "-q", "--no-ff", "-m", "merge main", &main]);
    r.write("crates/a/src/more.rs", "\n");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "after the merge"]);
    let h = r.out(&["rev-parse", "HEAD"]);
    let o = r.check_against(&main, &h, &[]);
    expect(&o, 0, "PASS commit-order", "verdict: PASS");
}

#[test]
fn commit_order_warns_on_an_evil_merge_before_the_order_commit() {
    let r = Repo::new();
    // E: an empty commit on the base.
    let e = r.commits(vec![Box::new(|_: &Repo| {})]);
    // M: a merge of the base and E whose own tree carries work neither parent has.
    r.git(&["checkout", "-q", "--detach", &r.base]);
    r.write("crates/a/src/evil.rs", "\n");
    r.git(&["add", "-A"]);
    let tree = r.out(&["write-tree"]);
    let m = r.out(&["commit-tree", &tree, "-p", &r.base, "-p", &e, "-m", "merge"]);
    // O: adds only the ORDER; then the REPORT.
    r.git(&["checkout", "-q", "--detach", &m]);
    r.order_a();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "order"]);
    r.write(REPORT_A, REPORT);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "report"]);
    let h = r.out(&["rev-parse", "HEAD"]);
    expect(
        &r.check(&h, &[]),
        0,
        "WARN commit-order",
        "is a merge before the commit that adds docs/loop/tasks/item-a/claude-1/ORDER.md",
    );
}

#[test]
fn commit_order_warns_on_an_order_re_pointed_after_its_commit() {
    let r = Repo::new();
    let h = r.green_a(|r| {
        r.write(
            ORDER_A,
            &order("claude", "item-noscope", "Scope widened.\n"),
        )
    });
    expect(
        &r.check(&h, &[]),
        0,
        "WARN commit-order",
        "`Item:` at the head differs from the ORDER as",
    );
    // Other ORDER edits after its commit stay allowed.
    let h = r.green_a(|r| r.write(ORDER_A, &order("claude", "item-a", "Scope widened.\n")));
    expect(&r.check(&h, &[]), 0, "PASS commit-order", "verdict: PASS");
}

#[test]
fn commit_order_counts_a_renamed_order_as_adding_it() {
    let r = Repo::new();
    // The base holds a draft in the ORDER's folder; the ORDER commit renames it to ORDER.md.
    let draft = "docs/loop/tasks/item-a/claude-1/DRAFT.md";
    let text = order("claude", "item-a", "");
    let base = r.commits(vec![Box::new(move |r: &Repo| r.write(draft, &text))]);
    r.git(&["mv", draft, ORDER_A]);
    r.git(&["commit", "-q", "-m", "order"]);
    r.write(REPORT_A, REPORT);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "report"]);
    let h = r.out(&["rev-parse", "HEAD"]);
    let o = r.check_against(&base, &h, &[]);
    expect(
        &o,
        0,
        "PASS commit-order",
        "adds docs/loop/tasks/item-a/claude-1/ORDER.md first",
    );
}

#[test]
fn scope_sees_a_submodule_a_config_would_hide() {
    let r = Repo::new();
    r.git(&["config", "diff.ignoreSubmodules", "all"]);
    r.git(&["checkout", "-q", "--detach", &r.base]);
    r.order_a();
    r.git(&["add", "-A"]);
    let cacheinfo = format!("160000,{},sub", r.base);
    r.git(&["update-index", "--add", "--cacheinfo", &cacheinfo]);
    r.git(&["commit", "-q", "-m", "order and a submodule"]);
    let h = r.out(&["rev-parse", "HEAD"]);
    expect(
        &r.check(&h, &[]),
        1,
        "WARN scope",
        "sub is outside item item-a's scope",
    );
}

#[test]
fn report_needs_every_section_in_order_and_non_empty() {
    let r = Repo::new();
    let h = r.commits(vec![Box::new(|r: &Repo| r.order_a())]);
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL report",
        "docs/loop/tasks/item-a/claude-1/REPORT.md is missing at the head",
    );
    let skills = "## Skills used\n\nNone.\n\n";
    let still = "## Still open\n\nNothing.\n\n";
    let cases: Vec<(String, &str)> = vec![
        (REPORT.replace(skills, ""), "no `## Skills used` heading"),
        (
            REPORT
                .replace(still, "")
                .replace("## Measurements", &format!("{still}## Measurements")),
            "out of order: `## Still open` comes before",
        ),
        (
            REPORT.replace("## Measurements\n\nNone.\n\n", "## Measurements\n\n  \n\n"),
            "the `## Measurements` section is empty",
        ),
        (
            REPORT.replace(skills, &format!("```md\n{skills}```\n\n")),
            "no `## Skills used` heading",
        ),
        (
            REPORT.replace(skills, &format!("~~~\n```\n{skills}~~~\n\n")),
            "no `## Skills used` heading",
        ),
    ];
    for (report, why) in cases {
        let h = r.green_a(move |r| r.write(REPORT_A, &report));
        expect(&r.check(&h, &[]), 1, "FAIL report", why);
    }
}

#[test]
fn a_github_change_without_owner_approval_fails() {
    let r = Repo::new();
    let h = r.green_a(|r| r.write(".github/workflows/x.yml", "on: push\n"));
    let o = r.check(&h, &[]);
    expect(
        &o,
        1,
        "FAIL owner-paths",
        ".github/workflows/x.yml changed (owner path .github/**)",
    );
    assert!(stdout(&o).contains("the ORDER has no `## Owner approval` section"));
}

#[test]
fn owner_paths_need_the_entry_a_quoted_answer_and_a_time() {
    let r = Repo::new();
    let gh = |r: &Repo| r.write(".github/workflows/x.yml", "on: push\n");
    let h = r.green(ORDER_A, &order("claude", "item-a", APPROVAL), gh);
    expect(
        &r.check(&h, &[]),
        0,
        "PASS owner-paths",
        "owner approval recorded for .github/**",
    );

    let cases = [
        (
            APPROVAL.replace("2026-10-21T15:25:16Z", "21 Oct"),
            "holds no ISO UTC time",
        ),
        (
            APPROVAL.replace("`.github/**`", "CI"),
            "does not name .github/**",
        ),
        (APPROVAL.replace('"', "'"), "holds no double-quoted answer"),
        (
            APPROVAL
                .replace("\"Approve (Recommended)\"", "'Approve'")
                .replace("\"Approve?\" Casey:", "\"Approve? Casey:\n yes\""),
            "holds no double-quoted answer",
        ),
        // A `# ` heading closes the section.
        (
            APPROVAL.replace("Recorded at", "# Next\n\nRecorded at"),
            "holds no ISO UTC time",
        ),
    ];
    for (approval, why) in cases {
        let h = r.green(ORDER_A, &order("claude", "item-a", &approval), gh);
        expect(&r.check(&h, &[]), 1, "FAIL owner-paths", why);
    }

    // An exact entry: CHARTER.md.
    let h = r.green_a(|r| r.write("CHARTER.md", "# Charter, changed\n"));
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL owner-paths",
        "CHARTER.md changed (owner path CHARTER.md)",
    );
}

#[test]
fn owner_paths_are_read_at_the_merge_base() {
    let r = Repo::new();
    let h = r.green_a(|r| {
        r.write(
            "docs/harness/owner-paths.json",
            &OWNER_PATHS.replace("  {\"path\": \".github/**\", \"why\": \"CI\"},\n", ""),
        );
        r.write(".github/workflows/x.yml", "on: push\n");
    });
    let o = r.check(&h, &[]);
    expect(
        &o,
        1,
        "FAIL owner-paths",
        ".github/workflows/x.yml changed (owner path .github/**)",
    );
}

#[test]
fn the_real_claude_16_order_approves_its_github_change() {
    let real = include_str!("../../../docs/loop/tasks/infra-loop-harness/claude-16/ORDER.md");
    let r = Repo::new();
    let h = r.green(ORDER_A, real, |r| {
        r.write(".github/workflows/gate.yml", "on: push\n")
    });
    let o = r.check(&h, &[]);
    let out = stdout(&o);
    assert!(
        out.lines()
            .any(|l| l == "PASS owner-paths owner approval recorded for .github/**"),
        "{out}"
    );
}

#[test]
fn order_fails_for_each_broken_condition() {
    let r = Repo::new();
    let cases: Vec<(&str, Edit)> = vec![
        (
            "adds no docs/loop/tasks",
            Box::new(|r: &Repo| r.write("crates/a/src/lib.rs", "//\n")),
        ),
        (
            "says `Lab: codex` but its folder is lab claude",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/loop/tasks/item-a/claude-1/ORDER.md",
                    &order("codex", "item-a", ""),
                )
            }),
        ),
        (
            "item item-zz does not exist at the merge base",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/loop/tasks/item-zz/claude-1/ORDER.md",
                    &order("claude", "item-zz", ""),
                )
            }),
        ),
        (
            "lab deepseek (from the folder) is not registered",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/loop/tasks/item-a/deepseek-1/ORDER.md",
                    &order("deepseek", "item-a", ""),
                )
            }),
        ),
        (
            "needs a `Setup row: <x>` line",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/loop/tasks/infra-loop-pr9/claude-1/ORDER.md",
                    &order("claude", "infra-loop-pr9", ""),
                )
            }),
        ),
        (
            "needs a `Setup row: <x>` line",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/loop/tasks/infra-loop-pr9/claude-1/ORDER.md",
                    &order("claude", "infra-loop-pr9", "Setup row:\n5\n"),
                )
            }),
        ),
        (
            "has no `Lab: <lab>` line",
            Box::new(|r: &Repo| {
                r.write(
                    ORDER_A,
                    "# ORDER\n\nLab:\nclaude. Item: [item-a](../../../../roadmap/items/item-a.md).\n",
                )
            }),
        ),
        (
            // One lab's repair chain is allowed (loop step 11); two labs are not.
            "the PR adds 2 ORDERs that are not one lab's repairs in one task folder",
            Box::new(|r: &Repo| {
                r.order_a();
                r.write(
                    "docs/loop/tasks/item-a/codex-2/ORDER.md",
                    &order("codex", "item-a", ""),
                );
            }),
        ),
    ];
    for (why, edit) in cases {
        let h = r.head(edit);
        expect(&r.check(&h, &[]), 1, "FAIL order", why);
    }
    // The setup row satisfies the setup-PR condition.
    let h = r.green(
        "docs/loop/tasks/infra-loop-pr9/claude-1/ORDER.md",
        &order("claude", "infra-loop-pr9", "Setup row: 5\n"),
        |_| {},
    );
    expect(&r.check(&h, &[]), 0, "PASS order", "verdict: PASS");
}

#[test]
fn an_out_of_scope_change_warns_scope() {
    let r = Repo::new();
    let h = r.head(|r| {
        r.order_a();
        r.write("crates/b/x.rs", "\n");
    });
    expect(
        &r.check(&h, &[]),
        1,
        "WARN scope",
        "crates/b/x.rs is outside item item-a's scope",
    );
}

/// D76: an otherwise green PR with an out-of-scope file passes, and the scope finding still
/// prints. Reverting the advisory demotion in `evaluate` turns this exit 0 into exit 1.
#[test]
fn an_out_of_scope_change_warns_but_passes_d76() {
    let r = Repo::new();
    let h = r.green_a(|r| r.write("crates/b/x.rs", "\n"));
    let o = r.check(&h, &[]);
    expect(
        &o,
        0,
        "WARN scope",
        "crates/b/x.rs is outside item item-a's scope",
    );
    assert!(stdout(&o).contains("advisory (D76)"), "{}", stdout(&o));
    assert!(stdout(&o).contains("verdict: PASS"), "{}", stdout(&o));
}

#[test]
fn scope_warns_on_a_scopeless_item_a_crossing_glob_and_a_rename_out() {
    let r = Repo::new();
    let h = r.head(|r| {
        r.write(
            "docs/loop/tasks/item-noscope/claude-1/ORDER.md",
            &order("claude", "item-noscope", ""),
        )
    });
    expect(
        &r.check(&h, &[]),
        1,
        "WARN scope",
        "item item-noscope has no scope: at the base",
    );

    let h = r.head(|r| {
        r.order_a();
        r.write("crates/ab/x.rs", "\n");
    });
    expect(
        &r.check(&h, &[]),
        1,
        "WARN scope",
        "crates/ab/x.rs is outside",
    );

    let h = r.head(|r| {
        r.order_a();
        r.write("crates/b/lib.rs", "");
        std::fs::remove_file(r.root.join("crates/b/lib.rs")).unwrap();
        r.git(&["mv", "crates/a/src/lib.rs", "crates/b/lib.rs"]);
    });
    let o = r.check(&h, &[]);
    expect(&o, 1, "WARN scope", "crates/b/lib.rs is outside");
    assert!(!stdout(&o).contains("crates/a/src/lib.rs is outside"));
}

#[test]
fn scope_allows_a_new_notebook_introduced_with_its_binding() {
    let r = Repo::new().with_base(|r| {
        r.write(
            "docs/roadmap/items/item-a.md",
            &ITEM_A
                .replace("scope: [", "scope: [docs/roadmap/items/*.md, ")
                .replace("notebook: docs/research/notebook/a.md\n", ""),
        );
    });
    let h = r.green_a(|r| {
        r.write(
            "docs/roadmap/items/item-a.md",
            &ITEM_A
                .replace("scope: [", "scope: [docs/roadmap/items/*.md, ")
                .replace(
                    "docs/research/notebook/a.md",
                    "docs/research/notebook/m2-grounded-reply.md",
                ),
        );
        r.write(
            "docs/research/notebook/m2-grounded-reply.md",
            "# Notebook: grounded reply (M2)\n\n## Results\n\n| Date | What ran | Number | Bar | Verdict | Artifact |\n|---|---|---|---|---|---|\n| | *No result rows yet.* | | | | |\n",
        );
    });
    expect(&r.check(&h, &[]), 0, "PASS scope", "verdict: PASS");
}

#[test]
fn scope_allows_a_new_notebook_for_another_item_bound_in_the_same_pr() {
    let r = Repo::new().with_base(|r| {
        r.write(
            "docs/roadmap/items/infra-loop-pr9.md",
            "---\nid: infra-loop-pr9\nstatus: ready\nscope: [docs/roadmap/items/*.md]\n---\n",
        );
        r.write(
            "docs/roadmap/items/item-a.md",
            &ITEM_A.replace("notebook: docs/research/notebook/a.md\n", ""),
        );
    });
    let h = r.green(
        "docs/loop/tasks/infra-loop-pr9/claude-1/ORDER.md",
        &order("claude", "infra-loop-pr9", "Setup row: 2\n"),
        |r| {
            r.write(
                "docs/roadmap/items/item-a.md",
                &ITEM_A.replace(
                    "docs/research/notebook/a.md",
                    "docs/research/notebook/m2-grounded-reply.md",
                ),
            );
            r.write(
                "docs/research/notebook/m2-grounded-reply.md",
                "# Notebook: grounded reply (M2)\n",
            );
        },
    );
    expect(&r.check(&h, &[]), 0, "PASS scope", "verdict: PASS");
}

#[test]
fn scope_always_allows_status_notebook_append_mistakes_and_filemap_rows() {
    let r = Repo::new();
    let pass: Vec<Edit> = vec![
        Box::new(|r: &Repo| {
            r.write(
                "docs/roadmap/items/item-a.md",
                &ITEM_A.replace("status: ready", "status: done"),
            )
        }),
        Box::new(|r: &Repo| {
            r.write(
                "docs/research/notebook/a.md",
                &format!("{NOTEBOOK}| 2 | second |\n"),
            )
        }),
        // Planted fault: a row added inside the table, before the trailing prose, was refused.
        Box::new(|r: &Repo| {
            r.write(
                "docs/research/notebook/a.md",
                &NOTEBOOK.replace("| 1 | first |\n", "| 1 | first |\n| 2 | second |\n"),
            )
        }),
        Box::new(|r: &Repo| r.write("docs/harness/mistakes/2026-10-21-x.md", "# x\n")),
        Box::new(|r: &Repo| {
            r.write(
                "docs/start/FILEMAP.md",
                &format!("{FILEMAP}| crates/a/src/new.rs | new |\n"),
            )
        }),
    ];
    for edit in pass {
        let h = r.green_a(edit);
        expect(&r.check(&h, &[]), 0, "PASS scope", "verdict: PASS");
    }
    let fail: Vec<(&str, Edit)> = vec![
        (
            "docs/roadmap/items/item-a.md is outside",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/roadmap/items/item-a.md",
                    &ITEM_A.replace("Because.", "Because of x."),
                )
            }),
        ),
        (
            "docs/research/notebook/a.md is outside",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/research/notebook/a.md",
                    &NOTEBOOK.replace("first", "rewritten"),
                )
            }),
        ),
        (
            "docs/start/FILEMAP.md is outside",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/start/FILEMAP.md",
                    &FILEMAP.replace("| crates/a | the a crate |\n", ""),
                )
            }),
        ),
    ];
    for (why, edit) in fail {
        let h = r.head(|r| {
            r.order_a();
            edit(r);
        });
        expect(&r.check(&h, &[]), 1, "WARN scope", why);
    }
}

#[test]
fn registries_are_read_at_the_merge_base() {
    let r = Repo::new();
    // Removing codex in the same PR does not unregister it for this PR.
    let h = r.head(|r| {
        r.write("docs/harness/labs.json", &labs(&["claude"]));
        r.write(
            "docs/loop/tasks/item-a/codex-1/ORDER.md",
            &order("codex", "item-a", ""),
        );
    });
    let o = r.check(&h, &[]);
    expect(
        &o,
        1,
        "PASS order",
        "docs/loop/tasks/item-a/codex-1/ORDER.md",
    );
    // Registering a new lab in the same PR does not let it use the new name.
    let h = r.head(|r| {
        r.write(
            "docs/harness/labs.json",
            &labs(&["claude", "codex", "kimi"]),
        );
        r.write(
            "docs/loop/tasks/item-a/kimi-1/ORDER.md",
            &order("kimi", "item-a", ""),
        );
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL order",
        "lab kimi (from the folder) is not registered",
    );
}

#[test]
fn the_matcher_never_crosses_a_slash_except_by_double_star() {
    assert!(glob_match("crates/a/*.rs", "crates/a/x.rs"));
    assert!(!glob_match("crates/a/*.rs", "crates/a/src/x.rs"));
    assert!(!glob_match("crates/a*", "crates/ab/x.rs"));
    assert!(glob_match("crates/a*", "crates/ab"));
    assert!(glob_match("docs/?.md", "docs/a.md"));
    assert!(!glob_match("docs/?.md", "docs/ab.md"));
    assert!(!glob_match("docs?a.md", "docs/a.md"));
    assert!(glob_match("crates/a/**", "crates/a"));
    assert!(glob_match("crates/a/**", "crates/a/x.rs"));
    assert!(glob_match("crates/a/**", "crates/a/src/deep/x.rs"));
    assert!(!glob_match("crates/a/**", "crates/ab/x.rs"));
    assert!(glob_match("crates/**/x.rs", "crates/x.rs"));
    assert!(glob_match("crates/**/x.rs", "crates/a/b/x.rs"));
    assert!(!glob_match("crates/**/x.rs", "crates/a/b/y.rs"));
    assert!(!glob_match("crates/a**", "crates/a/x.rs"));
}

#[test]
fn a_bad_rev_exits_2_and_json_carries_the_verdict() {
    let r = Repo::new();
    let o = clean(&mut Command::new(BIN))
        .args(["--repo", r.root.to_str().unwrap(), "loop", "pr-check"])
        .args(["--base", "no-such-rev", "--head", "HEAD"])
        .env("UOR_LAB", "claude")
        .current_dir(&r.root)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stderr).contains("no-such-rev"));

    let h = r.green_a(|r| r.write("crates/a/src/lib.rs", "//\n"));
    let o = r.check(&h, &["--json"]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("one JSON object");
    assert_eq!(v["verdict"], "pass");
    assert_eq!(v["merge_base"], r.base.as_str());
    assert_eq!(v["head"], h.as_str());
    assert_eq!(v["size"]["code"]["files"], 1);
    // The ORDER and the REPORT.
    assert_eq!(v["size"]["docs"]["files"], 2);
    assert_eq!(v["rules"][1]["rule"], "scope");
    assert_eq!(v["rules"][1]["verdict"], "pass");

    let h = r.head(|r| {
        r.order_a();
        r.write("crates/b/x.rs", "\n");
    });
    let o = r.check(&h, &["--json"]);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["verdict"], "fail");
}

/// `rule-sources`: rule files and rule headings outside the base list fail; listed files and the
/// exempt folders pass, and a PR cannot list its own new file.
#[test]
fn rule_sources_warns_on_rules_outside_the_list() {
    let r = Repo::new();
    r.git(&["checkout", "-q", "--detach", &r.base]);
    r.write(
        "docs/harness/rule-sources.json",
        r#"{"files": ["docs/harness/pm-rules.md"]}"#,
    );
    r.write("docs/harness/pm-rules.md", "# PM\n\n## Rules\n");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "list"]);
    let base = r.out(&["rev-parse", "HEAD"]);
    let on = |work: Box<dyn FnOnce(&Repo)>| {
        r.git(&["checkout", "-q", "--detach", &base]);
        r.order_a();
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "order"]);
        work(&r);
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "work"]);
        r.check_against(&base, &r.out(&["rev-parse", "HEAD"]), &[])
    };
    let o = on(Box::new(|r| r.write("docs/team/house-rules.md", "x\n")));
    expect(
        &o,
        1,
        "WARN rule-sources",
        "docs/team/house-rules.md adds a rule-named file",
    );
    let o = on(Box::new(|r| r.write("docs/a.md", "# a\n\n## Team rules\n")));
    expect(
        &o,
        1,
        "WARN rule-sources",
        "adds the rule heading `## Team rules`",
    );
    let o = on(Box::new(|r| {
        r.write(
            "docs/harness/pm-rules.md",
            "# PM\n\n## Rules\n\n## More rules\n",
        )
    }));
    expect(&o, 1, "PASS rule-sources", "no rule stated outside");
    let o = on(Box::new(|r| {
        r.write("docs/loop/tasks/item-a/claude-1/rules.md", "## Rule\n")
    }));
    expect(&o, 1, "PASS rule-sources", "no rule stated outside");
    let o = on(Box::new(|r| {
        r.write("docs/team/house-rules.md", "x\n");
        r.write(
            "docs/harness/rule-sources.json",
            r#"{"files": ["docs/harness/pm-rules.md", "docs/team/house-rules.md"]}"#,
        );
    }));
    expect(
        &o,
        1,
        "WARN rule-sources",
        "docs/team/house-rules.md adds a rule-named file",
    );
    for (text, heading) in [
        ("# a\n\n## _Rules_\n", "`## _Rules_`"),
        ("# a\n\n## __Rules__\n", "`## __Rules__`"),
        ("# a\n\n   ## Rules\n", "`   ## Rules`"),
        ("# a\n\nTeam rules\n====\n", "`Team rules`"),
        ("# a\n\nTeam rules\n---\n", "`Team rules`"),
        ("# a\n\n#\tRules\n", "`#\tRules`"),
    ] {
        let o = on(Box::new(move |r| r.write("docs/a.md", text)));
        let want = format!("adds the rule heading {heading}");
        expect(&o, 1, "WARN rule-sources", &want);
    }
    let o = on(Box::new(|r| r.write("docs/x/HOUSE-RULES.MD", "x\n")));
    expect(
        &o,
        1,
        "WARN rule-sources",
        "docs/x/HOUSE-RULES.MD adds a rule-named file",
    );
    // Moving a listed file's rule text to an unlisted path states it outside the list.
    let o = on(Box::new(|r| {
        r.git(&["mv", "docs/harness/pm-rules.md", "docs/notes.md"])
    }));
    expect(
        &o,
        1,
        "WARN rule-sources",
        "docs/notes.md adds the rule heading `## Rules`",
    );
}

/// `rendered-lock`: N/A with no view; a view edit fails off `kimi/render` and passes on it;
/// AGENTS.md counts only between its markers.
#[test]
fn rendered_lock_holds_the_views_to_kimi_render() {
    let r = Repo::new();
    let h = r.green_a(|_| {});
    expect(
        &r.check(&h, &[]),
        0,
        "N/A rendered-lock",
        "no rendered view exists yet",
    );
    r.git(&["checkout", "-q", "--detach", &r.base]);
    r.write("STATE.md", "# State\n");
    let agents = "# Agents\n<!-- loop:lessons-top-ten:start -->\n1. one\n<!-- loop:lessons-top-ten:end -->\n";
    r.write("AGENTS.md", agents);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "views"]);
    let base = r.out(&["rev-parse", "HEAD"]);
    let on = |text: &'static str, path: &'static str| {
        r.git(&["checkout", "-q", "--detach", &base]);
        r.order_a();
        r.write(path, text);
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "edit"]);
        r.out(&["rev-parse", "HEAD"])
    };
    let h = on("# State\nnew\n", "STATE.md");
    expect(
        &r.check_against(&base, &h, &[]),
        1,
        "WARN rendered-lock",
        "STATE.md changed",
    );
    let o = r.check_against(&base, &h, &["--head-ref", "kimi/render"]);
    assert!(stdout(&o).contains("PASS rendered-lock"), "{}", stdout(&o));
    let h = on(
        "# Agents, renamed\n<!-- loop:lessons-top-ten:start -->\n1. one\n<!-- loop:lessons-top-ten:end -->\n",
        "AGENTS.md",
    );
    let o = r.check_against(&base, &h, &[]);
    assert!(stdout(&o).contains("PASS rendered-lock"), "{}", stdout(&o));
    let h = on(
        "# Agents\n<!-- loop:lessons-top-ten:start -->\n1. two\n<!-- loop:lessons-top-ten:end -->\n",
        "AGENTS.md",
    );
    expect(
        &r.check_against(&base, &h, &[]),
        1,
        "WARN rendered-lock",
        "AGENTS.md (lessons-top-ten markers) changed",
    );
}

/// `rendered-lock`: the views' first commit passes on any branch while no view exists at the
/// base (loop PR 8's migration); once one view exists at the base, adding another view is an
/// edit, and the kimi/render rule applies.
#[test]
fn rendered_lock_passes_the_views_first_commit_only() {
    let r = Repo::new();
    // No view at the base; the head adds every view on a feature branch: the first commit passes.
    let h = r.head(|r| {
        r.order_a();
        r.write("STATE.md", "# State\n");
        r.write("HANDOFF.md", "# Handoff\n");
        r.write("docs/harness/LESSONS.md", "# Lessons\n");
        r.write(
            "AGENTS.md",
            "# Agents\n<!-- loop:lessons-top-ten:start -->\n1. one\n<!-- loop:lessons-top-ten:end -->\n",
        );
    });
    let o = r.check(&h, &[]);
    assert!(stdout(&o).contains("PASS rendered-lock"), "{}", stdout(&o));

    // One view at the base; the head adds a second on a feature branch: not a first commit.
    let mut r = Repo::new();
    r.git(&["checkout", "-q", "--detach", &r.base]);
    r.write("STATE.md", "# State\n");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "state view"]);
    r.base = r.out(&["rev-parse", "HEAD"]);
    let h = r.head(|r| {
        r.order_a();
        r.write("HANDOFF.md", "# Handoff\n");
    });
    expect(
        &r.check_against(&r.base.clone(), &h, &[]),
        1,
        "WARN rendered-lock",
        "HANDOFF.md changed",
    );
}

/// `read-path`: every over-budget item fails, grown or not, and so does a whole path above the
/// 76,800 B line (the former warnings, raised to errors in loop PR 11).
#[test]
fn read_path_warns_on_every_over_budget_item_and_the_warning_line() {
    let lines = |n: usize| "x\n".repeat(n);
    let r = Repo::new();
    r.git(&["checkout", "-q", "--detach", &r.base]);
    r.write("AGENTS.md", &lines(70));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "agents"]);
    let base = r.out(&["rev-parse", "HEAD"]);
    r.write("AGENTS.md", &lines(81));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "grow"]);
    let h = r.out(&["rev-parse", "HEAD"]);
    expect(
        &r.check_against(&base, &h, &[]),
        1,
        "WARN read-path",
        "AGENTS.md is 81 lines (base 70)",
    );

    let mut r = Repo::new();
    r.git(&["checkout", "-q", "--detach", &r.base]);
    r.write("AGENTS.md", &lines(85));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "agents"]);
    r.base = r.out(&["rev-parse", "HEAD"]);
    let h = r.green_a(|_| {});
    expect(&r.check(&h, &[]), 0, "WARN read-path", "but did not grow");

    let r = Repo::new();
    let big = order("claude", "item-a", &"y".repeat(5000));
    let h = r.green(ORDER_A, &big, |_| {});
    expect(&r.check(&h, &[]), 0, "WARN read-path", "the ORDER is 50");

    // A path already over its total, which the PR does not touch, fails: the ORDER it adds is not
    // growth, and the former warning is an error now.
    let mut r = Repo::new();
    r.git(&["checkout", "-q", "--detach", &r.base]);
    r.write("CHARTER.md", &"c".repeat(130_000));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "charter"]);
    r.base = r.out(&["rev-parse", "HEAD"]);
    let h = r.green_a(|r| r.write("crates/a/src/lib.rs", "//\n"));
    let o = r.check(&h, &[]);
    expect(&o, 0, "WARN read-path", "but did not grow");

    // A read-path file turned into a symlink fails.
    let r = Repo::new();
    let h = r.green_a(|r| {
        let p = r.root.join("AGENTS.md");
        let _ = std::fs::remove_file(&p);
        std::os::unix::fs::symlink("docs/a.md", &p).unwrap();
    });
    expect(
        &r.check(&h, &[]),
        0,
        "WARN read-path",
        "AGENTS.md is a symlink at the head",
    );
}

/// `filemap`: N/A without the map; a missing row and a stale row fail; an added row passes.
#[test]
fn filemap_needs_a_row_per_directory_and_no_stale_rows() {
    let mut r = Repo::new();
    let h = r.green_a(|r| {
        std::fs::remove_file(r.root.join("docs/start/FILEMAP.md")).unwrap();
    });
    expect(&r.check(&h, &[]), 0, "N/A filemap", "does not exist yet");
    let h = r.green_a(|_| {});
    expect(&r.check(&h, &[]), 0, "PASS filemap", "has a row for every");
    r.git(&["checkout", "-q", "--detach", &r.base]);
    r.write(
        "docs/roadmap/items/item-a.md",
        &ITEM_A.replace("scope: [", "scope: [crates/new/**, "),
    );
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "scope"]);
    r.base = r.out(&["rev-parse", "HEAD"]);
    let h = r.green_a(|r| r.write("crates/new/x.rs", "\n"));
    expect(
        &r.check(&h, &[]),
        0,
        "WARN filemap",
        "missing row: crates/new",
    );
    let h = r.green_a(|r| {
        r.write("crates/new/x.rs", "\n");
        r.write(
            "docs/start/FILEMAP.md",
            &format!("{FILEMAP}| `crates/new` | new |\n"),
        );
    });
    expect(&r.check(&h, &[]), 0, "PASS filemap", "has a row for every");
    let h = r.green_a(|r| {
        r.write(
            "docs/start/FILEMAP.md",
            &format!("{FILEMAP}| `crates/gone/` | gone |\n"),
        );
    });
    expect(
        &r.check(&h, &[]),
        0,
        "WARN filemap",
        "stale row: crates/gone",
    );
}

const SETUP_ORDER: &str = "docs/loop/tasks/infra-loop-pr99-x/claude-1/ORDER.md";

/// A green setup PR (`infra-loop-pr99-x`, ORDER lab claude, `Setup row: 99`).
fn setup_pr(r: &Repo) -> String {
    let text = order("claude", "infra-loop-pr99-x", "Setup row: 99\n");
    r.green(SETUP_ORDER, &text, |r| {
        r.write("crates/a/src/lib.rs", "//\n")
    })
}

/// A `uor-review v1` record comment by `lab` on PR 5 at `sha` saying `verdict` (APPROVE or
/// CHANGES) for the setup PR's ORDER, with `context: fresh`.
fn record(lab: &str, sha: &str, verdict: &str) -> serde_json::Value {
    record_on(SETUP_ORDER, lab, sha, verdict)
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

/// `review-record` runs the review gate on the PR's `uor-review v1` records: an exact-head
/// APPROVE that counts, with no later counted CHANGES, passes; a setup PR counts claude alone
/// (D59), others another vendor (D46); untagged and free-text records do not count.
#[test]
fn review_record_counts_exact_head_records_from_allowed_labs() {
    let r = Repo::new();
    let h = setup_pr(&r);
    let o = r.check_gh(&h, &fake(vec![record("claude", &h, "APPROVE")]));
    expect(&o, 1, "PASS review-record", "counted");

    let o = r.check_gh(&h, &fake(vec![record("codex", &h, "APPROVE")]));
    expect(
        &o,
        1,
        "FAIL review-record",
        "a setup PR under D59 counts only claude records",
    );

    let other = "0".repeat(40);
    let o = r.check_gh(&h, &fake(vec![record("claude", &other, "APPROVE")]));
    expect(&o, 1, "FAIL review-record", "not the current head");

    let o = r.check_gh(
        &h,
        &fake(vec![
            record("claude", &h, "APPROVE"),
            record("claude", &h, "CHANGES"),
        ]),
    );
    expect(
        &o,
        1,
        "FAIL review-record",
        "fail: a counted CHANGES from claude stands",
    );

    let o = r.check_gh(&h, &fake(vec![]));
    expect(&o, 1, "FAIL review-record", "fail: no counted record at");

    // An untagged record does not count.
    let tagged = record("claude", &h, "APPROVE")["body"]
        .as_str()
        .unwrap()
        .to_string();
    let untagged = tagged.replacen("[claude] ", "", 1);
    let o = r.check_gh(
        &h,
        &fake(vec![
            serde_json::json!({"author": "someone", "body": untagged}),
        ]),
    );
    expect(&o, 1, "FAIL review-record", "malformed record: untagged");

    // An old free-text record no longer counts.
    let body = format!("[claude] Exact-head review record for `{h}`\n\n**Verdict: APPROVE**\n");
    let free = serde_json::json!({"author": "someone", "body": body});
    let o = r.check_gh(&h, &fake(vec![free]));
    expect(&o, 1, "FAIL review-record", "fail: no counted record at");

    // A code PR (D66): the ORDER lab's own fresh-context APPROVE counts, as does another vendor's;
    // the ORDER lab's record without `context: fresh` does not.
    let h = r.green_a(|r| r.write("crates/a/src/lib.rs", "//\n"));
    let o = r.check_gh(&h, &fake(vec![record_on(ORDER_A, "claude", &h, "APPROVE")]));
    expect(&o, 1, "PASS review-record", "counted");
    let mut stale = record_on(ORDER_A, "claude", &h, "APPROVE");
    let without = stale["body"]
        .as_str()
        .unwrap()
        .replace("context: fresh\n", "");
    assert_ne!(without, stale["body"].as_str().unwrap());
    stale["body"] = serde_json::json!(without);
    let o = r.check_gh(&h, &fake(vec![stale]));
    expect(&o, 1, "FAIL review-record", "needs `context: fresh` (D66)");
    let o = r.check_gh(&h, &fake(vec![record_on(ORDER_A, "codex", &h, "APPROVE")]));
    expect(&o, 1, "PASS review-record", "counted");
}

/// Without GitHub: `--local` and `--ci` make both GitHub rules N/A; no mode at all fails them.
#[test]
fn github_rules_need_a_mode() {
    let r = Repo::new();
    let h = r.green_a(|r| r.write("crates/a/src/lib.rs", "//\n"));
    let o = r.check_raw(&r.base, &h, &["--local"], None);
    expect(&o, 0, "N/A review-record", "local run");
    expect(&o, 0, "N/A required-checks", "local run");
    let o = r.check_raw(&r.base, &h, &["--ci"], None);
    expect(&o, 0, "N/A review-record", "the policy job runs before");
    expect(&o, 0, "N/A required-checks", "the policy job runs before");
    let o = r.check_raw(&r.base, &h, &[], None);
    expect(&o, 1, "FAIL review-record", "needs --pr <n>, or --local");
    expect(&o, 1, "FAIL required-checks", "needs --pr <n>, or --local");
}

const REQUIRED: [&str; 5] = [
    "fmt / clippy / tests / no_std / κ",
    "cargo audit",
    "fuzz smoke (nightly libfuzzer)",
    "wasm-pack build (dashboard facade, deploy.yml parity)",
    "Gate C trend alarm",
];

/// A fake whose check runs are the five required names, each from `app` with `summary`, minus
/// `skip`, plus the commit statuses `statuses`.
fn checks(app: &str, summary: &str, skip: Option<&str>, statuses: &[&str]) -> String {
    let runs: Vec<serde_json::Value> = REQUIRED
        .iter()
        .filter(|n| Some(**n) != skip)
        .map(|n| serde_json::json!({"name": n, "app": app, "summary": summary}))
        .collect();
    let st: Vec<serde_json::Value> = statuses
        .iter()
        .map(|c| serde_json::json!({"context": c, "state": "success"}))
        .collect();
    serde_json::json!({"comments": [], "checkRuns": runs, "statuses": st}).to_string()
}

/// `required-checks`: every required name needs a github-actions check run with the policy record,
/// and a commit status under a required name fails.
#[test]
fn required_checks_need_the_policy_record_from_github_actions() {
    let r = Repo::new();
    let h = r.green_a(|r| r.write("crates/a/src/lib.rs", "//\n"));
    let pass = "policy: pass (run 42)";
    let o = r.check_gh(&h, &checks("github-actions", pass, None, &[]));
    expect(&o, 1, "PASS required-checks", "all 5 required checks");
    assert!(
        stdout(&o).contains("FAKE GitHub: UOR_PRCHECK_GH_FAKE="),
        "{}",
        stdout(&o)
    );

    let o = r.check_gh(
        &h,
        &checks("github-actions", pass, Some("cargo audit"), &[]),
    );
    expect(
        &o,
        1,
        "FAIL required-checks",
        "`cargo audit` has no check run",
    );

    let o = r.check_gh(&h, &checks("github-actions", pass, None, &["cargo audit"]));
    expect(
        &o,
        1,
        "FAIL required-checks",
        "a commit status named `cargo audit`",
    );

    let o = r.check_gh(&h, &checks("github-actions", "all green", None, &[]));
    expect(&o, 1, "FAIL required-checks", "policy: pass (run <n>)");

    let o = r.check_gh(
        &h,
        &checks("github-actions", "policy: fail (run 42)", None, &[]),
    );
    expect(&o, 1, "FAIL required-checks", "policy: pass (run <n>)");

    let o = r.check_gh(&h, &checks("some-app", pass, None, &[]));
    expect(&o, 1, "FAIL required-checks", "policy: pass (run <n>)");

    let o = r.check_gh(
        &h,
        &checks("github-actions", "policy: pass (run 42) x", None, &[]),
    );
    expect(&o, 1, "FAIL required-checks", "policy: pass (run <n>)");
}

/// The fake with the five required names passing, plus `extra` check runs for `cargo audit`
/// (each `(id, app, summary)`).
fn audit_runs(extra: &[(u64, &str, &str)]) -> String {
    let mut runs: Vec<serde_json::Value> = REQUIRED
        .iter()
        .enumerate()
        .map(|(i, n)| {
            serde_json::json!({"id": 100 + i, "name": n, "app": "github-actions", "summary": "policy: pass (run 1)"})
        })
        .collect();
    for (id, app, summary) in extra {
        runs.push(
            serde_json::json!({"id": id, "name": "cargo audit", "app": app, "summary": summary}),
        );
    }
    serde_json::json!({"comments": [], "checkRuns": runs, "statuses": []}).to_string()
}

/// `required-checks` judges each name by its newest check run (the highest id) only.
#[test]
fn required_checks_judge_the_newest_run_per_name() {
    let r = Repo::new();
    let h = r.green_a(|r| r.write("crates/a/src/lib.rs", "//\n"));
    let older_pass_newer_fail = audit_runs(&[(200, "github-actions", "policy: fail (run 7)\n")]);
    let o = r.check_gh(&h, &older_pass_newer_fail);
    expect(
        &o,
        1,
        "FAIL required-checks",
        "`cargo audit`: the newest check run (id 200",
    );

    let older_fail_newer_pass = audit_runs(&[(50, "github-actions", "policy: fail (run 7)")]);
    let o = r.check_gh(&h, &older_fail_newer_pass);
    expect(&o, 1, "PASS required-checks", "all 5 required checks");

    let newer_other_app = audit_runs(&[(200, "some-app", "policy: pass (run 7)")]);
    let o = r.check_gh(&h, &newer_other_app);
    expect(&o, 1, "FAIL required-checks", "app some-app");
}

/// The D59 switch is read at the merge base: a setup PR that itself adds the switch reading
/// `ended` and lists it as an owner path still sees D59 in force.
#[test]
fn the_d59_switch_is_read_at_the_merge_base() {
    let r = Repo::new();
    let text = order("claude", "infra-loop-pr99-x", "Setup row: 99\n");
    let h = r.green(SETUP_ORDER, &text, |r| {
        r.write("crates/a/src/lib.rs", "//\n");
        r.write("docs/harness/d59.json", r#"{"schema":1,"d59":"ended"}"#);
        r.write(
            "docs/harness/owner-paths.json",
            r#"{"schema":1,"paths":[{"path":"docs/harness/d59.json"}]}"#,
        );
    });
    let o = r.check_gh(&h, &fake(vec![record("codex", &h, "APPROVE")]));
    expect(
        &o,
        1,
        "FAIL review-record",
        "a setup PR under D59 counts only claude records",
    );
    let o = r.check_gh(&h, &fake(vec![record("claude", &h, "APPROVE")]));
    expect(&o, 1, "PASS review-record", "counted");
}

/// D76: a new file that compiles banned code fails, like an edit does. Before D76 the rule
/// skipped added files, so this exited 0.
#[test]
fn a_new_file_with_banned_code_fails_the_presence_rule() {
    let r = Repo::new();
    let h = r.green_a(|r| {
        r.write("crates/a/src/fresh.rs", GATED_HEAD_UNGATED);
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL banned-presence",
        "D23 sites in compiled code rise",
    );
}

/// D76 made `rendered-lock` and `read-path` advisory, and both used to catch some AGENTS.md
/// edits. The project's own owner-path list must keep AGENTS.md (and the other owner files)
/// under the hard `owner-paths` rule.
#[test]
fn the_project_owner_paths_still_cover_agents_charter_decisions_and_ci_d76() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/harness/owner-paths.json");
    let text = std::fs::read_to_string(&path).expect("read docs/harness/owner-paths.json");
    let v: serde_json::Value = serde_json::from_str(&text).expect("owner-paths.json is JSON");
    let listed: Vec<&str> = v["paths"]
        .as_array()
        .expect("paths array")
        .iter()
        .filter_map(|p| p["path"].as_str())
        .collect();
    for want in [
        "AGENTS.md",
        "CHARTER.md",
        "docs/integration/DECISIONS.md",
        ".github/**",
    ] {
        assert!(listed.contains(&want), "{want} missing from {listed:?}");
    }
}

#[test]
fn un_gating_a_banned_item_fails_the_presence_rule() {
    let r = Repo::new();
    let h = r.green_a(|r| {
        r.write("crates/a/src/gated.rs", GATED_HEAD_UNGATED);
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL banned-presence",
        "D23 sites in compiled code rise",
    );
}

#[test]
fn moving_a_banned_item_to_a_test_only_gate_fails_the_presence_rule() {
    let r = Repo::new();
    let h = r.green_a(|r| {
        r.write("crates/a/src/gated.rs", GATED_HEAD_TEST_ONLY);
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL banned-presence",
        "D23 sites in compiled code rise",
    );
}

#[test]
fn renaming_a_banned_item_does_not_bypass_the_presence_rule() {
    let r = Repo::new();
    let h = r.green_a(|r| {
        r.write("crates/a/src/gated.rs", GATED_HEAD_RENAMED);
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL banned-presence",
        "D23 sites in compiled code rise",
    );
}

#[test]
fn un_gating_benign_word_and_char_literal_items_passes() {
    for head in [GATED_HEAD_BENIGN, GATED_HEAD_PLAIN_WORD, GATED_HEAD_CHARLIT] {
        let r = Repo::new();
        let h = r.green_a(|r| {
            r.write("crates/a/src/gated.rs", head);
        });
        expect(
            &r.check(&h, &[]),
            0,
            "PASS banned-presence",
            "verdict: PASS",
        );
    }
}

#[test]
fn deleting_a_banned_gated_item_passes_the_presence_rule() {
    let r = Repo::new();
    let h = r.green_a(|r| {
        r.write("crates/a/src/gated.rs", GATED_HEAD_DELETED);
    });
    expect(
        &r.check(&h, &[]),
        0,
        "PASS banned-presence",
        "verdict: PASS",
    );
}

#[test]
fn moving_the_gate_to_the_parent_module_passes_the_presence_rule() {
    let r = Repo::new();
    let h = r.green_a(|r| {
        r.write("crates/a/src/gated.rs", GATED_HEAD_PARENT_GATE);
    });
    expect(
        &r.check(&h, &[]),
        0,
        "PASS banned-presence",
        "verdict: PASS",
    );
}

/// A function with one D25 site, assembled so this test file adds no ratchet site itself.
const D25_FN: &str = concat!("pub fn dense() {\n    value.mat", "mul(other);\n}\n");

#[test]
fn removing_a_gate_with_a_trailing_comment_fails_the_presence_rule() {
    let r = Repo::new().with_base(|r| {
        r.write(
            "crates/a/src/legacy.rs",
            &format!("#[cfg(any())] // note\n{D25_FN}"),
        );
    });
    let h = r.green_a(|r| {
        r.write("crates/a/src/legacy.rs", D25_FN);
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL banned-presence",
        "D25 sites in compiled code rise from 0 at the merge base to 1",
    );
}

#[test]
fn moving_a_gated_file_module_to_test_fails_the_presence_rule() {
    let r = Repo::new().with_base(|r| {
        r.write(
            "crates/a/src/lib.rs",
            "pub fn a() {}\n\n#[cfg(any())] // note\npub mod m;\n",
        );
        r.write("crates/a/src/m.rs", D25_FN);
    });
    let h = r.green_a(|r| {
        r.write(
            "crates/a/src/lib.rs",
            "pub fn a() {}\n\n#[cfg(test)]\npub mod m;\n",
        );
    });
    let o = r.check(&h, &[]);
    expect(
        &o,
        1,
        "FAIL banned-presence",
        "D25 sites in compiled code rise",
    );
    assert!(
        stdout(&o).contains("newly compiled module file(s) crates/a/src/m.rs"),
        "{}",
        stdout(&o)
    );
}

#[test]
fn moving_a_file_module_whose_item_is_gated_inside_passes_the_presence_rule() {
    let r = Repo::new().with_base(|r| {
        r.write(
            "crates/a/src/lib.rs",
            "pub fn a() {}\n\n#[cfg(any())] // note\npub mod m;\n",
        );
        r.write("crates/a/src/m.rs", &format!("#[cfg(any())]\n{D25_FN}"));
    });
    let h = r.green_a(|r| {
        r.write(
            "crates/a/src/lib.rs",
            "pub fn a() {}\n\n#[cfg(test)]\npub mod m;\n",
        );
    });
    expect(
        &r.check(&h, &[]),
        0,
        "PASS banned-presence",
        "verdict: PASS",
    );
}

#[test]
fn un_gating_a_float_function_in_the_d32_scope_fails_the_presence_rule() {
    let r = Repo::new();
    let h = r.green_b(|r| {
        r.write(
            "crates/uor-r4-discrete/src/gated_float.rs",
            GATED_FLOAT_HEAD,
        );
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL banned-presence",
        "D32 sites in compiled code rise",
    );
}

#[test]
fn keeping_a_float_function_gated_in_the_d32_scope_passes() {
    let r = Repo::new();
    let h = r.green_b(|r| {
        r.write(
            "crates/uor-r4-discrete/src/gated_float.rs",
            GATED_FLOAT_BASE,
        );
        r.write("crates/uor-r4-discrete/src/other.rs", "pub fn c() {}\n");
    });
    expect(
        &r.check(&h, &[]),
        0,
        "PASS banned-presence",
        "verdict: PASS",
    );
}

#[test]
fn un_gating_a_pub_crate_item_fails_the_presence_rule() {
    let r = Repo::new().with_base(|r| {
        r.write(
            "crates/a/src/legacy.rs",
            &format!(
                "#[cfg(any())]\npub(crate) fn helper() {{\n    {}\n}}\n",
                D25_FN
            ),
        );
    });
    let h = r.green_a(|r| {
        r.write(
            "crates/a/src/legacy.rs",
            &format!("pub(crate) fn helper() {{\n    {}\n}}\n", D25_FN),
        );
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL banned-presence",
        "D25 sites in compiled code rise",
    );
}

#[test]
fn an_inner_gate_is_respected() {
    let r = Repo::new().with_base(|r| {
        r.write(
            "crates/a/src/inner.rs",
            &format!("#![cfg(any())]\n{D25_FN}"),
        );
    });
    let h = r.green_a(|r| {
        r.write("crates/a/src/inner.rs", D25_FN);
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL banned-presence",
        "D25 sites in compiled code rise",
    );
}

#[test]
fn un_gating_an_half_float_in_the_d32_scope_fails() {
    let r = Repo::new().with_base(|r| {
        r.write(
            "crates/uor-r4-discrete/src/half.rs",
            "#[cfg(any())]\npub fn half(x: __half) -> __half {\n    x\n}\n",
        );
    });
    let h = r.green_b(|r| {
        r.write(
            "crates/uor-r4-discrete/src/half.rs",
            "pub fn half(x: __half) -> __half {\n    x\n}\n",
        );
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL banned-presence",
        "D32 sites in compiled code rise",
    );
}

#[test]
fn dense_helper_names_are_not_d25_sites() {
    let r = Repo::new().with_base(|r| {
        r.write(
            "crates/a/src/helpers.rs",
            "#[cfg(any())]\npub fn gemm_helper() {}\n",
        );
    });
    let h = r.green_a(|r| {
        r.write("crates/a/src/helpers.rs", "pub fn gemm_helper() {}\n");
    });
    expect(
        &r.check(&h, &[]),
        0,
        "PASS banned-presence",
        "verdict: PASS",
    );
}

#[test]
fn a_raw_string_gate_does_not_hide_the_next_item() {
    let r = Repo::new().with_base(|r| {
        r.write(
            "crates/a/src/raw.rs",
            "pub fn live() {\n    value.matmul(other);\n}\n\n#[cfg(any())]\npub const KERNEL: &str = r\"{ ;\";\n",
        );
    });
    let h = r.green_a(|r| {
        r.write(
            "crates/a/src/raw.rs",
            "pub fn live() {\n    value.matmul(other);\n}\n",
        );
    });
    expect(
        &r.check(&h, &[]),
        0,
        "PASS banned-presence",
        "verdict: PASS",
    );
}
