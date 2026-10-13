# INC-0173: WikiText-2 Replication — Native Concentration on Second Dataset

**Stage**: Publication strengthening — second-dataset replication
**Status**: Closed: KEEP.
**Proposed**: 2026-03-26
**Closed**: 2026-03-26
**Script**: `_inc0173_analysis.py`
**Results**: `results/analysis/inc0173_wt2_replication.json` (seed 0),
`results/analysis/inc0173_wt2_seed1.json` (seed 1),
`results/analysis/inc0173_wt2_confirm.json` (2-seed aggregate)

**Decision rationale:** PPL ratio HOPF/BASELINE = **1.081 (2-seed mean)** on WikiText-2 —
exactly matching PTB's 2-seed confirmed ratio (1.081). HOPF ≈ PERMUTED (|Δ|=0.03 ppl
mean, indistinguishable). BASELINE native concentration holds across seeds (eff_b=35.37
mean). eff_ratio vs DENSE = 1.56× (2-seed mean), above the 1.5× confirm threshold.
The result is confirmed at 2-seed standard on the second dataset. KEEP.

---

## Kill-List Stage

**Stage 7: Hardware-Efficiency Confirmation** (publication strengthening, not new claim)

This increment runs the INC-0171 native-concentration result on WikiText-2 instead of
PTB, with an identical model and training budget. This is a replication test, not a
new claim family. No architectural changes. No scope expansion.

---

## Mathematical Object Under Test

**Same object as INC-0171:** fixed Hopf angular routing vs learned top-1 gating (no
auxiliary loss) in a trainable 2-layer transformer FFN. The question is whether the
native-concentration result — that HOPF and PERMUTED stay within ~8–10% PPL of
BASELINE while both operate in the same concentration regime (eff_b ≈ 44–48 of 64) —
holds on a second text distribution.

---

## Motivation

INC-0171 established the result on Penn Treebank (PTB). WT2 is:
- ~2× larger training corpus (~2M vs ~930K tokens)
- Different domain (Wikipedia vs WSJ news)
- Richer vocabulary (~33K vs ~10K word types)
- Higher OOV rate at VOCAB_SIZE=5000 (intentionally kept constant for comparability)

If the native-concentration replacement result is robust, it should survive this
distribution shift under identical training conditions.

---

## Design

**Architecture:** Identical to INC-0171.
- 2-layer transformer, hidden_dim=128, 4 attention heads
- Vocabulary: top-5000 tokens from WT2 train split (same cap as INC-0171/PTB)
- seq_len=32, batch=64, K=64 experts, expert_dim=128
- Training: 4000 steps, cosine LR 3e-4, grad_clip=1.0
- Corpus: `data/lm_proxy/raw/wikitext2/`

**Four comparison conditions (same as INC-0171 confirm):**

| Condition | Gate | Routing | Notes |
|---|---|---|---|
| `DENSE` | Standard FFN (no routing) | N/A | Quality ceiling |
| `BASELINE` | Learned top-1 (no aux loss) | Supervised | Native concentration |
| `HOPF` | None (fixed geometry) | Hopf angular sectors | Replication target |
| `PERMUTED` | None (fixed geometry) | Hopf sectors permuted | Structure null |

**Design note:** VOCAB_SIZE=5000 is kept identical to INC-0171 to maximize
architectural comparability. WT2's higher OOV rate at this vocab cap is accepted.
The claim being tested is about the routing regime, not absolute PPL values.

---

## Success Condition

**Screen (1 seed, 4000 steps) — KEEP if ALL pass:**
1. `ppl_ratio_hopf_vs_baseline` ≤ 1.10 (HOPF within 10% of BASELINE — same threshold as INC-0171)
2. `ppl_ratio_permuted_vs_baseline` ≤ 1.10 (PERMUTED within threshold — confirms structural result)
3. HOPF and PERMUTED in same regime: |ppl_hopf - ppl_permuted| < 5 absolute PPL
4. HOPF no expert collapse: eff_buckets > K/4 = 16
5. BASELINE native concentration: eff_b ≤ 0.90 × K = 57.6 (same concentration guard as INC-0171)

**Confirm (2 seeds) only if screen is borderline (ratio in 1.10–1.15).**

---

## Falsification Condition

**KILL (replication fails):**
- `ppl_ratio_hopf_vs_baseline` > 1.15 — gap widens materially on WT2; result is PTB-specific
- HOPF and PERMUTED diverge by > 10 PPL — routing geometry matters on WT2 (unexpected)

**REFINE:**
- Ratio in 1.10–1.15: borderline; run 2-seed confirm before verdict

---

## Metrics

- `val_ppl` (primary)
- `eff_buckets` (routing concentration)
- `ppl_ratio_hopf_vs_baseline`
- `ppl_ratio_permuted_vs_baseline`
- `hopf_vs_permuted_delta_ppl`
- `eff_ratio_dense_vs_hopf`

---

## Seed Count

- **Screen:** 1 seed (seed=0), 4000 steps — run 2026-03-26
- **Confirm:** 2 seeds (seeds 0+1), 4000 steps — run 2026-03-26 ✅ COMPLETE

---

## Claim Impact If KEEP

**Current (INC-0171 only):**
"Fixed geometric routing replaces learned top-1 gating at 8% PPL cost on PTB (confirmed, 2 seeds)."

**Strengthened (INC-0171 + INC-0173 KEEP):**
"Fixed geometric routing replaces learned top-1 gating at 8–10% PPL cost across two
standard LM benchmarks (PTB and WikiText-2), under identical model architecture and
training budget (K=64, 2-layer toy scale, 4000 steps)."

## Still Unproven After INC-0173 KEEP

1. Geometry-specific advantage of Hopf over PERMUTED (expected to remain unproven)
2. Production scale LMs
3. Top-k > 1 routing
4. Attention routing
5. Large-K regime

---

## Screen Results (seed=0, 4000 steps, 2026-03-26)

| Condition | Val PPL | eff_b | params |
|---|---|---|---|
| BASELINE | 106.96 | 31.91 | 5,660,672 |
| HOPF | 113.65 | 45.38 | 5,644,288 |
| PERMUTED | 113.86 | 45.80 | 5,644,288 |

## Confirm Results (seeds 0+1, 4000 steps, 2026-03-26)

| Condition | Seed 0 PPL | Seed 1 PPL | Mean PPL | Std | Mean eff_b |
|---|---|---|---|---|---|
| BASELINE | 106.96 | 104.74 | 105.85 | 1.57 | 35.37 |
| HOPF | 113.65 | 115.25 | 114.45 | 1.14 | 40.94 |
| PERMUTED | 113.86 | 115.09 | 114.48 | 0.87 | 42.28 |

**Key ratios (2-seed mean):**
- HOPF / BASELINE PPL: **1.081** ✅ (threshold ≤ 1.10 — PASSES; **identical to PTB's 1.081**)
- |HOPF − PERMUTED| PPL: **0.03** ✅ (geometry irrelevance confirmed on WT2, |Δ| < 0.5 ppl)
- HOPF no collapse: mean eff_b=40.94 >> K/4=16 ✅
- BASELINE native concentration: mean eff_b=35.37 << 57.6 guard ✅
- HOPF vs DENSE eff_ratio: **1.56×** ✅ (2-seed mean, threshold ≥ 1.5×)

**WT2-specific observation:** BASELINE concentrates MORE aggressively on WT2
(eff_b=35.37 mean) than on PTB (eff_b=44.00). HOPF (fixed) cannot adapt, so its
eff_b varies more across seeds (36.51–45.38). This does NOT affect the PPL
replication claim. The efficiency claim is correctly reported relative to DENSE
(K=64): HOPF uses 64/40.94 = 1.56× fewer effective expert paths than uniform
dense routing — slightly better than PTB's 1.39× because the WT2 token
distribution drives stronger expert concentration overall.

**Seed variance note:** Seed 1 HOPF eff_b=36.51 is notably lower than seed 0's
45.38. This reflects initialization sensitivity in fixed routing under a higher-OOV
regime. The PPL ratios (1.063, 1.100) are both within threshold, and the 2-seed
mean (1.081) is stable.

**Verdict: KEEP (confirm, 2 seeds) — PPL replication confirmed on second dataset.**
