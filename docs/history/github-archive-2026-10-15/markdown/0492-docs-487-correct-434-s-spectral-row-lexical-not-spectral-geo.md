# #492 — docs(#487): correct #434's Spectral row — lexical, not spectral geometry

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T01:27:03Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-08-08T01:44:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/492
- **Merged:** 2026-08-08T01:44:52Z
- **Merge commit:** 50e6d0e893d5712120027a903c05447308280a37
- **Base / Head:** main / UOR-Foundation:docs/correct-434-spectral-record-487
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #487.

The #434 ablation record and its harness comment presented the Spectral arm's **0.7179 MRR** as a geometry-retrieval result. #484/#486 later measured the cosine term at chance on that path (routing-vector query vs content-vector store), so the score is word overlap dominated by `shared_count * 100`, not spectral geometry.

**Changes**
- `docs/geometry_ablation_434.md`: appended a correction block (record appended to, not rewritten, per repo practice) — the figures stand, the interpretation is corrected; disposition re-opened with the corrected framing.
- `crates/uor-r4-router/tests/geometry_ablation.rs`: fixed the "reference points, for scale" comment that cited cosine-ranked 0.8948 beside this harness's lexical 0.7179 as commensurable.
- Deliverable 2 (README) was already satisfied by the #491 docs rewrite; `docs/RESEARCH.md` already carries the #434→#487 correction.

Docs + one test comment; no behaviour change.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

