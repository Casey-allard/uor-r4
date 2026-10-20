# #311 — docs: #310 learned per-dimension weighting design note

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T21:35:18Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-07-31T21:55:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/311
- **Merged:** 2026-07-31T21:55:54Z
- **Merge commit:** a244cc9f0754af9179e7e3e178d315a16aa2cbdc
- **Base / Head:** main / UOR-Foundation:issue-310-learned-signature-weighting
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes the design-note portion of #310.

Adds `docs/learned_signature_weighting_design.md`: a review artifact sketching three transfers from a `sanity/renegade` review into the compilation/assignment paths:

1. **Weighted masked Hamming** — `d_w(s,p) = Σ_j popcount((s XOR p) AND m_j) << e_j`, strict generalization of the shipped path (b=1, e=0 = today), weights learned at compile time (contract §4), Phase A rows A-W(2)/A-W(4) proposed against the #243 decomposition matrix, with a ships-nothing exit rule.
2. **Exact k-means indexing** — Hamerly/Elkan only (byte-identical artifacts); approximate NN explicitly excluded from compiler outputs. Runtime stays exhaustive.
3. **Validation discipline** — held-out k0 comparison rows; update-store invalidation triggers (flagged as needing its own investigation).

Docs-only change; no code authorized by this document. ⚑ decision points are open pending sign-off per the doc's Sign-off section.

Claim-wording gate passes locally (`python3 scripts/check_claim_wording.py`).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

