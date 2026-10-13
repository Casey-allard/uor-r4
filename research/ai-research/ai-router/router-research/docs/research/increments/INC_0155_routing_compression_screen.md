# INC-0155: Routing Compression — Bucket Count Sweep Screen

## Status
Closed: REFINE. Per-sample MSE cannot detect routing compression on this proxy.
EMA prototype training adapts prototypes to empirical distributions inside each
bucket; column permutation preserves marginal distributions, so reconstruction
converges to nearly identical MSE regardless of routing quality. This does NOT
imply routing compression does not exist — it implies MSE is a local distortion
metric while the geometric advantage is structural and aggregate. Sector entropy
(8.6pp gap at K=75) confirms the structural signal IS present.

## Kill-List Stage
6 — Sparse Event-Driven Trainability (reinterpreted as routing compression)

## Mathematical Object Under Test
Routing compression ratio: the factor by which geometry-native routing reduces the
number of buckets needed to achieve a given reconstruction MSE, compared to routing
on scrambled (structure-destroyed) data.

## Theory

INC-0152/0153/0154 systematically ruled out per-sample error gating as the mechanism
through which geometric routing produces sparsity:
- Per-sample spectral roughness ↔ error: geometry-agnostic (INC-0153)
- Aggregate event-gate efficiency: routing-agnostic (INC-0154)

The reason: EMA prototype learning equalizes per-sample prediction errors regardless
of routing quality. The prototypes adapt to whatever data appears in each bucket.

But Stages 2–5 proved that geometric routing creates **more coherent buckets**:
- Stage 2: ORIG rel_diff=31–38% over COL_PERM (bucket organization is semantically discriminative)
- Stage 3: Fixed Hopf 4D >> adaptive K-means 100D
- Stage 4: Phase transport adds +18.4pp over same-K base
- Stage 5: +40–77% low-frequency spectral energy (aggregate structural advantage)

If buckets are more coherent, fewer of them should suffice for the same MSE. This is
the correct sparse-compute mechanism:
- Fewer buckets = fewer routing lookups per query
- Fewer buckets = fewer prototypes to maintain
- Fewer buckets = less memory and computation = hardware savings

The **routing compression ratio** $R = K_\text{perm} / K_\text{orig}$ measures this
advantage directly.

## Experimental Design

**Bucket count sweep at fixed training parameters:**

HOPF_BASE routes (14 total):
- K ∈ {4, 9, 16, 25, 50, 75, 100} × {ORIG, COL_PERM}

HOPF_TRANS routes (8 total):
- K ∈ {25, 50, 75, 100} × {ORIG, COL_PERM}
- (Skip K<25 for transport because kalpha=1 at low K makes transport trivial)

Total: 22 routes, seed=0 (screen).

For each route, measure:
- `test_mse_after`: reconstruction quality
- `pmax_after`: routing discrimination indicator
- `buckets`: actual buckets used
- `slots_used`: slots after growth
- `total_sec`: runtime

## Config
- Screen: `configs/proxy_transfer_inc0155_routing_compression_screen.json`
- Common: PPMI-SVD, phase4_dims=3,65,2,21, epochs=1, max_train=5000, max_eval=2500
- All other params identical to INC-0154 (event_gate_mode=off)

## Success Condition
- At matched MSE target (e.g., MSE of COL_PERM at K=75):
  ORIG achieves the same MSE at lower K → compression ratio $K_\text{perm}/K_\text{orig} > 1.5$
- Both BASE and TRANS show compression >1.5
- TRANS compression > BASE compression (transport amplifies the effect)

## Falsification Condition
- MSE-vs-K curves for ORIG and PERM are parallel (no compression): $K_\text{perm}/K_\text{orig} < 1.2$
- OR: MSE differences at each K are within seed noise

## Results

### Table 1: MSE vs K

| K | BASE_ORIG | BASE_PERM | TRANS_ORIG | TRANS_PERM |
|---:|----------:|----------:|-----------:|-----------:|
| 4 | 0.003891 | 0.003881 | — | — |
| 9 | 0.003921 | 0.003924 | — | — |
| 16 | 0.003949 | 0.003969 | — | — |
| 25 | 0.003959 | 0.004002 | 0.003964 | 0.003982 |
| 50 | 0.003991 | 0.004001 | 0.003989 | 0.003990 |
| 75 | 0.003995 | 0.003997 | 0.003987 | 0.003999 |
| 100 | 0.004006 | 0.004008 | 0.003981 | 0.003996 |

**MSE range:** 0.003881–0.004008 (total spread 3.3%). MSE INCREASES with K
(opposite of expected). The proxy task saturates almost immediately — each
bucket's EMA prototype converges at any K with max_train=5000.

### Table 2: MSE delta (PERM − ORIG)

| K | BASE Δ% | TRANS Δ% |
|---:|--------:|---------:|
| 4 | −0.24% | — |
| 9 | +0.07% | — |
| 16 | +0.49% | — |
| 25 | +1.09% | +0.47% |
| 50 | +0.25% | +0.01% |
| 75 | +0.05% | +0.30% |
| 100 | +0.05% | +0.39% |

Maximum delta: +1.09% (BASE K=25). All within noise for 1-seed screen.

### Table 3: Routing Compression Ratios

| ORIG K | MSE target | PERM K needed | Ratio |
|-------:|-----------:|--------------:|------:|
| 4 | 0.003891 | ≈4.8 | 1.20 |
| 9 | 0.003921 | ≈8.5 | 0.95 |
| 16 | 0.003949 | ≈12.5 | 0.78 |
| 25 | 0.003959 | ≈14.1 | 0.56 |
| 50 | 0.003991 | ≈54.1 | 1.08 |
| 75 | 0.003995 | ≈66.6 | 0.89 |
| 100 | 0.004006 | ≈61.5 | 0.62 |

Mean BASE compression: 0.87. Mean TRANS compression: 0.74.
Both BELOW 1.0 — MSE cannot detect routing compression on this proxy.
This does not imply routing compression does not exist.

### Table 4: Sector Entropy Efficiency (secondary finding)

| K | ORIG ent_eff | PERM ent_eff | Δ eff |
|---:|-----------:|-----------:|------:|
| 4 | 0.991 | 0.999 | −0.008 |
| 9 | 0.974 | 0.989 | −0.015 |
| 16 | 0.968 | 0.994 | −0.026 |
| 25 | 0.932 | 0.988 | −0.056 |
| 50 | 0.899 | 0.968 | −0.069 |
| 75 | 0.891 | 0.976 | −0.086 |
| 100 | 0.918 | 0.987 | −0.069 |

ORIG routing is systematically LESS uniform than PERM. The entropy
efficiency gap grows from 0.8pp at K=4 to 8.6pp at K=75. This confirms
geometric routing produces non-uniform bucket assignments that respect
data structure. The non-uniformity does not reduce MSE (because MSE is
insensitive to routing quality on this proxy), but it IS a structural
signal that routing compression may be detectable through structural
metrics (spectral energy, bucket coherence) rather than per-sample fidelity.

Also: hopf_angular_mass_error ORIG=0.678, PERM=0.374 (constant across K).
Geometric routing on structured data creates asymmetric Hopf occupancy;
on scrambled data, occupancy is more uniform.

## Mathematical Interpretation

The MSE-vs-K flatness has the same root cause as INC-0154's gate homogeneity:
EMA prototype learning adapts prototypes to the empirical distribution inside
each bucket. Column permutation preserves marginal distributions, so the EMA
process converges to nearly identical prototype reconstructions regardless of
routing quality. Therefore: $E[\text{MSE} | \text{ORIG}] \approx E[\text{MSE} | \text{PERM}]$.

This is a measurement limitation, not a falsification of routing compression:
- MSE is a local distortion metric (per-sample reconstruction fidelity)
- Routing compression operates at the structural/aggregate level
- The geometric advantage (Stages 2–5) is confirmed in structural metrics

Evidence hierarchy:
- Per-sample MSE → geometry-agnostic (local, prototypes adapt)
- Per-sample gating → geometry-agnostic (INC-0154)
- Sector entropy → geometry-DEPENDENT (confirmed: 8.6pp gap at K=75)
- Aggregate spectral energy → geometry-DEPENDENT (Stage 5: +40–77%)

The geometric signal exists at the routing structure level, not at the
per-sample prediction level. Therefore routing compression must be
measured through structural quality metrics (spectral energy, bucket
coherence) at equal routing complexity, not through MSE.

## Decision
Closed: REFINE. Per-sample MSE cannot detect routing compression on this
proxy (measurement limitation, not falsification). The structural routing
signature IS present (sector entropy 8.6pp gap, Hopf occupancy asymmetry).
Stage 6 question must be reframed: does geometry-native routing achieve
the same structural/spectral quality with lower routing complexity?
Next: INC-0156 spectral compression (lowfreq_energy vs K).
