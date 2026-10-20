# #633 — feat(#597): versioned source snapshot manifest with root κ

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T03:16:49Z
- **Updated:** 2026-10-09T14:17:06Z
- **Closed:** 2026-08-12T03:35:26Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/633
- **Merged:** 2026-08-12T03:35:26Z
- **Merge commit:** 44ddcebedd9bb9c3fe1a47e0f016e7d91258b6fc
- **Base / Head:** main / UOR-Foundation:i597-source-manifest
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

feat(#597): versioned source snapshot manifest with root kappa

One canonical, schema-versioned manifest (uor-r4-source-manifest/1)
binding repository, immutable revision, license, compiler version,
source-execution mode, and every admitted file (path, bytes, blake3)
for a pinned snapshot. Root kappa = uor_addr::json::address_blake3 over
the canonical bytes (per the #620 decision). download_source emits
source_manifest.json; the kappa threads request->flags->report through
uor-r4-api, graph-compiler compile/observe, graph-cli cover, and the
server auto-binds it when the snapshot carries a manifest. CoverReport
and ObservationManifest gain serde(default) optional fields - legacy
artifacts and fixtures stay byte-identical and readable. License now
populated from the pinned models/*.json descriptor at both call sites.
Tests: canonical ordering, digest coverage, tamper, schema rejection,
deterministic double-build, and every plumbing seam. Migration note in
docs/MODEL_LIFECYCLE.md: pre-#597 weight-only kappas are not relabeled.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

