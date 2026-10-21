# ORDER: infra-loop-harness, claude-7 (PR 6's review gate follows D59 for the setup PRs)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `80938b12e`.

## Goal

The owner asked for this on 21 Oct 2026: add a D59 note to the PR 6 roadmap item so that its review gate exempts the loop setup PRs.

As planned today, PR 6's gate requires an APPROVE from a lab of a different vendor than the author, and its planted fault refuses a same-vendor record. Under D59, as amended in #2900, every loop setup PR gets its exact-head review from Claude's fresh-context reviewers, whichever lab built it, until the other labs run the loop. A Claude-built setup PR with a Claude review would therefore be refused by the gate, and a Kimi review of a Codex-built setup PR would be accepted. Both outcomes contradict D59.

## Cancel probe (run first)

Does the PR 6 item or the loop page already make the gate follow D59?

Result at `80938b12e`: `grep -n "D59" docs/roadmap/items/infra-loop-pr06-review-records.md` finds nothing. On the loop page, D59 appears only in the build-plan note, not in "The review record" paragraph or in PR 6's row. The task stands. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-pr06-review-records.md`: the done-when, the planted fault, the review steps under "How to start", the delivery rule and the sources.
- `docs/loop/README.md`: one D59 sentence in "The review record" paragraph, and PR 6's planted fault in the build-plan table. These are the spec the item points the builder to, so they must say the same thing.
- `docs/loop/tasks/infra-loop-harness/claude-7/**`.

## Done when

- The item, the review-record paragraph and the table row agree on two points:
  - Until the other labs run the loop, the gate accepts a loop setup PR only with an APPROVE from Claude's fresh-context reviewers, whichever lab built it.
  - Outside the setup, D46's cross-vendor rule is unchanged.
- The planted fault covers the D59 case.
- `roadmap check` and `roadmap render --check` pass, along with the claim-wording, links and local copyright checks.

## Stop and ask if

- Any file outside scope needs to change.
- The wording would change D46, D55 or D59 rather than carry them into PR 6's spec.
