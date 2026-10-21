//! `uor-harness loop report` against temporary git repositories: a bare `origin` whose `main`
//! holds the real templates, and a branch that adds an ORDER. Every child process has every `GIT_*`
//! variable removed, and each repository is checked to sit inside its temporary directory.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const ORDER: &str = "docs/loop/tasks/item-a/claude-1/ORDER.md";
const REPORT: &str = "docs/loop/tasks/item-a/claude-1/REPORT.md";
const RECEIPTS: &str = "docs/loop/tasks/item-a/claude-1/receipts.jsonl";
const VALID: &str = "# REPORT\n\n## What did not work, first\n\nNothing.\n\n## What changed\n\nA.\n\n## Measurements\n\nNone.\n\n## Checks run at the head\n\n- tests\n\n## Skills used\n\nNone.\n\n## Still open\n\nNothing.\n\n## Plain summary\n\nDone.\n";

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB")
}

fn real(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn order_text(id: &str) -> String {
    format!("# ORDER: {id}, claude-1 (t)\n\nLab: claude. Item: [{id}](../../../../roadmap/items/{id}.md) (#1).\n")
}

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
    /// `main` (pushed to a bare origin) holds the templates; `feat` adds `orders`.
    fn new(orders: &[&str]) -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().canonicalize().unwrap();
        let root = base.join("repo");
        let origin = base.join("origin.git");
        assert!(root.starts_with(&base) && origin.starts_with(&base));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { _tmp: tmp, root };
        r.git(&["init", "-q", "--bare", origin.to_str().unwrap()]);
        r.git(&["init", "-q", "-b", "main"]);
        r.write(
            "docs/harness/labs.json",
            r#"{"labs": {"claude": {"vendor": "a", "client": "b", "headless": "c", "open_pr_limit": 2}}}"#,
        );
        for t in ["REPORT", "ORDER", "PICKUP"] {
            let rel = format!("docs/loop/templates/{t}.md");
            r.write(&rel, &real(&rel));
        }
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "base"]);
        r.git(&["remote", "add", "origin", origin.to_str().unwrap()]);
        r.git(&["push", "-q", "origin", "main"]);
        r.git(&["fetch", "-q", "origin"]);
        r.git(&["checkout", "-q", "-b", "feat"]);
        r.write("crates/x.rs", "// work\n");
        for o in orders {
            r.write(o, &order_text("item-a"));
        }
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "order"]);
        r
    }

    fn git(&self, args: &[&str]) -> String {
        let o = clean(&mut Command::new("git"))
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs");
        assert!(
            o.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.root.join(rel)).unwrap()
    }

    fn report(&self, lab: Option<&str>, args: &[&str]) -> (i32, String) {
        let mut cmd = Command::new(BIN);
        clean(&mut cmd);
        if let Some(l) = lab {
            cmd.env("UOR_LAB", l);
        }
        let o: Output = cmd
            .args(["--repo", self.root.to_str().unwrap(), "loop", "report"])
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("the binary runs");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
        (o.status.code().unwrap_or(-1), text)
    }
}

#[test]
fn a_report_missing_a_section_is_refused() {
    let r = Repo::new(&[ORDER]);
    r.write(REPORT, &VALID.replace("## Skills used\n\nNone.\n\n", ""));
    let (code, out) = r.report(Some("claude"), &["--check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL report"), "{out}");
    assert!(out.contains("Skills used"), "{out}");
}

#[test]
fn check_passes_a_complete_report_and_refuses_fill_lines_and_a_missing_file() {
    let r = Repo::new(&[ORDER]);
    let (code, out) = r.report(Some("claude"), &["--check"]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("FAIL report") && out.contains("missing"),
        "{out}"
    );

    r.write(REPORT, VALID);
    let (code, out) = r.report(Some("claude"), &["--check"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("PASS report"), "{out}");

    r.write(
        REPORT,
        &VALID.replace("A.\n", "A.\n<!-- fill: what changed -->\n"),
    );
    let (code, out) = r.report(Some("claude"), &["--check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL report"), "{out}");
    assert!(out.contains("left in `## What changed`"), "{out}");

    // Prose that names the marker is not a template line (the claude-17 REPORT does this).
    r.write(
        REPORT,
        &VALID.replace(
            "A.\n",
            "A.\nTemplate prompts are `<!-- fill: -->` lines, which `--check` refuses.\n  - It refuses a leftover `<!-- fill:` line.\n",
        ),
    );
    let (code, out) = r.report(Some("claude"), &["--check"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("PASS report"), "{out}");

    // A fill line inside a code fence is quoted text, not a template line.
    r.write(
        REPORT,
        &VALID.replace("A.\n", "A.\n```\n<!-- fill: x -->\n```\n"),
    );
    let (code, out) = r.report(Some("claude"), &["--check"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("PASS report"), "{out}");
}

#[test]
fn new_writes_the_template_once_and_the_fresh_template_fails_check() {
    let r = Repo::new(&[ORDER]);
    let (code, out) = r.report(Some("claude"), &["--new"]);
    assert_eq!(code, 0, "{out}");
    let written = r.read(REPORT);
    assert!(
        written.starts_with("# REPORT: item-a, claude-1\n"),
        "{written}"
    );
    assert!(written.contains("Lab: claude."), "{written}");
    assert!(!written.contains('{'), "{written}");

    r.write(REPORT, &(written.clone() + "edited\n"));
    let before = r.read(REPORT);
    let (code, out) = r.report(Some("claude"), &["--new"]);
    assert_eq!(code, 2, "{out}");
    assert_eq!(
        r.read(REPORT),
        before,
        "a second --new must not touch the file"
    );

    std::fs::write(r.root.join(REPORT), &written).unwrap();
    let (code, out) = r.report(Some("claude"), &["--check"]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("template line is left in `## What did not work, first`"),
        "{out}"
    );
}

/// The tree hash the harness records for a clean working tree at HEAD: `git hash-object` of the
/// length-prefixed `rev-parse HEAD` output and two empty length-prefixed sections.
fn clean_tree_hash(r: &Repo) -> String {
    use std::io::Write;
    let head_line = format!("{}\n", r.git(&["rev-parse", "HEAD"]));
    let mut bytes = (head_line.len() as u64).to_le_bytes().to_vec();
    bytes.extend_from_slice(head_line.as_bytes());
    bytes.extend_from_slice(&[0u8; 16]);
    let mut child = clean(&mut Command::new("git"))
        .args(["hash-object", "--stdin"])
        .current_dir(&r.root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&bytes).unwrap();
    let out = child.wait_with_output().unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

#[test]
fn receipts_skip_a_row_recorded_on_a_dirty_tree_at_head() {
    let r = Repo::new(&[ORDER]);
    let head = r.git(&["rev-parse", "HEAD"]);
    let tree = clean_tree_hash(&r);
    let row = |gate: &str, tree: &str| {
        format!(
            r#"{{"gate":"{gate}","verdict":"green","seconds":1.0,"commit":"{head}","tree":"{tree}","exit":0,"signal":null,"ts":1}}"#
        )
    };
    r.write(
        "local/claude/gates.jsonl",
        &(row("dirty", "not-the-clean-tree") + "\n" + &row("clean", &tree) + "\n"),
    );
    let (code, out) = r.report(Some("claude"), &["--receipts"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(r.read(RECEIPTS), row("clean", &tree) + "\n");

    r.write(
        "local/claude/gates.jsonl",
        &(row("dirty", "not-the-clean-tree") + "\n"),
    );
    let (code, out) = r.report(Some("claude"), &["--receipts"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("no gate results at HEAD"), "{out}");
}

#[test]
fn receipts_copy_the_latest_head_result_per_gate_sorted() {
    let r = Repo::new(&[ORDER]);
    let head = r.git(&["rev-parse", "HEAD"]);
    let old = r.git(&["rev-parse", "HEAD~1"]);
    let tree = clean_tree_hash(&r);
    let row = |gate: &str, commit: &str, verdict: &str, ts: u64| {
        format!(
            r#"{{"gate":"{gate}","verdict":"{verdict}","seconds":1.0,"commit":"{commit}","tree":"{tree}","exit":0,"signal":null,"ts":{ts}}}"#
        )
    };
    let lines = [
        row("zeta", &head, "red", 1),
        row("alpha", &old, "green", 2),
        row("zeta", &head, "green", 3),
        row("mid", &head, "green", 4),
        "not json".to_string(),
    ];
    r.write("local/claude/gates.jsonl", &(lines.join("\n") + "\n"));
    let (code, out) = r.report(Some("claude"), &["--receipts"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        r.read(RECEIPTS),
        format!(
            "{}\n{}\n",
            row("mid", &head, "green", 4),
            row("zeta", &head, "green", 3)
        )
    );

    r.write(
        "local/claude/gates.jsonl",
        &(row("alpha", &old, "green", 2) + "\n"),
    );
    let (code, out) = r.report(Some("claude"), &["--receipts"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("no gate results at HEAD"), "{out}");

    let (code, out) = r.report(None, &["--receipts"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("set UOR_LAB or pass --lab"), "{out}");
}

#[test]
fn the_order_is_found_from_the_branch_or_named() {
    let none = Repo::new(&[]);
    let (code, out) = none.report(Some("claude"), &["--check"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("adds no"), "{out}");
    none.write(
        "docs/loop/tasks/other/claude-2/ORDER.md",
        &order_text("item-b"),
    );
    none.write("docs/loop/tasks/other/claude-2/REPORT.md", VALID);
    let (code, out) = none.report(
        Some("claude"),
        &[
            "--check",
            "--order",
            "docs/loop/tasks/other/claude-2/ORDER.md",
        ],
    );
    assert_eq!(code, 0, "{out}");

    let two = Repo::new(&[ORDER, "docs/loop/tasks/item-b/claude-2/ORDER.md"]);
    let (code, out) = two.report(Some("claude"), &["--check"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("2 ORDERs") && out.contains(ORDER), "{out}");
    two.write(REPORT, VALID);
    let (code, out) = two.report(Some("claude"), &["--check", "--order", ORDER]);
    assert_eq!(code, 0, "{out}");
}

#[test]
fn the_real_report_template_has_the_seven_sections_in_order() {
    let t = real("docs/loop/templates/REPORT.md");
    let headings: Vec<&str> = t.lines().filter(|l| l.starts_with("## ")).collect();
    assert_eq!(
        headings,
        [
            "## What did not work, first",
            "## What changed",
            "## Measurements",
            "## Checks run at the head",
            "## Skills used",
            "## Still open",
            "## Plain summary",
        ]
    );
    assert_eq!(t.lines().filter(|l| l.starts_with("<!-- fill:")).count(), 7);
    let filled: String = t
        .lines()
        .map(|l| {
            if l.starts_with("<!-- fill:") {
                "Text."
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        uor_r4_harness::loopprcheck::report_problems(&filled),
        Vec::<String>::new()
    );
    assert!(real("docs/loop/templates/ORDER.md").len() < 2048);
    for t in ["REPORT", "ORDER", "PICKUP"] {
        let first = real(&format!("docs/loop/templates/{t}.md"));
        let first = first.lines().next().unwrap();
        assert!(
            first.starts_with("<!-- ") && first.ends_with(" -->"),
            "{t}: {first}"
        );
    }
}
