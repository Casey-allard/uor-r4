# #2002 — Add a token-identity matching term to the pointer score (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T09:06:13Z
- **Updated:** 2026-10-09T13:21:32Z
- **Closed:** 2026-10-09T09:07:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2002
- **Merged:** 2026-10-09T09:07:55Z
- **Merge commit:** 4e027c9d23db55b68ed7ab6310564de917ba5b0d
- **Base / Head:** main / UOR-Foundation:deepseek/pointer-identity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Add a token-identity matching term to the pointer score

MEASURED MOTIVATION. A pointer head lets this model recall values it has never seen: 35/84 = 0.417
exact copies of invented values absent from every store, against 0/84 for the same mixture without
a pointer. Two things were then measured about the matching step. The learned hidden-state key
pathway has reachability 0.473 and a selection rate still rising (0.108 -> 0.239). The token-id
ROUTE (pointer_route=ngram:W, keys from token ids) failed: its hit rate and reachability froze to
fifteen decimal places across four thousand steps and reachability was 61% lower (0.182), because
that route ADMITS candidates by shared prime factors and n-gram overlap, and a novel value shares
no n-gram with the question asking about it. So the identity mixture wants a token-identity KEY
while novel-value recall rejects a prime/ngram ADMISSION rule, and the change is to keep the
learned pathway and add an identity MATCHING term rather than swap the routing.

THE CHANGE. Optional pointer_identity=none|WEIGHT (0..=1, stored in basis points; 0 or none
clears). For query t and source j the term is a multiset Jaccard over the six tokens strictly
before each, times a fixed sharpness, excluding the source's own token (that token IS the value
being copied, so including it would penalise exactly the novel values) and scoring 0 when the two
windows would overlap. It mixes as (1-w)*dot + w*identity and then uses the same softmax,
selection and gate machinery as before. A saved head can take the term with pointer_identity on
init=, changing nothing but configuration.

DEFAULT UNCHANGED, VERIFIED BY A REAL BEFORE/AFTER RUN. A temporary fingerprint (loss bits, eight
next_scores bits plus argmax, twenty-four f64 target-NLL bits, config JSON) was identical on the
changed tree and on HEAD via a pathspec stash that was popped and sha256-verified afterwards. With
no identity the mixing factor is exactly 1.0, so old arithmetic is untouched; the serialized key is
absent when off.

A REAL BUG FOUND BY THE NEW TEST. PointerMixture::bwd did not carry the mixing factor into the dot
query/key gradients, so gradients were 2x off at w=0.5. The finite-difference test caught it and now
pins it.

TESTS. Seven library tests (identity picks a source the dot product cannot name; weight 0 is the
dot score bit for bit; an all-zero identity term only rescales the dot scores; weight bounds are
pure dot and pure identity; a pointer saved without the field scores as before and round-trips; the
dot gradients stay exact through the mixture; a routed key fold is inert without a route), one
extended export test (an identity pointer is refused - it has no integer port), and two CLI tests
including one that switches the term on for a saved head without new weights. All pass. rustfmt is
clean on the four touched files.

HONEST LIMITS. The window (6) and sharpness (4.0) are fixed constants, matching the route's
validated 1..=6 window; they are not configurable yet. The literal reading "source tokens appear in
the query's premise" is DEGENERATE - every source's premise is a subset of the query's premise, so
containment is 1.0 for every candidate and cannot discriminate - hence the local-window comparison,
which is documented in the code. An identity pointer runs on the host kernel rather than CUDA and
refuses bf16; export refuses it because there is no integer port for it. No end-to-end training run
was made here.

Fourteen library tests fail BEFORE AND AFTER this change, all panicking at dialogue_episodes.rs:613
with an arithmetic overflow in an unmodified file, verified identical at HEAD.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

