# #1157 — Extend geometric selection to literals and record preservation failures

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-06T22:59:22Z
- **Updated:** 2026-10-09T13:17:25Z
- **Closed:** 2026-09-06T23:26:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1157
- **Merged:** 2026-09-06T23:26:37Z
- **Merge commit:** a52575dd82e7e5b901d89f98a449c7bc6617ece2
- **Base / Head:** main / UOR-Foundation:codex/geometric-literal-selection
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Literal first answers remained on the older sparse selector, blocking complete computations. This adds an artifact-bound opt-in that reuses the existing learned H4 role component for literal Copy/Add/NoOperation selection, plus native training frames that cannot supply a preceding answer. It also reports the exact cases behind feature collisions and raises the bounded vocabulary ceiling to768 after observing a256-of601 truncation collision.

Actual execution improves new complete three-turn cases from8/16 to16/16 and new literal answers from7/16 to12/16 for both angular and matched exact-code fits. **Neither candidate is promoted.** Angular loses two previously correct computed-result cases and three identifier-copy responses; exact-code also loses a previously correct computed-result case. Four new abstention texts fail. `af337c28` remains accepted and old artifacts retain their behavior with the new flag absent. The record preserves failed capped fits and the missing-prefix correction, with full process costs and case-level comparisons.

The next implementation protects working computed-result parameters while learning bounded literal numeric admission against word-answer alternatives. This follows the measured interference rather than increasing general capacity. Serving adds no matmul, floating projection, dense transformer or LLM/provider correction.

Validation:
- Rebuilt the native probe and real `r4` CLI; reversed fenn/kira literals with a301 distractor generate `7.\n`.
- Seven focused typed tests and the kernel source check pass.
- Actual three-turn serving: zero allocations/bytes, checkpoint roundtrip and artifact mutation rejection pass.
- Generated preservation:48/48 dependent,28/28 exposed-name,28/28 long-context,5/5 persistent,6/6 prior numeric and three12-case prior role sets pass. Earlier62/62 and24/24 sets regress to60/62 and23/24; reported as failures.
- Formatting, native architecture policy, claim wording and diff checks pass. Broad dormant release QA and new general Rust reasoning are NOT_RUN.

317.230s local model work;663.529s monitored engineering. Cumulative model use2921.586/2970s. No additional storage allowance, deletions or external model compute. Source identities match after rebasing onto merged #1156. Required hosted checks are pending; historical compatibility acknowledgements are not model-quality evidence.

Advances #1139 and #1140 without closing their unmet capability criteria. See `docs/native_geometric_literal_selection_1139.md` and its bound evidence.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

