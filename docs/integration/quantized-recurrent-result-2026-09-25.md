# D8 rung 2 — quantized recurrent continuation

**Executed September 25, 2026. Decision: reject this paired quantization recipe at
its frozen retention scope.** The [prospective plan](quantized-recurrent-plan-2026-09-25.md)
and [execution freeze](../evidence/quantized-recurrent-execution-freeze-2026-09-25.json)
retain the original criteria. The [machine-readable result](../evidence/quantized-recurrent-result-2026-09-25.json)
contains all old/continued/packed source-edit responses, actual generation,
parameter statistics, source/data/artifact identities and descriptive diagnostics.
#973 remains open under programme #820. Delivery is [PR #1390](https://github.com/UOR-Foundation/uor-r4/pull/1390).

## Decision and project direction

Both final packed artifacts exceed the permitted +0.05 nats/target loss relative
to their equally exposed continuous controls. The ordinary Householder-pair arm
also fails the generation criterion with a period-two cycle. Neither arm passes
all five gates. No threshold, prompt, sampler, scale or final step was changed
after observing these outcomes.

Retain the joint recurrent attention/state/output model and its continuous
controls. Rung 1 remains accepted at its limited engineering scope; rung 2
retention remains unresolved. The next recommended comparison is the projected
shadow update policy specified below. It is **proposed, not implemented or run**
by this result. Integer serving and bounded admission remain downstream.

## Executed comparison

Both step-7,324 parents were continued to the fixed common step **8,348**.
Each transport has one QAT and one continuous continuation, with the same
starting weights/moments, B16/T256, counter-sampled data, two whole-window CPU
gradient shards, learning rate and 1,024 additional updates. Each arm consumes
**4,194,304 new target visits**; all four consume **16,777,216**. The eight
retained QAT profile updates per arm are included. Each lineage now contains
34,193,408 visits, including the historical 8,617,984-visit context-64 warmup;
all new continuation and evaluation use context 256. Midpoint checkpoints are
recovery material, never alternative quality-selected results.

QAT uses 256 ramp updates followed by 768 fully quantized updates. Development,
export and packed evaluation always use full quantization. The hard loader
reads actual packed codes and integer exponents without a floating shadow file.
The same learned shadows are also evaluated with all quantizers disabled.
Zero-update calibrated parents measure the starting packed behavior; their
comparison with final QAT includes the additional training exposure.

### Natural likelihood and all frozen gates

Lower NLL is better. These are the same **233,472 previously exposed development
targets**, separate from the 16,384-target tune prefix. One paired seed and no
fresh final holdout are involved. Cache NLL is 2.391786179; the historical offline
transformer reference is 1.574023596.

| Comparison-tail NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Matched continuous continuation | 2.090518499 | 2.064403221 |
| Final QAT packed artifact | 2.149631373 | 2.144230654 |
| Same QAT weights with quantizers disabled | 2.146147636 | 2.122299133 |
| Packed parent, zero QAT updates | 2.222929353 | 2.211900518 |
| Packed whole-prefix NoRead | 2.634758168 | 2.605525329 |
| Continuous whole-prefix NoRead | 2.578234214 | 2.543271225 |

| Frozen criterion | Quaternion | Householder pair |
|---|---|---|
| 1: packed minus continuous ≤0.05, and below cache | **FAIL: +0.059112874**; below cache | **FAIL: +0.079827433**; below cache |
| 2: combined NoRead penalty ≥0.02 | PASS: +0.485126795 | PASS: +0.461294675 |
| 3: five actual outputs avoid constant/short-cycle collapse | PASS at this limited scope | **FAIL: one period-two cycle** |
| 4: at most two old-correct first nouns lost | PASS: 2 lost | PASS: 0 lost |
| 5: packed reload, finite normalized probabilities, bound provenance | PASS | PASS |

The tune-prefix packed NLLs recomputed from target records are
2.243126696/2.232180273; matched continuous is 2.178568780/2.155612650.
Quick F32 tune reductions in fit reports differ in their final decimal places.
No tune value reselects the frozen final step.

## Actual generated behavior and source retention

Ordinary generation index 0, seed 2014, ends in a period-two short cycle,
repeating IDs 386 and 2218:

> No, daddy. We don't want his daddy's daddy or's daddy's daddy's daddy

Its recorded stop is `short_cycle`, period 2. The corresponding continuous,
same-shadow and zero-update packed-parent generations do not show that terminal
cycle. This single seeded contrast does not isolate a unique failure mechanism.
The other four packed outputs do not cancel the failed criterion.

The five quaternion packed outputs are nonconstant and avoid the observed
terminal short cycles; they still contain malformed words, confused roles,
repetition and a scene in which Lily is about to eat a train. The continuous
controls also remain semantically unreliable. None qualifies useful conversation.

| Source-edit measure | Quaternion old → continuous → packed | Ordinary old → continuous → packed |
|---|---:|---:|
| First noun correct /32 | 28 → 26 → 27 | 31 → 32 → 32 |
| Exact completion /32 | 28 → 23 → 19 | 23 → 26 → 25 |
| Both variants exactly complete /16 | 14 → 9 → 8 | 10 → 12 → 12 |
| Old-correct first nouns lost by packed | 2 | 0 |

Quaternion's first-noun losses are `story-source-edit-06/edited`, `cap.` to
`captain.`, and `story-source-edit-08/original`, `brush.` to `door.`. Its `doll.`
gain does not offset these under the frozen loss-count criterion. Ten old exact
completions are lost and one gained. Ordinary loses no old first noun or exact
completion and gains two exact completions, but still over-continues seven
answers. The evidence receipt preserves every row, including actual continued
controls. Both quaternion first-noun losses also occur in continuous. Against that
continued control, packed loses zero first nouns but loses eight exact completions
and gains four. Ordinary loses one exact completion against continuous:
`story-source-edit-11/edited`, `coin.` to `coin home.`. The continuous quaternion
also loses exact completions, so the whole old-to-packed decline cannot be
assigned to quantization alone.

The probes are 56–62 tokens long and do not isolate retrieval beyond 64
positions. Combined NoRead removes both value feedback and pointer copying.
Its effect does not identify the separate contribution of either branch or of
geometric transport.

## Diagnostics and their interpretation

QAT reduces packed-parent loss by **0.073297979/0.067669864 nats**, but fails the
matched continuous-retention requirement. Disabling quantizers on the final QAT
shadows leaves deficits of **0.055629137/0.057895912** relative to continuous
training; re-enabling them adds **0.003483738/0.021931521**. These are descriptive
contrasts between different trajectories and forward modes. Their additive
arithmetic does not bound the possible improvement of a new scale policy, prove
clipping is the cause, or isolate optimizer error from the learned solution.

| Positions (zero-based) | Q packed − continuous NLL | Ordinary packed − continuous NLL | Q packed − shadow NLL | Ordinary packed − shadow NLL |
|---|---:|---:|---:|---:|
| 0–63 | 0.059467153 | 0.071453967 | 0.003849168 | 0.018197128 |
| 64–127 | 0.056644727 | 0.084668475 | -0.000309642 | 0.025635619 |
| 128–191 | 0.058049879 | 0.082353781 | 0.002280305 | 0.022047143 |
| 192–255 | 0.062289739 | 0.080833509 | 0.008115121 | 0.021846194 |

Each likelihood quarter contains 58,368 comparison targets. A separate Rust
state diagnostic compares loaded packed computation with the same QAT shadows
on the **original 64 tune windows**, checking all 4,194,304 paired state
coordinates per arm. It first verifies that every loaded parameter is bitwise
equal to the frozen quantization of the selected shadow. Every observed state
coordinate is finite.

| State diagnostic | Quaternion | Householder pair |
|---|---:|---:|
| All-position RMS coordinate difference | 0.059484643 | 0.055451185 |
| Quarter 0–63 RMS difference | 0.056821291 | 0.053221358 |
| Quarter 64–127 RMS difference | 0.060207509 | 0.056145299 |
| Quarter 128–191 RMS difference | 0.060334998 | 0.055994879 |
| Quarter 192–255 RMS difference | 0.060496081 | 0.056383522 |
| Maximum absolute coordinate difference | 0.597404748 | 0.843987465 |
| Packed RMS state L2 norm | 4.468165356 | 4.256628813 |
| Shadow RMS state L2 norm | 4.408040972 | 4.152645304 |

The quarter averages show no large progressive rise on this population. They
are combined parameter/interface/recurrence/read/write differences, not isolated
transport error or a stability proof. All 256 position records are retained in
the hashed diagnostic reports. These state and likelihood summaries use
different populations and cannot be treated as the same per-example measurement.

Final parameter clipping is 49,215/52,102 coordinates, compared with
30,224/32,091 in the calibrated parents. Specifically, 49,213/52,100 of
1,672,704 four-bit coordinates and two of 5,762 sixteen-bit offsets per arm
lie outside the fixed representable ranges. Aggregate clipping does not locate
which coordinates caused the quality deficit. The full-hard STE masks current
gradients outside its range; inherited momentum and weight decay can still move
such shadows. They are not necessarily permanently frozen.

## Arithmetic and artifact boundary

All multiplicative parameters, including tied embedding and normalization gain,
use signed four-bit codes. Bias/age offsets use signed sixteen-bit codes. Fixed
row scales and recurrent grids are documented in the plan. Quaternion signs
remain distinct; unit coordinates are not renormalized off-grid. The ordinary
transport is a pair of reflections with positive determinant before rounding.

Both final packed exports have **zero measured full-context probability reload
difference**, with all 48 smoke-generation IDs and probability hashes matching.
Full-population evaluators check finite, normalized distributions under the
0.0002 tolerance. The parameter payload is 847,876 bytes per arm; the complete
required three-file model totals **1,168,836/1,168,844 bytes**, excluding the
separately bound tokenizer, executable and audit reports. Loading expands codes
into F32 tensors. Payload size alone is not measured process RAM or throughput.
The 537,877 interface observations per arm have zero clipping/nonfinite values,
strictly within the recorded 48-token smoke scope, not the whole corpus.

This remains an **F32 emulator**: dense matrix operations, dynamic products,
normalization, nonlinearities, probability mixture and sampling use floating
point. Integer serving, D5 sparse parameter access, exact H4/Z[phi], Hamiltonian
dynamics and complete-path energy savings remain unqualified.

## Recommended next comparison: projected shadow updates

Keep this recurrent attention model and compare **projected shadow updates** in
one paired continuation from the retained full-hard midpoint **7,836 → 8,348**.
Preserve the original negative and the existing unprojected second halves as
direct controls; the completed continuous branches remain the retention reference.
Use the same fixed scales, offsets, interfaces, AdamW moments/counters, B16/T256,
learning rate, data windows and final-only decision. No corpus expansion, learned
scale, auxiliary loss or geometry restriction is combined with this intervention.

For each fixed interval, define `P(w)=clip(w,q_min*s,q_max*s)`. The implemented
quantizer has **`Q(P(w))=Q(w)`**. At this midpoint the ramp is already complete,
so projecting the stored shadows preserves the effective hard model at entry.
Require unchanged packed codes and same-input hard probabilities before fitting.
A restart at step 7,324 would also change the partly floating ramp computation.
The midpoint is selected for its preserved full-hard recovery role, independently
of measured quality.

Project stored parameters as a detached optimizer operation at entry and after
each update. Retain moments and clocks and bind the policy explicitly in resume
metadata. The existing STE includes its endpoints, so formerly out-of-range
values regain a current surrogate derivative. Outward momentum can still pin
them at a boundary, and fixed ranges or interface quantization can remain limiting.
Projection also changes weight-decay inputs and potentially global gradient
clipping: its result measures the complete policy, not only gradient unmasking.

The dose is **512 updates / 2,097,152 target visits per arm**, 4,194,304 new visits
total. Freeze all five original gates and the common final step before execution;
project and charge complete local resources before using them. Success supports
only this policy at these retained starts, scales and exposure. Failure rejects
this bounded continuation. Neither verdict resolves geometric advantage, useful
general language, integer execution, sparse access or energy savings. The
successor remains **NOT_RUN** in this result.

[BNN, Algorithm 1](https://arxiv.org/html/1602.02830v3) supplies precedent for
clipping updated real weights. [LSQ](https://arxiv.org/html/1902.08153v3) provides
a separate task-loss-based scale-learning alternative. Their image classifiers
do not establish performance here. LSQ's strict-interior weight derivative
differs from this implementation's inclusive endpoints; that distinction is
essential to the proposed recovery argument. Learned dyadic scales remain a
separate possible mechanism requiring their own forward/surrogate and comparison.

## Verification, review and resources

Sixteen focused release checks passed for quantization/codec, causal shared core,
gradients, continuation and comparison. Formatting at `cb81e7c7` and the Rust
state example's release build passed. All final fit/export/evaluation/comparison
and state-diagnostic roots were sealed and verified. The recorded binary for
fits and population evaluation is bound to source `b5b5fe75`; the diagnostic
uses `cb81e7c7`, with the model library unchanged. Actual local execution carries
validation; GitHub compatibility acknowledgements execute no tests.

RDC launched independent DeepSeek mathematical/systems and Kimi architecture
reviews concurrently with local work. Two final Rust state diagnostics also ran
concurrently through RDC. RDC operated on the same Mac; it supplied process and
research coordination, not another physical training host. Principal review
corrected erroneous sign/reflection/STE/size claims and declined unsupported
causal conclusions and proposed replacement thresholds. Advisory agreement
does not establish a model result. The evidence receipt retains the findings,
principal dispositions and the narrowed successor proposal.

The [resource closeout](../evidence/quantized-recurrent-closeout-2026-09-25.json)
records unique complete-cycle wall time, separately summed model-process times,
current retained allocation, sampled RSS/physical free space and delivery-tail
accounting. All parents, negatives, continuous controls, packed exports, reports,
executed binaries and worktrees are preserved. Two validated cleanups removed
43 old compiler intermediates, with a net physical-free increase of
1,732,194,304 bytes across the recorded windows. No cleanup credit offsets the
campaign allocation. Historical gross compiler allocation remains UNRESOLVED;
current inventories do not retroactively certify it. No external model-training
hardware was used.
