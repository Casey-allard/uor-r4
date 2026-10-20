# A2 seed-1 diagnostic pilot

Codex/Sol,2026-10-14; References #2029. This is the operational record for the
owner-delegated D33 release path and the registered1640-batch pilot, not a full
training run or a language result. M2 and all other holds remain HELD.

## Completed diagnostic result

The qualified executable completed the admitted seed-1 pilot on its first attempt,
04:38:17Z–06:20:41Z, **exit 0**, on one RTX 5090. It recorded all **1,640 batches**,
**104,960 masked-target exposures**, and **236 accepted VectorBlock updates**.
There was no resume, retry, parameter-order change or additional training.
The release is consumed by this completed pilot; subsequent training remains HELD.

| Measurement | Checkpoint zero | Pilot end |
|---|---:|---:|
| DEV scored targets | 14,287 | 14,287 |
| Top-1 correct | 17 | 82 |
| Top-8 correct | 173 | 1,169 |
| Strict distance wins | 105 | 149 |
| Aggregate loss numerator; denominator per row = 256 | 13,816,032 | 13,884,032 |
| Token hinge sum | 679,708 | 671,712 |
| Address hinge sum | 261,824 | 265,948 |
| Distinct full vectors / 4,096 token identities | 16 | 155 |
| Occupied writer cells | 16 | 124 |

The exact masked-TRAIN count reference is unchanged: 1,226 top-1 and 4,060 top-8
correct on those same DEV targets. The separate DEV-response oracle is diagnostic
only. Of 1,640 vector proposals, **1,179** had some candidate cost different from
identity; **236** accepted a strictly improving move on their own fixed batch.
Route parameters were not attempted: their sensitivity remains **NOT_RUN**.

The shared writer changed and these DEV rank counts improved, while the aggregate
loss numerator increased and the model remained below the TRAIN-count reference.
This is a mixed, bounded first-learning observation. It establishes neither useful
language quality nor a mechanism verdict. The pilot visits only VectorBlock
0..1639 (token IDs 0..409), and 236 of those 410 token identities have zero masked
TRAIN target count. Their combined TRAIN share is 1,440,407 / 2,425,925 targets;
this is vocabulary-slice coverage, not the pilot's 104,960 target exposures.
Bars and guard remain **NOT_DECIDED_PILOT**, mechanism verdict **NOT_ASSIGNED**,
fresh transfer **UNAVAILABLE_NOT_RUN**. No negative, KEEP-with-transfer, full-run
permission or M2 release follows.

[Compact report](report-summary.json) omits only bulky per-batch/count arrays;
the complete report is preserved in the archive below.
[Receipt audit](receipt-audit.json): **PASS** for all 1,640 recorded steps,
cursor/dose continuity, strict loss decrease on accepted moves, unchanged
baseline/count reference, unchanged Route/gate state, and exact binding of the
final evaluation to the saved checkpoint. It is metadata verification, not a
second model run or a scientific acceptance bar.

**Next:** Kimi and Claude use these measured sensitivity, collision, coverage and
cost observations to review a prospective successor sheet. In particular, the
pre-registered collision-free integer initialization successor and broader
coordinate coverage remain proposals requiring amendment and a scoped release.
Do not silently alter initialization, reorder by frequency, or launch seed 2.
The [completion handoff](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6162275408)
states the owner of each next step; no review or release request is left unwritten.

## Complete preservation and cost

Final archive: `icloud:UOR-R4/results/codex/a2-pilot-complete-20261014.tar`,
**5,607,936 bytes**, MD5 `fb81eaa335a1d21ca1d05bde9771b3e3`, cloud-store
download round-trip verified. Its compressed pod bundle SHA256 is
`76b84e33b521fb6efda9ab9d5a9baf395f78dfe4c0371419b98765be5cb75bcf`;
the pod and downloaded copy match. The bundle contains all 27 checkpoints,
all step receipts, complete report, operational run-state pointers, job log,
both staged binaries, launcher and pinned plan/config/sheet/review metadata.
It excludes the Claude-owned release directory and owner record; Claude preserved
the full delegated release in his durable GitHub receipt linked below.

- Complete report SHA256: `e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3`.
- Final checkpoint SHA256: `46e8ed5ea0a282bcd8373922d6fa2c8e773b52efce9fe8c1ba2c4b5698ee0440`.
- Controller wall: **6,143.011024199 seconds** (1h42m23s), including admission,
  initial/final evaluation, checkpoint/report I/O and model work; not isolated
  kernel time. No extra evaluation or model retry was run.
- [Provider lifecycle](lifecycle.json): created 04:24:08Z, released 06:23:02Z,
  deleted 06:23:09Z. Provider uptime 7,138 seconds; timestamp span 7,141 seconds.
  At $0.892/h this lifecycle is **approximately $1.77**, a rate-times-uptime
  estimate, not an invoice. It includes setup, Rust installation/tool build,
  launch waiting, numerical work, monitoring, transfers and preservation.
- Metadata-only local receipt-auditor builds: 9.54s + 11.78s + 3.27s + 2.07s,
  overlapping the live pilot. D33 tool build: 18.61s, included in pod setup.
  Earlier harness build/qualification/D1–D2 costs keep their existing records
  and cumulative ledger entries; this lifecycle is not a programme-budget reset.
- Sampled GPU allocation was 4,556 MiB; this is an observation, not a measured
  peak. The final archive is 5.61 MB; source/corpus and previous qualification
  artifacts remain at their pinned Git/HF/iCloud locations.

Pod deletion was verified against fresh inventory. No owned GPU remains rented.

## Pilot launched

Claude executed the delegated release in
[#2029 receipt 6160973552](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6160973552),
and Kimi verified it in
[receipt 6160999031](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6160999031).
Codex independently checked the on-pod hashes and plan bindings, then started
the qualified executable under its GPU lease at **2026-10-14T04:38:17Z**.
Only seed 1, 1,640 batches and VectorBlock 0..1639 are released.

The initial checkpoint was written and the first observed progress sample had
40 completed batches, 14 accepted updates and 2,560 masked-target exposures.
The CUDA process was live on the RTX 5090. This is a RUNNING observation, not
a completed diagnostic result or a model-quality verdict.

Output: `/workspace/uor-r4/rung0-pilot-20261014/pilot-attempt-1`.
Job log: `/workspace/uor-r4/jobs/codex/20261014T043817Z-a2-pilot-20261014-gpu0.log`.
Admission SHA256:
`be866c7e646e469997dbeb9ffcf8dad92ed46c1f70fd8242f0982d6d47bc9391`.
Owner record SHA256 (read and hashed, not copied by Codex):
`6e6dea5ac3aa3a3a74c9c4d46955cbb4e9add67484dd747c1904393422904ab5`.

Next dependency is completed diagnostics and artifact preservation, owned by
Codex. At launch no further release was requested from Claude or Kimi for this
admitted scope. The completed outcome and lifecycle are recorded above.

## Recovery during the run

Immutable checkpoints 0 and 256 were copied off-pod, compared with the pod's
SHA256 digests, and preserved with a cloud-store download round-trip verification:
`icloud:UOR-R4/results/codex/a2-pilot-recovery-0256-20261014.tar`, 6,528,000 bytes,
MD5 `b10ae238b453132123d6c3244da6c278`. The archive excludes the Claude-owned
release directory and owner record. The recovery receipt and baseline observations
were posted in [#2029 receipt 6161248831](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6161248831).

Checkpoint 704 was subsequently preserved with the same SHA256 comparison and
download round-trip verification:
`icloud:UOR-R4/results/codex/a2-pilot-recovery-0704-20261014.tar`, 3,270,144 bytes,
MD5 `0a76a6cd54537540db3d7783e0f536bd`. This is another immutable recovery
snapshot taken while the final diagnostic report was still pending.

Checkpoint 1,152 was also preserved and download-verified:
`icloud:UOR-R4/results/codex/a2-pilot-recovery-1152-20261014.tar`, 3,270,144 bytes,
MD5 `11635c5cbe9c88a4b1cf596562522fa6`.

The [receipt auditor](receipt-audit/src/main.rs) checks recorded GPU receipts
using integer counters and serialized metadata only. It verifies dose, cursor
continuity, strict improvement for accepted moves, baseline/reference stability,
and the final evaluated snapshot's binding to the saved checkpoint. Snapshot
serialization order is explicit: the checkpoint uses a sorted JSON Value, while
the evaluator hashes the typed Snapshot; both representations are checked.
The checkpoint-zero serialization and completed-attempt checks both pass.
No model operation runs in this auditor. Reproduce from the extracted final
archive with `cargo run --locked --offline --manifest-path
docs/labs/a2-pilot-2026-10-14/receipt-audit/Cargo.toml --
<extracted>/rung0-pilot-20261014/pilot-attempt-1`, using a per-lab
`CARGO_TARGET_DIR`.

Checkpoint-zero P2 records 16 distinct full vector patterns and 16 occupied writer
cells over 4,096 token identities. P1 records 236 zero-count token identities in
the scheduled 410-token slice, which accounts for 1,440,407 / 2,425,925 masked
TRAIN targets. These are baseline and schedule observations; the admitted dose
continues unchanged. No final diagnostic result is inferred from them.

## Staging observation

Both requested Codex gates passed; see [the exact handoff](STAGING.md) and
[#2029 receipt6160885825](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6160885825).
Qualified pilot executable SHA256`d12877b6d8adc6a60445a014be682ab82459a781f760ac99a226b07335cbfcac`
returned the exact registered config and lineage digests on the restored
TRAIN/DEV/tokenizer files. Its metadata command created no CUDA context.

The separate D33 release tool is built from`ae9bfffc8` without the CUDA feature;
its SHA256 is`1dc78b6d0d58ad113730fe3c108e22501b47f87f30da653ea349a015741703bc`.
It is not the numerical pilot binary. Codex never invokes owner-release or
creates/copies the owner record; Claude's named delegation under D33 does not
extend to any other lab, configuration or hold.

Pod`vast-55864069`,1xRTX5090,$0.892/h, lease`codex/a2-pilot-20261014`;
working directory`/workspace/uor-r4/a2-stage`. Plan paths and bytes preserved;
only the parent of the run-state was created. The staged launcher is dormant
until Claude posts the delegated release receipt and the qualified binary
validates its identities. The entire pilot remains NOT_RUN at this staging
checkpoint; no release is inferred from build success or a FIT verdict.

Staging evidence is preserved at
`icloud:UOR-R4/results/codex/a2-pilot-staging-20261014.tar`,499,712bytes,
MD5`8016c93d0bd0fe0978109971534867db`, cloud-store download round-trip verified.
Nested staging-evidence.tgz SHA256
`0165dbb1174a425e5b16c3414cbec5e7cdabc98bf7cc1b98a78a4888bace3402`
matched the pod and local download. Corpus files remain in their pinned HF store;
this archive contains no owner-release record and no model output.

At the staging checkpoint, the next step was Claude's delegated release followed
by Codex's launch. That handoff is complete as recorded above. The pilot's bars/guard remain
NOT_DECIDED, Route sensitivity NOT_RUN, fresh transfer UNAVAILABLE/NOT_RUN.
Final provider lifecycle and complete diagnostic outcomes are recorded above.
