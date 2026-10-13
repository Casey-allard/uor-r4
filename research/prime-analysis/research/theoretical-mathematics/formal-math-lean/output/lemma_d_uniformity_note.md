# Lemma D Note: Base-Uniform Triangle Transfer

Generated: February 17, 2026
Artifact: `/Users/adminamn/Documents/New project/research/output/lemma_d_base_uniformity_probe_none_z128_ref512.json`

## Uniformity Target
For wheel family `\mathcal W={30,210,2310,30030}`, test whether one shared set of constants
`(a_ref,b_ref,C0_ref)` supports
\[
|E/\sqrt{x}-(a_{ref}H_W^{(M)}+b_{ref})|\le C0_{ref}+|a_{ref}|\,\Delta_M,
\]
uniformly over `W` and tested `x` windows.

## Calibrated Shared Constants
- `a_ref = -0.0047075423063228546`
- `b_ref = -0.07903726672397497`
- `C0_ref = 0.6126289168356691`

Configuration:
- kernel=`none`, `M=128`, `M_ref=512`
- `x<=10^6`, `x_step=2000`
- bases `30,210,2310,30030`

## Uniformity Results
- holds uniform triangle on tested grid: `True`
- max uniform gap `(lhs-rhs)`: `0.0`
- aggregate uniformity score (relative spread summary): `0.03330189089416037`

Per-`n_max` relative spreads stay small for residual and ratio metrics:
- `rel_spread_max_residual ~ 0.0072`
- `rel_spread_ratio_res_over_rhs <= 0.0033`

Interpretation: base dependence is present but controlled; one shared constant set is plausible on tested windows.

## Lemma D Candidate Statement (finite-window)
There exists a shared triple `(a_ref,b_ref,C0_ref)` such that for all tested `W in \mathcal W` and sampled `x`:
\[
|E/\sqrt{x}-(a_{ref}H_W^{(M)}+b_{ref})|\le C0_{ref}+|a_{ref}|\Delta_M.
\]

The remaining theorem step is asymptotic uplift: prove this uniformity beyond sampled windows and finite cutoffs.
