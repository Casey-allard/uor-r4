# CPU table BVG-K same-amplitude pivot

**Status:** READY FOR FIT REVIEW

TEST FITNESS: FIT - M2 #2030

## Question

Does assigning one exact Q12 unit across four tied-table coordinates change more
held-out predictions than assigning one Q12 unit to one coordinate, with both
arms trained on the same samples?

deepseek-81's four-Q12 K1 control lost one admitted target and stopped before
K4. This second bounded comparison fixes per-coordinate amplitude at one,
making spatial breadth the named variable. It does not retest step-size effects.

The two public artifact seeds each run baseline, K1-step-1 and K4-step-1.

## Base artifacts

The artifact paths and hashes are exactly those in
`cpu-training-pilot-table-bvgk-pivot-2026-11-03.md`.

## Data split

The token, mask and manifest paths and hashes are exactly those in
`cpu-training-pilot-table-bvgk-pivot-2026-11-03.md`. The split remains 200
training documents / 7,080 targets and 198 evaluation documents capped to
1,024 rows. The source manifest says `split=train`; model exposure is
unverified.

## Arms

Every arm uses trainer seed 1, 4,096 update attempts, checkpoints every 1,024,
evaluation stride 2 and the 1,024-row cap. Each arm starts from a fresh load of
its pinned artifact.

| Arm | Step Q12 | Coordinates per step | Admission enforcement |
|---|---:|---:|---|
| baseline | 1 | 1 | required |
| control-k1 | 1 | 1 | reported, not required |
| pivot-k4 | 1 | 4 | required |

The command template is the same as deepseek-81 with `--step-q12 1`, the arm's
coordinate count, and `--require-admission` on baseline and K4. K1 is run
without that switch so an expected/non-expected admission delta remains an
outcome rather than halting before the active arm.

## Bar

- **Instrument:** all six processes exit 0 except a K4 admission rejection,
  which exits 2 after writing its receipt. Every run reports finite readings,
  1,024 evaluated rows, zero query/token/kept-cell export mismatches, a
  reloadable final artifact and exact checkpoint resume.
- **Control:** K1's admission, top-1 and top-8 readings are reported and may
  differ from baseline.
- **Candidate admission:** K4's final target-admitted count is at least
  baseline; otherwise K4 is rejected and the comparison is a pivot negative.
- **Update activity:** both trained arms accept at least one step. K4 must
  accept at least one block containing more than one distinct coordinate; its
  full 1–64 histogram is reported.
- **Ranking:** top-1 is the primary metric. For each seed
  `p = baseline_top1 / 1024` and
  `F = ceil(3 * sqrt(1024 * p * (1-p)))`. K4 must gain at least `F` top-1
  rows over baseline, not fall in top-8, and strictly exceed K1 top-1.
- **Positive:** K4 satisfies every row on both seeds.
- **Negative:** anything else closes these K1/K4 same-amplitude configurations
  and triggers the required pivot note. No post-result metric switch or
  additional table-only tuning follows without new causal evidence.

## Measurements

For every run: accepted/refused updates, coordinate histogram and moves, epoch,
top-1, top-8, admission, margin/rank sums, timing, artifacts/checkpoints,
CPU/serving parity and peak RSS. Record source, binary, machine and every input
hash.

## Steps

1. Verify the release binary and all five artifact/data hashes.
2. Run baseline, K1 and K4 on seed 1 in that order.
3. Repeat in the same order on seed 2.
4. Compare the six receipts under the bar and write one result receipt plus
   `SHA256SUMS`.

## Minimum power, cost and stop

Two artifact seeds, 1,024 document-disjoint rows per seed and 4,096 attempts
per trained arm. This is a bounded development pivot, not fresh-phrasing or
language evidence.

Only the named Ryzen 9 9950X is used: 16 physical / 32 logical CPUs, no GPU,
pod, upload or paid API. Projected retained output is below 100 MiB and peak
RSS below 256 MiB. Stop before 30 minutes. Stop immediately on a non-K4
instrument failure, hash mismatch, non-finite reading, export mismatch or
failed checkpoint/artifact reload. Do not shorten a completed arm or tune
after observing results.

## Claim boundary

A gain is two-seed internal-development evidence for coordinate breadth only.
It is not fresh-transfer, useful-chat, GPU-equivalence or GPU-replacement
evidence.

## Verdict

TEST FITNESS: FIT - fresh non-author review of design SHA-256
`860f5260be8ed1bcc63721f47bd59ee2805a750bc095ffb3b15de630c10de678`;
PRE-RUN: PASS.

The reviewer confirmed the fresh baseline/K1/K4 schedule, K4 admission and
multi-coordinate telemetry, exact top-1 threshold, checkpoint/export controls,
bounded two-seed power, and public internal-development claim boundary. The K1
admission result is diagnostic; only K4 admission can reject the candidate.
