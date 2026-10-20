import WasmGemmGnaf.Foundation.Finite
set_option autoImplicit false

/-!
# Foundation: termination measures (SPEC §6.3)

SPEC §6.3 requires every recursive implementation to be structural or to use an
explicit well-founded measure, and every executable proof-producing function to
be total and computable.

This file supplies the explicit measures the later layers recurse on: a
natural-number measure and its well-foundedness, the two-component
lexicographic measure, structural bounded iteration, and the descending-chain
length bound that turns "the measure strictly decreases" into a concrete step
budget.
-/

namespace WasmGemmGnaf.Foundation
namespace Termination

/-! ## Natural-number measures -/

/-- The relation "the measure `μ` strictly decreases". -/
def MeasureLT {α : Type} (μ : α → Nat) (a b : α) : Prop := μ a < μ b

theorem measureLT_wf {α : Type} (μ : α → Nat) : WellFounded (MeasureLT μ) :=
  InvImage.wf μ Nat.lt_wfRel.wf

/-- The `WellFoundedRelation` instance to name in a `termination_by` clause. -/
@[reducible] def measureRel {α : Type} (μ : α → Nat) : WellFoundedRelation α :=
  ⟨MeasureLT μ, measureLT_wf μ⟩

/-- Any step relation that has a strictly decreasing natural measure is
well-founded. -/
theorem wellFounded_of_measure {α : Type} (μ : α → Nat) (r : α → α → Prop)
    (h : ∀ a b, r a b → μ a < μ b) : WellFounded r :=
  Subrelation.wf (fun {a b} hr => h a b hr) (measureLT_wf μ)

/-- A step relation with a strictly decreasing measure, bundled. -/
structure Measured (α : Type) where
  measure : α → Nat
  step : α → α → Prop
  decreasing : ∀ a b, step a b → measure a < measure b

namespace Measured

variable {α : Type}

theorem wf (m : Measured α) : WellFounded m.step :=
  wellFounded_of_measure m.measure m.step m.decreasing

/-- Recursion along a measured step relation. -/
def rec' (m : Measured α) {C : α → Sort _}
    (F : (x : α) → ((y : α) → m.step y x → C y) → C x) (x : α) : C x :=
  m.wf.fix F x

end Measured

/-! ## Lexicographic measures -/

/-- Two-component lexicographic measure. -/
def LexLT {α : Type} (μ₁ μ₂ : α → Nat) (a b : α) : Prop :=
  μ₁ a < μ₁ b ∨ (μ₁ a = μ₁ b ∧ μ₂ a < μ₂ b)

theorem lexLT_wf {α : Type} (μ₁ μ₂ : α → Nat) : WellFounded (LexLT μ₁ μ₂) :=
  Subrelation.wf
    (r := InvImage (Prod.lex Nat.lt_wfRel Nat.lt_wfRel).rel (fun a => (μ₁ a, μ₂ a)))
    (fun {a b} h => by
      rcases h with h | ⟨h1, h2⟩
      · exact Prod.Lex.left _ _ h
      · show Prod.Lex _ _ (μ₁ a, μ₂ a) (μ₁ b, μ₂ b)
        rw [h1]
        exact Prod.Lex.right _ h2)
    (InvImage.wf _ (Prod.lex Nat.lt_wfRel Nat.lt_wfRel).wf)

/-- The lexicographic `WellFoundedRelation` to name in a `termination_by`
clause. -/
@[reducible] def lexRel {α : Type} (μ₁ μ₂ : α → Nat) : WellFoundedRelation α :=
  ⟨LexLT μ₁ μ₂, lexLT_wf μ₁ μ₂⟩

theorem lexLT_of_fst {α : Type} (μ₁ μ₂ : α → Nat) {a b : α}
    (h : μ₁ a < μ₁ b) : LexLT μ₁ μ₂ a b := Or.inl h

theorem lexLT_of_snd {α : Type} (μ₁ μ₂ : α → Nat) {a b : α}
    (h₁ : μ₁ a = μ₁ b) (h₂ : μ₂ a < μ₂ b) : LexLT μ₁ μ₂ a b := Or.inr ⟨h₁, h₂⟩

/-! ## Descending chains

The concrete step budget: a chain along which a natural measure strictly
decreases is no longer than the measure of its head. -/

/-- `Descending μ a l` says `μ` strictly decreases along `a :: l`. -/
def Descending {α : Type} (μ : α → Nat) : α → List α → Prop
  | _, [] => True
  | a, b :: rest => μ b < μ a ∧ Descending μ b rest

@[simp] theorem descending_nil {α : Type} (μ : α → Nat) (a : α) :
    Descending μ a [] := by simp [Descending]

theorem descending_cons {α : Type} (μ : α → Nat) (a b : α) (rest : List α) :
    Descending μ a (b :: rest) ↔ μ b < μ a ∧ Descending μ b rest := by
  simp [Descending]

/-- A strictly `μ`-descending chain starting at `a` has at most `μ a` steps. -/
theorem descending_length_le {α : Type} (μ : α → Nat) :
    ∀ (l : List α) (a : α), Descending μ a l → l.length ≤ μ a := by
  intro l
  induction l with
  | nil => intro a _; simp
  | cons b rest ih =>
    intro a h
    rw [descending_cons] at h
    have hrest := ih b h.2
    simp only [List.length_cons]
    omega

/-! ## Structural bounded iteration

Executable proof-producing functions must be total, so search loops take an
explicit fuel and recurse structurally on it. -/

/-- Apply `f` exactly `n` times. -/
def repeatN {α : Type} (f : α → α) : Nat → α → α
  | 0, a => a
  | n + 1, a => repeatN f n (f a)

@[simp] theorem repeatN_zero {α : Type} (f : α → α) (a : α) : repeatN f 0 a = a := rfl

@[simp] theorem repeatN_succ {α : Type} (f : α → α) (n : Nat) (a : α) :
    repeatN f (n + 1) a = repeatN f n (f a) := rfl

/-- Iterate `f` until `p` holds or the fuel runs out. -/
def iterateUntil {α : Type} (f : α → α) (p : α → Bool) : Nat → α → α
  | 0, a => a
  | n + 1, a => if p a then a else iterateUntil f p n (f a)

@[simp] theorem iterateUntil_zero {α : Type} (f : α → α) (p : α → Bool) (a : α) :
    iterateUntil f p 0 a = a := rfl

theorem iterateUntil_succ {α : Type} (f : α → α) (p : α → Bool) (n : Nat) (a : α) :
    iterateUntil f p (n + 1) a =
      if p a then a else iterateUntil f p n (f a) := rfl

/-- If the fuel is enough and the measure strictly decreases on every step that
does not already satisfy `p`, the iteration stops at a point satisfying `p`.

`μ a ≤ fuel` is the concrete budget: a strictly decreasing natural measure can
be decremented at most `μ a` times. -/
theorem iterateUntil_sound {α : Type} (f : α → α) (p : α → Bool) (μ : α → Nat)
    (hstep : ∀ x, p x = false → μ (f x) < μ x) :
    ∀ (fuel : Nat) (a : α), μ a ≤ fuel → p (iterateUntil f p fuel a) = true := by
  intro fuel
  induction fuel with
  | zero =>
    intro a hμ
    rw [iterateUntil_zero]
    by_cases hp : p a = true
    · exact hp
    · have hfalse : p a = false := by simpa using hp
      have := hstep a hfalse
      omega
  | succ n ih =>
    intro a hμ
    rw [iterateUntil_succ]
    by_cases hp : p a = true
    · simp [hp]
    · have hfalse : p a = false := by simpa using hp
      have hlt := hstep a hfalse
      simp only [hfalse, Bool.false_eq_true, if_false]
      exact ih (f a) (by omega)

/-! ## Finite search

A decidable predicate over a finite type is searched by scanning the canonical
enumeration; termination is structural in the list. -/

/-- The first element of the enumeration satisfying `p`, if any. -/
def findFinite {α : Type} [Fintype α] (p : α → Bool) : Option α :=
  (Fintype.elems α).find? p

theorem findFinite_sound {α : Type} [Fintype α] {p : α → Bool} {a : α}
    (h : findFinite p = some a) : p a = true :=
  List.find?_some h

theorem findFinite_mem {α : Type} [Fintype α] {p : α → Bool} {a : α}
    (h : findFinite p = some a) : a ∈ Fintype.elems α :=
  List.mem_of_find?_eq_some h

/-- Completeness of the finite search: if nothing was found, nothing satisfies
`p`. -/
theorem findFinite_none {α : Type} [Fintype α] {p : α → Bool}
    (h : findFinite p = none) : ∀ a : α, p a = false := by
  intro a
  have := List.find?_eq_none.mp h a (Fintype.mem_elems a)
  simpa using this

end Termination
end WasmGemmGnaf.Foundation
