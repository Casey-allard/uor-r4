/- Diagnostic invariants for the frozen role region and S2/Pick support tests.
   Checked with the LexLean-R4 Lean toolchain v4.32.1. Integers are mathematical
   scores. Implementations must separately establish arithmetic bounds and parity. -/
import Std

namespace UorR4.Diagnostics

/-- Candidate eligibility is separate from score computation. A retained unique
    winner survives restriction of the eligible words at the identical threshold. -/
theorem role_winner_survives_region {Word : Type} (eligible region : Word → Prop)
    (score : Word → Int) (threshold : Int) (winner : Word)
    (admitted : eligible winner) (inside : region winner)
    (passes : threshold ≤ score winner)
    (unique : ∀ w, eligible w → w ≠ winner → score w < score winner) :
    (eligible winner ∧ region winner) ∧ threshold ≤ score winner ∧
    ∀ w, (eligible w ∧ region w) → w ≠ winner → score w < score winner :=
  ⟨⟨admitted, inside⟩, passes, fun w hw hne => unique w hw.1 hne⟩

/-- Deployed count masses are nonnegative. The representative g has largest
    unigram mass, with smallest original token ID at ties. Outside every explicit
    expert support, x gets no bonus. A positive unigram weight ensures g ranks
    ahead of x even when g itself has a bonus. The separate uniform weight is zero. -/
theorem fallback_representative_dominates
    (prior bonus : Nat → Nat) (g x weight : Nat) (positive : 0 < weight)
    (outside : bonus x = 0)
    (leader : prior x < prior g ∨ (prior x = prior g ∧ g < x)) :
    weight * prior x + bonus x < weight * prior g + bonus g ∨
      (weight * prior x + bonus x = weight * prior g + bonus g ∧ g < x) := by
  rw [outside, Nat.add_zero]
  rcases leader with h | ⟨h, tie⟩
  · exact Or.inl (Nat.lt_of_lt_of_le (Nat.mul_lt_mul_of_pos_left h positive)
      (Nat.le_add_right _ _))
  · rw [h]
    cases b : bonus g with
    | zero => exact Or.inr ⟨by simp, tie⟩
    | succ k => exact Or.inl (by omega)

/-- The union can overcount attainable winners, but cannot omit an actual
    winner under the stated mass, tie and support assumptions. -/
theorem chosen_in_support_or_fallback (prior bonus : Nat → Nat) (supported : Nat → Prop)
    (g chosen weight : Nat) (positive : 0 < weight)
    (zero_off_support : ∀ t, ¬ supported t → bonus t = 0)
    (g_first : ∀ t, t ≠ g → prior t < prior g ∨ (prior t = prior g ∧ g < t))
    (g_cannot_beat_chosen : ¬ (weight * prior chosen + bonus chosen <
        weight * prior g + bonus g ∨ (weight * prior chosen + bonus chosen =
        weight * prior g + bonus g ∧ g < chosen))) :
    supported chosen ∨ chosen = g := by
  by_cases hs : supported chosen
  · exact Or.inl hs
  · by_cases hg : chosen = g
    · exact Or.inr hg
    · exact False.elim (g_cannot_beat_chosen (fallback_representative_dominates
        prior bonus g chosen weight positive (zero_off_support chosen hs) (g_first chosen hg)))

end UorR4.Diagnostics
