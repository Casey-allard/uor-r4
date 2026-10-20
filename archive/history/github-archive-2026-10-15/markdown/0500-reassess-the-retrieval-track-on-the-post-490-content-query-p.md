# #500 — Reassess the retrieval track on the post-#490 content-query path; #480/#484 negatives were dead-cosine artifacts

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-08T14:09:34Z
- **Updated:** 2026-08-08T15:29:31Z
- **Closed:** 2026-08-08T15:29:31Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/500
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Three negative verdicts on the retrieval track — #480 ("matching query shape to storage shape does not pay"), #484 ("the lexical weight is inert"), and #434 ("Spectral vs VSA is indistinguishable as geometry") — were all measured against a cosine that #486 subsequently proved was **at chance**. #490 fixed the root cause by making the deployed query the content vector. In light of that new data, the three negatives need reassessing, and the harnesses that produced them need re-baselining so comparisons against them are not frivolous.

This issue is the reassessment + the harness-consistency fix. It does not open a new mechanism; it revisits mechanisms we may have set aside on bad initial data.

## The common root cause

`retrieve_geometric_resonance` ranks by `shared_count * 100 + sim * slice_norm + scope_boost`. Before #486 the query vector was built from the **routing** path while stored vectors are the **content** vector, so `sim` was noise by construction (target at the 0.4938 percentile — chance — even for the exact stored sentence). With a dead cosine, the `shared_count * 100` lexical term dominated by construction. That single fact explains all three negatives:

- **#480** compared band-only vs full-width *query projection*. Reshaping a query vector that is not comparable to the stored vector cannot pay at any shape, so the +0.0059 MRR it measured was reordering noise inside lexical ties.
- **#484** swept the lexical weight `W = 1 … 100,000` and found bit-identical retrieval. With the cosine at chance there was no geometric signal for `W` to trade against, so any `W` above ~0.4 yields the same lexicographic order.
- **#434** compared Spectral vs VSA. The Spectral arm's 0.7179 was the lexical term both geometries would have shared; neither geometry's cosine contributed.

The tell, already flagged in `geometry_ablation_434.md`: #434, #480 and #484 all re-derived the identical triple **0.6240 / 0.7179 / 0.9720** under three framings without noticing it was one dead-cosine measurement three times.

## What the new data says

Once #486/#490 make the query the content vector (full-width by nature), on the same corpus, caps, probe form and function, both original negatives **flip**:

- **#480's "shape doesn't pay" is superseded.** The full-width query *did* pay — about +0.136 MRR (0.7179 -> 0.8542) — but it arrived as the content vector (#490), not via `set_full_width_query`. The lever was real; it was mis-measured because the query was the wrong *kind* of object, not the wrong shape.
- **#484's "weight is inert" is superseded.** With a working cosine, dropping the lexical term (`W = 0`, bare cosine) is worth about **+0.022 MRR** (0.8542 -> 0.8763) and lifts recall 0.9720 -> 0.9900. The weight was inert only because the thing it traded against was noise.

Exact deployed-path numbers are re-confirmed on current main in this issue's PR.

## The consistency defect

The harnesses that produced these verdicts are still pinned to the dead-cosine baseline, which is why they "agreed":

1. **`query_projection.rs` (#480) is vacuous.** It never calls `set_content_query_vector`, so under #490's default (`new()` -> `content_query = true`) all three arms use the content vector and `set_full_width_query` is ignored. The `shipped` and `symmetric` arms are byte-identical; the verdict is exactly +0.0000. It is `#[ignore]`d, so CI never caught it.
2. **`lexical_weight.rs` (#484) calls the pre-#490 path "deployed."** Its default forces `content_query = false` and pins 0.6240 / 0.7179 as "the deployed configuration." Since #490 that is no longer what ships.

Two harnesses that disagree about what "deployed" means cannot produce commensurable baselines. That is the frivolous-comparison risk this issue closes.

## Work

1. **`query_projection.rs`** — set `content_query` explicitly per arm so the projection arms mean what the docstring says, and add a deployed content-query arm so the harness spans old->new baselines in one place. Rewrite the verdict: the full-width query pays, realized as the content vector via #490.
2. **`lexical_weight.rs`** — make the deployed content-query path the primary pin; keep the pre-#490 path as a labelled reproduction, not as "deployed."
3. **Shared reference row** — pin one post-#490 content-query reference triple across the retrieval harnesses so a future run cannot silently re-measure the dead baseline and call it new.
4. **RESEARCH.md** — reconcile the #480 and #484 entries from NEGATIVE to SUPERSEDED-BY-#490, with the flipped numbers and the shared root cause.
5. **Follow-up gate** — the `W = 0` (drop the lexical term) result is a serving-path simplification worth about +0.022 MRR and +recall; changing the default weight has blast radius on the non-content path, so file it as an adoption gate the way #486 filed #490, not as a silent flip here.

## Not in scope

Changing the deployed `DEFAULT_LEXICAL_WEIGHT` (that is the follow-up gate). VSA retrieval quality (#496). This issue re-baselines and reconciles; it does not alter serving behaviour.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

