# ORDER: m1-address-as-read, deepseek-72 (repair after the counted CHANGES on #3140)

Lab: deepseek. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Base: `dc5082f03`. Repair for PR #3140.

## Why this repair

The independent subagent review at `6c3963909` returned a counted **CHANGES** with two wording contradictions left by deepseek-71:

1. The lab README (and the sheet's registration bullet at :148) still said the PM "still selects the data source and freezes/hashes the panel" immediately before saying the panel is frozen (`panel-v2`, `369d8c24…`) — self-contradictory.
2. The sheet's verdict paragraph (:150) still repeated the old "from Claude or Kimi — deepseek has authored text in this sheet" rule without the claude-71 supersession marker.

## Fix

1. README + sheet :148: make the PM's selection/freeze **past tense** and record the frozen `panel-v2` result.
2. Sheet :150: mark the old FIT-source parenthetical as **superseded by the claude-71 fresh-context rule**, keeping the rest of the verdict sentence.

Both are wording-only; no rule, value, verdict, bar, tolerance, checkpoint, panel, hold or item status changes.

## Files in scope

- `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`
- `docs/labs/m1-address-as-read/README.md`
- this task folder (deepseek-71 stays immutable history)

## Done when

- A repo-wide grep finds no live statement that the PM still selects/freezes the panel, and no undated "from Claude or Kimi" FIT-source rule; the historical deepseek-38 sentences remain marked superseded.
- `loop report --check`, `loop pr-check --local`, `git diff --check` pass and a fresh independent review records at the new head.

## Stop and ask if

- A verdict/bar/tolerance/checkpoint/panel/hold changes, or a file outside the list is needed.
