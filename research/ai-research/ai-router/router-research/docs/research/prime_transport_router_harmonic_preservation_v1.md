# Prime Transport Router: Harmonic Preservation in Transport Dynamics v1

**Generated:** 2026-04-08T16:00:24Z  
**Config:** D=24, D_HIDDEN=32, B=512, N_eval=2048, steps=3500, LR=0.02, b_posLast_init=2.0  

**Task:** `target = mod12_initial_state % 4`  (state-specific x0)  

---

## Transport Variant Definitions

| variant | update rule | parameters |
|---------|-------------|------------|
| baseline | standard transport; no carry | eps_full=0.0, eps_high=0.0 |
| residual_0.1 | tau_{t+1} = 0.9*transport + 0.1*tau_t | eps_full=0.1, eps_high=0.0 |
| residual_0.25 | tau_{t+1} = 0.75*transport + 0.25*tau_t | eps_full=0.25, eps_high=0.0 |
| harmonic_split | k=1 full transport; k>=2 carry eps_high=0.5 | eps_full=0.0, eps_high=0.5 |

**Low-pass filter hypothesis:** The baseline transport computes a weighted average over neighbor states spanning all mod-12 classes. This averaging destroys high-frequency (k=3) class vectors while preserving low-frequency (k=1) structure — acting as a spatial low-pass filter on the harmonic basis.  

**Residual carry:** Adds a fraction ε of the previous tau to the transported result. The initial k=3 signal decays as ε^t at step t.  
  - ε=0.1:  initial k=3 signal at mid (t=11): 0.1^11 ≈ 1.00e-11  
  - ε=0.25: initial k=3 signal at mid (t=11): 0.25^11 ≈ 2.38e-07  

**Harmonic split:** k=1 pair updated by full transport; k>=2 pairs blended with previous tau (eps_high=0.5). Initial k=3 signal at mid (t=11): 0.5^11 ≈ 0.0005  

---

## Training and Task Results

| variant | regime | accuracy | solve_step | α_{D-1} | no_last | runtime_s |
|---------|--------|----------|------------|---------|---------|----------|
| baseline | reduced_k1 | 1.0000 | 2500 | 0.8745 | 0.2432 | 57.7 |
| baseline | fuller_k3 | 1.0000 | 2500 | 0.8572 | 0.2393 | 71.4 |
| residual_0.1 | reduced_k1 | 1.0000 | 2500 | 0.8749 | 0.2417 | 63.7 |
| residual_0.1 | fuller_k3 | 1.0000 | 2500 | 0.8587 | 0.2554 | 72.7 |
| residual_0.25 | reduced_k1 | 1.0000 | 2000 | 0.8754 | 0.2520 | 61.9 |
| residual_0.25 | fuller_k3 | 1.0000 | 3000 | 0.8594 | 0.2983 | 72.5 |
| harmonic_split | reduced_k1 | 1.0000 | 2500 | 0.8743 | 0.2407 | 58.1 |
| harmonic_split | fuller_k3 | 1.0000 | 2500 | 0.8590 | 0.3687 | 69.9 |

---

## Decodability Probes

Linear decodability of `mod12_class` from τ at three positions.  
**Baseline reference:** fuller_k3/initial=1.0000, mid=0.3076 (near chance), final=1.0000  

| variant | regime | initial | mid (t=11) | final (t=23) |
|---------|--------|---------|------------|-------------|
| baseline | reduced_k1 | 0.2937 | 0.2786 | 1.0000 |
| baseline | fuller_k3 | 1.0000 | 0.3076 | 1.0000 |
| residual_0.1 | reduced_k1 | 0.2937 | 0.2776 | 1.0000 |
| residual_0.1 | fuller_k3 | 1.0000 | 0.3123 | 1.0000 |
| residual_0.25 | reduced_k1 | 0.2937 | 0.2925 | 1.0000 |
| residual_0.25 | fuller_k3 | 1.0000 | 0.3179 | 1.0000 |
| harmonic_split | reduced_k1 | 0.2937 | 0.2786 | 1.0000 |
| harmonic_split | fuller_k3 | 1.0000 | 0.3367 | 1.0000 |

**Mid-trajectory improvement over baseline (fuller_k3 only):**  

| variant | mid_decodability | Δ_vs_baseline |
|---------|-----------------|---------------|
| baseline | 0.3076 | +0.0000 |
| residual_0.1 | 0.3123 | +0.0047 |
| residual_0.25 | 0.3179 | +0.0103 |
| harmonic_split | 0.3367 | +0.0291 |

---

## Key Questions

**Q1: Does any variant preserve k=3 above baseline at mid-trajectory?**  
Best mid-trajectory decodability (fuller_k3): 0.3367 from variant 'harmonic_split' (baseline: 0.3076), Δ=+0.0291.
All mid values remain near chance (chance = 0.25 for 4 classes). The improvement is the largest produced by any variant but does not constitute preservation — the k=3 signal is still largely destroyed at mid-trajectory by every variant tested.

**Q2: Does k=3 preservation improve no_last performance?**  
Best no_last accuracy (fuller_k3): 0.3687 from 'harmonic_split' (baseline: 0.2393), Δ=+0.1294.
This is the strongest measurable signal in the experiment. harmonic_split specifically prevents k>=2 angular pairs from being overwritten at each routing step, so some k=3 information accumulates in the trajectory and is readable at the final position even without injection. However, 0.3687 remains far below useful routing performance (~chance for a random-walk policy). The k=3 advantage is partial and not sufficient for routing to exploit it autonomously.

**Q3: Is the transport operator acting as a low-pass filter?**  
fuller_k3: initial decodability 1.0000 → mid 0.3076 (drop: 0.6924).  Reduced_k1: initial 0.2937 → mid 0.2786.  The collapse from 1.0 to chance for fuller_k3, while reduced_k1 remains at chance throughout, confirms the transport destroys high-frequency harmonics specifically — consistent with a low-pass filter.  

**Q4: Can minimal residual/structured updates prevent harmonic collapse?**  
Best mid decodability across all variants (fuller_k3): 0.3367.  NO — none of the tested variants prevent harmonic collapse at mid-trajectory.

---

## Honesty Section

**Whether baseline behaves as a low-pass filter:**  
YES — the drop from 1.0 to near-chance at mid-trajectory is severe and selective (only affects higher harmonics), consistent with low-pass filtering.  

**Which minimal modification (if any) improves preservation:**  
`harmonic_split` (eps_high=0.5) is the best of the tested variants:
- Mid decodability: 0.3367 (+0.0291 over baseline) — marginal, still near chance
- No_last accuracy: 0.3687 (+0.1294 over baseline) — a genuine signal, but well below useful routing threshold

The no_last improvement is mechanistically coherent: protecting k>=2 pairs from being overwritten prevents per-step erasure, allowing some cumulative k=3 content to persist. But eps_high=0.5 decays as 0.5^t, so after 11 steps the initial contribution is ~0.0005 — the observed improvement comes from partial preservation of intermediate states, not the original initial representation.

Residual carry (ε=0.1, ε=0.25) applied to the full hybrid tau shows minimal improvement (mid Δ=+0.0047, +0.0103) because the residual mixes both the k=1 and k=3 components equally, and the fundamental normalization step already partially counteracts the carry.  

**What remains unresolved:**  
- Whether a stronger harmonic freeze (eps_high → 1.0) would preserve k=3 (at cost of routing losing harmonic information for operator selection).  
- Whether a trainable per-harmonic decay rate could find an optimal tradeoff.  
- Whether architectures with explicit harmonic memory (e.g., separate slow/fast channels for different harmonic orders) would solve the preservation problem.  
- Whether the low-pass filtering is an inherent property of the angular routing operator or a consequence of the training objective.  

---

## HARMONIC CONTENT THROUGH TRANSPORT IS: DESTROYED

