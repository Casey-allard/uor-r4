# Released Zoology MQAR integration control (#1047)

- **Execution status:** preflight complete; C1/C2 `NOT_RUN_PREFLIGHT`
- **Policy:** `ZoologyMQARControlV1`
- **Authority:** [issue #1047](https://github.com/UOR-Foundation/uor-r4/issues/1047),
  child of #973 and successor to #1045
- **Source oracle:** HazyResearch/Zoology ICLR24 release
  [`de4e258784224e09909c257ff3ea040f089ed660`](https://github.com/HazyResearch/zoology/tree/de4e258784224e09909c257ff3ea040f089ed660),
  Apache-2.0
- **Provenance-only source:** later Zoology commit
  [`1ad20d193b6113cae1e8f3c655c300d7b4b3f4bb`](https://github.com/HazyResearch/zoology/tree/1ad20d193b6113cae1e8f3c655c300d7b4b3f4bb)
- **Claim boundary:** an open, CPU-only control of ordinary one-head causal
  softmax on MQAR; no R4, geometric-attention, English, generation, reasoning,
  recurrence, quantization, exact-lowering, browser/WASM, product, or release
  claim

This document preserves the frozen control and appends its execution evidence.
The runner stops at the first missed rung. At contract publication, the CPU
preflight and C0, C1, C2, and binding-permuted control were all `NOT_RUN`; the
publication ledger below retains that state. The executed preflight result is
appended after it. The frozen issue made #1045's landing through PR #1046 a
prerequisite.

## Decision and inherited evidence

#1045 measured near-perfect fit of seen assignments in its inherited ordinary
four-head causal-softmax cell but missed assignment-disjoint development:

- construction: `65,500/65,536` (`99.945068%`);
- final development: `7,137/8,192` (`87.121582%`);
- frozen result CID:
  `blake3:d920ad7b7f373c55cb564e27b3ddb1af8949a20c432e0d7cd2b39f1f69999557`;
- fitted-artifact CID, recorded for provenance and forbidden as #1047 model
  input:
  `blake3:92bb13caf71c9ef44885a9da39023d080de075118b5902b716d2ca9b0f61f611`;
- assignment-disjoint split CID:
  `blake3:d36937f974e5e96dc697b219db8a7eb448dff7192abdf88bf6b21000f58b1f48`.

An independent, no-write replay localized `893/1,055` development errors to a
different value already admitted in the same row. In `739/893` of those
errors, two second-layer heads both selected the wrongly predicted value slot.
The diagnostic record is
`blake3:ce11698f62561afb6d8ee5e8f816474df389802559d5e6519bff498c735b7736`.
This is diagnostic localization, not an attention or geometry attribution.

#1047 therefore asks one question: does the released Zoology one-head
causal-softmax cell generalize on the exact open #1045 bytes where the inherited
four-head cell missed? The control copies and credits the released mechanism
before any further UOR-specific model change.

## Source authority and attribution

The executable oracle is the ICLR24 release at `de4e258...`:

- [Figure 2 configuration](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/experiments/paper/figure2.py)
- [MQAR loader](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/data/associative_recall.py)
- [data-loader mechanics](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/data/utils.py)
- [causal attention](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/mixers/attention.py)
- [block, model, and initialization](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/model.py)
- [trainer](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/zoology/train.py)
- [Apache License 2.0](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/LICENSE.md)

The later `1ad20d1...` commit cited by #1045 is bound only as provenance. It is
not the executable oracle: that later tree changed batching and hard-coded CUDA
in token-embedding construction. Source-derived files belong under
`r4_softmax_trainer/zoology_control/` and are covered by that directory's
`NOTICE.md`. Every adaptation is named below. This is neither a byte-identical
port nor a reproduction of the full published Figure 2 sweep.

## Modulo-256 boundary

Modulo-256 semantics remain appropriate for discrete role codes, byte tables,
and possible later lowering. They do not implement continuous probability
normalization: `Z/256Z` has no general division and cannot represent even every
elementary normalized distribution. This control therefore retains the
released real-valued, numerically ordinary softmax.

The #1045 categorical `uint8` role bytes are validated only to prove that C2
loaded the intended rows. They are never embedded or passed to the Zoology
cell. H4 frames and all other geometric sidecars are also excluded. A positive
result would concern this ordinary causal-softmax control only.

## Frozen cell and training mechanics

The nested implementation copies the released discrete equations and
initialization with this one fixed configuration:

- fresh model/training seed `123`;
- model width `64` and two released `TransformerBlock`s;
- one attention head of dimension `64`;
- learned absolute position embeddings;
- biased combined Q/K/V projection and biased output projection;
- the released pre-norm residual flow, final residual merge, and final
  LayerNorm;
- identity state mixer and no MLP;
- attention dropout `0.1`, embedding dropout `0.1`, residual dropout `0`, and
  no stochastic-depth drop;
- tied token and vocabulary-output embeddings;
- the released normal initialization, including the second model-level
  `apply` and its residual-output rescaling;
- AdamW defaults with weight decay `0.1` and learning rate exactly
  `0.0004641588833612782` (the second released Figure 2 learning rate);
- no gradient clipping;
- a 64-epoch cosine schedule to zero.

The copied attention computes the combined Q/K/V projection, reshapes to one
head, scales by the inverse square root of head width, adds the released causal
upper-triangular mask, applies softmax and training-only attention dropout,
aggregates values, and applies the output projection. C0 binds these equations
and the released block/model initialization to source-generated golden values;
this prose is not a substitute for parity.

### Declared adaptations

Only these integration adaptations are permitted:

1. Construct and run on CPU without the released CUDA-only device assumption;
   CUDA and MPS remain forbidden.
2. Gather hidden states at labelled query positions before the tied vocabulary
   projection. This gives the same labelled logits and query-only
   cross-entropy as projecting every position and masking all unlabelled logits,
   while bounding CPU memory.
3. Use a deterministic form of the release's shuffled data-loader order so a
   repeated open run has a bound ordering.
4. Use create-once UOR provenance and result containers to bind inputs, plans,
   reads, work, artifacts, and decisions.

C1 is deliberately scaled from the published experiment, and C2 deliberately
substitutes the exact #1045 open serialization. Batch 64, the two-consecutive
gate, query-only scoring, the CPU timing gate, and create-once records are
control-contract choices. They must not be described as the full published
100,000/3,000 Figure 2 run.

## Ordered open controls

The controls run in order and stop at the first miss. There is one learning
arm per rung and no learning-rate, width, head-count, or other hyperparameter
sweep.

### C0 — source and mechanics

Bind golden loader and model outputs generated from the `de4e258...` release.
Require:

- exact integer parity for released loader examples, labels, and selected query
  positions;
- scale-aware floating-point parity for the attention, residual block, model,
  and initialization equations; and
- `100%` query top-1 while overfitting 32 open rows within at most 256 updates.

Any miss is `INVALID_CONTROL_PORT`. Repair only source parity; the miss permits
no scientific inference and C1 does not start.

### C1 — scaled source-native calibration

Use the released `_mqar` serialization and distribution with:

- zero filler (`random_non_queries=False`);
- construction seed `0` and development seed `10`;
- `input_seq_len=64`, four K/V pairs, power parameter `0.01`, and vocabulary
  `8,192`;
- 8,192 construction rows and 1,024 development rows;
- batch 64 and at most 64 epochs.

Require two consecutive evaluations in which construction query top-1 is
`>=99.5%` and development query top-1 is `>=99%`. This is a scaled
source-native calibration, not the published 100,000-construction/3,000-test
experiment. A miss is `SCALED_SOURCE_CALIBRATION_MISS`: stop before
interpreting the UOR bytes and do not modify R4. Outside this issue, the next
decision is whether the full released calibration is worth its measured cost.

### C2 — exact #1045 open bytes

Load #1045's exact 8,192 construction and 1,024 assignment-disjoint
development row identities, token bytes, selected query positions, and
targets. Validate the role bytes for provenance, then exclude roles, H4 frames,
and the failed #1045 fitted artifact from the model. Freeze context at `120`,
vocabulary at `4,096`, and every other cell/optimizer setting above.

Train with batch 64 for at most 64 epochs, capped at `4,194,304` query
presentations. Require two consecutive evaluations in which construction query
top-1 is `>=99.5%` and assignment-disjoint development query top-1 is `>=99%`.

Only after the primary C2 development gate passes, run the existing data-level
binding-permuted control. It must reduce development top-1 by at least 50
percentage points. Do not invent current-only, cache, transport, or
attention-off interventions absent from the copied cell.

## Frozen decisions

| Observation | Verdict | Binding next action |
| --- | --- | --- |
| C0 misses any parity or overfit gate | `INVALID_CONTROL_PORT` | Repair source parity only; make no scientific inference. |
| C1 misses its two-consecutive calibration gate | `SCALED_SOURCE_CALIBRATION_MISS` | Stop before C2; do not modify R4. Decide in a separate issue whether the full 100,000-row release calibration merits its measured cost. |
| C2 misses construction or the two-consecutive qualification | `STOCK_CELL_EXACT_QUALIFICATION_MISS` | Make no assignment-disjoint transfer inference. Isolate exact-byte fit or qualification mechanics only in a new frozen contract. |
| C2 passes construction but misses assignment-disjoint development | `STOCK_CELL_TRANSFER_MISS` | The copied positive does not cover the assignment-disjoint/random-filler/reduced-population contract. Do not change R4; isolate source serialization versus assignment-disjointness in one new frozen contract. |
| C2 development passes, binding-permuted drop `<50pp` | `NONASSOCIATIVE_SHORTCUT` | Do not accept the primary score as attention evidence. |
| C2 development passes, binding-permuted drop `>=50pp` | `STOCK_CELL_PASSES_EXACT_BYTES` | Localize the #1045 miss to inherited model/integration mechanics; next align the R4 cell to the demonstrated one-head, width-64 addressing boundary before English transfer. |

No branch authorizes tuning or rerunning #1047. The final row specifies a
future engineering direction; it is not itself an R4 or geometric-attention
result.

## CPU run contract

    metric to move:       #1045 exact-byte assignment-disjoint dev top-1, 87.121582% -> >=99%
    reachability ceiling: every query has one physically admitted source K/V; 100% reaches attention
    cheap instrument:     source parity + C0 + measured 1/4/8-thread batch-64 timing
    proceed only if:      C0 passes; projected C1+C2 <=900s; peak RSS <=8GiB
    if positive:          require binding-permutation drop, then align R4 addressing width/head shape
    if negative:          use the exact stop code above; do not tune this issue
    cost estimate:        timing proxy projects C2 about 317s; C1+C2 hard wall 900s

Run one create-once preflight. Measure at least 32 timed training batches and a
full development evaluation for each 1-, 4-, and 8-intra-op-thread batch-64
plan. Apply a `1.25` safety factor for final scoring, export, and fsync. Select
the fastest plan that projects C1+C2 within 900 seconds and holds peak RSS at
or below 8 GiB. If none is eligible, do not launch. The combined C1+C2
execution has a 900-second hard wall and receives no same-issue budget
extension. Use CPU only, one worker/arm as frozen by the implementation, and at
most one run for each reached rung.

## Provenance, ledger, and forbidden inputs

The create-once records must bind:

- the recursive #1047 implementation tree;
- authoritative Zoology hash `de4e258...` and provenance-only hash
  `1ad20d1...`;
- the #1045 result and assignment-disjoint split identities above;
- every generated C0/C1/C2 population and selected query/target sequence;
- the measured 1/4/8-thread plans and selected CPU plan;
- the read/work ledger; and
- the eventual artifact and result CIDs.

The package is invoked as:

```text
python -m r4_softmax_trainer.zoology_control
```

Forbidden inputs and actions are the #1045 fitted artifact as model input; any
sealed payload; future tokens or labels as inputs; teachers, providers,
Ollama, or Gemma; CUDA or MPS; any hyperparameter/width sweep; and R4/geometry,
English, generation, reasoning, recurrence, quantization, exact lowering,
browser/WASM, product, or release work. The #1045 top-level implementation and
immutable evidence remain untouched.

## Definition of done

- Source-derived code is credited and passes the frozen source-parity checks.
- One create-once CPU preflight and at most one run per reached rung are
  recorded.
- Exact metrics, CIDs, source/read/work ledger, controls, and stop decision are
  recorded in-tree and on #1047.
- Focused tests and the repository claim-language check pass.
- The result lands through a PR and the protected merge queue.
- The successor follows the frozen decision branch; no same-issue tuning occurs.

## Publication ledger

| Item | State at contract publication |
| --- | --- |
| Source/license review | bound to `de4e258...`; later `1ad20d1...` is provenance only |
| #1045 diagnostic | independently reproduced; CID and counts recorded above |
| CPU preflight | `NOT_RUN` |
| C0 source/mechanics | `NOT_RUN` |
| C1 scaled source-native calibration | `NOT_RUN` |
| C2 exact #1045 bytes | `NOT_RUN` |
| Binding-permuted control | `NOT_RUN` (conditional on C2 primary pass) |
| #1047 artifact/result | not created |

## Executed result — 2026-09-02

The create-once preflight completed correctly and stopped before C1 because no
frozen CPU plan met the 900-second admission policy. This is
`NOT_RUN_PREFLIGHT`, with no scientific verdict. It neither supports nor
falsifies the copied attention cell.

C0 passed every source/integration gate:

- the literal `de4e258...` loader inputs and labels were byte-exact;
- model parameters and selected logits were byte-exact, with scale-aware
  parity also passing;
- initialization replay, causal-prefix parity, and query-only projection
  parity passed; and
- the 32-row open overfit reached `128/128` query top-1 in 87 updates
  (`11,136` query presentations).

The preflight measured each arm under every frozen plan and applied the
predeclared `1.25` safety factor:

| CPU plan | Projected C1+C2 | Peak RSS | Admission |
| --- | ---: | ---: | --- |
| 1 thread, batch 64 | `1,522.035465 s` | `529,629,184` bytes | over wall |
| 4 threads, batch 64 | `1,036.667109 s` | `567,345,152` bytes | over wall |
| 8 threads, batch 64 | `959.212581 s` | `546,619,392` bytes | over wall |

The 8-thread plan used all eight physical/logical cores reported by the host
(four performance and four efficiency cores), but still projected `59.212581`
seconds above the frozen wall. Memory, deterministic replay, both-arm timing,
and full-development timing all passed. The selected plan is therefore `null`;
C1, C2, binding permutation, and model artifacts are `NOT_RUN_PREFLIGHT`.

The immutable identities are:

- implementation tree:
  `blake3:c848c05ae53bc3adc0a8f7099ceed43657b6348e4e00fe3aaef5cf1368cc38de`;
- preparation:
  `blake3:560dd6e9abbabbeccbb32b2dbec6b815b5f7842bbb30c387af79177bd32a98f8`;
- preflight:
  `blake3:78158700e632d303bf674ed544f997a0e14eb89947470f5032e6acc75c830c9b`;
- result:
  `blake3:b453abccc6ae0db9cc186c791aba268555dc0e75fe687c994e940254b0ac9ef6`.

All forbidden, sealed, future-value, role-model-input, H4-model-input,
provider, teacher, cache, and transport reads were zero. The binding action is
a fresh issue that preserves the exact cell, source/data identities, batch 64,
optimizer, gates, and CPU-only scope while widening only the execution wall
enough to admit the already measured all-core plan. That is a resource-policy
continuation, not tuning or rerunning #1047.
