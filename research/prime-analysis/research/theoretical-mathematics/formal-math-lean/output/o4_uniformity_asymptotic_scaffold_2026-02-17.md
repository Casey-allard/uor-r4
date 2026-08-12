# O4 Uniformity Asymptotic Scaffold

Generated: 2026-02-17T19:13:39.977156+00:00

## Quantified Statement

There exist x0>=3 and wheel-uniform constants such that for all x>=x0 and all W in {30,210,2310,30030}, the A1/A3 endpoint-transfer constants are common and satisfy a single uniform inequality family.

## Current Constant Candidates

- `a_ref = -0.0013474693715061251`
- `b_ref = -0.05436122353654979`
- `C0_ref = 0.6334997766223893`
- `C_delta = 4.6749702791170504e-05`
- `tau_m_tail_fp = 383.99980444354367`
- `C_H = 8.45323261007565e-07`

## Grid Uniformity Snapshot

- holds on grid: `True`
- ratio max: `0.7746942032162933`
- max gap (lhs-rhs): `-0.18056490311475804`

## Remaining O4 Blockers

- Replace finite-grid spread checks with asymptotic uniform-in-W proofs.
- Prove no hidden W-dependence in constants as x->infinity.
- Finalize common x0 threshold valid simultaneously for all W in the wheel family.
