# UOR-R4 Geometric Language Model - roadmap

**Current learning permission — D32:** discrete/exact GPU model computation and
instruments only; no floating-point forward, gradient, optimizer or matrix product.
The shared-writer FBS path and discrete flock/driver construction pass 25 admitted
CUDA controls at `e7d926fbe` (#2400). D1/D2 and the A2 admission controller (#2421)
landed; the D33-released seed-1 pilot completed 1,640 batches and 236 accepted
vector updates. DEV top-1/top-8 counts increased 17/173 → 82/1,169 over 14,287
targets, while aggregate loss increased. This is diagnostic only; the next
dependency is Kimi/Claude's prospective successor-sheet review, not another
launch. [Result and preserved artifacts](docs/labs/a2-pilot-2026-10-14/README.md).
M1's quality target remains SUSPENDED pending
a discriminating grader. Earlier operator/31/18-control float entries
are preserved historical source scopes, not executable training permissions
(preservation #2370). The completed pilot's release is consumed; further training
and separate M2 holds remain HELD. Kimi owns the
discrete successor sheet; a FIT review releases no hold.
[Discrete flock/writer record](docs/labs/discrete-flock-writer-integration-2026-10-13/README.md).

Updated 14 October 2026. **Pre-alpha.** Useful conversation, broad reasoning, coding, frontier
capability and lower complete-path energy are not established.

Milestones M1-M8 below, with the standing serving rules in section 0. Live status and claims:
tracker [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028); each milestone is its own issue. Results:
[current state](docs/integration/current-state.md). Owner decisions:
[DECISIONS](docs/integration/DECISIONS.md). Long-form plan and history:
[canonical plan](docs/integration/project-track.md). The 29 September roadmap is preserved in
[docs/history/roadmap-2026-09-29-snapshot.md](docs/history/roadmap-2026-09-29-snapshot.md).

## Earlier operator qualification (pre-D32, inactive)

**D29 compiled-source dependency:** corrected native-only source `1f1547bfd70f01edc9aa36d34a46e5bc43edc34c` passes the CUDA build and all 31 exact finite controls on RTX4090, nvcc12.8.93, compute_89, Rust1.97.1. The registered `geometric-stack` example uses the new native train/evaluate path. Historical core/integer model executors, model-source/GEMM, retired LUT engines and exporter/teacher modules are excluded. Static vendor PTX/dependency/import checks and all five project interlocks pass; exactly three project PTX modules remain. Failed driver, registration/controller and clipping attempts are preserved as corrected blockers. Earlier qualification receipts below remain source-scoped history. Independent Amendment6 FIT is recorded; protected delivery and explicit owner hold release remain. No optimizer update, corpus/training/CPU model execution or S1 read activation occurred. [Record](docs/labs/d29-vendor-boundary-2026-10-13/README.md).

[D25](docs/integration/DECISIONS.md#d25---geometric-operator-phase-and-current-lab-direction):
Kimi is project manager; no softmax/softplus/sigmoid/tanh or equivalent squashing,
no dense matrix multiplication in training/serving, GPU-only model/reference
compute and no host fallback. **Training and M2 training/rental remain held.**
[OP2 SpiralCore/FBS projection → OP1 G2 gate → OP3a geometric head](docs/labs/operator-phase-2026-10-12/README.md)
are source dependencies, followed by complete-path/device qualification, amended
prospective FIT and explicit release of the applicable owner hold before training.
No partial-trunk fit or fresh prohibited comparator. Rung0 is suspended; rung0b
follows the operator phase/resumed rung0, then teacher-free starling.
Both lines must learn/use one native vector representation and writer under the
owner's current clarification (E8/SpiralCore/icosian/FSB and flock/murmuration).
Existing 8D structural blocks do not freeze total width or import a MiniLM 384D
dependency. The [OP3a/shared writer](docs/labs/geometric-head-shared-writer-2026-10-13/README.md)
uses one learned table across Stack and compiler/store/emitter adapters; finite
CUDA controls pass and protected #2301 delivery is verified at `d41dc422f`.
The [complete-path repair](docs/labs/rung0-complete-path-2026-10-13/README.md)
implements ordinary recurrence dispatch, rate credit, CLI/row-cap/dev-metric
repairs at `05c84bece50789163bd0f049a2ae76b8a314ef45`;18 finite controls pass at the
actual rung-0 shape, delivered in protected #2318 merge
`5b22739a31eafb131d25e098bb7f02ab803d4fe7`. The new
[masked TRAIN count-reference repair](docs/labs/count-reference-2026-10-13/README.md)
is implemented at frozen source `2fd6358bbc52d2a202a872ad0954ebb12894606f`;
11 new finite controls PASS (9 metadata,2 CUDA); protected [PR #2328](https://github.com/UOR-Foundation/uor-r4/pull/2328) merged at `9e904a83448c451f738d5c1d45224dfdf73eb84e`. Amendment5
has independent scientific [Claude CONFIRM FIT6146716704](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6146716704), with unchanged bars.
Kimi's Amendment6 supplies the mechanical TRAIN-mask/SHA/CLI pins and restores
the physical sheet to independent **FIT**, scoped to the declared prior source. The DEV response oracle is diagnostic only.
Training/M2 holds remain HELD.
Complete model use, capacity,
intent compilation and D11 serving remain unqualified.

## Milestones

Two lines meet at M4: line A (language base, M1) and line B (grounded reply from exact memory,
M2 and M3). Thresholds marked PROPOSED on M1, M2 and M4 await owner confirmation; a milestone
closes only when its acceptance is met on the saved model.

| | Milestone | Status | Goal and acceptance summary | Latest result |
|---|---|---|---|---|
| M1 | [Language base](https://github.com/UOR-Foundation/uor-r4/issues/2029) (Kimi, DeepSeek; Claude read source) | diagnostic pilot complete; counterfactual finite controls PASS, Stage2 NOT_RELEASED; training held | Reply headline SUSPENDED/no target under R14; successor learning needs a prospective sheet and scoped release | [D33 pilot](docs/labs/a2-pilot-2026-10-14/README.md): 1,640 batches, 236 accepted vector updates; DEV top-1/top-8 17/173 → 82/1,169 over 14,287 targets, aggregate loss increased, count reference still ahead. Mixed diagnostic observation; no mechanism verdict or useful-language qualification. Route sensitivity NOT_RUN; fresh transfer UNAVAILABLE/NOT_RUN. Earlier operator/F32 and historical memory20/40, reply37/232 results retain their dated records and inactive scope. |
| M2 | [Grounded reply from exact memory](https://github.com/UOR-Foundation/uor-r4/issues/2030) (Codex) | in progress; training/rental held | Saved≥256/512 AND genuinely fresh≥40%, existing panels/graders unchanged | Saved437/512 + opened0/128 remain panel fitting; fresh NOT_RUN. Whole-path recovery source delivered; sampled law refuses; shared vector infrastructure finite CUDA-qualified, full M2 unqualified, count12/3 after masked TRAIN-reference #2328 delivery (prior11/3 retained; closeout is the same unit), no reset; new evaluator qualification11 controls PASS (9 metadata,2 CUDA) and Amendment5 independentCONFIRMFIT; Amendment6 sheetFIT; repaired compiled boundary 31 controls PASS at `1f1547bfd`; protected delivery pending |
| M3 | [Durable conversation memory](https://github.com/UOR-Foundation/uor-r4/issues/2031) | in progress | Memory persists across sessions and answers stay grounded | evaluator and session delivered; not qualified |
| M4 | [One served model, D11 and CLI](https://github.com/UOR-Foundation/uor-r4/issues/2032) (Claude read source) | in progress; compute blocked | One model under D11 and full D23–D25 compliance; old runtime-only/metric clauses require prospective owner amendment | Historical artifacts/components retained inactive; model/stack executors are excluded from the selected integer facade. No old SoftSort/dense-bootstrap/teacher follow-up; compliant trunk/rung0b precede teacher-free starling |
| M5 | [Laptop cost, D5](https://github.com/UOR-Foundation/uor-r4/issues/2033) | not started | Measured J/token, RSS and tokens/s on the M1 with selected parameter access | none |
| M6 | [Reasoning and coding](https://github.com/UOR-Foundation/uor-r4/issues/2034) | not started | Executable reasoning and coding on the same model | exact arithmetic is the visible gap |
| M7 | [API, WASM, Pages Studio](https://github.com/UOR-Foundation/uor-r4/issues/2035) | not started | The model reachable through API, WASM and the Studio | local API only |
| M8 | [Alpha release](https://github.com/UOR-Foundation/uor-r4/issues/2036) | not started | Owner-declared alpha | none |

Standing direction (owner, 3 October): no transformer baselines; mechanisms are compared with matched geometric alternatives and with the previous best geometric model.

Standing issues: compute board [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037); open bugs [#1542](https://github.com/UOR-Foundation/uor-r4/issues/1542), [#1476](https://github.com/UOR-Foundation/uor-r4/issues/1476),
[#1718](https://github.com/UOR-Foundation/uor-r4/issues/1718), [#1738](https://github.com/UOR-Foundation/uor-r4/issues/1738).

## Retained experiment evidence and proposals

| Experiment | Milestone | Status | Question |
| --- | --- | --- | --- |
| Native VSA retraining (4 arms × 2 seeds) | M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) | **done: KEEP** ([record](docs/labs/vsa-native-test-2026-10-09/README.md)) | Trained VSA (fixed codes) improves held-out BPB by 0.014–0.023. Icosian-root codes don't: they collapse token identity. Mode 2 (root + per-token residual) is worse than fixed codes on all 4 cells: not KEEP |
| Softmax-free reads: soft (A), flock rank (B), B + prime-route copy (C), Hamming-rank (D), learned rank (L, W), SoftSort (S) | M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) | inactive historical evidence; no rerun under D23–D25 | Can served reads drop the table-emulated softmax with no loss? Not yet: the best is +0.017 BPB |
| Ring-product read (Z/256 byte ring; quaternion/icosian arm) | M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) | retained proposal; no compute admission | Can a read scored and admitted by exact ring arithmetic, with no softmax and no multiplier, match the softmax read? |
| Spin-native model: quaternion and icosian weights, starling read, E8 orbit memory ([design](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6104974064)) | M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) | inactive historical evidence; prior rung 0 KEEP; exact weights at dense quality (KEEP); exact activations post-hoc +0.020, trained-in no better (historical proposed successor, held); starling: a training-time softmax surrogate teaches content ranking; starling read + base-3 tree + dense bootstrap: 3/5 seeds match softmax at 1536 (0.96–0.999), no softmax at runtime (historical proposed successor, held); E8 rung 1: half the MLPs +0.033 BPB at ~10× fewer ops, but learned keys beat E8 addresses (historical proposed successor, held) | Can SpiralCore's finite spin geometry (exact icosian table products, 600-cell neighbours, E8 orbits) replace the dense matrices, the softmax read and the MLPs at much lower compute per token? |
| Route-holonomy read | M1 #2029 | earlier registration held; amend under D25 before FIT | Can the angle of h_j⁻¹·h_t rank earlier positions, order-aware and softmax-free? |
| Exact icosian holonomy lanes (E1) | M1 #2029 | earlier registration held; amend under D25 before FIT | Does an exact 2I group product beside the r-layer help, beyond a shuffled-geometry control? |
| Octonion-signed binding, then transport | M1 #2029 | earlier registration held; amend under D25 before FIT | Does a Fano-signed XOR keep order and grouping that plain XOR loses? |
| Shared `BitCode` primitive | M4 #2032 | planned (engineering) | One Hamming/popcount type for the native learner, R4G1 and the integer engine |

All earlier run schedules in this table are inactive; their metrics and source remain
preserved at the original scope. None overrides the operator dependency or hold.

Pictures of the pre-registered mechanisms are in [docs/geometry.md § 11](docs/geometry.md#11-pre-registered-mechanisms-not-yet-measured); none of them has a measured result yet.

## 0. Mission and hard runtime rules

**North star: geometric intelligence.** The goal is a working geometric language model whose
runtime uses integer, fixed-point and geometric operations only: no transformer, no matrix
multiplication and no floating point. D32 extends the discrete requirement to
learning and numerical model instruments: no float forward, gradients or optimizer.
D25 prohibits matrix multiplication and exponential-family squashing;
D24 prohibits CPU model compute. Non-model
preparation/analysis/tooling may run locally. Training remains held.

The owner reaffirmed this target on 2026-09-27: "Keep the native, multiplier-free serving
target" ([#820](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5852795279)).

The rules below apply to every lab. A serving change that breaks any of R1–R4 does not enter
the mission path.

| Rule | Meaning | Evidence required |
|---|---|---|
| **R1: no float** | No f32/f64 instruction and no libm call in any served symbol. | An instruction audit of the release binary (`scripts/serving_multiplier_check.py`, `scripts/audit_zero_matmul_serving.py`). |
| **R2: no multiplier** | No integer multiply or divide instruction in served kernels ([D0-b](docs/integration/DECISIONS.md#d0-b--what-no-matmul-at-serving-means-adopted)). Products of runtime values use product or quarter-square tables, or exact geometric structure (signed permutations, ℤ[φ] shift-add). D10's hardware-multiplier exception is not adopted by any lab. | The same audit. |
| **R3: no matmul** | End state: no dense per-token access to a learned weight store ([D5](docs/integration/DECISIONS.md#d5--per-token-parameter-sparsity-is-the-terminal-serving-invariant)). D25 supersedes the earlier dense-map stepping stone: geometric/addressed operators are required in training and serving; a handwritten dense contraction is not a substitute. Preserve old maps as inactive evidence. | Per-token parameter reads in every serving report. |
| **R4: no transformer** | No served backbone whose primary mechanism is stacked dense all-pairs attention plus MLP. A model counts as a transformer when most of its token-mixing layers are dense all-pairs reads. Converted open-weight transformers (D10's SmolLM2) remain inactive historical evidence; D23–D25 prohibit new comparator/teacher execution through banned operations. Recurrence-primary hybrids (most token mixing recurrent) are interim, and their reads move to bounded geometric or exact addressing (D5/D6). | An architecture statement in the PR. |
| **R5: honest cost** | An integer path is not an efficiency result. Energy may be claimed only when measured on the M1 in J/token. Bytes touched per token set the energy floor, not multiplier count ([first-principles review §5](docs/integration/first-principles-review-2026-09-25.md)). | Measured J/token, RSS and tokens/s. |
