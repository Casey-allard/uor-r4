# A1 Reference Residual Note (Candidate Constants)

Generated: February 17, 2026
Artifact: `/Users/adminamn/Documents/New project/research/output/a1_reference_residual_probe_none_ref512.json`

## A1 Candidate Form
For reference process `H_W^{(M_ref)}` with fixed `(a_ref,b_ref)`, seek
\[
\left|E(x)/\sqrt{x}-(a_{ref}H_W^{(M_{ref})}(x)+b_{ref})\right|\le C_0(x).
\]

Finite-window candidate (none kernel, `M_ref=512`, bases `30,210,2310,30030`, up to `x=5e6`):
- `a_ref = -0.001347471211160521`
- `b_ref = -0.05436128386957597`
- `C0_max_over_tested = 0.6334997534895853`

## Growth Indicator
Best stabilized log-power model for max residual profile:
\[
C_0(x) \approx C(\log x)^\alpha
\]
with
- `alpha ~= 0.4`
- `C_max ~= 0.212890019973503`
- model `cv ~= 0.018076` (stable over tested scales).

Interpretation: residual envelope appears slowly varying/polylog-like on tested windows.

## Important Note
These `(a_ref,b_ref)` are re-fit on the larger scale set and therefore differ from earlier finite-window constants used in the initial triangle-transfer calibration. This is expected. The theorem track should either:
1. freeze one canonical `(a_ref,b_ref)` and prove all bounds for it, or
2. introduce a deterministic rule producing `(a_ref,b_ref)` independent of fitting.
