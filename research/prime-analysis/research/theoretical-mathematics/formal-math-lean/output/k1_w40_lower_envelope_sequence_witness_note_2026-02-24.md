# K1 W40 Lower-Envelope Sequence Witness Note (2026-02-24)

## Goal of this step
Advance the fixed-function contradiction route on
\[
E_\*(x)=\psi(x)-x
\]
by measuring constants that match the quantifier shape
\[
\forall X\ \exists x\ge X:\ |E_\*(x)|\ge c_\beta x^\beta
\]
on phase-aligned subsequences.

## New tool
- `/Users/adminamn/Documents/New project/research/fixed_error_psi_lower_envelope_ledger.py`

Key outputs:
- `/Users/adminamn/Documents/New project/research/output/k1_w40_fixed_error_psi_lower_envelope_ledger_2026-02-24_x1e7.json`
- `/Users/adminamn/Documents/New project/research/output/k1_w40_fixed_error_psi_lower_envelope_ledger_2026-02-24_x1e7.md`
- `/Users/adminamn/Documents/New project/research/output/k1_w40_fixed_error_psi_lower_envelope_ledger_2026-02-24_x3e7.json`
- `/Users/adminamn/Documents/New project/research/output/k1_w40_fixed_error_psi_lower_envelope_ledger_2026-02-24_x3e7.md`

## Core findings

### 1) Dominant log-phase mode is stable
Across tested `beta in {0.58,0.60,0.62}`, best tau is consistently:
- `tau_best = 14.134725142` (first zeta zero frequency)

This supports the intended main-mode extraction direction.

### 2) Uniform remainder domination is not yet achieved
Tail ratio
\[
\rho_{\sup} := \sup_{x\in\text{tail}} \frac{|R(x)|}{A}
\]
is large:
- `rho_sup ~ 3.69 .. 4.06` (depending on beta/window)

So the strong uniform condition
\[
|R(x)| \le \tfrac14 A x^\beta \quad (\text{all large }x)
\]
is not supported by current finite-window data.

### 3) Sequence-style witness constants are positive
Even though uniform domination fails, aligned subsequences show positive witness constants.

For `x <= 3e7`:
- `beta=0.58`: `w_tri_grid ~ 3.248e-02`
- `beta=0.60`: `w_tri_grid ~ 2.302e-02`
- `beta=0.62`: `w_tri_grid ~ 1.631e-02`

Here `w_tri_grid` is the finite-grid witness constant derived from
\[
A|\cos(\tau\log x+\phi)|-|R(x)|
\]
on phase-aligned tail points and threshold grid \(X\)-values.

`witness_triangle_all_positive = true` for all three betas in both windows, meaning every tested threshold had at least one later aligned point with positive triangle lower value.

### 4) Contradiction scales from witness constants
Using empirical endpoint envelope
\[
|E_\*(x)|\le C_{\mathrm{endpoint}}x^{1/2}(\log x)^2,
\]
the witness-triangle crossover scales are finite:
- `beta=0.58`: `x_cross(w_tri) ~ 5.58e41`
- `beta=0.60`: `x_cross(w_tri) ~ 6.27e32`
- `beta=0.62`: `x_cross(w_tri) ~ 1.70e27`

These are very large (expected), but they are finite and in the correct direction for a contradiction route.

## Interpretation
This step sharpens the math picture:

1. Main frequency extraction is stable (good).
2. Global uniform remainder domination is too strong for current evidence (expected frontier).
3. A weaker but relevant path is supported: **subsequence-based lower envelope** with positive witness constants on aligned phase points.

## Next theorem-facing math target (W41)
Derive a theorem in this shape for fixed canonical \(E_\*\):

\[
\exists \beta>\tfrac12,\ \exists c>0,\ \forall X\ \exists x\ge X:\ |E_\*(x)|\ge c\,x^\beta,
\]

using:
- explicit-formula main-mode extraction at a right-half zero,
- a subsequence remainder lemma (not full uniform \(A/4\) bound),
- phase recurrence on \(x_n=\exp((2\pi n-\phi)/\tau)\) and a non-circular control that gives infinitely many successful \(n\).

This is the mathematically relevant gate before endpoint contradiction formalization.

