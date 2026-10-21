//! **R10 pinned at every committed-write entry point.**
//!
//! Review finding F4: the guards added for R10 were covered by two tests, so deleting the guard
//! from `ledger-sync` (or `multilab sync-live`, `migrate-ledger`, `prep new`, `mistakes capture`,
//! `ratchet --update-baseline`, `git install-hooks`) left the whole unit suite green - a guard that
//! nothing would notice losing. Each entry point is therefore exercised here through the REAL
//! binary, in a real shared-layout clone, and every case asserts two things: the refusal names R10,
//! and the shared checkout is unchanged afterwards.
//!
//! The live tier is pointed at a scratch directory for every run. A test must never write the
//! machine's real shared state - that is the mistake `StatePaths::resolve` exists to prevent.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_uor-harness")
}

/// The project's own `docs/harness/`, copied into the fixture so commands that read registries
/// behave as they do for a lab. The refusal is asserted to be the R10 one, so a case that failed
/// earlier for a missing file fails this test rather than passing it.
fn harness_docs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/harness")
}

/// Register this fixture's lab in the fixture's own copy of `labs.json`.
///
/// An unregistered lab is refused before R10 is reached, so without this every case here would fail
/// for the wrong reason. A real lab name is not used instead because the copied ledger already holds
/// real labs' shards, and `the_lab_shard` needs the probe's shard to be the only one with its name.
fn register_probe_lab(path: &Path) {
    let text = std::fs::read_to_string(path).expect("labs.json reads");
    let mut doc: serde_json::Value = serde_json::from_str(&text).expect("labs.json parses");
    let template = doc["labs"]["claude"].clone();
    doc["labs"]["r10probe"] = template;
    std::fs::write(
        path,
        serde_json::to_string_pretty(&doc).expect("serialises"),
    )
    .expect("write");
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read_dir").flatten() {
        let target = to.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy");
        }
    }
}

struct Shared {
    _dir: tempfile::TempDir,
    primary: PathBuf,
    linked: PathBuf,
    live: PathBuf,
}

impl Shared {
    /// A clone in the shape R10 protects: a primary checkout with a linked worktree.
    fn with_linked_worktree() -> Shared {
        Self::build(true)
    }

    /// A plain single-checkout clone, the shape left behind once cleanup removes every worktree.
    fn single_checkout() -> Shared {
        Self::build(false)
    }

    fn build(linked_worktree: bool) -> Shared {
        let dir = tempfile::tempdir().expect("tempdir");
        let primary = dir.path().join("owner");
        let linked = dir.path().join("lab");
        let live = dir.path().join("live");
        std::fs::create_dir_all(&primary).expect("primary");
        std::fs::create_dir_all(&live).expect("live");
        copy_tree(&harness_docs(), &primary.join("docs/harness"));
        register_probe_lab(&primary.join("docs/harness/labs.json"));
        let git = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .current_dir(&primary)
                .output()
                .expect("git");
            assert!(
                out.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            out
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["config", "user.email", "t@t"]);
        git(&["config", "user.name", "t"]);
        std::fs::write(primary.join("f"), "1").expect("write");
        git(&["add", "-A"]);
        git(&["commit", "-qm", "one"]);
        if linked_worktree {
            git(&[
                "worktree",
                "add",
                "-q",
                "-b",
                "lab/task",
                linked.to_str().expect("utf8"),
            ]);
        }
        Shared {
            _dir: dir,
            primary,
            linked,
            live,
        }
    }

    /// Run the harness with this fixture's scratch live tier. `GIT_DIR`/`GIT_WORK_TREE` are removed
    /// so a hook's environment in the developer's shell cannot leak into a case.
    fn run(&self, cwd: &Path, args: &[&str]) -> Output {
        Command::new(bin())
            .args(args)
            .current_dir(cwd)
            .env("UOR_HARNESS_LIVE", &self.live)
            .env("UOR_LAB", "r10probe")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("run uor-harness")
    }

    fn git(&self, cwd: &Path, args: &[&str]) -> Output {
        Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git")
    }

    fn dirty(&self, cwd: &Path) -> String {
        String::from_utf8_lossy(&self.git(cwd, &["status", "--porcelain"]).stdout).to_string()
    }
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Every committed-write entry point, with arguments that reach its guard.
fn committed_writes() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "team claim",
            vec![
                "team",
                "claim",
                "--branch",
                "r10/probe",
                "--lab",
                "r10probe",
            ],
        ),
        ("ledger-sync", vec!["ledger-sync", "--lab", "r10probe"]),
        ("multilab sync-live", vec!["multilab", "sync-live"]),
        (
            "multilab migrate-ledger",
            vec!["multilab", "migrate-ledger"],
        ),
        (
            "prep new",
            vec!["prep", "new", "-m", "m1", "--mechanism", "r10-probe"],
        ),
        (
            "mistakes capture",
            vec![
                "mistakes",
                "capture",
                "--id",
                "r10-probe",
                "--title",
                "R10 probe",
            ],
        ),
        (
            "ratchet --update-baseline",
            vec!["ratchet", "d23", "--update-baseline"],
        ),
        ("git install-hooks", vec!["git", "install-hooks"]),
    ]
}

#[test]
fn every_committed_write_refuses_the_shared_checkout() {
    let shared = Shared::with_linked_worktree();
    for (name, args) in committed_writes() {
        let before = shared.dirty(&shared.primary);
        let out = shared.run(&shared.primary, &args);
        let err = stderr(&out);
        assert!(
            !out.status.success(),
            "{name} must fail in the shared checkout; stdout={} stderr={err}",
            stdout(&out)
        );
        assert!(err.contains("R10"), "{name} must refuse for R10: {err}");
        assert_eq!(
            shared.dirty(&shared.primary),
            before,
            "{name} changed the shared checkout"
        );
    }
}

/// The other half of the pin: the guard must not refuse a lab's OWN worktree. A test that only
/// asserted the refusal would pass if the guard refused everywhere.
#[test]
fn the_same_writes_are_not_refused_in_the_labs_own_worktree() {
    let shared = Shared::with_linked_worktree();
    for (name, args) in committed_writes() {
        let out = shared.run(&shared.linked, &args);
        let err = stderr(&out);
        assert!(
            !err.contains("R10"),
            "{name} must not be refused in a lab worktree: {err}"
        );
    }
}

/// `ledger-sync --dry-run` is read-only at the DISPATCH level, not only in `plan_sync`. The flag was
/// documented and ignored once, and a unit test of `plan_sync` alone would not have caught that.
#[test]
fn ledger_sync_dry_run_writes_nothing_and_the_real_sync_carries_one_line() {
    let shared = Shared::with_linked_worktree();
    let receipt = shared.run(
        &shared.linked,
        &["receipt", "--kind", "skill", "--note", "r10 dry-run probe"],
    );
    assert!(receipt.status.success(), "{}", stderr(&receipt));

    let before = shared.dirty(&shared.linked);
    let dry = shared.run(
        &shared.linked,
        &["ledger-sync", "--lab", "r10probe", "--dry-run"],
    );
    assert!(dry.status.success(), "{}", stderr(&dry));
    assert!(stdout(&dry).contains("dry-run"), "{}", stdout(&dry));
    assert_eq!(
        shared.dirty(&shared.linked),
        before,
        "the dry run wrote into the checkout"
    );

    let real = shared.run(&shared.linked, &["ledger-sync", "--lab", "r10probe"]);
    assert!(real.status.success(), "{}", stderr(&real));
    let shard = the_lab_shard(&shared.linked);
    let lines = std::fs::read_to_string(&shard).expect("shard");
    assert_eq!(
        lines.lines().filter(|l| !l.trim().is_empty()).count(),
        1,
        "the real sync must carry exactly the line the dry run planned: {}",
        shard.display()
    );
}

/// The committed shard for this fixture's lab, whatever month the run happened in.
fn the_lab_shard(root: &Path) -> PathBuf {
    let dir = root.join("docs/harness/ledger");
    let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|it| {
            it.flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with("r10probe-"))
                })
                .collect()
        })
        .unwrap_or_default();
    found.sort();
    assert_eq!(found.len(), 1, "expected one r10probe shard: {found:?}");
    found.remove(0)
}

/// **F1: the shape rule has a hole when cleanup removes every linked worktree.** A marked
/// single-checkout clone is still the shared checkout; an unmarked one is a lab's own.
#[test]
fn a_marked_single_checkout_refuses_and_an_unmarked_one_does_not() {
    let shared = Shared::single_checkout();
    let claim = [
        "team",
        "claim",
        "--branch",
        "r10/probe",
        "--lab",
        "r10probe",
    ];

    let unmarked = shared.run(&shared.primary, &claim);
    assert!(
        !stderr(&unmarked).contains("R10"),
        "an unmarked single checkout is writable: {}",
        stderr(&unmarked)
    );
    shared.git(&shared.primary, &["config", "uor.sharedCheckout", "true"]);
    let before = shared.dirty(&shared.primary);
    let marked = shared.run(&shared.primary, &claim);
    assert!(!marked.status.success(), "{}", stdout(&marked));
    assert!(stderr(&marked).contains("R10"), "{}", stderr(&marked));
    assert_eq!(
        shared.dirty(&shared.primary),
        before,
        "the checkout stayed clean"
    );
}

/// **F5: a git hook exports `GIT_DIR`/`GIT_WORK_TREE`.** Inherited, they make `git -C <worktree>`
/// answer for the MAIN worktree, so a lab's own checkout looked like the shared one and its write was
/// refused for the wrong reason. The guard clears them.
#[test]
fn a_hook_environment_does_not_make_a_lab_worktree_look_shared() {
    let shared = Shared::with_linked_worktree();
    let out = Command::new(bin())
        .args(["team", "claim", "--branch", "r10/hook", "--lab", "r10probe"])
        .current_dir(&shared.linked)
        .env("UOR_HARNESS_LIVE", &shared.live)
        .env("UOR_LAB", "r10probe")
        .env("GIT_DIR", shared.primary.join(".git"))
        .env("GIT_WORK_TREE", &shared.primary)
        .output()
        .expect("run uor-harness");
    let err = stderr(&out);
    assert!(
        !err.contains("R10"),
        "a lab worktree must not be mistaken for the shared checkout: {err}"
    );
    assert!(out.status.success(), "{err}");
}
