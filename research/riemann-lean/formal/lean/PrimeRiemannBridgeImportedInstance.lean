import PrimeRiemannBridgeImportedResults

namespace PrimeRiemannBridgeImportedInstance

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeCompletionKernel
open PrimeRiemannBridgeImportedResults

noncomputable section

/-!
Zero-axiom imported bridge:
the caller supplies a `PublishedZeroOscillationPack` term explicitly.
-/

def importedResultsOfPack (p : PublishedZeroOscillationPack) : ImportedPublishedResults where
  published_zero_oscillation_pack := p

theorem endpoint_to_rh_of_imported_published_pack
    (p : PublishedZeroOscillationPack) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_imported_results (r := importedResultsOfPack p)

end

end PrimeRiemannBridgeImportedInstance
