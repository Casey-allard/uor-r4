# REPORT: infra-loop-harness, claude-41 (give m1-a8-erosion-diagnosis its notebook page)

Lab: claude. Base: `b00d9edad`. Order: [ORDER.md](ORDER.md), committed first (`573ecfbdb`). The cancel probe found that the item declared no `notebook:` field and did not list the notebook in `scope:`, so the task stands.

## What did not work, first

- **The item asks for a notebook row its scope forbids.** Step 6 says "Record the result as one row in `docs/research/notebook/m1-discrete-attention.md`", but the page is outside `scope:`, and no roadmap item declares the validated `notebook:` field that `pr-check` honours (`loopprcheck.rs:468`). This item could not have closed without this edit.

## What changed

| Commit | What |
|---|---|
| `573ecfbdb` | This ORDER, alone. |
| `511619774` | `docs/roadmap/items/m1-a8-erosion-diagnosis.md`: one added line, `notebook: docs/research/notebook/m1-discrete-attention.md`. |
| this REPORT's commit | This REPORT. |

## Measurements

None.

## Checks run at the head

- `cargo xtask roadmap check`: 88 items OK.
- `roadmap render --check`: README current.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one line of one item file.

## Still open

- **The closing unit of `m1-a8-erosion-diagnosis`** adds the notebook row, flips the status and closes #2829. It comes after #2996 and this PR merge.
- **Other items' step 6 may hit the same gap.** All 88 items lack `notebook:`. A follow-up could add the field to each item whose text names a notebook page.

## Plain summary

The diagnosis task told us to log its result in the attention notebook but did not let us edit that page. This one line lets it.
