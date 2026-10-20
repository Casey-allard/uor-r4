# #497 — fix(#493): make the VSA switch honest — its zero is a scoring mismatch, not an empty store

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T03:56:44Z
- **Updated:** 2026-10-09T13:08:51Z
- **Closed:** 2026-08-08T04:17:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/497
- **Merged:** 2026-08-08T04:17:20Z
- **Merge commit:** cc6b288b1dcf0d4f2db27f90d3392e4aefe04480
- **Base / Head:** main / UOR-Foundation:fix/vsa-honesty-493
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #493 (disposition of #434 item 2, unblocked by #490).

## Measured, not assumed
The #434 record (and my own #487 correction) said the VSA `0.0000` was a wiring gap — "`index_corpus` never populates `facet_store`". Probing current code shows that is **stale**:

- `index_corpus` → `index_sentence_internal` → `index_sentence_routed` grounds each sentence and calls `index_semantic_object`, so `facet_store` **is** populated (5 type keys / 2 entity keys on a 5-sentence probe). The candidate **set** is real.
- The zero is a **scoring category error**: `retrieve_vsa_multi_facet_resonance` cosines the query's **1024-dim VSA hypervector** against the stored **512-dim spectral content vector**, and `cosine_similarity` returns exactly `0.0` on a length mismatch → every candidate scores `0.0`, dead ranking. Same class as #486.
- Making it commensurable does **not** rescue it: `VsaGeometry::ground` hashes the exact content string (`expand_atom`), so grounding is a **content-hash placeholder, not a semantic encoder** — a "quick brown fox" query ranked the fox sentence **last**. Nothing semantic to rank by.

## Disposition: option (b), make the switch honest
- `set_geometry_type("vsa")` now **warns loudly** that VSA retrieval does not rank by content similarity (fail-loud at the knob).
- `tests/vsa_scoring_honesty.rs` pins both corrected facts — facets populated, scoring degenerate — so the record can't silently drift and any future scoring change is forced to confront the disposition.
- `docs/geometry_ablation_434.md` carries the measured mechanism, superseding the stale wiring-gap claim.

The real **fix-vs-deprecate** call (VSA needs a semantic encoder) is an engine-design decision, filed as **#496** for the owners.

## Scope / safety
No deployed-path change: Spectral is the default geometry; VSA is opt-in and not set anywhere in serving. `selective_backoff` (the existing VSA test) still passes. fmt / clippy `--all-features` / claim-wording gate all clean.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

