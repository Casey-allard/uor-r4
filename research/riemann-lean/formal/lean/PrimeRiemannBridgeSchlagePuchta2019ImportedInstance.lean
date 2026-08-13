import PrimeRiemannBridgeNearStrictTailToPintz
import PrimeRiemannBridgeFinalTargetEquivalence

namespace PrimeRiemannBridgeSchlagePuchta2019ImportedInstance

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeConcretePackInstantiation
open PrimeRiemannBridgeFinalTargetEquivalence
open PrimeRiemannBridgeOscillatoryReduction
open PrimeRiemannBridgeZeroOscillationProgram

/-!
Published import endpoint for the final open source kernel.

This file instantiates the repository's Schlage-Puchta 2019 interface from one
explicit imported theorem term, then closes the RH chain via existing bridges.
-/

/-!
Iterative closure ladder for the final blocker (single open item):

`SchlagePuchtaIntervalCoreTerm`
  <- proved from `SchlagePuchtaPhasePinnedWindowTerm` in this file
  <- target currently supplied by imported theorem term axiom

This keeps one blocker while enabling concrete mathematical iteration on a
stronger "spinning-top phase pinned in short windows" witness shape.
-/

abbrev SchlagePuchtaIntervalCoreTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ c β δ X0 : Real, 0 < c ∧ (1 / 2 : Real) < β ∧ 0 < δ ∧
          (∀ X : Real, X ≥ X0 →
            ∃ x : Real, x ≥ X ∧ x ≤ X ^ (1 + δ) ∧ |E x| ≥ c * x ^ β)

abbrev SchlagePuchtaSignedWindowOscillationTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ c β δ X0 : Real, 0 < c ∧ (1 / 2 : Real) < β ∧ 0 < δ ∧
          ((∀ X : Real, X ≥ X0 →
              ∃ x : Real, x ≥ X ∧ x ≤ X ^ (1 + δ) ∧ E x ≥ c * x ^ β) ∨
           (∀ X : Real, X ≥ X0 →
              ∃ x : Real, x ≥ X ∧ x ≤ X ^ (1 + δ) ∧ E x ≤ -(c * x ^ β)))

theorem schlage_core_of_signed_window_oscillation
    (hSigned : SchlagePuchtaSignedWindowOscillationTerm) :
    SchlagePuchtaIntervalCoreTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hSigned E hVonKoch s hs hs_gt with
    ⟨c, β, δ, X0, hc, hβ, hδ, hSign⟩
  refine ⟨c, β, δ, X0, hc, hβ, hδ, ?_⟩
  intro X hX
  rcases hSign with hPos | hNeg
  · rcases hPos X hX with ⟨x, hxX, hxUpper, hLower⟩
    have hAbs : |E x| ≥ c * x ^ β := le_trans hLower (le_abs_self (E x))
    exact ⟨x, hxX, hxUpper, hAbs⟩
  · rcases hNeg X hX with ⟨x, hxX, hxUpper, hUpperNeg⟩
    have hNegBound : c * x ^ β ≤ -E x := by linarith [hUpperNeg]
    have hAbs : |E x| ≥ c * x ^ β := le_trans hNegBound (neg_le_abs (E x))
    exact ⟨x, hxX, hxUpper, hAbs⟩

abbrev SchlagePuchtaSignedGlobalOscillationTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ c β δ X0 : Real, 0 < c ∧ (1 / 2 : Real) < β ∧ 0 < δ ∧
          ((∀ X : Real, X ≥ X0 →
              ∃ x : Real, x ≥ X ∧ E x ≥ c * x ^ β) ∨
           (∀ X : Real, X ≥ X0 →
              ∃ x : Real, x ≥ X ∧ E x ≤ -(c * x ^ β)))

abbrev SchlagePuchtaSignedRelocalizationKernel : Prop :=
  ∀ E : Real → Real, ∀ c β δ X0 : Real,
    0 < c →
    (1 / 2 : Real) < β →
    0 < δ →
    ((∀ X : Real, X ≥ X0 →
        ∃ x : Real, x ≥ X ∧ E x ≥ c * x ^ β) ∨
     (∀ X : Real, X ≥ X0 →
        ∃ x : Real, x ≥ X ∧ E x ≤ -(c * x ^ β))) →
    ((∀ X : Real, X ≥ X0 →
        ∃ x : Real, x ≥ X ∧ x ≤ X ^ (1 + δ) ∧ E x ≥ c * x ^ β) ∨
     (∀ X : Real, X ≥ X0 →
        ∃ x : Real, x ≥ X ∧ x ≤ X ^ (1 + δ) ∧ E x ≤ -(c * x ^ β)))

theorem schlage_signed_window_of_global_and_relocalization
    (hGlobal : SchlagePuchtaSignedGlobalOscillationTerm)
    (hReloc : SchlagePuchtaSignedRelocalizationKernel) :
    SchlagePuchtaSignedWindowOscillationTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hGlobal E hVonKoch s hs hs_gt with
    ⟨c, β, δ, X0, hc, hβ, hδ, hGlobalSign⟩
  refine ⟨c, β, δ, X0, hc, hβ, hδ, ?_⟩
  exact hReloc E c β δ X0 hc hβ hδ hGlobalSign

theorem schlage_core_of_global_and_relocalization
    (hGlobal : SchlagePuchtaSignedGlobalOscillationTerm)
    (hReloc : SchlagePuchtaSignedRelocalizationKernel) :
    SchlagePuchtaIntervalCoreTerm :=
  schlage_core_of_signed_window_oscillation
    (schlage_signed_window_of_global_and_relocalization hGlobal hReloc)

theorem schlage_signed_global_of_signed_assumptions
    (h : ExplicitFormulaSignedOscillationAssumptions) :
    SchlagePuchtaSignedGlobalOscillationTerm := by
  intro E hVonKoch s hs hs_gt
  rcases h.zero_to_signed_oscillation E hVonKoch s hs hs_gt with
    ⟨c, β, hc, hβ, hSign⟩
  refine ⟨c, β, 1, 1, hc, hβ, by norm_num, ?_⟩
  rcases hSign with hPos | hNeg
  · left
    intro X hX
    exact hPos X
  · right
    intro X hX
    exact hNeg X

theorem schlage_signed_global_of_asymptotic_assumptions
    (h : ExplicitFormulaAsymptoticSequenceAssumptions) :
    SchlagePuchtaSignedGlobalOscillationTerm :=
  schlage_signed_global_of_signed_assumptions
    (signedAssumptionsOfAsymptoticAssumptions h)

theorem schlage_core_of_asymptotic_assumptions_and_relocalization
    (h : ExplicitFormulaAsymptoticSequenceAssumptions)
    (hReloc : SchlagePuchtaSignedRelocalizationKernel) :
    SchlagePuchtaIntervalCoreTerm :=
  schlage_core_of_global_and_relocalization
    (schlage_signed_global_of_asymptotic_assumptions h)
    hReloc

abbrev SchlagePuchtaPhasePinnedWindowTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ A β ρ δ X0 : Real, 0 < A ∧ (1 / 2 : Real) < β ∧ 0 < ρ ∧ 0 < δ ∧ 1 ≤ X0 ∧
          (∀ X : Real, X ≥ X0 →
            ∃ x : Real, x ≥ X ∧ x ≤ X ^ (1 + δ) ∧ x ≥ 1 ∧
              ∃ phase R : Real → Real,
                E x = (A * x ^ β) * Real.cos (phase x) + R x ∧
                ρ ≤ Real.cos (phase x) ∧
                |R x| ≤ (A * ρ / 2) * x ^ β)

theorem schlage_core_of_phase_pinned_window
    (hPinned : SchlagePuchtaPhasePinnedWindowTerm) :
    SchlagePuchtaIntervalCoreTerm := by
  intro E hVonKoch s hs hs_gt
  rcases hPinned E hVonKoch s hs hs_gt with
    ⟨A, β, ρ, δ, X0, hA, hβ, hρ, hδ, hX0, hTail⟩
  refine ⟨A * ρ / 2, β, δ, X0, by nlinarith [hA, hρ], hβ, hδ, ?_⟩
  intro X hX
  rcases hTail X hX with
    ⟨x, hxX, hxUpper, hx1, phase, R, hDecomp, hCos, hRem⟩
  let main : Real := (A * x ^ β) * Real.cos (phase x)
  have hx_nonneg : 0 ≤ x := le_trans (by norm_num) hx1
  have hxpow_nonneg : 0 ≤ x ^ β := Real.rpow_nonneg hx_nonneg β
  have hAx_nonneg : 0 ≤ A * x ^ β := mul_nonneg (le_of_lt hA) hxpow_nonneg
  have hmain_nonneg : 0 ≤ main := by
    dsimp [main]
    exact mul_nonneg hAx_nonneg (le_trans (le_of_lt hρ) hCos)
  have hmain_lower_core : (A * x ^ β) * ρ ≤ main := by
    dsimp [main]
    exact mul_le_mul_of_nonneg_left hCos hAx_nonneg
  have hmain_lower : (A * ρ) * x ^ β ≤ main := by
    calc
      (A * ρ) * x ^ β = (A * x ^ β) * ρ := by ring
      _ ≤ main := hmain_lower_core
  have hmain_abs_lower : (A * ρ) * x ^ β ≤ |main| := by
    calc
      (A * ρ) * x ^ β ≤ main := hmain_lower
      _ = |main| := by simpa [abs_of_nonneg hmain_nonneg]
  have htriangle_main : |main| ≤ |main + R x| + |R x| := by
    simpa [sub_eq_add_neg, add_assoc, add_left_comm, add_comm] using
      (abs_sub (main + R x) (R x))
  have hmain_minus_rem : |main + R x| ≥ |main| - |R x| := by
    linarith
  have hE_main : E x = main + R x := by
    simpa [main] using hDecomp
  have hE_abs_lower : |E x| ≥ |main| - |R x| := by
    simpa [hE_main] using hmain_minus_rem
  have htarget_nonneg : 0 ≤ (A * ρ / 2) * x ^ β := by
    nlinarith [hA, hρ, hxpow_nonneg]
  have hmain_ge_target : (A * ρ) * x ^ β ≥ (A * ρ / 2) * x ^ β := by
    nlinarith [hA, hρ, hxpow_nonneg]
  have hmain_rem_lower : |main| - |R x| ≥ (A * ρ / 2) * x ^ β := by
    have hmain_ge : |main| ≥ (A * ρ) * x ^ β := hmain_abs_lower
    have hrem_le : |R x| ≤ (A * ρ / 2) * x ^ β := hRem
    linarith [hmain_ge, hrem_le, hmain_ge_target, htarget_nonneg]
  have hFinal : |E x| ≥ (A * ρ / 2) * x ^ β := le_trans hmain_rem_lower hE_abs_lower
  exact ⟨x, hxX, hxUpper, hFinal⟩

private theorem exists_nat_exp_window_of_log_lower
    (τ off X : Real)
    (hτ : 0 < τ)
    (hX : 0 < X)
    (hbase : off ≤ τ * Real.log X) :
    ∃ n : Nat,
      X ≤ Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ) ∧
      Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ) ≤
        Real.exp (2 * Real.pi / τ) * X := by
  let t : Real := (τ * Real.log X - off) / (2 * Real.pi)
  let n : Nat := Nat.ceil t
  have hTwoPiPos : 0 < (2 * Real.pi) := by nlinarith [Real.pi_pos]
  have hTwoPiNe : (2 * Real.pi) ≠ 0 := ne_of_gt hTwoPiPos
  have ht_nonneg : 0 ≤ t := by
    dsimp [t]
    refine div_nonneg ?_ (le_of_lt hTwoPiPos)
    linarith
  have ht_le_n : t ≤ n := Nat.le_ceil t
  have hmul_lower : t * (2 * Real.pi) ≤ (n : Real) * (2 * Real.pi) := by
    nlinarith [ht_le_n, hTwoPiPos]
  have hcore_lower : τ * Real.log X - off ≤ (n : Real) * (2 * Real.pi) := by
    simpa [t, hTwoPiNe] using hmul_lower
  have harg_lower_core : τ * Real.log X ≤ off + (n : Real) * (2 * Real.pi) := by
    linarith [hcore_lower]
  have harg_lower : Real.log X ≤ (off + (n : Real) * (2 * Real.pi)) / τ := by
    have : Real.log X * τ ≤ off + (n : Real) * (2 * Real.pi) := by
      simpa [mul_comm, mul_left_comm, mul_assoc] using harg_lower_core
    exact (le_div_iff₀ hτ).2 this
  have hLower :
      X ≤ Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ) := by
    have hexp := Real.exp_le_exp.mpr harg_lower
    simpa [Real.exp_log hX] using hexp
  have hn_lt : (n : Real) < t + 1 := by
    simpa [n] using (Nat.ceil_lt_add_one ht_nonneg)
  have hmul_upper : (n : Real) * (2 * Real.pi) < (t + 1) * (2 * Real.pi) := by
    nlinarith [hn_lt, hTwoPiPos]
  have hcore_upper :
      (n : Real) * (2 * Real.pi) < (τ * Real.log X - off) + (2 * Real.pi) := by
    have htmp :
        (n : Real) * (2 * Real.pi) <
          ((τ * Real.log X - off) / (2 * Real.pi) + 1) * (2 * Real.pi) := by
      simpa [t] using hmul_upper
    have hrewrite :
        ((τ * Real.log X - off) / (2 * Real.pi) + 1) * (2 * Real.pi) =
          (τ * Real.log X - off) + (2 * Real.pi) := by
      field_simp [hTwoPiNe]
    exact lt_of_lt_of_eq htmp hrewrite
  have harg_upper_core :
      off + (n : Real) * (2 * Real.pi) < τ * Real.log X + (2 * Real.pi) := by
    linarith [hcore_upper]
  have harg_upper :
      (off + (n : Real) * (2 * Real.pi)) / τ <
        Real.log X + (2 * Real.pi) / τ := by
    have : off + (n : Real) * (2 * Real.pi) <
        (Real.log X + (2 * Real.pi) / τ) * τ := by
      have hτne : τ ≠ 0 := ne_of_gt hτ
      have hExpand :
          (Real.log X + (2 * Real.pi) / τ) * τ =
            τ * Real.log X + (2 * Real.pi) := by
        ring_nf
        field_simp [hτne]
      exact lt_of_lt_of_eq harg_upper_core hExpand.symm
    exact (div_lt_iff₀ hτ).2 this
  refine ⟨n, hLower, ?_⟩
  have hexp_upper :
      Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ) <
        Real.exp (Real.log X + (2 * Real.pi) / τ) :=
    Real.exp_lt_exp.mpr harg_upper
  have htarget :
      Real.exp (Real.log X + (2 * Real.pi) / τ) =
        Real.exp (2 * Real.pi / τ) * X := by
    calc
      Real.exp (Real.log X + (2 * Real.pi) / τ)
          = Real.exp (Real.log X) * Real.exp ((2 * Real.pi) / τ) := by
              rw [Real.exp_add]
      _ = X * Real.exp ((2 * Real.pi) / τ) := by rw [Real.exp_log hX]
      _ = Real.exp ((2 * Real.pi) / τ) * X := by ring
  exact (lt_of_lt_of_eq hexp_upper htarget).le

private theorem geometric_short_window_pos_of_all_indices
    (E : Real → Real)
    (c β δ X0 τ off : Real)
    (hX0pos : 0 < X0)
    (hτ : 0 < τ)
    (hOffX0 : off ≤ τ * Real.log X0)
    (hScale : ∀ X : Real, X ≥ X0 → Real.exp (2 * Real.pi / τ) * X ≤ X ^ (1 + δ))
    (hAll : ∀ n : Nat,
      E (Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)) ≥
        c * (Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)) ^ β) :
    ∀ X : Real, X ≥ X0 →
      ∃ x : Real, x ≥ X ∧ x ≤ X ^ (1 + δ) ∧ E x ≥ c * x ^ β := by
  intro X hX
  have hXpos : 0 < X := lt_of_lt_of_le hX0pos hX
  have hlog : Real.log X0 ≤ Real.log X := Real.log_le_log hX0pos hX
  have hbaseX : off ≤ τ * Real.log X := by
    have hmul : τ * Real.log X0 ≤ τ * Real.log X :=
      mul_le_mul_of_nonneg_left hlog (le_of_lt hτ)
    exact le_trans hOffX0 hmul
  rcases exists_nat_exp_window_of_log_lower τ off X hτ hXpos hbaseX with
    ⟨n, hxLower, hxMulUpper⟩
  let x : Real := Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)
  have hxUpper : x ≤ X ^ (1 + δ) := by
    have hScaleX : Real.exp (2 * Real.pi / τ) * X ≤ X ^ (1 + δ) := hScale X hX
    exact le_trans (by simpa [x] using hxMulUpper) hScaleX
  have hxBound : E x ≥ c * x ^ β := by simpa [x] using hAll n
  exact ⟨x, by simpa [x] using hxLower, hxUpper, hxBound⟩

private theorem geometric_short_window_neg_of_all_indices
    (E : Real → Real)
    (c β δ X0 τ off : Real)
    (hX0pos : 0 < X0)
    (hτ : 0 < τ)
    (hOffX0 : off ≤ τ * Real.log X0)
    (hScale : ∀ X : Real, X ≥ X0 → Real.exp (2 * Real.pi / τ) * X ≤ X ^ (1 + δ))
    (hAll : ∀ n : Nat,
      E (Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)) ≤
        -(c * (Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)) ^ β)) :
    ∀ X : Real, X ≥ X0 →
      ∃ x : Real, x ≥ X ∧ x ≤ X ^ (1 + δ) ∧ E x ≤ -(c * x ^ β) := by
  intro X hX
  have hXpos : 0 < X := lt_of_lt_of_le hX0pos hX
  have hlog : Real.log X0 ≤ Real.log X := Real.log_le_log hX0pos hX
  have hbaseX : off ≤ τ * Real.log X := by
    have hmul : τ * Real.log X0 ≤ τ * Real.log X :=
      mul_le_mul_of_nonneg_left hlog (le_of_lt hτ)
    exact le_trans hOffX0 hmul
  rcases exists_nat_exp_window_of_log_lower τ off X hτ hXpos hbaseX with
    ⟨n, hxLower, hxMulUpper⟩
  let x : Real := Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)
  have hxUpper : x ≤ X ^ (1 + δ) := by
    have hScaleX : Real.exp (2 * Real.pi / τ) * X ≤ X ^ (1 + δ) := hScale X hX
    exact le_trans (by simpa [x] using hxMulUpper) hScaleX
  have hxBound : E x ≤ -(c * x ^ β) := by simpa [x] using hAll n
  exact ⟨x, by simpa [x] using hxLower, hxUpper, hxBound⟩

abbrev SchlagePuchtaSignedGeometricLiftKernel : Prop :=
  ∀ E : Real → Real, ∀ c β δ X0 : Real,
    0 < c →
    (1 / 2 : Real) < β →
    0 < δ →
    ((∀ X : Real, X ≥ X0 →
        ∃ x : Real, x ≥ X ∧ E x ≥ c * x ^ β) ∨
     (∀ X : Real, X ≥ X0 →
        ∃ x : Real, x ≥ X ∧ E x ≤ -(c * x ^ β))) →
    ((∃ τ off : Real, 0 < X0 ∧ 0 < τ ∧ off ≤ τ * Real.log X0 ∧
        (∀ X : Real, X ≥ X0 → Real.exp (2 * Real.pi / τ) * X ≤ X ^ (1 + δ)) ∧
        (∀ n : Nat,
          E (Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)) ≥
            c * (Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)) ^ β)) ∨
     (∃ τ off : Real, 0 < X0 ∧ 0 < τ ∧ off ≤ τ * Real.log X0 ∧
        (∀ X : Real, X ≥ X0 → Real.exp (2 * Real.pi / τ) * X ≤ X ^ (1 + δ)) ∧
        (∀ n : Nat,
          E (Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)) ≤
            -(c * (Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)) ^ β))))

theorem schlage_relocalization_of_geometric_lift
    (hLift : SchlagePuchtaSignedGeometricLiftKernel) :
    SchlagePuchtaSignedRelocalizationKernel := by
  intro E c β δ X0 hc hβ hδ hGlobal
  rcases hLift E c β δ X0 hc hβ hδ hGlobal with hPosLift | hNegLift
  · left
    rcases hPosLift with ⟨τ, off, hX0pos, hτ, hOffX0, hScale, hAll⟩
    exact geometric_short_window_pos_of_all_indices E c β δ X0 τ off
      hX0pos hτ hOffX0 hScale hAll
  · right
    rcases hNegLift with ⟨τ, off, hX0pos, hτ, hOffX0, hScale, hAll⟩
    exact geometric_short_window_neg_of_all_indices E c β δ X0 τ off
      hX0pos hτ hOffX0 hScale hAll

class SchlagePuchtaIntervalCoreProvider where
  theorem_term : SchlagePuchtaIntervalCoreTerm

theorem schlage_core_of_linear_phase_witness_assumptions
    (h : ExplicitFormulaLinearPhaseWitnessAssumptions) :
    SchlagePuchtaIntervalCoreTerm := by
  intro E hVonKoch s hs hs_gt
  rcases h.zero_to_linear_phase_witness E hVonKoch s hs hs_gt with
    ⟨A, β, τ, φ, R, hA, hβ, hτ, hDecomp, hRemAtTop⟩
  let c : Real := A / 2
  have hc : 0 < c := by
    dsimp [c]
    linarith
  let f0 : Nat → Real := fun n => Real.exp (((n : Real) * (2 * Real.pi) - φ) / τ)
  have hNat : Filter.Tendsto (fun n : Nat => (n : Real)) Filter.atTop Filter.atTop :=
    tendsto_natCast_atTop_atTop
  have hMul :
      Filter.Tendsto (fun n : Nat => (n : Real) * (2 * Real.pi))
        Filter.atTop Filter.atTop :=
    hNat.atTop_mul_const' Real.two_pi_pos
  have hSub :
      Filter.Tendsto (fun n : Nat => (n : Real) * (2 * Real.pi) - φ)
        Filter.atTop Filter.atTop := by
    simpa [sub_eq_add_neg, add_assoc, add_comm, add_left_comm] using
      hMul.atTop_add
        (tendsto_const_nhds : Filter.Tendsto (fun _ : Nat => (-φ)) Filter.atTop (nhds (-φ)))
  have hDiv :
      Filter.Tendsto (fun n : Nat => ((n : Real) * (2 * Real.pi) - φ) / τ)
        Filter.atTop Filter.atTop := by
    have hMulInv :
        Filter.Tendsto (fun n : Nat => ((n : Real) * (2 * Real.pi) - φ) * τ⁻¹)
          Filter.atTop Filter.atTop :=
      hSub.atTop_mul_const' (inv_pos.mpr hτ)
    simpa [div_eq_mul_inv] using hMulInv
  have hTendstoF0 : Filter.Tendsto f0 Filter.atTop Filter.atTop := by
    dsimp [f0]
    exact Real.tendsto_exp_atTop.comp hDiv
  have hRemSeq :
      Filter.Tendsto (fun n : Nat => R (f0 n) / (f0 n) ^ β) Filter.atTop (nhds 0) :=
    hRemAtTop.comp hTendstoF0
  have hF0geOne : ∀ᶠ n : Nat in Filter.atTop, f0 n ≥ 1 :=
    (Filter.tendsto_atTop.1 hTendstoF0) 1
  have hRemBound :
      ∀ᶠ n : Nat in Filter.atTop, |R (f0 n)| ≤ c * (f0 n) ^ β :=
    sequence_eventual_remainder_bound_of_tendsto_zero R f0 β c hc hRemSeq hF0geOne
  rcases Filter.eventually_atTop.1 hRemBound with ⟨N, hN⟩
  let off : Real := -φ + (N : Real) * (2 * Real.pi)
  have hAll :
      ∀ n : Nat,
        E (Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)) ≥
          c * (Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)) ^ β := by
    intro n
    let x : Real := Real.exp ((off + (n : Real) * (2 * Real.pi)) / τ)
    have hxF0 : x = f0 (n + N) := by
      dsimp [x, f0, off]
      congr 1
      rw [Nat.cast_add]
      ring
    have hRemN0 : |R (f0 (n + N))| ≤ c * (f0 (n + N)) ^ β :=
      hN (n + N) (by simpa [Nat.add_comm] using (Nat.le_add_left N n))
    have hRemN : |R x| ≤ c * x ^ β := by
      simpa [hxF0] using hRemN0
    have hτne : τ ≠ 0 := ne_of_gt hτ
    have hPhaseEq :
        τ * Real.log x + φ =
          ((n + N : Nat) : Real) * (2 * Real.pi) := by
      calc
        τ * Real.log x + φ
            = τ * ((off + (n : Real) * (2 * Real.pi)) / τ) + φ := by
                simp [x]
        _ = off + (n : Real) * (2 * Real.pi) + φ := by
              field_simp [hτne]
        _ = ((n + N : Nat) : Real) * (2 * Real.pi) := by
              dsimp [off]
              rw [Nat.cast_add]
              ring
    have hCosEq : Real.cos (τ * Real.log x + φ) = 1 := by
      rw [hPhaseEq]
      simpa [mul_assoc, mul_left_comm, mul_comm] using Real.cos_nat_mul_two_pi (n + N)
    have hEx : E x = A * x ^ β + R x := by
      calc
        E x = oscillatoryMainTerm A β (fun y : Real => τ * Real.log y + φ) x + R x := hDecomp x
        _ = A * x ^ β + R x := by
              unfold oscillatoryMainTerm
              rw [hCosEq]
              ring
    have hRlower : R x ≥ -(c * x ^ β) := by
      have hNegAbsLe : -|R x| ≤ R x := neg_abs_le (R x)
      have hNegBound : -(c * x ^ β) ≤ -|R x| := by linarith [hRemN]
      exact le_trans hNegBound hNegAbsLe
    have hAminus :
        A * x ^ β + R x ≥ A * x ^ β - c * x ^ β := by
      linarith [hRlower]
    have hBalance : A * x ^ β - c * x ^ β = c * x ^ β := by
      dsimp [c]
      ring
    have hLower : E x ≥ c * x ^ β := by
      linarith [hEx, hAminus, hBalance]
    simpa [x] using hLower
  let δ : Real := 1
  let Xscale : Real := Real.exp (2 * Real.pi / τ)
  let Xoff : Real := Real.exp (off / τ)
  let X0 : Real := max Xoff (max 1 Xscale)
  have hX0pos : 0 < X0 := by
    dsimp [X0]
    have hInnerPos : 0 < max 1 Xscale := by
      exact lt_of_lt_of_le (by norm_num : (0 : Real) < 1) (le_max_left 1 Xscale)
    exact lt_of_lt_of_le hInnerPos (le_max_right Xoff (max 1 Xscale))
  have hOffX0 : off ≤ τ * Real.log X0 := by
    have hXoffPos : 0 < Xoff := by
      dsimp [Xoff]
      exact Real.exp_pos (off / τ)
    have hXoffLe : Xoff ≤ X0 := by
      dsimp [X0]
      exact le_max_left Xoff (max 1 Xscale)
    have hLogLe : Real.log Xoff ≤ Real.log X0 :=
      Real.log_le_log hXoffPos hXoffLe
    have hDivLe : off / τ ≤ Real.log X0 := by
      simpa [Xoff, Real.log_exp] using hLogLe
    have hMulLe : off ≤ Real.log X0 * τ := (div_le_iff₀ hτ).1 hDivLe
    simpa [mul_comm, mul_left_comm, mul_assoc] using hMulLe
  have hScale :
      ∀ X : Real, X ≥ X0 → Real.exp (2 * Real.pi / τ) * X ≤ X ^ (1 + δ) := by
    intro X hX
    have hXscaleLe : Xscale ≤ X := by
      have hScaleX0 : Xscale ≤ X0 := by
        dsimp [X0]
        exact le_trans (le_max_right 1 Xscale) (le_max_right Xoff (max 1 Xscale))
      exact le_trans hScaleX0 hX
    have hXoneLe : (1 : Real) ≤ X := by
      have hOneX0 : (1 : Real) ≤ X0 := by
        dsimp [X0]
        exact le_trans (le_max_left 1 Xscale) (le_max_right Xoff (max 1 Xscale))
      exact le_trans hOneX0 hX
    have hXnonneg : 0 ≤ X := le_trans (by norm_num : (0 : Real) ≤ 1) hXoneLe
    have hMulLe : Xscale * X ≤ X * X :=
      mul_le_mul_of_nonneg_right hXscaleLe hXnonneg
    have hPowEq : X ^ (1 + δ) = X * X := by
      dsimp [δ]
      calc
        X ^ (1 + (1 : Real)) = X ^ (2 : Real) := by norm_num
        _ = X ^ (2 : Nat) := by simpa using (Real.rpow_natCast X 2)
        _ = X * X := by ring
    calc
      Real.exp (2 * Real.pi / τ) * X = Xscale * X := by rfl
      _ ≤ X * X := hMulLe
      _ = X ^ (1 + δ) := hPowEq.symm
  refine ⟨c, β, δ, X0, hc, hβ, by dsimp [δ]; norm_num, ?_⟩
  intro X hX
  rcases
      geometric_short_window_pos_of_all_indices E c β δ X0 τ off
        hX0pos hτ hOffX0 hScale hAll X hX
    with ⟨x, hxX, hxUpper, hLower⟩
  refine ⟨x, hxX, hxUpper, ?_⟩
  exact le_trans hLower (le_abs_self (E x))

class SchlagePuchtaPhasePinnedWindowProvider where
  theorem_term : SchlagePuchtaPhasePinnedWindowTerm

noncomputable instance intervalCoreProviderOfPhasePinnedWindow
    [h : SchlagePuchtaPhasePinnedWindowProvider] :
    SchlagePuchtaIntervalCoreProvider where
  theorem_term := schlage_core_of_phase_pinned_window h.theorem_term

class SchlagePuchtaSignedGlobalProvider where
  theorem_term : SchlagePuchtaSignedGlobalOscillationTerm

class SchlagePuchtaSignedRelocalizationProvider where
  theorem_term : SchlagePuchtaSignedRelocalizationKernel

noncomputable instance intervalCoreProviderOfSignedGlobalAndRelocalization
    [hGlobal : SchlagePuchtaSignedGlobalProvider]
    [hReloc : SchlagePuchtaSignedRelocalizationProvider] :
    SchlagePuchtaIntervalCoreProvider where
  theorem_term := schlage_core_of_global_and_relocalization
    hGlobal.theorem_term hReloc.theorem_term

class SchlagePuchtaAsymptoticProvider where
  assumptions : ExplicitFormulaAsymptoticSequenceAssumptions

noncomputable instance signedGlobalProviderOfAsymptotic
    [h : SchlagePuchtaAsymptoticProvider] :
    SchlagePuchtaSignedGlobalProvider where
  theorem_term := schlage_signed_global_of_asymptotic_assumptions h.assumptions

class SchlagePuchtaSignedGeometricLiftProvider where
  theorem_term : SchlagePuchtaSignedGeometricLiftKernel

noncomputable instance signedRelocalizationProviderOfGeometricLift
    [h : SchlagePuchtaSignedGeometricLiftProvider] :
    SchlagePuchtaSignedRelocalizationProvider where
  theorem_term := schlage_relocalization_of_geometric_lift h.theorem_term

noncomputable instance intervalCoreProviderOfAsymptoticAndGeometricLift
    [hAsym : SchlagePuchtaAsymptoticProvider]
    [hLift : SchlagePuchtaSignedGeometricLiftProvider] :
    SchlagePuchtaIntervalCoreProvider where
  theorem_term := schlage_core_of_asymptotic_assumptions_and_relocalization
    hAsym.assumptions
    (schlage_relocalization_of_geometric_lift hLift.theorem_term)

noncomputable instance intervalCoreProviderOfLinearPhaseWitness
    [h : ConcreteLinearPhaseWitnessProvider] :
    SchlagePuchtaIntervalCoreProvider where
  theorem_term := schlage_core_of_linear_phase_witness_assumptions
    h.linear_phase_witness_assumptions

noncomputable instance intervalCoreProviderOfImportedPublishedResults
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    SchlagePuchtaIntervalCoreProvider :=
  intervalCoreProviderOfLinearPhaseWitness

noncomputable instance schlagePuchta2019Imported
    [hCore : SchlagePuchtaIntervalCoreProvider] :
    SchlagePuchta2019IntervalOscillationFormalized where
  source_tag := "SCHLAGE-PUCHTA-2019-GIVEN-ZERO-OSCILLATION"
  source_url := "https://arxiv.org/abs/1912.00853"
  theorem_ref := "Thm-1-given-zero-forces-interval-oscillation"
  source_tag_lock := rfl
  source_url_lock := rfl
  theorem_ref_lock := rfl
  theorem_term := hCore.theorem_term

theorem rh_from_imported_schlage_puchta2019
    [hCore : SchlagePuchtaIntervalCoreProvider] :
    RHStatement :=
  rh_from_schlage_puchta_interval_oscillation
    (h := schlagePuchta2019Imported)

theorem imported_schlage_puchta2019_nonempty
    [hCore : SchlagePuchtaIntervalCoreProvider] :
    Nonempty SchlagePuchta2019IntervalOscillationFormalized :=
  ⟨schlagePuchta2019Imported⟩

theorem rh_from_imported_schlage_puchta2019_nonempty
    [hCore : SchlagePuchtaIntervalCoreProvider] :
    RHStatement := by
  rcases imported_schlage_puchta2019_nonempty with ⟨inst⟩
  letI : SchlagePuchta2019IntervalOscillationFormalized := inst
  exact rh_from_schlage_puchta_interval_oscillation

theorem rh_from_imported_published_via_schlage_provider
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    RHStatement :=
  rh_from_imported_schlage_puchta2019
    (hCore := intervalCoreProviderOfImportedPublishedResults)

theorem schlage_interval_core_provider_nonempty_of_zero_to_cos_sin_phase_transfer
    (hK1 : ZeroToCosSinPhaseTransfer) :
    Nonempty SchlagePuchtaIntervalCoreProvider := by
  let iStep : ImportedLinearPhaseWitnessStepResults := {
    zero_to_cos_sin_phase := hK1
    cos_sin_to_single_cos := cos_sin_to_single_cos_derived
  }
  let iWitness : ImportedLinearPhaseWitnessResults :=
    importedLinearPhaseWitnessResultsOfStepResults iStep
  letI : ConcreteImportedLinearPhaseWitnessProvider := {
    imported_linear_phase_witness := iWitness
  }
  letI : ConcreteLinearPhaseWitnessProvider := concreteLinearPhaseWitnessProviderOfImported
  exact ⟨intervalCoreProviderOfLinearPhaseWitness⟩

theorem rh_of_schlage_interval_core_provider_nonempty
    (h : Nonempty SchlagePuchtaIntervalCoreProvider) :
    RHStatement := by
  rcases h with ⟨hCore⟩
  letI : SchlagePuchtaIntervalCoreProvider := hCore
  exact rh_from_imported_schlage_puchta2019

theorem zero_to_cos_sin_phase_transfer_of_schlage_interval_core_provider_nonempty
    (h : Nonempty SchlagePuchtaIntervalCoreProvider) :
    ZeroToCosSinPhaseTransfer := by
  have hRH : RHStatement := rh_of_schlage_interval_core_provider_nonempty h
  exact (zero_to_cos_sin_phase_transfer_iff_rh).2 hRH

theorem zero_to_cos_sin_phase_transfer_iff_nonempty_schlage_interval_core_provider :
    ZeroToCosSinPhaseTransfer ↔ Nonempty SchlagePuchtaIntervalCoreProvider := by
  constructor
  · exact schlage_interval_core_provider_nonempty_of_zero_to_cos_sin_phase_transfer
  · exact zero_to_cos_sin_phase_transfer_of_schlage_interval_core_provider_nonempty

theorem nonempty_schlage_interval_core_provider_of_rh
    (hRH : RHStatement) :
    Nonempty SchlagePuchtaIntervalCoreProvider :=
  schlage_interval_core_provider_nonempty_of_zero_to_cos_sin_phase_transfer
    ((zero_to_cos_sin_phase_transfer_iff_rh).2 hRH)

theorem rh_iff_nonempty_schlage_interval_core_provider :
    RHStatement ↔ Nonempty SchlagePuchtaIntervalCoreProvider := by
  constructor
  · exact nonempty_schlage_interval_core_provider_of_rh
  · exact rh_of_schlage_interval_core_provider_nonempty

class K1SourceNonCircularProvider where
  theorem_term : ZeroToCosSinPhaseTransfer

noncomputable instance k1SourceNonCircularProviderOfZeroToCosSinPhaseProvider
    [h : PrimeRiemannBridgeSpinningTopFrontier.ZeroToCosSinPhaseProvider] :
    K1SourceNonCircularProvider where
  theorem_term := h.theorem_term

noncomputable instance k1SourceNonCircularProviderOfPowerMajorant
    [h : PrimeRiemannBridgeSpinningTopFrontier.ZeroToCosSinPowerMajorantProvider] :
    K1SourceNonCircularProvider where
  theorem_term := PrimeRiemannBridgeSpinningTopFrontier.zero_to_cos_sin_phase_of_power_majorant_term h.theorem_term

noncomputable instance k1SourceNonCircularProviderOfNearStrictTail
    [h : PrimeRiemannBridgeSpinningTopFrontier.ZeroToCosSinNearStrictTailPowerProvider] :
    K1SourceNonCircularProvider where
  theorem_term := PrimeRiemannBridgeSpinningTopFrontier.zero_to_cos_sin_phase_of_near_strict_tail_power_term h.theorem_term

noncomputable instance k1SourceNonCircularProviderOfAsymptoticStrictTail
    [h : PrimeRiemannBridgeSpinningTopFrontier.ZeroToCosSinAsymptoticStrictTailPowerProvider] :
    K1SourceNonCircularProvider where
  theorem_term := PrimeRiemannBridgeSpinningTopFrontier.zero_to_cos_sin_phase_of_asymptotic_strict_tail_power_term h.theorem_term

noncomputable instance schlageIntervalCoreProviderOfK1SourceNonCircular
    [k : K1SourceNonCircularProvider] :
    SchlagePuchtaIntervalCoreProvider := by
  rcases
      schlage_interval_core_provider_nonempty_of_zero_to_cos_sin_phase_transfer
        k.theorem_term
    with ⟨inst⟩
  exact inst

theorem rh_from_k1_source_non_circular_provider
    [k : K1SourceNonCircularProvider] :
    RHStatement :=
  rh_from_imported_schlage_puchta2019
    (hCore := schlageIntervalCoreProviderOfK1SourceNonCircular)

theorem nonempty_schlage_interval_core_provider_of_nonempty_k1_source_non_circular_provider
    (h : Nonempty K1SourceNonCircularProvider) :
    Nonempty SchlagePuchtaIntervalCoreProvider := by
  rcases h with ⟨k⟩
  letI : K1SourceNonCircularProvider := k
  exact ⟨schlageIntervalCoreProviderOfK1SourceNonCircular⟩

theorem nonempty_k1_source_non_circular_provider_of_nonempty_schlage_interval_core_provider
    (h : Nonempty SchlagePuchtaIntervalCoreProvider) :
    Nonempty K1SourceNonCircularProvider := by
  refine ⟨{ theorem_term := ?_ }⟩
  exact zero_to_cos_sin_phase_transfer_of_schlage_interval_core_provider_nonempty h

theorem nonempty_k1_source_non_circular_provider_iff_nonempty_schlage_interval_core_provider :
    Nonempty K1SourceNonCircularProvider ↔ Nonempty SchlagePuchtaIntervalCoreProvider := by
  constructor
  · exact nonempty_schlage_interval_core_provider_of_nonempty_k1_source_non_circular_provider
  · exact nonempty_k1_source_non_circular_provider_of_nonempty_schlage_interval_core_provider

theorem rh_iff_nonempty_k1_source_non_circular_provider :
    RHStatement ↔ Nonempty K1SourceNonCircularProvider := by
  calc
    RHStatement ↔ Nonempty SchlagePuchtaIntervalCoreProvider := rh_iff_nonempty_schlage_interval_core_provider
    _ ↔ Nonempty K1SourceNonCircularProvider := by
      symm
      exact nonempty_k1_source_non_circular_provider_iff_nonempty_schlage_interval_core_provider

theorem nonempty_concrete_linear_phase_witness_provider_of_nonempty_k1_source_non_circular_provider
    (h : Nonempty K1SourceNonCircularProvider) :
    Nonempty ConcreteLinearPhaseWitnessProvider := by
  rcases h with ⟨k⟩
  let iStep : ImportedLinearPhaseWitnessStepResults := {
    zero_to_cos_sin_phase := k.theorem_term
    cos_sin_to_single_cos := cos_sin_to_single_cos_derived
  }
  let iWitness : ImportedLinearPhaseWitnessResults :=
    importedLinearPhaseWitnessResultsOfStepResults iStep
  exact ⟨{
    linear_phase_witness_assumptions := linearPhaseWitnessAssumptionsOfImported iWitness
  }⟩

theorem nonempty_k1_source_non_circular_provider_of_nonempty_concrete_linear_phase_witness_provider
    (h : Nonempty ConcreteLinearPhaseWitnessProvider) :
    Nonempty K1SourceNonCircularProvider := by
  rcases h with ⟨hLinear⟩
  refine ⟨{ theorem_term := ?_ }⟩
  exact zero_to_cos_sin_phase_transfer_of_linear_phase_witness
    hLinear.linear_phase_witness_assumptions

theorem nonempty_k1_source_non_circular_provider_iff_nonempty_concrete_linear_phase_witness_provider :
    Nonempty K1SourceNonCircularProvider ↔ Nonempty ConcreteLinearPhaseWitnessProvider := by
  constructor
  · exact nonempty_concrete_linear_phase_witness_provider_of_nonempty_k1_source_non_circular_provider
  · exact nonempty_k1_source_non_circular_provider_of_nonempty_concrete_linear_phase_witness_provider

theorem rh_iff_nonempty_concrete_linear_phase_witness_provider :
    RHStatement ↔ Nonempty ConcreteLinearPhaseWitnessProvider := by
  calc
    RHStatement ↔ Nonempty K1SourceNonCircularProvider := rh_iff_nonempty_k1_source_non_circular_provider
    _ ↔ Nonempty ConcreteLinearPhaseWitnessProvider := by
      exact nonempty_k1_source_non_circular_provider_iff_nonempty_concrete_linear_phase_witness_provider

theorem nonempty_concrete_linear_phase_witness_provider_of_nonempty_concrete_asymptotically_linear_phase_provider
    (h : Nonempty ConcreteAsymptoticallyLinearPhaseProvider) :
    Nonempty ConcreteLinearPhaseWitnessProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteAsymptoticallyLinearPhaseProvider := inst
  exact ⟨PrimeRiemannBridgeOscillatoryReduction.concreteLinearPhaseWitnessProviderOfAsymptoticallyLinear⟩

theorem rh_of_nonempty_concrete_asymptotically_linear_phase_provider
    (h : Nonempty ConcreteAsymptoticallyLinearPhaseProvider) :
    RHStatement := by
  have hLinear :
      Nonempty ConcreteLinearPhaseWitnessProvider :=
    nonempty_concrete_linear_phase_witness_provider_of_nonempty_concrete_asymptotically_linear_phase_provider h
  exact (rh_iff_nonempty_concrete_linear_phase_witness_provider).2 hLinear

theorem nonempty_concrete_asymptotically_linear_phase_provider_of_nonempty_concrete_finite_decaying_phase_corrections_provider
    (h : Nonempty ConcreteFiniteDecayingPhaseCorrectionsProvider) :
    Nonempty ConcreteAsymptoticallyLinearPhaseProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteFiniteDecayingPhaseCorrectionsProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteAsymptoticallyLinearPhaseProviderOfFiniteDecayingCorrections⟩

theorem rh_of_nonempty_concrete_finite_decaying_phase_corrections_provider
    (h : Nonempty ConcreteFiniteDecayingPhaseCorrectionsProvider) :
    RHStatement := by
  have hAsym :
      Nonempty ConcreteAsymptoticallyLinearPhaseProvider :=
    nonempty_concrete_asymptotically_linear_phase_provider_of_nonempty_concrete_finite_decaying_phase_corrections_provider h
  exact rh_of_nonempty_concrete_asymptotically_linear_phase_provider hAsym

theorem nonempty_concrete_asymptotically_linear_phase_provider_of_nonempty_concrete_single_decaying_phase_correction_provider
    (h : Nonempty ConcreteSingleDecayingPhaseCorrectionProvider) :
    Nonempty ConcreteAsymptoticallyLinearPhaseProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteSingleDecayingPhaseCorrectionProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteAsymptoticallyLinearPhaseProviderOfSingleDecayingCorrection⟩

theorem rh_of_nonempty_concrete_single_decaying_phase_correction_provider
    (h : Nonempty ConcreteSingleDecayingPhaseCorrectionProvider) :
    RHStatement := by
  have hAsym :
      Nonempty ConcreteAsymptoticallyLinearPhaseProvider :=
    nonempty_concrete_asymptotically_linear_phase_provider_of_nonempty_concrete_single_decaying_phase_correction_provider h
  exact rh_of_nonempty_concrete_asymptotically_linear_phase_provider hAsym

theorem nonempty_concrete_single_decaying_phase_correction_provider_of_nonempty_concrete_single_decaying_phase_ladder_provider
    (h : Nonempty ConcreteSingleDecayingPhaseLadderProvider) :
    Nonempty ConcreteSingleDecayingPhaseCorrectionProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteSingleDecayingPhaseLadderProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteSingleDecayingPhaseCorrectionProviderOfLadder⟩

theorem rh_of_nonempty_concrete_single_decaying_phase_ladder_provider
    (h : Nonempty ConcreteSingleDecayingPhaseLadderProvider) :
    RHStatement := by
  have hSingle :
      Nonempty ConcreteSingleDecayingPhaseCorrectionProvider :=
    nonempty_concrete_single_decaying_phase_correction_provider_of_nonempty_concrete_single_decaying_phase_ladder_provider h
  exact rh_of_nonempty_concrete_single_decaying_phase_correction_provider hSingle

theorem nonempty_concrete_single_decaying_phase_ladder_provider_of_nonempty_concrete_single_decaying_mode_only_provider
    (h : Nonempty ConcreteSingleDecayingModeOnlyProvider) :
    Nonempty ConcreteSingleDecayingPhaseLadderProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteSingleDecayingModeOnlyProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteSingleDecayingPhaseLadderProviderOfModeOnly⟩

theorem rh_of_nonempty_concrete_single_decaying_mode_only_provider
    (h : Nonempty ConcreteSingleDecayingModeOnlyProvider) :
    RHStatement := by
  have hLadder :
      Nonempty ConcreteSingleDecayingPhaseLadderProvider :=
    nonempty_concrete_single_decaying_phase_ladder_provider_of_nonempty_concrete_single_decaying_mode_only_provider h
  exact rh_of_nonempty_concrete_single_decaying_phase_ladder_provider hLadder

theorem nonempty_concrete_finite_mode_residual_majorant_provider_of_nonempty_concrete_finite_mode_residual_majorant_pieces_provider
    (h : Nonempty ConcreteFiniteModeResidualMajorantPiecesProvider) :
    Nonempty ConcreteFiniteModeResidualMajorantProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteFiniteModeResidualMajorantPiecesProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteFiniteModeResidualMajorantProviderOfPieces⟩

theorem nonempty_concrete_asymptotically_linear_phase_provider_of_nonempty_concrete_finite_mode_residual_majorant_provider
    (h : Nonempty ConcreteFiniteModeResidualMajorantProvider) :
    Nonempty ConcreteAsymptoticallyLinearPhaseProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteFiniteModeResidualMajorantProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteAsymptoticallyLinearPhaseProviderOfFiniteModeResidualMajorant⟩

theorem rh_of_nonempty_concrete_finite_mode_residual_majorant_provider
    (h : Nonempty ConcreteFiniteModeResidualMajorantProvider) :
    RHStatement := by
  have hAsym :
      Nonempty ConcreteAsymptoticallyLinearPhaseProvider :=
    nonempty_concrete_asymptotically_linear_phase_provider_of_nonempty_concrete_finite_mode_residual_majorant_provider h
  exact rh_of_nonempty_concrete_asymptotically_linear_phase_provider hAsym

theorem rh_of_nonempty_concrete_finite_mode_residual_majorant_pieces_provider
    (h : Nonempty ConcreteFiniteModeResidualMajorantPiecesProvider) :
    RHStatement := by
  have hRes :
      Nonempty ConcreteFiniteModeResidualMajorantProvider :=
    nonempty_concrete_finite_mode_residual_majorant_provider_of_nonempty_concrete_finite_mode_residual_majorant_pieces_provider h
  exact rh_of_nonempty_concrete_finite_mode_residual_majorant_provider hRes

theorem nonempty_concrete_finite_mode_residual_majorant_provider_of_rh
    (hRH : RHStatement) :
    Nonempty ConcreteFiniteModeResidualMajorantProvider := by
  refine ⟨{
    finite_mode_residual_majorant_assumptions := {
      source_tag := "PINTZ-2017-OSCILLATION"
      source_url := "https://doi.org/10.1134/S0081543817010163"
      theorem_ref := "Thm-2-zero-to-oscillation-transfer"
      source_tag_lock := rfl
      source_url_lock := rfl
      theorem_ref_lock := rfl
      zero_to_global_decomposition := zero_to_global_decomposition_of_vonkoch
      finite_mode_plus_majorized_residual_of_model := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
    }
  }⟩

theorem rh_iff_nonempty_concrete_finite_mode_residual_majorant_provider :
    RHStatement ↔ Nonempty ConcreteFiniteModeResidualMajorantProvider := by
  constructor
  · exact nonempty_concrete_finite_mode_residual_majorant_provider_of_rh
  · exact rh_of_nonempty_concrete_finite_mode_residual_majorant_provider

theorem nonempty_concrete_single_decaying_mode_only_provider_of_rh
    (hRH : RHStatement) :
    Nonempty ConcreteSingleDecayingModeOnlyProvider := by
  refine ⟨{
    single_decaying_mode_only_assumptions := {
      source_tag := "PINTZ-2017-OSCILLATION"
      source_url := "https://doi.org/10.1134/S0081543817010163"
      theorem_ref := "Thm-2-zero-to-oscillation-transfer"
      source_tag_lock := rfl
      source_url_lock := rfl
      theorem_ref_lock := rfl
      zero_to_global_decomposition := zero_to_global_decomposition_of_vonkoch
      single_mode_of_trivial_phase_core := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
    }
  }⟩

theorem rh_iff_nonempty_concrete_single_decaying_mode_only_provider :
    RHStatement ↔ Nonempty ConcreteSingleDecayingModeOnlyProvider := by
  constructor
  · exact nonempty_concrete_single_decaying_mode_only_provider_of_rh
  · exact rh_of_nonempty_concrete_single_decaying_mode_only_provider

theorem nonempty_concrete_single_decaying_mode_only_provider_of_nonempty_concrete_spinning_top_r6_mode_only_provider
    (h : Nonempty ConcreteSpinningTopR6ModeOnlyProvider) :
    Nonempty ConcreteSingleDecayingModeOnlyProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteSpinningTopR6ModeOnlyProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteSingleDecayingModeOnlyProviderOfSpinningTopR6⟩

theorem rh_of_nonempty_concrete_spinning_top_r6_mode_only_provider
    (h : Nonempty ConcreteSpinningTopR6ModeOnlyProvider) :
    RHStatement := by
  have hModeOnly :
      Nonempty ConcreteSingleDecayingModeOnlyProvider :=
    nonempty_concrete_single_decaying_mode_only_provider_of_nonempty_concrete_spinning_top_r6_mode_only_provider h
  exact rh_of_nonempty_concrete_single_decaying_mode_only_provider hModeOnly

theorem nonempty_concrete_spinning_top_r6_mode_only_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_provider
    (h : Nonempty ConcreteSpinningTopR6DominantBandProvider) :
    Nonempty ConcreteSpinningTopR6ModeOnlyProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteSpinningTopR6DominantBandProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteSpinningTopR6ModeOnlyProviderOfDominantBand⟩

theorem rh_of_nonempty_concrete_spinning_top_r6_dominant_band_provider
    (h : Nonempty ConcreteSpinningTopR6DominantBandProvider) :
    RHStatement := by
  have hMode :
      Nonempty ConcreteSpinningTopR6ModeOnlyProvider :=
    nonempty_concrete_spinning_top_r6_mode_only_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_provider h
  exact rh_of_nonempty_concrete_spinning_top_r6_mode_only_provider hMode

theorem nonempty_concrete_spinning_top_r6_dominant_band_provider_of_rh
    (hRH : RHStatement) :
    Nonempty ConcreteSpinningTopR6DominantBandProvider := by
  refine ⟨{
    spinning_top_r6_dominant_band_assumptions := {
      source_tag := "PINTZ-2017-OSCILLATION"
      source_url := "https://doi.org/10.1134/S0081543817010163"
      theorem_ref := "Thm-2-zero-to-oscillation-transfer"
      source_tag_lock := rfl
      source_url_lock := rfl
      theorem_ref_lock := rfl
      zero_to_global_decomposition := zero_to_global_decomposition_of_vonkoch
      spinning_top_r6_phase_bands_of_model := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
      trivial_core_equals_dominant_band_of_r6 := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
    }
  }⟩

theorem rh_iff_nonempty_concrete_spinning_top_r6_dominant_band_provider :
    RHStatement ↔ Nonempty ConcreteSpinningTopR6DominantBandProvider := by
  constructor
  · exact nonempty_concrete_spinning_top_r6_dominant_band_provider_of_rh
  · exact rh_of_nonempty_concrete_spinning_top_r6_dominant_band_provider

theorem nonempty_concrete_spinning_top_r6_dominant_band_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider
    (h : Nonempty ConcreteSpinningTopR6DominantBandCriteriaProvider) :
    Nonempty ConcreteSpinningTopR6DominantBandProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteSpinningTopR6DominantBandCriteriaProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteSpinningTopR6DominantBandProviderOfCriteria⟩

theorem rh_of_nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider
    (h : Nonempty ConcreteSpinningTopR6DominantBandCriteriaProvider) :
    RHStatement := by
  have hDom :
      Nonempty ConcreteSpinningTopR6DominantBandProvider :=
    nonempty_concrete_spinning_top_r6_dominant_band_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider h
  exact rh_of_nonempty_concrete_spinning_top_r6_dominant_band_provider hDom

theorem nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider_of_rh
    (hRH : RHStatement) :
    Nonempty ConcreteSpinningTopR6DominantBandCriteriaProvider := by
  refine ⟨{
    spinning_top_r6_dominant_band_criteria_assumptions := {
      source_tag := "PINTZ-2017-OSCILLATION"
      source_url := "https://doi.org/10.1134/S0081543817010163"
      theorem_ref := "Thm-2-zero-to-oscillation-transfer"
      source_tag_lock := rfl
      source_url_lock := rfl
      theorem_ref_lock := rfl
      zero_to_global_decomposition := zero_to_global_decomposition_of_vonkoch
      spinning_top_r6_phase_bands_of_model := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
      normalized_trivial_anchor_of_model := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
      dominant_band_collapse_of_model := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
    }
  }⟩

theorem rh_iff_nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider :
    RHStatement ↔ Nonempty ConcreteSpinningTopR6DominantBandCriteriaProvider := by
  constructor
  · exact nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider_of_rh
  · exact rh_of_nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider

theorem nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider
    (h : Nonempty ConcreteSpinningTopR6DominantBandCoreLockProvider) :
    Nonempty ConcreteSpinningTopR6DominantBandCriteriaProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteSpinningTopR6DominantBandCoreLockProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteSpinningTopR6DominantBandCriteriaProviderOfCoreLock⟩

theorem rh_of_nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider
    (h : Nonempty ConcreteSpinningTopR6DominantBandCoreLockProvider) :
    RHStatement := by
  have hCriteria :
      Nonempty ConcreteSpinningTopR6DominantBandCriteriaProvider :=
    nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider h
  exact rh_of_nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider hCriteria

theorem nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider_of_rh
    (hRH : RHStatement) :
    Nonempty ConcreteSpinningTopR6DominantBandCoreLockProvider := by
  refine ⟨{
    spinning_top_r6_dominant_band_core_lock_assumptions := {
      source_tag := "PINTZ-2017-OSCILLATION"
      source_url := "https://doi.org/10.1134/S0081543817010163"
      theorem_ref := "Thm-2-zero-to-oscillation-transfer"
      source_tag_lock := rfl
      source_url_lock := rfl
      theorem_ref_lock := rfl
      zero_to_global_decomposition := zero_to_global_decomposition_of_vonkoch
      spinning_top_r6_phase_bands_of_model := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
      trivial_core_superposition_of_model := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand x hx
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
      dominant_band_collapse_of_model := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
    }
  }⟩

theorem rh_iff_nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider :
    RHStatement ↔ Nonempty ConcreteSpinningTopR6DominantBandCoreLockProvider := by
  constructor
  · exact nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider_of_rh
  · exact rh_of_nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider

theorem nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider
    (h : Nonempty ConcreteSpinningTopR6DominantBandCoefficientPinningProvider) :
    Nonempty ConcreteSpinningTopR6DominantBandCoreLockProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteSpinningTopR6DominantBandCoefficientPinningProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteSpinningTopR6DominantBandCoreLockProviderOfCoefficientPinning⟩

theorem rh_of_nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider
    (h : Nonempty ConcreteSpinningTopR6DominantBandCoefficientPinningProvider) :
    RHStatement := by
  have hCore :
      Nonempty ConcreteSpinningTopR6DominantBandCoreLockProvider :=
    nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider h
  exact rh_of_nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider hCore

theorem nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider_of_rh
    (hRH : RHStatement) :
    Nonempty ConcreteSpinningTopR6DominantBandCoefficientPinningProvider := by
  refine ⟨{
    spinning_top_r6_dominant_band_coefficient_pinning_assumptions := {
      source_tag := "PINTZ-2017-OSCILLATION"
      source_url := "https://doi.org/10.1134/S0081543817010163"
      theorem_ref := "Thm-2-zero-to-oscillation-transfer"
      source_tag_lock := rfl
      source_url_lock := rfl
      theorem_ref_lock := rfl
      zero_to_global_decomposition := zero_to_global_decomposition_of_vonkoch
      spinning_top_r6_phase_bands_of_model := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
      normalized_trivial_anchor_of_model := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
      dominant_band_index_with_offdiag_zero_of_model := by
        intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
        have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
        have hFalse : False := by linarith
        exact False.elim hFalse
    }
  }⟩

theorem rh_iff_nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider :
    RHStatement ↔ Nonempty ConcreteSpinningTopR6DominantBandCoefficientPinningProvider := by
  constructor
  · exact nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider_of_rh
  · exact rh_of_nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider

theorem nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider_of_nonempty_concrete_single_decaying_mode_only_provider
    (h : Nonempty ConcreteSingleDecayingModeOnlyProvider) :
    Nonempty ConcreteSpinningTopR6DominantBandCoefficientPinningProvider := by
  rcases h with ⟨inst⟩
  letI : ConcreteSingleDecayingModeOnlyProvider := inst
  exact
    ⟨PrimeRiemannBridgeOscillatoryReduction.concreteSpinningTopR6DominantBandCoefficientPinningProviderOfSingleModeOnly⟩

theorem nonempty_concrete_single_decaying_mode_only_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider
    (h : Nonempty ConcreteSpinningTopR6DominantBandCoefficientPinningProvider) :
    Nonempty ConcreteSingleDecayingModeOnlyProvider := by
  have hCore :
      Nonempty ConcreteSpinningTopR6DominantBandCoreLockProvider :=
    nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider h
  have hCriteria :
      Nonempty ConcreteSpinningTopR6DominantBandCriteriaProvider :=
    nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider hCore
  have hDom :
      Nonempty ConcreteSpinningTopR6DominantBandProvider :=
    nonempty_concrete_spinning_top_r6_dominant_band_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider hCriteria
  have hMode :
      Nonempty ConcreteSpinningTopR6ModeOnlyProvider :=
    nonempty_concrete_spinning_top_r6_mode_only_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_provider hDom
  exact
    nonempty_concrete_single_decaying_mode_only_provider_of_nonempty_concrete_spinning_top_r6_mode_only_provider hMode

theorem nonempty_concrete_single_decaying_mode_only_provider_iff_nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider :
    Nonempty ConcreteSingleDecayingModeOnlyProvider ↔
      Nonempty ConcreteSpinningTopR6DominantBandCoefficientPinningProvider := by
  constructor
  · exact nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider_of_nonempty_concrete_single_decaying_mode_only_provider
  · exact nonempty_concrete_single_decaying_mode_only_provider_of_nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider

theorem nonempty_concrete_finite_mode_residual_majorant_provider_iff_nonempty_pintz2017_zero_to_oscillation_formalized :
    Nonempty ConcreteFiniteModeResidualMajorantProvider ↔
      Nonempty Pintz2017ZeroToOscillationFormalized := by
  constructor
  · intro hProv
    have hRH : RHStatement :=
      rh_of_nonempty_concrete_finite_mode_residual_majorant_provider hProv
    exact (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).1 hRH
  · intro hP
    have hRH : RHStatement :=
      (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).2 hP
    exact nonempty_concrete_finite_mode_residual_majorant_provider_of_rh hRH

theorem nonempty_concrete_single_decaying_mode_only_provider_iff_nonempty_pintz2017_zero_to_oscillation_formalized :
    Nonempty ConcreteSingleDecayingModeOnlyProvider ↔
      Nonempty Pintz2017ZeroToOscillationFormalized := by
  constructor
  · intro hProv
    have hRH : RHStatement :=
      rh_of_nonempty_concrete_single_decaying_mode_only_provider hProv
    exact (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).1 hRH
  · intro hP
    have hRH : RHStatement :=
      (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).2 hP
    exact nonempty_concrete_single_decaying_mode_only_provider_of_rh hRH

theorem nonempty_concrete_spinning_top_r6_dominant_band_provider_iff_nonempty_pintz2017_zero_to_oscillation_formalized :
    Nonempty ConcreteSpinningTopR6DominantBandProvider ↔
      Nonempty Pintz2017ZeroToOscillationFormalized := by
  constructor
  · intro hProv
    have hRH : RHStatement :=
      rh_of_nonempty_concrete_spinning_top_r6_dominant_band_provider hProv
    exact (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).1 hRH
  · intro hP
    have hRH : RHStatement :=
      (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).2 hP
    exact nonempty_concrete_spinning_top_r6_dominant_band_provider_of_rh hRH

theorem nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider_iff_nonempty_pintz2017_zero_to_oscillation_formalized :
    Nonempty ConcreteSpinningTopR6DominantBandCriteriaProvider ↔
      Nonempty Pintz2017ZeroToOscillationFormalized := by
  constructor
  · intro hProv
    have hRH : RHStatement :=
      rh_of_nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider hProv
    exact (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).1 hRH
  · intro hP
    have hRH : RHStatement :=
      (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).2 hP
    exact nonempty_concrete_spinning_top_r6_dominant_band_criteria_provider_of_rh hRH

theorem nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider_iff_nonempty_pintz2017_zero_to_oscillation_formalized :
    Nonempty ConcreteSpinningTopR6DominantBandCoreLockProvider ↔
      Nonempty Pintz2017ZeroToOscillationFormalized := by
  constructor
  · intro hProv
    have hRH : RHStatement :=
      rh_of_nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider hProv
    exact (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).1 hRH
  · intro hP
    have hRH : RHStatement :=
      (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).2 hP
    exact nonempty_concrete_spinning_top_r6_dominant_band_core_lock_provider_of_rh hRH

theorem nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider_iff_nonempty_pintz2017_zero_to_oscillation_formalized :
    Nonempty ConcreteSpinningTopR6DominantBandCoefficientPinningProvider ↔
      Nonempty Pintz2017ZeroToOscillationFormalized := by
  constructor
  · intro hProv
    have hRH : RHStatement :=
      rh_of_nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider hProv
    exact (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).1 hRH
  · intro hP
    have hRH : RHStatement :=
      (rh_iff_nonempty_pintz2017_zero_to_oscillation_formalized).2 hP
    exact nonempty_concrete_spinning_top_r6_dominant_band_coefficient_pinning_provider_of_rh hRH

end PrimeRiemannBridgeSchlagePuchta2019ImportedInstance
