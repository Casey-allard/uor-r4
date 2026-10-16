# Counterfactual finite-control execution — 16 October 2026

**Later follow-up:** protected image binding #2556 and a distinct reviewed
release produced the [full Stage1 control pass](../counterfactual-controls-pass-2026-10-16/README.md).
This original blocked attempt and its raw bytes remain unchanged. Stage2 still
needs separate pins, independent readiness and release.

**BLOCKED_INSTRUMENT_OR_FIXTURE: runtime/AOT register mismatch.** The single
Stage 1 invocation authorized by [release6224573422](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6224573422)
stopped before the finite-control dose. GPU initialization and snapshot validation
did run; proposal attempts, endpoint evaluations and admission-refusal attempts
are each0. The separate checkpoint decomposition diagnostic remains
NOT_RELEASED. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; seed2 and
apply remain excluded. This is M1 instrument qualification, count0/3 and
suspended headline37/232 unchanged.

## Observed blocker and its boundary

The [unaltered execution record](counterfactual-controls-record.json), SHA256
`67d6052d02b10679ec8c5bdf37e71f06a4424f4f30b9c1153b2b7a1f6e2e56bc`,
contains exactly two records: compiled identity and the first ineligible
fixture's resource attributes. Command exit2, `within_allowance:false`.

| Resource | U runtime / AOT | D runtime / AOT |
| --- | --- | --- |
| Registers | 90 / 88, delta+2 | 122 / 126, delta−4 |
| Local / stack bytes | 21,536 / 21,536 | 22,096 / 22,096 |
| Shared bytes | 0 runtime | 0 runtime |
| AOT spill stores / loads | 0 / 0 | 0 / 0 |
| Maximum launchable threads per block | 640 | 512 |

The required zero register deltas fail even though local bytes agree, the
560-byte D local increase fits the1,024-byte allowance, and both kernels admit
block16. The gate correctly stops before `carry_windows`, all40 proposals,
all128 endpoint evaluations and all16 API-refusal attempts. Those controls are
NOT_RUN; zero apply is retained. `Device::from_snapshot` has already executed
GPU `discrete_init` and `discrete_validate`, so this is not zero model compute.
No loss/status/proposal parity, timing or mechanism conclusion follows.

Source inspection locates the mismatch in artifact provenance: ordinary
`device.rs` and `decomposition_device.rs` load embedded PTX with
`Ptx::from_src`. Their resource reports instead come from the separate
`.cu → cubin` compilation in `build.rs`; those cubins are archived but not
loaded by these paths. Runtime attributes therefore describe driver-generated
machine code while the AOT reports describe a separate compilation. Runtime
driver595.104.02 differs from construction driver615.78.08; that difference
is an observed covariate, not an isolated cause established by this run.

**Next:** align the loaded executable with the resource report through a
separately reviewed source change, preserving the zero-delta gate. Loading
the exact sealed AOT images for both U and D with explicit image hashes is one
bounded candidate. Do not tune registers, weaken the threshold or retry this
binary on another machine to select a passing report. New source/design review,
protected construction-tested delivery, exact pins and a separate execution
release precede a retry. Stage2 remains gated.

## Authority and exact executable

[Source #2554](https://github.com/UOR-Foundation/uor-r4/pull/2554) merged
`7bdfe7aa52cb80ff4d3db1c93e08a3e2e31806a8`. Its complete tree equals reviewed
and construction-tested `70f1d03ec057d0fd595c8721cd8eb610b9b2176d`:
`e0f97b1bfe67ab56abeefb0c4ce3fa7d1f0f3bde`. Independent source/design FIT
[6224276470](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6224276470),
AGY source/packet concurrence
[6224487571](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6224487571),
and exact-head CUDA check/strict-Clippy/identity acceptance
[6224391929](https://github.com/UOR-Foundation/uor-r4/pull/2554#issuecomment-6224391929)
precede the separate bounded execution release. The independent execution
review reverified all30 construction archive manifest entries and the full
pin; parent verified its only subsequent change was the distinct release URL.

| Identity | Value |
| --- | --- |
| Final execution pin SHA256 | `baca4fd56cdbe6e9e3d88e393ff2399c973c25ba2ecf23b0a8cff92c6987f73c` |
| Linux CUDA executable SHA256 | `4b3d8413a7ae19de98397c65b6eb80086ef877c3bc54e8e004ea48ccbb2f4ff7` |
| Standalone source SHA256 | `871639c3c9f38acaf617a84fc218f9bdf263e34f3d1fef58a06c3ffefcc6892b` |
| Composite crate source SHA256 | `a071bc4e21a18c78d1c44c3ca60f88e436ff6b200320f470b4ae9387c880c6b9` |
| Ordinary U PTX SHA256 | `1a21d6c3e2773a427c14ebff9499e03386147ab41b3ce413b7b5c9123925fd28` |
| Diagnostic D PTX SHA256 | `29615646becefc333c697f685157a4c124847b5022e070ab00d7b9c749bc8506` |
| Geometry SHA256 | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| Fixture inventory SHA256 | `53d61d6c6e0942a274baa907df8a4a8e08a207290b6812fce35234807331c465` |

The exact [pin](controls-pin.json) and [observed identity](execution-identity.json)
match; the actual and expected identity files are byte-identical. The exact
executable was reused from the construction archive; no rebuild or
compiler selection follows observation. The [fixed source packet](../counterfactual-controls-2026-10-16/execution-packet.md)
declares40 U/D proposals,128 immutable one-row CUDA endpoints and16 typed API
refusals across the two admitted objective policies. Source-bound unreachable
CUDA2/4/9/12, isolated stale epoch NOT_EXECUTED and apply-only status3 exclusions
remain explicit; they are never claimed as observed CUDA fault coverage.

The fixture predictions were fixed before execution. No host carrier, writer,
objective or selection oracle is introduced. A failed resource, parity,
endpoint, refusal, trace or unchanged-state check stops further dose and keeps
the partial receipt as an instrument/fixture blocker.

## Operations and preservation

One admitted Vast RTX5090 lease, pod `vast-56297621`, session
`codex-cf1-controls-run1`, with local
and remote30GiB disk floors, total fleet≤4/≤$8h, exact binary/pin/identity
checks and runtime resource admission. The planning allowance does not cancel
healthy work. Independent operational review repaired the cleanup path before
rental: unknown or live runner state retains the lease; an atomic terminal
marker is written last before final receipt collection. SHA manifests and
cloud MD5 readback passed before teardown.

The Germany container took229s to expose SSH; the existing project key then
worked. No authentication change was needed. Local free space was31.39GiB at
admission; remote157,260,344KiB. Runtime RTX5090/driver595.104.02; command began
21:04:43Z and ended21:04:48Z. The5s interval includes metadata, CUDA setup and
validation; it is not a kernel measurement. [command transcript](command-transcript.txt) retains
the refusal. The admitted runner, binary identity and transferred pin checks
all passed before it.

The [remote manifest](SHA256SUMS.remote) covers22 files in the complete archive
(`SHA256SUMS.remote` is its retained name here). An independent receipt reviewer
reverified all22 entries,11 operator-manifest entries and2 input-manifest
entries, plus pin/identity/raw-record equality. The full archive is
`icloud:UOR-R4/results/codex/controls-run1-archive.tar`,22,936,576 bytes,
MD5`c41c3538975cea9213ab92f33569b741`. Cloud-store verified the remote bytes by
MD5 readback before lease release21:05:35Z and deletion21:05:40Z. Fleet0 was
verified afterward. The archive includes the binary, scripts, full plans,
driver/library metadata, raw records and separate operator receipts; these
small source records preserve the deciding evidence without duplicating the
executable into Git.

Construction used Rust/Cargo1.97.1 and CUDA12.8.93/compute_120. Static U88
registers/21,536 stack and D126/22,096 have zero spills. Runtime/AOT register
and local/stack deltas must be zero; shared equal, D≤U+1024 local/stack and
block16 launchability remain required. Both uncapped zero-spill D layouts
(126/22,096 and194/21,536) were observed under the superseded register ceiling;
no timing comparison selected between them. The capped56/56-spill build is
superseded. These controls provide no H1/H2/H3 timing attribution.

The provider ledger records307s at the quoted$0.96/h: **$0.081866667 estimate**,
including startup/setup/archive, not an invoice. Cumulative26-lease Codex
estimate **$3.009592222**, plus separately recorded PM cost; construction/startup
subtotal$0.697290277 is unchanged. Storage/transfer USD UNMEASURED. The prior source branch/worktree was
removed after verified preservation; its hygiene pass completed exit0.

The owner's [native-math direction](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6224381783)
continues to govern the next optimization: reuse the same exact geometric
operators and representation in native learning, artifacts, unpacking and
CUDA/serving; remove repeated expensive operations through that structure.
Control qualification and the later diagnostic guide the smallest justified
implementation change. Neither is itself a measured mathematical speedup.
