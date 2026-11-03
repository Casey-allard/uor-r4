# REPORT: final disclosure repair, deepseek-85

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `de9cfc84a`.

## What did not work

- The deepseek-82 REPORT claimed that no configuration switch occurred even
  though exact-head review required a seed-2 trainer-seed correction.
- The PR description retained the superseded seed-2 K4 count of 225 instead of
  the bound count of 238.

Both were disclosure defects. They did not change the evidence, exact export
or negative result.

## What changed

- The REPORT now distinguishes the protocol-compliance rerun from
  outcome-driven metric or configuration tuning.
- The PR description now names the bound 437/238 block counts and the expected
  seed-1 K4 exit 2.

## Measurements

No experiment was rerun for this disclosure repair. The bound readings remain:

| Seed | Baseline top-1/top-8/admitted | K4 top-1/top-8/admitted | K4 four-coordinate blocks |
|---:|---|---|---:|
| 1 | 278/384/396 | 278/383/395 | 437 |
| 2 | 224/285/292 | 224/285/292 | 238 |

All six receipts retain exact query, token and kept-cell parity.

## Checks run at the head

- Search confirms no bound report uses the superseded 225-block count as a
  result.
- Evidence JSON and `SHA256SUMS` pass.
- Formatting, `git diff --check`, local PR gate and required GitHub checks are
  recorded at the repaired head.

## Skills used

No available Codex skill applied. The review finding and bound evidence were
compared directly.

## Still open

- No tested CPU configuration has cleared a registered public ranking or
  capability bar.
- Trained float-shadow, complete-reply and fresh-phrasing outcomes remain
  outside this public CPU result.

## Plain summary

The last repair corrects wording and the PR summary; it does not alter data.
The registered K4 pivot remains a bounded negative with exact serving export.
