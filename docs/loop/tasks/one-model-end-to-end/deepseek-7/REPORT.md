# REPORT: capacity-sheet harness shape repair, deepseek-7

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base:
`4b0fb8028091e85c5e6c0a34ab0c632b7ccad498`.

## What did not work, first

- **Claude's merged capacity sheet broke the repository-wide harness with a
  documentation-shape error.** It lacked the script-discoverable headings
  `Base artifact`, `Data split`, `Bar`, `Steps` and `Minimum test power`, plus
  the literal `TEST FITNESS` line.
- **This was not a model or experiment failure.** The sheet's arm definitions,
  data pins, decision table, stop rule, readings and REPORT are unchanged.
- **No new FIT decision was made.** The added verdict lines restate the
  existing #3407 review recorded by Claude's REPORT.

## What changed

- Added `TEST FITNESS: FIT`, `PRE-RUN: PASS` and a shape-only amendment note.
- Added a discoverable `Base artifact` section describing the fixed recipe,
  build and output-store identity.
- Renamed `Data` to `Data split` and `Decision table` to `Bar`, leaving their
  contents intact.
- Added `Steps` and `Minimum test power` summaries derived only from the
  existing arms, instrument checks and recorded R1 power.

No arm, threshold, result, cost, stop rule or claim was changed.

## Measurements

No model compute ran. This unit is documentation-shape repair only.

## Checks run at the head

- `uor-harness prep-check`: FIT, complete, `may_compute=true`.
- `cargo test --locked -p uor-r4-harness`: 677 library tests passed, 1 ignored;
  all integration tests passed.
- `git diff --check`: passed.

## Skills used

No available Codex skill applied. The harness shape rule and the existing sheet
were read directly.

## Still open

Nothing in this documentation-shape unit. The capacity sweep's model result,
the width-512 export follow-up and the next copy-key experiment remain Claude's
research track.

## Plain summary

The capacity experiment was intact, but its pre-registration sheet no longer
matched the repository parser's required layout. This change adds the missing
labels and concise source-derived summaries without altering the experiment.
