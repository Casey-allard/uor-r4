# Mathlib PR Plan: Binet Kernel Infrastructure

This document outlines the proposed Mathlib PRs to support the Binet kernel and Binet's formula for the Gamma function.

## Current Status

The file `BinetKernel.lean` is complete except for 3 theorems:
- `tendsto_Ktilde_zero` - limit theorem
- `integrable_Ktilde_exp` - real integrability
- `integrable_Ktilde_exp_complex` - complex integrability

## Proposed PR Series

### PR 1: General Monotonicity Lemmas (Ready)

**File location**: `Mathlib.Analysis.Calculus.MeanValue`

**Lemmas**:
```lean
/-- If a function has nonnegative derivative on `[0, ∞)`, it is monotone there. -/
lemma monotoneOn_of_deriv_nonneg_Ici {f : ℝ → ℝ}
    (hf : DifferentiableOn ℝ f (Set.Ici 0))
    (hderiv : ∀ x ∈ Set.Ici 0, 0 ≤ deriv f x) :
    MonotoneOn f (Set.Ici 0)

/-- If `deriv f ≥ 0` on `[0, ∞)` and `f 0 = 0`, then `f x ≥ 0` for `x ≥ 0`. -/
lemma nonneg_of_deriv_nonneg_Ici {f : ℝ → ℝ}
    (hf : DifferentiableOn ℝ f (Set.Ici 0))
    (hderiv : ∀ x ∈ Set.Ici 0, 0 ≤ deriv f x) (h0 : f 0 = 0) :
    ∀ {x}, 0 ≤ x → 0 ≤ f x
```

**Status**: ✅ Complete - can be PRed immediately

---

### PR 2: Taylor Bounds for exp (Already in Mathlib)

**Existing lemmas**:
- `Real.add_one_le_exp` — `x + 1 ≤ exp x`
- `Real.quadratic_le_exp_of_nonneg` — `1 + x + x²/2 ≤ exp x`
- `Real.sum_le_exp_of_nonneg` — general Taylor bound `Σ x^k/k! ≤ exp x`

**No PR needed** — already in `Mathlib.Analysis.Complex.Exponential`

---

### PR 3: Binet Kernel Definitions and Basic Properties (Ready)

**File location**: `Mathlib.Analysis.SpecialFunctions.Gamma.BinetKernel`

**Definitions**:
```lean
/-- The Binet kernel: K(t) = 1/(e^t - 1) - 1/t + 1/2 -/
noncomputable def K (t : ℝ) : ℝ

/-- The normalized Binet kernel: K̃(t) = K(t)/t -/
noncomputable def Ktilde (t : ℝ) : ℝ
```

**Main lemmas**:
- `K_pos`, `Ktilde_pos` — explicit formulas for t > 0
- `Ktilde_zero` — K̃(0) = 1/12 (by continuous extension)
- `K_eq_alt'` — alternative formula K(t) = f(t)/(2t(e^t-1))
- `K_nonneg`, `Ktilde_nonneg` — non-negativity
- `f_nonneg` — non-negativity of numerator function
- `Ktilde_le` — upper bound K̃(t) ≤ 1/12

**Status**: ✅ Complete - can be PRed

---

### PR 4: Limit Theorem (Needs Work)

**Theorem**:
```lean
/-- K̃(t) → 1/12 as t → 0⁺ -/
theorem tendsto_Ktilde_zero :
    Tendsto Ktilde (𝓝[>] 0) (𝓝 (1/12 : ℝ))
```

**Proof strategy**:
1. Use Taylor expansion: `1/(e^t - 1) = 1/t - 1/2 + t/12 - t³/720 + ...`
2. Hence `K(t) = t/12 - t³/720 + O(t⁵)`
3. So `K̃(t) = 1/12 - t²/720 + O(t⁴) → 1/12`

**Dependencies**: May need additional Taylor/limit lemmas from Mathlib

**Status**: ⏳ Requires proof

---

### PR 5: Integrability (Needs Work)

**Theorems**:
```lean
/-- K̃(t) * e^{-tx} is integrable on (0, ∞) for x > 0 -/
theorem integrable_Ktilde_exp {x : ℝ} (hx : 0 < x) :
    Integrable (fun t => Ktilde t * Real.exp (-t * x))
      (MeasureTheory.Measure.restrict MeasureTheory.volume (Set.Ioi 0))

/-- Complex version for Re(z) > 0 -/
theorem integrable_Ktilde_exp_complex {z : ℂ} (hz : 0 < z.re) :
    MeasureTheory.Integrable
      (fun t : ℝ => (Ktilde t : ℂ) * Complex.exp (-t * z))
      (MeasureTheory.Measure.restrict MeasureTheory.volume (Set.Ioi 0))
```

**Proof strategy**:
1. K̃(t) is bounded by 1/12 (proven)
2. e^{-tx} is integrable on (0, ∞) for x > 0
3. Product of bounded and integrable is integrable

**Dependencies**: Standard dominated convergence / comparison theorems

**Status**: ⏳ Requires proof

---

## Auxiliary Lemmas (Internal, for Ktilde_le proof)

These are proven but could be generalized/PRed separately:

```lean
-- Auxiliary function for the Ktilde ≤ 1/12 bound
noncomputable def g_aux (t : ℝ) : ℝ := (t^2 - 6*t + 12) * exp t - (t^2 + 6*t + 12)
noncomputable def g_aux' (t : ℝ) : ℝ := (t^2 - 4*t + 6) * exp t - (2*t + 6)
noncomputable def g_aux'' (t : ℝ) : ℝ := (t^2 - 2*t + 2) * exp t - 2
noncomputable def g_aux''' (t : ℝ) : ℝ := t^2 * exp t

-- Derivative chain
lemma hasDerivAt_g_aux'' : HasDerivAt g_aux'' (g_aux''' t) t
lemma hasDerivAt_g_aux' : HasDerivAt g_aux' (g_aux'' t) t
lemma hasDerivAt_g_aux : HasDerivAt g_aux (g_aux' t) t

-- Non-negativity chain (each uses the previous + nonneg_of_deriv_nonneg_Ici)
lemma g_aux'''_nonneg : 0 ≤ g_aux''' t  -- base case: t²e^t ≥ 0
lemma g_aux''_nonneg : 0 ≤ g_aux'' t
lemma g_aux'_nonneg : 0 ≤ g_aux' t
lemma g_aux_nonneg : 0 ≤ g_aux t  -- used in Ktilde_le proof
```

---

## Summary Table

| PR # | Content | Status | Dependencies |
|------|---------|--------|--------------|
| 1 | Monotonicity lemmas | ✅ Ready | None |
| 2 | Exp Taylor bounds | ✅ In Mathlib | None |
| 3 | Binet kernel defs/bounds | ✅ Ready | PR 1 |
| 4 | Limit theorem | ⏳ Needs work | PR 3, Taylor lemmas |
| 5 | Integrability | ⏳ Needs work | PR 3, PR 4 |

---

## Next Steps

1. **Immediate**: Submit PR 1 (monotonicity lemmas) and PR 3 (Binet kernel)
2. **Short term**: Complete proofs for PR 4 and PR 5
3. **Long term**: Use these in `BinetFormula.lean` for the full Binet integral representation
