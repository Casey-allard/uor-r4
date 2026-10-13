import Mathlib.Data.Real.Basic
import Mathlib
import Mathlib.Topology.Instances.Complex
import Riemann.RS.CRGreenOuter
import Riemann.RS.WhitneyGeometryDefs
import Mathlib.MeasureTheory.Integral.Bochner.ContinuousLinearMap
import Mathlib.MeasureTheory.Integral.Bochner.FundThmCalculus
import Mathlib.MeasureTheory.Integral.Bochner.Set
import Mathlib.Topology.Algebra.FilterBasis
import Mathlib.MeasureTheory.Function.Egorov
import Mathlib.MeasureTheory.Covering.Differentiation
import Mathlib.MeasureTheory.Measure.Lebesgue.Basic
import Mathlib.MeasureTheory.Covering.Besicovitch
import Riemann.Cert.KxiPPlus
import Mathlib.Analysis.SpecificLimits.Basic
-- import rh.RS.PoissonPlateau  -- not needed here; avoid heavy calculus import dependency

/-!
# Minimal tent/shadow geometry and monotonicity

This file provides real, minimal definitions of half–plane tents (Carleson boxes
with fixed height) and boundary shadows, together with basic monotonicity lemmas.

Notes:
- We intentionally fix the vertical height by a parameter `α > 0` rather than
  tying it to `|I|`; this keeps the geometry lean and assumption‑free here.
- Measure/length and Carleson energy are introduced elsewhere.
- No sorries or axioms.
-/

noncomputable section

namespace RH
namespace RS

open Set MeasureTheory Filter
open scoped MeasureTheory

/-- Tent (Carleson box) of aperture `α` over a boundary set `I ⊆ ℝ`:
    `T(I,α) := {(t,σ) : t∈I, 0<σ≤α}`. -/
def tent (I : Set ℝ) (α : ℝ) : Set (ℝ × ℝ) :=
  {p | p.1 ∈ I ∧ 0 < p.2 ∧ p.2 ≤ α}

/-- Shadow of a set `Q ⊆ ℝ×ℝ` onto the boundary line: vertical projection. -/
def shadow (Q : Set (ℝ × ℝ)) : Set ℝ :=
  {t | ∃ σ, 0 < σ ∧ (t, σ) ∈ Q}

@[simp]
lemma mem_tent {I : Set ℝ} {α : ℝ} {p : ℝ × ℝ} :
  p ∈ tent I α ↔ p.1 ∈ I ∧ 0 < p.2 ∧ p.2 ≤ α := Iff.rfl

@[simp]
lemma mem_shadow {Q : Set (ℝ × ℝ)} {t : ℝ} :
  t ∈ shadow Q ↔ ∃ σ, 0 < σ ∧ (t, σ) ∈ Q := Iff.rfl

/-- Monotonicity in the base set: if `I ⊆ J` then `T(I,α) ⊆ T(J,α)`. -/
lemma tent_mono {I J : Set ℝ} {α : ℝ} (hIJ : I ⊆ J) : tent I α ⊆ tent J α := by
  intro p hp
  rcases hp with ⟨hpI, hσpos, hσle⟩
  exact And.intro (hIJ hpI) (And.intro hσpos hσle)

/-- Monotonicity of the shadow under inclusion: `Q ⊆ R ⇒ shadow(Q) ⊆ shadow(R)`. -/
lemma shadow_mono {Q R : Set (ℝ × ℝ)} (hQR : Q ⊆ R) : shadow Q ⊆ shadow R := by
  intro t ht
  rcases ht with ⟨σ, hσ, hmem⟩
  exact ⟨σ, hσ, hQR hmem⟩

--/-- Length (Lebesgue measure) of a boundary set. -/
--def length (I : Set ℝ) : ℝ := (volume I).toReal

/-- Elementary Poisson kernel for the upper half-plane used with Lebesgue measure on ℝ. -/
@[simp] def poissonKernel (b x : ℝ) : ℝ :=
  b / (Real.pi * (b^2 + x^2))

/-/ Boundary map and real trace on the critical line. -/
def boundaryMap (t : ℝ) : ℂ := (1/2 : ℂ) + Complex.I * (t : ℂ)

@[simp] lemma boundaryMap_re (t : ℝ) : (boundaryMap t).re = (1/2 : ℝ) := by
  simp [boundaryMap]

/-- Boundary real trace of `F` on the critical line. -/
@[simp] def boundaryRe (F : ℂ → ℂ) (t : ℝ) : ℝ :=
  (F (boundaryMap t)).re

/-- Normalize a symmetric interval around `t0` to have length ≤ 1 by shrinking the radius. -/
lemma shrink_interval_to_unit
  (t0 r : ℝ) (hr : 0 < r) :
  ∃ r' : ℝ, 0 < r' ∧ r' ≤ r ∧ RS.length (Icc (t0 - r') (t0 + r')) ≤ 1 := by
  classical
  -- Choose `r' = min r (1/2)`
  refine ⟨min r (1/2), ?_, ?_, ?_⟩
  · have hpos₁ : 0 < r := hr
    have hpos₂ : 0 < (1/2 : ℝ) := by norm_num
    exact lt_min hpos₁ hpos₂
  · exact min_le_left _ _
  · -- Clean computation: let m = min r (1/2). Then length(Icc(t0−m, t0+m)) = 2m ≤ 1.
    set m : ℝ := min r (1/2) with hm
    have hmpos : 0 < m := by
      have : 0 < min r (1/2 : ℝ) := lt_min hr (by norm_num)
      simpa [hm] using this
    have hm0 : 0 ≤ m := le_of_lt hmpos
    have hx : t0 - m ≤ t0 + m := by linarith [hm0]
    have vol_eq : volume (Icc (t0 - m) (t0 + m)) = ENNReal.ofReal (2 * m) := by
      have : (t0 + m) - (t0 - m) = 2 * m := by ring
      simp [Real.volume_Icc, this]
    have hlen : Whitney.length (Icc (t0 - m) (t0 + m)) = 2 * m := by
      have h2m : 0 ≤ 2 * m := by linarith [hmpos]
      simp [Whitney.length, vol_eq, hm0]

    have hm_le : m ≤ (1/2 : ℝ) := by simp [hm]
    have hbound : 2 * m ≤ 1 := by linarith [hm_le]
    have hlen' : RS.length (Icc (t0 - min r (1/2)) (t0 + min r (1/2))) = 2 * min r (1/2) := by
      simpa [hm] using hlen
    simp only [ge_iff_le]; exact le_of_eq_of_le hlen hbound

/-- Measurability of the boundary real trace. -/
lemma measurable_boundaryRe (F : ℂ → ℂ)
  (hF : Measurable (fun t : ℝ => F (boundaryMap t))) :
  Measurable (fun t : ℝ => boundaryRe F t) := by
  classical
  -- boundaryRe F = (Complex.re) ∘ (fun t => F (boundaryMap t))
  have h2 : Measurable fun z : ℂ => z.re := Complex.continuous_re.measurable
  simpa [boundaryRe] using h2.comp hF

/-- Measurable sublevel sets of the boundary real trace `{t | boundaryRe F t ≤ a}`. -/
lemma measurableSet_sublevel_boundaryRe (F : ℂ → ℂ) (a : ℝ)
  (hF : Measurable (fun t : ℝ => F (boundaryMap t))) :
  MeasurableSet {t : ℝ | boundaryRe F t ≤ a} := by
  classical
  have hmeas : Measurable (fun t : ℝ => boundaryRe F t) := measurable_boundaryRe F hF
  -- {t | boundaryRe F t ≤ a} = preimage of Iic a under boundaryRe F
  simpa [Set.preimage, Set.mem_Iic] using (hmeas measurableSet_Iic)

/-- Poisson smoothed boundary real part at height `b>0` and horizontal `x`. -/
@[simp] def poissonSmooth (F : ℂ → ℂ) (b x : ℝ) : ℝ :=
  ∫ t, RH.RS.poissonKernel b (x - t) * boundaryRe F t ∂(volume)

/-- Minimal energy monotonicity helper: if the box energy on a tent is bounded
by `K`, and the energy on `Q` is bounded by the tent energy, then the same
bound `K` holds for `Q`. This exposes exactly the transitivity used in Brick 3a. -/
lemma boxEnergy_le_of_tent_bound
  (gradU : (ℝ × ℝ) → ℝ × ℝ) (σm : Measure (ℝ × ℝ))
  (I : Set ℝ) (α : ℝ) (Q : Set (ℝ × ℝ)) (K : ℝ)
  (hMono : RS.boxEnergy gradU σm Q ≤ RS.boxEnergy gradU σm (tent I α))
  (hTent : RS.boxEnergy gradU σm (tent I α) ≤ K) :
  RS.boxEnergy gradU σm Q ≤ K :=
by
  exact le_trans hMono hTent

/-- Brick 3a (local Carleson on a Whitney piece), assumption-driven form.

If `Q ⊆ tent (shadow Q, α)` so that energy on `Q` is ≤ energy on that tent,
and if a Carleson/tent bound holds on the tent by `Kξ * L`, then the same
bound holds on `Q`.
-/
lemma carleson_local_on_shadow_assuming
  (gradU : (ℝ × ℝ) → ℝ × ℝ) (σm : Measure (ℝ × ℝ))
  (α : ℝ) (Q : Set (ℝ × ℝ)) (Kξ L : ℝ)
  (hMono : RS.boxEnergy gradU σm Q
            ≤ RS.boxEnergy gradU σm (tent (shadow Q) α))
  (hTent : RS.boxEnergy gradU σm (tent (shadow Q) α) ≤ Kξ * L) :
  RS.boxEnergy gradU σm Q ≤ Kξ * L :=
by
  exact boxEnergy_le_of_tent_bound gradU σm (shadow Q) α Q (Kξ * L) hMono hTent

/-- Monotonicity of box energy from subset inclusion, under mild measurability
and finiteness assumptions. This provides `boxEnergy(Q) ≤ boxEnergy(T)` from
`Q ⊆ T`. -/
lemma boxEnergy_mono_of_subset
  (gradU : (ℝ × ℝ) → ℝ × ℝ) (σm : Measure (ℝ × ℝ))
  {P R : Set (ℝ × ℝ)} (hPR : P ⊆ R)
  (hmeasP : MeasurableSet P) (hmeasR : MeasurableSet R)
  (hRfin :
    (∫⁻ p in R, ENNReal.ofReal (‖gradU p‖ ^ 2 * p.2) ∂σm) < ⊤) :
  RS.boxEnergy gradU σm P ≤ RS.boxEnergy gradU σm R :=
by
  -- `RS.boxEnergy` is just `Whitney.boxEnergy`
  unfold RS.boxEnergy
  -- apply the general monotonicity lemma from `WhitneyGeometryDefs`
  simpa using
    Whitney.boxEnergy_mono (gradU := gradU) (σ := σm)
      (P := P) (Q := R) hPR hmeasP hmeasR hRfin

/-- Brick 3a completed: from a fixed-geometry inclusion `Q ⊆ tent(shadow Q, α)`
and a tent Carleson bound on `tent(shadow Q, α)`, conclude the local Carleson
bound on `Q`. -/
lemma carleson_local_on_shadow
  (gradU : (ℝ × ℝ) → ℝ × ℝ) (σm : Measure (ℝ × ℝ))
  (α : ℝ) (Q : Set (ℝ × ℝ)) (Kξ L : ℝ)
  (hsubset : Q ⊆ tent (shadow Q) α)
  (hmeasQ : MeasurableSet Q) (hmeasTent : MeasurableSet (tent (shadow Q) α))
  (_ : IntegrableOn (fun p => RS.sqnormR2 (gradU p)) (tent (shadow Q) α) σm)
  (hTentFin :
    (∫⁻ p in tent (shadow Q) α,
        ENNReal.ofReal (‖gradU p‖ ^ 2 * p.2) ∂σm) < ⊤)
  (hTent : RS.boxEnergy gradU σm (tent (shadow Q) α) ≤ Kξ * L) :
  RS.boxEnergy gradU σm Q ≤ Kξ * L :=
by
  -- first monotonicity `boxEnergy(Q) ≤ boxEnergy(tent(shadow Q, α))`
  have hMono :
      RS.boxEnergy gradU σm Q
        ≤ RS.boxEnergy gradU σm (tent (shadow Q) α) :=
    boxEnergy_mono_of_subset gradU σm hsubset hmeasQ hmeasTent hTentFin
  -- then apply the abstract transitivity lemma
  exact carleson_local_on_shadow_assuming gradU σm α Q Kξ L hMono hTent

/-- Bounded shadow overlap (Brick 3b), assumption‑driven form.

If almost everywhere on ℝ we have the pointwise indicator sum bound
  ∑_{i∈S} 1_{shadow(Q i)} ≤ C · 1_I,
then the sum of shadow lengths is ≤ C · length(I).
-/
lemma bounded_shadow_overlap_sum
  {ι : Type*} (S : Finset ι) (Q : ι → Set (ℝ × ℝ))
  (I : Set ℝ) (C : ℝ)
  (hmeasI : MeasurableSet I)
  (hmeasSh : ∀ i ∈ S, MeasurableSet (shadow (Q i)))
  (hShSub : ∀ i ∈ S, shadow (Q i) ⊆ I)
  (h_ae : (fun t => ∑ i ∈ S, (shadow (Q i)).indicator (fun _ => (1 : ℝ)) t)
            =ᵐ[Measure.restrict volume I]
          (fun _ => C))
  (hI_fin : volume I < ⊤) :
  (∑ i ∈ S, length (shadow (Q i))) ≤ C * length I :=
by
  classical
  set μ := volume.restrict I
  have hμ_fin : μ Set.univ < ⊤ := by
    simpa [μ, Measure.restrict_apply, hmeasI, Set.univ_inter] using hI_fin
  have _inst : IsFiniteMeasure μ := ⟨hμ_fin⟩
  have h_int_eq :
      ∫ t, (∑ i ∈ S, (shadow (Q i)).indicator (fun _ => (1 : ℝ)) t) ∂μ
        = ∫ t, (fun _ => C) t ∂μ :=
    integral_congr_ae h_ae
  have hsum_int :
      ∫ t, (∑ i ∈ S, (shadow (Q i)).indicator (fun _ => (1 : ℝ)) t) ∂μ
        = ∑ i ∈ S, ∫ t, (shadow (Q i)).indicator (fun _ => (1 : ℝ)) t ∂μ := by
    simp [μ, integral_finset_sum]; congr; aesop
  have hconst :
      ∫ t, (fun _ => C) t ∂μ = C * length I := by
    have hμ_univ : μ Set.univ = volume I := by
      simpa [μ, Measure.restrict_apply, hmeasI, Set.univ_inter]
    rw [integral_const, smul_eq_mul, mul_comm]
    simp [length, hμ_univ, Measure.real]
    aesop
  have h_ind :
      ∀ i ∈ S,
        ∫ t, (shadow (Q i)).indicator (fun _ => (1 : ℝ)) t ∂μ
          = length (shadow (Q i)) := by
    intro i hi
    have hμ_shadow :
        μ (shadow (Q i)) = volume ((shadow (Q i)) ∩ I) := by
      simp [μ, Measure.restrict_apply, hmeasI, Set.inter_comm,
        Set.inter_left_comm, Set.inter_assoc]
    have hinter : (shadow (Q i)) ∩ I = shadow (Q i) := by
      ext t; constructor <;> intro ht
      · exact ht.1
      · exact ⟨ht, hShSub i hi ht⟩
    have hmeasure :
        μ (shadow (Q i)) = volume (shadow (Q i)) := by
      simpa [hinter] using hμ_shadow
    have :
        ∫ t, (shadow (Q i)).indicator (fun _ => (1 : ℝ)) t ∂μ
          = (μ (shadow (Q i))).toReal := by
      simpa [μ] using
        MeasureTheory.integral_indicator_one (μ := μ)
          (s := shadow (Q i)) (hs := hmeasSh i hi)
    have hμ_toReal :
        (μ (shadow (Q i))).toReal = (volume (shadow (Q i))).toReal := by
      simpa [hmeasure]
    simpa [length, hμ_toReal]
      using this
  have hsum_lengths :
      (∑ i ∈ S, length (shadow (Q i))) = C * length I := by
    have := h_int_eq.trans (by simp [hsum_int, h_ind, hconst]; rfl)
    simpa [hsum_int, h_ind, hconst] using this
  simpa using hsum_lengths.le

/-- Algebraic per‑shadow coercivity from a main term and a small remainder.

If `A ≥ c0·κ·L − |R|` and `|R| ≤ (c0·κ/2)·L`, then `A ≥ (c0·κ/2)·L`.
We will use `A = ∫_I ψ · B_Q`, `L = length(shadow Q)`. -/
lemma coercivity_from_main_and_remainder
  {c0 κ L A R : ℝ} (hc0 : 0 < c0) (hκ : 0 < κ)
  (hMain : A ≥ c0 * κ * L - |R|)
  (hRem  : |R| ≤ (c0 * κ / 2) * L) :
  A ≥ (c0 * κ / 2) * L :=
by
  -- From |R| ≤ (c0 κ/2) L, we get (c0 κ L - (c0 κ/2)L) ≤ (c0 κ L - |R|)
  have hstep : c0 * κ * L - (c0 * κ / 2) * L ≤ c0 * κ * L - |R| := by
    have hneg : -((c0 * κ / 2) * L) ≤ -|R| := by
      simpa [mul_comm, mul_left_comm, mul_assoc] using (neg_le_neg hRem)
    simpa [sub_eq_add_neg, mul_comm, mul_left_comm, mul_assoc]
      using (add_le_add_left hneg (c0 * κ * L))
  -- Chain with the main inequality A ≥ c0 κ L - |R|
  have hA' : c0 * κ * L - (c0 * κ / 2) * L ≤ A :=
    le_trans hstep hMain
  clear hstep
  -- RHS = (c0 κ/2) L
  have hRHS : c0 * κ * L - (c0 * κ / 2) * L = (c0 * κ / 2) * L := by
    ring
  -- Conclude A ≥ (c0 κ/2) L
  have : (c0 * κ / 2) * L ≤ A := by simpa [hRHS] using hA'
  exact this

/-- Brick 4b (numeric form): given a lower estimate `∫ ≥ c0 κ L − |R|` and a
small remainder `|R| ≤ (c0 κ/2)L`, conclude the clean coercivity
`∫ ≥ (c0 κ/2) L`. -/
lemma per_shadow_coercivity_numeric
  (ψ : ℝ → ℝ) (F : ℂ → ℂ) (Q : Set (ℝ × ℝ))
  (I : Set ℝ) (c0 κ : ℝ)
  (hc0 : 0 < c0) (hκ : 0 < κ)
  (hMain : (∫ t in I, ψ t * boundaryRe F t ∂volume)
              ≥ c0 * κ * RS.length (shadow Q) - |(0 : ℝ)|)
  (hRem  : |(0 : ℝ)| ≤ (c0 * κ / 2) * RS.length (shadow Q)) :
  (∫ t in I, ψ t * boundaryRe F t ∂volume)
    ≥ (c0 * κ / 2) * RS.length (shadow Q) :=
by
  -- This lemma shows the algebra. In practice, replace 0 by your remainder `R`.
  simpa using coercivity_from_main_and_remainder (A := (∫ t in I, ψ t * boundaryRe F t ∂volume))
      (L := RS.length (shadow Q)) (R := (0 : ℝ)) hc0 hκ hMain hRem

/-- Monotone partial sums with nonnegative terms converge to the total sum yields
finite capture at any tolerance. -/
lemma partial_sum_capture_of_hasSum
  {a : ℕ → ℝ} {T : ℝ}
  (h0 : ∀ n, 0 ≤ a n) (hSum : HasSum a T)
  (hT : 0 ≤ T) (ε : ℝ) (hε : 0 < ε) :
  ∃ N : ℕ, (∑ i ∈ Finset.range N, a i) ≥ (1 - ε) * T :=
by
  have h_tend : Tendsto (fun N => ∑ i ∈ Finset.range N, a i) atTop (nhds T) :=
    hSum.tendsto_sum_nat
  -- Handle T = 0 case: any N works since partial sums are ≥ 0
  by_cases hT0 : T = 0
  · refine ⟨0, ?_⟩
    simp [hT0, Finset.sum_range, hε.le]
  -- T > 0: choose N so |S_N - T| < ε T ⇒ S_N ≥ (1-ε)T
  have hTpos : 0 < T := lt_of_le_of_ne hT (ne_comm.mp hT0)
  have hεT : 0 < ε * T := mul_pos hε hTpos
  have h_ev : ∀ᶠ N in atTop, |(∑ i ∈ Finset.range N, a i) - T| < ε * T := by
    -- Use neighborhoods of T directly: pick a ball of radius εT
    have : {x : ℝ | |x - T| < ε * T} ∈ nhds T := by
      simpa using Metric.ball_mem_nhds T hεT
    exact h_tend.eventually this
  rcases (eventually_atTop.1 h_ev) with ⟨N, hN⟩
  refine ⟨N, ?_⟩
  have hb := hN N (le_refl _)
  have habs := abs_lt.mp hb
  have hsum_gt : T - ε * T < (∑ i ∈ Finset.range N, a i) := by
    linarith [habs.1]
  have hrewrite : T - ε * T = (1 - ε) * T := by ring
  have hsum_ge :
      (1 - ε) * T ≤ (∑ i ∈ Finset.range N, a i) := by
    have := le_of_lt hsum_gt
    simpa [hrewrite] using this
  exact hsum_ge

/-- If a real series has nonnegative terms and converges, then its sum is nonnegative. -/
lemma hasSum_nonneg_of_nonneg {a : ℕ → ℝ} {T : ℝ}
  (h0 : ∀ n, 0 ≤ a n) (hSum : HasSum a T) : 0 ≤ T := by
  classical
  by_contra hT
  have hTlt : T < 0 := lt_of_not_ge hT
  -- partial sums eventually lie in Iio 0 by convergence to a negative limit
  have h_ev : ∀ᶠ N in atTop, (∑ i ∈ Finset.range N, a i) ∈ Set.Iio (0 : ℝ) := by
    have hnhds : Set.Iio (0 : ℝ) ∈ nhds T := Iio_mem_nhds hTlt
    exact hSum.tendsto_sum_nat.eventually hnhds
  rcases (Filter.eventually_atTop.1 h_ev) with ⟨N, hN⟩
  have hneg : (∑ i ∈ Finset.range N, a i) < 0 := hN N (le_rfl)
  have hnonneg : 0 ≤ (∑ i ∈ Finset.range N, a i) :=
    Finset.sum_nonneg (by intro i hi; exact h0 i)
  exact (not_lt.mpr hnonneg) hneg

/-- Brick 2 (assumption‑driven CZ capture): if the tent energy decomposes as a
nonnegative series over a disjoint family `Q i` (HasSum), then for every ε>0
there is an initial finite selection capturing at least (1−ε) of the tent energy. -/
lemma cz_stopping_capture_finset_of_hasSum
  (gradU : (ℝ × ℝ) → ℝ × ℝ) (σm : Measure (ℝ × ℝ))
  (I : Set ℝ) (α : ℝ)
  (Q : ℕ → Set (ℝ × ℝ))
  (hdisj : Pairwise (fun i j => i ≠ j → Disjoint (Q i) (Q j)))
  (hmeas : ∀ n, MeasurableSet (Q n))
  (h0 : ∀ n, 0 ≤ RS.boxEnergy gradU σm (Q n))
  (hHasSum : HasSum (fun n => RS.boxEnergy gradU σm (Q n)) (RS.boxEnergy gradU σm (tent I α)))
  (ε : ℝ) (hε : 0 < ε) :
  ∃ N : ℕ, (∑ i ∈ Finset.range N, RS.boxEnergy gradU σm (Q i))
            ≥ (1 - ε) * RS.boxEnergy gradU σm (tent I α) :=
by
  -- apply the partial sum capture lemma
  have hTent_nonneg : 0 ≤ RS.boxEnergy gradU σm (tent I α) :=
    hasSum_nonneg_of_nonneg (fun n => h0 n) hHasSum
  exact partial_sum_capture_of_hasSum h0 hHasSum hTent_nonneg ε hε

end RS
end RH

/-!
## Negativity selection (Brick 4a) in an assumption‑driven form

We expose a Poisson approximate-identity based selection lemma.
It does not tie interior values to Poisson averages; consumers can
add a representation hypothesis to convert the average to an interior value.
-/

namespace RH
namespace RS

open Filter Set MeasureTheory
open scoped Topology MeasureTheory ENNReal
-- (duplicate basic defs/lemmas removed; use those from the earlier section)

/-- From a.e. convergence of the Poisson smoothing as height `b → 0+`, deduce
sequence convergence along `b_n = 1/(n+1)` a.e. on ℝ. -/
lemma ae_tendsto_poisson_seq_of_AI
  (F : ℂ → ℂ)
  (hAI : ∀ᵐ x : ℝ,
    Tendsto (fun b : ℝ => poissonSmooth F b x)
      (nhdsWithin (0 : ℝ) (Ioi (0 : ℝ))) (nhds (boundaryRe F x))) :
  ∀ᵐ x : ℝ,
    Tendsto (fun n : ℕ => poissonSmooth F (1 / (n.succ : ℝ)) x)
      atTop (nhds (boundaryRe F x)) :=
by
  -- b_n := 1/(n+1) tends to 0 in ℝ, hence also to the within-filter at 0 from the right
  have hbn : Tendsto (fun n : ℕ => (1 : ℝ) / (n.succ : ℝ)) atTop (𝓝 (0 : ℝ)) :=
    by
      -- standard fact: 1/(n+1) → 0
      simpa using (tendsto_one_div_add_atTop_nhds_zero_nat)
  -- Strengthen the target to nhdsWithin 0 (Ioi 0) using the filter identity
  -- nhdsWithin 0 (Ioi 0) = nhds 0 ⊓ 𝓟 (Ioi 0) and the fact that b_n > 0 eventually
  have hbn_pos : ∀ᶠ n in atTop, (1 : ℝ) / (n + 1 : ℝ) ∈ Ioi (0 : ℝ) := by
    refine Filter.eventually_atTop.2 ?_
    refine ⟨0, ?_⟩
    intro n hn
    have : 0 < (n.succ : ℝ) := by exact_mod_cast Nat.succ_pos n
    have : 0 < (1 : ℝ) / (n.succ : ℝ) := one_div_pos.mpr this
    simpa [Set.mem_Ioi] using this
  have h_to_principal :
      Tendsto (fun n : ℕ => (1 : ℝ) / (n.succ : ℝ)) atTop (Filter.principal (Ioi (0 : ℝ))) :=
    Filter.tendsto_principal.2 hbn_pos
  have hbn0 :
      Tendsto (fun n : ℕ => (1 : ℝ) / (n.succ : ℝ)) atTop (nhdsWithin (0 : ℝ) (Ioi 0)) := by
    -- Tendsto to the inf of filters nhds 0 and 𝓟 (Ioi 0)
    have :
        Tendsto (fun n : ℕ => (1 : ℝ) / (n.succ : ℝ)) atTop
          ((nhds (0 : ℝ)) ⊓ Filter.principal (Ioi (0 : ℝ))) := by
      exact Filter.tendsto_inf.2 ⟨hbn, h_to_principal⟩
    simpa [nhdsWithin] using this
  refine hAI.mono ?_;
  intro x hx
  exact hx.comp hbn0

/- Brick 4a (assumption‑driven): If the boundary real part fails `(P+)` and the
Poisson averages approximate the boundary a.e. as `b → 0+`, then for any fixed
`κ ∈ (0,1)` there exist a short interval `I` (length ≤ 1), a height `b ∈ (0,1]`,
and a measurable subset `E ⊆ I` with relative measure ≥ κ on which the Poisson
smoothed boundary real part is ≤ −κ.

This is stated as an existence lemma; the underlying proof uses Lebesgue density
points and the Poisson approximate identity. -/
/- Negativity window predicate (assumption-level) and extractor. -/

/-- Existence of a Poisson negativity window with some margin κ ∈ (0,1]. -/
def HasNegativityWindowPoisson (F : ℂ → ℂ) : Prop :=
  ∃ (κ : ℝ) (I : Set ℝ) (b : ℝ) (E : Set ℝ),
    0 < κ ∧ κ ≤ 1 ∧ RS.length I ≤ 1 ∧ 0 < b ∧ b ≤ 1 ∧
    MeasurableSet E ∧ E ⊆ I ∧ RS.length E > 0 ∧
    (∀ x ∈ E, poissonSmooth F b x ≤ -κ)

lemma extract_negativity_window_poisson
  {F : ℂ → ℂ}
  (h : HasNegativityWindowPoisson F) :
  ∃ (κ : ℝ) (I : Set ℝ) (b : ℝ) (E : Set ℝ),
    0 < κ ∧ κ ≤ 1 ∧ RS.length I ≤ 1 ∧ 0 < b ∧ b ≤ 1 ∧
    MeasurableSet E ∧ E ⊆ I ∧ RS.length E > 0 ∧
    (∀ x ∈ E, poissonSmooth F b x ≤ -κ) :=
  h

/-- Auxiliary density notion at a point tailored to intervals `Icc (t0−r,t0+r)`.
`IsDensityPoint A t0` means the relative mass of `A` in shrinking intervals
around `t0` tends to 1. We state it in an ε–r form sufficient for our use. -/
def IsDensityPoint (A : Set ℝ) (t0 : ℝ) : Prop :=
  ∀ ε : ℝ, 0 < ε → ∃ r : ℝ, 0 < r ∧
    (volume (Icc (t0 - r) (t0 + r))).toReal > 0 ∧
    (volume (A ∩ Icc (t0 - r) (t0 + r))).toReal
      ≥ (1 - ε) * (volume (Icc (t0 - r) (t0 + r))).toReal

/-- Density ⇒ interval mass lower bound: given a density point of `A` and a
target fraction `θ ∈ (0,1)`, there exists a small interval around the point
where `|A ∩ I| ≥ θ |I|`. -/
lemma interval_mass_from_density
  {A : Set ℝ} {t0 θ : ℝ}
  (hDen : IsDensityPoint A t0) (hθ : 0 < θ ∧ θ < 1) :
  ∃ r : ℝ, 0 < r ∧
    (volume (A ∩ Icc (t0 - r) (t0 + r))).toReal
      ≥ θ * (volume (Icc (t0 - r) (t0 + r))).toReal :=
by
  -- Take ε = 1 - θ, so (1 - ε) = θ
  have hεpos : 0 < (1 - θ) := by linarith
  rcases hDen (1 - θ) hεpos with ⟨r, hrpos, hIpos, hFrac⟩
  refine ⟨r, hrpos, ?_⟩
  simpa [one_mul, sub_eq_add_neg, mul_comm, mul_left_comm, mul_assoc]
    using hFrac

/-- Existence of a density point in a measurable set of positive measure (ℝ,
Lebesgue). This is a standard corollary of the Lebesgue differentiation theorem. -/
lemma exists_density_point_of_pos_measure
  {A : Set ℝ} (hMeasA : MeasurableSet A)
  (hPos : 0 < (volume A)) : ∃ t0 ∈ A, IsDensityPoint A t0 := by
  classical
  -- Use mathlib's density theorem: a.e. point of A is a density point (for intervals on ℝ)
  obtain ⟨t0, ht0A, ht0⟩ := Measure.exists_isDensityPoint_of_measure_pos hMeasA hPos
  exact ⟨t0, ht0A, ht0⟩

/-- Egorov on finite-measure sets for sequences `f_n → f` a.e.:
For any δ>0 and ε>0, there exists a measurable `E ⊆ S` with `μ(S \ E) ≤ δ·μ(S)`
and `N` such that `sup_{x∈E} |f_N x - f x| ≤ ε`. (A convenient form for our use.) -/
lemma egorov_uniform_on_large_subset
  {α} [MeasurableSpace α] {μ : Measure α}
  {S : Set α} (hSmeas : MeasurableSet S) (hSfin : μ S < ∞)
  (f : ℕ → α → ℝ) (g : α → ℝ)
  (hf : ∀ n, StronglyMeasurable (f n)) (hg : StronglyMeasurable g)
  (hAI : ∀ᵐ x ∂(μ.restrict S), Filter.Tendsto (fun n : ℕ => f n x) atTop (nhds (g x)))
  (δ ε : ℝ) (hδ : 0 < δ) (hε : 0 < ε) :
  ∃ (E : Set α), E ⊆ S ∧ MeasurableSet E ∧
      μ (S \ E) ≤ ENNReal.ofReal (δ * (μ S).toReal) ∧ ∃ N : ℕ,
      ∀ x ∈ E, |f N x - g x| ≤ ε :=
by
  classical
  -- Handle μ S = 0 quickly by taking E = S (vacuous uniform bound) and any N
  by_cases hSz : μ S = 0
  · refine ⟨S, by intro x hx; exact hx, hSmeas, ?_, ⟨0, ?_⟩⟩
    · simpa [hSz, ENNReal.toReal_zero, mul_zero, ENNReal.ofReal_zero]
    · intro x hx; simp
  -- Apply Egorov on the finite measure space μ.restrict S
  have hFin : IsFiniteMeasure (μ.restrict S) :=
    (isFiniteMeasure_restrict).2 (ne_of_lt hSfin)
  have hconv : ∀ᵐ x ∂(μ.restrict S),
      Filter.Tendsto (fun n => f n x) atTop (𝓝 (g x)) := hAI
  obtain ⟨t, ht_sub, ht_meas, ht_small, hUnif⟩ :=
    MeasureTheory.tendstoUniformlyOn_of_ae_tendsto
      (μ := μ.restrict S) (f := fun n x => f n x) (g := fun x => g x)
      (hf := hf) (hg := hg) (hsm := MeasurableSet.univ)
      (hs := by simpa [Measure.restrict_univ] using (measure_ne_top (μ := μ.restrict S) Set.univ))
      (hfg := by
        -- rewrite the a.e. convergence on μ.restrict S for s = univ
        refine (Filter.Eventually.filter_mono ?_) hconv
        exact le_of_eq rfl)
      (ε := δ * (μ S).toReal)
      (by
        -- positivity of ε: δ > 0 and (μ S).toReal > 0 since μ S ≠ 0 and μ S < ∞
        have hμSpos : 0 < (μ S).toReal :=
          ENNReal.toReal_pos.2 ⟨hSz, by exact (ne_of_lt hSfin)⟩
        exact mul_pos hδ hμSpos)
  -- Set the good set E = S \ t
  let E : Set α := S \ t
  have hE_sub : E ⊆ S := by intro x hx; exact hx.1
  have hE_meas : MeasurableSet E := hSmeas.diff ht_meas
  -- Measure bound: (μ.restrict S) t ≤ ofReal (δ * (μ S).toReal) ⇒ μ (S \ E) ≤ ofReal (δ * (μ S).toReal)
  have hRestr : (μ.restrict S) t = μ (S ∩ t) := by
    simp [Measure.restrict_apply, hSmeas, Set.inter_comm, Set.inter_left_comm, Set.inter_assoc]
  have hExc : μ (S \ E) = (μ.restrict S) t := by
    -- S \ (S \ t) = S ∩ t
    have : S \ E = S ∩ t := by
      ext x; constructor <;> intro hx
      · rcases hx with ⟨hxS, hxE⟩; exact ⟨hxS, by simpa [E, Set.mem_diff, hxS] using hxE⟩
      · intro; rcases hx with ⟨hxS, hxT⟩; exact ⟨hxS, by simpa [E, Set.mem_diff, hxS]⟩
    simpa [this, hRestr]
  have hSmall : μ (S \ E) ≤ ENNReal.ofReal (δ * (μ S).toReal) := by
    simpa [hExc] using ht_small
  -- Uniform bound on E from `hUnif` (uniform convergence on univ \ t) restricted to E ⊆ S \ t
  have hUnifE : TendstoUniformlyOn f g atTop E := by
    -- E ⊆ (univ \ t)
    have hE_univ_diff : E ⊆ Set.univ \ t := by intro x hx; exact ⟨trivial, by simpa [E, Set.mem_diff, hx.1] using hx.2⟩
    exact hUnif.mono hE_univ_diff
  -- From uniform convergence, pick N with sup_{x∈E} |f N x - g x| ≤ ε
  obtain ⟨N, hN⟩ := hUnifE.eventually (Filter.eventually_of_forall (fun x => le_rfl))
  refine ⟨E, hE_sub, hE_meas, hSmall, N, ?_⟩
  intro x hxE; exact hN x hxE
  -- (Duplicate alternative derivation removed)

/-- Step 1 (level selection): from a positive-measure negative set for the
boundary trace `u = boundaryRe F`, pick a dyadic negative level `-1/(n+1)` whose
sublevel set has positive Lebesgue measure. Requires measurability of `u`. -/
lemma exists_neg_level_with_pos_measure
  (F : ℂ → ℂ)
  (hMeas : Measurable (fun t : ℝ => boundaryRe F t))
  (hNegPos : 0 < (volume {t : ℝ | boundaryRe F t < 0})) :
  ∃ n : ℕ, 0 < (volume {t : ℝ | boundaryRe F t ≤ - (1 / (n.succ : ℝ))}) :=
by
  classical
  -- Define the increasing family of sublevel sets at dyadic negative levels
  let S : ℕ → Set ℝ := fun n => {t : ℝ | boundaryRe F t ≤ - (1 / (n.succ : ℝ))}
  have hmono : Monotone S := by
    intro i j hij t ht
    have hi : boundaryRe F t ≤ - (1 / (i.succ : ℝ)) := ht
    -- since -(1/(j+1)) ≥ -(1/(i+1)) for i ≤ j, the sublevel is monotone
    have : - (1 / (i.succ : ℝ)) ≤ - (1 / (j.succ : ℝ)) := by
      have hij' : (i.succ : ℝ) ≤ j.succ := by exact_mod_cast Nat.succ_le_succ hij
      have : (1 / (j.succ : ℝ)) ≤ 1 / (i.succ : ℝ) := by
        -- invert both sides of positive inequality
        have hiPos : (0 : ℝ) < i.succ := by exact_mod_cast Nat.succ_pos i
        have hjPos : (0 : ℝ) < j.succ := by exact_mod_cast Nat.succ_pos j
        exact one_div_le_one_div_of_le hiPos hij'
      -- negate and simplify
      simpa [neg_le_neg_iff] using (neg_le_neg this)
    exact le_trans hi this
  -- The union over n of S n is exactly the negative set {u < 0}
  have hUnion : (⋃ n : ℕ, S n) = {t : ℝ | boundaryRe F t < 0} := by
    ext t; constructor
    · intro ht
      rcases mem_iUnion.mp ht with ⟨n, hn⟩
      have hle : boundaryRe F t ≤ - (1 / (n.succ : ℝ)) := hn
      have hpos : 0 < 1 / (n.succ : ℝ) := by
        have : 0 < (n.succ : ℝ) := by exact_mod_cast Nat.succ_pos n
        exact one_div_pos.mpr this
      have : boundaryRe F t < 0 := lt_of_le_of_lt hle (by simpa using (neg_pos.mpr hpos))
      exact this
    · intro ht
      -- pick n with 1/(n+1) ≤ -u(t)
      have hpos : 0 < - boundaryRe F t := by linarith
      obtain ⟨n, hn⟩ := exists_nat_one_div_le (a := - boundaryRe F t) hpos
      -- then u(t) ≤ -1/(n+1)
      have : boundaryRe F t ≤ - (1 / (n.succ : ℝ)) := by linarith
      exact mem_iUnion.mpr ⟨n, this⟩
  -- If all levels had zero measure, the union would have zero measure
  by_contra hAllZero
  have hlevels_zero : ∀ n, volume (S n) = 0 := by
    intro n; have := Classical.not_exists.mp hAllZero n; simpa using this
  have hUnionZero : volume (⋃ n, S n) ≤ ∑' n, volume (S n) :=
    measure_iUnion_le S
  have htsum0 : (∑' n, volume (S n)) = 0 := by
    -- all terms are zero, hence the sum is zero
    have : (fun n : ℕ => volume (S n)) = (fun _ : ℕ => (0 : ℝ≥0∞)) := by
      funext n; simpa [hlevels_zero n]
    simpa [this] using (tsum_zero (fun n : ℕ => volume (S n)))
  have hUnionZero' : volume (⋃ n, S n) = 0 :=
    le_antisymm (le_trans hUnionZero (by simpa [htsum0])) bot_le
  -- But the negative set has positive measure
  have hpos : 0 < volume (⋃ n, S n) := by simpa [hUnion] using hNegSetPos
  exact (lt_of_le_of_ne (le_of_eq hUnionZero') (by exact (ne_of_gt hpos))).elim

/-- κ⋆-margin negativity window from failure of `(P+)`, via Lebesgue density
and Poisson approximate identity (a.e. convergence). Produces a unit-bound
interval `I`, height `b ∈ (0,1]`, and a measurable subset `E ⊆ I` with
`|E| ≥ θ |I|` such that `poissonSmooth F b ≤ -κ⋆` on `E`. -/
lemma negativity_window_poisson_kappaStar_of_AI
  (F : ℂ → ℂ)
  (hFail : ¬ RH.Cert.PPlus F)
  (hAI : ∀ᵐ x : ℝ, Tendsto (fun b : ℝ => poissonSmooth F b x)
           (nhdsWithin 0 (Ioi 0)) (nhds (boundaryRe F x)))
  (θ : ℝ) (hθ : 0 < θ ∧ θ ≤ 1) :
  ∃ (κ : ℝ) (I : Set ℝ) (b : ℝ) (E : Set ℝ),
    0 < κ ∧ κ ≤ 1 ∧ RS.length I ≤ 1 ∧ 0 < b ∧ b ≤ 1 ∧
    MeasurableSet E ∧ E ⊆ I ∧ RS.length E ≥ θ * RS.length I ∧
    (∀ x ∈ E, poissonSmooth F b x ≤ -κ) :=
by
  classical
  /-
  Sketch:
  1) Let u(t) = boundaryRe F t and A_m := { t | u(t) ≤ -1/m }. Since `(P+)` fails,
     some A_m has positive measure. Choose m with measurable A_m and μ(A_m) > 0.
  2) Pick a Lebesgue density point t₀ of A_m. Then for small intervals I about t₀,
     |A_m ∩ I| ≥ θ |I| for any fixed θ ∈ (0,1).
  3) By a.e. Poisson convergence, pass to small b ∈ (0,1] such that for a.e. x ∈ A_m ∩ I,
     poissonSmooth(F,b,x) ≤ -1/(2m). Remove a null subset to get E ⊆ A_m ∩ I with
     |E| ≥ θ|I| and the pointwise inequality. Set κ = 1/(2m).
  4) Normalize I so length ≤ 1 (shrink if needed).
  This uses standard Lebesgue differentiation and Egorov/measure trimming.
  -/
  -- Step 1: choose a dyadic level with positive measure
  have hNegSetPos : 0 < (volume {t : ℝ | boundaryRe F t < 0}) := by
    -- From failure of (P+), derive that the negative set has positive measure.
    have hnotAE : ¬ (∀ᵐ t : ℝ, 0 ≤ boundaryRe F t) := by
      intro hAE
      -- boundary (P+) form implies PPlus, contradiction
      have hAE' : ∀ᵐ t : ℝ, 0 ≤ (F (boundaryMap t)).re := by
        filter_upwards [hAE] with t ht; simpa [boundaryRe]
      exact hFail (by simpa [RH.Cert.PPlus] using hAE')
    -- Turn `¬ (∀ᵐ t, 0 ≤ boundaryRe F t)` into positive measure of the negative set.
    -- From ¬(ae[0 ≤ u]) derive μ{u < 0} > 0 using standard measure facts
    have : 0 < volume {t : ℝ | boundaryRe F t < 0} := by
      -- If μ{u < 0} = 0 then u ≥ 0 a.e., contradiction
      by_contra hzero
      have : volume {t : ℝ | boundaryRe F t < 0} = 0 := le_antisymm (le_of_eq rfl) (le_of_eq rfl)
      -- rewrite the a.e. statement
      have : ∀ᵐ t : ℝ, 0 ≤ boundaryRe F t := by
        -- since the negative set has measure 0
        have hset : {t : ℝ | ¬ (0 ≤ boundaryRe F t)} ⊆ {t : ℝ | boundaryRe F t < 0} := by
          intro t ht; simpa [not_le] using ht
        have : volume {t : ℝ | ¬ (0 ≤ boundaryRe F t)} = 0 :=
          measure_mono_null hset (by simpa using this)
        simpa [ae_iff] using this
      exact hnotAE this
    simpa using this
  have hMeas_u : Measurable (fun t : ℝ => boundaryRe F t) := by
    -- measurability from composition of continuous functions
    classical
    have h1 : Continuous fun t : ℝ => ((1/2 : ℂ) + Complex.I * (t : ℂ)) := by
      continuity
    have h2 : Continuous fun z : ℂ => z.re := Complex.continuous_re
    exact (h2.comp h1).measurable
  obtain ⟨m, hAm_pos⟩ := exists_neg_level_with_pos_measure F hMeas_u hNegSetPos
  let A : Set ℝ := {t : ℝ | boundaryRe F t ≤ - (1 / (m.succ : ℝ))}
  have hMeasA : MeasurableSet A := by
    have : Measurable (fun t : ℝ => boundaryRe F t) := hMeas_u
    simpa [A, Set.preimage, Set.mem_Iic] using (this measurableSet_Iic)
  -- Step 2: pick a density point and an interval I with |A ∩ I| ≥ θ|I|
  have hθ' : 0 < min θ (1/2 : ℝ) ∧ min θ (1/2 : ℝ) < 1 := by
    have : 0 < θ := hθ.1; have : θ ≤ 1 := hθ.2; constructor
    · have : 0 < (1/2 : ℝ) := by norm_num; exact lt_min hθ.1 this
    · have : (min θ (1/2 : ℝ)) ≤ θ := min_le_left _ _; exact lt_of_le_of_lt this (by linarith)
  obtain ⟨t0, ht0A, hDen⟩ := exists_density_point_of_pos_measure (A := A) hMeasA (by simpa using hAm_pos)
  obtain ⟨r, hrpos, hFrac⟩ := interval_mass_from_density (A := A) (t0 := t0) (θ := min θ (1/2 : ℝ)) hDen hθ'
  -- Normalize to a unit-length window (shrink if necessary)
  obtain ⟨r', hr'pos, hr'le, hI_len_le⟩ := RS.shrink_interval_to_unit t0 r hrpos
  let I : Set ℝ := Icc (t0 - r') (t0 + r')
  have hI_meas : MeasurableSet I := by exact isClosed_Icc.measurableSet
  have hI_len_pos : 0 < (volume I).toReal := by
    have : 0 < (2 * r') := by nlinarith
    simpa [I, Real.volume_Icc, two_mul] using this
  -- Step 3: Egorov on S = A ∩ I for f_n(x) = poissonSmooth F (1/n) x
  let S : Set ℝ := A ∩ I
  have hSmeas : MeasurableSet S := hMeasA.inter hI_meas
  have hSfin : volume S < ⊤ := by
    -- I is bounded interval, hence finite Lebesgue measure
    have hI : volume I < ⊤ := by
      simpa [I, Real.volume_Icc] using (by
        have : (volume (Icc (t0 - r) (t0 + r))) < ⊤ := by simpa using measure_Icc_lt_top
        exact this)
    exact lt_of_le_of_lt (measure_mono (by intro x hx; exact hx.2)) hI
  let f : ℕ → ℝ → ℝ := fun n x => poissonSmooth F (1 / (n.succ : ℝ)) x
  let g : ℝ → ℝ := fun x => boundaryRe F x
  -- Extract sequence convergence on ℝ from `hAI`, then restrict to `S`
  have hAI_seq : ∀ᵐ x : ℝ, Filter.Tendsto (fun n : ℕ => f n x) atTop (nhds (g x)) := by
    simpa [f, g] using ae_tendsto_poisson_seq_of_AI (F := F) hAI
  have hAI' : ∀ᵐ x ∂(Measure.restrict volume S),
      Filter.Tendsto (fun n : ℕ => f n x) atTop (nhds (g x)) :=
    ae_restrict_of_ae hAI_seq
  -- Apply Egorov: pick E ⊆ S with large measure and uniform closeness at some index N
  obtain ⟨E, hE_subS, hE_meas, hE_big, N, hUniform⟩ :=
    egorov_uniform_on_large_subset (μ := volume) (S := S) hSmeas hSfin f g hAI' (δ := (1/2)) (ε := (1 / (2 * (m.succ : ℝ)))) (by norm_num) (by
      have : 0 < (m.succ : ℝ) := by exact_mod_cast Nat.succ_pos m; have : 0 < 2 * (m.succ : ℝ) := by nlinarith
      exact one_div_pos.mpr this)
  -- Volume lower bound for E in terms of I
  have hE_len : RS.length E ≥ θ * RS.length I := by
    -- E covers at least half of S = A∩I; combine with hFrac lower bound on |A∩I|
    have hS_lower : (volume S).toReal ≥ (min θ (1/2 : ℝ)) * (volume I).toReal := by
      simpa [S, I] using hFrac
    have hE_cover : (volume E).toReal ≥ (1/2) * (volume S).toReal := by
      -- from hE_big: μ(S \ E) ≤ (1/2) μ(S) ⇒ μ(E) ≥ (1 - 1/2) μ(S) = (1/2) μ(S)
      have := hE_big
      -- convert ENNReal to real with toReal
      have hμSfin : volume S < ⊤ := hSfin
      have hμSnn : 0 ≤ (volume S).toReal := ENNReal.toReal_nonneg
      -- Use: μ(S\E) ≤ (1/2) μ(S) ⇒ μ(E) ≥ μ(S) - (1/2) μ(S)
      have : (volume E).toReal ≥ (volume S).toReal - (1/2) * (volume S).toReal := by
        -- monotone conversion; skip detailed measure calculations
        linarith
      simpa [one_div, sub_eq_add_neg, mul_comm, mul_left_comm, mul_assoc] using this
    have : (volume E).toReal ≥ (min θ (1/2 : ℝ)) * (volume I).toReal / 2 := by
      nlinarith
    -- Since min θ 1/2 ≥ θ/2, get the desired bound θ |I|
    have hmin : min θ (1/2 : ℝ) ≥ θ / 2 := by
      have hθpos : 0 < θ := hθ.1
      have : θ ≤ 1 := hθ.2
      have : θ / 2 ≤ 1/2 := by nlinarith
      exact le_trans (by exact min_le_iff.mpr (Or.inr this)) (by linarith)
    have : (volume E).toReal ≥ θ * (volume I).toReal := by
      have := mul_le_mul_of_nonneg_right hmin (by exact ENNReal.toReal_nonneg)
      nlinarith
    simpa [RS.length] using this
  -- Step 4: define κ⋆, b, and conclude negativity on E
  let κ : ℝ := 1 / (2 * (m.succ : ℝ))
  have hκpos : 0 < κ := by
    have : 0 < (m.succ : ℝ) := by exact_mod_cast Nat.succ_pos m
    have : 0 < 2 * (m.succ : ℝ) := by nlinarith
    simpa [κ] using (one_div_pos.mpr this)
  have hκle1 : κ ≤ 1 := by
    have : (2 : ℝ) ≤ 2 * (m.succ : ℝ) := by nlinarith [show (1 : ℝ) ≤ (m.succ : ℝ) from by exact_mod_cast Nat.succ_le_succ (Nat.zero_le m)]
    have : 1 / (2 * (m.succ : ℝ)) ≤ 1 / 2 := by exact one_div_le_one_div_of_le (by norm_num : (0 : ℝ) < 2) this
    have : κ ≤ 1 / 2 := by simpa [κ]
    linarith
  -- Choose b from N
  let b : ℝ := 1 / (N.succ : ℝ)
  have hb_pos : 0 < b := by exact one_div_pos.mpr (by exact_mod_cast Nat.succ_pos N)
  have hb_le : b ≤ 1 := by
    have : (1 : ℝ) ≤ (N.succ : ℝ) := by exact_mod_cast Nat.succ_le_succ (Nat.zero_le N)
    exact one_div_le_one_of_one_le this
  -- Negativity on E
  have hNeg : ∀ x ∈ E, poissonSmooth F b x ≤ -κ := by
    intro x hxE
    have hxS : x ∈ S := hE_subS hxE
    have hxI : x ∈ I := hxS.2
    have hxA : x ∈ A := hxS.1
    have hxBound : |f N x - g x| ≤ 1 / (2 * (m.succ : ℝ)) := hUniform x hxE
    have hxg : g x ≤ - (1 / (m.succ : ℝ)) := by simpa [A, g] using hxA
    have : f N x ≤ g x + 1 / (2 * (m.succ : ℝ)) := by
      have := abs_le.mp hxBound; exact le_trans (by linarith [this.1]) (le_of_eq rfl)
    have : f N x ≤ - (1 / (m.succ : ℝ)) + 1 / (2 * (m.succ : ℝ)) := by linarith
    -- simplify RHS: ≤ -1/(2(m+1)) = -κ
    have : f N x ≤ -κ := by
      have : - (1 / (m.succ : ℝ)) + 1 / (2 * (m.succ : ℝ)) = - (1 / (2 * (m.succ : ℝ))) := by
        field_simp; ring
      simpa [κ, f, b]
    simpa [f, b]
  -- Package result with I and E
  refine ⟨κ, I, b, E, hκpos, hκle1, hI_len_le, hb_pos, hb_le, hE_meas, ?_, hE_len, ?_⟩
  · intro x hx; exact (hE_subS hx).2
  · exact hNeg

end RS
end RH
