# ORDER: written-state-count-model, claude-442 (decision card: the per-distance Pick split in S5)

Lab: claude. Item: [written-state-count-model](../../../../roadmap/items/written-state-count-model.md)
(#4063). Base: `origin/main` `47c0c82c8` (claude-441, #4127). Doc-only.

## Why this unit

claude-441 (PR 4127, merged as `47c0c82c8`) split S5's Pick buckets by pair distance (0, 1, 2, 3 or
more) and scored the split frozen on DEV-tune at 100% of C [claude]. Its REPORT leaves one choice
open and names it a decision card [claude]:

> By the sheet's map, the REPORT recommends the split for S5; a decision card under §5.1 makes the
> change.

- **A1s**, the split, scores 1,360,781 micro-BPB on DEV-tune, against A1's 1,360,806. The paired band
  A1s − A1 is [−32, −16] [claude].
- **C3s**, the same split over a permuted token → code map, gains nearly as much: C3s − C3 is
  [−27, −10]. A1s − C3s is [−241, −28], about where A1 − C3 was, [−232, −24] [claude].
- **No DEV-tune or R target fell at distance 1**, and only 36,362 of 6,690,318 fired near reads are at
  distance 2 [claude].

claude-440's card set S5's copy path to A1 and named "per-distance weights" as "a future unit, not a
choice made here" [claude]. codex-33 (#4130) now serves A1's three-class S5 fit from one D11 through
the ordinary client, and its REPORT names the next step: an R-only fit and freeze of the full-C
candidate [codex, read by claude]. That unit needs to know which bucket rule its S5 fit uses [claude].

Under CHARTER §5.1 and D68 this is a decision card. The lab's top session approves it after its own
whole-project decision review, it does not wait on the owner, and it is logged for Casey, who can
reverse it [claude]. Nothing in §3 or the caps changes [claude].

## What this unit does

1. Writes `DECISION.md`: the choice, the evidence, its cost in the deployed format, the alternatives
   turned down, a reversal rule, and what the card does not decide.
2. Runs a fresh-context decision review that reads the whole track first: PLAN.md's P1 and P2 rows,
   claude-409's and claude-440's cards, codex-14's DECISION, claude-441's REPORT and `run1/` JSON,
   codex-33's REPORT and the S5 runtime code (`frozen_pick.rs`, `runtime/rank.rs`,
   `runtime/mixture.rs`, `runtime/code_pick.rs`). The review is saved as `DECISION-REVIEW.md`.
3. Adds one sentence to PLAN.md's P1 row on S5's bucket rule, linking the card.
4. Writes `REPORT.md`.

It writes no code, runs nothing on a GPU and rents nothing, so §4.5 asks for no sheet [claude].

## Files in scope

- `docs/loop/tasks/written-state-count-model/claude-442/**` (ORDER, DECISION, DECISION-REVIEW,
  REPORT);
- `PLAN.md`: one sentence in the P1 row under "Approved sequences".

## Stop list

- No code, no build beyond the harness, no GPU, no pod, no wick job.
- `CHARTER.md`, `AGENTS.md` and `docs/integration/DECISIONS.md` (an owner path) are not edited.
- No new number is measured. The card cites claude-441's packet and reads the code at `47c0c82c8`; it
  does not re-read DEV-tune and never reads DEV-test.
- codex's units and files are not edited. What the card asks of the packing unit goes to #4063 as a
  note, not into codex's folders.

## Done when

1. ORDER is the first commit on the branch and REPORT the last.
2. `DECISION.md` and `DECISION-REVIEW.md` are in, and the review's verdict is APPROVE, or APPROVE
   WITH CHANGES with each change made.
3. Every number in the card matches claude-441's REPORT, its `run1/` JSON or the code at `47c0c82c8`.
4. `UOR_LAB=claude uor-harness check` shows no new failure against `origin/main`.
5. `UOR_LAB=claude uor-harness loop pr-check --local` passes, or its output is reported.
