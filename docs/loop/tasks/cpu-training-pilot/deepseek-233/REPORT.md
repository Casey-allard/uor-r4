# REPORT: cpu-training-pilot, deepseek-233

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[calibrated-branch sheet](../../../../harness/prep/cpu-training-pilot-calibrated-branch-2026-11-06.md).
Base: `ddc66e665`.

## Status

Complete as **INSUFFICIENT_POWER** before fresh-panel generation.

Seed 2788 was not generated. No fresh endpoint-selection result is claimed.

## What did not work

- Four consumed dev panels, seeds 2783-2786, supplied **1,832** calibration
  rows but only **58** groups with both multiple boundary branches and an exact
  endpoint.
- The same panels supplied **0** groups where an exact endpoint followed a
  wrong first boundary. The selector again had no example of the decision it
  was meant to learn.
- Preflight candidate rows were **233** and positive rows were **58**. The
  matched label shuffle changed **84** labels, but the power gate stopped
  before using the weights.

The distribution shift was present in the panels, but exact complete endpoints
were too sparse and were already selected by the first boundary whenever they
appeared. Previously consumed fresh panels therefore do not supply the needed
endpoint preference under this candidate construction.

## What changed

Extended the deepseek-231 branch driver with repeatable pinned calibration
panels:

- each calibration directory is bound by its manifest SHA-256;
- the manifest binds its token and mask bytes;
- calibration panels are explicitly listed in preflight and result receipts;
- an evaluation-panel overlap fails closed;
- the fresh evaluation panel is not opened until the power gate passes;
- the calib-backed seat applies but does not replace the TRAIN-only fold path.

The code change is small; the failed calibration configuration is retained as
the result. No serving default, artifact format, `UORLUT01`, CUDA, float path
or Claude-owned source changed.

## Measurements

| Input | Rows | Groups | Baseline wrong | Candidates | Positives |
|---|---:|---:|---:|---:|---:|
| dev seeds 2783-2786 | 1,832 | 58 | 0 | 233 | 58 |

Power floors were 96 groups and one baseline-wrong group. Neither passed.

Resources for the seed-1 artifact preflight:

- wall: **28.98 s**;
- user/system: **456.84 / 0.04 s**;
- average CPU: **1,576%**;
- peak RSS: **136,532 KiB**;
- retained evidence: **11,836 bytes**.

Calibration manifests:

- 2783:
  `90e8f9e9a1dabc393b124a66e2c1fed0c9d3f670bf0128b60b00674032d87b78`;
- 2784:
  `03a01091a2907ec7dd3252964894c6ebae83797fa691d1f9ece3f9027fa0a68c`;
- 2785:
  `a5214c2f8430a9a3c22e4b468e40d6594247dbe5ac1e9fc30025e31cba9dcbf1`;
- 2786:
  `19d9629273ea423f5d39bef81740ef34c018a440ff4206359575290e1e8b773b`.

Compact result: [result.json](evidence/result.json). Raw preflight:
[preflight.json](evidence/preflight.json).

## Interpretation

The prior recommendation split into two paths: distribution-matched supervision
or a reference-free objective. This unit closes the first path for exact
complete endpoint labels in the tested branch construction. More consumed
panels are unlikely to help because exact endpoint candidates are the sparse
condition, not the panel count.

The next unit must score completion properties without requiring an exact
reference endpoint. Candidate features already include self-contained evidence
that can be used for a fixed reference-free objective, including boundary
specificity, continuation support, tail stability and repetition.

## Checks run at the head

- focused joint-branch tests: **7 passed**;
- driver test: **1 passed**;
- full CPU library: **415 passed, 4 expected ignores**;
- scoped strict Clippy passed with the same pre-existing `type_complexity` and
  `needless_borrow` exemptions;
- Rustfmt check on the changed Rust files and `git diff --check` passed;
- all three evidence checksums passed;
- seed-2788 panel and result paths remained absent.

## Skills used

No system-provided Codex skill applied.

## Still open

- a reference-free complete-branch objective;
- complete-reply and fresh-phrasing capability;
- broader full-model CPU training;
- Claude's independent end-to-end track.

## Plain summary

Old fresh panels looked like the right calibration data, but their exact
endpoint candidates were too rare and never followed a wrong first stop. The
registered power gate stopped the run before seed 2788. The next useful work is
reference-free scoring of complete branches, not more exact-label calibration.
