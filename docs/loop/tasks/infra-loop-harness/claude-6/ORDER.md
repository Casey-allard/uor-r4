# ORDER: infra-loop-harness, claude-6 (amend D59: Claude reviews every loop setup PR)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `4611538bb`.

## Goal

Record the owner's amendment to D59 of 21 Oct 2026, verbatim and the same day, as charter §5.1 requires. D59 as merged in #2899 sends each setup PR to fresh-context reviewers of the lab that built it. Setup PRs built by Codex, DeepSeek or Antigravity (PRs 2–7, 9 and 10–12 under D52) would then be reviewed only by their own lab. The owner chose instead that those PRs get fresh-context Claude reviews, because Claude leads the architecture. No lab that is not running the loop is asked to review the setup.

## Cancel probe (run first)

Does the charter, the decision index or the loop page already give Claude the review of setup PRs that other labs build?

Result at `4611538bb`: `grep -n "Claude's fresh-context\|whichever lab built" CHARTER.md docs/rules/changes.md docs/integration/DECISIONS.md docs/loop/README.md` finds nothing. The §5.5 sentence still says "fresh-context reviewers of the authoring lab". The task stands. Class: instrument.

## Files in scope

- `CHARTER.md`: the D59 sentence in §5.5, and the D59 clause on §6's retired D46 entry. The scope was widened before that edit, after the review found the clause stale: since the amendment, same-lab review covers only the setup PRs Claude builds.
- `docs/rules/changes.md`: one new entry.
- `docs/integration/DECISIONS.md`: row D59 keeps its text, with the amendment appended.
- `docs/loop/README.md`: the D59 note in the build plan.
- `docs/loop/tasks/infra-loop-harness/claude-6/**`.

## Done when

All four places say the same thing: whichever lab built a loop setup PR, its exact-head review comes from Claude's fresh-context reviewers until the other labs run the loop. The owner's words are quoted verbatim. The claim-wording, links, roadmap and local copyright checks pass.

## Stop and ask if

- Any file outside scope needs to change.
- The wording would change any rule beyond this amendment, for example D46 outside the setup, or D59's end condition.
