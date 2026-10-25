# Lab Record — Secant Surrogate Descent Check (`r-secant-surrogate-descent`, #2886)

- **Roadmap item:** [`r-secant-surrogate-descent`](../../roadmap/items/r-secant-surrogate-descent.md) (#2886)
- **Pre-registration sheet:** [`docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`](../../harness/prep/r-secant-surrogate-descent-2026-10-25.md)
- **Downstream items gated by this check:**
  - [`hamilton-gate-offline-float-sheet`](../../roadmap/items/hamilton-gate-offline-float-sheet.md) (#2824)
  - [`recurrence-gates-six-recurrence-fit`](../../roadmap/items/recurrence-gates-six-recurrence-fit.md) (#2890)
  - [`spiralcore-fbs-offline-float-sheet`](../../roadmap/items/spiralcore-fbs-offline-float-sheet.md) (#2891)
- **Prerequisite module:** [`float-training-module`](../../roadmap/items/float-training-module.md) (#2823, `crates/uor-r4-float-train`)

---

## 1. Exact Secant Surrogate Equations in the Repository

Both mechanisms replace discontinuous hard table/tree selection (whose true derivative is zero almost everywhere) with a declared geometric secant surrogate that contains zero exponentials, zero squashing functions, and zero dense matrix multiplications (`CHARTER.md` §3.1–§3.3):

### 1.1 Recurrence Gates (`RecGates::Geometric`, `rate_select_fwd` / `rate_select_bwd`)
- **Source:** [`crates/uor-r4-training/src/cuda_stack.cu`](../../../crates/uor-r4-training/src/cuda_stack.cu) (lines 231–291) and [`crates/uor-r4-training/src/geometric_stack_native.rs`](../../../crates/uor-r4-training/src/geometric_stack_native.rs) (lines 1469–1555).
- **Forward:** Clamped linear opening $s = \mathrm{clamp}(g + d, 0, 1)$ selects index $i = \min(\lfloor 16 s \rfloor, 15)$ from the 16-entry dyadic fade table $\lambda_i = 2^{-(i+1)}$ (`RATE_TABLE`) and precomputed unit-circle keep table $\kappa_i = \sqrt{1 - \lambda_i^2}$ (`KEEP_TABLE`), driving the quaternion scan $h_t = \lambda_t u_t \otimes h_{t-1} + \kappa_t b_t$.
- **Declared secant backward (`rate_select_bwd`):** Pairs incoming adjoints $(d\lambda, d\kappa)$ with the counterfactual table displacements to the immediate neighbours $j \in \{i-1, i+1\} \cap [0, 15]$ across the fixed bin width $\Delta s = 1/16$:
  $$d s = 16 \cdot \frac{1}{|\mathcal{N}(i)|} \sum_{j \in \{i-1, i+1\} \cap [0, 15]} \mathrm{sgn}(j - i)\Big(d\lambda\cdot(\lambda_j - \lambda_i) + d\kappa\cdot(\kappa_j - \kappa_i)\Big).$$

### 1.2 SpiralCore/FBS Transport (`QuaternionWeights::SpiralcoreFbs`, `spiralcore_fbs_fwd` / `spiralcore_fbs_bwd`)
- **Source:** [`crates/uor-r4-training/src/spiralcore_fbs.cu`](../../../crates/uor-r4-training/src/spiralcore_fbs.cu) (lines 124–183) and [`crates/uor-r4-training/src/spiralcore_fbs.rs`](../../../crates/uor-r4-training/src/spiralcore_fbs.rs).
- **Forward:** For each output block $o$ and binder stage $m \in \{0, \dots, M-1\}$, query $q_{o,m} \in \mathbb{R}^8$ picks the nearest root $r_{\text{goal}(m)}$ of the 16-root $E_8$ anchor orbit (`frontier_count` choices at stage $0$, $3$ ternary sibling branches at stages $m > 0$), composing a 64-state $\mathrm{Cl}(0,6)$ signed-axis action $A_o$ applied to leaf atom $x_{\sigma(o)} \in \mathbb{R}^8$: $y_o = A_o x_{\sigma(o)}$.
- **Declared secant backward (`spiralcore_fbs_bwd`):** Scatters the exact inverse action $A_o^{-1} d y_o$ to the selected input atom $x_{\sigma(o)}$, and resolves each stage's query credit along the $E_8$ root edge to every counterfactual sibling $b \neq c_m$:
  $$d q_{o,m} \mathrel{+}= \sum_{b \neq c_m} \frac{\langle d y_o,\, y_o^{(m \leftarrow b)} - y_o \rangle}{\|r_{\text{goal}(m,b)} - r_{\text{goal}(m,c_m)}\|_2^2} \Big(r_{\text{goal}(m,b)} - r_{\text{goal}(m,c_m)}\Big).$$

---

## 2. Instrument vs. Mechanism Classification (`CHARTER.md` §4.1, §4.2)

Before calling any descent or no-descent outcome a mechanism result, the protocol in [`docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`](../../harness/prep/r-secant-surrogate-descent-2026-10-25.md) enforces three instrument gates:

1. **`I-Finite-CUDA`:** Zero device error codes (`error == 0`), zero `NaN`/`Inf` values, and non-zero step-0 surrogate credit norms ($\|d s\|_2 > 0$, $\|d q\|_2 > 0$).
2. **`I-Boundary-Crossings` (step size vs. cell width):** Because hard selection is piecewise constant inside each table bin ($[i/16, (i+1)/16)$) or $E_8$ Voronoi cell, an optimizer step size that never moves $s$ or $q_{o,m}$ across a cell boundary produces zero forward change even when the secant direction is exact. At least $1.0\%$ of active lanes/queries must cross a threshold over 256 steps; otherwise the run is classified **`instrument` (learning rate below cell width)** and re-run with a calibrated gate/query step size.
3. **`I-Keep-Plateau` (`f32` precision saturation in `KEEP_TABLE[12..15]`):** In `crates/uor-r4-training/src/cuda_stack.cu:249`, `KEEP_TABLE[12..15]` are all `1.0f` in `f32` because $1 - 2^{-26} < \mathrm{ulp}(1.0\text{f})$, making $\kappa_j - \kappa_i = 0$ on the keep channel for $i \ge 12$. If $> 80\%$ of lanes initialize or trap in $i \ge 12$ with $|d\lambda| < 10^{-7}$, the run is classified **`instrument` (`f32` keep-table plateau)** rather than a mechanism negative.

---

## 3. Readiness and Dependency Hold (`float-training-module`, #2823)

- **Pre-registration & FIT:** [`docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`](../../harness/prep/r-secant-surrogate-descent-2026-10-25.md) is staged and FIT-reviewed (`TEST FITNESS: FIT`, [`#2886#issuecomment-6395161953`](https://github.com/UOR-Foundation/uor-r4/issues/2886#issuecomment-6395161953), `CHARTER.md` §4.5).
- **Why GPU execution is held (`status: held` on `float-training-module`, #2823):** Under `CHARTER.md` §3.7 (D37), float forward, gradients, and optimizers are legal offline only in a separately scanned module (`crates/uor-r4-float-train`, [`float-training-module`](../../roadmap/items/float-training-module.md), #2823) with zero dependency on `crates/uor-r4-training`. Because `#2823` is currently `status: ready` and `crates/uor-r4-float-train` has not yet been extracted, `r-secant-surrogate-descent` (#2886) is set to `status: held` under a fresh-context D68 decision review ([`#2886#issuecomment-6395161953`](https://github.com/UOR-Foundation/uor-r4/issues/2886#issuecomment-6395161953)). As soon as `#2823` lands on `main`, this sheet runs on a leased `uor-pod` RTX 5090 under the lab's own D67 pre-run review.
