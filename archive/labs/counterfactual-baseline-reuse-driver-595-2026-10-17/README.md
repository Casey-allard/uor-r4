# Baseline-reuse finite environment qualification — 17 October 2026

**Delivery and subsequent setup:** this record is protected in #2568 at
`66154156dad423f5bf4c822236509f6ba659ab8f`, with full reviewed-tree equality
and completed own cleanup/hygiene. The separately released
[timing2 attempt](../counterfactual-baseline-reuse-timing-uuid-blocker-2026-10-17/README.md)
stopped on a different actual GPU UUID before any binary. This finite pass
remains valid at its recorded scope; timing is still unmeasured.

**FINITE_CONTROL_PASS on RTX 5090 / driver 595.71.05, independently reviewed and
accepted at the exact observed finite scope.** One unchanged finite-control invocation recorded **32 exact
U/R pairs, 64 proposal attempts (56 successes and eight expected status-8
outcomes), 16 typed admission refusals and zero apply**. This is environment
qualification evidence for the exact artifact below. It is not a timing result
or a release to execute timing.

The [raw record](counterfactual-reuse-controls-record.json) is **426,092 bytes**,
SHA256 `7f36d6f805bcb7082467373e3a9f17c23070feb96d9269c6f7e64ce0405567a5`.
The [independent actual-outcome review](outcome-review.md) approves these exact
bytes; [PM acceptance6228377410](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6228377410)
records that bounded acceptance.
The earlier [580.95.05 finite result](../counterfactual-baseline-reuse-finite-pass-2026-10-17/README.md)
remains separate evidence. The [timing startup refusal](../counterfactual-baseline-reuse-timing-startup-2026-10-17/README.md)
correctly rejected 595.71.05 before instrument invocation and remains a sealed
zero-dose blocker, not a timing observation.

## Prospective authority and exact identity

The [pre-registration](../../harness/prep/m1-counterfactual-baseline-reuse-2026-10-16.md)
retains design FIT6226136376. The finite-only prospective environment review
[FIT6228300063](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6228300063)
and [readiness6228307955](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6228307955)
preceded the distinct [execution release6228335295](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6228335295).
The [review pin](review-pin.json), SHA256
`3fd484da479e3e5eb4e374289d5852ba0b069c0545bc455250ec129c121103b6`,
and [final pin](finite-pin.json), SHA256
`072ca797cf2e0d883cfa5c566a6a99de7f688205b6334cbfcdc2a8b650d26154`,
differ only in execution_release_reference. Source, images, fixtures, dose and
resource predicates were unchanged; only this new finite environment was released.
[Hygiene completion](hygiene-complete.json) preceded allocation.

| Identity | Executed value |
| --- | --- |
| Construction source | `5f533cbfd49ecef9a4ea2b217bb31a93002b3dc2` |
| Protected source delivery | `95ee863c34e6120689e01774f59ced6097736d9c` (#2562) |
| Source delivery tree | `3e3a86954d0e42564b14f5711eaf85807ed101d5` |
| Executable SHA256 | `671917acb13b541c148b6b8122b92ae613ac0cdff8577a3f72fb6c05faa06209` |
| Compiled composite SHA256 | `aca299c484d5434291f7f65cc9315d6b52b976952bcd68f39a45cd3341bf2825` |
| Standalone source SHA256 | `d0b4c317334fe6afd3259bc3045f1d74b7c72f997a9df5f45bc18617ce97af32` |
| Fixture inventory SHA256 | `e4afe3d46e82a88ceaf6ee315708af04efd30dde19f4c84f1aea7071b1f6c7c2` |
| Geometry SHA256 | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| GPU | NVIDIA GeForce RTX 5090, 32,607 MiB |
| GPU UUID | `GPU-1daf8397-88e7-c3e5-e50c-0dbc1459969c` |
| Actual driver | **595.71.05** |

The [actual identity](actual-identity.json) equals the expected identity byte for
byte. The pin retains every ordinary/decomposition/reuse source, PTX, cubin and
own-AOT report identity. Construction remains Rust/Cargo1.97.1, NVCC12.8.93,
sm_120, `compiled/release-uor-r4-discrete-07f94cfaa101289b`; no rebuild or on-lease
repair occurred. Finite controls use literal fixtures, not the original checkpoint
or its bridge. Checkpoint/config identities in the pin name the future comparator;
this observation establishes no checkpoint0 parity.

## Finite observations and resource admission

Both `nearest_address_margin_v1` and `occupied_cell_rank_hinge_v1` cover the nine
fixed cases: ineligible, eligible, mixed, tie, virtual-ineligible, causal Route,
and 16/17/64-row boundaries. The raw executed inventory reports 64 attempts,
56 successes, eight expected status8, 16 admission refusals and zero endpoints.
The 32 control checks report PASS, retaining all sixteen candidate losses,
statuses, five proposal words where present, raw R audits, producer/consumer
masks and before/after state/epoch. The record retains 32 decoded-control audits,
18 fixture starts and 18 resource admissions.

The independent [exact-u64 Rust readback audit](audit.rs) and its
[stdout](audit.stdout.txt) verify **32 U/R pairs, 512 R lane records, 414 producer-row
outcomes, 16 typed refusals and 18 strict resource admissions**. It checks every
loss/status/proposal word, full-width masks, producer partition/count, consumer
and virtual masks, tie choices, mixed eligibility and unchanged state/epoch.
This is analysis of retained GPU evidence, not CPU model/reference execution.
The [source delta](audit-source-delta.patch) from the prior accepted audit changes
only the expected execution-release URL, from6227632827 to6228335295; no numeric
criterion, algorithm or fixture changed. Original source is retained in the
[prior finite record](../counterfactual-baseline-reuse-finite-pass-2026-10-17/audit.rs).
The reviewer compiler/audit command and exit0 receipts are described in the
outcome review; no audit rerun was needed for this documentation delivery.

The causal Route fixture reports `[0,64,64,0,1]` under both policies: sensitivity
with equal **64→64** loss, **not an improving update**. VectorBlock improving
ties retain candidate3 over candidate12; Route ties retain identity. Producer
and consumer masks remain distinct, including virtual-ineligible early exit.

Existing source-scoped limitations remain: the typed
host refusals do not establish CUDA fault-path parity; no corrupt-ABI injection,
apply-created stale epoch or invented overflow control was introduced. The
128 prior immutable endpoint evaluations were not rerun. No warmup, timing,
training, seed2, extra dose or CPU model/reference computation occurred.

All 18 U/R resource admissions report within_allowance=true under the unchanged
own-AOT predicate:

| Resource | U | R |
| --- | ---: | ---: |
| Runtime registers | 88 | 115 |
| Runtime local bytes | 21,536 | 21,536 |
| Runtime static shared bytes | 0 | **512** |
| Runtime maximum threads/block | 640 | 512 |
| Own-AOT spill stores / loads | 0 / 0 | 0 / 0 |
| Launched threads/block | 16 | 16 |
| Dynamic shared bytes | 0 | 0 |

Raw R shared512 is observed directly, separate from the retained ELF1536 and
reserved-cap1024 construction values. No subtraction, predicate change or waiver
was used. U and R run on the same actual GPU and driver. This repeats the finite
bar on the observed environment; it is not a cross-driver equivalence theorem,
a general 595-series qualification or evidence that an earlier driver caused a
prior resource discrepancy.

## Selection, lifecycle and preservation

The exact-driver selector delivered in #2567,
`48478154ea4f0ff3d2b6fd28e76fd2292d7a75ed`, selected only **595.71.05**. The
[session ledger](session-ledger.jsonl) retains the precreate selection at
02:10:10Z: offer45028230, machine8914, host18, Vast Alberta, CA,
**raw quote $0.9083333333333332/h**, **admission-rounded price $0.908/h**, with query
and selection timestamps. Creation is recorded02:10:11Z, SSH ready after34s;
podvast-56330183, sessioncodex-cf1-reuse-finite-run3. The delivered runner was
materialized into the packet, rather than using the owner's older checkout:
runnerSHA256 `1ae8e6021dc6853d2cf62571842c9d429b5d319acad8aa7d18a42ea889cc8fd9`;
new helperSHA256 `cb524800bf583d1bffc956516f430e74128dc999224b5970b294588521cdc13d`.
The prebinary [GPU receipt](gpu.csv) confirms actual595.71.05. No fallback,
placement wait/retry, source patch or allowlist widening occurred.

[Started](started.txt) **02:11:02Z**, [ended](ended.txt) **02:11:06Z**,
[exit0](command.exit); one `reuse-controls --pin finite-pin.json` invocation.
The [remote operator](run-remote.sh), [lease operator](lease.sh),
[checked plan](checked-plan.json) and [execution plan](execution-plan.json)
preserve the exact commands and contract. Operator exit0 is retained in the
[final operator log](operator-final.log). Local and remote disk exceeded30GiB;
remote preflight157,270,100KiB. The strict fleet/rate/ownership admission passed.

Complete archive: `icloud:UOR-R4/results/codex/reuse-finite-run3-archive.tar`,
**7,470,080 bytes**, full cloud-readback MD5
**bb98d16c9a6a442b6121bf767a62e095**. The [cloud receipt](cloud-readback.log) and
final operator log show complete SHA verification and full MD5 readback before
lease release02:11:57Z and deletion02:12:03Z; [final fleet](fleet-after.log)
records zero pods. The [final session ledger](session-ledger-final.jsonl) includes
renew/release/down events after archive capture. The separately retained earlier
[session ledger](session-ledger.jsonl) remains the prearchive snapshot.
The original archive [remote](archive-SHA256SUMS),
[operator](archive-OPERATOR-SHA256SUMS) and [input](archive-INPUT-SHA256SUMS)
manifests cover **23 remote, 35 operator and seven input entries**. They name
paths inside the complete archive, not this selected evidence directory.
[EVIDENCE-SHA256SUMS](EVIDENCE-SHA256SUMS) covers the copied evidence here;
raw bytes, including cosmetic literal escapes in the original packet, are
preserved without normalization. The binary and full cloud tar are not duplicated.

[PM accounting](cost.json): creation02:10:11Z to deletion02:12:03Z is **112s**;
at the raw quote$0.9083333333333332/h, the lifecycle estimate is **$0.028259259**.
The provider's109s uptime gives **$0.027502315** separately at that same raw rate.
The legacy admission-rounded$0.908/h remains explicit in the final ledger; it
is not substituted for the raw quote in these estimates. Cumulative **37 closed
Codex leases: $3.446524815**, including unchanged startup/construction carry
**$0.947947500**, plus separate PM receipts. Previous cumulative estimates remain
unchanged. Estimates are not provider invoices; storage/transfer USD UNMEASURED.
Local32,015,996KiB and remote157,270,100KiB passed the30GiB floor.

## Decision and next dependency

**Accept the fixed finite controls and strict resources for this exact artifact
and recorded environment**, per the independent outcome review and PM acceptance.
Complete protected delivery, fresh-main verification,
owned branch/worktree cleanup and hygiene before any successor lease. Only
independent actual acceptance and protected finite delivery may qualify this
exact recorded environment for a future separately reviewed timing packet.
Timing still requires distinct readiness and execution release; this record
grants neither. No automatic timing or rerun follows this pass.

No checkpoint0 comparison, causal speedup, p95, training rate, language capability,
energy result, production activation or general driver-family claim follows.
Training/M2/Stage B **HELD**; Q4 **NOT_EXECUTABLE**; seed2 excluded. Line: M1
qualification/cost tooling; count0/3 and suspended headline37/232 unchanged.
