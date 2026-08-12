# prime_transport_router_reintegration_v6_torch_d24_diagnostic

**Branch:** bounded readout diagnostic  
**Date:** 2026-04-07  
**Surface:** canonical v6_torch, D=24, D_HIDDEN=32  
**Goal:** determine why alpha0 = 1/D=0.042 at D=24 despite 0.503 accuracy after 10K batches

---

## 1. Experiment Setup

- D=24, D_HIDDEN=32 (canonical), BATCH_SIZE=256, BUDGET=10,000
- Diagnostic batch B_DIAG=1024 for stable gradient estimates
- torch.set_num_threads(1) (canonical setting)
- Checkpoints: [0, 500, 1000, 2000, 5000, 10000]
- Diagnostic forward: unscripted (allows retain_grad on a_scores)
- Training forward: torch.jit.script (canonical speed)
- No architecture changes. No training extensions.

---

## 2. Checkpoint Summary

| ckpt | loss | acc | acc_unif | Δacc | a_pos_var | W_attn_g | W_inj_g | W_router_g | note |
|---|---|---|---|---|---|---|---|---|---|
|      0 | 1.395 | 0.238 | 0.238 | +0.000 | 0.0000 | 0.0001 | 0.0040 | 0.0033 | INIT |
|    500 | 1.385 | 0.277 | 0.277 | +0.000 | 0.0000 | 0.0000 | 0.0037 | 0.0010 | chance |
|   1000 | 1.387 | 0.245 | 0.246 | -0.001 | 0.0000 | 0.0000 | 0.0037 | 0.0018 | chance |
|   2000 | 1.386 | 0.249 | 0.249 | +0.000 | 0.0000 | 0.0001 | 0.0042 | 0.0014 | chance |
|   5000 | 1.381 | 0.343 | 0.343 | +0.000 | 0.0000 | 0.0003 | 0.0070 | 0.0033 | learning |
|  10000 | 1.355 | 0.443 | 0.445 | -0.002 | 0.0000 | 0.0016 | 0.0179 | 0.0091 | learning |

**Δacc = normal_accuracy − uniform_attention_accuracy**  
If Δacc ≈ 0 at a checkpoint, the model is NOT using attention for retrieval.

---

## 3. Per-Position Attention Analysis (final checkpoint = 10000)

| pos | attn_logit | attn_weight | attn_grad | tau_norm | tau_var | tau_x0_bv |
|---|---|---|---|---|---|---|
| 0 ← inject pos | -0.0002 | 0.0417 | 0.000031 | 1.7581 | 0.0833 | 0.056341 |
| 1 | -0.0006 | 0.0417 | 0.000004 | 1.4623 | 0.0399 | 0.000154 |
| 2 | -0.0006 | 0.0417 | 0.000004 | 1.4565 | 0.0377 | 0.000183 |
| 3 | -0.0006 | 0.0417 | 0.000004 | 1.4594 | 0.0376 | 0.000187 |
| 6 | -0.0008 | 0.0417 | 0.000004 | 1.4635 | 0.0352 | 0.000142 |
| 12 | -0.0008 | 0.0417 | 0.000004 | 1.4700 | 0.0328 | 0.000133 |
| 18 | -0.0010 | 0.0417 | 0.000004 | 1.4682 | 0.0299 | 0.000104 |
| 23 | -0.0010 | 0.0417 | 0.000004 | 1.4719 | 0.0286 | 0.000124 |

chance = 0.250. Expected alpha_weight if uniform = 0.0417.

---

## 4. Diagnostic Findings

### 4.1 Is attention symmetric?

**Attention logits remain nearly symmetric** (position variance = 0.0000 ≈ 0). The model has not learned any position preference.

Attention position variance evolution:
- Early (ckpt=500): 0.0000
- Final (ckpt=10000):  0.0000

### 4.2 Is the gradient to step-0 attention materially weaker?

**Gradient to position 0 is STRONGER than other positions, not weaker.**  
`grad_norm[0]=0.000031` vs. position mean `0.000005` — position 0 receives ~7.8× the
gradient signal of any other position. This is because tau_0 IS discriminative for x0
(see §4.3), so `∂loss/∂a_scores[:,0]` is larger.

BPTT dilution through the attention path is confirmed NOT to be the primary bottleneck.

Weight gradient norms at final checkpoint:
- W_attn:       0.0016  (tiny — attention parameters barely learning)
- W_tok_inject: 0.0179  
- W2 (router):  0.0091  (6× larger than W_attn — routing path learns faster)

### 4.3 Are tau states at position 0 discriminative for x0?

**Tau_0 is discriminative AND growing over training:**

| ckpt | tau_x0_bv (pos 0) | tau_x0_bv (pos 1) | note |
|---|---|---|---|
| 0 | 0.00255 | 0.00018 | random init — noise |
| 500 | 0.00253 | 0.00018 | still noise |
| 2000 | 0.00323 | 0.00019 | W_tok_inject starting to learn |
| 5000 | 0.00885 | 0.00014 | clear x0 signal emerging |
| 10000 | 0.05634 | 0.00015 | **22× stronger than any other position** |

W_tok_inject IS being trained (gradient 0.0179 at final ckpt). By 10K batches, tau_0 
contains a clear x0 signal — 22× the between-group variance of any other position.
The injection mechanism works. The bottleneck is NOT the injection.

**The bottleneck is that the attention scorer (W_attn) never learns to detect this
discriminative signal.** W_attn gradient = 0.0016 (6× smaller than the router).
The routing-based solution converges first and captures the gradient signal before
attention can develop position preference.

### 4.4 Is D=24 accuracy from indirect routing, not step-0 retrieval?

**Indirect heuristic confirmed:** normal accuracy ≈ uniform-attention accuracy (Δ=-0.002)
at every checkpoint from 0 through 10K. The model is solving D=24 through routing
trajectory statistics, not through step-0 attention retrieval. The attention mechanism
is bypassed completely.

**Why does the model bypass attention despite tau_0 being discriminative?**

Two solutions compete during training:

1. **Attention-based** (correct): attend to tau_0 → retrieve x0 directly.
   Requires W_attn to learn to distinguish tau_0 from the 23 other tau states.
   W_attn gradient is 0.0016 — extremely slow learning.

2. **Routing-based** (indirect): the injection at t=0 biases routing decisions at t=0,
   which propagates x0 information into subsequent tau states probabilistically.
   Uniform averaging over all 24 positions captures this diffuse signal.
   W2 (router) gradient is 0.0091 — 6× faster learning.

The routing-based solution converges faster and gets there first. Once the model is
using the routing path for accuracy, the attention-based gradient signal becomes
relatively weaker, and the model stays stuck in this basin.

The root problem is **symmetry at initialization**: all attention logits start near 0,
all positions have equal initial weight (1/24). There is no inductive bias pushing
the attention toward position 0. The model never breaks this symmetry because the
routing path offers a faster gradient descent path to reduced loss.

---

## 5. Root Cause Summary

At final checkpoint (ckpt=10000):
- Accuracy (normal attention):   0.443
- Accuracy (uniform attention):  0.445  ← nearly identical — attention not used
- Attention position variance:   0.0000 ← perfectly symmetric throughout all training
- Tau_0 x0-discriminability:     0.056341 ← **22× position mean** — injection signal IS present
- Position-0 attn gradient norm: 0.000031 ← 7.8× position mean — gradient IS reaching pos 0
- Mean attn gradient norm:       0.000005

**Root cause: initialization symmetry + competing faster routing path.**

tau_0 contains the x0 signal (between-group variance 22× mean). The gradient to
the attention scorer at position 0 is reaching it (7.8× stronger than other positions).
BUT the attention parameters (W_attn, v_attn) are learning 6× slower than the router
(W2), and the router finds a routing-based solution before the attention develops
any position preference. Once stuck in the routing-based basin, attention stays
symmetric because the router is already extracting the available signal.

**This is a symmetry-breaking / initialization problem, not a capacity or gradient
dilution problem.**

---

## 6. Honesty Section

- Were any core files modified? No
- Were any operators rebuilt? No
- Is full exact spin_H solved? No
- Were any architecture changes made? No
- Is this a training extension? No — same 10K budget as canonical

---

## 7. Next Step (exactly one)

**make one specific readout interface fix:** add an explicit position-0 bias to the attention scorer at initialization (a_scores[:, 0] += constant) to break the symmetry and give step-0 retrieval an inductive bias. This is a targeted single-parameter initialization change, not an architectural modification.
