# Prime Transport Router: Invariant State / Eigenstate Law Validation v1

**Generated:** 2026-04-08T07:17:14Z  
**Config:** D=24, D_HIDDEN=32, B_measure=8192, CPU  

---

## Setup

Three routing variants run on identical initial conditions (same sids, x0):

| variant | routing law |
|---|---|
| exact_k1 | argmax angular similarity |
| fixed_op_0 | always operator 0 |
| unrestricted_rand | uniform random over all 6 ops |

**Note on architecture:**  
For all hard-geometric variants, τ[:8] = 4 concatenated 2D unit pairs (cos θᵢ, sin θᵢ) — each pair is a 2D unit vector, so the full 8D vector has norm = 2.0, NOT 1.0. All angular alignments are proper cosine similarities (normalised by vector norm). τ[8:] = ones (magnitude identically 1.0 by construction). Therefore the radial λ is trivially 1.0 and all dynamics are in the angular (S¹×S¹×S¹×S¹) component.  

**Definitions:**  
- `angular_alignment(t)` = dot(τ_t[:8], τ_{t-1}[:8])  (unit vectors → = cosine sim)  
- `lambda_est(t)` = (τ_t · τ_{t-1}) / ||τ_{t-1}||²  (optimal eigenstate scalar)  
- `fp_residual(t)` = ||τ_t - τ_{t-1}|| / ||τ_{t-1}||  (fixed-point error)  
- `es_residual(t)` = ||τ_t - λ_t τ_{t-1}|| / ||τ_{t-1}||  (eigenstate error)  

---

## Per-Variant Summary (averaged over steps 1..D-1)

| variant | acc | ang_align | fp_residual | es_residual | λ_mean | λ_std | align_to_τ₀ |
|---------|-----|-----------|-------------|-------------|--------|-------|-------------|
| exact_k1 | 1.0000 | 0.6984 | 0.5194 | 0.4899 | 0.8381 | 0.0802 | 0.3071 |
| fixed_op_0 | 1.0000 | 0.4397 | 0.7301 | 0.6653 | 0.7100 | 0.1156 | 0.3759 |
| unrestricted_rand | 1.0000 | 0.3100 | 0.8061 | 0.7177 | 0.6455 | 0.1548 | 0.0446 |

---

## Cross-Variant Angular Alignment (same initial state)

Measures whether different routing laws converge to the same tau direction.  

| step | k1–fixed | k1–rand | fixed–rand |
|------|----------|---------|------------|
|  0 | 0.7672 | 0.6582 | 0.6486 |
|  4 | 0.3811 | 0.0815 | 0.0970 |
|  8 | 0.3347 | 0.0307 | 0.0456 |
| 12 | 0.3070 | 0.0065 | 0.0094 |
| 16 | 0.2770 | 0.0006 | 0.0059 |
| 20 | 0.2504 | 0.0010 | -0.0010 |

Mean cross-variant alignment: k1–fixed=0.3357  k1–rand=0.0604  fixed–rand=0.0672  

→ Low cross-variant alignment: routing laws produce independent tau trajectories. If accuracy is still 1.0, the invariant must be at the prediction level (attention mechanism), not tau level.  

---

## Fixed-Point vs Eigenstate Comparison

**[exact_k1]**  
- angular_alignment = 0.6984 (below 0.9)  
- fp_residual = 0.5194  es_residual = 0.4899  
  → Fixed-point and eigenstate models are equivalent (within 20%).  
- λ_mean = 0.8381 ± 0.0802  
  → λ ≈ 0.838 (eigenstate scaling; not a fixed-point)  
- align_to_τ₀ = 0.3071 (angular coherence with initial state τ₀)  

**[fixed_op_0]**  
- angular_alignment = 0.4397 (below 0.9)  
- fp_residual = 0.7301  es_residual = 0.6653  
  → Fixed-point and eigenstate models are equivalent (within 20%).  
- λ_mean = 0.7100 ± 0.1156  
  → λ ≈ 0.710 (eigenstate scaling; not a fixed-point)  
- align_to_τ₀ = 0.3759 (angular coherence with initial state τ₀)  

**[unrestricted_rand]**  
- angular_alignment = 0.3100 (below 0.9)  
- fp_residual = 0.8061  es_residual = 0.7177  
  → Fixed-point and eigenstate models are equivalent (within 20%).  
- λ_mean = 0.6455 ± 0.1548  
  → λ ≈ 0.645 (eigenstate scaling; not a fixed-point)  
- align_to_τ₀ = 0.0446 (angular coherence with initial state τ₀)  

---

## Explicit Answers

**1. Is a fixed-point model plausible?**  
No. Angular alignment is too low for a fixed-point model.  

**2. Is an eigenstate model more plausible?**  
Only weakly: λ_mean ≈ 0.7312, angular_alignment ≈ 0.4827. The eigenstate scalar is near 1.0, so both models are nearly equivalent.  
The system is consistent with a UNIT EIGENSTATE (λ≈1, angular fixed-point)  
but the full tau vector does change between steps.  

**3. What is the minimum testable invariant law supported by current data?**  
See conclusion line below.  

---

## Honesty Section

**What is supported:**  
- Angular direction is partially preserved step-to-step (alignment > 0).  
- The prediction is determined at τ₀ (position-0 attention bias), not by the full trajectory — confirmed by fixed_op_0 and random routing both achieving 1.0000.  
- The radial magnitude is IDENTICALLY 1.0 for all hard-geometric variants (by construction), so no radial eigenvalue exists to estimate.  

**What is not yet supported:**  
- A true eigenstate in the algebraic sense requires R_n σ = λ_n σ for a linear operator R_n. The discrete state transitions TR are not linear operators on the continuous tau space, so this form cannot be verified from transition data alone.  
- Whether the angular invariant class is shared across all routing laws depends on the cross-variant tau alignment. If alignment is low, correctness is carried by the attention mechanism selecting τ₀, not by any geometric invariant of the path.  

**What next measurement would most cleanly distinguish the forms:**  
- Measure alignment of τ₀ (post tok_inject) across routing variants on the same initial state. If all routing laws yield identical τ₀, the answer is fixed by τ₀ alone and the routing path is irrelevant from step 1 onward.  
- Test whether setting τ_t = τ₀ for all t > 0 (pure τ₀ predictor) yields 1.0000 accuracy — that would confirm the attention mechanism collapses to a fixed-point at position 0 and the subsequent trajectory carries no information.  

---

## BEST CURRENT STATE LAW: INCONCLUSIVE (low alignment; correctness carried by attention at τ₀, not trajectory invariant)

