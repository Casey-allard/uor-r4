# Unified Harness IDE Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver one simple Apple-like web and Tauri harness IDE with real chat, files, Git, guarded terminal, model and MCP workflows, plus a verified macOS app bundle.

**Architecture:** One staged frontend and one backend state. Tauri uses its asset origin and IPC for privileged work; a host proxy reaches only the verified `uor-chat` sidecar. Browser-native APIs remain fallbacks, never fake native controls.

**Tech Stack:** Vanilla HTML/CSS/JS, Vitest/Node tests, Rust/Tauri v2, `tauri-plugin-dialog`, `reqwest`/rustls, real loopback and compiled MCP fixtures.

**Spec:** `docs/loop/tasks/infra-desktop-client-tauri/kimi-1/DESIGN.md` at `d5bda90f12400730fd6ad1b50e74e308624ff596`

## Global Constraints

- Preserve the approved Comparator B design: one asset origin, one backend state, one sidecar, no second RPC server or old byte-mod bridge.
- No useful-conversation, visual-understanding, production-readiness or M3/M7/M8 claim. Conversation history is not exact sidecar hidden-state resume.
- No unrestricted file/model paths, page-enabled terminal, arbitrary MCP process, token in argv/logs/storage, unbounded process output, or autonomous tool calls.
- Custom OpenAI providers and remote model URLs are browser-only; Tauri shows an honest unavailable state.
- New dependencies are limited to optional `tauri-plugin-dialog` and `reqwest`/rustls, with focused tests and no headless GUI dependency leak.
- All runtime packaging and Pages publication use the same allowlist. No binaries, corpora, secrets or development files are committed.
- Protected PR, independent exact-head review, fresh-main verification, actual `.app` launch and at least 30 GiB free after cleanup are required.

## Review Focus

- Origin/token/lifecycle: sidecar is reachable only through the host, readiness is authenticated, stop cancels, normal exit kills, and no token reaches logs/storage/argv. Owned by Tasks 3 and 8.
- Grant/consent authority: project/model folders and terminal enablement require native confirmation on one backend state; outside paths and interpreters fail. Owned by Task 4.
- Unbounded/hostile input: long editor writes, large Git/terminal output, split/coalesced MCP frames, huge MCP results and inert markup remain bounded and inert. Owned by Tasks 4–6.
- Browser regressions: direct fetch, WASM, history, long paste, image attachments, model catalog and all submenus keep working or fail honestly. Owned by Tasks 1, 3 and 5.
- Package truth: clean checkout contains no runtime/source secrets or development files; sidecar/resources hash-match and the actual `.app` launches. Owned by Task 7.

---

### Task 1: Canonical interface for every view and submenu

**Files:**
- Modify: `web/studio/index.html`
- Modify: `web/studio/index.css`
- Modify: `web/studio/assets/js/studio_ui.js`
- Test: `web/studio/src/adapters/conversationFirstTheme.test.ts`
- Test: `web/studio/src/adapters/studioNavigation.test.ts`
- Test: `web/studio/src/adapters/studioNativeTheme.test.ts`
- Test: `web/studio/src/adapters/studioOverlayState.test.ts`

**Interfaces:**
- Consumes: existing `setView`, `activateSection`, model/workspace/guide DOM and all current theme tokens.
- Produces: `Chat`, `Files`, `Changes`, `Terminal`, `Connections`, `Settings` navigation, canonical `harness-*` component classes, and no legacy panel-only presentation.

- [ ] **Step 1: Write failing view-map and submenu regressions**
  - Assert `setView` accepts `changes` and `terminal`, each required view has one H1, and every nav destination has a visible control.
  - Assert canonical tokens/components style nav, editor tabs, source cards, session menus, build menus, dialogs, toasts and mobile drawers with no old `agent-console` label or panel-specific skin.
  - Assert mobile controls remain at least 24px and form text at least 16px, with visible focus and forced-colors fallbacks.
- [ ] **Step 2: Run focused tests to verify intended failures**
  - Run: `npm test --prefix web/studio -- src/adapters/conversationFirstTheme.test.ts src/adapters/studioNavigation.test.ts src/adapters/studioNativeTheme.test.ts src/adapters/studioOverlayState.test.ts`
  - Expected: FAIL for missing Changes/Terminal view maps and canonical submenu rules.
- [ ] **Step 3: Implement the canonical shell and consolidate legacy CSS**
  - Add Changes and Terminal view shells without changing existing engine APIs. Keep Chat, Files and Connections workflows; make Settings and Guide use the same components.
  - Consolidate the current token-based styling into one vocabulary for every view/menu/modal/drawer. Preserve IDs, event handlers, readiness, attachments and model cards; update outdated tests deliberately rather than deleting assertions.
- [ ] **Step 4: Run focused tests and the complete Studio suite**
  - Run the focused command, then `npm test --prefix web/studio`.
  - Expected: focused PASS and all Studio tests PASS.
- [ ] **Step 5: Inspect every view before committing**
  - Serve the staged page locally and inspect Chat, Files, Connections, Changes, Terminal, Settings/Guide and their menus at 1440×900 and 390×852. Record missing/overflow states explicitly; do not commit a known old-looking view.
- [ ] **Step 6: Commit**
  - `git add web/studio/index.html web/studio/index.css web/studio/assets/js/studio_ui.js web/studio/src/adapters/*`
  - `git commit -m '[kimi] Unify every harness view with the Apple-like shell'`

### Task 2: Shared runtime allowlist and native-hook contract

**Files:**
- Create: `scripts/studio/runtime-files.sh`
- Create: `scripts/studio/prepare-tauri-runtime.sh`
- Create: `scripts/tests/test_tauri_runtime.py`
- Modify: `src-tauri/.gitignore`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `web/studio/verify_desktop_client.mjs`

**Interfaces:**
- Consumes: published `uor-chat` hash `f4f36c72baea6b0fb76c72ff511cf8e9615fd328022d3b08937660b28cc05f7e`, synthetic stack `d60f88d6…`, tokenizer `6758318d…`, and current five-card `models.json` `96e296b7…12554a`.
- Produces: `src-tauri/runtime/studio`, `src-tauri/binaries/uor-chat-aarch64-apple-darwin`, `externalBin: ["binaries/uor-chat"]`, and a deliberate native-hook/CSP contract.

- [ ] **Step 1: Write failing runtime hygiene tests**
  - Test the allowlist rejects `src`, tests, `package*.json`, `tsconfig.json`, `vite.config.ts`, declarations, verifier files, README and `node_modules`.
  - Test sidecar/externalBin naming, required resources and CSP/native-hook declarations. Test no published URL is contacted during staging.
- [ ] **Step 2: Run the tests to verify missing implementation**
  - Run: `python3 scripts/tests/test_tauri_runtime.py`
  - Expected: FAIL because the runtime allowlist/preparer and packaging declarations are absent.
- [ ] **Step 3: Implement deterministic runtime preparation**
  - `runtime-files.sh` is the single copy allowlist used by Tauri and later Pages publication.
  - `prepare-tauri-runtime.sh` copies only those files, fetches the published sidecar and synthetic artifacts over HTTPS, verifies their exact hashes, and refuses stale/mismatched bytes. Generated runtime/binaries stay ignored.
  - Configure runtime-only `frontendDist`, externalBin, resources, narrow navigation/CSP, and a real vector-icon generation path. Keep unsigned local packaging explicitly separate from signed distribution.
- [ ] **Step 4: Update the native verifier deliberately**
  - Replace the obsolete “no native hooks” assertion with checks for the declared host adapter, navigation guard and narrow CSP. Do not weaken direct-browser network traps.
- [ ] **Step 5: Run hygiene checks**
  - Run `python3 scripts/tests/test_tauri_runtime.py`, `bash -n scripts/studio/runtime-files.sh scripts/studio/prepare-tauri-runtime.sh`, and `node web/studio/verify_desktop_client.mjs`.
  - Expected: all PASS; generated directories contain only allowlisted runtime files and verified sidecar/assets.
- [ ] **Step 6: Commit**
  - `git add scripts/studio scripts/tests/test_tauri_runtime.py src-tauri/.gitignore src-tauri/tauri.conf.json web/studio/verify_desktop_client.mjs`
  - `git commit -m '[kimi] Add verified Tauri runtime staging and native contracts'`

### Task 3: Sidecar lifecycle and full host OpenAI proxy

**Files:**
- Create: `src-tauri/src/sidecar.rs`
- Create: `web/studio/assets/js/desktop_openai.js`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `web/studio/assets/js/studio_api.js`
- Test: `src-tauri/tests/sidecar_proxy.rs`
- Test: `web/studio/assets/js/desktop_openai.test.mjs`

**Interfaces:**
- Consumes: runtime sidecar/stack/tokenizer from Task 2 and existing `fetchWithConnectionGuidance`.
- Produces: `SidecarProcess::{start, models, decompose, stream_chat, cancel}`, `NativeChatEvent`, JS `registerDesktopOpenAI` with a `ReadableStream` Response shim and request-ID cancellation.

- [ ] **Step 1: Write failing lifecycle/proxy tests**
  - Rust: port `127.0.0.1:0`, authenticated `/v1/models`, `/v1/images/decompose`, SSE, non-streaming JSON, 413/501, cancel and normal-exit kill. Assert no token in argv/log output.
  - JS: status/header/body-chunk events form a real Response reader; usage/`uor` metadata and error bodies survive; AbortController calls `host_cancel_chat(requestId)`; queue overflow aborts; direct browser fetch is unchanged.
- [ ] **Step 2: Run the tests to verify missing proxy behavior**
  - Run: `CARGO_TARGET_DIR=local/kimi/harness-target cargo test --manifest-path src-tauri/Cargo.toml --test sidecar_proxy` and `node --test web/studio/assets/js/desktop_openai.test.mjs`
  - Expected: FAIL because sidecar/proxy and adapter do not exist.
- [ ] **Step 3: Implement sidecar and proxy**
  - Start with `--serve 127.0.0.1:0`, parse the actual port while draining output, wait for authenticated `/v1/models`, and kill/wait on normal drop. Never log token values.
  - Implement bounded `/v1/models`, `/v1/images/decompose` and `/v1/chat/completions` JSON/SSE through host events. Preserve exact headers and bodies. No mutex across external I/O.
  - Implement cancellation, bounded queues and terminal error events. The page never directly contacts the sidecar.
- [ ] **Step 4: Implement the JS transport seam and native bootstrap**
  - Expose only the declared adapter; convert host status/header/body events into a `ReadableStream` Response consumed unchanged by `studio_api.js`.
  - Register an in-memory native transport descriptor, map AbortController to the host cancel command, and cap queued bytes. Keep direct browser fetch unchanged.
- [ ] **Step 5: Run focused and existing native checks**
  - Run both focused commands plus `CARGO_TARGET_DIR=local/kimi/harness-target cargo test --manifest-path src-tauri/Cargo.toml` and `node --test web/studio/assets/js/*.test.mjs`.
  - Expected: all PASS.
- [ ] **Step 6: Commit**
  - `git add src-tauri/src/sidecar.rs src-tauri/src/lib.rs src-tauri/src/main.rs src-tauri/tests/sidecar_proxy.rs web/studio/assets/js/desktop_openai.js web/studio/assets/js/desktop_openai.test.mjs web/studio/assets/js/studio_api.js`
  - `git commit -m '[kimi] Proxy native OpenAI streaming through one Tauri host'`

### Task 4: Granted folders, bounded Git and guarded terminal

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `web/studio/assets/js/studio_files.js`
- Create: `web/studio/assets/js/desktop_actions.js`
- Test: `src-tauri/tests/desktop_client_controls.rs`
- Test: `web/studio/src/adapters/desktopActions.test.ts`

**Interfaces:**
- Consumes: `TauriDesktopState` from Task 3 and the native dialog dependency.
- Produces: no-argument project/model-folder pickers, no-argument terminal enablement, bounded `native_git_diff`, and host-approved terminal execution with allowlist/timeout/output cap.

- [ ] **Step 1: Write failing authority and bounds regressions**
  - Outside project/model directories are refused before scan/read; sibling ungranted paths fail; grants made by one native picker are visible to subsequent commands on the same state.
  - Terminal enablement requires no page boolean; `bash`, `sh`, `python3`, `osascript`, `env` and relative scripts refuse. Approved `pwd` succeeds once. Long Git/terminal output truncates explicitly and external I/O does not hold the state mutex.
  - Git uses `status -z`, no pager/external diff/textconv and the declared environment. A long operation cannot block file reads.
- [ ] **Step 2: Run focused tests to verify current gaps**
  - Run: `CARGO_TARGET_DIR=local/kimi/harness-target cargo test --manifest-path src-tauri/Cargo.toml --test desktop_client_controls` and `npm test --prefix web/studio -- src/adapters/desktopActions.test.ts`
  - Expected: FAIL for missing picker/model grant, allowlist, diff and output bounds.
- [ ] **Step 3: Implement trusted dialogs and path/model authority**
  - Add optional `tauri-plugin-dialog` and main-window permissions. No-argument host commands choose project/model folders and enable terminal; no page path or enable boolean creates authority.
  - Keep all grant/approval state in one backend and release the lock before process/network I/O.
- [ ] **Step 4: Implement allowlist, bounded Git and terminal execution**
  - Enforce exact program allowlist and relative-script refusal. Preserve direct spawn, metacharacter checks and granted paths.
  - Add bounded Git status/diff with hardened environment and `status -z`, plus timeout/output caps and explicit truncation/child kill.
- [ ] **Step 5: Wire shared Files/Changes/Terminal actions**
  - `desktop_actions.js` exposes only declared host calls. `studio_files.js` uses native project/model folder pickers before the browser File System Access fallback and maps native full paths into the existing editor.
  - Changes and Terminal render real status/diff/consent/output. Custom providers/remote model URLs show “available in the browser build.”
- [ ] **Step 6: Run focused and full desktop/Studio tests**
  - Run both focused commands, the full desktop crate suite and `npm test --prefix web/studio`.
  - Expected: all PASS.
- [ ] **Step 7: Commit**
  - `git add src-tauri web/studio/assets/js/studio_files.js web/studio/assets/js/desktop_actions.js web/studio/src/adapters/desktopActions.test.ts`
  - `git commit -m '[kimi] Add trusted folder, Git and terminal workflows'`

### Task 5: Remote MCP in the browser and shared Connections UI

**Files:**
- Create: `web/studio/assets/js/mcp_client.js`
- Create: `web/studio/assets/js/mcp_client.test.mjs`
- Modify: `web/studio/assets/js/studio_ui.js`
- Modify: `web/studio/index.html`
- Test: `web/studio/src/adapters/mcpExperience.test.ts`

**Interfaces:**
- Consumes: Connections from Task 1 and optional desktop host transport from Task 6.
- Produces: `connectRemote(config)`, `listTools()`, `callTool(name,args)`, JSON/SSE normalization, memory-only credentials, and explicit browser-stdio-unavailable errors.

- [ ] **Step 1: Write failing protocol and inert-markup tests**
  - A real Node loopback fixture covers initialize/initialized, tools list/call, JSON and SSE, `Mcp-Session-Id`, `MCP-Protocol-Version`, CORS/preflight, auth failure, timeout, size cap and pagination cap.
  - Tool text containing `<img onerror=...>` is rendered with `textContent`, never HTML. Credentials vanish on reload. HTTPS Pages to insecure loopback receives the native-setup error, not a generic fetch failure.
- [ ] **Step 2: Run protocol/UI tests to verify missing client**
  - Run: `node --test web/studio/assets/js/mcp_client.test.mjs` and `npm test --prefix web/studio -- src/adapters/mcpExperience.test.ts`
  - Expected: FAIL because remote MCP and its UI do not exist.
- [ ] **Step 3: Implement bounded remote MCP and Connections UI**
  - Implement initialize/initialized, tools list/call, JSON/SSE, session/protocol headers, timeout/size limits, pagination cap and actionable transport/auth/CORS errors.
  - Show configured state, tools and results as text only. Connect/call requires an explicit action; browser stdio is honestly unavailable.
- [ ] **Step 4: Run protocol and full browser checks**
  - Run both focused commands, all Node tests, `npm test --prefix web/studio`, and `node web/studio/verify_studio_page_load.mjs --strict-router`.
  - Expected: all PASS; no external request occurs before the explicit Connect action.
- [ ] **Step 5: Commit**
  - `git add web/studio/assets/js/mcp_client.js web/studio/assets/js/mcp_client.test.mjs web/studio/assets/js/studio_ui.js web/studio/index.html web/studio/src/adapters/mcpExperience.test.ts`
  - `git commit -m '[kimi] Add bounded remote MCP to harness connections'`

### Task 6: Host-mediated remote and native stdio MCP

**Files:**
- Create: `src-tauri/src/mcp.rs`
- Create: `src-tauri/tests/mcp_protocol.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `web/studio/assets/js/mcp_client.js`

**Interfaces:**
- Consumes: `connectRemote/listTools/callTool` from Task 5 and `TauriDesktopState` from Task 3.
- Produces: host remote JSON/SSE transport and host-approved stdio transport with identical UI results and bounded process lifecycle.

- [ ] **Step 1: Write failing native transport fixtures**
  - Real loopback Rust HTTP fixture covers CORS/auth/JSON/SSE/session/protocol headers, timeout and size cap.
  - Compiled stdio fixture covers exact-path approval, outside-cwd refusal, newline JSON, partial/coalesced frames, stderr, exit, timeout and kill/orphan cleanup. Process count and credentials are bounded and memory-only.
- [ ] **Step 2: Run native fixtures to verify missing transports**
  - Run: `CARGO_TARGET_DIR=local/kimi/harness-target cargo test --manifest-path src-tauri/Cargo.toml --test mcp_protocol`
  - Expected: FAIL because native transports do not exist.
- [ ] **Step 3: Implement `reqwest` remote MCP and direct-spawn stdio MCP**
  - Use rustls, bounded headers/body/timeouts/redirects and JSON/SSE channel results. No `curl`, wildcard origin or broad CSP.
  - Require host-approved exact executable/args/cwd, direct spawn, bounded frames/timeouts/processes, stderr drain and cleanup. Implement the same initialize/list/call surface and pagination cap.
- [ ] **Step 4: Route Tauri MCP through the host without changing browser behavior**
  - `mcp_client.js` selects host transport only when the declared desktop adapter is present; direct browser remote behavior remains unchanged. Browser stdio stays unavailable.
- [ ] **Step 5: Run native and cross-platform MCP tests**
  - Run the Rust fixture, Node MCP tests, full desktop crate suite and Studio suite.
  - Expected: all PASS.
- [ ] **Step 6: Commit**
  - `git add src-tauri/src/mcp.rs src-tauri/tests/mcp_protocol.rs src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock web/studio/assets/js/mcp_client.js`
  - `git commit -m '[kimi] Add host-mediated remote and stdio MCP'`

### Task 7: Packaged app and end-to-end GUI verification

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `scripts/studio/verify-tauri-app.sh`
- Create: `scripts/tests/test_tauri_app.py`
- Create: `docs/loop/tasks/infra-desktop-client-tauri/kimi-1/REPORT.md`

**Interfaces:**
- Consumes: Tasks 1–6.
- Produces: one launchable macOS arm64 `.app` with staged runtime, externalBin, resources, real icon and explicit unsigned-development status.

- [ ] **Step 1: Add failing package and GUI acceptance checks**
  - Verify the packaged bundle contains sidecar, stack, tokenizer, catalog and real icon, excludes development files, and has matching resource hashes.
  - Verify GUI chat streams, model list/test, image decomposition, project/model-folder grants, file save, bounded Git/terminal, local history and both MCP transports. Stop/exit closes the sidecar.
- [ ] **Step 2: Build and run the real app to expose gaps**
  - Run the runtime preparer, focused Rust/Studio suites, and `cargo tauri build --bundles app` with a dedicated temporary target. Install `tauri-cli` only if the documented command is unavailable.
  - Expected: current build fails or acceptance checks are incomplete; record actual errors rather than claiming package readiness.
- [ ] **Step 3: Resolve packaging and GUI integration defects test-first**
  - Fix only demonstrated issues in lifecycle, resource paths, dialog permissions, CSP/WASM worker, UI actions or packaging. Add a failing regression before each fix.
  - Use an unsigned local `.app` only for development verification; do not claim signed distribution.
- [ ] **Step 4: Repeat complete native and browser verification**
  - Launch the packaged `.app` and exercise every acceptance row with real GUI interaction and fixtures. Verify no token appears in process args, logs, storage or exported history.
  - Run all Studio/Node/Rust/desktop/MCP suites, worker/page traps, WASM audit, runtime hygiene and independent desktop/mobile inspection.
- [ ] **Step 5: Record evidence and commit**
  - Write the required REPORT with negatives first, actual package hash, GUI evidence, exact unsupported limits and remaining follow-up.
  - `git add src-tauri scripts/studio scripts/tests/test_tauri_app.py docs/loop/tasks/infra-desktop-client-tauri/kimi-1/REPORT.md`
  - `git commit -m '[kimi] Verify the unified Tauri harness app'`

### Task 8: Independent review, protected delivery, Pages and cleanup

**Files:**
- Modify: `docs/products/desktop-client.md`
- Modify: `docs/products/web-client.md`
- Modify: `docs/integration/current-state.md`
- Modify: `docs/research/notebook/m4-serving.md`
- Modify: `docs/roadmap/items/infra-desktop-client-tauri.md`
- Modify: `docs/loop/tasks/infra-desktop-client-tauri/kimi-1/REPORT.md`

**Interfaces:**
- Consumes: the verified Task 7 head and artifacts.
- Produces: protected merged records and a clean runtime-only Pages publication, or an explicit blocked reason.

- [ ] **Step 1: Run final review and all repository gates**
  - Obtain independent exact-head review for the whole branch; fix Critical/Important findings RED→GREEN and retain final transcripts.
  - Run `UOR_LAB=kimi uor-harness loop pr-check --local` and `UOR_LAB=kimi uor-harness loop gates --full`.
  - Expected: all required checks and review pass.
- [ ] **Step 2: Deliver through the protected merge queue**
  - Open PR with `References #2826`, watch required checks, merge through the queue, and compare merge tree to the reviewed tree.
  - Expected: merged source commit contains the reviewed implementation without a conflict-resolution rewrite.
- [ ] **Step 3: Verify fresh main and publish runtime**
  - Run fresh detached-main checks. Publish only the Task 2 allowlist, preserve `models.json` byte-identically, and verify hosted desktop/mobile views and development-file 404s.
  - Expected: hosted source/runtime matches the reviewed implementation and Pages reports built.
- [ ] **Step 4: Record current state and clean resources**
  - Update only evidence-backed product/current-state/notebook/roadmap/REPORT claims, with desktop generation distinguished from mobile layout and unsigned local packaging distinguished from distribution.
  - Remove merged branches/worktrees and regenerable outputs while preserving unique evidence and shared runtimes. Verify at least 30 GiB free.
  - Expected: fresh-main record passes, hosted page is live, and disk floor holds.
