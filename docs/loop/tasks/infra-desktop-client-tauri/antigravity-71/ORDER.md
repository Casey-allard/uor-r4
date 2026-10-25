# ORDER: infra-desktop-client-tauri, antigravity-71

Lab: antigravity. Item: [infra-desktop-client-tauri](../../../../roadmap/items/infra-desktop-client-tauri.md) (#2826). Base: `f22a3097a55a0d4f9cdef5736554878222cd30ee`.

## Goal

The item's done-when reads:

> A Tauri app sharing the web interface that runs the integer engine natively, offline, with durable memory, matching the written specification. Until a trained model exists, "runs the engine" means the synthetic stand-in.

This unit completes the Tauri v2 WebView shell wiring (`#[cfg(feature = "gui")]` with `tauri = "2"` and `tauri-build = "2"`, `#[tauri::command]` handlers for all 11 native desktop commands, `tauri::Builder` state management, icon assets, and `src-tauri/Cargo.lock`), verifies both the default headless/CI build (`cargo test --manifest-path src-tauri/Cargo.toml --locked`) and the native macOS Tauri v2 GUI build (`cargo test --manifest-path src-tauri/Cargo.toml --features gui --locked`) plus `node web/studio/verify_desktop_client.mjs`, updates `docs/products/desktop-client.md`, `docs/products/README.md`, and `docs/labs/infra-desktop-client-tauri/README.md`, and closes `#2826` (`status: done`).

## Cancel probe (run first)

Ran `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml --locked --no-fail-fast` and `node web/studio/verify_desktop_client.mjs`, and inspected `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, and `#2826` (`issuecomment-6384884315`).
- **Probe result (classified `instrument` — missing Tauri v2 `gui` feature + `#[tauri::command]` / `tauri::Builder` shell wiring in `src-tauri/`):**
  1. All `6/6` existing native integration controls (`src-tauri/tests/desktop_client_controls.rs`) and `node web/studio/verify_desktop_client.mjs` (`38` invoked commands, `32` streamed tokens, `0` external URLs, `446` `UOROBJ01` records) pass on the `seed = 5` synthetic stand-in (`469,252 B`, `sha256:d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`).
  2. As identified in `#2826` (`issuecomment-6384884315`), the only remaining gap keeping `infra-desktop-client-tauri` open is wiring the feature-gated `tauri = "2"` + `tauri-build = "2"` WebView shell (`gui` feature in `src-tauri/Cargo.toml`, `tauri_build::build()` in `src-tauri/build.rs`, `#[tauri::command]` wrappers and `tauri::Builder` entrypoint in `src-tauri/src/{lib,main}.rs`, icon bundle assets, and locked `src-tauri/Cargo.lock`) while keeping the default feature set GUI-free so headless Linux CI runs without system WebKit libraries.

## Files in scope

- `src-tauri/**`
- `web/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-object-store/**`
- `docs/products/**`
- `Cargo.toml`
- `docs/labs/infra-desktop-client-tauri/**`
- `docs/loop/tasks/infra-desktop-client-tauri/**`

## Done when

- `src-tauri/` links `tauri = "2"` and `tauri-build = "2"` under `--features gui` (`#[tauri::command]` + `tauri::Builder` shell) with `#![forbid(unsafe_code)]`, passes `cargo test --manifest-path src-tauri/Cargo.toml --locked` (both default and `--features gui`) and `node web/studio/verify_desktop_client.mjs`, and sets `docs/roadmap/items/infra-desktop-client-tauri.md` to `status: done` (`Closes #2826`).

## Stop and ask if

- Any change introduces softmax, squashing, dense matmul, or `unsafe` code in `src-tauri/`.
