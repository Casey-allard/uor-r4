# Final mounted Studio acceptance design

## Context

PR #3862 merged one Vite `dist` for the typed preview, `native-runner.sh --local`, and unsigned Tauri. The remaining Phase A gap is proof and repair of complete browser/desktop workflows, not another UI rewrite. This unit does not publish production or begin Phase B.

## Decisions

1. **Process truth.** `run_bounded_process` keeps its signature and limits. When pipes close, it may wait only until the existing deadline for a normal child exit; it must still terminate a child that remains running at the deadline. Descendant-pipe and output-cap behavior stays bounded and truthful.
2. **One existing loopback host.** Add optional `--studio-root <dir>` to `uor-desktop-client --serve-studio`, defaulting to the current `web/studio`. Use `web/studio/dist` for acceptance. Existing token, Host, Origin, canonical-path, and static-file checks are unchanged.
3. **Host-only approval.** Add `--enable-terminal` and `--approve-command <cwd> <command>` as explicit host CLI controls using `DesktopBackend::native_set_terminal_enabled` and `host_approve_terminal_command`. These are not test hooks and do not bypass the 60-second, one-shot, exact cwd/command grant. Browser RPC only reads state through `host_terminal_command_is_approved`; page RPC cannot enable or approve. A normal Run consumes the grant exactly once.
4. **Capability matrix.** Static Pages use local WASM and browser file APIs with explicit unsupported native Git/terminal. `uor-chat` native runner retains its current native-generation/session path and explicit host-action limitations. The existing desktop loopback host supplies full browser folder/Git/approved-command behavior. Packaged Tauri uses native IPC and per-process grants. No second service or new protocol is added.
5. **Evidence.** A CDP-driven real Chromium workflow and actual built-in-browser inspection exercise the loopback host. macOS accessibility repeats packaged Tauri workflows. Verify restored sessions, attachments, editor bytes, and non-timeout Git inspection without using real browser data or credentials. Preserve all negatives and generated evidence under `local/kimi/studio-final-acceptance/`.
6. **Delivery.** Use red-first tests, complete acceptance at an exact committed head, fresh `uor-delivery-reviewer`, protected merge, and detached fresh-main checks. Typed production publication and final launch acceptance are a separate following unit.

## Rejected shortcuts

- No browser-side enable/approve mutation, persistent approval, broader command allowlist, real credential, external endpoint, source-TypeScript mount, second UI, or new dependency.
- No deleting the timeout observation, accepting fixture-only behavior as GUI evidence, or treating static/native unsupported labels as successful native actions.
- No production publish, signed-release claim, useful-chat/M7/M8/accessibility-certification claim, or Phase B compute.
