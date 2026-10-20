# #1336 — Repair sparse emitter learning and reconcile geometric computation direction

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T17:42:02Z
- **Updated:** 2026-10-09T13:17:10Z
- **Closed:** 2026-09-21T18:12:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1336
- **Merged:** 2026-09-21T18:12:11Z
- **Merge commit:** 036e9c4310ca3d8d2440ac367c577a1df7edf58f
- **Base / Head:** main / UOR-Foundation:codex/consistent-emission-step
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The contextual emitter produces 25/120 final answers versus 108/120 for a development-fitted selected-value lookup, but its training used all output rows before serving pruned to 64, and the output head was not refitted after map changes. This PR retains the experimental negative, repairs those learning boundaries, and withdraws the expressivity-only diagnosis.

Training and export now share sparse ternary projection and the integer hard forward. Output fitting retains the best served CE; coefficient and map refinement use calibrated CE with an explicit alias priority. The actual runner executes output → maps → output, exports the final refit, and records committed losses/hashes. No corrected full model fit is claimed.

Independent saved-data reconstruction verifies both seven-file seals, source/executable/artifact identities, injective saved value codes and categorical counts. The second attempt is a report-only replay of the same final draw. Three short loaded generations ran and remain degenerate; missing geometric per-row predictions prevent full aggregate reconstruction. Original artifacts and failed criteria are preserved.

The principal review and canonical roadmap separate exact evidence, contextual addressing, relational computation and lexical emission. A learned lexical decoder is a reusable donor, while useful query-and-content-dependent composition, dependent reads and broader language remain goals. The next DeepSeek brief grants substantive research judgment and standing-authorized local extensions while retaining principal architectural review, preserved evidence and physical resource limits. It includes the finite-H4 spectral bridge and conditions for role/scope/Hopf/S7 reuse.

Validation: 8 contextual-emission tests, 2 read-conditioned tests, 1 map-search test, offline competitive-reader check, cargo fmt --check, claim-wording check, JSON and 920 local links, git diff --check. CI status names are compatibility acknowledgements, not these local test results. Reconciled the missing prior debit; no principal deletion or paid compute.

References #820, #973, #1139, #962, #963, #964. No model promotion or issue completion.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

