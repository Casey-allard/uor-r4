# Prime Transport Runtime-First Geometric Refactor v1

**Branch:** runtime_first_geometric_refactor_v1  
**Date:** 2026-04-09T05:50:04Z  
**Execution policy:** 8 workers × 1 thread(s)

## Current Fusion Summary

```
RouterLockedStack._soft_step():
  embs   = W_emb[tokens_t]                    # token embedding
  ctrl   = cat([embs, proj(tau_prev), mags])  # policy input
  h      = tanh(ctrl @ W1 + b1)               # hidden
  logits = h @ W2 + b2                        # policy logits
  w      = gumbel_softmax(logits / temp)      # op weights
  base   = einsum(w, tn)                      # transport direction
  hybrid = apply_split_transport(base, ...)   # new state

  Problem: policy training (W1/W2/W_emb gradients) is the SOLE source
  of transport direction differentiation. Runtime trajectory generation
  is fused to the learned policy path.
```

## Refactor Summary

```
RouterGeometricStack._soft_step():
  cur_dir = tau_prev[:, :d_ang]               # current angular direction
  ang_sim = einsum(cur_dir, tn)               # geometric similarity
  w       = gumbel_softmax(ang_sim / temp)    # op weights from geometry
  base    = einsum(w, tn)                     # transport direction
  hybrid  = apply_split_transport(base, ...)  # new state

  W1, W2, W_emb: NOT DEFINED in this model.
  Trajectory is fully determined by geometry at every step.
  Only W_attn, W_pred, and positional biases are trainable.
```

## Parameter Count

- current_fused_path D=20: 908 trainable parameters
- runtime_first_geometric D=20: 214 trainable parameters

## Before/After Runtime Table

| config | variant | D | runtime_s |
|--------|---------|---|----------|
| current_fused_path | original_s42 | 20 | 117.4 |
| runtime_first_geometric | original_s42 | 20 | 53.7 |
| current_fused_path | original_s42 | 32 | 170.8 |
| runtime_first_geometric | original_s42 | 32 | 79.2 |
| current_fused_path | shift1_s42 | 20 | 118.0 |
| runtime_first_geometric | shift1_s42 | 20 | 52.3 |
| current_fused_path | shift1_s42 | 32 | 171.0 |
| runtime_first_geometric | shift1_s42 | 32 | 78.8 |

| metric | value |
|--------|-------|
| avg current_fused_path runtime | 144.3s |
| avg runtime_first_geometric runtime | 66.0s |
| runtime reduction | 54.3% |

## Before/After Accuracy Table

| config | variant | D | final_acc | solve_step | decodability_final |
|--------|---------|---|-----------|------------|-------------------|
| current_fused_path | original_s42 | 20 | 0.6748 | 500 | 1.0000 |
| runtime_first_geometric | original_s42 | 20 | 0.9980 | ? | 1.0000 |
| current_fused_path | original_s42 | 32 | 0.9814 | ? | 1.0000 |
| runtime_first_geometric | original_s42 | 32 | 0.9927 | ? | 1.0000 |
| current_fused_path | shift1_s42 | 20 | 0.9756 | ? | 1.0000 |
| runtime_first_geometric | shift1_s42 | 20 | 0.9995 | 3500 | 1.0000 |
| current_fused_path | shift1_s42 | 32 | 0.9932 | ? | 1.0000 |
| runtime_first_geometric | shift1_s42 | 32 | 0.9902 | ? | 1.0000 |

| metric | value |
|--------|-------|
| avg accuracy delta (geometric - fused) | +0.0888 |
| avg decodability delta | +0.0000 |

## Honesty Section

### What was removed from trajectory generation

- W_emb: token embedding lookup — removed entirely
- W1/b1: first policy layer — removed entirely
- W2/b2: second policy layer / op logit projection — removed entirely
- The entire ctrl input construction path was removed
- tokens_t is passed to _soft_step for interface compatibility but is NOT used

### What remained trainable

- W_attn, b_attn, v_attn: attention readout over the trajectory
- W_pred, b_pred: output projection to VOCAB
- b_pos0, b_posLast: positional attention biases
- The trajectory itself has no trainable parameters

### Whether the old training-time policy path was actually unnecessary

YES — runtime_first_geometric achieves functionally equivalent accuracy (Δ=+0.0888) with 54.3% less wall-clock time. The W1/W2/W_emb policy path was not necessary for the measured behavior. The readout (W_attn/W_pred) can be trained against a geometrically-determined trajectory and achieve the same task performance.

## One-Line Conclusion

**RUNTIME-FIRST GEOMETRIC REFACTOR STATUS: VIABLE**
