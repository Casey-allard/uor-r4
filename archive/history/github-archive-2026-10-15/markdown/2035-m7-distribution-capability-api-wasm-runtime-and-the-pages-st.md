# #2035 — [M7] Distribution: capability API, WASM runtime and the Pages Studio

- **Type:** Issue
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-09T14:41:47Z
- **Updated:** 2026-10-13T02:52:23Z
- **Labels:** roadmap, milestone, status:not-started
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/2035
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Part of the roadmap tracker #2028.

Expose the accepted native artifact through one capability API (library/CLI/service/WASM, with isolation, cancellation and error semantics) and run it in the browser in the existing [Pages Studio](https://github.com/Casey-allard/uor-r4-wasm-chat). No canned replies; verify the actual browser, backend, artifact and output.

### Current state
- A local Rust API exists (capability API tests in uor-r4-integer).
- WASM qualification hasn't started.
- No Studio deployment of the native model.

<details><summary>Acceptance and history carried over verbatim from #1172</summary>

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — DEPENDENCY.** PR #1564 / D19; adoption task #1563.

Parent #1508; shared implementation #973. **Reopened from a September 29 tracking transfer, not from a failed or revoked capability qualification.** The full original acceptance remains below.

Needed local session/save-load interfaces may be built now under #962/#973; broad API/WASM qualification follows an accepted model and #964. Exercise actual artifacts across declared library/CLI/service/WASM surfaces with isolation, cancellation, error semantics and truthful capability reporting. Browser work is not a gate before useful local memory.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

Roadmap position **10/12** · Programme #820 · **Not complete**

## Why this work exists

One coherent model must expose its actual abilities to applications before Studio integration can be meaningful.

## Evidence already available

Library/CLI/service and separate WASM/Studio infrastructure exist at distinct scopes; current 321e990f root CLI and Studio execution are NOT_RUN.

## What to implement

Define and implement a common artifact/session contract across Rust library, CLI/service and WASM. Cover loading/validation, input, completions and streaming, stopping/cancellation, typed errors, model identity/capabilities, context/memory operations and resource limits. Capability metadata must truthfully distinguish implemented primitives from qualified language, attention, reasoning and coding. Reuse crates/uor-r4-api and the actual native core instead of wrapping historical compile/teacher interfaces and calling that model integration. Bring the accepted artifact and serving operators through a browser-compatible memory/serialization path with measured parity. Transfer interface obligations from #962 and #1084 here.

## Acceptance and stopping decision

The accepted native artifact produces matching bounded outputs/state through the named library/CLI/service/WASM surfaces, with tests for loading, cancellation/errors and session isolation. Browser execution has no hidden remote/source-model response dependency. Document supported capabilities and limits; an OpenAI-shaped endpoint alone proves none.

## Order and dependencies

Start interface contracts when useful; final parity consumes an accepted native model. API completion precedes claiming the integrated Pages product. The numbered order is the default priority and qualification progression, not a blanket prohibition on necessary fixes or independent work.

## Operating instructions

Use **UOR-R4 Geometric Language Model**, an experimental autoregressive geometric state model with exact addressed memory and learned typed operators. Use Rust for preparation, training, artifacts and inference. Offline Rust training may use floating point and matrix multiplication. Owner-adopted [D0-b](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/DECISIONS.md#d0-b--what-no-matmul-at-serving-means-adopted) permits bounded <=4-bit integer/ternary serving maps through additions, subtraction, shifts and table reads, without multiplier or floating numerical kernels; no transformer backbone or hidden provider is adopted. Deterministic geometric address/page selection is allowed. Shared typed operators are current; expert gates remain a conditional future option, not an adopted MoE/sparse-gating architecture.

Refresh [the canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md), [current evidence/artifact](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/current-state.md) and [source-linked direction](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/model-direction-2026-09.md). Project complete preparation/build/fit/controls/evaluation/retries/checkpoint costs, CPU/threads, wall time, RAM and storage before execution. Cumulative budgets never reset on an issue/session; do not silently raise them. **Standing owner authorization (2026-09-06):** necessary local model/time/storage allowance extensions are already authorized. Record the complete projection, reason, increment and updated cumulative limit before using each extension; retain cumulative charges and the 128 MiB storage stop margin. Do not ask the owner to approve the same class of necessary increase again. This authorizes neither destructive deletion nor paid/external compute, and does not require spending unused allowance. Use focused direct behavior checks and protected PR delivery. Keep positive, negative and NOT_RUN results at their exact scope. Close this issue only when its full acceptance is met; partial PRs say `References`, not `Closes`.

## Reconciliation and preserved history

Owner-directed September 7 reconciliation. Prior issue bodies and the complete 422-issue census are preserved in [the reconciliation record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/issue-reconciliation-2026-09.json). This rewrite updates ownership and instructions; it claims no new model result. Earlier completed issues remain closed historical evidence.


</details>


</details>

<details><summary>Acceptance and history carried over verbatim from #1173</summary>

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


</details>

### How to work on this issue
- Claim with one comment: lab, goal, expected cost; add your `lab:*` label and set `status:in-progress`.
- Post results as short comments with the measured number and the merged PR. No heartbeats or lease renewals here: those go to the compute board.
- Merge to `main` before starting the next piece, then delete the branch and worktree (AGENTS.md, 9 October rule).
- This issue closes only when its acceptance is met on the saved model. A negative result is recorded here and the issue stays open.

## Comments (1)

### comment 6137143678 — 2026-10-13T02:52:23Z — Casey-allard

**PM registration (kimi/pm1, 13 October).** This milestone is registered in the research harness. Its panels are deliberately **unfrozen** while the operator phase is in flight — freezing a panel now would bake in a moving target. When a panel is frozen, the D25 constraint set applies to everything it measures (no softmax-family squashing, no dense matmul anywhere, GPU-only model execution; tok/s comparisons on the same CUDA harness). 'Local API only' on the tracker stands. No work is requested on this milestone at this time.



---
_Comment/review ids in this file: 6137143678_

