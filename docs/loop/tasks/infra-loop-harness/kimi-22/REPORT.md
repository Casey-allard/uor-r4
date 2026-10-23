# REPORT: infra-loop-harness, kimi-22 (m4-flock-coupling-prereg item names the current draft)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (PM maintenance of item files, as with kimi-2's scopes/closes edits).
Base: `7080fd6cf`. ORDER: [ORDER.md](ORDER.md), committed first (`2be984abb`).

## What did not work, first

- The item file `docs/roadmap/items/m4-flock-coupling-prereg.md` line 23 named the superseded 18 October draft (`m4-c1-flock-coupling-DRAFT-2026-10-18.md`) as "the draft", sending the reader to the wrong file. The current sheet is the 22 October successor (`m4-flock-coupling-prereg-DRAFT-2026-10-22.md`, antigravity-25), which declares the supersession and retains the 10-18 draft byte-identical.
- kimi-21's proposed scope widening for this item was withdrawn: the item's scope glob (`docs/harness/prep/m4-flock-coupling-prereg*`) already covers the 10-22 sheet, so no scope change was needed — only this prose pointer was stale.

## What changed

| Commit | What |
|---|---|
| `2be984abb` | The ORDER, alone. |
| `433ea83ac` | The item file, line 23: now names the 10-22 sheet as the current draft and notes it supersedes the 10-18 draft (retained byte-identical). |
| this REPORT's commit | This REPORT. |

## Measurements

- Class **instrument** (docs metadata; no GPU, no model compute), matching the ORDER's cancel probe (`grep -n "m4-c1-flock-coupling-DRAFT-2026-10-18" docs/roadmap/items/m4-flock-coupling-prereg.md` found line 23 at the base).
- Verified before editing: the 10-22 sheet's status block declares the supersession, its status is still `TEST FITNESS: PENDING`, and its §7 carries the same five open PM/owner decisions the item lists — so the rest of the sentence and the bullet list remain accurate.
- Diff size outside this task folder: 1 file, 1 line.

## Checks run at the head

- `uor-harness loop report --check` and `loop pr-check --local`: run in this worktree before the PR opened; results in the PR body. The item file is in the umbrella's scope.

## Skills used

- None beyond the loop flow itself.

## Still open

- The five §7 decisions on the 10-22 sheet remain unregistered; that is deepseek-47's PR #3052, whose NOT_FIT verdict must land on the 10-22 sheet per claude's standing CHANGES.
- Cards #3041 and #3042 (harness bugs in `prep.rs`) remain open and unassigned; both touch `prep.rs`, outside the umbrella's scope.

## Plain summary

The m4-flock-coupling-prereg item pointed at a draft that was superseded on 22 October. It now points at the current sheet, with the supersession noted.

References #2904
