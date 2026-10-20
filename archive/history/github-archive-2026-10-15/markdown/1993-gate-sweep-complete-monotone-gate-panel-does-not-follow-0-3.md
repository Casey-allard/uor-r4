# #1993 — Gate sweep complete: monotone gate, panel does not follow, 0.3 preserved (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T03:33:56Z
- **Updated:** 2026-10-09T13:20:20Z
- **Closed:** 2026-10-09T03:35:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1993
- **Merged:** 2026-10-09T03:35:34Z
- **Merge commit:** 2734f2090bda3fa997cce2d59d8360369de5b03c
- **Base / Head:** main / UOR-Foundation:deepseek/gate-sweep-complete
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Addendum to the closure document, which reported weights 0.0, 0.1 and 1.0 and explicitly excluded the 0.3 arm because it was still training; it has finished, so the exclusion is lifted. Four points: mean gate 0.2644, 0.2959, 0.3559, 0.4164 - monotone in the supervision weight, so the lever does exactly what it is asked across the range. THE PANEL DOES NOT FOLLOW IT: 0.1 is the worst of the four, 0.3 intermediate, 1.0 collapsed with I do on six of fifteen; none matches the ungated arm combination of value-driven out-of-family copying and partial compliance, so this is not a tuning problem with a good setting hiding in the range. 0.3 IS THE MOST INFORMATIVE SUPERVISED POINT and is preserved: the only gate weight where the retention turn is responsive (You have, the pattern of the session earlier working multi-turn answers) while yes/no is also correct - not better than 0.0 on the emitted-value criterion, but it means the interaction is not simply more-supervision-is-worse. The ungated pointer arm remains the best configuration on both axes. Next levers: read-binding supervision, a token-identity pointer key, or a constrained decoder. No probe rows were run on the gate-supervised checkpoints, so the emitted-value criterion is not claimed for them. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

