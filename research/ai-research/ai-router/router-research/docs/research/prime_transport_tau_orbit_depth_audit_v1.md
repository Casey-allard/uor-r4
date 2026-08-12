# Prime Transport — Tau-Holonomy Orbit Depth Audit

**Audit type:** Read-only bounded stratified orbit audit
**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`
**Total surface states:** 525,355

---

## Sampling and bounding parameters

| Parameter | Value |
|---|---|
| Samples per stratum per operator | 500 |
| Strata | sigma-stable, non-sigma-stable (under each operator) |
| Maximum orbit iterations (cap) | 200 |
| Random seed | 42 |
| Tau-return criterion | tau(Oᵏ(s)) == tau(s) for smallest k ≥ 1 |
| Cycle-detection criterion | full state Oᵏ(s) ∈ previously seen states |

---

## Stratum sizes per operator

| Operator | Cluster | sigma-stable count | non-sigma-stable count |
|---|---|---|---|
| T_b | non-transport | 56,464 | 468,891 |
| T_x | non-transport | 42,323 | 483,032 |
| T_y | non-transport | 214,836 | 310,519 |
| T_c | transport | 209,227 | 316,128 |
| T_z' | transport | 207,484 | 317,871 |
| T_r* | transport | 197,789 | 327,566 |

---

## Per-operator pooled results (both strata combined)

| Operator | Cluster | sample | τ-return count | τ-return fraction | avg return depth | median return depth | avg cycle | max cycle | unresolved |
|---|---|---|---|---|---|---|---|---|---|
| T_b | non-transport | 1000 | 619 | 0.6190 | 23.8433 | 18 | 38.8100 | 120 | 0.0000 |
| T_x | non-transport | 1000 | 493 | 0.4930 | 5.9310 | 6 | 6 | 6 | 0.0000 |
| T_y | non-transport | 1000 | 606 | 0.6060 | 11.6122 | 9.0000 | 18.5960 | 120 | 0.0000 |
| T_c | transport | 1000 | 256 | 0.2560 | 27.4375 | 18.0000 | 25.0390 | 135 | 0.0000 |
| T_z' | transport | 1000 | 367 | 0.3670 | 33.4986 | 27 | 36.0250 | 144 | 0.0000 |
| T_r* | transport | 1000 | 608 | 0.6080 | 58.1776 | 47.0000 | 52.1818 | 192 | 0.3070 |

---

## Per-stratum detail

### T_b (non-transport)

| Stratum | sample | τ-return fraction | avg depth | median depth | avg cycle | unresolved |
|---|---|---|---|---|---|---|
| sigma_stable | 500 | 0.5040 | 13.3333 | 12.0000 | 22.0800 | 0.0000 |
| non_sigma_stable | 500 | 0.7340 | 31.0599 | 24 | 55.5400 | 0.0000 |

### T_x (non-transport)

| Stratum | sample | τ-return fraction | avg depth | median depth | avg cycle | unresolved |
|---|---|---|---|---|---|---|
| sigma_stable | 500 | 0.6520 | 6 | 6.0000 | 6 | 0.0000 |
| non_sigma_stable | 500 | 0.3340 | 5.7964 | 5 | 6 | 0.0000 |

### T_y (non-transport)

| Stratum | sample | τ-return fraction | avg depth | median depth | avg cycle | unresolved |
|---|---|---|---|---|---|---|
| sigma_stable | 500 | 0.6220 | 10.9293 | 8 | 17.9080 | 0.0000 |
| non_sigma_stable | 500 | 0.5900 | 12.3322 | 9 | 19.2840 | 0.0000 |

### T_c (transport)

| Stratum | sample | τ-return fraction | avg depth | median depth | avg cycle | unresolved |
|---|---|---|---|---|---|---|
| sigma_stable | 500 | 0.2660 | 25.2782 | 18 | 23.8460 | 0.0000 |
| non_sigma_stable | 500 | 0.2460 | 29.7724 | 19 | 26.2320 | 0.0000 |

### T_z' (transport)

| Stratum | sample | τ-return fraction | avg depth | median depth | avg cycle | unresolved |
|---|---|---|---|---|---|---|
| sigma_stable | 500 | 0.3720 | 32.4516 | 25.5000 | 36.6980 | 0.0000 |
| non_sigma_stable | 500 | 0.3620 | 34.5746 | 28 | 35.3520 | 0.0000 |

### T_r* (transport)

| Stratum | sample | τ-return fraction | avg depth | median depth | avg cycle | unresolved |
|---|---|---|---|---|---|---|
| sigma_stable | 500 | 0.6060 | 58.7096 | 48 | 50.4813 | 0.3060 |
| non_sigma_stable | 500 | 0.6100 | 57.6492 | 47 | 53.8873 | 0.3080 |

---

## Cluster-level comparison

| Metric | non-transport {T_b, T_x, T_y} | transport {T_c, T_z', T_r*} |
|---|---|---|
| mean tau-return fraction  | 0.5727   | 0.4103 |
| mean avg return depth     | 13.7955  | 39.7046 |
| mean avg cycle length     | 21.1353      | 37.7486 |
| mean unresolved fraction  | 0.0000 | 0.1023 |

---

## Interpretation

### Tau-return structure

Transport operators have a **longer mean tau return depth** (39.7046) than non-transport operators (13.7955). This is consistent with the hypothesis that transport operators drive deeper tau excursions before the holonomy coordinate returns.

Transport operators have a **lower tau-return rate within cap** (0.4103) compared to non-transport (0.5727). Transport tau excursions are harder to close within the cap, suggesting longer or more complex tau orbits.

### Tau orbit and the two-cluster picture

Given that tau_holonomy was identified as the primary motion axis for both clusters in the field-signature audit, the tau orbit depth reveals whether the **rate** and **depth** of tau motion differs between clusters or whether the distinction lies purely in *which other fields* accompany the tau motion.

---

## Honesty section

| Question | Answer |
|---|---|
| Were any files modified? | **No** |
| Were any operators rebuilt in this step? | **No** |
| Is full exact spin_H solved? | **No** |

---

## Recommended next step

Run a bounded **tau-holonomy cycle-length census on the full surface**: for each operator, exhaustively compute the tau-return depth for every state on the bounded v25 surface (not just a sample).  This would confirm whether the sampled tau orbit distributions are representative and whether any operator has states with structurally anomalous tau return depths that the sample did not capture.
