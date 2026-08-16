import WasmGemmGnaf.Gemm.Shape
set_option autoImplicit false

/-!
# Gemm: views, address arithmetic and the complete layout predicate (SPEC §8.3)

SPEC §8.3 fixes the address map and the layout predicate verbatim:

> Each view offset and byte length is `u64`; each row, column, and batch stride
> is the two's-complement `i64` shown in the header. **Address arithmetic is
> performed over mathematical integers before conversion to wasm32.** For a
> nontransposed A, logical `(b,i,t)` addresses
> `ptr + offset + b·batchStride + i·rowStride + t·columnStride`; transposed A
> swaps `i` and `t`. B uses `(b,t,j)` and swaps `t` and `j` when transposed. C
> uses `(b,i,j)`. Every addressed element byte must lie both in the view's
> `[offset, offset + byteLength)` interval and in memory, without mathematical
> or machine wraparound. Negative strides are valid under that test. Zero
> strides and repeated addresses are valid for read-only A and B, including
> broadcasts. Distinct logical C elements must have disjoint element-byte
> ranges; zero C stride is valid only along an extent at most one. Empty
> logical tensors perform no element address and still require the declared
> interval endpoints to be representable.

Every address here is an `Int`; nothing is reduced modulo `2^32` before the
in-bounds test, which is exactly what "without mathematical or machine
wraparound" demands.  The alignment story of SPEC §8.1 is the derived constant
one (`ScalarKind.alignment`), so no view carries an alignment field.
-/

namespace WasmGemmGnaf.Gemm

/-- A declared matrix view: byte interval plus the three signed strides. -/
structure View where
  /-- Byte offset of the interval, relative to `ptr`. -/
  offset : Nat
  /-- Byte length of the interval. -/
  byteLength : Nat
  /-- Signed row stride in bytes. -/
  rowStride : Int
  /-- Signed column stride in bytes. -/
  colStride : Int
  /-- Signed batch stride in bytes. -/
  batchStride : Int
  deriving DecidableEq, Repr, Inhabited

/-- The memory window an invocation sees: its base pointer, its complete extent,
and the number of addressable memory bytes. -/
structure MemoryWindow where
  /-- Base pointer of the raw invocation. -/
  ptr : Nat
  /-- `len` is the complete invocation extent (SPEC §8.3). -/
  len : Nat
  /-- Bytes actually backed by memory. -/
  memoryBytes : Nat
  deriving DecidableEq, Repr, Inhabited

namespace MemoryWindow

/-- The invocation fits in memory. -/
def Fits (M : MemoryWindow) : Prop := M.ptr + M.len ≤ M.memoryBytes

instance (M : MemoryWindow) : Decidable M.Fits := inferInstanceAs (Decidable (_ ≤ _))

end MemoryWindow

namespace View

/-- The address of a logical element, *before* transposition, relative to `ptr`
and computed over the mathematical integers. -/
def elementAddr (v : View) (x : Index) : Int :=
  (v.offset : Int) + (x.b : Int) * v.batchStride + (x.i : Int) * v.rowStride
    + (x.j : Int) * v.colStride

/-- The address of a logical element.  A transposed operand swaps the two inner
coordinates, exactly as SPEC §8.3 states. -/
def addrOf (v : View) : Bool → Index → Int
  | true, x => v.elementAddr ⟨x.b, x.j, x.i⟩
  | false, x => v.elementAddr x

@[simp] theorem addrOf_false (v : View) (x : Index) :
    v.addrOf false x = v.elementAddr x := rfl

@[simp] theorem addrOf_true (v : View) (x : Index) :
    v.addrOf true x = v.elementAddr ⟨x.b, x.j, x.i⟩ := rfl

/-- The extent that `rowStride` runs over, taking transposition into account. -/
def rowExtent (s : Shape) : Bool → Nat
  | true => s.cols
  | false => s.rows

/-- The extent that `colStride` runs over, taking transposition into account. -/
def colExtent (s : Shape) : Bool → Nat
  | true => s.rows
  | false => s.cols

theorem rowExtent_pos {s : Shape} {t : Bool} (h : s.isEmpty = false) :
    0 < rowExtent s t := by
  rw [Shape.isEmpty_eq_false_iff] at h
  cases t <;> simp only [rowExtent] <;> omega

theorem colExtent_pos {s : Shape} {t : Bool} (h : s.isEmpty = false) :
    0 < colExtent s t := by
  rw [Shape.isEmpty_eq_false_iff] at h
  cases t <;> simp only [colExtent] <;> omega

/-- The alignment promise of every view: the derived constant one (SPEC §8.1). -/
def alignment (_v : View) : Nat := 1

@[simp] theorem alignment_eq_one (v : View) : v.alignment = 1 := rfl

/-! ## Extremal addresses

The layout predicate quantifies over every logical element.  Because the address
map is affine in each coordinate, the quantifier collapses to two extremal
addresses, and both are attained.  That is what makes the predicate decidable
without enumerating `batch · rows · cols` elements. -/

/-- Extremal value of `i · t` for `i < n`: least when `lo`, greatest otherwise. -/
def span : Bool → Nat → Int → Int
  | true, n, t => if t < 0 then ((n - 1 : Nat) : Int) * t else 0
  | false, n, t => if t < 0 then 0 else ((n - 1 : Nat) : Int) * t

/-- The coordinate attaining `span`. -/
def spanArg : Bool → Nat → Int → Nat
  | true, n, t => if t < 0 then n - 1 else 0
  | false, n, t => if t < 0 then 0 else n - 1

theorem spanArg_lt (lo : Bool) {n : Nat} (t : Int) (h : 0 < n) : spanArg lo n t < n := by
  cases lo <;> by_cases ht : t < 0 <;> simp [spanArg, ht] <;> omega

theorem spanArg_spec (lo : Bool) (n : Nat) (t : Int) :
    ((spanArg lo n t : Nat) : Int) * t = span lo n t := by
  cases lo <;> by_cases ht : t < 0 <;> simp [spanArg, span, ht]

theorem span_bounds {n : Nat} {t : Int} {i : Nat} (h : i < n) :
    span true n t ≤ (i : Int) * t ∧ (i : Int) * t ≤ span false n t := by
  have hle : ((i : Nat) : Int) ≤ ((n - 1 : Nat) : Int) := by omega
  simp only [span]
  split
  · next ht =>
    refine ⟨Int.mul_le_mul_of_nonpos_right hle (Int.le_of_lt ht), ?_⟩
    have : (i : Int) * t ≤ 0 :=
      Int.mul_nonpos_of_nonneg_of_nonpos (by omega) (Int.le_of_lt ht)
    simpa using this
  · next ht =>
    have ht0 : (0 : Int) ≤ t := by omega
    refine ⟨?_, Int.mul_le_mul_of_nonneg_right hle ht0⟩
    have : (0 : Int) ≤ (i : Int) * t := Int.mul_nonneg (by omega) ht0
    simpa using this

/-- Extremal address the view reaches over the whole logical shape. -/
def addrExtreme (v : View) (s : Shape) (t : Bool) (lo : Bool) : Int :=
  (v.offset : Int) + span lo s.batch v.batchStride
    + span lo (rowExtent s t) v.rowStride + span lo (colExtent s t) v.colStride

/-- Least address the view reaches. -/
def addrLo (v : View) (s : Shape) (t : Bool) : Int := v.addrExtreme s t true

/-- Greatest address the view reaches. -/
def addrHi (v : View) (s : Shape) (t : Bool) : Int := v.addrExtreme s t false

/-- Every addressed element lies between the two extremal addresses. -/
theorem addr_bounds {v : View} {s : Shape} {t : Bool} {x : Index} (h : x.Mem s) :
    v.addrLo s t ≤ v.addrOf t x ∧ v.addrOf t x ≤ v.addrHi s t := by
  obtain ⟨hb, hi, hj⟩ := h
  have sb := span_bounds (n := s.batch) (t := v.batchStride) (i := x.b) hb
  cases t with
  | false =>
    have sr := span_bounds (n := s.rows) (t := v.rowStride) (i := x.i) hi
    have sc := span_bounds (n := s.cols) (t := v.colStride) (i := x.j) hj
    simp only [addrOf_false, elementAddr, addrLo, addrHi, addrExtreme, rowExtent,
      colExtent]
    omega
  | true =>
    have sr := span_bounds (n := s.cols) (t := v.rowStride) (i := x.j) hj
    have sc := span_bounds (n := s.rows) (t := v.colStride) (i := x.i) hi
    simp only [addrOf_true, elementAddr, addrLo, addrHi, addrExtreme, rowExtent,
      colExtent]
    omega

theorem addrLo_le_addrHi (v : View) (s : Shape) (t : Bool) (h : s.isEmpty = false) :
    v.addrLo s t ≤ v.addrHi s t := by
  have hm : (Index.mk 0 0 0).Mem s := by
    rw [Shape.isEmpty_eq_false_iff] at h
    exact ⟨h.1, h.2.1, h.2.2⟩
  obtain ⟨h1, h2⟩ := addr_bounds (v := v) (t := t) hm
  omega

/-- The index attaining the extremal address (`lo = true` for the least). -/
def argExtreme (v : View) (s : Shape) (t : Bool) (lo : Bool) : Index :=
  match t with
  | true => ⟨spanArg lo s.batch v.batchStride,
             spanArg lo (colExtent s t) v.colStride,
             spanArg lo (rowExtent s t) v.rowStride⟩
  | false => ⟨spanArg lo s.batch v.batchStride,
              spanArg lo (rowExtent s t) v.rowStride,
              spanArg lo (colExtent s t) v.colStride⟩

theorem argExtreme_mem {v : View} {s : Shape} {t lo : Bool} (h : s.isEmpty = false) :
    (v.argExtreme s t lo).Mem s := by
  have hb : 0 < s.batch := by rw [Shape.isEmpty_eq_false_iff] at h; omega
  have hre := rowExtent_pos (s := s) (t := t) h
  have hce := colExtent_pos (s := s) (t := t) h
  cases t with
  | true =>
    refine ⟨spanArg_lt lo _ hb, ?_, ?_⟩
    · have := spanArg_lt lo v.colStride hce
      simpa only [colExtent] using this
    · have := spanArg_lt lo v.rowStride hre
      simpa only [rowExtent] using this
  | false =>
    refine ⟨spanArg_lt lo _ hb, ?_, ?_⟩
    · have := spanArg_lt lo v.rowStride hre
      simpa only [rowExtent] using this
    · have := spanArg_lt lo v.colStride hce
      simpa only [colExtent] using this

theorem argExtreme_addr (v : View) (s : Shape) (t : Bool) (lo : Bool) :
    v.addrOf t (v.argExtreme s t lo) = v.addrExtreme s t lo := by
  cases t <;>
    simp only [argExtreme, addrOf_false, addrOf_true, elementAddr, addrExtreme,
      spanArg_spec]

/-! ## The complete layout predicate -/

/-- Every addressed element byte lies in the view's declared interval. -/
def ElementsInInterval (v : View) (s : Shape) (t : Bool) (w : Nat) : Prop :=
  ∀ x : Index, x.Mem s →
    (v.offset : Int) ≤ v.addrOf t x ∧
      v.addrOf t x + (w : Int) ≤ (v.offset : Int) + (v.byteLength : Int)

/-- The decidable extremal form of `ElementsInInterval`. -/
def checkInterval (v : View) (s : Shape) (t : Bool) (w : Nat) : Bool :=
  s.isEmpty ||
    (decide ((v.offset : Int) ≤ v.addrLo s t) &&
      decide (v.addrHi s t + (w : Int) ≤ (v.offset : Int) + (v.byteLength : Int)))

/-- The extremal check is *exactly* the elementwise predicate. -/
theorem checkInterval_iff (v : View) (s : Shape) (t : Bool) (w : Nat) :
    checkInterval v s t w = true ↔ ElementsInInterval v s t w := by
  constructor
  · intro h x hx
    simp only [checkInterval, Bool.or_eq_true, Bool.and_eq_true, decide_eq_true_eq] at h
    rcases h with h | ⟨h1, h2⟩
    · exfalso
      obtain ⟨hb, hi, hj⟩ := hx
      simp only [Shape.isEmpty, Bool.or_eq_true, beq_iff_eq] at h
      omega
    · obtain ⟨hlo, hhi⟩ := addr_bounds (v := v) (t := t) hx
      omega
  · intro h
    by_cases he : s.isEmpty = true
    · simp [checkInterval, he]
    · simp only [Bool.not_eq_true] at he
      obtain ⟨h1, h2⟩ := h _ (argExtreme_mem (v := v) (t := t) (lo := true) he)
      obtain ⟨h3, h4⟩ := h _ (argExtreme_mem (v := v) (t := t) (lo := false) he)
      rw [argExtreme_addr] at h1
      rw [argExtreme_addr] at h4
      simp only [checkInterval, he, Bool.and_eq_true,
        decide_eq_true_eq, Bool.false_or]
      exact ⟨h1, h4⟩

instance (v : View) (s : Shape) (t : Bool) (w : Nat) :
    Decidable (ElementsInInterval v s t w) :=
  decidable_of_iff _ (checkInterval_iff v s t w)

/-- Distinct logical elements have disjoint element-byte ranges.  SPEC §8.3
requires this of `C` only; `A` and `B` are read-only and may broadcast. -/
def DistinctElementsDisjoint (v : View) (s : Shape) (t : Bool) (w : Nat) : Prop :=
  ∀ x y : Index, x.Mem s → y.Mem s → x ≠ y →
    (v.addrOf t x + (w : Int) ≤ v.addrOf t y ∨ v.addrOf t y + (w : Int) ≤ v.addrOf t x)

/-- SPEC §8.3: "zero C stride is valid only along an extent at most one". -/
def NonzeroStrideOnWideExtents (v : View) (s : Shape) (t : Bool) : Prop :=
  (2 ≤ s.batch → v.batchStride ≠ 0) ∧
    (2 ≤ rowExtent s t → v.rowStride ≠ 0) ∧
    (2 ≤ colExtent s t → v.colStride ≠ 0)

/-- The zero-stride rule is not an extra axiom: it is *forced* by the
disjointness requirement whenever the element width is positive. -/
theorem nonzeroStride_of_disjoint {v : View} {s : Shape} {t : Bool} {w : Nat}
    (hw : 0 < w) (hs : s.isEmpty = false) (h : DistinctElementsDisjoint v s t w) :
    NonzeroStrideOnWideExtents v s t := by
  rw [Shape.isEmpty_eq_false_iff] at hs
  obtain ⟨hb, hr, hc⟩ := hs
  have mem : ∀ b i j : Nat, b < s.batch → i < s.rows → j < s.cols →
      (Index.mk b i j).Mem s := fun _ _ _ h1 h2 h3 => ⟨h1, h2, h3⟩
  have origin := mem 0 0 0 (by omega) (by omega) (by omega)
  refine ⟨?_, ?_, ?_⟩
  · intro h2 hzero
    have hy := mem 1 0 0 (by omega) (by omega) (by omega)
    have hd := h _ _ origin hy (by decide)
    cases t <;>
      simp only [addrOf_false, addrOf_true, elementAddr, hzero] at hd <;> omega
  · intro h2 hzero
    cases t with
    | false =>
      simp only [rowExtent] at h2
      have hy := mem 0 1 0 (by omega) (by omega) (by omega)
      have hd := h _ _ origin hy (by decide)
      simp only [addrOf_false, elementAddr, hzero] at hd
      omega
    | true =>
      simp only [rowExtent] at h2
      have hy := mem 0 0 1 (by omega) (by omega) (by omega)
      have hd := h _ _ origin hy (by decide)
      simp only [addrOf_true, elementAddr, hzero] at hd
      omega
  · intro h2 hzero
    cases t with
    | false =>
      simp only [colExtent] at h2
      have hy := mem 0 0 1 (by omega) (by omega) (by omega)
      have hd := h _ _ origin hy (by decide)
      simp only [addrOf_false, elementAddr, hzero] at hd
      omega
    | true =>
      simp only [colExtent] at h2
      have hy := mem 0 1 0 (by omega) (by omega) (by omega)
      have hd := h _ _ origin hy (by decide)
      simp only [addrOf_true, elementAddr, hzero] at hd
      omega

/-! ## Layout well-formedness -/

/-- SPEC §8.3's complete layout predicate for one view.

`endpointsRepresentable` is the "empty logical tensors still require the declared
interval endpoints to be representable" clause together with "`len` is the
complete invocation extent.  Every view … must lie inside it". -/
structure LayoutOk (v : View) (s : Shape) (t : Bool) (w : Nat) (M : MemoryWindow) :
    Prop where
  endpointsRepresentable : v.offset + v.byteLength ≤ M.len
  elementsInInterval : ElementsInInterval v s t w

instance (v : View) (s : Shape) (t : Bool) (w : Nat) (M : MemoryWindow) :
    Decidable (LayoutOk v s t w M) :=
  if h : v.offset + v.byteLength ≤ M.len ∧ ElementsInInterval v s t w then
    isTrue ⟨h.1, h.2⟩
  else
    isFalse (fun hc => h ⟨hc.endpointsRepresentable, hc.elementsInInterval⟩)

/-- Membership in the declared interval, plus the invocation fitting in memory,
already forces every addressed byte into memory: SPEC §8.3's "and in memory,
without mathematical or machine wraparound" clause is a *consequence*, not an
extra check. -/
theorem LayoutOk.inMemory {v : View} {s : Shape} {t : Bool} {w : Nat}
    {M : MemoryWindow} (h : LayoutOk v s t w M) (hM : M.Fits)
    {x : Index} (hx : x.Mem s) :
    0 ≤ v.addrOf t x ∧
      (M.ptr : Int) + v.addrOf t x + (w : Int) ≤ (M.memoryBytes : Int) := by
  obtain ⟨h1, h2⟩ := h.elementsInInterval x hx
  have h3 := h.endpointsRepresentable
  have h4 : M.ptr + M.len ≤ M.memoryBytes := hM
  omega

/-- No addressed byte wraps around: every address is nonnegative and its last
byte is strictly below the memory bound. -/
theorem LayoutOk.noWraparound {v : View} {s : Shape} {t : Bool} {w : Nat}
    {M : MemoryWindow} (h : LayoutOk v s t w M) (hM : M.Fits) (hw : 0 < w)
    {x : Index} (hx : x.Mem s) :
    0 ≤ v.addrOf t x ∧ v.addrOf t x < (M.memoryBytes : Int) := by
  obtain ⟨h1, h2⟩ := h.inMemory hM hx
  omega

end View

end WasmGemmGnaf.Gemm
