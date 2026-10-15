# #501 — test(#500): reassess the retrieval track on the content-query path — #480/#484 negatives were dead-cosine artifacts

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T14:42:23Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-08-08T15:29:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/501
- **Merged:** 2026-08-08T15:29:29Z
- **Merge commit:** b0f72ecb04eb34597024a6af6fcd3868ba192ab0
- **Base / Head:** main / UOR-Foundation:reassess/500-retrieval-baseline
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #500.

## What this is

A reassessment, not a new mechanism. Three separately-filed retrieval negatives — #480 ("query shape does not pay"), #484 ("lexical weight is inert"), #434 ("Spectral vs VSA indistinguishable") — were all measured against a cosine that #486 proved was **at chance**, and all three re-derived the identical `0.6240 / 0.7179 / 0.9720` dead-cosine triple without noticing it was one measurement three times. #490 fixed the cosine (query built from the content vector). On that path both quantitative negatives flip.

## Re-measured on `main` (d0de35f), standard caps

| path | W | top-1 | MRR | recall@20 |
|---|---:|---:|---:|---:|
| pre-#490 routing query (dead cosine) | 100 | 0.6240 | 0.7179 | 0.9720 |
| #490 content query (deployed) | 100 | 0.7840 | 0.8542 | 0.9900 |
| #490 content query, lexical term dropped | 0 | 0.8160 | 0.8763 | 0.9900 |

- **#480 "shape does not pay" → SUPERSEDED.** The full-width query pays ~+0.136 MRR — but as the content vector (#490), not via `set_full_width_query`. The lever was real; it was mis-measured because the query was the wrong *kind* of object, not the wrong shape.
- **#484 "weight is inert" → SUPERSEDED.** On the old path every `W >= 1` is bit-identical; on the content path `W = 0` separates (+0.022 MRR, recall 0.9720 → 0.9900). Inert only because the cosine it traded against was noise.

## Consistency fixes (the "not frivolous" part)

- **`query_projection.rs` had gone vacuous**: it never set `content_query`, so under #490's default all three arms used the content vector and `set_full_width_query` was a no-op — `shipped == symmetric`, verdict exactly `+0.0000`, and `#[ignore]`d so CI never caught it. Now the projection arms set `content_query=false` explicitly and a fourth `content (deployed)` arm shows the realized #490 win beside them.
- **`lexical_weight.rs` called the pre-#490 path "deployed."** Now pins two labelled baselines — the dead-cosine `SHIPPED_*` (tight, the canonical tell) in default mode and the deployed `CONTENT_*` in `R4_LEXW_CONTENT_QUERY=1` mode — with an assert that the deployed row stays clearly above 0.7179 so the baselines cannot silently converge again.

## Verification

`cargo fmt --check`, `cargo clippy -p uor-r4-router --tests -D warnings`, and the claim-wording gate all pass. The `#[ignore]`d harnesses were re-run at full caps to produce the numbers above; reproduction commands are in `docs/retrieval_reassessment_500.md`.

## Follow-up (not in this PR)

Adopting `W = 0` (drop the lexical term) as the serving default is worth ~+0.022 MRR and simplifies the scorer, but flipping `DEFAULT_LEXICAL_WEIGHT` has blast radius on the non-content path, so it is left as an adoption gate — the same discipline that turned #486 into #490. Filed separately.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

