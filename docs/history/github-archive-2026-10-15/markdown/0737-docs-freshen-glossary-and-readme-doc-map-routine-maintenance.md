# #737 — docs: freshen GLOSSARY and README doc-map (routine maintenance)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T18:30:31Z
- **Updated:** 2026-10-09T14:17:02Z
- **Closed:** 2026-08-16T18:48:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/737
- **Merged:** 2026-08-16T18:48:34Z
- **Merge commit:** e956c5429705e4ec6a8899c684f6bd6fa5752913
- **Base / Head:** main / UOR-Foundation:docs-drift-audit-2026-08-16
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

No specific issue -- routine documentation maintenance, per standing request to keep docs current as the repo grows.

## What this fixes

While working #643/#655/#637 this session, found `docs/transformerless/GLOSSARY.md`'s "Recent Architecture" section hadn't been touched since Epic #201, and the glossary had no entries at all for the TARGET-operator framework (#602/#604/#643), the #515 dormant-lane convention, PROV/1 (#637), or the #655-C0 release-bundle manifest -- despite these being some of the largest recent additions to the project.

- Adds a "Target operators and dormant lanes (Epic #602)" glossary section: TARGET operator template, dormant-lane convention, `r4-route-attention/1`, `msa-structured-selector/1` (including its just-closed #643 A/B result), PROV/1, release-bundle manifest.
- Links `docs/matrix_operation_census.md` from README's documentation map -- it exists and is actively maintained (#655-A/B1/B2/#704) but wasn't indexed anywhere.

No code change, no behavior change.

## Verification

- Prose-only diff to two markdown files; confirmed all referenced paths/issue numbers exist (`docs/matrix_operation_census.md`, `crates/uor-r4-graph-format/src/prov.rs`, `crates/uor-r4-api/src/release_bundle.rs`, `crates/uor-r4-graph-certify/src/msa_ab_harness.rs`).
- No `.rs` files touched; no test/gate depends on `GLOSSARY.md`'s content.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

