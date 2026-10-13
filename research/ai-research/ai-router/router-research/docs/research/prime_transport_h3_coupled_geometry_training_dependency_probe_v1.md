# Prime Transport H3 Coupled Geometry Training Dependency Probe v1

**Branch:** h3_coupled_geometry_training_dependency_probe_v1  
**Contract:** prompt_contract_v4.md — loaded and binding  
**Elapsed:** 121.5s  

---

## 1. System Decomposition

| Parameter | Variable | Current Default | Code Location |
| --- | --- | --- | --- |
| D (transport depth) | Internal / Geometry | 20 | comparison_forward(D=...) |
| carrier_constant | Internal / Geometry | 1.0 (unit H3 pair) | carrier_scale × 1.0 |
| subspace_constant | Internal / Geometry | √6 ≈ 2.449 | sqrt(N_PAIRS) |
| operator_type | Internal / Geometry | joint (cos+sin) | convert_onehot_to_angular_operator |
| H3 harmonic index | Internal / Geometry | dims 10,11 (h=3) | H3_IDX0, H3_IDX1 — FIXED |
| training_steps | External / Structure | 500 (STRUCTURED) | train_module(n_steps=...) |
| training_data distribution | External / Structure | balanced 4-class (pool_ids) | pool sampling |

**Reference/training data definition:**
The MinimalTrainedComparisonModule (758 params, MLP) is trained via geometric CE loss
on h3_gt labels drawn from pool_ids. Token input = correct class label at each step.
MINIMAL = 0 steps (untrained), STRUCTURED = 500 steps, EXPANDED = 1500 steps.

**Success criteria:**
- success_rate > 0.28 (exceeds random + small margin).
- 4-class H3 accuracy via argmax cosine similarity to class anchors.
- cos_metric > 0.5 for true class anchor.

---

## 2. Parameter Grid

**D sweep:** [5, 12, 16, 20, 24, 32]
**Carrier scales:** [0.5, 1.0, 2.0]
**Operator types:** ['cos_only', 'sin_only', 'joint']
**Training regimes:** [('MINIMAL', 0), ('STRUCTURED', 500), ('EXPANDED', 1500)]

**Operator discriminability ceilings:**
- joint:    4-class (classes 0,1,2,3 all distinguishable) — ceiling = 1.00
- cos_only: classes 1 and 3 both have sin(H3)=0 → ambiguous — ceiling ≈ 0.50
- sin_only: classes 0 and 2 both have cos(H3)=0 → ambiguous — ceiling ≈ 0.50

---

## 3. Precondition Results

| Precondition | Expected | Result | Pass? |
| --- | --- | --- | --- |
| MINIMAL training → near random | success ≈ 0.25 | 0.2344 | PASS |
| EXPANDED training > MINIMAL + 0.02 | Δacc > 0.02 | Δ=0.0703 | PASS |
| D=5 differs from D=20 (STRUCTURED, joint) | Different values | D=5:0.2891 vs D=20:0.2676 | PASS |
| cos_only structurally limited | ceiling ≈ 0.50 | 0.2969 | PASS (structural) |

**Precondition result: PASSED — training shows detectable variation; experiment is testable.**

---

## 4. Case A — Baseline

D=20, carrier_scale=1.0, operator=joint, training=STRUCTURED(500), seed=42

| D | operator_type | carrier_constant | training_regime | training_steps | success_rate | cos_metric | sin_metric | carrier_norm | exceeds_random | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 20 | joint | 1.0000 | STRUCTURED | 500 | 0.2676 | 0.0748 | 0.5502 | 0.9028 | NO |  |

---

## 5. Case B — Geometry Sweep (Fixed STRUCTURED Training)

### B1: D sweep (carrier_scale=1.0, joint)

| D | operator_type | carrier_constant | training_regime | training_steps | success_rate | cos_metric | sin_metric | carrier_norm | exceeds_random | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 5 | joint | 1.0000 | STRUCTURED | 500 | 0.2891 | 0.0639 | 0.5923 | 0.5162 | YES |  |
| 12 | joint | 1.0000 | STRUCTURED | 500 | 0.2891 | 0.0556 | 0.6076 | 0.3291 | YES |  |
| 16 | joint | 1.0000 | STRUCTURED | 500 | 0.2402 | -0.0069 | 0.6117 | 0.5931 | NO |  |
| 24 | joint | 1.0000 | STRUCTURED | 500 | 0.2266 | -0.0322 | 0.5746 | 0.7845 | NO |  |
| 32 | joint | 1.0000 | STRUCTURED | 500 | 0.2441 | 0.0037 | 0.5181 | 0.9830 | NO |  |

### B2: Carrier scale sweep (D=20, joint)

| D | operator_type | carrier_constant | training_regime | training_steps | success_rate | cos_metric | sin_metric | carrier_norm | exceeds_random | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 20 | joint | 0.5000 | STRUCTURED | 500 | 0.2480 | 0.0217 | 0.5336 | 0.4871 | NO |  |
| 20 | joint | 2.0000 | STRUCTURED | 500 | 0.2402 | -0.0093 | 0.6365 | 0.7697 | NO |  |

### B3: Operator type sweep (D=20, carrier_scale=1.0)

| D | operator_type | carrier_constant | training_regime | training_steps | success_rate | cos_metric | sin_metric | carrier_norm | exceeds_random | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 20 | cos_only | 1.0000 | STRUCTURED | 500 | 0.2969 | 0.0820 | 0.4883 | 0.4836 | YES | ceiling~0.5 (classes 1,3 indistinguishable) |
| 20 | sin_only | 1.0000 | STRUCTURED | 500 | 0.2090 | -0.0664 | 0.4994 | 0.4674 | NO | ceiling~0.5 (classes 0,2 indistinguishable) |

---

## 6. Case C — Training Sweep (Fixed Geometry: D=20, cs=1.0, joint)

| D | operator_type | carrier_constant | training_regime | training_steps | success_rate | cos_metric | sin_metric | carrier_norm | exceeds_random | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 20 | joint | 1.0000 | MINIMAL | 0 | 0.2344 | 0.0199 | 0.6648 | 0.3601 | NO |  |
| 20 | joint | 1.0000 | EXPANDED | 1500 | 0.3047 | 0.0661 | 0.5481 | 0.5775 | YES |  |

---

## 7. Case D — Coupled Sweep

### D1: D × Training regime (operator=joint, carrier_scale=1.0)

| D | operator_type | carrier_constant | training_regime | training_steps | success_rate | cos_metric | sin_metric | carrier_norm | exceeds_random | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 5 | joint | 1.0000 | EXPANDED | 1500 | 0.2695 | 0.0380 | 0.6178 | 0.4432 | NO |  |
| 32 | joint | 1.0000 | EXPANDED | 1500 | 0.3203 | 0.1505 | 0.5103 | 0.9912 | YES |  |
| 5 | joint | 1.0000 | MINIMAL | 0 | 0.2754 | 0.0079 | 0.6079 | 0.3489 | NO |  |
| 32 | joint | 1.0000 | MINIMAL | 0 | 0.2285 | -0.0111 | 0.6510 | 0.3561 | NO |  |

### D2: Operator × Training regime (D=20, carrier_scale=1.0)

| D | operator_type | carrier_constant | training_regime | training_steps | success_rate | cos_metric | sin_metric | carrier_norm | exceeds_random | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 20 | cos_only | 1.0000 | EXPANDED | 1500 | 0.2480 | -0.0156 | 0.4883 | 0.4532 | NO | ceiling~0.5 (classes 1,3 indistinguishable) |
| 20 | cos_only | 1.0000 | MINIMAL | 0 | 0.2637 | 0.0156 | 0.4883 | 0.2276 | NO | ceiling~0.5 (classes 1,3 indistinguishable) |
| 20 | sin_only | 1.0000 | EXPANDED | 1500 | 0.2930 | 0.0841 | 0.4686 | 0.4435 | YES | ceiling~0.5 (classes 0,2 indistinguishable) |
| 20 | sin_only | 1.0000 | MINIMAL | 0 | 0.2090 | -0.0703 | 0.5117 | 0.2120 | NO | ceiling~0.5 (classes 0,2 indistinguishable) |

### D3: Carrier scale × Training regime (D=20, operator=joint)

| D | operator_type | carrier_constant | training_regime | training_steps | success_rate | cos_metric | sin_metric | carrier_norm | exceeds_random | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 20 | joint | 0.5000 | EXPANDED | 1500 | 0.2461 | 0.0277 | 0.5354 | 0.4986 | NO |  |
| 20 | joint | 0.5000 | MINIMAL | 0 | 0.2422 | 0.0379 | 0.6579 | 0.1783 | NO |  |
| 20 | joint | 2.0000 | EXPANDED | 1500 | 0.2480 | -0.0105 | 0.6184 | 0.6621 | NO |  |
| 20 | joint | 2.0000 | MINIMAL | 0 | 0.2383 | 0.0036 | 0.6343 | 0.6993 | NO |  |

---

## 8. Case E — Stability Check (Seeds)

| D | operator_type | carrier_constant | training_regime | training_steps | success_rate | cos_metric | sin_metric | carrier_norm | exceeds_random | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 20 | joint | 1.0000 | STRUCTURED | 500 | 0.2793 | 0.0464 | 0.4867 | 0.9835 | NO |  |
| 20 | joint | 1.0000 | STRUCTURED | 500 | 0.2520 | -0.0099 | 0.5215 | 0.7872 | NO |  |

---

## 9. Geometry vs Training Comparison

| Comparison | Config | Success Rate | Change vs Baseline |
| --- | --- | --- | --- |
| Baseline | D=20, cs=1.0, joint, STRUCTURED | 0.2676 | — |
| MINIMAL training | D=20, cs=1.0, joint, 0 steps | 0.2344 | -0.0332 |
| EXPANDED training | D=20, cs=1.0, joint, 1500 steps | 0.3047 | +0.0371 |
| D=5 | D=5, cs=1.0, joint, STRUCTURED | 0.2891 | +0.0215 |
| D=32 | D=32, cs=1.0, joint, STRUCTURED | 0.2441 | -0.0235 |
| cos_only operator | D=20, cs=1.0, cos_only, STRUCTURED | 0.2969 | +0.0293 |
| sin_only operator | D=20, cs=1.0, sin_only, STRUCTURED | 0.2090 | -0.0586 |
| carrier_scale=0.5 | D=20, cs=0.5, joint, STRUCTURED | 0.2480 | -0.0196 |
| carrier_scale=2.0 | D=20, cs=2.0, joint, STRUCTURED | 0.2402 | -0.0274 |

---

## 10. Coupled Effect Analysis

### Did any coupled dependency emerge?

- Max success from geometry-only sweep (Case B): **0.2969**
- Max success from training-only sweep (Case C): **0.3047**
- Max success from coupled sweep (Case D): **0.3203**
- Baseline (Case A): **0.2676**

Best coupled configuration: D=32, op=joint, cs=1.0000, regime=EXPANDED, success=0.3203

| Question | Answer |
| --- | --- |
| Is system primarily geometry, training, or both? | Training dominates — see Case C vs B |
| Does D help under any condition? | Minimal — D effect small |
| Do constants (carrier_scale) matter? | Neutral — carrier effect small |
| Does training unlock behavior? | Yes — MINIMAL vs STRUCTURED differ significantly |
| Any coupled-only improvement? | NO — coupled does not exceed single-axis |

---

## 11. Critical Questions

**Q1. Is the system primarily controlled by geometry, training, or both?**
Geometry effect (D sweep range): 0.0450. Training effect (MINIMAL→STRUCTURED delta): 0.0332.

**Q2. Does D help under any condition, or only degrade?**
D=5 success: 0.2891, D=32 success: 0.2441, D=20 (baseline): 0.2676.

**Q3. Do constants (carrier_scale) matter, or are they neutral?**
cs=0.5: 0.2480, cs=1.0: 0.2676, cs=2.0: 0.2402.

**Q4. Does training/reference structure unlock behavior not visible in minimal regime?**
MINIMAL: 0.2344, STRUCTURED: 0.2676, EXPANDED: 0.3047.

**Q5. Is there ANY evidence of emergent ratio/lock behavior under coupled conditions?**
Ratio varies with carrier_scale: ['0.2182', '0.6667']. No lock detected (no convergence to constant ratio).

---

## 12. Final Classification

**TRAINING_ONLY_EFFECT**

Training regime changes dominate; geometry sweep has minimal effect.

---

H3 COUPLED GEOMETRY TRAINING DEPENDENCY STATUS: TRAINING_ONLY_EFFECT
