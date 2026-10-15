# #193 — feat(ui/wasm): Web Worker offloading and WASM event-loop isolation for non-blocking browser UI

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-26T01:31:43Z
- **Updated:** 2026-07-26T01:54:49Z
- **Closed:** 2026-07-26T01:54:49Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/193
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: Master Roadmap Epic
Relates to: #160 (runtime-side execution safety), #184 (typed integration facade).
Source: `index.html`; `docs/r4_graph_compiler_implementation_plan.md` (Phase 5 WASM demotion/isolation).

## Motivation

Today, `index.html` calls WASM methods (`window.wasm_module.generate_r4g1_response`) directly on the browser's main DOM thread. Because candidate graph scoring and vector bundling are CPU-intensive operations, executing them synchronously on the main thread locks DOM rendering, freezes user input, and degrades the UI experience.

To ensure non-blocking execution while maintaining zero-copy browser performance, all browser WASM inference must be offloaded to a dedicated Web Worker (`r4_worker.js`) via asynchronous `postMessage` event passing.

## Scope

- **Web Worker Architecture**: Create `r4_worker.js` to initialize the WASM module (`uor_r4_wasm_router.js` / `.wasm`) off the main thread.
- **Asynchronous Protocol**: Implement message handlers for `INIT_ENGINE`, `GENERATE_RESPONSE`, `CANCEL_GENERATION`, and `PROGRESS_EVENT`.
- **UI Integration in `index.html`**: Update DOM logic to communicate with `r4_worker.js` via `postMessage`, displaying loading/streaming states without locking the main thread.
- **Fallback Path**: Maintain standard REST/WebSocket fallback when WASM initialization fails or is disabled by user settings.

## Non-Goals

- Modifying the underlying WASM API signatures or `UorR4Router` Rust code.
- Adding GPU/accelerator requirements to browser execution.

## Affected Crates / Modules / Files

- `index.html`
- `r4_worker.js` (new)
- `pkg/` (WASM output targets)

## Deliverables

1. `r4_worker.js` Web Worker module handling WASM instantiation and background execution.
2. `index.html` refactored to delegate WASM computation to `r4_worker.js`.
3. Non-blocking UI verification: DOM animations and input handlers remain responsive (60fps) during active generation.

## Acceptance Criteria

- Main DOM thread never freezes during active WASM generation; verified via Chrome DevTools Performance trace (0 long tasks > 50ms on main thread).
- `r4_worker.js` handles WASM module load errors gracefully and signals `index.html` to fall back to HTTP REST/WS.
- Four gates green (`cargo test`, `cargo clippy`, `cargo fmt`, `no_std`).

## Formal Claim Classification

- "WASM inference in the browser executes without locking the main DOM thread": **Guarantee**, status **Witnessed** (Browser E2E test).

## Test Requirements

- **Unit / Integration:** WASM worker message parsing and state handling tests.
- **BDD / Cucumber:** Scenario covering Web Worker initialization and non-blocking generation fallback.

## Memory & Determinism Considerations

- Web Worker memory stays isolated in worker heap context.
- Identical seed input to WASM produces identical output string tokens across workers.


## Comments (1)

### comment 5081405783 — 2026-07-26T01:32:00Z — Casey-allard

Parent Master Epic: #198


---
_Comment/review ids in this file: 5081405783_

