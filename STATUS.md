# Status

**Pre-alpha, evidence checked 18 October 2026.** No model holds a useful conversation;
0/8 milestones are complete. M1's reply headline remains **37/232 SUSPENDED**;
the Dose2 Q1 diagnostic stop establishes no efficacy or fresh-transfer result.

| Evidence | What it establishes |
|---|---|
| [D33 diagnostic pilot](docs/labs/a2-pilot-2026-10-14/README.md) | 1,640 batches, 236 accepted vector updates. On 14,287 DEV targets top-1 moved 17 → 82 and top-8 173 → 1,169, while aggregate loss increased; the count reference remains ahead. |
| [Original paired Dose1](docs/labs/attention-dose1-strategic-stop-2026-10-18/README.md) | Owner-stopped strategically at 904 treatment / 1,065 frozen-zero batches, before Q1. Partials and raw exits143 preserved; custody independently accepted; own pod down 05:16:47 UTC. This was not a mechanism failure. |
| [Rows parity attempt 3](docs/labs/counterfactual-rows-parity-2026-10-18/evidence/attempt3/rows-parity.json) | 684 Route + 108 Flock + 288 Relation proposal cases, three refusals and two forced terminals: **1,085 cases**, zero updates. ~56× synchronized proposal microbenchmark at V4096/B4/C192/K8/64 rows; `timing.qualified=false`, no whole-training or quality speedup. |
| [Rows Dose2 Q1 result](docs/labs/attention-rows-q1-diagnosis-2026-10-18/README.md) and [scientific review](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6248429872) | Both arms stopped at the registered admission-collapse gate after 4,096 batches (`Q1_STOP_FOR_DIAGNOSIS`, exit0). P_succ target admission: 2,034 → 225 treatment /205 control of 3,847. These are admission counts, not accuracy or a treatment win. Configured admission collapse in both arms; no implementation or identity fault found, no registered efficacy hypothesis fires, no KEEP or mechanism-negative verdict. Fresh remains sealed. |
| Custody and publication | Pod `56510253` was released and down at 07:53:30 UTC after preserved custody. [Scorer compatibility #2652](https://github.com/UOR-Foundation/uor-r4/pull/2652) and [diagnostic publication #2653](https://github.com/UOR-Foundation/uor-r4/pull/2653) are merged. Compatibility does not supply unavailable full-dose inputs or license scoring. |

The shared vector/combine pathway is a plausible explanation for the admission
collapse, including in the relation-disabled control, but causality is not established.
No registered paired efficacy comparison is licensed at Q1.

**Immediate priority — 18 October 2026:** prepare and qualify the existing FIT'd
K16-A versus K16-R admission redirect on `rows_v1`, under the
[owner direction](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6250410397) and [PM gate audit](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6250526855).
**Actual qualification build blocker:** the selectedbccc native build passed, but the
CUDA timing build failedE0063/exit101 before model controls. Two standalone Route
Snapshot constructors now explicitly bind the existing zero-gate initialization;
repaired real CUDA builds and qualification remain owed. See the
[source/failure record](docs/labs/k16-rows-timing-source-2026-10-18/README.md).

**Source/packet preparation GO; packet acceptance and training NO-GO** pending exact-head
qualification, measured timing/budget and acceptance of the regenerated D-Packet.
Keep Route/K16, seeds 1 and 2, `CanonicalVectorThenReadV1`, the registered
16,404-batch schedule and scientific bars unchanged. Qualification requires
same-device, same-state, same-R-objective lanes/rows parity with **>16 occupied cells and an
actual R>0 row**, 25/25 controls plus `admit_ptx`, G-Pre, C-R1..C-R6,
required refusal/terminal coverage, wrong-objective/broken-negative checks and zero apply.
The [owner Q4 timing waiver6251027125](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6251027125) retains only checkpoint-0 A/R timing, **100 samples each** plus
one excluded warmup. The two Q4 cells are waived; the importer is off the critical path. [PR #2658](https://github.com/UOR-Foundation/uor-r4/pull/2658) is merged; its
source-scoped parity and five-sample diagnostic timings do not establish
new-head qualification, the registered timing or packet acceptance.
The separate K8 Relation/R successor is deferred planning, not an added K16 gate.
The existing conditional release preserves qualification → accepted packet →
training → fresh before KEEP; fresh execution is not a training-start gate.
Other training/M2/Stage B holds remain.
The [explicit rows timing adapter](docs/labs/k16-rows-timing-source-2026-10-18/README.md)
binds the production path and rows image identities at source level. Its new-head
CUDA/timing execution is NOT_RUN by this delivery. The owner-waived Q4
importer is preserved as preparation, without blocking qualification.
The [architecture](docs/architecture.md) traces current computation to source;
[current-state](docs/integration/current-state.md) retains the chronological evidence.

[D37](docs/integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)
permits offline floating point, retains matmul/GEMM and softmax/squashing prohibitions,
and requires D11 integer export qualification. The active discrete model remains integer;
model/reference computation is GPU-only.

## Earlier source-scoped status records

The dated entries below retain their original qualification and release scope.
Current dependencies and dispatch status are summarized above.

**Current result — D32/D33, 14 October:** the admitted seed-1 discrete pilot
completed all 1,640 batches on CUDA, with 236 accepted shared-vector updates.
On 14,287 pinned DEV targets, top-1 correct moved 17 → 82 and top-8 173 → 1,169;
the aggregate loss numerator increased 13,816,032 → 13,884,032. The masked-TRAIN
count reference remains ahead at 1,226 / 4,060. This is a mixed diagnostic
observation, not useful-language qualification or a mechanism verdict. Complete
artifacts are preserved and the pod is deleted. The pilot release is consumed;
all further training, seed 2 and M2 remain HELD pending a prospective sheet and
scoped release. [Result, scope, costs and next action](docs/labs/a2-pilot-2026-10-14/README.md).

**Completion preparation:** [D5 and the paired diagnosis](docs/labs/a2-pilot-diagnosis-2026-10-14/README.md)
preserve the pilot while adding separate owner-release scopes, driver-only lineage
migration and checked quarter-evaluation recovery. Local Rust metadata controls
pass; the new CUDA build/runtime remains UNVERIFIED. Kimi's completion-sheet
amendment and Claude's FIT/exact-source review precede a fresh owner decision.
No initialization, objective, schedule or model compute changed in this preparation.

**Construction/cost baseline before that pilot:** the discrete shared-codebook SpiralCore/FBS path
with the discrete flock read and D1 driver passes all twenty-five admitted CUDA
construction controls at `e7d926fbe38537f4faa90b817705c61a780778d6`, after fresh
non-author Codex FIT (#2400): eleven Route, six flock and eight driver controls.
The previous seventeen-control result at `76198c3b5` keeps its source scope and
Claude FIT attribution; #2379's earlier observations keep their review-order labels.
Exactly eleven authored fixture apply calls produced eight disposable updates,
with zero corpus updates. D2 operation-cost measurement **PASS**, with30/30 timed
API calls and12/12 proposal samples, no applied update. At V4096/B4/C192/64 rows,
median Route/Flock forward takes37.84/40.58ms; vector proposals take3.90/4.21s.
Counterfactual serial replay is the cost bottleneck; fixed-batch parameter-pass
projections of17.8/19.2hours remain **UNVERIFIED**, not corpus-training throughput.
[Raw cost receipt](docs/labs/discrete-driver-cost-2026-10-13/receipts/measurement.json).
Before admission delivery #2421, corpus updates stayed behind an unissued opaque
capability. The generic training CLI still returns `TrainingHeld`; D33 admitted
only the now-completed pilot. M1's quality target remains SUSPENDED. No floating-point model, gradient/optimizer or numerical
qualification instrument may execute on the discrete path. D37 (18 October) permits offline float
training only through a new FIT'd pre-registration with a D11 integer export, and releases no hold. Earlier 31/18-control float results below
remain historical source-scoped evidence, preserved inactive through #2370.
Outside the completed pilot, training holds remain HELD; no chat-quality result is claimed.
[Discrete flock/writer record](docs/labs/discrete-flock-writer-integration-2026-10-13/README.md).

Snapshot of 14 October 2026. **Pre-alpha: no model holds a useful conversation yet.**
Broad reasoning, coding, frontier capability and complete-path energy savings
remain unestablished. Results/artifact identities are in [current state](docs/integration/current-state.md);
[ROADMAP](ROADMAP.md) owns milestones and [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028)
owns live claims/status. The [earlier status snapshot](https://github.com/UOR-Foundation/uor-r4/blob/aaa210ef46301623f10164ae5af586d7f558483a/STATUS.md)
and its linked experiment records retain all historical readings.

## Historical operator-phase dependency and hold (12–14 October)

**Historical pre-D32 compiled boundary (inactive float source scope):** the native-only source cut, corrected and frozen at `1f1547bfd70f01edc9aa36d34a46e5bc43edc34c`, passes CUDA compilation and all 31 exact finite controls on RTX4090, nvcc12.8.93, compute_89, Rust1.97.1. It excludes historical core/integer model executors, teachers, LUT engines and exporters; `geometric-stack` now selects the native train/evaluate CLI and lineage/optimizer admission refusals. The vendor PTX export/instruction census, selected dependency/features, four executable import checks and five project interlocks pass, with exactly three project PTX modules. Earlier setup/driver/controller and clipping failures are preserved as corrected blockers. Protected delivery is pending; no optimizer update/corpus/training/CPU model ran. Earlier 18/11-control results below retain their original source scope. [D29 record](docs/labs/d29-vendor-boundary-2026-10-13/README.md). Independent Amendment6 FIT and the separate finite FIT sheet release no hold.

**Training remains held; M2 training/rental remains held separately.** Kimi is
project manager under [D25](docs/integration/DECISIONS.md#d25---geometric-operator-phase-and-current-lab-direction).
No softmax/softplus/sigmoid/tanh or equivalent squashing anywhere in an executed
model/objective/gradient/output/teacher/comparator path; no dense matrix
multiplication in training or serving; model/reference compute is GPU-only with
host fallback refused. D32 excluded floating-point learning and numerical
model instruments; D37 re-admits them for offline training only. D11 retains its multiplier-free serving contract. The predecessor declared recurrence-only path has finite CUDA forward/backward/output
qualification at its original rung-0 source/shape; it does not qualify the new compiled boundary. The optimizer update is compiled and
source-reviewed, not executed. Amendment5 supersedes the whole-DEV count-reference scope and has independent
scientific [Claude CONFIRM FIT6146716704](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6146716704) on merged source #2328. The physical
sheet verdict is **FIT** under Kimi's Amendment6 mechanical TRAIN-mask/checksum/CLI pins
and independent Claude confirmation, at the declared prior source scope. Explicit owner release
still remains required.
The owner clarified that both model lines share one directly learned native
geometric vector representation and writer, using E8/SpiralCore/icosian/FSB
plus flock/murmuration attention. Total block count is unqualified; the historical
MiniLM 384D teacher width is not a dependency. The [signed-axis source repair](docs/labs/native-vector-transport-2026-10-12/README.md)
is static geometry construction, not unified-writer/model qualification.

The [operator phase](docs/labs/operator-phase-2026-10-12/README.md) is source first:
OP2 SpiralCore/FBS projection → OP1 G2 gate → OP3a geometric head → complete-path
source/device qualification and prospective FIT. Training additionally needs
explicit release of the applicable owner hold. No partial-trunk OP1 fit, new
prohibited comparator or CPU model reference; no FIT or source delivery releases
a hold. The [SpiralCore/FBS projection](docs/labs/spiralcore-fbs-projection-2026-10-12/README.md)
replaces the dense quaternion contraction, with owner-authorized bounded pod
build/kernel qualification; training remains held. The [OP3a product head and shared native vector writer](docs/labs/geometric-head-shared-writer-2026-10-13/README.md)
now use the same embedding Tensor/Var across token input, raw-distance output and
the compiler/store/emitter vector channel; five finite CUDA controls pass.
The predecessor's head-only tests did not qualify recurrence. The earlier composed
successor has finite forward/backward/output qualification at its named source below; trained language,
learned intent/compiler use and D11 serving remain unqualified. The head/writer unit delivered as #2301, merge `d41dc422f`.
[Complete-path repairs](docs/labs/rung0-complete-path-2026-10-13/README.md) are
implemented and finite CUDA-qualified at source `05c84bece50789163bd0f049a2ae76b8a314ef45`;
all 18 predecessor controls pass. Protected #2318 merge is verified at
`5b22739a31eafb131d25e098bb7f02ab803d4fe7`. The new
[masked TRAIN count-reference repair](docs/labs/count-reference-2026-10-13/README.md)
is implemented at source `2fd6358bbc52d2a202a872ad0954ebb12894606f`; 11 new finite controls PASS (9 metadata,2 CUDA); protected [PR #2328](https://github.com/UOR-Foundation/uor-r4/pull/2328) merged at `9e904a83448c451f738d5c1d45224dfdf73eb84e`. Amendment5 has independent [Claude CONFIRM FIT6146716704](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6146716704), with unchanged bars;
Kimi's Amendment6 records the mechanical pins and restores the physical sheet's FIT verdict.
Its DEV response oracle is diagnostic only. Training remains HELD.
The rank-margin objective is source, not a language result. The active sheet
pins the existing rank/anti-vacuity bars, units and paired seeds; tokens/s uses
the same synchronized CUDA harness. No model training or quality run occurred.

## Measured positions and next source dependencies

| Line | Retained reading and limitation | Current next dependency |
| --- | --- | --- |
| M1 language base | Historical v5 memory20/40, reply37/232 (targets34/40 and116/232); historical19.9M served0.886838 BPB at512 windows ([protocol](docs/labs/criterion2-protocol-pin-2026-10-09/README.md)). No new quality result | D32 [decomposition](docs/labs/counterfactual-decomposition-run-2026-10-16/README.md) resolves writer/baseline cost in four cells; [baseline reuse](docs/labs/counterfactual-baseline-reuse-implementation-2026-10-16/README.md) is implemented and [finite-qualified on580.95.05](docs/labs/counterfactual-baseline-reuse-finite-pass-2026-10-17/README.md). After the [timing startup refusal](docs/labs/counterfactual-baseline-reuse-timing-startup-2026-10-17/README.md) and protected [exact-driver selector](docs/labs/driver-aware-pod-selection-2026-10-17/README.md), a separately reviewed/released [finite qualification on595.71.05](docs/labs/counterfactual-baseline-reuse-driver-595-2026-10-17/README.md) passes32 exact U/R pairs,16 typed refusals and18 strict resource admissions including R shared512, unchanged state/epoch and apply0. Archive verified before teardown; fleet0. Finite record #2568 is protected and cleaned. A separately released [timing2 attempt](docs/labs/counterfactual-baseline-reuse-timing-uuid-blocker-2026-10-17/README.md) refused a changed GPU UUID before any binary; archive-copy recovery was verified before teardown. Next: protect the setup record and prospectively resolve device admission before a new release. Speedup, checkpoint0 parity and training remain unmeasured; holds remain |
| M2 grounded reply | Saved437/512 development + opened0/128 transfer diagnostic remain panel fitting; genuinely fresh qualification NOT_RUN. [Whole-path recovery source](docs/labs/whole-path-recovery-2026-10-12/README.md) delivered; CUDA execution unqualified; sampled law refuses. Head/shared vector channel and the repaired compiled boundary have finite CUDA qualification at `1f1547bfd`; full M2 remains unqualified. Unified line count12/3 after masked-reference source delivery #2328 (previous11/3 retained; closeout is the same unit), no reset | Masked TRAIN-reference repair delivered in #2328; Claude CONFIRM FIT recorded; D29 finite qualification passes; protected delivery remains; coherent full-M2 intent/compiler path and explicit owner hold release remain dependencies |
| M3 durable memory | Evaluator/session delivered; unqualified | Preserve interfaces; no automatic experiment when M2 is held |
| M4 one served model | Historical D11 bit-exact artifacts and rank-read components retained; historical artifacts include prohibited paths; their model executors are excluded from the selected build. [Starling bootstrap](docs/labs/starling-bootstrap-2026-10-12/README.md) is inactive evidence | Claude discrete murmuration read against shared integer carrier; reviewed finite qualification, then owner-released discrete learning; no float gradient path |

Historical BPB/NLL is preserved at its original scope and cannot authorize a new
normalized-exponential evaluator. Raw-score/answer quality and speed are separate;
missing measurements are NOT_RUN/UNAVAILABLE, not model negatives. No milestone,
headline, acceptance criterion or mechanism retirement is changed here.

## Compute and labs

Kimi coordinates the single plan; Claude, DeepSeek and Codex retain their assigned
source. Claims go on #2029–#2036, leases on [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037),
pods through uor-pod within ≤4 pods/≤$8/h and all applicable holds. Local source,
build, static checks, Rust preparation and grading stored outputs are non-model
work; CPU model execution is prohibited. Protected delivery needs actual
exact-head checks/review; queue acknowledgements execute no tests. Preserve all
peer work, source and evidence; remove only verified redundant owned material.
