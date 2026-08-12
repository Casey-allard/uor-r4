# K1 W31 External Theorem Constants (TeX-Exact) (2026-02-24)

## Scope
Lock the exact theorem inputs (from TeX source, not OCR/PDF text) needed for the L2 omitted-tail majorant derivation.

## Source extractions

### Bellotti 2024 (arXiv:2405.12545)
Source:
- `/Users/adminamn/Documents/New project/research/external/papers/src/2405.12545/final.tex:72`

Exact statement used:
\[
N(\sigma,T)\le C T^{B(1-\sigma)}
\]
for `T in (T0, T1]`, `sigma in [alpha0, 1]` (table-driven constants).

Uniform explicit example recorded in the paper:
- `/Users/adminamn/Documents/New project/research/external/papers/src/2405.12545/final.tex:79`
- `B = 1.448`, `C = 1.62 * 10^11` on `3*10^12 < T <= exp(6.7*10^12)` and `sigma in [0.9927, 1]`.

### Johnston 2024/2025 (arXiv:2411.13791)
Source:
- `/Users/adminamn/Documents/New project/research/external/papers/src/2411.13791/main.tex:199`

Main transfer theorem (generic zero-free + zero-density):
\[
\Delta_i(x)\ll
\exp(-\omega(x))
\exp\!\left(2A\omega(x)\left(\frac{\omega(x)}{\log x}\right)^B\right)
\omega(x)^C
\]
assuming
\[
N(\sigma,t)\ll t^{A(1-\sigma)^B}(\log t)^C.
\]

Two explicit corollaries used:
- Log-free case (`A=5/2` input): `/Users/adminamn/Documents/New project/research/external/papers/src/2411.13791/main.tex:238`
\[
\Delta_i(x)\ll\exp(-\omega(x))\exp\!\left(5\frac{\omega(x)^2}{\log x}\right),
\quad
\omega(x)=O(\sqrt{\log x}) \Rightarrow \Delta_i(x)\ll\exp(-\omega(x)).
\]
- VK-density case: `/Users/adminamn/Documents/New project/research/external/papers/src/2411.13791/main.tex:261`
\[
\Delta_i(x)\ll
\exp(-\omega(x))
\exp\!\left(117\frac{\omega(x)^{5/2}}{(\log x)^{3/2}}\right)\omega(x)^{15},
\]
and asymptotically
\[
\Delta_i(x)\ll \exp(-\omega(x))\frac{(\log x)^9}{(\log\log x)^3}.
\]

### Bellotti 2025 (arXiv:2508.02041)
Sources:
- `/Users/adminamn/Documents/New project/research/external/papers/src/2508.02041/semi-explicit.tex:85`
- `/Users/adminamn/Documents/New project/research/external/papers/src/2508.02041/semi-explicit.tex:116`

Near-1-line zero-density corollary:
\[
N(\sigma,T)\ll e^{55A}
\]
for
\[
\sigma\ge 1-A(\log T)^{-2/3}(\log\log T)^{-1/3},\quad A>1/48.0718.
\]

PNT error theorem:
\[
\Delta(x)=\frac{|\psi(x)-x|}{x}
\ll \exp(55A_0)\exp(-\omega(x)),
\]
with `A0` the KV zero-free constant and
\[
\omega(x)\asymp (\log x)^{3/5}(\log\log x)^{-1/5}.
\]

### Fiori-Kadiri-Swidinsky 2022/2023 (arXiv:2204.02588)
Source:
- `/Users/adminamn/Documents/New project/research/external/papers/src/2204.02588/main.tex:160`

Global explicit corollary:
\[
E_\psi(x)=\frac{|\psi(x)-x|}{x}
< 9.22022(\log x)^{3/2}\exp(-0.8476836\sqrt{\log x}),
\quad x>2.
\]

### Schlage-Puchta 2019 (arXiv:1912.00853)
Source:
- `/Users/adminamn/Documents/New project/research/external/papers/src/1912.00853/main.tex:39`

Right-half zero implies oscillation lower bound (Theorem 3):
if `rho0 = sigma0 + i gamma0` is a zero with `sigma0 >= 1/2 + eps`,
then for large enough `X` there exists `x in [X, X^(1+eps)]` such that
\[
|\Delta(x)| > c_2\,\frac{x^{\sigma_0}}{\gamma_0^{1+\epsilon}}.
\]
This is direct evidence against any blanket finite-mode exact cancellation claim without controlling omitted-spectrum terms.

## Immediate L2 implication
These sources support a bound pipeline of the form
\[
Y(x)=\frac{R(x)}{x^\beta}=S_N(x)+\mathcal E_N(x),\qquad
|\mathcal E_N(x)|\le C_N x^{-\eta_N},
\]
with `C_N, eta_N` derived from zero-density + zero-free inputs.

They do not provide an exact theorem of the form `E_N(x) == 0` for fixed finite `N=256`.

## Sanity check at x = 10^21
Using quoted explicit envelopes:
- FKS bound gives `E_psi(10^21) <= 8.539247`.
- Bellotti 2025 Theorem 3 model (with `A0 = 1/48.0718`) gives `Delta(10^21) <= 0.596201`.

So published explicit bounds are consistent with control/majorant behavior, but are not remotely an exact finite-tail collapse witness at the locked endpoint.
