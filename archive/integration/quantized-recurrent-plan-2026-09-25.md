# D8 rung 2 — quantized recurrent continuation

**Recorded before model execution, September 25, 2026.** This is the next
bounded implementation of the [canonical ladder](project-track.md), following
the [completed recurrent learner](joint-recurrent-result-2026-09-25.md).
#973 owns the outcome under #820. The owner requested continuation with
concurrent RDC research. The final objective and D0-b/D5 remain unchanged.

## Question and controls

Can the learned recurrent attention/state/output path retain its language and
context effects after the multiplicative parameters are restricted to four-bit
codes and the recurrent interfaces to declared fixed-point grids?

Continue each selected step-7,324 parent with its weights, Adam moments, global
sampler clock, B16/T256, two CPU gradient shards and data seed intact. Fit a
quantization-aware quaternion arm and ordinary Householder-pair arm, plus a
continuous continuation of each parent at identical additional exposure. No
teacher, architecture expansion, extra loss or new corpus is introduced.

The planning dose is 1,024 additional updates (4,194,304 targets) per arm, with
256 ramp updates and 768 fully quantized updates. A complete-step profile must
confirm the resource projection before this dose is frozen in the run receipt.
Report the final common step; preserve the midpoint for recovery, without
selecting whichever comparison-tail result is most favorable. Previously
exposed development remains development; no new final holdout is opened.

## Declared forward and backward

For a frozen power-of-two step `s=2^e`, use

`Q(x) = s * round_away_from_zero(clip(x/s, q_min, q_max))`.

Every `.weight`, including tied embedding and output normalization gain, uses
signed four-bit codes `[-7,7]`; minus eight is reserved. Matrices use one exponent
per output row. A vector uses one exponent. Additive biases and read-age entries
use signed sixteen-bit codes `[-32767,32767]`, because they are additions rather
than multiplicative coefficients. The wider offsets are explicit storage cost.

Calibration reads only the parent parameters. For each four-bit row, minimize
reconstruction squared error over exponents `e_ceiling-2`, `e_ceiling-1`, and
`e_ceiling`, where `e_ceiling=ceil(log2(max(abs(x))/7))`; bound and deduplicate the
candidates to `[-24,16]`, preferring the larger exponent on an exact tie.
Sixteen-bit vectors use the ceiling rule. An all-zero row uses exponent zero.
These exponents are frozen for the entire continuation and bound in every
checkpoint. No learned scale variables or optimizer reset is hidden here.

At full strength, the surrogate is
`stop_gradient(Q(x)) + (C(x) - stop_gradient(C(x)))`, where `C` has value `x`
inside and at the representable endpoints and a detached clipped value outside.
Its derivative is one inside, zero strictly outside. Parenthesizing the exact
zero difference avoids cancellation that could move the forward off its grid.
Training uses `(1-alpha)*x + alpha*Q_STE(x)` with
`alpha=min(1,(completed_step+1-parent_step)/ramp_steps)`. Every development,
generation and packed evaluation uses **alpha=1**. The separate shadow
evaluation disables all quantizers on the same trained floating parameters.

| Interface | Code range | Exponent |
|---|---|---:|
| Transport output, provisional state, read vector, final recurrent state | -32767..32767 | -11 |
| RMS-normalized vectors and final output hidden vector | -32767..32767 | -10 |
| Fused affine outputs, query/key/null, read scores, vocabulary logits | -32767..32767 | -8 |
| Tanh candidate/update/value and normalized transport coordinates | -32767..32767 | -14 |
| Sigmoid gates, including exact zero and one | 0..32768 | -15 |

Quantized parameter tensors are prepared once per full-window gradient shard,
retaining the original Var identities. Incremental sessions snapshot them once.
No graph-bearing parameter cache survives an optimizer update. Reads still
precede current writes, and full-window training retains all earlier-write
gradient edges. The exact observed token/occurrence tape is unchanged.

## Geometric and numerical scope

Quantize signed quaternion/Householder unit coordinates after normalization;
do not renormalize them off-grid. Quaternion `q` and `-q` remain distinct.
Coordinate rounding of a unit four-vector at spacing `2^-14` gives Euclidean
error at most `2^-14`, before subsequent computation. This is a local rounding
bound, not a bound on 256 recurrent steps. Both transports become approximately
orthogonal. State drift and likelihood by position must be reported.

The ordinary map is `H(v)H(e0)`: two reflections, determinant positive one for
exact unit vectors. It is not a single reflection. Immediate 600-cell/2I
snapping adds a separate transport restriction; it is deferred to the matched
codebook/transport rung. The identity-centered parameterization alone does not
prove that trained rotations stay small. A metric using `abs(q dot g)` would
silently identify the two quaternion signs and is inappropriate here.

This rung executes **quantized values in an F32 emulator**. It still uses dense
matmul, elementwise products, RMS/unit normalization, sigmoid/tanh/softmax,
probability mixture and sampling. Softmax probabilities are kept normalized;
no unnormalized probability rounding is inserted. Fixed numerical constants
also remain floating. There is no integer-kernel, multiplier-free, bounded
admission, sparse parameter-access, exact `Z[phi]`, Hamiltonian or energy claim.
Those execution changes require their own matched retained-quality evidence.

## Actual artifact and early inspection

Training checkpoints contain floating shadow parameters, moments, frozen scales,
schedule and step. A separate export contains packed four-bit codes, sixteen-bit
offsets, integer exponents, model configuration and hashes. Its loader reads no
shadow checkpoint. It reconstructs exact dyadic parameter values for the shared
F32 emulator. Verify the complete file set, packed decode, same-input output
and generated-token parity, and inspect actual loaded generation before long
fitting. Export statistics retain per-parameter saturation, reconstruction error
and code usage. Interface saturation counters cover the recorded smoke prompt
only; they are not a corpus-wide range proof.

The four-bit scalar grid already specifies its codebook and gain. No vector
commitment/usage regularizer is needed for this deterministic quantizer. A
matched H4/k-means/random vector-codebook comparison remains separately unrun.

## Frozen acceptance before fitting

Evaluate the final common step on the identical 233,472-target comparison tail,
with the original 16,384-target development prefix reported separately.

1. For each transport, loaded packed read-enabled NLL is at most **0.05 nats per
   target above its matched continuous continuation**, and below count/cache
   **2.391786178860742**. Report hard minus its own floating shadow and hard
   minus the zero-update calibrated parent as diagnostics, without selecting on
   these comparisons.
2. Combined NoRead (value feedback and pointer copy disabled) increases packed
   NLL by at least **0.02 nats**. Do not attribute the entire effect to one branch.
3. Actual packed generations on the five frozen prompts remain nonconstant and
   free of short-cycle collapse under explicit inspection. This retains the old
   engineering criterion, not a claim of useful or semantically reliable prose.
4. Report every frozen source-edit response against the old and continued
   parents, separating first-noun, exact completion and complete-pair results.
   At most **two previously correct first-noun rows per arm** may be lost from
   its selected rung-1 parent. Exact completions are reported without hiding
   individual regressions behind aggregates.
5. Packed reload preserves same-input hard outputs and fixed-seed generated
   IDs. All scored target probabilities/NLLs must be finite and normalized under
   the existing evaluator tolerance. Source, executable, data, scales and clocks
   must bind to the actual executed artifact.

Report early versus late context-position NLL gaps (0..63,64..127,128..191,
192..255) to expose accumulated error; do not claim long-range retrieval from
the short source-edit panel. Failure remains a rung-2 failure at this scope. Do
not relax a gate or patch individual prompts after seeing results.

## Research synthesis and independent review

[Jacob et al.](https://arxiv.org/html/1712.05877v1) motivates training with the
quantization seen at inference, including activation boundaries; its complete
integer graph is not delivered by a fake-quantized F32 forward alone.
[LSQ](https://arxiv.org/html/1902.08153v3) supplies a useful clipped-surrogate
comparison, but this design does **not** implement its learned step sizes.
[Bengio et al.](https://arxiv.org/html/1308.3432v1) motivates the estimator family,
not empirical language retention in this model. The pinned Candle CPU source
was checked for rounding and backward behavior; raw `round` has no useful
backward gradient, so the explicit surrogate is necessary.

Two RDC-launched DeepSeek sessions read the actual source independently for
mathematics and systems review, while native implementation subtasks used
explicit file ownership. RDC operates on the same Mac, not another compute
host. Adopted systems findings include once-per-window weight preparation,
immutable quantization bindings, the separate hard loader and continuous
controls. The mathematical reviewer made incorrect paired-reflection,
quaternion-metric and STE claims; those are corrected above. Its exploratory
NumPy reconstruction went beyond the requested scratch algebra and is retained
as advisory material only, not adopted as a model implementation or used as
Rust evidence. No reported NumPy likelihood/range number is an acceptance result.
Learnable exponents, probability rounding and extra auxiliary losses are not
added on that basis.

## Resources and delivery

The [prospective resource receipt](../evidence/quantized-recurrent-budget-2026-09-25.json)
sets the complete-cycle projection, cumulative extension, physical reserve and
storage allowance before build/model execution. Use the measured two CPU
gradient shards per process, at most two fits concurrently, nested BLAS/Rayon
threads one, one Cargo process, and 256-token context throughout. Reuse the warm
target cache; preserve both parents, negatives, source and worktrees. Charge
profiles, failed attempts, resumes, evaluation and delivery without resetting the
shared ledger. Physical energy remains unmeasured. Deliver code and the exact
accepted/rejected result through protected GitHub review and update #973/#820.
