# Compound-binding preservation through unchanged R4 — #1075

## Frozen contract (2026-09-02; before fitted-model inference)

Issue [#1075](https://github.com/UOR-Foundation/uor-r4/issues/1075) continues
#973 after [#1073](r4_zoology_compound_binding_1073.md), delivered in #1074 at
`d7a369cb53a0311ff35111bb20a99c1aaba7dfde`. The compound model qualified at
100% supported/absent construction and held-out-combination development in
all four cyclic fact orders, and separately passed its value-only causal control. This
experiment preserves that exact learned behavior through the existing R4
gauge law. It performs zero training or new-population generation.

The source artifact is 1,148,672 bytes, with 286,976 unique parameters:
`blake3:9c055cc6ea09548bf960e37288276535b30515b94a50a96aa929b5e55afea3c4`.
Its learned-state identity is
`blake3:abbdbcaafc2d9eb36543ce75fbb0101b6788119d80a6ed9c017bb9d06fbeac59`.
Bind all four published #1073 envelopes and the 270-file source closure.
Load no checkpoint, optimizer or saved RNG payload. Keep original data,
vocabulary, parameters, aliases, normalizations, projections and tied full
4,096-token head unchanged.

## Rectangular coordinate interface

Reuse the complete native #1059 frame export qualified by
[#1061](r4_zoology_exact_coherent_inference_1061.md). The frame bundle tree is
`blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c`.
Keep its 8,192-token map, 120 registered f64 matrices, exact multiplication
table and native prefix witnesses unchanged. Do not slice the map, rebuild
native geometry, change precision, or expand its reachable subgroup.

The old square-attention wrapper cannot directly accept this model. Add a
parameter-free rectangular wrapper while leaving historical source untouched.
Ordinary execution delegates to the frozen source model. Coherent execution
reuses the source's owner-object Q/K and location-V projection modules.

Frame assignment folds **only input positions 0 through 37**, using the
existing native ordered prefix law. Query frame is at position 37; fact K/V
frames are at positions 7, 15, 23 and 31. The learned null has no token source
and uses the atlas identity frame. Fixed grammar supplies these roles; no
target, equality matcher, answer filter or future token enters assignment.

**Definition — coordinate execution.** Treat each 64-dimensional vector as
16 consecutive four-lane blocks. For query frame Fq and true source frame Fj,
including the fifth/null entry:

```text
q_local = transpose(Fq) q
k_local_j = transpose(Fj) k_j
v_local_j = transpose(Fj) v_j
T_qj = transpose(Fq) Fj
score_j = f32(sum_over_16_blocks(q_local dot (T_qj k_local_j))) / 8
a = softmax_f32(score)
context = f32(Fq sum_over_all_5_entries(a_j T_qj v_local_j))
logits = original_head(original_output_LN(original_Wout(context)))
```

Frame operations, dot accumulation and weighted value transport use native
f64; softmax uses f32. Match #1073's arithmetic boundary: cast the completed
dot to f32 **before dividing by eight**, rather than prescaling its keys as
the older square adapter does. Decode to f32 before the original output
projection. Return actual attention `[batch,1,1,5]` and full-head logits.
Orthogonal coordinate cancellation motivates the comparison; finite-precision
behavior must satisfy the empirical criteria below.

Encode, transport, score and aggregate **all five entries**, including null.
In #1073, mean null attention was 54.64% on supported construction and 91.25%
on absent queries. Neither hard fact selection nor an argmax-null absence rule
would preserve the demonstrated mechanism.

## Population, exact reference and primary decision

Use the same 10,240 construction rows (8,192 supported, 2,048 absent) and
1,280 development rows (1,024 supported, 256 absent), in their original
canonical file order and right-cyclic fact rotations 0/1/2/3. These are
**46,080 decisions per arm**, comprising 36,864 supported and 9,216 absent.
The rotations are correlated views; development was already examined in
#1073. This is preservation on observed held-out combinations, not an additional
fresh or sealed generalization result.

Keep batch size **256**, eight CPU/Apple Accelerate threads and one inter-op
thread, matching #1073. Reuse its rectangular scorer and exact reduction/digest
ordering. All eight ordinary views must reproduce **every field** of their
published all/supported/unknown score records before any R4 forward. Those
records include counts, NLL and full logits/prediction/attention CIDs. An
ordinary mismatch stops with `COMPOUND_R4_REFERENCE_MISMATCH` and zero R4 or
control decisions.

**Empirical preservation criteria**, in every population/order and every
all/supported/unknown stratum:

- Identical top-1 predictions.
- Maximum absolute difference across the full vocabulary logits **<=0.005**.
- Maximum attention-weight difference **<=1e-5**.
- Absolute mean NLL difference **<=1e-5 nats**.
- Complete causal support, null participation and transport accounting; no
  future reads or future score slots. All group-level binding counts preserved.
- Unchanged model state, parameter count, tied head, eval mode, source/data,
  native frames and implementation identities before/after execution.

These are the existing #1059/#1061 preservation tolerances, not a replacement
learning threshold. Record decoded prediction IDs, group summaries, all
stratified scores/digests, differences and measured work.

## Conditional inconsistent-transport control

Only after **all eight primary views pass**, run one same-model, same-support
control. Encode each K/V in its true frame, but substitute source frame
`(j + 1) modulo 4` in the transport connection for the four facts. Leave the
null identity frame fixed. Do not combine this with #1073's value-cycle control.
Queries, original K/V, the five support entries, full softmax, parameters and
work remain unchanged; the resulting attention need not remain unchanged.

Require complete supported/absent decisions, equal coherent/control work,
causal reads, all four fact frame positions shifted and actual matrix changes
matching the preflight. Only a valid control with **at least 50 percentage
points of supported accuracy loss in every population/order** establishes
strong transport sensitivity. Report changed predictions, group behavior and
absent-query effects separately. UNKNOWN preservation belongs to the primary
comparison; it is not a broken-transport success condition.

- Primary miss: `COMPOUND_R4_PRESERVATION_MISS`; retain the source model and
  investigate only the adapter/numerical preservation failure.
- Primary pass, invalid control: `COMPOUND_R4_PRESERVED_CONTROL_INVALID`;
  retain preservation and repair the control's integrity separately.
- Primary pass, weak control: `COMPOUND_R4_PRESERVED_CONTROL_WEAK`; retain
  preservation and reconsider the intervention without claiming strong sensitivity.
- Primary and strong valid control pass: `COMPOUND_R4_PRESERVED`; retain the
  R4 binding path and next freeze a bounded language-interface step addressing
  fixed-position role extraction. No such follow-up fit runs in this issue.
- Resource interruption: `INCOMPLETE_RESOURCE`, with last completed progress
  and consumed clock retained. No replacement run, retuning or renewed budget.

## Reachability, cheap instrument and resource cap

All 46,080 decisions traverse the new interface. Each arm has **230,400**
fact/null score slots, including 46,080 null pairs; no triangular token mask is
needed. Each R4 row encodes 16 query blocks and 80 key/value blocks each,
transports 80 key/value blocks each, and decodes 16 output blocks. Per complete
R4 arm this is **737,280 query/decode blocks** and **3,686,400 key/value
encoding and transport blocks each**. Plain has the same support and zero
coordinate-transform counts.

The label-free frame preflight, before any model forward, completed in
**0.593 seconds** and passed. Every view reaches the same **24 of 120 frames**.
Actual changed source matrices across construction rotations are
**39,213 / 39,205 / 39,070 / 39,098** of 40,960; development counts are
**4,817 / 4,817 / 4,862 / 4,814** of 5,120. All supported rows have a changed
source frame except two construction rows in rotation 2. The resulting
supported-loss reachability ceilings are **100% everywhere except
99.9755859375% there**, above the 50-point criterion. These are opportunity
ceilings, not predicted model losses. Actual frame-matrix changes, rather than
merely shifted positions, remain part of the control integrity record.

Use **900 seconds cumulative for run plus replay**, **4 GiB peak RSS**, one
CPU process, eight intra-op threads and one inter-op thread. The clock includes
live source/frame/preflight validation. Retaining ordinary full-head logits for
all eight views uses 754,974,720 bytes (720 MiB). The run/replay pair at full
three-arm coverage has 276,480 decisions. Scaling #1073's 7.146-second,
87,040-decision evaluation by that volume and a conservative 4x adapter allowance
projects about **91 seconds**; the measured cap, not this estimate, is binding.

## Review, freeze and execution

Fourteen focused synthetic checks cover causal frame assignment, full-width
scaling/null aggregation, transport-control equations, source-state/RNG
preservation, historical binding/root relocation, exact-reference mapping,
per-stratum tolerances, work integrity, conditional control access, preflight
ceiling and complete question groups. Independent source review found no
blocker. Broad workspace, BDD, WASM, fuzz, audit, native rebuild/export and old
mechanics campaigns remain dormant. Queue statuses acknowledge transport only.

Publish source and the immutable preparation before the first fitted forward.
One run evaluates ordinary, coherent and conditional control; one fresh-process
replay must reproduce the complete deterministic evidence. Exclusive start
markers reject silent repetition. Source/model/frame bindings and learned
state are checked before and after. This offline floating-point experiment
does not change the deployed integer/table runtime or establish H4 superiority,
softmax removal, learned English parsing, free generation, reasoning or chat.

From `tools/r4-softmax-trainer` in the frozen offline environment:

```bash
.venv/bin/python -m r4_softmax_trainer.zoology_compound_r4 prepare \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1075-compound-r4 \
  --source-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1073-zoology-compound-binding \
  --frame-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1059-zoology-r4-inference/frames
.venv/bin/python -m r4_softmax_trainer.zoology_compound_r4 run \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1075-compound-r4
.venv/bin/python -m r4_softmax_trainer.zoology_compound_r4 verify \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1075-compound-r4
```

Append the observed result, reconcile six current mirrors and native trackers,
and close only #1075 after protected delivery. #973 stays open and #954 blocked.


## Published pre-inference preparation (2026-09-02)

Source freeze: `d18e00705b1c6523f52c3fb9cacf029e548f2631`. Fourteen focused checks passed
(0.064 seconds), along with Ruff, claim wording, and independent source/record
review. There have been zero fitted-model forwards in this issue.

The [immutable preparation](r4_zoology_compound_r4_1075_preparation.json) binds:

- Preparation: `blake3:f034f6af62f0f1e6f1f7be33a93f728bfd2582b1ea10025ec1c1f5ae1835240b`.
- 282-file implementation: `blake3:af31d67ed421af579bbdd10b370285898d1f0fbb50ac07c2ad2dd69e5ce30331`.
- Source result: `blake3:1f3c5bee5ebd0e8e34f9f1a5fa03d514b397928638fd66deaf64b8abf7946041`.
- Source model: `blake3:9c055cc6ea09548bf960e37288276535b30515b94a50a96aa929b5e55afea3c4`.
- Source data manifest: `blake3:574d667e61b70e32c39b26d43547d5aeb29e92f16260fa840ddd4eda30c4e694`.
- Unchanged native frame bundle: `blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c`.

The actual preparation reproduces the eight-view frame preflight and every
source/data/frame binding. Preparation reads already observed inputs and hashes
the fitted file without instantiating its model. The primary, conditional
control and fresh-process replay remain unrun at this publication.


## Observed result (2026-09-02)

**`COMPOUND_R4_PRESERVED`.** Ordinary execution exactly reproduced all eight
published #1073 score records in all three strata before the first R4 forward.
Coherent R4 then preserved all **46,080 top-1 predictions**: 36,864 supported
and 9,216 absent decisions, across both observed populations and all four orders.
The unchanged 286,976-parameter model retained 100% supported and UNKNOWN
accuracy in every view. The valid inconsistent-transport control exceeded the
frozen 50-point supported-loss criterion in every view.

### Primary preservation

| Population | Fact orders | Supported correct per order | Absent correct per order | Complete supported quartets per order |
|---|---|---:|---:|---:|
| Construction | 0, 1, 2, 3 | 8,192 / 8,192 | 2,048 / 2,048 | 2,048 / 2,048 |
| Observed development | 0, 1, 2, 3 | 1,024 / 1,024 | 256 / 256 | 256 / 256 |

Both question families remain complete: each order preserves all 1,024
construction quartets per family and all 128 development quartets per family.
All five-answer groups, including UNKNOWN, remain complete. The worst
differences over every view and all/supported/unknown strata were:

| Quantity | Observed maximum | Frozen ceiling |
|---|---:|---:|
| Full 4,096-head logit absolute difference | 0.000006794929504394531 | 0.005 |
| Attention absolute difference | 0.00000035762786865234375 | 0.00001 |
| Mean NLL absolute difference (nats) | 0.00000000046475179260596633 | 0.00001 |
| Changed top-1 predictions | 0 | 0 |

The complete four-fact-plus-null softmax mixture traversed the adapter. Each
R4 arm accounted for 230,400 admitted/materialized score slots, 46,080 null
pairs, 737,280 query-encoding/output-decoding blocks each, and 3,686,400 blocks
each for key/value encoding and transport. Every view reached the same 24
native frames. Future-position reads and future score slots were zero.

### Same-work broken transport

The four fact source-frame positions were cycled only in the connection; true
K/V encodings and the null identity remained fixed. All integrity checks passed
in every view, including equal coherent/control work and the exact preflight
matrix-change counts (175,896 actual changes across 184,320 shifted fact
positions). The control made all 46,080 decisions.

| Population | Rotation | Supported correct | Supported loss (percentage points) | Absent correct | Complete supported quartets |
|---|---:|---:|---:|---:|---:|
| construction | 0 | 1,468 / 8,192 | 82.080078125 | 1,416 / 2,048 | 27 |
| construction | 1 | 1,442 / 8,192 | 82.397460938 | 1,392 / 2,048 | 21 |
| construction | 2 | 1,428 / 8,192 | 82.568359375 | 1,386 / 2,048 | 23 |
| construction | 3 | 1,509 / 8,192 | 81.579589844 | 1,374 / 2,048 | 30 |
| development | 0 | 94 / 1,024 | 90.820312500 | 187 / 256 | 2 |
| development | 1 | 75 / 1,024 | 92.675781250 | 183 / 256 | 1 |
| development | 2 | 78 / 1,024 | 92.382812500 | 192 / 256 | 0 |
| development | 3 | 86 / 1,024 | 91.601562500 | 189 / 256 | 2 |

Coherent R4 preserves the learned binding while inconsistent coordinates
substantially damage it. This supports transport sensitivity in this adapter.
It does **not** establish an advantage over ordinary attention or a need for
more geometry. Absent-query control effects are descriptive; the frozen
preservation criterion applies to coherent execution. The earlier value-only
causal control remains the separate #1073 result and was not combined or rerun.

### Exact replay, provenance and resources

One fresh process reproduced the complete deterministic evidence exactly.
The run took **7.763146833 seconds**, replay
**7.828975459 seconds**, cumulative
**15.592131417 seconds** against the 900-second cap.
Peak RSS was **2,221,703,168 bytes**
(**2.069122314 GiB**) against 4 GiB.
No replacement run or budget reset occurred. Runtime matched #1073: CPU,
Apple Accelerate, eight intra-op threads, one inter-op thread, Python 3.12.14
and Torch 2.7.1. These are aggregate experiment costs, not a serving benchmark.

- [Result](r4_zoology_compound_r4_1075_result.json): `blake3:358506253f842a1843dce2652f1004a792a8a6b056a361f8ac66ddd5babb31af`.
- Complete deterministic evidence: `blake3:e22b210d087b9e814a03553d2541ef4f49d0c2b39e9fe0a961ec2363bb50e5b9`.
- [Fresh-process replay](r4_zoology_compound_r4_1075_replay.json): `blake3:0310a7797f8db047cc54b9641a5b90b987b3a3b4ac706f4d9e580738f0fa8de1`.
- Preparation: `blake3:f034f6af62f0f1e6f1f7be33a93f728bfd2582b1ea10025ec1c1f5ae1835240b`.
- 282-file implementation: `blake3:af31d67ed421af579bbdd10b370285898d1f0fbb50ac07c2ad2dd69e5ce30331`.

Public envelopes are exact copies of the retained local files. The model
file, learned-state CID, tied head, parameter count, eval/no-grad state, source
data and frame identities remained unchanged before/after execution. Each
process loaded the one fixed candidate once; optimizer updates, new parameters,
checkpoint/optimizer/saved-RNG reads, new population generation, native exports
and geometry changes were all zero. Preparation review additionally guarded
model construction/loading/forward and observed zero calls.

The 14 focused synthetic checks and independent source/preparation reviews
passed before inference. The one run and one exact replay are the scientific
evidence. Broad workspace, BDD, WASM, fuzz, audit and historical mechanics QA
remain `NOT_RUN`; protected-queue compatibility statuses carry no scientific
or product qualification.

### Decision and next action

**Retain the R4 compound-binding path. Keep the current geometry.** The next
separately frozen experiment should replace fixed-position role extraction
with a bounded learned language interface: identify owner, object and location
from controlled sentence variants, then feed the established compound Q/K/V
binding path. Freeze the ordinary reference and meaningful unseen wording/order
combinations first; compare preserved answers and causal controls before any
R4 interface qualification. If learned roles fail, repair that interface while
retaining this qualified binding core. No follow-up fit runs in #1075.

This record preserves the already observed #1073 construction/development
behavior. The fixed grammar still supplies semantic roles, and cyclic fact
orders are correlated views. General English parsing, open-ended generation,
correctness, reasoning, chat, H4 superiority, softmax removal and the deployed
integer/table runtime remain outside this result. #973 stays open and #954
remains blocked.


## Delivery review (2026-09-02)

Independent review verified the actual preparation, result and replay bindings,
all primary/control counts, numerical differences, transport work, resources
and claim boundaries. Public envelope bytes match the retained local files.
The six current mirrors and programme direction now reflect this result;
claim wording and whitespace checks pass. Source remains exactly at the
published pre-inference freeze. Protected delivery and final native
tracker/milestone closure are recorded on issue #1075.
