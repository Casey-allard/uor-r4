# D8 learned neighboring-code assignment — one paired recipe

**Prospective, September 25.** The owner authorized the
[precision result's successor](precision-factorial-result-2026-09-25.md#recommended-next-learn-rounding-decisions-within-the-existing-parameter-grids)
and requested concurrent RDC work with DeepSeek. This plan fixes one substantive
learning intervention. No candidate result is known at this freeze.

## Question and fixed representation

Can jointly choosing legal neighboring parameter codes retain the existing
language behavior under the original five hard-artifact gates? The previous
four-cell diagnosis identifies parameter-family loss, without guaranteeing that
this legal assignment exists or that this optimizer can reach it.

Use both exact projected step-8,348 parents, original per-row dyadic scales,
signed four-bit multiplicative weights and signed sixteen-bit additive offsets,
all five interface grids, tied embedding/readout, and recurrent architectures.
Keep all continuous/unprojected/projected/precision results and artifacts.
[Input identities](../evidence/learned-rounding-inputs-2026-09-25.json) bind the
actual checkpoints and prospective recipe inputs. No new corpus or final holdout
is opened; #1017 remains an offline reference, never a serving provider.

For each original shadow `w` and frozen scale `s`, freeze
`l=floor(w/s)`, `u=ceil(w/s)` in its current legal interval. Exact-grid values
have `l=u`, stay fixed and do not enter the commitment denominator. Learn one
alpha per coordinate; tied parameters have one shared decision. The offline
relaxation is `d=clip(1.2 sigmoid(alpha)-0.1,0,1)`, with inverse initialization
from the original fractional part. Soft values are `s*(l+(u-l)*d)`. The final
hard choice uses the alpha midpoint; ties retain the original round-away-zero
code. Hard values are materialized independently before the existing codec
exports/reloads them. The original parent and its optimizer remain untouched. This refines the prior
illustrative floor-plus-binary equation by fixing exact-grid coordinates at
their single floor/ceil neighbor; it narrows the allowed search set explicitly.
A negative result cannot rule out the larger two-code convention.

The actual full-256-step language graph consumes those differentiable parameter
tensors once, with all retained interface quantizers and their surrogates active.
Do not quantize the soft parameters a second time. Independent batch shards
preserve whole sequences; their mean alpha gradients are combined before one
optimizer step. This remains an F32 learner/emulator, with no integer, D5
sparsity, Hamiltonian, geometric-advantage, general-language or energy claim.

## Frozen learning and coefficient initialization

- Both arms: **512 updates, batch16/context256, 2,097,152 fitted target visits**.
  Reuse data seed240924 and the exact retained valid-window sampler, counters
  8348..8859, with no window crossing the two retained training stores.
- Alpha-only named AdamW: learning rate0.01, weight decay0, existing beta1=0.9,
  beta2=0.999, epsilon1e-8, global clip1, alpha numeric ceiling30. Parent Adam
  moments/clocks are neither repurposed nor presented as rounding updates.
- Loss: population-mean next-token NLL plus
  `lambda * mean_non_degenerate(1-|2d-1|^beta)`. The mean is across coordinates,
  not a mean of per-tensor means. Penalty is zero for the first102 updates;
  beta follows the configured cosine20→2 on completed-update counters102..512.
  The last update evaluates the schedule at511; the final hard decision is512.
- **One training-only initialization rule fixes lambda before any update.**
  On the initial soft parameters and unchanged hard interfaces, average alpha
  language gradients over the first eight declared B16/T256 training batches.
  Set lambda to their L2 norm divided by the L2 norm of the unit-coefficient
  global-mean commitment gradient at beta2. Use ratio1, with no sweep or
  evaluation choice. Reject zero/nonfinite norms. The sealed initialization
  report binds the exact sampled input/target hash and resolved numeric recipe.
  Apply this identical rule to both arms; derived coefficients may differ.
- Initialization adds32,768 target visits per arm, zero updates; fitting revisits
  those first eight batches under the frozen sampler. Combined processing dose
  is2,129,920 targets per arm, separately accounted. Norm balancing is a declared
  heuristic, not proof of optimal regularization or expected recovery.
- Save alpha/optimizer/cursor at256 and final512; a resource stop saves its
  actual final state and is incomplete, not a result at512. Resume requires the
  same resolved recipe and lineage. No checkpoint selection occurs.
- Evaluate only the final hard model on the existing full evaluator, read and
  whole-prefix NoRead, all five seeded generations and all16 source-edit pairs.
  The export's early loaded smoke precedes full population analysis. Training
  batches and the learning curve stay available for interpretation.

## Independent review and principal decisions

RDC launches a DeepSeek source/mathematics review and a DeepSeek implementation
session while the principal integrates the language graph and campaign. Their
roles have separate ownership. No Kimi session is used. Source inspection and
executed behavior carry validation; provider agreement does not.

The reviewer correctly identified a normalization issue: a coefficient on a
million-coordinate mean penalty cannot be copied from a layer reconstruction
objective using a sum. It also caught evaluation-only memory limits below the
prior measured training peak; the prospective training limits are corrected
before any model execution. The principal adopts measured train-gradient norm
initialization and retains the declared cosine/warmup mechanism. The reviewer's
suggested fixed lambda40/beta1 and Gaussian flip-rate predictions are **not**
adopted: transformed gradient estimates from one historical step are not actual
alpha measurements, Gaussian/noise assumptions were untested, and neither a
particular flip fraction nor recovery is guaranteed. Adam momentum can carry a
parameter through a clamp boundary, so a zero current derivative alone is not
proof that a coordinate can never move again. A gradient magnitude that is
larger would make a fixed penalty relatively smaller, not falsify that concern.
The implementation handoff also proposed ruling out code choice after a failed
recipe; that inference is rejected. The principal corrected error conversions
and a numerical tie risk: compute the initial fractional subtraction in F64
and choose hard codes directly from the sign of alpha, preserving distinct F32
values immediately on either side of a half-step.

[AdaRound](https://proceedings.mlr.press/v119/nagel20a/nagel20a.pdf) motivates
learning jointly useful rounding through a binary relaxation. The principal
inspected Qualcomm's actual
[loss implementation](https://github.com/qualcomm/aimet/blob/develop/TrainingExtensions/torch/src/python/aimet_torch/_base/adaround/adaround_loss.py)
and [weight wrapper](https://github.com/qualcomm/aimet/blob/develop/TrainingExtensions/torch/src/python/aimet_torch/_base/adaround/adaround_wrapper.py):
they use a warmup, summed penalty, cosine beta and a rectified sigmoid, while
reconstructing local outputs. This Rust candidate instead trains full recurrent
NLL with an explicit global mean and a fixed gradient-norm initialization rule.
No Python model/code dependency or external implementation is imported; the
paper's vision results do not predict this learner's outcome.

## Acceptance and decision

Keep the [original five criteria](quantized-recurrent-plan-2026-09-25.md#frozen-acceptance-before-fitting):

1. Each packed comparison-tail NLL must be no more than0.05 above its retained
   matched continuous control and below count/cache2.391786178860742.
   Continuous controls are2.090518498969539 quaternion and2.0644032207510112 ordinary.
2. Combined whole-prefix NoRead must worsen packed NLL by at least0.02.
3. Inspect all five real generated continuations for nonconstant output and
   short-cycle collapse; this is an engineering criterion, not useful prose.
4. At most two selected-rung-1-correct first nouns may be lost per arm. Compare
   every source row against all retained relevant artifacts; report exact
   completions and full pairs with gains/losses separate.
5. Hard export/reload preserves probabilities and seeded output; full evaluator
   probabilities/NLLs are finite/normalized, with actual source/executable/data/
   grid/clock provenance. Report early/late context-quarter results.

A successful paired result retains the numerical bridge at this scope and
permits rung3 trained bounded admission/transport, then rung4 actual integer
execution/useful outputs. A miss rejects this recipe; it does not prove scales,
optimization, geometry or prior learning history causal. Preserve the candidate
and choose the next direction from the complete outcome. No fallback sweep or
prompt-specific patch is authorized by a failure.

## Resources and delivery

The [prospective budget](../evidence/learned-rounding-budget-2026-09-25.json)
charges all preparation/build/calibration/fitting/evaluation/review/delivery
against the shared cumulative ledger. Two model processes, two whole-sequence
CPU workers each, nested backend threads1; one Cargo process reuses the warm
cache. Model RSS limits4.5GiB/process and8GiB aggregate follow prior measured
training peaks. New allocation ceiling2.5GiB; physical reserve18GiB plus128MiB
stop margin and64MiB checkpoint headroom. Preserve every unique artifact and
worktree; no cleanup or paid external training compute. The complete-cycle
ceiling is two hours, with per-fit start-update deadline2700seconds and a
checkpoint stop. Compiler peaks and physical energy remain unmeasured unless
explicitly observed. The standing authorization covers the recorded prospective
cumulative extension, never a silent reset.

**Execution resource adjustment, before resuming.** The original18GiB reserve
stopped both fits after one completed update when physical free space sampled
19,524,063,232bytes. Both exact alpha/Adam checkpoints are retained. Under the
standing local-resource authorization, record2GiB additional physical headroom:
reserve16GiB plus the unchanged128MiB stop and64MiB checkpoint margins. The
2.5GiB new-allocation ceiling and two-hour cumulative cycle ceiling remain.
The complete recipe resumes from update1, without recalibration or discarded
learning. The two original stop receipts are preserved under attempt-specific
names before clearing their operational stop paths. A configured2700second
process boundary may likewise checkpoint and resume the remaining updates of
the identical recipe within the complete-cycle limit; it never adds updates.
Physical-space movement is observed without attributing it to another app.

Compile the changed Rust path once with focused gradient/causality, legal-code,
serialization and checkpoint checks; add checks only for a concrete unresolved
risk. Record actual loaded outputs. Update the result/current state and owning
issues, and deliver through the protected PR path; queue acknowledgements do
not run tests. Do not turn the completed fit into a new diagnostic campaign.
