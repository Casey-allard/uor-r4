# Interactive PTY terminal design

## Intent and why

The current Terminal panel invokes one approved command and displays returned text. The owner explicitly requires a real terminal, not one button per command, and supplied `https://github.com/tsl0922/ttyd` as a reference. The target is the ttyd interaction model inside the existing UOR workbench: a persistent pseudo-terminal, streamed output, direct keyboard input, working-directory and shell state, resize, and explicit stop. It must work in the recovered native app and later use the same session contract through the harness service.

## Approaches

- **Selected: native `portable-pty` plus xterm over the existing native command channel.** A real PTY runs under a host-approved working directory. The frontend writes raw bytes to xterm and sends xterm input directly to that PTY. This preserves offline native use, approvals, and bounded sessions without a public socket.
- **Rejected: ttyd as a sidecar.** It provides PTY/WebSocket behavior but adds another process, network authentication and origin boundary. A public or silently started ttyd endpoint is not acceptable here.
- **Rejected: keep one-shot pipes.** They cannot supply a real interactive terminal's session state, control sequences, resize, or continuous input.

The existing one-shot command API remains for compatibility and bounded automation. It is not the interactive terminal UI.

## Session and authority contract

- Existing app-session terminal enablement is still required. Opening each PTY additionally requires explicit host approval of the canonical working directory and `/bin/zsh` or `/bin/sh`. A page-supplied boolean cannot create that approval.
- One approval opens one PTY; subsequent keystrokes are ordinary terminal input, not one-shot commands or new approvals. The default shell is `/bin/zsh`.
- Each session has a unique ID, monotonically increasing event sequence, bounded input size, bounded retained output, and a bounded session count. Stale session IDs/events cannot alter another terminal or a reopened workspace.
- Raw PTY bytes are hex-encoded on the native boundary and decoded by the frontend before `Terminal.write`. No terminal output is interpreted as HTML.
- Closing a session, changing its workspace, or unmounting its pane stops input, kills/reaps the child where supported, closes the PTY, and removes listeners/pollers. Later input is refused. A closed session cannot produce a new action.
- No real credential/profile, remote endpoint, public terminal socket, or claim of detached-process escape prevention is made. Tests use an isolated synthetic HOME and explicit fixture grants.

## Interfaces

- `DesktopBackend::open_terminal(cwd, shell, rows, cols) -> TerminalSnapshot`
- `DesktopBackend::read_terminal(session_id, after_sequence) -> TerminalSnapshot`
- `DesktopBackend::write_terminal(session_id, bytes) -> Result<()>`
- `DesktopBackend::resize_terminal(session_id, rows, cols) -> Result<()>`
- `DesktopBackend::close_terminal(session_id) -> TerminalSnapshot`
- Native commands: `native_open_terminal`, `native_read_terminal`, `native_terminal_input`, `native_resize_terminal`, `native_close_terminal`.
- Browser adapter: `openTerminal`, `readTerminal`, `inputTerminal`, `resizeTerminal`, `closeTerminal`.

## Dependencies

Only these new runtime libraries are authorized: Rust `portable-pty = "0.9.0"` (MIT), JavaScript `@xterm/xterm = "6.0.0"` and `@xterm/addon-fit = "0.11.0"` (MIT). They are pinned exactly. No CDN, ttyd installation, new service, or other dependency is introduced.

## Acceptance

- Rust tests prove approval/refusal, actual PTY I/O across multiple inputs, isolated environment, session identity, resize, output/input bounds, stop, and no action after close.
- Component tests prove streaming bytes, direct input, accessible status, workspace cleanup, unmount cleanup, stale-event refusal, and no HTML execution. Existing one-shot tests are replaced by equivalent or stronger session tests, not silently removed.
- A real native PTY session accepts `printf`, `read`, an answer, `pwd`, `stty size`, and `exit` through the xterm input path, without a Run command button.
- Existing typed/plain-JS/default/GUI/browser checks and protected delivery pass. Static web without the session service remains explicitly unavailable; no full agent/runtime/API claim follows.
