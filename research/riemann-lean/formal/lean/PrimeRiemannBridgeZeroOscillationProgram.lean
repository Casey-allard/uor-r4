import PrimeRiemannBridgeImportedInstance

namespace PrimeRiemannBridgeZeroOscillationProgram

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeCompletionKernel
open PrimeRiemannBridgeImportedInstance

/-!
Final theorem program for the remaining bridge:
if a formalized explicit-formula result provides signed oscillation from
`Re(s) > 1/2` zeros, we can construct the published pack and derive RH.
-/

structure ExplicitFormulaSignedOscillationAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_signed_oscillation :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ c β : Real, 0 < c ∧ (1 / 2 : Real) < β ∧
            ((∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≥ c * x ^ β) ∨
             (∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≤ - (c * x ^ β)))

theorem signed_pos_from_sequence
    (E : Real → Real)
    (c β : Real)
    (f : Nat → Real)
    (hTendsto : Filter.Tendsto f Filter.atTop Filter.atTop)
    (hBound : ∀ n : Nat, E (f n) ≥ c * (f n) ^ β) :
    ∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≥ c * x ^ β := by
  intro X
  have hEventually : ∀ᶠ n : Nat in Filter.atTop, X ≤ f n := (Filter.tendsto_atTop.1 hTendsto) X
  rcases Filter.eventually_atTop.1 hEventually with ⟨N, hN⟩
  refine ⟨f N, hN N le_rfl, ?_⟩
  simpa using hBound N

theorem signed_neg_from_sequence
    (E : Real → Real)
    (c β : Real)
    (f : Nat → Real)
    (hTendsto : Filter.Tendsto f Filter.atTop Filter.atTop)
    (hBound : ∀ n : Nat, E (f n) ≤ - (c * (f n) ^ β)) :
    ∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≤ - (c * x ^ β) := by
  intro X
  have hEventually : ∀ᶠ n : Nat in Filter.atTop, X ≤ f n := (Filter.tendsto_atTop.1 hTendsto) X
  rcases Filter.eventually_atTop.1 hEventually with ⟨N, hN⟩
  refine ⟨f N, hN N le_rfl, ?_⟩
  simpa using hBound N

theorem signed_pos_from_sequence_eventually
    (E : Real → Real)
    (c β : Real)
    (f : Nat → Real)
    (hTendsto : Filter.Tendsto f Filter.atTop Filter.atTop)
    (hBound : ∀ᶠ n : Nat in Filter.atTop, E (f n) ≥ c * (f n) ^ β) :
    ∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≥ c * x ^ β := by
  intro X
  have hEventuallyX : ∀ᶠ n : Nat in Filter.atTop, X ≤ f n := (Filter.tendsto_atTop.1 hTendsto) X
  have hBoth : ∀ᶠ n : Nat in Filter.atTop, X ≤ f n ∧ E (f n) ≥ c * (f n) ^ β :=
    Filter.Eventually.and hEventuallyX hBound
  rcases Filter.eventually_atTop.1 hBoth with ⟨N, hN⟩
  have hNX : X ≤ f N := (hN N le_rfl).1
  have hNB : E (f N) ≥ c * (f N) ^ β := (hN N le_rfl).2
  exact ⟨f N, hNX, hNB⟩

theorem signed_neg_from_sequence_eventually
    (E : Real → Real)
    (c β : Real)
    (f : Nat → Real)
    (hTendsto : Filter.Tendsto f Filter.atTop Filter.atTop)
    (hBound : ∀ᶠ n : Nat in Filter.atTop, E (f n) ≤ - (c * (f n) ^ β)) :
    ∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≤ - (c * x ^ β) := by
  intro X
  have hEventuallyX : ∀ᶠ n : Nat in Filter.atTop, X ≤ f n := (Filter.tendsto_atTop.1 hTendsto) X
  have hBoth : ∀ᶠ n : Nat in Filter.atTop, X ≤ f n ∧ E (f n) ≤ - (c * (f n) ^ β) :=
    Filter.Eventually.and hEventuallyX hBound
  rcases Filter.eventually_atTop.1 hBoth with ⟨N, hN⟩
  have hNX : X ≤ f N := (hN N le_rfl).1
  have hNB : E (f N) ≤ - (c * (f N) ^ β) := (hN N le_rfl).2
  exact ⟨f N, hNX, hNB⟩

theorem sequence_eventual_pos_from_main_remainder
    (E M R : Real → Real)
    (c β : Real)
    (f : Nat → Real)
    (hDecomp : ∀ n : Nat, E (f n) = M (f n) + R (f n))
    (hMain : ∀ᶠ n : Nat in Filter.atTop, M (f n) ≥ (2 * c) * (f n) ^ β)
    (hRem : ∀ᶠ n : Nat in Filter.atTop, |R (f n)| ≤ c * (f n) ^ β) :
    ∀ᶠ n : Nat in Filter.atTop, E (f n) ≥ c * (f n) ^ β := by
  filter_upwards [hMain, hRem] with n hnMain hnRem
  have hRlower : R (f n) ≥ -(c * (f n) ^ β) := by
    have hneg : -|R (f n)| ≤ R (f n) := neg_abs_le (R (f n))
    have hbound : - (c * (f n) ^ β) ≤ R (f n) := by
      linarith
    exact hbound
  have hE : E (f n) = M (f n) + R (f n) := hDecomp n
  rw [hE]
  linarith

theorem sequence_eventual_neg_from_main_remainder
    (E M R : Real → Real)
    (c β : Real)
    (f : Nat → Real)
    (hDecomp : ∀ n : Nat, E (f n) = M (f n) + R (f n))
    (hMain : ∀ᶠ n : Nat in Filter.atTop, M (f n) ≤ -((2 * c) * (f n) ^ β))
    (hRem : ∀ᶠ n : Nat in Filter.atTop, |R (f n)| ≤ c * (f n) ^ β) :
    ∀ᶠ n : Nat in Filter.atTop, E (f n) ≤ - (c * (f n) ^ β) := by
  filter_upwards [hMain, hRem] with n hnMain hnRem
  have hRupper : R (f n) ≤ c * (f n) ^ β := by
    exact le_trans (le_abs_self (R (f n))) hnRem
  have hE : E (f n) = M (f n) + R (f n) := hDecomp n
  rw [hE]
  linarith

theorem eventually_abs_le_of_tendsto_zero
    (g : Nat → Real)
    (c : Real)
    (hc : 0 < c)
    (hTendsto : Filter.Tendsto g Filter.atTop (nhds 0)) :
    ∀ᶠ n : Nat in Filter.atTop, |g n| ≤ c := by
  have hIcc : Set.Icc (-c) c ∈ nhds (0 : Real) := by
    exact Icc_mem_nhds (by linarith) hc
  refine Filter.mem_of_superset (hTendsto hIcc) ?_
  intro n hn
  exact abs_le.mpr hn

theorem sequence_eventual_remainder_bound_of_tendsto_zero
    (R : Real → Real)
    (f : Nat → Real)
    (β c : Real)
    (hc : 0 < c)
    (hTendstoQuot :
      Filter.Tendsto (fun n : Nat => R (f n) / (f n) ^ β) Filter.atTop (nhds 0))
    (hFgeOne : ∀ᶠ n : Nat in Filter.atTop, f n ≥ 1) :
    ∀ᶠ n : Nat in Filter.atTop, |R (f n)| ≤ c * (f n) ^ β := by
  have hAbsQuot : ∀ᶠ n : Nat in Filter.atTop, |R (f n) / (f n) ^ β| ≤ c := by
    exact eventually_abs_le_of_tendsto_zero
      (g := fun n : Nat => R (f n) / (f n) ^ β)
      (c := c) hc hTendstoQuot
  filter_upwards [hAbsQuot, hFgeOne] with n hnAbs hnF
  have hf_pos : 0 < f n := lt_of_lt_of_le (by norm_num : (0 : Real) < 1) hnF
  have hpow_pos : 0 < (f n) ^ β := Real.rpow_pos_of_pos hf_pos β
  have hpow_abs : |(f n) ^ β| = (f n) ^ β := abs_of_nonneg (le_of_lt hpow_pos)
  have hnAbs' : |R (f n)| / |(f n) ^ β| ≤ c := by
    simpa [abs_div] using hnAbs
  have hmul :
      (|R (f n)| / |(f n) ^ β|) * |(f n) ^ β| ≤ c * |(f n) ^ β| :=
    mul_le_mul_of_nonneg_right hnAbs' (abs_nonneg ((f n) ^ β))
  have hleft :
      (|R (f n)| / |(f n) ^ β|) * |(f n) ^ β| = |R (f n)| := by
    have hden_ne : |(f n) ^ β| ≠ 0 := by
      rw [hpow_abs]
      exact ne_of_gt hpow_pos
    field_simp [hden_ne]
  have hright : c * |(f n) ^ β| = c * (f n) ^ β := by
    rw [hpow_abs]
  calc
    |R (f n)| = (|R (f n)| / |(f n) ^ β|) * |(f n) ^ β| := by symm; exact hleft
    _ ≤ c * |(f n) ^ β| := hmul
    _ = c * (f n) ^ β := hright

structure ExplicitFormulaSequenceOscillationAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_sequence_oscillation :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ c β : Real, 0 < c ∧ (1 / 2 : Real) < β ∧
            ((∃ f : Nat → Real, Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ n : Nat, E (f n) ≥ c * (f n) ^ β)) ∨
             (∃ f : Nat → Real, Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ n : Nat, E (f n) ≤ - (c * (f n) ^ β))))

def signedAssumptionsOfSequence
    (h : ExplicitFormulaSequenceOscillationAssumptions) :
    ExplicitFormulaSignedOscillationAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_signed_oscillation := by
    intro E hVonKoch s hs hs_gt
    rcases h.zero_to_sequence_oscillation E hVonKoch s hs hs_gt with ⟨c, β, hc, hβ, hSeq⟩
    refine ⟨c, β, hc, hβ, ?_⟩
    rcases hSeq with hPos | hNeg
    · rcases hPos with ⟨f, hTendsto, hBound⟩
      exact Or.inl (signed_pos_from_sequence E c β f hTendsto hBound)
    · rcases hNeg with ⟨f, hTendsto, hBound⟩
      exact Or.inr (signed_neg_from_sequence E c β f hTendsto hBound)

def signedPackOfAssumptions
    (h : ExplicitFormulaSignedOscillationAssumptions) :
    PublishedZeroOscillationSignedPack where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  right_half_zero_forces_lower_envelope_signed := h.zero_to_signed_oscillation

def strongPackOfAssumptions
    (h : ExplicitFormulaSignedOscillationAssumptions) :
    PublishedZeroOscillationPack :=
  strengthenPublishedZeroOscillationPack (weakenSignedToWeakPack (signedPackOfAssumptions h))

theorem endpoint_to_rh_of_explicit_formula_signed_oscillation
    (h : ExplicitFormulaSignedOscillationAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_imported_published_pack (strongPackOfAssumptions h)

theorem endpoint_to_rh_of_explicit_formula_sequence_oscillation
    (h : ExplicitFormulaSequenceOscillationAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_explicit_formula_signed_oscillation (signedAssumptionsOfSequence h)

structure ExplicitFormulaSequenceEventuallyOscillationAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_sequence_eventual_oscillation :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ c β : Real, 0 < c ∧ (1 / 2 : Real) < β ∧
            ((∃ f : Nat → Real, Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ᶠ n : Nat in Filter.atTop, E (f n) ≥ c * (f n) ^ β)) ∨
             (∃ f : Nat → Real, Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ᶠ n : Nat in Filter.atTop, E (f n) ≤ - (c * (f n) ^ β))))

structure ExplicitFormulaDecompositionSequenceAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_sequence_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ c β : Real, 0 < c ∧ (1 / 2 : Real) < β ∧
            ((∃ f : Nat → Real, ∃ M R : Real → Real,
                Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ n : Nat, E (f n) = M (f n) + R (f n)) ∧
                (∀ᶠ n : Nat in Filter.atTop, M (f n) ≥ (2 * c) * (f n) ^ β) ∧
                (∀ᶠ n : Nat in Filter.atTop, |R (f n)| ≤ c * (f n) ^ β)) ∨
             (∃ f : Nat → Real, ∃ M R : Real → Real,
                Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ n : Nat, E (f n) = M (f n) + R (f n)) ∧
                (∀ᶠ n : Nat in Filter.atTop, M (f n) ≤ -((2 * c) * (f n) ^ β)) ∧
                (∀ᶠ n : Nat in Filter.atTop, |R (f n)| ≤ c * (f n) ^ β)))

structure ExplicitFormulaAsymptoticSequenceAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
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

def decompositionAssumptionsOfAsymptotic
    (h : ExplicitFormulaAsymptoticSequenceAssumptions) :
    ExplicitFormulaDecompositionSequenceAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_sequence_decomposition := by
    intro E hVonKoch s hs hs_gt
    rcases h.zero_to_sequence_asymptotic E hVonKoch s hs hs_gt with ⟨c, β, hc, hβ, hAsy⟩
    refine ⟨c, β, hc, hβ, ?_⟩
    rcases hAsy with hPos | hNeg
    · rcases hPos with ⟨f, M, R, hTendsto, hDecomp, hMain, hRemTendsto⟩
      have hFgeOne : ∀ᶠ n : Nat in Filter.atTop, f n ≥ 1 := (Filter.tendsto_atTop.1 hTendsto) 1
      have hRem :
          ∀ᶠ n : Nat in Filter.atTop, |R (f n)| ≤ c * (f n) ^ β :=
        sequence_eventual_remainder_bound_of_tendsto_zero R f β c hc hRemTendsto hFgeOne
      exact Or.inl ⟨f, M, R, hTendsto, hDecomp, hMain, hRem⟩
    · rcases hNeg with ⟨f, M, R, hTendsto, hDecomp, hMain, hRemTendsto⟩
      have hFgeOne : ∀ᶠ n : Nat in Filter.atTop, f n ≥ 1 := (Filter.tendsto_atTop.1 hTendsto) 1
      have hRem :
          ∀ᶠ n : Nat in Filter.atTop, |R (f n)| ≤ c * (f n) ^ β :=
        sequence_eventual_remainder_bound_of_tendsto_zero R f β c hc hRemTendsto hFgeOne
      exact Or.inr ⟨f, M, R, hTendsto, hDecomp, hMain, hRem⟩

def sequenceEventuallyAssumptionsOfDecomposition
    (h : ExplicitFormulaDecompositionSequenceAssumptions) :
    ExplicitFormulaSequenceEventuallyOscillationAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_sequence_eventual_oscillation := by
    intro E hVonKoch s hs hs_gt
    rcases h.zero_to_sequence_decomposition E hVonKoch s hs hs_gt with ⟨c, β, hc, hβ, hDec⟩
    refine ⟨c, β, hc, hβ, ?_⟩
    rcases hDec with hPos | hNeg
    · rcases hPos with ⟨f, M, R, hTendsto, hDecomp, hMain, hRem⟩
      have hPosEv : ∀ᶠ n : Nat in Filter.atTop, E (f n) ≥ c * (f n) ^ β := by
        exact sequence_eventual_pos_from_main_remainder
          (E := E) (M := M) (R := R)
          (c := c) (β := β) (f := f)
          (hDecomp := hDecomp) (hMain := hMain) (hRem := hRem)
      exact Or.inl ⟨f, hTendsto, hPosEv⟩
    · rcases hNeg with ⟨f, M, R, hTendsto, hDecomp, hMain, hRem⟩
      have hNegEv : ∀ᶠ n : Nat in Filter.atTop, E (f n) ≤ - (c * (f n) ^ β) := by
        exact sequence_eventual_neg_from_main_remainder
          (E := E) (M := M) (R := R)
          (c := c) (β := β) (f := f)
          (hDecomp := hDecomp) (hMain := hMain) (hRem := hRem)
      exact Or.inr ⟨f, hTendsto, hNegEv⟩

def signedAssumptionsOfSequenceEventually
    (h : ExplicitFormulaSequenceEventuallyOscillationAssumptions) :
    ExplicitFormulaSignedOscillationAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_signed_oscillation := by
    intro E hVonKoch s hs hs_gt
    rcases h.zero_to_sequence_eventual_oscillation E hVonKoch s hs hs_gt with ⟨c, β, hc, hβ, hSeq⟩
    refine ⟨c, β, hc, hβ, ?_⟩
    rcases hSeq with hPos | hNeg
    · rcases hPos with ⟨f, hTendsto, hBound⟩
      exact Or.inl (signed_pos_from_sequence_eventually E c β f hTendsto hBound)
    · rcases hNeg with ⟨f, hTendsto, hBound⟩
      exact Or.inr (signed_neg_from_sequence_eventually E c β f hTendsto hBound)

theorem endpoint_to_rh_of_explicit_formula_sequence_eventually_oscillation
    (h : ExplicitFormulaSequenceEventuallyOscillationAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_explicit_formula_signed_oscillation (signedAssumptionsOfSequenceEventually h)

theorem endpoint_to_rh_of_explicit_formula_decomposition_sequence
    (h : ExplicitFormulaDecompositionSequenceAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_explicit_formula_sequence_eventually_oscillation
    (sequenceEventuallyAssumptionsOfDecomposition h)

theorem endpoint_to_rh_of_explicit_formula_asymptotic_sequence
    (h : ExplicitFormulaAsymptoticSequenceAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_explicit_formula_decomposition_sequence
    (decompositionAssumptionsOfAsymptotic h)

end PrimeRiemannBridgeZeroOscillationProgram
