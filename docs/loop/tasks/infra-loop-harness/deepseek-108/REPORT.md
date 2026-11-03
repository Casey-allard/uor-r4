# REPORT: correct the recorded base, deepseek-108

> **Superseded by [deepseek-109](../deepseek-109/REPORT.md).** A final review
> found the following commit's report-change description incomplete and the
> PR-body ORDER link premature. The verified base and gate receipt are retained.

Lab: deepseek. Item:
[infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Order: [ORDER.md](ORDER.md). Base:
`9e0403b132da34752499c657def95dfce3235538`.

## What did not work, first

All records created by deepseek-104 through deepseek-107, and the copied PR
body, named `9e0403b13e72cdc9cb840e9b62898ec33e0d614d`. Git reports that object
does not exist. The actual merge base is
`9e0403b132da34752499c657def95dfce3235538`.

The mistaken digest had the correct leading bytes and prefix but transposed
trailing digits, so visual review did not catch it. Object lookup and a direct
merge-base comparison do.

## What changed

Corrected the base line in every task ORDER and REPORT from deepseek-104 through
deepseek-107, and created this active record with the verified base. The image
item, requirements, scope, implementation plan and evidence are unchanged.

The exact PR diff is 18 files: 653 insertions and no deletions.

## Measurements

No model or serving measurement applies. The verified facts are:

- merge base and current `origin/main`:
  `9e0403b132da34752499c657def95dfce3235538`;
- invalid recorded base `9e0403b13e72...`: not present in the object database;
- image issue mapping remains `geometric-image-input -> 3619`.

## Checks run at the head

- `git merge-base HEAD origin/main` and commit-object validation: PASS.
- Full gates ran at content commit
  `76a4c4e241650a38d56dfdaf0d6559a7f7485f24`.
- `cargo run -q -p xtask -- roadmap check`: PASS.
- `git diff --check`, `loop report --check`, `loop pr-check --local` and
  `loop gates --full`: PASS.
- The final accounting commit adds the receipts file, records the tested
  content commit and updates this report's exact-size line.

## Skills used

No available Codex skill applied.

## Still open

- Implement the deterministic integer RGBA-to-R4 core.
- Add bounded native `uor_r4_image` ingestion and Studio Canvas input.
- Run real-browser, native/WASM and trained visual-understanding qualification.

## Plain summary

The image item is correct, but its task records named a base commit that does
not exist. This repair replaces that digest everywhere with the verified merge
base. No product behavior or implementation plan changed.
