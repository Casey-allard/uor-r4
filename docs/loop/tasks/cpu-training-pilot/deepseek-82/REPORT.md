# REPORT: same-amplitude table BVG-K pivot, deepseek-82

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `de9cfc84a`. Source commit:
`8a0defae5`.

## What did not work

- **Four coordinates did not improve ranking at one Q12 per coordinate.**
  Against seed-1 baseline top-1/top-8/admitted 278/384/396, K4 produced
  278/383/395. It lost one top-8 row and one admitted target, so the K4
  admission gate rejected it.
- **The seed-2 replication was rank-neutral.** Against baseline 224/285/292,
  both K1 and K4 remained 224/285/292. K4 accepted 238 four-coordinate blocks
  but did not exceed K1 or clear the pre-registered 40-row top-1 threshold.
- **The larger margin improvement did not transfer to target ranking.** On
  seed 1, K4 reduced the held-out margin-loss sum from `11,739,473` to
  `11,731,518` while leaving top-1 unchanged. On seed 2 it reduced the sum
  from `29,717,075` to `29,471,323`, again without a top-1 gain.
- **The K1 control was not a positive route on seed 1.** At the same one-Q12
  amplitude, K1 changed 278/384/396 to 278/383/395 and increased the
  margin-loss sum to `11,747,984`.

This is a negative for the tested one-versus-four coordinate breadth
configuration. It is not a rejection of rank-margin learning, BVG-K or CPU
training generally.

## What changed

The implementation at `8a0defae5` adds opt-in K-coordinate BVG-K blocks to the
exact tied-token trainer. K1 preserves the historical selection and artifact
path; K4 patches up to four distinct interior coordinates, reparses the full
artifact and requires an exact joint-margin improvement with target admission
preserved. Checkpoints bind the coordinate count, and the corpus driver records
accepted-coordinate counts and a 1-64 histogram.

The FIT-reviewed design fixed baseline, K1-step-1 and K4-step-1 on both public
artifact seeds before any result in this unit was observed. No metric or
outcome-driven configuration search followed the result. The later seed-2
rerun corrected a preregistration deviation: the first seed-2 batch used
trainer seed 2 instead of the registered trainer seed 1.

## Measurements

All arms used trainer seed 1, margin Q12 1, step Q12 1, parameter limit Q12
8,192, 4,096 attempts for trained arms, checkpoints every 1,024, evaluation
stride 2 and the fixed 1,024-row public internal-development panel. An initial
seed-2 run used trainer seed 2, contrary to the FIT sheet. Exact-head review
caught that deviation; fresh baseline/K1/K4 runs with trainer seed 1 superseded
it, and only the registered receipts are bound below.

| Seed | Arm | Accepted | Four-coordinate blocks | Top-1 | Top-8 | Admitted | Artifact |
|---:|---|---:|---:|---:|---:|---:|---|
| 1 | baseline | 0 | 0 | 278 | 384 | 396 | `a9df5263...` |
| 1 | control-k1 | 431 | 0 | 278 | 383 | 395 | `9d59da8d...` |
| 1 | pivot-k4 | 437 | 437 | 278 | 383 | 395 | `349e0f15...` |
| 2 | baseline | 0 | 0 | 224 | 285 | 292 | `da56fe96...` |
| 2 | control-k1 | 228 | 0 | 224 | 285 | 292 | `44817e41...` |
| 2 | pivot-k4 | 238 | 238 | 224 | 285 | 292 | `04a9887c...` |

K4 moved 1,748 coordinates on seed 1 and 952 on seed 2; every accepted K4 block
contained four distinct coordinates. Five runs exited zero; seed-1 K4 wrote its
complete receipt and then exited 2 at the required admission gate, as
pre-registered. Every arm evaluated 1,024 rows and checked 262,144 Q12 query
coordinates and 8,192 kept cells with zero query, token or kept-cell
mismatches. Final artifacts reloaded and checkpoint/export controls passed.

Total wall time was 621.118 seconds on the Ryzen 9 9950X. Peak RSS was
53,992 KiB. The release driver SHA-256 was
`cc73bc0dcdcec1440c27bc95811d1ae4f72aa9bd8d9a18196cd9cd1e0e4096d7`.

## Checks run at the head

- Full CPU all-target suite at `8a0defae5`: 128 library tests passed, 5 driver
  tests passed, all auxiliary binaries passed, 4 release benchmarks ignored.
- Strict Clippy with `--all-targets --no-deps -- -D warnings`: passed.
- Formatting and `git diff --check`: passed.
- FIT and pre-run review: FIT, complete, `may_compute=true`.
- Preflight hashes: both artifacts and all three data inputs matched the
  registry; the release driver hash was recorded.
- Evidence `SHA256SUMS`: verified after collection.

## Skills used

No available Codex skill applied. The project rules, FIT sheet, CPU trainer,
driver, public artifacts and evidence receipts were inspected directly.

## Still open

- No tested CPU configuration has cleared a registered public ranking or
  capability bar.
- A genuinely trained float shadow remains unavailable.
- Credentialed sealed complete-reply and fresh-phrasing grading remain
  `NOT_RUN`.
- CPU support for the 19-table M1 trunk remains open.

## Plain summary

Four simultaneous one-unit coordinate moves reached the same small or negative
ranking reading as one move. The trainer did exactly what it claimed: it made
multi-coordinate edits, improved the internal margin objective, preserved
deterministic integer serving on both seeds, and reported where admission or
ranking was lost. Those results close this same-amplitude breadth configuration
without supporting it as the route to useful CPU learning.
