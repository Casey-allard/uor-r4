import PrimeRiemannBridgeInghamImportedSlot
import PrimeRiemannBridgeSpinningTopFrontier

namespace PrimeRiemannBridgeFinalTargetEquivalence

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeOscillatoryReduction
open PrimeRiemannBridgeConcretePackInstantiation
open PrimeRiemannBridgeInghamImportedSlot
open PrimeRiemannBridgeSpinningTopFrontier

abbrev Pintz2017TheoremTerm : Prop :=
  InghamImportedPayloadTerm

theorem pintz2017_theorem_term_iff_rh :
    Pintz2017TheoremTerm ↔ RHStatement := by
  simpa [Pintz2017TheoremTerm] using rh_iff_ingham_imported_payload.symm

theorem zero_to_cos_sin_phase_transfer_iff_rh :
    ZeroToCosSinPhaseTransfer ↔ RHStatement := by
  simpa [ZeroToCosSinPhaseTransfer, ZeroToCosSinPhaseTerm] using
    zero_to_cos_sin_phase_iff_rh

theorem rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized :
    RHStatement ↔ Nonempty Pintz2017ZeroToOscillationFormalized := by
  constructor
  · intro hRH
    exact ⟨{
      theorem_term := ingham_imported_payload_of_rh hRH
    }⟩
  · intro hInst
    rcases hInst with ⟨p⟩
    letI : Pintz2017ZeroToOscillationFormalized := p
    exact rh_from_pintz2017_formalized

theorem nonempty_pintz2017_zero_to_oscillation_formalized_iff_transfer :
    Nonempty Pintz2017ZeroToOscillationFormalized ↔ ZeroToCosSinPhaseTransfer := by
  constructor
  · intro hInst
    have hRH :
        RHStatement :=
      (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).2 hInst
    exact (zero_to_cos_sin_phase_transfer_iff_rh).2 hRH
  · intro hTransfer
    have hRH : RHStatement := (zero_to_cos_sin_phase_transfer_iff_rh).1 hTransfer
    exact (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).1 hRH

theorem rh_iff_nonempty_asymptotic_strict_tail_power_provider :
    RHStatement ↔ Nonempty ZeroToCosSinAsymptoticStrictTailPowerProvider := by
  constructor
  · intro hRH
    refine ⟨{ theorem_term := ?_ }⟩
    exact zero_to_cos_sin_asymptotic_strict_tail_power_of_rh hRH
  · intro hProv
    rcases hProv with ⟨p⟩
    letI : ZeroToCosSinAsymptoticStrictTailPowerProvider := p
    exact rh_from_asymptotic_strict_tail_power_provider

theorem rh_iff_nonempty_published_asymptotic_strict_tail_power_provider :
    RHStatement ↔ Nonempty PublishedAsymptoticStrictTailPowerProvider := by
  constructor
  · intro hRH
    refine ⟨{
      pack := {
        source_tag := "PINTZ-2017-OSCILLATION"
        source_url := "https://doi.org/10.1134/S0081543817010163"
        theorem_ref := "explicit-formula-zero-pair-asymptotic-strict-tail"
        source_tag_lock := rfl
        source_url_lock := rfl
        theorem_ref_lock := rfl
        asymptotic_strict_tail_power :=
          zero_to_cos_sin_asymptotic_strict_tail_power_of_rh hRH
      }
    }⟩
  · intro hProv
    rcases hProv with ⟨p⟩
    letI : PublishedAsymptoticStrictTailPowerProvider := p
    exact rh_from_published_asymptotic_strict_tail_power_provider

theorem rh_iff_nonempty_published_zero_to_cos_sin_power_majorant_provider :
    RHStatement ↔ Nonempty PublishedZeroToCosSinPowerMajorantProvider := by
  constructor
  · intro hRH
    let hAsym : ZeroToCosSinAsymptoticStrictTailPowerTerm :=
      zero_to_cos_sin_asymptotic_strict_tail_power_of_rh hRH
    let hPower : ZeroToCosSinPhasePowerMajorantTerm :=
      zero_to_cos_sin_power_majorant_of_asymptotic_strict_tail_power_term hAsym
    refine ⟨{
      pack := {
        source_tag := "PINTZ-2017-OSCILLATION"
        source_url := "https://doi.org/10.1134/S0081543817010163"
        theorem_ref := "explicit-formula-zero-pair-power-majorant-tail"
        source_tag_lock := rfl
        source_url_lock := rfl
        theorem_ref_lock := rfl
        zero_to_cos_sin_power_majorant := hPower
      }
    }⟩
  · intro hProv
    rcases hProv with ⟨p⟩
    letI : PublishedZeroToCosSinPowerMajorantProvider := p
    exact rh_from_published_zero_to_cos_sin_power_majorant_provider

end PrimeRiemannBridgeFinalTargetEquivalence
