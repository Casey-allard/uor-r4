# Session-manifold signature lane (#247)

The graph runtime already accepts a 288-bit signature for geometric routing,
but the serving callers derived it from the same eight-token window used as
the context. That made the lane blind to conversation history outside the
window.

This change adds an opt-in two-lane API:

- the context signature remains the input to ROUT fallback;
- the session signature is used by the existing emission-affinity bonus;
- the two lanes are deliberately separate so the first experiment cannot
  route an uncalibrated persistent-state signature into graph prototypes.

The server quantizes the persistent 512-dimensional manifold state with a
versioned deterministic three-coordinate signed projection. The direct chat
client, which has no manifold owner, uses an order-sensitive token-history
fallback. Both produce the graph's 288-bit / 36-byte signature width.

The graph runtime adds no floating-point or multiplication operations. Its
session lane reuses the existing masked Hamming distance and shift/add bonus.
ROUT fallback remains context-only until a held-out evaluation establishes
that session-state prototypes are calibrated for that path.

## A/B guard

The direct-chat regression test constructs two histories with the same final
eight-token context and different prefixes. Their session signatures differ,
while the context slices are byte-identical. The runtime test additionally
checks that enabling the lane does not change the current ROUT fallback token.

This is an architecture-level A/B guard, not a claim of general language
quality improvement; the empirical quality gate remains a follow-up measurement
on a pinned multi-turn fixture.

## Held-out measurement harness

The graph-runtime crate now includes a deterministic held-out A/B harness:

```bash
cargo bench -p uor-r4-graph-runtime --bench session_signature -- \
  .uor-models/compiled/smollm2-135m-instruct 256
```

For each sampled held-out position it keeps the eight-token context fixed,
derives one session signature from the full story prefix, and compares the
context-only result with the session-bias result. It reports corpus top-1,
teacher-argmax agreement, changed predictions, and changed scores. A reversed
prefix is used as a second deterministic A/B session history. The harness
does not opt session state into ROUT fallback and does not assert an
improvement threshold; those remain calibration decisions requiring a pinned
multi-turn quality target.
