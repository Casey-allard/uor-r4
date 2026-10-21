//! `uor-harness loop render` on fixture repositories (loop PR 3): determinism, `--check` at the
//! stamp, the planted fault (a hand-edited STATE fails), and commit-pinned inputs. Every child
//! process has every `GIT_*` variable removed, and each repository is checked to sit inside its
//! own temporary directory before anything writes to it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd
}

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
    fn git(&self, args: &[&str]) -> String {
        let o = clean(&mut Command::new("git"))
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

    fn render(&self, args: &[&str]) -> Output {
        self.render_env(args, &[])
    }

    /// `loop render` with extra environment set after every `GIT_*` variable is removed.
    fn render_env(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(BIN);
        clean(&mut cmd)
            .args(["--repo", self.root.to_str().unwrap(), "loop", "render"])
            .args(args)
            .env("UOR_LAB", "claude");
        for (k, v) in env {
            cmd.env(k, v);
        }
        cmd.output().unwrap()
    }

    /// A stand-in for cargo (`CARGO`) that exits with `code`, so no test reaches a real cargo.
    fn fake_cargo(&self, code: i32) -> String {
        use std::os::unix::fs::PermissionsExt;
        let p = self._tmp.path().join(format!("fake-cargo-{code}"));
        std::fs::write(&p, format!("#!/bin/sh\nexit {code}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p.to_str().unwrap().to_string()
    }

    /// `loop render` with `--out <dir>` first, so nothing is written into the repository.
    fn render_out(&self, dir: &Path, args: &[&str]) -> Output {
        let mut a = vec!["--out", dir.to_str().unwrap()];
        a.extend_from_slice(args);
        self.render(&a)
    }

    fn out_dir(&self, name: &str) -> PathBuf {
        let d = self._tmp.path().join(name);
        assert!(d.starts_with(self._tmp.path()));
        d
    }
}

const LABS: &str = r#"{"schema":1,"labs":{"kimi":{"vendor":"Moonshot","client":"Kimi","headless":"kimi -p","open_pr_limit":3},"claude":{"vendor":"Anthropic","client":"Claude Code","headless":"claude -p","open_pr_limit":3}}}"#;

fn repo() -> Repo {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("repo");
    std::fs::create_dir_all(&root).unwrap();
    assert!(root.starts_with(tmp.path()));
    let r = Repo { _tmp: tmp, root };
    r.git(&["init", "-q", "-b", "main"]);
    r.write("docs/harness/labs.json", LABS);
    r.write(
        "docs/harness/milestones.json",
        r#"{"schema":1,"milestones":{"m2":{"id":"M2","title":"Two","status":"blocked-on-owner","headline":null,"training_hold":true},"m1":{"id":"M1","title":"One","status":"in-progress","headline":{"label":"reply acceptable","current":37,"target":116,"of":232,"unit":"replies"},"training_hold":false}}}"#,
    );
    r.write(
        "docs/roadmap/items/b.md",
        "---\nid: b\nstatus: open\n---\n\nbody\n",
    );
    r.write(
        "docs/roadmap/items/a.md",
        "---\nid: a\nstatus: held\n---\n\nbody\n",
    );
    r.write(
        "docs/research/notebook/line-a.md",
        "# Notebook\n\n| Date | What ran | Number | Bar | Verdict | Artifact |\n|---|---|---|---|---|---|\n| 2026-10-13 | first | 1 | 2 | MISS | x |\n| 2026-10-14 | second | 3 | 2 | PASS | y |\n",
    );
    r.write(
        "docs/research/notebook/line-b.md",
        "# Notebook\n\n| Date | What ran | Number | Bar | Verdict | Artifact |\n|---|---|---|---|---|---|\n| | *No result rows yet.* | | | | |\n",
    );
    r.write(
        "docs/mechanisms/README.md",
        "# Board\n\n## ACTIVE: on path (1)\n\n| Mechanism | x |\n|---|---|\n| [Zeta](z.md) | x |\n\n## PARKED: closed (1)\n\n| Mechanism | x |\n|---|---|\n| [Alpha](a.md) | x |\n\n## Other\n\n| [Ignored](i.md) | x |\n",
    );
    r.commit("fixture");
    r
}

fn state(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("STATE.md")).unwrap()
}

#[test]
fn two_renders_of_one_commit_are_byte_identical() {
    let r = repo();
    let (a, b) = (r.out_dir("a"), r.out_dir("b"));
    assert!(r.render(&["--out", a.to_str().unwrap()]).status.success());
    assert!(r.render(&["--out", b.to_str().unwrap()]).status.success());
    let s = state(&a);
    assert_eq!(s, state(&b));
    let head = r.git(&["rev-parse", "HEAD"]);
    assert_eq!(
        s.lines().next().unwrap(),
        format!("<!-- rendered by uor-harness loop render from {head}; do not edit -->")
    );
    // Every section, in order, with sorted rows.
    let order = [
        "## Milestone bars and holds",
        "| M1 One | in-progress | reply acceptable: 37 of target 116 (out of 232 replies) | released |",
        "| M2 Two | blocked-on-owner | none | HELD |",
        "## Roadmap items by status",
        "| held | 1 |",
        "| open | 1 |",
        "## Newest notebook row per research line",
        "| line-a | 2026-10-14 | 3 | 2 | PASS |",
        "| line-b | no rows yet | | | |",
        "## Mechanisms",
        "- ACTIVE: Zeta",
        "- PARKED: Alpha",
        "## Lab registry",
        "| claude | Anthropic | Claude Code | 3 |",
        "| kimi | Moonshot | Kimi | 3 |",
    ];
    let mut at = 0;
    for want in order {
        let pos = s[at..]
            .find(want)
            .unwrap_or_else(|| panic!("{want:?} in\n{s}"));
        at += pos + want.len();
    }
    assert!(!s.contains("Ignored"), "{s}");
}

#[test]
fn check_passes_on_a_fresh_render() {
    let r = repo();
    let d = r.out_dir("v");
    assert!(r.render_out(&d, &[]).status.success());
    assert!(d.join("STATE.md").exists());
    assert!(
        !r.root.join("STATE.md").exists(),
        "--out must not write in place"
    );
    let o = r.render_out(&d, &["--check"]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
}

/// The planted fault: a hand edit to STATE.md must turn `--check` red.
#[test]
fn a_hand_edited_state_fails_check() {
    let r = repo();
    let d = r.out_dir("v");
    assert!(r.render_out(&d, &[]).status.success());
    let p = d.join("STATE.md");
    let edited = std::fs::read_to_string(&p)
        .unwrap()
        .replace("| HELD |", "| released |");
    std::fs::write(&p, edited).unwrap();
    let o = r.render_out(&d, &["--check"]);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("first at line"), "{out}");
    assert!(out.contains("| M2 Two |"), "{out}");
}

#[test]
fn a_garbled_or_missing_stamp_exits_2() {
    let r = repo();
    let d = r.out_dir("v");
    let o = r.render_out(&d, &["--check"]);
    assert_eq!(o.status.code(), Some(2), "missing file: {o:?}");
    assert!(r.render_out(&d, &[]).status.success());
    let p = d.join("STATE.md");
    let text = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, text.replacen("from ", "from x", 1)).unwrap();
    let o = r.render_out(&d, &["--check"]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("stamp"),
        "{o:?}"
    );
}

#[test]
fn a_later_input_change_leaves_the_earlier_render_alone() {
    let r = repo();
    let first = r.git(&["rev-parse", "HEAD"]);
    let before = r.out_dir("before");
    assert!(r
        .render(&["--out", before.to_str().unwrap()])
        .status
        .success());
    r.write(
        "docs/harness/labs.json",
        &LABS.replace("Moonshot", "Changed"),
    );
    r.commit("change a lab");
    // An uncommitted edit is not an input either.
    r.write("docs/roadmap/items/c.md", "---\nid: c\nstatus: open\n---\n");
    let again = r.out_dir("again");
    let o = r.render(&["--out", again.to_str().unwrap(), "--at", &first]);
    assert!(o.status.success(), "{o:?}");
    assert_eq!(state(&before), state(&again));
    let o = r.render(&["--check", "--out", before.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let head = r.out_dir("head");
    assert!(r
        .render(&["--out", head.to_str().unwrap()])
        .status
        .success());
    assert!(state(&head).contains("| kimi | Changed |"));
    assert!(
        !state(&head).contains("| open | 2 |"),
        "working tree was read"
    );
}

#[test]
fn at_renders_an_older_commit() {
    let r = repo();
    let first = r.git(&["rev-parse", "HEAD"]);
    r.write("docs/roadmap/items/c.md", "---\nid: c\nstatus: open\n---\n");
    r.commit("add an item");
    let old = r.out_dir("old");
    assert!(r
        .render(&["--out", old.to_str().unwrap(), "--at", "HEAD~1"])
        .status
        .success());
    let s = state(&old);
    assert!(s.starts_with(&format!(
        "<!-- rendered by uor-harness loop render from {first};"
    )));
    assert!(s.contains("| open | 1 |"), "{s}");
    let o = r.render(&["--at", "no-such-ref", "--out", old.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
}

fn read(dir: &Path, rel: &str) -> String {
    std::fs::read_to_string(dir.join(rel)).unwrap()
}

fn item(
    id: &str,
    status: &str,
    open_to: &str,
    deps: &str,
    extra_front: &str,
    extra_body: &str,
) -> String {
    format!(
        "---\nid: {id}\ntitle: Title {id}\nstatus: {status}\nopen_to: {open_to}\n{extra_front}depends_on: [{deps}]\n---\n\n## Why it matters\n\nx\n\n## Done when\n\nFirst {id} sentence. Second sentence.\nStill the first paragraph.\n\nA later paragraph.\n{extra_body}\n## Sources\n\n- y\n"
    )
}

/// Items in every status, written in an order that is not the rendered order.
fn items(r: &Repo) {
    r.write(
        "docs/roadmap/items/a.md",
        &item("a", "held", "lab", "", "", ""),
    );
    r.write(
        "docs/roadmap/items/b.md",
        &item("b", "ready", "anyone", "a, c", "", ""),
    );
    r.write(
        "docs/roadmap/items/c.md",
        &item("c", "decision", "lab", "", "", ""),
    );
    r.write(
        "docs/roadmap/items/d.md",
        &item("d", "held", "owner", "", "", ""),
    );
    r.write(
        "docs/roadmap/items/e.md",
        &item("e", "done", "lab", "", "", ""),
    );
    r.write(
        "docs/roadmap/items/f.md",
        &item("f", "ready", "lab", "", "", ""),
    );
    r.commit("items");
}

fn in_order(s: &str, wants: &[&str]) {
    let mut at = 0;
    for want in wants {
        let pos = s[at..]
            .find(want)
            .unwrap_or_else(|| panic!("{want:?} after byte {at} in\n{s}"));
        at += pos + want.len();
    }
}

#[test]
fn handoff_lists_open_items_by_status_with_not_recorded_for_absent_fields() {
    let r = repo();
    items(&r);
    let d = r.out_dir("h");
    assert!(r.render(&["--out", d.to_str().unwrap()]).status.success());
    let s = read(&d, "HANDOFF.md");
    let head = r.git(&["rev-parse", "HEAD"]);
    assert_eq!(
        s.lines().next().unwrap(),
        format!("<!-- rendered by uor-harness loop render from {head}; do not edit -->")
    );
    in_order(
        &s,
        &[
            "## Open items",
            "### ready (2)",
            "| b | Title b | First b sentence. | not recorded | not recorded | ready; after a, c |",
            "| f | Title f | First f sentence. | not recorded | not recorded | ready |",
            "## Blocked",
            "| c | Title c | decision | owner |",
            "| a | Title a | held | lab |",
            "| d | Title d | held | owner |",
            "## Not doing",
            "none recorded",
        ],
    );
    assert!(!s.contains("| e |"), "a done item is not open:\n{s}");
    assert!(!s.contains("Second sentence"), "{s}");
}

#[test]
fn handoff_shows_cancel_if_and_size_when_present() {
    let r = repo();
    items(&r);
    r.write(
        "docs/roadmap/items/f.md",
        &item(
            "f",
            "ready",
            "lab",
            "",
            "size: M\n",
            "\n## Cancel if\n\nA renderer already exists. More.\n\n",
        ),
    );
    r.commit("cancel and size");
    let d = r.out_dir("h");
    assert!(r.render(&["--out", d.to_str().unwrap()]).status.success());
    let s = read(&d, "HANDOFF.md");
    assert!(
        s.contains("| f | Title f | First f sentence. | A renderer already exists. | M | ready |"),
        "{s}"
    );
}

#[test]
fn lessons_list_id_bearing_files_by_id_and_the_rest_under_the_no_id_heading() {
    let r = repo();
    let m = "docs/harness/mistakes";
    r.write(&format!("{m}/README.md"), "# Registry readme\n");
    r.write(
        &format!("{m}/a-late.md"),
        "---\nid: L-010\ntitle: Ten from front\n---\n\n# Ten heading\n",
    );
    r.write(
        &format!("{m}/b-early.md"),
        "---\nid: L-002\ntitle: Two from front\n---\n\nbody\n",
    );
    r.write(
        &format!("{m}/d-kebab.md"),
        "---\nid: d-kebab\ntitle: Kebab id\n---\n\nbody\n",
    );
    r.write(&format!("{m}/c-bare.md"), "no frontmatter\n");
    r.commit("mistakes");
    let d = r.out_dir("l");
    assert!(r.render(&["--out", d.to_str().unwrap()]).status.success());
    let s = read(&d, "docs/harness/LESSONS.md");
    in_order(
        &s,
        &[
            "## By lesson id",
            "- **L-002**: Two from front ([`b-early.md`](mistakes/b-early.md))",
            "- **L-010**: Ten heading ([`a-late.md`](mistakes/a-late.md))",
            "## Without a lesson id yet (assigned in loop PR 8)",
            "- c-bare ([`c-bare.md`](mistakes/c-bare.md))",
            "- Kebab id ([`d-kebab.md`](mistakes/d-kebab.md))",
        ],
    );
    assert!(!s.contains("Registry readme"), "{s}");
    assert_eq!(s.matches("**L-").count(), 2, "no id is invented:\n{s}");
}

#[test]
fn a_hand_edited_handoff_fails_check() {
    let r = repo();
    items(&r);
    let d = r.out_dir("v");
    assert!(r.render_out(&d, &[]).status.success());
    assert_eq!(r.render_out(&d, &["--check"]).status.code(), Some(0));
    let p = d.join("HANDOFF.md");
    let edited = std::fs::read_to_string(&p).unwrap().replace(
        "| d | Title d | held | owner |",
        "| d | Title d | held | lab |",
    );
    assert_ne!(edited, std::fs::read_to_string(&p).unwrap());
    std::fs::write(&p, edited).unwrap();
    let o = r.render_out(&d, &["--check"]);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("HANDOFF.md: differs"), "{out}");
}

#[test]
fn a_hand_edited_lessons_fails_check() {
    let r = repo();
    r.write(
        "docs/harness/mistakes/x.md",
        "---\nid: L-001\ntitle: One\n---\n",
    );
    r.commit("a lesson");
    let d = r.out_dir("v");
    assert!(r.render_out(&d, &[]).status.success());
    assert_eq!(r.render_out(&d, &["--check"]).status.code(), Some(0));
    let p = d.join("docs/harness/LESSONS.md");
    let edited = std::fs::read_to_string(&p)
        .unwrap()
        .replace("**L-001**: One", "**L-001**: Two");
    std::fs::write(&p, edited).unwrap();
    let o = r.render_out(&d, &["--check"]);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("docs/harness/LESSONS.md: differs"), "{out}");
}

const AGENTS_WITH_MARKERS: &str = "# Agents\n\nBefore.\n\n<!-- loop:lessons-top-ten:start -->\nstale text\n<!-- loop:lessons-top-ten:end -->\n\nAfter.\n";

fn lesson_files(r: &Repo) {
    r.write(
        "docs/harness/mistakes/x.md",
        "---\nid: L-001\ntitle: One\n---\n\n# One heading\n",
    );
}

#[test]
fn agents_block_is_rendered_between_markers_and_a_hand_edit_fails_check() {
    let r = repo();
    lesson_files(&r);
    r.write("AGENTS.md", AGENTS_WITH_MARKERS);
    r.commit("agents with markers");
    let d = r.out_dir("v");
    let o = r.render_out(&d, &[]);
    assert!(o.status.success(), "{o:?}");
    let s = read(&d, "AGENTS.md");
    assert!(s.starts_with("# Agents\n\nBefore.\n\n<!-- loop:lessons-top-ten:start -->\n<!-- rendered by uor-harness loop render from "), "{s}");
    assert!(s.contains("- **L-001**: One heading ([`x.md`](docs/harness/mistakes/x.md))\n<!-- loop:lessons-top-ten:end -->\n\nAfter.\n"), "{s}");
    assert!(!s.contains("stale text"), "{s}");
    assert_eq!(r.render_out(&d, &["--check"]).status.code(), Some(0));
    // Text outside the markers is not compared.
    let p = d.join("AGENTS.md");
    std::fs::write(&p, s.replace("Before.", "Changed outside.")).unwrap();
    assert_eq!(r.render_out(&d, &["--check"]).status.code(), Some(0));
    // The planted fault: a hand edit inside the markers.
    std::fs::write(&p, s.replace("**L-001**: One heading", "**L-001**: Edited")).unwrap();
    let o = r.render_out(&d, &["--check"]);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("AGENTS.md top-ten block: differs"), "{out}");
}

#[test]
fn agents_without_markers_is_left_byte_unchanged_with_a_note() {
    let r = repo();
    let plain = "# Agents\n\nNo markers here.\n";
    r.write("AGENTS.md", plain);
    r.commit("plain agents");
    let d = r.out_dir("v");
    let o = r.render_out(&d, &[]);
    assert!(o.status.success(), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        out.contains("AGENTS.md: no markers; top-ten block not rendered"),
        "{out}"
    );
    assert!(!d.join("AGENTS.md").exists());
    assert_eq!(
        std::fs::read_to_string(r.root.join("AGENTS.md")).unwrap(),
        plain
    );
    let o = r.render_out(&d, &["--check"]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("no markers"),
        "{o:?}"
    );
}

#[test]
fn held_and_decision_items_appear_only_in_the_blocked_table() {
    let r = repo();
    items(&r);
    let d = r.out_dir("h");
    assert!(r.render_out(&d, &[]).status.success());
    let s = read(&d, "HANDOFF.md");
    assert!(!s.contains("### held"), "{s}");
    assert!(!s.contains("### decision"), "{s}");
    let (open, blocked) = s.split_once("## Blocked").unwrap();
    for id in ["a", "c", "d"] {
        assert!(
            !open.contains(&format!("| {id} |")),
            "{id} listed as open:\n{open}"
        );
        assert_eq!(
            s.matches(&format!("| {id} | Title {id} |")).count(),
            1,
            "{s}"
        );
        assert!(blocked.contains(&format!("| {id} | Title {id} |")), "{s}");
    }
    assert!(
        open.contains("| b | Title b |") && open.contains("| f | Title f |"),
        "{s}"
    );
}

#[test]
fn in_place_render_and_check_refuse_the_roadmap_without_xtask() {
    let r = repo();
    let none = [("CARGO", "/nonexistent/uor-loop-render-test/cargo")];
    let o = r.render_env(&[], &none);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("docs/roadmap/README.md: REFUSED"), "{out}");
    let o = r.render_env(&["--check"], &none);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("docs/roadmap/README.md: REFUSED"), "{out}");
}

#[test]
fn in_place_render_keeps_uncommitted_bytes_outside_the_markers() {
    let r = repo();
    lesson_files(&r);
    r.write("AGENTS.md", AGENTS_WITH_MARKERS);
    r.commit("agents with markers");
    let disk = AGENTS_WITH_MARKERS.replace("After.\n", "After.\nUncommitted line.\n");
    r.write("AGENTS.md", &disk);
    let ok = r.fake_cargo(0);
    let o = r.render_env(&[], &[("CARGO", &ok)]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let s = read(&r.root, "AGENTS.md");
    assert!(
        s.ends_with("<!-- loop:lessons-top-ten:end -->\n\nAfter.\nUncommitted line.\n"),
        "{s}"
    );
    assert!(
        s.starts_with("# Agents\n\nBefore.\n\n<!-- loop:lessons-top-ten:start -->\n"),
        "{s}"
    );
    assert!(!s.contains("stale text") && s.contains("**L-001**"), "{s}");
    // Markers missing on disk while the commit has them: refused, nothing written.
    let gone = disk.replace("<!-- loop:lessons-top-ten:end -->\n", "");
    r.write("AGENTS.md", &gone);
    std::fs::remove_file(r.root.join("STATE.md")).unwrap();
    let o = r.render_env(&[], &[("CARGO", &ok)]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert_eq!(read(&r.root, "AGENTS.md"), gone);
    assert!(!r.root.join("STATE.md").exists());
}

#[test]
fn a_missing_marker_on_disk_fails_check() {
    let r = repo();
    lesson_files(&r);
    r.write("AGENTS.md", AGENTS_WITH_MARKERS);
    r.commit("agents with markers");
    let d = r.out_dir("v");
    assert!(r.render_out(&d, &[]).status.success());
    let s = read(&d, "AGENTS.md");
    let p = d.join("AGENTS.md");
    std::fs::write(&p, s.replace("<!-- loop:lessons-top-ten:end -->", "")).unwrap();
    let o = r.render_out(&d, &["--check"]);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        out.contains("AGENTS.md: markers missing or malformed"),
        "{out}"
    );
    let edited = s
        .replace("**L-001**: One heading", "**L-001**: Edited")
        .replace("<!-- loop:lessons-top-ten:start -->", "");
    std::fs::write(&p, edited).unwrap();
    assert_eq!(r.render_out(&d, &["--check"]).status.code(), Some(1));
}

#[test]
fn a_duplicate_start_marker_in_the_commit_is_refused() {
    let r = repo();
    lesson_files(&r);
    let dup = AGENTS_WITH_MARKERS.replace(
        "Before.\n",
        "Before.\n<!-- loop:lessons-top-ten:start -->\n",
    );
    r.write("AGENTS.md", &dup);
    r.commit("duplicate start marker");
    let d = r.out_dir("v");
    let o = r.render_out(&d, &[]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(!d.join("AGENTS.md").exists());
    assert_eq!(r.render_out(&d, &["--check"]).status.code(), Some(2));
}

#[test]
fn xtask_exit_1_is_refused_in_render_and_a_difference_only_in_check() {
    let r = repo();
    let (ok, one, two) = (r.fake_cargo(0), r.fake_cargo(1), r.fake_cargo(2));
    // xtask exits 1 for an item-validation error as well, so render cannot call it a difference.
    let o = r.render_env(&[], &[("CARGO", &one)]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stdout).contains("docs/roadmap/README.md: REFUSED"));
    assert_eq!(r.render_env(&[], &[("CARGO", &ok)]).status.code(), Some(0));
    let o = r.render_env(&["--check"], &[("CARGO", &one)]);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stdout).contains("docs/roadmap/README.md: differs"));
    let o = r.render_env(&["--check"], &[("CARGO", &two)]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stdout).contains("docs/roadmap/README.md: REFUSED"));
}

#[test]
fn at_an_older_commit_skips_the_roadmap_view() {
    let r = repo();
    r.write(
        "docs/roadmap/items/z.md",
        "---\nid: z\nstatus: open\n---\n\nbody\n",
    );
    r.commit("second");
    let none = [("CARGO", "/nonexistent/uor-loop-render-test/cargo")];
    let o = r.render_env(&["--at", "HEAD~1"], &none);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        out.contains("docs/roadmap/README.md: skipped with --at; xtask renders the working tree"),
        "{out}"
    );
    let o = r.render_env(&["--check"], &none);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stdout).contains("docs/roadmap/README.md: skipped"));
}

#[test]
fn a_git_dir_in_the_environment_does_not_redirect_the_render() {
    let r = repo();
    let other = repo();
    other.write("extra.md", "other\n");
    let other_head = other.commit("other");
    let head = r.git(&["rev-parse", "HEAD"]);
    assert_ne!(head, other_head);
    let d = r.out_dir("v");
    let git_dir = other.root.join(".git");
    let o = r.render_env(
        &["--out", d.to_str().unwrap()],
        &[("GIT_DIR", git_dir.to_str().unwrap())],
    );
    assert!(o.status.success(), "{o:?}");
    let s = read(&d, "STATE.md");
    assert!(s.lines().next().unwrap().contains(&head), "{s}");
    assert!(!s.contains(&other_head), "{s}");
}

/// Render to a scratch `--out` (cargo pointed at nothing) and expect a refusal, exit 2, whose
/// message contains every one of `needles`.
fn refused(r: &Repo, needles: &[&str]) {
    let d = r.out_dir("refused");
    let o = r.render_env(
        &["--out", d.to_str().unwrap()],
        &[("CARGO", "/nonexistent/cargo")],
    );
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    let err = String::from_utf8_lossy(&o.stderr);
    for n in needles {
        assert!(err.contains(n), "{n:?} in {err}");
    }
    assert!(!d.join("STATE.md").exists(), "a refusal writes nothing");
}

#[test]
fn frontmatter_without_a_closing_line_is_refused() {
    let r = repo();
    r.write(
        "docs/roadmap/items/z.md",
        "---\nid: z\nstatus: open\n\n## Done when\n\nNever closed.\n",
    );
    r.commit("unclosed frontmatter");
    refused(&r, &["docs/roadmap/items/z.md", "no closing `---`"]);
}

#[test]
fn a_block_list_depends_on_is_refused() {
    let r = repo();
    r.write(
        "docs/roadmap/items/z.md",
        "---\nid: z\nstatus: open\ndepends_on:\n  - a\n  - b\n---\n\nbody\n",
    );
    r.commit("block list");
    refused(
        &r,
        &["docs/roadmap/items/z.md: depends_on must be an inline [a, b] list"],
    );
}

#[test]
fn a_duplicate_or_partial_notebook_table_is_refused_and_subfolders_are_not_read() {
    let table = "| Date | What ran | Number | Bar | Verdict |\n|---|---|---|---|---|\n| 2026-10-14 | x | 1 | 2 | PASS |\n";
    // A notebook in a subfolder is not a line: it is never read, so its missing table is fine.
    let r = repo();
    r.write("docs/research/notebook/sub/deep.md", "# no table here\n");
    r.commit("subfolder");
    let d = r.out_dir("v");
    let o = r.render_out(&d, &[]);
    assert!(o.status.success(), "{o:?}");
    assert!(!state(&d).contains("deep"), "{}", state(&d));

    let r = repo();
    r.write(
        "docs/research/notebook/two.md",
        &format!("{table}\n{table}"),
    );
    r.commit("two tables");
    refused(
        &r,
        &[
            "docs/research/notebook/two.md",
            "more than one results table",
        ],
    );

    let r = repo();
    r.write(
        "docs/research/notebook/part.md",
        "| Date | What ran | Number | Bar |\n|---|---|---|---|\n| 2026-10-14 | x | 1 | 2 |\n",
    );
    r.commit("no verdict column");
    refused(
        &r,
        &["docs/research/notebook/part.md", "no `Verdict` column"],
    );
}

#[test]
fn a_stray_board_row_or_an_empty_board_is_refused() {
    let r = repo();
    r.write(
        "docs/mechanisms/README.md",
        "# Board\n\n## ACTIVE: on path (1)\n\n| Mechanism | x |\n|---|---|\n| [Zeta](z.md) | x |\n| Stray | x |\n",
    );
    r.commit("stray row");
    refused(&r, &["docs/mechanisms/README.md", "| Stray | x |"]);

    let r = repo();
    r.write(
        "docs/mechanisms/README.md",
        "# Board\n\n## ACTIVE: on path (0)\n\n| Mechanism | x |\n|---|---|\n",
    );
    r.commit("empty board");
    refused(
        &r,
        &["docs/mechanisms/README.md", "no mechanism rows found"],
    );
}

#[test]
fn malformed_or_duplicate_lesson_ids_are_refused_and_kebab_ids_are_no_id() {
    for bad in ["l-012", "L-12a", "L-1"] {
        let r = repo();
        r.write(
            "docs/harness/mistakes/bad.md",
            &format!("---\nid: {bad}\n---\n\n# Bad\n"),
        );
        r.commit("bad id");
        refused(&r, &["docs/harness/mistakes/bad.md", bad]);
    }

    let r = repo();
    r.write(
        "docs/harness/mistakes/one.md",
        "---\nid: L-001\n---\n\n# One\n",
    );
    r.write(
        "docs/harness/mistakes/two.md",
        "---\nid: L-001\n---\n\n# Two\n",
    );
    r.commit("duplicate id");
    refused(
        &r,
        &[
            "docs/harness/mistakes/one.md",
            "docs/harness/mistakes/two.md",
            "L-001",
        ],
    );

    let r = repo();
    r.write(
        "docs/harness/mistakes/kebab.md",
        "---\nid: archive-is-not-deletion\n---\n\n# Kebab\n",
    );
    r.commit("kebab id");
    let d = r.out_dir("v");
    assert!(r.render_out(&d, &[]).status.success());
    let l = read(&d, "docs/harness/LESSONS.md");
    let no_id = l.find("Kebab").unwrap_or_else(|| panic!("{l}"));
    assert!(l[..no_id].contains("Without a lesson id"), "{l}");
}

#[test]
fn a_frontmatter_comment_is_never_the_lesson_title() {
    let r = repo();
    r.write(
        "docs/harness/mistakes/c.md",
        "---\n# a yaml comment\nid: L-001\ntitle: Real title\n---\n\nNo heading in the body.\n",
    );
    r.commit("comment in frontmatter");
    let d = r.out_dir("v");
    assert!(r.render_out(&d, &[]).status.success());
    let l = read(&d, "docs/harness/LESSONS.md");
    assert!(l.contains("Real title"), "{l}");
    assert!(!l.contains("a yaml comment"), "{l}");
}

#[test]
fn milestones_sort_by_number() {
    let r = repo();
    r.write(
        "docs/harness/milestones.json",
        r#"{"schema":1,"milestones":{"m10":{"id":"M10","title":"Ten","status":"s"},"x":{"id":"X","title":"Other","status":"s"},"m2":{"id":"M2","title":"Two","status":"s"}}}"#,
    );
    r.commit("m10");
    let d = r.out_dir("v");
    assert!(r.render_out(&d, &[]).status.success());
    in_order(&state(&d), &["| M2 Two |", "| M10 Ten |", "| X Other |"]);
}

#[test]
fn a_stamp_moved_to_an_older_commit_with_other_inputs_fails_check() {
    let r = repo();
    let old = r.git(&["rev-parse", "HEAD"]);
    r.write(
        "docs/harness/labs.json",
        &LABS.replace("Moonshot", "Changed"),
    );
    let head = r.commit("change a lab");
    let d = r.out_dir("v");
    assert!(r.render_out(&d, &[]).status.success());
    let p = d.join("STATE.md");
    let text = std::fs::read_to_string(&p).unwrap();
    assert!(text.contains(&head));
    std::fs::write(&p, text.replacen(&head, &old, 1)).unwrap();
    let o = r.render_env(
        &["--check", "--out", d.to_str().unwrap()],
        &[("CARGO", "/nonexistent/cargo")],
    );
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("STATE.md: differs"),
        "{o:?}"
    );
}

#[test]
fn check_with_at_exits_2() {
    let r = repo();
    let d = r.out_dir("v");
    assert!(r.render_out(&d, &[]).status.success());
    let o = r.render_env(
        &["--check", "--out", d.to_str().unwrap(), "--at", "HEAD"],
        &[("CARGO", "/nonexistent/cargo")],
    );
    assert_eq!(o.status.code(), Some(2), "{o:?}");
}

#[test]
fn two_renders_give_identical_state_handoff_and_lessons() {
    let r = repo();
    lesson_files(&r);
    items(&r);
    let (a, b) = (r.out_dir("a"), r.out_dir("b"));
    assert!(r.render_out(&a, &[]).status.success());
    assert!(r.render_out(&b, &[]).status.success());
    for v in ["STATE.md", "HANDOFF.md", "docs/harness/LESSONS.md"] {
        let x = read(&a, v);
        assert!(x.len() > 100, "{v}: {x}");
        assert_eq!(x, read(&b, v), "{v}");
    }
}
