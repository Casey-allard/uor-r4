# Three-Paper Split Progress Tracker

This file tracks the execution of `THREE_PAPER_SPLIT_PLAN.md`.

**Rule of engagement:** one LaTeX `\section{...}` written per session.

## Papers and status

### Paper 1 (Unconditional): `paper1_farfield.tex`

- **Title (working):** *A certified zero-free region for the Riemann zeta function in the half-plane \(\Re s \ge 0.6\)*  
- **Status:** completed (first complete draft)
- **Sections**
  - `\section{Introduction}` — **completed** (Session 1)
  - `\section{Definitions and main objects}` — **completed** (Session 2)
  - `\section{Schur/Herglotz pinch mechanism}` — **completed** (Session 3)
  - `\section{Hybrid certification: rectangle + Pick + asymptotics}` — **completed** (Session 4)
  - `\section*{Conclusion and limitations (unconditional status)}` — **completed** (Session 18)
  - `\section*{Statements and Declarations}` — **completed** (Session 20)
  - `\appendix` audit manifest section — **completed** (Session 5)

### Paper 2 (Effective): `paper2_energy_barrier.tex`

- **Title (working):** *Energy barriers and Carleson budgets for off-critical zeros of the Riemann zeta function*  
- **Status:** completed (first complete draft)
- **Sections**
  - `\section{Introduction}` — **completed** (Session 6)
  - `\section{Setup: windowing, phase conventions, and Carleson energy}` — **completed** (Session 7)
  - `\section{Phase-cost lower bound (Blaschke trigger)}` — **completed** (Session 8)
  - `\section{Carleson energy upper bound (CR--Green / box budget)}` — **completed** (Session 9)
  - `\section{Energy barrier inequality and definition of $T_{\rm safe}(\eta)$}` — **completed** (Session 10)
  - `\section{Discharging the budget bound: primes + zeros}` — **completed** (Session 11)
  - `\section*{Conclusion and limitations (effective status)}` — **completed** (Session 12)
  - `\section*{Statements and Declarations}` — **completed** (Session 21)

### Paper 3 (Conditional): `paper3_cutoff_conditional.tex`

- **Title (working):** *A cutoff principle and conditional closure of the Riemann Hypothesis*  
- **Status:** completed (first complete draft)
- **Sections**
  - `\section{Introduction}` — **completed** (Session 13; polished Session 51; polished Session 52)
  - `\section{From Nyquist cutoff to a uniform arithmetic blocker}` — **completed** (Session 14; polished Session 39; polished Session 53)
  - `\section{From the arithmetic blocker to a uniform Carleson bound}` — **completed** (Session 15; polished Session 42)
  - `\section{Barrier closure and conditional RH}` — **completed** (Session 16; polished Session 44; polished Session 50; polished Session 51)
  - `\section*{Conclusion and limitations (conditional status)}` — **completed** (Session 17; polished Session 45)
  - `\section*{Statements and Declarations}` — **completed** (Session 22; polished Session 46)

## Session log

- **Session 1 (2026-01-02):** completed
  - Created: `riemann_common_preamble.tex`, `riemann_bibliography.tex`
  - Created: `paper1_farfield.tex`, `paper2_energy_barrier.tex`, `paper3_cutoff_conditional.tex`
  - Wrote: Paper 1 — `\section{Introduction}` (and compiled `paper1_farfield.pdf`)
  - Next queued section: Paper 1 — `\section{Definitions and main objects}`

- **Session 2 (2026-01-02):** completed
  - Wrote: Paper 1 — `\section{Definitions and main objects}` (and compiled `paper1_farfield.pdf`)
  - Next queued section: Paper 1 — `\section{Schur/Herglotz pinch mechanism}`

- **Session 3 (2026-01-02):** completed
  - Wrote: Paper 1 — `\section{Schur/Herglotz pinch mechanism}` (and compiled `paper1_farfield.pdf`)
  - Next queued section: Paper 1 — `\section{Hybrid certification: rectangle + Pick + asymptotics}`

- **Session 4 (2026-01-02):** completed
  - Wrote: Paper 1 — `\section{Hybrid certification: rectangle + Pick + asymptotics}` (and compiled `paper1_farfield.pdf`)
  - Next queued section: Paper 1 — optional `\appendix` audit table + artifact manifest

- **Session 5 (2026-01-02):** completed
  - Wrote: Paper 1 — appendix audit manifest section (and compiled `paper1_farfield.pdf`)
  - Next queued section: Paper 2 — `\section{Introduction}`

- **Session 18 (2026-01-02):** completed
  - Wrote: Paper 1 — `\section*{Conclusion and limitations (unconditional status)}` (and compiled `paper1_farfield.pdf`)
  - Next queued section: (none; Paper 1 remains complete)

- **Session 19 (2026-01-02):** completed
  - Paper 3 polish: updated the definition/proof of the Carleson box energy in `\section{From the arithmetic blocker to a uniform Carleson bound}` to match the weighted half-plane convention (and compiled `paper3_cutoff_conditional.pdf`)

- **Session 20 (2026-01-02):** completed
  - Wrote: Paper 1 — `\section*{Statements and Declarations}` (and compiled `paper1_farfield.pdf`)

- **Session 21 (2026-01-02):** completed
  - Wrote: Paper 2 — `\section*{Statements and Declarations}` (and compiled `paper2_energy_barrier.pdf`)

- **Session 22 (2026-01-02):** completed
  - Wrote: Paper 3 — `\section*{Statements and Declarations}` (and compiled `paper3_cutoff_conditional.pdf`)

- **Session 23 (2026-01-02):** completed
  - Paper 3 polish: reflowed the conclusion paragraph(s) to eliminate an overfull hbox warning (and compiled `paper3_cutoff_conditional.pdf`)

- **Session 24 (2026-01-02):** completed
  - Paper 3 polish: updated `\section*{Statements and Declarations}` to use consistent LaTeX formatting for file references (and compiled `paper3_cutoff_conditional.pdf`)

- **Session 25 (2026-01-02):** completed
  - Added `Makefile` with `make paper1/paper2/paper3/all/clean` targets and updated `README.md`; verified `make all` compiles all three papers.

- **Session 26 (2026-01-02):** completed
  - Paper 3 polish: made the `\section*{Statements and Declarations}` pointer consistent (“Paper~I” terminology) and recompiled `paper3_cutoff_conditional.pdf`.

- **Session 27 (2026-01-02):** completed
  - Paper 3 polish: cleaned “draft scaffolding” in the abstract/Introduction (removed “in preparation / to be written” language; updated the roadmap to point to existing sections) and recompiled `paper3_cutoff_conditional.pdf`.

- **Session 28 (2026-01-02):** completed
  - Paper 2 polish: cleaned “draft scaffolding” in the abstract/Introduction (removed “in preparation / will produce” language; strengthened the roadmap with section cross-references) and recompiled `paper2_energy_barrier.pdf`.

- **Session 29 (2026-01-02):** completed
  - Paper 1 polish: tightened the Pick-tail perturbation argument and upgraded the large-height tail lemma write-up inside `\section{Hybrid certification: rectangle + Pick + asymptotics}`; recompiled `paper1_farfield.pdf`.

- **Session 30 (2026-01-02):** completed
  - Paper 2 polish: removed “(to be proved later)” scaffolding from Proposition~`prop:budget-bound` inside `\section{Energy barrier inequality and definition of $T_{\rm safe}(\eta)$}`, added an explicit pointer to Section~`sec:budget`, and recompiled `paper2_energy_barrier.pdf`.

- **Session 31 (2026-01-02):** completed
  - Paper 2 polish: upgraded the CR--Green lemma write-up inside `\section{Carleson energy upper bound (CR--Green / box budget)}` (removed “Proof sketch” framing; tightened cross-references) and recompiled `paper2_energy_barrier.pdf`.

- **Session 32 (2026-01-02):** completed
  - Paper 2 polish: removed “Derivation sketch” framing from the prime-layer and zero-layer propositions inside `\section{Discharging the budget bound: primes + zeros}` and tightened their proof narratives/citations; recompiled `paper2_energy_barrier.pdf`.

- **Session 33 (2026-01-02):** completed
  - Paper 3 polish: removed the last “Proof sketch” marker by rewriting the proof of Proposition~`prop:barrier-ineq` inside `\section{Barrier closure and conditional RH}` as a clean cited proof narrative (referencing Paper~II for details); recompiled `paper3_cutoff_conditional.pdf`.

- **Session 34 (2026-01-02):** completed
  - Paper 1 polish: strengthened `\section{Definitions and main objects}` by adding a short remark clarifying the role of the outer normalizer `\mathcal O_{\mathrm{can}}` and by making the nonvanishing of `\dettwo(I-A(s))` on `\Omega` explicit; recompiled `paper1_farfield.pdf`.

- **Session 35 (2026-01-02):** completed
  - Paper 1 polish: clarified in Lemma~`lem:rect-cert` (inside `\section{Hybrid certification: rectangle + Pick + asymptotics}`) that the rectangle certification also checks denominator nonvanishing on the certified cover (so $\Theta$ is holomorphic on the rectangle); recompiled `paper1_farfield.pdf`.

- **Session 36 (2026-01-02):** completed
  - Paper 1 polish: made Lemma~`lem:rect-cert` (inside `\section{Hybrid certification: rectangle + Pick + asymptotics}`) more audit-friendly by quoting the explicit certified lower bounds for the denominators from the shipped rectangle artifact (\texttt{min\_abs\_zeta\_lower}, \texttt{min\_abs\_O\_lower}); recompiled `paper1_farfield.pdf`.

- **Session 37 (2026-01-02):** completed
  - Paper 1 polish: strengthened `\section{Hybrid certification: rectangle + Pick + asymptotics}` by adding a new Pick certificate at $\sigma_0=0.6$ (artifact `pick_sigma06_raw_zeta_N16.json`, $\delta_{\rm cert}\approx 0.595$) and using it in Proposition~`prop:hybrid-schur` to certify the Schur bound on the full open half-plane $\{\Re s>0.6\}$ without relying on the $\sigma_0=0.7$ half-plane step; updated Table~`tab:artifact-data` and Remark~`rem:artifact-repro`; recompiled `paper1_farfield.pdf`.

- **Session 38 (2026-01-02):** completed
  - Paper 1 polish: rewrote `\section{Introduction}` (and minimally updated the abstract) to reflect the now-primary audit path: a Pick-matrix certificate at $\sigma_0=0.6$ (strict gap) certifying the Schur bound on $\{\Re s>0.6\}$, with the rectangle and $\sigma_0=0.7$ Pick artifacts presented as additional audit checkpoints; removed misleading “three-part tail covering” language from the front matter; recompiled `paper1_farfield.pdf`.

- **Session 39 (2026-01-02):** completed
  - Paper 1 polish: tightened `\section{Schur/Herglotz pinch mechanism}` by removing the vague “present normalization” phrase in Corollary~`cor:no-poles` and replacing it with an explicit Maximum Modulus Principle dichotomy (`\Theta(s_0)=1` forces `\Theta\equiv 1`) together with a concrete exclusion for the far-half-plane application (using Remark~`rem:Ocan-role`, $\Theta(\sigma+it)\to 1/3$ as $\Re s\to+\infty$); recompiled `paper1_farfield.pdf`.

- **Session 40 (2026-01-02):** completed
  - Paper 1 polish: updated the Appendix `\section{Audit manifest (verifier commands and expected fields)}` to include the new $\sigma_0=0.6$ Pick artifact (`pick_sigma06_raw_zeta_N16.json`) in the fast-audit checklist and added the corresponding regeneration command; clarified that there are now three primary artifacts (rectangle + two Pick certificates); recompiled `paper1_farfield.pdf`.

- **Session 41 (2026-01-02):** completed
  - Paper 1 polish: updated `\section*{Statements and Declarations}` so the “Data and materials availability” list includes the new Pick artifact `pick_sigma06_raw_zeta_N16.json` alongside the rectangle and $\sigma_0=0.7$ Pick artifacts; recompiled `paper1_farfield.pdf`.

- **Session 42 (2026-01-02):** completed
  - Paper 1 polish: updated `\section*{Conclusion and limitations (unconditional status)}` to reflect the current primary audit path (strict Pick gap at $\sigma_0=0.6$, Proposition~`prop:pick-gap06`) and to describe the rectangle and $\sigma_0=0.7$ Pick artifacts as additional audit checkpoints; recompiled `paper1_farfield.pdf`.

- **Session 6 (2026-01-02):** completed
  - Wrote: Paper 2 — `\section{Introduction}` (and compiled `paper2_energy_barrier.pdf`)
  - Next queued section: Paper 2 — `\section{Setup: windowing, phase conventions, and Carleson energy}`

- **Session 7 (2026-01-02):** completed
  - Wrote: Paper 2 — `\section{Setup: windowing, phase conventions, and Carleson energy}` (and compiled `paper2_energy_barrier.pdf`)
  - Next queued section: Paper 2 — `\section{Phase-cost lower bound (Blaschke trigger)}`

- **Session 8 (2026-01-02):** completed
  - Wrote: Paper 2 — `\section{Phase-cost lower bound (Blaschke trigger)}` (and compiled `paper2_energy_barrier.pdf`)
  - Next queued section: Paper 2 — `\section{Carleson energy upper bound (CR--Green / box budget)}`

- **Session 9 (2026-01-02):** completed
  - Wrote: Paper 2 — `\section{Carleson energy upper bound (CR--Green / box budget)}` (and compiled `paper2_energy_barrier.pdf`)
  - Next queued section: Paper 2 — `\section{Energy barrier inequality and definition of $T_{\rm safe}(\eta)$}`

- **Session 10 (2026-01-02):** completed
  - Wrote: Paper 2 — `\section{Energy barrier inequality and definition of $T_{\rm safe}(\eta)$}` (and compiled `paper2_energy_barrier.pdf`)
  - Next queued section: Paper 2 — `\section{Discharging the budget bound: primes + zeros}`

- **Session 11 (2026-01-02):** completed
  - Wrote: Paper 2 — `\section{Discharging the budget bound: primes + zeros}` (and compiled `paper2_energy_barrier.pdf`)
  - Next queued section: Paper 2 — `\section*{Conclusion and limitations (effective status)}`

- **Session 12 (2026-01-02):** completed
  - Wrote: Paper 2 — `\section*{Conclusion and limitations (effective status)}` (and compiled `paper2_energy_barrier.pdf`)
  - Next queued section: Paper 3 — `\section{Introduction}`

- **Session 13 (2026-01-02):** completed
  - Wrote: Paper 3 — `\section{Introduction}` (and compiled `paper3_cutoff_conditional.pdf`)
  - Next queued section: Paper 3 — `\section{From Nyquist cutoff to a uniform arithmetic blocker}`

- **Session 14 (2026-01-02):** completed
  - Wrote: Paper 3 — `\section{From Nyquist cutoff to a uniform arithmetic blocker}` (and compiled `paper3_cutoff_conditional.pdf`)
  - Next queued section: Paper 3 — `\section{From the arithmetic blocker to a uniform Carleson bound}`

- **Session 15 (2026-01-02):** completed
  - Wrote: Paper 3 — `\section{From the arithmetic blocker to a uniform Carleson bound}` (and compiled `paper3_cutoff_conditional.pdf`)
  - Next queued section: Paper 3 — `\section{Barrier closure and conditional RH}`

- **Session 16 (2026-01-02):** completed
  - Wrote: Paper 3 — `\section{Barrier closure and conditional RH}` (and compiled `paper3_cutoff_conditional.pdf`)
  - Next queued section: Paper 3 — `\section*{Conclusion and limitations (conditional status)}`

- **Session 17 (2026-01-02):** completed
  - Wrote: Paper 3 — `\section*{Conclusion and limitations (conditional status)}` (and compiled `paper3_cutoff_conditional.pdf`)
  - Next queued section: (none; Paper 3 complete)

- **Session 39 (2026-01-02):** completed
  - Paper 3 polish: revised `\section{From Nyquist cutoff to a uniform arithmetic blocker}` to make the cutoff-implied finiteness of the prime sum explicit, separate pointwise vs uniform bounds cleanly, and add a concrete scaling example in Remark~`rem:why-cutoff`; recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — `\section{From the arithmetic blocker to a uniform Carleson bound}`

- **Session 40 (2026-01-02):** completed
  - Paper 2 polish: rewrote `\section{Setup: windowing, phase conventions, and Carleson energy}` to clarify near-field scale/intervals (`I_{\gamma,L}` vs. the support interval `I^\ast`), streamline the window definitions, and make the phase convention/Carleson-box normalization explicit; recompiled `paper2_energy_barrier.pdf`.
  - Next queued section: Paper 2 — `\section{Phase-cost lower bound (Blaschke trigger)}`

- **Session 41 (2026-01-02):** completed
  - Paper 2 polish: revised `\section{Phase-cost lower bound (Blaschke trigger)}` to remove ambiguity in the lemma hypothesis (“contains/dominates” $\leadsto$ an explicit distributional inequality), and to streamline the proof to a clean windowed lower bound computation; recompiled `paper2_energy_barrier.pdf`.
  - Next queued section: Paper 2 — `\section*{Conclusion and limitations (effective status)}`

- **Session 42 (2026-01-02):** completed
  - Paper 3 polish: revised `\section{From the arithmetic blocker to a uniform Carleson bound}` to (i) make explicit that Corollary~`cor:nyquist-carleson` bounds the cutoff prime-frequency field, (ii) state the identity $W_{L,t_0}(0,t_0)=S_{L,t_0}$ under the adopted Fourier convention, and (iii) align Remark~`rem:carleson-gap` with the Paper~II prime/zero-layer decomposition while pointing to the admissibility caveat; recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — `\section{Barrier closure and conditional RH}`

- **Session 42 (2026-01-02):** completed
  - Paper 2 polish: revised `\section*{Conclusion and limitations (effective status)}` to sharpen claim hygiene (explicit “effective/height-limited” scope) and to make the cost-vs-budget mechanism and the height-growth obstruction statement more explicit; recompiled `paper2_energy_barrier.pdf`.
  - Next queued section: Paper 2 — (none; Paper 2 remains complete; only minor typos/formatting remain)

- **Session 43 (2026-01-02):** completed
  - Paper 2 polish: revised `\section{Carleson energy upper bound (CR--Green / box budget)}` to make the distributional interpretation of the boundary phase pairing explicit, clarify the support interval role, and tighten the “neutralization” hypothesis wording; recompiled `paper2_energy_barrier.pdf`.
  - Next queued section: Paper 2 — `\section{Energy barrier inequality and definition of $T_{\rm safe}(\eta)$}` (final claim-hygiene/notation pass)

- **Session 44 (2026-01-02):** completed
  - Paper 3 polish: revised `\section{Barrier closure and conditional RH}` to (i) make explicit that the closure step invokes the $M_\Phi<\infty$ hypothesis from Corollary~`cor:nyquist-carleson`, and (ii) clarify the conditional RH implication via zeros of the completed zeta function $\xi$ (avoiding any ambiguity about trivial zeros); recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — `\section*{Conclusion and limitations (conditional status)}` (final claim-hygiene/wording pass)

- **Session 45 (2026-01-03):** completed
  - Paper 3 polish: revised `\section*{Conclusion and limitations (conditional status)}` to align claim hygiene with the closure section by explicitly listing the additional uniformity/admissibility assumptions used in the barrier closure (notably the $M_\Phi<\infty$ hypothesis required by Corollary~`cor:nyquist-carleson`); recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — (none; Paper 3 remains complete; only minor typos/formatting remain)

- **Session 46 (2026-01-03):** completed
  - Paper 3 polish: revised `\section*{Statements and Declarations}` to (i) add an explicit “all files needed to compile” sentence, and (ii) tighten the reproducibility statement to point precisely to Paper~I / Paper~II and to the conditional hypotheses used in the closure step; recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — (none; Paper 3 remains complete; only minor typos/formatting remain)

- **Session 47 (2026-01-03):** completed
  - Paper 3 polish: revised `\section{Introduction}` to align claim hygiene between the headline statements and the closure: Theorem~`thm:nyquist-nearfield` and Corollary~`cor:conditional-rh` now explicitly assume the uniformity/admissibility conditions invoked in Section~`sec:closure` (in particular the uniform bound $M_\Phi<\infty$ from Lemma~`lem:nyquist-implies-blocker`); recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — (none; Paper 3 remains complete; only minor typos/formatting remain)

- **Session 48 (2026-01-03):** completed
  - Paper 3 polish: revised `\section{From Nyquist cutoff to a uniform arithmetic blocker}` to (i) make the “uniform in $(L,t_0)$” quantifiers explicit by tying the $M_\Phi$ supremum to the parameter range of Hypothesis~`hyp:nyquist-cutoff`, and (ii) soften “only” wording to conservative “key input” language (with an explicit pointer to the admissibility caveat); recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — (none; Paper 3 remains complete; only minor typos/formatting remain)

- **Session 48 (2026-01-03):** completed
  - Paper 1 polish: revised `\section{Hybrid certification: rectangle + Pick + asymptotics}` to match the current primary audit path (strict Pick gap at $\sigma_0=0.6$), clarified the rectangle and $\sigma_0=0.7$ Pick artifacts as auxiliary audit checkpoints, simplified the “domain decomposition” prose accordingly, and added a short remark explaining the artifact gauge/normalization choices (\texttt{outer\_zeta\_proj} vs \texttt{raw\_zeta}); recompiled `paper1_farfield.pdf`.
  - Next queued section: Paper 1 — (none; Paper 1 remains complete; only minor typos/formatting remain)

- **Session 49 (2026-01-03):** completed
  - Paper 1 polish: fixed a normalization ambiguity inside `\section{Hybrid certification: rectangle + Pick + asymptotics}` by renaming the rectangle artifact’s normalizer from $\mathcal O_{\mathrm{can}}$ to $\mathcal O_{\mathrm{proj}}$ (matching the \texttt{outer\_zeta\_proj} gauge) and tightening Remark~`rem:gauges` to state pole-set invariance only on regions where the normalizing factor is certified nonvanishing; recompiled `paper1_farfield.pdf`.
  - Next queued section: Paper 1 — (none; Paper 1 remains complete; only minor typos/formatting remain)

- **Session 49 (2026-01-03):** completed
  - Paper 3 polish: revised `\section{From the arithmetic blocker to a uniform Carleson bound}` to (i) clarify that the corollary’s $M_\Phi$ supremum ranges over the same $(L,t_0)$ parameter set as Hypothesis~`hyp:nyquist-cutoff`, and (ii) tighten wording to avoid any accidental over-identification of the cutoff prime-field estimate with the full zeta-ratio budget; recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — `\section{Barrier closure and conditional RH}`

- **Session 50 (2026-01-03):** completed
  - Paper 3 polish: revised `\section{Barrier closure and conditional RH}` to (i) further separate the cutoff prime-field Carleson estimate from the full zeta-ratio Carleson budget language (stating it explicitly as the prime-frequency input), and (ii) make the admissibility assumption in Remark~`rem:blockers` explicit in the conditional proofs; recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — (none; Paper 3 remains complete; only minor typos/formatting remain)

- **Session 51 (2026-01-03):** completed
  - Paper 3 polish: revised `\section{Introduction}` to state the headline theorem/corollary assumptions directly (Hypothesis~`hyp:nyquist-cutoff` + admissibility from Remark~`rem:blockers` + $M_\Phi<\infty$) and to soften “removes height-growth” wording to a conservative conditional formulation; recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — (none; Paper 3 remains complete; only minor typos/formatting remain)

- **Session 52 (2026-01-03):** completed
  - Paper 3 polish: revised `\section{Introduction}` to make the Nyquist-cutoff hypothesis fully explicit about the $(L,t_0)$ parameter range and regularity (treating $\Phi_{L,t_0}\in L^1(\R)$ so the Fourier transform is well-defined, and stating that the paper only invokes the hypothesis for the near-field range $0<L\le 0.2$); recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — (none; Paper 3 remains complete; only minor typos/formatting remain)

- **Session 51 (2026-01-03):** completed
  - Paper 3 polish: revised `\section{Barrier closure and conditional RH}` to (i) explicitly flag the distributional interpretation of the windowed phase pairing in Proposition~`prop:barrier-ineq`, (ii) define the neutralized holomorphic/zero-free function $F$ used in the CR--Green summary line, and (iii) clarify the RH implication via zeros of the completed zeta function $\xi$ (stating that these coincide with the nontrivial zeros of $\zeta$); recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — (none; Paper 3 remains complete; only minor typos/formatting remain)

- **Session 52 (2026-01-03):** completed
  - Paper 3 polish: revised `\section{From Nyquist cutoff to a uniform arithmetic blocker}` to make the $(L,t_0)$ quantifier regime fully explicit at the point where $M_\Phi$ is defined (recording the near-field closure range $0<L\le 0.2$ and $t_0\in\R$), eliminating any ambiguity about what “uniform in $(L,t_0)$” means; recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — (none; Paper 3 remains complete; only minor typos/formatting remain)

- **Session 53 (2026-01-03):** completed
  - Paper 3 polish: revised `\section{From Nyquist cutoff to a uniform arithmetic blocker}` to fix a minor grammatical issue in Lemma~`lem:nyquist-implies-blocker` (removing the stray leading “and” before the $M_\Phi<\infty$ assumption), keeping the $(L,t_0)$ quantifier regime explicit; recompiled `paper3_cutoff_conditional.pdf`.
  - Next queued section: Paper 3 — (none; Paper 3 remains complete; only minor typos/formatting remain)

- **Session 54 (2026-01-03):** completed
  - Paper 1 polish: revised `\section{Definitions and main objects}` to make the artifact/verification gauges consistent with the manuscript objects by (i) defining the arithmetic ratio \(\mathcal J\) and Cayley field \(\Theta\) in the raw \(\zeta\)-gauge \(\mathcal O\equiv 1\) by default, (ii) allowing auxiliary holomorphic nonvanishing normalizers on compact regions (with gauge declared per artifact), and (iii) updating \eqref{eq:J-def} and Remarks~`rem:Ocan-role`, `rem:poles` accordingly; recompiled `paper1_farfield.pdf`.
  - Next queued section: Paper 1 — (none; Paper 1 remains complete; only minor typos/formatting remain)

- **Session 55 (2026-01-03):** completed
  - Paper 1 polish: revised `\section{Schur/Herglotz pinch mechanism}` by strengthening Corollary~`cor:no-poles` so it no longer assumes (implicitly) that \(\Theta\) has no Cayley-singularities: it now starts from \(\Theta\) meromorphic on \(U\) and \(|\Theta|\le 1\) away from poles, deduces holomorphic extension across all poles by removability, and then applies the Maximum Modulus Principle to exclude \(\Theta=1\) (hence deduce holomorphicity of the Cayley inverse and absence of poles of \(\mathcal J\)); recompiled `paper1_farfield.pdf`.
  - Next queued section: Paper 1 — (none; Paper 1 remains complete; only minor typos/formatting remain)

- **Session 56 (2026-01-03):** completed
  - Paper 1 polish: replaced the non-explicit large-$|t|$ tail lemma in `\section{Hybrid certification: rectangle + Pick + asymptotics}` with an explicit certified Pick-matrix half-plane certificate at $\sigma_0=0.599$ (new artifact `pick_sigma0599_raw_zeta_N16.json`), so the claimed line $\Re s=0.6$ is an interior subset of the certified Schur domain; removed Lemma~`lem:tail-bound`; updated Table~`tab:artifact-data`, Remark~`rem:artifact-repro`, the audit manifest Appendix, and `README.md` accordingly; recompiled `paper1_farfield.pdf`.
  - Next queued section: Paper 1 — (none; Paper 1 remains complete; only minor typos/formatting remain)


