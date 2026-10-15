//! `git clean-merged` releases the claims on the branches it proves merged.
//!
//! The mechanical conflict class from #2028: a claim outlives its branch. The PR merges, the branch
//! is deleted, and the entry stays in `docs/harness/claims.json` until the next lab collides with a
//! claim on work that is already in main. The proof the release needs is the one `clean-merged`
//! already makes - mergedness by CONTENT, because a squash merge leaves no ancestry link - so the
//! post-merge step every lab runs is where the claim goes too.
//!
//! The other half matters as much: an UNMERGED branch keeps its claim. A release that cleared
//! everything would be a lock nobody could hold.

use std::path::PathBuf;

use std::process::{Command, Output};

/// Three claims: one on a branch that reaches main by squash, one on a branch still open, and one on
/// a branch that exists nowhere - the residue `--apply` must REPORT but must never release, because
/// "gone" is not "merged".
const CLAIMS: &str = concat!(
    "{\n",
    "  \"ghost/gone\": {\"branch\": \"ghost/gone\", \"claimed_at\": \"2026-10-15T00:00:00-04:00\", \"head\": \"000\", \"lab\": \"ghost\", \"note\": \"\"},\n",
    "  \"lab/done\": {\"branch\": \"lab/done\", \"claimed_at\": \"2026-10-15T00:00:00-04:00\", \"head\": \"abc\", \"lab\": \"lab\", \"note\": \"\"},\n",
    "  \"lab/open\": {\"branch\": \"lab/open\", \"claimed_at\": \"2026-10-15T00:00:00-04:00\", \"head\": \"def\", \"lab\": \"lab\", \"note\": \"\"}\n",
    "}\n"
);

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_uor-harness")
}

struct Fixture {
    _dir: tempfile::TempDir,
    repo: PathBuf,
    live: PathBuf,
}

impl Fixture {
    /// main (pushed to a bare origin), one branch squash-merged into it, one branch left open.
    fn new() -> Fixture {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("repo");
        let live = dir.path().join("live");
        let origin = dir.path().join("origin.git");
        std::fs::create_dir_all(repo.join("docs/harness")).expect("dirs");
        std::fs::create_dir_all(&live).expect("live");
        std::fs::create_dir_all(&origin).expect("origin");
        let git = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .current_dir(&repo)
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
        std::fs::write(repo.join("f"), "one\n").expect("write");
        std::fs::write(repo.join("docs/harness/claims.json"), CLAIMS).expect("claims");
        git(&["add", "-A"]);
        git(&["commit", "-qm", "one"]);
        git(&[
            "init",
            "-q",
            "--bare",
            "-b",
            "main",
            origin.to_str().expect("utf8"),
        ]);
        git(&["remote", "add", "origin", origin.to_str().expect("utf8")]);
        git(&["push", "-q", "-u", "origin", "main"]);
        // A branch whose work reaches main by SQUASH: same tree, no ancestry link.
        git(&["checkout", "-q", "-b", "lab/done"]);
        std::fs::write(repo.join("f"), "one\ndone\n").expect("write");
        git(&["commit", "-qam", "done"]);
        git(&["checkout", "-q", "main"]);
        git(&["merge", "-q", "--squash", "lab/done"]);
        git(&["commit", "-qm", "squash lab/done"]);
        git(&["push", "-q", "origin", "main"]);
        // A branch that is NOT merged: its claim must survive.
        git(&["checkout", "-q", "-b", "lab/open"]);
        std::fs::write(repo.join("g"), "open\n").expect("write");
        git(&["add", "g"]);
        git(&["commit", "-qm", "open"]);
        git(&["checkout", "-q", "main"]);
        Fixture {
            _dir: dir,
            repo,
            live,
        }
    }

    fn run(&self, args: &[&str]) -> Output {
        let repo = self.repo.to_str().expect("utf8");
        let mut full: Vec<&str> = vec!["--repo", repo];
        full.extend_from_slice(args);
        Command::new(bin())
            .args(&full)
            .env("UOR_HARNESS_LIVE", &self.live)
            .env("UOR_LAB", "claimprobe")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("run uor-harness")
    }

    /// The committed store in the checkout.
    fn claims_file(&self, root: &std::path::Path) -> String {
        std::fs::read_to_string(root.join("docs/harness/claims.json")).expect("claims")
    }

    /// The live mirror. **Not** under `docs/harness`: the live tier keeps `claims.json` at its root,
    /// which is what makes it a coordination store with no checkout in the path.
    fn live_claims(&self) -> String {
        std::fs::read_to_string(self.live.join("claims.json")).expect("live claims")
    }
}

#[test]
fn a_merged_branchs_claim_is_released_and_an_open_branchs_is_kept() {
    let f = Fixture::new();
    let before = f.claims_file(&f.repo);
    assert!(
        before.contains("lab/done") && before.contains("lab/open"),
        "{before}"
    );

    // The report names the stale claim and writes nothing.
    let report = f.run(&["git", "clean-merged"]);
    let out = String::from_utf8_lossy(&report.stdout).to_string();
    assert!(out.contains("claims on merged branches (1)"), "{out}");
    assert!(out.contains("lab/done"), "{out}");
    // R2: the claim whose branch is gone is NAMED, with the command that clears it, and the report
    // says plainly that it is not released.
    assert!(out.contains("claims whose branch is gone (1)"), "{out}");
    assert!(out.contains("ghost/gone"), "{out}");
    assert!(
        out.contains("team release --branch ghost/gone --lab ghost"),
        "the residue must come with its fix: {out}"
    );
    assert!(out.contains("gone is not merged"), "{out}");
    assert_eq!(f.claims_file(&f.repo), before, "a report must not write");

    // Apply: the merged branch is deleted AND its claim released; the open branch keeps both.
    let applied = f.run(&["git", "clean-merged", "--apply"]);
    let out = String::from_utf8_lossy(&applied.stdout).to_string();
    assert!(
        out.contains("released 1 claim(s) on merged branches"),
        "{out}"
    );
    let after = f.claims_file(&f.repo);
    assert!(
        !after.contains("lab/done"),
        "the merged claim is gone: {after}"
    );
    assert!(
        after.contains("lab/open"),
        "the open claim survives: {after}"
    );
    assert!(
        after.contains("ghost/gone"),
        "a branch that is gone is NOT released on a guess: {after}"
    );

    // And it is gone from the LIVE mirror too, or the next reader resurrects it.
    let live = f.live_claims();
    assert!(
        !live.contains("lab/done"),
        "the mirror must not keep it: {live}"
    );
    assert!(live.contains("lab/open"), "{live}");
}
