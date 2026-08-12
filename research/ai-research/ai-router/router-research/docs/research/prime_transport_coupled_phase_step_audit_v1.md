# Prime Transport — Coupled-Phase Step-Size Distribution Audit

**Audit type:** Read-only bounded coupled_phase step-size distribution
**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`
**Sample per operator:** 1000 states (500 sigma-stable + 500 non-sigma-stable, seed=42)
**Sample reuse:** Same RNG seed and operator order as all prior tau audits (sample NOT identical because stratification re-runs per operator, but seed/order are held fixed)

## Coupled-Phase Step Formula

The canonical coupled-phase step in the repo (`geometry_native_operator_model_v3.py`, `geometry_native_spinH_candidate_v3.py`):

```
interaction_step = (qs + 2*bs + gcd(qs,bs)) % 5
new_coupled_phase = (old_coupled_phase + interaction_step) % 5
```

The rebuilt operators (v20–v25) use a sigma-mediated step that combines `sigma.regressive_phase`, `sigma.family_holonomy_class`, and an operator-specific geometric term.  The effective step is still reduced mod 5.

**T_x** (v24) and **T_y** (v25) explicitly preserve `coupled_phase` by construction — their step is identically 0.

## Per-Operator Step-Size Distributions

### T_b  (non_transport)

- Sample: 1000 states
- Distinct steps observed: 5 / 5
- Shannon entropy: 2.3179 bits  (max = 2.3219 for uniform over 5)

| step | count | fraction |
|------|-------|----------|
| 0 | 192 | 0.1920 |
| 1 | 202 | 0.2020 |
| 2 | 212 | 0.2120 |
| 3 | 218 | 0.2180 |
| 4 | 176 | 0.1760 |

**T_b** exhibits broadly dispersed coupled_phase stepping (entropy=2.3179 bits).  Dominant step is 3 (21.80% of applications).

### T_x  (non_transport)

- Sample: 1000 states
- Distinct steps observed: 1 / 5
- Shannon entropy: 0.0000 bits  (max = 2.3219 for uniform over 5)

| step | count | fraction |
|------|-------|----------|
| 0 | 1000 | 1.0000 |
| 1 | 0 | 0.0000 |
| 2 | 0 | 0.0000 |
| 3 | 0 | 0.0000 |
| 4 | 0 | 0.0000 |

**T_x always produces step=0** — `coupled_phase` is preserved by construction.  This operator has no coupled_phase dynamics.

### T_y  (non_transport)

- Sample: 1000 states
- Distinct steps observed: 1 / 5
- Shannon entropy: 0.0000 bits  (max = 2.3219 for uniform over 5)

| step | count | fraction |
|------|-------|----------|
| 0 | 1000 | 1.0000 |
| 1 | 0 | 0.0000 |
| 2 | 0 | 0.0000 |
| 3 | 0 | 0.0000 |
| 4 | 0 | 0.0000 |

**T_y always produces step=0** — `coupled_phase` is preserved by construction.  This operator has no coupled_phase dynamics.

### T_c  (transport)

- Sample: 1000 states
- Distinct steps observed: 5 / 5
- Shannon entropy: 2.3198 bits  (max = 2.3219 for uniform over 5)

| step | count | fraction |
|------|-------|----------|
| 0 | 213 | 0.2130 |
| 1 | 191 | 0.1910 |
| 2 | 207 | 0.2070 |
| 3 | 184 | 0.1840 |
| 4 | 205 | 0.2050 |

**T_c** exhibits broadly dispersed coupled_phase stepping (entropy=2.3198 bits).  Dominant step is 0 (21.30% of applications).

### T_z'  (transport)

- Sample: 1000 states
- Distinct steps observed: 5 / 5
- Shannon entropy: 2.3185 bits  (max = 2.3219 for uniform over 5)

| step | count | fraction |
|------|-------|----------|
| 0 | 204 | 0.2040 |
| 1 | 195 | 0.1950 |
| 2 | 203 | 0.2030 |
| 3 | 220 | 0.2200 |
| 4 | 178 | 0.1780 |

**T_z'** exhibits broadly dispersed coupled_phase stepping (entropy=2.3185 bits).  Dominant step is 3 (22.00% of applications).

### T_r*  (transport)

- Sample: 1000 states
- Distinct steps observed: 5 / 5
- Shannon entropy: 2.3203 bits  (max = 2.3219 for uniform over 5)

| step | count | fraction |
|------|-------|----------|
| 0 | 204 | 0.2040 |
| 1 | 209 | 0.2090 |
| 2 | 188 | 0.1880 |
| 3 | 210 | 0.2100 |
| 4 | 189 | 0.1890 |

**T_r*** exhibits broadly dispersed coupled_phase stepping (entropy=2.3203 bits).  Dominant step is 3 (21.00% of applications).

## Cluster-Level Comparison

Non-transport cluster: **{T_b, T_x, T_y}**  |  Transport cluster: **{T_c, T_z', T_r*}**

| metric | non-transport | transport |
|--------|---------------|-----------|
| mean coupled_phase entropy (bits) | 0.7726 | 2.3195 |
| mean step-0 fraction              | 0.7307 | 0.2070 |
| mean non-trivial step fraction    | 0.2693 | 0.7930 |

Excluding T_x / T_y (step=0 by construction), the active non-transport operator (T_b) has coupled_phase entropy 2.3179 bits vs transport cluster mean 2.3195 bits.

Active non-transport (T_b) and transport operators show similar coupled_phase step entropy — the diversity level is comparable.

## Interpretation

The prior tau sub-field decomposition found `coupled_phase` as the secondary tau bottleneck for T_b, T_c, T_z', and T_r*.  T_x and T_y have swap_phase and twist_phase as their respective secondary bottlenecks, not coupled_phase.

**Why coupled_phase is secondary for T_b, T_c, T_z', T_r*:**

1. **Non-trivial step distribution.**  These four operators produce steps drawn from at least part of {0,1,2,3,4}, meaning coupled_phase cycles through multiple residues and the return period for coupled_phase alone is ≥ 5 in many orbits.

2. **Modulus 5 is coprime to moduli 2 and 12.**  coupled_phase mod 5 introduces a prime factor (5) that is absent from swap (mod 2) and twist (mod 2), and interacts with lift (mod 12) only through their LCM = 60.  This means coupled_phase provides a bottleneck that neither swap nor twist can replicate, even though swap and twist share the same modulus.

3. **T_x and T_y preserve coupled_phase.**  Their tau bottleneck is therefore determined by whichever of swap / twist is harder to return — not by coupled_phase.

**Summary:** coupled_phase is the secondary bottleneck specifically because it is the only sub-field with modulus 5, making it structurally independent from the modulus-2 fields (swap, twist) and contributing a distinct factor to the LCM-60 period structure.

## Honesty Section

- **Files modified:** No.  Only the read-only runner script was created.
- **Operators rebuilt:** No.
- **Full exact spin_H solved:** No.
- **Conclusion strength:** Strong for T_x and T_y (step=0 by construction, zero ambiguity).  Strong for active operators — the step-size distributions are computed directly from the v25 operator applications; no estimation involved.

## Recommended Next Step

Run a bounded **tau full-period factorization audit**: for each operator, take the observed full tau return periods from the periodicity-class audit and compute their prime factorizations.  Determine whether full-period primes are drawn exclusively from {2, 3, 5} (the prime factors of 60) or whether any operator introduces additional prime factors — which would indicate a genuine departure from LCM-60 arithmetic structure.

