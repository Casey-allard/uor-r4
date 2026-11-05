# REPORT: unified Apple-like web and Tauri harness IDE

Lab: kimi. Item: `infra-desktop-client-tauri` (#2826). Delivery scope: development verification only; the expanded harness and hosted update are not yet delivered.

## What did not work

- The sidecar serves the synthetic stack, not useful conversation or visual understanding. No production-readiness, M3/M7/M8-completion, exact sidecar hidden-state resume, signed-distribution or single-binary claim is made.
- Normal app exit killed the app, `uor-chat` and stdio fixture. Earlier interrupted/forced shutdowns left sidecars that were stopped by exact PID; forced-kill cleanup is not claimed. The ORDER requires normal exit only.
- Cmd+R did not reload the Tauri page or end the stdio child, so it was not evidence about credential clearing. A real close/restart left the credential field empty and MCP disconnected, while the saved conversation remained.
- Model/file/durable-store operations can still serialize on the backend mutex. Blocking Tauri commands run on workers, keeping the main UI thread free; parallel storage access is not claimed. The terminal is an explicitly approved tool runner, not a general process or filesystem sandbox.
- The initial packaged native transport accepted origin aliases/non-default ports, dropped delayed/partial responses, panicked on async remote-MCP errors, mishandled cancellation, allowed attached-option and Git-hook escapes, and had unsafe durable-restore paths. Later review added Git filter-driver, process-group, readiness, non-SSE cancellation, UTF-8 error, absolute-static-path and tool-list cleanup repairs.
- Native file tabs originally shared only relative paths, so a same-named file in another project reused the old buffer. Root-aware `(path, rootPath)` identity now covers opening, switching, closing, persistence, new/inserted tabs, the palette and save. The first real A/B attempt exposed this before the focused regression was expanded.
- The first toolbar implementation did not record the initial Chat section, leaving Back disabled after the first navigation. Boot now initializes history from the current view. The duplicate New chat header action was removed.
- The new review test initially needed the repository's plain-JavaScript type-suppression convention; TypeScript then passed. One rebuild invocation used the preparation script from the wrong working directory and stopped before compiling; the corrected command built the app. These were instrument corrections, not model results.
- During GUI verification, an empty MCP URL produced `relative URL without a base`, and an executable outside the granted folder was refused. Entering the actual fixture URL and placing a byte-identical fixture inside the grant resolved those fixture/setup failures without weakening host checks.
- Exact-head independent review, protected merge, fresh-main verification, hosted Pages publication and the 30 GiB free-space target remain unfinished at this report's current state.

## What changed

- One staged frontend/backend state and one Tauri asset-origin family: `tauri://localhost` plus declared `http(s)://tauri.localhost` aliases with exact host/default-port and no userinfo. A host proxy reaches the pinned `uor-chat` sidecar through Tauri IPC; the page never receives its token or contacts it directly.
- Persistent byte buffers and 15-second idle deadlines preserve delayed/split JSON and SSE byte-for-byte. Readiness requires HTTP 200 with retries. Non-SSE reads honor cancellation, process groups receive TERM then KILL on timeout, and bounded process waits cannot hang after fd closure.
- Chat, sessions/history, long paste/attachments, native model list/test and exact image decomposition use the shared OpenAI transport. Request IDs are page-unique; cancellation and bounded event queues are explicit.
- A Claude Code desktop-inspired session toolbar now exposes sidebar, Back/Forward, native session selection, Changes, Terminal, Files, New chat and view options. Section history starts with the actual initial section, and every action uses the shared controller rather than decorative navigation.
- Native project/model-folder grants share file, bounded Git and terminal state. Same-named native files retain independent buffers and granted roots across project switches and persisted tabs. Guarded Git refuses configured filter drivers as well as hooks and external diff/process paths.
- Terminal enablement and each exact command use native approval, with a 60-second one-shot grant, direct allowlisted execution, metacharacter/attached-file-option refusal, hardened Git and bounded output/time.
- Durable manifests use collision-resistant scope filenames, same-directory temporary files and atomic rename; restore rejects `scopes` symlinks and verifies the granted manifest path. This older engine path is not represented as sidecar hidden-state resume.
- Browser remote MCP and native remote/stdio MCP implement bounded initialize/list/call, JSON/SSE normalization and explicit approval. Failed tool listing disconnects the session. Browser arguments use UTF-8 byte limits; native errors truncate by characters. Credentials remain memory-only.
- Runtime-only allowlists stage the same files for Tauri and Pages, excluding development sources/tests/configuration/declarations/verifiers/README/`node_modules`. Staging now rejects unexpected runtime files as well as missing allowlisted files.
- The design record separately assesses GitHub authentication. Device flow is a native candidate with a configured public OAuth client ID and no secret. Static Pages cannot embed a secret and direct browser device authorization was not established by a live CORS probe, so no sign-in control is claimed.

## Measurements

### Real native GUI evidence

The final packaged app was exercised through the actual macOS window and native sheets:

- Existing `Hi` history displayed 16 completion tokens, exact usage `19/16/35`, `14985.9 tok/s`, TTFT `0.459 ms`, total `1.068 ms`. This is the earlier synthetic timing record, not a new quality benchmark.
- At the merged head, a fresh GUI `Hi` returned 16 tokens, exact usage `19/16/35`, `22984.4 tok/s`, TTFT `0.348 ms`, total `0.696 ms`. Normal exit left no app or sidecar. This is synthetic protocol evidence, not useful conversation.
- Two projects each containing `note.txt` were opened through native folder grants. Project B displayed `Project B original`; switching back to A restored `Project A edited through native host`. Saving A produced exactly `Project A edited through native host\n` while B remained exactly `Project B original\n`. The UI reported `Saved to disk`.
- Remote MCP at `127.0.0.1:9000/mcp` showed `Connected · 1 tool`; the parented approval sheet explicitly said `A credential will be sent to this server.` `read_file` returned `native remote result`.
- Stdio approval showed the exact in-grant executable, `[]` args and project A working directory. `read_file` returned `stdio result`. The separate out-of-grant attempt was refused before execution.
- Native Cancel displayed `Connection approval was cancelled; nothing started.` Normal Cmd+Q left no app (PID 88100), sidecar (PID 88102) or stdio child (PID 41016), checked directly by `ps`.
- A real restart showed an empty credential field and no MCP session, with saved chat history intact.

### Package evidence

Prepare/build commands:

```sh
./scripts/studio/prepare-tauri-runtime.sh
PATH=/Users/casey.allard/.cargo/bin:$PATH \
CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/kimi/harness-app-target \
  cargo tauri build --bundles app -- --features gui
```

Bundle: `/Users/casey.allard/uor-r4/local/kimi/harness-app-target/release/bundle/macos/UOR Harness.app` (unsigned development package).

- `UOR_TAURI_APP=... UOR_DESKTOP_SHA256=<fresh executable hash> python3 scripts/tests/test_tauri_app.py`: **2/2 PASS**.
- Desktop executable SHA-256: `55597310da1b886eea364abbb7e36608fd441650a0bc364b6b1d7e489bb0bd4a`.
- Sidecar SHA-256: `f4f36c72baea6b0fb76c72ff511cf8e9615fd328022d3b08937660b28cc05f7e`.
- Stack SHA-256: `d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`.
- Tokenizer SHA-256: `6758318dbcad5925a550103a5c2ee41e35898c30a7daf066da09b0159a41c57b`.
- Catalog SHA-256: `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`.

## Checks run at the head

- Studio TypeScript typecheck and **306/306** Vitest tests.
- JavaScript/Node **65/65** (the earlier incorrect working-directory glob reported zero tests and was discarded, not counted).
- Default desktop: **3** library, **12** controls, **5** MCP, **7** review and **8** sidecar tests passed.
- GUI-feature desktop: **5** library, **16** controls, **6** MCP, **7** review and **9** sidecar tests passed.
- Python runtime preparation **6/6** and packaged app **2/2** passed.
- `cargo fmt --check`, `git diff --check`, runtime preparation, resource identity and packaged app verification passed. No GPU or paid compute was used.
- The first local PR check failed only because an earlier REPORT lacked the required headings; those are now corrected. Current `loop pr-check --local` is PASS with only pre-existing advisory warnings, and `loop gates --full` is **12/12 GREEN**.

## Skills used

- `uor-research-architect`, `working-discipline`, `scope-guard`, `context-guard`, `solution-integrity`, systematic debugging, brainstorming and test-driven red/green regression work.
- `superpowers:verification-before-completion` and `superpowers:finishing-a-development-branch` for measured delivery and protected integration.
- `computer-use` and `kimi-cu` for the Claude desktop reference, real native GUI observations, consent sheets and process verification.
- `modern-web-guidance`: checked preserve-editor-tabs and section-history topics; no guide covered the root-identity or initial-history state rules, so none was applied. Existing outbound-connection and CSP-compatible fallbacks remain in force.
- `session-close` and `compaction` for durable handoff state.

## Still open

Run fresh-context exact-head adversarial review, deliver through the protected PR/queue, verify fresh `main`, publish only the runtime allowlist to Pages, and clean regenerable local files without deleting evidence or other labs' work. #2826 remains open; use `References #2826`, not `Closes`.

## Plain summary

- The shared web/Tauri harness works at development scope, including native `uor-chat` streaming, same-named local files, guarded terminal/Git, visible session tools, and both native MCP transports.
- Review and real GUI checks found and fixed origin, stream, threading, consent, durable-store, root-identity and initial-navigation defects; focused regressions now cover their fixes.
- The packaged app and actual native dialogs were exercised, but the engine remains synthetic and forced-kill cleanup is not guaranteed.
- Protected delivery, fresh-main and hosted verification, and the 30 GiB cleanup target are still pending.
