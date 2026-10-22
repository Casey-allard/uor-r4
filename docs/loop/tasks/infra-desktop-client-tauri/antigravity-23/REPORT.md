# REPORT: infra-desktop-client-tauri, antigravity-23

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Inner doc comments in `include!("../../crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs")`:** Replacing the duplicated `StandinLcg` / `StandinBuilder` in [`src-tauri/src/lib.rs`](../../../../../src-tauri/src/lib.rs) with a direct `include!("../../crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs")` failed with `error[E0753]: expected outer doc comment` because `mk-synthetic-stack.rs` starts with 12 lines of inner module doc comments (`//!`). Adding [`src-tauri/build.rs`](../../../../../src-tauri/build.rs) to normalize `//!` to `//` into `$OUT_DIR/mk_synthetic_stack_included.rs` (with `cargo:rerun-if-changed`) allowed `src-tauri` to compile the canonical `mk-synthetic-stack.rs` builder directly with zero code duplication.
- **Loopback HTTP traversal check on relative `studio_root` (`403` vs `200`):** The initial run of `rpc_and_loopback_http_studio_host_serve_commands_and_static_assets` in [`src-tauri/tests/desktop_client_controls.rs`](../../../../../src-tauri/tests/desktop_client_controls.rs) failed (`left: 403, right: 200`) because `handle_studio_http_request` called `reject_parent_dir_traversal(&candidate)` where `studio_root` itself contained `../web/studio`. Checking `reject_parent_dir_traversal(Path::new(rel))` on the request path and canonicalizing `studio_root` fixed the false positive while keeping `GET /../Cargo.toml` blocked with `403`.
- **Deliberate-break set-aside check (`reject_parent_dir_traversal`):** Temporarily replacing `reject_parent_dir_traversal` in [`src-tauri/src/lib.rs`](../../../../../src-tauri/src/lib.rs) with `Ok(())` and running `cargo test --manifest-path src-tauri/Cargo.toml --locked --test desktop_client_controls` caused 2 tests to fail immediately (`scoped_file_and_git_commands_enforce_granted_folder_and_refuse_dangling_symlinks` at `tests/desktop_client_controls.rs:281:5: assertion failed: matches!(trav_err, DesktopClientError::PathTraversalRefused(_))` and `rpc_and_loopback_http_studio_host_serve_commands_and_static_assets` at `tests/desktop_client_controls.rs:444:5: left: 404, right: 403`). Reverting the break restored `6 passed; 0 failed`.

## What changed

- **Code (`+1,735 / -800` lines across 5 files at `3acd3af96`):**
  - [`src-tauri/build.rs`](../../../../../src-tauri/build.rs), [`src-tauri/src/lib.rs`](../../../../../src-tauri/src/lib.rs), and [`src-tauri/src/main.rs`](../../../../../src-tauri/src/main.rs):
    - Enforced symlink and dangling-symlink refusal (`reject_if_symlink`, `reject_symlinks_between_root_and_target`, `DesktopClientError::SymlinkRefused`) across `native_grant_folder`, `verify_granted_existing_path`, `verify_granted_write_path`, and `set_durable_store_dir`.
    - Enforced `granted_folders` confinement on `set_durable_store_dir`, kept `native_set_terminal_enabled` and `host_approve_terminal_command` host-owned (not callable over page RPC), and confined all path-like arguments (`/`-containing, `.`-prefixed, or `~`-prefixed) in `native_run_terminal_command` to `granted_folders`.
    - Passed `-c core.fsmonitor=false -c core.hooksPath=/dev/null` to `git` in `native_git_status` and reported `"HEAD (unborn)"` when `rev-parse` fails on an empty repository.
    - Stream-checked the 8-byte `b"UORLUT01"` magic header in `native_list_local_models`, skipped unpinned/non-`UORLUT01` files so unrelated `.bin` files cannot hide pinned models, deduplicated `add_model_search_dir`, capped `native_read_file` at `16 MiB`, and moved `dispatch_rpc_command`, `handle_rpc_line`, `run_self_test_receipt`, and `handle_studio_http_request` into `src-tauri/src/lib.rs`.
  - [`web/studio/assets/js/uor_offline_vendor.js`](../../../../../web/studio/assets/js/uor_offline_vendor.js) and [`web/studio/index.html`](../../../../../web/studio/index.html):
    - Added `monaco.editor.setModelLanguage`, `getModelLanguage`, `setTheme`, `defineTheme`, and a real LCS line-by-line unified diff (`createDiffEditor` with `.uor-diff-del`, `.uor-diff-ins`, and `getLineChanges()`).
    - Upgraded `marked.parse` (unordered/ordered lists, blockquotes, pipe tables, links, italics, bold, code), `katex.renderToString` + `renderMathInElement` (fractions, square roots, superscripts, subscripts, Greek letters, operators), `THREE` (2D-canvas software 3D projection shim), and `pdfjsLib` (ASCII text extraction + offline notice).
    - Gated `callGithubApi` off in desktop/offline mode while preserving online browser mode, and updated `createTauriNativeModelWorker` to stream tokens step-by-step (`maxNewTokens: 1` per IPC step).
- **Tests (`+568 / -185` lines across 2 files):**
  - [`src-tauri/tests/desktop_client_controls.rs`](../../../../../src-tauri/tests/desktop_client_controls.rs): expanded to 6 integration controls covering unpinned `.bin` skipping, granted-folder `storeDir` confinement, dangling/outside symlink refusal, empty-repo `"HEAD (unborn)"` reporting, terminal path-argument confinement (`cat ../outside.txt`, `cat ~/.ssh/id_ed25519`), and `handle_rpc_line` / `handle_studio_http_request`.
  - [`web/studio/verify_desktop_client.mjs`](../../../../../web/studio/verify_desktop_client.mjs): spawns the real `uor-desktop-client --stdio` child process (never skipping, zero mock `invoke`) and evaluates the `<script type="module">` from `web/studio/index.html` (`openFileInMonaco`, `switchEditorTab`, `toggleMonacoDiffView`, `renderRichMarkdown`, `initThreeVisualizer`, `callGithubApi`).
- **Docs & config (`+105 / -61` lines across 5 files):**
  - [`src-tauri/capabilities/default.json`](../../../../../src-tauri/capabilities/default.json), [`docs/products/desktop-client.md`](../../../../products/desktop-client.md), [`docs/products/README.md`](../../../../products/README.md), [`docs/labs/infra-desktop-client-tauri/README.md`](../../../../labs/infra-desktop-client-tauri/README.md), and [`docs/loop/tasks/infra-desktop-client-tauri/antigravity-22/ORDER.md`](../antigravity-22/ORDER.md).
- **Ratio against ORDER estimate:** [`ORDER.md`](ORDER.md) estimated ~500 lines across 9 files; the repair touched 12 files (`+2,190 / -861` lines, net `+1,329` lines, `2.66x` net, driven by the complete `THREE` 2D-canvas software 3D projection shim, the LCS diff engine in `uor_offline_vendor.js`, and the DOM + real `--stdio` harness in `verify_desktop_client.mjs`).

## Measurements

- **Native Rust integration controls (`CARGO_TARGET_DIR=local/antigravity/tauri-target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --locked`):** `6 passed; 0 failed` in `src-tauri/tests/desktop_client_controls.rs`.
- **Real `--stdio` & `index.html` DOM verification (`node web/studio/verify_desktop_client.mjs`):**
  - `verdict = "ALL DESKTOP CLIENT & OFFLINE STUDIO CHECKS PASSED"`, `offlineExternalUrls = 0`, `streamedTokensCount = 32`, `diffLineChangesCount = 5`, `threeRenderCount = 1`, `invokedCommandsCount = 38`, `remoteRequests = 0`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/antigravity/tauri-target ~/.cargo/bin/cargo fmt --manifest-path src-tauri/Cargo.toml --check`: exit `0`
- `CARGO_TARGET_DIR=local/antigravity/tauri-target ~/.cargo/bin/cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings`: exit `0` (`0` warnings)
- `CARGO_TARGET_DIR=local/antigravity/tauri-target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --locked`: `6 passed; 0 failed`
- `node web/studio/verify_desktop_client.mjs`: exit `0` (`ALL DESKTOP CLIENT & OFFLINE STUDIO CHECKS PASSED`)
- `uor-harness loop report --check`: `PASS`
- `uor-harness loop pr-check --local`: `PASS`
- `uor-harness loop gates --full`: `GREEN`

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`)

## Still open

- Vendor-fetching the `tauri` v2 crate tree into `Cargo.lock` and wiring `tauri::Builder` / `#[tauri::command]` WebView window hosting, plus copying the two six-column research notebook rows from [`docs/labs/infra-desktop-client-tauri/README.md`](../../../../labs/infra-desktop-client-tauri/README.md) (`§3`) into `docs/research/notebook/m4-serving.md` and `docs/research/notebook/m3-durable-memory.md` before marking `#2826` (`infra-desktop-client-tauri`) `done`. References #2826.

## Plain summary

This repair (`antigravity-23`) addresses all findings from review `6349725818` on PR `#2990` (`#2826`). In the Rust desktop backend (`src-tauri/`), we added symlink and dangling-symlink refusal across all folder-scoped operations, required the durable object-store directory (`uor-r4-object-store`, which stores conversation snapshots as content-addressed 32-byte Merkle records) to live inside a user-granted folder, made terminal enablement and per-command approval strictly host-owned while confining every path argument in a terminal command to granted folders, disabled Git repository hooks (`core.fsmonitor=false`, `core.hooksPath=/dev/null`) during status checks, stream-checked the 8-byte `UORLUT01` model header before reading files, and compiled the synthetic stand-in builder directly from `crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs` with zero code duplication. In the offline Studio page (`web/studio/`), we added `monaco.editor.setModelLanguage`, a real line-by-line unified diff viewer, full markdown list/table/link/blockquote formatting, LaTeX math symbol formatting, a 2D-canvas software 3D projection shim for `THREE`, step-by-step token streaming, and a verification script (`verify_desktop_client.mjs`) that drives the real `uor-desktop-client --stdio` binary and exercises the actual `web/studio/index.html` application functions.
