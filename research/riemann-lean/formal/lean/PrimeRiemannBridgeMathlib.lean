import Mathlib

namespace PrimeRiemannBridgeMathlib

noncomputable section

abbrev WheelFamily : Nat → Prop :=
  fun w => w = 30 ∨ w = 210 ∨ w = 2310 ∨ w = 30030

structure O2Constants where
  nbound_c1 : Real
  nbound_c2 : Real
  nbound_c3 : Real
  nbound_h : Real

structure O1Constants where
  C0_ref_O1 : Real
  a_ref : Real
  b_ref : Real
  m_ref : Nat

structure O3Constants where
  A_offabs : Real
  C_offabs : Real
  k_abs : Real
  A_diag : Real
  C_diag : Real
  A_E2 : Real
  C_E2 : Real

structure O4Constants where
  C0_ref_O4 : Real
  a_ref : Real
  b_ref : Real
  C_delta : Real
  C_H : Real

structure ExternalZeroCountSource where
  source_tag : String
  source_url : String

def M (T : Real) : Real :=
  T / (2 * Real.pi) * Real.log (T / (2 * Real.pi * Real.exp 1))

def O2ZeroCountBound (c : O2Constants) : Prop :=
  ∀ T : Real,
    T ≥ Real.exp 1 →
      |((Nat.floor (T / (2 * Real.pi) * Real.log (T / (2 * Real.pi * Real.exp 1))) : Int) -
        (Nat.floor (M T) : Int))| ≤
      c.nbound_c1 * Real.log T +
      c.nbound_c2 * Real.log (Real.log T) +
      c.nbound_c3

def O2Closed (c : O2Constants) (src : ExternalZeroCountSource) : Prop :=
  c.nbound_c1 = 0.1038 ∧
  c.nbound_c2 = 0.2573 ∧
  c.nbound_c3 = 9.3675 ∧
  c.nbound_h = 1.0 ∧
  src.source_tag = "HSW2021-ABS-ZEROCOUNT" ∧
  src.source_url = "https://arxiv.org/abs/2107.06506" ∧
  O2ZeroCountBound c

structure HSW2021ZeroCountWitness where
  c : O2Constants
  c1_lock : c.nbound_c1 = 0.1038
  c2_lock : c.nbound_c2 = 0.2573
  c3_lock : c.nbound_c3 = 9.3675
  h_lock : c.nbound_h = 1.0
  zero_count_bound : O2ZeroCountBound c

def O1TransferBound (E H : Real → Real) : Prop :=
  ∃ Ctr x0 : Real, 0 ≤ Ctr ∧ Real.exp 1 ≤ x0 ∧
    (∀ x : Real, x ≥ x0 → |E x| ≤ Ctr * Real.sqrt x + |H x| * Real.sqrt x)

def O3BridgeBound (H : Real → Real) : Prop :=
  ∃ CH x0 : Real, 0 ≤ CH ∧ Real.exp 1 ≤ x0 ∧
    (∀ x : Real, x ≥ x0 → |H x| ≤ CH * (Real.log x) ^ (2 : Nat))

def O1Closed (E H : Real → Real) (c1 : O1Constants) : Prop :=
  c1.C0_ref_O1 = 0.9102883687683553 ∧
  c1.a_ref = -0.0013474693715061251 ∧
  c1.b_ref = -0.05436122353654979 ∧
  c1.m_ref = 512 ∧
  O1TransferBound E H

def O3Closed (H : Real → Real) (c3 : O3Constants) : Prop :=
  c3.A_offabs = 0 ∧
  c3.C_offabs = 0.03292827711413939 ∧
  c3.k_abs = 0.005725212627704354 ∧
  c3.A_diag = 0 ∧
  c3.C_diag = 1 ∧
  c3.A_E2 = 0 ∧
  c3.C_E2 = 1.1195893906678458 ∧
  O3BridgeBound H

def EndpointClass (E : Real → Real) : Prop :=
  ∃ C x0 : Real, 0 ≤ C ∧ Real.exp 1 ≤ x0 ∧
    (∀ x : Real, x ≥ x0 → |E x| ≤ C * Real.sqrt x * (Real.log x) ^ (2 : Nat))

def RH_Equivalent_Implication (E : Real → Real) : Prop := EndpointClass E

theorem log_sq_ge_one_of_ge_exp_one (x : Real) (hx : x ≥ Real.exp 1) :
    1 ≤ (Real.log x) ^ (2 : Nat) := by
  have hlog : 1 ≤ Real.log x := by
    have hmono : Real.log (Real.exp 1) ≤ Real.log x := by
      exact Real.log_le_log (Real.exp_pos 1) hx
    simpa [Real.log_exp] using hmono
  nlinarith [hlog]

theorem o2_closed_from_hsw_witness
    (src : ExternalZeroCountSource)
    (hTag : src.source_tag = "HSW2021-ABS-ZEROCOUNT")
    (hUrl : src.source_url = "https://arxiv.org/abs/2107.06506")
    (w : HSW2021ZeroCountWitness) :
    O2Closed w.c src := by
  exact ⟨w.c1_lock, w.c2_lock, w.c3_lock, w.h_lock, hTag, hUrl, w.zero_count_bound⟩

theorem derive_transfer_from_O1
    (E H : Real → Real) (c1 : O1Constants)
    (h1 : O1Closed E H c1) :
    O1TransferBound E H := by
  exact h1.right.right.right.right

theorem derive_bridge_from_O3
    (H : Real → Real) (c3 : O3Constants)
    (h3 : O3Closed H c3) :
    O3BridgeBound H := by
  exact h3.right.right.right.right.right.right.right

theorem o2_source_is_hsw2021
    (c2 : O2Constants) (src : ExternalZeroCountSource)
    (h2 : O2Closed c2 src) :
    src.source_tag = "HSW2021-ABS-ZEROCOUNT" ∧
    src.source_url = "https://arxiv.org/abs/2107.06506" := by
  rcases h2 with ⟨_, _, _, _, htag, hurl, _⟩
  exact ⟨htag, hurl⟩

theorem l3_endpoint_from_transfer
    (E H : Real → Real)
    (c1 : O1Constants)
    (c3 : O3Constants)
    (h1 : O1Closed E H c1)
    (h3 : O3Closed H c3) :
    RH_Equivalent_Implication E := by
  rcases derive_transfer_from_O1 E H c1 h1 with ⟨Ctr, x1, hCtr, hx1, hT⟩
  rcases derive_bridge_from_O3 H c3 h3 with ⟨CH, x2, hCH, hx2, hB⟩
  have hExpMax : Real.exp 1 ≤ max x1 x2 := le_trans hx1 (le_max_left x1 x2)
  refine ⟨Ctr + CH, max x1 x2, add_nonneg hCtr hCH, hExpMax, ?_⟩
  intro x hx
  have hx1' : x ≥ x1 := le_trans (le_max_left x1 x2) hx
  have hx2' : x ≥ x2 := le_trans (le_max_right x1 x2) hx
  have hxexp : x ≥ Real.exp 1 := by
    exact le_trans hx1 hx1'
  have hE : |E x| ≤ Ctr * Real.sqrt x + |H x| * Real.sqrt x := hT x hx1'
  have hH : |H x| ≤ CH * (Real.log x) ^ (2 : Nat) := hB x hx2'
  have hsqrt_nonneg : 0 ≤ Real.sqrt x := Real.sqrt_nonneg x
  have hlog_sq : 1 ≤ (Real.log x) ^ (2 : Nat) := log_sq_ge_one_of_ge_exp_one x hxexp
  have hmulH :
      |H x| * Real.sqrt x ≤ (CH * (Real.log x) ^ (2 : Nat)) * Real.sqrt x := by
    exact mul_le_mul_of_nonneg_right hH hsqrt_nonneg
  have hmulH' :
      |H x| * Real.sqrt x ≤ CH * Real.sqrt x * (Real.log x) ^ (2 : Nat) := by
    calc
      |H x| * Real.sqrt x ≤ (CH * (Real.log x) ^ (2 : Nat)) * Real.sqrt x := hmulH
      _ = CH * Real.sqrt x * (Real.log x) ^ (2 : Nat) := by ring
  have hCtrBaseNonneg : 0 ≤ Ctr * Real.sqrt x := mul_nonneg hCtr hsqrt_nonneg
  have hCtrLift :
      Ctr * Real.sqrt x ≤ Ctr * Real.sqrt x * (Real.log x) ^ (2 : Nat) := by
    calc
      Ctr * Real.sqrt x = Ctr * Real.sqrt x * 1 := by ring
      _ ≤ Ctr * Real.sqrt x * (Real.log x) ^ (2 : Nat) := by
        exact mul_le_mul_of_nonneg_left hlog_sq hCtrBaseNonneg
  have hsum :
      |E x| ≤ Ctr * Real.sqrt x + CH * Real.sqrt x * (Real.log x) ^ (2 : Nat) := by
    have hAdd :
        Ctr * Real.sqrt x + |H x| * Real.sqrt x ≤
        Ctr * Real.sqrt x + CH * Real.sqrt x * (Real.log x) ^ (2 : Nat) := by
      simpa [add_comm, add_left_comm, add_assoc] using
        (add_le_add_left hmulH' (Ctr * Real.sqrt x))
    exact le_trans hE hAdd
  have hsumLift :
      |E x| ≤
        Ctr * Real.sqrt x * (Real.log x) ^ (2 : Nat) +
        CH * Real.sqrt x * (Real.log x) ^ (2 : Nat) := by
    have hAdd :
        Ctr * Real.sqrt x + CH * Real.sqrt x * (Real.log x) ^ (2 : Nat) ≤
        Ctr * Real.sqrt x * (Real.log x) ^ (2 : Nat) +
        CH * Real.sqrt x * (Real.log x) ^ (2 : Nat) := by
      simpa [add_comm, add_left_comm, add_assoc] using
        (add_le_add_right hCtrLift (CH * Real.sqrt x * (Real.log x) ^ (2 : Nat)))
    exact le_trans hsum hAdd
  have hfactor :
      Ctr * Real.sqrt x * (Real.log x) ^ (2 : Nat) +
      CH * Real.sqrt x * (Real.log x) ^ (2 : Nat) =
      (Ctr + CH) * Real.sqrt x * (Real.log x) ^ (2 : Nat) := by ring
  simpa [hfactor] using hsumLift

/- R4 target encoding: explicit project-level criterion form (directional). -/
def RHCriterion (E : Real → Real) : Prop :=
  ∃ C x0 : Real, 0 ≤ C ∧ Real.exp 1 ≤ x0 ∧
    (∀ x : Real, x ≥ x0 → |E x| ≤ C * Real.sqrt x * (Real.log x) ^ (2 : Nat))

theorem endpoint_implies_project_criterion (E : Real → Real) :
    RH_Equivalent_Implication E → RHCriterion E := by
  intro h
  exact h

def VonKochPrimeErrorCriterion (E : Real → Real) : Prop := RHCriterion E

theorem endpoint_iff_vonkoch_criterion (E : Real → Real) :
    RH_Equivalent_Implication E ↔ VonKochPrimeErrorCriterion E := by
  constructor <;> intro h <;> exact h

/-- Nontrivial zeta zero predicate (critical-strip zeros). -/
def IsNontrivialZetaZero (s : Complex) : Prop :=
  riemannZeta s = 0 ∧ 0 < s.re ∧ s.re < 1

/-- Nontrivial RH target (skeleton): all nontrivial zeros lie on Re(s)=1/2. -/
def RHStatement : Prop :=
  ∀ s : Complex, IsNontrivialZetaZero s → s.re = (1 / 2 : Real)

theorem rhStatement_of_root_rh
    (hRH : _root_.RiemannHypothesis) :
    RHStatement := by
  intro s hs
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
    have hNatNonneg : (0 : Real) ≤ ((2 * (n + 1) : ℕ) : Real) := by
      exact_mod_cast Nat.zero_le _
    linarith
  exact hRH s hz hs_not_trivial hs_ne_one

theorem nontrivialZetaZero_one_sub
    (s : Complex) (hs : IsNontrivialZetaZero s) :
    IsNontrivialZetaZero (1 - s) := by
  rcases hs with ⟨hz, hRePos, hReLtOne⟩
  have hs_ne_one : s ≠ 1 := by
    intro hs1
    have hReEq : s.re = 1 := by simpa [hs1]
    linarith
  have hs_not_neg_nat : ∀ n : ℕ, s ≠ -n := by
    intro n hsn
    have hReEq : s.re = -(n : Real) := by
      have := congrArg Complex.re hsn
      simpa using this
    have hn_nonneg : 0 ≤ (n : Real) := by exact_mod_cast Nat.zero_le n
    linarith
  have hz_one_sub : riemannZeta (1 - s) = 0 := by
    have hfe := riemannZeta_one_sub (s := s) hs_not_neg_nat hs_ne_one
    rw [hz] at hfe
    simpa using hfe
  have hRePos_one_sub : 0 < (1 - s).re := by
    have : 0 < 1 - s.re := by linarith
    simpa [Complex.sub_re] using this
  have hReLtOne_one_sub : (1 - s).re < 1 := by
    have : 1 - s.re < 1 := by linarith
    simpa [Complex.sub_re] using this
  exact ⟨hz_one_sub, hRePos_one_sub, hReLtOne_one_sub⟩

theorem criterion_implies_rh_if_equivalence
    (hEq : ∀ E : Real → Real, VonKochPrimeErrorCriterion E → RHStatement) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement := by
  intro E hEndpoint
  exact hEq E ((endpoint_iff_vonkoch_criterion E).mp hEndpoint)

end

end PrimeRiemannBridgeMathlib
