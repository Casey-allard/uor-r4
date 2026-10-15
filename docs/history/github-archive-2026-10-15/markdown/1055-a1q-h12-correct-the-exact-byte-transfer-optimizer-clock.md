# #1055 — A1Q-H12: correct the exact-byte transfer optimizer clock

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T15:03:52Z
- **Updated:** 2026-09-02T16:38:14Z
- **Closed:** 2026-09-02T16:34:53Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1055
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Objective and project value
Correct the optimizer-clock mismatch found in #1053 so the already-working copied causal-softmax cell receives a source-comparable optimization opportunity on our exact open data. Preserve #1050's positive attention evidence. This is a bridge to coherent R4 transport, not a new architecture or another broad qualification campaign.

## Frozen single-arm scope
- Bind #1053 result `blake3:e2d1deb55a4612015ba924a94051beacd517f3c062c714c4972ba954f57621a1`, its preparation, exact primary/control tensor bytes and source/lock closure; read no fitted predecessor weights.
- Fresh seed 123; unchanged width64/two-layer/one-head copied cell, V4096/T120/eight K/V, batch512, initial LR0.00046415888336127773, AdamW weight decay0.1 and source dropout.
- Correct the clock only: 20 source-clock blocks of196 optimizer updates (3,920 maximum). CosineAnnealingLR T_max64 advances once per source-clock block after a miss, not once per pass through8192 rows.
- Cycle the unchanged8192 training rows with shuffled complete permutations. Preserve the partially consumed permutation/cursor across block boundaries and restart; do not discard/reset it. Train/development shuffles and dropout share the global Torch RNG.
- Evaluate the existing1024-row development split after each196-update block. Strict top1 >0.99 stops immediately; no more optimizer/scheduler steps after a passing block.
- This matches optimizer opportunities and evaluation/scheduler cadence, NOT source data ordering, query exposure or arithmetic. Source ends each100000-row epoch with a160-row batch; here all batches have512 rows. Full dose is16,056,320 training queries and at most163,840 primary-development queries.
- Only after a primary pass, evaluate the already-frozen physical binding-permuted development input once without fitting. Require a top1 drop of at least50 percentage points. No other model controls or probes.

## Minimal implementation and admission
Reuse the credited cell, exact data adapters, serializer and scorer; do not modify historically CID-bound #1050/#1053 code. Add only the clocked trainer, its causal lifecycle/implementation bindings and focused checks. Bind pyproject.toml and uv.lock. Reuse the already-passed source-cell C0 evidence because the copied model/data code is unchanged; do not repeat the source reproduction or overfit campaign.
Activate only focused schedule/order/cycling/RNG/resume/control-boundary tests, Python format/baseline lint, claim wording and diff integrity. Broad Rust/workspace/BDD/WASM/QA stays dormant.
CPU only; CUDA/MPS forbidden. Measure the materially plausible1/4/8-thread plans on the unchanged batch shape and choose the fastest stable eligible plan. Reuse #1053 deterministic mechanical evidence without a new cross-backend campaign. Safety-adjusted full-run projection <=1800 seconds and measured peak RSS <=8 GiB. One training process, no multiprocessing of dependent updates.
Provide flushed progress, completed/remaining updates, measured ETA, and checkpoint at least every16 updates and each block boundary. Persist current permutation/cursor and global RNG; a saved pass finalizes without another update. A resource stop is incomplete, never a model failure.

## Decisions
- Primary >99% and binding drop >=50pp: CLOCK_MATCHED_TRANSFER_PASSES. Scope the separate coherent-R4 transport/replacement comparison of this learned cell.
- Primary misses by3920 updates: CLOCK_MATCHED_TRANSFER_MISS. Preserve the model, curve and prior positives; do not add another learning-rate/width/epoch sweep. Return a bounded explanation of what remains unresolved and recommend a distinct next action toward the project goal.
- Primary passes but binding drop misses: NONASSOCIATIVE_SHORTCUT.
- No eligible CPU plan: NOT_RUN_PREFLIGHT. Mechanics invalid: INVALID_CLOCK_MECHANICS. Resource/interruption without a complete decision: INCOMPLETE, not attention failure.

## Run contract and DoD
Metric: exact-data development top1, currently12.01171875%, targetstrict >99%; all8192 targets have admitted causal K/V evidence (structural ceiling100%). Instrument: focused clock/resume checks plus measured CPU admission. Positive and negative actions differ above.
Scope ends after one terminal result, fresh-process artifact replay, exact work/read/CID ledger, concise current README/ROADMAP/RESEARCH/completion updates and append-only record, protected PR merge, and terminal child/#973 comments. Queue acknowledgements are transport only, not tests. No R4 code, new corpus, role/geometry/model future inputs, teacher/provider/hidden/sealed access, generation, reasoning, W8/lowering, product or release work in this issue.

## Comments (6)

### comment 5511887841 — 2026-09-02T15:18:24Z — Casey-allard

_edited 2026-09-02T15:18:40Z_

Implementation frozen at `47ebdabc` before execution. Preparation: `blake3:935363a19038bb9573cda29c29179f98b4b2a80f4d2e0ac9b64b46ae399f5916`; implementation: `blake3:bf45a18a2b4ed1aee607220f8d0331c32254bb43e94c2e1d7bece70d33634de3`. The 40 inherited source/dependency files are unchanged; 8 clock source/test files are newly bound.

15 focused clock/contract/sampler/recovery tests passed in 0.982 s. Python formatting, scoped baseline lint, claim wording, and diff integrity passed. The independent review confirmed partial-permutation recovery, strict pass stopping, cached-result CPU restoration, and conditional control. No broad QA was activated.

Reused #1053 C0 exactly as authorized; no source reproduction or overfit rerun. Exact primary/control bytes remain in place and no predecessor fitted weights were read. Next: measure the 1/4/8 CPU plans, publish the admitted plan, then launch the single frozen arm if eligible.

### comment 5511923259 — 2026-09-02T15:20:53Z — Casey-allard

The frozen CPU admission attempt did not select a plan: preflight CID `blake3:a0f88c9328505f56860a3e2cfdb947be83fb5d5877d9acac2e9ab3f4ff92ffd5`.

| Threads | Timed 8-update repeats (s) | Safety-adjusted projection (s) | Stable | Peak RSS (bytes) |
|---|---|---|---|---|
|1|3.376828 / 3.319727|2080.005|yes|1162067968|
|4|2.378479 / 5.017547|3086.198|no|1197031424|
|8|6.401626 / 10.149635|6234.237|no|958251008|

All repeated losses matched within each CPU plan; all RSS measurements were below8GiB. No plan met both stability and the frozen1800-second projection. `NOT_RUN_PREFLIGHT` is a resource admission outcome, NOT `CLOCK_MATCHED_TRANSFER_MISS`: zero primary updates, no primary development evaluation, no control, and no model artifact. The corrected mechanism remains implemented and its15 focused checks passed.

Read-only system inspection showed load average8.86 and substantial simultaneous WebKit/Studio, WindowServer and filesystem-event CPU activity; this is consistent with contention, not a demonstrated sole cause. No thermal warning was reported. I did not close or alter other applications.

Requested user coordination to pause the active Studio/browser workload and explicitly allow timing-only remeasurement, keeping the frozen model/data/update/scheduler contract unchanged. No timing retry or training run has been launched. Keep #1055 open pending that coordination; do not turn this resource pause into a scientific negative or open another model-tuning issue.

### comment 5512568057 — 2026-09-02T16:07:08Z — Casey-allard

User confirmed the competing work has stopped and requested continuation. Reacquiring #1055 and authorizing one CPU-timing-only remeasurement in a separate evidence directory, `issue-1055-zoology-optimizer-clock-attempt-2`. Preserve the original preflight unchanged. There has still been no primary fit. The exact model, data bytes, seed, optimizer/update/scheduler clock, success/control criteria, resource limits, and implementation CID remain unchanged; no focused-test or source-reproduction rerun is needed.

### comment 5512580006 — 2026-09-02T16:08:01Z — Casey-allard

User-authorized timing-only retry passed with the unchanged preparation/implementation CIDs. Retry evidence directory: `issue-1055-zoology-optimizer-clock-attempt-2`; preflight CID `blake3:9f184ae09fea32d6a797303a3770ad10f5603ec4ac86bd2010bfc063ce39cdf6`.

Stable safety-adjusted projections: 1 thread1865.098s; 4 threads1353.474s; 8 threads1306.501s. Selected the fastest eligible plan: 8 intra-op threads, 1 inter-op, 1 CPU process, batch512, Apple Accelerate; peak timing RSS1,211,580,416bytes. All three repeated-loss sequences matched within their respective plans.

Launching the first and only primary fit now: maximum3920 updates, development every196, corrected source-clock cosine schedule, strict >99% early stop, existing binding control only after primary pass. No code changes, repeated QA, source reproduction, fitted predecessor weights, CUDA or MPS. Original refused timing attempt is preserved unchanged.

### comment 5512898669 — 2026-09-02T16:31:56Z — Casey-allard

The first/only frozen primary run completed at `CLOCK_MATCHED_TRANSFER_MISS`, with substantial retained improvement: `5,594/8,192 = 68.2861328125%` development top1 versus #1053's `984/8,192 = 12.01171875%` (+56.2744140625pp). NLL fell from6.79664158821106 to2.6761019229888916. The strict >99% target was not reached, so the binding control remains `NOT_RUN_PRIMARY_MISS` with zero decisions.

Exactly20 blocks/3,920 updates,16,056,320 training query presentations and163,840 development queries ran. Primary elapsed1078.926785s, total1079.297882s; eight CPU threads, Apple Accelerate, peakRSS1,151,451,136bytes. ResultCID `blake3:3cb810f09a118cfb70752643f5d9e60d0e42780dc6e47dc4f99224cbd69af0ee`; artifactCID `blake3:2a225b691ffde7b40afd41ac888c5a4449a6b3ba48c2773a2108e1e407d6f8b4`.

Fresh-process verification exactly reproduced final scores/logitCID/result, adding8,192 inference decisions and zero updates. Independent read-only audit matched all20 exported tensors, tiedhead, evaluationRNG, optimizersteps3920, schedulerlast_epoch20, sampler245completecycles/cursor8192. The original15 focused tests remain valid; no code changed or broad QA reran. Teacher/provider/sealed/future-value/role/geometry/predecessor-weight model reads were zero.

Interpretation: the last blocks rose36.0%→55.1%→63.8%→68.3%. This stopped at a preset budget, not demonstrated convergence. The same8,192 unique rows were reused; this is exposure/update learning, not measured unique-data scaling. No ordinary-attention mechanism is falsified and prior #1050/HELM-D-R4/#1014 positives remain.

The user has now explicitly approved a separately bounded continuation of this exact checkpoint: up to another3,920 updates with the same data/model/optimizer/RNG/cosine policy and strict >99% stop, no fresh fit or parameter sweep. It will receive a new contract after #1055 delivery; it is not silently appended to or a reclassification of this result. No continuation has started.

### comment 5512980378 — 2026-09-02T16:38:13Z — Casey-allard

Completed and closed through protected PR #1056, merge `f2f2396bedeafaa8377eb67232f02e82ca8b45d9`. Verified merged main has the exact submitted tree. The measured gain, immutable target miss, full curve, artifact, fresh replay and independent state audit are recorded in `docs/r4_zoology_optimizer_clock_1055.md`. Queue acknowledgements were transport only.

The user's separately authorized continuation is now #1057, with native parent #973 and resolved blocker #1055. It will resume this exact checkpoint without changing #1055's result. No source model, dataset, or training-policy change is authorized.


---
_Comment/review ids in this file: 5511887841, 5511923259, 5512568057, 5512580006, 5512898669, 5512980378_

