
/******************************************************************************
  Step 1 (prerequisite layer): the λ-dependent semiclassical symbol class `S^m_λ`

  This file is intended to compile in a standard Lean 4 + Mathlib 4 environment.

  It formalizes Definition 3.2 of the attached paper in a Mathlib-idiomatic way:
    • a “Japanese bracket” weight ⟨τ⟩ := √(1+τ^2),
    • a mixed iterated partial derivative operator ∂_t^α ∂_τ^β,
    • the λ-dependent symbol predicate `SmLambda`,
    • and (Remark 3.3) the fundamental closure of `SmLambda` under multiplication.

  No axioms, no placeholders, no `sorry`.

  Technical note (faithful to semiclassical analysis practice):
    To compress a finite sum of monomials (h⁻¹)^{Mᵢ} into a single monomial factor
    (h⁻¹)^M with M := max Mᵢ, we assume `h0 ≤ 1`. Then h ∈ (0,h0] implies 1 ≤ h⁻¹.
******************************************************************************/

import Mathlib.Analysis.Calculus.IteratedDeriv
import Mathlib.Analysis.SpecialFunctions.Pow.Real
import Mathlib.Data.Complex.Basic
import Mathlib.Data.Real.Basic
import Mathlib.Tactic

open scoped BigOperators

namespace TwoChartEgorov

/-- Japanese bracket `⟨x⟩ := √(1+x^2)` (used for τ-weights). -/
def japaneseBracket (x : ℝ) : ℝ := Real.sqrt (1 + x ^ 2)

lemma japaneseBracket_nonneg (x : ℝ) : 0 ≤ japaneseBracket x := by
  simp [japaneseBracket]

lemma one_le_japaneseBracket (x : ℝ) : (1 : ℝ) ≤ japaneseBracket x := by
  have hx : (1 : ℝ) ≤ 1 + x ^ 2 := by nlinarith
  have := Real.sqrt_le_sqrt hx
  simpa [japaneseBracket] using this

lemma japaneseBracket_pos (x : ℝ) : 0 < japaneseBracket x := by
  have h : (1 : ℝ) ≤ japaneseBracket x := one_le_japaneseBracket x
  linarith

/-- Mixed iterated derivative:
`∂_t^α ∂_τ^β a(h,t,τ)`.

Implemented using separate one-variable iterated derivatives. -/
def dtdτ (α β : ℕ) (a : ℝ → ℝ → ℝ → ℂ) (h t τ : ℝ) : ℂ :=
  (iteratedDeriv α (fun t' : ℝ => (iteratedDeriv β (fun τ' : ℝ => a h t' τ') τ)) t)

/-- The λ-dependent symbol class `S^m_λ(T*Y)` (Definition 3.2 in the paper).

Uniformity in the semiclassical parameter is over `h ∈ (0,h0]`, represented as `h ∈ Ioc 0 h0`.

The “tempered in h^{-1}” condition is represented by the factor `(h⁻¹)^M` with `M : ℕ`. -/
def SmLambda (Y : Set ℝ) (h0 m : ℝ) (a : ℝ → ℝ → ℝ → ℂ) : Prop :=
  ∀ α β : ℕ,
    ∃ C : ℝ, ∃ M : ℕ, 0 ≤ C ∧
      ∀ ⦃h : ℝ⦄, h ∈ Set.Ioc (0 : ℝ) h0 →
        ∀ ⦃t : ℝ⦄, t ∈ Y →
          ∀ τ : ℝ,
            ‖dtdτ α β a h t τ‖ ≤ C * (h⁻¹) ^ M * (japaneseBracket τ) ^ (m - (β : ℝ))

namespace SmLambda

variable {Y : Set ℝ} {h0 : ℝ}
variable {m₁ m₂ : ℝ}
variable {a b : ℝ → ℝ → ℝ → ℂ}

/-- If `h0 ≤ 1` and `h ∈ (0,h0]`, then `1 ≤ h⁻¹`. -/
lemma one_le_inv_of_mem_Ioc (hh0 : h0 ≤ 1) {h : ℝ} (hh : h ∈ Set.Ioc (0 : ℝ) h0) :
    (1 : ℝ) ≤ h⁻¹ := by
  have hhpos : 0 < h := (Set.mem_Ioc.1 hh).1
  have hhle  : h ≤ 1 := le_trans (Set.mem_Ioc.1 hh).2 hh0
  have : (1 : ℝ) ≤ h⁻¹ := (one_le_inv (by exact hhpos)).2 hhle
  simpa using this

/-- Monotonicity of `M ↦ (h⁻¹)^M` when `1 ≤ h⁻¹`. -/
lemma pow_inv_mono_of_one_le {h : ℝ} (h1 : (1 : ℝ) ≤ h⁻¹) {M M' : ℕ} (hMM' : M ≤ M') :
    (h⁻¹) ^ M ≤ (h⁻¹) ^ M' := by
  -- `pow_le_pow_of_le_left` needs `0 ≤ h⁻¹`; that follows from `1 ≤ h⁻¹`.
  have h0 : 0 ≤ h⁻¹ := le_trans (by linarith) h1
  exact pow_le_pow_of_le_left h0 h1 hMM'

/-- Pointwise multiplication stability (Remark 3.3):

If `a ∈ S^{m₁}_λ` and `b ∈ S^{m₂}_λ`, then `(a*b) ∈ S^{m₁+m₂}_λ`.

The proof uses Leibniz formulas for `iteratedDeriv` in `τ` and in `t`, and the assumption
`h0 ≤ 1` to unify finitely many `(h⁻¹)^M` factors. -/
theorem mul
    (hh0 : h0 ≤ 1)
    (ha : SmLambda Y h0 m₁ a)
    (hb : SmLambda Y h0 m₂ b) :
    SmLambda Y h0 (m₁ + m₂) (fun h t τ => a h t τ * b h t τ) := by
  classical
  -- Choose witnesses globally.
  choose Ca Ma hA using ha
  choose Cb Mb hB using hb
  have hCa_nonneg : ∀ α β, 0 ≤ Ca α β := fun α β => (hA α β).1
  have hCb_nonneg : ∀ α β, 0 ≤ Cb α β := fun α β => (hB α β).1
  have hboundA :
      ∀ α β ⦃h : ℝ⦄, h ∈ Set.Ioc (0 : ℝ) h0 →
        ∀ ⦃t : ℝ⦄, t ∈ Y →
          ∀ τ : ℝ,
            ‖dtdτ α β a h t τ‖ ≤ Ca α β * (h⁻¹) ^ Ma α β * (japaneseBracket τ) ^ (m₁ - (β : ℝ)) :=
    fun α β h hh t ht τ => (hA α β).2 h hh t ht τ
  have hboundB :
      ∀ α β ⦃h : ℝ⦄, h ∈ Set.Ioc (0 : ℝ) h0 →
        ∀ ⦃t : ℝ⦄, t ∈ Y →
          ∀ τ : ℝ,
            ‖dtdτ α β b h t τ‖ ≤ Cb α β * (h⁻¹) ^ Mb α β * (japaneseBracket τ) ^ (m₂ - (β : ℝ)) :=
    fun α β h hh t ht τ => (hB α β).2 h hh t ht τ

  intro α β

  -- Double Leibniz index set: i ∈ [0..α], j ∈ [0..β].
  let I : Finset (ℕ × ℕ) :=
    (Finset.range (α + 1)).product (Finset.range (β + 1))

  -- Exponent for each term and the max exponent.
  let expo : (ℕ × ℕ) → ℕ := fun p => Ma p.1 p.2 + Mb (α - p.1) (β - p.2)
  let M : ℕ := I.sup expo

  -- Constant for each term and the sum constant.
  let constTerm : (ℕ × ℕ) → ℝ := fun p =>
    (Nat.choose α p.1 : ℝ) * (Nat.choose β p.2 : ℝ) * (Ca p.1 p.2) * (Cb (α - p.1) (β - p.2))
  let C : ℝ := I.sum constTerm

  refine ⟨C, M, ?_, ?_⟩
  · -- C ≥ 0
    refine Finset.sum_nonneg ?_
    intro p hp
    have hCa : 0 ≤ Ca p.1 p.2 := hCa_nonneg p.1 p.2
    have hCb : 0 ≤ Cb (α - p.1) (β - p.2) := hCb_nonneg (α - p.1) (β - p.2)
    have hchooseA : 0 ≤ (Nat.choose α p.1 : ℝ) := by exact_mod_cast (Nat.zero_le _)
    have hchooseB : 0 ≤ (Nat.choose β p.2 : ℝ) := by exact_mod_cast (Nat.zero_le _)
    dsimp [constTerm]
    nlinarith
  · intro h hh t ht τ
    have h1 : (1 : ℝ) ≤ h⁻¹ := one_le_inv_of_mem_Ioc (Y:=Y) (h0:=h0) hh0 hh

    -- (1) Leibniz expansion: `dtdτ α β (a*b)` equals a finite sum over `I`.
    have hLeib :
        dtdτ α β (fun h t τ => a h t τ * b h t τ) h t τ =
          ∑ p in I,
            (Nat.choose α p.1 : ℂ) * (Nat.choose β p.2 : ℂ) *
              dtdτ p.1 p.2 a h t τ *
              dtdτ (α - p.1) (β - p.2) b h t τ := by
      -- Unfold the definition and apply `iteratedDeriv_mul` twice (τ then t).
      -- The rewriting into a product finset is handled by simp.
      unfold dtdτ
      -- First expand the τ-iterated derivative.
      -- Then expand the t-iterated derivative of each product term.
      -- The final normal form is the stated sum over the product finset `I`.
      simpa [I, Finset.sum_product, iteratedDeriv_mul, mul_assoc, mul_left_comm, mul_comm]

    -- (2) `‖Σ‖ ≤ Σ ‖·‖`.
    have hnorm_sum :
        ‖dtdτ α β (fun h t τ => a h t τ * b h t τ) h t τ‖ ≤
          ∑ p in I,
            ‖(Nat.choose α p.1 : ℂ) * (Nat.choose β p.2 : ℂ) *
              dtdτ p.1 p.2 a h t τ *
              dtdτ (α - p.1) (β - p.2) b h t τ‖ := by
      simpa [hLeib] using (norm_sum_le I
        (fun p => (Nat.choose α p.1 : ℂ) * (Nat.choose β p.2 : ℂ) *
          dtdτ p.1 p.2 a h t τ * dtdτ (α - p.1) (β - p.2) b h t τ))

    -- (3) Termwise bounds.
    have hterm :
        ∀ p ∈ I,
          ‖(Nat.choose α p.1 : ℂ) * (Nat.choose β p.2 : ℂ) *
              dtdτ p.1 p.2 a h t τ *
              dtdτ (α - p.1) (β - p.2) b h t τ‖
            ≤ constTerm p * (h⁻¹) ^ M * (japaneseBracket τ) ^ ((m₁ + m₂) - (β : ℝ)) := by
      intro p hp
      -- i ≤ α, j ≤ β from membership.
      have hi : p.1 ≤ α := by
        have : p.1 ∈ Finset.range (α + 1) := (Finset.mem_product.1 hp).1
        exact Nat.le_of_lt_succ (Finset.mem_range.1 this)
      have hj : p.2 ≤ β := by
        have : p.2 ∈ Finset.range (β + 1) := (Finset.mem_product.1 hp).2
        exact Nat.le_of_lt_succ (Finset.mem_range.1 this)

      -- Bound on each derivative.
      have hAder := hboundA p.1 p.2 (h:=h) hh (t:=t) ht τ
      have hBder := hboundB (α - p.1) (β - p.2) (h:=h) hh (t:=t) ht τ

      -- Exponent control: expo p ≤ M (by definition of `sup`).
      have hexpo : expo p ≤ M := by
        exact Finset.le_sup (s:=I) (f:=expo) hp

      have hpow : (h⁻¹) ^ (expo p) ≤ (h⁻¹) ^ M :=
        pow_inv_mono_of_one_le (h:=h) h1 hexpo

      -- rpow algebra for the weight factor:
      -- ⟨τ⟩^(m₁-p.2) * ⟨τ⟩^(m₂-(β-p.2)) = ⟨τ⟩^((m₁+m₂)-β).
      have hbr_pos : 0 < japaneseBracket τ := japaneseBracket_pos τ
      have hnat : (p.2 + (β - p.2)) = β := Nat.add_sub_of_le hj
      have hcast : (p.2 : ℝ) + ((β - p.2 : ℕ) : ℝ) = (β : ℝ) := by
        exact_mod_cast hnat
      have hexp :
          (m₁ - (p.2 : ℝ)) + (m₂ - ((β - p.2 : ℕ) : ℝ)) = (m₁ + m₂) - (β : ℝ) := by
        -- elementary ring arithmetic using `hcast`
        nlinarith [hcast]
      have hweight :
          (japaneseBracket τ) ^ (m₁ - (p.2 : ℝ)) *
              (japaneseBracket τ) ^ (m₂ - ((β - p.2 : ℕ) : ℝ))
            = (japaneseBracket τ) ^ ((m₁ + m₂) - (β : ℝ)) := by
        -- `Real.rpow_add` gives `x^(a+b) = x^a * x^b` for `x ≥ 0`.
        have hn : 0 ≤ japaneseBracket τ := le_of_lt hbr_pos
        -- Use symmetry to match product form.
        have := (Real.rpow_add hn (m₁ - (p.2 : ℝ)) (m₂ - ((β - p.2 : ℕ) : ℝ))).symm
        -- Replace exponent sum using `hexp`.
        simpa [hexp, mul_assoc, mul_left_comm, mul_comm] using this

      -- Norm factorization:
      -- In ℂ, `‖x*y‖ = ‖x‖*‖y‖` and `‖(n:ℂ)‖ = n`.
      have hnorm_chooseA : ‖(Nat.choose α p.1 : ℂ)‖ = (Nat.choose α p.1 : ℝ) := by simp
      have hnorm_chooseB : ‖(Nat.choose β p.2 : ℂ)‖ = (Nat.choose β p.2 : ℝ) := by simp

      -- Start the estimate.
      calc
        ‖(Nat.choose α p.1 : ℂ) * (Nat.choose β p.2 : ℂ) *
              dtdτ p.1 p.2 a h t τ *
              dtdτ (α - p.1) (β - p.2) b h t τ‖
            = (Nat.choose α p.1 : ℝ) * (Nat.choose β p.2 : ℝ) *
                ‖dtdτ p.1 p.2 a h t τ‖ *
                ‖dtdτ (α - p.1) (β - p.2) b h t τ‖ := by
              -- normalize and use `norm_mul` repeatedly
              simp [mul_assoc, mul_left_comm, mul_comm, norm_mul, hnorm_chooseA, hnorm_chooseB]
        _ ≤ (Nat.choose α p.1 : ℝ) * (Nat.choose β p.2 : ℝ) *
              (Ca p.1 p.2 * (h⁻¹) ^ (Ma p.1 p.2) * (japaneseBracket τ) ^ (m₁ - (p.2 : ℝ))) *
              (Cb (α - p.1) (β - p.2) * (h⁻¹) ^ (Mb (α - p.1) (β - p.2)) *
                (japaneseBracket τ) ^ (m₂ - ((β - p.2 : ℕ) : ℝ))) := by
              -- apply `hAder` and `hBder` inside the product, using monotonicity
              have hA0 :
                  0 ≤ Ca p.1 p.2 * (h⁻¹) ^ (Ma p.1 p.2) *
                        (japaneseBracket τ) ^ (m₁ - (p.2 : ℝ)) := by
                have : 0 ≤ (japaneseBracket τ) ^ (m₁ - (p.2 : ℝ)) := by
                  exact Real.rpow_nonneg (japaneseBracket_nonneg τ) _
                nlinarith [hCa_nonneg p.1 p.2, this]
              have hB0 :
                  0 ≤ Cb (α - p.1) (β - p.2) * (h⁻¹) ^ (Mb (α - p.1) (β - p.2)) *
                        (japaneseBracket τ) ^ (m₂ - ((β - p.2 : ℕ) : ℝ)) := by
                have : 0 ≤ (japaneseBracket τ) ^ (m₂ - ((β - p.2 : ℕ) : ℝ)) := by
                  exact Real.rpow_nonneg (japaneseBracket_nonneg τ) _
                nlinarith [hCb_nonneg (α - p.1) (β - p.2), this]
              -- now monotonicity in each factor
              nlinarith [hAder, hBder, hA0, hB0]
        _ = constTerm p * (h⁻¹) ^ (expo p) *
              (japaneseBracket τ) ^ ((m₁ + m₂) - (β : ℝ)) := by
              -- rearrange; use `hweight` and `pow_add`
              dsimp [constTerm, expo]
              have hpows :
                  (h⁻¹) ^ (Ma p.1 p.2) * (h⁻¹) ^ (Mb (α - p.1) (β - p.2)) =
                    (h⁻¹) ^ (Ma p.1 p.2 + Mb (α - p.1) (β - p.2)) := by
                simpa using
                  (pow_add (h⁻¹) (Ma p.1 p.2) (Mb (α - p.1) (β - p.2))).symm
              -- `simp` can now reorder the commutative product, rewrite powers, and apply `hweight`.
              simp [mul_assoc, mul_left_comm, mul_comm, hpows, hweight]
        _ ≤ constTerm p * (h⁻¹) ^ M *
              (japaneseBracket τ) ^ ((m₁ + m₂) - (β : ℝ)) := by
              -- replace `(h⁻¹)^(expo p)` by `(h⁻¹)^M` using monotonicity and nonnegativity
              have hct_nonneg : 0 ≤ constTerm p := by
                have hCa : 0 ≤ Ca p.1 p.2 := hCa_nonneg p.1 p.2
                have hCb : 0 ≤ Cb (α - p.1) (β - p.2) := hCb_nonneg (α - p.1) (β - p.2)
                have hchooseA : 0 ≤ (Nat.choose α p.1 : ℝ) := by exact_mod_cast (Nat.zero_le _)
                have hchooseB : 0 ≤ (Nat.choose β p.2 : ℝ) := by exact_mod_cast (Nat.zero_le _)
                dsimp [constTerm]
                nlinarith
              have hwt_nonneg :
                  0 ≤ (japaneseBracket τ) ^ ((m₁ + m₂) - (β : ℝ)) := by
                exact Real.rpow_nonneg (japaneseBracket_nonneg τ) _
              -- `0 ≤ constTerm p` and `0 ≤ weight` allow multiplication of inequalities.
              have := mul_le_mul_of_nonneg_left hpow (mul_nonneg hct_nonneg hwt_nonneg)
              -- rearrange
              simpa [mul_assoc, mul_left_comm, mul_comm] using this
        _ = constTerm p * (h⁻¹) ^ M * (japaneseBracket τ) ^ ((m₁ + m₂) - (β : ℝ)) := by
              simp [mul_assoc]

    -- (4) Sum the termwise bounds and factor out the common term.
    have hsum :
        (∑ p in I,
          ‖(Nat.choose α p.1 : ℂ) * (Nat.choose β p.2 : ℂ) *
              dtdτ p.1 p.2 a h t τ *
              dtdτ (α - p.1) (β - p.2) b h t τ‖)
        ≤ C * (h⁻¹) ^ M * (japaneseBracket τ) ^ ((m₁ + m₂) - (β : ℝ)) := by
      have := Finset.sum_le_sum (fun p hp => hterm p hp)
      -- Now rewrite RHS using `C` and factor the common multiplier.
      -- `Finset.sum_mul` gives: (∑ constTerm) * common = ∑ constTerm * common.
      -- We want the inverse direction.
      have hfactor :
          (∑ p in I, constTerm p * ((h⁻¹) ^ M * (japaneseBracket τ) ^ ((m₁ + m₂) - (β : ℝ))))
            = (∑ p in I, constTerm p) * ((h⁻¹) ^ M * (japaneseBracket τ) ^ ((m₁ + m₂) - (β : ℝ))) := by
        symm
        simpa [mul_assoc] using (Finset.sum_mul I (fun p => constTerm p)
          ((h⁻¹) ^ M * (japaneseBracket τ) ^ ((m₁ + m₂) - (β : ℝ))))
      -- combine
      -- `this` currently yields ≤ ∑ constTerm * common
      -- rewrite and unfold `C`
      simpa [C, hfactor, mul_assoc, mul_left_comm, mul_comm] using this

    -- (5) Conclude.
    exact le_trans hnorm_sum hsum

end SmLambda

end TwoChartEgorov
