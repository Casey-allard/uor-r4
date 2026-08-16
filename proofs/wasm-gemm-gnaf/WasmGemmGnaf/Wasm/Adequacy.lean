/-
  Wasm/Adequacy.lean --- the conformance map from pinned rule identifiers to
  Lean declarations.

  Normative source: SPEC.md section 7.1: "`Adequacy` proves correspondence
  between these declarations and the vendored rule identifiers in the
  conformance map", and

    "`profile_matches_pinned_revision` means that the concrete model and map are
     identity-bound to the vendored revision and that every enabled vendored
     rule has exactly one mapped Lean declaration.  It does not claim that Lean
     can derive English prose from bytes.  The reviewed transcription from
     normative rules to initial Lean definitions is an explicitly disclosed
     authority boundary; all subsequent GEMM, cost, coverage, and optimality
     reasoning is kernel checked."

  ## What this file establishes

  * `PinnedCoreRuleId` is a finite inductive with a complete enumeration
    (`mem_all`) that is proved duplicate free (`all_nodup`), so it carries a
    `Fintype` instance.
  * every identifier carries a feature family and a status, and the status is
    proved to agree with the release feature matrix of `Wasm/Feature.lean`
    (`status_eq_rejected_iff`): an identifier has no Lean declaration exactly
    when its family is rejected by the profile.
  * the map `leanDeclaration?` is **total** over the enabled identifiers
    (`adequacy_map_total_on_enabled`) and **injective** there
    (`adequacy_map_injective_on_enabled`); the same holds for the fully
    qualified names (`fullDeclaration?_injective_on_enabled`) and for the
    identifier spellings themselves (`ruleId_injective`).
  * the map object `core3AdequacyMap` carries the pinned commit, and that
    commit is proved equal to the commit every lawful `Wasm.Profile` carries
    (`adequacy_profile_revision_agree`).  This is the identity binding to the
    pinned revision.
  * `mapped_declarations_referenced` mentions every mapped Lean declaration, so
    renaming or deleting one breaks this file rather than silently invalidating
    the map.

  ## The authority boundary --- what this file does NOT establish

  This file does **not** claim that Lean derives English prose from bytes, and
  it does not prove `profile_matches_pinned_revision` of SPEC section 15.  Three
  gaps are disclosed explicitly:

  1. **The pinned tree is not vendored.**  `authority/manifest.json` records
     `wasmCore.vendored = false` and `vendor/wasm-spec/` contains only a README.
     The identifier spellings below are therefore a *hand transcription* of the
     pinned `wg-3.0` rule names, not a list read from a vendored tree, and no
     theorem here compares them with one.  Release gate step 1 fails until the
     tree is vendored; that failure is the conforming behaviour.
  2. **Strings are not reflected.**  `mapped_declarations_referenced` proves
     that a declaration with each mapped *role* exists in the environment, but
     Lean cannot check --- without metaprogramming, which this development does
     not use --- that the *string* `"Step.nop"` names the declaration
     `Step.nop`.  That correspondence is by review.
  3. **Coverage is relative to this enumeration.**  Every "total" and "complete"
     theorem below quantifies over `PinnedCoreRuleId`, which is the list written
     in this file.  It is evidence that the map has no hole *inside* the
     declared rule set; it is not evidence that the declared rule set is the
     whole pinned rule set.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Binary
import WasmGemmGnaf.Wasm.Run
import WasmGemmGnaf.Wasm.Table
import WasmGemmGnaf.Wasm.Vector

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Two general lemmas -/

/-- A list whose image under `f` is duplicate free is one on which `f` is
injective. -/
theorem nodup_map_injOn {α β : Type} (f : α → β) :
    ∀ (l : List α), (l.map f).Nodup → ∀ a b, a ∈ l → b ∈ l → f a = f b → a = b := by
  intro l
  induction l with
  | nil => intro _ a b ha _ _; exact absurd ha (by simp)
  | cons x xs ih =>
    intro h a b ha hb hf
    rw [List.map_cons, List.nodup_cons] at h
    rcases List.mem_cons.mp ha with rfl | ha'
    · rcases List.mem_cons.mp hb with rfl | hb'
      · rfl
      · exact absurd (show f a ∈ xs.map f by rw [hf]; exact List.mem_map_of_mem hb') h.1
    · rcases List.mem_cons.mp hb with rfl | hb'
      · exact absurd (show f b ∈ xs.map f by rw [← hf]; exact List.mem_map_of_mem ha') h.1
      · exact ih h.2 a b ha' hb' hf

/-- Appending a fixed prefix to a string is injective. -/
theorem string_append_left_cancel {p a b : String} (h : p ++ a = p ++ b) : a = b := by
  have hd : p.toList ++ a.toList = p.toList ++ b.toList := by
    rw [← String.toList_append, ← String.toList_append, h]
  have h2 := List.append_cancel_left hd
  have h3 := congrArg String.ofList h2
  rwa [String.ofList_toList, String.ofList_toList] at h3

/-! ## The status of a pinned rule -/

/-- What the released development does with a pinned rule. -/
inductive RuleStatus
  /-- The rule has a Lean declaration and is reachable in the released
  machine. -/
  | modelled
  /-- The rule has a Lean declaration, but the released validator rejects the
  instruction, so no released execution reaches it.  See the scope theorems in
  `Wasm/Table.lean` and `Wasm/Vector.lean`. -/
  | modelledUnreachable
  /-- The rule belongs to a family the profile rejects; it has no Lean
  declaration at all. -/
  | rejectedByProfile
  deriving DecidableEq, Repr, Inhabited

namespace RuleStatus

def all : List RuleStatus := [modelled, modelledUnreachable, rejectedByProfile]

theorem mem_all (s : RuleStatus) : s ∈ all := by cases s <;> decide

theorem all_nodup : all.Nodup := by decide

instance instFintype : Fintype RuleStatus where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

end RuleStatus

/-! ## The pinned rule identifiers -/

/-- The identifiers of the pinned Core 3.0 rules covered by this development.
See the header for the exact scope of this enumeration. -/
inductive PinnedCoreRuleId
  | decodeModule
  | decodeUleb128
  | validUnreachable
  | validNop
  | validI32Const
  | validDrop
  | validIBinOp
  | validITestOp
  | validIRelOp
  | validLocalGet
  | validLocalSet
  | validLocalTee
  | validGlobalGet
  | validGlobalSet
  | validLoad
  | validStore
  | validMemorySize
  | validMemoryGrow
  | validBlock
  | validLoop
  | validIfThenElse
  | validBr
  | validBrIf
  | validThrowTag
  | validExprNil
  | validExprCons
  | validModule
  | validFunc
  | validMems
  | validGemmExport
  | validClosed
  | storeAlloc
  | storeLoadBytes
  | storeStoreBytes
  | storeMemoryGrow
  | execUnreachable
  | execNop
  | execI32Const
  | execDrop
  | execIBinOp
  | execIBinOpTrap
  | execITestOp
  | execIRelOp
  | execLocalGet
  | execLocalSet
  | execLocalTee
  | execGlobalGet
  | execGlobalSet
  | execLoad
  | execLoadTrap
  | execStore
  | execStoreTrap
  | execMemorySize
  | execMemoryGrowSucceed
  | execMemoryGrowRefuse
  | execBlock
  | execLoop
  | execIfFalse
  | execIfTrue
  | execBrLoop
  | execBrBlock
  | execBrIfFalse
  | execBrIfLoop
  | execBrIfBlock
  | execThrowTag
  | execExitLabel
  | execReturnGemm
  | execEnterGemm
  | execInstallTrap
  | trapUnreachable
  | trapOutOfBounds
  | trapDivideByZero
  | nondetMemoryGrow
  | nondetNanResult
  | tableGet
  | tableSet
  | tableSize
  | tableGrow
  | tableFill
  | tableCopy
  | tableInit
  | tableElemDrop
  | vectorExtractLane
  | vectorReplaceLane
  | vectorSplat
  | vectorLanewise
  | vectorShapeWidth
  | refNull
  | refFunc
  | rejectMemory64Load
  | rejectAtomicRmw
  | rejectRelaxedFma
  | rejectComponentModel
  deriving DecidableEq, Repr, Inhabited

namespace PinnedCoreRuleId

/-- The complete enumeration of pinned rule identifiers. -/
def all : List PinnedCoreRuleId :=
  [ .decodeModule
  , .decodeUleb128
  , .validUnreachable
  , .validNop
  , .validI32Const
  , .validDrop
  , .validIBinOp
  , .validITestOp
  , .validIRelOp
  , .validLocalGet
  , .validLocalSet
  , .validLocalTee
  , .validGlobalGet
  , .validGlobalSet
  , .validLoad
  , .validStore
  , .validMemorySize
  , .validMemoryGrow
  , .validBlock
  , .validLoop
  , .validIfThenElse
  , .validBr
  , .validBrIf
  , .validThrowTag
  , .validExprNil
  , .validExprCons
  , .validModule
  , .validFunc
  , .validMems
  , .validGemmExport
  , .validClosed
  , .storeAlloc
  , .storeLoadBytes
  , .storeStoreBytes
  , .storeMemoryGrow
  , .execUnreachable
  , .execNop
  , .execI32Const
  , .execDrop
  , .execIBinOp
  , .execIBinOpTrap
  , .execITestOp
  , .execIRelOp
  , .execLocalGet
  , .execLocalSet
  , .execLocalTee
  , .execGlobalGet
  , .execGlobalSet
  , .execLoad
  , .execLoadTrap
  , .execStore
  , .execStoreTrap
  , .execMemorySize
  , .execMemoryGrowSucceed
  , .execMemoryGrowRefuse
  , .execBlock
  , .execLoop
  , .execIfFalse
  , .execIfTrue
  , .execBrLoop
  , .execBrBlock
  , .execBrIfFalse
  , .execBrIfLoop
  , .execBrIfBlock
  , .execThrowTag
  , .execExitLabel
  , .execReturnGemm
  , .execEnterGemm
  , .execInstallTrap
  , .trapUnreachable
  , .trapOutOfBounds
  , .trapDivideByZero
  , .nondetMemoryGrow
  , .nondetNanResult
  , .tableGet
  , .tableSet
  , .tableSize
  , .tableGrow
  , .tableFill
  , .tableCopy
  , .tableInit
  , .tableElemDrop
  , .vectorExtractLane
  , .vectorReplaceLane
  , .vectorSplat
  , .vectorLanewise
  , .vectorShapeWidth
  , .refNull
  , .refFunc
  , .rejectMemory64Load
  , .rejectAtomicRmw
  , .rejectRelaxedFma
  , .rejectComponentModel ]

theorem mem_all (id : PinnedCoreRuleId) : id ∈ all := by cases id <;> decide

/-- **Duplicate-free coverage.** -/
theorem all_nodup : all.Nodup := by decide

theorem all_length : all.length = 93 := rfl

instance instFintype : Fintype PinnedCoreRuleId where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

/-- The vendored spelling of a rule identifier (see the header: this is a
hand transcription, not a list read from a vendored tree). -/
def ruleId : PinnedCoreRuleId → String
  | .decodeModule => "decode.module"
  | .decodeUleb128 => "decode.uleb128"
  | .validUnreachable => "valid.unreachable"
  | .validNop => "valid.nop"
  | .validI32Const => "valid.i32-const"
  | .validDrop => "valid.drop"
  | .validIBinOp => "valid.i-binop"
  | .validITestOp => "valid.i-testop"
  | .validIRelOp => "valid.i-relop"
  | .validLocalGet => "valid.local-get"
  | .validLocalSet => "valid.local-set"
  | .validLocalTee => "valid.local-tee"
  | .validGlobalGet => "valid.global-get"
  | .validGlobalSet => "valid.global-set"
  | .validLoad => "valid.load"
  | .validStore => "valid.store"
  | .validMemorySize => "valid.memory-size"
  | .validMemoryGrow => "valid.memory-grow"
  | .validBlock => "valid.block"
  | .validLoop => "valid.loop"
  | .validIfThenElse => "valid.if"
  | .validBr => "valid.br"
  | .validBrIf => "valid.br-if"
  | .validThrowTag => "valid.throw"
  | .validExprNil => "valid.expr-nil"
  | .validExprCons => "valid.expr-cons"
  | .validModule => "valid.module"
  | .validFunc => "valid.func"
  | .validMems => "valid.mems"
  | .validGemmExport => "valid.gemm-export"
  | .validClosed => "valid.closed"
  | .storeAlloc => "store.alloc"
  | .storeLoadBytes => "store.load-bytes"
  | .storeStoreBytes => "store.store-bytes"
  | .storeMemoryGrow => "store.memory-grow"
  | .execUnreachable => "exec.unreachable"
  | .execNop => "exec.nop"
  | .execI32Const => "exec.i32-const"
  | .execDrop => "exec.drop"
  | .execIBinOp => "exec.i-binop"
  | .execIBinOpTrap => "exec.i-binop-trap"
  | .execITestOp => "exec.i-testop"
  | .execIRelOp => "exec.i-relop"
  | .execLocalGet => "exec.local-get"
  | .execLocalSet => "exec.local-set"
  | .execLocalTee => "exec.local-tee"
  | .execGlobalGet => "exec.global-get"
  | .execGlobalSet => "exec.global-set"
  | .execLoad => "exec.load"
  | .execLoadTrap => "exec.load-trap"
  | .execStore => "exec.store"
  | .execStoreTrap => "exec.store-trap"
  | .execMemorySize => "exec.memory-size"
  | .execMemoryGrowSucceed => "exec.memory-grow-succeed"
  | .execMemoryGrowRefuse => "exec.memory-grow-refuse"
  | .execBlock => "exec.block"
  | .execLoop => "exec.loop"
  | .execIfFalse => "exec.if-false"
  | .execIfTrue => "exec.if-true"
  | .execBrLoop => "exec.br-loop"
  | .execBrBlock => "exec.br-block"
  | .execBrIfFalse => "exec.br-if-false"
  | .execBrIfLoop => "exec.br-if-loop"
  | .execBrIfBlock => "exec.br-if-block"
  | .execThrowTag => "exec.throw"
  | .execExitLabel => "exec.exit-label"
  | .execReturnGemm => "exec.return-gemm"
  | .execEnterGemm => "exec.enter-gemm"
  | .execInstallTrap => "exec.install-trap"
  | .trapUnreachable => "trap.unreachable"
  | .trapOutOfBounds => "trap.out-of-bounds"
  | .trapDivideByZero => "trap.divide-by-zero"
  | .nondetMemoryGrow => "nondet.memory-grow"
  | .nondetNanResult => "nondet.nan-result"
  | .tableGet => "table.get"
  | .tableSet => "table.set"
  | .tableSize => "table.size"
  | .tableGrow => "table.grow"
  | .tableFill => "table.fill"
  | .tableCopy => "table.copy"
  | .tableInit => "table.init"
  | .tableElemDrop => "table.elem-drop"
  | .vectorExtractLane => "vector.extract-lane"
  | .vectorReplaceLane => "vector.replace-lane"
  | .vectorSplat => "vector.splat"
  | .vectorLanewise => "vector.lanewise"
  | .vectorShapeWidth => "vector.shape-width"
  | .refNull => "ref.null"
  | .refFunc => "ref.func"
  | .rejectMemory64Load => "reject.memory64-load"
  | .rejectAtomicRmw => "reject.atomic-rmw"
  | .rejectRelaxedFma => "reject.relaxed-fma"
  | .rejectComponentModel => "reject.component-model"

/-- The Core 3.0 feature family a rule belongs to. -/
def family : PinnedCoreRuleId → FeatureFamily
  | .decodeModule => .scalarCore
  | .decodeUleb128 => .scalarCore
  | .validUnreachable => .scalarCore
  | .validNop => .scalarCore
  | .validI32Const => .scalarCore
  | .validDrop => .scalarCore
  | .validIBinOp => .scalarCore
  | .validITestOp => .scalarCore
  | .validIRelOp => .scalarCore
  | .validLocalGet => .scalarCore
  | .validLocalSet => .scalarCore
  | .validLocalTee => .scalarCore
  | .validGlobalGet => .scalarCore
  | .validGlobalSet => .scalarCore
  | .validLoad => .bulkMemoryMultipleMemoriesTables
  | .validStore => .bulkMemoryMultipleMemoriesTables
  | .validMemorySize => .bulkMemoryMultipleMemoriesTables
  | .validMemoryGrow => .bulkMemoryMultipleMemoriesTables
  | .validBlock => .multiValueAndExtendedConst
  | .validLoop => .multiValueAndExtendedConst
  | .validIfThenElse => .multiValueAndExtendedConst
  | .validBr => .scalarCore
  | .validBrIf => .scalarCore
  | .validThrowTag => .exceptionHandling
  | .validExprNil => .scalarCore
  | .validExprCons => .scalarCore
  | .validModule => .scalarCore
  | .validFunc => .scalarCore
  | .validMems => .bulkMemoryMultipleMemoriesTables
  | .validGemmExport => .scalarCore
  | .validClosed => .scalarCore
  | .storeAlloc => .scalarCore
  | .storeLoadBytes => .bulkMemoryMultipleMemoriesTables
  | .storeStoreBytes => .bulkMemoryMultipleMemoriesTables
  | .storeMemoryGrow => .bulkMemoryMultipleMemoriesTables
  | .execUnreachable => .scalarCore
  | .execNop => .scalarCore
  | .execI32Const => .scalarCore
  | .execDrop => .scalarCore
  | .execIBinOp => .scalarCore
  | .execIBinOpTrap => .scalarCore
  | .execITestOp => .scalarCore
  | .execIRelOp => .scalarCore
  | .execLocalGet => .scalarCore
  | .execLocalSet => .scalarCore
  | .execLocalTee => .scalarCore
  | .execGlobalGet => .scalarCore
  | .execGlobalSet => .scalarCore
  | .execLoad => .bulkMemoryMultipleMemoriesTables
  | .execLoadTrap => .bulkMemoryMultipleMemoriesTables
  | .execStore => .bulkMemoryMultipleMemoriesTables
  | .execStoreTrap => .bulkMemoryMultipleMemoriesTables
  | .execMemorySize => .bulkMemoryMultipleMemoriesTables
  | .execMemoryGrowSucceed => .bulkMemoryMultipleMemoriesTables
  | .execMemoryGrowRefuse => .bulkMemoryMultipleMemoriesTables
  | .execBlock => .multiValueAndExtendedConst
  | .execLoop => .multiValueAndExtendedConst
  | .execIfFalse => .multiValueAndExtendedConst
  | .execIfTrue => .multiValueAndExtendedConst
  | .execBrLoop => .scalarCore
  | .execBrBlock => .scalarCore
  | .execBrIfFalse => .scalarCore
  | .execBrIfLoop => .scalarCore
  | .execBrIfBlock => .scalarCore
  | .execThrowTag => .exceptionHandling
  | .execExitLabel => .multiValueAndExtendedConst
  | .execReturnGemm => .scalarCore
  | .execEnterGemm => .scalarCore
  | .execInstallTrap => .bulkMemoryMultipleMemoriesTables
  | .trapUnreachable => .scalarCore
  | .trapOutOfBounds => .bulkMemoryMultipleMemoriesTables
  | .trapDivideByZero => .scalarCore
  | .nondetMemoryGrow => .bulkMemoryMultipleMemoriesTables
  | .nondetNanResult => .scalarCore
  | .tableGet => .bulkMemoryMultipleMemoriesTables
  | .tableSet => .bulkMemoryMultipleMemoriesTables
  | .tableSize => .bulkMemoryMultipleMemoriesTables
  | .tableGrow => .bulkMemoryMultipleMemoriesTables
  | .tableFill => .bulkMemoryMultipleMemoriesTables
  | .tableCopy => .bulkMemoryMultipleMemoriesTables
  | .tableInit => .bulkMemoryMultipleMemoriesTables
  | .tableElemDrop => .bulkMemoryMultipleMemoriesTables
  | .vectorExtractLane => .fixedWidthSimd128
  | .vectorReplaceLane => .fixedWidthSimd128
  | .vectorSplat => .fixedWidthSimd128
  | .vectorLanewise => .fixedWidthSimd128
  | .vectorShapeWidth => .fixedWidthSimd128
  | .refNull => .referenceTypesTypedFunctionReferencesGc
  | .refFunc => .referenceTypesTypedFunctionReferencesGc
  | .rejectMemory64Load => .memory64
  | .rejectAtomicRmw => .sharedMemoriesAtomicsThreads
  | .rejectRelaxedFma => .relaxedSimd
  | .rejectComponentModel => .componentModelAndPostCore3

/-- What the released development does with the rule. -/
def status : PinnedCoreRuleId → RuleStatus
  | .decodeModule => .modelled
  | .decodeUleb128 => .modelled
  | .validUnreachable => .modelled
  | .validNop => .modelled
  | .validI32Const => .modelled
  | .validDrop => .modelled
  | .validIBinOp => .modelled
  | .validITestOp => .modelled
  | .validIRelOp => .modelled
  | .validLocalGet => .modelled
  | .validLocalSet => .modelled
  | .validLocalTee => .modelled
  | .validGlobalGet => .modelled
  | .validGlobalSet => .modelled
  | .validLoad => .modelled
  | .validStore => .modelled
  | .validMemorySize => .modelled
  | .validMemoryGrow => .modelled
  | .validBlock => .modelled
  | .validLoop => .modelled
  | .validIfThenElse => .modelled
  | .validBr => .modelled
  | .validBrIf => .modelled
  | .validThrowTag => .modelled
  | .validExprNil => .modelled
  | .validExprCons => .modelled
  | .validModule => .modelled
  | .validFunc => .modelled
  | .validMems => .modelled
  | .validGemmExport => .modelled
  | .validClosed => .modelled
  | .storeAlloc => .modelled
  | .storeLoadBytes => .modelled
  | .storeStoreBytes => .modelled
  | .storeMemoryGrow => .modelled
  | .execUnreachable => .modelled
  | .execNop => .modelled
  | .execI32Const => .modelled
  | .execDrop => .modelled
  | .execIBinOp => .modelled
  | .execIBinOpTrap => .modelled
  | .execITestOp => .modelled
  | .execIRelOp => .modelled
  | .execLocalGet => .modelled
  | .execLocalSet => .modelled
  | .execLocalTee => .modelled
  | .execGlobalGet => .modelled
  | .execGlobalSet => .modelled
  | .execLoad => .modelled
  | .execLoadTrap => .modelled
  | .execStore => .modelled
  | .execStoreTrap => .modelled
  | .execMemorySize => .modelled
  | .execMemoryGrowSucceed => .modelled
  | .execMemoryGrowRefuse => .modelled
  | .execBlock => .modelled
  | .execLoop => .modelled
  | .execIfFalse => .modelled
  | .execIfTrue => .modelled
  | .execBrLoop => .modelled
  | .execBrBlock => .modelled
  | .execBrIfFalse => .modelled
  | .execBrIfLoop => .modelled
  | .execBrIfBlock => .modelled
  | .execThrowTag => .modelled
  | .execExitLabel => .modelled
  | .execReturnGemm => .modelled
  | .execEnterGemm => .modelled
  | .execInstallTrap => .modelled
  | .trapUnreachable => .modelled
  | .trapOutOfBounds => .modelled
  | .trapDivideByZero => .modelled
  | .nondetMemoryGrow => .modelled
  | .nondetNanResult => .modelled
  | .tableGet => .modelledUnreachable
  | .tableSet => .modelledUnreachable
  | .tableSize => .modelledUnreachable
  | .tableGrow => .modelledUnreachable
  | .tableFill => .modelledUnreachable
  | .tableCopy => .modelledUnreachable
  | .tableInit => .modelledUnreachable
  | .tableElemDrop => .modelledUnreachable
  | .vectorExtractLane => .modelledUnreachable
  | .vectorReplaceLane => .modelledUnreachable
  | .vectorSplat => .modelledUnreachable
  | .vectorLanewise => .modelledUnreachable
  | .vectorShapeWidth => .modelledUnreachable
  | .refNull => .modelledUnreachable
  | .refFunc => .modelledUnreachable
  | .rejectMemory64Load => .rejectedByProfile
  | .rejectAtomicRmw => .rejectedByProfile
  | .rejectRelaxedFma => .rejectedByProfile
  | .rejectComponentModel => .rejectedByProfile

/-- The Lean declaration implementing the rule, relative to the
`WasmGemmGnaf.Wasm` namespace.  `none` is exactly a rejected family. -/
def leanDeclaration? : PinnedCoreRuleId → Option String
  | .decodeModule => some "decode"
  | .decodeUleb128 => some "decodeULEB"
  | .validUnreachable => some "InstrTyping.unreachable"
  | .validNop => some "InstrTyping.nop"
  | .validI32Const => some "InstrTyping.i32Const"
  | .validDrop => some "InstrTyping.drop"
  | .validIBinOp => some "InstrTyping.iBinOp"
  | .validITestOp => some "InstrTyping.iTestOp"
  | .validIRelOp => some "InstrTyping.iRelOp"
  | .validLocalGet => some "InstrTyping.localGet"
  | .validLocalSet => some "InstrTyping.localSet"
  | .validLocalTee => some "InstrTyping.localTee"
  | .validGlobalGet => some "InstrTyping.globalGet"
  | .validGlobalSet => some "InstrTyping.globalSet"
  | .validLoad => some "InstrTyping.load"
  | .validStore => some "InstrTyping.store"
  | .validMemorySize => some "InstrTyping.memorySize"
  | .validMemoryGrow => some "InstrTyping.memoryGrow"
  | .validBlock => some "InstrTyping.block"
  | .validLoop => some "InstrTyping.loop"
  | .validIfThenElse => some "InstrTyping.ifThenElse"
  | .validBr => some "InstrTyping.br"
  | .validBrIf => some "InstrTyping.brIf"
  | .validThrowTag => some "InstrTyping.throwTag"
  | .validExprNil => some "ExprTyping.nil"
  | .validExprCons => some "ExprTyping.cons"
  | .validModule => some "validate"
  | .validFunc => some "Module.checkFunc"
  | .validMems => some "Module.checkMems"
  | .validGemmExport => some "Module.checkGemmExport"
  | .validClosed => some "Module.checkClosed"
  | .storeAlloc => some "Store.alloc"
  | .storeLoadBytes => some "Store.loadBytes"
  | .storeStoreBytes => some "Store.storeBytes"
  | .storeMemoryGrow => some "Memory.grow"
  | .execUnreachable => some "Step.unreachable"
  | .execNop => some "Step.nop"
  | .execI32Const => some "Step.i32Const"
  | .execDrop => some "Step.drop"
  | .execIBinOp => some "Step.iBinOp"
  | .execIBinOpTrap => some "Step.iBinOpTrap"
  | .execITestOp => some "Step.iTestOp"
  | .execIRelOp => some "Step.iRelOp"
  | .execLocalGet => some "Step.localGet"
  | .execLocalSet => some "Step.localSet"
  | .execLocalTee => some "Step.localTee"
  | .execGlobalGet => some "Step.globalGet"
  | .execGlobalSet => some "Step.globalSet"
  | .execLoad => some "Step.load"
  | .execLoadTrap => some "Step.loadTrap"
  | .execStore => some "Step.store"
  | .execStoreTrap => some "Step.storeTrap"
  | .execMemorySize => some "Step.memorySize"
  | .execMemoryGrowSucceed => some "Step.memoryGrowSucceed"
  | .execMemoryGrowRefuse => some "Step.memoryGrowRefuse"
  | .execBlock => some "Step.block"
  | .execLoop => some "Step.loop"
  | .execIfFalse => some "Step.ifFalse"
  | .execIfTrue => some "Step.ifTrue"
  | .execBrLoop => some "Step.brLoop"
  | .execBrBlock => some "Step.brBlock"
  | .execBrIfFalse => some "Step.brIfFalse"
  | .execBrIfLoop => some "Step.brIfLoop"
  | .execBrIfBlock => some "Step.brIfBlock"
  | .execThrowTag => some "Step.throwTag"
  | .execExitLabel => some "Step.exitLabel"
  | .execReturnGemm => some "Step.returnGemm"
  | .execEnterGemm => some "Step.enterGemm"
  | .execInstallTrap => some "Step.installTrap"
  | .trapUnreachable => some "Trap.unreachable"
  | .trapOutOfBounds => some "Trap.outOfBounds"
  | .trapDivideByZero => some "Trap.divideByZero"
  | .nondetMemoryGrow => some "Num.growResults"
  | .nondetNanResult => some "Num.nanResults32"
  | .tableGet => some "TableInst.get"
  | .tableSet => some "TableInst.set"
  | .tableSize => some "TableInst.size"
  | .tableGrow => some "TableInst.grow"
  | .tableFill => some "TableInst.fill"
  | .tableCopy => some "TableInst.copy"
  | .tableInit => some "TableInst.init"
  | .tableElemDrop => some "ElemInst.dropSeg"
  | .vectorExtractLane => some "V128.extractLane"
  | .vectorReplaceLane => some "V128.replaceLane"
  | .vectorSplat => some "V128.splat"
  | .vectorLanewise => some "V128.zipLanes"
  | .vectorShapeWidth => some "VecShape.lanes_mul_laneWidth"
  | .refNull => some "Ref.null"
  | .refFunc => some "Ref.func"
  | .rejectMemory64Load => none
  | .rejectAtomicRmw => none
  | .rejectRelaxedFma => none
  | .rejectComponentModel => none

/-- A rule is enabled when the profile does not reject its family. -/
def RuleEnabled (id : PinnedCoreRuleId) : Prop := status id ≠ .rejectedByProfile

instance instDecidableRuleEnabled (id : PinnedCoreRuleId) :
    Decidable (RuleEnabled id) := by
  unfold RuleEnabled
  infer_instance

/-- The status agrees with the release feature matrix: a rule has no Lean
declaration exactly when its family is rejected by the profile. -/
theorem status_eq_rejected_iff (id : PinnedCoreRuleId) :
    status id = .rejectedByProfile ↔ Rejected (family id) := by
  cases id <;> decide

theorem ruleEnabled_iff_family_enabled (id : PinnedCoreRuleId) :
    RuleEnabled id ↔ Enabled (family id) := by
  cases id <;> decide

/-- **Totality over the enabled rule set.**  Every enabled identifier has a
mapped Lean declaration, and every rejected one has none. -/
theorem adequacy_map_total_on_enabled (id : PinnedCoreRuleId) :
    (leanDeclaration? id).isSome = true ↔ RuleEnabled id := by
  cases id <;> decide

theorem leanDeclaration?_eq_none_iff (id : PinnedCoreRuleId) :
    leanDeclaration? id = none ↔ Rejected (family id) := by
  cases id <;> decide

theorem exists_leanDeclaration_of_enabled {id : PinnedCoreRuleId}
    (h : RuleEnabled id) : ∃ name, leanDeclaration? id = some name := by
  have hs := (adequacy_map_total_on_enabled id).mpr h
  cases hd : leanDeclaration? id with
  | none => rw [hd] at hs; exact absurd hs (by simp)
  | some name => exact ⟨name, rfl⟩

/-- The enabled identifiers, in enumeration order. -/
def enabledRuleIds : List PinnedCoreRuleId :=
  [ .decodeModule
  , .decodeUleb128
  , .validUnreachable
  , .validNop
  , .validI32Const
  , .validDrop
  , .validIBinOp
  , .validITestOp
  , .validIRelOp
  , .validLocalGet
  , .validLocalSet
  , .validLocalTee
  , .validGlobalGet
  , .validGlobalSet
  , .validLoad
  , .validStore
  , .validMemorySize
  , .validMemoryGrow
  , .validBlock
  , .validLoop
  , .validIfThenElse
  , .validBr
  , .validBrIf
  , .validThrowTag
  , .validExprNil
  , .validExprCons
  , .validModule
  , .validFunc
  , .validMems
  , .validGemmExport
  , .validClosed
  , .storeAlloc
  , .storeLoadBytes
  , .storeStoreBytes
  , .storeMemoryGrow
  , .execUnreachable
  , .execNop
  , .execI32Const
  , .execDrop
  , .execIBinOp
  , .execIBinOpTrap
  , .execITestOp
  , .execIRelOp
  , .execLocalGet
  , .execLocalSet
  , .execLocalTee
  , .execGlobalGet
  , .execGlobalSet
  , .execLoad
  , .execLoadTrap
  , .execStore
  , .execStoreTrap
  , .execMemorySize
  , .execMemoryGrowSucceed
  , .execMemoryGrowRefuse
  , .execBlock
  , .execLoop
  , .execIfFalse
  , .execIfTrue
  , .execBrLoop
  , .execBrBlock
  , .execBrIfFalse
  , .execBrIfLoop
  , .execBrIfBlock
  , .execThrowTag
  , .execExitLabel
  , .execReturnGemm
  , .execEnterGemm
  , .execInstallTrap
  , .trapUnreachable
  , .trapOutOfBounds
  , .trapDivideByZero
  , .nondetMemoryGrow
  , .nondetNanResult
  , .tableGet
  , .tableSet
  , .tableSize
  , .tableGrow
  , .tableFill
  , .tableCopy
  , .tableInit
  , .tableElemDrop
  , .vectorExtractLane
  , .vectorReplaceLane
  , .vectorSplat
  , .vectorLanewise
  , .vectorShapeWidth
  , .refNull
  , .refFunc ]

theorem mem_enabledRuleIds_iff (id : PinnedCoreRuleId) :
    id ∈ enabledRuleIds ↔ RuleEnabled id := by cases id <;> decide

theorem enabledRuleIds_nodup : enabledRuleIds.Nodup := by decide

theorem enabledRuleIds_length : enabledRuleIds.length = 89 := rfl

theorem rejectedRuleIds_length :
    all.length - enabledRuleIds.length = 4 := rfl

/-- The mapped declaration name of an identifier, with the rejected ones
collapsed to the empty string.  Used only to state duplicate freedom of the
mapped names as a decidable computation. -/
def declarationOf (id : PinnedCoreRuleId) : String := (leanDeclaration? id).getD ""

/-- **The mapped Lean declarations are pairwise distinct.** -/
theorem enabledDeclarations_nodup :
    (enabledRuleIds.map declarationOf).Nodup := by decide

/-- **The identifier spellings are pairwise distinct.** -/
theorem ruleIds_nodup : (all.map ruleId).Nodup := by decide

/-- **Injectivity over the enabled rule set.**  Two enabled identifiers with the
same mapped Lean declaration are the same identifier: no declaration is claimed
by two rules. -/
theorem adequacy_map_injective_on_enabled {a b : PinnedCoreRuleId}
    (ha : RuleEnabled a) (hb : RuleEnabled b)
    (h : leanDeclaration? a = leanDeclaration? b) : a = b := by
  refine nodup_map_injOn declarationOf enabledRuleIds enabledDeclarations_nodup a b
    ((mem_enabledRuleIds_iff a).mpr ha) ((mem_enabledRuleIds_iff b).mpr hb) ?_
  unfold declarationOf
  rw [h]

/-- The identifier spelling is injective on the whole enumeration. -/
theorem ruleId_injective {a b : PinnedCoreRuleId} (h : ruleId a = ruleId b) :
    a = b :=
  nodup_map_injOn ruleId all ruleIds_nodup a b (mem_all a) (mem_all b) h

/-! ### Fully qualified names -/

/-- The namespace every mapped declaration lives in. -/
def declarationNamespace : String := "WasmGemmGnaf.Wasm."

/-- The fully qualified Lean declaration name of a rule. -/
def fullDeclaration? (id : PinnedCoreRuleId) : Option String :=
  (leanDeclaration? id).map (fun name => declarationNamespace ++ name)

theorem fullDeclaration?_isSome_iff (id : PinnedCoreRuleId) :
    (fullDeclaration? id).isSome = true ↔ RuleEnabled id := by
  unfold fullDeclaration?
  rw [← adequacy_map_total_on_enabled id]
  cases leanDeclaration? id <;> simp

/-- Injectivity survives qualification. -/
theorem fullDeclaration?_injective_on_enabled {a b : PinnedCoreRuleId}
    (ha : RuleEnabled a) (hb : RuleEnabled b)
    (h : fullDeclaration? a = fullDeclaration? b) : a = b := by
  obtain ⟨na, hna⟩ := exists_leanDeclaration_of_enabled ha
  obtain ⟨nb, hnb⟩ := exists_leanDeclaration_of_enabled hb
  unfold fullDeclaration? at h
  rw [hna, hnb] at h
  simp only [Option.map_some, Option.some.injEq] at h
  have : na = nb := string_append_left_cancel h
  exact adequacy_map_injective_on_enabled ha hb (by rw [hna, hnb, this])

end PinnedCoreRuleId

/-! ## The conformance map object -/

/-- One row of the conformance map: the pinned rule identifier, the Lean
declaration implementing it, its feature family, and its status. -/
structure AdequacyRow where
  /-- The vendored rule identifier spelling. -/
  ruleId : String
  /-- The Lean declaration, relative to the `WasmGemmGnaf.Wasm` namespace. -/
  leanDeclaration : String
  /-- The Core 3.0 feature family. -/
  family : FeatureFamily
  /-- What the released development does with the rule. -/
  status : RuleStatus
  deriving DecidableEq, Repr, Inhabited

/-- The conformance map: the pinned commit it is bound to, and its rows. -/
structure AdequacyMap where
  /-- The commit of the pinned WebAssembly Core revision. -/
  revisionCommit : String
  /-- One row per enabled pinned rule identifier. -/
  rows : List AdequacyRow
  deriving DecidableEq, Repr, Inhabited

/-- The row of an enabled rule identifier. -/
def PinnedCoreRuleId.row (id : PinnedCoreRuleId) : AdequacyRow :=
  { ruleId := id.ruleId
    leanDeclaration := id.declarationOf
    family := id.family
    status := id.status }

/-- The released conformance map: the pinned commit together with one row per
enabled identifier. -/
def core3AdequacyMap : AdequacyMap :=
  { revisionCommit := core3RevisionCommit
    rows := PinnedCoreRuleId.enabledRuleIds.map PinnedCoreRuleId.row }

/-- **Identity binding to the pinned revision.**  The map carries the pinned
`wg-3.0` commit. -/
theorem core3AdequacyMap_revisionCommit :
    core3AdequacyMap.revisionCommit = core3Revision.commit := rfl

/-- **The model and the map are bound to the same revision.**  Every lawful
profile carries the commit the map carries; a profile pinned to a different
revision cannot satisfy this. -/
theorem adequacy_profile_revision_agree (profile : Profile) :
    profile.body.revisionCommit = core3AdequacyMap.revisionCommit :=
  profile.revisionCommit_eq

/-- Distinct revisions have distinct canonical identities, so the binding above
is a real one. -/
theorem adequacy_revision_identity_eq_iff (r : RevisionBody) :
    RevisionBody.identity r = RevisionBody.identity core3Revision ↔
      r = core3Revision :=
  RevisionBody.identity_eq_iff

theorem core3AdequacyMap_rows_length :
    core3AdequacyMap.rows.length = PinnedCoreRuleId.enabledRuleIds.length := by
  simp [core3AdequacyMap]

/-- Every enabled identifier has a row. -/
theorem core3AdequacyMap_covers {id : PinnedCoreRuleId}
    (h : PinnedCoreRuleId.RuleEnabled id) : id.row ∈ core3AdequacyMap.rows :=
  List.mem_map_of_mem ((PinnedCoreRuleId.mem_enabledRuleIds_iff id).mpr h)

/-- No rejected identifier has a row: a rejected family contributes no Lean
declaration to the map. -/
theorem core3AdequacyMap_excludes_rejected {id : PinnedCoreRuleId}
    (h : ¬ PinnedCoreRuleId.RuleEnabled id) :
    id.ruleId ∉ core3AdequacyMap.rows.map AdequacyRow.ruleId := by
  intro hmem
  simp only [core3AdequacyMap, List.map_map, List.mem_map] at hmem
  obtain ⟨other, hother, heq⟩ := hmem
  have : other = id := PinnedCoreRuleId.ruleId_injective heq
  subst this
  exact h ((PinnedCoreRuleId.mem_enabledRuleIds_iff other).mp hother)

/-- **The map is duplicate free in its identifiers.** -/
theorem core3AdequacyMap_ruleIds_nodup :
    (core3AdequacyMap.rows.map AdequacyRow.ruleId).Nodup := by decide

/-- **The map is duplicate free in its Lean declarations**: no Lean declaration
is claimed by two pinned rules. -/
theorem core3AdequacyMap_declarations_nodup :
    (core3AdequacyMap.rows.map AdequacyRow.leanDeclaration).Nodup := by decide

/-- Every row of the map records a status that the release feature matrix
admits: no row of the map belongs to a rejected family. -/
theorem core3AdequacyMap_rows_enabled {row : AdequacyRow}
    (h : row ∈ core3AdequacyMap.rows) : Enabled row.family := by
  simp only [core3AdequacyMap, List.mem_map] at h
  obtain ⟨id, hid, rfl⟩ := h
  exact (PinnedCoreRuleId.ruleEnabled_iff_family_enabled id).mp
    ((PinnedCoreRuleId.mem_enabledRuleIds_iff id).mp hid)

/-! ## Non-vacuity: the mapped declarations exist

Each mapped Lean declaration is mentioned below, so deleting or renaming one
breaks this file.  As disclosed in the header, this establishes that a
declaration with each mapped *role* exists; it does not establish that the
recorded string is that declaration's name, which Lean cannot check without
metaprogramming. -/

theorem mapped_declarations_referenced : True := by
  have _ := @decode
  have _ := @decodeULEB
  have _ := @InstrTyping.unreachable
  have _ := @InstrTyping.nop
  have _ := @InstrTyping.i32Const
  have _ := @InstrTyping.drop
  have _ := @InstrTyping.iBinOp
  have _ := @InstrTyping.iTestOp
  have _ := @InstrTyping.iRelOp
  have _ := @InstrTyping.localGet
  have _ := @InstrTyping.localSet
  have _ := @InstrTyping.localTee
  have _ := @InstrTyping.globalGet
  have _ := @InstrTyping.globalSet
  have _ := @InstrTyping.load
  have _ := @InstrTyping.store
  have _ := @InstrTyping.memorySize
  have _ := @InstrTyping.memoryGrow
  have _ := @InstrTyping.block
  have _ := @InstrTyping.loop
  have _ := @InstrTyping.ifThenElse
  have _ := @InstrTyping.br
  have _ := @InstrTyping.brIf
  have _ := @InstrTyping.throwTag
  have _ := @ExprTyping.nil
  have _ := @ExprTyping.cons
  have _ := @validate
  have _ := @WasmGemmGnaf.Wasm.Module.checkFunc
  have _ := @WasmGemmGnaf.Wasm.Module.checkMems
  have _ := @WasmGemmGnaf.Wasm.Module.checkGemmExport
  have _ := @WasmGemmGnaf.Wasm.Module.checkClosed
  have _ := @Store.alloc
  have _ := @Store.loadBytes
  have _ := @Store.storeBytes
  have _ := @Memory.grow
  have _ := @Step.unreachable
  have _ := @Step.nop
  have _ := @Step.i32Const
  have _ := @Step.drop
  have _ := @Step.iBinOp
  have _ := @Step.iBinOpTrap
  have _ := @Step.iTestOp
  have _ := @Step.iRelOp
  have _ := @Step.localGet
  have _ := @Step.localSet
  have _ := @Step.localTee
  have _ := @Step.globalGet
  have _ := @Step.globalSet
  have _ := @Step.load
  have _ := @Step.loadTrap
  have _ := @Step.store
  have _ := @Step.storeTrap
  have _ := @Step.memorySize
  have _ := @Step.memoryGrowSucceed
  have _ := @Step.memoryGrowRefuse
  have _ := @Step.block
  have _ := @Step.loop
  have _ := @Step.ifFalse
  have _ := @Step.ifTrue
  have _ := @Step.brLoop
  have _ := @Step.brBlock
  have _ := @Step.brIfFalse
  have _ := @Step.brIfLoop
  have _ := @Step.brIfBlock
  have _ := @Step.throwTag
  have _ := @Step.exitLabel
  have _ := @Step.returnGemm
  have _ := @Step.enterGemm
  have _ := @Step.installTrap
  have _ := @Trap.unreachable
  have _ := @Trap.outOfBounds
  have _ := @Trap.divideByZero
  have _ := @Num.growResults
  have _ := @Num.nanResults32
  have _ := @TableInst.get
  have _ := @TableInst.set
  have _ := @TableInst.size
  have _ := @TableInst.grow
  have _ := @TableInst.fill
  have _ := @TableInst.copy
  have _ := @TableInst.init
  have _ := @ElemInst.dropSeg
  have _ := @V128.extractLane
  have _ := @V128.replaceLane
  have _ := @V128.splat
  have _ := @V128.zipLanes
  have _ := @VecShape.lanes_mul_laneWidth
  have _ := @Ref.null
  have _ := @Ref.func
  trivial

/-! ## Scope of the `modelledUnreachable` status

`Wasm/Table.lean` proves that the released validator rejects every table
instruction, that a validating module declares no table or element segment at
all, and that `Wasm/Step.lean` enumerates no successor for a table instruction.
The same holds of the vector instructions, and is proved here because this is
the first module importing both `Wasm/Vector.lean` and `Wasm/Step.lean`.

Together these are the machine-checked meaning of `RuleStatus.modelledUnreachable`:
the mapped declaration exists and its laws are proved, but no released execution
reaches it. -/

/-- The released validator types no vector instruction. -/
theorem checkInstr_vector_rejected (C : Ctx) (h : Nat) (s : VecShape)
    (lane : Nat) (lo hi : UInt64) (arg : MemArg) (lanes : List Nat)
    (u : VecUnOp) (b : VecBinOp) (r : VecRelOp) (ext : Option SignExt) :
    checkInstr C h (.vecConst lo hi) = none ∧
    checkInstr C h (.vecUnOp s u) = none ∧
    checkInstr C h (.vecBinOp s b) = none ∧
    checkInstr C h (.vecRelOp s r) = none ∧
    checkInstr C h .vecBitselect = none ∧
    checkInstr C h (.vecSplat s) = none ∧
    checkInstr C h (.vecExtractLane s lane ext) = none ∧
    checkInstr C h (.vecReplaceLane s lane) = none ∧
    checkInstr C h (.vecShuffle lanes) = none ∧
    checkInstr C h (.vecLoad arg) = none ∧
    checkInstr C h (.vecStore arg) = none :=
  ⟨rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl⟩

/-- The released reduction relation enumerates no successor for a vector
instruction. -/
theorem successorsOfInstr_vector_empty (c : Config) (rest : List Instr)
    (s : VecShape) (lane : Nat) (lo hi : UInt64) (arg : MemArg)
    (lanes : List Nat) (u : VecUnOp) (b : VecBinOp) (r : VecRelOp)
    (ext : Option SignExt) :
    successorsOfInstr c (.vecConst lo hi) rest = [] ∧
    successorsOfInstr c (.vecUnOp s u) rest = [] ∧
    successorsOfInstr c (.vecBinOp s b) rest = [] ∧
    successorsOfInstr c (.vecRelOp s r) rest = [] ∧
    successorsOfInstr c .vecBitselect rest = [] ∧
    successorsOfInstr c (.vecSplat s) rest = [] ∧
    successorsOfInstr c (.vecExtractLane s lane ext) rest = [] ∧
    successorsOfInstr c (.vecReplaceLane s lane) rest = [] ∧
    successorsOfInstr c (.vecShuffle lanes) rest = [] ∧
    successorsOfInstr c (.vecLoad arg) rest = [] ∧
    successorsOfInstr c (.vecStore arg) rest = [] :=
  ⟨rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl, rfl⟩

end WasmGemmGnaf.Wasm
