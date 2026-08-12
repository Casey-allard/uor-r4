# Prime Transport Router: Split-Transport Forward Use Test v1

**Generated:** 2026-04-08T16:39:22Z  
**Config:** D=24, D_HIDDEN=32, B=512, N_eval=2048, steps=3500, LR=0.02  
**Geometry:** `fuller_k3` only  
**Branch:** split-transport adoption  

---

## Split-Transport Definition (Canonical Rule)

```
k=1 (fundamental harmonic):
    tau_k1_{t+1} = normalize(weighted_avg_neighbor_k1(tau_t))
    Full transport — low-frequency component updated normally

k>=2 (higher harmonics):  eps_high = 1.0
    tau_high_{t+1} = (1 - 1.0) * transport_high + 1.0 * tau_high_t
                   = tau_high_t
    Frozen — high-frequency components held at initial value
```

**Rationale (locked findings):**  
- Baseline transport collapses k=3 signal: mid_dec = 0.3076 (near chance) from initial=1.0  
- eps_high=1.0 fully preserves k=3 signal: mid_dec = 1.0, no_last = 1.0, solve_step = 500  

---

## Task Definitions

**Both tasks use:** `fuller_k3` geometry, target = `mod12_initial_state % 4` (4 classes)  

| task | inject | b_posLast_init | description |
|------|--------|---------------|-------------|
| task_A_inject | yes | 2.0 | inject x0 token at t=D-1 (current task) |
| task_B_no_inject | no | 0.0 | no injection — tau-only decoding (harder) |

**Why task_B is harder:**  
- No injection shortcut at t=D-1  
- Model must decode mod12 class from tau trajectory alone  
- 24 transport steps potentially degrade harmonic signal  
- With baseline transport: k=3 collapses by t=11 → decoding should be near-chance  
- With split_canonical: k=3 frozen at all positions → decoding should remain decodable  

---

## Main Comparison: Accuracy and Solve Step

| transport | task | accuracy | solve_step | no_last | α_{D-1} | runtime_s |
|-----------|------|----------|------------|---------|---------|----------|
| baseline | task_A_inject | 1.0000 | 2500 | 0.2446 | 0.8565 | 67.1 |
| split_canonical | task_A_inject | 1.0000 | 500 | 1.0000 | 0.6358 | 66.9 |
| baseline | task_B_no_inject | 0.2461 | — | 0.2271 | 0.0417 | 67.3 |
| split_canonical | task_B_no_inject | 1.0000 | 500 | 1.0000 | 0.0417 | 66.3 |

---

## Decodability Progression (initial / mid / final)

**Reference (locked):** baseline/task_A initial=1.0, mid=0.3076, final=1.0  
**Reference (locked):** split_canonical/task_A initial=1.0, mid=1.0, final=1.0  

| transport | task | initial | mid (t=11) | final (t=23) |
|-----------|------|---------|------------|-------------|
| baseline | task_A_inject | 1.0000 | 0.3076 | 1.0000 |
| split_canonical | task_A_inject | 1.0000 | 1.0000 | 1.0000 |
| baseline | task_B_no_inject | 1.0000 | 0.3076 | 0.2930 |
| split_canonical | task_B_no_inject | 1.0000 | 1.0000 | 1.0000 |

---

## Questions Answered

**Q1: Does split_canonical outperform baseline on Task A (inject, current)?**  
baseline: solve=2500, no_last=0.2446, mid_dec=0.3076  
split_canonical: solve=500, no_last=1.0000, mid_dec=1.0000  
**YES** — split_canonical outperforms baseline on Task A.  

**Q2: Does split_canonical remain beneficial on Task B (no inject, harder)?**  
baseline: acc=0.2461, solve=—, mid_dec=0.3076  
split_canonical: acc=1.0000, solve=500, mid_dec=1.0000  
**YES** — split_canonical outperforms baseline on Task B.  

**Q3: Is split_canonical ready to adopt as the working architecture?**  
split_canonical solves both Task A (acc=1.0000) and Task B (acc=1.0000).  
**YES — split_canonical is viable on both tested tasks.**  

---

## SPLIT-TRANSPORT WORKING STATUS: GENERALIZING

---

## Honesty Section

### What is now established

- split_canonical (eps_high=1.0) is reproducibly better than baseline on Task A  
  (confirmed in both harmonic_preservation_threshold_v1 and this experiment)  
- k=3 harmonic structure is preserved through all 24 transport steps with split_canonical  
- The solve advantage (500 vs 2500 steps on Task A) is stable across runs  

### What remains local to the current task

- All results so far are on `mod12_initial_state % 4` (4-class, inject@last)  
- Task B is the first test of split_canonical without the injection shortcut  
- Whether split_canonical advantage persists on Task B is the result of this experiment  

### What next limitation appears

- split_canonical solves Task B (acc=1.0000, solve=500)  
- Next question: does the advantage hold on tasks with MORE CLASSES or longer sequences?  
- The current test is still limited to mod-12 quarter-phase (4 classes)  
- Generalization to fundamentally different label structures is untested  

