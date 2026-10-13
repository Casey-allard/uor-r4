# K1 W42 Subsequence Contradiction Lemma Candidate (2026-02-24)

## Purpose
Convert the current empirical route into a precise mathematical lemma target that avoids the overly strong global remainder condition.

## Setup (fixed canonical error)
Let
\[
E_\*(x)=x^\beta\big(A\cos(\tau\log x+\phi)+R_\beta(x)\big),\qquad \beta>\tfrac12,\ A>0.
\]

Define phase-aligned points \(x_n\) by
\[
\tau\log x_n+\phi = 2\pi n.
\]

## Candidate Lemma L-SUBSEQ
Assume there exist constants \(a_0\in(0,1]\), \(\delta\in(0,a_0)\), and infinitely many indices \(n\) such that
\[
|\cos(\tau\log x_n+\phi)|\ge a_0,\qquad
\frac{|R_\beta(x_n)|}{A}\le a_0-\delta.
\]
Then for infinitely many \(n\),
\[
|E_\*(x_n)|\ge \delta A\,x_n^\beta.
\]

### Proof (direct)
\[
\frac{|E_\*(x_n)|}{x_n^\beta}
 = \left|A\cos(\tau\log x_n+\phi)+R_\beta(x_n)\right|
 \ge A|\cos(\tau\log x_n+\phi)|-|R_\beta(x_n)|
 \ge A(a_0-(a_0-\delta))=A\delta.
\]
Done.

## Candidate Lemma L-COFINAL
If the L-SUBSEQ hypothesis holds on a cofinal set of phase-aligned points (equivalently: for every \(X\), some \(x_n\ge X\) satisfies the inequalities), then
\[
\forall X\ \exists x\ge X:\ |E_\*(x)|\ge (\delta A)x^\beta.
\]

This is exactly the lower-envelope quantifier shape needed in the contradiction route.

## Link to W40/W41 numeric evidence
From W40 (`x<=3e7`, `beta=0.62`):
- `A ≈ 2.920877e-02`
- `a0 = 0.98` (alignment gate)
- finite-grid witness constant from triangle channel:
  \[
  w_{\mathrm{tri}} \approx 1.631385e-02.
  \]

So at witness points, effective
\[
\delta_{\mathrm{eff}} \approx \frac{w_{\mathrm{tri}}}{A} \approx 0.5583,
\]
and therefore
\[
\frac{|R_\beta(x_n)|}{A} \lesssim a_0-\delta_{\mathrm{eff}} \approx 0.4217
\]
on those witness points.

Interpretation: current data is consistent with **subsequence remainder domination** at many aligned points, even though global
\[
\sup_{x\in\text{tail}} |R_\beta(x)|/A
\]
is much larger than 1.

## Remaining hard theorem obligation
Prove non-circularly (from explicit formula + right-half-zero hypothesis) that such cofinal aligned points exist with
\[
|R_\beta(x_n)|/A \le a_0-\delta
\]
for some fixed \(\delta>0\).

This replaces the too-strong global condition \(|R_\beta(x)|\le A/4\) for all large \(x\), and matches the observed behavior.

