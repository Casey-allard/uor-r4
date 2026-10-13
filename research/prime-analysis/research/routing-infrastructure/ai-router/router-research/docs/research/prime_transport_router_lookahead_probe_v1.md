# Prime Transport Router — Lookahead / Candidate Fan-Out Probe v1

> **Objective**: Measure candidate generation and scoring fan-out
> in the live forward pass. Not investigating tree search or recursion.

---

## Probe Setup

| Parameter | Value |
|-----------|-------|
| Forward passes probed | 20 |
| Batch size B | 256 |
| Sequence length D | 24 |
| Candidates per step N_OPS | 6 |
| Temperature (eval) | 0.05 |
| Instrumented function | `RouterAngularHybrid.forward` |

## 1. Candidate Count per Routing Decision

Every routing decision evaluates exactly **6 candidates** (N_OPS = 6).

```python
tn = model.TN[state_ids]               # (B, N_OPS=6, 8)  ← CANDIDATE_GEN
logits = MLP(emb, tau_prev)            # (B, N_OPS=6)     ← CANDIDATE_SCORE
w  = softmax(logits / temp)            # (B, N_OPS=6)     ← normalised scores
base = einsum('bi,bij->bj', w, tn)     # (B, 8)           ← CANDIDATE_COMB (uses all 6)
k_hard = argmax(w)                     # (B,)             ← select 1 for state advance
```

All 6 candidates are:
- **Generated**: `TN[state_ids]` fetches the tau-next row for every operator
- **Scored**: the MLP output has N_OPS=6 dimensions (one score per candidate)
- **Combined**: all 6 are used in the soft mixture `einsum(w, tn)` for tau update
- **Selectively advanced**: only 1 (argmax) advances the hard state

## 2. Future Candidates Scored?

**Future candidate depth evaluated: 0**

The MLP scores candidates using only `tau_prev` — the tau state from the
**current step t**. It does not query TN or score any future step's candidates
before that step is reached. No look-ahead into t+1, t+2, …

```python
for t in range(D):              # sequential, never skips ahead
    tn      = TN[state_ids]     # only current state_ids at step t
    logits  = MLP(emb, tau_prev)  # tau_prev is from step t-1 only
    # ... no reference to t+1 or t+k state/tau ...
```

## 3. Repeated Score Recomputation

**Repeated score recomputation: True** (state overlap events = 9052)

Some state IDs coincidentally reappear at different step positions
(9052 overlap events across 480 step checks).
This is random coincidence in the state space, not intentional recomputation.

## 4. Functions Performing Candidate Generation and Scoring

Both operations live inside a single function, in the step loop:

| Operation | Code Location | Description |
|-----------|---------------|-------------|
| Candidate generation | `RouterAngularHybrid.forward`, line: `tn = model.TN[state_ids]` | Tensor index lookup, returns (B, 6, 8) |
| Candidate scoring    | `RouterAngularHybrid.forward`, lines: `h @ W2 + b2` → `softmax` | One MLP forward, output shape (B, 6) |
| Candidate combination| `RouterAngularHybrid.forward`, line: `einsum('bi,bij->bj', w, tn)` | Weighted mixture, uses all 6 outputs |
| Candidate selection  | `RouterAngularHybrid.forward`, line: `argmax(w)` + `TR.gather` | Picks 1 of 6 for hard state advance |

## 5. Runtime Breakdown

Probed over 20 forward calls × 24 steps = 480 step invocations each.

| Component | Calls | Mean/fwd (ms) | Total (s) | % of fwd | Role |
|-----------|-------|---------------|-----------|----------|------|
| `CAND_GEN` | 480 | 1.139 | 0.0228 | 12.4% | candidate generation |
| `CAND_SCORE` | 480 | 2.165 | 0.0433 | 23.6% | candidate scoring |
| `CAND_COMB` | 480 | 1.072 | 0.0214 | 11.7% | candidate combination (soft mix) |
| `STATE_ADV` | 480 | 0.621 | 0.0124 | 6.8% | state advance (1 of 6) |
| `ATTN_POOL` | 20 | 0.428 | 0.0086 | 4.7% | trajectory pooling (not scoring) |
| **TOTAL_FWD** | 20 | — | 0.1839 | 100.0% | — |

**Candidate work (GEN + SCORE + COMB): 47.6% of forward time**
(0.0875s of 0.1839s total)

## 6. Conclusion

```
LOOKAHEAD STRUCTURE:
  NONE — no multi-step look-ahead, no future candidate scoring

CANDIDATE FAN-OUT:
  6 candidates evaluated per routing decision (N_OPS = 6)
  All 6 are generated (TN lookup), scored (MLP), and combined (einsum)
  Only 1 advances the hard state (argmax → TR.gather)

FUTURE DEPTH SCORED:  0
REPEATED RECOMPUTATION: True  (overlap events = 9052)
CANDIDATE WORK % OF FWD: 47.6%
```

### What this means

The live path evaluates **6 candidates per step × D=24 steps = 144 candidate
evaluations per sample per forward call**. All 6 candidates are scored and
soft-mixed at every step, even though only 1 advances the hard state.

The soft mixture `einsum(w, tn)` uses all 6 candidate tau-nexts to produce
the next tau state. This is the current design: the tau update sees a
weighted blend of all 6 possible next states, not just the argmax winner.

This is not look-ahead. It is a **flat fan-out of 6 at every step**,
resolved immediately by soft blending rather than tree expansion.
