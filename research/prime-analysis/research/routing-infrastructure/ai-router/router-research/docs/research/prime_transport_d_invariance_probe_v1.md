# Prime Transport D-Invariance Probe v1

**Branch:** d_invariance_probe_v1  
**Mechanism:** H3 phase readout — strictly train-free, no weights, no label lookup

## Question

Is D irrelevant for correctness (pure runtime knob), or must it scale with input complexity?

## Result: D_invariance = YES

| D | accuracy | H3_error | phase_error | runtime_ms |
|---|----------|----------|-------------|------------|
| 0 | 1.0 | 0.00e+00 | 2.30e-07 | 0.047 |
| 1 | 1.0 | 0.00e+00 | 2.30e-07 | 7.157 |
| 2 | 1.0 | 0.00e+00 | 2.30e-07 | 2.026 |
| 3 | 1.0 | 0.00e+00 | 2.30e-07 | 2.187 |
| 5 | 1.0 | 0.00e+00 | 2.30e-07 | 3.454 |
| 8 | 1.0 | 0.00e+00 | 2.30e-07 | 5.886 |
| 10 | 1.0 | 0.00e+00 | 2.30e-07 | 8.947 |
| 12 | 1.0 | 0.00e+00 | 2.30e-07 | 12.46 |
| 15 | 1.0 | 0.00e+00 | 2.30e-07 | 14.334 |
| 20 | 1.0 | 0.00e+00 | 2.30e-07 | 17.677 |
| 24 | 1.0 | 0.00e+00 | 2.30e-07 | 21.142 |
| 32 | 1.0 | 0.00e+00 | 2.30e-07 | 27.78 |
| 36 | 1.0 | 0.00e+00 | 2.30e-07 | 34.383 |
| 50 | 1.0 | 0.00e+00 | 2.30e-07 | 39.879 |
| 64 | 1.0 | 0.00e+00 | 2.30e-07 | 49.174 |
| 100 | 1.0 | 0.00e+00 | 2.30e-07 | 90.419 |

## D_min Per Class

| class | D_min |
|-------|-------|
| 0 | 0 |
| 1 | 0 |
| 2 | 0 |
| 3 | 0 |

D_min = 0 for all classes.
D_min=0 means H3 is already present in the initial geometric state. No trajectory steps are required for class extraction.

## Adaptive D Evaluation

| formula | D | accuracy | H3_error |
|---------|---|----------|----------|
| k1_x_cycle | 12 | 1.0 | 0.00e+00 |
| k2_x_cycle | 24 | 1.0 | 0.00e+00 |
| k1_x_harmonics | 3 | 1.0 | 0.00e+00 |
| k2_x_harmonics | 6 | 1.0 | 0.00e+00 |
| k1_x_log_pool | 8 | 1.0 | 0.00e+00 |
| k2_x_log_pool | 17 | 1.0 | 0.00e+00 |
| d_min_stability | 1 | 1.0 | 0.00e+00 |

All adaptive D formulas achieve accuracy=1.0.
They are interchangeable because D_min=0.

## Scaling Law

**D_min = constant = 0**

H3 is preserved at D=0 (initial state contains the class invariant). D_min does not scale with cycle_length, n_harmonics, log(pool_size), or any other input complexity metric. All adaptive D formulas achieve identical accuracy (1.0) because the invariant is never degraded by transport steps.

### What D actually controls

D controls trajectory diversity (h1 phase coverage) and runtime cost only. For D>=1, each step updates h1 (fundamental) while preserving h2 and h3 exactly. The class readout is independent of how many steps were taken.

## Input Complexity Metrics

| metric | value |
|--------|-------|
| h3_energy_mean | 1.0 |
| h3_energy_std | 0.0 |
| h2_energy_mean | 1.0 |
| h1_phase_circ_var | 0.816966 |
| cycle_length | 12 |
| n_harmonics | 3 |
| partition_period | 4 |
| pool_size | 4000 |
| log_pool_size | 8.294 |

Note: cycle_length, n_harmonics, partition_period are constant across all samples
for this task. Per-sample variation exists only in h1_phase trajectory (which
does not affect H3 or class extraction).

## Conclusion

**D is a pure runtime knob. The H3 harmonic invariant (tau_final[:,10:12]) equals tau_init[:,10:12] to machine precision for ALL D >= 0, because apply_split_transport(eps_high=1.0) assigns zero weight to new transport values for h>=2. D_min = 0 for all input classes and complexity levels. No adaptive D formula is needed: D=1 (operationally) is sufficient. D affects only the h1 trajectory diversity and wall-clock cost, which scale linearly with D. The class is encoded in the initial geometric state and is never disturbed by subsequent transport steps.**
