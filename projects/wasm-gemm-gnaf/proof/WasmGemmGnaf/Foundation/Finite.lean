import WasmGemmGnaf.Foundation.Canonical
set_option autoImplicit false

/-!
# Foundation: finiteness (SPEC §6.3)

SPEC §6.3 requires every enumerator to expose a `Fintype` instance or a
finite-list coverage theorem.  Only Lean core and `Std` are available, so the
class is defined here: a duplicate-free list together with the proof that it
covers the type.

`Finite.fold` is the aggregation used by `Cost.ExactAggregateCost` (SPEC §9.1),
and `ComponentwiseAdd` / `ComponentwiseMax` are the two operations it is folded
with, with their associativity, commutativity and identity laws proved.
-/

namespace WasmGemmGnaf.Foundation

/-- A type with a duplicate-free finite enumeration covering it. -/
class Fintype (α : Type) where
  protected elemsList : List α
  protected complete : ∀ a : α, a ∈ elemsList
  protected nodupList : elemsList.Nodup

namespace Fintype

/-- The canonical enumeration of `α`. -/
def elems (α : Type) [inst : Fintype α] : List α := inst.elemsList

theorem mem_elems {α : Type} [inst : Fintype α] (a : α) : a ∈ elems α :=
  inst.complete a

theorem elems_nodup (α : Type) [inst : Fintype α] : (elems α).Nodup :=
  inst.nodupList

/-- The number of elements of `α`. -/
def card (α : Type) [Fintype α] : Nat := (elems α).length

/-- Finite-list coverage: every element of `α` occurs in `elems α`. -/
theorem coverage {α : Type} [Fintype α] : ∀ a : α, a ∈ elems α := mem_elems

/-! ## Instances -/

instance instEmpty : Fintype Empty where
  elemsList := []
  complete := fun a => a.elim
  nodupList := List.nodup_nil

instance instUnit : Fintype Unit where
  elemsList := [()]
  complete := fun a => by cases a; simp
  nodupList := by simp

instance instBool : Fintype Bool where
  elemsList := [false, true]
  complete := fun a => by cases a <;> simp
  nodupList := by decide

theorem nodup_finRange (n : Nat) : (List.finRange n).Nodup := by
  rw [List.Nodup, List.pairwise_iff_getElem]
  intro i j hi hj hij
  rw [List.getElem_finRange, List.getElem_finRange]
  simp only [ne_eq, Fin.ext_iff]
  simp
  omega

instance instFin (n : Nat) : Fintype (Fin n) where
  elemsList := List.finRange n
  complete := List.mem_finRange
  nodupList := nodup_finRange n

instance instOption {α : Type} [Fintype α] : Fintype (Option α) where
  elemsList := none :: (elems α).map some
  complete := fun a => by
    cases a with
    | none => simp
    | some a => exact List.mem_cons_of_mem _ (List.mem_map_of_mem (mem_elems a))
  nodupList := by
    rw [List.nodup_cons]
    refine ⟨by simp, ?_⟩
    rw [List.Nodup, List.pairwise_map]
    exact (elems_nodup α).imp (by intro a b h; simpa using h)

instance instSum {α β : Type} [Fintype α] [Fintype β] : Fintype (α ⊕ β) where
  elemsList := (elems α).map Sum.inl ++ (elems β).map Sum.inr
  complete := fun a => by
    cases a with
    | inl a => exact List.mem_append_left _ (List.mem_map_of_mem (mem_elems a))
    | inr b => exact List.mem_append_right _ (List.mem_map_of_mem (mem_elems b))
  nodupList := by
    rw [List.Nodup, List.pairwise_append]
    refine ⟨?_, ?_, ?_⟩
    · rw [List.pairwise_map]
      exact (elems_nodup α).imp (by intro a b h; simpa using h)
    · rw [List.pairwise_map]
      exact (elems_nodup β).imp (by intro a b h; simpa using h)
    · intro x hx y hy
      simp only [List.mem_map] at hx hy
      obtain ⟨a, _, rfl⟩ := hx
      obtain ⟨b, _, rfl⟩ := hy
      simp

instance instProd {α β : Type} [Fintype α] [Fintype β] : Fintype (α × β) where
  elemsList := (elems α).flatMap (fun a => (elems β).map (fun b => (a, b)))
  complete := fun p => by
    rw [List.mem_flatMap]
    exact ⟨p.1, mem_elems p.1, by
      rw [List.mem_map]
      exact ⟨p.2, mem_elems p.2, rfl⟩⟩
  nodupList := by
    rw [List.Nodup, List.pairwise_flatMap]
    constructor
    · intro a _
      rw [List.pairwise_map]
      exact (elems_nodup β).imp (by intro x y h; simpa using h)
    · refine (elems_nodup α).imp ?_
      intro a b hab x hx y hy
      simp only [List.mem_map] at hx hy
      obtain ⟨_, _, rfl⟩ := hx
      obtain ⟨_, _, rfl⟩ := hy
      simp only [ne_eq, Prod.mk.injEq, not_and]
      intro h
      exact absurd h hab

instance instCanonicalDomainTag : Fintype CanonicalDomainTag where
  elemsList := CanonicalDomainTag.all
  complete := CanonicalDomainTag.mem_all
  nodupList := CanonicalDomainTag.all_nodup

end Fintype

/-! ## Folding over a finite type -/

namespace Finite

/-- Fold a binary operation over the whole of a finite type.

This is the aggregation used by `Cost.ExactAggregateCost` (SPEC §9.1). -/
def fold {α β : Type} [Fintype α] (op : β → β → β) (init : β) (f : α → β) : β :=
  (Fintype.elems α).foldl (fun acc a => op acc (f a)) init

theorem fold_eq_foldl {α β : Type} [Fintype α] (op : β → β → β) (init : β)
    (f : α → β) :
    fold op init f = (Fintype.elems α).foldl (fun acc a => op acc (f a)) init := rfl

/-- Folding an associative and commutative operation is invariant under
permutation of the enumeration. -/
theorem foldl_perm {β γ : Type} (op : β → β → β)
    (assoc : ∀ a b c, op (op a b) c = op a (op b c))
    (comm : ∀ a b, op a b = op b a)
    (f : γ → β) {l₁ l₂ : List γ} (h : l₁.Perm l₂) :
    ∀ init : β,
      l₁.foldl (fun acc a => op acc (f a)) init =
        l₂.foldl (fun acc a => op acc (f a)) init := by
  induction h with
  | nil => intro init; rfl
  | cons x _ ih => intro init; simpa using ih (op init (f x))
  | swap x y l =>
    intro init
    simp only [List.foldl_cons]
    have : op (op init (f y)) (f x) = op (op init (f x)) (f y) := by
      rw [assoc, comm (f y) (f x), ← assoc]
    rw [this]
  | trans _ _ ih₁ ih₂ => intro init; rw [ih₁ init, ih₂ init]

/-! ### Natural-number aggregation -/

theorem foldl_add_ge_init {α : Type} (f : α → Nat) :
    ∀ (l : List α) (init : Nat), init ≤ l.foldl (fun acc x => acc + f x) init := by
  intro l
  induction l with
  | nil => intro init; exact Nat.le_refl init
  | cons x xs ih =>
    intro init
    simp only [List.foldl_cons]
    exact Nat.le_trans (Nat.le_add_right init (f x)) (ih (init + f x))

theorem foldl_add_mem {α : Type} (f : α → Nat) :
    ∀ (l : List α) (init : Nat) (a : α), a ∈ l →
      f a ≤ l.foldl (fun acc x => acc + f x) init := by
  intro l
  induction l with
  | nil => intro _ a ha; simp at ha
  | cons x xs ih =>
    intro init a ha
    simp only [List.foldl_cons]
    rcases List.mem_cons.mp ha with rfl | ha'
    · exact Nat.le_trans (Nat.le_add_left (f a) init)
        (foldl_add_ge_init f xs (init + f a))
    · exact ih (init + f x) a ha'

theorem foldl_max_ge_init {α : Type} (f : α → Nat) :
    ∀ (l : List α) (init : Nat),
      init ≤ l.foldl (fun acc x => Nat.max acc (f x)) init := by
  intro l
  induction l with
  | nil => intro init; exact Nat.le_refl init
  | cons x xs ih =>
    intro init
    simp only [List.foldl_cons]
    exact Nat.le_trans (Nat.le_max_left init (f x)) (ih (Nat.max init (f x)))

theorem foldl_max_mem {α : Type} (f : α → Nat) :
    ∀ (l : List α) (init : Nat) (a : α), a ∈ l →
      f a ≤ l.foldl (fun acc x => Nat.max acc (f x)) init := by
  intro l
  induction l with
  | nil => intro _ a ha; simp at ha
  | cons x xs ih =>
    intro init a ha
    simp only [List.foldl_cons]
    rcases List.mem_cons.mp ha with rfl | ha'
    · exact Nat.le_trans (Nat.le_max_right init (f a))
        (foldl_max_ge_init f xs (Nat.max init (f a)))
    · exact ih (Nat.max init (f x)) a ha'

/-- Total of `f` over the whole finite type. -/
def sum {α : Type} [Fintype α] (f : α → Nat) : Nat := fold (· + ·) 0 f

/-- Componentwise maximum of `f` over the whole finite type. -/
def max {α : Type} [Fintype α] (f : α → Nat) : Nat := fold Nat.max 0 f

/-- Finite coverage: every value is bounded by the total. -/
theorem le_sum {α : Type} [Fintype α] (f : α → Nat) (a : α) : f a ≤ sum f :=
  foldl_add_mem f (Fintype.elems α) 0 a (Fintype.mem_elems a)

/-- Finite coverage: every value is bounded by the maximum. -/
theorem le_max {α : Type} [Fintype α] (f : α → Nat) (a : α) : f a ≤ max f :=
  foldl_max_mem f (Fintype.elems α) 0 a (Fintype.mem_elems a)

end Finite

/-! ## Componentwise operations

A cost vector is a family of natural-number coordinates indexed by a coordinate
type.  `ComponentwiseAdd` and `ComponentwiseMax` are the two operations SPEC
§9.1 folds with. -/

/-- The all-zero coordinate vector. -/
def componentwiseZero {ι : Type} : ι → Nat := fun _ => 0

/-- Componentwise addition of coordinate vectors. -/
def ComponentwiseAdd {ι : Type} (a b : ι → Nat) : ι → Nat := fun i => a i + b i

/-- Componentwise maximum of coordinate vectors. -/
def ComponentwiseMax {ι : Type} (a b : ι → Nat) : ι → Nat :=
  fun i => Nat.max (a i) (b i)

/-- Componentwise order on coordinate vectors. -/
def ComponentwiseLE {ι : Type} (a b : ι → Nat) : Prop := ∀ i, a i ≤ b i

namespace ComponentwiseAdd

theorem assoc {ι : Type} (a b c : ι → Nat) :
    ComponentwiseAdd (ComponentwiseAdd a b) c =
      ComponentwiseAdd a (ComponentwiseAdd b c) := by
  funext i
  simp [ComponentwiseAdd, Nat.add_assoc]

theorem comm {ι : Type} (a b : ι → Nat) :
    ComponentwiseAdd a b = ComponentwiseAdd b a := by
  funext i
  simp [ComponentwiseAdd, Nat.add_comm]

theorem zero_left {ι : Type} (a : ι → Nat) :
    ComponentwiseAdd componentwiseZero a = a := by
  funext i
  simp [ComponentwiseAdd, componentwiseZero]

theorem zero_right {ι : Type} (a : ι → Nat) :
    ComponentwiseAdd a componentwiseZero = a := by
  funext i
  simp [ComponentwiseAdd, componentwiseZero]

end ComponentwiseAdd

namespace ComponentwiseMax

theorem assoc {ι : Type} (a b c : ι → Nat) :
    ComponentwiseMax (ComponentwiseMax a b) c =
      ComponentwiseMax a (ComponentwiseMax b c) := by
  funext i
  simp [ComponentwiseMax, Nat.max_assoc]

theorem comm {ι : Type} (a b : ι → Nat) :
    ComponentwiseMax a b = ComponentwiseMax b a := by
  funext i
  simp [ComponentwiseMax, Nat.max_comm]

theorem zero_left {ι : Type} (a : ι → Nat) :
    ComponentwiseMax componentwiseZero a = a := by
  funext i
  simp [ComponentwiseMax, componentwiseZero]

theorem zero_right {ι : Type} (a : ι → Nat) :
    ComponentwiseMax a componentwiseZero = a := by
  funext i
  simp [ComponentwiseMax, componentwiseZero]

end ComponentwiseMax

namespace Finite

/-- Componentwise total over a finite index type. -/
def foldAdd {α ι : Type} [Fintype α] (f : α → ι → Nat) : ι → Nat :=
  fold ComponentwiseAdd componentwiseZero f

/-- Componentwise maximum over a finite index type. -/
def foldMax {α ι : Type} [Fintype α] (f : α → ι → Nat) : ι → Nat :=
  fold ComponentwiseMax componentwiseZero f

theorem foldl_add_apply {α ι : Type} (f : α → ι → Nat) (i : ι) :
    ∀ (l : List α) (init : ι → Nat),
      (l.foldl (fun acc a => ComponentwiseAdd acc (f a)) init) i =
        l.foldl (fun acc a => acc + f a i) (init i) := by
  intro l
  induction l with
  | nil => intro init; rfl
  | cons x xs ih => intro init; simpa [ComponentwiseAdd] using ih (ComponentwiseAdd init (f x))

theorem foldl_max_apply {α ι : Type} (f : α → ι → Nat) (i : ι) :
    ∀ (l : List α) (init : ι → Nat),
      (l.foldl (fun acc a => ComponentwiseMax acc (f a)) init) i =
        l.foldl (fun acc a => Nat.max acc (f a i)) (init i) := by
  intro l
  induction l with
  | nil => intro init; rfl
  | cons x xs ih => intro init; simpa [ComponentwiseMax] using ih (ComponentwiseMax init (f x))

/-- Finite coverage, componentwise: every coordinate of every summand is bounded
by the corresponding coordinate of the total. -/
theorem le_foldAdd {α ι : Type} [Fintype α] (f : α → ι → Nat) (a : α) (i : ι) :
    f a i ≤ foldAdd f i := by
  rw [foldAdd, fold_eq_foldl, foldl_add_apply]
  exact foldl_add_mem (fun x => f x i) (Fintype.elems α) (componentwiseZero i) a
    (Fintype.mem_elems a)

/-- Finite coverage, componentwise: every coordinate is bounded by the
componentwise maximum. -/
theorem le_foldMax {α ι : Type} [Fintype α] (f : α → ι → Nat) (a : α) (i : ι) :
    f a i ≤ foldMax f i := by
  rw [foldMax, fold_eq_foldl, foldl_max_apply]
  exact foldl_max_mem (fun x => f x i) (Fintype.elems α) (componentwiseZero i) a
    (Fintype.mem_elems a)

theorem componentwiseLE_foldAdd {α ι : Type} [Fintype α] (f : α → ι → Nat) (a : α) :
    ComponentwiseLE (f a) (foldAdd f) := fun i => le_foldAdd f a i

theorem componentwiseLE_foldMax {α ι : Type} [Fintype α] (f : α → ι → Nat) (a : α) :
    ComponentwiseLE (f a) (foldMax f) := fun i => le_foldMax f a i

end Finite

end WasmGemmGnaf.Foundation
