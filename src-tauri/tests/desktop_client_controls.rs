//! Host control tests for the UOR-R4 desktop client backend (`infra-desktop-client-tauri`, #2826).
#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde_json::json;
use uor_r4_desktop_client::{
    build_synthetic_stack_artifact, generate_launch_token, handle_rpc_line,
    handle_studio_http_request, run_self_test_receipt, sha256_bytes, DesktopBackend,
    DesktopClientError, PinRegistry, TerminalCommandRequest, SYNTHETIC_STANDIN_FILE_NAME,
    SYNTHETIC_STANDIN_SEED,
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
    assert!(html_str.contains("Developer Studio"));
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
    let caps = invoke_tauri_command_for_test(
        &state,
        "native_load_synthetic_standin",
        json!({}),
    )
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
