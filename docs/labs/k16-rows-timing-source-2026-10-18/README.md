# K16 rows timing source and release integration — 18 October 2026

The existing K16-A versus K16-R admission redirect now has an explicit `rows_v1`
selector in its synchronized timing instrument. Both the excluded warmup and
measured Route/VectorBlock proposals call the same `Device::propose_rows_cached`
entry used by the production training driver. This is source and host-metadata
work: **new-head CUDA construction, qualification, timing and training are NOT_RUN
by this delivery**. No kernel, objective, scientific bar or learning schedule changes.

## Actual qualification build failure and narrow repair

The single owner-released qualification attempt selected protected source
`bccc1f172c9790b5edc531185f5146b1de6669e8`. Setup completed on one RTX5090,
Ubuntu24.04.3, NVCC12.8 and Rust1.97.1. The native release build passed in52.62s.
The standalone timing release build then failed with E0063 at
`counterfactual_controls.rs:514`: its CUDA-only `Snapshot` literal omitted the
required `read_initialization` field. The native stage0 job stopped at
2026-10-18T15:19:20Z/exit101; fixture build, clippies, controls, G-Pre, C-R,
registered timing, corpus updates, fresh scoring and training were NOT_RUN.
[Actual failure handoff6251622438](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6251622438).

The same stale literal exists in the fixture runner's `make_route_snapshot`.
This source repair explicitly selects `ReadInitialization::ZeroGatesV1` in both
Route constructors. Both already construct zero gates; the existing default is
skipped during serialization. No kernel, driver, schema, objective, fixture data
or scientific bar changes. The two standalone manifests have detached workspaces
and default to no CUDA feature, so host tests did not type-check these literals.
The production snapshot constructor already records this field.

The observed compiler failure is the failing regression evidence. After this
repair, host metadata tests pass: timing49PASS/0FAIL/1ignored with the same
explicitly filtered inherited reuse-identity test; fixture11PASS/0FAIL. These
checks do **not** verify the repaired CUDA-feature build. Both real standalone
CUDA builds and the entire qualifying receipt sequence remain owed at the
reviewed source. No accepted on-pod source patch, model replay or new allocation
is performed by this source repair. The failed attempt and its operational
lock-inheritance fault remain preserved separately; neither is a scientific
negative. Current native total quote is$0.908/h, not an invoice; custody/close and
actual elapsed cost are still pending at this record's preparation.

[Owner-delegated acceptance6251237195](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6251237195)
now belongs to Claude alone for the single qualifying packet after stored
receipts and digests. DeepSeek's copy cross-check is parallel. This delegation
releases no training: dispatch still requires the actual owner's release.
AGY's consumer repair proceeds independently. No Kimi acknowledgment is added.

## Existing decision and concrete release sequence

The [owner decision relayed by Claude6250410397](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6250410397)
selects the existing [admission-redirect sheet](../../harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md),
[FIT6199263447](https://github.com/UOR-Foundation/uor-r4/pull/2515#issuecomment-6199263447)
and [conditional owner release6201365250](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201365250).
Both A and R use rows_v1; Route, canonical vector-then-read schedule,
V4096/B4/C192/K16, seeds1/2,16,404 batches each and all registered bars remain.
This is four fresh-initialized trajectories, with frozen DEV checkpoints at
0/4,101/8,202/12,303/16,404. The earlier K8 Relation/frozen-zero attempt and its
4,096-batch collapse gate are separate consumed configurations.

[Kimi's current disposition6250526855](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6250526855)
is source/packet preparation GO, training/packet acceptance NO-GO pending the
actual predicates. The ordered chain is:

1. Reviewed and protected-delivered timing source and Claude's R/K16 rows qualifier;
   four newly pinned rows configs, runtime paths and operational packet. The
   [owner Q4 timing waiver6251027125](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6251027125) removes the two Q4 cells and evaluation-only
   importer from the critical path. The unchanged source already supports selecting
   only checkpoint-0 A/R; no Q4 runtime readiness or importer implementation is claimed.
2. Actual new-source build/executable/PTX/geometry/compiler/GPU identities and
   one qualification session:25/25 finite controls plus `admit_ptx`, C-R1..6,
   G-Pre≥4,894, nonvacuous R/K16 rows parity, refusals/negative/zero-update controls,
   and the registered timing.
3. **Two timing cells**, checkpoint0 × A/R, **100 completed samples
   each plus1excluded warmup**. The owner explicitly waived Q4. Preserve
   raw samples, counts, snapshot/epoch equality, path and image identities.
4. Accepted completed D-Packet with genuine receipts, exact four arm/seed identities,
   original fresh-source binding and measured all-in budget; then the existing
   conditional release permits the four training trajectories. Fresh scoring is
   before KEEP/capability promotion, rather than a training-start prerequisite.

The conditional release does not need another strategic owner acknowledgment.
Actual packet review and PM dispatch remain operational work. Five-sample
qualifier timing, earlier99-sample records or old checkpoint0 rates cannot fill
this100-sample/two-cell slot. The measured unit includes carry, Route and vector
proposal/observation on a frozen state; it is **forward-only, not training-dose
throughput**, and the budget must retain evaluation, accepted-update, I/O and
custody costs. A healthy run is not cancelled because an estimate expires.

## Selector and receipt contract

`Pin.counterfactual_path` reuses `training::CounterfactualPath`. Omission selects
`lanes_v1` and remains omitted on serialization, preserving legacy pin bytes.
Explicit `rows_v1` selects the production cached rows image. Unknown variants
are refused. The separately scoped subphase diagnostic profiler remains
lanes-only; combining it with rows is refused during host validation, before a
CUDA context or source artifact is opened.

Plan/record headers, per-proposal and batch receipts, and the final cell summary
name the selected path. Rows cells also record the exact embedded
source/PTX/cubin/resource-report hashes through `compiled_rows_image_sha256`.
The ordinary compiled identity remains separately reported. Synchronization,
100+1 window allocation, rate denominator, policy/artifact admission, partial
fault preservation and unchanged snapshot/epoch assertions are retained.

## Verification and limits

The new test first failed because the old strict Pin rejected
`counterfactual_path`. After the adapter, all three new host tests pass:
rows plan/record binding, omitted-default serialization and unknown-path/
rows-profiler refusal. The focused instrument suite reports **49 passed,
0 failed,1 ignored,1 explicitly filtered historical test**. Formatting and
host `cargo clippy --all-targets` pass, with inherited dependency warnings.

The unfiltered suite reports49passed/1failed/1ignored. The inherited failure is
`counterfactual_reuse::tests::exact_loaded_images_and_distinct_stage_authority_are_required`:
its synthetic positive metadata cannot override the actual included CUDA source,
which changed from historical `f679f873…` to current `8cc15ccb…` in#2601.
Both that guard/test source and the included CUDA file are unchanged here. The
historical image guard is retained; its frozen source hash is not relabeled to
make the archived comparator pass. This limitation is unrelated to rows timing
and is not a CUDA/runtime qualification waiver. The existing ignored case stays
ignored. No CPU model/reference, corpus evaluation, GPU, provider or scorer runs.

All test/build/review records are retained as official delivery evidence in the
private operator packet; they are not disposable debug output. This source unit
uses no paid resource. The earlier cumulative resource ledger remains separate.

## Ownership and next dependency

Codex owns this timing adapter and the external operator/config/D-Packet integration.
Claude owns the R/K16 rows qualifier and exact source review; AGY reuses the
existing redirect runbook/consumer with the actual four identities. Kimi owns
operational packet acceptance/dispatch. While Kimi is offline, DeepSeek offers
source-queue cover under the owner's autonomous direction; model acceptance and
honest machine owner/admission attribution remain separate. No duplicate
scorer or proposed K8 R-only/B3 trajectory is introduced.

The original transfer archive has been restored once and all16 original manifest
members SHA-verified:2,000 eligible targets across200 documents,505,085 payload
tokens, Split::Fresh. It remains distinct from attention FreshV2. The four host
configs change only the explicit rows selector and five physical input paths;
all input SHA pins match. Native runtime staging/lineage/build and genuine
qualification still must bind their actual resulting bytes.

The [prior rows Q1 result](../attention-rows-q1-diagnosis-2026-10-18/README.md)
now has the designated scientific outcome: configured admission collapse with
no implementation/identity fault found. It supplies no mechanism negative,
paired advantage, KEEP or count reset. **Reply37/232 remains SUSPENDED; count0/3
for that attention line is unchanged.** The K16 redirect retains its own sheet
and progress accounting; source preparation cannot move a headline number.

Delivery line: `k16-admission-redirect`, registered count1/3, headline reply37/232
SUSPENDED. This source unit moves no headline and claims no reset. The already
merged#2653 transport claim is removed after fresh verification that its own
worktree/local/remote branches are absent; its verified source/claim-delta cloud
restoration remains preserved. Peer claims are unchanged, and the active timing
claim remains. This is scoped bookkeeping in the normal source unit.
