# REPORT: infra-loop-harness, claude-19 (mark merged loop items done)

Lab: claude. Base: `1b2f5c9d1`. Order: [ORDER.md](ORDER.md), committed first (`032c0ea17`). The cancel probe found all six items still `status: ready`, and their issues open with no comments.

## What did not work, first

- **The items were never closed.** Loop PRs 2, 4, 5, 6, 7a and 7b each said `References #<item>` rather than `Closes #<item>`, and none of them set its item to `status: done`. So all six issues stayed open after the merges, with no comment on any of them at the probe. Claude posted a delivery comment on each after the base commit. The roadmap page says a finishing PR uses `Closes #N`, and that an issue is claimed with a comment before work starts. Claude did neither.
- **The tool repeats the slip.** `loop close --session` builds the PR body with `References #<issue>` for the ORDER's item and never `Closes`, so a lab following the loop leaves its item open too.
- **Loop PR 3 is not finished.** Its done-when names `docs/loop/baseline.json`, which does not exist on `main`, so #2906 stays open.

## What changed

- Six item files changed one line each, from `status: ready` to `status: done`. Each item's merge commit is on `origin/main`:

  | Item | Issue | PR | Merge commit |
  |---|---|---|---|
  | infra-loop-pr02-item-keys | #2905 | #2920 | `501349aaf` |
  | infra-loop-pr04-gates | #2907 | #2921 | `32092fca6` |
  | infra-loop-pr05-policy-check | #2908 | #2924 | `8c2c76e42` |
  | infra-loop-pr06-review-records | #2909 | #2926 | `1b2f5c9d1` |
  | infra-loop-pr07a-claim | #2910 | #2922 | `24130f6df` |
  | infra-loop-pr07b-session | #2911 | #2925 | `987f72259` |

- `docs/roadmap/README.md` was re-rendered by `roadmap render`: 51 ready and 6 done, and the six rows moved to the end of the table (`4e7f08d08`).
- This PR closes the six issues: closes #2905, closes #2907, closes #2908, closes #2909, closes #2910, closes #2911.
- The planted fault, an item marked done before its PR merged, is answered by the table above: `git merge-base --is-ancestor <commit> origin/main` holds for all six.

## Measurements

None. This order changes roadmap status only.

## Checks run at the head

- `roadmap check`: 87 items OK (ready 51, held 26, decision 4, done 6). `roadmap render --check` passes.
- Claim-wording passes. `xtask links` finds 0 broken links in the changed files. The copyright check finds 0 matching 8-word runs.
- `loop pr-check --local` fails only `scope`, as for every loop PR until scopes are backfilled.

## Skills used

None beyond the delivery path in CHARTER §5.5.

## Still open

- The closing keywords in this PR close the six issues at merge, so the ORDER's last done-when changes: the `roadmap sync` dry run after merge should list no change for these six, and the applied sync then confirms them closed.
- #2906 (loop PR 3) needs `docs/loop/baseline.json` from `loop render`.
- `loop close --session` should write `Closes #<issue>` when the diff sets the ORDER's item to `status: done`, and `References` otherwise. This is a code change, for loop PR 7c or its own order.
- From now on, each loop PR claims its issue with a comment, says `Closes #<item>`, and sets its item to `done` in the same PR.

## Plain summary

Six pieces of the loop tool were finished and merged, but their tracking issues were never closed. This change marks them done so the roadmap tool can close them. One piece, loop PR 3, stays open because one file it promised is still missing.
