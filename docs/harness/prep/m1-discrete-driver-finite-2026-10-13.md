# Pre-registration — finite discrete driver and cost instrument

- Date: 2026-10-13. Milestone: M1 #2029, learning/output interface #2032.
- Scope: D1 driver/evaluator/update-boundary construction and D2 measured operation cost. This is not the corpus-training sheet.
- Executable source: `e7d926fbe38537f4faa90b817705c61a780778d6`, crate tree `218b2e7bc2a5e41332f10dcba666b10275046ef6`, [PR #2400](https://github.com/UOR-Foundation/uor-r4/pull/2400), based on main `1ef7ef415`. Release binary/PTX/geometry hashes are frozen below before execution. No prior executable qualification transfers to the new evaluator or apply controls. Later documentation-only pins do not change this executable source.
- Brief: [discrete geometric learning](../../mechanisms/discrete-geometric-learning.md).
- Authority: D23–D32 and R12–R14. Integer-only GPU model and reference arithmetic; no normalized exponential, dense contraction, floating model/instrument, CPU model/reference or NVRTC.
- Training and M2 remain **HELD**. Owner-directed D1/D2 development includes the disposable finite transition qualification below, subject to prospective exact-head independent FIT and root's scoped pod admission. FIT alone releases no owner hold. No corpus, optimizer, language evaluation or trained checkpoint is admitted.

[Claude's prospective design review6157700698](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6157700698) states the hold interpretation:

> My reading: `apply` executed only on authored, disposable literal fixtures counts as **construction qualification of the transition mechanism, not training**, under Casey's explicit "finish D1/D2 ... qualify driver ... while training HELD". It stays that only if the sheet pins all four of these:
> - (a) no corpus or UORT data enters any `apply` call;
> - (b) fixture models are created and dropped in-process, so no snapshot after an apply is saved, archived or promoted;
> - (c) the number of apply calls is literal and bounded (counted in the report as `applied_updates: N`, not 0);
> - (d) the train CLI keeps returning `TrainingHeld`, and **no code path reachable from the CLI** calls the transition loop on corpus windows. A test should assert that the loop entry point is reachable only through the qualification controller or its fixtures.

This sheet adopts those four boundaries. Corpus transition requires a private, unissued `CorpusUpdateAdmission`; `request_corpus_update_admission` and training admission must return `TrainingHeld`. Only the qualification controller applies the authored literal fixtures. In-memory snapshots used below are dropped in-process and never written to the result archive.

## Base artifact

Release executable built from source `e7d926fbe38537f4faa90b817705c61a780778d6`:
SHA256 `399ab42713589439dd963ec50515133e723abda79cd4b1d90330d9710d80c117`;
PTX `f864c085e3054df9254646a694e73fc0e582162a11a179324df10236491246b5`;
geometry `3c7a88bfc82e0fc2f4031ec35cfd69516596dce385b792767675ea937763cc50`.
Build PASS: Cargo release16.06s, controller17s, Rust1.97.1, nvcc12.8.93 with
`-ptx -std=c++17 -arch=compute_120`; RTX5090 driver580.178.04, Ubuntu24.04.1.
The AOT checker admits exactly11 entries and rejects prohibited floating/tensor
instructions. Model initialization is seeded; no learned checkpoint is inherited.

## Data split

Finite authored literal fixtures and fixed synthetic cost windows only. No real
TRAIN/DEV/UORT input enters an apply. The small UORT bytes in masking/resume controls
test metadata and count-reference boundaries; they do not train a corpus model.
The separate full-corpus metadata preparation in the canonical record is not a
numerical model run and does not enter this finite controller.

## Exact finite scope and bar

Execute **25 controls** once at the frozen source: eleven unchanged Route regressions, six unchanged Flock regressions and all eight new `driver_qualification::NAMES` controls below. Require exact witnesses, zero skipped/ignored/NOT_RUN and complete independent result records. The names must agree exactly with the compiled source lists before FIT. A generic CUDA failure does not satisfy a domain-refusal witness. A CUDA fault hard-stops further launches and preserves the remaining inventory as NOT_RUN.

The existing seventeen controls retain all bars from the [flock construction sheet](m4-discrete-flock-construction-2026-10-13.md) and each suite reports applied_updates0. They must rerun on this executable because the shared writer/kernel ABI changed; old PASS receipts do not substitute. Route inventory: `exact_writer`, `virtual_proposals`, `context_transport`, `isolated_windows`, `collision_and_omitted_target`, `nonvacuity`, `artifact_and_identity`, `inventory_bounds`, `variable_length_prefixes`, `target_independent_admission`, `token_relabelling_preserves_geometry`. Flock inventory: `flock_identity_read`, `flock_gate_counterfactual`, `flock_transport_witnesses`, `flock_payload_witness`, `flock_selection_and_eligibility`, `flock_refusals_and_noop`. `list-controls`, `list-flock-controls` and `list-driver-controls` must match eleven/six/eight respectively; the numerical commands are `qualify`, `qualify-flock`, `qualify-driver` and separately `measure-cost`, each using an exclusive report directory.

| Control | Literal observation |
| --- | --- |
| `isolated_apply_boundary` | Route mode, V2/B2/C1/kept2; token0 `[a,a]`, token1 `[-a,-a]`, query token1, target0. VectorBlock0 proposes ordinal7, 640→544, epoch0. One device application changes only its first8 coordinates to `[-2,2,0,0,0,0,0,0]`, advances epoch to1, and fresh carry scores544. Foreign proposal refuses without mutation; stale proposal and stale carry refuse exactly. All16 one-leaf Route candidate costs are544; CURRENT tie returns `Ok(false)`, with identical snapshot and epoch1. |
| `route_apply_reality` | V2/B2/C2/kept2, context `[-a,-a]`, target token1. Route0 ordinal1/B12 takes640→256; routes become `[1,0]`, epoch1, unchanged table/gates. Recarry gives literal output `[-a,a]` and exact256 cost. |
| `gate_apply_reality` | Flock K1/V2/B2/C2/kept2, token0 `[a,b]`, token1 `[a,a]`, context `[1,0]`, target1. Gate1 ordinal2 takes448→64; gates `[0,1]`, epoch1, unchanged table/routes. Recarry gives `[a,a]` and exact64 cost. GPU evaluation gives token hinge8, address hinge0, strict token win count1 but token-unviolated count0, distinguishing prediction from margin satisfaction. |
| `fixture_loop_resume` | Two independent disposable models run VectorBlock0 then1, each ordinal7, literal costs640→544→448 and epochs1→2. A third model restores the first model's epoch1 snapshot **in memory** and repeats only the second step. Receipt sequences and final table/routes/gates agree exactly; no learned artifact leaves the process. Five improving apply calls in this control. |
| `native_top8_ties` | V10/B2/C1, all vectors `[a,a]`, kept1. Both rows' writer top8 is `[0,1,2,3,4,5,6,7]`; targets7/8 discriminate recall1/0, top1 both0. Each token hinge16, address hinge0, numerator128, token rivals9, address rivals0, one admitted cell. GPU totals: rows2, top1 hits0, top8 hits1, token hinge32, numerator256, token-eligible/any-eligible rows2, address-eligible0; all eligibility-qualified unviolated counts0, denominator per row128. A separate absent-rival row reports zero eligibility, zero unviolated-success counts and zero penalty; proposing on it refuses exact status8. |
| `count_reference_excludes_dev` | Metadata admits authored TRAIN `[0,9,9,3,3,4,0,9,4]` with mask `[0,0,0,1,1,1,0,0,1]`; actual selected targets are `[3,3,4,4]`. Unmasked9→8 does not change selected targets. GPU TRAIN counts are `[0,0,0,2,2,0,0,0,0,0]`, top8 `[3,4,0,1,2,5,6,7]`. DEV `[9,9,9,4]` gives primary top1 hits0/top8 hits1/zero-count targets3. DEV oracle separately counts9:3/4:1, top1 hits3/top8 hits4 and is explicitly diagnostic only. Changing DEV9→8 leaves TRAIN counts/top8 unchanged. |
| `seeded_initialization` | CUDA seed1 repeated on V10/B2/C3/kept2 gives identical table/routes/gates with distinct process identities; seed2 binds seed2 and a different initialized table. A foreign same-seed carried batch refuses. No CPU initializer/reference is executed. |
| `corpus_schedule_resume` | Authored TRAIN above/context3/batch3: positions `[3,4,5]` give windows `[[0,9,9],[9,9,3],[9,3,3]]`; the tail position8 gives `[[0,9]]`, record1, target4, with no BOS crossing. All-masked input refuses positive eligible inventory. V10/B2/C3 schedule has24 coordinates: VectorBlock0..19 then Route0..3 (or Gate0..3 in Flock); paired seeds have identical order/windows. After24 no-update metadata advances, coordinate0, corpus-pass12, target0. In-memory cursor resume preserves the tail; changed seed/mask lineage and wrong coordinate refuse. Actual epoch0 `PreparedDriver::checkpoint/restore` round-trips in memory and matches original/restored GPU evaluation on that authored tail; no apply occurs on those windows. Corpus update capability/training admission remain HELD. Mask/window/schedule arithmetic is metadata only. |

Literal geometry: `a=[2,2,0,0,0,0,0,0]`, `-a=[-2,-2,0,0,0,0,0,0]`, `b=[0,0,2,0,2,0,0,0]`; the sixteen-root anchor orbit and64 signed actions remain the bounded substrate, not full E8 learning. Fixture applications are separate from cost-shape measurement. There is no forged private proposal and no invented non-improvement refusal: the public current/no-op application must return false.

The report counts every actual apply call: **eleven calls**, comprising eight improving updates, one refused foreign call, one refused stale call and one equal-loss no-op. A complete successful suite expects **eight isolated updates, zero corpus updates**. A CUDA fault preserves observed counts and marks them incomplete rather than claiming a verified transition outcome. Training admission continues to refuse. `qualify-driver` is the separate controller command; source/module/CLI inventory must agree before execution.

## D2 bounded cost observation

Run the separate `measurement::run` command once, creating its output directory exclusively before CUDA. Use a **release-profile** executable built AOT with `UOR_NVCC_ARCH=compute_120`; record the actual nvcc command/default optimization, compiler, device, driver and executable/PTX hashes. Compilation is not a timing result.

The new AOT inventory has eleven exact entries: the prior seven plus `discrete_evaluate`, `discrete_evaluation_aggregate`, `discrete_count` and `discrete_count_score`. Bind and admit the actual integer-only emitted PTX at the final source; the old seven-entry receipt is not the new admission.

Shape: vocabulary4096, four8D blocks (width32), context192, writer kept cells8, rows64, fixed seed7. Measure **Route** and **active Flock K8** separately. Active Flock is an authored seeded-codebook fixture with all gate trits+1, established before any apply; all-zero gates would skip ranking and would not price active attention. This is a cost shape, not a selected learning arm or language panel. Fixed synthetic Identity windows only; no private corpus is downloaded or scored for this slice.

Each mode loads/initializes first, then performs one warm-up carry/proposal/evaluation of every measured entry before the **three** timed repetitions. Context setup/memory query, combined module-load/JIT/initialization and first-use warm-up durations are reported separately; there is no false claim to isolate only the driver's internal JIT time.

Three repetitions each measure synchronized native GPU API wall duration for:

- carry/encoding of64×192 physical tokens;
- Route0 proposals with16 active candidates in Route mode;
- Gate0 proposals with3 active candidates in active Flock mode;
- VectorBlock0 proposals with16 active candidates in each mode;
- production forward without top8/aggregate instrumentation,64 target outputs;
- exact native evaluation of64 rows, including top8 and GPU integer aggregation.

The timed dose is **three Route, three Route-VectorBlock, three Gate and three Flock-VectorBlock attempts**:153 active candidate evaluations, at most9792 candidate rows,384 plain forward rows,384 evaluation rows and73728 carried physical tokens. Warm-up adds four attempts,51 active candidates/3264 candidate rows,128 plain forward rows,128 evaluation rows and24576 carried physical tokens; it is charged separately, never hidden. Baseline replay is additional work in the actual implementation, not hidden from elapsed time. No proposal is applied. Snapshot comparison verifies unchanged table/routes/gates/epoch. Snapshots are outside individual timed intervals and must be charged separately in total preparation/provider cost.

`std::time::Instant::elapsed().as_nanos()` reports integer host wall nanoseconds around the synchronized APIs. No floating CUDA-event milliseconds, floating conversion, model arithmetic or host model-metric reduction is used. Keep all per-repetition durations and integer min/median. Floor rates use `work_units*1000000000/median_ns`, reporting physical context tokens and output-target rows separately. Evaluation is teacher-forced target scoring, not an autoregressive generation/decode throughput claim. No speed bar is registered. Failed operation timings and CUDA faults remain in the exclusive report; no recovery silently converts them to PASS.

The JSON labels total dose as **declared capacity**, not completed work. Attempted/completed timed API and proposal counts come from actual recorded events. A setup or warm-up fault cannot claim all planned candidates/rows were evaluated. Warm-up's individual child completion is not inferred from a failed combined warm-up event.

The isolated V2/B2/C1 control separately records integer nanoseconds for actual apply and recarry. Those values are **tiny-fixture operation cost**, not production-shape apply throughput. No post-apply checkpoint is saved or archived. Production measurement has applied updates0. GPU free/total byte point samples are reported before/after model/carry/evaluation with forward/counterfactual/evaluation `CU_FUNC_ATTRIBUTE_LOCAL_SIZE_BYTES`; samples are not claimed peak memory.

At this shape S=5, VB=16384, BS=20, total16404 coordinates. Each mode projects a fixed-batch parameter pass as `VB*median_VectorBlock_ns + BS*median_Route_or_Gate_ns`. This is a projection, not a measured full sweep or corpus pass; Route-Vector timing is not substituted for the heavier Flock-Vector path. Projection status stays UNVERIFIED. No corpus dose or parameter pass is actually executed.

Any later sweep/dose/dollar projection must bind its actual parameter schedule, active candidate inventory, eligible row replays, applied updates, recarry, checkpoint/evaluation cadence, setup/retries and preservation. Source-loop ceilings are not measured GPU instructions. Monetary/rate projections use exact numerators and denominators and stay UNVERIFIED until their measurement exists. This slice neither chooses a corpus dose nor runs training.

## Steps

1. Freeze the integrated source, exact control names, caller/module ABI, evaluator/count metadata contracts and expected values. Build on an owned admitted CUDA12.8+ Ubuntu24 pod through `scripts/pod/uor-pod`; no agent executes a model locally.
2. Obtain independent prospective FIT at the exact sheet/source. Record scoped admission for the eleven authored apply calls/eight updates; the separate corpus-training/M2 holds remain.
3. Verify release binary/source/PTX/geometry/device pins and compiled control inventory. Root registers the explicit driver qualification and cost commands; no nonexistent command is assumed ready by this draft.
4. Execute the exact finite controls and separate no-update cost measurement once. Preserve exclusive JSON reports, stdout/stderr/exit codes and partial records. Any CUDA fault stops subsequent launches.
5. Preserve all setup/build attempts and exact source/artifact identities, charge cumulative provider/build/control/preservation time, and verify archive round-trip before pod deletion. Preserve peer work. No checkpoint may be promoted as a trained result.

At most one owned admitted pod, within the all-lab limit of4 and$8/h. At registration, pod `vast-55833746`, $0.908/h, had completed setup/release compilation while numerical execution was NOT_RUN and operation cost UNMEASURED. The later outcome below does not alter these prospective bars. The twelve timed plus four warm-up production-shape proposals may dominate this finite slice; they resolve D2's training-dose feasibility and do not train. A healthy run is not cancelled because a self-set estimate expires; actual hard stops are owner caps, disk floor, failed/unsafe CUDA and source/contract mismatch.

## Stop rule

Any failed exact construction control blocks qualification and prevents cost
measurement until diagnosed. A CUDA fault hard-stops further launches; partial
records remain NOT_RUN, never PASS. Source/artifact mismatch, owner caps or disk
floor block execution. No performance threshold rules out a mechanism; the cost
report informs the next implementation or dose decision. A repaired controller
uses a fresh directory and renewed exact-source FIT where source/bars changed.

## Minimum test power

These are deterministic construction witnesses, not statistical learning claims:
25 declared controls, zero skipped/NOT_RUN, exact integer identities and explicit
eligibility. The timing instrument retains three warmed repetitions per entry,
without claiming a confidence interval or performance guarantee. No language bar,
two-seed learning comparison, fresh-transfer KEEP or mechanism rejection follows.

## R12 author answers, for independent review

1. **Quantity:** exact native update/epoch/refusal behavior, top8 and masked TRAIN reference/evaluator correctness, seed/schedule/resume identity, and synchronized API cost at a named nontraining shape.
2. **Computable:** literal integer fixtures and GPU operations at the frozen integrated source, which compiles in release mode with the AOT instruction/entry admission; metadata coverage is host-only. Current source/API/build pins are recorded above; prior qualification is not substituted.
3. **Discriminating instrument:** 640→544 requires the actual codebook update; target7/8 discriminates top8 membership; TRAIN3/4 versus DEV9 distinguishes reference leakage; explicit eligibility prevents zero-rival penalties being labelled learning. Exact snapshots/cursors expose mutation and resume mistakes. Timing observes the real proposed training API, including synchronizations.
4. **Stage:** this construction/cost slice is necessary before D1/D2 can support Kimi's eventual learning sheet. It authorizes no corpus dose, language/KEEP result, full E8/D11 claim or training-hold release. Instrument/fixture defects are repaired and preserved rather than treated as mechanism negatives.

## Verdict

TEST FITNESS: FIT

Prospective independent non-author Codex reviewer `/root/independent_a1_fit`, accepted by root before execution: [PR verdict6158277830](https://github.com/UOR-Foundation/uor-r4/pull/2400#issuecomment-6158277830), [M1 verdict6158278165](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6158278165). Exact source and artifacts are pinned above. All A1 and optional instrument changes were independently confirmed; no law, literal, inventory or dose changed.

Claude's [original conditional FIT6158017847](https://github.com/UOR-Foundation/uor-r4/pull/2400#issuecomment-6158017847) remains preserved. Claude's subsequent confirmation was not received: the scheduled inbox summarized into a separate chat without waking the working session, and a safe CLI follow-up failed expired OAuth. This new independent verdict uses the owner's standing exact-head/self-review allowance; it is not attributed to Claude. The previous9179 build remains archived as superseded construction evidence.

R12 reviewer answers:
1. Quantity: exact update/refusal/epoch, evaluator/count reference, seed/schedule/resume and synchronized GPU API cost; no trained language quality.
2. Computable: declared GPU controls have concrete integer witnesses, and the release build, compiled inventory, PTX, geometry and environment are bound above. Timing uses integer nanoseconds around synchronized APIs.
3. Discrimination/possible bar: literal values, nonvacuous counts and full-state comparisons can pass and distinguish wrong behavior. Cost has no speed-rejection bar; three warmed samples are observations, and parameter-pass projections remain UNVERIFIED.
4. Stage: finite construction/cost before learning dose selection. No corpus update, post-apply artifact retention, language conclusion, mechanism rejection or owner-hold release follows.

At registration all numerical controls and cost observations remain NOT_RUN. Execution receipts will be recorded separately, without altering these prospective bars.


## Execution outcome (added after the registered run)

Exact pinned executable: all25 controls PASS, no failure/skip/NOT_RUN; apply calls11,
disposable fixture updates8, corpus updates0. Separate no-update measurement PASS
in66.633432500s. Controller ran00:42:39Z–00:43:57Z on14OctoberUTC.
[Complete canonical result, raw reports, cost and preservation](../../labs/discrete-driver-cost-2026-10-13/README.md).
The evidence archive was verified before pod deletion at00:48:44Z; no post-apply
model is retained. Training/M2 HELD; no language or mechanism-negative conclusion.
