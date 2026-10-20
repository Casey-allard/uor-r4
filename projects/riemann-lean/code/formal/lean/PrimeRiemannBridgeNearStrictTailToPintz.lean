import PrimeRiemannBridgeSpinningTopFrontier

namespace PrimeRiemannBridgeNearStrictTailToPintz

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeConcretePackInstantiation
open PrimeRiemannBridgeSpinningTopFrontier

def pintz_term_of_near_strict_tail_power_term
    (h : ZeroToCosSinNearStrictTailPowerTerm) :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β) :=
  k1_term_of_near_strict_tail_power_term h

noncomputable def pintz2017_formalized_of_near_strict_tail_power_term
    (h : ZeroToCosSinNearStrictTailPowerTerm) :
    Pintz2017ZeroToOscillationFormalized where
  theorem_term := pintz_term_of_near_strict_tail_power_term h

class NearStrictTailPowerPintzProvider where
  theorem_term : ZeroToCosSinNearStrictTailPowerTerm

noncomputable instance pintz2017_of_near_strict_tail_power_provider
    [h : NearStrictTailPowerPintzProvider] :
    Pintz2017ZeroToOscillationFormalized where
  theorem_term := pintz_term_of_near_strict_tail_power_term h.theorem_term

theorem rh_from_near_strict_tail_power_term_via_pintz
    (h : ZeroToCosSinNearStrictTailPowerTerm) :
    RHStatement := by
  letI : Pintz2017ZeroToOscillationFormalized :=
    pintz2017_formalized_of_near_strict_tail_power_term h
  exact rh_from_pintz2017_formalized

theorem rh_from_near_strict_tail_power_provider_via_pintz
    [h : NearStrictTailPowerPintzProvider] :
    RHStatement :=
  rh_from_pintz2017_formalized
    (p := pintz2017_of_near_strict_tail_power_provider)

def pintz_term_of_asymptotic_strict_tail_power_term
    (h : ZeroToCosSinAsymptoticStrictTailPowerTerm) :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β) :=
  pintz_term_of_near_strict_tail_power_term
    (zero_to_cos_sin_near_strict_tail_of_asymptotic_strict_tail_power_term h)

noncomputable def pintz2017_formalized_of_asymptotic_strict_tail_power_term
    (h : ZeroToCosSinAsymptoticStrictTailPowerTerm) :
    Pintz2017ZeroToOscillationFormalized where
  theorem_term := pintz_term_of_asymptotic_strict_tail_power_term h

class AsymptoticStrictTailPowerPintzProvider where
  theorem_term : ZeroToCosSinAsymptoticStrictTailPowerTerm

noncomputable instance pintz2017_of_asymptotic_strict_tail_power_provider
    [h : AsymptoticStrictTailPowerPintzProvider] :
    Pintz2017ZeroToOscillationFormalized where
  theorem_term := pintz_term_of_asymptotic_strict_tail_power_term h.theorem_term

theorem rh_from_asymptotic_strict_tail_power_term_via_pintz
    (h : ZeroToCosSinAsymptoticStrictTailPowerTerm) :
    RHStatement := by
  letI : Pintz2017ZeroToOscillationFormalized :=
    pintz2017_formalized_of_asymptotic_strict_tail_power_term h
  exact rh_from_pintz2017_formalized

theorem rh_from_asymptotic_strict_tail_power_provider_via_pintz
    [h : AsymptoticStrictTailPowerPintzProvider] :
    RHStatement :=
  rh_from_pintz2017_formalized
    (p := pintz2017_of_asymptotic_strict_tail_power_provider)

noncomputable def pintz2017_formalized_of_published_asymptotic_strict_tail_power_pack
    (p : PublishedAsymptoticStrictTailPowerPack) :
    Pintz2017ZeroToOscillationFormalized where
  theorem_term :=
    pintz_term_of_asymptotic_strict_tail_power_term p.asymptotic_strict_tail_power

theorem rh_from_published_asymptotic_strict_tail_power_pack_via_pintz
    (p : PublishedAsymptoticStrictTailPowerPack) :
    RHStatement := by
  letI : Pintz2017ZeroToOscillationFormalized :=
    pintz2017_formalized_of_published_asymptotic_strict_tail_power_pack p
  exact rh_from_pintz2017_formalized

noncomputable instance pintz2017_of_published_asymptotic_strict_tail_power_provider
    [h : PublishedAsymptoticStrictTailPowerProvider] :
    Pintz2017ZeroToOscillationFormalized where
  theorem_term :=
    pintz_term_of_asymptotic_strict_tail_power_term h.pack.asymptotic_strict_tail_power

theorem rh_from_published_asymptotic_strict_tail_power_provider_via_pintz
    [h : PublishedAsymptoticStrictTailPowerProvider] :
    RHStatement :=
  rh_from_pintz2017_formalized
    (p := pintz2017_of_published_asymptotic_strict_tail_power_provider)

theorem rh_from_published_zero_to_cos_sin_power_majorant_pack_via_pintz
    (p : PublishedZeroToCosSinPowerMajorantPack) :
    RHStatement :=
  rh_from_published_asymptotic_strict_tail_power_pack_via_pintz
    (publishedAsymptoticStrictTailPowerPackOfPowerMajorantPack p)

theorem rh_from_published_zero_to_cos_sin_power_majorant_provider_via_pintz
    [h : PublishedZeroToCosSinPowerMajorantProvider] :
    RHStatement :=
  rh_from_published_zero_to_cos_sin_power_majorant_pack_via_pintz h.pack

end PrimeRiemannBridgeNearStrictTailToPintz
