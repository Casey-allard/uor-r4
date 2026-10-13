# Prime Transport Displacement Audit v1

## Purpose

Measure per-operator state displacement and pairwise image-set divergence
on the bounded v25 surface.  Determine whether transport operators produce
systematically larger displacement and lower image-set overlap than
non-transport operators.

This is a read-only audit.  No files were modified.  No operators were rebuilt.

---

## Surface and operators

| Item | Value |
|------|-------|
| Surface | `geometry_native_operator_model_v25.py`, depth=8 |
| State count (N) | 525,355 |
| Displacement metric | count of top-level fields changed (max=10): `b`, `phi`, `r`, `composite_compat_class`, `query_semiprime`, `binding_semiprime`, `admissible_transition`, `twist`, `spin_h`, `tau` |

---

## Per-operator displacement summary

| Operator | Cluster | Avg disp | Median | Max | Stdev | Q1 | Q3 | P10 | P90 | Distinct outputs |
|----------|---------|----------|--------|-----|-------|----|----|-----|-----|-----------------|
| `T_b` | non-transport | 2.8914 | 3.0 | 3 | 0.3114 | 3 | 3 | 2 | 3 | 393,415 (74.89%) |
| `T_x` | non-transport | 3.9194 | 4.0 | 4 | 0.2722 | 4 | 4 | 4 | 4 | 284,021 (54.06%) |
| `T_y` | non-transport | 2.5834 | 3.0 | 3 | 0.5067 | 2 | 3 | 2 | 3 | 366,479 (69.76%) |
| `T_c` | transport | 2.5942 | 3.0 | 3 | 0.4982 | 2 | 3 | 2 | 3 | 285,632 (54.37%) |
| `T_z'` | transport | 2.5932 | 3.0 | 3 | 0.4998 | 2 | 3 | 2 | 3 | 353,161 (67.22%) |
| `T_r*` | transport | 3.2775 | 3.0 | 4 | 0.6837 | 3 | 4 | 2 | 4 | 359,228 (68.38%) |

### Displacement histograms (displacement value → state count)

- **`T_b`** (non-transport): 1:44  2:56,973  3:468,338
- **`T_x`** (non-transport): 3:42,323  4:483,032
- **`T_y`** (non-transport): 1:3,603  2:211,651  3:310,101
- **`T_c`** (transport): 1:1,868  2:209,462  3:314,025
- **`T_z'`** (transport): 1:2,221  2:209,292  3:313,842
- **`T_r*`** (transport): 1:681  2:68,080  3:241,364  4:215,230

---

## Transport vs non-transport displacement comparison

| Statistic | Non-transport mean | Transport mean | Transport higher? |
|-----------|-------------------|----------------|-------------------|
| Avg displacement | 3.1314 | 2.8216 | **no** |
| Median displacement | 3.0 | 3.0 | **no** |
| Max displacement | 4 | 4 | **no** |
| Distinct output fraction | 0.6624 | 0.6332 | no (transport lower) |

---

## Pairwise image-set overlap and divergence

**Overlap fraction** = |image_i ∩ image_j| / N.  **Divergence** = 1 − overlap_fraction.  **Jaccard** = |image_i ∩ image_j| / |image_i ∪ image_j|.

### Non-transport intra-cluster pairs

| Pair | Image i size | Image j size | Overlap | Overlap frac | Divergence | Jaccard |
|------|-------------|-------------|---------|--------------|------------|---------|
| `{T_b, T_x}` | 393,415 | 284,021 | 177,234 | 0.3374 | 0.6626 | 0.3543 |
| `{T_b, T_y}` | 393,415 | 366,479 | 224,053 | 0.4265 | 0.5735 | 0.4181 |
| `{T_x, T_y}` | 284,021 | 366,479 | 162,531 | 0.3094 | 0.6906 | 0.3331 |
| **Non-transport mean** | — | — | — | **0.3577** | **0.6423** | — |

### Transport intra-cluster pairs

| Pair | Image i size | Image j size | Overlap | Overlap frac | Divergence | Jaccard |
|------|-------------|-------------|---------|--------------|------------|---------|
| `{T_c, T_z'}` | 285,632 | 353,161 | 156,815 | 0.2985 | 0.7015 | 0.3254 |
| `{T_c, T_r*}` | 285,632 | 359,228 | 160,066 | 0.3047 | 0.6953 | 0.3302 |
| `{T_z', T_r*}` | 353,161 | 359,228 | 201,876 | 0.3843 | 0.6157 | 0.3954 |
| **Transport mean** | — | — | — | **0.3291** | **0.6709** | — |

### Cross-cluster pairs

| Pair | Image i size | Image j size | Overlap | Overlap frac | Divergence | Jaccard |
|------|-------------|-------------|---------|--------------|------------|---------|
| `{T_b, T_c}` | 393,415 | 285,632 | 172,883 | 0.3291 | 0.6709 | 0.3416 |
| `{T_b, T_z'}` | 393,415 | 353,161 | 215,978 | 0.4111 | 0.5889 | 0.4070 |
| `{T_b, T_r*}` | 393,415 | 359,228 | 220,017 | 0.4188 | 0.5812 | 0.4131 |
| `{T_x, T_c}` | 284,021 | 285,632 | 148,987 | 0.2836 | 0.7164 | 0.3542 |
| `{T_x, T_z'}` | 284,021 | 353,161 | 158,863 | 0.3024 | 0.6976 | 0.3321 |
| `{T_x, T_r*}` | 284,021 | 359,228 | 175,079 | 0.3333 | 0.6667 | 0.3740 |
| `{T_y, T_c}` | 366,479 | 285,632 | 162,147 | 0.3086 | 0.6914 | 0.3309 |
| `{T_y, T_z'}` | 366,479 | 353,161 | 202,719 | 0.3859 | 0.6141 | 0.3922 |
| `{T_y, T_r*}` | 366,479 | 359,228 | 204,723 | 0.3897 | 0.6103 | 0.3930 |
| **Cross-cluster mean** | — | — | — | **0.3514** | **0.6486** | — |

---

## Conclusion: 'transport = fast displacement' hypothesis

**PARTIALLY SUPPORTED.** Transport operators produce lower intra-cluster overlap (0.3291 vs 0.3577) but do NOT show higher average displacement (transport 2.8216, non-transport 3.1314).

### Supporting detail

- Non-transport cluster mean displacement: **3.1314** fields/state
- Transport cluster mean displacement: **2.8216** fields/state
- Ratio (transport / non-transport): **0.90×**
- Non-transport intra-cluster mean overlap fraction: **0.3577**
- Transport intra-cluster mean overlap fraction: **0.3291**
- Cross-cluster mean overlap fraction: **0.3514**

The displacement metric counts how many of the 10 top-level state fields
differ between input and output for each operator application.  It is an
integer in [0, 10]; a value of 0 means the operator acted as the identity
on that state.

---

## Honesty section

| Question | Answer |
|----------|--------|
| Were any files modified? | **no** — read-only audit on the accepted v25 surface |
| Were any operators rebuilt? | **no** |
| Is full exact `spin_H` solved? | **no** — full exact spin_H remains an open problem |

---

## Next step

Run a bounded fixed-point census on the v25 surface: for each operator,
count and characterise the states `s` where `O(s) == s` (displacement = 0).
Fixed points reveal the invariant sub-spaces of each operator and may
identify structurally degenerate states that do not participate in transport.

Do not benchmark.  Do not rebuild any operator.  Do not touch any core file.
