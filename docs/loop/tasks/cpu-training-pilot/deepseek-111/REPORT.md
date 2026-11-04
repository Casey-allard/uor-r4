# REPORT: cpu-training-pilot, deepseek-111 (reply termination diagnostic)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Prep:
[FIT termination diagnostic](../../../harness/prep/cpu-training-pilot-reply-termination-2026-11-04.md).
Base: `3f7f8605b`.

## What did not work, first

- **Termination is mixed, dominated by missing admission.** Across 1,824
  generated decisions per arm, 1,710–1,772 decisions never admitted BOS.
  Training reduced that count only from 1,772 to 1,717 on seed 1 and did not
  change it on seed 2.
- **Admission alone would not solve termination.** When BOS was admitted, it
  never reached top-8 or top-1. Its best rank was 47 on seed 1's trained arm and
  87–89 on the other arm/seed combinations.
- **No decoder suppression was observed.** `top1_suppressed` and `chosen_bos`
  were zero in every arm on both seeds, so the `reply_stop` path was not active.
- **This is not a new training configuration.** The exact 32-update replay
  reconstructs the pinned deepseek-109 artifacts.

## What changed

The CPU FBS forward now diagnoses a target token's writer decision without
mutating state:

- target cell and admission;
- one-based rank among admitted writer candidates;
- top-8/top-1 status;
- target/chosen L1 distances and their signed difference;
- exact mutually exclusive classes for never admitted, admitted-not-top-1,
  top-1-suppressed, chosen BOS, no decisions and mixed.

The bounded reply evaluator records these fields at every generated step while
preserving the generated sequence and digest.

## Measurements

The 32-update replay reproduced trained artifacts
`c856ce21…b88de4` and `10f91f73…40e6`; all updates were accepted. Every arm's
reply digest matched the deepseek-110 pin exactly, with 114 rows and 1,824
decisions per arm.

| Seed | Arm | Never admitted | Admitted, not top-1 | Top-8 | Chosen BOS | Best rank | Class |
|---:|---|---:|---:|---:|---:|---:|---|
| 1 | frozen | 1,772 | 52 | 0 | 0 | 89 | mixed |
| 1 | trained | 1,717 | 107 | 0 | 0 | 47 | mixed |
| 1 | memory-off | 1,772 | 52 | 0 | 0 | 89 | mixed |
| 1 | trained-memory-off | 1,772 | 52 | 0 | 0 | 89 | mixed |
| 2 | frozen | 1,710 | 114 | 0 | 0 | 87 | mixed |
| 2 | trained | 1,710 | 114 | 0 | 0 | 87 | mixed |
| 2 | memory-off | 1,710 | 114 | 0 | 0 | 87 | mixed |
| 2 | trained-memory-off | 1,710 | 114 | 0 | 0 | 87 | mixed |

`top1_suppressed` was zero everywhere. Never-admitted steps were 97.15% /
94.13% of seed-1 frozen/trained decisions and 93.75% of all seed-2 arms.

Walls were 40.862 s and 40.039 s; peak RSS was 390,468 and 394,128 KiB. The
fresh seed-1 repeat matched every non-timing field and ran in 39.794 s at
388,692 KiB. Five served teacher-forced arms retained zero mismatches over
4,333 decisions, 1,109,248 Q12 coordinates and 34,664 kept cells per seed.

Source was `555a20b9ee5f4e2ad813ad3ec7367ee4f30428a5`; release binary
`9f7b061565e9568325a4cc5a0ee12596feb98680357737a28dd394505a7a9c45`;
host was the AMD Ryzen 9 9950X on Linux 7.0.0-42-generic with Rust 1.97.1.

The detailed record is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the binary and raw reports.

## Checks run at the head

- Eleven focused reply-evaluator tests passed, including every classification
  branch and mixed partitioning.
- The writer diagnostic test passed.
- Full CPU release suite: 176 library tests passed, 4 release benchmarks
  ignored; every binary suite passed.
- Strict all-target CPU Clippy and formatting passed.
- Both public seeds and the seed-1 repeat completed under FIT.
- All four diagnostic reply digests matched their deepseek-110 pins per seed.

## Skills used

The repository's `experiment-prereg` instructions governed the FIT sheet.
No system-provided skill applied.

## Reading

Termination is not a single decoder bug. The dominant failure is geometric
admission: 93.75–97.15% of generated steps exclude the BOS cell entirely.
Among the minority of admitted steps, BOS remains far outside top-8, with a
best rank of 47. The decoder correctly emits the winning candidate; there is no
stop-contract suppression.

The next mechanism must address both factors, beginning with a geometric
termination-cell admission rule because it accounts for most failures. Even a
perfect admission fix would leave the admitted BOS distance outside top-8, so a
served BOS movement objective must follow unless the admission rule also
changes the target-cell evidence.

## Still open

- Geometric BOS admission.
- A served BOS movement objective for admitted-but-distant steps.
- Complete-reply/fresh-phrasing capability and useful chat.
- Independent integer-serving parity before any target override.
- GPU qualification.

## Plain summary

The decoder now records whether BOS is admitted and how it ranks at every
generated step. On both seeds, 93.75–97.15% of decisions never admit BOS. When
admitted, BOS still ranks no better than 47th. The next work must fix admission
first and then the BOS query distance; decoder stopping is not the defect.
