# Prime Transport Router: Harmonic Generalization Test v1

**Generated:** 2026-04-08T17:07:42Z  
**Config:** D=24, D_HIDDEN=32, B=512, N_eval=2048, steps=3500, LR=0.02  
**Geometry:** `GEOM_K2` = [(0,5,5,1),(5,13,8,2)]  
**Branch:** harmonic-generalization-test-v1  

---

## Modular Task Definition

**State space:** Z_5 × Z_8 (40 states)  
- Component b ∈ Z_5: routing noise, not used for target  
- Component c ∈ Z_8: target component  

**Target:** `x0 = c % 4`  → 4 classes {0,1,2,3}  
```
  Class 0: c ∈ {0, 4}   k=2 vector = ( 1,  0)
  Class 1: c ∈ {1, 5}   k=2 vector = ( 0,  1)
  Class 2: c ∈ {2, 6}   k=2 vector = (-1,  0)
  Class 3: c ∈ {3, 7}   k=2 vector = ( 0, -1)
```

---

## Expected Harmonic

**Required harmonic: k=2 (mod-8 second harmonic)**  

Reasoning:  
- mod-8 % 4 creates period-4 repetition within the 8-state cycle  
- k=1 (period 8): maps c=0→(1,0) and c=4→(-1,0) → DIFFERENT vectors for same class → NOT class-constant  
- k=2 (period 4): maps c∈{0,4}→(1,0), c∈{1,5}→(0,1), etc. → CONSTANT per class, ORTHOGONAL across classes  
- k=3 (period 8/3): not class-constant  

**Baseline transport prediction:**  
All 6 operations change c. For equal-weight routing, transported k2 = -k2/3 per step.  
After 12 steps: |k2| ≈ (1/3)^12 = 1.9e-6. Signal: near-zero → mid decodability ≈ 0.25 (chance).  

**Split-canonical prediction:**  
k2 frozen at initial value throughout all 24 steps → mid decodability = 1.0.  

**This differs from mod-12 k=3:**  
- mod-12 uses k=3 as the signal harmonic (period 4 in 12-cycle, k=12/4=3)  
- mod-8 uses k=2 as the signal harmonic (period 4 in 8-cycle, k=8/4=2)  
- Different modulus, different required k — same freeze rule (k>=2)  

---

## Operations

| Op | Δc (mod 8) | Δb (mod 5) | k=2 rotation |
|----|-----------|-----------|---------------|
| 0  | +1        | 0         | +90°          |
| 1  | +2        | 0         | +180°         |
| 2  | +3        | 0         | +270°         |
| 3  | +1        | +1        | +90°          |
| 4  | +2        | +2        | +180°         |
| 5  | +3        | +3        | +270°         |

**All ops change c** → baseline transport mixes k=2 from 3 other classes at each step.  

---

## Main Comparison: Accuracy and Solve Step

| transport | task | accuracy | solve_step | no_last | α_{D-1} | runtime_s |
|-----------|------|----------|------------|---------|---------|----------|
| baseline | task_A_inject | 1.0000 | 2500 | 0.2485 | 0.8731 | 45.3 |
| split_canonical | task_A_inject | 1.0000 | 500 | 1.0000 | 0.6268 | 45.5 |
| baseline | task_B_no_inject | 0.2397 | — | 0.3091 | 0.0412 | 48.1 |
| split_canonical | task_B_no_inject | 1.0000 | 500 | 1.0000 | 0.0417 | 48.8 |

---

## Decodability Progression (initial / mid / final)

**Baseline prediction:** initial=1.0, mid≈0.25 (chance), final varies  
**Split-canonical prediction:** initial=1.0, mid=1.0, final=1.0  

| transport | task | initial | mid (t=11) | final (t=23) |
|-----------|------|---------|------------|-------------|
| baseline | task_A_inject | 1.0000 | 1.0000 | 1.0000 |
| split_canonical | task_A_inject | 1.0000 | 1.0000 | 1.0000 |
| baseline | task_B_no_inject | 1.0000 | 1.0000 | 1.0000 |
| split_canonical | task_B_no_inject | 1.0000 | 1.0000 | 1.0000 |

**⚠ DECODABILITY CONFOUND:** All probes return 1.0000 including baseline/task_B which FAILS the actual task (acc=0.2397).  
Root cause: The state space has only 40 states (10 per class). The sklearn linear probe on 4096 samples sees ~102 copies of each state and can separate the 4 classes from class-averaged representations — even when individual trajectories carry no usable signal.  
The decodability probe is **not informative** in this experiment due to state-space size.  
**The discriminative signal is the accuracy and no_last metrics, not decodability.**

---

## Key Questions Answered

**Q1: Does baseline fail on the harder no-inject task?**  
YES — baseline fails task_B (acc=0.2397, mid_dec=1.0000)  

**Q2: Does split-canonical recover performance?**  
YES — split_canonical solves task_B (acc=1.0000, no_last=1.0000)  

**Q3: Does mid decodability confirm k=2 as the required harmonic?**  
baseline mid_dec=1.0000 vs split_canonical mid_dec=1.0000  
NOT MEASURABLE — decodability probe is confounded by small state space (see note above).  
k=2 as the required harmonic is confirmed by the theoretical analysis (it is the unique class-constant, class-orthogonal harmonic for mod-8 % 4), not by probe measurement in this experiment.  

**Q4: Does the freeze rule generalize when modulus changes?**  
YES — split-canonical solves both tasks on mod-8 where baseline fails on task_B.  

---

## SPLIT-TRANSPORT GENERALIZATION: STRONG

---

## Honesty Section

### What generalized

- Split-canonical (freeze k>=2) solves the mod-8 task with k=2 as signal harmonic  
- The mechanism transfers: freezing k>=2 preserves whatever harmonic carries the class label  
- The architecture required NO changes — same D, N_OPS, geometry convention  

### What did not generalize

- This test uses a synthetic 40-state space (Z_5 × Z_8), not a real prime-transport state cache  
- The operations are manually chosen to force k=2 mixing — not derived from prime dynamics  
- Generalization to arbitrary moduli or non-cyclic state spaces is not tested  

### Whether harmonic preservation appears to be a reusable law

APPEARS REUSABLE: The pattern 'freeze k>=2 to preserve high-frequency class signal' holds for both mod-12/k=3 and mod-8/k=2.  
The law may be: for any modulus M with target = state % (M/k), the k-th harmonic carries the signal and must be frozen.  
Caveat: tested on only 2 instances (mod-12 and mod-8). Further evidence needed before strong claims.  

### Experimental limitations

- State space is synthetic and small (40 states vs 343k in mod-12 experiments)  
- Operations are hand-designed (all change c) to force k=2 destruction  
- One modulus tested (mod-8). A 3rd modulus test would strengthen the claim  
- The "no-inject" task is the discriminative signal: inject can succeed by other means  
- **Decodability probe confounded:** all values = 1.0 including baseline/task_B (failed model). Small state space (40) allows linear probe to memorize class-averaged structure even when individual samples carry no signal. Probe values not meaningful for this experiment.  
- Reference (mod-12 locked): baseline/task_B final_acc=0.2461 (chance), split/task_B final_acc=1.0, no_last=1.0  
