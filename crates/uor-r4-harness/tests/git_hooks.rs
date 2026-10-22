//! The git hooks (loop PR 12): plan and install, the commit-message check, the pre-push check, and
//! the hook scripts end to end. Every child process has every GIT_* variable removed; each
//! repository is asserted to sit inside its temporary directory, and no test touches the network.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const PRE_COMMIT: &str = include_str!("../rules/precommit-hook.sh");
const COMMIT_MSG: &str = include_str!("../../../rules/commit-msg-hook.sh");
const PRE_PUSH: &str = include_str!("../../../rules/prepush-hook.sh");
const ORDER: &str = "docs/loop/tasks/infra-loop-harness/deepseek-5/ORDER.md";

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(&k);
        }
    }
    cmd.env_remove("UOR_LAB").env_remove("CI")
}

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
    fn new() -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap().join("repo");
        assert!(root.starts_with(tmp.path().canonicalize().unwrap()));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { _tmp: tmp, root };
        for args in [
            &["init", "-q", "-b", "main"][..],
            &["config", "user.email", "t@example.invalid"][..],
            &["config", "user.name", "t"][..],
            &["config", "core.hooksPath", ".git/hooks"][..],
        ] {
            r.git(args);
        }
        r.write("README.md", "# r\n");
        r.write(
            "docs/harness/labs.json",
            r#"{"schema":1,"labs":{"deepseek":{"vendor":"DeepSeek","client":"dsh","headless":"dsh headless","open_pr_limit":3}}}"#,
        );
        r.write("src/a.txt", "a\n");
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "init"]);
        let remote = r._tmp.path().canonicalize().unwrap().join("remote.git");
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

    fn git(&self, args: &[&str]) -> String {
        let st = clean(&mut Command::new("git"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs");
        assert!(st.status.success(), "git {args:?}: {st:?}");
        String::from_utf8_lossy(&st.stdout).trim().to_string()
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.root.join(rel);
        assert!(p.starts_with(&self.root));
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn run(&self, args: &[&str], envs: &[(&str, &str)], stdin: Option<&str>) -> Output {
        let mut built = Command::new(BIN);
        let cmd = clean(&mut built);
        cmd.args(["--repo", self.root.to_str().unwrap()])
            .args(args)
            .current_dir(&self.root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in envs {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().expect("uor-harness runs");
        if let Some(text) = stdin {
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(text.as_bytes())
                .unwrap();
        }
        child.wait_with_output().expect("wait")
    }

    fn hooks_dir(&self) -> PathBuf {
        let raw = self.git(&["rev-parse", "--git-path", "hooks"]);
        let p = self.root.join(raw);
        assert!(
            p.starts_with(&self.root),
            "hooks outside the temp repo: {p:?}"
        );
        p
    }

    fn message(&self, name: &str, body: &str) -> PathBuf {
        let p = self.root.join(name);
        assert!(p.starts_with(&self.root));
        std::fs::write(&p, body).unwrap();
        p
    }

    fn branch(&self, name: &str) {
        self.git(&["checkout", "-q", "-b", name]);
    }

    fn install(&self) -> Output {
        self.run(&["git", "install-hooks", "--install"], &[], None)
    }
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn assert_code(o: &Output, want: i32, what: &str) {
    assert_eq!(
        o.status.code(),
        Some(want),
        "{what} (wanted exit {want}): {}",
        text(o)
    );
}

fn run_script(repo: &Repo, name: &str, stdin: &str, envs: &[(&str, &str)]) -> Output {
    let script = repo.hooks_dir().join(name);
    let bin = Path::new(BIN).parent().expect("bin dir");
    let mut shell = Command::new("/bin/sh");
    let cmd = clean(&mut shell);
    cmd.arg(&script)
        .arg(repo.root.join("msg.txt"))
        .arg("origin")
        .arg("git@example.invalid:repo.git")
        .current_dir(&repo.root)
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().expect("hook runs");
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().expect("wait")
}

#[test]
fn install_hooks_is_a_dry_run_by_default() {
    let r = Repo::new();
    let o = r.run(&["git", "install-hooks"], &[], None);
    let out = text(&o);
    assert_code(&o, 0, "a dry run is not an error");
    assert!(out.contains("would install:"), "{out}");
    assert!(out.contains("--install"), "{out}");
    for name in ["pre-commit", "commit-msg", "pre-push"] {
        assert!(
            !r.hooks_dir().join(name).exists(),
            "{name} was written by a dry run"
        );
    }
}

#[test]
fn install_hooks_installs_all_three_and_a_rerun_is_unchanged() {
    let r = Repo::new();
    let o = r.install();
    let out = text(&o);
    assert_code(&o, 0, "install");
    assert_eq!(out.matches("installed:").count(), 3, "{out}");
    for (name, body) in [
        ("pre-commit", PRE_COMMIT),
        ("commit-msg", COMMIT_MSG),
        ("pre-push", PRE_PUSH),
    ] {
        let p = r.hooks_dir().join(name);
        assert_eq!(
            std::fs::read_to_string(&p).expect("hook reads"),
            body,
            "{name}"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&p).unwrap().permissions().mode();
            assert_eq!(mode & 0o111, 0o111, "{name} not executable, mode {mode:o}");
        }
    }
    let again = r.install();
    assert_code(&again, 0, "rerun");
    assert_eq!(
        text(&again).matches("unchanged:").count(),
        3,
        "{}",
        text(&again)
    );
}

#[test]
fn install_hooks_is_all_or_none_and_a_foreign_hook_is_refused() {
    let r = Repo::new();
    let dir = r.hooks_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let foreign = "#!/bin/sh\n# somebody else writes this hook\nexit 0\n";
    std::fs::write(dir.join("commit-msg"), foreign).unwrap();
    let o = r.install();
    let out = text(&o);
    assert_code(&o, 2, "a foreign hook is a refusal");
    assert!(out.contains("was not written by the harness"), "{out}");
    assert_eq!(
        std::fs::read_to_string(dir.join("commit-msg")).unwrap(),
        foreign
    );
    for name in ["pre-commit", "pre-push"] {
        assert!(
            !dir.join(name).exists(),
            "{name} was written despite the refusal"
        );
    }
}

#[test]
fn install_hooks_backs_up_a_hook_it_replaces() {
    let r = Repo::new();
    let dir = r.hooks_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let old = format!(
        "#!/bin/sh\n{} # older body\nexit 0\n",
        "# >>> uor-harness pre-commit v2 (managed by crates/uor-r4-harness) >>>"
    );
    std::fs::write(dir.join("pre-commit"), &old).unwrap();
    let o = r.install();
    assert_code(&o, 0, "install over a managed hook");
    let mut backups: Vec<PathBuf> = Vec::new();
    let root = r.root.join("local/loop/install-backups");
    for entry in std::fs::read_dir(&root).expect("backups dir") {
        let p = entry.unwrap().path().join("pre-commit");
        if p.is_file() {
            backups.push(p);
        }
    }
    assert_eq!(backups.len(), 1, "one backup: {backups:?}");
    assert_eq!(std::fs::read_to_string(&backups[0]).unwrap(), old);
}

/// The stock body Git LFS installs as `pre-push`, simplified: replaceable, and backed up.
const STOCK_LFS_PRE_PUSH: &str = "#!/bin/sh\ncommand -v git-lfs >/dev/null 2>&1 || { printf >&2 \"This repository is configured for Git LFS but git-lfs was not found.\"; exit 2; }\ngit lfs pre-push \"$@\"\n";

#[test]
fn the_stock_lfs_pre_push_is_replaced_and_backed_up() {
    let r = Repo::new();
    let dir = r.hooks_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pre-push"), STOCK_LFS_PRE_PUSH).unwrap();
    let o = r.install();
    assert_code(&o, 0, "the stock LFS pre-push is replaceable");
    assert_eq!(
        std::fs::read_to_string(dir.join("pre-push")).unwrap(),
        PRE_PUSH,
        "our chained hook is installed"
    );
    let mut backups: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(r.root.join("local/loop/install-backups")).expect("backups") {
        let p = entry.unwrap().path().join("pre-push");
        if p.is_file() {
            backups.push(p);
        }
    }
    assert_eq!(backups.len(), 1, "the stock hook is backed up: {backups:?}");
    assert_eq!(
        std::fs::read_to_string(&backups[0]).unwrap(),
        STOCK_LFS_PRE_PUSH
    );
}

#[test]
fn install_hooks_refuses_a_lab_session() {
    let r = Repo::new();
    let o = r.run(
        &["git", "install-hooks", "--install"],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    let out = text(&o);
    assert_code(&o, 2, "a lab session may not install the shared hooks");
    assert!(out.contains("owner"), "{out}");
    assert!(!r.hooks_dir().join("pre-commit").exists());
}

fn trailer_message() -> String {
    format!("a subject\n\nwhy\n\nLab: deepseek\nOrder: {ORDER}\n")
}

#[test]
fn a_lab_branch_message_without_trailers_is_refused() {
    let r = Repo::new();
    r.branch("deepseek/topic");
    let m = r.message("msg.txt", "a message with no trailers\n");
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(
        &o,
        1,
        "a message without trailers must be refused on a lab branch",
    );
    assert!(text(&o).contains("no Lab:"), "{}", text(&o));
}

#[test]
fn a_lab_branch_message_with_trailers_passes() {
    let r = Repo::new();
    r.branch("deepseek/topic");
    let m = r.message("msg.txt", &trailer_message());
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(&o, 0, "trailers in the final paragraph pass");
}

#[test]
fn trailers_only_count_in_the_final_paragraph_and_keys_are_case_insensitive() {
    let r = Repo::new();
    r.branch("deepseek/topic");
    let m = r.message(
        "msg.txt",
        &format!("subject\n\nLab: deepseek\nOrder: {ORDER}\n\nbody after the trailers\n"),
    );
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(&o, 1, "trailers above the final paragraph do not count");
    let m = r.message(
        "msg2.txt",
        &format!("subject\n\nlab: deepseek\norder: {ORDER}\n"),
    );
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(&o, 0, "keys are case-insensitive");
}

/// Git runs commit-msg before it strips comments: an editor message ends with a `#` block.
#[test]
fn an_editor_message_is_judged_after_the_comment_block_is_stripped() {
    let r = Repo::new();
    r.branch("deepseek/topic");
    let editor = format!(
        "a subject\n\nwhy\n\nLab: deepseek\nOrder: {ORDER}\n\n# Please enter the commit message for your changes.\n# Lines starting with a hash will be ignored.\n#\n# On branch deepseek/topic\n"
    );
    let m = r.message("msg.txt", &editor);
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(
        &o,
        0,
        "the comment block is stripped before the trailer read",
    );
    let no_trailers = "a subject\n\n# Please enter the commit message for your changes.\n# On branch deepseek/topic\n";
    let m = r.message("msg2.txt", no_trailers);
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(&o, 1, "no trailers even without the comment block");
}

#[test]
fn the_lab_trailer_must_match_uor_lab() {
    let r = Repo::new();
    r.branch("deepseek/topic");
    let m = r.message(
        "msg.txt",
        &format!("a subject\n\nLab: kimi\nOrder: {ORDER}\n"),
    );
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    let out = text(&o);
    assert_code(&o, 1, "the Lab trailer must match UOR_LAB");
    assert!(out.contains("does not match UOR_LAB"), "{out}");
}

#[test]
fn only_gits_own_generated_forms_skip_the_check() {
    let r = Repo::new();
    r.branch("deepseek/topic");
    for body in [
        "Merge branch x into main\n",
        "Merge pull request #1 from x/y\n",
        "Merge remote-tracking branch origin/main\n",
        "Revert \"a change\"\n",
        "fixup! a subject\n",
        "squash! a subject\n",
    ] {
        let m = r.message("msg.txt", body);
        let o = r.run(
            &["git", "check-message", m.to_str().unwrap()],
            &[("UOR_LAB", "deepseek")],
            None,
        );
        assert_code(&o, 0, "a generated message passes");
    }
    let m = r.message("msg.txt", "Merge something by hand\n");
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(&o, 1, "a hand-written Merge subject is not exempt");
}
#[test]
fn generated_and_fixup_messages_pass_on_a_lab_branch() {
    let r = Repo::new();
    r.branch("deepseek/topic");
    for body in [
        "Merge branch main into topic\n",
        "Revert \"a change\"\n",
        "fixup! a subject\n",
        "squash! a subject\n",
    ] {
        let m = r.message("msg.txt", body);
        let o = r.run(
            &["git", "check-message", m.to_str().unwrap()],
            &[("UOR_LAB", "deepseek")],
            None,
        );
        assert_code(&o, 0, "generated message passes");
    }
}

#[test]
fn only_lab_branches_with_uor_lab_are_checked() {
    let r = Repo::new();
    let m = r.message("msg.txt", "a message with no trailers\n");
    // On main, even with UOR_LAB set, the owner-style commit is not checked.
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(&o, 0, "main is not a lab branch");
    // On a lab branch without UOR_LAB, non-loop work is not checked.
    r.branch("deepseek/topic");
    let o = r.run(&["git", "check-message", m.to_str().unwrap()], &[], None);
    assert_code(&o, 0, "no UOR_LAB means no loop check");
}

#[test]
fn the_commit_msg_hook_blocks_and_allows_a_real_commit() {
    let r = Repo::new();
    assert_code(&r.install(), 0, "install");
    r.branch("deepseek/topic");
    r.write("src/b.txt", "b\n");
    r.git(&["add", "-A"]);
    let denied = clean(&mut Command::new("git"))
        .args(["commit", "-q", "-m", "no trailers here"])
        .current_dir(&r.root)
        .env("UOR_LAB", "deepseek")
        .env(
            "PATH",
            format!(
                "{}:/usr/bin:/bin",
                Path::new(BIN).parent().unwrap().display()
            ),
        )
        .output()
        .expect("git commit runs");
    assert!(!denied.status.success(), "the hook must block: {denied:?}");
    let allowed = clean(&mut Command::new("git"))
        .args([
            "commit",
            "-q",
            "-m",
            "a subject",
            "-m",
            &format!("Lab: deepseek\nOrder: {ORDER}"),
        ])
        .current_dir(&r.root)
        .env("UOR_LAB", "deepseek")
        .env(
            "PATH",
            format!(
                "{}:/usr/bin:/bin",
                Path::new(BIN).parent().unwrap().display()
            ),
        )
        .output()
        .expect("git commit runs");
    assert!(
        allowed.status.success(),
        "the trailers allow it: {allowed:?}"
    );
}

#[test]
fn push_check_skips_a_non_lab_ref_and_a_delete() {
    let r = Repo::new();
    let sha = r.git(&["rev-parse", "HEAD"]);
    let zeros = "0".repeat(40);
    let main = format!("refs/heads/main {sha} refs/heads/main {sha}\n");
    let o = r.run(&["git", "push-check"], &[], Some(&main));
    assert_code(&o, 0, "main skips");
    assert!(
        text(&o).contains("not a <lab>/<topic> branch"),
        "{}",
        text(&o)
    );
    let delete = format!("(delete) {zeros} refs/heads/deepseek/gone {sha}\n");
    let o = r.run(&["git", "push-check"], &[], Some(&delete));
    assert_code(&o, 0, "a delete skips");
    assert!(text(&o).contains("deleting a branch"), "{}", text(&o));
}

#[test]
fn push_check_checks_a_new_lab_branch() {
    let r = Repo::new();
    let sha = r.git(&["rev-parse", "HEAD"]);
    let zeros = "0".repeat(40);
    let new_branch = format!("refs/heads/deepseek/topic {sha} refs/heads/deepseek/topic {zeros}\n");
    let o = r.run(&["git", "push-check"], &[], Some(&new_branch));
    assert_code(
        &o,
        1,
        "a new lab branch is checked and its pr-check fails here",
    );
    let out = text(&o);
    assert!(out.contains("check deepseek/topic at"), "{out}");
    assert!(out.contains("(new branch)"), "{out}");
}

#[test]
fn push_check_does_not_block_when_pr_check_cannot_run() {
    let r = Repo::new();
    r.git(&["remote", "remove", "origin"]);
    let sha = r.git(&["rev-parse", "HEAD"]);
    let line = format!("refs/heads/deepseek/topic {sha} refs/heads/deepseek/topic {sha}\n");
    let o = r.run(&["git", "push-check"], &[], Some(&line));
    assert_code(&o, 0, "cannot run is not a policy failure");
    assert!(text(&o).contains("cannot run pr-check"), "{}", text(&o));
}

#[test]
fn the_prepush_hook_runs_git_lfs_first_on_the_same_stdin() {
    use std::os::unix::fs::PermissionsExt;
    let r = Repo::new();
    assert_code(&r.install(), 0, "install");
    let bin = r._tmp.path().canonicalize().unwrap().join("fakebin");
    std::fs::create_dir_all(&bin).unwrap();
    let marker = r.root.join("lfs-stdin.txt");
    let lfs = bin.join("git-lfs");
    std::fs::write(
        &lfs,
        format!("#!/bin/sh\ncat > {}\nexit 0\n", marker.display()),
    )
    .unwrap();
    std::fs::set_permissions(&lfs, std::fs::Permissions::from_mode(0o755)).unwrap();
    let sha = r.git(&["rev-parse", "HEAD"]);
    let zeros = "0".repeat(40);
    let line = format!("refs/heads/deepseek/topic {sha} refs/heads/deepseek/topic {zeros}\n");
    let hook = run_script(
        &r,
        "pre-push",
        &line,
        &[
            (
                "PATH",
                &format!(
                    "{}:{}:/usr/bin:/bin",
                    bin.display(),
                    Path::new(BIN).parent().unwrap().display()
                ),
            ),
            ("UOR_LAB", "deepseek"),
        ],
    );
    let seen = std::fs::read_to_string(&marker).expect("git lfs ran");
    assert_eq!(seen, line, "LFS got the same stdin");
    assert!(!hook.status.success(), "the policy failure still blocks");
}

#[test]
fn a_hook_without_uor_harness_on_path_allows_the_commit() {
    let r = Repo::new();
    assert_code(&r.install(), 0, "install");
    r.branch("deepseek/topic");
    r.message("msg.txt", "no trailers\n");
    let script = r.hooks_dir().join("commit-msg");
    let out = clean(&mut Command::new("/bin/sh"))
        .arg(&script)
        .arg(r.root.join("msg.txt"))
        .current_dir(&r.root)
        .env("PATH", "/usr/bin:/bin")
        .env("UOR_LAB", "deepseek")
        .output()
        .expect("hook runs");
    assert_code(&out, 0, "no harness means no block");
}

#[test]
fn the_prepush_hook_exits_2_when_git_lfs_is_missing() {
    let r = Repo::new();
    assert_code(&r.install(), 0, "install");
    let sha = r.git(&["rev-parse", "HEAD"]);
    let line = format!("refs/heads/deepseek/topic {sha} refs/heads/deepseek/topic {sha}\n");
    let script = r.hooks_dir().join("pre-push");
    // A PATH that has the shell tools the hook uses but no git-lfs, on any runner.
    let bin = r._tmp.path().canonicalize().unwrap().join("no-lfs-bin");
    std::fs::create_dir_all(&bin).unwrap();
    for tool in ["git", "mktemp", "cat"] {
        std::os::unix::fs::symlink(format!("/usr/bin/{tool}"), bin.join(tool)).unwrap();
    }
    let mut shell = Command::new("/bin/sh");
    let cmd = clean(&mut shell);
    let mut child = cmd
        .arg(&script)
        .current_dir(&r.root)
        .env("PATH", &bin)
        .env("UOR_LAB", "deepseek")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("hook runs");
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(line.as_bytes())
        .unwrap();
    let o = child.wait_with_output().expect("wait");
    assert_code(&o, 2, "no git-lfs is the stock refusal");
    assert!(text(&o).contains("Git LFS"), "{}", text(&o));
}

#[test]
fn a_custom_comment_char_is_honoured() {
    let r = Repo::new();
    r.git(&["config", "core.commentChar", ";"]);
    r.branch("deepseek/topic");
    let with =
        format!("a subject\n\nLab: deepseek\nOrder: {ORDER}\n\n; a comment line\n; another\n");
    let m = r.message("msg.txt", &with);
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(&o, 0, "a custom comment char block is stripped");
    let without = "a subject\n\n; a comment line\n; another\n";
    let m = r.message("msg2.txt", without);
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(&o, 1, "no trailers under a custom comment char");
}

#[test]
fn an_auto_comment_char_is_read_from_gits_inserted_block() {
    let r = Repo::new();
    r.git(&["config", "core.commentChar", "auto"]);
    r.branch("deepseek/topic");
    let with = format!(
        "a subject\n\nLab: deepseek\nOrder: {ORDER}\n\n# Please enter the commit message for your changes.\n# Lines starting with a hash are ignored.\n"
    );
    let m = r.message("msg.txt", &with);
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(&o, 0, "auto: the hash block is stripped");
}

#[test]
fn the_scissors_block_is_dropped() {
    let r = Repo::new();
    r.branch("deepseek/topic");
    let good = format!(
        "a subject\n\nLab: deepseek\nOrder: {ORDER}\n\n# ------------------------ >8 ------------------------\n# Lab: fake\n"
    );
    let m = r.message("msg.txt", &good);
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(&o, 0, "trailers above the scissors count");
    let bad = "a subject\n\n# ------------------------ >8 ------------------------\n# Lab: fake\n# Order: docs/loop/tasks/x/ORDER.md\n";
    let m = r.message("msg2.txt", bad);
    let o = r.run(
        &["git", "check-message", m.to_str().unwrap()],
        &[("UOR_LAB", "deepseek")],
        None,
    );
    assert_code(&o, 1, "fake trailers below the scissors do not count");
}
