# The operator phase: geometric replacements for the gate, projections and head

## Status 2026-10-13

OP1 G2 Hamilton gate landed (#2260). OP2's dense HamiltonLinear was superseded
by the owner-directed SpiralCore BFS + retained FBS projection (#2279); that
replacement satisfies the plan's OP2 source entry. Recurrence geometric gates
landed (#2293), making the declared recurrence squash-free by source sweep.
OP3a and the shared native vector writer are Codex-owned under the ratified
split, implemented and finite CUDA-qualified on source
`09b3ee6df084beba1b9c6e98ef1e597bf2dc82bb`; protected delivery merged as
[#2301](https://github.com/UOR-Foundation/uor-r4/pull/2301) (`d41dc422f`).
The [head/writer record](../geometric-head-shared-writer-2026-10-13/README.md)
retains the five-test device qualification and limits. It does not qualify
Kimi's recurrence or the complete trunk. The rung-0 sheet
[retains its live bars and dependency list](../../harness/prep/m1-rank-margin-rung0-2026-10-13.md).
The [composed-path repair](../rung0-complete-path-2026-10-13/README.md) is
implemented and finite CUDA-qualified at `05c84bece50789163bd0f049a2ae76b8a314ef45`,
including ordinary dispatch, signed rate credit, GPU clipping and native
CLI/masked dev metrics:18 final controls pass at the actual rung-0 shape.
Optimizer update is compiled/source-reviewed only. Protected #2318 merge is
verified at `5b22739a31eafb131d25e098bb7f02ab803d4fe7`; its Amendment4 FIT
retains its original reference scope. Current Amendment5 has independent
scientific [Claude CONFIRM FIT6146716704](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6146716704), with unchanged bars. The physical
sheet remains PENDING for Kimi's mechanical TRAIN-mask/SHA/CLI/source pins
and FIT verdict restoration: the primary count reference must use masked
TRAIN targets, and the masked DEV oracle is diagnostic only. The
[reference/input-binding repair](../count-reference-2026-10-13/README.md)
is implemented at frozen source `2fd6358bbc52d2a202a872ad0954ebb12894606f`;
11 new finite controls PASS (9 metadata,2 CUDA); protected [PR #2328](https://github.com/UOR-Foundation/uor-r4/pull/2328) merged at `9e904a83448c451f738d5c1d45224dfdf73eb84e`.
[PM board6146134405](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6146134405)
now requires Kimi's mechanical sheet pins/verdict restoration, followed by
any explicit owner hold-release decision; Claude's source/design confirmation
is recorded above. Global training and the separate M2
training/rental hold remain HELD; scientific FIT cannot release them.

The original 2026-10-12 registration and repair are linked below as historical
evidence; this living plan includes the newer source and qualification status.

References #2029 (M1) and #2032 (M4). Kimi is project manager and owns
recurrence/gate source; Codex owns OP2/OP3a and the shared writer under the
[ratified split](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138381873). This record is repaired under the owner's
[D25 direction](../../integration/DECISIONS.md#d25---geometric-operator-phase-and-current-lab-direction).
The [late owner clarification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6135511346)
sets **OP2 first, G2-only OP1, OP3a, rung0, then OP4/rung0b**; OP3b stays later
research. A geometric feed-forward block is permitted. The constraint set is
frozen through the first compliant rung0 result; new concerns are deferred
proposal cards except cost/safety. This repair adds no constraint.
The [original registration](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6134571032)
and [original plan at aaa210ef4](https://github.com/UOR-Foundation/uor-r4/blob/aaa210ef46301623f10164ae5af586d7f558483a/docs/labs/operator-phase-2026-10-12/README.md)
remain historical evidence. This amendment supersedes their execution sequence,
CPU/cuBLAS comparator permissions and mixed-unit quality bars; it is **not FIT clearance**.

## Current decision and holds

**Training remains held.** M2's separate training/rental hold remains; the owner
authorized bounded pod development/build/finite kernel tests for the named
Codex projection claim and the subsequently owner-requested head/shared writer
finite qualification, without releasing either training hold.
OP1–OP3 are source/kernel implementation dependencies, not separate training
campaigns on an otherwise prohibited trunk. No full-model training, evaluation,
benchmark or generation runs until the entire executed path complies, its source
and device behavior are qualified, prospective TEST FITNESS is FIT, and the
owner explicitly releases the applicable hold. A FIT verdict does not release it.
An isolated lawful GPU kernel qualification requires a separate named decision,
prospective FIT and resource admission; it cannot contain banned operations,
execute a CPU model reference or become an implicit training run.

Retained/default historical source contains prohibited paths. The declared
native configuration replaces the dense projection/head; Kimi recurrence source
#2293 is integrated. Those deliveries and the margin objective do not establish
complete-path compliance or authorize a model run.
Every executed forward/backward/objective/output/auxiliary path must exclude
softmax, softplus, sigmoid, tanh, their equivalents and dense matrix contractions;
model compute is GPU-only with host fallback refused. Replacing cuBLAS with a
handwritten dense contraction does not satisfy the geometric replacement rule.
Historical source/artifacts/results stay preserved and inactive.

## OP1 — geometric gate source

Implement the candidate forward **and backward** on CUDA, including all gate,
retention/decay and output call sites reached by the declared configuration.
Trace recurrence squashes too; removing SwiGLU alone is insufficient.

- **G2 Hamilton gate:** quaternion gate `q(g)` rotates each value block by
  `q(g) ⊗ u`. Specify unit-normalization, zero/near-zero behavior, icosian snap,
  exact hard-forward geometry and the actual gradient bridge.
**G1 is withdrawn by the owner as non-geometric.** Its source/proposal history
remains preserved; it is not a comparator arm or fallback. A G2 configuration
miss leads to geometric iteration under D22, never a non-geometric substitution.

Source deliverable: implemented device forward/backward, declared arithmetic and
identity/orientation behavior, build checks and static dispatch review. Any
qualification kernel executes only the lawful operator on GPU, using stored or
analytically specified expected values; no CPU reference forward or fresh banned
SwiGLU comparator. **No OP1 language fit on a partial trunk. OP2 is implemented first; OP3a
and the rest of the lawful path must exist before integrated training.**

## OP2 — SpiralCore BFS plus retained Atomic Ternary FBS projection

The owner explicitly required both supplied mechanisms together, replacing the
handwritten dense quaternion contraction too. The former `quaternion_weights=all`
implementation in #2262 was an all-input learned contraction even after GEMM
was removed. Its CPU/CUDA forward/adjoints, dense expansion and old comparator
source are [preserved inactive](../../history/branch-archive/INDEX.md).

[The replacement](../spiralcore-fbs-projection-2026-10-12/README.md) uses
`projection=spiralcore_fbs`: retained ordered ternary children and forest frontier
admit an exact 8D carrier; actual BFS edges on the SpiralCore signed-action graph
compose its transport. F32 query points select geometric roots; carrier storage
is F32/BF16. One carrier is transported per output block, with no learned
input/output coefficient grid or all-input sum. The input adjoint is exact;
the query pullback is an explicitly declared root-edge secant surrogate, not
the exact derivative of discontinuous selection. The FBS binder abstraction is
coupled to this graph by the new design; it is not the full marked FBS object
or a theorem providing semantic distance.

The owner authorized development/build/finite kernel tests for this claim on a
CUDA 12.8+ devel Ubuntu24 pod. The [prospective kernel FIT](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6137419532)
uses fixed GPU gather/sign and secant controls, complete child traversal,
F32/BF16 and loud CPU/numerical refusals. It does not permit a CPU model
reference, dense comparator, optimizer update, corpus campaign or training.
Numerical/tooling failures are blockers to repair, not mechanism negatives.

Native artifacts bind source, retained children, graph tables, BFS histories,
compiler/kernel hashes and query shapes. Old dense artifacts cannot silently
convert or execute. Native vocabulary/NoRead 2D fallthrough refuses execution;
OP3a and the common vector channel now have the finite qualification recorded
below; complete-path qualification remains a dependency. Local G2 Hamilton
rotation is retained. D11 integer/table serving remains unqualified. No
whole-model compliance or language result follows from OP2 alone.

## OP3 — geometric head source (OP3a first; OP3b later research)

- **OP3a product-cell head implemented:** `head=spiralcore_product` requires
  `projection=spiralcore_fbs`. The first/last 8D query ports score fixed SpiralCore
  atlas product cells; exact identity modulo 256 partitions address slots and
  supplies no semantic metric. Up to16 cells×32 slots are admitted, default 8
  cells/128 comparisons atV=4096. Typed IDs/masks accompany negative mean-L1 raw
  token-vector scores at fixed scale 1; ties choose smallest identity and padding
  cannot emit. Native token input, head and compiler/store/emitter channel borrow
  the same embedding Tensor/Var. The target receives explicit raw-score credit
  even when omitted; candidates remain uninjected. Token and auxiliary address
  margins have weight 1 each. Address credit is a surrogate, not the hard top-K
  derivative. Five finite CUDA controls pass under the
  [prospective FIT](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6138479221);
  [source, controls and limits](../geometric-head-shared-writer-2026-10-13/README.md).
  Artifacts bind source/compiler geometry/shape and refuse missing/stale head
  bindings; the former probability producer refuses with indexed restoration
  source preserved. Protected #2301 delivery is complete. These interfaces do not
  qualify full-trunk learning, a learned intent compiler or D11 serving.
- **OP3b murmuration fixed-address proposal:** token addresses use the
  prime-indexed H4/icosian codebook; quaternion correlation and hard/rank selection
  produce raw scores without exponential normalization. Preserve token identity,
  chirality/orientation, codebook collision handling and target credit. The full
  vocabulary score pass touches V addresses; report that scan and its cost.

Every head needs an objective over actual admitted candidates and raw scores,
explicit omitted-target treatment and GPU credit through the complete learning
path. OP3a's finite head/interface tests resolve that boundary, not the complete
trunk's credit/optimizer qualification. OP3b retains these prospective obligations. Greedy decoding uses raw-score selection.
OP3a is the owner-directed first head implementation; it must itself meet the
existing compliance/FIT requirements. OP3b is later research, not a required
pre-rung0 head comparison. Source delivery or a configuration miss retires no
fixed-address mechanism. A compliant, FIT-reviewed alternative
may be retained only after the predeclared experiment resolves its own target
and fresh transfer under D22; mechanism stops remain owner decisions.

## Whole-path admission and quality measures

After OP1–OP3 source lands, freeze the actual configuration and trace the complete
embedding, recurrence, gate, projection, memory/read, head, objective, gradient,
optimizer, output and sampling path. The recurrence-only rung has no read layer,
but still needs lawful recurrence gates, projections and head. Device qualification
must demonstrate GPU execution and loud refusal of host fallback. Existing textual
ratchets are partial instruments, not semantic certification.

The prior phrase “BPB within 0.02 nats” is withdrawn: bits/byte and nats/token are
different units, and a normalized-exponential NLL evaluator is prohibited.
Historical BPB/NLL results keep their original scope and cannot be converted into
a new objective or quality gate by renaming them. Proposed lawful diagnostics are:

| Measure | Definition and unit | What it can establish |
| --- | --- | --- |
| Raw-score top-1 | Fraction of fixed next-token targets whose raw score wins; deterministic tie rule | Prediction accuracy on that exact panel, not likelihood or conversation |
| Target rank / recall@K | Rank of the target and fraction admitted to the declared kept list | Address/read/head selection and omitted-target failures |
| Margin hinge | `mean_k max(0, 1 - z_target + z_rival)` over declared rivals; raw-score units with fixed score scale | Development objective behavior, not BPB/NLL; compare only matched definitions/scales |
| Complete replies and memory | Existing answer-membership graders, complete answer/EOS, fixed panels and later fresh transfer | Existing M1/M2 capability obligations, unchanged |
| Tokens/s | Useful tokens divided by synchronized GPU elapsed seconds, on the same harness | Speed at named shape/dtype/context/batch; not quality or energy |

No numerical replacement quality bar is frozen here. The amended registration
must pin panels, graders, actual initialization/parameters, candidate sets, score
scale, seeds, training dose, noise, mechanism-own target and distinct outcome
decisions before FIT. Decisive model comparisons use at least two seeds or 200
fresh rows, train the mechanism in, report fresh transfer for a KEEP, and do not
weaken existing milestone criteria. Where an old criterion cannot be executed
lawfully, record it BLOCKED pending prospective owner-approved amendment.

Frozen historical throughput is context only unless its harness, source, shape,
dtype, measurement boundary and useful work match. Missing matched timing is
UNAVAILABLE; do not recreate a prohibited comparator to fill it. Use lawful
matched geometric alternatives once they exist, and report total path tokens/s
beside operator timing. Preserve the original 5%/10% proposals as history, not
reviewed admission bars.

## Sequence, owners and resources

1. **Preparation:** Kimi coordinates OP2 → OP1 G2 → OP3a; Codex delivers OP2 and the ratified OP3a/shared writer claim with finite CUDA controls; Kimi recurrence source #2293 is integrated; OP3b stays later research; Codex maintains enforcement
   and M2 source dependencies; Claude retains starling/murmuration source and
   DeepSeek retains invariant-addressed router/memory source. Claims identify
   exclusive paths; no takeover, duplicate campaign or peer cleanup.
2. **Admission:** complete-path source/device qualification and amended
   prospective TEST FITNESS. Separate lawful kernel cards may resolve arithmetic
   risks; they are not model fits and lift no hold.
3. **After explicit owner hold release:** resumed compliant rung 0, followed by
   rung 0b rank-read CUDA kernels and rank-consistent gradients, then the
   margin-trained starling readout. Old dense bootstrap/teacher/SoftSort schedules
   remain inactive. M2 still needs its own hold release and qualification.

The earlier 19.93M/w512/h8/ctx384 bench and ≤$13 phase projection are retained
planning context, not admission or a new spend authorization. Changing projections
and head may change parameter count. Each amended card projects source/build,
qualification/fit/controls/evaluation/retries/storage cost on the actual design.
Pods use uor-pod, one job/GPU, existing ≤4 pods and ≤$8/h caps, no new spending
class; existing holds win. The original plan repair spent **$0 on GPU and ran no model**. The separately
authorized projection qualification carries its actual pod costs in its record.

Dtype intent: f32 masters/optimizer moments, bf16 activation storage with declared
CUDA accumulation, u16 disk tokens/u32 index tensor, TF32 off. Any precision arm
requires explicit design and device qualification; retained f64 CPU references are
source evidence only. Serving remains D11. Numerical/tooling failures are blockers;
configuration negatives preserve the mechanism, and D22 fresh-transfer/noise rules
remain. No source/policy delivery resets the capability headline or research count.
