# Plan: Resolve the V ≥ 0 / box-energy circularity

**Date:** 2026-02-07  
**Status:** Open  
**File:** `run_out/paper1_zerozeta-v19.tex`, Proposition `prop:Cbox-finite`

---

## The problem (precise statement)

We need an **unconditional** bound on the weighted Dirichlet energy

$$
E(I) := \iint_{Q(\alpha' I)} |\nabla U|^2 \,\sigma\,d\sigma\,dt
$$

where $U = \log|\mathcal J_{\rm out}|$ on $\Omega \setminus Z(\zeta)$ and $I$ is a Whitney interval at scale $L = c/\log\langle t_0\rangle$.

The current proof sets $V = -U \ge 0$ and applies a strip comparison + interior gradient estimate.  
**The gap:** $V \ge 0$ is equivalent to $|\mathcal J_{\rm out}| \le 1$, which amounts to $\mathcal J_{\rm out}$ being an inner function — and that is essentially the *conclusion* of the theorem (no zeros of $\zeta$ in $\Omega$).

Near a zero $\rho$ of $\zeta$ (a pole of $\mathcal J_{\rm out}$), $|J_{\rm out}| \to \infty$, so $V \to -\infty$.  
Even after Blaschke neutralization, zeros of $\zeta$ just *outside* the box can make $|\mathcal J_{\rm out}|$ large on $\partial D$, violating $\tilde V \ge 0$.

**What we need instead:** A bound $E(I) \le C(L) \cdot |I|$ where $\sqrt{C(L)} \cdot L \to 0$ as $c \to 0$, proved without assuming $V \ge 0$ (or any inner/Schur property of $\mathcal J_{\rm out}$).

---

## What we have unconditionally

| Fact | Source | Status |
|------|--------|--------|
| $\det_2(I-A)$ holomorphic, zero-free on $\Omega$ | Hilbert–Schmidt theory | ✅ |
| $\mathcal O_\zeta$ outer, zero-free on $\Omega$ | Outer function construction | ✅ |
| $\|\mathcal J_{\rm out}(\tfrac12+it)\| = 1$ a.e. | By construction ($F/\mathcal O_\zeta$) | ✅ |
| $U = 0$ on $\partial\Omega$ (a.e.) | From $\|J_{\rm out}\| = 1$ on boundary | ✅ |
| $U$ harmonic on $Q(\alpha'I)$ after neutralizing $O(1)$ near zeros | Blaschke neutralization + RvM | ✅ |
| $V \le C_0 \sigma$ on $\{0 < \sigma < 1\}$ | Strip comparison (upper bound on $V = -U$) | ✅ |
| Phase-velocity: $-w' \ge 0$ (positive measure) | Canonical factorization on Whitney regions | ✅ |
| CR–Green: $\int\phi(-w') \le C_{\rm test}\sqrt{E(I)}$ | Green's identity + Cauchy–Schwarz | ✅ |
| $N(T;H) \le A_0 + A_1 H\log\langle T\rangle$ | Riemann–von Mangoldt (1905) | ✅ |

**What we do NOT have unconditionally:**
- $V \ge 0$ (equivalently $|\mathcal J_{\rm out}| \le 1$) — **this is the gap**
- A lower bound on $U$ (or equivalently an upper bound on $|\mathcal J_{\rm out}|$) on the interior of $\Omega$

---

## Candidate strategies

### Strategy A: Carleson embedding for the Cauchy transform

**Idea:** Bypass the $V \ge 0$ issue entirely by bounding $E(I)$ directly via the logarithmic derivative decomposition

$$
\frac{\mathcal J_{\rm out}'}{\mathcal J_{\rm out}}
= \frac{(\det_2)'}{(\det_2)}
- \frac{\mathcal O_\zeta'}{\mathcal O_\zeta}
- \frac{\zeta'}{\zeta}
+ \frac{1}{s-1} - \frac{1}{s},
$$

so $|\nabla U|^2 = |\mathcal J_{\rm out}'/\mathcal J_{\rm out}|^2$, and

$$
E(I) \le 5\bigl(E_{\det_2} + E_{\rm out} + E_\zeta + E_{\rm smooth}\bigr).
$$

- $E_{\det_2} \le K_0 |I|$ — **done** (Lemma `carleson-arith`).
- $E_{\rm out} \le C'|I|$ — follows from BMO/Fefferman–Stein if $\log|\mathcal O_\zeta|$ has Carleson gradient. Needs explicit verification (see sub-task A2).
- $E_{\rm smooth} = O(|I|)$ — trivial.
- **$E_\zeta$** = $\iint_Q |\zeta'/\zeta|^2 \sigma$ — **this is the hard term**.

The hard term $E_\zeta$ involves the Cauchy transform of the zero-counting measure:

$$
\frac{\zeta'}{\zeta}(s) = B_0 + \sum_\rho \Bigl(\frac{1}{s-\rho} + \frac{1}{\rho}\Bigr)
$$

where the sum runs over nontrivial zeros. On $Q(\alpha'I)$, after subtracting the smooth $B_0 + \sum 1/\rho$ part, we need:

$$
\iint_{Q(\alpha'I)} \Bigl|\sum_\rho \frac{1}{s-\rho}\Bigr|^2 \sigma\,d\sigma\,dt \;\le\; C\,|I|.
$$

**Key theorem needed (Carleson embedding for the Cauchy transform):**

> *If $\mu$ is a positive measure on $\Omega$ satisfying the Carleson condition $\mu(Q(J)) \le C_0 |J|$ for every interval $J$, and $\mathcal C\mu(s) = \int (s-w)^{-1}\,d\mu(w)$, then $|\mathcal C\mu|^2 \sigma\,d\sigma\,dt$ is also a Carleson measure with norm $\le C(C_0)$.*

This is a consequence of the **T(1) theorem** (David–Journé, 1984) or can be proved via the **Carleson embedding theorem** combined with duality. The zero-counting measure $\mu_\zeta$ is Carleson by Riemann–von Mangoldt.

**Sub-tasks:**

| ID | Task | Status |
|----|------|--------|
| A1 | State the Carleson-embedding/T(1) result precisely for the half-plane Cauchy kernel | ☐ |
| A2 | Verify $E_{\rm out} \le C'|I|$ (outer function gradient is Carleson) | ☐ |
| A3 | Verify the zero-counting measure $\mu_\zeta$ is Carleson on Whitney scale (already essentially done via RvM) | ☐ |
| A4 | Apply the theorem to conclude $E_\zeta \le C|I|$ | ☐ |
| A5 | Assemble: $E(I) \le C_{\rm univ}|I|$, giving $\sqrt{E}\cdot L \le C_{\rm univ}^{1/2} L^{3/2} \to 0$ | ☐ |

**Pros:** Gives a *uniform* (height-independent) $C_{\rm box}$. No $V \ge 0$ needed. Uses only the zero density (RvM), not the location of zeros.  
**Cons:** Requires citing a nontrivial theorem (T(1) or Carleson embedding for Cauchy transforms). The precise formulation for the half-plane with the $\sigma$-weight needs care.

**Key references:**
- David–Journé, "Opérateurs de Calderón–Zygmund…" (Ann. Math. 1984)
- Christ, "Lectures on Singular Integral Operators" (CBMS 1990)
- Garnett, *Bounded Analytic Functions*, Ch. VIII (Carleson measures and BMO)
- Stein, *Harmonic Analysis*, Ch. II, VII
- Tolsa, "L² boundedness of the Cauchy integral operator for continuous measures" (Duke 2003) — more general but covers our case

---

### Strategy B: Two-sided strip comparison (no inner factorization)

**Idea:** Instead of using $V \ge 0$, bound $|U|$ on the strip directly using growth estimates for each factor of $\mathcal J_{\rm out}$.

On the strip $\{0 < \sigma < 1\}$:

$$
U(\tfrac12+\sigma+it) = \log|\det_2| - \log|\mathcal O_\zeta| - \log|\zeta| + \log|s-1| - \log|s|
$$

- $|\log|\det_2(\tfrac12+\sigma+it)|| \le C_1(\sigma)$ — from the explicit prime product (converges for $\sigma > 0$)
- $|\log|\mathcal O_\zeta(\tfrac12+\sigma+it)|| \le C_2(\sigma, t)$ — Poisson integral of boundary data
- $\log|\zeta(\tfrac12+\sigma+it)|$ — the problematic term; near a zero $\rho$, $\log|\zeta| \to -\infty$
- Smooth terms: $O(\log|t|)$

**Problem with this approach:** The $\log|\zeta|$ term is *unbounded below* near zeros of $\zeta$. So we can't get a two-sided bound $|U| \le M$ without knowing where the zeros are.

After Blaschke neutralization of near zeros:

$$
\tilde U = U - \sum_j \log|b(s,\rho_j)| = \log|\mathcal J_{\rm out} \cdot B_{\rm near}|
$$

is harmonic on $D$, but we need $|\tilde U| \le M$ on $\partial D$. On the top edge, $|\tilde U|$ involves $|\log|\zeta \cdot B_{\rm near}^{-1}||$ which blows up near un-neutralized zeros outside $D$.

**Verdict:** This approach has the same fundamental obstacle — controlling $|U|$ or $|\tilde U|$ from both sides near poles/zeros.

**Sub-tasks (if pursued):**

| ID | Task | Status |
|----|------|--------|
| B1 | Obtain an unconditional upper bound on $|\zeta(\tfrac12+\sigma+it)|^{-1}$ for $0 < \sigma \le 1$ | ☐ |
| B2 | Determine if classical convexity/subconvexity bounds suffice for the needed growth rate | ☐ |
| B3 | Check if the resulting $M$ is small enough that $\sqrt{M^2|I|}\cdot L \to 0$ | ☐ |

**Pros:** Would avoid citing heavy machinery.  
**Cons:** Likely fails because $|\zeta|^{-1}$ grows as $\exp(c\log T/\log\log T)$ unconditionally (Littlewood), which is too fast.

---

### Strategy C: Work with $\zeta'/\zeta$ directly on the boundary (avoid interior energy)

**Idea:** Instead of bounding interior gradient energy $E(I)$ and using CR–Green, bound the *boundary* phase variation $\int_I (-w')$ directly using the phase-velocity identity and Poisson averaging.

From the phase-velocity identity:

$$
\int_I (-w') = \pi \int_I d\mu_{\rm off} + \pi \int_I d\nu_{\rm sing} + \pi \sum_{\gamma \in I} m_\gamma
$$

All terms are positive. The question is whether each is $\ll L$ on Whitney scale.

- **$\mu_{\rm off}$ on $I$:** This is the Poisson balayage of off-critical zeros. For a zero at $\rho = \beta+i\gamma$ with $\beta > 1/2$:

$$
\int_I d\mu_{\rm off}(\rho) = \int_I \frac{2(\beta-1/2)}{(\beta-1/2)^2 + (t-\gamma)^2}\,dt
$$

For $\gamma \in I$ (zero at the same height as the window): contribution $\approx 2\pi$ if $\beta - 1/2 \gg L$, or $\approx 4L/(\beta-1/2)$ if $\beta - 1/2 \ll L$.

The total from all zeros: $\sum_\rho (\text{Poisson on } I)$. This is controlled if there are few zeros close to $I$ (by RvM) and the far zeros contribute small Poisson tails.

- **$\nu_{\rm sing}$ on $I$:** The singular inner measure. No a priori bound without knowing $\mathcal J_{\rm out}$ better.

- **Atoms on $I$:** Zeros/poles on the critical line. Controlled by RvM.

**Problem:** The $\nu_{\rm sing}$ term has no unconditional bound. And even for $\mu_{\rm off}$, the total Poisson mass on $I$ from ALL zeros is a sum that requires careful handling (similar to the annular-sum issue).

**Verdict:** This approach trades the interior energy gap for a boundary singular-measure gap. It doesn't obviously help.

---

### Strategy D: Modify the proof architecture to avoid the energy bound entirely

**Idea:** Instead of the CR–Green → energy → contradiction chain, use a different mechanism to detect zeros.

Possible alternatives:
1. **Jensen's formula** on discs centered at the hypothetical zero: directly relate the number of zeros to the boundary average of $\log|F|$.
2. **Argument principle** on a contour enclosing the hypothetical zero: count zeros via the winding number of $\mathcal J_{\rm out}$.
3. **Turán's power-sum method** or **explicit-formula approach**: bound the contribution of a hypothetical zero to an explicit sum.

These are all classical zero-detection tools. The question is whether any of them can be made to work with the $\det_2/\zeta$ ratio and give the needed $\sigma \ge 0.6$ zero-free region.

**Sub-tasks:**

| ID | Task | Status |
|----|------|--------|
| D1 | Evaluate Jensen's formula for $\mathcal J_{\rm out}$ on discs at height $T$ | ☐ |
| D2 | Check if the explicit formula for $\sum \Lambda(n) n^{-s}$ can be adapted | ☐ |
| D3 | Survey whether any known zero-detection method gives $\sigma \ge 0.6$ unconditionally | ☐ |

**Pros:** Might bypass all the Hardy-space/energy machinery.  
**Cons:** Known zero-detection methods give zero-free regions of the form $\sigma > 1 - c/\log T$ (de la Vallée-Poussin type), not fixed $\sigma \ge 0.6$.

---

## Recommended path

**Strategy A (Carleson embedding)** is the most promising because:

1. It provides a *uniform* $C_{\rm box} < \infty$ (not height-dependent).
2. It requires only the Carleson property of $\mu_\zeta$ (which follows from RvM).
3. The T(1) theorem / Carleson embedding for Cauchy transforms is a well-established result with standard references.
4. It avoids the $V \ge 0$ issue entirely — it bounds $E_\zeta$ via the *density* of zeros, not their *location* relative to the box.

**Immediate next steps:**

1. [ ] **Find and cite** the precise version of the Carleson-embedding/Cauchy-transform theorem needed (half-plane, $\sigma$-weighted measure).
2. [ ] **Verify** that the $\zeta$-zero counting measure is Carleson in the required sense (this should follow directly from RvM + Whitney scaling).
3. [ ] **Verify** the outer-function gradient bound $E_{\rm out} \le C'|I|$.
4. [ ] **Rewrite** `prop:Cbox-finite` using the decomposition $E \le 5(E_{\det_2} + E_{\rm out} + E_\zeta + E_{\rm smooth})$ and cite the Carleson embedding result.
5. [ ] **Re-audit** the full proof chain.

---

## Open questions

1. **Is the $\sigma$-weighted Carleson embedding theorem for Cauchy transforms in the standard literature?** The unweighted version ($dA$ instead of $\sigma\,dA$) is standard. The $\sigma$-weighted version is equivalent (by conformal mapping to the disc) but needs a precise citation.

2. **Does the outer-function gradient term $E_{\rm out}$ require the T(1) theorem too, or does the BMO/Fefferman–Stein route suffice?** This term involves $|\mathcal O_\zeta'/\mathcal O_\zeta|^2 \sigma$ where $\mathcal O_\zeta$ is the Poisson extension of the boundary log-modulus. If $\log|F^*| \in \text{BMO}$, then its harmonic extension has Carleson gradient energy. This should follow from the Fefferman–Stein characterization without needing T(1).

3. **Can the singular inner factor $\nu_{\rm sing}$ be shown to be trivial?** If $\mathcal J_{\rm out}$ has no singular inner factor (i.e., $\nu_{\rm sing} = 0$), several arguments simplify. This might follow from the specific structure of $\det_2/\zeta$ but requires a separate argument.
