import WasmGemmGnaf.GNAF.Object
set_option autoImplicit false

/-!
# GNAF: shapes, layouts, blocking and traversal order (SPEC §8.1, §11.1)

SPEC §11.1 requires the plan language to represent shape/layout dispatch,
blocking, tiling and traversal order.  SPEC §8.1 requires a layout to encode
base, extent, signed strides, element width and the accessible region, and
requires every address calculation to be justified rather than assumed.

Two address notions are kept apart on purpose:

* `Layout.address` is the *general* signed-stride address of SPEC §8.3,
  computed over mathematical integers (negative and zero strides are legal);
* `Shape.packedIndex` is the packed row-major linear index, for which
  injectivity on the logical domain is actually proved
  (`packedIndex_injective`) — that is the fact a packing or tiling
  transformation needs.
-/

namespace WasmGemmGnaf.GNAF

open WasmGemmGnaf.Foundation

/-! ## Shapes -/

/-- A logical GEMM tensor shape: batch, rows, columns (SPEC §8.1). -/
structure Shape where
  batch : Nat
  rows : Nat
  cols : Nat
  deriving DecidableEq, Repr, Inhabited

namespace Shape

/-- The number of logical elements. -/
def count (s : Shape) : Nat := s.batch * s.rows * s.cols

/-- The packed row-major linear index of a logical position. -/
def packedIndex (s : Shape) (b i j : Nat) : Nat := (b * s.rows + i) * s.cols + j

/-- Swap rows and columns (the `transposeA` / `transposeB` bits of SPEC §8.1). -/
def transpose (s : Shape) : Shape := ⟨s.batch, s.cols, s.rows⟩

@[simp] theorem transpose_transpose (s : Shape) : s.transpose.transpose = s := by
  cases s; rfl

@[simp] theorem count_transpose (s : Shape) : s.transpose.count = s.count := by
  cases s
  simp [transpose, count, Nat.mul_assoc, Nat.mul_comm, Nat.mul_left_comm]

/-- A shape is empty exactly when one of its extents is zero. -/
theorem count_eq_zero_iff (s : Shape) :
    s.count = 0 ↔ s.batch = 0 ∨ s.rows = 0 ∨ s.cols = 0 := by
  simp [count, Nat.mul_eq_zero, or_assoc]

/-- Every logical position of a shape has a packed index below its element
count. -/
theorem packedIndex_lt (s : Shape) {b i j : Nat}
    (hb : b < s.batch) (hi : i < s.rows) (hj : j < s.cols) :
    s.packedIndex b i j < s.count := by
  have h1 : b * s.rows + i < s.batch * s.rows := by
    have : (b + 1) * s.rows ≤ s.batch * s.rows := Nat.mul_le_mul_right _ hb
    rw [Nat.succ_mul] at this
    omega
  have h2 : (b * s.rows + i) + 1 ≤ s.batch * s.rows := h1
  have h3 : ((b * s.rows + i) + 1) * s.cols ≤ (s.batch * s.rows) * s.cols :=
    Nat.mul_le_mul_right _ h2
  rw [Nat.succ_mul] at h3
  simp only [packedIndex, count]
  omega

/-- Auxiliary: a quotient/remainder pair below the divisor is unique. -/
theorem split_unique {c a j a' j' : Nat} (hc : 0 < c) (hj : j < c) (hj' : j' < c)
    (h : a * c + j = a' * c + j') : a = a' ∧ j = j' := by
  have hdiv : (j + a * c) / c = (j' + a' * c) / c := by
    rw [Nat.add_comm j (a * c), Nat.add_comm j' (a' * c)]; exact congrArg (· / c) h
  rw [Nat.add_mul_div_right _ _ hc, Nat.add_mul_div_right _ _ hc,
    Nat.div_eq_of_lt hj, Nat.div_eq_of_lt hj'] at hdiv
  have ha : a = a' := by omega
  subst ha
  exact ⟨rfl, by omega⟩

/-- The packed row-major index is injective on the logical domain: no two
distinct logical positions of a nonempty shape share a packed slot. -/
theorem packedIndex_injective (s : Shape) {b i j b' i' j' : Nat}
    (hb : b < s.batch) (hi : i < s.rows) (hj : j < s.cols)
    (hb' : b' < s.batch) (hi' : i' < s.rows) (hj' : j' < s.cols)
    (h : s.packedIndex b i j = s.packedIndex b' i' j') :
    b = b' ∧ i = i' ∧ j = j' := by
  have hcols : 0 < s.cols := Nat.lt_of_le_of_lt (Nat.zero_le _) hj
  have hrows : 0 < s.rows := Nat.lt_of_le_of_lt (Nat.zero_le _) hi
  obtain ⟨h1, h2⟩ := split_unique hcols hj hj' h
  obtain ⟨h3, h4⟩ := split_unique hrows hi hi' h1
  exact ⟨h3, h4, h2⟩

end Shape

/-! ## Layouts -/

/-- A stored view of a logical tensor (SPEC §8.1, §8.3): base offset, byte
extent, element width, and the three signed strides. -/
structure Layout where
  offset : Nat
  byteLength : Nat
  elemWidth : Nat
  rowStride : Int
  colStride : Int
  batchStride : Int
  deriving DecidableEq, Repr, Inhabited

namespace Layout

/-- The SPEC §8.3 address of a logical position, over mathematical integers.
Negative and zero strides are legal. -/
def address (l : Layout) (b i j : Nat) : Int :=
  (l.offset : Int) + (b : Int) * l.batchStride + (i : Int) * l.rowStride
    + (j : Int) * l.colStride

@[simp] theorem address_zero (l : Layout) : l.address 0 0 0 = (l.offset : Int) := by
  simp [address]

/-- Stepping the batch index adds exactly one batch stride. -/
theorem address_batch_step (l : Layout) (b i j : Nat) :
    l.address (b + 1) i j = l.address b i j + l.batchStride := by
  simp [address, Int.add_mul]
  omega

/-- Stepping the row index adds exactly one row stride. -/
theorem address_row_step (l : Layout) (b i j : Nat) :
    l.address b (i + 1) j = l.address b i j + l.rowStride := by
  simp [address, Int.add_mul]
  omega

/-- Stepping the column index adds exactly one column stride. -/
theorem address_col_step (l : Layout) (b i j : Nat) :
    l.address b i (j + 1) = l.address b i j + l.colStride := by
  simp [address, Int.add_mul]
  omega

/-- A zero-stride axis addresses the same byte for every index on that axis:
the broadcast case that SPEC §8.3 admits for read-only A and B. -/
theorem address_col_broadcast (l : Layout) (h : l.colStride = 0) (b i j j' : Nat) :
    l.address b i j = l.address b i j' := by
  simp [address, h]

/-- The packed row-major layout of a shape at a given element width. -/
def rowMajor (s : Shape) (w offset : Nat) : Layout where
  offset := offset
  byteLength := w * s.count
  elemWidth := w
  rowStride := (s.cols * w : Nat)
  colStride := (w : Nat)
  batchStride := (s.rows * s.cols * w : Nat)

/-- The packed column-major layout of a shape at a given element width. -/
def colMajor (s : Shape) (w offset : Nat) : Layout where
  offset := offset
  byteLength := w * s.count
  elemWidth := w
  rowStride := (w : Nat)
  colStride := (s.rows * w : Nat)
  batchStride := (s.rows * s.cols * w : Nat)

/-- The packed row-major byte address, as a natural number. -/
def packedByte (s : Shape) (w offset b i j : Nat) : Nat :=
  offset + w * s.packedIndex b i j

/-- The row-major layout addresses exactly the packed byte address. -/
theorem rowMajor_address (s : Shape) (w offset b i j : Nat) :
    (rowMajor s w offset).address b i j = (packedByte s w offset b i j : Nat) := by
  have hnat : offset + b * (s.rows * s.cols * w) + i * (s.cols * w) + j * w
      = packedByte s w offset b i j := by
    simp only [packedByte, Shape.packedIndex, Nat.add_mul, Nat.mul_add,
      Nat.mul_assoc, Nat.mul_comm, Nat.mul_left_comm, Nat.add_assoc,
      Nat.add_comm, Nat.add_left_comm]
  rw [← hnat]
  simp [address, rowMajor]

/-- Distinct logical positions of a nonempty shape occupy distinct packed byte
addresses when the element width is positive. -/
theorem packedByte_injective (s : Shape) {w offset b i j b' i' j' : Nat}
    (hw : 0 < w)
    (hb : b < s.batch) (hi : i < s.rows) (hj : j < s.cols)
    (hb' : b' < s.batch) (hi' : i' < s.rows) (hj' : j' < s.cols)
    (h : packedByte s w offset b i j = packedByte s w offset b' i' j') :
    b = b' ∧ i = i' ∧ j = j' := by
  simp only [packedByte] at h
  have hidx : s.packedIndex b i j = s.packedIndex b' i' j' :=
    Nat.eq_of_mul_eq_mul_left hw (by omega)
  exact s.packedIndex_injective hb hi hj hb' hi' hj' hidx

/-- Every packed byte of a shape lies inside the declared byte extent. -/
theorem packedByte_lt_byteLength (s : Shape) {w offset b i j : Nat} (hw : 0 < w)
    (hb : b < s.batch) (hi : i < s.rows) (hj : j < s.cols) :
    packedByte s w offset b i j < offset + (rowMajor s w offset).byteLength := by
  have h : s.packedIndex b i j + 1 ≤ s.count := s.packedIndex_lt hb hi hj
  have h2 : w * (s.packedIndex b i j + 1) ≤ w * s.count :=
    Nat.mul_le_mul (Nat.le_refl w) h
  rw [Nat.mul_succ] at h2
  simp only [packedByte, rowMajor]
  omega

end Layout

/-! ## Layout classes and dispatch -/

/-- The closed set of layout classes on which a plan may dispatch
(SPEC §11.1). -/
inductive LayoutClass
  | rowMajor | colMajor | general
  deriving DecidableEq, Repr, Inhabited

namespace LayoutClass

def tag : LayoutClass → Nat
  | rowMajor => 0 | colMajor => 1 | general => 2

def all : List LayoutClass := [rowMajor, colMajor, general]

theorem mem_all (c : LayoutClass) : c ∈ all := by cases c <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance : Foundation.Fintype LayoutClass where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

instance decidableForall (p : LayoutClass → Prop) [DecidablePred p] :
    Decidable (∀ c, p c) :=
  decidable_of_iff (∀ c ∈ all, p c) ⟨fun h c => h c (mem_all c), fun h c _ => h c⟩

theorem tag_injective : Function.Injective tag := by
  intro a b h; cases a <;> cases b <;> simp_all [tag]

end LayoutClass

namespace Layout

/-- Total layout classification: the dispatch decision of SPEC §11.1.  It is a
total function of the declared view, never inferred from an address. -/
def classify (l : Layout) (s : Shape) : LayoutClass :=
  if l = rowMajor s l.elemWidth l.offset then .rowMajor
  else if l = colMajor s l.elemWidth l.offset then .colMajor
  else .general

theorem classify_rowMajor (s : Shape) (w offset : Nat) :
    (rowMajor s w offset).classify s = .rowMajor := by
  unfold classify
  have h : rowMajor s w offset
      = rowMajor s (rowMajor s w offset).elemWidth (rowMajor s w offset).offset := rfl
  rw [if_pos h]

/-- Classification is exact: a layout classified `rowMajor` *is* the packed
row-major layout, so the dispatch cannot silently accept a different view. -/
theorem classify_eq_rowMajor (l : Layout) (s : Shape)
    (h : l.classify s = .rowMajor) : l = rowMajor s l.elemWidth l.offset := by
  unfold classify at h
  by_cases h1 : l = rowMajor s l.elemWidth l.offset
  · exact h1
  · rw [if_neg h1] at h
    by_cases h2 : l = colMajor s l.elemWidth l.offset
    · rw [if_pos h2] at h; exact absurd h (by decide)
    · rw [if_neg h2] at h; exact absurd h (by decide)

theorem classify_eq_colMajor (l : Layout) (s : Shape)
    (h : l.classify s = .colMajor) : l = colMajor s l.elemWidth l.offset := by
  unfold classify at h
  by_cases h1 : l = rowMajor s l.elemWidth l.offset
  · rw [if_pos h1] at h; exact absurd h (by decide)
  · rw [if_neg h1] at h
    by_cases h2 : l = colMajor s l.elemWidth l.offset
    · exact h2
    · rw [if_neg h2] at h; exact absurd h (by decide)

end Layout

/-! ## Blocking, tiling and traversal order -/

/-- A blocking of the three GEMM extents (SPEC §11.1). -/
structure Tiling where
  mBlock : Nat
  nBlock : Nat
  kBlock : Nat
  deriving DecidableEq, Repr, Inhabited

namespace Tiling

/-- The number of tiles needed to cover `extent` with blocks of size `block`. -/
def tileCount (extent block : Nat) : Nat :=
  if block = 0 then 0 else (extent + block - 1) / block

@[simp] theorem tileCount_zero_block (extent : Nat) : tileCount extent 0 = 0 := by
  simp [tileCount]

@[simp] theorem tileCount_zero_extent {block : Nat} (h : 0 < block) :
    tileCount 0 block = 0 := by
  simp only [tileCount, if_neg (by omega : ¬ block = 0)]
  exact Nat.div_eq_of_lt (by omega)

/-- The tiles cover the extent: a blocked traversal visits every index. -/
theorem tileCount_mul_ge {extent block : Nat} (h : 0 < block) :
    extent ≤ tileCount extent block * block := by
  simp only [tileCount, if_neg (by omega : ¬ block = 0)]
  rw [Nat.mul_comm]
  have hd := Nat.div_add_mod (extent + block - 1) block
  have hm : (extent + block - 1) % block < block := Nat.mod_lt _ h
  omega

/-- Blocking never invents work beyond one extra partial tile. -/
theorem tileCount_mul_lt {extent block : Nat} (h : 0 < block) :
    tileCount extent block * block < extent + block := by
  simp only [tileCount, if_neg (by omega : ¬ block = 0)]
  rw [Nat.mul_comm]
  have hd := Nat.div_add_mod (extent + block - 1) block
  have hm : (extent + block - 1) % block < block := Nat.mod_lt _ h
  omega

/-- An exactly dividing block size yields exactly the quotient many tiles. -/
theorem tileCount_of_dvd {extent block q : Nat} (h : 0 < block) (hq : extent = q * block) :
    tileCount extent block = q := by
  subst hq
  cases q with
  | zero => simpa using tileCount_zero_extent h
  | succ n =>
    simp only [tileCount, if_neg (by omega : ¬ block = 0)]
    have hrw : (n + 1) * block + block - 1 = (block - 1) + n * block + block := by
      rw [Nat.succ_mul]; omega
    rw [hrw, Nat.add_div_right _ h, Nat.add_mul_div_right _ _ h,
      Nat.div_eq_of_lt (by omega)]
    omega

/-- Total number of tiles of a blocked GEMM traversal. -/
def totalTiles (t : Tiling) (m n k : Nat) : Nat :=
  tileCount m t.mBlock * tileCount n t.nBlock * tileCount k t.kBlock

/-- A blocking whose three block sizes are positive. -/
def Positive (t : Tiling) : Prop := 0 < t.mBlock ∧ 0 < t.nBlock ∧ 0 < t.kBlock

instance (t : Tiling) : Decidable t.Positive := by unfold Positive; infer_instance

/-- The trivial blocking traverses one element per tile. -/
def unit : Tiling := ⟨1, 1, 1⟩

theorem unit_positive : unit.Positive := ⟨by decide, by decide, by decide⟩

@[simp] theorem tileCount_one (extent : Nat) : tileCount extent 1 = extent := by
  simp [tileCount]

theorem totalTiles_unit (m n k : Nat) : unit.totalTiles m n k = m * n * k := by
  simp [totalTiles, unit]

end Tiling

/-- The closed set of traversal orders a plan may declare (SPEC §11.1). -/
inductive TraversalOrder
  | ijk | ikj | jik | jki | kij | kji
  deriving DecidableEq, Repr, Inhabited

namespace TraversalOrder

def tag : TraversalOrder → Nat
  | ijk => 0 | ikj => 1 | jik => 2 | jki => 3 | kij => 4 | kji => 5

def all : List TraversalOrder := [ijk, ikj, jik, jki, kij, kji]

theorem mem_all (o : TraversalOrder) : o ∈ all := by cases o <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance : Foundation.Fintype TraversalOrder where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

instance decidableForall (p : TraversalOrder → Prop) [DecidablePred p] :
    Decidable (∀ o, p o) :=
  decidable_of_iff (∀ o ∈ all, p o) ⟨fun h o => h o (mem_all o), fun h o _ => h o⟩

theorem tag_injective : Function.Injective tag := by
  intro a b h; cases a <;> cases b <;> simp_all [tag]

/-- The number of loop-nest permutations is exactly six. -/
theorem all_length : all.length = 6 := rfl

end TraversalOrder

end WasmGemmGnaf.GNAF
