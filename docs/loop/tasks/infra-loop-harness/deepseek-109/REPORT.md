# REPORT: final image-registration accounting, deepseek-109

> **Superseded by [deepseek-110](../deepseek-110/REPORT.md).** A final review
> found the source item was present but its generated roadmap view had not been
> refreshed.

Lab: deepseek. Item:
[infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Order: [ORDER.md](ORDER.md). Base:
`9e0403b132da34752499c657def95dfce3235538`.

## What did not work, first

The previous closing commit was described as adding only receipts and a size
update, but it also added the tested content-commit line. The copied PR body
also linked the new ORDER path through `main`, where the file does not exist
before merge.

## What changed

The deepseek-108 report now enumerates all three final-commit changes and points
to this active record. The PR body will name the active ORDER as an inline path
instead of linking a pre-merge `main` URL.

The image item, source code, scope and acceptance criteria are unchanged.

## Measurements

No model or serving measurement applies. The verified base remains
`9e0403b132da34752499c657def95dfce3235538`; the image issue mapping remains
`geometric-image-input -> 3619`.

## Checks run at the head

- Full gates ran at content commit
  `34c1912e3f24156a14eef48f889b4e65bf3edad9`.
- `cargo run -q -p xtask -- roadmap check`, `git diff --check`,
  `loop report --check`, `loop pr-check --local` and `loop gates --full` passed
  at that content commit.
- The following accounting commit adds `receipts.jsonl`, records the tested
  content commit here and updates the exact-size line.
- The final cumulative diff is 21 files: 753 insertions and no deletions.

## Skills used

No available Codex skill applied.

## Still open

- Implement the deterministic integer RGBA-to-R4 core.
- Add bounded native `uor_r4_image` ingestion and Studio Canvas input.
- Run real-browser, native/WASM and trained visual-understanding qualification.

## Plain summary

The image registration itself is correct. This final repair makes the closing
commit description complete and removes a link that could not work until after
merge. No behavior or scope changed.
