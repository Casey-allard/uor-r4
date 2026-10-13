# K1 W36 Eta-Gap Necessity Derivation (2026-02-24)

Let
\[
Y_{\mathrm{band}}(x)=x^{\frac12-\beta}\,2S_1(x^\theta),
\]
with
\[
S_1(b)=\sum_{\gamma_n<\gamma\le b}\frac1\gamma.
\]
For a target tail majorant exponent `eta>0`, we need finite
\[
C_{\mathrm{band}}=\sup_{x\ge x_1} x^\eta Y_{\mathrm{band}}(x)
 = \sup_{x\ge x_1} x^{\eta+\frac12-\beta}\,2S_1(x^\theta).
\]

Using explicit zero-count asymptotics/bounds (`N(T)=O(T\log T)`), one gets
\[
S_1(b)=O((\log b)^2)=O((\log x)^2).
\]
Hence
\[
x^\eta Y_{\mathrm{band}}(x)=
O\!\left(x^{\eta+\frac12-\beta}(\log x)^2\right).
\]

So:
- if \(\eta+\frac12-\beta>0\), this diverges polynomially;
- if \(\eta+\frac12-\beta=0\), this diverges like \((\log x)^2\);
- only if \(\eta+\frac12-\beta<0\) can it be uniformly bounded.

Therefore, a necessary condition for finite global `C_band` is
\[
\beta>\frac12+\eta.
\]

For the project-locked target `eta=0.01`, this is
\[
\beta>0.51.
\]

This is exactly what the W36 numeric probe confirms:
- `/Users/adminamn/Documents/New project/research/output/k1_w36_eta_band_necessity_probe_2026-02-24.md`.
