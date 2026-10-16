# K16 qualification restart 4 — mandatory fixture clippy gate halted

**STEP0_BLOCKER_TO_FIX / NOT YET PROMOTED.** At merged source
`5fc68a9a6a4f2c1dcff3f73790865e5978e907a5`, all three CUDA release builds passed.
The fixture runner's mandatory clippy gate then exited 101 on
`clippy::identity_op`. Execution stopped at that first failed gate; no patch,
retry, G-Pre, C-R fixture, timing, model computation or corpus apply followed.
This is a D22/D31 tooling blocker, **not a mechanism negative**.
Line: exact addressed memory; count 0/3 unchanged; suspended M1 headline
37/232 unchanged. Codex/Sol owns this record; Kimi dispatches exact-head review.

## Authority and identity

- [Owner qualification release](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201365250).
- [PM order](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6205684341):
  step 0 → G-Pre → C-R1..C-R6 → checkpoint-0-only timing.
- [FIT sheet](../../harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md),
  [runbook](../admission-rank-qualification-runbook-2026-10-15/README.md),
  [D-Packet](../admission-redirect-packet-2026-10-15/README.md).
- [Pre-lease acknowledgment](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6208084436)
  and [pod acknowledgment](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6208114898).
- #2536 merged as the source above. Empty instrument-directory diff against
  approved `11e662a812f2e19c249a816346cb2a4f2828b3ee` verifies byte identity.
  The earlier [19.70-second CUDA check](https://github.com/UOR-Foundation/uor-r4/pull/2536#issuecomment-6207818632)
  and [confined approval](https://github.com/UOR-Foundation/uor-r4/pull/2536#issuecomment-6207821947)
  remain source/compile evidence, not clippy or runtime qualification.
- Rust/Cargo 1.97.1, NVCC 12.8 V12.8.93, `UOR_NVCC_ARCH=compute_120`;
  RTX 5090, driver 580.105.08, 32,607 MiB.
- All five frozen train/dev/tokenizer file SHA256s matched locally and on pod;
  [data-file-sha256.txt](data-file-sha256.txt) retains the pins. No fresh-transfer
  rows were staged or scored. Source remained unchanged after the attempt.

## Step-0 result and first failure

[step0-commands.jsonl](step0-commands.jsonl) records command exit statuses and
start/end times. Full command lines and all six compiler transcripts are sealed
in the archive. `--all-targets` compiles test targets; it did not execute tests.

| Gate, in actual order | Exit | Wall seconds |
| --- | --- | --- |
| discrete CUDA release build | 0 | 24 |
| timing CUDA release build | 0 | 18 |
| fixture CUDA release build | 0 | 17 |
| discrete CUDA clippy, all targets | 0 | 15 |
| timing CUDA clippy, all targets | 0 | 15 |
| fixture CUDA clippy, all targets, `-D warnings` | **101** | 14 |

Passing commands emitted warnings; exit 0 is not a claim of warning-free code.
Total step-0 command wall was 103 seconds. The last command was:

```sh
cargo clippy --locked --features cuda --all-targets \
  --manifest-path docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml \
  -- -D warnings
```

The [full failed-gate transcript](fixtures-clippy.log) reports:

```text
error: this operation has no effect
   --> src/main.rs:564:51
564 |                 parameter: Parameter::VectorBlock(3 * 4 + 0),
    |                                                   ^^^^^^^^^ help: consider reducing it to: (3 * 4)
    = note: -D clippy::identity-op implied by -D warnings
```

This excerpt omits diagnostic formatting; the linked log preserves exact bytes.
The declared fixture gate requires `-D warnings`; it was not weakened.
[Immediate halt notification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6208146828)
routes the bounded repair to PM. No instrument source changed in this record.

## Downstream disposition

| Stage | Status |
| --- | --- |
| Step 0 overall | FAIL / BLOCKER_TO_FIX |
| G-Pre, both seeds and K8/K16 | NOT_RUN; no admission ceiling measured |
| C-R1, C-R2, C-R3, C-R4, C-R5, C-R6 | Each NOT_RUN |
| 100-batch checkpoint-0 timing | NOT_RUN; K16 rate UNMEASURED |
| Q4 timing | NOT_EXECUTABLE |
| Training / corpus apply / model-state mutation | None |

The prepared `gates.sh` is archived for recovery but **was never invoked**.
There is no qualification PASS, language result, KEEP/REJECT decision or promotion.
Training/M2/Stage B remain HELD. The C-R6 receipt repair is not reopened by this
different clippy finding.

**Next:** [PM ruling 6208187624](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6208187624)
routes the bounded fixture clippy repair to DeepSeek. Its pre-merge exact-head
pod transcript must cover both `cargo check --features cuda` and
`cargo clippy --features cuda --all-targets -- -D warnings`, followed by Codex's
confined review and protected merge. A separately authorized fresh qualification attempt starts at
step 0 again; it must not skip the failed gate or silently rerun on patched pod
source. No model or acceptance bars change.

## Archive, teardown and storage

- Archive: `icloud:UOR-R4/results/codex/codex-k16-qualification-20261016-4.tar`.
- **162,304 bytes**, MD5 `3f2ec85f8ff9963697c6a02991736ed5`.
  Upload/download MD5 round-trip completed **before** release/delete.
- All 26 manifest files verified; SHA256SUMS digest:
  `2c0dbbb970c5c3c2379d65a563bebbfa15bfa8ee549258c92aac40cf3928568c`.
- Contains full setup/job/build/clippy transcripts, exact source head and hashes,
  empty tracked-source diff, staged-data verification, scripts, command exits,
  NOT_RUN disposition and prior-cost ledger. It does not claim a GPU result.
- Restore with `cloud-store fetch codex-k16-qualification-20261016-4 DEST`.
- Pod `vast-56199663`, session `codex-k16-qual-20261016-4`, one Vast RTX 5090,
  South Korea, $0.563/h. Created 04:28:07Z; released 04:34:57Z; deleted
  04:35:14Z on 16 October UTC. Full wall 427 seconds (7m07s), reported provider
  uptime 414 seconds. Post-delete fleet: **0 pods, $0/h**.
- Ephemeral allocation deleted; no retained paid pod storage. Existing iCloud
  plan used; incremental storage/egress invoice is UNMEASURED, not asserted zero.
  Eight Codex archives in this episode total 640,512 bytes; peer archive sizes
  are not inferred. Final teardown/cost is in [receipt.json](receipt.json),
  after the pre-teardown immutable archive was sealed.

## Full episode cost, including repair leases

Basis: actual shared `uor-pod down` rate × provider uptime /3600, not invoices.
The prior record's $0.567979 covered the initial halt, five setup/check leases
and restart 3. Three subsequent #2536 checks add $0.149983, making pre-restart-4
overhead **$0.717961**, rather than treating the rough ~$0.60 as an exact total.
This attempt adds **$0.064745**; cumulative episode estimate **$0.782706**.
The PM routing's approximate three-minute/$0.03 description is not the full
lease cost: setup, source/data transfer, sealing and teardown are included here.

| Pod | Lab / session | Uptime seconds | USD/hour | Estimated USD |
| --- | --- | --- | --- | --- |
| 56182561 | codex / codex-k16-qual-20261016 | 447 | 0.815 | 0.101196 |
| 56184953 | deepseek / k16cuda | 136 | 1.775 | 0.067056 |
| 56185442 | deepseek / k16cuda2 | 121 | 1.775 | 0.059660 |
| 56186258 | deepseek / k16cuda3 | 144 | 4.696 | 0.187840 |
| 56187939 | codex / codex-k16-cudacheck | 303 | 0.908 | 0.076423 |
| 56189413 | codex / codex-k16-cudacheck-2 | 183 | 0.908 | 0.046157 |
| 56190701 | codex / codex-k16-qual-20261016-3 | 177 | 0.603 | 0.029647 |
| 56194344 | codex / codex-k16-cudacheck-3 | 515 | 0.603 | 0.086263 |
| 56196329 | codex / codex-k16-cudacheck-4 | 154 | 0.708 | 0.030287 |
| 56197212 | codex / codex-k16-cudacheck-4 | 170 | 0.708 | 0.033433 |
| 56199663 | codex / codex-k16-qual-20261016-4 | 414 | 0.563 | 0.064745 |

The current lease's conservative full-wall estimate is separately
**$0.066778** (427 seconds); the cumulative table consistently uses provider
uptime and does not mix denominators. Prior check comments also reported their
full-wall estimates; their raw source receipts remain preserved. No failed/setup
attempt is omitted. No `codex-k16-qual-20261016-2` lease occurred.

STATUS/ROADMAP/#2028 headline and public README capability claims are unchanged.
Packet, runbook, fixture record and newest-first current-state entry point here.
