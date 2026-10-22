# ORDER: infra-desktop-client-tauri, antigravity-22

Lab: antigravity. Item: [infra-desktop-client-tauri](../../../../roadmap/items/infra-desktop-client-tauri.md) (#2826). Base: `0aac25aca`.

## Goal

> A Tauri app sharing the web interface that runs the integer engine natively, offline, with durable memory, matching the written specification. Until a trained model exists, "runs the engine" means the synthetic stand-in.

## Cancel probe (run first)

At `0aac25aca`, `src-tauri/` does not exist and `web/studio/index.html` loads 16 external CDN URLs. Result: live (`code + docs`, est. ~3,500 lines across `src-tauri/**`, `web/**`, and `docs/**`).

## Calls

- Build `src-tauri/` linking `uor-r4-integer` and `uor-r4-object-store`, verifying SHA-256 model pins before `format.rs`, granted-folder path scoping, durable `UR4SESS1` session fold/restore, and opt-in confirmed terminal commands.
- Make `web/studio/index.html` run offline via `uor_offline_vendor.js` and `uor_offline_fonts.css`, and document in `docs/products/` and `docs/labs/infra-desktop-client-tauri/README.md`.

## Files in scope

- `src-tauri/**`, `web/**`, `crates/uor-r4-integer/**`, `crates/uor-r4-object-store/**`, `docs/products/**`, `Cargo.toml`, `docs/labs/infra-desktop-client-tauri/**`

## Done when

- `cargo test --manifest-path src-tauri/Cargo.toml --locked`, `node web/studio/verify_desktop_client.mjs`, `loop gates --full`, `report --check`, and `pr-check --local` pass.

## Stop and ask if

- Out-of-scope files, owner paths, or GPU compute are needed.
