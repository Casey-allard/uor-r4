# REPORT: infra-loop-harness, claude-10 (the 14 loop items are GitHub issues)

Lab: claude. Base: `9aa7ad275` (#2903 merged). Order: [ORDER.md](ORDER.md), committed (`54cb922f4`) before the dry run. Its cancel probe found no `infra-loop` key in `docs/roadmap/issues.json`, and no issue carrying an `infra-loop` marker. A GitHub search returned four loose full-text matches, each carrying a different item's marker.

## What did not work, first

- **The sync cannot be limited to chosen items.** `cargo xtask roadmap sync` plans every item. Besides the 14 creates, the dry run planned one update the owner had not named: the body of #2839 (`m1-export-gap-and-toks`). #2894 edited that item's file at 23:54Z on 20 Oct, after the issue was last synced at 21:40Z, so the issue had fallen behind `main`.
  - Per the ORDER, the apply waited for the owner. The owner chose "Include #2839 (Recommended)", an option worded by Claude.
  - The ORDER was widened (`623082bb7`) before the apply.

## What changed

**On GitHub, done by the apply:**
- **14 new issues:**
  - #2904 `infra-loop-harness`
  - #2905 `pr02-item-keys`
  - #2906 `pr03-render`
  - #2907 `pr04-gates`
  - #2908 `pr05-policy-check`
  - #2909 `pr06-review-records`
  - #2910 `pr07a-claim`
  - #2911 `pr07b-session`
  - #2912 `pr07c-queue-board`
  - #2913 `pr08-migration`
  - #2914 `pr09-client-adapters`
  - #2915 `pr10-supervisor`
  - #2916 `pr11-progress-window`
  - #2917 `pr12-git-hooks`

  Each one is open, carries its `<!-- roadmap:id=… -->` marker and the `roadmap:item` label, and was checked one by one after the apply.
- **Depends-on numbers.** The sync's designed second pass filled in the "Depends on" numbers in 11 of the new issues.
- **#2839.** Its body now matches its item file on `main`.
- **Nothing else.** No labels were created, no issues closed, and no milestone checklist changed.

**In the repository:**

| File | Class | What |
|---|---|---|
| `docs/roadmap/issues.json` | generated | 14 keys added (87 in total). None removed, none changed. |
| `docs/roadmap/README.md` | generated | 14 rows go from "not yet synced" to their issue links. |
| `docs/loop/tasks/infra-loop-harness/claude-10/` | docs | This ORDER and REPORT. |

## Measurements

None. This was a sync, not a run.

## Checks run

- The apply exited 0.
- A second dry run planned "0 labels to create, 0 issues to create, 0 to update, 0 to close, 0 milestone checklists to change".
- `roadmap check` passed with 87 items OK, and `render --check` reported current.
- All 14 new issues passed the per-issue marker, label and open-state check.
- The PR body lists the rest: claim-wording, links, the harness real-repo tests and the local copyright check.

## Skills used

None of the private kit's skills.

## Still open

- **The new issues are in ready status but not yet runnable.** Builders start them in the D52 team order, and all reviews follow D59.
- **A correction made after the first review.** The ORDER first gave the owner's consent time as "about 08:00Z". That is later than the apply, which started at 07:56:41Z, while the consent was recorded at 07:56:27Z. A fresh-context review of `f507b569a` caught this. Since an ORDER is never changed after merge, the time was corrected before the merge.

## Plain summary

Every loop build step is now a GitHub issue that a lab can claim. The issue numbers are recorded in the repository.
