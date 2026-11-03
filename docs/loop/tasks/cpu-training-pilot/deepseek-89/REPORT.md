# REPORT: discrete joint-loss eligibility repair, deepseek-89

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `498e36574`.

## What did not work

- Exact-head review found that the first joint-loss implementation skipped a
  row if the proposed block removed that row's target eligibility. A joint
  candidate must be invalid in that case, matching single-candidate scoring.
- The first six-run evidence therefore came from a superseded binary and could
  not be the bound K1/K4 result.
- The experiment remains negative: the corrected rerun reproduces the same
  admission and ranking failures.

## What changed

- A shared `accumulate_joint_loss` rule rejects any moved row with no target
  rival or sentinel loss.
- A focused regression pins valid accumulation and both invalidation cases.
- All six arms were rerun from exact head `5b9e304d1` with release driver
  SHA-256 `27fc5389…7810d`.
- Comparison, REPORT and notebook totals were regenerated from the fresh
  receipts.

## Measurements

| Seed | Arm | Top-1 | Top-8 | Admitted | Gate |
|---:|---|---:|---:|---:|---|
| 1 | baseline | 38 | 108 | 361 | accept |
| 1 | K1 | 19 | 82 | 248 | reject admission |
| 1 | K4 | 13 | 63 | 256 | reject admission |
| 2 | baseline | 38 | 108 | 361 | accept |
| 2 | K1 | 19 | 82 | 248 | reject admission |
| 2 | K4 | 13 | 63 | 256 | reject admission |

K4 committed 71 moves across 46 accepted blocks, with 14 multi-coordinate
blocks. All six CPU/serving exports were exact. Total wall time was 460.986
seconds and peak RSS was 20,312 KiB.

## Checks run at the head

- CPU all-target suite: 133 library tests passed, 4 ignored; all driver and
  auxiliary binaries passed.
- Strict package Clippy, package formatting and `git diff --check`: passed.
- FIT design and exact-head pre-run: PASS.
- Six corrected receipts and `SHA256SUMS`: passed.

## Skills used

No available Codex skill applied. The joint scorer, BVG-K decision path,
driver, receipts and review finding were inspected directly.

## Still open

- The corrected K1/K4 configurations remain a bounded negative.
- Full-path trained Hamming/memory/point-key coverage and complete replies
  remain open.

## Plain summary

The first joint-block run skipped a row that should have invalidated the block.
The rule is now fixed and tested, then all six runs were repeated from the
correct binary. The same negative result remained, but now it is valid evidence
for the registered K1/K4 comparison.
