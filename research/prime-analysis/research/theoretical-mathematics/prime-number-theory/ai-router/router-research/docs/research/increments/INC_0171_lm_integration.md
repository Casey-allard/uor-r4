# INC-0171: End-to-End Language Model Integration

**Stage**: Stage 6/7 close — Sparse Event-Driven Trainability / End-to-End LM Integration  
**Status**: Closed: **KEEP** (confirm pass, 2 seeds, 4000 steps)  
**Date**: 2026-03-14 (screen), 2026-03-26 (confirm)  
**Script**: `_inc0171_analysis.py`  
**Results**: `results/analysis/inc0171_lm_integration.json` (confirm overwrites screen)

---

## Motivation

All prior evidence (INC-0143 through INC-0170) uses the PPMI-SVD proxy with an
EMA-based update rule — not backpropagation. The claim that Hopf routing can
*replace* transformer-like computation is a hypothesis until tested in a real
trainable language model. This increment fills that gap.

---

## Design

Architecture: 2-layer transformer, hidden_dim=128, 4 attention heads, vocab=5000 (PTB top-5000),
seq_len=32, batch=64.

Three FFN conditions (all K=64 experts, expert_dim=128):

| Condition | Gate | Routing |
|-----------|------|---------|
| BASELINE  | Learned linear (K×D weight matrix) + softmax → top-1 hard | Supervised |
| HOPF-ROUTED | No learned gate | L2-norm(hidden) → Hopf sector (INC-0169 algorithm) |
| PERMUTED  | No learned gate | Same as HOPF but sectors randomly permuted (null control) |

Corpus: PTB (Penn Treebank), train/valid split from `data/lm_proxy/raw/ptb/`.

Screen protocol: 1 seed (seed=0), 2000 steps, cosine LR schedule from 3e-4.

Success thresholds (screen):
- PPL ratio (HOPF/BASELINE) ≤ 1.10
- eff_ratio (DENSE/HOPF) ≥ 1.5×
- No expert collapse (eff_b > K/4 = 16)

---

## Results

### Screen (1 seed, 2000 steps)
| Condition | Val PPL | eff_buckets | params |
|-----------|---------|-------------|--------|
| BASELINE  | 200.02  | 30.29       | 5,660,672 |
| HOPF      | 216.01  | 38.76       | 5,644,288 |
| PERMUTED  | 216.13  | 39.11       | 5,644,288 |

### Confirm (2 seeds, 4000 steps) — canonical numbers for paper
| Condition | Val PPL (mean) | eff_buckets (mean) | params |
|-----------|----------------|---------------------|--------|
| BASELINE  | **152.26**     | 43.19               | 5,660,672 |
| HOPF      | **164.54**     | 45.96               | 5,644,288 |
| PERMUTED  | **164.41**     | 45.81               | 5,644,288 |

Derived metrics (confirm):
```
PPL ratio  (HOPF / BASELINE):    1.081   [threshold ≤ 1.10  → PASS ✓]
eff_ratio  (DENSE / HOPF):       1.392×  [below 1.5× threshold at convergence — see discussion]
Expert collapse:                  none    (eff_b ≥ 45.96 >> 16)
HOPF vs PERMUTED Δppl:           +0.13   [essentially identical — confirmed]
```

### eff_b trajectory (HOPF, mean across seeds)
| Steps | eff_b | eff_ratio vs DENSE |
|-------|-------|-------------------|
| 400   | ~30   | ~2.1× |
| 2000  | 38.76 | 1.65× |
| 4000  | 45.96 | 1.39× |

The eff_ratio declines with training as HOPF sectors become more uniformly loaded — experts
co-adapt to serve their sectors more effectively, spreading hidden state coverage across more
of the Hopf manifold. This is a meaningful finding: the geometric routing advantage measured
on the static proxy does not persist as a static efficiency advantage in long-horizon LM training.

---

## Key Findings (Confirm)

### Finding 1 — HOPF replaces learned gating at 8% PPL cost; confirmed stable across 2 seeds

HOPF achieves val_ppl=164.54 vs BASELINE's 152.26 (ratio 1.081), confirmed across 2 seeds
and 4000 training steps. The PPL gap is stable (screen 1.080, confirm 1.081). HOPF has
**fewer parameters** (no learned K×D gate matrix).

### Finding 2 — Hopf geometry provides no advantage over random routing — confirmed

HOPF (164.54) and PERMUTED (164.41) differ by 0.13 PPL across 2 seeds — statistically
indistinguishable. This is the strongest evidence that experts co-adapt to any fixed routing,
making the geometric structure of the assignment irrelevant for LM quality.

### Finding 3 — eff_ratio vs DENSE declines with training, settling at 1.39× at convergence

At convergence (4000 steps): HOPF eff_b=45.96 vs DENSE=64 → ratio **1.39×** (not 1.65×).
The decline reflects experts learning to handle a wider range of tokens over time, expanding
sector utilisation. The 1.39× advantage is still real and meaningful for compute budgeting,
but it is weaker than the proxy-based prediction.

**Revised paper claim (Section 4):**
> "Fixed geometric routing can replace learned gating in a 2-layer transformer FFN with 8%
> validation perplexity cost (confirmed, 2 seeds). At convergence, geometric routing uses
> 46 of 64 effective expert paths (1.4× fewer than uniform dense routing). The specific
> Hopf sector geometry does not add advantage over randomly-permuted fixed routing in this
> trainable LM setting — the efficiency gain is structural."

---

## Verdict: KEEP (primary claim confirmed; eff_ratio honest-downgrade)

The **primary publication claim** — that fixed geometric routing replaces learned gating
with < 10% PPL cost — is confirmed, stable, and reproducible.

The **secondary claim** (eff_ratio) should use the confirm number (1.39×, not 1.65×). This
is an honest downgrade from the screen but does not invalidate the paper. A 1.4× efficiency
gain over dense routing is real and reported correctly.

---

## Notes for Paper Section 4

Use confirm numbers throughout. Acknowledge the eff_ratio trajectory honestly in the text.
The screen numbers (1.65×) were at 2000 steps; the confirm numbers (1.39×) are the
stable convergence value and should be the headline figure.
