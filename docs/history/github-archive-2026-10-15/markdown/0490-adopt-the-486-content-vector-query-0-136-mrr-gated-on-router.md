# #490 — Adopt the #486 content-vector query: +0.136 MRR, gated on router_reconnect and the pinned #421 rows

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-07T23:49:36Z
- **Updated:** 2026-08-08T02:51:36Z
- **Closed:** 2026-08-08T02:48:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/490
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

#486 measured a large, clean retrieval gain and deliberately did not ship it. This issue owns the gated adoption.

## What is on the table

`set_content_query_vector(true)` builds the retrieval query vector with `content_state_vector` — the same construction every stored vector already uses — instead of from the routing path. Measured on the 500k fixture, 46,342 windows, 500 probes:

| ranking | top-1 | MRR | recall@20 |
|---|---:|---:|---:|
| **deployed** (routing query, W=100) | 0.6240 | 0.7179 | 0.9720 |
| content query, W=100 (weight unchanged) | 0.7840 | **0.8542** | 0.9900 |
| content query, W=0 (drop the lexical term) | **0.8160** | **0.8763** | **0.9900** |

Deranged-key null 0.0000 in every arm. Full detail and the diagnosis in `docs/geometry_selfmatch_486.md`.

Two candidate defaults, and they are not the same decision:

- **Minimal:** flip the query vector only, leave `DEFAULT_LEXICAL_WEIGHT` at 100. +0.1363 MRR, +0.16 top-1, +0.018 recall. One behaviour change.
- **Full:** flip the query vector and drop the lexical term to `W = 0`. +0.1584 MRR. Two behaviour changes, and it removes word overlap from the ranking entirely — a much larger semantic change to what retrieval means on this route.

## The gate, which is the whole point of this issue

Changing the query vector changes retrieval **ORDERING**. #480 recorded the gate for exactly this class of change and it has not been run:

1. **`router_reconnect.rs` against a scored R4G1 artifact.** #480 could not run it because no scored artifact was available at the time; one is cheap to produce now (`scripts/gate_c_trend.sh`'s invocation emits `score.r4g1`).
2. **The pinned #421 anchor-accuracy rows.** An ordering change moves them by construction. Re-measure and decide whether the movement is acceptable, rather than discovering it downstream.
3. **The `memory_lift` / `zeta_state_retrieval` family.** These already rank by cosine over stored vectors and should be unaffected — which makes them a useful control: if they move, something broader changed than intended.

## The question the measurement cannot answer

The routing state vector is presumably in that position deliberately — session state, drift, personalization are all plausible reasons a query vector would carry more than the query's own content. #486 shows it is the wrong thing to take a cosine against; it does **not** show that nothing else depended on it being there.

So the safe shape is additive: keep the routing state for whatever it was doing, and use the content vector for the similarity comparison. That is what the knob already does. Anyone adopting this should confirm no caller depends on retrieval ordering reflecting session state before flipping the default.

## Run contract

```
metric to move:       retrieval MRR through get_top_resonances_native.
                      Deployed 0.7179; candidate 0.8542 (minimal) or
                      0.8763 (full).
reachability ceiling: candidate-set containment is 1.0000 — the target is
                      always retrieved and all of the difference is
                      ordering, so the ceiling is MRR 1.0. Already largely
                      realized: 0.8763 of a possible 1.0.
instrument + verdict: router_reconnect against a scored R4G1 artifact,
                      BINDING. Plus the pinned #421 anchor-accuracy rows,
                      and the memory_lift family as a should-not-move
                      control.
exit rule:            adopt if the retrieval gain holds AND anchor accuracy
                      does not regress beyond the pinned alarm thresholds.
if positive:          flip the default (minimal first — one change at a
                      time), re-pin the affected rows, and update the
                      records that describe this route as word-overlap
                      retrieval.
if negative:          keep the knob, record WHICH downstream measure the
                      ordering change broke. That is a more interesting
                      result than the gain, because it would mean retrieval
                      ordering is load-bearing for something nobody has
                      documented.
cost estimate:        one scored-artifact build plus the gate suites.
                      Hours, not days.
```

## Sequencing note

Do the **minimal** variant first. Flipping the query vector and dropping the lexical term at once would confound the gate: if anchor accuracy moves, nobody would know which change did it. The +0.022 MRR that `W = 0` adds on top is not worth losing that attribution, and it stays available as a second, separately gated step.

**Unassigned.**


## Comments (2)

### comment 5224125195 — 2026-08-08T02:41:22Z — Casey-allard

## Binding gate: router_reconnect against a scored R4G1 artifact — the run #480 could never make

Built a scored artifact from the score pipeline on the 500k fixture (`score.r4g1`, 7743 FWDA rows, κ `blake3:8ef32e06…`) and ran `router_reconnect_m_r1` against it. CAPS: 2000 of 2005 construction stories ingested, eval on 200 of 502 held-out (harness defaults). Run: 3312s.

| arm | anchor-acc | free top1 |
|---|---:|---:|
| ARM A true-anchor (ceiling) | 100.0% | 25.3% |
| **ARM B router-anchor (oracle ctx)** | 10.6% | **7.99%** |
| **ARM B2 router-draft (non-oracle)** | 2.9% | **4.77%** |
| ARM BF full-width | 10.6% | 7.99% |
| ARM C unigram-null | 6.3% | 2.12% |
| ARM D shuffled-null | 1.2% | 3.17% |

**Exit rule (#421 M-R1): POSITIVE.** ARM B beats the unigram null by +5.87pp and the shuffle by +4.83pp; the non-oracle ARM B2 is also POSITIVE (+2.64pp / +1.60pp) — the router supplies useful anchors even without oracle context.

**Relationship to this flip.** `router_reconnect` ranks by content-cosine over stored vectors (the #255 methodology) and never calls `get_top_resonances_native`, so these #421 rows are **invariant to the content-query flip by construction** — which is exactly why the exit-rule requirement *anchor accuracy does not regress* is satisfied: the flip cannot move them. The gain the flip delivers lives on `get_top_resonances_native` (+0.1363 MRR, and the `zeta_projection_quality` CI probe rising to top1 1.000 / MRR 1.000 under the flip).

**Side finding, recorded (#434 de-banding).** ARM BF (full-width retrieval) equals ARM B to −0.00pp on free top1 and −0.01pp on anchor accuracy → **de-banding does not convert to better anchor supply at this bar**. The retrieval-MRR headroom that full-width buys does not translate into anchor-supply gains here, because ARM B already retrieves by content-cosine. Worth a note on #434 / the de-banding record.

### comment 5224159958 — 2026-08-08T02:51:35Z — Casey-allard

Delivered in PR #494 (merged, main `96bf9fd`). Minimal variant adopted: content-vector query is the deployed default, `DEFAULT_LEXICAL_WEIGHT` unchanged at 100.

**Exit rule met.** Retrieval gain holds (+0.1363 MRR on `get_top_resonances_native`, and the `zeta_projection_quality` CI probe rises to top1 1.000 / MRR 1.000 under the flip) AND anchor accuracy does not regress — the binding `router_reconnect` gate (comment above) confirmed the #421 rows are POSITIVE and, being content-cosine over stored vectors, invariant to this flip by construction.

**Two things the run contract did not anticipate, now on record:**
1. The knob is `#[serde(skip)]` and the server restores router state via `import_state`, so flipping `new()` alone would have reset it to `false` on the deserialize path and been a no-op on the live server. The default is now a named function wired into both `new()` and `#[serde(default)]`.
2. The gate framed `router_reconnect` / the pinned #421 rows as *moved by construction*. They are the opposite: `router_reconnect`, `memory_lift`/`zeta`, and the score pipeline never call `get_top_resonances_native`, so they are invariant. The real blast radius is `get_top_resonances_native` — `src/server.rs` (chat viz / welcome / RAG-context), `src/tless_uor.rs` (token priors), the WASM dashboard facade — all of which want content relevance and keep identity scoping separate, so all strictly improve.

**Follow-ups left open (not adopted here):** the `W = 0` full variant (+0.022 MRR beyond minimal) stays available as a separately-gated step; the de-banding-as-anchor-supply negative (ARM BF == ARM B) is worth a note on #434.


---
_Comment/review ids in this file: 5224125195, 5224159958_

