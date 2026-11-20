# REPORT: written-state-count-model, claude-442 (decision card: the per-distance Pick split in S5)

Lab: claude. Order: [ORDER.md](ORDER.md). Card: [DECISION.md](DECISION.md). Review:
[DECISION-REVIEW.md](DECISION-REVIEW.md). Doc-only; nothing ran on a GPU and nothing was rented.

**Decision: S5 takes the per-distance Pick split at its next refit, bundled with it, and the split
never delays P1(b).** The split beats the deployed three-class rule by 16 to 32 micro-BPB on DEV-tune
at 100% of C, about 2 parts in 100,000. It costs nothing at run time and about 39 KB of weights, so it
rides a refit that happens anyway. No unit is spent on it alone.

## What did not work, first

- **The gain is too small for any bar to see** [claude]. 16 to 32 micro-BPB is far below P1(b)'s 3%
  margin over R\*. The card adopts the split because it is nearly free, not because it moves P1.
- **The card credits no geometry** [claude]. A permuted code map gains nearly as much from the same
  split (C3s − C3 [−27, −10]), and the codes' edge over that map is unchanged by the split.
- **Adopting is not free in the format** [claude]. `UORS5W01` fixes three classes and carries no class
  count, and codex-31's CUDA fit port is sized 528×14. The refit unit has to version the format and
  run the fit at 880×14; that is why the card bundles the split rather than giving it a unit.
- **Each reviewer ran out of context before its slice was done** [claude]. The first reviewer
  verified the bands and the arithmetic only. The rest was split into two narrower reviews, and the
  top session closed the remaining gaps itself (DECISION-REVIEW, "Gaps closed").
- **The four-class saving rests on an unchecked claim** [claude]: that no two D11 codes are 1 bit
  apart. The card allows that rule only after the code set is read.

## What changed

- `docs/loop/tasks/written-state-count-model/claude-442/`: ORDER, DECISION (the card),
  DECISION-REVIEW (three reviews and their disposition), and this REPORT.
- `PLAN.md`: one sentence in P1's row under "Approved sequences" gives S5's bucket rule and links the
  card.

## Measurements

None new. Every number is claude-441's (`claude-441/run1/pick-p1000-retry.json`, checked with jq) or
read from the code at `47c0c82c8`:

| Arm | DEV-tune all | DEV-tune novel |
|---|---|---|
| A1, three classes | 1,360,806 | 1,651,188 |
| A1s, split | 1,360,781 | 1,651,154 |
| C3, permuted, three classes | 1,360,932 | 1,651,364 |
| C3s, permuted, split | 1,360,913 | 1,651,338 |

Bands: A1s − A1 [−32, −16] (novel [−44, −23]); C3s − C3 [−27, −10]; A1s − C3s [−241, −28].
S5's weight file: 7,392 words (59,180 bytes) at three classes, 12,320 words split, 39,424 bytes more.

## What the numbers say

- The split is a real gain whose band clears 0, and it is tiny.
- Its run-time cost is nil: the serving read already returns the pair distance, and only the class
  mapping and bucket index change.
- Its artifact cost is 39,424 bytes, about 17 parts in a million of codex-33's D11 and tokenizer.

## Checks run at the head

- `UOR_LAB=claude uor-harness check`: one error, `status.protected_branches_present` ("protected branch
  \"gh-pages-preview\" is missing from origin"). It is about a remote branch, not this diff, so it is
  not a new failure against `origin/main`. The rest are warnings and info lines (orphan local branches,
  the disk floor, the budget note).
- `UOR_LAB=claude uor-harness loop pr-check --local`: run at this head; its result is posted on the PR.
- The decision review: round 1 APPROVE WITH CHANGES (partial), round 2 APPROVE WITH CHANGES and
  APPROVE; the eight minor findings are disposed of in DECISION-REVIEW.
- The card's numbers were checked against claude-441's JSON by jq, its code claims against the files
  at `47c0c82c8`, and its codex quotes against codex-31's and codex-33's REPORTs.

## Skills used

None beyond the loop's own steps: ORDER, a decision card under §5.1 / D68, a fresh-context decision
review, and REPORT.

## Still open

- **The refit unit's four items** (DECISION, "What the refit unit carries"): one shared class
  function, both arms fitted and banded, the class in the ≥ 64-prompt replay with a planted negative,
  and the three-class format still loading. A note goes to #4063.
- **Whether the D11 code set has no two codes 1 bit apart**, which would allow the four-class rule.
- **A finer split of the 3-or-more class** is untested.

## Plain summary

The previous unit found that the counting model's copy helper is right about twice as often when its
near copy is two steps off as when it is further off. Using that improves predictions by about 2 parts in
100,000, which is real but too small to matter on its own. It costs nothing when the model runs and
about 39 KB of file. So we will switch it on the next time the copy helper is refitted anyway, and we
will not spend a separate piece of work on it, or let it hold up the model.
