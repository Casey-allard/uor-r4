# UOR-R4 Studio baseline for a Cursor-hosted-experience upgrade

**Baseline identity.** Worktree `/Users/casey.allard/uor-r4/.worktrees/kimi-cursor-hosted-research`, branch `kimi/cursor-hosted-research`, HEAD `452615bc52a0186c1c01007986ff5c1a767eb068`. The stated base `6fcf57f7822286e5b6bd7bcfe42d668116b23afa` is an ancestor; the only commit above it adds `docs/loop/tasks/studio-ide-harness/kimi-18/ORDER.md`. The worktree was clean at inspection. No build, test, server or model run was performed for this baseline; every test count below is quoted from a retained log or record and is labelled accordingly.

**Project state.** M1 closed on 30 October 2026; 1 of 8 milestones is complete and no model holds a useful conversation (`AGENTS.md:3`, `CHARTER.md:9`). The web client is a pre-alpha research explorer, not a product (`docs/products/web-client.md:7`).

**Evidence classes used below.**
- **Observed code** — the named file and line exist at this base and contain what is described. This proves existence and shape, not runtime behavior.
- **Retained executed evidence** — a committed record or an ignored local log/JSON/screenshot produced by an earlier run, with an identity where available. Not rerun here.
- **Inference** — a conclusion drawn from code plus retained evidence.
- **Unknown / unverified** — no executed evidence at this base; do not promote without a new run.

**Target framing.** `docs/products/harness-parity.md` is a Claude Cowork/Code matrix, not a Cursor matrix. `docs/loop/tasks/studio-ide-harness/kimi-18/ORDER.md:3` states that the older Cowork target is superseded for this planning unit by the owner's Cursor direction and that the unit changes no capability status. No Cursor-specific screen, contract, fixture or acceptance exists in the repository at this base.

---

## 1. What current navigation, project/session, composer, editor/diff/terminal/models/guide/panes, responsive/theme, attachments and persistence behaviors exist?

### Takeaway
The deployed Studio is a single-window, single-active-session workbench over the legacy plain-JavaScript Studio. It has real navigation, a real editor textarea with tabs and disk save, bounded Git status/diff, a real PTY in the native host, model source management including OpenAI-compatible SSE chat, a static guide, two-pane splitting, both themes and responsive breakpoints. It has **no project entity, no task/plan/tool timeline, no concurrent sessions, no approval objects, no account, no scheduler and no agent loop**. The typed shell is the built default, but the source entry still defaults to legacy unless the Vite build plugin rewrites it.

### Cited findings

**Navigation and panes (observed code)**
- The typed section union is `'agent' | 'workspace' | 'changes' | 'terminal' | 'models' | 'guide' | 'browser'` (`web/studio/src/components/types.ts:1`); the sidebar renders six destinations and omits `browser` (`web/studio/src/components/ProjectSidebar.tsx:7-11`), while the pane dock adds a Browser target (`web/studio/src/components/PaneDock.tsx:17-23`).
- `StudioApp` owns local UI state for active section, secondary section, palette, settings, inspector and error (`web/studio/src/components/StudioApp.tsx:38-49`), and dispatches sections to `AgentComposer`, `WorkspaceEditor`, `ChangesPanel`, `TerminalPanel`, `ModelPicker`+`McpConnections`, a browser placeholder, or `Guide` (`StudioApp.tsx:96-105`).
- The browser pane is an explicit unsupported placeholder: "Browser preview requires a harness service" (`StudioApp.tsx:103`).
- The pane dock supports exactly one secondary pane, a keyboard-operable splitter, and a 50% default split held in component state (`PaneDock.tsx:32-44`; `web/studio/src/components/ui/Splitter.tsx`). Split state is not persisted and is reset when the secondary section equals the primary (`StudioApp.tsx:62`).
- The command palette filters and runs a flat command list with arrow/Enter handling (`web/studio/src/components/CommandPalette.tsx:10-55`). In the typed shell it is opened only by the toolbar button (`SessionToolbar.tsx:18`). There is no typed global shortcut handler: `AppShell.tsx:38-42` only closes the mobile drawer on Escape, the "⌘K" label is decorative (`SessionToolbar.tsx:18`), and the palette's `shortcut: 'Ctrl+S'` is a label, not a binding (`StudioApp.tsx:89`). **Coexistence caveat (observed code, behavior not executed):** the legacy `studio_ui.js` still binds a document-level keydown and opens its own `#commandPalette` on ⌘K (`web/studio/assets/js/studio_ui.js:1122-1134`); the typed shell hides the whole `#app` subtree (`index.html:12-14`), and the legacy palette markup sits inside it (`index.html:538`), so ⌘K in a typed build may open an invisible legacy overlay. No Playwright test covers ⌘K in the typed shell; this needs a real browser check before any shortcut is advertised.

**Project and session (observed code)**
- "Project" is local component state: default name `uor-r4` (`AppShell.tsx:33`), a name field and a read-only folder field in a "New project" modal (`AppShell.tsx:57-63`). Submitting only sets the name and navigates to Workspace (`AppShell.tsx:57`). The modal itself says project memory and scheduled tasks "are added by the harness service" (`AppShell.tsx:61`), which is truthful but confirms they do not exist.
- The toolbar's Environment and Permission controls are `<select>` elements whose only alternative options are `disabled` ("Harness service (not connected)", "Autonomous (requires service)") (`SessionToolbar.tsx:11-13`). They change no authority; the Playwright test only asserts their visibility (`web/studio/tests/e2e/shell.spec.ts:29-33`).
- Session state is a single active id plus a list (`web/studio/src/state/lifecycle.ts:71-79`), synchronized from the legacy `SessionApi` (`web/studio/src/adapters/workflow.ts:53-68`). `createAgentSessionController` verifies create/resume/rename/duplicate/delete against the legacy API and refreshes the store (`web/studio/src/adapters/agentSession.ts:23-199`). Messages are read synchronously from `sessions.getConversationMessages()` (`AgentComposer.tsx:155`), so there is no per-task transcript object in the typed state.
- The session list offers new, resume, rename, duplicate, export JSON and delete (`web/studio/src/components/SessionList.tsx:25-40`). Export is a downloaded JSON file (`agentSession.ts:150-159`), not a resumable event log.

**Composer and transcript (observed code)**
- The composer has a prompt textarea, Send/Predict button, Stop, Enter/Shift+Enter, `/` task templates and `@` mentions (`AgentComposer.tsx:296-314`). Six build-task templates are hard-coded (`AgentComposer.tsx:46-53`); `/` filters those and `@` offers the active editor file plus retained attachments (`AgentComposer.tsx:55-86`).
- The context drawer attaches the current editor file, arbitrary text files through a file picker, or pasted text, and lists/removes attachments (`AgentComposer.tsx:302-307`). Text attachments are limited to 1 MiB per file and 4 MiB total; unsupported/binary files and NUL-containing content are refused with an error (`web/studio/assets/js/composer_input.js:1-2`, `:36`, `:39`, `:60`).
- The typed composer is **text-only**: its file handler filters out `kind === 'image'` and calls `readTextFiles` (`AgentComposer.tsx:242-250`). The legacy page still has the bounded raster image path (`assets/js/image_input.js`; native `uor_r4_image` decomposition at `assets/js/studio_api.js:133-140`, 256×256 limit), but the typed shell does not expose it. A Cursor-like screenshot/drag-image workflow therefore needs new typed attachment handling, not just a restyle.
- The transcript renders role, content, an optional tool tag (`m6-exact-arithmetic` only) and four actions: copy, insert into editor, save as file, retry (`AgentTranscript.tsx:8-21`; `workflow.ts:47-51`). There is no plan step, tool call/result card, usage row, approval card, child-task row or stop boundary.
- Composer send/stop delegates to a generation adapter (`web/studio/src/adapters/generation.ts:13-82`). It refuses unless `store.canRun()`, checks the legacy mode is `wasm` or `api`, calls `sendPrompt`/`predict`, and reconciles a generation id. Completion is a DOM event (`uor-generation-finished`) rather than an event stream (`generation.ts:4`, `:15-24`).

**Editor, files, changes (observed code)**
- The typed Workspace is an open-folder header, open tabs, a single `<textarea aria-label="File contents">` with a line-number gutter, Tab insertion, dirty state, Save and Save as (`WorkspaceEditor.tsx:89-113`). There is no searchable file tree, syntax highlighting, multi-cursor, search/replace, LSP, breadcrumbs or per-file diff.
- The legacy page still contains the tree markup (`web/studio/index.html:264-267`) and a second plain `<textarea id="editorText">` (`index.html:287`), but the typed shell never renders `state.tree`; `WorkspaceApi.treeSummary()` exposes only a flat summary (`workflow.ts:28-45`).
- Folder access is host-aware: `studio_files.js` uses `UORDesktopActions.chooseProjectFolder()` + `listDirectory()` in native mode (`web/studio/assets/js/studio_files.js:155-181`), otherwise `showDirectoryPicker` (`:183-203`), otherwise a read-only `importFiles()` fallback (`:205-...`). Native reads/writes go through the granted root (`:412-413`); browser writes use `queryPermission`/`createWritable` (`:654-661`).
- Changes is a repository-path input, a Refresh button, a branch/change-count summary, and the entire raw `git diff` in a `<pre>` with truncation/timeout flags and Copy diff (`web/studio/src/components/DesktopTools.tsx:21-48`). There is no per-file expansion, staging, line comment, or comment persistence.

**Terminal (observed code and retained evidence)**
- The terminal pane mounts xterm, polls `readTerminal(sessionId, cursor)` every 50 ms, writes hex-decoded output, handles resize, close and a 1 MiB retained-output notice (`web/studio/src/components/TerminalSession.tsx:159-194`, `:144-147`). Opening requires host enablement then a native shell approval (`TerminalSession.tsx:196-232`). In a static browser it displays "Interactive PTY is unavailable in this browser build." and the Open button is disabled (`TerminalSession.tsx:236`; `web/studio/tests/e2e/terminal.spec.ts:4-20`).
- The host side is a real PTY with bounded sessions/input/output; it is not a general sandbox (`docs/products/desktop-client.md:55-57`, `:90`). Retained evidence records an isolated loopback run of `printf`, `read` + answer, `pwd`, `stty size`, `exit`, and post-close refusal (`docs/products/harness-parity.md:28`).

**Models and MCP (observed code)**
- Models lists the published catalog, bundled synthetic demo, local file, URL and API providers; it can add/edit/remove/test providers and load an artifact with download progress (`web/studio/src/components/ModelPicker.tsx:29-120` and following; catalog loader `web/studio/assets/js/model_sources.js:423-484`).
- API providers are stored in `localStorage` under `uor.studio.modelSources.v1` (`model_sources.js:45`, `:167`, `:204`); keys stay in the browser except as an `Authorization` header on a user-triggered request (`web/studio/assets/js/studio_api.js:9-12`). Native bootstrap credentials are ephemeral.
- API chat is an OpenAI-compatible `/chat/completions` SSE client (`studio_api.js:357` and following; `:473` decode loop; `:511-539` non-streaming diagnostics). It parses `choices[].delta.content`, `finish_reason`, usage and optional `uor` token metadata. It has no tool-call delta path.
- MCP in the browser is remote Streamable HTTP only; stdio requires the native host. The typed panel lists tools, takes raw JSON arguments, calls once, shows text results and can attach the result JSON explicitly (`web/studio/src/components/McpConnections.tsx:40-...`; `web/studio/src/adapters/mcpConnections.ts:1-15`). It does not run a model/tool loop.

**Guide, settings, theme, responsive (observed code)**
- Guide is static prose describing the manual workflow and limits (`web/studio/src/components/Guide.tsx:1-17`).
- Settings covers theme, reduce motion, editor font size/tab size/word wrap, and clearing sessions or editor tabs (`web/studio/src/components/Settings.tsx:74-105`). It has no account, profile, privacy, notification, connector-credential or model-policy section.
- Theme tokens are dark `#141413` / light `#faf9f5`, accent `#d97757`, blue `#6a9bcc` (`web/studio/src/studio.css:3-12`). The `data-theme` attribute is set on `<html>` from storage or URL before boot (`index.html:17-31`).
- Responsive breakpoints are 1180 px (hide status/shortcut labels) and 800 px (drawer sidebar, single-pane mobile, hidden splitter, 16 px form text) at `studio.css:192-211`; coarse-pointer target sizing, forced-colors focus and reduced motion follow at `:212-215`.
- The mobile drawer is open by default only above 800 px (`AppShell.tsx:31`); Escape closes it, and the mobile split hides the primary pane (`studio.css:193-204`).

**Persistence and migration (observed code)**
- The browser store is `localStorage` only. The migration map is: `uor.studio.conversations.v1 → uor.studio.lifecycle.conversations.v2`, `activeConversation.v1 → .activeConversation.v2`, `editor.v1 → .editor.v2`, `settings.v1 → .settings.v2`, `ui.v2 → .ui.v3`, `ui.v1 → .ui.v2`, `attachments.v1 → .attachments.v2`, `tabs.v2 → .tabs.v3`, `recent.v2 → .recent.v3`, `theme → .theme.v1` (`web/studio/assets/js/lifecycle_persistence.js:6-30`).
- `uor.studio.modelSources.v1` is deliberately a live compatibility key and is **not** migrated (`lifecycle_persistence.js:34-36`, `:67-69`).
- Migration is copy-forward with destination-conflict detection, verification and rollback; it writes marker `uor.studio.lifecycle.migration.v1` = `{"schema":"uor-r4.studio-lifecycle-migration/1"}` (`lifecycle_persistence.js:84-164`). Routing is active only after a successful/current marker (`:166-195`). The script self-activates only when `?typed=1` (`:208-209`); the typed bootstrap then calls `activateStudioMigration(true)` (`web/studio/src/bootstrap.ts:40`).
- Conversations, their drafts, and the active id are persisted by the legacy inline module under `uor.studio.conversations.v1` / `uor.studio.activeConversation.v1` (`index.html:590`, `:676-711`). Tabs, editor buffers, attachments, recent files, UI and settings live under their own keys (`studio_files.js:7`, `studio_ui.js:8-15`).
- There is no IndexedDB, no browser object store, no server-side transcript, no event cursor and no cross-device store. Session export is a one-shot JSON download; worker model-state export/import is a separate `UR4SESS1` checkpoint (`web/studio/assets/js/uor_model_worker.js:654`, `:672`), not the chat transcript.

**Source default vs built default (observed code)**
- `typed_entry.js` imports the typed app only when `?typed=1` (`web/studio/assets/js/typed_entry.js:1-3`).
- The Vite build plugin rewrites that guard to `legacy !== '1' && typed !== '0'` (`web/studio/vite.config.ts:4-13`), and `bootstrap.ts` treats anything except `legacy=1`/`typed=0` as typed (`bootstrap.ts:14-17`). So the **published/built** page defaults to typed; the raw source page defaults to legacy. Playwright builds first (`playwright.config.ts` webServer `npm run build`), which is why its default-URL test passes (`tests/e2e/shell.spec.ts:4-12`).

### Retained executed evidence
- `local/kimi/default-cutover/fresh-main/typed-tests.log`: **455/455** Vitest tests across 52 files, 7.36 s.
- `local/kimi/default-cutover/fresh-main/playwright.log`: **47 passed** in 2.1 m, including default typed shell, both legacy selectors, neutral palette, landmarks, selectors, pane switching, mobile drawer/Escape, split and session create/resume (`web/studio/tests/e2e/shell.spec.ts`, `workflows.spec.ts`, `visual.spec.ts`, `terminal.spec.ts`).
- `local/kimi/default-cutover/fresh-main/runtime-contracts-rerun.log`: **15 tests OK**; the first run's four failures/one error were missing generated `pkg`/`dist` inputs, not merged-source behavior (`docs/loop/tasks/studio-ide-harness/kimi-17/REPORT.md:8`).
- `local/kimi/default-cutover/fresh-main/live-acceptance.json`: public `/studio/` desktop/mobile neutral workbench visible; public index SHA-256 `889f136…`; 14 requests, zero external/raw-TypeScript/runtime errors; one shell; `legacyMcp=0`, `typedMcp=1`; artifact ready; exact `17 + 25 = 42`; isolated/restored sessions; rollback `6540e271…`.
- `local/kimi/default-cutover/fresh-main/public-runtime.json`: the visible transcript is the exact-arithmetic tool answer, not generated model prose. The only retained text after `You / 17+25` is `42` plus the place-value trace and "tool, not model generation".
- `docs/products/harness-parity.md:17-18` records PASS for the neutral shell and light/dark responsive shell (browser only); `:38` records PASS for the static default-route preview; `:19-37` and `:39-41` record NOT_RUN or PARTIAL for projects, memory, parallel tasks, plan/tool trace, stop/redirect, side questions, permission modes, editor/files, diff review, browser, computer use, connectors, artifacts, schedules, Dispatch, environments, PR/CI, session/task resume, R4 API integration, packaged full matrix and hosted production.
- `docs/loop/tasks/studio-ide-harness/kimi-17/REPORT.md:16` records the same 455/47/15/11/7 counts at the fresh-main verification head; `:41` says the shared agent runtime, parallel tasks, project memory, steering/orchestration, Dispatch, connectors, embedded browser/computer use, live R4 API, signed release and useful-model behavior "remain unbuilt".

### Inferences
- The Studio is best classified as a **manual research workbench with a chat transcript**, not an agent/editor. The current typed layer adds visual structure and lifecycle guards over global legacy singletons; it does not add a domain model for projects, tasks, plans, tools or approvals.
- Because session, file and model state are single-valued (`lifecycle.ts:81-90`), two concurrent tasks cannot be represented without either a per-task store or replacing the reducer. Adding a task list to the current shell without that change would create a false concurrency affordance.
- The browser build's real capability ceiling is exactly what the parity matrix says: shell, themes, responsive layout, local WASM artifact loading/prediction, exact arithmetic, manual attachments, session create/resume, and explicit unsupported states.
- The "Project" modal and the Environment/Permission selects are honest placeholders today, but a Cursor-like design must replace them with real objects and authority or keep them visibly disabled; otherwise the UI would imply capability that does not exist.

### Gaps
- No executed evidence at this base for: concurrent tasks, plan/tool trace, stop preventing later tools, redirect at a safe boundary, side questions, child fan-out/join, approval forgery refusal, diff comments, searchable tree, artifact preview, embedded browser, computer use, scheduling, notifications, cross-device resume, account/auth, or hosted service.
- No Cursor-specific interaction inventory or acceptance exists; the six-facet Cursor research is a separate kimi-18 facet, not this baseline.
- The retained 47/47 Playwright run does not exercise real model chat; its default-page test asserts the shell and an exact-arithmetic answer only.
- `web/studio/README.md:3` still names gh-pages `6540e271…` as the current publish, while `docs/integration/current-state.md:3` and `live-acceptance.json` name `4a435ab8…`. This is record drift in a product README, not a code defect; do not repeat the stale identity in a new plan.

---

## 2. What shared state, adapters and host contracts can be reused instead of creating a second frontend or authority channel?

### Takeaway
Reuse the existing single Vite/Preact frontend, the typed lifecycle/adapters as the presentation-side guard layer, the legacy workflow APIs as the current browser services, the Tauri command + loopback RPC contract as the only host authority, the `uor-r4-object-store` only for existing Tauri session snapshots and optional bounded exports, and the `uor-chat` sidecar's OpenAI-compatible text stream behind a narrow adapter. Browser task/event persistence is IndexedDB, not object-store. Do **not** introduce a second state owner, a second shell, a second model API, or a page-supplied approval boolean.

### Cited findings

**One frontend, one build (observed code and retained evidence)**
- One Vite build supplies the mounted web preview, the native runner and the Tauri app (`docs/products/web-client.md:49`; `web/studio/vite.config.ts:22-29`; `src-tauri/tauri.conf.json:7` `frontendDist: "./runtime/studio"`). `scripts/studio/prepare-tauri-runtime.sh` stages it; `scripts/studio/native-runner.sh` serves it locally.
- The typed shell is mounted into `#studio-typed-root` while the legacy `#app` remains in the same document and is hidden by CSS (`index.html:12-14`, `:37`). This is a shared document, not two frontends, but it is also a coexistence seam that later work must not break (`docs/products/web-client.md:51`).

**Presentation-side state and adapters (observed code)**
- `LifecycleStore` is a small synchronous reducer with listener subscription, operation/generation/save identity counters, and typed transitions for artifact load, API selection/test, folder/file, session, and generation (`web/studio/src/state/lifecycle.ts:120-417`). `canRun()`/`canSave()` encode the current source/artifact/file preconditions (`:149-169`).
- `useLifecycle` is the Preact binding (`web/studio/src/state/useLifecycle.ts:1-9`).
- `WorkspaceApi`, `SessionApi`, `StudioUiApi`, `Attachment` and `StudioSettings` are the typed seam over the legacy globals (`web/studio/src/adapters/workflow.ts:28-116`); `createLegacyWorkflowApis` reads `window.__studioFiles` and `window.__studioUi` (`:124-130`).
- `createOperationGate` rejects stale async success/error by operation version (`web/studio/src/adapters/operations.ts:5-20`). `saveActiveFile` serializes saves and verifies the active path/save id (`web/studio/src/adapters/fileSave.ts:4-48`).
- `createAgentSessionController` verifies every session mutation against the legacy API's resulting state instead of trusting a boolean (`agentSession.ts:59-148`). `createGenerationAdapter` bridges the legacy generation id and the DOM completion event (`generation.ts:13-82`). `createModelSourceAdapter` verifies worker filename/SHA-256/capabilities and ignores stale load responses (`modelSourceAdapter.ts:185-238`).
- These are genuine reuse assets for a Cursor-like shell: they already encode stale-result refusal, save serialization, session identity verification and artifact identity checks. They are not an agent runtime and must not be presented as one.

**Host contract — Tauri commands (observed code)**
- `desktop_actions.js` is the page-side contract and maps camelCase JS arguments to snake_case command arguments (`web/studio/assets/js/desktop_actions.js:29-57`). When `__TAURI__.core.invoke` is absent it returns `available:false` and every action rejects with a clear unavailable error (`:3-28`).
- The Tauri builder registers 36 command handlers (`src-tauri/src/lib.rs:3677-3714`). They cover models, exact arithmetic, image decomposition, sidecar chat + cancel, model load/step/generate, session export/import/reset/durable save/restore, granted folders, file read/write/list, Git status/diff, project/model folder pickers, terminal enable/confirm/run/open/read/input/resize/close, and MCP remote/stdio connect/list/call/disconnect. The doc comment at `lib.rs:3647-3648` says "30" handlers; that count is stale against the 36 registered names.
- The GUI window is created in code with `index.html?typed=1`, 1440×900, navigation confined to the app asset origin; external http(s) links open in the system browser (`src-tauri/src/lib.rs:3652-3674`). `tauri.conf.json:11` has `"windows": []` because the window is built here, not declaratively.
- The page RPC surface is a single `POST /__tauri_invoke` on the loopback host. It requires exact `Host`, exact `Origin`, `Content-Type: application/json`, and a per-launch `X-UOR-Desktop-Token` (`src-tauri/src/lib.rs:2590-2651`). The token is injected into the served `index.html` at request time (`:2691-2703`). Static files are served from a canonical root with traversal/symlink refusal and `.ts`/`.tsx`/`.map` 404s (`:2652-2717`).
- The same backend is reachable as a newline-delimited stdio JSON-RPC loop (`src-tauri/src/main.rs:1-12`; `handle_rpc_line` at `src-tauri/src/lib.rs:2515`). `dispatch_rpc_command` is the shared command switch (`:2094-2514`), so tests and both transports exercise one authority path.

**Host authority boundaries (observed code)**
- Folder grants are host-owned and canonicalized; all file/Git/terminal paths are checked against `granted_folders` with `..` and symlink refusal (`src-tauri/src/lib.rs:1024-1073`; `docs/products/desktop-client.md:89`).
- Terminal enablement and exact-command approval are host-owned; page RPC can only report state and run an already-approved command once. CLI automation uses `--enable-terminal`, `--approve-shell`, `--approve-command` (`src-tauri/src/main.rs:15-71`; `docs/products/desktop-client.md:45`).
- MCP remote/stdio sessions require native confirmation dialogs and are bounded (`src-tauri/src/lib.rs:3559-3645`; `src-tauri/src/mcp.rs:23`, `:555-...`).
- The sidecar is host-owned, started with a token, and proxies OpenAI JSON/SSE without exposing the token to the page (`src-tauri/src/sidecar.rs:23-30`, `:261-383`; `docs/products/desktop-client.md:42`).

**Durable persistence (observed code)**
- `uor-r4-object-store` provides a content-addressed `Store` with `fold`/`range`/manifests (`crates/uor-r4-object-store/src/lib.rs:15-24`; `src/store.rs:170`, `:262-270`) and a scoped durable-session consumer with `IdentityScope` (`user/project/session`), `set_ur4sess1_snapshot`, `export_scope` and restore (`src/durable_consumer.rs:46`, `:283`, `:569`).
- The Tauri backend already uses it for `native_save_session_durable` / `native_restore_session_durable` with user/project/conversation ids (`src-tauri/src/lib.rs:3285-3329`).
- Object-store is scoped to existing Tauri session snapshots and optional bounded exports only; it is not the task authority or the browser run log. Future Cursor task/event persistence belongs in browser IndexedDB with per-run event records and verified snapshot manifests. The serialized-writer constraint remains relevant only to any Tauri export (`local/kimi/studio-agent-order-draft.md:22`).

**Streaming/transport reuse (observed code)**
- The browser already has an SSE parser and cancellable fetch (`web/studio/assets/js/studio_api.js:205-217`, `:357-...`; `AbortController` at `:205`). Stop aborts the API request or posts worker `cancel` (`index.html:1846-1854`).
- The sidecar exposes `/v1/chat/completions` with `stream:true` and `stream_options.include_usage`; the Tauri host relays SSE bytes through a `tauri::ipc::Channel` (`src-tauri/src/lib.rs:3106-3152`; `src-tauri/src/sidecar.rs:261`). This is a real, executed transport path for **text**.
- It is not a tool-call or agent-event transport. The sidecar's own capability document lists accepted fields and omits `tools`/`tool_choice` (`crates/uor-r4-integer/src/openai.rs:497`; unsupported families include assistants/threads at `:498`).

### Retained executed evidence
- `src-tauri/tests/sidecar_proxy.rs` names tests for native OpenAI surface, partial headers/JSON/SSE bytes, image bounds, explicit cancellation, SSE-before-completion, one live sidecar, shutdown cancelling an active stream, startup failure reaping, and readiness/cancellation (`src-tauri/tests/sidecar_proxy.rs:28-269`). The report records these suites as passing at the relevant heads (`docs/loop/tasks/studio-ide-harness/kimi-17/REPORT.md:16`; `docs/products/desktop-client.md:29-30`).
- `src-tauri/tests/desktop_client_controls.rs` names tests for model pin/tamper refusal, durable close/resume, grants/symlink refusal, terminal approval and allowlist, bounded Git, RPC/loopback static serving, single authority state across pickers/Git, GUI wrappers, async dialogs and dialog parenting (`src-tauri/tests/desktop_client_controls.rs:29-1949`).
- `src-tauri/tests/interactive_terminal.rs` names real PTY input/resize/close, per-open approval, and grant/input/output/session bounds (`:79-180`).
- `src-tauri/tests/mcp_protocol.rs` names remote JSON/SSE sharing, auth/timeout/size caps, stdio framing/approval/cleanup, async error survival, bounded process count/reuse, multibyte errors, slot release and descendant timeout (`:237-506`).
- The retained loopback/packaged acceptance records exact arithmetic `42`, a real `uor-chat` text stream with usage `19/16/35`, two-session create/resume, current-file/pasted attachments, a real approved `pwd`, exact edit/save bytes, and reopen restoring transcript/attachment/editor while resetting grants (`docs/products/desktop-client.md:61-65`; `docs/integration/current-state.md:787-793`). These are **development** host results on synthetic fixtures.

### Inferences
- The cheapest correct path is a browser TypeScript task runtime/state layer (page or worker) with per-run IndexedDB event records and verified snapshot manifests. Rust is limited to an optional schema/canonical reference (`crates/uor-r4-studio-agent`) and existing host adapters; it is not a host-owned agent runtime. Adapt the existing presentation components to render browser events. Reusing the lifecycle store as the per-task UI store is possible, but it must not become the authority for task/event identity.
- The Tauri command + loopback RPC contract is the only host authority worth extending. A second HTTP service, a second MCP client, or a page-side "approval" flag would duplicate or weaken it.
- The object store is scoped to existing Tauri session snapshots and optional bounded exports, not the browser task/event log. The sidecar is a good fit only for text generation. A tool loop runs in the browser task runtime and calls host tools through the approved adapter; it must not pretend the model emitted host authority.

### Gaps
- There is no shared event schema, no generated TypeScript contract, and no `AgentClient`. The current cross-module events are untyped `CustomEvent`s: `uor-generation-finished` (`generation.ts:4`), `uor-files-changed`/`uor-workspace-changed` (`WorkspaceEditor.tsx:44-50`), `uor-view-changed`/`uor-attachments-changed` (`StudioApp.tsx:52-60`; `AgentComposer.tsx:139`). A Cursor-like runtime needs a typed event contract, not more DOM events.
- The loopback RPC is request/response only. Server push today is polling (terminal, 50 ms) or Tauri IPC channels for chat (native only). A hosted browser client would need an authenticated SSE/WebSocket service, which does not exist.
- No object-store schema for agent events/tasks exists; the current durable APIs store model-session snapshots, not task transcripts. No browser IndexedDB task/event schema exists either.

---

## 3. What is the actual current `crates/uor-r4-api` surface, and what streaming/tool contract is NOT available?

### Takeaway
The compiled `uor-r4-api` library is a 17-line re-export shim over `uor_r4_core::native_geometric`. The historical `lib.rs` and `native_capability_api.rs` are preserved but not built. There is **no `ModelApi`, no `ModelApi::stream(ModelRequest)`, no typed model-event stream, and no tool-call contract anywhere in the repository**. The only executable text-stream contract is the `uor-chat` sidecar's OpenAI-compatible SSE, which accepts text and usage options but not tools.

### Cited findings

**Crate root and compiled surface (observed code)**
- `crates/uor-r4-api/Cargo.toml:12-17` disables autobins/autoexamples/autotests and sets `[lib] path = "src/native_support.rs"`.
- `src/native_support.rs:1-17` says every historical API module depends on retired code and is preserved as uncompiled evidence; the only export is `pub use uor_r4_core::native_geometric;`.
- `src/lib.rs` (147 lines) still declares `native_capability_api`, `compile`, `engine`, `serving`, `release_*`, `capability_suite`, etc. (`:35-68`) and re-exports many types (`:104-135`), but this file is not the crate root, so none of that is reachable as `uor_r4_api::...` in the built library.
- A repository-wide search for `uor-r4-api`/`uor_r4_api` outside `crates/uor-r4-api/` found **no consumer**. The workspace registers it (`Cargo.toml:22`) but no product crate depends on it.

**Preserved-but-unbuilt façade (observed code, not a current contract)**
- `native_capability_api.rs:141-149` `SessionConfig` already models `session_id`, `user_id`, `project_id`, `control`, `max_output_tokens`, `temperature`, `top_k` — a useful identity shape, but not compiled.
- `native_capability_api.rs:167-172` `CompletionRequest` has only `prompt`, `max_tokens`, `temperature`, `stop_sequences`.
- `native_capability_api.rs:1448-1457` `complete` and `complete_stream<F: FnMut(&str) -> bool>` stream **text pieces** through a callback. There is no event enum, no tool call, no usage/finish/error event, and no cancellation token type. It is not built by the active lib path.
- `src/bin/r4-native-chat.rs` uses that façade (`:3`, `:88`, `:126`), but it is an unbuilt example binary (`autobins = false`), not evidence of a public harness contract.

**Actual serving contract (observed code)**
- `crates/uor-r4-integer/src/openai.rs:487-496` declares routes `/healthz`, `/v1/models`, `/v1/models/{id}`, `/v1/chat/completions`, `/v1/completions`, `/v1/responses`, `/v1/images/decompose`, `/v1/capabilities`; chat/completions, completions and responses are `streaming: true`.
- `openai.rs:497` lists accepted inputs: text strings, text content parts, `uor_r4_image` parts, max token fields, `stream`, `stream_options.include_usage`, `temperature: 0`, `n: 1`. **`tools` and `tool_choice` are absent.**
- `openai.rs:498` lists unsupported families including embeddings, files, uploads, batches, fine-tuning, assistants and threads.
- `openai.rs:484-485` states `vision: false`, native image content part with decomposition, and `visual_understanding: false`.
- `crates/uor-r4-integer/src/loopback.rs:468` dispatches `POST /v1/chat/completions`; `:726` writes `data: [DONE]`; `:755` writes each chunk. This is standard text SSE.
- The browser client's stream contract is `choices[].delta.content`, `finish_reason`, usage and optional `uor` token metadata (`studio_api.js:388-473`); no `tool_calls` branch exists.

**What is explicitly not available**
- A typed `ModelApi` trait, `ModelRequest` type, `ModelEvent` enum, or `stream()` method: a repository-wide search found **zero** occurrences of `ModelApi`, `ModelRequest`, `ModelEvent`, `AgentRuntime`, `AgentEvent`, or `AgentClient`. The only occurrence of `ModelApi::stream(ModelRequest)` is in the **local draft** `local/kimi/studio-agent-order-draft.md:19`, where it is described as the expected Claude-owned contract to wait for.
- Complete tool calls, tool results, usage/finish/error as ordered events, cancellation as a first-class stream operation, and per-request event identity. The SSE path has finish/usage chunks and AbortController cancellation, but no tool-call event and no server-side task identity.
- A hosted model endpoint. GitHub Pages is static and cannot host `/v1/*`; the native runner serves `uor-chat` only on loopback (`docs/products/web-client.md:31`; `web/studio/README.md` conversation-first section).
- Any model capability claim. The public catalog is synthetic-only (`docs/loop/tasks/studio-ide-harness/kimi-17/REPORT.md:11`); M1 is next-token prediction, not chat (`docs/products/web-client.md:7`).

### Inferences
- A Cursor-like agent runtime cannot be built by calling a nonexistent `ModelApi`. The lawful options are: (a) implement a narrow harness adapter over the sidecar's text-only SSE and keep tool execution in the harness, explicitly not calling it a model tool-call API; or (b) wait for the Claude-owned merged streaming/tool surface and test against its exact head. The draft chooses (b) for the contract test and allows fixtures before it.
- Even option (a) cannot produce model-originated tool calls. Any tool loop would be harness-scripted or a deterministic tool pre-step, which must be labelled as such and must not be presented as model agent capability.
- The uncompiled `complete_stream` is a tempting false positive. It must not be cited as an existing streaming contract; its callback takes text only and it is not the crate root.

### Gaps
- The owner/Claude-side API surface, its exact event schema, tool-call shape, identity fields, and merged head are unknown from this repository. `docs/products/harness-parity.md:39` records the R4 model API integration row as NOT_RUN.
- No fixture exists for a tool-call stream, malformed/partial events, duplicate/out-of-order events, or stale-generation rejection.
- No measurement exists for browser/native latency or throughput of an agent loop; the only retained throughput numbers are synthetic text decode on specific artifacts (e.g. `docs/research/notebook/m4-serving.md:35-36`).

---

## 4. What does `/Users/casey.allard/uor-r4/local/kimi/studio-agent-order-draft.md` already specify for the shared runtime, events/snapshots, tasks, stop/redirect, approvals, persistence and acceptance?

### Takeaway
The draft is a precise, non-implemented ORDER for the next protected unit. It proposes a new `crates/uor-r4-studio-agent` runtime, narrow `src-tauri` adapters, a typed `agentClient.ts`/`state/agent.ts`, a versioned event schema, independent task cursors, generation-scoped cancellation, next-safe-boundary redirection, host-consumed approvals, proposed object-store snapshots with serialized writers, one schema for Tauri and authenticated HTTP/SSE, and red-first acceptance. It explicitly says the current `complete_stream` façade is not a verified streaming/tool contract and that the real R4 API test remains NOT_RUN until Claude's merged surface exists. The draft's Rust-runtime and object-store task persistence is not adopted in §5; this baseline recommends a browser TypeScript runtime with per-run IndexedDB event records, with `crates/uor-r4-studio-agent` retained only as an optional schema/canonical reference.

### Cited findings (all are draft text, not implemented code)
- **Base and status:** base `fbf15e56f3ab1961b30610893392e7a97fd0ab18`; "local preparation draft, not a claim that the runtime exists" (`local/kimi/studio-agent-order-draft.md:1-3`).
- **Scope:** new `crates/uor-r4-studio-agent/` with model, events, runtime, persistence, scheduler, contract exporter and tests; root workspace registration only; `src-tauri/src/agent_host.rs` and `agent_server.rs` as narrow adapters over existing host grants/tools; `web/studio/src/adapters/agentClient.ts`, `state/agent.ts`, generated contract reader and hostile transport/component tests (`:5-10`).
- **Dependencies:** reuse `uor-r4-object-store`, `serde`, `serde_json`, `sha2`, `reqwest`, locked `futures = "0.3.33"`; no new database, Electron, Svelte, node-pty, public socket, CDN or second model API; worker threads can drive futures streams without a second async runtime; any actually missing dependency must be declared before install (`:12-14`).
- **Contract boundary:** the active `native_support.rs` only re-exports `native_geometric`; the preserved `native_capability_api.rs` contains `complete_stream` but is not built by the active lib path and is "not a verified harness streaming/tool contract. Wait for Claude's actual merged API surface" (`:18`).
- **Model stream:** `ModelApi::stream(ModelRequest)` yields text, complete tool calls, usage, finish and error events through a cancellable stream; malformed/partial events are errors, never fabricated assistant/tool results (`:19`). This is a **target dependency**, not current code.
- **Runtime:** `AgentRuntime::{create_task, steer, stop, approve, resume, schedule}` emits ordered versioned events keyed by project/task/sequence. Approval requests carry identity only; the host consumes actual grants; a page-supplied boolean is never authority (`:20`).
- **Tasks:** independent event cursors, artifacts, permissions and model state; cancellation is generation-scoped; redirection applies at the next safe tool boundary; stale events cannot affect another task or restarted generation (`:21`).
- **Persistence:** content-addressed object-store records and bounded versioned payloads; serialize writers because the store does not guarantee concurrent writers or directory-metadata crash durability; restore by verified manifest and event identity, not filename trust (`:22`).
- **Transport:** embedded Tauri and authenticated HTTP/SSE clients consume one schema; loopback default; remote mode requires explicit HTTPS configuration; static web has explicit unsupported states (`:23`).
- **Red-first acceptance:** prove malformed/partial streams, duplicate and out-of-order events, parallel independent tasks, stop preventing later tools, queued redirection, side-question isolation, child fan-out/join, approval forgery refusal, restart/event-cursor recovery, stale-generation rejection and truncated/corrupt snapshot refusal before implementing behavior; exercise real object-store save/reopen and two live worker tasks; run the new crate, desktop default/GUI, TypeScript contract, existing browser, local policy and full repository checks serially; exact-head review and protected merge; real Claude-owned R4 API contract test remains NOT_RUN until its exact merged surface is available; do not infer capability from fixture model text (`:26-29`).
- **Stop:** stop before weakening an approval or identity check, adding an undeclared dependency, exposing a public unauthenticated endpoint, using real credentials/profiles, or claiming hosted production, useful model behavior or full Cowork parity from a fixture (`:31-33`).

### Inferences
- The draft is consistent with the actual code baseline in the important places: it does not assume the API exists, it does not assume concurrent object-store writers, and it keeps approval authority host-side.
- Its file plan is a good starting point for a Cursor-like plan, but the draft's vocabulary and acceptance are Cowork-oriented (`side questions`, `child tasks`, `Dispatch`) and would need Cursor-specific UI acceptance for editor/diff/terminal/integrations/account.
- Because the draft's base differs from this worktree's base, any later ORDER must re-verify that the named files and line-level assumptions still hold at the new base.

### Gaps
- The draft does not define the concrete event JSON schema, field names, version string, error taxonomy, cursor encoding, or approval object shape.
- It does not specify a Cursor-like UI, file tree, diff comments, account/settings, or hosted service deployment.
- It does not say which exact tests are expected to fail first (names/paths), only the categories.
- It does not resolve the static-browser story beyond "explicit unsupported states"; it does not define what a static Cursor-like shell may show.

---

## 5. Which exact files and tests should later tasks create or modify for Cursor-like UI, runtime, integrations, account/settings and acceptance, without changing model/API implementation or owner metadata?

### Takeaway
The following is a **proposed file map** derived from the baseline, not approved scope. It respects the owning item's rules: no edits to `crates/uor-r4-api/**`, model/engine/worker/tokenizer, owner roadmap metadata (`docs/roadmap/README.md`, `docs/roadmap/issues.json`), `.github/**`, or the owner's checkout. Browser runtime/state work goes in `web/studio/src/runtime/agent` and `web/studio/src/state`; the Rust crate is optional schema/canonical reference only; host tool adapters go in `src-tauri/src`; acceptance goes in tests and the existing product/loop records.

### Proposed create/modify map

**Browser task runtime (create; primary)**
- `web/studio/src/runtime/agent/taskRuntime.ts` — task/run state machine, queue/steer/stop/retry, generation-scoped cancellation, next-safe-boundary redirect.
- `web/studio/src/runtime/agent/eventLog.ts` — per-run append-only event records in IndexedDB, cursor encoding, stale-generation rejection, restart recovery.
- `web/studio/src/runtime/agent/snapshot.ts` — verified snapshot manifests in IndexedDB; restore by manifest/event identity, not filename trust; optional bounded export to the Tauri object store.
- `web/studio/src/runtime/agent/scheduler.ts` — recurring tasks only if scheduling is in scope; otherwise omit and mark NOT_RUN.
- `web/studio/src/state/agent.ts` — per-task stores and event cursor; do not reuse the global `LifecycleStore` as the task authority.
- `web/studio/src/adapters/agentClient.ts` — transport-neutral client over Tauri IPC or optional HTTPS/SSE; consumes the generated schema.
- `web/studio/src/generated/agentContract.ts` — generated from the schema; no hand-forked event union.
- `web/studio/src/runtime/agent/{taskRuntime,eventLog,snapshot}.test.ts`, `web/studio/src/state/agent.test.ts` — hostile event, cursor, restart, isolation and snapshot tests.
- `web/studio/src/runtime/agent/fixtures/{model_streams,events,snapshots}/` — malformed, partial, duplicate, out-of-order, stale-generation, truncated/corrupt.

**Schema/canonical reference crate (optional; create only if needed)**
- `crates/uor-r4-studio-agent/Cargo.toml`
- `crates/uor-r4-studio-agent/src/lib.rs`
- `crates/uor-r4-studio-agent/src/events.rs` — versioned `AgentEvent` enum/JSON schema, sequencing, cursor encoding and error taxonomy; canonical reference only, not a runtime.
- `crates/uor-r4-studio-agent/src/contract.rs` — exported JSON schema/version consumed by TypeScript generation.
- `crates/uor-r4-studio-agent/tests/contract_events.rs` — schema/validation conformance.
- Root `Cargo.toml` — add workspace member only if the schema crate is added (observed workspace list at `Cargo.toml:2-22`).

**Host adapters (create/modify)**
- `src-tauri/src/agent_host.rs` (create) — translates host grants/tools into the browser runtime's host-tool contract; consumes approvals; no task authority.
- `src-tauri/src/agent_server.rs` (optional; only if a hosted service is in scope) — authenticated event/export transport; not the local runtime and not task authority.
- `src-tauri/src/lib.rs` (modify) — register new host commands in `generate_handler!`, extend `dispatch_rpc_command`, keep existing commands unchanged.
- `src-tauri/capabilities/default.json` (modify only if a new window/permission is genuinely required; currently `core:default` and `windows:["main"]` at `:5-8`).
- `src-tauri/tests/{agent_host.rs,agent_authority.rs}` (create) — approval forgery, out-of-grant path, stale task, concurrent host calls, authority checks; add `agent_server.rs` tests only if the optional service exists.
- `src-tauri/tests/desktop_client_controls.rs` (modify only to add regression coverage for shared authority; do not weaken existing tests).

**Presentation integration (create/modify)**
- The client, task state and generated contract are listed under the browser runtime above; do not create a second copy.
- `web/studio/src/adapters/workflow.ts` (modify) — widen `WorkspaceApi` only as needed for a file tree/search; keep the legacy seam.
- `web/studio/src/state/lifecycle.ts` (modify cautiously) — add only presentation state needed by the new shell; do not encode task/event identity here.
- `web/studio/src/persistence/migration.ts` and `web/studio/assets/js/lifecycle_persistence.js` (modify) — add a new versioned key for agent/task UI state; preserve copy-forward, marker and rollback; do not rewrite existing keys in place.

**UI components (create/modify)**
- Create: `web/studio/src/components/ProjectHome.tsx`, `TaskTimeline.tsx`, `ApprovalCard.tsx`, `FileTree.tsx`, `EditorTabs.tsx`, `DiffReview.tsx`, `BrowserPane.tsx`, `ArtifactPreview.tsx`, `EnvironmentPicker.tsx`, `PermissionPicker.tsx`, `AccountSettings.tsx`, `NotificationCenter.tsx`, `SchedulePanel.tsx` (only if scheduling is in scope).
- Modify: `AppShell.tsx`, `SessionToolbar.tsx`, `ProjectSidebar.tsx`, `PaneDock.tsx`, `AgentComposer.tsx`, `AgentTranscript.tsx`, `SessionList.tsx`, `WorkspaceEditor.tsx`, `DesktopTools.tsx`, `Settings.tsx`, `StudioApp.tsx`, `types.ts`, `studio.css` (or split per-surface CSS).
- `web/studio/src/bootstrap.ts` / `main.ts` (modify) — mount the new shell; keep `?legacy=1`/`?typed=0` behavior; do not break the built-default rewrite in `vite.config.ts`.
- Reuse `@xterm/xterm` and `@xterm/addon-fit` (`web/studio/package.json:12-16`); do not add a code editor dependency without an explicit ORDER, because the current editor is dependency-free and the plan's global constraint is no undisclosed runtime dependency (`docs/superpowers/specs/2026-11-08-claude-cowork-parity-design.md:43`).

**Tests (create/modify)**
- Component/adapter tests: `web/studio/src/components/{ProjectHome,TaskTimeline,ApprovalCard,FileTree,DiffReview,BrowserPane,ArtifactPreview,AccountSettings}.test.tsx`; `web/studio/src/adapters/agentClient.test.ts`; `web/studio/src/state/agent.test.ts`.
- E2E: `web/studio/tests/e2e/{cursor-shell.spec.ts,projects.spec.ts,tasks.spec.ts,editor.spec.ts,integrations.spec.ts,account.spec.ts,hostile.spec.ts}`; extend `support.ts` with a real runtime fixture or an explicit `NOT_RUN` static path.
- Existing tests to preserve: `web/studio/tests/e2e/{shell,visual,workflows,terminal}.spec.ts` and the 52 Vitest files. Do not delete a passing assertion without an equivalent or stronger named replacement (`docs/superpowers/specs/2026-11-08-claude-cowork-parity-design.md:152`).
- Host tests: add to `src-tauri/tests/` as above; keep the existing default/GUI matrices and the 50 ms terminal polling contract regression.

**Integrations and account/settings (create/modify)**
- `web/studio/src/components/McpConnections.tsx` (modify) — tool results must enter a task only through the runtime, not only through `addAttachment`.
- `web/studio/src/components/DesktopTools.tsx` (modify) — split Git status into per-file entries and a diff view; add comment persistence only through the new store.
- `web/studio/src/adapters/desktopActions.ts` (modify only if new host commands are added; keep the unavailable fallback).
- `web/studio/src/components/AccountSettings.tsx` (create) — local profile/settings only; no real credentials or account service. If a hosted account is later wanted, it is a service-side item, not a static-page item.
- `web/studio/src/components/NotificationCenter.tsx` (create) — local completion/approval notices; static page cannot claim background notifications.

**Acceptance and records (create/modify in later units)**
- `docs/products/harness-parity.md` — add a Cursor-specific matrix or a clearly separate target table; do not rewrite Cowork rows into claims.
- `docs/products/web-client.md`, `docs/products/desktop-client.md` — truthful capability and host updates.
- `docs/integration/current-state.md`, `docs/research/notebook/m4-serving.md` — one row per measured result.
- `docs/loop/tasks/studio-ide-harness/kimi-18/**` — ORDER/REPORT for the planning unit; later units get their own folders.
- Evidence under ignored `local/kimi/cursor-hosted-research/**`; no generated assets, screenshots, binaries or credentials committed.

### What must not be touched
- `crates/uor-r4-api/**` (explicit rule, `docs/roadmap/items/studio-ide-harness.md:39`; design constraint `docs/superpowers/specs/2026-11-08-claude-cowork-parity-design.md:37`).
- Model, engine, worker and tokenizer implementations; `web/studio/assets/js/uor_model_worker.js`; `crates/uor-r4-integer/**` except through an agreed adapter.
- `docs/roadmap/README.md` and `docs/roadmap/issues.json` (owner metadata).
- `.github/**`, production deployment, real credentials/profiles, paid services, second model API.
- The owner's checkout and the two pre-existing modified roadmap files.

### Inferences
- A Cursor-like UI needs a new editor surface (file tree/search, tabs, diff comments), not just a restyle of `WorkspaceEditor`. The current textarea is sufficient for a minimal manual editor, not for a Cursor-like editing experience.
- Account/settings should be scoped honestly: local settings and local profiles can be built now; hosted auth/account is blocked on a service that does not exist.
- The most defensible first vertical slice is: event contract + single-task runtime + one project/task view + approval card + stop/redirect + restart recovery, with the existing editor/diff/terminal panes reused as task surfaces. UI breadth before runtime would be capability theater.

### Gaps
- No approved Cursor-specific target/scope exists yet; the roadmap item still names Cowork and says "Claude Desktop Cowork/Code-style" (`docs/roadmap/items/studio-ide-harness.md:16`).
- No decision on whether the Cursor-like shell keeps the legacy `#app` coexistence or replaces it; both are possible, but replacing it would require migrating all legacy selectors and tests.
- No decision on a code editor dependency (Monaco/CodeMirror) or a file-tree/search implementation. The current plan's dependency rule requires naming it in an ORDER before install.

---

## 6. What are the static-browser, Tauri/local-companion and hosted-service boundaries, and what are the current evidence limits?

### Takeaway
There are three distinct execution environments, and only two exist in executable form. **Static browser** (GitHub Pages) has the shell, WASM artifact loading/prediction, exact arithmetic, localStorage persistence and explicit unsupported states; it has no host tools, no API host, no background work. **Tauri/local companion** is an unsigned development app with 36 host commands, a token-protected loopback bridge, granted folders, a real PTY, remote/stdio MCP and a host-managed `uor-chat` sidecar; it is not a signed or production release. **Hosted service** does not exist: no account, no background task service, no authenticated public endpoint, no cross-device resume.

### Cited findings

**Static browser**
- The published `/studio/` is the neutral workbench; `?legacy=1` and `?typed=0` retain the legacy interface (`docs/integration/current-state.md:5`; `live-acceptance.json` `legacySelectors`).
- Static terminal says "Interactive PTY is unavailable in this browser build." and disables Open (`TerminalSession.tsx:236`; `tests/e2e/terminal.spec.ts:4-20`).
- Changes says "Native Git is unavailable in this browser build; use the desktop build." when `actions.available` is false (`DesktopTools.tsx:44`).
- The browser pane says a harness service is required and does not import cookies or expose existing tabs (`StudioApp.tsx:103`).
- The page makes zero external requests at load; retained live evidence reports 14 requests with zero external/raw-TypeScript/runtime errors (`live-acceptance.json` `publicRuntime`; `docs/products/harness-parity.md:38`).
- The public catalog is synthetic-only (1,346 bytes, SHA-256 `fa33aa23…`), and the bundled artifact is the 458 KB synthetic stack (`docs/loop/tasks/studio-ide-harness/kimi-17/REPORT.md:11`; `public-runtime.json` `bodyText`).
- GitHub Pages cannot host an API; the native runner exists because of that (`docs/products/web-client.md:31`; `web/studio/README.md` conversation-first section).

**Tauri/local companion**
- The app identity is `org.uor.r4.desktop.cowork` (`src-tauri/tauri.conf.json:5`), product name "UOR Harness", bundle target `app`, external binary `binaries/uor-chat` (`:20`), resources `runtime/studio/pkg`, `models.json`, `SHA256SUMS` (`:21-25`).
- The window is created in code at `index.html?typed=1`, 1440×900, with navigation confined to the app asset origin (`src-tauri/src/lib.rs:3652-3674`). `tauri.conf.json:11` deliberately has no declarative windows.
- The CSP is `default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; ... connect-src 'self' ipc: http://ipc.localhost; worker-src 'self'` (`tauri.conf.json:13`). Remote MCP is a host-mediated exception, not a page fetch.
- 36 `#[tauri::command]` handlers are registered (`lib.rs:3677-3714`), plus the stdio RPC loop (`main.rs:1-12`) and the loopback `--serve-studio` host (`main.rs:1-9`; `lib.rs:2590-2717`).
- Folder access is grant-based; the page cannot call `native_grant_folder` (`docs/products/desktop-client.md:89`). Terminal is disabled by default and requires native enablement plus exact command approval (`:90`).
- The packaged app is **unsigned development evidence**. The record says the resource contract passes and native Workspace/Split/New session interactions pass, but the complete native matrix is NOT_RUN (`docs/products/harness-parity.md:40`). The latest GUI acceptance is historical at code head `42bc46f…`, executable `c8ddeb40…`; later package builds are contract-only and no fresh GUI acceptance is claimed at the delivery head (`docs/products/desktop-client.md:61`).
- `src-tauri/tauri.conf.json:11` `windows: []` plus the code-created window is a trap for a reviewer: a plain `cargo build` can embed older staged assets and is not desktop evidence (`docs/products/desktop-client.md:51`).

**Hosted service**
- No hosted agent service, account system, auth, scheduling, notifications, Dispatch or cross-device resume exists. The parity matrix marks all of these NOT_RUN (`docs/products/harness-parity.md:19-37`), and the kimi-17 REPORT says they "remain unbuilt" (`kimi-17/REPORT.md:41`).
- Hosted production is NOT_RUN until an owner-provided remote host is verified (`docs/products/harness-parity.md:41`; `docs/roadmap/items/studio-ide-harness.md:31`).
- The roadmap item's "done when" requires one versioned agent contract driving the same runtime in Tauri/local and hosted-service modes, durable event/snapshot persistence and explicit static-preview limits (`docs/roadmap/items/studio-ide-harness.md:27`). This note reads "same runtime" as the browser task runtime: Rust supplies the schema/canonical reference, Tauri supplies host tools and optional bounded export, and the hosted service, if any, relays events. That is the target, not current state.
- The only loopback service that exists is the native `uor-chat` runner (`127.0.0.1:8000`), which is local, token-protected, and text-only (`docs/products/web-client.md:31`). It is not a hosted multi-user service.

**Evidence limits**
- This baseline did not execute anything. The 455/47/15/11/7 counts are from retained logs at the kimi-17 fresh-main head; they are not a fresh verification at `452615bc5`.
- The public runtime's only model-visible output is exact arithmetic, not generated prose. The 47/47 Playwright run does not exercise real model chat.
- The packaged GUI acceptance is historical and unsigned; the AI cannot claim a fresh packaged result at this base.
- The public catalog is synthetic-only. The M1 release exists on Hugging Face, but it is not in the current public catalog and is a next-token predictor, not a chat model (`docs/products/web-client.md:7`; `docs/loop/tasks/studio-ide-harness/kimi-17/REPORT.md:11`).
- No measurement exists for a Cursor-like task loop, concurrent tasks, hosted latency, account/auth, or energy. Any such number in a later plan must come from its own run.

### Inferences
- Static Pages can host a **Cursor-like shell preview** and a local WASM model path, but not the Cursor-like hosted agent experience. The honest static product is a demo/explorer with explicit "service required" states.
- The Tauri app is the correct first home for host tool authority (grants, PTY, MCP, sidecar); the browser is the first home for the task runtime. The missing pieces are the browser runtime, the typed event contract and the UI, not a new host.
- A hosted service would need a separate authenticated server, durable storage, per-user identity and an event transport. None of that should be simulated in the static page. If the owner wants hosted production, that is a new service item with its own security review, not a frontend change.
- The current single-user/local-first model (`docs/superpowers/specs/2026-11-08-claude-cowork-parity-design.md:159`) is the safest boundary for the first Cursor-like slice; cross-device/account features should remain NOT_RUN until a service exists.

### Gaps
- No owner-provided hosted endpoint, auth provider, account model, or deployment target exists.
- No decision on whether a hosted service is in scope for the Cursor plan or remains a later item.
- No evidence for service restart/background completion, two-client resume, notification delivery, or approval notification latency.
- No evidence for a Cursor-like editor, diff comments, browser pane, computer use, or connector loop in either static or native mode.

---

## Handoff summary for the lead

**Reusable paths (do use)**
1. One Vite/Preact frontend and one staged `dist` for web, native runner and Tauri (`vite.config.ts:22-29`; `tauri.conf.json:7`; `docs/products/web-client.md:49`).
2. Typed lifecycle + adapters as presentation guards: stale-operation refusal, save serialization, verified session mutations, artifact SHA/capability checks (`operations.ts:5-20`; `fileSave.ts:4-48`; `agentSession.ts:59-148`; `modelSourceAdapter.ts:185-238`).
3. Tauri command + loopback `/__tauri_invoke` + stdio JSON-RPC as the only host authority, with Host/Origin/Content-Type/token checks (`lib.rs:2590-2651`, `:3677-3714`; `main.rs:1-12`).
4. Existing grants, PTY, Git, MCP and sidecar boundaries (`lib.rs:1024-1073`, `:3559-3645`; `terminal.rs:369-...`; `mcp.rs:555-...`; `sidecar.rs:261-383`).
5. `uor-r4-object-store` `IdentityScope` + manifests for existing Tauri session snapshots and optional bounded exports; browser task/event records live in IndexedDB (`durable_consumer.rs:46`, `:283`, `:569`; `lib.rs:15-24`).
6. The browser's OpenAI-compatible SSE client and the native SSE relay for text only (`studio_api.js:357-...`; `lib.rs:3106-3152`).

**Gaps (do not paper over)**
1. No project/task/plan/tool/approval/event model; session/file/model are single-valued (`lifecycle.ts:71-90`; `workflow.ts:47-68`).
2. No file tree, search, syntax highlighting or diff comments in the typed shell (`WorkspaceEditor.tsx:89-113`; `DesktopTools.tsx:21-48`).
3. No account/auth/scheduler/notification/hosted service (`harness-parity.md:19-37,41`).
4. No `ModelApi`/streaming tool contract; `crates/uor-r4-api` compiles only `native_support.rs`, and the sidecar accepts no tools (`uor-r4-api/Cargo.toml:12-17`; `native_support.rs:17`; `openai.rs:497`).
5. No concurrency, stop/redirect, side-question, child-task, approval-forgery, restart/cursor or hostile-transport evidence.

**Boundary reminders**
- Do not edit `crates/uor-r4-api/**`, model/engine/worker/tokenizer, owner roadmap metadata or `.github/**`.
- Static page may show only a limited preview; native is unsigned development; hosted production is NOT_RUN.
- The current public proof is exact arithmetic (`17 + 25 = 42`) and a neutral shell, not useful conversation or agent behavior (`live-acceptance.json`; `public-runtime.json`).
