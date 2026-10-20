# Contextual emission principal review: preserve the signal, correct the learning contract

September 21, 2026. Review of PR #1335 submitted head
`51b275bf828ee7555425648fa7dd8d4464e5fa29`, based on reviewed PR #1334
`cfc5f7b6f1dfba9e44767f83ff123fbac41f4d8a`.

## Decision

Continue **one-read contextual transformation with shared low-bit emission**.
Do not increase width on the submitted capacity diagnosis. The changed older
payload task is a better causal instrument than its predecessor, and the saved
reports contain correct uncopied outputs. However, the optimizer and numerical
contract are defective, correct source selection was not measured, the final
population was exposed before design selection, and loaded-artifact behavior
was not exercised. No integrated artifact or H4 advantage is qualified.

The next [comprehensive execution prompt](deepseek-consistent-emission-step-2026-09-21.md)
implements and fits a consistent hard-forward learner at the existing width
before any conditional representation change. This is constructive model work,
not another repetition of the old frozen-row or scalar-gate experiments.

## What survives, and what does not

| Submitted observation | Review disposition |
| --- | --- |
| H4 dev 35/180, tune 22/120, last population 13/120; pairs both 6/90 and 1/60 | Retained **reported aggregates** from the sealed experiment, not independently reconstructed per-position scores. The report lacks those rows. Some short saved trajectories are inspectable |
| C120 27/180, 16/120, 12/120; pairs both 0/90 and 1/60 | Retained reported aggregates. Their embedding bytes are identical: seed initialization ORs both nominal seeds with one. The small repeated-panel difference does not establish algebraic advantage |
| Local, scalar-copy, ReadDisabled and UpdateDisabled zero on reported scored panels | Retained scoped aggregates. They support continuing the noncopy mechanism; full loaded-predictor parity and useful complete answers remain unmeasured |
| 90/90 identical local inputs, outputs absent from payload bank | Construction/source checks support the intended identical-tail causal design; keep actual rows and fail-closed assertions in the successor |
| 180/180 selected sources | Counts **any read**, not the correct value or occurrence. Do not describe this as perfect source selection |
| Six ambiguous `(q0, relation, selected payload)` signatures | The target is determined by the relevant value. Ambiguity therefore indicates upstream selected-value disagreement for at least some constituents; it cannot exonerate selection |
| 15561 -> 7987/8773 “bits” | Raw-integer-logit loss at the wrong temperature, not canonical model CE. Do not divide the final number by a scale to repair nonlinear log-sum-exp |
| More coordinate passes change nothing | Does not establish convergence; committed parameters can disagree with the remembered incumbent score |
| Residual bound 32768 exceeds deficit 16640 | Loose necessary upper-bound screen, not an attainable shared-parameter margin certificate |
| Width 16 with 22 active rows implies insufficient capacity | Unsupported. No proof, exact feasibility result or valid corrected-fit evidence identifies width as the bottleneck |

All four roots `contextual-emission-{1,2,3,4}` remain unchanged. The first run
already evaluated the population called “fresh”; optimization and row budget
changed afterward. It is now an **exposed regression population**. Repeating it
three times provides neither three independent experiments nor an untouched
final evaluation. The [audit](../evidence/contextual-emission-principal-review-2026-09-21.json)
distinguishes verified seals/hashes from unavailable numerical reconstruction.

## Concrete defects and principal repairs

1. The ternary coordinate search saved the original coefficient once. Accepting
   an earlier candidate then rejecting a later one restored the original while
   retaining the better score. Commit the best coefficient and its matching
   score together; test this exact counterexample.
2. `train_output_map` and `served_nll_bits` treated fixed-point integers as nats.
   Convert with the parent `f_bits` in both the float objective and quantized
   evaluation. Train on the declared residual shift, set **before** fitting.
   In the submitted run the float fit used shift zero and serving used shift ten.
3. Fitting called `choose_scored`; serving called `read_step`/`choose_with`.
   Use the target-free causal reader at both boundaries and retain intended
   versus selected exact occurrence, value and payload position separately.
4. Exports were reloaded after evaluation and compared as structs, then
   discarded. Future evaluation must actually consume independently reloaded
   parameters and compare full predictions with the original objects.
5. Three copies of the residual arithmetic differed in clamping and used scalar
   numeric multiplication. Centralize signed accumulation plus power-of-two
   shift with common clamping. This is a scoped source repair, **not** a
   whole-path instruction, allocation, latency or energy qualification.
6. Decode actual artifacts before inferring initialization differences. The
   nominal seeds become identical after OR with one, and saved embedding hashes
   match. Keep common initial parameters and matched learning budgets explicit;
   there is no seed confound here.

The principal patch adds focused regressions and corrects these boundaries.
No corrected model fit or fresh evaluation is claimed by this review. The
future learner still needs a consistent quantized objective and useful
end-to-end behavior; compilation does not establish either.

## Mathematical diagnosis before expansion

**An actual upstream information loss is already visible in the saved maps.**
H4 value codes are `[22,22,11,22,4,89,89,27]`: payloads `{485,486,488}`
share one code and `{491,500}` share another. C120 also merges two pairs.
On identical-query/source-role pairs, equal value codes imply equal update
states when the correct sources are selected. The different required outputs
then cannot both be produced by any head. The reconstructed H4 fixtures have
14/90 development, 11/60 tune and 10/60 reused-final pairs of this type; C120
has 5/90, 4/60 and 7/60. These are conditional representation obstructions,
not independently measured source-correctness rows or a complete error budget.
Repair learning so useful value distinctions survive; readout width alone
cannot resolve them.

A constant output selected by development frequency would score **20/120** on
the reconstructed reused panel, versus H4's reported 13/120. The constant
solves no pair. Thus paired context use and single-target accuracy are both
needed: one correct pair is useful component evidence, not reliable task
competence or an advantage over a reasonable task-level predictor.

For actual feature `d_i = R(q1_i)-R(q0_i)` and shift `s`, put
`phi_i = 2^s d_i`. With lowest-ID tie breaking, unsaturated integer logits require

```text
(W[target]-W[competitor]) . phi_i >=
    local[competitor]-local[target] + (competitor < target ? 1 : 0).
```

Check saturation separately on the actual path. Ternary target and competitor
rows can change their difference by at most `2*||phi_i||_1`; if the competitor
row is fixed zero, the bound is `||phi_i||_1`. These necessary bounds do not show
that one shared W can satisfy every example. Source mistakes, feature aliases,
row support, incompatible margins and optimization are distinct failure modes.
A feasible continuous relaxation is not proof of ternary feasibility; an
infeasible valid relaxation can exclude its contained ternary family. Avoid an
unbounded solver campaign: use a concrete witness only if the repaired fit stalls.

There is a conditional geometric bridge more specific than “more capacity.”
Write a state-potential residual as `h(q*g)-h(q)`. On a complete finite orbit,

```text
sum_j [h(q_j*g)-h(q_j)] = 0,  q_j = q0*g^j.
```

If one target must gain a strictly positive pairwise correction against the
same competitor at every point of that orbit, this family cannot do it.
Increasing feature width does not remove the telescoping identity. **No such
witness has yet been measured in this run.** Query support may be far smaller.

If an actual nuisance-frame/cycle witness appears, test the existing relative
transport before higher dimension:

```text
d = inverse(q0) * q1 = T[relation] * V[payload]
residual = h(d) - h(identity).
```

The C120 control uses `(q1-q0) mod 120`. Both give exact zero for NoRead. The
relative element is invariant under common-left frame change because
`inverse(a*q0)*(a*q1) = inverse(q0)*q1`. Arbitrary seeded features do not make the
original emitter equivariant. This alternative deliberately removes absolute
frame information, so retain it only when that information is nuisance for the
task; do not assume it is universally irrelevant to language.

## Place in the whole geometric roadmap

1. **Now:** learn a useful, causally validated one-read transformation through a
   shared bounded emitter, with correct source/action/score/export contracts.
2. **Then:** reuse owned references and shared Read/Update/Emit/Stop for one
   dependent read and a derived answer absent from all source payloads.
3. **Supporting when required:** contextual role/scope persistence and exact
   versions/lifetimes, separate from learned relation and emission state. Match
   retained bytes against FIFO/shared-state alternatives on actual distractors,
   nested scopes and corrections.
4. **Broaden the same model:** source-separated prose/conversation, durable
   memory and executed Rust composition; retain old controls when proposing
   integration, not merely when a new toy panel improves.
5. **Scale and qualify:** useful quality plus complete M1 latency, memory/table
   traffic and measured energy; one coherent API/WASM/Studio artifact.

The [mechanism synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md)
remains the inventory of current and retired donors. Prime/UOR identity owns
exactness; signed H4/Spin supplies finite directed transport; Hopf observation
must retain fiber if its distinctions matter; paired-H4/icosian structure is a
specific coupled construction, not two independent free memories. Normalized
E8 vectors lie on S7 but do not gain storage dimensions. Spherical harmonics
are orthogonal under an integration inner product, not automatically at finite
sampled/quantized token states; learned routing, aliasing and capacity still
need tests. Scalar compatibility is already useful as a selection score. Twin
primes, literal supersymmetry, quantum spin and spacetime analogies presently
provide no implemented missing language operator. Keep the useful mathematical
operations and reject unsupported semantic claims.

## Relevant primary research checked September 21

- [Learned Step Size Quantization](https://arxiv.org/abs/1902.08153), ICLR 2020:
  quantizer scales and gradient scaling belong in the training contract. We
  retain a declared power-of-two serving shift rather than importing arbitrary
  learned floating-point scales.
- [ParetoQ](https://arxiv.org/abs/2502.02631), revised October 2025: very-low-bit
  learning and quantizer design can materially alter results. It supports
  resolving training before blaming width; its transformer measurements do not
  predict UOR-R4 performance.
- [Low-Rank Ternary Adaptation](https://arxiv.org/abs/2608.24469), August 2026:
  a current discrete-domain adaptation reference. Its multiplicative adapter
  is unsuitable as a direct replacement for zero-initialized output weights:
  multiplication cannot activate a zero coefficient. Reuse the hard ternary
  forward/latent training idea only after inspecting its implementation; no
  transformer or Kronecker backbone is adopted.

These sources motivate bounded design choices, not claimed UOR capability.
The orbit observation above is an elementary derivation for this operator,
not an experimental finding or an appeal to these papers.

## Delivery, resources and qualification

PR #1335 was open at review start; the owner checkout was clean on `74fef088`
and is preserved. Work uses the existing isolated principal-review worktree.
The [ledger](resource-ledger-2026-09-19.md) reconciles the missing reported
3400000-ms debit, preserves the already-applied allowance, and records the
principal checks and physical storage. The four report roots and original raw
receipt are preserved. Read current-state and the final check receipt for the
executed tests and resulting cumulative balance. No model promotion, fresh
final, useful prose, H4 dominance or whole-path D0-b/energy claim follows.


Executed validation: **seven focused tests pass**, offline touched-runner check,
formatting, wording, JSON/link and diff checks pass. Existing warnings are
retained. [Check and resource receipt](../evidence/contextual-emission-principal-checks-2026-09-21.json).
These checks exercise the source repairs; the next corrected model fit and
loaded full-panel behavior remain NOT_RUN.
