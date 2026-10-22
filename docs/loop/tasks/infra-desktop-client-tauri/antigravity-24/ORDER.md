# ORDER: infra-desktop-client-tauri, antigravity-24

Lab: antigravity. Item: [infra-desktop-client-tauri](../../../../roadmap/items/infra-desktop-client-tauri.md) (#2826). Base: `18c03cebc`.

## Goal

Repair the loopback Studio host (`/--serve-studio`) finding from review `6351343795` on PR `#2990` at `5edb1d38ec3fe485c48d1f470cd78800ba3c28e7`.

## Cancel probe (run first)

At `5edb1d38e`, `handle_studio_http_request` accepts `/__tauri_invoke` without a per-launch token or `Host`/`Origin` check, and `dispatch_rpc_command` exposes `native_grant_folder` to page RPC. Result: live (`code + docs`, est. ~250 lines across 6 files).

## Files in scope

- `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, `src-tauri/tests/desktop_client_controls.rs`:
  1. Require a per-launch 256-bit token (`X-UOR-Desktop-Token` header or `"token"` RPC field) on `/__tauri_invoke`, and embed `window.UOR_DESKTOP_SESSION_TOKEN` in `index.html` when served by `--serve-studio`.
  2. Refuse any `/__tauri_invoke` or static request whose `Host` header does not match the bound `127.0.0.1:<port>` or whose `Origin` header (on `POST /__tauri_invoke`) does not match `http://127.0.0.1:<port>` (plus require `Content-Type: application/json`).
  3. Make `native_grant_folder` host-owned (`--grant <dir>` CLI flag on `uor-desktop-client`, refused over page RPC).
  4. Add tests for missing/wrong token, foreign `Host`, foreign `Origin`, non-JSON `Content-Type`, and page RPC `native_grant_folder` refusal, and record the planted-fault deliberate break in `REPORT.md`.
- `web/studio/assets/js/uor_offline_vendor.js`, `web/studio/index.html`, `web/studio/verify_desktop_client.mjs`: pass `X-UOR-Desktop-Token` in `createLoopbackTauriCore` and pass `--grant <dir>` to `uor-desktop-client --stdio`.
- `docs/products/desktop-client.md`, `docs/labs/infra-desktop-client-tauri/README.md`, and `docs/loop/tasks/infra-desktop-client-tauri/antigravity-24/REPORT.md`.

## Done when

1. `cargo test --manifest-path src-tauri/Cargo.toml --locked` (`7/7` controls) and `node web/studio/verify_desktop_client.mjs` pass.
2. Planted-fault deliberate break fails when the token or `Host`/`Origin` check is removed.

## Stop and ask if

- Never needed inside this repair scope.
