/-
  Wasm/Types.lean --- Core 3.0 types.

  Normative source: the pinned WebAssembly Core `wg-3.0` type grammar, as
  restricted by the release profile of SPEC.md section 7.2.  Because the
  release profile enables reference types, typed function references, GC and
  exception handling, the type syntax here carries heap types, storage/field
  types, structure and array types, recursive/sub types, and tag types.  It
  also carries the `memory64` address-type distinction: SPEC section 7.2 says
  disabled forms "decode when grammatically valid and then fail profile
  validation", so the grammar must be able to *express* an `i64` address type
  in order for validation to reject it.

  This file owns types only.  Modules, instructions, validation judgments and
  execution live in later files (`Syntax`, `Validate`, `Step`, `Run`).

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Feature

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Number, vector and packed types -/

/-- Core number types. -/
inductive NumType
  | i32 | i64 | f32 | f64
  deriving DecidableEq, Repr, Inhabited

/-- Core vector types.  The release profile enables the standard fixed-width
128-bit vector type only. -/
inductive VecType
  | v128
  deriving DecidableEq, Repr, Inhabited

/-- Packed storage types, available inside GC structure and array fields. -/
inductive PackedType
  | i8 | i16
  deriving DecidableEq, Repr, Inhabited

/-- The address type of a memory or table.  `i64` is the `memory64` form; it is
grammatically expressible and rejected by release validation. -/
inductive AddressType
  | i32 | i64
  deriving DecidableEq, Repr, Inhabited

namespace NumType

def tag : NumType → UInt8
  | .i32 => 0 | .i64 => 1 | .f32 => 2 | .f64 => 3

theorem tag_injective : Function.Injective tag := by
  intro a b h
  cases a <;> cases b <;> first | rfl | exact absurd h (by decide)

def bytes (t : NumType) : List UInt8 := Bytes.u8Bytes (tag t)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.u8Bytes_prefixFree.comp tag_injective

def all : List NumType := [.i32, .i64, .f32, .f64]

theorem mem_all (t : NumType) : t ∈ all := by cases t <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance instFintype : Fintype NumType where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

end NumType

namespace VecType

def tag : VecType → UInt8
  | .v128 => 0

theorem tag_injective : Function.Injective tag := by
  intro a b _; cases a; cases b; rfl

def bytes (t : VecType) : List UInt8 := Bytes.u8Bytes (tag t)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.u8Bytes_prefixFree.comp tag_injective

def all : List VecType := [.v128]

theorem mem_all (t : VecType) : t ∈ all := by cases t <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance instFintype : Fintype VecType where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

end VecType

namespace PackedType

def tag : PackedType → UInt8
  | .i8 => 0 | .i16 => 1

theorem tag_injective : Function.Injective tag := by
  intro a b h
  cases a <;> cases b <;> first | rfl | exact absurd h (by decide)

def bytes (t : PackedType) : List UInt8 := Bytes.u8Bytes (tag t)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.u8Bytes_prefixFree.comp tag_injective

def all : List PackedType := [.i8, .i16]

theorem mem_all (t : PackedType) : t ∈ all := by cases t <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance instFintype : Fintype PackedType where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

end PackedType

namespace AddressType

def tag : AddressType → UInt8
  | .i32 => 0 | .i64 => 1

theorem tag_injective : Function.Injective tag := by
  intro a b h
  cases a <;> cases b <;> first | rfl | exact absurd h (by decide)

def bytes (t : AddressType) : List UInt8 := Bytes.u8Bytes (tag t)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.u8Bytes_prefixFree.comp tag_injective

def all : List AddressType := [.i32, .i64]

theorem mem_all (t : AddressType) : t ∈ all := by cases t <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance instFintype : Fintype AddressType where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

/-- The number type an address of this width is represented by. -/
def numType : AddressType → NumType
  | .i32 => .i32
  | .i64 => .i64

/-- The feature family that permits this address type: `i64` addresses are
exactly the `memory64` proposal. -/
def requiredFeature : AddressType → FeatureFamily
  | .i32 => .scalarCore
  | .i64 => .memory64

end AddressType

/-! ## Heap and reference types -/

/-- Abstract heap types of Core 3.0 (the GC and exception hierarchies). -/
inductive AbsHeapType
  | any | eq | i31 | struct | array | none
  | func | nofunc
  | extern | noextern
  | exn | noexn
  deriving DecidableEq, Repr, Inhabited

namespace AbsHeapType

def tag : AbsHeapType → UInt8
  | .any => 0 | .eq => 1 | .i31 => 2 | .struct => 3 | .array => 4 | .none => 5
  | .func => 6 | .nofunc => 7
  | .extern => 8 | .noextern => 9
  | .exn => 10 | .noexn => 11

theorem tag_injective : Function.Injective tag := by
  intro a b h
  cases a <;> cases b <;> first | rfl | exact absurd h (by decide)

def bytes (t : AbsHeapType) : List UInt8 := Bytes.u8Bytes (tag t)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.u8Bytes_prefixFree.comp tag_injective

def all : List AbsHeapType :=
  [ .any, .eq, .i31, .struct, .array, .none
  , .func, .nofunc, .extern, .noextern, .exn, .noexn ]

theorem mem_all (t : AbsHeapType) : t ∈ all := by cases t <;> simp [all]

theorem all_nodup : all.Nodup := by decide

theorem all_length : all.length = 12 := rfl

instance instFintype : Fintype AbsHeapType where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

end AbsHeapType

/-- Heap types: an abstract heap type or a concrete defined type index. -/
inductive HeapType
  | abs (t : AbsHeapType)
  | concrete (typeIndex : Nat)
  deriving DecidableEq, Repr, Inhabited

namespace HeapType

def toSum : HeapType → AbsHeapType ⊕ Nat
  | .abs t => .inl t
  | .concrete i => .inr i

theorem toSum_injective : Function.Injective toSum := by
  intro a b h
  cases a <;> cases b <;> simp_all [toSum]

def bytes (t : HeapType) : List UInt8 :=
  Bytes.sumBytes AbsHeapType.bytes Bytes.natBytes (toSum t)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.sumBytes_prefixFree AbsHeapType.bytes_prefixFree
    Bytes.natBytes_prefixFree).comp toSum_injective

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end HeapType

/-- Reference types: a nullability flag and a heap type. -/
structure RefType where
  nullable : Bool
  heapType : HeapType
  deriving DecidableEq, Repr, Inhabited

namespace RefType

def bytes (t : RefType) : List UInt8 :=
  Bytes.boolBytes t.nullable ++ HeapType.bytes t.heapType

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Bytes.boolBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := HeapType.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

/-- The canonical nullable function reference type. -/
def funcRef : RefType := { nullable := true, heapType := .abs .func }

/-- The canonical nullable external reference type. -/
def externRef : RefType := { nullable := true, heapType := .abs .extern }

/-- The canonical nullable exception reference type. -/
def exnRef : RefType := { nullable := true, heapType := .abs .exn }

end RefType

/-! ## Value types -/

/-- Core value types. -/
inductive ValType
  | num (t : NumType)
  | vec (t : VecType)
  | ref (t : RefType)
  deriving DecidableEq, Repr, Inhabited

namespace ValType

def toSum : ValType → NumType ⊕ (VecType ⊕ RefType)
  | .num t => .inl t
  | .vec t => .inr (.inl t)
  | .ref t => .inr (.inr t)

theorem toSum_injective : Function.Injective toSum := by
  intro a b h
  cases a <;> cases b <;> simp_all [toSum]

def bytes (t : ValType) : List UInt8 :=
  Bytes.sumBytes NumType.bytes
    (Bytes.sumBytes VecType.bytes RefType.bytes) (toSum t)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.sumBytes_prefixFree NumType.bytes_prefixFree
    (Bytes.sumBytes_prefixFree VecType.bytes_prefixFree
      RefType.bytes_prefixFree)).comp toSum_injective

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

/-- Prefix-free encoding of a list of value types (a result type). -/
def listBytes (ts : List ValType) : List UInt8 := Bytes.listBytes bytes ts

theorem listBytes_prefixFree : Bytes.PrefixFree listBytes :=
  Bytes.listBytes_prefixFree bytes_prefixFree

/-- A value type has a default value exactly when it is numeric, vector, or a
nullable reference. -/
def isDefaultable : ValType → Bool
  | .num _ => true
  | .vec _ => true
  | .ref t => t.nullable

/-- The feature family that a value type belongs to. -/
def requiredFeature : ValType → FeatureFamily
  | .num _ => .scalarCore
  | .vec _ => .fixedWidthSimd128
  | .ref _ => .referenceTypesTypedFunctionReferencesGc

/-- Every Core 3.0 value type is admitted by the release profile: the three
families it can belong to are all enabled. -/
theorem requiredFeature_enabled (t : ValType) : Enabled (requiredFeature t) := by
  cases t <;> rfl

end ValType

/-! ## Storage, field, structure, array, function and tag types -/

/-- Storage types: an unpacked value type, or a packed `i8`/`i16` field. -/
inductive StorageType
  | val (t : ValType)
  | packed (t : PackedType)
  deriving DecidableEq, Repr, Inhabited

namespace StorageType

def toSum : StorageType → ValType ⊕ PackedType
  | .val t => .inl t
  | .packed t => .inr t

theorem toSum_injective : Function.Injective toSum := by
  intro a b h
  cases a <;> cases b <;> simp_all [toSum]

def bytes (t : StorageType) : List UInt8 :=
  Bytes.sumBytes ValType.bytes PackedType.bytes (toSum t)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.sumBytes_prefixFree ValType.bytes_prefixFree
    PackedType.bytes_prefixFree).comp toSum_injective

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end StorageType

/-- Field types of GC structures and arrays. -/
structure FieldType where
  mutable : Bool
  storage : StorageType
  deriving DecidableEq, Repr, Inhabited

namespace FieldType

def bytes (t : FieldType) : List UInt8 :=
  Bytes.boolBytes t.mutable ++ StorageType.bytes t.storage

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Bytes.boolBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := StorageType.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

def listBytes (ts : List FieldType) : List UInt8 := Bytes.listBytes bytes ts

theorem listBytes_prefixFree : Bytes.PrefixFree listBytes :=
  Bytes.listBytes_prefixFree bytes_prefixFree

end FieldType

/-- GC structure types: a declared-order list of fields. -/
structure StructType where
  fields : List FieldType
  deriving DecidableEq, Repr, Inhabited

namespace StructType

def bytes (t : StructType) : List UInt8 := FieldType.listBytes t.fields

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  obtain ⟨h1, h⟩ := FieldType.listBytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end StructType

/-- GC array types: one element field type. -/
structure ArrayType where
  element : FieldType
  deriving DecidableEq, Repr, Inhabited

namespace ArrayType

def bytes (t : ArrayType) : List UInt8 := FieldType.bytes t.element

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  obtain ⟨h1, h⟩ := FieldType.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end ArrayType

/-- Function types.  Multi-value results are enabled, so `results` is a list. -/
structure FuncType where
  params : List ValType
  results : List ValType
  deriving DecidableEq, Repr, Inhabited

namespace FuncType

def bytes (t : FuncType) : List UInt8 :=
  ValType.listBytes t.params ++ ValType.listBytes t.results

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := ValType.listBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := ValType.listBytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

def arity (t : FuncType) : Nat × Nat := (t.params.length, t.results.length)

/-- A function type is multi-value exactly when it returns more than one
result. -/
def isMultiValue (t : FuncType) : Bool := 1 < t.results.length

end FuncType

/-- Exception tag types.  A Core tag type is a function type whose results are
empty; the parameters are the exception payload. -/
structure TagType where
  funcType : FuncType
  deriving DecidableEq, Repr, Inhabited

namespace TagType

def bytes (t : TagType) : List UInt8 := FuncType.bytes t.funcType

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  obtain ⟨h1, h⟩ := FuncType.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

/-- The payload value types carried by a thrown exception of this tag. -/
def payload (t : TagType) : List ValType := t.funcType.params

/-- Core well-formedness of a tag type: no results. -/
def WellFormed (t : TagType) : Prop := t.funcType.results = []

instance instDecidableWellFormed (t : TagType) : Decidable (WellFormed t) := by
  unfold WellFormed; exact inferInstance

end TagType

/-! ## Composite, sub and recursive types -/

/-- Composite types: the body of a defined type. -/
inductive CompType
  | func (t : FuncType)
  | struct (t : StructType)
  | array (t : ArrayType)
  deriving DecidableEq, Repr, Inhabited

namespace CompType

def toSum : CompType → FuncType ⊕ (StructType ⊕ ArrayType)
  | .func t => .inl t
  | .struct t => .inr (.inl t)
  | .array t => .inr (.inr t)

theorem toSum_injective : Function.Injective toSum := by
  intro a b h
  cases a <;> cases b <;> simp_all [toSum]

def bytes (t : CompType) : List UInt8 :=
  Bytes.sumBytes FuncType.bytes
    (Bytes.sumBytes StructType.bytes ArrayType.bytes) (toSum t)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.sumBytes_prefixFree FuncType.bytes_prefixFree
    (Bytes.sumBytes_prefixFree StructType.bytes_prefixFree
      ArrayType.bytes_prefixFree)).comp toSum_injective

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end CompType

/-- Sub types: a finality flag, declared supertype indices, and a body. -/
structure SubType where
  final : Bool
  supertypes : List Nat
  body : CompType
  deriving DecidableEq, Repr, Inhabited

namespace SubType

def bytes (t : SubType) : List UInt8 :=
  Bytes.boolBytes t.final ++
    (Enc.natsBytes t.supertypes ++ CompType.bytes t.body)

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Bytes.boolBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := Enc.natsBytes_prefixFree _ _ _ _ h
  obtain ⟨h3, h⟩ := CompType.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

def listBytes (ts : List SubType) : List UInt8 := Bytes.listBytes bytes ts

theorem listBytes_prefixFree : Bytes.PrefixFree listBytes :=
  Bytes.listBytes_prefixFree bytes_prefixFree

end SubType

/-- Recursive type groups. -/
structure RecType where
  types : List SubType
  deriving DecidableEq, Repr, Inhabited

namespace RecType

def bytes (t : RecType) : List UInt8 := SubType.listBytes t.types

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  obtain ⟨h1, h⟩ := SubType.listBytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end RecType

/-! ## Limits, memory, table, global and external types -/

/-- Resource limits of a memory or table. -/
structure Limits where
  min : Nat
  max : Option Nat
  deriving DecidableEq, Repr, Inhabited

namespace Limits

def bytes (l : Limits) : List UInt8 :=
  Bytes.natBytes l.min ++ Bytes.optionBytes Bytes.natBytes l.max

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Bytes.natBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ :=
    (Bytes.optionBytes_prefixFree Bytes.natBytes_prefixFree) _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

/-- The limits are within an upper bound: `min` and, when present, `max` do not
exceed it, and `min` does not exceed `max`. -/
def WithinBound (l : Limits) (bound : Nat) : Prop :=
  l.min ≤ bound ∧ (∀ m, l.max = some m → l.min ≤ m ∧ m ≤ bound)

instance instDecidableWithinBound (l : Limits) (bound : Nat) :
    Decidable (WithinBound l bound) := by
  cases l with
  | mk min max =>
    cases max with
    | none =>
      exact decidable_of_iff (min ≤ bound)
        ⟨fun h => ⟨h, by intro m hm; exact absurd hm (by simp)⟩, fun h => h.1⟩
    | some m =>
      exact decidable_of_iff (min ≤ bound ∧ min ≤ m ∧ m ≤ bound)
        ⟨fun h => ⟨h.1, by intro k hk; cases hk; exact h.2⟩,
         fun h => ⟨h.1, by
           have := h.2 m rfl
           exact this⟩⟩

end Limits

/-- Memory types. -/
structure MemType where
  addressType : AddressType
  limits : Limits
  deriving DecidableEq, Repr, Inhabited

namespace MemType

def bytes (t : MemType) : List UInt8 :=
  AddressType.bytes t.addressType ++ Limits.bytes t.limits

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := AddressType.bytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := Limits.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end MemType

/-- Table types. -/
structure TableType where
  addressType : AddressType
  limits : Limits
  element : RefType
  deriving DecidableEq, Repr, Inhabited

namespace TableType

def bytes (t : TableType) : List UInt8 :=
  AddressType.bytes t.addressType ++
    (Limits.bytes t.limits ++ RefType.bytes t.element)

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := AddressType.bytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := Limits.bytes_prefixFree _ _ _ _ h
  obtain ⟨h3, h⟩ := RefType.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end TableType

/-- Global types. -/
structure GlobalType where
  mutable : Bool
  valType : ValType
  deriving DecidableEq, Repr, Inhabited

namespace GlobalType

def bytes (t : GlobalType) : List UInt8 :=
  Bytes.boolBytes t.mutable ++ ValType.bytes t.valType

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := Bytes.boolBytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := ValType.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x; cases y; simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end GlobalType

/-- External types: the type of an import or export item. -/
inductive ExternType
  | func (t : FuncType)
  | table (t : TableType)
  | mem (t : MemType)
  | global (t : GlobalType)
  | tag (t : TagType)
  deriving DecidableEq, Repr, Inhabited

namespace ExternType

def toSum : ExternType →
    FuncType ⊕ (TableType ⊕ (MemType ⊕ (GlobalType ⊕ TagType)))
  | .func t => .inl t
  | .table t => .inr (.inl t)
  | .mem t => .inr (.inr (.inl t))
  | .global t => .inr (.inr (.inr (.inl t)))
  | .tag t => .inr (.inr (.inr (.inr t)))

theorem toSum_injective : Function.Injective toSum := by
  intro a b h
  cases a <;> cases b <;> simp_all [toSum]

def bytes (t : ExternType) : List UInt8 :=
  Bytes.sumBytes FuncType.bytes
    (Bytes.sumBytes TableType.bytes
      (Bytes.sumBytes MemType.bytes
        (Bytes.sumBytes GlobalType.bytes TagType.bytes))) (toSum t)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.sumBytes_prefixFree FuncType.bytes_prefixFree
    (Bytes.sumBytes_prefixFree TableType.bytes_prefixFree
      (Bytes.sumBytes_prefixFree MemType.bytes_prefixFree
        (Bytes.sumBytes_prefixFree GlobalType.bytes_prefixFree
          TagType.bytes_prefixFree)))).comp toSum_injective

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end ExternType

/-! ## The GEMM ABI type

SPEC section 7.2 requires one exported function named `gemm` with ABI type
`(i32, i32) -> i32`. -/

/-- The pinned GEMM ABI function type. -/
def gemmFuncType : FuncType :=
  { params := [.num .i32, .num .i32], results := [.num .i32] }

theorem gemmFuncType_params :
    gemmFuncType.params = [ValType.num .i32, ValType.num .i32] := rfl

theorem gemmFuncType_results : gemmFuncType.results = [ValType.num .i32] := rfl

theorem gemmFuncType_arity : gemmFuncType.arity = (2, 1) := rfl

theorem gemmFuncType_not_multiValue : gemmFuncType.isMultiValue = false := rfl

/-! ## Feature attribution of the address types

SPEC section 7.2 rejects "memory64 address types and instructions".  The `i64`
address type is expressible --- it must be, since a disabled form "decodes when
grammatically valid and then fails profile validation" --- and belongs to the
rejected family. -/

theorem AddressType.i32_requiredFeature_enabled :
    Enabled (AddressType.requiredFeature .i32) := rfl

theorem AddressType.i64_requiredFeature_rejected :
    Rejected (AddressType.requiredFeature .i64) := rfl

end WasmGemmGnaf.Wasm
