# #1057 — A1Q-H13: continue the learned exact-data attention checkpoint

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T16:35:55Z
- **Updated:** 2026-09-02T17:18:42Z
- **Closed:** 2026-09-02T17:17:37Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1057
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Objective and authorization

The user approved one bounded continuation of #1055's still-improving checkpoint after its development curve rose 36.0% → 55.1% → 63.8% → 68.3%. Test remaining learning under the same mechanism before coherent-R4 integration. This is more exposure to the same 8,192 rows, not a claim about scaling unique data.

## Frozen continuation

- Start only after #1055 is merged. Preserve its terminal result and artifact unchanged.
- Bind parent result `blake3:3cb810f09a118cfb70752643f5d9e60d0e42780dc6e47dc4f99224cbd69af0ee`, artifact `blake3:2a225b691ffde7b40afd41ac888c5a4449a6b3ba48c2773a2108e1e407d6f8b4`, and checkpoint `blake3:e41064d74f31d45b51ecf07590ef1ae12e3f1efddce535e92b8b1b1eabd15386`.
- Restore the exact 3,920-update checkpoint: model, AdamW moments/counters, cosine scheduler at block 20 with T_max 64, sampler permutation/cursor, and global Torch RNG. No fresh fit or reset. The first 20 history entries stay unchanged.
- Same exact primary/control data, 8,192 training rows, 1,024 development rows, width 64, two layers, one head, V4096/T120/eight K/V, batch 512, existing dropout and optimizer policy.
- One additional window of at most 20 blocks × 196 updates = 3,920 additional updates; total cap 7,840. At most 16,056,320 additional training queries and 163,840 additional development queries.
- Continue the existing cosine schedule. Evaluate after every 196 updates. Strict development top-1 >99% stops immediately, with no further optimizer/scheduler step.
- Only after that pass, evaluate the already-frozen physical binding-permuted control once without fitting; require at least a 50-percentage-point drop.
- No new data, learning-rate change, width change, parameter grid, restart from initialization, geometry change, additional control, or automatic further extension.

## Minimal implementation and CPU execution

Use a new evidence root and a small sibling continuation runner. Reuse the unchanged cell, loaders, scorer, sampler, step, checkpoint writer, and artifact serializer; do not modify historically CID-bound implementation. Bind parent checkpoint/parent binding plus the new continuation contract. Avoid module-global monkeypatching and do not disguise the old terminal result as a new run.

Reuse the recent eight-intra-op-thread Apple Accelerate CPU plan, one inter-op thread and one training process. The observed same-shape 3,920-update run took 1,078.93 seconds; a 1.25 safety factor projects about 1,348.66 seconds. No new CPU timing matrix, source reproduction, C0, or broad QA. CUDA/MPS forbidden.

Apply the 1,800-second and 8-GiB resource limits to additional execution only. Keep inherited, additional, and total work/time separate. Checkpoint at least every 16 updates and block boundary, with state and accumulated additional elapsed time. Recovery must not renew the time budget or regenerate an unconsumed permutation. Emit progress and ETA.

Activate only focused continuation equality, inherited/additional accounting and restart-budget, strict-pass/control exclusion, and predecessor-tamper checks; scoped Python format/lint, claim wording, and diff integrity. Preserve #1055's existing checks without repeating them.

## Decisions and completion

Metric: current 5,594/8,192 = 68.2861328125% development top-1, NLL 2.6761019229888916; target strict >99%. All target bindings are present causally, so the structural ceiling is 100%, not a claimed model guarantee.

- Primary >99% plus binding drop >=50pp: qualify the continued artifact for a separate inference-only coherent-R4 integration.
- Primary misses by total update 7,840: retain the entire curve and artifact; do not infer a capacity ceiling or launch another extension/sweep. Recommend the next distinct project action, with the already-qualified #1050 R4 integration available.
- Primary pass but binding drop misses: nonassociative-shortcut result; no geometric promotion.
- Invalid inherited state: stop before fitting as invalid continuation mechanics.
- Resource/interruption without complete decision: incomplete, not model failure.

Finish with fresh-process final artifact replay, exact inherited/additional/total work and read ledger, result identities, concise current documentation and append-only record, protected PR delivery, and child/#973 outcome comments. Queue acknowledgements are transport only. No English generation, reasoning, exact lowering, product promotion, or release in this issue.

## Comments (3)

### comment 5513160718 — 2026-09-02T16:52:10Z — Casey-allard

Pre-run freeze for the user-approved exact-checkpoint continuation.

Implementation commit: 94b75a66 (based on protected main f2f2396bedeafaa8377eb67232f02e82ca8b45d9).
Preparation: blake3:4a01fada99551fa7360520fdda18890bdd3d4f3b8c5ed03cfb41aee20150eeaa.
Implementation closure: blake3:39530199c1f0cfec2d2d85be7684c6c66688b74b22cbd86b91b6a672eb0010fd.
Parent checkpoint: blake3:e41064d74f31d45b51ecf07590ef1ae12e3f1efddce535e92b8b1b1eabd15386 (3,753,269 bytes).

The seven new focused continuation checks passed in 2.167 seconds, including uninterrupted-versus-resumed trajectory equality, inherited/additional work and restart-time accounting, pass finalization/control guard, and parent tamper rejection. Historical training/test suites and CPU timing matrices were not rerun.

Preparation passed. Reuse the measured CPU-only plan: one process, eight intra-op threads, one inter-op thread, batch 512, Apple Accelerate. Projected additional wall time with the frozen 1.25 safety factor is 1,348.658 seconds; additional run budget is 1,800 seconds and 8 GiB. No CUDA or MPS.

Starting from the preserved #1055 state at 3,920 updates and 5,594/8,192 development queries correct (68.2861%). Up to 20 more 196-update blocks; same unique data and all learned/optimizer/scheduler/sampler/RNG state. Strict top-1 >99% stops training immediately and is the only permission to read the frozen binding-permuted control. No automatic further extension.

### comment 5513353757 — 2026-09-02T17:06:57Z — Casey-allard

User clarification during execution: preserve this learned checkpoint and its measured progress regardless of whether the fixed >99% target is reached. A narrowly unmet target must not be presented as "false", failed attention, a capacity ceiling, or grounds to discard the artifact or halt geometric integration.

Reporting will distinguish (1) the exact measured recall/NLL and learning gain, (2) whether the preset numerical target was met, and (3) which binding-control evidence was actually collected. Existing #1050 99.1667% evidence remains valid on its own different configuration/population. No cross-population head-to-head superiority is implied.

The already-running continuation, fixed limits, checkpoint preservation, and conditional control rule remain unchanged; no retuning, automatic extension, or retroactive threshold change is being made. If the raw frozen terminal is CONTINUATION_MISS, current project summaries will explain it as a target-not-met status with retained learning progress, not a falsification of the mechanism.

### comment 5513502705 — 2026-09-02T17:18:42Z — Casey-allard

Delivered through protected PR #1058, merged at d424dbee0272eeaa61f23262ff863495be956593. The merged tree matches submitted head 7fd8b686f798dbaf777a00551e8cf85650952a83; native issue #1057 is closed.

Retained result: 8,071/8,192 = 98.52294921875% final recall, versus 68.2861328125% inherited (+30.23681640625 percentage points). NLL fell from 2.6761019229888916 to 0.09141154401004314. Best observed was block 36 at 98.6572265625%; the preserved final artifact is block 40.

Per the user's explicit direction, this is meaningful near-target learning progress, not false/failed attention, discarded work, a capacity ceiling, or a project stop. Raw CONTINUATION_MISS records only that strict >99% was narrowly unmet. The conditional binding control remains NOT_RUN_PRIMARY_MISS with zero decisions. No threshold or history was changed, and no unrun control evidence is claimed.

Exactly 3,920 additional updates, 7,840 total, on the same 8,192 unique training rows. Additional execution took 1,096.986830 seconds on eight CPU threads, with one inter-op thread/process and Apple Accelerate; no CUDA/MPS. This measures more training exposure, not more unique data.

Seven focused continuation checks passed. Fresh-process artifact replay and independent read-only final-checkpoint/accounting audit passed; no old QA, source reproduction, CPU timing matrix, or additional fit was rerun. Queue acknowledgements were transport only.

Result CID: blake3:35b1cedfd51385bf98277a4527b1ce05f5dd3b93fffe125a5ea28c2a34b6387c.
Final artifact CID: blake3:69af5586eccfceab4214e9f13524eeea578eb3facaea4fdedec89f0b5d217445.
Final checkpoint CID: blake3:fd24e6b84af9891c1dad1eb2a13c6d86b8e8833ac02ab40f0e4e95d4530d140a.
Evidence retained at .uor-models/research/issue-1057-zoology-checkpoint-continuation.
Complete record: https://github.com/UOR-Foundation/uor-r4/blob/d424dbee0272eeaa61f23262ff863495be956593/docs/r4_zoology_checkpoint_continuation_1057.md.

Current README, roadmap, research ledger, completion plan, architecture companion, and agent guidance now reflect this retained result. Recommended next action is a separately scoped inference-only coherent-R4 integration using the preserved #1050 99.1667% qualified reference, retaining #1057 as a valuable exact-data artifact for subsequent inference work. The configurations/populations differ, so these are not head-to-head scores. Existing HELM-D-R4/#1014 positives remain. No automatic third training window or new generation/reasoning/lowering claim.


---
_Comment/review ids in this file: 5513160718, 5513353757, 5513502705_

