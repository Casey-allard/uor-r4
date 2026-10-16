# K16 qualification restart 5 — G-Pre/C-R pass, timing sample-count blocker

**TIMING_BLOCKER_TO_FIX / NOT YET PROMOTED.** The six step-0 commands,
G-Pre and C-R1..C-R6 pass. Timing exits 0 and emits `status: complete`, but
each cell contains only **99 timed batches**, so the executor's declared
100-batch receipt gate exits 1. Execution stops without patch or retry.
This is a D22/D31 instrument blocker, not a mechanism negative or count increment.

Execution source: `812459c7f30510db23f585c724fe70f7f1f98a99`.
Codex/Sol executes the owner-authorized qualification scope; Kimi PM dispatches
independent exact-head receipt review. This record does not authorize training,
promote KEEP, or claim a language result. Line: exact addressed memory;
count 0/3 and suspended M1 headline 37/232 unchanged.

## Authority and identity

- [Owner qualification release](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201365250),
  [PM gate order](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6205684341),
  [restart-5 GO](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6208485483).
- [FIT sheet](../../harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md),
  [runbook](../admission-rank-qualification-runbook-2026-10-15/README.md),
  [D-Packet](../admission-redirect-packet-2026-10-15/README.md).
- Fresh main includes #2538 receipts and #2537 lint repair. Empty instrument
  diff against approved/gate-checked `3e3a5892ea23410d50478956ca04eb1c45c99dcc`
  confirms source byte identity. The pre-merge gate transcript does not replace
  this attempt's six merged-source step-0 commands.
- [Pre-lease acknowledgment](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6208508926),
  [pod acknowledgment](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6208545838),
  [G-Pre/control progress](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6208666220).
- Pod `vast-56202766`, session `codex-k16-qual-20261016-5`, RTX 5090,
  $0.842/hour. Rust/Cargo 1.97.1, NVCC 12.8 V12.8.93,
  `UOR_NVCC_ARCH=compute_120`. All five frozen tokenizer/train/dev SHA-256
  pins match before execution. No fresh-transfer data is staged or scored.
- Production driver source SHA-256:
  `ed4655a22dbec24316cfe5df16a43407860651ce6aa7a690ee8fcd5bedaf05d8`;
  geometry `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`.
  The four emitted typed configuration hashes match the runbook's exact pins.

## Step 0 and G-Pre

All three CUDA release builds (discrete driver, timing instrument, fixture
runner) and all three declared CUDA clippy gates exit 0. Fixture clippy retains
`--all-targets -- -D warnings`; the other two retain their declared flags.
Dependency warnings remain in the full transcripts; exit 0 is not a claim that
all dependencies are warning-free. No host model/reference execution occurs.

G-Pre initializes fresh checkpoint-0 models and evaluates the frozen 14,287-row
dev panel. Each receipt records epoch 0, accepted updates 0, evaluation updates
0, and `evaluation_mutates_model: false`. The K8 seed-1 sealed reproduction
guard matches exactly, including admitted strict wins 0 and address-hinge sum
261,824. Both K8 controls and every K16 arm exceed the unchanged 4,894 ceiling bar.

| Configuration | Admitted / 14,287 | Top-1 | Top-8 | Strict-win eligible | Occupied cells |
| --- | ---: | ---: | ---: | ---: | ---: |
| K8 A, seed 1 | 5,680 | 17 | 173 | 105 | 16 |
| K8 A, seed 2 | 6,205 | 20 | 173 | 102 | 16 |
| K16 A, seed 1 | 14,287 | 19 | 151 | 0 | 16 |
| K16 R, seed 1 | 14,287 | 19 | 151 | 0 | 16 |
| K16 A, seed 2 | 14,287 | 19 | 151 | 0 | 16 |
| K16 R, seed 2 | 14,287 | 19 | 151 | 0 | 16 |

This is an untrained admission ceiling/control result, not learned language
quality or a mechanism comparison. The measured K16 strict-win baseline is
`u = 0/14287` for both seeds. The production GPU count-reference path also runs
inside `eval-corpus`; no CPU scoring substitute is used.

**Ceiling is not discrimination:** checkpoint 0 occupies exactly 16 cells, so
K16 admits every occupied cell. Full-panel admission is the runbook's expected
capacity/pass-through boundary, not evidence of selective routing or correct
token ranking. Top-1 is only 19/14,287. This explicit limitation answers the
[PM review caveat](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6208739484);
independent receipt acceptance is still required and no registered bar is changed.

Digest domains are explicit: checkpoint `model_sha256` hashes the JSON `Value`
storage representation, whereas evaluation `model_snapshot_sha256` hashes the
typed `Snapshot` serialization. A direct cross-domain equality check is invalid.
Metadata-only reconstruction in the declared `Snapshot`/`Spec` field order
matches all six evaluation digests; A/R initial table/routes/gates match per seed.
This serialization check performs no CPU model/reference computation.

## C-R controls

The reviewed fixture runner consumes this attempt's G-Pre receipts and exits 0.
All six ordered receipts report PASS; each includes its own digest-bound file pin.

| Control | Status | Scope |
| --- | --- | --- |
| C-R1 | PASS | Exact rank/admission equivalence and maximum-rank boundary |
| C-R2 | PASS | Single count per occupied cell; occupancy changes |
| C-R3 | PASS | Virtual versus independently restored candidate parity |
| C-R4 | PASS | Absent-rival refusal and occupancy-under-K bounds |
| C-R5 | PASS | Integer bounds/overflow, dual losses and distributions |
| C-R6 | PASS | Stored baseline, checkpoint-0 occupancy, G-Pre receipt contract |

These are actual CUDA fixture receipts, not host scaffold tests. Corpus apply
calls are zero; authored candidate snapshots do not mutate the G-Pre models.
The historical 25 finite controls were not separately rerun in this explicitly
ordered C-R qualification scope and are not claimed as newly passed here.

## Timing: observed 99 batches, required 100

The admitted timing source generates 100 distinct synthetic windows per policy,
64 target rows and C192 padded contexts at V4096/B4/K16, Route reading, with
one excluded warmup per cell. It then skips that first window, leaving only
**99 timed batches**. Every timed batch includes carry plus BOTH
`Route(0)` and `VectorBlock(0)` proposal/observation phases, each synchronized.
This is not an 80/20 family-stratified coordinate sample, a canonical coordinate
sweep, or measured training throughput. The original packet's 80/20 proposal
must not be used to describe this executed runner. The full-unit rate denominator
includes both proposal families; padded context tokens are not target rows.

Source cause at the executed revision: `k16-timing/instrument/src/main.rs:541`
uses `batches(pin.batches)` while line 591 iterates with `.skip(pin.warmups)`.
The pin declares `batches: 100`, `warmups: 1`. The record has two cell summaries
with `timed_batches: 99` and 198 batch records (indices 1..99 for each policy).
The timing process runs for 2,905 seconds (48m25s), exits 0, and has no internal
fault entry. The separate [receipt gate](gate.failed) correctly rejects it
([exact gate output](timing-status.log)); `complete` is not qualification PASS.

| Diagnostic-only cell | Timed batches | Full timed unit, ns | Target rows/s, floor | Padded context tokens/s, floor |
| --- | ---: | ---: | ---: | ---: |
| Checkpoint0 A (8T+32A) | 99 | 1,437,687,162,188 | 4 | 846 |
| Checkpoint0 R (8T+32R) | 99 | 1,438,077,351,561 | 4 | 845 |

Each cell measures 6,336 target rows and 1,216,512 padded context tokens, not
the prescribed 6,400 / 1,228,800. Snapshot digests before/after match, both epochs
are 0, and every batch/proposal/cell has `applied: false`. The rates are retained
as **99-batch diagnostic observations only**, not substituted into the required
100-batch D-Packet slot. Carry-only rates are not the headline denominator.
The [cell summary](timing-cell-summary.json) is a labelled extraction from the
unaltered full record in the archive; the archive retains every batch record.

[Immediate halt and repair routing request](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6209213499).
DeepSeek owns the bounded timing source repair; PM dispatches exact-head review
and any later execution. No silent extra batch, patched binary, relaxed gate,
receipt rewrite or same-lease retry occurred. The undercount does not erase the
independently completed G-Pre and C-R receipts, but overall qualification remains
blocked. Q4 stays NOT_EXECUTABLE; accepted-update cost, fresh transfer, training
and Stage B/M2 remain unexecuted. No fresh-scoring or training-start gate is waived.

## Where the timing went (sealed-record analysis)

[Exact sums and distributions](cost-attribution.json) are derived from all 198
unaltered batch records. The five phase sums reproduce each raw cell's
`unit_ns` exactly; this is metadata analysis, not another model run.

| Phase | Total seconds, 198 batches | Mean per batch | Share of measured units |
| --- | ---: | ---: | ---: |
| Carry | 0.166262156 | 0.839708 ms | 0.005781% |
| Route proposal | 1,442.065670315 | 7.283160 s | 50.145471% |
| Route observation | 0.005921684 | 0.029907 ms | 0.000206% |
| Vector-block proposal | 1,433.520765784 | 7.240004 s | 49.848336% |
| Vector-block observation | 0.005893810 | 0.029767 ms | 0.000205% |

**The synchronized proposal calls account for 99.993808% of measured time.**
The expensive-carry hypothesis is not supported at this exact C192 scope.
The proposal medians are 7.283270 s (Route) and 7.239918 s (vector); p95 values
are 7.285438 s and 7.242359 s. This is sustained per-call cost, not a few stalls.

Measured phases total 2,875.764513749 seconds, versus 2,905 seconds of
process wall time. The remaining approximately 29.235486251 seconds also
contain two excluded warmups, loading, snapshots and outside-phase work.
Warmups have no separate timer and process timestamps have second precision;
the residual is not a measurement of pure loop overhead. Even attributing it
all to overhead cannot explain the roughly 48-minute runtime.

The timers bracket host API calls plus synchronization, not isolated GPU
kernels. At the executed source, `Device::propose` includes admission/epoch
checks, transfers, allocation/zeroing, the counterfactual and chooser kernels,
status reads and synchronization. Thus the receipts locate the cost in the
**proposal path**, but cannot distinguish GPU execution from overhead within
that API. They do not establish either a slow carry kernel or dominant
outer-loop accumulation.

The smallest discriminating follow-up instrument would retain the full-unit
timer and add CUDA-event intervals for counterfactual/choose, separate host
intervals for allocation/transfers/admission/status/synchronization, and
explicit warmup/outside-phase timers. That needs source review and separately
authorized execution; none is added or executed here. These remain diagnostic
99-batch measurements, not qualified 100-batch rates or training throughput.

## Evidence and cost

Subsequent [#2539 compile-only review overhead](review-overhead-2539.json) is
accounted separately from the frozen restart-5 run receipt: check exits 0,
strict CUDA clippy exits 101, verdict CHANGES. Its 357-second provider uptime
at $1.137/h adds $0.1127525, extending the 14-lease episode estimate to
$1.8861519444. Full wall is 360 seconds ($0.1137 alternate estimate).
The 80,384-byte transcript archive is MD5-round-trip verified before release;
pod deletion and an empty fleet are verified. No model work or retry occurs.
The earlier 13-lease closeout below remains unchanged rather than backdated.

Closeout accounting is recorded in [receipt.json](receipt.json) after verified
pod deletion. The archive is
`icloud:UOR-R4/results/codex/codex-k16-qualification-20261016-5.tar`,
109,805,056 bytes, MD5 `21a4633d605289bbede3f19c4581b3dc`.
All 133 [SHA256SUMS](SHA256SUMS) entries pass locally after transfer; the manifest
digest is `4b127c40c55419f3c7accdcd4953a593ad8addc4b4049b8ca90374b6d529db46`.
The cloud MD5 download round-trip completes **before** release and teardown.

Pod created 05:05:41Z, released 06:06:22Z, deleted 06:06:47Z on 16 October.
Fleet afterward: **0 pods / $0/hour**. Full wall is 3,666 seconds (61m06s);
provider uptime is 3,644 seconds. At $0.842/hour, this lease costs an estimated
**$0.852291**; the alternative full-wall estimate is $0.857437, not added again.
The approximate 30-minute wall estimate was extended for the live bounded timing
job, with [PM-bus notice](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6208813613)
and renewals at 05:30:26Z and 05:57:04Z. No new job or spending class was added.

The complete 13-lease episode is **$1.773399** on consistent provider-uptime
accounting: restart-4 carry-in $0.782706 + the #2537 dual-gate lease $0.138402
+ this attempt $0.852291. The full per-lease table is in the receipt, including
DeepSeek setup failures and every Codex repair/check attempt. These are rate
estimates, not invoices. Storage/transfer invoice is UNMEASURED; no paid pod
storage remains. Codex evidence archives in this chain total 110,516,736 bytes,
including this attempt and the latest 71,168-byte pre-merge check archive.

Full compiler/linter logs, both job transcripts, executed scripts, all raw G-Pre
init/eval receipts, six C-R reports, timing pin/full record, three binaries and
compiled PTX are retained in the private archive. Public summaries never replace
the original sealed records. Raw G-Pre checksums bind the extracted public summary.

Public [G-Pre summary](g-pre-summary.json) and [timing cell summary](timing-cell-summary.json)
are explicit JSON projections, not replacements for the raw reports. C-R reports,
their [suite summary](c-r-fixtures/c-r-suite-summary.json), command journals,
input pins and executable/PTX hashes are exact copies of archived files. Source
remains unchanged on the pod. No qualification PASS, trained-model result or
training release follows from this receipt PR. Next: PM-routed timing repair,
reviewed regression for the exact warmup/sample count, exact-head CUDA gates,
protected merge and separately authorized continuation; preserve these receipts.
