# LexLean Studio Tools Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Repair silent Studio model-loading failures and expose UOR's existing exact geometric arithmetic as a truthful shared tool in the browser and Tauri clients.

**Architecture:** One bounded Rust parser/formatter calls `GeometricArithmeticEngine`; thin WASM and Tauri RPC adapters return its result without invoking a model. The desktop transport returns an OpenAI-compatible JSON fallback with zero model usage, while ordinary prompts keep the existing worker/sidecar paths.

**Tech Stack:** Rust, wasm-bindgen, existing browser Worker JavaScript, Tauri IPC, Vitest, Node tests, existing Python/runtime verifiers.

**Spec:** `docs/loop/tasks/m7-lexlean-studio-tools/kimi-1/ORDER.md` and `DESIGN.md`.

## Global Constraints

No LexLean default, second runtime, heuristic corpus, new dependency, committed binary, paid compute, fake token usage or useful-chat claim. Do not modify `.github/**`, model catalogs/weights, `openai.rs`, `loopback.rs`, sidecar binaries or `Cargo.lock`. Every production behavior starts with a failing regression.

## Review Focus

- Lookalike prose must fall through to the model rather than being hijacked by an arithmetic parser; cover in Task 1.
- Overflow, unsigned subtraction underflow, overlong prompts and malformed operands must fail loudly; cover in Task 1.
- Tool responses must not fabricate completion IDs, token rates or model-generated attribution; cover in Tasks 2 and 3.
- Ordinary synthetic generation and native identity must remain unchanged; cover in Tasks 2 and 3.
- A stale/mismatched worker result must become an error without marking a newer or different artifact ready; cover in Task 2.

---

### Task 1: Shared exact arithmetic tool

**Files:**
- Modify: `crates/uor-r4-integer/src/m6_exact_arithmetic.rs`

**Interfaces:**
- Produces: `pub fn exact_arithmetic_tool(prompt: &str) -> Option<Result<ExactArithmeticToolAnswer>>`; the answer exposes exact `expression`, `result`, and formatted `text` fields. Ordinary prompts return `None`; recognized invalid requests return `Some(Err(...))`.

- [ ] **Step 1: Write failing tests** for `17 + 25`, `483 + 259`, a borrow case, number-first/trace labels, non-arithmetic fallthrough, malformed operands and underflow.
- [ ] **Step 2: Run the focused test** with `$HOME/.cargo/bin/cargo test -p uor-r4-integer --lib exact_arithmetic_tool`; expect failure because the public function/type is absent.
- [ ] **Step 3: Implement the strict parser and answer formatter** using `GeometricArithmeticEngine`, bounded operands/prompt length and actual `PlaceStepRecord` values.
- [ ] **Step 4: Rerun the focused test**, then the arithmetic and integer-library suites; expect PASS with no changed existing expectations.

### Task 2: Browser loading and tool behavior

**Files:**
- Modify: `crates/uor-r4-integer/src/wasm_bindgen_exports.rs`
- Modify: `web/studio/assets/js/uor_model_worker.js`
- Modify: `web/studio/src/adapters/modelSourceAdapter.ts`
- Test: `web/studio/verify_studio_worker.mjs`
- Test: `web/studio/src/adapters/modelSourceAdapter.test.ts`
- Test: `web/studio/assets/js/model_sources.test.mjs`

**Interfaces:**
- Consumes: Task 1's shared function.
- Produces: WASM `exactArithmetic(prompt) -> JSON`; worker `generate_complete.tool === 'm6-exact-arithmetic'` with `tokenIds: []`; explicit artifact identity errors.

- [ ] **Step 1: Add failing worker, adapter and default-selection regressions.** Require exact arithmetic, ordinary fallthrough, zero model token IDs, and explicit filename/hash mismatch errors.
- [ ] **Step 2: Run each regression and verify the intended RED result** before production edits.
- [ ] **Step 3: Add the WASM export and thin worker branch**, then dispatch explicit mismatch errors without completing a pending operation incorrectly.
- [ ] **Step 4: Rerun focused checks and all browser suites**; verify a real generated `42` and an ordinary synthetic response.

### Task 3: Tauri exact-tool transport

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `web/studio/assets/js/desktop_openai.js`
- Test: `src-tauri/tests/desktop_client_controls.rs`
- Test: `web/studio/assets/js/desktop_openai.test.mjs`

**Interfaces:**
- Produces: `native_exact_arithmetic(request_json) -> {handled, text?, expression?, result?}`; handled requests return an OpenAI-compatible JSON response with `uor.tool`, zero usage and no model-token IDs; unhandled requests continue to the sidecar.

- [ ] **Step 1: Add failing RPC and desktop-transport tests** for handled arithmetic, fallthrough, zero usage and preservation of the existing `uor-chat` model id.
- [ ] **Step 2: Verify RED** with the focused Rust and Node commands.
- [ ] **Step 3: Implement the host RPC and desktop JSON fallback**, leaving the sidecar and normal streaming unchanged.
- [ ] **Step 4: Rerun focused and complete default/GUI matrices**, then build and exercise the packaged app.

### Task 4: Records and protected delivery

**Files:**
- Modify: `docs/products/web-client.md`, `docs/products/desktop-client.md`, `docs/research/notebook/m4-serving.md`
- Modify: `docs/loop/tasks/m7-lexlean-studio-tools/kimi-1/REPORT.md`

- [ ] **Step 1: Record the measured LexLean rejection scope, exact behavior, tests and remaining limits** without changing milestone status or model claims.
- [ ] **Step 2: Run every declared check and inspect the diff/stat**, generated files and disk use.
- [ ] **Step 3: Obtain fresh exact-head adversarial review, queue the protected PR, verify fresh main, publish Pages with the existing five-card catalog, and remove generated outputs.**
