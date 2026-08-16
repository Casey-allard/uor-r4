/-
  Wasm/Syntax.lean --- Core 3.0 module syntax.

  Normative source: SPEC.md section 7.1 (ownership) and section 7.2 (the
  released portable profile and its feature matrix).

  SPEC section 7.1: "`Syntax` owns types, functions, globals, tables, memories,
  elements, data, imports, exports, and administrative instructions."  This
  file therefore carries `Wasm.Module` (with every one of those component
  lists) and `Wasm.Instr` / `Wasm.Expr` (the instruction syntax).  The Core 3.0
  *type* grammar is owned by `Wasm/Types.lean` and is imported, not restated.

  ## Declared instruction coverage

  SPEC section 7.2 allows a released subset only if the decision for each
  family is explicit.  The families covered by the `Instr` inductive below are
  exactly:

  * **control**: `unreachable`, `nop`, `block`, `loop`, `if`/`else`, `br`,
    `br_if`, `br_table`, `return`, `call`, `call_indirect`;
  * **tail calls**: `return_call`, `return_call_indirect`;
  * **exception handling**: `throw`, `throw_ref`, `try_table` with all four
    catch forms;
  * **parametric**: `drop`, `select` (both the untyped and the typed form);
  * **variables**: `local.get`/`local.set`/`local.tee`,
    `global.get`/`global.set`;
  * **references**: `ref.null`, `ref.is_null`, `ref.func`, `ref.eq`,
    `ref.as_non_null`, `ref.test`, `ref.cast`, `ref.i31`, `i31.get_s`,
    `i31.get_u`, `any.convert_extern`, `extern.convert_any`;
  * **garbage collection (structures and arrays)**: `struct.new`,
    `struct.new_default`, `struct.get`(`_s`/`_u`), `struct.set`, `array.new`,
    `array.new_default`, `array.get`(`_s`/`_u`), `array.set`, `array.len`;
  * **tables**: `table.get`, `table.set`, `table.size`, `table.grow`,
    `table.fill`, `table.copy`, `table.init`, `elem.drop`;
  * **bulk memory and multiple memories**: `t.load`/`t.store` in every width
    and sign-extension form, `v128.load`/`v128.store`, `memory.size`,
    `memory.grow`, `memory.fill`, `memory.copy`, `memory.init`, `data.drop`
    --- every one of them carrying an explicit memory index, so multiple
    memories are expressible;
  * **numeric**: `i32`/`i64`/`f32`/`f64` constants, the integer unary, binary,
    test and relational operator families (including the sign-extension
    operators), the floating point unary, binary and relational operator
    families, and the full conversion family;
  * **standard fixed-width 128-bit SIMD**: `v128.const`, the shape-indexed
    unary, binary and relational operator families, `v128.bitselect`,
    `t.splat`, `t.extract_lane`(`_s`/`_u`), `t.replace_lane`, and
    `i8x16.shuffle`.

  Deliberately **not** modelled here, and therefore not expressible:

  * `memory64` address *instructions* --- the `i64` address *type* is
    expressible (see `Wasm/Types.lean`) precisely so that release validation
    can reject it, as SPEC section 7.2 requires;
  * shared memories, atomics and thread instructions (rejected family);
  * relaxed-SIMD operators (rejected family);
  * component-model forms (rejected family).

  The GC family is covered in the structure/array/i31 subset listed above;
  the remaining GC bulk operators (`array.copy`, `array.fill`,
  `array.new_data`, `array.init_elem`, ...) are outside the declared subset.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Types

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Names

A Core name is a byte string.  Carrying it as `List UInt8` rather than as a
`String` keeps the syntax closed under decoding: every byte string is a
syntactically well-formed name, and the UTF-8 well-formedness requirement is a
*validation* condition, not a syntactic one. -/

/-- A Core name: a raw byte string. -/
abbrev Name := List UInt8

/-- The name denoted by an ASCII/UTF-8 Lean string literal. -/
def nameOfString (s : String) : Name := s.toByteArray.toList

theorem nameOfString_injective : Function.Injective nameOfString := by
  intro s t h
  apply Enc.toByteArray_injective
  have : (String.toByteArray s).data.toList = (String.toByteArray t).data.toList := by
    simpa [nameOfString, Bytes.byteArray_toList] using h
  exact Bytes.toList_injective (by simpa [Bytes.byteArray_toList] using this)

/-- The two required release export names of SPEC section 7.2. -/
def gemmExportName : Name := nameOfString "gemm"

/-- The required memory export name of SPEC section 7.2. -/
def memoryExportName : Name := nameOfString "memory"

theorem gemmExportName_ne_memoryExportName :
    gemmExportName ≠ memoryExportName := by
  intro h
  exact absurd (nameOfString_injective h) (by decide)

/-! ## Immediate operand kinds -/

/-- The width of an integer operator family. -/
inductive IntWidth
  | i32 | i64
  deriving DecidableEq, Repr, Inhabited

/-- The width of a floating point operator family. -/
inductive FloatWidth
  | f32 | f64
  deriving DecidableEq, Repr, Inhabited

/-- Signedness of a narrowing/widening operator. -/
inductive SignExt
  | signed | unsigned
  deriving DecidableEq, Repr, Inhabited

/-- The narrowed access width of a partial memory load or store. -/
inductive MemWidth
  | w8 | w16 | w32
  deriving DecidableEq, Repr, Inhabited

/-- A memory immediate: the memory index, the alignment exponent, and the
static offset.  Every memory instruction carries an explicit index, which is
what makes the multiple-memories family expressible. -/
structure MemArg where
  memory : Nat
  align : Nat
  offset : Nat
  deriving DecidableEq, Repr, Inhabited

/-- The result type annotation of a structured control instruction. -/
inductive BlockType
  | empty
  | value (t : ValType)
  | typeIndex (i : Nat)
  deriving DecidableEq, Repr, Inhabited

/-- The four catch forms of the exception-handling `try_table`. -/
inductive CatchKind
  | catch | catchRef | catchAll | catchAllRef
  deriving DecidableEq, Repr, Inhabited

/-- One catch clause of a `try_table`.  For the `catch_all` forms the `tag`
field is ignored by validation; it is kept in the syntax so that the clause has
a single uniform shape. -/
structure Catch where
  kind : CatchKind
  tag : Nat
  label : Nat
  deriving DecidableEq, Repr, Inhabited

/-! ## Numeric operator families -/

/-- Integer unary operators, including the sign-extension operators. -/
inductive IUnOp
  | clz | ctz | popcnt | extend8S | extend16S | extend32S
  deriving DecidableEq, Repr, Inhabited

/-- Integer binary operators. -/
inductive IBinOp
  | add | sub | mul | divS | divU | remS | remU
  | and | or | xor | shl | shrS | shrU | rotl | rotr
  deriving DecidableEq, Repr, Inhabited

/-- Integer test operators. -/
inductive ITestOp
  | eqz
  deriving DecidableEq, Repr, Inhabited

/-- Integer relational operators. -/
inductive IRelOp
  | eq | ne | ltS | ltU | gtS | gtU | leS | leU | geS | geU
  deriving DecidableEq, Repr, Inhabited

/-- Floating point unary operators. -/
inductive FUnOp
  | abs | neg | ceil | floor | trunc | nearest | sqrt
  deriving DecidableEq, Repr, Inhabited

/-- Floating point binary operators. -/
inductive FBinOp
  | add | sub | mul | div | min | max | copysign
  deriving DecidableEq, Repr, Inhabited

/-- Floating point relational operators. -/
inductive FRelOp
  | eq | ne | lt | gt | le | ge
  deriving DecidableEq, Repr, Inhabited

/-- The Core 3.0 conversion family. -/
inductive CvtOp
  | i32WrapI64
  | i32TruncF32S | i32TruncF32U | i32TruncF64S | i32TruncF64U
  | i64ExtendI32S | i64ExtendI32U
  | i64TruncF32S | i64TruncF32U | i64TruncF64S | i64TruncF64U
  | f32ConvertI32S | f32ConvertI32U | f32ConvertI64S | f32ConvertI64U
  | f32DemoteF64
  | f64ConvertI32S | f64ConvertI32U | f64ConvertI64S | f64ConvertI64U
  | f64PromoteF32
  | i32ReinterpretF32 | i64ReinterpretF64
  | f32ReinterpretI32 | f64ReinterpretI64
  | i32TruncSatF32S | i32TruncSatF32U | i32TruncSatF64S | i32TruncSatF64U
  | i64TruncSatF32S | i64TruncSatF32U | i64TruncSatF64S | i64TruncSatF64U
  deriving DecidableEq, Repr, Inhabited

/-! ## Vector operator families (standard fixed-width 128-bit SIMD) -/

/-- The lane shapes of the standard 128-bit vector type. -/
inductive VecShape
  | i8x16 | i16x8 | i32x4 | i64x2 | f32x4 | f64x2
  deriving DecidableEq, Repr, Inhabited

/-- Shape-indexed vector unary operators, together with the whole-vector
bitwise unary operator `v128.not`. -/
inductive VecUnOp
  | not | neg | abs | sqrt | ceil | floor | trunc | nearest
  | popcnt | anyTrue | allTrue | bitmask
  deriving DecidableEq, Repr, Inhabited

/-- Shape-indexed vector binary operators, together with the whole-vector
bitwise binary operators. -/
inductive VecBinOp
  | and | andnot | or | xor
  | add | sub | mul | div | min | max | pmin | pmax
  | addSatS | addSatU | subSatS | subSatU
  | avgrU | shl | shrS | shrU
  deriving DecidableEq, Repr, Inhabited

/-- Shape-indexed vector relational operators. -/
inductive VecRelOp
  | eq | ne | ltS | ltU | gtS | gtU | leS | leU | geS | geU | lt | gt | le | ge
  deriving DecidableEq, Repr, Inhabited

/-! ## Instructions

`Instr` and `Expr` are declared mutually: `Expr` is the Core notion of an
*instruction sequence*, and the structured control instructions embed it.
Using a dedicated cons-list rather than `List Instr` gives genuine mutual
recursors, which is what makes the decoder proofs in `Wasm/Binary.lean`
go through. -/

mutual

/-- A Core 3.0 instruction, restricted to the declared subset above. -/
inductive Instr where
  -- control
  | unreachable
  | nop
  | block (bt : BlockType) (body : Expr)
  | loop (bt : BlockType) (body : Expr)
  | ifThenElse (bt : BlockType) (thenBody : Expr) (elseBody : Expr)
  | br (label : Nat)
  | brIf (label : Nat)
  | brTable (labels : List Nat) (default : Nat)
  | ret
  | call (func : Nat)
  | callIndirect (table : Nat) (type : Nat)
  -- tail calls
  | returnCall (func : Nat)
  | returnCallIndirect (table : Nat) (type : Nat)
  -- exception handling
  | throw (tag : Nat)
  | throwRef
  | tryTable (bt : BlockType) (catches : List Catch) (body : Expr)
  -- parametric
  | drop
  | select (types : Option (List ValType))
  -- variables
  | localGet (idx : Nat)
  | localSet (idx : Nat)
  | localTee (idx : Nat)
  | globalGet (idx : Nat)
  | globalSet (idx : Nat)
  -- references
  | refNull (ht : HeapType)
  | refIsNull
  | refFunc (idx : Nat)
  | refEq
  | refAsNonNull
  | refTest (nullable : Bool) (ht : HeapType)
  | refCast (nullable : Bool) (ht : HeapType)
  | refI31
  | i31GetS
  | i31GetU
  | anyConvertExtern
  | externConvertAny
  -- garbage collection: structures and arrays
  | structNew (ty : Nat)
  | structNewDefault (ty : Nat)
  | structGet (ty : Nat) (field : Nat) (ext : Option SignExt)
  | structSet (ty : Nat) (field : Nat)
  | arrayNew (ty : Nat)
  | arrayNewDefault (ty : Nat)
  | arrayGet (ty : Nat) (ext : Option SignExt)
  | arraySet (ty : Nat)
  | arrayLen
  -- tables
  | tableGet (table : Nat)
  | tableSet (table : Nat)
  | tableSize (table : Nat)
  | tableGrow (table : Nat)
  | tableFill (table : Nat)
  | tableCopy (dst : Nat) (src : Nat)
  | tableInit (table : Nat) (elem : Nat)
  | elemDrop (elem : Nat)
  -- memory
  | load (t : NumType) (narrow : Option (MemWidth × SignExt)) (arg : MemArg)
  | store (t : NumType) (narrow : Option MemWidth) (arg : MemArg)
  | vecLoad (arg : MemArg)
  | vecStore (arg : MemArg)
  | memorySize (mem : Nat)
  | memoryGrow (mem : Nat)
  | memoryFill (mem : Nat)
  | memoryCopy (dst : Nat) (src : Nat)
  | memoryInit (mem : Nat) (data : Nat)
  | dataDrop (data : Nat)
  -- numeric
  | i32Const (value : Int)
  | i64Const (value : Int)
  | f32Const (bits : UInt32)
  | f64Const (bits : UInt64)
  | iUnOp (w : IntWidth) (op : IUnOp)
  | iBinOp (w : IntWidth) (op : IBinOp)
  | iTestOp (w : IntWidth) (op : ITestOp)
  | iRelOp (w : IntWidth) (op : IRelOp)
  | fUnOp (w : FloatWidth) (op : FUnOp)
  | fBinOp (w : FloatWidth) (op : FBinOp)
  | fRelOp (w : FloatWidth) (op : FRelOp)
  | cvtOp (op : CvtOp)
  -- standard fixed-width 128-bit SIMD
  | vecConst (lo : UInt64) (hi : UInt64)
  | vecUnOp (shape : VecShape) (op : VecUnOp)
  | vecBinOp (shape : VecShape) (op : VecBinOp)
  | vecRelOp (shape : VecShape) (op : VecRelOp)
  | vecBitselect
  | vecSplat (shape : VecShape)
  | vecExtractLane (shape : VecShape) (lane : Nat) (ext : Option SignExt)
  | vecReplaceLane (shape : VecShape) (lane : Nat)
  | vecShuffle (lanes : List Nat)

/-- A Core instruction sequence. -/
inductive Expr where
  | nil
  | cons (head : Instr) (tail : Expr)

end

deriving instance DecidableEq for Instr, Expr

instance : Inhabited Instr := ⟨Instr.nop⟩
instance : Inhabited Expr := ⟨Expr.nil⟩

namespace Expr

/-- The instruction sequence as an ordinary list. -/
def toList : Expr → List Instr
  | .nil => []
  | .cons i e => i :: toList e

/-- The instruction sequence of an ordinary list. -/
def ofList : List Instr → Expr
  | [] => .nil
  | i :: is => .cons i (ofList is)

@[simp] theorem toList_ofList (l : List Instr) : toList (ofList l) = l := by
  induction l with
  | nil => rfl
  | cons i is ih => simp [toList, ofList, ih]

@[simp] theorem ofList_toList : ∀ e : Expr, ofList (toList e) = e
  | .nil => rfl
  | .cons i e => by
      show Expr.cons i (ofList (toList e)) = Expr.cons i e
      rw [ofList_toList e]

theorem ofList_injective : Function.Injective ofList := by
  intro a b h
  have := congrArg toList h
  simpa using this

theorem toList_injective : Function.Injective toList := by
  intro a b h
  have := congrArg ofList h
  simpa using this

/-- Number of top-level instructions. -/
def length : Expr → Nat
  | .nil => 0
  | .cons _ e => 1 + length e

@[simp] theorem length_toList : ∀ e : Expr, e.toList.length = e.length
  | .nil => rfl
  | .cons i e => by
      show e.toList.length + 1 = 1 + e.length
      rw [length_toList e]; omega

end Expr

/-! ## Module components -/

/-- An import description. -/
inductive ImportDesc
  | func (type : Nat)
  | table (type : TableType)
  | mem (type : MemType)
  | global (type : GlobalType)
  | tag (type : TagType)
  deriving DecidableEq, Repr, Inhabited

/-- An import: a two-level name and a described item. -/
structure Import where
  module : Name
  name : Name
  desc : ImportDesc
  deriving DecidableEq, Repr, Inhabited

/-- An export description. -/
inductive ExportDesc
  | func (idx : Nat)
  | table (idx : Nat)
  | mem (idx : Nat)
  | global (idx : Nat)
  | tag (idx : Nat)
  deriving DecidableEq, Repr, Inhabited

/-- An export: a name and a described item. -/
structure Export where
  name : Name
  desc : ExportDesc
  deriving DecidableEq, Repr, Inhabited

/-- A defined function. -/
structure Func where
  type : Nat
  locals : List ValType
  body : Expr
  deriving DecidableEq, Inhabited

/-- A defined table, with its Core 3.0 initializer expression. -/
structure Table where
  type : TableType
  init : Expr
  deriving DecidableEq, Inhabited

/-- A defined memory. -/
structure Mem where
  type : MemType
  deriving DecidableEq, Repr, Inhabited

/-- A defined global. -/
structure Global where
  type : GlobalType
  init : Expr
  deriving DecidableEq, Inhabited

/-- The mode of an element segment. -/
inductive ElemMode
  | passive
  | active (table : Nat) (offset : Expr)
  | declarative
  deriving DecidableEq, Inhabited

/-- An element segment. -/
structure Elem where
  type : RefType
  init : List Expr
  mode : ElemMode
  deriving DecidableEq, Inhabited

/-- The mode of a data segment. -/
inductive DataMode
  | passive
  | active (mem : Nat) (offset : Expr)
  deriving DecidableEq, Inhabited

/-- A data segment. -/
structure Data where
  init : List UInt8
  mode : DataMode
  deriving DecidableEq, Inhabited

/-! ## Modules -/

/-- A Core 3.0 module.

Field order is the pinned section order of the binary format: types, imports,
functions, tables, memories, tags, globals, exports, start, elements, data. -/
structure Module where
  types : List RecType
  imports : List Import
  funcs : List Func
  tables : List Table
  mems : List Mem
  tags : List TagType
  globals : List Global
  exports : List Export
  start : Option Nat
  elems : List Elem
  datas : List Data
  deriving DecidableEq, Inhabited

namespace Module

/-- The empty module: every component list empty and no start function. -/
def empty : Module :=
  { types := [], imports := [], funcs := [], tables := [], mems := []
    tags := [], globals := [], exports := [], start := none
    elems := [], datas := [] }

@[simp] theorem empty_types : empty.types = [] := rfl
@[simp] theorem empty_imports : empty.imports = [] := rfl
@[simp] theorem empty_funcs : empty.funcs = [] := rfl
@[simp] theorem empty_exports : empty.exports = [] := rfl
@[simp] theorem empty_start : empty.start = none := rfl

/-- The exports of a module carrying a given name. -/
def exportsNamed (m : Module) (n : Name) : List Export :=
  m.exports.filter (fun e => e.name = n)

theorem mem_exportsNamed {m : Module} {n : Name} {e : Export} :
    e ∈ m.exportsNamed n → e ∈ m.exports ∧ e.name = n := by
  intro h
  have h' := List.mem_filter.mp h
  exact ⟨h'.1, of_decide_eq_true h'.2⟩

/-- The module declares no imports (the closed release policy of SPEC
section 7.2). -/
def IsClosed (m : Module) : Prop := m.imports = []

instance (m : Module) : Decidable (m.IsClosed) := by
  unfold IsClosed; exact inferInstance

@[simp] theorem empty_isClosed : empty.IsClosed := rfl

end Module

end WasmGemmGnaf.Wasm
