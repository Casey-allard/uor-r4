# REPORT: infra-desktop-client-tauri, antigravity-22

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Initial failing test probe (`src-tauri/tests/desktop_client_controls.rs`):** Before implementing the native desktop command handlers in [`src-tauri/src/lib.rs`](../../../../../src-tauri/src/lib.rs), `native_list_local_models_returns_synthetic_standin_and_refuses_tampered_hash` failed with `left: 0, right: 469252` (`assertion 'left == right' failed`). Implementing `DesktopState`, `GrantedFolders`, `native_list_local_models`, `native_load_model`, `native_step_tokens`, `native_save_session_durable`, `native_restore_session_durable`, `native_read_file`, `native_write_file`, `native_list_directory`, `native_git_status`, and `native_run_terminal_command` resolved all 5 controls (`5 passed; 0 failed`).
- **Tauri worker bridge action alias (`load_local_model` vs `load_model`):** The first run of [`web/studio/verify_desktop_client.mjs`](../../../../../web/studio/verify_desktop_client.mjs) failed (`FAIL: load_local_model did not emit model_loaded`) because `linkLocalDesktopModel` in [`web/studio/index.html`](../../../../../web/studio/index.html) dispatches `{ action: 'load_local_model', payload: { path } }` whereas `createTauriNativeModelWorker` in [`web/studio/assets/js/uor_offline_vendor.js`](../../../../../web/studio/assets/js/uor_offline_vendor.js) originally matched only `action === 'load_model'` and `payload.filePath`. Updating `uor_offline_vendor.js` to accept both `load_model` and `load_local_model` (`payload.filePath || payload.path`) resolved the failure.
- **Scope boundary (`docs/research/notebook/**` and `docs/roadmap/items/infra-desktop-client-tauri.md`):** The item file [`docs/roadmap/items/infra-desktop-client-tauri.md`](../../../../roadmap/items/infra-desktop-client-tauri.md) declares `scope: [src-tauri/**, web/**, crates/uor-r4-integer/**, crates/uor-r4-object-store/**, docs/products/**, Cargo.toml, docs/labs/infra-desktop-client-tauri/**]`, which excludes `docs/research/notebook/**` and `docs/roadmap/items/**`. Per `loop.pr_check` scope enforcement, both six-column research notebook rows (`m4-serving.md` and `m3-durable-memory.md`) are recorded verbatim in [`docs/labs/infra-desktop-client-tauri/README.md`](../../../../labs/infra-desktop-client-tauri/README.md) (`§3`), and this PR references `#2826` without touching files outside `scope:`.

## What changed

- **Code (`+2,186 / -62` lines across 6 files):**
  - [`Cargo.toml`](../../../../../Cargo.toml): added `"src-tauri"` to workspace `exclude` so `cargo test --workspace` stays free of system GUI/WebView dependencies.
  - [`src-tauri/Cargo.toml`](../../../../../src-tauri/Cargo.toml), [`src-tauri/src/lib.rs`](../../../../../src-tauri/src/lib.rs), and [`src-tauri/src/main.rs`](../../../../../src-tauri/src/main.rs): implemented the `#![forbid(unsafe_code)]` native desktop client linking [`crates/uor-r4-integer`](../../../../../crates/uor-r4-integer/) (`LoadedLutStack`, `LutStackSession`, `mk_synthetic_stack_bytes`) and [`crates/uor-r4-object-store`](../../../../../crates/uor-r4-object-store/) (`Store`, `encode_memory_envelope`, `fold_memory_envelope`, `reconstruct_memory_envelope`, `verify_memory_envelope`), pre-parse SHA-256 verification before `format.rs`, `GrantedFolders` path allowlist enforcement, opt-in + user-confirmed terminal command execution, and the `--self-test` / `--serve-addr` native host runner.
  - [`web/studio/assets/css/uor_offline_fonts.css`](../../../../../web/studio/assets/css/uor_offline_fonts.css), [`web/studio/assets/js/uor_offline_vendor.js`](../../../../../web/studio/assets/js/uor_offline_vendor.js), and [`web/studio/index.html`](../../../../../web/studio/index.html): replaced all external CDN links/scripts with self-hosted local vendor shims (`marked`, `katex`, `monaco`, AMD `require`, and `createTauriNativeModelWorker`), wired `native_grant_folder` + `native_list_directory` in `loadTauriGitWorkspace`, wired `save_session_durable` on `generate_complete`, and gated `native_run_terminal_command` behind explicit `window.confirm` + `userConfirmed: true` + `confirmedCommand: cmd`.
- **Tests (`+562 / -0` lines across 2 files):**
  - [`src-tauri/tests/desktop_client_controls.rs`](../../../../../src-tauri/tests/desktop_client_controls.rs): 5 integration tests covering `UORLUT01` discovery + tampered hash refusal, 32-token trajectory parity with `docs/wasm-serving/verify.mjs`, `UORMEM01` + `UR4SESS1` close-and-resume across fresh `DesktopState` instances (`16 + 16 == 32`), `GrantedFolders` file/Git allowlist enforcement, and default-off + explicit-consent terminal command guards.
  - [`web/studio/verify_desktop_client.mjs`](../../../../../web/studio/verify_desktop_client.mjs): automated Node verification of zero external URLs in Studio HTML/CSS/JS, CSP restrictions in `tauri.conf.json`, `uor-desktop-client --self-test`, and `createTauriNativeModelWorker` IPC commands under network traps.
- **Docs & config (`+533 / -28` lines across 6 files):**
  - [`src-tauri/Cargo.lock`](../../../../../src-tauri/Cargo.lock), [`src-tauri/tauri.conf.json`](../../../../../src-tauri/tauri.conf.json), and [`src-tauri/capabilities/default.json`](../../../../../src-tauri/capabilities/default.json).
  - [`docs/products/desktop-client.md`](../../../../products/desktop-client.md), [`docs/products/README.md`](../../../../products/README.md), and [`docs/labs/infra-desktop-client-tauri/README.md`](../../../../labs/infra-desktop-client-tauri/README.md).
- **Ratio against ORDER estimate:** [`ORDER.md`](ORDER.md) estimated ~650 lines across 11 files (`+/- 2x` tolerance); excluding the generated `src-tauri/Cargo.lock` (`396` lines), the implementation and verification suite totals `2,975` net lines across 14 files (`4.58x`, driven by the complete self-hosted offline vendor bundle in `uor_offline_vendor.js` and the native IPC / HTTP bridge + self-test runner in `src-tauri/src/lib.rs` and `src-tauri/src/main.rs`).

## Measurements

- **Native CLI self-test (`CARGO_TARGET_DIR=local/tauri-target ~/.cargo/bin/cargo run --manifest-path src-tauri/Cargo.toml --locked -- --self-test`):**
  - `verdict = "PASS"`, `engine = "uor-r4-integer/native-desktop"`
  - Synthetic stand-in (`seed = 5`): `artifactBytes = 469,252`, `artifactSha256 = "d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df"`, `weightsPerToken = 15,968`
  - 32-token trajectory from `PROMPT = [0]`: `[38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30]` (exact token-for-token match with `docs/wasm-serving/verify.mjs`)
  - Durable session snapshot in `uor-r4-object-store::Store`: `ur4sess1Bytes = 9,782`, `storeRecords = 446`, `objectVersion = "bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca"`
- **Offline Studio & desktop bridge verification (`UOR_DESKTOP_BIN=./local/tauri-target/debug/uor-desktop-client node web/studio/verify_desktop_client.mjs`):**
  - `verdict = "ALL DESKTOP CLIENT & OFFLINE STUDIO CHECKS PASSED"`, `offlineExternalUrls = 0`, `remoteRequests = 0`.

## Checks run at the head

- `~/.cargo/bin/cargo fmt --manifest-path src-tauri/Cargo.toml --check`: exit `0`
- `CARGO_TARGET_DIR=local/tauri-target ~/.cargo/bin/cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings`: exit `0` (`0` warnings)
- `CARGO_TARGET_DIR=local/tauri-target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --locked`: `5 passed; 0 failed` in `src-tauri/tests/desktop_client_controls.rs`
- `UOR_DESKTOP_BIN=./local/tauri-target/debug/uor-desktop-client node web/studio/verify_desktop_client.mjs`: exit `0` (`ALL DESKTOP CLIENT & OFFLINE STUDIO CHECKS PASSED`)
- `uor-harness loop gates --full`: `all 17 gates green`
- `uor-harness loop report --check`: `REPORT ok`
- `uor-harness loop pr-check --local`: `ok`

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)

## Still open

- Copying the two six-column research notebook rows from [`docs/labs/infra-desktop-client-tauri/README.md`](../../../../labs/infra-desktop-client-tauri/README.md) (`§3`) into `docs/research/notebook/m4-serving.md` and `docs/research/notebook/m3-durable-memory.md` and setting `status: done` on `docs/roadmap/items/infra-desktop-client-tauri.md` once an item scope includes those paths. References #2826.
- Qualifying a trained model on M1–M8 (`0/8` milestones complete; all verification here uses the `seed = 5` synthetic stand-in only).

## Plain summary

We built the native desktop backend (`src-tauri/`) for the UOR-R4 Developer Studio (`#2826`) and made the shared Studio web page (`web/studio/index.html`) run 100% offline with zero external CDN requests. The Rust desktop backend links the integer serving crate (`uor-r4-integer`, which runs lookup-table inference using only integer additions and bit shifts) and the content-addressed object store (`uor-r4-object-store`, which stores conversation history and `UR4SESS1` session snapshots as verified 32-byte Merkle-tree records). Every file and Git operation is restricted to folders explicitly granted by the user, terminal commands are disabled by default and require per-command user confirmation inside a granted folder, and closing and reopening a session resumes token generation identically on the 469,252-byte synthetic stand-in model.
