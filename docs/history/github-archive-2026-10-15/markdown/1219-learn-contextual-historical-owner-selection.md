# #1219 — Learn contextual historical owner selection

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-10T00:05:29Z
- **Updated:** 2026-10-09T13:17:36Z
- **Closed:** 2026-09-10T00:07:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1219
- **Merged:** 2026-09-10T00:07:19Z
- **Merge commit:** 606dbd6be30e3b715fe00234edd3f89d390e9a34
- **Base / Head:** main / UOR-Foundation:codex/historical-query-context
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Long historical questions followed by an owner-first instruction could return `Unknown`, and an initial fit selected the first owner when multiple revision chains were present. This change extends the existing historical reader's bounded query context and learns owner-specific selection plus current-intent deferral in the shared signed-H4 router. Retained artifact `84f82191` restores parent `8cacb2fe` exactly when its outer witness is removed; records, version links, dictionary, landmarks and other parent parameters remain frozen.

Actual prompt-only generation passes 48/48 construction targets and 64/64 separately frozen fresh targets, preserving 171/171 and 227/227 inherited cases. Both owner orders, both queried owners and absent owners are included. The rejected first-owner candidate and all original results are preserved. The window-only control preserves historical target recall but wrongly emits historical values on 16 long current requests; Full preserves their existing `Unknown` result. This is bounded historical selection, not correct long-current answers or general prose.

Validation executed locally:

- 1,300 retained construction, 72 query-owner and 216 historical-transfer comparisons remain exact; 14 older behavior/control reports have zero differences.
- The two earlier historical-field panels repair 40/40 declared failures and preserve 246/246 other cases. Twelve histories, 24 word/Rust cases and ordinary/stress Copy→Add/action checks pass; generated Rust compiles and executes.
- 44 focused tests pass. Actual-artifact validation covers 22 three-turn sequences, 6,461 input and 1,308 output checkpoint positions, 66 malformed-proof rejections and 66 disabled-parent turn comparisons.
- 330 native API checks, touched core/API release builds, formatting and claim-wording checks pass. A corrected build failure and missing-input API attempt remain charged and recorded.

The evidence records source/artifact hashes, fresh-draw ordering, matched controls and cumulative resources. The next measured gap is current-intent dispatch: a false dependent NoRead blocks current readers, and downstream source choices also need correction. General prose, general reasoning/program synthesis, alpha/frontier capability, new-artifact browser/WASM/HTTP, allocation instrumentation and complete-path energy remain unqualified or unrun. Queue statuses are compatibility acknowledgements, not tests.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

