# K1 W49 Analytic Tail-Split Assumption Note (2026-02-24)

## Objective
Pin the exact analytic assumption needed to close the admissible-mode gate, using current robust tau constants and published explicit envelopes.

## New artifact
- Gap ledger:
  - `/Users/adminamn/Documents/New project/research/output/k1_w49_mode_admissibility_assumption_gap_2026-02-24.md`

## Core numeric conclusion
For robust admissible taus (`beta=0.62`, `a0=0.98`), the normalized target channel is:
\[
\frac{|R(x)|}{A} < a_0.
\]

Using published global explicit envelopes mapped to
\[
\frac{|E(x)|}{A x^\beta},
\]
the best-case gap factors are huge:

- min FKS gap at `x=1e7`: `~3.17e5`
- min FKS gap at `x=1e21`: `~2.85e10`
- min Bellotti-like gap at `x=1e7`: `~6.66e2`
- min Bellotti-like gap at `x=1e21`: `~4.23e6`

So global PNT-error envelopes cannot directly certify the admissibility inequality in this normalization.

## Mathematical implication
The missing theorem input is not another global bound; it is a **mode-specific aligned remainder bound**.

Required assumption shape (candidate):
\[
\exists \tau\in\mathcal T,\ \exists A_\tau>0,\ \exists \phi_\tau,\ \exists q_\tau<a_0,\ \exists X_0,\ \forall X\ge X_0\ \exists x\ge X:
\]
\[
|\cos(\tau\log x+\phi_\tau)|\ge a_0
\quad\text{and}\quad
\frac{|R_\tau(x)|}{A_\tau}\le q_\tau.
\]

Then with \(\delta_\tau=a_0-q_\tau>0\):
\[
|E_\*(x)|\ge \delta_\tau A_\tau x^\beta
\]
on a cofinal aligned subsequence, giving the contradiction-route lower envelope.

## Why this resolves current loop
It separates:
1. **What global explicit bounds can do** (coarse global control, insufficient for admissibility ratio),
2. **What we actually need** (aligned-mode tail cancellation control).

This is now the single theorem-grade mathematical gate to attack.

## Next theorem-facing step
Draft a precise “Aligned Mode Admissibility Lemma” with assumptions stated in explicit-formula components (`main mode + omitted tail + smoothing remainder`) and map each assumption to either:
- existing published theorem inputs, or
- a clearly isolated new lemma to prove.

