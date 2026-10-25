# REPORT: infra-desktop-client-tauri, antigravity-71

Lab: antigravity. Item: [infra-desktop-client-tauri](../../../../roadmap/items/infra-desktop-client-tauri.md) (#2826).
Closes #2826

## What did not work

- **Compiling `tauri::generate_context!()` without a bundle icon file at `src-tauri/icons/icon.png` failed (`classified: instrument` — proc-macro build requirement):**
  - Even with `"bundle": { "active": false }` in `src-tauri/tauri.conf.json`, `tauri::generate_context!()` inspects `src-tauri/icons/icon.png` at macro-expansion time and panics with `failed to open icon .../src-tauri/icons/icon.png: No such file or directory`.
  - Committing a binary `.png` file would conflict with repository binary-hygiene rules (`CHARTER §3.13`). Instead, `src-tauri/build.rs` now generates the 68-byte 1x1 RGBA PNG at `src-tauri/icons/icon.png` before `tauri_build::build()` when `--features gui` is enabled, and `src-tauri/.gitignore` ignores `/icons/` so zero binary files are committed.
- **Enabling `tauri = "2"` in the default feature set would require system WebKit libraries (`libwebkit2gtk`) on headless Linux CI runners:**
  - Kept `default = []` in `src-tauri/Cargo.toml` and gated `tauri = "2"` and `tauri-build = "2"` behind `gui = ["dep:tauri", "dep:tauri-build"]`, so `cargo test --manifest-path src-tauri/Cargo.toml --locked` runs anywhere without GUI libraries while `cargo test --manifest-path src-tauri/Cargo.toml --features gui --locked` builds and tests the full Tauri v2 WebView shell on macOS.

## What changed

- **Files changed:** 11 files (`src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`, `src-tauri/.gitignore`, `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, `src-tauri/tests/desktop_client_controls.rs`, `docs/products/desktop-client.md`, `docs/products/README.md`, `docs/labs/infra-desktop-client-tauri/README.md`, `docs/roadmap/items/infra-desktop-client-tauri.md`, `docs/roadmap/README.md`, plus `ORDER.md` and `REPORT.md`).
- **Tauri v2 WebView Shell Wiring (`src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`):**
  - Added optional `tauri = "2"` and `tauri-build = "2"` dependencies under feature `gui = ["dep:tauri", "dep:tauri-build"]` and locked `src-tauri/Cargo.lock`.
  - Implemented `pub mod tauri_gui` in `src-tauri/src/lib.rs` under `#![forbid(unsafe_code)]`:
    - `TauriDesktopState` (`Arc<Mutex<DesktopBackend>>`) managing the native `uor-r4-integer` + `uor-r4-object-store` backend;
    - All 16 `#[tauri::command]` handlers (`native_list_local_models`, `native_load_synthetic_standin`, `native_load_model`, `native_step_tokens`, `native_generate_prompt`, `native_export_session`, `native_import_session`, `native_reset_session`, `native_save_session_durable`, `native_restore_session_durable`, `native_list_granted_folders`, `native_read_file`, `native_write_file`, `native_list_directory`, `native_git_status`, `native_run_terminal_command`);
    - `build_tauri_app_builder` (`tauri::Builder::default().manage(state).invoke_handler(tauri::generate_handler![...])`) and `run_tauri_gui` (`tauri::generate_context!()`).
  - Added integration test `tauri_gui_command_wrappers_and_builder_dispatch_through_shared_state` in `src-tauri/tests/desktop_client_controls.rs` (shown failing with `E0432: unresolved import uor_r4_desktop_client::tauri_gui` before implementation, then passing `7/7`).
  - Updated `docs/products/desktop-client.md`, `docs/products/README.md`, `docs/labs/infra-desktop-client-tauri/README.md`, and set `docs/roadmap/items/infra-desktop-client-tauri.md` to `status: done` (`Closes #2826`).

## Measurements

- **`CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --features gui --locked --no-fail-fast`:**
  - `7/7` integration tests pass in `0.30s` (`tauri_gui_command_wrappers_and_builder_dispatch_through_shared_state ... ok`), verifying `build_tauri_app_builder`, `native_load_synthetic_standin` (`469,252 B`, `sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`), `native_step_tokens` (`[38, 9, 33, 8]`), and `native_read_file` inside a granted workspace.
- **`CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --locked --no-fail-fast`:**
  - `6/6` default GUI-free integration tests pass in `0.24s`.
- **`node web/studio/verify_desktop_client.mjs`:**
  - `{"verdict":"ALL DESKTOP CLIENT & OFFLINE STUDIO CHECKS PASSED","offlineExternalUrls":0,"streamedTokensCount":32,"diffLineChangesCount":5,"threeRenderCount":1,"invokedCommandsCount":38,"remoteRequests":0}`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --features gui --locked --no-fail-fast`: PASS (`7/7`)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --locked --no-fail-fast`: PASS (`6/6`)
- `node web/studio/verify_desktop_client.mjs`: PASS
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap render`: PASS
- `uor-harness loop report --check --order docs/loop/tasks/infra-desktop-client-tauri/antigravity-71/ORDER.md`: PASS
- `uor-harness loop pr-check --local`: PASS
- `uor-harness loop gates --full`: PASS

## Skills used

- `scope-guard`: Kept all changes strictly within `infra-desktop-client-tauri`'s declared scope (`src-tauri/**`, `docs/products/**`, `docs/labs/infra-desktop-client-tauri/**`, `docs/loop/tasks/infra-desktop-client-tauri/**`) and always-allowed roadmap files.
- `solution-integrity`: Feature-gated `tauri = "2"` and `tauri-build = "2"` under `gui` so headless CI runs without system WebKit libraries while `--features gui` compiles the complete Tauri v2 WebView shell on macOS with zero `unsafe` code (`#![forbid(unsafe_code)]`) and zero committed binary assets.

## Still open

- None for `infra-desktop-client-tauri` (`#2826`). Until a trained model passes milestones M1–M8, the desktop client runs against the `seed = 5` synthetic `UORLUT01` stand-in, and native laptop decode speed/energy measurement belongs to Milestone M5 (`#2033`).

## Plain summary

The UOR-R4 desktop client (`src-tauri/`) previously had its native Rust engine backend (`uor-r4-integer` + `uor-r4-object-store`), `--stdio` bridge, and `--serve-studio` loopback host built, but had not yet locked and wired the `tauri` v2 WebView crate (`tauri::Builder` and `#[tauri::command]`). This unit locks `tauri = "2"` and `tauri-build = "2"` in `src-tauri/Cargo.lock` behind `--features gui` (keeping the default build free of system GUI libraries for headless CI), implements `pub mod tauri_gui` in `src-tauri/src/lib.rs` with all 16 `#[tauri::command]` handlers and `tauri::Builder` state management under `#![forbid(unsafe_code)]`, generates the required bundle icon at build time so no binary files are committed, verifies all `7/7` integration controls and the offline Studio test suite on the synthetic stand-in, and closes `#2826`.
