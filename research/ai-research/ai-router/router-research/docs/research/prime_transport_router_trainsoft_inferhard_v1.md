# Prime Transport Router: Train-Soft / Infer-Hard Probe v1

**Generated:** 2026-04-08T06:09:04Z  
**Config:** D_HIDDEN=32, B=256, CPU, cache-loaded  
**Cache:** 343,665 states in 0.389s  
**D_ladder:** [24, 32, 48, 64]  

---

## Training Protocol

- **Training:** standard soft scaffold — k=6, all candidates, MLP routing,
  Gumbel-softmax blend, geometric exponential temperature schedule
- **Soft inference:** standard MLP forward pass at T=0.05
- **Hard inference:** k=1 angular proximity selection, no MLP in decision path,
  hard tau update (exact TN table entry, magnitude=1.0)

---

## Head-to-Head: Soft vs Hard Inference Across D

| D | soft_acc | hard_acc | Δ | soft_fwd_ms | hard_fwd_ms | speedup | geom_mlp_agree | solve_step |
|---|----------|----------|---|-------------|------------|---------|----------------|------------|
| 24 | 1.0000 | 1.0000 | +0.0000 ✓ | 4.603ms | 2.292ms | 2.008x | 0.1796 | 2500 |
| 32 | 1.0000 | 1.0000 | +0.0000 ✓ | 6.275ms | 3.289ms | 1.908x | 0.1704 | 3000 |
| 48 | 1.0000 | 1.0000 | +0.0000 ✓ | 8.864ms | 4.988ms | 1.777x | 0.1969 | 3500 |
| 64 | 1.0000 | 1.0000 | +0.0000 ✓ | 11.661ms | 5.951ms | 1.96x | 0.1051 | 4500 |

---

## Speed Advantage Across D

| D | soft_fwd_ms | hard_fwd_ms | speedup | stable? |
|---|-------------|-------------|---------|--------|
| 24 | 4.603ms | 2.292ms | 2.008x | yes |
| 32 | 6.275ms | 3.289ms | 1.908x | yes |
| 48 | 8.864ms | 4.988ms | 1.777x | yes |
| 64 | 11.661ms | 5.951ms | 1.96x | yes |

---

## First Degradation Point

Hard geometry inference shows **no meaningful degradation** across the full D ladder [24, 32, 48, 64].

---

## Geometry/MLP Agreement Across D

Agreement = fraction of routing steps where geom-k1 and MLP argmax select the same operator.

| D | geom_mlp_agreement | interpretation |
|---|--------------------|----------------|
| 24 | 0.1796 | geometry and MLP take largely independent paths |
| 32 | 0.1704 | geometry and MLP take largely independent paths |
| 48 | 0.1969 | geometry and MLP take largely independent paths |
| 64 | 0.1051 | geometry and MLP take largely independent paths |

---

## Honesty Section

### What holds

- Hard geometric inference matches soft inference accuracy through D=64
- D=24: hard 1.0000 vs soft 1.0000, speedup 2.008x
- D=32: hard 1.0000 vs soft 1.0000, speedup 1.908x
- D=48: hard 1.0000 vs soft 1.0000, speedup 1.777x
- D=64: hard 1.0000 vs soft 1.0000, speedup 1.96x
- Geometry/MLP agreement is low (< 25%) yet both achieve correct routing — the manifold has multiple valid paths
- Speed advantage is present at all tested D levels

### What breaks (or degrades)

- No accuracy degradation observed within tested D range

### What remains uncertain

- Whether hard geometric inference holds beyond D=64 (not tested)
- Whether the result generalises to D_HIDDEN > 32 or different model capacity
- The causal direction: does the geometry work *because* the manifold is correct,
  or *despite* it (multiple independent valid paths that both happen to succeed)?
- Training stability at larger D under the current fixed MAX_STEPS budget

---

## HARD GEOMETRIC INFERENCE VALID THROUGH: D=64
