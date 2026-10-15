# #1271 — Qualify exact phrase-order identity on the unchanged geometric model

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T08:13:33Z
- **Updated:** 2026-10-09T13:17:47Z
- **Closed:** 2026-09-14T08:14:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1271
- **Merged:** 2026-09-14T08:14:51Z
- **Merge commit:** d3326fb8d4feb7d8a30a9004045a4b81975bf7cb
- **Base / Head:** main / UOR-Foundation:codex/phrase-order-identity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The qualified correspondence artifact had not been tested with reversed phrases containing the same words competing under the same relation. This adds a separate raw-text oracle and an 80-case diagnostic covering phrase order, content gaps, active/inactive changes, missing exact endpoints and conflicting exact endpoints in both relation directions and all record positions.

**PASS_PHRASE_ORDER_IDENTITY:** the unchanged artifact passes all 80 complete cases (48 answers, 16 missing, 16 conflicting), all 16 matched families, full-trace ExactIdentity parity and reloads. Checks include actual payload/lookahead, committed queries, canonical encodings, exact compatible source identities and EOS. StructureDisabled preserves only 16/80 complete cases (24/80 output outcomes); read/update removal loses all 48 correct answers. No fit or runtime change; normal model 15baec48 remains unpromoted.

Validation: optimized offline Rust build, 29 focused tests, fmt, claim-wording and diff checks. One earlier compile-field error is preserved and charged. Model/build/evaluation time 174,801 ms; no allowance extension, external spending or cleanup. All 65 sealed report roots/1,628 files and original dirty checkouts are preserved. Prior model-control panels were not rerun because the runtime/artifact bytes are unchanged. Local checks carry validation; queue compatibility acknowledgements are not tests.

[Result](docs/native_geometric_phrase_order_973.md) and [evidence](docs/evidence/native_geometric_phrase_order_973.json) retain counts, source/artifact lineage and limits. This remains familiar authored grammar, with final independent qualification NOT_RUN. Current state records the next unchanged-artifact diagnostic for lexical identities serving different occurrence roles.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

