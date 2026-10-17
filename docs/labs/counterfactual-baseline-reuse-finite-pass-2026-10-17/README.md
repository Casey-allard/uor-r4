# Baseline-reuse finite controls — 17 October 2026

Subsequent environment qualification: the same retained executable passed the
unchanged finite controls on [RTX5090/driver595.71.05](../counterfactual-baseline-reuse-driver-595-2026-10-17/README.md)
under a new prospective review and separate release. Both finite observations
retain their own environment and artifact scope. Timing still requires its own
reviewed packet and distinct release; no cross-driver performance claim follows.

Follow-up: the separately released [timing startup](../counterfactual-baseline-reuse-timing-startup-2026-10-17/README.md)
stopped before any instrument command because the allocated driver595.71.05
was outside the admitted580.95.05. This finite result remains accepted;
checkpoint0 timing and speedup remain NOT_RUN.

**FINITE_CONTROL_PASS, independently reviewed and accepted at the pinned artifact
scope.** One invocation compared the ordinary U path with call-local baseline
reuse R: **32 exact U/R pairs, 64 proposal attempts (56 successes and eight
expected status-8 outcomes), 16 typed admission refusals, zero apply**. Every
fixture preserved state and epoch. This qualifies the finite comparison on the
observed RTX 5090 / driver **580.95.05**; timing and checkpoint0 parity remain
NOT_RUN for R.

The [raw record](counterfactual-reuse-controls-record.json) is **426,092 bytes**,
SHA256 `c6a5b3f857d77bf979ed439e61575f04fce2f398b9854286681eac53203cd4f4`.
The [independent outcome review](outcome-review.md) and
[PM acceptance6227712402](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6227712402)
cover these exact bytes. The prior [driver-blocked attempt](../counterfactual-baseline-reuse-controls-2026-10-17/README.md)
remains a separately sealed zero-dose operational blocker.

## Prospective authority and executed identity

The [design sheet](../../harness/prep/m1-counterfactual-baseline-reuse-2026-10-16.md)
received design FIT6226136376. Replacement finite readiness6227360052 reviewed
the [review pin](review-pin.json), SHA256
`27975a2e3a2d6c20278c8778c37f9ec88b7807df116068ed3a08955be0335b30`.
The separate [execution release6227632827](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6227632827)
preceded rental and admitted exactly drivers580.95.05 or580.105.08. The
[final pin](finite-pin.json), SHA256
`e3f5ff59036a164cfb15ab18cbc1be41ad1e409ebe00cebe5f3a34463504a7c4`,
differs only by the new release URL. No source, image, dose or resource predicate
was changed after review.

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
| GPU UUID | `GPU-2ba3170b-c80a-3c40-258d-d84f26b8eca3` |
| Actual driver | `580.95.05` |

The [actual identity](actual-identity.json) equals the expected identity byte for
byte; all ordinary/decomposition/reuse PTX, cubin and own-AOT report hashes are
retained in the pin. Construction used Rust/Cargo1.97.1, NVCC12.8.93, sm_120,
selected `compiled/release-uor-r4-discrete-07f94cfaa101289b`; no rebuild occurred
in this finite lease. Source paths remained byte-identical after AGY #2563
merged. Checkpoint/config identities in the pin name the future timing comparator;
the finite phase uses literal fixtures and does not load that checkpoint.

## Finite observations

Both policies (`nearest_address_margin_v1`, `occupied_cell_rank_hinge_v1`)
covered all nine fixture cases: ineligible, eligible, mixed, tie,
virtual-ineligible, causal Route, and16/17/64-row boundaries. The independent
[Rust readback checker](audit.rs) uses exact u64 metadata values, including
MAX losses and64-row masks; its [retained stdout](audit.stdout.txt) records
PASS. It reads existing GPU evidence and performs no CPU model/reference run.

- Every pair has identical sixteen losses and statuses. All28 successful pairs
  have identical five proposal words; all four refused pairs agree on missing
  proposals and typed errors.
- All512 R lane records and414 producer-row outcomes agree with raw decoded
  masks/counters. Producer partitions are disjoint and cover every admitted row
  once, including the16/17/64 boundaries. Consumer/virtual masks match the
  fixture expectations. These are audit observations, not a speedup estimate.
- Mixed eligibility preserves the eligible reference. For virtual-ineligible,
  producers cover both rows, identity consumes both, and nonidentity candidates
  terminate after the first row. VectorBlock improving ties retain candidate3
  over candidate12:448→352 for nearest-address and192→160 for occupied-rank.
  Route ties retain identity. The causal Route fixture reports `[0,64,64,0,1]`
  under both policies; sensitivity is observed without an improving update.
- All16 typed admission refusals preserve before/after state and epoch0. Their
  source-bound unsubmitted-audit guards support the exact host API boundary;
  they do not establish CUDA fault-path parity. Prior endpoint evaluations were
  not rerun. The record reports64 attempts,56 successes,eight status8,16 refusals,
  zero endpoint evaluations, zero apply, one invocation.

All18 U/R module admissions passed the unchanged runtime predicate:

| Resource | U | R |
| --- | ---: | ---: |
| Runtime registers | 88 | 115 |
| Runtime local bytes | 21,536 | 21,536 |
| Runtime static shared bytes | 0 | **512** |
| Runtime maximum threads/block | 640 | 512 |
| Own-AOT spill stores / loads | 0 / 0 | 0 / 0 |
| Launched threads/block | 16 | 16 |
| Dynamic shared bytes | 0 | 0 |

Runtime R shared512 is now observed, separate from the retained raw
ELF/cuobjdump total1536 and reserved-cap1024. No subtraction or waiver was used.
Driver580.105.08 was prospectively admitted but not exercised by this invocation;
cross-driver parity remains NOT_RUN.

## Preservation, lifecycle and cost

Remote command [started](started.txt)00:58:01Z and [ended](ended.txt)00:58:04Z,
[exit0](command.exit). Parent and reviewer verified22 remote manifest entries,
20 operator entries and four transferred input identities. The archive
[remote manifest](archive-SHA256SUMS), [operator manifest](archive-OPERATOR-SHA256SUMS)
and [input manifest](archive-INPUT-SHA256SUMS) name paths inside the complete
archive, not this selected evidence directory. [EVIDENCE-SHA256SUMS](EVIDENCE-SHA256SUMS)
covers the copied evidence here. Raw files are copied without normalization.

Complete archive: `icloud:UOR-R4/results/codex/reuse-finite-run2-archive.tar`,
**7,257,088 bytes**, full cloud-readback MD5
**db89eef2659326af31c0e6cad71034e1**. The [final operator receipt](operator-final.log)
shows SHA/readback success before release00:59:08Z and deletion00:59:15Z;
operator exit0 and fleet0. Sessioncodex-cf1-reuse-finite-run2,
podvast-56322885, Vast offer45669036, one RTX5090 at$0.563/h.
Creation00:55:18Z→deletion00:59:15Z is237s, **$0.037064167 lifecycle estimate**.
Provider uptime234s gives$0.036595000 separately. SSH became ready after126s;
no authentication failure was observed.

Cumulative35 closed Codex leases: **$3.396070000**, including unchanged
startup/construction$0.925751944, plus separate PM. Estimates are not invoices;
storage/transfer USD **UNMEASURED**. Local32,185,544KiB and remote157,276,120KiB
passed the30GiB floor; fleet/rate/ownership gates passed. Before admission, only
owned closed copies were consolidated with complete SHA and cloud MD5 readback:
`icloud:UOR-R4/results/codex/codex-sealed-copy-consolidation-20261017.tar`,
640,853,504 bytes, MD5 `02a4a09635241aaf2f86245f5d47f2de`.
The retained restore map preserves original paths; active executable/run packets
and future checkpoint inputs were retained locally. No peer source was removed.

## Decision and next dependency

Accept finite fixture parity for this exact artifact and observed driver.
Deliver this record through independent exact-head review and protected merge,
verify delivered bytes, remove owned branch/worktree and run hygiene. Then
prepare the already-designed32-proposal timing packet (eight excluded warmups,
24 timed proposals/12 pairs across four cells), pin this raw record and acceptance,
and obtain distinct readiness review and execution release before rental.

This delivery grants no timing or production activation. It establishes no
checkpoint0 result, causal speedup, p95, training rate, language capability or
energy saving. Training/M2/Stage B **HELD**; Q4 **NOT_EXECUTABLE**; seed2 excluded.
Line: M1 qualification/cost tooling; count0/3 and suspended headline37/232 unchanged.
