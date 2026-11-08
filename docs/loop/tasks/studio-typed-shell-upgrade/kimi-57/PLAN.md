# Shared agent/session authority implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the existing lifecycle store the single verified owner of session identity, catalog, revision, and generation/session binding while preserving the legacy UOR session/worker implementation.

**Architecture:** Move the shared session-summary type into the lifecycle module and re-export it through the workflow adapter. A new `createAgentSessionController` wraps the existing `SessionApi` and generation adapter, verifies post-command state, and synchronizes the store; `SessionList` and `StudioApp` consume that controller. No second persistence engine is introduced.

**Tech Stack:** Existing TypeScript, Preact, Vitest/jsdom, legacy JavaScript `SessionApi`, Vite, and the dependency-free Chrome verifier.

**Spec:** `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`; bounded ORDER at `docs/loop/tasks/studio-typed-shell-upgrade/kimi-57/ORDER.md`.

## Global Constraints

- Preserve the selected Preact/Vite architecture and the existing UOR integer engine, session persistence, and public `SessionApi` behavior.
- Do not touch real browser data, add migration markers/keys, duplicate credentials, add dependencies, contact external endpoints, or publish/cut over.
- Return accepted start, not fabricated completion, from `send(prompt): boolean`. Stop and synchronize a run whose active session changes unexpectedly.
- All existing source/privacy, generation-ID, worker-reuse, legacy-page, and offline assertions remain intact.

## Review Focus

- A boolean legacy command returns success but the active ID is missing/wrong: verify the post-read and do not activate it (Tasks 1–2).
- A second session becomes active during generation: stop it and do not apply its completion to the new session (Tasks 1–2).
- Initial legacy sessions exist before the typed mount: synchronize identity/catalog without creating activity or changing storage (Tasks 1 and 4).
- A no-session first send creates a legacy conversation: reconcile the actual post-send ID rather than assuming the pre-send null ID (Task 2).
- The browser already has a real response in one session: navigating away must show the other transcript, and returning must restore the original transcript, not just the title (Task 4).

---

### Task 1: Verified lifecycle session state

**Files:**
- Modify `web/studio/src/state/lifecycle.ts` and `lifecycle.test.ts`.
- Modify `web/studio/src/adapters/workflow.ts` to re-export the shared summary type.

**Interfaces:**
- Add `SessionSummary` in `lifecycle.ts` and use `SessionState.activeId: string | null`, `revision: number`, and `items: SessionSummary[]`.
- Add `session/synchronized` with `activeId: string | null` and `items: SessionSummary[]`; add `sessionId?: string | null` to `GenerationSnapshot` and `generation/started`.
- `createGenerationAdapter` supplies `sessionId` from the legacy API. Existing null-session tests remain valid.

- [ ] Write failing tests for initial catalog/identity synchronization, revision increments only on identity change, cloned catalog entries, null empty state, generation/session pinning, and refusal to relabel a changed run.
- [ ] Run `npm test -- src/state/lifecycle.test.ts`; confirm failures identify missing state/action behavior.
- [ ] Implement the smallest state/action changes and workflow type re-export; rerun the focused suite and typecheck.

### Task 2: Agent/session controller and generation binding

**Files:**
- Create `web/studio/src/adapters/agentSession.ts` and `agentSession.test.ts`.
- Modify `web/studio/src/adapters/generation.ts` and `generation.test.ts`.

**Interfaces:**
- `createAgentSessionController(store: LifecycleStore, sessions: SessionApi): AgentSessionController`.
- Controller: `refresh(): boolean`, `create(): SessionSummary | null`, `resume(id: string): SessionSummary | null`, `rename(id, title): boolean`, `duplicate(id): SessionSummary | null`, `remove(id): SessionSummary | null`, `export(id): { name: string; content: string } | null`, `send(prompt): boolean`, `stop(): void`, `dispose(): void`.
- The controller wraps `createGenerationAdapter`; it does not reimplement generation, persistence, or storage keys.

- [ ] Write the missing-module test plus fixtures where boolean success has a missing/wrong active ID, where valid commands return the verified active summary, and where external session change occurs during a run.
- [ ] Run `npm test -- src/adapters/agentSession.test.ts src/adapters/generation.test.ts`; capture the missing module and identity/binding failures before implementation.
- [ ] Implement verified post-command reads and generation/session capture. On unexpected active-ID change, call legacy stop once, finish the pinned run as stopped, and synchronize the actual catalog/identity. Never apply that run to the new session.
- [ ] Cover initial legacy resume, first-send conversation reconciliation, invalid/duplicate terminal events, disposal, and all existing generation cases. Rerun both suites and typecheck.

### Task 3: Route composed session workflows through the authority

**Files:**
- Modify `web/studio/src/components/SessionList.tsx` and `SessionList.test.tsx`.
- Modify `web/studio/src/components/StudioApp.tsx` and `StudioApp.test.tsx`.
- Modify `web/studio/src/main.ts` only to export the new controller.

**Interfaces:**
- `SessionList` receives the controller and lifecycle store; session mutations no longer call `SessionApi` directly.
- `StudioApp` creates/disposes one controller per store/API pair, uses its generation interface for `AgentComposer`, and derives composer keys from `state.session.activeId` and `state.session.revision`.
- Session rows expose the verified active item with `aria-current="true"`; existing operation names/error callbacks remain compatible.

- [ ] Add failing composed tests proving New/Select use verified controller results, active state follows the store rather than a direct legacy read, and mismatch failure leaves phase/identity unchanged.
- [ ] Run the two component suites; confirm failures precede the wiring change.
- [ ] Replace direct New/Select/Settings generation/session dispatches with the controller, preserving rename/duplicate/export/delete callbacks and existing generation-disabled behavior. Rerun all component suites and typecheck.

### Task 4: Real browser session/create/resume/UOR response

**Files:**
- Modify `web/studio/verify_studio_page_load.mjs` for the typed built scenario only.

**Interfaces:**
- Reuse the existing typed bundled-artifact load and zero-external/runtime-error traps. Interact only with visible typed controls in the temporary Chrome profile.

- [ ] Extend the typed browser scenario to create two sessions, issue `17+25` in one, wait for the actual arithmetic answer `42`, resume the other session and confirm that answer is absent, then resume the original and confirm the answer is present. Assert active titles and one shell/MCP throughout.
- [ ] Build the real application, then run `UOR_STUDIO_TYPED=1 node verify_studio_page_load.mjs --strict-router`; diagnose instrument versus product failures without weakening checks. Label the arithmetic result as UOR protocol evidence, not useful chat.

### Task 5: Evidence and protected delivery

**Files:**
- Update only the ORDER-named product/roadmap/notebook records and this task's `REPORT.md`.

- [ ] Run focused tests, complete `npm test`, plain-JS tests, typecheck, Vite build, four legacy checks, harness report/local-PR checks, and full gates at one clean committed head. Preserve exact-head identity/diff/logs under `local/kimi/studio-agent-session/`.
- [ ] Record actual totals, browser transcript evidence, negatives, exclusions, and the fact that migration/native/package/publication remain open. Obtain a fresh exact-head review using only the item chain, ORDER, and diff.
- [ ] Only after APPROVE: push, post `uor-review v1`, keep `References #3500`, require green GitHub checks, queue through the protected path, verify the squash/fresh-main tree, rerun required fresh-main checks, and perform guarded cleanup. Do not publish production or begin Phase B.
