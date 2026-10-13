# K1 W50 Aligned Mode Admissibility Lemma Skeleton (2026-02-24)

## Purpose
Define the exact theorem statement needed to close the current math gate, with assumptions mapped to existing sources vs genuinely new work.

---

## Lemma L-AMA (Aligned Mode Admissibility)

Let \(E_\*(x)=\psi(x)-x\). Fix \(\beta>1/2\), and define normalized signal
\[
Y_\beta(x):=\frac{E_\*(x)}{x^\beta}.
\]

Assume there exists a mode \(\tau>0\), amplitude \(A>0\), phase \(\phi\), and remainder \(R\) such that
\[
Y_\beta(x)=A\cos(\tau\log x+\phi)+R(x)\quad (x\ge X_0).
\]

Fix \(a_0\in(0,1)\). Suppose there exists \(q<a_0\) such that:
\[
\forall X\ge X_1\ \exists x\ge X:
\quad |\cos(\tau\log x+\phi)|\ge a_0
\quad\text{and}\quad
\frac{|R(x)|}{A}\le q.
\]

Then
\[
\forall X\ge X_1\ \exists x\ge X:\ |E_\*(x)|\ge (a_0-q)A\,x^\beta.
\]

### Proof sketch
At a witness \(x\):
\[
\frac{|E_\*(x)|}{x^\beta}=|A\cos(\tau\log x+\phi)+R(x)|
\ge A|\cos(\cdot)|-|R(x)|
\ge A(a_0-q).
\]
Hence the claimed lower envelope with \(c=(a_0-q)A>0\).

---

## Source-Mapped Assumption Ledger

### AMA-1: Fixed-function explicit-formula decomposition
Need:
\[
Y_\beta(x)=A\cos(\tau\log x+\phi)+R(x)
\]
for canonical \(E_\*\), with one selected \(\tau\) (from zero spectrum).

Status:
- **Partially sourced / partially new**.
- Explicit-formula framework and zero-driven oscillatory terms are standard; project has numerical extraction support.
- Remaining theorem-grade extraction for chosen mode in this exact normalized form is still open in-repo.

### AMA-2: Cofinal aligned-point existence
Need:
\[
\forall X\ \exists x\ge X:\ |\cos(\tau\log x+\phi)|\ge a_0.
\]

Status:
- **Classical / straightforward** (phase recurrence on log scale).
- Should be formally easy once \(\tau,\phi\) are fixed.

### AMA-3: Cofinal aligned remainder-ratio bound
Need:
\[
\forall X\ \exists x\ge X:\ \frac{|R(x)|}{A}\le q,\quad q<a_0.
\]

Status:
- **Genuinely open hard gate**.
- W49 shows global explicit envelopes are too weak by large factors in this normalization.
- Must be proved via mode-specific aligned tail cancellation/remainder theory (not generic global \(\Delta(x)\) bounds).

---

## Conservative Candidate Constants (Current empirical guidance)
From W48 robust admissible set (`beta=0.62`, `a0=0.98`), best conservative candidate:
- `tau = 14.134725142`
- `A_min ≈ 2.920877e-02`
- `rr_max ≈ 0.439166`
- `delta = a0 - rr_max ≈ 0.540834`
- `c_cons = A_min * delta ≈ 1.631385e-02`

These are finite-window research constants, not theorem constants.

---

## Exact new theorem tasks (no renaming, no remapping)

1. **T1 (Mode extraction theorem)**:
Prove normalized single-mode-plus-remainder decomposition for fixed \(E_\*\) and one admissible \(\tau\).

2. **T2 (Aligned remainder theorem)**:
Prove existence of \(q<a_0\) with cofinal aligned-point ratio bound \(|R|/A\le q\).

3. **T3 (Contradiction integration)**:
Combine L-AMA output with endpoint upper bound theorem to complete the contradiction route.

T2 is the principal unsolved mathematical blocker.

