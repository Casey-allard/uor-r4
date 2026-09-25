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
exact-memory scaffolds. The joint recurrent-memory learner now passes its frozen engineering continuation
gate. Next continue these same learned checkpoints through the explicit
soft-to-hard bridge, retaining the competitive ordinary control.
The transformerless integer/table serving goal and D0-b/D4–D6 remain unchanged.

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

## Latest result: D8 rung 1 complete at its engineering scope

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

## Next: D8 rung 2 on the retained student

Proceed with quantization-aware continuation of these same quaternion and
ordinary checkpoints. Specify the quantized forward graph, surrogate gradients,
scales/gains, rounding, saturation and hardening schedule; freeze quality-retention
and cost criteria before fitting. Compare continuous and actual hard paths and
inspect loaded generation early. Retain exact token/occurrence identity. The
[canonical ladder](project-track.md) keeps bounded admission/transport and final
integer serving as subsequent responsibilities. A1–A4 selector tuning remains
parked. Rung 2 and the optional 600-cell diagnostic are **NOT_RUN**.

Artifact container:
`/Users/casey.allard/uor-r4-investigations/joint-recurrent-20260924`.
Selected roots: `fit256-quaternion-3/checkpoint-final` and
`fit256-householder_pair-3/checkpoint-final`; all final fit/evaluation/comparison
roots are sealed. Executed source: `ad4e639fedecf9d490035f9986be736117bd3e29`.
The result receipt binds the exact executable, source, checkpoints and data.

## Delivery, resources and unresolved limits

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
