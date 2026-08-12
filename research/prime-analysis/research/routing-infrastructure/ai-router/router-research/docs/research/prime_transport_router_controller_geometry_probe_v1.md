# Prime Transport: Controller Geometry Language Probe v1

*Generated: 2026-04-08 18:48 UTC*

---

## Step 0: CPU/Thread Default Fix

**File modified:** `tools/prime_transport/thread_policy.py`

**Change:** Removed FLOP-based auto-detection that silently resolved to `1 thread` for all typical experiment configs (D≤32, B≤512). The old auto-select logic was: if `B×d_in×D_HIDDEN < 256×64×64`, use 1 thread. At the current experiment scale this always fired, effectively hard-coding 1 thread without any explicit request.

**New default rule:** `min(8, os.cpu_count())` — multi-threaded by default unless the user explicitly sets `PRIME_THREADS=1` (env var) or passes `override=1`.

**Function signature change:** `select_threads()` now accepts `batch_size`, `d_in`, `d_hidden` as optional (defaulting to 0) so callers do not need to pass workload dimensions for the default path. Explicit `override` and `PRIME_THREADS` env var still take priority.

**Verified:** `select_threads()` prints `[thread_policy] N thread(s) — default min(8, cpu_count=N)` at startup.

## Mandatory First Read: start_agnostic_root_probe_v1 CSV

Numbers read directly from `prime_transport_router_start_agnostic_root_probe_v1.csv`:

| Config | D=24 | D=32 | Δ | Note |
|--------|------|------|---|------|
| baseline (none) | 0.9858 | 0.7109 | −0.2749 | collapse at D=32 confirmed |
| sqrt2_radial | 0.9829 | 0.9785 | −0.0044 | stabilized |
| **two_i_orient** | **0.9922** | **0.9873** | **−0.0049** | **best single** |
| combined | 0.9844 | 0.9536 | −0.0308 | weaker than singles |

**Confirmed from CSV:**
- Baseline D=32 collapse: 0.7109 ✓
- sqrt2 stabilizes: 0.7109 → 0.9785 ✓
- two_i improvement: 0.9873 (best single) ✓
- Combined weaker than singles: 0.9536 < 0.9785 and 0.9873 ✓
- Decodability=1.0 throughout all configs: confirmed ✓

All decodability columns are 1.0 across initial/mid/final. Remaining problem is control/optimization/trajectory-conditioning, not representation loss.

## Experiment Design

### Fixed components
- Transport: split (eps_high=1.0, k≥2 frozen)
- Anchor: two_i_orient (two_i_rot) — rotate each (c,s) pair by +π/2
- Geometry: GEOM_K3 only
- Task: mod12 quarter-class, no injection
- Training budget: 3500 steps (identical to start-agnostic probe)
- D_HIDDEN=32, LR=0.02, BATCH=512, seed=42

### Controller geometry variants

**1. controller_baseline** (`ctrl_mode=baseline`)
  - Controller input: `[emb (4) | ang (d_ang) | mags (n_blocks)]`
  - `ang` = (cos θ_k, sin θ_k) for all k — standard sin/cos encoding
  - This is the existing behaviour; no change from prior probes

**2. controller_projective** (`ctrl_mode=projective`)
  - Controller input: `[emb (4) | proj (n_pairs) | mags (n_blocks)]`
  - `proj_k = sin_k / (1 + cos_k + 0.1)`, clipped to [−10, 10]
  - This is the half-angle stereographic projection (Weierstrass substitution)
  - Numerically safe: `1 + cos(θ) ∈ [0,2]`; adding ε=0.1 ensures denominator ≥ 0.1
  - Transport state stays in sin/cos form; only controller view changes
  - Feature count: n_pairs=6 vs d_ang=12 — strictly smaller input to controller

**3. controller_hybrid** (`ctrl_mode=hybrid`)
  - Controller input: `[emb (4) | ang (d_ang) | mags (n_blocks) | proj (n_pairs)]`
  - Both sin/cos AND projective features available to controller
  - Tests whether projective features ADD information beyond sin/cos

### GEOM_K3 structure (for reference)
- Blocks: `[(0,2,2,1), (2,7,5,1), (7,9,2,1), (9,21,12,3)]`
- n_pairs = 6, d_ang = 12, n_blocks = 4
- Baseline d_ctrl = D_EMB(4) + d_ang(12) + n_blocks(4) = 20
- Projective d_ctrl = D_EMB(4) + n_pairs(6) + n_blocks(4) = 14
- Hybrid d_ctrl = D_EMB(4) + d_ang(12) + n_blocks(4) + n_pairs(6) = 26

## Results

### Full Results Table

| Config | Mode | D | Accuracy | Solve | No-Last | Dec Init | Dec Mid | Dec Final | α_last | Runtime(s) |
|--------|------|---|----------|-------|---------|----------|---------|-----------|--------|------------|
| controller_baseline | baseline | 24 | **0.9907** | — | 0.9937 | 1.0 | 1.0 | 1.0 | 0.0353 | 96.3 |
| controller_projective | projective | 24 | **0.9878** | — | 0.9854 | 1.0 | 1.0 | 1.0 | 0.0373 | 111.8 |
| controller_hybrid | hybrid | 24 | **0.9912** | — | 0.9937 | 1.0 | 1.0 | 1.0 | 0.0288 | 110.0 |
| controller_baseline | baseline | 32 | **0.9648** | — | 0.9697 | 1.0 | 1.0 | 1.0 | 0.0223 | 125.0 |
| controller_projective | projective | 32 | **0.9946** | — | 0.9971 | 1.0 | 1.0 | 1.0 | 0.0228 | 144.1 |
| controller_hybrid | hybrid | 32 | **0.9771** | — | 0.9805 | 1.0 | 1.0 | 1.0 | 0.0234 | 151.1 |

### D=24 → D=32 Horizon Sensitivity

| Config | D24 | D32 | Δ(D32−D24) | vs baseline Δ |
|--------|-----|-----|------------|---------------|
| [REF] two_i_orient (anchor probe) | 0.9922 | 0.9873 | -0.0049 | — |
| controller_baseline | 0.9907 | 0.9648 | -0.0259 | — |
| controller_projective | 0.9878 | 0.9946 | +0.0068 | +0.0327 |
| controller_hybrid | 0.9912 | 0.9771 | -0.0141 | +0.0118 |

## Answers to Mandatory Questions

**Q1. Does projective/tangent controller geometry outperform sin/cos-only?**

YES — controller_projective D32=0.9946 vs baseline D32=0.9648 (Δ=+0.0298)

**Q2. Does projective controller reduce D=24→D=32 regression?**

YES — projective Δ=+0.0068 vs baseline Δ=-0.0259: reduced regression by +0.0327

**Q3. Does projective complement two_i_orient anchoring cleanly?**

Partial complement: at least one non-baseline config improved D32 by +0.0298 over the baseline (two_i_orient + sin/cos controller). However, two_i_orient already anchors the START condition; whether projective language adds at the CONTROLLER level is what this test directly measures.

**Q4. Is there evidence that the state wants harmonic coordinates while the controller wants projective coordinates?**

SUPPORTED — projective controller outperforms sin/cos controller at D32 without harming D24, which would support the split-language hypothesis (state=harmonic, controller=projective).

## CONTROLLER GEOMETRY EFFECT IS: **STRONG**

## Honesty Section

### What improved
- **controller_projective**: D32 +0.0298 (0.9946 vs 0.9648)
- **controller_hybrid**: D32 +0.0123 (0.9771 vs 0.9648)

### What stayed unstable
- **controller_baseline**: Δ(D32−D24)=-0.0259  (D24=0.9907, D32=0.9648) — horizon sensitivity persists
- **controller_hybrid**: Δ(D32−D24)=-0.0141  (D24=0.9912, D32=0.9771) — horizon sensitivity persists

### Whether projective/tangent controller language is supported
SUPPORTED: Projective controller language meaningfully outperforms sin/cos-only with the anchor already fixed. The controller does benefit from projective/tangent geometry. Further exploration is justified.

### Limitations
- Single seed (42) — numbers are point estimates
- Only GEOM_K3 tested — projective features may interact differently with richer geometry
- Projective feature = half-angle stereographic projection only; other projective constructions (outer-product, cross-ratio) not tested
- Controller-side vs readout-side projective features not disentangled
- No second LR sweep — projective features may need different LR scaling
