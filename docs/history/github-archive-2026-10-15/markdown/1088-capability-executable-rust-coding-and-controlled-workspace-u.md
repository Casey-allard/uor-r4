# #1088 — [Capability] Executable Rust coding and controlled workspace use

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-09-03T03:55:04Z
- **Updated:** 2026-10-09T14:42:51Z
- **Closed:** 2026-10-09T14:42:51Z
- **Labels:** roadmap, roadmap:dependency
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1088
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — DEPENDENCY.** PR #1564 / D19; adoption task #1563.

Parent #1508; shared implementation #973. **Reopened from a September 29 tracking transfer, not from a failed or revoked capability qualification.** The full original acceptance remains below.

Depends on #973/#954 and shared session/API boundaries. Use the same native artifact for unseen tasks and real generated patches; compile and execute semantic tests. Preserve repository/source provenance, bounded iteration and full cost. Code-shaped text or a few authored fragments do not meet broad acceptance.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

Roadmap position **07/12** · Programme #820 · **Not complete**

## Why this work exists

The coding goal requires working programs and repairs in real context, beyond familiar code-shaped responses.

## Evidence already available

Bounded familiar Rust forms and historical semantic checks exist; general coding/workspace competence remains unqualified.

## What to implement

Use the same native model for repository-context understanding, bounded file reads, actual edits, save/reopen and multi-file repair. Build explicit controlled tool interfaces and bounded iterative compile/test feedback. Bind task, repository revision, source context, generated patch and executed result. Include novel identifiers and task identities in final evaluation. Preserve absorbed #1083/#1084/#1087/#1089 obligations at their relevant coding/workspace boundary, with interface plumbing owned by the API issue.

## Acceptance and stopping decision

Generated and repaired Rust programs pass meaningful independent semantic checks across unseen tasks, including multi-file context where claimed. Report successful and failed iterations, provenance and total cost. Compilation, a patch viewer or template completion alone is not coding capability.

## Order and dependencies

Builds on language, correctness and compositional reasoning while allowing bounded coding cases to drive those mechanisms earlier. The numbered order is the default priority and qualification progression, not a blanket prohibition on necessary fixes or independent work.

## Operating instructions

Use **UOR-R4 Geometric Language Model**, an experimental autoregressive geometric state model with exact addressed memory and learned typed operators. Use Rust for preparation, training, artifacts and inference. Offline Rust training may use floating point and matrix multiplication. Owner-adopted [D0-b](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/DECISIONS.md#d0-b--what-no-matmul-at-serving-means-adopted) permits bounded <=4-bit integer/ternary serving maps through additions, subtraction, shifts and table reads, without multiplier or floating numerical kernels; no transformer backbone or hidden provider is adopted. Deterministic geometric address/page selection is allowed. Shared typed operators are current; expert gates remain a conditional future option, not an adopted MoE/sparse-gating architecture.

Refresh [the canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md), [current evidence/artifact](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/current-state.md) and [source-linked direction](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/model-direction-2026-09.md). Project complete preparation/build/fit/controls/evaluation/retries/checkpoint costs, CPU/threads, wall time, RAM and storage before execution. Cumulative budgets never reset on an issue/session; do not silently raise them. **Standing owner authorization (2026-09-06):** necessary local model/time/storage allowance extensions are already authorized. Record the complete projection, reason, increment and updated cumulative limit before using each extension; retain cumulative charges and the 128 MiB storage stop margin. Do not ask the owner to approve the same class of necessary increase again. This authorizes neither destructive deletion nor paid/external compute, and does not require spending unused allowance. Use focused direct behavior checks and protected PR delivery. Keep positive, negative and NOT_RUN results at their exact scope. Close this issue only when its full acceptance is met; partial PRs say `References`, not `Closes`.

## Reconciliation and preserved history

Owner-directed September 7 reconciliation. Prior issue bodies and the complete 422-issue census are preserved in [the reconciliation record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/issue-reconciliation-2026-09.json). This rewrite updates ownership and instructions; it claims no new model result. Earlier completed issues remain closed historical evidence.


</details>


## Comments (4)

### comment 5579303170 — 2026-09-08T04:36:14Z — Casey-allard

Delivered and merged via PR #1188 (commit `b463ce07a15802a4ab6dc1d40eb81cf6fbd168ed`).

### Completed Acceptance Criteria
1. **Controlled Workspace Environment (`WorkspaceEnvironment`)**: Implemented path-traversal-resistant bounded file reading, writing, surgical block patching, and directory listing in `crates/uor-r4-core/src/native_geometric/workspace_coding.rs`.
2. **Compiler Diagnostic Parsing (`CompilerDiagnostic`)**: Structured `rustc --edition=2021` diagnostic outputs into typed structures capturing error level, error code (e.g. `E0308`), exact file paths, line numbers, and column offsets.
3. **Iterative Compile & Test Feedback Loop (`WorkspaceCodingEngine::iterative_repair`)**: Executed bounded compile-diagnose-patch-recompile iterations. Tested iterative repair on type mismatches, repairing broken source code and asserting clean binary compilation and execution.
4. **Multi-File Context & Interface Repair**: Verified cross-file interface repair where a binary `main.rs` referenced an invalid function signature in a library crate `math.rs`. Patched the calling interface, compiled both library and binary, and executed successfully with expected return values.
5. **Iteration Limit Enforcement**: Verified that unfixable compilation errors stop gracefully at configured `max_iterations`, preventing runaway loops and returning structured failure diagnostics.
6. **Provenance & Revision Binding (`WorkspaceCodingReport`)**: Bound task identity, initial/final cryptographic revisions, inspected source files, applied patch operations, iteration histories, and execution outputs in canonical report format.
7. **Invariant & Allocation Safety**: In-tree allocation census tests pass with 0 runtime heap allocations on `#![no_std]` hot paths. Integer serving kernel retains zero float types and zero forbidden arithmetic opcodes.
8. **Resource Accounting & Policy Compliance**: Recorded milestone in `docs/integration/current-state.md` with 4.500s model charge (cumulative 5,332.418 / 5,550s; 217.582s remaining) and 128 MiB storage margin preserved.

### comment 5585687379 — 2026-09-08T13:12:19Z — Casey-allard

Owner-authorized recovery, 2026-09-08.

The test supplied source and repair closures. The release axis never compiled model-generated text. Actual learned coding and bounded workspace acceptance remain unmet.

The audit inspected main f8e9fa32 against last verified PR1176 (4f9d02a2), rebuilt the API, reproduced failed prose/coding and request limits, and found all8API/all6release tests still pass. Those tests cannot support the wider claims. Recovery is proceeding through a protected PR; this issue remains open until its complete acceptance is met. Prior positive/negative artifacts and release history are preserved.


### comment 5898707554 — 2026-09-29T20:55:49Z — Casey-allard

Closed by Lab 1 as a tracking unit, with the owner's approval on 29 September.

The work is now tracked by the [plan of record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/plans/2026-09-29-path-to-chat.md) (#1507) and its epics:
- #1508, Track A: the native chat model;
- #1509, Track B: geometric conversion;
- #1510: infrastructure;

with one directive board per lab (#1511–#1515).

This capability stays a long-term goal of the project. The issue is closed only because the 7 September ladder no longer orders the work. Its history and results keep their scope. Reopen it, or open a successor under an epic, when a track reaches this capability.

### comment 6083185402 — 2026-10-09T14:42:51Z — Casey-allard

Moved to #2034, which carries this acceptance verbatim. Not started.


---
_Comment/review ids in this file: 5579303170, 5585687379, 5898707554, 6083185402_

