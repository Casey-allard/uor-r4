# Baseline-reuse timing startup — 17 October 2026

Follow-up: the [opt-in exact-driver selector](../driver-aware-pod-selection-2026-10-17/README.md) addresses advertised mismatches before allocation. It changes no evidence or admission in this stopped attempt and grants no replacement execution release.

**BLOCKED_SETUP_DRIVER, before any instrument binary invocation.** The allocated
RTX 5090 reported **595.71.05**, outside the prospectively admitted **580.95.05**.
The unchanged wrapper refused it before CUDA/model initialization. Instrument
invocations, proposals, warmups, timed pairs, apply and training are all **zero**.
This is an operational admission blocker; it provides no timing, parity or
mechanism-negative result. The accepted [finite controls](../counterfactual-baseline-reuse-finite-pass-2026-10-17/README.md)
on580.95.05 remain unchanged evidence.

## Prospective authority and stopping boundary

The [baseline-reuse sheet](../../harness/prep/m1-counterfactual-baseline-reuse-2026-10-16.md)
has design FIT6226136376. Finite acceptance6227712402 was delivered in protected
#2565 at `c754f160cf140e87fef675698e3939d2244f6c9c`, full approved tree
`c914520c36c33a276c94a0590aa7906da15456c1`. Its own transport cleanup and required
hygiene completed before timing release. Independent timing readiness
[6227870279](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6227870279)
and separate execution release
[6227893756](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6227893756)
preceded rental. That release is now **consumed**.

The [wrapper](run-remote.sh), [command log](command.log), [exit1](command.exit)
and [GPU receipt](gpu.csv) show the refusal at01:22:42Z, the same second as
startup. Driver admission precedes `ldd`, transferred-input checking,
`reuse-identity`, `reuse-check`, `reuse-plan`, invocation-marker creation and
`reuse-timing`. Those output files, the marker and the timing-output directory
are absent from the complete archive. No instrument-generated timing record
exists. The [independent boundary review](outcome-review.md) confirms this scope.

The [final pin](timing-pin.json), SHA256
`dea48273933cbacb869f41fe5cbba63e0dcfc3cd625cdbc3570b4a1958514f70`,
differs from the [review pin](timing-pin.review.json), SHA256
`46d59b5385a4a6ab4244894e64150e82ae35512b1ef6e0199582f3af84f7c8fd`,
only by the distinct release URL. The reviewed wrapper SHA256 remains
`678f0e61979ba900ccf43b5dfebe55f30a3469abbe884697cd4ffde158dece02`;
no on-lease edit, allowlist broadening or retry occurred.

| Retained identity | SHA256 / commit |
| --- | --- |
| Construction source | `5f533cbfd49ecef9a4ea2b217bb31a93002b3dc2` |
| Protected source delivery | `95ee863c34e6120689e01774f59ced6097736d9c` |
| Executable | `671917acb13b541c148b6b8122b92ae613ac0cdff8577a3f72fb6c05faa06209` |
| Compiled composite | `aca299c484d5434291f7f65cc9315d6b52b976952bcd68f39a45cd3341bf2825` |
| Original checkpoint | `9a76743e401759b7d3aabc708ab0a26a9470ea474dc140f328c7b1aab59e49e4` |
| Configuration | `eff7a77e1c3c8454324c9ddf31a6c08bdbe1b213d3324fe7b7889c0fe7665ae4` |
| Accepted finite raw record | `c6a5b3f857d77bf979ed439e61575f04fce2f398b9854286681eac53203cd4f4` |

The remaining image/report/geometry/source-pair pins are retained without
normalization. These are transferred artifact identities, not observations
from executing them on595.71.05. Runtime resources and checkpoint0 U/R parity
were not queried or measured on this attempt. The fixed32-proposal timing dose
(eight excluded warmups,24 timed/12 pairs/four cells) remains **NOT_RUN**.

## Preservation and cost

Parent and independent reviewer verified **20 remote,24 operator and10 input**
SHA entries from the retained archive. These are post-run hash checks; the
stopped wrapper did not reach its own input-check command. The
[remote](archive-SHA256SUMS), [operator](archive-OPERATOR-SHA256SUMS) and
[input](archive-INPUT-SHA256SUMS) manifests name full-archive paths.
[EVIDENCE-SHA256SUMS](EVIDENCE-SHA256SUMS) covers selected copied evidence here.

Full archive: `icloud:UOR-R4/results/codex/reuse-timing-run1-archive.tar`,
**9,547,264 bytes**, full-readback MD5 **ef4197f63542ecd58a366b80316b308d**.
The [final operator receipt](operator-final.log) shows complete SHA/cloud-MD5
verification before release01:23:20Z and deletion01:23:25Z; final fleet0.
Operator exit1 reflects the preserved admission failure. Session
codex-cf1-reuse-timing-run1, podvast-56325466, Vast offer45028230, one RTX5090
in Alberta at$0.908/h. GPU UUID
`GPU-1daf8397-88e7-c3e5-e50c-0dbc1459969c`.

Creation01:21:57Z→deletion01:23:25Z is88s,
**$0.022195556 lifecycle estimate**; provider uptime86s gives$0.021691111
separately. SSH succeeded after23s; no authentication failure was observed.
Cumulative36 closed Codex leases: **$3.418265556**, plus separate PM;
storage/transfer USD **UNMEASURED**. The prior startup/construction carry
$0.925751944 plus this setup-only lease is$0.947947500. Estimates are not invoices.
Local31,959,828KiB admission (31,962,556KiB at operator start) and remote157,268,752KiB
passed the30GiB floor; fleet, rate and ownership gates passed.

## Decision and next dependency

The retained [read-only provider investigation](provider-selection/findings.md)
shows that Vast exposes `driver_version` before allocation, while the current
provider rejects that query field and drops it from safe offer output. The
captured search at 01:25:10Z found zero exact 580.95.05 offers under the existing
placement constraints. This is point-in-time availability, not a lasting claim;
the selected-field captures disclose their transcription and timestamp limits.
Their seven-file [manifest](provider-selection/SHA256SUMS) has SHA256
`b7d8345cb4f8e65bf20c6da7bd9b330a32209fe7e25bd964c6ef0dbe0fae7f15`.

Seal this stopped attempt through independent exact-head review and protected
delivery, then verify bytes, clean owned transport and run hygiene. Commission
the smallest independently reviewed exact-driver selector: retain and filter the
advertised string before allocation, refuse missing/mismatched metadata, and
return unavailable with zero allocation when no offer qualifies. Preserve the
remote driver check for metadata drift. No driver predicate,
source, helper, instrument or kernel is changed in this delivery; the read-only
investigation is not an implementation or new execution release.

A replacement packet needs its own review, fresh disk/fleet/price/ownership
admission and a distinct release. Do not reuse this consumed release, retry
automatically or treat a different driver as finite-qualified by assumption.
Training/M2/Stage B **HELD**; Q4 **NOT_EXECUTABLE**; seed2 excluded. M1
qualification/cost tooling count0/3 and suspended headline37/232 unchanged.
No speedup, p95, training-rate, language or energy result follows.
