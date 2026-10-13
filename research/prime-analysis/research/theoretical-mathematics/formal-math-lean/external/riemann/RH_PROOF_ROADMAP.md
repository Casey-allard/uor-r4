## RH boundary-wedge route: current status, coverage, and proof map

This note is a **referee-facing map** for the “boundary wedge (P+) → Herglotz/Schur → pinch” route currently written in `Riemann-active.tex`.

### What this route claims (coverage)

`Riemann-active.tex` states **Theorem `thm:RH` (Riemann Hypothesis)**: there are **no zeros of** \(\xi(s)\) in the open half-plane \(\Omega=\{\Re s>\tfrac12\}\); by the functional equation and symmetry, this implies **all nontrivial zeros of** \(\zeta(s)\) lie on \(\Re s=\tfrac12\).

Equivalently, the claimed coverage is **full RH**, i.e. the entire critical strip \((0<\Re s<1)\) with the exact critical line location of nontrivial zeros.

### Unconditional status (as written)

The manuscript is written as **unconditional** (no hypotheses such as RH or “cutoff” assumptions). Numerical constants are present but are explicitly framed as **diagnostic / non-load-bearing** and can be gated by macros (see `\numericlockfalse` and `\shownumericsfalse` near the top of `Riemann-active.tex`).

That said, because RH is a major open problem, **independent expert verification is required**. This roadmap is meant to make that verification effort efficient by listing the exact logical dependencies.

### Proof chain (high level)

1. **Definitions**:
   - Define the prime-diagonal operator \(A(s)\) on \(\ell^2(\mathbb P)\) and the regularized determinant \(\det_2(I-A(s))\).
   - Define a normalized ratio \(\mathcal J(s)\) and Cayley field \(\Theta(s)\) (see “Standing setup” in the globalization section):
     \[
       \mathcal J(s)=\frac{\det_2(I-A(s))}{\mathcal O(s)\,\xi(s)},\quad F(s)=2\mathcal J(s),\quad
       \Theta(s)=\frac{F(s)-1}{F(s)+1}.
     \]

2. **Boundary phase–velocity identity**:
   - Theorem `thm:phase-velocity-quant` produces a distributional identity relating the boundary phase derivative \(-w'(t)\) to a nonnegative measure sourced by off-critical zeros, after outer normalization and boundary passage.

3. **Certificate ⇒ boundary wedge (P+)**:
   - Theorem `thm:psc-certificate-stage2` (“Boundary wedge from the product certificate (atom-safe)”) proves the boundary wedge inequality (P+) on \(\Re s=\tfrac12\):
     \[
       \Re\,F\!\left(\tfrac12+it\right)\ge 0 \quad \text{for a.e. }t,
     \]
     using:
     - a Poisson plateau lower bound \(c_0(\psi)\),
     - a CR–Green pairing upper bound with test functions on Whitney boxes,
     - an all-interval Carleson/box-energy bound \(C_{\rm box}^{(\zeta)}<\infty\) (built from a prime-side bound \(K_0\) and a neutralized zero-side bound \(K_\xi\)),
     - a quantitative wedge criterion requiring \(\Upsilon_{\rm Whit}(c)<1/2\).

4. **(P+) ⇒ interior Herglotz/Schur on \(\Omega\setminus Z(\xi)\)**:
   - In `sec:globalization`, the manuscript uses the Poisson integral to propagate the boundary inequality into the half-plane (on \(\Omega\setminus Z(\xi)\), where division by \(\xi\) makes sense), giving \(\Re F\ge 0\) (Herglotz), hence \(|\Theta|\le 1\) (Schur).

5. **Pinch/removability across a putative zero**:
   - Under two standing properties:
     - (N1) right-edge normalization \(\Theta(\sigma+it)\to-1\) as \(\sigma\to+\infty\) (uniformly on compact \(t\)-intervals),
     - (N2) non-cancellation at \(\xi\)-zeros (\(\det_2(I-A(\rho))\neq0\) and \(\mathcal O(\rho)\neq0\)),
   the argument shows any off-critical zero \(\rho\in\Omega\) would force \(\Theta(\rho)=1\) while the Schur bound plus maximum modulus forces \(\Theta\) to be a unimodular constant on a connected punctured domain; comparing to (N1) yields a contradiction. Hence no such \(\rho\) exists.

6. **Conclude RH**:
   - With \(Z(\xi)\cap\Omega=\varnothing\), symmetry and the functional equation force nontrivial zeros to lie on \(\Re s=\tfrac12\).

### Where the main “analytic number theory” input sits

The nontrivial quantitative input is the Carleson/box-energy control that feeds the wedge criterion:

- \(K_0\): prime-tail side bound (from explicit prime sum/tail inequalities; see the “prime tail” enclosure sections).
- \(K_\xi\): “neutralized zero energy” bound derived from unconditional zero-density (Vinogradov–Korobov type) plus annular \(L^2\) aggregation; see Appendix `app:vk-annuli-kxi`.

The manuscript emphasizes **non-circular sequencing**: \(K_\xi\) is derived independently of any quantity (like \(M_\psi\)) that depends on \(K_\xi\).

### Suggested next concrete step

If the goal is a referee-ready RH submission packet, the immediate task is to produce an **audit checklist** for the analytic chain (not numerical artifacts):

- Each “Assume (P+)” passage should point explicitly to Theorem `thm:psc-certificate-stage2`.
- Each “Assume Carleson box-energy bound” passage should point to the exact lemma/corollary proving it (and the exact VK zero-density citation and constants used).

