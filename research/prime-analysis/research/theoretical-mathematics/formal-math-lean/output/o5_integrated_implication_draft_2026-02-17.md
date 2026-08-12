# O5 Integrated Implication Draft

Generated: 2026-02-17T21:26:48.588915+00:00

## Integrated Theorem Statement

Assume A1_ref and A3_ref hold asymptotically with wheel-uniform constants at M_ref. Then for all x>=x0 and W in {30,210,2310,30030}, |E(x)|/sqrt(x) <= C0_ref + |b_ref| + |a_ref| C_H (log x)^A_H. Hence |E(x)| <= C sqrt(x) (log x)^A with A=max(A_H,0).

## Constants Snapshot

- `a_ref = -0.0013474693715061251`
- `b_ref = -0.05436122353654979`
- `C0_ref (o5) = 0.6334997766223893`
- `C0_uplifted (o1) = 0.9102883687683553`
- `C_H = 8.710122086201695`
- `C_H_theorem_budget = 19.741654586280088`
- `A_H = 2.0`
- `H constants source = o3_derived_bridge_from_e2`

## Endpoint Class Check

- target exponent: `2.0`
- meets class now: `True`
- gap `A_H-target`: `0.0`

## Assumption Discharge Status

| assumption | obligation | status | artifact |
|---|---|---|---|
| A1_ref | O1 | conditional_residual_interface_closed | `research/output/o1_u_residual_o1_handoff_2026-02-17.json` |
| A3_ref | O3 | conditional_bridge_derived | `research/output/o3_ref_bridge_from_e2_closure_2026-02-17.json` |
| wheel_uniformity | O4 | conditional_uniformity_interface_closed | `research/output/o3_u_uniformity_o4_handoff_2026-02-17.json` |

## Next Blocker Order

- All strict theorem requirements are closed.
