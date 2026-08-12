# Prime Transport — Tau-Periodicity Arithmetic-Factor Test

**Audit type:** Read-only arithmetic alignment test (no orbit recomputation)
**Input:** `prime_transport_tau_periodicity_classes_v1.csv`
**Canonical moduli source:** `geometry_native_spinH_candidate_v3.py`

---

## Canonical arithmetic moduli used

Sourced directly from `geometry_native_spinH_candidate_v3.py`:

| Constant | Value | Role |
|---|---|---|
| `SWAP_PHASE_MODULUS_V3`   | 2  | tau.swap_phase period |
| `COUPLED_PHASE_MODULUS_V3`| 5  | tau.coupled_phase period |
| `TWIST_PHASE_MODULUS_V3`  | 2  | tau.twist_phase period |
| `LIFT_PHASE_MODULUS_V3`   | 12 | tau.lift_phase period |
| LCM of all four           | **60** | maximum fixed-step tau period |
| Product of all four       | **240** | tau state-space cardinality |

Note: `coupled_phase` step size depends on `(query_semiprime × binding_semiprime) % 5`
and `lift_phase` step size depends on `phi` and `spin_h` bits — so the actual orbit
period can exceed 60 for state-dependent stepping.

---

## Relations tested

| Relation | Test | Interpretation |
|---|---|---|
| `divides_60`  | L \| 60   | L is a divisor of the tau LCM |
| `divides_120` | L \| 120  | L is a divisor of 2 × tau LCM |
| `divides_240` | L \| 240  | L is a divisor of tau state-space size |
| `multiple_60` | 60 \| L   | L is a multiple of the tau LCM |
| `mult_2`      | 2 \| L    | L divisible by swap/twist sub-modulus |
| `mult_5`      | 5 \| L    | L divisible by coupled sub-modulus |
| `mult_12`     | 12 \| L   | L divisible by lift sub-modulus |

---

## Null baseline

For each operator, 100,000 random integers were drawn uniformly from
`[1, max_observed_cycle_length]` and the same alignment fractions were computed.
Enrichment ratio = observed_rate / null_rate.
Enrichment ≥ 2.0 is noted as substantial; ≥ 5.0 is noted as strong.

---

## Per-operator results

### T_b (non-transport)

Observed cycle lengths: 11 distinct | max = 120 | total sample = 1000

| Relation | obs rate | null rate | enrichment | assessment |
|---|---|---|---|---|
| divides_60   | 0.8360 | 0.1011 | 8.2674 | **strong enrichment** |
| divides_120  | 0.9570 | 0.1340 | 7.1418 | **strong enrichment** |
| divides_240  | 0.9570 | 0.1595 | 6.0015 | **strong enrichment** |
| multiple_60  | 0.3080 | 0.0166 | 18.5430 | **strong enrichment** |
| mult_2       | 0.7800 | 0.5016 | 1.5551 | moderate enrichment |
| mult_5       | 1.0000 | 0.2025 | 4.9378 | substantial enrichment |
| mult_12      | 0.3080 | 0.0819 | 3.7598 | substantial enrichment |

Top-5 cycle lengths and LCM-60 divisor status:

| Cycle length | count | divides_60? |
|---|---|---|
| 60 | 227 | yes |
| 30 | 193 | yes |
| 20 | 179 | yes |
| 15 | 152 | yes |
| 120 | 81 | no |

### T_x (non-transport)

Observed cycle lengths: 1 distinct | max = 6 | total sample = 1000

| Relation | obs rate | null rate | enrichment | assessment |
|---|---|---|---|---|
| divides_60   | 1.0000 | 1.0000 | 1.0000 | near null |
| divides_120  | 1.0000 | 1.0000 | 1.0000 | near null |
| divides_240  | 1.0000 | 1.0000 | 1.0000 | near null |
| multiple_60  | 0.0000 | 0.0000 | ∞ | null impossible; perfect alignment |
| mult_2       | 1.0000 | 0.4980 | 2.0079 | substantial enrichment |
| mult_5       | 0.0000 | 0.1677 | 0.0000 | depleted |
| mult_12      | 0.0000 | 0.0000 | ∞ | null impossible; perfect alignment |

Top-5 cycle lengths and LCM-60 divisor status:

| Cycle length | count | divides_60? |
|---|---|---|
| 6 | 1000 | yes |

### T_y (non-transport)

Observed cycle lengths: 20 distinct | max = 120 | total sample = 1000

| Relation | obs rate | null rate | enrichment | assessment |
|---|---|---|---|---|
| divides_60   | 0.6260 | 0.1000 | 6.2619 | **strong enrichment** |
| divides_120  | 0.7080 | 0.1343 | 5.2702 | **strong enrichment** |
| divides_240  | 0.7240 | 0.1585 | 4.5678 | substantial enrichment |
| multiple_60  | 0.0180 | 0.0177 | 1.0181 | near null |
| mult_2       | 1.0000 | 0.5021 | 1.9916 | moderate enrichment |
| mult_5       | 0.1190 | 0.2007 | 0.5928 | depleted |
| mult_12      | 0.2570 | 0.0838 | 3.0665 | substantial enrichment |

Top-5 cycle lengths and LCM-60 divisor status:

| Cycle length | count | divides_60? |
|---|---|---|
| 6 | 286 | yes |
| 12 | 135 | yes |
| 18 | 110 | no |
| 30 | 59 | yes |
| 2 | 52 | yes |

### T_c (transport)

Observed cycle lengths: 27 distinct | max = 135 | total sample = 1000

| Relation | obs rate | null rate | enrichment | assessment |
|---|---|---|---|---|
| divides_60   | 0.3940 | 0.0875 | 4.5034 | substantial enrichment |
| divides_120  | 0.5320 | 0.1173 | 4.5338 | substantial enrichment |
| divides_240  | 0.5320 | 0.1397 | 3.8076 | substantial enrichment |
| multiple_60  | 0.0210 | 0.0152 | 1.3825 | moderate enrichment |
| mult_2       | 0.6370 | 0.4988 | 1.2772 | moderate enrichment |
| mult_5       | 0.1400 | 0.2013 | 0.6954 | depleted |
| mult_12      | 0.1610 | 0.0814 | 1.9779 | moderate enrichment |

Top-5 cycle lengths and LCM-60 divisor status:

| Cycle length | count | divides_60? |
|---|---|---|
| 2 | 176 | yes |
| 9 | 127 | no |
| 8 | 106 | no |
| 27 | 65 | no |
| 4 | 58 | yes |

### T_z' (transport)

Observed cycle lengths: 35 distinct | max = 144 | total sample = 1000

| Relation | obs rate | null rate | enrichment | assessment |
|---|---|---|---|---|
| divides_60   | 0.4440 | 0.0830 | 5.3520 | **strong enrichment** |
| divides_120  | 0.4570 | 0.1108 | 4.1238 | substantial enrichment |
| divides_240  | 0.4680 | 0.1309 | 3.5755 | substantial enrichment |
| multiple_60  | 0.0840 | 0.0135 | 6.2454 | **strong enrichment** |
| mult_2       | 0.6070 | 0.5000 | 1.2141 | moderate enrichment |
| mult_5       | 0.2370 | 0.1954 | 1.2130 | moderate enrichment |
| mult_12      | 0.1430 | 0.0821 | 1.7422 | moderate enrichment |

Top-5 cycle lengths and LCM-60 divisor status:

| Cycle length | count | divides_60? |
|---|---|---|
| 6 | 155 | yes |
| 2 | 85 | yes |
| 60 | 71 | yes |
| 21 | 57 | no |
| 9 | 50 | no |

### T_r* (transport)

Observed cycle lengths: 37 distinct | max = 519 | total sample = 1000

| Relation | obs rate | null rate | enrichment | assessment |
|---|---|---|---|---|
| divides_60   | 0.0820 | 0.0232 | 3.5299 | substantial enrichment |
| divides_120  | 0.1700 | 0.0307 | 5.5285 | **strong enrichment** |
| divides_240  | 0.1890 | 0.0386 | 4.8926 | substantial enrichment |
| multiple_60  | 0.0340 | 0.0156 | 2.1809 | substantial enrichment |
| mult_2       | 0.5480 | 0.4986 | 1.0991 | near null |
| mult_5       | 0.1920 | 0.1997 | 0.9613 | near null |
| mult_12      | 0.2110 | 0.0809 | 2.6078 | substantial enrichment |

Top-5 cycle lengths and LCM-60 divisor status:

| Cycle length | count | divides_60? |
|---|---|---|
| 75 | 78 | no |
| 519 | 54 | no |
| 76 | 54 | no |
| 234 | 53 | no |
| 51 | 53 | no |

---

## Cluster-level enrichment summary

Mean enrichment ratio across operators per cluster:

| Relation | non-transport mean enrich | transport mean enrich |
|---|---|---|
| divides_60   | 5.1764 | 4.4618 |
| divides_120  | 4.4707 | 4.7287 |
| divides_240  | 3.8564 | 4.0919 |
| multiple_60  | 9.7806 | 3.2696 |
| mult_2       | 1.8515 | 1.1968 |
| mult_5       | 1.8435 | 0.9566 |
| mult_12      | 3.4131 | 2.1093 |

---

## Interpretation

### Operators with strong enrichment (ratio ≥ 5×)

- **T_b** (non-transport): `divides_60` — enrichment 8.2674×, observed rate 0.8360
- **T_b** (non-transport): `divides_120` — enrichment 7.1418×, observed rate 0.9570
- **T_b** (non-transport): `divides_240` — enrichment 6.0015×, observed rate 0.9570
- **T_b** (non-transport): `multiple_60` — enrichment 18.5430×, observed rate 0.3080
- **T_y** (non-transport): `divides_60` — enrichment 6.2619×, observed rate 0.6260
- **T_y** (non-transport): `divides_120` — enrichment 5.2702×, observed rate 0.7080
- **T_z'** (transport): `divides_60` — enrichment 5.3520×, observed rate 0.4440
- **T_z'** (transport): `multiple_60` — enrichment 6.2454×, observed rate 0.0840
- **T_r*** (transport): `divides_120` — enrichment 5.5285×, observed rate 0.1700

### Non-transport cluster

The non-transport operators (T_b, T_x, T_y) show strong `divides_60` enrichment if their dominant periods (6, 12, 20, 30, 60) are divisors of the tau LCM (60). This would confirm that their tau-cycle structure is governed directly by the fixed-step tau sub-moduli defined in the repo.

### Transport cluster

The transport operators (T_c, T_z', T_r*) have more dispersed periodicity structures. State-dependent coupled_phase and lift_phase steps allow orbit periods to exceed 60. Enrichment for `mult_2`, `mult_5`, or `mult_12` in the transport cluster would indicate that the tau sub-moduli still impose partial constraints even when the step is variable.

### Arithmetic evidence strength

Non-transport `divides_60` mean enrichment: **5.1764×**
Transport `divides_60` mean enrichment:     **4.4618×**

The arithmetic evidence for non-transport tau cycles aligning with the LCM-60 structure of the tau sub-moduli is **strong** — the enrichment ratio is well above the null baseline.

---

## Honesty section

| Question | Answer |
|---|---|
| Were any files modified? | **No** |
| Were any operators rebuilt in this step? | **No** |
| Is full exact spin_H solved? | **No** |
| Is arithmetic evidence strong, weak, or inconclusive? | See enrichment ratios above |

---

## Recommended next step

Run a bounded **tau sub-field periodicity decomposition**: for each operator, separately track the period of each tau sub-field (swap_phase, coupled_phase, twist_phase, lift_phase) in the sampled orbits, and report the per-sub-field period distribution.  This would reveal whether the observed cycle-length arithmetic structure arises from one dominant sub-field or from the interaction of all four tau components.
