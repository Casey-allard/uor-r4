# #2042 — Record the binding read-out results in current state, with scope and limits (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T15:02:22Z
- **Updated:** 2026-10-09T15:05:03Z
- **Closed:** 2026-10-09T15:04:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2042
- **Merged:** 2026-10-09T15:04:15Z
- **Merge commit:** 1505e7ae393903bdf331d5ead5b5922adf80ffd4
- **Base / Head:** main / UOR-Foundation:deepseek/state-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

current-state.md records the serving result (0.93277 BPB, 4.70x) but not the binding line that followed it. This adds it: the no-pointer control at 0/84 against the pointer at 35/84; a serving-time gate floor recovering exact recall of never-seen values on every arm with zero cells lost (49/84, 59/84 = 0.7024, 52/84) with span and read mass bit-identical across the floor, so the floor moves only the mixing weight; termination as the residual failure (value present in 79/84 at F=1.0 but 6/84 exact); the run-length stop at 65/84 = 0.7738 which fires on the prompt template (span inside the value's own span on only 20 of 79); and the token-identity stop at a NEW BEST 72/84 = 0.8571 [0.7667, 0.9163] that also recovers F=1.0 from 6 to 71. Significance is stated honestly: against 65/84 the Newcombe interval CONTAINS ZERO (p=0.0654), so it is a better point estimate and not significant on 84 cells, while it is significant against the 59/84 plateau (p=0.00235). Limits recorded: the mechanism is 'end the reply at the end of the first span the pointer sustains by token identity', not 'recognise the value'; extra fine-tuning alone costs about 13 points, so extend-the-run is retracted for this axis; and the whole line is OFFLINE ONLY because export refuses a pointer with an identity term - no integer port - so it must never be reported as served. Also records the defect this work exposed: #2009 landed half the copy-stop change and main could not compile its lib tests until #2040. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

