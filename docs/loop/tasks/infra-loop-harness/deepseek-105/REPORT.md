# REPORT: image-item registration repair, deepseek-105

> **Superseded by [deepseek-106](../deepseek-106/REPORT.md).** A fresh review
> found the diff-size prose stale, this report's final-check wording still
> ambiguous about the receipt-only commit, two issue requirements generalized,
> and the PR-body links unsuited to copied Markdown.

Lab: deepseek. Item:
[infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Order: [ORDER.md](ORDER.md). Base:
`9e0403b132da34752499c657def95dfce3235538`.

## What did not work, first

- A fresh review found the image item's acceptance text required wiring
  `uor-r4-vision` into the integer API but its scope omitted
  `crates/uor-r4-integer/Cargo.toml`. A future legal implementation would
  otherwise stop on an out-of-scope manifest edit.
- The composer source link used `../loop/...`, which resolved under
  `docs/roadmap/loop/` instead of `docs/loop/`.
- The superseded report said full gates ran "at the head" even though its
  receipts were committed one documentation-only commit later. The timing was
  valid, but the wording hid that distinction.

## What changed

Added `crates/uor-r4-integer/Cargo.toml` to the image item's scope, corrected
the composer report link to `../../loop/...`, and marked deepseek-104
superseded. Its checks now identify implementation head `42f228e5b` and the
later receipt-only commit explicitly. The acceptance requirements, status and
implementation boundary are unchanged.

The cumulative diff at this head was 9 files and 332 insertions.

## Measurements

No model or serving measurement applies. Registration remains one `ready` item
mapped to issue #3619. The change enables a future implementation order to add
the vision dependency and native route without widening scope.

## Checks run at the head

- `cargo run -q -p xtask -- roadmap check`: PASS.
- The corrected composer-report target exists. Repository-wide
  `cargo run -q -p xtask -- links` reports the project's existing broken-link
  backlog as advisory output.
- `git diff --check`, `loop report --check`, `loop pr-check --local` and
  `loop gates --full`: PASS.
- `geometric-image-input` in `docs/roadmap/issues.json` maps to #3619.

## Skills used

No available Codex skill applied.

## Still open

- Implement the deterministic integer RGBA-to-R4 core under the repaired scope.
- Add bounded native `uor_r4_image` ingestion and Studio Canvas input.
- Run real-browser, native/WASM and trained visual-understanding qualification.

## Plain summary

The image roadmap item asked for code in a crate whose manifest its scope did
not allow editing. This repair adds that manifest to the scope, fixes a broken
relative link, and clarifies which commit carried the gate receipts. It does
not implement images or change the promised behavior.
