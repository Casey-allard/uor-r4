# ORDER: infra-loop-harness, claude-11 (amend D52: Claude builds the loop setup while it runs alone)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `e61746628`.

## Goal

The owner wrote in Claude's chat on 21 Oct 2026, about 08:10Z: "I think it'll only be claude running at first, so what is next?"

Claude answered that D52 gives most setup PRs to labs that will not be running: PRs 2–7 and 10–12 to Codex and DeepSeek, and PR 9 to Antigravity. Claude offered to record an amendment. The owner chose this answer, worded by Claude: "Yes: record a D52 amendment that Claude builds every loop setup PR while it runs alone (other labs' adapters added when they join), then start loop PR 2 (#2905) and PR 4 (#2907) through the delivery path."

This PR records that amendment wherever the team split is stated. Starting PRs 2 and 4 is separate work, with its own ORDERs.

## Cancel probe (run first)

Is the amendment already recorded?

Result at `e61746628`: `git grep -n "runs alone"` finds nothing in `docs/integration/DECISIONS.md`, `docs/rules/changes.md`, `docs/loop/README.md` or `docs/roadmap/items/`. Twelve items still name another lab as builder in their team step: 10 say "codex or deepseek", PR 6 says the same in a variant wording, and PR 9 says "antigravity". The task stands. Class: instrument.

## Files in scope

- `docs/integration/DECISIONS.md`: row D52. Its original text stays; the amendment is appended.
- `docs/rules/changes.md`: one new entry.
- `docs/loop/README.md`: the build plan's Teams line.
- `docs/roadmap/items/infra-loop-harness.md`: the team-split step.
- The 12 loop items whose team step names another lab (PR 2–7, 7a–7c and 9–12): that step only.
- `docs/roadmap/items/infra-loop-pr09-client-adapters.md`: one sentence in its done-when. The Claude Code adapter comes first, and each other client's adapter is added when that lab joins.
- `docs/loop/tasks/infra-loop-harness/claude-11/**`.

The charter does not state the team split, so it is not in scope.

## Done when

- Every place that states the team split says the same thing: while Claude runs alone, Claude builds every loop setup PR, and each other lab's client adapter is added when that lab joins.
- D52's other parts, D59 and D46 are unchanged.
- Both of the owner's messages are quoted verbatim.
- `roadmap check`, `render --check`, claim-wording, links and the local copyright check pass.

## Stop and ask if

- Any file outside scope needs to change.
- The wording would change D52 beyond who builds the setup, or would change D59.
