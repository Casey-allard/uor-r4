/-
  Wasm/Validate.lean --- the declarative typing judgment and its decidable
  checker.

  Normative source: SPEC.md section 7.1 ("`Validate` owns all context and
  declaration judgments"), section 7.2 ("disabled forms decode when
  grammatically valid and then fail profile validation") and section 7.3, which
  requires

      theorem validate_bool_iff (module : Wasm.Module) :
        Wasm.validate module = true ↔ Wasm.DeclarativelyValid module

  `Wasm/Syntax.lean` expresses the whole pinned Core 3.0 grammar, so every
  Core form can be *written*.  Validation is what selects the released
  executable subset: an instruction outside the declared subset has no typing
  rule, hence no derivation, hence `validate` rejects it.  That is exactly the
  behaviour SPEC section 7.2 prescribes for out-of-profile forms, and it is the
  reason the judgment below is stated positively (rules only for admitted
  forms) rather than by a list of prohibitions.

  The declared executable subset is: `unreachable`, `nop`, `i32.const`, `drop`,
  the wrapping and unsigned `i32` binary operators, `i32.eqz`, the `i32`
  relational operators, the local and global accessors, full-width `i32`
  loads/stores on memory 0, `memory.size`/`memory.grow` on memory 0, the
  structured control forms `block`, `loop` and `if` with empty block type,
  `br`/`br_if`, and `throw`.  Every other Core form --- SIMD, GC, reference,
  table, bulk-memory, tail-call, `try_table`, 64-bit and floating point --- is
  grammatically expressible and is rejected here.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Syntax
import WasmGemmGnaf.Wasm.Numeric
import WasmGemmGnaf.Wasm.Memory

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Validation contexts -/

/-- A validation context: the number of locals in scope, the declared global
types, the number of declared tags, and the stack heights required by the
labels currently in scope (innermost first). -/
structure Ctx where
  /-- Number of locals of the enclosing function. -/
  numLocals : Nat
  /-- The module's global types. -/
  globals : List GlobalType
  /-- The number of declared tags. -/
  numTags : Nat
  /-- Required operand-stack height at each label in scope, innermost first. -/
  labels : List Nat
  deriving DecidableEq, Repr, Inhabited

/-- The `i32` binary operators of the declared subset: the wrapping arithmetic,
the unsigned division/remainder (which trap on a zero divisor), the bitwise
operators and the unsigned shifts. -/
def SubsetIBinOp : IBinOp → Bool
  | .add | .sub | .mul | .divU | .remU => true
  | .and | .or | .xor | .shl | .shrU => true
  | _ => false

/-! ## The declarative typing judgment

`InstrTyping C h i h'` says that in context `C` the instruction `i` takes an
operand stack of height `h` to one of height `h'`.  Because every value of the
declared subset is an `i32`, a height is a complete stack type. -/

mutual

/-- The declarative typing judgment of a single instruction. -/
inductive InstrTyping : Ctx → Nat → Instr → Nat → Prop
  | unreachable {C h} : InstrTyping C h .unreachable h
  | nop {C h} : InstrTyping C h .nop h
  | i32Const {C h v} : InstrTyping C h (.i32Const v) (h + 1)
  | drop {C h} : InstrTyping C (h + 1) .drop h
  | iBinOp {C h op} (hop : SubsetIBinOp op = true) :
      InstrTyping C (h + 2) (.iBinOp .i32 op) (h + 1)
  | iTestOp {C h} : InstrTyping C (h + 1) (.iTestOp .i32 .eqz) (h + 1)
  | iRelOp {C h op} : InstrTyping C (h + 2) (.iRelOp .i32 op) (h + 1)
  | localGet {C h i} (hi : i < C.numLocals) : InstrTyping C h (.localGet i) (h + 1)
  | localSet {C h i} (hi : i < C.numLocals) : InstrTyping C (h + 1) (.localSet i) h
  | localTee {C h i} (hi : i < C.numLocals) :
      InstrTyping C (h + 1) (.localTee i) (h + 1)
  | globalGet {C h i g} (hg : C.globals[i]? = some g)
      (ht : g.valType = .num .i32) : InstrTyping C h (.globalGet i) (h + 1)
  | globalSet {C h i g} (hg : C.globals[i]? = some g)
      (ht : g.valType = .num .i32) (hm : g.mutable = true) :
      InstrTyping C (h + 1) (.globalSet i) h
  | load {C h arg} (hm : arg.memory = 0) :
      InstrTyping C (h + 1) (.load .i32 none arg) (h + 1)
  | store {C h arg} (hm : arg.memory = 0) :
      InstrTyping C (h + 2) (.store .i32 none arg) h
  | memorySize {C h} : InstrTyping C h (.memorySize 0) (h + 1)
  | memoryGrow {C h} : InstrTyping C (h + 1) (.memoryGrow 0) (h + 1)
  | block {C h body}
      (hb : ExprTyping { C with labels := h :: C.labels } h body h) :
      InstrTyping C h (.block .empty body) h
  | loop {C h body}
      (hb : ExprTyping { C with labels := h :: C.labels } h body h) :
      InstrTyping C h (.loop .empty body) h
  | ifThenElse {C h t e}
      (ht : ExprTyping { C with labels := h :: C.labels } h t h)
      (he : ExprTyping { C with labels := h :: C.labels } h e h) :
      InstrTyping C (h + 1) (.ifThenElse .empty t e) h
  | br {C h n} (hl : C.labels[n]? = some h) : InstrTyping C h (.br n) h
  | brIf {C h n} (hl : C.labels[n]? = some h) : InstrTyping C (h + 1) (.brIf n) h
  | throwTag {C h tag} (ht : tag < C.numTags) :
      InstrTyping C (h + 1) (.throw tag) h

/-- The declarative typing judgment of an instruction sequence. -/
inductive ExprTyping : Ctx → Nat → Expr → Nat → Prop
  | nil {C h} : ExprTyping C h .nil h
  | cons {C h₁ h₂ h₃ i e} (hi : InstrTyping C h₁ i h₂)
      (he : ExprTyping C h₂ e h₃) : ExprTyping C h₁ (.cons i e) h₃

end

/-! ## The decidable checker -/

mutual

/-- The stack height after `i`, or `none` if `i` is not typable in `C` at
height `h`. -/
def checkInstr (C : Ctx) (h : Nat) : Instr → Option Nat
  | .unreachable => some h
  | .nop => some h
  | .i32Const _ => some (h + 1)
  | .drop => match h with | 0 => none | k + 1 => some k
  | .iBinOp w op =>
      if w = .i32 ∧ SubsetIBinOp op = true then
        match h with | k + 2 => some (k + 1) | _ => none
      else none
  | .iTestOp w op =>
      if w = .i32 ∧ op = .eqz then
        match h with | 0 => none | k + 1 => some (k + 1)
      else none
  | .iRelOp w _ =>
      if w = .i32 then match h with | k + 2 => some (k + 1) | _ => none else none
  | .localGet i => if i < C.numLocals then some (h + 1) else none
  | .localSet i =>
      if i < C.numLocals then (match h with | 0 => none | k + 1 => some k) else none
  | .localTee i =>
      if i < C.numLocals then
        (match h with | 0 => none | k + 1 => some (k + 1))
      else none
  | .globalGet i =>
      match C.globals[i]? with
      | some g => if g.valType = .num .i32 then some (h + 1) else none
      | none => none
  | .globalSet i =>
      match C.globals[i]? with
      | some g =>
          if g.valType = .num .i32 ∧ g.mutable = true then
            (match h with | 0 => none | k + 1 => some k)
          else none
      | none => none
  | .load t narrow arg =>
      if t = .i32 ∧ narrow = none ∧ arg.memory = 0 then
        (match h with | 0 => none | k + 1 => some (k + 1))
      else none
  | .store t narrow arg =>
      if t = .i32 ∧ narrow = none ∧ arg.memory = 0 then
        (match h with | k + 2 => some k | _ => none)
      else none
  | .memorySize mem => if mem = 0 then some (h + 1) else none
  | .memoryGrow mem =>
      if mem = 0 then (match h with | 0 => none | k + 1 => some (k + 1)) else none
  | .block bt body =>
      if bt = .empty then
        (if checkExpr { C with labels := h :: C.labels } h body = some h then some h
         else none)
      else none
  | .loop bt body =>
      if bt = .empty then
        (if checkExpr { C with labels := h :: C.labels } h body = some h then some h
         else none)
      else none
  | .ifThenElse bt t e =>
      if bt = .empty then
        (match h with
         | 0 => none
         | k + 1 =>
             if checkExpr { C with labels := k :: C.labels } k t = some k ∧
                checkExpr { C with labels := k :: C.labels } k e = some k then some k
             else none)
      else none
  | .br n => if C.labels[n]? = some h then some h else none
  | .brIf n =>
      match h with
      | 0 => none
      | k + 1 => if C.labels[n]? = some k then some k else none
  | .throw tag =>
      if tag < C.numTags then (match h with | 0 => none | k + 1 => some k) else none
  | _ => none

/-- The stack height after an instruction sequence, or `none` if it is not
typable. -/
def checkExpr (C : Ctx) (h : Nat) : Expr → Option Nat
  | .nil => some h
  | .cons i e =>
      match checkInstr C h i with
      | some h₂ => checkExpr C h₂ e
      | none => none

end

/-! ## Equivalence of the checker and the judgment -/

mutual

theorem checkInstr_eq_some_iff (C : Ctx) (h : Nat) (i : Instr) (h' : Nat) :
    checkInstr C h i = some h' ↔ InstrTyping C h i h' := by
  match i with
  | .unreachable =>
    constructor
    · intro hc
      have : h' = h := by simp [checkInstr] at hc; omega
      subst this; exact .unreachable
    · intro hd; cases hd; simp [checkInstr]
  | .nop =>
    constructor
    · intro hc
      have : h' = h := by simp [checkInstr] at hc; omega
      subst this; exact .nop
    · intro hd; cases hd; simp [checkInstr]
  | .i32Const v =>
    constructor
    · intro hc
      have : h' = h + 1 := by simp [checkInstr] at hc; omega
      subst this; exact .i32Const
    · intro hd; cases hd; simp [checkInstr]
  | .drop =>
    cases h with
    | zero =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | succ k =>
      constructor
      · intro hc
        have : h' = k := by simp [checkInstr] at hc; omega
        subst this; exact .drop
      · intro hd; cases hd; simp [checkInstr]
  | .iBinOp w op =>
    match w with
    | .i64 =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .i32 =>
      by_cases hop : SubsetIBinOp op = true
      · match h with
        | 0 =>
          constructor
          · intro hc; simp [checkInstr, hop] at hc
          · intro hd; cases hd
        | 1 =>
          constructor
          · intro hc; simp [checkInstr, hop] at hc
          · intro hd; cases hd
        | k + 2 =>
          constructor
          · intro hc
            have : h' = k + 1 := by simp [checkInstr, hop] at hc; omega
            subst this; exact .iBinOp hop
          · intro hd; cases hd; simp [checkInstr, hop]
      · constructor
        · intro hc; simp [checkInstr, hop] at hc
        · intro hd; cases hd; rename_i hop'; exact absurd hop' hop
  | .iTestOp w op =>
    match w, op with
    | .i64, .eqz =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .i32, .eqz =>
      cases h with
      | zero =>
        constructor
        · intro hc; simp [checkInstr] at hc
        · intro hd; cases hd
      | succ k =>
        constructor
        · intro hc
          have : h' = k + 1 := by simp [checkInstr] at hc; omega
          subst this; exact .iTestOp
        · intro hd; cases hd; simp [checkInstr]
  | .iRelOp w op =>
    match w with
    | .i64 =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .i32 =>
      match h with
      | 0 =>
        constructor
        · intro hc; simp [checkInstr] at hc
        · intro hd; cases hd
      | 1 =>
        constructor
        · intro hc; simp [checkInstr] at hc
        · intro hd; cases hd
      | k + 2 =>
        constructor
        · intro hc
          have : h' = k + 1 := by simp [checkInstr] at hc; omega
          subst this; exact .iRelOp
        · intro hd; cases hd; simp [checkInstr]
  | .localGet idx =>
    by_cases hi : idx < C.numLocals
    · constructor
      · intro hc
        have : h' = h + 1 := by simp [checkInstr, hi] at hc; omega
        subst this; exact .localGet hi
      · intro hd; cases hd; simp [checkInstr, hi]
    · constructor
      · intro hc; simp [checkInstr, hi] at hc
      · intro hd; cases hd; rename_i hi'; exact absurd hi' hi
  | .localSet idx =>
    by_cases hi : idx < C.numLocals
    · cases h with
      | zero =>
        constructor
        · intro hc; simp [checkInstr, hi] at hc
        · intro hd; cases hd
      | succ k =>
        constructor
        · intro hc
          have : h' = k := by simp [checkInstr, hi] at hc; omega
          subst this; exact .localSet hi
        · intro hd; cases hd; simp [checkInstr, hi]
    · constructor
      · intro hc; simp [checkInstr, hi] at hc
      · intro hd; cases hd; rename_i hi'; exact absurd hi' hi
  | .localTee idx =>
    by_cases hi : idx < C.numLocals
    · cases h with
      | zero =>
        constructor
        · intro hc; simp [checkInstr, hi] at hc
        · intro hd; cases hd
      | succ k =>
        constructor
        · intro hc
          have : h' = k + 1 := by simp [checkInstr, hi] at hc; omega
          subst this; exact .localTee hi
        · intro hd; cases hd; simp [checkInstr, hi]
    · constructor
      · intro hc; simp [checkInstr, hi] at hc
      · intro hd; cases hd; rename_i hi'; exact absurd hi' hi
  | .globalGet idx =>
    cases hg : C.globals[idx]? with
    | none =>
      constructor
      · intro hc; simp [checkInstr, hg] at hc
      · intro hd
        cases hd with
        | globalGet hg' _ => rw [hg] at hg'; simp at hg'
    | some g =>
      by_cases ht : g.valType = ValType.num .i32
      · constructor
        · intro hc
          have : h' = h + 1 := by simp [checkInstr, hg, ht] at hc; omega
          subst this; exact .globalGet hg ht
        · intro hd; cases hd; simp [checkInstr, hg, ht]
      · constructor
        · intro hc; simp [checkInstr, hg, ht] at hc
        · intro hd
          cases hd with
          | globalGet hg' ht' =>
            rw [hg] at hg'
            injection hg' with hg'
            subst hg'
            exact absurd ht' ht
  | .globalSet idx =>
    cases hg : C.globals[idx]? with
    | none =>
      constructor
      · intro hc; simp [checkInstr, hg] at hc
      · intro hd
        cases hd with
        | globalSet hg' _ _ => rw [hg] at hg'; simp at hg'
    | some g =>
      by_cases ht : g.valType = ValType.num .i32 ∧ g.mutable = true
      · cases h with
        | zero =>
          constructor
          · intro hc; simp [checkInstr, hg, ht.1, ht.2] at hc
          · intro hd; cases hd
        | succ k =>
          constructor
          · intro hc
            have : h' = k := by simp [checkInstr, hg, ht.1, ht.2] at hc; omega
            subst this; exact .globalSet hg ht.1 ht.2
          · intro hd; cases hd; simp [checkInstr, hg, ht.1, ht.2]
      · constructor
        · intro hc; simp [checkInstr, hg, ht] at hc
        · intro hd
          cases hd with
          | globalSet hg' ht1 ht2 =>
            rw [hg] at hg'
            injection hg' with hg'
            subst hg'
            exact absurd ⟨ht1, ht2⟩ ht
  | .load t narrow arg =>
    match t, narrow with
    | .i32, none =>
      by_cases hm : arg.memory = 0
      · cases h with
        | zero =>
          constructor
          · intro hc; simp [checkInstr, hm] at hc
          · intro hd; cases hd
        | succ k =>
          constructor
          · intro hc
            have : h' = k + 1 := by simp [checkInstr, hm] at hc; omega
            subst this; exact .load hm
          · intro hd; cases hd; simp [checkInstr, hm]
      · constructor
        · intro hc; simp [checkInstr, hm] at hc
        · intro hd; cases hd; rename_i hm'; exact absurd hm' hm
    | .i32, some nw =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .i64, nw =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .f32, nw =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .f64, nw =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
  | .store t narrow arg =>
    match t, narrow with
    | .i32, none =>
      by_cases hm : arg.memory = 0
      · match h with
        | 0 =>
          constructor
          · intro hc; simp [checkInstr, hm] at hc
          · intro hd; cases hd
        | 1 =>
          constructor
          · intro hc; simp [checkInstr, hm] at hc
          · intro hd; cases hd
        | k + 2 =>
          constructor
          · intro hc
            have : h' = k := by simp [checkInstr, hm] at hc; omega
            subst this; exact .store hm
          · intro hd; cases hd; simp [checkInstr, hm]
      · constructor
        · intro hc; simp [checkInstr, hm] at hc
        · intro hd; cases hd; rename_i hm'; exact absurd hm' hm
    | .i32, some nw =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .i64, nw =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .f32, nw =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .f64, nw =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
  | .memorySize mem =>
    by_cases hm : mem = 0
    · subst hm
      constructor
      · intro hc
        have : h' = h + 1 := by simp [checkInstr] at hc; omega
        subst this; exact .memorySize
      · intro hd; cases hd; simp [checkInstr]
    · constructor
      · intro hc; simp [checkInstr, hm] at hc
      · intro hd; cases hd; exact absurd rfl hm
  | .memoryGrow mem =>
    by_cases hm : mem = 0
    · subst hm
      cases h with
      | zero =>
        constructor
        · intro hc; simp [checkInstr] at hc
        · intro hd; cases hd
      | succ k =>
        constructor
        · intro hc
          have : h' = k + 1 := by simp [checkInstr] at hc; omega
          subst this; exact .memoryGrow
        · intro hd; cases hd; simp [checkInstr]
    · constructor
      · intro hc; simp [checkInstr, hm] at hc
      · intro hd; cases hd; exact absurd rfl hm
  | .block bt body =>
    match bt with
    | .empty =>
      by_cases hb : checkExpr { C with labels := h :: C.labels } h body = some h
      · constructor
        · intro hc
          have : h' = h := by simp [checkInstr, hb] at hc; omega
          subst this
          exact .block ((checkExpr_eq_some_iff _ _ body _).mp hb)
        · intro hd; cases hd; simp [checkInstr, hb]
      · constructor
        · intro hc; simp [checkInstr, hb] at hc
        · intro hd
          cases hd
          rename_i hd
          exact absurd ((checkExpr_eq_some_iff _ _ body _).mpr hd) hb
    | .value vt =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .typeIndex ti =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
  | .loop bt body =>
    match bt with
    | .empty =>
      by_cases hb : checkExpr { C with labels := h :: C.labels } h body = some h
      · constructor
        · intro hc
          have : h' = h := by simp [checkInstr, hb] at hc; omega
          subst this
          exact .loop ((checkExpr_eq_some_iff _ _ body _).mp hb)
        · intro hd; cases hd; simp [checkInstr, hb]
      · constructor
        · intro hc; simp [checkInstr, hb] at hc
        · intro hd
          cases hd
          rename_i hd
          exact absurd ((checkExpr_eq_some_iff _ _ body _).mpr hd) hb
    | .value vt =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .typeIndex ti =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
  | .ifThenElse bt thenBody elseBody =>
    match bt with
    | .empty =>
      cases h with
      | zero =>
        constructor
        · intro hc; simp [checkInstr] at hc
        · intro hd; cases hd
      | succ k =>
        by_cases hb :
            checkExpr { C with labels := k :: C.labels } k thenBody = some k ∧
              checkExpr { C with labels := k :: C.labels } k elseBody = some k
        · constructor
          · intro hc
            have : h' = k := by simp [checkInstr, hb.1, hb.2] at hc; omega
            subst this
            exact .ifThenElse ((checkExpr_eq_some_iff _ _ thenBody _).mp hb.1)
              ((checkExpr_eq_some_iff _ _ elseBody _).mp hb.2)
          · intro hd; cases hd; simp [checkInstr, hb.1, hb.2]
        · constructor
          · intro hc; simp [checkInstr, hb] at hc
          · intro hd
            cases hd
            rename_i hdt hde
            exact absurd ⟨(checkExpr_eq_some_iff _ _ thenBody _).mpr hdt,
              (checkExpr_eq_some_iff _ _ elseBody _).mpr hde⟩ hb
    | .value vt =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | .typeIndex ti =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
  | .br n =>
    by_cases hl : C.labels[n]? = some h
    · constructor
      · intro hc
        have : h' = h := by simp [checkInstr, hl] at hc; omega
        subst this; exact .br hl
      · intro hd; cases hd; simp [checkInstr, hl]
    · constructor
      · intro hc; simp [checkInstr, hl] at hc
      · intro hd; cases hd; rename_i hl'; exact absurd hl' hl
  | .brIf n =>
    cases h with
    | zero =>
      constructor
      · intro hc; simp [checkInstr] at hc
      · intro hd; cases hd
    | succ k =>
      by_cases hl : C.labels[n]? = some k
      · constructor
        · intro hc
          have : h' = k := by simp [checkInstr, hl] at hc; omega
          subst this; exact .brIf hl
        · intro hd; cases hd; simp [checkInstr, hl]
      · constructor
        · intro hc; simp [checkInstr, hl] at hc
        · intro hd; cases hd; rename_i hl'; exact absurd hl' hl
  | .throw tag =>
    by_cases ht : tag < C.numTags
    · cases h with
      | zero =>
        constructor
        · intro hc; simp [checkInstr, ht] at hc
        · intro hd; cases hd
      | succ k =>
        constructor
        · intro hc
          have : h' = k := by simp [checkInstr, ht] at hc; omega
          subst this; exact .throwTag ht
        · intro hd; cases hd; simp [checkInstr, ht]
    · constructor
      · intro hc; simp [checkInstr, ht] at hc
      · intro hd; cases hd; rename_i ht'; exact absurd ht' ht
  | .brTable _ _ | .ret | .call _ | .callIndirect _ _ | .returnCall _
  | .returnCallIndirect _ _ | .throwRef | .tryTable _ _ _ | .select _
  | .refNull _ | .refIsNull | .refFunc _ | .refEq | .refAsNonNull
  | .refTest _ _ | .refCast _ _ | .refI31 | .i31GetS | .i31GetU
  | .anyConvertExtern | .externConvertAny
  | .structNew _ | .structNewDefault _ | .structGet _ _ _ | .structSet _ _
  | .arrayNew _ | .arrayNewDefault _ | .arrayGet _ _ | .arraySet _ | .arrayLen
  | .tableGet _ | .tableSet _ | .tableSize _ | .tableGrow _ | .tableFill _
  | .tableCopy _ _ | .tableInit _ _ | .elemDrop _
  | .vecLoad _ | .vecStore _ | .memoryFill _ | .memoryCopy _ _
  | .memoryInit _ _ | .dataDrop _
  | .i64Const _ | .f32Const _ | .f64Const _
  | .iUnOp _ _ | .fUnOp _ _ | .fBinOp _ _ | .fRelOp _ _ | .cvtOp _
  | .vecConst _ _ | .vecUnOp _ _ | .vecBinOp _ _ | .vecRelOp _ _
  | .vecBitselect | .vecSplat _ | .vecExtractLane _ _ _
  | .vecReplaceLane _ _ | .vecShuffle _ =>
    constructor
    · intro hc; simp [checkInstr] at hc
    · intro hd; cases hd

theorem checkExpr_eq_some_iff (C : Ctx) (h : Nat) (e : Expr) (h' : Nat) :
    checkExpr C h e = some h' ↔ ExprTyping C h e h' := by
  match e with
  | .nil =>
    simp only [checkExpr, Option.some.injEq]
    constructor
    · rintro rfl; exact .nil
    · rintro (_ | _); rfl
  | .cons i rest =>
    constructor
    · intro hc
      simp only [checkExpr] at hc
      cases hi : checkInstr C h i with
      | none => rw [hi] at hc; exact absurd hc (by simp)
      | some h₂ =>
        rw [hi] at hc
        exact .cons ((checkInstr_eq_some_iff C h i h₂).mp hi)
          ((checkExpr_eq_some_iff C h₂ rest h').mp hc)
    · intro hd
      cases hd with
      | @cons _ _ h₂ _ _ _ hi he =>
        simp only [checkExpr]
        rw [(checkInstr_eq_some_iff C h i h₂).mpr hi]
        exact (checkExpr_eq_some_iff C h₂ rest h').mpr he

end

instance instDecidableInstrTyping (C : Ctx) (h : Nat) (i : Instr) (h' : Nat) :
    Decidable (InstrTyping C h i h') :=
  decidable_of_iff _ (checkInstr_eq_some_iff C h i h')

instance instDecidableExprTyping (C : Ctx) (h : Nat) (e : Expr) (h' : Nat) :
    Decidable (ExprTyping C h e h') :=
  decidable_of_iff _ (checkExpr_eq_some_iff C h e h')

/-- A typing derivation determines the resulting height. -/
theorem InstrTyping.functional {C : Ctx} {h : Nat} {i : Instr} {a b : Nat}
    (ha : InstrTyping C h i a) (hb : InstrTyping C h i b) : a = b := by
  have := (checkInstr_eq_some_iff C h i a).mpr ha
  have := (checkInstr_eq_some_iff C h i b).mpr hb
  simp_all

/-- A sequence typing derivation determines the resulting height. -/
theorem ExprTyping.functional {C : Ctx} {h : Nat} {e : Expr} {a b : Nat}
    (ha : ExprTyping C h e a) (hb : ExprTyping C h e b) : a = b := by
  have := (checkExpr_eq_some_iff C h e a).mpr ha
  have := (checkExpr_eq_some_iff C h e b).mpr hb
  simp_all

/-! ## Module validation -/

namespace Module

/-- The flattened sub-type list of the module's recursive type groups. -/
def flatTypes (m : Module) : List SubType := m.types.flatMap (·.types)

/-- The function type at index `i`, when that index names a function type. -/
def funcTypeAt (m : Module) (i : Nat) : Option FuncType :=
  match (flatTypes m)[i]? with
  | some st => match st.body with
      | .func ft => some ft
      | _ => none
  | none => none

/-- All parameters, results and locals of the released subset are `i32`. -/
def isI32 (t : ValType) : Bool := t = ValType.num .i32

/-- The validation context of a defined function. -/
def funcCtx (m : Module) (ft : FuncType) (f : Func) : Ctx :=
  { numLocals := ft.params.length + f.locals.length
    globals := m.globals.map (·.type)
    numTags := m.tags.length
    labels := [] }

/-- A defined function is valid: its declared type exists, is entirely `i32`,
its locals are `i32`, and its body takes the empty operand stack to exactly the
number of results. -/
def checkFunc (m : Module) (f : Func) : Bool :=
  match funcTypeAt m f.type with
  | none => false
  | some ft =>
      ft.params.all isI32 && ft.results.all isI32 && f.locals.all isI32 &&
      (checkExpr (funcCtx m ft f) 0 f.body = some ft.results.length)

/-- A defined global is valid: type `i32` and a constant initializer. -/
def checkGlobal (g : Global) : Bool :=
  isI32 g.type.valType &&
    (match g.init with
     | .cons (.i32Const _) .nil => true
     | _ => false)

/-- A declared tag is valid for the subset: one `i32` payload, no results. -/
def checkTag (t : TagType) : Bool :=
  t.funcType.params == [ValType.num .i32] && t.funcType.results == []

/-- The single memory of the released profile. -/
def checkMems (m : Module) : Bool :=
  match m.mems with
  | [mem] =>
      mem.type.addressType == AddressType.i32 &&
      decide (mem.type.limits.min ≤ Memory.hardMaxPages) &&
      (match mem.type.limits.max with
       | none => true
       | some k => decide (mem.type.limits.min ≤ k) && decide (k ≤ Memory.hardMaxPages))
  | _ => false

/-- The module exports the required memory. -/
def exportsMemory (m : Module) : Bool :=
  m.exports.any (fun e => e.name == memoryExportName && e.desc == ExportDesc.mem 0)

/-- The index of the exported `gemm` function, if any. -/
def gemmIndex? (m : Module) : Option Nat :=
  match m.exports.find? (fun e => e.name == gemmExportName) with
  | some ⟨_, .func i⟩ => some i
  | _ => none

/-- The exported `gemm` function exists and has the pinned ABI type. -/
def checkGemmExport (m : Module) : Bool :=
  match gemmIndex? m with
  | none => false
  | some i =>
      match m.funcs[i]? with
      | none => false
      | some f => funcTypeAt m f.type == some gemmFuncType

/-- The optional start function exists and has type `() → ()`. -/
def checkStart (m : Module) : Bool :=
  match m.start with
  | none => true
  | some i =>
      match m.funcs[i]? with
      | none => false
      | some f => funcTypeAt m f.type == some { params := [], results := [] }

/-- The released profile is closed and carries no table, element or data
section: those Core forms are grammatically expressible and rejected here. -/
def checkClosed (m : Module) : Bool :=
  m.imports.isEmpty && m.tables.isEmpty && m.elems.isEmpty && m.datas.isEmpty

end Module

/-- The decidable release validator. -/
def validate (m : Module) : Bool :=
  Module.checkClosed m &&
  Module.checkMems m &&
  m.globals.all Module.checkGlobal &&
  m.tags.all Module.checkTag &&
  m.funcs.all (Module.checkFunc m) &&
  Module.exportsMemory m &&
  Module.checkGemmExport m &&
  Module.checkStart m

/-- The declarative validity judgment of the released profile: the conjunction
of the closedness, memory, global, tag, function, export and start conditions
of SPEC section 7.2, with every function body carrying a derivation of the
declarative typing judgment. -/
def DeclarativelyValid (m : Module) : Prop :=
  Module.checkClosed m = true ∧
  Module.checkMems m = true ∧
  (∀ g ∈ m.globals, Module.checkGlobal g = true) ∧
  (∀ t ∈ m.tags, Module.checkTag t = true) ∧
  (∀ f ∈ m.funcs, ∃ ft : FuncType,
      Module.funcTypeAt m f.type = some ft ∧
      (∀ t ∈ ft.params, Module.isI32 t = true) ∧
      (∀ t ∈ ft.results, Module.isI32 t = true) ∧
      (∀ t ∈ f.locals, Module.isI32 t = true) ∧
      ExprTyping (Module.funcCtx m ft f) 0 f.body ft.results.length) ∧
  Module.exportsMemory m = true ∧
  Module.checkGemmExport m = true ∧
  Module.checkStart m = true

theorem checkFunc_eq_true_iff (m : Module) (f : Func) :
    Module.checkFunc m f = true ↔
      ∃ ft : FuncType,
        Module.funcTypeAt m f.type = some ft ∧
        (∀ t ∈ ft.params, Module.isI32 t = true) ∧
        (∀ t ∈ ft.results, Module.isI32 t = true) ∧
        (∀ t ∈ f.locals, Module.isI32 t = true) ∧
        ExprTyping (Module.funcCtx m ft f) 0 f.body ft.results.length := by
  unfold Module.checkFunc
  cases hft : Module.funcTypeAt m f.type with
  | none =>
    simp only [Bool.false_eq_true, false_iff]
    rintro ⟨ft, hc, _⟩
    simp at hc
  | some ft =>
    constructor
    · intro hc
      simp only [Bool.and_eq_true, List.all_eq_true] at hc
      obtain ⟨⟨⟨hp, hr⟩, hl⟩, hbody⟩ := hc
      exact ⟨ft, rfl, hp, hr, hl,
        (checkExpr_eq_some_iff _ _ _ _).mp (of_decide_eq_true hbody)⟩
    · rintro ⟨ft', hc, hp, hr, hl, hbody⟩
      injection hc with hc
      subst hc
      simp only [Bool.and_eq_true, List.all_eq_true]
      exact ⟨⟨⟨hp, hr⟩, hl⟩,
        decide_eq_true ((checkExpr_eq_some_iff _ _ _ _).mpr hbody)⟩

/-- **SPEC section 7.3.** The executable validator decides the declarative
validity judgment. -/
theorem validate_bool_iff (m : Module) :
    validate m = true ↔ DeclarativelyValid m := by
  unfold validate DeclarativelyValid
  simp only [Bool.and_eq_true, List.all_eq_true]
  constructor
  · rintro ⟨⟨⟨⟨⟨⟨⟨hclosed, hmems⟩, hglob⟩, htags⟩, hfuncs⟩, hexp⟩, hgemm⟩, hstart⟩
    exact ⟨hclosed, hmems, hglob, htags,
      fun f hf => (checkFunc_eq_true_iff m f).mp (hfuncs f hf), hexp, hgemm, hstart⟩
  · rintro ⟨hclosed, hmems, hglob, htags, hfuncs, hexp, hgemm, hstart⟩
    exact ⟨⟨⟨⟨⟨⟨⟨hclosed, hmems⟩, hglob⟩, htags⟩,
      fun f hf => (checkFunc_eq_true_iff m f).mpr (hfuncs f hf)⟩, hexp⟩, hgemm⟩, hstart⟩

instance instDecidableDeclarativelyValid (m : Module) :
    Decidable (DeclarativelyValid m) :=
  decidable_of_iff _ (validate_bool_iff m)

/-- Validity is a decidable property, so the validator is total: it either
accepts or rejects, and never diverges. -/
theorem validate_eq_false_iff (m : Module) :
    validate m = false ↔ ¬ DeclarativelyValid m := by
  rw [← validate_bool_iff]
  cases validate m <;> simp

end WasmGemmGnaf.Wasm
