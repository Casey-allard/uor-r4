import PrimeRiemannBridgeConcretePackInstantiation
import PrimeRiemannBridgeW2bImportedInstance
import PrimeRiemannBridgeAsymptoticImportedProvider

namespace PrimeRiemannBridgeInghamImportedSlot

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeConcretePackInstantiation
open PrimeRiemannBridgeW2bImportedInstance
open PrimeRiemannBridgeAsymptoticImportedBoundary
open PrimeRiemannBridgeAsymptoticImportedProvider

/-!
Single final import slot:
provide one Ingham-style theorem term and RH closes via both pack and W2b routes.
-/

class InghamImportedTheoremSlot where
  theorem_term :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)

noncomputable instance inghamImportedTheoremSlotOfImportedPublishedResults
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    InghamImportedTheoremSlot where
  theorem_term := r.published_zero_oscillation_pack.right_half_zero_forces_lower_envelope

noncomputable instance ingham1932FormalizedOfImportedSlot
    [h : InghamImportedTheoremSlot] :
    Ingham1932ZeroToOmegaFormalized :=
  ingham1932FormalizedOfTerm h.theorem_term

theorem rh_from_ingham_imported_slot
    [h : InghamImportedTheoremSlot] :
    RHStatement :=
  rh_from_ingham1932_formalized
    (h := ingham1932FormalizedOfImportedSlot)

theorem rh_from_ingham_imported_slot_via_w2b
    [h : InghamImportedTheoremSlot] :
    RHStatement :=
  rh_from_ingham1932_via_w2b_linear_phase_slot
    (h := ingham1932FormalizedOfImportedSlot)

theorem rh_from_imported_published_via_ingham_slot
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    RHStatement :=
  rh_from_ingham_imported_slot
    (h := inghamImportedTheoremSlotOfImportedPublishedResults)

theorem rh_from_imported_published_via_ingham_slot_w2b
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    RHStatement :=
  rh_from_ingham_imported_slot_via_w2b
    (h := inghamImportedTheoremSlotOfImportedPublishedResults)

def inghamTermOfAsymptoticImportedTerm
    (t : ImportedAsymptoticSequenceTheoremTerm) :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β) :=
  pintzTermOfAsymptoticAssumptions
    (assumptionsOfImportedTheoremTerm t)

class AsymptoticImportedTheoremSlot where
  theorem_term : ImportedAsymptoticSequenceTheoremTerm

noncomputable instance inghamImportedSlotOfAsymptoticImportedSlot
    [h : AsymptoticImportedTheoremSlot] :
    InghamImportedTheoremSlot where
  theorem_term := inghamTermOfAsymptoticImportedTerm h.theorem_term

theorem rh_from_asymptotic_imported_slot
    [h : AsymptoticImportedTheoremSlot] :
    RHStatement :=
  rh_from_ingham_imported_slot
    (h := inghamImportedSlotOfAsymptoticImportedSlot)

noncomputable instance asymptoticImportedTheoremSlotOfProvider
    [p : ImportedAsymptoticSequenceTheoremProvider] :
    AsymptoticImportedTheoremSlot where
  theorem_term := p.imported_theorem_term

theorem rh_from_asymptotic_imported_provider
    [p : ImportedAsymptoticSequenceTheoremProvider] :
    RHStatement :=
  rh_from_asymptotic_imported_slot
    (h := asymptoticImportedTheoremSlotOfProvider)

noncomputable instance asymptoticImportedProviderOfImportedPublishedResults
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    ImportedAsymptoticSequenceTheoremProvider where
  imported_theorem_term := theoremTermOfPublishedPack r.published_zero_oscillation_pack

theorem rh_from_imported_published_via_asymptotic_provider
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    RHStatement :=
  rh_from_asymptotic_imported_provider
    (p := asymptoticImportedProviderOfImportedPublishedResults)

abbrev InghamImportedPayloadTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ β : Real, (1 / 2 : Real) < β ∧
          (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)

theorem ingham_imported_payload_of_rh
    (hRH : RHStatement) :
    InghamImportedPayloadTerm := by
  intro E hVonKoch s hs hs_gt
  have _ := hVonKoch
  have hs_eq : s.re = (1 / 2 : Real) := hRH s hs
  have hFalse : False := by linarith
  exact False.elim hFalse

theorem rh_iff_ingham_imported_payload :
    RHStatement ↔ InghamImportedPayloadTerm := by
  constructor
  · intro hRH
    exact ingham_imported_payload_of_rh hRH
  · intro hPayload
    exact rh_from_ingham1932_term hPayload

noncomputable def inghamImportedSlotOfRH
    (hRH : RHStatement) :
    InghamImportedTheoremSlot where
  theorem_term := ingham_imported_payload_of_rh hRH

theorem rh_iff_nonempty_ingham_imported_slot :
    RHStatement ↔ Nonempty InghamImportedTheoremSlot := by
  constructor
  · intro hRH
    exact ⟨inghamImportedSlotOfRH hRH⟩
  · intro hSlot
    rcases hSlot with ⟨slot⟩
    letI : InghamImportedTheoremSlot := slot
    exact rh_from_ingham_imported_slot

end PrimeRiemannBridgeInghamImportedSlot
