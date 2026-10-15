# #387 — outcome_bits: transition_offset applied to candidates but not the uncovered-vocab floor — inflates bits/token by ~offset

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-03T18:19:22Z
- **Updated:** 2026-08-03T18:56:07Z
- **Closed:** 2026-08-03T18:56:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/387
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

`outcome_bits` charges uncovered vocabulary at the root floor while candidate
scores carry `transition_offset`. The offset is not applied to the floor, so on
any slice where it is large the bits/token figure is inflated by roughly its
magnitude — and a token **in** the candidate set is scored worse than one nobody
ever observed.

## Measurement

Real `SmolLM2-135M-Instruct-7e27bd9f9532` bundle, 362-region pinned cover,
`--context-order 2 --gate-c-context-window true`, bits split by whether the
teacher token was in the candidate set:

| slice | bits (all) | teacher **present** | teacher **absent** | n present | n absent | mean offset (nats) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| exact_context | 13.31 | 10.47 | 15.52 | 17,269 | 22,255 | **0.00** |
| graph | 115.05 | **143.71** | **46.75** | 460 | 193 | **−117.90** |
| novel | 91.03 | 125.55 | 41.80 | 97 | 68 | **−111.44** |

Bits are **higher when the teacher is retrieved** than when it is absent — 143.71
vs 46.75 on the graph slice. That inversion is the signature.

## Mechanism

```rust
fn outcome_bits(scorer, candidates, next) -> f64 {
    let floor = scorer.root_floor().raw();
    let max_s = candidates.iter().map(|&(_, s)| s.raw()).max()…;
    let weight = |s| ((s - max_s) / 65536.0).exp();
    …
    let uncovered = scorer.vocab() - candidates.len();
    sum += uncovered as f64 * weight(floor);
```

Candidate scores are `root_score + residual + transition_offset`. The floor is
`root_floor`, untouched by the offset. With `transition_offset ≈ −118 nats`,
every candidate sits ~118 nats below where the floor prices unseen tokens, so:

- the uncovered mass (`vocab − candidates`, ~49k tokens) dominates `sum`;
- a **retrieved** teacher is charged its depressed candidate weight → enormous bits;
- an **absent** teacher is charged `w_floor` → comparatively cheap.

`exact_context` has offset 0.00 and behaves normally (13.31 bits), which isolates
the offset as the cause: same scorer, same corpus, same run.

## Why it was invisible until now

The offset is added to **every** candidate, so it is rank-neutral. No top-1,
argmax, recall, or teacher-rank measurement can detect it. Every accuracy check
in #364 passed over it. It only surfaces in bits, and only where the offset is
large.

## Impact

- **bits/token on offset-bearing slices is inflated by roughly the offset**, so
  the graph/novel slices' 91–127 bits figures are not model error. Plausible real
  error is nearer the exact_context scale (~13 bits).
- Any bits-based comparison spanning offset-bearing and offset-free scores is not
  like-for-like. That includes several figures I posted on #364 — the residual
  "harm" of +3.09/+1.32 bits and the 12.00 root-only baseline behind the parity
  ceiling — because `bits_per_token_root_only` is computed from `root + penalty`
  with **no** offset while the with-residual figure carries it.
- **top-1 conclusions are unaffected** (rank-neutrality), so the emission-selection
  defect and its fix (#366), the exonerations, and the 1.41× cover lift all stand.

## Suggested fix

Either shift the floor by the same offset when pricing uncovered mass, or exclude
`transition_offset` from the scores passed to `outcome_bits` — it is rank-neutral
by design, so removing it changes no decision. Then re-derive the bits-based
conclusions in #364.

Note the fixture corpus appears unaffected (its graph-slice bits sit at 12–13,
in line with exact_context), which is why this only surfaced on the real bundle.

Measurement branch: `graph-bits-offset-check`.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

