import WasmGemmGnaf.Gemm.Arithmetic
set_option autoImplicit false

/-!
# Gemm: logical shapes and index order (SPEC §8.1, §8.3)

A GEMM descriptor names three logical tensors, `A : (batch, m, k)`,
`B : (batch, k, n)` and `C : (batch, m, n)`.  SPEC §8.2 and §8.3 both refer to
the canonical `(batch, row, column)` order — for the signaling-NaN scan, for the
checked-overflow scan (`(batch, row, column, k)`), and for the status-detail
rule "then the lexicographically least logical index".  This module fixes that
order once, as a decidable linear order on indices, and proves it *is* a linear
order.

Everything is over `Nat`; SPEC §8.3 requires index and address arithmetic to be
performed over mathematical integers before any conversion to wasm32.
-/

namespace WasmGemmGnaf.Gemm

/-- A logical three-dimensional extent `(batch, rows, cols)`. -/
structure Shape where
  batch : Nat
  rows : Nat
  cols : Nat
  deriving DecidableEq, Repr, Inhabited

namespace Shape

/-- Number of logical elements. -/
def count (s : Shape) : Nat := s.batch * s.rows * s.cols

/-- An empty logical tensor performs no element address (SPEC §8.3). -/
def isEmpty (s : Shape) : Bool := s.batch == 0 || s.rows == 0 || s.cols == 0

theorem count_eq_zero_iff (s : Shape) : s.count = 0 ↔ s.isEmpty = true := by
  simp [count, isEmpty, Nat.mul_eq_zero]

theorem isEmpty_eq_false_iff (s : Shape) :
    s.isEmpty = false ↔ (0 < s.batch ∧ 0 < s.rows ∧ 0 < s.cols) := by
  simp only [isEmpty, Bool.or_eq_false_iff, beq_eq_false_iff_ne, ne_eq]
  omega

end Shape

/-- A logical index into a three-dimensional tensor, in canonical
`(batch, row, column)` order. -/
structure Index where
  b : Nat
  i : Nat
  j : Nat
  deriving DecidableEq, Repr, Inhabited

namespace Index

/-- The index lies inside the shape. -/
def Mem (x : Index) (s : Shape) : Prop :=
  x.b < s.batch ∧ x.i < s.rows ∧ x.j < s.cols

instance (x : Index) (s : Shape) : Decidable (x.Mem s) :=
  inferInstanceAs (Decidable (_ ∧ _ ∧ _))

/-- Row-major linearisation in canonical `(batch, row, column)` order. -/
def linear (x : Index) (s : Shape) : Nat :=
  (x.b * s.rows + x.i) * s.cols + x.j

theorem linear_lt_count {x : Index} {s : Shape} (h : x.Mem s) :
    x.linear s < s.count := by
  obtain ⟨hb, hi, hj⟩ := h
  have h1 : x.b * s.rows + x.i < s.batch * s.rows := by
    have : x.b + 1 ≤ s.batch := hb
    calc x.b * s.rows + x.i < x.b * s.rows + s.rows := by omega
      _ = (x.b + 1) * s.rows := by rw [Nat.succ_mul]
      _ ≤ s.batch * s.rows := Nat.mul_le_mul_right _ this
  calc (x.b * s.rows + x.i) * s.cols + x.j
      < (x.b * s.rows + x.i) * s.cols + s.cols := by omega
    _ = (x.b * s.rows + x.i + 1) * s.cols := by rw [Nat.succ_mul]
    _ ≤ (s.batch * s.rows) * s.cols := Nat.mul_le_mul_right _ h1
    _ = s.count := rfl

theorem div_mod_pair {q r c : Nat} (h : r < c) :
    (r + q * c) / c = q ∧ (r + q * c) % c = r := by
  have hc : 0 < c := by omega
  refine ⟨?_, ?_⟩
  · rw [Nat.add_mul_div_right _ _ hc, Nat.div_eq_of_lt h, Nat.zero_add]
  · rw [Nat.add_mul_mod_self_right, Nat.mod_eq_of_lt h]

theorem linear_injective {x y : Index} {s : Shape}
    (hx : x.Mem s) (hy : y.Mem s) (h : x.linear s = y.linear s) : x = y := by
  obtain ⟨hxb, hxi, hxj⟩ := hx
  obtain ⟨hyb, hyi, hyj⟩ := hy
  simp only [linear] at h
  have h' : x.j + (x.b * s.rows + x.i) * s.cols
      = y.j + (y.b * s.rows + y.i) * s.cols := by omega
  obtain ⟨e1, e3⟩ := div_mod_pair (q := x.b * s.rows + x.i) hxj
  obtain ⟨e2, e4⟩ := div_mod_pair (q := y.b * s.rows + y.i) hyj
  have hrow : x.b * s.rows + x.i = y.b * s.rows + y.i := by rw [← e1, ← e2, h']
  have hj : x.j = y.j := by rw [← e3, ← e4, h']
  have h'' : x.i + x.b * s.rows = y.i + y.b * s.rows := by omega
  obtain ⟨f1, f3⟩ := div_mod_pair (q := x.b) hxi
  obtain ⟨f2, f4⟩ := div_mod_pair (q := y.b) hyi
  have hb : x.b = y.b := by rw [← f1, ← f2, h'']
  have hi : x.i = y.i := by rw [← f3, ← f4, h'']
  cases x; cases y; simp_all

/-! ## The canonical `(batch, row, column)` order -/

/-- Strict lexicographic order on `(b, i, j)`. -/
def lt (x y : Index) : Prop :=
  x.b < y.b ∨ (x.b = y.b ∧ (x.i < y.i ∨ (x.i = y.i ∧ x.j < y.j)))

/-- Non-strict lexicographic order on `(b, i, j)`. -/
def le (x y : Index) : Prop := x = y ∨ x.lt y

instance (x y : Index) : Decidable (x.lt y) :=
  inferInstanceAs (Decidable (_ ∨ _))

instance (x y : Index) : Decidable (x.le y) :=
  inferInstanceAs (Decidable (_ ∨ _))

theorem le_refl (x : Index) : x.le x := Or.inl rfl

theorem lt_irrefl (x : Index) : ¬ x.lt x := by
  simp only [lt]; omega

theorem lt_trans {x y z : Index} (h1 : x.lt y) (h2 : y.lt z) : x.lt z := by
  simp only [lt] at *; omega

theorem le_trans {x y z : Index} (h1 : x.le y) (h2 : y.le z) : x.le z := by
  rcases h1 with rfl | h1
  · exact h2
  · rcases h2 with rfl | h2
    · exact Or.inr h1
    · exact Or.inr (lt_trans h1 h2)

theorem le_antisymm {x y : Index} (h1 : x.le y) (h2 : y.le x) : x = y := by
  rcases h1 with rfl | h1
  · rfl
  · rcases h2 with rfl | h2
    · rfl
    · exact absurd (lt_trans h1 h2) (lt_irrefl x)

theorem le_total (x y : Index) : x.le y ∨ y.le x := by
  cases x; cases y
  simp only [le, lt, Index.mk.injEq]
  omega

theorem lt_iff_le_and_ne {x y : Index} : x.lt y ↔ (x.le y ∧ x ≠ y) := by
  constructor
  · intro h
    refine ⟨Or.inr h, ?_⟩
    rintro rfl
    exact lt_irrefl x h
  · rintro ⟨h | h, hne⟩
    · exact absurd h hne
    · exact h

/-! ## Canonical enumeration -/

/-- Every index of `s`, in canonical `(batch, row, column)` order. -/
def enumerate (s : Shape) : List Index :=
  (List.range s.batch).flatMap fun b =>
    (List.range s.rows).flatMap fun i =>
      (List.range s.cols).map fun j => ⟨b, i, j⟩

theorem mem_enumerate_iff (s : Shape) (x : Index) :
    x ∈ enumerate s ↔ x.Mem s := by
  cases x with
  | mk b i j =>
    simp only [enumerate, List.mem_flatMap, List.mem_map, List.mem_range, Mem,
      Index.mk.injEq]
    constructor
    · rintro ⟨b', hb, i', hi, j', hj, rfl, rfl, rfl⟩
      exact ⟨hb, hi, hj⟩
    · rintro ⟨hb, hi, hj⟩
      exact ⟨b, hb, i, hi, j, hj, rfl, rfl, rfl⟩

theorem enumerate_nil_of_isEmpty {s : Shape} (h : s.isEmpty = true) :
    enumerate s = [] := by
  cases hb : s.batch with
  | zero => simp [enumerate, hb]
  | succ b =>
    cases hr : s.rows with
    | zero => simp [enumerate, hr]
    | succ r =>
      cases hc : s.cols with
      | zero => simp [enumerate, hc]
      | succ c => simp [Shape.isEmpty, hb, hr, hc] at h

/-- The first index of a nonempty shape in canonical order is the origin. -/
theorem origin_mem {s : Shape} (h : s.isEmpty = false) :
    (Index.mk 0 0 0).Mem s := by
  rw [Shape.isEmpty_eq_false_iff] at h
  exact ⟨h.1, h.2.1, h.2.2⟩

theorem origin_le {s : Shape} {x : Index} (_h : x.Mem s) :
    (Index.mk 0 0 0).le x := by
  cases x with
  | mk b i j =>
    simp only [le, lt, Index.mk.injEq]
    omega

end Index

/-! ## The three GEMM shapes (SPEC §8.1) -/

/-- `A` has logical shape `(batch, m, k)`. -/
def shapeA (batch m k : Nat) : Shape := ⟨batch, m, k⟩
/-- `B` has logical shape `(batch, k, n)`. -/
def shapeB (batch k n : Nat) : Shape := ⟨batch, k, n⟩
/-- `C` has logical shape `(batch, m, n)`. -/
def shapeC (batch m n : Nat) : Shape := ⟨batch, m, n⟩

@[simp] theorem shapeA_count (batch m k : Nat) :
    (shapeA batch m k).count = batch * m * k := rfl
@[simp] theorem shapeB_count (batch k n : Nat) :
    (shapeB batch k n).count = batch * k * n := rfl
@[simp] theorem shapeC_count (batch m n : Nat) :
    (shapeC batch m n).count = batch * m * n := rfl

end WasmGemmGnaf.Gemm
