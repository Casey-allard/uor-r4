# Three retrieval negatives were one dead-cosine artifact (issue #500)

Measured 2026-08-08 on the committed 500k fixture corpus, 2,000 construction
stories → 46,342 distinct eight-token windows, 500 held-out probes, through
`get_top_resonances_native` — the production retrieval surface. Same corpus,
caps, probe form and function as #434 / #480 / #484 / #486, deliberately, so the
numbers are comparable rather than merely similar.

Reproduce:

```
# deployed path (content-query, #490 default)
R4_LEXW_CONTENT_QUERY=1 cargo test --release -p uor-r4-router --test lexical_weight -- --ignored --nocapture
# pre-#490 dead-cosine baseline (the regression tell)
cargo test --release -p uor-r4-router --test lexical_weight -- --ignored --nocapture
# projection arms + deployed content arm side by side
cargo test --release -p uor-r4-router --test query_projection -- --ignored --nocapture
```

## The claim

Three separately-filed negative results on the retrieval track —

- **#480**: matching the query projection shape to the storage shape "does not
  pay" (+0.0059 MRR, below the +0.05 bar);
- **#484**: the `shared_count * 100` lexical weight is "inert" (`W = 1 … 100,000`
  bit-identical);
- **#434**: Spectral vs VSA geometry is indistinguishable (both collapse to the
  same number),

— were all measured against a cosine that #486 then proved was **at chance**, and
all three flip once #490 makes that cosine real. They were not three findings.
They were one measurement error, seen three times.

## The single root cause

`retrieve_geometric_resonance` ranks by

```text
relevance = shared_count * 100 + sim * slice_norm + scope_boost
```

Before #486 the query vector was built from the **routing** path while stored
vectors are the **content** vector. The two are different objects, so `sim` was
noise: the target sat at the 0.4938 percentile of all candidates even when the
probe was the exact stored sentence (chance is 0.5). With a dead cosine, the
`shared_count * 100` lexical term dominates by construction — and that single
fact produces all three negatives:

| filed as | what it varied | why it read flat |
|---|---|---|
| #480 | query SHAPE (band vs full projection) | reshaping a non-comparable vector can't pay at any shape |
| #484 | lexical WEIGHT (`W`) | no geometric signal for `W` to trade against |
| #434 | GEOMETRY (Spectral vs VSA) | neither geometry's cosine contributed; 0.7179 is the shared lexical term |

The tell, already noted in `geometry_ablation_434.md`: #434, #480 and #484 each
re-derived the identical triple **0.6240 / 0.7179 / 0.9720** without noticing it
was one dead-cosine measurement three times.

## What the fixed cosine says

#490 builds the query from the content vector — the same construction the stored
side uses — which is full-width by nature and comparable to what is stored. On
the deployed path, on the same corpus and caps:

| path | W | top-1 | MRR | recall@20 |
|---|---:|---:|---:|---:|
| pre-#490 routing query (dead cosine) | 100 | 0.6240 | 0.7179 | 0.9720 |
| #490 content query (deployed) | 100 | 0.7840 | 0.8542 | 0.9900 |
| #490 content query, lexical term dropped | 0 | 0.8160 | 0.8763 | 0.9900 |

Re-measured on current `main` (d0de35f) at the standard caps. The deployed
content row (0.8542 MRR) is the diagnostic full-list 0.8545 and the `W = 1 … 10`
sweep rows to four places; `W = 0` lifts both top-1 (+0.032) and MRR (+0.022) and
takes recall to 0.9900. On the pre-#490 path every `W ≥ 1` is bit-identical (the
#484 "inert" observation); on the content path `W = 0` separates from `W ≥ 1`,
which is the whole reassessment in one line — the weight only looked inert
because the cosine it traded against was noise.

Both original negatives flip:

- **#480 "shape doesn't pay" → the full-width query DID pay**, ~+0.136 MRR — but
  it arrived as the content vector (#490), not via `set_full_width_query`. The
  lever was real; it was mis-measured because the query was the wrong KIND of
  object, not the wrong shape.
- **#484 "weight is inert" → the weight matters once the cosine works.** Dropping
  the lexical term is worth ~+0.022 MRR and lifts recall to 0.9900. Inert only
  because the thing it traded against was noise.

## The consistency defect this closes

The harnesses that produced these verdicts were still pinned to the dead-cosine
baseline, which is exactly why they kept "agreeing":

- **`query_projection.rs` had gone vacuous.** It never set
  `set_content_query_vector`, so under #490's default (`new()` →
  `content_query = true`) all three arms used the content vector and
  `set_full_width_query` was a no-op. `shipped` and `symmetric` were
  byte-identical; the verdict was exactly `+0.0000`. Being `#[ignore]`d, CI never
  caught it. Now the projection arms set `content_query = false` explicitly (so
  they measure the shape lever they claim to), and a fourth `content (deployed)`
  arm shows the realized #490 win beside them.
- **`lexical_weight.rs` called the pre-#490 path "deployed."** Its default forced
  `content_query = false` and pinned 0.6240 / 0.7179 as "the deployed
  configuration." Since #490 that is not what ships. It now pins two labelled
  baselines: the dead-cosine row (`SHIPPED_*`, tight pin, the canonical tell) in
  default mode, and the deployed content row (`CONTENT_*`) in
  `R4_LEXW_CONTENT_QUERY=1` mode, with an assertion that the deployed row sits
  clearly above 0.7179 so the two baselines cannot silently converge again.

Two harnesses that disagreed about what "deployed" means cannot produce
commensurable baselines. Pinning one post-#490 reference row across the track is
what makes future comparisons non-frivolous.

## Disposition

- #480 and #484 records reconciled from NEGATIVE to SUPERSEDED-by-#490 in
  `RESEARCH.md`, with the flipped numbers and the shared root cause.
- Harness vacuity and stale "deployed" pins fixed as above.
- The `W = 0` (drop the lexical term) result is a serving-path simplification
  worth ~+0.022 MRR and +recall. Because flipping `DEFAULT_LEXICAL_WEIGHT` has
  blast radius on the non-content path (where the scaled `W = 0` arm is confounded
  by `slice_norm`), it is filed as an adoption gate (#502) — the same discipline
  that turned #486 into #490 — not flipped here.

## The general lesson

An instrument calibrated against a broken baseline will keep confirming the
broken baseline. The cheap guard, now in place, is to pin a KNOWN reference row
and assert against it: had the harnesses done that from the start, the identical
0.7179 triple appearing under three different framings would have tripped an
assertion instead of being written down three times as three findings.
