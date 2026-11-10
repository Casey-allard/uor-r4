# Cursor-hosted Studio upgrade — design (revised after builder check)

**Status:** design only, revised 2026-11-09 after the independent builder check returned **REVISE**. **Owner approval is required before any implementation.** This document does not authorize code, dependencies, compute, credentials, service deployment or a capability claim.

**Date:** 2026-11-09. **Item:** [`studio-ide-harness`](../../roadmap/items/studio-ide-harness.md) (#3569). **PR base:** `6fcf57f7822286e5b6bd7bcfe42d668116b23afa`. **ORDER/worktree commit:** `452615bc52a0186c1c01007986ff5c1a767eb068` (not the PR base).

**Research:** [Cursor hosted experience](../../research/reports/Cursor%20hosted%20experience.md) and its six notes plus [source audit](../../research/research_notes/Cursor%20hosted%20experience/source-audit.md). **Executable plan:** [2026-11-09-cursor-hosted-studio-upgrade.md](../plans/2026-11-09-cursor-hosted-studio-upgrade.md).

**Authority order.** `CHARTER.md` and the roadmap item win over this design; this design wins over the plan; the plan wins over the research report where it makes an implementation choice. The report is background evidence, not a specification of private Cursor behavior.

**Revision note.** The first design put the task runtime and durable task store in the Tauri host. The builder check found that this contradicts the owner's model-in-browser requirement and makes the companion mandatory for the core loop. This revision adopts **browser-first loop A**: the runtime, reducer, cursor and durable task persistence live in the page/Web Worker; the companion is an optional tool executor; the service is a separate owner item. It also corrects the contract, canonical serialization, persistence, protected-selector, transport, data-source and staging defects recorded in the builder review.

---

## 1. Decision in one page

Build **browser-first loop A**:

1. **One Preact/Vite frontend.** The existing `web/studio` build remains the only shell for the static page, the native runner and the Tauri app. No second frontend, editor product or task-state owner.
2. **The R4 model runs in the browser** in a Web Worker, from the exported integer artifact. Prompts, replies and model artifacts stay on the device in local mode.
3. **The task runtime lives in the page/Web Worker.** Task loop, reducer, cursor, queue/steer/force/stop, approvals, event log and durable task storage are browser-owned. Durable task/event storage uses **IndexedDB**, with a canonical JSON digest chain and a verified snapshot manifest.
4. **The model port is a TypeScript worker-side port**, not a Rust-only trait. The executable stages use a clearly labelled `ReplayModelPort` fixture. The accepted M1 artifact is prediction-only and is not the task brain; real generation is `NOT_RUN` until a generation-capable artifact and Claude's merged streaming/tool contract exist.
5. **A companion is optional.** The Tauri app executes approved tools (files, Git, PTY, MCP stdio/HTTP) and holds grants/approvals. It **never owns the task loop or the task store**. Static browser supports the local model, task loop, transcript and review through user-mediated files/uploads; only real terminal/Git/MCP stdio requires the companion.
6. **A full hosted service is a separate future owner item and security review.** It is outside this plan. No public endpoint, credentials or deployment is authorized here.

The target is Cursor's **loop**, not its chrome: a durable task that accepts a queue, a steer, a force-send and a stop; a review handoff; an optional tool executor; and a service boundary only where a browser and a host process cannot do the job.

**What this is not:** a Cursor clone, a full-parity claim, a hosted production plan, a real R4 generation adapter, a model-capability claim, a VS Code replacement, or an authorization to implement.

---

## 2. Owner direction and gate

The owner's latest direction is the **Cursor hosted experience**. The Cursor report is the current target; the Cowork material is historical context. This unit does **not** change:

- the roadmap item's `status`, `scope`, `done_class`, `depends_on`, `open_to` or `done when`;
- any capability status, bar, acceptance text or `NOT_RUN`/`PASS` row in [`docs/products/harness-parity.md`](../../products/harness-parity.md);
- the owner's checkout, roadmap metadata (`docs/roadmap/README.md`, `docs/roadmap/issues.json`), rendered STATE/HANDOFF, or `.github/**`;
- the model, engine, worker, tokenizer, `crates/uor-r4-api/**` or any current API path.

**Approval gate — owner-only.** Before Task 1 of the plan starts, **the owner** must approve this design and the plan in a new ORDER that names the correct item: the **new Cursor-hosted Studio item** if the recommended route is chosen, or `studio-ide-harness` **only** if the owner also explicitly supersedes its same-Rust-runtime `Done when`. The lead cannot substitute a delegated approval for this gate, even though CHARTER §5.1 normally allows delegation; this ORDER requires owner approval. Until that approval exists, every implementation task is `BLOCKED`. Approval of the research report, or of the first design, is not approval of this revision.

**Owner acceptance item — required before Task 1.** The roadmap item's existing `Done when` line still requires "the same Rust runtime in Tauri/local and hosted-service modes" and is **not edited** by this revision. Before Task 1, the owner must either (recommended) create a **new Cursor-hosted Studio roadmap item** whose acceptance names the browser-owned runtime, leaving #3569's Rust-runtime bar untouched, or explicitly supersede that `Done when` in the approval ORDER. This is an owner-only decision; the lead cannot settle it by documentation. Until it is recorded, no implementation is authorized and #3569 cannot be closed by this plan.

**Scope note.** The kimi-18 ORDER authorizes this design and plan. The roadmap item's frontmatter `scope` is owner metadata and is not edited by this planning unit. A future implementation ORDER must name every new path (`web/studio/src/agent/**`, `web/studio/src/agent/generated/**`, `web/studio/src/components/**`, `src-tauri/src/**`, `src-tauri/tests/**`, `scripts/studio/**`, Cargo lockfiles) before code starts; a path not named there is out of scope. `services/**` is a separate owner item.

---

## 3. What the Cursor record supports, and what it does not

Cursor's hosted product is five surfaces sharing one backend, not one responsive web IDE:

| Capability | Web `cursor.com/agents` | Desktop Agents Window | Desktop IDE | Origin web | iOS |
|---|---|---|---|---|---|
| Code editing | Editing surface exists; full VS Code-class depth **unverified** | Yes (file view/edit, LSP per Cursor 3 blog) | Yes (VS Code workbench) | Code browsing, no editor | No (diff review only) |
| File tree | Yes per mobile FAQ/release notes | Yes | Yes | Yes (Code tab) | No |
| Integrated terminal | Yes, remote PTY per release notes | Yes | Yes, local PTY | No | No |
| Cloud agents | Primary surface | Yes | No since 3.0 | Can author PRs/tours | Control/review |
| Command palette | Not documented | `Cmd+K` palette | `Cmd+Shift+P` | Not documented | No |
| Themes / extensions | Not documented; not the IDE | Themes require **Open IDE**; not VS Code extensions | Yes | No | No |
| LSP / debugger / notebooks | **Unverified** | LSP yes; debugger/notebooks undocumented | Yes | No | No |
| Tab autocomplete | Not documented | Not documented | Yes | No | No |
| Inline edit | Retired | Retired (`Cmd+K` is palette) | Retired | No | No |
| Voice chat | Documented on web | Yes | Yes | No | Yes |
| Remote SSH/containers | Not documented as a web client | Remote SSH | Yes | No | Remote Control |

Sources: [Cloud Agents](https://cursor.com/docs/cloud-agent), [Agents Window](https://cursor.com/docs/agent/agents-window), [Cursor 3 blog](https://cursor.com/blog/cursor-3), [Cursor 3.0 changelog](https://cursor.com/changelog/3-0), [iOS](https://cursor.com/docs/cloud-agent/mobile), [keyboard shortcuts](https://cursor.com/docs/reference/keyboard-shortcuts).

**Evidence rules used throughout.** `[OBS]` public UI observed 2026-11-09; `[DOC]` official Cursor/VS Code documentation, help, changelog or release notes; `[INF]` inference from cited facts; `[UNV]` not verifiable without an account, task or paid team. Exact strings from documentation are **copy candidates**, not observations of the signed-in app. The public marketing mock is not layout truth. The auth host uses a different design system from the product.

**Facts the design must respect.**

- `/agents` redirects unauthenticated visitors to WorkOS; signed-in information architecture, real placeholders, empty states and ARIA are **unverified** (audit G1/G9/G10).
- Desktop Agents Window chrome (Worked sections, Plan pill, Changes scopes, PR-tab memory, archive undo, artifact cards, merge controls, 75-pin limit) is mostly documented in the desktop release stream, not for web. Do not promote it into web behavior (C1).
- The web has an editing surface, but its depth is unverified; it is not documented as Monaco/VS Code and has no documented LSP, debugger, extensions, themes, notebooks or split view (C3, G3).
- Hosted checkpoint restore is **not documented**; official checkpoints are an Agent/IDE feature (C2). No hosted Restore Checkpoint control.
- Pause, Delete, Logs and per-task dollar cost are **not documented** in the web/desktop task UI. That is a documentation gap, not proof of absence. Say "not documented; UI status unknown" (C15).
- MCP SSE conflicts across pages: cloud capabilities say unsupported, the API schema enumerates `sse`; only HTTP and stdio are safe documented transports for cloud. Treat SSE as unverified (C11).
- Current X Subscription and legacy Cursor Pro/Pro+/Ultra/Teams cohorts coexist. UOR uses neither; no plan names, prices or cohort language enter UOR defaults (C12).
- Mobile push has conflicting sources; do not design push behavior from it.
- `Cmd+K` is surface-specific; `Tab` is the most dangerous global binding; inline edit is retired. A UOR shell must not bind either globally (C6, C16).

**What this means for UOR.** Copy the interaction loop and the review handoff. Do not copy Cursor's backend, fonts, assets, trade dress, account/plan behavior or private UI. Every UOR requirement below carries a surface tag and an evidence status; unknown Cursor behavior stays `UNVERIFIED`. UOR-only bindings (for example `Cmd/Ctrl+Shift+P` for the UOR palette) are **UOR choices**, not Cursor facts.

---

## 4. Product target and non-goals

### 4.1 The one-screen target

A UOR-branded, low-chrome task workspace:

- **Left rail:** tasks with search, pins and `Today / Last 7 days / Older` groups; archived tasks collapsed; new task.
- **Center:** transcript-first task detail: header (status, starting prompt, project/repo, branch, environment), run status, plan card, assistant/tool timeline, approval cards, error/retry, and a composer anchored below.
- **Composer:** multiline text; paste/drop image and document attachments; `@` context; `/` skills and templates; model, mode and environment pickers; queue/steer/force-send/stop; accessible validation. The existing `textarea[aria-label="Prompt"]` and visible `Send` button remain protected.
- **Work surface (tabs):** Changes, PR, Artifacts, Terminal (companion only), Editor (hidden until Stage 4). Each tab is honest about its source and depth.
- **Local settings:** theme, reduced motion, editor font/tab/wrap, local data. No account menu until a service exists.
- **Visual system:** UOR tokens, one accent for focus/selection, hairline borders, restrained radii, strong focus discipline. Not Cursor's palette or fonts.

### 4.2 Non-goals for this plan

- Full VS Code-class editing (LSP, debugger, extensions, notebooks, themes, multi-cursor, split view).
- A hosted VM/worker platform, cloud desktop streaming, signed agent commits, repository installation tokens, webhooks or a public agent endpoint.
- iOS/Android apps, voice chat/dictation, computer use, embedded browser automation, scheduled/background tasks, team sharing, audit, SSO/SCIM, billing.
- Hosted checkpoint restore; a Pause button; a Delete/Logs tab; per-task dollar cost.
- A real R4 generation/tool adapter before the artifact and API exist; any useful-conversation claim.
- Full Cursor parity at any intermediate stage. Parity is a per-control, per-surface claim and remains `NOT_RUN` until executed.

---

## 5. Approaches compared

| Approach | What it buys | What it costs | Verdict |
|---|---|---|---|
| **A. Browser-first loop (recommended)** | Model, runtime, reducer, cursor and durable task log all in the page/worker; companion optional; static browser supports the local loop; privacy preserved | Browser storage limits; no host tools without companion; tool execution needs an explicit intent/result protocol | **Chosen.** Only architecture consistent with model-in-browser. |
| **B. Backend-first Cursor clone** | Full execution plane, multi-user, integrations, background work | Rebuilds Cursor's hardest infrastructure before the shell; makes the browser model secondary; large security burden | **Rejected as first move.** |
| **C. Static-only Cursor clone** | Fast; honest demo; no backend | No real terminal/Git/MCP stdio; no companion tools; every host action simulated or disabled | **Not sufficient alone.** Static is the default shell, not the whole product. |
| **D. Embed a VS Code-like editor / VS Code** | Familiar editing, syntax highlighting | Bundle weight; a second shell/backend/authority for `code-server`; LSP/debugger still need a server | **Optional component later; not the architecture.** No editor dependency in this plan. |

**The static browser cannot reproduce Cursor's backend.** Repository installation tokens, webhooks, isolated VMs, PTYs, MCP stdio servers, secret storage, signed commits, background execution and cross-device resume require a server or a host process. The honest static product is a local model + local task loop + user-mediated files + explicit unsupported states.

---

## 6. Architecture and authority

```
Browser page / Web Worker (authority for the task loop)
  ├─ R4 model runtime (WASM, integer artifact, tokenizer, exact memory)
  ├─ AgentRuntime (TS): task/run loop, queue/steer/force/stop/retry, approvals, events
  ├─ AgentStore (TS): reducer, cursor, IndexedDB task/event/snapshot persistence
  ├─ ModelPort (TS): ReplayModelPort now; generation-capable adapter NOT_RUN
  └─ HostClient (TS): tool intents / results, or NullHostTransport in static
        ├─ Tauri webview: agent_host_invoke + agent_subscribe Channel (or poll fallback)
        ├─ Loopback browser: fetch POST intents + fetch NDJSON stream with token
        └─ Static: in-page local files/uploads; no terminal/Git/MCP stdio

Optional companion (Tauri, tool executor only)
  ├─ granted folders, file read/write/list, Git status/diff
  ├─ PTY (enable + exact-command approval)
  ├─ MCP stdio/HTTP (native confirmation)
  ├─ approvals (identity + host receipt)
  └─ optional bounded snapshot export through the object store (not authority)

Separate future owner item (not this plan)
  └─ hosted service: identity, secrets, background, notifications, integrations, audit
```

### 6.1 Authority owners

| Owner | Owns | Must never do |
|---|---|---|
| **Browser page/Worker** | Task runtime, reducer, cursor, IndexedDB task/event storage, model port, queue/steer/force/stop, transcript, local attachments, local comments | Read arbitrary host paths; spawn a shell; hold a repository token or signing key; grant itself approval; store agent secrets in `localStorage` |
| **Optional companion (Tauri)** | Approved tool execution: file/Git/PTY/MCP; grants; approvals; host receipts; optional bounded snapshot export | Own the task loop or task store; trust a page-supplied boolean; widen a grant; expose its token to the page; run a second authority channel |
| **Future service (separate item)** | Identity/session, secrets, background/cross-device, webhooks, notifications, team/admin/audit | Become a second local host authority; accept an unauthenticated public request; claim a capability the plan does not deliver |

**One authority channel.** The existing Tauri command + loopback `/__tauri_invoke` and the existing loopback Studio listener remain the only local host authority. The new agent host routes are added to that existing listener with its exact Host/Origin/Content-Type/token checks; no second listener, no remote/TLS server. The runtime never calls host tools directly; it sends a `HostToolIntent` and receives a `HostToolResult`.

**One frontend.** `web/studio` remains the only frontend. The typed shell mounts into `#studio-typed-root`; `#app` stays in the document and is hidden only when the typed root is active. Do not create `web/studio2`, a second Vite entry, or a second **task-state** owner. `LifecycleStore` remains the legacy shell/session/model store; the browser `AgentStore` owns task/run/event/cursor state only.

**One event schema.** The browser runtime owns `AgentEvent`; TypeScript types are generated from a Rust-owned schema document. Rust supplies the schema/canonicalization reference; the browser runtime is the producer. Tauri tool results become `AgentEvent`s only after the runtime validates them. No hand-forked event union.

### 6.2 Privacy by mode

| Mode | Leaves the device | Stays local | Default |
|---|---|---|---|
| **Static browser, local model** | Nothing at page load; no telemetry | Prompts, replies, model artifact, task/event log in IndexedDB, attachments | Yes |
| **Static browser, user-configured API** | Prompt/attachments to the endpoint the user chose | Task/event log, UI state | Explicit user action only |
| **Local companion** | Nothing unless the user connects a remote MCP server or a model provider | Files within grants, PTY output, Git, task events in the page, optional export | Yes for host tools |
| **Future service** | As required by the selected feature | Local model if the service is used only for review/PR | Separate owner item |

**Local model rule.** In local WASM mode no prompt, reply or model artifact is sent anywhere. Attachments are held in memory or IndexedDB, not sent to a host unless the user chooses a host tool. The page makes zero external requests at load. Any network action is user-initiated and visibly labelled.

**Secret rule — agent-scoped.** No **agent** secret (provider key, GitHub token, webhook secret, signing key) is stored in `localStorage`. The existing user-configured provider key store `uor.studio.modelSources.v1` is a documented base exception for user-entered provider keys; it is not an agent secret store and this plan does not silently migrate it. A future host/session migration for provider keys is separate and needs its own decision.

---

## 7. Surface contract

Every requirement carries one of these tags:

| Tag | Meaning | UOR position |
|---|---|---|
| **Web** | The hosted/static browser surface, `cursor.com/agents` analog | Primary target. Shell, local model, browser runtime, task view, review, explicit unsupported states. |
| **Companion** | The desktop Agents Window analog, but optional and tool-only | Optional. Terminal, files, Git, MCP stdio, approvals, bounded export. |
| **IDE** | Full VS Code-derived workbench | **Not a UOR target.** Do not claim LSP/debugger/extensions/notebooks/Remote SSH. |
| **iOS** | Native mobile app | **Not a UOR target.** Responsive web is the mobile story. |
| **API** | Cursor Cloud Agents API | **Not copied.** UOR defines its own browser runtime contract. |
| **Managed/self-hosted service** | Cursor cloud/self-hosted workers | **Not built.** Separate owner item. |

**Promotion rule.** A capability documented for the desktop Agents Window or IDE is not a web requirement. A capability documented for the API is not a UI requirement. A capability observed on public marketing is not evidence of the signed-in app. Where the report says `UNVERIFIED`, the UOR design either omits the control or labels it explicitly.

**Protected selectors and names.** The typed shell must preserve the existing baseline selectors and accessible names: `nav[aria-label="Studio"]`, `textarea[aria-label="Prompt"]`, a visible `Send` button, `.agent-workspace`, `.agent-transcript`, `.agent-composer`, `.workspace-grid`, `.pane-dock`, `.session-list`, `[data-control=…]`, `[data-pane-target=…]`, `[data-action="toggle-projects"]`, and the existing single skip link. Do not create a second skip link; retarget the existing one at typed mount (or hide it in typed mode). Keep baseline tests valid rather than renaming tests to fit a redesign. Use a distinct task-workspace class (`.task-workspace`), not `.studio-shell`, to avoid the existing collision.

**Web editing rule.** The web editing surface exists, but its depth is unverified. UOR ships a **UOR text editor** with file tree, tabs, dirty/save and bounded search. It is not Monaco/VS Code and has no LSP/debugger/extension/theme/notebook claim. A code-editor component is a later optional decision, not this plan's default.

---

## 8. Domain model

| Object | Identity | Meaning | Lifetime |
|---|---|---|---|
| **Project** | `prj_<16 hex>` | Workspace boundary: name, optional repo/branch, local folder grant reference | Durable |
| **Task** | `tsk_<16 hex>` | The durable unit the user opens, searches, pins and archives. One task has many runs. | Durable; archive is reversible |
| **TaskSummary** | projection of `TaskRecord` + active run | Task list/detail header: title, project, repo, branch, pinned, archived, state | Derived |
| **Run** | `run_<16 hex>` | One bounded execution started by a create, follow-up, retry or resume. | Durable log; terminal |
| **Turn** | `turn_<run_id>_<index>` | One model/tool loop inside a run | Run-scoped |
| **Message** | `msg_<16 hex>` | A user message, queued/steered/forced, with attachments | Task-scoped |
| **ToolCall** | `tc_<16 hex>` | One approved host tool invocation with a stable envelope | Run-scoped |
| **Approval** | `apr_<16 hex>` | An identity object asking the host to authorize one tool call | Expires |
| **Artifact** | `art_<16 hex>` | A file, image, log or link produced by a run | Task-scoped; digest-bound |
| **Snapshot** | `snap_<run_id>` | A verified manifest of events/digest chain at a point in time | Durable |
| **Cursor** | `uorse1…` | A resumable position in one run's event log | Client-held |

### 8.1 Task state is derived; runs are authoritative

**Task state** is not a separately stored state machine. It is derived from `TaskRecord.archived` plus the active run:

| Condition | Task state |
|---|---|
| `archived` | `archived` |
| active run `queued` | `queued` |
| active run `running` | `running` |
| active run `waiting_for_approval` | `waiting_for_approval` |
| active run `waiting_for_background` | `waiting_for_background` |
| active run `cancelling` | `cancelling` |
| `TaskRecord.hold` set (stop unacknowledged) | `waiting_for_background` |
| no active run and no hold | `idle` |

**Run states:** `queued` → `running` → `waiting_for_approval` | `waiting_for_background` → `finished` | `failed` | `cancelled` | `expired`. `cancelled` is terminal; continuing requires a new run (retry). `expired` is a terminal retention outcome; the client must read the verified snapshot, not fabricate a terminal result.

**Generation.** `generation` starts at `1` for a task and increments on every new run, retry and resume-after-reload. It is part of every event and cursor. An event with a generation lower than the receiving run's current generation is rejected as `stale_generation`; an event from another task/run is rejected as `cross_task_event`.

### 8.2 Run semantics (one task, many runs)

- **Follow-up** while idle starts a new run on the same task.
- **Follow-up** while active is a `queue`, `steer` or `force` message, not a new run.
- **Stop** requests cancellation at a safe boundary; it never kills an in-flight write. `run.cancelled` is emitted only after the host acknowledges the in-flight tool or the tool completes.
- **Retry** creates a new run and carries `retry_of_run_id` on `run.started`; it is allowed for `failed` with `retryable: true` and for `cancelled`, never for `finished`.
- **Resume** after reload loads the verified snapshot and continues the same run in a new generation; it never replays prior events as new output.

---

## 9. Browser runtime contract v1

This is a **new browser-owned contract**. It is not `crates/uor-r4-api`, not `ModelApi`, not the `uor-chat` OpenAI-compatible API, not Cursor's Cloud Agents API, and not a host-owned runtime. It may be implemented only after owner approval of this design.

### 9.1 Schema identifiers

| Constant | Value |
|---|---|
| Agent event schema | `uor-r4.studio.agent-event/1` |
| Agent error schema | `uor-r4.studio.agent-error/1` |
| Canonical JSON | `uor-r4.canonical-json/1` |
| Cursor token prefix | `uorse1` |
| Contract version | `1` (major); additive fields are minor-compatible |
| Schema dialect | JSON Schema draft 2020-12 |

### 9.2 Event envelope

```json
{
  "schema": "uor-r4.studio.agent-event/1",
  "event_id": "evt_run_7f3a2c1b9d4e6a80_2_000000000001",
  "project_id": "prj_4b1d9e0c7a2f6e51",
  "task_id": "tsk_9a3c5e7b1d2f4a60",
  "run_id": "run_7f3a2c1b9d4e6a80",
  "generation": 2,
  "sequence": 1,
  "recorded_at_ms": 1794283200123,
  "type": "run.started",
  "data": {
    "started_at_ms": 1794283200123,
    "model": { "id": "replay", "kind": "fixture" },
    "mode": "agent",
    "environment": { "kind": "local-browser" },
    "retry_of_run_id": null
  }
}
```

**Identity and scoping rules.**

- Every event is **run-scoped**: `run_id` is non-null, `generation >= 1`, `sequence >= 0`. There is no `task.created` event; task identity/metadata is a `TaskRecord`, and `TaskSummary` is a projection.
- `run.queued` is sequence `0`; `run.started` is sequence `1`; later events follow. A fixture that starts a run must contain both.
- `event_id` is deterministic: `evt_<run_id>_<generation>_<sequence padded to 12 digits>`. Same identity + same canonical digest = duplicate to ignore; same identity + different digest = `duplicate_event` and a hard refusal.
- `recorded_at_ms` is wall-clock for display only; ordering uses `sequence`, never time.
- `data` is event-specific JSON. Unknown `type` is preserved with opaque `data` and never turned into assistant text.
- Maximum event JSON size: **1 MiB**. Maximum events per run: **10,000**. Exceeding either is `limit_exceeded`, not truncation.

### 9.3 Canonical JSON and digests

`uor-r4.canonical-json/1` is a precise integer/ASCII-key subset, implemented identically in Rust and TypeScript:

1. UTF-8, no BOM, no insignificant whitespace.
2. Object keys are ASCII `[a-z0-9_.-]`, sorted by ASCII code unit; no duplicate keys.
3. Values are `null`, `true`, `false`, strings, arrays, or non-negative integers `0..=9007199254740991`. **No floats**, no `-0`, no NaN/Infinity, no bigints beyond the safe integer range.
4. Strings are JSON-escaped minimally: `"` → `\"`, `\` → `\\`, control characters `U+0000..U+001F` as `\u00XX`; other Unicode is emitted as UTF-8 (not `\u` escaped).
5. Arrays preserve order; objects are re-serialized with sorted keys.
6. `digest = "sha256:" + lower-hex(SHA-256(canonical_json(event)))`.

If a value cannot be canonicalized (float, non-ASCII key, unsafe integer), the event is rejected with `invalid_event`; the runtime never rounds or silently coerces. `event_id` is derived from run/generation/sequence, not from the digest, so both runtimes produce the same identity. Cross-runtime tests must show different insertion order produces the same digest.

### 9.4 Known event types and payloads

| Type | Data fields (required unless marked optional) |
|---|---|
| `run.queued` | `queued_at_ms` |
| `run.started` | `started_at_ms`, `model`, `mode: "plan" \| "agent"`, `environment`, `retry_of_run_id: string \| null` |
| `run.status` | `status`, `detail?` — **projection only; not authoritative terminal** |
| `run.finished` | `status: "finished"`, `finished_at_ms`, `duration_ms` — **authoritative terminal** |
| `run.failed` | `code`, `message`, `retryable`, `stop_outcome?`, `in_flight_tool_id?` — **authoritative terminal** |
| `run.cancelled` | `safe_boundary: true`, `stopped_at_tool_id?`, `finished_at_ms` — **authoritative terminal** |
| `run.expired` | `expired_at_ms`, `reason` — **authoritative terminal** |
| `turn.started` | `turn_id`, `index` |
| `turn.finished` | `turn_id`, `finish_reason: "stop" \| "length" \| "tool" \| "cancelled" \| "error"` |
| `assistant.delta` | `message_id`, `text_delta` |
| `assistant.message` | `message_id`, `text` |
| `thinking.delta` | `text_delta` (optional; tolerated) |
| `thinking.message` | `text` (optional; tolerated) |
| `plan.updated` | `plan_id`, `title`, `steps: [{ step_id, label, state: "pending" \| "active" \| "done" \| "blocked" }]` |
| `message.queued` | `message_id`, `delivery: "queue" \| "steer" \| "force"`, `boundary: "next_turn" \| "next_tool"?`, `text_preview` |
| `message.steered` | `message_id`, `boundary: "next_tool" \| "next_turn"` |
| `message.forced` | `message_id`, `boundary: "next_tool" \| "next_turn"` |
| `tool.requested` | `tool_call_id`, `tool`, `arguments_digest`, `arguments_preview` |
| `tool.approval_required` | `approval_id`, `tool_call_id`, `tool`, `scope`, `request_digest`, `expires_at_ms` |
| `tool.approval_resolved` | `approval_id`, `decision: "approved" \| "denied" \| "expired"`, `resolver: "host"` |
| `tool.started` | `tool_call_id` |
| `tool.completed` | `tool_call_id`, `status: "ok" \| "error" \| "cancelled"`, `output_digest`, `output_preview`, `truncated` |
| `tool.failed` | `tool_call_id`, `code`, `message` |
| `artifact.created` | `artifact_id`, `kind: "file" \| "image" \| "log" \| "link"`, `name`, `media_type`, `size_bytes`, `digest` |
| `changes.snapshot` | `scope: "last_turn" \| "branch"`, `files`, `head_digest`, `captured_at_ms` |
| `diff.comment` | `comment_id`, `path`, `line`, `body`, `head_digest` — local comment; remote comments deferred |
| `usage.updated` | `input_tokens`, `output_tokens`, `cache_read_tokens?`, `cache_write_tokens?`, `model_calls` |
| `heartbeat` | `nonce` (transport keepalive; tolerated) |
| `error` | `code`, `message`, `retryable` |

`run.status` is a UI projection. Reducers must not treat `run.status: "finished"` as terminal; only the authoritative terminal events set a terminal run state. `run.failed` with `stop_outcome: "ack_timeout"` means the host never acknowledged a stop; the task is `waiting_for_background`, not idle. `run.failed` with `stop_outcome: "host_refused"` means the host refused the stop; the task is `idle`.

### 9.5 Cursor token

```
uorse1.<b64url(project_id)>.<b64url(task_id)>.<b64url(run_id)>.<generation>.<sequence>
```

- Base64url is unpadded ASCII.
- `generation` and `sequence` are decimal `u64`.
- The token is opaque to the page; the client treats it as a string.
- A cursor is valid only for one `(task_id, run_id, generation)`. A cursor from another generation is `stale_generation`; from another task is `cross_task_event`.
- `last_event_id` is the event's `event_id`; the cursor token is the client's own resume position.

### 9.6 Error envelope and codes

```json
{
  "schema": "uor-r4.studio.agent-error/1",
  "code": "runtime_unavailable",
  "message": "the browser task runtime is not initialized",
  "retryable": false,
  "task_id": null,
  "run_id": null
}
```

`code` is one of:

`invalid_event`, `invalid_arguments`, `malformed_event`, `truncated_frame`, `duplicate_event`, `out_of_order`, `stale_generation`, `cross_task_event`, `cursor_expired`, `snapshot_corrupt`, `snapshot_missing`, `approval_forbidden`, `approval_expired`, `approval_digest_mismatch`, `task_not_found`, `run_not_found`, `task_archived`, `run_not_retryable`, `stop_unacknowledged`, `runtime_unavailable`, `host_unavailable`, `host_denied`, `model_unavailable`, `model_protocol_error`, `quota_exceeded` (future service only), `transport_closed`, `limit_exceeded`, `internal`.

`runtime_unavailable` means the browser runtime is not initialized; `host_unavailable` means the optional companion is not connected. They are distinct and must not be used interchangeably. `quota_exceeded` is future-service-only and must not appear in local-mode UI.

### 9.7 Approval identity and host tool intent/result

```json
{
  "approval_id": "apr_5c2e8a1f9b7d3e40",
  "task_id": "tsk_9a3c5e7b1d2f4a60",
  "run_id": "run_7f3a2c1b9d4e6a80",
  "generation": 2,
  "tool_call_id": "tc_02",
  "tool": "git.commit",
  "scope": "write",
  "request_digest": "sha256:6f1c...",
  "requested_at_ms": 1794283200123,
  "expires_at_ms": 1794283260123
}
```

**Rules.**

- The page may send only `{ approval_id, decision }` to the host. It may not send `granted: true`, a path, a command, or a scope.
- The host validates the approval ID, task/run/generation, `request_digest`, expiry and its own grant/approval state. It injects a host receipt; the page never sees or forges it.
- A forged, expired, cross-task or digest-mismatched approval is refused with `approval_forbidden` / `approval_expired` / `approval_digest_mismatch`.
- Approval TTL: **10 minutes** (`600_000 ms`) from `requested_at_ms`, configurable local policy (not a Cursor fact). A late decision is refused; the runtime emits `tool.approval_resolved` with `expired`.
- In static web, approval controls are disabled with: "This action needs host approval. Approve it in the local companion." No static boolean is authority.

**Runtime → host tool intent:**

```json
{
  "schema": "uor-r4.studio.host-intent/1",
  "intent_id": "hti_7f3a2c1b9d4e6a80_000000000004",
  "task_id": "tsk_9a3c5e7b1d2f4a60",
  "run_id": "run_7f3a2c1b9d4e6a80",
  "generation": 2,
  "tool_call_id": "tc_02",
  "tool": "file.read",
  "arguments": { "grant_id": "gr_1", "path": "src/a.ts" },
  "arguments_digest": "sha256:...",
  "approval_id": "apr_5c2e8a1f9b7d3e40"
}
```

**Host → runtime result/event:**

```json
{
  "schema": "uor-r4.studio.host-result/1",
  "intent_id": "hti_7f3a2c1b9d4e6a80_000000000004",
  "task_id": "tsk_9a3c5e7b1d2f4a60",
  "run_id": "run_7f3a2c1b9d4e6a80",
  "generation": 2,
  "tool_call_id": "tc_02",
  "status": "ok",
  "output_digest": "sha256:...",
  "output_preview": "…",
  "artifact": { "artifact_id": "art_...", "digest": "sha256:...", "media_type": "text/plain", "size_bytes": 12 },
  "approval_receipt": "hmac-sha256:...",
  "occurred_at_ms": 1794283200456
}
```

The runtime validates the result against the intent, task/run/generation, digest and approval receipt, then emits `tool.completed`/`tool.failed`/`artifact.created`. Host results are **not** events; the runtime owns event creation. `artifact` is optional at the top level; a file-read result carries it so `agent_read_artifact` can return bounded, digest-verified bytes for the editor. Tool `arguments` use the canonical subset in §9.3; a float or non-ASCII key is rejected with `invalid_arguments` before an intent is sent.

### 9.8 Transport matrix

| Client | Runtime location | Host tool intents | Host → runtime stream | Must not use |
|---|---|---|---|---|
| **Static browser** | In-page Web Worker | `NullHostTransport` (explicit `host_unavailable`) | In-page runtime events only | EventSource, host fetch, terminal/Git/MCP stdio |
| **Tauri webview** | In-page Web Worker | `agent_host_invoke` | `agent_subscribe` `Channel<HostToolResult>` when available; `agent_host_poll` fallback for the offline invoke shim | EventSource; AgentEvent channel; Channel assumption in the shim |
| **Loopback browser** | In-page Web Worker | `POST /v1/studio/host/intents` | `GET /v1/studio/host/results` `application/x-ndjson` via `fetch` + `ReadableStream`, token header | EventSource; remote/TLS; second listener |

**Loopback rules.** Add the routes to the existing loopback Studio listener and its exact `Host`, `Origin`, `Content-Type` and per-launch token checks. No new listener, no remote bind, no TLS server. The stream is NDJSON, not SSE. `Last-Event-ID` is not used; the cursor is a query parameter. A `410`/`cursor_expired` response forces snapshot resync. The stream carries `HostToolResult`, never `AgentEvent`; the browser runtime creates events.

**Tauri rules.** `agent_host_invoke` accepts a `HostToolIntent` and returns/streams a `HostToolResult`. `agent_subscribe` uses `tauri::ipc::Channel<HostToolResult>` where the GUI exposes it. The offline invoke shim does not expose a Channel; `agent_host_poll` returns results after a cursor. No host runtime process, no host task store. All `AgentClient` data calls (`tree`, branch `changesSnapshot`, host `readArtifact`) go through `HostClient.execute`/`agent_host_invoke`; no component or client calls `window.__TAURI__.core.invoke` directly. The loopback browser uses the same existing token/Origin-protected POST path for data tools.

### 9.9 Compatibility rules

- Additive fields are allowed in a minor version; clients ignore unknown fields.
- No event type is removed or renamed without a major schema bump.
- Unknown event types are preserved and ignored by the reducer.
- The generated TypeScript file carries the schema digest; `agent_contract_is_generated_from_schema` fails if the checked-in file and the Rust schema disagree.
- The schema is exported by `crates/uor-r4-studio-agent` and generated by `scripts/studio/generate-agent-contract.mjs`; there is no hand-edited TS event union.
- TypeScript canonicalization is generated from the same schema; the Rust and TS digests must match on the same fixture.

---

## 10. Browser runtime semantics and persistence

### 10.1 Runtime surface (TypeScript)

`web/studio/src/agent/runtime.ts` owns:

```ts
export interface AgentRuntime {
  listTasks(input?: ListTasks): Promise<TaskSummary[]>;
  createTask(input: CreateTask): Promise<TaskSummary>;
  startRun(input: StartRun): Promise<RunSummary>;
  queueMessage(input: QueueMessage): Promise<MessageReceipt>;
  steer(input: SteerMessage): Promise<MessageReceipt>;
  forceSend(input: ForceSend): Promise<MessageReceipt>;
  pinTask(input: PinTask): Promise<TaskSummary>;
  archiveTask(input: ArchiveTask): Promise<TaskSummary>;
  stop(input: StopTask): Promise<StopReceipt>;
  retry(input: RetryRun): Promise<RunSummary>;
  resolveApproval(input: ResolveApproval): Promise<ApprovalReceipt>;
  resumeTask(input: ResumeTask): Promise<TaskSnapshot>;
  subscribe(input: Subscribe): Promise<EventStream>;
}
```

`AgentStore.apply(event)` is **synchronous** and pure; it validates and reduces an event. Async canonicalization/digest/store append is a separate `appendEvent` path and is awaited by the caller/tests. Do not make `apply` async.

**Model port (TypeScript, worker-side):**

```ts
export interface ModelPort {
  open(request: ModelRequest): AsyncIterable<ModelStreamEvent>;
}
export type ModelStreamEvent =
  | { type: 'text-delta'; text: string }
  | { type: 'tool-call'; toolCallId: string; tool: string; arguments: unknown }
  | { type: 'usage'; inputTokens: number; outputTokens: number; modelCalls: number }
  | { type: 'finished'; reason: 'stop' | 'length' | 'tool' | 'cancelled' | 'error' }
  | { type: 'failed'; code: string; message: string; retryable: boolean };
```

`ReplayModelPort` is the only implementation in the executable stages. It is clearly labelled synthetic. The accepted M1 artifact is prediction-only; `WasmModelPort` is not a task brain and is not wired as one. `R4ModelPort` is `NOT_RUN` until a generation-capable artifact and Claude's merged streaming/tool contract exist. The `uor-chat` text stream is not a tool-call contract.

### 10.2 Queue, steer, force, stop, retry

- **Queue:** append FIFO; processed at the next turn boundary. `message.queued` carries `delivery: "queue"` and a bounded `text_preview`.
- **Steer:** delivered at the next safe tool boundary. If the run finishes before a boundary, the fallback order is exactly `message.steered { boundary: "next_turn" }` → `message.queued { delivery: "steer", boundary: "next_turn" }` → `run.finished`; the queued message then starts a **new run with a new generation** (`run.queued` sequence 0, `run.started` sequence 1). No event is appended after the terminal event of the old run.
- **Force:** bypasses the queue and appends to the most recent user turn; it is delivered at the next safe tool boundary. If the run finishes before that boundary, the fallback order is exactly `message.forced { boundary: "next_turn" }` → `message.queued { delivery: "force", boundary: "next_turn" }` → `run.finished` → new run with a new generation; never append after the old run's terminal event, and never interrupt an in-flight tool mid-write.
- **Stop:** sets `cancel_requested`; emits `run.status` `cancelling`; forbids new tool dispatch; waits for the host's in-flight tool acknowledgement; emits `tool.completed`/`tool.failed` for the in-flight call; then `run.cancelled` with `safe_boundary: true`. If the host does not acknowledge within `STOP_ACK_TIMEOUT_MS` (30 s), emit `run.failed` with `code: "stop_unacknowledged"`, `stop_outcome: "ack_timeout"`, `in_flight_tool_id` and `retryable: false`; the task is `waiting_for_background`. If the host refuses, emit `run.failed` with `code: "host_denied"`, `stop_outcome: "host_refused"`; the task is `idle`. Never claim cancellation without ack.
- **Retry:** allowed for `failed` with `retryable: true` and for `cancelled`; creates a new run and generation; `run.started` carries `retry_of_run_id`. A `finished` run is not retryable.
- **Stop at safe boundary** is a hard acceptance test: no later tool event may appear after `run.cancelled`, and no later tool may be dispatched after a terminal stop outcome.

### 10.3 Hostile input rules

- **Malformed JSON / partial frame:** buffer until a complete NDJSON line; at EOF emit `truncated_frame`; never apply partial data.
- **Duplicate event:** same `event_id` + same canonical digest → ignore; same `event_id` + different digest → `duplicate_event` and stop applying that stream.
- **Out of order:** `sequence <= last_applied` → duplicate/gap handling; `sequence > last_applied + 1` → `out_of_order`; client must resync from snapshot, not apply later events.
- **Unknown event:** preserve, do not render as assistant text; `thinking`/`heartbeat` are known optional and tolerated.
- **Stale generation:** reject; do not let an old run mutate a new run's transcript.
- **Cross-task:** reject; one task's events can never affect another.
- **Approval forgery:** reject; no page-supplied boolean, path, scope or command.
- **Cursor expiry:** `410`/`cursor_expired` → snapshot resync; no fabricated completion.
- **Corruption:** snapshot digest/chain mismatch → `snapshot_corrupt`; refuse to restore; keep the last verified event log.

### 10.4 IndexedDB persistence

Database `uor-r4.studio.agent/1` (versioned; additive migrations only):

| Store | Key | Value |
|---|---|---|
| `tasks` | `task_id` | `TaskRecord { task_id, project_id, title, repo?, branch?, created_at_ms, archived, pinned_local, latest_run_id?, hold?: { reason: "waiting_for_background"; run_id; tool_call_id } }` |
| `runs` | `[task_id, run_id]` | `RunRecord { task_id, run_id, generation, state, retry_of_run_id: string \| null, started_at_ms, ended_at_ms?, last_sequence, last_event_id, last_digest }` |
| `events` | `[task_id, run_id, generation, sequence]` | `EventRecord { event_id, canonical, digest, prev_digest, recorded_at_ms }`; index `by_event_id` |
| `snapshots` | `[task_id, run_id]` | `SnapshotRecord { schema, task_id, run_id, generation, last_event_id, last_sequence, head_digest, created_at_ms }` |
| `artifacts` | `artifact_id` | `ArtifactRecord { artifact_id, media_type, size_bytes, digest, blob? }` |
| `approvals` | `approval_id` | `ApprovalRecord { approval_id, task_id, run_id, generation, tool_call_id, request_digest, expires_at_ms, decision? }` |
| `changes` | `[task_id, run_id, head_digest]` | `ChangeRecord { task_id, run_id, scope: "last_turn", files, head_digest, captured_at_ms }` |
| `comments` | `[task_id, path, line, head_digest]` | `DiffComment` |
| `leases` | `task_id` | `LeaseRecord { task_id, writer_id, started_at_ms, heartbeat_at_ms }` |

- **Per-run keyed event records**, not one object-store blob for the whole run. Each event record stores its canonical string and digest; `prev_digest` chains the run. A snapshot manifest stores `last_event_id`, `last_sequence` and `head_digest`.
- **Canonical NDJSON export/import** is a bounded, explicit user action; import verifies every digest and the chain before applying. Export is not the runtime's authority.
- **Writer lease:** acquire in a readwrite transaction; `started_at_ms` is set once, `heartbeat_at_ms` is refreshed while the runtime is alive. A lease is stale when `now - heartbeat_at_ms > 30_000`; recovery reads the snapshot and verifies `head_digest` before writing. Do not compare only `started_at_ms`; use the heartbeat for liveness and `started_at_ms` for display/ordering.
- **Atomicity:** IndexedDB transactions are atomic per store; a multi-store write (event + run + snapshot) uses one readwrite transaction. Do not duplicate `Store::save` atomic rename semantics or require one Store to hold 10,000 1 MiB events.
- **Optional companion export:** if the Tauri companion exports a snapshot, it is a **bounded** export through the existing object store (≤8 MiB, one snapshot), explicitly user-initiated. It is not the browser runtime's task store or authority.
- **Save/reload proof:** tests use a persistent IndexedDB fixture transport (or a real browser profile) for reload/cursor/approval checks; in-memory fixtures are not restart proof.
- **Changes snapshot:** the browser runtime records a `changes.snapshot` event and a `changes` record when a write tool completes; `scope` is `last_turn`, `head_digest` binds the file set. `agent_git_diff` is **branch-only**; it never fabricates a last-turn diff.
- **Test API gate:** `window.__studioTest` exists only when `VITE_STUDIO_TEST=1` **and** the URL has `?studioTest=1`. Default/production builds do not expose it; a negative test asserts absence. The Playwright webServer uses `VITE_STUDIO_TEST=1 npm run build`, and test-only specs navigate with `?studioTest=1`.

### 10.5 Limits (UOR policy, not Cursor values)

| Limit | Value |
|---|---|
| Event JSON | 1 MiB |
| Events per run | 10,000 |
| Snapshot | 8 MiB |
| Image attachment | 10 MiB each |
| Document attachment | 4 MiB each |
| Attachments per message | 5 |
| Total attachments per message | 25 MiB |
| Approval TTL | 10 minutes, configurable local policy |
| Stop acknowledgement | 30 seconds |
| Steer/force boundary wait | 30 seconds |
| Lease stale threshold | 30 seconds since heartbeat |

---

## 11. UI and UX

### 11.1 Information architecture

```
Task rail (left)                  Task detail (center)                 Work surface (right/tab)
  New task                          Header: status, prompt, repo         Changes
  Search                            Run status / plan card               PR
  Pins (This device)                Transcript (assistant, tool,         Artifacts
  Today / Last 7 days / Older         approval, error, usage)            Terminal (companion)
  Archived (collapsed)              Composer                             Editor (Stage 4)
```

The task rail is the navigation; the detail is transcript-first. The work surface is a tabbed panel, not a second app. On narrow screens the rail becomes a drawer and the work surface replaces the detail (with a clear back control).

### 11.2 Protected baseline selectors

The typed shell must keep the baseline test surface valid:

- `nav[aria-label="Studio"]`
- `textarea[aria-label="Prompt"]`
- a visible `Send` button
- `.agent-workspace`, `.agent-transcript`, `.agent-composer`
- `.workspace-grid`, `.pane-dock`, `.session-list`
- `[data-control=…]`, `[data-pane-target=…]`, `[data-action="toggle-projects"]`
- the existing single skip link (retarget it at typed mount; do not add a second)

The new task-workspace class is `.task-workspace`, not `.studio-shell`. Do not rename existing selectors to fit the redesign. The existing `AppShell` threshold stays as-is; the task rail's narrow behavior is a drawer: below 1024 it is hidden only when the drawer is closed, `[data-action="toggle-projects"]` reopens it, and exactly 1024 does not auto-collapse. The test asserts: **1023 closes and reopens; 1024 does not auto-collapse**. At 1024 a user may still close the rail manually; that is not an automatic collapse.

### 11.3 Task rail

- Search by task title, project, repo and status. Search is local and bounded.
- Pins: local-device pins labelled **"This device"** in the first stages. Account-scoped pins only after a service identity exists. Do not claim the desktop 75-pin limit; the web limit is unverified.
- Groups: `Today`, `Last 7 days`, `Older`; Older collapsed. This is the release-note grouping, not the marketing mock's "This Week / This Month".
- Archive is reversible and does not delete events. No permanent Delete control in the task UI; deletion is an API/service concern and its UI status is unknown in Cursor. Do not invent a Logs tab, Pause button or per-task dollar cost.

### 11.4 Task detail

- Header: status, starting prompt, project/repo, branch, environment, model. Status is a text label, not a color-only pill.
- Run status: `queued`, `running`, `waiting for approval`, `waiting for background`, `cancelling`, `finished`, `failed`, `cancelled`, `expired`. A `waiting` state names what it waits for.
- Plan card: a UOR event (`plan.updated`) with steps `pending/active/done/blocked`; it is not the desktop Plan pill.
- Transcript: assistant deltas collapse into one message; tool calls show request/completion pair; approval cards show tool, scope, expiry and host identity; errors show code, message and `Retry` only when retryable; usage shows tokens and model calls only if emitted. No per-task dollar cost.
- Stop is a button while a run is active; it shows `cancelling` until the host acknowledges. No Pause.
- Checkpoints: no hosted Restore Checkpoint control. If a future source establishes one, it is a new design.

### 11.5 Composer

- Reuse `createComposerInput` for the existing keyboard/IME/paste/history behavior. Only add the missing **image/document reader** behavior; do not rewrite the input.
- The textarea keeps `aria-label="Prompt"`; the visible `Send` button remains. Additional `Queue`, `Steer`, `Force send`, `Stop` controls are grouped and labelled; `Send` maps to queue-when-active / send-when-idle.
- `Enter` queues (or sends when idle); `Shift+Enter` inserts a newline at the caret; `Cmd/Ctrl+Enter` force; `Cmd/Ctrl+Shift+Enter` steer; `Escape` closes menus/dialogs and returns focus.
- Attachments: paste, drag/drop and picker; images and documents; memory/IndexedDB-local in local mode; exact size/type errors before reading; no ZIP.
- `@` context: editor file, task attachments, task artifacts, recent tasks. It does not enumerate arbitrary host paths; host paths require a companion grant.
- `/` skills: local templates and skills; attach to one message; no claim of Cursor's skill catalog.
- Pickers: model, mode (`Plan` / `Agent`), environment (`Local browser`, `Local companion` only when connected). Disabled options are explicit, not fake.

### 11.6 Work surface

- **Changes:** diffs from `AgentClient.changesSnapshot(taskId, scope)`; `ChangeSnapshot` is `{ scope: "last_turn" | "branch", files, head_digest }`, and `head_digest` binds the file set and comment identity. Scopes `Last agent turn` and `Branch` are UOR task views, not a claim that Cursor web exposes the desktop Changes scopes. `Last agent turn` comes from the browser runtime's `changes.snapshot`; `Branch` comes from the companion `agent_git_diff` (branch-only) or the static uploaded-file diff. Do not fabricate a host last-turn diff. Per-file expansion, line comments via local `diff.comment` events with stable task/file/line/head-digest identity; remote comments are deferred to the service appendix.
- **PR:** link/read-only PR summary in the first stages; no merge button without a service identity and GitHub integration. A PR tab must not claim Cursor's five-sub-tab memory on web.
- **Artifacts:** `AgentClient.readArtifact(artifactId)` reads a browser artifact from IndexedDB first and falls back to the companion `agent_read_artifact` for host-side artifacts; it returns bytes/metadata and verifies `digest`; no nonexistent artifact URL. Preview is inert; expired links are not reused.
- **Terminal:** companion-only; explicit unsupported state in static web; existing PTY enable + exact-command approval; shell owns `Ctrl+K`/`Ctrl+F` inside the terminal.
- **Editor:** hidden until Stage 4; then a UOR text editor with file tree via `AgentClient.tree`, tabs, dirty/save, bounded search. No LSP/debugger/extensions/themes/notebooks claim.

### 11.7 Account, admin, billing, integrations

Do not render fake menus. No account/profile/team/billing/SSO/SCIM/audit UI before a service identity exists. Local settings are labelled "Local settings". No X Subscription or legacy Cursor cohort names, prices or plan tiers anywhere in UOR. GitHub-first integration and other providers are in the gated service appendix, not this plan.

### 11.8 Visual tokens

Use UOR tokens, not Cursor's fonts, colors or assets. Existing UOR base: dark `#141413`, light `#faf9f5`, accent `#d97757`, blue `#6a9bcc`.

| Token | Dark | Light |
|---|---|---|
| `--uor-canvas` | `#141413` | `#faf9f5` |
| `--uor-surface` | `#1c1b19` | `#ffffff` |
| `--uor-surface-raised` | `#242320` | `#f1efe8` |
| `--uor-border` | `#33312c` | `#d9d5c9` |
| `--uor-text` | `#f5f3ee` | `#1f1e1a` |
| `--uor-muted` | `#a8a49a` | `#6b675e` |
| `--uor-accent` | `#d97757` | `#b85a3c` |
| `--uor-focus` | `#f5f3ee` | `#1f1e1a` |
| `--uor-radius-sm/md` | `4px` / `8px` | same |

Rules: near-black/near-white canvas, one surface step, hairline borders, 4/8 px card radii, one accent for focus/selection, no shadows as the primary hierarchy, strong 2 px focus ring with 2 px offset. Public marketing measurements are not signed-in app evidence; do not copy CursorGothic, cursorSans, berkeleyMono or Cursor assets.

### 11.9 Responsive, keyboard, IME, accessibility

- **Viewports:** 390, 1024, 1440 CSS px. No horizontal overflow. The task rail collapses **below 1024**; at exactly 1024 it does **not auto-collapse** (the user may still close it manually). Single column with sticky composer below 520; work surface replaces detail on narrow screens with a back control.
- **Layout:** CSS Grid for the shell, Flexbox for rows, container queries for the detail/composer, media queries for global layout and `prefers-reduced-motion`.
- **Keyboard:** scoped shortcuts only. No global `Tab`; no global `Cmd+K`. The palette button is primary; the UOR `Cmd/Ctrl+Shift+P` binding is scoped to the app shell and is a UOR choice, not a Cursor fact. Install `legacyShortcutGuard` in the protected baseline **before** the typed workflow wait/render: it closes/cancels any existing legacy palette, clears `#command-palette`/`#palette`, and suppresses later `Meta/Ctrl+K`/hash opens. A delayed legacy `setTimeout(150)` open must not beat the guard. The 1023 drawer is reopened only by `[data-action="toggle-projects"]`.
- **IME:** reuse `createComposerInput`; existing composition handling is **regression acceptance**, not a new red test. Add genuinely new dead-key, RTL paste, long-paste and caret-preservation tests with expected failures where the current behavior is insufficient. `Shift+Enter` newline; never submit on composition confirmation.
- **Accessibility:** landmarks (`header`, `nav`, `main`, `aside`); one `h1` per view; correct heading order; the existing single skip link (retargeted, not duplicated); `aria-live="polite"` for run status; `role="alert"` for errors; `aria-busy` on the transcript while streaming; `aria-expanded` on menus; listbox/option for `@` and `/`; focus return after dialogs; visible focus; no `aria-hidden` on focusable content; 24×24 minimum targets, 44×44 on coarse pointers; 16 px mobile form text; reduced motion honored.

### 11.10 Error, empty and unsupported states

| Situation | Required state |
|---|---|
| No runtime initialized | "The task runtime is not initialized. Reload the page." (`runtime_unavailable`) |
| No companion | "The local companion is not connected. File and terminal tools are unavailable." (`host_unavailable`) |
| Static terminal | "Interactive terminal requires the local companion. This browser build cannot start a shell." |
| Static Git | "Git requires the local companion. Uploaded files can still be reviewed locally." |
| Account | "Accounts and team settings require an optional service. Local settings are stored on this device." |
| Background/cross-device | "Background and cross-device tasks require an optional service. Closing this tab stops local work." |
| Approval without host | "This action needs host approval. Approve it in the local companion." |
| Editor depth | "UOR text editor. Language services, debugger, extensions and notebooks are not available." |
| Model unavailable | "The selected model source is not loaded. Load an artifact or choose a source." |
| Malformed event | "The event stream contained an invalid event. The transcript was not changed." |
| Cursor expired | "The stream expired. Reload the task snapshot to continue." |

No error state may fabricate assistant text, a tool result or a completion.

---

## 12. Privacy and security

### 12.1 Threats and controls

| Threat | Control | Test |
|---|---|---|
| Prompt/reply leak from local mode | No network at load; model worker local; attachments memory/IndexedDB-local; no telemetry | `static_browser_makes_zero_external_requests_on_load` |
| Agent secret in `localStorage` | No agent secret storage in page; future service stores/injects; documented `modelSources.v1` provider-key exception only | `no_agent_secret_is_written_to_local_storage` |
| Browser holds a signing key | Signed commits only future service/host; no key in page | `signed_commit_requires_service` (appendix) |
| Forged approval | Approval identity + host digest + expiry; page sends only ID/decision | `forged_approval_is_refused_by_host` |
| Cross-task authority | Events/approvals keyed by task/run/generation; host checks | `agent_host_refuses_cross_task_approval` |
| Path traversal / symlink | Existing canonicalized grants and symlink refusal; no new path API | `agent_host_refuses_out_of_grant_path` |
| Hostile event stream | Strict validation; no fabrication; resync | `rejects_malformed_json_frame`, `tolerates_unknown_event_type_without_assistant_text` |
| Replay / duplicate | Deterministic event ID; canonical digest duplicate rule | `accepts_duplicate_event_with_same_digest` |
| Loopback token leak | Existing token/Origin checks; no EventSource URL token | `loopback_stream_requires_token_header` |
| Webhook spoof (future service) | HMAC signature verification, timestamp window, replay cache | `github_webhook_rejects_bad_signature` (appendix) |
| Unauthenticated endpoint | Existing loopback token + exact origin; no public route | `no_unauthenticated_route_is_registered` |

### 12.2 Companion boundary

The companion is optional. It executes only tools the runtime requested and the host approved. It does not own the task loop, task state, event log or cursor. A companion failure yields `host_unavailable`; the browser task loop remains usable for local model/review.

### 12.3 Service boundary (appendix only)

A full hosted service is a separate future owner item with its own design, threat model, auth provider, deployment target and security review. It is not authorized here. It may never become a second local host authority or a public unauthenticated agent endpoint.

---

## 13. Testing strategy

**Red first where behavior is new.** Every executable task writes the failing test, runs it, records the expected failure, implements the minimum, reruns, and commits. Existing behavior (notably IME composition handling in `createComposerInput`) is **regression acceptance**, not a fabricated red test. New dead-key/RTL/long-paste/caret cases are red only where the current behavior is insufficient.

**State contract.** `AgentStore.apply(event)` is synchronous; `appendEvent`/IndexedDB writes are async and are awaited by callers/tests. `retry_of_run_id` is `string | null` everywhere. Test-only APIs require `VITE_STUDIO_TEST=1` and `?studioTest=1`; default builds expose none.

| Layer | Location | What it proves |
|---|---|---|
| Contract | `crates/uor-r4-studio-agent/tests/contract_events.rs` | Schema identity, envelope, cursor, approval, error codes, canonical JSON, JSON example |
| Canonical cross-runtime | `crates/uor-r4-studio-agent/tests/canonical_json.rs`, `web/studio/src/agent/canonical.test.ts` | Same digest across Rust/TS, insertion-order independence |
| Browser store | `web/studio/tests/e2e/cursor-store.spec.ts` | IndexedDB save/reload, per-run records, digest chain, corruption/truncation refusal |
| Browser runtime | `web/studio/src/agent/runtime.test.ts` | run.queued/started, queue/steer/force/stop/retry, terminal vs status |
| Hostile | `web/studio/src/agent/ingest.test.ts` | Malformed/partial/duplicate/out-of-order/unknown/stale/cross-task/410 |
| Acceptance | `web/studio/src/agent/acceptance.test.ts` | Two live tasks, persistent reload, safe stop, approval forgery |
| Companion data | `src-tauri/tests/agent_data.rs` | `agent_list_directory`, `agent_git_diff`, `agent_read_artifact`, grants, digest |
| Companion authority | `src-tauri/tests/agent_authority.rs` | Tool intents/results, approvals, forgery, cross-task, no task runtime |
| Loopback browser | `src-tauri/tests/agent_loopback.rs` | fetch NDJSON stream, token header, no EventSource, exact origin |
| Components | `web/studio/src/components/*.test.tsx` | Exact UI behavior, protected names, accessible states |
| Browser E2E | `web/studio/tests/e2e/cursor-*.spec.ts` | Actual DOM, keyboard/IME/a11y, 390/1024/1440, static unsupported |
| Legacy | Existing `shell/visual/workflows/terminal` specs and Vitest suite | `?legacy=1`, `?typed=0`, `#app`, `#studio-typed-root`, protected selectors |

**Tauri commands.** Run from the repository root with `cargo test --manifest-path src-tauri/Cargo.toml --features gui --test <target>` (and default-feature variants where appropriate). Do not use `cargo test -p uor-r4-desktop-client` from the root workspace; `src-tauri` is a separate workspace.

**Real model and publication are separate.** The accepted M1 artifact is prediction-only. No shell/runtime test uses it as a task brain. `R4ModelPort` and useful-agent evaluation are `NOT_RUN` until a generation-capable artifact and Claude's merged streaming/tool contract exist. A prediction-only top-8 measurement is a separate appendix row, not useful-agent evidence. Production publication is a separate owner decision.

---

## 14. Coverage matrix — Cursor feature families

**How to read this.** `PLANNED` means the design and a named test exist in the plan; it is not executed. `NOT_RUN` means no implementation is authorized or no service exists. `UNVERIFIED` means Cursor's own behavior is not established; UOR must not claim it. `NOT A UOR TARGET` means we deliberately do not build it. This matrix is the answer to "cover every menu/input" without inventing private Cursor behavior.

**Evidence class.** `U` = Rust unit/contract or TS unit/component test; `H` = Tauri host/integration test; `E` = Playwright browser E2E; `L` = live model/API evaluation; `N` = no implementation authorized / `NOT_RUN`; `V` = Cursor behavior unverified. A row's class is the strongest evidence its planned test can reach; no row is `L` at plan time.

| # | Feature family | Cursor surface / evidence | UOR target | Class | Planned evidence | Plan-time status |
|---|---|---|---|---|---|---|
| 1 | Hosted access and auth gate | Web; `[DOC]` WorkOS redirect; signed-in IA `[UNV]` | Future service appendix only | N/V | `auth.rs`; `tenant.rs` (future proposed) | NOT_RUN / UNVERIFIED |
| 2 | Task list, search, pins, groups | Web/AW; `[DOC]` web search/source filter; AW groups/pins; 75-pin desktop `[DOC]`, web limit `[UNV]` | Stage 1 shell + Stage 2 data | U/E | `TaskRail.test.tsx`; `cursor-shell.spec.ts` | PLANNED |
| 3 | Task creation / composer | Web/AW/IDE/iOS; `[DOC]` prompt, images, docs, `@`, `/`, model/mode/env; real placeholder `[UNV]` | Stage 1 | U/E | `ComposerSurface.test.tsx`; `cursor-ime.spec.ts` | PLANNED |
| 4 | Queue / steer / force-send / stop / retry | Web/AW/IDE; `[DOC]` queue, force, steer, stop; `Tab` default `[UNV]` | Stage 2 | U/E | `runtime.test.ts`; `cursor-tasks.spec.ts` | PLANNED |
| 5 | Transcript and run stream | Web/AW/iOS; `[DOC]` shared transcript, stream events; `thinking`/`heartbeat` `[DOC]` | Stage 2 | U/H/E | `contract_events.rs`; `ingest.test.ts`; `TaskTimeline.test.tsx` | PLANNED |
| 6 | Plan card / Worked / Plan pill / status pills | AW mostly; `[DOC]` desktop release stream; web `[UNV]` | Stage 2 plan events; no Plan pill claim | U/E, V | `TaskTimeline.test.tsx` | PLANNED / UNVERIFIED for pill |
| 7 | Subagents / side chats / child tasks | Web/AW/iOS; `[DOC]` nested inline; web `[UNV]` | Not in first slice | N | — | NOT_RUN |
| 8 | Checkpoints / restore | IDE/Agent; `[DOC]` local; hosted `[UNV]`, no source | None | V/N | `cursor-review.spec.ts` asserts no UOR control | NOT A UOR TARGET (hosted) |
| 9 | Changes scopes | AW; `[DOC]` Last Agent Turn, branch, picked commits; web `[UNV]` | Stage 4 UOR last-turn + branch | U/E, V | `DiffReview.test.tsx` | PLANNED; picked commits UNVERIFIED |
| 10 | PR review / merge | Web/AW/iOS; `[DOC]` web review/merge, AW five sub-tabs; web parity `[UNV]` | Stage 4 link/read-only; service appendix for integration | U/E, N | `PrPanel.test.tsx` | PLANNED / merge NOT_RUN |
| 11 | Terminal | Web remote PTY; AW; IDE local; `[DOC]` | Stage 3 companion; static unsupported | H/E, N | `agent_authority.rs`; existing `interactive_terminal.rs`; `cursor-static.spec.ts` | PLANNED; web remote PTY NOT_RUN |
| 12 | Cloud desktop | AW; `[DOC]` desktop only | None | N | — | NOT A UOR TARGET |
| 13 | Artifacts | AW/API; `[DOC]` cards/download; web cards `[UNV]` | Stage 4 `readArtifact` | U/E | `ArtifactPanel.test.tsx` | PLANNED |
| 14 | Editor, file tree, tabs | Web/AW/IDE; `[DOC]` web/AW tree; depth `[UNV]` | Stage 4 UOR editor | U/E, V | `FileTree.test.tsx`; `EditorPane.test.tsx` | PLANNED; depth UNVERIFIED |
| 15 | LSP / debugger / extensions / themes / notebooks | IDE; `[DOC]`; AW LSP per blog; web `[UNV]` | None | N | — | NOT A UOR TARGET |
| 16 | Command palette / keyboard | IDE/AW; `[DOC]` `Cmd+K` AW, `Cmd+Shift+P` IDE; web `[UNV]` | Stage 1 scoped UOR shortcuts | E | `cursor-keyboard.spec.ts` | PLANNED; no global Cmd+K/Tab |
| 17 | Voice chat / dictation | Web/AW/IDE/iOS; `[DOC]` | None in this plan | N | — | NOT_RUN |
| 18 | MCP servers | Web/AW; `[DOC]` HTTP/stdio; SSE conflict `[UNV]` | Stage 3 companion | H, V | `mcp_protocol.rs`; `agent_authority.rs` | PLANNED (existing transport; task integration); SSE UNVERIFIED |
| 19 | Secrets / env vars | Dashboard; `[DOC]` three secret types, injection/redaction | Future service appendix only | N | `secrets.rs` (future proposed) | NOT_RUN |
| 20 | Automations / subscriptions / CI autofix | Cloud/service; `[DOC]` Teams/Enterprise; 180-day subscriptions | Future service appendix only | N | `background.rs`; `notifications.rs` (future proposed) | NOT_RUN |
| 21 | GitHub integration | Web/service; `[DOC]` App install, clone/branch/PR/checks | Future service appendix, GitHub-first | N | `github.rs` (future proposed) | NOT_RUN |
| 22 | Other providers (GitLab, Bitbucket, Azure, Slack, Teams, Jira, Linear, Notion, Tailscale) | Service; `[DOC]` varying betas/plans | Later, not planned | N | — | NOT_RUN |
| 23 | Account / admin / billing / SSO / SCIM / audit / privacy | Dashboard/Enterprise; `[DOC]`; nav `[UNV]` | Future service appendix only | N | `auth.rs`; `tenant.rs`; `secrets.rs` (future proposed) | NOT_RUN |
| 24 | iOS / Android / API / CLI / SDK / Origin / Apps | Separate surfaces; `[DOC]` | None; API contract is UOR's own | N | — | NOT A UOR TARGET |
| 25 | Visual tokens / responsive / a11y | Marketing/docs `[OBS]`; signed-in app `[UNV]` | Stage 1 + Stage 5 | U/E | `cursor-a11y.spec.ts`; `cursor-shell.spec.ts` | PLANNED |
| 26 | Self-hosted / managed workers | Service; `[DOC]` per-agent VMs, pools, My Machines | Future service appendix only | N | future proposed (no executable test) | NOT_RUN |
| 27 | Model / browser privacy | Web local model; `[INF]` from UOR baseline | Stage 2 + Stage 5 | U/E | `static_browser_makes_zero_external_requests_on_load`; `agentClient.test.ts` | PLANNED |
| 28 | Pause / Delete / Logs / per-task cost | No documented UI; `[UNV]` | None | V | `cursor-review.spec.ts` asserts no invented UOR controls | UNVERIFIED (not documented; UI status unknown) |
| 29 | Hosted checkpoint restore | No documented hosted feature; `[UNV]` | None | V/N | `cursor-review.spec.ts` | UNVERIFIED / NOT A UOR TARGET |
| 30 | Signed commits / installation tokens / webhooks | Service/VM; `[DOC]` | Future service appendix only | N | `github.rs`; `secrets.rs` (future proposed) | NOT_RUN |

**What this matrix deliberately does not do.** It does not turn a Cursor family into a UOR requirement merely because it exists. It does not claim a desktop-only control on web. It does not invent a Pause/Delete/Logs/per-task-cost UI. It does not claim LSP/debugger/extensions/themes/notebooks. It does not claim Cursor's authenticated UI. UOR-only bindings (palette, pins label, local comments) are labelled as UOR choices.

---

## 15. Verification queue (optional, future, read-only)

No credentials, account, task or repository are used now. If a later owner-approved read-only walkthrough happens, it is evidence for the design, not a requirement of this plan. Capture only public/authorized screens; do not mutate a task, repo or account; store captures under `local/kimi/cursor-hosted-research/` (ignored) and do not commit proprietary assets.

| # | Question | What it would change |
|---|---|---|
| V1 | Authenticated `/agents` IA, default view, empty state, real placeholder, ARIA/focus | Task rail/detail/composer layout; `G1` |
| V2 | Same task on Web vs Agents Window: which Worked/Plan pill/Changes scopes/PR memory/artifact/merge controls exist on web | Whether any Stage 2/4 control is promoted or omitted; `G2` |
| V3 | Web editing depth: tree, tabs, LSP, debugger, themes, extensions, notebooks, split view | Stage 4 editor scope; `G3` |
| V4 | Hosted checkpoint search after 2026-11-09 | Whether a future checkpoint row exists; `G4` |
| V5 | Pause/Resume, Delete, Logs, per-task cost UI | Whether these are real controls or only documentation gaps; `G5` |
| V6 | Per-account plan cohort and billing page | Whether any cohort language is safe; `G6` |
| V7 | MCP SSE against a cloud agent | Whether SSE is supported or remains unverified; `G7` |
| V8 | Remote Control plans/admin/privacy | Whether a companion-like remote mode is relevant; `G8` |
| V9 | Account/admin nav order, icons, responsive | Future service IA; `G9` |
| V10 | Keyboard + JAWS/VoiceOver + CJK IME + 200% zoom + reduced motion | A11y acceptance details; `G10` |

**Freshness.** The Cursor snapshot is 2026-11-09. Re-check release notes after any Cursor release; treat older UI claims as historical. A missing source remains `UNVERIFIED`, not "absent".

---

## 16. Effort, cost and risk

**No paid compute in this unit.** The plan is docs-only. Implementation effort below is in **focused sittings**, not calendar promises and not a total-project guarantee.

| Stage | Scope | Effort | Security/review gate |
|---|---|---|---|
| 0 | Contract, canonical JSON, generated TS, protected baseline | 6–10 sittings | Contract review |
| 1 | Shell, task rail, composer media, a11y/IME regression | 6–11 | Accessibility/IME review |
| 2 | Browser runtime, IndexedDB, two workers, task UI | 13–18 | Runtime/security review |
| 3 | Companion data APIs, tool intents, loopback stream | 8–12 | **Separate host-authority review** |
| 4 | File tree/editor, diff/comments, PR/artifacts | 5–7 | Path/grant review |
| 5 | Final local acceptance | 3–5 | Final exact-head review |
| **Total** | **23 tasks** | **41–63 sittings** | Excludes service appendix and model compute |

**Risks.** Contract/canonical drift between Rust and TS; IndexedDB corruption or cross-tab races; capability theater in static web; host authority leakage; cross-task races; IME regression; legacy shortcut/hash collision; dependency creep; real API absence. Mitigations are the named tests and gates in the plan.

**Stop conditions.** Stop before: weakening an approval/identity check; adding an undeclared dependency; exposing a public unauthenticated endpoint; using real credentials, profiles or paid services; deleting or rewriting legacy behavior; changing owner metadata; claiming hosted production, full Cursor parity, useful-agent generation or a working R4 adapter from a fixture.

---

## 17. Execution handoff

1. **Owner review of this revised design.** Record owner approval in a new ORDER. No code before that. The lead cannot substitute a delegated approval for this gate.
2. **Fresh exact-head delivery review.** Two independent builder checks returned **REVISE**; neither approved the work. Delivery reviewer `agent-94` returned **CHANGES** on the prior report head for research-record consistency, and `agent-95` returned **CHANGES** for staged-table/audit/record arithmetic. Those corrections are **applied in this PR; fresh delivery review pending**. Do not dispatch an unnamed third builder check. The remaining review gate is a **fresh exact-head delivery review**, followed by owner acceptance/design approval. No approval is implied by this revision.
3. **Choose execution.** Subagent-driven (recommended: fresh implementer + fresh reviewer per task; interface-heavy work benefits from independent checks) or native (one session, one whole-branch review at the end).
4. **Execute the plan.** Each executable stage has checkbox tasks and a stage acceptance. The appendices have no checkboxes and are not executable tasks.
5. **Deliver.** Exact-head review, protected merge, fresh-main verification, delete branch/worktree. Service and real-R4 work remain gated.

---

## 18. Assumptions (lead rulings from the builder check)

These are **lead rulings**, not user decisions. They resolve the builder's unanswered defaults and are binding for this revision.

1. **Architecture A.** The task runtime, reducer, cursor and durable task persistence live in the page/Web Worker. IndexedDB is the task/event store. There is no host-owned task runtime.
2. **Companion optional.** The Tauri companion executes approved tools and holds grants/approvals; it never owns the task loop. Static browser supports the local model, task loop, transcript and user-mediated files; real terminal/Git/MCP stdio requires the companion.
3. **Service separate.** A full hosted service is a separate future owner item/security review and outside this plan.
4. **M1 is prediction-only.** The accepted M1 artifact is not the task brain. Shell/runtime tests use clearly labelled synthetic/Replay fixtures. Real R4 generation/useful-agent evaluation is `NOT_RUN` until a generation-capable artifact and Claude's real streaming/tool contract exist. A prediction-only top-8 measurement is a separate appendix row, not useful-agent evidence.
5. **Owner-only approval.** The owner must approve this design before code. The lead cannot delegate this approval for this ORDER.
6. **Local pins** are labelled **"This device"** until a service identity exists.
7. **Editor tab** is hidden until the review/editor stage (Stage 4). It is not part of the first shell.
8. **Per-run event directories** are represented as per-run keyed records under one IndexedDB task store; not one object-store blob for the whole task.
9. **No host runtime process.** The in-process-vs-child question is resolved: there is no host runtime; the browser owns the loop.
10. **Force-send** is delivered only at the next safe boundary; if the run ends first, it becomes a queued follow-up. The UI labels this clearly.
11. **Approval TTL** is 10 minutes as configurable local policy, not a Cursor fact.
12. **Static shell persistence:** settings and drafts persist until the browser runtime exists; then real IndexedDB task/event records are used. The static shell never writes a fake task log.
13. **Separate owner service item** is required for any account, team, secrets, background, webhook or integration work.
14. **Fetch streaming** is the loopback transport; no EventSource. Sequence 0 `run.queued` fixtures are mandatory.
15. **UOR-only bindings** (palette, local pins, local comments) are UOR choices, not Cursor facts.

**Remaining owner-only decisions for owner approval:** two independent builder checks returned **REVISE**; delivery reviewers `agent-94` and `agent-95` returned **CHANGES** on research-record, staged-table/audit and record-arithmetic issues. None approved this revision. The remaining gates are a new fresh exact-head delivery review and owner acceptance/design approval; no third builder check is pending. The owner must decide whether to accept this architecture revision; whether the browser-owned IndexedDB store is the right long-term durability boundary; whether the optional companion should be packaged at all in the first implementation unit; and the owner acceptance item. The recommended default is a **new Cursor-hosted Studio item** with browser-owned-runtime acceptance, leaving #3569's Rust-runtime `Done when` untouched; the alternative is an explicit owner ORDER superseding that `Done when`. This revision does not edit the old acceptance text, and no implementation is authorized until the owner records one of those choices.
