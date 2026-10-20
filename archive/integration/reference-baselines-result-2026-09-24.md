# D8 rung 0 — matched reference, count and cache result

**Decision: rung 0 is complete within its declared reference/comparator scope.**
Proceed to the canonical recurrent-memory learning campaign. This milestone
trains no neural model and promotes no native candidate. #973 and #820 remain open.

## What executed

The shared offline Rust tool loads the actual #1017 checkpoint, reproduces its
population likelihood and generates the five retained seeded continuations.
A new normalized, count-pruned, fixed-discount interpolated Kneser–Ney 5-gram
and its causal token-cache mixture are fitted and evaluated on the same tokenizer
and source population. The [v2 evaluator](reference-evaluator-v2.json) was committed
and independently reviewed before model execution. The prior v1 manifest remains
preserved. No numerical gate was changed after seeing the results.

The [compact evidence](../evidence/reference-baselines-result-2026-09-24.json)
binds source `60be78d9273ffc0b4ae5dc5e62c398df51ac3a1f`, executable
`edc57672f39d30267d6bff97fb68c94d28d02fcf312a1b0cd0e187cb4ed275e6`,
the evaluator, retained model and complete sealed reports. Full artifacts are at
`/Users/casey.allard/uor-r4-investigations/reference-baselines-20260924`.

## Common population and results

The count fit consumes both inherited training stores: 30,000,000 and
119,996,416 raw token IDs, keeping their boundary separate. These are the sources
used by #1014 and its #1017 continuation. Historical neural training recorded
149,995,520 sampled target visits through overlapping windows. Counting each
source occurrence once matches available source population, not optimization
exposure or parameter count.

Evaluation uses independent 256-input blocks, next-token targets, no inserted BOS,
and no special reset at EOS inside a block. Of 250,000 stored IDs, 976 full blocks
score 249,856 targets; the incomplete tail leaves 143 targets unscored. The first
64 blocks calibrate count discount and then cache mixture. Settings are saved
before evaluating the remaining 912 blocks. The entire development population
previously selected the neural checkpoint; the tail is separate from current
count calibration, **not a fresh final holdout**.

| Arm | Calibration: 16,384 targets | Comparison: 233,472 targets | Full: 249,856 targets |
|---|---:|---:|---:|
| #1017 reference | 1.668842 | **1.574024** | 1.580241 |
| Count-pruned interpolated KN 5-gram | 2.500302 | 2.405627 | 2.411835 |
| Same 5-gram plus causal cache | 2.487131 | 2.391786 | 2.398038 |

All entries are mean negative log likelihood in **nats/token; lower is better**.
The selected fixed discount is **0.9** and cache mixture is **0.05**, using a
256-token cache reset per block. The cache reduces NLL by **0.013841 nats/token** relative to the
pure 5-gram on the comparison tail. The reference is **0.817763 nats/token** better
than that cache mixture. An independent row audit matches every input, target,
block and offset to the retained token store and recomputes every reported mean.

The reference full-population NLL is `1.5802411896906157`, versus historical
`1.580241072373312`: absolute error `1.173173e-7`, inside the predeclared `1e-4`
gate. Batch-two versus independent single-block logits match exactly on 2,097,152
values. Editing input positions 128–255 leaves all 524,288 earlier logits
unchanged. The actual evaluation uses detached parameters and batches of 16.

### Comparator implementation and cost

Lower-order counts use distinct left extensions from **unpruned raw types**.
Pruning retains original context totals; both removed and discounted mass back
off. The unigram floor is normalized. This is fixed-discount interpolated KN,
with explicit count pruning and an additive unigram floor; it is not the
multi-discount modified algorithm. See [the implementation](../../crates/uor-r4-training/src/ngram.rs)
and its primary smoothing references. The causal cache observes the current
input before predicting the next token and cannot ingest that target early.

The fitted count artifact is **369,454,767 bytes**, compared with **28,627,504
bytes** for the reference safetensors. These are different representations and
sizes, not a parameter-matched comparison. Count fitting took **13.973 seconds**
inside the driver and **17.61 seconds** for fit/export/reload/sealing as a process;
peak process RSS was **2,738,044,928 bytes**. All 64 actual-input probability probes
survive export/reload bit-exactly. Calibration plus full count evaluation took
**2.66 process seconds**.

The calibrated count baseline comprises `count-fit/model.ng5` **and**
`count-evaluation/count-selection.json`: evaluation overrides the artifact's
serialized default discount with the selected value. Cache tuning holds that
discount fixed. No claim is made about optimal count models or a joint search
over all discount/mixture pairs. Per-order pruning counts and mass remain in the
sealed fit report.

## Actual generation

All five freshly generated continuations reproduce the retained historical
**generated token IDs, decoded responses and stop reasons**. Independent review
also checked raw text, tokenizer, prompts, seeds, sampler and each of the **582**
per-decision records. Actual outputs were persisted before the generator opened
historical golden outputs. Historical R4 gauge audit metadata is outside this
cross-implementation comparison.

| Seed | Generated IDs | Stop | Output replay |
|---:|---:|---|---|
| 2014 | 74 | EOS | Exact |
| 2015 | 128 | Cap | Exact |
| 2016 | 124 | EOS | Exact |
| 2017 | 128 | Cap | Exact |
| 2018 | 128 | Cap | Exact |

Replay equality is not answer correctness. Actual text retains repetition and
semantic drift, including “Tim and the bird the bird”; three outputs reach the
cap. This is a bounded TinyStories language reference, not useful general chat
or a native geometric model. Generation recomputes the full prefix without a KV
cache, so its **6.85 process seconds** are an offline reconstruction cost, not
optimized serving throughput. Full reference evaluation took **57.04 process
seconds**, including input verification and preflight.

## Verification and resources

- Optimized offline Rust build and six focused library tests pass: continuation
  counts/boundaries/normalization, causal cache, serialization, target alignment,
  stable vocabulary loss and historical sampling/stopping.
- The shared forward refactor also passes the actual 32-position CPU integrity
  check: 131,072 logits, maximum error `0.000015259`, 32/32 top-one matches, finite
  nonzero Q/K/V gradients, all three finite differences and exact restoration.
- Independent review verifies all four sealed roots and actual generation files.
  Model execution required no retries or gate changes.
- Remote Desktop Commander supervises the overlapping local count-fit and
  reference pipelines, then count evaluation. Its connected device is the same
  M1 Mac, not extra remote hardware. Three agents contribute implementation and
  independent review with separate ownership.
- All five model/integrity process times sum to **85.48 seconds**; overlapping
  processes mean this sum is not campaign elapsed time. Build/tests and final
  binary build took **202.71 + 5.71 seconds**. GPU-active kernel time, physical
  energy and future native training throughput are not measured.
- The [prospective budget](../evidence/reference-baselines-budget-2026-09-24.json)
  and [closeout](../evidence/reference-baselines-closeout-2026-09-24.json) retain
  complete-cycle charges, storage and the physical reserve. No unique material
  was deleted; no paid compute was used.

## Engineering conclusion and next action

The retained neural reference genuinely executes and decisively outperforms this
local count/cache model. The comparison does not isolate which architectural
component causes that advantage; it establishes a working language target and
an honest baseline. More A1–A4 selector repair would not supply the missing joint
language-learning path.

Implement **one continuous recurrent-memory learner** in the shared Rust training
tool: observed token and prior state → provisional gated state → causal vector
Q/K/V read with NoRead → state update → normalized vocabulary/copy output. Exact
occurrence and token identity stay in the tape. Language credit must reach the
read, written representations, recurrent update and output together. Use matched
ordinary recurrent and proposed quaternion-transport arms, preserving identical
information, reset and exposure rules.

Freeze the complete campaign before fitting, using measured local throughput and
the plan's 30-million-token-per-arm planning anchor. Include resumable optimizer
checkpoints, retained natural-language fit and development curves, actual loaded
free generation, and complete changed-source/NoRead outputs. The working-learner
decision and comparative geometry promotion are separate; an initial one-seed
campaign is exploratory, with the canonical multi-seed requirement still applying
to promotion. Quantizer diagnostics, hard admission and integer export remain
later responsibilities, with D0-b and terminal D5 serving constraints intact.
