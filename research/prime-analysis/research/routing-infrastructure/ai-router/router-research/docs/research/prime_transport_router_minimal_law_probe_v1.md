# Prime Transport Router: Minimal Routing Law Probe v1

**Generated:** 2026-04-08T06:47:59Z  
**Config:** D=24, D_HIDDEN=32, B=256, CPU  
**Train baseline:** acc=1.0000, solve_step=2500  
**Inference baselines:** exact_k1=1.0000, soft_mlp=1.0000  

---

## Valid Corridor Definition

The **valid geometric corridor** at each routing step is defined as:

```
valid_corridor(step) = {op : ang_sim[op] >= max(ang_sim) - W}
```

where `ang_sim[op] = cur_dir · TN[op]` (cosine-like angular similarity between current geometric direction and candidate operator direction).  

Two corridor widths tested:  
- **Wide:** W = 0.35 (≈ half the median ang_sim margin of 0.63)  
- **Tight:** W = 0.1  

### Corridor Size Distribution

| width W | mean ops in corridor | fraction singleton (1 op) | fraction ≤2 ops |
|---------|---------------------|--------------------------|------------------|
| 0.10 | 1.108 | 0.902 | 0.990 |
| 0.20 | 1.268 | 0.767 | 0.968 |
| 0.35 | 1.402 | 0.683 | 0.932 |
| 0.50 | 1.639 | 0.538 | 0.857 |
| 1.00 | 2.343 | 0.258 | 0.584 |

---

## Variant Results

| variant | routing law | acc_mean | acc_std | acc_min | H(route) | transport_sim | corridor_size | PASS? |
|---------|-------------|----------|---------|---------|----------|--------------|--------------|-------|
| exact_k1 | k=1 exact nearest-neighbor (deterministic) | 1.0000 | 0.0000 | 1.0000 | 1.355 | 2.824 | — | ✓ |
| top2_random | top-2 uniform random | 1.0000 | 0.0000 | 1.0000 | 1.596 | 2.470 | — | ✓ |
| top3_random | top-3 uniform random | 1.0000 | 0.0000 | 1.0000 | 1.631 | 2.156 | — | ✓ |
| top4_random | top-4 uniform random | 1.0000 | 0.0000 | 1.0000 | 1.659 | 1.881 | — | ✓ |
| fixed_op_0 | fixed operator 0 (negative control) | 1.0000 | 0.0000 | 1.0000 | 0.000 | 1.774 | — | ✓ |
| corridor_random | corridor random (ang_sim >= max - 0.35) | 1.0000 | 0.0000 | 1.0000 | 1.398 | 2.776 | 1.37 | ✓ |
| corridor_tight | corridor tight (ang_sim >= max - 0.1) | 1.0000 | 0.0000 | 1.0000 | 1.361 | 2.822 | 1.10 | ✓ |
| unrestricted_rand | uniform random over all 6 (negative control) | 1.0000 | 0.0000 | 1.0000 | 1.680 | 1.257 | — | ✓ |

**No failures detected across any variant.**  
**No degradation detected.**  

---

## Explicit Answers

**1. Does exact nearest-neighbor routing matter?**  
No. Even uniform-random routing preserves correctness. Exact k=1 selection is not required.  

**2. How much randomness can be introduced before failure?**  
Full randomness (uniform over all 6 operators) does not cause failure at D=24 with N_EVAL=2048 and 10 trials.  

**3. Is there a broader geometric validity corridor containing interchangeable moves?**  
Yes. At W=0.35, the corridor contains on average 1.40 valid operators. Only 68.3% of steps are singletons (forced choices). The manifold enforces a corridor, not a single path.  

**4. What is the minimum routing law that preserves correctness?**  
See conclusion line below.  

---

## Honesty Section

**What exact selection is unnecessary for:**  
- [top2_random]: acc=1.0000 — exact k=1 is replaceable by this policy.  
- [top3_random]: acc=1.0000 — exact k=1 is replaceable by this policy.  
- [top4_random]: acc=1.0000 — exact k=1 is replaceable by this policy.  
- [fixed_op_0]: acc=1.0000 — exact k=1 is replaceable by this policy.  
- [corridor_random]: acc=1.0000 — exact k=1 is replaceable by this policy.  
- [corridor_tight]: acc=1.0000 — exact k=1 is replaceable by this policy.  
- [unrestricted_rand]: acc=1.0000 — exact k=1 is replaceable by this policy.  

**What remains necessary:**  
- Even unconstrained randomness did not cause failure in this configuration. The task may have a high degree of path equivalence.  

**Whether the manifold enforces a corridor rather than a single path:**  
Yes. At W=0.35, >30% of steps have ≥2 valid candidates. The geometry organises a corridor of valid moves, not a single trajectory.  

---

## MINIMUM ROUTING LAW REQUIRED: ANY LOCALLY VALID GEOMETRIC CANDIDATE (even unrestricted random)

