# Baseline-reuse timing UUID blocker — 17 October 2026

**BLOCKED_SETUP_GPU_IDENTITY before any instrument binary invocation.** The
allocated RTX 5090 reported the admitted driver **595.71.05**, but its actual
UUID differed from the specifically finite-qualified GPU. The unchanged wrapper
refused at **02:33:21Z**, exit1. Instrument invocations, proposals, warmups, timed
pairs, apply and training were all **zero**. Original U/R timing remains
**unmeasured**. This is an operational admission blocker, not a mechanism negative.
The [independent outcome review](setup-outcome-review.md) and
[PM acceptance6228597454](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6228597454)
accept the preserved operational-blocker receipt and completed preservation-only
recovery at this exact scope.

The same attempt also exposed a preservation-copy defect: the author's frozen
operator omitted `*.csv` from its local-operator archive copy. Its manifest check
failed and correctly retained and renewed the pod. A preservation-only recovery
copied the exact missing qualification CSV, retained the original failure and
manifest, verified every manifest and full cloud MD5 readback, then released and
deleted the pod. No gate, executable, model input or dose changed during recovery.

## Prospective authority and stopping boundary

The [baseline-reuse pre-registration](../../harness/prep/m1-counterfactual-baseline-reuse-2026-10-16.md)
retains design FIT6226136376. [Finite3 acceptance6228377410](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6228377410)
was protected through #2568 at
`66154156dad423f5bf4c822236509f6ba659ab8f`, approved tree
`d9e141fef471ec49dfdbe062993133921ea90e60`. Its exact finite result remains accepted
in the [recorded finite3 environment](../counterfactual-baseline-reuse-driver-595-2026-10-17/README.md).
Required hygiene41126 [completed before this lease](hygiene-complete.json).

The [independent timing readiness and narrow environment FIT6228520779](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6228520779)
and distinct [execution release6228540541](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6228540541)
preceded rental. That release is now **consumed**. The frozen 21-entry review
manifest has SHA256
`5ef4910b47033381c7b32a67cf80d4054b630e08aebd99a910c65581307f7d06`.
The [final timing pin](timing-pin.json), SHA256
`b506fc3b59ab747dc0aae0f4b171c13df8f34a87d814311f859972d5c986012e`,
differs from the [review pin](timing-pin.review.json), SHA256
`f8846c4c1fccc49260447cee98b13ab3031c67194c4439dc1f4b0d2f3e184d95`,
only by the distinct release URL. [Environment disposition](environment-disposition.json)
requires actual GPU equality; advertised machine/host provenance is not a substitute.

| Admission identity | Required / finite-qualified | Actually allocated |
| --- | --- | --- |
| GPU | NVIDIA GeForce RTX 5090 | NVIDIA GeForce RTX 5090 |
| Driver | 595.71.05 | 595.71.05 |
| Memory | 32,607 MiB | 32,607 MiB |
| UUID | `GPU-1daf8397-88e7-c3e5-e50c-0dbc1459969c` | `GPU-35bf2e2c-cbce-cd85-2bf6-8a0e045557b6` |
| Advertised machine | 8914, supporting provenance | 35115 |
| Advertised host | 18, supporting provenance | 18 |

The [actual GPU CSV](gpu.csv), [qualified CSV](qualified-gpu.csv),
[command log](command.log), [terminal exit1](command.exit) and frozen
[remote operator](run-remote.sh) show this exact boundary. The driver and GPU
name checks passed; full qualified receipt equality then failed. It stopped
before `ldd`, actual library hashing, transferred-input checking, `reuse-identity`,
`reuse-check`, `reuse-plan`, invocation-marker creation or `reuse-timing`.
Their output files and timing-output directory are absent from the complete
archive. No instrument-generated timing record, actual compiled-identity output,
checkpoint bridge or CUDA/model initialization was reached.

The prospective library condition remains a disclosed limitation: finite3
retained loader SONAME/path mappings, not complete shared-library bytes,
OS/package identity or a container digest. The reviewer accepted that limit
only for the original paired scope. This attempt did not reach the actual loader
mapping check. [Qualified loader receipts](qualified-ldd.txt) are prior expectations,
not new observations. No new platform qualification follows.

## Unchanged artifact and unexecuted dose

| Retained identity | SHA256 / commit |
| --- | --- |
| Construction source | `5f533cbfd49ecef9a4ea2b217bb31a93002b3dc2` |
| Protected source delivery | `95ee863c34e6120689e01774f59ced6097736d9c` |
| Executable | `671917acb13b541c148b6b8122b92ae613ac0cdff8577a3f72fb6c05faa06209` |
| Compiled composite | `aca299c484d5434291f7f65cc9315d6b52b976952bcd68f39a45cd3341bf2825` |
| Original checkpoint | `9a76743e401759b7d3aabc708ab0a26a9470ea474dc140f328c7b1aab59e49e4` |
| Configuration | `eff7a77e1c3c8454324c9ddf31a6c08bdbe1b213d3324fe7b7889c0fe7665ae4` |
| Accepted finite3 raw record | `7f36d6f805bcb7082467373e3a9f17c23070feb96d9269c6f7e64ce0405567a5` |

The remaining U/D/R source/PTX/cubin/report/geometry/source-pair pins remain
unchanged and retained. These identify transferred artifacts; they do not imply
that those artifacts ran on the newly allocated GPU. The checkpoint0 bridge,
U/R resources, state/parity and all timing outcomes remain NOT_RUN in this attempt.
The released dose was exactly 32 proposals: eight excluded warmups, 24 timed
proposals / 12 U/R pairs across four cells, seed1/epoch0/rows64 and the original
checkpoint/configuration/generator. **None of that dose was reached.** There was
no finite or endpoint rerun, source/CUDA build, on-lease tuning, fallback, retry,
post-allocation UUID relaxation, apply or training.

## Preservation failure and recovery

The original [lease operator](lease.sh), SHA256
`6a0ca14a59d92c78f861e741ddf4948b919648aadc16b993dcccb8d04c9884ec`,
and [remote operator](run-remote.sh), SHA256
`5b967fedf8b106e1459123faa52e6f101498304d58ca1329ea0aa6871bc67ec0`,
remain frozen. The [initial operator log](operator-initial-failed.log) ends in
`RETAIN ... complete SHA256 verification failed`. The omitted CSV was present
and correct remotely, and the remote/input checks passed. The failure was the
nested reviewed-packet verification inside `local-operator`: its copy glob
included JSON/Markdown/shell/log/text files but omitted CSV, although the frozen
review manifest required `qualified-gpu.csv`. The operator did not upload,
release or delete the pod on this preservation failure; it retained and renewed.

The preservation-only [recovery script](preservation-recovery.sh) and
[recovery note](preservation-recovery-note.md) copy the exact frozen CSV after
comparing it against the complete downloaded remote copy. The original
[50-entry operator manifest](archive-OPERATOR-SHA256SUMS.initial), original
failure log and operator bytes remain retained. The
[updated 51-entry operator manifest](archive-OPERATOR-SHA256SUMS) adds the missing
CSV without rewriting old entries. The [four-entry recovery manifest](archive-RECOVERY-SHA256SUMS)
retains the failure/recovery explanation, script and session receipt. This is a
preservation repair, not a source/kernel fix or a renewed execution permission.

The [recovery transcript](preservation-recovery.log) verifies all **24 remote,
14 input, 50 original operator, 51 complete operator, four recovery and 21 frozen
review entries** before cloud storage. The independent review's retained
[remote](SHA256SUMS.stdout.txt), [input](INPUT-SHA256SUMS.stdout.txt),
[initial operator](OPERATOR-SHA256SUMS.initial.stdout.txt),
[complete operator](OPERATOR-SHA256SUMS.stdout.txt),
[recovery](RECOVERY-SHA256SUMS.stdout.txt),
[review-packet](recovered-review-sha.stdout.txt) and
[final-authorization](final-pin-sha.stdout.txt) verification transcripts agree. The [cloud receipt](cloud-readback.log)
records `icloud:UOR-R4/results/codex/reuse-timing-run2-archive.tar`,
**9,870,336 bytes**, full remote-readback MD5
**2e882d487871ff5edc249c15f37c14cf**, before lease release **02:35:01Z** and deletion
**02:35:07Z**. Recovery completed without any instrument retry.

The original [remote](archive-SHA256SUMS), [input](archive-INPUT-SHA256SUMS),
operator and recovery manifests name complete-archive paths, not this selected
evidence directory. [EVIDENCE-SHA256SUMS](EVIDENCE-SHA256SUMS) covers copied
selected evidence here. Raw bytes, including frozen packet wording and failure
transcripts, are preserved without normalization. The binary, checkpoint and
full archive are retained in the cloud archive rather than duplicated here.

## Placement, cost and disk incident

The [final session ledger](session-ledger-final.jsonl) records exact-driver
preselection **02:32:10Z**: offer45508898, machine35115, host18, Alberta CA,
raw quote **$0.9083333333333332/h**, admission-rounded **$0.908/h**. The two
advertised 595.71.05 candidates were not reservations; the actual selected GPU
was different from the qualified UUID. Same host/driver did not pass the actual
identity gate. Creation02:32:11Z, SSH ready after44s, podvast-56332437,
sessioncodex-cf1-reuse-timing-run2. No SSH authentication failure was observed.
The [final fleet](fleet-after.log) shows no owned Codex pod; one separately owned
peer pod remained at the displayed $1.39/h and was left untouched.

[Cost metadata](cost.json): creation **02:32:11Z** to deletion **02:35:07Z** is
**176s**, **$0.044407407 lifecycle estimate** at the raw quote. Provider uptime174s
gives **$0.043902778** at the same raw quote; its legacy rounded-rate estimate is
**$0.043886667**, separately retained. Cumulative **38 closed Codex leases:
$3.490932222**, with unchanged startup/construction carry **$0.947947500**,
plus separate PM. Prior cumulative estimates remain unchanged. Estimates are
not provider invoices; storage/transfer USD remain UNMEASURED.

A separate [post-teardown disk incident](workspace-disk-incident.md) records the
new docs worktree briefly materializing redundant full tracked content. Free
space fell to **31,315,980 KiB**, below the **31,457,280 KiB** floor, after this
pod had already been deleted. No new lease or model work began. Parent refreshed
the index and reapplied the declared sparse checkout, preserving unique/peer
material and restoring **31,808,948 KiB**. Future worktree creation must avoid
full materialization; fresh disk admission remains mandatory before any lease.
This operational deviation changes no model or timing observation.

## Decision and next dependency

The independent review and PM accept the operational blocker and preservation
recovery only. Deliver this consumed release, zero-dose UUID refusal and honest
failure/recovery evidence through exact-head review and protected merge, then
verify bytes, clean owned transport and run hygiene. The old UUID rule was
enforced correctly; it is not relaxed retrospectively.

A separate [prospective physical-device assessment](prospective-device-disposition.md)
explains why a future explicitly scoped same-RTX5090/sm120/exact595.71.05 packet
may compare U and R on the same newly recorded device without an extra finite
programme solely for a UUID change. This is **future design assessment only**,
not approval of a current packet, a claim that the new GPU is finite-qualified,
or a rental/release. After protected delivery/cleanup, a successor must repair
the archive-copy inventory including CSV, preserve all strict resource/parity/
state/accounting gates and original32 dose/order, and obtain fresh admissions,
independent sealed readiness and a distinct execution release. No automatic
retry, device search or opportunistic extra dose is authorized here.

Timing remains unmeasured; the accepted finite records remain unchanged. No
speedup, p95, training-rate, checkpoint0 parity, language, energy, production
activation or mechanism-negative conclusion follows. Training/M2/Stage B **HELD**;
Q4 **NOT_EXECUTABLE**; seed2 excluded. M1 qualification/cost tooling count0/3
and suspended headline37/232 unchanged.
