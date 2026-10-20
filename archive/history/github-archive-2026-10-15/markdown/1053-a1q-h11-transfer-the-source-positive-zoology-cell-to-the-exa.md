# #1053 — A1Q-H11: transfer the source-positive Zoology cell to the exact #1045 bytes

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T13:30:32Z
- **Updated:** 2026-09-02T14:20:14Z
- **Closed:** 2026-09-02T14:19:31Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1053
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Objective

Answer one question: can a freshly initialized width-64, two-block, one-head copied Zoology causal-softmax cell learn the exact open #1045 8,192-row construction and 1,024-row assignment-disjoint development population when trained with the #1050-positive semantics?

This is the shortest honest bridge from the released-configuration positive to a later coherent-R4 transport/replacement comparison. It is not a new source reproduction and does not fine-tune the #1050 artifact.

## Bound inputs and mechanics

- Bind #1050 result CID blake3:bd16d012c01262ffb8c5197e4cf316c6fee1d722cf0700a0048386180a8122e0 and the exact open #1045 population bytes/CIDs.
- Initialize from seed 123; do not read #1050 fitted weights.
- Preserve the copied width-64, two-layer, one-head attention cell and source training order.
- Make only the required data-shape changes: vocabulary 4,096, context 120, eight K/V pairs.
- Validate #1045 role bytes for provenance only. Model reads of role, H4, R4, W8, teacher, provider, sealed, or future values remain zero.
- Freeze one learning rate only: exact 0.00046415888336127773. No sweep or same-issue tuning.
- Batch 512, AdamW weight decay 0.1, cosine schedule capped at 64 epochs, shared train/development DataLoader Torch RNG, train then development, strict development accuracy >0.99 early stop, scheduler step only after a miss.
- CPU only. Measure 1/4/8 intra-op threads with one worker and select the fastest stable eligible plan. CUDA and MPS are forbidden.

## Required lifecycle repairs before the run

- Include pyproject.toml and uv.lock in the implementation/dependency commitment.
- When a restored checkpoint already ends in a strict passing epoch, finalize that pass without another optimizer step.
- Prove both behaviors with focused tests before admission. Do not modify or reinterpret the immutable #1050 evidence.

## Ordered decision

1. Bind exact inputs and pass focused source-cell, causal, query-only projection, DataLoader RNG, dependency-CID, and passing-checkpoint-finalization checks.
2. Admit only a measured CPU plan whose safety-adjusted primary projection is at most 900 seconds and peak RSS is at most 8 GiB.
3. Run the single primary arm for at most 4,194,304 query presentations.
4. Stop negative if assignment-disjoint development does not exceed 99% by epoch 64.
5. Only after a primary pass, evaluate the existing exact data-level binding-permuted development control without retraining. Require a drop of at least 50 percentage points.

## Decisions

- Primary strict >99% plus binding-permuted drop >=50pp: STOCK_CELL_PASSES_EXACT_BYTES. Open a separate coherent-R4 K/V transport/replacement parity issue.
- Primary miss: STOCK_CELL_TRANSFER_MISS. Stop without R4 changes; this is a serialization/population-transfer miss, not an ordinary-attention falsifier.
- Primary pass but destructive drop <50pp: NONASSOCIATIVE_SHORTCUT. Reject the score as binding evidence.
- C0 or lifecycle repair miss: INVALID_TRANSFER_MECHANICS. Repair only in a new issue.
- No eligible CPU plan: NOT_RUN_PREFLIGHT.

No outcome authorizes a rerun, learning-rate sweep, width/head change, current-only or attention-off arm, English, generation, reasoning, R4 geometry, W8/modulo-256 softmax, lowering, product work, or broad QA in this issue.

## Run contract

metric to move: #1045 assignment-disjoint development top-1, 87.121582% to strict >99%
reachability ceiling: all 8,192 development query targets are directly scoreable from one admitted causal K/V binding
cheap instrument: focused C0/lifecycle checks plus measured 1/4/8-thread CPU preflight
if positive: run one binding-permuted evaluation; if its drop passes, scope coherent-R4 replacement
if negative: stop and record exact transfer miss; do not modify R4
cost ceiling: safety-adjusted primary wall <=900 seconds and peak RSS <=8 GiB

## Definition of done

- Exact input, implementation/dependency, preparation, preflight, artifact, state, control, and result CIDs.
- Complete causal/read/work ledger and exact query-presentation count.
- Fresh-process artifact/result verification.
- Focused loader/model/lifecycle tests, Python format/baseline lint, claim-wording, Rust format, and diff integrity only; no broad workspace campaign.
- Canonical append-only record and updates to README, ROADMAP, RESEARCH, and the intelligence completion plan.
- Terminal issue and #973 comments with the protected merge reference.

## Comments (4)

### comment 5510794765 — 2026-09-02T14:03:58Z — Casey-allard

Implementation is frozen at ec7b0770. The 31 focused source-cell, loader, query-only loss/gradient, shared-RNG and transfer lifecycle checks passed; baseline Python lint/format, Rust format, claim wording and diff checks passed. Historical #1050 source/evidence was not changed. The dependency commitment includes pyproject.toml and uv.lock, and a focused crash/restart test proves a saved passing checkpoint finalizes without another optimizer step.

Exact preparation: blake3:1de63b0377a0f64f3181de37e706306dfaa8242c40f667faf47f4aba812272f2. Implementation: blake3:e01665c060f03cd2dac1dadd8df9b8cfdcfea94d99eab6732a878f7fd97babdf. Exact population: blake3:54982556bc986ad8aa59bb408945fad85a5990b2afe29eb1d1b11d5db19e44c9. Primary tensor-file CID: blake3:96f154042f0fd920c7f6f3b1b650a6ce20f11c401f9ae0c81734f47ae231b7f1.

C0 and the measured 1/4/8-thread CPU matrix are running before primary admission. The 900-second figure is the safety-adjusted admission projection; an additional epoch-boundary guard preserves an incomplete result if exceeded before a later epoch, rather than misreporting a model miss. Primary remains one seed, one learning rate, at most 64 epochs, no same-issue tuning. The binding-permuted model evaluation remains forbidden until a primary strict >99% pass. Broad QA is dormant; queue acknowledgements are transport, not test evidence.

### comment 5510807236 — 2026-09-02T14:04:48Z — Casey-allard

C0 PASS: literal source loader/model golden checks, causal/query projection mechanics, and tiny overfit 128/128 after87 updates. Preflight CID blake3:93f11afcc7734d6305e965646a88f54a6c7d1db11e7d7891f50f1f30845a1cf9.

All CPU plans were stable and repeat-deterministic. Safety-adjusted64-epoch projections:1thread540.388s,4threads367.999s,8threads377.026s. Peak RSS respectively1,172,389,888 /1,113,374,720 /1,200,177,152bytes. Selected CPU4 intra-op/1 inter-op, one training process, batch512; measured training CPU use1.89–1.94 core-equivalents. CUDA/MPS forbidden and unused.

The single independently initialized primary is now admitted and launching. Fixed LR0.00046415888336127773, seed123, source train/development shared Torch RNG, strict >99% early stop, maximum64epochs/4,194,304 training query presentations. Only a primary pass permits the existing binding-permuted evaluation; a miss ends this issue without geometry changes or tuning.

### comment 5510909386 — 2026-09-02T14:11:53Z — Casey-allard

The single primary completed at the frozen 64-epoch cap: STOCK_CELL_TRANSFER_MISS. Final assignment-disjoint development was 984/8,192 (12.01171875%), NLL 6.79664158821106. Work: 1,024 optimizer updates, 4,194,304 training query presentations and 524,288 development presentations. Primary wall was 324.061369 seconds; total run wall 325.013940 seconds. The binding-permuted evaluation remains NOT_RUN_PRIMARY_MISS, with zero control model queries and no further fitting.

Fresh-process artifact inference and causal result bindings reproduced exactly. Result CID: blake3:e2d1deb55a4612015ba924a94051beacd517f3c062c714c4972ba954f57621a1. Artifact CID: blake3:4857d0fdaba7d3cbc64acba39ffb43200c2d6d78ae312aded1c5c89389505fed. State CID: blake3:0ded701c7eca62c7fde69f619d63921079519358a31ab1b72bab60b6a34b27c1.

Interpretation correction: this result does not uniquely identify a serialization defect. With batch 512, source #1050 had 196 optimizer updates per epoch, whereas these exact bytes supply 16. Our entire 64-epoch cap therefore allowed only 1,024 updates and annealed the learning rate nearly to zero; source binding first jumped at 2,744–2,940 updates and passed at 3,920. Query exposure also differs (4,194,304 here versus 8,000,000 at the source pass). This is a bounded population/training-recipe transfer miss, not an ordinary-attention or geometry falsifier. The positive #1050 result remains unchanged.

Recommendation for a separate issue: retain exact data/model/batch/initial LR and fresh initialization, but align the optimizer/scheduler clock with the source-positive run, capped at 3,920 updates. Matching updates would not match total query work because this task has eight rather than four K/V pairs. No such run, geometry change, sweep or extra evaluation is authorized or executed within #1053. Documentation and protected delivery are being completed now.

### comment 5511030274 — 2026-09-02T14:20:14Z — Casey-allard

Delivered and closed through protected PR #1054 at merge commit 1cc1ce115a84d5d10b13d0c97ec0232425b70e0b.

DoD complete: exact source/data/dependency/preparation/preflight/artifact/state/result identities and read/work ledgers are in docs/r4_zoology_exact_transfer_1053.md; current README, ROADMAP, RESEARCH and completion-plan summaries are reconciled, with an append-only #1050 successor note. Thirty-one focused tests, C0, formatting/baseline lint, claim wording and diff integrity passed. Fresh-process inference reproduced the exact final result, and the independent checkpoint audit matched all 20 exported tensors and all 1,024 optimizer-step counters. Queue acknowledgements supplied transport only, not QA evidence.

Terminal remains STOCK_CELL_TRANSFER_MISS: 984/8,192 (12.01171875%), NLL 6.79664158821106, 324.061369 seconds primary wall. Result CID blake3:e2d1deb55a4612015ba924a94051beacd517f3c062c714c4972ba954f57621a1. Control NOT_RUN_PRIMARY_MISS; no geometry change, second fit, sweep, English/generation/reasoning/lowering or broad QA ran. #1050 remains positive. The separately scoped optimizer-clock correction is a recommendation only; #973 remains the active parent and #954 remains blocked.


---
_Comment/review ids in this file: 5510794765, 5510807236, 5510909386, 5511030274_

