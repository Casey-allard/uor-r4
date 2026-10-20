# Discrete driver and measured enumeration cost

Date: 2026-10-13. Codex/Sol, M1 #2029; shared writer/read interface M4 #2032.
Line: discrete read/writer construction. No language headline or learning KEEP/count reset.
Status: 25/25 finite CUDA controls PASS; Route and active-Flock cost observation PASS. Delivery: [PR #2400](https://github.com/UOR-Foundation/uor-r4/pull/2400); GitHub owns its merge state.
Training and M2 remain **HELD**.

This unit supplies the D1 corpus/evaluation/schedule interface and D2 cost instrument
for Kimi's separate learning sheet. Three Codex agents implement disjoint slices;
Claude reviews the prospective scope and exact source. It preserves the merged
SpiralCore/E8/FBS carrier, shared token codebook/writer and discrete flock read.
The atlas remains the sixteen-root anchor orbit, not all 240 E8 roots.

## Driver contract

`prepare-plan CONFIG NEW_REPORT` is metadata-only. The Rust loader verifies file
SHA256, canonical 64-byte UORT header, token count/vocabulary, binary response mask,
explicit tokenizer/BOS identity and expected eligible counts. Each selected target
uses its preceding causal prefix, clipped at BOS and the configured context. A final
short batch is retained. The target and future positions are excluded from input.
Masked positions never enter the count reference or target eligibility.

The [example configuration](driver-config.json) pins the existing TRAIN/DEV and
tokenizer bytes. Its paths are restoration paths, not committed datasets. Metadata
admission observed 2,425,925 TRAIN targets and 14,287 DEV targets; see the
[coverage plan](receipts/coverage-plan.json). The DEV manifest's historical `split:
train` label is not its role: the explicit config pins it as DEV. Corpus files were
restored from the existing private HF store and checked against the sheet's hashes.
No corpus model computation follows from this metadata admission.

`canonical_vector_then_read_v1` visits ascending codebook blocks, then active Route
coordinates or Flock gates, once per coordinate pass. One coordinate is attempted
per batch; the corpus advances in ascending eligible-target order and wraps only
after complete coverage. Initialization has an explicit integer seed; data and
coordinate order are identical across paired seeds. No shuffle is implied.

`init-corpus CONFIG NEW_REPORT` constructs an epoch-zero GPU checkpoint.
`eval-corpus CONFIG DRIVER_CHECKPOINT NEW_REPORT` validates the checkpoint's full
source/config/data/mask/tokenizer/role/cursor lineage before restoring it. The
checkpoint stores complete integer model state and the coordinate/data cursor;
resume rejects changed identities. `init` and `init-read` require an explicit seed.
Every report directory is exclusive and reserved before model loading.

The complete corpus transition loop exists behind an opaque, unissued
`CorpusUpdateAdmission`; its constructor returns `TrainingHeld`. The public train
CLI always refuses. There is no flag, environment override or deserializable token
that releases this hold. Owner release needs a separately reviewed admission change.

## GPU evaluation

The native CUDA evaluator uses the same read and writer as inference. Global top8
orders all admitted token identities by exact distance then ID; it never concatenates
per-cell top lists or inserts the answer. Component hinges, exact denominators,
rival eligibility, hits and batch/corpus aggregation execute on the GPU. The host
decodes completed receipts, prepares input metadata and stores results.

The frequency reference counts response-masked TRAIN targets on CUDA. A separate
DEV-response oracle is labelled diagnostic; changing DEV cannot alter the TRAIN
histogram or ranking. The pre-existing no-float/no-softmax/no-hidden-matmul contract
remains. No Python model dependency or CPU model implementation is introduced.

## Prospective qualification and cost scope

[Claude's prospective review](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6157700698)
requires replay to match actual applied state, all three parameter kinds, stale and
foreign refusals, no-op epoch stability, deterministic resume, mask/BOS coverage,
and measurement of active flock as well as Route. The
[finite sheet](../../harness/prep/m1-discrete-driver-finite-2026-10-13.md) owns the
exact inventory and prospective FIT. The exact execution receipts below distinguish qualification from prospective review.

Only literal, authored, disposable fixture models may receive bounded construction
updates. They are dropped in-process; no post-apply checkpoint is saved or promoted.
The report counts every accepted update. Corpus data never enters an apply call.

Cost uses a release Rust build and CUDA 12.8+ on one owned Vast GPU. The measured
shape is vocabulary4096, four8D blocks, context192, batch64, writerK8, with both Route
and FlockK8. Active flock uses authored +1 gates because zero gates skip the read;
this is a cost fixture, not a selected learning arm. Warmup precedes three raw
repeats. Integer host nanoseconds surround synchronized device calls. Setup/module
load/JIT/init is separate; no unsupported exact JIT subset or GPU peak-memory claim.
Device memory samples and compiled per-thread local-memory attributes are reported
as observations. Tiny-fixture apply timings are not production apply throughput.

Raw timings, integer min/median and floor-divided tokens/s will support projections
of a parameter pass and bounded dose. A projection is not an executed corpus sweep
or measured training throughput. A slow configuration informs implementation/dose;
it cannot rule out a mechanism. No learning or language-quality result is claimed.

## Delivery instrumentation repair

CI after UTC midnight exposed a hard-coded October13 timestamp in both live Rust
status comparisons in `scripts/harness/parity.py`, while Python status used the
current date. The receipts check compared different days, not different readings
of the same data. The bounded fix supplies a current timezone-aware local timestamp
in both comparisons and preserves every assertion. DeepSeek/Kimi were notified on
#2028 before the repair. This existing tooling change adds no model dependency and
does not change the frozen CUDA source. The initial missing standard prep headings
were also corrected; the harness now reports this finite sheet complete/PENDING,
before the prospective FIT was recorded. After DeepSeek #2401 added Python clock injection, both status comparisons now pass the same current timezone-aware instant to Rust and Python; all assertions remain.

## Observed qualification and cost

Executable source `e7d926fbe38537f4faa90b817705c61a780778d6`, crate tree
`218b2e7bc2a5e41332f10dcba666b10275046ef6`; all later delivery changes are
records or harness integration. [Artifact pins](receipts/PINS.json) and
[build/inventory receipt](receipts/build-inventory.log) bind the release executable,
11-entry AOT PTX and generated geometry. GPU: one RTX5090, driver580.178.04,
Ubuntu24.04.1, Rust1.97.1, CUDA12.8.93, explicit compute_120. A1 release build
Cargo16.06s/controller17s; superseded initial build36.42s/controller38s retained.

[Prospective independent FIT](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6158278165)
was posted before the controller started at00:42:39Z on14OctoberUTC.
The fresh non-author Codex reviewer confirmed every A1 item and supplied all four
R12 answers. Claude's original conditional review remains attributed to Claude;
this follow-up verdict is explicitly Codex's. The desktop inbox summarized requests
in separate chats without waking the working session; CLI fallback failed expired
OAuth. The standing owner exact-head/self-review allowance enabled the independent
review without silently changing a literal, bar, scope or dose.

**25/25 controls PASS**, zero failures, skips or NOT_RUN:
[11 Route](receipts/route-controls.json), [6 Flock](receipts/flock-controls.json),
[8 driver](receipts/driver-controls.json). The driver recorded11 apply calls:
eight improving authored fixture updates, foreign/stale refusals and one no-op.
Zero corpus updates; no post-apply model was retained. The controller's
[raw timing/exit record](receipts/controller.jsonl) and [final result](receipts/RESULT.json)
show all four commands exited0. Controls took5.989098677s,2.833462330s,2.948602761s;
measurement took66.633432500s. The numerical slice finished00:43:57Z.

[Raw cost observation](receipts/measurement.json): V4096, B4 (32 coordinates),
C192,64 rows, writerK8, seed7; FlockK8 has all authored gate trits+1. Three warmed
repetitions per operation, all30 timed APIs and all12 timed proposal calls complete,
zero applies and unchanged model snapshots.

| Synchronized API, median | Route | Active Flock |
| --- | ---: | ---: |
| Carry64×192 physical tokens | 1.314284ms | 0.435395ms |
| Plain forward64 target rows | 37.837521ms | 40.576018ms |
| Evaluate64 rows, top8 + aggregate | 42.143551ms | 44.765762ms |
| VectorBlock proposal | 3.903280515s | 4.206217974s |
| Route0 / Gate0 proposal | 3.959623663s | 4.180529788s |
| Forward target rows/s, integer floor | 1691 | 1577 |

These are fixed-batch native API observations, **not autoregressive decode rates**.
Context-token rates count192 physical inputs per target, so they must not be presented
as generated tokens/s. Candidate proposals replay64 rows serially per lane and rescan
the writer; that is the practical dose constraint observed here. No speed threshold
rejects the geometric mechanism. Tiny V2/B2/C1 fixture apply47,772ns and recarry36,457ns
are not production-shape update throughput.

The report's fixed-batch parameter-pass projection is
`16384 * median_vector_ns + 20 * median_read_ns`:
Route64,030.540431020s (17.786h), Flock68,998.285881776s (19.166h).
At the quoted908milliUSD/hour, this corresponds to about$16.150/$17.403 for one
such projected parameter pass. **UNVERIFIED projection**, not an executed parameter
sweep, corpus epoch, training dose or authorized spend. It excludes apply/evaluation,
data variability, changing learned state and setup. The one-coordinate-per-batch
schedule does not multiply that whole parameter inventory by every corpus batch.
Kimi must select and review the actual finite learning dose before owner release.

Setup is separately recorded: combined load/JIT/init0.136535238s/0.127749825s,
first-use warmup7.949718160s/8.476495931s, plus context and snapshot setup.
CUDA reported per-thread local allocation16,624B forward/17,136B counterfactual/
16,784B evaluate. Device free-memory samples after warmup were28,892,200,960B Route
and28,890,103,808B Flock out of33,668,726,784B. These are point samples and compiler
attributes, not measured peak resident memory or an isolated JIT benchmark.

## Preservation, resources and next dependency

Pod `vast-55833746` was released and deleted at00:48:44Z; fresh provider status
confirmed zero running pods/$0h. Its final ledger reports2,544s uptime at$0.908/h:
**$0.641653… quoted lifecycle cost**, rounded$0.642, including setup, both builds,
review wait, qualification, cost measurement and preservation. This is a provider
rate-times-uptime estimate, not an invoice. Earlier discrete qualification spending
of about$1.078 is retained, giving approximately$1.720 cumulative across these
qualified slices; this task does not reset the shared ledger.

Before deletion, complete pod evidence (source archive + exact A1 patch, both build
logs/binaries, PTX, geometry, setup, controller and all raw results) was archived to
`icloud:UOR-R4/results/codex/discrete-driver-e7d926fbe-20261014.tar`.
Cloud archive82,316,800bytes, MD5 `51c2c89e54eb1dfc90dc42c960892e23`, download-round-trip
verified. Inner pod evidence SHA256
`0e2c36275695e3111317fbdeda74ab76041b3014eee05c7d6ca0c0c3bd155a6f`
matched pod and local download. Restored TRAIN/DEV/tokenizer bytes remain pinned to
existing HF stores; no private corpus is committed here.

Kimi can now bind D1's exact interface and D2's measured cost in the next learning
sheet. Raw top1/recall8 versus masked-TRAIN frequency remain distinct from the
same-seed strict-distance-win anti-vacuity diagnostic. No raw(td,rival-distance)
histogram is implemented. The sixteen-root anchor orbit and restricted transport
remain explicit configuration limits. **Training/M2 HELD; R14 reply-quality target
SUSPENDED.** This construction result is not learned language evidence or a KEEP
count reset. Corpus admission stays unissued pending a separately reviewed release.
