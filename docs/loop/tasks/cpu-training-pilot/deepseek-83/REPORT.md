# REPORT: registered-trainer-seed repair, deepseek-83

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `de9cfc84a`. Source:
`8a0defae5`.

## What did not work

- **The first seed-2 batch violated the registered trainer seed.** The FIT
  sheet fixed trainer seed 1, but the initial seed-2 receipts used trainer seed
  2. Exact-head review caught the deviation before merge.
- **The corrected seed-2 batch remained rank-neutral.** With trainer seed 1,
  baseline, K1 and K4 all produced 224/285/292
  top-1/top-8/admitted.
- **K4 still failed the registered ranking comparison.** It accepted 238
  four-coordinate blocks and reduced the margin-loss sum from `29,717,075` to
  `29,471,323`, but top-1 and top-8 did not move. The combined result remains a
  configuration negative.

The superseded trainer-seed-2 run is preserved in Git history and is not used
as registered evidence.

## What changed

- Reran seed-2 baseline, K1-step-1 and K4-step-1 with trainer seed 1 and fresh
  output directories.
- Replaced only the three bound seed-2 receipts.
- Updated the comparison, REPORT, notebook, program, roadmap and evidence
  checksums to the repaired readings.
- Added this repair ORDER and REPORT so the protocol deviation and correction
  remain explicit.

## Measurements

| Seed | Arm | Trainer seed | Accepted | Four-coordinate blocks | Top-1 | Top-8 | Admitted |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | baseline | 1 | 0 | 0 | 278 | 384 | 396 |
| 1 | control-k1 | 1 | 431 | 0 | 278 | 383 | 395 |
| 1 | pivot-k4 | 1 | 437 | 437 | 278 | 383 | 395 |
| 2 | baseline | 1 | 0 | 0 | 224 | 285 | 292 |
| 2 | control-k1 | 1 | 228 | 0 | 224 | 285 | 292 |
| 2 | pivot-k4 | 1 | 238 | 238 | 224 | 285 | 292 |

K4 moved 1,748 coordinates on seed 1 and 952 on seed 2. Every accepted K4 block
contained four distinct coordinates. All six receipts evaluated 1,024 rows and
checked 262,144 Q12 query coordinates and 8,192 kept cells with zero query,
token or kept-cell mismatches. Five processes exited zero; seed-1 K4 wrote its
complete receipt and exited 2 at the required admission gate.

The six runs took 621.118 seconds total. Peak RSS was 53,992 KiB. The release
driver SHA-256 remained
`cc73bc0dcdcec1440c27bc95811d1ae4f72aa9bd8d9a18196cd9cd1e0e4096d7`.

## Checks run at the head

- CPU all-target suite: 128 library tests passed, 5 driver tests passed, all
  auxiliary binaries passed and 4 release benchmarks were ignored.
- Strict package Clippy: passed.
- Targeted formatting, JSON parsing, evidence checksums and `git diff --check`:
  passed.
- `uor-harness prep-check`: the FIT sheet is complete and cleared for compute.
- Local PR gate and GitHub required checks: recorded at the repaired head.

## Skills used

No available Codex skill applied. The project rules, FIT sheet, trainer,
driver, receipts and review finding were inspected directly.

## Still open

- No tested CPU configuration has cleared a registered public ranking or
  capability bar.
- Trained float-shadow, complete-reply and fresh-phrasing outcomes remain
  outside this public CPU result.

## Plain summary

The review caught that seed 2 had used the wrong trainer seed. The corrected
run used the registered seed and still produced no ranking gain: K4 made the
right four-coordinate moves, preserved exact serving, and lowered the margin
objective, but held-out predictions stayed flat. The bounded negative now
matches the preregistered protocol.
