# ORDER: infra-desktop-client-tauri, antigravity-23

Lab: antigravity. Item: [infra-desktop-client-tauri](../../../../roadmap/items/infra-desktop-client-tauri.md) (#2826). Base: `9753a285d`.

## Goal

> A Tauri app sharing the web interface that runs the integer engine natively, offline, with durable memory, matching the written specification. Until a trained model exists, "runs the engine" means the synthetic stand-in.

Repair PR `#2990` review `6349725818` (`References #2826`, est. ~1,500 lines).

## Cancel probe (run first)

At `9753a285d`, `uor-harness loop next` reports `GO repair #2990 — a counted CHANGES at its head stands (claude)` (`6349725818`). Result: live (`code + docs`).

## Calls

- Add `--serve-studio <addr>` loopback Studio host in `src-tauri` and relabel docs (`References #2826`).
- Fix `uor_offline_vendor.js` (`setModelLanguage`, real line diff, lists/links/italics/tables in `marked.parse`, `katex`, 2D-canvas `THREE`, step streaming), extract inline script so CSP drops `'unsafe-inline'`, and gate `callGithubApi` only in desktop/offline mode.
- Refuse symlinks (`symlink_metadata`) on file/dir grants, verify `storeDir` inside `grants`, move terminal confirmation/enablement to host state and confine path args to `grants`, pass `-c core.fsmonitor=false -c core.hooksPath=/dev/null` to `git`, stream-check `UORLUT01` header without failing the listing on unpinned `.bin` files, cap `native_read_file` at 16 MiB, deduplicate `model_dirs`, and test real binary + DOM in `verify_desktop_client.mjs` and `desktop_client_controls.rs`.

## Files in scope

- `src-tauri/**`, `web/**`, `crates/uor-r4-integer/**`, `crates/uor-r4-object-store/**`, `docs/products/**`, `Cargo.toml`, `docs/labs/infra-desktop-client-tauri/**`, `docs/loop/tasks/infra-desktop-client-tauri/antigravity-23/**`

## Done when

- `cargo test --manifest-path src-tauri/Cargo.toml --locked`, set-aside break checks, `node web/studio/verify_desktop_client.mjs`, `loop gates --full`, `report --check`, and `pr-check --local` pass.

## Stop and ask if

- Out-of-scope files, owner paths, or GPU compute are needed.
