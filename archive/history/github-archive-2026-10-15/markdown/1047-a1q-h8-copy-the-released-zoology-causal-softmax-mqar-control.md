# #1047 — A1Q-H8: copy the released Zoology causal-softmax MQAR control

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T07:19:16Z
- **Updated:** 2026-09-02T08:38:03Z
- **Closed:** 2026-09-02T08:38:03Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1047
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Parent and decision

Child of #973 and successor to #1045. Blocked until #1045 lands through PR #1046.

#1045 proved that the inherited ordinary causal-softmax path can fit seen assignments (`99.945068%` construction) but stopped at `87.121582%` on assignment-disjoint development. A no-write replay localized `893/1,055` errors to another value already admitted in the same row; in `739/893` such errors, two second-layer heads both selected the wrongly predicted value slot. Diagnostic CID: `blake3:ce11698f62561afb6d8ee5e8f816474df389802559d5e6519bff498c735b7736`.

This issue answers one question: **does the released Zoology one-head causal-softmax cell generalize on the same open bytes where the inherited four-head UOR cell missed?** It copies and credits the released mechanism before another R4-specific change.

## Source authority and attribution

Executable oracle: HazyResearch/Zoology ICLR24 tag `de4e258784224e09909c257ff3ea040f089ed660`, Apache-2.0.

- Figure 2 configuration: https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/experiments/paper/figure2.py
- MQAR loader: https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/data/associative_recall.py
- attention: https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/mixers/attention.py
- block/model/init: https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/model.py
- trainer: https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/train.py
- license: https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/LICENSE.md

#1045 cited later commit `1ad20d193b6113cae1e8f3c655c300d7b4b3f4bb`; bind it as provenance only. The ICLR24 release is authoritative because the later tree changed batching and hard-coded CUDA in token embedding construction. Preserve an Apache-2.0 notice and identify every adaptation.

## Frozen mechanism

Policy: `ZoologyMQARControlV1`.

Port the released discrete equations and initialization into a nested trainer package:

- fresh seed 123; source normal initialization including its second model-level `apply`
- width 64, two released `TransformerBlock`s, one 64-dimensional attention head
- learned absolute positions; biased combined QKV and output projections
- pre-norm LayerNorm residual structure; identity state mixer, no MLP
- attention dropout `0.1`, embedding dropout `0.1`, residual dropout `0`
- tied token/output embeddings
- query-position-only vocabulary projection, mathematically identical to masking all unlabelled logits but required for bounded CPU memory
- AdamW defaults, weight decay `0.1`, exact second released LR `0.0004641588833612782`, no gradient clipping, 64-epoch cosine schedule to zero

Deliberate adaptations: device-neutral CPU construction; query-only gather; deterministic release-style shuffle; create-once UOR provenance/result containers. No CUDA or MPS. Do not call this byte-identical or a full published Figure 2 reproduction.

## Ordered open controls

The runner stops at the first miss.

### C0 — port mechanics

Bind golden loader/model outputs generated from the ICLR24 release and require exact integer loader parity plus scale-aware float parity for attention/model equations. Overfit 32 open rows at 100% query top-1 within 256 updates. Any miss is `INVALID_CONTROL_PORT` and authorizes no scientific inference.

### C1 — source-native calibration

Use the released `_mqar` serialization and distribution with zero filler, seed 0/10, `input_seq_len=64`, four K/V pairs, vocabulary 8,192, 8,192 construction rows and 1,024 development rows. This scaled calibration is not the published 100,000/3,000 experiment. Train the frozen cell with batch 64 for at most 64 epochs and require two consecutive construction `>=99.5%` and development `>=99%` checks. A miss is `SCALED_SOURCE_CALIBRATION_MISS`; stop before interpreting the UOR bytes.

### C2 — exact #1045 open bytes

Use #1045's exact 8,192 train and 1,024 assignment-disjoint development row identities, token bytes, selected query positions, and targets. Validate role bytes for provenance, then never pass roles, H4 frames, or the failed #1045 artifact to the stock model. Context is 120 and vocabulary is 4,096; all other model/optimizer settings stay frozen. Batch 64, maximum 64 epochs / `4,194,304` query presentations. Require two consecutive construction `>=99.5%` and development `>=99%` checks.

After a C2 primary pass only, run the existing data-level binding-permuted control and require a drop of at least 50 percentage points. Do not invent current-only, cache, transport, or attention-off interventions that do not exist in the copied cell.

## Divergent decisions

- C0 miss: `INVALID_CONTROL_PORT`; repair only source parity.
- C1 miss: `SCALED_SOURCE_CALIBRATION_MISS`; do not modify R4. The next decision, outside this issue, is whether the full released 100,000-row calibration has sufficient value for its measured cost.
- C2 construction miss, or primary metrics pass only once: `STOCK_CELL_EXACT_QUALIFICATION_MISS`; make no assignment-disjoint transfer inference. Isolate exact-byte fit or temporal-qualification mechanics only in a new frozen contract.
- C2 train pass, development miss: `STOCK_CELL_TRANSFER_MISS`; the assignment-disjoint/random-filler/reduced-population contract is not covered by the copied positive. Do not change R4 yet; next isolate source serialization versus assignment-disjointness in one new contract.
- C2 development pass but binding-permuted drop `<50pp`: `NONASSOCIATIVE_SHORTCUT`; do not accept as attention evidence.
- C2 development pass plus binding-permuted drop `>=50pp`: `STOCK_CELL_PASSES_EXACT_BYTES`; the #1045 miss is localized to inherited model/integration mechanics. Next align the R4 cell to the proven one-head 64-dimensional addressing boundary before English transfer.

## CPU run contract

    metric to move:       #1045 exact-byte assignment-disjoint dev top-1, 87.121582% -> >=99%
    reachability ceiling: every query has one physically admitted source K/V; 100% reaches attention
    cheap instrument:     source parity + C0 + measured 1/4/8-thread batch-64 timing
    proceed only if:      C0 passes; projected C1+C2 <=900s; peak RSS <=8GiB
    if positive:          require binding-permutation drop, then align R4 addressing width/head shape
    if negative:          use the exact stop code above; do not tune this issue
    cost estimate:        timing proxy projects C2 about 317s; C1+C2 hard wall 900s

Measure at least 32 timed training batches plus full development evaluation and include a 1.25 safety factor for final scoring, export, and fsync. Select the fastest eligible CPU thread count from 1, 4, and 8; use all useful cores, not a nominal fixed count. One learning arm per rung, no LR/width sweep.

## Files and invariants

Add only a nested package such as `r4_softmax_trainer/zoology_control/`, its focused tests, Apache attribution, and `docs/r4_zoology_mqar_control_1047.md`. Invoke it with `python -m r4_softmax_trainer.zoology_control`; do not modify the #1045 top-level implementation files or immutable evidence. Bind the recursive #1047 implementation tree, both Zoology source hashes, #1045 result/split identities, all generated populations, selected CPU plan, work ledger, artifact, and result CIDs.

Forbidden: #1045 fitted artifact as model input; any sealed payload; teacher/provider/Ollama/Gemma; CUDA/MPS; hyperparameter or width sweep; R4/geometry claims; English, generation, reasoning, recurrence, quantization, exact lowering, browser/WASM, product, or release work.

## Definition of done

- source-derived code is credited and parity-checked
- one create-once CPU preflight and at most one run per reached rung
- exact metrics, CIDs, read/work ledger, controls, and stop decision recorded in-tree and on this issue
- focused tests and claim-language checks pass
- result delivered by PR/merge queue
- successor action follows the frozen decision branch; no same-issue tuning


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

