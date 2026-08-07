# The query-projection asymmetry is real, and fixing it does not pay

Issue #480. Measured 2026-08-07 on the committed 500k fixture, 2,000
construction stories → 46,342 distinct windows, 500 held-out probes, through
`get_top_resonances_native`.

Reproduce with:

```
cargo test --release -p uor-r4-router --test query_projection -- --ignored --nocapture
```

About eleven minutes for three arms; no teacher, no checkpoint.

## The asymmetry

PR #465 adopted full-width content-bearing storage — `banded_storage` defaults
to false and stored vectors keep all 512 coefficients. The query side did not
follow. `retrieve_geometric_resonance` built a band-only projection, zero
outside `active_range`. Zeroed query coordinates contribute nothing to a dot
product, so the cosine only ever saw the band however wide the stored vector
was.

That is a genuine wiring asymmetry and it is exactly what the #465 scope note
suspected. The hypothesis attached to it — that the adopted de-banding gain
(retrieval MRR 0.2348 → 0.8948, router anchor accuracy 9.3% → 11.4%) was
therefore going unrealized at serving — is **refuted**.

## Result

| arm | store / query | top-1 | MRR | recall@20 | tie-mass |
|---|---|---:|---:|---:|---:|
| `banded` | banded / banded (pre-#465) | 0.6240 | 0.7195 | 0.9640 | 0.7580 |
| **`shipped`** | **full / banded (deployed)** | **0.6240** | **0.7179** | **0.9720** | 0.8120 |
| `symmetric` | full / full (the proposed fix) | 0.6320 | 0.7238 | 0.9540 | 0.7860 |

Deranged-key null 0.0000 in every arm. The `shipped` row reproduces the #479
figures exactly, which is the check that the knob added here leaves deployed
behaviour untouched.

**Fixing the asymmetry is worth +0.0059 MRR and +0.0080 top-1, and costs
0.0180 of recall@20**, against a pre-declared +0.05 MRR bar. Roughly a tenth of
the threshold on the metric it was supposed to move, and negative on recall.

Note also that `banded/banded` beats `shipped` on MRR (0.7195 vs 0.7179). The
de-banding gain does not reach this path from the storage side either.

## Why it cannot pay here

The relevance form is

```
relevance = shared_count * 100 + sim * slice_norm + scope_boost
```

`shared_count` is the number of query primes present in the item, multiplied
by **100**. `sim` is bounded by 1 and `slice_norm` is a band-slice norm.
Observed relevances look like 415 / 215 / 115 — four, two and one shared words
plus a fractional cosine. **The ranking is lexical; the cosine is a
tie-breaker.**

A vector-shape change can therefore only reorder candidates already tied on
word overlap. The harness reports that tie-mass directly: 81.2% of probes do
have a same-bucket competitor, so the reachable slice is large — and even
across that slice, widening the query vector moves almost nothing. It is not
that the cosine had no room; it is that the cosine's input shape barely
changes the ordering it produces.

The 0.8948 figure in #442 came from a harness that ranked by **cosine alone**,
with no lexical term. That number was never going to appear on this path, and
the original filing (mine) was wrong to treat the gap as unrealized gain. The
correct statement: de-banding is worth what #442 measured *for cosine-ranked
retrieval*, and this serving path is not cosine-ranked.

## Disposition

**Not adopted. Deployed behaviour is unchanged.** The symmetric shape sits
behind `set_full_width_query`, default off, for three reasons:

1. the measured gain is a tenth of the pre-declared bar and recall moves the
   wrong way;
2. changing retrieval ordering moves the pinned #421 anchor-accuracy rows, and
   the regression gate that would catch downstream damage
   (`router_reconnect.rs`) needs a scored R4G1 artifact that was not available
   for this run — shipping an ordering change without it would be exactly the
   kind of unmeasured move the run-contract discipline exists to prevent;
3. it becomes the right default the moment the lexical term stops dominating,
   so the capability is worth keeping reachable rather than deleting.

`retrieve_geometric_resonance` now carries a comment saying the asymmetry is
known, measured and deliberate, pointing here — so this is not re-filed from a
fresh reading of the code.

## What would change the answer

The lexical term is the reason this lever is dead, so the live question is
whether `shared_count * 100` is the right ranking at all. It is a hard-coded
weight, never measured against alternatives, and it dominates a geometric
retrieval stack by two orders of magnitude. Measuring *that* weight — or
ranking by cosine within lexical buckets rather than adding the two — is the
experiment this record points at. It is not filed as an issue: it needs an
owner and a pre-declared exit rule, and it is a larger question than #480 was.

## Scope limits

- Synthetic word renderings of token ids (`t00042`), the standing #422/#423
  law for this stack.
- One corpus, one probe form, `top_n = 20`. The recall figure is a
  within-top-20 measure and the 0.018 movement is roughly 2 standard errors at
  500 probes — small, but pointing the wrong way, which is enough at a +0.0059
  MRR gain.
- `slice_norm` was deliberately left on the band slice in the symmetric arm:
  it is a scale factor on the cosine term, not part of the vector shape, and
  moving both at once would have confounded the measurement.
