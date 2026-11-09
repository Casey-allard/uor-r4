# Interactive PTY Terminal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task in the lead session. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the one-command Terminal panel with a real interactive PTY and streamed terminal UI.

**Architecture:** A bounded native PTY session owns the child, writer, reader thread, and sequenced raw-byte events. The workbench opens a session once with host approval, then sends direct keyboard input and streams bytes into xterm. Existing one-shot commands remain a separate compatibility API.

**Tech Stack:** Existing Rust/Tauri/Preact/Vite/TypeScript, `portable-pty = "0.9.0"`, `@xterm/xterm = "6.0.0"`, `@xterm/addon-fit = "0.11.0"`.

**Spec:** `docs/superpowers/specs/2026-11-08-interactive-terminal-design.md` and `docs/loop/tasks/studio-ide-harness/kimi-4/ORDER.md`.

## Global Constraints

- Keep host-owned terminal enablement and per-shell approval. No page-created authority, arbitrary startup command, public socket, real credential/profile, or external endpoint.
- Preserve one-shot `native_run_terminal_command` for compatibility; no per-keystroke approval or Run command button in the interactive panel.
- Hex-encode raw PTY bytes; never render them as HTML. Bound sessions, input, retained output, and dimensions. Stop/reap/clean up on close, workspace change, and unmount.
- Pin only the three declared dependencies. Do not edit the R4 API/model/engine/worker/tokenizer or install ttyd.
- Static web has no interactive PTY until the harness service implements this contract. No full Cowork parity or useful-model claim.

## Review Focus

- Unapproved/out-of-grant shell starts and non-shell startup programs: refuse before spawning (Task 1).
- Partial UTF-8/control sequences and HTML-looking output: preserve raw bytes for xterm, not DOM markup (Tasks 1–2).
- Multiple writes and read prompts: one persistent PTY handles them without another command approval (Tasks 1 and 3).
- Stale events, workspace changes, close and unmount: no later write or event changes the new view (Tasks 1–2).
- Output floods, oversized input and excessive sessions: explicit bounds and refusals, not unbounded memory/processes (Task 1).

---

### Task 1: Persistent native PTY session

**Files:**
- Create: `src-tauri/src/terminal.rs`, `src-tauri/tests/interactive_terminal.rs`
- Modify: `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`

**Interfaces:**
- `open_terminal(cwd, shell, rows, cols) -> TerminalSnapshot`
- `read_terminal(session_id, after_sequence) -> TerminalSnapshot`
- `write_terminal(session_id, bytes)`, `resize_terminal(session_id, rows, cols)`, `close_terminal(session_id)`
- `host_approve_terminal_shell(cwd, shell)` and `--approve-shell <cwd> <shell>` for explicit fixture/operator approval.

- [ ] Write integration tests for refusal before approval, real PTY open, multiple inputs and a `read` reply, environment/cwd, resize, close and later-input refusal, bounded output/input/session count. Run them and watch the missing API fail.
- [ ] Add `portable-pty = "0.9.0"` exactly. Implement a session map with child, PTY master/writer, reader thread, monotonic sequence, raw-byte hex events, bounded retained output, and explicit close/reap. Keep independent one-shot behavior unchanged.
- [ ] Expose the named native commands and host-only shell approval. Re-run focused tests and complete default/GUI suites; check formatting.

### Task 2: Streaming xterm panel and adapter

**Files:**
- Create: `web/studio/src/components/TerminalSession.tsx`, `web/studio/src/components/TerminalSession.test.tsx`, `web/studio/tests/e2e/terminal.spec.ts`
- Modify: `web/studio/src/components/DesktopTools.tsx`, `web/studio/src/components/DesktopTools.test.tsx`, `web/studio/src/adapters/desktopActions.ts`, `web/studio/assets/js/desktop_actions.js`, `web/studio/src/studio.css`, `web/studio/package.json`, `web/studio/package-lock.json`, `web/studio/src/components/StudioApp.tsx`

**Interfaces:**
- `openTerminal(cwd, rows, cols)`, `readTerminal(id, after)`, `inputTerminal(id, bytes)`, `resizeTerminal(id, rows, cols)`, `closeTerminal(id)`.
- `TerminalPanel` uses the same public props as before plus an optional test terminal factory.

- [ ] Add exact `@xterm/xterm` and `@xterm/addon-fit` dependencies. Write failing component tests for streamed bytes, direct input, no Run command button, accessible status, close/workspace/unmount cleanup, stale events, and inert HTML-looking output. Replace the old one-command tests with equivalent or stronger session cases.
- [ ] Implement the adapter and `TerminalPanel`: enable/open once, xterm write/onData, fit/resize, bounded read cursors, explicit close, and listener/poller cleanup. Static browser without native/session support is explicit.
- [ ] Run focused tests, all typed tests, typecheck and Vite build, then desktop/mobile Playwright. Do not treat a fake terminal as a real PTY result.

### Task 3: Live native acceptance, records, and delivery

**Files:**
- Modify: `web/studio/README.md`, `docs/products/desktop-client.md`, `docs/products/harness-parity.md`, `docs/research/notebook/m4-serving.md`, `docs/integration/current-state.md`, `docs/loop/tasks/studio-ide-harness/kimi-4/{ORDER,REPORT}.md`

**Interfaces:** measured PTY session evidence with exact source, host executable, working directory, shell, and event identities.

- [ ] Start an isolated fixture PTY and exercise `printf`, `read` plus a typed answer, `pwd`, `stty size`, and `exit` through the actual native/xterm path. Confirm no Run command button and no process after close.
- [ ] Run all existing typed/plain-JS/default/GUI/offline checks and repository gates; retain failures and exact bounds. Do not claim detached-process containment or a hosted interactive service.
- [ ] Record what is real versus unimplemented, obtain fresh exact-head review, deliver through protected PR/merge, and verify fresh main. Continue the shared agent runtime afterward.
