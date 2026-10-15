# #636 — feat(#600): explicit source/compiled geometry + versioned projection

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T10:07:26Z
- **Updated:** 2026-10-09T14:17:07Z
- **Closed:** 2026-08-12T10:26:22Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/636
- **Merged:** 2026-08-12T10:26:22Z
- **Merge commit:** cfb10d3f22707e8787855ec0e9834588205c3140
- **Base / Head:** main / UOR-Foundation:i600-geometry
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

feat(#600): explicit source/compiled geometry + versioned projection

The hidden 576->288 bucket-average becomes the named, versioned
projection bucket-average/1: a free deterministic function
(bucket_average_project) with identical iteration order and arithmetic,
a typed GeometryProjection record (id, version, widths, params,
implementation digest = blake3 over the pinned parameter declaration -
not source text, documented why), and a registry that refuses unknown
(id,version) via the sanctioned error surface. TeacherOracle gains a
defaulted geometry_projection() (HF oracle overrides). Provenance rides
the #597 seams: flag -> options -> CoverReport + ObservationManifest
(serde(default) additive; legacy bytes unchanged when unset); observe
paths bind automatically from the oracle; server binds opportunistically.
Artifact CONTAINER bytes deliberately untouched: binding the record
into TLA/R4G1 bytes cannot be additive (PROV section is defined but
never emitted; emitting breaks existing readers) and would move kappas
of fresh identical compiles - the era decision is filed separately
rather than chosen silently. All kappa/rebuild/era fixtures green
without re-pin (deterministic_rebuild --include-ignored reproduces the
pinned container byte-identically). 15 new tests incl. non-divisible
widths and canonical-serialization byte stability. Docs section +
migration note (absent metadata = legacy-era interpretation, not
relabeling).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

