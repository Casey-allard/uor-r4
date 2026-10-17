# Baseline-reuse finite admission — 17 October 2026

Follow-up: a separately reviewed and released replacement invocation passed the
[finite controls](../counterfactual-baseline-reuse-finite-pass-2026-10-17/README.md)
on driver580.95.05. This original stopped attempt remains unchanged evidence;
its release is consumed and its dose was zero.

**BLOCKED_SETUP_DRIVER, before instrument initialization.** The single wrapper
invocation under [finite release6227215454](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6227215454)
rejected driver **580.105.08** because the independently reviewed packet admitted
**580.95.05** only. The RTX5090 and source images were not changed to pass it.
Instrument invocations, finite proposals, typed admission-refusal dose attempts,
apply and training are all **zero**. This is an operational admission blocker,
not a CUDA resource observation, parity result or mechanism negative.

## Exact boundary and identities

The unmodified [wrapper](run-remote.sh), [command log](command.log),
[exit](command.exit) and [GPU receipt](gpu.csv) show exit1 at00:19:55Z, the same
second as startup. The driver check precedes `ldd`, transferred-input checking,
`reuse-identity`, `reuse-check`, `reuse-plan`, the invocation marker and
`reuse-controls`. None of those output files, the marker or the finite-output
directory exists in the complete archive. No instrument-generated control
record exists; this document does not substitute one.

The [final pin](finite-pin.json), SHA256
`eca01552f7a7a9faf1eaec69ce902953bec91ffe3a9a3aadc9d0cb4b19ea3004`,
differs from the [review pin](review-pin.json), SHA256
`01627dc92aed5800e074f0ecf14fcfe5f1dbe3906b56b53adeb199a9e1f8b6c7`,
only by the release URL. It binds executable
`671917acb13b541c148b6b8122b92ae613ac0cdff8577a3f72fb6c05faa06209`,
compiled composite
`aca299c484d5434291f7f65cc9315d6b52b976952bcd68f39a45cd3341bf2825`,
and source5f533cbfd49ecef9a4ea2b217bb31a93002b3dc2. Protected #2562 merge
95ee863c34e6120689e01774f59ced6097736d9c equals that complete reviewed tree
3e3a86954d0e42564b14f5711eaf85807ed101d5. Source cleanup and hygiene completed
before this release. AGY-owned cache cleanup6227206557 restored the disk floor;
fresh parent checks passed before rental. Source/kernel/dose were unchanged.

Independent reviewer `review_2551` and parent verified16 remote manifest entries,
21 operator entries and four transferred input identities from the retained
archive. These post-run hash checks do not imply the wrapper reached its own
input-check command. The [raw manifests](archive-SHA256SUMS) and
[operator manifest](archive-OPERATOR-SHA256SUMS) refer to the full archive paths.
All runtime resource equalities and finite parity remain NOT_RUN.

## Preservation and cost

Archive: `icloud:UOR-R4/results/codex/reuse-finite-run1-archive.tar`,
**6,783,488 bytes**, full-readback MD5 **7d22b276f4054fd5eadcc6478baf2fa8**.
SHA verification and full cloud MD5 readback preceded release00:20:45Z and
pod deletion00:20:53Z; fleet0. Podvast-56319182, sessioncodex-cf1-reuse-finite-run1,
Vast offer43995413, $0.563/h. Creation00:17:01Z to deletion00:20:53Z is232s,
**$0.036282222 lifecycle estimate**; provider uptime228s gives$0.035656667.
SSH became ready after137s. No authentication failure was observed.
Cumulative34-lease Codex estimate **$3.359005833**, including
startup/construction$0.925751944, plus separate PM; storage/transfer USD
**UNMEASURED**. Local/pod disk and fleet/rate gates passed.

## Disposition and next dependency

Preserve this consumed release and stopped attempt without retrying it.
The [prospective driver disposition](next-driver-policy.md) permits preparing a
replacement finite-only packet for the two exact observed driver versions.
It is not a new execution release. Protected receipt delivery/cleanup/hygiene,
independent replacement-packet readiness and a new distinct release precede
another rental. The unchanged64 proposals/16 refusals remain unexecuted.
Timing needs finite acceptance and its protected record first.

Training/M2/Stage B **HELD**; Q4 **NOT_EXECUTABLE**; seed2 excluded. M1
qualification/cost tooling, count0/3 and suspended headline37/232 unchanged.
No speedup, cross-driver parity, language, energy or training-rate claim.
