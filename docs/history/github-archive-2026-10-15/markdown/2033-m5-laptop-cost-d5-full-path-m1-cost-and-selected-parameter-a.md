# #2033 — [M5] Laptop cost (D5): full-path M1 cost and selected parameter access

- **Type:** Issue
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-09T14:41:44Z
- **Updated:** 2026-10-13T02:52:21Z
- **Labels:** roadmap, milestone, status:not-started
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/2033
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Part of the roadmap tracker #2028.

Measure the complete serving path of the M4 artifact on an M1-class laptop: cold load, ingestion, routing/index, parameter and head access, prefill/decode, persistence, RAM, storage and energy per useful task (D5: selected parameter access is the final cost target). No transformer comparators (owner, 3 October); compare geometric variants.

Depends on M4.

<details><summary>Acceptance and history carried over verbatim from #963</summary>

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — READY.** PR #1564 / D19; adoption task #1563.

Parent #1508; shared implementation #973. **Reopened from a September 29 tracking transfer, not from a failed or revoked capability qualification.** The full original acceptance remains below.

Design/accounting may advance now with #1512; performance qualification depends on a useful saved artifact from #973/#964. Measure ingestion, compiler/router/index, learned parameter/output-head access, prefill/decode, persistence, cold/warm latency and RSS. Physical J/token needs actual measured energy; count rows/bytes separately. No automatic scaling run or fabricated power proxy.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

<!-- geometric-attention-synthesis-2026-09-20:start -->
**Merged PR #1352 principal review: V3 observation and feedback component retained; transferable language remains open.** The loaded artifact matches 31/31 exposed authored responses (23 fitting cases; eight familiar non-computed cases from three other documents), with 791/791 in-sample export/reload teacher steps. The retained finite table scores 20/31 with less information. All computed/post-copy `now` cases were fitted; the 4/4 pair varies operation and supplied `key_changed` along with payload, so it does not isolate learned content use. A changed derived address is not an entity correction, and `became` remains authored wording. Ordinary-text learning is NOT_RUN; no geometric superiority or energy reduction is established.

The principal replay reused identical learned artifacts and all 155 case rows, verified three 24-file sealed manifests, and passed 48 distinct focused library tests. It actually executes S checked inference and continuation beside E as separate components; this is not a matched scoped language comparison. No neural refit or fresh acceptance draw is claimed. [Review](https://github.com/UOR-Foundation/uor-r4/blob/44c2ccd27cea5d0178f19cca2f33332db4a95ce1/docs/integration/transferable-lexical-principal-review-2026-09-23.md) · [checks](https://github.com/UOR-Foundation/uor-r4/blob/44c2ccd27cea5d0178f19cca2f33332db4a95ce1/docs/evidence/transferable-lexical-principal-checks-2026-09-23.json) · [canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/44c2ccd27cea5d0178f19cca2f33332db4a95ce1/docs/integration/project-track.md) · [DeepSeek next brief](https://github.com/UOR-Foundation/uor-r4/blob/44c2ccd27cea5d0178f19cca2f33332db4a95ce1/docs/integration/deepseek-ordinary-lexical-step-2026-09-23.md).

Cost acceptance remains open: the E and S probes are component executions, not a quality-matched whole-path laptop benchmark; energy remains UNAVAILABLE and no complete-path advantage is claimed. Broad issue acceptance stays OPEN. DeepSeek has substantive design discretion within the stable native contract and prospectively recorded local resource authorization. Preserve original artifacts and sealed roots; no arbitrary timer/retry quota or paid compute.
<!-- geometric-attention-synthesis-2026-09-20:end -->

Roadmap position **08/12** · Programme #820 · **Not complete**

## Current cost/resource correction after PR #1312

The [principal review](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/occurrence-reader-review-2026-09-20.md) preserves the numerical evidence but corrects the reported 0.53–0.80ms timings: the evaluator warms an unbounded token-pair parent-logit cache (16390×4096i32=268533760payload bytes) and precomputes residual-logit tables (1966080bytes each), excluding construction. E timing also performs unused S row work. Serialized parent 454788bytes plus reader 53555bytes is not the live evaluator footprint. These are cached harness timings; direct serving latency at this boundary and physical energy remain UNAVAILABLE. No complete-path efficiency claim.

The next [complete DeepSeek prompt](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/deepseek-occurrence-reader-step-2026-09-20.md) requires the actual declared serving path and total live state/cost. A bounded cache/compiled table may be evaluated explicitly with construction, memory, invalidation and hit assumptions, not silently used only for timing. No full attribution rerun is needed.

Live JSON 178538565/180500000ms, remaining 1961435ms; preserve the prior 2400000ms mixed measured/estimated debit. UTC/phase records are partial; do not recast the debit as wholly measured or refund it. Review has no build/model work and changes no balance. Next proposal 10800000ms complete work, standing-authorized increment 10800000 to 191300000 BEFORE use, not applied here. Caps/revision latitude in the prompt.

Owner-authorized [cleanup](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/storage-cleanup-2026-09-20.md) recovered 23.21decimalGB (21.62GiB), leaving51.17GB (47.65GiB). Model/research/session data, source worktrees and release binaries preserved. Main debug deps and incremental caches partly removed; measure actual rebuild growth. Whole-machine free capacity and the 128MiB experiment stop margin are distinct. No paid compute. [Review delivery PR #1313](https://github.com/UOR-Foundation/uor-r4/pull/1313). Broad cost acceptance remains open.

## Why this work exists

The project exists to reduce energy and wasted compute. Owner-adopted D0-b targets bounded low-bit additive execution with no multiplier in the declared numerical kernel; reducing arithmetic or parameter traffic does not itself establish lower energy or useful speed.

## Evidence already available

Scoped integer/table and allocation/kernel results exist. Latest timing excludes input ingestion; energy remains unmeasured. #1019 full campaign was unavailable under its historical resource window, not a model-quality failure.

## What to implement

Measure quality-matched complete tasks on consumer M1 hardware: cold loading, ingestion, geometric lookup, state/operator execution, emission, persistence, memory traffic, RAM, storage, energy and sustained thermal behavior. Specify whether a submillisecond target concerns a decision, token or completed response and report distributions. Scale deterministic address/page access and packed finite operators only against observed bottlenecks. Preserve resumable training, compatible checkpoints, useful measured parallelism, exact reductions where required and explicit approximation/fidelity tradeoffs. Model tables must not grow as an uncontrolled Cartesian product.

## Acceptance and stopping decision

Reproducible quality/cost curves with full timing scope, sustained resource measurements, compatible resume/rollback and bounded growth. Demonstrate improvement per useful task rather than isolated kernel speed. Record hardware, artifact, configuration and comparison quality. No blanket submillisecond or energy claim without measurement.

## Order and dependencies

Cost accounting starts immediately; substantial scaling follows a useful repeatable capability workload. This absorbs historical #954 performance work. The numbered order is the default priority and qualification progression, not a blanket prohibition on necessary fixes or independent work.

## Operating instructions

Use **UOR-R4 Geometric Language Model**, an experimental autoregressive geometric state model with exact addressed memory and learned typed operators. Use Rust for preparation, training, artifacts and inference. Offline training matmul is permitted; D0-b permits bounded <=4-bit add/subtract/shift/table linear maps with no multiplier instruction in the declared numerical kernel and no floating/transcendental serving computation or transformer backbone. Deterministic geometric address/page selection is allowed. Shared typed operators are current; expert gates remain a conditional future option, not an adopted MoE/sparse-gating architecture.

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

### comment 6137143341 — 2026-10-13T02:52:21Z — Casey-allard

**PM registration (kimi/pm1, 13 October).** This milestone is registered in the research harness. Its panels are deliberately **unfrozen** while the operator phase is in flight — freezing a panel now would bake in a moving target. When a panel is frozen, the D25 constraint set applies to everything it measures (no softmax-family squashing, no dense matmul anywhere, GPU-only model execution; tok/s comparisons on the same CUDA harness). Note: the body's reference to #1512 is closed-historical. The exact-arithmetic gap noted on the tracker stands. No work is requested on this milestone at this time.



---
_Comment/review ids in this file: 6137143341_

