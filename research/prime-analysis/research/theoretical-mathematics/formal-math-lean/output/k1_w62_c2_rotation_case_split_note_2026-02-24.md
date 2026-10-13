# K1 W62 C2 Rotation Case-Split Note (2026-02-24)

## Purpose
Pin down the exact analytic step needed for constructive C2:
prove infinitely many tau1-anchor indices with \(\cos_2 \ge 0\).

## Anchor-phase reduction
On exact tau1 anchors \(x_k^\*\) defined by
\[
\tau_1\log x_k^\*+\phi_1 = 2\pi k,
\]
the tau2 phase is
\[
\Theta_k := \tau_2\log x_k^\*+\phi_2
= 2\pi\rho k + \beta_0,\quad
\rho:=\tau_2/\tau_1,\ \beta_0:=\phi_2-\rho\phi_1.
\]
So C2 sign question reduces to \(\cos(2\pi\rho k+\beta_0)\).

## Case-split lemma template

### Case A: \(\rho\in\mathbb{Q}\setminus\mathbb{Z}\)
Write \(\rho=p/q\) (coprime, \(q\ge2\)).
Then \(\Theta_k\) is periodic mod \(2\pi\) with period \(q\).
Over one period:
\[
\sum_{j=0}^{q-1} e^{i(2\pi pj/q+\beta_0)}
= e^{i\beta_0}\sum_{j=0}^{q-1}\omega^j = 0.
\]
Taking real parts:
\[
\sum_{j=0}^{q-1}\cos(2\pi pj/q+\beta_0)=0.
\]
Hence at least one period value is nonnegative; periodicity gives infinitely many \(k\) with \(\cos\Theta_k\ge0\).

### Case B: \(\rho\notin\mathbb{Q}\)
By Kronecker/Weyl equidistribution of irrational rotations,
\(\{k\rho+\beta_0/(2\pi)\}\) is equidistributed mod 1.
Therefore the set \(\{k:\cos\Theta_k\ge0\}\) has density \(1/2\), in particular is infinite.

### Conclusion
If \(\rho\notin\mathbb{Z}\), then infinitely many \(k\) satisfy \(\cos\Theta_k\ge0\).

## Numeric guardrail for this project
Using current tau constants:
- \(\tau_1=14.134725142\)
- \(\tau_2=21.022039639\)
- \(\rho=\tau_2/\tau_1\approx1.487262003881\in(1,2)\), so not an integer.

Artifact:
- `research/output/k1_w62_tau_ratio_noninteger_certificate_2026-02-24.md`

## Rotation-sequence evidence (finite-N)
For fitted \((\phi_1,\phi_2)\) from each window, rotation probe gives \(\approx 0.5\) nonnegative fraction up to \(N=200000\):
- `research/output/k1_w62_tau12_rotation_sign_probe_2026-02-24.md`

This supports the C2 direction numerically, but theorem closure still requires importing/proving the equidistribution step formally.
