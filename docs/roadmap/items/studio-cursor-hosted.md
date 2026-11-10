---
id: studio-cursor-hosted
title: Deliver the Cursor-style browser-owned Studio task workspace
milestone: M7
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: []
done_class: host
scope: [Cargo.toml, Cargo.lock, crates/uor-r4-studio-agent/**, scripts/studio/generate-agent-contract.mjs, web/studio/package.json, web/studio/package-lock.json, web/studio/playwright.config.ts, web/studio/index.html, web/studio/src/studio.css, web/studio/src/bootstrap.ts, web/studio/src/agent/**, web/studio/src/state/agent.ts, web/studio/src/state/agent.test.ts, web/studio/src/styles/**, web/studio/src/hooks/**, web/studio/src/components/**, web/studio/src/adapters/composerInput.ts, web/studio/tests/e2e/**, web/studio/README.md, src-tauri/Cargo.toml, src-tauri/Cargo.lock, src-tauri/src/lib.rs, src-tauri/src/main.rs, src-tauri/src/agent_data.rs, src-tauri/src/agent_host.rs, src-tauri/src/agent_server.rs, src-tauri/tests/agent_data.rs, src-tauri/tests/agent_authority.rs, src-tauri/tests/agent_loopback.rs, src-tauri/capabilities/default.json, docs/loop/tasks/studio-cursor-hosted/**, docs/roadmap/items/studio-cursor-hosted.md, docs/roadmap/issues.json, docs/superpowers/specs/2026-11-09-cursor-hosted-studio-upgrade-design.md, docs/superpowers/plans/2026-11-09-cursor-hosted-studio-upgrade.md, docs/products/harness-parity.md, docs/products/web-client.md, docs/products/desktop-client.md, docs/integration/current-state.md]
---

## Why it matters

The owner approved autonomous implementation of the revised Cursor-hosted Studio plan. The useful next product increment is a clean, responsive task workspace whose durable task loop works locally in the browser, with an optional desktop companion for explicitly approved tools. This is a new local-runtime item, not a reinterpretation of the older hosted/Rust-runtime acceptance.

## What is missing

The existing neutral Studio provides the protected typed shell and native PTY foundation, but not a durable multi-task agent workspace. Missing are the shared Rust/TypeScript event contract, one typed task list and send path, searchable local task rail, attachment composer, browser-owned queue/steer/force/stop/retry/resume state machine, digest-verified IndexedDB persistence, host-approved tool intents, and real review/editor data flows. The companion needs grant-bounded directory, branch-diff and artifact-read APIs and a nonblocking loopback transport.

The accepted M1 model is prediction-only. Executable work uses labelled `ReplayModelPort` fixtures; real R4 agent generation and useful-model evaluation are not available. A hosted account/service, production deployment and full Cursor parity are separate, unimplemented work.

## Done when

- All 23 tasks in the approved plan have their required red-first tests and stage acceptance checks passing. The versioned event/error contracts and canonical JSON agree across Rust and TypeScript, including insertion-order-independent digests, hostile input refusal, and explicit `run.resumed` semantics without duplicate start markers.
- One typed task list and one runtime-owned composer send path provide the responsive UOR task shell, local search/pins/archive, accessible media/document attachments and queue/steer/force/stop controls. Existing selectors, IME behavior, `?legacy=1`, `?typed=0`, skip-link behavior and the exact 1023/1024 rail boundary pass regression checks.
- The page/Web Worker exclusively owns task/run/event/cursor state. IndexedDB `uor-r4.studio.agent/1` persists bounded per-run records, canonical digest chains, verified snapshots, changes and comments. Parallel tasks, reload/resume, corruption/quota recovery and lease behavior pass their declared checks; an empty task list is not a runtime failure.
- The optional Tauri companion only executes approved tools. Grant-bounded directory, branch-only diff and digest-verified artifact APIs, nonblocking loopback transport, and native approval authority pass default/GUI host tests. A page cannot approve its own intent; receipts are opaque host capabilities or asymmetrically verifiable, with no page HMAC secret. Static web explicitly reports unsupported host functions.
- Review/editor workflows use real browser changes, companion/uploaded branch diffs, local comments, and inert digest-verified artifacts. Read-only PR metadata is local/imported or companion-supplied without credentials or a page-initiated network request; static web labels live GitHub PR information unsupported. No merge, LSP, debugger, extension or hosted-checkpoint capability is implied.
- The test-only API requires both opt-ins and is absent from the default build; page load makes zero external requests and new agent secrets never enter `localStorage`. The existing provider-key store remains unchanged and is not copied into IndexedDB.
- Independent task and exact-head reviews, required GitHub checks, protected merges and fresh-main verification complete. Local acceptance evidence and a handoff identify actual executed checks and every `NOT_RUN`/`UNVERIFIED` row.

The older `studio-ide-harness` acceptance (#3569) remains open and unchanged, including its same-Rust-runtime, hosted-service and broader requirements; this item does not satisfy them. Hosted services/integrations, real R4 generation, useful-model evaluation, production publication and full Cursor parity are `NOT_RUN`, not implied by fixture or exact-arithmetic checks.

## Cancel if

The approved browser-owned runtime can be supplied by an existing current implementation without duplicating state authority, or the base already contains the complete item. A probe finding only missing UI hooks or tests is not cancellation. Any need for a public service, real credentials, model/API changes, a second task-state owner, or weakening protected legacy behavior requires a new order/scope decision rather than silent expansion.

## How to start

Read the approved [design](../../superpowers/specs/2026-11-09-cursor-hosted-studio-upgrade-design.md), [23-task plan](../../superpowers/plans/2026-11-09-cursor-hosted-studio-upgrade.md), and the [parent registration record](../../loop/tasks/studio-ide-harness/kimi-19/ORDER.md). After official issue sync, read the active `docs/loop/tasks/studio-cursor-hosted/` ORDER created at the merged item base; do not recreate a self-item registration ORDER. Execute serially from the contract and protected baseline through shell/composer, browser runtime/persistence, optional companion transport, review/editor and final local acceptance. Run the real loopback streaming experiment before choosing NDJSON or polling. Do not start either gated appendix.

Registration found no other open PR touching Studio source, Tauri or product-document paths. Later #3569 work and this item's active ORDERs must be disjoint on overlapping paths; the loop's scope/claim checks enforce that coordination without changing #3569's status or acceptance.

## Rules that apply

- Do not edit `crates/uor-r4-api/**`, model/engine/worker/tokenizer implementations, `services/**`, `.github/**`, rendered STATE/HANDOFF, #3569, the owner's checkout, or another lab's work.
- Keep `LifecycleStore` for legacy shell/session/model state and `AgentStore` as the sole task/run/event/cursor owner. The host never owns the task loop or task store.
- No new default runtime dependency, Electron, Monaco, node-pty, EventSource, second listener, public endpoint or real credential. Do not give the page a host MAC key.
- Preserve all unique evidence and local work. No production publish, destructive cleanup, paid compute or capability claim beyond executed host/browser checks.

## Sources

- Owner direction: "I approve of this plan, please implement it end to end autonomously" (Kimi chat, 2026-11-09).
- The revised Cursor-hosted design and implementation plan; merged research/design PR #3912.
- Existing `studio-ide-harness` (#3569), neutral Studio/PTY delivery and protected typed-shell regression suites.
