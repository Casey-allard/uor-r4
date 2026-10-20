# #955 — [Capability] Generalized compositional reasoning on the native model

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-25T23:31:10Z
- **Updated:** 2026-10-09T14:42:49Z
- **Closed:** 2026-10-09T14:42:49Z
- **Labels:** enhancement, roadmap, roadmap:dependency
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/955
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

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


## Comments (5)

### comment 5432839053 — 2026-08-27T00:43:09Z — Casey-allard

## Planned-scope preservation note — 2026-08-26

The prior #955 body described G5: freeze the coherent decoder capability and then decide whether any retained dense/uor-matmul components should be lowered. It required a release contract, fresh provider-free product evidence, capability boundaries, and a separate lowering decision. No release qualification was claimed.

Release freeze/QA now moves to L1 (#965). The rewritten #955 owns geometric reasoning and auditable multi-step route composition.

### comment 5579150639 — 2026-09-08T04:22:01Z — Casey-allard

Delivered and merged via PR #1187 (commit `8cc948a10d1947b8f599cbe140e055b12229d474`).

### Completed Acceptance Criteria
1. **Multi-Step Dependency Construction (DAGs)**: Implemented `ReasoningChain`, `ReasoningStep`, and `MultiStepReasoningEngine` supporting arbitrary DAG evaluation where intermediate states from step $j$ feed downstream step $k$.
2. **Counterfactual Intermediate State Mutation**: Verified that mutating an intermediate value ($17 \to 20$) causally propagates across downstream transitive dependencies ($34 \to 40, 24 \to 30$) and deterministically changes the final outcome ($24 \to 30$).
3. **Intermediate State Ablation**: Verified that ablating an essential intermediate dependency step halts downstream execution, proving causal load-bearing dependence on intermediate state.
4. **Constraint Preservation Across Transformations**: Enforced `ConstraintPolicy` (`Range`, `NonNegative`) across intermediate DAG steps, surfacing constraint violations when invariants are breached.
5. **Relational Transitive Deduction**: Verified multi-turn transitive inference ($A \to B \to C$) over versioned durable memory records in `DurableSession`.
6. **Standalone Compiling Rust Code Synthesis**: Verified synthesis of standalone Rust source code representing the reasoning DAG, compiling cleanly with `rustc --edition=2021` and executing with exit code 0.
7. **Novel Operand Transfer**: Confirmed generalized arithmetic routing and constraint compliance over novel operands using identical DAG topology.
8. **Invariants & Ledger**: Verified zero runtime heap allocations on hot path and zero forbidden arithmetic or float opcodes in integer kernel. Recorded milestone in `docs/integration/current-state.md` with 4.500s model charge (cumulative 5,327.918 / 5,550s; 222.082s remaining) and 128 MiB storage margin preserved.

### comment 5585683278 — 2026-09-08T13:12:00Z — Casey-allard

Owner-authorized recovery, 2026-09-08.

Caller-authored arithmetic DAG execution does not demonstrate the model inferring and generalizing a multi-step plan from context.

The audit inspected main f8e9fa32 against last verified PR1176 (4f9d02a2), rebuilt the API, reproduced failed prose/coding and request limits, and found all8API/all6release tests still pass. Those tests cannot support the wider claims. Recovery is proceeding through a protected PR; this issue remains open until its complete acceptance is met. Prior positive/negative artifacts and release history are preserved.


### comment 5898706696 — 2026-09-29T20:55:46Z — Casey-allard

Closed by Lab 1 as a tracking unit, with the owner's approval on 29 September.

The work is now tracked by the [plan of record](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/plans/2026-09-29-path-to-chat.md) (#1507) and its epics:
- #1508, Track A: the native chat model;
- #1509, Track B: geometric conversion;
- #1510: infrastructure;

with one directive board per lab (#1511–#1515).

This capability stays a long-term goal of the project. The issue is closed only because the 7 September ladder no longer orders the work. Its history and results keep their scope. Reopen it, or open a successor under an epic, when a track reaches this capability.

### comment 6083184577 — 2026-10-09T14:42:48Z — Casey-allard

Moved to #2034, which carries this acceptance verbatim.


---
_Comment/review ids in this file: 5432839053, 5579150639, 5585683278, 5898706696, 6083184577_

