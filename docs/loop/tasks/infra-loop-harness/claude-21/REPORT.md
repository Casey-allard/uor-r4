# REPORT: infra-loop-pr07c-queue-board, claude-21 (loop PR 7c: loop queue and loop board)

Lab: claude. Base: `7864b0516`. Order: [ORDER.md](ORDER.md), committed first (`d52419621`). The cancel probe found that `loop queue` and `loop board` both printed "not built yet".

Two fresh-context builders each wrote one commit in parallel, one for `loop queue` and one for `loop board`. The main session folded both onto this branch, resolved the one conflict, re-ran every builder gate (the two negative gates included), and wrote the docs and this REPORT.

## What did not work, first

- **Review round 1 found a hole in my own design.** Three fresh-context lenses at `51d39b743` (the queue code, its tests, and the board with the docs and REPORT) each returned APPROVE WITH LOWS. The queue lens showed that architecture call 2 ("a `uor-review v1` record, valid or not, is left to the gate") let a hold through. Suppose the owner posts "Hold — the uor-review v1 APPROVE above missed X" after an APPROVE. That comment parses as a malformed record, the gate ignores malformed records, and the after-approval rule skipped it, so the PR would have been queued. `458068ff8` now refuses on any malformed record after the approval, and the ORDER's call 2 says so.
- **Today `loop queue` refuses every loop PR, this one included.** Every loop PR's policy record is `fail`, because its item has no `scope:` line, so `required-checks` never sees `policy: pass`. That is the lock working as the item asks, and it lasts until Kimi's scope PR merges. Until then, labs merge by hand under CHARTER §5.5, after a counted APPROVE.
- **One negative gate first missed the planted fault.** With only the `pr-check` refusal set aside, the planted-fault test still passed. `loop queue` refuses a later counted CHANGES twice:
  - through `pr-check`'s `review-record` rule;
  - again in `after-approval`, which re-reads the comments after the rules have run, so a CHANGES posted in between is still caught.

  With both set aside, the test fails. Neither path alone is pinned by a test that fails when only it is removed.
- **I pushed without running `pr-check --local`, and the ORDER was too long.** The ORDER's own done-when names `pr-check --local`, but it was first run (through the live `loop queue` dry run) only after the PR was open. Its `read-path` rule failed: the ORDER was 4,331 bytes, over the 4,096 limit. I shortened the Goal and call 4 to 3,986 bytes; what the ORDER asks for is unchanged.
- **The first full suite run failed one test.** 927 passed and 1 failed: `harnessrules::tests::the_real_repo_passes_the_ported_rules`. The docs check keeps its own list of `loop` subcommands, and neither builder had added `queue` or `board` to it, so the new README lines counted as commands that do not exist. Fixed in `222b2c84e`, which adds both to the list in `doccheck.rs`. The test now passes.
- **The top-level parser swallows `--json`.** Both builders had to add their command to the list in `main.rs` that forwards it. Their commits conflicted on that line; I resolved it by hand and updated the comment above it, which still named only three commands.
- **The board's claim counts are inflated.** It merges live claims with the ones committed in `docs/harness/claims.json`, so stale committed entries count too. On the first live run Claude showed 34 and Codex 36.
- **The board's first live run** showed Kimi, DeepSeek and Antigravity as STALE, about 58 hours since their last check-in. None has sent a heartbeat since joining the loop.

## What changed

| Commit | What |
|---|---|
| `59de8f082` | `loop board` (`loopboard.rs`, `tests/loop_board.rs`, 7 tests). It lists every lab in `labs.json` with vendor, client, role, presence, claims and its `[lab]`-tagged open PRs, and lists untagged PRs separately. Its GitHub trait has no write method. |
| `e5a0d6743` | `loop queue` (`loopqueue.rs`, `tests/loop_queue.rs`, 13 tests): `pr-state`, `head`, every `pr-check` rule, then `after-approval`, then the merge, which is its one write. Two helpers were factored rather than copied: `loopprcheck::evaluate` (now also used by `cli`) and `ReviewContext`, shared with `rule_review_record` with unchanged failure strings. Also new: `loopreview::last_counted_approve`, with a test in `tests/loop_review.rs`. |
| `222b2c84e` | Adds `queue` and `board` to the docs check's list of `loop` subcommands, so the README lines that document them pass `harness.documented_commands`. |
| `458068ff8` | The review fixes:<br>- **Fail closed:** a malformed `uor-review v1` record after the approval refuses.<br>- **Base:** `pr-state` also requires the PR's base to be `main`, read from `baseRefName`; a missing field exits 2.<br>- **Merge failure:** a failed merge gets its own `FAIL merge` line and verdict `MERGE-FAILED`, and exits 2.<br>- **Tests:** seven added to `loop_queue`, now 20 (closed PR, base not `main`, malformed record, labs read from `main` not the head, `policy: fail`, a run from another app, failed merge). One added to `loop_review`, now 27 (`last_counted_approve` on a loop item with no valid setup row). `loop_board`'s no-write test now snapshots the scratch live tier before and after the run. |
| `d52419621`, `b48d4b9f3`, `51d39b743` and the review-fix doc commit | The ORDER, this REPORT, and their later edits. |
| `8eb8e8067` | The first docs commit. The harness README gains the `loop queue` and `loop board` sections. The loop page's status line now says most commands are built, and step 12 and build-plan row 7c are marked built. The item reads `status: done`, and `docs/roadmap/README.md` is re-rendered. |

This PR closes the item's issue: closes #2912.

## Measurements

None. This is tooling, with no model run.

## Checks run at the head

- fmt is clean and whole-crate clippy with `-D warnings` is clean.
- `loop_queue` 20, `loop_board` 7, `loop_prcheck` 28 and `loop_review` 27 passed; the `pr-check` verdicts are unchanged.
- Negative gates, each run by the main session:
  - with both queue refusals set aside, `planted_fault_a_later_counted_changes_refuses` fails;
  - with `after-approval` made never to refuse, `an_untagged_comment_after_the_approval_refuses` fails;
  - with the `[<lab>]` title match set aside, the two attribution tests in `loop_board` fail;
  - with the malformed-record refusal set aside, `a_malformed_record_after_the_approval_refuses` fails;
  - with the base check set aside, `a_pr_not_based_on_main_refuses` fails;
  - all pass again once restored.
- Full harness suite at `222b2c84e`: 928 passed, 0 failed. The first run, before that fix, failed one test; see above. After the review fixes, at `5c536c3b8`: 936 passed, 0 failed.
- `loop board` on live data rendered all five labs and exited 0.
- **The live `loop queue --pr 2931 --dry-run`, run on this PR, refused it**, exit 1:
  - while the PR was a draft, at `pr-state`;
  - once it was ready, `pr-state`, `head` (the fetched commit equals GitHub's), `order`, `commit-order`, `report`, `owner-paths` and `rule-sources` passed;
  - `scope`, `review-record`, `required-checks` and `after-approval` failed, as expected before review and before scopes land. `read-path` also failed; see above.
- Claim-wording passes. `xtask links` finds no broken link in the changed files. The copyright check finds 0 matching 8-word runs. `loop pr-check --local` fails only `scope`, as for every loop PR.

## Skills used

solution-integrity: `loop queue` calls `pr-check`'s rules instead of a second copy of them, so the queue and the policy check cannot drift apart.

## Still open

- **`loop close --session` writes only `References #<issue>`.** It should write `Closes` when the diff sets the item to `done` (claude-19 REPORT). Until then, the REPORT carries a closes line.
- **The board's hook canary** comes with loop PR 9.
- **The board's claim counts** include stale committed claims; `docs/harness/claims.json` needs cleaning, or the archive the loop design plans for it.
- **`loop queue` becomes usable** once Kimi's scope PR gives every loop item a `scope:` line.
- **Low:** the two refusal paths for a later CHANGES are not pinned one at a time.
- **Low:** only the PR's issue comments are read, so a hold left as a review body or an inline review comment is not seen. The harness README says so.
- **Nit:** the head is fetched into `FETCH_HEAD`, which a concurrent fetch in the same worktree can overwrite. That fails closed, because a different commit refuses.

## Plain summary

The labs now have a command that merges a pull request only when every rule is met: the checks passed, a valid approval stands, and nobody objected after it. They also have a single screen showing what each lab is doing. For now the merge command turns everything away, because one piece of bookkeeping is still missing from the roadmap items.
