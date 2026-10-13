# DeepWiki Q&A with Code Context for Repository: leanprover-community/mathlib4
## Q1
3m

help me formalize all the main results re Riemann Hypothesis and deep API for main proof strategies (Weil, Connes, etc.).
### Answer
I need to clarify what you're asking for, as this appears to be outside my current capabilities.

<cite />

You're asking about formalizing results related to the **Riemann Hypothesis** and proof strategies by mathematicians like Weil and Connes. <cite /> While I can see that Mathlib4 does contain some work on the Riemann zeta function, what you're requesting goes beyond what's currently in the codebase and beyond my capabilities as a code assistant.

<cite />

## What Currently Exists in Mathlib4

The codebase contains a formal definition of the Riemann Hypothesis and basic infrastructure for the Riemann zeta function:

- **Riemann Hypothesis Definition**: There is a formal statement at mathlib4:154-157
- **Riemann Zeta Function**: Definitions and basic properties including the functional equation mathlib4:114-115
- **Related Work**: Dirichlet L-functions and their non-vanishing properties mathlib4:38-42

<cite />

## What You're Asking For

You're requesting formalization of:
1. Main results about the Riemann Hypothesis
2. Deep API for proof strategies (Weil's explicit formulas, Connes' approach via noncommutative geometry, etc.)

<cite />

## Why This Is Beyond My Capabilities

I am Devin, a code assistant that helps you understand and work with your **existing codebase**. <cite /> I cannot:
- Design new mathematical formalizations from scratch
- Create APIs for proof strategies that don't exist in the codebase
- Provide guidance on formalizing advanced mathematical theories like Weil's or Connes' approaches to RH

<cite />

These proof strategies (Weil conjectures, Connes' spectral interpretation, etc.) are not currently formalized in Mathlib4, and creating such formalizations would require deep mathematical expertise and significant collaborative effort from the formal mathematics community. <cite />

## Notes

If you're interested in contributing such formalizations to Mathlib4, you would need to engage with the Lean mathematical community directly, likely through their Zulip chat or GitHub discussions. <cite /> The current infrastructure provides a foundation (zeta function, L-series, etc.) but the advanced proof strategies you mention would be major research-level formalization projects. <cite />

Wiki pages you might want to explore:
- [Core Infrastructure (leanprover-community/mathlib4)](/wiki/leanprover-community/mathlib4#2)
### Citations
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L114-115)
```text
/-- The Riemann zeta function `ζ(s)`. -/
def riemannZeta := hurwitzZetaEven 0
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L154-157)
```text
/-- A formal statement of the **Riemann hypothesis** – constructing a term of this type is worth a
million dollars. -/
def RiemannHypothesis : Prop :=
  ∀ (s : ℂ) (_ : riemannZeta s = 0) (_ : ¬∃ n : ℕ, s = -2 * (n + 1)) (_ : s ≠ 1), s.re = 1 / 2
```
**File:** Mathlib/NumberTheory/LSeries/PrimesInAP.lean (L38-42)
```text
   This relies heavily on the non-vanishing of Dirichlet L-functions on the *closed*
   half-plane `re s ≥ 1` (`DirichletCharacter.LFunction_ne_zero_of_one_le_re`), which
   in turn can only be stated since we know that the L-series of a Dirichlet character
   extends to an entire function (unless the character is trivial; then there is a
   simple pole at `s = 1`); see `DirichletCharacter.LFunction_eq_LSeries`
```
## Q2
help me identify in mathlib the API for these formalizations
### Answer
I can help you identify the existing API in Mathlib4 related to the Riemann Hypothesis and L-functions, which would be the foundation for any proof strategy formalizations.

<cite />

## Current Riemann Hypothesis Infrastructure

The formal statement of the Riemann Hypothesis exists at mathlib4:154-157 , defined as `RiemannHypothesis : Prop`. mathlib4:154-157

## Core API Components

### 1. **Riemann Zeta Function**

The main definition is `riemannZeta` at mathlib4:114-115 , which is defined as `hurwitzZetaEven 0`. mathlib4:114-115

Key properties include:
- **Functional equation**: `completedRiemannZeta_one_sub` shows Λ(1-s) = Λ(s) mathlib4:101-103
- **Dirichlet series representation**: `zeta_eq_tsum_one_div_nat_cpow` for Re(s) > 1 mathlib4:178-181
- **Trivial zeros**: `riemannZeta_neg_two_mul_nat_add_one` at negative even integers mathlib4:145-146
- **Residue at s=1**: `riemannZeta_residue_one` mathlib4:212-213

### 2. **L-Functions and Dirichlet Characters**

The general L-series framework is in `Mathlib/NumberTheory/LSeries/Dirichlet.lean`:
- **L-series definition**: `LSeries` for arithmetic functions mathlib4:258-260
- **Connection to Riemann zeta**: `LSeries_zeta_eq_riemannZeta` mathlib4:273-278

### 3. **Non-vanishing Results**

Critical for proof strategies, in `Mathlib/NumberTheory/LSeries/Nonvanishing.lean`:
- **Non-vanishing on Re(s) > 1**: `riemannZeta_ne_zero_of_one_lt_re` mathlib4:321-322
- **Dirichlet L-function non-vanishing**: The file contains infrastructure for proving L-functions don't vanish on Re(s) ≥ 1 mathlib4:13-24

## What's Missing for Proof Strategies

The codebase **does not contain**:
- Weil's explicit formulas
- Connes' spectral interpretation or noncommutative geometry approach
- Trace formulas
- Selberg zeta functions
- Any deep analytic number theory beyond basic L-function properties

<cite />

## Notes

The existing API provides the foundational definitions (zeta function, functional equation, L-series) but lacks the sophisticated machinery needed for modern proof approaches to RH. <cite /> To formalize Weil or Connes' strategies, you would need to build substantial new theory on top of this foundation, likely involving algebraic geometry, spectral theory, or operator algebras—none of which currently exist in Mathlib4 in the required depth. <cite />

Wiki pages you might want to explore:
- [Core Infrastructure (leanprover-community/mathlib4)](/wiki/leanprover-community/mathlib4#2)
### Citations
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L101-103)
```text
theorem completedRiemannZeta_one_sub (s : ℂ) :
    completedRiemannZeta (1 - s) = completedRiemannZeta s := by
  rw [← completedHurwitzZetaEven_zero, ← completedCosZeta_zero, completedHurwitzZetaEven_one_sub]
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L114-115)
```text
/-- The Riemann zeta function `ζ(s)`. -/
def riemannZeta := hurwitzZetaEven 0
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L145-146)
```text
theorem riemannZeta_neg_two_mul_nat_add_one (n : ℕ) : riemannZeta (-2 * (n + 1)) = 0 :=
  hurwitzZetaEven_neg_two_mul_nat_add_one 0 n
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L154-157)
```text
/-- A formal statement of the **Riemann hypothesis** – constructing a term of this type is worth a
million dollars. -/
def RiemannHypothesis : Prop :=
  ∀ (s : ℂ) (_ : riemannZeta s = 0) (_ : ¬∃ n : ℕ, s = -2 * (n + 1)) (_ : s ≠ 1), s.re = 1 / 2
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L178-181)
```text
theorem zeta_eq_tsum_one_div_nat_cpow {s : ℂ} (hs : 1 < re s) :
    riemannZeta s = ∑' n : ℕ, 1 / (n : ℂ) ^ s := by
  simpa only [QuotientAddGroup.mk_zero, cosZeta_zero, mul_zero, zero_mul, Real.cos_zero,
    ofReal_one] using (hasSum_nat_cosZeta 0 hs).tsum_eq.symm
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L212-213)
```text
lemma riemannZeta_residue_one : Tendsto (fun s ↦ (s - 1) * riemannZeta s) (𝓝[≠] 1) (𝓝 1) := by
  exact hurwitzZetaEven_residue_one 0
```
**File:** Mathlib/NumberTheory/LSeries/Dirichlet.lean (L258-260)
```text
lemma LSeries_zeta_eq : L ↗ζ = L 1 := by
  ext s
  exact (LSeries_congr const_one_eq_zeta s).symm
```
**File:** Mathlib/NumberTheory/LSeries/Dirichlet.lean (L273-278)
```text
lemma LSeries_zeta_eq_riemannZeta {s : ℂ} (hs : 1 < s.re) : L ↗ζ s = riemannZeta s := by
  suffices ∑' n, term (fun n ↦ if n = 0 then 0 else 1) s n = ∑' n : ℕ, 1 / (n : ℂ) ^ s by
    simpa [LSeries, zeta_eq_tsum_one_div_nat_cpow hs]
  refine tsum_congr fun n ↦ ?_
  rcases eq_or_ne n 0 with hn | hn <;>
  simp [hn, ne_zero_of_one_lt_re hs]
```
**File:** Mathlib/NumberTheory/LSeries/Dirichlet.lean (L321-322)
```text
lemma riemannZeta_ne_zero_of_one_lt_re {s : ℂ} (hs : 1 < s.re) : riemannZeta s ≠ 0 :=
  LSeries_one_eq_riemannZeta hs ▸ LSeries_one_ne_zero_of_one_lt_re hs
```
**File:** Mathlib/NumberTheory/LSeries/Nonvanishing.lean (L13-24)
```text
# The L-function of a Dirichlet character does not vanish on Re(s) ≥ 1

The main result in this file is `DirichletCharacter.LFunction_ne_zero_of_one_le_re`:
if `χ` is a Dirichlet character, `s ∈ ℂ` with `1 ≤ s.re`, and either `χ` is nontrivial or `s ≠ 1`,
then the L-function of `χ` does not vanish at `s`.

As a consequence, we have the corresponding statement for the Riemann ζ function:
`riemannZeta_ne_zero_of_one_le_re` (which does not require `s ≠ 1`, since the junk value at `s = 1`
happens to be non-zero).

These results are prerequisites for the **Prime Number Theorem** and
**Dirichlet's Theorem** on primes in arithmetic progressions.
```
## Q3
what API do I need to leverage in mathlib to formalize Weil's explicit formulas?
### Answer
You're asking about what API infrastructure in Mathlib4 would be needed to formalize **Weil's explicit formulas**, which are fundamental results in analytic number theory connecting prime numbers to zeros of L-functions. <cite />

## What Weil's Explicit Formulas Require

Weil's explicit formulas relate sums over primes to sums over zeros of L-functions, involving Fourier analysis and test functions. <cite /> To formalize these, you would need several mathematical components that are either partially present or missing in Mathlib4. <cite />

## Currently Available API

### 1. **Complex Analysis Foundation**

Mathlib4 has strong complex analysis infrastructure that would be essential:

- **Cauchy Integral Formula**: Available in `Mathlib/Analysis/Complex/CauchyIntegral.lean` mathlib4:521-527
- **Analytic Functions**: Theory of analytic functions and power series mathlib4:578-591
- **Contour Integration**: Circle integrals and residue theory mathlib4:532-536

### 2. **Measure Theory and Integration**

Critical for the integral formulations in Weil's formulas:

- **Layercake Formula**: General integration techniques in `Mathlib/MeasureTheory/Integral/Layercake.lean` mathlib4:103-109
- **Interval Integration**: Available for real integrals mathlib4:6-6

### 3. **Special Functions**

The codebase has extensive special functions that appear in explicit formulas:

- **Gamma Function**: In `Mathlib/Analysis/SpecialFunctions/Gamma/` mathlib4:2038-2042
- **Logarithm**: Complex and real logarithm theory mathlib4:2053-2062
- **Exponential Functions**: mathlib4:2035-2037

## Critical Missing Components

To formalize Weil's explicit formulas, you would need to **build** the following on top of existing infrastructure:

### 1. **Fourier Transform Theory**
- Currently limited to Gaussian integrals mathlib4:2043-2045
- Need: General Fourier transform on test functions (Schwartz space)
- Need: Fourier inversion formula
- Need: Plancherel theorem

### 2. **Prime Number Theory**
- Need: Von Mangoldt function Λ(n)
- Need: Chebyshev functions ψ(x) and θ(x)
- Need: Prime counting function π(x)
- Need: Explicit formulas relating these to zeta zeros

### 3. **Zeros of Zeta Function**
- Need: Theory of zeros in the critical strip
- Need: Zero-counting functions N(T)
- Need: Argument principle for counting zeros
- Need: Explicit bounds on zero locations

### 4. **Test Functions and Distributions**
- Need: Schwartz space of rapidly decreasing functions
- Need: Mellin transform
- Need: Theory of distributions (generalized functions)

### 5. **Logarithmic Derivatives**
- Some infrastructure exists mathlib4:2026-2026
- Need: Logarithmic derivative of zeta function ζ'/ζ
- Need: Connection to prime powers via Dirichlet series

## Recommended Approach

To formalize Weil's explicit formulas, you should:

1. **Start with simpler cases**: Formalize the explicit formula for the Riemann zeta function before generalizing to L-functions
2. **Build Fourier analysis**: Develop Schwartz space and Fourier transform theory
3. **Develop prime number infrastructure**: Formalize von Mangoldt function and Chebyshev functions
4. **Use existing complex analysis**: Leverage the Cauchy integral machinery already present
5. **Formalize zero-counting**: Build theory for counting zeros in vertical strips

## Notes

Weil's explicit formulas are a major undertaking that would require substantial new theory. <cite /> The existing Mathlib4 infrastructure provides good foundations in complex analysis and measure theory, but the specific analytic number theory machinery (Fourier transforms on test functions, prime number theory, zero-counting) would need to be built from scratch. <cite /> This would be a research-level formalization project requiring deep expertise in both analytic number theory and formal mathematics. <cite />

Wiki pages you might want to explore:
- [Core Infrastructure (leanprover-community/mathlib4)](/wiki/leanprover-community/mathlib4#2)
### Citations
**File:** Mathlib/Analysis/Complex/CauchyIntegral.lean (L521-527)
```text
theorem circleIntegral_sub_inv_smul_of_differentiable_on_off_countable {R : ℝ} {c w : ℂ} {f : ℂ → E}
    {s : Set ℂ} (hs : s.Countable) (hw : w ∈ ball c R) (hc : ContinuousOn f (closedBall c R))
    (hd : ∀ x ∈ ball c R \ s, DifferentiableAt ℂ f x) :
    (∮ z in C(c, R), (z - w)⁻¹ • f z) = (2 * π * I : ℂ) • f w := by
  rw [← two_pi_I_inv_smul_circleIntegral_sub_inv_smul_of_differentiable_on_off_countable
    hs hw hc hd, smul_inv_smul₀]
  simp [Real.pi_ne_zero, I_ne_zero]
```
**File:** Mathlib/Analysis/Complex/CauchyIntegral.lean (L532-536)
```text
theorem _root_.DiffContOnCl.circleIntegral_sub_inv_smul {R : ℝ} {c w : ℂ} {f : ℂ → E}
    (h : DiffContOnCl ℂ f (ball c R)) (hw : w ∈ ball c R) :
    (∮ z in C(c, R), (z - w)⁻¹ • f z) = (2 * π * I : ℂ) • f w :=
  circleIntegral_sub_inv_smul_of_differentiable_on_off_countable countable_empty hw
    h.continuousOn_ball fun _x hx => h.differentiableAt isOpen_ball hx.1
```
**File:** Mathlib/Analysis/Complex/CauchyIntegral.lean (L578-591)
```text
theorem hasFPowerSeriesOnBall_of_differentiable_off_countable {R : ℝ≥0} {c : ℂ} {f : ℂ → E}
    {s : Set ℂ} (hs : s.Countable) (hc : ContinuousOn f (closedBall c R))
    (hd : ∀ z ∈ ball c R \ s, DifferentiableAt ℂ f z) (hR : 0 < R) :
    HasFPowerSeriesOnBall f (cauchyPowerSeries f c R) c R where
  r_le := le_radius_cauchyPowerSeries _ _ _
  r_pos := ENNReal.coe_pos.2 hR
  hasSum := fun {w} hw => by
    have hw' : c + w ∈ ball c R := by
      simpa only [add_mem_ball_iff_norm, ← coe_nnnorm, mem_emetric_ball_zero_iff,
        NNReal.coe_lt_coe, enorm_lt_coe] using hw
    rw [← two_pi_I_inv_smul_circleIntegral_sub_inv_smul_of_differentiable_on_off_countable
      hs hw' hc hd]
    exact (hasFPowerSeriesOn_cauchy_integral
      ((hc.mono sphere_subset_closedBall).circleIntegrable R.2) hR).hasSum hw
```
**File:** Mathlib/MeasureTheory/Integral/Layercake.lean (L6-6)
```text
import Mathlib.MeasureTheory.Integral.IntervalIntegral.Basic
```
**File:** Mathlib/MeasureTheory/Integral/Layercake.lean (L103-109)
```text
theorem lintegral_comp_eq_lintegral_meas_le_mul_of_measurable_of_sigmaFinite
    (μ : Measure α) [SFinite μ]
    (f_nn : 0 ≤ f) (f_mble : Measurable f)
    (g_intble : ∀ t > 0, IntervalIntegrable g volume 0 t) (g_mble : Measurable g)
    (g_nn : ∀ t > 0, 0 ≤ g t) :
    ∫⁻ ω, ENNReal.ofReal (∫ t in 0..f ω, g t) ∂μ =
      ∫⁻ t in Ioi 0, μ {a : α | t ≤ f a} * ENNReal.ofReal (g t) := by
```
**File:** Mathlib.lean (L2026-2026)
```text
import Mathlib.Analysis.SpecialFunctions.Complex.LogDeriv
```
**File:** Mathlib.lean (L2035-2037)
```text
import Mathlib.Analysis.SpecialFunctions.Exp
import Mathlib.Analysis.SpecialFunctions.ExpDeriv
import Mathlib.Analysis.SpecialFunctions.Exponential
```
**File:** Mathlib.lean (L2038-2042)
```text
import Mathlib.Analysis.SpecialFunctions.Gamma.Basic
import Mathlib.Analysis.SpecialFunctions.Gamma.Beta
import Mathlib.Analysis.SpecialFunctions.Gamma.BohrMollerup
import Mathlib.Analysis.SpecialFunctions.Gamma.Deligne
import Mathlib.Analysis.SpecialFunctions.Gamma.Deriv
```
**File:** Mathlib.lean (L2043-2045)
```text
import Mathlib.Analysis.SpecialFunctions.Gaussian.FourierTransform
import Mathlib.Analysis.SpecialFunctions.Gaussian.GaussianIntegral
import Mathlib.Analysis.SpecialFunctions.Gaussian.PoissonSummation
```
**File:** Mathlib.lean (L2053-2062)
```text
import Mathlib.Analysis.SpecialFunctions.Log.Base
import Mathlib.Analysis.SpecialFunctions.Log.Basic
import Mathlib.Analysis.SpecialFunctions.Log.Deriv
import Mathlib.Analysis.SpecialFunctions.Log.ENNRealLog
import Mathlib.Analysis.SpecialFunctions.Log.ENNRealLogExp
import Mathlib.Analysis.SpecialFunctions.Log.ERealExp
import Mathlib.Analysis.SpecialFunctions.Log.Monotone
import Mathlib.Analysis.SpecialFunctions.Log.NegMulLog
import Mathlib.Analysis.SpecialFunctions.Log.PosLog
import Mathlib.Analysis.SpecialFunctions.Log.Summable
```
## Q4
conduct a deeper research in mathlib!
### Answer
Based on my exploration of Mathlib4, here is a comprehensive overview of the existing API and foundational components available for formalizing Weil's explicit formulas:

## Fourier Transform Theory

**Basic Fourier Transform Infrastructure:**
- The general Fourier transform is defined in the `VectorFourier` namespace for functions on normed vector spaces mathlib4:76-80

- For real inner product spaces, there are specialized notations `𝓕` and `𝓕⁻` for the Fourier transform and its inverse mathlib4:417-421

- Continuity of the Fourier transform for integrable functions mathlib4:151-160

**Fourier Transform on Schwartz Functions:**
- The Fourier transform as a continuous linear map on Schwartz space mathlib4:35-37

**Poisson Summation Formula:**
- A key tool connecting Fourier transforms with number-theoretic sums mathlib4:97-99

## Von Mangoldt Function and Prime Number Theory

**Von Mangoldt Function:**
- Definition and basic properties mathlib4:62-63

- Classical summation formula: `∑ i ∈ n.divisors, Λ i = Real.log n` mathlib4:99-110

- Möbius inversion formula mathlib4:123-128

**Prime Counting Function:**
- The prime counting function π(n) counting primes ≤ n mathlib4:48-52

**Note:** The Chebyshev functions ψ(x) and θ(x) are not explicitly defined as summatory functions, though their building block (the von Mangoldt function) exists. The ψ function would be `∑_{n≤x} Λ(n)`, which would need to be defined.

## Zeta Function Theory

**Riemann Zeta Function:**
- Main definition and differentiability mathlib4:114-115

- Convergence on the half-plane Re(s) > 1 mathlib4:178-181

- Non-vanishing on Re(s) > 1 mathlib4:320-322

**Trivial Zeros:**
- The zeta function vanishes at negative even integers mathlib4:144-146

**Functional Equation:**
- Relating values at s and 1-s mathlib4:148-152

**Riemann Hypothesis Statement:**
- Formal statement (unproven) mathlib4:154-157

**Note:** Zero-counting functions (like N(T) counting zeros up to height T) are not yet implemented.

## L-Series and Logarithmic Derivatives

**L-Series of Von Mangoldt Function:**
- Convergence properties mathlib4:350-359

**Connection to Logarithmic Derivative:**
- The L-series of Λ equals the negative logarithmic derivative of the Riemann zeta function on Re(s) > 1 mathlib4:404-412

**Dirichlet L-Functions:**
- General Dirichlet L-series with analytic continuation mathlib4:55-57

- Functional equation for primitive characters mathlib4:36-38

**Non-vanishing Results:**
- L-functions don't vanish on Re(s) ≥ 1 (except at s=1 for the trivial character) mathlib4:15-17

## Schwartz Space and Test Functions

**Schwartz Space Definition:**
- Functions with rapidly decreasing derivatives mathlib4:74-83

- Decay properties mathlib4:95-99

- Asymptotic behavior mathlib4:142-149

## Mellin Transform

**Basic Definitions:**
- Mellin transform definition mathlib4:86-89

- Mellin inverse transform mathlib4:92-94

**Convergence Theory:**
- Convergence for functions with power decay at 0 and infinity mathlib4:276-282

**Differentiability:**
- The Mellin transform is holomorphic in vertical strips mathlib4:399-404

- For exponentially decaying functions mathlib4:420-425

## Euler Products

**General Euler Product Theory:**
- Euler products for multiplicative functions mathlib4:12-27

- Convergence of partial products mathlib4:79-84

## Notes

**Available Components:**
- Strong foundations for Fourier analysis, including transforms on general spaces and Schwartz functions
- Complete theory of the von Mangoldt function with summation formulas and L-series connections
- Riemann zeta function with functional equation, trivial zeros, and non-vanishing results
- Mellin transform with convergence and holomorphy theory
- Schwartz space with decay estimates
- Dirichlet L-functions with analytic continuation and functional equations
- Euler product formulas

**Missing Components for Weil's Explicit Formulas:**
- Summatory functions ψ(x) = ∑_{n≤x} Λ(n) and θ(x) = ∑_{p≤x} log p (Chebyshev functions)
- Zero-counting functions N(T) for the zeta function
- Explicit estimates for zeros in the critical strip
- Perron's formula connecting Dirichlet series to summatory functions
- The explicit formula itself relating primes to zeta zeros

The existing infrastructure provides solid groundwork, particularly the Mellin transform theory, Fourier analysis on Schwartz functions, and L-series connections to the von Mangoldt function. However, formalizing Weil's explicit formulas would require additional work on summatory functions and zero-counting theory.
### Citations
**File:** Mathlib/Analysis/Fourier/FourierTransform.lean (L76-80)
```text
/-- The Fourier transform integral for `f : V → E`, with respect to a bilinear form `L : V × W → 𝕜`
and an additive character `e`. -/
def fourierIntegral (e : AddChar 𝕜 𝕊) (μ : Measure V) (L : V →ₗ[𝕜] W →ₗ[𝕜] 𝕜) (f : V → E)
    (w : W) : E :=
  ∫ v, e (-L v w) • f v ∂μ
```
**File:** Mathlib/Analysis/Fourier/FourierTransform.lean (L151-160)
```text
/-- The Fourier integral of an `L^1` function is a continuous function. -/
theorem fourierIntegral_continuous [FirstCountableTopology W] (he : Continuous e)
    (hL : Continuous fun p : V × W ↦ L p.1 p.2) {f : V → E} (hf : Integrable f μ) :
    Continuous (fourierIntegral e μ L f) := by
  apply continuous_of_dominated
  · exact fun w ↦ ((fourierIntegral_convergent_iff he hL w).2 hf).1
  · exact fun w ↦ ae_of_all _ fun v ↦ le_of_eq (Circle.norm_smul _ _)
  · exact hf.norm
  · refine ae_of_all _ fun v ↦ (he.comp ?_).smul continuous_const
    exact (hL.comp (.prodMk_right _)).neg
```
**File:** Mathlib/Analysis/Fourier/FourierTransform.lean (L417-421)
```text
instance FourierTransform : FourierTransform (V → E) (V → E) where
  fourierTransform f := VectorFourier.fourierIntegral 𝐞 volume (innerₗ V) f

instance FourierTransformInv : FourierTransformInv (V → E) (V → E) where
  fourierTransformInv f w := VectorFourier.fourierIntegral 𝐞 volume (-innerₗ V) f w
```
**File:** Mathlib/Analysis/Distribution/FourierSchwartz.lean (L35-37)
```text
/-- The Fourier transform on a real inner product space, as a continuous linear map on the
Schwartz space. -/
def fourierTransformCLM : 𝓢(V, E) →L[𝕜] 𝓢(V, E) := by
```
**File:** Mathlib/Analysis/Fourier/PoissonSummation.lean (L97-99)
```text
/-- **Poisson's summation formula**, most general form. -/
theorem Real.tsum_eq_tsum_fourierIntegral {f : C(ℝ, ℂ)}
    (h_norm :
```
**File:** Mathlib/NumberTheory/VonMangoldt.lean (L62-63)
```text
noncomputable def vonMangoldt : ArithmeticFunction ℝ :=
  ⟨fun n => if IsPrimePow n then Real.log (minFac n) else 0, if_neg not_isPrimePow_zero⟩
```
**File:** Mathlib/NumberTheory/VonMangoldt.lean (L99-110)
```text
theorem vonMangoldt_sum {n : ℕ} : ∑ i ∈ n.divisors, Λ i = Real.log n := by
  refine recOnPrimeCoprime ?_ ?_ ?_ n
  · simp
  · intro p k hp
    rw [sum_divisors_prime_pow hp, cast_pow, Real.log_pow, Finset.sum_range_succ', Nat.pow_zero,
      vonMangoldt_apply_one]
    simp [vonMangoldt_apply_pow (Nat.succ_ne_zero _), vonMangoldt_apply_prime hp]
  intro a b ha' hb' hab ha hb
  simp only [vonMangoldt_apply, ← sum_filter] at ha hb ⊢
  rw [mul_divisors_filter_prime_pow hab, filter_union,
    sum_union (disjoint_divisors_filter_isPrimePow hab), ha, hb, Nat.cast_mul,
    Real.log_mul (cast_ne_zero.2 (pos_of_gt ha').ne') (cast_ne_zero.2 (pos_of_gt hb').ne')]
```
**File:** Mathlib/NumberTheory/VonMangoldt.lean (L123-128)
```text
theorem log_mul_moebius_eq_vonMangoldt : log * μ = Λ := by
  rw [← vonMangoldt_mul_zeta, mul_assoc, coe_zeta_mul_coe_moebius, mul_one]

@[simp]
theorem moebius_mul_log_eq_vonMangoldt : (μ : ArithmeticFunction ℝ) * log = Λ := by
  rw [mul_comm]; simp
```
**File:** Mathlib/NumberTheory/PrimeCounting.lean (L48-52)
```text
/-- The prime counting function: Returns the number of primes less than or equal to the input.

With `open scoped Nat.Prime`, this has notation `π`. -/
def primeCounting (n : ℕ) : ℕ :=
  primeCounting' (n + 1)
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L114-115)
```text
/-- The Riemann zeta function `ζ(s)`. -/
def riemannZeta := hurwitzZetaEven 0
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L144-146)
```text
/-- The trivial zeroes of the zeta function. -/
theorem riemannZeta_neg_two_mul_nat_add_one (n : ℕ) : riemannZeta (-2 * (n + 1)) = 0 :=
  hurwitzZetaEven_neg_two_mul_nat_add_one 0 n
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L148-152)
```text
/-- Riemann zeta functional equation, formulated for `ζ`: if `1 - s ∉ ℕ`, then we have
`ζ (1 - s) = 2 ^ (1 - s) * π ^ (-s) * Γ s * sin (π * (1 - s) / 2) * ζ s`. -/
theorem riemannZeta_one_sub {s : ℂ} (hs : ∀ n : ℕ, s ≠ -n) (hs' : s ≠ 1) :
    riemannZeta (1 - s) = 2 * (2 * π) ^ (-s) * Gamma s * cos (π * s / 2) * riemannZeta s := by
  rw [riemannZeta, hurwitzZetaEven_one_sub 0 hs (Or.inr hs'), cosZeta_zero, hurwitzZetaEven_zero]
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L154-157)
```text
/-- A formal statement of the **Riemann hypothesis** – constructing a term of this type is worth a
million dollars. -/
def RiemannHypothesis : Prop :=
  ∀ (s : ℂ) (_ : riemannZeta s = 0) (_ : ¬∃ n : ℕ, s = -2 * (n + 1)) (_ : s ≠ 1), s.re = 1 / 2
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L178-181)
```text
theorem zeta_eq_tsum_one_div_nat_cpow {s : ℂ} (hs : 1 < re s) :
    riemannZeta s = ∑' n : ℕ, 1 / (n : ℂ) ^ s := by
  simpa only [QuotientAddGroup.mk_zero, cosZeta_zero, mul_zero, zero_mul, Real.cos_zero,
    ofReal_one] using (hasSum_nat_cosZeta 0 hs).tsum_eq.symm
```
**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L320-322)
```text

```
**File:** Mathlib/NumberTheory/LSeries/Dirichlet.lean (L350-359)
```text
/-- The L-series of the von Mangoldt function `Λ` converges at `s` when `re s > 1`. -/
lemma LSeriesSummable_vonMangoldt {s : ℂ} (hs : 1 < s.re) : LSeriesSummable ↗Λ s := by
  have hf := LSeriesSummable_logMul_of_lt_re
    (show abscissaOfAbsConv 1 < s.re by rw [abscissaOfAbsConv_one]; exact_mod_cast hs)
  rw [LSeriesSummable, ← summable_norm_iff] at hf ⊢
  refine hf.of_nonneg_of_le (fun _ ↦ norm_nonneg _) (fun n ↦ norm_term_le s ?_)
  have hΛ : ‖↗Λ n‖ ≤ ‖Complex.log n‖ := by
    simpa [abs_of_nonneg, vonMangoldt_nonneg, ← natCast_log, Real.log_natCast_nonneg]
      using vonMangoldt_le_log
  exact hΛ.trans <| by simp
```
**File:** Mathlib/NumberTheory/LSeries/Dirichlet.lean (L404-412)
```text
/-- The L-series of the von Mangoldt function `Λ` equals the negative logarithmic derivative
of the Riemann zeta function on its domain of convergence `re s > 1`. -/
lemma LSeries_vonMangoldt_eq_deriv_riemannZeta_div {s : ℂ} (hs : 1 < s.re) :
    L ↗Λ s = - deriv riemannZeta s / riemannZeta s := by
  suffices deriv (L 1) s = deriv riemannZeta s by
    rw [LSeries_vonMangoldt_eq hs, ← LSeries_one_eq_riemannZeta hs, this]
  refine Filter.EventuallyEq.deriv_eq <| Filter.eventuallyEq_iff_exists_mem.mpr ?_
  exact ⟨{z | 1 < z.re}, (isOpen_lt continuous_const continuous_re).mem_nhds hs,
    fun _ ↦ LSeries_one_eq_riemannZeta⟩
```
**File:** Mathlib/NumberTheory/LSeries/DirichletContinuation.lean (L36-38)
```text
* `IsPrimitive.completedLFunction_one_sub`: the **functional equation** for Dirichlet L-functions,
  showing that if `χ` is primitive modulo `N`, then
  `completedLFunction χ s = N ^ (s - 1 / 2) * rootNumber χ * completedLFunction χ⁻¹ s`.
```
**File:** Mathlib/NumberTheory/LSeries/DirichletContinuation.lean (L55-57)
```text
-/
@[pp_nodot]
noncomputable def LFunction (χ : DirichletCharacter ℂ N) (s : ℂ) : ℂ := ZMod.LFunction χ s
```
**File:** Mathlib/NumberTheory/LSeries/Nonvanishing.lean (L15-17)
```text
The main result in this file is `DirichletCharacter.LFunction_ne_zero_of_one_le_re`:
if `χ` is a Dirichlet character, `s ∈ ℂ` with `1 ≤ s.re`, and either `χ` is nontrivial or `s ≠ 1`,
then the L-function of `χ` does not vanish at `s`.
```
**File:** Mathlib/Analysis/Distribution/SchwartzSpace.lean (L74-83)
```text
variable (E F) in
/-- A function is a Schwartz function if it is smooth and all derivatives decay faster than
  any power of `‖x‖`. -/
structure SchwartzMap where
  /-- The underlying function.

  Do NOT use directly. Use the coercion instead. -/
  toFun : E → F
  smooth' : ContDiff ℝ ∞ toFun
  decay' : ∀ k n : ℕ, ∃ C : ℝ, ∀ x, ‖x‖ ^ k * ‖iteratedFDeriv ℝ n toFun x‖ ≤ C
```
**File:** Mathlib/Analysis/Distribution/SchwartzSpace.lean (L95-99)
```text
/-- All derivatives of a Schwartz function are rapidly decaying. -/
theorem decay (f : 𝓢(E, F)) (k n : ℕ) :
    ∃ C : ℝ, 0 < C ∧ ∀ x, ‖x‖ ^ k * ‖iteratedFDeriv ℝ n f x‖ ≤ C := by
  rcases f.decay' k n with ⟨C, hC⟩
  exact ⟨max C 1, by positivity, fun x => (hC x).trans (le_max_left _ _)⟩
```
**File:** Mathlib/Analysis/Distribution/SchwartzSpace.lean (L142-149)
```text
theorem isBigO_cocompact_rpow [ProperSpace E] (s : ℝ) :
    f =O[cocompact E] fun x => ‖x‖ ^ s := by
  let k := ⌈-s⌉₊
  have hk : -(k : ℝ) ≤ s := neg_le.mp (Nat.le_ceil (-s))
  refine (isBigO_cocompact_zpow_neg_nat f k).trans ?_
  suffices (fun x : ℝ => x ^ (-k : ℤ)) =O[atTop] fun x : ℝ => x ^ s
    from this.comp_tendsto tendsto_norm_cocompact_atTop
  simp_rw [Asymptotics.IsBigO, Asymptotics.IsBigOWith]
```
**File:** Mathlib/Analysis/MellinTransform.lean (L86-89)
```text
/-- The Mellin transform of a function `f` (for a complex exponent `s`), defined as the integral of
`t ^ (s - 1) • f` over `Ioi 0`. -/
def mellin (f : ℝ → E) (s : ℂ) : E :=
  ∫ t : ℝ in Ioi 0, (t : ℂ) ^ (s - 1) • f t
```
**File:** Mathlib/Analysis/MellinTransform.lean (L92-94)
```text
the integral of `y ↦ x ^ -(σ + yi) • f (σ + yi)`. -/
def mellinInv (σ : ℝ) (f : ℂ → E) (x : ℝ) : E :=
  (1 / (2 * π)) • ∫ y : ℝ, (x : ℂ) ^ (-(σ + y * I)) • f (σ + y * I)
```
**File:** Mathlib/Analysis/MellinTransform.lean (L276-282)
```text
theorem mellinConvergent_of_isBigO_rpow [NormedSpace ℂ E] {a b : ℝ} {f : ℝ → E} {s : ℂ}
    (hfc : LocallyIntegrableOn f <| Ioi 0) (hf_top : f =O[atTop] (· ^ (-a)))
    (hs_top : s.re < a) (hf_bot : f =O[𝓝[>] 0] (· ^ (-b))) (hs_bot : b < s.re) :
    MellinConvergent f s := by
  rw [MellinConvergent,
    mellin_convergent_iff_norm Subset.rfl measurableSet_Ioi hfc.aestronglyMeasurable]
  exact mellin_convergent_of_isBigO_scalar hfc.norm hf_top.norm_left hs_top hf_bot.norm_left hs_bot
```
**File:** Mathlib/Analysis/MellinTransform.lean (L399-404)
```text
theorem mellin_differentiableAt_of_isBigO_rpow [NormedSpace ℂ E] {a b : ℝ}
    {f : ℝ → E} {s : ℂ} (hfc : LocallyIntegrableOn f <| Ioi 0)
    (hf_top : f =O[atTop] (· ^ (-a))) (hs_top : s.re < a)
    (hf_bot : f =O[𝓝[>] 0] (· ^ (-b))) (hs_bot : b < s.re) :
    DifferentiableAt ℂ (mellin f) s :=
  (mellin_hasDerivAt_of_isBigO_rpow hfc hf_top hs_top hf_bot hs_bot).2.differentiableAt
```
**File:** Mathlib/Analysis/MellinTransform.lean (L420-425)
```text
theorem mellin_differentiableAt_of_isBigO_rpow_exp [NormedSpace ℂ E] {a b : ℝ}
    (ha : 0 < a) {f : ℝ → E} {s : ℂ} (hfc : LocallyIntegrableOn f <| Ioi 0)
    (hf_top : f =O[atTop] fun t => exp (-a * t)) (hf_bot : f =O[𝓝[>] 0] (· ^ (-b)))
    (hs_bot : b < s.re) : DifferentiableAt ℂ (mellin f) s :=
  mellin_differentiableAt_of_isBigO_rpow hfc
    (hf_top.trans (isLittleO_exp_neg_mul_rpow_atTop ha _).isBigO) (lt_add_one _) hf_bot hs_bot
```
**File:** Mathlib/NumberTheory/EulerProduct/Basic.lean (L12-27)
```text
# Euler Products

The main result in this file is `EulerProduct.eulerProduct_hasProd`, which says that
if `f : ℕ → R` is norm-summable, where `R` is a complete normed commutative ring and `f` is
multiplicative on coprime arguments with `f 0 = 0`, then
`∏' p : Primes, ∑' e : ℕ, f (p^e)` converges to `∑' n, f n`.

`ArithmeticFunction.IsMultiplicative.eulerProduct_hasProd` is a version
for multiplicative arithmetic functions in the sense of
`ArithmeticFunction.IsMultiplicative`.

There is also a version `EulerProduct.eulerProduct_completely_multiplicative_hasProd`,
which states that `∏' p : Primes, (1 - f p)⁻¹` converges to `∑' n, f n`
when `f` is completely multiplicative with values in a complete normed field `F`
(implemented as `f : ℕ →*₀ F`).

```
**File:** Mathlib/NumberTheory/EulerProduct/Basic.lean (L79-84)
```text
/-- We relate a finite product over primes in `s` to an infinite sum over `s`-factored numbers. -/
lemma summable_and_hasSum_factoredNumbers_prod_filter_prime_tsum
    (hsum : ∀ {p : ℕ}, p.Prime → Summable (fun n : ℕ ↦ ‖f (p ^ n)‖)) (s : Finset ℕ) :
    Summable (fun m : factoredNumbers s ↦ ‖f m‖) ∧
      HasSum (fun m : factoredNumbers s ↦ f m)
        (∏ p ∈ s with p.Prime, ∑' n : ℕ, f (p ^ n)) := by
```
# DeepWiki Q&A with Code Context for Repository: AlexKontorovich/PrimeNumberTheoremAnd
## Q1
review the below formalization and help me leverage this repository as needed:
import Mathlib.Analysis.Distribution.SchwartzSpace
import Mathlib.Analysis.SpecialFunctions.Gamma.Deriv
import Mathlib.Analysis.SpecialFunctions.Complex.LogDeriv
import Mathlib.NumberTheory.LSeries.RiemannZeta
import Mathlib.NumberTheory.VonMangoldt
import Mathlib.MeasureTheory.Integral.Bochner
import Mathlib.Analysis.Complex.CauchyIntegral

/-!
# Weil's Explicit Formula for the Riemann Zeta Function

This file defines the structural components of Weil's Explicit Formula, relating
a sum over the nontrivial zeros of the Riemann Zeta function (Spectral Side)
to a sum over prime powers and analytical terms (Geometric Side).

## Main Definitions

* `WeilTestFunction`: A structure bundling the properties required for the test function `g`.
* `weilTransform`: The analytic transform `Φ(s) = ∫ g(x) e^{(s - 1/2)x} dx`.
* `spectralSide`: The sum `∑ Φ(ρ)` over nontrivial zeros.
* `geometricSide`: The sum over primes, archimedean terms, and boundary terms.

## Notation

We utilize the normalization where the critical line is `Re(s) = 1/2`.
-/

noncomputable section

open scoped BigOperators Real Complex
open Complex Real MeasureTheory SchwartzMap Topology Filter Set ArithmeticFunction vonMangoldt

namespace NumberTheory.WeilExplicit

/--
A Weil test function is a Schwartz function on ℝ satisfying specific symmetry
and decay properties allowing for the convergence of the Explicit Formula.
-/
class IsWeilTestFunction (g : SchwartzMap ℝ ℂ) : Prop where
  /-- The function must be even, corresponding to the symmetry of the functional equation. -/
  even : ∀ x, g x = g (-x)
  /-- Strong decay ensures the transform `Φ(s)` is entire or analytic in a wide strip. -/
  decay : ∃ (C ε : ℝ), 0 < ε ∧ ∀ x, ‖g x‖ ≤ C * Real.exp (- (1/2 + ε) * |x|)
  /-- Decay of the Fourier transform ensures absolute convergence of the prime sum. -/
  ft_decay : ∃ (C' ε' : ℝ), 0 < ε' ∧ ∀ ξ, ‖fourierTransformCLM ℂ g ξ‖ ≤ C' * Real.exp (- (1/2 + ε') * |ξ|)

variable (g : SchwartzMap ℝ ℂ) [IsWeilTestFunction g]

/-! ### The Analytic Transform -/

/--
The Weil transform `Φ(s)`.
This is effectively a bilateral Laplace transform shifted to center on `s = 1/2`.
`Φ(s) = ∫_{-∞}^{∞} g(x) e^{(s - 1/2)x} dx`
-/
def weilTransform (s : ℂ) : ℂ :=
  ∫ x : ℝ, g x * Complex.exp ((s - 0.5) * x)

lemma weilTransform_convergent (s : ℂ) (h_strip : |s.re - 0.5| < 0.5) :
    Integrable (fun x => g x * Complex.exp ((s - 0.5) * x)) := by
  -- Proof would use g.decay
  sorry

/-! ### The Spectral Side (Zeros) -/

/-- Predicate for nontrivial zeros of Riemann Zeta in the critical strip. -/
def IsZetaNontrivialZero (s : ℂ) : Prop :=
  riemannZeta s = 0 ∧ 0 < s.re ∧ s.re < 1

/--
The spectral side of the Explicit Formula: `∑_ρ Φ(ρ)`.
Note: Convergence of this sum depends on the density of zeros and decay of `Φ`.
-/
def spectralSide : ℂ :=
  ∑' (ρ : {s // IsZetaNontrivialZero s}), weilTransform g ρ

/-! ### The Geometric Side (Primes + Archimedean) -/

/--
The contribution from prime powers `p^k`.
`Sum_{n} Λ(n)/√n * (g(log n) + g(-log n))`
-/
def primeTerm : ℂ :=
  - ∑' n : ℕ, if n = 0 then 0 else
    ((vonMangoldt n : ℂ) / Real.sqrt n) *
      (g (Real.log n) + g (-Real.log n))

/--
The Archimedean contribution arising from the Gamma factor in the functional equation.
`1/2π ∫ g(x) Re(Ψ(1/4 + ix/2)) dx` ... (simplified form for definition)
-/
def archimedeanTerm : ℂ :=
  let digamma := fun s => (logDeriv Complex.Gamma) s
  let kernel := fun x => digamma (1/4 + (x / 2) * I) + digamma (1/4 - (x / 2) * I)
  (1 / (4 * π)) * ∫ x : ℝ, g x * kernel x - (g 0) * Real.log π

/--
The boundary terms arising from the poles of the L-function (at s=0 and s=1).
For Riemann Zeta, this is `Φ(0) + Φ(1)`.
-/
def boundaryTerm : ℂ :=
  weilTransform g 0 + weilTransform g 1

/--
The geometric side of the Explicit Formula.
-/
def geometricSide : ℂ :=
  boundaryTerm g + primeTerm g + archimedeanTerm g

/-! ### Main Theorem Statement -/

/--
**Weil's Explicit Formula for the Riemann Zeta Function**.

Given a suitable test function `g` (even, Schwartz, exp decay), the sum over the
nontrivial zeros of `ζ(s)` equals the sum over prime powers plus analytical terms.
-/
theorem weil_explicit_formula :
    spectralSide g = geometricSide g := by
  -- 1. Define contour integral I_T around rectangle [1-ε, ε] x [-T, T]
  -- 2. Apply Residue Theorem to Φ(s) * ξ'(s)/ξ(s)
  -- 3. Take limit as T → ∞
  -- 4. Match terms to definitions
  sorry

end NumberTheory.WeilExplicit
### Answer
# Answer

You're working on formalizing **Weil's Explicit Formula** for the Riemann zeta function in Lean 4, and you want to understand how to leverage the `AlexKontorovich/PrimeNumberTheoremAnd` repository to support this formalization.

<cite />

## Key Connections to the Repository

Your formalization shares several critical components with this repository's infrastructure:

### 1. **Zeta Function Analysis**

Your `IsZetaNontrivialZero` predicate and the spectral side sum directly relate to the zeta function bounds and zero-free region analysis in this repository. PrimeNumberTheoremAnd:654-690  The repository provides `riemannZetaLogDerivResidue`, which establishes that `ζ'/ζ(s) = -1/(s-1) + O(1)` near `s=1` — this is exactly the logarithmic derivative you'll need for your contour integration in `weil_explicit_formula`. <cite />

The repository also has zero-free region work in progress PrimeNumberTheoremAnd:3396-3450 , including `ZetaNoZerosOn1Line` and `ZetaNoZerosInBox`, which you'll need to control the spectral side sum's convergence. <cite />

### 2. **von Mangoldt Function**

Your `primeTerm` uses the von Mangoldt function `Λ(n)`, which is already extensively used throughout the repository. PrimeNumberTheoremAnd:54-63  The `LogDerivativeDirichlet` theorem establishes that `-ζ'/ζ(s) = ∑ Λ(n)/n^s` for `Re(s) > 1`, which is the Dirichlet series representation you'll need when matching your contour integral to the geometric side. <cite />

### 3. **Contour Integration Framework**

Your proof sketch mentions defining a contour integral around a rectangle — the repository has exactly this infrastructure. PrimeNumberTheoremAnd:28-42  The `RectangleIntegral` and related definitions (`HIntegral`, `VIntegral`, `UpperUIntegral`, `LowerUIntegral`) provide the geometric framework for contour deformation. <cite />

The Medium PNT approach uses a sophisticated 9-segment contour integration strategy PrimeNumberTheoremAnd:1085-1120  with integrals `I₁` through `I₉`, which demonstrates the pattern you'll follow for Weil's formula. <cite />

### 4. **Schwartz Functions and Fourier Analysis**

Your `IsWeilTestFunction` uses `SchwartzMap ℝ ℂ` with decay conditions. The repository's Wiener-Ikehara approach PrimeNumberTheoremAnd:1-60  extensively uses Schwartz functions and Fourier transforms, though the rate-limiting step is proving the Fourier transform is a bijection on the Schwartz class. <cite />

## Recommended Leverage Strategy

1. **Reuse Zeta Bounds**: Import and use `riemannZetaLogDerivResidue` and the zero-free region lemmas directly for your residue calculations and convergence proofs. <cite />

2. **Adapt Contour Integration**: Study the `RectangleIntegral` framework and the Medium PNT's contour deformation strategy PrimeNumberTheoremAnd:972-1038  as a template for your `weil_explicit_formula` proof. <cite />

3. **Use von Mangoldt Infrastructure**: The `LogDerivativeDirichlet` connection between `-ζ'/ζ` and `∑ Λ(n)/n^s` is already proven, saving you significant work on the prime term. <cite />

4. **Schwartz Function Properties**: While the repository doesn't have all Schwartz function properties you need (especially Fourier transform decay), it has the basic infrastructure and you can extend it for your specific decay requirements. <cite />

## Notes

The repository focuses on proving PNT via three approaches (Wiener-Ikehara, Complex Analysis, and Hadamard factorization) PrimeNumberTheoremAnd:48-63 , while Weil's Explicit Formula is a more general framework that encompasses these as special cases. Your formalization would be a natural extension of this work, particularly building on the Complex Analysis approach's contour integration machinery. <cite />

The Hoffstein-Lockhart file PrimeNumberTheoremAnd:1-69  is currently a stub but outlines zero-free region work that would strengthen your spectral side convergence arguments. <cite />

Wiki pages you might want to explore:
- [Three Approaches to PNT (AlexKontorovich/PrimeNumberTheoremAnd)](/wiki/AlexKontorovich/PrimeNumberTheoremAnd#1.2)
- [Zeta Function Analysis (AlexKontorovich/PrimeNumberTheoremAnd)](/wiki/AlexKontorovich/PrimeNumberTheoremAnd#2.1)
- [Medium PNT (AlexKontorovich/PrimeNumberTheoremAnd)](/wiki/AlexKontorovich/PrimeNumberTheoremAnd#3.2)
### Citations
**File:** PrimeNumberTheoremAnd/ZetaBounds.lean (L654-690)
```text
theorem riemannZetaLogDerivResidue :
    ∃ U ∈ 𝓝 1, BddAbove (norm ∘ (-(ζ' / ζ) - (fun s ↦ (s - 1)⁻¹)) '' (U \ {1})) := by
  obtain ⟨U,U_in_nhds, hU⟩ := riemannZetaResidue
  have hU' : BddAbove (norm ∘ (ζ - fun s ↦ 1 * (s - 1)⁻¹) '' (U \ {1})) := by
    simp only [Function.comp_apply, Pi.sub_apply, one_mul] at hU ⊢
    exact hU
  obtain ⟨V,V_in_nhds, V_is_open, hV⟩ := nonZeroOfBddAbove U_in_nhds one_ne_zero hU'
  let W := V ∩ interior U
  have hW : ∀ s ∈ W \ {1}, ζ s ≠ 0 := by
    intro s hs
    have s_in_V_diff : s ∈ V \ {1} := ⟨hs.1.1, hs.2⟩
    exact hV s s_in_V_diff
  have ζ_holc: HolomorphicOn ζ (W \ {1}) := by
    intro y hy
    simp only [mem_diff, mem_singleton_iff] at hy
    refine DifferentiableAt.differentiableWithinAt ?_
    apply differentiableAt_riemannZeta hy.2
  have W_in_nhds : W ∈ 𝓝 1 := by
    refine inter_mem V_in_nhds ?_
    exact interior_mem_nhds.mpr U_in_nhds
  have := logDerivResidue'' hW ζ_holc W_in_nhds one_ne_zero
  have HW : BddAbove (norm ∘ (ζ - fun s ↦ (s - 1)⁻¹) '' (W \ {1})) := by
    obtain ⟨c, hc⟩ := bddAbove_def.mp hU
    apply bddAbove_def.mpr
    use c
    rintro y ⟨x, x_in_W, fxy⟩
    apply hc
    exact ⟨x, ⟨interior_subset x_in_W.1.2, x_in_W.2⟩, fxy⟩
  simp only [one_mul] at this
  have aux: ∀ a, ‖-(deriv ζ a / ζ a) - (a - 1)⁻¹‖ = ‖(deriv ζ a / ζ a) + (a - 1)⁻¹‖ := by
    intro a
    calc ‖-(deriv ζ a / ζ a) - (a - 1)⁻¹‖
         = ‖-((deriv ζ a / ζ a) + (a - 1)⁻¹)‖ := by ring_nf
       _ = ‖(deriv ζ a / ζ a) + (a - 1)⁻¹‖ := by rw [norm_neg]
  simp only [Function.comp_apply, Pi.sub_apply] at hU
  simp only [Function.comp_apply, Pi.sub_apply, Pi.neg_apply, Pi.div_apply, aux]
  apply this HW
```
**File:** PrimeNumberTheoremAnd/ZetaBounds.lean (L3396-3450)
```text
lemma ZetaNoZerosOn1Line (t : ℝ) : ζ (1 + t * I) ≠ 0 := by
  refine riemannZeta_ne_zero_of_one_le_re ?_
  simp
/-%%
\begin{proof}\leanok
This fact is already proved in Stoll's work.
\end{proof}
%%-/

-- **Begin collaboration with the Alpha Proof team! 5/29/25**

lemma ZetaCont : ContinuousOn ζ (univ \ {1}) := by
  apply continuousOn_of_forall_continuousAt (fun x hx ↦ ?_)
--  simp only [mem_diff, mem_univ, mem_singleton_iff, true_and] at hx
  apply DifferentiableAt.continuousAt (𝕜 := ℂ)
  convert differentiableAt_riemannZeta ?_
  simp only [mem_diff, mem_univ, mem_singleton_iff, true_and] at hx
  exact hx

/-%%
Then, since $\zeta$ doesn't vanish on the 1-line, there is a $\sigma<1$ (depending on $T$), so that
the box $[\sigma,1] \times_{ℂ} [-T,T]$ is free of zeros of $\zeta$.
\begin{lemma}[ZetaNoZerosInBox]\label{ZetaNoZerosInBox}\lean{ZetaNoZerosInBox}\leanok
For any $T>0$, there is a constant $\sigma<1$ so that
$$
\zeta(\sigma'+it) \ne 0
$$
for all $|t| \leq T$ and $\sigma' \ge \sigma$.
\end{lemma}
%%-/

lemma ZetaNoZerosInBox (T : ℝ) :
    ∃ (σ : ℝ) (_ : σ < 1), ∀ (t : ℝ) (_ : |t| ≤ T)
    (σ' : ℝ) (_ : σ' ≥ σ), ζ (σ' + t * I) ≠ 0 := by
  by_contra h
  push_neg at h

  have hn (n : ℕ) := h (σ := 1 - 1 / (n + 1)) (sub_lt_self _ (by positivity))

  have : ∃ (tn : ℕ → ℝ) (σn : ℕ → ℝ), (∀ n, σn n ≤ 1) ∧
    (∀ n, (1 : ℝ) - 1 / (n + 1) ≤ σn n) ∧ (∀ n, |tn n| ≤ T) ∧
    (∀ n, ζ (σn n + tn n * I) = 0) := by
    choose t ht σ' hσ' hζ using hn
    refine ⟨t, σ', ?_, hσ', ht, hζ⟩
    intro n
    by_contra hσn
    push_neg at hσn
    have := riemannZeta_ne_zero_of_one_lt_re (s := σ' n + t n * I)
    simp only [add_re, ofReal_re, mul_re, I_re, mul_zero, ofReal_im, I_im, mul_one, sub_self,
      add_zero, ne_eq] at this
    exact this hσn (hζ n)

  choose t σ' hσ'_le hσ'_ge ht hζ using this

  have σTo1 : Filter.Tendsto σ' Filter.atTop (𝓝 1) := by
```
**File:** PrimeNumberTheoremAnd/MediumPNT.lean (L54-63)
```text
theorem LogDerivativeDirichlet (s : ℂ) (hs : 1 < s.re) :
    - deriv riemannZeta s / riemannZeta s = ∑' n, Λ n / (n : ℂ) ^ s := by
  rw [← ArithmeticFunction.LSeries_vonMangoldt_eq_deriv_riemannZeta_div hs]
  dsimp [LSeries, LSeries.term]
  nth_rewrite 2 [Summable.tsum_eq_add_tsum_ite (b := 0) ?_]
  · simp
  · have := ArithmeticFunction.LSeriesSummable_vonMangoldt hs
    dsimp [LSeriesSummable] at this
    convert this; rename ℕ => n
    by_cases h : n = 0 <;> simp [LSeries.term, h]
```
**File:** PrimeNumberTheoremAnd/MediumPNT.lean (L972-1038)
```text
noncomputable def I₁ (SmoothingF : ℝ → ℝ) (ε X T : ℝ) : ℂ :=
  (1 / (2 * π * I)) * (I * (∫ t : ℝ in Iic (-T),
      SmoothedChebyshevIntegrand SmoothingF ε X ((1 + (Real.log X)⁻¹) + t * I)))

/-%%
\begin{definition}[I₂]\label{I2}\lean{I₂}\leanok
$$
I_2(\nu, \epsilon, X, T, \sigma_1) := \frac{1}{2\pi i} \int_{\sigma_1}^{\sigma_0}
\left(
\frac{-\zeta'}\zeta(\sigma - i T)
\right)
  \mathcal M(\widetilde 1_\epsilon)(\sigma - i T)
X^{\sigma - i T} \ d\sigma
$$
\end{definition}
%%-/
noncomputable def I₂ (SmoothingF : ℝ → ℝ) (ε T X σ₁ : ℝ) : ℂ :=
  (1 / (2 * π * I)) * ((∫ σ in σ₁..(1 + (Real.log X)⁻¹),
    SmoothedChebyshevIntegrand SmoothingF ε X (σ - T * I)))

/-%%
\begin{definition}[I₃₇]\label{I37}\lean{I₃₇}\leanok
$$
I_{37}(\nu, \epsilon, X, T, \sigma_1) := \frac{1}{2\pi i} \int_{-T}^{T}
\left(
\frac{-\zeta'}\zeta(\sigma_1 + t i)
\right)
  \mathcal M(\widetilde 1_\epsilon)(\sigma_1 + t i)
X^{\sigma_1 + t i} \ i \ dt
$$
\end{definition}
%%-/
noncomputable def I₃₇ (SmoothingF : ℝ → ℝ) (ε T X σ₁ : ℝ) : ℂ :=
  (1 / (2 * π * I)) * (I * (∫ t in (-T)..T,
    SmoothedChebyshevIntegrand SmoothingF ε X (σ₁ + t * I)))

/-%%
\begin{definition}[I₈]\label{I8}\lean{I₈}\leanok
$$
I_8(\nu, \epsilon, X, T, \sigma_1) := \frac{1}{2\pi i} \int_{\sigma_1}^{\sigma_0}
\left(
\frac{-\zeta'}\zeta(\sigma + T i)
\right)
  \mathcal M(\widetilde 1_\epsilon)(\sigma + T i)
X^{\sigma + T i} \ d\sigma
$$
\end{definition}
%%-/
noncomputable def I₈ (SmoothingF : ℝ → ℝ) (ε T X σ₁ : ℝ) : ℂ :=
  (1 / (2 * π * I)) * ((∫ σ in σ₁..(1 + (Real.log X)⁻¹),
    SmoothedChebyshevIntegrand SmoothingF ε X (σ + T * I)))

/-%%
\begin{definition}[I₉]\label{I9}\lean{I₉}\leanok
$$
I_9(\nu, \epsilon, X, T) := \frac{1}{2\pi i} \int_{T}^{\infty}
\left(
\frac{-\zeta'}\zeta(\sigma_0 + t i)
\right)
  \mathcal M(\widetilde 1_\epsilon)(\sigma_0 + t i)
X^{\sigma_0 + t i} \ i \ dt
$$
\end{definition}
%%-/
noncomputable def I₉ (SmoothingF : ℝ → ℝ) (ε X T : ℝ) : ℂ :=
  (1 / (2 * π * I)) * (I * (∫ t : ℝ in Ici T,
      SmoothedChebyshevIntegrand SmoothingF ε X ((1 + (Real.log X)⁻¹) + t * I)))
```
**File:** PrimeNumberTheoremAnd/MediumPNT.lean (L1085-1120)
```text
noncomputable def I₄ (SmoothingF : ℝ → ℝ) (ε X σ₁ σ₂ : ℝ) : ℂ :=
  (1 / (2 * π * I)) * ((∫ σ in σ₂..σ₁,
    SmoothedChebyshevIntegrand SmoothingF ε X (σ - 3 * I)))

/-%%
\begin{definition}[I₆]\label{I6}\lean{I₆}\leanok
$$
I_6(\nu, \epsilon, X, \sigma_1, \sigma_2) := \frac{1}{2\pi i} \int_{\sigma_2}^{\sigma_1}
\left(
\frac{-\zeta'}\zeta(\sigma + 3 i)
\right)
  \mathcal M(\widetilde 1_\epsilon)(\sigma + 3 i)
X^{\sigma + 3 i} \ d\sigma
$$
\end{definition}
%%-/
noncomputable def I₆ (SmoothingF : ℝ → ℝ) (ε X σ₁ σ₂ : ℝ) : ℂ :=
  (1 / (2 * π * I)) * ((∫ σ in σ₂..σ₁,
    SmoothedChebyshevIntegrand SmoothingF ε X (σ + 3 * I)))

/-%%
\begin{definition}[I₅]\label{I5}\lean{I₅}\leanok
$$
I_5(\nu, \epsilon, X, \sigma_2) := \frac{1}{2\pi i} \int_{-3}^{3}
\left(
\frac{-\zeta'}\zeta(\sigma_2 + t i)
\right)
  \mathcal M(\widetilde 1_\epsilon)(\sigma_2 + t i)
X^{\sigma_2 + t i} \ i \ dt
$$
\end{definition}
%%-/
noncomputable def I₅ (SmoothingF : ℝ → ℝ) (ε X σ₂ : ℝ) : ℂ :=
  (1 / (2 * π * I)) * (I * (∫ t in (-3)..3,
    SmoothedChebyshevIntegrand SmoothingF ε X (σ₂ + t * I)))

```
**File:** PrimeNumberTheoremAnd/ResidueCalcOnRectangles.lean (L28-42)
```text
/-%%
\begin{definition}[RectangleIntegral]\label{RectangleIntegral}\lean{RectangleIntegral}\leanok
A RectangleIntegral of a function $f$ is one over a rectangle determined by $z$ and $w$ in $\C$.
We will sometimes denote it by $\int_{z}^{w} f$. (There is also a primed version, which is $1/(2\pi i)$ times the original.)
\end{definition}
%%-/
/-- A `RectangleIntegral` of a function `f` is one over a rectangle determined by
  `z` and `w` in `ℂ`. -/
noncomputable def RectangleIntegral (f : ℂ → E) (z w : ℂ) : E := HIntegral f z.re w.re z.im -
    HIntegral f z.re w.re w.im + VIntegral f w.re z.im w.im - VIntegral f z.re z.im w.im

/-- A `RectangleIntegral'` of a function `f` is one over a rectangle determined by
  `z` and `w` in `ℂ`, divided by `2 * π * I`. -/
noncomputable abbrev RectangleIntegral' (f : ℂ → E) (z w : ℂ) : E :=
    (1 / (2 * π * I)) • RectangleIntegral f z w
```
**File:** PrimeNumberTheoremAnd/Wiener.lean (L1-60)
```text
import Mathlib.Analysis.Fourier.FourierTransform
import Mathlib.Analysis.Fourier.FourierTransformDeriv
import Mathlib.NumberTheory.ArithmeticFunction
import Mathlib.NumberTheory.LSeries.PrimesInAP
import Mathlib.Topology.Algebra.Support
import Mathlib.Analysis.Calculus.ContDiff.Defs
import Mathlib.Geometry.Manifold.PartitionOfUnity
import Mathlib.Tactic.FunProp
import Mathlib.Analysis.Normed.Group.Tannery
import Mathlib.Algebra.Order.Field.Basic
import Mathlib.Order.Filter.ZeroAndBoundedAtFilter
import Mathlib.Analysis.Fourier.RiemannLebesgueLemma
import Mathlib.Analysis.SumIntegralComparisons
import Mathlib.Algebra.GroupWithZero.Units.Basic
import Mathlib.Analysis.Distribution.FourierSchwartz
import Mathlib.Topology.UniformSpace.UniformConvergence
import Mathlib.NumberTheory.MulChar.Lemmas

import PrimeNumberTheoremAnd.Fourier
import PrimeNumberTheoremAnd.BrunTitchmarsh
import PrimeNumberTheoremAnd.Mathlib.Analysis.Asymptotics.Asymptotics
import PrimeNumberTheoremAnd.SmoothExistence

set_option lang.lemmaCmd true

-- note: the opening of ArithmeticFunction introduces a notation σ that seems
-- impossible to hide, and hence parameters that are traditionally called σ will
-- have to be called σ' instead in this file.

open Real BigOperators ArithmeticFunction MeasureTheory Filter Set FourierTransform LSeries Asymptotics SchwartzMap
open Complex hiding log
open scoped Topology
open scoped ContDiff

variable {n : ℕ} {A a b c d u x y t σ' : ℝ} {ψ Ψ : ℝ → ℂ} {F G : ℂ → ℂ} {f : ℕ → ℂ} {𝕜 : Type} [RCLike 𝕜]

/-%%
The Fourier transform of an absolutely integrable function $\psi: \R \to \C$ is defined by the formula
$$ \hat \psi(u) := \int_\R e(-tu) \psi(t)\ dt$$
where $e(\theta) := e^{2\pi i \theta}$.

Let $f: \N \to \C$ be an arithmetic function such that $\sum_{n=1}^\infty \frac{|f(n)|}{n^\sigma} < \infty$ for all $\sigma>1$.  Then the Dirichlet series
$$ F(s) := \sum_{n=1}^\infty \frac{f(n)}{n^s}$$
is absolutely convergent for $\sigma>1$.
%%-/

noncomputable
def nterm (f : ℕ → ℂ) (σ' : ℝ) (n : ℕ) : ℝ := if n = 0 then 0 else ‖f n‖ / n ^ σ'

lemma nterm_eq_norm_term {f : ℕ → ℂ} : nterm f σ' n = ‖term f σ' n‖ := by
  by_cases h : n = 0 <;> simp [nterm, term, h]

theorem norm_term_eq_nterm_re (s : ℂ) :
    ‖term f s n‖ = nterm f (s.re) n := by
  simp only [nterm, term, apply_ite (‖·‖), norm_zero, norm_div]
  apply ite_congr rfl (fun _ ↦ rfl)
  intro h
  congr
  refine norm_natCast_cpow_of_pos (by omega) s

```
**File:** blueprint/blueprint.tex (L48-63)
```tex
There are (at least) three approaches to PNT that we may want to pursue simultaneously. The quickest, at this stage, is likely to
follow
 the ``Euler Products'' project by Michael Stoll, which has a proof of PNT missing only the Wiener-Ikehara Tauberian theorem.

The second develops some complex analysis (residue calculus on rectangles, argument principle, Mellin transforms), to pull contours and derive a PNT with an error term which is stronger than any power of log savings.

The third approach, which will be the most general of the three, is to: (1) develop the residue calculus et al, as above, (2) add the Hadamard factorization theorem, (3) use it to prove the zero-free region for zeta via Hoffstein-Lockhart+Goldfeld-Hoffstein-Liemann (which generalizes to higher degree L-functions), and (4) use this to prove the prime number theorem with exp-root-log savings.

A word about the expected ``rate-limiting-steps'' in each of the approaches.

(*) In approach (1), I think it will be the fact that the Fourier transform is a bijection on the Schwartz class. There is a recent PR (\url{https://github.com/leanprover-community/mathlib4/pull/9773}) with David Loeffler and Heather Macbeth making the first steps in that direction, just computing the (Frechet) derivative of the Fourier transform. One will need to iterate on that to get arbitrary derivatives, to conclude that the transform of a Schwartz function is Schwartz. Then to get the bijection, we need an inversion formula. We can derive Fourier inversion *from* Mellin inversion! So it seems that the most important thing to start is Perron's formula.

(*) In approach (2), there are two rate-limiting-steps, neither too serious (in my estimation). The first is how to handle meromorphic functions on rectangles. Perhaps in this project, it should not be done in any generality, but on a case by case basis. There are two simple poles whose residues need to be computed in the proof of the Perron formula, and one simple pole in the log-derivative of zeta, nothing too complicated, and maybe we shouldn't get bogged down in the general case. The other is the fact that the $\epsilon$-smoothed Chebyshev function differs from the unsmoothed by $\epsilon X$ (and not $\epsilon X \log X$, as follows from a trivial bound). This needs a Brun-Titchmarsh type theorem, perhaps can be done even more easily in this case with a Selberg sieve, on which there is  ... (truncated)

(*) In approach (3), it's obviously the Hadamard factorization, which needs quite a lot of nontrivial mathematics. (But after that, the math is not hard, on top of things in approach (2) -- and if we're getting the strong error term, we can afford to lose $\log X$ in the Chebyshev discussion above...).

```
**File:** PrimeNumberTheoremAnd/HoffsteinLockhart.lean (L1-69)
```text
import Mathlib.Analysis.Complex.CauchyIntegral
import Mathlib.NumberTheory.VonMangoldt
import Mathlib.NumberTheory.ArithmeticFunction
import Mathlib.NumberTheory.LSeries.RiemannZeta
import Mathlib.Analysis.Meromorphic.Basic

open Complex BigOperators Finset Nat Classical

open scoped ArithmeticFunction Interval

/-%%

In this file, we use the Hoffstein-Lockhart construction to prove a zero-free region for zeta.

Hoffstein-Lockhart + Goldfeld-Hoffstein-Liemann

Instead of the ``slick'' identity $3+4\cos\theta+\cos2\theta=2(\cos\theta+1)^2\ge0$, we use the following  more robust identity.
\begin{theorem}\label{thm:HLineq}
For any $p>0$ and $t\in\mathbb{R}$,
$$
3+p^{2it}+p^{-2it}+2p^{it}+2p^{-it} \ge 0.
$$
\end{theorem}
\begin{proof}
This follows immediately from the identity
$$
|1+p^{it}+p^{-it}|^2=1+p^{2it}+p^{-2it}+2p^{it}+2p^{-it}+2.
$$
\end{proof}
[Note: identities of this type will work in much greater generality, especially for
higher degree $L$-functions.]

This means that, for fixed $t$, we define the following alternate function.
\begin{definition}\label{FsigmaDef} For $\sigma>1$ and $t\in\mathbb{R}$, define
$$
F(\sigma) := \zeta^3(\sigma)\zeta^2(\sigma+it)\zeta^2(\sigma-it)\zeta(\sigma+2it)\zeta(\sigma-2it).
$$
\end{definition}
\begin{theorem}\label{FsigmaThm}
Then $F$ is real-valued, and
whence $F(\sigma)\ge1$ there.
\end{theorem}
\begin{proof}
\uses{thm:HLineq, FsigmaDef}
That
$\log F(\sigma)\ge0$ for $\sigma>1$ follows from
Theorem \ref{thm:HLineq}.
\end{proof}
[Note: I often prefer to avoid taking logs of functions that, even if real-valued, have to be justified as being such. Instead, I like to start with ``logF'' as a convergent
Dirichlet series, show that it is real-valued and non-negative, and then exponentiate...]

From this and Hadamard factorization, we deduce the following.
\begin{theorem}\label{thm:StrongZeroFree}
There is a constant $c>0$, so that $\zeta(s)$ does not vanish in
the region $\sigma>1-\frac{c}{\log t}$, and moreover,
$$
-\frac{\zeta'}{\zeta}(\sigma+it) \ll (\log t)^2
$$
there.
\end{theorem}
\begin{proof}
\uses{FsigmaThm}
Use Theorem \ref{FsigmaThm} and Hadamard factorization.
\end{proof}

This allows us to quantify precisely the relationship between $T$ and $\delta$ in
Theorem \ref{ZetaNoZerosInBox}....

```
# DeepWiki Q&A with Code Context for Repository: math-inc/strongpnt
## Q1
review the below formalization and help me leverage this repository as needed:
--
import Mathlib.Analysis.Distribution.SchwartzSpace
import Mathlib.Analysis.SpecialFunctions.Gamma.Deriv
import Mathlib.Analysis.SpecialFunctions.Complex.LogDeriv
import Mathlib.NumberTheory.LSeries.RiemannZeta
import Mathlib.NumberTheory.VonMangoldt
import Mathlib.MeasureTheory.Integral.Bochner
import Mathlib.Analysis.Complex.CauchyIntegral

/-!
# Weil's Explicit Formula for the Riemann Zeta Function

This file defines the structural components of Weil's Explicit Formula, relating
a sum over the nontrivial zeros of the Riemann Zeta function (Spectral Side)
to a sum over prime powers and analytical terms (Geometric Side).

## Main Definitions

* `WeilTestFunction`: A structure bundling the properties required for the test function `g`.
* `weilTransform`: The analytic transform `Φ(s) = ∫ g(x) e^{(s - 1/2)x} dx`.
* `spectralSide`: The sum `∑ Φ(ρ)` over nontrivial zeros.
* `geometricSide`: The sum over primes, archimedean terms, and boundary terms.

## Notation

We utilize the normalization where the critical line is `Re(s) = 1/2`.
-/

noncomputable section

open scoped BigOperators Real Complex
open Complex Real MeasureTheory SchwartzMap Topology Filter Set ArithmeticFunction vonMangoldt

namespace NumberTheory.WeilExplicit

/--
A Weil test function is a Schwartz function on ℝ satisfying specific symmetry
and decay properties allowing for the convergence of the Explicit Formula.
-/
class IsWeilTestFunction (g : SchwartzMap ℝ ℂ) : Prop where
  /-- The function must be even, corresponding to the symmetry of the functional equation. -/
  even : ∀ x, g x = g (-x)
  /-- Strong decay ensures the transform `Φ(s)` is entire or analytic in a wide strip. -/
  decay : ∃ (C ε : ℝ), 0 < ε ∧ ∀ x, ‖g x‖ ≤ C * Real.exp (- (1/2 + ε) * |x|)
  /-- Decay of the Fourier transform ensures absolute convergence of the prime sum. -/
  ft_decay : ∃ (C' ε' : ℝ), 0 < ε' ∧ ∀ ξ, ‖fourierTransformCLM ℂ g ξ‖ ≤ C' * Real.exp (- (1/2 + ε') * |ξ|)

variable (g : SchwartzMap ℝ ℂ) [IsWeilTestFunction g]

/-! ### The Analytic Transform -/

/--
The Weil transform `Φ(s)`.
This is effectively a bilateral Laplace transform shifted to center on `s = 1/2`.
`Φ(s) = ∫_{-∞}^{∞} g(x) e^{(s - 1/2)x} dx`
-/
def weilTransform (s : ℂ) : ℂ :=
  ∫ x : ℝ, g x * Complex.exp ((s - 0.5) * x)

lemma weilTransform_convergent (s : ℂ) (h_strip : |s.re - 0.5| < 0.5) :
    Integrable (fun x => g x * Complex.exp ((s - 0.5) * x)) := by
  -- Proof would use g.decay
  sorry

/-! ### The Spectral Side (Zeros) -/

/-- Predicate for nontrivial zeros of Riemann Zeta in the critical strip. -/
def IsZetaNontrivialZero (s : ℂ) : Prop :=
  riemannZeta s = 0 ∧ 0 < s.re ∧ s.re < 1

/--
The spectral side of the Explicit Formula: `∑_ρ Φ(ρ)`.
Note: Convergence of this sum depends on the density of zeros and decay of `Φ`.
-/
def spectralSide : ℂ :=
  ∑' (ρ : {s // IsZetaNontrivialZero s}), weilTransform g ρ

/-! ### The Geometric Side (Primes + Archimedean) -/

/--
The contribution from prime powers `p^k`.
`Sum_{n} Λ(n)/√n * (g(log n) + g(-log n))`
-/
def primeTerm : ℂ :=
  - ∑' n : ℕ, if n = 0 then 0 else
    ((vonMangoldt n : ℂ) / Real.sqrt n) *
      (g (Real.log n) + g (-Real.log n))

/--
The Archimedean contribution arising from the Gamma factor in the functional equation.
`1/2π ∫ g(x) Re(Ψ(1/4 + ix/2)) dx` ... (simplified form for definition)
-/
def archimedeanTerm : ℂ :=
  let digamma := fun s => (logDeriv Complex.Gamma) s
  let kernel := fun x => digamma (1/4 + (x / 2) * I) + digamma (1/4 - (x / 2) * I)
  (1 / (4 * π)) * ∫ x : ℝ, g x * kernel x - (g 0) * Real.log π

/--
The boundary terms arising from the poles of the L-function (at s=0 and s=1).
For Riemann Zeta, this is `Φ(0) + Φ(1)`.
-/
def boundaryTerm : ℂ :=
  weilTransform g 0 + weilTransform g 1

/--
The geometric side of the Explicit Formula.
-/
def geometricSide : ℂ :=
  boundaryTerm g + primeTerm g + archimedeanTerm g

/-! ### Main Theorem Statement -/

/--
**Weil's Explicit Formula for the Riemann Zeta Function**.

Given a suitable test function `g` (even, Schwartz, exp decay), the sum over the
nontrivial zeros of `ζ(s)` equals the sum over prime powers plus analytical terms.
-/
theorem weil_explicit_formula :
    spectralSide g = geometricSide g := by
  -- 1. Define contour integral I_T around rectangle [1-ε, ε] x [-T, T]
  -- 2. Apply Residue Theorem to Φ(s) * ξ'(s)/ξ(s)
  -- 3. Take limit as T → ∞
  -- 4. Match terms to definitions
  sorry

end NumberTheory.WeilExplicit
### Answer
# Answer

You're asking about formalizing **Weil's Explicit Formula** for the Riemann zeta function in Lean 4, and how to leverage the `math-inc/strongpnt` repository to support this work.<cite/>

## Overview of Your Formalization

Your code defines the key components of Weil's Explicit Formula:
- **Test function properties** (`IsWeilTestFunction`) requiring evenness and exponential decay<cite/>
- **Weil transform** `Φ(s) = ∫ g(x) e^{(s-1/2)x} dx`<cite/>
- **Spectral side**: sum over nontrivial zeros `∑_ρ Φ(ρ)`<cite/>
- **Geometric side**: prime terms, archimedean terms, and boundary terms<cite/>

## How to Leverage the strongpnt Repository

The `math-inc/strongpnt` repository provides several components directly relevant to your formalization:

### 1. **Von Mangoldt Function and Logarithmic Derivative**

Your `primeTerm` uses the von Mangoldt function `Λ(n)`.<cite/> The strongpnt repo establishes the fundamental identity:

$$-\frac{\zeta'(s)}{\zeta(s)} = \sum_{n=1}^\infty \frac{\Lambda(n)}{n^s}$$

This is proven in `LogDerivativeDirichlet` strongpnt:58-67  and used extensively via `neg_logDeriv_zeta_eq_vonMangoldt_sum` strongpnt:1834-1859 . This connection is essential for relating your spectral and geometric sides.

### 2. **Zero-Free Regions**

Your `IsZetaNontrivialZero` predicate defines zeros in the critical strip.<cite/> The strongpnt repo provides:

- **Zero set definitions**: `zeroZ` and `ZetaZerosNearPoint` strongpnt:4-6  define the set of zeta zeros
- **Finiteness results**: `ZetaZerosNearPoint_finite` strongpnt:8-60  proves zeros are finite in bounded regions, crucial for convergence of your `spectralSide`
- **Zero-free region theorem**: Establishes `ζ(σ + it) ≠ 0` for `σ ≥ 1 - c/log(|t|+2)`, which bounds where your nontrivial zeros can occur

### 3. **Holomorphicity of ζ'/ζ**

Your proof sketch mentions contour integration with `Φ(s) * ξ'(s)/ξ(s)`.<cite/> The strongpnt repo provides:

- `LogDerivZetaHoloOn'` strongpnt:166-173  proves holomorphicity of `ζ'/ζ` on sets where `ζ ≠ 0`
- `LogDerivZetaHolcSmallT'` and `LogDerivZetaHolcLargeT'` establish holomorphicity on specific rectangular domains needed for contour integration

### 4. **Contour Integration Framework**

The strongpnt repo implements sophisticated contour integration in `PNT5_Strong.lean` with rectangular paths decomposed into segments `I₁` through `I₉`.<cite/> This machinery could be adapted for your Weil formula proof, which also requires rectangular contour integration.

### 5. **Summability Results**

Your `spectralSide` and `primeTerm` require convergence proofs.<cite/> The repo provides:
- `vonMangoldt_LSeriesSummable` strongpnt:1891-1893  for L-series convergence
- `summable_of_support_singleton` strongpnt:1895-1913  for finite support functions

## Recommended Integration Strategy

1. **Import the strongpnt modules**: Add `import StrongPNT.PNT4_ZeroFreeRegion` and `import StrongPNT.ZetaZeroFree` to access zero-free region results<cite/>

2. **Use existing zero definitions**: Replace your `IsZetaNontrivialZero` with references to `zeroZ` and add constraints `0 < s.re ∧ s.re < 1`<cite/>

3. **Leverage von Mangoldt series**: Use `neg_logDeriv_zeta_eq_vonMangoldt_sum` to connect your spectral and geometric sides<cite/>

4. **Adapt contour integration**: Study the `SmoothedChebyshevIntegrand` approach strongpnt:86-89  as a template for your `Φ(s) * ξ'(s)/ξ(s)` contour integral

## Notes

The strongpnt repository focuses on the Prime Number Theorem rather than Weil's Explicit Formula directly, but the underlying analytic machinery (zero-free regions, logarithmic derivatives, contour integration) is highly relevant.<cite/> Your formalization would benefit from the explicit bounds and holomorphicity results already established there. The main gap is that strongpnt doesn't handle the Gamma factor functional equation terms (your `archimedeanTerm`), which you'll need to develop separately using Mathlib's gamma function derivatives.<cite/>

Wiki pages you might want to explore:
- [Zero-Free Region Theory (PNT4) (math-inc/strongpnt)](/wiki/math-inc/strongpnt#2.4)
- [Strong Prime Number Theorem (PNT5) (math-inc/strongpnt)](/wiki/math-inc/strongpnt#2.5)
- [Supporting Mathematical Results (math-inc/strongpnt)](/wiki/math-inc/strongpnt#2.6)
### Citations
**File:** StrongPNT/PNT5_Strong.lean (L58-67)
```text
theorem LogDerivativeDirichlet (s : ℂ) (hs : 1 < s.re) :
    - deriv riemannZeta s / riemannZeta s = ∑' n, Λ n / (n : ℂ) ^ s := by
  rw [← ArithmeticFunction.LSeries_vonMangoldt_eq_deriv_riemannZeta_div hs]
  dsimp [LSeries, LSeries.term]
  nth_rewrite 2 [Summable.tsum_eq_add_tsum_ite (b := 0) ?_]
  · simp
  · have := ArithmeticFunction.LSeriesSummable_vonMangoldt hs
    dsimp [LSeriesSummable] at this
    convert this; rename ℕ => n
    by_cases h : n = 0 <;> simp [LSeries.term, h]
```
**File:** StrongPNT/PNT5_Strong.lean (L86-89)
```text
noncomputable abbrev SmoothedChebyshevIntegrand (SmoothingF : ℝ → ℝ) (ε : ℝ) (X : ℝ) : ℂ → ℂ :=
  fun s ↦ (- deriv riemannZeta s) / riemannZeta s *
    𝓜 ((Smooth1 SmoothingF ε) ·) s * (X : ℂ) ^ s

```
**File:** StrongPNT/PNT4_ZeroFreeRegion.lean (L4-6)
```text
def zeroZ : Set ℂ := {s : ℂ | riemannZeta s = 0}

def ZetaZerosNearPoint (t : ℝ) : Set ℂ := { ρ : ℂ | ρ ∈ zeroZ ∧ ‖ρ - ((3/2 : ℂ) + t * Complex.I)‖ ≤ (5/6 : ℝ) }
```
**File:** StrongPNT/PNT4_ZeroFreeRegion.lean (L8-60)
```text
lemma ZetaZerosNearPoint_finite (t : ℝ) : Set.Finite (ZetaZerosNearPoint t) := by
  -- Center and radius of the disk
  let c : ℂ := (3/2 : ℂ) + t * Complex.I
  let R : ℝ := (5/6 : ℝ)
  have hRpos : 0 < R := by norm_num
  -- Define H(s) = (s - 1) * ζ(s) with the removable singularity at s = 1 filled in by setting H(1) = 1.
  -- This H is differentiable (entire). We'll use g(z) = H (z + c).
  let H : ℂ → ℂ := Function.update (fun s : ℂ => (s - 1) * riemannZeta s) 1 1
  have hH_diff : Differentiable ℂ H := by
    -- Show differentiability everywhere by splitting on s = 1.
    intro s
    rcases eq_or_ne s 1 with rfl | hs
    · -- differentiable at 1 via removable singularity: differentiable on punctured nhds + continuity
      refine (Complex.analyticAt_of_differentiable_on_punctured_nhds_of_continuousAt ?_ ?_).differentiableAt
      · -- differentiable on punctured nhds around 1
        filter_upwards [self_mem_nhdsWithin] with t ht
        -- On t ≠ 1, H agrees with (t-1)*ζ t; prove differentiableAt via congr
        have hdiff : DifferentiableAt ℂ (fun u : ℂ => (u - 1) * riemannZeta u) t := by
          have h1 : DifferentiableAt ℂ (fun u : ℂ => u - 1) t :=
            (differentiableAt_id.sub_const 1)
          have h2 : DifferentiableAt ℂ riemannZeta t :=
            (differentiableAt_riemannZeta ht)
          exact h1.mul h2
        apply DifferentiableAt.congr_of_eventuallyEq hdiff
        filter_upwards [eventually_ne_nhds ht] with u hu using by
          simp [H, Function.update_of_ne hu]
      · -- continuity of H at 1 from the known residue/limit lemma
        simpa [H, continuousAt_update_same] using riemannZeta_residue_one
    · -- s ≠ 1: H agrees with (s-1)ζ(s), hence differentiable
      have hdiff : DifferentiableAt ℂ (fun u : ℂ => (u - 1) * riemannZeta u) s := by
        have h1 : DifferentiableAt ℂ (fun u : ℂ => u - 1) s :=
          (differentiableAt_id.sub_const 1)
        have h2 : DifferentiableAt ℂ riemannZeta s :=
          (differentiableAt_riemannZeta hs)
        exact h1.mul h2
      apply DifferentiableAt.congr_of_eventuallyEq hdiff
      filter_upwards [eventually_ne_nhds hs] with u hu using by
        simp [H, Function.update_of_ne hu]

  -- Define a translated function g so that zeros of ζ in the closed ball around c
  -- correspond to zeros of g in the closed ball around 0. If the pole at 1 lies
  -- in the ball, multiply by (z + c - 1) to remove it.
  by_cases hPoleIn : ‖1 - c‖ ≤ R
  · -- Pole at 1 is inside: use g z = (z + c - 1) * ζ(z + c)
    let g : ℂ → ℂ := fun z => H (z + c)
    -- Witness that g is not identically zero: evaluate at z = 0. Here c.re = 3/2 > 1, so ζ(c) ≠ 0.
    have hzeta_c_ne : riemannZeta c ≠ 0 := by
      -- Use non-vanishing for Re > 1
      have : c.re = (3/2 : ℝ) := by
        simp [c, Complex.add_re, Complex.mul_re, Complex.I_re]
      have hgt : c.re > 1 := by simpa [this] using (by norm_num : (3:ℝ)/2 > 1)
      -- riemannZeta ≠ 0 for Re ≥ 1, in particular for Re > 1
      exact riemannZeta_ne_zero_of_one_le_re (by
```
**File:** StrongPNT/PNT4_ZeroFreeRegion.lean (L1834-1859)
```text
lemma neg_logDeriv_zeta_eq_vonMangoldt_sum (s : ℂ) (hs : 1 < s.re) : -(deriv riemannZeta s / riemannZeta s) = ∑' (n : ℕ), (ArithmeticFunction.vonMangoldt n : ℂ) * (n : ℂ) ^ (-s) := by
  -- Use the key lemma relating L-series of vonMangoldt to the logarithmic derivative
  have h1 := ArithmeticFunction.LSeries_vonMangoldt_eq_deriv_riemannZeta_div hs
  -- h1: LSeries (fun n => ↑(ArithmeticFunction.vonMangoldt n)) s = -deriv riemannZeta s / riemannZeta s
  -- From this we get: -deriv riemannZeta s / riemannZeta s = LSeries (fun n => ↑(ArithmeticFunction.vonMangoldt n)) s
  have h2 : -deriv riemannZeta s / riemannZeta s = LSeries (fun n => ↑(ArithmeticFunction.vonMangoldt n)) s := h1.symm
  -- Now deal with the parentheses: -(deriv riemannZeta s / riemannZeta s) = -deriv riemannZeta s / riemannZeta s
  have h3 : -(deriv riemannZeta s / riemannZeta s) = -deriv riemannZeta s / riemannZeta s := by ring
  rw [h3, h2]
  -- Now goal is: LSeries (fun n => ↑(ArithmeticFunction.vonMangoldt n)) s = ∑' (n : ℕ), ↑(ArithmeticFunction.vonMangoldt n) * ↑n ^ (-s)
  -- This follows from the definition of LSeries as a tsum of terms
  rw [LSeries]
  congr 1
  ext n
  rw [LSeries.term_def]
  split_ifs with h_zero
  · -- Case n = 0: both sides are 0
    simp [h_zero]
  · -- Case n ≠ 0: show vonMangoldt n / n ^ s = vonMangoldt n * n ^ (-s)
    rw [div_eq_mul_inv, Complex.cpow_neg]

lemma zeta1zetaseries {s : ℂ} (hs : 1 < s.re) :
-logDerivZeta s = ∑' (n : ℕ), (ArithmeticFunction.vonMangoldt n : ℂ) * (n : ℂ) ^ (-s) := by
  unfold logDerivZeta
  -- Goal: -(deriv riemannZeta s / riemannZeta s) = ∑' (n : ℕ), (ArithmeticFunction.vonMangoldt n : ℂ) * (n : ℂ) ^ (-s)
  exact neg_logDeriv_zeta_eq_vonMangoldt_sum s hs
```
**File:** StrongPNT/PNT4_ZeroFreeRegion.lean (L1891-1893)
```text
lemma vonMangoldt_LSeriesSummable (s : ℂ) (hs : 1 < s.re) : LSeriesSummable (fun n => (ArithmeticFunction.vonMangoldt n : ℂ)) s := by
  -- Use the existing theorem ArithmeticFunction.LSeriesSummable_vonMangoldt
  exact ArithmeticFunction.LSeriesSummable_vonMangoldt hs
```
**File:** StrongPNT/PNT4_ZeroFreeRegion.lean (L1895-1913)
```text
lemma summable_of_support_singleton {α : Type*} [SeminormedAddCommGroup α] (f : ℕ → α) (n₀ : ℕ) (h : ∀ n : ℕ, n ≠ n₀ → f n = 0) : Summable f := by
  -- Show that f has finite support, so it's summable
  have h_finite_support : Set.Finite (Function.support f) := by
    -- The support is contained in {n₀}
    have h_subset : Function.support f ⊆ {n₀} := by
      intro n hn
      -- hn : n ∈ Function.support f means f n ≠ 0
      -- We need to show n ∈ {n₀}, i.e., n = n₀
      by_contra h_ne
      -- If n ≠ n₀, then by hypothesis h, f n = 0
      have h_zero : f n = 0 := h n h_ne
      -- But this contradicts f n ≠ 0
      simp [Function.mem_support] at hn
      exact hn h_zero
    -- Since support ⊆ {n₀} and {n₀} is finite, support is finite
    exact Set.Finite.subset (Set.finite_singleton n₀) h_subset

  -- Use the fact that functions with finite support are summable
  exact summable_of_finite_support h_finite_support
```
**File:** StrongPNT/ZetaZeroFree.lean (L166-173)
```text
lemma LogDerivZetaHoloOn' {S : Set ℂ} (s_ne_one : 1 ∉ S)
    (nonzero : ∀ s ∈ S, ζ s ≠ 0) :
    HolomorphicOn (fun s ↦ ζ' s / ζ s) S := by
  apply DifferentiableOn.div _ _ nonzero <;> intro s hs <;> apply DifferentiableAt.differentiableWithinAt
  · apply differentiableAt_deriv_riemannZeta
    exact ne_of_mem_of_not_mem hs s_ne_one
  · apply differentiableAt_riemannZeta
    exact ne_of_mem_of_not_mem hs s_ne_one
```
Based on my search of the Mathlib4 codebase, I need to inform you that **Weil's Explicit Formula is not currently implemented in Mathlib4**, and the **residue theorem is also not yet available**. However, there are several foundational APIs that can be leveraged for parts of your project:

## For Schwartz Functions and Exponential Decay

**Schwartz Space Foundation:**
- The `SchwartzMap` structure defines functions with rapid decay of all derivatives [1](#4-0)

**Integrability of Schwartz Functions:**
- `SchwartzMap.integrable` proves that Schwartz functions are integrable [2](#4-1)

**Asymptotic Decay Bounds:**
- `SchwartzMap.isBigO_cocompact_rpow` provides O-bounds for polynomial decay at infinity [3](#4-2)

**Integration of Schwartz Functions:**
- `SchwartzMap.integralCLM` provides integration as a continuous linear map [4](#4-3)

## For Logarithmic Derivatives and Prime Power Sums

**Logarithmic Derivative Definition:**
- `logDeriv` is defined as `deriv f / f` [5](#4-4)

**Von Mangoldt Function (Prime Powers):**
- The von Mangoldt function `Λ` returns `log p` for prime powers `p^k` [6](#4-5)

**Key Result: Zeta's Logarithmic Derivative Equals L-series of von Mangoldt:**
- This is the fundamental connection between the zeta function and prime powers [7](#4-6)

## For Complex Analysis (Limited Contour Integration Support)

**Circle Integrals:**
- `circleIntegral` notation for integrals around circles [8](#4-7)

**Cauchy Integral Formulas:**
- Available but not the full residue theorem [9](#4-8)

**Mellin Transform:**
- Useful for functional equations and transforming integrals [10](#4-9)

**Riemann Zeta Functional Equation:**
- The completed zeta satisfies `Λ(1-s) = Λ(s)` [11](#4-10)

## Notes

**Important Limitations:**
1. The **residue theorem is not yet implemented** in Mathlib4, though it's listed as planned future work
2. **Weil's Explicit Formula is not formalized** in the current codebase
3. You would need to build the residue theorem and explicit formula proofs from the available Cauchy integral foundations

For proving integral convergence of `g(x) * exp((s-0.5)*x)` with a Schwartz function `g`, you can combine the Schwartz integrability results with exponential integrability lemmas, but you may need to add custom convergence proofs for the specific exponential weight function.

### Citations

**File:** Mathlib/Analysis/Distribution/SchwartzSpace.lean (L77-83)
```text
structure SchwartzMap where
  /-- The underlying function.

  Do NOT use directly. Use the coercion instead. -/
  toFun : E → F
  smooth' : ContDiff ℝ ∞ toFun
  decay' : ∀ k n : ℕ, ∃ C : ℝ, ∀ x, ‖x‖ ^ k * ‖iteratedFDeriv ℝ n toFun x‖ ≤ C
```

**File:** Mathlib/Analysis/Distribution/SchwartzSpace.lean (L142-154)
```text
theorem isBigO_cocompact_rpow [ProperSpace E] (s : ℝ) :
    f =O[cocompact E] fun x => ‖x‖ ^ s := by
  let k := ⌈-s⌉₊
  have hk : -(k : ℝ) ≤ s := neg_le.mp (Nat.le_ceil (-s))
  refine (isBigO_cocompact_zpow_neg_nat f k).trans ?_
  suffices (fun x : ℝ => x ^ (-k : ℤ)) =O[atTop] fun x : ℝ => x ^ s
    from this.comp_tendsto tendsto_norm_cocompact_atTop
  simp_rw [Asymptotics.IsBigO, Asymptotics.IsBigOWith]
  refine ⟨1, (Filter.eventually_ge_atTop 1).mono fun x hx => ?_⟩
  rw [one_mul, Real.norm_of_nonneg (Real.rpow_nonneg (zero_le_one.trans hx) _),
    Real.norm_of_nonneg (zpow_nonneg (zero_le_one.trans hx) _), ← Real.rpow_intCast, Int.cast_neg,
    Int.cast_natCast]
  exact Real.rpow_le_rpow_of_exponent_le hx hk
```

**File:** Mathlib/Analysis/Distribution/SchwartzSpace.lean (L942-944)
```text
lemma integrable (f : 𝓢(D, V)) : Integrable f μ :=
  (f.integrable_pow_mul μ 0).mono f.continuous.aestronglyMeasurable
    (Eventually.of_forall (fun _ ↦ by simp))
```

**File:** Mathlib/Analysis/Distribution/SchwartzSpace.lean (L947-964)
```text
/-- The integral as a continuous linear map from Schwartz space to the codomain. -/
def integralCLM : 𝓢(D, V) →L[𝕜] V := by
  refine mkCLMtoNormedSpace (∫ x, · x ∂μ)
    (fun f g ↦ integral_add f.integrable g.integrable) (integral_smul · ·) ?_
  rcases hμ.exists_integrable with ⟨n, h⟩
  let m := (n, 0)
  use Finset.Iic m, 2 ^ n * ∫ x : D, (1 + ‖x‖) ^ (- (n : ℝ)) ∂μ
  refine ⟨by positivity, fun f ↦ (norm_integral_le_integral_norm f).trans ?_⟩
  have h' : ∀ x, ‖f x‖ ≤ (1 + ‖x‖) ^ (-(n : ℝ)) *
      (2 ^ n * ((Finset.Iic m).sup (fun m' => SchwartzMap.seminorm 𝕜 m'.1 m'.2) f)) := by
    intro x
    rw [rpow_neg (by positivity), ← div_eq_inv_mul, le_div_iff₀' (by positivity), rpow_natCast]
    simpa using one_add_le_sup_seminorm_apply (m := m) (k := n) (n := 0) le_rfl le_rfl f x
  apply (integral_mono (by simpa using f.integrable_pow_mul μ 0) _ h').trans
  · unfold schwartzSeminormFamily
    rw [integral_mul_const, ← mul_assoc, mul_comm (2 ^ n)]
  apply h.mul_const

```

**File:** Mathlib/Analysis/Calculus/LogDeriv.lean (L26-29)
```text
/-- The logarithmic derivative of a function defined as `deriv f /f`. Note that it will be zero
at `x` if `f` is not DifferentiableAt `x`. -/
def logDeriv (f : 𝕜 → 𝕜') :=
  deriv f / f
```

**File:** Mathlib/NumberTheory/VonMangoldt.lean (L52-63)
```text
/--
The `vonMangoldt` function is the function on natural numbers that returns `log p` if the input can
be expressed as `p^k` for a prime `p`.
In the case when `n` is a prime power, `Nat.minFac` will give the appropriate prime, as it is the
smallest prime factor.

In the `ArithmeticFunction` locale, we have the notation `Λ` for this function.
This is also available in the `ArithmeticFunction.vonMangoldt` locale, allowing for selective
access to the notation.
-/
noncomputable def vonMangoldt : ArithmeticFunction ℝ :=
  ⟨fun n => if IsPrimePow n then Real.log (minFac n) else 0, if_neg not_isPrimePow_zero⟩
```

**File:** Mathlib/NumberTheory/LSeries/Dirichlet.lean (L404-412)
```text
/-- The L-series of the von Mangoldt function `Λ` equals the negative logarithmic derivative
of the Riemann zeta function on its domain of convergence `re s > 1`. -/
lemma LSeries_vonMangoldt_eq_deriv_riemannZeta_div {s : ℂ} (hs : 1 < s.re) :
    L ↗Λ s = - deriv riemannZeta s / riemannZeta s := by
  suffices deriv (L 1) s = deriv riemannZeta s by
    rw [LSeries_vonMangoldt_eq hs, ← LSeries_one_eq_riemannZeta hs, this]
  refine Filter.EventuallyEq.deriv_eq <| Filter.eventuallyEq_iff_exists_mem.mpr ?_
  exact ⟨{z | 1 < z.re}, (isOpen_lt continuous_const continuous_re).mem_nhds hs,
    fun _ ↦ LSeries_one_eq_riemannZeta⟩
```

**File:** Mathlib/MeasureTheory/Integral/CircleIntegral.lean (L17-24)
```text

## Main definitions

* `CircleIntegrable f c R`: a function `f : ℂ → E` is integrable on the circle with center `c` and
  radius `R` if `f ∘ circleMap c R` is integrable on `[0, 2π]`;

* `circleIntegral f c R`: the integral $\oint_{|z-c|=|R|} f(z)\,dz$, defined as
  $\int_{0}^{2π}(c + Re^{θ i})' f(c+Re^{θ i})\,dθ$;
```

**File:** Mathlib/Analysis/Complex/CauchyIntegral.lean (L54-60)
```text

* `Complex.circleIntegral_sub_inv_smul_of_differentiable_on_off_countable`,
  `Complex.two_pi_I_inv_smul_circleIntegral_sub_inv_smul_of_differentiable_on_off_countable`
  **Cauchy integral formula**: if `f : ℂ → E` is continuous on a closed disc of radius `R` and is
  complex differentiable on the corresponding open disc, then for any `w` in the corresponding open
  disc the integral of `(z - w)⁻¹ • f z` over the boundary of the disc is equal to `2πif(w)`.
  Two versions of the lemma put the multiplier `2πi` at the different sides of the equality.
```

**File:** Mathlib/Analysis/MellinTransform.lean (L10-23)
```text
/-! # The Mellin transform

We define the Mellin transform of a locally integrable function on `Ioi 0`, and show it is
differentiable in a suitable vertical strip.

## Main statements

- `mellin` : the Mellin transform `∫ (t : ℝ) in Ioi 0, t ^ (s - 1) • f t`,
  where `s` is a complex number.
- `HasMellin`: shorthand asserting that the Mellin transform exists and has a given value
  (analogous to `HasSum`).
- `mellin_differentiableAt_of_isBigO_rpow` : if `f` is `O(x ^ (-a))` at infinity, and
  `O(x ^ (-b))` at 0, then `mellin f` is holomorphic on the domain `b < re s < a`.

```

**File:** Mathlib/NumberTheory/LSeries/RiemannZeta.lean (L94-98)
```text
`Λ₀(1 - s) = Λ₀ s`. -/
theorem completedRiemannZeta₀_one_sub (s : ℂ) :
    completedRiemannZeta₀ (1 - s) = completedRiemannZeta₀ s := by
  rw [← completedHurwitzZetaEven₀_zero, ← completedCosZeta₀_zero, completedHurwitzZetaEven₀_one_sub]

```


Based on my thorough search of the repository, I must provide a clear answer: **The `AlexKontorovich/PrimeNumberTheoremAnd` repository does not contain implementations of Weil's Explicit Formula or the specific terms you mentioned.**

Specifically:

1. **`weilTransform_convergent` lemma**: This lemma does not exist in the repository. There are no references to "Weil" anywhere in the codebase.

2. **`spectralSide` sum convergence**: This identifier does not appear in the repository. There are no references to "spectral" in the codebase.

3. **`primeTerm` derivation from contour integrals**: While the repository does contain contour integration machinery, there is no specific `primeTerm` identifier or function in the context of Weil's Explicit Formula.

4. **`archimedeanTerm` calculation**: This term does not exist in the repository. There are no references to "archimedean" anywhere in the codebase.

5. **`boundaryTerm`**: While the word "boundary" appears in the context of zeta function bounds, it is not related to Weil's Explicit Formula.

## What the Repository Actually Contains

The repository is focused on **formalizing the Prime Number Theorem** (not Weil's Explicit Formula) using three different approaches:

1. The Wiener-Ikehara Tauberian theorem approach
2. The Medium PNT via complex analysis and contour integration
3. The Strong PNT via Hadamard factorization

The repository does contain related mathematical machinery that could potentially be used as building blocks for Weil's Explicit Formula, including:
- Mellin transforms and convergence lemmas
- Perron formula
- Contour integration on rectangles
- Zeta function analysis

However, Weil's Explicit Formula itself has not been formalized in this repository.

## Notes

If you are looking to implement Weil's Explicit Formula or are working on a different repository that contains these specific lemmas, please verify the repository name. The `AlexKontorovich/PrimeNumberTheoremAnd` repository is specifically dedicated to the Prime Number Theorem and related results such as the Prime Number Theorem in Arithmetic Progressions, as stated in its README.
