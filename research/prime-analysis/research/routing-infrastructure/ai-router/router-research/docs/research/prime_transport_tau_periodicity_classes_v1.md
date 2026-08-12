# Prime Transport — Tau-Holonomy Periodicity-Class Audit

**Audit type:** Read-only bounded periodicity enumeration
**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`
**Total surface states:** 525,355

---

## Sample and method

| Parameter | Value |
|---|---|
| Sample reuse | **Yes** — exact same seeds as prior tau orbit audits |
| Samples per stratum per operator | 500 (sigma_stable + non_sigma_stable) |
| Total seeds per operator | 1000 |
| Orbit cap | 1000 |
| Random seed | 42 |
| Top-k reported | 10 |

**Cycle length** = step at which the full operator-state orbit first recurs.
**Tau return depth** = step at which the tau sub-state first returns to its initial value.
Note: tau return depth always divides cycle length (standard orbit-theory result).

---

## Summary table

| Operator | Cluster | resolved | distinct cycles | norm entropy | top-1 fraction | concentration |
|---|---|---|---|---|---|---|
| T_b | non-transport | 1000/1000 | 11 | 0.289 | 0.227 | dominated by few periods |
| T_x | non-transport | 1000/1000 | 1 | 0.000 | 1.000 | single period |
| T_y | non-transport | 1000/1000 | 20 | 0.358 | 0.286 | moderately dispersed |
| T_c | transport | 1000/1000 | 27 | 0.408 | 0.176 | moderately dispersed |
| T_z' | transport | 1000/1000 | 35 | 0.453 | 0.155 | moderately dispersed |
| T_r* | transport | 1000/1000 | 37 | 0.475 | 0.078 | moderately dispersed |

---

## Per-operator periodicity-class details

### T_b (non-transport)

- Sample: 1000 | Resolved: 1000 | Unresolved: 0
- Distinct observed cycle lengths: **11**
- Normalised cycle-length entropy: **0.289** (0 = single period, 1 = maximally dispersed)
- Concentration assessment: **dominated by few periods**

**Top-10 observed cycle lengths:**

| Rank | Cycle length | Count | Fraction | Bar |
|---|---|---|---|---|
| 1 | 60 | 227 | 0.2270 | █████░░░░░░░░░░░░░░░ |
| 2 | 30 | 193 | 0.1930 | ████░░░░░░░░░░░░░░░░ |
| 3 | 20 | 179 | 0.1790 | ████░░░░░░░░░░░░░░░░ |
| 4 | 15 | 152 | 0.1520 | ███░░░░░░░░░░░░░░░░░ |
| 5 | 120 | 81 | 0.0810 | ██░░░░░░░░░░░░░░░░░░ |
| 6 | 10 | 60 | 0.0600 | █░░░░░░░░░░░░░░░░░░░ |
| 7 | 40 | 40 | 0.0400 | █░░░░░░░░░░░░░░░░░░░ |
| 8 | 25 | 37 | 0.0370 | █░░░░░░░░░░░░░░░░░░░ |
| 9 | 5 | 25 | 0.0250 | ░░░░░░░░░░░░░░░░░░░░ |
| 10 | 105 | 4 | 0.0040 | ░░░░░░░░░░░░░░░░░░░░ |

**Top-5 tau return depths (within this sample):**

| Tau return depth | Count | Fraction |
|---|---|---|
| 6 | 47 | 0.0470 |
| 12 | 41 | 0.0410 |
| 4 | 34 | 0.0340 |
| 3 | 30 | 0.0300 |
| 18 | 22 | 0.0220 |

### T_x (non-transport)

- Sample: 1000 | Resolved: 1000 | Unresolved: 0
- Distinct observed cycle lengths: **1**
- Normalised cycle-length entropy: **0.000** (0 = single period, 1 = maximally dispersed)
- Concentration assessment: **single period**

**Top-1 observed cycle lengths:**

| Rank | Cycle length | Count | Fraction | Bar |
|---|---|---|---|---|
| 1 | 6 | 1000 | 1.0000 | ████████████████████ |

**Top-5 tau return depths (within this sample):**

| Tau return depth | Count | Fraction |
|---|---|---|
| 6 | 343 | 0.3430 |
| 4 | 26 | 0.0260 |
| 5 | 24 | 0.0240 |
| 7 | 22 | 0.0220 |
| 8 | 20 | 0.0200 |

### T_y (non-transport)

- Sample: 1000 | Resolved: 1000 | Unresolved: 0
- Distinct observed cycle lengths: **20**
- Normalised cycle-length entropy: **0.358** (0 = single period, 1 = maximally dispersed)
- Concentration assessment: **moderately dispersed**

**Top-10 observed cycle lengths:**

| Rank | Cycle length | Count | Fraction | Bar |
|---|---|---|---|---|
| 1 | 6 | 286 | 0.2860 | ██████░░░░░░░░░░░░░░ |
| 2 | 12 | 135 | 0.1350 | ███░░░░░░░░░░░░░░░░░ |
| 3 | 18 | 110 | 0.1100 | ██░░░░░░░░░░░░░░░░░░ |
| 4 | 30 | 59 | 0.0590 | █░░░░░░░░░░░░░░░░░░░ |
| 5 | 2 | 52 | 0.0520 | █░░░░░░░░░░░░░░░░░░░ |
| 6 | 42 | 51 | 0.0510 | █░░░░░░░░░░░░░░░░░░░ |
| 7 | 4 | 43 | 0.0430 | █░░░░░░░░░░░░░░░░░░░ |
| 8 | 8 | 39 | 0.0390 | █░░░░░░░░░░░░░░░░░░░ |
| 9 | 36 | 37 | 0.0370 | █░░░░░░░░░░░░░░░░░░░ |
| 10 | 24 | 34 | 0.0340 | █░░░░░░░░░░░░░░░░░░░ |

**Top-5 tau return depths (within this sample):**

| Tau return depth | Count | Fraction |
|---|---|---|
| 3 | 72 | 0.0720 |
| 2 | 48 | 0.0480 |
| 8 | 37 | 0.0370 |
| 6 | 37 | 0.0370 |
| 7 | 34 | 0.0340 |

### T_c (transport)

- Sample: 1000 | Resolved: 1000 | Unresolved: 0
- Distinct observed cycle lengths: **27**
- Normalised cycle-length entropy: **0.408** (0 = single period, 1 = maximally dispersed)
- Concentration assessment: **moderately dispersed**

**Top-10 observed cycle lengths:**

| Rank | Cycle length | Count | Fraction | Bar |
|---|---|---|---|---|
| 1 | 2 | 176 | 0.1760 | ████░░░░░░░░░░░░░░░░ |
| 2 | 9 | 127 | 0.1270 | ███░░░░░░░░░░░░░░░░░ |
| 3 | 8 | 106 | 0.1060 | ██░░░░░░░░░░░░░░░░░░ |
| 4 | 27 | 65 | 0.0650 | █░░░░░░░░░░░░░░░░░░░ |
| 5 | 4 | 58 | 0.0580 | █░░░░░░░░░░░░░░░░░░░ |
| 6 | 18 | 57 | 0.0570 | █░░░░░░░░░░░░░░░░░░░ |
| 7 | 6 | 50 | 0.0500 | █░░░░░░░░░░░░░░░░░░░ |
| 8 | 36 | 40 | 0.0400 | █░░░░░░░░░░░░░░░░░░░ |
| 9 | 96 | 35 | 0.0350 | █░░░░░░░░░░░░░░░░░░░ |
| 10 | 12 | 33 | 0.0330 | █░░░░░░░░░░░░░░░░░░░ |

**Top-5 tau return depths (within this sample):**

| Tau return depth | Count | Fraction |
|---|---|---|
| 6 | 17 | 0.0170 |
| 2 | 10 | 0.0100 |
| 12 | 9 | 0.0090 |
| 5 | 8 | 0.0080 |
| 16 | 8 | 0.0080 |

### T_z' (transport)

- Sample: 1000 | Resolved: 1000 | Unresolved: 0
- Distinct observed cycle lengths: **35**
- Normalised cycle-length entropy: **0.453** (0 = single period, 1 = maximally dispersed)
- Concentration assessment: **moderately dispersed**

**Top-10 observed cycle lengths:**

| Rank | Cycle length | Count | Fraction | Bar |
|---|---|---|---|---|
| 1 | 6 | 155 | 0.1550 | ███░░░░░░░░░░░░░░░░░ |
| 2 | 2 | 85 | 0.0850 | ██░░░░░░░░░░░░░░░░░░ |
| 3 | 60 | 71 | 0.0710 | █░░░░░░░░░░░░░░░░░░░ |
| 4 | 21 | 57 | 0.0570 | █░░░░░░░░░░░░░░░░░░░ |
| 5 | 9 | 50 | 0.0500 | █░░░░░░░░░░░░░░░░░░░ |
| 6 | 7 | 49 | 0.0490 | █░░░░░░░░░░░░░░░░░░░ |
| 7 | 10 | 45 | 0.0450 | █░░░░░░░░░░░░░░░░░░░ |
| 8 | 57 | 44 | 0.0440 | █░░░░░░░░░░░░░░░░░░░ |
| 9 | 144 | 41 | 0.0410 | █░░░░░░░░░░░░░░░░░░░ |
| 10 | 54 | 39 | 0.0390 | █░░░░░░░░░░░░░░░░░░░ |

**Top-5 tau return depths (within this sample):**

| Tau return depth | Count | Fraction |
|---|---|---|
| 1 | 15 | 0.0150 |
| 5 | 14 | 0.0140 |
| 12 | 13 | 0.0130 |
| 6 | 13 | 0.0130 |
| 30 | 9 | 0.0090 |

### T_r* (transport)

- Sample: 1000 | Resolved: 1000 | Unresolved: 0
- Distinct observed cycle lengths: **37**
- Normalised cycle-length entropy: **0.475** (0 = single period, 1 = maximally dispersed)
- Concentration assessment: **moderately dispersed**

**Top-10 observed cycle lengths:**

| Rank | Cycle length | Count | Fraction | Bar |
|---|---|---|---|---|
| 1 | 75 | 78 | 0.0780 | ██░░░░░░░░░░░░░░░░░░ |
| 2 | 519 | 54 | 0.0540 | █░░░░░░░░░░░░░░░░░░░ |
| 3 | 76 | 54 | 0.0540 | █░░░░░░░░░░░░░░░░░░░ |
| 4 | 234 | 53 | 0.0530 | █░░░░░░░░░░░░░░░░░░░ |
| 5 | 51 | 53 | 0.0530 | █░░░░░░░░░░░░░░░░░░░ |
| 6 | 50 | 52 | 0.0520 | █░░░░░░░░░░░░░░░░░░░ |
| 7 | 27 | 52 | 0.0520 | █░░░░░░░░░░░░░░░░░░░ |
| 8 | 237 | 51 | 0.0510 | █░░░░░░░░░░░░░░░░░░░ |
| 9 | 93 | 50 | 0.0500 | █░░░░░░░░░░░░░░░░░░░ |
| 10 | 8 | 46 | 0.0460 | █░░░░░░░░░░░░░░░░░░░ |

**Top-5 tau return depths (within this sample):**

| Tau return depth | Count | Fraction |
|---|---|---|
| 2 | 14 | 0.0140 |
| 27 | 12 | 0.0120 |
| 5 | 12 | 0.0120 |
| 50 | 12 | 0.0120 |
| 8 | 12 | 0.0120 |

---

## Cluster-level periodicity comparison

| Metric | non-transport {T_b, T_x, T_y} | transport {T_c, T_z', T_r*} |
|---|---|---|
| mean distinct cycle lengths | 10.667 | 33 |
| mean normalised entropy     | 0.216  | 0.445 |
| mean top-1 fraction         | 0.504     | 0.136 |

---

## Interpretation

### Concentration vs dispersion

- **T_b** (non-transport): dominated by few periods (11 distinct lengths; top period = 60 at 0.227 of sample; entropy = 0.289)
- **T_x** (non-transport): single period (1 distinct lengths; top period = 6 at 1.000 of sample; entropy = 0.000)
- **T_y** (non-transport): moderately dispersed (20 distinct lengths; top period = 6 at 0.286 of sample; entropy = 0.358)
- **T_c** (transport): moderately dispersed (27 distinct lengths; top period = 2 at 0.176 of sample; entropy = 0.408)
- **T_z'** (transport): moderately dispersed (35 distinct lengths; top period = 6 at 0.155 of sample; entropy = 0.453)
- **T_r*** (transport): moderately dispersed (37 distinct lengths; top period = 75 at 0.078 of sample; entropy = 0.475)

### Transport vs non-transport periodicity structure

The transport cluster has **higher cycle-length entropy** than the non-transport cluster — transport operators are more broadly dispersed across periodicity classes. The non-transport cluster tends toward more concentrated, repeatable cycle structures.

### Whether any operator is dominated by a single period

- **T_x** has 1.000 of resolved orbits with cycle length 6 — this operator is **dominated by a single period**.

### Observation on tau return depth vs cycle length

Since tau return depth divides cycle length, operators with short tau return depths have tau structures that are 'fast-cycling' components of the full orbit. Operators with tau return depths close to cycle length have tau that tracks nearly the full orbit period before resetting.

- **T_b**: median cycle=30.0, median tau-return=18, ratio=0.600 (tau tracks ~full orbit)
- **T_x**: median cycle=6.0, median tau-return=6, ratio=1.000 (tau tracks ~full orbit)
- **T_y**: median cycle=12.0, median tau-return=9.0, ratio=0.750 (tau tracks ~full orbit)
- **T_c**: median cycle=9.0, median tau-return=18.0, ratio=2.000 (tau tracks ~full orbit)
- **T_z'**: median cycle=21.0, median tau-return=27, ratio=1.286 (tau tracks ~full orbit)
- **T_r***: median cycle=75.0, median tau-return=50.0, ratio=0.667 (tau tracks ~full orbit)

---

## Honesty section

| Question | Answer |
|---|---|
| Were any files modified? | **No** |
| Were any operators rebuilt in this step? | **No** |
| Is full exact spin_H solved? | **No** |

---

## Recommended next step

Run a bounded **tau-periodicity arithmetic-factor test**: for each operator, test whether the most commonly observed cycle lengths are divisors or small multiples of the primorial moduli already defined in the repo (e.g., `W_210`, `W_2310`, `W_30030`).  This directly tests the arithmetic-constraint hypothesis suggested by the concentrated periodicity structure observed here.
