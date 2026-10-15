# #493 — VSA-wiring disposition (from #434): decide once the deployed geometric term is not at chance

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-08T01:46:15Z
- **Updated:** 2026-08-08T04:21:12Z
- **Closed:** 2026-08-08T04:17:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/493
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Split out from #434 / #487 (its deliverable 4).

#434 left a disposition open: what to do about the VSA wiring gap — `index_corpus` populates `corpus_index_by_identity` (which Spectral reads) and never `facet_store` (which VSA reads), so `set_geometry_type(Vsa)` after `index_corpus` silently returns zero-relevance results. Two candidate fixes were recorded: (a) have `index_corpus` also populate the facet store and re-run the ablation; (b) make the switch honest — a Vsa router that has only seen `index_corpus` input should refuse/warn rather than return zeros.

**Corrected framing (#487).** The original #434 record read the Spectral arm's 0.7179 MRR as evidence that the spectral geometry retrieves. #484/#486 showed that score is lexical word overlap (`shared_count * 100`) with the cosine term at chance on that path, because the query vector is built from the routing path, not the content vector. So the #434 ablation compared *(lexical ranking + a dead cosine)* against *(an empty index)* and told us nothing about either geometry.

**Therefore choosing between Spectral and VSA is premature.** The prerequisite is #490 (adopt the #486 `set_content_query_vector` fix so the deployed cosine compares like objects). Once retrieval on the Spectral path is actually geometric — not lexical — re-run `crates/uor-r4-router/tests/geometry_ablation.rs` and *then* the VSA-vs-Spectral disposition has a real signal to decide on. Until then, option (b) (make `set_geometry_type(Vsa)`-after-`index_corpus` fail loud) is a safe, cheap honesty fix that can land independently.

Blocked-informative on #490. Unassigned.

## Comments (1)

### comment 5224476374 — 2026-08-08T04:21:12Z — Casey-allard

Delivered in PR #497 (merged, main `cc6b288`).

**Measured, not assumed** (the #434 premise was stale):
- `index_corpus` **does** populate `facet_store` now (via `index_sentence_routed` → `index_semantic_object`), so the candidate set is real — contra the #434 / #487 "never touches facet_store" claim.
- The VSA `0.0000` is a scoring category error: the scorer cosines the query's **1024-dim VSA hypervector** against the stored **512-dim spectral content vector**, and `cosine_similarity` returns exactly `0.0` on the length mismatch, so every candidate scores 0.0.
- A commensurable comparison ranks **at chance** — a "quick brown fox" query ranked the fox sentence **last** — because `VsaGeometry::ground` hashes the exact content string (`expand_atom`): a content-hash placeholder, not a semantic encoder. Nothing semantic to rank by.

**Disposition: option (b), make the switch honest.** `set_geometry_type("vsa")` now warns loudly that VSA retrieval does not rank by content similarity; `tests/vsa_scoring_honesty.rs` pins the corrected facts (facets populated, scoring degenerate) so the record can't drift; `docs/geometry_ablation_434.md` carries the measured mechanism (superseding both the #434 and the #487 stale claims).

The real **fix-vs-deprecate** decision (VSA needs a semantic encoder) is an engine-design call, filed as **#496** for the owners. No deployed-path change: Spectral is the default; VSA is opt-in and unused in serving.



---
_Comment/review ids in this file: 5224476374_

