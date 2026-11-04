# REPORT: 4,096-row 19-table CPU count training, deepseek-125

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[trunk count dose](../../../harness/prep/cpu-training-pilot-trunk-count-dose-2026-11-04.md).
Base: `09c4f4abb`.

## What did not work, first

- **The 4,096-row exact update regressed every held-out decision surface.**
  No-update and shuffled-target arms retained admitted / top-1 / top-8 =
  332 / 246 / 270, while exact targets fell to **331 / 245 / 269**.
- **The exact update was not inert.** It changed four held-out decision rows
  relative to both controls; the shuffled arm changed none. Thus the larger
  dose is behaviorally active, but its direction is negative on this panel.
- **The registered bar failed before either paired margin mattered.** Exact
  top-1 did not exceed no update or shuffled targets and admitted/top-8 both
  lost one row.
- **This is bounded development evidence, not fresh transfer.** The 512 rows
  are document-disjoint from this training slice but belong to the public DEV
  panel and are not fresh with respect to the source model.
- The first FIT review returned **NOT_FIT** because the dose rationale and
  panel freshness boundary were stated imprecisely. The repaired sheet records
  the 128-row null as motivation, fixes 4,096 independently as the existing
  driver ceiling, and labels the panel as bounded development. The second
  review returned FIT at docs head
  `0c108f6c8c78c6d9e8ca596680a7741754fc2072`.

## What changed

No model source changed. The merged CPU count driver ran at its existing
4,096-row ceiling on the fixed public artifact, exact-target and
shuffled-target controls, independent full-table reconstruction and exact
CPU/integer parity.

## Measurements

The fixed split held 200 training documents / 7,080 targets and 193 evaluation
documents / 512 targets. The first 4,096 training rows were added.

| Reading | No update | Exact | Shuffled |
|---|---:|---:|---:|
| Admitted | 332 | 331 | 332 |
| Top-1 | 246 | 245 | 246 |
| Top-8 | 270 | 269 | 270 |
| Changed rows vs no update | 0 | 4 | 0 |

Each of the unigram and 12 sparse tables increased by exactly 4,096 entries:
**1,049,856 → 1,053,952**. Independent full `(key, token, count)`
reconstruction passed for both updated artifacts, and semantic sections stayed
unchanged.

All arms had zero query-coordinate, admitted-cell, top-8 and token mismatches
over **512 rows / 16,384 Q12 coordinates**. Exact and shuffled artifacts were
86,815,864 and 87,042,864 bytes. Walls were **9.778/9.711 s**, peak RSS
**1,543,944/1,541,688 KiB**, and both artifact pairs repeated byte-identically.
Normalized report SHA-256 is
`64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f`;
the binary SHA-256 is
`eee04141652007864d58e870104ee0923a0a51cd6146f5d27b9989698c30e521`.
The two roots occupy 347,729,988 bytes.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the binary, reports and two updated
19-table artifacts.

## Checks run at the head

- FIT passed at docs head
  `0c108f6c8c78c6d9e8ca596680a7741754fc2072`, bound to source
  `09c4f4abb5c864ec7ac11b98cbfb4e1059394081`.
- Both runs and the byte-identical repeat completed under the registered
  600-second and 2-GiB stops; all per-root checksum inventories verify.
- Full `uor-r4-cpu-train` release suite: 197 library tests passed, 4 release
  benchmarks ignored, plus all binary suites (226 tests passed in total).
- Strict no-deps all-target CPU Clippy with `-D warnings`: passed.
- Formatting, `git diff --check`, report check, roadmap check (139 items),
  documentation catalog (1,175 records, zero missing paths), prep check, D23,
  D25 and D32 ratchets, and all 11 full loop gates passed.

## Skills used

No system-provided skill applied.

## Still open

- A different count schedule or statistic that changes held-out decisions in a
  positive direction without losing admission.
- Fresh phrasing, complete replies, useful local chat and GPU qualification.

## Plain summary

This unit asks whether a 32-times larger additive count dose can change and
improve public held-out decisions where 128 rows did not. It does change four
decisions, but loses one admitted, top-1 and top-8 row. The fixed 4,096-row
additive configuration is therefore negative; it is not a rejection of other
count schedules or geometric training.
