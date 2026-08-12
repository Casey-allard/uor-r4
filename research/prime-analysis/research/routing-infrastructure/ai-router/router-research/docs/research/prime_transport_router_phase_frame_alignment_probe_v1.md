# Prime Transport Router: Phase / Frame Alignment Probe v1

**Generated:** 2026-04-08T07:56:44Z  
**Config:** D=24, D_HIDDEN=32, B_train=512, N_eval=2048  

**Baseline train acc:** 1.0000  

## Context

The delayed-injection regime (inject@last) breaks τ₀-sufficiency but fails to train.
This probe tests whether the injected signal, propagated state, and readout
share a geometrically consistent frame.

**T^4 manifold definition:**  
The angular part of τ_hyb is composed of 4 phase pairs: (cos θ_k, sin θ_k).
On T^4: all phase pair norms = 1.0.  
Off T^4: at least one pair norm ≠ 1.0.  

All measurements below use the TRAINED BASELINE model (inject@first, SGD/LR=0.02, 3500 steps) unless noted.  

---

## A. Manifold Membership (Phase Pair Norms)

| component | mean_norm | std_norm | mean_deviation | on_T4_manifold |
|-----------|-----------|----------|----------------|----------------|
| tau0_table (pool) | 1.0000 | 0.0000 | 0.0000 | YES |
| TN_ang (all entries) | 1.0000 | 0.0000 | 0.0000 | YES |
| hard_geom τ at t=0 | 1.0000 | 0.0000 | 0.0000 | YES |
| hard_geom τ at t=D-1 | 1.0000 | 0.0000 | 0.0000 | YES |
| W_tok_inject (4 vecs) | 0.7915 | 0.4746 | 0.4533 | NO |

> **Interpretation:** If W_tok_inject is OFF T^4 (mean_deviation >> 0), it adds signal orthogonal to the manifold, potentially in a frame the readout was not trained on.  

---

## B. Background Variance vs Injection SNR at Each Position

| position | bg_std | inj_sep | snr_proxy | cross_inj_acc |
|----------|--------|---------|-----------|---------------|
| t= 0 | 1.0968 | 3.9029 | 3.5584 | 1.0000 |
| t= 6 | 1.0980 | 3.9029 | 3.5546 | 1.0000 |
| t=12 | 1.0992 | 3.9029 | 3.5508 | 1.0000 |
| t=18 | 1.0983 | 3.9029 | 3.5536 | 1.0000 |
| t=23 | 1.0946 | 3.9029 | 3.5657 | 1.0000 |

> **bg_std:** std of τ_t across all eval samples (routing background noise).  
> **inj_sep:** mean L2 distance between W_tok_inject[a] and W_tok_inject[b] for a≠b.  
> **snr_proxy:** inj_sep / bg_std — signal strength relative to background.  
> **cross_inj_acc:** accuracy when injecting at position p and forcing attention to p (using trained baseline W_pred).  

---

## C. Angular Drift Across Steps

| step | cos_sim_to_t0 | step_cos_sim | mean_pair_norm | std_pair_norm |
|------|---------------|--------------|----------------|---------------|
|  0 | 1.0000 | 1.0000 | 1.0000 | 0.0000 |
|  1 | 0.7030 | 0.7030 | 1.0000 | 0.0000 |
|  2 | 0.5709 | 0.7031 | 1.0000 | 0.0000 |
|  3 | 0.5387 | 0.6970 | 1.0000 | 0.0000 |
|  4 | 0.5016 | 0.7101 | 1.0000 | 0.0000 |
|  5 | 0.4622 | 0.7058 | 1.0000 | 0.0000 |
|  6 | 0.4760 | 0.7064 | 1.0000 | 0.0000 |
|  7 | 0.4362 | 0.6993 | 1.0000 | 0.0000 |
|  8 | 0.4145 | 0.6997 | 1.0000 | 0.0000 |
|  9 | 0.4319 | 0.7055 | 1.0000 | 0.0000 |
| 10 | 0.3970 | 0.7037 | 1.0000 | 0.0000 |
| 11 | 0.3769 | 0.7051 | 1.0000 | 0.0000 |
| 12 | 0.3872 | 0.7088 | 1.0000 | 0.0000 |
| 13 | 0.3652 | 0.7106 | 1.0000 | 0.0000 |
| 14 | 0.3453 | 0.7070 | 1.0000 | 0.0000 |
| 15 | 0.3708 | 0.7117 | 1.0000 | 0.0000 |
| 16 | 0.3540 | 0.7054 | 1.0000 | 0.0000 |
| 17 | 0.3411 | 0.7091 | 1.0000 | 0.0000 |
| 18 | 0.3517 | 0.7146 | 1.0000 | 0.0000 |
| 19 | 0.3384 | 0.7098 | 1.0000 | 0.0000 |
| 20 | 0.3089 | 0.7062 | 1.0000 | 0.0000 |
| 21 | 0.3253 | 0.7115 | 1.0000 | 0.0000 |
| 22 | 0.3118 | 0.7057 | 1.0000 | 0.0000 |
| 23 | 0.2972 | 0.7109 | 1.0000 | 0.0000 |

> **cos_sim_to_t0:** angular alignment between τ_t and τ_0 (1.0 = same direction).  
> **step_cos_sim:** angular alignment between τ_t and τ_{t-1} (local continuity).  
> **mean_pair_norm:** mean ||phase_pair_k|| at step t (1.0 = on T^4 manifold).  

---

## D. Radial Consistency Across Steps

| step | mean_radial | std_radial | min_radial | max_radial |
|------|-------------|------------|------------|------------|
|  0 | 1.9839 | 0.0671 | 1.2500 | 2.0000 |
|  1 | 1.9857 | 0.0622 | 1.2500 | 2.0000 |
|  2 | 1.9848 | 0.0608 | 1.5000 | 2.0000 |
|  3 | 1.9850 | 0.0628 | 1.4045 | 2.0000 |
|  4 | 1.9876 | 0.0584 | 1.2500 | 2.0000 |
|  5 | 1.9850 | 0.0650 | 1.5000 | 2.0000 |
|  6 | 1.9851 | 0.0664 | 1.2500 | 2.0000 |
|  7 | 1.9869 | 0.0594 | 1.5000 | 2.0000 |
|  8 | 1.9888 | 0.0504 | 1.5000 | 2.0000 |
|  9 | 1.9875 | 0.0603 | 1.5000 | 2.0000 |
| 10 | 1.9864 | 0.0574 | 1.5000 | 2.0000 |
| 11 | 1.9875 | 0.0570 | 1.5000 | 2.0000 |
| 12 | 1.9880 | 0.0596 | 1.2500 | 2.0000 |
| 13 | 1.9891 | 0.0569 | 1.2500 | 2.0000 |
| 14 | 1.9896 | 0.0521 | 1.5000 | 2.0000 |
| 15 | 1.9870 | 0.0598 | 1.2500 | 2.0000 |
| 16 | 1.9890 | 0.0543 | 1.2500 | 2.0000 |
| 17 | 1.9878 | 0.0587 | 1.2500 | 2.0000 |
| 18 | 1.9903 | 0.0505 | 1.5000 | 2.0000 |
| 19 | 1.9883 | 0.0597 | 1.2500 | 2.0000 |
| 20 | 1.9888 | 0.0604 | 1.2500 | 2.0000 |
| 21 | 1.9907 | 0.0520 | 1.5000 | 2.0000 |
| 22 | 1.9912 | 0.0474 | 1.5000 | 2.0000 |
| 23 | 1.9900 | 0.0509 | 1.5000 | 2.0000 |

> Dynamic radial = ang_sim confidence, range [0,2]. Consistent radial across steps indicates stable routing geometry.  

---

## E. W_tok_inject vs TN Angular Alignment

| x0 | max_TN_cos | mean_TN_cos | inject_norm | pair_norms |
|----|------------|-------------|-------------|------------|
| 0 | 0.5592 | -0.0002 | 2.4074 | [0.2121, 0.6967, 1.5183, 0.7338] |
| 1 | 0.7364 | 0.0000 | 2.3752 | [0.3273, 0.4625, 0.8438, 0.9014] |
| 2 | 0.4593 | -0.0001 | 2.3642 | [0.4443, 0.5727, 1.6525, 0.5355] |
| 3 | 0.7693 | 0.0002 | 2.4311 | [0.2473, 0.7286, 1.108, 1.6796] |

> **max_TN_cos:** max cosine similarity between W_tok_inject[x0][:8] and any TN entry.  
> **pair_norms:** norm of each phase pair in W_tok_inject[x0] (1.0 = on T^4; other = off-manifold injection).  

---

## F. Injection Signal Separability

| pair | l2_dist | cos_sim | angle_deg |
|------|---------|---------|----------|
| x0=0-1 | 3.8144 | -0.2722 | 105.8° |
| x0=0-2 | 3.9656 | -0.3814 | 112.4° |
| x0=0-3 | 3.9305 | -0.3198 | 108.7° |
| x0=1-2 | 3.9160 | -0.3654 | 111.4° |
| x0=1-3 | 4.0169 | -0.3969 | 113.4° |
| x0=2-3 | 3.7739 | -0.2386 | 103.8° |

**Mean L2 distance:** 3.9029  
**Mean angle:** 109.2°  

> Large distances / large angles → class vectors are separable → W_pred can learn to discriminate.  

---

## Synthesis: Injection / Propagation / Readout Frame Analysis

Cross-injection accuracy is **1.0000 at every tested position** (t=0, 6, 12, 18, 23) using the trained baseline W_pred. Background variance (bg_std ≈ 1.097) and SNR (≈ 3.56) are flat across all positions — routing does not accumulate noise. The decoder trained on inject@first can decode injection placed at any position, including inject@last, with perfect accuracy. The frame is geometrically consistent across positions. The delayed-injection training failure is **not** a frame-consistency issue, **not** an SNR issue, and **not** a manifold issue. It is a cold-start bootstrapping problem: in the restored regime, no positional prior focuses attention on τ_{D-1} during early training, so W_tok_inject receives only α_{D-1} ≈ 1/24 of the gradient signal. The minimal fix is a trainable positional bias at t=D-1 (analogous to b_pos0), not a frame redesign.

---

## Honesty Section

**What lines up:**  
- TN_ang and tau0_table are by construction on T^4 (angular part = unit phase pairs).  
- Hard geom routing preserves T^4 membership (TN unit vectors → output on T^4).  
- The coordinate convention (PHASE_BLOCKS-derived angles) is shared by all table entries.  

**What drifts:**  
- Angular direction drifts step-to-step (cos_sim_to_t0 falls from 1.0 → 0.30 over 24 steps); local step-to-step cos_sim stabilises at ~0.71 after step 1.  
- W_tok_inject (random init or trained) is NOT constrained to T^4: phase pair norms vary wildly ([0.21, 0.70, 1.52, 0.73] for x0=0). Injection always adds an off-manifold component to τ.  
- **What does NOT drift:** background variance is essentially flat across all positions (bg_std ≈ 1.097 at t=0 and t=D-1, −0.2% change). SNR is flat. The routing does not accumulate noise.

**Whether the delayed-injection failure is frame-consistency or training-budget:**  
- SNR at t=0: 3.5584  →  SNR at t=D-1: 3.5657  (change: +0.2% — negligible)  
- Background std: 1.0968 at t=0 → 1.0946 at t=D-1 — routing does NOT degrade SNR.  
- Cross-injection accuracy: 1.0000 at every tested position (t=0, 6, 12, 18, 23).  
- **Conclusion: frame mismatch is RULED OUT as a cause of training failure.**  
- The decoder (W_pred trained on inject@first) decodes injection at ANY position perfectly, including t=D-1.  
- **The actual cause is a cold-start bootstrapping problem:**  
  In the baseline, b_pos0=2.0 creates α₀ ≈ 0.25 from step 1 → W_tok_inject gets meaningful gradient immediately → W_pred and W_tok_inject co-evolve.  
  In the restored regime, b_pos0=0.0 and there is no b_posLast bias → α_{D-1} ≈ 1/24 ≈ 0.042 from step 1 → W_tok_inject gradient is diluted by ~6× → neither W_pred nor W_tok_inject can bootstrap the other.  
  A positional bias at t=D-1 (equivalent to b_pos0 for the last position) would be the minimal fix to break the bootstrapping deadlock without changing the frame.  

---

## PHASE / FRAME ALIGNMENT IS: CONSISTENT

