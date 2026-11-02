# REPORT: reconcile final CPU training item states, deepseek-6

Lab: deepseek. Item: [cpu-training-program](../../../../roadmap/items/cpu-training-program.md) (#3507). Base: `20422589a`.

## What did not work, first

- **The program page still said the shared-vector bounded pilot had not run.** PR #3550 executed the merged FIT sheet on both public artifact seeds and retained a negative result.
- **The child-item table still carried #3503 and #3505 as open after their stated done-when contracts were met.** The remaining complete-reply, fresh-phrasing and public-run restart work belongs to #3506, not to those closed component items.
- **The matrix called the bounded pilot `NOT_RUN`.** The ranking bar failed, but the exact-export and admission evidence did run and must remain visible as a configuration negative.

## What changed

- Marked `cpu-quantization-exact-memory` (#3503) and `cpu-shared-vector-train` (#3505) done.
- Recorded the public two-seed pilot result in the durable program matrix and ordered-work section.
- Updated #3506 readiness to `public pilot negative; full item open` rather than `NOT_ADMITTED`.
- Kept the complete-reply, genuinely fresh-phrasing, trained-float-shadow and public-run interval-restart gaps explicit.
- Added the measured update-only throughput, wall time and peak RSS to the performance section.

## Measurements

- No compute ran in this unit.
- The cited pilot is the merged pre-registered run in PR #3550 / `12f2c7ef7`: both seeds kept admission and exact export, but top-1/top-8 gains were zero against bars of +43/+47 and +40/+44.

## Checks run at the head

- `cargo run -q --locked -p xtask -- roadmap check`.
- `cargo run -q --locked -p xtask -- roadmap render --check`.
- `git diff --check`.

## Skills used

No repository skill was invoked. This is documentation-only status reconciliation based on the merged CPU REPORTS.

## Still open

- #3506: a configuration clearing a registered bar, public-corpus interval restart, trained float shadow, and credentialed complete-reply/fresh-phrasing grading.

## Plain summary

The CPU component items now report the truth: their machinery is complete, the first bounded public training configuration is a clean negative, and the full pilot remains open only for evidence that could not be obtained from the public inputs.
