# A3 Sign-Sensitive Constant Replacement Pack

Generated: 2026-02-17T18:24:39.601183+00:00

## O3 Primary vs Deterministic Replacement

- Primary `A_eta=4.0`, `C_eta=1.21701344784`, `A_H=1.2`, `C_H=2.60722567538`
- Replacement `A_eta=4.0`, `C_eta=39.2847268831`
- replacement/primary ratio: `32.2796160987`

## Deterministic Chain Check

- `eta_pos <= eta_ss` holds: `True`
- max gap `eta_pos-eta_ss`: `-167.074072104`
- held-out ratio over replacement `C_eta`: `0.814343331994`
- lag-tail constant used: `1`
- tail mode: `deterministic`

## Status

- replacement valid on cached grid: `True`
- replacement tighter than primary: `False`
- deterministic tail mode: `True`

## Remaining Analytic Gap

- Replace empirical lag-tail calibration k_tail_used by theorem-side explicit lag-sum constant.
- Convert finite-grid replacement constants into asymptotic x>=x0 statement.
