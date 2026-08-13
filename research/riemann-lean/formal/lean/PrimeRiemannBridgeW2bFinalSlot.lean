import PrimeRiemannBridgeOscillatoryReduction

namespace PrimeRiemannBridgeW2bFinalSlot

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeOscillatoryReduction

/-!
Final W2b closure handle:
once one `ImportedLinearPhaseOnlyResults` theorem term is supplied,
the witness bridge already derives `RHStatement`.
-/

theorem endpoint_to_rh_from_w2b_final_slot
    [h : ImportedLinearPhaseOnlyResults] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_imported_linear_phase_only_results_instance

theorem rh_from_w2b_final_slot
    [h : ImportedLinearPhaseOnlyResults] :
    RHStatement :=
  rh_from_imported_linear_phase_only_results_instance

theorem rh_from_imported_published_results_via_w2b
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    RHStatement :=
  rh_from_w2b_final_slot

end PrimeRiemannBridgeW2bFinalSlot
