# Plan: Split `Riemann-RS.tex` into Three Papers

This plan describes how to split the current monolithic manuscript `Riemann-RS.tex` into three referee-friendly papers with **clean claim boundaries**:

- **Paper 1 (Unconditional)**: Certified zero-free region for \(\zeta(s)\) in the half-plane \(\Re s \ge 0.6\).
- **Paper 2 (Effective / analytic)**: Energy barriers and Carleson budgets for off-critical zeros of \(\zeta(s)\).
- **Paper 3 (Conditional / hypothesis-driven)**: A cutoff principle and conditional closure of RH.

The goal is to **reduce referee load**, eliminate “mixed-status” confusion (unconditional vs effective vs conditional), and make each paper publishable on its own merits.

---

## Goals and constraints

- **Goal**: Produce **three standalone LaTeX papers**, each compilable to its own PDF, each with a crisp headline theorem and an honest status (“unconditional”, “effective”, or “conditional”).
- **Goal**: Keep **Paper 1** maximally classical and self-contained; it should stand even if a reader dislikes RS or near-field heuristics.
- **Goal**: Keep a **single source of truth** for technical macros/notation to avoid drift.
- **Constraint**: Preserve the current repository audit workflow (interval arithmetic artifacts + Pick matrix artifact + verifier).
- **Constraint**: Keep authorship consistent with the current decision (**Jonathan Washburn only**).

---

## High-level structure of the split

### Paper 1: Far-field certification (unconditional)

- **Working title**: *A certified zero-free region for the Riemann zeta function in the half-plane \(\Re s \ge 0.6\)*.
- **Claim type**: **Unconditional theorem** (no RS assumptions, no “effective height” qualifiers).
- **Main ingredients**:
  - Arithmetic Cayley field \(\Theta\), Schur property, pinch/globalization across \(Z(\xi)\).
  - Hybrid certification:
    - Rectangle interval-arithmetic certification on \([0.6,0.7]\times[0,20]\) (artifact).
    - Pick certificate at \(\sigma_0=0.7\), \(N=16\), spectral gap \(\delta_{\rm cert}\) (artifact).
    - Asymptotic bounds for \(|t|>20\) (analytic).
  - Theorems/labels to keep (minimum set):
    - `prop:farfield-hybrid`, `thm:pick-global-positivity`, `thm:pick-criterion`,
      `def:arith-taylor`, `def:arith-pick-matrix`, `lem:pick-matrix-coeff-formula`,
      `lem:theta-asymptotic`, plus the Schur pinch/globalization theorem(s) used to
      eliminate zeros once \(|\Theta|\le 1\) is certified.
  - Keep the artifact table + reproducibility pointer:
    - Table `tab:artifact-data`, Remark `rem:artifact-repro`, and the repo `README.md`.

### Paper 2: Near-field barrier + Carleson budgets (effective)

- **Working title**: *Energy barriers and Carleson budgets for off-critical zeros of the Riemann zeta function*.
- **Claim type**: **Effective / height-limited** (explicit \(T_{\rm safe}(\eta)\)).
- **Main ingredients**:
  - Windowed phase lower bound (“Blaschke trigger”) vs. CR–Green energy upper bound.
  - Scale-tracked prime-layer energy bound (unconditional ingredient).
  - Full Carleson bound including the height-dependent term (the source of the “gap”).
  - Protection height definition and example magnitudes.
  - Theorems/labels to keep (minimum set):
    - `lem:energy-barrier`, `thm:scale-tracked-final`, `thm:full-carleson`,
      `cor:unconditional-closure`, plus the chain of lemmas that feed these.
  - **Optional**: A short “relation to Paper 1” sentence:
    - Paper 2 can stand alone as a barrier paper; if desired, add: “Combined with the
      certified far-field zero-freeness of Paper 1, this yields two-regime elimination up
      to \(T_{\rm safe}(\eta)\).”

### Paper 3: Cutoff principle and conditional closure (conditional)

- **Working title**: *A cutoff principle and conditional closure of the Riemann Hypothesis*.
- **Claim type**: **Conditional** (explicit hypothesis; explicit blockers).
- **Main ingredients**:
  - The Nyquist cutoff / bandlimit hypothesis (currently `hyp:nyquist-cutoff`).
  - Lemma that bandlimit \(\Rightarrow\) uniform arithmetic blocker.
  - Conditional corollaries: uniform Carleson bound and conditional RH closure.
  - Explicit “bridge gap” language and “this may be RH-strength classically” framing.
  - Theorems/labels to keep (minimum set):
    - `hyp:nyquist-cutoff`, `lem:nyquist-implies-blocker`, `cor:T7-Carleson`, `cor:T7-RH`,
      plus the “BLOCKER 1/2” discussion and any RS motivation you want to retain as
      *motivation* (not proof).

---

## File layout proposal (minimizes duplication)

### 1) Freeze the current monolith

- Keep `Riemann-RS.tex` as the “collected version” (for internal reference).
- Optionally copy it to `Riemann-RS_COLLECTED.tex` so the split work doesn’t destroy it.

### 2) Create a shared preamble file

Create:

- `riemann_common_preamble.tex`

Contents:

- packages, theorem environments, macros, numeric-lock toggles, constant macros, etc.
- **No** `\documentclass`, **no** `\begin{document}`.

Each paper then does:

```tex
\documentclass[11pt]{article}
\input{riemann_common_preamble.tex}
% title/author/date here
\begin{document}
...
\end{document}
```

### 3) Create a shared bibliography include (optional)

Since `Riemann-RS.tex` uses `\begin{thebibliography}{99}`, we have two options:

- **Option A (fast)**: copy the full bibliography into each paper.
- **Option B (cleaner)**: create `riemann_bibliography.tex` containing the `thebibliography` environment and `\input{riemann_bibliography.tex}` from each paper.

Start with **Option B** (avoid duplication), prune later if desired.

---

## Content extraction plan (how to cut the monolith)

### Step 0: Identify “cut points”

Use existing major headings already present in `Riemann-RS.tex`:

- Far-field certification content clusters around:
  - Pick matrix section(s), far-field hybrid proposition, artifact table, pick audit appendix.
- Near-field barrier content clusters around:
  - `lem:energy-barrier`, Carleson energy lemmas, `thm:scale-tracked-final`, `thm:full-carleson`,
    and the closure section `sec:unconditional-closure`.
- Conditional cutoff content clusters around:
  - `hyp:nyquist-cutoff` and conditional corollaries; “BLOCKER 1/2”; RS motivation sections.

### Step 1: Build Paper 1 by inclusion

Start Paper 1 as “only far-field”:

- Copy in:
  - Definitions of \(\mathcal J\), \(\Theta\), and the Schur-pinch framework needed for the far-half-plane.
  - Pick matrix definitions and the finite-gap-to-infinite argument.
  - Far-field hybrid certification proposition and the artifact table.
  - The appendix `app:pick-audit` (or include as “Supplementary Audit”).

- Remove:
  - Energy barrier lemma, Carleson budget machinery, all RS and conditional material.

### Step 2: Build Paper 2 by inclusion

Start Paper 2 as “barrier + budgets”:

- Copy in:
  - Minimal setup: \(\mathcal J\), phase \(w\), Carleson box definitions, window \(\psi\).
  - The full near-field analytic chain needed for:
    - `lem:energy-barrier`
    - `thm:scale-tracked-final`
    - `thm:full-carleson`
    - effective \(T_{\rm safe}(\eta)\) and its examples

- Remove:
  - Pick matrix / far-field certification details (only cite Paper 1 if needed).
  - Nyquist cutoff hypothesis and conditional closure.

### Step 3: Build Paper 3 by inclusion

Start Paper 3 as “cutoff principle”:

- Copy in:
  - Minimal setup: explicit formula objects needed to state the hypothesis.
  - `hyp:nyquist-cutoff` and all conditional consequences.
  - The explicit disclaimer language: bridge gap, admissibility, RH-strength suspicion.

- Remove:
  - Most of the technical Carleson budget machinery (cite Paper 2 if required).
  - Pick matrix / far-field certification (cite Paper 1 if required).

---

## Cross-paper referencing strategy (avoid LaTeX cross-refs across PDFs)

- Replace internal references like “Theorem~\ref{...}” across papers with:
  - “Paper I, Theorem X” / “Paper II, Lemma Y” / “Paper III, Hypothesis Z”
  - or a BibTeX-style citation once you have arXiv IDs / preprint numbers.

Practical approach:

- Each paper should compile standalone with its own numbering.
- Add a 3-line “Related papers in this series” paragraph in each introduction:
  - Paper I (unconditional far-field)
  - Paper II (effective near-field barrier)
  - Paper III (conditional closure under cutoff)

---

## Reproducibility / artifacts policy after the split

- Keep the current repo-level `README.md` as the audit guide for the unconditional far-field artifacts.
- Paper 1 should keep:
  - Table `tab:artifact-data`
  - Remark `rem:artifact-repro` + pointer to `README.md`.
- Papers 2–3 should **not** mention ARB artifacts unless they introduce new audited data.

---

## Implementation checklist (deliverables)

- **Files (new)**:
  - `riemann_common_preamble.tex`
  - `riemann_bibliography.tex` (optional but recommended)
  - `paper1_farfield.tex`
  - `paper2_energy_barrier.tex`
  - `paper3_cutoff_conditional.tex`

- **Build outputs**:
  - `paper1_farfield.pdf`
  - `paper2_energy_barrier.pdf`
  - `paper3_cutoff_conditional.pdf`

- **Sanity checks**:
  - Each PDF compiles with `pdflatex -interaction=nonstopmode`.
  - No dangling references or missing labels inside each paper.
  - Titles/abstracts match the correct claim status (“unconditional/effective/conditional”).

---

## Risks and mitigations

- **Risk: duplicated setup drifts** (macros/constants differ across papers)  
  - **Mitigation**: shared `riemann_common_preamble.tex`.

- **Risk: cross-paper references break**  
  - **Mitigation**: remove cross-file `\ref{}` usage; replace with “Paper I/II/III”.

- **Risk: Paper 2 depends implicitly on Paper 1** (far-field used in “two-regime elimination”)  
  - **Mitigation**: keep Paper 2 as an “effective barrier” result; if you want the combined statement, cite Paper 1 explicitly.

- **Risk: referee confusion about artifacts**  
  - **Mitigation**: only Paper 1 carries the audit manifest + artifact table; Paper 2 and 3 avoid it.


