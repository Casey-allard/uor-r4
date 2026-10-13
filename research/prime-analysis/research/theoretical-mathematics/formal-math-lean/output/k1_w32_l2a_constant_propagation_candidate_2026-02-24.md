# K1 W32 L2A Constant Propagation Candidate (2026-02-24)

## Purpose
Construct a first explicit `eta=0.01` constant ledger using the new endpoint-valid truncation schedule candidate
\[
T(x)=x^{0.13},\quad x\in[10^{21},10^{30}],
\]
as a bridge toward theorem-level `C_256, eta_256`.

## Inputs
1. Split ledger (`xpow=0.13`):
   - `/Users/adminamn/Documents/New project/research/output/k1_w32_l2a_split_ledger_xpow013_2026-02-24.json`
2. External envelope constants at `eta=0.01`:
   - from `/Users/adminamn/Documents/New project/research/output/k1_w31_l2a_constant_ledger_2026-02-24.json`
3. Remainder model:
   \[
   \mathcal R_T(x)\sim \frac{(\log x)^2}{T}=\frac{(\log x)^2}{x^{0.13}}.
   \]

## Eta-target conversion (`eta = 0.01`)
To write each component as `<= C * x^{-0.01}`, use:
\[
C = \sup_{x\in[10^{21},10^{30}]} x^{0.01}\,|component(x)|.
\]

### Component constants
- Empirical finite-band (`E_{256,<=T}`) from split ledger:
  - `C_band(eta=0.01) = 0.0150796767`
- Empirical high tail (`E_{>T}`) from split ledger:
  - `C_high_emp(eta=0.01) = 0.0230719245`
- External high-tail envelope candidate (Bellotti model; smaller than FKS at this eta):
  - `C_high_ext(eta=0.01) = 0.9669252367`
- Truncation remainder term with `T=x^0.13`:
  - `C_rem(eta=0.01) = sup (log x)^2 x^{-0.12} = 7.0610610572`

## Combined candidates
1. Pure empirical surrogate:
\[
C_{256,\text{emp}}(0.01)=0.0257915549.
\]

2. Hybrid (empirical band + external high + analytic remainder):
\[
C_{256,\text{hybrid}}(0.01)\approx
C_{\text{band}}+C_{\text{high,ext}}+C_{\text{rem}}
=8.0430659706.
\]

## Interpretation
- The exponent target `eta=0.01` is numerically compatible with the `x^0.13` split in local surrogate data.
- The dominant term in the current hybrid ledger is the truncation remainder constant, not the oscillatory band/high pieces.
- Next theorem pass should focus on tightening the remainder constant path and replacing empirical band terms with explicit density-based inequalities.
