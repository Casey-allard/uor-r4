# REPORT: setup-row chain repair, deepseek-111

> **Superseded by [deepseek-112](../deepseek-112/REPORT.md).** A final review
> found the ORDER chain still omitted the charter-required cancel probe.

Lab: deepseek. Item:
[infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance).
Order: [ORDER.md](ORDER.md). Base:
`9e0403b132da34752499c657def95dfce3235538`.

## What did not work, first

The fresh review of deepseek-110 approved the content, but the counted-review
command refused it because the manually written maintenance ORDER chain did not
name the required setup row.

## What changed

Added `Setup row: 2 (item-file maintenance)` to every ORDER from deepseek-104
through deepseek-111. The image item, generated roadmap, requirements, links,
mapping and implementation plan are unchanged.

The exact PR diff is 28 files: 973 insertions and 2 deletions.

## Measurements

No model or serving measurement applies. This remains a documentation-only
registration for the deterministic image-decomposition item.

## Checks run at the head

- Every ORDER from deepseek-104 through deepseek-111 names setup row 2.
- The final generated roadmap reports 24 ready/139 total.
- `roadmap render --check`, `roadmap check`, `git diff --check`,
  `loop report --check`, `loop pr-check --local` and `loop gates --full` pass.
- Full gates ran at content commit
  `afdfd0f7fcc8f1dc48c14af1706c9193f9548624`.
- The following accounting commit adds its receipts file, records that tested
  content commit here and updates this report's exact-size line.

## Skills used

No available Codex skill applied.

## Still open

- Implement the deterministic integer RGBA-to-R4 core.
- Add bounded native `uor_r4_image` ingestion and Studio Canvas input.
- Run real-browser, native/WASM and trained visual-understanding qualification.

## Plain summary

The image registration itself was complete, but the harness could not accept a
review because every manually written maintenance ORDER lacked the required
setup-row marker. This repair adds the marker across the chain. No product
behavior or item scope changed.
