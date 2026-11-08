# UOR-R4 Studio shell upgrade — rebased and rescoped 1 November 2026

## Intent and why

Rebuild the Studio's browser shell so its model, file, session, and privacy states are coherent by construction, and so future work on the Studio does not require another full-page reskin. The owner approved the recommended typed-shell direction: Preact + Vite + TypeScript, with the existing WASM worker, model-source/API adapters, offline behavior, and publication pipeline preserved.

The reviewed defect is fragmented state ownership. Independent DOM writers report model/file/session status without one authoritative lifecycle store. Static markup starts with `Saved` beside `No file open`, and selected model source, loaded artifact, and mirrored status are computed separately. Two presentation redesigns improved the shell's appearance but did not make contradictory states impossible.

## Rebased review and scope decision

- Reviewed at `381ae4d98b0a5100d241a268d283629ab9b5fd1f` and merge-forwarded to `03800ffd24c183e806c6690f684495becbe1cf25`: Studio source is byte-identical to the original design base and unchanged by the merge-forward; there is no newer implementation to adopt. The source is 11,068 lines across HTML, CSS, and JavaScript with no component build system. `node web/studio/verify_studio_page.mjs --strict-router` passes its static zero-external-load check.
- DeepSeek's completed v2 and its historical plain-JS constraint stay untouched. This is a new owner-approved scope, tracked separately, not a reopening or rejection of that work.
- Keep the selected Preact + Vite + TypeScript approach. The defect is still architectural, but delivery is phased: a foundation unit first, workflow migrations second, full-parity cutover last. There is no implementation in this design PR.
- Correct the original loading assumption: `model_sources.js` already emits real downloaded-byte counts, totals when known, and percentages. The new store must expose them. Only finalization stages without events are indeterminate.
- Current persistence is part of the parity contract, not an optional migration detail. Existing keys are inventoried below, including the current `ui.v2` and older `ui.v1` forms, and no real browser data is read or written in the foundation unit.

## Goals

- One typed lifecycle store for model sources, loaded artifacts, files, sessions, and privacy mode.
- Every badge, button, status line, empty state, and enabled action derives from that store.
- A clear primary workflow: open workspace → attach context → choose model → run → review/edit/save.
- Separate source selection from loaded model state; separate local next-token prediction from chat.
- Real byte-download progress when reported, with indeterminate finalization and stale-load rejection when another source supersedes an operation.
- Responsive, keyboard-complete, higher-contrast UI with useful empty states and no duplicate status bars.
- Semantic landmarks and controls, native modal dialogs, and native progress elements rather than custom focus/status mirrors.
- A component architecture that supports phased migration and behavioral tests without requiring a big-bang replacement.
- Self-hosted static build output with no runtime CDN or new page-load network requests.

## Non-goals

- No model training, model-mechanism, API, worker, tokenizer, or engine changes.
- No changes to model artifacts, acceptance bars, M1–M8 status, or Claude's active files/worktrees.
- No new backend, hosted service, external account, analytics, or runtime third-party request.
- No production deployment or modification of `.github/**`.
- No attempt to make M1 appear to be a conversational assistant.

## Anti-goals and failure conditions

- A visual redesign that leaves contradictory model/file/session status possible.
- Replacing the current shell before the new workflows pass their behavioral and runtime checks.
- Clearing or rewriting existing browser sessions, drafts, provider settings, or credentials during migration.
- Weakening existing engine/offline checks, deleting failing assertions, or hiding unsupported behavior behind compatibility controls.
- New runtime dependencies on external assets or a required build-time service beyond npm on a developer machine.

## Constraints and preservation boundaries

- Work only in an isolated lab worktree and deliver each unit through its protected PR path.
- Foundation implementation scope is limited to `web/studio/src/**`, shell entry/config/package/test files, and two additive hooks in `web/studio/assets/js/model_sources.js`: structured progress notification and optional operation metadata for stale-load suppression. Entry-point, publication-helper, and documentation changes wait for later units.
- `studio_api.js`, `uor_model_worker.js`, `uor_tokenizer_artifact.js`, and the engine's worker/WASM boundary remain unchanged. The typed adapter calls existing public interfaces and the two new hooks; it does not scrape DOM status or duplicate the downloader, hashing, tokenizer, or persistence logic.
- `crates/**`, model artifacts, `.uor-models/**`, `.github/**`, Claude paths, and generated `pkg/` contents are out of scope.
- Existing `uor.studio.*` browser data must be migrated non-destructively. The foundation unit uses fixtures only: no real browser-data read/write, migration marker, or credentials in tests. Real copy-forward migration is a later cutover unit.
- Pin npm dependencies in the lockfile when implementation begins; do not install dependencies for this design-only PR. `node_modules/` and build output remain uncommitted.
- No production publish. Preview/cutover requires its own verified PR and the existing owner-controlled publish path.
- If `main` advances, merge it forward and re-check affected Studio behavior rather than overwriting other labs' work.

## Approaches considered

### A. Zero-dependency custom elements and a central store

Keep the current static application and introduce custom elements with idempotent setup/teardown and one lifecycle store. This minimizes build changes and dependencies, but leaves more manual component composition and type checking to conventions. It is the fallback if the owner later rejects new build dependencies.

### B. Typed Preact + Vite shell — selected

Use Preact for a small component runtime, Vite for a static self-hosted build, and TypeScript for explicit lifecycle contracts. Use a small typed event store with `useReducer`/context rather than adding a state-library dependency. Vitest + jsdom covers state transitions and components; the existing Node/Chrome harness covers real browser behavior without adding a large browser-framework dependency.

This is the best fit for a substantial upgrade because it makes lifecycle mismatches compile-time and behavior-test concerns while keeping the runtime small and the existing engine adapter intact. The cost is a new npm build toolchain and migration effort, which the owner accepted when choosing this direction.

### C. Native/shared-product shell

Build a shared web/Tauri product architecture now. Rejected for this unit because it entangles the shell with unfinished M7, desktop, and backend/API work and increases the chance of touching Claude's active model path.

## Selected architecture

### 1. Typed lifecycle store

Define explicit transitions and invariants for:

- Artifact source: `unselected → selected → downloading (bytes known/unknown) → finalizing (indeterminate) → ready | error`. Structured byte counts come only from the existing downloader through an additive public progress hook. A zero/unknown total cannot produce a percentage, and download completion is not readiness.
- Every load operation carries a monotonically increasing operation id and its source id. The optional operation metadata suppresses stale selection changes inside `model_sources.js`; the store also ignores older progress, errors, and completion after another source supersedes them.
- Selecting a new artifact immediately revokes run readiness. A previous resident artifact may remain visible for diagnostics, but it cannot receive a new run and is never presented as the selected ready artifact. A failed replacement therefore cannot silently fall back to the previous model.
- Loaded artifact: identity, schema, bytes, hash verification, engine capability, and load error. A selected source cannot display `ACTIVE` unless the existing adapter confirms that its own artifact is loaded and ready for the current operation.
- API source is separate from the artifact lifecycle: `unselected → selected (untested) → generating | error`. A valid selection permits the existing send behavior without claiming reachability or an artifact. An independent connection-test state is `untested → testing → passed | failed`; it never fabricates an artifact-ready badge and is not required merely to display selection.
- File state: `no folder → no open file → clean/dirty → saving → saved | error`. Save and prompt actions are disabled when no file is open; demo/local-draft state is labeled explicitly.
- Session state: `new → active → generating → complete | stopped | error`. Each generation pins its source and privacy classification at send time. Changing the picker cannot relabel an active generation, and a new send always uses the current selection's readiness and configuration.

All UI status derives from this store. Direct DOM status writes outside components are not part of the new architecture.

### 2. Components

- `AppShell`: one `<main>`, named navigation landmarks, a responsive drawer, and one top-level status region rather than independent status mirrors.
- `ModelPicker`: source selection, loaded artifact and capability state, and a native `<progress>` element with an accessible name for real byte totals. Unknown totals and finalization use indeterminate text, never an invented percentage or verification stage.
- `WorkspaceEditor`: folder/file selection, tabs, dirty/save state, empty-state action guards.
- `AgentComposer`: context attachments, local-predictor versus API-chat modes, generation state.
- `SessionList`: non-destructive local session operations and migration.
- `Inspector`: on-demand details rather than a permanent duplicate status pane.
- `Guide` and `Settings`: actual implemented controls only. Modal settings/help/palette surfaces use native `<dialog>` with `.showModal()`, accessible names, and native close/focus behavior rather than manual focus traps, `inert` toggling, or redundant Escape handlers.
- Dynamic errors use an appropriate live region; ordinary loading progress does not repeatedly announce interstitial text.

### 3. Engine adapter

A narrow TypeScript interface wraps the existing public engine/model-source/API calls plus the two additive model-source hooks. It does not reimplement tokenization, model loading, generation, hashing, or persistence semantics. `studio_api.js`, `uor_model_worker.js`, `uor_tokenizer_artifact.js`, and the engine remain byte-identical. `model_sources.js` keeps its existing API and behavior for current callers; only the structured progress hook and optional operation metadata are additive. Behavioral checks compare the same synthetic model trajectory, save/restore, offline boundaries, legacy calls without metadata, and new calls with superseded operations.

### 4. Static build and migration

- Pin `preact`, `vite`, `typescript`, the Preact Vite preset, `vitest`, and `jsdom` in the lockfile. No state framework or UI kit.
- Vite uses the `/studio/` base and emits self-hosted static assets. Existing generated model/tokenizer/pkg paths stay unchanged.
- Develop and verify the new shell without replacing the current page. The old page remains available during migration; no live cutover occurs mid-verification.
- Preserve existing verifiers during migration. At cutover, add new behavioral selectors/contracts and keep or move the old checks to the retained legacy page rather than deleting or weakening them.
- Migrate `uor.studio.*` data through a versioned reader with defaults, unknown-field tolerance, and no credential logging.

## Backward chain and guards

For the finished upgrade to be true:

1. The new page must produce coherent status in every reachable state.
2. Therefore each status must derive from one lifecycle store, with tests for contradictory inputs and transitions.
3. Real model and file workflows must use that same store through the existing engine adapter.
4. Therefore adapter tests must prove the real existing worker/API calls are used, not a test reimplementation.
5. The new page must be safe to cut over.
6. Therefore legacy and new checks must both pass at their respective pages, with no assertion weakened, before the entry point changes.
7. Existing user data and offline/privacy behavior must survive.
8. Therefore migration, offline-request, and privacy-mode tests must pass before publication.

Cheap checks come before expensive ones: state-transition unit tests → component tests → static build → existing worker tests → real browser workflows → accessibility/mobile/offline checks → preview/cutover review.

## Required verification

- This design-only unit runs the existing static page-load check; it claims no new browser, worker, build, or accessibility result.
- The foundation unit proves type checking and the production Vite build, with state-transition tests for: source selected but not loaded; real byte totals and unknown totals; indeterminate finalization; a second source superseding an active load and stale completion; failed replacement leaving run disabled rather than falling back to the previous model; API selection without a reachability claim; hash/load failure; no file open; dirty then saved; empty session; generation stopped/error; and source/privacy pinned at send time while the picker changes.
- Foundation adapter tests prove that existing model-source/worker/API calls are used rather than reimplemented, legacy calls without metadata are unchanged, and structured progress/operation hooks reject stale events. Fixture-backed persistence tests cover every key below, malformed/unknown fields, idempotence, rollback preservation, and no credential logging or second secret store. No real browser data is touched.
- Later component tests assert visible status, native progress/dialog behavior, accessible names, and enabled/disabled actions for those states.
- Existing worker/page checks pass at their declared legacy/new pages, including synthetic trajectory and bit-identical save/restore where applicable.
- Real-browser tests cover model loading, file editing/saving, agent modes, keyboard/focus, native-dialog focus return, mobile 390 px, and zero external page-load requests.
- Tests include negative cases; a failed build/check cannot be reported as a pass. Visual inspection covers desktop, mobile, light/dark, model loading/ready/error, and workspace empty/dirty states.
- Final review confirms no forbidden model/API/worker/Claude changes entered the diff.

## Builder-check decisions

The independent builder check identified choices that materially affected planning. They are settled as follows; these are lead decisions, not owner-approved feature removals.

1. **Full feature parity is the final cutover gate, not the first implementation unit.** The new shell must eventually retain the current command palette, six build-task templates, message copy/insert/save/retry, session rename/duplicate/export/delete, save-as/download, import-files fallback, API provider add/test/edit/remove, research section, editor preferences, and data clearing, in addition to the named components and primary workflow. No feature is silently dropped. Each gets a behavioral check or an explicit legacy-parity record before cutover.
2. **Development, legacy, and verifier paths.** Vite root is `web/studio`, source lives under `src/`, and output is the uncommitted `web/studio/dist/`. The current `index.html` remains the development legacy page until cutover. At cutover it moves to `legacy.html` as a local-only check artifact and is not published as a second application. `verify_studio_page.mjs`, `verify_studio_page_load.mjs`, `verify_studio_worker.mjs`, `verify_desktop_client.mjs`, and `verify_m7_parity.mjs` all default to `index.html` and accept `UOR_STUDIO_PAGE=legacy.html`; the runtime page-load trap maps that value to `/studio/legacy.html`. Old checks use the legacy value and new checks use the default built `index.html`, with no assertion deleted or weakened. The publisher stages `dist/*` at `/studio/` while preserving generated `pkg/` and other required assets. `scripts/studio/build-and-publish.sh` may run `npm ci && npm run build`; `.github/**` remains untouched. None of these entry-point, verifier, or publisher changes belongs to the foundation unit.
3. **One-time copy-forward persistence, with a live model-source exception.** The migration inventory is `conversations.v1`, `activeConversation.v1`, `editor.v1`, `modelSources.v1`, `settings.v1`, `ui.v2`, legacy `ui.v1`, `attachments.v1`, `tabs.v2`, `recent.v2`, and `theme`, all under `uor.studio.`. The foundation implements only a fixture-backed versioned reader with defaults and unknown-field tolerance. The later cutover unit reads the migratable keys once and writes an idempotent marker; old migratable keys remain byte-identical for rollback, with no dual-write or post-cutover synchronization promise. **`modelSources.v1` is different:** it remains the live provider-configuration/legacy compatibility key because the existing module and API fallback read and write it. It is never copied into a second secret store and may keep changing under the existing writer. The lifecycle store owns current selection through operation-guarded events rather than copying credentials or treating a legacy `active` value as proof of artifact readiness. Plaintext provider credentials are never logged or included in test fixtures.
4. **Truthful loading progress and operation ownership.** The existing downloader supplies real byte counts and totals, but its only public source-changed event is too late for progress display. The scoped additive hook forwards those structured values from inside `model_sources.js`; it does not scrape DOM text or duplicate the downloader. Unknown totals never show a percentage, local-file/bundled loads without progress events remain indeterminate, and hash/tokenizer/worker finalization has no invented stages. Optional operation metadata prevents stale completion from changing selection inside `model_sources.js`; existing callers without metadata retain their behavior, while the store independently filters stale events. Only the adapter's actual ready result can mark an artifact loaded.
5. **Pre-existing R4G1 assertion left untouched.** `crates/uor-r4-wasm-router/tests/bdd.rs` already expects option markup absent from the current page. The crate and assertion are out of scope and the failure is pre-existing. The new page will not add hidden compatibility markup. Release QA must not be reported as green; this known stale assertion is disclosed in the handoff rather than repaired or concealed.
6. **File-save evidence.** Automated browser verification uses import-files plus download-save and a stubbed directory handle for dirty/save transitions. The real File System Access picker requires a user gesture and remains a named manual/headed verification gap, not an automated pass.
7. **Explicit privacy states.** `localWASM` means prompts stay in the browser worker; `loopbackAPI` means prompts go to a service on the same device; `privateNetworkAPI` means prompts go to a LAN/private host; `remoteAPI` means prompts go to the configured remote endpoint. Loopback/private/remote classification comes from the parsed endpoint hostname, with tests for localhost, `127.0.0.1`, `::1`, private IPv4/IPv6, and public hosts. No generic "local" badge covers API traffic. This changes presentation only, not endpoint behavior.

## Builder latitude

The builder may choose component file layout, exact type names, accessible icon treatment, and design tokens. The builder may not change the phased delivery, selected state architecture, preservation boundaries, dependency class, offline/privacy requirements, full-feature-parity requirement, or cutover requirements without returning to the owner.

## Delivery decision

Deliver in small protected PRs, each with its own ORDER at the merged item base:

1. **This design/scope PR only.** Rebase, review, correct, and register the phased item. No implementation or dependency installation.
2. **Foundation.** Add the typed store, narrow engine/persistence adapters, migration reader, fixture-backed tests, and build/test configuration. Do not mount or replace the page, change the publisher, or read/write real browser data.
3. **Core workflows.** Migrate AppShell, ModelPicker, WorkspaceEditor, AgentComposer, SessionList, Inspector, Guide, and Settings against the store and adapters. Keep the legacy page live and retain all current capabilities while their parity records are completed.
4. **Parity and migration.** Finish every feature/browser-data migration and all legacy/new behavioral, worker, offline, accessibility, and mobile checks. No production publication.
5. **Cutover.** Only after full parity and fresh exact-head approval, change the entry point and publisher. Production publication remains a separate owner decision through the existing controlled path; generated roadmap status is updated only on delivered evidence.

## Mounted migration implementation addendum — 8 November 2026

The mounted-preview implementation exposed an additional constraint: a copy-only migration would leave the typed application calling legacy writers, allowing later edits to change the old rollback keys. This addendum selects the implementation mechanism under the active autonomous Phase A goal. It does not change the approved phased architecture, persistence contract, dependency class, or production boundary.

### Approaches

- **Copy-only:** rejected because it cannot preserve old keys after the first typed edit.
- **Independent typed persistence implementations:** rejected for this unit because reimplementing every session/file/settings writer would duplicate proven behavior and widen the review surface without improving the migration guarantee.
- **Early shared migration and key routing — selected:** run one transaction before any legacy storage reader/writer, then route only migratable keys to their destinations through the existing writers. This is not another application state owner. `modelSources.v1` bypasses routing and remains the only provider-credential store.

### Transaction and failure protocol

1. Typed-preview activation performs migration before the inline theme reader, conversation initialization, UI initialization, or file restoration. Default legacy activation performs no migration or routing. Final default cutover remains a later unit.
2. **Only when no valid current marker exists,** preflight reads every migratable source and destination before any write. Unknown JSON fields survive by copying the exact raw source string. Missing sources create no destination. A destination may already exist only when it is byte-identical to its source; any other conflict aborts before writes. Malformed JSON, an invalid existing marker, or a read failure also aborts before writes.
3. The live `modelSources.v1` key is excluded from destination writes and migration diagnostics. No migration report contains source values or provider credentials.
4. After successful preflight, each missing destination is written in raw-exact form and read back for equality. Existing identical destinations are not rewritten. Before the marker is written, every source is re-read and must equal its preflight raw value.
5. The marker is the versioned value `{"schema":"uor-r4.studio-lifecycle-migration/1"}`. It is written last and read back for equality. Any destination/source/marker write or verification failure removes and verifies only destinations created by this attempt, leaving pre-existing destinations and every source untouched. A rollback failure is reported generically and still prevents typed mounting.
6. Only a verified current marker enables key routing. Subsequent `getItem`, `setItem`, and `removeItem` calls for migratable keys resolve to their destinations through the existing legacy implementations. Migration failure uses identity reads but makes migratable keys read-only, so legacy initialization cannot overwrite malformed or conflicting sources; the default non-typed legacy application retains ordinary writes, while the typed mount explicitly refuses to start. The established typed error page remains non-interactive; it does not pretend the failed typed preview is a usable legacy session. No destructive reset or silent fallback to an empty typed session after a failed first migration is permitted.
7. **A valid current marker short-circuits all source preflight and copying.** From then on destinations, not sources, are authoritative and may legitimately diverge through typed edits. A missing destination means an empty or explicitly cleared domain and is never recreated from its old source; this prevents deleted data from being resurrected. No source/destination equality check runs on marked reloads.
8. Storage quota failure is fail-closed with a safe quota-specific diagnostic. No selective subset migration or projected-quota guarantee is offered because browser quotas vary. Recovery is explicit user action to free origin storage, repair malformed input, or remove a reviewed conflicting destination/marker; this unit never deletes data automatically for recovery.
9. The first migration claim is limited to single-tab activation. Source re-reads detect the fixture's intervening writes, but `localStorage` supplies no cross-tab transaction and no Web Locks dependency is added. Concurrent first activation/editing in another tab is an explicit limitation, not an atomicity claim. Ordinary post-marker routed storage retains the existing multi-tab semantics.
10. `ui.v1` is copied for archival preservation only. The current UI reader remains on the migrated `ui.v2` destination; no unverified legacy-v1 restore or format-upgrade behavior is invented.
11. Real-browser fixtures, never actual browser profiles, seed every key and prove restored workflows, old-key preservation after edits, marked reload persistence, missing-destination non-resurrection, one-shot injected quota/conflict/malformed refusal, and zero external requests. At a controlled 1440×900 viewport, light theme becomes dark, expanded navigation collapses, a new attachment and edited workspace diverge from old raws and reappear after reload, and the recent-files view renders empty after destination deletion. Fresh UI/workspace module loads also re-read divergent destinations. Actual system quota is not claimed. Later native/package and production units reuse the same reviewed runtime rather than adding another migrator.
12. Global runtime initialization is singleton/idempotent. A second evaluation cannot activate or replace the existing instance. First success remains `migrated`, a marked reload becomes `current`, and a one-shot first failure remains visible instead of being retried into a successful typed mount. Independent test factories may still exercise transaction behavior directly without replacing the global instance.
