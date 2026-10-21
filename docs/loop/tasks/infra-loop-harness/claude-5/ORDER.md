# ORDER: infra-loop-harness, claude-5 (record D59: no lab review of the loop setup)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `54df5db35`. The ORDER was written at `fc96b4bea` and rebased onto loop PR 1 (#2898) before the first push.

## Goal

Record the owner's direction of 21 Oct 2026 the way charter §5.1 requires: verbatim, and the same day. Until the other labs are running the loop, the other labs do not review the loop's setup PRs; fresh-context reviewers of the authoring lab do.

## Cancel probe (run first)

Does the charter or the decision index already record this exception?

Result at `fc96b4bea`: `grep -n "D59\|onboarded" CHARTER.md docs/rules/changes.md docs/integration/DECISIONS.md` finds no D59 and no such exception. The task stands. Class: instrument. Re-run at `54df5db35` after the rebase: still no D59 and no such exception in any of the three files.

## Files in scope

- `CHARTER.md`: one sentence in §5.5, and a D59 note on the retired D46 entry in §6 (scope widened before the edit, after the review found that §5.5 and §6 read against each other without it).
- `docs/loop/README.md`: one D59 note in the build plan (scope widened before the edit, after the review found that the plan's review lines would otherwise send setup PRs to other labs).
- `docs/rules/changes.md`: one new entry.
- `docs/integration/DECISIONS.md`: row D59.
- `docs/loop/tasks/infra-loop-harness/claude-5/**`.
- `AGENTS.md`: only the decision-index range label, D0–D58 to D0–D59 (scope widened before the edit, because a new row makes the label stale).

## Done when

The sentence, the entry and the row are in. The owner's words are quoted verbatim. The claim-wording, links, roadmap and local copyright checks pass.

## Stop and ask if

- Any file outside scope needs to change.
- The wording would change any rule beyond this exception.
