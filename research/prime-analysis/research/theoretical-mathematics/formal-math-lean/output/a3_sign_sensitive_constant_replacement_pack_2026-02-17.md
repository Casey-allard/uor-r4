# A3 Sign-Sensitive Constant Replacement Pack

Generated: 2026-02-17T18:14:10.897325+00:00

## O3 Primary vs Deterministic Replacement

- Primary `A_eta=4.0`, `C_eta=1.21701344784`, `A_H=1.2`, `C_H=2.60722567538`
- Replacement `A_eta=4.0`, `C_eta=21.2240856288`
- replacement/primary ratio: `17.439483242`

## Deterministic Chain Check

- `eta_pos <= eta_ss` holds: `True`
- max gap `eta_pos-eta_ss`: `-5.84342163813`
- held-out ratio over replacement `C_eta`: `0.651427852522`
- lag-tail constant used: `0.432171486471`

## Status

- replacement valid on cached grid: `True`
- replacement tighter than primary: `False`

## Remaining Analytic Gap

- Replace empirical lag-tail calibration k_tail_used by theorem-side explicit lag-sum constant.
- Convert finite-grid replacement constants into asymptotic x>=x0 statement.
