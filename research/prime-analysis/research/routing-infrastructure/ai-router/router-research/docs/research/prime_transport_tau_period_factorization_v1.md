# Prime Transport — Tau Full-Period Factorization Audit

**Audit type:** Read-only arithmetic analysis of existing periodicity data
**Source data:** `prime_transport_tau_periodicity_classes_v1.csv` (tau periodicity-class audit, cap=1000, sample=1000, seed=42)
**No orbit recomputation:** Pure arithmetic factorization on previously observed periods.
**Canonical prime set:** {2, 3, 5} (prime factors of LCM=60)

## Per-Operator Factorization Summary

### T_b  (non_transport)

- Observed distinct periods: 11
- Total orbit-observations: 1000
- Fraction fully supported on {2,3,5}: **0.9960** (996/1000)
- Fraction containing extra prime factors: **0.0040** (4/1000)
- Largest prime factor observed: **7**
- Distinct periods with extra primes: 1 / 11

| period | count | fraction | factorization | all in {2,3,5}? | extra primes |
|--------|-------|----------|---------------|-----------------|--------------|
| 60 | 227 | 0.2270 | 2^2 × 3 × 5 | ✓ | — |
| 30 | 193 | 0.1930 | 2 × 3 × 5 | ✓ | — |
| 20 | 179 | 0.1790 | 2^2 × 5 | ✓ | — |
| 15 | 152 | 0.1520 | 3 × 5 | ✓ | — |
| 120 | 81 | 0.0810 | 2^3 × 3 × 5 | ✓ | — |
| 10 | 60 | 0.0600 | 2 × 5 | ✓ | — |
| 40 | 40 | 0.0400 | 2^3 × 5 | ✓ | — |
| 25 | 37 | 0.0370 | 5^2 | ✓ | — |
| 5 | 25 | 0.0250 | 5 | ✓ | — |
| 105 | 4 | 0.0040 | 3 × 5 × 7 | ✗ | 7 |
| 75 | 2 | 0.0020 | 3 × 5^2 | ✓ | — |

**Top extra-prime periods by orbit frequency:**
  - period 105 = 3 × 5 × 7  (extra: [7], count=4)

### T_x  (non_transport)

- Observed distinct periods: 1
- Total orbit-observations: 1000
- Fraction fully supported on {2,3,5}: **1.0000** (1000/1000)
- Fraction containing extra prime factors: **0.0000** (0/1000)
- Largest prime factor observed: **3**
- Distinct periods with extra primes: 0 / 1

| period | count | fraction | factorization | all in {2,3,5}? | extra primes |
|--------|-------|----------|---------------|-----------------|--------------|
| 6 | 1000 | 1.0000 | 2 × 3 | ✓ | — |

**No extra-prime periods observed — all periods fully supported on {2,3,5}.**

### T_y  (non_transport)

- Observed distinct periods: 20
- Total orbit-observations: 1000
- Fraction fully supported on {2,3,5}: **0.9020** (902/1000)
- Fraction containing extra prime factors: **0.0980** (98/1000)
- Largest prime factor observed: **11**
- Distinct periods with extra primes: 4 / 20

| period | count | fraction | factorization | all in {2,3,5}? | extra primes |
|--------|-------|----------|---------------|-----------------|--------------|
| 6 | 286 | 0.2860 | 2 × 3 | ✓ | — |
| 12 | 135 | 0.1350 | 2^2 × 3 | ✓ | — |
| 18 | 110 | 0.1100 | 2 × 3^2 | ✓ | — |
| 30 | 59 | 0.0590 | 2 × 3 × 5 | ✓ | — |
| 2 | 52 | 0.0520 | 2 | ✓ | — |
| 42 | 51 | 0.0510 | 2 × 3 × 7 | ✗ | 7 |
| 4 | 43 | 0.0430 | 2^2 | ✓ | — |
| 8 | 39 | 0.0390 | 2^3 | ✓ | — |
| 36 | 37 | 0.0370 | 2^2 × 3^2 | ✓ | — |
| 24 | 34 | 0.0340 | 2^3 × 3 | ✓ | — |
| 10 | 31 | 0.0310 | 2 × 5 | ✓ | — |
| 14 | 22 | 0.0220 | 2 × 7 | ✗ | 7 |
| 84 | 18 | 0.0180 | 2^2 × 3 × 7 | ✗ | 7 |
| 16 | 16 | 0.0160 | 2^4 | ✓ | — |
| 54 | 16 | 0.0160 | 2 × 3^3 | ✓ | — |
| 72 | 15 | 0.0150 | 2^3 × 3^2 | ✓ | — |
| 20 | 11 | 0.0110 | 2^2 × 5 | ✓ | — |
| 120 | 9 | 0.0090 | 2^3 × 3 × 5 | ✓ | — |
| 60 | 9 | 0.0090 | 2^2 × 3 × 5 | ✓ | — |
| 66 | 7 | 0.0070 | 2 × 3 × 11 | ✗ | 11 |

**Top extra-prime periods by orbit frequency:**
  - period 42 = 2 × 3 × 7  (extra: [7], count=51)
  - period 14 = 2 × 7  (extra: [7], count=22)
  - period 84 = 2^2 × 3 × 7  (extra: [7], count=18)
  - period 66 = 2 × 3 × 11  (extra: [11], count=7)

### T_c  (transport)

- Observed distinct periods: 27
- Total orbit-observations: 1000
- Fraction fully supported on {2,3,5}: **0.9210** (921/1000)
- Fraction containing extra prime factors: **0.0790** (79/1000)
- Largest prime factor observed: **31**
- Distinct periods with extra primes: 8 / 27

| period | count | fraction | factorization | all in {2,3,5}? | extra primes |
|--------|-------|----------|---------------|-----------------|--------------|
| 2 | 176 | 0.1760 | 2 | ✓ | — |
| 9 | 127 | 0.1270 | 3^2 | ✓ | — |
| 8 | 106 | 0.1060 | 2^3 | ✓ | — |
| 27 | 65 | 0.0650 | 3^3 | ✓ | — |
| 4 | 58 | 0.0580 | 2^2 | ✓ | — |
| 18 | 57 | 0.0570 | 2 × 3^2 | ✓ | — |
| 6 | 50 | 0.0500 | 2 × 3 | ✓ | — |
| 36 | 40 | 0.0400 | 2^2 × 3^2 | ✓ | — |
| 96 | 35 | 0.0350 | 2^5 × 3 | ✓ | — |
| 12 | 33 | 0.0330 | 2^2 × 3 | ✓ | — |
| 135 | 32 | 0.0320 | 3^3 × 5 | ✓ | — |
| 15 | 32 | 0.0320 | 3 × 5 | ✓ | — |
| 24 | 32 | 0.0320 | 2^3 × 3 | ✓ | — |
| 117 | 24 | 0.0240 | 3^2 × 13 | ✗ | 13 |
| 30 | 22 | 0.0220 | 2 × 3 × 5 | ✓ | — |
| 60 | 21 | 0.0210 | 2^2 × 3 × 5 | ✓ | — |
| 51 | 17 | 0.0170 | 3 × 17 | ✗ | 17 |
| 39 | 16 | 0.0160 | 3 × 13 | ✗ | 13 |
| 75 | 13 | 0.0130 | 3 × 5^2 | ✓ | — |
| 25 | 13 | 0.0130 | 5^2 | ✓ | — |
| 11 | 8 | 0.0080 | 11 | ✗ | 11 |
| 90 | 7 | 0.0070 | 2 × 3^2 × 5 | ✓ | — |
| 31 | 5 | 0.0050 | 31 | ✗ | 31 |
| 33 | 5 | 0.0050 | 3 × 11 | ✗ | 11 |
| 3 | 2 | 0.0020 | 3 | ✓ | — |
| 21 | 2 | 0.0020 | 3 × 7 | ✗ | 7 |
| 7 | 2 | 0.0020 | 7 | ✗ | 7 |

**Top extra-prime periods by orbit frequency:**
  - period 117 = 3^2 × 13  (extra: [13], count=24)
  - period 51 = 3 × 17  (extra: [17], count=17)
  - period 39 = 3 × 13  (extra: [13], count=16)
  - period 11 = 11  (extra: [11], count=8)
  - period 31 = 31  (extra: [31], count=5)

### T_z'  (transport)

- Observed distinct periods: 35
- Total orbit-observations: 1000
- Fraction fully supported on {2,3,5}: **0.6240** (624/1000)
- Fraction containing extra prime factors: **0.3760** (376/1000)
- Largest prime factor observed: **59**
- Distinct periods with extra primes: 17 / 35

| period | count | fraction | factorization | all in {2,3,5}? | extra primes |
|--------|-------|----------|---------------|-----------------|--------------|
| 6 | 155 | 0.1550 | 2 × 3 | ✓ | — |
| 2 | 85 | 0.0850 | 2 | ✓ | — |
| 60 | 71 | 0.0710 | 2^2 × 3 × 5 | ✓ | — |
| 21 | 57 | 0.0570 | 3 × 7 | ✗ | 7 |
| 9 | 50 | 0.0500 | 3^2 | ✓ | — |
| 7 | 49 | 0.0490 | 7 | ✗ | 7 |
| 10 | 45 | 0.0450 | 2 × 5 | ✓ | — |
| 57 | 44 | 0.0440 | 3 × 19 | ✗ | 19 |
| 144 | 41 | 0.0410 | 2^4 × 3^2 | ✓ | — |
| 54 | 39 | 0.0390 | 2 × 3^3 | ✓ | — |
| 5 | 36 | 0.0360 | 5 | ✓ | — |
| 87 | 33 | 0.0330 | 3 × 29 | ✗ | 29 |
| 65 | 32 | 0.0320 | 5 × 13 | ✗ | 13 |
| 30 | 31 | 0.0310 | 2 × 3 × 5 | ✓ | — |
| 102 | 26 | 0.0260 | 2 × 3 × 17 | ✗ | 17 |
| 26 | 21 | 0.0210 | 2 × 13 | ✗ | 13 |
| 78 | 20 | 0.0200 | 2 × 3 × 13 | ✗ | 13 |
| 59 | 19 | 0.0190 | 59 | ✗ | 59 |
| 18 | 18 | 0.0180 | 2 × 3^2 | ✓ | — |
| 23 | 16 | 0.0160 | 23 | ✗ | 23 |
| 63 | 15 | 0.0150 | 3^2 × 7 | ✗ | 7 |
| 33 | 13 | 0.0130 | 3 × 11 | ✗ | 11 |
| 120 | 13 | 0.0130 | 2^3 × 3 × 5 | ✓ | — |
| 48 | 11 | 0.0110 | 2^4 × 3 | ✓ | — |
| 42 | 11 | 0.0110 | 2 × 3 × 7 | ✗ | 7 |
| 44 | 9 | 0.0090 | 2^2 × 11 | ✗ | 11 |
| 12 | 7 | 0.0070 | 2^2 × 3 | ✓ | — |
| 11 | 5 | 0.0050 | 11 | ✗ | 11 |
| 3 | 5 | 0.0050 | 3 | ✓ | — |
| 15 | 5 | 0.0050 | 3 × 5 | ✓ | — |
| 75 | 4 | 0.0040 | 3 × 5^2 | ✓ | — |
| 19 | 4 | 0.0040 | 19 | ✗ | 19 |
| 4 | 4 | 0.0040 | 2^2 | ✓ | — |
| 27 | 4 | 0.0040 | 3^3 | ✓ | — |
| 13 | 2 | 0.0020 | 13 | ✗ | 13 |

**Top extra-prime periods by orbit frequency:**
  - period 21 = 3 × 7  (extra: [7], count=57)
  - period 7 = 7  (extra: [7], count=49)
  - period 57 = 3 × 19  (extra: [19], count=44)
  - period 87 = 3 × 29  (extra: [29], count=33)
  - period 65 = 5 × 13  (extra: [13], count=32)

### T_r*  (transport)

- Observed distinct periods: 37
- Total orbit-observations: 1000
- Fraction fully supported on {2,3,5}: **0.4910** (491/1000)
- Fraction containing extra prime factors: **0.5090** (509/1000)
- Largest prime factor observed: **173**
- Distinct periods with extra primes: 16 / 37

| period | count | fraction | factorization | all in {2,3,5}? | extra primes |
|--------|-------|----------|---------------|-----------------|--------------|
| 75 | 78 | 0.0780 | 3 × 5^2 | ✓ | — |
| 519 | 54 | 0.0540 | 3 × 173 | ✗ | 173 |
| 76 | 54 | 0.0540 | 2^2 × 19 | ✗ | 19 |
| 234 | 53 | 0.0530 | 2 × 3^2 × 13 | ✗ | 13 |
| 51 | 53 | 0.0530 | 3 × 17 | ✗ | 17 |
| 50 | 52 | 0.0520 | 2 × 5^2 | ✓ | — |
| 27 | 52 | 0.0520 | 3^3 | ✓ | — |
| 237 | 51 | 0.0510 | 3 × 79 | ✗ | 79 |
| 93 | 50 | 0.0500 | 3 × 31 | ✗ | 31 |
| 8 | 46 | 0.0460 | 2^3 | ✓ | — |
| 6 | 44 | 0.0440 | 2 × 3 | ✓ | — |
| 72 | 40 | 0.0400 | 2^3 × 3^2 | ✓ | — |
| 231 | 38 | 0.0380 | 3 × 7 × 11 | ✗ | 7, 11 |
| 228 | 36 | 0.0360 | 2^2 × 3 × 19 | ✗ | 19 |
| 342 | 36 | 0.0360 | 2 × 3^2 × 19 | ✗ | 19 |
| 192 | 35 | 0.0350 | 2^6 × 3 | ✓ | — |
| 84 | 34 | 0.0340 | 2^2 × 3 × 7 | ✗ | 7 |
| 120 | 29 | 0.0290 | 2^3 × 3 × 5 | ✓ | — |
| 9 | 21 | 0.0210 | 3^2 | ✓ | — |
| 4 | 20 | 0.0200 | 2^2 | ✓ | — |
| 33 | 18 | 0.0180 | 3 × 11 | ✗ | 11 |
| 48 | 16 | 0.0160 | 2^4 × 3 | ✓ | — |
| 49 | 13 | 0.0130 | 7^2 | ✗ | 7 |
| 24 | 13 | 0.0130 | 2^3 × 3 | ✓ | — |
| 34 | 12 | 0.0120 | 2 × 17 | ✗ | 17 |
| 45 | 9 | 0.0090 | 3^2 × 5 | ✓ | — |
| 18 | 9 | 0.0090 | 2 × 3^2 | ✓ | — |
| 5 | 6 | 0.0060 | 5 | ✓ | — |
| 90 | 5 | 0.0050 | 2 × 3^2 × 5 | ✓ | — |
| 60 | 5 | 0.0050 | 2^2 × 3 × 5 | ✓ | — |
| 35 | 4 | 0.0040 | 5 × 7 | ✗ | 7 |
| 15 | 4 | 0.0040 | 3 × 5 | ✓ | — |
| 12 | 3 | 0.0030 | 2^2 × 3 | ✓ | — |
| 16 | 3 | 0.0030 | 2^4 | ✓ | — |
| 78 | 2 | 0.0020 | 2 × 3 × 13 | ✗ | 13 |
| 21 | 1 | 0.0010 | 3 × 7 | ✗ | 7 |
| 54 | 1 | 0.0010 | 2 × 3^3 | ✓ | — |

**Top extra-prime periods by orbit frequency:**
  - period 519 = 3 × 173  (extra: [173], count=54)
  - period 76 = 2^2 × 19  (extra: [19], count=54)
  - period 234 = 2 × 3^2 × 13  (extra: [13], count=53)
  - period 51 = 3 × 17  (extra: [17], count=53)
  - period 237 = 3 × 79  (extra: [79], count=51)

## Cluster-Level Comparison

| metric | non-transport {T_b, T_x, T_y} | transport {T_c, T_z', T_r*} |
|--------|-------------------------------|------------------------------|
| total orbit-observations         | 3000 | 3000 |
| orbits with extra prime periods  | 102 (3.4%) | 964 (32.1%) |
| largest prime factor observed    | 11 | 173 |

**Transport operators depart from the {2,3,5} arithmetic envelope far more often than non-transport operators** (32.1% vs 3.4% extra-prime orbit fraction, ratio ≈ 9.5×).

The largest prime factor in the non-transport cluster is **11** (from T_y, period 66 = 2×3×11).  The largest prime factor in the transport cluster is **173** (from T_r*, period 519 = 3×173) — a factor of 15× larger.

## Interpretation

The LCM-60 arithmetic envelope is **not fully preserved** across all operators on the bounded v25 surface.

**Non-transport cluster ({T_b, T_x, T_y}) — primarily inside the 60-envelope:**
  - T_x is completely contained (period=6=2×3, 100% canonical).
  - T_b shows only 0.4% extra-prime orbit fraction — one period (105=3×5×7) introduces prime 7.
  - T_y shows 9.8% extra-prime orbit fraction, driven by periods containing prime 7 (42=2×3×7, 14=2×7, 84=2²×3×7) and prime 11 (66=2×3×11).

**Transport cluster ({T_c, T_z', T_r*}) — substantially outside the 60-envelope:**
  - T_c: 7.9% extra-prime fraction; introduces primes 7, 11, 13, 17, 31.
  - T_z': 37.6% extra-prime fraction; introduces primes 7, 11, 13, 17, 19, 23, 29, 59.
  - T_r*: 50.9% extra-prime fraction; introduces primes 7, 11, 13, 17, 19, 31, 79, 173 — the largest prime factor observed across the entire operator layer.

**Arithmetic interpretation:** The LCM-60 scaffold generated by {swap=2, coupled=5, twist=2, lift=12} constrains the sub-field periods individually but does NOT fully constrain the joint orbit period of the full tau vector. Transport operators, which induce deeper tau excursions and more complex synchronization, can generate full-period lengths that require prime factors well outside {2,3,5}.  This indicates that the full-period arithmetic structure of the transport cluster genuinely departs from a pure LCM-60 regime.

**Non-transport interpretation:** T_b and T_y also show slight departures, suggesting the 60-envelope is an approximate, not exact, bound for full tau orbits. T_x (period=6 only) remains perfectly inside the envelope.

## Honesty Section

- **Files modified:** No.
- **Operators rebuilt:** No.
- **Full exact spin_H solved:** No.
- **Factorization evidence:** Strong. The prime factorizations are exact arithmetic facts; no estimation or sampling is involved in the factorization step. The orbit-count fractions depend on the bounded sample (cap=1000, n=1000 per operator), so the precise percentages have sampling uncertainty, but the qualitative pattern (transport cluster departing far more than non-transport) is clear and robust.

## Recommended Next Step

Run a bounded **tau extra-prime factor census**: for the transport operators (T_c, T_z', T_r*), enumerate every distinct extra prime factor (outside {2,3,5}) observed in the tau orbit periods, record their frequency distribution, and determine whether the extra primes appear to be bounded (concentrated near small values like 7, 11, 13) or genuinely unbounded (growing larger with deeper orbits as suggested by T_r*'s period 519=3×173).

