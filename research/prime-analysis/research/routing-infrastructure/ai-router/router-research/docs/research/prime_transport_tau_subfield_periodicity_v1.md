# Prime Transport — Tau Sub-Field Periodicity Decomposition

**Audit type:** Read-only bounded sub-field periodicity decomposition
**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`
**Total surface states:** 525,355

---

## Sample and method

| Parameter | Value |
|---|---|
| Sample reuse | **Yes** — exact same seeds as prior tau orbit audits |
| Samples per stratum per operator | 500 |
| Total seeds per operator | 1000 |
| Orbit cap | 1000 |
| Random seed | 42 |

For each sampled seed, the orbit is iterated to cap=1000 and the
first-return step for each of the four tau sub-fields is recorded independently.
The full tau return step (all sub-fields simultaneously) is also recorded.

**Synchronization type classification per orbit:**
- `pure_lcm`: full tau period == LCM of all four sub-field periods
- `single_dominant`: full tau period == max of sub-field periods (one bottleneck)
- `lcm_multiple`: full tau period is a multiple of LCM(sub-periods) but not equal
- `complex`: full tau period doesn't fit the above patterns
- `incomplete`: at least one sub-field did not return within cap
- `tau_unresolved`: tau itself did not return within cap

---

## Per-operator sub-field period summary

### T_b (non-transport)

Sample: 1000

**Per-sub-field dominant period and top-3 distribution:**

| Sub-field | canonical modulus | top-1 period | top-1 fraction | top-2 | top-3 | unresolved |
|---|---|---|---|---|---|---|
| swap       | 2 | 1 | 1.0000 | — | — | 0.0000 |
| coupled    | 5 | 1 | 0.1920 | 2 (0.164) | 4 (0.131) | 0.0360 |
| twist      | 2 | 1 | 1.0000 | — | — | 0.0000 |
| lift       | 12 | 6 | 0.1530 | 3 (0.118) | 4 (0.101) | 0.0970 |

**Synchronization type distribution:**

| Sync type | count | fraction |
|---|---|---|
| tau_unresolved | 381 | 0.3810 |
| complex | 311 | 0.3110 |
| pure_lcm | 183 | 0.1830 |
| lcm_multiple | 67 | 0.0670 |
| single_dominant | 58 | 0.0580 |

**Bottleneck sub-field frequency** (sub-field whose period equals max sub-period):

| Sub-field | bottleneck count | fraction |
|---|---|---|
| swap | 31 | 0.0310 |
| coupled | 353 | 0.3530 |
| twist | 31 | 0.0310 |
| lift | 708 | 0.7080 |

### T_x (non-transport)

Sample: 1000

**Per-sub-field dominant period and top-3 distribution:**

| Sub-field | canonical modulus | top-1 period | top-1 fraction | top-2 | top-3 | unresolved |
|---|---|---|---|---|---|---|
| swap       | 2 | 2 | 0.5990 | 1 (0.276) | 3 (0.068) | 0.0000 |
| coupled    | 5 | 1 | 1.0000 | — | — | 0.0000 |
| twist      | 2 | 1 | 1.0000 | — | — | 0.0000 |
| lift       | 12 | 6 | 0.3580 | 3 (0.073) | 5 (0.053) | 0.3100 |

**Synchronization type distribution:**

| Sync type | count | fraction |
|---|---|---|
| tau_unresolved | 507 | 0.5070 |
| pure_lcm | 434 | 0.4340 |
| single_dominant | 46 | 0.0460 |
| complex | 12 | 0.0120 |
| lcm_multiple | 1 | 0.0010 |

**Bottleneck sub-field frequency** (sub-field whose period equals max sub-period):

| Sub-field | bottleneck count | fraction |
|---|---|---|
| swap | 342 | 0.3420 |
| coupled | 107 | 0.1070 |
| twist | 107 | 0.1070 |
| lift | 664 | 0.6640 |

### T_y (non-transport)

Sample: 1000

**Per-sub-field dominant period and top-3 distribution:**

| Sub-field | canonical modulus | top-1 period | top-1 fraction | top-2 | top-3 | unresolved |
|---|---|---|---|---|---|---|
| swap       | 2 | 1 | 1.0000 | — | — | 0.0000 |
| coupled    | 5 | 1 | 1.0000 | — | — | 0.0000 |
| twist      | 2 | 1 | 0.4290 | 2 (0.330) | 3 (0.123) | 0.0230 |
| lift       | 12 | 3 | 0.1220 | 2 (0.080) | 4 (0.071) | 0.2150 |

**Synchronization type distribution:**

| Sync type | count | fraction |
|---|---|---|
| tau_unresolved | 394 | 0.3940 |
| pure_lcm | 333 | 0.3330 |
| complex | 138 | 0.1380 |
| single_dominant | 104 | 0.1040 |
| lcm_multiple | 31 | 0.0310 |

**Bottleneck sub-field frequency** (sub-field whose period equals max sub-period):

| Sub-field | bottleneck count | fraction |
|---|---|---|
| swap | 111 | 0.1110 |
| coupled | 111 | 0.1110 |
| twist | 293 | 0.2930 |
| lift | 753 | 0.7530 |

### T_c (transport)

Sample: 1000

**Per-sub-field dominant period and top-3 distribution:**

| Sub-field | canonical modulus | top-1 period | top-1 fraction | top-2 | top-3 | unresolved |
|---|---|---|---|---|---|---|
| swap       | 2 | 1 | 1.0000 | — | — | 0.0000 |
| coupled    | 5 | 1 | 0.2130 | 2 (0.153) | 3 (0.115) | 0.0980 |
| twist      | 2 | 1 | 0.5060 | 2 (0.263) | 3 (0.116) | 0.0130 |
| lift       | 12 | 4 | 0.1010 | 1 (0.089) | 3 (0.066) | 0.1950 |

**Synchronization type distribution:**

| Sync type | count | fraction |
|---|---|---|
| tau_unresolved | 744 | 0.7440 |
| complex | 138 | 0.1380 |
| pure_lcm | 73 | 0.0730 |
| single_dominant | 27 | 0.0270 |
| lcm_multiple | 18 | 0.0180 |

**Bottleneck sub-field frequency** (sub-field whose period equals max sub-period):

| Sub-field | bottleneck count | fraction |
|---|---|---|
| swap | 73 | 0.0730 |
| coupled | 342 | 0.3420 |
| twist | 179 | 0.1790 |
| lift | 585 | 0.5850 |

### T_z' (transport)

Sample: 1000

**Per-sub-field dominant period and top-3 distribution:**

| Sub-field | canonical modulus | top-1 period | top-1 fraction | top-2 | top-3 | unresolved |
|---|---|---|---|---|---|---|
| swap       | 2 | 1 | 1.0000 | — | — | 0.0000 |
| coupled    | 5 | 1 | 0.2040 | 2 (0.156) | 3 (0.138) | 0.0150 |
| twist      | 2 | 1 | 0.5150 | 2 (0.233) | 3 (0.127) | 0.0010 |
| lift       | 12 | 1 | 0.1120 | 2 (0.096) | 3 (0.069) | 0.0750 |

**Synchronization type distribution:**

| Sync type | count | fraction |
|---|---|---|
| tau_unresolved | 633 | 0.6330 |
| complex | 216 | 0.2160 |
| pure_lcm | 67 | 0.0670 |
| single_dominant | 54 | 0.0540 |
| lcm_multiple | 30 | 0.0300 |

**Bottleneck sub-field frequency** (sub-field whose period equals max sub-period):

| Sub-field | bottleneck count | fraction |
|---|---|---|
| swap | 25 | 0.0250 |
| coupled | 345 | 0.3450 |
| twist | 114 | 0.1140 |
| lift | 642 | 0.6420 |

### T_r* (transport)

Sample: 1000

**Per-sub-field dominant period and top-3 distribution:**

| Sub-field | canonical modulus | top-1 period | top-1 fraction | top-2 | top-3 | unresolved |
|---|---|---|---|---|---|---|
| swap       | 2 | 1 | 1.0000 | — | — | 0.0000 |
| coupled    | 5 | 1 | 0.2040 | 2 (0.171) | 3 (0.130) | 0.0020 |
| twist      | 2 | 1 | 0.5150 | 2 (0.261) | 3 (0.121) | 0.0010 |
| lift       | 12 | 1 | 0.1220 | 3 (0.078) | 2 (0.073) | 0.0150 |

**Synchronization type distribution:**

| Sync type | count | fraction |
|---|---|---|
| complex | 465 | 0.4650 |
| tau_unresolved | 362 | 0.3620 |
| pure_lcm | 71 | 0.0710 |
| single_dominant | 51 | 0.0510 |
| lcm_multiple | 51 | 0.0510 |

**Bottleneck sub-field frequency** (sub-field whose period equals max sub-period):

| Sub-field | bottleneck count | fraction |
|---|---|---|
| swap | 11 | 0.0110 |
| coupled | 318 | 0.3180 |
| twist | 66 | 0.0660 |
| lift | 701 | 0.7010 |

---

## Cross-operator summary

| Operator | Cluster | primary bottleneck | top sync type | pure_lcm fraction | single_dominant fraction |
|---|---|---|---|---|---|
| T_b | non-transport | lift | tau_unresolved | 0.1830 | 0.0580 |
| T_x | non-transport | lift | tau_unresolved | 0.4340 | 0.0460 |
| T_y | non-transport | lift | tau_unresolved | 0.3330 | 0.1040 |
| T_c | transport | lift | tau_unresolved | 0.0730 | 0.0270 |
| T_z' | transport | lift | tau_unresolved | 0.0670 | 0.0540 |
| T_r* | transport | lift | complex | 0.0710 | 0.0510 |

| Metric | non-transport mean | transport mean |
|---|---|---|
| pure_lcm fraction     | 0.3167 | 0.0703 |
| single_dominant fraction | 0.0693 | 0.0440 |

---

## Interpretation

### Which sub-field is the primary bottleneck?

- **T_b** (non-transport): primary bottleneck = `lift` (0.708 of orbits); secondary = `coupled` (0.353)
- **T_x** (non-transport): primary bottleneck = `lift` (0.664 of orbits); secondary = `swap` (0.342)
- **T_y** (non-transport): primary bottleneck = `lift` (0.753 of orbits); secondary = `twist` (0.293)
- **T_c** (transport): primary bottleneck = `lift` (0.585 of orbits); secondary = `coupled` (0.342)
- **T_z'** (transport): primary bottleneck = `lift` (0.642 of orbits); secondary = `coupled` (0.345)
- **T_r*** (transport): primary bottleneck = `lift` (0.701 of orbits); secondary = `coupled` (0.318)

### LCM-60 arithmetic alignment: which sub-field drives it?

The prior audit showed strong `divides_60` enrichment across operators.
The tau LCM of 60 = lcm(2, 5, 2, 12).
If `lift_phase` (mod 12) is the dominant bottleneck, then the 60 alignment
is primarily driven by the lift sub-field.
If `coupled_phase` (mod 5) is the bottleneck, it is driven by the coupled sub-field.
Pure-LCM synchronization means the 60 arises from combining multiple sub-fields.

Operators where `lift_phase` is primary bottleneck (>30% of orbits): **T_b, T_x, T_y, T_c, T_z', T_r***. The LCM-60 alignment for these operators is primarily driven by the lift sub-field.
Operators where `pure_lcm` synchronization is dominant (>30%): **T_x, T_y**. The 60-alignment in these operators arises from composite sub-field synchronization.

### Transport vs non-transport cluster comparison

Non-transport pure_lcm mean fraction: 0.3167
Transport pure_lcm mean fraction:     0.0703
Non-transport single_dominant mean fraction: 0.0693
Transport single_dominant mean fraction:     0.0440

**Non-transport operators are more often pure-LCM synchronized** — their tau periods arise from combining all four sub-field periods. Transport operators show more complex or incomplete synchronization.

### What this means for the rebuilt algebra

The sub-field decomposition reveals which component of the tau holonomy structure
acts as the period-setting bottleneck for each operator. The `lift_phase` sub-field
(mod 12) has the widest modulus and the most variable step size, making it the
natural candidate for the dominant period driver across all operators.
The `coupled_phase` sub-field (mod 5, variable step from semiprime interaction)
can introduce non-trivial period contributions when its step size is coprime to
other sub-field periods.

---

## Honesty section

| Question | Answer |
|---|---|
| Were any files modified? | **No** |
| Were any operators rebuilt in this step? | **No** |
| Is full exact spin_H solved? | **No** |
| Arithmetic evidence strength after decomposition | see sync type fractions above |

---

## Recommended next step

Run a bounded **coupled_phase step-size distribution audit**: for each operator, compute the distribution of the effective coupled_phase step sizes observed across sampled states (`(query_semiprime × binding_semiprime) % 5`), and determine whether step-size concentration or variation explains the different coupled_phase period structures observed between transport and non-transport operators.
