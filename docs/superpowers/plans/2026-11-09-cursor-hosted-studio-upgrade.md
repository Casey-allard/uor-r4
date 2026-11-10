# Cursor-hosted Studio Upgrade Implementation Plan (revised after builder check)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. **Do not start Task 1 until the owner has approved the revised [design](../specs/2026-11-09-cursor-hosted-studio-upgrade-design.md) in a new ORDER.** The lead cannot substitute a delegated approval for this gate. Before Task 1 the owner must also resolve the roadmap-item acceptance conflict: either create a new Cursor-hosted Studio item with browser-owned-runtime acceptance (recommended), with the approval ORDER naming that new item, or explicitly supersede the old Rust-runtime `Done when` in an ORDER that names `studio-ide-harness`. Every implementation task below is `BLOCKED` until both owner actions exist.

**Goal:** Upgrade the existing UOR-R4 Studio into a Cursor-grade, browser-first task workspace: one Preact/Vite shell, the R4 model local in the browser, a browser-owned task runtime and IndexedDB task log, one versioned Studio event contract, an optional host-approved companion for real tools, and a gated future service boundary.

**Architecture:** The page/Web Worker owns the task loop, reducer, cursor, queue/steer/force/stop and IndexedDB task/event persistence. A TypeScript worker-side `ModelPort` is the model boundary; executable stages use a labelled `ReplayModelPort`. The Tauri companion is optional and tool-only: it executes approved file/Git/PTY/MCP intents and holds grants/approvals, never the task loop or task store. The loopback browser uses `fetch` + `ReadableStream` NDJSON with the existing token/Origin checks; the Tauri webview uses a Channel where available and a poll fallback for the offline invoke shim. No EventSource, no SSE, no second listener. A full hosted service and real R4 generation are gated appendices, not executable tasks.

**Tech Stack:** Existing Preact 11 + Vite 8 + TypeScript 7; existing Rust 2021 workspace; `serde`, `serde_json`, `sha2`; browser IndexedDB; existing Tauri 2 command/loopback authority and `@xterm/xterm`; Playwright 1.65 as a pinned development-only browser dependency; Vitest 5. No new default runtime dependency, Electron, Svelte, node-pty, Monaco, second model API, EventSource or public endpoint.

**Spec:** [`docs/superpowers/specs/2026-11-09-cursor-hosted-studio-upgrade-design.md`](../specs/2026-11-09-cursor-hosted-studio-upgrade-design.md). Read it before Task 1. The spec is authority for every interface, field, state and acceptance condition; the [research report](../../research/reports/Cursor%20hosted%20experience.md) is background evidence only.

**PR base:** `6fcf57f7822286e5b6bd7bcfe42d668116b23afa`. **ORDER/worktree commit:** `452615bc52a0186c1c01007986ff5c1a767eb068` (not the PR base). Line anchors are from the ORDER/worktree commit; re-read a file before editing if it has moved.

---

## Global Constraints

- **Owner-only gate.** No code before **owner** approval of the revised design in a new ORDER. The lead cannot delegate this approval for this ORDER. Before Task 1 the owner must also either create a new Cursor-hosted Studio item with browser-owned-runtime acceptance (recommended), with the approval ORDER naming that new item, or explicitly supersede #3569's old Rust-runtime `Done when` in an ORDER naming `studio-ide-harness`; until then no implementation is authorized and #3569 cannot be closed by this plan. This plan does not authorize dependencies, compute, credentials, deployment or a capability claim.
- **Browser owns the loop.** The task runtime, reducer, cursor, queue/steer/force/stop, approvals and durable task/event storage live in the page/Web Worker. IndexedDB is the task/event store. There is no host-owned task runtime, host task store or host loop.
- **Companion is optional.** The Tauri companion executes approved tools and holds grants/approvals. Static browser supports the local model, task loop, transcript and user-mediated files/uploads; real terminal/Git/MCP stdio requires the companion.
- **Service is separate.** A full hosted service is a separate future owner item/security review. Do not add `services/**` code in this plan.
- **Protected selector/name block.** Preserve `nav[aria-label="Studio"]`, `textarea[aria-label="Prompt"]`, a visible `Send` button, `.agent-workspace`, `.agent-transcript`, `.agent-composer`, `.workspace-grid`, `.pane-dock`, `.session-list`, `[data-control=…]`, `[data-pane-target=…]`, `[data-action="toggle-projects"]`, and the existing single skip link. Do not create a second skip link; retarget the existing one at typed mount or hide it in typed mode. Keep baseline tests valid rather than renaming tests to fit a redesign.
- **1024 boundary exact.** The task rail collapses below 1024 and does **not auto-collapse** at exactly 1024; a user may still close it manually. Do not change the existing `AppShell` threshold to force a match; use the shell CSS/mount behavior.
- **Distinct class.** Use `.task-workspace`, not `.studio-shell`, to avoid the existing class collision.
- **Contract values (exact):** event schema `uor-r4.studio.agent-event/1`; error schema `uor-r4.studio.agent-error/1`; canonical JSON `uor-r4.canonical-json/1`; cursor prefix `uorse1`; event id `evt_<run_id>_<generation>_<sequence:012>`; `run.queued` sequence 0, `run.started` sequence 1; limits from spec §10.5 (1 MiB event, 10,000 events/run, 8 MiB snapshot, 10 MiB image, 4 MiB document, 5 attachments, 25 MiB total, 10-minute approval, 30-second stop/boundary waits, 30-second lease heartbeat).
- **Canonical JSON.** ASCII `[a-z0-9_.-]` keys sorted by ASCII code unit; non-negative safe integers only; no floats; minimal JSON string escaping; SHA-256 over canonical bytes. Different insertion order must produce the same digest in Rust and TS.
- **No `task.created`.** Every event is run-scoped with non-null `run_id`, `generation >= 1`, `sequence >= 0`. `TaskSummary` is a projection of `TaskRecord` + active run. `run.retried` is removed; `retry_of_run_id` is on `run.started`.
- **No host store as the run log.** Do not use `uor-r4-object-store` as the browser runtime's task store. Per-run keyed event records live in IndexedDB with a canonical digest chain and a verified snapshot manifest. An optional Tauri snapshot export is bounded and not authority.
- **Transport.** Loopback browser uses `fetch` POST + `fetch` NDJSON `ReadableStream` with the existing token header; never EventSource, never SSE. Tauri webview uses `agent_subscribe` Channel where available and `agent_host_poll` for the offline invoke shim. No second listener, no remote/TLS server.
- **Tauri tests.** Run from the repository root with `cargo test --manifest-path src-tauri/Cargo.toml --features gui --test <target>` (and default-feature variants where appropriate). Do not use `cargo test -p uor-r4-desktop-client` from the root workspace; `src-tauri` is a separate workspace.
- **No agent secret in `localStorage`.** The existing user-configured provider key store `uor.studio.modelSources.v1` is a documented base exception for user-entered provider keys, not an agent secret store. Do not silently migrate it.
- **Model truth.** The accepted M1 artifact is prediction-only, not the task brain. Shell/runtime tests use clearly labelled `ReplayModelPort` synthetic fixtures. Real R4 generation/useful-agent evaluation is `NOT_RUN` until a generation-capable artifact and Claude's merged streaming/tool contract exist. A prediction-only top-8 measurement is a separate appendix row.
- **No new default dependency.** No Electron, Svelte, node-pty, Monaco, CodeMirror, database, CDN, second model API, EventSource or public endpoint. A missing runtime parser/library needs a new ORDER.
- **Commits and locks.** One focused commit per task. Include root `Cargo.lock` when the workspace dependency graph changes and `src-tauri/Cargo.lock` when the Tauri crate changes. Do not commit generated build outputs, `dist`, `pkg`, `node_modules`, screenshots or credentials.
- **Run serially.** Focused tests first, then the full gates. Do not run two suites that share a target directory or browser profile at the same time.

---

## Review Focus

These are the five input classes most likely to break the product even though no single task's happy path exercises them. Each is pinned to the task that owns it.

1. **Canonical drift** — Rust and TS must produce identical event IDs/digests for the same event, independent of insertion order. Pinned in Task 1 (`canonical_json.rs`) and Task 2 (`canonical.test.ts`).
2. **IndexedDB reload/corruption** — a reload, truncated record, digest mismatch or stale cursor must recover from the verified snapshot or refuse, never silently lose or duplicate work. Pinned in Task 10 (`cursor-store.spec.ts`) and Task 15 (`acceptance.test.ts`).
3. **Hostile stream** — malformed, partial, duplicate, out-of-order, unknown, stale-generation or cross-task events must never become assistant text, a tool result or a completion. Pinned in Task 13 (`ingest.test.ts`) and Task 15 (`acceptance.test.ts`).
4. **Approval forgery / cross-task authority** — a page-provided boolean, forged approval ID, expired approval or another task's approval must not authorize a write. Pinned in Task 11 (`runtime.test.ts`), Task 17 (`agent_authority.rs`) and Task 19 (`agent_authority.rs`).
5. **Capability theater / protected-surface regression** — static web, an unconnected runtime, an unavailable companion or an unconfigured service must show an explicit unsupported state; protected selectors and the 1024 boundary must not regress. Pinned in Task 4 (`cursor-baseline.spec.ts`), Task 9 (`cursor-a11y.spec.ts`) and Task 19 (`cursor-static.spec.ts`).

---

## Task map and dependency order

| Stage | Tasks | Deliverable | Gate |
|---|---|---|---|
| 0. Contract and protected baseline | 1–4 | Rust schema/canonical reference, generated TS, client/state skeleton, protected selectors/1024 guards | Contract review |
| 1. Shell and composer | 5–9 | Task rail, transcript-first shell, composer media/attachments, send modes, a11y/IME regression | Accessibility/IME review |
| 2. Browser runtime and persistence | 10–15 | IndexedDB store, runtime state machine, host intents, parallel/reload, task UI, acceptance | Runtime/security review |
| 3. Companion data APIs and transport | 16–19 | `agent_list_directory`, `agent_git_diff`, `agent_read_artifact`, tool intents/approvals, loopback NDJSON | **Separate host-authority review** |
| 4. Review and editor | 20–22 | File tree/tabs/editor, changes snapshot/comments, PR/artifacts | Path/grant review |
| 5. Final local acceptance | 23 | Responsive/keyboard/IME/a11y, protected selectors, full gates | Final exact-head review |

Dependencies: stages run in order; tasks within a stage run in numeric order. Task 2/3 need Task 1; Task 5 needs Task 4; Task 10 needs Task 3 and starts after Stage 1; Task 16 needs Task 15; Task 20 needs Task 19; Task 23 needs all executable tasks. Stages 1 and 2 may overlap only if they touch disjoint files and the task owner confirms it; otherwise run sequentially.

**Appendices A and B are not executable tasks.** They have no checkboxes and no numbered task IDs. Appendix A is the gated service/integrations boundary. Appendix B is the gated real-R4 qualification boundary.

---

## Stage 0 — Contract and protected baseline

**Stage outcome:** a Rust-owned schema/canonical reference, generated TypeScript contract/canonicalizer, a browser `AgentClient`/reducer skeleton, and protected baseline guards. **Stage acceptance:** `cargo test -p uor-r4-studio-agent` passes contract/canonical tests; `npm test -- --run src/agent/agentContract.test.ts src/agent/canonical.test.ts src/agent/agentClient.test.ts src/state/agent.test.ts` passes; `npx playwright test tests/e2e/cursor-baseline.spec.ts` passes; the existing shell/legacy tests still pass. **No UI capability claim.**

### Task 1: Rust schema, canonical JSON and event contract

**Files:**
- Create: `crates/uor-r4-studio-agent/Cargo.toml`, `crates/uor-r4-studio-agent/src/lib.rs`, `src/ids.rs`, `src/error.rs`, `src/contract.rs`, `src/canonical.rs`
- Create: `crates/uor-r4-studio-agent/src/bin/export-contract.rs`
- Create: `crates/uor-r4-studio-agent/tests/contract_events.rs`, `tests/canonical_json.rs`, `tests/common/mod.rs`, `tests/fixtures/event-run-queued.json`, `tests/fixtures/event-run-started.json`
- Modify: root `Cargo.toml` (add workspace member only), root `Cargo.lock` (if it changes)

**Interfaces:**
- Consumes: `serde`, `serde_json`, `sha2`; no object store, no model API.
- Produces: `AGENT_EVENT_SCHEMA`, `AGENT_ERROR_SCHEMA`, `CANONICAL_JSON_SCHEMA`, `CURSOR_PREFIX`; `AgentEvent`, `AgentError`, `AgentErrorCode`, `AgentCursor`, `ApprovalRequest`, `validate_event`, `canonical_json`, `digest_event`, `schema_document`, `event_id`.

**Depends on:** owner approval. **Cost:** 2–3 focused sittings. **Acceptance:** contract/canonical tests pass; `schema_document()` is stable sorted; no runtime/store code in Rust.

- [ ] **Step 1: Write the failing contract tests.**

```rust
// tests/contract_events.rs
#[test] fn contract_schema_id_is_stable() { assert_eq!(AGENT_EVENT_SCHEMA, "uor-r4.studio.agent-event/1"); }
#[test] fn event_id_is_deterministic() {
    assert_eq!(event_id("run_7f3a2c1b9d4e6a80", 2, 1), "evt_run_7f3a2c1b9d4e6a80_2_000000000001");
}
#[test] fn event_envelope_requires_run_scope() {
    let value = serde_json::json!({"schema": AGENT_EVENT_SCHEMA, "task_id": "tsk_1"});
    assert!(validate_event(&value).is_err());
}
#[test] fn unknown_event_type_preserves_opaque_data() { /* type future.unknown, data {"x":1} */ }
#[test] fn cursor_token_round_trips() { /* encode/parse spec §9.5 */ }
#[test] fn approval_object_requires_digest_and_expiry() { /* missing digest/expiry -> err */ }
#[test] fn error_codes_include_runtime_and_host_unavailable() {
    assert_eq!(AgentErrorCode::RuntimeUnavailable.as_str(), "runtime_unavailable");
    assert_eq!(AgentErrorCode::HostUnavailable.as_str(), "host_unavailable");
}
#[test] fn fixture_run_queued_and_started_satisfy_contract() { /* seq 0/1 */ }
```

- [ ] **Step 2: Write the failing canonical tests.**

```rust
// tests/canonical_json.rs
#[test] fn keys_are_sorted_ascii() {
    let a = canonical_json(&serde_json::json!({"b": 2, "a": 1})).unwrap();
    assert_eq!(a, "{\"a\":1,\"b\":2}");
}
#[test] fn insertion_order_does_not_change_digest() {
    let a = digest_event(&event_with_order(true));
    let b = digest_event(&event_with_order(false));
    assert_eq!(a, b);
}
#[test] fn floats_are_rejected() {
    assert_eq!(canonical_json(&serde_json::json!({"x": 1.5})).unwrap_err().code(), "invalid_event");
}
#[test] fn non_ascii_keys_are_rejected() { /* {"é":1} -> invalid_event */ }
#[test] fn schema_document_is_stable_sorted() {
    let first = schema_document();
    let second = schema_document();
    assert_eq!(first, second);
    assert!(first.contains("\"run.queued\""));
}
```

- [ ] **Step 3: Run to verify failure.**

Run: `cargo test -p uor-r4-studio-agent --test contract_events --test canonical_json`
Expected: FAIL with `error[E0433]: failed to resolve: use of undeclared crate or module 'uor_r4_studio_agent'`.

- [ ] **Step 4: Implement the contract crate.**

`contract.rs` defines the run-scoped envelope, known event types from spec §9.4, `TaskSummary`/`RunState`, cursor encode/parse, approval validation and error codes (including `runtime_unavailable`, `host_unavailable`, `stop_unacknowledged`). The fixture pair is `run.queued` sequence 0 and `run.started` sequence 1; a run without both is invalid. `canonical.rs` implements the integer/ASCII-key subset exactly; `canonical_json(value) -> Result<String, AgentError>` and `digest_event(event) -> Result<String, AgentError>`. `schema_document()` builds a `serde_json::Value` and returns a **stable sorted canonical string**; `export-contract.rs` writes it. `ids.rs` uses `sha2` over a monotonic counter + process nonce for `prj_/tsk_/run_/msg_/tc_/apr_/art_` IDs. No `unwrap`/`expect` on recoverable paths. `tests/common/mod.rs` provides `event(sequence)`, `event_for(task_id, sequence)`, `last_event(sequence)`, `event_with_order(bool)`.

- [ ] **Step 5: Run the tests.**

Run: `cargo test -p uor-r4-studio-agent --test contract_events --test canonical_json`
Expected: PASS (8 contract + 5 canonical tests). Include root `Cargo.lock` if it changed.

- [ ] **Step 6: Commit.**

```bash
git add Cargo.toml Cargo.lock crates/uor-r4-studio-agent
git commit -m "feat(studio-agent): add run-scoped contract and canonical JSON reference"
```

**Stop if:** canonicalization needs a third-party JCS library, or the schema needs a `task.created` event. Re-scope.

---

### Task 2: Export schema and generate TypeScript contract/canonicalizer

**Files:**
- Create: `scripts/studio/generate-agent-contract.mjs`
- Create: `crates/uor-r4-studio-agent/contract.schema.json` (checked in; digest-test input)
- Create: `web/studio/src/agent/generated/agentContract.ts`, `web/studio/src/agent/generated/agentCanonical.ts`
- Create: `web/studio/src/agent/agentContract.test.ts`, `web/studio/src/agent/canonical.test.ts`
- Modify: `web/studio/package.json` (add a `contract:agent` script only), root `Cargo.lock` if changed

**Interfaces:**
- Consumes: `schema_document()` from Task 1.
- Produces: `AGENT_EVENT_SCHEMA`, `AGENT_ERROR_SCHEMA`, `CANONICAL_JSON_SCHEMA`, `AGENT_CONTRACT_DIGEST`; `AgentEvent`, `AgentEventDataMap`, `KnownAgentEventType`, `AgentError`, `AgentCursor`, `ApprovalRequest`; `canonicalJson(value: unknown): string`, `digestEvent(event: AgentEvent): Promise<string>`.

**Depends on:** Task 1. **Cost:** 1–2 sittings. **Acceptance:** generated files are deterministic; TS canonicalizer matches Rust on insertion order; generated types include `run.queued`, `run.started` with `retry_of_run_id`, and `runtime_unavailable`.

- [ ] **Step 1: Write the failing generation/canonical tests.**

```ts
// web/studio/src/agent/agentContract.test.ts
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { AGENT_CONTRACT_DIGEST, AGENT_EVENT_SCHEMA, type AgentEventDataMap } from './generated/agentContract.js';
import { canonicalJson, digestEvent } from './generated/agentCanonical.js';

test('agent_contract_is_generated_from_schema', () => {
  expect(AGENT_EVENT_SCHEMA).toBe('uor-r4.studio.agent-event/1');
  expect(AGENT_CONTRACT_DIGEST).toMatch(/^[0-9a-f]{64}$/);
});
test('agent_contract_digest_matches_schema_file', async () => {
  const bytes = await readFile(new URL('../../../../crates/uor-r4-studio-agent/contract.schema.json', import.meta.url));
  const digest = createHash('sha256').update(bytes).digest('hex');
  expect(AGENT_CONTRACT_DIGEST).toBe(digest);
});
test('generated_types_include_sequence_zero_and_retry_field', () => {
  const queued: AgentEventDataMap['run.queued'] = { queued_at_ms: 1 };
  const started: AgentEventDataMap['run.started'] = {
    started_at_ms: 1, model: { id: 'replay', kind: 'fixture' }, mode: 'agent',
    environment: { kind: 'local-browser' }, retry_of_run_id: null
  };
  expect(queued.queued_at_ms).toBe(1);
  expect(started.retry_of_run_id).toBeNull();
});
test('canonical_json_sorts_keys', () => {
  expect(canonicalJson({ b: 2, a: 1 })).toBe('{"a":1,"b":2}');
});
test('different_insertion_order_same_digest', async () => {
  expect(await digestEvent(eventA)).toBe(await digestEvent(eventB));
});
test('floats_are_rejected', () => {
  expect(() => canonicalJson({ x: 1.5 })).toThrow(/invalid_event/);
});
```

- [ ] **Step 2: Run to verify failure.**

Run: `npm test -- --run src/agent/agentContract.test.ts src/agent/canonical.test.ts`
Expected: FAIL with `Cannot find module './generated/agentContract.js'`.

- [ ] **Step 3: Implement the generator and script.**

`generate-agent-contract.mjs` resolves the repo root from `import.meta.url` (not CWD), reads `crates/uor-r4-studio-agent/contract.schema.json` (JSON Schema 2020-12), validates the schema id, and writes:
- `agentContract.ts`: schema constants, `KnownAgentEventType`, `AgentEventDataMap`, `AgentEvent`, `AgentError`, `AgentCursor`, `ApprovalRequest`, `AGENT_CONTRACT_DIGEST = sha256(schema bytes)`.
- `agentCanonical.ts`: `canonicalJson` and `digestEvent` implementing spec §9.3 (ASCII keys sorted, safe integers only, no floats, minimal escaping, SHA-256 via `crypto.subtle` or a small synchronous test helper). The browser path may use `crypto.subtle.digest`; tests can use a deterministic Node helper.
Add `"contract:agent": "node ../../scripts/studio/generate-agent-contract.mjs"`. The script resolves the repo root and the schema/generator paths from `import.meta.url`, invokes `cargo run --manifest-path <repo>/Cargo.toml -p uor-r4-studio-agent --bin export-contract -- --write <repo>/crates/uor-r4-studio-agent/contract.schema.json`, and writes the generated files under `<repo>/web/studio/src/agent/generated/`. Do not depend on the caller's CWD.

- [ ] **Step 4: Run the generator and tests.**

Run: `npm run contract:agent && npm test -- --run src/agent/agentContract.test.ts src/agent/canonical.test.ts && npm run typecheck`
Expected: PASS. A second generator run must leave `git diff --exit-code` empty.

- [ ] **Step 5: Commit.**

```bash
git add scripts/studio/generate-agent-contract.mjs crates/uor-r4-studio-agent/contract.schema.json web/studio/src/agent/generated web/studio/src/agent/agentContract.test.ts web/studio/src/agent/canonical.test.ts web/studio/package.json Cargo.lock
git commit -m "feat(studio): generate the TypeScript contract and canonicalizer"
```

**Stop if:** the generator needs a schema library or crypto dependency not already present. Declare it in a new ORDER; do not hand-fork the event union.

---

### Task 3: Browser `AgentClient`, fixture transport and reducer skeleton

**Files:**
- Create: `web/studio/src/agent/client.ts`, `web/studio/src/agent/fixtureTransport.ts`, `web/studio/src/state/agent.ts`
- Create: `web/studio/src/agent/agentClient.test.ts`, `web/studio/src/state/agent.test.ts`
- Create: `web/studio/src/agent/fixtures.ts`

**Interfaces:**
- Consumes: `AgentEvent`, `AgentCursor`, `ApprovalRequest`, `canonicalJson` from Task 2.
- Produces: `AgentClient` (`listTasks`, `createTask`, `startRun`, `queueMessage`, `steer`, `forceSend`, `pinTask`, `archiveTask`, `stop`, `retry`, `resolveApproval`, `resumeTask`, `subscribe`, `tree`, `changesSnapshot`, `readArtifact`, `hostCapabilities`); `FixtureTransport`; `AgentStore` (`apply(event)` synchronous, `snapshot()`, `selectTask(id)`); `TaskView`, `RunView`.

**Depends on:** Task 2. **Cost:** 1–2 sittings. **Acceptance:** reducer rejects gaps/stale/cross-task, tolerates unknown events, treats `run.status` as projection, and never sends a boolean approval.

- [ ] **Step 1: Write the failing client/reducer tests.**

```ts
// web/studio/src/state/agent.test.ts
test('run_queued_is_sequence_zero', () => { /* apply run.queued seq0; task state queued */ });
test('terminal_events_are_authoritative_over_run_status', () => {
  const store = createAgentStore();
  store.apply(event(0, 'run.queued')); store.apply(event(1, 'run.started'));
  store.apply(event(2, 'run.status', { status: 'finished' }));
  expect(store.snapshot().runs[0].state).not.toBe('finished');
  store.apply(event(3, 'run.finished', { finished_at_ms: 2, duration_ms: 1 }));
  expect(store.snapshot().runs[0].state).toBe('finished');
});
test('reducer_ignores_duplicate_event', ...);
test('reducer_flags_gap_and_requires_resync', ...);
test('reducer_does_not_render_unknown_event_as_message', ...);
test('reducer_marks_stale_generation', ...);
test('reducer_rejects_cross_task_event', ...);
// web/studio/src/agent/agentClient.test.ts
test('unavailable_client_reports_runtime_unavailable', ...);
test('resolve_approval_sends_only_id_and_decision', ...);
```

- [ ] **Step 2: Run to verify failure.**

Run: `npm test -- --run src/agent/agentClient.test.ts src/state/agent.test.ts`
Expected: FAIL with `Cannot find module './client.js'`.

- [ ] **Step 3: Implement the client and reducer.**

`AgentClient` is the browser API; `FixtureTransport` emits labelled synthetic events (no timers, no hidden fetch). `AgentStore.apply(event)` is **synchronous and pure**; async canonicalization/digest/store append is a separate `appendEvent` path awaited by tests. `AgentStore` keys runs by `[task_id, run_id]`, tracks `last_sequence`/`last_event_id`, applies only the current generation, and treats `run.queued`/`run.started`/terminal events as authoritative; `run.status` only updates a projection. `resolveApproval` signature is `{ approvalId, decision }`; no boolean. `tree`/`changesSnapshot`/`readArtifact` read local uploaded files/IndexedDB first; host-only branches throw `host_unavailable` until Stage 3/4. `fixtures.ts` builds sequence-0 `run.queued` and sequence-1 `run.started` events.

- [ ] **Step 4: Run tests/typecheck.**

Run: `npm test -- --run src/agent/agentClient.test.ts src/state/agent.test.ts && npm run typecheck`
Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add web/studio/src/agent/client.ts web/studio/src/agent/fixtureTransport.ts web/studio/src/agent/fixtures.ts web/studio/src/state/agent.ts web/studio/src/agent/agentClient.test.ts web/studio/src/state/agent.test.ts
git commit -m "feat(studio): add browser agent client and reducer skeleton"
```

**Stop if:** the client needs a direct `desktop_actions` call or the reducer needs host authority. Both violate the browser-owned loop.

---

### Task 4: Protected selectors, skip-link retarget and 1024 baseline

**Files:**
- Create: `web/studio/tests/e2e/cursor-baseline.spec.ts`
- Create: `web/studio/src/agent/legacyShortcutGuard.ts` (protected-baseline guard; Task 8 may extend)
- Modify: `web/studio/src/bootstrap.ts` (install/close/cancel/clear before typed workflow wait/render; retarget the existing skip link)
- Modify: `web/studio/src/components/AppShell.tsx`/`AppShell.test.tsx` (children prop test fix only)
- Modify: `web/studio/tests/e2e/support.ts` (helpers only)

**Interfaces:**
- Consumes: existing `bootstrap.ts`, `vite.config.ts`, `index.html`, `typed_entry.js`.
- Produces: a browser guard for the protected selector/name block, one skip link, and the exact 1024 boundary.

**Depends on:** Task 3. **Cost:** 2–3 sittings. **Acceptance:** baseline selectors pass; exactly one visible skip link; at 1024 the rail does not auto-collapse (a user may close it manually); at 1023 a closed drawer hides the rail and `[data-action="toggle-projects"]` reopens it; the delayed legacy `setTimeout(150)` palette cannot open; `?legacy=1`/`?typed=0` unchanged.

- [ ] **Step 1: Write the failing baseline tests.**

```ts
// cursor-baseline.spec.ts
test('protected_selectors_are_preserved', async ({ page }) => {
  await page.goto('/');
  for (const selector of ['nav[aria-label="Studio"]', 'textarea[aria-label="Prompt"]',
    '.agent-workspace', '.agent-transcript', '.agent-composer', '.workspace-grid',
    '.pane-dock', '.session-list', '[data-control="environment"]',
    '[data-pane-target]', '[data-action="toggle-projects"]']) {
    await expect(page.locator(selector).first()).toBeAttached();
  }
  await expect(page.getByRole('button', { name: 'Send' })).toBeVisible();
});
test('only_one_skip_link_and_it_targets_active_main', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('a.skip-link')).toHaveCount(1);
  const href = await page.locator('a.skip-link').getAttribute('href');
  await expect(page.locator(href!)).toBeVisible();
});
test('1024_does_not_auto_collapse', async ({ page }) => {
  await page.setViewportSize({ width: 1024, height: 900 }); await page.goto('/');
  await expect(page.locator('[data-region="task-rail"]')).toBeVisible();
});
test('1023_closed_drawer_hides_and_toggle_reopens', async ({ page }) => {
  await page.setViewportSize({ width: 1023, height: 900 }); await page.goto('/');
  const rail = page.locator('[data-region="task-rail"]');
  if (await rail.isVisible()) await page.locator('[data-action="toggle-projects"]').click();
  await expect(rail).toBeHidden();
  await page.locator('[data-action="toggle-projects"]').click();
  await expect(rail).toBeVisible();
});
test('typed_mount_closes_existing_legacy_palette', async ({ page }) => { /* seed #commandPalette open; typed mount closes it */ });
test('typed_mount_clears_palette_hashes', async ({ page }) => { /* #command-palette/#palette -> cleared */ });
test('legacy_palette_delayed_open_cannot_beat_guard', async ({ page }) => { /* legacy setTimeout(150) open stays closed */ });
test('legacy_query_keeps_app_visible_and_typed_root_hidden', async ({ page }) => { /* ?legacy=1 */ });
```

- [ ] **Step 2: Run to verify failure.**

Run: `npm run build && npx playwright test tests/e2e/cursor-baseline.spec.ts`
Expected: FAIL on protected selectors, duplicate skip link or the 1024 assertion because the current shell uses different hooks/threshold behavior.

- [ ] **Step 3: Implement only the guards/hooks.**

Add `data-region="task-rail"`/`"task-detail"` hooks without renaming existing selectors. Create `legacyShortcutGuard.ts` with `installLegacyShortcutGuard(root)`. In `bootstrap.ts`, install it **before** `waitForWorkflowApis`/`render(StudioApp, root)` and before any legacy listener or delayed `setTimeout(150)` can fire; at mount it closes/cancels any existing legacy palette, clears `#command-palette`/`#palette` from the URL/hash, and suppresses later opens. Retarget the existing `a.skip-link` to `#harness-main`; do not append a new skip link. Keep `AppShell` threshold as-is; at 1023 the rail is hidden only while the drawer is closed, and `[data-action="toggle-projects"]` reopens it; exactly 1024 does not auto-collapse (manual close remains possible). Fix the `AppShell.test.tsx` `children` prop so the test compiles. Do not change `vite.config.ts` or `typed_entry.js`.

- [ ] **Step 4: Run the baseline and legacy suites.**

Run: `npm run build && npx playwright test tests/e2e/cursor-baseline.spec.ts tests/e2e/shell.spec.ts tests/e2e/workflows.spec.ts`
Expected: PASS; existing checks remain green.

- [ ] **Step 5: Commit.**

```bash
git add web/studio/tests/e2e/cursor-baseline.spec.ts web/studio/tests/e2e/support.ts web/studio/src/agent/legacyShortcutGuard.ts web/studio/src/bootstrap.ts web/studio/src/components/AppShell.tsx web/studio/src/components/AppShell.test.tsx web/studio/index.html
git commit -m "test(studio): pin protected selectors, skip link and 1024 boundary"
```

**Stop if:** a fix requires renaming a protected selector, changing the AppShell threshold, or adding a second skip link. Stop and re-scope.

---

## Stage 1 — Shell and composer

**Stage outcome:** the existing shell becomes a task workspace in presentation, with a task rail, transcript-first detail, media/attachment composer and scoped send modes. **Stage acceptance:** `cursor-shell.spec.ts`, `cursor-keyboard.spec.ts`, `cursor-ime.spec.ts`, `cursor-a11y.spec.ts` pass; protected selectors remain; the existing 47 Playwright checks still pass. **No model or runtime capability claim.**

### Task 5: UOR tokens and `.task-workspace` shell

**Files:**
- Create: `web/studio/src/styles/tokens.css`, `shell.css`, `responsive.css`
- Create: `web/studio/src/components/ProjectHome.tsx`, `ProjectHome.test.tsx`
- Modify: `web/studio/src/studio.css`, `AppShell.tsx`, `SessionToolbar.tsx`, `ProjectSidebar.tsx`, `StudioApp.tsx`
- Test: `AppShell.test.tsx`, `ProjectHome.test.tsx`, `cursor-shell.spec.ts`

**Interfaces:**
- Consumes: protected `ShellStatus`/`StudioSection` slots.
- Produces: `.task-workspace` layout with `data-region="task-rail"`, `"task-detail"`, `"work-surface"`; UOR tokens; one `h1` per view.

**Depends on:** Task 4. **Cost:** 1–2 sittings. **Acceptance:** UOR tokens; no green placeholder; protected class names unchanged; `.task-workspace` is distinct; 1440 rail + detail.

- [ ] **Step 1: Write the failing shell tests.**

```tsx
test('renders_task_rail_and_detail_regions', ...);
test('does_not_render_green_placeholder_token', ...);
test('project_home_shows_project_name_environment_and_empty_state', ...);
test('task_workspace_class_is_distinct_from_studio_shell', ...);
```

- [ ] **Step 2: Run to verify failure.**

Run: `npm test -- --run src/components/AppShell.test.tsx src/components/ProjectHome.test.tsx`
Expected: FAIL on the missing `data-region`/`.task-workspace` hooks.

- [ ] **Step 3: Implement tokens and shell.**

Use spec §11.8 tokens. `.task-workspace` is the shell grid; keep the existing `.studio-shell` class for compatibility and add `.task-workspace` for new rules rather than redefining `.studio-shell`. Keep `.agent-workspace`, `.agent-transcript`, `.agent-composer`, `.workspace-grid`, `.pane-dock`, `.session-list` intact. `ProjectHome` renders project name, environment and an explicit "No tasks yet" empty state; it does not invent a project or memory. Keep `nav[aria-label="Studio"]` and `[data-action="toggle-projects"]`.

- [ ] **Step 4: Run component/typecheck/build.**

Run: `npm test -- --run src/components/AppShell.test.tsx src/components/ProjectHome.test.tsx && npm run typecheck && npm run build`
Expected: PASS.

- [ ] **Step 5: Add e2e layout assertions.**

```ts
test('renders_task_rail_and_transcript_first_detail_at_1440', ...);
test('390_has_no_horizontal_overflow', ...);
```

- [ ] **Step 6: Commit.**

```bash
git add web/studio/src/styles web/studio/src/studio.css web/studio/src/components/AppShell.tsx web/studio/src/components/SessionToolbar.tsx web/studio/src/components/ProjectSidebar.tsx web/studio/src/components/StudioApp.tsx web/studio/src/components/ProjectHome.tsx web/studio/src/components/ProjectHome.test.tsx web/studio/src/components/AppShell.test.tsx web/studio/tests/e2e/cursor-shell.spec.ts
git commit -m "feat(studio): add UOR task-workspace shell without protected-selector regressions"
```

**Stop if:** the shell change renames `.agent-*`/`.workspace-grid` or changes the AppShell threshold.

---

### Task 6: Task rail with search, local pins and groups

**Files:**
- Create: `web/studio/src/components/TaskRail.tsx`, `TaskSearch.tsx`, `TaskGroup.tsx`, `TaskRail.test.tsx`
- Modify: `ProjectSidebar.tsx`, `StudioApp.tsx`, `state/agent.ts` (task summaries only)
- Modify: `cursor-shell.spec.ts`

**Interfaces:**
- Consumes: `AgentClient.listTasks()`, `pinTask(taskId, pinned)`, `archiveTask(taskId, archived)`/`TaskSummary` from Task 3.
- Produces: `TaskRail` props `{ tasks, query, onQuery, onSelect, onPin, onArchive, selectedTaskId }`.

**Depends on:** Task 5. **Cost:** 1–2 sittings. **Acceptance:** Today/Last 7 days/Older; search; pins labelled "This device"; archive reversible; empty state distinguishes `runtime_unavailable`.

- [ ] **Step 1: Write failing tests** (`renders_today_last_seven_days_older_groups`, `search_filters_tasks_by_title_and_project`, `pins_are_local_device_only_without_service`, `pin_task_calls_agent_client`, `archive_task_calls_agent_client_and_reopens`, `empty_state_explains_runtime_unavailable`).
- [ ] **Step 2: Run** `npm test -- --run src/components/TaskRail.test.tsx`; expected FAIL with missing module.
- [ ] **Step 3: Implement** the rail with `nav aria-label="Tasks"`, local pin label "This device", archived collapsed, no fake rows.
- [ ] **Step 4: Add e2e rail assertion** and run `npm run build && npx playwright test tests/e2e/cursor-shell.spec.ts`.
- [ ] **Step 5: Commit** `feat(studio): add task rail with search, local pins and groups`.

**Stop if:** the rail needs to invent tasks or claim account-scoped pins before a service exists.

---

### Task 7: Composer media/attachments reusing `createComposerInput`

**Files:**
- Create: `web/studio/src/components/ComposerAttachments.tsx`, `ComposerMenu.tsx`, `EnvironmentPicker.tsx`, `PermissionPicker.tsx`, `ComposerSurface.test.tsx`
- Modify: `web/studio/src/components/AgentComposer.tsx` (render the new attachment/menu pieces; keep `textarea[aria-label="Prompt"]` and visible `Send`), `web/studio/src/adapters/composerInput.ts` only to export a missing reader hook
- Modify: `web/studio/src/agent/fixtures.ts` (media fixture only)
- Test: `ComposerSurface.test.tsx`, `cursor-ime.spec.ts` (Stage 1/Task 9)

**Interfaces:**
- Consumes: existing `createComposerInput`, `readTextFiles`; `AgentClient` send methods.
- Produces: `AttachmentDraft { id, name, kind: 'image' | 'document' | 'text', bytes, mediaType }`; `ComposerSurface` props `{ value, onChange, attachments, onAttach, onRemove, onSend, onQueue, onSteer, onForce, onStop, mode, onMode, model, onModel, environment, onEnvironment, disabled, runtimeAvailable }`.

**Depends on:** Task 6. **Cost:** 2–3 sittings. **Acceptance:** existing IME/composition behavior is regression-accepted; new image/document reader; `aria-label="Prompt"`; visible `Send`; `Shift+Enter` newline; exact size/type errors; no image in `localStorage`.

- [ ] **Step 1: Write failing/new tests.**

```tsx
test('existing_ime_composition_regression', () => { /* current createComposerInput behavior; expect PASS at base */ });
test('shift_enter_inserts_newline_at_caret', ...); // new
test('paste_image_creates_memory_only_attachment', ...); // new
test('document_over_limit_reports_exact_size', ...); // new
test('at_menu_lists_editor_file_and_attachments_only', ...); // new
test('permission_picker_is_disabled_without_host', ...); // new
test('visible_send_button_and_prompt_label_are_preserved', ...); // protected
```

- [ ] **Step 2: Run to verify the new cases fail.**

Run: `npm test -- --run src/components/ComposerSurface.test.tsx`
Expected: NEW cases FAIL with missing `ComposerAttachments`/image handler; `existing_ime_composition_regression` passes at base and must keep passing. Do not claim a red IME test.

- [ ] **Step 3: Implement the media reader and menus.**

Reuse `createComposerInput` for Enter/IME/paste/history/caret. Add image/document validation and memory-only attachment chips; `URL.createObjectURL` for image preview, revoke on remove; no `localStorage`. Keep `textarea[aria-label="Prompt"]` and the visible `Send` button. `@` lists editor file, attachments and artifacts only; `/` lists local templates. `PermissionPicker` disables autonomous modes without a host.

- [ ] **Step 4: Run tests/typecheck.**

Run: `npm test -- --run src/components/ComposerSurface.test.tsx && npm run typecheck`
Expected: PASS; existing IME regression remains green.

- [ ] **Step 5: Commit.**

```bash
git add web/studio/src/components/ComposerAttachments.tsx web/studio/src/components/ComposerMenu.tsx web/studio/src/components/EnvironmentPicker.tsx web/studio/src/components/PermissionPicker.tsx web/studio/src/components/ComposerSurface.test.tsx web/studio/src/components/AgentComposer.tsx web/studio/src/adapters/composerInput.ts web/studio/src/agent/fixtures.ts
git commit -m "feat(studio): add media attachments and menus without rewriting composer input"
```

**Stop if:** the composer rewrite removes `createComposerInput` or changes `aria-label="Prompt"`/visible `Send`.

---

### Task 8: Send modes, scoped shortcuts and legacy palette suppression

**Files:**
- Create: `web/studio/src/components/SendModeControls.tsx`, `web/studio/src/hooks/useScopedShortcuts.ts`, `useScopedShortcuts.test.tsx`
- Modify: `web/studio/src/agent/legacyShortcutGuard.ts` (extend the Task 4 guard; do not recreate it), `AgentComposer.tsx`, `StudioApp.tsx`, `bootstrap.ts`, `client.ts`
- Test: `cursor-keyboard.spec.ts`

**Interfaces:**
- Consumes: `ComposerSurface`, `AgentClient`.
- Produces: `SendModeControls` props `{ activeRun, onQueue, onSteer, onForce, onStop }`; `useScopedShortcuts({ onPalette, onSettings })`; extends `installLegacyShortcutGuard` from Task 4.

**Depends on:** Task 7. **Cost:** 1–2 sittings. **Acceptance:** no global `Tab`/`Cmd+K`; palette button primary; `Cmd/Ctrl+Shift+P` is a scoped UOR choice; `#command-palette`/`#palette` hash-open is suppressed in typed mode; hidden legacy palette never opens.

- [ ] **Step 1: Write failing keyboard tests.**

```ts
test('cmd_k_does_not_open_hidden_legacy_palette', ...);
test('typed_mode_suppresses_hash_palette', async ({ page }) => {
  await page.goto('/#command-palette');
  await expect(page.locator('#commandPalette')).toBeHidden();
  await page.goto('/#palette');
  await expect(page.locator('#commandPalette')).toBeHidden();
});
test('tab_moves_focus_normally_outside_composer', ...);
test('escape_returns_focus_to_composer', ...);
```

- [ ] **Step 2: Run to verify failure.**

Run: `npm run build && npx playwright test tests/e2e/cursor-keyboard.spec.ts`
Expected: FAIL on the hidden legacy palette/hash routes.

- [ ] **Step 3: Implement the guard and shortcuts.**

Extend `installLegacyShortcutGuard` from Task 4; it already runs in capture phase before legacy handlers, closes any existing palette at mount, clears `#command-palette`/`#palette`, and suppresses `Meta/Ctrl+K` plus delayed legacy opens in typed mode only. Do not create a second guard or duplicate listeners. `useScopedShortcuts` ignores text fields and handles only `Cmd/Ctrl+Shift+P` and `Cmd/Ctrl+,`; no `Tab` handler. `SendModeControls` keeps `Send` visible and adds Queue/Steer/Force/Stop with distinct labels; all disabled without a runtime.

- [ ] **Step 4: Run tests and commit.**

Run: `npm run build && npx playwright test tests/e2e/cursor-keyboard.spec.ts && npm test -- --run src/components/ComposerSurface.test.tsx`
Expected: PASS.

```bash
git add web/studio/src/components/SendModeControls.tsx web/studio/src/hooks/useScopedShortcuts.ts web/studio/src/agent/legacyShortcutGuard.ts web/studio/src/hooks/useScopedShortcuts.test.tsx web/studio/src/components/AgentComposer.tsx web/studio/src/components/StudioApp.tsx web/studio/src/bootstrap.ts web/studio/src/agent/client.ts web/studio/tests/e2e/cursor-keyboard.spec.ts
git commit -m "feat(studio): add scoped send modes and suppress legacy palette routes"
```

**Stop if:** the guard changes legacy behavior under `?legacy=1` or requires editing `studio_ui.js`.

---

### Task 9: Accessibility, responsive and IME regression acceptance

**Files:**
- Create: `web/studio/tests/e2e/cursor-ime.spec.ts`, `web/studio/tests/e2e/cursor-a11y.spec.ts`
- Modify: `responsive.css`, `ComposerSurface.tsx`, `SessionToolbar.tsx` (focus/targets only)
- Test: the two new spec files plus `visual.spec.ts`

**Interfaces:** acceptance only.

**Depends on:** Task 8. **Cost:** 1–2 sittings. **Acceptance:** 390/1024/1440; 1024 does not auto-collapse, 1023 closes/reopens; 24/44 px targets; focus return; reduced motion; existing IME regression plus new dead-key/RTL/long-paste/caret cases.

- [ ] **Step 1: Write the IME/a11y tests.**

```ts
test('existing_ime_composition_regression', ...); // must pass at base
test('dead_key_composition_preserves_text', ...); // new
test('rtl_paste_preserves_caret', ...); // new
test('long_paste_preserves_caret_and_scroll', ...); // new
test('every_interactive_target_is_at_least_24_css_pixels', ...);
test('coarse_pointer_targets_are_at_least_44_css_pixels', ...); // hasTouch context
test('headings_are_in_order', ...); // visible headings only
test('reduced_motion_disables_nonessential_animation', ...);
test('focus_ring_is_visible_in_both_themes', ...);
test('1024_boundary_is_exact', ...);
```

- [ ] **Step 2: Run to verify only the new cases fail.**

Run: `npm run build && npx playwright test tests/e2e/cursor-ime.spec.ts tests/e2e/cursor-a11y.spec.ts`
Expected: existing IME regression passes. New dead-key/RTL/long-paste/caret and target/focus cases either pass as regression or fail where current behavior is insufficient; record the result honestly and implement only the failing cases. Do not invent a red IME result.

- [ ] **Step 3: Implement the missing CSS/behavior only.**

Add target sizing, reduced-motion, heading-order/focus fixes in the typed shell. Keep 16 px mobile form text. Do not restyle legacy `#app`. Do not add global keyboard handlers.

- [ ] **Step 4: Run all Stage 1 tests and commit.**

Run: `npm run build && npx playwright test tests/e2e/cursor-shell.spec.ts tests/e2e/cursor-keyboard.spec.ts tests/e2e/cursor-ime.spec.ts tests/e2e/cursor-a11y.spec.ts && npm test -- --run src/components`
Expected: PASS.

```bash
git add web/studio/tests/e2e/cursor-ime.spec.ts web/studio/tests/e2e/cursor-a11y.spec.ts web/studio/src/styles/responsive.css web/studio/src/components/ComposerSurface.tsx web/studio/src/components/SessionToolbar.tsx
git commit -m "test(studio): accept responsive, keyboard, IME regression and accessibility"
```

**Stop if:** a fix requires changing the legacy page, a protected selector or a global shortcut.

---

## Stage 2 — Browser runtime and persistence

**Stage outcome:** the browser owns one task with many runs, a canonical IndexedDB event log, queue/steer/force/stop/retry, approvals, two live tasks and a task detail that renders real events without fabrication. **Stage acceptance:** `npm test -- --run src/agent` passes; `npx playwright test tests/e2e/cursor-store.spec.ts tests/e2e/cursor-tasks.spec.ts` passes with real IndexedDB reload; the task detail uses `ReplayModelPort` fixtures only. **No real model claim.**

### Task 10: IndexedDB store, canonical records, digest chain and lease

**Files:**
- Create: `web/studio/src/agent/store.ts`, `web/studio/src/agent/storeTypes.ts`
- Create: `web/studio/tests/e2e/cursor-store.spec.ts`
- Modify: `web/studio/src/agent/client.ts` (store wiring), `web/studio/playwright.config.ts` (webServer `VITE_STUDIO_TEST=1 npm run build`)
- Test: `cursor-store.spec.ts` (real browser IndexedDB; no fake-indexeddb dependency)

**Interfaces:**
- Consumes: `canonicalJson`, `digestEvent`, `AgentEvent`.
- Produces: `AgentTaskStore` (`open`, `createTask`, `appendEvent` async, `readRun`, `writeSnapshot`, `readSnapshot`, `listTasks`, `pinTask`, `archiveTask`, `recordChanges`, `readChanges`, `acquireLease`, `releaseLease`, `exportNdjson`, `importNdjson`); `TaskRecord`, `RunRecord`, `EventRecord`, `SnapshotRecord`, `ChangeRecord`, `LeaseRecord`.

**Depends on:** Task 3 and Stage 1. **Cost:** 3–4 sittings. **Acceptance:** per-run keyed records; save/reload preserves cursor; digest chain; truncated/corrupt snapshot refused; lease uses heartbeat; no object-store whole-run log.

- [ ] **Step 1: Write the failing browser tests.**

```ts
// cursor-store.spec.ts
test('indexeddb_save_reload_preserves_cursor', async ({ page }) => {
  await page.goto('/?studioTest=1'); await page.evaluate(() => window.__studioTest.createTaskAndEvents());
  await page.reload();
  expect(await page.evaluate(() => window.__studioTest.cursor())).toEqual({ sequence: 2, eventId: 'evt_...' });
});
test('indexeddb_corrupt_snapshot_is_refused', ...);
test('indexeddb_truncated_event_record_is_refused', ...);
test('indexeddb_lease_uses_heartbeat_not_started_at', ...);
test('indexeddb_per_run_records_are_keyed', ...);
test('indexeddb_changes_store_records_snapshot', ...);
test('pin_and_archive_are_reversible_in_store', ...);
```

- [ ] **Step 2: Run to verify failure.**

Run: `npm run build && npx playwright test tests/e2e/cursor-store.spec.ts`
Expected: FAIL with `window.__studioTest`/store undefined.

- [ ] **Step 3: Implement the store.**

Create DB `uor-r4.studio.agent/1` with stores from spec §10.4. `appendEvent` writes `{event_id, canonical, digest, prev_digest}` in one readwrite transaction with the run's `last_sequence`/`last_event_id`/`last_digest`. `writeSnapshot` stores `head_digest`; `readSnapshot` verifies the chain and refuses `snapshot_corrupt`/`snapshot_missing`. Lease: `started_at_ms` set once; `heartbeat_at_ms` refreshed; stale if `now - heartbeat_at_ms > 30_000`; recovery verifies `head_digest`. `exportNdjson`/`importNdjson` are bounded; import verifies every digest and chain. Add the `changes` store and `pinTask`/`archiveTask` record updates. Do not use `uor-r4-object-store` here. `window.__studioTest` exists only when `VITE_STUDIO_TEST=1` **and** `?studioTest=1`; the Playwright webServer builds with `VITE_STUDIO_TEST=1`, and default/production builds expose nothing.

- [ ] **Step 4: Run tests and commit.**

Run: `npm run build && npx playwright test tests/e2e/cursor-store.spec.ts`
Expected: PASS.

```bash
git add web/studio/src/agent/store.ts web/studio/src/agent/storeTypes.ts web/studio/src/agent/client.ts web/studio/tests/e2e/cursor-store.spec.ts web/studio/playwright.config.ts
git commit -m "feat(studio): add IndexedDB task store with canonical digest chain"
```

**Stop if:** the store needs one object-store blob for the whole run, duplicates `Store::save`, or requires a new dependency.

---

### Task 11: Runtime state machine, run.queued/started and send semantics

**Files:**
- Create: `web/studio/src/agent/runtime.ts`, `web/studio/src/agent/modelPort.ts`, `web/studio/src/agent/runtime.test.ts`
- Modify: `web/studio/src/agent/client.ts`
- Test: `runtime.test.ts`

**Interfaces:**
- Consumes: `AgentTaskStore`, `ModelPort`, `AgentEvent`.
- Produces: `AgentRuntime` methods from spec §10.1; `ReplayModelPort`; `RunState`, `TaskState`, `MessageReceipt`, `StopReceipt`, `ApprovalReceipt`.

**Depends on:** Task 10. **Cost:** 3–4 sittings. **Acceptance:** `run.queued` seq 0, `run.started` seq 1; queue/steer/force/stop/retry; force fallback; stop timeout; terminal events authoritative; approval digest/expiry.

- [ ] **Step 1: Write failing tests.**

```ts
test('run_queued_is_sequence_zero', ...);
test('run_started_is_sequence_one_with_retry_of_run_id_null_or_string', ...);
test('queue_message_is_delivered_at_next_turn', ...);
test('steer_is_delivered_at_next_tool_boundary', ...);
test('steer_falls_back_to_queue_when_run_finishes', ...);
test('force_send_falls_back_to_queue_when_run_finishes', ...);
test('fallback_queue_event_precedes_run_finished', ...);
test('fallback_starts_new_run_with_new_generation', ...);
test('write_tool_completion_records_changes_snapshot', ...);
test('stop_prevents_later_tool_dispatch', ...);
test('stop_ack_timeout_marks_waiting_for_background', ...);
test('host_resolution_clears_background_hold', ...);
test('stop_host_refusal_marks_idle', ...);
test('retry_creates_new_run_and_sets_retry_of_run_id', ...);
test('terminal_events_are_authoritative_over_run_status', ...);
test('approval_resolution_requires_matching_digest', ...);
test('approval_expiry_refuses_late_resolution', ...);
```

- [ ] **Step 2: Run to verify failure.**

Run: `npm test -- --run src/agent/runtime.test.ts`
Expected: FAIL with `Cannot find module './runtime.js'`.

- [ ] **Step 3: Implement the runtime.**

`ReplayModelPort` yields deterministic text/tool/usage/finished events; it is labelled synthetic. `startRun` writes `run.queued` (0), `run.started` (1) with `retry_of_run_id: string | null`, then model events. Queue is FIFO; steer at next tool boundary; force at next boundary. If the run ends first, emit `message.steered`/`message.forced { boundary:"next_turn" }` then `message.queued` **before** `run.finished`, then start a new run with a new generation. Never append after the old run's terminal event. On a write tool completion, record a `changes.snapshot` event (`scope:"last_turn"`, files, head_digest) and the `changes` store record. Stop: `run.status cancelling`, wait for host ack; on ack emit `tool.completed` then `run.cancelled`; on timeout emit `run.failed` `stop_unacknowledged`/`ack_timeout` and set `TaskRecord.hold = { reason: 'waiting_for_background', run_id, tool_call_id }` so the task stays `waiting_for_background` after the terminal run; a host resolution clears the hold. On refusal emit `run.failed` `host_denied`/`host_refused` and task idle. `run.status` never sets terminal. Approval validation checks ID/digest/task/run/generation/expiry; no boolean.

- [ ] **Step 4: Run tests and commit.**

Run: `npm test -- --run src/agent/runtime.test.ts`
Expected: PASS.

```bash
git add web/studio/src/agent/runtime.ts web/studio/src/agent/modelPort.ts web/studio/src/agent/runtime.test.ts web/studio/src/agent/client.ts
git commit -m "feat(studio): add browser runtime with sequence-zero runs and safe stop"
```

**Stop if:** the runtime needs a host process or the model port is implemented as a Rust-only trait.

---

### Task 12: Host client interfaces and transport matrix

**Files:**
- Create: `web/studio/src/agent/hostClient.ts`, `web/studio/src/agent/transports.ts`, `web/studio/src/agent/hostClient.test.ts`
- Modify: `web/studio/src/agent/client.ts`, `runtime.ts`
- Test: `hostClient.test.ts`

**Interfaces:**
- Consumes: `HostToolIntent`, `HostToolResult`, `ApprovalReceipt`.
- Produces: `HostClient` (`capabilities`, `execute`, `subscribe`, `approve`); `NullHostTransport` (`host_unavailable`), `TauriHostTransport` (`Channel<HostToolResult>`/poll), `LoopbackHostTransport` (fetch POST + NDJSON `HostToolResult` stream), `StaticHostTransport` (null).

**Depends on:** Task 11. **Cost:** 1–2 sittings. **Acceptance:** correct transport per surface; canonical tool arguments; float/non-ASCII keys rejected with `invalid_arguments`; no EventSource; no Channel assumption in the offline shim; `host_unavailable` in static.

- [ ] **Step 1: Write failing tests** (`static_host_is_unavailable`, `tauri_transport_uses_channel_when_available`, `tauri_offline_shim_uses_poll`, `loopback_transport_uses_fetch_stream_not_event_source`, `host_result_is_validated_against_intent`, `rejects_non_canonical_tool_arguments`).
- [ ] **Step 2: Run** `npm test -- --run src/agent/hostClient.test.ts`; expected FAIL with missing module.
- [ ] **Step 3: Implement** the interfaces; transports are stubs throwing `host_unavailable` until Stage 3; no real fetch yet.
- [ ] **Step 4: Run tests and commit** `feat(studio): define host tool intent/result transports`.

**Stop if:** the transport needs a second listener, EventSource, remote TLS or a host runtime.

---

### Task 13: Parallel tasks, persistent reload and hostile ingestion

**Files:**
- Create: `web/studio/src/agent/ingest.ts`, `web/studio/src/agent/ingest.test.ts`
- Modify: `runtime.ts`, `store.ts`
- Test: `ingest.test.ts`, `cursor-store.spec.ts`

**Interfaces:**
- Consumes: runtime/store.
- Produces: `IngestOutcome { Applied, Duplicate, Rejected(AgentErrorCode), Unknown }` with `code()`; `decodeFrame(bytes)`, `decodeEof(buffer)`; two live run workers in the page.

**Depends on:** Task 12. **Cost:** 2–3 sittings. **Acceptance:** two live tasks independent; persistent reload restores both; malformed/partial/duplicate/out-of-order/unknown/stale/cross-task never become output.

- [ ] **Step 1: Write failing tests** (`two_live_tasks_keep_independent_cursors`, `two_live_tasks_do_not_share_artifacts`, `reload_restores_two_tasks`, `rejects_malformed_frame`, `rejects_truncated_frame`, `rejects_sequence_gap`, `accepts_duplicate_same_digest`, `rejects_duplicate_different_digest`, `tolerates_unknown_event`, `rejects_stale_generation`, `rejects_cross_task_event`).
- [ ] **Step 2: Run** `npm test -- --run src/agent/ingest.test.ts && npm run build && npx playwright test tests/e2e/cursor-store.spec.ts`; expected FAIL on missing `ingest`/reload helpers.
- [ ] **Step 3: Implement** ingest and two-run scheduling; use real IndexedDB for reload proof, not an in-memory fixture.
- [ ] **Step 4: Run tests and commit** `feat(studio): run two browser tasks and reject hostile event streams`.

**Stop if:** restart proof uses an in-memory fixture or a host store.

---

### Task 14: Task detail, timeline, approval and error/retry UI

**Files:**
- Create: `TaskDetail.tsx`, `TaskTimeline.tsx`, `TimelineEvent.tsx`, `ApprovalCard.tsx`, `RunStatus.tsx`, `TaskError.tsx`, `TaskDetail.test.tsx`, `TaskTimeline.test.tsx`
- Modify: `StudioApp.tsx`, `AgentTranscript.tsx`
- Test: `TaskDetail.test.tsx`, `TaskTimeline.test.tsx`, `cursor-tasks.spec.ts`

**Interfaces:**
- Consumes: `AgentStore`, `TaskView`, `AgentClient`, `ComposerSurface`.
- Produces: `TaskDetail` props `{ task, client, onBack, onOpenSurface }`; `TaskTimeline` props `{ events, approvals, onResolve }`; `ApprovalCard` props `{ approval, hostIdentity, onResolve }`.

**Depends on:** Task 13. **Cost:** 2–3 sittings. **Acceptance:** deltas collapse; tool pair; unknown not text; malformed error row; approval host/expiry; stop hides later tools; retry only retryable; no checkpoint/Pause/Delete/Logs/cost.

- [ ] **Step 1: Write failing tests** as spec §13; include `no_checkpoint_restore_control_is_rendered`, `no_pause_delete_logs_or_cost_controls`.
- [ ] **Step 2: Run** `npm test -- --run src/components/TaskTimeline.test.tsx`; expected FAIL.
- [ ] **Step 3: Implement** the detail using `ReplayModelPort` events; approval buttons disabled in static (`host_unavailable`).
- [ ] **Step 4: Run component/e2e and commit** `feat(studio): render browser task detail and approvals`.

**Stop if:** the UI fabricates output or adds a host-owned task state.

---

### Task 15: Browser runtime acceptance

**Files:**
- Create: `web/studio/src/agent/acceptance.test.ts`, `web/studio/tests/e2e/cursor-runtime.spec.ts`
- Test: both files

**Interfaces:** acceptance only.

**Depends on:** Task 14. **Cost:** 2 sittings. **Acceptance:** two live tasks; persistent reload; safe stop; forged approval refused; hostile events never output; `run.queued` seq0 fixtures; no real model.

- [ ] **Step 1: Write acceptance tests** (`two_live_tasks_run_concurrently`, `reload_restores_both_tasks`, `stop_at_safe_boundary_blocks_later_tools`, `forged_approval_is_refused`, `malformed_partial_duplicate_out_of_order_never_become_output`, `unknown_thinking_and_heartbeat_tolerated`).
- [ ] **Step 2: Run** `npm test -- --run src/agent` and `npx playwright test tests/e2e/cursor-runtime.spec.ts`; expected FAIL until harness/barriers exist.
- [ ] **Step 3: Implement only the harness; fix runtime semantics with regressions if a test exposes a bug.**
- [ ] **Step 4: Run all Stage 2 tests and commit** `test(studio): accept browser runtime persistence and hostile streams`.

**Stop if:** a test needs a real model, a host runtime or a network.

---

## Stage 3 — Companion data APIs and transport

**Stage outcome:** the optional Tauri companion executes approved tools, exposes bounded data APIs, and serves the existing loopback listener as an NDJSON stream. **Stage acceptance:** `cargo test --manifest-path src-tauri/Cargo.toml --features gui --test agent_data --test agent_authority --test agent_loopback` and default variants pass; `cursor-static.spec.ts` passes; no host task runtime. **This stage needs a separate host-authority review.**

### Task 16: Companion data APIs — directory, Git diff, artifact read

**Files:**
- Create: `src-tauri/src/agent_data.rs`, `src-tauri/tests/agent_data.rs`
- Modify: `src-tauri/src/lib.rs` (register commands), `src-tauri/Cargo.toml`/`src-tauri/Cargo.lock` if needed
- Create: `web/studio/src/agent/dataClient.test.ts`
- Modify: `web/studio/src/agent/client.ts` (`tree`, `changesSnapshot`, `readArtifact`)
- Test: `agent_data.rs`, `dataClient.test.ts`

**Interfaces:**
- Consumes: existing `DesktopBackend` grants/file/Git authority.
- Produces: `agent_list_directory(grantId, path) -> FileNode[]`; `agent_git_diff(grantId, branchScope) -> ChangeSnapshot { scope: "branch", files, head_digest }` (branch-only); `agent_read_artifact(artifactId, range?) -> ArtifactBytes`; `AgentClient.tree`, branch `changesSnapshot`, host `readArtifact`. All data calls go through `HostClient.execute`/`agent_host_invoke`; no direct invoke.

**Depends on:** Task 15. **Cost:** 2–3 sittings. **Acceptance:** grant-bounded tree; branch diff; digest-verified artifact bytes; no nonexistent URL; no host task runtime.

- [ ] **Step 1: Write failing host tests** (`agent_list_directory_respects_grant`, `agent_list_directory_refuses_symlink`, `agent_git_diff_is_bounded`, `agent_git_diff_is_branch_only`, `agent_read_artifact_verifies_digest`, `agent_read_artifact_refuses_oversize`).
- [ ] **Step 2: Run** `cargo test --manifest-path src-tauri/Cargo.toml --features gui --test agent_data`; expected FAIL with missing module.
- [ ] **Step 3: Implement** commands through existing canonicalized grants; `agent_read_artifact` returns bytes + digest + media type and verifies the requested digest; no URL. `AgentClient.readArtifact` reads a browser artifact from IndexedDB first and only calls the companion for a host-side artifact. `agent_git_diff` is branch-only and never fabricates a last-turn diff; last-turn changes come from the browser `changes` store. Include `src-tauri/Cargo.lock` if changed.
- [ ] **Step 4: Run host + TS tests and commit.** Run `cargo test --manifest-path src-tauri/Cargo.toml --features gui --test agent_data` and `npm test -- --run src/agent/dataClient.test.ts`; then commit `feat(desktop): add grant-bounded data APIs for tree, diff and artifacts`.

**Stop if:** the command widens a grant, reads outside a grant, or introduces a host task store.

---

### Task 17: Companion tool intents and approvals

**Files:**
- Create: `src-tauri/src/agent_host.rs`, `src-tauri/tests/agent_authority.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`/lock if needed, `src-tauri/capabilities/default.json` only if required
- Modify: `web/studio/src/agent/transports.ts` (`TauriHostTransport`)

**Interfaces:**
- Consumes: `HostToolIntent`, `HostToolResult`, existing terminal/MCP/grant approvals.
- Produces: `agent_host_capabilities`, `agent_host_invoke`, `agent_subscribe` (`Channel<HostToolResult>`), `agent_host_poll`, `agent_host_resolve_approval`; `AgentHost` with no runtime.

**Depends on:** Task 16. **Cost:** 3–4 sittings. **Acceptance:** page cannot grant/approve itself; forged/cross-task/expired approval refused; in-flight write not killed; terminal/MCP existing approvals preserved; no host runtime.

- [ ] **Step 1: Write failing tests** (`agent_host_consumes_approval_by_identity`, `agent_host_refuses_page_supplied_grant`, `agent_host_refuses_forged_origin`, `agent_host_refuses_cross_task_approval`, `agent_host_stop_does_not_kill_in_flight_write`, `agent_host_terminal_requires_existing_approval`, `agent_host_mcp_requires_existing_confirmation`, `agent_host_rejects_non_canonical_arguments`, `tauri_subscribe_channel_emits_host_results`, `agent_host_has_no_task_runtime`).
- [ ] **Step 2: Run** `cargo test --manifest-path src-tauri/Cargo.toml --features gui --test agent_authority`; expected FAIL.
- [ ] **Step 3: Implement** intent validation, host receipt, existing tool dispatch; no task loop/state/store. `agent_host_poll` returns results after a cursor for the offline shim.
- [ ] **Step 4: Run tests and commit** `feat(desktop): execute approved tool intents without a host runtime`.

**Stop if:** the host starts owning task state, event logs or cursors.

---

### Task 18: Loopback NDJSON stream on the existing listener

**Files:**
- Create: `src-tauri/src/agent_server.rs`, `src-tauri/tests/agent_loopback.rs`
- Modify: `src-tauri/src/lib.rs`/`main.rs` (register routes on the existing listener; no new bind)
- Modify: `web/studio/src/agent/transports.ts` (`LoopbackHostTransport`)
- Test: `agent_loopback.rs`, `hostClient.test.ts`

**Interfaces:**
- Consumes: `AgentHost` dispatch; `AgentEvent`/host results.
- Produces: `POST /v1/studio/host/intents`, `GET /v1/studio/host/results` (`application/x-ndjson`), cursor query, token header; the stream carries `HostToolResult`, never `AgentEvent`.

**Depends on:** Task 17. **Cost:** 2–3 sittings. **Acceptance:** existing token/Origin checks; fetch stream; no EventSource; no remote/TLS; no second listener; `cursor_expired`/410 forces resync.

- [ ] **Step 1: Write failing tests** (`loopback_stream_requires_token_header`, `loopback_stream_requires_exact_origin`, `loopback_stream_is_ndjson_not_event_source`, `loopback_stream_replays_from_cursor`, `loopback_stream_returns_410_for_expired_cursor`, `loopback_data_uses_existing_post_path`, `no_remote_tls_route_is_registered`, `no_second_listener_is_opened`).
- [ ] **Step 2: Run** `cargo test --manifest-path src-tauri/Cargo.toml --features gui --test agent_loopback`; expected FAIL.
- [ ] **Step 3: Implement** route handlers on the existing loopback server; `fetch` POST + NDJSON `ReadableStream`; token header; no EventSource; no new bind/TLS.
- [ ] **Step 4: Add a loopback-browser test** in `cursor-runtime.spec.ts` or `agent_loopback.rs`; run and commit `feat(desktop): stream host results over the existing loopback listener`.

**Stop if:** the implementation opens a second listener, binds remotely or accepts a URL token.

---

### Task 19: Companion acceptance and static unsupported states

**Files:**
- Create: `web/studio/tests/e2e/cursor-static.spec.ts`
- Modify: `TaskDetail.tsx`, `ApprovalCard.tsx`, `DesktopTools.tsx`, `TerminalSession.tsx` (copy only)
- Test: `agent_authority.rs`, `cursor-static.spec.ts`

**Interfaces:** acceptance only.

**Depends on:** Task 18. **Cost:** 1–2 sittings. **Acceptance:** companion authority tests pass; static terminal/Git/account/background explicit; zero external requests at load; no host runtime.

- [ ] **Step 1: Write static e2e tests** (`static_browser_disables_terminal_with_exact_copy`, `static_browser_disables_git_with_exact_copy`, `static_browser_disables_account_and_background`, `static_browser_makes_zero_external_requests_on_load`).
- [ ] **Step 2: Run** `cargo test --manifest-path src-tauri/Cargo.toml --features gui --test agent_authority --test agent_data --test agent_loopback && npm run build && npx playwright test tests/e2e/cursor-static.spec.ts`; expected FAIL on missing copy.
- [ ] **Step 3: Add the exact unsupported copy; do not weaken host checks.**
- [ ] **Step 4: Run and commit** `test(desktop): accept companion data APIs and static unsupported states`.

**Stop if:** a copy fix changes a capability claim rather than the UI.

---

## Stage 4 — Review and editor

**Stage outcome:** file tree/tabs/editor through `AgentClient.tree`, changes snapshot with last-turn/branch scopes, local diff comments, and PR/artifacts through `AgentClient.readArtifact`. **Stage acceptance:** `FileTree.test.tsx`, `EditorPane.test.tsx`, `DiffReview.test.tsx`, `PrPanel.test.tsx`, `ArtifactPanel.test.tsx`, `cursor-review.spec.ts` pass; no checkpoint/merge/LSP claim. **No disabled panel is evidence.**

### Task 20: File tree, tabs and UOR editor through `AgentClient.tree`

**Files:**
- Create: `FileTree.tsx`, `EditorTabs.tsx`, `EditorPane.tsx`, `FileTree.test.tsx`, `EditorPane.test.tsx`
- Modify: `WorkspaceEditor.tsx`, `client.ts` (`tree`), `types.ts` (`FileNode`)
- Test: `FileTree.test.tsx`, `EditorPane.test.tsx`

**Interfaces:**
- Consumes: `AgentClient.tree(taskId?) -> FileNode[]`; existing save adapter.
- Produces: `FileNode`, `FileTree`, `EditorTabs`, `EditorPane`; editor hidden until this task.

**Depends on:** Task 19. **Cost:** 2–3 sittings. **Acceptance:** bounded tree search; dirty close confirmation; save serialization; large-file refusal; focus return; "UOR text editor" label.

- [ ] **Step 1: Write failing tests** (`file_tree_search_is_bounded`, `file_tree_uses_agent_client_tree`, `dirty_tab_close_requires_confirmation`, `save_serializes_and_verifies_active_path`, `large_file_is_refused`, `editor_labels_it_is_not_vs_code_or_lsp`).
- [ ] **Step 2: Run** `npm test -- --run src/components/FileTree.test.tsx src/components/EditorPane.test.tsx`; expected FAIL.
- [ ] **Step 3: Implement** using `AgentClient.tree`; no second host path API; static uses uploaded files; no code-editor dependency.
- [ ] **Step 4: Run and commit** `feat(studio): add tree, tabs and UOR editor from agent data`.

**Stop if:** the tree needs a host path outside `agent_list_directory` or a new dependency.

---

### Task 21: Changes snapshot, diff review and local comments

**Files:**
- Create: `DiffReview.tsx`, `DiffComment.tsx`, `DiffReview.test.tsx`
- Modify: `DesktopTools.tsx`, `client.ts` (`changesSnapshot`), `runtime.ts` (`diff.comment`)
- Test: `DiffReview.test.tsx`, `cursor-review.spec.ts`

**Interfaces:**
- Consumes: `ChangeSnapshot { scope: 'last_turn' | 'branch', files, head_digest }`; local `diff.comment` event.
- Produces: `DiffReview` props `{ taskId, scope, onScope, comments, onComment }`; local comments persisted in IndexedDB.

**Depends on:** Task 20. **Cost:** 2 sittings. **Acceptance:** last-turn scope from runtime; branch scope from `agent_git_diff`/uploaded diff; comments survive reload with head digest; no merge/checkpoint.

- [ ] **Step 1: Write failing tests** (`changes_snapshot_last_turn_scope` (browser store), `changes_snapshot_branch_scope` (host branch-only), `static_browser_reviews_uploaded_file_locally`, `diff_comment_survives_reload`, `stale_comment_line_is_marked`, `no_merge_control_without_service_identity`, `no_checkpoint_restore_control_is_rendered`).
- [ ] **Step 2: Run** `npm test -- --run src/components/DiffReview.test.tsx`; expected FAIL.
- [ ] **Step 3: Implement** `changesSnapshot`; last-turn from the browser `changes` store (`changes.snapshot`), branch from `agent_git_diff`/upload only; `diff.comment` local event; comments keyed by head digest; no remote comments; no fabricated host last-turn diff.
- [ ] **Step 4: Run and commit** `feat(studio): add task-scoped changes review and local comments`.

**Stop if:** last-turn diff would be fabricated or remote comments would require a service.

---

### Task 22: PR link/read-only and digest-bound artifacts

**Files:**
- Create: `PrPanel.tsx`, `ArtifactPanel.tsx`, `ArtifactPreview.tsx`, `PrPanel.test.tsx`, `ArtifactPanel.test.tsx`
- Modify: `PaneDock.tsx`/`StudioApp.tsx` (tabs), `client.ts` (`readArtifact`)
- Test: `PrPanel.test.tsx`, `ArtifactPanel.test.tsx`

**Interfaces:**
- Consumes: `AgentClient.readArtifact(artifactId, range?) -> ArtifactBytes`; optional `PrSummary`.
- Produces: `PrSummary`, `ArtifactPanel`, `ArtifactPreview`; no merge.

**Depends on:** Task 21. **Cost:** 1–2 sittings. **Acceptance:** PR link/read-only; artifact bytes digest-verified; preview inert; expired link not reused; no nonexistent URL.

- [ ] **Step 1: Write failing tests** (`pr_tab_is_read_only_link_without_integration`, `no_merge_control_without_service_identity`, `artifact_read_verifies_digest`, `artifact_preview_is_inert`, `expired_artifact_link_is_not_reused`, `no_checkpoint_restore_control_is_rendered`).
- [ ] **Step 2: Run** `npm test -- --run src/components/PrPanel.test.tsx src/components/ArtifactPanel.test.tsx`; expected FAIL.
- [ ] **Step 3: Implement** `readArtifact`; no URL fetch; PR read-only; no merge.
- [ ] **Step 4: Run and commit** `feat(studio): add read-only PR and digest-bound artifacts`.

**Stop if:** artifacts need a URL/token in the page or a merge control.

---

## Stage 5 — Final local acceptance

### Task 23: Final responsive, keyboard, IME, accessibility and gates

**Files:**
- Create: `web/studio/tests/e2e/cursor-final.spec.ts`
- Modify: `cursor-a11y.spec.ts` only for new named checks
- Test: all `cursor-*.spec.ts`, existing Studio suites, full gates

**Interfaces:** acceptance only.

**Depends on:** all executable tasks. **Cost:** 3–5 sittings. **Acceptance:** 390/1024/1440; protected selectors; no second skip link; 1024 exact; keyboard/IME/a11y; static unsupported; no model claim.

- [ ] **Step 1: Write final acceptance tests** (`final_protected_selectors`, `final_1024_boundary`, `final_keyboard_only_task_flow`, `final_ime_cjk_dead_key_rtl`, `final_a11y_targets_headings_focus`, `final_static_unsupported_states`, `no_agent_secret_is_written_to_local_storage`, `model_sources_key_is_documented_exception`, `default_build_does_not_expose_studio_test_api`, `matrix_planned_evidence_names_reconciled`).
- [ ] **Step 2: Run** `npm run build && npx playwright test tests/e2e/cursor-final.spec.ts`; expected FAIL until final CSS/copy.
- [ ] **Step 3: Fix only the typed shell; do not restyle legacy.**
- [ ] **Step 4: Run all `cursor-*.spec.ts`, existing Studio suites, `uor-harness check`, the repository's full gate set from `docs/harness/README.md`, then `uor-harness loop report --check` and `uor-harness loop pr-check --local`, serially. Run a separate default `npm run build` (without `VITE_STUDIO_TEST`) and assert the served page does not expose `__studioTest`. Do not report a model latency/tokens-per-second/RAM number; real model is appendix-only.**
- [ ] **Step 5: Reconcile the coverage matrix against actual test names; every appendix row uses `auth.rs`, `tenant.rs`, `secrets.rs`, `background.rs`, `notifications.rs`, `github.rs` or is marked "future proposed". Resolve every row to an executed test or `NOT_RUN`/`UNVERIFIED`.
- [ ] **Step 6: Commit** `test(studio): final local acceptance for browser-owned task workspace`.

---

## Appendix A — Gated future service/integrations (not executable)

**No checkboxes and no numbered tasks.** This appendix defines the boundary for a separate owner-approved item and security review. It is not authorized by this plan. No `services/**` code, real credentials, public endpoint, deployment or provider token is added here.

- **Identity/session/tenant:** auth provider, session expiry, tenant isolation, no unauthenticated route. Proposed files `services/studio-service/src/{auth,tenant,routes}.rs`; tests `tests/auth.rs`, `tests/tenant.rs`. `NOT_RUN`.
- **Secrets/background/notifications:** encrypted secret store, injection/redaction, background restart, notification delivery. Proposed files `secrets.rs`, `background.rs`, `notifications.rs`. `NOT_RUN`.
- **GitHub-first integration:** App install, webhook signature/timestamp/replay checks, PR/checks readback, explicit write approval; other providers later. Proposed files `integrations/github.rs`; tests `github.rs`. `NOT_RUN`.
- **Account/admin/billing/SSO/SCIM/audit/privacy:** capability-gated, no fake menus, no X Subscription/legacy cohort copy. `NOT_RUN`.
- **Evidence class:** all appendix rows are `N`/`V`; none is a UOR capability. A future item must repeat the owner gate, threat model, deployment target and security review.

## Appendix B — Gated real-R4 qualification (not executable)

**No checkboxes and no numbered tasks.** The accepted M1 artifact is **prediction-only**; it is not the task brain. This appendix records the qualification boundary, not a claim.

- **Real generation/tool adapter:** `R4ModelPort`/worker adapter only after a generation-capable artifact and Claude's merged streaming/tool contract exist at an exact head. Test `r4_model_port.rs`; `NOT_RUN`. Do not edit `crates/uor-r4-api/**`.
- **Useful-agent evaluation:** sealed fresh panel with denominator/uncertainty, artifact/tokenizer identity, no fixture output counted; `NOT_RUN` until owner pre-registration.
- **Prediction-only top-8 measurement:** may be described separately as a prediction-only measurement of the accepted M1 artifact; it is not useful-agent generation and does not establish chat or task capability. `NOT_RUN` in this plan.
- **Performance:** first-token/tokens-per-second/RAM/energy are `NOT_RUN` for the agent loop until a generation-capable artifact exists; do not derive them from the prediction-only M1 artifact.
- **Evidence class:** `L` only after a real run; otherwise `N`.

---

## Verification — plan-level acceptance bar

- **Contract:** run-scoped envelope, `run.queued` seq0/`run.started` seq1, no `task.created`, `retry_of_run_id`, `runtime_unavailable`, canonical JSON, cross-runtime digests.
- **Runtime:** browser-owned loop; one task many runs; queue/steer/force/stop/retry; force fallback; stop timeout; terminal vs status; two live tasks; reload/corruption.
- **Companion:** optional tool executor; data APIs; approval identity; existing listener/token/Origin; NDJSON fetch; no EventSource/TLS/second listener; no host runtime.
- **UI:** protected selectors/names; one skip link; 1024 does not auto-collapse; `.task-workspace`; task rail/search/pins/groups; composer media/`@`/`/`; send modes; editor hidden until Stage 4; changes/comments/PR/artifacts; 390/1024/1440; a11y/IME regression.
- **Privacy:** local model; zero external requests at load; no agent secret in `localStorage`; documented `modelSources.v1` exception; no signing key in page.
- **Static honesty:** terminal/Git/account/background explicit; no fake task log; no useful-agent claim.
- **Delivery:** owner approval, exact-head review, protected merge, fresh-main verification, no owner metadata edits, no out-of-scope files.

## Self-review against the writing-plans checklist

1. **Spec coverage:** architecture (§6 → Tasks 3/10–15), contract/canonical (§9 → Tasks 1–3), shell/composer (§11 → Tasks 5–9), browser persistence/runtime (§10 → Tasks 10–15), companion/data/transport (§9.7–9.8, §12 → Tasks 16–19), review/editor (§11.6 → Tasks 20–22), final acceptance (§14–16 → Task 23), service/R4 appendices. No spec requirement is left without an owner task or a gated appendix.
2. **Step scan:** executable tasks have checkbox steps, exact tests/commands; appendices have no checkboxes and no numbered tasks; later tasks reference earlier Interfaces blocks.
3. **Type consistency:** `AgentEvent` fields are identical in spec §9.2, Task 1, Task 3, Task 10 and Task 14. `TaskSummary`/`RunState` are identical in §8 and Tasks 3/11/14. `HostToolIntent`/`HostToolResult` are identical in §9.7 and Tasks 12/16/17/18. `AgentClient` methods are identical in Task 3 and Tasks 16/20/22. No `ModelApi` is introduced; `ModelPort` is TypeScript.
4. **Review Focus:** all five classes have a named owning task/test; existing IME is regression, not a fabricated red.
5. **Proportion:** 23 executable tasks in six stages; appendices are compact and checkbox-free. Code blocks are test assertions/signatures, not implementation bodies.

## Execution handoff

**Plan complete and saved to `docs/superpowers/plans/2026-11-09-cursor-hosted-studio-upgrade.md`.** The revised design is `docs/superpowers/specs/2026-11-09-cursor-hosted-studio-upgrade-design.md`.

**Owner approval is required before Task 1.** Two independent builder checks returned **REVISE**; neither approved this plan. Delivery reviewer `agent-94` returned **CHANGES** on the prior report head for research-record consistency, and `agent-95` returned **CHANGES** for staged-table/audit/record arithmetic. Those corrections are **applied in this PR; fresh delivery review pending**. Final exact-head approval remains required. The plan is not approved until the owner reviews this revision and either creates a new Cursor-hosted Studio item with browser-owned-runtime acceptance (recommended), with the approval ORDER naming that new item, or explicitly supersedes #3569's old Rust-runtime `Done when` in an ORDER naming `studio-ide-harness`. #3569 cannot be closed by this plan. Recommended execution: **subagent-driven** — a fresh implementer per task and a fresh reviewer before the next task, then a whole-branch review. Native execution is acceptable if the owner/lead prefers one session.

**Does the revised design capture what you want?** If yes, record owner approval in a new ORDER. If not, revise the design first; do not start a task.
