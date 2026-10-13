# K1 W60 Constructive-Gate Lemma Chain (2026-02-24)

## Goal
Translate the current strongest finite-window path into a theorem-facing sequence for the remaining T2 blocker.

Working normalized function:
\[
Y_\beta(x)=\frac{\psi(x)-x}{x^\beta},\quad \beta=0.62.
\]

Principal two-mode representation:
\[
Y_\beta(x)=A_1\cos(\tau_1\log x+\phi_1)+A_2\cos(\tau_2\log x+\phi_2)+R_2(x),
\]
with \(\tau_1=14.134725142,\ \tau_2=21.022039639\).

---

## Constructive-gate lemmas (target form)

### Lemma C1 (Tau1 anchor recurrence)
There exist infinitely many integer anchors \(n_k\to\infty\) such that
\[
|\cos(\tau_1\log n_k+\phi_1)|\ge a_1
\]
for fixed \(a_1\in(0,1)\).

### Lemma C2 (Nonnegative tau2 gate on anchor neighborhoods)
For each large anchor index \(k\), there exists \(m_k\) in a bounded neighborhood of the tau1 anchor point such that
\[
|\cos(\tau_1\log m_k+\phi_1)|\ge a_1,\qquad
\cos(\tau_2\log m_k+\phi_2)\ge 0.
\]
This yields a constructive subsequence \(m_k\to\infty\).

### Lemma C3 (One-sided reduction)
On this constructive subsequence:
\[
Y_\beta(m_k)\ge A_1 a_1 + R_2(m_k),
\]
since \(A_2\cos(\tau_2\log m_k+\phi_2)\ge0\).
Hence lower-envelope control reduces to an upper bound on the negative part of \(R_2\):
\[
R_2^-(x):=\max(-R_2(x),0).
\]

### Lemma C4 (M4 one-sided tail split)
Write
\[
R_2 = T_4 + E_4,
\]
where \(T_4\) is the finite-mode tail block (modes 3..4 relative to the two-mode base) and \(E_4\) is the remainder after 4-mode truncation.
Then
\[
R_2^- \le T_4^- + E_4^-,
\]
so it suffices to prove
\[
\frac{T_4^-}{A_1}\le q_T,\qquad
\frac{E_4^-}{A_1}\le q_E
\]
cofinally on the constructive subsequence with
\[
q_T+q_E<a_1.
\]

### Lemma C5 (Admissibility closure)
If C1-C4 hold, then for \(q=q_T+q_E\):
\[
Y_\beta(m_k)\ge A_1(a_1-q)
\]
cofinally, yielding a positive lower-envelope constant for contradiction integration.

---

## Finite-window guidance for constants (not proof)

From W60 (five windows \(10^7\) to \(5\cdot10^7\), \(M=4\), search window \(W=100\)):
- For \(a_1=0.98\):
  - \(q_{\text{split,neg,max}} \approx 0.430301\)
  - \(\delta_{\text{split,neg,min}}=a_1-q_{\text{split,neg,max}}\approx0.549699\)
- Robust under \(a_1\in\{0.95,0.97,0.98\}\):
  - \(q_{\text{split,neg,max}}\) unchanged in current sweep
  - \(\delta_{\text{split,neg,min}}\approx 0.519699,\ 0.539699,\ 0.549699\).

Artifacts:
- `research/output/k1_w60_tau14_tau21_tail_split_a095_five_window_summary_M4.md`
- `research/output/k1_w60_tau14_tau21_tail_split_a097_five_window_summary_M4.md`
- `research/output/k1_w60_tau14_tau21_tail_split_a098_five_window_summary_M4.md`

---

## Remaining genuine math obligations
1. Prove C1/C2 (constructive existence) without finite-window dependence.
2. Prove C4 one-sided component bounds (\(q_T,q_E\)) symbolically on the constructive subsequence.
3. Integrate C5 into the existing contradiction endpoint chain.

This is now the precise non-scaffold path to close T2.
