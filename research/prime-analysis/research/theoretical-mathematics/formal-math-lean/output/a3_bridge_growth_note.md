# A3 Bridge Growth Note (Candidate Constants)

Generated: February 17, 2026
Artifact: `/Users/adminamn/Documents/New project/research/output/a3_bridge_growth_probe_none_z128_v2.json`

## A3 Candidate Form
For `M=128` and wheel family `W in {30,210,2310,30030}` on tested windows,
\[
|H_W^{(M)}(x)| \le C_H (\log x)^{A_H}.
\]

## Calibrated Values (finite-window)
- `A_H = 7.2`
- `C_H = 1.9424205508648547e-07`
- tested scales: `x<=3e5,1e6,2e6,5e6`
- envelope check: zero violations on anchor scales
- ratio range `|H|/rhs`: `[0.942266, 1.0]`

## Interpretation
1. This is a conservative polylog envelope candidate for the bridge process over tested windows.
2. Exponent is larger than endpoint indicator exponents from Lemma E; this reflects conservative max-over-base envelope fitting.
3. It is suitable as an A3 hypothesis candidate but still requires asymptotic proof.

## Next Proof Task
Derive analytic upper bounds for channel sums that imply the same or better exponent, with constants independent of sampled windows.
