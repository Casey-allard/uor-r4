import WasmGemmGnaf.Gemm.Aliasing
import WasmGemmGnaf.Wasm.Profile
set_option autoImplicit false
set_option exponentiation.threshold 400
set_option synthInstance.maxSize 1000
set_option synthInstance.maxHeartbeats 1000000

/-!
# Gemm: the descriptor and its well-formedness (SPEC §8.1, §8.2, §8.3)

SPEC §8.1 shapes the descriptor:

```lean
structure Gemm.Descriptor (P : Wasm.Profile) where
  m n k batch : Nat
  aKind bKind cKind accumulatorKind : ScalarKind
  aLayout : Layout aKind (batch, m, k)
  …
  kindsCompatible : arithmetic.Compatible
  wellFormed : DescriptorWellFormed P m n k batch …
```

Here `DescriptorBody` is the proof-free carrier and `Descriptor` adds exactly
the two proof fields SPEC names.  Neither proof field is a stored conclusion:
`kindsCompatible` is the decidable table membership of §8.2 and `wellFormed` is
the decidable layout/alias predicate of §8.3.
-/

namespace WasmGemmGnaf.Gemm

/-! ## Arithmetic contracts (SPEC §8.1) -/

/-- The arithmetic contract of a kind tuple.  It carries only the mode; every
equation, codec, interval, rounding rule and special-value case is a first-order
value of `Gemm/Arithmetic.lean`, never a typeclass or callback (SPEC §8.2). -/
structure ArithmeticContract (a b c acc : ScalarKind) where
  mode : ArithmeticMode
  deriving DecidableEq, Repr, Inhabited

namespace ArithmeticContract

variable {a b c acc : ScalarKind}

/-- SPEC §8.1's `arithmetic.Compatible`. -/
def Compatible (ac : ArithmeticContract a b c acc) : Prop :=
  Gemm.Compatible ac.mode a b c acc

instance (ac : ArithmeticContract a b c acc) : Decidable ac.Compatible :=
  inferInstanceAs (Decidable (Gemm.Compatible _ _ _ _ _))

end ArithmeticContract

/-! ## Resource contract (SPEC §8.2)

"The release resource constants are Lean naturals: raw invocation extent at most
`2^32 - 1` bytes, ordinary memory at most 65,536 pages, tables at most `2^32 - 1`
elements, live Core-GC and exception heap at most `2^32 - 1` canonical abstract
bytes, peak total live value slots … at most `2^64 - 1`, and at most `2^320`
costed reduction steps per raw invocation." -/

/-- The six released resource constants. -/
structure ResourceContract where
  maxInvocationBytes : Nat
  maxPages : Nat
  maxTableElements : Nat
  maxHeapBytes : Nat
  maxValueSlots : Nat
  maxCostedSteps : Nat
  deriving DecidableEq, Repr, Inhabited

/-- The release values printed in SPEC §8.2. -/
def releaseResourceContract : ResourceContract where
  maxInvocationBytes := 2 ^ 32 - 1
  maxPages := 65536
  maxTableElements := 2 ^ 32 - 1
  maxHeapBytes := 2 ^ 32 - 1
  maxValueSlots := 2 ^ 64 - 1
  maxCostedSteps := 2 ^ 320

theorem releaseResourceContract_maxInvocationBytes :
    releaseResourceContract.maxInvocationBytes = 2 ^ 32 - 1 := rfl
theorem releaseResourceContract_maxPages :
    releaseResourceContract.maxPages = 65536 := rfl
theorem releaseResourceContract_maxTableElements :
    releaseResourceContract.maxTableElements = 2 ^ 32 - 1 := rfl
theorem releaseResourceContract_maxHeapBytes :
    releaseResourceContract.maxHeapBytes = 2 ^ 32 - 1 := rfl
theorem releaseResourceContract_maxValueSlots :
    releaseResourceContract.maxValueSlots = 2 ^ 64 - 1 := rfl
theorem releaseResourceContract_maxCostedSteps :
    releaseResourceContract.maxCostedSteps = 2 ^ 320 := rfl

/-! ## The descriptor body -/

/-- Byte length of the ABI header (SPEC §8.3). -/
def headerBytes : Nat := 256

/-- Byte length of the status-detail record (SPEC §8.3). -/
def statusDetailBytes : Nat := 32

/-- The proof-free descriptor carrier. -/
structure DescriptorBody where
  m : Nat
  n : Nat
  k : Nat
  batch : Nat
  aKind : ScalarKind
  bKind : ScalarKind
  cKind : ScalarKind
  accumulatorKind : ScalarKind
  aView : View
  bView : View
  cView : View
  transposeA : Bool
  transposeB : Bool
  mode : ArithmeticMode
  alphaBits : Nat
  betaBits : Nat
  aliasing : AliasingTag
  scratch : ByteRange
  statusDetail : ByteRange
  deriving DecidableEq, Repr, Inhabited

namespace DescriptorBody

/-- `A` has logical shape `(batch, m, k)` (SPEC §8.1). -/
def shapeA (d : DescriptorBody) : Shape := ⟨d.batch, d.m, d.k⟩
/-- `B` has logical shape `(batch, k, n)`. -/
def shapeB (d : DescriptorBody) : Shape := ⟨d.batch, d.k, d.n⟩
/-- `C` has logical shape `(batch, m, n)`. -/
def shapeC (d : DescriptorBody) : Shape := ⟨d.batch, d.m, d.n⟩

/-- The arithmetic contract induced by the descriptor's tags. -/
def arithmetic (d : DescriptorBody) :
    ArithmeticContract d.aKind d.bKind d.cKind d.accumulatorKind :=
  ⟨d.mode⟩

/-- The six ABI regions this descriptor declares. -/
def regions (d : DescriptorBody) : Regions where
  header := ⟨0, headerBytes⟩
  a := ByteRange.ofView d.aView
  b := ByteRange.ofView d.bView
  c := ByteRange.ofView d.cView
  scratch := d.scratch
  statusDetail := d.statusDetail

/-- Every view's alignment promise is the derived constant one (SPEC §8.1). -/
theorem alignment_eq_one (d : DescriptorBody) :
    d.aView.alignment = 1 ∧ d.bView.alignment = 1 ∧ d.cView.alignment = 1 :=
  ⟨rfl, rfl, rfl⟩

end DescriptorBody

/-! ## Decidable elementwise disjointness of C

SPEC §8.3: "Distinct logical C elements must have disjoint element-byte ranges."
The bounded form below is literally the same proposition, written so that the
core `Nat.decidableBallLT` instances make it decidable — the classifier must be
a *total computable* function, and this is the clause that makes the decision
procedure exist. -/

/-- Bounded (hence decidable) form of `View.DistinctElementsDisjoint`. -/
def DistinctDisjointBounded (v : View) (s : Shape) (t : Bool) (w : Nat) : Prop :=
  ∀ b1, b1 < s.batch → ∀ i1, i1 < s.rows → ∀ j1, j1 < s.cols →
  ∀ b2, b2 < s.batch → ∀ i2, i2 < s.rows → ∀ j2, j2 < s.cols →
    (Index.mk b1 i1 j1) ≠ (Index.mk b2 i2 j2) →
      (v.addrOf t ⟨b1, i1, j1⟩ + (w : Int) ≤ v.addrOf t ⟨b2, i2, j2⟩ ∨
        v.addrOf t ⟨b2, i2, j2⟩ + (w : Int) ≤ v.addrOf t ⟨b1, i1, j1⟩)

instance (v : View) (s : Shape) (t : Bool) (w : Nat) :
    Decidable (DistinctDisjointBounded v s t w) := by
  unfold DistinctDisjointBounded
  infer_instance

theorem distinctDisjointBounded_iff (v : View) (s : Shape) (t : Bool) (w : Nat) :
    DistinctDisjointBounded v s t w ↔ View.DistinctElementsDisjoint v s t w := by
  constructor
  · intro h x y hx hy hne
    cases x; cases y
    exact h _ hx.1 _ hx.2.1 _ hx.2.2 _ hy.1 _ hy.2.1 _ hy.2.2 hne
  · intro h b1 h1 i1 h2 j1 h3 b2 h4 i2 h5 j2 h6 hne
    exact h _ _ ⟨h1, h2, h3⟩ ⟨h4, h5, h6⟩ hne

/-! ## Descriptor well-formedness (SPEC §8.3) -/

/-- SPEC §8.3's complete descriptor predicate: the three views satisfy the
layout predicate, distinct C elements are byte-disjoint, the alias contract
holds, the scratch and status-detail ranges lie inside the invocation extent,
and `alpha`/`beta` fit the C width. -/
structure DescriptorWellFormed (d : DescriptorBody) (M : MemoryWindow) : Prop where
  layoutA : View.LayoutOk d.aView d.shapeA d.transposeA d.aKind.byteWidth M
  layoutB : View.LayoutOk d.bView d.shapeB d.transposeB d.bKind.byteWidth M
  layoutC : View.LayoutOk d.cView d.shapeC false d.cKind.byteWidth M
  cElementsDisjoint : DistinctDisjointBounded d.cView d.shapeC false d.cKind.byteWidth
  scratchInside : d.scratch.stop ≤ M.len
  statusInside : d.statusDetail.stop ≤ M.len
  statusIsRecord : d.statusDetail.length = statusDetailBytes
  headerInside : headerBytes ≤ M.len
  aliasOk : Regions.AliasOk d.aliasing d.regions
  alphaFits : d.alphaBits < 2 ^ (8 * d.cKind.byteWidth)
  betaFits : d.betaBits < 2 ^ (8 * d.cKind.byteWidth)

instance (d : DescriptorBody) (M : MemoryWindow) :
    Decidable (DescriptorWellFormed d M) :=
  if h : View.LayoutOk d.aView d.shapeA d.transposeA d.aKind.byteWidth M ∧
      View.LayoutOk d.bView d.shapeB d.transposeB d.bKind.byteWidth M ∧
      View.LayoutOk d.cView d.shapeC false d.cKind.byteWidth M ∧
      DistinctDisjointBounded d.cView d.shapeC false d.cKind.byteWidth ∧
      d.scratch.stop ≤ M.len ∧ d.statusDetail.stop ≤ M.len ∧
      d.statusDetail.length = statusDetailBytes ∧ headerBytes ≤ M.len ∧
      Regions.AliasOk d.aliasing d.regions ∧
      d.alphaBits < 2 ^ (8 * d.cKind.byteWidth) ∧
      d.betaBits < 2 ^ (8 * d.cKind.byteWidth) then
    isTrue ⟨h.1, h.2.1, h.2.2.1, h.2.2.2.1, h.2.2.2.2.1, h.2.2.2.2.2.1,
      h.2.2.2.2.2.2.1, h.2.2.2.2.2.2.2.1, h.2.2.2.2.2.2.2.2.1,
      h.2.2.2.2.2.2.2.2.2.1, h.2.2.2.2.2.2.2.2.2.2⟩
  else
    isFalse (fun hc => h ⟨hc.layoutA, hc.layoutB, hc.layoutC, hc.cElementsDisjoint,
      hc.scratchInside, hc.statusInside, hc.statusIsRecord, hc.headerInside,
      hc.aliasOk, hc.alphaFits, hc.betaFits⟩)

/-! ## The descriptor -/

/-- SPEC §8.1's descriptor: the proof-free body, the memory window it is read
against, and exactly the two proof obligations SPEC names. -/
structure Descriptor (P : Wasm.Profile) where
  body : DescriptorBody
  window : MemoryWindow
  kindsCompatible : (body.arithmetic).Compatible
  wellFormed : DescriptorWellFormed body window

namespace Descriptor

variable {P : Wasm.Profile}

/-- A descriptor never mixes stored kinds (SPEC §8.2). -/
theorem kinds_equal (d : Descriptor P) :
    d.body.aKind = d.body.bKind ∧ d.body.bKind = d.body.cKind :=
  compatible_kinds_equal d.kindsCompatible

/-- No stored kind of a descriptor is `exactDyadic` (SPEC §8.1). -/
theorem stored_not_exactDyadic (d : Descriptor P) :
    d.body.aKind ≠ .exactDyadic ∧ d.body.bKind ≠ .exactDyadic ∧
      d.body.cKind ≠ .exactDyadic :=
  compatible_stored_not_exactDyadic d.kindsCompatible

/-- The accumulator of a descriptor is one of the five released accumulators. -/
theorem accumulator_released (d : Descriptor P) :
    d.body.accumulatorKind = .u32 ∨ d.body.accumulatorKind = .u64 ∨
    d.body.accumulatorKind = .i64 ∨ d.body.accumulatorKind = .binary32 ∨
    d.body.accumulatorKind = .binary64 ∨ d.body.accumulatorKind = .exactDyadic :=
  compatible_accumulator d.kindsCompatible

/-- Every addressed `A` byte lies in memory and does not wrap. -/
theorem aBytes_in_memory (d : Descriptor P) (hM : d.window.Fits)
    {x : Index} (hx : x.Mem d.body.shapeA) :
    0 ≤ d.body.aView.addrOf d.body.transposeA x ∧
      (d.window.ptr : Int) + d.body.aView.addrOf d.body.transposeA x
        + (d.body.aKind.byteWidth : Int) ≤ (d.window.memoryBytes : Int) :=
  d.wellFormed.layoutA.inMemory hM hx

/-- Every addressed `B` byte lies in memory and does not wrap. -/
theorem bBytes_in_memory (d : Descriptor P) (hM : d.window.Fits)
    {x : Index} (hx : x.Mem d.body.shapeB) :
    0 ≤ d.body.bView.addrOf d.body.transposeB x ∧
      (d.window.ptr : Int) + d.body.bView.addrOf d.body.transposeB x
        + (d.body.bKind.byteWidth : Int) ≤ (d.window.memoryBytes : Int) :=
  d.wellFormed.layoutB.inMemory hM hx

/-- Every addressed `C` byte lies in memory and does not wrap. -/
theorem cBytes_in_memory (d : Descriptor P) (hM : d.window.Fits)
    {x : Index} (hx : x.Mem d.body.shapeC) :
    0 ≤ d.body.cView.addrOf false x ∧
      (d.window.ptr : Int) + d.body.cView.addrOf false x
        + (d.body.cKind.byteWidth : Int) ≤ (d.window.memoryBytes : Int) :=
  d.wellFormed.layoutC.inMemory hM hx

/-- SPEC §8.3's zero-stride rule for `C` is a consequence of byte-disjointness,
not a separate axiom. -/
theorem cNonzeroStrides (d : Descriptor P) (hw : 0 < d.body.cKind.byteWidth)
    (hne : d.body.shapeC.isEmpty = false) :
    View.NonzeroStrideOnWideExtents d.body.cView d.body.shapeC false :=
  View.nonzeroStride_of_disjoint hw hne
    ((distinctDisjointBounded_iff _ _ _ _).mp d.wellFormed.cElementsDisjoint)

/-- C never overlaps both A and B, whatever the alias tag (SPEC §8.3). -/
theorem c_not_overlapping_both (d : Descriptor P) :
    ¬ ((ByteRange.ofView d.body.cView).Overlaps (ByteRange.ofView d.body.aView) ∧
       (ByteRange.ofView d.body.cView).Overlaps (ByteRange.ofView d.body.bView)) :=
  Regions.not_c_overlaps_both d.wellFormed.aliasOk

/-- The status-detail range of a valid descriptor is exactly 32 bytes. -/
theorem statusDetail_size (d : Descriptor P) :
    d.body.statusDetail.length = 32 :=
  d.wellFormed.statusIsRecord

end Descriptor

end WasmGemmGnaf.Gemm
