# Long-range context: the ceiling, and what the Bott-Fock decay constant costs

Issue #424. Measured 2026-08-07. Corpus `blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf`
(from-text observation bundle, 3,000 stories / 361,693 positions).

Reproduce with:

```
scripts/obs_bundle_to_corpus.py <obs-dir> /tmp/obs
R4_CORPUS_META=/tmp/obs_meta.bin R4_CORPUS_RECS=/tmp/obs_recs.bin \
  cargo test --release -p uor-r4-graph-certify --test long_range_ceiling -- --ignored --nocapture
cargo test -p uor-r4-core --test context_horizon
```

## What this settles

`docs/r4_furey_quantum_geometric_plan.md` names `bott_fock.rs` the priority
candidate for "the long-context gap", and `docs/deferral_record_2026_08_05.md`
records it as implemented, unit-tested, and unused — awaiting a flagged Gate C
A/B against the hard 8-token window. #424 asked for that A/B.

The A/B was not run. Under the `AGENTS.md` reachability gate it should not be:
the ceiling on what it could show is below the effect any exit rule would be
written for, and the arithmetic that establishes this costs a minute rather
than hours. Both halves of that arithmetic are now shipped as instruments, so
the finding is reproducible and re-checkable rather than a claim in a comment.

Two questions had to be separated, because they have different answers:

1. **Is there long-range signal on this corpus?** Yes — worth about **+1.02pp**
   of top-1, and it is real (a stranger's document history does not reproduce
   it). But it is a one-point lever, not the multi-point prize the "long-context
   gap" framing implies.
2. **Can the fold as shipped collect it?** No. Its decay constant retains
   **16%** of that ceiling. The mechanism is right and the constant is wrong.

## Ceiling A — how much long-range signal exists (carrier-independent)

`crates/uor-r4-graph-certify/tests/long_range_ceiling.rs`. Baseline is a
backoff trigram → bigram → unigram argmax over construction stories — the shape
of the shipped NGRAM rows — evaluated on a document-level held-out split.
DISTANT means every token in the same story strictly before the 8-token window.

| Arm | best λ | top-1 | GAIN vs base | null gain |
|---|---:|---:|---:|---:|
| BASE (window only) | — | 21.62% ± 0.16pp (n=66,536) | — | — |
| CACHE (order-free bag of DISTANT) | 0.05 | 21.99% | **+0.36pp** | +0.01pp |
| INDUCTION (order-sensitive, unbounded horizon) | 1.0 | 22.64% | **+1.02pp** | −1.29pp |

**CEILING = +1.02pp**, of which +0.36pp is topical (recoverable from an
order-free bag) and **+0.66pp is order-carried**. The order-sensitive family is
therefore the right shape for whatever collects this — which is a point in
`bott_fock`'s favour, and the reason this record does not simply close the
mechanism out.

The null arm re-runs each arm with DISTANT taken from a *different* story
(fixed derangement, same position, same length). The observed gain does not
survive it — at λ=1 the null is −1.29pp — so the gain is this document's
history, not a prior on frequent continuations.

### The null is a validity gate, not a subtraction

An earlier pass of this harness reported the ceiling as `observed − null` and
got **+3.13pp**. That number is wrong for the decision. `observed − null`
measures how much better a document's own history is than a stranger's; a
deployed system's alternative is *no* distant evidence, not *wrong* distant
evidence. At high λ a wrong donor actively damages the baseline, so
`observed − null` keeps growing while the achievable gain does not. Reporting
it would have promised a carrier three points that no Gate C A/B could ever
confirm. The harness now keys on `observed − base` and uses the null only to
admit or reject the gain. The discarded figure is recorded here because the
failure mode — a null that flatters by degrading — generalises past this issue.

## Ceiling B — how far the fold can actually see (carrier-specific)

`crates/uor-r4-core/tests/context_horizon.rs`. Two streams identical except for
the token *k* positions back are folded, and the surviving L1 difference in the
256-entry state is the influence of a token *k* steps back.

| lag k | 0 | 8 | 16 | 24 | 32 | 48 | **64** |
|---|---:|---:|---:|---:|---:|---:|---:|
| L1 influence | 1,363,342 | 134,203 | 14,514 | 1,369 | 141 | 2 | **0** |
| share of immediate | 100% | 9.8% | 1.1% | 0.10% | 0.010% | ~0% | **0%** |

The update is `cell <- cell - (cell >> 2)` plus a saturated injection: a
geometric decay of ratio 3/4 per token. A token 64 positions back leaves a
bit-identical state — not a faint trace, none. About 90% of the fold's
representational mass sits inside the eight most recent tokens, which is the
window the runtime already has.

This is a property of the decay ratio alone, not of the O(1) state size, and
`bounded_horizon_is_decay_not_state_collapse` confirms the state has not simply
saturated: 32 distinct long streams still fold to 32 distinct states.

## The two ceilings together

Replaying the fold's inductive bias **losslessly** — exact counts, exact
distances, no 256-cell bottleneck — bounds what the fold itself could achieve,
because anything the fold can express this arm can express and not conversely.
Sweeping the decay constant then separates "wrong mechanism" from "wrong
constant":

| decay | horizon (tokens) | top-1 | GAIN | share of ceiling retained |
|---:|---:|---:|---:|---:|
| **0.75 (shipped)** | 24 | 21.79% | **+0.16pp** | **16%** |
| 0.85 | 43 | 21.86% | +0.24pp | 23% |
| 0.90 | 66 | 21.94% | +0.32pp | 31% |
| 0.95 | 135 | 22.24% | +0.61pp | 60% |
| 0.97 | 227 | 22.45% | +0.83pp | 82% |
| 0.99 | 687 | 22.69% | +1.06pp | 104% |
| 0.999 | 6,904 | 22.56% | +0.94pp | 92% |

The curve rises monotonically with the horizon and saturates near decay 0.99.
It is not flat, so the mechanism is not the problem. At the shipped constant
the lossless upper bound on the fold is **+0.16pp — one standard error.** No
A/B could resolve it.

`>> 2` was chosen for the P-4 operator discipline (shift, not multiply), and
any `>> n` satisfies that discipline equally. `>> 7` gives 127/128 ≈ 0.992 and
lands on the saturation point of the curve above. The constant that forfeits
84% of the available signal was never a measured choice; it was the first shift
that kept the state bounded.

## Scope limits, stated plainly

- **Documents in this bundle cap at 128 tokens**, so DISTANT never exceeds 119.
  This measures long-range dependency *within 128 tokens*. It licenses no claim
  about book-length context, and a corpus with longer documents could show a
  larger ceiling. That is the named follow-up if the long-context question is
  ever reopened.
- The baseline is a backoff n-gram at 21.62%, weaker than the full serving
  stack. A stronger baseline has already captured more of the same signal, so
  **+1.02pp is generous** — the real headroom against the shipped stack is at
  most this and probably less.
- The ceiling is an upper bound on the *lossless* replay. A real 256-cell
  integer fold pays quantization and interference costs on top.
- Two independent implementations (a Python prototype and the shipped Rust
  harness) agree to two decimal places on every arm, which is the cross-check
  that the counting is right.

## Disposition

The A/B #424 specified is **not reachable** and was not run. In its place:

- **Do not activate `BottFockContextStore` at the shipped decay constant.** The
  lossless upper bound is one standard error.
- **The long-context gap is real but small on this corpus: ~1pp of top-1**,
  two thirds of it order-carried. That is the answer to the gap question the
  #290 negative left open, and it is below the ±2pp threshold recent exit rules
  have been written at.
- **If the gap is reopened, the first move is one character.** Retuning the
  decay from `>> 2` to `>> 7` costs nothing, keeps the P-4 discipline, and
  moves the lossless bound from +0.16pp to the +1.02pp ceiling. Only then is a
  flagged A/B worth its wall-clock.
- This sits below the open evidence-quality work in priority. It is consistent
  with the pattern in `README.md`: the levers that have paid improved evidence
  quality per key, and a long-range carrier is more evidence per key — but only
  about one point of it.

`context_horizon.rs` pins the shipped horizon so a future change to the decay
constant cannot silently alter the mechanism's reach without updating this
record.
