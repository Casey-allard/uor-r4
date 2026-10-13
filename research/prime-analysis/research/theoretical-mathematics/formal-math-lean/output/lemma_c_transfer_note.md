# Lemma C Note: Transfer Through Explicit Surrogate

Generated: February 17, 2026
Program artifact: `/Users/adminamn/Documents/New project/research/output/lemma_c_transfer_program_none_z64.json`

## Transfer Template
Define explicit-formula surrogate
\[
S_M(x):=-2\sum_{\gamma_j\le \gamma_M}\Re\!\left(\frac{e^{i\gamma_j\log x}}{1/2+i\gamma_j}\right)
=-2\sum_{j\le M}\frac{0.5\cos(\gamma_j\log x)+\gamma_j\sin(\gamma_j\log x)}{0.25+\gamma_j^2}.
\]

Candidate transfer chain:
\[
H_W \xrightarrow[]{\text{affine}} S_M \xrightarrow[]{\text{affine}} E/\sqrt{x},
\quad E(x)=\psi(x)-x.
\]

The theorem target is to replace finite-window affine fits with analytic kernel bounds.

## Canonical Calibration
Configuration:
- kernel = none
- `M=64`
- bases `W in {30,210,2310,30030}`
- `x<=10^6`, `x_step=2000`
- weights `(-1,-1,0,-1)`

Summary metrics:
- mean `|corr(H,S)| = 0.47130210808057177`
- mean `|corr(S,E/\sqrt{x})| = 0.8928372092757377`
- mean `|corr(H,E/\sqrt{x})| = 0.4273833596384538`
- mean `|corr(compose(H->S->E),E/\sqrt{x})| = 0.4273833596384537`
- global composed residual envelope (tested windows):
  \[
  C_C := \max_{W,x\le 10^6}\left|E(x)/\sqrt{x}-(a_{SE}(a_{HS}H_W+b_{HS})+b_{SE})\right|
  \le 0.560106228778126.
  \]

## Interpretation
1. `S_M` tracks `E/\sqrt{x}` strongly in tested windows (`~0.89` mean abs correlation).
2. `H_W` is moderately aligned to `S_M` and hence transfers to `E/\sqrt{x}` with stable sign and base-consistent behavior.
3. This supports Lemma C as a structured two-step transfer candidate rather than a direct black-box regression.

## Next Theorem Step For Lemma C
Promote the finite-window chain to an inequality of the form
\[
\left|E(x)/\sqrt{x}-(\alpha H_W(x)+\beta)\right|
\le C_1\,\Delta_M(x)+C_2\,R_{\text{smooth}}(x),
\]
where `\Delta_M` is explicit truncation/tail control (Lemma B) and `R_smooth` is an analytic smoothing remainder from explicit formula.
