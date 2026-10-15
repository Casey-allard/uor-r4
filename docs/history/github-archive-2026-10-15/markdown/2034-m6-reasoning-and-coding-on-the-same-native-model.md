# #2034 — [M6] Reasoning and coding on the same native model

- **Type:** Issue
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-09T14:41:46Z
- **Updated:** 2026-10-13T02:52:22Z
- **Labels:** roadmap, milestone, status:not-started
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/2034
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Part of the roadmap tracker #2028.

Compositional reasoning on unseen compositions and executable Rust coding (real patches on unseen tasks, compiled and tested) by the same native artifact.

### Current state
- A DAG engine exists (#1187), but it runs caller-authored plans; the model doesn't infer them.
- The 214M chat model writes code of the right shape with wrong logic, and sets up math with wrong arithmetic. Exact computation is the visible gap. Routing arithmetic and variable tracking through exact addressed memory is the proposed direction.

### Acceptance
The acceptance of #955 and #1088, carried over verbatim below.

<details><summary>Acceptance and history carried over verbatim from #955</summary>

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — DEPENDENCY.** PR #1564 / D19; adoption task #1563.

Parent #1508; shared implementation #973. **Reopened from a September 29 tracking transfer, not from a failed or revoked capability qualification.** The full original acceptance remains below.

Depends on the shared usable model #973, grounding #954 and memory #962. Evaluate unseen compositions and changed constraints with causal intermediate-state controls and independent task checks. Early small reasoning probes may guide representation, but familiar arithmetic is not generalized reasoning. No second model is adopted.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

Roadmap position **06/12** · Programme #820 · **Not complete**

## Why this work exists

Executing a familiar arithmetic operator is not flexible reasoning. The model must compose accepted operations and preserve constraints on genuinely changed problems.

## Evidence already available

Exact intermediates, selected arithmetic and dependent reads provide primitives. Broad general reasoning is not established.

## What to implement

Learn and evaluate multi-step dependency construction, alternatives, counterfactuals, constraint preservation and error correction with independently checkable outcomes. Require committed intermediate values/state to affect subsequent decisions. Use bounded branching and reuse only where implemented and fully costed. Distinguish model reasoning from external tool answers, memorized responses and post-hoc explanations. Carry meaningful conversation and coding cases without duplicating #1088 workspace implementation. Historical release/lowering obligations have moved to #965.

## Acceptance and stopping decision

Novel compositions transfer across operands, wording, dependencies and task identities; intermediate-state interventions establish causality; independent semantic checks validate final results and constraints. Report failures and search/resource limits. Do not close from plausible explanations or two fixed operations alone.

## Order and dependencies

Uses the correctness evidence from #954 and shared composition from #1140 for the capabilities actually claimed; preserve open development throughout. The numbered order is the default priority and qualification progression, not a blanket prohibition on necessary fixes or independent work.

## Operating instructions

Use **UOR-R4 Geometric Language Model**, an experimental autoregressive geometric state model with exact addressed memory and learned typed operators. Use Rust for preparation, training, artifacts and inference. Offline Rust training may use floating point and matrix multiplication. Owner-adopted [D0-b](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/DECISIONS.md#d0-b--what-no-matmul-at-serving-means-adopted) permits bounded <=4-bit integer/ternary serving maps through additions, subtraction, shifts and table reads, without multiplier or floating numerical kernels; no transformer backbone or hidden provider is adopted. Deterministic geometric address/page selection is allowed. Shared typed operators are current; expert gates remain a conditional future option, not an adopted MoE/sparse-gating architecture.

Refresh [the canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md), [current evidence/artifact](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/current-state.md) and [source-linked direction](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/model-direction-2026-09.md). Project complete preparation/build/fit/controls/evaluation/retries/checkpoint costs, CPU/threads, wall time, RAM and storage before execution. Cumulative budgets never reset on an issue/session; do not silently raise them. **Standing owner authorization (2026-09-06):** necessary local model/time/storage allowance extensions are already authorized. Record the complete projection, reason, increment and updated cumulative limit before using each extension; retain cumulative charges and the 128 MiB storage stop margin. Do not ask the owner to approve the same class of necessary increase again. This authorizes neither destructive deletion nor paid/external compute, and does not require spending unused allowance. Use focused direct behavior checks and protected PR delivery. Keep positive, negative and NOT_RUN results at their exact scope. Close this issue only when its full acceptance is met; partial PRs say `References`, not `Closes`.

## Reconciliation and preserved history

Owner-directed September 7 reconciliation. Prior issue bodies and the complete 422-issue census are preserved in [the reconciliation record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/issue-reconciliation-2026-09.json). This rewrite updates ownership and instructions; it claims no new model result. Earlier completed issues remain closed historical evidence.


</details>


</details>

<details><summary>Acceptance and history carried over verbatim from #1088</summary>

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


</details>

### How to work on this issue
- Claim with one comment: lab, goal, expected cost; add your `lab:*` label and set `status:in-progress`.
- Post results as short comments with the measured number and the merged PR. No heartbeats or lease renewals here: those go to the compute board.
- Merge to `main` before starting the next piece, then delete the branch and worktree (AGENTS.md, 9 October rule).
- This issue closes only when its acceptance is met on the saved model. A negative result is recorded here and the issue stays open.

## Comments (1)

### comment 6137143491 — 2026-10-13T02:52:22Z — Casey-allard

**PM registration (kimi/pm1, 13 October).** This milestone is registered in the research harness. Its panels are deliberately **unfrozen** while the operator phase is in flight — freezing a panel now would bake in a moving target. When a panel is frozen, the D25 constraint set applies to everything it measures (no softmax-family squashing, no dense matmul anywhere, GPU-only model execution; tok/s comparisons on the same CUDA harness). The exact-arithmetic visibility gap noted on the tracker stands. No work is requested on this milestone at this time.



---
_Comment/review ids in this file: 6137143491_

