This review evaluates the provided Lean 4 formalization of Nevanlinna Theory against the standards of rigorous mathematics (as exemplified by journals like the *Annals of Mathematics*), gold-standard references (e.g., Bombieri & Gubler, Ahlfors), and the conventions of the `mathlib` library. The goal is to guide this effort towards a state-of-the-art (SOTA), deep, and general formalization suitable for inclusion in `mathlib`.

The provided skeleton correctly identifies the key components ($n, N, m, T$) and organizational structure of the theory. However, several critical issues regarding mathematical foundations, implementation details, and generality must be addressed.

### 1\. Mathematical Rigor and Foundations

#### A. Handling of Orders and Valuations (Implementation Flaw)

The definitions of `poleOrder` and `zeroOrder` are technically incorrect based on standard mathematical definitions and likely `mathlib` implementations of `MeromorphicOn.order`.

```lean
def poleOrder (z : ℂ) : ℕ :=
  (hf z).order.getOrElse 0
```

The "order" (or valuation) of a meromorphic function at a point $z$ is an integer $k \in \mathbb{Z}$. If $k>0$, it is a zero of multiplicity $k$; if $k<0$, it is a pole of multiplicity $-k$. If `(hf z).order` returns an `Option ℤ` (or similar), using `getOrElse 0` and casting to `ℕ` will not correctly extract the multiplicity of a pole (e.g., casting $-3$ to `ℕ` does not yield $3$).

**Recommendation:** The definitions must correctly utilize the integer valuation and separate the positive and negative parts.

```lean
-- Assuming (hf z).order returns the valuation as an integer (ℤ)
def valuation (f : ℂ → ℂ) (hf : MeromorphicOn f ⊤) (z : ℂ) : ℤ := sorry -- Access the actual valuation

def zeroMultiplicity (f : ℂ → ℂ) (hf : MeromorphicOn f ⊤) (z : ℂ) : ℕ :=
  Int.toNat (max 0 (valuation f hf z))

def poleMultiplicity (f : ℂ → ℂ) (hf : MeromorphicOn f ⊤) (z : ℂ) : ℕ :=
  Int.toNat (max 0 (-(valuation f hf z)))
```

#### B. Analytical Foundations: Logarithms and Infinities

A subtle but critical issue arises from the interaction between the mathematical definitions and Lean's implementation of real functions. In `mathlib`, `Real.log x = 0` if $x \le 0$, and division by zero often yields $0$.

Mathematically, $\log(0) = -\infty$. This behavior is essential for Nevanlinna theory:

1.  **Jensen's Formula:** The integral $\int \log|f(z)| dz$ must account for the $-\infty$ singularities at the zeros of $f$. If `Real.log` truncates to 0, the formula fails.
2.  **Proximity Function `m(r, a)`:** This involves $\log^+(1/|f(z)-a|)$. If $f(z)=a$, this should be $\log^+(\infty) = \infty$. The current implementation likely evaluates this to 0.

**Recommendation:** A rigorous formalization must use extended real numbers (e.g., `EReal` or `ENNReal`) for the integrals and the definitions of $m(r, a)$ to correctly handle the behavior of $\log$ near zero and the resulting infinities in the measure-theoretic framework.

#### C. Finiteness and the Enumerating Function $n(r, a)$

The definitions of `nPoles` and `nZeros` rely on a crucial `sorry`:

```lean
  have h_fin : poles.Finite := sorry
```

**Recommendation:** This must be proven. It requires formalizing that the zeros and poles of a non-constant meromorphic function are isolated (Identity Theorem) and combining this with the compactness of the closed disk $\overline{D(0, r)}$.

### 2\. Generality and SOTA Architecture

`Mathlib` prioritizes generality. The current formalization is limited in scope and architecture.

#### A. Domain Restriction

The formalization assumes `f` is meromorphic on the entire complex plane (`MeromorphicOn f ⊤`).

**Recommendation:** The theory should be generalized to functions meromorphic on a disk $D(0, R)$, where $R$ can be finite or infinite (using `ENNRReal`).

#### B. The Riemann Sphere and Symmetry

The formalization manually separates the case $a=\infty$ (poles) from finite $a \in \mathbb{C}$. This breaks the natural symmetry of the theory and leads to code duplication.

**Recommendation (SOTA Approach):** A state-of-the-art formalization should treat the target space as the Riemann Sphere ($\mathbb{P}^1(\mathbb{C})$ or `ℂ∞`).

1.  **Chordal Metric:** Define the proximity function $m(r, a)$ using the chordal metric $d(f(z), a)$ on the sphere. This unifies the treatment of all $a \in \mathbb{P}^1(\mathbb{C})$.
2.  **Ahlfors-Shimizu Characteristic:** This geometric definition of the characteristic function (based on the spherical derivative and area) naturally emerges from the spherical viewpoint and is equivalent to the classical Nevanlinna characteristic up to $O(1)$. This provides a deeper, more modern formalization.

### 3\. Completeness and Missing Proofs

The skeleton relies on `sorry` for the core theorems.

#### A. The Counting Function $N(r, a)$ and the Stieltjes Integral

The theorem `integrated_counting_eq_sum_log` is essential for connecting the definition of $N(r)$ to Jensen's formula.

$$N(r) = \int_0^r \frac{n(t)-n(0)}{t} dt + C \quad \Longleftrightarrow \quad N(r) = \sum_{|z_k| < r} \log \frac{r}{|z_k|}$$

The proof typically involves Stieltjes integration or integration by parts, which are complex to formalize.

**Recommendation (Pragmatic Approach):** To bypass difficult integration techniques, define $N(r)$ using the *summation* form (the RHS). Then, prove that the derivative of this summation with respect to $r$ is $n(r)/r$. By the Fundamental Theorem of Calculus, this establishes the equivalence robustly.

#### B. Jensen's Formula

`jensen_nevanlinna` is the cornerstone of the theory.

**Recommendation:** This requires a generalized version of Jensen's formula that handles meromorphic functions.

1.  **Integrability:** Prove that the integrand $\log|f(z)|$ is integrable, even with logarithmic singularities at zeros and poles on the boundary circle (this connects back to the need for extended reals).
2.  **The Constant Term:** The `sorry`'d constant term (the leading Laurent coefficient at the origin) needs a formal definition derived from the valuation.

#### C. Analytical Rigor in $m(r, a)$ and $N(r, a)$

The definitions of $N$ and $m$ involve integrals that must be shown to converge.

  * For $N(r, a)$, the behavior of $(n(t)-n(0))/t$ near $t=0$ must be analyzed to ensure integrability (it should be locally zero).
  * For $m(r, a)$, the behavior near poles on the circle $|z|=r$ must be handled (logarithmic singularities are integrable, but this requires proof).

### 4\. Mathlib Style and Conventions

  * **Variable Handling:** Definitions should take `f` and `hf` as explicit arguments rather than relying on ambient `variable` declarations.
  * **Naming:** Use snake\_case (e.g., `n_poles`, `t_char`) instead of CamelCase (`NPoles`).
  * **Modularity:** The `logPlus` function is general and should be moved outside the `Nevanlinna` namespace to a fundamental analysis file.

### Summary Roadmap

To achieve a SOTA formalization for `mathlib`, the following steps are prioritized:

1.  **Correct Foundational Definitions:** Implement `valuation`, `zeroMultiplicity`, and `poleMultiplicity` correctly.
2.  **Adopt Extended Reals:** Rework the definitions involving logarithms and integrals to use `EReal` or `ENNReal` to handle infinities correctly.
3.  **Prove Finiteness:** Formalize the Identity Theorem and compactness arguments to prove the finiteness of zeros/poles in a disk.
4.  **Define N(r) via Summation:** Define $N(r, a)$ as the sum of logs and prove its derivative is $n(r)/r$.
5.  **Formalize Generalized Jensen:** Prove the generalized Poisson-Jensen formula using the robust definitions above.
6.  **Generalize:** Refactor the theory for functions meromorphic on a disk $D(0, R)$.
7.  **Architectural Enhancement (SOTA):** Implement the unified approach using the Riemann Sphere and the chordal metric.

---

[11 tools called]

### High‑level review of the alternative block

- **Good design choices:**
  - Wrapping `ValueDistribution.logCounting`, `proximity`, and `characteristic` into `NPoles`, `NZeros`, `mPoles`, `mZeros`, and `T` is exactly the right SOTA direction. It uses the canonical `mathlib` Nevanlinna API instead of re‑implementing the theory.
  - Introducing an Ahlfors–Shimizu characteristic `TSpherical` and eventually relating it to `T` is also the right long‑term goal, but it should live in a separate “geometric” file and be built on the existing `RiemannSphere` API rather than ad‑hoc `ℂ` formulas.
- **Issues to fix for mathlib style / correctness:**
  - Your new `NPoles`, `NZeros`, `mPoles`, `mZeros`, `T` should not depend on `hf : MeromorphicOn f ⊤`; they’re pure wrappers around `f`, while meromorphicity is only needed in theorems.
  - The sketched `first_main_theorem` proof is not yet wired correctly: it references `characteristic_sub_characteristic_inv_le` and `abs_characteristic_sub_characteristic_shift_le` but does not actually rewrite your `m+N` into `characteristic` nor combine the two `isBigO` results.

Below is a cleaned‑up Stage‑1+ wrapper layer that sits on top of `ValueDistribution` at mathlib level of generality, and sets you up cleanly for the deeper FMT/Ahlfors–Shimizu work.

---

### 1. SOTA wrappers around `ValueDistribution` (replace the commented block)

Add this near the bottom of `BombieriGubler.lean` (after the existing `Nevanlinna` section), or merge into the existing namespace:

```lean:Riemann/Mathlib/Analysis/Complex/Nevanlinna/BombieriGubler.lean
import Mathlib.Analysis.Complex.ValueDistribution.CountingFunction
import Mathlib.Analysis.Complex.ValueDistribution.ProximityFunction
import Mathlib.Analysis.Complex.ValueDistribution.CharacteristicFunction
import Mathlib.Analysis.Complex.ValueDistribution.FirstMainTheorem
import Mathlib.MeasureTheory.Integral.Average
import Mathlib.Analysis.SpecialFunctions.Log.PosLog
import Mathlib.Topology.MetricSpace.Basic
import Mathlib.Geometry.Manifold.RiemannSphere

open Complex MeasureTheory Set Filter Topology Real
open ValueDistribution

namespace Nevanlinna

noncomputable section

variable (f : ℂ → ℂ)

/-!
### 1. Counting and proximity functions as `mathlib` wrappers
These are just renamings of the `ValueDistribution` objects for Bombieri–Gubler notation.
-/

/-- Counting function for poles: `N(r, ∞, f)`. -/
noncomputable def NPoles (r : ℝ) : ℝ :=
  logCounting f ⊤ r

/-- Counting function for zeros of `f - a`: `N(r, a, f)`. -/
noncomputable def NZeros (a : ℂ) (r : ℝ) : ℝ :=
  logCounting f a r

/-- Proximity function to infinity: `m(r, ∞, f)`. -/
noncomputable def mPoles (r : ℝ) : ℝ :=
  proximity f ⊤ r

/-- Proximity function to a finite value: `m(r, a, f)`. -/
noncomputable def mZeros (a : ℂ) (r : ℝ) : ℝ :=
  proximity f a r

/-- Nevanlinna characteristic `T(r, f) = m(r, ∞) + N(r, ∞)`. -/
noncomputable def T (r : ℝ) : ℝ :=
  characteristic f ⊤ r

/-- Unfold `T` as `m(r, ∞) + N(r, ∞)`; this is just `characteristic`’s definition. -/
lemma T_def (r : ℝ) :
    T f r = mPoles f r + NPoles f r := by
  unfold T mPoles NPoles
  simp [characteristic]

end
end Nevanlinna
```

This:

- Aligns your notation *exactly* with `mathlib` (`ValueDistribution`), avoiding the duplication we had earlier in the file.
- Keeps `hf : MeromorphicOn f ⊤` out of the *definitions* and reserved only for theorems.

---

### 2. Next deep step: wrapping the First Main Theorem

`Mathlib`’s First Main Theorem is already present in `ValueDistribution.FirstMainTheorem.lean` as big‑O results:

- `isBigO_characteristic_sub_characteristic_inv`:
  \((T(f,∞) - T(1/f,∞)) = O(1)\)
- `isBigO_characteristic_sub_characteristic_shift`:
  \((T(f,∞) - T(f - a₀,∞)) = O(1)\)

To obtain the classical Bombieri–Gubler statement

\[
m(r,a,f) + N(r,a,f) = T(r,f) + O(1)
\]

you need to show that, for each finite `a : ℂ`,

\[
T(f,a) - T(f,∞) = (m(r,a) + N(r,a)) - T(r,f)
\]

is `O(1)`. The strategy (mathematical, then formal) is:

1. Use the `logCounting_coe_eq_logCounting_sub_const_zero` and `proximity_coe_eq_proximity_sub_const_zero` lemmas to rewrite:

   \[
   T(f,a) = T(f - a, 0)
   \]

2. Use `proximity_inv` and `logCounting_inv` to identify

   \[
   T(f - a, 0) = T((f - a)^{-1}, ∞).
   \]

3. Combine:

   \[
   T(f,a) - T(f,∞) = \bigl(T((f - a)^{-1},∞) - T(f - a,∞)\bigr)
                      + \bigl(T(f - a,∞) - T(f,∞)\bigr)
   \]

   and apply:

   - `isBigO_characteristic_sub_characteristic_inv` to `g := f - a`,
   - `isBigO_characteristic_sub_characteristic_shift` to `f` and `a₀ := a`.

4. Finally rephrase `T(f,a)` and `T(f,∞)` back in terms of `mZeros + NZeros` and `T` using the wrappers you defined above.

You can package this as:

```lean
namespace Nevanlinna
open Asymptotics ValueDistribution

noncomputable section

variable (f : ℂ → ℂ) (hf : MeromorphicOn f ⊤)

/-- Big‑O version of the First Main Theorem:
`(m(r,a) + N(r,a) - T(r,f)) = O(1)` as `r → ∞`. -/
theorem first_main_theorem_bigO (a : ℂ) :
    (fun r : ℝ => mZeros f a r + NZeros f a r - T f r) =O[atTop] (1 : ℝ → ℝ) := by
  -- Step 0: rewrite `m+N` and `T` as characteristic functions.
  have h_char :
      (fun r : ℝ => mZeros f a r + NZeros f a r - T f r) =
        (fun r => characteristic f a r - characteristic f ⊤ r) := by
    funext r
    simp [Nevanlinna.mZeros, Nevanlinna.NZeros,
          Nevanlinna.T, characteristic, proximity, logCounting]
  -- Step 1: express `characteristic f a` via `(f - a)⁻¹` and characteristic at `∞`.
  -- (use `logCounting_coe_eq_logCounting_sub_const_zero`,
  --  `proximity_coe_eq_proximity_sub_const_zero`, `proximity_inv`, `logCounting_inv`).
  -- This step amounts to proving:
  --   `characteristic f a = characteristic (fun z => (f z - a)⁻¹) ⊤`.
  have h_char_eq :
      characteristic f a =
        characteristic (fun z => (f z - a)⁻¹) (⊤ : WithTop ℂ) := by
    -- TODO: combine `proximity_coe_eq_proximity_sub_const_zero`,
    --       `logCounting_coe_eq_logCounting_sub_const_zero`,
    --       `proximity_inv`, `logCounting_inv`.
    sorry

  -- Step 2: apply the two `isBigO` lemmas from `ValueDistribution.FirstMainTheorem`.
  have h_inv :
      (characteristic (fun z => f z - a) ⊤ -
        characteristic (fun z => (f z - a)⁻¹) ⊤) =O[atTop] (1 : ℝ → ℝ) :=
    isBigO_characteristic_sub_characteristic_inv
      (f := fun z => f z - a) (h := hf.sub (MeromorphicOn.const _))
  have h_shift :
      (characteristic f ⊤ - characteristic (fun z => f z - a) ⊤) =O[atTop] (1 : ℝ → ℝ) :=
    isBigO_characteristic_sub_characteristic_shift
      (f := f) (a₀ := a) hf
  -- Step 3: combine `h_inv` and `h_shift` to get
  --   `characteristic f a - characteristic f ⊤ =O[atTop] 1`.
  have h_char_diff :
      (fun r => characteristic f a r - characteristic f ⊤ r) =O[atTop] (1 : ℝ → ℝ) := by
    -- rewrite with `h_char_eq` and add the two `isBigO` estimates
    -- (using `isBigO.add` and algebra on functions).
    sorry

  -- Step 4: put everything together.
  -- From `h_char` we see our target function equals the characteristic difference,
  -- so we can transfer the `isBigO` result.
  refine (h_char ▸ h_char_diff)

end
end Nevanlinna
```

This is the mathematically correct SOTA path:

- It keeps **all analytic depth** in the existing `ValueDistribution` theorems.
- Your `Nevanlinna.first_main_theorem_bigO` becomes a thin “coordinate change” layer from Bombieri–Gubler’s notation to mathlib’s.
- The only remaining TODOs in this wrapper (`h_char_eq`, `h_char_diff`) are **algebraic rewrites** using `proximity_*` and `logCounting_*` lemmas and `isBigO` calculus; they do **not** require new analytic estimates.

---

### 3. Ahlfors–Shimizu characteristic (`TSpherical`)

For `TSpherical` and `ahlfors_shimizu_equivalence`, the alternative block is conceptually right, but to be mathlib‑grade you should:

- Define the chordal metric using `RiemannSphere` and its existing metric structure, not a raw formula on `ℂ`.
- Define `sphericalDerivative` as the norm of the differential with respect to the spherical metric, again using `RiemannSphere`’s Riemannian structure if available (or via the standard Euclidean expression, but justified from the geometry).
- Put `TSpherical` and `ahlfors_shimizu_equivalence` in a separate file (e.g. `Nevanlinna/AhlforsShimizu.lean`), importing both `ValueDistribution` and `RiemannSphere`.

Those theorems are deep and will require substantial new analysis; for now, it’s enough to have clean **definitions** and precise theorem statements, with `sorry` placeholders.

---

In summary: the alternative formalization’s architecture (wrapping `ValueDistribution` and aiming at `TSpherical`) is exactly what you want. The next concrete SOTA steps are:

- Replace the ad‑hoc redefinitions by the simple wrappers shown above;
- Add the `first_main_theorem_bigO` skeleton that reduces to the existing `ValueDistribution` lemmas;
- Move the Ahlfors–Shimizu part into its own geometric file and start with clean definitions there.
