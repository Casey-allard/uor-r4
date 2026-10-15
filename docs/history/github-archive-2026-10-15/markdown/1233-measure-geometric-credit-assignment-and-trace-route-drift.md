# #1233 — Measure geometric credit assignment and trace route drift

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T00:55:04Z
- **Updated:** 2026-10-09T13:17:21Z
- **Closed:** 2026-09-13T00:57:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1233
- **Merged:** 2026-09-13T00:57:17Z
- **Merge commit:** 11efc0d496946662df8ea15f3bdf506b61bef52a
- **Base / Head:** main / UOR-Foundation:codex/geometric-credit-assignment
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The shared geometric learner needs credit for decisions whose later uses depend on earlier state. This adds a development-only Rust categorical occurrence-credit prototype and a direct tied-parameter oracle comparison. The frozen probe fails: 0/8 proposed updates improve hard loss; the one harmful read update has 69/120 improvement-direction disagreements. A separate trace reproduces its saved losses and shows baseline nine accesses become five or six under the full replacement, explaining why summing isolated occurrence effects can predict the wrong sign.

Adds aligned categorical Hamming for state roots, selected sources and predictions, preserving actual loss alongside drift counts. Production runtime/training/artifact sources and all model artifacts are unchanged. No fit, holdout, generation rerun or promotion. The next development action is a categorical learner using complete tied trajectories, checked on a bounded joint block before a larger campaign.

Also preserves the owner-supplied SpiralCore v68 HTML and FBS revision 3.1 PDF (LFS) byte-for-byte, with provenance and a scoped attention/address/Bell/trace review. Their identity/history/observable distinction informs the trace; no imported routing or simulation mechanism is inserted into serving. Original Downloads files and v63 adapter remain untouched.

Validation: four focused release tests; actual saved-artifact probe; actual follow-up trace reproducing tied and occurrence losses; formatting, claim wording, source/retained hash and sealed file verification. Model/build/test charge 289229/420000 ms; added storage 43.1 MB. A pre-recorded 400000 ms local cycle increase stays within the unchanged cumulative ledger ceiling; no paid compute or cleanup. Queue acknowledgements are not tests.

References #973 and #964. Both remain open; general language and project acceptance are not met.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

