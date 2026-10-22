# REPORT: infra-desktop-client-tauri, antigravity-24

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Planted-fault set-aside check on `handle_studio_http_request`:** Temporarily disabling the `Origin` / `Content-Type` / `X-UOR-Desktop-Token` guard in [`src-tauri/src/lib.rs`](../../../../../src-tauri/src/lib.rs) (`if false && (!origin_ok || !content_type_ok || !token_ok)`) and running `cargo test --manifest-path src-tauri/Cargo.toml --locked --test desktop_client_controls rpc_and_loopback_http_studio_host_serve_commands_and_static_assets` caused `rpc_and_loopback_http_studio_host_serve_commands_and_static_assets` to fail immediately at `tests/desktop_client_controls.rs:520:5: assertion left == right failed: left: 200, right: 403`. Reverting the planted fault restored `6 passed; 0 failed`.
- **Page-invoked `native_grant_folder` in `verify_desktop_client.mjs` and `web/studio/index.html`:** Once `native_grant_folder` was made host-owned in [`src-tauri/src/lib.rs`](../../../../../src-tauri/src/lib.rs) (`dispatch_rpc_command` refuses `native_grant_folder` over page RPC), `verify_desktop_client.mjs` and `loadTauriGitWorkspace` in `web/studio/index.html` could no longer self-grant folders from page JavaScript. Updating `uor-desktop-client` to accept repeatable `--grant <dir>` CLI flags on the host, updating `loadTauriGitWorkspace` to query `native_list_granted_folders`, and updating `verify_desktop_client.mjs` to pass `['--stdio', '--grant', tmpGrantedStoreRoot]` while explicitly asserting that page-invoked `native_grant_folder` is refused resolved this cleanly.

## What changed

- **Code (`+232 / -32` lines across 4 files at `6a360204d`):**
  - [`src-tauri/src/lib.rs`](../../../../../src-tauri/src/lib.rs):
    - Added `generate_launch_token()` producing a 256-bit (64-hex-character) per-launch token from `/dev/urandom` + process/timestamp entropy.
    - Updated `handle_studio_http_request` to require `Host` matching `bound_host` (`127.0.0.1:<port>`) on every HTTP request, and on `POST /__tauri_invoke` to require `Origin == http://<bound_host>`, `Content-Type: application/json`, and `X-UOR-Desktop-Token == expected_token` (returning `403 Forbidden` otherwise), and injected `window.UOR_DESKTOP_SESSION_TOKEN` alongside `window.UOR_ENABLE_LOOPBACK_TAURI = true` in the served `index.html`.
    - Refused `native_grant_folder` in `dispatch_rpc_command` so folder grants are strictly host-owned (`--grant <dir>` CLI flag or native OS file dialog) and cannot be self-granted over page RPC.
  - [`src-tauri/src/main.rs`](../../../../../src-tauri/src/main.rs):
    - Added repeatable `--grant <dir>` CLI flag handling before starting `--stdio` or `--serve-studio`, generated the per-launch token in `serve_studio_http`, and enforced `Host`, `Origin`, `Content-Type`, and `X-UOR-Desktop-Token` via `handle_studio_http_request`.
  - [`web/studio/assets/js/uor_offline_vendor.js`](../../../../../web/studio/assets/js/uor_offline_vendor.js) and [`web/studio/index.html`](../../../../../web/studio/index.html):
    - Updated `createLoopbackTauriCore` to send `'X-UOR-Desktop-Token': String(globalScope.UOR_DESKTOP_SESSION_TOKEN || '')` on `POST /__tauri_invoke`.
    - Updated `loadTauriGitWorkspace` in `web/studio/index.html` to query host-granted folders via `native_list_granted_folders` instead of calling `native_grant_folder` from page script.
- **Tests (`+139 / -13` lines across 2 files):**
  - [`src-tauri/tests/desktop_client_controls.rs`](../../../../../src-tauri/tests/desktop_client_controls.rs): expanded `rpc_and_loopback_http_studio_host_serve_commands_and_static_assets` to verify that `native_grant_folder` is refused over `handle_rpc_line`, that `GET /` injects `UOR_DESKTOP_SESSION_TOKEN`, that `POST /__tauri_invoke` returns `403` on missing token, wrong token, foreign `Origin` (`https://evil.example`), non-JSON `Content-Type` (`text/plain`), or DNS-rebinding `Host` (`evil.rebind.attacker:14200`), and that `POST /__tauri_invoke` succeeds (`200`) when all headers match.
  - [`web/studio/verify_desktop_client.mjs`](../../../../../web/studio/verify_desktop_client.mjs): spawns `uor-desktop-client --stdio --grant <tmpGrantedStoreRoot>`, asserts `index.html` never calls `native_grant_folder`, verifies `realStdioTauriCore.invoke('native_grant_folder', ...)` is refused over RPC, verifies `PathNotGranted` on `tmpUngrantedStoreRoot`, and verifies durable save/restore on `tmpGrantedStoreRoot`.
- **Docs (`+3 / -3` lines across 2 files):**
  - [`docs/products/desktop-client.md`](../../../../products/desktop-client.md) and [`docs/labs/infra-desktop-client-tauri/README.md`](../../../../labs/infra-desktop-client-tauri/README.md): documented host-owned `--grant <dir>` and the `--serve-studio` per-launch `X-UOR-Desktop-Token` + `Host`/`Origin`/`Content-Type` enforcement.
- **Ratio against ORDER estimate:** [`ORDER.md`](ORDER.md) estimated ~220 lines across 7 files; the repair touched 8 files (`+353 / -47` lines, net `+306` lines, `1.39x` net).

## Measurements

- **Native Rust integration controls (`CARGO_TARGET_DIR=local/tauri-target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --locked`):** `6 passed; 0 failed` in `src-tauri/tests/desktop_client_controls.rs`.
- **Real `--stdio` & `index.html` DOM verification (`node web/studio/verify_desktop_client.mjs`):**
  - `verdict = "ALL DESKTOP CLIENT & OFFLINE STUDIO CHECKS PASSED"`, `offlineExternalUrls = 0`, `streamedTokensCount = 32`, `diffLineChangesCount = 5`, `threeRenderCount = 1`, `invokedCommandsCount = 38`, `remoteRequests = 0`.

## Checks run at the head

- `~/.cargo/bin/cargo fmt --manifest-path src-tauri/Cargo.toml --check`: exit `0`
- `CARGO_TARGET_DIR=local/tauri-target ~/.cargo/bin/cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings`: exit `0` (`0` warnings)
- `CARGO_TARGET_DIR=local/tauri-target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --locked`: `6 passed; 0 failed`
- `node web/studio/verify_desktop_client.mjs`: exit `0` (`ALL DESKTOP CLIENT & OFFLINE STUDIO CHECKS PASSED`)
- `uor-harness loop report --check`: `PASS`
- `uor-harness loop pr-check --local`: `PASS`
- `uor-harness loop gates --full`: `GREEN`

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)

## Still open

- Vendor-fetching the `tauri` v2 crate tree into `Cargo.lock` and wiring `tauri::Builder` / `#[tauri::command]` WebView window hosting, plus copying the two six-column research notebook rows from [`docs/labs/infra-desktop-client-tauri/README.md`](../../../../labs/infra-desktop-client-tauri/README.md) (`§3`) into `docs/research/notebook/m4-serving.md` and `docs/research/notebook/m3-durable-memory.md` before marking `#2826` (`infra-desktop-client-tauri`) `done`. References #2826.

## Plain summary

This repair (`antigravity-24`) resolves the loopback HTTP security finding from review `6351343795` on PR `#2990` (`#2826`). First, `uor-desktop-client --serve-studio` now generates a 256-bit per-launch random token (`X-UOR-Desktop-Token`), embeds it into the served Studio `index.html`, and requires every `POST /__tauri_invoke` request to carry the matching `X-UOR-Desktop-Token`, `Origin: http://127.0.0.1:<port>`, `Host: 127.0.0.1:<port>`, and `Content-Type: application/json`, blocking cross-origin browser `no-cors` form/fetch requests and DNS rebinding. Second, `native_grant_folder` is now strictly host-owned via the `--grant <dir>` CLI flag and refused over page RPC so an unprivileged caller or script cannot self-grant arbitrary folders on the user's disk. Third, we verified these controls with planted-fault tests in `src-tauri/tests/desktop_client_controls.rs` and `web/studio/verify_desktop_client.mjs`.
