# ORDER: replace one-shot Terminal with a real interactive PTY, kimi-4

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `f86f4c1165d71cd553dd179d8f37db5cfc19b619`, after the canonical launch repair. Owner direction: "make it a real terminal not a bullshit one button press one", with https://github.com/tsl0922/ttyd as the reference.

## Decision review

The current panel calls `native_run_terminal_command` once and displays text; it is not a terminal. Implement the approved streaming-terminal plan using ttyd's PTY/writable-terminal model without its network-service exposure. The selected design and executable plan are `docs/superpowers/specs/2026-11-08-interactive-terminal-design.md` and `docs/superpowers/plans/2026-11-08-interactive-terminal.md`. The old one-shot API remains for compatibility; it is not the interactive UI.

## Files in scope

- `src-tauri/Cargo.{toml,lock}`, `src-tauri/src/{lib,main,terminal,process_control}.rs`, and `src-tauri/tests/interactive_terminal.rs`. `src-tauri/tests/mcp_protocol.rs` reuses only the predecessor's test-fixture stabilization, not its ORDER or production changes.
- `web/studio/src/{components/DesktopTools.tsx,components/DesktopTools.test.tsx,components/TerminalSession.tsx,components/TerminalSession.test.tsx,components/StudioApp.tsx,components/StudioApp.test.tsx,adapters/desktopActions.ts,studio.css}`, `web/studio/assets/js/desktop_actions.js`, `web/studio/tests/e2e/terminal.spec.ts`, `web/studio/verify_interactive_terminal.mjs`, `web/studio/verify_studio_page_load.mjs` (opt-in retained-tab acceptance only), and `web/studio/package{,-lock}.json`.
- `docs/superpowers/{specs,plans}/2026-11-08-interactive-terminal*.md`, this task folder, `docs/roadmap/items/studio-ide-harness.md`, `docs/products/{desktop-client,harness-parity}.md`, `docs/integration/current-state.md`, `docs/research/notebook/m4-serving.md`, and `web/studio/README.md`.
- Ignored fixture/evidence under `local/kimi/harness-parity/`.

Explicitly authorized new runtime dependencies, exact pins: `portable-pty = "0.9.0"`, `@xterm/xterm = "6.0.0"`, `@xterm/addon-fit = "0.11.0"`. No ttyd install, public socket, CDN, or other dependency. Do not edit R4 API/model/engine/worker/tokenizer files or the predecessor worktree.

## Done when

- Red-first tests prove approval/refusal, actual PTY open, multiple direct inputs and a `read` reply, cwd/isolated environment, resize, output/input/session bounds, close/reap, and later-input refusal. Existing one-shot behavior remains compatible.
- Xterm displays raw streamed bytes without HTML execution, accepts direct keyboard input, resizes, and has no Run command button. Workspace change/unmount/close stop input, kill/reap, and remove listeners/pollers; stale events cannot alter the next view.
- Actual native PTY acceptance exercises `printf`, `read` plus an answer, `pwd`, `stty size`, and `exit` with exact source/host/event identities. Static web without a session service stays explicitly unavailable; no detached-process-containment claim.
- Typed/plain-JS/default/GUI/browser/repository checks pass, reports preserve all negatives, fresh exact-head review and protected delivery complete, then the shared agent runtime follows.

## Stop and report

Stop before spawning an unapproved/out-of-grant shell, weakening a security assertion, hiding a failed run, using real credentials/profiles/endpoints, or claiming a real terminal from a mock. Keep evidence of failed approaches and preserve old unique state. Report size and dependencies explicitly; do not silently add another library or broaden the claimed capability.
