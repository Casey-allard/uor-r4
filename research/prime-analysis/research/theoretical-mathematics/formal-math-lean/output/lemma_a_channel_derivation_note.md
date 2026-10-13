# Lemma A Derivation Note (Frozen Candidate B)

Generated: February 17, 2026
Checker artifact: `/Users/adminamn/Documents/New project/research/output/lemma_a_channel_derivation_check_none_z64_n1m.json`

## Setup
For cutoff `M`, define
\[
F_M(u)=\sum_{j\le M}\frac{e^{i\gamma_j u}}{\tfrac12+i\gamma_j},\quad
F'_M(u)=\sum_{j\le M}\frac{i\gamma_j e^{i\gamma_j u}}{\tfrac12+i\gamma_j}.
\]
At integer `n`, let
\[
(fr,fi,gr,gi)=(\Re F_M(\log n),\Im F_M(\log n),\Re F'_M(\log n),\Im F'_M(\log n)).
\]
Define
\[
\theta_n=\arg(F_M(\log n))=\operatorname{atan2}(fi,fr),\quad
\phi^{(2)}_n=\arg(F'_M(\log n))=\operatorname{atan2}(gi,gr),\quad
\chi_n=\frac{\phi^{(2)}_n+\pi}{2}.
\]

## Channel identities
The frozen channels used in Candidate B admit explicit coordinate formulas:

1. `phase_vel`:
\[
\Delta\theta_n=\operatorname{unwrap}(\theta_{n+1}-\theta_n).
\]

2. `su2_trace`:
From the `su2_step` parametrization
\[
U_n=\cos(\Delta\theta_n/2)I+i\sin(\Delta\theta_n/2)(\hat n_n\cdot\sigma),
\]
its real trace is
\[
\operatorname{Tr}(U_n)_{\Re}=2\cos(\Delta\theta_n/2).
\]

3. `Y11s`:
\[
Y11s_n=\sqrt{\frac{3}{4\pi}}\,\sin\chi_n\,\sin\theta_n.
\]
Using half-angle identities with `R_1=\sqrt{fr^2+fi^2}`, `R_2=\sqrt{gr^2+gi^2}`,
\[
\sin\chi_n=\sqrt{\frac{1+gr/R_2}{2}},\qquad \sin\theta_n=fi/R_1,
\]
thus
\[
Y11s_n=\sqrt{\frac{3}{4\pi}}\,\sqrt{\frac{1+gr/R_2}{2}}\,\frac{fi}{R_1}.
\]

4. `Y20`:
\[
Y20_n=\sqrt{\frac{5}{16\pi}}\,(3\cos^2\chi_n-1),\qquad
\cos^2\chi_n=\frac{1-gr/R_2}{2},
\]
hence
\[
Y20_n=\sqrt{\frac{5}{16\pi}}\left(3\frac{1-gr/R_2}{2}-1\right).
\]

These identities show each frozen channel is an explicit deterministic functional of `(F_M,F'_M)` coordinates and one-step phase transport.

## Numerical verification (canonical regime)
Configuration:
- `M=64`, `u=log`, kernel=`none`
- bases `30,210,2310,30030`
- `n_max=10^6`

Worst-case max absolute discrepancy vs pipeline implementation:
- `Y11s`: `3.914945314931487e-12`
- `Y20`: `6.106226635438361e-16`
- `su2_trace`: `0.0`
- `phase_vel`: `0.0`

Conclusion: Lemma A identities match implementation to machine precision on the canonical regime.
