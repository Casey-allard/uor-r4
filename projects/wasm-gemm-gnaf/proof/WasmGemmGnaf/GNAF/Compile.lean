import WasmGemmGnaf.GNAF.Typing
import WasmGemmGnaf.Wasm.Validate

set_option autoImplicit false

/-!
# GNAF: the verified compiler to Core Wasm (SPEC §11.4)

`GNAF.compile` translates a `CheckedPlan` into a `Wasm.Module`.  It is a total,
structurally recursive function on the plan datatype of `GNAF/Plan.lean`, and
every plan constructor has a real translation clause.

## The target

The target is *not* the whole Core grammar of `Wasm/Syntax.lean`: it is the
released executable subset that `Wasm/Validate.lean` accepts — `i32` only, one
memory, no imports, no tables, no element or data segments, structured control
(`block`/`loop`/`if`), `br`/`br_if`, full-width `i32` loads and stores on
memory 0, locals, and the wrapping/unsigned `i32` operators.  The main theorem
of this file, `GNAF.compile_validates`, proves that the emitted module really
is in that subset, so the emitted module is not merely syntactically
well-formed: it passes the released profile validator.

## The machine model of the translation

The GNAF machine of `GNAF/Semantics.lean` has a cell-addressed memory, a
private scratch store, a scalar register file, code/data tables, a status word
and an output region.  The translation fixes the following representation.

* GNAF memory cell `i` is the `i32` at byte address `4 * i`.  The compiler
  therefore assumes the *word* raw-invocation form: cell `i` of the raw input
  is the `i32` at `4 * i`, which is the form in which the harness of
  `Wasm/Config.lean` must install the invocation for this module.  No
  byte-repacking prologue is emitted, and no claim is made here about the
  harness byte form.
* The scratch store follows the memory image, the code/data tables follow the
  scratch, and the status word and the two output-descriptor words follow the
  tables.  All of it is one exported memory, sized by `CompileEnv.pages`.
* Scalar register `r` is local `2 + r` (locals `0` and `1` are the two ABI
  parameters).  One extra local per loop-nesting level carries loop counters
  and dispatch results.
* Every GNAF value is a `Nat`; every Wasm value is an `i32`.  The translation
  is therefore exact only modulo `2 ^ 32`.  This is stated, not proved: this
  file proves nothing about execution (see `GNAF/CompileCorrect.lean` for what
  the current state of the Wasm layer does support).

## The honest domain: `Plan.inReleasedSubset`

Two plan constructs cannot be expressed by the released profile at all:

* `vectorOp` — the released profile has no SIMD, so no v128 instruction is
  admitted by `Wasm.validate`;
* `reduce` under an arithmetic contract that is not modular with a `u32`
  accumulator — every other contract needs an accumulator wider than `i32`,
  or a checked-overflow test that `i32` cannot perform.

Rather than emit code that computes something *different* from the plan, the
compiler emits `unreachable` for exactly those nodes: the artifact traps
instead of lying.  `Plan.inReleasedSubset` is the decidable predicate that
excludes them, `GNAF.code_no_unreachable` proves that the translation of an
in-subset plan contains no `unreachable` at any nesting depth, and
`GNAF.hasType_no_vector` proves that a plan typed at a *scalar interface*
(`Sig.lanes = 0`) never contains a `vectorOp`, so restricting `CheckedPlan` to
scalar interfaces already discharges half of the restriction through the
typing judgment.

Everything outside that subset is still compiled — `GNAF.compile` is total on
every `CheckedPlan` — and `GNAF.compile_validates` holds for all of them; what
the subset buys is that inside it no node was refused.

Nothing in this file assumes anything.  Every declaration is proved.
-/

namespace WasmGemmGnaf.GNAF

open WasmGemmGnaf.Foundation

/-! ## The compilation environment -/

/-- The static layout parameters of one compilation: the sizes of the GNAF
machine's stores, and the loop-nesting depth of the plan. -/
structure CompileEnv where
  /-- Size of the scalar register file. -/
  regs : Nat
  /-- Number of GNAF memory cells. -/
  memWords : Nat
  /-- Static upper bound on the number of scratch cells. -/
  scratchWords : Nat
  /-- Number of code/data tables. -/
  tableCount : Nat
  /-- Number of cells reserved for each code/data table. -/
  tableStride : Nat
  /-- Static bound on the plan's nesting depth. -/
  depthBound : Nat
  deriving DecidableEq, Repr, Inhabited

namespace CompileEnv

/-- Byte address of GNAF memory cell `i`. -/
def memAddr (_e : CompileEnv) (i : Nat) : Nat := 4 * i

/-- Byte address of the first scratch cell. -/
def scratchBase (e : CompileEnv) : Nat := 4 * e.memWords

/-- Byte address of scratch cell `i`. -/
def scratchAddr (e : CompileEnv) (i : Nat) : Nat := e.scratchBase + 4 * i

/-- Byte address of the first code/data table cell. -/
def tableBase (e : CompileEnv) : Nat := e.scratchBase + 4 * e.scratchWords

/-- Byte address of the status word. -/
def statusAddr (e : CompileEnv) : Nat :=
  e.tableBase + 4 * (e.tableCount * e.tableStride)

/-- Byte address of the output pointer word. -/
def outPtrAddr (e : CompileEnv) : Nat := e.statusAddr + 4

/-- Byte address of the output length word. -/
def outLenAddr (e : CompileEnv) : Nat := e.statusAddr + 8

/-- Total number of bytes the layout needs. -/
def byteSize (e : CompileEnv) : Nat := e.outLenAddr + 4

/-- The number of memory pages the emitted module declares.  It is clamped to
the hard page limit of `Wasm/Memory.lean`, which is what makes the emitted
memory type admissible for every environment. -/
def pages (e : CompileEnv) : Nat :=
  Nat.min Wasm.Memory.hardMaxPages (e.byteSize / 65536 + 1)

theorem pages_le_hard (e : CompileEnv) : e.pages ≤ Wasm.Memory.hardMaxPages :=
  Nat.min_le_left _ _

/-- Local index of scalar register `r`. -/
def regLocal (_e : CompileEnv) (r : Nat) : Nat := 2 + r

/-- Local index of the scratch local of nesting depth `d`. -/
def tempLocal (e : CompileEnv) (d : Nat) : Nat := 2 + e.regs + d

/-- Number of locals declared by the emitted function, beyond the two ABI
parameters. -/
def declaredLocals (e : CompileEnv) : Nat := e.regs + e.depthBound + 1

/-- Total number of locals in scope in the emitted function. -/
def numLocals (e : CompileEnv) : Nat := 2 + e.declaredLocals

end CompileEnv

/-! ## Instruction shorthands

Every shorthand below is a member of the released executable subset of
`Wasm/Validate.lean`. -/

/-- The memory immediate of every emitted access: memory 0, natural alignment,
zero static offset. -/
def memArg : Wasm.MemArg := { memory := 0, align := 2, offset := 0 }

/-- A non-negative `i32` constant. -/
def constI (n : Nat) : Wasm.Instr := .i32Const (Int.ofNat n)

/-- A full-width `i32` load from memory 0. -/
def loadW : Wasm.Instr := .load .i32 none memArg

/-- A full-width `i32` store to memory 0. -/
def storeW : Wasm.Instr := .store .i32 none memArg

def addI : Wasm.Instr := .iBinOp .i32 .add
def subI : Wasm.Instr := .iBinOp .i32 .sub
def mulI : Wasm.Instr := .iBinOp .i32 .mul
def andI : Wasm.Instr := .iBinOp .i32 .and
def orI : Wasm.Instr := .iBinOp .i32 .or
def eqI : Wasm.Instr := .iRelOp .i32 .eq
def ltUI : Wasm.Instr := .iRelOp .i32 .ltU
def gtUI : Wasm.Instr := .iRelOp .i32 .gtU
def geUI : Wasm.Instr := .iRelOp .i32 .geU

/-- A two-armed `if` with empty block type. -/
def ifE (t f : List Wasm.Instr) : Wasm.Instr :=
  .ifThenElse .empty (Wasm.Expr.ofList t) (Wasm.Expr.ofList f)

/-- A `block` with empty block type. -/
def blockE (b : List Wasm.Instr) : Wasm.Instr :=
  .block .empty (Wasm.Expr.ofList b)

/-- A `loop` with empty block type. -/
def loopE (b : List Wasm.Instr) : Wasm.Instr :=
  .loop .empty (Wasm.Expr.ofList b)

/-- Load the `i32` at a fixed byte address. -/
def loadAt (addr : Nat) : List Wasm.Instr := [constI addr, loadW]

/-- The counting loop `for c := 0 while c < n do body; c := c + 1`.  The
counter is the local `c`; `body` must be operand-stack neutral. -/
def countLoop (c n : Nat) (body : List Wasm.Instr) : List Wasm.Instr :=
  [constI 0, .localSet c,
   blockE
     [loopE
       ([.localGet c, constI n, geUI, .brIf 1] ++ body ++
        [.localGet c, constI 1, addI, .localSet c, .br 0])]]

/-- The counting loop `for c := 0 while c < !ext do body; c := c + 1`, whose
bound is the *contents of local `ext`* rather than a constant.  It is
`countLoop` with the `i32.const n` of the exit test replaced by a `local.get`,
which is the whole difference between a plan whose trip count is fixed by the
plan text and a plan whose trip count is fixed by its input.  The extent local
is read once per test; the body never assigns it, since `Plan.code` gives
`loopReg` a body that writes only the counter register and the plan's own
registers. -/
def countLoopVar (c ext : Nat) (body : List Wasm.Instr) : List Wasm.Instr :=
  [constI 0, .localSet c,
   blockE
     [loopE
       ([.localGet c, .localGet ext, geUI, .brIf 1] ++ body ++
        [.localGet c, constI 1, addI, .localSet c, .br 0])]]

/-- Dispatch on a local holding a small tag. -/
def dispatchOn (t k : Nat) (a b : List Wasm.Instr) : List Wasm.Instr :=
  [.localGet t, constI k, eqI, ifE a b]

/-! ## The compatibility table as emitted code

`Machine.classify` consults `compatible` (SPEC §8.2).  The emitted code
compares a packed key against the closed list of accepted triples;
`mem_compatibleKeys_iff` proves the list is exactly the accepted set, so the
emitted comparison chain is the compatibility table and not an approximation
of it. -/

/-- The packed key of a compatibility triple. -/
def compatKey (m : ArithmeticMode) (stored acc : ScalarKind) : Nat :=
  m.tag * 169 + stored.tag * 13 + acc.tag

/-- The accepted compatibility keys of SPEC §8.2. -/
def compatibleKeys : List Nat :=
  [5, 18, 31, 44, 57, 70, 85, 98,
   175, 189, 201, 215, 227, 241, 253, 267,
   452, 453, 465, 466, 478, 479, 492,
   623, 636, 649, 662]

/-- The emitted key list is exactly the compatibility relation of SPEC §8.2. -/
theorem mem_compatibleKeys_iff (m : ArithmeticMode) (stored acc : ScalarKind) :
    compatKey m stored acc ∈ compatibleKeys ↔ compatible m stored acc = true := by
  revert m stored acc
  decide

theorem compatibleKeys_length : compatibleKeys.length = 27 := rfl

/-! ## Header, classification and layout code -/

/-- Push GNAF memory cell `i`. -/
def cellCode (e : CompileEnv) (i : Nat) : List Wasm.Instr := loadAt (e.memAddr i)

/-- Push the little-endian 16-bit field at cell `i`, exactly
`Machine.u16At i = byteAt i + 256 * byteAt (i+1)`. -/
def u16Code (e : CompileEnv) (i : Nat) : List Wasm.Instr :=
  cellCode e i ++ [constI 256] ++ cellCode e (i + 1) ++ [mulI, addI]

/-- Compare the top of the stack against a constant. -/
def eqConst (n : Nat) : List Wasm.Instr := [constI n, eqI]

/-- Push `Machine.headerOk`: the exact ABI magic, version and header size. -/
def headerOkCode (e : CompileEnv) : List Wasm.Instr :=
  cellCode e 0 ++ eqConst 87 ++
  cellCode e 1 ++ eqConst 71 ++ [andI] ++
  cellCode e 2 ++ eqConst 78 ++ [andI] ++
  cellCode e 3 ++ eqConst 71 ++ [andI] ++
  u16Code e 4 ++ eqConst 1 ++ [andI] ++
  u16Code e 6 ++ eqConst headerSize ++ [andI]

/-- Push whether all three declared tags decode, exactly the `isSome` of
`Machine.declaredTags`. -/
def tagsKnownCode (e : CompileEnv) : List Wasm.Instr :=
  cellCode e 8 ++ [constI 13, ltUI] ++
  cellCode e 11 ++ [constI 13, ltUI, andI] ++
  cellCode e 12 ++ [constI 4, ltUI, andI]

/-- Push the packed compatibility key of the declared tags. -/
def keyCode (e : CompileEnv) : List Wasm.Instr :=
  cellCode e 12 ++ [constI 169, mulI] ++
  cellCode e 8 ++ [constI 13, mulI, addI] ++
  cellCode e 11 ++ [addI]

/-- Or-chain of the accepted compatibility keys. -/
def keyChain (e : CompileEnv) : List Nat → List Wasm.Instr
  | [] => []
  | k :: ks => keyCode e ++ [constI k, eqI, orI] ++ keyChain e ks

/-- Push whether the declared triple is compatible. -/
def compatCode (e : CompileEnv) : List Wasm.Instr :=
  [constI 0] ++ keyChain e compatibleKeys

/-- Compute the classification of SPEC §8.3 into local `t`, in the fixed
precedence order of `Machine.classify`. -/
def classifyCode (e : CompileEnv) (t : Nat) : List Wasm.Instr :=
  if e.memWords < headerSize then [constI 1, .localSet t]
  else
    [constI (if e.memWords ≤ maxRawExtent then 0 else 3), .localSet t] ++
    headerOkCode e ++
    [ifE
      (tagsKnownCode e ++
        [ifE (compatCode e ++ [ifE [] [constI 1, .localSet t]])
             [constI 2, .localSet t]])
      [constI 1, .localSet t]]

/-- Chain computing `Machine.storedWidth`: the byte width of the decoded
stored-kind tag, and zero when the tag decodes to nothing. -/
def widthChain (e : CompileEnv) : List ScalarKind → List Wasm.Instr
  | [] => []
  | k :: ks =>
      cellCode e 8 ++
      [constI k.tag, eqI, constI k.byteWidth, mulI, addI] ++ widthChain e ks

/-- Push `Machine.storedWidth`. -/
def widthCode (e : CompileEnv) : List Wasm.Instr :=
  [constI 0] ++ widthChain e ScalarKind.all

/-- One layout test: `if u16At field = storedWidth then t := cls`. -/
def layoutTestCode (e : CompileEnv) (t field cls : Nat) : List Wasm.Instr :=
  u16Code e field ++ (widthCode e ++ [eqI, ifE [constI cls, .localSet t] []])

/-- Compute the layout class of SPEC §11.1 into local `t`.  The column-major
test is emitted first and the row-major test second, so that the row-major
class wins when both hold, exactly as `Machine.layoutClass` prescribes. -/
def layoutCode (e : CompileEnv) (t : Nat) : List Wasm.Instr :=
  [constI 2, .localSet t] ++ (layoutTestCode e t 56 1 ++ layoutTestCode e t 64 0)

/-- Push the value of a branch condition.  `scratchAtLeast` is decided against
the statically declared scratch extent `scr`, which the typing judgment keeps a
lower bound of the machine's actual scratch. -/
def condCode (e : CompileEnv) (scr : Nat) : Cond → List Wasm.Instr
  | .statusIs s => loadAt e.statusAddr ++ eqConst s.code
  | .regEq r v => [.localGet (e.regLocal r), constI v, eqI]
  | .regLt r v => [.localGet (e.regLocal r), constI v, ltUI]
  | .scratchAtLeast n => [constI (if n ≤ scr then 1 else 0)]

/-- The stores that initialize one code/data table. -/
def tableStores (base : Nat) : Nat → List Nat → List Wasm.Instr
  | _, [] => []
  | j, v :: vs => [constI (base + 4 * j), constI v, storeW] ++ tableStores base (j + 1) vs

/-! ## The released arithmetic subset -/

/-- The arithmetic contracts the released `i32` profile evaluates exactly: the
modular mode with a `u32` accumulator, whose step `(acc + a * b) % 2 ^ 32` is
precisely wrapping `i32` multiply-accumulate. -/
def ArithmeticContract.releasedB (c : ArithmeticContract) : Bool :=
  c.mode == .modular && c.accumulator == .u32

/-- A released contract really has accumulator modulus `2 ^ 32`. -/
theorem ArithmeticContract.accModulus_of_releasedB {c : ArithmeticContract}
    (h : c.releasedB = true) : c.accModulus = 4294967296 := by
  unfold releasedB at h
  simp only [Bool.and_eq_true, beq_iff_eq] at h
  simp [accModulus, h.2, ScalarKind.modulus, ScalarKind.byteWidth]

/-- A released contract's step is exactly wrapping `i32` multiply-accumulate. -/
theorem ArithmeticContract.step_of_releasedB {c : ArithmeticContract}
    (h : c.releasedB = true) (acc a b : Nat) :
    c.step acc a b = (acc + a * b) % 4294967296 := by
  have hm : c.mode = .modular := by
    unfold releasedB at h
    simp only [Bool.and_eq_true, beq_iff_eq] at h
    exact h.1
  simp [step, hm, accModulus_of_releasedB h]

/-- A released contract never reports a checked overflow. -/
theorem ArithmeticContract.overflows_of_releasedB {c : ArithmeticContract}
    (h : c.releasedB = true) (acc a b : Nat) : c.overflows acc a b = false := by
  have hm : c.mode = .modular := by
    unfold releasedB at h
    simp only [Bool.and_eq_true, beq_iff_eq] at h
    exact h.1
  simp [overflows, hm]

/-! ## The plan subset the released profile expresses -/

namespace Plan

/-- The decidable predicate carving out exactly the plans the released profile
can express: no vector operation (there is no SIMD in the profile) and only
released arithmetic contracts. -/
def inReleasedSubset : Plan → Bool
  | nop => true
  | seq a b => a.inReleasedSubset && b.inReleasedSubset
  | classifyRaw v i u e =>
      v.inReleasedSubset && i.inReleasedSubset && u.inReleasedSubset && e.inReleasedSubset
  | dispatchLayout r c g =>
      r.inReleasedSubset && c.inReleasedSubset && g.inReleasedSubset
  | branch _ t f => t.inReleasedSubset && f.inReleasedSubset
  | pack _ _ _ _ => true
  | unpack _ _ _ _ => true
  | storeReg _ _ _ _ => true
  | loadReg _ _ _ _ => true
  | loopNest _ body => body.inReleasedSubset
  | loopReg _ _ _ body => body.inReleasedSubset
  | tiled _ _ _ body => body.inReleasedSubset
  | reduce c _ _ _ => c.releasedB
  | allocScratch _ body => body.inReleasedSubset
  | setReg _ _ => true
  | scalarOp _ _ _ _ => true
  | vectorOp _ _ _ _ _ => false
  | emitTable _ _ => true
  | tableLoad _ _ _ => true
  | setStatus _ => true
  | buildOutput _ => true
  | opaqueProcess _ body => body.inReleasedSubset

/-- Whether the plan mentions a vector operation anywhere. -/
def usesVector : Plan → Bool
  | nop => false
  | seq a b => a.usesVector || b.usesVector
  | classifyRaw v i u e =>
      v.usesVector || i.usesVector || u.usesVector || e.usesVector
  | dispatchLayout r c g => r.usesVector || c.usesVector || g.usesVector
  | branch _ t f => t.usesVector || f.usesVector
  | pack _ _ _ _ => false
  | unpack _ _ _ _ => false
  | storeReg _ _ _ _ => false
  | loadReg _ _ _ _ => false
  | loopNest _ body => body.usesVector
  | loopReg _ _ _ body => body.usesVector
  | tiled _ _ _ body => body.usesVector
  | reduce _ _ _ _ => false
  | allocScratch _ body => body.usesVector
  | setReg _ _ => false
  | scalarOp _ _ _ _ => false
  | vectorOp _ _ _ _ _ => true
  | emitTable _ _ => false
  | tableLoad _ _ _ => false
  | setStatus _ => false
  | buildOutput _ => false
  | opaqueProcess _ body => body.usesVector

/-- The number of cells a plan's code/data tables need. -/
def tableWords : Plan → Nat
  | nop => 0
  | seq a b => Nat.max a.tableWords b.tableWords
  | classifyRaw v i u e =>
      Nat.max (Nat.max v.tableWords i.tableWords) (Nat.max u.tableWords e.tableWords)
  | dispatchLayout r c g =>
      Nat.max r.tableWords (Nat.max c.tableWords g.tableWords)
  | branch _ t f => Nat.max t.tableWords f.tableWords
  | pack _ _ _ _ => 0
  | unpack _ _ _ _ => 0
  | storeReg _ _ _ _ => 0
  | loadReg _ _ _ _ => 0
  | loopNest _ body => body.tableWords
  | loopReg _ _ _ body => body.tableWords
  | tiled _ _ _ body => body.tableWords
  | reduce _ _ _ _ => 0
  | allocScratch _ body => body.tableWords
  | setReg _ _ => 0
  | scalarOp _ _ _ _ => 0
  | vectorOp _ _ _ _ _ => 0
  | emitTable _ data => data.length
  | tableLoad _ index _ => index + 1
  | setStatus _ => 0
  | buildOutput _ => 0
  | opaqueProcess _ body => body.tableWords

end Plan

/-- **A register store is inside the released subset.**  Its translation is one
full-width `i32` store on memory 0 and a handful of `i32` arithmetic
instructions, all of which the released profile of `Wasm/Validate.lean`
admits. -/
@[simp] theorem storeReg_inReleasedSubset (dst : RegionRef) (map : IndexMap)
    (width src : Nat) :
    (Plan.storeReg dst map width src).inReleasedSubset = true := rfl

/-- **A register load is inside the released subset.**  Its translation is one
full-width `i32` load on memory 0, a `local.set` and a handful of `i32`
arithmetic instructions, all of which the released profile of
`Wasm/Validate.lean` admits. -/
@[simp] theorem loadReg_inReleasedSubset (dst : Nat) (src : RegionRef)
    (map : IndexMap) (width : Nat) :
    (Plan.loadReg dst src map width).inReleasedSubset = true := rfl

/-- **A register-driven loop is inside the released subset exactly when its body
is.**  The node itself needs only `block`/`loop`/`br_if`, `local.get` and
`i32` comparison, all of which the released profile admits; it adds no new
refusal point. -/
@[simp] theorem loopReg_inReleasedSubset (ir er : Nat) (map : IndexMap)
    (body : Plan) :
    (Plan.loopReg ir er map body).inReleasedSubset = body.inReleasedSubset := rfl

/-- A plan typed at a *scalar* interface — one admitting no vector lane at all
— contains no vector operation.  The `lanes = 0` restriction on `CheckedPlan`
is therefore exactly a restriction to the plans whose vector fragment the
released profile does not have to express. -/
theorem hasType_no_vector {s : Sig} {p : Plan} {t : Sig} (h : HasType s p t)
    (hl : s.lanes = 0) : p.usesVector = false := by
  induction h with
  | nop s => rfl
  | @seq s u t a b ha _ iha ihb =>
    have hu : u.lanes = s.lanes := (hasType_fixed_resources ha).2.2.1
    simp [Plan.usesVector, iha hl, ihb (by rw [hu, hl])]
  | classifyRaw _ _ _ _ ihv ihi ihu ihe =>
    simp [Plan.usesVector, ihv hl, ihi hl, ihu hl, ihe hl]
  | dispatchLayout _ _ _ ihr ihc ihg =>
    simp [Plan.usesVector, ihr hl, ihc hl, ihg hl]
  | branch _ _ _ ihx ihy => simp [Plan.usesVector, ihx hl, ihy hl]
  | pack => rfl
  | unpack => rfl
  | storeReg => rfl
  | loadReg => rfl
  | loopNest _ _ ih => exact ih hl
  | loopReg _ _ _ ih => exact ih hl
  | tiled _ _ ih => exact ih hl
  | reduce => rfl
  | allocScratch _ ih => exact ih hl
  | setReg => rfl
  | scalarOp => rfl
  | @vectorOp s op lanes dst a b hpos hle _ _ _ =>
    rw [hl] at hle
    exact absurd (Nat.lt_of_lt_of_le hpos hle) (by omega)
  | emitTable => rfl
  | tableLoad => rfl
  | setStatus => rfl
  | buildOutput => rfl
  | opaqueProcess _ _ _ ih => exact ih hl

/-! ## The translation

`code e d scr p` is the operand-stack-neutral instruction sequence of plan `p`
at loop-nesting depth `d` with statically declared scratch extent `scr`. -/

/-- The translation of a plan into the released executable subset. -/
def code (e : CompileEnv) : Nat → Nat → Plan → List Wasm.Instr
  | _, _, .nop => []
  | d, scr, .seq a b => code e d scr a ++ code e d scr b
  | d, scr, .classifyRaw v i u x =>
      classifyCode e (e.tempLocal d) ++
      dispatchOn (e.tempLocal d) 0 (code e d scr v)
        (dispatchOn (e.tempLocal d) 1 (code e d scr i)
          (dispatchOn (e.tempLocal d) 2 (code e d scr u) (code e d scr x)))
  | d, scr, .dispatchLayout r c g =>
      layoutCode e (e.tempLocal d) ++
      dispatchOn (e.tempLocal d) 0 (code e d scr r)
        (dispatchOn (e.tempLocal d) 1 (code e d scr c) (code e d scr g))
  | d, scr, .branch co t f =>
      condCode e scr co ++ [ifE (code e d scr t) (code e d scr f)]
  | d, _, .pack src dst map _ =>
      countLoop (e.tempLocal d) src.count
        ([constI (e.scratchAddr dst.base), .localGet (e.tempLocal d), constI 4, mulI,
          addI] ++
         [constI (e.memAddr (src.base + map.c0)), .localGet (e.tempLocal d),
          constI (4 * map.cb), mulI, addI, loadW] ++
         [storeW])
  | d, _, .unpack src dst map _ =>
      countLoop (e.tempLocal d) src.count
        ([constI (e.memAddr dst.base), .localGet (e.tempLocal d), constI 4, mulI, addI] ++
         [constI (e.scratchAddr (src.base + map.c0)), .localGet (e.tempLocal d),
          constI (4 * map.cb), mulI, addI, loadW] ++
         [storeW])
  | _, _, .storeReg dst map _ src =>
      [constI (e.memAddr (dst.base + map.c0)),
       .localGet (e.regLocal 0), constI (4 * map.cb), mulI, addI,
       .localGet (e.regLocal 1), constI (4 * map.ci), mulI, addI,
       .localGet (e.regLocal 2), constI (4 * map.cj), mulI, addI,
       .localGet (e.regLocal src), storeW]
  | _, _, .loadReg dst src map _ =>
      [constI (e.memAddr (src.base + map.c0)),
       .localGet (e.regLocal 0), constI (4 * map.cb), mulI, addI,
       .localGet (e.regLocal 1), constI (4 * map.ci), mulI, addI,
       .localGet (e.regLocal 2), constI (4 * map.cj), mulI, addI,
       loadW, .localSet (e.regLocal dst)]
  | d, scr, .loopNest axis body =>
      countLoop (e.tempLocal d) axis.extent
        ([.localGet (e.tempLocal d), .localSet (e.regLocal axis.indexReg)] ++
         code e (d + 1) scr body)
  | d, scr, .loopReg ir er _ body =>
      countLoopVar (e.tempLocal d) (e.regLocal er)
        ([.localGet (e.tempLocal d), .localSet (e.regLocal ir)] ++
         code e (d + 1) scr body)
  | d, scr, .tiled _ tiling extents body =>
      countLoop (e.tempLocal d) (tiling.totalTiles extents.m extents.n extents.k)
        ((if 0 < e.regs then [.localGet (e.tempLocal d), .localSet (e.regLocal 0)] else []) ++
         code e (d + 1) scr body)
  | d, _, .reduce c acc lhs rhs =>
      if c.releasedB then
        countLoop (e.tempLocal d) (Nat.min lhs.count rhs.count)
          ([.localGet (e.regLocal acc)] ++
           [constI (e.memAddr lhs.base), .localGet (e.tempLocal d), constI 4, mulI,
            addI, loadW] ++
           [constI (e.memAddr rhs.base), .localGet (e.tempLocal d), constI 4, mulI,
            addI, loadW] ++
           [mulI, addI, .localSet (e.regLocal acc)])
      else [.unreachable]
  | d, scr, .allocScratch bytes body => code e d (scr + bytes) body
  | _, _, .setReg dst v => [constI v, .localSet (e.regLocal dst)]
  | _, _, .scalarOp op dst a b =>
      match op with
      | .add =>
          [.localGet (e.regLocal a), .localGet (e.regLocal b), addI,
           .localSet (e.regLocal dst)]
      | .mul =>
          [.localGet (e.regLocal a), .localGet (e.regLocal b), mulI,
           .localSet (e.regLocal dst)]
      | .sub =>
          [.localGet (e.regLocal a), .localGet (e.regLocal b), geUI,
           ifE [.localGet (e.regLocal a), .localGet (e.regLocal b), subI,
                .localSet (e.regLocal dst)]
               [constI 0, .localSet (e.regLocal dst)]]
      | .max =>
          [.localGet (e.regLocal a), .localGet (e.regLocal b), gtUI,
           ifE [.localGet (e.regLocal a), .localSet (e.regLocal dst)]
               [.localGet (e.regLocal b), .localSet (e.regLocal dst)]]
      | .min =>
          [.localGet (e.regLocal a), .localGet (e.regLocal b), ltUI,
           ifE [.localGet (e.regLocal a), .localSet (e.regLocal dst)]
               [.localGet (e.regLocal b), .localSet (e.regLocal dst)]]
  | _, _, .vectorOp _ _ _ _ _ => [.unreachable]
  | _, _, .emitTable index data =>
      tableStores (e.tableBase + 4 * (index * e.tableStride)) 0 data
  | _, _, .tableLoad table index dst =>
      loadAt (e.tableBase + 4 * (table * e.tableStride + index)) ++
      [.localSet (e.regLocal dst)]
  | _, _, .setStatus s => [constI e.statusAddr, constI s.code, storeW]
  | _, _, .buildOutput src =>
      [constI e.outPtrAddr, constI (e.memAddr src.base), storeW,
       constI e.outLenAddr, constI src.count, storeW]
  | d, scr, .opaqueProcess _ body => code e d scr body

/-- The complete body of the emitted `gemm` function: the plan's code, then the
status word, which is the ABI result. -/
def bodyCode (e : CompileEnv) (scr : Nat) (p : Plan) : List Wasm.Instr :=
  code e 0 scr p ++ loadAt e.statusAddr

/-! ## The emitted module -/

/-- The single recursive type group of the emitted module: the pinned GEMM ABI
function type. -/
def gemmRecType : Wasm.RecType :=
  { types := [{ final := true, supertypes := [], body := .func Wasm.gemmFuncType }] }

/-- The module emitted from a layout and a function body. -/
def moduleOf (e : CompileEnv) (body : List Wasm.Instr) : Wasm.Module :=
  { types := [gemmRecType]
    imports := []
    funcs :=
      [{ type := 0
         locals := List.replicate e.declaredLocals (Wasm.ValType.num .i32)
         body := Wasm.Expr.ofList body }]
    tables := []
    mems := [{ type := { addressType := .i32
                         limits := { min := e.pages, max := some e.pages } } }]
    tags := []
    globals := []
    exports :=
      [{ name := Wasm.gemmExportName, desc := .func 0 },
       { name := Wasm.memoryExportName, desc := .mem 0 }]
    start := none
    elems := []
    datas := [] }

/-- The compilation environment of a plan checked at interface `s`. -/
def envOf (s : Sig) (p : Plan) : CompileEnv :=
  { regs := s.regs
    memWords := s.mem
    scratchWords := s.scratch + p.charges.scratchPeak
    tableCount := s.tables
    tableStride := p.tableWords
    depthBound := p.depth }

/-- **SPEC §11.4.**  The GNAF-to-Wasm compiler. -/
def compile {s t : Sig} (c : CheckedPlan s t) : Wasm.Module :=
  moduleOf (envOf s c.plan) (bodyCode (envOf s c.plan) s.scratch c.plan)

/-- **SPEC §11.4, `compile_deterministic`.**  Compilation is a function: one
checked plan has exactly one emitted module. -/
theorem compile_deterministic {s t : Sig} (c c' : CheckedPlan s t) (h : c = c') :
    compile c = compile c' := by rw [h]

/-- Compilation depends only on the plan, not on the typing derivation: two
checked plans with the same underlying plan emit the same module. -/
theorem compile_plan_congr {s t : Sig} (c c' : CheckedPlan s t)
    (h : c.plan = c'.plan) : compile c = compile c' := by
  unfold compile
  rw [h]

/-! ## Typing of the emitted instruction sequences

`Wasm/Validate.lean` decides typing on `Wasm.Expr`; the compiler produces
`List Wasm.Instr`.  `checkList` is the checker transported along `Expr.ofList`,
and the lemmas below build up to `checkList_code`: the translation of *any*
well-typed plan is operand-stack neutral in every context with enough locals. -/

/-- The release checker on an instruction list. -/
def checkList (C : Wasm.Ctx) (h : Nat) (l : List Wasm.Instr) : Option Nat :=
  Wasm.checkExpr C h (Wasm.Expr.ofList l)

@[simp] theorem checkList_nil (C : Wasm.Ctx) (h : Nat) : checkList C h [] = some h := rfl

@[simp] theorem checkList_cons (C : Wasm.Ctx) (h : Nat) (i : Wasm.Instr)
    (l : List Wasm.Instr) :
    checkList C h (i :: l) =
      (Wasm.checkInstr C h i).bind (fun h' => checkList C h' l) := by
  show Wasm.checkExpr C h (.cons i (Wasm.Expr.ofList l)) = _
  cases hi : Wasm.checkInstr C h i <;> simp [Wasm.checkExpr, hi, checkList]

theorem checkList_append (C : Wasm.Ctx) (h : Nat) (a b : List Wasm.Instr) :
    checkList C h (a ++ b) = (checkList C h a).bind (fun h' => checkList C h' b) := by
  induction a generalizing h with
  | nil => simp
  | cons i l ih => cases hi : Wasm.checkInstr C h i <;> simp [hi, ih]

/-- Composition of two typed segments. -/
theorem checkList_app2 {C : Wasm.Ctx} {h k m : Nat} {a b : List Wasm.Instr}
    (ha : checkList C h a = some k) (hb : checkList C k b = some m) :
    checkList C h (a ++ b) = some m := by
  rw [checkList_append, ha]; exact hb

/-- Composition of a leading instruction and a typed segment. -/
theorem checkList_cons2 {C : Wasm.Ctx} {h k m : Nat} {i : Wasm.Instr}
    {l : List Wasm.Instr} (hi : Wasm.checkInstr C h i = some k)
    (hl : checkList C k l = some m) : checkList C h (i :: l) = some m := by
  simp [hi, hl]

/-- The context in which a structured control instruction checks its body. -/
abbrev pushLabel (C : Wasm.Ctx) (h : Nat) : Wasm.Ctx := { C with labels := h :: C.labels }

@[simp] theorem pushLabel_numLocals (C : Wasm.Ctx) (h : Nat) :
    (pushLabel C h).numLocals = C.numLocals := rfl

theorem checkList_ifE (C : Wasm.Ctx) (h : Nat) (a b : List Wasm.Instr)
    (ha : checkList (pushLabel C h) h a = some h)
    (hb : checkList (pushLabel C h) h b = some h) :
    checkList C (h + 1) [ifE a b] = some h := by
  unfold checkList at *
  simp only [Wasm.Expr.ofList, Wasm.checkExpr, ifE, Wasm.checkInstr, ha, hb, and_self,
    if_pos]

theorem checkList_blockE (C : Wasm.Ctx) (h : Nat) (b : List Wasm.Instr)
    (hb : checkList (pushLabel C h) h b = some h) :
    checkList C h [blockE b] = some h := by
  unfold checkList at *
  simp only [Wasm.Expr.ofList, Wasm.checkExpr, blockE, Wasm.checkInstr, hb, if_pos]

theorem checkList_loopE (C : Wasm.Ctx) (h : Nat) (b : List Wasm.Instr)
    (hb : checkList (pushLabel C h) h b = some h) :
    checkList C h [loopE b] = some h := by
  unfold checkList at *
  simp only [Wasm.Expr.ofList, Wasm.checkExpr, loopE, Wasm.checkInstr, hb, if_pos]

theorem checkList_localGet (C : Wasm.Ctx) (h i : Nat) (hi : i < C.numLocals) :
    checkList C h [Wasm.Instr.localGet i] = some (h + 1) := by
  simp [Wasm.checkInstr, hi]

theorem checkList_localSet (C : Wasm.Ctx) (h i : Nat) (hi : i < C.numLocals) :
    checkList C (h + 1) [Wasm.Instr.localSet i] = some h := by
  simp [Wasm.checkInstr, hi]

theorem checkList_setConst (C : Wasm.Ctx) (h i n : Nat) (hi : i < C.numLocals) :
    checkList C h [constI n, Wasm.Instr.localSet i] = some h := by
  simp [Wasm.checkInstr, constI, hi]

/-- The counting loop is operand-stack neutral. -/
theorem checkList_countLoop (C : Wasm.Ctx) (h c n : Nat) (body : List Wasm.Instr)
    (hc : c < C.numLocals)
    (hbody : checkList (pushLabel (pushLabel C h) h) h body = some h) :
    checkList C h (countLoop c n body) = some h := by
  refine checkList_app2 (checkList_setConst C h c 0 hc) ?_
  refine checkList_blockE C h _ ?_
  refine checkList_loopE _ h _ ?_
  rw [List.append_assoc]
  refine checkList_app2 (k := h) (b := body ++ _) ?_ ?_
  · simp [Wasm.checkInstr, constI, geUI, hc]
  · refine checkList_app2 hbody ?_
    simp [Wasm.checkInstr, constI, addI, Wasm.SubsetIBinOp, hc]

/-- The register-bounded counting loop is operand-stack neutral. -/
theorem checkList_countLoopVar (C : Wasm.Ctx) (h c ext : Nat) (body : List Wasm.Instr)
    (hc : c < C.numLocals) (hext : ext < C.numLocals)
    (hbody : checkList (pushLabel (pushLabel C h) h) h body = some h) :
    checkList C h (countLoopVar c ext body) = some h := by
  refine checkList_app2 (checkList_setConst C h c 0 hc) ?_
  refine checkList_blockE C h _ ?_
  refine checkList_loopE _ h _ ?_
  rw [List.append_assoc]
  refine checkList_app2 (k := h) (b := body ++ _) ?_ ?_
  · simp [Wasm.checkInstr, geUI, hc, hext]
  · refine checkList_app2 hbody ?_
    simp [Wasm.checkInstr, constI, addI, Wasm.SubsetIBinOp, hc]

/-- A tag dispatch is operand-stack neutral. -/
theorem checkList_dispatchOn (C : Wasm.Ctx) (h t k : Nat) (a b : List Wasm.Instr)
    (ht : t < C.numLocals)
    (ha : checkList (pushLabel C h) h a = some h)
    (hb : checkList (pushLabel C h) h b = some h) :
    checkList C h (dispatchOn t k a b) = some h := by
  unfold dispatchOn
  refine checkList_app2 (a := [Wasm.Instr.localGet t, constI k, eqI]) ?_
    (checkList_ifE C h a b ha hb)
  simp [Wasm.checkInstr, constI, eqI, ht]

/-! ### Typing of the fixed code fragments -/

theorem checkList_cellCode (e : CompileEnv) (C : Wasm.Ctx) (h i : Nat) :
    checkList C h (cellCode e i) = some (h + 1) := by
  simp [cellCode, loadAt, Wasm.checkInstr, constI, loadW, memArg]

theorem checkList_loadAt (C : Wasm.Ctx) (h a : Nat) :
    checkList C h (loadAt a) = some (h + 1) := by
  simp [loadAt, Wasm.checkInstr, constI, loadW, memArg]

theorem checkList_u16Code (e : CompileEnv) (C : Wasm.Ctx) (h i : Nat) :
    checkList C h (u16Code e i) = some (h + 1) := by
  simp [u16Code, cellCode, loadAt, Wasm.checkInstr, constI, loadW, mulI, addI, memArg,
    Wasm.SubsetIBinOp]

theorem checkList_headerOkCode (e : CompileEnv) (C : Wasm.Ctx) (h : Nat) :
    checkList C h (headerOkCode e) = some (h + 1) := by
  simp [headerOkCode, cellCode, u16Code, loadAt, eqConst, Wasm.checkInstr, constI, loadW,
    eqI, andI, mulI, addI, memArg, Wasm.SubsetIBinOp]

theorem checkList_tagsKnownCode (e : CompileEnv) (C : Wasm.Ctx) (h : Nat) :
    checkList C h (tagsKnownCode e) = some (h + 1) := by
  simp [tagsKnownCode, cellCode, loadAt, Wasm.checkInstr, constI, loadW, ltUI, andI,
    memArg, Wasm.SubsetIBinOp]

theorem checkList_keyCode (e : CompileEnv) (C : Wasm.Ctx) (h : Nat) :
    checkList C h (keyCode e) = some (h + 1) := by
  simp [keyCode, cellCode, loadAt, Wasm.checkInstr, constI, loadW, mulI, addI, memArg,
    Wasm.SubsetIBinOp]

theorem checkList_keyChain (e : CompileEnv) (C : Wasm.Ctx) (h : Nat) :
    ∀ ks : List Nat, checkList C (h + 1) (keyChain e ks) = some (h + 1) := by
  intro ks
  induction ks with
  | nil => rfl
  | cons k ks ih =>
    rw [keyChain, List.append_assoc]
    refine checkList_app2 (checkList_keyCode e C (h + 1)) ?_
    refine checkList_app2 (a := [constI k, eqI, orI]) ?_ ih
    simp [Wasm.checkInstr, constI, eqI, orI, Wasm.SubsetIBinOp]

theorem checkList_compatCode (e : CompileEnv) (C : Wasm.Ctx) (h : Nat) :
    checkList C h (compatCode e) = some (h + 1) := by
  refine checkList_app2 (a := [constI 0]) ?_ (checkList_keyChain e C h compatibleKeys)
  simp [Wasm.checkInstr, constI]

theorem checkList_widthChain (e : CompileEnv) (C : Wasm.Ctx) (h : Nat) :
    ∀ ks : List ScalarKind, checkList C (h + 1) (widthChain e ks) = some (h + 1) := by
  intro ks
  induction ks with
  | nil => rfl
  | cons k ks ih =>
    rw [widthChain, List.append_assoc]
    refine checkList_app2 (checkList_cellCode e C (h + 1) 8) ?_
    refine checkList_app2
      (a := [constI k.tag, eqI, constI k.byteWidth, mulI, addI]) ?_ ih
    simp [Wasm.checkInstr, constI, eqI, mulI, addI, Wasm.SubsetIBinOp]

theorem checkList_widthCode (e : CompileEnv) (C : Wasm.Ctx) (h : Nat) :
    checkList C h (widthCode e) = some (h + 1) := by
  refine checkList_app2 (a := [constI 0]) ?_ (checkList_widthChain e C h ScalarKind.all)
  simp [Wasm.checkInstr, constI]

theorem checkList_classifyCode (e : CompileEnv) (C : Wasm.Ctx) (h t : Nat)
    (ht : t < C.numLocals) : checkList C h (classifyCode e t) = some h := by
  unfold classifyCode
  split
  · exact checkList_setConst C h t 1 ht
  · simp only [List.append_assoc]
    refine checkList_app2 (checkList_setConst C h t _ ht) ?_
    refine checkList_app2 (checkList_headerOkCode e C h) ?_
    refine checkList_ifE C h _ _ ?_ (checkList_setConst _ h t 1 ht)
    refine checkList_app2 (checkList_tagsKnownCode e _ h) ?_
    refine checkList_ifE _ h _ _ ?_ (checkList_setConst _ h t 2 ht)
    refine checkList_app2 (checkList_compatCode e _ h) ?_
    exact checkList_ifE _ h _ _ (checkList_nil _ h) (checkList_setConst _ h t 1 ht)

theorem checkList_layoutTestCode (e : CompileEnv) (C : Wasm.Ctx) (h t field cls : Nat)
    (ht : t < C.numLocals) : checkList C h (layoutTestCode e t field cls) = some h := by
  unfold layoutTestCode
  refine checkList_app2 (checkList_u16Code e C h field) ?_
  refine checkList_app2 (checkList_widthCode e C (h + 1)) ?_
  refine checkList_cons2 (k := h + 1) ?_
    (checkList_ifE C h _ _ (checkList_setConst _ h t cls ht) (checkList_nil _ h))
  simp [Wasm.checkInstr, eqI]

theorem checkList_layoutCode (e : CompileEnv) (C : Wasm.Ctx) (h t : Nat)
    (ht : t < C.numLocals) : checkList C h (layoutCode e t) = some h := by
  unfold layoutCode
  refine checkList_app2 (checkList_setConst C h t 2 ht) ?_
  exact checkList_app2 (checkList_layoutTestCode e C h t 56 1 ht)
    (checkList_layoutTestCode e C h t 64 0 ht)

theorem checkList_tableStores (C : Wasm.Ctx) (h base : Nat) :
    ∀ (vs : List Nat) (j : Nat), checkList C h (tableStores base j vs) = some h := by
  intro vs
  induction vs with
  | nil => intro j; rfl
  | cons v vs ih =>
    intro j
    refine checkList_app2 (a := [constI (base + 4 * j), constI v, storeW]) ?_ (ih (j + 1))
    simp [Wasm.checkInstr, constI, storeW, memArg]

theorem checkList_condCode (e : CompileEnv) (C : Wasm.Ctx) (h scr : Nat) (co : Cond)
    (hco : ∀ r, co.readsReg = some r → e.regLocal r < C.numLocals) :
    checkList C h (condCode e scr co) = some (h + 1) := by
  cases co with
  | statusIs s =>
    refine checkList_app2 (checkList_loadAt C h e.statusAddr) ?_
    simp [eqConst, Wasm.checkInstr, constI, eqI]
  | regEq r v =>
    have hr : e.regLocal r < C.numLocals := hco r rfl
    simp [condCode, Wasm.checkInstr, constI, eqI, hr]
  | regLt r v =>
    have hr : e.regLocal r < C.numLocals := hco r rfl
    simp [condCode, Wasm.checkInstr, constI, ltUI, hr]
  | scratchAtLeast n => simp [condCode, Wasm.checkInstr, constI]

/-! ### Typing of the translation -/

/-- **The translation is well typed.**  For every well-typed plan, in every
context declaring enough locals, the emitted instruction sequence is accepted by
the release checker and is operand-stack neutral. -/
theorem checkList_code {s : Sig} {p : Plan} {t : Sig} (hp : HasType s p t) :
    ∀ (e : CompileEnv) (d scr : Nat) (C : Wasm.Ctx) (h : Nat),
      e.regs = s.regs → 2 + e.regs + d + p.depth + 1 ≤ C.numLocals →
      checkList C h (code e d scr p) = some h := by
  induction hp with
  | nop s => intro e d scr C h _ _; rfl
  | @seq s u t a b ha _ iha ihb =>
    intro e d scr C h hreg hloc
    have hu : u.regs = s.regs := (hasType_fixed_resources ha).1
    simp only [Plan.depth] at hloc
    have h1 : a.depth ≤ Nat.max a.depth b.depth := Nat.le_max_left _ _
    have h2 : b.depth ≤ Nat.max a.depth b.depth := Nat.le_max_right _ _
    simp only [code]
    exact checkList_app2 (iha e d scr C h hreg (by omega))
      (ihb e d scr C h (by rw [hreg, hu]) (by omega))
  | @classifyRaw s t v i u x hv _ _ _ ihv ihi ihu ihx =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    have h1 : v.depth ≤ Nat.max (Nat.max v.depth i.depth) (Nat.max u.depth x.depth) :=
      Nat.le_trans (Nat.le_max_left _ _) (Nat.le_max_left _ _)
    have h2 : i.depth ≤ Nat.max (Nat.max v.depth i.depth) (Nat.max u.depth x.depth) :=
      Nat.le_trans (Nat.le_max_right _ _) (Nat.le_max_left _ _)
    have h3 : u.depth ≤ Nat.max (Nat.max v.depth i.depth) (Nat.max u.depth x.depth) :=
      Nat.le_trans (Nat.le_max_left _ _) (Nat.le_max_right _ _)
    have h4 : x.depth ≤ Nat.max (Nat.max v.depth i.depth) (Nat.max u.depth x.depth) :=
      Nat.le_trans (Nat.le_max_right _ _) (Nat.le_max_right _ _)
    have hT : e.tempLocal d < C.numLocals := by
      unfold CompileEnv.tempLocal; omega
    have hT' : ∀ k : Nat, e.tempLocal d < (pushLabel C k).numLocals := fun _ => hT
    simp only [code]
    refine checkList_app2 (checkList_classifyCode e C h _ hT) ?_
    refine checkList_dispatchOn C h _ 0 _ _ hT
      (ihv e d scr _ h hreg (by simp only [pushLabel_numLocals]; omega)) ?_
    refine checkList_dispatchOn _ h _ 1 _ _ (hT' h)
      (ihi e d scr _ h hreg (by simp only [pushLabel_numLocals]; omega)) ?_
    exact checkList_dispatchOn _ h _ 2 _ _ (hT' h)
      (ihu e d scr _ h hreg (by simp only [pushLabel_numLocals]; omega))
      (ihx e d scr _ h hreg (by simp only [pushLabel_numLocals]; omega))
  | @dispatchLayout s t r c g _ _ _ ihr ihc ihg =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    have h1 : r.depth ≤ Nat.max r.depth (Nat.max c.depth g.depth) := Nat.le_max_left _ _
    have h2 : c.depth ≤ Nat.max r.depth (Nat.max c.depth g.depth) :=
      Nat.le_trans (Nat.le_max_left _ _) (Nat.le_max_right _ _)
    have h3 : g.depth ≤ Nat.max r.depth (Nat.max c.depth g.depth) :=
      Nat.le_trans (Nat.le_max_right _ _) (Nat.le_max_right _ _)
    have hT : e.tempLocal d < C.numLocals := by
      unfold CompileEnv.tempLocal; omega
    have hT' : ∀ k : Nat, e.tempLocal d < (pushLabel C k).numLocals := fun _ => hT
    simp only [code]
    refine checkList_app2 (checkList_layoutCode e C h _ hT) ?_
    refine checkList_dispatchOn C h _ 0 _ _ hT
      (ihr e d scr _ h hreg (by simp only [pushLabel_numLocals]; omega)) ?_
    exact checkList_dispatchOn _ h _ 1 _ _ (hT' h)
      (ihc e d scr _ h hreg (by simp only [pushLabel_numLocals]; omega))
      (ihg e d scr _ h hreg (by simp only [pushLabel_numLocals]; omega))
  | @branch s t co x y hco _ _ ihx ihy =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    have h1 : x.depth ≤ Nat.max x.depth y.depth := Nat.le_max_left _ _
    have h2 : y.depth ≤ Nat.max x.depth y.depth := Nat.le_max_right _ _
    simp only [code]
    refine checkList_app2 (checkList_condCode e C h scr co ?_) ?_
    · intro r hr
      have : r < s.regs := by
        cases co with
        | statusIs _ => simp [Cond.readsReg] at hr
        | regEq r' v => cases hr; exact of_decide_eq_true hco
        | regLt r' v => cases hr; exact of_decide_eq_true hco
        | scratchAtLeast _ => simp [Cond.readsReg] at hr
      unfold CompileEnv.regLocal; omega
    · exact checkList_ifE C h _ _
        (ihx e d scr _ h hreg (by simp only [pushLabel_numLocals]; omega))
        (ihy e d scr _ h hreg (by simp only [pushLabel_numLocals]; omega))
  | @pack s src dst map w _ _ _ =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    simp only [code]
    have hT : e.tempLocal d < C.numLocals := by unfold CompileEnv.tempLocal; omega
    refine checkList_countLoop C h _ _ _ hT ?_
    simp [Wasm.checkInstr, constI, mulI, addI, loadW, storeW, memArg,
      Wasm.SubsetIBinOp, hT]
  | @unpack s src dst map w _ _ _ =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    simp only [code]
    have hT : e.tempLocal d < C.numLocals := by unfold CompileEnv.tempLocal; omega
    refine checkList_countLoop C h _ _ _ hT ?_
    simp [Wasm.checkInstr, constI, mulI, addI, loadW, storeW, memArg,
      Wasm.SubsetIBinOp, hT]
  | @storeReg s dst map w src _ _ hsrc hregs =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    simp only [code]
    have h0 : e.regLocal 0 < C.numLocals := by unfold CompileEnv.regLocal; omega
    have h1 : e.regLocal 1 < C.numLocals := by unfold CompileEnv.regLocal; omega
    have h2 : e.regLocal 2 < C.numLocals := by unfold CompileEnv.regLocal; omega
    have hv : e.regLocal src < C.numLocals := by unfold CompileEnv.regLocal; omega
    simp [Wasm.checkInstr, constI, mulI, addI, storeW, memArg, Wasm.SubsetIBinOp,
      h0, h1, h2, hv]
  | @loadReg s dst src map w _ _ hdst hregs =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    simp only [code]
    have h0 : e.regLocal 0 < C.numLocals := by unfold CompileEnv.regLocal; omega
    have h1 : e.regLocal 1 < C.numLocals := by unfold CompileEnv.regLocal; omega
    have h2 : e.regLocal 2 < C.numLocals := by unfold CompileEnv.regLocal; omega
    have hv : e.regLocal dst < C.numLocals := by unfold CompileEnv.regLocal; omega
    simp [Wasm.checkInstr, constI, mulI, addI, loadW, memArg, Wasm.SubsetIBinOp,
      h0, h1, h2, hv]
  | @loopReg s ir er map body hir her _ ih =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    simp only [code]
    have hT : e.tempLocal d < C.numLocals := by unfold CompileEnv.tempLocal; omega
    have hI : e.regLocal ir < C.numLocals := by unfold CompileEnv.regLocal; omega
    have hE : e.regLocal er < C.numLocals := by unfold CompileEnv.regLocal; omega
    refine checkList_countLoopVar C h _ _ _ hT hE ?_
    refine checkList_app2
      (a := [Wasm.Instr.localGet (e.tempLocal d),
             Wasm.Instr.localSet (e.regLocal ir)]) ?_
      (ih e (d + 1) scr _ h hreg (by simp only [pushLabel_numLocals]; omega))
    simp [Wasm.checkInstr, hT, hI]
  | @loopNest s axis body hix _ ih =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    simp only [code]
    have hT : e.tempLocal d < C.numLocals := by unfold CompileEnv.tempLocal; omega
    have hR : e.regLocal axis.indexReg < C.numLocals := by
      unfold CompileEnv.regLocal; omega
    refine checkList_countLoop C h _ _ _ hT ?_
    refine checkList_app2
      (a := [Wasm.Instr.localGet (e.tempLocal d),
             Wasm.Instr.localSet (e.regLocal axis.indexReg)]) ?_
      (ih e (d + 1) scr _ h hreg (by simp only [pushLabel_numLocals]; omega))
    simp [Wasm.checkInstr, hT, hR]
  | @tiled s o ti ex body _ _ ih =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    simp only [code]
    have hT : e.tempLocal d < C.numLocals := by unfold CompileEnv.tempLocal; omega
    refine checkList_countLoop C h _ _ _ hT ?_
    refine checkList_app2 ?_
      (ih e (d + 1) scr _ h hreg (by simp only [pushLabel_numLocals]; omega))
    by_cases hr : 0 < e.regs
    · have hR : e.regLocal 0 < C.numLocals := by unfold CompileEnv.regLocal; omega
      simp [hr, Wasm.checkInstr, hT, hR]
    · simp [hr]
  | @reduce s c acc lhs rhs hacc _ _ _ =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    simp only [code]
    have hT : e.tempLocal d < C.numLocals := by unfold CompileEnv.tempLocal; omega
    have hR : e.regLocal acc < C.numLocals := by unfold CompileEnv.regLocal; omega
    split
    · refine checkList_countLoop C h _ _ _ hT ?_
      simp [Wasm.checkInstr, constI, mulI, addI, loadW, memArg, Wasm.SubsetIBinOp,
        hT, hR]
    · simp [Wasm.checkInstr]
  | @allocScratch s t n body _ ih =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    simp only [code]
    exact ih e d (scr + n) C h hreg (by omega)
  | @setReg s dst v hdst =>
    intro e d scr C h hreg hloc
    have hR : e.regLocal dst < C.numLocals := by unfold CompileEnv.regLocal; omega
    simp only [code]
    exact checkList_setConst C h _ v hR
  | @scalarOp s op dst a b hdst hha hhb =>
    intro e d scr C h hreg hloc
    have hD : e.regLocal dst < C.numLocals := by unfold CompileEnv.regLocal; omega
    have hA : e.regLocal a < C.numLocals := by unfold CompileEnv.regLocal; omega
    have hB : e.regLocal b < C.numLocals := by unfold CompileEnv.regLocal; omega
    simp only [code]
    cases op with
    | add => simp [Wasm.checkInstr, addI, Wasm.SubsetIBinOp, hD, hA, hB]
    | mul => simp [Wasm.checkInstr, mulI, Wasm.SubsetIBinOp, hD, hA, hB]
    | sub =>
      refine checkList_cons2 (k := h + 1) (by simp [Wasm.checkInstr, hA]) ?_
      refine checkList_cons2 (k := h + 2) (by simp [Wasm.checkInstr, hB]) ?_
      refine checkList_cons2 (k := h + 1) (by simp [Wasm.checkInstr, geUI]) ?_
      refine checkList_ifE C h _ _ ?_ ?_
      · simp [Wasm.checkInstr, subI, Wasm.SubsetIBinOp, hD, hA, hB]
      · simp [Wasm.checkInstr, constI, hD]
    | max =>
      refine checkList_cons2 (k := h + 1) (by simp [Wasm.checkInstr, hA]) ?_
      refine checkList_cons2 (k := h + 2) (by simp [Wasm.checkInstr, hB]) ?_
      refine checkList_cons2 (k := h + 1) (by simp [Wasm.checkInstr, gtUI]) ?_
      exact checkList_ifE C h _ _ (by simp [Wasm.checkInstr, hD, hA])
        (by simp [Wasm.checkInstr, hD, hB])
    | min =>
      refine checkList_cons2 (k := h + 1) (by simp [Wasm.checkInstr, hA]) ?_
      refine checkList_cons2 (k := h + 2) (by simp [Wasm.checkInstr, hB]) ?_
      refine checkList_cons2 (k := h + 1) (by simp [Wasm.checkInstr, ltUI]) ?_
      exact checkList_ifE C h _ _ (by simp [Wasm.checkInstr, hD, hA])
        (by simp [Wasm.checkInstr, hD, hB])
  | @vectorOp s op lanes dst a b _ _ _ _ _ =>
    intro e d scr C h hreg hloc
    simp only [code]
    simp [Wasm.checkInstr]
  | @emitTable s idx data _ =>
    intro e d scr C h hreg hloc
    simp only [code]
    exact checkList_tableStores C h _ data 0
  | @tableLoad s tbl idx dst _ hdst =>
    intro e d scr C h hreg hloc
    have hD : e.regLocal dst < C.numLocals := by unfold CompileEnv.regLocal; omega
    simp only [code]
    refine checkList_app2 (checkList_loadAt C h _) ?_
    exact checkList_localSet C h _ hD
  | setStatus s st =>
    intro e d scr C h hreg hloc
    simp only [code]
    simp [Wasm.checkInstr, constI, storeW, memArg]
  | @buildOutput s src _ _ =>
    intro e d scr C h hreg hloc
    simp only [code]
    simp [Wasm.checkInstr, constI, storeW, memArg]
  | @opaqueProcess s t spec body _ _ _ ih =>
    intro e d scr C h hreg hloc
    simp only [Plan.depth] at hloc
    simp only [code]
    exact ih e d scr C h hreg (by omega)

/-! ## Structural invariants of the emitted module -/

namespace CompiledModule

variable (e : CompileEnv) (b : List Wasm.Instr)

@[simp] theorem imports : (moduleOf e b).imports = [] := rfl
@[simp] theorem tables : (moduleOf e b).tables = [] := rfl
@[simp] theorem elems : (moduleOf e b).elems = [] := rfl
@[simp] theorem datas : (moduleOf e b).datas = [] := rfl
@[simp] theorem globals : (moduleOf e b).globals = [] := rfl
@[simp] theorem tags : (moduleOf e b).tags = [] := rfl
@[simp] theorem start : (moduleOf e b).start = none := rfl
@[simp] theorem funcs_length : (moduleOf e b).funcs.length = 1 := rfl
@[simp] theorem mems_length : (moduleOf e b).mems.length = 1 := rfl

theorem funcs_eq :
    (moduleOf e b).funcs =
      [{ type := 0
         locals := List.replicate e.declaredLocals (Wasm.ValType.num .i32)
         body := Wasm.Expr.ofList b }] := rfl

theorem exports_eq :
    (moduleOf e b).exports =
      [{ name := Wasm.gemmExportName, desc := .func 0 },
       { name := Wasm.memoryExportName, desc := .mem 0 }] := rfl

end CompiledModule

/-- **SPEC §11.4, structural invariant.**  The emitted module declares no
imports: it is closed. -/
theorem compile_isClosed {s t : Sig} (c : CheckedPlan s t) :
    (compile c).IsClosed := rfl

/-- **SPEC §11.4, structural invariant.**  The emitted module exports the
required `gemm` function and the required memory, in that order. -/
theorem compile_exports {s t : Sig} (c : CheckedPlan s t) :
    (compile c).exports =
      [{ name := Wasm.gemmExportName, desc := .func 0 },
       { name := Wasm.memoryExportName, desc := .mem 0 }] := rfl

/-- The emitted module has exactly one function, one memory, and no other
definitions: its section contents are exactly what the released profile
admits. -/
theorem compile_sections {s t : Sig} (c : CheckedPlan s t) :
    (compile c).types.length = 1 ∧ (compile c).imports = [] ∧
      (compile c).funcs.length = 1 ∧ (compile c).tables = [] ∧
      (compile c).mems.length = 1 ∧ (compile c).tags = [] ∧
      (compile c).globals = [] ∧ (compile c).exports.length = 2 ∧
      (compile c).start = none ∧ (compile c).elems = [] ∧
      (compile c).datas = [] :=
  ⟨rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl⟩

/-! ## Validity of the emitted module -/

theorem all_replicate_isI32 (n : Nat) :
    (List.replicate n (Wasm.ValType.num .i32)).all Wasm.Module.isI32 = true := by
  induction n with
  | zero => rfl
  | succ k ih => simpa [List.replicate, Wasm.Module.isI32] using ih

theorem funcTypeAt_zero (e : CompileEnv) (b : List Wasm.Instr) :
    Wasm.Module.funcTypeAt (moduleOf e b) 0 = some Wasm.gemmFuncType := rfl

theorem funcCtx_numLocals (e : CompileEnv) (b : List Wasm.Instr) :
    (Wasm.Module.funcCtx (moduleOf e b) Wasm.gemmFuncType
      (moduleOf e b).funcs[0]!).numLocals = e.numLocals := by
  simp [Wasm.Module.funcCtx, moduleOf, Wasm.gemmFuncType, CompileEnv.numLocals]

/-- The pinned export list carries the required memory export. -/
theorem exportsMemory_gemmExports :
    ([{ name := Wasm.gemmExportName, desc := .func 0 },
      { name := Wasm.memoryExportName, desc := .mem 0 }] : List Wasm.Export).any
      (fun x => x.name == Wasm.memoryExportName && x.desc == Wasm.ExportDesc.mem 0) =
      true := by simp

/-- The pinned export list names function 0 as `gemm`. -/
theorem gemmIndex_gemmExports (e : CompileEnv) (b : List Wasm.Instr) :
    Wasm.Module.gemmIndex? (moduleOf e b) = some 0 := by
  simp [Wasm.Module.gemmIndex?, CompiledModule.exports_eq]

/-- The emitted module passes release validation whenever its single function
body is accepted at the ABI arity. -/
theorem validate_moduleOf (e : CompileEnv) (b : List Wasm.Instr)
    (hbody : Wasm.checkExpr
      { numLocals := e.numLocals, globals := [], numTags := 0, labels := [] } 0
      (Wasm.Expr.ofList b) = some 1) :
    Wasm.validate (moduleOf e b) = true := by
  have hfunc : Wasm.Module.checkFunc (moduleOf e b)
      { type := 0
        locals := List.replicate e.declaredLocals (Wasm.ValType.num .i32)
        body := Wasm.Expr.ofList b } = true := by
    have hctx :
        Wasm.Module.funcCtx (moduleOf e b) Wasm.gemmFuncType
          { type := 0
            locals := List.replicate e.declaredLocals (Wasm.ValType.num .i32)
            body := Wasm.Expr.ofList b } =
        { numLocals := e.numLocals, globals := [], numTags := 0, labels := [] } := by
      simp [Wasm.Module.funcCtx, moduleOf, Wasm.gemmFuncType, CompileEnv.numLocals]
    simp only [Wasm.Module.checkFunc, funcTypeAt_zero, hctx, hbody]
    simp [Wasm.gemmFuncType, Wasm.Module.isI32, all_replicate_isI32]
  have hclosed : Wasm.Module.checkClosed (moduleOf e b) = true := rfl
  have hmems : Wasm.Module.checkMems (moduleOf e b) = true := by
    have h1 : decide (e.pages ≤ Wasm.Memory.hardMaxPages) = true :=
      decide_eq_true e.pages_le_hard
    simp [Wasm.Module.checkMems, moduleOf, h1]
  have hglobals : (moduleOf e b).globals.all Wasm.Module.checkGlobal = true := rfl
  have htags : (moduleOf e b).tags.all Wasm.Module.checkTag = true := rfl
  have hfuncs : (moduleOf e b).funcs.all (Wasm.Module.checkFunc (moduleOf e b)) = true := by
    simpa [moduleOf] using hfunc
  have hexpmem : Wasm.Module.exportsMemory (moduleOf e b) = true :=
    exportsMemory_gemmExports
  have hgemm : Wasm.Module.checkGemmExport (moduleOf e b) = true := by
    have hidx : Wasm.Module.gemmIndex? (moduleOf e b) = some 0 := gemmIndex_gemmExports e b
    simp only [Wasm.Module.checkGemmExport, hidx, CompiledModule.funcs_eq,
      List.getElem?_cons_zero, funcTypeAt_zero]
    rfl
  have hstart : Wasm.Module.checkStart (moduleOf e b) = true := rfl
  unfold Wasm.validate
  rw [hclosed, hmems, hglobals, htags, hfuncs, hexpmem, hgemm, hstart]
  rfl

/-- **SPEC §11.4 / §7.3.**  The compiler emits only modules that pass release
validation: the artifact is inside the released portable profile. -/
theorem compile_validates {s t : Sig} (c : CheckedPlan s t) :
    Wasm.validate (compile c) = true := by
  refine validate_moduleOf (envOf s c.plan) (bodyCode (envOf s c.plan) s.scratch c.plan) ?_
  have hloc : 2 + (envOf s c.plan).regs + 0 + c.plan.depth + 1 ≤
      ({ numLocals := (envOf s c.plan).numLocals, globals := [], numTags := 0,
         labels := [] } : Wasm.Ctx).numLocals := by
    simp only [envOf, CompileEnv.numLocals, CompileEnv.declaredLocals]
    omega
  have hcode := checkList_code c.typed (envOf s c.plan) 0 s.scratch
    { numLocals := (envOf s c.plan).numLocals, globals := [], numTags := 0, labels := [] }
    0 (by simp [envOf]) hloc
  exact checkList_app2 hcode (checkList_loadAt _ 0 _)

/-- The emitted module is declaratively valid, in the sense of
`Wasm.DeclarativelyValid` (SPEC §7.3). -/
theorem compile_declarativelyValid {s t : Sig} (c : CheckedPlan s t) :
    Wasm.DeclarativelyValid (compile c) :=
  (Wasm.validate_bool_iff _).mp (compile_validates c)

/-! ## The emitted module's static resource bound

`compile_resources` bounds the *size of the emitted code* — the number of
instructions of the module, counting the bodies of structured control
instructions — by a fixed multiple of the plan's own declared static cost
coordinates of SPEC §9.1 (`Charges.moduleBytes` and `Charges.dataBytes`).

Scope, stated explicitly: this is a bound on the emitted artifact, not on its
execution.  It says nothing about the number of Wasm reduction steps the module
takes, which is `Plan.stepBound`'s subject and would need the refinement
theorem this file does not prove. -/

mutual

/-- The number of instructions of a Core instruction, counting the bodies of
structured control instructions. -/
def instrSize : Wasm.Instr → Nat
  | .block _ b => 1 + exprSize b
  | .loop _ b => 1 + exprSize b
  | .ifThenElse _ a b => 1 + exprSize a + exprSize b
  | .tryTable _ _ b => 1 + exprSize b
  | _ => 1

/-- The number of instructions of a Core instruction sequence. -/
def exprSize : Wasm.Expr → Nat
  | .nil => 0
  | .cons i e => instrSize i + exprSize e

end

/-- The number of instructions of an instruction list. -/
def listSize : List Wasm.Instr → Nat
  | [] => 0
  | i :: l => instrSize i + listSize l

@[simp] theorem listSize_append (a b : List Wasm.Instr) :
    listSize (a ++ b) = listSize a + listSize b := by
  induction a with
  | nil => simp [listSize]
  | cons i l ih => simp [listSize, ih]; omega

@[simp] theorem exprSize_ofList (l : List Wasm.Instr) :
    exprSize (Wasm.Expr.ofList l) = listSize l := by
  induction l with
  | nil => rfl
  | cons i l ih => simp [Wasm.Expr.ofList, exprSize, listSize, ih]

@[simp] theorem instrSize_i32Const (v : Int) : instrSize (.i32Const v) = 1 := rfl
@[simp] theorem instrSize_localGet (i : Nat) : instrSize (.localGet i) = 1 := rfl
@[simp] theorem instrSize_localSet (i : Nat) : instrSize (.localSet i) = 1 := rfl
@[simp] theorem instrSize_br (i : Nat) : instrSize (.br i) = 1 := rfl
@[simp] theorem instrSize_brIf (i : Nat) : instrSize (.brIf i) = 1 := rfl
@[simp] theorem instrSize_unreachable : instrSize .unreachable = 1 := rfl
@[simp] theorem instrSize_load (t : Wasm.NumType) (nw : Option (Wasm.MemWidth × Wasm.SignExt))
    (a : Wasm.MemArg) : instrSize (.load t nw a) = 1 := rfl
@[simp] theorem instrSize_store (t : Wasm.NumType) (nw : Option Wasm.MemWidth)
    (a : Wasm.MemArg) : instrSize (.store t nw a) = 1 := rfl
@[simp] theorem instrSize_iBinOp (w : Wasm.IntWidth) (o : Wasm.IBinOp) :
    instrSize (.iBinOp w o) = 1 := rfl
@[simp] theorem instrSize_iRelOp (w : Wasm.IntWidth) (o : Wasm.IRelOp) :
    instrSize (.iRelOp w o) = 1 := rfl

@[simp] theorem instrSize_ifE (a b : List Wasm.Instr) :
    instrSize (ifE a b) = 1 + listSize a + listSize b := by simp [ifE, instrSize]

@[simp] theorem instrSize_blockE (b : List Wasm.Instr) :
    instrSize (blockE b) = 1 + listSize b := by simp [blockE, instrSize]

@[simp] theorem instrSize_loopE (b : List Wasm.Instr) :
    instrSize (loopE b) = 1 + listSize b := by simp [loopE, instrSize]

theorem listSize_countLoop (c n : Nat) (body : List Wasm.Instr) :
    listSize (countLoop c n body) = 13 + listSize body := by
  simp [countLoop, listSize, constI, geUI, addI]
  omega

theorem listSize_countLoopVar (c ext : Nat) (body : List Wasm.Instr) :
    listSize (countLoopVar c ext body) = 13 + listSize body := by
  simp [countLoopVar, listSize, constI, geUI, addI]
  omega

theorem listSize_dispatchOn (t k : Nat) (a b : List Wasm.Instr) :
    listSize (dispatchOn t k a b) = 4 + listSize a + listSize b := by
  simp [dispatchOn, listSize, constI, eqI]
  omega

theorem listSize_headerOkCode (e : CompileEnv) : listSize (headerOkCode e) = 39 := by
  simp [headerOkCode, cellCode, u16Code, loadAt, eqConst, listSize, constI, loadW, eqI,
    andI, mulI, addI]

theorem listSize_tagsKnownCode (e : CompileEnv) : listSize (tagsKnownCode e) = 14 := by
  simp [tagsKnownCode, cellCode, loadAt, listSize, constI, loadW, ltUI, andI]

theorem listSize_keyCode (e : CompileEnv) : listSize (keyCode e) = 12 := by
  simp [keyCode, cellCode, loadAt, listSize, constI, loadW, mulI, addI]

theorem listSize_keyChain (e : CompileEnv) :
    ∀ ks : List Nat, listSize (keyChain e ks) = 15 * ks.length := by
  intro ks
  induction ks with
  | nil => rfl
  | cons k ks ih =>
    rw [keyChain, listSize_append, listSize_append, ih, listSize_keyCode]
    simp [listSize, constI, eqI, orI]
    omega

theorem listSize_compatCode (e : CompileEnv) : listSize (compatCode e) = 406 := by
  rw [compatCode, listSize_append, listSize_keyChain, compatibleKeys_length]
  simp [listSize, constI]

theorem listSize_widthChain (e : CompileEnv) :
    ∀ ks : List ScalarKind, listSize (widthChain e ks) = 7 * ks.length := by
  intro ks
  induction ks with
  | nil => rfl
  | cons k ks ih =>
    rw [widthChain, listSize_append, listSize_append, ih]
    simp [listSize, cellCode, loadAt, constI, eqI, mulI, addI, loadW]
    omega

theorem listSize_widthCode (e : CompileEnv) : listSize (widthCode e) = 92 := by
  rw [widthCode, listSize_append, listSize_widthChain]
  simp [listSize, constI, ScalarKind.all]

theorem listSize_classifyCode (e : CompileEnv) (t : Nat) :
    listSize (classifyCode e t) ≤ 470 := by
  unfold classifyCode
  split
  · simp [listSize, constI]
  · rw [listSize_append, listSize_append, listSize_headerOkCode]
    simp only [listSize, instrSize_ifE, listSize_append, listSize_tagsKnownCode,
      listSize_compatCode, instrSize_i32Const, instrSize_localSet, constI]
    omega

theorem listSize_layoutTestCode (e : CompileEnv) (t field cls : Nat) :
    listSize (layoutTestCode e t field cls) = 103 := by
  rw [layoutTestCode, listSize_append, listSize_append, listSize_widthCode]
  simp [listSize, u16Code, cellCode, loadAt, constI, eqI, mulI, addI, loadW]

theorem listSize_layoutCode (e : CompileEnv) (t : Nat) :
    listSize (layoutCode e t) = 208 := by
  rw [layoutCode, listSize_append, listSize_append, listSize_layoutTestCode,
    listSize_layoutTestCode]
  simp [listSize, constI]

theorem listSize_condCode (e : CompileEnv) (scr : Nat) (co : Cond) :
    listSize (condCode e scr co) ≤ 4 := by
  cases co <;>
    simp [condCode, listSize, loadAt, loadW, eqConst, constI, eqI, ltUI]

theorem listSize_tableStores (base : Nat) :
    ∀ (vs : List Nat) (j : Nat), listSize (tableStores base j vs) = 3 * vs.length := by
  intro vs
  induction vs with
  | nil => intro j; rfl
  | cons v vs ih =>
    intro j
    rw [tableStores, listSize_append, ih]
    simp [listSize, constI, storeW]
    omega

/-! ### Charge projections -/

@[simp] theorem charges_add_moduleBytes (a b : Charges) :
    (Charges.add a b).moduleBytes = a.moduleBytes + b.moduleBytes := rfl

@[simp] theorem charges_add_dataBytes (a b : Charges) :
    (Charges.add a b).dataBytes = a.dataBytes + b.dataBytes := rfl

@[simp] theorem charges_node_moduleBytes (n : Nat) : (Charges.node n).moduleBytes = n := rfl

@[simp] theorem charges_node_dataBytes (n : Nat) : (Charges.node n).dataBytes = 0 := rfl

@[simp] theorem charges_iterate_dataBytes (c : Charges) (n : Nat) :
    (c.iterate n).dataBytes = c.dataBytes := rfl

/-- The static code budget of a plan: a fixed multiple of the plan's own
declared static cost coordinates. -/
def Plan.codeBudget (p : Plan) : Nat :=
  1024 * (p.charges.moduleBytes + p.charges.dataBytes)

/-- **The emitted code fits the plan's static budget.** -/
theorem listSize_code (e : CompileEnv) :
    ∀ (d scr : Nat) (p : Plan), listSize (code e d scr p) ≤ p.codeBudget := by
  intro d scr p
  induction p generalizing d scr with
  | nop => simp [code, listSize, Plan.codeBudget, Plan.charges, Charges.zero]
  | seq a b iha ihb =>
    have h1 := iha d scr
    have h2 := ihb d scr
    simp only [code, listSize_append, Plan.codeBudget, Plan.charges,
      charges_add_moduleBytes, charges_add_dataBytes] at *
    omega
  | classifyRaw v i u x ihv ihi ihu ihx =>
    have h1 := ihv d scr
    have h2 := ihi d scr
    have h3 := ihu d scr
    have h4 := ihx d scr
    have hc := listSize_classifyCode e (e.tempLocal d)
    simp only [code, listSize_append, listSize_dispatchOn, Plan.codeBudget, Plan.charges,
      charges_add_moduleBytes, charges_add_dataBytes, charges_node_moduleBytes,
      charges_node_dataBytes] at *
    omega
  | dispatchLayout r c g ihr ihc ihg =>
    have h1 := ihr d scr
    have h2 := ihc d scr
    have h3 := ihg d scr
    have hl := listSize_layoutCode e (e.tempLocal d)
    simp only [code, listSize_append, listSize_dispatchOn, Plan.codeBudget, Plan.charges,
      charges_add_moduleBytes, charges_add_dataBytes, charges_node_moduleBytes,
      charges_node_dataBytes] at *
    omega
  | branch co t f iht ihf =>
    have h1 := iht d scr
    have h2 := ihf d scr
    have hc := listSize_condCode e scr co
    simp only [code, listSize_append, listSize, instrSize_ifE, Plan.codeBudget,
      Plan.charges, charges_add_moduleBytes, charges_add_dataBytes,
      charges_node_moduleBytes, charges_node_dataBytes] at *
    omega
  | pack src dst map w =>
    simp only [code, listSize_countLoop, Plan.codeBudget, Plan.charges]
    simp [listSize, constI, mulI, addI, loadW, storeW]
  | unpack src dst map w =>
    simp only [code, listSize_countLoop, Plan.codeBudget, Plan.charges]
    simp [listSize, constI, mulI, addI, loadW, storeW]
  | storeReg dst map w src =>
    simp [code, listSize, constI, mulI, addI, storeW, Plan.codeBudget, Plan.charges,
      Charges.node, Charges.zero]
  | loadReg dst src map w =>
    simp [code, listSize, constI, mulI, addI, loadW, Plan.codeBudget, Plan.charges,
      Charges.node, Charges.zero]
  | loopReg ir er mp body ih =>
    have h1 := ih (d + 1) scr
    simp only [code, listSize_countLoopVar, listSize_append, Plan.codeBudget, Plan.charges,
      charges_add_moduleBytes, charges_add_dataBytes, charges_node_moduleBytes,
      charges_node_dataBytes, Charges.iterate_moduleBytes,
      charges_iterate_dataBytes, instrSize_localGet, instrSize_localSet] at *
    simp only [listSize, instrSize_localGet, instrSize_localSet] at *
    omega
  | loopNest axis body ih =>
    have h1 := ih (d + 1) scr
    simp only [code, listSize_countLoop, listSize_append, Plan.codeBudget, Plan.charges,
      charges_add_moduleBytes, charges_add_dataBytes, charges_node_moduleBytes,
      charges_node_dataBytes, Charges.iterate_moduleBytes,
      charges_iterate_dataBytes, instrSize_localGet, instrSize_localSet] at *
    simp only [listSize, instrSize_localGet, instrSize_localSet] at *
    omega
  | tiled o ti ex body ih =>
    have h1 := ih (d + 1) scr
    simp only [code, listSize_countLoop, listSize_append, Plan.codeBudget, Plan.charges,
      charges_add_moduleBytes, charges_add_dataBytes, charges_node_moduleBytes,
      charges_node_dataBytes, Charges.iterate_moduleBytes,
      charges_iterate_dataBytes] at *
    by_cases hr : 0 < e.regs
    · simp only [hr, if_pos, listSize, instrSize_localGet, instrSize_localSet] at *
      omega
    · simp only [hr, listSize, if_false, if_neg, List.nil_append] at *
      omega
  | reduce c acc lhs rhs =>
    simp only [code, Plan.codeBudget, Plan.charges]
    split
    · rw [listSize_countLoop]
      simp [listSize, constI, mulI, addI, loadW]
    · simp [listSize]
  | allocScratch bytes body ih =>
    have h1 := ih d (scr + bytes)
    simp only [code, Plan.codeBudget, Plan.charges, charges_add_moduleBytes,
      charges_add_dataBytes, charges_node_moduleBytes, charges_node_dataBytes] at *
    omega
  | setReg dst v => simp [code, listSize, constI, Plan.codeBudget, Plan.charges]
  | scalarOp op dst a b =>
    cases op <;>
      simp [code, listSize, constI, addI, mulI, subI, geUI, gtUI, ltUI,
        Plan.codeBudget, Plan.charges]
  | vectorOp op lanes dst a b =>
    simp [code, listSize, Plan.codeBudget, Plan.charges]
  | emitTable index data =>
    simp only [code, listSize_tableStores, Plan.codeBudget, Plan.charges]
    simp
    omega
  | tableLoad table index dst =>
    simp [code, listSize, loadAt, constI, loadW, Plan.codeBudget, Plan.charges]
  | setStatus st => simp [code, listSize, constI, storeW, Plan.codeBudget, Plan.charges]
  | buildOutput src =>
    simp [code, listSize, constI, storeW, Plan.codeBudget, Plan.charges]
  | opaqueProcess spec body ih =>
    have h1 := ih d scr
    simp only [code, Plan.codeBudget, Plan.charges, charges_add_moduleBytes,
      charges_add_dataBytes, charges_node_moduleBytes, charges_node_dataBytes] at *
    omega

/-! ## The refusal points are exactly the out-of-subset nodes

The translation emits `unreachable` — a trap — for the two constructs the
released profile cannot express (`vectorOp`, and `reduce` under a contract that
is not modular with a `u32` accumulator).  `code_no_unreachable` proves the
converse claim that makes this a real restriction rather than a slogan: the
translation of an in-subset plan contains **no** `unreachable` anywhere,
including inside the bodies of the structured control instructions it emits.
So a compiled artifact traps on entry to a refused node and nowhere else that
the compiler put it. -/

mutual

/-- Whether an instruction contains `unreachable`, at any nesting depth. -/
def instrHasUnreachable : Wasm.Instr → Bool
  | .unreachable => true
  | .block _ b => exprHasUnreachable b
  | .loop _ b => exprHasUnreachable b
  | .ifThenElse _ a b => exprHasUnreachable a || exprHasUnreachable b
  | .tryTable _ _ b => exprHasUnreachable b
  | _ => false

/-- Whether an instruction sequence contains `unreachable`, at any nesting
depth. -/
def exprHasUnreachable : Wasm.Expr → Bool
  | .nil => false
  | .cons i e => instrHasUnreachable i || exprHasUnreachable e

end

/-- Whether an instruction list contains `unreachable`, at any nesting depth. -/
def listHasUnreachable : List Wasm.Instr → Bool
  | [] => false
  | i :: l => instrHasUnreachable i || listHasUnreachable l

@[simp] theorem listHasUnreachable_append (a b : List Wasm.Instr) :
    listHasUnreachable (a ++ b) = (listHasUnreachable a || listHasUnreachable b) := by
  induction a with
  | nil => simp [listHasUnreachable]
  | cons i l ih => simp [listHasUnreachable, ih, Bool.or_assoc]

@[simp] theorem exprHasUnreachable_ofList (l : List Wasm.Instr) :
    exprHasUnreachable (Wasm.Expr.ofList l) = listHasUnreachable l := by
  induction l with
  | nil => rfl
  | cons i l ih =>
    simp [Wasm.Expr.ofList, exprHasUnreachable, listHasUnreachable, ih]

@[simp] theorem instrHasUnreachable_unreachable :
    instrHasUnreachable .unreachable = true := rfl
@[simp] theorem instrHasUnreachable_i32Const (v : Int) :
    instrHasUnreachable (.i32Const v) = false := rfl
@[simp] theorem instrHasUnreachable_localGet (i : Nat) :
    instrHasUnreachable (.localGet i) = false := rfl
@[simp] theorem instrHasUnreachable_localSet (i : Nat) :
    instrHasUnreachable (.localSet i) = false := rfl
@[simp] theorem instrHasUnreachable_br (i : Nat) :
    instrHasUnreachable (.br i) = false := rfl
@[simp] theorem instrHasUnreachable_brIf (i : Nat) :
    instrHasUnreachable (.brIf i) = false := rfl
@[simp] theorem instrHasUnreachable_load (t : Wasm.NumType)
    (nw : Option (Wasm.MemWidth × Wasm.SignExt)) (a : Wasm.MemArg) :
    instrHasUnreachable (.load t nw a) = false := rfl
@[simp] theorem instrHasUnreachable_store (t : Wasm.NumType)
    (nw : Option Wasm.MemWidth) (a : Wasm.MemArg) :
    instrHasUnreachable (.store t nw a) = false := rfl
@[simp] theorem instrHasUnreachable_iBinOp (w : Wasm.IntWidth) (o : Wasm.IBinOp) :
    instrHasUnreachable (.iBinOp w o) = false := rfl
@[simp] theorem instrHasUnreachable_iRelOp (w : Wasm.IntWidth) (o : Wasm.IRelOp) :
    instrHasUnreachable (.iRelOp w o) = false := rfl

@[simp] theorem instrHasUnreachable_ifE (a b : List Wasm.Instr) :
    instrHasUnreachable (ifE a b) =
      (listHasUnreachable a || listHasUnreachable b) := by
  simp [ifE, instrHasUnreachable]

@[simp] theorem instrHasUnreachable_blockE (b : List Wasm.Instr) :
    instrHasUnreachable (blockE b) = listHasUnreachable b := by
  simp [blockE, instrHasUnreachable]

@[simp] theorem instrHasUnreachable_loopE (b : List Wasm.Instr) :
    instrHasUnreachable (loopE b) = listHasUnreachable b := by
  simp [loopE, instrHasUnreachable]

theorem listHasUnreachable_countLoop (c n : Nat) (body : List Wasm.Instr) :
    listHasUnreachable (countLoop c n body) = listHasUnreachable body := by
  simp [countLoop, listHasUnreachable, constI, geUI, addI]

theorem listHasUnreachable_countLoopVar (c ext : Nat) (body : List Wasm.Instr) :
    listHasUnreachable (countLoopVar c ext body) = listHasUnreachable body := by
  simp [countLoopVar, listHasUnreachable, constI, geUI, addI]

theorem listHasUnreachable_dispatchOn (t k : Nat) (a b : List Wasm.Instr) :
    listHasUnreachable (dispatchOn t k a b) =
      (listHasUnreachable a || listHasUnreachable b) := by
  simp [dispatchOn, listHasUnreachable, constI, eqI]

theorem listHasUnreachable_cellCode (e : CompileEnv) (i : Nat) :
    listHasUnreachable (cellCode e i) = false := by
  simp [cellCode, loadAt, listHasUnreachable, constI, loadW]

theorem listHasUnreachable_u16Code (e : CompileEnv) (i : Nat) :
    listHasUnreachable (u16Code e i) = false := by
  simp [u16Code, cellCode, loadAt, listHasUnreachable, constI, loadW, mulI, addI]

theorem listHasUnreachable_headerOkCode (e : CompileEnv) :
    listHasUnreachable (headerOkCode e) = false := by
  simp [headerOkCode, cellCode, u16Code, loadAt, eqConst, listHasUnreachable, constI,
    loadW, eqI, andI, mulI, addI]

theorem listHasUnreachable_tagsKnownCode (e : CompileEnv) :
    listHasUnreachable (tagsKnownCode e) = false := by
  simp [tagsKnownCode, cellCode, loadAt, listHasUnreachable, constI, loadW, ltUI, andI]

theorem listHasUnreachable_keyCode (e : CompileEnv) :
    listHasUnreachable (keyCode e) = false := by
  simp [keyCode, cellCode, loadAt, listHasUnreachable, constI, loadW, mulI, addI]

theorem listHasUnreachable_keyChain (e : CompileEnv) :
    ∀ ks : List Nat, listHasUnreachable (keyChain e ks) = false := by
  intro ks
  induction ks with
  | nil => rfl
  | cons k ks ih =>
    simp [keyChain, listHasUnreachable_keyCode, ih, listHasUnreachable, constI, eqI, orI]

theorem listHasUnreachable_compatCode (e : CompileEnv) :
    listHasUnreachable (compatCode e) = false := by
  simp [compatCode, listHasUnreachable, constI, listHasUnreachable_keyChain]

theorem listHasUnreachable_widthChain (e : CompileEnv) :
    ∀ ks : List ScalarKind, listHasUnreachable (widthChain e ks) = false := by
  intro ks
  induction ks with
  | nil => rfl
  | cons k ks ih =>
    simp [widthChain, listHasUnreachable_cellCode, ih, listHasUnreachable, constI, eqI,
      mulI, addI]

theorem listHasUnreachable_widthCode (e : CompileEnv) :
    listHasUnreachable (widthCode e) = false := by
  simp [widthCode, listHasUnreachable, constI, listHasUnreachable_widthChain]

theorem listHasUnreachable_classifyCode (e : CompileEnv) (t : Nat) :
    listHasUnreachable (classifyCode e t) = false := by
  unfold classifyCode
  split
  · simp [listHasUnreachable, constI]
  · simp [listHasUnreachable, constI, listHasUnreachable_headerOkCode,
      listHasUnreachable_tagsKnownCode, listHasUnreachable_compatCode]

theorem listHasUnreachable_layoutCode (e : CompileEnv) (t : Nat) :
    listHasUnreachable (layoutCode e t) = false := by
  simp [layoutCode, layoutTestCode, listHasUnreachable, constI, eqI,
    listHasUnreachable_u16Code, listHasUnreachable_widthCode]

theorem listHasUnreachable_condCode (e : CompileEnv) (scr : Nat) (co : Cond) :
    listHasUnreachable (condCode e scr co) = false := by
  cases co <;>
    simp [condCode, listHasUnreachable, loadAt, loadW, eqConst, constI, eqI, ltUI]

theorem listHasUnreachable_tableStores (base : Nat) :
    ∀ (vs : List Nat) (j : Nat),
      listHasUnreachable (tableStores base j vs) = false := by
  intro vs
  induction vs with
  | nil => intro j; rfl
  | cons v vs ih =>
    intro j
    simp [tableStores, listHasUnreachable, constI, storeW, ih]

/-- **The refusal points are exactly the out-of-subset nodes.**  The translation
of a plan inside the released subset emits no `unreachable` at any nesting
depth. -/
theorem code_no_unreachable (e : CompileEnv) :
    ∀ (d scr : Nat) (p : Plan), p.inReleasedSubset = true →
      listHasUnreachable (code e d scr p) = false := by
  intro d scr p
  induction p generalizing d scr with
  | nop => intro _; rfl
  | seq a b iha ihb =>
    intro h
    simp only [Plan.inReleasedSubset, Bool.and_eq_true] at h
    simp [code, iha d scr h.1, ihb d scr h.2]
  | classifyRaw v i u x ihv ihi ihu ihx =>
    intro h
    simp only [Plan.inReleasedSubset, Bool.and_eq_true] at h
    simp [code, listHasUnreachable_dispatchOn, listHasUnreachable_classifyCode,
      ihv d scr h.1.1.1, ihi d scr h.1.1.2, ihu d scr h.1.2, ihx d scr h.2]
  | dispatchLayout r c g ihr ihc ihg =>
    intro h
    simp only [Plan.inReleasedSubset, Bool.and_eq_true] at h
    simp [code, listHasUnreachable_dispatchOn, listHasUnreachable_layoutCode,
      ihr d scr h.1.1, ihc d scr h.1.2, ihg d scr h.2]
  | branch co a b iha ihb =>
    intro h
    simp only [Plan.inReleasedSubset, Bool.and_eq_true] at h
    simp [code, listHasUnreachable, listHasUnreachable_condCode,
      iha d scr h.1, ihb d scr h.2]
  | pack src dst map w =>
    intro _
    simp [code, listHasUnreachable_countLoop, listHasUnreachable, constI, mulI, addI,
      loadW, storeW]
  | unpack src dst map w =>
    intro _
    simp [code, listHasUnreachable_countLoop, listHasUnreachable, constI, mulI, addI,
      loadW, storeW]
  | storeReg dst map w src =>
    intro _
    simp [code, listHasUnreachable, constI, mulI, addI, storeW]
  | loadReg dst src map w =>
    intro _
    simp [code, listHasUnreachable, constI, mulI, addI, loadW]
  | loopReg ir er mp body ih =>
    intro h
    simp [code, listHasUnreachable_countLoopVar, listHasUnreachable, ih (d + 1) scr h]
  | loopNest axis body ih =>
    intro h
    simp [code, listHasUnreachable_countLoop, listHasUnreachable, ih (d + 1) scr h]
  | tiled o ti ex body ih =>
    intro h
    simp only [Plan.inReleasedSubset] at h
    simp only [code, listHasUnreachable_countLoop, listHasUnreachable_append,
      ih (d + 1) scr h, Bool.or_false]
    by_cases hr : 0 < e.regs <;> simp [hr, listHasUnreachable]
  | reduce c acc lhs rhs =>
    intro h
    simp only [Plan.inReleasedSubset] at h
    simp only [code, if_pos h]
    simp [listHasUnreachable_countLoop, listHasUnreachable, constI, mulI, addI, loadW]
  | allocScratch bytes body ih => intro h; exact ih d (scr + bytes) h
  | setReg dst v => intro _; simp [code, listHasUnreachable, constI]
  | scalarOp op dst a b =>
    intro _
    cases op <;>
      simp [code, listHasUnreachable, constI, addI, mulI, subI, geUI, gtUI, ltUI]
  | vectorOp op lanes dst a b =>
    intro h
    simp only [Plan.inReleasedSubset] at h
    exact absurd h (by simp)
  | emitTable index data =>
    intro _; simp [code, listHasUnreachable_tableStores]
  | tableLoad table index dst =>
    intro _; simp [code, listHasUnreachable, loadAt, constI, loadW]
  | setStatus st => intro _; simp [code, listHasUnreachable, constI, storeW]
  | buildOutput src => intro _; simp [code, listHasUnreachable, constI, storeW]
  | opaqueProcess spec body ih => intro h; exact ih d scr h

/-- The body emitted for a plan inside the released subset contains no
`unreachable`. -/
theorem bodyCode_no_unreachable (e : CompileEnv) (scr : Nat) (p : Plan)
    (h : p.inReleasedSubset = true) :
    listHasUnreachable (bodyCode e scr p) = false := by
  simp [bodyCode, code_no_unreachable e 0 scr p h, loadAt, listHasUnreachable,
    constI, loadW]

/-- The number of instructions of every function body of a module. -/
def moduleCodeSize (m : Wasm.Module) : Nat :=
  m.funcs.foldl (fun acc f => acc + exprSize f.body) 0

/-- **SPEC §11.4, `compile_resources`.**  The emitted module's code fits the
plan's own static budget: the emitted instruction count is at most a fixed
multiple of the plan's declared static cost coordinates, plus the two-instruction
ABI epilogue. -/
theorem compile_resources {s t : Sig} (c : CheckedPlan s t) :
    moduleCodeSize (compile c) ≤ c.plan.codeBudget + 2 := by
  have h := listSize_code (envOf s c.plan) 0 s.scratch c.plan
  show 0 + exprSize (Wasm.Expr.ofList (bodyCode (envOf s c.plan) s.scratch c.plan)) ≤ _
  rw [exprSize_ofList, bodyCode, listSize_append]
  simp only [loadAt, listSize, constI, loadW, instrSize]
  omega

end WasmGemmGnaf.GNAF
