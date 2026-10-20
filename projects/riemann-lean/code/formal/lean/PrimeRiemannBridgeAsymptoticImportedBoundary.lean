import PrimeRiemannBridgeZeroOscillationProgram

namespace PrimeRiemannBridgeAsymptoticImportedBoundary

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeCompletionKernel
open PrimeRiemannBridgeZeroOscillationProgram

/-!
Single remaining import boundary:
the caller supplies only the zero-to-sequence asymptotic theorem term.
Citation locks stay hardcoded in this repository.
-/

structure ImportedAsymptoticSequenceTheoremTerm where
  zero_to_sequence_asymptotic :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ c β : Real, 0 < c ∧ (1 / 2 : Real) < β ∧
            ((∃ f : Nat → Real, ∃ M R : Real → Real,
                Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ n : Nat, E (f n) = M (f n) + R (f n)) ∧
                (∀ᶠ n : Nat in Filter.atTop, M (f n) ≥ (2 * c) * (f n) ^ β) ∧
                Filter.Tendsto (fun n : Nat => R (f n) / (f n) ^ β) Filter.atTop (nhds 0)) ∨
             (∃ f : Nat → Real, ∃ M R : Real → Real,
                Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ n : Nat, E (f n) = M (f n) + R (f n)) ∧
                (∀ᶠ n : Nat in Filter.atTop, M (f n) ≤ -((2 * c) * (f n) ^ β)) ∧
                Filter.Tendsto (fun n : Nat => R (f n) / (f n) ^ β) Filter.atTop (nhds 0)))

def assumptionsOfImportedTheoremTerm
    (t : ImportedAsymptoticSequenceTheoremTerm) :
    ExplicitFormulaAsymptoticSequenceAssumptions where
  source_tag := "PINTZ-2017-OSCILLATION"
  source_url := "https://doi.org/10.1134/S0081543817010163"
  theorem_ref := "Thm-2-zero-to-oscillation-transfer"
  source_tag_lock := rfl
  source_url_lock := rfl
  theorem_ref_lock := rfl
  zero_to_sequence_asymptotic := t.zero_to_sequence_asymptotic

class ImportedAsymptoticSequenceResults where
  theorem_term : ImportedAsymptoticSequenceTheoremTerm

theorem endpoint_to_rh_from_imported_asymptotic_results
    (r : ImportedAsymptoticSequenceResults) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_explicit_formula_asymptotic_sequence
    (assumptionsOfImportedTheoremTerm r.theorem_term)

def importedAsymptoticResultsOfTheoremTerm
    (t : ImportedAsymptoticSequenceTheoremTerm) :
    ImportedAsymptoticSequenceResults where
  theorem_term := t

theorem endpoint_to_rh_of_imported_asymptotic_theorem_term
    (t : ImportedAsymptoticSequenceTheoremTerm) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_explicit_formula_asymptotic_sequence
    (assumptionsOfImportedTheoremTerm t)

theorem endpoint_to_rh_of_imported_asymptotic_assumptions
    (h : ExplicitFormulaAsymptoticSequenceAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_explicit_formula_asymptotic_sequence h

theorem tendsto_atTop_of_nat_lower_bound
    (f : Nat → Real)
    (hLower : ∀ n : Nat, (n : Real) ≤ f n) :
    Filter.Tendsto f Filter.atTop Filter.atTop := by
  refine Filter.tendsto_atTop.2 ?_
  intro X
  rcases exists_nat_ge X with ⟨N, hN⟩
  refine Filter.eventually_atTop.2 ⟨N, ?_⟩
  intro n hn
  have hNn : (N : Real) ≤ (n : Real) := by exact_mod_cast hn
  exact le_trans (le_trans hN hNn) (hLower n)

theorem signed_envelope_of_abs_envelope
    (E : Real → Real)
    (β : Real)
    (hAbs : ∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β) :
    ((∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≥ x ^ β) ∨
     (∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≤ - (x ^ β))) := by
  classical
  by_cases hPos : ∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≥ x ^ β
  · exact Or.inl hPos
  · have hPosBound : ∃ X0 : Real, ∀ x : Real, x ≥ X0 → E x < x ^ β := by
      rcases not_forall.mp hPos with ⟨X0, hX0⟩
      refine ⟨X0, ?_⟩
      intro x hx
      have hNotGe : ¬ E x ≥ x ^ β := by
        intro hGe
        exact hX0 ⟨x, hx, hGe⟩
      exact lt_of_not_ge hNotGe
    right
    intro X
    by_contra hNoNeg
    have hNegLower : ∀ x : Real, x ≥ X → E x > - (x ^ β) := by
      intro x hx
      have hNotLe : ¬ E x ≤ - (x ^ β) := by
        intro hLe
        exact hNoNeg ⟨x, hx, hLe⟩
      exact lt_of_not_ge hNotLe
    rcases hPosBound with ⟨X0, hPosUpper⟩
    let Xmax : Real := max (max X0 X) 1
    rcases hAbs Xmax with ⟨x, hxmax, hAbsx⟩
    have hX0le : X0 ≤ Xmax := by
      exact le_trans (le_max_left X0 X) (le_max_left (max X0 X) 1)
    have hXle : X ≤ Xmax := by
      exact le_trans (le_max_right X0 X) (le_max_left (max X0 X) 1)
    have hxX0 : x ≥ X0 := le_trans hX0le hxmax
    have hxX : x ≥ X := le_trans hXle hxmax
    have hUpper : E x < x ^ β := hPosUpper x hxX0
    have hLower : - (x ^ β) < E x := hNegLower x hxX
    have hAbsLt : |E x| < x ^ β := abs_lt.mpr ⟨hLower, hUpper⟩
    exact (not_lt_of_ge hAbsx) hAbsLt

noncomputable def theoremTermOfPublishedPack
    (p : PublishedZeroOscillationPack) :
    ImportedAsymptoticSequenceTheoremTerm where
  zero_to_sequence_asymptotic := by
    intro E hVonKoch s hs hs_gt
    rcases p.right_half_zero_forces_lower_envelope E hVonKoch s hs hs_gt with ⟨β, hβ, hAbs⟩
    have hSigned :
        (∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≥ x ^ β) ∨
        (∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≤ - (x ^ β)) :=
      signed_envelope_of_abs_envelope E β hAbs
    refine ⟨(1 / 2 : Real), β, by norm_num, hβ, ?_⟩
    rcases hSigned with hPos | hNeg
    · let f : Nat → Real := fun n => Classical.choose (hPos (n : Real))
      let M : Real → Real := E
      let R : Real → Real := fun _ => 0
      have hf_ge : ∀ n : Nat, (n : Real) ≤ f n := by
        intro n
        exact (Classical.choose_spec (hPos (n : Real))).1
      have hTendsto : Filter.Tendsto f Filter.atTop Filter.atTop :=
        tendsto_atTop_of_nat_lower_bound f hf_ge
      have hDecomp : ∀ n : Nat, E (f n) = M (f n) + R (f n) := by
        intro n
        simp [M, R]
      have hMainAll : ∀ n : Nat, M (f n) ≥ (2 * (1 / 2 : Real)) * (f n) ^ β := by
        intro n
        have hBound : E (f n) ≥ (f n) ^ β := (Classical.choose_spec (hPos (n : Real))).2
        simpa [M] using (show E (f n) ≥ (2 * (1 / 2 : Real)) * (f n) ^ β from by
          simpa using hBound)
      have hMain :
          ∀ᶠ n : Nat in Filter.atTop, M (f n) ≥ (2 * (1 / 2 : Real)) * (f n) ^ β :=
        Filter.Eventually.of_forall hMainAll
      have hRemTendsto :
          Filter.Tendsto (fun n : Nat => R (f n) / (f n) ^ β) Filter.atTop (nhds 0) := by
        simpa [R] using
          (Filter.tendsto_const_nhds : Filter.Tendsto (fun _ : Nat => (0 : Real)) Filter.atTop (nhds 0))
      exact Or.inl ⟨f, M, R, hTendsto, hDecomp, hMain, hRemTendsto⟩
    · let f : Nat → Real := fun n => Classical.choose (hNeg (n : Real))
      let M : Real → Real := E
      let R : Real → Real := fun _ => 0
      have hf_ge : ∀ n : Nat, (n : Real) ≤ f n := by
        intro n
        exact (Classical.choose_spec (hNeg (n : Real))).1
      have hTendsto : Filter.Tendsto f Filter.atTop Filter.atTop :=
        tendsto_atTop_of_nat_lower_bound f hf_ge
      have hDecomp : ∀ n : Nat, E (f n) = M (f n) + R (f n) := by
        intro n
        simp [M, R]
      have hMainAll : ∀ n : Nat, M (f n) ≤ -((2 * (1 / 2 : Real)) * (f n) ^ β) := by
        intro n
        have hBound : E (f n) ≤ -((f n) ^ β) := (Classical.choose_spec (hNeg (n : Real))).2
        simpa [M] using (show E (f n) ≤ -((2 * (1 / 2 : Real)) * (f n) ^ β) from by
          simpa using hBound)
      have hMain :
          ∀ᶠ n : Nat in Filter.atTop, M (f n) ≤ -((2 * (1 / 2 : Real)) * (f n) ^ β) :=
        Filter.Eventually.of_forall hMainAll
      have hRemTendsto :
          Filter.Tendsto (fun n : Nat => R (f n) / (f n) ^ β) Filter.atTop (nhds 0) := by
        simpa [R] using
          (Filter.tendsto_const_nhds : Filter.Tendsto (fun _ : Nat => (0 : Real)) Filter.atTop (nhds 0))
      exact Or.inr ⟨f, M, R, hTendsto, hDecomp, hMain, hRemTendsto⟩

theorem endpoint_to_rh_of_published_pack_via_asymptotic_term
    (p : PublishedZeroOscillationPack) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_imported_asymptotic_theorem_term (theoremTermOfPublishedPack p)

end PrimeRiemannBridgeAsymptoticImportedBoundary
