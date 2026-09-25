# D8 rung 2: projected-shadow continuation completed and rejected

**September 25, 2026. Decision: reject this paired recipe for retention.** Both
arms complete the prescribed continuation and pass four of the five frozen
gates. Both fail the likelihood allowance. No model is promoted, no gate is
changed, and the [previous negative](quantized-recurrent-result-2026-09-25.md)
remains intact. #973 and programme #820 remain open.

The [prospective plan](projected-recurrent-plan-2026-09-25.md) changes one complete
optimizer policy: project stored parameter shadows into their existing dyadic
ranges at entry and immediately after AdamW. Both arms start from their retained
fully quantized step **7,836**, preserve moments, clocks, scales, data and B16/T256,
and finish at fixed step **8,348**. Each adds **512 updates / 2,097,152 target
visits**; total additional work is **1,024 updates / 4,194,304 visits**. This
replaces the original second-half trajectory for comparison, but its new work is
charged additionally. No checkpoint is chosen using outcome quality.

## Loaded results and unchanged gates

All figures below use the same **233,472-target comparison tail**, separately
from the 16,384-target tune prefix. This is one paired seed on previously exposed
development, not a fresh final holdout. Lower NLL is better.

| Comparison-tail NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Matched continuous step 8,348 | 2.090518499 | 2.064403221 |
| Prior unprojected packed QAT | 2.149631373 | 2.144230654 |
| New projected packed model | **2.150319798** | **2.147928712** |
| New projected shadows, all quantizers disabled | 2.113220567 | 2.098025647 |
| New packed whole-prefix NoRead | 2.637288928 | 2.618333390 |
| Count/cache | 2.391786179 | 2.391786179 |

| Frozen gate | Quaternion | Householder pair |
|---|---|---|
| 1. Packed minus matched continuous ≤ +0.05, and below cache | **FAIL: +0.059801299**; below cache | **FAIL: +0.083525491**; below cache |
| 2. Combined NoRead penalty ≥ +0.02 nats | PASS: +0.486969130 | PASS: +0.470404679 |
| 3. All five actual generations nonconstant, without short-cycle collapse | PASS, limited panel | PASS, limited panel |
| 4. Lose at most two rung-1-correct first nouns | PASS: two lost | PASS: one lost; one gain does not cancel it |
| 5. Loaded parity, finite normalized predictions and actual provenance | PASS at the declared measured boundaries | PASS at the declared measured boundaries |

The [machine-readable result](../evidence/projected-recurrent-result-2026-09-25.json)
retains row comparisons, source-edit responses, all generations and exact input
bindings. Original control evaluations are reused as sealed evidence on the same
population; their training is not rerun. The Rust comparison independently joins
all 249,856 targets against the retained reference/count/cache and reproduces
the recorded means.

### Generated text and source retention

The principal inspected all ten new packed continuations. Neither arm produces
a constant or short-cycle continuation on this panel. The ordinary arm's former
period-two `daddy` cycle is absent: its corresponding continuation is now
`and daddy had a very special day.` followed by EOS. This does not establish
reliable prose: other samples drift between characters, objects and situations,
contain malformed words or confused causality, or reach the token cap.

| Source-edit measure | Q rung 1 | Q continuous | Q unprojected | Q projected | Ordinary rung 1 | Ordinary continuous | Ordinary unprojected | Ordinary projected |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| First noun, /32 | 28 | 26 | 27 | 26 | 31 | 32 | 32 | 31 |
| Exact completion, /32 | 28 | 23 | 19 | 23 | 23 | 26 | 25 | 22 |
| Both edited pair answers exact, /16 | 14 | 9 | 8 | 11 | 10 | 12 | 12 | 10 |

Quaternion loses the two previously correct `brush.` / `comb.` rows of
`story-source-edit-08`, answering `door.`. It loses five original exact completions
and gains none. Versus the unprojected sibling, it loses two exact completions
and gains six; versus continuous it loses four and gains four. Equal totals do
not imply equal answers. Ordinary loses the old-correct `bowl.` row, answering
`favorite toy.`, and separately gains `cap.`. It loses three original exact
completions and gains two. Versus continuous it loses four exact completions;
versus unprojected it loses three, with no gains in either comparison. Every
individual response is retained in the evidence receipt.

These prompts remain 56–62 tokens long. They do not establish retrieval beyond
64 positions. Whole-prefix NoRead disables both value feedback and pointer-copy
probability, so its large effect is their combined contribution, not isolated
geometric attention or transport causality.

## What projection actually did

At entry, **41,184 quaternion / 43,770 ordinary** shadow coordinates were outside
their fixed ranges; projection reduced both counts to **zero**. All **1,678,466**
hard parameter values/codes and all **1,048,576** probabilities from an actual
256-token forward remained bit-identical. AdamW moments, global and named
clocks, variable identities and next sampled inputs/targets were preserved.
There were no entry optimizer updates or sampler advances.

The first scheduled full-hard batch loss also exactly matches the original
unprojected step 7,837 in each arm: **2.099524736 / 2.075936317**. This is a
measured entry witness, not an assertion that later trajectories are identical.
All 512 new updates are fully quantized and record post-AdamW projection.
Repeated projected-coordinate events total **4,495,543 / 5,173,075**; a coordinate
can be counted more than once. Final clipped coordinates are zero in both arms.
The final packed codes differ from their unprojected siblings at **203,479 /
231,360** of 1,678,466 coordinates.

The policy therefore executes and changes the learned model. Nevertheless,
packed NLL is **0.000688425 / 0.003698058 worse** than the original unprojected
sibling. The shadow NLL improves by **0.032927069 / 0.024273486**, while the new
packed-minus-own-shadow gaps are **0.037099231 / 0.049903065**. This separates
successful range enforcement from successful language retention. It does not
prove that weight rounding, fixed scales, interface precision or optimization
alone caused the remaining error.

| Zero-based positions | Q packed − continuous | Ordinary packed − continuous | Q packed − shadow | Ordinary packed − shadow |
|---|---:|---:|---:|---:|
| 0–63 | 0.058277509 | 0.078305386 | 0.034216056 | 0.047556000 |
| 64–127 | 0.060138520 | 0.086789040 | 0.037509735 | 0.051821384 |
| 128–191 | 0.058100407 | 0.085908415 | 0.035986413 | 0.049869550 |
| 192–255 | 0.062688760 | 0.083099123 | 0.040684722 | 0.050365325 |

Each quarter has 58,368 comparison targets. The error is present throughout the
window; these aggregates do not establish a distant-memory failure.

### State difference is not a quality surrogate

The separately loaded state diagnostic uses the **original 64 tune windows**,
not the likelihood comparison tail. It observes 4,194,304 finite coordinate
pairs per arm, after verifying packed parameters against the quantized shadows.

| State metric | Quaternion | Householder pair |
|---|---:|---:|
| RMS coordinate difference, all positions | 0.045207091 | 0.047281852 |
| RMS, positions 0–63 | 0.043815056 | 0.045437235 |
| RMS, positions 64–127 | 0.045798085 | 0.048123611 |
| RMS, positions 128–191 | 0.045559645 | 0.047481249 |
| RMS, positions 192–255 | 0.045626958 | 0.048035412 |
| Maximum absolute difference | 0.670405507 | 0.556860283 |
| Packed RMS state L2 norm | 4.470895713 | 4.257426832 |
| Shadow RMS state L2 norm | 4.453793344 | 4.222923885 |

The all-position RMS differences fall from the prior **0.059484643 /
0.055451185**, yet packed likelihood does not improve. Lower state reconstruction
error is therefore insufficient as the decision objective here. Quarter averages
show no large progressive rise on this population, but are not a stability proof
or an isolated transport measurement.

## Execution and artifact boundary

Executed source is **`66349cdb69883dd7d438c76394c22f4e19000c73`**. The pinned Rust
training/evaluation executable SHA256 is
`b14698807c4ab2635ca1de777f2c882d75b1d1ca96ebe61005581f6bcf360cec`;
the state diagnostic executable is
`0e20b1a3683765f32abfa3a827910b3aaa6d02796d675b6acbda5588abb1ec18`.
All model runs execute optimized CPU code with Accelerate. Both fits run
concurrently through RDC, with two complete-sequence workers each and nested
BLAS/Rayon threads limited to one. The paired fit envelope is **2,167.629 seconds**
(about 36 minutes); this is not an energy or inference-throughput measurement.

Artifact container:
`/Users/casey.allard/uor-r4-investigations/projected-recurrent-20260925`.
The final roots are `fit-{quaternion|householder_pair}-projected-1/checkpoint-final`
and `final-packed-{quaternion|householder_pair}-1`; all fitted parents, controls,
negative candidates and exclusive attempt roots are preserved. Fit, export,
evaluation, comparison and state reports are sealed and verified.

Both packed reloads have **zero measured 256-token probability difference** and
equal fixed-seed generated IDs/distribution hashes. Maximum population
normalization errors are **2.8971e-6 / 3.1775e-6**, below the existing 0.0002
tolerance. Both 48-token smoke interface audits observe 537,877 values with
zero clipping/nonfinite values; that is not a corpus-wide clipping bound.

Packed parameter payloads remain **847,876 bytes per arm**. The complete required
three-file models total **1,168,748 / 1,168,818 bytes**, excluding tokenizer,
executable and audit reports. Four-bit multiplicative weights and sixteen-bit
additive offsets still expand into an **F32 numerical emulator**. Dense matrix
operations, nonlinearities, normalization, dynamic products and sampling remain
floating point. This does not qualify D0-b integer execution, D5 parameter
sparsity, exact H4/Z[phi] serving, Hamiltonian dynamics or energy savings.

Local validation includes **22 focused optimized Rust tests**, formatting,
claim wording and actual loaded artifact execution. An initial Cargo command
used an incorrect binary target name and exited before any model launch; the
correct command built successfully. The first analysis script compared packed
metadata directly with its floating parent's metadata and failed. The corrected
analysis verifies the exact source-defined packed transformation plus the actual
parent checkpoint/campaign hashes. It changes no Rust model, data or gate and
requires no training rerun. Both original error logs are retained.

The [resource closeout](../evidence/projected-recurrent-closeout-2026-09-25.json)
records cumulative time, observed memory/storage and unmeasured limits.
Across 584 resource samples, peak aggregate model RSS is **7,530,168,320 bytes**
(7.01 GiB), below the configured 8 GiB stop. Minimum sampled physical free space
is **22,765,936,640 bytes**, above the **21,676,163,072-byte** stop. Current
retained worktree/new-or-replaced compiler/campaign allocation plus full
temporary and metadata allowances is **2,044,174,336 bytes**, below the 2.5 GiB
ceiling. These are sampled/inventoried values, not continuous compiler-peak or
historical gross-allocation certification. No cleanup occurred this milestone.
The complete-cycle charge through **14:09:29.505525 UTC** is **5,309,506 ms**,
bringing the cumulative ledger to **525,236,590 / 532,800,000 ms**; the prospective
one-hour allowance extension is recorded before execution. The remaining
protected-delivery wall is charged separately after merge in the local
`delivery-closeout.json` and owning issue updates, without counting concurrent
process durations twice.
The [independent review](projected-recurrent-review-2026-09-25.md) records RDC
DeepSeek/Kimi findings and principal dispositions. Queue compatibility
acknowledgements execute no tests.

## Recommended next milestone: isolate the numerical bridge once

**NOT_RUN.** Keep the joint language learner, exact occurrence tape, quaternion
arm and competitive ordinary control. Do not extend the same projected recipe
on the assumption that additional exposure will fix retention. The current
two-mode comparison disables parameter and interface quantization together, so
it cannot choose which numerical intervention deserves training next.

Implement one explicit, evaluation-only **2 × 2 parameter/interface precision
comparison** on both fixed projected final checkpoints. The four modes are:

| Mode | Parameters | All declared interfaces |
|---|---|---|
| FF | Stored floating shadows | Floating |
| QF | Existing fixed packed-code values | Floating |
| FQ | Stored floating shadows | Existing fixed grids |
| QQ | Existing fixed packed-code values | Existing fixed grids |

FF and QQ must reproduce the retained shadow and packed endpoints before the
new mixed modes can inform a decision. Do not train, recalibrate, alter moments,
reselect checkpoints or inspect a fresh final holdout during this diagnosis.
Bind mode, parent, source, grid and evaluator in each exclusive report. Use the
same full 249,856-target population, separated tune/comparison, row-aligned
likelihood and actual five generations plus all source-edit responses in each
new mixed mode. Limited causal/serialization checks support the new switch;
another broad test campaign is unnecessary.

For each arm report `L(QF)-L(FF)`, `L(FQ)-L(FF)` and the interaction
`L(QQ)-L(QF)-L(FQ)+L(FF)`, as well as both conditional costs against QQ. They
are deterministic forward interventions on one learned state, not additive
causal shares of training error or a prediction of the gain from retraining.
The mixed modes are diagnostic only and cannot pass the serving gate.

Use that complete result to choose **one** coherent bridge change. A dominant
weight effect motivates loss-aware fixed dyadic scale/group or rounding design
within the existing weight-bit contract. A dominant interface effect motivates
precision/range design for the affected recurrent/read/output path. A large
interaction requires a joint numerical design rather than assigning blame from
one marginal. None is chosen now. Each subsequent recipe must retain a matched
ordinary control, an explicit integer-realizable arithmetic plan, the original
five retention requirements and a complete prospective resource budget.

This is a finite diagnostic decision, not an open-ended ablation search. Bound
the complete build/evaluation/review cycle prospectively using the measured
costs; two CPU evaluation processes can run concurrently. Then return to the
canonical ladder: quality retention → bounded trained admission/transport →
integer export and useful complete outputs → workload/capacity expansion.
Primary literature supports treating weight and state precision separately,
but does not establish the winning recipe here; the review links the checked
sources and their limits.
