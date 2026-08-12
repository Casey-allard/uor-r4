# Audit Log: “Unconditional proof route” check for `paper1_zerozeta-v19.tex`

**Goal of this document**

Confirm (only) the following:

- The manuscript’s **proof route is written as an unconditional implication chain** to its stated claim.
- The previously identified bottleneck around **finiteness of the box-energy constant** is now explicitly discharged *in-text* (via a standalone Green-potential lemma + a proposition that uses it).

Explicitly *not* claimed here:

- I am **not certifying the mathematical truth** of the theorem’s conclusion.
- I am **not** providing an expert referee report on correctness of every cited classical theorem—only that the manuscript’s dependence graph is unconditional and the gap is closed at the level of stated lemmas/propositions.

**Audit target**

- Primary file (user-attached): `/Users/jonathanwashburn/.cursor/worktrees/Riemann/kuc/run_out/paper1_zerozeta-v19.tex`
- Secondary copy (workspace): `/Users/jonathanwashburn/Projects/Riemann/run_out/paper1_zerozeta-v19.tex`

If these differ materially, this audit applies to the **primary file** above.

---

## Plan (what I will do)

1. **Extract the stated claim** verbatim (Theorem label + region).
2. **Trace the proof references end-to-end**:
   - Theorem → “proof section” → (P+) → Appendix proof chain → return to Schur/pinch mechanism.
3. **List “load-bearing” steps** (anything required for the implication chain to reach the theorem).
4. **Verify the bottleneck closure**:
   - Standalone lemma: “\(U=\Re\log J\) is (up to compensator) a Green potential of the \(\xi\)-zero configuration”
   - Proposition: “\(C_{\rm box}^{(\zeta)}<\infty\)” (Whitney-uniform box energy), and that later lemmas actually use it.
5. **Check for hidden assumptions**:
   - Any “if we assume …” statements
   - Any computation that becomes load-bearing
   - Any conditional macros that could hide load-bearing content (e.g. `\rsadd{...}`), or any non-defined macro use.
6. **Write a conclusion**:
   - The claim (verbatim)
   - A one-paragraph statement of what is confirmed (unconditional chain as written) and what is not (truth certification).
   - A short “risk register” (weakest points a referee would probe).

---

## Progress checklist (live)

- [x] **P0** Create this audit log document.
- [x] **P1** Record the theorem/claim verbatim.
- [x] **P2** Map the dependency chain (Theorem → Schur bound → (P+) → appendix lemmas).
- [x] **P3** Confirm the Green-potential lemma exists and is used by the box-energy proposition.
- [x] **P4** Confirm the box-energy finiteness statement is used where the wedge assembly needs it.
- [x] **P5** Confirm the final theorem uses only the analytic chain; computation is explicitly non-load-bearing.
- [x] **P6** Scan for hidden assumptions / conditional content toggles / macro hazards.
- [x] **P7** Write final conclusion + risks.

---

## Findings (filled in as I go)

### Claim (verbatim)

**Source:** `/Users/jonathanwashburn/.cursor/worktrees/Riemann/kuc/run_out/paper1_zerozeta-v19.tex`, Theorem `\ref{thm:farfield}`.

> **Theorem (far-field zero-freeness).** *The Riemann zeta function has no zeros in the region*  
> \(\{\,s\in\mathbb C:\ \Re s\ge 0.6\,\}\).

The abstract also states the equivalent claim: \(\zeta(s)\neq 0\) for \(\Re s\ge 0.6\).

### Dependency chain (high level)

**Status:** recorded (labels traced in the audit target file).

Load-bearing implication chain (as written):

1. **Main claim** is Theorem `\ref{thm:farfield}` (“no zeros for \(\Re s\ge 0.6\)”).
2. The proof is in `\S\ref{sec:proof-farfield}` and proceeds:
   - `\ref{prop:herglotz-schur-transport}` gives that the Cayley field \(\Theta_{\rm out}\) is **Schur** on each connected component of \(\Omega\setminus Z(\zeta)\).
   - `\ref{cor:no-poles}` (Schur/Herglotz pinch) is applied on a half-plane \(U_\varepsilon=\{\Re s>0.6-\varepsilon\}\) to conclude **no poles** of \(\widetilde{\mathcal J}=e^{-im}\mathcal J_{\rm out}\) (hence none for \(\mathcal J_{\rm out}\)).
   - Since \(\det_2(I-A)\) and the outer factor \(\mathcal O_\zeta\) are holomorphic and nonvanishing on \(\Omega\), poles of \(\mathcal J_{\rm out}\) inside \(\Omega\) can only come from **zeros of \(\zeta\)**; therefore there are no zeros in \(\{\Re s\ge 0.6\}\).
3. `\ref{prop:herglotz-schur-transport}` depends on:
   - The boundary wedge certificate **(P+)**, stated as Theorem `\ref{thm:Pplus}`.
   - Smirnov/outer regularity (Lemma `\ref{lem:smirnov-regularity}`), which depends on `\ref{lem:F-boundary-admissible}` and `\ref{lem:outer-factor-halfplane}`.
4. Theorem `\ref{thm:Pplus}` is proved in Appendix A (Appendix `\ref{app:pplus-proof}`), explicitly pointing to §§`\ref{app:phase-velocity}`–`\ref{app:assemble-pplus}`.
5. In Appendix A, the final proof of (P+) (Theorem `\ref{thm:pplus-proof-complete}`) uses:
   - The phase–velocity identity (Theorem `\ref{thm:phase-velocity-quant}`),
   - The CR–Green / “length-free” inequality (Proposition `\ref{prop:length-free}`) which bounds a windowed phase derivative by \(\sqrt{C_{\rm box}^{(\zeta)}}\),
   - The finiteness of the box-energy constant \(C_{\rm box}^{(\zeta)}\) (Proposition `\ref{prop:Cbox-finite}`),
   - The Whitney window schedule / wedge conversion (Lemma `\ref{lem:whitney-uniform-wedge}` and Lemma `\ref{lem:local-to-global-wedge}`).

Non-load-bearing computation is stated explicitly in the introduction and by the table caption “Supplementary computational artifacts (not used in the proof)”.

### Bottleneck: \(C_{\rm box}^{(\zeta)}<\infty\)

**Status:** recorded (lemma + proposition present and referenced in the Appendix A assembly).

What was added/required for closing the gap:

- **Green potential lemma (standalone).** Lemma `\ref{lem:green-potential-Jout}` states (distributionally on \(\Omega\)) that
  \[
    -\Delta\bigl(U-2\log|B|\bigr)=2\pi\,\mu_\xi,
    \qquad U=\Re\log\mathcal J_{\rm out},\quad B(s)=(s-1)/s,
  \]
  i.e. **up to the explicit compensator \(2\log|B|\)**, the field \(U\) is the **Green potential** of the \(\xi\)-zero configuration in \(\Omega\).

- **Whitney-uniform finiteness proposition.** Proposition `\ref{prop:Cbox-finite}` defines
  \[
  C_{\rm box}^{(\zeta)}:=\sup_{I=[t_0-L,t_0+L],\,L=c/\log\angles{t_0}}
  \frac1{|I|}\iint_{Q(\alpha'I)}|\nabla U|^2\,\sigma,
  \]
  and proves \(C_{\rm box}^{(\zeta)}<\infty\), with an explicit (large) bound \(C_{\rm univ}(\alpha',c)\).

How the proof uses the Green-potential lemma:

- In the proof of Proposition `\ref{prop:Cbox-finite}`, the first step explicitly cites Lemma `\ref{lem:green-potential-Jout}` to identify \(U\) (mod compensator) with the \(\xi\)-zero Green potential, and then bounds the weighted Dirichlet energy on Whitney boxes by:
  - decomposing zeros into Whitney annuli in ordinate,
  - applying the annular Poisson–balayage \(L^2\) estimate (Lemma `\ref{lem:annular-balayage}`),
  - using only the short-interval Riemann–von Mangoldt zero-density bound on Whitney scale.

Where finiteness is consumed downstream (so it is genuinely load-bearing):

- The final Appendix A proof of (P+) (Theorem `\ref{thm:pplus-proof-complete}`) cites Proposition `\ref{prop:Cbox-finite}` as the step establishing \(C_{\rm box}^{(\zeta)}<\infty\), which is then fed into the Whitney-uniform wedge lemma `\ref{lem:whitney-uniform-wedge}` to get \(\Upsilon_{\rm Whit}(c)<1/2\) for small \(c\).

### Hidden assumptions / hazards scan

**Status:** recorded.

#### Explicit “unconditional / not RH” positioning

- The paper explicitly frames the result as an “unconditional, fixed half-plane statement” (Introduction).
- It explicitly states: **“We do not claim the Riemann Hypothesis here.”** (Conclusion section).
- A text search found **no** “Assume RH / under RH / assuming RH” style dependency.

#### Computation is non-load-bearing (explicitly stated)

The manuscript repeatedly states the computational artifacts are supplementary and **not used** in the logical proof chain:

- “not logically required for the all-heights implication chain in the proof” (Introduction).
- “Supplementary computational cross-checks (not used in the proof)” (Introduction header).
- Table caption: “Supplementary computational artifacts (not used in the proof).”

This is consistent with the proof references: Theorem `\ref{thm:farfield}` points only to `\S\ref{sec:proof-farfield}`, and (P+) points only to Appendix A sections.

#### Macro hazards / conditional toggles

- Editorial macros are defined in the preamble:
  - `\editblue{...}` and `\editgreen{...}` are **identity** (disabled markup).
  - `\rsadd{...}` is defined as **colored text** (`{\color{violet}#1}`), so it **does not hide** content.
- Note: Proposition `\ref{prop:Cbox-finite}` (load-bearing) and an expository remark are wrapped in `\rsadd{...}`. This is logically fine *given the current macro definition*, but it is a mild “fragility” if someone later redefines `\rsadd` to drop its argument.

#### Minor typesetting issue (non-logical)

- In Table 1’s tabular, there is a stray literal `ottomrule` line (should likely be `\bottomrule`). This does not affect the proof but will affect final PDF polish.

#### File identity check

- The “primary” file (user-attached worktree path) and the workspace copy compare equal (byte-identical) via `diff -q`, so the audit applies to both paths.

---

## Conclusion (to be completed)

**Claim (verbatim, Theorem `\ref{thm:farfield}`):** The Riemann zeta function has no zeros in the region \(\{s\in\mathbb C:\Re s\ge 0.6\}\).

**Confirmed (what this audit *does* certify):**

- The manuscript’s proof route is written as an **unconditional implication chain** to the stated claim.
- The previously flagged bottleneck “\(C_{\rm box}^{(\zeta)}<\infty\)” is now discharged explicitly in-text via:
  - the standalone Green-potential lemma `\ref{lem:green-potential-Jout}`, and
  - the Whitney-uniform finiteness proposition `\ref{prop:Cbox-finite}`,
  and the (P+) assembly explicitly cites `\ref{prop:Cbox-finite}` where the finiteness is consumed.
- The manuscript explicitly states the shipped computational artifacts are **supplementary** and **not used** in the all-heights proof chain, consistent with the internal references.

**Not claimed (what this audit does *not* certify):**

- I **cannot certify that the theorem is mathematically true**.
- I can only attest that the manuscript’s proof route is now written as an unconditional argument to the stated claim, with the missing structural lemma added and cited, and with the computation explicitly non-load-bearing.

**Risks / weakest points a referee would probe (still unconditional, but technically delicate):**

- **Appendix A technical heart**: the quantified phase–velocity identity `\ref{thm:phase-velocity-quant}` (canonical factorization on half-planes/components, distributional boundary argument, handling of possible singular inner factors).
- **Whitney box-energy bound**: Proposition `\ref{prop:Cbox-finite}` (the pointwise “Poisson-type” domination of Green-kernel gradients, and the summation over Whitney annuli using Lemma `\ref{lem:annular-balayage}` + short-interval zero counts).
- **Componentwise boundary transport** on \(\Omega\setminus Z(\zeta)\): use of “discrete implies polar implies harmonic-measure zero” in Lemma `\ref{lem:schur-transport-omega}`.
- **Editorial markup**: load-bearing content wrapped in `\rsadd{...}` is safe with the current definition (colored text), but could become fragile if `\rsadd` is later redefined to drop content.

---

## Deeper internal-math spot check (nontrivial)

### Check: geometric-series summability in the (now removed) Lemma `\ref{lem:carleson-xi}`

**Result (historical):** I found what appeared to be a **hard arithmetic error** in the written proof of Lemma `\ref{lem:carleson-xi}` (analytic Carleson energy for \(\xi\)).

**Status:** This was subsequently **repaired in the TeX source** by removing Lemma `\ref{lem:carleson-xi}` from the manuscript and rewriting downstream dependencies so the box-energy finiteness is proved via Lemma `\ref{lem:green-potential-Jout}` + Lemma `\ref{lem:annular-balayage}` + Riemann–von Mangoldt on Whitney scale (see Proposition `\ref{prop:Cbox-finite}` in the updated file).

In the proof, the far-zero contribution is bounded by a sum of the form
\[
|I|\sum_{k\ge 1} 2^{-k}\,\nu_k^2,
\]
and \(\nu_k\) is bounded by \(\nu_k\ll 2^kL\log\angles{T}+\log\angles{T}\).
Substituting this into \(\sum 2^{-k}\nu_k^2\) yields a term
\[
\sum_{k\ge 1} 2^{-k}\cdot 4^k\,L^2\log^2\angles{T}=\log^2\angles{T}\,L^2\sum_{k\ge 1}2^{k},
\]
which **diverges**.

However, the manuscript claims this sum is \(\ll L^2\log^2\angles{T}+\log^2\angles{T}\), which would require (incorrectly) that \(\sum_{k\ge 1}2^{-k}4^k\) converges.

Because Lemma `\ref{lem:carleson-xi}` is used later (e.g. in Proposition `\ref{prop:Cbox-finite}` and in the (P+) assembly), this arithmetic issue is **load-bearing**: as written, it means the proof does not currently establish the stated uniform Carleson bound.

**Interpretation:** The lemma statement may still be true, but the displayed estimate chain cannot be correct as written; it likely needs a different bound that is **linear in \(\nu_k\)** (not quadratic), e.g. via the already-stated annular balayage Lemma `\ref{lem:annular-balayage}` or a standard Cauchy-transform/Carleson-measure estimate that exploits cancellation/orthogonality rather than the crude \(\nu_k^2\) bound.

