//! Exercise the exact Pages workflow shells against local build artifacts and a bare Git remote.
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const DOCS: &str = include_str!("../../../.github/workflows/docs-site.yml");
const CLEANUP: &str = include_str!("../../../.github/workflows/branch-cleanup.yml");
const STUDIO: &str = include_str!("../../../scripts/studio/build-and-publish.sh");
const REGISTRY: &str = include_str!("../../../docs/harness/branches.json");
const GATE: &str =
    "github.event_name == 'workflow_dispatch' && github.ref == 'refs/heads/main' && inputs.target == 'gh-pages'";

fn shell_step(name: &str) -> String {
    let (_, step) = DOCS.split_once(&format!("      - name: {name}\n")).unwrap();
    let (_, block) = step.split_once("        run: |\n").unwrap();
    block
        .lines()
        .take_while(|line| line.is_empty() || line.starts_with("          "))
        .map(|line| line.strip_prefix("          ").unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

fn run(dir: &Path, program: &str, args: &[&str]) -> Output {
    let mut command = Command::new(program);
    // The fixture must never inherit a checkout, credentials or a push target from the caller.
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("GIT_") {
            command.env_remove(name);
        }
    }
    command
        .env_remove("GITHUB_STEP_SUMMARY")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_ALLOW_PROTOCOL", "file")
        .env("GIT_AUTHOR_NAME", "fixture")
        .env("GIT_COMMITTER_NAME", "fixture")
        .env("GIT_AUTHOR_EMAIL", "test@localhost")
        .env("GIT_COMMITTER_EMAIL", "test@localhost")
        .env("GITHUB_SHA", "fixture")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn ok(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn git(dir: &Path, args: &[&str]) -> String {
    ok(run(dir, "git", args))
}

fn pages_fixture(root: &Path) -> String {
    git(root, &["init", "-q", "--bare", "origin.git"]);
    git(root, &["init", "-q", "-b", "gh-pages", "site"]);
    let site = root.join("site");
    fs::create_dir(site.join("studio")).unwrap();
    fs::write(site.join("studio/index.html"), "Studio").unwrap();
    fs::write(site.join("old.html"), "old book").unwrap();
    git(&site, &["add", "."]);
    git(&site, &["commit", "-qm", "initial Pages"]);
    git(
        &site,
        &[
            "remote",
            "add",
            "origin",
            root.join("origin.git").to_str().unwrap(),
        ],
    );
    for dest in ["gh-pages", "archive", "main"] {
        git(
            &site,
            &["push", "-q", "origin", &format!("HEAD:refs/heads/{dest}")],
        );
    }
    git(&site, &["rev-parse", "HEAD"])
}

#[test]
fn only_explicit_manual_main_dispatch_can_write_pages() {
    let (build, publish) = DOCS.split_once("\n  publish:\n").unwrap();
    assert!(build.contains("  push:\n    branches: [main]"));
    assert!(
        build.contains("          - build-only\n          - gh-pages\n        default: build-only")
    );
    assert!(build.contains("\npermissions:\n  contents: read\n"));
    assert!(!build.contains("contents: write"));
    assert!(build.contains("persist-credentials: false"));
    let (_, upload) = build
        .split_once("      - name: Pass the checked book to the manual publisher\n")
        .unwrap();
    assert!(upload.starts_with(&format!("        if: {GATE}\n")));
    assert!(upload.contains("path: checked-book.tar\n"));
    assert!(publish.contains(&format!(
        "    needs: site\n    if: {GATE}\n    permissions:\n      contents: write\n"
    )));
    assert!(publish.contains("ref: refs/heads/gh-pages"));
    assert!(publish.contains("name: checked-book\n          path: incoming"));
}

#[test]
fn both_cleanup_guards_match_the_registry() {
    let registry: serde_json::Value = serde_json::from_str(REGISTRY).unwrap();
    let mut permanent = registry["protected"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    permanent.sort();
    assert_eq!(permanent, ["archive", "gh-pages", "main"]);
    for (job, var) in [("merged-head", "BRANCH"), ("sweep", "name")] {
        let (_, body) = CLEANUP.split_once(&format!("\n  {job}:\n")).unwrap();
        let (guard, _) = body.split_once("--force-with-lease=").unwrap();
        let (_, names) = guard.split_once(&format!("case \"${var}\" in ")).unwrap();
        let mut names = names
            .split_once(')')
            .unwrap()
            .0
            .split('|')
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(names, permanent);
    }
}

#[test]
fn packaging_rejects_metadata_links_and_reserved_roots() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let book = root.join("book");
    fs::create_dir(&book).unwrap();
    fs::write(book.join("index.html"), "book").unwrap();
    let package = shell_step("Package the checked book");
    for path in ["studio", ".git", ".nested/.git"] {
        fs::create_dir_all(book.join(path)).unwrap();
        assert!(
            !run(root, "bash", &["-c", &package]).status.success(),
            "{path}"
        );
        fs::remove_dir(book.join(path)).unwrap();
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root, book.join("link")).unwrap();
        assert!(!run(root, "bash", &["-c", &package]).status.success());
    }
}

#[test]
fn hidden_asset_survives_the_artifact_handoff_and_publish_preserves_studio() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let original = pages_fixture(root);
    let source = root.join("book");
    fs::create_dir_all(source.join(".assets")).unwrap();
    fs::write(
        source.join("index.html"),
        r#"<img src=".assets/diagram.svg">"#,
    )
    .unwrap();
    fs::write(source.join(".assets/diagram.svg"), "<svg/>").unwrap();
    ok(run(
        root,
        "bash",
        &["-c", &shell_step("Package the checked book")],
    ));
    fs::rename(&source, root.join("original-runner-book")).unwrap();
    fs::create_dir(root.join("incoming")).unwrap();
    fs::rename(
        root.join("checked-book.tar"),
        root.join("incoming/checked-book.tar"),
    )
    .unwrap();
    ok(run(
        root,
        "bash",
        &["-c", &shell_step("Unpack the checked book")],
    ));
    let remote = root.join("origin.git");
    let publish = shell_step("Publish");
    for name in ["studio", ".git"] {
        fs::create_dir(source.join(name)).unwrap();
        assert!(!run(root, "bash", &["-c", &publish]).status.success());
        assert_eq!(git(&remote, &["rev-parse", "gh-pages"]), original);
        fs::remove_dir(source.join(name)).unwrap();
    }
    ok(run(root, "bash", &["-c", &publish]));
    assert_eq!(
        git(&remote, &["show", "gh-pages:.assets/diagram.svg"]),
        "<svg/>"
    );
    assert_eq!(
        git(&remote, &["rev-parse", "gh-pages:studio"]),
        git(&remote, &["rev-parse", &format!("{original}:studio")])
    );
    assert!(!git(&remote, &["ls-tree", "--name-only", "gh-pages"]).contains("old.html"));
    assert_eq!(
        git(
            &remote,
            &["for-each-ref", "--format=%(refname:short)", "refs/heads"]
        ),
        "archive\ngh-pages\nmain"
    );
    for name in ["main", "archive"] {
        assert_eq!(git(&remote, &["rev-parse", name]), original);
    }
}

#[test]
fn studio_requires_an_explicit_live_target_and_forbids_the_archive() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for target in ["archive", "gh-pages-preview", "main"] {
        let out = run(
            &root,
            "bash",
            &[
                "scripts/studio/build-and-publish.sh",
                "--push",
                "--yes",
                "--target",
                target,
            ],
        );
        assert_eq!(out.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&out.stderr).contains("--target must be gh-pages"));
    }
    let out = run(
        &root,
        "bash",
        &["scripts/studio/build-and-publish.sh", "--push", "--yes"],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("explicit --target gh-pages"));
}

fn studio_publish_shell(previous: &str) -> String {
    let (before, rest) = STUDIO.split_once("publish_studio() {\n").unwrap();
    let (body, after) = rest.split_once("\n}\n").unwrap();
    assert!(!before.contains("--force-with-lease") && !after.contains("--force-with-lease"));
    assert!(after.contains("if [ \"$push\" = 1 ]; then\n        publish_studio "));
    format!("set -eu\nsay() {{ echo \"$*\"; }}\npublish_studio() {{\n{body}\n}}\npublish_studio . {previous}")
}

#[test]
fn a_reported_rollback_cannot_overwrite_a_newer_pages_publication() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let previous = pages_fixture(root);
    let site = root.join("site");
    git(&site, &["commit", "--allow-empty", "-qm", "new Studio"]);
    let published = git(&site, &["rev-parse", "HEAD"]);
    let shell = studio_publish_shell(&previous);
    let published_message = ok(run(&site, "bash", &["-c", &shell]));
    let rollback = published_message
        .lines()
        .find_map(|line| line.strip_prefix("rollback if needed: "))
        .unwrap();
    assert_eq!(rollback, format!("git push --force-with-lease=refs/heads/gh-pages:{published} origin {previous}:refs/heads/gh-pages"));
    // Even a refreshed origin/gh-pages cannot loosen the printed explicit lease.
    git(&site, &["commit", "--allow-empty", "-qm", "later Pages"]);
    git(&site, &["push", "origin", "HEAD:refs/heads/gh-pages"]);
    git(&site, &["fetch", "origin", "gh-pages"]);
    assert!(!run(&site, "bash", &["-c", rollback]).status.success());
    assert_eq!(
        git(&root.join("origin.git"), &["rev-parse", "gh-pages"]),
        git(&site, &["rev-parse", "HEAD"])
    );
    git(&site, &["remote", "remove", "origin"]);
    let failed = run(&site, "bash", &["-c", &shell]);
    assert!(!failed.status.success());
    assert!(!String::from_utf8_lossy(&failed.stdout).contains("rollback"));
}

// Interleave the fixture action after the actual book shell commits and before its push.
fn publish_book_with_post_commit(root: &Path, action: &str) -> Output {
    fs::create_dir(root.join("book")).unwrap();
    fs::write(root.join("book/index.html"), "new book").unwrap();
    let hook = root.join("site/.git/hooks/post-commit");
    fs::write(&hook, format!("#!/bin/sh\n{action}\n")).unwrap();
    ok(run(root, "chmod", &["+x", hook.to_str().unwrap()]));
    run(root, "bash", &["-c", &shell_step("Publish")])
}

#[test]
fn neither_in_flight_publisher_can_restore_a_rolled_back_studio() {
    for publisher in ["book", "Studio"] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let original = pages_fixture(root);
        let site = root.join("site");
        let remote = root.join("origin.git");
        fs::write(site.join("studio/index.html"), "rolled-back Studio").unwrap();
        git(&site, &["commit", "-qam", "publish new Studio"]);
        let newer = git(&site, &["rev-parse", "HEAD"]);
        let published = ok(run(
            &site,
            "bash",
            &["-c", &studio_publish_shell(&original)],
        ));
        let rollback = published
            .lines()
            .find_map(|line| line.strip_prefix("rollback if needed: "))
            .unwrap();

        let stale = if publisher == "book" {
            publish_book_with_post_commit(root, rollback)
        } else {
            git(&site, &["commit", "--allow-empty", "-qm", "staged Studio"]);
            ok(run(&site, "bash", &["-c", rollback]));
            run(&site, "bash", &["-c", &studio_publish_shell(&newer)])
        };
        assert_eq!(git(&site, &["rev-parse", "HEAD^"]), newer);
        assert!(
            !stale.status.success(),
            "{publisher} revived a rolled-back publication"
        );
        assert_eq!(
            git(&remote, &["rev-parse", "gh-pages"]),
            original,
            "{publisher}"
        );
        assert_eq!(
            git(&remote, &["show", "gh-pages:studio/index.html"]),
            "Studio"
        );
    }
}

#[test]
fn a_forward_lease_never_authorizes_a_non_linear_pages_commit() {
    for publisher in ["book", "Studio"] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let original = pages_fixture(root);
        let site = root.join("site");
        let other_parent = git(&site, &["commit-tree", "HEAD^{tree}", "-m", "other root"]);
        let merge = git(
            &site,
            &[
                "commit-tree",
                "HEAD^{tree}",
                "-p",
                &original,
                "-p",
                &other_parent,
                "-m",
                "merge",
            ],
        );
        let refused = if publisher == "book" {
            publish_book_with_post_commit(root, &format!("git update-ref HEAD {merge}"))
        } else {
            git(&site, &["update-ref", "HEAD", &merge]);
            run(&site, "bash", &["-c", &studio_publish_shell(&original)])
        };
        assert_eq!(git(&site, &["rev-parse", "HEAD"]), merge);
        assert!(
            !refused.status.success(),
            "{publisher} published a merge commit"
        );
        assert_eq!(
            git(&root.join("origin.git"), &["rev-parse", "gh-pages"]),
            original
        );
    }
}

#[test]
fn git_control_files_are_refused_before_upload_and_again_before_publication() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let original = pages_fixture(root);
    let book = root.join("book");
    fs::create_dir_all(book.join(".assets")).unwrap();
    fs::write(
        book.join("index.html"),
        r#"<img src=".assets/diagram.svg">"#,
    )
    .unwrap();
    fs::write(book.join(".assets/diagram.svg"), "<svg/>").unwrap();
    let package = shell_step("Package the checked book");
    let publish = shell_step("Publish");
    for parent in [&book, &book.join(".assets")] {
        for (name, contents) in [
            (".gitignore", "diagram.svg\n"),
            (".gitattributes", "*.svg text eol=lf\n"),
            (".gitmodules", ""),
            (".git", "gitdir: elsewhere\n"),
        ] {
            let path = parent.join(name);
            fs::write(&path, contents).unwrap();
            for (boundary, shell) in [("upload", &package), ("publication", &publish)] {
                let attempted = run(root, "bash", &["-c", shell]);
                assert!(
                    !attempted.status.success(),
                    "{boundary} accepted {}",
                    path.display()
                );
                assert_eq!(
                    git(&root.join("origin.git"), &["rev-parse", "gh-pages"]),
                    original
                );
            }
            fs::remove_file(path).unwrap();
        }
    }
}

#[test]
fn staged_book_files_must_match_checked_bytes_even_with_local_git_settings() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let original = pages_fixture(root);
    let book = root.join("book");
    let image = b"<svg>\r\n</svg>\r\n";
    fs::create_dir_all(book.join(".assets")).unwrap();
    fs::write(
        book.join("index.html"),
        r#"<img src=".assets/diagram.svg">"#,
    )
    .unwrap();
    fs::write(book.join(".assets/diagram.svg"), image).unwrap();
    let remote = root.join("origin.git");
    let publish = shell_step("Publish");
    for (name, value) in [
        ("exclude", "diagram.svg\n"),
        ("attributes", "*.svg text eol=lf\n"),
    ] {
        let setting = root.join("site/.git/info").join(name);
        fs::write(&setting, value).unwrap();
        assert!(
            !run(root, "bash", &["-c", &publish]).status.success(),
            "unchecked {name} changed the published tree"
        );
        assert_eq!(git(&remote, &["rev-parse", "gh-pages"]), original);
        fs::remove_file(setting).unwrap();
    }
    ok(run(root, "bash", &["-c", &publish]));
    let blob = run(&remote, "git", &["show", "gh-pages:.assets/diagram.svg"]);
    assert!(blob.status.success());
    assert_eq!(blob.stdout, image);
}
