# D8 precision comparison — parameter quantization dominates the measured gap

**Completed September 25, 2026. No new model is promoted.** The
[prospectively fixed comparison](precision-factorial-plan-2026-09-25.md) runs
both final projected parents through all four precision modes. Parameter
quantization has the dominant conditional **mean likelihood cost** in both
arms and every fixed partition. The present interface grids contribute much
smaller mean costs, while still changing generated behavior. This selects the
parameter representation/code-choice boundary for the next numerical repair;
it does not prove that one rounding algorithm, scale or tensor caused the gap.

## What actually ran

Rust source **`49dfd0b15486ba3a7a763c598257f651e857279f`**, executable SHA256
**`cf5ce19a5eb63b674bf9ee23fe671ceed477cb26ad8d5e3e796c974c04a01f9f`**.
Both sealed projected parents remain at step **8,348**. All eight runs use
**batch 16 / context 256**, the identical evaluator and **249,856 targets**:
16,384 tune and 233,472 exposed comparison targets. Every mode also executes
five seeded free generations and 16 source-edit pairs, giving **40 complete
free-generation records and 256 source responses**. There are zero optimizer
updates, scale changes, checkpoint selections or newly opened final holdouts.

All four fresh FF/QQ endpoint replays match **every byte of the retained target
records** and every field of the complete generation/source JSON except
`elapsed_seconds`. The arithmetic join verifies seals, exact targets, unchanged
parent/configuration/quantizer identities and every file in the eight bound
input roots. The new QQ command re-quantizes a floating parent; it is numerically
matched to the retained packed-loader output and does not claim to load that
packed root itself. A focused test also checks the current packed loader.

[Full arithmetic, identities, per-target changes and responses](../evidence/precision-factorial-result-2026-09-25.json),
[all decoded outputs](precision-factorial-outputs-2026-09-25.md),
[endpoint receipt](../evidence/precision-factorial-endpoints-2026-09-25.json).

## Fixed comparison-tail likelihood

First letter: parameters. Second letter: all five declared interface grids.
F bypasses that quantizer; Q applies the existing frozen grid. Lower NLL is better.

| NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| FF — shadows, floating interfaces | 2.113220567 | 2.098025647 |
| QF — quantized parameters, floating interfaces | 2.150291052 | 2.147912004 |
| FQ — shadows, quantized interfaces | 2.113217828 | 2.098039459 |
| QQ — quantized parameters and interfaces | 2.150319798 | 2.147928712 |

| Signed conditional cost, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Parameters with floating interfaces: QF−FF | +0.037070486 | +0.049886357 |
| Parameters with quantized interfaces: QQ−FQ | +0.037101970 | +0.049889253 |
| Interfaces with floating parameters: FQ−FF | −0.000002739 | +0.000013812 |
| Interfaces with quantized parameters: QQ−QF | +0.000028746 | +0.000016708 |
| Interaction: QQ−QF−FQ+FF | +0.000031484 | +0.000002896 |

The identity is `QQ−FF = (QF−FF) + (FQ−FF) + interaction`.
Interaction is symmetric and is not assigned to either factor. Parameter cost
stays positive and dominant on the tune prefix and all four comparison quarters:
quaternion costs span about **0.030871–0.038191**; ordinary **0.047090–0.052782**
across the two conditionings and these partitions. Small interface and interaction
effects sometimes change sign. This does not reverse the dominant-family
finding; it rules out treating their signs as a stable universal law. Block
standard errors in the evidence are descriptive, without assuming independent
corpus blocks or claiming a population confidence interval.

With floating interfaces, quantizing parameters changes **34,906/38,583**
comparison top-1 predictions. With floating parameters, quantizing interfaces
changes **441/452**. The switches therefore produce observed differences; their
declarations alone are not the evidence.

## Actual outputs prevent an overbroad conclusion

| Complete source answers, out of 32 | Quaternion | Householder pair |
|---|---:|---:|
| FF | 25 | 28 |
| QF | 22 | 22 |
| FQ | 25 | 28 |
| QQ | 23 | 22 |

These are short authored source-edit responses, not general language accuracy.
Against FF, QQ loses **four** previously complete quaternion answers and gains
two; ordinary loses **seven** and gains one. Gains do not cancel the losses.
Quaternion source-edit-08 edited changes the correct `comb.` to the wrong
`door.` when parameters are quantized. Several other failures preserve the noun
but continue past the required answer: `ball stuck in the wooden.`,
`shell under the wooden bench.`, and ordinary `coin home.`. The full record
retains each of the 64 source cases across all modes.

Interface quantization preserves those aggregate first-noun and completion
counts under floating parameters, but changes one wrong quaternion answer.
Under quantized parameters it recovers one quaternion completion (`coin.`)
and changes an ordinary continuation without making it correct.

More decisively, changing interfaces alters **9/10** complete seeded free
continuations with floating parameters and **8/10** with quantized parameters.
The principal inspected all 40 outputs, grouping identical texts without
discarding any mode. They retain semantic drift, pronoun/role mistakes, malformed
words and repetitive content. Examples include a goose wanting to wear another
goose and a bike climbing a tree. A tiny aggregate NLL difference is therefore
not behavioral interchangeability, and interfaces stay enabled in the next
candidate's fitting and hard evaluation.

## What the result does and does not identify

This is a valid forward intervention on **these two fixed learned parents**.
Each mode creates its own recurrent state, written keys/values and copy
distribution. Conditional costs include the entire resulting 256-step causal
trajectory. The parameter bundle includes four-bit multiplicative weights and
sixteen-bit additive quantities; tied input/output embeddings are one shared
parameter. The table does not localize error to one tensor, isolate scaling
from rounding, estimate a retraining gain, or establish a transport advantage.

The [projected candidates](projected-recurrent-result-2026-09-25.md) retain their
original failed +0.05-nat likelihood gate. Even **QF** is worse than matched
continuous by **+0.059773/+0.083509**. Removing the declared interface quantizers
does not close that gap. Conversely, the projected FF shadows themselves are
worse than continuous by **+0.022702/+0.033622**. The full quality deficit includes
that learning-path difference; it cannot all be called immediate rounding loss.

Both old quantized recipes and all prior controls remain preserved. Current
computation is still an F32 emulator with dense parameter access, floating
normalization/nonlinearities/probabilities, and no qualified integer serving,
D5 sparsity, Hamiltonian dynamics, general language/reasoning or energy result.
The programme remains at D8 rung 2 retention; rungs 3–4 are downstream.

## Recommended next: learn rounding decisions within the existing parameter grids

**One paired candidate, NOT_RUN.** Keep these exact projected parents, fixed
dyadic scales, bit widths, five interface grids, tied parameters and recurrent
architecture. Replace independent nearest rounding with **jointly learned
choices between the neighboring integer codes**, using the full 256-step
language loss on a fixed training-only calibration draw. Apply the same
procedure and exposure to the ordinary arm. This directly targets the measured
family while leaving the packed representation and inference contract fixed.

For each frozen scalar `w_i` with scale `s_i`, let `n_i=floor(w_i/s_i)`.
The candidate may choose

```text
c_i = clip(n_i + d_i, q_min, q_max),  d_i in {0,1}
w_i_hard = s_i * c_i
```

Offline optimization may use a differentiable relaxation of `d_i`, with a
declared penalty/schedule toward binary decisions. Its objective is the actual
end-to-end next-token NLL across the student-generated recurrent/read/write
history, with all current interfaces quantized. One shared decision must serve
both uses of a tied parameter. This is a Rust learning mechanism inspired by
adaptive rounding, not a reuse of the evaluation-only precision view for
training and not independent layer reconstruction. Temporary relaxed values
and optimizer state are learning artifacts; acceptance uses only the final
hard codes after export and fresh-process reload.

The exact objective coefficients, initialization, calibration sample identities,
optimizer, hardening schedule, fixed stopping point and complete resource
projection must be frozen before the first update. Use one selected recipe and
one final hard decision point, with open development used only as prospectively
declared. Do not score/select rounding changes on the comparison tail or source
answer panel. No new runtime teacher, floating adapter, wider weights, scale
search, per-tensor sweep or extra corpus is part of this candidate.

**Why these parents:** they provide exact paired FF/QQ references, a previously
verified packed path and zero clipped coordinates. The stronger continuous
parents are retained quality references, but swapping parents and recalibrating
their grids would combine another intervention. This clean comparison can
answer whether better code choices at the current representation suffice.
It does not assert that the projected parents are the best final models.

**Strongest alternative:** the fixed grid or prior learning path may be the
limiting factor. Nearest rounding minimizes scalar error, and a different code
assignment can exploit cross-parameter task interactions, but the factorial
does not establish that a better legal assignment is reachable. Floating FF is
not a promised attainable four-bit model. To restore the original likelihood
gate on these parents, hard loss must fall by at least **0.009801/0.033525**
nats in quaternion/ordinary. The latter is a demanding recovery; no guarantee
is inferred from the small interface cost or interaction.

**Advance only on the original five hard-artifact gates**, including the same
+0.05 likelihood allowance against the retained continuous controls and the
original read-effect, noncollapse, source-retention and provenance requirements.
Compare every source row with prior artifacts; gains do not hide losses. If the
fixed-budget candidate misses any gate, preserve its exact result and reject
that recipe. Do not automatically assign the residual to scales, the optimizer,
capacity or geometry, and do not launch a fallback sweep. Reconsider the
representation/learning contract from that complete outcome.

A retained success advances to **rung 3: trained bounded admission and geometric
transport**, then **rung 4: integer/table execution and useful complete outputs**.
It would still not qualify general language, Hamiltonian attention, D5 parameter
sparsity or energy savings. The long-term geometric model goal is unchanged;
this is the numerical repair needed to carry a learned model toward that goal.

## Validation and resource boundary

The optimized Rust build and **25 focused tests** pass, including all three new
precision-mode tests, packed reload, full-window gradients, causal incremental
sessions and codec/projection invariants. Actual model execution and complete
endpoint matching supplement those tests. No broad test-fixing campaign was
performed. The first formatting check found a line-wrap difference; it was
corrected before the executed source commit. Existing dependency dead-code
warnings were left at their original scope. The full staged whitespace check flags
18 trailing-space lines in literal generated text; those bytes are preserved.
The full base-to-head check passes for authored files with that one verbatim
output file excluded, and its complete contents match the retained output.

RDC runs two model processes concurrently on the same M1, with nested backend
threads set to one. The independent mathematics/systems reviews and the final
architecture decision are documented in the
[review and primary literature record](precision-factorial-review-2026-09-25.md).
Principal corrections to review arithmetic and stale in-flight source
observations are explicit. Queue compatibility acknowledgements execute no tests.

The [resource closeout](../evidence/precision-factorial-closeout-2026-09-25.json)
records preparation/build/evaluation/review/delivery wall, sampled model RSS,
physical storage and the shared cumulative ledger. No unique artifacts or owner
material are deleted. The [prospective budget](../evidence/precision-factorial-budget-2026-09-25.json)
is a two-hour complete cycle, not a fresh per-process allowance. Compiler peaks,
physical energy and historical gross storage allocation remain unmeasured or
unresolved at their stated scope.

## Recompute from the retained artifacts

The campaign-specific arithmetic source is
[`analyze_precision_factorial.py`](../../scripts/research/analyze_precision_factorial.py).
It implements no model and invokes the pinned Rust executable only as `verify`.
With the retained local input and output roots available, its `--endpoints-only`
pass precedes full analysis; `--root`, `--prior`, `--worktree`, `--exe` and
`--source-commit` select the explicitly bound locations. Existing analysis files
are never overwritten. The published JSON also binds the exact executed script,
so changing the arithmetic is a separately identified analysis attempt.

Model evaluation uses:

```text
uor-r4-training joint-evaluate-precision SEALED_SHADOW_CHECKPOINT EVALUATOR_JSON NEW_REPORT_ROOT cpu MODE 16
```

Use MODE `FF`, `QF`, `FQ` or `QQ`, with a separate exclusive root for each arm
and mode. The present artifact container is
`/Users/casey.allard/uor-r4-investigations/precision-factorial-20260925`;
the input container is `projected-recurrent-20260925`. Large retained local
checkpoints/target records are identified by manifests rather than embedded in
GitHub. An absent local artifact makes such a replay unavailable, not a pass.
