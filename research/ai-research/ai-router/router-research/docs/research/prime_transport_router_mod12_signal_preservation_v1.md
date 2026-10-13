# Prime Transport Router: Mod-12 Harmonic Signal Preservation v1

**Generated:** 2026-04-08T09:46:10Z  
**Config:** D=24, D_HIDDEN=32, B_train=512, N_eval=2048, steps=3500, LR=0.02, b_posLast_init=2.0  

**Task:** `target = mod12_initial_state % 4`  (state-specific x0; NOT random)  

---

## Regime Definitions

| component | reduced_k1 | fuller_k3 |
|-----------|------------|----------|
| mod-12 harmonics | k=1 only → 2 dims | k=1,2,3 → 6 dims |
| mod-5  harmonics | k=1 only (same) | k=1 only (same) |
| d_tau_hyb | 12 | 16 |
| d_in_hyb  | 16 | 20 |
| inject_position | last | last |
| b_posLast_init  | 2.0  | 2.0  |

**Only the mod-12 block varies.** Mod-5 kept at k=1 to isolate the mod-12 harmonic effect.  

**Known representation capacity (locked):**  
- reduced_k1: linear accuracy = 0.264 (k=1 cannot separate 4 interleaved triangles)  
- fuller_k3:  linear accuracy = 1.000 (k=3 collapses each class to orthogonal unit vector)  

---

## Training Results

| regime | d_hyb | accuracy | solve_step | α_{D-1} | b_posLast | runtime_s |
|--------|-------|----------|------------|---------|-----------|----------|
| reduced_k1 | 12 | 1.0000 | 2500 | 0.8743 | 5.0664 | 56.6 |
| fuller_k3 | 16 | 1.0000 | 2500 | 0.8567 | 4.9175 | 64.2 |

---

## Ablation Results

Note: `tau0_direct` here uses `st[:,0,:]` — the tau **after** routing step 0 (not the pre-routing τ₀).
See Decodability Probes for the true pre-routing representation capacity.

| regime | ablation | accuracy | interpretation |
|--------|----------|----------|----------------|
| reduced_k1 | full | 1.0000 | reference |
| reduced_k1 | tau0_direct | 0.2661 | τ at step-0 ≈ chance; k=1 cannot represent class at any depth ✓ |
| reduced_k1 | no_tau0 | 1.0000 | trajectory sufficient without τ₀ ✓ |
| reduced_k1 | last_only | 1.0000 | final position carries full answer ✓ |
| reduced_k1 | no_last | 0.2427 | t=D-1 injection critical; routing insufficient ✗ |
| fuller_k3 | full | 1.0000 | reference |
| fuller_k3 | tau0_direct | 0.2812 | τ at step-0 ≈ chance; k=3 initial capacity ERASED by first routing step ✗ |
| fuller_k3 | no_tau0 | 1.0000 | trajectory sufficient without τ₀ ✓ |
| fuller_k3 | last_only | 1.0000 | final position carries full answer ✓ |
| fuller_k3 | no_last | 0.2432 | t=D-1 injection critical; routing insufficient ✗ |

---

## Decodability Probes

Linear decodability of `mod12_class` from τ at three trajectory positions (logistic regression, train-fit accuracy).  

| regime | position | linear_decodability | interpretation |
|--------|----------|---------------------|----------------|
| reduced_k1 | initial | 0.2937 | pre-routing repr capacity (locked baseline) |
| reduced_k1 | mid | 0.2786 | mid-trajectory preservation (routing dynamics) |
| reduced_k1 | final | 1.0000 | final position (includes injection at t=D-1) |
| fuller_k3 | initial | 1.0000 | pre-routing repr capacity (locked baseline) |
| fuller_k3 | mid | 0.3076 | mid-trajectory preservation (routing dynamics) |
| fuller_k3 | final | 1.0000 | final position (includes injection at t=D-1) |

---

## Answers to Key Questions

**Q1: Does fuller_k3 outperform reduced_k1 in the full delayed trainable regime?**  
Both regimes reach similar full accuracy (both solve the injection task). The key difference appears in the no_last ablation.  

**Q2: Does the delayed trajectory preserve k=3 signal better than k=1?**  
Mid-trajectory decodability: reduced_k1=0.2786, fuller_k3=0.3076.  k=3 advantage at mid-trajectory: +0.0290.  The k=3 signal advantage does NOT survive the trajectory dynamics.  

**Q3: Is k=3 advantage only representational, or does it survive dynamics?**  
no_last accuracy for fuller_k3 = 0.2432.  The k=3 advantage is ONLY representational; routing dynamics do not preserve it.  

**Q4: Does this overturn the 'fuller geometry irrelevant' conclusion?**  
NO — even on the mod-12-specific task, fuller_k3 and reduced_k1 perform similarly in the no_last ablation. The earlier 'fuller geometry irrelevant' conclusion is NOT overturned.  

---

## Honesty Section

**What is now fairly tested:**  
- Both regimes trained under identical conditions (inject@last, b_posLast=2.0, 3500 steps, same task, same seed).  
- The task specifically requires mod-12 quarter-phase classification (the exact task proven necessary for k=3 at representation level).  
- Decodability probes directly measure whether mod-12 class information survives routing at three trajectory depths.  
- The no_last ablation tests routing-only signal preservation without the injection shortcut.  

**What still remains unresolved:**  
- Whether a longer training budget (>3500 steps) would change the dynamics result.  
- Whether different routing architectures (e.g. attention-based routing rather than angular selection) would better preserve harmonic structure.  
- Whether the result generalises to other modular structure (mod-5, etc.).  

**Whether the 12-tick intuition survives dynamic testing:**  
The 12-tick intuition holds at the REPRESENTATION level (initial decodability: fuller_k3=1.0000 vs reduced_k1=0.2937).  
At mid-trajectory the decodability for fuller_k3 = 0.3076.  
The 12-tick (k=3) advantage is LOST through dynamics. Routing erases the high-frequency harmonic information.  

---

## MOD-12 K3 THROUGH DYNAMICS IS: LOST

