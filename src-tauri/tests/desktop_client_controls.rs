//! Host control tests for the UOR-R4 desktop client backend (`infra-desktop-client-tauri`, #2826).
#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde_json::json;
use uor_r4_desktop_client::{
    build_synthetic_stack_artifact, dispatch_rpc_command, generate_launch_token, handle_rpc_line,
    handle_studio_http_request, run_self_test_receipt, sha256_bytes, validate_terminal_program,
    DesktopBackend, DesktopClientError, PinRegistry, TerminalCommandRequest,
    SYNTHETIC_STANDIN_FILE_NAME, SYNTHETIC_STANDIN_SEED,
};

const EXPECTED_SYNTHETIC_32: [u32; 32] = [
    38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    30, 30, 30, 30, 30, 30, 30, 30,
];

fn unique_temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uor-tauri-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn native_list_local_models_returns_synthetic_standin_and_refuses_tampered_hash() {
    let tmp = unique_temp_dir("models");

    let standin_bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    assert_eq!(
        standin_bytes.len(),
        469_252,
        "seed-5 synthetic stand-in must match mk-synthetic-stack byte length"
    );
    let standin_sha = sha256_bytes(&standin_bytes);
    let standin_path = tmp.join(SYNTHETIC_STANDIN_FILE_NAME);
    fs::write(&standin_path, &standin_bytes).expect("write synthetic stand-in");

    // Place an unrelated .bin and .gguf file alongside the pinned stand-in:
    // they must NOT hide the pinned model during discovery.
    let unrelated_bin = tmp.join("unrelated.bin");
    fs::write(&unrelated_bin, b"GGUF_NOT_UORLUT01_PAYLOAD_12345678").expect("write unrelated bin");

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend.add_model_search_dir(&tmp);
    backend.add_model_search_dir(&tmp);
    assert_eq!(
        backend.model_search_dirs().len(),
        1,
        "add_model_search_dir must deduplicate repeated directory registrations"
    );
    backend
        .native_grant_folder(&tmp)
        .expect("grant model directory");

    let listed = backend
        .native_list_local_models()
        .expect("list local models with valid pinned stand-in and unrelated .bin");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, SYNTHETIC_STANDIN_FILE_NAME);
    assert_eq!(listed[0].format, "UORLUT01");
    assert_eq!(listed[0].size_bytes, 469_252);
    assert_eq!(listed[0].sha256, standin_sha);
    assert!(listed[0].synthetic);
    assert_eq!(listed[0].status_label, "synthetic stand-in");

    // Flip one byte in the payload so the SHA-256 differs from its pin while the UORLUT01 magic stays intact.
    let mut tampered = standin_bytes.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0x01;
    fs::write(&standin_path, &tampered).expect("write tampered stand-in");

    let after_tamper = backend
        .native_list_local_models()
        .expect("list local models skips tampered file");
    assert!(
        after_tamper.is_empty(),
        "tampered model must not be listed as a valid pinned model"
    );

    let err = backend
        .native_load_model(&standin_path, None)
        .expect_err("tampered model file whose hash differs from its pin must be refused");
    assert!(
        matches!(err, DesktopClientError::UnpinnedOrTamperedModel { .. }),
        "expected UnpinnedOrTamperedModel, got {err:?}"
    );

    // Also verify non-UORLUT01 model formats are refused before format.rs.
    let fmt_err = backend
        .native_load_model(&unrelated_bin, None)
        .expect_err("non-UORLUT01 file must be refused");
    assert!(
        matches!(fmt_err, DesktopClientError::UnsupportedModelFormat(_)),
        "expected UnsupportedModelFormat, got {fmt_err:?}"
    );

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn native_engine_step_tokens_matches_reference_32_token_trajectory() {
    let standin_bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    let standin_sha = sha256_bytes(&standin_bytes);

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let caps = backend
        .native_load_model_bytes(
            SYNTHETIC_STANDIN_FILE_NAME,
            &standin_bytes,
            Some(&standin_sha),
        )
        .expect("load pinned synthetic stand-in");
    assert_eq!(caps.weights_per_token, 15_968);
    assert_eq!(caps.shape.vocab, 40);
    assert_eq!(caps.shape.width, 32);
    assert!(caps.synthetic);

    let mut streamed_tokens = Vec::new();
    let receipt = backend
        .native_step_tokens("conv-32", &[0], 32, |ev| {
            streamed_tokens.push(ev.token_id);
        })
        .expect("step 32 tokens on synthetic stand-in");

    assert_eq!(receipt.token_ids, EXPECTED_SYNTHETIC_32);
    assert_eq!(streamed_tokens, EXPECTED_SYNTHETIC_32);
    assert_eq!(receipt.checkpoint_bytes, 9_782);
}

#[test]
fn durable_memory_object_store_close_and_resume_continues_token_for_token() {
    let root = unique_temp_dir("durable-store");
    let store_dir = root.join("store");
    let ungranted_store = root.join("outside-store");
    let standin_bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    let standin_sha = sha256_bytes(&standin_bytes);

    // 1. Run the first 16 tokens in backend_a and persist to uor-r4-object-store on disk.
    let first_16_tokens: Vec<u32>;
    let save_receipt = {
        let mut backend_a = DesktopBackend::new(PinRegistry::with_synthetic_standin());
        // Setting store_dir before granting must be refused.
        let ungranted_err = backend_a
            .set_durable_store_dir(&ungranted_store)
            .expect_err("storeDir outside granted folders must be refused");
        assert!(matches!(
            ungranted_err,
            DesktopClientError::PathNotGranted(_)
        ));

        backend_a
            .native_grant_folder(&root)
            .expect("grant root folder for store");
        backend_a
            .set_durable_store_dir(&store_dir)
            .expect("configure durable store dir inside granted folder on backend_a");
        backend_a
            .native_load_model_bytes(
                SYNTHETIC_STANDIN_FILE_NAME,
                &standin_bytes,
                Some(&standin_sha),
            )
            .expect("load synthetic stand-in in backend_a");

        let gen16 = backend_a
            .native_step_tokens("conv-resume", &[0], 16, |_| {})
            .expect("step first 16 tokens");
        assert_eq!(gen16.token_ids.len(), 16);
        assert_eq!(gen16.checkpoint_bytes, 5_366);
        first_16_tokens = gen16.token_ids;

        let saved = backend_a
            .native_save_session_durable("local-user", "studio", "conv-resume")
            .expect("save session snapshot to object store");
        assert_eq!(saved.ur4sess1_bytes_len, 5_366);
        assert!(saved.store_record_count > 0);
        saved
    };

    // 2. Start a fresh DesktopBackend process (backend_b), reload the object store from disk,
    //    restore the UR4SESS1 session, and continue 16 more tokens.
    let mut backend_b = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend_b
        .native_grant_folder(&root)
        .expect("grant root folder on backend_b");
    backend_b
        .set_durable_store_dir(&store_dir)
        .expect("reload durable store dir on backend_b");
    backend_b
        .native_load_model_bytes(
            SYNTHETIC_STANDIN_FILE_NAME,
            &standin_bytes,
            Some(&standin_sha),
        )
        .expect("load synthetic stand-in in backend_b");

    let restored = backend_b
        .native_restore_session_durable("local-user", "studio", "conv-resume", None)
        .expect("restore session from object store on disk");
    assert_eq!(restored.position, 16);
    assert_eq!(restored.ur4sess1_bytes_len, 5_366);
    assert_eq!(restored.artifact_sha256, standin_sha);

    let last_tok = first_16_tokens[15];
    let gen_second_16 = backend_b
        .native_step_tokens("conv-resume", &[last_tok], 16, |_| {})
        .expect("continue 16 tokens after object-store restore");

    let mut combined = first_16_tokens;
    combined.extend_from_slice(&gen_second_16.token_ids);
    assert_eq!(
        combined, EXPECTED_SYNTHETIC_32,
        "16 + 16 close-and-resume through uor-r4-object-store must equal uninterrupted 32-token stream"
    );

    // 3. Corrupting the manifest bytes must fail verification on restore.
    let mut bad_manifest = save_receipt.manifest_bytes.clone();
    let tail = bad_manifest.len() - 1;
    bad_manifest[tail] ^= 0xff;
    let restore_err = backend_b
        .native_restore_session_durable("local-user", "studio", "conv-resume", Some(&bad_manifest))
        .expect_err("corrupted manifest must be refused by object-store digest verification");
    assert!(matches!(restore_err, DesktopClientError::ObjectStore(_)));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn scoped_file_and_git_commands_enforce_granted_folder_and_refuse_dangling_symlinks() {
    let root = unique_temp_dir("scoped-fs");
    let granted_dir = root.join("granted-project");
    let outside_dir = root.join("outside-secret");
    fs::create_dir_all(&granted_dir).expect("create granted dir");
    fs::create_dir_all(&outside_dir).expect("create outside dir");

    let inside_file = granted_dir.join("hello.rs");
    let outside_file = outside_dir.join("secret.txt");
    let outside_new_target = outside_dir.join("escaped_write.txt");
    fs::write(&outside_file, "classified").expect("write outside file");

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());

    // Before granting, writing inside granted_dir is refused.
    let ungranted_err = backend
        .native_write_file(&inside_file, "fn main() {}\n")
        .expect_err("writing to ungranted folder must fail");
    assert!(matches!(
        ungranted_err,
        DesktopClientError::PathNotGranted(_)
    ));

    // Grant granted_dir.
    backend
        .native_grant_folder(&granted_dir)
        .expect("grant folder");

    backend
        .native_write_file(&inside_file, "fn main() {}\n")
        .expect("write inside granted folder");
    let read_back = backend
        .native_read_file(&inside_file)
        .expect("read inside granted folder");
    assert_eq!(read_back, "fn main() {}\n");

    let listed = backend
        .native_list_directory(&granted_dir, 4)
        .expect("list granted folder");
    assert_eq!(listed, vec!["hello.rs".to_string()]);

    // Reading outside_file (not in granted_folders) is refused.
    let outside_err = backend
        .native_read_file(&outside_file)
        .expect_err("reading outside granted folders must fail");
    assert!(matches!(outside_err, DesktopClientError::PathNotGranted(_)));

    // Parent-dir traversal (`..`) from inside granted_dir is refused.
    let traversal = granted_dir.join("../outside-secret/secret.txt");
    let trav_err = backend
        .native_read_file(&traversal)
        .expect_err("parent directory traversal must fail");
    assert!(matches!(
        trav_err,
        DesktopClientError::PathTraversalRefused(_)
    ));

    // Dangling symlink inside granted_dir pointing outside (`granted/dangling -> outside_new_target`)
    // MUST be refused on write and never create `outside_new_target`.
    #[cfg(unix)]
    {
        let dangling_link = granted_dir.join("dangling_escape.txt");
        std::os::unix::fs::symlink(&outside_new_target, &dangling_link)
            .expect("create dangling symlink");
        let sym_err = backend
            .native_write_file(&dangling_link, "pwned")
            .expect_err("dangling symlink write must be refused");
        assert!(
            matches!(sym_err, DesktopClientError::SymlinkRefused(_)),
            "expected SymlinkRefused, got {sym_err:?}"
        );
        assert!(
            !outside_new_target.exists(),
            "dangling symlink target outside grant must not be created"
        );
    }

    // Initialize an empty git repo inside granted_dir (no commits yet -> unborn HEAD)
    // and verify native_git_status reports "HEAD (unborn)" rather than fabricating "main".
    let git_init = Command::new("git")
        .arg("init")
        .current_dir(&granted_dir)
        .output()
        .expect("git init");
    assert!(git_init.status.success());
    let git_report = backend
        .native_git_status(&granted_dir)
        .expect("native_git_status on unborn repo");
    assert_eq!(git_report.branch, "HEAD (unborn)");
    assert!(!git_report.is_clean);
    assert!(git_report.untracked_files.contains(&"hello.rs".to_string()));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn terminal_command_requires_host_approval_and_confines_arguments_to_granted_folders() {
    let root = unique_temp_dir("terminal-guard");
    let granted_dir = root.join("workspace");
    let ungranted_dir = root.join("ungranted");
    fs::create_dir_all(&granted_dir).expect("create granted dir");
    fs::create_dir_all(&ungranted_dir).expect("create ungranted dir");
    let inside_file = granted_dir.join("note.txt");
    fs::write(&inside_file, "safe-content\n").expect("write inside file");
    let outside_file = ungranted_dir.join("secret.txt");
    fs::write(&outside_file, "secret-content\n").expect("write outside file");

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&granted_dir)
        .expect("grant workspace");

    assert!(
        !backend.terminal_enabled(),
        "terminal execution must be off by default"
    );

    let valid_req = TerminalCommandRequest {
        cwd: granted_dir.clone(),
        command: "pwd".to_string(),
        user_confirmed: true,
        confirmed_command: "pwd".to_string(),
    };

    // 1. Off by default -> refused even with user confirmation and granted folder.
    let disabled_err = backend
        .native_run_terminal_command(&valid_req)
        .expect_err("terminal command must be refused while disabled by default");
    assert_eq!(disabled_err, DesktopClientError::TerminalDisabledByDefault);

    // 2. Page RPC cannot enable the terminal (`native_set_terminal_enabled` is host-only).
    let rpc_enable = handle_rpc_line(
        &mut backend,
        r#"{"id":1,"command":"native_set_terminal_enabled","args":{"enabled":true}}"#,
    );
    assert_eq!(rpc_enable["ok"], json!(false));
    assert!(!backend.terminal_enabled());

    // 3. Host enables terminal, but page caller has not received host approval -> refused!
    backend.native_set_terminal_enabled(true);
    assert_eq!(
        backend
            .native_run_terminal_command(&valid_req)
            .expect_err("page-only user_confirmed without host approval must be refused"),
        DesktopClientError::TerminalConsentRequired
    );

    // 4. Host approves `pwd`, then execution succeeds once and consumes the one-shot approval.
    backend
        .host_approve_terminal_command(&granted_dir, "pwd")
        .expect("host approve pwd");
    let out = backend
        .native_run_terminal_command(&valid_req)
        .expect("host-approved command in granted folder succeeds");
    assert!(!out.trim().is_empty());

    // Re-running without a fresh host approval is refused.
    assert_eq!(
        backend
            .native_run_terminal_command(&valid_req)
            .expect_err("reused approval must be refused"),
        DesktopClientError::TerminalConsentRequired
    );

    // 5. Command arguments referencing `~`, `..`, or paths outside `granted_folders` are refused
    //    even if host-approved!
    for escaped_cmd in [
        "cat ~/.ssh/id_ed25519",
        "cat ../ungranted/secret.txt",
        &format!("cat {}", outside_file.display()),
    ] {
        backend
            .host_approve_terminal_command(&granted_dir, escaped_cmd)
            .expect("stage approval for escape test");
        let req = TerminalCommandRequest {
            cwd: granted_dir.clone(),
            command: escaped_cmd.to_string(),
            user_confirmed: true,
            confirmed_command: escaped_cmd.to_string(),
        };
        let err = backend
            .native_run_terminal_command(&req)
            .expect_err("argument escaping granted folder must be refused");
        assert!(
            matches!(
                err,
                DesktopClientError::PathNotGranted(_) | DesktopClientError::PathTraversalRefused(_)
            ),
            "unexpected error for {escaped_cmd:?}: {err:?}"
        );
    }

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn model_discovery_and_loading_require_granted_directories() {
    let root = unique_temp_dir("model-grants");
    let model_dir = root.join("models");
    let outside_dir = root.join("outside");
    fs::create_dir_all(&model_dir).expect("create model dir");
    fs::create_dir_all(&outside_dir).expect("create outside dir");
    let bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    let model_path = model_dir.join(SYNTHETIC_STANDIN_FILE_NAME);
    let outside_path = outside_dir.join(SYNTHETIC_STANDIN_FILE_NAME);
    fs::write(&model_path, &bytes).expect("write granted model");
    fs::write(&outside_path, &bytes).expect("write outside model");

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let err = backend
        .native_load_model(&outside_path, None)
        .expect_err("model file outside every grant must be refused before reading");
    assert!(
        matches!(err, DesktopClientError::PathNotGranted(_)),
        "{err:?}"
    );
    let refused = dispatch_rpc_command(
        &mut backend,
        "native_list_local_models",
        &json!({ "modelDir": outside_dir }),
    )
    .expect_err("page-supplied model directory outside every grant must be refused");
    assert!(refused.contains("PathNotGranted"), "{refused}");

    backend
        .grant_model_search_dir(&model_dir)
        .expect("host-granted model directory");
    let models = backend
        .native_list_local_models()
        .expect("list granted models");
    assert_eq!(models.len(), 1);
    assert_eq!(
        models[0].path,
        fs::canonicalize(&model_path)
            .expect("canonical model path")
            .to_string_lossy()
    );
    backend
        .native_load_model(&model_path, None)
        .expect("load model inside grant");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn git_status_and_diff_are_nul_safe_hardened_and_bounded() {
    let root = unique_temp_dir("bounded-git");
    let repo = root.join("repo");
    fs::create_dir_all(&repo).expect("create repo");
    let run_git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(&repo)
            .output()
            .expect("run git fixture");
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run_git(&["init"]);
    run_git(&["config", "user.email", "studio@example.invalid"]);
    run_git(&["config", "user.name", "Studio Fixture"]);
    fs::write(repo.join("tracked.txt"), "before\n").expect("write tracked file");
    run_git(&["add", "tracked.txt"]);
    run_git(&["commit", "-m", "fixture"]);
    fs::write(repo.join("tracked.txt"), "after\n").expect("modify tracked file");
    fs::write(repo.join("line\nbreak.txt"), "nul-safe status\n").expect("write newline filename");
    let marker = root.join("external-command-ran");
    let hook = repo.join("external.sh");
    fs::write(&hook, format!("#!/bin/sh\ntouch '{}'\n", marker.display()))
        .expect("write external marker hook");
    run_git(&["config", "diff.external", &hook.to_string_lossy()]);
    run_git(&["config", "core.fsmonitor", &hook.to_string_lossy()]);
    run_git(&["config", "core.pager", &hook.to_string_lossy()]);

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&repo)
        .expect("grant repository");
    let status = backend.native_git_status(&repo).expect("nul-safe status");
    assert!(
        status
            .untracked_files
            .contains(&"line\nbreak.txt".to_string()),
        "{status:?}"
    );
    let diff = backend
        .native_git_diff(&repo)
        .expect("hardened working-tree diff");
    assert!(
        diff.diff.contains("before") && diff.diff.contains("after"),
        "{}",
        diff.diff
    );
    assert!(!diff.truncated);
    assert!(
        !marker.exists(),
        "external diff, pager or fsmonitor executed"
    );

    fs::write(
        repo.join("tracked.txt"),
        format!("after\n{}\n", "x".repeat(70_000)),
    )
    .expect("write large diff");
    let bounded = backend.native_git_diff(&repo).expect("bounded diff");
    assert!(bounded.truncated, "large diff must report truncation");
    assert!(
        bounded.diff.len() <= 65_536,
        "diff exceeded its byte cap: {}",
        bounded.diff.len()
    );
    assert!(
        !marker.exists(),
        "bounded diff executed an external command"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn terminal_allowlist_and_output_cap_are_enforced() {
    for refused in ["bash", "sh", "python3", "osascript", "env", "./script.sh"] {
        assert!(
            validate_terminal_program(refused).is_err(),
            "{refused} must be refused before spawn"
        );
    }
    assert!(validate_terminal_program("pwd").is_ok());

    let root = unique_temp_dir("bounded-terminal");
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&root)
        .expect("grant terminal cwd");
    backend.native_set_terminal_enabled(true);
    let command = format!("echo {}", "x".repeat(70_000));
    backend
        .host_approve_terminal_command(&root, &command)
        .expect("approve long echo");
    let request = TerminalCommandRequest {
        cwd: root.clone(),
        command: command.clone(),
        user_confirmed: true,
        confirmed_command: command,
    };
    let output = backend
        .native_run_terminal_command(&request)
        .expect("bounded echo output");
    assert!(
        output.contains("truncated"),
        "output must say it was truncated: {output:?}"
    );
    assert!(
        output.len() < 70_000,
        "output was not bounded: {} bytes",
        output.len()
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn terminal_execution_flags_and_git_discovery_cannot_escape_grants() {
    let root = unique_temp_dir("terminal-escapes");
    let workspace = root.join("workspace");
    let repo = root.join("repo");
    fs::create_dir_all(&workspace).expect("create workspace");
    fs::create_dir_all(&repo).expect("create repository");
    let init = Command::new("git")
        .arg("init")
        .current_dir(&repo)
        .output()
        .expect("init repository");
    assert!(init.status.success());
    fs::write(repo.join("tracked.txt"), "content\n").expect("write tracked file");

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&workspace)
        .expect("grant workspace");
    backend.native_set_terminal_enabled(true);
    for command_text in [
        format!("git -C{} status", repo.display()),
        "find . -maxdepth 0 -exec pwd {} +".to_string(),
    ] {
        backend
            .host_approve_terminal_command(&workspace, &command_text)
            .expect("approve escape probe");
        let request = TerminalCommandRequest {
            cwd: workspace.clone(),
            command: command_text.clone(),
            user_confirmed: true,
            confirmed_command: command_text,
        };
        assert!(
            backend.native_run_terminal_command(&request).is_err(),
            "escape command succeeded"
        );
    }

    let nested = repo.join("src");
    fs::create_dir_all(&nested).expect("create nested directory");
    backend
        .native_grant_folder(&nested)
        .expect("grant nested directory");
    assert!(
        matches!(
            backend.native_git_status(&nested),
            Err(DesktopClientError::PathNotGranted(_))
        ),
        "parent repository discovery escaped grant"
    );
    assert!(
        matches!(
            backend.native_git_diff(&nested),
            Err(DesktopClientError::PathNotGranted(_))
        ),
        "parent diff discovery escaped grant"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn durable_store_refuses_scope_symlink_escape() {
    let root = unique_temp_dir("durable-symlink");
    let store = root.join("store");
    let outside = root.join("outside");
    fs::create_dir_all(&store).expect("create store");
    fs::create_dir_all(&outside).expect("create outside");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, store.join("scopes")).expect("create scopes symlink");

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&root)
        .expect("grant durable root");
    backend
        .set_durable_store_dir(&store)
        .expect("configure store");
    let bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    let sha = sha256_bytes(&bytes);
    backend
        .native_load_model_bytes(SYNTHETIC_STANDIN_FILE_NAME, &bytes, Some(&sha))
        .expect("load model");
    backend
        .native_step_tokens("scope-escape", &[0], 1, |_| {})
        .expect("step model");
    let error = backend
        .native_save_session_durable("local-user", "studio", "scope-escape")
        .expect_err("scope symlink must be refused");
    assert!(
        matches!(
            error,
            DesktopClientError::SymlinkRefused(_) | DesktopClientError::PathNotGranted(_)
        ),
        "{error:?}"
    );
    assert!(
        fs::read_dir(&outside)
            .expect("outside listing")
            .next()
            .is_none(),
        "durable write escaped into outside directory"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn native_exact_arithmetic_rpc_returns_a_tool_answer_or_falls_through() {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let arithmetic = dispatch_rpc_command(
        &mut backend,
        "native_exact_arithmetic",
        &json!({"model":"uor-chat","messages":[{"role":"user","content":"What is 17 + 25? Answer with the number first."}],"stream":true}),
    )
    .expect("arithmetic RPC should succeed");
    assert_eq!(arithmetic["handled"], json!(true));
    assert_eq!(arithmetic["expression"], json!("17 + 25"));
    assert_eq!(arithmetic["result"], json!("42"));
    assert!(arithmetic["text"]
        .as_str()
        .unwrap_or_default()
        .starts_with("42\n"));
    assert!(arithmetic["text"]
        .as_str()
        .unwrap_or_default()
        .contains("tool, not model generation"));

    let ordinary = dispatch_rpc_command(
        &mut backend,
        "native_exact_arithmetic",
        &json!({"model":"uor-chat","messages":[{"role":"user","content":"Hi"}],"stream":true}),
    )
    .expect("ordinary RPC should succeed");
    assert_eq!(ordinary["handled"], json!(false));

    let underflow = dispatch_rpc_command(
        &mut backend,
        "native_exact_arithmetic",
        &json!({"model":"uor-chat","messages":[{"role":"user","content":"1 - 2"}]}),
    )
    .expect("underflow RPC should succeed");
    assert_eq!(underflow["handled"], json!(true));
    assert!(underflow["error"]
        .as_str()
        .unwrap_or_default()
        .contains("underflow"));

    for prompt in [
        "What is 17 * 25?",
        "What is 17.5 + 25?",
        "What is (17 + 25)?",
        "What is 17 % 25?",
        "What is 17 ^ 25?",
        "What is ((17 + 25))?",
        "What is {17 + 25}?",
        "What is +17 * 25?",
        "What is -17 % 25?",
        "What is +17 ^ 25?",
        "What is +17 / 25?",
        "What is +17 × 25?",
        "What is +17 ÷ 25?",
        "What is .5 + 25?",
        "What is -17.5 + 25?",
        "What is (-17 * 25)?",
        "What is % 25?",
        "What is - 25?",
        "What is -?",
        "What is [17 + 25]?",
        "2026-11-05 + 1",
        "2026-11-05 - 1",
        "1 -",
        "What is 1 -?",
        "2026-11-05 -",
    ] {
        let unsupported = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("unsupported arithmetic RPC should succeed");
        assert_eq!(
            unsupported["handled"],
            json!(true),
            "fell through: {prompt}"
        );
        assert!(unsupported["error"]
            .as_str()
            .is_some_and(|error| !error.is_empty()));
    }

    for spaces in [157, 159, 160] {
        let over_limit = format!("1{}+ 1", " ".repeat(spaces));
        let over_limit_response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":over_limit}]}),
        )
        .expect("overlong arithmetic RPC should succeed");
        assert_eq!(over_limit_response["handled"], json!(true));
        assert!(over_limit_response["error"]
            .as_str()
            .unwrap_or_default()
            .contains("160-byte"));
    }

    for prompt in [
        format!("{}1 + 1", " ".repeat(200)),
        format!("1 + 1{}", " ".repeat(200)),
        format!("{}1 -", " ".repeat(200)),
        format!("{}1 - ?", " ".repeat(200)),
        format!("{}1 - Show your work", " ".repeat(200)),
        format!("{}2026-11-05 - ?", " ".repeat(200)),
        format!("{} - ?", "1".repeat(200)),
    ] {
        let padded = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("whitespace-padded arithmetic RPC should succeed");
        assert_eq!(padded["handled"], json!(true));
        assert!(padded["error"]
            .as_str()
            .unwrap_or_default()
            .contains("160-byte"));
    }

    for expression in ["-", "- 25", "- 2.5", "-?"] {
        for prompt in [
            format!("{}{}", " ".repeat(200), expression),
            format!("{}{}", expression, " ".repeat(200)),
        ] {
            let padded = dispatch_rpc_command(
                &mut backend,
                "native_exact_arithmetic",
                &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
            )
            .expect("padded leading-minus RPC should succeed");
            assert_eq!(padded["handled"], json!(true));
            assert!(padded["error"]
                .as_str()
                .unwrap_or_default()
                .contains("leading minus"));
        }
    }

    for prompt in [
        "2026-11-05",
        "2026-11-05T17:25Z",
        "2026-11-05T17:25+01:00",
        "2026-11-05T17:25-01:00",
        "2026-11-05 17:25+01:00",
        "2026-11-05T17:25Z?",
        "2026-11-05T17:25+01:00?",
        "2026-11-05T17:25-01:00?",
        "What is 2026-11-05T17:25+01:00?",
        "What is 2026-11-05T17:25Z?",
        "- this is ordinary prose",
        "- this is ordinary prose with a longer padded bullet",
        "-42 is the answer",
        "+42 is the answer",
        "- 1. Introduction",
        "1. Introduction",
        "1) Introduction",
        "1 - Introduction",
        "2026 - Year in review",
        "2026-11-05 - Year in review",
        "What is +42, really?",
        "-42,",
        "2026-11-05T17:25+01:00,",
        "2026-11-05T17:25-01:00,",
        "2026-11-05T17:25+01:00, what day is it?",
        "2026-11-05T17:25-01:00, what day is it?",
        "What is 2026-11-05T17:25+01:00 in UTC?",
        "What is 2026-11-05T17:25-01:00 in UTC?",
        "What is 2026-11-05T17:25Z in UTC?",
        "2026-11-05T17:25+01:00, 5pm",
        "2026-11-05T17:25-01:00, 5pm",
        "2026-11-05T17:25Z, 5pm",
    ] {
        let iso = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("ISO date/time RPC should succeed");
        assert_eq!(iso["handled"], json!(false), "intercepted: {prompt}");
    }
}

#[test]
fn native_exact_arithmetic_review18_boundary_regressions() {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    for prompt in [
        "2026-11-05  - 1",
        "2026-11-05, - 1",
        "2026-11-05T17:25+01:00, - 1",
        "2026-11-05  -",
        "2026-11-05 +",
        "What is 17 & 25?",
        "17 | 25",
        "17 << 2",
        "17 = 25",
        "17 (25)",
        "+1 800 555 0100 + 2",
        "10 - 20 minutes + 3",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("review arithmetic boundary RPC should succeed");
        assert_eq!(response["handled"], json!(true), "fell through: {prompt}");
        assert!(response["error"]
            .as_str()
            .is_some_and(|error| !error.is_empty()));
    }
    for prompt in [
        "2026-11-05  - 1",
        "2026-11-05, - 1",
        "2026-11-05T17:25+01:00, - 1",
        "2026-11-05 +",
    ] {
        let prompt = format!("{}{}", " ".repeat(200), prompt);
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("overlong review arithmetic boundary RPC should succeed");
        assert_eq!(response["handled"], json!(true));
        assert!(response["error"]
            .as_str()
            .unwrap_or_default()
            .contains("160-byte"));
    }
    for prompt in ["+1 800 555 0100", "+1 2 3", "10 - 20 minutes"] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("ordinary signed/range prose RPC should succeed");
        assert_eq!(response["handled"], json!(false), "intercepted: {prompt}");
    }
}

#[test]
fn native_exact_arithmetic_review20_boundary_regressions() {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    for prompt in [
        "~17",
        "17 ~ 25",
        "~17 + 25",
        "17 − 25",
        "10 - 20 minutes + 3",
        "10 - 20 (25)",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("review20 arithmetic boundary RPC should succeed");
        assert_eq!(response["handled"], json!(true), "fell through: {prompt}");
        assert!(response["error"]
            .as_str()
            .is_some_and(|error| !error.is_empty()));
    }
    for prompt in [
        "10 - 20 °C",
        "10 - 20 km/h",
        "10 - 20%",
        "10 - 20 (approx)",
        "10 - 20 min, then rest",
        "5pm - 6pm",
        "9am - 5pm",
        "5:00 - 6:00",
        "2026-11-05T17:25+01:00, 5pm - 6pm",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("ordinary range/time prose RPC should succeed");
        assert_eq!(response["handled"], json!(false), "intercepted: {prompt}");
    }
}

#[test]
fn native_exact_arithmetic_review23_boundary_regressions() {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    for prompt in [
        "17 ≤ 25",
        "17 ≥ 25",
        "17 ≠ 25",
        "17 · 25",
        "+",
        "+?",
        "+1-800-555-0100 + 2",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("review23 arithmetic boundary RPC should succeed");
        assert_eq!(response["handled"], json!(true), "fell through: {prompt}");
        assert!(response["error"]
            .as_str()
            .is_some_and(|error| !error.is_empty()));
    }
    for prompt in [
        "5 pm - 6 pm",
        "5:00 PM - 6:00 PM",
        "5:00:00 - 6:00:00",
        "2026-11-05T17:25+01:00, 5 pm - 6 pm",
        "2026-11-05T17:25+01:00, 5:00:00 - 6:00:00",
        "10 - 20?",
        "10 - 20.",
        "10 - 20°",
        "10 - 20$",
        "10 - 20€",
        "+1-800-555-0100",
        "+1 (800) 555-0100",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("ordinary review23 prose RPC should succeed");
        assert_eq!(response["handled"], json!(false), "intercepted: {prompt}");
    }
    let subtraction = dispatch_rpc_command(
        &mut backend,
        "native_exact_arithmetic",
        &json!({"model":"uor-chat","messages":[{"role":"user","content":"50 - 12?"}]}),
    )
    .expect("valid subtraction with sentence punctuation should succeed");
    assert_eq!(subtraction["handled"], json!(true));
    assert_eq!(subtraction["result"], json!("38"));
}

#[test]
fn native_exact_arithmetic_review27_boundary_regressions() {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    for prompt in ["17-", "What is 17-?", "17--25", "2026-11-05 - 1 minute"] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("review27 arithmetic boundary RPC should succeed");
        assert_eq!(response["handled"], json!(true), "fell through: {prompt}");
        assert!(response["error"]
            .as_str()
            .is_some_and(|error| !error.is_empty()));
    }
    for prompt in [
        "24/7 support",
        "1/2 cup",
        "11-05-2026",
        "05-11-2026",
        "1-800-555-0100",
        "123-45-6789",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("ordinary ratio/date/telephone prose RPC should succeed");
        assert_eq!(response["handled"], json!(false), "intercepted: {prompt}");
    }
}

#[test]
fn native_exact_arithmetic_review28_compact_subtraction() {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    for prompt in [
        "50-12",
        "What is 50-12?",
        "50 -12",
        "50- 12",
        "What is 50 -12?",
        "What is 50- 12?",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("compact subtraction RPC should succeed");
        assert_eq!(response["handled"], json!(true), "fell through: {prompt}");
        assert_eq!(response["result"], json!("38"), "wrong result: {prompt}");
    }
    for (prompt, expected) in [("11 -05", "6"), ("2026 -11", "2015")] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("one-sided date-like subtraction RPC should succeed");
        assert_eq!(response["handled"], json!(true), "fell through: {prompt}");
        assert_eq!(
            response["result"],
            json!(expected),
            "wrong result: {prompt}"
        );
    }
    for prompt in ["17-25", "What is 17-25?"] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("compact underflow RPC should succeed");
        assert_eq!(response["handled"], json!(true), "fell through: {prompt}");
        assert!(response["error"]
            .as_str()
            .is_some_and(|error| error.contains("underflow")));
    }
    let over_limit = format!("{}50-12", " ".repeat(200));
    let response = dispatch_rpc_command(
        &mut backend,
        "native_exact_arithmetic",
        &json!({"model":"uor-chat","messages":[{"role":"user","content":over_limit}]}),
    )
    .expect("overlong compact subtraction RPC should succeed");
    assert_eq!(response["handled"], json!(true));
    assert!(response["error"]
        .as_str()
        .is_some_and(|error| error.contains("160-byte")));
    for prompt in [
        "11-05",
        "2026-11",
        "What is 11-05?",
        "What is 2026-11?",
        "11-05-2026",
        "05-11-2026",
        "1-800-555-0100",
        "123-45-6789",
        "3/4 of the students passed",
        "5-10 minutes",
        "50-12 minutes",
        "2026-11-05 - 2026-11-10",
        "12:00 - 1, then lunch",
        "5:00 - 6",
        "12 - 1:00",
        "5 - 6:00",
        "3/4 of the 20 students passed",
        "5-10 minutes, 3 times",
        "5-10 km",
        "5-10 cm",
        "5-10 min",
        "5-10 students",
        "5-10 dollars",
        "2026-11-05 - 2026-11-10, 3 days",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("ordinary compact date/telephone/range RPC should succeed");
        assert_eq!(response["handled"], json!(false), "intercepted: {prompt}");
    }
    for prompt in [
        "What is 10 -20?",
        "What is 12 -25?",
        "2026-11-05 -1",
        "2026-11-05, -1",
        "2026-11-05,-1",
        "2026-11-05,- 1",
        "2026-11-05 , -1",
        "2026-11-05T17:25Z,-1",
        "2026-11-05T17:25-01:00,-1",
        "2026-11-05T17:25+01:00, -1",
        "2026-11-05 +1",
        "17/25 of 3",
        "5-10 minutes + 3",
        "2026-11-05 - 2026-11-10 + 3",
        "17/25 in decimal",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("continued arithmetic RPC should succeed");
        assert_eq!(response["handled"], json!(true), "fell through: {prompt}");
        assert!(response["error"]
            .as_str()
            .is_some_and(|error| !error.is_empty()));
    }
}

#[test]
fn native_exact_arithmetic_final_review_range_and_punctuation_regressions() {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    for (prompt, result) in [
        ("What is 10 - 2?", "8"),
        ("What is 100 - 20?", "80"),
        ("What is 50 - 12,", "38"),
        ("What is 50 - 12 please", "38"),
        ("2026 - 11", "2015"),
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("valid final-review arithmetic RPC should succeed");
        assert_eq!(response["handled"], json!(true), "fell through: {prompt}");
        assert_eq!(response["result"], json!(result), "wrong result: {prompt}");
    }
    for prompt in [
        "What is 17 - 25?",
        "What is 17 - 25,",
        "What is 10 - 20?",
        "What is 50 - 12 + 1",
        "What is 20 + 5 %",
        "What is 20 + 5 / two",
        "What is 20 + 5 (two)",
        "What is 17 + 25 in binary",
        "What is 20 + 10, then double it",
        "What is 20 - 10 in binary",
        "2026 - 11 - 05",
        "5 pm - 6 pm + 3",
        "1 (800) 555-0100 + 2",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("final-review arithmetic boundary RPC should succeed");
        assert_eq!(response["handled"], json!(true), "fell through: {prompt}");
        assert!(
            response["error"]
                .as_str()
                .is_some_and(|error| !error.is_empty()),
            "missing explicit error: {prompt}"
        );
    }
    for prompt in [
        "10 - 20?",
        "10 - 20 minutes",
        "10 - 20 please",
        "10 - 20, please",
        "10 - 20, show your work",
        "10 - 20, answer with the number first",
        "10 - 20, please show your work",
        "5 pm - 6 pm, then dinner",
        "2026-11-05T17:25 - 18:00",
        "5 pm - 6 pm?",
        "9am - 5pm?",
        "5:00 PM - 6:00 PM?",
        "+1-800-555-0100?",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("ordinary final-review prose RPC should succeed");
        assert_eq!(response["handled"], json!(false), "intercepted: {prompt}");
    }
    let mut intercepted = Vec::new();
    for prompt in [
        "5 pm - 6, then dinner",
        "12 - 1 pm, then lunch",
        "5 people attended - 3 left",
        "1 (800) 555-0100",
        "(800) 555-0100",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("ordinary final-review prose RPC should succeed");
        if response["handled"] != json!(false) {
            intercepted.push(prompt);
        }
    }
    assert!(
        intercepted.is_empty(),
        "ordinary final-review prose was intercepted: {intercepted:?}"
    );
    let subtraction_prose = dispatch_rpc_command(
        &mut backend,
        "native_exact_arithmetic",
        &json!({"model":"uor-chat","messages":[{"role":"user","content":"5 - 3 left"}]}),
    )
    .expect("tool-handled subtraction prose RPC should succeed");
    assert_eq!(
        subtraction_prose["handled"],
        json!(true),
        "5 - 3 left must remain tool-handled"
    );
    let mut missing_errors = Vec::new();
    for prompt in [
        "What is (1234567-8)?",
        "(1234567-8)",
        "1 (800) 555-0100 -2",
        "(800) 555-0100 -2",
        "5 people attended - 3 left + 2",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("grouped arithmetic boundary RPC should succeed");
        let explicit_error = response["handled"] == json!(true)
            && response["error"]
                .as_str()
                .is_some_and(|error| !error.is_empty());
        if !explicit_error {
            missing_errors.push(prompt);
        }
    }
    assert!(
        missing_errors.is_empty(),
        "grouped or continued arithmetic must be explicit non-empty errors: {missing_errors:?}"
    );
    for prompt in ["24/7 support", "1/2 cup", "1/2 cup sugar"] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("ordinary ratio prose RPC should succeed");
        assert_eq!(
            response["handled"],
            json!(false),
            "ordinary ratio prose was intercepted: {prompt}"
        );
    }
    let mut intercepted_units = Vec::new();
    for prompt in ["1/2 gallon", "1/2 gram", "1/2 liter", "1/2 dozen"] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("ordinary unit ratio RPC should succeed");
        if response["handled"] != json!(false) {
            intercepted_units.push(prompt);
        }
    }
    assert!(
        intercepted_units.is_empty(),
        "ordinary unit ratio prose was intercepted: {intercepted_units:?}"
    );
    let mut ratio_failures: Vec<String> = Vec::new();
    for prompt in [
        "17/25 show your work",
        "17/25 please",
        "17/25 answer with the number first",
        "17/25 in binary",
        "17/25 in hex",
        "17/25 in decimal",
        "17/25 as a decimal",
        "17/25 to decimal",
        "17/25 percent",
        "17/25 as a percentage",
        "17/25 simplified",
        "1/2 cup + 3",
    ] {
        let response = dispatch_rpc_command(
            &mut backend,
            "native_exact_arithmetic",
            &json!({"model":"uor-chat","messages":[{"role":"user","content":prompt}]}),
        )
        .expect("instruction-suffixed division RPC should succeed");
        let explicit_error = response["handled"] == json!(true)
            && response["error"]
                .as_str()
                .is_some_and(|error| !error.is_empty());
        if !explicit_error {
            ratio_failures.push(prompt.to_string());
        }
    }
    let padded_ratio = format!("{}17/25 show your work", " ".repeat(200));
    let padded_response = dispatch_rpc_command(
        &mut backend,
        "native_exact_arithmetic",
        &json!({"model":"uor-chat","messages":[{"role":"user","content":padded_ratio}]}),
    )
    .expect("padded instruction-suffixed division RPC should succeed");
    let padded_ok = padded_response["handled"] == json!(true)
        && padded_response["error"]
            .as_str()
            .is_some_and(|error| error.contains("160-byte"));
    if !padded_ok {
        ratio_failures.push("over-160-byte instruction-suffixed division".to_string());
    }
    let padded_decimal = format!("{}17/25 in decimal", " ".repeat(200));
    let padded_decimal_response = dispatch_rpc_command(
        &mut backend,
        "native_exact_arithmetic",
        &json!({"model":"uor-chat","messages":[{"role":"user","content":padded_decimal}]}),
    )
    .expect("padded in-decimal division RPC should succeed");
    let padded_decimal_ok = padded_decimal_response["handled"] == json!(true)
        && padded_decimal_response["error"]
            .as_str()
            .is_some_and(|error| error.contains("160-byte"));
    if !padded_decimal_ok {
        ratio_failures.push("over-160-byte in-decimal division".to_string());
    }
    assert!(
        ratio_failures.is_empty(),
        "instruction-suffixed division must be an explicit non-empty error: {ratio_failures:?}"
    );
}

#[test]
fn native_write_file_enforces_its_size_cap() {
    let root = unique_temp_dir("write-cap");
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&root)
        .expect("grant write root");
    let oversized = "x".repeat(16 * 1024 * 1024 + 1);
    let error = backend
        .native_write_file(&root.join("large.txt"), &oversized)
        .expect_err("oversized write must fail");
    assert!(
        matches!(error, DesktopClientError::FileTooLarge { .. }),
        "{error:?}"
    );
    assert!(!root.join("large.txt").exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn rpc_and_loopback_http_studio_host_serve_commands_and_static_assets() {
    let receipt = run_self_test_receipt().expect("self-test receipt");
    assert_eq!(receipt["verdict"], json!("PASS"));
    assert_eq!(receipt["artifactBytes"], json!(469_252));

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let studio_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../web/studio")
        .canonicalize()
        .expect("canonicalize studio_root");
    let bound_host = "127.0.0.1:1420";
    let token = generate_launch_token();
    assert_eq!(token.len(), 64);

    // 1. Page RPC must refuse host-owned `native_grant_folder` and `native_set_terminal_enabled`:
    let grant_rpc = handle_rpc_line(
        &mut backend,
        r#"{"id":1,"command":"native_grant_folder","args":{"folderPath":"/tmp"}}"#,
    );
    assert_eq!(grant_rpc["ok"], json!(false));
    assert!(grant_rpc["error"]
        .as_str()
        .unwrap_or("")
        .contains("host-owned"));

    let term_rpc = handle_rpc_line(
        &mut backend,
        r#"{"id":2,"command":"native_set_terminal_enabled","args":{"enabled":true}}"#,
    );
    assert_eq!(term_rpc["ok"], json!(false));

    // 2. GET / with matching Host header returns 200 and embeds the per-launch token:
    let get_headers = vec![("Host".to_string(), bound_host.to_string())];
    let (status, ctype, html_bytes) = handle_studio_http_request(
        &mut backend,
        &studio_root,
        bound_host,
        &token,
        "GET",
        "/",
        &get_headers,
        b"",
    );
    assert_eq!(status, 200);
    assert_eq!(ctype, "text/html; charset=utf-8");
    let html_str = String::from_utf8_lossy(&html_bytes);
    assert!(html_str.contains("UOR-R4 Studio — local chat"));
    assert!(html_str.contains(&token));

    // 3. GET / with foreign Host header (DNS rebinding) or traversal path is refused with 403:
    let bad_host_headers = vec![("Host".to_string(), "evil.example:1420".to_string())];
    let (status_rebind, _, _) = handle_studio_http_request(
        &mut backend,
        &studio_root,
        bound_host,
        &token,
        "GET",
        "/",
        &bad_host_headers,
        b"",
    );
    assert_eq!(status_rebind, 403);

    let (status_trav, _, _) = handle_studio_http_request(
        &mut backend,
        &studio_root,
        bound_host,
        &token,
        "GET",
        "/../Cargo.toml",
        &get_headers,
        b"",
    );
    assert_eq!(status_trav, 403);

    // 4. POST /__tauri_invoke without token, with wrong token, with foreign Origin, with foreign
    //    Host, or with text/plain Content-Type (no-cors simple POST) must all be refused with 403:
    let invoke_body = br#"{"id":7,"command":"native_load_synthetic_standin","args":{}}"#;

    let missing_token_headers = vec![
        ("Host".to_string(), bound_host.to_string()),
        ("Origin".to_string(), format!("http://{bound_host}")),
        ("Content-Type".to_string(), "application/json".to_string()),
    ];
    let (status_no_tok, _, _) = handle_studio_http_request(
        &mut backend,
        &studio_root,
        bound_host,
        &token,
        "POST",
        "/__tauri_invoke",
        &missing_token_headers,
        invoke_body,
    );
    assert_eq!(status_no_tok, 403);

    let wrong_origin_headers = vec![
        ("Host".to_string(), bound_host.to_string()),
        ("Origin".to_string(), "https://evil.example".to_string()),
        ("Content-Type".to_string(), "application/json".to_string()),
        ("X-UOR-Desktop-Token".to_string(), token.clone()),
    ];
    let (status_bad_origin, _, _) = handle_studio_http_request(
        &mut backend,
        &studio_root,
        bound_host,
        &token,
        "POST",
        "/__tauri_invoke",
        &wrong_origin_headers,
        invoke_body,
    );
    assert_eq!(status_bad_origin, 403);

    let wrong_host_post_headers = vec![
        ("Host".to_string(), "evil.example:1420".to_string()),
        ("Origin".to_string(), format!("http://{bound_host}")),
        ("Content-Type".to_string(), "application/json".to_string()),
        ("X-UOR-Desktop-Token".to_string(), token.clone()),
    ];
    let (status_bad_host_post, _, _) = handle_studio_http_request(
        &mut backend,
        &studio_root,
        bound_host,
        &token,
        "POST",
        "/__tauri_invoke",
        &wrong_host_post_headers,
        invoke_body,
    );
    assert_eq!(status_bad_host_post, 403);

    let nocors_text_headers = vec![
        ("Host".to_string(), bound_host.to_string()),
        ("Origin".to_string(), format!("http://{bound_host}")),
        ("Content-Type".to_string(), "text/plain".to_string()),
        ("X-UOR-Desktop-Token".to_string(), token.clone()),
    ];
    let (status_nocors, _, _) = handle_studio_http_request(
        &mut backend,
        &studio_root,
        bound_host,
        &token,
        "POST",
        "/__tauri_invoke",
        &nocors_text_headers,
        invoke_body,
    );
    assert_eq!(status_nocors, 403);

    // 5. POST /__tauri_invoke with valid Host, Origin, Content-Type, and X-UOR-Desktop-Token succeeds:
    let valid_post_headers = vec![
        ("Host".to_string(), bound_host.to_string()),
        ("Origin".to_string(), format!("http://{bound_host}")),
        ("Content-Type".to_string(), "application/json".to_string()),
        ("X-UOR-Desktop-Token".to_string(), token),
    ];
    let (status_post, ctype_post, resp_bytes) = handle_studio_http_request(
        &mut backend,
        &studio_root,
        bound_host,
        valid_post_headers[3].1.as_str(),
        "POST",
        "/__tauri_invoke",
        &valid_post_headers,
        invoke_body,
    );
    assert_eq!(status_post, 200);
    assert_eq!(ctype_post, "application/json; charset=utf-8");
    let parsed: serde_json::Value = serde_json::from_slice(&resp_bytes).expect("parse rpc json");
    assert_eq!(parsed["id"], json!(7));
    assert_eq!(parsed["ok"], json!(true));
    assert_eq!(parsed["result"]["artifact_bytes"], json!(469_252));
}

#[cfg(feature = "gui")]
#[test]
fn native_picker_grants_share_project_model_and_git_state() {
    use uor_r4_desktop_client::tauri_gui::TauriDesktopState;

    let root = unique_temp_dir("native-picker-state");
    let project = root.join("project");
    let models = root.join("models");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&models).expect("create models");
    fs::write(project.join("note.txt"), "granted\n").expect("write project file");
    fs::write(
        models.join(SYNTHETIC_STANDIN_FILE_NAME),
        build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED),
    )
    .expect("write model");
    let init = Command::new("git")
        .arg("init")
        .current_dir(&project)
        .output()
        .expect("init project");
    assert!(init.status.success());

    let backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let state = TauriDesktopState::new(backend);
    state
        .grant_project_folder(&project)
        .expect("project picker grant");
    state
        .grant_model_folder(&models)
        .expect("model picker grant");
    let read = state
        .with_backend(|backend| {
            backend
                .native_read_file(&project.join("note.txt"))
                .map_err(|error| error.to_string())
        })
        .expect("read through picker grant");
    assert_eq!(read, "granted\n");
    let listed = state
        .with_backend(|backend| {
            backend
                .native_list_local_models()
                .map_err(|error| error.to_string())
        })
        .expect("models through picker grant");
    assert_eq!(listed.len(), 1);
    let status = state
        .git_status(&project)
        .expect("status outside backend lock");
    assert!(!status.is_clean);
    let diff = state.git_diff(&project).expect("diff outside backend lock");
    assert!(!diff.truncated);
    let _ = fs::remove_dir_all(&root);
}

#[cfg(feature = "gui")]
#[test]
fn tauri_gui_command_wrappers_and_builder_dispatch_through_shared_state() {
    use uor_r4_desktop_client::tauri_gui::{
        build_tauri_app_builder, invoke_tauri_command_for_test, TauriDesktopState,
    };

    let root = unique_temp_dir("tauri-gui-state");
    let granted_dir = root.join("workspace");
    fs::create_dir_all(&granted_dir).expect("create granted workspace");
    let sample_file = granted_dir.join("note.txt");
    fs::write(&sample_file, "tauri-v2-native-check\n").expect("write note.txt");

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(&granted_dir)
        .expect("grant folder");
    let state = TauriDesktopState::new(backend);

    // 1. Verify Tauri Builder constructs cleanly with managed state and all registered #[tauri::command] handlers:
    let _builder = build_tauri_app_builder(state.clone());

    // 2. Load synthetic stand-in via #[tauri::command] wrapper:
    let caps = invoke_tauri_command_for_test(&state, "native_load_synthetic_standin", json!({}))
        .expect("native_load_synthetic_standin via tauri_gui");
    assert_eq!(caps["artifact_bytes"], json!(469_252));
    assert_eq!(
        caps["artifact_sha256"],
        json!("d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df")
    );

    // 3. Step 4 tokens via #[tauri::command] wrapper:
    let stepped = invoke_tauri_command_for_test(
        &state,
        "native_step_tokens",
        json!({
            "conversationId": "gui-test",
            "tokenIds": [0],
            "maxNewTokens": 4
        }),
    )
    .expect("native_step_tokens via tauri_gui");
    assert_eq!(stepped["receipt"]["token_ids"], json!([38, 9, 33, 8]));

    // 4. Scoped file read via #[tauri::command] wrapper:
    let read_out = invoke_tauri_command_for_test(
        &state,
        "native_read_file",
        json!({
            "filePath": sample_file.to_string_lossy()
        }),
    )
    .expect("native_read_file via tauri_gui");
    assert_eq!(read_out, json!("tauri-v2-native-check\n"));

    let _ = fs::remove_dir_all(&root);
}

#[cfg(feature = "gui")]
#[test]
fn native_dialog_commands_return_futures_instead_of_blocking_the_main_thread() {
    let source = include_str!("../src/lib.rs");
    for command in [
        "native_choose_project_folder",
        "native_choose_model_folder",
        "native_enable_terminal",
        "native_confirm_terminal_command",
        "native_mcp_connect_remote",
        "native_mcp_connect_stdio",
    ] {
        assert!(
            source.contains(&format!(
                "#[tauri::command(async)]\n    pub async fn {command}("
            )),
            "{command} must remain async so its native dialog cannot park the main event loop"
        );
    }
}

#[cfg(feature = "gui")]
#[test]
fn terminal_and_mcp_confirmations_are_parented_to_the_main_window() {
    let source = include_str!("../src/lib.rs");
    assert!(source.contains("window.dialog().message(message).parent(&window)"));
    assert_eq!(
        source
            .matches("let confirmed = confirmation_dialog(")
            .count(),
        4,
        "terminal and MCP confirmations must use native sheets on the main window"
    );
}
