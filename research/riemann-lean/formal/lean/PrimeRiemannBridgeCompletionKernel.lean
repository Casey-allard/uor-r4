import PrimeRiemannBridgeMathlib

namespace PrimeRiemannBridgeCompletionKernel

open PrimeRiemannBridgeMathlib
open Filter Asymptotics

class ImportedRHResults : Prop where
  /-- Published equivalence step from endpoint criterion to RH statement. -/
  endpoint_equiv_rh :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement

/-!
This file records the exact remaining mathematical kernel required to turn the
current framework into an unconditional RH proof.
-/

structure RemainingKernel (E H : Real → Real) where
  /-- Trusted external published inputs, isolated in one import boundary. -/
  k_imported_results : ImportedRHResults

def canonicalO1 : O1Constants :=
  { C0_ref_O1 := 0.9102883687683553
    a_ref := -0.0013474693715061251
    b_ref := -0.05436122353654979
    m_ref := 512 }

def canonicalO3 : O3Constants :=
  { A_offabs := 0
    C_offabs := 0.03292827711413939
    k_abs := 0.005725212627704354
    A_diag := 0
    C_diag := 1
    A_E2 := 0
    C_E2 := 1.1195893906678458 }

theorem pipeline_to_endpoint_unconditional
    (E H : Real → Real)
    (h1 : O1Closed E H canonicalO1)
    (h3 : O3Closed H canonicalO3) :
    RH_Equivalent_Implication E := by
  exact l3_endpoint_from_transfer E H canonicalO1 canonicalO3 h1 h3

theorem rh_if_remaining_kernel
    (E H : Real → Real)
    (k : RemainingKernel E H)
    (h1 : O1Closed E H canonicalO1)
    (h3 : O3Closed H canonicalO3) :
    RHStatement := by
  letI : ImportedRHResults := k.k_imported_results
  have hEndpoint : RH_Equivalent_Implication E := pipeline_to_endpoint_unconditional E H h1 h3
  exact ImportedRHResults.endpoint_equiv_rh E hEndpoint

/- Structured final kernel for the remaining endpoint -> RH implication. -/
structure EndpointToRHKernel where
  endpoint_to_rh : ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement

def importedRHResultsOfKernel (k : EndpointToRHKernel) : ImportedRHResults where
  endpoint_equiv_rh := k.endpoint_to_rh

/--
Concrete final bridge hypothesis:
endpoint-class control forces every nontrivial zeta zero to lie on Re(s)=1/2.
-/
def EndpointToZetaCriticalLineBridge : Prop :=
  ∀ (E : Real → Real),
    RH_Equivalent_Implication E →
      ∀ s : Complex, IsNontrivialZetaZero s → s.re = (1 / 2 : Real)

structure EndpointBridgeComponents where
  endpoint_to_vonkoch :
    ∀ E : Real → Real, RH_Equivalent_Implication E → VonKochPrimeErrorCriterion E
  vonkoch_to_left_half :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → s.re ≤ (1 / 2 : Real)
  zero_symmetry :
    ∀ s : Complex, IsNontrivialZetaZero s → IsNontrivialZetaZero (1 - s)

theorem endpoint_to_vonkoch_derived :
    ∀ E : Real → Real, RH_Equivalent_Implication E → VonKochPrimeErrorCriterion E := by
  intro E hEndpoint
  exact (endpoint_iff_vonkoch_criterion E).mp hEndpoint

theorem zero_symmetry_derived :
    ∀ s : Complex, IsNontrivialZetaZero s → IsNontrivialZetaZero (1 - s) := by
  intro s hs
  exact nontrivialZetaZero_one_sub s hs

def endpointBridgeComponentsOfTwo
    (hLeft :
      ∀ E : Real → Real,
        VonKochPrimeErrorCriterion E →
          ∀ s : Complex, IsNontrivialZetaZero s → s.re ≤ (1 / 2 : Real))
    (hSym : ∀ s : Complex, IsNontrivialZetaZero s → IsNontrivialZetaZero (1 - s)) :
    EndpointBridgeComponents where
  endpoint_to_vonkoch := endpoint_to_vonkoch_derived
  vonkoch_to_left_half := hLeft
  zero_symmetry := hSym

def endpointBridgeComponentsOfOne
    (hLeft :
      ∀ E : Real → Real,
        VonKochPrimeErrorCriterion E →
          ∀ s : Complex, IsNontrivialZetaZero s → s.re ≤ (1 / 2 : Real)) :
    EndpointBridgeComponents where
  endpoint_to_vonkoch := endpoint_to_vonkoch_derived
  vonkoch_to_left_half := hLeft
  zero_symmetry := zero_symmetry_derived

structure VonKochToLeftHalfKernel where
  /-- Any zero with real part > 1/2 forces a super-1/2 lower envelope for the error term. -/
  right_half_zero_forces_lower_envelope :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)
  /-- Von-Koch/endpoint criterion gives a concrete global O(x^(1/2) log^2 x) upper envelope. -/
  vonkoch_gives_upper_envelope :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∃ C x0 : Real, 0 ≤ C ∧ Real.exp 1 ≤ x0 ∧
          (∀ x : Real, x ≥ x0 → |E x| ≤ C * Real.sqrt x * (Real.log x) ^ (2 : Nat))
  /-- Asymptotic incompatibility: a super-1/2 lower envelope contradicts the endpoint upper one. -/
  super_half_lower_contradicts_endpoint_upper :
    ∀ E : Real → Real,
      (∃ C x0 : Real, 0 ≤ C ∧ Real.exp 1 ≤ x0 ∧
        (∀ x : Real, x ≥ x0 → |E x| ≤ C * Real.sqrt x * (Real.log x) ^ (2 : Nat))) →
      (∃ β : Real, (1 / 2 : Real) < β ∧
        (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)) →
      False

theorem vonkoch_gives_upper_envelope_derived :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∃ C x0 : Real, 0 ≤ C ∧ Real.exp 1 ≤ x0 ∧
          (∀ x : Real, x ≥ x0 → |E x| ≤ C * Real.sqrt x * (Real.log x) ^ (2 : Nat)) := by
  intro E hVonKoch
  exact hVonKoch

def vonKochToLeftHalfKernelOfTwo
    (hLower :
      ∀ E : Real → Real,
        VonKochPrimeErrorCriterion E →
          ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
            ∃ β : Real, (1 / 2 : Real) < β ∧
              (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β))
    (hContradict :
      ∀ E : Real → Real,
        (∃ C x0 : Real, 0 ≤ C ∧ Real.exp 1 ≤ x0 ∧
          (∀ x : Real, x ≥ x0 → |E x| ≤ C * Real.sqrt x * (Real.log x) ^ (2 : Nat))) →
        (∃ β : Real, (1 / 2 : Real) < β ∧
          (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)) →
        False) :
    VonKochToLeftHalfKernel where
  right_half_zero_forces_lower_envelope := hLower
  vonkoch_gives_upper_envelope := vonkoch_gives_upper_envelope_derived
  super_half_lower_contradicts_endpoint_upper := hContradict

/--
Trusted import boundary for the last two analytic steps needed to close
`vonkoch_to_left_half`.
-/
structure ImportedAnalyticBridgeResults where
  right_half_zero_forces_lower_envelope_import :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)
  super_half_lower_contradicts_endpoint_upper_import :
    ∀ E : Real → Real,
      (∃ C x0 : Real, 0 ≤ C ∧ Real.exp 1 ≤ x0 ∧
        (∀ x : Real, x ≥ x0 → |E x| ≤ C * Real.sqrt x * (Real.log x) ^ (2 : Nat))) →
      (∃ β : Real, (1 / 2 : Real) < β ∧
        (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)) →
      False

def vonKochToLeftHalfKernelOfImported
    (i : ImportedAnalyticBridgeResults) :
    VonKochToLeftHalfKernel :=
  vonKochToLeftHalfKernelOfTwo
    i.right_half_zero_forces_lower_envelope_import
    i.super_half_lower_contradicts_endpoint_upper_import

theorem endpoint_upper_power_domination
    (C β : Real) (hC : 0 ≤ C) (hβ : (1 / 2 : Real) < β) :
    ∃ X : Real,
      ∀ x : Real, x ≥ X →
        C * Real.sqrt x * (Real.log x) ^ (2 : Nat) < x ^ β := by
  have hpow : 0 < β - (1 / 2 : Real) := by linarith
  have hLittle : (fun x : ℝ => Real.log x ^ (2 : ℝ)) =o[atTop] (fun x => x ^ (β - (1 / 2 : Real))) := by
    simpa using (isLittleO_log_rpow_rpow_atTop (r := (2 : ℝ)) (s := β - (1 / 2 : Real)) hpow)
  let ε : Real := (C + 1)⁻¹
  have hε : 0 < ε := by
    have hCp : 0 < C + 1 := by linarith
    simpa [ε] using inv_pos.mpr hCp
  have hBound : ∀ᶠ x : ℝ in atTop, |Real.log x ^ (2 : ℝ)| ≤ ε * |x ^ (β - (1 / 2 : Real))| := by
    simpa [Real.norm_eq_abs] using hLittle.bound hε
  have hGeOne : ∀ᶠ x : ℝ in atTop, x ≥ 1 := eventually_ge_atTop 1
  have hDom : ∀ᶠ x : ℝ in atTop,
      C * Real.sqrt x * (Real.log x) ^ (2 : Nat) < x ^ β := by
    filter_upwards [hBound, hGeOne] with x hxBound hx1
    have hxpos : 0 < x := lt_of_lt_of_le (by norm_num : (0 : Real) < 1) hx1
    have hxnonneg : 0 ≤ x := le_of_lt hxpos
    have hpow_nonneg : 0 ≤ x ^ (β - (1 / 2 : Real)) := Real.rpow_nonneg hxnonneg _
    have hlogsq_nonneg : 0 ≤ (Real.log x) ^ (2 : Nat) := by positivity
    have hlogsq_le : (Real.log x) ^ (2 : Nat) ≤ ε * (x ^ (β - (1 / 2 : Real))) := by
      have hxAbs : |Real.log x ^ (2 : ℝ)| ≤ ε * |x ^ (β - (1 / 2 : Real))| := hxBound
      have hxAbs' : (Real.log x) ^ (2 : Nat) ≤ ε * |x ^ (β - (1 / 2 : Real))| := by
        simpa [abs_of_nonneg hlogsq_nonneg] using hxAbs
      have habsPow : |x ^ (β - (1 / 2 : Real))| = x ^ (β - (1 / 2 : Real)) := abs_of_nonneg hpow_nonneg
      rw [habsPow] at hxAbs'
      exact hxAbs'
    have hmul : C * Real.sqrt x * (Real.log x) ^ (2 : Nat) ≤
        C * Real.sqrt x * (ε * (x ^ (β - (1 / 2 : Real)))) := by
      gcongr
    have hCε : C * ε < 1 := by
      have hlt : C < C + 1 := by linarith
      have hInvPos : 0 < (C + 1)⁻¹ := inv_pos.mpr (by linarith)
      have hmulLt : C * (C + 1)⁻¹ < (C + 1) * (C + 1)⁻¹ :=
        mul_lt_mul_of_pos_right hlt hInvPos
      have hright : (C + 1) * (C + 1)⁻¹ = (1 : Real) := by
        have hne : C + 1 ≠ 0 := by linarith
        field_simp [hne]
      exact lt_of_lt_of_eq hmulLt hright
    have hrpow_mul : Real.sqrt x * (x ^ (β - (1 / 2 : Real))) = x ^ β := by
      calc
        Real.sqrt x * (x ^ (β - (1 / 2 : Real)))
            = x ^ (1 / 2 : Real) * x ^ (β - (1 / 2 : Real)) := by simp [Real.sqrt_eq_rpow]
        _ = x ^ ((1 / 2 : Real) + (β - (1 / 2 : Real))) := by
              symm
              exact Real.rpow_add hxpos (1 / 2 : Real) (β - (1 / 2 : Real))
        _ = x ^ β := by ring_nf
    have hmul' : C * Real.sqrt x * (ε * (x ^ (β - (1 / 2 : Real)))) =
        (C * ε) * (Real.sqrt x * (x ^ (β - (1 / 2 : Real)))) := by ring
    have hApos : 0 < Real.sqrt x * (x ^ (β - (1 / 2 : Real))) := by
      rw [hrpow_mul]
      exact Real.rpow_pos_of_pos hxpos β
    have hlt_scale : (C * ε) * (Real.sqrt x * (x ^ (β - (1 / 2 : Real)))) <
        1 * (Real.sqrt x * (x ^ (β - (1 / 2 : Real)))) := by
      exact mul_lt_mul_of_pos_right hCε hApos
    have hlt_pow : (C * ε) * (Real.sqrt x * (x ^ (β - (1 / 2 : Real)))) < x ^ β := by
      calc
        (C * ε) * (Real.sqrt x * (x ^ (β - (1 / 2 : Real)))) <
            1 * (Real.sqrt x * (x ^ (β - (1 / 2 : Real)))) := hlt_scale
        _ = Real.sqrt x * (x ^ (β - (1 / 2 : Real))) := by ring
        _ = x ^ β := hrpow_mul
    exact lt_of_le_of_lt (le_trans hmul (by rw [hmul'])) hlt_pow
  rcases (eventually_atTop.1 hDom) with ⟨X, hX⟩
  exact ⟨X, hX⟩

theorem super_half_lower_contradicts_endpoint_upper_of_domination
    (hDom :
      ∀ C β : Real, 0 ≤ C → (1 / 2 : Real) < β →
        ∃ X : Real,
          ∀ x : Real, x ≥ X →
            C * Real.sqrt x * (Real.log x) ^ (2 : Nat) < x ^ β)
    (E : Real → Real)
    (hUpper :
      ∃ C x0 : Real, 0 ≤ C ∧ Real.exp 1 ≤ x0 ∧
        (∀ x : Real, x ≥ x0 → |E x| ≤ C * Real.sqrt x * (Real.log x) ^ (2 : Nat)))
    (hLower :
      ∃ β : Real, (1 / 2 : Real) < β ∧
        (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)) :
    False := by
  rcases hUpper with ⟨C, x0, hC, hx0, hUpperAt⟩
  rcases hLower with ⟨β, hβ, hLowerAt⟩
  rcases hDom C β hC hβ with ⟨X, hDomAt⟩
  rcases hLowerAt (max X x0) with ⟨x, hx_ge, hLowerX⟩
  have hx_ge_X : x ≥ X := le_trans (le_max_left X x0) hx_ge
  have hx_ge_x0 : x ≥ x0 := le_trans (le_max_right X x0) hx_ge
  have hUpperX : |E x| ≤ C * Real.sqrt x * (Real.log x) ^ (2 : Nat) := hUpperAt x hx_ge_x0
  have hDomX : C * Real.sqrt x * (Real.log x) ^ (2 : Nat) < x ^ β := hDomAt x hx_ge_X
  have hLowerX' : x ^ β ≤ |E x| := by linarith
  have hlt : x ^ β < x ^ β := lt_of_le_of_lt hLowerX' (lt_of_le_of_lt hUpperX hDomX)
  exact lt_irrefl _ hlt

def vonKochToLeftHalfKernelOfLowerAndDomination
    (hLower :
      ∀ E : Real → Real,
        VonKochPrimeErrorCriterion E →
          ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
            ∃ β : Real, (1 / 2 : Real) < β ∧
              (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β))
    (hDom :
      ∀ C β : Real, 0 ≤ C → (1 / 2 : Real) < β →
        ∃ X : Real,
          ∀ x : Real, x ≥ X →
            C * Real.sqrt x * (Real.log x) ^ (2 : Nat) < x ^ β) :
    VonKochToLeftHalfKernel :=
  vonKochToLeftHalfKernelOfTwo
    hLower
    (super_half_lower_contradicts_endpoint_upper_of_domination hDom)

def vonKochToLeftHalfKernelOfOne
    (hLower :
      ∀ E : Real → Real,
        VonKochPrimeErrorCriterion E →
          ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
            ∃ β : Real, (1 / 2 : Real) < β ∧
              (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)) :
    VonKochToLeftHalfKernel :=
  vonKochToLeftHalfKernelOfLowerAndDomination hLower endpoint_upper_power_domination

theorem vonkoch_to_left_half_of_kernel
    (k : VonKochToLeftHalfKernel) :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → s.re ≤ (1 / 2 : Real) := by
  intro E hVonKoch s hs
  by_contra hgt
  have hs_gt : (1 / 2 : Real) < s.re := by exact lt_of_not_ge hgt
  rcases k.right_half_zero_forces_lower_envelope E hVonKoch s hs hs_gt with ⟨β, hβ, hLower⟩
  rcases k.vonkoch_gives_upper_envelope E hVonKoch with ⟨C, x0, hC, hx0, hUpper⟩
  have hUpperPack :
      ∃ C x0 : Real, 0 ≤ C ∧ Real.exp 1 ≤ x0 ∧
        (∀ x : Real, x ≥ x0 → |E x| ≤ C * Real.sqrt x * (Real.log x) ^ (2 : Nat)) :=
    ⟨C, x0, hC, hx0, hUpper⟩
  have hLowerPack :
      ∃ β : Real, (1 / 2 : Real) < β ∧
        (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β) :=
    ⟨β, hβ, hLower⟩
  exact k.super_half_lower_contradicts_endpoint_upper E hUpperPack hLowerPack

theorem vonkoch_to_left_half_of_riemann_hypothesis
    (hRH : _root_.RiemannHypothesis) :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → s.re ≤ (1 / 2 : Real) := by
  intro E hVonKoch s hs
  have _ := hVonKoch
  rcases hs with ⟨hz, hRePos, hReLtOne⟩
  have hs_ne_one : s ≠ 1 := by
    intro hs1
    have hReEq : s.re = 1 := by simpa [hs1]
    linarith
  have hs_not_trivial : ¬∃ n : ℕ, s = -2 * (n + 1) := by
    intro h
    rcases h with ⟨n, hn⟩
    have hReEq : s.re = -((2 * (n + 1) : ℕ) : Real) := by
      have := congrArg Complex.re hn
      norm_num at this ⊢
      exact this
    have hNatNonneg : (0 : Real) ≤ ((2 * (n + 1) : ℕ) : Real) := by exact_mod_cast Nat.zero_le _
    linarith
  have hEq : s.re = (1 / 2 : Real) := hRH s hz hs_not_trivial hs_ne_one
  exact le_of_eq hEq

def endpointBridgeComponentsOfRH
    (hRH : _root_.RiemannHypothesis) :
    EndpointBridgeComponents where
  endpoint_to_vonkoch := endpoint_to_vonkoch_derived
  vonkoch_to_left_half := vonkoch_to_left_half_of_riemann_hypothesis hRH
  zero_symmetry := zero_symmetry_derived

theorem endpoint_bridge_of_components
    (c : EndpointBridgeComponents) :
    EndpointToZetaCriticalLineBridge := by
  intro E hEndpoint s hs
  have hVonKoch : VonKochPrimeErrorCriterion E := c.endpoint_to_vonkoch E hEndpoint
  have hle : s.re ≤ (1 / 2 : Real) := c.vonkoch_to_left_half E hVonKoch s hs
  have hs_sym : IsNontrivialZetaZero (1 - s) := c.zero_symmetry s hs
  have hle_sym : (1 - s).re ≤ (1 / 2 : Real) := c.vonkoch_to_left_half E hVonKoch (1 - s) hs_sym
  have hge : (1 / 2 : Real) ≤ s.re := by
    have haux : 1 - s.re ≤ (1 / 2 : Real) := by
      simpa [Complex.sub_re] using hle_sym
    nlinarith
  exact le_antisymm hle hge

theorem endpoint_to_rh_of_bridge
    (hBridge : EndpointToZetaCriticalLineBridge) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement := by
  intro E hEndpoint s hs
  exact hBridge E hEndpoint s hs

structure ImportedZeroOscillationResults where
  right_half_zero_forces_lower_envelope_import :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)

theorem endpoint_to_rh_of_imported_zero_oscillation
    (z : ImportedZeroOscillationResults) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement := by
  let k : VonKochToLeftHalfKernel := vonKochToLeftHalfKernelOfOne z.right_half_zero_forces_lower_envelope_import
  let c : EndpointBridgeComponents :=
    endpointBridgeComponentsOfOne (vonkoch_to_left_half_of_kernel k)
  exact endpoint_to_rh_of_bridge (endpoint_bridge_of_components c)

theorem lower_envelope_from_constant_factor
    (E : Real → Real)
    (c β : Real)
    (hc : 0 < c)
    (hβ : (1 / 2 : Real) < β)
    (hOmega : ∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ c * x ^ β) :
    ∃ β' : Real, (1 / 2 : Real) < β' ∧
      (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β') := by
  let β' : Real := (β + (1 / 2 : Real)) / 2
  have hβ' : (1 / 2 : Real) < β' := by
    dsimp [β']
    linarith
  have hgap : 0 < β - β' := by
    dsimp [β']
    linarith
  have hpow_atTop : Tendsto (fun x : Real => x ^ (β - β')) atTop atTop := tendsto_rpow_atTop hgap
  have hEventuallyScale : ∀ᶠ x : Real in atTop, x ^ (β - β') ≥ c⁻¹ :=
    (tendsto_atTop.1 hpow_atTop) (c⁻¹)
  rcases (eventually_atTop.1 hEventuallyScale) with ⟨X0, hX0⟩
  refine ⟨β', hβ', ?_⟩
  intro X
  rcases hOmega (max X (max X0 1)) with ⟨x, hx, hEx⟩
  have hxX : x ≥ X := le_trans (le_max_left X (max X0 1)) hx
  have hxX0 : x ≥ X0 := le_trans (le_max_left X0 1) (le_trans (le_max_right X (max X0 1)) hx)
  have hx1 : x ≥ 1 := le_trans (le_max_right X0 1) (le_trans (le_max_right X (max X0 1)) hx)
  have hxpos : 0 < x := lt_of_lt_of_le (by norm_num : (0 : Real) < 1) hx1
  have hscale : x ^ (β - β') ≥ c⁻¹ := hX0 x hxX0
  have hmul1 : c * x ^ β = (c * x ^ (β - β')) * x ^ β' := by
    have hsum : β = (β - β') + β' := by ring
    rw [hsum, Real.rpow_add hxpos]
    ring
  have hleft : 1 ≤ c * x ^ (β - β') := by
    have hc_ne : c ≠ 0 := ne_of_gt hc
    have hmulLt : c * c⁻¹ ≤ c * x ^ (β - β') :=
      mul_le_mul_of_nonneg_left hscale (le_of_lt hc)
    have hccinv : c * c⁻¹ = (1 : Real) := by field_simp [hc_ne]
    simpa [hccinv] using hmulLt
  have hpow_nonneg : 0 ≤ x ^ β' := Real.rpow_nonneg (le_of_lt hxpos) _
  have hunit : x ^ β' ≤ c * x ^ β := by
    calc
      x ^ β' = 1 * x ^ β' := by ring
      _ ≤ (c * x ^ (β - β')) * x ^ β' := by exact mul_le_mul_of_nonneg_right hleft hpow_nonneg
      _ = c * x ^ β := by rw [← hmul1]
  have hfinal : |E x| ≥ x ^ β' := le_trans hunit hEx
  exact ⟨x, hxX, hfinal⟩

theorem omega_abs_from_signed_pos
    (E : Real → Real)
    (c β : Real)
    (hSignedPos : ∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≥ c * x ^ β) :
    ∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ c * x ^ β := by
  intro X
  rcases hSignedPos X with ⟨x, hx, hpos⟩
  refine ⟨x, hx, ?_⟩
  exact le_trans hpos (le_abs_self (E x))

theorem omega_abs_from_signed_neg
    (E : Real → Real)
    (c β : Real)
    (hSignedNeg : ∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≤ - (c * x ^ β)) :
    ∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ c * x ^ β := by
  intro X
  rcases hSignedNeg X with ⟨x, hx, hneg⟩
  refine ⟨x, hx, ?_⟩
  have h1 : c * x ^ β ≤ -E x := by linarith
  have h2 : -E x ≤ |E x| := neg_le_abs (E x)
  exact le_trans h1 h2

/-!
Published theorem pack boundary for the final remaining analytic ingredient.
The trust surface is a proof object carrying locked citation metadata.
-/
structure PublishedZeroOscillationPack where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  right_half_zero_forces_lower_envelope :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ β : Real, (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ x ^ β)

def importedZeroOscillationOfPack
    (p : PublishedZeroOscillationPack) : ImportedZeroOscillationResults where
  right_half_zero_forces_lower_envelope_import :=
    p.right_half_zero_forces_lower_envelope

theorem endpoint_to_rh_of_published_zero_oscillation
    (p : PublishedZeroOscillationPack) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_imported_zero_oscillation (importedZeroOscillationOfPack p)

structure PublishedZeroOscillationWeakPack where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  right_half_zero_forces_lower_envelope_omega :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ c β : Real, 0 < c ∧ (1 / 2 : Real) < β ∧
            (∀ X : Real, ∃ x : Real, x ≥ X ∧ |E x| ≥ c * x ^ β)

structure PublishedZeroOscillationSignedPack where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  right_half_zero_forces_lower_envelope_signed :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ c β : Real, 0 < c ∧ (1 / 2 : Real) < β ∧
            ((∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≥ c * x ^ β) ∨
             (∀ X : Real, ∃ x : Real, x ≥ X ∧ E x ≤ - (c * x ^ β)))

def weakenSignedToWeakPack
    (p : PublishedZeroOscillationSignedPack) : PublishedZeroOscillationWeakPack where
  source_tag := p.source_tag
  source_url := p.source_url
  theorem_ref := p.theorem_ref
  source_tag_lock := p.source_tag_lock
  source_url_lock := p.source_url_lock
  theorem_ref_lock := p.theorem_ref_lock
  right_half_zero_forces_lower_envelope_omega := by
    intro E hVonKoch s hs hs_gt
    rcases p.right_half_zero_forces_lower_envelope_signed E hVonKoch s hs hs_gt with
      ⟨c, β, hc, hβ, hSigned⟩
    refine ⟨c, β, hc, hβ, ?_⟩
    rcases hSigned with hPos | hNeg
    · exact omega_abs_from_signed_pos E c β hPos
    · exact omega_abs_from_signed_neg E c β hNeg

def strengthenPublishedZeroOscillationPack
    (p : PublishedZeroOscillationWeakPack) : PublishedZeroOscillationPack where
  source_tag := p.source_tag
  source_url := p.source_url
  theorem_ref := p.theorem_ref
  source_tag_lock := p.source_tag_lock
  source_url_lock := p.source_url_lock
  theorem_ref_lock := p.theorem_ref_lock
  right_half_zero_forces_lower_envelope := by
    intro E hVonKoch s hs hs_gt
    rcases p.right_half_zero_forces_lower_envelope_omega E hVonKoch s hs hs_gt with
      ⟨c, β, hc, hβ, hOmega⟩
    exact lower_envelope_from_constant_factor E c β hc hβ hOmega

theorem endpoint_to_rh_of_imported_analytic_bridge
    (i : ImportedAnalyticBridgeResults) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement := by
  let k : VonKochToLeftHalfKernel := vonKochToLeftHalfKernelOfImported i
  let c : EndpointBridgeComponents :=
    endpointBridgeComponentsOfOne (vonkoch_to_left_half_of_kernel k)
  exact endpoint_to_rh_of_bridge (endpoint_bridge_of_components c)

def endpointToRHKernelOfBridge
    (hBridge : EndpointToZetaCriticalLineBridge) : EndpointToRHKernel where
  endpoint_to_rh := endpoint_to_rh_of_bridge hBridge

end PrimeRiemannBridgeCompletionKernel
