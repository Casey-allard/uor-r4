# ORDER: complete browser and packaged Studio acceptance, kimi-71

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `496ae315224e1faa328ef2c7032d0581408fcf17`, tree `5cf1600295f8cf0982133eb9d2c7b820aec415fd`; original merge base is the same.

## Findings to repair

- The shared runtime is merged, but the complete Phase A workflow has not been demonstrated in a real browser. Static-WASM and `uor-chat` native-runner browser builds explicitly lack native Git/terminal actions; the existing desktop loopback host can supply them, but serves only `web/studio` rather than the reviewed `dist`.
- Packaged Changes displayed `HEAD (timeout)` and `diff timed out` while returning complete output. `run_bounded_process` can mistake pipe closure immediately before a normal child exit for a wall-clock timeout.
- The loopback host has no supported way for an operator to preauthorize a bounded terminal command for browser automation. Any acceptance path must reuse host-owned approval, preserve the 60-second one-shot grant and exact command/cwd match, and never let page RPC enable or approve itself.
- Existing packaged evidence proves session reopen, attachment, native save, Git output, and approved `pwd`, but final file/workspace state, non-timeout Git inspection, and complete acceptance evidence remain open.

## Selected approach and cancel probe

Keep one mounted Vite `dist` and the merged engine/model/session architecture. Add an optional `--studio-root` to the existing token-protected `--serve-studio` host. Reuse `DesktopBackend::native_set_terminal_enabled` and `host_approve_terminal_command` only through explicit host CLI arguments; page RPC merely reports existing enablement/approval and cannot mutate either. Exercise full browser workflows against that host, record explicit static-WASM/`uor-chat` limitations separately, and repeat packaged Tauri workflows with the process-race repair.

The cancel probe is a deterministic child that closes stdout/stderr before exiting, a browser attempt to enable or approve itself, a wrong/cancelled/expired command approval, and any attempt to serve source TypeScript or use an external test endpoint. No new dependency, second UI, external service, production publish, or Phase B work.

## Files in scope

- `src-tauri/src/main.rs` only for `--studio-root` and host-owned CLI preauthorization, `src-tauri/src/lib.rs` for the bounded process exit and read-only RPC approval state, and focused `src-tauri/tests/**`.
- `web/studio/verify_*.mjs` and directly related focused adapter/runtime tests or fixtures. Preserve `uor_model_worker.js`, tokenizer/artifact code, `studio_api.js`, engine behavior, and release defaults.
- `web/studio/README.md`, `docs/products/web-client.md`, `docs/products/desktop-client.md`, `docs/integration/current-state.md`, the roadmap item/notebook, the approved design, and this task folder.
- Task-owned evidence under `local/kimi/studio-final-acceptance/` and Cargo output only under `target/kimi-final-acceptance`.

No new dependency, real browser profile/credential, external test endpoint, production publisher, generated binary commit, owner-path edit, or model compute. Preserve existing static/native-runner capability labels, host token/Origin/grant checks, and the one-shot approval boundary.

## Done when

- Normal child exit after pipe closure is not labeled a timeout; actual timeout/output-cap/descendant boundaries still pass.
- The loopback host serves the reviewed dist with its existing token/Origin/grant checks. A real browser completes workspace selection, session create/resume, context attachment, actual UOR response or explicit unsupported state, activity/change inspection, approved command execution, edit/save, and reopened-state verification. Static-WASM and `uor-chat` differences remain explicit.
- Packaged Tauri repeats those workflows with native IPC, non-timeout Git inspection, and restored state; the app and its evidence are tied to an exact executable hash.
- Complete product/legacy/browser/desktop/repository checks, fresh exact-head approval, protected merge, and fresh-main verification pass. Typed production publication and final launch acceptance remain a separate following unit.

## Stop and report

Stop before weakening a permission check, letting page RPC create approval, adding a service/dependency, using real browser data or external endpoints, hiding an unsupported capability, or invoking production publication. Do not claim full Phase A, useful chat, M7/M8, accessibility certification, or Phase B from this unit.
