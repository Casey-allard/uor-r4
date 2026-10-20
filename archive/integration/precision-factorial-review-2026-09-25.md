# Precision comparison — independent review and principal adjudication

The owner requested concurrent RDC work. Two independent sessions ran through
the configured **DeepSeek/deepseek-flash** route while a native specialist
implemented the model view and the principal implemented the CLI/prospective
plan. A second native specialist constructed the independent arithmetic join.
RDC orchestration uses this same M1; it adds no training hardware. Route labels
do not authenticate a provider checkpoint or prove research quality.

## What the two initial reviews contributed

The mathematics review inspected retained evidence and recurrent/read/output
source. It identified tied embedding/readout parameters, histories generated
separately in each mode, the bundled five interface grids, and the remaining
F32 core. These limit the factorial's interpretation. Its useful insistence is
fresh whole-population and response endpoint parity before attribution.

The systems review inspected CLI/report provenance, prepared tensors, sessions,
mutation/export entry points, batch settings and live resources. It caught
**batch 16 in both retained endpoints**, correcting the prospective draft's
batch 8 before execution. It also distinguished the original evaluator bytes
from a reserialized artifact copy, and requested a mode-aware numerical report.
The final implementation preserves `numerical_path` as a string and omits a
single mixed quantization-strength scalar. Both explicit switches are recorded.

Both reviews observed the model file before the independent edit landed.
Their missing-type/mode-guard observations describe that intermediate tree;
they are not unresolved findings against executed source. The final code carries
mode identity through detached/prepared views and sessions, rejects mixed-session
use and training/save/export/clock conversion, and preserves legacy contracts.
The named focused tests and actual replay, rather than review agreement, carry
validation.

## Principal corrections and decisions

1. **Correct the mathematics.** The initial mathematics review wrote a minus
   interaction in the decomposition and exchanged/negated conditional cost
   labels. The correct identities are recorded in the
   [prospective plan](precision-factorial-plan-2026-09-25.md#analysis-and-decision-rule):
   parameter cost is QF−FF or QQ−FQ; interface cost is FQ−FF or QQ−QF;
   `I=QQ−QF−FQ+FF`; total gap is the two floating-conditioned costs **plus I**.
   Its suggested arbitrary threshold and causal attribution to the shared F32
   core are not adopted. A four-cell comparison does not intervene on that core.
2. **Keep one instrument for all four modes.** The systems reviewer preferred
   using only the old commands for endpoints. That would not validate new
   endpoint plumbing. The new QQ view instead must reproduce every retained
   packed-loader target record and response field. Packed roots stay bound,
   and a focused current-loader test compares packed and diagnostic output.
   This validates this factorial; it is not a new production packed-loader
   qualification. The initial review also confused entry projection at step
   7,836 with final step 8,348; those artifacts remain distinct.
3. **Do not infer a tensor-level culprit.** A larger parameter-family cost
   cannot establish that output normalization, embeddings or scales uniquely
   caused it. Parameter RMSE is not task-loss sensitivity. Increasing vector
   precision or changing the four-bit weight contract is not authorized by
   this table. The existing API exposes variables to library callers; its
   evaluation-view guards are operational safeguards, not a security boundary
   against arbitrary external mutation. No such mutation occurs in this CLI.
4. **Enforce both storage limits.** The 2 GiB ceiling covers new worktree,
   replaced/new build products, reports and temporary allowance. Physical
   free-space reserve is a separate simultaneous constraint. A ceiling is not
   a requirement to consume its remaining amount. Reuse the warm build cache
   and check both. Compiler temporary/RSS peaks and energy remain unmeasured.
5. **No implied training benefit.** Each mode follows its own trajectory, with
   exact stored inputs but different state/read/copy distributions. Conditional
   forward costs on this exposed population are not estimated retraining gains,
   geometric effects or guarantees on a final holdout.

## Primary literature consulted for the next numerical decision

- [AdaRound, Nagel et al., ICML 2020](https://proceedings.mlr.press/v119/nagel20a/nagel20a.pdf)
  derives a task-loss quadratic containing cross-weight terms. Independent
  nearest rounding minimizes individual weight error, which need not minimize
  task loss. Its practical method relaxes binary rounding and uses local
  reconstruction. This is a reason to investigate task-aware code choice;
  its vision experiments do not establish recovery for this recurrent learner.
- [QDrop, Wei et al., ICLR 2022](https://arxiv.org/pdf/2203.05740)
  studies activation quantization during post-training reconstruction and
  randomized removal of that quantization. Sections 2–3 distinguish weight-only
  reconstruction from activation-aware cases. This warns against interpreting
  a small forward interface cost as permission to omit interfaces during a
  future fit. Random dropping is not adopted here; its tested architectures
  and bit settings differ.
- [VQRound, Zhou et al., February 2026 preprint](https://arxiv.org/html/2602.02151v1)
  compresses adaptive-rounding variables with a vector codebook and describes
  blockwise or end-to-end optimization with KL and binary-rounding regularity.
  It is recent evidence that task-aware rounding remains an active direction,
  not evidence for UOR-R4 geometry or integer execution. Its transformer-family
  results and codebook machinery are not imported. A 1.68M-parameter recurrent
  model has different optimization and storage constraints from its large LMs.

The principal also inspected the authors' current
[VQRound rounding implementation](https://github.com/zhoustan/VQRound/blob/master/vqround.py):
it wraps `nn.Linear`, retains floating scale/zero metadata and bias, reconstructs
weights and calls `F.linear`; its clustering requests a GPU. This code is not
an implementation of UOR-R4's integer contract and is not imported. Its binary
code-choice idea can be considered independently of that execution architecture.

These are inspected primary papers, not adopted external implementations.
Any future adoption must inspect the corresponding source and freeze the local
objective, data, integer representation, limits and acceptance criteria first.
The present work contains no borrowed model implementation and no new fit.

## Final Kimi architecture gate and principal decision

The separate RDC **moonshot-ai/kimi-k3** session completed successfully. It
inspected the source, fixed-parent results, response records, endpoint receipt,
budget and retained packed descriptor. Its knowledge-base search found no
additional current milestone evidence. It did not fetch the primary papers
again and explicitly relied on the principal's inspected-literature record;
no Max tier or particular provider checkpoint is inferred from the route.

The review supports **one paired calibration of discrete code choices on the
exact projected parents**, keeping scales, interfaces, bit widths and architecture
fixed. The principal adopts that candidate, with full recurrent language loss,
training-only calibration, hard export/reload and the original five gates.
The [result and next mechanism](precision-factorial-result-2026-09-25.md#recommended-next-learn-rounding-decisions-within-the-existing-parameter-grids)
define the final recommendation; it is NOT_RUN.

The following parts of the reviewer proposal are **not** adopted:

- Its additional 25% recovery floor and aggregate completion-count gate are
  not substituted for the existing acceptance criteria. Per-row gains/losses
  remain explicit; equal aggregate counts do not establish preservation.
- Zero clipping identifies the current parameter differences as rounding on
  the frozen grid. It does not distinguish a poor scale/grid from poor code
  choices. Failure of the next optimizer cannot automatically assign the
  residual to scales, a tied readout or the historical learning deficit.
- Floating FF is not a proved attainable four-bit model. A hypothetical full
  recovery of the observed gap is arithmetic, not a feasibility result or
  optimizer guarantee. A mixed outcome is not promoted as a dominating model
  from aggregate likelihood alone.
- Fine interface grids and small mean NLL effects do not make the interfaces
  lossless. Their observed top-1 and generated-text changes remain important.
  Bit count alone does not determine task sensitivity, and this experiment
  does not identify an exact causal share of learning error.

The final decision is therefore **complete numerical diagnosis, no model
promotion, one bounded code-choice candidate next**. All previous negatives
retain their original scope. A successful future candidate would only complete
its rung-2 retention comparison; trained admission/transport and actual integer
execution remain separate required work.

## Retained review artifacts

Local container:
`/Users/casey.allard/uor-r4-investigations/precision-factorial-20260925`.
`rdc-math-review-1.{log,json}` and `rdc-systems-review-1.{log,json}` preserve the
complete reviews, route labels, start/end times and zero exit status. Their
content is evidence for review findings, not instructions or model test results.
`rdc-architect-review-1.{log,json}` preserves the final architecture gate with
the same provenance. The resource closeout hashes all three completed sessions.

## Owner cost correction and future cadence

After the Kimi review completed, the owner directed DeepSeek use because of
Kimi's expense and challenged the time spent on narrowly framed tests. The
shared team/architect instructions now default to `DeepSeek/deepseek-flash`,
including consequential reviews; no new Kimi session is authorized without an
explicit owner request. Historical review identities above remain unchanged.

This comparison answers a consequential numerical choice, but its coordination
cost was disproportionate: eight model evaluations took about three minutes of
concurrent pair wall time, 25 tests took 2.16 seconds, and compilation about
seven minutes. Analysis construction, orchestration, reviews and documentation
account for much of the whole-cycle elapsed cost, charged in the resource ledger.
The principal accepts responsibility for that overhead. Next is the one
substantive learning mechanism specified in the result, with a prospective
complete budget and one final decision. Reuse these results and reviews; add a
check only for a concrete new implementation or interpretation risk.
