# REPORT: cpu-training-pilot, deepseek-185

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `1c5209f8e`; run implementation:
`1f02345e6`; final parallel implementation: `40542adf0`.

## Status

Complete instrument on both public artifacts. The table-anchor decoding loss
is attributable to the recurrent input table, not direct writer-row mismatch.

## What did not work

No instrument check failed. The prior eight-step query-port objective remains
a two-seed **REJECT**; this unit did not retrain it.

The table-wide anchor remains unsuitable as an input-table transformation.
On both seeds, the anchored recurrent input reproduced essentially the entire
combined loss. The direct writer-table change did not.

## What changed

- Added an explicit CPU diagnostic forward mode with independent input and
  writer tables while preserving the artifact-backed one-table default.
- Added a four-arm driver for raw/raw, input-only, writer-only and combined
  table substitutions.
- Reconstructed and hash-checked the anchored table, required combined equality
  with the anchored artifact forward, and required raw/raw equality with the
  integer serving session.
- Added fail-closed refusal for Hamming, turn memory and malformed table shapes.
- Added deterministic row signatures for queries, tokens and kept cells.

No serving, CUDA, float-training, exporter or Claude-owned source changed.
The hybrid table pairs are in-memory diagnostics and are not claimed to be
exportable.

## Measurements

- FIT and D67 review head: `a5e472c36`;
- first-run driver SHA-256:
  `ab39c515f15856245d5681dc2230f1fa184a959224739d0933dcf10a7ec58e84`;
- review-repair driver SHA-256:
  `4623d93c0a1e099b5fc942b079f0e0ce2482300749250ff5817bb9bf06815bcc`;
- final parallel driver SHA-256:
  `e195e52ec20cac06287410fbe15bdd2d269b5ca4f37f0a4edb6616229ce850c6`;
- panel power: **58** conversations / **616** response rows per seed;
- first discovery runs (serial despite a 16-thread pool): seed 1
  **35.283 s / 47,980 KiB**, seed 2 **35.008 s / 47,872 KiB**;
- serial review-repair runs: seed 1 **35.449 s / 47,680 KiB**, seed 2
  **35.226 s / 47,912 KiB**;
- final ordered parallel verification at 16 workers: seed 1
  **2.598 s / 115,880 KiB**, seed 2 **2.593 s / 115,980 KiB**;
- both seeds: zero raw CPU/serving query, token and kept-cell mismatches;
- every arm repeated exactly in-process;
- both review-repair runs and the final parallel run reproduced every arm
  field exactly.

Raw, input-only, writer-only and combined readings:

| Seed | Arm | Admitted | Top-1 | Top-8 | Port distance |
|---|---|---:|---:|---:|---:|
| 1 | raw/raw | 245 | 181 | 236 | 29,882,444 |
| 1 | input-only | 120 | 55 | 89 | 25,493,709 |
| 1 | writer-only | 245 | 195 | 233 | 29,882,444 |
| 1 | combined | 120 | 55 | 86 | 25,493,709 |
| 2 | raw/raw | 171 | 136 | 164 | 23,870,298 |
| 2 | input-only | 29 | 9 | 20 | 27,634,674 |
| 2 | writer-only | 171 | 137 | 162 | 23,870,298 |
| 2 | combined | 29 | 10 | 20 | 27,634,674 |

Input-only reproduced combined admission exactly on both seeds and combined
top-1 exactly on seed 1. Writer-only preserved raw admission, kept cells and
port distance exactly. Across the two seeds it changed top-1 by `+14/+1` and
top-8 by `-3/-2`.

The port-distance movement was not consistent evidence of better reads:
input-only reduced the seed-1 sum by 4,388,735 but increased the seed-2 sum by
3,764,376. The root projection does not preserve learned input identity.

Compact evidence:
[table-anchor-summary.json](evidence/table-anchor-summary.json).

## Checks run at the head

- fresh-context FIT: **FIT**;
- fresh-context D67 pre-run review: **GO**;
- exact-head review repairs: the driver now requires the registered 616 rows
  rather than accepting any panel with at least 200, tests the malformed writer
  table, and evaluates rows and serving parity with ordered Rayon workers;
- one-worker and four-worker driver tests produced identical ordered rows and
  parity;
- preserved serial discovery, serial review-repair and final 16-worker
  parallel verification each produced sealed reports;
- raw/raw exact CPU/serving parity on both seeds;
- exact combined-hybrid/artifact-forward equality on both seeds;
- table reconstruction, complete row coverage and deterministic in-process
  repeat on every arm;
- full CPU package: **351 library tests passed**, 4 expected ignores, all
  binary tests passed;
- scoped strict Clippy:
  `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`;
- rustfmt, `git diff --check`, D23 and D32 ratchets;
- `loop pr-check --local`, report/evidence and all **11 / 11** loop gates.

The workspace/dependency Clippy form without `--no-deps` is red on pre-existing
Rust 1.97 lints in `uor-r4-integer` and `uor-r4-discrete`; those files were not
changed.

## Delivery

PR #3739 was reviewed at exact head
`c5e26e8286701d9a48d9b2a6f3764d2239f57ba0` after merging current
`origin/main`. All required checks passed and the protected queue merged it as
`5e7d5c681534be2ba41a050aadf642488d8f1543`.

Fresh detached `origin/main` passed the full CPU package (**351 library tests
passed**, 4 expected ignores, all binary tests), scoped strict Clippy,
formatting, diff, report, FIT/prep evidence, D23/D32 ratchets and claim-wording
checks at the merge commit.

## Size

The implementation adds one diagnostic driver, one optional CPU forward table
role, focused tests, the pre-registration, compact evidence and records. No
model artifact or corpus is committed.

## Skills used

No system-provided Codex skill applied. Repository pre-registration, D67,
evidence and delivery procedures were used.

## Still open

The current one-table artifact cannot carry the successful diagnostic split.
The next representation must either store separate recurrent-input and
writer-address rows, or train root-addressed writer rows with an explicit
preservation objective for the learned input code. A later trained artifact
must still pass exact export, CPU/serving and language-panel gates.

## Plain summary

Changing only the table rows used by the recurrent input caused nearly all of
the decoding collapse. Changing only the rows used to score writer candidates
preserved admission and moved relatively few decisions. Root-addressed writer
scoring remains viable, but it cannot be obtained by overwriting the input
code in the shared table.
