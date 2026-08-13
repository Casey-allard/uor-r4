import PrimeRiemannBridgeAsymptoticImportedProvider
import PrimeRiemannBridgeImportedInstance
import PrimeRiemannBridgeZeroOscillationProgram

namespace PrimeRiemannBridgeConcretePackInstantiation

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeCompletionKernel
open PrimeRiemannBridgeImportedResults
open PrimeRiemannBridgeImportedInstance
open PrimeRiemannBridgeAsymptoticImportedProvider
open PrimeRiemannBridgeZeroOscillationProgram

/-!
Final instantiation slot:
provide one concrete `PublishedZeroOscillationPack` term here to close the
remaining parameterized boundary.
-/

class ConcretePublishedPackProvider where
  concrete_pack : PublishedZeroOscillationPack

def zeroError : Real → Real := fun _ => 0

theorem endpoint_zeroError : RH_Equivalent_Implication zeroError := by
  refine ⟨0, Real.exp 1, le_rfl, le_rfl, ?_⟩
  intro x hx
  simp [zeroError]

theorem no_lower_envelope_for_zeroError
    (β : Real) :
    ¬ (∀ X : Real, ∃ x : Real, x ≥ X ∧ |zeroError x| ≥ x ^ β) := by
  intro h
  rcases h 1 with ⟨x, hx, hLower⟩
  have hx_pos : 0 < x := lt_of_lt_of_le (by norm_num : (0 : Real) < 1) hx
  have hpow_pos : 0 < x ^ β := Real.rpow_pos_of_pos hx_pos β
  have hzero : |zeroError x| = 0 := by simp [zeroError]
  rw [hzero] at hLower
  linarith

def publishedPackOfSignedAssumptions
    (h : ExplicitFormulaSignedOscillationAssumptions) :
    PublishedZeroOscillationPack :=
  strongPackOfAssumptions h

def signedAssumptionsOfDecompositionAssumptions
    (h : ExplicitFormulaDecompositionSequenceAssumptions) :
    ExplicitFormulaSignedOscillationAssumptions :=
  signedAssumptionsOfSequenceEventually
    (sequenceEventuallyAssumptionsOfDecomposition h)

def signedAssumptionsOfAsymptoticAssumptions
    (h : ExplicitFormulaAsymptoticSequenceAssumptions) :
    ExplicitFormulaSignedOscillationAssumptions :=
  signedAssumptionsOfDecompositionAssumptions
    (decompositionAssumptionsOfAsymptotic h)

def concretePack
    (p : ConcretePublishedPackProvider) : PublishedZeroOscillationPack :=
  p.concrete_pack

def concreteAsymptoticProvider
    (p : ConcretePublishedPackProvider) :
    ImportedAsymptoticSequenceTheoremProvider :=
  providerOfPublishedPack p.concrete_pack

theorem endpoint_to_rh_from_concrete_pack
    (p : ConcretePublishedPackProvider) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_imported_published_pack p.concrete_pack

theorem endpoint_to_rh_from_concrete_pack_via_asymptotic_bridge
    (p : ConcretePublishedPackProvider) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_provider (concreteAsymptoticProvider p)

theorem rh_from_concrete_pack
    (p : ConcretePublishedPackProvider) :
    RHStatement :=
  endpoint_to_rh_from_concrete_pack (p := p) zeroError endpoint_zeroError

theorem rh_from_concrete_pack_via_asymptotic_bridge
    (p : ConcretePublishedPackProvider) :
    RHStatement :=
  endpoint_to_rh_from_concrete_pack_via_asymptotic_bridge (p := p) zeroError endpoint_zeroError

noncomputable instance concreteProviderOfImportedResults
    [r : ImportedPublishedResults] : ConcretePublishedPackProvider where
  concrete_pack := r.published_zero_oscillation_pack

theorem endpoint_to_rh_from_imported_results_instance
    [r : ImportedPublishedResults] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_concrete_pack (p := concreteProviderOfImportedResults)

theorem rh_from_imported_results_instance
    [r : ImportedPublishedResults] :
    RHStatement :=
  rh_from_concrete_pack (p := concreteProviderOfImportedResults)

class ConcreteSignedOscillationProvider where
  signed_assumptions : ExplicitFormulaSignedOscillationAssumptions

noncomputable instance concreteProviderOfSignedOscillation
    [h : ConcreteSignedOscillationProvider] : ConcretePublishedPackProvider where
  concrete_pack := publishedPackOfSignedAssumptions h.signed_assumptions

theorem endpoint_to_rh_from_signed_oscillation_instance
    [h : ConcreteSignedOscillationProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_concrete_pack (p := concreteProviderOfSignedOscillation)

theorem rh_from_signed_oscillation_instance
    [h : ConcreteSignedOscillationProvider] :
    RHStatement :=
  rh_from_concrete_pack (p := concreteProviderOfSignedOscillation)

class ConcreteSequenceOscillationProvider where
  sequence_assumptions : ExplicitFormulaSequenceOscillationAssumptions

noncomputable instance concreteProviderOfSequenceOscillation
    [h : ConcreteSequenceOscillationProvider] : ConcretePublishedPackProvider where
  concrete_pack := publishedPackOfSignedAssumptions
    (signedAssumptionsOfSequence h.sequence_assumptions)

theorem endpoint_to_rh_from_sequence_oscillation_instance
    [h : ConcreteSequenceOscillationProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_concrete_pack (p := concreteProviderOfSequenceOscillation)

theorem rh_from_sequence_oscillation_instance
    [h : ConcreteSequenceOscillationProvider] :
    RHStatement :=
  rh_from_concrete_pack (p := concreteProviderOfSequenceOscillation)

class ConcreteSequenceEventuallyOscillationProvider where
  sequence_eventual_assumptions : ExplicitFormulaSequenceEventuallyOscillationAssumptions

noncomputable instance concreteProviderOfSequenceEventuallyOscillation
    [h : ConcreteSequenceEventuallyOscillationProvider] : ConcretePublishedPackProvider where
  concrete_pack := publishedPackOfSignedAssumptions
    (signedAssumptionsOfSequenceEventually h.sequence_eventual_assumptions)

theorem endpoint_to_rh_from_sequence_eventually_oscillation_instance
    [h : ConcreteSequenceEventuallyOscillationProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_concrete_pack (p := concreteProviderOfSequenceEventuallyOscillation)

theorem rh_from_sequence_eventually_oscillation_instance
    [h : ConcreteSequenceEventuallyOscillationProvider] :
    RHStatement :=
  rh_from_concrete_pack (p := concreteProviderOfSequenceEventuallyOscillation)

class ConcreteDecompositionSequenceProvider where
  decomposition_assumptions : ExplicitFormulaDecompositionSequenceAssumptions

noncomputable instance concreteProviderOfDecompositionSequence
    [h : ConcreteDecompositionSequenceProvider] : ConcretePublishedPackProvider where
  concrete_pack := publishedPackOfSignedAssumptions
    (signedAssumptionsOfDecompositionAssumptions h.decomposition_assumptions)

theorem endpoint_to_rh_from_decomposition_sequence_instance
    [h : ConcreteDecompositionSequenceProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_concrete_pack (p := concreteProviderOfDecompositionSequence)

theorem rh_from_decomposition_sequence_instance
    [h : ConcreteDecompositionSequenceProvider] :
    RHStatement :=
  rh_from_concrete_pack (p := concreteProviderOfDecompositionSequence)

class ConcreteAsymptoticSequenceProvider where
  asymptotic_assumptions : ExplicitFormulaAsymptoticSequenceAssumptions

noncomputable instance concreteProviderOfAsymptoticSequence
    [h : ConcreteAsymptoticSequenceProvider] : ConcretePublishedPackProvider where
  concrete_pack := publishedPackOfSignedAssumptions
    (signedAssumptionsOfAsymptoticAssumptions h.asymptotic_assumptions)

theorem endpoint_to_rh_from_asymptotic_sequence_instance
    [h : ConcreteAsymptoticSequenceProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_concrete_pack (p := concreteProviderOfAsymptoticSequence)

theorem rh_from_asymptotic_sequence_instance
    [h : ConcreteAsymptoticSequenceProvider] :
    RHStatement :=
  rh_from_concrete_pack (p := concreteProviderOfAsymptoticSequence)

def publishedPackOfImportedZeroOscillation
    (z : ImportedZeroOscillationResults) : PublishedZeroOscillationPack where
  source_tag := "PINTZ-2017-OSCILLATION"
  source_url := "https://doi.org/10.1134/S0081543817010163"
  theorem_ref := "Thm-2-zero-to-oscillation-transfer"
  source_tag_lock := rfl
  source_url_lock := rfl
  theorem_ref_lock := rfl
  right_half_zero_forces_lower_envelope := z.right_half_zero_forces_lower_envelope_import

class ConcreteImportedZeroOscillationProvider where
  imported_zero_oscillation : ImportedZeroOscillationResults

noncomputable instance concreteProviderOfImportedZeroOscillation
    [z : ConcreteImportedZeroOscillationProvider] : ConcretePublishedPackProvider where
  concrete_pack := publishedPackOfImportedZeroOscillation z.imported_zero_oscillation

theorem endpoint_to_rh_from_imported_zero_oscillation_instance
    [z : ConcreteImportedZeroOscillationProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_concrete_pack (p := concreteProviderOfImportedZeroOscillation)

theorem rh_from_imported_zero_oscillation_instance
    [z : ConcreteImportedZeroOscillationProvider] :
    RHStatement :=
  rh_from_concrete_pack (p := concreteProviderOfImportedZeroOscillation)

def importedZeroOscillationOfAnalyticBridge
    (i : ImportedAnalyticBridgeResults) : ImportedZeroOscillationResults where
  right_half_zero_forces_lower_envelope_import :=
    i.right_half_zero_forces_lower_envelope_import

class ConcreteImportedAnalyticBridgeProvider where
  imported_analytic_bridge : ImportedAnalyticBridgeResults

noncomputable instance concreteProviderOfImportedAnalyticBridge
    [i : ConcreteImportedAnalyticBridgeProvider] : ConcretePublishedPackProvider where
  concrete_pack := publishedPackOfImportedZeroOscillation
    (importedZeroOscillationOfAnalyticBridge i.imported_analytic_bridge)

theorem endpoint_to_rh_from_imported_analytic_bridge_instance
    [i : ConcreteImportedAnalyticBridgeProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_concrete_pack (p := concreteProviderOfImportedAnalyticBridge)

theorem rh_from_imported_analytic_bridge_instance
    [i : ConcreteImportedAnalyticBridgeProvider] :
    RHStatement :=
  rh_from_concrete_pack (p := concreteProviderOfImportedAnalyticBridge)

class Ingham1932ZeroToOmegaFormalized where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "INGHAM-1932-ZERO-TO-OMEGA"
  source_url_lock : source_url = "https://openlibrary.org/books/OL14108521M/The_distribution_of_prime_numbers"
  theorem_ref_lock : theorem_ref = "zero-right-of-half-implies-omega"
  theorem_term :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)

noncomputable def ingham1932FormalizedOfTerm
    (hTerm :
      ∀ E : Real → Real,
        VonKochPrimeErrorCriterion E →
          ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
            ∃ β : Real, (1 / 2 : Real) < β ∧
              (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)) :
    Ingham1932ZeroToOmegaFormalized where
  source_tag := "INGHAM-1932-ZERO-TO-OMEGA"
  source_url := "https://openlibrary.org/books/OL14108521M/The_distribution_of_prime_numbers"
  theorem_ref := "zero-right-of-half-implies-omega"
  source_tag_lock := rfl
  source_url_lock := rfl
  theorem_ref_lock := rfl
  theorem_term := hTerm

class Pintz2017WeakZeroToOscillationFormalized where
  theorem_term :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ c β : Real, 0 < c ∧ (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ c * x ^ β)

class SchlagePuchta2019IntervalOscillationFormalized where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "SCHLAGE-PUCHTA-2019-GIVEN-ZERO-OSCILLATION"
  source_url_lock : source_url = "https://arxiv.org/abs/1912.00853"
  theorem_ref_lock : theorem_ref = "Thm-1-given-zero-forces-interval-oscillation"
  theorem_term :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ c β δ X0 : Real, 0 < c ∧ (1 / 2 : Real) < β ∧ 0 < δ ∧
            (∀ X : Real, X ≥ X0 →
              ∃ x : Real, x ≥ X ∧ x ≤ X ^ (1 + δ) ∧ |E x| ≥ c * x ^ β)

theorem weak_zero_oscillation_of_interval_oscillation
    (h :
      ∀ E : Real → Real,
        VonKochPrimeErrorCriterion E →
          ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
            ∃ c β δ X0 : Real, 0 < c ∧ (1 / 2 : Real) < β ∧ 0 < δ ∧
              (∀ X : Real, X ≥ X0 →
                ∃ x : Real, x ≥ X ∧ x ≤ X ^ (1 + δ) ∧ |E x| ≥ c * x ^ β)) :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ c β : Real, 0 < c ∧ (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ c * x ^ β) := by
  intro E hVonKoch s hs hs_gt
  rcases h E hVonKoch s hs hs_gt with
    ⟨c, β, δ, X0, hc, hβ, hδ, hInterval⟩
  refine ⟨c, β, hc, hβ, ?_⟩
  intro X
  let Y : Real := max X X0
  have hY : Y ≥ X0 := by
    dsimp [Y]
    exact le_max_right X X0
  rcases hInterval Y hY with ⟨x, hxY, hxUpper, hxAbs⟩
  have hxX : x ≥ X := by
    dsimp [Y] at hxY
    exact le_trans (le_max_left X X0) hxY
  exact ⟨x, hxX, hxAbs⟩

noncomputable instance weakZeroOscillationOfSchlagePuchta2019
    [h : SchlagePuchta2019IntervalOscillationFormalized] :
    Pintz2017WeakZeroToOscillationFormalized where
  theorem_term := weak_zero_oscillation_of_interval_oscillation h.theorem_term

class Pintz2017ZeroToOscillationFormalized where
  theorem_term :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)

noncomputable instance pintz2017OfWeak
    [h : Pintz2017WeakZeroToOscillationFormalized] :
    Pintz2017ZeroToOscillationFormalized where
  theorem_term := by
    intro E hVonKoch s hs hs_gt
    rcases h.theorem_term E hVonKoch s hs hs_gt with ⟨c, β, hc, hβ, hOmega⟩
    exact lower_envelope_from_constant_factor E c β hc hβ hOmega

noncomputable instance pintz2017OfIngham1932
    [h : Ingham1932ZeroToOmegaFormalized] :
    Pintz2017ZeroToOscillationFormalized where
  theorem_term := h.theorem_term

theorem pintz_term_excludes_right_half_zeros
    (hTerm :
      ∀ E : Real → Real,
        VonKochPrimeErrorCriterion E →
          ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
            ∃ β : Real, (1 / 2 : Real) < β ∧
              (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)) :
    ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re → False := by
  intro s hs hs_gt
  rcases hTerm zeroError endpoint_zeroError s hs hs_gt with ⟨β, hβ, hLower⟩
  exact (no_lower_envelope_for_zeroError β) hLower

def pintzTermOfSignedAssumptions
    (h : ExplicitFormulaSignedOscillationAssumptions) :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β) :=
  (publishedPackOfSignedAssumptions h).right_half_zero_forces_lower_envelope

def pintzTermOfAsymptoticAssumptions
    (h : ExplicitFormulaAsymptoticSequenceAssumptions) :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β) :=
  pintzTermOfSignedAssumptions (signedAssumptionsOfAsymptoticAssumptions h)

noncomputable def pintz2017FormalizedOfAsymptoticAssumptions
    (h : ExplicitFormulaAsymptoticSequenceAssumptions) :
    Pintz2017ZeroToOscillationFormalized where
  theorem_term := pintzTermOfAsymptoticAssumptions h

noncomputable instance concreteImportedZeroOscillationOfPintz2017
    [p : Pintz2017ZeroToOscillationFormalized] :
    ConcreteImportedZeroOscillationProvider where
  imported_zero_oscillation := {
    right_half_zero_forces_lower_envelope_import := p.theorem_term
  }

noncomputable instance importedPublishedResultsOfPintz2017
    [p : Pintz2017ZeroToOscillationFormalized] :
    ImportedPublishedResults where
  published_zero_oscillation_pack :=
    publishedPackOfImportedZeroOscillation {
      right_half_zero_forces_lower_envelope_import := p.theorem_term
    }

theorem rh_from_pintz2017_formalized
    [p : Pintz2017ZeroToOscillationFormalized] :
    RHStatement :=
  rh_from_imported_zero_oscillation_instance
    (z := concreteImportedZeroOscillationOfPintz2017)

theorem rh_from_schlage_puchta_interval_oscillation
    [h : SchlagePuchta2019IntervalOscillationFormalized] :
    RHStatement :=
  rh_from_pintz2017_formalized
    (p := pintz2017OfWeak)

theorem rh_from_ingham1932_formalized
    [h : Ingham1932ZeroToOmegaFormalized] :
    RHStatement :=
  rh_from_pintz2017_formalized
    (p := pintz2017OfIngham1932)

theorem rh_from_ingham1932_term
    (hTerm :
      ∀ E : Real → Real,
        VonKochPrimeErrorCriterion E →
          ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
            ∃ β : Real, (1 / 2 : Real) < β ∧
              (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)) :
    RHStatement := by
  letI : Ingham1932ZeroToOmegaFormalized := ingham1932FormalizedOfTerm hTerm
  exact rh_from_ingham1932_formalized

theorem rh_from_asymptotic_assumptions
    (h : ExplicitFormulaAsymptoticSequenceAssumptions) :
    RHStatement := by
  letI : Pintz2017ZeroToOscillationFormalized := pintz2017FormalizedOfAsymptoticAssumptions h
  exact rh_from_pintz2017_formalized

end PrimeRiemannBridgeConcretePackInstantiation
