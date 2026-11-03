# REPORT: repair the served-rounding item registration, deepseek-79

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Order: [ORDER.md](ORDER.md). Base: `a9d0461e2`.

## What did not work

- **The first review found the repair scope incomplete.** Profiling can select
  activation or table-copy work, but the new item named only the integer
  snapshot file. It now names the activation, export-rounding and served-loop
  sources that own those mechanisms.
- **The first acceptance text allowed a failed rerun to count as done.** It now
  requires both seeds to pass the instrument and 2x operational doors, with the
  registered quality-smoke rule not finding served materially below plain.
- **The first report accounting was stale.** It counted partial commits rather
  than the exact PR diff. The deepseek-78 REPORT now names the final file and
  line totals.
- **The second review found two stale report sentences.** deepseek-80 qualifies
  the initial scope and removes the already-completed push from this report's
  open list.

## What changed

`docs/roadmap/items/served-rounding-cost.md` now scopes
`crates/uor-r4-float-train/src/fbs_export.rs`,
`crates/uor-r4-float-train/src/hamilton_sheet.rs` and
`crates/uor-r4-float-train/src/in_space_long.rs` in addition to the integer
snapshot quantizer. This registration still changes no model, artifact, rule or
serving behavior.

Size: this repair adds one ORDER and this REPORT and updates the item and
deepseek-78 REPORT. The complete PR size is reported by the pre-push loop gate.

## Measurements

No new performance measurement was made for this documentation repair. The
diagnostic values and their limits remain in the deepseek-78 REPORT. CUDA
profiling and the four-job rerun remain `NOT_RUN`.

## Checks run at the head

- `cargo run --locked -p xtask -- roadmap check`: 135 items OK.
- `cargo run -q --locked -p uor-r4-claim-wording -- .`: passed.
- `uor-harness loop gates --quick`: all three gates green.
- `git diff --check origin/main...HEAD`: passed.

## Skills used

No available Codex skill applied. The repair follows the fresh reviewer's
concrete findings and the repository loop records.

## Still open

- Obtain a fresh exact-head review and green required checks at the final head.
- Merge and verify the new item on fresh `main`.

## Plain summary

The first version of the new work item pointed only at one source file even
though profiling could send the repair into other files, and it did not
explicitly require the 2x speed gate to pass. This repair fixes both gaps and
corrects the record's size accounting.
