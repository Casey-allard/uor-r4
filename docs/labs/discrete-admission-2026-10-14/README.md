# Discrete pilot admission and reporting

Codex/Sol, 2026-10-14; References #2029 and #2032. Line: discrete read/writer
construction. No language headline movement or learning count reset.
Training and M2 remain **HELD**. Claude's prospective finite-control FIT carries
to the constrained C1/C2 corrections below. The pinned source passed all25
CUDA controls. Exact-head C1/C2 review found no source blocker; Claude's
final pilot-admission review remains separately required. No corpus pilot has run.

This is the admission diff required by Amendment 2 of the
[learning sheet](../../harness/prep/m1-discrete-rung0-training-2026-10-13.md).
The numerical baseline is `e7d926fbe38537f4faa90b817705c61a780778d6`. The
only CUDA change appends one sensitivity observation to the proposal receipt:
any active candidate's cost differs from CURRENT. This includes nonimproving
candidates and excludes Gate padding. It changes no objective, transport,
selection, tie order, writer, initialization, schedule or update law.

## Controller and admission

The registered pilot is seed1, Route, vocabulary4096, blocks4, context192,
writerK8, batch64, exactly1640 ascending VectorBlock coordinates. It is diagnostic
only: bars and guard are not decided, Route sensitivity is NOT_RUN, fresh
transfer is UNAVAILABLE/NOT_RUN, and the full two-seed run remains a later owner
decision. No frequency reordering or collision-free initializer is adopted.

`pilot CONFIG ADMISSION NEW_REPORT_DIR` starts this bounded run only with a
complete release manifest. `pilot-resume` uses the same inputs in a fresh report
directory and restores the registered latest durable checkpoint. `eval-pilot`
evaluates that registered trained checkpoint under the same source/data/admission
binding. Ordinary `train` and an unissued corpus-update request remain held.
Existing `eval-corpus` remains limited to untrained checkpoints; the new named
entry point makes the release provenance of trained evaluation explicit.

The release manifest binds the source baseline and reviewed commit, embedded
source/PTX/geometry/driver digests, executable digest, configuration, lineage,
FIT sheet, Claude review receipt and Casey's pilot-only D4 release receipt.
Missing or mismatched evidence refuses before CUDA initialization. These are
operator-supplied, hash-pinned operational receipts, **not cryptographic proof
of authorship**: operators must obtain and verify the real review and owner
instruction; the program never manufactures either. No released manifest is
shipped with this source. A build or a FIT verdict alone grants no permission.

Before the first apply, the controller writes admission identities, a full-dev
checkpoint-zero evaluation and a recovery snapshot. Step receipts preserve
attempted/sensitive/accepted observations. Checkpoints and reports use exclusive
files; the operational latest-checkpoint pointer is separate from immutable
report roots. Resume continues the canonical model/corpus/coordinate cursor.
Interrupted segments are replayed from their last durable snapshot and must be
charged as retries; their prior logs are retained. A crash may leave `active.lock`.
The explicit metadata-only command is:

```sh
discrete-stack pilot-unlock RUN_STATE EXPECTED_LOCK_SHA256 NEW_RECEIPT_DIR
```

It requires the original lock's SHA256, matching host and boot identities, and
a demonstrably dead PID. It checks liveness twice, refuses changed, live,
foreign or ambiguous locks, serializes recovery attempts, and preserves the
original lock with a receipt in a fresh disjoint directory. It does not kill a
process or release a training hold. A reboot or replacement pod changes the
boot/host identity and therefore refuses automatic recovery; preserve the old
state and establish worker liveness before an explicit operator disposition.
One controller owns the run state; recovery is never permission to remove
another live session's lock.

## P1 and P2

P1 summarizes the already-computed GPU response-masked TRAIN count table for
visited token IDs0..409: every count, sum, zero-count token IDs/count and exact
share numerator/denominator. No DEV count enters it. P2 counts byte-identical
full vectors and exact ordered endpoint-block pairs in completed GPU snapshots
at checkpoint0 and pilot end. This is the snapshot metadata operation expressly
allowed by the sheet; it does not evaluate model geometry on the host.

No accepted update is not the same as no sensitivity. The new CUDA observation
and per-kind counters preserve that distinction. A tied or insensitive pilot is
a dose/schedule observation; it does not close a mechanism or configuration.

## Verification and evidence scope

The [finite construction sheet](../../harness/prep/m1-discrete-admission-controls-2026-10-14.md)
registers the composed25 CUDA controls and sensitivity witnesses, with the same
11 fixture apply calls/8 disposable updates and no corpus training. One extra
read-only authored proposal distinguishes nonimproving-but-sensitive from all-tied.
Claude recorded [TEST FITNESS: FIT (prospective)](https://github.com/UOR-Foundation/uor-r4/pull/2421#issuecomment-6159846324)
and [confirmed its carry to the constrained C1/C2 correction](https://github.com/UOR-Foundation/uor-r4/pull/2421#issuecomment-6159902239).
The corrected source is `35a48f5863ed07711f70602ffa30d8d86209bf27`.
CUDA release build and all25 controls passed; see the observed receipts below.
Previous #2400 measurements do not qualify this changed proposal ABI or the
new controller.

Local metadata tests, formatting and ratchets passed at the corrected
source head. They are not CUDA runtime or learned-evaluation results. The pilot's
known-component estimate remains1.784167939h/$1.620024488 at the prior quoted rate;
controller, production apply, data/IO, setup/build, snapshots, retries and
preservation remain additional unmeasured cost. This task changes no estimate
into a measured all-in budget.

Next: protected delivery under the owner's exact-head review rule. Claude
confirms the final pilot-admission packet; Kimi binds the final
executable/PTX/source and resources; only Casey releases D4. Finite controls and
metadata checks do not establish corpus-controller or production learned-evaluation execution.

## Operator handoff

No released manifest is supplied. Kimi obtains exact identity values with
`discrete-stack build-identity CONFIG`.
This reads compiled/file metadata and does not create a CUDA context. The config
hash is over canonical serialized `DriverConfig`, not the pretty-printed input
file; this command also prints the admitted corpus lineage hash.

`ReleaseManifest` has schema `uor-r4.pilot-owner-admission/1`, scope
`m1-a2-seed1-vector0..1639-diagnostic-pilot`, an `execution` object (the fields of
`ExecutionIdentity` in `controller.rs`), SHA-pinned `sheet`, `claude_review` and
`owner_release` files, and an absolute dedicated `run_state_directory` outside
all report roots. Each file pin has `path` and `sha256`. The reviewed commit is
attested provenance; embedded content hashes and the executable digest provide
the actual binary identity checks.

The review record carries `reviewer: Claude`,
`verdict: APPROVE_A2_ADMISSION_DIFF`, the same `scope` and full `execution`, and
its real GitHub `receipt_url`. Kimi may prepare an `OwnerReleasePlan` with schema
`uor-r4.pilot-owner-release-plan/1`, status `HELD_OWNER_ACTION_REQUIRED`, the
complete `execution`, `config`, SHA-pinned `sheet` and `claude_review`, and the
absolute disjoint `run_state_directory`. This plan contains no owner decision.

[Casey's C1 decision](https://github.com/UOR-Foundation/uor-r4/pull/2421#issuecomment-6159895665)
is binding: **no lab writes, edits, copies or generates `owner.json`, or any
file that stands in for the owner-release record.** Casey alone runs this
command from his own terminal, only when he chooses to release the pilot:

```sh
env -u UOR_LAB discrete-stack owner-release PILOT_PLAN.json NEW_RELEASE_DIR
```

Labs must never invoke or automate this command. It requires an interactive
terminal, validates and displays the pinned plan, then asks Casey to type
`I am Casey; release D4 for the 1640-batch diagnostic pilot only`. It exclusively
creates the owner record and admission manifest in the new directory; the
owner record binds the exact execution and Claude review with an owner-terminal
receipt. It starts no model computation. The TTY and phrase are operational
safeguards, not authentication against a shared OS account. The separate M2
hold and all other training holds remain unaffected. No D4 release has occurred.

If initialization fails before a durable checkpoint0 exists, `pilot-resume`
refuses. Preserve that failed output and verify there was no apply before
preparing a fresh dedicated state directory and attempt. Each resumed segment's
wall time belongs to cumulative cost, even if its uncheckpointed work is replayed.

At corrected source `35a48f586`, all18 metadata tests PASS, including explicit
lock recovery; formatting, D23 ratchet1641, D25 ratchet93, claim-wording and
diff checks PASS. Owner-record
test fixtures now remain in memory and never write an owner-release file.
Finite CUDA build/runtime passed. Production apply/evaluation and corpus pilot remain NOT_RUN.

## Observed finite CUDA qualification

On2026-10-14 at03:13:38–03:13:53Z, the pre-pinned source at `35a48f586`
passed all25 controls once: [Route11](receipts/route.json),
[Flock6](receipts/flock.json), [Driver8](receipts/driver.json). Each suite
exited0 with no skipped/NOT_RUN controls. Driver dose was exactly11 fixture
apply calls/8 disposable accepted updates/0 corpus updates. Post-apply model
snapshots were not saved. The added sensitivity witnesses passed without
changing the learning law or adding an apply call.

[Build identities](receipts/build-identity.json) match the
[pre-execution sheet pins](../../harness/prep/m1-discrete-admission-controls-2026-10-14.md).
The exact11-entry PTX admission passed; all14 uploaded crate-file digests
matched the reviewed source. Build: Ubuntu24.04.3, Rust1.97.1,
nvcc12.8.93, `compute_120`, RTX5090 driver595.84. AOT release build40.04s.
[Suite wall times](receipts/timings.txt): Route7.461352087s,
Flock3.285109629s, Driver3.652831279s; sum14.399292995s.
These are finite construction timings, not production throughput.

Scope limits: no corpus controller run, production apply/evaluation,
learned-checkpoint result, fresh transfer or owner release. The finite controls
qualify the changed receipt ABI and regression behavior; metadata tests/source
review cover controller admission and persistence. Sensitivity means an active
cost changed, including an ineligible MAX cost, not that a move could improve.
The source review's nonblocking hardening suggestions remain future work:
explicit parameter-kind refusal in pilot counters, executable inode/mtime
recheck, output disjointness against prior report roots and a direct dose-cap
test. Foreign-host/boot lock recovery requires separate verified disposition.

Full archive: `admission-qualified-20261014` (iCloud Codex results; final receipt
and provider lifecycle cost recorded below). Its nested `qualification.tgz`
SHA256 is `7253b51e07fbed74301c866d3419176064f9c8f8f760276bba1f8479192c13b1`;
pod-to-local byte identity verified before archival. Training/M2 remain HELD.

Archive round-trip: `icloud:UOR-R4/results/codex/admission-qualified-20261014.tar`,
1,400,320bytes, MD5`6bd19fb4b87c388d49ff5f6195f8da4d`, verified by cloud-store
download before pod teardown. Build/control logs and executable/PTX/geometry
are retained; the small exact control receipts are also committed above.

[Provider ledger](receipts/provider.jsonl): up03:03:51Z, deleted03:16:50Z;
779seconds event-to-event at$0.815/h = $0.176356944 conservative incremental
charge. The provider adapter's uptime field is770seconds ($0.174319444);
the larger event interval is charged here to include setup/teardown margin.
This adds to, and does not reset, the existing shared cumulative ledger.
No numerical retries occurred. Metadata/review/orchestration and archive work
are separate from14.399292995s of finite control time. The estimate does not
include any unreported provider storage/network billing or future pilot work.

Final source review: an independent Codex reviewer inspected C1/C2, then
rechecked branch head`82fb14d12` against qualified source`35a48f586` and all
committed control receipts. No blocking source finding remained. This closes
the implementation corrections for protected delivery under the owner's
exact-head/self-review rule; it does not impersonate Claude's approval or
satisfy the distinct Claude review required in a future pilot manifest.
