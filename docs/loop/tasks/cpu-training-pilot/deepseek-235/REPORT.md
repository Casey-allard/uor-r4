# REPORT: cpu-training-pilot, deepseek-235

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[reference-free closure sheet](../../../../harness/prep/cpu-training-pilot-reference-free-closure-2026-11-06.md).
Base: `91be6c9ac`.

## Status

Complete as **INSUFFICIENT_POWER** after the first fresh artifact.

Seed 2788 was opened once under recorded FIT/GO. The fixed power gate stopped
the run before artifact seed 2 and before repeats. The seed-1 quality readings
are retained as measurements, but they are not promoted to a mechanism
negative.

## What did not work

- The power floor required **100** selected branches different from
  first-boundary and **20** rows with an exact candidate. Seed 1 produced
  **21** changed selections and **18** exact-candidate rows.
- Only **195/428** reachable rows had any boundary candidate, so the candidate
  spine itself could not supply a complete reply for the remaining 233 rows.
- Closure made **16** exact replies and **876** body positions, versus
  first-boundary **18/876** and shuffled closure **18/876**. Premature BOS was
  **67** versus **70** for both controls.
- Full retrieval remained stronger at **44** exact and **803** body positions
  with zero premature BOS.

These readings fail the registered quality bars, but the power stop takes
precedence. Seed 2 and the repeat were not run.

## What changed

Added two reference-free arms to `joint_branch`:

- **closure** chooses the earliest candidate with no deeper global
  continuation advantage and at least four local-or-global decisions in its
  eight-step tail;
- **shuffled closure features** cyclically permutes the
  endpoint-advantage/tail-support tuples within each candidate list while
  preserving the tuple multiset and candidate lengths.

The driver exposes `--primary reference-free-closure`; the primary-specific
claim, control, comparisons and fail-closed power gate are bound into output.
No reference token enters candidate generation or closure selection.

## Measurements

Fresh panel:

- tokens:
  `1d049e5e3c0a95cceb5a45e5acd569a64bc17c3087d59f1c1aca66880c2bde3d`;
- mask:
  `8b79a29c5bf27616e2337bdf05b444364f94e7d07acd3e144dc7714e96c9fe68`;
- manifest:
  `997f1edc6a493283e212506e8758feb08aae7ec40318e2c1d8ac5e0a8af0d146`;
- seed 2788, `dev,dev`, protocol 2, 128 conversations, 428 reachable rows.

| Arm | Exact | Body | Premature BOS | Changed from first |
|---|---:|---:|---:|---:|
| closure | 16 | 876 | 67 | 21 |
| shuffled closure | 18 | 876 | 70 | 0 |
| first boundary | 18 | 876 | 70 | 0 |
| full retrieval | 44 | 803 | 0 | — |

Selection and evidence:

- rows with candidate branches: **195**;
- closure exact-candidate rows: **18**;
- closure selections: **195**;
- complete outcome changed from shuffled closure: **yes**;
- panel generator: **0.60 s**, 7,560 KiB peak RSS;
- selector: **8.25 s**, **931%** average CPU, 135,584 KiB peak RSS;
- retained evidence: **67,998 bytes**.

Raw report: [seed1.json](evidence/seed1.json). Compact result:
[result.json](evidence/result.json).

## Interpretation

The closure rule is executable and the matched control is active, but the
registered candidate-spine power was not present on seed 2788. The useful
diagnostic is that usable boundary candidates cover under half of reachable
rows, so endpoint selection alone cannot complete the majority of replies.

This does not close reference-free scoring generally. It closes the fixed
four-support, eight-step closure rule and this candidate-spine power floor as
an underpowered fresh run.

## Checks run at the head

- fresh-context FIT: **FIT** at implementation `4166a6f1b`;
- focused joint-branch tests: **9 passed**;
- driver test: **1 passed**;
- full CPU library: **417 passed, 4 expected ignores**;
- scoped strict Clippy passed with the existing `type_complexity` and
  `needless_borrow` exemptions;
- Rustfmt, `git diff --check`, prep-check and evidence checksums passed;
- no float, dense, artifact-format or serving-default change;
- Claude's files and PR were not touched.

## Skills used

No system-provided Codex skill applied.

## Still open

- candidate coverage for rows where the current content spine never reaches a
  boundary;
- a reference-free objective over a denser candidate representation;
- complete-reply and fresh-phrasing capability;
- broader full-model CPU training;
- Claude's independent end-to-end track.

## Plain summary

The label-free closure rule ran once on seed 2788. It changed too few endpoint
choices and had too few exact candidates to meet its own power floor, so the
run stopped before seed 2. The result is insufficient power, not a claim that
reference-free closure can or cannot work. The next attempt needs denser
candidate coverage before another selector is tested.
