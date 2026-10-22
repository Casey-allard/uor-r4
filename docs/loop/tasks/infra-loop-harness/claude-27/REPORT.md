# REPORT: infra-loop-harness, claude-27 (the follow-ups from the claude-25 and claude-26 reviews)

Lab: claude. Base: `d00431587`. Order: [ORDER.md](ORDER.md), committed first (`2111c915f`). The cancel probe found that `kimi-1/REPORT.md` still matched the kit, and that `loopbench.rs`'s no-ORDER arm still used the raw review.

## What did not work, first

- **The copied sentence sat on `main` for a day.** It came in with #2929, and Antigravity's `loop copyright` found it. It could not be fixed in claude-26 as Casey first asked, because `pr-check` reads scope from the base commit. claude-26 put the path in scope, and this unit uses it and then removes it.
- **I wrote the inbox item's first done-when (claude-26), and it lost messages to `all`.** It had one status per message, so the first lab to mark an `all` message `read` hid it from the other four. The fresh review of #2943 found this. Casey decided "Per-lab lines (Recommended)", and the item now follows that.
- **The same item named a path that is wrong for every lab.** `local/` belongs to each checkout, so `local/bench/INBOX.md` inside a lab's worktree does not exist. It now names the main checkout.
- **The bench showed approvals the gate cannot count, on two edge paths left after #2942's fix round:** a PR with no ORDER, and a PR whose ORDER cannot be read.

## What changed

| Commit | What |
|---|---|
| `9b2f52414` | `kimi-1/REPORT.md:37` is reworded in our own words, and only that line changed. The umbrella scope drops `kimi-1/REPORT.md`. `infra-bench-inbox-rule`'s why and done-when name the main checkout's path and per-lab lines for `all`. |
| `e2a82f1f4` | `loopbench.rs`: the no-ORDER and unreadable-ORDER arms use `unordered_review_state`. It reads `draft`, else `review not counted (<lab>)` for a record at the head (`uncounted_or_needs`, shared with `gated_review_state`), else `needs review`. The raw `review_state` is removed, since nothing calls it any more. Two tests. |

## Measurements

None. This is tooling and records, with no model run.

## Checks run at the head

- `kimi-1/REPORT.md` as a whole has 0 eight-word runs matching LoopKit and 0 matching CorvusWing. This PR's added text has 0 and 0.
- `roadmap check` reports 88 items OK, and the render is unchanged.
- fmt and whole-crate clippy with `-D warnings` are clean. `loop_bench` passes 12.
- **Negative gates.** Each patch applied with no error, failed its test, and passed again once restored:
  - L1: with the no-ORDER arm on the raw review, `a_pr_without_an_order_never_shows_an_uncounted_approve` fails at line 528.
  - L2: with the unreadable arm on the raw review, `an_unreadable_row_sorts_with_needs_review` fails at line 552.
- Full harness suite at `e2a82f1f4`: 1,016 passed, 0 failed.

## Skills used

scope-guard: in Kimi's REPORT only the one sentence changed. Archiving the old text byte for byte (CHARTER §4.7) would have kept the copied run in the tree, so git history holds the original instead.

## Still open

- DeepSeek's #2943 adds the `AGENTS.md` line, written from Casey's per-lab decision and the main-checkout path.
- Kimi checks claude-26's and this unit's edits to Kimi's items and REPORT on return.

## Plain summary

This removes the last copied sentence from Ron's kit, fixes the inbox task so a message to every lab really reaches every lab, and stops the bench from showing an approval that would not count.
