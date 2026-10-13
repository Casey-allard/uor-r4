# Prime Transport — Tau-Return-Depth Histogram Audit

**Audit type:** Read-only bounded histogram characterization
**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`
**Total surface states:** 525,355

---

## Sample and method

| Parameter | Value |
|---|---|
| Sample reuse | **Yes** — exact same seeds as `run_tau_orbit_depth_audit_v1.py` |
| Samples per stratum per operator | 500 (sigma_stable + non_sigma_stable) |
| Total seeds per operator | 1000 |
| Orbit cap | 1000 |
| Random seed | 42 |

### Bin edges

| Bin label | Range (tau return depth) | Interpretation |
|---|---|---|
| 1–4       | [1, 5)    | very shallow |
| 5–14      | [5, 15)   | shallow |
| 15–49     | [15, 50)  | medium |
| 50–149    | [50, 150) | deep |
| 150+      | [150, 1000] | very deep |
| no_tau_return | cycle resolved; tau never returned | non-returning |

---

## Per-operator histograms

### T_b (non-transport)

Sample: 1000 | tau returns: 619 | avg depth: 23.84 | median: 18 | stdev: 21.63

| Bin | count | fraction | bar |
|---|---|---|---|
| 1–4             |    76 | 0.0760 | ██░░░░░░░░░░░░░░░░░░░░░░░ |
| 5–14            |   193 | 0.1930 | █████░░░░░░░░░░░░░░░░░░░░ |
| 15–49           |   275 | 0.2750 | ███████░░░░░░░░░░░░░░░░░░ |
| 50–149          |    75 | 0.0750 | ██░░░░░░░░░░░░░░░░░░░░░░░ |
| 150+            |     0 | 0.0000 | ░░░░░░░░░░░░░░░░░░░░░░░░░ |
| no_tau_return   |   381 | 0.3810 | ██████████░░░░░░░░░░░░░░░ |

### T_x (non-transport)

Sample: 1000 | tau returns: 493 | avg depth: 5.93 | median: 6 | stdev: 1.52

| Bin | count | fraction | bar |
|---|---|---|---|
| 1–4             |    61 | 0.0610 | ██░░░░░░░░░░░░░░░░░░░░░░░ |
| 5–14            |   432 | 0.4320 | ███████████░░░░░░░░░░░░░░ |
| 15–49           |     0 | 0.0000 | ░░░░░░░░░░░░░░░░░░░░░░░░░ |
| 50–149          |     0 | 0.0000 | ░░░░░░░░░░░░░░░░░░░░░░░░░ |
| 150+            |     0 | 0.0000 | ░░░░░░░░░░░░░░░░░░░░░░░░░ |
| no_tau_return   |   507 | 0.5070 | █████████████░░░░░░░░░░░░ |

### T_y (non-transport)

Sample: 1000 | tau returns: 606 | avg depth: 11.61 | median: 9.00 | stdev: 9.94

| Bin | count | fraction | bar |
|---|---|---|---|
| 1–4             |   161 | 0.1610 | ████░░░░░░░░░░░░░░░░░░░░░ |
| 5–14            |   275 | 0.2750 | ███████░░░░░░░░░░░░░░░░░░ |
| 15–49           |   165 | 0.1650 | ████░░░░░░░░░░░░░░░░░░░░░ |
| 50–149          |     5 | 0.0050 | ░░░░░░░░░░░░░░░░░░░░░░░░░ |
| 150+            |     0 | 0.0000 | ░░░░░░░░░░░░░░░░░░░░░░░░░ |
| no_tau_return   |   394 | 0.3940 | ██████████░░░░░░░░░░░░░░░ |

### T_c (transport)

Sample: 1000 | tau returns: 256 | avg depth: 27.44 | median: 18.00 | stdev: 27.21

| Bin | count | fraction | bar |
|---|---|---|---|
| 1–4             |    30 | 0.0300 | █░░░░░░░░░░░░░░░░░░░░░░░░ |
| 5–14            |    74 | 0.0740 | ██░░░░░░░░░░░░░░░░░░░░░░░ |
| 15–49           |   110 | 0.1100 | ███░░░░░░░░░░░░░░░░░░░░░░ |
| 50–149          |    42 | 0.0420 | █░░░░░░░░░░░░░░░░░░░░░░░░ |
| 150+            |     0 | 0.0000 | ░░░░░░░░░░░░░░░░░░░░░░░░░ |
| no_tau_return   |   744 | 0.7440 | ███████████████████░░░░░░ |

### T_z' (transport)

Sample: 1000 | tau returns: 367 | avg depth: 33.50 | median: 27 | stdev: 28.69

| Bin | count | fraction | bar |
|---|---|---|---|
| 1–4             |    34 | 0.0340 | █░░░░░░░░░░░░░░░░░░░░░░░░ |
| 5–14            |    85 | 0.0850 | ██░░░░░░░░░░░░░░░░░░░░░░░ |
| 15–49           |   152 | 0.1520 | ████░░░░░░░░░░░░░░░░░░░░░ |
| 50–149          |    95 | 0.0950 | ██░░░░░░░░░░░░░░░░░░░░░░░ |
| 150+            |     1 | 0.0010 | ░░░░░░░░░░░░░░░░░░░░░░░░░ |
| no_tau_return   |   633 | 0.6330 | ████████████████░░░░░░░░░ |

### T_r* (transport)

Sample: 1000 | tau returns: 638 | avg depth: 67.84 | median: 50.00 | stdev: 64.05

| Bin | count | fraction | bar |
|---|---|---|---|
| 1–4             |    36 | 0.0360 | █░░░░░░░░░░░░░░░░░░░░░░░░ |
| 5–14            |    84 | 0.0840 | ██░░░░░░░░░░░░░░░░░░░░░░░ |
| 15–49           |   196 | 0.1960 | █████░░░░░░░░░░░░░░░░░░░░ |
| 50–149          |   259 | 0.2590 | ██████░░░░░░░░░░░░░░░░░░░ |
| 150+            |    63 | 0.0630 | ██░░░░░░░░░░░░░░░░░░░░░░░ |
| no_tau_return   |   362 | 0.3620 | █████████░░░░░░░░░░░░░░░░ |

---

## Summary statistics

| Operator | Cluster | avg depth | median | stdev | mode bin | no_tau_return fraction |
|---|---|---|---|---|---|---|
| T_b | non-transport | 23.84 | 18 | 21.63 | 15–49 | 0.3810 |
| T_x | non-transport | 5.93 | 6 | 1.52 | 5–14 | 0.5070 |
| T_y | non-transport | 11.61 | 9.00 | 9.94 | 5–14 | 0.3940 |
| T_c | transport | 27.44 | 18.00 | 27.21 | 15–49 | 0.7440 |
| T_z' | transport | 33.50 | 27 | 28.69 | 15–49 | 0.6330 |
| T_r* | transport | 67.84 | 50.00 | 64.05 | 50–149 | 0.3620 |

---

## Cluster-level bin comparison

Mean bin fraction across each cluster:

| Bin | non-transport mean | transport mean | direction |
|---|---|---|---|
| 1–4             | 0.0993 | 0.0333 | non-transport higher |
| 5–14            | 0.3000 | 0.0810 | non-transport higher |
| 15–49           | 0.1467 | 0.1527 | similar |
| 50–149          | 0.0267 | 0.1320 | transport higher |
| 150+            | 0.0000 | 0.0213 | transport higher |
| no_tau_return   | 0.4273 | 0.5797 | transport higher |

---

## Interpretation

### Distribution shapes

- **T_b** (non-transport): bimodal / broadly spread across [15–49] and [5–14]  (avg=23.84, stdev=21.63)
- **T_x** (non-transport): right-skewed, concentrated in [5–14] with tail in [1–4]  (avg=5.93, stdev=1.52)
- **T_y** (non-transport): right-skewed, concentrated in [5–14] with tail in [15–49]  (avg=11.61, stdev=9.94)
- **T_c** (transport): bimodal / broadly spread across [15–49] and [5–14]  (avg=27.44, stdev=27.21)
- **T_z'** (transport): bimodal / broadly spread across [15–49] and [50–149]  (avg=33.50, stdev=28.69)
- **T_r*** (transport): bimodal / broadly spread across [50–149] and [15–49]  (avg=67.84, stdev=64.05)

### Transport vs non-transport cluster

Non-transport combined shallow (bins 1–14) mean fraction: **0.3993**
Transport combined shallow (bins 1–14) mean fraction:     **0.1143**

Non-transport combined deep (bins 50+) mean fraction: **0.0267**
Transport combined deep (bins 50+) mean fraction:     **0.1533**

Transport operators are **heavier-tailed** in the deep tau-depth bins. The transport cluster systematically produces deeper tau excursions, consistent with the cluster mean avg-depth finding from the prior audit.

### T_r* distribution shape vs T_c and T_z'

T_r* avg depth (67.84) vs T_c (27.44) vs T_z' (33.50).
Deep-bin (50+) fractions: T_r*=0.3220, T_c=0.0420, T_z'=0.0960.

**T_r* is structurally distinct within the transport cluster**, not merely deeper on average: it has a substantially larger fraction of deep-bin returns compared to T_c and T_z'. T_c and T_z' are more concentrated in the medium bin.

---

## Honesty section

| Question | Answer |
|---|---|
| Were any files modified? | **No** |
| Were any operators rebuilt in this step? | **No** |
| Is full exact spin_H solved? | **No** |

---

## Recommended next step

Run a bounded **tau-holonomy periodicity class audit**: for each operator, enumerate the distinct cycle lengths observed in the sampled orbits and determine whether cycle lengths cluster around specific values (e.g., factors of the primorial used to construct the surface).  This would test whether the tau orbit structure is arithmetically constrained by the surface construction or is effectively aperiodic within the bounded surface.
