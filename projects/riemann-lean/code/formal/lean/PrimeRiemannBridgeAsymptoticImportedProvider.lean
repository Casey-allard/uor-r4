import PrimeRiemannBridgeAsymptoticImportedBoundary

namespace PrimeRiemannBridgeAsymptoticImportedProvider

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeCompletionKernel
open PrimeRiemannBridgeAsymptoticImportedBoundary

/-!
Provider-facing endpoint:
instantiate one theorem-term object and obtain the endpoint=>RH theorem.
-/

class ImportedAsymptoticSequenceTheoremProvider where
  imported_theorem_term : ImportedAsymptoticSequenceTheoremTerm

def importedResultsOfProvider
    (p : ImportedAsymptoticSequenceTheoremProvider) :
    ImportedAsymptoticSequenceResults where
  theorem_term := p.imported_theorem_term

theorem endpoint_to_rh_from_provider
    (p : ImportedAsymptoticSequenceTheoremProvider) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_imported_asymptotic_results
    (r := importedResultsOfProvider p)

theorem endpoint_to_rh_from_provider_term
    (p : ImportedAsymptoticSequenceTheoremProvider) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_imported_asymptotic_theorem_term p.imported_theorem_term

def providerOfPublishedPack
    (p : PublishedZeroOscillationPack) :
    ImportedAsymptoticSequenceTheoremProvider where
  imported_theorem_term := theoremTermOfPublishedPack p

theorem endpoint_to_rh_from_published_pack_via_provider
    (p : PublishedZeroOscillationPack) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_provider (providerOfPublishedPack p)

end PrimeRiemannBridgeAsymptoticImportedProvider
