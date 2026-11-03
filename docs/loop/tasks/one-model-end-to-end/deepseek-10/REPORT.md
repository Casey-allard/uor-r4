# REPORT: served-comparison result, deepseek-10

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base: `75086c1d3`.

## What did not work

- **The operational door failed.** Served training took 6.96× plain at seed 1 and 6.83× at seed 2, against the registered 2× ceiling. The wiring is not ready for a longer language run.
- **The quality smoke is inconclusive.** Served minus plain top-1 was −25 and −55. The spread is 30, so the required band is 30; seed 1 is inside it.
- **The local review could not open the private store object.** No Hugging Face credential is configured on this workstation. The decision uses Claude's sealed handoff summary rather than claiming an independent download.
- **The slowdown is not attributed.** The 49-pass activation-scale loop is a plausible cause, not a measured one.

## What changed

The existing FIT sheet and deepseek-5 REPORT now record the completed run and apply their registered decision table. The sheet status is `EXECUTED`; the result is instrument PASS, operational FAIL and quality INCONCLUSIVE.

The sealed handoff is
`results/claude/one-model-end-to-end-copy-key-2026-11-02/s1/s1.tar`,
SHA-256 `738457b26af18e9176bd0e3ece76f0502d4505cdf0a3f249381fc0ccf574ad1d`.

## Measurements

- Plain/served training: 51/355 s at seed 1 and 52/355 s at seed 2.
- Served-evaluator top-1: 36/11 at seed 1 and 61/6 at seed 2.
- Nearest hits: 225/23 and 229/20.
- Openings: 22/1 and 15/1.
- Served decode: 49.2–49.4, 40.8–41.1 and 28.6–28.8 tok/s at contexts 16, 64 and 192.
- Both served checkpoints were unchanged by export rounding; four of five files
  changed for each plain checkpoint.

## Checks run at the head

- Arithmetic checked: 355/51 = 6.9608×; 355/52 = 6.8269×; both exceed 2×.
- Quality band checked: `max(24, |−55 − (−25)|) = 30`; seed 1's 25-row gap is inside.
- `git diff --check`: passed.
- `cargo run -q --locked -p uor-r4-claim-wording -- .`: passed.
- `uor-harness prep-check`: the executed sheet remains complete and FIT.
- Fresh exact-head review: APPROVE.

## Skills used

No available Codex skill applied. The registered sheet, deepseek-5 records and Claude's sealed run summary were read directly.

## Still open

- Profile and cost-reduce `activation_quantize16` before another matched run.
- Rerun this exact four-job comparison after the repair; do not lengthen training or change the quality bar first.

## Plain summary

The served-surrogate path is finite and export-consistent, but it is nearly seven times slower than plain training, so it fails the readiness gate. The short quality smoke is inconclusive because the seeds disagree with the required band. The next step is to repair quantizer cost and repeat the same bounded comparison.
