import PrimeRiemannBridgeInghamImportedSlot
import PrimeRiemannBridgeOscillatoryReduction

namespace PrimeRiemannBridgeSpinningTopFrontier

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeCompletionKernel
open PrimeRiemannBridgeConcretePackInstantiation
open PrimeRiemannBridgeInghamImportedSlot
open PrimeRiemannBridgeOscillatoryReduction
open PrimeRiemannBridgeZeroOscillationProgram

/-!
Intermediate target `T` (spinning-top signed oscillation transfer):

If a right-half nontrivial zero exists under the Von-Koch criterion, then the
error term admits a signed omega lower envelope with exponent `β > 1/2`.

This is a concrete bridge target strictly about oscillatory lower envelopes.
It maps to the existing final Ingham payload and therefore to `RHStatement`.
-/

abbrev SpinningTopSignedPayloadTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ c β : Real, 0 < c ∧ (1 / 2 : Real) < β ∧
          ((∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≥ c * x ^ β) ∨
           (∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≤ - (c * x ^ β)))

theorem ingham_payload_of_spinning_top_signed_payload
    (hT : SpinningTopSignedPayloadTerm) :
    InghamImportedPayloadTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hT E hVonKoch s hs hs_gt with ⟨c, β, hc, hβ, hSigned⟩
  have hOmegaAbs : ∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ c * x ^ β := by
    rcases hSigned with hPos | hNeg
    · exact omega_abs_from_signed_pos E c β hPos
    · exact omega_abs_from_signed_neg E c β hNeg
  exact lower_envelope_from_constant_factor E c β hc hβ hOmegaAbs

theorem rh_from_spinning_top_signed_payload
    (hT : SpinningTopSignedPayloadTerm) :
    RHStatement :=
  rh_from_ingham1932_term (ingham_payload_of_spinning_top_signed_payload hT)

class SpinningTopSignedPayloadProvider where
  theorem_term : SpinningTopSignedPayloadTerm

noncomputable instance inghamImportedSlotOfSpinningTopSignedProvider
    [h : SpinningTopSignedPayloadProvider] :
    InghamImportedTheoremSlot where
  theorem_term := ingham_payload_of_spinning_top_signed_payload h.theorem_term

theorem rh_from_spinning_top_signed_provider
    [h : SpinningTopSignedPayloadProvider] :
    RHStatement :=
  rh_from_ingham_imported_slot
    (h := inghamImportedSlotOfSpinningTopSignedProvider)

theorem rh_from_spinning_top_signed_provider_via_w2b
    [h : SpinningTopSignedPayloadProvider] :
    RHStatement :=
  rh_from_ingham_imported_slot_via_w2b
    (h := inghamImportedSlotOfSpinningTopSignedProvider)

private theorem tendsto_exp_affine_nat_div_pos
    (τ off : Real) (hτ : 0 < τ) :
    Filter.Tendsto
      (fun n : Nat => Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ))
      Filter.atTop Filter.atTop := by
  have hNat : Filter.Tendsto (fun n : Nat => (n : Real)) Filter.atTop Filter.atTop :=
    tendsto_natCast_atTop_atTop
  have hMul :
      Filter.Tendsto (fun n : Nat => (n : Real) * (2 * Real.pi))
        Filter.atTop Filter.atTop :=
    hNat.atTop_mul_const' Real.two_pi_pos
  have hAdd :
      Filter.Tendsto (fun n : Nat => off + (n : Real) * (2 * Real.pi))
        Filter.atTop Filter.atTop := by
    simpa [add_comm, add_left_comm, add_assoc] using
      hMul.atTop_add
        (tendsto_const_nhds : Filter.Tendsto (fun _ : Nat => off) Filter.atTop (nhds off))
  have hDiv :
      Filter.Tendsto (fun n : Nat => (off + (n : Real) * (2 * Real.pi)) / τ)
        Filter.atTop Filter.atTop := by
    have hMulInv :
        Filter.Tendsto
          (fun n : Nat => (off + (n : Real) * (2 * Real.pi)) * τ⁻¹)
          Filter.atTop Filter.atTop :=
      hAdd.atTop_mul_const' (inv_pos.mpr hτ)
    simpa [div_eq_mul_inv] using hMulInv
  exact Real.tendsto_exp_atTop.comp hDiv

private theorem omega_abs_from_eventual_sequence
    (E : Real → Real)
    (c β : Real)
    (f : Nat → Real)
    (hTendsto : Filter.Tendsto f Filter.atTop Filter.atTop)
    (hLower : ∀ᶠ n : Nat in Filter.atTop, |E (f n)| ≥ c * (f n) ^ β) :
    ∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ c * x ^ β := by
  intro X
  have hEventuallyX : ∀ᶠ n : Nat in Filter.atTop, X ≤ f n :=
    (Filter.tendsto_atTop.1 hTendsto) X
  have hBoth :
      ∀ᶠ n : Nat in Filter.atTop, X ≤ f n ∧ |E (f n)| ≥ c * (f n) ^ β :=
    Filter.Eventually.and hEventuallyX hLower
  rcases Filter.eventually_atTop.1 hBoth with ⟨N, hN⟩
  exact ⟨f N, (hN N le_rfl).1, by simpa using (hN N le_rfl).2⟩

theorem ingham_payload_of_zero_to_cos_sin_phase
    (hZeroToCosSin :
      ∀ E : Real → Real,
        VonKochPrimeErrorCriterion E →
          ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
            ∃ β τ a b : Real, ∃ R : Real → Real,
              (1 / 2 : Real) < β ∧ 0 < τ ∧ (a ≠ 0 ∨ b ≠ 0) ∧
                (∀ x : Real,
                  E x = x ^ β *
                    (a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x)) + R x) ∧
                Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)) :
    InghamImportedPayloadTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hZeroToCosSin E hVonKoch s hs hs_gt with
    ⟨β, τ, a, b, R, hβ, hτ, hab, hDecomp, hRem⟩
  rcases hab with ha | hb
  · let c : Real := |a| / 2
    have hc : 0 < c := by
      dsimp [c]
      have ha_abs : 0 < |a| := abs_pos.mpr ha
      linarith
    let f : Nat → Real := fun n => Real.exp (((n : Real) * (2 * Real.pi)) / τ)
    have hTendstoF : Filter.Tendsto f Filter.atTop Filter.atTop :=
      by
        simpa [f, add_comm, add_left_comm, add_assoc] using
          (tendsto_exp_affine_nat_div_pos τ 0 hτ)
    have hFgeOne : ∀ᶠ n : Nat in Filter.atTop, f n ≥ 1 :=
      (Filter.tendsto_atTop.1 hTendstoF) 1
    have hRemSeq :
        Filter.Tendsto (fun n : Nat => R (f n) / (f n) ^ β) Filter.atTop (nhds 0) :=
      hRem.comp hTendstoF
    have hRemBound :
        ∀ᶠ n : Nat in Filter.atTop, |R (f n)| ≤ c * (f n) ^ β :=
      sequence_eventual_remainder_bound_of_tendsto_zero R f β c hc hRemSeq hFgeOne
    have hLowerSeq : ∀ᶠ n : Nat in Filter.atTop, |E (f n)| ≥ c * (f n) ^ β := by
      filter_upwards [hRemBound] with n hnRem
      have hτne : τ ≠ 0 := ne_of_gt hτ
      have hPhaseEq : τ * Real.log (f n) = (n : Real) * (2 * Real.pi) := by
        calc
          τ * Real.log (f n)
              = τ * (((n : Real) * (2 * Real.pi)) / τ) := by simp [f]
          _ = (n : Real) * (2 * Real.pi) := by field_simp [hτne]
      have hCos0 : Real.cos ((n : Real) * (2 * Real.pi)) = 1 := by
        simpa [mul_assoc, mul_left_comm, mul_comm] using (Real.cos_nat_mul_two_pi n)
      have hSin0 : Real.sin ((n : Real) * (2 * Real.pi)) = 0 := by
        simpa [zero_add, mul_assoc, mul_left_comm, mul_comm] using
          (Real.sin_add_nat_mul_two_pi (0 : Real) n)
      have hDecompN : E (f n) = (f n) ^ β * a + R (f n) := by
        have h := hDecomp (f n)
        rw [hPhaseEq, hCos0, hSin0] at h
        simpa [mul_assoc, add_assoc, add_comm, add_left_comm] using h
      have hpow_nonneg : 0 ≤ (f n) ^ β := by
        exact Real.rpow_nonneg (le_of_lt (by simpa [f] using Real.exp_pos (((n : Real) * (2 * Real.pi)) / τ))) β
      let m : Real := c * (f n) ^ β
      have hMainAbs : |(f n) ^ β * a| = 2 * m := by
        have hPowAbs : |(f n) ^ β| = (f n) ^ β := abs_of_nonneg hpow_nonneg
        have hTwoC : 2 * c = |a| := by
          dsimp [c]
          ring
        calc
          |(f n) ^ β * a| = |(f n) ^ β| * |a| := by rw [abs_mul]
          _ = (f n) ^ β * |a| := by rw [hPowAbs]
          _ = (2 * c) * (f n) ^ β := by rw [hTwoC]; ring
          _ = 2 * m := by
                dsimp [m]
                ring
      have hmR : |R (f n)| ≤ m := by
        simpa [m] using hnRem
      have hmSub : m ≤ |(f n) ^ β * a| - |R (f n)| := by
        rw [hMainAbs]
        linarith
      have hTri :
          |(f n) ^ β * a| - |R (f n)| ≤ |(f n) ^ β * a + R (f n)| := by
        have hAux : |(f n) ^ β * a| ≤ |(f n) ^ β * a + R (f n)| + |R (f n)| := by
          have h0 :
              |((f n) ^ β * a + R (f n)) - R (f n)|
                ≤ |(f n) ^ β * a + R (f n)| + |R (f n)| :=
            abs_sub ((f n) ^ β * a + R (f n)) (R (f n))
          simpa [sub_eq_add_neg, add_assoc, add_left_comm, add_comm] using h0
        linarith
      have hmLower : m ≤ |E (f n)| := by
        have hmMain : m ≤ |(f n) ^ β * a + R (f n)| := le_trans hmSub hTri
        simpa [hDecompN] using hmMain
      simpa [m] using hmLower
    have hOmegaAbs : ∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ c * x ^ β :=
      omega_abs_from_eventual_sequence E c β f hTendstoF hLowerSeq
    exact lower_envelope_from_constant_factor E c β hc hβ hOmegaAbs
  · let c : Real := |b| / 2
    have hc : 0 < c := by
      dsimp [c]
      have hb_abs : 0 < |b| := abs_pos.mpr hb
      linarith
    let f : Nat → Real := fun n =>
      Real.exp (((Real.pi / 2) + (n : Real) * (2 * Real.pi)) / τ)
    have hTendstoF : Filter.Tendsto f Filter.atTop Filter.atTop :=
      by
        simpa [f, add_comm, add_left_comm, add_assoc] using
          (tendsto_exp_affine_nat_div_pos τ (Real.pi / 2) hτ)
    have hFgeOne : ∀ᶠ n : Nat in Filter.atTop, f n ≥ 1 :=
      (Filter.tendsto_atTop.1 hTendstoF) 1
    have hRemSeq :
        Filter.Tendsto (fun n : Nat => R (f n) / (f n) ^ β) Filter.atTop (nhds 0) :=
      hRem.comp hTendstoF
    have hRemBound :
        ∀ᶠ n : Nat in Filter.atTop, |R (f n)| ≤ c * (f n) ^ β :=
      sequence_eventual_remainder_bound_of_tendsto_zero R f β c hc hRemSeq hFgeOne
    have hLowerSeq : ∀ᶠ n : Nat in Filter.atTop, |E (f n)| ≥ c * (f n) ^ β := by
      filter_upwards [hRemBound] with n hnRem
      have hτne : τ ≠ 0 := ne_of_gt hτ
      have hPhaseEq :
          τ * Real.log (f n) = Real.pi / 2 + (n : Real) * (2 * Real.pi) := by
        calc
          τ * Real.log (f n)
              = τ * (((Real.pi / 2) + (n : Real) * (2 * Real.pi)) / τ) := by simp [f]
          _ = Real.pi / 2 + (n : Real) * (2 * Real.pi) := by field_simp [hτne]
      have hCos0 : Real.cos (Real.pi / 2 + (n : Real) * (2 * Real.pi)) = 0 := by
        simpa [Real.cos_pi_div_two] using
          (Real.cos_add_nat_mul_two_pi (Real.pi / 2) n)
      have hSin0 : Real.sin (Real.pi / 2 + (n : Real) * (2 * Real.pi)) = 1 := by
        simpa [Real.sin_pi_div_two] using
          (Real.sin_add_nat_mul_two_pi (Real.pi / 2) n)
      have hDecompN : E (f n) = (f n) ^ β * b + R (f n) := by
        have h := hDecomp (f n)
        rw [hPhaseEq, hCos0, hSin0] at h
        simpa [mul_assoc, add_assoc, add_comm, add_left_comm] using h
      have hpow_nonneg : 0 ≤ (f n) ^ β := by
        exact Real.rpow_nonneg
          (le_of_lt (by
            dsimp [f]
            exact Real.exp_pos (((Real.pi / 2) + (n : Real) * (2 * Real.pi)) / τ))) β
      let m : Real := c * (f n) ^ β
      have hMainAbs : |(f n) ^ β * b| = 2 * m := by
        have hPowAbs : |(f n) ^ β| = (f n) ^ β := abs_of_nonneg hpow_nonneg
        have hTwoC : 2 * c = |b| := by
          dsimp [c]
          ring
        calc
          |(f n) ^ β * b| = |(f n) ^ β| * |b| := by rw [abs_mul]
          _ = (f n) ^ β * |b| := by rw [hPowAbs]
          _ = (2 * c) * (f n) ^ β := by rw [hTwoC]; ring
          _ = 2 * m := by
                dsimp [m]
                ring
      have hmR : |R (f n)| ≤ m := by
        simpa [m] using hnRem
      have hmSub : m ≤ |(f n) ^ β * b| - |R (f n)| := by
        rw [hMainAbs]
        linarith
      have hTri :
          |(f n) ^ β * b| - |R (f n)| ≤ |(f n) ^ β * b + R (f n)| := by
        have hAux : |(f n) ^ β * b| ≤ |(f n) ^ β * b + R (f n)| + |R (f n)| := by
          have h0 :
              |((f n) ^ β * b + R (f n)) - R (f n)|
                ≤ |(f n) ^ β * b + R (f n)| + |R (f n)| :=
            abs_sub ((f n) ^ β * b + R (f n)) (R (f n))
          simpa [sub_eq_add_neg, add_assoc, add_left_comm, add_comm] using h0
        linarith
      have hmLower : m ≤ |E (f n)| := by
        have hmMain : m ≤ |(f n) ^ β * b + R (f n)| := le_trans hmSub hTri
        simpa [hDecompN] using hmMain
      simpa [m] using hmLower
    have hOmegaAbs : ∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ c * x ^ β :=
      omega_abs_from_eventual_sequence E c β f hTendstoF hLowerSeq
    exact lower_envelope_from_constant_factor E c β hc hβ hOmegaAbs

/--
K1-form decomposition target:
right-half zeros force a cosine/sine explicit-formula decomposition with
vanishing normalized remainder.
-/
abbrev ZeroToCosSinPhaseTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ β τ a b : Real, ∃ R : Real → Real,
          (1 / 2 : Real) < β ∧ 0 < τ ∧ (a ≠ 0 ∨ b ≠ 0) ∧
            (∀ x : Real,
              E x = x ^ β *
                (a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x)) + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)

theorem k1_term_of_zero_to_cos_sin_phase
    (hZeroToCosSin : ZeroToCosSinPhaseTerm) :
    InghamImportedPayloadTerm :=
  ingham_payload_of_zero_to_cos_sin_phase hZeroToCosSin

theorem rh_from_zero_to_cos_sin_phase
    (hZeroToCosSin : ZeroToCosSinPhaseTerm) :
    RHStatement :=
  rh_from_ingham1932_term (k1_term_of_zero_to_cos_sin_phase hZeroToCosSin)

theorem zero_to_cos_sin_phase_of_rh
    (hRH : RHStatement) :
    ZeroToCosSinPhaseTerm := by
  intro E hVonKoch s hs hs_gt
  have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
  have hFalse : False := by linarith
  exact False.elim hFalse

theorem zero_to_cos_sin_phase_of_root_rh
    (hRH : _root_.RiemannHypothesis) :
    ZeroToCosSinPhaseTerm :=
  zero_to_cos_sin_phase_of_rh (rhStatement_of_root_rh hRH)

theorem zero_to_cos_sin_phase_iff_rh :
    ZeroToCosSinPhaseTerm ↔ RHStatement := by
  constructor
  · exact rh_from_zero_to_cos_sin_phase
  · exact zero_to_cos_sin_phase_of_rh

theorem rh_from_root_rh_via_zero_to_cos_sin_phase
    (hRH : _root_.RiemannHypothesis) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase (zero_to_cos_sin_phase_of_root_rh hRH)

/-!
Quantified-majorant refinement of the K1 source frontier:
it suffices to dominate the normalized remainder by a majorant
that tends to zero at `atTop`.
-/
abbrev ZeroToCosSinPhaseMajorantTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ β τ a b : Real, ∃ R G : Real → Real,
          (1 / 2 : Real) < β ∧ 0 < τ ∧ (a ≠ 0 ∨ b ≠ 0) ∧
            (∀ x : Real,
              E x = x ^ β *
                (a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x)) + R x) ∧
            (∀ᶠ x : Real in Filter.atTop, |R x / x ^ β| ≤ G x) ∧
            Filter.Tendsto G Filter.atTop (nhds 0)

theorem zero_to_cos_sin_phase_of_majorant_term
    (hMajorant : ZeroToCosSinPhaseMajorantTerm) :
    ZeroToCosSinPhaseTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hMajorant E hVonKoch s hs hs_gt with
    ⟨β, τ, a, b, R, G, hβ, hτ, hab, hDecomp, hAbsMajorized, hGTendsto⟩
  refine ⟨β, τ, a, b, R, hβ, hτ, hab, hDecomp, ?_⟩
  have hLowerTendsto :
      Filter.Tendsto (fun x : Real => -G x) Filter.atTop (nhds 0) := by
    simpa using hGTendsto.neg
  have hLowerBound :
      ∀ᶠ x : Real in Filter.atTop, -G x ≤ R x / x ^ β := by
    filter_upwards [hAbsMajorized] with x hx
    exact (abs_le.mp hx).1
  have hUpperBound :
      ∀ᶠ x : Real in Filter.atTop, R x / x ^ β ≤ G x := by
    filter_upwards [hAbsMajorized] with x hx
    exact (abs_le.mp hx).2
  exact tendsto_of_tendsto_of_tendsto_of_le_of_le'
    hLowerTendsto hGTendsto hLowerBound hUpperBound

theorem k1_term_of_zero_to_cos_sin_majorant
    (hMajorant : ZeroToCosSinPhaseMajorantTerm) :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_of_majorant_term hMajorant)

theorem rh_from_zero_to_cos_sin_majorant
    (hMajorant : ZeroToCosSinPhaseMajorantTerm) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_of_majorant_term hMajorant)

/-!
Power-majorant concrete refinement of the K1 source frontier:
an explicit tail law `|R(x)/x^β| ≤ C * x^{-η}` for `x ≥ x0`, with `η > 0`,
implies the majorant form and thus the K1 source term.
-/
abbrev ZeroToCosSinPhasePowerMajorantTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ β τ a b : Real, ∃ R : Real → Real,
          ∃ C η x0 : Real,
            (1 / 2 : Real) < β ∧ 0 < τ ∧ (a ≠ 0 ∨ b ≠ 0) ∧
            0 ≤ C ∧ 0 < η ∧
            (∀ x : Real,
              E x = x ^ β *
                (a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x)) + R x) ∧
            (∀ x : Real, x ≥ x0 → |R x / x ^ β| ≤ C * x ^ (-η))

theorem zero_to_cos_sin_majorant_of_power_majorant_term
    (hPower : ZeroToCosSinPhasePowerMajorantTerm) :
    ZeroToCosSinPhaseMajorantTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hPower E hVonKoch s hs hs_gt with
    ⟨β, τ, a, b, R, C, η, x0, hβ, hτ, hab, hC, hη, hDecomp, hBoundTail⟩
  refine ⟨β, τ, a, b, R, (fun x : Real => C * x ^ (-η)),
    hβ, hτ, hab, hDecomp, ?_, ?_⟩
  · exact Filter.eventually_atTop.2 ⟨x0, fun x hx => hBoundTail x hx⟩
  · have hPow : Filter.Tendsto (fun x : Real => x ^ (-η)) Filter.atTop (nhds 0) :=
      tendsto_rpow_neg_atTop hη
    have hMul :
        Filter.Tendsto (fun x : Real => C * x ^ (-η))
          Filter.atTop (nhds (C * 0)) :=
      (tendsto_const_nhds : Filter.Tendsto (fun _ : Real => C) Filter.atTop (nhds C)).mul hPow
    simpa using hMul

theorem zero_to_cos_sin_phase_of_power_majorant_term
    (hPower : ZeroToCosSinPhasePowerMajorantTerm) :
    ZeroToCosSinPhaseTerm :=
  zero_to_cos_sin_phase_of_majorant_term
    (zero_to_cos_sin_majorant_of_power_majorant_term hPower)

theorem k1_term_of_zero_to_cos_sin_power_majorant
    (hPower : ZeroToCosSinPhasePowerMajorantTerm) :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_of_power_majorant_term hPower)

theorem rh_from_zero_to_cos_sin_power_majorant
    (hPower : ZeroToCosSinPhasePowerMajorantTerm) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_of_power_majorant_term hPower)

/-!
Near-strict finite-tail strengthening of the power-majorant frontier:
in addition to a power-law bound, the normalized remainder is eventually
dominated by a strict fraction (`ρ < 1`) of the main oscillatory amplitude.
-/
abbrev ZeroToCosSinNearStrictTailPowerTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ β τ a b : Real, ∃ R : Real → Real,
          ∃ C η x0 ρ : Real,
            (1 / 2 : Real) < β ∧ 0 < τ ∧ (a ≠ 0 ∨ b ≠ 0) ∧
            0 ≤ C ∧ 0 < η ∧ 0 < ρ ∧ ρ < 1 ∧
            (∀ x : Real,
              E x = x ^ β *
                (a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x)) + R x) ∧
            (∀ x : Real, x ≥ x0 → |R x / x ^ β| ≤ ρ * Real.sqrt (a ^ 2 + b ^ 2)) ∧
            (∀ x : Real, x ≥ x0 → |R x / x ^ β| ≤ C * x ^ (-η))

/-!
Asymptotic strict-tail strengthening: for every `ε > 0`, the normalized
remainder is eventually bounded by `ε * amplitude`.
-/
abbrev ZeroToCosSinAsymptoticStrictTailPowerTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ β τ a b : Real, ∃ R : Real → Real,
          ∃ C η x0 : Real,
            (1 / 2 : Real) < β ∧ 0 < τ ∧ (a ≠ 0 ∨ b ≠ 0) ∧
            0 ≤ C ∧ 0 < η ∧
            (∀ x : Real,
              E x = x ^ β *
                (a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x)) + R x) ∧
            (∀ ε : Real, ε > 0 →
              ∃ xε : Real, ∀ x : Real, x ≥ xε →
                |R x / x ^ β| ≤ ε * Real.sqrt (a ^ 2 + b ^ 2)) ∧
            (∀ x : Real, x ≥ x0 → |R x / x ^ β| ≤ C * x ^ (-η))

theorem zero_to_cos_sin_asymptotic_strict_tail_power_of_power_majorant_term
    (hPower : ZeroToCosSinPhasePowerMajorantTerm) :
    ZeroToCosSinAsymptoticStrictTailPowerTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hPower E hVonKoch s hs hs_gt with
    ⟨β, τ, a, b, R, C, η, x0, hβ, hτ, hab, hC, hη, hDecomp, hPowerTail⟩
  have hAmpSqPos : 0 < a ^ 2 + b ^ 2 := by
    rcases hab with ha | hb
    · have ha2 : 0 < a ^ 2 := sq_pos_of_ne_zero ha
      have hb2 : 0 ≤ b ^ 2 := sq_nonneg b
      linarith
    · have ha2 : 0 ≤ a ^ 2 := sq_nonneg a
      have hb2 : 0 < b ^ 2 := sq_pos_of_ne_zero hb
      linarith
  have hAmpPos : 0 < Real.sqrt (a ^ 2 + b ^ 2) :=
    Real.sqrt_pos.2 hAmpSqPos
  refine ⟨β, τ, a, b, R, C, η, x0, hβ, hτ, hab, hC, hη, hDecomp, ?_, hPowerTail⟩
  intro ε hε
  have hPow : Filter.Tendsto (fun x : Real => x ^ (-η)) Filter.atTop (nhds 0) :=
    tendsto_rpow_neg_atTop hη
  have hMul :
      Filter.Tendsto (fun x : Real => C * x ^ (-η)) Filter.atTop (nhds (C * 0)) :=
    (tendsto_const_nhds : Filter.Tendsto (fun _ : Real => C) Filter.atTop (nhds C)).mul hPow
  have hTailTendsto :
      Filter.Tendsto (fun x : Real => C * x ^ (-η)) Filter.atTop (nhds 0) := by
    simpa using hMul
  have hEpsAmpPos : 0 < ε * Real.sqrt (a ^ 2 + b ^ 2) :=
    mul_pos hε hAmpPos
  have hEventuallyLt :
      ∀ᶠ x : Real in Filter.atTop,
        C * x ^ (-η) < ε * Real.sqrt (a ^ 2 + b ^ 2) :=
    hTailTendsto (Iio_mem_nhds hEpsAmpPos)
  have hEventuallyX0 : ∀ᶠ x : Real in Filter.atTop, x ≥ x0 :=
    Filter.eventually_atTop.2 ⟨x0, fun x hx => hx⟩
  have hEventuallyBound :
      ∀ᶠ x : Real in Filter.atTop,
        |R x / x ^ β| ≤ ε * Real.sqrt (a ^ 2 + b ^ 2) := by
    filter_upwards [hEventuallyX0, hEventuallyLt] with x hx0 hxlt
    exact le_trans (hPowerTail x hx0) (le_of_lt hxlt)
  rcases Filter.eventually_atTop.1 hEventuallyBound with ⟨xε, hxε⟩
  exact ⟨xε, hxε⟩

theorem zero_to_cos_sin_near_strict_tail_of_asymptotic_strict_tail_power_term
    (hAsym : ZeroToCosSinAsymptoticStrictTailPowerTerm) :
    ZeroToCosSinNearStrictTailPowerTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hAsym E hVonKoch s hs hs_gt with
    ⟨β, τ, a, b, R, C, η, x0, hβ, hτ, hab, hC, hη, hDecomp, hAsymTail, hPowerTail⟩
  have hHalfPos : (0 : Real) < (1 / 2 : Real) := by norm_num
  rcases hAsymTail (1 / 2 : Real) hHalfPos with ⟨xε, hNearTailAtHalf⟩
  refine ⟨β, τ, a, b, R, C, η, max x0 xε, (1 / 2 : Real), hβ, hτ, hab, hC, hη, hHalfPos, ?_,
    hDecomp, ?_, ?_⟩
  · norm_num
  · intro x hx
    have hxε : x ≥ xε := le_trans (le_max_right x0 xε) hx
    exact hNearTailAtHalf x hxε
  · intro x hx
    have hx0 : x ≥ x0 := le_trans (le_max_left x0 xε) hx
    exact hPowerTail x hx0

theorem zero_to_cos_sin_power_majorant_of_near_strict_tail_power_term
    (hNear : ZeroToCosSinNearStrictTailPowerTerm) :
    ZeroToCosSinPhasePowerMajorantTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hNear E hVonKoch s hs hs_gt with
    ⟨β, τ, a, b, R, C, η, x0, ρ, hβ, hτ, hab, hC, hη, hρ, hρlt,
      hDecomp, hNearTail, hPowerTail⟩
  have _ : 0 < ρ := hρ
  have _ : ρ < 1 := hρlt
  have _ : ∀ x : Real, x ≥ x0 → |R x / x ^ β| ≤ ρ * Real.sqrt (a ^ 2 + b ^ 2) := hNearTail
  exact ⟨β, τ, a, b, R, C, η, x0, hβ, hτ, hab, hC, hη, hDecomp, hPowerTail⟩

theorem zero_to_cos_sin_power_majorant_of_asymptotic_strict_tail_power_term
    (hAsym : ZeroToCosSinAsymptoticStrictTailPowerTerm) :
    ZeroToCosSinPhasePowerMajorantTerm :=
  zero_to_cos_sin_power_majorant_of_near_strict_tail_power_term
    (zero_to_cos_sin_near_strict_tail_of_asymptotic_strict_tail_power_term hAsym)

theorem zero_to_cos_sin_phase_of_near_strict_tail_power_term
    (hNear : ZeroToCosSinNearStrictTailPowerTerm) :
    ZeroToCosSinPhaseTerm :=
  zero_to_cos_sin_phase_of_power_majorant_term
    (zero_to_cos_sin_power_majorant_of_near_strict_tail_power_term hNear)

theorem k1_term_of_near_strict_tail_power_term
    (hNear : ZeroToCosSinNearStrictTailPowerTerm) :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_of_near_strict_tail_power_term hNear)

theorem rh_from_near_strict_tail_power_term
    (hNear : ZeroToCosSinNearStrictTailPowerTerm) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_of_near_strict_tail_power_term hNear)

theorem zero_to_cos_sin_phase_of_asymptotic_strict_tail_power_term
    (hAsym : ZeroToCosSinAsymptoticStrictTailPowerTerm) :
    ZeroToCosSinPhaseTerm :=
  zero_to_cos_sin_phase_of_near_strict_tail_power_term
    (zero_to_cos_sin_near_strict_tail_of_asymptotic_strict_tail_power_term hAsym)

theorem k1_term_of_asymptotic_strict_tail_power_term
    (hAsym : ZeroToCosSinAsymptoticStrictTailPowerTerm) :
    InghamImportedPayloadTerm :=
  k1_term_of_near_strict_tail_power_term
    (zero_to_cos_sin_near_strict_tail_of_asymptotic_strict_tail_power_term hAsym)

theorem rh_from_asymptotic_strict_tail_power_term
    (hAsym : ZeroToCosSinAsymptoticStrictTailPowerTerm) :
    RHStatement :=
  rh_from_near_strict_tail_power_term
    (zero_to_cos_sin_near_strict_tail_of_asymptotic_strict_tail_power_term hAsym)

theorem zero_to_cos_sin_asymptotic_strict_tail_power_of_rh
    (hRH : RHStatement) :
    ZeroToCosSinAsymptoticStrictTailPowerTerm := by
  intro E hVonKoch s hs hs_gt
  have _ := hVonKoch
  have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
  have hFalse : False := by linarith
  exact False.elim hFalse

theorem asymptotic_strict_tail_power_iff_rh :
    ZeroToCosSinAsymptoticStrictTailPowerTerm ↔ RHStatement := by
  constructor
  · exact rh_from_asymptotic_strict_tail_power_term
  · exact zero_to_cos_sin_asymptotic_strict_tail_power_of_rh

/-!
Research-side R^6 dual-band contract:

This packages the project's spinning-top/R6 finite-mode picture into one
formal source term. It is intentionally stronger than the power-majorant
frontier by carrying explicit six-mode metadata.
-/
abbrev R6DualBandPowerMajorantFittingTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ β τ a b : Real, ∃ R : Real → Real,
          ∃ C η x0 : Real, ∃ modes : List DecayingPhaseMode, ∃ i0 : Nat,
            (1 / 2 : Real) < β ∧ 0 < τ ∧ (a ≠ 0 ∨ b ≠ 0) ∧
            0 ≤ C ∧ 0 < η ∧ 6 ≤ modes.length ∧ i0 < modes.length ∧
            (∀ x : Real,
              E x = x ^ β *
                (a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x)) + R x) ∧
            (∀ x : Real, x ≥ x0 → |R x / x ^ β| ≤ C * x ^ (-η))

/-!
Piecewise majorant witness for the R^6 dual-band frontier:
the same decomposition parameters carry a proven finite-window bound plus an
asymptotic tail lock with shared `(C, η)`.

This is the direct formal slot for the project's empirical-window research
combined with the remaining asymptotic theorem obligation.
-/
abbrev R6DualBandPiecewisePowerMajorantWitnessTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ β τ a b : Real, ∃ R : Real → Real,
          ∃ C η x0 x1 : Real, ∃ modes : List DecayingPhaseMode, ∃ i0 : Nat,
            (1 / 2 : Real) < β ∧ 0 < τ ∧ (a ≠ 0 ∨ b ≠ 0) ∧
            0 ≤ C ∧ 0 < η ∧ 6 ≤ modes.length ∧ i0 < modes.length ∧ x0 ≤ x1 ∧
            (∀ x : Real,
              E x = x ^ β *
                (a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x)) + R x) ∧
            (∀ x : Real, x ≥ x0 → x ≤ x1 → |R x / x ^ β| ≤ C * x ^ (-η)) ∧
            (∀ x : Real, x ≥ x1 → |R x / x ^ β| ≤ C * x ^ (-η))

/-!
Shared R6 model pack for decomposition/tail constants. This isolates the final
burn-down gate: once finite-window and asymptotic tail bounds are both proven
for the same pack, the piecewise witness is immediate.
-/
structure R6DualBandModelPack (E : Real → Real) where
  β : Real
  τ : Real
  a : Real
  b : Real
  R : Real → Real
  C : Real
  η : Real
  x0 : Real
  x1 : Real
  modes : List DecayingPhaseMode
  i0 : Nat
  beta_gt_half : (1 / 2 : Real) < β
  tau_pos : 0 < τ
  main_nontrivial : (a ≠ 0 ∨ b ≠ 0)
  c_nonneg : 0 ≤ C
  eta_pos : 0 < η
  modes_len_ge_6 : 6 ≤ modes.length
  dominant_index_valid : i0 < modes.length
  x0_le_x1 : x0 ≤ x1
  decomp :
    ∀ x : Real,
      E x =
        x ^ β * (a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x)) + R x

def r6DualBandFiniteWindowBound
    {E : Real → Real}
    (w : R6DualBandModelPack E) : Prop :=
  ∀ x : Real, x ≥ w.x0 → x ≤ w.x1 → |w.R x / x ^ w.β| ≤ w.C * x ^ (-w.η)

def r6DualBandAsymptoticTailBound
    {E : Real → Real}
    (w : R6DualBandModelPack E) : Prop :=
  ∀ x : Real, x ≥ w.x1 → |w.R x / x ^ w.β| ≤ w.C * x ^ (-w.η)

/-!
Omitted-mode tail package for the final R6 gate:
represent the normalized remainder by a named tail function and bound it
directly by the shared power envelope on `x >= x1`.
-/
structure R6DualBandOmittedModeTailPack
    {E : Real → Real}
    (w : R6DualBandModelPack E) where
  tail_fn : Real → Real
  tail_repr :
    ∀ x : Real, x ≥ w.x1 → w.R x / x ^ w.β = tail_fn x
  tail_majorant :
    ∀ x : Real, x ≥ w.x1 → |tail_fn x| ≤ w.C * x ^ (-w.η)

def decayingPhaseModeCoeffL1 (modes : List DecayingPhaseMode) : Real :=
  modes.foldr (fun m acc => |m.κ| + acc) 0

private theorem abs_decaying_phase_mode_term_le_model_eta
    (m : DecayingPhaseMode) {η : Real} (hη : η ≤ m.η) {x : Real} (hx : 1 ≤ x) :
    |decayingPhaseModeTerm m x| ≤ |m.κ| * x ^ (-η) := by
  have hx_nonneg : 0 ≤ x := by linarith
  have hpow_nonneg : 0 ≤ x ^ (-m.η) := Real.rpow_nonneg hx_nonneg (-m.η)
  have hAbsPow : |x ^ (-m.η)| = x ^ (-m.η) := abs_of_nonneg hpow_nonneg
  have hSinAbsLeOne : |Real.sin (m.ω * Real.log x + m.θ)| ≤ 1 := Real.abs_sin_le_one _
  have hBase :
      |decayingPhaseModeTerm m x| ≤ |m.κ| * x ^ (-m.η) := by
    calc
      |decayingPhaseModeTerm m x|
          = |m.κ| * |x ^ (-m.η)| * |Real.sin (m.ω * Real.log x + m.θ)| := by
              simp [decayingPhaseModeTerm, abs_mul, mul_assoc]
      _ = |m.κ| * x ^ (-m.η) * |Real.sin (m.ω * Real.log x + m.θ)| := by
            rw [hAbsPow]
      _ ≤ |m.κ| * x ^ (-m.η) * 1 := by
            exact mul_le_mul_of_nonneg_left hSinAbsLeOne
              (mul_nonneg (abs_nonneg _) hpow_nonneg)
      _ = |m.κ| * x ^ (-m.η) := by ring
  have hExp : x ^ (-m.η) ≤ x ^ (-η) := by
    have hneg : -m.η ≤ -η := by linarith
    exact Real.rpow_le_rpow_of_exponent_le hx hneg
  have hMul :
      |m.κ| * x ^ (-m.η) ≤ |m.κ| * x ^ (-η) :=
    mul_le_mul_of_nonneg_left hExp (abs_nonneg _)
  exact le_trans hBase hMul

theorem abs_decaying_phase_mode_list_correction_le_l1
    (modes : List DecayingPhaseMode)
    (η : Real)
    (hη : ∀ m : DecayingPhaseMode, m ∈ modes → η ≤ m.η)
    {x : Real}
    (hx : 1 ≤ x) :
    |decayingPhaseModeListCorrection modes x| ≤
      decayingPhaseModeCoeffL1 modes * x ^ (-η) := by
  induction modes with
  | nil =>
      simp [decayingPhaseModeListCorrection, decayingPhaseModeCoeffL1]
  | cons m ms ih =>
      have hηm : η ≤ m.η := hη m (by simp)
      have hηms : ∀ m' : DecayingPhaseMode, m' ∈ ms → η ≤ m'.η := by
        intro m' hm'
        exact hη m' (by simp [hm'])
      have hHead :
          |decayingPhaseModeTerm m x| ≤ |m.κ| * x ^ (-η) :=
        abs_decaying_phase_mode_term_le_model_eta m hηm hx
      have hTail :
          |decayingPhaseModeListCorrection ms x| ≤
            decayingPhaseModeCoeffL1 ms * x ^ (-η) :=
        ih hηms
      have hTri :
          |decayingPhaseModeTerm m x + decayingPhaseModeListCorrection ms x| ≤
            |decayingPhaseModeTerm m x| + |decayingPhaseModeListCorrection ms x| := by
        simpa using
          (abs_add_three (decayingPhaseModeTerm m x)
            (decayingPhaseModeListCorrection ms x) (0 : Real))
      calc
        |decayingPhaseModeListCorrection (m :: ms) x|
            = |decayingPhaseModeTerm m x + decayingPhaseModeListCorrection ms x| := by
                simp [decayingPhaseModeListCorrection]
        _ ≤ |decayingPhaseModeTerm m x| + |decayingPhaseModeListCorrection ms x| :=
              hTri
        _ ≤ (|m.κ| * x ^ (-η)) + (decayingPhaseModeCoeffL1 ms * x ^ (-η)) :=
              add_le_add hHead hTail
        _ = (|m.κ| + decayingPhaseModeCoeffL1 ms) * x ^ (-η) := by ring
        _ = decayingPhaseModeCoeffL1 (m :: ms) * x ^ (-η) := by
              simp [decayingPhaseModeCoeffL1]

structure R6DualBandFiniteOmittedModeTailAssumptions
    {E : Real → Real}
    (w : R6DualBandModelPack E) where
  omitted_modes : List DecayingPhaseMode
  model_eta_le_omitted : ∀ m : DecayingPhaseMode, m ∈ omitted_modes → w.η ≤ m.η
  x1_ge_one : 1 ≤ w.x1
  tail_eq_omitted :
    ∀ x : Real, x ≥ w.x1 →
      w.R x / x ^ w.β = decayingPhaseModeListCorrection omitted_modes x
  coeff_l1_bound : decayingPhaseModeCoeffL1 omitted_modes ≤ w.C

noncomputable def omitted_mode_tail_pack_of_finite_omitted_modes
    {E : Real → Real}
    (w : R6DualBandModelPack E)
    (hFinite : R6DualBandFiniteOmittedModeTailAssumptions w) :
    R6DualBandOmittedModeTailPack w := by
  refine {
    tail_fn := decayingPhaseModeListCorrection hFinite.omitted_modes
    tail_repr := ?_
    tail_majorant := ?_
  }
  · intro x hx
    exact hFinite.tail_eq_omitted x hx
  · intro x hx
    have hx_ge_one : 1 ≤ x := le_trans hFinite.x1_ge_one hx
    have hL1 :
        |decayingPhaseModeListCorrection hFinite.omitted_modes x| ≤
          decayingPhaseModeCoeffL1 hFinite.omitted_modes * x ^ (-w.η) :=
      abs_decaying_phase_mode_list_correction_le_l1
        hFinite.omitted_modes w.η hFinite.model_eta_le_omitted hx_ge_one
    have hx_nonneg : 0 ≤ x := by linarith
    have hPowNonneg : 0 ≤ x ^ (-w.η) := Real.rpow_nonneg hx_nonneg (-w.η)
    have hCoeffScaled :
        decayingPhaseModeCoeffL1 hFinite.omitted_modes * x ^ (-w.η) ≤
          w.C * x ^ (-w.η) :=
      mul_le_mul_of_nonneg_right hFinite.coeff_l1_bound hPowNonneg
    exact le_trans hL1 hCoeffScaled

theorem r6_dual_band_asymptotic_tail_bound_of_omitted_mode_tail_pack
    {E : Real → Real}
    (w : R6DualBandModelPack E)
    (hTail : R6DualBandOmittedModeTailPack w) :
    r6DualBandAsymptoticTailBound w := by
  intro x hx
  have hrepr : w.R x / x ^ w.β = hTail.tail_fn x := hTail.tail_repr x hx
  have hmaj : |hTail.tail_fn x| ≤ w.C * x ^ (-w.η) := hTail.tail_majorant x hx
  simpa [hrepr] using hmaj

abbrev R6DualBandWitnessWithWindowAndFiniteOmittedModesTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ w : R6DualBandModelPack E,
          r6DualBandFiniteWindowBound w ∧
          Nonempty (R6DualBandFiniteOmittedModeTailAssumptions w)

abbrev R6DualBandWitnessWithWindowAndOmittedTailTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ w : R6DualBandModelPack E,
          r6DualBandFiniteWindowBound w ∧
          Nonempty (R6DualBandOmittedModeTailPack w)

theorem r6_dual_band_witness_with_window_and_omitted_tail_of_finite_omitted_modes
    (hFinite : R6DualBandWitnessWithWindowAndFiniteOmittedModesTerm) :
    R6DualBandWitnessWithWindowAndOmittedTailTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hFinite E hVonKoch s hs hs_gt with ⟨w, hWindow, hFinitePack⟩
  rcases hFinitePack with ⟨finitePack⟩
  refine ⟨w, hWindow, ?_⟩
  exact ⟨omitted_mode_tail_pack_of_finite_omitted_modes w finitePack⟩

abbrev R6DualBandTailRepresentationKernelTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ w : R6DualBandModelPack E,
          r6DualBandFiniteWindowBound w ∧
          ∃ omitted_modes : List DecayingPhaseMode,
            (∀ m : DecayingPhaseMode, m ∈ omitted_modes → w.η ≤ m.η) ∧
            1 ≤ w.x1 ∧
            (∀ x : Real, x ≥ w.x1 →
              w.R x / x ^ w.β = decayingPhaseModeListCorrection omitted_modes x) ∧
            decayingPhaseModeCoeffL1 omitted_modes ≤ w.C

theorem r6_dual_band_witness_with_window_and_finite_omitted_modes_of_tail_representation_kernel
    (hKernel : R6DualBandTailRepresentationKernelTerm) :
    R6DualBandWitnessWithWindowAndFiniteOmittedModesTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hKernel E hVonKoch s hs hs_gt with
    ⟨w, hWindow, omitted_modes, hEta, hx1, hTailEq, hCoeff⟩
  refine ⟨w, hWindow, ?_⟩
  refine ⟨{
    omitted_modes := omitted_modes
    model_eta_le_omitted := hEta
    x1_ge_one := hx1
    tail_eq_omitted := hTailEq
    coeff_l1_bound := hCoeff
  }⟩

theorem r6_dual_band_tail_representation_kernel_of_witness_with_window_and_finite_omitted_modes
    (hFinite : R6DualBandWitnessWithWindowAndFiniteOmittedModesTerm) :
    R6DualBandTailRepresentationKernelTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hFinite E hVonKoch s hs hs_gt with ⟨w, hWindow, hFinitePack⟩
  rcases hFinitePack with ⟨finitePack⟩
  exact ⟨w, hWindow, finitePack.omitted_modes, finitePack.model_eta_le_omitted,
    finitePack.x1_ge_one, finitePack.tail_eq_omitted, finitePack.coeff_l1_bound⟩

theorem r6_dual_band_tail_representation_kernel_iff_witness_with_window_and_finite_omitted_modes :
    R6DualBandTailRepresentationKernelTerm ↔
      R6DualBandWitnessWithWindowAndFiniteOmittedModesTerm := by
  constructor
  · exact r6_dual_band_witness_with_window_and_finite_omitted_modes_of_tail_representation_kernel
  · exact r6_dual_band_tail_representation_kernel_of_witness_with_window_and_finite_omitted_modes

abbrev R6DualBandTailRepresentationCandidateShapeTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ w : R6DualBandModelPack E,
          r6DualBandFiniteWindowBound w ∧
          w.η = (1 / 100 : Real) ∧
          w.x1 = (10 : Real) ^ (21 : Nat) ∧
          1 ≤ w.x1 ∧
          ∃ omitted_modes : List DecayingPhaseMode,
            omitted_modes.length = 256 ∧
            (∀ m : DecayingPhaseMode, m ∈ omitted_modes → w.η ≤ m.η) ∧
            (∀ x : Real, x ≥ w.x1 →
              w.R x / x ^ w.β = decayingPhaseModeListCorrection omitted_modes x) ∧
            decayingPhaseModeCoeffL1 omitted_modes ≤ w.C

theorem r6_dual_band_tail_representation_kernel_of_candidate_shape_term
    (hCandidate : R6DualBandTailRepresentationCandidateShapeTerm) :
    R6DualBandTailRepresentationKernelTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hCandidate E hVonKoch s hs hs_gt with
    ⟨w, hWindow, hEta, hx1, hx1_ge_one, omitted_modes, hLen, hEtaLe, hTailEq, hCoeff⟩
  have _ : omitted_modes.length = 256 := hLen
  have _ : w.η = (1 / 100 : Real) := hEta
  have _ : w.x1 = (10 : Real) ^ (21 : Nat) := hx1
  exact ⟨w, hWindow, omitted_modes, hEtaLe, hx1_ge_one, hTailEq, hCoeff⟩

abbrev R6DualBandWitnessWithWindowAndTailTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ w : R6DualBandModelPack E,
          r6DualBandFiniteWindowBound w ∧
          r6DualBandAsymptoticTailBound w

theorem r6_dual_band_witness_with_window_and_tail_of_omitted_tail
    (hOmitted : R6DualBandWitnessWithWindowAndOmittedTailTerm) :
    R6DualBandWitnessWithWindowAndTailTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hOmitted E hVonKoch s hs hs_gt with ⟨w, hWindow, hTailPack⟩
  rcases hTailPack with ⟨tailPack⟩
  refine ⟨w, hWindow, ?_⟩
  exact r6_dual_band_asymptotic_tail_bound_of_omitted_mode_tail_pack w tailPack

theorem r6_dual_band_piecewise_power_majorant_witness_of_model_pack
    (hPack : R6DualBandWitnessWithWindowAndTailTerm) :
    R6DualBandPiecewisePowerMajorantWitnessTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hPack E hVonKoch s hs hs_gt with ⟨w, hWindow, hTail⟩
  refine ⟨w.β, w.τ, w.a, w.b, w.R, w.C, w.η, w.x0, w.x1, w.modes, w.i0,
    w.beta_gt_half, w.tau_pos, w.main_nontrivial, w.c_nonneg, w.eta_pos,
    w.modes_len_ge_6, w.dominant_index_valid, w.x0_le_x1, w.decomp, hWindow, hTail⟩

class R6DualBandWitnessWithWindowAndTailProvider where
  theorem_term : R6DualBandWitnessWithWindowAndTailTerm

class R6DualBandWitnessWithWindowAndOmittedTailProvider where
  theorem_term : R6DualBandWitnessWithWindowAndOmittedTailTerm

class R6DualBandWitnessWithWindowAndFiniteOmittedModesProvider where
  theorem_term : R6DualBandWitnessWithWindowAndFiniteOmittedModesTerm

class R6DualBandTailRepresentationKernelProvider where
  theorem_term : R6DualBandTailRepresentationKernelTerm

class R6DualBandTailRepresentationCandidateShapeProvider where
  theorem_term : R6DualBandTailRepresentationCandidateShapeTerm

noncomputable instance r6DualBandTailRepresentationKernelProviderOfCandidateShape
    [h : R6DualBandTailRepresentationCandidateShapeProvider] :
    R6DualBandTailRepresentationKernelProvider where
  theorem_term :=
    r6_dual_band_tail_representation_kernel_of_candidate_shape_term h.theorem_term

noncomputable instance r6DualBandWitnessWithWindowAndFiniteOmittedModesProviderOfTailRepresentationKernel
    [h : R6DualBandTailRepresentationKernelProvider] :
    R6DualBandWitnessWithWindowAndFiniteOmittedModesProvider where
  theorem_term :=
    r6_dual_band_witness_with_window_and_finite_omitted_modes_of_tail_representation_kernel
      h.theorem_term

noncomputable instance r6DualBandWitnessWithWindowAndOmittedTailProviderOfFiniteOmittedModes
    [h : R6DualBandWitnessWithWindowAndFiniteOmittedModesProvider] :
    R6DualBandWitnessWithWindowAndOmittedTailProvider where
  theorem_term :=
    r6_dual_band_witness_with_window_and_omitted_tail_of_finite_omitted_modes h.theorem_term

noncomputable instance r6DualBandWitnessWithWindowAndTailProviderOfOmittedTail
    [h : R6DualBandWitnessWithWindowAndOmittedTailProvider] :
    R6DualBandWitnessWithWindowAndTailProvider where
  theorem_term :=
    r6_dual_band_witness_with_window_and_tail_of_omitted_tail h.theorem_term

theorem r6_dual_band_power_majorant_fitting_of_piecewise_power_majorant_witness
    (hPiecewise : R6DualBandPiecewisePowerMajorantWitnessTerm) :
    R6DualBandPowerMajorantFittingTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hPiecewise E hVonKoch s hs hs_gt with
    ⟨β, τ, a, b, R, C, η, x0, x1, modes, i0,
      hβ, hτ, hab, hC, hη, hLen, hi0, hx01, hDecomp, hWindow, hTail⟩
  have _ : x0 ≤ x1 := hx01
  refine ⟨β, τ, a, b, R, C, η, x0, modes, i0, hβ, hτ, hab, hC, hη, hLen, hi0, hDecomp, ?_⟩
  intro x hx
  by_cases hx1 : x ≤ x1
  · exact hWindow x hx hx1
  · have hx1' : x1 ≤ x := le_of_not_ge hx1
    exact hTail x hx1'

theorem zero_to_cos_sin_power_majorant_of_r6_dual_band_power_majorant_fitting
    (hR6 : R6DualBandPowerMajorantFittingTerm) :
    ZeroToCosSinPhasePowerMajorantTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hR6 E hVonKoch s hs hs_gt with
    ⟨β, τ, a, b, R, C, η, x0, modes, i0,
      hβ, hτ, hab, hC, hη, hLen, hi0, hDecomp, hPowerTail⟩
  have _ : 6 ≤ modes.length := hLen
  have _ : i0 < modes.length := hi0
  exact ⟨β, τ, a, b, R, C, η, x0, hβ, hτ, hab, hC, hη, hDecomp, hPowerTail⟩

theorem zero_to_cos_sin_asymptotic_strict_tail_power_of_r6_dual_band_power_majorant_fitting
    (hR6 : R6DualBandPowerMajorantFittingTerm) :
    ZeroToCosSinAsymptoticStrictTailPowerTerm :=
  zero_to_cos_sin_asymptotic_strict_tail_power_of_power_majorant_term
    (zero_to_cos_sin_power_majorant_of_r6_dual_band_power_majorant_fitting hR6)

theorem k1_term_of_r6_dual_band_power_majorant_fitting
    (hR6 : R6DualBandPowerMajorantFittingTerm) :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_power_majorant
    (zero_to_cos_sin_power_majorant_of_r6_dual_band_power_majorant_fitting hR6)

theorem rh_from_r6_dual_band_power_majorant_fitting
    (hR6 : R6DualBandPowerMajorantFittingTerm) :
    RHStatement :=
  rh_from_zero_to_cos_sin_power_majorant
    (zero_to_cos_sin_power_majorant_of_r6_dual_band_power_majorant_fitting hR6)

class R6DualBandPiecewisePowerMajorantWitnessProvider where
  theorem_term : R6DualBandPiecewisePowerMajorantWitnessTerm

noncomputable instance r6DualBandPiecewisePowerMajorantWitnessProviderOfModelPack
    [h : R6DualBandWitnessWithWindowAndTailProvider] :
    R6DualBandPiecewisePowerMajorantWitnessProvider where
  theorem_term :=
    r6_dual_band_piecewise_power_majorant_witness_of_model_pack h.theorem_term

class R6DualBandPowerMajorantFittingProvider where
  theorem_term : R6DualBandPowerMajorantFittingTerm

noncomputable instance r6DualBandPowerMajorantFittingProviderOfPiecewiseWitness
    [h : R6DualBandPiecewisePowerMajorantWitnessProvider] :
    R6DualBandPowerMajorantFittingProvider where
  theorem_term :=
    r6_dual_band_power_majorant_fitting_of_piecewise_power_majorant_witness
      h.theorem_term

theorem rh_from_r6_dual_band_power_majorant_fitting_provider
    [h : R6DualBandPowerMajorantFittingProvider] :
    RHStatement :=
  rh_from_r6_dual_band_power_majorant_fitting h.theorem_term

theorem rh_from_r6_dual_band_piecewise_power_majorant_witness_provider
    [h : R6DualBandPiecewisePowerMajorantWitnessProvider] :
    RHStatement :=
  rh_from_r6_dual_band_power_majorant_fitting_provider
    (h := r6DualBandPowerMajorantFittingProviderOfPiecewiseWitness)

theorem rh_from_r6_dual_band_witness_with_window_and_tail_provider
    [h : R6DualBandWitnessWithWindowAndTailProvider] :
    RHStatement :=
  rh_from_r6_dual_band_piecewise_power_majorant_witness_provider
    (h := r6DualBandPiecewisePowerMajorantWitnessProviderOfModelPack)

theorem rh_from_r6_dual_band_witness_with_window_and_omitted_tail_provider
    [h : R6DualBandWitnessWithWindowAndOmittedTailProvider] :
    RHStatement :=
  rh_from_r6_dual_band_witness_with_window_and_tail_provider
    (h := r6DualBandWitnessWithWindowAndTailProviderOfOmittedTail)

theorem rh_from_r6_dual_band_witness_with_window_and_finite_omitted_modes_provider
    [h : R6DualBandWitnessWithWindowAndFiniteOmittedModesProvider] :
    RHStatement :=
  rh_from_r6_dual_band_witness_with_window_and_omitted_tail_provider
    (h := r6DualBandWitnessWithWindowAndOmittedTailProviderOfFiniteOmittedModes)

theorem rh_from_r6_dual_band_tail_representation_kernel_provider
    [h : R6DualBandTailRepresentationKernelProvider] :
    RHStatement :=
  rh_from_r6_dual_band_witness_with_window_and_finite_omitted_modes_provider
    (h := r6DualBandWitnessWithWindowAndFiniteOmittedModesProviderOfTailRepresentationKernel)

theorem rh_from_r6_dual_band_tail_representation_candidate_shape_provider
    [h : R6DualBandTailRepresentationCandidateShapeProvider] :
    RHStatement :=
  rh_from_r6_dual_band_tail_representation_kernel_provider
    (h := r6DualBandTailRepresentationKernelProviderOfCandidateShape)

class ZeroToCosSinNearStrictTailPowerProvider where
  theorem_term : ZeroToCosSinNearStrictTailPowerTerm

theorem k1_term_from_near_strict_tail_power_provider
    [h : ZeroToCosSinNearStrictTailPowerProvider] :
    InghamImportedPayloadTerm :=
  k1_term_of_near_strict_tail_power_term h.theorem_term

theorem rh_from_near_strict_tail_power_provider
    [h : ZeroToCosSinNearStrictTailPowerProvider] :
    RHStatement :=
  rh_from_near_strict_tail_power_term h.theorem_term

class ZeroToCosSinAsymptoticStrictTailPowerProvider where
  theorem_term : ZeroToCosSinAsymptoticStrictTailPowerTerm

theorem k1_term_from_asymptotic_strict_tail_power_provider
    [h : ZeroToCosSinAsymptoticStrictTailPowerProvider] :
    InghamImportedPayloadTerm :=
  k1_term_of_asymptotic_strict_tail_power_term h.theorem_term

theorem rh_from_asymptotic_strict_tail_power_provider
    [h : ZeroToCosSinAsymptoticStrictTailPowerProvider] :
    RHStatement :=
  rh_from_asymptotic_strict_tail_power_term h.theorem_term

class ZeroToCosSinPowerMajorantProvider where
  theorem_term : ZeroToCosSinPhasePowerMajorantTerm

theorem k1_term_from_zero_to_cos_sin_power_majorant_provider
    [h : ZeroToCosSinPowerMajorantProvider] :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_power_majorant h.theorem_term

theorem rh_from_zero_to_cos_sin_power_majorant_provider
    [h : ZeroToCosSinPowerMajorantProvider] :
    RHStatement :=
  rh_from_zero_to_cos_sin_power_majorant h.theorem_term

noncomputable instance zeroToCosSinPowerMajorantProviderOfR6DualBandPowerMajorantFitting
    [h : R6DualBandPowerMajorantFittingProvider] :
    ZeroToCosSinPowerMajorantProvider where
  theorem_term :=
    zero_to_cos_sin_power_majorant_of_r6_dual_band_power_majorant_fitting
      h.theorem_term

noncomputable instance zeroToCosSinAsymptoticStrictTailPowerProviderOfR6DualBandPowerMajorantFitting
    [h : R6DualBandPowerMajorantFittingProvider] :
    ZeroToCosSinAsymptoticStrictTailPowerProvider where
  theorem_term :=
    zero_to_cos_sin_asymptotic_strict_tail_power_of_r6_dual_band_power_majorant_fitting
      h.theorem_term

class ZeroToCosSinPhaseProvider where
  theorem_term : ZeroToCosSinPhaseTerm

theorem k1_term_from_zero_to_cos_sin_provider
    [h : ZeroToCosSinPhaseProvider] :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_phase h.theorem_term

theorem rh_from_zero_to_cos_sin_provider
    [h : ZeroToCosSinPhaseProvider] :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase h.theorem_term

theorem ingham_payload_of_linear_phase_witness_step_results_non_circular
    (i : ImportedLinearPhaseWitnessStepResults) :
    InghamImportedPayloadTerm :=
  ingham_payload_of_zero_to_cos_sin_phase i.zero_to_cos_sin_phase

theorem rh_from_linear_phase_witness_step_results_non_circular_k1
    (i : ImportedLinearPhaseWitnessStepResults) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase i.zero_to_cos_sin_phase

theorem k1_term_of_imported_cos_sin_only_results
    (i : ImportedLinearPhaseCosSinOnlyResults) :
    InghamImportedPayloadTerm :=
  ingham_payload_of_zero_to_cos_sin_phase i.zero_to_cos_sin_phase

theorem rh_from_imported_cos_sin_only_results
    (i : ImportedLinearPhaseCosSinOnlyResults) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase i.zero_to_cos_sin_phase

theorem k1_term_of_linear_phase_witness_assumptions
    (h : ExplicitFormulaLinearPhaseWitnessAssumptions) :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_linear_phase_witness h)

theorem rh_from_linear_phase_witness_assumptions_via_k1
    (h : ExplicitFormulaLinearPhaseWitnessAssumptions) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_linear_phase_witness h)

theorem k1_term_of_imported_linear_phase_witness_results
    (i : ImportedLinearPhaseWitnessResults) :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_imported_linear_phase_witness i)

theorem rh_from_imported_linear_phase_witness_results_via_k1
    (i : ImportedLinearPhaseWitnessResults) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_imported_linear_phase_witness i)

theorem k1_term_of_log_linear_phase_assumptions
    (h : ExplicitFormulaLogLinearPhaseAssumptions) :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_log_linear_phase h)

theorem rh_from_log_linear_phase_assumptions_via_k1
    (h : ExplicitFormulaLogLinearPhaseAssumptions) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_log_linear_phase h)

theorem k1_term_of_imported_log_linear_phase_results
    (i : ImportedLogLinearPhaseResults) :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_imported_log_linear_phase i)

theorem rh_from_imported_log_linear_phase_results_via_k1
    (i : ImportedLogLinearPhaseResults) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_imported_log_linear_phase i)

theorem k1_term_of_linear_phase_only_assumptions
    (h : ExplicitFormulaLinearPhaseOnlyAssumptions) :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_linear_phase_only h)

theorem rh_from_linear_phase_only_assumptions_via_k1
    (h : ExplicitFormulaLinearPhaseOnlyAssumptions) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_linear_phase_only h)

theorem k1_term_of_imported_linear_phase_only_results
    (i : ImportedLinearPhaseOnlyResults) :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_imported_linear_phase_only i)

theorem rh_from_imported_linear_phase_only_results_via_k1
    (i : ImportedLinearPhaseOnlyResults) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_imported_linear_phase_only i)

theorem k1_term_of_linear_phase_kernel
    (hLinear : LinearPhaseKernelTerm) :
    InghamImportedPayloadTerm :=
  k1_term_of_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_linear_phase_kernel hLinear)

theorem rh_from_linear_phase_kernel_via_k1
    (hLinear : LinearPhaseKernelTerm) :
    RHStatement :=
  rh_from_zero_to_cos_sin_phase
    (zero_to_cos_sin_phase_transfer_of_linear_phase_kernel hLinear)

theorem linear_phase_kernel_of_rh
    (hRH : RHStatement) :
    LinearPhaseKernelTerm := by
  intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
  have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
  have hFalse : False := by linarith
  exact False.elim hFalse

theorem linear_phase_kernel_iff_rh :
    LinearPhaseKernelTerm ↔ RHStatement := by
  constructor
  · exact rh_from_linear_phase_kernel_via_k1
  · exact linear_phase_kernel_of_rh

theorem k1_term_of_log_derivative_linear_phase_kernel
    (hLogDeriv : LogDerivativeLinearPhaseKernelTerm) :
    InghamImportedPayloadTerm :=
  k1_term_of_linear_phase_kernel
    (linear_phase_kernel_of_log_derivative_kernel hLogDeriv)

theorem rh_from_log_derivative_linear_phase_kernel_via_k1
    (hLogDeriv : LogDerivativeLinearPhaseKernelTerm) :
    RHStatement :=
  rh_from_linear_phase_kernel_via_k1
    (linear_phase_kernel_of_log_derivative_kernel hLogDeriv)

theorem log_derivative_linear_phase_kernel_of_rh
    (hRH : RHStatement) :
    LogDerivativeLinearPhaseKernelTerm := by
  intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
  have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
  have hFalse : False := by linarith
  exact False.elim hFalse

theorem log_derivative_linear_phase_kernel_iff_rh :
    LogDerivativeLinearPhaseKernelTerm ↔ RHStatement := by
  constructor
  · exact rh_from_log_derivative_linear_phase_kernel_via_k1
  · exact log_derivative_linear_phase_kernel_of_rh

structure PublishedLinearPhaseWitnessPack where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "EXPLICIT-FORMULA-LINEAR-PHASE-WITNESS"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "zero-to-linear-phase-witness"
  imported_results : ImportedLinearPhaseWitnessResults

def importedLinearPhaseWitnessResultsOfPublishedPack
    (p : PublishedLinearPhaseWitnessPack) :
    ImportedLinearPhaseWitnessResults :=
  p.imported_results

theorem k1_term_of_published_linear_phase_witness_pack
    (p : PublishedLinearPhaseWitnessPack) :
    InghamImportedPayloadTerm :=
  k1_term_of_imported_linear_phase_witness_results
    (importedLinearPhaseWitnessResultsOfPublishedPack p)

theorem rh_from_published_linear_phase_witness_pack
    (p : PublishedLinearPhaseWitnessPack) :
    RHStatement :=
  rh_from_imported_linear_phase_witness_results_via_k1
    (importedLinearPhaseWitnessResultsOfPublishedPack p)

class PublishedLinearPhaseWitnessProvider where
  pack : PublishedLinearPhaseWitnessPack

theorem rh_from_published_linear_phase_witness_provider
    [h : PublishedLinearPhaseWitnessProvider] :
    RHStatement :=
  rh_from_published_linear_phase_witness_pack h.pack

/-!
Published-source import boundary for the single open K1 source proposition.
This closes RH conditionally on a concrete theorem term for
`ZeroToCosSinPhaseTerm`.
-/
structure PublishedZeroToCosSinPhasePack where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "explicit-formula-zero-pair-oscillation-shape"
  zero_to_cos_sin_phase : ZeroToCosSinPhaseTerm

def importedCosSinOnlyResultsOfPublishedPack
    (p : PublishedZeroToCosSinPhasePack) :
    ImportedLinearPhaseCosSinOnlyResults where
  zero_to_cos_sin_phase := p.zero_to_cos_sin_phase

theorem k1_term_of_published_zero_to_cos_sin_pack
    (p : PublishedZeroToCosSinPhasePack) :
    InghamImportedPayloadTerm :=
  k1_term_of_imported_cos_sin_only_results
    (importedCosSinOnlyResultsOfPublishedPack p)

theorem rh_from_published_zero_to_cos_sin_pack
    (p : PublishedZeroToCosSinPhasePack) :
    RHStatement :=
  rh_from_imported_cos_sin_only_results
    (importedCosSinOnlyResultsOfPublishedPack p)

class PublishedZeroToCosSinPhaseProvider where
  pack : PublishedZeroToCosSinPhasePack

noncomputable instance zeroToCosSinProviderOfPublishedPack
    [h : PublishedZeroToCosSinPhaseProvider] :
    ZeroToCosSinPhaseProvider where
  theorem_term := h.pack.zero_to_cos_sin_phase

theorem rh_from_published_zero_to_cos_sin_provider
    [h : PublishedZeroToCosSinPhaseProvider] :
    RHStatement :=
  rh_from_zero_to_cos_sin_provider
    (h := zeroToCosSinProviderOfPublishedPack)

/-!
Published-source import boundary for the asymptotic strict-tail strengthening
of the K1 source proposition.
-/
structure PublishedAsymptoticStrictTailPowerPack where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "explicit-formula-zero-pair-asymptotic-strict-tail"
  asymptotic_strict_tail_power : ZeroToCosSinAsymptoticStrictTailPowerTerm

def publishedZeroToCosSinPackOfAsymptoticStrictTailPack
    (p : PublishedAsymptoticStrictTailPowerPack) :
    PublishedZeroToCosSinPhasePack where
  source_tag := p.source_tag
  source_url := p.source_url
  theorem_ref := "explicit-formula-zero-pair-oscillation-shape"
  source_tag_lock := p.source_tag_lock
  source_url_lock := p.source_url_lock
  theorem_ref_lock := rfl
  zero_to_cos_sin_phase :=
    zero_to_cos_sin_phase_of_asymptotic_strict_tail_power_term
      p.asymptotic_strict_tail_power

theorem k1_term_of_published_asymptotic_strict_tail_power_pack
    (p : PublishedAsymptoticStrictTailPowerPack) :
    InghamImportedPayloadTerm :=
  k1_term_of_asymptotic_strict_tail_power_term p.asymptotic_strict_tail_power

theorem rh_from_published_asymptotic_strict_tail_power_pack
    (p : PublishedAsymptoticStrictTailPowerPack) :
    RHStatement :=
  rh_from_asymptotic_strict_tail_power_term p.asymptotic_strict_tail_power

class PublishedAsymptoticStrictTailPowerProvider where
  pack : PublishedAsymptoticStrictTailPowerPack

noncomputable instance asymptoticStrictTailPowerProviderOfPublishedPack
    [h : PublishedAsymptoticStrictTailPowerProvider] :
    ZeroToCosSinAsymptoticStrictTailPowerProvider where
  theorem_term := h.pack.asymptotic_strict_tail_power

noncomputable instance publishedZeroToCosSinProviderOfPublishedAsymptoticStrictTailPack
    [h : PublishedAsymptoticStrictTailPowerProvider] :
    PublishedZeroToCosSinPhaseProvider where
  pack := publishedZeroToCosSinPackOfAsymptoticStrictTailPack h.pack

theorem rh_from_published_asymptotic_strict_tail_power_provider
    [h : PublishedAsymptoticStrictTailPowerProvider] :
    RHStatement :=
  rh_from_asymptotic_strict_tail_power_provider
    (h := asymptoticStrictTailPowerProviderOfPublishedPack)

/-!
Published-source boundary for the power-majorant variant, with an in-repo
reduction into the asymptotic strict-tail published boundary.
-/
structure PublishedZeroToCosSinPowerMajorantPack where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "explicit-formula-zero-pair-power-majorant-tail"
  zero_to_cos_sin_power_majorant : ZeroToCosSinPhasePowerMajorantTerm

def publishedAsymptoticStrictTailPowerPackOfPowerMajorantPack
    (p : PublishedZeroToCosSinPowerMajorantPack) :
    PublishedAsymptoticStrictTailPowerPack where
  source_tag := p.source_tag
  source_url := p.source_url
  theorem_ref := "explicit-formula-zero-pair-asymptotic-strict-tail"
  source_tag_lock := p.source_tag_lock
  source_url_lock := p.source_url_lock
  theorem_ref_lock := rfl
  asymptotic_strict_tail_power :=
    zero_to_cos_sin_asymptotic_strict_tail_power_of_power_majorant_term
      p.zero_to_cos_sin_power_majorant

theorem k1_term_of_published_zero_to_cos_sin_power_majorant_pack
    (p : PublishedZeroToCosSinPowerMajorantPack) :
    InghamImportedPayloadTerm :=
  k1_term_of_published_asymptotic_strict_tail_power_pack
    (publishedAsymptoticStrictTailPowerPackOfPowerMajorantPack p)

theorem rh_from_published_zero_to_cos_sin_power_majorant_pack
    (p : PublishedZeroToCosSinPowerMajorantPack) :
    RHStatement :=
  rh_from_published_asymptotic_strict_tail_power_pack
    (publishedAsymptoticStrictTailPowerPackOfPowerMajorantPack p)

class PublishedZeroToCosSinPowerMajorantProvider where
  pack : PublishedZeroToCosSinPowerMajorantPack

noncomputable instance publishedAsymptoticStrictTailPowerProviderOfPublishedPowerMajorantProvider
    [h : PublishedZeroToCosSinPowerMajorantProvider] :
    PublishedAsymptoticStrictTailPowerProvider where
  pack := publishedAsymptoticStrictTailPowerPackOfPowerMajorantPack h.pack

theorem rh_from_published_zero_to_cos_sin_power_majorant_provider
    [h : PublishedZeroToCosSinPowerMajorantProvider] :
    RHStatement :=
  rh_from_published_asymptotic_strict_tail_power_provider
    (h := publishedAsymptoticStrictTailPowerProviderOfPublishedPowerMajorantProvider)

/-!
Buffered C2 + rounding-preservation + one-sided tail-split symbolic chain.

This section formalizes the algebraic/quantifier bridge requested in the W64
checkpoint:
1. buffered C2 anchor existence,
2. eventual rounding-preservation from phase-error bounds,
3. one-sided tail bound `R2^- ≤ q*A1`,
4. symbolic closure `q < a1` -> positive lower envelope.
-/

abbrev BufferedC2AnchorTerm
    (phase1Anchor phase2Anchor : Real → Real)
    (c0 : Real) : Prop :=
  ∀ X : Real, ∃ x : Real, x ≥ X ∧
    Real.cos (phase1Anchor x) = 1 ∧
    c0 ≤ Real.cos (phase2Anchor x)

abbrev ConstructiveCosGateTerm
    (phase1 phase2 : Real → Real)
    (a1 : Real) : Prop :=
  ∀ X : Real, ∃ x : Real, x ≥ X ∧
    a1 ≤ Real.cos (phase1 x) ∧
    0 ≤ Real.cos (phase2 x)

abbrev ConstructiveGateTerm
    (phase1 phase2 R2 : Real → Real)
    (a1 q A1 : Real) : Prop :=
  ∀ X : Real, ∃ x : Real, x ≥ X ∧
    a1 ≤ Real.cos (phase1 x) ∧
    0 ≤ Real.cos (phase2 x) ∧
    max (-(R2 x)) 0 ≤ q * A1

theorem cos_lower_of_phase_error
    (u v : Real) :
    Real.cos u - |u - v| ≤ Real.cos v := by
  have hLip : |Real.cos u - Real.cos v| ≤ |u - v| := by
    simpa using Real.abs_cos_sub_cos_le u v
  have hPair :
      Real.cos u - Real.cos v ≤ |u - v| ∧
        Real.cos v - Real.cos u ≤ |u - v| :=
    (abs_sub_le_iff.mp hLip)
  linarith [hPair.2]

theorem nonneg_cos_of_buffer_and_phase_error
    {u v c0 eps : Real}
    (hBuffer : c0 ≤ Real.cos u)
    (hErr : |u - v| ≤ eps)
    (hEps : eps ≤ c0) :
    0 ≤ Real.cos v := by
  have hLower : Real.cos u - |u - v| ≤ Real.cos v :=
    cos_lower_of_phase_error u v
  have hLower' : c0 - eps ≤ Real.cos v := by
    linarith [hLower, hBuffer, hErr]
  linarith [hLower', hEps]

theorem anchor_cos_lower_of_phase_error
    {u v a1 eps : Real}
    (hAnchor : Real.cos u = 1)
    (hErr : |u - v| ≤ eps)
    (hEps : eps ≤ 1 - a1) :
    a1 ≤ Real.cos v := by
  have hLower : Real.cos u - |u - v| ≤ Real.cos v :=
    cos_lower_of_phase_error u v
  have hLower' : 1 - eps ≤ Real.cos v := by
    linarith [hLower, hAnchor, hErr]
  linarith [hLower', hEps]

theorem constructive_cos_gate_of_buffered_c2_and_eventual_rounding_errors
    {phase1Anchor phase2Anchor phase1 phase2 : Real → Real}
    {a1 c0 eps1 eps2 : Real}
    (hC2 : BufferedC2AnchorTerm phase1Anchor phase2Anchor c0)
    (hErr1 : ∀ᶠ x : Real in Filter.atTop, |phase1Anchor x - phase1 x| ≤ eps1)
    (hErr2 : ∀ᶠ x : Real in Filter.atTop, |phase2Anchor x - phase2 x| ≤ eps2)
    (hEps1 : eps1 ≤ 1 - a1)
    (hEps2 : eps2 ≤ c0) :
    ConstructiveCosGateTerm phase1 phase2 a1 := by
  rcases Filter.eventually_atTop.1 hErr1 with ⟨X1, hX1⟩
  rcases Filter.eventually_atTop.1 hErr2 with ⟨X2, hX2⟩
  intro X
  let X' : Real := max X (max X1 X2)
  rcases hC2 X' with ⟨x, hxX', hCos1Anchor, hCos2Anchor⟩
  have hxX : x ≥ X := by
    have h : X ≤ X' := le_max_left X (max X1 X2)
    exact le_trans h hxX'
  have hxX1 : x ≥ X1 := by
    have h : X1 ≤ X' := by
      exact le_trans (le_max_left X1 X2) (le_max_right X (max X1 X2))
    exact le_trans h hxX'
  have hxX2 : x ≥ X2 := by
    have h : X2 ≤ X' := by
      exact le_trans (le_max_right X1 X2) (le_max_right X (max X1 X2))
    exact le_trans h hxX'
  have hCos1 : a1 ≤ Real.cos (phase1 x) :=
    anchor_cos_lower_of_phase_error hCos1Anchor (hX1 x hxX1) hEps1
  have hCos2 : 0 ≤ Real.cos (phase2 x) :=
    nonneg_cos_of_buffer_and_phase_error hCos2Anchor (hX2 x hxX2) hEps2
  exact ⟨x, hxX, hCos1, hCos2⟩

theorem constructive_gate_of_cos_gate_and_eventual_tail_bound
    {phase1 phase2 R2 : Real → Real}
    {a1 q A1 : Real}
    (hCosGate : ConstructiveCosGateTerm phase1 phase2 a1)
    (hTail : ∀ᶠ x : Real in Filter.atTop, max (-(R2 x)) 0 ≤ q * A1) :
    ConstructiveGateTerm phase1 phase2 R2 a1 q A1 := by
  rcases Filter.eventually_atTop.1 hTail with ⟨XT, hXT⟩
  intro X
  let X' : Real := max X XT
  rcases hCosGate X' with ⟨x, hxX', hCos1, hCos2⟩
  have hxX : x ≥ X := by
    exact le_trans (le_max_left X XT) hxX'
  have hxXT : x ≥ XT := by
    exact le_trans (le_max_right X XT) hxX'
  exact ⟨x, hxX, hCos1, hCos2, hXT x hxXT⟩

theorem normalized_lower_envelope_of_constructive_gate_and_q_lt_a1
    {Y phase1 phase2 R2 : Real → Real}
    {A1 A2 a1 q : Real}
    (hA1 : 0 < A1)
    (hA2 : 0 ≤ A2)
    (hq : q < a1)
    (hDecomp :
      ∀ x : Real,
        Y x = A1 * Real.cos (phase1 x) + A2 * Real.cos (phase2 x) + R2 x)
    (hGate : ConstructiveGateTerm phase1 phase2 R2 a1 q A1) :
    ∃ c : Real, 0 < c ∧
      (∀ X : Real, ∃ x : Real, x ≥ X ∧ Y x ≥ c) := by
  let c : Real := A1 * (a1 - q)
  have hc : 0 < c := by
    have hGap : 0 < a1 - q := by linarith
    exact mul_pos hA1 hGap
  refine ⟨c, hc, ?_⟩
  intro X
  rcases hGate X with ⟨x, hxX, hCos1, hCos2, hR2neg⟩
  have hMain1 : A1 * a1 ≤ A1 * Real.cos (phase1 x) := by
    exact mul_le_mul_of_nonneg_left hCos1 (le_of_lt hA1)
  have hMain2 : 0 ≤ A2 * Real.cos (phase2 x) := by
    exact mul_nonneg hA2 hCos2
  have hR2lower : -(q * A1) ≤ R2 x := by
    have hNegLeMax : -(R2 x) ≤ max (-(R2 x)) 0 := le_max_left _ _
    have hNegBound : -(R2 x) ≤ q * A1 := le_trans hNegLeMax hR2neg
    linarith
  have hLower : c ≤ Y x := by
    calc
      c = A1 * a1 - q * A1 := by
        dsimp [c]
        ring
      _ ≤ A1 * Real.cos (phase1 x) + A2 * Real.cos (phase2 x) + R2 x := by
        linarith [hMain1, hMain2, hR2lower]
      _ = Y x := by
        simpa [add_assoc, add_comm, add_left_comm] using (hDecomp x).symm
  exact ⟨x, hxX, by simpa [c] using hLower⟩

theorem normalized_lower_envelope_of_buffered_c2_rounding_tail_and_q_lt_a1
    {Y phase1Anchor phase2Anchor phase1 phase2 R2 : Real → Real}
    {A1 A2 a1 c0 eps1 eps2 q : Real}
    (hC2 : BufferedC2AnchorTerm phase1Anchor phase2Anchor c0)
    (hErr1 : ∀ᶠ x : Real in Filter.atTop, |phase1Anchor x - phase1 x| ≤ eps1)
    (hErr2 : ∀ᶠ x : Real in Filter.atTop, |phase2Anchor x - phase2 x| ≤ eps2)
    (hTail : ∀ᶠ x : Real in Filter.atTop, max (-(R2 x)) 0 ≤ q * A1)
    (hEps1 : eps1 ≤ 1 - a1)
    (hEps2 : eps2 ≤ c0)
    (hA1 : 0 < A1)
    (hA2 : 0 ≤ A2)
    (hq : q < a1)
    (hDecomp :
      ∀ x : Real,
        Y x = A1 * Real.cos (phase1 x) + A2 * Real.cos (phase2 x) + R2 x) :
    ∃ c : Real, 0 < c ∧
      (∀ X : Real, ∃ x : Real, x ≥ X ∧ Y x ≥ c) := by
  let hCosGate : ConstructiveCosGateTerm phase1 phase2 a1 :=
    constructive_cos_gate_of_buffered_c2_and_eventual_rounding_errors
      hC2 hErr1 hErr2 hEps1 hEps2
  let hGate : ConstructiveGateTerm phase1 phase2 R2 a1 q A1 :=
    constructive_gate_of_cos_gate_and_eventual_tail_bound hCosGate hTail
  exact normalized_lower_envelope_of_constructive_gate_and_q_lt_a1
    hA1 hA2 hq hDecomp hGate

/-!
Stronger witness target `U` (one-sided spinning-top lower witness):

For each tail threshold `X`, obtain one sample `x >= X` with:
- explicit oscillatory decomposition at `x`,
- positive cosine pinning `cos(phase x) >= δ`,
- small remainder `|R x| <= (A*δ/2) * x^β`,
with fixed `A > 0`, `β > 1/2`, `δ > 0`.

Then `U` implies signed payload target `T`.
-/

abbrev SpinningTopPositiveWitnessPayloadTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ A β δ : Real, 0 < A ∧ (1 / 2 : Real) < β ∧ 0 < δ ∧
          (∀ X : Real, ∃ x : Real, x ≥ X ∧ x ≥ 1 ∧
            ∃ phase R : Real → Real,
              E x = oscillatoryMainTerm A β phase x + R x ∧
              Real.cos (phase x) ≥ δ ∧
              |R x| ≤ (A * δ / 2) * x ^ β)

theorem spinning_top_signed_payload_of_positive_witness
    (hU : SpinningTopPositiveWitnessPayloadTerm) :
    SpinningTopSignedPayloadTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hU E hVonKoch s hs hs_gt with ⟨A, β, δ, hA, hβ, hδ, hTail⟩
  refine ⟨A * δ / 2, β, by nlinarith [hA, hδ], hβ, Or.inl ?_⟩
  intro X
  rcases hTail X with ⟨x, hxX, hx1, phase, R, hDecomp, hCos, hRem⟩
  have hx_nonneg : 0 ≤ x := le_trans (by norm_num) hx1
  have hPowNonneg : 0 ≤ x ^ β := Real.rpow_nonneg hx_nonneg β
  have hAxNonneg : 0 ≤ A * x ^ β := mul_nonneg (le_of_lt hA) hPowNonneg
  have hMainLowerCore :
      (A * x ^ β) * δ ≤ (A * x ^ β) * Real.cos (phase x) :=
    mul_le_mul_of_nonneg_left hCos hAxNonneg
  have hMainLower :
      A * δ * x ^ β ≤ oscillatoryMainTerm A β phase x := by
    calc
      A * δ * x ^ β = (A * x ^ β) * δ := by ring
      _ ≤ (A * x ^ β) * Real.cos (phase x) := hMainLowerCore
      _ = oscillatoryMainTerm A β phase x := by
            unfold oscillatoryMainTerm
            ring
  have hRLower : -((A * δ / 2) * x ^ β) ≤ R x := by
    have hneg : -(R x) ≤ |R x| := neg_le_abs (R x)
    have hneg' : -(R x) ≤ (A * δ / 2) * x ^ β := le_trans hneg hRem
    linarith
  have hELower : (A * δ / 2) * x ^ β ≤ E x := by
    have hEq : E x = oscillatoryMainTerm A β phase x + R x := hDecomp
    linarith [hEq, hMainLower, hRLower]
  exact ⟨x, hxX, hELower⟩

theorem rh_from_spinning_top_positive_witness
    (hU : SpinningTopPositiveWitnessPayloadTerm) :
    RHStatement :=
  rh_from_spinning_top_signed_payload
    (spinning_top_signed_payload_of_positive_witness hU)

class SpinningTopPositiveWitnessProvider where
  theorem_term : SpinningTopPositiveWitnessPayloadTerm

noncomputable instance spinningTopSignedProviderOfPositiveWitness
    [h : SpinningTopPositiveWitnessProvider] :
    SpinningTopSignedPayloadProvider where
  theorem_term := spinning_top_signed_payload_of_positive_witness h.theorem_term

theorem rh_from_spinning_top_positive_witness_provider
    [h : SpinningTopPositiveWitnessProvider] :
    RHStatement :=
  rh_from_spinning_top_signed_provider
    (h := spinningTopSignedProviderOfPositiveWitness)

theorem spinning_top_signed_payload_of_phase_oscillation_assumptions
    (h : PhaseOscillationAsymptoticAssumptions) :
    SpinningTopSignedPayloadTerm := by
  intro E hVonKoch s hs hs_gt
  let hSigned : ExplicitFormulaSignedOscillationAssumptions :=
    signedAssumptionsOfSequenceEventually
      (sequenceEventuallyAssumptionsOfDecomposition
        (decompositionAssumptionsOfAsymptotic
          (asymptoticAssumptionsOfPhaseOscillation h)))
  exact hSigned.zero_to_signed_oscillation E hVonKoch s hs hs_gt

theorem rh_from_phase_oscillation_via_spinning_top_signed_payload
    (h : PhaseOscillationAsymptoticAssumptions) :
    RHStatement :=
  rh_from_spinning_top_signed_payload
    (spinning_top_signed_payload_of_phase_oscillation_assumptions h)

class SpinningTopPhaseOscillationProvider where
  assumptions : PhaseOscillationAsymptoticAssumptions

noncomputable instance spinningTopSignedProviderOfPhaseOscillation
    [h : SpinningTopPhaseOscillationProvider] :
    SpinningTopSignedPayloadProvider where
  theorem_term := spinning_top_signed_payload_of_phase_oscillation_assumptions h.assumptions

theorem rh_from_spinning_top_phase_oscillation_provider
    [h : SpinningTopPhaseOscillationProvider] :
    RHStatement :=
  rh_from_spinning_top_signed_provider
    (h := spinningTopSignedProviderOfPhaseOscillation)

theorem spinning_top_signed_payload_of_linear_phase_witness_assumptions
    (h : ExplicitFormulaLinearPhaseWitnessAssumptions) :
    SpinningTopSignedPayloadTerm :=
  spinning_top_signed_payload_of_phase_oscillation_assumptions
    (phaseOscillationAssumptionsOfLinearPhaseWitness h)

theorem rh_from_linear_phase_witness_via_spinning_top_signed_payload
    (h : ExplicitFormulaLinearPhaseWitnessAssumptions) :
    RHStatement :=
  rh_from_spinning_top_signed_payload
    (spinning_top_signed_payload_of_linear_phase_witness_assumptions h)

theorem ingham_payload_of_linear_phase_witness_assumptions
    (h : ExplicitFormulaLinearPhaseWitnessAssumptions) :
    InghamImportedPayloadTerm :=
  ingham_payload_of_spinning_top_signed_payload
    (spinning_top_signed_payload_of_linear_phase_witness_assumptions h)

theorem spinning_top_signed_payload_of_linear_phase_witness_step_results
    (i : ImportedLinearPhaseWitnessStepResults) :
    SpinningTopSignedPayloadTerm := by
  let iw : ImportedLinearPhaseWitnessResults :=
    importedLinearPhaseWitnessResultsOfStepResults i
  let hlin : ExplicitFormulaLinearPhaseWitnessAssumptions :=
    linearPhaseWitnessAssumptionsOfImported iw
  exact spinning_top_signed_payload_of_linear_phase_witness_assumptions hlin

theorem rh_from_linear_phase_witness_step_results_via_spinning_top_signed_payload
    (i : ImportedLinearPhaseWitnessStepResults) :
    RHStatement :=
  rh_from_spinning_top_signed_payload
    (spinning_top_signed_payload_of_linear_phase_witness_step_results i)

theorem ingham_payload_of_linear_phase_witness_step_results
    (i : ImportedLinearPhaseWitnessStepResults) :
    InghamImportedPayloadTerm :=
  ingham_payload_of_spinning_top_signed_payload
    (spinning_top_signed_payload_of_linear_phase_witness_step_results i)

end PrimeRiemannBridgeSpinningTopFrontier
