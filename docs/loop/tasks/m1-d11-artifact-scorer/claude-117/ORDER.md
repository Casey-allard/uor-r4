# ORDER: m1-d11-artifact-scorer, claude-117 (repair)

Lab: claude. Item: [m1-d11-artifact-scorer](../../../../roadmap/items/m1-d11-artifact-scorer.md). Base: `dd89bb1c6c96cde030f771c1b30f07d4a1bba8c9`.

## Goal

This repairs the merge of #3249 (claude-116). `uor-harness loop close --verify --pr 3249` failed its content step: "a file the PR changed differs between dd89bb1c6… and the PR head 7e04f7f4f…, and the merge's patch is not the PR's". The file is `docs/roadmap/README.md`.

Two PRs re-rendered its status counts from the same base, 24 ready and 52 done:

- antigravity's #3245, merged first as `d60adcda6`, which set `recurrence-gates-six-recurrence-fit` to done;
- #3249, which set `m1-d11-artifact-scorer` to done.

Each moved the counts to 23 and 53. Git took the two identical edits as one, so main now reads 23 ready and 53 done. A render on main gives 22 and 54. No other file of #3249 differs on main.

This repair re-renders `docs/roadmap/README.md` on main with `cargo run -p xtask -- roadmap render` and changes nothing else.

## Cancel probe (run first)

Class: instrument (a generated view, not a result). Run at `dd89bb1c6`; all must hold:

- **Main's render is stale.** In a detached checkout of `dd89bb1c6`, `roadmap render` changes only the two count rows, `ready` 23 → 22 and `done` 53 → 54. **Holds.**
- **No other lab has already fixed it.** No open PR touches `docs/roadmap/README.md` for this.
- **No other file of #3249 was lost.** Of the 22 files #3249 changed, all but `docs/roadmap/README.md` are identical at `dd89bb1c6` and at its head `7e04f7f4f`. **Holds.**

## Files in scope

- `docs/roadmap/README.md` (generated)
- `docs/loop/tasks/m1-d11-artifact-scorer/**`

## Done when

- `docs/roadmap/README.md` equals a fresh `roadmap render` of main. The only lines that change are the two count rows.
- A REPORT records the cause, and that #3249's verify cannot pass after the fact: its squash dropped the duplicate count edit, and this PR restores it.
- The PR goes through the exact-head review, `loop close --session`, `loop queue` and `loop close --verify`, with `References #3248` (the item is already done).

## Stop and ask if

Stop and decide, never stop and wait.

- **Another merge re-renders the file first.** Then re-render on the new main; if nothing is left to change, close this PR as superseded.
- **The render changes any line other than the counts.** Then the drift is not this one, and it gets its own ORDER.
