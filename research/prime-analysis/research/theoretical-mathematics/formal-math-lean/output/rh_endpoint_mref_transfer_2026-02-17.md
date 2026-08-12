# RH Endpoint Mref Transfer Audit

Generated: 2026-02-17T19:01:54.957737+00:00

## Core Result

- In the `M=M_ref` regime, `Delta_M=0` so the A2 `beta` exponent does not enter the endpoint growth class.
- Implied endpoint exponent class from A3: `A_H=1.2`
- RH-style target exponent: `2.0`
- Reachable in this regime: `True`
- Exponent gap `A_H-target`: `-0.8`

## Constants Used

- `a_ref=-0.0013474693715061251`
- `C0_ref=0.6334997766223893`
- `C_H=3.47471643488075`
- `A_H=1.2`

## Empirical Grid Check (No Delta Term)

- `holds_on_grid=True`
- `ratio_max=0.8487096648414026`
- `gap_min=0.11067341654404417`

## Remaining Theorem Work (O5)

- Prove the endpoint transfer inequality in theorem form at M_ref (not only on sampled grids).
- Prove A3 asymptotic bound |H(x)| <= C_H (log x)^A_H with explicit hypotheses.
- Map resulting endpoint bound to a standard RH-equivalent formulation with complete assumptions.
