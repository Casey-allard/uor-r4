# #2036 — [M8] Alpha release: the same model for conversation, memory and coding

- **Type:** Issue
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-09T14:41:49Z
- **Updated:** 2026-10-13T02:52:24Z
- **Labels:** roadmap, milestone, status:not-started
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/2036
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Part of the roadmap tracker #2028.

Qualify and release the alpha after M1–M7, per the **Alpha acceptance** section of `docs/integration/project-track.md`: useful coherent prose and instruction following, causal contextual attention, correct grounded answers with abstention, durable isolated memory, novel compositional reasoning and executed coding, with full-path M1 cost, all on the same native artifact.

<details><summary>Acceptance and history carried over verbatim from #965</summary>

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — DEPENDENCY.** PR #1564 / D19; adoption task #1563.

Parent #1508; shared implementation #973. **Reopened from a September 29 tracking transfer, not from a failed or revoked capability qualification.** The full original acceptance remains below.

Release follows #962/#954/#964 and #955/#1088 on the same native path, with #963 cost evidence and #1172/#1173 distribution scope. A clearly labelled grounded-conversation preview may come first. Preserve full original alpha acceptance; it is neither merely a memory demo nor proof of frontier capability.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

Roadmap position **12/12** · Programme #820 · **Not complete**

## Why this work exists

Alpha requires integrated conversation/memory and coding/reasoning on the delivered model; frontier capability is a longer-term evidence-driven objective.

## Evidence already available

The project is pre-alpha. #1017 reference generation, native bounded successes and Studio readiness retain separate evidence scopes.

## What to implement

Assemble a frozen artifact/revision capability scorecard covering prose, attention/context, inference, memory, grounded correctness, reasoning and executable coding. Preserve #1090 scorecard and #940 governance responsibilities, plus historical #955 release/lowering obligations. Verify installation, model loading, migration/rollback, portability, security, support boundaries and reproducible release inputs. Run named native/WASM/release checks when the actual boundary is ready. Publish known limitations and cost/energy evidence, then choose the next capability/capacity work from failures instead of declaring the ultimate goal complete at alpha.

## Acceptance and stopping decision

Both capability groups meet declared independent acceptance on the same delivered artifact, with complete-path laptop evidence, reproducible installation and truthful Studio/API behavior where released. Release notes bind exact results and unrun checks. An issue merge, CI acknowledgement or visible UI cannot qualify alpha or frontier capability.

## Order and dependencies

Integrates the roadmap capabilities and serving assurance. Release qualification is the final milestone in this iteration; frontier progress continues through measured successors. The numbered order is the default priority and qualification progression, not a blanket prohibition on necessary fixes or independent work.

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

### comment 6137143871 — 2026-10-13T02:52:24Z — Casey-allard

**PM registration (kimi/pm1, 13 October).** This milestone is registered in the research harness. Its panels are deliberately **unfrozen** while the operator phase is in flight — freezing a panel now would bake in a moving target. When a panel is frozen, the D25 constraint set applies to everything it measures (no softmax-family squashing, no dense matmul anywhere, GPU-only model execution; tok/s comparisons on the same CUDA harness). M8 is owner-held; no work is requested. No work is requested on this milestone at this time.



---
_Comment/review ids in this file: 6137143871_

