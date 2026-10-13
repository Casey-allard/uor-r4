# Lemma E Endpoint Note (Finite-Window Asymptotic Indicator)

Generated: February 17, 2026
Artifact: `/Users/adminamn/Documents/New project/research/output/lemma_e_endpoint_probe_none_z128_ref512.json`

## Endpoint Target
Translate triangle-transfer constants into RH-style envelope candidate:
\[
|E(x)|/\sqrt{x} \le C(\log x)^A
\]
on tested windows, then assess stability of `A` and `C` across increasing `x` scales.

## Configuration
- bases: `30,210,2310,30030`
- `M=128`, `M_ref=512`
- kernel=`none`
- scales tested: `x<=3e5, 1e6, 2e6, 5e6`

## Fitted Envelope Indicators
From max-over-bases envelopes:
- observed residual (`|E/\sqrt{x}-(a_ref H+b_ref)|`) best stabilized exponent:
  - `A ~= 0.7`
  - `C_max ~= 0.100397`
  - `cv ~= 0.043371`
- triangle RHS (`C0 + |a_ref|\Delta_M`) best stabilized exponent:
  - `A ~= 0.5`
  - `C_max ~= 0.181840`
  - `cv ~= 0.012860`

Interpretation: tested envelopes are consistent with slow polylog growth (sub-polynomial in `x`).

## Caution
This is still finite-window evidence, not an RH proof. It does not establish asymptotic validity of the exponent model; it only indicates a stable candidate form for the next theorem stage.

## Next Theorem Step
Prove a true asymptotic endpoint implication of the form
\[
\forall x\ge x_0:\ |E(x)|/\sqrt{x}\le C(\log x)^A,
\]
with explicit `x_0,C,A`, derived from proved Lemma B (truncation) + Lemma C/D (uniform transfer) bounds rather than empirical fitting.
