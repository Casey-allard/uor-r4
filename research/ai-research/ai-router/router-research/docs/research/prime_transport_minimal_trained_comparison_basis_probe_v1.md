# Prime Transport — Minimal Trained Comparison Basis Probe v1

**Branch**: minimal_trained_comparison_basis_probe_v1
**Date**: 2026-04-09
**Elapsed**: 10.2s

---

## 1. Mechanism Lock

### Minimal Trainable Component Restored

A 2-layer MLP with token embedding. **Total: 758 parameters**.

| Parameter | Shape | Count |
|-----------|-------|-------|
| W_emb     | (4, 4)  | 16 |
| W1        | (16, 32)| 512 |
| b1        | (32,)   | 32 |
| W2        | (32, 6) | 192 |
| b2        | (6,)    | 6 |
| **Total** |         | **758** |

At each phase step t:
```
emb    = W_emb[tok_t]                        # (B, D_EMB=4)
h_in   = cat([emb, tau_ang_prev], dim=1)     # (B, 16)
h      = tanh(h_in @ W1 + b1)               # (B, 32)
logits = h @ W2 + b2                         # (B, 6)
w      = softmax(logits / temp)              # (B, 6)
base   = einsum("bi,bij->bj", w, TN_ang)     # (B, 12) comparison-language formation
```

NOT restored: attention readout, prediction head, W_tok_inject, Gumbel noise.

### Loss Function

Geometric cross-entropy. After D=20 comparison steps:
- Take `tau_final[:, 10:12]` (H3 pair)
- Compute dot product with 4 fixed class anchor vectors: `anchor_k = (-sin(πk/2), cos(πk/2))`
- Cross-entropy with GT class label

No additional learnable head. Readout is fixed geometry.

### Training Setup

| Parameter | Value |
|-----------|-------|
| N_TRAIN_STEPS | 500 |
| BATCH_SIZE | 128 |
| Optimizer | Adam |
| LR | 0.01 |
| Temp annealing | 2.0 → 0.1 |
| Training type | Shared (input-conditioned via token) |
| Checkpoints | [0, 100, 250, 500] |

### Alignment Success

`h3_class(tau_final) == h3_gt`, where h3_class is the argmax over cosine similarities
of `tau[:,10:12]` to the 4 geometric class anchor vectors. No modular arithmetic.

---

## 2. Geometry Lock

State representation: `tau_ang` — 12-dimensional angular vector.
- Block 3 (indices 6–11): H1@[6,7], H2@[8,9], H3@[10,11]
- H3 encodes class identity with period 4 = VOCAB
- Class anchors: `anchor_k = (-sin(πk/2), cos(πk/2))` for k ∈ {0,1,2,3}
- d_ang=12, n_pairs=6
- Structural norm (Unit angular state): √6 ≈ 2.449490

---

## 3. Baseline vs Trained Comparison (at t=20)

| Case | full_r mean | full_r std | subspace_r mean | subspace_r std | ratio mean | ratio std | dir mean | dir std |
|------|-----------|-----------|---------------|-------------|----------|---------|---------|--------|
| Stripped Baseline (no MLP) | 2.44949 | 0.0 | 1.0 | 0.0 | 0.408248 | 0.0 | -0.017578 | 0.689368 |
| B: Untrained matched | 1.349048 | 0.169212 | 0.37439 | 0.196285 | 0.274197 | 0.133929 | 0.00556 | 0.721701 |
| B: Untrained mismatched | 1.344855 | 0.175886 | 0.390255 | 0.199567 | 0.288019 | 0.137872 | 0.05757 | 0.722777 |
| C: Trained matched | 2.268898 | 0.283719 | 0.897376 | 0.201217 | 0.391723 | 0.074842 | 0.031056 | 0.718445 |
| C: Trained mismatched | 2.277479 | 0.277175 | 0.895468 | 0.20047 | 0.389051 | 0.068231 | -0.004477 | 0.718604 |


H3 class recovery rates:
- CASE A (stripped, no MLP): **0.2285**
- CASE B untrained matched:  **0.2637**
- CASE B untrained mismatch: **0.2891**
- CASE C trained matched:    **0.2773**
- CASE C trained mismatch:   **0.2520**

---

## 4. Training Progression Analysis

| Training Step | full_r mean | full_r std | ratio mean | dir mean | dir std | success_rate |
|--------------|-----------|-----------|----------|---------|---------|-------------|
| 0 | 1.284212 | 0.166571 | 0.282001 | -0.002288 | 0.714568 | 0.232422 |
| 100 | 1.290114 | 0.172238 | 0.263256 | 0.030012 | 0.698817 | 0.246094 |
| 250 | 1.368484 | 0.184496 | 0.276359 | -0.065996 | 0.698699 | 0.212891 |
| 500 | 2.268898 | 0.283719 | 0.391723 | 0.031056 | 0.718445 | 0.277344 |


---

## 5. Matched vs Mismatched Analysis

| Condition | full_r mean | full_r std | ratio mean | dir mean | acc |
|-----------|-----------|-----------|----------|---------|-----|
| B Untrained matched | 1.349048 | 0.169212 | 0.274197 | 0.00556 | 0.2637 |
| B Untrained mismatched | 1.344855 | 0.175886 | 0.288019 | 0.05757 | 0.2891 |
| C Trained matched | 2.268898 | 0.283719 | 0.391723 | 0.031056 | 0.2773 |
| C Trained mismatched | 2.277479 | 0.277175 | 0.389051 | -0.004477 | 0.2520 |


Matched vs mismatched gap (trained):
- full_r:   2.268898 vs 2.277479  Δ=-0.008582
- dir_mean: 0.031056 vs -0.004477  Δ=+0.035533
- acc:      0.2773 vs 0.2520   Δ=+0.0254

---

## 6. Phase Evolution of Radial and Directional Signals

| Timestep | A full_r_std | B_untrain full_r_std | C_trained full_r_std | A dir_mean | B_untrain dir_mean | C_trained dir_mean |
|---------|-------------|---------------------|--------------------|----------|------------------|------------------|
|  0 | 0.0 | 0.0 | 0.0 | 1.0 | 1.0 | 1.0 |
|  4 | 0.0 | 0.167864 | 0.296758 | 0.005859 | 0.057244 | 0.038631 |
|  8 | 0.0 | 0.155944 | 0.289436 | -0.015625 | -0.029344 | -0.051026 |
| 12 | 0.0 | 0.167649 | 0.291479 | 0.003906 | -0.02801 | 0.048139 |
| 16 | 0.0 | 0.156925 | 0.283914 | -0.019531 | -0.045678 | 0.094504 |
| 20 | 0.0 | 0.169212 | 0.283719 | -0.017578 | 0.00556 | 0.031056 |


Mean full_r_std across all timesteps:
- CASE A (stripped):   0.000000  (trivially ≈0: deterministic constant norm)
- CASE B (untrained):  0.159451
- CASE C (trained):    0.265234
- Trained/Untrained std ratio: 1.663  (honest comparison baseline)
- Trained/Stripped std ratio:  265234285.714  (uninformative — A is trivially 0)

Direction_metric variance across timesteps:
- CASE A: 0.224862
- CASE B: 0.215610
- CASE C: 0.212514
- Trained/Untrained dir_var ratio: 0.986

---

## 7. Do Radial/Directional Signals Emerge Only With Training?

**Evaluation criteria:**

| Criterion | Value | Pass? |
|-----------|-------|-------|
| acc_trained_matched > acc_stripped + 0.05 | +0.0488 | NO |
| acc_trained_matched > acc_untrained_matched + 0.05 | +0.0137 | NO |
| acc_trained_matched > acc_trained_mismatched + 0.05 | +0.0254 | NO |
| radial_std_ratio (trained/UNTRAINED) > 1.5 | 1.663 | YES |
| dir_var_ratio (trained/UNTRAINED) > 1.5 | 0.986 | NO |

Note: radial/dir ratio comparisons use UNTRAINED (Case B) as denominator — not stripped (Case A).
Stripped has std≈0 trivially (deterministic constant-norm system); comparing to it is uninformative.

**Strong criteria met**: 1/4
**Weak criteria met**: 2/3

**Assessment:**

Radial variation (full_r_std) increased by 1.66x vs untrained — training does affect the radial structure of the comparison basis. However, accuracy improvement over untrained is marginal (+0.0137), not exceeding the 0.05 threshold. This weakens any claim that trained signals are predictive. Matched vs mismatched discrimination is also weak (+0.0254). The module does not reliably distinguish correct from incorrect input pairings. Overall: training introduces some radial structure change but does not produce clearly predictive or input-discriminating signals at this training budget and configuration. Result is WEAK_SUPPORT — not sufficient to confirm the dynamic comparison-language hypothesis.

**Note on CASE A**: The stripped baseline (no MLP) operates deterministically via hard
angular-similarity routing. Its full_r_std across timesteps is approximately
0.000000, characteristic of a static system. The comparison module's ability
(or failure) to exceed this variability determines whether training introduces
genuine dynamic comparison.

**Note on prior branch error**: The previous branch (dynamic_decoding_language_probe_v1)
used FIXED RANDOM weights and called it "minimal comparison." This was incorrect:
untrained soft-blending ≠ input-conditioned dynamic comparison formation.
This branch tests whether TRAINING is required. The prior branch's B_rc case is
replicated here as "B_untrained" to establish the proper control.

---

MINIMAL TRAINED COMPARISON STATUS: WEAK_SUPPORT
