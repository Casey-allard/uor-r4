# Prime Transport Router: Harmonic Preservation Threshold v1

**Generated:** 2026-04-08T16:16:52Z  
**Config:** D=24, D_HIDDEN=32, B=512, N_eval=2048, steps=3500, LR=0.02, b_posLast_init=2.0  
**Regime:** `fuller_k3` only  
**Transport:** harmonic-split only — k=1 full transport; k≥2 freeze sweep  
**Task:** `target = mod12_initial_state % 4`  (state-specific x0)  

---

## Transport Rule

```
k=1 (low-freq):  tau_k1_{t+1}   = normalize(transport(tau_k1_t))
k≥2 (high-freq): tau_high_{t+1} = (1-eps_high)*transport_high + eps_high*tau_high_t
```

| eps_high | description | expected k=3 fraction at t=11 |
|----------|-------------|--------------------------------|
| 0.50 | baseline harmonic-split (from v1) | 0.0005 |
| 0.75 | moderate freeze | 0.0422 |
| 0.90 | strong freeze | 0.3138 |
| 1.00 | full freeze — k≥2 components static | 1.0000 |

---

## Training and Task Results

| variant | eps_high | accuracy | solve_step | α_{D-1} | no_last | runtime_s |
|---------|----------|----------|------------|---------|---------|----------|
| eps_high_0.50 | 0.50 | 1.0000 | 2500 | 0.8582 | 0.3677 | 69.6 |
| eps_high_0.75 | 0.75 | 1.0000 | 2500 | 0.8581 | 0.6318 | 72.5 |
| eps_high_0.90 | 0.90 | 1.0000 | 1500 | 0.8210 | 0.9717 | 69.5 |
| eps_high_1.00 | 1.00 | 1.0000 | 500 | 0.6357 | 1.0000 | 70.5 |

---

## Decodability Progression (initial / mid / final)

**Locked reference (v1 baseline, fuller_k3):** initial=1.0000, mid=0.3076 (near chance), final=1.0000  

| eps_high | initial | mid (t=11) | final (t=23) | Δ_mid_vs_baseline |
|----------|---------|------------|-------------|-------------------|
| 0.50 | 1.0000 | 0.3367 | 1.0000 | +0.0291 |
| 0.75 | 1.0000 | 0.4463 | 1.0000 | +0.1387 |
| 0.90 | 1.0000 | 0.9294 | 1.0000 | +0.6218 |
| 1.00 | 1.0000 | 1.0000 | 1.0000 | +0.6924 |

---

## no_last Accuracy vs eps_high

| eps_high | no_last | final_accuracy | notes |
|----------|---------|----------------|-------|
| 0.50 | 0.3677 | 1.0000 | Δ_vs_baseline=+0.1284 |
| 0.75 | 0.6318 | 1.0000 | Δ_vs_baseline=+0.3925 |
| 0.90 | 0.9717 | 1.0000 | Δ_vs_baseline=+0.7324 |
| 1.00 | 1.0000 | 1.0000 | Δ_vs_baseline=+0.7607 — full freeze |

---

## Key Questions

**Q1: Does mid-trajectory decodability increase monotonically with eps_high?**  
Mid decodabilities by eps_high: 0.50→0.3367, 0.75→0.4463, 0.90→0.9294, 1.00→1.0000.  
YES — decodability increases (weakly) with eps_high, consistent with progressive harmonic preservation.  

**Q2: Is there a threshold where mid decodability is significantly above chance (>0.45)?**  
YES — threshold crossed at eps_high=0.90 (mid decodability=0.9294 > 0.45).  
Peak mid decodability: 1.0000 at eps_high=1.00.  

**Q3: Does strong preservation (eps_high ≥ 0.9) improve no_last accuracy further?**  
Strong results (eps_high=0.90→0.9717, eps_high=1.00→1.0000) vs moderate reference (eps_high=0.75→0.6318).  
YES — strong preservation further improves no_last.  

**Q4: Does freezing (eps_high = 1.0) break learning or still allow correct final accuracy?**  
LEARNING IS INTACT — eps_high=1.0 achieves accuracy=1.0000 (solve_step=500).  
Full freeze of k≥2 components does not prevent the network from learning via the injection mechanism.  

---

## Threshold Identification

**STRONG threshold detected:** mid-trajectory decodability exceeds 0.70, indicating k=3 signal is substantially preserved at high eps_high values.  

**Summary interpretation:**  

- Transport CAN be made partially signal-preserving by increasing eps_high.  
- Full freeze (eps_high=1.0) does NOT harm final-step learning (acc=1.0 maintained). The injection mechanism compensates.  

---

## HARMONIC PRESERVATION THRESHOLD IS: STRONG

