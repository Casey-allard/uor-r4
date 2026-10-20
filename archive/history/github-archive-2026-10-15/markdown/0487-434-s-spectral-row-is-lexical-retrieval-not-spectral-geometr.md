# #487 — #434's Spectral row is lexical retrieval, not spectral geometry — correct the record

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-07T22:18:46Z
- **Updated:** 2026-08-08T01:46:29Z
- **Closed:** 2026-08-08T01:44:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/487
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

`docs/geometry_ablation_434.md` reports the Spectral arm at **0.6240 top-1 / 0.7179 MRR / 0.9720 recall@20** and concludes "Spectral wins by 0.7179 MRR against a pre-declared 0.05 margin." The README carries the same figures as evidence the spectral geometry retrieves.

#484 measured the identical triple — **0.6240 / 0.7179 / 0.9720** — for a ranking with the geometric term at chance, on the same corpus, the same caps (2,000 construction stories, 500 probes, `TOP_N = 20`), the same probe form, through the same function.

They are the same measurement. **That score is word overlap, not spectral geometry.**

## What is and is not wrong

**Correct, and it stands.** #434's factual findings are unaffected: the Spectral arm scores 0.7179, the VSA arm scores 0.0000, and the cause of the VSA zero is a wiring gap — `index_corpus` populates `corpus_index_by_identity` (which Spectral reads) and never `facet_store` (which VSA reads), so `set_geometry_type(Vsa)` after `index_corpus` silently disables retrieval. That is a real and useful result.

**Wrong, and it needs correcting.** The interpretation. `retrieve_geometric_resonance` ranks by `shared_count * 100 + sim * slice_norm + scope_boost`, and #484 established that the cosine term is at chance on this path — the target sits at median rank 21,082 of 46,342 even when the probe is the exact stored sentence, against a random median of 23,171. So the Spectral arm's 0.7179 comes from `shared_count`, which both arms would have shared had VSA's index not been empty.

**The ablation compared (lexical ranking + a dead cosine) against (an empty index).** It isolated the config switch correctly and told us nothing about either geometry.

## Deliverable

1. Amend `docs/geometry_ablation_434.md` with a correction block: the Spectral row is not evidence of spectral retrieval quality, with the #484 cross-reference and the reasoning above. Do not rewrite the original findings — the repo's practice is to append the correction, so the record shows what was believed and when.
2. Fix the README line that quotes 0.7179 / 0.972 as the Spectral result.
3. Fix the "reference points on this stack, for scale" comment at the top of `crates/uor-r4-router/tests/geometry_ablation.rs`, which cites `content-full-width 0.8948 MRR` beside these numbers as though they were commensurable. They are not: 0.8948 is cosine-ranked, 0.7179 is lexical.
4. Re-open the disposition question #434 left open (what to do about the VSA wiring gap) with the corrected framing: choosing between two geometries is premature while the deployed one's geometric term is at chance.

## Why this is filed separately from the diagnosis

The correction is worth making whether or not anyone runs the root-cause diagnostic in the companion issue. A wrong claim is currently sitting in the README and in a closed issue's record, and the repo's discipline is that negative and corrected results are kept rather than quietly superseded. It is also a docs-only change, so it takes the fast CI path.

## The general lesson, worth writing into the record

Three separate records — #434, #480, #484 — reported the triple 0.6240 / 0.7179 / 0.9720 without anyone noticing it was the same number, because each harness re-derived it under its own framing. **Identical metric triples across supposedly different arms are a signal, not a coincidence.** The cheap guard is to pin a known reference row and assert against it, which is what #484's harness does (its `W = 100` arm asserts #480's shipped row to within 0.0005) and what would have caught this a week earlier.

**Unassigned.**


## Comments (1)

### comment 5223914516 — 2026-08-08T01:46:28Z — Casey-allard

Delivered in PR #492 (squash-merged, main `50e6d0e`).

- `docs/geometry_ablation_434.md`: appended a correction block — the figures stand as measured; the *interpretation* of the Spectral row is corrected. Its 0.7179 MRR is lexical (`shared_count * 100`) with the cosine term at chance on that path (#484/#486), so the ablation compared *(lexical + dead cosine)* vs *(empty index)* and is not evidence of spectral retrieval quality. Record appended, not rewritten.
- `crates/uor-r4-router/tests/geometry_ablation.rs`: fixed the reference-points comment that cited cosine-ranked 0.8948 beside this harness's lexical 0.7179 as commensurable.
- Deliverables 1 and 3 above. **Deliverable 2** (the README line) was already corrected by the #491 docs rewrite — the main README no longer carries the figures, and `docs/RESEARCH.md` already records the #434→#487 correction. **Deliverable 4** (re-open the VSA-wiring disposition with the corrected framing) filed as #493.

Note the PR touched one `.rs` file (the harness comment), so it did not take the docs-only fast path; that is unavoidable given deliverable 3.


---
_Comment/review ids in this file: 5223914516_

