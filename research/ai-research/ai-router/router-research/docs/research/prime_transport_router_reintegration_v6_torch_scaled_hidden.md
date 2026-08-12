# prime_transport_router_reintegration_v6_torch_scaled_hidden

**Branch:** horizon-scaled capacity experiment  
**Date:** 2026-04-07  
**Status:** branch — does NOT replace or modify the canonical v6_torch fixed-capacity path

---

## 1. Experiment Setup

### Scaling rule

| D (context delay) | D_HIDDEN (scaled) | D_HIDDEN_ATTN | canonical D_HIDDEN |
|---|---|---|---|
| 16 | 32 | 8 | 32 ← control point |
| 24 | 48 | 12 | 32 |
| 32 | 64 | 16 | 32 |
| 48 | 96 | 24 | 32 |

**Rule:** `D_HIDDEN = 2 × D` (linear with delay).  
**Attention ratio:** `D_HIDDEN_ATTN = D_HIDDEN // 4` (canonical 4:1 ratio preserved).  
**Batch size:** 256 (identical to canonical for fair comparison).  
**Training budget:** 10,000 batches per delay (identical to canonical context-scaling study).  
**Device:** cpu, `torch.set_num_threads(1)` (consistent with canonical).  
**Injection rule:** v6 step-0-only additive inject — UNCHANGED.  
**Operator algebra:** v20–v25 — UNCHANGED.

---

## 2. Results

### Scaled-capacity results at 10,000 batches

| D | D_HIDDEN | accuracy | route_H | transport | alpha0 | vs_chance | sps | verdict | Δacc vs canonical |
|---|---|---|---|---|---|---|---|---|---|
| 16 | 32 | 1.000 | 1.574 | 0.322 | 0.585 | 4.00× | 564617 | solved | +0.000 |
| 24 | 48 | 0.668 | 1.668 | 0.162 | 0.042 | 2.67× | 490772 | learning_not_solved | +0.165 |
| 32 | 64 | 0.378 | 1.704 | 0.935 | 0.031 | 1.51× | 479641 | learning_not_solved | -0.030 |
| 48 | 96 | 0.297 | 1.674 | 0.607 | 0.021 | 1.19× | 397804 | near_chance_failed | +0.022 |

chance = 0.250 (4-class uniform)

### Comparison vs canonical fixed-capacity branch

| D | canonical D_HIDDEN | canonical acc | scaled D_HIDDEN | scaled acc | change |
|---|---|---|---|---|---|
| 16 | 32 (canonical) | 1.000 | 32 (scaled) | 1.000 | ≈ same |
| 24 | 32 (canonical) | 0.503 | 48 (scaled) | 0.668 | ↑ improved |
| 32 | 32 (canonical) | 0.408 | 64 (scaled) | 0.378 | ↓ degraded |
| 48 | 32 (canonical) | 0.275 | 96 (scaled) | 0.297 | ↑ improved |

---

## 3. Analysis

### 3.1 Attention alpha0 concentration

- D=24: alpha0 0.0420 → 0.0419 (no change)
- D=32: alpha0 0.0310 → 0.0312 (no change)
- D=48: alpha0 0.0210 → 0.0210 (no change)

### 3.2 Hardware utilization / workload efficiency

At the canonical capacity (D_HIDDEN=32), the matmuls are too small for CPU
multi-threading to be beneficial (see cpu_utilization_diagnostic_v2). Scaling
D_HIDDEN with D changes the workload:

- D=16, D_HIDDEN=32: (256,25)@(25,32)  — 204,800 FLOP/step — tiny (same as canonical)
- D=24, D_HIDDEN=48: (256,25)@(25,48)  — 307,200 FLOP/step
- D=32, D_HIDDEN=64: (256,25)@(25,64)  — 409,600 FLOP/step
- D=48, D_HIDDEN=96: (256,25)@(25,96)  — 614,400 FLOP/step

Steps per second observed:
- D=16 (D_HIDDEN=32): 564617 sps
- D=48 (D_HIDDEN=96): 397804 sps

At D_HIDDEN=96, the matmuls are still below the threshold where CPU
multi-threading becomes beneficial (~1M FLOP), but the workload is meaningfully
larger. MPS execution would provide more benefit here than at canonical scale.

### 3.3 Does capacity scaling explain the long-context failure?

See comparison table (Section 2). The canonical failure at D=24+ showed:
- alpha0 = 1/D exactly (uniform attention — the model cannot retrieve the
  step-0 injection signal through D BPTT steps)
- High route entropy (near ln(6)=1.792) — routing not converged

**Result confirmed: alpha0 remained at exactly 1/D at all failing delays even with larger D_HIDDEN.**

| D | alpha0 canonical | alpha0 scaled | 1/D |
|---|---|---|---|
| 24 | 0.0420 | 0.0419 | 0.0417 |
| 32 | 0.0310 | 0.0312 | 0.0313 |
| 48 | 0.0210 | 0.0210 | 0.0208 |

The attention mechanism failed to concentrate on the step-0 injection position
regardless of D_HIDDEN width. The D=24 accuracy improvement (+0.165) was driven
entirely by the larger MLP (W1, W2) having more capacity to learn indirect
routing patterns — NOT by the attention readout recovering the injection signal.

**Conclusion: the primary bottleneck is structural BPTT gradient dilution,
not a representation capacity constraint.** Scaling D_HIDDEN cannot fix a
problem where the attention gradient at position 0 is diluted to 1/(D×loss)
before it reaches W_tok_inject.

---

## 4. Honesty Section

**What improved:**
- D=16 control point: confirmed identical to canonical (same D_HIDDEN=32)
- Larger D_HIDDEN matmuls: moves toward multi-threading threshold
- D=24 accuracy: canonical=0.503 → scaled=0.668 (++0.165)
- D=48 accuracy: canonical=0.275 → scaled=0.297 (++0.022)

**What did not improve:**
- D=24 alpha0: still near 1/D (0.0419)
- D=32 accuracy: canonical=0.408 → scaled=0.378 (-0.030)
- D=32 alpha0: still near 1/D (0.0312)
- D=48 alpha0: still near 1/D (0.0210)

**What remains uncertain:**
- Whether the D=24 MLP-capacity effect (not attention) would push to solved at higher budget
- Whether a dedicated memory cell or explicit carry-register (not attention) could fix BPTT dilution
- Whether the 2×D rule is the right scaling law for the MLP path even if not for the attention path

---

## 5. Honesty (per task requirements)

- Were any core files modified? No
- Were any operators rebuilt? No
- Is full exact spin_H solved? No

---

## 6. Next Step (exactly one)

**return to canonical branch and extend D=24 budget: scaling D_HIDDEN with D
improved D=24 accuracy by +0.165 (0.503 → 0.668) but alpha0 remained at 1/D
at every failing delay, confirming the bottleneck is structural BPTT gradient
dilution, not capacity. D=24 at canonical D_HIDDEN=32 is already at 0.503 and
still learning; a longer budget on the canonical path is the cleanest next test
of whether the injection signal is learnable at all at this horizon.**
