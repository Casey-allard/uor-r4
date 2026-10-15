# #2238 — Research: W(3,3) full-corpus overview for UOR-R4 (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T03:37:20Z
- **Updated:** 2026-10-12T03:44:15Z
- **Closed:** 2026-10-12T03:38:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2238
- **Merged:** 2026-10-12T03:38:21Z
- **Merge commit:** d6d03fdec6b879b9ea876e72fd40c196dd2a738a
- **Base / Head:** main / UOR-Foundation:claude/w33-overview
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** documentation (no line count) · headline unchanged. References #2032.

**What:** the owner asked for a full-corpus overview of W(3,3) Theory, a project research resource, as repo documentation to draw from.
- **Coverage:** eight read-only Sonnet agents, one shard each, at upstream `5ee26b9a`. Nothing upstream was executed.
- **Builds on** the 2026-09-03 [NEMESIS/W33 review](docs/integration/nemesis-w33-relevance.md) rather than repeating it.
- **Linked** from `PROJECT_MAP.md`.

**Key findings for UOR-R4:**
- **HoloBox:** a radix-40 Merkle tree with persistent path-copy. The pattern transfers at fan-out 3 as the long-context memory layer. It is hash-consing, not compression.
- **Exact base-3 fibre hierarchy:** 240 → 120 → 40.
- **A no-go:** W33's selector graphs don't embed in the 600-cell's 12-regular adjacency.
- **Not a source** for icosian or Z[φ] tables.
- **The corpus's own method audit:** 22.9% of its files are uncited rediscovery, and its verifiers need verifying.

**Checks:** docs only; the claim-wording gate passes.

**Would the next step differ without it?** Yes. It sets which W33 pieces feed ladder step 3 (HoloBox/FBS memory) and which don't.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5486323527 — 2026-10-12T03:37:21Z — Casey-allard — COMMENTED

Self-review at exact head 9db47263669893b8911293e8e41568e611220fd3: docs only; every claim is attributed to an upstream path or marked as inference; the upstream PASS labels are stated as self-reported; the claim-wording gate passes. APPROVE (self).


---
_Comment/review ids in this file: 5486323527_

