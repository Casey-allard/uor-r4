# ORDER: written-state-count-model, claude-440 (decision card: P1's S5 copy path, A1 or C2)

Lab: claude. Item: [written-state-count-model](../../../../roadmap/items/written-state-count-model.md)
(#4063). Base: `origin/main` `67d54aa0a` (codex-28, #4123). Doc-only.

## Why this unit

claude-439 (PR 4107, merged as `3f0aaa294`) fitted the S5 Pick expert on reserved TRAIN and scored it
frozen on DEV-tune at 100% of C [claude]. Its REPORT leaves one choice open and names it a decision
card, not that unit's call [claude]:

> If P1 deploys one copy path, the choice is between C2's cost and A1's extra 4,318 to 5,232
> micro-BPB. That choice is a decision card, not this unit's.

- **A1** is Pick over the D11 trunk codes (Identity transport): 1,360,806 micro-BPB on DEV-tune, read
  on the device in about 9 min per pass at 100% [claude].
- **C2** is exact token pairs only: 1,365,580 micro-BPB, read on the host in about 1,300 ms [claude].
- The paired band A1 − C2 is [−5,232, −4,318]. A1 − C3, against a permuted token → code map, is
  [−232, −24] [claude].

The choice is now on the critical path. codex-28 (#4123) binds the reserved-TRAIN S5 fit for P1(b)'s
deployable count model and keeps all of its weights, and the next prediction unit packages that
mixture into an active D11 (codex-27's REPORT) [codex, read by claude]. That packer needs to know
which S5 column it carries [claude].

Under CHARTER §5.1 and D68, a scope choice like this is a decision card. The lab's top session
approves it after its own whole-project decision review, it does not wait on the owner, and it is
logged for Casey, who can reverse it [claude]. Nothing in §3 or the caps changes [claude].

## What this unit does

1. Writes `DECISION.md`: the choice, the evidence, the alternatives turned down, a reversal rule and
   what the card does not decide (the geometry label stays with claude-439's reading and P2's bar).
2. Runs a fresh-context decision review that reads the whole track first: PLAN.md's P1 and P2 rows,
   claude-409's card, codex-14's DECISION, claude-439's REPORT and packets, and codex-26 to codex-28's
   REPORTs. The review is saved as `DECISION-REVIEW.md`.
3. Adds one sentence to PLAN.md's P1 row naming the S5 copy path and linking the card.
4. Writes `REPORT.md`.

It writes no code, runs nothing on a GPU and rents nothing, so §4.5 asks for no sheet [claude].

## Files in scope

- `docs/loop/tasks/written-state-count-model/claude-440/**` (ORDER, DECISION, DECISION-REVIEW,
  REPORT);
- `PLAN.md`: one sentence in the P1 row under "Approved sequences".

## Stop list

- No code, no build beyond the harness, no GPU, no pod, no wick job.
- `CHARTER.md`, `AGENTS.md` and `docs/integration/DECISIONS.md` (an owner path) are not edited.
- No new number is measured. The card cites claude-439's packets; it does not re-read DEV-tune and
  never reads DEV-test.
- codex's units and files are not edited. If the review finds that the card conflicts with codex's
  work, the card says so and the conflict goes to #4063, not into codex's folders.

## Done when

1. ORDER is the first commit on the branch and REPORT the last.
2. `DECISION.md` and `DECISION-REVIEW.md` are in, and the review's verdict is APPROVE, or APPROVE
   WITH CHANGES with each change made.
3. Every number in the card matches claude-439's REPORT or its `run2/` JSON.
4. `UOR_LAB=claude uor-harness check` shows no new failure against `origin/main`.
5. `UOR_LAB=claude uor-harness loop pr-check --local` passes, or its output is reported.
