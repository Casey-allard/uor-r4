#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uor_r4_desktop_client::{
    build_synthetic_stack_artifact, sha256_bytes, DesktopBackend, DesktopClientError, PinRegistry,
    TerminalCommandRequest, SYNTHETIC_STANDIN_FILE_NAME, SYNTHETIC_STANDIN_SEED,
};

fn temp_dir(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("uor-review-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create regression fixture");
    path
}

fn approved_request(cwd: &Path, command: &str) -> TerminalCommandRequest {
    TerminalCommandRequest {
        cwd: cwd.to_path_buf(),
        command: command.to_string(),
        user_confirmed: true,
        confirmed_command: command.to_string(),
    }
}

#[test]
fn attached_file_options_cannot_follow_a_symlink_outside_the_grant() {
    let root = temp_dir("attached-file-option");
    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace");
    fs::write(root.join("outside.txt"), "outside-secret\n").expect("write outside fixture");
    fs::write(workspace.join("input"), "input\n").expect("write input");
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("outside.txt"), workspace.join("secret"))
        .expect("outside symlink");
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&workspace)
        .expect("grant workspace");
    backend.native_set_terminal_enabled(true);
    backend
        .host_approve_terminal_command(&workspace, "grep -fsecret input")
        .expect("approve exact command");
    let error = backend
        .native_run_terminal_command(&approved_request(&workspace, "grep -fsecret input"))
        .expect_err("attached file option must not bypass path validation");
    assert!(
        matches!(
            error,
            DesktopClientError::InvalidInput(_)
                | DesktopClientError::SymlinkRefused(_)
                | DesktopClientError::PathNotGranted(_)
        ),
        "{error:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn terminal_git_uses_the_same_hardened_configuration_as_changes() {
    let root = temp_dir("terminal-git-hook");
    let marker = root.join("hook-ran");
    let hook = root.join("fsmonitor.sh");
    fs::write(&hook, format!("#!/bin/sh\ntouch '{}'\n", marker.display())).expect("write hook");
    let mut permissions = fs::metadata(&hook).expect("hook metadata").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(&hook, permissions).expect("make hook executable");
    let init = Command::new("git")
        .arg("init")
        .current_dir(&root)
        .output()
        .expect("init repository");
    assert!(init.status.success());
    let config = Command::new("git")
        .args(["config", "core.fsmonitor"])
        .arg(&hook)
        .current_dir(&root)
        .output()
        .expect("configure fsmonitor");
    assert!(config.status.success());
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&root)
        .expect("grant repository");
    backend.native_set_terminal_enabled(true);
    backend
        .host_approve_terminal_command(&root, "git status --porcelain")
        .expect("approve git status");
    backend
        .native_run_terminal_command(&approved_request(&root, "git status --porcelain"))
        .expect("hardened git status");
    assert!(
        !marker.exists(),
        "a repository-controlled fsmonitor command ran"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn terminal_consent_rejects_visual_control_characters_before_the_dialog() {
    let root = temp_dir("terminal-control");
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend.native_grant_folder(&root).expect("grant cwd");
    backend.native_set_terminal_enabled(true);
    let error = backend
        .host_approve_terminal_command(&root, "echo safe\u{202e}evil")
        .expect_err("bidi formatting must be refused");
    assert!(
        matches!(error, DesktopClientError::InvalidInput(_)),
        "{error:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn durable_restore_rejects_scopes_symlink_before_reading() {
    let root = temp_dir("restore-scope-symlink");
    let store = root.join("store");
    let restored_store = root.join("restored-store");
    let outside = root.join("outside");
    fs::create_dir_all(&restored_store).expect("create restored store");
    fs::create_dir_all(&outside).expect("create outside directory");
    let bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    let sha = sha256_bytes(&bytes);
    let mut first = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    first.native_grant_folder(&root).expect("grant store root");
    first
        .set_durable_store_dir(&store)
        .expect("configure original store");
    first
        .native_load_model_bytes(SYNTHETIC_STANDIN_FILE_NAME, &bytes, Some(&sha))
        .expect("load original model");
    first
        .native_step_tokens("scope", &[0], 1, |_| {})
        .expect("step original model");
    let saved = first
        .native_save_session_durable("local-user", "studio", "scope")
        .expect("save original scope");
    let entries = fs::read_dir(store.join("scopes"))
        .expect("list manifests")
        .collect::<Result<Vec<_>, _>>()
        .expect("manifest entries");
    fs::copy(entries[0].path(), outside.join("manifest.uorman01"))
        .expect("copy valid manifest outside");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, restored_store.join("scopes"))
        .expect("create scopes symlink");
    let mut restored = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    restored
        .native_grant_folder(&root)
        .expect("grant restored store root");
    restored
        .set_durable_store_dir(&restored_store)
        .expect("configure restored store");
    restored
        .native_load_model_bytes(SYNTHETIC_STANDIN_FILE_NAME, &bytes, Some(&sha))
        .expect("load restored model");
    assert!(!saved.manifest_bytes.is_empty());
    let error = restored
        .native_restore_session_durable("local-user", "studio", "scope", None)
        .expect_err("restore must not follow the scopes symlink");
    assert!(
        matches!(
            error,
            DesktopClientError::SymlinkRefused(_) | DesktopClientError::PathNotGranted(_)
        ),
        "{error:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn durable_scope_filenames_do_not_collapse_distinct_identities() {
    let root = temp_dir("scope-name-collision");
    let store = root.join("store");
    let bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    let sha = sha256_bytes(&bytes);
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&root)
        .expect("grant store root");
    backend
        .set_durable_store_dir(&store)
        .expect("configure store");
    backend
        .native_load_model_bytes(SYNTHETIC_STANDIN_FILE_NAME, &bytes, Some(&sha))
        .expect("load model");
    for conversation in ["a/b", "a_b"] {
        backend
            .native_step_tokens(conversation, &[0], 1, |_| {})
            .expect("step scoped model");
        backend
            .native_save_session_durable("local-user", "studio", conversation)
            .expect("save distinct scope");
    }
    let names = fs::read_dir(store.join("scopes"))
        .expect("list scope files")
        .map(|entry| entry.expect("scope entry").file_name())
        .collect::<Vec<_>>();
    assert_eq!(
        names.len(),
        2,
        "distinct scopes overwrote one manifest: {names:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn legacy_static_handler_rejects_absolute_double_slash_paths() {
    let root = temp_dir("legacy-absolute-path");
    let studio = root.join("studio");
    fs::create_dir_all(&studio).expect("create studio root");
    fs::write(studio.join("index.html"), "<html></html>").expect("write index");
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let (status, _, _) = uor_r4_desktop_client::handle_studio_http_request(
        &mut backend,
        &studio,
        "127.0.0.1:1420",
        "launch-token",
        "GET",
        "//etc/passwd",
        &[("Host".to_string(), "127.0.0.1:1420".to_string())],
        b"",
    );
    assert_eq!(status, 403, "an absolute path escaped the Studio root");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn git_clean_filter_cannot_execute_from_a_granted_repository() {
    let root = temp_dir("git-clean-filter");
    let marker = root.join("clean-filter-ran");
    let hook = root.join("filter.sh");
    fs::write(
        &hook,
        format!("#!/bin/sh\ntouch '{}'\ncat\n", marker.display()),
    )
    .expect("write filter");
    let mut permissions = fs::metadata(&hook).expect("filter metadata").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(&hook, permissions).expect("make filter executable");
    let run_git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .expect("run fixture git");
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run_git(&["init"]);
    run_git(&["config", "user.email", "studio@example.invalid"]);
    run_git(&["config", "user.name", "Studio Fixture"]);
    run_git(&["config", "filter.attack.clean", &hook.to_string_lossy()]);
    fs::write(root.join(".gitattributes"), "tracked.txt filter=attack\n")
        .expect("write attributes");
    fs::write(root.join("tracked.txt"), "before\n").expect("write tracked file");
    run_git(&["add", "tracked.txt", ".gitattributes"]);
    run_git(&["commit", "-m", "fixture"]);
    fs::write(root.join("tracked.txt"), "after\n").expect("modify tracked file");
    let _ = fs::remove_file(&marker);
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&root)
        .expect("grant repository");
    let error = backend
        .native_git_diff(&root)
        .expect_err("configured filter drivers must be refused explicitly");
    assert!(
        matches!(
            error,
            DesktopClientError::InvalidInput(_) | DesktopClientError::CommandFailed(_)
        ),
        "{error:?}"
    );
    assert!(!marker.exists(), "a repository-local clean filter executed");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn legacy_static_handler_rejects_a_symlinked_parent_outside_studio() {
    let root = temp_dir("legacy-symlink-parent");
    let studio = root.join("studio");
    let outside = root.join("outside");
    fs::create_dir_all(&studio).expect("create studio root");
    fs::create_dir_all(&outside).expect("create outside directory");
    fs::write(studio.join("index.html"), "<html></html>").expect("write index");
    fs::write(outside.join("secret.txt"), "outside-secret\n").expect("write secret");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, studio.join("linked")).expect("create parent symlink");
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let (status, _, body) = uor_r4_desktop_client::handle_studio_http_request(
        &mut backend,
        &studio,
        "127.0.0.1:1420",
        "launch-token",
        "GET",
        "/linked/secret.txt",
        &[("Host".to_string(), "127.0.0.1:1420".to_string())],
        b"",
    );
    assert_eq!(status, 403, "a symlinked parent escaped the Studio root");
    assert_eq!(body, b"Forbidden", "outside content was returned: {body:?}");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn git_filter_guard_covers_included_and_worktree_configuration() {
    for worktree in [false, true] {
        let root = temp_dir(if worktree {
            "git-worktree-filter"
        } else {
            "git-included-filter"
        });
        let marker = root.join("filter-ran");
        let hook = root.join("filter.sh");
        fs::write(
            &hook,
            format!("#!/bin/sh\ntouch '{}'\ncat\n", marker.display()),
        )
        .expect("write filter");
        let mut permissions = fs::metadata(&hook).expect("filter metadata").permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&hook, permissions).expect("make filter executable");
        let run_git = |args: &[&str]| {
            let output = Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .expect("run fixture git");
            assert!(
                output.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        run_git(&["init"]);
        run_git(&["config", "user.email", "studio@example.invalid"]);
        run_git(&["config", "user.name", "Studio Fixture"]);
        fs::write(root.join(".gitattributes"), "tracked.txt filter=attack\n")
            .expect("write attributes");
        fs::write(root.join("tracked.txt"), "before\n").expect("write tracked file");
        run_git(&["add", "tracked.txt", ".gitattributes"]);
        run_git(&["commit", "-m", "fixture"]);
        let filter_config = format!("[filter \"attack\"]\n\tclean = {}\n", hook.display());
        if worktree {
            run_git(&["config", "extensions.worktreeConfig", "true"]);
            fs::write(root.join(".git/config.worktree"), filter_config)
                .expect("write worktree filter");
        } else {
            let included = root.join("included-filter-config");
            fs::write(&included, filter_config).expect("write included filter");
            run_git(&["config", "include.path", &included.to_string_lossy()]);
        }
        fs::write(root.join("tracked.txt"), "after\n").expect("modify tracked file");
        let _ = fs::remove_file(&marker);
        let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
        backend
            .native_grant_folder(&root)
            .expect("grant repository");
        let error = backend.native_git_diff(&root).expect_err(if worktree {
            "worktree filter configuration must be refused"
        } else {
            "included filter configuration must be refused"
        });
        assert!(
            matches!(
                error,
                DesktopClientError::InvalidInput(_) | DesktopClientError::CommandFailed(_)
            ),
            "{error:?}"
        );
        assert!(
            !marker.exists(),
            "included/worktree filter executed (worktree={worktree})"
        );
        let _ = fs::remove_dir_all(&root);
    }
}

#[test]
fn git_diff_and_repository_config_writes_are_serialized() {
    let root = temp_dir("git-filter-race");
    let marker = root.join("race-filter-ran");
    let hook = root.join("filter.sh");
    fs::write(
        &hook,
        format!("#!/bin/sh\ntouch '{}'\ncat\n", marker.display()),
    )
    .expect("write filter");
    let mut permissions = fs::metadata(&hook).expect("filter metadata").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(&hook, permissions).expect("make filter executable");
    let run_git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .expect("run fixture git");
        assert!(output.status.success(), "git {args:?} failed");
    };
    run_git(&["init"]);
    run_git(&["config", "user.email", "studio@example.invalid"]);
    run_git(&["config", "user.name", "Studio Fixture"]);
    fs::write(root.join(".gitattributes"), "tracked.txt filter=attack\n")
        .expect("write attributes");
    fs::write(root.join("tracked.txt"), "before\n").expect("write tracked file");
    run_git(&["add", "tracked.txt", ".gitattributes"]);
    run_git(&["commit", "-m", "fixture"]);
    fs::write(root.join("tracked.txt"), "after\n").expect("modify tracked file");
    let _ = fs::remove_file(&marker);
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&root)
        .expect("grant repository");
    let stop = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|scope| {
        let writer = scope.spawn(|| {
            let benign = "[core]\n\trepositoryformatversion = 0\n\tbare = false\n\tlogallrefupdates = true\n";
            let malicious = format!(
                "[core]\n\trepositoryformatversion = 0\n\tbare = false\n\tlogallrefupdates = true\n[filter \"attack\"]\n\tclean = {}\n",
                hook.display()
            );
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = backend.native_write_file(&root.join(".git/config"), benign);
                let _ = backend.native_write_file(&root.join(".git/config"), &malicious);
                std::thread::yield_now();
            }
        });
        for _ in 0..400 {
            let _ = backend.native_git_diff(&root);
            if marker.exists() {
                break;
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        writer.join().expect("join config writer");
    });
    assert!(
        !marker.exists(),
        "a filter was installed between the guard and real Git diff"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn rg_pre_and_git_configuration_overrides_are_refused_before_execution() {
    for command in ["rg --pre ./hook pattern", "rg --pre=./hook pattern"] {
        let root = temp_dir("rg-pre-exec");
        let marker = root.join("rg-pre-ran");
        let hook = root.join("hook");
        fs::write(&hook, format!("#!/bin/sh\ntouch '{}'\n", marker.display()))
            .expect("write rg pre fixture");
        let mut permissions = fs::metadata(&hook).expect("hook metadata").permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&hook, permissions).expect("make hook executable");
        let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
        backend.native_grant_folder(&root).expect("grant project");
        backend.native_set_terminal_enabled(true);
        backend
            .host_approve_terminal_command(&root, command)
            .expect("approve exact command");
        let error = backend
            .native_run_terminal_command(&approved_request(&root, command))
            .expect_err("rg --pre must be refused before execution");
        assert!(
            matches!(error, DesktopClientError::InvalidInput(_)),
            "{error:?}"
        );
        assert!(
            !marker.exists(),
            "rg --pre executed project code: {command}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    let root = temp_dir("git-config-exec");
    let marker = root.join("git-config-ran");
    let hook = root.join("hook");
    fs::write(&hook, format!("#!/bin/sh\ntouch '{}'\n", marker.display()))
        .expect("write fsmonitor fixture");
    let mut permissions = fs::metadata(&hook).expect("hook metadata").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(&hook, permissions).expect("make hook executable");
    let init = Command::new("git")
        .arg("init")
        .current_dir(&root)
        .output()
        .expect("init repository");
    assert!(init.status.success());
    let command = "git -c core.fsmonitor=./hook status --porcelain";
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&root)
        .expect("grant repository");
    backend.native_set_terminal_enabled(true);
    backend
        .host_approve_terminal_command(&root, command)
        .expect("approve exact command");
    let error = backend
        .native_run_terminal_command(&approved_request(&root, command))
        .expect_err("Git configuration overrides must be refused");
    assert!(
        matches!(error, DesktopClientError::InvalidInput(_)),
        "{error:?}"
    );
    assert!(!marker.exists(), "Git -c re-enabled an execution hook");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn git_exec_path_override_is_refused_before_execution() {
    let root = temp_dir("git-exec-path");
    let command = "git --exec-path=/tmp status --porcelain";
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&root)
        .expect("grant repository");
    backend.native_set_terminal_enabled(true);
    backend
        .host_approve_terminal_command(&root, command)
        .expect("approve exact command");
    let error = backend
        .native_run_terminal_command(&approved_request(&root, command))
        .expect_err("Git exec-path override must be refused");
    assert!(
        matches!(error, DesktopClientError::InvalidInput(_)),
        "{error:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn attached_git_config_and_worktree_overrides_are_refused_before_execution() {
    for command in [
        "git -ccore.fsmonitor=./hook status --porcelain",
        "git -C/tmp status --porcelain",
    ] {
        let root = temp_dir("attached-git-overrides");
        let marker = root.join("attached-git-ran");
        let hook = root.join("hook");
        fs::write(&hook, format!("#!/bin/sh\ntouch '{}'\n", marker.display()))
            .expect("write attached override fixture");
        let mut permissions = fs::metadata(&hook).expect("hook metadata").permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&hook, permissions).expect("make hook executable");
        let init = Command::new("git")
            .arg("init")
            .current_dir(&root)
            .output()
            .expect("init repository");
        assert!(init.status.success());
        let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
        backend
            .native_grant_folder(&root)
            .expect("grant repository");
        backend.native_set_terminal_enabled(true);
        backend
            .host_approve_terminal_command(&root, command)
            .expect("approve exact command");
        let error = backend
            .native_run_terminal_command(&approved_request(&root, command))
            .expect_err("attached Git overrides must be refused");
        assert!(
            matches!(error, DesktopClientError::InvalidInput(_)),
            "{error:?}"
        );
        assert!(
            !marker.exists(),
            "attached Git override executed: {command}"
        );
        let _ = fs::remove_dir_all(&root);
    }
}
