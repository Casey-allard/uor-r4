# ORDER: infra-loop-harness, claude-9 (three D59 wording leftovers)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `72071ac75`.

## Goal

The owner wrote in Claude's chat on 21 Oct 2026, about 06:35Z: "Yes: fix the three D59 wording leftovers (loop page line 45, Claude-first parenthetical in step 12 and PR 7c, AGENTS.md item 6) in one small PR through the same delivery path."

`AGENTS.md` is listed in `docs/harness/owner-paths.json`. That message, quoted verbatim above, is the owner's approval for the one-line edit to it.

The three leftovers come from the review of #2902:
1. **Loop page line 45** states the steady-state review rule with no setup exception.
2. **The "(another vendor's, or Claude's …)" parenthetical** can be misread as letting another vendor approve a setup PR. It appears in step 12, in PR 7c's done-when, and in the same pattern in the supervisor queue rule that restates step 12.
3. **`AGENTS.md` item 6**, the onboarding summary of delivery, names only "another vendor's lab".

## Cancel probe (run first)

Are these already fixed?

Result at `72071ac75`:
- line 45 of `docs/loop/README.md` names no D59;
- `grep -n "another vendor's, or Claude's"` finds the parenthetical at `docs/loop/README.md` lines 65 and 132 and at `docs/roadmap/items/infra-loop-pr07c-queue-board.md` line 22;
- `AGENTS.md` item 6 names no D59.

The task stands. Class: instrument.

## Files in scope

- `docs/loop/README.md`: line 45, step 12 (line 65), and the supervisor queue rule (line 132).
- `docs/roadmap/items/infra-loop-pr07c-queue-board.md`: the done-when parenthetical.
- `AGENTS.md`: item 6 only, as one line, with the file's length unchanged.
- `docs/loop/tasks/infra-loop-harness/claude-9/**`.

## Done when

- Line 45 and `AGENTS.md` item 6 name the D59 setup exception.
- All three parentheticals put Claude first, with "otherwise another vendor's" after.
- PR 7c's planted fault is untouched, so it still matches its row word for word.
- D46, D55 and D59 are unchanged.
- The claim-wording, links, roadmap and local copyright checks pass.

## Stop and ask if

- Any file outside scope needs to change.
- The wording would change a rule rather than restate D59.
