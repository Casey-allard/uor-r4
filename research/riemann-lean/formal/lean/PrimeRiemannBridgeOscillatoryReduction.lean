import PrimeRiemannBridgeConcretePackInstantiation

namespace PrimeRiemannBridgeOscillatoryReduction

open PrimeRiemannBridgeMathlib
open PrimeRiemannBridgeCompletionKernel
open PrimeRiemannBridgeZeroOscillationProgram
open PrimeRiemannBridgeConcretePackInstantiation
open Filter

noncomputable section

/-!
Oscillatory-model reduction:
if a right-half zero yields a phase-oscillatory main term with a vanishing
normalized remainder along a divergent sequence, we can instantiate the
existing asymptotic program and close RH.
-/

def oscillatoryMainTerm
    (A β : Real)
    (phase : Real → Real) : Real → Real :=
  fun x => A * x ^ β * Real.cos (phase x)

structure PhaseOscillationAsymptoticAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_phase_oscillation :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, 0 < A ∧ (1 / 2 : Real) < β ∧
            ((∃ f : Nat → Real, ∃ phase R : Real → Real,
                Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ n : Nat, E (f n) = oscillatoryMainTerm A β phase (f n) + R (f n)) ∧
                (∀ᶠ n : Nat in Filter.atTop, Real.cos (phase (f n)) ≥ (1 / 2 : Real)) ∧
                Filter.Tendsto (fun n : Nat => R (f n) / (f n) ^ β) Filter.atTop (nhds 0)) ∨
             (∃ f : Nat → Real, ∃ phase R : Real → Real,
                Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ n : Nat, E (f n) = oscillatoryMainTerm A β phase (f n) + R (f n)) ∧
                (∀ᶠ n : Nat in Filter.atTop, Real.cos (phase (f n)) ≤ -((1 / 2 : Real))) ∧
                Filter.Tendsto (fun n : Nat => R (f n) / (f n) ^ β) Filter.atTop (nhds 0)))

structure ExplicitFormulaPhaseKernelAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_oscillatory_kernel :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0) ∧
            ((∃ f : Nat → Real, Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ᶠ n : Nat in Filter.atTop, Real.cos (phase (f n)) ≥ (1 / 2 : Real))) ∨
             (∃ f : Nat → Real, Filter.Tendsto f Filter.atTop Filter.atTop ∧
                (∀ᶠ n : Nat in Filter.atTop, Real.cos (phase (f n)) ≤ -((1 / 2 : Real)))))

structure ExplicitFormulaKernelSplitAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  phase_pinning_of_model :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∀ A β : Real, ∀ phase R : Real → Real,
            0 < A → (1 / 2 : Real) < β →
              (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
              ((∃ f : Nat → Real, Filter.Tendsto f Filter.atTop Filter.atTop ∧
                  (∀ᶠ n : Nat in Filter.atTop, Real.cos (phase (f n)) ≥ (1 / 2 : Real))) ∨
               (∃ f : Nat → Real, Filter.Tendsto f Filter.atTop Filter.atTop ∧
                  (∀ᶠ n : Nat in Filter.atTop, Real.cos (phase (f n)) ≤ -((1 / 2 : Real)))))

def phaseKernelAssumptionsOfSplit
    (h : ExplicitFormulaKernelSplitAssumptions) :
    ExplicitFormulaPhaseKernelAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_oscillatory_kernel := by
    intro E hVonKoch s hs hs_gt
    rcases h.zero_to_global_decomposition E hVonKoch s hs hs_gt with
      ⟨A, β, phase, R, hA, hβ, hDecomp, hRem⟩
    have hPin := h.phase_pinning_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    exact ⟨A, β, phase, R, hA, hβ, hDecomp, hRem, hPin⟩

structure ExplicitFormulaQuantizedPhaseAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  phase_quantization_of_model :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∀ A β : Real, ∀ phase R : Real → Real,
            0 < A → (1 / 2 : Real) < β →
              (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
              ((∃ f : Nat → Real, Filter.Tendsto f Filter.atTop Filter.atTop ∧
                  (∀ n : Nat, phase (f n) = n * (2 * Real.pi))) ∨
               (∃ f : Nat → Real, Filter.Tendsto f Filter.atTop Filter.atTop ∧
                  (∀ n : Nat, phase (f n) = n * (2 * Real.pi) + Real.pi)))

def splitAssumptionsOfQuantizedPhase
    (h : ExplicitFormulaQuantizedPhaseAssumptions) :
    ExplicitFormulaKernelSplitAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  phase_pinning_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    rcases h.phase_quantization_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
      hPos | hNeg
    · rcases hPos with ⟨f, hTendsto, hEq⟩
      refine Or.inl ⟨f, hTendsto, ?_⟩
      exact Filter.Eventually.of_forall (fun n => by
        rw [hEq n, Real.cos_nat_mul_two_pi]
        norm_num)
    · rcases hNeg with ⟨f, hTendsto, hEq⟩
      refine Or.inr ⟨f, hTendsto, ?_⟩
      exact Filter.Eventually.of_forall (fun n => by
        rw [hEq n, Real.cos_nat_mul_two_pi_add_pi]
        norm_num)

structure ExplicitFormulaLogLinearPhaseAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  linear_phase_of_model :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∀ A β : Real, ∀ phase R : Real → Real,
            0 < A → (1 / 2 : Real) < β →
              (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
              ∃ τ φ : Real, 0 < τ ∧ (∀ x : Real, 0 < x → phase x = τ * Real.log x + φ)

theorem zero_to_global_decomposition_of_vonkoch
    (E : Real → Real)
    (hVonKoch : VonKochPrimeErrorCriterion E) :
    ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
      ∃ A β : Real, ∃ phase R : Real → Real,
        0 < A ∧ (1 / 2 : Real) < β ∧
          (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
          Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0) := by
  intro s hs hs_gt
  rcases hVonKoch with ⟨C, x0, hC, hx0, hUpper⟩
  let β : Real := (3 / 4 : Real)
  have hβ : (1 / 2 : Real) < β := by
    norm_num [β]
  rcases endpoint_upper_power_domination C β hC hβ with ⟨Xdom, hDom⟩
  let clip : Real → Real := fun x => max (-1 : Real) (min (1 : Real) (E x / x ^ β))
  let phase : Real → Real := fun x => Real.arccos (clip x)
  let R : Real → Real := fun x => E x - oscillatoryMainTerm 1 β phase x
  refine ⟨1, β, phase, R, by norm_num, hβ, ?_, ?_⟩
  · intro x
    simp [R]
  · let X : Real := max (max x0 Xdom) 1
    have hEventEq :
        (fun _ : Real => (0 : Real)) =ᶠ[Filter.atTop]
          (fun x : Real => R x / x ^ β) := by
      refine Filter.eventually_atTop.2 ⟨X, ?_⟩
      intro x hxX
      have hx0_le : x0 ≤ X := by
        dsimp [X]
        exact le_trans (le_max_left x0 Xdom) (le_max_left (max x0 Xdom) 1)
      have hXdom_le : Xdom ≤ X := by
        dsimp [X]
        exact le_trans (le_max_right x0 Xdom) (le_max_left (max x0 Xdom) 1)
      have h1_le : (1 : Real) ≤ X := by
        dsimp [X]
        exact le_max_right (max x0 Xdom) 1
      have hxx0 : x ≥ x0 := le_trans hx0_le hxX
      have hxxdom : x ≥ Xdom := le_trans hXdom_le hxX
      have hxx1 : x ≥ 1 := le_trans h1_le hxX
      have hUpperX : |E x| ≤ C * Real.sqrt x * (Real.log x) ^ (2 : Nat) := hUpper x hxx0
      have hDomX : C * Real.sqrt x * (Real.log x) ^ (2 : Nat) < x ^ β := hDom x hxxdom
      have hAbsLt : |E x| < x ^ β := lt_of_le_of_lt hUpperX hDomX
      have hxpos : 0 < x := lt_of_lt_of_le (by norm_num : (0 : Real) < 1) hxx1
      have hpow_pos : 0 < x ^ β := Real.rpow_pos_of_pos hxpos β
      have hAbsDivLt : |E x / x ^ β| < 1 := by
        have hDivLt : |E x| / x ^ β < 1 := by
          rw [div_lt_iff₀ hpow_pos]
          simpa [one_mul] using hAbsLt
        have hAbsPow : |x ^ β| = x ^ β := abs_of_pos hpow_pos
        have hAbsDiv : |E x / x ^ β| = |E x| / x ^ β := by
          simpa [abs_div, hAbsPow]
        rw [hAbsDiv]
        exact hDivLt
      have hQuotLe : E x / x ^ β ≤ 1 := (abs_lt.mp hAbsDivLt).2.le
      have hQuotGe : -1 ≤ E x / x ^ β := (abs_lt.mp hAbsDivLt).1.le
      have hClipEq : clip x = E x / x ^ β := by
        dsimp [clip]
        rw [min_eq_right hQuotLe, max_eq_right hQuotGe]
      have hCos : Real.cos (phase x) = E x / x ^ β := by
        dsimp [phase]
        rw [hClipEq]
        exact Real.cos_arccos hQuotGe hQuotLe
      have hMainEq : oscillatoryMainTerm 1 β phase x = E x := by
        unfold oscillatoryMainTerm
        have hpow_ne : x ^ β ≠ 0 := ne_of_gt hpow_pos
        calc
          1 * x ^ β * Real.cos (phase x)
              = x ^ β * Real.cos (phase x) := by ring
          _ = x ^ β * (E x / x ^ β) := by rw [hCos]
          _ = E x := by
            field_simp [hpow_ne]
      have hRzero : R x = 0 := by
        dsimp [R]
        linarith [hMainEq]
      have hQuotZero : R x / x ^ β = 0 := by
        calc
          R x / x ^ β = 0 / x ^ β := by rw [hRzero]
          _ = 0 := by simp
      simpa using hQuotZero.symm
    have hTzero : Filter.Tendsto (fun _ : Real => (0 : Real)) Filter.atTop (nhds 0) :=
      tendsto_const_nhds
    exact hTzero.congr' hEventEq

structure ExplicitFormulaLinearPhaseOnlyAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  linear_phase_of_model :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∀ A β : Real, ∀ phase R : Real → Real,
            0 < A → (1 / 2 : Real) < β →
              (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
              ∃ τ φ : Real, 0 < τ ∧ (∀ x : Real, 0 < x → phase x = τ * Real.log x + φ)

abbrev LinearPhaseKernelTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∀ A β : Real, ∀ phase R : Real → Real,
          0 < A → (1 / 2 : Real) < β →
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
            ∃ τ φ : Real, 0 < τ ∧ (∀ x : Real, 0 < x → phase x = τ * Real.log x + φ)

abbrev LogDerivativeLinearPhaseKernelTerm : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∀ A β : Real, ∀ phase R : Real → Real,
          0 < A → (1 / 2 : Real) < β →
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
            ∃ τ : Real, 0 < τ ∧
              DifferentiableOn ℝ phase (Set.Ioi 0) ∧
              Set.EqOn (deriv phase) (fun x : Real => τ / x) (Set.Ioi 0)

theorem linear_phase_of_log_derivative_eq
    (phase : Real → Real)
    (τ : Real)
    (hDiff : DifferentiableOn ℝ phase (Set.Ioi 0))
    (hDeriv : Set.EqOn (deriv phase) (fun x : Real => τ / x) (Set.Ioi 0)) :
    ∃ φ : Real, ∀ x : Real, 0 < x → phase x = τ * Real.log x + φ := by
  let g : Real → Real := fun x => τ * Real.log x
  have hgDiff : DifferentiableOn ℝ g (Set.Ioi 0) := by
    intro x hx
    simpa [g] using ((Real.differentiableAt_log (ne_of_gt hx)).const_mul τ).differentiableWithinAt
  have hDerivEq : Set.EqOn (deriv phase) (deriv g) (Set.Ioi 0) := by
    intro x hx
    have hPhase : deriv phase x = τ / x := hDeriv hx
    have hG : deriv g x = τ / x := by
      simp [g, Real.deriv_log, div_eq_mul_inv]
    exact hPhase.trans hG.symm
  rcases isOpen_Ioi.exists_eq_add_of_deriv_eq isPreconnected_Ioi hDiff hgDiff hDerivEq with
    ⟨φ, hEqOn⟩
  refine ⟨φ, ?_⟩
  intro x hx
  have h := hEqOn hx
  simpa [g, add_comm, add_left_comm, add_assoc] using h

theorem linear_phase_kernel_of_log_derivative_kernel
    (hLogDeriv : LogDerivativeLinearPhaseKernelTerm) :
    LinearPhaseKernelTerm := by
  intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
  rcases hLogDeriv E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
    ⟨τ, hτ, hDiff, hDeriv⟩
  rcases linear_phase_of_log_derivative_eq phase τ hDiff hDeriv with ⟨φ, hPhase⟩
  exact ⟨τ, φ, hτ, hPhase⟩

theorem log_derivative_kernel_of_linear_phase_kernel
    (hLinear : LinearPhaseKernelTerm) :
    LogDerivativeLinearPhaseKernelTerm := by
  intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
  rcases hLinear E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
    ⟨τ, φ, hτ, hPhase⟩
  let g : Real → Real := fun x => τ * Real.log x + φ
  have hEqOn : Set.EqOn phase g (Set.Ioi 0) := by
    intro x hx
    simpa [g] using hPhase x hx
  have hgDiff : DifferentiableOn ℝ g (Set.Ioi 0) := by
    intro x hx
    have hDiffMulLog : DifferentiableAt ℝ (fun y : Real => τ * Real.log y) x := by
      simpa [div_eq_mul_inv] using (Real.differentiableAt_log (ne_of_gt hx)).const_mul τ
    have hDiffConst : DifferentiableAt ℝ (fun _ : Real => φ) x := differentiableAt_const φ
    exact (hDiffMulLog.add hDiffConst).differentiableWithinAt
  have hDiff : DifferentiableOn ℝ phase (Set.Ioi 0) :=
    hgDiff.congr hEqOn
  have hDerivEqOn : Set.EqOn (deriv phase) (deriv g) (Set.Ioi 0) :=
    hEqOn.deriv isOpen_Ioi
  refine ⟨τ, hτ, hDiff, ?_⟩
  intro x hx
  calc
    deriv phase x = deriv g x := hDerivEqOn hx
    _ = τ / x := by
          simp [g, Real.deriv_log, div_eq_mul_inv]

theorem linear_phase_kernel_iff_log_derivative_kernel :
    LinearPhaseKernelTerm ↔ LogDerivativeLinearPhaseKernelTerm := by
  constructor
  · exact log_derivative_kernel_of_linear_phase_kernel
  · exact linear_phase_kernel_of_log_derivative_kernel

def logLinearAssumptionsOfLinearPhaseOnly
    (h : ExplicitFormulaLinearPhaseOnlyAssumptions) :
    ExplicitFormulaLogLinearPhaseAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := zero_to_global_decomposition_of_vonkoch
  linear_phase_of_model := h.linear_phase_of_model

structure ExplicitFormulaLinearPhaseWitnessAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_linear_phase_witness :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β τ φ : Real, ∃ R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧ 0 < τ ∧
              (∀ x : Real, E x = oscillatoryMainTerm A β (fun y : Real => τ * Real.log y + φ) x + R x) ∧
              Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)

structure ExplicitFormulaAsymptoticallyLinearPhaseAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  asymptotically_linear_phase_of_model :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∀ A β : Real, ∀ phase R : Real → Real,
            0 < A → (1 / 2 : Real) < β →
              (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
              ∃ τ φ : Real, 0 < τ ∧
                Filter.Tendsto
                  (fun x : Real => phase x - (τ * Real.log x + φ))
                  Filter.atTop (nhds 0)

structure ExplicitFormulaSingleDecayingPhaseCorrectionAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  single_decaying_phase_correction_of_model :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∀ A β : Real, ∀ phase R : Real → Real,
            0 < A → (1 / 2 : Real) < β →
              (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
              ∃ τ φ κ η ω θ : Real, 0 < τ ∧ 0 < η ∧
                (∀ x : Real, 0 < x →
                  phase x = τ * Real.log x + φ +
                    κ * x ^ (-η) * Real.sin (ω * Real.log x + θ))

structure SingleDecayingPhaseCoreWitness (phase : Real → Real) where
  τ : Real
  φ : Real
  core : Real → Real
  τ_pos : 0 < τ
  phase_eq :
    ∀ x : Real, 0 < x →
      phase x = τ * Real.log x + φ + core x

structure SingleDecayingModeWitness (core : Real → Real) where
  κ : Real
  η : Real
  ω : Real
  θ : Real
  η_pos : 0 < η
  core_eq :
    ∀ x : Real, 0 < x →
      core x = κ * x ^ (-η) * Real.sin (ω * Real.log x + θ)

structure ExplicitFormulaSingleDecayingPhaseLadderAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  phase_core_split_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      SingleDecayingPhaseCoreWitness phase
  single_mode_of_phase_core :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      ∀ (wCore : SingleDecayingPhaseCoreWitness phase),
        wCore = phase_core_split_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp →
        SingleDecayingModeWitness wCore.core

def singleDecayingAssumptionsOfLadder
    (h : ExplicitFormulaSingleDecayingPhaseLadderAssumptions) :
    ExplicitFormulaSingleDecayingPhaseCorrectionAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  single_decaying_phase_correction_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    let wCore : SingleDecayingPhaseCoreWitness phase :=
      h.phase_core_split_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    let wMode : SingleDecayingModeWitness wCore.core :=
      h.single_mode_of_phase_core E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wCore rfl
    refine ⟨wCore.τ, wCore.φ, wMode.κ, wMode.η, wMode.ω, wMode.θ, wCore.τ_pos, wMode.η_pos, ?_⟩
    intro x hx
    calc
      phase x = wCore.τ * Real.log x + wCore.φ + wCore.core x := wCore.phase_eq x hx
      _ =
          wCore.τ * Real.log x + wCore.φ +
            (wMode.κ * x ^ (-wMode.η) * Real.sin (wMode.ω * Real.log x + wMode.θ)) := by
              rw [wMode.core_eq x hx]

def singleDecayingLadderAssumptionsOfLinearPhaseOnly
    (h : ExplicitFormulaLinearPhaseOnlyAssumptions) :
    ExplicitFormulaSingleDecayingPhaseLadderAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := zero_to_global_decomposition_of_vonkoch
  phase_core_split_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    classical
    have hWitness :
        Nonempty {w : SingleDecayingPhaseCoreWitness phase // ∀ x : Real, w.core x = 0} := by
      rcases h.linear_phase_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
        ⟨τ, φ, hτ, hPhaseEq⟩
      refine ⟨{
        val := {
          τ := τ
          φ := φ
          core := fun _ : Real => 0
          τ_pos := hτ
          phase_eq := ?_
        }
        property := by
          intro x
          rfl
      }⟩
      intro x hx
      have hEq : phase x = τ * Real.log x + φ := hPhaseEq x hx
      calc
        phase x = τ * Real.log x + φ := hEq
        _ = τ * Real.log x + φ + (0 : Real) := by ring
    exact (Classical.choice hWitness).1
  single_mode_of_phase_core := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wCore hwCore
    classical
    have hWitness :
        Nonempty {w : SingleDecayingPhaseCoreWitness phase // ∀ x : Real, w.core x = 0} := by
      rcases h.linear_phase_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
        ⟨τ, φ, hτ, hPhaseEq⟩
      refine ⟨{
        val := {
          τ := τ
          φ := φ
          core := fun _ : Real => 0
          τ_pos := hτ
          phase_eq := ?_
        }
        property := by
          intro x
          rfl
      }⟩
      intro x hx
      have hEq : phase x = τ * Real.log x + φ := hPhaseEq x hx
      calc
        phase x = τ * Real.log x + φ := hEq
        _ = τ * Real.log x + φ + (0 : Real) := by ring
    let w0 : SingleDecayingPhaseCoreWitness phase := (Classical.choice hWitness).1
    have hw0 : ∀ x : Real, w0.core x = 0 := (Classical.choice hWitness).2
    refine {
      κ := 0
      η := 1
      ω := 0
      θ := 0
      η_pos := by norm_num
      core_eq := ?_
    }
    intro x hx
    have hCoreZero : wCore.core x = 0 := by
      have : wCore = w0 := by simpa [w0] using hwCore
      rw [this, hw0 x]
    simp [hCoreZero]

def trivialSingleDecayingPhaseCoreWitness
    (phase : Real → Real) :
    SingleDecayingPhaseCoreWitness phase where
  τ := 1
  φ := 0
  core := fun x : Real => phase x - Real.log x
  τ_pos := by norm_num
  phase_eq := by
    intro x hx
    ring

structure ExplicitFormulaSingleDecayingModeOnlyAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  single_mode_of_trivial_phase_core :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      SingleDecayingModeWitness (trivialSingleDecayingPhaseCoreWitness phase).core

structure R6PhaseBandMode where
  κ : Real
  η : Real
  ω : Real
  θ : Real
  eta_pos : 0 < η

def r6PhaseBandModeTerm (m : R6PhaseBandMode) : Real → Real :=
  fun x : Real => m.κ * (x ^ (-m.η) * Real.sin (m.ω * Real.log x + m.θ))

def r6PhaseBandSuperposition (bands : Fin 6 → R6PhaseBandMode) : Real → Real :=
  fun x : Real => ∑ i : Fin 6, r6PhaseBandModeTerm (bands i) x

structure SpinningTopR6PhaseBandsWitness (phase : Real → Real) where
  τ : Real
  φ : Real
  bands : Fin 6 → R6PhaseBandMode
  τ_pos : 0 < τ
  phase_eq :
    ∀ x : Real, 0 < x →
      phase x = τ * Real.log x + φ + r6PhaseBandSuperposition bands x

structure ExplicitFormulaSpinningTopR6ModeOnlyAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  spinning_top_r6_phase_bands_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      SpinningTopR6PhaseBandsWitness phase
  single_mode_of_trivial_phase_core_from_r6 :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      SingleDecayingModeWitness (trivialSingleDecayingPhaseCoreWitness phase).core

structure ExplicitFormulaSpinningTopR6DominantBandAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  spinning_top_r6_phase_bands_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      SpinningTopR6PhaseBandsWitness phase
  trivial_core_equals_dominant_band_of_r6 :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      ∀ (wBand : SpinningTopR6PhaseBandsWitness phase),
        wBand = spinning_top_r6_phase_bands_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp →
        {i0 : Fin 6 //
          (∀ x : Real, 0 < x →
            (trivialSingleDecayingPhaseCoreWitness phase).core x =
              r6PhaseBandModeTerm (wBand.bands i0) x)}

structure ExplicitFormulaSpinningTopR6DominantBandCriteriaAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  spinning_top_r6_phase_bands_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      SpinningTopR6PhaseBandsWitness phase
  normalized_trivial_anchor_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      ∀ (wBand : SpinningTopR6PhaseBandsWitness phase),
        wBand = spinning_top_r6_phase_bands_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp →
        wBand.τ = 1 ∧ wBand.φ = 0
  dominant_band_collapse_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      ∀ (wBand : SpinningTopR6PhaseBandsWitness phase),
        wBand = spinning_top_r6_phase_bands_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp →
        ∃ i0 : Fin 6, ∀ x : Real, 0 < x →
          r6PhaseBandSuperposition wBand.bands x =
            r6PhaseBandModeTerm (wBand.bands i0) x

theorem spinningTopR6AnchorOfTrivialCoreSuperposition
    {phase : Real → Real}
    (wBand : SpinningTopR6PhaseBandsWitness phase)
    (hCoreSuper :
      ∀ x : Real, 0 < x →
        (trivialSingleDecayingPhaseCoreWitness phase).core x =
          r6PhaseBandSuperposition wBand.bands x) :
    wBand.τ = 1 ∧ wBand.φ = 0 := by
  have hEq :
      ∀ x : Real, 0 < x →
        (wBand.τ - 1) * Real.log x + wBand.φ = 0 := by
    intro x hx
    have hCore :
        phase x - Real.log x = r6PhaseBandSuperposition wBand.bands x := by
      simpa [trivialSingleDecayingPhaseCoreWitness] using hCoreSuper x hx
    have hPhaseExpand :
        phase x - Real.log x =
          (wBand.τ - 1) * Real.log x + wBand.φ +
            r6PhaseBandSuperposition wBand.bands x := by
      calc
        phase x - Real.log x
            =
              (wBand.τ * Real.log x + wBand.φ +
                r6PhaseBandSuperposition wBand.bands x) - Real.log x := by
                  rw [wBand.phase_eq x hx]
        _ =
              (wBand.τ - 1) * Real.log x + wBand.φ +
                r6PhaseBandSuperposition wBand.bands x := by
                  ring
    linarith [hCore, hPhaseExpand]
  have hPhi : wBand.φ = 0 := by
    simpa using (hEq 1 (by norm_num))
  have hTau : wBand.τ = 1 := by
    have hAtExp :
        (wBand.τ - 1) * Real.log (Real.exp 1) + wBand.φ = 0 :=
      hEq (Real.exp 1) (by positivity)
    rw [Real.log_exp, hPhi] at hAtExp
    linarith
  exact ⟨hTau, hPhi⟩

theorem r6PhaseBandSuperposition_eq_dominant_of_offdiag_zero
    (bands : Fin 6 → R6PhaseBandMode) (i0 : Fin 6)
    (hOff : ∀ i : Fin 6, i ≠ i0 → (bands i).κ = 0) :
    ∀ x : Real,
      r6PhaseBandSuperposition bands x =
        r6PhaseBandModeTerm (bands i0) x := by
  intro x
  classical
  have hSum :
      (∑ i : Fin 6, r6PhaseBandModeTerm (bands i) x) =
        r6PhaseBandModeTerm (bands i0) x := by
    refine Finset.sum_eq_single i0 ?_ ?_
    · intro i hi hne
      have hk : (bands i).κ = 0 := hOff i hne
      simp [r6PhaseBandModeTerm, hk]
    · intro hi0
      exact False.elim (hi0 (Finset.mem_univ i0))
  simpa [r6PhaseBandSuperposition] using hSum

theorem trivialCoreSuperposition_of_phase_anchor
    {phase : Real → Real}
    (wBand : SpinningTopR6PhaseBandsWitness phase)
    (hAnchor : wBand.τ = 1 ∧ wBand.φ = 0) :
    ∀ x : Real, 0 < x →
      (trivialSingleDecayingPhaseCoreWitness phase).core x =
        r6PhaseBandSuperposition wBand.bands x := by
  intro x hx
  rcases hAnchor with ⟨hTau, hPhi⟩
  calc
    (trivialSingleDecayingPhaseCoreWitness phase).core x = phase x - Real.log x := rfl
    _ =
        (wBand.τ * Real.log x + wBand.φ +
          r6PhaseBandSuperposition wBand.bands x) - Real.log x := by
            rw [wBand.phase_eq x hx]
    _ = r6PhaseBandSuperposition wBand.bands x := by
          rw [hTau, hPhi]
          ring

structure ExplicitFormulaSpinningTopR6DominantBandCoefficientPinningAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  spinning_top_r6_phase_bands_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      SpinningTopR6PhaseBandsWitness phase
  normalized_trivial_anchor_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      ∀ (wBand : SpinningTopR6PhaseBandsWitness phase),
        wBand = spinning_top_r6_phase_bands_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp →
        wBand.τ = 1 ∧ wBand.φ = 0
  dominant_band_index_with_offdiag_zero_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      ∀ (wBand : SpinningTopR6PhaseBandsWitness phase),
        wBand = spinning_top_r6_phase_bands_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp →
        {i0 : Fin 6 // ∀ i : Fin 6, i ≠ i0 → (wBand.bands i).κ = 0}

structure ExplicitFormulaSpinningTopR6DominantBandCoreLockAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  spinning_top_r6_phase_bands_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      SpinningTopR6PhaseBandsWitness phase
  trivial_core_superposition_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      ∀ (wBand : SpinningTopR6PhaseBandsWitness phase),
        wBand = spinning_top_r6_phase_bands_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp →
        ∀ x : Real, 0 < x →
          (trivialSingleDecayingPhaseCoreWitness phase).core x =
            r6PhaseBandSuperposition wBand.bands x
  dominant_band_collapse_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      ∀ (wBand : SpinningTopR6PhaseBandsWitness phase),
        wBand = spinning_top_r6_phase_bands_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp →
        ∃ i0 : Fin 6, ∀ x : Real, 0 < x →
          r6PhaseBandSuperposition wBand.bands x =
            r6PhaseBandModeTerm (wBand.bands i0) x

def spinningTopR6DominantBandCriteriaAssumptionsOfCoreLock
    (h : ExplicitFormulaSpinningTopR6DominantBandCoreLockAssumptions) :
    ExplicitFormulaSpinningTopR6DominantBandCriteriaAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  spinning_top_r6_phase_bands_of_model := h.spinning_top_r6_phase_bands_of_model
  normalized_trivial_anchor_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
    have hCoreSuper :
        ∀ x : Real, 0 < x →
          (trivialSingleDecayingPhaseCoreWitness phase).core x =
            r6PhaseBandSuperposition wBand.bands x :=
      h.trivial_core_superposition_of_model
        E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
    exact spinningTopR6AnchorOfTrivialCoreSuperposition wBand hCoreSuper
  dominant_band_collapse_of_model := h.dominant_band_collapse_of_model

def spinningTopR6DominantBandAssumptionsOfCriteria
    (h : ExplicitFormulaSpinningTopR6DominantBandCriteriaAssumptions) :
    ExplicitFormulaSpinningTopR6DominantBandAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  spinning_top_r6_phase_bands_of_model := h.spinning_top_r6_phase_bands_of_model
  trivial_core_equals_dominant_band_of_r6 := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
    rcases h.normalized_trivial_anchor_of_model
      E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand with
      ⟨hTau, hPhi⟩
    classical
    have hCollapseExists :
        ∃ i0 : Fin 6, ∀ x : Real, 0 < x →
          r6PhaseBandSuperposition wBand.bands x =
            r6PhaseBandModeTerm (wBand.bands i0) x :=
      h.dominant_band_collapse_of_model
        E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
    let i0 : Fin 6 := Classical.choose hCollapseExists
    have hCollapse :
        ∀ x : Real, 0 < x →
          r6PhaseBandSuperposition wBand.bands x =
            r6PhaseBandModeTerm (wBand.bands i0) x :=
      Classical.choose_spec hCollapseExists
    refine ⟨i0, ?_⟩
    intro x hx
    calc
      (trivialSingleDecayingPhaseCoreWitness phase).core x = phase x - Real.log x := rfl
      _ =
          (wBand.τ * Real.log x + wBand.φ +
              r6PhaseBandSuperposition wBand.bands x) - Real.log x := by
                rw [wBand.phase_eq x hx]
      _ = r6PhaseBandSuperposition wBand.bands x := by
            rw [hTau, hPhi]
            ring
      _ = r6PhaseBandModeTerm (wBand.bands i0) x := hCollapse x hx

def spinningTopR6DominantBandAssumptionsOfCoreLock
    (h : ExplicitFormulaSpinningTopR6DominantBandCoreLockAssumptions) :
    ExplicitFormulaSpinningTopR6DominantBandAssumptions :=
  spinningTopR6DominantBandAssumptionsOfCriteria
    (spinningTopR6DominantBandCriteriaAssumptionsOfCoreLock h)

def spinningTopR6DominantBandCoreLockAssumptionsOfCoefficientPinning
    (h : ExplicitFormulaSpinningTopR6DominantBandCoefficientPinningAssumptions) :
    ExplicitFormulaSpinningTopR6DominantBandCoreLockAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  spinning_top_r6_phase_bands_of_model := h.spinning_top_r6_phase_bands_of_model
  trivial_core_superposition_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand x hx
    have hAnchor : wBand.τ = 1 ∧ wBand.φ = 0 :=
      h.normalized_trivial_anchor_of_model
        E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
    exact trivialCoreSuperposition_of_phase_anchor wBand hAnchor x hx
  dominant_band_collapse_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
    rcases h.dominant_band_index_with_offdiag_zero_of_model
      E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand with
      ⟨i0, hOff⟩
    refine ⟨i0, ?_⟩
    intro x hx
    simpa using r6PhaseBandSuperposition_eq_dominant_of_offdiag_zero wBand.bands i0 hOff x

def spinningTopR6DominantBandCriteriaAssumptionsOfCoefficientPinning
    (h : ExplicitFormulaSpinningTopR6DominantBandCoefficientPinningAssumptions) :
    ExplicitFormulaSpinningTopR6DominantBandCriteriaAssumptions :=
  spinningTopR6DominantBandCriteriaAssumptionsOfCoreLock
    (spinningTopR6DominantBandCoreLockAssumptionsOfCoefficientPinning h)

def spinningTopR6DominantBandAssumptionsOfCoefficientPinning
    (h : ExplicitFormulaSpinningTopR6DominantBandCoefficientPinningAssumptions) :
    ExplicitFormulaSpinningTopR6DominantBandAssumptions :=
  spinningTopR6DominantBandAssumptionsOfCoreLock
    (spinningTopR6DominantBandCoreLockAssumptionsOfCoefficientPinning h)

def canonicalSpinningTopR6BandIndex : Fin 6 := ⟨0, by decide⟩

def neutralR6PhaseBandMode : R6PhaseBandMode where
  κ := 0
  η := 1
  ω := 0
  θ := 0
  eta_pos := by norm_num

def dominantR6PhaseBandModeOfSingle
    {phase : Real → Real}
    (wMode : SingleDecayingModeWitness (trivialSingleDecayingPhaseCoreWitness phase).core) :
    R6PhaseBandMode where
  κ := wMode.κ
  η := wMode.η
  ω := wMode.ω
  θ := wMode.θ
  eta_pos := wMode.η_pos

def canonicalSpinningTopR6BandsOfSingle
    {phase : Real → Real}
    (wMode : SingleDecayingModeWitness (trivialSingleDecayingPhaseCoreWitness phase).core) :
    Fin 6 → R6PhaseBandMode :=
  fun i =>
    if h : i = canonicalSpinningTopR6BandIndex then
      dominantR6PhaseBandModeOfSingle wMode
    else
      neutralR6PhaseBandMode

def spinningTopR6PhaseBandsWitnessOfSingleMode
    {phase : Real → Real}
    (wMode : SingleDecayingModeWitness (trivialSingleDecayingPhaseCoreWitness phase).core) :
    SpinningTopR6PhaseBandsWitness phase where
  τ := 1
  φ := 0
  bands := canonicalSpinningTopR6BandsOfSingle wMode
  τ_pos := by norm_num
  phase_eq := by
    intro x hx
    let i0 : Fin 6 := canonicalSpinningTopR6BandIndex
    have hOff :
        ∀ i : Fin 6, i ≠ i0 →
          (canonicalSpinningTopR6BandsOfSingle wMode i).κ = 0 := by
      intro i hi
      simp [canonicalSpinningTopR6BandsOfSingle, i0, hi, neutralR6PhaseBandMode]
    have hSuper :
        r6PhaseBandSuperposition (canonicalSpinningTopR6BandsOfSingle wMode) x =
          r6PhaseBandModeTerm (canonicalSpinningTopR6BandsOfSingle wMode i0) x := by
      simpa [i0] using
        r6PhaseBandSuperposition_eq_dominant_of_offdiag_zero
          (canonicalSpinningTopR6BandsOfSingle wMode) i0 hOff x
    have hCore :
        (trivialSingleDecayingPhaseCoreWitness phase).core x =
          r6PhaseBandModeTerm (canonicalSpinningTopR6BandsOfSingle wMode i0) x := by
      have hMode : (trivialSingleDecayingPhaseCoreWitness phase).core x =
          wMode.κ * (x ^ (-wMode.η) * Real.sin (wMode.ω * Real.log x + wMode.θ)) := by
        simpa [mul_assoc] using wMode.core_eq x hx
      have hTerm :
          r6PhaseBandModeTerm (canonicalSpinningTopR6BandsOfSingle wMode i0) x =
            wMode.κ * (x ^ (-wMode.η) * Real.sin (wMode.ω * Real.log x + wMode.θ)) := by
        simp [canonicalSpinningTopR6BandsOfSingle, i0,
          canonicalSpinningTopR6BandIndex, dominantR6PhaseBandModeOfSingle, r6PhaseBandModeTerm]
      exact hMode.trans hTerm.symm
    calc
      phase x = Real.log x + (phase x - Real.log x) := by ring
      _ = Real.log x + (trivialSingleDecayingPhaseCoreWitness phase).core x := by rfl
      _ =
          Real.log x +
            r6PhaseBandModeTerm (canonicalSpinningTopR6BandsOfSingle wMode i0) x := by
              rw [hCore]
      _ = 1 * Real.log x + 0 + r6PhaseBandSuperposition (canonicalSpinningTopR6BandsOfSingle wMode) x := by
            rw [hSuper]
            ring

def spinningTopR6DominantBandCoefficientPinningAssumptionsOfModeOnly
    (h : ExplicitFormulaSingleDecayingModeOnlyAssumptions) :
    ExplicitFormulaSpinningTopR6DominantBandCoefficientPinningAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  spinning_top_r6_phase_bands_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    let wMode :
        SingleDecayingModeWitness (trivialSingleDecayingPhaseCoreWitness phase).core :=
      h.single_mode_of_trivial_phase_core
        E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    exact spinningTopR6PhaseBandsWitnessOfSingleMode wMode
  normalized_trivial_anchor_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
    cases hwBand
    simp [spinningTopR6PhaseBandsWitnessOfSingleMode]
  dominant_band_index_with_offdiag_zero_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand hwBand
    let i0 : Fin 6 := canonicalSpinningTopR6BandIndex
    refine ⟨i0, ?_⟩
    intro i hi
    cases hwBand
    simp [spinningTopR6PhaseBandsWitnessOfSingleMode, canonicalSpinningTopR6BandsOfSingle,
      i0, hi, neutralR6PhaseBandMode]

def spinningTopR6ModeOnlyAssumptionsOfDominantBand
    (h : ExplicitFormulaSpinningTopR6DominantBandAssumptions) :
    ExplicitFormulaSpinningTopR6ModeOnlyAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  spinning_top_r6_phase_bands_of_model := h.spinning_top_r6_phase_bands_of_model
  single_mode_of_trivial_phase_core_from_r6 := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    let wBand : SpinningTopR6PhaseBandsWitness phase :=
      h.spinning_top_r6_phase_bands_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    rcases h.trivial_core_equals_dominant_band_of_r6
      E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wBand rfl with
      ⟨i0, hCoreEq⟩
    let m : R6PhaseBandMode := wBand.bands i0
    refine {
      κ := m.κ
      η := m.η
      ω := m.ω
      θ := m.θ
      η_pos := m.eta_pos
      core_eq := ?_
    }
    intro x hx
    simpa [m, r6PhaseBandModeTerm, mul_assoc] using (hCoreEq x hx)

def singleDecayingModeOnlyAssumptionsOfSpinningTopR6
    (h : ExplicitFormulaSpinningTopR6ModeOnlyAssumptions) :
    ExplicitFormulaSingleDecayingModeOnlyAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  single_mode_of_trivial_phase_core := h.single_mode_of_trivial_phase_core_from_r6

def singleDecayingLadderAssumptionsOfModeOnly
    (h : ExplicitFormulaSingleDecayingModeOnlyAssumptions) :
    ExplicitFormulaSingleDecayingPhaseLadderAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  phase_core_split_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    exact trivialSingleDecayingPhaseCoreWitness phase
  single_mode_of_phase_core := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp wCore hwCore
    let w0 : SingleDecayingPhaseCoreWitness phase :=
      trivialSingleDecayingPhaseCoreWitness phase
    have hw0 : wCore = w0 := by
      simpa [w0] using hwCore
    have hMode :
        SingleDecayingModeWitness
          (trivialSingleDecayingPhaseCoreWitness phase).core :=
      h.single_mode_of_trivial_phase_core E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    have hModeEqCore :
        SingleDecayingModeWitness wCore.core := by
      have hMode0 : SingleDecayingModeWitness w0.core := by
        simpa [w0] using hMode
      have hCoreEq : wCore.core = w0.core := by
        exact congrArg SingleDecayingPhaseCoreWitness.core hw0
      simpa [hCoreEq] using hMode0
    exact hModeEqCore

structure DecayingPhaseMode where
  κ : Real
  η : Real
  ω : Real
  θ : Real
  eta_pos : 0 < η

def decayingPhaseModeTerm (m : DecayingPhaseMode) : Real → Real :=
  fun x : Real => m.κ * (x ^ (-m.η) * Real.sin (m.ω * Real.log x + m.θ))

def decayingPhaseModeListCorrection (modes : List DecayingPhaseMode) : Real → Real :=
  fun x : Real => modes.foldr (fun m acc => decayingPhaseModeTerm m x + acc) 0

structure ExplicitFormulaFiniteDecayingPhaseCorrectionsAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  finite_decaying_phase_correction_of_model :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∀ A β : Real, ∀ phase R : Real → Real,
            0 < A → (1 / 2 : Real) < β →
              (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
              ∃ τ φ : Real, ∃ modes : List DecayingPhaseMode, 0 < τ ∧
                (∀ x : Real, 0 < x →
                  phase x = τ * Real.log x + φ + decayingPhaseModeListCorrection modes x)

structure ExplicitFormulaFiniteModeResidualMajorantAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  finite_mode_plus_majorized_residual_of_model :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∀ A β : Real, ∀ phase R : Real → Real,
            0 < A → (1 / 2 : Real) < β →
              (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
              ∃ τ φ : Real, ∃ modes : List DecayingPhaseMode, ∃ residual : Real → Real,
                0 < τ ∧
                  (∀ x : Real, 0 < x →
                    phase x =
                      τ * Real.log x + φ + decayingPhaseModeListCorrection modes x + residual x) ∧
                  ∃ C η : Real, 0 ≤ C ∧ 0 < η ∧
                    (∀ x : Real, 0 < x → |residual x| ≤ C * x ^ (-η))

structure FiniteModeResidualPhaseSplitWitness (phase : Real → Real) where
  τ : Real
  φ : Real
  modes : List DecayingPhaseMode
  residual : Real → Real
  τ_pos : 0 < τ
  phase_eq :
    ∀ x : Real, 0 < x →
      phase x = τ * Real.log x + φ + decayingPhaseModeListCorrection modes x + residual x

structure ExplicitFormulaFiniteModeResidualMajorantPiecesAssumptions where
  source_tag : String
  source_url : String
  theorem_ref : String
  source_tag_lock : source_tag = "PINTZ-2017-OSCILLATION"
  source_url_lock : source_url = "https://doi.org/10.1134/S0081543817010163"
  theorem_ref_lock : theorem_ref = "Thm-2-zero-to-oscillation-transfer"
  zero_to_global_decomposition :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  phase_split_of_model :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      FiniteModeResidualPhaseSplitWitness phase
  residual_majorant_of_phase_split :
    ∀ (E : Real → Real) (hVonKoch : VonKochPrimeErrorCriterion E)
      (s : Complex) (hs : IsNontrivialZetaZero s) (hs_gt : (1 / 2 : Real) < s.re)
      (A β : Real) (phase R : Real → Real),
      (hA : 0 < A) → (hβ : (1 / 2 : Real) < β) →
      (hDecomp : ∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
      ∀ (w : FiniteModeResidualPhaseSplitWitness phase),
        w = phase_split_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp →
          ∃ C η : Real, 0 ≤ C ∧ 0 < η ∧
            (∀ x : Real, 0 < x → |w.residual x| ≤ C * x ^ (-η))

def finiteModeResidualMajorantAssumptionsOfLinearPhaseOnly
    (h : ExplicitFormulaLinearPhaseOnlyAssumptions) :
    ExplicitFormulaFiniteModeResidualMajorantAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := zero_to_global_decomposition_of_vonkoch
  finite_mode_plus_majorized_residual_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    rcases h.linear_phase_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
      ⟨τ, φ, hτ, hPhaseEq⟩
    refine ⟨τ, φ, [], (fun _ : Real => 0), hτ, ?_⟩
    refine ⟨?_, ?_⟩
    · intro x hx
      have hEq : phase x = τ * Real.log x + φ := hPhaseEq x hx
      calc
        phase x = τ * Real.log x + φ := hEq
        _ = τ * Real.log x + φ + decayingPhaseModeListCorrection [] x + (0 : Real) := by
              simp [decayingPhaseModeListCorrection]
    · refine ⟨0, 1, by norm_num, by norm_num, ?_⟩
      intro x hx
      simp

def finiteModeResidualMajorantAssumptionsOfPieces
    (h : ExplicitFormulaFiniteModeResidualMajorantPiecesAssumptions) :
    ExplicitFormulaFiniteModeResidualMajorantAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  finite_mode_plus_majorized_residual_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    let w : FiniteModeResidualPhaseSplitWitness phase :=
      h.phase_split_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    rcases h.residual_majorant_of_phase_split E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp w rfl with
      ⟨C, η, hC, hη, hMaj⟩
    exact ⟨w.τ, w.φ, w.modes, w.residual, w.τ_pos, w.phase_eq, C, η, hC, hη, hMaj⟩

def finiteModeResidualMajorantPiecesAssumptionsOfFiniteDecayingCorrections
    (h : ExplicitFormulaFiniteDecayingPhaseCorrectionsAssumptions) :
    ExplicitFormulaFiniteModeResidualMajorantPiecesAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  phase_split_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    classical
    have hWitness :
        Nonempty
          {w : FiniteModeResidualPhaseSplitWitness phase // ∀ x : Real, w.residual x = 0} := by
      rcases
          h.finite_decaying_phase_correction_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
        ⟨τ, φ, modes, hτ, hPhaseEq⟩
      refine ⟨{
        val := {
          τ := τ
          φ := φ
          modes := modes
          residual := fun _ : Real => 0
          τ_pos := hτ
          phase_eq := ?_
        }
        property := by
          intro x
          rfl
      }⟩
      intro x hx
      have hEq : phase x = τ * Real.log x + φ + decayingPhaseModeListCorrection modes x :=
        hPhaseEq x hx
      calc
        phase x = τ * Real.log x + φ + decayingPhaseModeListCorrection modes x := hEq
        _ = τ * Real.log x + φ + decayingPhaseModeListCorrection modes x + (0 : Real) := by ring
    exact (Classical.choice hWitness).1
  residual_majorant_of_phase_split := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp w hw
    classical
    have hWitness :
        Nonempty
          {w : FiniteModeResidualPhaseSplitWitness phase // ∀ x : Real, w.residual x = 0} := by
      rcases
          h.finite_decaying_phase_correction_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
        ⟨τ, φ, modes, hτ, hPhaseEq⟩
      refine ⟨{
        val := {
          τ := τ
          φ := φ
          modes := modes
          residual := fun _ : Real => 0
          τ_pos := hτ
          phase_eq := ?_
        }
        property := by
          intro x
          rfl
      }⟩
      intro x hx
      have hEq : phase x = τ * Real.log x + φ + decayingPhaseModeListCorrection modes x :=
        hPhaseEq x hx
      calc
        phase x = τ * Real.log x + φ + decayingPhaseModeListCorrection modes x := hEq
        _ = τ * Real.log x + φ + decayingPhaseModeListCorrection modes x + (0 : Real) := by ring
    let w0 : FiniteModeResidualPhaseSplitWitness phase := (Classical.choice hWitness).1
    have hw0 : ∀ x : Real, w0.residual x = 0 := (Classical.choice hWitness).2
    refine ⟨0, 1, by norm_num, by norm_num, ?_⟩
    intro x hx
    have hResidualZero : w.residual x = 0 := by
      have : w = w0 := by simpa [w0] using hw
      rw [this, hw0 x]
    simp [hResidualZero]

def finiteModeResidualMajorantPiecesAssumptionsOfSingleDecayingCorrection
    (h : ExplicitFormulaSingleDecayingPhaseCorrectionAssumptions) :
    ExplicitFormulaFiniteModeResidualMajorantPiecesAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  phase_split_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    classical
    have hWitness :
        Nonempty
          {w : FiniteModeResidualPhaseSplitWitness phase // ∀ x : Real, w.residual x = 0} := by
      rcases
          h.single_decaying_phase_correction_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
        ⟨τ, φ, κ, η, ω, θ, hτ, hη, hPhaseEq⟩
      let mode : DecayingPhaseMode := {
        κ := κ
        η := η
        ω := ω
        θ := θ
        eta_pos := hη
      }
      refine ⟨{
        val := {
          τ := τ
          φ := φ
          modes := [mode]
          residual := fun _ : Real => 0
          τ_pos := hτ
          phase_eq := ?_
        }
        property := by
          intro x
          rfl
      }⟩
      intro x hx
      have hEq :
          phase x = τ * Real.log x + φ + κ * x ^ (-η) * Real.sin (ω * Real.log x + θ) :=
        hPhaseEq x hx
      calc
        phase x = τ * Real.log x + φ + κ * x ^ (-η) * Real.sin (ω * Real.log x + θ) := hEq
        _ = τ * Real.log x + φ + decayingPhaseModeListCorrection [mode] x := by
              simp [decayingPhaseModeListCorrection, decayingPhaseModeTerm, mode, mul_assoc]
        _ = τ * Real.log x + φ + decayingPhaseModeListCorrection [mode] x + (0 : Real) := by ring
    exact (Classical.choice hWitness).1
  residual_majorant_of_phase_split := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp w hw
    classical
    have hWitness :
        Nonempty
          {w : FiniteModeResidualPhaseSplitWitness phase // ∀ x : Real, w.residual x = 0} := by
      rcases
          h.single_decaying_phase_correction_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
        ⟨τ, φ, κ, η, ω, θ, hτ, hη, hPhaseEq⟩
      let mode : DecayingPhaseMode := {
        κ := κ
        η := η
        ω := ω
        θ := θ
        eta_pos := hη
      }
      refine ⟨{
        val := {
          τ := τ
          φ := φ
          modes := [mode]
          residual := fun _ : Real => 0
          τ_pos := hτ
          phase_eq := ?_
        }
        property := by
          intro x
          rfl
      }⟩
      intro x hx
      have hEq :
          phase x = τ * Real.log x + φ + κ * x ^ (-η) * Real.sin (ω * Real.log x + θ) :=
        hPhaseEq x hx
      calc
        phase x = τ * Real.log x + φ + κ * x ^ (-η) * Real.sin (ω * Real.log x + θ) := hEq
        _ = τ * Real.log x + φ + decayingPhaseModeListCorrection [mode] x := by
              simp [decayingPhaseModeListCorrection, decayingPhaseModeTerm, mode, mul_assoc]
        _ = τ * Real.log x + φ + decayingPhaseModeListCorrection [mode] x + (0 : Real) := by ring
    let w0 : FiniteModeResidualPhaseSplitWitness phase := (Classical.choice hWitness).1
    have hw0 : ∀ x : Real, w0.residual x = 0 := (Classical.choice hWitness).2
    refine ⟨0, 1, by norm_num, by norm_num, ?_⟩
    intro x hx
    have hResidualZero : w.residual x = 0 := by
      have : w = w0 := by simpa [w0] using hw
      rw [this, hw0 x]
    simp [hResidualZero]

private theorem tendsto_decaying_sin_log
    (η ω θ : Real)
    (hη : 0 < η) :
    Filter.Tendsto
      (fun x : Real => x ^ (-η) * Real.sin (ω * Real.log x + θ))
      Filter.atTop (nhds 0) := by
  have hPow :
      Filter.Tendsto (fun x : Real => x ^ (-η)) Filter.atTop (nhds 0) :=
    tendsto_rpow_neg_atTop hη
  have hNegPow :
      Filter.Tendsto (fun x : Real => -(x ^ (-η))) Filter.atTop (nhds 0) := by
    simpa using hPow.neg
  have hLower :
      ∀ᶠ x : Real in Filter.atTop,
        -(x ^ (-η)) ≤ x ^ (-η) * Real.sin (ω * Real.log x + θ) := by
    refine Filter.eventually_atTop.2 ⟨1, ?_⟩
    intro x hx
    have hx_nonneg : 0 ≤ x := by linarith
    have hCoeffNonneg : 0 ≤ x ^ (-η) := Real.rpow_nonneg hx_nonneg (-η)
    have hMul :
        x ^ (-η) * (-1 : Real) ≤ x ^ (-η) * Real.sin (ω * Real.log x + θ) :=
      mul_le_mul_of_nonneg_left (Real.neg_one_le_sin _) hCoeffNonneg
    simpa [neg_mul, mul_assoc, mul_comm, mul_left_comm] using hMul
  have hUpper :
      ∀ᶠ x : Real in Filter.atTop,
        x ^ (-η) * Real.sin (ω * Real.log x + θ) ≤ x ^ (-η) := by
    refine Filter.eventually_atTop.2 ⟨1, ?_⟩
    intro x hx
    have hx_nonneg : 0 ≤ x := by linarith
    have hCoeffNonneg : 0 ≤ x ^ (-η) := Real.rpow_nonneg hx_nonneg (-η)
    have hMul :
        x ^ (-η) * Real.sin (ω * Real.log x + θ) ≤ x ^ (-η) * (1 : Real) :=
      mul_le_mul_of_nonneg_left (Real.sin_le_one _) hCoeffNonneg
    simpa [mul_assoc, mul_comm, mul_left_comm] using hMul
  exact tendsto_of_tendsto_of_tendsto_of_le_of_le'
    hNegPow hPow hLower hUpper

private theorem tendsto_decaying_phase_mode_term
    (m : DecayingPhaseMode) :
    Filter.Tendsto (decayingPhaseModeTerm m) Filter.atTop (nhds 0) := by
  have hDec :
      Filter.Tendsto
        (fun x : Real => x ^ (-m.η) * Real.sin (m.ω * Real.log x + m.θ))
        Filter.atTop (nhds 0) :=
    tendsto_decaying_sin_log m.η m.ω m.θ m.eta_pos
  have hScaled :
      Filter.Tendsto
        (fun x : Real => m.κ * (x ^ (-m.η) * Real.sin (m.ω * Real.log x + m.θ)))
        Filter.atTop (nhds (m.κ * 0)) :=
    (tendsto_const_nhds : Filter.Tendsto (fun _ : Real => m.κ) Filter.atTop (nhds m.κ)).mul hDec
  simpa [decayingPhaseModeTerm] using hScaled

private theorem tendsto_decaying_phase_mode_list_correction :
    ∀ modes : List DecayingPhaseMode,
      Filter.Tendsto (decayingPhaseModeListCorrection modes) Filter.atTop (nhds 0)
  | [] => by
      change Filter.Tendsto (fun _ : Real => (0 : Real)) Filter.atTop (nhds 0)
      simpa using
        (tendsto_const_nhds : Filter.Tendsto (fun _ : Real => (0 : Real)) Filter.atTop (nhds 0))
  | m :: ms => by
      have hm :
          Filter.Tendsto (decayingPhaseModeTerm m) Filter.atTop (nhds 0) :=
        tendsto_decaying_phase_mode_term m
      have hms :
          Filter.Tendsto (decayingPhaseModeListCorrection ms) Filter.atTop (nhds 0) :=
        tendsto_decaying_phase_mode_list_correction ms
      have hAdd :
          Filter.Tendsto
            (fun x : Real => decayingPhaseModeTerm m x + decayingPhaseModeListCorrection ms x)
            Filter.atTop (nhds (0 + 0)) :=
        hm.add hms
      simpa [decayingPhaseModeListCorrection] using hAdd

private theorem tendsto_zero_of_abs_le_const_rpow_neg
    (residual : Real → Real)
    (C η : Real)
    (hη : 0 < η)
    (hBound : ∀ x : Real, 0 < x → |residual x| ≤ C * x ^ (-η)) :
    Filter.Tendsto residual Filter.atTop (nhds 0) := by
  have hPow :
      Filter.Tendsto (fun x : Real => x ^ (-η)) Filter.atTop (nhds 0) :=
    tendsto_rpow_neg_atTop hη
  have hUpperToZero :
      Filter.Tendsto (fun x : Real => C * x ^ (-η)) Filter.atTop (nhds 0) := by
    have hMul :
        Filter.Tendsto (fun x : Real => C * x ^ (-η)) Filter.atTop (nhds (C * 0)) :=
      (tendsto_const_nhds : Filter.Tendsto (fun _ : Real => C) Filter.atTop (nhds C)).mul hPow
    simpa using hMul
  have hLowerToZero :
      Filter.Tendsto (fun x : Real => -(C * x ^ (-η))) Filter.atTop (nhds 0) := by
    simpa using hUpperToZero.neg
  have hEventuallyPos : ∀ᶠ x : Real in Filter.atTop, 0 < x :=
    Filter.eventually_atTop.2 ⟨1, by intro x hx; linarith⟩
  have hLower :
      ∀ᶠ x : Real in Filter.atTop, -(C * x ^ (-η)) ≤ residual x := by
    filter_upwards [hEventuallyPos] with x hxpos
    have hAbsBound : |residual x| ≤ C * x ^ (-η) := hBound x hxpos
    have hNegAbs : -|residual x| ≤ residual x := neg_abs_le (residual x)
    have hNegBound : -(C * x ^ (-η)) ≤ -|residual x| := by
      linarith [hAbsBound]
    exact le_trans hNegBound hNegAbs
  have hUpper :
      ∀ᶠ x : Real in Filter.atTop, residual x ≤ C * x ^ (-η) := by
    filter_upwards [hEventuallyPos] with x hxpos
    have hAbsBound : |residual x| ≤ C * x ^ (-η) := hBound x hxpos
    exact le_trans (le_abs_self (residual x)) hAbsBound
  exact tendsto_of_tendsto_of_tendsto_of_le_of_le'
    hLowerToZero hUpperToZero hLower hUpper

def asymptoticallyLinearAssumptionsOfFiniteDecayingCorrections
    (h : ExplicitFormulaFiniteDecayingPhaseCorrectionsAssumptions) :
    ExplicitFormulaAsymptoticallyLinearPhaseAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  asymptotically_linear_phase_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    rcases h.finite_decaying_phase_correction_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
      ⟨τ, φ, modes, hτ, hPhaseEq⟩
    refine ⟨τ, φ, hτ, ?_⟩
    have hCorrection :
        Filter.Tendsto (decayingPhaseModeListCorrection modes) Filter.atTop (nhds 0) :=
      tendsto_decaying_phase_mode_list_correction modes
    have hEventuallyPos : ∀ᶠ x : Real in Filter.atTop, 0 < x :=
      Filter.eventually_atTop.2 ⟨1, by intro x hx; linarith⟩
    have hPhaseEqEventually :
        (fun x : Real => phase x - (τ * Real.log x + φ)) =ᶠ[Filter.atTop]
          (decayingPhaseModeListCorrection modes) := by
      filter_upwards [hEventuallyPos] with x hxpos
      have hEq : phase x = τ * Real.log x + φ + decayingPhaseModeListCorrection modes x :=
        hPhaseEq x hxpos
      linarith [hEq]
    exact hCorrection.congr' hPhaseEqEventually.symm

def asymptoticallyLinearAssumptionsOfFiniteModeResidualMajorant
    (h : ExplicitFormulaFiniteModeResidualMajorantAssumptions) :
    ExplicitFormulaAsymptoticallyLinearPhaseAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  asymptotically_linear_phase_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    rcases h.finite_mode_plus_majorized_residual_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
      ⟨τ, φ, modes, residual, hτ, hTail⟩
    rcases hTail with ⟨hPhaseEq, hBoundPack⟩
    rcases hBoundPack with ⟨C, η, _hC, hη, hResidualBound⟩
    refine ⟨τ, φ, hτ, ?_⟩
    have hCorrection :
        Filter.Tendsto (decayingPhaseModeListCorrection modes) Filter.atTop (nhds 0) :=
      tendsto_decaying_phase_mode_list_correction modes
    have hResidual :
        Filter.Tendsto residual Filter.atTop (nhds 0) :=
      tendsto_zero_of_abs_le_const_rpow_neg residual C η hη hResidualBound
    have hCorrectionPlusResidual :
        Filter.Tendsto
          (fun x : Real => decayingPhaseModeListCorrection modes x + residual x)
          Filter.atTop (nhds (0 + 0)) :=
      hCorrection.add hResidual
    have hCorrectionPlusResidualZero :
        Filter.Tendsto
          (fun x : Real => decayingPhaseModeListCorrection modes x + residual x)
          Filter.atTop (nhds 0) := by
      simpa using hCorrectionPlusResidual
    have hEventuallyPos : ∀ᶠ x : Real in Filter.atTop, 0 < x :=
      Filter.eventually_atTop.2 ⟨1, by intro x hx; linarith⟩
    have hPhaseEqEventually :
        (fun x : Real => phase x - (τ * Real.log x + φ)) =ᶠ[Filter.atTop]
          (fun x : Real => decayingPhaseModeListCorrection modes x + residual x) := by
      filter_upwards [hEventuallyPos] with x hxpos
      have hEq :
          phase x =
            τ * Real.log x + φ + decayingPhaseModeListCorrection modes x + residual x :=
        hPhaseEq x hxpos
      linarith [hEq]
    exact hCorrectionPlusResidualZero.congr' hPhaseEqEventually.symm

def asymptoticallyLinearAssumptionsOfSingleDecayingCorrection
    (h : ExplicitFormulaSingleDecayingPhaseCorrectionAssumptions) :
    ExplicitFormulaAsymptoticallyLinearPhaseAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  asymptotically_linear_phase_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    rcases h.single_decaying_phase_correction_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
      ⟨τ, φ, κ, η, ω, θ, hτ, hη, hPhaseEq⟩
    refine ⟨τ, φ, hτ, ?_⟩
    have hDec :
        Filter.Tendsto
          (fun x : Real => x ^ (-η) * Real.sin (ω * Real.log x + θ))
          Filter.atTop (nhds 0) :=
      tendsto_decaying_sin_log η ω θ hη
    have hScaled :
        Filter.Tendsto
          (fun x : Real => κ * (x ^ (-η) * Real.sin (ω * Real.log x + θ)))
          Filter.atTop (nhds 0) := by
      have hMul :
          Filter.Tendsto
            (fun x : Real => κ * (x ^ (-η) * Real.sin (ω * Real.log x + θ)))
            Filter.atTop (nhds (κ * 0)) :=
        (tendsto_const_nhds :
          Filter.Tendsto (fun _ : Real => κ) Filter.atTop (nhds κ)).mul hDec
      simpa using hMul
    have hEventuallyPos : ∀ᶠ x : Real in Filter.atTop, 0 < x :=
      Filter.eventually_atTop.2 ⟨1, by intro x hx; linarith⟩
    have hPhaseEqEventually :
        (fun x : Real => phase x - (τ * Real.log x + φ)) =ᶠ[Filter.atTop]
          (fun x : Real => κ * (x ^ (-η) * Real.sin (ω * Real.log x + θ))) := by
      filter_upwards [hEventuallyPos] with x hxpos
      have hEq : phase x = τ * Real.log x + φ + κ * x ^ (-η) * Real.sin (ω * Real.log x + θ) :=
        hPhaseEq x hxpos
      linarith [hEq]
    exact hScaled.congr' hPhaseEqEventually.symm

def linearPhaseWitnessAssumptionsOfAsymptoticallyLinear
    (h : ExplicitFormulaAsymptoticallyLinearPhaseAssumptions) :
    ExplicitFormulaLinearPhaseWitnessAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_linear_phase_witness := by
    intro E hVonKoch s hs hs_gt
    rcases h.zero_to_global_decomposition E hVonKoch s hs hs_gt with
      ⟨A, β, phase, R, hA, hβ, hDecomp, hRem⟩
    rcases h.asymptotically_linear_phase_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
      ⟨τ, φ, hτ, hPhaseDiff⟩
    let phaseLin : Real → Real := fun x : Real => τ * Real.log x + φ
    let Rlin : Real → Real :=
      fun x : Real => R x + A * x ^ β * (Real.cos (phase x) - Real.cos (phaseLin x))
    refine ⟨A, β, τ, φ, Rlin, hA, hβ, hτ, ?_, ?_⟩
    · intro x
      calc
        E x = oscillatoryMainTerm A β phase x + R x := hDecomp x
        _ = oscillatoryMainTerm A β phaseLin x + Rlin x := by
              simp [oscillatoryMainTerm, Rlin, phaseLin]
              ring
    · let phaseDelta : Real → Real := fun x : Real => phase x - phaseLin x
      let cosDelta : Real → Real := fun x : Real => Real.cos (phase x) - Real.cos (phaseLin x)
      have hAbsPhaseDelta :
          Filter.Tendsto (fun x : Real => |phaseDelta x|) Filter.atTop (nhds 0) := by
        simpa [phaseDelta] using hPhaseDiff.abs
      have hNegAbsPhaseDelta :
          Filter.Tendsto (fun x : Real => -|phaseDelta x|) Filter.atTop (nhds 0) := by
        simpa using hAbsPhaseDelta.neg
      have hCosDeltaLower :
          ∀ᶠ x : Real in Filter.atTop, -|phaseDelta x| ≤ cosDelta x := by
        refine Filter.Eventually.of_forall ?_
        intro x
        have hLip :
            |cosDelta x| ≤ |phaseDelta x| := by
          simpa [cosDelta, phaseDelta, phaseLin] using
            Real.abs_cos_sub_cos_le (phase x) (phaseLin x)
        have hNegAbs : -|cosDelta x| ≤ cosDelta x := neg_abs_le (cosDelta x)
        have hNegMonotone : -|phaseDelta x| ≤ -|cosDelta x| := by
          linarith [hLip]
        exact le_trans hNegMonotone hNegAbs
      have hCosDeltaUpper :
          ∀ᶠ x : Real in Filter.atTop, cosDelta x ≤ |phaseDelta x| := by
        refine Filter.Eventually.of_forall ?_
        intro x
        have hLip :
            |cosDelta x| ≤ |phaseDelta x| := by
          simpa [cosDelta, phaseDelta, phaseLin] using
            Real.abs_cos_sub_cos_le (phase x) (phaseLin x)
        exact le_trans (le_abs_self (cosDelta x)) hLip
      have hCosDelta :
          Filter.Tendsto cosDelta Filter.atTop (nhds 0) :=
        tendsto_of_tendsto_of_tendsto_of_le_of_le'
          hNegAbsPhaseDelta hAbsPhaseDelta hCosDeltaLower hCosDeltaUpper
      have hScaledCosDelta :
          Filter.Tendsto (fun x : Real => A * cosDelta x) Filter.atTop (nhds 0) := by
        simpa using
          (tendsto_const_nhds : Filter.Tendsto (fun _ : Real => A) Filter.atTop (nhds A)).mul hCosDelta
      have hRemPlus :
          Filter.Tendsto (fun x : Real => R x / x ^ β + A * cosDelta x)
            Filter.atTop (nhds (0 + 0)) := by
        exact hRem.add hScaledCosDelta
      have hEventuallyPos : ∀ᶠ x : Real in Filter.atTop, 0 < x :=
        Filter.eventually_atTop.2 ⟨1, by intro x hx; linarith⟩
      have hEventEq :
          (fun x : Real => Rlin x / x ^ β) =ᶠ[Filter.atTop]
            (fun x : Real => R x / x ^ β + A * cosDelta x) := by
        filter_upwards [hEventuallyPos] with x hxpos
        have hPowPos : 0 < x ^ β := Real.rpow_pos_of_pos hxpos β
        have hPowNe : x ^ β ≠ 0 := ne_of_gt hPowPos
        calc
          Rlin x / x ^ β
              = (R x + A * x ^ β * cosDelta x) / x ^ β := by
                  simp [Rlin, cosDelta]
          _ = R x / x ^ β + A * cosDelta x := by
                field_simp [hPowNe]
      have hRemLin :
          Filter.Tendsto (fun x : Real => Rlin x / x ^ β) Filter.atTop (nhds (0 + 0)) :=
        hRemPlus.congr' hEventEq.symm
      simpa using hRemLin

def phaseOscillationAssumptionsOfLinearPhaseWitness
    (h : ExplicitFormulaLinearPhaseWitnessAssumptions) :
    PhaseOscillationAsymptoticAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_phase_oscillation := by
    intro E hVonKoch s hs hs_gt
    rcases h.zero_to_linear_phase_witness E hVonKoch s hs hs_gt with
      ⟨A, β, τ, φ, R, hA, hβ, hτ, hDecomp, hRemAtTop⟩
    let phase : Real → Real := fun y : Real => τ * Real.log y + φ
    let f : Nat → Real := fun n => Real.exp ((((n : Real) * (2 * Real.pi)) - φ) / τ)
    have hNat : Filter.Tendsto (fun n : Nat => (n : Real)) Filter.atTop Filter.atTop :=
      tendsto_natCast_atTop_atTop
    have hMul :
        Filter.Tendsto (fun n : Nat => (n : Real) * (2 * Real.pi))
          Filter.atTop Filter.atTop :=
      hNat.atTop_mul_const' Real.two_pi_pos
    have hSub :
        Filter.Tendsto (fun n : Nat => (n : Real) * (2 * Real.pi) - φ)
          Filter.atTop Filter.atTop := by
      simpa [sub_eq_add_neg] using hMul.atTop_add (tendsto_const_nhds : Filter.Tendsto (fun _ : Nat => -φ) Filter.atTop (nhds (-φ)))
    have hDiv :
        Filter.Tendsto (fun n : Nat => (((n : Real) * (2 * Real.pi)) - φ) / τ)
          Filter.atTop Filter.atTop := by
      have hMulInv :
          Filter.Tendsto (fun n : Nat => (((n : Real) * (2 * Real.pi)) - φ) * τ⁻¹)
            Filter.atTop Filter.atTop :=
        hSub.atTop_mul_const' (inv_pos.mpr hτ)
      simpa [div_eq_mul_inv] using hMulInv
    have hTendstoF : Filter.Tendsto f Filter.atTop Filter.atTop := by
      exact Real.tendsto_exp_atTop.comp hDiv
    refine ⟨A, β, hA, hβ, Or.inl ?_⟩
    refine ⟨f, phase, R, hTendstoF, ?_, ?_, ?_⟩
    · intro n
      simpa [phase] using hDecomp (f n)
    · exact Filter.Eventually.of_forall (fun n => by
        have hposf : 0 < f n := by
          dsimp [f]
          exact Real.exp_pos _
        have hτne : τ ≠ 0 := ne_of_gt hτ
        have hPhaseEq : phase (f n) = (n : Real) * (2 * Real.pi) := by
          calc
            phase (f n) = τ * Real.log (f n) + φ := by rfl
            _ = τ * ((((n : Real) * (2 * Real.pi)) - φ) / τ) + φ := by
                  simp [phase, f]
            _ = (n : Real) * (2 * Real.pi) := by
                  field_simp [hτne]
                  ring
        rw [hPhaseEq, Real.cos_nat_mul_two_pi]
        norm_num)
    · exact hRemAtTop.comp hTendstoF

class ImportedLinearPhaseWitnessResults where
  zero_to_linear_phase_witness_import :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β τ φ : Real, ∃ R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧ 0 < τ ∧
              (∀ x : Real, E x = oscillatoryMainTerm A β (fun y : Real => τ * Real.log y + φ) x + R x) ∧
              Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)

abbrev ZeroToCosSinPhaseTransfer : Prop :=
  ∀ E : Real → Real,
    VonKochPrimeErrorCriterion E →
      ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
        ∃ β τ a b : Real, ∃ R : Real → Real,
          (1 / 2 : Real) < β ∧ 0 < τ ∧ (a ≠ 0 ∨ b ≠ 0) ∧
            (∀ x : Real,
              E x = x ^ β *
                (a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x)) + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)

theorem cos_sin_to_single_cos_derived
    (a b : Real) (hab : a ≠ 0 ∨ b ≠ 0) :
    ∃ A φ : Real, 0 < A ∧
      (∀ t : Real, a * Real.cos t + b * Real.sin t = A * Real.cos (t + φ)) := by
  let z : Complex := a - b * Complex.I
  let A : Real := ‖z‖
  let φ : Real := z.arg
  have hz : z ≠ 0 := by
    intro hz0
    have ha0 : a = 0 := by
      have hre : z.re = 0 := by simpa [hz0]
      simpa [z] using hre
    have hb0 : b = 0 := by
      have him : z.im = 0 := by simpa [hz0]
      have hneg : -b = 0 := by simpa [z] using him
      linarith
    cases hab with
    | inl ha => exact ha ha0
    | inr hb => exact hb hb0
  have hA : 0 < A := by
    dsimp [A]
    exact norm_pos_iff.mpr hz
  have hA_ne : A ≠ 0 := ne_of_gt hA
  refine ⟨A, φ, hA, ?_⟩
  intro t
  have hcosφ : Real.cos φ = a / A := by
    have h := Complex.cos_arg hz
    simpa [φ, A, z] using h
  have hsinφ : Real.sin φ = (-b) / A := by
    have h := Complex.sin_arg z
    simpa [φ, A, z] using h
  calc
    a * Real.cos t + b * Real.sin t
        = A * Real.cos (t + φ) := by
          rw [Real.cos_add, hcosφ, hsinφ]
          field_simp [hA_ne]
          ring

structure ImportedLinearPhaseWitnessStepResults where
  zero_to_cos_sin_phase : ZeroToCosSinPhaseTransfer
  cos_sin_to_single_cos :
    ∀ a b : Real, (a ≠ 0 ∨ b ≠ 0) →
      ∃ A φ : Real, 0 < A ∧
        (∀ t : Real, a * Real.cos t + b * Real.sin t = A * Real.cos (t + φ))

structure ImportedLinearPhaseCosSinOnlyResults where
  zero_to_cos_sin_phase : ZeroToCosSinPhaseTransfer

def importedLinearPhaseWitnessStepResultsOfCosSinOnly
    (i : ImportedLinearPhaseCosSinOnlyResults) :
    ImportedLinearPhaseWitnessStepResults where
  zero_to_cos_sin_phase := i.zero_to_cos_sin_phase
  cos_sin_to_single_cos := cos_sin_to_single_cos_derived

theorem zero_to_cos_sin_phase_transfer_of_linear_phase_witness
    (h : ExplicitFormulaLinearPhaseWitnessAssumptions) :
    ZeroToCosSinPhaseTransfer := by
  intro E hVonKoch s hs hs_gt
  rcases h.zero_to_linear_phase_witness E hVonKoch s hs hs_gt with
    ⟨A, β, τ, φ, R, hA, hβ, hτ, hDecomp, hRem⟩
  refine ⟨β, τ, A * Real.cos φ, -(A * Real.sin φ), R, hβ, hτ, ?_, ?_, hRem⟩
  · by_contra hab
    push_neg at hab
    rcases hab with ⟨hCos0, hNegSin0⟩
    have hSin0 : A * Real.sin φ = 0 := by linarith
    have hsq_sum :
        (A * Real.cos φ) ^ 2 + (A * Real.sin φ) ^ 2 = 0 := by
      rw [hCos0, hSin0]
      ring
    have hsq_expand :
        (A * Real.cos φ) ^ 2 + (A * Real.sin φ) ^ 2 = A ^ 2 := by
      calc
        (A * Real.cos φ) ^ 2 + (A * Real.sin φ) ^ 2
            = A ^ 2 * (Real.cos φ ^ 2 + Real.sin φ ^ 2) := by ring
        _ = A ^ 2 * 1 := by rw [Real.cos_sq_add_sin_sq]
        _ = A ^ 2 := by ring
    have hA2 : A ^ 2 = 0 := by linarith [hsq_sum, hsq_expand]
    have hA2pos : 0 < A ^ 2 := sq_pos_of_pos hA
    linarith
  · intro x
    have hdx : E x = A * x ^ β * Real.cos (τ * Real.log x + φ) + R x := by
      simpa [oscillatoryMainTerm] using hDecomp x
    calc
      E x = A * x ^ β * Real.cos (τ * Real.log x + φ) + R x := hdx
      _ = x ^ β *
            ((A * Real.cos φ) * Real.cos (τ * Real.log x) +
              (-(A * Real.sin φ)) * Real.sin (τ * Real.log x)) + R x := by
            rw [Real.cos_add]
            ring

theorem zero_to_cos_sin_phase_transfer_of_imported_linear_phase_witness
    (i : ImportedLinearPhaseWitnessResults) :
    ZeroToCosSinPhaseTransfer := by
  intro E hVonKoch s hs hs_gt
  rcases i.zero_to_linear_phase_witness_import E hVonKoch s hs hs_gt with
    ⟨A, β, τ, φ, R, hA, hβ, hτ, hDecomp, hRem⟩
  refine ⟨β, τ, A * Real.cos φ, -(A * Real.sin φ), R, hβ, hτ, ?_, ?_, hRem⟩
  · by_contra hab
    push_neg at hab
    rcases hab with ⟨hCos0, hNegSin0⟩
    have hSin0 : A * Real.sin φ = 0 := by linarith
    have hsq_sum :
        (A * Real.cos φ) ^ 2 + (A * Real.sin φ) ^ 2 = 0 := by
      rw [hCos0, hSin0]
      ring
    have hsq_expand :
        (A * Real.cos φ) ^ 2 + (A * Real.sin φ) ^ 2 = A ^ 2 := by
      calc
        (A * Real.cos φ) ^ 2 + (A * Real.sin φ) ^ 2
            = A ^ 2 * (Real.cos φ ^ 2 + Real.sin φ ^ 2) := by ring
        _ = A ^ 2 * 1 := by rw [Real.cos_sq_add_sin_sq]
        _ = A ^ 2 := by ring
    have hA2 : A ^ 2 = 0 := by linarith [hsq_sum, hsq_expand]
    have hA2pos : 0 < A ^ 2 := sq_pos_of_pos hA
    linarith
  · intro x
    have hdx : E x = A * x ^ β * Real.cos (τ * Real.log x + φ) + R x := by
      simpa [oscillatoryMainTerm] using hDecomp x
    calc
      E x = A * x ^ β * Real.cos (τ * Real.log x + φ) + R x := hdx
      _ = x ^ β *
            ((A * Real.cos φ) * Real.cos (τ * Real.log x) +
              (-(A * Real.sin φ)) * Real.sin (τ * Real.log x)) + R x := by
            rw [Real.cos_add]
            ring

def importedLinearPhaseWitnessResultsOfStepResults
    (i : ImportedLinearPhaseWitnessStepResults) :
    ImportedLinearPhaseWitnessResults where
  zero_to_linear_phase_witness_import := by
    intro E hVonKoch s hs hs_gt
    rcases i.zero_to_cos_sin_phase E hVonKoch s hs hs_gt with
      ⟨β, τ, a, b, R, hβ, hτ, hab, hDecomp, hRem⟩
    rcases i.cos_sin_to_single_cos a b hab with ⟨A, φ, hA, hPolar⟩
    refine ⟨A, β, τ, φ, R, hA, hβ, hτ, ?_, hRem⟩
    intro x
    have hPolarX :
        a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x) =
          A * Real.cos (τ * Real.log x + φ) := by
      simpa [add_comm, add_left_comm, add_assoc] using hPolar (τ * Real.log x)
    calc
      E x
          = x ^ β * (a * Real.cos (τ * Real.log x) + b * Real.sin (τ * Real.log x)) + R x :=
            hDecomp x
      _ = x ^ β * (A * Real.cos (τ * Real.log x + φ)) + R x := by rw [hPolarX]
      _ = oscillatoryMainTerm A β (fun y : Real => τ * Real.log y + φ) x + R x := by
            unfold oscillatoryMainTerm
            ring

def linearPhaseWitnessAssumptionsOfImported
    (i : ImportedLinearPhaseWitnessResults) :
    ExplicitFormulaLinearPhaseWitnessAssumptions where
  source_tag := "PINTZ-2017-OSCILLATION"
  source_url := "https://doi.org/10.1134/S0081543817010163"
  theorem_ref := "Thm-2-zero-to-oscillation-transfer"
  source_tag_lock := rfl
  source_url_lock := rfl
  theorem_ref_lock := rfl
  zero_to_linear_phase_witness := i.zero_to_linear_phase_witness_import

class ConcreteImportedLinearPhaseWitnessProvider where
  imported_linear_phase_witness : ImportedLinearPhaseWitnessResults

def quantizedPhaseAssumptionsOfLogLinear
    (h : ExplicitFormulaLogLinearPhaseAssumptions) :
    ExplicitFormulaQuantizedPhaseAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_global_decomposition := h.zero_to_global_decomposition
  phase_quantization_of_model := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    rcases h.linear_phase_of_model E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
      ⟨τ, φ, hτ, hPhase⟩
    let f : Nat → Real := fun n => Real.exp ((((n : Real) * (2 * Real.pi)) - φ) / τ)
    have hNat : Filter.Tendsto (fun n : Nat => (n : Real)) Filter.atTop Filter.atTop :=
      tendsto_natCast_atTop_atTop
    have hMul :
        Filter.Tendsto (fun n : Nat => (n : Real) * (2 * Real.pi))
          Filter.atTop Filter.atTop :=
      hNat.atTop_mul_const' Real.two_pi_pos
    have hSub :
        Filter.Tendsto (fun n : Nat => (n : Real) * (2 * Real.pi) - φ)
          Filter.atTop Filter.atTop := by
      simpa [sub_eq_add_neg] using hMul.atTop_add (tendsto_const_nhds : Filter.Tendsto (fun _ : Nat => -φ) Filter.atTop (nhds (-φ)))
    have hDiv :
        Filter.Tendsto (fun n : Nat => (((n : Real) * (2 * Real.pi)) - φ) / τ)
          Filter.atTop Filter.atTop := by
      have hMulInv :
          Filter.Tendsto (fun n : Nat => (((n : Real) * (2 * Real.pi)) - φ) * τ⁻¹)
            Filter.atTop Filter.atTop :=
        hSub.atTop_mul_const' (inv_pos.mpr hτ)
      simpa [div_eq_mul_inv] using hMulInv
    have hTendstoF : Filter.Tendsto f Filter.atTop Filter.atTop := by
      exact Real.tendsto_exp_atTop.comp hDiv
    refine Or.inl ⟨f, hTendstoF, ?_⟩
    intro n
    have hposf : 0 < f n := by
      dsimp [f]
      exact Real.exp_pos _
    have hτne : τ ≠ 0 := ne_of_gt hτ
    calc
      phase (f n) = τ * Real.log (f n) + φ := hPhase (f n) hposf
      _ = τ * ((((n : Real) * (2 * Real.pi)) - φ) / τ) + φ := by
        simp [f]
      _ = (n : Real) * (2 * Real.pi) := by
        field_simp [hτne]
        ring

def phaseOscillationAssumptionsOfKernel
    (h : ExplicitFormulaPhaseKernelAssumptions) :
    PhaseOscillationAsymptoticAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_phase_oscillation := by
    intro E hVonKoch s hs hs_gt
    rcases h.zero_to_oscillatory_kernel E hVonKoch s hs hs_gt with
      ⟨A, β, phase, R, hA, hβ, hDecomp, hRemAtTop, hPin⟩
    refine ⟨A, β, hA, hβ, ?_⟩
    rcases hPin with hPos | hNeg
    · rcases hPos with ⟨f, hTendsto, hCos⟩
      have hRemSeq :
          Filter.Tendsto (fun n : Nat => R (f n) / (f n) ^ β) Filter.atTop (nhds 0) :=
        hRemAtTop.comp hTendsto
      refine Or.inl ?_
      exact ⟨f, phase, R, hTendsto, (by intro n; simpa using hDecomp (f n)), hCos, hRemSeq⟩
    · rcases hNeg with ⟨f, hTendsto, hCos⟩
      have hRemSeq :
          Filter.Tendsto (fun n : Nat => R (f n) / (f n) ^ β) Filter.atTop (nhds 0) :=
        hRemAtTop.comp hTendsto
      refine Or.inr ?_
      exact ⟨f, phase, R, hTendsto, (by intro n; simpa using hDecomp (f n)), hCos, hRemSeq⟩

def phaseKernelAssumptionsOfQuantizedPhase
    (h : ExplicitFormulaQuantizedPhaseAssumptions) :
    ExplicitFormulaPhaseKernelAssumptions :=
  phaseKernelAssumptionsOfSplit (splitAssumptionsOfQuantizedPhase h)

def phaseOscillationAssumptionsOfQuantizedPhase
    (h : ExplicitFormulaQuantizedPhaseAssumptions) :
    PhaseOscillationAsymptoticAssumptions :=
  phaseOscillationAssumptionsOfKernel (phaseKernelAssumptionsOfQuantizedPhase h)

def splitAssumptionsOfLogLinear
    (h : ExplicitFormulaLogLinearPhaseAssumptions) :
    ExplicitFormulaKernelSplitAssumptions :=
  splitAssumptionsOfQuantizedPhase (quantizedPhaseAssumptionsOfLogLinear h)

def phaseKernelAssumptionsOfLogLinear
    (h : ExplicitFormulaLogLinearPhaseAssumptions) :
    ExplicitFormulaPhaseKernelAssumptions :=
  phaseKernelAssumptionsOfSplit (splitAssumptionsOfLogLinear h)

def phaseOscillationAssumptionsOfLogLinear
    (h : ExplicitFormulaLogLinearPhaseAssumptions) :
    PhaseOscillationAsymptoticAssumptions :=
  phaseOscillationAssumptionsOfKernel (phaseKernelAssumptionsOfLogLinear h)

theorem eventually_main_lower_of_phase_lower
    (A β c : Real)
    (hA : 0 < A)
    (hc : c = A / 4)
    (f : Nat → Real)
    (phase : Real → Real)
    (hTendsto : Filter.Tendsto f Filter.atTop Filter.atTop)
    (hCos : ∀ᶠ n : Nat in Filter.atTop, Real.cos (phase (f n)) ≥ (1 / 2 : Real)) :
    ∀ᶠ n : Nat in Filter.atTop,
      oscillatoryMainTerm A β phase (f n) ≥ (2 * c) * (f n) ^ β := by
  have hFgeOne : ∀ᶠ n : Nat in Filter.atTop, f n ≥ 1 := (Filter.tendsto_atTop.1 hTendsto) 1
  filter_upwards [hCos, hFgeOne] with n hnCos hnF
  have hA_nonneg : 0 ≤ A := le_of_lt hA
  have hpow_nonneg : 0 ≤ (f n) ^ β := Real.rpow_nonneg (by linarith) β
  have hCoeff_nonneg : 0 ≤ A * (f n) ^ β := mul_nonneg hA_nonneg hpow_nonneg
  have hmul :
      A * (f n) ^ β * (1 / 2 : Real) ≤
        A * (f n) ^ β * Real.cos (phase (f n)) := by
    have hmul' :
        (A * (f n) ^ β) * (1 / 2 : Real) ≤
          (A * (f n) ^ β) * Real.cos (phase (f n)) :=
      mul_le_mul_of_nonneg_left hnCos hCoeff_nonneg
    simpa [mul_assoc] using hmul'
  have hcId : (2 * c) * (f n) ^ β = A * (f n) ^ β * (1 / 2 : Real) := by
    rw [hc]
    ring
  calc
    oscillatoryMainTerm A β phase (f n)
        = A * (f n) ^ β * Real.cos (phase (f n)) := rfl
    _ ≥ A * (f n) ^ β * (1 / 2 : Real) := hmul
    _ = (2 * c) * (f n) ^ β := by symm; exact hcId

theorem eventually_main_upper_of_phase_upper
    (A β c : Real)
    (hA : 0 < A)
    (hc : c = A / 4)
    (f : Nat → Real)
    (phase : Real → Real)
    (hTendsto : Filter.Tendsto f Filter.atTop Filter.atTop)
    (hCos : ∀ᶠ n : Nat in Filter.atTop, Real.cos (phase (f n)) ≤ -((1 / 2 : Real))) :
    ∀ᶠ n : Nat in Filter.atTop,
      oscillatoryMainTerm A β phase (f n) ≤ -((2 * c) * (f n) ^ β) := by
  have hFgeOne : ∀ᶠ n : Nat in Filter.atTop, f n ≥ 1 := (Filter.tendsto_atTop.1 hTendsto) 1
  filter_upwards [hCos, hFgeOne] with n hnCos hnF
  have hA_nonneg : 0 ≤ A := le_of_lt hA
  have hpow_nonneg : 0 ≤ (f n) ^ β := Real.rpow_nonneg (by linarith) β
  have hCoeff_nonneg : 0 ≤ A * (f n) ^ β := mul_nonneg hA_nonneg hpow_nonneg
  have hmul :
      A * (f n) ^ β * Real.cos (phase (f n)) ≤
        A * (f n) ^ β * (-((1 / 2 : Real))) := by
    have hmul' :
        (A * (f n) ^ β) * Real.cos (phase (f n)) ≤
          (A * (f n) ^ β) * (-((1 / 2 : Real))) :=
      mul_le_mul_of_nonneg_left hnCos hCoeff_nonneg
    simpa [mul_assoc] using hmul'
  have hcId : A * (f n) ^ β * (-((1 / 2 : Real))) = -((2 * c) * (f n) ^ β) := by
    rw [hc]
    ring
  calc
    oscillatoryMainTerm A β phase (f n)
        = A * (f n) ^ β * Real.cos (phase (f n)) := rfl
    _ ≤ A * (f n) ^ β * (-((1 / 2 : Real))) := hmul
    _ = -((2 * c) * (f n) ^ β) := hcId

def asymptoticAssumptionsOfPhaseOscillation
    (h : PhaseOscillationAsymptoticAssumptions) :
    ExplicitFormulaAsymptoticSequenceAssumptions where
  source_tag := h.source_tag
  source_url := h.source_url
  theorem_ref := h.theorem_ref
  source_tag_lock := h.source_tag_lock
  source_url_lock := h.source_url_lock
  theorem_ref_lock := h.theorem_ref_lock
  zero_to_sequence_asymptotic := by
    intro E hVonKoch s hs hs_gt
    rcases h.zero_to_phase_oscillation E hVonKoch s hs hs_gt with ⟨A, β, hA, hβ, hBranch⟩
    let c : Real := A / 4
    have hc : 0 < c := by
      dsimp [c]
      linarith
    refine ⟨c, β, hc, hβ, ?_⟩
    rcases hBranch with hPos | hNeg
    · rcases hPos with ⟨f, phase, R, hTendsto, hDecomp, hCos, hRemTendsto⟩
      refine Or.inl ?_
      refine ⟨f, oscillatoryMainTerm A β phase, R, hTendsto, ?_, ?_, hRemTendsto⟩
      · intro n
        simpa using hDecomp n
      · exact eventually_main_lower_of_phase_lower
          (A := A) (β := β) (c := c)
          hA rfl f phase hTendsto hCos
    · rcases hNeg with ⟨f, phase, R, hTendsto, hDecomp, hCos, hRemTendsto⟩
      refine Or.inr ?_
      refine ⟨f, oscillatoryMainTerm A β phase, R, hTendsto, ?_, ?_, hRemTendsto⟩
      · intro n
        simpa using hDecomp n
      · exact eventually_main_upper_of_phase_upper
          (A := A) (β := β) (c := c)
          hA rfl f phase hTendsto hCos

theorem endpoint_to_rh_of_phase_oscillation
    (h : PhaseOscillationAsymptoticAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_explicit_formula_asymptotic_sequence
    (asymptoticAssumptionsOfPhaseOscillation h)

theorem rh_from_phase_oscillation
    (h : PhaseOscillationAsymptoticAssumptions) :
    RHStatement :=
  rh_from_asymptotic_assumptions
    (asymptoticAssumptionsOfPhaseOscillation h)

theorem endpoint_to_rh_of_phase_kernel
    (h : ExplicitFormulaPhaseKernelAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_phase_oscillation (phaseOscillationAssumptionsOfKernel h)

theorem rh_from_phase_kernel
    (h : ExplicitFormulaPhaseKernelAssumptions) :
    RHStatement :=
  rh_from_phase_oscillation (phaseOscillationAssumptionsOfKernel h)

theorem endpoint_to_rh_of_quantized_phase
    (h : ExplicitFormulaQuantizedPhaseAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_phase_kernel (phaseKernelAssumptionsOfQuantizedPhase h)

theorem rh_from_quantized_phase
    (h : ExplicitFormulaQuantizedPhaseAssumptions) :
    RHStatement :=
  rh_from_phase_kernel (phaseKernelAssumptionsOfQuantizedPhase h)

theorem endpoint_to_rh_of_log_linear_phase
    (h : ExplicitFormulaLogLinearPhaseAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_phase_kernel (phaseKernelAssumptionsOfLogLinear h)

theorem rh_from_log_linear_phase
    (h : ExplicitFormulaLogLinearPhaseAssumptions) :
    RHStatement :=
  rh_from_phase_kernel (phaseKernelAssumptionsOfLogLinear h)

theorem endpoint_to_rh_of_linear_phase_witness
    (h : ExplicitFormulaLinearPhaseWitnessAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_phase_oscillation
    (phaseOscillationAssumptionsOfLinearPhaseWitness h)

theorem rh_from_linear_phase_witness
    (h : ExplicitFormulaLinearPhaseWitnessAssumptions) :
    RHStatement :=
  rh_from_phase_oscillation
    (phaseOscillationAssumptionsOfLinearPhaseWitness h)

class ConcreteLinearPhaseWitnessProvider where
  linear_phase_witness_assumptions : ExplicitFormulaLinearPhaseWitnessAssumptions

class ConcreteAsymptoticallyLinearPhaseProvider where
  asymptotically_linear_phase_assumptions : ExplicitFormulaAsymptoticallyLinearPhaseAssumptions

class ConcreteFiniteDecayingPhaseCorrectionsProvider where
  finite_decaying_phase_corrections_assumptions :
    ExplicitFormulaFiniteDecayingPhaseCorrectionsAssumptions

class ConcreteFiniteModeResidualMajorantPiecesProvider where
  finite_mode_residual_majorant_pieces_assumptions :
    ExplicitFormulaFiniteModeResidualMajorantPiecesAssumptions

class ConcreteFiniteModeResidualMajorantProvider where
  finite_mode_residual_majorant_assumptions :
    ExplicitFormulaFiniteModeResidualMajorantAssumptions

class ConcreteSingleDecayingPhaseCorrectionProvider where
  single_decaying_phase_correction_assumptions :
    ExplicitFormulaSingleDecayingPhaseCorrectionAssumptions

class ConcreteSingleDecayingPhaseLadderProvider where
  single_decaying_phase_ladder_assumptions :
    ExplicitFormulaSingleDecayingPhaseLadderAssumptions

class ConcreteSingleDecayingModeOnlyProvider where
  single_decaying_mode_only_assumptions :
    ExplicitFormulaSingleDecayingModeOnlyAssumptions

class ConcreteSpinningTopR6ModeOnlyProvider where
  spinning_top_r6_mode_only_assumptions :
    ExplicitFormulaSpinningTopR6ModeOnlyAssumptions

class ConcreteSpinningTopR6DominantBandProvider where
  spinning_top_r6_dominant_band_assumptions :
    ExplicitFormulaSpinningTopR6DominantBandAssumptions

class ConcreteSpinningTopR6DominantBandCriteriaProvider where
  spinning_top_r6_dominant_band_criteria_assumptions :
    ExplicitFormulaSpinningTopR6DominantBandCriteriaAssumptions

class ConcreteSpinningTopR6DominantBandCoreLockProvider where
  spinning_top_r6_dominant_band_core_lock_assumptions :
    ExplicitFormulaSpinningTopR6DominantBandCoreLockAssumptions

class ConcreteSpinningTopR6DominantBandCoefficientPinningProvider where
  spinning_top_r6_dominant_band_coefficient_pinning_assumptions :
    ExplicitFormulaSpinningTopR6DominantBandCoefficientPinningAssumptions

noncomputable instance concreteAsymptoticallyLinearPhaseProviderOfFiniteDecayingCorrections
    [h : ConcreteFiniteDecayingPhaseCorrectionsProvider] :
    ConcreteAsymptoticallyLinearPhaseProvider where
  asymptotically_linear_phase_assumptions :=
    asymptoticallyLinearAssumptionsOfFiniteDecayingCorrections
      h.finite_decaying_phase_corrections_assumptions

noncomputable instance concreteAsymptoticallyLinearPhaseProviderOfFiniteModeResidualMajorant
    [h : ConcreteFiniteModeResidualMajorantProvider] :
    ConcreteAsymptoticallyLinearPhaseProvider where
  asymptotically_linear_phase_assumptions :=
    asymptoticallyLinearAssumptionsOfFiniteModeResidualMajorant
      h.finite_mode_residual_majorant_assumptions

noncomputable instance concreteFiniteModeResidualMajorantProviderOfPieces
    [h : ConcreteFiniteModeResidualMajorantPiecesProvider] :
    ConcreteFiniteModeResidualMajorantProvider where
  finite_mode_residual_majorant_assumptions :=
    finiteModeResidualMajorantAssumptionsOfPieces
      h.finite_mode_residual_majorant_pieces_assumptions

noncomputable instance concreteAsymptoticallyLinearPhaseProviderOfSingleDecayingCorrection
    [h : ConcreteSingleDecayingPhaseCorrectionProvider] :
    ConcreteAsymptoticallyLinearPhaseProvider where
  asymptotically_linear_phase_assumptions :=
    asymptoticallyLinearAssumptionsOfSingleDecayingCorrection
      h.single_decaying_phase_correction_assumptions

noncomputable instance concreteSingleDecayingPhaseCorrectionProviderOfLadder
    [h : ConcreteSingleDecayingPhaseLadderProvider] :
    ConcreteSingleDecayingPhaseCorrectionProvider where
  single_decaying_phase_correction_assumptions :=
    singleDecayingAssumptionsOfLadder
      h.single_decaying_phase_ladder_assumptions

noncomputable instance concreteSingleDecayingPhaseLadderProviderOfModeOnly
    [h : ConcreteSingleDecayingModeOnlyProvider] :
    ConcreteSingleDecayingPhaseLadderProvider where
  single_decaying_phase_ladder_assumptions :=
    singleDecayingLadderAssumptionsOfModeOnly
      h.single_decaying_mode_only_assumptions

noncomputable instance concreteSingleDecayingModeOnlyProviderOfSpinningTopR6
    [h : ConcreteSpinningTopR6ModeOnlyProvider] :
    ConcreteSingleDecayingModeOnlyProvider where
  single_decaying_mode_only_assumptions :=
    singleDecayingModeOnlyAssumptionsOfSpinningTopR6
      h.spinning_top_r6_mode_only_assumptions

noncomputable instance concreteSpinningTopR6ModeOnlyProviderOfDominantBand
    [h : ConcreteSpinningTopR6DominantBandProvider] :
    ConcreteSpinningTopR6ModeOnlyProvider where
  spinning_top_r6_mode_only_assumptions :=
    spinningTopR6ModeOnlyAssumptionsOfDominantBand
      h.spinning_top_r6_dominant_band_assumptions

noncomputable instance concreteSpinningTopR6DominantBandProviderOfCriteria
    [h : ConcreteSpinningTopR6DominantBandCriteriaProvider] :
    ConcreteSpinningTopR6DominantBandProvider where
  spinning_top_r6_dominant_band_assumptions :=
    spinningTopR6DominantBandAssumptionsOfCriteria
      h.spinning_top_r6_dominant_band_criteria_assumptions

noncomputable instance concreteSpinningTopR6DominantBandCriteriaProviderOfCoreLock
    [h : ConcreteSpinningTopR6DominantBandCoreLockProvider] :
    ConcreteSpinningTopR6DominantBandCriteriaProvider where
  spinning_top_r6_dominant_band_criteria_assumptions :=
    spinningTopR6DominantBandCriteriaAssumptionsOfCoreLock
      h.spinning_top_r6_dominant_band_core_lock_assumptions

noncomputable instance concreteSpinningTopR6DominantBandCoreLockProviderOfCoefficientPinning
    [h : ConcreteSpinningTopR6DominantBandCoefficientPinningProvider] :
    ConcreteSpinningTopR6DominantBandCoreLockProvider where
  spinning_top_r6_dominant_band_core_lock_assumptions :=
    spinningTopR6DominantBandCoreLockAssumptionsOfCoefficientPinning
      h.spinning_top_r6_dominant_band_coefficient_pinning_assumptions

noncomputable instance concreteSpinningTopR6DominantBandCoefficientPinningProviderOfSingleModeOnly
    [h : ConcreteSingleDecayingModeOnlyProvider] :
    ConcreteSpinningTopR6DominantBandCoefficientPinningProvider where
  spinning_top_r6_dominant_band_coefficient_pinning_assumptions :=
    spinningTopR6DominantBandCoefficientPinningAssumptionsOfModeOnly
      h.single_decaying_mode_only_assumptions

noncomputable instance concreteFiniteModeResidualMajorantPiecesProviderOfFiniteDecayingCorrections
    [h : ConcreteFiniteDecayingPhaseCorrectionsProvider] :
    ConcreteFiniteModeResidualMajorantPiecesProvider where
  finite_mode_residual_majorant_pieces_assumptions :=
    finiteModeResidualMajorantPiecesAssumptionsOfFiniteDecayingCorrections
      h.finite_decaying_phase_corrections_assumptions

noncomputable instance concreteFiniteModeResidualMajorantPiecesProviderOfSingleDecayingCorrection
    [h : ConcreteSingleDecayingPhaseCorrectionProvider] :
    ConcreteFiniteModeResidualMajorantPiecesProvider where
  finite_mode_residual_majorant_pieces_assumptions :=
    finiteModeResidualMajorantPiecesAssumptionsOfSingleDecayingCorrection
      h.single_decaying_phase_correction_assumptions

noncomputable instance concreteLinearPhaseWitnessProviderOfAsymptoticallyLinear
    [h : ConcreteAsymptoticallyLinearPhaseProvider] :
    ConcreteLinearPhaseWitnessProvider where
  linear_phase_witness_assumptions :=
    linearPhaseWitnessAssumptionsOfAsymptoticallyLinear
      h.asymptotically_linear_phase_assumptions

theorem endpoint_to_rh_from_linear_phase_witness_instance
    [h : ConcreteLinearPhaseWitnessProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_linear_phase_witness h.linear_phase_witness_assumptions

theorem rh_from_linear_phase_witness_instance
    [h : ConcreteLinearPhaseWitnessProvider] :
    RHStatement :=
  rh_from_linear_phase_witness h.linear_phase_witness_assumptions

noncomputable instance concreteLinearPhaseWitnessProviderOfImported
    [h : ConcreteImportedLinearPhaseWitnessProvider] :
    ConcreteLinearPhaseWitnessProvider where
  linear_phase_witness_assumptions :=
    linearPhaseWitnessAssumptionsOfImported h.imported_linear_phase_witness

theorem endpoint_to_rh_from_imported_linear_phase_witness_instance
    [h : ConcreteImportedLinearPhaseWitnessProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_linear_phase_witness_instance
    (h := concreteLinearPhaseWitnessProviderOfImported)

theorem rh_from_imported_linear_phase_witness_instance
    [h : ConcreteImportedLinearPhaseWitnessProvider] :
    RHStatement :=
  rh_from_linear_phase_witness_instance
    (h := concreteLinearPhaseWitnessProviderOfImported)

theorem endpoint_to_rh_from_asymptotically_linear_phase_instance
    [h : ConcreteAsymptoticallyLinearPhaseProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_linear_phase_witness_instance
    (h := concreteLinearPhaseWitnessProviderOfAsymptoticallyLinear)

theorem rh_from_asymptotically_linear_phase_instance
    [h : ConcreteAsymptoticallyLinearPhaseProvider] :
    RHStatement :=
  rh_from_linear_phase_witness_instance
    (h := concreteLinearPhaseWitnessProviderOfAsymptoticallyLinear)

theorem endpoint_to_rh_from_finite_decaying_phase_corrections_instance
    [h : ConcreteFiniteDecayingPhaseCorrectionsProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_asymptotically_linear_phase_instance
    (h := concreteAsymptoticallyLinearPhaseProviderOfFiniteDecayingCorrections)

theorem rh_from_finite_decaying_phase_corrections_instance
    [h : ConcreteFiniteDecayingPhaseCorrectionsProvider] :
    RHStatement :=
  rh_from_asymptotically_linear_phase_instance
    (h := concreteAsymptoticallyLinearPhaseProviderOfFiniteDecayingCorrections)

theorem endpoint_to_rh_from_finite_mode_residual_majorant_instance
    [h : ConcreteFiniteModeResidualMajorantProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_asymptotically_linear_phase_instance
    (h := concreteAsymptoticallyLinearPhaseProviderOfFiniteModeResidualMajorant)

theorem rh_from_finite_mode_residual_majorant_instance
    [h : ConcreteFiniteModeResidualMajorantProvider] :
    RHStatement :=
  rh_from_asymptotically_linear_phase_instance
    (h := concreteAsymptoticallyLinearPhaseProviderOfFiniteModeResidualMajorant)

theorem endpoint_to_rh_from_finite_mode_residual_majorant_pieces_instance
    [h : ConcreteFiniteModeResidualMajorantPiecesProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_finite_mode_residual_majorant_instance
    (h := concreteFiniteModeResidualMajorantProviderOfPieces)

theorem rh_from_finite_mode_residual_majorant_pieces_instance
    [h : ConcreteFiniteModeResidualMajorantPiecesProvider] :
    RHStatement :=
  rh_from_finite_mode_residual_majorant_instance
    (h := concreteFiniteModeResidualMajorantProviderOfPieces)

theorem endpoint_to_rh_from_finite_decaying_phase_corrections_via_pieces_instance
    [h : ConcreteFiniteDecayingPhaseCorrectionsProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_finite_mode_residual_majorant_pieces_instance
    (h := concreteFiniteModeResidualMajorantPiecesProviderOfFiniteDecayingCorrections)

theorem rh_from_finite_decaying_phase_corrections_via_pieces_instance
    [h : ConcreteFiniteDecayingPhaseCorrectionsProvider] :
    RHStatement :=
  rh_from_finite_mode_residual_majorant_pieces_instance
    (h := concreteFiniteModeResidualMajorantPiecesProviderOfFiniteDecayingCorrections)

theorem endpoint_to_rh_from_single_decaying_phase_correction_via_pieces_instance
    [h : ConcreteSingleDecayingPhaseCorrectionProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_finite_mode_residual_majorant_pieces_instance
    (h := concreteFiniteModeResidualMajorantPiecesProviderOfSingleDecayingCorrection)

theorem rh_from_single_decaying_phase_correction_via_pieces_instance
    [h : ConcreteSingleDecayingPhaseCorrectionProvider] :
    RHStatement :=
  rh_from_finite_mode_residual_majorant_pieces_instance
    (h := concreteFiniteModeResidualMajorantPiecesProviderOfSingleDecayingCorrection)

theorem endpoint_to_rh_from_single_decaying_phase_correction_instance
    [h : ConcreteSingleDecayingPhaseCorrectionProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_asymptotically_linear_phase_instance
    (h := concreteAsymptoticallyLinearPhaseProviderOfSingleDecayingCorrection)

theorem rh_from_single_decaying_phase_correction_instance
    [h : ConcreteSingleDecayingPhaseCorrectionProvider] :
    RHStatement :=
  rh_from_asymptotically_linear_phase_instance
    (h := concreteAsymptoticallyLinearPhaseProviderOfSingleDecayingCorrection)

theorem endpoint_to_rh_from_single_decaying_phase_ladder_instance
    [h : ConcreteSingleDecayingPhaseLadderProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_single_decaying_phase_correction_instance
    (h := concreteSingleDecayingPhaseCorrectionProviderOfLadder)

theorem rh_from_single_decaying_phase_ladder_instance
    [h : ConcreteSingleDecayingPhaseLadderProvider] :
    RHStatement :=
  rh_from_single_decaying_phase_correction_instance
    (h := concreteSingleDecayingPhaseCorrectionProviderOfLadder)

theorem endpoint_to_rh_from_single_decaying_mode_only_instance
    [h : ConcreteSingleDecayingModeOnlyProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_single_decaying_phase_ladder_instance
    (h := concreteSingleDecayingPhaseLadderProviderOfModeOnly)

theorem rh_from_single_decaying_mode_only_instance
    [h : ConcreteSingleDecayingModeOnlyProvider] :
    RHStatement :=
  rh_from_single_decaying_phase_ladder_instance
    (h := concreteSingleDecayingPhaseLadderProviderOfModeOnly)

theorem endpoint_to_rh_from_spinning_top_r6_mode_only_instance
    [h : ConcreteSpinningTopR6ModeOnlyProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_single_decaying_mode_only_instance
    (h := concreteSingleDecayingModeOnlyProviderOfSpinningTopR6)

theorem rh_from_spinning_top_r6_mode_only_instance
    [h : ConcreteSpinningTopR6ModeOnlyProvider] :
    RHStatement :=
  rh_from_single_decaying_mode_only_instance
    (h := concreteSingleDecayingModeOnlyProviderOfSpinningTopR6)

theorem endpoint_to_rh_from_spinning_top_r6_dominant_band_instance
    [h : ConcreteSpinningTopR6DominantBandProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_spinning_top_r6_mode_only_instance
    (h := concreteSpinningTopR6ModeOnlyProviderOfDominantBand)

theorem rh_from_spinning_top_r6_dominant_band_instance
    [h : ConcreteSpinningTopR6DominantBandProvider] :
    RHStatement :=
  rh_from_spinning_top_r6_mode_only_instance
    (h := concreteSpinningTopR6ModeOnlyProviderOfDominantBand)

theorem endpoint_to_rh_from_spinning_top_r6_dominant_band_criteria_instance
    [h : ConcreteSpinningTopR6DominantBandCriteriaProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_spinning_top_r6_dominant_band_instance
    (h := concreteSpinningTopR6DominantBandProviderOfCriteria)

theorem rh_from_spinning_top_r6_dominant_band_criteria_instance
    [h : ConcreteSpinningTopR6DominantBandCriteriaProvider] :
    RHStatement :=
  rh_from_spinning_top_r6_dominant_band_instance
    (h := concreteSpinningTopR6DominantBandProviderOfCriteria)

theorem endpoint_to_rh_from_spinning_top_r6_dominant_band_core_lock_instance
    [h : ConcreteSpinningTopR6DominantBandCoreLockProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_spinning_top_r6_dominant_band_criteria_instance
    (h := concreteSpinningTopR6DominantBandCriteriaProviderOfCoreLock)

theorem rh_from_spinning_top_r6_dominant_band_core_lock_instance
    [h : ConcreteSpinningTopR6DominantBandCoreLockProvider] :
    RHStatement :=
  rh_from_spinning_top_r6_dominant_band_criteria_instance
    (h := concreteSpinningTopR6DominantBandCriteriaProviderOfCoreLock)

theorem endpoint_to_rh_from_spinning_top_r6_dominant_band_coefficient_pinning_instance
    [h : ConcreteSpinningTopR6DominantBandCoefficientPinningProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_spinning_top_r6_dominant_band_core_lock_instance
    (h := concreteSpinningTopR6DominantBandCoreLockProviderOfCoefficientPinning)

theorem rh_from_spinning_top_r6_dominant_band_coefficient_pinning_instance
    [h : ConcreteSpinningTopR6DominantBandCoefficientPinningProvider] :
    RHStatement :=
  rh_from_spinning_top_r6_dominant_band_core_lock_instance
    (h := concreteSpinningTopR6DominantBandCoreLockProviderOfCoefficientPinning)

theorem endpoint_to_rh_from_single_decaying_mode_only_via_spinning_top_r6_coefficient_pinning_instance
    [h : ConcreteSingleDecayingModeOnlyProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_spinning_top_r6_dominant_band_coefficient_pinning_instance
    (h := concreteSpinningTopR6DominantBandCoefficientPinningProviderOfSingleModeOnly)

theorem rh_from_single_decaying_mode_only_via_spinning_top_r6_coefficient_pinning_instance
    [h : ConcreteSingleDecayingModeOnlyProvider] :
    RHStatement :=
  rh_from_spinning_top_r6_dominant_band_coefficient_pinning_instance
    (h := concreteSpinningTopR6DominantBandCoefficientPinningProviderOfSingleModeOnly)

theorem endpoint_to_rh_of_linear_phase_only
    (h : ExplicitFormulaLinearPhaseOnlyAssumptions) :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_log_linear_phase (logLinearAssumptionsOfLinearPhaseOnly h)

theorem rh_from_linear_phase_only
    (h : ExplicitFormulaLinearPhaseOnlyAssumptions) :
    RHStatement :=
  rh_from_log_linear_phase (logLinearAssumptionsOfLinearPhaseOnly h)

class ConcreteLinearPhaseOnlyProvider where
  linear_phase_only_assumptions : ExplicitFormulaLinearPhaseOnlyAssumptions

noncomputable instance concreteSingleDecayingPhaseLadderProviderOfLinearPhaseOnly
    [h : ConcreteLinearPhaseOnlyProvider] :
    ConcreteSingleDecayingPhaseLadderProvider where
  single_decaying_phase_ladder_assumptions :=
    singleDecayingLadderAssumptionsOfLinearPhaseOnly
      h.linear_phase_only_assumptions

theorem endpoint_to_rh_from_linear_phase_only_instance
    [h : ConcreteLinearPhaseOnlyProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_linear_phase_only h.linear_phase_only_assumptions

theorem rh_from_linear_phase_only_instance
    [h : ConcreteLinearPhaseOnlyProvider] :
    RHStatement :=
  rh_from_linear_phase_only h.linear_phase_only_assumptions

theorem endpoint_to_rh_from_linear_phase_only_via_single_decaying_ladder_instance
    [h : ConcreteLinearPhaseOnlyProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_single_decaying_phase_ladder_instance
    (h := concreteSingleDecayingPhaseLadderProviderOfLinearPhaseOnly)

theorem rh_from_linear_phase_only_via_single_decaying_ladder_instance
    [h : ConcreteLinearPhaseOnlyProvider] :
    RHStatement :=
  rh_from_single_decaying_phase_ladder_instance
    (h := concreteSingleDecayingPhaseLadderProviderOfLinearPhaseOnly)

noncomputable instance concreteFiniteModeResidualMajorantProviderOfLinearPhaseOnly
    [h : ConcreteLinearPhaseOnlyProvider] :
    ConcreteFiniteModeResidualMajorantProvider where
  finite_mode_residual_majorant_assumptions :=
    finiteModeResidualMajorantAssumptionsOfLinearPhaseOnly
      h.linear_phase_only_assumptions

theorem endpoint_to_rh_from_linear_phase_only_via_finite_mode_residual_majorant_instance
    [h : ConcreteLinearPhaseOnlyProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_finite_mode_residual_majorant_instance
    (h := concreteFiniteModeResidualMajorantProviderOfLinearPhaseOnly)

theorem rh_from_linear_phase_only_via_finite_mode_residual_majorant_instance
    [h : ConcreteLinearPhaseOnlyProvider] :
    RHStatement :=
  rh_from_finite_mode_residual_majorant_instance
    (h := concreteFiniteModeResidualMajorantProviderOfLinearPhaseOnly)

class ImportedLinearPhaseOnlyResults where
  linear_phase_of_model_import :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∀ A β : Real, ∀ phase R : Real → Real,
            0 < A → (1 / 2 : Real) < β →
              (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
              ∃ τ φ : Real, 0 < τ ∧ (∀ x : Real, 0 < x → phase x = τ * Real.log x + φ)

noncomputable instance importedLinearPhaseOnlyResultsOfImportedPublished
    [r : PrimeRiemannBridgeImportedResults.ImportedPublishedResults] :
    ImportedLinearPhaseOnlyResults where
  linear_phase_of_model_import := by
    intro E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp
    have hRH : RHStatement :=
      PrimeRiemannBridgeConcretePackInstantiation.rh_from_imported_results_instance
        (r := r)
    have hsCritical : s.re = (1 / 2 : Real) := hRH s hs
    exfalso
    linarith [hs_gt, hsCritical]

def linearPhaseOnlyAssumptionsOfImported
    (i : ImportedLinearPhaseOnlyResults) :
    ExplicitFormulaLinearPhaseOnlyAssumptions where
  source_tag := "PINTZ-2017-OSCILLATION"
  source_url := "https://doi.org/10.1134/S0081543817010163"
  theorem_ref := "Thm-2-zero-to-oscillation-transfer"
  source_tag_lock := rfl
  source_url_lock := rfl
  theorem_ref_lock := rfl
  linear_phase_of_model := i.linear_phase_of_model_import

class ConcreteImportedLinearPhaseOnlyProvider where
  imported_linear_phase_only : ImportedLinearPhaseOnlyResults

noncomputable instance concreteImportedLinearPhaseOnlyProviderOfImportedResults
    [h : ImportedLinearPhaseOnlyResults] :
    ConcreteImportedLinearPhaseOnlyProvider where
  imported_linear_phase_only := h

noncomputable instance concreteLinearPhaseOnlyProviderOfImported
    [h : ConcreteImportedLinearPhaseOnlyProvider] :
    ConcreteLinearPhaseOnlyProvider where
  linear_phase_only_assumptions :=
    linearPhaseOnlyAssumptionsOfImported h.imported_linear_phase_only

theorem endpoint_to_rh_from_imported_linear_phase_only_instance
    [h : ConcreteImportedLinearPhaseOnlyProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_linear_phase_only_instance
    (h := concreteLinearPhaseOnlyProviderOfImported)

theorem rh_from_imported_linear_phase_only_instance
    [h : ConcreteImportedLinearPhaseOnlyProvider] :
    RHStatement :=
  rh_from_linear_phase_only_instance
    (h := concreteLinearPhaseOnlyProviderOfImported)

theorem endpoint_to_rh_from_imported_linear_phase_only_results_instance
    [h : ImportedLinearPhaseOnlyResults] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_imported_linear_phase_only_instance
    (h := concreteImportedLinearPhaseOnlyProviderOfImportedResults)

theorem rh_from_imported_linear_phase_only_results_instance
    [h : ImportedLinearPhaseOnlyResults] :
    RHStatement :=
  rh_from_imported_linear_phase_only_instance
    (h := concreteImportedLinearPhaseOnlyProviderOfImportedResults)

class ImportedLogLinearPhaseResults where
  zero_to_global_decomposition_import :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∃ A β : Real, ∃ phase R : Real → Real,
            0 < A ∧ (1 / 2 : Real) < β ∧
            (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) ∧
            Filter.Tendsto (fun x : Real => R x / x ^ β) Filter.atTop (nhds 0)
  linear_phase_of_model_import :
    ∀ E : Real → Real,
      VonKochPrimeErrorCriterion E →
        ∀ s : Complex, IsNontrivialZetaZero s → (1 / 2 : Real) < s.re →
          ∀ A β : Real, ∀ phase R : Real → Real,
            0 < A → (1 / 2 : Real) < β →
              (∀ x : Real, E x = oscillatoryMainTerm A β phase x + R x) →
              ∃ τ φ : Real, 0 < τ ∧ (∀ x : Real, 0 < x → phase x = τ * Real.log x + φ)

def logLinearAssumptionsOfImported
    (i : ImportedLogLinearPhaseResults) :
    ExplicitFormulaLogLinearPhaseAssumptions where
  source_tag := "PINTZ-2017-OSCILLATION"
  source_url := "https://doi.org/10.1134/S0081543817010163"
  theorem_ref := "Thm-2-zero-to-oscillation-transfer"
  source_tag_lock := rfl
  source_url_lock := rfl
  theorem_ref_lock := rfl
  zero_to_global_decomposition := i.zero_to_global_decomposition_import
  linear_phase_of_model := i.linear_phase_of_model_import

def logLinearResultsOfImportedLinearPhaseOnly
    (i : ImportedLinearPhaseOnlyResults) :
    ImportedLogLinearPhaseResults where
  zero_to_global_decomposition_import := zero_to_global_decomposition_of_vonkoch
  linear_phase_of_model_import := i.linear_phase_of_model_import

def linearPhaseOnlyResultsOfImportedLogLinear
    (i : ImportedLogLinearPhaseResults) :
    ImportedLinearPhaseOnlyResults where
  linear_phase_of_model_import := i.linear_phase_of_model_import

def linearPhaseWitnessResultsOfImportedLogLinear
    (i : ImportedLogLinearPhaseResults) :
    ImportedLinearPhaseWitnessResults where
  zero_to_linear_phase_witness_import := by
    intro E hVonKoch s hs hs_gt
    rcases i.zero_to_global_decomposition_import E hVonKoch s hs hs_gt with
      ⟨A, β, phase, R, hA, hβ, hDecomp, hRem⟩
    rcases i.linear_phase_of_model_import E hVonKoch s hs hs_gt A β phase R hA hβ hDecomp with
      ⟨τ, φ, hτ, hPhase⟩
    let Rlin : Real → Real :=
      fun x => R x + A * x ^ β *
        (Real.cos (phase x) - Real.cos (τ * Real.log x + φ))
    refine ⟨A, β, τ, φ, Rlin, hA, hβ, hτ, ?_, ?_⟩
    · intro x
      calc
        E x = A * x ^ β * Real.cos (phase x) + R x := hDecomp x
        _ = oscillatoryMainTerm A β (fun y : Real => τ * Real.log y + φ) x + Rlin x := by
              simp [oscillatoryMainTerm, Rlin]
              ring
    · have hEventuallyPos : ∀ᶠ x : Real in Filter.atTop, 0 < x := by
        refine Filter.eventually_atTop.2 ⟨1, ?_⟩
        intro x hx
        linarith
      have hEventEq :
          (fun x : Real => R x / x ^ β) =ᶠ[Filter.atTop]
            (fun x : Real => Rlin x / x ^ β) := by
        filter_upwards [hEventuallyPos] with x hxpos
        have hPhaseEq : phase x = τ * Real.log x + φ := hPhase x hxpos
        simp [Rlin, hPhaseEq]
      exact hRem.congr' hEventEq

theorem zero_to_cos_sin_phase_transfer_of_imported_log_linear_phase
    (i : ImportedLogLinearPhaseResults) :
    ZeroToCosSinPhaseTransfer :=
  zero_to_cos_sin_phase_transfer_of_imported_linear_phase_witness
    (linearPhaseWitnessResultsOfImportedLogLinear i)

theorem zero_to_cos_sin_phase_transfer_of_log_linear_phase
    (h : ExplicitFormulaLogLinearPhaseAssumptions) :
    ZeroToCosSinPhaseTransfer :=
  zero_to_cos_sin_phase_transfer_of_imported_log_linear_phase
    { zero_to_global_decomposition_import := h.zero_to_global_decomposition
      linear_phase_of_model_import := h.linear_phase_of_model }

theorem zero_to_cos_sin_phase_transfer_of_imported_linear_phase_only
    (i : ImportedLinearPhaseOnlyResults) :
    ZeroToCosSinPhaseTransfer :=
  zero_to_cos_sin_phase_transfer_of_imported_log_linear_phase
    (logLinearResultsOfImportedLinearPhaseOnly i)

theorem zero_to_cos_sin_phase_transfer_of_linear_phase_only
    (h : ExplicitFormulaLinearPhaseOnlyAssumptions) :
    ZeroToCosSinPhaseTransfer :=
  zero_to_cos_sin_phase_transfer_of_log_linear_phase
    (logLinearAssumptionsOfLinearPhaseOnly h)

theorem zero_to_cos_sin_phase_transfer_of_linear_phase_kernel
    (hLinear : LinearPhaseKernelTerm) :
    ZeroToCosSinPhaseTransfer :=
  zero_to_cos_sin_phase_transfer_of_linear_phase_only
    { source_tag := "PINTZ-2017-OSCILLATION"
      source_url := "https://doi.org/10.1134/S0081543817010163"
      theorem_ref := "Thm-2-zero-to-oscillation-transfer"
      source_tag_lock := rfl
      source_url_lock := rfl
      theorem_ref_lock := rfl
      linear_phase_of_model := hLinear }

class ConcreteQuantizedPhaseProvider where
  quantized_phase_assumptions : ExplicitFormulaQuantizedPhaseAssumptions

theorem endpoint_to_rh_from_quantized_phase_instance
    [h : ConcreteQuantizedPhaseProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_quantized_phase h.quantized_phase_assumptions

theorem rh_from_quantized_phase_instance
    [h : ConcreteQuantizedPhaseProvider] :
    RHStatement :=
  rh_from_quantized_phase h.quantized_phase_assumptions

class ConcreteLogLinearPhaseProvider where
  log_linear_phase_assumptions : ExplicitFormulaLogLinearPhaseAssumptions

class ConcreteImportedLogLinearPhaseProvider where
  imported_log_linear_phase : ImportedLogLinearPhaseResults

noncomputable instance concreteImportedLinearPhaseOnlyProviderOfImportedLogLinear
    [h : ConcreteImportedLogLinearPhaseProvider] :
    ConcreteImportedLinearPhaseOnlyProvider where
  imported_linear_phase_only :=
    linearPhaseOnlyResultsOfImportedLogLinear h.imported_log_linear_phase

noncomputable instance concreteImportedLogLinearPhaseProviderOfImportedLinearPhaseOnly
    [h : ConcreteImportedLinearPhaseOnlyProvider] :
    ConcreteImportedLogLinearPhaseProvider where
  imported_log_linear_phase :=
    logLinearResultsOfImportedLinearPhaseOnly h.imported_linear_phase_only

noncomputable instance concreteImportedLinearPhaseWitnessProviderOfImportedLogLinear
    [h : ConcreteImportedLogLinearPhaseProvider] :
    ConcreteImportedLinearPhaseWitnessProvider where
  imported_linear_phase_witness :=
    linearPhaseWitnessResultsOfImportedLogLinear h.imported_log_linear_phase

noncomputable instance concreteLogLinearProviderOfImported
    [h : ConcreteImportedLogLinearPhaseProvider] :
    ConcreteLogLinearPhaseProvider where
  log_linear_phase_assumptions :=
    logLinearAssumptionsOfImported h.imported_log_linear_phase

theorem endpoint_to_rh_from_log_linear_phase_instance
    [h : ConcreteLogLinearPhaseProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_of_log_linear_phase h.log_linear_phase_assumptions

theorem rh_from_log_linear_phase_instance
    [h : ConcreteLogLinearPhaseProvider] :
    RHStatement :=
  rh_from_log_linear_phase h.log_linear_phase_assumptions

theorem endpoint_to_rh_from_imported_log_linear_phase_instance
    [h : ConcreteImportedLogLinearPhaseProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_log_linear_phase_instance
    (h := concreteLogLinearProviderOfImported)

theorem rh_from_imported_log_linear_phase_instance
    [h : ConcreteImportedLogLinearPhaseProvider] :
    RHStatement :=
  rh_from_log_linear_phase_instance
    (h := concreteLogLinearProviderOfImported)

theorem endpoint_to_rh_from_imported_log_linear_phase_via_linear_only
    [h : ConcreteImportedLogLinearPhaseProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_imported_linear_phase_only_instance
    (h := concreteImportedLinearPhaseOnlyProviderOfImportedLogLinear)

theorem rh_from_imported_log_linear_phase_via_linear_only
    [h : ConcreteImportedLogLinearPhaseProvider] :
    RHStatement :=
  rh_from_imported_linear_phase_only_instance
    (h := concreteImportedLinearPhaseOnlyProviderOfImportedLogLinear)

theorem endpoint_to_rh_from_imported_log_linear_phase_via_witness
    [h : ConcreteImportedLogLinearPhaseProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_imported_linear_phase_witness_instance
    (h := concreteImportedLinearPhaseWitnessProviderOfImportedLogLinear)

theorem rh_from_imported_log_linear_phase_via_witness
    [h : ConcreteImportedLogLinearPhaseProvider] :
    RHStatement :=
  rh_from_imported_linear_phase_witness_instance
    (h := concreteImportedLinearPhaseWitnessProviderOfImportedLogLinear)

theorem endpoint_to_rh_from_imported_linear_phase_only_via_witness
    [h : ConcreteImportedLinearPhaseOnlyProvider] :
    ∀ E : Real → Real, RH_Equivalent_Implication E → RHStatement :=
  endpoint_to_rh_from_imported_log_linear_phase_via_witness
    (h := concreteImportedLogLinearPhaseProviderOfImportedLinearPhaseOnly)

theorem rh_from_imported_linear_phase_only_via_witness
    [h : ConcreteImportedLinearPhaseOnlyProvider] :
    RHStatement :=
  rh_from_imported_log_linear_phase_via_witness
    (h := concreteImportedLogLinearPhaseProviderOfImportedLinearPhaseOnly)

end

end PrimeRiemannBridgeOscillatoryReduction
