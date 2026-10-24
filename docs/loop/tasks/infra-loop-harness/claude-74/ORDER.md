# ORDER: infra-loop-harness, claude-74 (phase 0: record the single-lab rule)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: A. Base: `3ac553790`, merged forward onto claude-73 (#3159), which scopes `CHARTER.md` and `docs/rules/changes.md` here.

## Goal

From 23 Oct 14:10Z to 24 Oct 13:43Z the labs merged 70 PRs and closed no issue. Then every lab stopped behind posted asks nobody answered. This unit records the owner's 24 Oct direction as the rules, with no tooling change:

- the single-lab rule;
- review inside the unit, with audits after the merge;
- the pre-run review in place of the owner release and the holds;
- the decision review in place of cards to the PM or owner;
- the owner's queue as a log;
- the finishing PR closes its issue;
- a supervisor that restarts idle labs.

His words are quoted verbatim in the change-log entry this unit adds.

## Cancel probe (run first)

At `3ac553790`, does the charter let one lab finish a run, a decision or a code PR alone? No:
- §5.1 holds Training, M2 and Stage B for an owner release and says a pending owner question means wait;
- §5.2 sends decision cards to the PM or owner;
- §5.5 needs another vendor's review (D46).

The unit stands. Class: instrument.

## Files in scope

- `CHARTER.md` (§5.1, §5.2, §5.5, §5.10, §6; not §3, §4 or the caps)
- `AGENTS.md` (the lines restating those clauses; at most 80 lines)
- `docs/integration/DECISIONS.md` (D65–D71 appended; D62–D64 left to #3149)
- `docs/rules/changes.md` (one dated entry)
- `docs/loop/tasks/infra-loop-harness/claude-74/**`

## Done when

- The charter states all seven, quoting the owner, with §3, §4 and the caps unchanged.
- D65–D71 each have a reason, alternatives and cost to undo.
- `AGENTS.md` agrees with the charter, at most 80 lines.
- The change log has a dated entry: who decided, his words, side effects.
- `loop report --check`, `loop pr-check --local`, `git diff --check`, the quick gates and the roadmap check pass.
- The exact-head review is Claude's own fresh-context review. It counts on a doc-only PR at the base, and it is the rule recorded here.

## Stop and ask if

§3, §4 or a cap would change, or a file outside the list is needed.

## Owner approval

Owner paths changed: `CHARTER.md`, `AGENTS.md`, `docs/integration/DECISIONS.md`, `docs/rules/changes.md`.

Casey, in Claude's chat, received by 2026-10-24T21:00:26Z: "Yes, start phase 0: open the records PR with my words verbatim (single-lab rule, review inside the unit, pre-run and decision reviews, post-merge audits, owner queue as a log) and post the #2028 note with each lab's first move. I'll reopen the labs."

The words recorded are his answers in the same chat, about 20:35–20:58Z, quoted verbatim in `docs/rules/changes.md` under "24 October 2026: the single-lab rule".
