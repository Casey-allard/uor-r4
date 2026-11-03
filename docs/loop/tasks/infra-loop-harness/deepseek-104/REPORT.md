# REPORT: recover the image-decomposition item, deepseek-104

> **Superseded by [deepseek-105](../deepseek-105/REPORT.md).** A fresh review
> found the registered item scope incomplete, one relative link wrong, and this
> report's gate wording ambiguous about the later receipt-only commit.

Lab: deepseek. Item:
[infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Order: [ORDER.md](ORDER.md). Base:
`9e0403b132da34752499c657def95dfce3235538`.

## What did not work, first

- **The advertised item and task were missing.** Open issue #3619 contains the
  complete image-decomposition request and marker
  `roadmap:id=geometric-image-input`, but
  `docs/roadmap/items/geometric-image-input.md`, its issue mapping and
  `docs/loop/tasks/geometric-image-input/kimi-1/ORDER.md` are absent from
  `main`. The referenced branch is also absent from the remote.
- **Automated claim hit the recurring trailer defect.** `loop claim
  infra-loop-harness --lab-topic geometric-image-registration` created a
  skeleton commit without `Lab:` and `Order:` trailers, the hook refused it,
  and the tool rolled back.
- **The issue cannot itself authorize an implementation unit.** The loop needs
  the item in `main` before a scoped implementation ORDER can be claimed, so
  this unit restores only the registration.

## What changed

Added the missing `geometric-image-input` roadmap item with issue #3619's
required integer decomposition, native API, Studio Canvas input, evidence and
truthful no-visual-understanding boundary. Its scope names a future
`uor-r4-vision` crate, the integer API files, Studio assets/tests and the
product documentation without a root-wide glob.

Added `geometric-image-input: 3619` to the generated issue map. No implementation
code, model, tokenizer, API behavior, generated roadmap view or item status was
changed. The item is ready for a later implementation ORDER.

## Measurements

No model or serving measurement was made. Registration coverage is:

- `cargo run -q -p xtask -- roadmap check`: **139 items OK**;
- item status distribution includes the new item as `ready`;
- `geometric-image-input` maps to issue #3619 in `docs/roadmap/issues.json`.

## Checks run at the head

- `cargo run -q -p xtask -- roadmap check`: PASS.
- `git diff --check`: PASS.
- At implementation head `42f228e5b`, `uor-harness loop report --check`,
  `loop pr-check --local` and `loop gates --full` passed. Head `0f92abc70`
  then added only this task's receipts and branch claim, so the retained gate
  receipts intentionally predate that receipt-only commit.

## Skills used

No available Codex skill applied.

## Still open

- The actual `uor-r4-vision` integer decomposition core.
- Native `POST /v1/images/decompose` and validated `uor_r4_image` parts.
- Studio Canvas paste/drop/picker previews and explicit-send image context.
- Real browser, native/WASM and trained visual-understanding qualification.

## Plain summary

The project had an open issue asking for geometric image decomposition, but the
roadmap file that gives the issue its implementation scope had been lost before
reaching `main`. This change restores that file and its issue mapping. It does
not implement image support; it makes a correctly scoped implementation unit
possible.
