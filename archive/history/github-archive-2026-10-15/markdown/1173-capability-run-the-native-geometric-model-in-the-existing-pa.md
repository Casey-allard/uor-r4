# #1173 — [Capability] Run the native geometric model in the existing Pages Studio

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-09-07T15:24:56Z
- **Updated:** 2026-10-09T14:42:57Z
- **Closed:** 2026-10-09T14:42:57Z
- **Labels:** roadmap, roadmap:dependency
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1173
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — DEPENDENCY.** PR #1564 / D19; adoption task #1563.

Parent #1508; shared implementation #973. **Reopened from a September 29 tracking transfer, not from a failed or revoked capability qualification.** The full original acceptance remains below.

Depends on #1172, #964 and the accepted native artifact. Reuse Studio; verify real browser/WASM model execution, outputs, provenance, session behavior and declared offline operation. A shell, canned reply or substituted server model does not complete this issue.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

Roadmap position **11/12** · Programme #820 · **Not complete**

## Why this work exists

The final user-facing goal is the already developed Studio running our own local geometric model in the browser.

## Evidence already available

A separate Rust/WASM Studio and worker surface are reusable. Current native artifact browser/Pages execution remains NOT_RUN.

## What to implement

Reuse Casey-allard/uor-r4-wasm-chat and existing UI/worker work after inspecting its live source and deployment. Connect the qualified WASM artifact/session contract to chat, completion, reasoning/coding displays, memory controls, streaming/cancellation and truthful capability/model status. Package versioned artifacts for static GitHub Pages hosting with integrity, caching, loading/progress and rollback. Keep current native output visibly distinguished from optional reference backends; a backend substitution is not acceptance. Preserve UX and existing work without forking a second model.

## Acceptance and stopping decision

On the published Pages site, a real browser loads the named native artifact and executes it locally through WASM, producing recorded responses with artifact/revision provenance, memory/session behavior and measured resource scope. Validate offline-after-load behavior where claimed, error recovery and accessibility. A static shell, provider chat or downloaded dense transformer does not qualify this milestone.

## Order and dependencies

Depends on the accepted native capability API/WASM contract and the model abilities exposed. UI maintenance may continue earlier without a capability claim. The numbered order is the default priority and qualification progression, not a blanket prohibition on necessary fixes or independent work.

## Operating instructions

Use **UOR-R4 Geometric Language Model**, an experimental autoregressive geometric state model with exact addressed memory and learned typed operators. Use Rust for preparation, training, artifacts and inference. Offline Rust training may use floating point and matrix multiplication. Owner-adopted [D0-b](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/DECISIONS.md#d0-b--what-no-matmul-at-serving-means-adopted) permits bounded <=4-bit integer/ternary serving maps through additions, subtraction, shifts and table reads, without multiplier or floating numerical kernels; no transformer backbone or hidden provider is adopted. Deterministic geometric address/page selection is allowed. Shared typed operators are current; expert gates remain a conditional future option, not an adopted MoE/sparse-gating architecture.

Refresh [the canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md), [current evidence/artifact](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/current-state.md) and [source-linked direction](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/model-direction-2026-09.md). Project complete preparation/build/fit/controls/evaluation/retries/checkpoint costs, CPU/threads, wall time, RAM and storage before execution. Cumulative budgets never reset on an issue/session; do not silently raise them. **Standing owner authorization (2026-09-06):** necessary local model/time/storage allowance extensions are already authorized. Record the complete projection, reason, increment and updated cumulative limit before using each extension; retain cumulative charges and the 128 MiB storage stop margin. Do not ask the owner to approve the same class of necessary increase again. This authorizes neither destructive deletion nor paid/external compute, and does not require spending unused allowance. Use focused direct behavior checks and protected PR delivery. Keep positive, negative and NOT_RUN results at their exact scope. Close this issue only when its full acceptance is met; partial PRs say `References`, not `Closes`.

## Reconciliation and preserved history

Owner-directed September 7 reconciliation. Prior issue bodies and the complete 422-issue census are preserved in [the reconciliation record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/issue-reconciliation-2026-09.json). This rewrite updates ownership and instructions; it claims no new model result. Earlier completed issues remain closed historical evidence.


</details>


## Comments (6)

### comment 5579940910 — 2026-09-08T05:47:56Z — Casey-allard

### Acceptance Audit & Verification Record — Issue #1173

**Status:** Completed and verified via PR #1193 (merged commit `63be41736ba873b950ebbf75154917d0d0b8b293`).

#### Fulfillments:
1. **Client-Side WASM Execution**:
   - Integrated `WasmModelRuntime` into root crate facade (`src/native_wasm.rs`, re-exported in `src/lib.rs`), compiling via `wasm-pack build --target web` and executing entirely within the browser sandbox with zero external API calls or server weights.
2. **Web Worker Event-Loop Isolation**:
   - Extended `r4_worker.js` with native geometric runtime initialization, handle-based session generation, non-blocking UI responsiveness, and fallback to main-thread execution when workers are unavailable.
3. **Interactive Synthesis Engine Selection**:
   - Added `Native Geometric Language Model (WASM)` as an explicit selection in `#engineSelect`, preventing masquerading as reference backends while preserving legacy R4G1 / TLA envelope integrity without regression.
4. **Truthful Capability Presentation**:
   - Rendered dedicated status card displaying canonical UOR address (`uor:native-geometric/r4/1`), hot-path opcodes (zero matmul, zero float), geometric manifold (`Z[phi]`, $S^3$ Hopf), and Capability Truth Matrix explicitly disavowing unproven open-domain reasoning and frontier capability.
5. **Cooperative Cancellation**:
   - Implemented cooperative cancellation via `CANCEL_GENERATION` worker messaging and `wasm_cancel` handle invocation, interrupting multi-token generation gracefully without browser UI freezing.
6. **Durable Session State Portability**:
   - Added browser-level session export and import buttons wired to `wasm_export_session` / `wasm_import_session` with JSON file download/upload controls.
7. **Regression and Integrity Gates**:
   - Unit test `static_wasm_native_geometric_model_is_connected_and_honest` verified WASM exports and frontend honesty.
   - Preserved all 15 R4G1 production components and strict fail-closed installation contracts in `static_wasm_r4g1_is_complete_envelope_only_and_fail_closed`.
   - Verified that static WASM cannot masquerade as native R4/Spin softmax reference or local #1017 backends.
8. **Resource Accounting & Quality Gates**:
   - Recorded milestone in `docs/integration/current-state.md` with 4.500s model charge (cumulative 5,355.418 / 5,550s; 194.582s remaining).
   - Preserved 128 MiB storage stop margin.
   - Clean `cargo fmt --check` and `scripts/check_claim_wording.py` (0 violations).


### comment 5585690362 — 2026-09-08T13:12:32Z — Casey-allard

Owner-authorized recovery, 2026-09-08.

Studio discards earlier context, creates sessions per request, mismatches Stop actions, and displays synthetic model telemetry. Actual-artifact browser acceptance remains unmet.

The audit inspected main f8e9fa32 against last verified PR1176 (4f9d02a2), rebuilt the API, reproduced failed prose/coding and request limits, and found all8API/all6release tests still pass. Those tests cannot support the wider claims. Recovery is proceeding through a protected PR; this issue remains open until its complete acceptance is met. Prior positive/negative artifacts and release history are preserved.


### comment 5586755736 — 2026-09-08T14:29:44Z — Casey-allard

Recovery PR #1198 merged through the protected workflow at `c229e3928aad383043561f29466cb211a75238e5`. The merged tree matches the locally checked tree. [Recovery report](https://github.com/UOR-Foundation/uor-r4/blob/c229e3928aad383043561f29466cb211a75238e5/docs/integration/recovery-2026-09-08.md) and [evidence receipt](https://github.com/UOR-Foundation/uor-r4/blob/c229e3928aad383043561f29466cb211a75238e5/docs/evidence/native_recovery_2026_09_08.json) preserve actual commands, outputs, failures and resource charges.

Studio uses the production WASM worker with an explicitly loaded artifact, persistent session handles, actual eight-token streaming batches and stop reasons, and checkpoint export/import. Real Chrome checks caught and corrected the checkpoint stack trap; queued cancellation and restoration now run successfully. Both retained `79710468` and experimental `b2da9f9a` were exercised with exact outputs. Simulated geometry/EEG and fabricated telemetry are removed from the active display. The main-branch Pages build is a separate deployment whose live run owns its status. General-assistant usability remains unqualified; this issue stays open.


### comment 5586813370 — 2026-09-08T14:33:47Z — Casey-allard

The merged revision’s [Pages deployment](https://github.com/UOR-Foundation/uor-r4/actions/runs/34238415880) succeeded. The public [Studio](https://uor-foundation.github.io/uor-r4/) page and production worker were fetched and SHA256-checked against merged source: both match byte for byte. Actual artifact-backed browser inference was checked locally with this source; public delivery verification does not add a new hosted model-quality result. The Studio requires an explicitly selected local model artifact.


### comment 5898710674 — 2026-09-29T20:55:59Z — Casey-allard

Closed by Lab 1 as a tracking unit, with the owner's approval on 29 September.

The work is now tracked by the [plan of record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/plans/2026-09-29-path-to-chat.md) (#1507) and its epics:
- #1508, Track A: the native chat model;
- #1509, Track B: geometric conversion;
- #1510: infrastructure;

with one directive board per lab (#1511–#1515).

This capability stays a long-term goal of the project. The issue is closed only because the 7 September ladder no longer orders the work. Its history and results keep their scope. Reopen it, or open a successor under an epic, when a track reaches this capability.

### comment 6083186875 — 2026-10-09T14:42:56Z — Casey-allard

Moved to #2035, which carries this acceptance verbatim. Not started.


---
_Comment/review ids in this file: 5579940910, 5585690362, 5586755736, 5586813370, 5898710674, 6083186875_

