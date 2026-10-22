# Lab Record: Native Desktop Backend (`src-tauri/`), Loopback Studio Host, and Durable Object-Store Sessions (`#2826`)

> **Claim discipline:** 0/8 milestones are complete; no capability is established. All native serving and durable memory verification in this record uses the 469,252-byte `mk-synthetic-stack` (`seed = 5`, `UORLUT01`) synthetic stand-in artifact (`sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`). No trained model exists (`M1`–`M4` `0/8`), and no native vs. WebAssembly decode speed is claimed (`M5` `0/8`).

## 1. Summary

This record documents the implementation of the native Rust desktop client backend (`src-tauri/`, Issue `#2826`, [`docs/roadmap/items/infra-desktop-client-tauri.md`](../../roadmap/items/infra-desktop-client-tauri.md)) and the offline Studio desktop bridge ([`web/studio/index.html`](../../../web/studio/index.html), [`web/studio/assets/js/uor_offline_vendor.js`](../../../web/studio/assets/js/uor_offline_vendor.js)):

1. **Workspace placement & Tauri WebView scope:** [`src-tauri/Cargo.toml`](../../../src-tauri/Cargo.toml) is a standalone Cargo package listed under `exclude = [..., "src-tauri"]` in the root [`Cargo.toml`](../../../Cargo.toml) so `cargo test --workspace` stays free of system GUI/WebView dependencies while `cargo test --manifest-path src-tauri/Cargo.toml --locked` compiles and tests the native desktop backend directly. Because `--locked` against the offline workspace `Cargo.lock` precludes fetching the ~300-crate `tauri` v2 GUI dependency tree without a maintainer lockfile vendor update, `src-tauri` provides the zero-GUI-crate native Rust `DesktopBackend`, `--stdio` JSON-RPC bridge, and `--serve-studio 127.0.0.1:<port>` loopback HTTP Studio host (`References #2826`), while linking `tauri::Builder` / `#[tauri::command]` remains an explicit follow-up once Tauri v2 is vendor-fetched into `Cargo.lock`.
2. **Native D11 integer serving (`uor-r4-integer`):** [`src-tauri/src/lib.rs`](../../../src-tauri/src/lib.rs) links [`crates/uor-r4-integer`](../../../crates/uor-r4-integer/) (`IntegerStackModel`, `LutStackSession`) with `#![forbid(unsafe_code)]`, zero softmax/squashing (D23), and zero matmul (D25), and compiles [`crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs`](../../../crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs) directly via [`src-tauri/build.rs`](../../../src-tauri/build.rs) with zero code duplication. `native_list_local_models` stream-checks the 8-byte `b"UORLUT01"` magic header before reading, skips unpinned/non-`UORLUT01` files so unrelated `.bin` files cannot hide pinned models, and `native_load_model` verifies `b"UORLUT01"` magic and `PinRegistry` SHA-256 before invoking `IntegerStackModel::from_bytes`.
3. **Durable session memory (`uor-r4-object-store`):** `native_save_session_durable` and `native_restore_session_durable` wrap `UR4SESS1` session snapshots (`session_codec::encode` / `decode`) inside a `UORMEM01` envelope (`encode_memory_envelope`, `fold_memory_envelope`, `reconstruct_memory_envelope`, `verify_memory_envelope`) persisted in `uor-r4-object-store::Store` (`UOROBJ01`) inside a user-granted store folder (`set_durable_store_dir` enforces `granted_folders`).
4. **Scoped file, Git, symlink, terminal, and loopback HTTP controls:** `native_grant_folder` is host-owned (`--grant <dir>` CLI flag on `uor-desktop-client`, refused over page RPC). `granted_folders` canonicalizes user-granted workspace roots and refuses symlinks and dangling symlinks (`reject_if_symlink`, `reject_symlinks_between_root_and_target`, `DesktopClientError::SymlinkRefused`) as well as any target outside granted roots across `native_read_file` (capped at `16 MiB`), `native_write_file`, `native_list_directory`, `native_git_status` (running `git` with `-c core.fsmonitor=false -c core.hooksPath=/dev/null` and reporting `"HEAD (unborn)"` on empty repos), and `set_durable_store_dir`. `native_run_terminal_command` is disabled by default (`terminal_enabled = false`), keeps both `native_set_terminal_enabled` and `host_approve_terminal_command` host-owned (not callable via page RPC), executes allowlisted binaries directly via `Command::new(program).args(...)` (no `sh -c`), and confines every path-like argument (`/`-containing, `.`-prefixed, or `~`-prefixed) to `granted_folders`. On `--serve-studio 127.0.0.1:<port>`, `POST /__tauri_invoke` requires a per-launch random `X-UOR-Desktop-Token` (embedded in the served `index.html`), `Host == 127.0.0.1:<port>`, `Origin == http://127.0.0.1:<port>`, and `Content-Type: application/json` to reject cross-origin browser requests and DNS rebinding.
5. **100% offline Studio frontend & vendor shims:** [`web/studio/index.html`](../../../web/studio/index.html) replaces all external CDN `<script>` and `<link>` tags (`marked`, `KaTeX`, `pdf.js`, `three.js`, `Monaco`) with [`web/studio/assets/css/uor_offline_fonts.css`](../../../web/studio/assets/css/uor_offline_fonts.css) and [`web/studio/assets/js/uor_offline_vendor.js`](../../../web/studio/assets/js/uor_offline_vendor.js) (`0` external `http://` or `https://` URLs), gates `callGithubApi` off in desktop/offline mode while preserving online browser mode, streams tokens step-by-step (`maxNewTokens: 1` per IPC step) in `createTauriNativeModelWorker`, and provides full offline shims for `monaco.editor` (`setModelLanguage`, `getModelLanguage`, and LCS line-by-line diff in `createDiffEditor`), `marked.parse` (lists, ordered lists, blockquotes, pipe tables, links, italics, bold, code), `katex.renderToString` (fractions, roots, superscripts, subscripts, Greek letters, operators), `THREE` (2D-canvas software 3D projection), and `pdfjsLib` (ASCII text extraction + offline notice).

## 2. Verification Receipts

### 2.1 Native Rust Integration Controls (`src-tauri/tests/desktop_client_controls.rs`)

Command:

```sh
CARGO_TARGET_DIR=local/tauri-target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --locked
```

All `6/6` integration controls pass:

| Test | Verified contract | Result |
|---|---|---|
| `native_list_local_models_returns_synthetic_standin_and_refuses_tampered_hash` | Discovers `UORLUT01` (`469,252 B`, `sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`), skips unrelated/unpinned `.bin` files without hiding pinned models, refuses non-`UORLUT01` magic, and refuses tampered SHA-256 before `format.rs` | `ok` |
| `native_engine_step_tokens_matches_reference_32_token_trajectory` | 32-token greedy trajectory from `PROMPT = [0]` matches `docs/wasm-serving/verify.mjs` (`[38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30]`), `weights_per_token = 15,968` | `ok` |
| `durable_memory_object_store_close_and_resume_continues_token_for_token` | Refuses `set_durable_store_dir` outside `granted_folders`; steps 16 tokens (`UR4SESS1` `5,366 B`), folds `UORMEM01` into `uor-r4-object-store::Store` on disk inside a granted folder, drops `DesktopBackend`, reconstructs and verifies `UORMEM01` + `UR4SESS1` in a fresh `DesktopBackend`, and steps 16 more tokens; combined `16 + 16` matches the unsplit 32-token trajectory token-for-token | `ok` |
| `scoped_file_and_git_commands_enforce_granted_folder_and_refuse_dangling_symlinks` | Refuses `native_read_file`, `native_write_file`, `native_list_directory`, and `native_git_status` outside `granted_folders`, refuses `../` traversal, refuses symlinks and dangling symlinks (`SymlinkRefused`) pointing outside the granted root, and reports `"HEAD (unborn)"` on an empty Git repository | `ok` |
| `terminal_command_requires_host_approval_and_confines_arguments_to_granted_folders` | Refuses `native_run_terminal_command` when `terminal_enabled == false`, when `host_approve_terminal_command` was not called by the host, when `user_confirmed == false`, when `confirmed_command != command`, when `cwd` is outside `granted_folders`, when shell metacharacters are present, or when any path argument (`cat ../outside.txt`, `cat ~/.ssh/id_ed25519`) leaves `granted_folders` | `ok` |
| `rpc_and_loopback_http_studio_host_serve_commands_and_static_assets` | Verifies `run_self_test_receipt()`, `handle_rpc_line` (including refusal of `native_set_terminal_enabled` and `native_grant_folder` over page RPC), and `handle_studio_http_request` (`GET /` serving `web/studio/index.html` with injected `UOR_DESKTOP_SESSION_TOKEN`, `403` on `/../Cargo.toml`, `403` on missing/wrong `X-UOR-Desktop-Token`, foreign `Origin`, foreign `Host`, or non-JSON `Content-Type`, and `200` on authenticated `POST /__tauri_invoke`) | `ok` |

### 2.2 Native CLI Self-Test (`uor-desktop-client --self-test`)

Command:

```sh
CARGO_TARGET_DIR=local/tauri-target ~/.cargo/bin/cargo run --manifest-path src-tauri/Cargo.toml --locked -- --self-test
```

Output:

```json
{"artifactBytes":469252,"artifactSha256":"d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df","durableScope":"local-user/studio/self-test","engine":"uor-r4-integer/native-desktop","objectVersion":"bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca","storeRecords":446,"streamedCount":32,"tokens32":[38,9,33,8,25,38,30,25,17,30,30,30,17,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30],"ur4sess1Bytes":9782,"verdict":"PASS","weightsPerToken":15968}
```

### 2.3 Offline Studio & Real `--stdio` Bridge Verification (`web/studio/verify_desktop_client.mjs`)

Command:

```sh
node web/studio/verify_desktop_client.mjs
```

Output:

```json
{"verdict":"ALL DESKTOP CLIENT & OFFLINE STUDIO CHECKS PASSED","offlineExternalUrls":0,"streamedTokensCount":32,"diffLineChangesCount":5,"threeRenderCount":1,"invokedCommandsCount":38,"selfTestReceipt":{"artifactBytes":469252,"artifactSha256":"d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df","durableScope":"local-user/studio/self-test","engine":"uor-r4-integer/native-desktop","objectVersion":"bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca","storeRecords":446,"streamedCount":32,"tokens32":[38,9,33,8,25,38,30,25,17,30,30,30,17,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30],"ur4sess1Bytes":9782,"verdict":"PASS","weightsPerToken":15968},"remoteRequests":0}
```

## 3. Six-Column Research Notebook Rows

Because `docs/research/notebook/` is outside the `scope:` declaration on [`docs/roadmap/items/infra-desktop-client-tauri.md`](../../roadmap/items/infra-desktop-client-tauri.md), the two six-column research notebook rows for `m4-serving.md` and `m3-durable-memory.md` are recorded here for verbatim inclusion when a notebook-scoped item lands:

### Row for `docs/research/notebook/m4-serving.md`

| Date | Decision / Issue | Change | Result | Classification | Artifact / Commit |
|---|---|---|---|---|---|
| 2026-10-22 | `#2826` (`infra-desktop-client-tauri`, M4 `#2032` / M7 `#2035`) | Implemented `src-tauri/` native Rust desktop backend + `--stdio` / `--serve-studio` Studio host linking `uor-r4-integer` (`IntegerStackModel`, `LutStackSession`) and self-hosted Studio offline vendor shims (`uor_offline_vendor.js`, `uor_offline_fonts.css`) with zero external URLs | `6/6` integration controls pass; 32-token synthetic stand-in (`seed = 5`, `469,252 B`, `sha256:d60f88d6...`) matches `docs/wasm-serving/verify.mjs` token-for-token (`[38,9,33,8,25,38,30,25,17,30,30,30,17,30...]`, `15,968 B` weights/tok, `32` step-by-step `stream_token` events over real `--stdio` IPC); `0` remote requests | **Proof / finite host check** (synthetic stand-in only; no trained model or decode speed claim) | `src-tauri/src/lib.rs`, `web/studio/verify_desktop_client.mjs`, [`docs/labs/infra-desktop-client-tauri/README.md`](./README.md) |

### Row for `docs/research/notebook/m3-durable-memory.md`

| Date | Decision / Issue | Change | Result | Classification | Artifact / Commit |
|---|---|---|---|---|---|
| 2026-10-22 | `#2826` (`infra-desktop-client-tauri`, M3 `#2031` / D19) | Wired `native_save_session_durable` and `native_restore_session_durable` in `src-tauri/` to fold and verify `UORMEM01` envelopes containing `UR4SESS1` session snapshots in `uor-r4-object-store::Store` inside granted workspace folders | 32-token synthetic session folds into `446` `UOROBJ01` records (`UR4SESS1` `9,782 B`, `object_version = bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca`); `16 + 16` close-and-resume across fresh `DesktopBackend` instances matches the unsplit 32-token trajectory token-for-token | **Proof / finite host check** (synthetic stand-in only; M3 `0/8` until tested on a trained model) | `src-tauri/src/lib.rs`, `src-tauri/tests/desktop_client_controls.rs`, [`docs/labs/infra-desktop-client-tauri/README.md`](./README.md) |
