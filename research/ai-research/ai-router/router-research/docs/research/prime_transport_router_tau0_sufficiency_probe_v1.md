# Prime Transport Router: τ₀ Sufficiency Probe v1

**Generated:** 2026-04-08T07:24:57Z  
**Config:** D=24, D_HIDDEN=32, B=2048, CPU  
**Trained b_pos0:** 4.9651  

---

## Hard Geometric Routing (exact_k1) — Readout Ablations

| variant | accuracy | Δ_vs_ref | alpha₀ | runtime_s | ablated_component |
|---------|----------|----------|--------|-----------|-------------------|
| reference | 1.0000 | +0.0000 | 0.8620 | 0.003 | none (full trajectory) |
| tau0_direct | 1.0000 | +0.0000 | 1.0000 | 0.000 | attention bypassed; pred=τ₀@W_pred+b_pred |
| tau0_repeated | 1.0000 | +0.0000 | 0.8617 | 0.003 | τ_t (t>0) replaced with τ₀ |
| no_tau0 | 0.3062 | -0.6938 | 0.0000 | 0.003 | position 0 masked in attention (alpha[0]=0) |
| early_2 | 1.0000 | +0.0000 | 0.9931 | 0.003 | positions 2..D-1 masked (only pos 0,1 used) |
| late_half | 0.2832 | -0.7168 | 0.0000 | 0.003 | positions 0..11 masked (only pos 12..23 used) |
| random_t_gt_0 | 1.0000 | +0.0000 | 0.8619 | 0.005 | τ_t (t>0) replaced with random tau table entries |

## Soft MLP Routing — Readout Ablations

| variant | accuracy | Δ_vs_ref | alpha₀ | runtime_s | ablated_component |
|---------|----------|----------|--------|-----------|-------------------|
| reference | 1.0000 | +0.0000 | 0.8620 | 0.003 | none (full trajectory) |
| tau0_direct | 1.0000 | +0.0000 | 1.0000 | 0.000 | attention bypassed; pred=τ₀@W_pred+b_pred |
| tau0_repeated | 1.0000 | +0.0000 | 0.8617 | 0.003 | τ_t (t>0) replaced with τ₀ |
| no_tau0 | 0.2500 | -0.7500 | 0.0000 | 0.003 | position 0 masked in attention (alpha[0]=0) |
| early_2 | 1.0000 | +0.0000 | 0.9931 | 0.003 | positions 2..D-1 masked (only pos 0,1 used) |
| late_half | 0.2588 | -0.7412 | 0.0000 | 0.003 | positions 0..11 masked (only pos 12..23 used) |
| random_t_gt_0 | 1.0000 | +0.0000 | 0.8627 | 0.003 | τ_t (t>0) replaced with random tau table entries |

---

## Explicit Answers

**1. Is τ₀ alone sufficient for 1.0000?**  
YES. tau0_direct (pred = τ₀ @ W_pred + b_pred, no attention) achieves 1.0000. The attention mechanism is not required; τ₀ alone predicts correctly.  

**2. If τ₀ is repeated across all positions, does performance stay perfect?**  
YES. tau0_repeated achieves 1.0000 (Δ=+0.0000). Filling all positions with τ₀ does not degrade prediction.  

**3. If τ₀ is removed, does accuracy collapse?**  
YES — complete collapse. no_tau0 accuracy = 0.3062 (chance ≈ 0.25). Without τ₀ the model cannot predict.  

**4. Are τ_t for t>0 contributing anything measurable at D=24?**  
NO measurable contribution. random_t_gt_0 achieves 1.0000 after replacing all t>0 taus with random values. τ₀ alone is sufficient; subsequent taus carry no necessary information.  

---

## Honesty Section

**What definitely matters:**  
- Ablating [no_tau0] → acc=0.3062 (Δ=-0.6938). This component is load-bearing.  
- Ablating [late_half] → acc=0.2832 (Δ=-0.7168). This component is load-bearing.  

**What definitely does not matter:**  
- [tau0_direct]: acc=1.0000 (no degradation). The ablated component is not load-bearing.  
- [tau0_repeated]: acc=1.0000 (no degradation). The ablated component is not load-bearing.  
- [early_2]: acc=1.0000 (no degradation). The ablated component is not load-bearing.  
- [random_t_gt_0]: acc=1.0000 (no degradation). The ablated component is not load-bearing.  

**What remains ambiguous:**  
- No moderate failures (0.95-0.99): all ablations are either harmless or catastrophic.  

---

## TAU0 ALONE IS: SUFFICIENT

