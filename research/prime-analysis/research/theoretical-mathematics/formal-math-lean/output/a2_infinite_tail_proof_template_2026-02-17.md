# A2 Infinite-Tail Proof Template (O2)

Generated: 2026-02-17
Companion skeleton: `research/output/a2_infinite_tail_lemma_skeleton_2026-02-17.md`

## Target Lemma (theorem form)
For wheel family `W={30,210,2310,30030}`, there exist explicit constants
`x0>=3`, `M0`, `beta>=0`, `C_delta>=0`, and a monotone function `tau_infty(M)` with
`tau_infty(M) -> 0` as `M->infty`, such that for all `x>=x0`, `M>=M0`, `W in W`:

`Delta_M(x;W) <= C_delta * (log x)^beta * tau_infty(M)`.

Where `Delta_M(x;W) = |H_W^{(M)}(x)-H_W^{(M_ref)}(x)|` (or infinite-reference analog).

## Proof Structure

### Step 1: Representation-level decomposition
- Write `Delta_M` as contribution of omitted zeros `gamma_j > gamma_M` through channel map.
- Isolate kernel-weighted tail terms of type
  `sum_{j>M} K(gamma_j) * A(gamma_j, x)`.

Deliverable:
- exact algebraic decomposition lemma (`A(gamma_j,x)` explicit and uniformly bounded in `x`).

### Step 2: Uniform per-zero majorant
- Prove a pointwise bound
  `|A(gamma,x)| <= C_A * (log x)^beta * gamma/sqrt(1/4+gamma^2)`
  (or sharper variant).
- This defines the summand prototype for `tau_infty(M)`.

Deliverable:
- explicit `(C_A,beta)` with no data-fit constants.

### Step 3: Sum-to-density tail replacement
- For omitted zeros use rigorous bound
  `sum_{gamma>G} f(gamma) <= Integral_G^inf f(t) dN(t)`.
- Replace with explicit upper envelope on `N'(t)` or Stieltjes form using known zero-density/count estimates.

Deliverable:
- fully explicit inequality
  `tau_infty(M) <= T_majorant(gamma_M)`.

### Step 4: Monotonicity and limit
- Show `tau_infty(M)` nonincreasing in `M`.
- Show `tau_infty(M) -> 0` from integrability of weighted tail.

Deliverable:
- short monotonicity+convergence lemma.

### Step 5: Assemble global constant
- Combine Steps 1-4 to produce theorem constants `C_delta,beta,x0,M0`.
- Verify resulting theorem statement is independent of sampled grid.

Deliverable:
- final A2 theorem lemma statement inserted into proof skeleton.

## Frozen Calibration Anchors (for sanity checks only)
- `beta_fixed = 2.6`
- `C_delta_uplifted = 0.0013231345976502436`
- gaussian kernel scale `100`
- `m_ref=512`, `gamma_ref=826.905810954`
- finite-window valid ratio max: `0.947859440603`

These are not proof premises; they are post-proof consistency checks.

## Immediate Next Subtask
- Implement a symbolic/numeric checker script that, given a candidate analytic `N_up(t)`, recomputes
  `tau_infty(M)` and emits whether it upper-bounds the finite tail table for all `M in {64,128,192,256}`.
- This will let us iterate rigorous `N_up(t)` candidates while reusing existing cached zero data.

