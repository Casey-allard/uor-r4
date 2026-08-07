# The lexical weight is not the problem — there is no geometry to suppress (#484)

Record for issue #484. Measured 2026-08-07 on the committed 500k fixture,
2,000 construction stories → 46,342 distinct windows, 500 held-out probes,
through `get_top_resonances_native`. Corpus, probe form, caps and deranged key
are `query_projection.rs`'s verbatim, so these numbers are comparable to
`docs/query_projection_480.md` rather than merely similar to it.

Reproduce with:

```
cargo test --release -p uor-r4-router --test lexical_weight -- --ignored --nocapture
```

About thirty-five minutes; no teacher, no checkpoint.

## What was asked

#480 closed by finding that this stack's retrieval ranking is lexical, not
geometric:

```rust
relevance = shared_count * 100.0 + sim * slice_norm + scope_boost
```

and named the follow-on question without filing it: the `100.0` is a
hard-coded literal, never measured against alternatives, sitting two orders of
magnitude above the cosine it is added to. The hypothesis was that it might be
**suppressing** a geometric signal — the reason to look being that #442
measured 0.8948 MRR ranking by cosine alone while this path measures 0.7179.

## The weight is on a plateau

`W` swept over five decades, everything else at the deployed default:

| W | top-1 | MRR | recall@20 | tie-mass | null MRR |
|---:|---:|---:|---:|---:|---:|
| 0 | 0.0000 | 0.0000 | 0.0000 | — | 0.0000 |
| 1 | 0.6240 | 0.7179 | 0.9720 | 0.8120 | 0.0000 |
| 10 | 0.6240 | 0.7179 | 0.9720 | 0.8120 | 0.0000 |
| **100 (shipped)** | **0.6240** | **0.7179** | **0.9720** | 0.8120 | 0.0000 |
| 1,000 | 0.6240 | 0.7179 | 0.9720 | 0.8120 | 0.0000 |
| 100,000 | 0.6240 | 0.7179 | 0.9720 | 0.8120 | 0.0000 |

The `W = 100` arm reproduces #480's shipped row exactly (0.6240 / 0.7179 /
0.9720), which is the harness's binding validity check — that arm *is* the
deployed configuration, so anything else would mean the corpus or the probes
moved and the sweep said nothing.

**Every weight from 1 to 100,000 gives bit-identical results.** The reason is
measurable and had never been measured: the harness reports the dynamic range
of the non-lexical part of relevance, and it is **about 0.37**
(`sim * slice_norm` spans roughly −0.119 to +0.249, because `slice_norm` is
small). So any lexical weight above ~0.4 already yields strict lexicographic
order with the cosine as a within-bucket tie-break. The shipped `100.0` is not
a tuning parameter; it sits on a plateau whose left edge is below 1.

**NEGATIVE against the pre-declared exit rule** (≥ +0.05 MRR): the best
alternative weight moves MRR by +0.0000.

## The bigger finding: the cosine is noise

`W = 0` scoring 0.0000 on every metric is an all-zero arm, and this repository's
standing rule is that an all-zero arm is a harness bug until proven otherwise.
Two checks were added rather than assuming, and both were necessary.

**First check — the confound.** `W = 0` is *not* a cosine ranking.
`slice_norm` is a per-window-**bucket** scalar, so `sim * slice_norm` is not
comparable across buckets and the resulting order is driven by which bucket
has the largest slice norm. A `set_unscaled_geometric_term` knob was added to
rank by the bare `sim`. It scores 0.0000 too — so the confound did not change
the answer, but reporting the scaled row as the cosine comparator would have
been the wrong quantity presented as the right one.

**Second check — the full list.** A truncated `recall@20` of zero cannot
distinguish "no signal" from "broken harness" from "good ranking, just outside
twenty". Ranking all 46,342 candidates settles it:

| ranking | probe | containment | median rank | full-list MRR |
|---|---|---:|---:|---:|
| shipped (W=100) | shipped | 1.0000 | **1** | 0.7186 |
| bare cosine | shipped | 1.0000 | 21,401 | 0.0001 |
| bare cosine | ordered | 1.0000 | 21,401 | 0.0001 |
| bare cosine | **identity** | 1.0000 | **21,082** | 0.0001 |

Random over this candidate set is a median rank of 23,171.

The target is in the candidate set for **every** probe under **every** ranking
— retrieval is perfect and all of the difference is ordering. The lexical term
puts the target at median rank **1**. The cosine puts it at median **21,401**,
about 8% better than chance.

The `identity` row is the one that closes the question. It queries with the
**exact stored sentence** — not a subsample, not reversed — and the cosine
still ranks that sentence at median 21,082 out of 46,342. The `ordered` row
removes the reversal alone and changes nothing. So this is not a probe form
that defeats a working geometry, and it is not subsampling or word order:

**on this path the geometric similarity does not identify a sentence from
itself.** All measured retrieval quality is word overlap.

## What this settles

- **The lexical weight is not suppressing anything.** There is no signal
  underneath it. The premise this issue was filed on — mine — is refuted.
- **#480's negative is now explained mechanistically** rather than
  empirically. Reshaping the query vector could not pay because the cosine it
  feeds carries no retrieval signal at any shape.
- **#442's 0.8948 cannot be a property of this retrieval path.** Whatever that
  harness measured, ranking this path by cosine gives full-list MRR 0.0001.
  The de-banding gain is real for what #442 measured; it does not describe
  `get_top_resonances_native`.
- **Vector-shape and query-projection work on this path should not be
  re-proposed** on the premise that a geometric gain is being masked. That
  thread, opened by #480 and carried as folklore since, is closed.

## What it does NOT settle, stated so it is not over-read

- **Scope.** Synthetic word renderings of token ids (`t00042`), the standing
  #422/#423 law for this stack; one corpus; one indexing path. This measures
  `get_top_resonances_native` as deployed, not "R⁴ geometry" in general — the
  Spectral retrieval measured in `geometry_ablation.rs` (#434: 0.7179 MRR /
  0.972 recall@20) is a **different** path and is not implicated.
- **Cause not diagnosed.** This says the cosine between a query's state vector
  and a stored sentence's state vector does not identify the sentence. It does
  not say why. Candidates worth an owner: the query projection is band-only
  while storage is full-width (#480 — measured not to matter for *ordering*,
  but never checked for whether the vectors are comparable at all); the state
  vector may be a routing/phase state rather than a content embedding, in
  which case cosine is simply the wrong comparison for it.
- **No default was changed.** `DEFAULT_LEXICAL_WEIGHT` stays at 100.0 and the
  two new knobs default off. Changing retrieval ordering moves the pinned #421
  anchor-accuracy rows and needs `router_reconnect` against a scored R4G1
  artifact — the gate #480 recorded. Nothing here asks to move it: the sweep
  is flat, so there is no ordering change worth gating.

## Follow-up worth an owner

**Why does the cosine not identify a sentence from itself?** That is now a
sharp, cheap question with a harness already pointed at it — the `identity`
row of this record is a one-line change away from bisecting it (compare the
query's state vector against the stored vector directly, before any ranking).
Until it is answered, the honest description of `get_top_resonances_native` is
a word-overlap retriever with a decorative geometric tie-break, and the
`DEFAULT_LEXICAL_WEIGHT` doc comment now says so.
