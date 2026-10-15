# #484 — The `shared_count * 100` lexical weight has never been measured against alternatives

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-07T18:39:06Z
- **Updated:** 2026-08-07T21:31:29Z
- **Closed:** 2026-08-07T21:14:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/484
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

`docs/query_projection_480.md` closed #480 by finding that this stack's retrieval ranking is **lexical, not geometric**, and named the follow-on question without filing it. Filing it now with an owner and a pre-declared exit rule.

## The quantity

`retrieve_geometric_resonance` (`crates/uor-r4-router/src/lib.rs:2025`) ranks by

```rust
let relevance = (shared_count as f64) * 100.0 + (sim * slice_norm) + scope_boost;
```

`shared_count` is the number of query primes present in the item. The `100.0` is a **hard-coded literal that has never been measured against any alternative**, and it sits two orders of magnitude above the cosine term it is added to. Observed relevances look like 415 / 215 / 115 — four, two and one shared words, plus a fractional cosine and a `scope_boost` of 15 that cannot outrank a single shared word either.

Consequences already measured, both from #480:

- The de-banding gain adopted in #465 (retrieval MRR 0.2348 → 0.8948, #442) **cannot reach this path**. That number came from a harness ranking by cosine ALONE.
- Making the query projection symmetric with storage — a real wiring fix — is worth +0.0059 MRR, a tenth of its bar, *because the cosine only reorders candidates already tied on word overlap*.

So the `100.0` is not an incidental constant. It is the reason a shipped geometry improvement is invisible at serving, and it bounds every future geometry lever on this path.

## The hypothesis, stated so it can lose

The weight is a free parameter on a continuum. At `w = 0` the ranking is pure cosine; as `w → ∞` it is strict lexicographic order with the cosine as a tie-break. **The shipped value sits at one arbitrary point on that continuum and no one has looked at the others.**

The reason to look, rather than assume 100 is fine: **#442 measured 0.8948 MRR ranking by cosine alone, and this path measures 0.7179 MRR with the lexical term.** Those are different harnesses over different corpora and are **not directly comparable** — that is precisely why they need to be put on one harness. If the geometric ranking is genuinely better here, the lexical term is not a helpful prior, it is a 0.17-MRR tax.

## Run contract

```
metric to move:       retrieval MRR through `get_top_resonances_native`,
                      #480 harness, corpus and caps held identical.
                      Shipped: 0.7179 MRR / 0.6240 top-1 / 0.9720 recall@20.

reachability ceiling: unlike #480, the reachable slice is NOT the 81.2%
                      tie-mass. Changing w reorders ACROSS lexical buckets,
                      not just within them, so essentially every probe is
                      reachable. The ceiling is candidate-set containment —
                      whether the target is in the scored list at all before
                      truncation — which is >= 0.9720 and will be measured
                      and printed as an arm-independent row. Headroom is
                      therefore ~0.25 MRR, which is large for this stack;
                      that is what justifies the run.

instrument + verdict: the same harness, same corpus, same probes, same
                      deranged null as #480. BINDING VALIDITY CHECK: the
                      w = 100 arm must reproduce #480's shipped row
                      (0.7179 MRR / 0.6240 top-1 / 0.9720 recall) exactly.
                      If it does not, the sweep is void and the harness is
                      wrong, not the weight.

exit rule:            POSITIVE if some w beats shipped by >= 0.05 MRR AND
                      recall@20 falls by no more than 0.02, with the
                      deranged-key null below 0.02 in EVERY arm.
                      (Same bar #480 was held to. A weight change is an
                      ordering change and gets no discount for being a
                      one-line diff.)

if positive:          do NOT flip the default in this PR. #480 recorded the
                      gate: an ordering change moves the pinned #421
                      anchor-accuracy rows and needs `router_reconnect.rs`
                      against a scored R4G1 artifact. File that as the
                      adoption issue, ship the knob + the measurement, and
                      say plainly that the default is unchanged pending it.

if negative (flat):   the ranking is INSENSITIVE to a weight spanning
                      decades. That retires "the lexical term is suppressing
                      the geometry" as a live explanation instead of leaving
                      it as folklore, and it closes the thread #480 opened.
                      Record it and stop; do not re-propose vector-shape work
                      on this path.

cost estimate:        ~11 min per 3 arms (#480 measured); 6 arms ~= 25 min.
                      No teacher, no checkpoint. Nothing else blocks on it.
```

## Method

Add `set_lexical_weight(f64)` beside the existing `set_banded_storage` / `set_full_width_query` measurement knobs — same pattern, default 100.0, deployed behaviour untouched. Sweep `w ∈ {0, 1, 10, 100, 1000}` plus a strict-lexicographic arm, reusing `query_projection.rs`'s corpus, probe form, caps and deranged key verbatim so the numbers are comparable to the #480 record rather than merely similar to it.

Report alongside each arm the **dynamic range of the three terms** — `shared_count`, `sim * slice_norm`, `scope_boost`. "The lexical term is 100x the cosine" is currently an inference from three observed relevance values; `slice_norm` is a per-window scalar whose distribution nobody has measured, and the ratio that matters is `w` against the *actual* spread of `sim * slice_norm`, not against `sim`'s bound of 1.

## Scope limits carried from #480

Synthetic word renderings of token ids (the standing #422/#423 law for this stack), one corpus, one probe form. A weight tuned on synthetic word overlap is a finding about this ranking form, not a tuned production constant — which is another reason a positive result routes to an adoption issue rather than straight to a default change.


## Comments (1)

### comment 5222207742 — 2026-08-07T21:31:27Z — Casey-allard

Delivered in **#485**. Record: `docs/lexical_weight_484.md`. Posting against the run contract I declared before launching.

## The run contract, answered

```
metric to move:       retrieval MRR through `get_top_resonances_native`,
                      #480's corpus and caps held identical.
                      Shipped: 0.7179 MRR / 0.6240 top-1 / 0.9720 recall@20.
  → unchanged by every weight tested.

reachability ceiling: I predicted the reachable slice was ~every probe, with
                      headroom ~0.25 MRR, because changing W reorders ACROSS
                      lexical buckets rather than within them.
  → the ceiling was measured, not assumed: candidate-set containment is
    1.0000 under every ranking. The target is ALWAYS retrieved. So the
    headroom was real and the whole 0.28 MRR gap to 1.0 is ordering. That
    made the run worth launching and it is why the negative below is
    informative rather than merely null.

instrument + verdict: BINDING — the W=100 arm had to reproduce #480's
                      shipped row or the sweep was void.
  → reproduced exactly: 0.6240 / 0.7179 / 0.9720, asserted against pinned
    constants rather than eyeballed. Deranged-key null 0.0000 in all seven
    arms.

exit rule:            POSITIVE if some W beats shipped by >= 0.05 MRR with
                      recall giving up no more than 0.02.
  → **NEGATIVE. The best alternative weight moves MRR +0.0000.**

if negative:          "the ranking is insensitive to a weight spanning
                      decades; retire 'the lexical term is suppressing the
                      geometry' and stop re-proposing vector-shape work on
                      this path."
  → that is what happened, and the mechanism turned out to be stronger than
    the exit rule anticipated.

cost estimate:        ~25 min for 6 arms.
  → ~41 min for 7 arms plus the full-list diagnostic, over four runs
    (three were mine to redo — see below). Nothing else blocked.
```

## The sweep is flat, and now we know why

| W | top-1 | MRR | recall@20 |
|---:|---:|---:|---:|
| 0 | 0.0000 | 0.0000 | 0.0000 |
| 1 | 0.6240 | 0.7179 | 0.9720 |
| 10 | 0.6240 | 0.7179 | 0.9720 |
| **100 (shipped)** | **0.6240** | **0.7179** | **0.9720** |
| 1,000 | 0.6240 | 0.7179 | 0.9720 |
| 100,000 | 0.6240 | 0.7179 | 0.9720 |

Five decades, bit-identical. The reason is a quantity nobody had measured and that I only got by making the harness print it: **the geometric term's dynamic range is ~0.37**, not the ~1 that `sim`'s bound suggests, because `slice_norm` is small. Any weight above ~0.4 therefore already yields strict lexicographic order. The shipped `100.0` is not a mis-tuned constant — it sits on a plateau whose left edge is below 1, so there was never a tuning opportunity here at all.

## Why the answer is bigger than the weight

`W = 0` came back all zeros, which is the pattern this repo treats as a harness bug until proven otherwise. Two checks were needed, and I would have published a wrong claim without either.

**First: `W = 0` is not a cosine ranking.** `slice_norm` is a per-window-**bucket** scalar, so with the lexical term gone the order is driven by which bucket has the largest slice norm, not by similarity. I added `set_unscaled_geometric_term` and a bare-`sim` arm. It also scores 0.0000 — the confound did not change the answer, but reporting the scaled row as "the cosine comparator that speaks to #442" would have been the wrong quantity under the right label.

**Second: a truncated `recall@20` of zero is unreadable.** It cannot separate "no signal" from "broken harness" from "ranked well, just outside twenty". Ranking against all 46,342 candidates settles it:

| ranking | probe | containment | median rank | full-list MRR |
|---|---|---:|---:|---:|
| shipped (W=100) | shipped | 1.0000 | **1** | 0.7186 |
| bare cosine | shipped | 1.0000 | 21,401 | 0.0001 |
| bare cosine | ordered | 1.0000 | 21,401 | 0.0001 |
| bare cosine | **identity** | 1.0000 | **21,082** | 0.0001 |

Random over this set is median 23,171.

The `identity` row is the one that closes the question. It queries with the **exact stored sentence** — not subsampled, not reversed — and the cosine still ranks that sentence at median 21,082 of 46,342. The `ordered` row removes the reversal alone and changes nothing.

**So it is not the probe form, not subsampling, not word order: on this path the geometric similarity does not identify a sentence from itself.** There was never a signal for the lexical term to suppress. The premise I filed this issue on is refuted.

## What this retires

- **#480's negative is now mechanistic.** Reshaping the query vector could not pay because the cosine it feeds carries no retrieval signal at any shape.
- **#442's 0.8948 does not describe this path.** It is a cosine-ranked figure; ranking this path by cosine gives full-list MRR 0.0001. The README quoted it as a serving number and now says otherwise in place.
- **Vector-shape and query-projection work on this path should not be re-proposed** on the premise that a geometric gain is masked. That thread, opened by #480 and carried since as folklore, is closed.

## What I deliberately did not claim

This measures `get_top_resonances_native` as deployed, on synthetic word renderings of token ids (the #422/#423 law), one corpus, one indexing path. It is **not** a statement about R⁴ geometry generally — the Spectral retrieval in `geometry_ablation.rs` (#434: 0.7179 MRR / 0.972 recall@20) is a different path and is not implicated.

And it does not diagnose the cause. It says the cosine between a query's state vector and a stored sentence's state vector fails to identify the sentence; it does not say why.

**Nothing shipped.** `DEFAULT_LEXICAL_WEIGHT` stays 100.0 and both new knobs default off. A flat sweep means there is no ordering change to gate through #421 / `router_reconnect`.

## The follow-up this creates

**Why does the cosine not identify a sentence from itself?** That is now a sharp, cheap question with the harness already pointed at it — the `identity` row is one change away from bisecting it (compare the query's state vector against the stored vector directly, before any ranking). Leaving it unfiled pending your word rather than filing a third issue on my own initiative; until it is answered, the honest description of this function is a word-overlap retriever with a decorative geometric tie-break, and the `DEFAULT_LEXICAL_WEIGHT` doc comment now says so.



---
_Comment/review ids in this file: 5222207742_

