/-
  Wasm/Profile.lean --- the released portable profile.

  Normative sources: SPEC.md section 6.2 (the first-order body / lawfulness
  split and the canonical identity), section 7.1 (raw invocations), section 7.2
  (the released portable profile) and section 7.5 (the cost contribution law
  and the canonical GC/exception widths).

  SPEC section 6.2 forbids a scope identity from encoding functions or proof
  terms, so everything that is *identified* lives in `ProfileBody`, a
  first-order record, and every law about it is a separate proposition
  (`ProfileLawful`) proved, never stored.  Nothing in this file states a
  conclusion of the release theorem, and nothing is assumed.

  Every declaration in this file is proved.
-/
import WasmGemmGnaf.Foundation.SchemaRegistry
import WasmGemmGnaf.Wasm.Types
import WasmGemmGnaf.Cost.Vector

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Additional encoders -/

namespace Enc

/-- The empty encoding of the unit payload of a nullary constructor. -/
def unitBytes (_ : Unit) : List UInt8 := []

theorem unitBytes_prefixFree : Bytes.PrefixFree unitBytes :=
  Bytes.prefixFree_of_constLength unitBytes 0 (fun _ => rfl)
    (fun a b _ => by cases a; cases b; rfl)

end Enc

/-! ## Pages

The normative wasm32 page size, and the page count needed to cover a byte
count.  `pagesFor` is used by `Wasm.InvocationLawful` (SPEC section 7.1). -/

/-- The normative WebAssembly page size in bytes. -/
def pageSize : Nat := 65536

theorem pageSize_pos : 0 < pageSize := by decide

/-- The number of whole pages needed to cover `bytes` bytes. -/
def pagesFor (bytes : Nat) : Nat := (bytes + pageSize - 1) / pageSize

theorem pagesFor_zero : pagesFor 0 = 0 := by decide

theorem pagesFor_one : pagesFor 1 = 1 := by decide

theorem pagesFor_pageSize : pagesFor pageSize = 1 := by decide

/-- The pages counted really do cover the bytes requested. -/
theorem le_pagesFor_mul (bytes : Nat) : bytes ≤ pagesFor bytes * pageSize := by
  have h := Nat.div_add_mod (bytes + pageSize - 1) pageSize
  have h2 := Nat.mod_lt (bytes + pageSize - 1) pageSize_pos
  unfold pagesFor
  rw [Nat.mul_comm]
  omega

/-- Page counting is monotone. -/
theorem pagesFor_mono {a b : Nat} (h : a ≤ b) : pagesFor a ≤ pagesFor b :=
  Nat.div_le_div_right (by omega)

/-! ## Canonical GC and exception layout (SPEC section 7.5)

"For the released wasm32 GC profile, canonical cost widths are: references and
`i32`/`f32` fields 4 bytes, `i64`/`f64` fields 8, `v128` fields 16, packed
`i8`/`i16` fields 1/2, a struct header 8, and an array header 16.  Struct
fields occur in declared order at the next multiple of `min(fieldWidth, 8)` and
the total rounds to a multiple of 8.  Array stride is the element width rounded
to its `min(width, 8)` alignment and total size is header plus
`length x stride`, rounded to 8.  An exception object has a 16-byte header; its
payload fields use the same declared-order field layout as a struct after that
header, and its total also rounds to 8." -/

/-- Round `n` up to the next multiple of `a`.  `a = 0` is the identity so that
the function is total. -/
def alignTo (a n : Nat) : Nat :=
  if a = 0 then n else ((n + a - 1) / a) * a

@[simp] theorem alignTo_zero_align (n : Nat) : alignTo 0 n = n := rfl

theorem alignTo_pos_eq {a : Nat} (ha : 0 < a) (n : Nat) :
    alignTo a n = ((n + a - 1) / a) * a := by
  unfold alignTo
  rw [if_neg (by omega)]

/-- Alignment never loses bytes. -/
theorem le_alignTo {a : Nat} (ha : 0 < a) (n : Nat) : n ≤ alignTo a n := by
  rw [alignTo_pos_eq ha]
  have h := Nat.div_add_mod (n + a - 1) a
  have h2 := Nat.mod_lt (n + a - 1) ha
  rw [Nat.mul_comm]
  omega

/-- Alignment lands on a multiple of the alignment. -/
theorem alignTo_dvd {a : Nat} (ha : 0 < a) (n : Nat) : a ∣ alignTo a n := by
  rw [alignTo_pos_eq ha]
  exact ⟨(n + a - 1) / a, Nat.mul_comm _ _⟩

/-- Alignment adds strictly less than one alignment unit. -/
theorem alignTo_lt {a : Nat} (ha : 0 < a) (n : Nat) : alignTo a n < n + a := by
  rw [alignTo_pos_eq ha]
  have h := Nat.div_add_mod (n + a - 1) a
  have h2 := Nat.mod_lt (n + a - 1) ha
  rw [Nat.mul_comm]
  omega

/-- An already aligned offset is unchanged. -/
theorem alignTo_of_dvd {a n : Nat} (ha : 0 < a) (h : a ∣ n) :
    alignTo a n = n := by
  obtain ⟨k, rfl⟩ := h
  rw [alignTo_pos_eq ha]
  have hrw : a * k + a - 1 = a * k + (a - 1) := by omega
  rw [hrw, Nat.mul_add_div ha, Nat.div_eq_of_lt (by omega), Nat.add_zero,
    Nat.mul_comm]

theorem alignTo_mod {a : Nat} (ha : 0 < a) (n : Nat) : alignTo a n % a = 0 :=
  Nat.mod_eq_zero_of_dvd (alignTo_dvd ha n)

/-- The pinned abstract cost widths of the released wasm32 GC profile.  These
are first-order data carried by the cost table body, not host layout claims. -/
structure GcLayoutConstants where
  referenceWidth : Nat
  i32Width : Nat
  i64Width : Nat
  f32Width : Nat
  f64Width : Nat
  v128Width : Nat
  packedI8Width : Nat
  packedI16Width : Nat
  structHeaderWidth : Nat
  arrayHeaderWidth : Nat
  exceptionHeaderWidth : Nat
  maxFieldAlignment : Nat
  totalRounding : Nat
  deriving DecidableEq, Repr, Inhabited

namespace GcLayoutConstants

def toNats (L : GcLayoutConstants) : List Nat :=
  [ L.referenceWidth, L.i32Width, L.i64Width, L.f32Width, L.f64Width
  , L.v128Width, L.packedI8Width, L.packedI16Width, L.structHeaderWidth
  , L.arrayHeaderWidth, L.exceptionHeaderWidth, L.maxFieldAlignment
  , L.totalRounding ]

theorem toNats_injective : Function.Injective toNats := by
  intro x y h
  cases x; cases y
  simp only [toNats, List.cons.injEq, and_true] at h
  obtain ⟨e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13⟩ := h
  subst e1; subst e2; subst e3; subst e4; subst e5; subst e6; subst e7
  subst e8; subst e9; subst e10; subst e11; subst e12; subst e13
  rfl

def bytes (L : GcLayoutConstants) : List UInt8 := Enc.natsBytes (toNats L)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.natsBytes_prefixFree.comp toNats_injective

/-- All widths and the two rounding parameters are positive.  Every layout
theorem below is stated under this side condition, and the canonical constants
satisfy it. -/
def Positive (L : GcLayoutConstants) : Prop :=
  0 < L.referenceWidth ∧ 0 < L.i32Width ∧ 0 < L.i64Width ∧ 0 < L.f32Width ∧
    0 < L.f64Width ∧ 0 < L.v128Width ∧ 0 < L.packedI8Width ∧
    0 < L.packedI16Width ∧ 0 < L.maxFieldAlignment ∧ 0 < L.totalRounding

instance instDecidablePositive (L : GcLayoutConstants) : Decidable (Positive L) := by
  unfold Positive; exact inferInstance

/-- The abstract cost width of a value type. -/
def valTypeWidth (L : GcLayoutConstants) : ValType → Nat
  | .num .i32 => L.i32Width
  | .num .i64 => L.i64Width
  | .num .f32 => L.f32Width
  | .num .f64 => L.f64Width
  | .vec .v128 => L.v128Width
  | .ref _ => L.referenceWidth

/-- The abstract cost width of a packed storage type. -/
def packedWidth (L : GcLayoutConstants) : PackedType → Nat
  | .i8 => L.packedI8Width
  | .i16 => L.packedI16Width

/-- The abstract cost width of a storage type. -/
def storageWidth (L : GcLayoutConstants) : StorageType → Nat
  | .val t => valTypeWidth L t
  | .packed t => packedWidth L t

/-- The abstract cost width of a field. -/
def fieldWidth (L : GcLayoutConstants) (f : FieldType) : Nat :=
  storageWidth L f.storage

/-- The alignment of a field: `min(fieldWidth, 8)` for the canonical
constants. -/
def fieldAlignment (L : GcLayoutConstants) (f : FieldType) : Nat :=
  min (fieldWidth L f) L.maxFieldAlignment

theorem valTypeWidth_pos {L : GcLayoutConstants} (h : Positive L) (t : ValType) :
    0 < valTypeWidth L t := by
  obtain ⟨h1, h2, h3, h4, h5, h6, _, _, _, _⟩ := h
  cases t with
  | num n => cases n <;> assumption
  | vec v => cases v <;> assumption
  | ref _ => exact h1

theorem packedWidth_pos {L : GcLayoutConstants} (h : Positive L) (t : PackedType) :
    0 < packedWidth L t := by
  obtain ⟨_, _, _, _, _, _, h7, h8, _, _⟩ := h
  cases t <;> assumption

theorem storageWidth_pos {L : GcLayoutConstants} (h : Positive L)
    (t : StorageType) : 0 < storageWidth L t := by
  cases t with
  | val v => exact valTypeWidth_pos h v
  | packed p => exact packedWidth_pos h p

theorem fieldWidth_pos {L : GcLayoutConstants} (h : Positive L) (f : FieldType) :
    0 < fieldWidth L f :=
  storageWidth_pos h f.storage

theorem fieldAlignment_pos {L : GcLayoutConstants} (h : Positive L)
    (f : FieldType) : 0 < fieldAlignment L f := by
  have h9 : 0 < L.maxFieldAlignment := h.2.2.2.2.2.2.2.2.1
  exact Nat.lt_min.mpr ⟨fieldWidth_pos h f, h9⟩

/-- Declared-order field layout: each field starts at the next multiple of its
alignment, and the result is the offset just past the last field. -/
def layoutFields (L : GcLayoutConstants) : Nat → List FieldType → Nat
  | offset, [] => offset
  | offset, f :: fs =>
      layoutFields L (alignTo (fieldAlignment L f) offset + fieldWidth L f) fs

/-- The declared-order field offsets. -/
def fieldOffsets (L : GcLayoutConstants) : Nat → List FieldType → List Nat
  | _, [] => []
  | offset, f :: fs =>
      alignTo (fieldAlignment L f) offset ::
        fieldOffsets L (alignTo (fieldAlignment L f) offset + fieldWidth L f) fs

theorem fieldOffsets_length (L : GcLayoutConstants) :
    ∀ (offset : Nat) (fs : List FieldType),
      (fieldOffsets L offset fs).length = fs.length
  | _, [] => rfl
  | offset, f :: fs => by
      simp only [fieldOffsets, List.length_cons]
      rw [fieldOffsets_length L _ fs]

/-! The layout functions are total structural recursions on the declared field
list; these are their defining equations. -/

@[simp] theorem layoutFields_nil (L : GcLayoutConstants) (offset : Nat) :
    layoutFields L offset [] = offset := rfl

@[simp] theorem layoutFields_cons (L : GcLayoutConstants) (offset : Nat)
    (f : FieldType) (fs : List FieldType) :
    layoutFields L offset (f :: fs) =
      layoutFields L (alignTo (fieldAlignment L f) offset + fieldWidth L f) fs :=
  rfl

@[simp] theorem fieldOffsets_nil (L : GcLayoutConstants) (offset : Nat) :
    fieldOffsets L offset [] = [] := rfl

@[simp] theorem fieldOffsets_cons (L : GcLayoutConstants) (offset : Nat)
    (f : FieldType) (fs : List FieldType) :
    fieldOffsets L offset (f :: fs) =
      alignTo (fieldAlignment L f) offset ::
        fieldOffsets L (alignTo (fieldAlignment L f) offset + fieldWidth L f)
          fs :=
  rfl

/-- Layout never moves backwards. -/
theorem le_layoutFields {L : GcLayoutConstants} (h : Positive L) :
    ∀ (offset : Nat) (fs : List FieldType), offset ≤ layoutFields L offset fs
  | offset, [] => Nat.le_refl offset
  | offset, f :: fs => by
      refine Nat.le_trans ?_ (le_layoutFields h _ fs)
      exact Nat.le_trans (le_alignTo (fieldAlignment_pos h f) offset)
        (Nat.le_add_right _ _)

/-- Every declared field is laid out strictly inside the object: its offset
plus its width never exceeds the end of the layout. -/
theorem layout_covers_first_field {L : GcLayoutConstants} (h : Positive L)
    (offset : Nat) (f : FieldType) (fs : List FieldType) :
    alignTo (fieldAlignment L f) offset + fieldWidth L f ≤
      layoutFields L offset (f :: fs) :=
  le_layoutFields h _ fs

/-- Round a total size to a multiple of the profile's rounding unit. -/
def roundTotal (L : GcLayoutConstants) (n : Nat) : Nat :=
  alignTo L.totalRounding n

theorem le_roundTotal {L : GcLayoutConstants} (h : Positive L) (n : Nat) :
    n ≤ roundTotal L n :=
  le_alignTo h.2.2.2.2.2.2.2.2.2 n

theorem roundTotal_dvd {L : GcLayoutConstants} (h : Positive L) (n : Nat) :
    L.totalRounding ∣ roundTotal L n :=
  alignTo_dvd h.2.2.2.2.2.2.2.2.2 n

/-- The abstract cost size of a GC structure object. -/
def structSize (L : GcLayoutConstants) (s : StructType) : Nat :=
  roundTotal L (layoutFields L L.structHeaderWidth s.fields)

/-- The abstract cost stride of a GC array element. -/
def arrayStride (L : GcLayoutConstants) (f : FieldType) : Nat :=
  alignTo (fieldAlignment L f) (fieldWidth L f)

/-- The abstract cost size of a GC array object of the given length. -/
def arraySize (L : GcLayoutConstants) (a : ArrayType) (length : Nat) : Nat :=
  roundTotal L (L.arrayHeaderWidth + length * arrayStride L a.element)

/-- The payload of an exception tag, as immutable declared-order fields. -/
def tagPayloadFields (t : TagType) : List FieldType :=
  t.payload.map (fun v => { mutable := false, storage := .val v })

/-- The abstract cost size of an exception object. -/
def exceptionSize (L : GcLayoutConstants) (t : TagType) : Nat :=
  roundTotal L (layoutFields L L.exceptionHeaderWidth (tagPayloadFields t))

/-! ### Totality and rounding of the layout functions -/

theorem structSize_dvd {L : GcLayoutConstants} (h : Positive L) (s : StructType) :
    L.totalRounding ∣ structSize L s :=
  roundTotal_dvd h _

theorem arraySize_dvd {L : GcLayoutConstants} (h : Positive L) (a : ArrayType)
    (length : Nat) : L.totalRounding ∣ arraySize L a length :=
  roundTotal_dvd h _

theorem exceptionSize_dvd {L : GcLayoutConstants} (h : Positive L) (t : TagType) :
    L.totalRounding ∣ exceptionSize L t :=
  roundTotal_dvd h _

theorem structHeaderWidth_le_structSize {L : GcLayoutConstants} (h : Positive L)
    (s : StructType) : L.structHeaderWidth ≤ structSize L s :=
  Nat.le_trans (le_layoutFields h _ s.fields) (le_roundTotal h _)

theorem arrayHeaderWidth_le_arraySize {L : GcLayoutConstants} (h : Positive L)
    (a : ArrayType) (length : Nat) :
    L.arrayHeaderWidth ≤ arraySize L a length :=
  Nat.le_trans (Nat.le_add_right _ _) (le_roundTotal h _)

theorem exceptionHeaderWidth_le_exceptionSize {L : GcLayoutConstants}
    (h : Positive L) (t : TagType) :
    L.exceptionHeaderWidth ≤ exceptionSize L t :=
  Nat.le_trans (le_layoutFields h _ (tagPayloadFields t)) (le_roundTotal h _)

theorem arrayStride_pos {L : GcLayoutConstants} (h : Positive L) (f : FieldType) :
    0 < arrayStride L f :=
  Nat.lt_of_lt_of_le (fieldWidth_pos h f)
    (le_alignTo (fieldAlignment_pos h f) (fieldWidth L f))

/-- An allocated array charges at least one stride per element. -/
theorem length_mul_arrayStride_le_arraySize {L : GcLayoutConstants}
    (h : Positive L) (a : ArrayType) (length : Nat) :
    length * arrayStride L a.element ≤ arraySize L a length :=
  Nat.le_trans (Nat.le_add_left _ _) (le_roundTotal h _)

end GcLayoutConstants

/-- The canonical release GC/exception widths of SPEC section 7.5. -/
def canonicalGcLayout : GcLayoutConstants :=
  { referenceWidth := 4
    i32Width := 4
    i64Width := 8
    f32Width := 4
    f64Width := 8
    v128Width := 16
    packedI8Width := 1
    packedI16Width := 2
    structHeaderWidth := 8
    arrayHeaderWidth := 16
    exceptionHeaderWidth := 16
    maxFieldAlignment := 8
    totalRounding := 8 }

theorem canonicalGcLayout_referenceWidth :
    canonicalGcLayout.referenceWidth = 4 := rfl
theorem canonicalGcLayout_i32Width : canonicalGcLayout.i32Width = 4 := rfl
theorem canonicalGcLayout_f32Width : canonicalGcLayout.f32Width = 4 := rfl
theorem canonicalGcLayout_i64Width : canonicalGcLayout.i64Width = 8 := rfl
theorem canonicalGcLayout_f64Width : canonicalGcLayout.f64Width = 8 := rfl
theorem canonicalGcLayout_v128Width : canonicalGcLayout.v128Width = 16 := rfl
theorem canonicalGcLayout_packedI8Width :
    canonicalGcLayout.packedI8Width = 1 := rfl
theorem canonicalGcLayout_packedI16Width :
    canonicalGcLayout.packedI16Width = 2 := rfl
theorem canonicalGcLayout_structHeaderWidth :
    canonicalGcLayout.structHeaderWidth = 8 := rfl
theorem canonicalGcLayout_arrayHeaderWidth :
    canonicalGcLayout.arrayHeaderWidth = 16 := rfl
theorem canonicalGcLayout_exceptionHeaderWidth :
    canonicalGcLayout.exceptionHeaderWidth = 16 := rfl
theorem canonicalGcLayout_maxFieldAlignment :
    canonicalGcLayout.maxFieldAlignment = 8 := rfl
theorem canonicalGcLayout_totalRounding :
    canonicalGcLayout.totalRounding = 8 := rfl

theorem canonicalGcLayout_positive : canonicalGcLayout.Positive := by decide

/-- Field alignment under the canonical constants is exactly
`min(fieldWidth, 8)`. -/
theorem canonicalGcLayout_fieldAlignment (f : FieldType) :
    canonicalGcLayout.fieldAlignment f =
      min (canonicalGcLayout.fieldWidth f) 8 := rfl

/-- Every canonical struct size is a multiple of 8 (SPEC section 7.5). -/
theorem canonical_structSize_mod_eight (s : StructType) :
    canonicalGcLayout.structSize s % 8 = 0 :=
  Nat.mod_eq_zero_of_dvd
    (GcLayoutConstants.structSize_dvd canonicalGcLayout_positive s)

/-- Every canonical array size is a multiple of 8 (SPEC section 7.5). -/
theorem canonical_arraySize_mod_eight (a : ArrayType) (length : Nat) :
    canonicalGcLayout.arraySize a length % 8 = 0 :=
  Nat.mod_eq_zero_of_dvd
    (GcLayoutConstants.arraySize_dvd canonicalGcLayout_positive a length)

/-- Every canonical exception-object size is a multiple of 8
(SPEC section 7.5). -/
theorem canonical_exceptionSize_mod_eight (t : TagType) :
    canonicalGcLayout.exceptionSize t % 8 = 0 :=
  Nat.mod_eq_zero_of_dvd
    (GcLayoutConstants.exceptionSize_dvd canonicalGcLayout_positive t)

/-- An empty struct still costs its 8-byte header. -/
theorem canonical_structSize_empty :
    canonicalGcLayout.structSize { fields := [] } = 8 := by decide

/-- Worked instance of the declared-order rule: an `i32` field then an `i64`
field, after the 8-byte header, occupy offsets 8 and 16 and round to 24. -/
theorem canonical_structSize_i32_i64 :
    canonicalGcLayout.structSize
        { fields :=
          [ { mutable := false, storage := .val (.num .i32) }
          , { mutable := false, storage := .val (.num .i64) } ] } = 24 := by
  decide

theorem canonical_fieldOffsets_i32_i64 :
    canonicalGcLayout.fieldOffsets 8
        [ { mutable := false, storage := .val (.num .i32) }
        , { mutable := false, storage := .val (.num .i64) } ] = [8, 16] := by
  decide

/-- A `v128` element has stride 16 even though its alignment is capped at 8. -/
theorem canonical_arrayStride_v128 :
    canonicalGcLayout.arrayStride
      { mutable := false, storage := .val (.vec .v128) } = 16 := by decide

theorem canonical_arrayStride_packedI8 :
    canonicalGcLayout.arrayStride
      { mutable := false, storage := .packed .i8 } = 1 := by decide

/-- A reference field costs four abstract bytes, so a runtime-sized GC
allocation is never a unit-cost free store. -/
theorem canonical_arraySize_refs (n : Nat) :
    canonicalGcLayout.arraySize
      { element := { mutable := true, storage := .val (.ref RefType.funcRef) } }
      n = alignTo 8 (16 + n * 4) := rfl

/-! ## Cost table body (SPEC section 7.5)

The cost table body is first-order: unit counts, the canonical widths, and one
row per pinned rule identifier.  The contribution *law* is stated as proved
theorems about a lawful table, never stored as a field. -/

/-- One row of the cost table: a pinned rule identifier from the vendored
conformance map, and the exact dynamic cost it contributes. -/
structure CostRuleRow where
  ruleId : String
  contribution : Cost.DynamicVector
  deriving DecidableEq, Repr, Inhabited

namespace CostRuleRow

/-- The sixteen dynamic coordinates of a contribution, in normative order. -/
def contributionNats (v : Cost.DynamicVector) : List Nat :=
  [ v.instantiationSteps, v.dispatchSteps, v.preparationSteps, v.wasmRuleSteps
  , v.scalarOps, v.vectorLaneOps, v.bytesRead, v.bytesWritten
  , v.memoryGrowPages, v.tableElementsAllocated, v.gcObjectsAllocated
  , v.gcBytesInitialized, v.peakStackValues, v.peakPages, v.peakGcLiveBytes
  , v.outputBytes ]

theorem contributionNats_injective : Function.Injective contributionNats := by
  intro x y h
  cases x; cases y
  simp only [contributionNats, List.cons.injEq, and_true] at h
  obtain ⟨e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15,
    e16⟩ := h
  subst e1; subst e2; subst e3; subst e4; subst e5; subst e6; subst e7
  subst e8; subst e9; subst e10; subst e11; subst e12; subst e13; subst e14
  subst e15; subst e16
  rfl

def bytes (row : CostRuleRow) : List UInt8 :=
  Enc.stringBytes row.ruleId ++ Enc.natsBytes (contributionNats row.contribution)

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Enc.stringBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := Enc.natsBytes_prefixFree _ _ _ _ h
  have h2' := contributionNats_injective h2
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

def listBytes (rows : List CostRuleRow) : List UInt8 :=
  Bytes.listBytes bytes rows

theorem listBytes_prefixFree : Bytes.PrefixFree listBytes :=
  Bytes.listBytes_prefixFree bytes_prefixFree

end CostRuleRow

/-- The first-order cost-table body carried by every profile body
(SPEC section 7.5).  Its canonical identity is the profile's cost-semantics
identity. -/
structure CostTableBody where
  /-- Units charged per consumed byte by the decoder. -/
  decodeUnitPerByte : Nat
  /-- The terminal accept/reject unit charged by the decoder. -/
  decodeTerminalUnit : Nat
  /-- Units per node of the canonical declarative validation derivation. -/
  validationNodeUnit : Nat
  /-- Units per premise edge of that derivation. -/
  validationEdgeUnit : Nat
  /-- `wasmRuleSteps` charged by one relational Core `Step`. -/
  ruleStepUnit : Nat
  /-- `preparationSteps` charged by one raw installation. -/
  installationPreparationUnit : Nat
  /-- `bytesWritten` charged per installed raw byte. -/
  installedByteWriteUnit : Nat
  /-- Byte lanes charged by a whole-vector shuffle. -/
  wholeVectorShuffleLanes : Nat
  /-- The pinned abstract GC and exception widths. -/
  layout : GcLayoutConstants
  /-- One row per pinned Core rule identifier. -/
  ruleRows : List CostRuleRow
  /-- One row per harness initialization event identifier. -/
  initializationRows : List CostRuleRow
  deriving DecidableEq, Repr, Inhabited

namespace CostTableBody

def unitNats (t : CostTableBody) : List Nat :=
  [ t.decodeUnitPerByte, t.decodeTerminalUnit, t.validationNodeUnit
  , t.validationEdgeUnit, t.ruleStepUnit, t.installationPreparationUnit
  , t.installedByteWriteUnit, t.wholeVectorShuffleLanes ]

def bytes (t : CostTableBody) : List UInt8 :=
  Enc.natsBytes (unitNats t) ++
    (GcLayoutConstants.bytes t.layout ++
      (CostRuleRow.listBytes t.ruleRows ++
        CostRuleRow.listBytes t.initializationRows))

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Enc.natsBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := GcLayoutConstants.bytes_prefixFree _ _ _ _ h
  obtain ⟨h3, h⟩ := CostRuleRow.listBytes_prefixFree _ _ _ _ h
  obtain ⟨h4, h⟩ := CostRuleRow.listBytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y
  simp only [unitNats, List.cons.injEq, and_true] at h1
  obtain ⟨e1, e2, e3, e4, e5, e6, e7, e8⟩ := h1
  subst e1; subst e2; subst e3; subst e4; subst e5; subst e6; subst e7; subst e8
  simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

/-- The frozen canonical schema of a cost table body. -/
def identitySchema : CanonicalSchema CostTableBody :=
  CanonicalSchema.ofPrefixFree 1 .costObjective
    (TypeTag.leaf (Enc.nameBytes "wasm.cost-table.body/1"))
    (TypeTag.leaf_size_pos _)
    bytes bytes_prefixFree

/-- The cost-semantics identity of SPEC section 7.5. -/
def identity (t : CostTableBody) : CanonicalObjectId :=
  CanonicalObjectId.ofTyped (Identity identitySchema t)

theorem identity_eq_iff {a b : CostTableBody} : identity a = identity b ↔ a = b :=
  CanonicalObjectId.ofTyped_Identity_eq_iff identitySchema

/-! ### The contribution law -/

/-- SPEC section 7.5: one unit per consumed byte plus one terminal
accept/reject unit, independent of decoder implementation. -/
def decodeCost (t : CostTableBody) (bytes : ByteArray) : Nat :=
  t.decodeUnitPerByte * bytes.size + t.decodeTerminalUnit

/-- SPEC section 7.5: one unit per node plus one unit per premise edge of the
unique canonical declarative-validation derivation. -/
def validationCost (t : CostTableBody) (nodes edges : Nat) : Nat :=
  t.validationNodeUnit * nodes + t.validationEdgeUnit * edges

/-- SPEC section 7.5: every relational Core `Step` contributes one
`wasmRuleSteps` unit. -/
def stepCost (t : CostTableBody) : Nat := t.ruleStepUnit

/-- SPEC section 7.5: raw installation contributes one `preparationSteps` and
one `bytesWritten` unit per installed byte. -/
def installationCost (t : CostTableBody) (installedBytes : Nat) : Nat × Nat :=
  (t.installationPreparationUnit, t.installedByteWriteUnit * installedBytes)

/-- Lookup of the row for a pinned rule identifier. -/
def rowFor? (t : CostTableBody) (ruleId : String) : Option CostRuleRow :=
  t.ruleRows.find? (fun row => row.ruleId == ruleId)

theorem rowFor?_ruleId {t : CostTableBody} {ruleId : String} {row : CostRuleRow}
    (h : t.rowFor? ruleId = some row) : row.ruleId = ruleId := by
  have hf := List.find?_some (p := fun r => r.ruleId == ruleId) (l := t.ruleRows)
    (by simpa [rowFor?] using h)
  simpa using hf

theorem rowFor?_mem {t : CostTableBody} {ruleId : String} {row : CostRuleRow}
    (h : t.rowFor? ruleId = some row) : row ∈ t.ruleRows :=
  List.mem_of_find?_eq_some (by simpa [rowFor?] using h)

/-- With duplicate-free rule identifiers, a row in the table is *the* row its
identifier looks up: no rule identifier can be charged two different costs. -/
theorem find?_rule_eq_of_mem (rows : List CostRuleRow)
    (hnd : (rows.map CostRuleRow.ruleId).Nodup) {row : CostRuleRow}
    (hmem : row ∈ rows) :
    rows.find? (fun r => r.ruleId == row.ruleId) = some row := by
  induction rows with
  | nil => exact absurd hmem (by simp)
  | cons head tail ih =>
    rw [List.map_cons] at hnd
    have hnd' := List.nodup_cons.mp hnd
    rcases List.mem_cons.mp hmem with hh | ht
    · subst hh
      simp
    · have hne : ¬ ((fun r : CostRuleRow => r.ruleId == row.ruleId) head) = true := by
        simp only [beq_iff_eq]
        intro heq
        have hin : row.ruleId ∈ List.map CostRuleRow.ruleId tail :=
          List.mem_map_of_mem ht
        rw [← heq] at hin
        exact hnd'.1 hin
      rw [show List.find? (fun r : CostRuleRow => r.ruleId == row.ruleId)
              (head :: tail)
            = List.find? (fun r : CostRuleRow => r.ruleId == row.ruleId) tail from
          List.find?_cons_of_neg hne]
      exact ih hnd'.2 ht

theorem rowFor?_eq_of_mem {t : CostTableBody}
    (hnd : (t.ruleRows.map CostRuleRow.ruleId).Nodup)
    {row : CostRuleRow} (hmem : row ∈ t.ruleRows) :
    t.rowFor? row.ruleId = some row :=
  find?_rule_eq_of_mem t.ruleRows hnd hmem

/-- Lookup of the row for a pinned harness initialization event identifier. -/
def initRowFor? (t : CostTableBody) (eventId : String) : Option CostRuleRow :=
  t.initializationRows.find? (fun row => row.ruleId == eventId)

theorem initRowFor?_ruleId {t : CostTableBody} {eventId : String}
    {row : CostRuleRow} (h : t.initRowFor? eventId = some row) :
    row.ruleId = eventId := by
  have hf := List.find?_some (p := fun r => r.ruleId == eventId)
    (l := t.initializationRows) (by simpa [initRowFor?] using h)
  simpa using hf

theorem initRowFor?_mem {t : CostTableBody} {eventId : String}
    {row : CostRuleRow} (h : t.initRowFor? eventId = some row) :
    row ∈ t.initializationRows :=
  List.mem_of_find?_eq_some (by simpa [initRowFor?] using h)

theorem initRowFor?_eq_of_mem {t : CostTableBody}
    (hnd : (t.initializationRows.map CostRuleRow.ruleId).Nodup)
    {row : CostRuleRow} (hmem : row ∈ t.initializationRows) :
    t.initRowFor? row.ruleId = some row :=
  find?_rule_eq_of_mem t.initializationRows hnd hmem

end CostTableBody

/-! ## Profile body components -/

/-- Which semantic layer an identity names. -/
inductive SemanticsLayer
  | decoding | validation | execution | costed
  deriving DecidableEq, Repr, Inhabited

namespace SemanticsLayer

def tag : SemanticsLayer → UInt8
  | .decoding => 0 | .validation => 1 | .execution => 2 | .costed => 3

theorem tag_injective : Function.Injective tag := by
  intro a b h
  cases a <;> cases b <;> first | rfl | exact absurd h (by decide)

def bytes (l : SemanticsLayer) : List UInt8 := Bytes.u8Bytes (tag l)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.u8Bytes_prefixFree.comp tag_injective

end SemanticsLayer

/-- The first-order body identifying one semantic rule set: which pinned
revision it transcribes, which layer it governs, and its rule-set version. -/
structure SemanticsIdentityBody where
  revisionCommit : String
  layer : SemanticsLayer
  ruleSetVersion : Nat
  deriving DecidableEq, Repr, Inhabited

namespace SemanticsIdentityBody

def bytes (s : SemanticsIdentityBody) : List UInt8 :=
  Enc.stringBytes s.revisionCommit ++
    (SemanticsLayer.bytes s.layer ++ Bytes.natBytes s.ruleSetVersion)

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Enc.stringBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := SemanticsLayer.bytes_prefixFree _ _ _ _ h
  obtain ⟨h3, h⟩ := Bytes.natBytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end SemanticsIdentityBody

/-- The canonical semantics identity for one layer of the pinned revision. -/
def canonicalSemanticsIdentity (layer : SemanticsLayer) : SemanticsIdentityBody :=
  { revisionCommit := core3RevisionCommit, layer := layer, ruleSetVersion := 1 }

/-- A permitted import. -/
structure ImportRequirement where
  moduleName : String
  itemName : String
  externType : ExternType
  deriving DecidableEq, Repr, Inhabited

namespace ImportRequirement

def bytes (i : ImportRequirement) : List UInt8 :=
  Enc.stringBytes i.moduleName ++
    (Enc.stringBytes i.itemName ++ ExternType.bytes i.externType)

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Enc.stringBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := Enc.stringBytes_prefixFree _ _ _ _ h
  obtain ⟨h3, h⟩ := ExternType.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

def listBytes (l : List ImportRequirement) : List UInt8 :=
  Bytes.listBytes bytes l

theorem listBytes_prefixFree : Bytes.PrefixFree listBytes :=
  Bytes.listBytes_prefixFree bytes_prefixFree

end ImportRequirement

/-- The import policy.  SPEC section 7.2: the release profile is closed --- no
host functions, clocks, filesystem, network, accelerator, environment query,
shared mutable host state, or unmodeled import.  Each of those is an explicit
first-order flag so that closure is checkable rather than implied. -/
structure ImportPolicy where
  permitted : List ImportRequirement
  hostFunctions : Bool
  clocks : Bool
  filesystem : Bool
  network : Bool
  accelerators : Bool
  environmentQuery : Bool
  sharedMutableHostState : Bool
  deriving DecidableEq, Repr, Inhabited

namespace ImportPolicy

def bytes (p : ImportPolicy) : List UInt8 :=
  ImportRequirement.listBytes p.permitted ++
    (Bytes.boolBytes p.hostFunctions ++
      (Bytes.boolBytes p.clocks ++
        (Bytes.boolBytes p.filesystem ++
          (Bytes.boolBytes p.network ++
            (Bytes.boolBytes p.accelerators ++
              (Bytes.boolBytes p.environmentQuery ++
                Bytes.boolBytes p.sharedMutableHostState))))))

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := ImportRequirement.listBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := Bytes.boolBytes_prefixFree _ _ _ _ h
  obtain ⟨h3, h⟩ := Bytes.boolBytes_prefixFree _ _ _ _ h
  obtain ⟨h4, h⟩ := Bytes.boolBytes_prefixFree _ _ _ _ h
  obtain ⟨h5, h⟩ := Bytes.boolBytes_prefixFree _ _ _ _ h
  obtain ⟨h6, h⟩ := Bytes.boolBytes_prefixFree _ _ _ _ h
  obtain ⟨h7, h⟩ := Bytes.boolBytes_prefixFree _ _ _ _ h
  obtain ⟨h8, h⟩ := Bytes.boolBytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end ImportPolicy

/-- The closed release import policy: nothing imported, no host surface. -/
def closedImportPolicy : ImportPolicy :=
  { permitted := []
    hostFunctions := false
    clocks := false
    filesystem := false
    network := false
    accelerators := false
    environmentQuery := false
    sharedMutableHostState := false }

theorem closedImportPolicy_empty : closedImportPolicy.permitted = [] := rfl

/-- The kind of item a required export must be. -/
inductive ExportKindRequirement
  | memory
  | function (type : FuncType)
  deriving DecidableEq, Repr, Inhabited

namespace ExportKindRequirement

def toSum : ExportKindRequirement → Unit ⊕ FuncType
  | .memory => .inl ()
  | .function t => .inr t

theorem toSum_injective : Function.Injective toSum := by
  intro a b h
  cases a <;> cases b <;> simp_all [toSum]

def bytes (k : ExportKindRequirement) : List UInt8 :=
  Bytes.sumBytes Enc.unitBytes FuncType.bytes (toSum k)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.sumBytes_prefixFree Enc.unitBytes_prefixFree
    FuncType.bytes_prefixFree).comp toSum_injective

end ExportKindRequirement

/-- One required export of the profile's ABI. -/
structure ExportRequirement where
  name : String
  kind : ExportKindRequirement
  deriving DecidableEq, Repr, Inhabited

namespace ExportRequirement

def bytes (e : ExportRequirement) : List UInt8 :=
  Enc.stringBytes e.name ++ ExportKindRequirement.bytes e.kind

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Enc.stringBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := ExportKindRequirement.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

def listBytes (l : List ExportRequirement) : List UInt8 :=
  Bytes.listBytes bytes l

theorem listBytes_prefixFree : Bytes.PrefixFree listBytes :=
  Bytes.listBytes_prefixFree bytes_prefixFree

end ExportRequirement

/-- SPEC section 7.2: one memory named `memory` and one function named `gemm`
with ABI type `(i32, i32) -> i32`. -/
def requiredReleaseExports : List ExportRequirement :=
  [ { name := "memory", kind := .memory }
  , { name := "gemm", kind := .function gemmFuncType } ]

theorem requiredReleaseExports_memory :
    { name := "memory", kind := ExportKindRequirement.memory } ∈
      requiredReleaseExports := by simp [requiredReleaseExports]

theorem requiredReleaseExports_gemm :
    { name := "gemm", kind := ExportKindRequirement.function gemmFuncType } ∈
      requiredReleaseExports := by simp [requiredReleaseExports]

theorem requiredReleaseExports_length : requiredReleaseExports.length = 2 := rfl

theorem requiredReleaseExports_names_nodup :
    (requiredReleaseExports.map ExportRequirement.name).Nodup := by decide

/-- The module, memory, table, managed-heap, stack and invocation limits bound
by the profile (SPEC section 7.2). -/
structure ResourceLimits where
  maxModuleBytes : Nat
  maxTypes : Nat
  maxFunctions : Nat
  maxTables : Nat
  maxMemories : Nat
  maxGlobals : Nat
  maxTags : Nat
  maxElementSegments : Nat
  maxDataSegments : Nat
  maxImports : Nat
  maxExports : Nat
  maxTableElements : Nat
  maxLocals : Nat
  maxStackValues : Nat
  maxManagedHeapBytes : Nat
  deriving DecidableEq, Repr, Inhabited

namespace ResourceLimits

def toNats (l : ResourceLimits) : List Nat :=
  [ l.maxModuleBytes, l.maxTypes, l.maxFunctions, l.maxTables, l.maxMemories
  , l.maxGlobals, l.maxTags, l.maxElementSegments, l.maxDataSegments
  , l.maxImports, l.maxExports, l.maxTableElements, l.maxLocals
  , l.maxStackValues, l.maxManagedHeapBytes ]

theorem toNats_injective : Function.Injective toNats := by
  intro x y h
  cases x; cases y
  simp only [toNats, List.cons.injEq, and_true] at h
  obtain ⟨e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15⟩ := h
  subst e1; subst e2; subst e3; subst e4; subst e5; subst e6; subst e7
  subst e8; subst e9; subst e10; subst e11; subst e12; subst e13; subst e14
  subst e15
  rfl

def bytes (l : ResourceLimits) : List UInt8 := Enc.natsBytes (toNats l)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Enc.natsBytes_prefixFree.comp toNats_injective

end ResourceLimits

/-- Every count in the pinned Core binary format is a `u32`, so `2^32 - 1` is
the normative ceiling for each index space and byte count.  The profile fixes
the managed-heap and stack ceilings at the same value, so that every charged
resource of the release profile is uniformly bounded by the wasm32 address
space. -/
def wasm32Ceiling : Nat := 2 ^ 32 - 1

theorem wasm32Ceiling_eq : wasm32Ceiling = 4294967295 := by decide

/-- The release resource limits. -/
def canonicalResourceLimits : ResourceLimits :=
  { maxModuleBytes := wasm32Ceiling
    maxTypes := wasm32Ceiling
    maxFunctions := wasm32Ceiling
    maxTables := wasm32Ceiling
    maxMemories := wasm32Ceiling
    maxGlobals := wasm32Ceiling
    maxTags := wasm32Ceiling
    maxElementSegments := wasm32Ceiling
    maxDataSegments := wasm32Ceiling
    maxImports := wasm32Ceiling
    maxExports := wasm32Ceiling
    maxTableElements := wasm32Ceiling
    maxLocals := wasm32Ceiling
    maxStackValues := wasm32Ceiling
    maxManagedHeapBytes := wasm32Ceiling }

/-- The permitted nondeterminism of a profile.  SPEC section 7.2 permits
exactly the numeric nondeterminism of the pinned semantics. -/
inductive NondeterminismPolicy
  /-- Exactly the relation in the pinned Core semantics; nothing else. -/
  | pinnedCoreNumericOnly
  /-- Host-extended nondeterminism.  Expressible, and rejected by the release
  profile. -/
  | hostExtended
  deriving DecidableEq, Repr, Inhabited

namespace NondeterminismPolicy

def tag : NondeterminismPolicy → UInt8
  | .pinnedCoreNumericOnly => 0
  | .hostExtended => 1

theorem tag_injective : Function.Injective tag := by
  intro a b h
  cases a <;> cases b <;> first | rfl | exact absurd h (by decide)

def bytes (p : NondeterminismPolicy) : List UInt8 := Bytes.u8Bytes (tag p)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.u8Bytes_prefixFree.comp tag_injective

end NondeterminismPolicy

/-! ## The profile body -/

/-- The first-order profile body (SPEC section 6.2's mandated split): no
functions, no proofs, no conclusions. -/
structure ProfileBody where
  /-- The pinned WebAssembly Core revision commit. -/
  revisionCommit : String
  /-- Address width: 32 for the release wasm32 profile. -/
  addressBits : Nat
  /-- The normative memory page limit. -/
  maxPages : Nat
  /-- The largest raw invocation the profile admits. -/
  maxInvocationBytes : Nat
  /-- The complete Core 3.0 feature matrix, as first-order rows. -/
  featureTable : List FeatureRow
  /-- The import policy. -/
  importPolicy : ImportPolicy
  /-- The exported ABI the profile requires. -/
  requiredExports : List ExportRequirement
  /-- Module, table, managed-heap and stack limits. -/
  limits : ResourceLimits
  /-- The permitted nondeterminism. -/
  nondeterminism : NondeterminismPolicy
  /-- The decoding semantics identity. -/
  decodingSemantics : SemanticsIdentityBody
  /-- The validation semantics identity. -/
  validationSemantics : SemanticsIdentityBody
  /-- The execution semantics identity. -/
  executionSemantics : SemanticsIdentityBody
  /-- The cost semantics: one first-order cost table body. -/
  costTableBody : CostTableBody
  deriving DecidableEq, Repr, Inhabited

namespace ProfileBody

/-- Prefix-free canonical encoding of the profile body. -/
def bytes (b : ProfileBody) : List UInt8 :=
  Enc.stringBytes b.revisionCommit ++
    (Bytes.natBytes b.addressBits ++
      (Bytes.natBytes b.maxPages ++
        (Bytes.natBytes b.maxInvocationBytes ++
          (featureTableBytes b.featureTable ++
            (ImportPolicy.bytes b.importPolicy ++
              (ExportRequirement.listBytes b.requiredExports ++
                (ResourceLimits.bytes b.limits ++
                  (NondeterminismPolicy.bytes b.nondeterminism ++
                    (SemanticsIdentityBody.bytes b.decodingSemantics ++
                      (SemanticsIdentityBody.bytes b.validationSemantics ++
                        (SemanticsIdentityBody.bytes b.executionSemantics ++
                          CostTableBody.bytes b.costTableBody)))))))))))

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Enc.stringBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := Bytes.natBytes_prefixFree _ _ _ _ h
  obtain ⟨h3, h⟩ := Bytes.natBytes_prefixFree _ _ _ _ h
  obtain ⟨h4, h⟩ := Bytes.natBytes_prefixFree _ _ _ _ h
  obtain ⟨h5, h⟩ := featureTableBytes_prefixFree _ _ _ _ h
  obtain ⟨h6, h⟩ := ImportPolicy.bytes_prefixFree _ _ _ _ h
  obtain ⟨h7, h⟩ := ExportRequirement.listBytes_prefixFree _ _ _ _ h
  obtain ⟨h8, h⟩ := ResourceLimits.bytes_prefixFree _ _ _ _ h
  obtain ⟨h9, h⟩ := NondeterminismPolicy.bytes_prefixFree _ _ _ _ h
  obtain ⟨h10, h⟩ := SemanticsIdentityBody.bytes_prefixFree _ _ _ _ h
  obtain ⟨h11, h⟩ := SemanticsIdentityBody.bytes_prefixFree _ _ _ _ h
  obtain ⟨h12, h⟩ := SemanticsIdentityBody.bytes_prefixFree _ _ _ _ h
  obtain ⟨h13, h⟩ := CostTableBody.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

/-- The frozen canonical schema of the profile body (SPEC section 6.2). -/
def identitySchema : CanonicalSchema ProfileBody :=
  CanonicalSchema.ofPrefixFree 1 .wasmProfile
    (TypeTag.leaf (Enc.nameBytes "wasm.profile.body/1"))
    (TypeTag.leaf_size_pos _)
    bytes bytes_prefixFree

theorem identitySchema_domain :
    identitySchema.domain = CanonicalDomainTag.wasmProfile := rfl

end ProfileBody

/-! ## Lawfulness

Every requirement SPEC section 7.2 places on the release profile, as one
decidable proposition about the first-order body.  No conclusion of the release
theorem appears here. -/

/-- The release profile's lawfulness conditions (SPEC sections 7.2 and 7.5). -/
def ProfileLawful (body : ProfileBody) : Prop :=
  body.revisionCommit = core3RevisionCommit ∧
  body.addressBits = 32 ∧
  body.maxPages = 65536 ∧
  body.maxInvocationBytes = wasm32Ceiling ∧
  body.featureTable = releaseFeatureTable ∧
  body.importPolicy = closedImportPolicy ∧
  body.requiredExports = requiredReleaseExports ∧
  body.limits = canonicalResourceLimits ∧
  body.nondeterminism = NondeterminismPolicy.pinnedCoreNumericOnly ∧
  body.decodingSemantics = canonicalSemanticsIdentity .decoding ∧
  body.validationSemantics = canonicalSemanticsIdentity .validation ∧
  body.executionSemantics = canonicalSemanticsIdentity .execution ∧
  body.costTableBody.layout = canonicalGcLayout ∧
  body.costTableBody.decodeUnitPerByte = 1 ∧
  body.costTableBody.decodeTerminalUnit = 1 ∧
  body.costTableBody.validationNodeUnit = 1 ∧
  body.costTableBody.validationEdgeUnit = 1 ∧
  body.costTableBody.ruleStepUnit = 1 ∧
  body.costTableBody.installationPreparationUnit = 1 ∧
  body.costTableBody.installedByteWriteUnit = 1 ∧
  body.costTableBody.wholeVectorShuffleLanes = 16 ∧
  (body.costTableBody.ruleRows.map CostRuleRow.ruleId).Nodup ∧
  (body.costTableBody.initializationRows.map CostRuleRow.ruleId).Nodup

instance instDecidableProfileLawful (body : ProfileBody) :
    Decidable (ProfileLawful body) := by
  unfold ProfileLawful; exact inferInstance

namespace ProfileLawful

variable {body : ProfileBody}

theorem revisionCommit (h : ProfileLawful body) :
    body.revisionCommit = core3RevisionCommit := h.1

theorem addressBits (h : ProfileLawful body) : body.addressBits = 32 := h.2.1

theorem maxPages (h : ProfileLawful body) : body.maxPages = 65536 := h.2.2.1

theorem maxInvocationBytes (h : ProfileLawful body) :
    body.maxInvocationBytes = wasm32Ceiling := h.2.2.2.1

theorem featureTable (h : ProfileLawful body) :
    body.featureTable = releaseFeatureTable := h.2.2.2.2.1

theorem importPolicy (h : ProfileLawful body) :
    body.importPolicy = closedImportPolicy := h.2.2.2.2.2.1

theorem requiredExports (h : ProfileLawful body) :
    body.requiredExports = requiredReleaseExports := h.2.2.2.2.2.2.1

theorem limits (h : ProfileLawful body) :
    body.limits = canonicalResourceLimits := h.2.2.2.2.2.2.2.1

theorem nondeterminism (h : ProfileLawful body) :
    body.nondeterminism = NondeterminismPolicy.pinnedCoreNumericOnly :=
  h.2.2.2.2.2.2.2.2.1

theorem decodingSemantics (h : ProfileLawful body) :
    body.decodingSemantics = canonicalSemanticsIdentity .decoding :=
  h.2.2.2.2.2.2.2.2.2.1

theorem validationSemantics (h : ProfileLawful body) :
    body.validationSemantics = canonicalSemanticsIdentity .validation :=
  h.2.2.2.2.2.2.2.2.2.2.1

theorem executionSemantics (h : ProfileLawful body) :
    body.executionSemantics = canonicalSemanticsIdentity .execution :=
  h.2.2.2.2.2.2.2.2.2.2.2.1

theorem layout (h : ProfileLawful body) :
    body.costTableBody.layout = canonicalGcLayout :=
  h.2.2.2.2.2.2.2.2.2.2.2.2.1

theorem decodeUnitPerByte (h : ProfileLawful body) :
    body.costTableBody.decodeUnitPerByte = 1 :=
  h.2.2.2.2.2.2.2.2.2.2.2.2.2.1

theorem decodeTerminalUnit (h : ProfileLawful body) :
    body.costTableBody.decodeTerminalUnit = 1 :=
  h.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1

theorem validationNodeUnit (h : ProfileLawful body) :
    body.costTableBody.validationNodeUnit = 1 :=
  h.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1

theorem validationEdgeUnit (h : ProfileLawful body) :
    body.costTableBody.validationEdgeUnit = 1 :=
  h.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1

theorem ruleStepUnit (h : ProfileLawful body) :
    body.costTableBody.ruleStepUnit = 1 :=
  h.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1

theorem installationPreparationUnit (h : ProfileLawful body) :
    body.costTableBody.installationPreparationUnit = 1 :=
  h.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1

theorem installedByteWriteUnit (h : ProfileLawful body) :
    body.costTableBody.installedByteWriteUnit = 1 :=
  h.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1

theorem wholeVectorShuffleLanes (h : ProfileLawful body) :
    body.costTableBody.wholeVectorShuffleLanes = 16 :=
  h.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1

theorem ruleRows_nodup (h : ProfileLawful body) :
    (body.costTableBody.ruleRows.map CostRuleRow.ruleId).Nodup :=
  h.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1

theorem initializationRows_nodup (h : ProfileLawful body) :
    (body.costTableBody.initializationRows.map CostRuleRow.ruleId).Nodup :=
  h.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2

end ProfileLawful

/-! ## The profile -/

/-- SPEC section 6.2: a profile is a first-order body plus a proof that it is
lawful.  The lawfulness is never stored as data. -/
structure Profile where
  body : ProfileBody
  lawful : ProfileLawful body

namespace Profile

/-- Build a profile from a body whose lawfulness is decided. -/
def checked (body : ProfileBody) (lawful : ProfileLawful body := by decide) :
    Profile :=
  { body := body, lawful := lawful }

@[simp] theorem checked_body (body : ProfileBody) (h : ProfileLawful body) :
    (checked body h).body = body := rfl

/-- Executable check: a body is admitted exactly when it is lawful. -/
def check? (body : ProfileBody) : Option Profile :=
  if h : ProfileLawful body then some { body := body, lawful := h } else none

theorem check?_isSome_iff (body : ProfileBody) :
    (check? body).isSome = true ↔ ProfileLawful body := by
  unfold check?
  by_cases h : ProfileLawful body <;> simp [h]

theorem check?_body {body : ProfileBody} {profile : Profile}
    (h : check? body = some profile) : profile.body = body := by
  unfold check? at h
  by_cases hl : ProfileLawful body
  · rw [dif_pos hl] at h
    cases h
    rfl
  · rw [dif_neg hl] at h
    exact absurd h (by simp)

/-- Two profiles with the same body are equal: the lawfulness proof is
proof-irrelevant, so the body is the whole identity. -/
theorem eq_of_body_eq {p q : Profile} (h : p.body = q.body) : p = q := by
  cases p; cases q
  cases h
  rfl

def costTableBody (profile : Profile) : CostTableBody :=
  profile.body.costTableBody

def addressBits (profile : Profile) : Nat := profile.body.addressBits

def maxPages (profile : Profile) : Nat := profile.body.maxPages

def maxInvocationBytes (profile : Profile) : Nat :=
  profile.body.maxInvocationBytes

def featureTable (profile : Profile) : List FeatureRow :=
  profile.body.featureTable

/-- The cost-semantics identity of the profile (SPEC section 7.5). -/
def costSemanticsId (profile : Profile) : CanonicalObjectId :=
  CostTableBody.identity profile.costTableBody

theorem addressBits_eq (profile : Profile) : profile.addressBits = 32 :=
  profile.lawful.addressBits

theorem maxPages_eq (profile : Profile) : profile.maxPages = 65536 :=
  profile.lawful.maxPages

theorem maxInvocationBytes_eq (profile : Profile) :
    profile.maxInvocationBytes = wasm32Ceiling :=
  profile.lawful.maxInvocationBytes

/-- Every profile is wasm32: SPEC section 7.2's address model is not an
implementation choice. -/
theorem addressSpace (profile : Profile) : 2 ^ profile.addressBits = 2 ^ 32 := by
  rw [profile.addressBits_eq]

/-- The feature decision of a profile: read off the stored table. -/
def decisionFor (profile : Profile) (f : FeatureFamily) :
    Option FeatureDecision :=
  lookupDecision profile.featureTable f

/-- No feature defaults (SPEC section 7.2): every family of the Core 3.0 matrix
has a stored decision, and it is exactly the release decision. -/
theorem decisionFor_eq (profile : Profile) (f : FeatureFamily) :
    profile.decisionFor f = some (releaseDecision f) := by
  unfold decisionFor featureTable
  rw [profile.lawful.featureTable]
  exact lookupDecision_releaseFeatureTable f

theorem decisionFor_isSome (profile : Profile) (f : FeatureFamily) :
    (profile.decisionFor f).isSome = true := by
  rw [profile.decisionFor_eq]; rfl

/-! ### The feature matrix of SPEC section 7.2, row by row -/

theorem scalarCore_enabled (profile : Profile) :
    profile.decisionFor .scalarCore = some .enabled := profile.decisionFor_eq _

theorem multiValueAndExtendedConst_enabled (profile : Profile) :
    profile.decisionFor .multiValueAndExtendedConst = some .enabled :=
  profile.decisionFor_eq _

theorem bulkMemoryMultipleMemoriesTables_enabled (profile : Profile) :
    profile.decisionFor .bulkMemoryMultipleMemoriesTables = some .enabled :=
  profile.decisionFor_eq _

theorem fixedWidthSimd128_enabled (profile : Profile) :
    profile.decisionFor .fixedWidthSimd128 = some .enabled :=
  profile.decisionFor_eq _

theorem referenceTypesTypedFunctionReferencesGc_enabled (profile : Profile) :
    profile.decisionFor .referenceTypesTypedFunctionReferencesGc =
      some .enabled :=
  profile.decisionFor_eq _

theorem tailCalls_enabled (profile : Profile) :
    profile.decisionFor .tailCalls = some .enabled := profile.decisionFor_eq _

theorem exceptionHandling_enabled (profile : Profile) :
    profile.decisionFor .exceptionHandling = some .enabled :=
  profile.decisionFor_eq _

theorem memory64_rejected (profile : Profile) :
    profile.decisionFor .memory64 = some .rejected := profile.decisionFor_eq _

theorem sharedMemoriesAtomicsThreads_rejected (profile : Profile) :
    profile.decisionFor .sharedMemoriesAtomicsThreads = some .rejected :=
  profile.decisionFor_eq _

theorem relaxedSimd_rejected (profile : Profile) :
    profile.decisionFor .relaxedSimd = some .rejected := profile.decisionFor_eq _

theorem componentModelAndPostCore3_rejected (profile : Profile) :
    profile.decisionFor .componentModelAndPostCore3 = some .rejected :=
  profile.decisionFor_eq _

/-! ### The pinned revision

This is *not* `profile_matches_pinned_revision` of SPEC section 15: that name
additionally requires every enabled vendored rule to have exactly one mapped
Lean declaration, which needs the conformance map and the concrete semantics.
These theorems state only what this layer can prove: the profile body and each
of its semantics identities carry the pinned commit. -/

theorem revisionCommit_eq (profile : Profile) :
    profile.body.revisionCommit = core3Revision.commit :=
  profile.lawful.revisionCommit

theorem decodingSemantics_revision (profile : Profile) :
    profile.body.decodingSemantics.revisionCommit = core3Revision.commit := by
  rw [profile.lawful.decodingSemantics]
  rfl

theorem validationSemantics_revision (profile : Profile) :
    profile.body.validationSemantics.revisionCommit = core3Revision.commit := by
  rw [profile.lawful.validationSemantics]
  rfl

theorem executionSemantics_revision (profile : Profile) :
    profile.body.executionSemantics.revisionCommit = core3Revision.commit := by
  rw [profile.lawful.executionSemantics]
  rfl

/-- The three semantics identities are distinct: a profile cannot satisfy its
validation obligation with its decoding rule set. -/
theorem semantics_identities_distinct (profile : Profile) :
    profile.body.decodingSemantics ≠ profile.body.validationSemantics ∧
    profile.body.validationSemantics ≠ profile.body.executionSemantics ∧
    profile.body.decodingSemantics ≠ profile.body.executionSemantics := by
  rw [profile.lawful.decodingSemantics, profile.lawful.validationSemantics,
    profile.lawful.executionSemantics]
  refine ⟨?_, ?_, ?_⟩ <;> simp [canonicalSemanticsIdentity]

/-- The profile is closed: no import is permitted and no host surface is
exposed (SPEC section 7.2). -/
theorem imports_empty (profile : Profile) :
    profile.body.importPolicy.permitted = [] := by
  rw [profile.lawful.importPolicy]
  rfl

theorem no_host_functions (profile : Profile) :
    profile.body.importPolicy.hostFunctions = false := by
  rw [profile.lawful.importPolicy]
  rfl

theorem no_clocks (profile : Profile) :
    profile.body.importPolicy.clocks = false := by
  rw [profile.lawful.importPolicy]
  rfl

theorem no_filesystem (profile : Profile) :
    profile.body.importPolicy.filesystem = false := by
  rw [profile.lawful.importPolicy]
  rfl

theorem no_network (profile : Profile) :
    profile.body.importPolicy.network = false := by
  rw [profile.lawful.importPolicy]
  rfl

theorem no_accelerators (profile : Profile) :
    profile.body.importPolicy.accelerators = false := by
  rw [profile.lawful.importPolicy]
  rfl

theorem no_environment_query (profile : Profile) :
    profile.body.importPolicy.environmentQuery = false := by
  rw [profile.lawful.importPolicy]
  rfl

theorem no_shared_host_state (profile : Profile) :
    profile.body.importPolicy.sharedMutableHostState = false := by
  rw [profile.lawful.importPolicy]
  rfl

/-- The required exports are exactly the memory `memory` and the function
`gemm : (i32, i32) -> i32`. -/
theorem requiredExports_eq (profile : Profile) :
    profile.body.requiredExports = requiredReleaseExports :=
  profile.lawful.requiredExports

/-- Only the pinned numeric nondeterminism is permitted. -/
theorem nondeterminism_pinned (profile : Profile) :
    profile.body.nondeterminism = NondeterminismPolicy.pinnedCoreNumericOnly :=
  profile.lawful.nondeterminism

/-! ### The cost contribution law holds for every profile (SPEC section 7.5) -/

/-- `decodeCost bytes = bytes.size + 1`. -/
theorem decodeCost_eq (profile : Profile) (bytes : ByteArray) :
    profile.costTableBody.decodeCost bytes = bytes.size + 1 := by
  unfold CostTableBody.decodeCost costTableBody
  rw [profile.lawful.decodeUnitPerByte, profile.lawful.decodeTerminalUnit,
    Nat.one_mul]

/-- One unit per derivation node plus one unit per premise edge. -/
theorem validationCost_eq (profile : Profile) (nodes edges : Nat) :
    profile.costTableBody.validationCost nodes edges = nodes + edges := by
  unfold CostTableBody.validationCost costTableBody
  rw [profile.lawful.validationNodeUnit, profile.lawful.validationEdgeUnit,
    Nat.one_mul, Nat.one_mul]

/-- Every relational Core `Step` contributes exactly one `wasmRuleSteps`. -/
theorem stepCost_eq (profile : Profile) : profile.costTableBody.stepCost = 1 :=
  profile.lawful.ruleStepUnit

/-- Raw installation contributes one preparation step and one written byte per
installed byte. -/
theorem installationCost_eq (profile : Profile) (installedBytes : Nat) :
    profile.costTableBody.installationCost installedBytes =
      (1, installedBytes) := by
  unfold CostTableBody.installationCost costTableBody
  rw [profile.lawful.installationPreparationUnit,
    profile.lawful.installedByteWriteUnit, Nat.one_mul]

/-- A whole-vector shuffle contributes sixteen byte lanes. -/
theorem wholeVectorShuffleLanes_eq (profile : Profile) :
    profile.costTableBody.wholeVectorShuffleLanes = 16 :=
  profile.lawful.wholeVectorShuffleLanes

/-- The GC and exception widths of any profile are the canonical ones. -/
theorem layout_eq (profile : Profile) :
    profile.costTableBody.layout = canonicalGcLayout :=
  profile.lawful.layout

theorem layout_positive (profile : Profile) :
    profile.costTableBody.layout.Positive := by
  rw [profile.layout_eq]
  exact canonicalGcLayout_positive

/-- No pinned rule identifier is charged two different costs. -/
theorem rule_cost_unique (profile : Profile) {left right : CostRuleRow}
    (hleft : left ∈ profile.costTableBody.ruleRows)
    (hright : right ∈ profile.costTableBody.ruleRows)
    (hid : left.ruleId = right.ruleId) : left = right := by
  have hl := CostTableBody.rowFor?_eq_of_mem profile.lawful.ruleRows_nodup hleft
  have hr := CostTableBody.rowFor?_eq_of_mem profile.lawful.ruleRows_nodup hright
  rw [hid] at hl
  rw [hl] at hr
  exact Option.some.inj hr

end Profile

/-- SPEC section 6.2: the canonical identity of a profile. -/
def ProfileId (profile : Profile) : WasmGemmGnaf.Foundation.ProfileId :=
  CanonicalObjectId.ofTyped (Identity ProfileBody.identitySchema profile.body)

/-- Profiles with equal identities are equal.  Identity is by canonical body,
not by digest: no proof assumes collision resistance. -/
theorem ProfileId_eq_iff (p q : Profile) : ProfileId p = ProfileId q ↔ p = q := by
  unfold ProfileId
  constructor
  · intro h
    exact Profile.eq_of_body_eq
      ((CanonicalObjectId.ofTyped_Identity_eq_iff ProfileBody.identitySchema).mp h)
  · intro h
    rw [h]

theorem ProfileId_domain (profile : Profile) :
    (ProfileId profile).domain = CanonicalDomainTag.wasmProfile := rfl

/-! ## Registry entries

SPEC section 6.2 requires the finite schema registry to retain every schema the
release uses.  These are the `Wasm` layer's entries, together with the proof
that they have pairwise distinct registry keys, so a later registry can include
them without re-proving anything. -/

/-- Entries with different schema domains have different registry keys. -/
theorem key_ne_of_domain_ne {a b : SchemaEntry}
    (h : a.schema.domain ≠ b.schema.domain) : a.key ≠ b.key := by
  intro hk
  exact h (congrArg SchemaKey.domain hk)

/-- The schemas this layer contributes to the release registry. -/
def schemaEntries : List SchemaEntry :=
  [ { Body := RevisionBody, schema := RevisionBody.identitySchema }
  , { Body := CostTableBody, schema := CostTableBody.identitySchema }
  , { Body := ProfileBody, schema := ProfileBody.identitySchema } ]

theorem schemaEntries_distinctKeys :
    schemaEntries.Pairwise (fun a b => a.key ≠ b.key) := by
  refine List.pairwise_cons.mpr ⟨?_, List.pairwise_cons.mpr ⟨?_,
    List.pairwise_cons.mpr ⟨?_, List.Pairwise.nil⟩⟩⟩
  · intro b hb
    simp only [List.mem_cons, List.not_mem_nil, or_false] at hb
    rcases hb with rfl | rfl <;> exact key_ne_of_domain_ne (by decide)
  · intro b hb
    simp only [List.mem_cons, List.not_mem_nil, or_false] at hb
    rcases hb with rfl
    exact key_ne_of_domain_ne (by decide)
  · intro b hb
    exact absurd hb (by simp)

theorem schemaEntries_length : schemaEntries.length = 3 := rfl

/-! ## The canonical release profile body

SPEC section 7.2: "the concrete release carrier is a checked literal, not a
profile parameter chosen by an implementation".  `canonicalCore3Wasm32ProfileBody`
is the proof-free record whose projections are exactly the feature decisions,
limits, imports, exports, nondeterminism, semantics identities and cost
identity of that section.  It has no default-feature or environment
argument. -/
def canonicalCore3Wasm32ProfileBody
    (revisionCommit : String) (costTableBody : CostTableBody) : ProfileBody :=
  { revisionCommit := revisionCommit
    addressBits := 32
    maxPages := 65536
    maxInvocationBytes := wasm32Ceiling
    featureTable := releaseFeatureTable
    importPolicy := closedImportPolicy
    requiredExports := requiredReleaseExports
    limits := canonicalResourceLimits
    nondeterminism := .pinnedCoreNumericOnly
    decodingSemantics := canonicalSemanticsIdentity .decoding
    validationSemantics := canonicalSemanticsIdentity .validation
    executionSemantics := canonicalSemanticsIdentity .execution
    costTableBody := costTableBody }

@[simp] theorem canonicalCore3Wasm32ProfileBody_costTableBody
    (revisionCommit : String) (t : CostTableBody) :
    (canonicalCore3Wasm32ProfileBody revisionCommit t).costTableBody = t := rfl

@[simp] theorem canonicalCore3Wasm32ProfileBody_addressBits
    (revisionCommit : String) (t : CostTableBody) :
    (canonicalCore3Wasm32ProfileBody revisionCommit t).addressBits = 32 := rfl

@[simp] theorem canonicalCore3Wasm32ProfileBody_maxPages
    (revisionCommit : String) (t : CostTableBody) :
    (canonicalCore3Wasm32ProfileBody revisionCommit t).maxPages = 65536 := rfl

/-- The canonical body is lawful exactly when the pinned commit is used and the
cost table carries the canonical units, widths and duplicate-free rows. -/
theorem canonicalCore3Wasm32ProfileBody_lawful (t : CostTableBody)
    (hlayout : t.layout = canonicalGcLayout)
    (hdecodeByte : t.decodeUnitPerByte = 1)
    (hdecodeTerminal : t.decodeTerminalUnit = 1)
    (hnode : t.validationNodeUnit = 1)
    (hedge : t.validationEdgeUnit = 1)
    (hstep : t.ruleStepUnit = 1)
    (hprep : t.installationPreparationUnit = 1)
    (hwrite : t.installedByteWriteUnit = 1)
    (hshuffle : t.wholeVectorShuffleLanes = 16)
    (hrules : (t.ruleRows.map CostRuleRow.ruleId).Nodup)
    (hinit : (t.initializationRows.map CostRuleRow.ruleId).Nodup) :
    ProfileLawful (canonicalCore3Wasm32ProfileBody core3RevisionCommit t) :=
  ⟨rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, hlayout,
    hdecodeByte, hdecodeTerminal, hnode, hedge, hstep, hprep, hwrite, hshuffle,
    hrules, hinit⟩

/-! ### The canonical rule rows (SPEC section 7.5)

SPEC section 7.5 requires one row per pinned Core rule identifier, carrying
that rule's exact contribution.  The rows are literal first-order data here, so
that the cost table is a *checked* object rather than a description of one.

`Wasm/Costed.lean` owns the rule identifiers themselves (`Wasm.RuleId`, one
constructor per constructor of `Wasm.Step`) and the harness initialization
event identifiers (`Wasm.InitEventId`), and proves that the row list below is
exactly a duplicate-free cover of them and that each row's contribution is
exactly the contribution the cost law charges.  Those proofs cannot live here:
`Wasm/Costed.lean` imports this file, not the other way round.  What lives here
is the data and its duplicate-freedom. -/

/-- The charge of one ordinary relational Core `Step` under the canonical
units: one `wasmRuleSteps` unit and nothing else. -/
def canonicalRuleStepContribution : Cost.DynamicVector :=
  { Cost.DynamicVector.zero with wasmRuleSteps := 1 }

/-- The charge of a dispatching rule under the canonical units: one rule step
and one dispatch step.  SPEC section 7.5 restricts `dispatchSteps` to the
branch, branch-table, direct and indirect call, return, exception transfer,
tail-call transfer and harness phase-transition rules. -/
def canonicalDispatchContribution : Cost.DynamicVector :=
  { canonicalRuleStepContribution with dispatchSteps := 1 }

/-- The charge of one initialization step under the canonical units, matching
`Cost.Event.instantiationStep`: one `instantiationSteps` unit and the one
`wasmRuleSteps` unit every semantic transition contributes. -/
def canonicalInstantiationContribution : Cost.DynamicVector :=
  { Cost.DynamicVector.zero with instantiationSteps := 1, wasmRuleSteps := 1 }

/-- The abstract byte width of the `i32` that the memory rules of the declared
subset transfer.  `Wasm.i32TransferBytes_eq` in `Wasm/Costed.lean` proves this
is the length of the ABI image `Wasm.u32ToBytes`. -/
def canonicalTransferBytes : Nat := 4

/-- The abstract width of one exception object under `canonicalGcLayout`: a
16-byte header plus one `i32` payload word rounded to a multiple of eight.
`Wasm.canonical_exceptionObjectBytes` in `Wasm/Costed.lean` proves this equals
`exceptionObjectBytes canonicalGcLayout`. -/
def canonicalExceptionBytes : Nat := 24

/-- A row of the canonical cost table. -/
def canonicalRow (id : String) (v : Cost.DynamicVector) : CostRuleRow :=
  { ruleId := id, contribution := v }

@[simp] theorem canonicalRow_ruleId (id : String) (v : Cost.DynamicVector) :
    (canonicalRow id v).ruleId = id := rfl

@[simp] theorem canonicalRow_contribution (id : String)
    (v : Cost.DynamicVector) : (canonicalRow id v).contribution = v := rfl

/-- **SPEC section 7.5.**  One row per pinned Core rule identifier, in the
pinned order of `Wasm.RuleId.all`.

Reading the coordinates: every row charges exactly one `wasmRuleSteps` unit,
because every row names one relational `Step`.  `dispatchSteps` is one exactly
on the taken branches, the two `if` arms, the return, the exception transfer
and the harness phase transition.  `scalarOps` is one exactly on the three
completed scalar primitive numeric rules and zero on the trapping division.
`bytesRead`/`bytesWritten` carry the exact transferred width, and are zero on
every trapping access: a trap contributes its attempted rule step and no
completed transfer.  `memoryGrowPages` on `memory.grow` and `bytesWritten` on
the harness entry are *per-unit* charges — one per requested page and one per
installed raw byte respectively — because those two rules are the only ones
whose transferred quantity is an operand rather than a constant of the rule;
`Wasm.canonicalCostTable_charges_exactly` states that scaling precisely.

`vectorLaneOps` and `tableElementsAllocated` are zero in every row, and that is
exact rather than lazy: the declared executed subset (`Wasm.Step`) contains no
SIMD rule and no table, element or data rule at all — release validation
rejects modules carrying tables, element segments or data segments
(`Wasm.Module.checkClosed`), and the 128-bit lane charge lives in
`Wasm.wholeVectorShuffleCharge` over `wholeVectorShuffleLanes = 16`, which is
not a `Step` rule. -/
def canonicalRuleRows : List CostRuleRow :=
  [ canonicalRow "core3/step/unreachable" canonicalRuleStepContribution
  , canonicalRow "core3/step/nop" canonicalRuleStepContribution
  , canonicalRow "core3/step/i32.const" canonicalRuleStepContribution
  , canonicalRow "core3/step/drop" canonicalRuleStepContribution
  , canonicalRow "core3/step/i32.binop"
      { canonicalRuleStepContribution with scalarOps := 1 }
  , canonicalRow "core3/step/i32.binop-trap" canonicalRuleStepContribution
  , canonicalRow "core3/step/i32.testop"
      { canonicalRuleStepContribution with scalarOps := 1 }
  , canonicalRow "core3/step/i32.relop"
      { canonicalRuleStepContribution with scalarOps := 1 }
  , canonicalRow "core3/step/local.get" canonicalRuleStepContribution
  , canonicalRow "core3/step/local.set" canonicalRuleStepContribution
  , canonicalRow "core3/step/local.tee" canonicalRuleStepContribution
  , canonicalRow "core3/step/global.get" canonicalRuleStepContribution
  , canonicalRow "core3/step/global.set" canonicalRuleStepContribution
  , canonicalRow "core3/step/i32.load"
      { canonicalRuleStepContribution with bytesRead := canonicalTransferBytes }
  , canonicalRow "core3/step/i32.load-trap" canonicalRuleStepContribution
  , canonicalRow "core3/step/i32.store"
      { canonicalRuleStepContribution with
          bytesWritten := canonicalTransferBytes }
  , canonicalRow "core3/step/i32.store-trap" canonicalRuleStepContribution
  , canonicalRow "core3/step/memory.size" canonicalRuleStepContribution
  , canonicalRow "core3/step/memory.grow-succeed"
      { canonicalRuleStepContribution with memoryGrowPages := 1 }
  , canonicalRow "core3/step/memory.grow-refuse" canonicalRuleStepContribution
  , canonicalRow "core3/step/block" canonicalRuleStepContribution
  , canonicalRow "core3/step/loop" canonicalRuleStepContribution
  , canonicalRow "core3/step/if-false" canonicalDispatchContribution
  , canonicalRow "core3/step/if-true" canonicalDispatchContribution
  , canonicalRow "core3/step/br-loop" canonicalDispatchContribution
  , canonicalRow "core3/step/br-block" canonicalDispatchContribution
  , canonicalRow "core3/step/br_if-false" canonicalRuleStepContribution
  , canonicalRow "core3/step/br_if-loop" canonicalDispatchContribution
  , canonicalRow "core3/step/br_if-block" canonicalDispatchContribution
  , canonicalRow "core3/step/throw"
      { canonicalDispatchContribution with
          gcObjectsAllocated := 1
          gcBytesInitialized := canonicalExceptionBytes }
  , canonicalRow "core3/step/exit-label" canonicalRuleStepContribution
  , canonicalRow "core3/step/return-gemm"
      { canonicalDispatchContribution with
          outputBytes := canonicalTransferBytes }
  , canonicalRow "core3/step/enter-gemm"
      { canonicalDispatchContribution with
          preparationSteps := 1
          bytesWritten := 1 }
  , canonicalRow "core3/step/install-trap" canonicalRuleStepContribution ]

/-- **SPEC section 7.5.**  One row per harness initialization event, in the
pinned order of `Wasm.InitEventId.all`.  The events are exactly the steps
`Wasm.initialConfig` performs and the three `Wasm.InstantiationFault` outcomes
it can report.

`allocate-memory` charges one `memoryGrowPages` unit per declared minimum page
and `allocate-globals` one `bytesWritten` unit per initialized global byte;
both are per-unit charges, since the quantity is a property of the module and
not of the rule.  The three fault rows charge the attempted initialization step
and no completed transfer.  No initialization event is a dispatch, so
`dispatchSteps` is zero throughout: SPEC section 7.5 restricts `dispatchSteps`
to the branch, call, return, exception-transfer and phase-transition rules, and
initialization is none of those. -/
def canonicalInitializationRows : List CostRuleRow :=
  [ canonicalRow "core3/init/validate-module" canonicalInstantiationContribution
  , canonicalRow "core3/init/allocate-memory"
      { canonicalInstantiationContribution with memoryGrowPages := 1 }
  , canonicalRow "core3/init/allocate-globals"
      { canonicalInstantiationContribution with bytesWritten := 1 }
  , canonicalRow "core3/init/resolve-gemm-export"
      canonicalInstantiationContribution
  , canonicalRow "core3/init/build-harness-frame"
      { canonicalInstantiationContribution with preparationSteps := 1 }
  , canonicalRow "core3/init/select-start-function"
      canonicalInstantiationContribution
  , canonicalRow "core3/init/fault-invalid-module"
      canonicalInstantiationContribution
  , canonicalRow "core3/init/fault-allocation-failed"
      canonicalInstantiationContribution
  , canonicalRow "core3/init/fault-missing-gemm-export"
      canonicalInstantiationContribution ]

theorem canonicalRuleRows_length : canonicalRuleRows.length = 34 := rfl

theorem canonicalInitializationRows_length :
    canonicalInitializationRows.length = 9 := rfl

/-- The pinned rule identifiers are duplicate-free.  SPEC section 7.5 admits
only a duplicate-free cover, so this is a precondition of the table, machine
checked. -/
theorem canonicalRuleRows_nodup :
    (canonicalRuleRows.map CostRuleRow.ruleId).Nodup := by decide

/-- The pinned initialization identifiers are duplicate-free. -/
theorem canonicalInitializationRows_nodup :
    (canonicalInitializationRows.map CostRuleRow.ruleId).Nodup := by decide

/-- No row is degenerate: every pinned rule charges at least one
`wasmRuleSteps` unit.  A cost table whose rows all charged zero would typecheck
and be worthless; this rules that out for the released table. -/
theorem canonicalRuleRows_wasmRuleSteps_pos :
    ∀ row ∈ canonicalRuleRows, 0 < row.contribution.wasmRuleSteps := by decide

/-- No initialization row is degenerate either. -/
theorem canonicalInitializationRows_instantiationSteps_pos :
    ∀ row ∈ canonicalInitializationRows,
      0 < row.contribution.instantiationSteps := by decide

/-- The canonical cost-table units, together with the exact duplicate-free
cover of every pinned Core rule identifier and every harness initialization
event identifier (SPEC section 7.5). -/
def canonicalCostTableUnits : CostTableBody :=
  { decodeUnitPerByte := 1
    decodeTerminalUnit := 1
    validationNodeUnit := 1
    validationEdgeUnit := 1
    ruleStepUnit := 1
    installationPreparationUnit := 1
    installedByteWriteUnit := 1
    wholeVectorShuffleLanes := 16
    layout := canonicalGcLayout
    ruleRows := canonicalRuleRows
    initializationRows := canonicalInitializationRows }

@[simp] theorem canonicalCostTableUnits_ruleRows :
    canonicalCostTableUnits.ruleRows = canonicalRuleRows := rfl

@[simp] theorem canonicalCostTableUnits_initializationRows :
    canonicalCostTableUnits.initializationRows = canonicalInitializationRows :=
  rfl

/-- Non-vacuity: the lawfulness conditions are satisfiable, with the full rule
table in place. -/
theorem unitWitnessProfileBody_lawful :
    ProfileLawful
      (canonicalCore3Wasm32ProfileBody core3RevisionCommit
        canonicalCostTableUnits) :=
  canonicalCore3Wasm32ProfileBody_lawful _ rfl rfl rfl rfl rfl rfl rfl
    rfl rfl canonicalRuleRows_nodup canonicalInitializationRows_nodup

/-- The witness profile itself. -/
def unitWitnessProfile : Profile :=
  Profile.checked
    (canonicalCore3Wasm32ProfileBody core3RevisionCommit canonicalCostTableUnits)
    unitWitnessProfileBody_lawful

theorem unitWitnessProfile_decodeCost (bytes : ByteArray) :
    unitWitnessProfile.costTableBody.decodeCost bytes = bytes.size + 1 :=
  unitWitnessProfile.decodeCost_eq bytes

/-! ## Raw invocations (SPEC section 7.1) -/

/-- The first-order body of a raw GEMM invocation: a pointer, a length, and the
raw bytes to install. -/
structure InvocationBody (P : Profile) where
  ptr : UInt32
  len : UInt32
  bytes : ByteArray
  deriving DecidableEq

/-- SPEC section 7.1's lawfulness conditions on a raw invocation. -/
def InvocationLawful (P : Profile) (body : InvocationBody P) : Prop :=
  body.bytes.size = body.len.toNat ∧
  body.ptr.toNat + body.len.toNat ≤ 2 ^ P.addressBits ∧
  pagesFor (body.ptr.toNat + body.len.toNat) ≤ P.maxPages

instance instDecidableInvocationLawful (P : Profile) (body : InvocationBody P) :
    Decidable (InvocationLawful P body) := by
  unfold InvocationLawful; exact inferInstance

/-- A raw invocation: first-order body plus its lawfulness proof. -/
structure Invocation (P : Profile) where
  body : InvocationBody P
  lawful : InvocationLawful P body

/-- SPEC section 7.1's constructor. -/
def Invocation.gemmRaw {P : Profile} (body : InvocationBody P)
    (lawful : InvocationLawful P body) : Invocation P :=
  { body, lawful }

namespace Invocation

@[simp] theorem gemmRaw_body {P : Profile} (body : InvocationBody P)
    (lawful : InvocationLawful P body) : (gemmRaw body lawful).body = body := rfl

theorem eq_of_body_eq {P : Profile} {a b : Invocation P}
    (h : a.body = b.body) : a = b := by
  cases a; cases b; cases h; rfl

/-- The installed bytes are exactly the declared length. -/
theorem size_eq_len {P : Profile} (invocation : Invocation P) :
    invocation.body.bytes.size = invocation.body.len.toNat :=
  invocation.lawful.1

/-- A lawful invocation fits in the wasm32 address space. -/
theorem fits_address_space {P : Profile} (invocation : Invocation P) :
    invocation.body.ptr.toNat + invocation.body.len.toNat ≤ 2 ^ 32 := by
  have h := invocation.lawful.2.1
  rwa [P.addressBits_eq] at h

/-- A lawful invocation fits in the normative page limit. -/
theorem fits_pages {P : Profile} (invocation : Invocation P) :
    pagesFor (invocation.body.ptr.toNat + invocation.body.len.toNat) ≤ 65536 := by
  have h := invocation.lawful.2.2
  rwa [P.maxPages_eq] at h

/-- Every lawful invocation is within the profile's invocation-byte limit. -/
theorem size_le_maxInvocationBytes {P : Profile} (invocation : Invocation P) :
    invocation.body.bytes.size ≤ P.maxInvocationBytes := by
  rw [invocation.size_eq_len, P.maxInvocationBytes_eq, wasm32Ceiling_eq]
  have h := invocation.body.len.toNat_lt
  omega

/-- The empty invocation at pointer zero is lawful, for every profile: the
invocation type is inhabited. -/
theorem empty_lawful (P : Profile) :
    InvocationLawful P { ptr := 0, len := 0, bytes := ByteArray.empty } := by
  refine ⟨rfl, ?_, ?_⟩
  · rw [P.addressBits_eq]
    exact Nat.le_trans
      (Nat.le_of_eq (rfl : (0 : UInt32).toNat + (0 : UInt32).toNat = 0))
      (Nat.zero_le _)
  · rw [P.maxPages_eq]
    exact Nat.le_trans
      (Nat.le_of_eq
        (rfl : pagesFor ((0 : UInt32).toNat + (0 : UInt32).toNat) = 0))
      (Nat.zero_le _)

/-- The empty raw invocation. -/
def empty (P : Profile) : Invocation P :=
  gemmRaw { ptr := 0, len := 0, bytes := ByteArray.empty } (empty_lawful P)

end Invocation

end WasmGemmGnaf.Wasm
