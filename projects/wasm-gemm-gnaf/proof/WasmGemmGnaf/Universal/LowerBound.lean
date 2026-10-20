/-
  Universal lower bounds on the release objective, and an exact statement of what
  a lower bound cannot do.

  SCOPE.  The bounds here quantify over EVERY competitor -- they are not
  restricted to bilinear schemes as `Universal/BilinearLowerBound.lean` is.  They
  are nonetheless NOT the obligation UOR-GNAF section 19.3 calls "a universal
  lower bound attained", because they are not tight: see
  `lower_bound_below_released_is_not_optimality` at the end of this file, which
  proves in Lean precisely why.

  `Universal.universal_sublevel_coverage` and
  `Artifact.released_wasm_gemm_gnaf_global_optimal` remain absent.
-/
import WasmGemmGnaf.Universal.Sublevel

set_option autoImplicit false

namespace WasmGemmGnaf.Universal

open Foundation Foundation.Fintype Foundation.Finite

/-! ## Aggregation lower bounds -/

/-- Folding `(· + f ·)` over a list whose entries all cost at least `k`
accumulates at least `k` per element. -/
theorem foldl_add_ge_card_mul {α : Type} (f : α → Nat) (k : Nat) :
    ∀ (l : List α) (init : Nat), (∀ a ∈ l, k ≤ f a) →
      init + k * l.length ≤ l.foldl (fun b a => b + f a) init
  | [], init, _ => by simp
  | a :: t, init, h => by
      have ha : k ≤ f a := h a (by simp)
      have ht := foldl_add_ge_card_mul f k t (init + f a)
        (fun x hx => h x (by simp [hx]))
      have hmul : k * (t.length + 1) = k * t.length + k := Nat.mul_succ k t.length
      simp only [List.foldl_cons, List.length_cons]
      omega

/--
**Full-domain aggregation lower bound.**

If every raw invocation costs at least `k` on some charged coordinate, the
full-domain sum of that coordinate is at least `k` times the size of the domain.

This is the shape every universal lower bound in this repository must take:
SPEC section 9.2 aggregates the dynamic vector by summing over the complete raw
invocation domain, so a per-invocation floor multiplies by the domain size.
-/
theorem sum_ge_card_mul {α : Type} [Fintype α] (f : α → Nat) (k : Nat)
    (h : ∀ a : α, k ≤ f a) : k * card α ≤ sum f := by
  have := foldl_add_ge_card_mul f k (elems α) 0 (fun a _ => h a)
  simpa [sum, fold_eq_foldl, card] using this

/-- Specialised to the minimal per-invocation charge: a coordinate that is
nonzero on every raw invocation sums to at least the size of the domain. -/
theorem sum_ge_card {α : Type} [Fintype α] (f : α → Nat)
    (h : ∀ a : α, 1 ≤ f a) : card α ≤ sum f := by
  have := sum_ge_card_mul f 1 h
  simpa using this

/-! ## Transfer to the score -/

/--
Any lower bound on a single charged coordinate is a lower bound on the score,
because every coordinate weight is one (`Cost.coordinate_le_score`).
-/
theorem score_ge_of_coordinate_ge
    (c : Cost.ArtifactVector) (co : Cost.ArtifactCoordinate) (k : Nat)
    (h : k ≤ co.value c) : k ≤ Cost.CanonicalObjective.score c :=
  Nat.le_trans h (Cost.coordinate_le_score c co)

/-- The score of any competitor is at least its module byte count. -/
theorem score_ge_moduleBytes (c : Cost.ArtifactVector) :
    c.static.moduleBytes ≤ Cost.CanonicalObjective.score c :=
  Cost.moduleBytes_le_score c

/-- The score of any competitor is at least its summed executed rule steps. -/
theorem score_ge_sumWasmRuleSteps (c : Cost.ArtifactVector) :
    c.dynamicSum.wasmRuleSteps ≤ Cost.CanonicalObjective.score c :=
  Cost.wasmRuleSteps_le_score c

/-! ## What a lower bound cannot do -/

/--
**The attainment gap, stated exactly.**

Suppose `L` is a proved lower bound: every competitor scores at least `L`.
Suppose the released artifact scores `S`.  If `L < S`, then `L` does **not**
establish that the released artifact is minimal -- there is nothing ruling out a
competitor scoring strictly between `L` and `S`, and in particular a competitor
scoring exactly `L` would beat the release.

Formally: from `L < S` it is false that every score at least `L` is also at least
`S`.  So a lower bound closes the release theorem only when it is *attained*,
`L = S`.  This is exactly the third conjunct of UOR-GNAF section 19.3 --
"complete admission, attainment, and ... a universal lower bound attained" -- and
exactly what is undischarged here.

This theorem is why `LB-002` and the bounds above, though genuinely universal,
do not close `GO-001`.  The repository states the gap rather than papering over
it.
-/
theorem lower_bound_below_released_is_not_optimality
    (lowerBound releasedScore : Nat) (hgap : lowerBound < releasedScore) :
    ¬ (∀ competitorScore : Nat,
        lowerBound ≤ competitorScore → releasedScore ≤ competitorScore) := by
  intro h
  exact absurd (h lowerBound (Nat.le_refl lowerBound)) (Nat.not_le.mpr hgap)

/--
The converse: an **attained** lower bound does close it.  If every competitor
scores at least `L` and the released artifact scores exactly `L`, the released
artifact is minimal.

This is UOR-GNAF Theorem 11.1 (lower-bound attainment) in scalar form.  It is
proved here so that the missing piece is isolated precisely: the inference is
available, the attained bound is not.
-/
theorem attained_lower_bound_is_optimal
    (lowerBound releasedScore : Nat)
    (hattained : releasedScore = lowerBound)
    (competitorScore : Nat) (hcomp : lowerBound ≤ competitorScore) :
    releasedScore ≤ competitorScore := by
  rw [hattained]; exact hcomp

end WasmGemmGnaf.Universal
