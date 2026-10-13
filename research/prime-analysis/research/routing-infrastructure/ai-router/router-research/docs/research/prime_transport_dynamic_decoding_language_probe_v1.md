# Prime Transport — Dynamic Decoding Language Probe v1

**Branch:** `dynamic_decoding_language_probe_v1`

**CANONICAL SOURCES:**
- `results/prime_transport_recursive_system/radial_ratio_revalidation_probe_v1.csv`
- `results/prime_transport_recursive_system/train_free_geometric_probe_v1.csv`

**CONTRAST SOURCES (for historical comparison only):**
- `results/prime_transport_recursive_system/prime_transport_router_reintegration_v6_torch.csv`

---

## 1. Mechanism Lock Summary

### What was minimally reconstructed

The comparison loop from `RouterV6.forward()` (`run_router_reintegration_v6_torch.py`),
stripped to its bare minimum:

```
At each step t (of D total steps):
  emb    = W_emb[tok_t]                       # (B, D_EMB)  — token embedding
  h_in   = cat([emb, tau_prev], dim=1)        # (B, D_EMB + d_ang)
  h      = tanh(h_in @ W1 + b1)              # (B, D_HIDDEN)
  logits = h @ W2 + b2                        # (B, N_OPS)
  w      = softmax(logits / 0.05)             # (B, N_OPS)  — sharp, eval mode
  base   = einsum("bi,bij->bj", w, TN_ang)   # (B, d_ang)  — KEY: soft blend
  tau_{t+1} = base
```

This is the "active comparison-language formation" step: all 6 operator
outcomes are compared simultaneously and blended proportionally to the
router's confidence given the current state + token.  The result is a
**non-unit-norm** vector whose norm and direction are **input-specific**.

### What was NOT restored

- Training / backpropagation / optimizer
- Trajectory attention readout (W_attn, v_attn)
- Prediction head (W_pred, b_pred)
- Token injection separate from routing (W_tok_inject)
- Gumbel sampling noise (eval mode: sharp softmax)

### Weights

Fixed random initialization (INIT_SCALE=0.05, seed=42).
No training. Same scale as v6 but no parameter updates.

---

## 2. Cases Tested

| Case | Description |
|------|-------------|
| A_stripped_baseline | Deterministic hard routing + apply_split_transport(eps_high=1.0) |
| B_uniform_blend | Uniform soft blend over all 6 operators (no router MLP) |
| Brc_conditioned | Token-conditioned soft blend (random fixed MLP) |
| Brc_conditioned (matched) | Correct class token at every step |
| Brc_conditioned (mismatched) | Wrong class token (+1 mod VOCAB) at every step |
| D_success | Matched-condition samples where H3 class survives to step D |
| D_failure | Matched-condition samples where H3 class is lost by step D |
| D_success_mismatch | Mismatched-condition samples where H3 class survives |
| D_failure_mismatch | Mismatched-condition samples where H3 class is lost |

Structural prediction for stripped system:
- All (cos,sin) pairs are unit-norm → `||tau_ang||` = sqrt(6) ≈ 2.449490
- Radial ratios are structural constants — confirmed by radial_ratio_revalidation_probe_v1.

---

## 3. Stripped Baseline vs Dynamic Comparison Process (at step D=20)

| Case | full_r mean | full_r std | h3_r mean | h3_r std | ratio mean | ratio std | H3 cos mean | H3 cos std |
|------|------------|-----------|----------|---------|----------|----------|------------|------------|
| A: Stripped baseline | 2.44948983 | 0.0 | 1.0 | 0.0 | 0.40824836 | 9e-08 | 1.0 | 0.0 |
| B: Uniform soft blend | 1.26768935 | 0.16902687 | 0.35802117 | 0.18075177 | 0.28172666 | 0.13423595 | 0.06395026 | 0.71817017 |
| B_rc: Cond. matched | 1.33963156 | 0.16335802 | 0.36998871 | 0.1775232 | 0.27456674 | 0.12418946 | 0.01073849 | 0.69864851 |
| B_rc: Cond. mismatched | 1.34151721 | 0.17139219 | 0.36871549 | 0.17918253 | 0.27355444 | 0.12543429 | 0.0018656 | 0.70321012 |


**Key diagnostic:**
- `full_radius_std` for stripped baseline:    0.0  (expected ≈ 0)
- `full_radius_std` for uniform soft blend:   0.16902687
- `full_radius_std` for conditioned matched:  0.16335802
- `full_radius_std` for conditioned mismatch: 0.17139219

A non-zero std in the dynamic cases but not in the stripped case would confirm
that radial variation is produced by the comparison mechanism, not the geometry.

---

## 4. Matched vs Mismatched Conditions (Case B_rc at step D=20)

| Condition | full_r mean | full_r std | ratio mean | ratio std | H3 cos mean | H3 cos std |
|-----------|------------|-----------|----------|---------|------------|------------|
| Matched (correct class token) | 1.33963156 | 0.16335802 | 0.27456674 | 0.12418946 | 0.01073849 | 0.69864851 |
| Mismatched (wrong class token) | 1.34151721 | 0.17139219 | 0.27355444 | 0.12543429 | 0.0018656 | 0.70321012 |


---

## 5. Success vs Failure Trajectories (Case D, proxy = H3 class recovery)

| Condition | Subset | full_r mean | full_r std | ratio mean | H3 cos mean |
|-----------|--------|------------|-----------|----------|------------|
| Matched / H3 success | success | 1.3461591 | 0.17022438 | 0.26838604 | 0.89413327 |
| Matched / H3 failure | failure | 1.33749509 | 0.16104652 | 0.27658957 | -0.2783829 |
| Mismatched / H3 success | success | 1.32855439 | 0.17697938 | 0.28285959 | 0.89904368 |
| Mismatched / H3 failure | failure | 1.34580445 | 0.16934426 | 0.27047685 | -0.29486188 |


---

## 6. Do radial/directional quantities become informative only during active comparison-language formation?

### Test criteria

| Criterion | Required observation |
|-----------|---------------------|
| Input-specific radial variation | `full_radius_std` >> 0 in Cases B/B_rc; ≈ 0 in Case A |
| Phase-specific variation | `full_radius` changes across timesteps in Cases B/B_rc; constant in Case A |
| Match-condition discriminability | Matched vs mismatched differ in radius or H3 cosine |
| Predictive for success | Success subset has different radial signature than failure subset |

### Observed evidence

The stripped baseline (CASE A) has `full_radius_std` ≈ 0.0 at all timesteps,
confirming the structural constant property established in radial_ratio_revalidation_probe_v1.

The uniform soft blend (CASE B) produces `full_radius_std` = 0.16902687 at step D=20.
This variation arises because the mean of 6 unit-norm vectors is not unit-norm; the degree
of constructive/destructive interference varies by state.  This is **architectural**, not
dynamic: it is fully determined by which operators are available at each state, independent
of any input token.

The token-conditioned blend (CASE B_rc) with matched tokens produces std = 0.16335802
and with mismatched tokens produces std = 0.17139219.  The **difference** between
these two reflects genuine input-conditioning of the comparison process.

Whether this difference is large enough to constitute a discriminative signal, and whether
it predicts H3 class recovery, is the key measurement (see Tables 3 and 4 above).

---

## 7. Final Conclusion

### Evidence summary

| Property | CASE A | CASE B | CASE B_rc | Conclusion |
|----------|--------|--------|-----------|------------|
| Radial variation across samples | None (structural const) | Present (architectural) | Present (input+arch) | B/Brc vary; A does not |
| Variation across timesteps | None | Present | Present | B/Brc evolve; A does not |
| Matched vs mismatched differ | N/A | N/A | See Table 3 | Needs table values |
| Predicts class recovery | N/A | N/A | See Table 4 | Needs table values |

### Interpretation

The soft-blend comparison mechanism does produce radial variation that is absent
in the stripped system.  The question is whether this variation is:
(a) purely structural/architectural (same regardless of input class), or
(b) genuinely dynamic (depends on alignment between token and state).

If (a): WEAK_SUPPORT at best — the variation is real but not informative beyond
        the operator-diversity structure of the state graph.

If (b): possible STRONG_SUPPORT — but only if the matched/mismatched and
        success/failure tables show significant and consistent differences.

The H3 direction metric (cosine between tau_final[10:12] and tau_init[10:12])
is the primary diagnostic.  If it differs significantly between matched and
mismatched, and between success and failure, that constitutes evidence that
the comparison process is forming an input-specific "decoding language."


DYNAMIC DECODING LANGUAGE STATUS: NO_SUPPORT

*Reason: H3 cosine diff matched/mismatch=0.008873 (threshold 0.05), full_radius diff success/failure=0.008664 (threshold 0.05). Radial variation exists (vs stripped std=0) but is purely architectural — does not discriminate matched vs mismatched or predict class recovery.*


---

*Generated by `run_dynamic_decoding_language_probe_v1.py` in 7.7s.*
