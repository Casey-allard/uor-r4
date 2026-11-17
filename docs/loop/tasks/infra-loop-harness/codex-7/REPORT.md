# REPORT for HANDOFF decision ownership

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). [ORDER](ORDER.md). References #3380.

## What did not work

The existing renderer printed owner for every decision, even when the source assigned it to a lab or left the assignment unspecified. The revised fixture failed on the old renderer with one failure, expecting lab and receiving owner. A preview at the real base also printed owner for the M2 multi-turn bar despite its source declaring open_to: lab.

## What changed

The blocked table now displays its item's open_to field for decisions and held items. Missing assignments render as “not recorded.” The fixture covers a lab decision, an explicit owner decision, a missing decision assignment and held items assigned to a lab or the owner.

## Measurements

With the same roadmap input at `16b088064183fce67458e4e9d6bdae916dd096ea`, the before and after previews differ in two lines of HANDOFF: the column explanation and the M2 multi-turn row's owner. That row now reads lab. All 31 integration tests for the renderer passed after the repair.

## Checks run at the head

On Rust 1.97.1, run `cargo test --locked -p uor-r4-harness --test loop_render`. The failing-first filter was `handoff_lists_open_items_by_status_with_not_recorded_for_absent_fields`. To examine the real row, use the repaired harness to run `loop render --at 16b088064 --out <temporary-directory>` and read the M2 multi-turn entry in its HANDOFF.md. Package formatting and whitespace checks passed.

## Skills used

Inspection of the roadmap item and its normal renderer, with a failing-first fixture and a commit-pinned before/after comparison.

## Still open

Publication of the tracked generated views and the final issue readback belong to the public documentation unit. This source repair depends on its registration of the renderer source path.

## Plain summary

HANDOFF now names the lab responsible for the M2 multi-turn decision and keeps explicit owner assignments visible.
