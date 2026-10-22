# REPORT: infra-loop-harness, claude-30 (end D59, at the owner's direction)

Lab: claude. Base: `4bb1feede`, merged forward to `603acff74` (claude-29, #2971, which put these paths in the umbrella's scope). Order: [ORDER.md](ORDER.md), committed first (`0d4ab97b2`).

The cancel probe found no `docs/harness/d59.json` at the base, so D59 was in force.

## What did not work, first

- **D59 had no timeout.** Setup PRs counted only Claude's record. From about 08:24Z to 12:35Z on 22 Oct, Claude's session was blocked on an owner question. Kimi's #2954 and #2959 waited on it, and DeepSeek's #2949 waited on those. The owner ended D59 at 2026-10-22T13:56:44Z: "End D59 now".
- **The existing switch test pinned the old state.** `the_switch_ends_d59_only_when_listed_and_ended` asserted that the repository's real `owner-paths.json` keeps D59 in force. Ending D59 changes that specified behaviour, so the assertion now reads the repository's own listing and switch, and expects D59 ended. With the new `owner-paths.json` line set aside, the test fails at `loop_review.rs:364`; restored, it passes.
- **My first `sed` for the `DECISIONS.md` row failed** because its replacement contained `#` ("#2028"), which was also the delimiter. Only the `owner-paths.json` edit had applied, and the rest were redone with a different delimiter. All five edits are checked in the diff.

## What changed

| Commit | What |
|---|---|
| `0d4ab97b2` | This ORDER, alone. |
| `0f2d7ba01` | `docs/harness/d59.json` (`"d59": "ended"`, the owner's words, the time and the #2028 record) is listed in `docs/harness/owner-paths.json`. D59's `DECISIONS.md` row is marked ended in its reversal cell. `AGENTS.md:19` and `docs/loop/README.md:45`/`:219` say D59 ended. The switch test is updated. |
| `e1279abd1` | Merged `origin/main` (claude-29's scope). |
| this REPORT's commit | This REPORT. |

No code changed. `d59_in_force` (`loopreview.rs:319`) reads both files at the merge base. The loop page's conditional lines ("while D59 is in force", `:63`, `:65`, `:241`, `:244`) stay true and are unchanged.

## Measurements

None. This is a governance switch, with no model run.

## Checks run at the head

- `loop pr-check --local`:
  - `scope`: PASS, all 7 paths are inside `infra-loop-harness`'s scope.
  - `commit-order`: PASS.
  - `owner-paths`: PASS ("owner approval recorded for AGENTS.md, docs/integration/DECISIONS.md, docs/harness/owner-paths.json").
- `cargo test -p uor-r4-harness --test loop_review`: 27 passed. The owner-path existence test passes (1).
- Claim-wording finds nothing in the changed files, and `xtask links` finds 930 broken, as on `main`. The copyright result is in the PR body.

## Skills used

scope-guard: two JSON files, three governance lines, one test assertion, and this task folder.

## Still open

- **After this merges,** any other vendor's fresh-context record counts on setup PRs. claude-31 (no idle stops) and every later setup PR take review from Kimi, DeepSeek, Antigravity or Codex.
- Kimi records the D59 change in the PM notes.

## Plain summary

Until now, every change to the loop's own setup waited for Claude's review alone, so one busy reviewer could stall everyone. At Casey's word this ends that rule, and any other lab can now review those changes.
