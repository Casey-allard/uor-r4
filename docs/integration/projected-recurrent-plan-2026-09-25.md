# D8 rung 2 — projected-shadow continuation

**Frozen before execution, September 25, 2026.** The owner authorized the
[previous result's recommendation](quantized-recurrent-result-2026-09-25.md#recommended-next-comparison-projected-shadow-updates).
This comparison addresses quantized quality retention in the same jointly trained
recurrent attention/state/output learner. It preserves the transformerless
integer/table serving objective and the [canonical D8 ladder](project-track.md).
#973 owns the outcome under programme #820.

## Why this intervention

The previous recipe improves zero-update parent quantization but fails the frozen
packed-minus-continuous +0.05-nat requirement: quaternion +0.059112874, ordinary
Householder pair +0.079827433. Ordinary also produces a period-two cycle. Same-QAT
shadows evaluated without quantization remain worse than their continuous
controls. This establishes a retention deficit; it does not isolate clipping,
scale choice, exposure or transport as the unique cause.

The current clipped STE has derivative one inside and **at** the endpoints,
zero outside. For each existing dyadic interval define

`P(w) = clip(w, q_min*s, q_max*s)`.

The unchanged quantizer satisfies `Q(P(w)) = Q(w)`. Projecting at an already
full-hard checkpoint therefore preserves its effective hard model while changing
subsequent optimization. The original midpoint **7,836** is chosen because it is
the retained full-hard recovery boundary, independently of its quality. The ramp
from step7,324 ended earlier; this comparison never revisits its partly floating
forward computation. Every original negative remains retained.

[BNN Algorithm1](https://arxiv.org/html/1602.02830v3) supplies precedent for clipping
updated real weights. [LSQ](https://arxiv.org/html/1902.08153v3) has a different
strict-interior weight derivative and learned scales. Their image-classifier
results are not evidence of success here. The present comparison adopts neither
learned scales nor LSQ's endpoint rule.

## Treatment, inherited computation and controls

Load the exact sealed midpoint of each prior QAT arm:

- `quantized-recurrent-20260925/fit-quaternion-qat-1/checkpoint-00007836`
- `quantized-recurrent-20260925/fit-householder_pair-qat-1/checkpoint-00007836`

The [input receipt](../evidence/projected-recurrent-inputs-2026-09-25.json) binds
checkpoint, parameters, optimizer, campaign, evaluator and retained evaluations.
Continue **512 updates to fixed final step8,348**, **2,097,152 new target visits
per arm**, **4,194,304 in total**. There is no checkpoint quality selection.
The inherited lineage remains explicit: the projected branch replaces the old
second-half trajectory for comparison; its new work is charged additionally.

Preserve model dimensions, signs, transport family, exact occurrence/token tape,
fixed row scales, interface grids, full-strength quantized forward, inclusive
STE, AdamW learning rate/weight decay, both moments, per-parameter and global
clocks, B16/T256, two complete-sequence gradient shards, data seed and counter
sampler. The inherited window and quantization-transition declarations remain
unchanged. No extra loss, scale calibration, rounding rule, corpus or architecture
change is combined with projection.

**The sole new learning policy:** project every stored parameter shadow into its
existing representable interval at declared entry and immediately after each
AdamW update, before development, checkpointing or the next forward. Preserve
interior values rather than rounding them to codes. Execute projection detached
from the differentiable graph; do not add another forward clamp. Include both
four-bit multiplicative parameters and sixteen-bit additive offsets using their
original ranges. Record counts and maximum corrections with the existing curve.

The completed unprojected second halves are the direct treatment controls.
Their continuous step8,348 counterparts remain the original quality-retention
references. Reuse their sealed evaluations on the same population; re-executing
unchanged control fitting would add no missing comparison. The same-QAT shadow
forward is reported diagnostically; disabling its quantizers does not turn its
learned trajectory into the continuous control.

Projection retains momentum. Outward moments or gradients may hold a value at
a boundary. Projection changes the decay inputs and may change global gradient
clipping through the restored gradients. Thus any effect belongs to this complete
policy; a success would not isolate gradient unmasking alone. Fixed parameter
ranges and interface precision remain possible limitations.

## Entry, resume and loaded-artifact requirements

Before any update, validate exact parent hashes, full quantization strength,
fixed specification and all unchanged learning settings. Projected policy must
be explicitly bound to the exact parent and carried through resume/checkpoint/
export/evaluation. Old absent-policy artifacts retain ordinary behavior. Reject
silent on/off changes, rebinding or a ramp-stage transition.

Witness **all-parameter hard-code/value equality** and **same-input full-context
hard probability equality** before versus after entry projection. Record actual
coordinate counts, projection statistics and unchanged optimizer clocks/state.
An equality failure is an implementation failure and stops fitting. It cannot be
reclassified as a quality result or repaired by changing the tolerance.

Preserve Var identities and trained values through detached projection. Reject
malformed/nonfinite inputs before mutation where possible; a backend copy failure
requires checkpoint reload. Use focused tests for row/vector bounds, interior
preservation, hard-forward identity, idempotence, endpoint derivatives and
resume/lineage continuity. Compile and run the actual changed Rust path.

Use exclusive claimed attempt roots; seal and verify complete reports, including
failures. Parents and controls remain immutable. A recovery checkpoint may be
written at8,092; final8,348 is the only quality decision point. Every interruption,
resume or rejected attempt is preserved and charged.

## Five unchanged acceptance gates

Apply the original [rung2 gates](quantized-recurrent-plan-2026-09-25.md#frozen-acceptance-before-fitting)
to **each final projected arm**, using the same evaluator-v2 population and
prompts/sampler. Both arms must pass all five for paired recipe acceptance.

1. Packed comparison-tail NLL is at most **+0.05 nats/target** above its matched
   continuous step8,348 control and below cache **2.391786178860742**.
2. Whole-prefix combined NoRead increases packed NLL by at least **0.02 nats**.
3. All five actual packed generations remain nonconstant and free of short-cycle
   collapse under explicit principal inspection.
4. At most **two originally correct first-noun rows** are lost versus the selected
   rung1 parent. Gains never cancel losses. Report every source-edit response
   against original, continuous and unprojected-QAT artifacts, with first-noun,
   exact completion and complete-pair counts and individual losses/gains.
5. Packed reload has matching hard outputs and fixed-seed IDs/distribution hashes;
   all scored probabilities and NLLs are finite and normalized under the existing
   evaluator tolerance, with actual source/data/parameter/clock provenance.

Report the 16,384-target tune prefix separately from the **233,472-target
comparison tail**, which remains previously exposed development. Record paired
likelihood differences by four64-position quarters, the same-shadow diagnostic,
and descriptive packed-versus-shadow state drift on original tune64. State and
comparison-tail likelihood summaries have different populations. Entry parity
and projection correctness are implementation requirements, not replacement
quality gates. No fresh final holdout is opened during design.

A pass supports only this projected continuation at these starts, scales and
dose. A failure rejects this bounded treatment. Neither establishes geometric
advantage, reliable conversation/code, integer execution, sparse parameter
access, Hamiltonian dynamics or energy savings. Values are still executed by the
existing F32 emulator. Broader context/capacity work remains downstream of the
retained-quality dependency, subject to the actual result and principal review.

## Resources, concurrency and delivery

The [prospective budget](../evidence/projected-recurrent-budget-2026-09-25.json)
projects the complete3-hour cycle and a standing-authorized local1-hour cumulative
extension before execution. Model fits use at most two concurrent processes,
two sequence-gradient workers each, nested BLAS/Rayon threads one, one Cargo
process, aggregate8GiB/per-process6GiB RSS stops, and the existing physical reserve
of20GiB plus128MiB margin plus64MiB checkpoint headroom. New worktree/compiler/
artifact allocation is budgeted before build against a fresh cache inventory.
Compiler temporary allowance is a projection unless actually measured; historical
gross allocation remains unresolved. No paid external training hardware.

RDC runs independent DeepSeek mathematics and systems reviews alongside native
implementation with explicit file ownership. An architecture review at the
consequential result gate remains advisory; source and artifact evidence decide.
Record reviewer corrections and incomplete checks honestly. Preserve unique
source/research/models/worktrees. Deliver the implemented and executed result
through a protected PR, synchronize current state and #973/#820, and charge unique
complete-cycle wall once, reporting overlapping process timings separately.
