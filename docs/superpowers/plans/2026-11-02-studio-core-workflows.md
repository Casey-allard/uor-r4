# Studio Core Workflows Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build all eight approved typed Studio workflow components against the lifecycle store and existing public browser modules, without replacing the live page or changing its engine.

**Architecture:** Preact components are composed around one lifecycle store through subscribing views and prop/callback-coordinated views, with narrow injected TypeScript bridges over `__studioFiles`, `__studioUi`, `UORModelSources`, and `UORStudioApi`. Small additive legacy hooks expose editor content, generation completion, and settings updates; components never scrape DOM or duplicate model/file/API/persistence behavior. The library exports an unmounted `StudioApp`; the legacy `index.html` remains live.

**Tech Stack:** TypeScript/TSX, Preact, Vite, Vitest/jsdom, existing JavaScript browser modules, Node's built-in test runner.

**Spec:** `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`

## Global Constraints

- Keep the legacy page live: no entry-point replacement, publisher, deployment, real browser-data migration, or production claim.
- Keep `uor_model_worker.js`, `uor_tokenizer_artifact.js`, `studio_api.js`, the engine/WASM boundary, model artifacts, and `.github/**` byte-identical.
- Call existing public APIs; do not scrape DOM or duplicate tokenization, generation, hashing, downloader, file-save, session, or provider persistence logic.
- All visible state and enabled actions derive from the lifecycle store. Unknown progress totals and finalization never acquire percentages or invented stages.
- Native `<progress>` and `<dialog>` semantics; accessible names, keyboard-complete controls, no manual focus traps/inert toggles/redundant Escape handlers.
- API keys stay only in the existing `modelSources.v1` writer. Fixtures contain no real browser data or credentials. No new dependency beyond the pinned Preact toolchain.

## Review Focus

- A late model/API/generation completion must update the subscribed UI without relabeling a newer selection or generation (Tasks 1, 2, 4, 6).
- Empty/no-file, dirty/saving/error, unsupported-prediction, and no-endpoint states must not expose enabled actions that cannot work (Tasks 4–7).
- Native progress/dialog behavior must preserve unknown totals, accessible names, and native focus return (Tasks 1, 3, 4, 6, 7).
- Editor content and message actions must reach the existing file/save implementations without losing edits or bypassing their persistence (Tasks 2, 5, 6).
- Session/provider/data operations must retain existing storage semantics and never expose secrets in state, logs, or fixtures (Tasks 2, 4, 6, 7).

---

### Task 1: Subscribed lifecycle and TSX test/runtime scaffold

**Files:** `web/studio/src/state/lifecycle.ts`, `lifecycle.test.ts`, `src/state/useLifecycle.ts`, `src/components/Modal.tsx`, `src/components/Modal.test.tsx`, `web/studio/tsconfig.json`, `web/studio/vite.config.ts`.

**Interfaces:** add `LifecycleStore.subscribe(listener: () => void): () => void`; `useLifecycle(store): LifecycleState`; `Modal({ title, onClose, children })` using a named native dialog. Existing dispatch/getState behavior remains compatible.

- [ ] Write failing tests proving subscribe fires after an external `api/selected` transition, unsubscribe stops notifications, and Modal calls `showModal()`/`close()` with an accessible title and native close event.
- [ ] Run `npm test -- src/state/lifecycle.test.ts src/components/Modal.test.tsx`; expect missing subscribe/TSX failures.
- [ ] Implement subscription by applying the existing reducer then notifying only changed states; add `useSyncExternalStore` through `preact/compat`, Preact JSX compiler settings, and `src/**/*.test.{ts,tsx}` discovery.
- [ ] Rerun both tests plus `npm run typecheck` and `npm run build`; expect PASS.
- [ ] Commit the subscription and scaffold.

### Task 2: Typed legacy bridges and additive public hooks

**Files:** create `src/adapters/workflow.ts`, `generation.ts`, and tests; modify `assets/js/studio_files.js`, `studio_ui.js`, `index.html`, and `model_sources.d.ts`.

**Interfaces:** define `WorkspaceApi`, `SessionApi`, `StudioUiApi`, `SessionSummary`, `ChatMessage`, `Attachment`, `StudioSettings`, and `createLegacyWorkflowApis()` over existing globals. Add legacy `setActiveContent(content)`, `sendPrompt(prompt)`, `stopGeneration()`, `updateSettings(patch)`, `templatePrompt(kind)`, `clearSessions()`, and `clearEditor()` hooks; emit `uor-generation-finished` with `{ outcome: 'complete'|'stopped'|'error' }`. Add `createGenerationAdapter(store, sessions): { send(prompt): boolean; stop(): void; dispose(): void }`.

- [ ] Write failing adapter tests for content forwarding, send readiness/source snapshots, stopped/error completion, listener disposal, settings/template calls, and provider list/add/update/remove declarations. Test additive legacy hook behavior through the existing page/verifier fixture where practical; do not use real storage or network.
- [ ] Run the new Vitest/Node checks; expect missing-hook failures.
- [ ] Implement only forwarding wrappers and the small public hooks, reusing existing internal functions. Do not alter existing storage keys or engine/API implementations.
- [ ] Rerun bridges, foundation, typecheck, build, and `node verify_studio_page.mjs --strict-router`; expect PASS.
- [ ] Commit the bridges and hooks.

### Task 3: AppShell and command palette

**Files:** create `src/components/AppShell.tsx`, `CommandPalette.tsx`, their `.test.tsx` files, and `src/components/types.ts`.

**Interfaces:** `AppShell({ activeSection, onSection, status, sessions, children })`; four named destinations Agent/Workspace/Models/Guide, one `<main>`, one top-level status region, labeled responsive navigation drawer, and Inspector toggle. `CommandPalette({ commands, onClose })` filters and runs only supplied implemented commands.

- [ ] Write failing tests for one main/navigation/status region, four accessible destinations, drawer `aria-expanded`, keyboard palette filtering/ArrowUp/ArrowDown/Enter, native dialog close/focus return, and no disabled action execution.
- [ ] Run the component tests; expect missing-component failures.
- [ ] Implement the shell and palette with semantic controls and native Modal; no page entry or CSS compatibility IDs.
- [ ] Rerun component and foundation tests, typecheck, and build; expect PASS.
- [ ] Commit the shell and palette.

### Task 4: ModelPicker and truthful artifact/API states

**Files:** create `src/components/ModelPicker.tsx` and `.test.tsx`; modify `src/state/lifecycle.ts`, `src/adapters/modelSourceAdapter.ts`, and tests as needed.

**Interfaces:** `ModelPicker({ store, adapter, sources, onError })`. Cover bundled/local-file/URL/published loads; existing API list/add/use/test/edit/remove; native progress and artifact metadata. Add optional `canPredict` to worker/resident capability mapping; discrete `canGenerate: false` is labeled next-token prediction, never conversational chat.

- [ ] Write failing tests for selected-not-ready, known/unknown totals, finalizing/error, stale replacement disabled runs, SHA/engine/capability details, accessible native progress, provider dialogs, and no secret in rendered state.
- [ ] Run ModelPicker and adapter/store tests; expect missing-component/capability failures.
- [ ] Implement the component using Task 2 bridges and existing model-source methods; preserve operation/selection epochs and fail-closed endpoint behavior.
- [ ] Rerun all model/foundation tests, typecheck, build, and Node model-source tests; expect PASS.
- [ ] Commit ModelPicker.

### Task 5: WorkspaceEditor and existing file-save behavior

**Files:** create `src/components/WorkspaceEditor.tsx` and `.test.tsx`; modify `src/adapters/workflow.ts` only if a test exposes a bridge mismatch.

**Interfaces:** `WorkspaceEditor({ store, files, onError })` with folder/import/demo/open-tab actions, active-file editor, dirty/saving/saved/error status, save/save-as, and disabled empty-state actions. Content updates go only through `files.setActiveContent`; saves use `saveActive`/`saveAsCopy`.

- [ ] Write failing tests for no folder/no file, folder/file/tab transitions, dirty content, save start/success/failure, save-as, close, and no enabled empty-state save/attach action.
- [ ] Run WorkspaceEditor and bridge tests; expect missing-component failures.
- [ ] Implement the component and synchronize file events with the lifecycle store; never mutate a returned file object or reimplement persistence/download behavior.
- [ ] Rerun workspace/foundation tests, typecheck, and build; expect PASS.
- [ ] Commit WorkspaceEditor.

### Task 6: AgentComposer, SessionList, and Inspector

**Files:** create `src/components/AgentComposer.tsx`, `SessionList.tsx`, `Inspector.tsx`, and `.test.tsx` files.

**Interfaces:** AgentComposer supports API chat versus local next-token labeling, attachments, all six existing build templates, prompt send/stop, and message copy/insert/save/retry through injected APIs. SessionList supports new/select/rename/duplicate/export/delete with native confirmation. Inspector is on-demand details, not a permanent status mirror.

- [ ] Write failing tests for empty sessions, pinned source/privacy through selection changes, send/stop/error states, attachment add/remove/clear, six template choices, message actions, every session operation, and accessible dialog semantics.
- [ ] Run the three component suites; expect missing-component failures.
- [ ] Implement components using only Task 2 APIs; never print provider credentials, add runtime requests, or relabel M1 prediction as chat.
- [ ] Rerun all component/foundation tests, typecheck, and build; expect PASS.
- [ ] Commit the primary agent/session workflow.

### Task 7: Guide, Settings, composed library, parity record, and verification

**Files:** create `src/components/Guide.tsx`, `Settings.tsx`, `StudioApp.tsx`, tests, and `src/studio.css`; modify `src/main.ts`, `web/studio/README.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`, and this unit's `REPORT.md`.

**Interfaces:** `StudioApp({ store, adapter, sources, files, sessions, ui, download? })` composes every component without mounting. Settings exposes only implemented appearance/editor/data controls through existing writers. Guide documents actual behavior and M1 prediction limits. CSS supplies responsive/high-contrast component styling without changing `index.css`.

- [ ] Write failing composed-app tests for all four destinations, settings theme/reduce-motion/font/tab/wrap updates, confirmed data clearing, guide accuracy, and one coherent status source. Add an explicit parity record separating implemented core behavior from remaining real-browser, migration, and cutover checks.
- [ ] Run all tests; expect missing composition/settings failures.
- [ ] Implement Guide, Settings, StudioApp, exports, and styling; update docs without claiming browser parity, deployment, or cutover. Keep #3500 ready.
- [ ] Run `npm test && npm run typecheck && npm run build`, Node model-source tests, legacy strict page checks, full repository gates, claim wording, local policy, and diff/status checks. Record exact counts/size and known R4G1/manual File System Access gaps.
- [ ] Commit, obtain fresh exact-head review, require green GitHub checks, merge through the protected queue, verify fresh `main`, and remove only this branch/worktree.
