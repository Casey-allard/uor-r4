# ORDER: infra-loop-harness, claude-8 (carry D59 into the remaining loop items and steps)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `500c61c2c`.

## Goal

The owner asked on 21 Oct 2026 for the D59 review note to go into:
- the other 12 loop items;
- the parent item;
- the loop page's steps 10 and 12;
- PR 7c's row;
- the three low findings from #2901's delta check.

Today, every one of these still tells a builder that the exact-head review comes from another vendor. PR 8's item names Kimi as the reviewer. Under D59 (CHARTER §5.5, amended in #2900), every loop setup PR gets its exact-head review from Claude's fresh-context reviewers, whichever lab built it, until the other labs run the loop. No other lab is asked.

## Cancel probe (run first)

Do these files already carry D59?

Result at `500c61c2c`:
- `grep -l "D59" docs/roadmap/items/infra-loop-pr0[2-5]*.md docs/roadmap/items/infra-loop-pr0[7-9]*.md docs/roadmap/items/infra-loop-pr1*.md docs/roadmap/items/infra-loop-harness.md` finds none of the 13 files.
- The loop page's steps 10 and 12 and PR 7c's row name no D59.

The task stands. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-pr02-item-keys.md`, `-pr03-render.md`, `-pr04-gates.md`, `-pr05-policy-check.md`, `-pr07a-claim.md`, `-pr07b-session.md`, `-pr07c-queue-board.md`, `-pr08-migration.md`, `-pr09-client-adapters.md`, `-pr10-supervisor.md`, `-pr11-progress-window.md`, `-pr12-git-hooks.md`. In each: the team step, the review step (which also gains the `Setup row:` line from #2901) and the delivery rule. For PR 7c, also its done-when and planted fault.
- `docs/roadmap/items/infra-loop-harness.md`: the team-split step and the delivery rule. The scope was widened before the edit, after the sweep's own check found two more sentences: the review sentences in "Why it matters" and "What is missing". The second one describes today's practice as "a review from another vendor".
- `docs/roadmap/items/infra-loop-pr06-review-records.md`: delta-check low 1. Until the owner lists the switch's path, the gate treats D59 as in force.
- `docs/loop/README.md`: steps 10 and 12 in the steps table, and PR 7c's row in the build-plan table. The scope was widened before the edit, after the sweep's own check found one more line: the supervisor's queue rule in the advisory-policy paragraph, which restates step 12.
- `docs/loop/tasks/infra-loop-harness/claude-8/**`. The REPORT records delta-check lows 2 and 3 as corrections to claude-7's REPORT. claude-7's REPORT itself is not edited: the loop page's records table says ORDER and REPORT are "never changed after merge".

## Done when

- None of the 13 items, steps 10 and 12, or PR 7c's row tells a builder to get a setup PR's review from another vendor or from Kimi.
- PR 7c's planted fault is word for word the same in its item and its row.
- D46, D55 and D59 are unchanged.
- `roadmap check`, `roadmap render --check`, claim-wording, links and the local copyright check pass.

## Stop and ask if

- Any file outside scope needs to change.
- The wording would change D46, D55, D59 or D52's team assignments rather than carry D59 into them.
