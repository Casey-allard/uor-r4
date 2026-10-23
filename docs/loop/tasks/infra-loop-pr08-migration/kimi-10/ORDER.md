# ORDER: infra-loop-pr08-migration, kimi-10 (repair: the §4.8 reconciliation is an owner call)

Lab: kimi. Item: [infra-loop-pr08-migration](../../../../roadmap/items/infra-loop-pr08-migration.md) (#2913).
Setup row: 8. Base: `fb6df367b`.
Parent: [kimi-9/ORDER.md](../kimi-9/ORDER.md).

## Owner approval

**"ok well I approve of #1"** — Casey in the kimi chat, received 2026-10-22T16:49Z, assigning PR 8 to kimi and approving this PR's changes to the owner paths `AGENTS.md` and `docs/integration/DECISIONS.md` (owner-paths.json). Carried unchanged from kimi-9 (the chain rule keeps Lab/Item/Setup row and this section identical).

## Goal

Repair after the self-found defect in kimi-9's done-when 7: reconciling M1's registry
headline to CHARTER §4 row 8 is NOT mechanical. The registry's M1 headline is the reply bar
(37→116/232, blocked under R14), while §4.8's closing bar is the D11-export top-1 table
contrast (1,226 / 6,300 of 14,287, two seeds, tokens/s). Aligning them picks which metric
heads M1 — a bar decision, the owner's. The edit also rippled into `docs/labs/session-goal.md`
(out of scope) and three repo-content tests pinned on the current headline
(`briefingrules::the_real_repo_agrees_with_its_own_prompts`,
`prgates::registry_gate_tests::the_headline_gate_fires_only_on_a_self_contradicting_body`,
`prgates::check_pr_tests::every_ported_gate_and_the_dispatcher_agree_with_python`), which fail
on the edited registry and pass on the reverted one (verified at this branch).

Finished steps, in the past tense: the moves, pointers and catalog entries (`7648d3287`); the
authored content and renders (`f535563bc`, `4c7970ea7`, `3128086f4`); the claims cleanup
(`3db90ce8d`); the verifier, tests and gate (`3a4e8f4b5`, `a499b24d1`); the ORDER's
owner-approval section (`ea..`); the renderer's link re-rooting fix and re-render (`853b9756b`);
the lesson keying with the validator extension (`16c6b5647`); the FILEMAP/AGENTS budget fixes
(`d331d28cc`).

## This repair

1. REVERT the M1 headline edit in `docs/harness/milestones.json` (the registry keeps the reply
   headline, consistent with session-goal.md and the three pinned tests).
2. CARD the mismatch to the owner on #2028 and the kimi chat: which metric heads M1 — the
   §4.8 closing bar or the R14-blocked reply bar — with the ripple named.
3. Done-when 7 of kimi-9 is amended to read: "milestones.json bars **verified against** CHARTER
   §4.8 and any mismatch carded to the owner" — the match itself is his call.

## Files in scope

As kimi-9, narrowed to: `docs/harness/milestones.json` (the revert) and this task folder.

## Done when

- The registry equals the base's content for M1; the three repo-content tests pass at the head.
- The owner card is posted and linked here.
- The gates stay GREEN and the receipts re-bind at the final head.

## Stop and ask if

- The owner answers in the kimi chat before this PR merges — his answer then rides as a
  follow-up, never folded into this branch mid-review.
