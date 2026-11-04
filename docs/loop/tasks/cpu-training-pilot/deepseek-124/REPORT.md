# REPORT: public 19-table CPU count training, deepseek-124

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[trunk count training](../../../harness/prep/cpu-training-pilot-trunk-count-2026-11-04.md).
Base: `2326bece3`.

## What did not work, first

- **The accepted 128-row update changed no held-out served decision.** The
  source, exact-update and shuffled-target artifacts all retained admitted /
  top-1 / top-8 = **332 / 246 / 270** and the same decision checksum across 512
  document-disjoint rows. This is not surprising: 128 additions are tiny
  against the source's 1,049,856 rows, and the result is intentionally not
  treated as a capability test.
- **Exact and shuffled artifacts differed even though their decisions did not.**
  The update targets changed counts and serialized bytes, but did not move the
  held-out argmaxes. The writeback mechanism is proven; behavioural influence
  at this dose is not.
- **Attempts 1 and 2 hit the resource stop.** The current read-only CPU trunk
  path could not initially write back under the registered 2 GiB limit:
  attempt 1 reached 3,063,024 KiB and attempt 2 reached 2,297,824 KiB. Both are
  preserved and excluded as evidence. Streaming the reader and artifact phases
  reduced the accepted attempts to 1,537,652/1,538,376 KiB.
- **This is not GPU qualification or language capability.** The public DEV
  panel was used as a bounded internal train/evaluation split, and no complete
  reply or fresh-phrasing claim follows.

The first FIT review returned **NOT_FIT**. Aggregate table totals did not prove
which `(key, token, count)` entries changed, and the unigram best-token tie had
no negative test. The driver now independently rebuilds every expected table
from the training rows and refuses any decoded difference; a finite tie test
requires the smaller token id, the exact and shuffled artifacts must differ,
and changed served-decision rows are counted rather than represented only by a
boolean checksum difference.

The second review returned FIT at docs head
`3ff9ca748ad76cd22f5fa2ae8c27e35a1ac4c249`, bound to source
`b70ae7de6c2a6fe0946ca14fffe0a63bb62525d2`.

That FIT run was executed inadvertently. It completed the mechanism checks but
reached **3,063,024 KiB** peak RSS, above the registered 2 GiB cap. The run is
preserved as a resource-stop attempt at
`local/deepseek/trunk-count/2026-11-04-attempt-1/`; its outputs are not accepted
as this unit's evidence. Source `d5108222a` drops the source serving model and
the training reader before later verification and evaluation. Attempt 2 uses a
fresh build and output root.

The third review returned FIT at docs head
`e94f21783e10e1a1d570d34d634698ea870657a0`, bound to source
`d5108222aef6882007aba31eeadc95796b9c8aca`.

Attempt 2 also hit the registered stop at **2,297,824 KiB**, despite dropping
the model and reader. It is preserved at
`local/deepseek/trunk-count/2026-11-04-attempt-2/` and is not accepted.
Source `1ca202ef2` now consumes each training reader, writes and releases one
artifact before constructing the other, and reloads each artifact only for its
own verification/evaluation phase. Attempt 3 uses new roots and requires a new
FIT.

The fourth review returned FIT at docs head
`3baa15318af8b9e9fab855ab1eea5ff0fc54b02c`, bound to source
`1ca202ef217918d4c8444499ebaabc3a91e1878a`.

## What changed

`CpuDiscreteTrunkReader` now:

- derives exact teacher-forced `TrunkTrainRow` values from its current CPU
  read, recurrence and query path;
- exposes an additive count-training export over the unigram and all 12 sparse
  tables;
- validates every unigram and sparse increment, preserves the smaller-token
  global tie, and round-trips all 19 tables.

`cpu-trunk-count-driver` adds deterministic document-disjoint train/evaluation
selection, exact and shuffled-target artifacts, independent full
`(key, token, count)` reconstruction, semantic-section comparison, per-row
decision deltas, and CPU/integer serving parity. It writes and releases one
86 MiB artifact before constructing the other, keeping measured RSS below the
registered cap.

## Measurements

The fixed split used 200 training documents / 7,080 eligible targets and 193
evaluation documents / 512 held-out targets. The first 128 training rows were
added. The source artifact and both updated artifacts retained all 19 tables.

| Table family | Source total | Exact total | Shuffled total |
|---|---:|---:|---:|
| Unigram | 1,049,856 | 1,049,984 | 1,049,984 |
| Each of 12 sparse tables | 1,049,856 | 1,049,984 | 1,049,984 |

The driver independently rebuilt every decoded count-table entry from the
training rows and required exact equality. Codebook, routes, gates, recurrence
steps and normalization frames were unchanged.

| Arm | Artifact SHA-256 | Admitted | Top-1 | Top-8 | Decisions changed vs no update |
|---|---|---:|---:|---:|---:|
| no update | `04652a3d…147d` | 332 | 246 | 270 | 0 |
| exact | `015a621a…35d9` | 332 | 246 | 270 | 0 |
| shuffled | `ad0bf763…872a` | 332 | 246 | 270 | 0 |

Every arm had zero mismatches over **512 rows and 16,384 Q12 query coordinates**
for query coordinates, admitted cells, top-8 and token. The exact artifact is
86,545,604 bytes and the shuffled artifact 86,552,056 bytes. Accepted walls
were **9.873/10.079 s**, peak RSS **1,537,652/1,538,376 KiB**, and the seed-1
repeat was byte-identical for both artifacts with normalized report SHA-256
`6dbaac2b9c7a6ac4f122dc05e271ef290399cec9c8a31c0ba66787f2ac56eee4`.
All roots used binary SHA-256
`eee04141652007864d58e870104ee0923a0a51cd6146f5d27b9989698c30e521`.
The accepted roots occupy 346,207,857 bytes.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the accepted binary, reports and both
19-table artifacts. Raw accepted artifacts remain under
`local/deepseek/trunk-count/2026-11-04-attempt-3/`; stopped attempts remain in
attempt-1 and attempt-2.

## Checks run at the head

- Focused CPU tests: four `cpu_trunk_*` tests and the count-driver test passed,
  covering exact row derivation, all-table round-trip, count ties, shuffled
  targets and artifact export.
- Full `uor-r4-cpu-train` release suite: 197 library tests passed, 4 release
  benchmarks ignored, plus all binary suites (226 tests passed in total).
- Strict no-deps all-target CPU Clippy with `-D warnings`: passed.
- Formatting, `git diff --check`, report check, roadmap check (139 items),
  documentation catalog (1,175 records, zero missing paths), prep check, D23,
  D25 and D32 ratchets passed. The first full loop-gate attempt hit the known
  fake-`gh` `ETXTBSY` race in `loop-drive` after 27/28 tests; the retained
  immediate rerun passed all 11 gates. Both attempts are recorded in
  [loop-gates.txt](evidence/loop-gates.txt).

## Skills used

No system-provided skill applied.

## Still open

- Determine a larger but still bounded count-training dose that changes served
  decisions without the discarded 128-row result being retuned.
- Complete replies, fresh phrasing, useful local chat and GPU qualification.

## Plain summary

This unit makes the project's public 19-table model writable and additively
trainable on CPU rather than read-only. The 128 added rows changed every count
table by exactly 128, exported cleanly and preserved all 19 tables, but they
did not move any of the 512 held-out decisions. That is useful infrastructure
with a neutral capability reading, not evidence of learning.
