# The serving path was comparing a routing vector to a content vector (#486)

Record for issue #486. Measured 2026-08-07 on the committed 500k fixture,
2,000 construction stories → 46,342 distinct windows, through
`get_top_resonances_native`.

Reproduce with:

```
cargo test --release -p uor-r4-router --test geometry_selfmatch -- --ignored --nocapture
R4_LEXW_CONTENT_QUERY=1 cargo test --release -p uor-r4-router --test lexical_weight -- --ignored --nocapture
```

## What #484 left open

Ranking by bare cosine, the target sat at median rank 21,082 of 46,342 **even
when the probe was the exact stored sentence**. Probe form, subsampling and
word order were ruled out. Nobody knew why.

## The diagnosis

Self-match over 100 probes, reporting where the target's own cosine falls
among all 46,342 candidates' cosines. **0.5 is chance; 1.0 is perfect.**

| arm | self pct | self sim | cross sim | cross sd | separation | top-1 |
|---|---:|---:|---:|---:|---:|---:|
| identity probe, band query (deployed) | 0.4938 | −0.0006 | +0.0003 | 0.0447 | −0.01 sd | 0.0000 |
| identity probe, full-width query | 0.4690 | +0.0056 | +0.0083 | 0.0430 | −0.06 sd | 0.0000 |
| shipped probe, band query | 0.4879 | −0.0018 | +0.0004 | 0.0449 | −0.05 sd | 0.0000 |
| shipped probe, full-width query | 0.4690 | +0.0056 | +0.0083 | 0.0430 | −0.06 sd | 0.0000 |
| **identity probe, CONTENT query** | **1.0000** | **+1.0000** | +0.0855 | 0.1005 | **+9.42 sd** | **1.0000** |
| **shipped probe, CONTENT query** | **1.0000** | **+0.7156** | +0.0585 | 0.0963 | **+7.57 sd** | **0.8000** |

Two hypotheses died on the first four rows:

- **Not saturated stored vectors.** The cross distribution carries real spread
  (0.045 sd). The stored vectors are perfectly distinguishable from each
  other; the query simply does not land near its own.
- **Not the band projection.** `set_full_width_query(true)` does not help. #480
  measured that the symmetric shape does not change *ordering*; it does not
  change *self-identification* either.

Indexing is deterministic (two independent indexings retrieve identically), so
none of this is noise.

## The cause

`retrieve_geometric_resonance` builds its query vector from the **routing**
path — `route_query_to_manifold_internal`, then a band of `RouteInfo::state_vector`.
`index_sentence_internal` stores a **content** vector, the L2-normalized sum of
the zeta-seeded vocabulary vectors of the sentence's words (`content_state_vector`,
issue #245).

**Those are different objects from different code paths, and their cosine is
noise by construction.** Not a tuning problem, not a shape problem — a category
error in what gets compared.

The harnesses that measured cosine retrieval *working* on this stack (#442,
MRR 0.2348 → 0.8948 — `memory_lift_ab.rs`, `zeta_projection_quality.rs`)
sidestepped it without anyone noticing: they build the query vector by indexing
the query text as a corpus item under a scratch identity and reading its stored
vector back, so both sides go through the same encode. That is why 0.8948 was
real and why it never appeared at serving.

## The fix, measured

`set_content_query_vector(true)` (default OFF) builds the query vector with
`content_state_vector` — the same construction every stored vector uses. Then
the #484 weight sweep, re-run:

| ranking | top-1 | MRR | recall@20 |
|---|---:|---:|---:|
| **deployed** (routing query, W=100) | 0.6240 | 0.7179 | 0.9720 |
| content query, W=100 (weight unchanged) | 0.7840 | **0.8542** | 0.9900 |
| content query, W=1 / 10 / 1000 | 0.7840 | 0.8542 | 0.9900 |
| **content query, W=0 (pure geometry)** | **0.8160** | **0.8763** | **0.9900** |

Deranged-key null 0.0000 in every arm.

**Changing one thing — the query vector — is worth +0.1363 MRR and +0.16 top-1,
with recall *up* 0.018.** Dropping the lexical term as well takes it to +0.1584
MRR. The pre-declared bar on this path has been +0.05 throughout; this is three
times it.

The 0.8763 also lands beside #442's 0.8948 on a different corpus and caps,
which is the check that this is the same quantity that measurement was talking
about all along.

## What this revises

**#484's flat weight sweep was conditional on a broken cosine, and the record
now says so.** "The ranking is insensitive to a weight spanning decades" was
true *of a ranking whose geometric term was noise*. With a working geometric
term the curve lifts from 0.7179 to 0.854–0.876 and the optimum moves to
`W = 0` — the opposite end from the shipped value. The plateau for `W >= 1`
survives (the lexical term still outranks a geometric term of bounded range),
but it is now a plateau at 0.8542 rather than 0.7179.

**#480's negative is fully explained.** Reshaping a query vector that was never
comparable to the stored vector could not have paid at any shape.

**#487's correction stands and sharpens.** #434's Spectral row (0.6240 / 0.7179
/ 0.9720) is the lexical ranking; the geometry it was crediting was inert.

## Not adopted here, and why

`set_content_query_vector` defaults **off** and no deployed behaviour changes in
this PR. Changing the query vector changes retrieval **ordering**, which moves
the pinned #421 anchor-accuracy rows, and `router_reconnect.rs` against a
scored R4G1 artifact is the gate #480 recorded for exactly this class of
change. That gate has not been run. Filed as the adoption issue.

There is also a design question the measurement cannot settle: the routing
state vector is presumably in that position for a reason (session state, drift,
personalization). This record shows it is the wrong thing to take a cosine
against; it does not show that nothing else depended on it. The adoption issue
carries that question.

## Scope limits

Synthetic word renderings of token ids (the standing #422/#423 law), one
corpus, one probe form, `top_n = 20`, 100 probes for the self-match table and
500 for the sweep. The `identity probe, CONTENT query` row scoring exactly
+1.0000 self-similarity is a tautology by construction — identical text through
an identical encode — and is included as the harness's sanity anchor, not as a
result.
