# REPORT: cancel-probe chain repair, deepseek-112

Lab: deepseek. Item:
[infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance).
Order: [ORDER.md](ORDER.md). Base:
`9e0403b132da34752499c657def95dfce3235538`.

## What did not work, first

The final review confirmed the registration, generated roadmap, scope and
accounting, but found that repair ORDERs 105 through 111 lacked the
charter-required cancel-probe section. Mechanical `pr-check` does not detect
that omission.

## What changed

Added an explicit cancel probe to every repair ORDER from deepseek-105 through
deepseek-112. Each records the reproduced condition at its starting head and
why the unit stands. The image item, generated roadmap, requirements, scope,
mapping and implementation plan are unchanged.

The exact PR diff is 31 files: 1129 insertions and 2 deletions.

## Measurements

No model or serving measurement applies. The registration remains one `ready`
image-input item mapped to issue #3619.

## Checks run at the head

- Every ORDER from deepseek-104 through deepseek-112 has a non-empty cancel
  probe.
- `roadmap render --check`, `roadmap check`, `git diff --check`,
  `loop report --check`, `loop pr-check --local` and `loop gates --full`: PASS.
- Full gates ran at content commit
  `e2c5d69145d68208dd55a96c7bbfcfa8007abe2e`.
- The following accounting commit adds its receipts file, records that tested
  content commit here and updates this report's exact-size line.

## Skills used

No available Codex skill applied.

## Still open

- Implement the deterministic integer RGBA-to-R4 core.
- Add bounded native `uor_r4_image` ingestion and Studio Canvas input.
- Run real-browser, native/WASM and trained visual-understanding qualification.

## Plain summary

The image registration and its generated roadmap were correct. The repair
orders explaining the registration fixes still lacked a required statement of
what would cancel each repair. This record adds those cancel probes without
changing the item or promised behavior.
