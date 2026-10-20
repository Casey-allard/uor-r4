# Independent finite run3 actual-outcome review

**APPROVE the observed finite qualification only.** The retained executable passed its fixed U/R controls on the actually observed RTX 5090 / driver 595.71.05 environment. This is not a timing result, timing release, checkpoint0 comparison, cross-driver equivalence theorem, training release or general environment qualification.

## Evidence and execution boundary

Archive reviewed: `/Users/casey.allard/uor-r4-local/codex-cf1-reuse-finite-run3/reuse-finite-run3-archive`.

- Raw `finite-output/counterfactual-reuse-controls-record.json`: 426092 bytes, SHA256 `7f36d6f805bcb7082467373e3a9f17c23070feb96d9269c6f7e64ce0405567a5`.
- All 23 remote, 35 operator and seven input SHA-manifest entries pass. Archived 16-entry REVIEW manifest also passes. Remote/operator/input verification stdout is retained beside this review.
- Final pin SHA256 `072ca797cf2e0d883cfa5c566a6a99de7f688205b6334cbfcdc2a8b650d26154`; its only difference from reviewed pin `3fd484da479e3e5eb4e374289d5852ba0b069c0545bc455250ec129c121103b6` is execution_release_reference set to `https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6228335295`.
- Actual and expected no-context identity are byte-identical. Retained binary SHA256 remains `671917acb13b541c148b6b8122b92ae613ac0cdff8577a3f72fb6c05faa06209`, instrument source `5f533cbfd49ecef9a4ea2b217bb31a93002b3dc2`, composite `aca299c484d5434291f7f65cc9315d6b52b976952bcd68f39a45cd3341bf2825`. All image/report, fixture inventory and geometry bindings remain unchanged.
- Archived operators match the reviewed operators; expected identity and review pin match the sealed local packet. The hygiene completion log's actual SHA matches its recorded `c24129d5adc0d1c3bc54cba1b4091697d4758cd87465884e0078267497187d3d`.
- Actual GPU: NVIDIA GeForce RTX 5090, driver **595.71.05**, UUID `GPU-1daf8397-88e7-c3e5-e50c-0dbc1459969c`, reported 32607 MiB. Advertised offer45028230/machine8914/host18 driver selection was recorded before create and linked into up metadata.
- Command started `2026-10-17T02:11:02Z`, ended `02:11:06Z`, exit0. Invocation marker is exactly1. Full fixed finite dose was reached; this was not a zero-model-compute startup stop.

## Independent exact-integer receipt audit

The only change from the retained run2 audit source is its expected release URL, from6227632827 to6228335295. No raw record, algorithm, expected numeric criterion, fixture, mask or resource predicate was changed.

Original audit source SHA256: `d9f8838c0f47cf639c88343329ee35f096ee6a34574815a49fe6a1b3fb2ed146`.

Run3 `audit.rs` SHA256: `4ce3cffeaf0bdca0c48eeb8ec600bf4b8cbcda8ce880847b1b0f9742c26e6dc3`.

Direct metadata-only compiler invocation (no Cargo target tree or model execution):

```sh
/Users/casey.allard/.cargo/bin/rustc +1.97.1 --edition=2021 /Users/casey.allard/uor-r4-local/review_2551/reuse-finite-run3/audit.rs -L dependency=/Users/casey.allard/uor-r4/local/harness-target/debug/deps --extern serde_json=/Users/casey.allard/uor-r4/local/harness-target/debug/deps/libserde_json-97f3c5da5ec49b54.rlib -C debuginfo=0 -o /Users/casey.allard/uor-r4-local/review_2551/reuse-finite-run3/audit
/Users/casey.allard/uor-r4-local/review_2551/reuse-finite-run3/audit /Users/casey.allard/uor-r4-local/codex-cf1-reuse-finite-run3/reuse-finite-run3-archive
```

Both exited0. `audit.stdout.txt` SHA256 is `64b221b8ee66b503d130ea62f66599c73946535285bdcfaeb5dbf9d191f42ec3`:

```text
Route sensitivity nearest_address_margin_v1: proposal=[0,64,64,0,1]
Route sensitivity occupied_cell_rank_hinge_v1: proposal=[0,64,64,0,1]
PASS exact-u64 metadata audit: 32 U/R pairs, 56 success/8 status8 attempts, 512 R lanes, 414 producer rows, 16 typed refusals, 18 strict resource admissions, unchanged state/epoch/apply, zero endpoints/timing.
```

The audit parses loss, mask and status words as exact u64 values, including MAX and full-width boundary masks. It verifies the complete record-kind inventory, every fixed policy/fixture/parameter pair, all16 U/R loss words, all five successful proposal words, error/status parity, mixed-row equivalence to its eligible reference, candidate3 VectorBlock tie choice, Route identity tie, and the Route sensitivity receipt above. It checks all32 R raw160-word traces and decoded row/lane records, producer partitions/counts, full consumer/eligible masks, the virtual-ineligible early-exit masks, rows16/17/64 boundaries, 16 exact typed refusal receipts and their unsubmitted trace guards, unchanged before/after state and epoch0, apply0, and no readback errors. There are no extra record kinds or endpoint/timing calls.

All18 resource receipts satisfy the existing exact runtime/AOT equality. U reports88 registers,21536 local bytes,shared0,maxthreads640; R reports115 registers,21536 local bytes,shared512,maxthreads512. Their own AOT reports have those same registers/stack and zero spill loads/stores. Thus R512 is observed directly on this driver; no ELF subtraction or predicate waiver was used.

## Preservation, closure and accounting boundary

The operator lease log records the successful full-cloud tar receipt at line143 before lease release at144 and pod deletion at145. The pinned tool reports **7470080 bytes**, full remote-readback MD5 `bb98d16c9a6a442b6121bf767a62e095`, at `icloud:UOR-R4/results/codex/reuse-finite-run3-archive.tar`. This review verifies the retained pinned-tool receipt and ordering; it does not perform another cloud transfer.

Live ledger closure records lease release at `02:11:57Z`, down at `02:12:03Z`, provider uptime109s. Final retained fleet status reports zero running/countable pods and $0/h; operator ends `LEASE_RESULT=0`. Precreate selection retained raw quote `$0.9083333333333332/h` and legacy admission `$0.908/h` separately. Up02:10:11Z to down02:12:03Z is112s: a conservative timestamp lifecycle estimate at the raw quote is `$0.028259259`; provider109s at the raw quote is `$0.027502315`. The legacy down-ledger109s multiplied by rounded admission price yields `$0.027492222`, a distinct estimate. These are estimates, not a provider invoice. Parent owns final cumulative reconciliation. Separate PM/storage/transfer USD remain unmeasured.

The tiny direct audit compile kept local free space at31998168KiB, above31457280KiB. No new Cargo target tree was created.

## Scope and next dependency

This accepts only the fixed finite controls and strict resources for this unchanged artifact on the recorded hardware/driver/library environment. It does not establish arbitrary model correctness, isolate driver causality, compare runtime performance across drivers, exercise checkpoint0 or repair the source-bound unreachable-fault/stale-epoch limitations. Constructor validation and all admitted finite GPU work actually ran; no training/apply occurred.

Protect-deliver the accepted finite record and reconcile current status/cost before any successor. A separately reviewed timing packet must prospectively bind the actual qualified environment and receive a new distinct execution release. No automatic retry, timing allowlist expansion or timing execution follows from this review. Prior blocked records and consumed releases remain preserved. Training/M2/Stage B remain HELD; Q4 remains NOT_EXECUTABLE.

Reviewer operations were retained-file/git-object inspection, SHA/cmp verification and exact-integer readback analysis only. No provider/SSH/pod/model/reference execution, source mutation or external publication was performed.
