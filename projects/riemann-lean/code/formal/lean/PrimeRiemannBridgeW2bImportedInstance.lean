import PrimeRiemannBridgeW2bFinalSlot

namespace PrimeRiemannBridgeW2bImportedInstance

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeOscillatoryReduction
open PrimeRiemannBridgeW2bFinalSlot

/-!
Single integration slot for W2b:
provide one concrete `ImportedLinearPhaseOnlyResults` term and RH closes.
-/

class ConcreteW2bImportedLinearPhase where
  imported_linear_phase_only_results : ImportedLinearPhaseOnlyResults

noncomputable instance importedLinearPhaseOnlyResultsOfConcreteW2b
    [h : ConcreteW2bImportedLinearPhase] :
    ImportedLinearPhaseOnlyResults :=
  h.imported_linear_phase_only_results

theorem endpoint_to_rh_from_concrete_w2b
    [h : ConcreteW2bImportedLinearPhase] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_w2b_final_slot

theorem rh_from_concrete_w2b
    [h : ConcreteW2bImportedLinearPhase] :
    RHStatement :=
  rh_from_w2b_final_slot

noncomputable instance concreteW2bOfImportedPublishedResults
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    ConcreteW2bImportedLinearPhase where
  imported_linear_phase_only_results :=
    PrimeRiemannBridgeOscillatoryReduction.importedLinearPhaseOnlyResultsOfImportedPublished
      (r := r)

noncomputable instance importedLinearPhaseOnlyResultsOfPintz2017
    [p : PrimeRiemannBridgeConcretePackInstantiation.Pintz2017ZeroToOscillationFormalized] :
    ImportedLinearPhaseOnlyResults :=
  PrimeRiemannBridgeOscillatoryReduction.importedLinearPhaseOnlyResultsOfImportedPublished
    (r := PrimeRiemannBridgeConcretePackInstantiation.importedPublishedResultsOfPintz2017)

theorem endpoint_to_rh_from_imported_published_via_concrete_w2b
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_concrete_w2b
    (h := concreteW2bOfImportedPublishedResults)

theorem rh_from_imported_published_via_concrete_w2b
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    RHStatement :=
  rh_from_concrete_w2b
    (h := concreteW2bOfImportedPublishedResults)

theorem rh_from_pintz2017_via_concrete_w2b
    [p : PrimeRiemannBridgeConcretePackInstantiation.Pintz2017ZeroToOscillationFormalized] :
    RHStatement :=
  rh_from_imported_published_via_concrete_w2b
    (r := PrimeRiemannBridgeConcretePackInstantiation.importedPublishedResultsOfPintz2017)

theorem endpoint_to_rh_from_pintz2017_via_w2b_linear_phase_slot
    [p : PrimeRiemannBridgeConcretePackInstantiation.Pintz2017ZeroToOscillationFormalized] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_w2b_final_slot
    (h := importedLinearPhaseOnlyResultsOfPintz2017)

theorem rh_from_pintz2017_via_w2b_linear_phase_slot
    [p : PrimeRiemannBridgeConcretePackInstantiation.Pintz2017ZeroToOscillationFormalized] :
    RHStatement :=
  rh_from_w2b_final_slot
    (h := importedLinearPhaseOnlyResultsOfPintz2017)

theorem rh_from_ingham1932_via_w2b_linear_phase_slot
    [h : PrimeRiemannBridgeConcretePackInstantiation.Ingham1932ZeroToOmegaFormalized] :
    RHStatement :=
  rh_from_pintz2017_via_w2b_linear_phase_slot
    (p := PrimeRiemannBridgeConcretePackInstantiation.pintz2017OfIngham1932)

theorem endpoint_to_rh_from_pintz2017_via_imported_linear_phase_only
    [p : PrimeRiemannBridgeConcretePackInstantiation.Pintz2017ZeroToOscillationFormalized] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  PrimeRiemannBridgeOscillatoryReduction.endpoint_to_rh_from_imported_linear_phase_only_results_instance

theorem rh_from_pintz2017_via_imported_linear_phase_only
    [p : PrimeRiemannBridgeConcretePackInstantiation.Pintz2017ZeroToOscillationFormalized] :
    RHStatement :=
  PrimeRiemannBridgeOscillatoryReduction.rh_from_imported_linear_phase_only_results_instance

end PrimeRiemannBridgeW2bImportedInstance
