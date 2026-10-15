# #1256 — Learn content-controlled geometric read and emit scheduling

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T20:28:53Z
- **Updated:** 2026-10-09T13:17:04Z
- **Closed:** 2026-09-13T20:29:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1256
- **Merged:** 2026-09-13T20:29:53Z
- **Merge commit:** bb29d3ae302f412c99925d8bbb27d75bcf4ca01a
- **Base / Head:** main / UOR-Foundation:codex/adaptive-geometric-attention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The geometric reader previously executed a fixed two-read schedule. This adds a learned byte-conditioned read/emit policy around the retained reader, query update and codec, allowing content to determine when the loop emits. The four-read resource cap returns exhaustion, never a forced answer.

The first fit passes the frozen typed mixed-depth gate: 192/192 correct development answers and stopping depths, all96changed-source pairs, and32/32four-read paths absent from training. AlwaysEmit gets96/192, FixedTwo56/192, AlwaysRead/ReadDisabled0/192. Disabling query update or its selected payload loses all96long cases. The new adaptive loop and old paths both retain all8256one-read and2176dependent cases;2240new outputs reproduce after reload.

Only a256-entry content-action table is trained, from actual final-answer matches along retained geometric rollouts. No expected depth/stop position enters inference. Terminal/link byte domains are declared; all development action bytes are seen in training. This establishes context/depth composition and content-class stopping, not unseen byte-class transfer, joint gradient training, variable-length text or prose. Retain15baec48; no promotion.

Validation:3new focused tests and14retained tests, separately executed sealed fit/report, format/claim wording/diff checks. Independent review strengthened retention to exercise the new loop itself and guarded checkpoint replay deadlines. Original dirty checkouts, all35priorsealedroots/1060files and artifacts are preserved. Complete model/build/test charge126359/1800000ms, with no allowance extension, paid compute or cleanup. CI statuses are compatibility acknowledgements, not test execution.

Result/evidence and current state record exact controls, artifact/source identities, cumulative resources and the next short variable-length text/common-objective learning boundary. Final independent qualification remains NOT_RUN.

References #973, #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

