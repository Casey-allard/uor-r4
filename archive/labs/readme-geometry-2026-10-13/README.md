# The README, the geometry page and the figures, brought to the current path (2026-10-13)

**Question.** The owner asked for the README and the geometry page to be redone so that they reflect the
current path, and for the README to read as a presentable research project with good bookkeeping. The
pictures were to be updated along with the mechanisms. The old README had grown to 518 lines of
accumulated history. The geometry page described the pre-D23 float form (softmax reads and head, sigmoid
copy gate, SwiGLU over dense projections) as its main path. The status figures still showed that form.

**State: delivered as documentation and a figure generator. No model compute and no model numbers.**
This record moves no headline (reply 37/232, v5 memory 20/40).

## What changed

- **[README](../../../README.md)** was rewritten. The visible sections are: the question, the architecture,
  where we are (M1–M8), the current path, the rules (D11, D22–D27), the mechanisms, the key results with
  their scope, bookkeeping, how the project is run and the quick start. All results with negatives, models
  and data, the repository map and the statistics are in `<details>` sections. Every result row is
  labelled with its scope, and pre-D23 float-form results are labelled *historical*. The roster matches
  the PM's [R9](../../harness/pm-rules.md).
- **[Geometry page](../../geometry.md)** now leads with the compliant next-token path and the operators of
  the compliant trunk: the G2 Hamilton gate, the SpiralCore/FBS projection, the geometric recurrence gates
  and the rank-margin objective. It states what is missing: the head, device qualification and the read
  layers. The mathematical sections are kept, with their status lines corrected:
  - §2 describes the geometric gates;
  - §7 marks the softmax route mix as historical;
  - §8 reports the finished VSA test;
  - §9 and §10 record that SpiralCore's signed action is now the trunk's projection (#2279).
- **The complete previous page** is preserved unchanged at
  [history/geometry-float-form-2026-10.md](../../history/geometry-float-form-2026-10.md), with its links
  re-pointed. This includes the float path, "life of one token", the pre-registered designs, the M2 input
  access and the retained protected learning.
- **[Mechanism index](../../mechanisms/README.md)** has a Status column and is split into compliant-trunk
  and other mechanisms. Each status was checked against its brief. E8 orbit memory and spin-native weights
  have KEEP rungs; they were not "design" as a first draft had it.
- **[Bookkeeping](../../bookkeeping.md)** (new, proposed for the PM's plan) maps each README surface to its
  source of truth and to what changes it, sets five rules, and splits the work between labs, the PM and a
  README keeper.

## The figure generator (D27)

`xtask/src/figures.rs` generates the seven status figures in Rust (`cargo run -p xtask -- figures --write`).
Check mode (`cargo run -p xtask -- figures`) fails when a committed SVG differs from what the code
produces. The generator also refuses any label wider than its box, using an estimate of 0.55 × font size per
character, or 0.6 for bold. Five figures were regenerated:

- `architecture.svg`
- `milestones.svg`
- `training-pipeline.svg`
- `serving-pipeline.svg`
- `geometry-stack.svg`

Two are new: `geometry/compliant-path.svg` and `geometry/operators.svg`. All seven were inspected rendered
in a browser at full width. The mathematical figures (`docs/figures/geometry/generate.py`, Python standard
library) are unchanged. Porting them to Rust is a recorded follow-up.

## Checks run at the delivery head

Listed in the PR body with their output.

## Limits

- The README's statistics are a snapshot from 13 October 2026, and its milestone numbers are copied from
  the milestone issues and the registry. Both go stale by design, and [bookkeeping](../../bookkeeping.md)
  says who refreshes them and when.
- "Source, tested" in the current-path table means merged code with focused tests. It does not mean a
  trained or device-qualified operator, and it is not a capability claim.
- The README-keeper role is a proposal to the PM, not a ratified rule.

**Next:** the PM decides whether to adopt [bookkeeping](../../bookkeeping.md) into the shared plan. Separately,
Claude re-reviews the amended rung-0 sheet.
