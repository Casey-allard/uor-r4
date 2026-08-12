# Prime Transport Policy Path Ablation Probe v1

**Branch:** policy_path_ablation_probe_v1  
**Date:** 2026-04-09T05:31:00Z  
**Canonical files:** runtime_bottleneck_audit_v1.csv, run_runtime_bottleneck_audit_v1.py

---

## One-Line Conclusion

**POLICY PATH ABLATION STATUS: STRUCTURALLY IMPOSSIBLE — TRUE NO-SUBSTITUTE ABLATION NOT ACHIEVABLE IN CURRENT ARCHITECTURE**

---

## Code-Path Lock Summary

### Training path — exact dependency chain for W1/W2/W_emb

```
_soft_step(tau_prev, state_ids, tokens_t, temp):

  embs   = W_emb[tokens_t]                        # (B, D_EMB=4)
  ctrl   = cat([embs, proj(tau_prev), mags])       # (B, d_ctrl=14)
  h      = tanh(ctrl @ W1 + b1)                   # (B, 32)
  logits = h @ W2 + b2                            # (B, N_OPS=6)
  w      = gumbel_softmax(logits / temp)          # (B, 6)
  base   = einsum("bi,bij->bj", w, tn)            # (B, d_ang)  ← TRANSPORT DIRECTION
  hybrid = apply_split_transport(base, tau_prev)  # (B, d_hyb)  ← NEW STATE
  new_sids = TR[state_ids].gather(argmax(w))      # (B,)

  return hybrid, new_sids
```

`forward()` calls `_soft_step()` D times. The output of each call (`hybrid`) is the
input to the next call (`tau_prev`). The full trajectory `soft_taus` feeds the readout.

### Eval path — W1/W2/W_emb not used

```
eval_acc() / run_decodability_final():
  ang_sim = einsum(cur_dir, tn)   # purely geometric, no policy network
  best_op = argmax(ang_sim)
  embs = W_emb[toks[:,t]]         # computed but never used (dead code in canonical probe)
  readout: W_attn, W_pred only
```

---

## Impossibility Finding

True no-substitute ablation of W1/W2/W_emb is **structurally impossible** in the
current `RouterLockedStack.forward()`.

### Exact structural reason

`w` — the output of `gumbel_softmax(logits)` where `logits = h @ W2 + b2` — is
the **sole and only input** to:

```python
base = einsum("bi,bij->bj", w, tn)
```

`base` is the sole input to `apply_split_transport(base, tau_prev, ...)`, which
produces `hybrid`. `hybrid` is the only state update produced by `_soft_step`.
Without `hybrid`, the `for t in range(D)` loop in `forward()` produces no trajectory.
Without a trajectory, there is no readout, no logits, no loss, no backward pass.

The policy computation (W1/W2/W_emb → logits → w) **is not separable** from
the transport computation (w → base → hybrid). They are fused in `_soft_step`.

### Why every attempted ablation introduced a substitute

| Attempt | What it did | Why it fails the necessity test |
|---------|-------------|--------------------------------|
| `logits = zeros(B, N_OPS)` | Uniform softmax → average transport direction | Introduces a new routing signal (uniform averaging) — not neutral |
| `logits = ang_sim.detach()` | Geometric angular similarity | Introduces a substitute routing signal (eval-consistent geometry) — still a controller |

Both attempts required producing *something* for `w`, because the forward structure
cannot proceed without it. Any value assigned to `w` is a substitute, not an absence.

### What a true no-substitute ablation would require

One of the following, all of which violate branch constraints:

1. **Remove the transport loop** — changes forward signature (architectural change, forbidden)
2. **Provide a bypass path** that allows `forward()` to run without `_soft_step` — architectural change (forbidden)
3. **Accept a substitute** — explicitly forbidden by the branch contract

---

## RNG Fix (applied regardless of ablation outcome)

The canonical probe contains `torch.rand_like(logits)` in `_soft_step`, which draws
from the global PyTorch RNG. This contaminates downstream runs in serial execution.

**Fix applied in this branch's model:**
```python
# __init__:
self._gumbel_gen = torch.Generator().manual_seed(seed + 9973)

# _soft_step (training):
# BEFORE: u = torch.rand_like(logits).clamp_(1e-20, 1.0)
# AFTER:  u = torch.rand(logits.shape, generator=self._gumbel_gen).clamp_(1e-20, 1.0)
```

This fix was applied to both `baseline_full` and any surrogate configs.
It is an independent correctness fix — not part of the ablation logic.

---

## Surrogate Run Data (REJECTED methodology — preserved for reference only)

Two surrogate approaches were attempted and run to completion before being rejected.
The data is preserved in the CSV under `config=surrogate_ang_sim_detach_REJECTED`.
It is **not a valid necessity test** and must not be interpreted as one.

Surrogate used: `logits = ang_sim.detach()` — geometric angular similarity, same shape,
detached from W1/W2/W_emb gradient path.

| config | variant | D | final_acc | rt_s | canon_acc |
|--------|---------|---|-----------|------|-----------|
| baseline_full | original_s42 | 20 | 0.6748 | 133.2 | 0.9956 |
| baseline_full | original_s42 | 32 | 0.9814 | 194.6 | 0.9395 |
| baseline_full | shift1_s42 | 20 | 0.9756 | 134.6 | 0.9990 |
| baseline_full | shift1_s42 | 32 | 0.9932 | 213.7 | 0.9727 |
| surrogate (REJECTED) | original_s42 | 20 | 0.9990 | 51.6 | 0.9956 |
| surrogate (REJECTED) | original_s42 | 32 | 0.9961 | 85.3 | 0.9395 |
| surrogate (REJECTED) | shift1_s42 | 20 | 0.9990 | 52.9 | 0.9990 |
| surrogate (REJECTED) | shift1_s42 | 32 | 0.9873 | 86.4 | 0.9727 |

These numbers show the surrogate converges well and runs ~2.4x faster. They do NOT
establish that W1/W2/W_emb are unnecessary — they only show that a particular
geometric substitute produces similar accuracy to the full policy.

---

## Honesty Section

### What was actually ablated
Nothing. No valid ablation was possible.

### What remained unchanged
The canonical model structure is unchanged. The RNG fix is the only modification
that should propagate forward.

### Whether the 98% runtime cost is actually removable via this branch
**No.** The 98% cost (forward+backward through the training loop) cannot be
removed by ablating W1/W2/W_emb while preserving forward semantics. The policy
path is fused with the transport mechanism.

### What a future branch could test
- **Freeze ablation** (`W1/W2/W_emb.requires_grad_(False)`): tests whether policy
  *learning* is necessary, not whether the policy path itself is necessary.
- **Architecture separation branch**: decouple policy selection from transport
  execution so either can be independently disabled. Requires a new model design.

---

## POLICY PATH ABLATION STATUS: STRUCTURALLY IMPOSSIBLE
