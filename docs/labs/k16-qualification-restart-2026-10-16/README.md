# K16 qualification restart — G-Pre/C-R6 receipt-interface blocker

**PRE_EXECUTION_SOURCE_BLOCKER / NOT YET PROMOTED.** The restart stopped before
any build or model command: G-Pre's producer and C-R6's consumer disagree on the
top-level metrics key. This is static source evidence, **not an executed fixture
failure, measured ceiling or mechanism negative** (D22/D31). Line: exact addressed
memory; count 0/3 and suspended M1 headline 37/232 unchanged. Codex/Sol owns this
record; Kimi dispatches its exact-head review.

## Authority and exact source

The [PM confirmation](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6207017795),
[execution order](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6205684341)
and [owner release](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201365250)
authorize step0 build/clippy → G-Pre → C-R1..6 → checkpoint0 timing, not training.
Fresh main was `bb536cf68720ccfca102994b372a485005e5847e` (#2533). An empty
instrument-directory diff against checked/approved
`b55c02767feb16c7329babda95609df0455a9fd5` verified merge identity.

The earlier E0609/E0425 approval remains valid within its confined scope. This
different defect was found while checking the prepared execution script against
its producer and consumer, after the fresh lease request but before execution.

## Deterministic source mismatch

| Exact-source boundary | Behavior |
| --- | --- |
| `crates/uor-r4-discrete/src/training.rs:962` | `PreparedDriver::evaluate` emits `"aggregate": aggregate`, not a top-level `evaluation` |
| `crates/uor-r4-discrete/src/main.rs:81` | `eval-corpus` passes `driver.evaluate(&model)` through `report_command`, which writes its JSON without a wrapper |
| Fixture instrument `src/main.rs:1031` | Opens `g-pre-k8-A-seed1-eval/report.json`, then reads `k8_val["evaluation"]` at line1032 |
| Fixture instrument `src/main.rs:1055` | Reads `val["evaluation"]` for all four K16 G-Pre reports too |

Thus the K8 guard sees null metrics and cannot match the sealed counts regardless
of G-Pre's actual outcome; the K16 reader also cannot obtain
`target_admitted_rows`. No receipt was rewritten to manufacture the missing key.
No GPU work was spent on a control known to be invalid, and no fixture was run
to manufacture a redundant failure. The exact three source files and empty
merge-identity diff are in the archive.

**Next:** PM routes the bounded receipt-schema correction, with a metadata-level
regression control using the actual producer shape and a pod-side CUDA check if
CUDA-gated source changes. Exact-head review and protected merge precede a fresh
lease starting at step0. Keep the sealed counts and >=4,894 admission bar; do not
relax controls or silently transform sealed records. The
[notification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6207053650)
records the blocker; DeepSeek [acknowledged it](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6207073030)
and offered repair pending PM routing.

## Execution disposition

One Vast RTX5090, `vast-56190701`, session `codex-k16-qual-20261016-3`, South Korea,
$0.603/h. SSH ready after97s. The only remote command collected date, GPU identity
and checkout absence: `NO_SOURCE_CHECKOUT_NO_EXECUTION`. Driver595.58.03,
32,607MiB. No source clone, toolchain setup or model command ran on this pod.
Source evidence comes from pinned local Git, not a fabricated pod transcript.

| Stage | Result |
| --- | --- |
| Merge identity | Verified |
| Receipt-interface source preflight | BLOCKER_TO_FIX |
| Step0 CUDA build and clippy | NOT_RUN on this restart |
| G-Pre, both seeds, K8/K16 | NOT_RUN; admission ceiling unmeasured |
| C-R1, C-R2, C-R3, C-R4, C-R5, C-R6 | Each NOT_RUN |
| Synchronized100-batch checkpoint0 timing | NOT_RUN; K16 rate UNMEASURED |
| Q4 timing | NOT_EXECUTABLE |
| Training, corpus apply, model-state mutation | None |

The [initial attempt](../k16-qualification-2026-10-16/README.md) remains a separate
CUDA-build failure. Codex's two pre-merge checks remain separate:
[E0425 failure at d41301562](https://github.com/UOR-Foundation/uor-r4/pull/2533#issuecomment-6206777066),
then [PASS at b55c02767](https://github.com/UOR-Foundation/uor-r4/pull/2533#issuecomment-6206916632)
and [confined APPROVE](https://github.com/UOR-Foundation/uor-r4/pull/2533#issuecomment-6206929805).
None is runtime qualification. Q4 and training/M2/Stage B holds remain unchanged.

## Archive and teardown

- Archive: `icloud:UOR-R4/results/codex/codex-k16-qualification-20261016-3.tar`.
- **158,208 bytes**, MD5 `9b2b8a5923ec701585dcb61f9a24ab43`; cloud-store download
  round-trip verified **before** release/delete. All7 manifest files verified.
- SHA256SUMS digest: `55fd57b2af9604ac02b7cf8cd94d653e71ea7a9a152eba7aa32976dd115cd639`.
- Contains exact source files, approved-source diff, pod metadata, lease events
  and NOT_RUN disposition. Restore: `cloud-store fetch codex-k16-qualification-20261016-3 DEST`.
- Created02:59:44Z; released03:02:37Z; deleted03:02:53Z,16October UTC.
  Fleet then **0 pods**. Wall189s; provider uptime177s.
- Default150GB ephemeral allocation destroyed; no retained paid pod storage.
  Existing iCloud plan used; storage/transfer invoice charges **UNMEASURED**, not zero.

## Complete qualification-chain overhead,16October

The shared `uor-pod down` ledger supplies hourly rate × provider uptime /3600.
These are rental estimates, not provider invoices; raw values are preserved in
[receipt.json](receipt.json). Metadata accounting is not model computation.

| Attempt / pod | GPUs | Uptime(s) | Reported$/h | Estimated$ |
| --- | --- | --- | --- | --- |
| Initial Codex halt /56182561 | 1 | 447 | 0.815 | 0.101196 |
| DeepSeek k16cuda /56184953 | 2 | 136 | 1.775 | 0.067056 |
| DeepSeek k16cuda2 /56185442 | 2 | 121 | 1.775 | 0.059660 |
| DeepSeek k16cuda3 /56186258 | 2 | 144 | 4.696 | 0.187840 |
| Codex failing check /56187939 | 1 | 303 | 0.908 | 0.076423 |
| Codex passing check /56189413 | 1 | 183 | 0.908 | 0.046157 |
| This restart /56190701 | 1 | 177 | 0.603 | 0.029647 |

**Pre-merge check/setup overhead: $0.437135. Initial halt + all pre-merge overhead
+ this restart: $0.567979**, summed before rounding. The three DeepSeek attempts
never ran the requested check, per their published corrections; the shared
ledger records two GPUs each. Actual rate/uptime receipts supersede the PM's
rough $0.23 pre-merge subtotal, without changing scientific disposition. No cost
is omitted because setup failed. This subtotal covers this chain, not all
project spending. No `codex-k16-qual-20261016-2` lease occurred.

Codex's pre-merge archives under the same cloud prefix are46,592bytes
(`codex-k16-cudacheck-20261016.tar`, MD5 `321bbe6188d670a48697d7b5d1c4400a`) and
45,056bytes (`codex-k16-cudacheck-20261016-2.tar`, MD5 `98f990ba8c7f8b7e62bd66b19ecd7ee1`).
Both were round-trip verified before teardown. With the initial97,280-byte
archive and this158,208-byte archive, four Codex archives total347,136bytes.
Peer archive sizes are not inferred. No language or mechanism conclusion follows.
