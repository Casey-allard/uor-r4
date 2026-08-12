# Prime Transport Router: Pure Mod-12 Representation Capacity Probe v1

**Generated:** 2026-04-08T09:20:25Z  
**Scope:** Representation-only. No routing, no trajectory, no attention, no training.

---

## Representation Definitions

**Input:** `tau0_oh[:, 9:21]` — the mod-12 one-hot block of each state's initial representation.  
**mod12 state:** `j = tau0_oh[:, 9:21].argmax()` ∈ {0, 1, ..., 11}  
**Target:** `j % 4` → 4 quarter-phase classes  

| Class | States j | Quarter-phase |
|-------|----------|---------------|
| 0 | {0,4,8} | every 4th tick from tick 0 |
| 1 | {1,5,9} | every 4th tick from tick 1 |
| 2 | {2,6,10} | every 4th tick from tick 2 |
| 3 | {3,7,11} | every 4th tick from tick 3 |

### reduced_k1  (k=1 harmonic only)

```
rep(j) = [cos(2πj/12), sin(2πj/12)]    (2 dims)
```

Analytic behavior:  
- Class 0: j∈{0,4,8}  → angles {0°, 120°, 240°}  — **equilateral triangle** on the unit circle  
- Class 1: j∈{1,5,9}  → angles {30°, 150°, 270°} — **equilateral triangle**, rotated 30°  
- Class 2: j∈{2,6,10} → angles {60°, 180°, 300°} — **equilateral triangle**, rotated 60°  
- Class 3: j∈{3,7,11} → angles {90°, 210°, 330°} — **equilateral triangle**, rotated 90°  
- Four interleaved equilateral triangles with identical centroid (0,0) — **NOT linearly separable**.  

### fuller_k3  (k=1,2,3 harmonics)

```
rep(j) = [cos(2πj/12), sin(2πj/12),
          cos(4πj/12), sin(4πj/12),
          cos(6πj/12), sin(6πj/12)]    (6 dims)
```

The k=3 harmonic: `cos(6πj/12) = cos(πj/2)`, `sin(πj/2)`  
| j | k=3 pair | Class |
|---|----------|-------|
| 0 | (+1.0000, +0.0000) | 0 |
| 1 | (+0.0000, +1.0000) | 1 |
| 2 | (-1.0000, +0.0000) | 2 |
| 3 | (-0.0000, -1.0000) | 3 |
| 4 | (+1.0000, -0.0000) | 0 |
| 5 | (+0.0000, +1.0000) | 1 |
| 6 | (-1.0000, +0.0000) | 2 |
| 7 | (-0.0000, -1.0000) | 3 |
| 8 | (+1.0000, -0.0000) | 0 |
| 9 | (+0.0000, +1.0000) | 1 |
| 10 | (-1.0000, +0.0000) | 2 |
| 11 | (-0.0000, -1.0000) | 3 |
- All 3 members of each class map to the **same point** under k=3 harmonic.  
- Class 0 → (1,0), Class 1 → (0,1), Class 2 → (-1,0), Class 3 → (0,-1).  
- Four **orthogonal unit vectors** — **trivially linearly separable**.  

---

## Analytic Class Points

### reduced_k1 (k=1..1)

| Class | Members (repr of each j) | Exact collapse? |
|-------|--------------------------|----------------|
| 0 | [(1.0, 0.0), (-0.5, 0.866025), (-0.5, -0.866025)] | NO |
| 1 | [(0.866025, 0.5), (-0.866025, 0.5), (-0.0, -1.0)] | NO |
| 2 | [(0.5, 0.866025), (-1.0, 0.0), (0.5, -0.866025)] | NO |
| 3 | [(0.0, 1.0), (-0.866025, -0.5), (0.866025, -0.5)] | NO |
**Exact collapse (all classes):** NO  

### fuller_k3 (k=1..3)

| Class | Members (repr of each j) | Exact collapse? |
|-------|--------------------------|----------------|
| 0 | [(1.0, 0.0, 1.0, 0.0, 1.0, 0.0), (-0.5, 0.866025, -0.5, -0.866025, 1.0, -0.0), (-0.5, -0.866025, -0.5, 0.866025, 1.0, -0.0)] | NO |
| 1 | [(0.866025, 0.5, 0.5, 0.866025, 0.0, 1.0), (-0.866025, 0.5, 0.5, -0.866025, 0.0, 1.0), (-0.0, -1.0, -1.0, 0.0, 0.0, 1.0)] | NO |
| 2 | [(0.5, 0.866025, -0.5, 0.866025, -1.0, 0.0), (-1.0, 0.0, 1.0, -0.0, -1.0, 0.0), (0.5, -0.866025, -0.5, -0.866025, -1.0, 0.0)] | NO |
| 3 | [(0.0, 1.0, -1.0, 0.0, -0.0, -1.0), (-0.866025, -0.5, 0.5, 0.866025, -0.0, -1.0), (0.866025, -0.5, 0.5, -0.866025, 0.0, -1.0)] | NO |
**Exact collapse (all classes):** NO  

---

## Empirical Results (All 343,665 States, 5-Fold CV Logistic Regression)

| regime | repr_dims | linear_accuracy | linear_std | between_class_distance | within_class_variance | exact_collapse | separable |
|--------|-----------|-----------------|------------|------------------------|----------------------|----------------|-----------|
| reduced_k1 | 2 | 0.264121 | 0.063033 | 0.0 | 0.5 | False | NO |
| fuller_k3 | 6 | 1.0 | 0.0 | 1.609476 | 0.333333 | False | YES |

Chance level: 0.2500 (4 classes, balanced).  

---

## Explicit Answers

**1. Is k=1 alone linearly sufficient?**  
NO — linear accuracy = 0.264121 (≈ chance 0.25). The 4 quarter-phase classes form 4 interleaved equilateral triangles with centroid (0,0). No linear classifier can separate them. This is both analytically proven and empirically confirmed.  

**2. Are k=1,2,3 linearly sufficient?**  
YES — linear accuracy = 1.0. The k=3 harmonic (cos(πj/2), sin(πj/2)) maps all 3 members of each quarter-phase class to the same point (within-class variance in the k=3 subspace = 0.0 analytically). The full 6-dim within-class variance is 0.333333 (k=1 and k=2 components still vary within classes, but the k=3 component alone is sufficient for trivial linear separation). Empirical CV accuracy = 1.0 ± 0.0 on all 343,665 states.  

**3. Does this cleanly support the 12-tick / higher-harmonic intuition at the representation level?**  
YES — unambiguously at the representation level. The k=3 harmonic (period-4 within mod-12) is NECESSARY and SUFFICIENT for linear decoding of quarter-phase classes. k=1 alone cannot resolve this structure regardless of learning budget.  

---

## Honesty Section

**What is now cleanly proven (no trajectory confounds):**  
- k=1 representation of mod-12 cannot linearly decode quarter-phase classes (j%4). This is analytically exact and empirically confirmed on all 343,665 states.  
- k=3 representation makes this trivially solvable. The k=3 subspace (cos(πj/2), sin(πj/2)) has exact within-class variance = 0.0 analytically (all 3 class members map to the same point). The full 6-dim within-class variance is 0.333333 because k=1 and k=2 components vary within classes, but the k=3 component alone suffices for perfect linear separation.  
- The representational gap is not a matter of degree — it is a categorical impossibility vs trivial sufficiency.  

**What still requires dynamic testing later:**  
- Whether a routing system with fuller_k3 can actually exploit this representational advantage when the mod-12 class information must be preserved across D routing steps.  
- Whether the training gap (+0.0938 from the prior stress probe) would grow to full solution with more steps, a shorter routing depth, or a preservation mechanism.  
- Tangent / higher-dimensional operator extensions are not tested here and remain open.  

---

## PURE MOD-12 HARMONIC CAPACITY IS: K1 INSUFFICIENT / K3 NECESSARY
