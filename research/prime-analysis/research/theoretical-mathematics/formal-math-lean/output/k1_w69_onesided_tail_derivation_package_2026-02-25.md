# K1 W69 One-Sided Tail Derivation Package (2026-02-25)

## Goal
Derive explicit one-sided tail constants in the required form
\[
\left|\frac{R(x)}{x^\beta}\right|\le Cx^{-\eta}
\]
with fixed `beta=0.62`, `eta=0.01`, `x>=10^21`, using source-locked constants only.

## Source-Locked Inputs Used
1. Truncated explicit formula remainder (Cully-Hugill-Johnston tuple used in FKS 2022/2023):
   - admissible constants: `M=2.091`, `log x_M=40`, `alpha=1/2`, `omega=0`,
   - shape:
     \[
     |Rem_A(x,T)| \le C_A\frac{x(\log x)^2}{T},\quad C_A=2.091.
     \]

2. Finite-band coefficient envelope from explicit `N(T)` upper majorant:
   \[
   S_1(b)\le U(b)/b+\int_{\gamma_N}^{b}U(t)t^{-2}\,dt,
   \]
   with Trudgian-style explicit `U(T)` model and `gamma_N=gamma_{256}=478.942181535`.

3. Band contribution bound:
   \[
   |Band(x)|\le x^{1/2-\beta}\cdot 2S_1(x^\theta).
   \]

## Derived Constant Formula
Set `T(x)=x^theta` and
\[
\delta:=\theta+\beta-1-\eta.
\]
If `delta>0`, then
\[
(\log x)^2x^{-(\theta+\beta-1)}
=\big((\log x)^2x^{-\delta}\big)x^{-\eta}
\le M(x_1,\delta)x^{-\eta},
\]
where
\[
M(x_1,\delta)=\sup_{x\ge x_1}(\log x)^2x^{-\delta}.
\]
Hence
\[
\left|\frac{R(x)}{x^\beta}\right|
\le\Big(C_{\text{band}}(\theta)+C_A\,M(x_1,\delta)\Big)x^{-\eta}
=C_{\text{total}}(\theta)x^{-\eta}.
\]

## Feasibility Constraints (checked at `x1=10^21`)
For the CHJ source tuple:
- `x1 >= e^40` (satisfied),
- `T(x1) > max(51, log x1)`,
- `T(x1) < (sqrt(x1)-2)/5`,
- `theta < 1/2`,
- `delta > 0`.

These force `theta` into a narrow feasible interval, ending at about `0.466`.

## W69 Scan Result (`theta in [0.39,0.499]`)
Artifact:
- `/Users/adminamn/Documents/New project/research/output/k1_w69_onesided_tail_theorem_grade_beta062_eta001_CHJ_2026-02-25.json`

Best feasible row:
- `theta = 0.466`
- `delta = 0.076`
- `C_band = 0.324511020225`
- `M(x1,delta) = 59.2747821987`
- `C_A * M = 123.943569578`
- `C_total = 124.268080598`.

Therefore:
\[
\left|\frac{R(x)}{x^{0.62}}\right|
\le 124.268080598 \, x^{-0.01},
\qquad x\ge 10^{21}.
\]

## q<a1 Consequence (one-sided closure shape)
For any `q_target in (0, a1)` with `a1=0.98`,
\[
x\ge X_q:=\max\!\left(10^{21},\left(\frac{124.268080598}{q_{target}}\right)^{100}\right)
\Rightarrow
\left|\frac{R(x)}{x^{0.62}}\right|\le q_{target}.
\]
In particular, explicit finite thresholds were computed for `q_target=0.882, 0.686, 0.49`.

## Status
This supplies an explicit-constant, theorem-grade candidate for the one-sided tail power bound using source-locked constants and no finite-window fit constants.
