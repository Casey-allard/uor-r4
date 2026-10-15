# #1951 — Terminal-weight bracket complete: 60x best, compliance band narrow (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T18:53:14Z
- **Updated:** 2026-10-09T13:22:41Z
- **Closed:** 2026-10-08T18:54:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1951
- **Merged:** 2026-10-08T18:54:52Z
- **Merge commit:** ce965ab0f1bc66117b54ec7a8480173037f10939
- **Base / Head:** main / UOR-Foundation:deepseek/terminal-bracket
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The terminal-weight bracket is complete: 60x is the best dose and the compliance band is narrow

Four doses, same shape, corpus, mask, policy, steps and seed; only UOR_TERMINAL_WEIGHT differs.
At 20x yes/no complies 2 of 3 and one retention turn answers. At 60x yes/no complies and BOTH
retention turns answer. At 100x yes/no fails to list artifacts while retention still holds. At
200x compliance reverts, retention breaks and artifacts return.

So the ordering is 60 > 20 > 100 > 200, and the yes/no compliance band is NARROW - present at 20
and 60, gone by 100 - while retention and follow-up survive from 20 through 100 and break only at
200. The two behaviours respond at different thresholds, which is why a window exists in which
both hold and why 60x sits in it.

That completes the arc the eliminations opened. The change implied by token-level MLE length bias
- weighting the response's terminating <|eos|>, identified by the mask proven byte-identical to
the builder's hash - was written, tested, merged, and has now been shown to move compliance up,
down, and to a best point in between. It is the only intervention tonight that moved instruction
compliance at all, and no data, scale or decoding change came close.

The metric lesson is fully documented: weighted development loss rises monotonically with the
dose while behaviour peaks in the middle, so any sweep selected on the loss would have chosen the
collapse.

What remains: one-word and numeric turns comply at no dose, producing procedures, phrases and
list openers rather than the requested token. The narrowness of the yes/no band suggests those
classes need something further rather than a different weight - a much smaller question than the
one this window opened with.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

