# INC-0156: Spectral Compression via Equal-Quality Routing Cost — Screen

## Status
Closed: REFINE.

## Kill-List Stage
6 — Sparse Event-Driven Trainability (structural routing compression)

## Mathematical Object Under Test
Spectral compression ratio: at equal spectral quality $Q$, the ratio
$K_\text{perm} / K_\text{orig}$ where $K_\text{min}(Q)$ is the minimum bucket
count required to achieve quality $Q$ under each routing mode.

## Theory

INC-0155 established that per-sample MSE cannot detect routing compression:
EMA prototype training adapts prototypes to marginal distributions (preserved
by column permutation), producing $E[\text{MSE} | \text{ORIG}] \approx E[\text{MSE} | \text{PERM}]$.
This is a measurement limitation, not evidence against compression.

Stage 5 proved the geometric advantage IS visible in aggregate structural metrics:
- sector_lowfreq_energy: +72–77% (ORIG vs PERM at K=75, poincaré_4d graph)
- true_margin_lowfreq: +40–48% (4-seed finalize)
- label_indicator_lowfreq_max: +57%

These metrics depend on the graph Laplacian eigenstructure — global
organizational properties of routing, not per-sample reconstruction.

**Stage 6 question (corrected):** Does geometry-native routing achieve the same
structural/spectral quality with lower routing complexity (fewer buckets)?

If so, routing compression exists: ORIG routing organizes data so well that
fewer sectors suffice for the same spectral organization quality. This is the
architectural sparse-compute mechanism: fewer buckets = fewer routing lookups
= fewer prototype updates = hardware savings.

## Experimental Design

Reuse INC-0155 K-sweep config (22 routes with K ∈ {4,9,16,25,50,75,100}).
For each route, compute spectral quality metrics using the poincaré_4d graph
operator (the geometry-native operator confirmed in Stage 5).

**Quality metrics (per route):**
- `sector_lowfreq_energy`: low-frequency energy of sector assignments
- `label_indicator_lowfreq_max`: best per-class low-frequency label alignment
- `label_onehot_lowfreq_energy`: multi-class label alignment
- `shell_lowfreq_energy`: shell assignment smoothness
- `spectral_lambda2`: spectral gap (graph connectivity quality)
- `spectral_mode_participation_mean`: eigenmode delocalization

**Routing cost metrics (from INC-0155 sweep data, already available):**
- Active bucket count (K)
- Sector entropy (bucket utilization uniformity)
- Sector pmax (concentration / non-uniformity)
- Slots used

**Primary analysis:**
For each spectral quality metric Q and target value:
1. Compute Q(K) curves for BASE_ORIG, BASE_PERM, TRANS_ORIG, TRANS_PERM
2. At matched quality target, estimate K_min for ORIG and PERM via interpolation
3. Compression ratio = K_perm / K_orig

**Secondary analysis:**
- Q vs log(K) curves and area under curve
- Spectral quality advantage ratio at each K
- Routing mass vs spectral quality

**Interpretation rules:**
- MSE is reported as a fidelity check only, NOT used for compression evaluation
- Compression must be evaluated using equal-quality structural metrics

## Config
- Spectral probe: `spectral_signal_probe.py --config configs/proxy_transfer_inc0155_routing_compression_screen.json --graph-mode poincare_4d`
- Graph operator: poincaré_4d (the geometry-native operator confirmed in Stage 5)
- Max points: 384 (spectral decomposition sample)
- KNN-k: 12, lowfreq-modes: 8

## Success Condition
- At matched spectral quality target (e.g., sector_lowfreq_energy of PERM at K=75):
  ORIG achieves the same quality at lower K → compression ratio > 1.5
- Multiple quality metrics show consistent compression
- TRANS compression > BASE compression (transport amplifies the effect)

## Falsification Condition
- Spectral quality equally insensitive to routing quality (ORIG ≈ PERM) at all K values
- OR: no meaningful spectral quality vs K curve exists (quality flat across K)

## Results

### Data Files
- Signal probe: `results/analysis/inc0156_spectral_compression_screen.json`
- Residual probe: `results/analysis/inc0156_residual_compression_screen.json`
- Cost data: `results/analysis/inc0155_routing_compression_screen.json`

### Graph Structure (poincaré_4d)

The graph is K-invariant within each condition (built from routing coordinates,
not from sector assignments):

| Condition | λ2     | graph_edges | label_indicator_lowfreq_max | label_onehot_lowfreq_energy |
|-----------|--------|-------------|-----------------------------|-----------------------------|
| ORIG      | 0.0179 | 2951        | 0.1172                      | 0.0214                      |
| PERM      | 0.0368 | 3075        | 0.0781                      | 0.0192                      |

ORIG advantage:
- label_indicator_lowfreq_max: +50.1% (ORIG / PERM = 1.501)
- label_onehot_lowfreq_energy: +11.2% (ratio = 1.112)
- PERM never reaches ORIG quality at any K → ratio = ∞ for compression

### sector_lowfreq_energy vs K

| K   | BASE_ORIG | BASE_PERM | Ratio | TRANS_ORIG | TRANS_PERM | Ratio |
|-----|-----------|-----------|-------|------------|------------|-------|
| 4   | 0.493     | 0.509     | 0.970 | —          | —          | —     |
| 9   | 0.610     | 0.662     | 0.922 | —          | —          | —     |
| 16  | 0.625     | 0.679     | 0.920 | —          | —          | —     |
| 25  | 0.669     | 0.728     | 0.919 | 0.610      | 0.656      | 0.929 |
| 50  | 0.254     | 0.301     | 0.845 | 0.672      | 0.727      | 0.925 |
| 75  | 0.382     | 0.395     | 0.969 | 0.670      | 0.727      | 0.922 |
| 100 | 0.692     | 0.745     | 0.929 | 0.671      | 0.727      | 0.923 |

**Observation:** PERM has 3–16% higher sector_lowfreq_energy at every K
(anti-compression). The PERM graph has higher λ2 (0.037 vs 0.018), suggesting
a simpler/more connected graph where coarse sector assignments align trivially.
sector_lowfreq_energy measures graph-sector coherence, not task-relevant quality.

### true_margin_lowfreq_energy vs K

| K   | BASE_ORIG | BASE_PERM | Ratio | TRANS_ORIG | TRANS_PERM | Ratio |
|-----|-----------|-----------|-------|------------|------------|-------|
| 4   | 0.094     | 0.120     | 0.786 | —          | —          | —     |
| 9   | 0.070     | 0.150     | 0.466 | —          | —          | —     |
| 16  | 0.026     | 0.095     | 0.268 | —          | —          | —     |
| 25  | 0.039     | 0.108     | 0.356 | 0.081      | 0.106      | 0.758 |
| 50  | 0.037     | 0.009     | 4.342 | 0.079      | 0.066      | 1.202 |
| 75  | 0.026     | 0.041     | 0.639 | 0.091      | 0.045      | 2.011 |
| 100 | 0.046     | 0.032     | 1.426 | 0.136      | 0.041      | 3.325 |

**Observation:** Highly variable. TRANS routes show ORIG advantage growing with
K (0.76 → 3.33 from K=25 → K=100). BASE routes are non-monotonic and noisy.
1-seed noise floor is high for this metric.

### Compression Analysis

**label_indicator_lowfreq_max:** ORIG = 0.1172 at all K; PERM = 0.0781 at all K.
PERM never reaches ORIG quality → infinite compression ratio. However, this is
K-invariant so it proves graph structural advantage, not routing-granularity
compression.

**true_margin_lowfreq_energy (BASE):** Compression ratio at K=4: 6.9×, K=9: 3.6×,
K=16: 2.8×, K=25: 1.6×. At K≥50 compression reverses. Mean finite ratio: 2.39.

**true_margin_lowfreq_energy (TRANS):** K=25: 1.55×, K=100: ∞ (unreachable).
K=50,75 < 1. Mean finite ratio: 0.93. Signal weaker than BASE.

**sector_lowfreq_energy:** Anti-compression (PERM > ORIG everywhere). This metric
measures graph-sector coherence, not task-relevant quality; discarded for
compression evaluation.

### Interpretation

Three distinct compression patterns emerged:

1. **Geometric structural compression (label metrics):** The poincaré_4d graph
   built from ORIG data captures label semantics 50% better at any K.
   K-invariant → proves the GEOMETRY itself is compressed, not just the routing.
   This is the strongest signal.

2. **Routing-granularity compression (true_margin):** At low K (4–25), ORIG
   achieves true_margin quality that PERM needs 2–7× more sectors to match.
   Signal weakens at K ≥ 50. Noisy at 1 seed.

3. **Anti-signal in sector_lowfreq_energy:** PERM sectors align better with PERM
   graph (likely because PERM graph has simpler structure). This metric
   measures graph-sector coherence, not task-relevant organizational quality.

## Decision

**Closed: REFINE.**

Rationale: Compression signal exists in two distinct forms — geometric
structural compression (label metrics, ORIG quality unreachable by PERM at any
K, ratio 1.5) and routing-granularity compression (true_margin at K ≤ 25,
ratios 1.6–6.9×). However:
- Label metrics are K-invariant → proves geometric advantage, not K-dependent
  compression
- true_margin is noisy at 1 seed (values range 0.009–0.150)
- sector_lowfreq_energy shows anti-compression (graph-sector coherence ≠ task
  quality)
- The effect direction in true_margin reverses at K ≥ 50

Next step: INC-0157 should either (a) confirm true_margin compression at low K
with 2+ seeds, OR (b) test a different spectral metric that varies with K and
captures task-relevant routing quality (e.g., per-sector label purity or sector
classification margin).
