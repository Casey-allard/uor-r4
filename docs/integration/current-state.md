# Current UOR-R4 research state

Updated September 25, 2026. **Pre-alpha; no useful general-language, coding,
frontier, geometric-advantage or full-path energy qualification.**

## Decision and active work

The owner supplied a whole-project stuck-point review and directed warranted
corrections before the pending merge. The [independent assessment](stuck-point-review-response-2026-09-24.md)
agrees with its central learning-method criticism, corrects outdated and
mathematical claims, and establishes [D8](DECISIONS.md#d8--correct-the-training-method-and-reference-ladder).
The [canonical plan](project-track.md) owns the persistent training/reference
ladder. #973 remains active under programme #820; their full acceptance is open.

**Park further A1–A4 local selector tuning.** Preserve its native serving and
exact-memory scaffolds. The joint recurrent-memory learner passes its frozen
continuation gate. The learned neighboring-code successor now passes all five
original hard-artifact retention gates in both arms; the two older quantized
negatives remain preserved. The shared bounded-admission successor now completes
its paired comparison but fails source retention in both arms. Keep the accepted
learned-code parents and full 256-token access. The recent64 training follow-up
is withdrawn under the owner-directed [D9 correction](DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract).
Advance the quantized transport/integer execution bridge; admission pruning is
a deferred optimization. No additional rounding, hash or scale sweep.
The transformerless integer/table serving goal and D0-b/D4–D6 remain unchanged.

## Latest result: bounded admission complete; source retention fails

The [paired result](bounded-admission-result-2026-09-25.md) completes one shared
bounded-admission implementation and256 updates /1,048,576 fitted target visits
per arm at B16/T256. The original stricter physical-reserve guard checkpointed
at166; both arms resumed the same optimizer/data state for90 more updates.
Actual data hashes match in both segments. No learning or artifact was discarded.

| Comparison-tail NLL / complete source answers | Quaternion | Householder pair |
|---|---:|---:|
| Accepted learned-code parent | 2.114226 /28 of32 | 2.110880 /24 of32 |
| **Final orthant64** | **2.129595 /17 of32** | **2.124890 /17 of32** |
| Same final weights, full | 2.080041 /27 of32 | 2.069663 /25 of32 |
| Same final weights, recent64 | 2.119169 /27 of32 | 2.108628 /25 of32 |
| Same final weights, exact_cache64 | 2.146802 /23 of32 | 2.136447 /18 of32 |

Both frozen retention vectors are **PASS/PASS/PASS/FAIL/PASS**. First-noun losses
against the immediate parents are11/8 and against rung1 are11/9, above maximum2.
NLL, combined NoRead effect, limited noncollapse and mechanical/export checks
pass. Indexed complement utility also passes versus recent32 (17 additional
complete answers; late-position NLL gains0.110267/0.105152), but cannot override
retention failure. Same-weights full/recent64 recovery identifies the access
policy as a material cause. Neither a diagnostic override nor a lower average
loss promotes a failed candidate. The accepted learned-code parents remain current.

**Next:** implement quantized R4 transport and the integer execution bridge
from the accepted learned-code parents, with full 256-token access and the
matched ordinary arm. Reuse the existing evaluator and retained evidence.
The proposed recent64 fit is withdrawn before execution; its same-weights
results remain diagnostics. Sparse admission is parked until new causal evidence
and an implementation need justify reopening it. Neither bounded-candidate
retention nor another selection sweep blocks full-access numerical implementation.
See the active contract below for the required deliverable and validation scope.

Executed source: `78fde5711ea01f80783eb9911e01197a61406d1a`.
Rejected but retained artifacts: `fit-{quaternion,householder_pair}-bounded-2/packed-model`
under `/Users/casey.allard/uor-r4-investigations/bounded-admission-20260925`.
[Arithmetic and sealed bindings](../evidence/bounded-admission-result-2026-09-25.json),
[principal decision](../evidence/bounded-admission-decision-2026-09-25.json),
[all outputs](bounded-admission-outputs-2026-09-25.md),
[plan/review](bounded-admission-plan-2026-09-25.md),
[resources](../evidence/bounded-admission-closeout-2026-09-25.json).
Three DeepSeek sessions and paired Rust jobs ran through RDC on this M1; no Kimi.
Training supervision totals24.67minutes;62 optimized focused checks pass in4.79seconds.
F32 dense parameter computation, allocations, unreliable prose and the256-token
ceiling remain. No geometric advantage, Hamiltonian dynamics, integer-serving,
terminal sparsity or energy qualification is claimed.

## Active execution contract

Owner-directed correction, September 25; governed by
[D9](DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract)
and the [progress-control rules](agent-execution-policy.md#progress-control--owner-correction-september-25).

| Quantity | Required next baseline |
|---|---|
| Training / evaluation / session context | 256 / 256 / 256 tokens |
| Direct context access | Every causally available event within that window; `full` admission |
| Candidate cap | Up to the complete causal window, not an imposed 64-event subset |
| Read representation width | 64 coordinates; a vector dimension, not a token horizon |
| State width | 256 coordinates; separate from context and admission |
| Accepted starting artifacts | `learned-rounding-20260925/fit-{quaternion,householder_pair}-rounding-3/packed-model` |
| Evaluator | Existing `reference-evaluator-v2.json`; no new panel or holdout during implementation |
| Deferred branch | recent64 fitting and sign-index/width/hash sweeps |

**Implementation work card (#973 under #820):**

- **Deliverable:** load the accepted learned codes into a Rust integer execution
  bridge, implementing quantized R4 transport as part of that same path. Retain
  full context access and the ordinary transport comparator. Identify any
  remaining float/multiplier operation explicitly; partial lowering is not full
  serving qualification.
- **Observed blocker / change:** packed codes currently execute in an F32
  emulator. Replace the numerical execution of the existing learned operators;
  keep model identity, context and admission fixed. Candidate selection need
  not change to implement integer arithmetic.
- **Necessary verification:** compile the changed Rust path; check changed
  arithmetic/overflow, artifact loading and causal interfaces; measure state and
  output drift against the existing emulator; inspect actual loaded full-context
  generation with the retained evaluator. Reuse unaffected checks. Freeze the
  numerical tolerance for each changed operator before its comparison. Full
  model acceptance still needs its existing behavior criteria.
- **Decision:** retained behavior and declared numerical compliance permit
  integrating that implemented path. A mismatch localizes the next numerical
  repair. Retraining is justified only by measured approximation error that the
  implementation cannot resolve; it is not the default next experiment. If the
  instrument cannot distinguish the cause, repair that named instrument only.
- **Cost:** no model run is launched by this work card. Before numerical work,
  use source inspection and retained throughput to project implementation/build,
  focused checks, loaded evaluation and any specifically justified learning
  against the existing cumulative budget. No new review or benchmark programme.

The 256-token full-access baseline is finite and bounded; it does not satisfy
terminal D5 parameter sparsity or establish economical scaling to larger memory.
Do not silently shorten access to meet a cost target. The integer bridge is
**NOT_RUN**; this correction changes the next implementation, not model quality.

## Retained result: paired learned-code retention accepted

The [completed fixed-recipe learning](learned-rounding-result-2026-09-25.md)
adds512 alpha-only updates /2,097,152 fitted target visits per arm, B16/T256,
plus32,768 training-only normalization visits. Both projected parents and their
scales, bit widths, interfaces and architecture remain fixed. Resumes preserve
all learning; actual sample hashes agree across both arms in every segment.

| Comparison-tail NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Retained continuous reference | 2.090518499 | 2.064403221 |
| Prior projected packed | 2.150319798 | 2.147928712 |
| **New learned hard codes** | **2.114226169** | **2.110879800** |
| Hard minus continuous; allowance0.05 | **+0.023707670** | **+0.046476579** |
| Whole-prefix NoRead | 2.609608832 | 2.607989949 |

Both five-gate vectors are **PASS/PASS/PASS/PASS/PASS** at the original limited
scope. Combined NoRead penalties are+0.495383/+0.497110. All five real
read-enabled generations per arm avoid constant/short-cycle collapse. First-noun
losses against rung1 are2/1, within the frozen maximum2; full-context packed
reload probability delta is zero and seeded outputs agree. Principal arithmetic
verifies32 sealed roots,44 binding records, complete target identities and all
source-row changes. No fresh final holdout or checkpoint selection occurs.

Exact source completions are28/32 and24/32. **Both still lose3 previously correct
complete answers against rung1**, with gains reported separately. All ten free
generations remain semantically unreliable. The continuous references are the
original step8,348 anchors; new code candidates receive extra learning. This
one-paired-seed result does not establish equal-compute superiority, geometric
advantage, Hamiltonian dynamics, useful conversation/code, integer execution,
terminal D5 parameter sparsity or physical energy savings. Serving of these
packed artifacts is still F32 numerical emulation.

**Historical successor, now executed above:** implement the actual bounded serving admission rule in the common
training/session graph. Preserve exact occurrence identity, causality and NoRead;
keep full-context and matched ordinary/exact-cache controls. Count the complete
admission cost, then integrate observable quantized group transport and measure
accumulated state/output error. Integer execution and useful complete outputs
follow retained behavior under those constraints. No new diagnostic sweep.

Executed Rust source: `036eabcc86e453f581aa0563e2967cc1f7db96d5`.
Final artifacts: `fit-{quaternion,householder_pair}-rounding-3/packed-model` under
`/Users/casey.allard/uor-r4-investigations/learned-rounding-20260925`.
[Full result/bindings](../evidence/learned-rounding-result-2026-09-25.json),
[all responses](learned-rounding-outputs-2026-09-25.md),
[resolved recipes](../evidence/learned-rounding-resolved-recipes-2026-09-25.json),
[plan/research/review](learned-rounding-plan-2026-09-25.md),
[resource closeout](../evidence/learned-rounding-closeout-2026-09-25.json).

RDC ran two concurrent DeepSeek expert sessions and both local Rust fits.
No Kimi model or external training hardware was used. All42 focused optimized
Rust checks pass in3.59seconds; paired fit supervision totals48.16minutes.
A physical-reserve stop and a configured process-time boundary checkpointed and
resumed without repeated updates. No artifact/worktree was deleted. The ledger
charges the whole preparation/build/fit/evaluation/review/delivery cycle.

## Retained result: parameter/interface precision diagnosis complete

The [fixed-parent comparison](precision-factorial-result-2026-09-25.md) evaluates
all four precision combinations in both retained step-8,348 parents. Both new
FF/QQ endpoint pairs reproduce every retained target record and response field
apart from elapsed time. No training, scale search or new holdout occurs.

| Comparison-tail NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Floating parameters / floating interfaces (FF) | 2.113220567 | 2.098025647 |
| Quantized parameters / floating interfaces (QF) | 2.150291052 | 2.147912004 |
| Floating parameters / quantized interfaces (FQ) | 2.113217828 | 2.098039459 |
| Quantized parameters / quantized interfaces (QQ) | 2.150319798 | 2.147928712 |

At floating interfaces, parameter quantization costs **+0.037070/+0.049886**
nats/target. Interface quantization costs **−0.000002739/+0.000013812** at
floating parameters; interaction is **+0.000031484/+0.000002896**. Parameter
cost dominates in both arms, the tune prefix and every comparison quarter.
Even QF misses the original +0.05 allowance against continuous. This diagnoses
these fixed forward paths; it does not isolate individual tensors, predict
retraining recovery or establish geometric advantage.

Small mean interface loss is not behavioral identity: FF→FQ changes nine of ten
free generations, and QF→QQ changes eight of ten. All 40 free generations and
256 source responses are retained in the [complete output record](precision-factorial-outputs-2026-09-25.md).
Exact source completions, in FF/QF/FQ/QQ order, are **25/22/25/23** of 32 for
quaternion and **28/22/28/22** for ordinary; row gains and losses stay separate.
Prose remains semantically unreliable. Both prior quantized recipes remain
rejected and all computation here remains F32 emulation.

**Historical successor, now executed above:** [one paired learning of neighboring integer-code choices](precision-factorial-result-2026-09-25.md#recommended-next-learn-rounding-decisions-within-the-existing-parameter-grids)
with fixed parents/scales/bit widths/interfaces, the full 256-step recurrent
language objective and training-only calibration. Freeze one recipe and complete
resource projection before fitting; decide once on the final hard export/reload
using the original five gates. No scale/per-tensor sweep or automatic fallback
is adopted. Success would permit trained bounded admission/transport, followed
by integer execution and useful complete outputs; it would not itself qualify
them.

Executed Rust source: `49dfd0b15486ba3a7a763c598257f651e857279f`.
Artifact container: `/Users/casey.allard/uor-r4-investigations/precision-factorial-20260925`.
[Full arithmetic and bindings](../evidence/precision-factorial-result-2026-09-25.json),
[endpoint receipt](../evidence/precision-factorial-endpoints-2026-09-25.json),
[resource closeout](../evidence/precision-factorial-closeout-2026-09-25.json),
[independent review and primary research](precision-factorial-review-2026-09-25.md).
Eight model evaluations took about three minutes of concurrent pair wall time;
25 focused tests took 2.16 seconds, with about seven minutes of compilation.
Whole-cycle accounting includes preparation, analysis, review and delivery.
The owner's September 25 cost direction makes DeepSeek the default for all
specialists, including architecture; Kimi requires a new explicit owner request.

## Retained result: D8 projected continuation completed and rejected

[PR #1391](https://github.com/UOR-Foundation/uor-r4/pull/1391) delivers the
[completed paired result](projected-recurrent-result-2026-09-25.md) under the
unchanged [prospective plan](projected-recurrent-plan-2026-09-25.md).
Each arm continues its preserved fully quantized midpoint **7,836 → 8,348**,
adding **512 updates / 2,097,152 target visits** at matched B16/T256. Projection
is applied at entry and after every AdamW update, preserving moments, clocks,
fixed scales and all five original gates. There is no new checkpoint selection.

| Comparison-tail NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Matched continuous continuation | 2.090518499 | 2.064403221 |
| Original unprojected packed QAT | 2.149631373 | 2.144230654 |
| New projected packed model | 2.150319798 | 2.147928712 |
| Same projected shadows, quantizers disabled | 2.113220567 | 2.098025647 |
| Projected packed whole-prefix NoRead | 2.637288928 | 2.618333390 |

Both packed-minus-continuous gaps (**+0.059801/+0.083525**) exceed **+0.05**,
so both candidates are rejected for retention. The other four frozen gates pass
at their declared scope: combined NoRead penalties **+0.486969/+0.470405**,
five noncollapsed generations per arm, two/one lost rung-1-correct first nouns,
and loaded parity/finite normalization/provenance. Ordinary's former short cycle
is absent on this panel. Exact completions are **23/32 and 22/32**; source gains
never cancel losses. Prose remains semantically unreliable.

Entry projection brings **41,184/43,770** out-of-range shadows to zero while
preserving all hard codes, all actual full-context probabilities and optimizer/
sampler state. Both final models still have zero clipped parameter coordinates.
Shadow likelihood improves and packed-versus-shadow state RMS falls, yet packed
likelihood does not improve. The [full evidence](../evidence/projected-recurrent-result-2026-09-25.json)
retains every response, row comparison, quarter and artifact binding. This is
one paired seed on exposed development, with no fresh holdout opened.

The [previous unprojected result](quantized-recurrent-result-2026-09-25.md),
[PR #1390](https://github.com/UOR-Foundation/uor-r4/pull/1390), remains rejected
at its original gates, including the ordinary period-two cycle. Its final
continuous and packed artifacts are preserved without rewriting that outcome.
All model paths here remain F32 emulators: integer serving, D5 parameter
sparsity, Hamiltonian dynamics, general language and energy are unqualified.

### Successor diagnosis completed

The recommended fixed-checkpoint precision comparison is complete above. Its
[one code-choice successor](precision-factorial-result-2026-09-25.md#recommended-next-learn-rounding-decisions-within-the-existing-parameter-grids)
is the current NOT_RUN recommendation. This projected experiment's artifacts
and rejection remain unchanged.

Artifact container:
`/Users/casey.allard/uor-r4-investigations/projected-recurrent-20260925`.
Final roots: `fit-{quaternion|householder_pair}-projected-1/checkpoint-final`
and `final-packed-{quaternion|householder_pair}-1`.
All Rust model/diagnostic runs use source
`66349cdb69883dd7d438c76394c22f4e19000c73`; report roots are sealed.
[Resources and executed checks](../evidence/projected-recurrent-closeout-2026-09-25.json),
[independent RDC review](projected-recurrent-review-2026-09-25.md).
The two CPU fits complete concurrently in about36 minutes on the same M1;
22 focused optimized Rust tests passed. Queue acknowledgements execute no tests.

## Retained integer-path result: A4

Four matched eight-epoch continuations consume 4,499,104 new token updates and
185.945 seconds of joint fitting. A4 selects the correct source on 2/24 new first
decisions in each arm; both controls select 0/24. All four produce **0/12 complete
correct read-enabled answers**. Loaded A4 retains only 28/167 C120 and 24/167 2I
admitted fit sources despite each eligible local update succeeding immediately.
Development bits/token is C120 A4/control **6.839477/6.799839**, and 2I
**6.798995/6.872785**. There is no model promotion.

The dedicated address coefficients stay fixed, but shared State updates and
causal inputs change actual fine codes. This corrects the original A4 report's
freeze wording. All 112 generation rows and eight full files per arm reproduce
in independent replay. 33 focused checks passed. See the [A4 result](integrated-attention-a4-result-2026-09-24.md)
and [compact evidence](../evidence/integrated-attention-a4-result-2026-09-24.json).

## Retained reference and shared evaluator

**D8 rung 0 is complete at its declared reference/comparator scope.** The shared
offline Rust tool reproduces #1017's full 249,856-target development NLL at
**1.580241190 nats**, within `1.173e-7` of the historical value. All five actual
seeded continuations reproduce all **582 generated IDs**, decoded text and stop
reasons. Batch isolation and future-input prefix checks have zero measured error.
The shared forward path retains its CPU language-gradient/parity check; the
previous CPU/Metal integrity results remain preserved.

On the 233,472-target comparison tail, mean NLL is **1.574024** for #1017,
**2.405627** for the new normalized, count-pruned interpolated 5-gram, and
**2.391786** for that 5-gram plus causal cache. Lower is better. Count fitting uses
both inherited training stores, totaling 149,996,416 raw IDs. Selected discount
is 0.9 and cache mixture 0.05. Four report roots are sealed and independently
verified; every scored row matches the actual input/target population.
[Result and interpretation](reference-baselines-result-2026-09-24.md),
[compact evidence](../evidence/reference-baselines-result-2026-09-24.json),
[evaluator v2](reference-evaluator-v2.json).

This is previously exposed development: the comparison tail is separate from
current count calibration but was used in historical neural checkpoint selection.
Five exact replays are not five correct answers; outputs retain repetition,
semantic drift and three capped continuations. #1017 is an ordinary floating-point
transformer used offline, never target serving. No neural optimizer step or
native model promotion occurs. The reference advantage does not isolate attention
or geometric causality. #1014 retains the historical attention-off evidence at
its original scope.

## Retained result: D8 rung 1 complete at its engineering scope

The [joint recurrent-memory campaign](joint-recurrent-result-2026-09-25.md)
completed **29,999,104 target visits and 7,324 updates per arm**. This includes a
matched 8,617,984-visit warmup at context 64 and 21,381,120 visits with training
and evaluation both at context 256. The owner correctly challenged the mismatch;
weights, AdamW moments and clocks were preserved across its declared correction
and two disk-guard checkpoints. Both final save/reload loss differences are zero.

Both arms selected final step 7,324 from the prescribed midpoint/final choices
using recorded tune-prefix scores **before** population evaluation. The Rust
comparison joins all 249,856 targets and reproduces all 21 partition means.
[Selection](../evidence/joint-recurrent-selection-2026-09-25.json),
[full result and actual responses](../evidence/joint-recurrent-result-2026-09-25.json).

| Comparison-tail NLL, nats/token | Read enabled | Whole-prefix NoRead |
|---|---:|---:|
| Quaternion recurrent learner | 2.110368 | 2.592991 |
| Matched ordinary recurrent learner | 2.085241 | 2.561712 |
| Count/cache baseline | 2.391786 | — |
| Historical offline transformer reference | 1.574024 | — |

Both learners improve their retained loss, beat count/cache, and generate varied
loaded text without constant or short-cycle collapse. They pass the **frozen
rung 1 engineering gate**. Text still shows semantic drift, role confusion,
malformed words and repetition; sustained coherent prose and useful general
conversation are not qualified. The read intervention removes both recurrent
value feedback and pointer-copy probability; the 0.482623/0.476471 loss penalties
establish their combined contribution, not isolated geometric or distant access.

On 16 frozen source-edit pairs, quaternion produces 28/32 exact completions and
ordinary 23/32; first-noun correctness is 28/32 versus 31/32. Both NoRead arms
produce 0/32. The strict-answer difference includes ordinary over-continuation;
it is not a geometric retrieval win. Prompts are only 56–62 tokens. Neither the
probe nor later-position likelihood isolates retrieval beyond 64 positions.
This is one paired seed on exposed development, with no fresh final holdout.

The learner executes quaternion transport in its prediction graph, with shared
language credit through state, contextual Q/K/V reads/writes and normalized
vocabulary/copy output. It remains **floating-point offline Rust**, with dense
affine maps and full soft context access. Integer export, bounded prime/zeta
admission, exact H4 serving, Hamiltonian dynamics and energy are unqualified.

## Retained rung 1 identities

These parents and their original acceptance remain preserved. The latest rung 2
result and next comparison above supersede their earlier scheduling statement.
The optional 600-cell diagnostic remains NOT_RUN.

Artifact container:
`/Users/casey.allard/uor-r4-investigations/joint-recurrent-20260924`.
Selected roots: `fit256-quaternion-3/checkpoint-final` and
`fit256-householder_pair-3/checkpoint-final`; all final fit/evaluation/comparison
roots are sealed. Executed source: `ad4e639fedecf9d490035f9986be736117bd3e29`.
The result receipt binds the exact executable, source, checkpoints and data.

## Retained rung 1 delivery/resources and programme limits

The preceding correction was protected [PR #1387](https://github.com/UOR-Foundation/uor-r4/pull/1387),
merged as `942645b264f73ec49507cffd7c2f4cbd95de3aa2`. Rung0 was delivered through
protected [PR #1388](https://github.com/UOR-Foundation/uor-r4/pull/1388), merged as
`c84b198df3b47bc8326307dd9964dc9c87cb830c`. The completed learner campaign is delivered through
[protected PR #1389](https://github.com/UOR-Foundation/uor-r4/pull/1389). Queue compatibility acknowledgements
execute no tests; local executed checks carry validation.

The [campaign closeout](../evidence/joint-recurrent-closeout-2026-09-25.json)
charges measured complete-cycle wall time once; overlapping training-process
seconds are reported separately. A measured delivery tail is charged after the
committed cutoff. Eight focused release checks and all actual population/generation
runs passed their stated execution checks; queue labels are not those results.

RDC used the same local 8-core, 16-GiB M1. Measured CPU/GPU profiles selected two
sequence-gradient workers per arm, both arms concurrent, with nested backend
threads limited to one. Sustained final fitting achieved about 1,362/1,365 target
visits per second; Metal and four workers per arm were slower. Final-phase peak
sampled combined RSS was 6.01 GiB. No energy or serving-throughput claim follows.

The excessive 26.1-GB disk guard was corrected prospectively to **20 GiB reserve
plus 128 MiB stop margin and 64 MiB checkpoint headroom**. The final phase's
minimum measured free space was 25,032,024,064 bytes, above that stop. Inspected
inactive compiler intermediates reclaimed 3,652,976,640 physical bytes in the
main cleanup; all models, source, binaries, reports and worktrees remain. Exact
gross new compiler allocation is **UNRESOLVED**; retained size and free-space
measurements do not certify the gross-storage ceiling. No paid compute was used.

The old #1017 revealed test remains a regression set. Persistent sessions still
need exact posting membership preserved through saturated-page eviction/restore.
Final integer export, geometry attribution, useful complete outputs, terminal D5
parameter access and physical energy remain gates.

## History and authority

The complete previous 4,324-line state record is preserved in the
[dated archive](current-state-archive-through-2026-09-24.md), with all original
relative evidence links. Read scoped history as needed. Start routine work from
this page, the [plan](project-track.md), [decisions](DECISIONS.md), and the exact
source/artifacts for the active rung; do not restart a whole-project survey.
