/-
  Wasm/Costed.lean --- the costed semantics of SPEC section 7.5.

  Normative source: SPEC.md section 7.5, which requires that every semantic
  transition be labelled with exact abstract cost events, and fixes the total
  contribution law:

      every relational Core `Step` contributes one `wasmRuleSteps` unit;
      `dispatchSteps` additionally counts exactly branch, branch-table, direct
      or indirect call, return, exception transfer, tail-call transfer, and
      harness phase-transition rule identifiers; `scalarOps` is the number of
      scalar primitive numeric operations performed by the rule; `vectorLaneOps`
      is the exact number of active 128-bit lanes operated on, with a
      whole-vector shuffle contributing 16 byte lanes; each
      load/store/table/data/memory event contributes its exact accessed or
      allocated byte, page, or element count; failed or trapping accesses
      contribute the attempted rule step but no completed transfer; ...
      `peakStackValues`, `peakPages`, and every other peak are maxima over the
      complete costed configuration sequence, while all nonpeak coordinates are
      sums.

  Three design decisions carry the anti-cheat weight of this file.

  * A costed event is *computed from the source configuration*, never supplied.
    `label : Config → Event → Option CostedEvent` reads the pending instruction,
    the control stack, the operand stack and the harness descriptor, and
    `CostedStep c ce c'` holds only when `label c ce.erase = some ce`.  A
    competitor therefore cannot pair a real transition with a cheaper label:
    the installed-byte count of the harness entry rule, for instance, is
    `c.harness.rawBytes.length` by construction.  `label` is `Option`-valued
    with `none` on every combination it does not recognise, so a reduction rule
    added later without a cost row has *no* costed step at all rather than a
    silently free one.

  * `EventContribution` is an inductive relation with exactly one constructor
    per `CostedEvent` constructor and **no default arm**, so
    `wasm_cost_table_total` is a real totality-and-exclusivity statement: adding
    a costed event without deciding its charge breaks the proof.

  * The three peak coordinates are contributed by the *configuration snapshots*
    only, and the snapshots are pinned structurally by `CostedReduces` to the
    configurations actually visited.  A costed trace cannot understate a peak.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Run
import WasmGemmGnaf.Wasm.Profile
import WasmGemmGnaf.Cost.Aggregate

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Event determinism

The costed layer needs one property of the plain machine that `Wasm/Step.lean`
does not state: a configuration and an event determine the successor.  It is
read off the successor enumerator, whose *keys* are duplicate free —
`memory.grow` really does branch, but its two successors carry the two distinct
outcome labels. -/

theorem eq_of_key_nodup {α β : Type} :
    ∀ {l : List (α × β)} {a : α} {b₁ b₂ : β},
      (l.map Prod.fst).Nodup → (a, b₁) ∈ l → (a, b₂) ∈ l → b₁ = b₂ := by
  intro l
  induction l with
  | nil => intro a b₁ b₂ _ h₁ _; exact absurd h₁ (by simp)
  | cons hd tl ih =>
    intro a b₁ b₂ hnd h₁ h₂
    rw [List.map_cons, List.nodup_cons] at hnd
    have key : ∀ {b : β}, (a, b) = hd → hd.1 = a := by
      intro b hb; rw [← hb]
    have tailMem : ∀ {b : β}, (a, b) ∈ tl → a ∈ tl.map Prod.fst := by
      intro b hb
      have := List.mem_map_of_mem (f := (Prod.fst : α × β → α)) hb
      simpa using this
    rcases List.mem_cons.mp h₁ with e₁ | m₁
    · rcases List.mem_cons.mp h₂ with e₂ | m₂
      · exact (Prod.mk.inj (e₁.trans e₂.symm)).2
      · exact absurd (key e₁ ▸ tailMem m₂) hnd.1
    · rcases List.mem_cons.mp h₂ with e₂ | m₂
      · exact absurd (key e₂ ▸ tailMem m₁) hnd.1
      · exact ih hnd.2 m₁ m₂

theorem branchTo_keys_nodup (c : Config) (st : List UInt32) (n : Nat) :
    ((branchTo c st n).map Prod.fst).Nodup := by
  unfold branchTo
  split
  · simp
  · split <;> simp

theorem successorsAtEnd_keys_nodup (c : Config) :
    ((successorsAtEnd c).map Prod.fst).Nodup := by
  unfold successorsAtEnd
  repeat' split
  all_goals simp

theorem successorsOfInstr_keys_nodup (c : Config) (i : Instr) (rest : List Instr) :
    ((successorsOfInstr c i rest).map Prod.fst).Nodup := by
  unfold successorsOfInstr
  repeat' split
  all_goals simp [branchTo_keys_nodup]

/-- The successor enumeration is keyed by its event: a configuration and an
event determine at most one successor. -/
theorem successors_keys_nodup (c : Config) :
    ((successors c).map Prod.fst).Nodup := by
  unfold successors
  split
  · split
    · exact successorsAtEnd_keys_nodup c
    · exact successorsOfInstr_keys_nodup c _ _
  · simp

/-- **Event determinism.**  The permitted nondeterminism of the released
profile is recorded entirely in the event label, so a configuration together
with its event determines the successor configuration. -/
theorem Step.deterministic {c c₁ c₂ : Config} {e : Event}
    (h₁ : Step c e c₁) (h₂ : Step c e c₂) : c₁ = c₂ :=
  eq_of_key_nodup (successors_keys_nodup c)
    ((mem_successors_iff_step c e c₁).mpr h₁)
    ((mem_successors_iff_step c e c₂).mpr h₂)

/-- Consequently a reduction sequence is determined by its trace. -/
theorem Reduces.deterministic {c a b : Config} {tr : List Event}
    (h₁ : Reduces c tr a) (h₂ : Reduces c tr b) : a = b := by
  induction h₁ generalizing b with
  | refl _ => cases h₂ with
    | refl _ => rfl
  | @cons c e c' tr' c'' hs hr ih =>
    cases h₂ with
    | @cons _ _ d' _ _ hs₂ hr₂ =>
      have hd : c' = d' := Step.deterministic hs hs₂
      subst hd
      exact ih hr₂

/-! ## Configuration resource snapshots

SPEC section 7.5: "`ConfigResourceSnapshot` contains total live value slots
across every operand stack, frame-local vector, label/handler payload, and
exception payload; ordinary-memory pages; table size; live managed-GC/exception
bytes". -/

/-- The tag type of the exception values of the declared subset: exactly one
`i32` payload word, which is `ExceptionValue.payload`. -/
def exceptionValueTagType : TagType :=
  { funcType := { params := [ValType.num .i32], results := [] } }

/-- The abstract cost width of one live exception object, from the pinned
canonical exception header, payload layout and total rounding. -/
def exceptionObjectBytes (L : GcLayoutConstants) : Nat :=
  L.exceptionSize exceptionValueTagType

/-- Under the canonical release widths an exception object costs a 16-byte
header plus one `i32` payload word, rounded to a multiple of eight: 24 abstract
bytes.  It is not a unit-cost free store. -/
theorem canonical_exceptionObjectBytes :
    exceptionObjectBytes canonicalGcLayout = 24 := by decide

namespace CtrlEntry

/-- The live value slots held by one label of the control stack.

**Scope.**  Every label admitted by the reduction relation is pushed by
`Step.block`, `Step.loop`, `Step.ifTrue` or `Step.ifFalse`, each of which
requires block type `.empty`; a label of the declared subset therefore holds no
operand values of its own.  The values it will resume with are exactly the ones
at or below its recorded `height` in the single operand stack `Config.stack`,
already counted once by `Config.operandSlots`; charging them again here would
double count them.  This does **not** establish that a label with a nonempty
result type is free: it records that the declared subset has none. -/
def payloadSlots (_k : CtrlEntry) : Nat := 0

theorem payloadSlots_eq_zero (k : CtrlEntry) : payloadSlots k = 0 := rfl

end CtrlEntry

namespace Config

/-- Live value slots on the operand stack. -/
def operandSlots (c : Config) : Nat := c.stack.length

/-- Live value slots in the frame-local vector. -/
def localSlots (c : Config) : Nat := c.locals.length

/-- Live value slots held by the label/handler stack. -/
def labelPayloadSlots (c : Config) : Nat :=
  (c.ctrl.map CtrlEntry.payloadSlots).sum

/-- Live value slots held by an exception payload.  An uncaught exception of
the declared subset carries exactly one `i32` word. -/
def exceptionPayloadSlots (c : Config) : Nat :=
  match c.status with
  | .thrown _ => 1
  | _ => 0

/-- The number of live managed exception objects. -/
def liveExceptionObjects (c : Config) : Nat := c.exceptionPayloadSlots

/-- SPEC section 7.5: total live value slots across every operand stack, frame
local vector, label/handler payload and exception payload. -/
def liveValueSlots (c : Config) : Nat :=
  c.operandSlots + c.localSlots + c.labelPayloadSlots + c.exceptionPayloadSlots

/-- The live ordinary-memory pages of a configuration. -/
def livePages (c : Config) : Nat := c.store.memory.pages

/-- The live table size of a configuration.

**Scope.**  The released `Store` has exactly two components, memory zero and
the globals (`Wasm/Store.lean`): the profile admits no table instance, so the
live table size of every reachable configuration is exactly zero.  This does
**not** establish that table operations are free; it records that the released
subset performs none, and `tableElementsAllocated` remains a summed coordinate
of the cost vector for the rules that do. -/
def tableSize (_c : Config) : Nat := 0

theorem tableSize_eq_zero (c : Config) : c.tableSize = 0 := rfl

/-- Live managed-GC and exception bytes of a configuration.

**Scope.**  The declared subset has no `struct.new`, `array.new` or
`array.new_default` reduction rule, so the only managed object a reachable
configuration can hold live is the uncaught exception object of a `thrown`
status.  This does **not** establish that GC allocation is free: the canonical
widths are applied here exactly as `gcAllocationContribution` applies them. -/
def liveGcBytes (L : GcLayoutConstants) (c : Config) : Nat :=
  c.liveExceptionObjects * exceptionObjectBytes L

theorem liveGcBytes_thrown (L : GcLayoutConstants) (c : Config)
    {ev : ExceptionValue} (h : c.status = .thrown ev) :
    c.liveGcBytes L = exceptionObjectBytes L := by
  simp [liveGcBytes, liveExceptionObjects, exceptionPayloadSlots, h]

theorem operandSlots_le_liveValueSlots (c : Config) :
    c.operandSlots ≤ c.liveValueSlots := by
  simp only [liveValueSlots]; omega

theorem localSlots_le_liveValueSlots (c : Config) :
    c.localSlots ≤ c.liveValueSlots := by
  simp only [liveValueSlots]; omega

end Config

/-- SPEC section 7.5: the peak inputs of one visited configuration. -/
structure ConfigResourceSnapshot where
  /-- Total live value slots across operand stacks, frame locals, label and
  handler payloads and exception payloads. -/
  liveValueSlots : Nat
  /-- Ordinary-memory pages. -/
  memoryPages : Nat
  /-- Table size. -/
  tableSize : Nat
  /-- Live managed-GC and exception-object bytes. -/
  liveGcBytes : Nat
  deriving DecidableEq, Repr, Inhabited

/-- The exact resource snapshot of a configuration. -/
def snapshotOf (L : GcLayoutConstants) (c : Config) : ConfigResourceSnapshot :=
  { liveValueSlots := c.liveValueSlots
    memoryPages := c.livePages
    tableSize := c.tableSize
    liveGcBytes := c.liveGcBytes L }

/-! ## Rule identifiers -/

/-- The rule identifier of every reduction rule of the released machine: one
constructor per constructor of `Wasm.Step`. -/
inductive RuleId
  | unreachable | nop | i32Const | drop | iBinOp | iBinOpTrap | iTestOp
  | iRelOp | localGet | localSet | localTee | globalGet | globalSet
  | load | loadTrap | store | storeTrap | memorySize
  | memoryGrowSucceed | memoryGrowRefuse | block | loop | ifFalse | ifTrue
  | brLoop | brBlock | brIfFalse | brIfLoop | brIfBlock | throwTag
  | exitLabel | returnGemm | enterGemm | installTrap
  deriving DecidableEq, Repr, Inhabited

namespace RuleId

/-- The finite cover of the rule identifiers. -/
def all : List RuleId :=
  [ .unreachable, .nop, .i32Const, .drop, .iBinOp, .iBinOpTrap, .iTestOp
  , .iRelOp, .localGet, .localSet, .localTee, .globalGet, .globalSet
  , .load, .loadTrap, .store, .storeTrap, .memorySize
  , .memoryGrowSucceed, .memoryGrowRefuse, .block, .loop, .ifFalse, .ifTrue
  , .brLoop, .brBlock, .brIfFalse, .brIfLoop, .brIfBlock, .throwTag
  , .exitLabel, .returnGemm, .enterGemm, .installTrap ]

theorem mem_all (r : RuleId) : r ∈ all := by cases r <;> simp [all]

theorem all_nodup : all.Nodup := by decide

theorem all_length : all.length = 34 := rfl

end RuleId

/-! ## Costed events

A costed event is the exact abstract cost label of one semantic transition.  It
carries the rule that fired together with the data the contribution law needs
and the data required to reconstruct the plain event it erases to, and nothing
else. -/

/-- SPEC section 7.5: the exact abstract cost label of one transition. -/
inductive CostedEvent
  /-- `unreachable`. -/
  | unreachableTrap
  /-- `nop`. -/
  | nop
  /-- `i32.const`. -/
  | i32Const
  /-- `drop`. -/
  | drop
  /-- A completed `i32` binary operation: one scalar primitive numeric op. -/
  | iBinOp
  /-- A trapping `i32.div_u`/`i32.rem_u`: the attempted rule step only. -/
  | iBinOpTrap
  /-- `i32.eqz`. -/
  | iTestOp
  /-- An `i32` comparison. -/
  | iRelOp
  | localGet
  | localSet
  | localTee
  | globalGet
  | globalSet
  /-- A completed `i32.load`: four bytes read. -/
  | load
  /-- A trapping `i32.load`: the attempted rule step, no completed transfer. -/
  | loadTrap
  /-- A completed `i32.store`: four bytes written. -/
  | store
  /-- A trapping `i32.store`: the attempted rule step, no completed transfer. -/
  | storeTrap
  | memorySize
  /-- A successful `memory.grow` of `deltaPages` pages from `previousPages`. -/
  | memoryGrowSucceed (deltaPages : Nat) (previousPages : Nat)
  /-- A refused `memory.grow`: no completed allocation. -/
  | memoryGrowRefuse
  | blockEnter
  | loopEnter
  /-- The `else` arm of an `if`: a dispatch step. -/
  | ifFalse
  /-- The `then` arm of an `if`: a dispatch step. -/
  | ifTrue
  /-- A taken `br` to a `loop` label: a dispatch step. -/
  | brLoop
  /-- A taken `br` to a `block` label: a dispatch step. -/
  | brBlock
  /-- An untaken `br_if`: an ordinary rule step. -/
  | brIfFalse
  /-- A taken `br_if` to a `loop` label: a dispatch step. -/
  | brIfLoop
  /-- A taken `br_if` to a `block` label: a dispatch step. -/
  | brIfBlock
  /-- An exception transfer: a dispatch step that allocates one exception
  object of the canonical abstract width. -/
  | throwTag (exceptionValue : ExceptionValue)
  | exitLabel
  /-- The terminal return of the exported `gemm`. -/
  | returnGemm (value : UInt32)
  /-- The harness phase transition: one preparation step and one written byte
  per installed raw byte. -/
  | enterGemm (installedBytes : Nat)
  /-- A trapping raw installation: the attempted rule step, no completed
  transfer. -/
  | installTrap
  deriving DecidableEq, Repr, Inhabited

namespace CostedEvent

/-- The rule that fired. -/
def rule : CostedEvent → RuleId
  | .unreachableTrap => .unreachable
  | .nop => .nop
  | .i32Const => .i32Const
  | .drop => .drop
  | .iBinOp => .iBinOp
  | .iBinOpTrap => .iBinOpTrap
  | .iTestOp => .iTestOp
  | .iRelOp => .iRelOp
  | .localGet => .localGet
  | .localSet => .localSet
  | .localTee => .localTee
  | .globalGet => .globalGet
  | .globalSet => .globalSet
  | .load => .load
  | .loadTrap => .loadTrap
  | .store => .store
  | .storeTrap => .storeTrap
  | .memorySize => .memorySize
  | .memoryGrowSucceed _ _ => .memoryGrowSucceed
  | .memoryGrowRefuse => .memoryGrowRefuse
  | .blockEnter => .block
  | .loopEnter => .loop
  | .ifFalse => .ifFalse
  | .ifTrue => .ifTrue
  | .brLoop => .brLoop
  | .brBlock => .brBlock
  | .brIfFalse => .brIfFalse
  | .brIfLoop => .brIfLoop
  | .brIfBlock => .brIfBlock
  | .throwTag _ => .throwTag
  | .exitLabel => .exitLabel
  | .returnGemm _ => .returnGemm
  | .enterGemm _ => .enterGemm
  | .installTrap => .installTrap

/-- Every rule identifier is the identifier of some costed event: the labelling
covers the whole reduction relation. -/
theorem rule_surjective (r : RuleId) : ∃ ce : CostedEvent, ce.rule = r := by
  cases r
  case unreachable => exact ⟨.unreachableTrap, rfl⟩
  case nop => exact ⟨.nop, rfl⟩
  case i32Const => exact ⟨.i32Const, rfl⟩
  case drop => exact ⟨.drop, rfl⟩
  case iBinOp => exact ⟨.iBinOp, rfl⟩
  case iBinOpTrap => exact ⟨.iBinOpTrap, rfl⟩
  case iTestOp => exact ⟨.iTestOp, rfl⟩
  case iRelOp => exact ⟨.iRelOp, rfl⟩
  case localGet => exact ⟨.localGet, rfl⟩
  case localSet => exact ⟨.localSet, rfl⟩
  case localTee => exact ⟨.localTee, rfl⟩
  case globalGet => exact ⟨.globalGet, rfl⟩
  case globalSet => exact ⟨.globalSet, rfl⟩
  case load => exact ⟨.load, rfl⟩
  case loadTrap => exact ⟨.loadTrap, rfl⟩
  case store => exact ⟨.store, rfl⟩
  case storeTrap => exact ⟨.storeTrap, rfl⟩
  case memorySize => exact ⟨.memorySize, rfl⟩
  case memoryGrowSucceed => exact ⟨.memoryGrowSucceed 0 0, rfl⟩
  case memoryGrowRefuse => exact ⟨.memoryGrowRefuse, rfl⟩
  case block => exact ⟨.blockEnter, rfl⟩
  case loop => exact ⟨.loopEnter, rfl⟩
  case ifFalse => exact ⟨.ifFalse, rfl⟩
  case ifTrue => exact ⟨.ifTrue, rfl⟩
  case brLoop => exact ⟨.brLoop, rfl⟩
  case brBlock => exact ⟨.brBlock, rfl⟩
  case brIfFalse => exact ⟨.brIfFalse, rfl⟩
  case brIfLoop => exact ⟨.brIfLoop, rfl⟩
  case brIfBlock => exact ⟨.brIfBlock, rfl⟩
  case throwTag => exact ⟨.throwTag default, rfl⟩
  case exitLabel => exact ⟨.exitLabel, rfl⟩
  case returnGemm => exact ⟨.returnGemm 0, rfl⟩
  case enterGemm => exact ⟨.enterGemm 0, rfl⟩
  case installTrap => exact ⟨.installTrap, rfl⟩

/-- **Erasure of one label.**  The plain event the costed event carries. -/
def erase : CostedEvent → Event
  | .unreachableTrap => .trapEvent .unreachable
  | .nop => .step
  | .i32Const => .step
  | .drop => .step
  | .iBinOp => .step
  | .iBinOpTrap => .trapEvent .divideByZero
  | .iTestOp => .step
  | .iRelOp => .step
  | .localGet => .step
  | .localSet => .step
  | .localTee => .step
  | .globalGet => .step
  | .globalSet => .step
  | .load => .step
  | .loadTrap => .trapEvent .outOfBounds
  | .store => .step
  | .storeTrap => .trapEvent .outOfBounds
  | .memorySize => .step
  | .memoryGrowSucceed _ previousPages => .growAttempt (.grown previousPages)
  | .memoryGrowRefuse => .growAttempt .refused
  | .blockEnter => .step
  | .loopEnter => .step
  | .ifFalse => .branch
  | .ifTrue => .branch
  | .brLoop => .branch
  | .brBlock => .branch
  | .brIfFalse => .step
  | .brIfLoop => .branch
  | .brIfBlock => .branch
  | .throwTag ev => .throwEvent ev
  | .exitLabel => .step
  | .returnGemm v => .returnEvent v
  | .enterGemm _ => .enterGemm
  | .installTrap => .trapEvent .outOfBounds

end CostedEvent

/-- **Erasure of a costed trace.** -/
def eraseCosts (costedTrace : List CostedEvent) : List Event :=
  costedTrace.map CostedEvent.erase

@[simp] theorem eraseCosts_nil : eraseCosts [] = [] := rfl

@[simp] theorem eraseCosts_cons (ce : CostedEvent) (ct : List CostedEvent) :
    eraseCosts (ce :: ct) = ce.erase :: eraseCosts ct := rfl

theorem eraseCosts_append (a b : List CostedEvent) :
    eraseCosts (a ++ b) = eraseCosts a ++ eraseCosts b := by
  simp [eraseCosts]

@[simp] theorem eraseCosts_length (ct : List CostedEvent) :
    (eraseCosts ct).length = ct.length := List.length_map _

/-! ## The exact label of a transition

`label` is a *function of the configuration and the event*: nothing in it is
supplied by the object being measured.  It is `Option`-valued and returns `none`
on every combination it does not recognise, so an unrecognised transition has no
costed step at all. -/

/-- The exact cost label of a transition out of a configuration whose
instruction sequence is exhausted. -/
def labelAtEnd (c : Config) (e : Event) : Option CostedEvent :=
  match c.ctrl with
  | _ :: _ => match e with
      | .step => some .exitLabel
      | _ => none
  | [] => match e with
      | .enterGemm => some (.enterGemm c.harness.rawBytes.length)
      | .returnEvent v => some (.returnGemm v)
      | .trapEvent .outOfBounds => some .installTrap
      | _ => none

/-- The exact cost label of a transition whose pending instruction is `i`. -/
def labelOfInstr (c : Config) (i : Instr) (e : Event) : Option CostedEvent :=
  match i with
  | .unreachable => match e with
      | .trapEvent .unreachable => some .unreachableTrap
      | _ => none
  | .nop => match e with | .step => some .nop | _ => none
  | .i32Const _ => match e with | .step => some .i32Const | _ => none
  | .drop => match e with | .step => some .drop | _ => none
  | .iBinOp .i32 _ => match e with
      | .step => some .iBinOp
      | .trapEvent .divideByZero => some .iBinOpTrap
      | _ => none
  | .iTestOp .i32 .eqz => match e with | .step => some .iTestOp | _ => none
  | .iRelOp .i32 _ => match e with | .step => some .iRelOp | _ => none
  | .localGet _ => match e with | .step => some .localGet | _ => none
  | .localSet _ => match e with | .step => some .localSet | _ => none
  | .localTee _ => match e with | .step => some .localTee | _ => none
  | .globalGet _ => match e with | .step => some .globalGet | _ => none
  | .globalSet _ => match e with | .step => some .globalSet | _ => none
  | .load .i32 none _ => match e with
      | .step => some .load
      | .trapEvent .outOfBounds => some .loadTrap
      | _ => none
  | .store .i32 none _ => match e with
      | .step => some .store
      | .trapEvent .outOfBounds => some .storeTrap
      | _ => none
  | .memorySize 0 => match e with | .step => some .memorySize | _ => none
  | .memoryGrow 0 => match e with
      | .growAttempt (.grown previousPages) =>
          some (.memoryGrowSucceed (c.stack.headD 0).toNat previousPages)
      | .growAttempt .refused => some .memoryGrowRefuse
      | _ => none
  | .block .empty _ => match e with | .step => some .blockEnter | _ => none
  | .loop .empty _ => match e with | .step => some .loopEnter | _ => none
  | .ifThenElse .empty _ _ => match e with
      | .branch => if c.stack.headD 0 = 0 then some .ifFalse else some .ifTrue
      | _ => none
  | .br n => match e with
      | .branch => match c.ctrl[n]? with
          | some k => if k.isLoop = true then some .brLoop else some .brBlock
          | none => none
      | _ => none
  | .brIf n => match e with
      | .step => some .brIfFalse
      | .branch => match c.ctrl[n]? with
          | some k => if k.isLoop = true then some .brIfLoop else some .brIfBlock
          | none => none
      | _ => none
  | .throw _ => match e with
      | .throwEvent ev => some (.throwTag ev)
      | _ => none
  | _ => none

/-- **SPEC section 7.5.**  The exact abstract cost label of the transition of
`c` under event `e`, computed from the configuration. -/
def label (c : Config) (e : Event) : Option CostedEvent :=
  match c.code with
  | [] => labelAtEnd c e
  | i :: _ => labelOfInstr c i e

/-- Every reduction has a label, and that label erases back to its event. -/
theorem step_label {c c' : Config} {e : Event} (h : Step c e c') :
    ∃ ce : CostedEvent, label c e = some ce ∧ ce.erase = e := by
  cases h with
  | unreachable hs hc =>
      exact ⟨.unreachableTrap, by simp [label, hc, labelOfInstr], rfl⟩
  | nop hs hc => exact ⟨.nop, by simp [label, hc, labelOfInstr], rfl⟩
  | i32Const hs hc => exact ⟨.i32Const, by simp [label, hc, labelOfInstr], rfl⟩
  | drop hs hc hst => exact ⟨.drop, by simp [label, hc, labelOfInstr], rfl⟩
  | iBinOp hs hc hst hv => exact ⟨.iBinOp, by simp [label, hc, labelOfInstr], rfl⟩
  | iBinOpTrap hs hc hst hv =>
      exact ⟨.iBinOpTrap, by simp [label, hc, labelOfInstr], rfl⟩
  | iTestOp hs hc hst => exact ⟨.iTestOp, by simp [label, hc, labelOfInstr], rfl⟩
  | iRelOp hs hc hst => exact ⟨.iRelOp, by simp [label, hc, labelOfInstr], rfl⟩
  | localGet hs hc hv => exact ⟨.localGet, by simp [label, hc, labelOfInstr], rfl⟩
  | localSet hs hc hst hw =>
      exact ⟨.localSet, by simp [label, hc, labelOfInstr], rfl⟩
  | localTee hs hc hst hw =>
      exact ⟨.localTee, by simp [label, hc, labelOfInstr], rfl⟩
  | globalGet hs hc hg =>
      exact ⟨.globalGet, by simp [label, hc, labelOfInstr], rfl⟩
  | globalSet hs hc hst hg =>
      exact ⟨.globalSet, by simp [label, hc, labelOfInstr], rfl⟩
  | load hs hc hst hb => exact ⟨.load, by simp [label, hc, labelOfInstr], rfl⟩
  | loadTrap hs hc hst hb =>
      exact ⟨.loadTrap, by simp [label, hc, labelOfInstr], rfl⟩
  | store hs hc hst hb => exact ⟨.store, by simp [label, hc, labelOfInstr], rfl⟩
  | storeTrap hs hc hst hb =>
      exact ⟨.storeTrap, by simp [label, hc, labelOfInstr], rfl⟩
  | memorySize hs hc =>
      exact ⟨.memorySize, by simp [label, hc, labelOfInstr], rfl⟩
  | memoryGrowSucceed hs hc hst hg =>
      exact ⟨.memoryGrowSucceed (c.stack.headD 0).toNat c.store.memory.pages,
        by simp [label, hc, labelOfInstr], rfl⟩
  | memoryGrowRefuse hs hc hst =>
      exact ⟨.memoryGrowRefuse, by simp [label, hc, labelOfInstr], rfl⟩
  | block hs hc => exact ⟨.blockEnter, by simp [label, hc, labelOfInstr], rfl⟩
  | loop hs hc => exact ⟨.loopEnter, by simp [label, hc, labelOfInstr], rfl⟩
  | ifFalse hs hc hst hcnd =>
      exact ⟨.ifFalse, by simp [label, hc, labelOfInstr, hst, hcnd], rfl⟩
  | ifTrue hs hc hst hcnd =>
      exact ⟨.ifTrue, by simp [label, hc, labelOfInstr, hst, hcnd], rfl⟩
  | brLoop hs hc hk hl =>
      exact ⟨.brLoop, by simp [label, hc, labelOfInstr, hk, hl], rfl⟩
  | brBlock hs hc hk hl =>
      exact ⟨.brBlock, by simp [label, hc, labelOfInstr, hk, hl], rfl⟩
  | brIfFalse hs hc hst hcnd =>
      exact ⟨.brIfFalse, by simp [label, hc, labelOfInstr], rfl⟩
  | brIfLoop hs hc hst hcnd hk hl =>
      exact ⟨.brIfLoop, by simp [label, hc, labelOfInstr, hk, hl], rfl⟩
  | brIfBlock hs hc hst hcnd hk hl =>
      exact ⟨.brIfBlock, by simp [label, hc, labelOfInstr, hk, hl], rfl⟩
  | @throwTag rest tag v st hs hc hst =>
      exact ⟨.throwTag ⟨tag, v⟩, by simp [label, hc, labelOfInstr], rfl⟩
  | exitLabel hs hc hk =>
      exact ⟨.exitLabel, by simp [label, hc, labelAtEnd, hk], rfl⟩
  | returnGemm hs hc hk hp =>
      exact ⟨.returnGemm (c.stack.headD 0),
        by simp [label, hc, labelAtEnd, hk], rfl⟩
  | enterGemm hs hc hk hp hi =>
      exact ⟨.enterGemm c.harness.rawBytes.length,
        by simp [label, hc, labelAtEnd, hk], rfl⟩
  | installTrap hs hc hk hp hi =>
      exact ⟨.installTrap, by simp [label, hc, labelAtEnd, hk], rfl⟩

/-! ## The costed reduction relation

SPEC section 7.5: "Cost instrumentation SHALL not alter control, values, traps,
memory, or observable results."  A costed step is therefore *exactly* a plain
step together with the exact label of that step, and nothing else. -/

/-- **SPEC section 7.5.**  The costed reduction relation: a plain `Step`
carrying the exact cost label computed from its source configuration. -/
def CostedStep (c : Config) (ce : CostedEvent) (c' : Config) : Prop :=
  Step c ce.erase c' ∧ label c ce.erase = some ce

namespace CostedStep

/-- **Erasure of a costed step is structural.** -/
theorem step {c c' : Config} {ce : CostedEvent} (h : CostedStep c ce c') :
    Step c ce.erase c' := h.1

theorem labelled {c c' : Config} {ce : CostedEvent} (h : CostedStep c ce c') :
    label c ce.erase = some ce := h.2

/-- Every plain step carries a costed label. -/
theorem of_step {c c' : Config} {e : Event} (h : Step c e c') :
    ∃ ce : CostedEvent, ce.erase = e ∧ CostedStep c ce c' := by
  obtain ⟨ce, hlab, herase⟩ := step_label h
  refine ⟨ce, herase, ?_, ?_⟩
  · rw [herase]; exact h
  · rw [herase]; exact hlab

/-- **The label is not forgeable.**  Two costed steps out of the same
configuration whose plain events agree carry the same label and reach the same
successor. -/
theorem unique {c c₁ c₂ : Config} {ce₁ ce₂ : CostedEvent}
    (h₁ : CostedStep c ce₁ c₁) (h₂ : CostedStep c ce₂ c₂)
    (he : ce₁.erase = ce₂.erase) : ce₁ = ce₂ ∧ c₁ = c₂ := by
  have hlab : some ce₁ = some ce₂ := by
    rw [← h₁.2, ← h₂.2, he]
  have hce : ce₁ = ce₂ := Option.some.inj hlab
  refine ⟨hce, ?_⟩
  exact Step.deterministic h₁.1 (by rw [he]; exact h₂.1)

/-- A costed step never runs from a terminal configuration. -/
theorem running {c c' : Config} {ce : CostedEvent} (h : CostedStep c ce c') :
    c.status = Status.running := h.1.running

end CostedStep

/-- **SPEC section 7.5.**  A costed reduction sequence, recording the
configurations it visits.  The visited list is pinned structurally: it is the
sequence of configurations actually passed through, so no peak can be
understated. -/
inductive CostedReduces : Config → List CostedEvent → List Config → Config → Prop
  /-- The empty costed reduction visits exactly its own configuration. -/
  | refl (c : Config) : CostedReduces c [] [c] c
  /-- One costed step, then the rest. -/
  | cons {c c' c'' : Config} {ce : CostedEvent} {ct : List CostedEvent}
      {cs : List Config} (h : CostedStep c ce c') (hr : CostedReduces c' ct cs c'') :
      CostedReduces c (ce :: ct) (c :: cs) c''

namespace CostedReduces

/-- **Erasure of a costed reduction sequence is structural.** -/
theorem erase {c c' : Config} {ct : List CostedEvent} {cs : List Config}
    (h : CostedReduces c ct cs c') : Reduces c (eraseCosts ct) c' := by
  induction h with
  | refl c => exact .refl c
  | cons hs _ ih => exact .cons hs.step ih

/-- Every plain reduction sequence has a costed labelling. -/
theorem of_reduces {c c' : Config} {tr : List Event} (h : Reduces c tr c') :
    ∃ (ct : List CostedEvent) (cs : List Config),
      eraseCosts ct = tr ∧ CostedReduces c ct cs c' := by
  induction h with
  | refl c => exact ⟨[], [c], rfl, .refl c⟩
  | @cons c e c₁ tr' c₂ hs _ ih =>
    obtain ⟨ct, cs, hct, hred⟩ := ih
    obtain ⟨ce, herase, hcs⟩ := CostedStep.of_step hs
    exact ⟨ce :: ct, c :: cs, by rw [eraseCosts_cons, herase, hct], .cons hcs hred⟩

/-- The visited list has exactly one more entry than the trace has events. -/
theorem visited_length {c c' : Config} {ct : List CostedEvent} {cs : List Config}
    (h : CostedReduces c ct cs c') : cs.length = ct.length + 1 := by
  induction h with
  | refl _ => rfl
  | cons _ _ ih => simp [ih]

/-- The visited list is never empty. -/
theorem visited_ne_nil {c c' : Config} {ct : List CostedEvent} {cs : List Config}
    (h : CostedReduces c ct cs c') : cs ≠ [] := by
  intro hnil
  have := h.visited_length
  rw [hnil] at this
  simp at this

/-- The source configuration is the head of the visited list. -/
theorem visited_head {c c' : Config} {ct : List CostedEvent} {cs : List Config}
    (h : CostedReduces c ct cs c') : ∃ rest, cs = c :: rest := by
  cases h with
  | refl _ => exact ⟨[], rfl⟩
  | cons _ hr => exact ⟨_, rfl⟩

/-- **The costed trace, the visited configurations and the final configuration
are all determined by the source configuration and the erased trace.**  A
mislabelled or resource-understating costed trace is not a costed reduction of
the same run. -/
theorem unique {c a b : Config} {ct₁ ct₂ : List CostedEvent}
    {cs₁ cs₂ : List Config}
    (h₁ : CostedReduces c ct₁ cs₁ a) (h₂ : CostedReduces c ct₂ cs₂ b)
    (he : eraseCosts ct₁ = eraseCosts ct₂) :
    ct₁ = ct₂ ∧ cs₁ = cs₂ ∧ a = b := by
  induction h₁ generalizing ct₂ cs₂ b with
  | refl c =>
    cases h₂ with
    | refl _ => exact ⟨rfl, rfl, rfl⟩
    | cons _ _ => simp [eraseCosts] at he
  | @cons c c' c'' ce ct cs hs hr ih =>
    cases h₂ with
    | refl _ => simp [eraseCosts] at he
    | @cons _ d' d'' de dt ds hs₂ hr₂ =>
      rw [eraseCosts_cons, eraseCosts_cons] at he
      have hhd : ce.erase = de.erase := (List.cons.inj he).1
      have htl : eraseCosts ct = eraseCosts dt := (List.cons.inj he).2
      obtain ⟨hce, hcfg⟩ := CostedStep.unique hs hs₂ hhd
      subst hce
      subst hcfg
      obtain ⟨h1, h2, h3⟩ := ih hr₂ htl
      exact ⟨by rw [h1], by rw [h2], h3⟩

end CostedReduces

/-! ## Costed finite executions -/

/-- **SPEC section 7.5.**  A costed finite execution: a costed reduction from
`initial` to a terminal configuration producing `observation`, whose recorded
resource snapshots are exactly the snapshots of the configurations visited and
whose erasure is exactly the observation's plain trace. -/
def CostedFiniteExecution (P : Profile) (initial : Config)
    (costedTrace : List CostedEvent) (configs : List ConfigResourceSnapshot)
    (observation : ExecutionObservation) : Prop :=
  ∃ (visited : List Config) (final : Config),
    CostedReduces initial costedTrace visited final ∧
    configs = visited.map (snapshotOf P.costTableBody.layout) ∧
    observation.trace = eraseCosts costedTrace ∧
    observationOfConfig observation.trace final = some observation

namespace CostedFiniteExecution

variable {P : Profile} {initial : Config} {ct : List CostedEvent}
  {configs : List ConfigResourceSnapshot} {observation : ExecutionObservation}

/-- **Erasure of a costed finite execution is the plain finite execution.** -/
theorem erase (h : CostedFiniteExecution P initial ct configs observation) :
    FiniteExecution initial observation := by
  obtain ⟨visited, final, hred, _, htr, hobs⟩ := h
  exact ⟨final, by rw [htr]; exact hred.erase, hobs⟩

/-- The costed trace erases to exactly the observation's plain trace. -/
theorem trace (h : CostedFiniteExecution P initial ct configs observation) :
    observation.trace = eraseCosts ct := by
  obtain ⟨_, _, _, _, htr, _⟩ := h
  exact htr

/-- The snapshot list is nonempty and has one entry per visited
configuration. -/
theorem configs_length (h : CostedFiniteExecution P initial ct configs observation) :
    configs.length = ct.length + 1 := by
  obtain ⟨visited, final, hred, hcfg, _, _⟩ := h
  rw [hcfg, List.length_map, hred.visited_length]

theorem configs_ne_nil (h : CostedFiniteExecution P initial ct configs observation) :
    configs ≠ [] := by
  intro hnil
  have := h.configs_length
  rw [hnil] at this
  simp at this

end CostedFiniteExecution

/-! ## The cost contribution law

SPEC section 7.5 fixes the exact charge of every rule.  `eventContribution` is
the exhaustive structural implementation; `EventContribution` is the same law as
an inductive relation with one constructor per costed event and no default
arm. -/

/-- The charge every relational Core `Step` contributes: one `wasmRuleSteps`
unit and nothing else. -/
def ruleStepCharge (t : CostTableBody) : Cost.DynamicVector :=
  { Cost.DynamicVector.zero with wasmRuleSteps := t.ruleStepUnit }

/-- The charge of a dispatching rule: one rule step and one dispatch step. -/
def dispatchCharge (t : CostTableBody) : Cost.DynamicVector :=
  { ruleStepCharge t with dispatchSteps := 1 }

/-- The abstract byte width of the `i32` the memory rules of the declared
subset transfer.  It is the length of the ABI image `Wasm.u32ToBytes`. -/
def i32TransferBytes : Nat := 4

theorem i32TransferBytes_eq (v : UInt32) :
    i32TransferBytes = (u32ToBytes v).length := rfl

/-- **SPEC section 7.5, the contribution law.**  An exhaustive structural match
over the costed events, with no default arm. -/
def eventContribution (t : CostTableBody) : CostedEvent → Cost.DynamicVector
  | .unreachableTrap => ruleStepCharge t
  | .nop => ruleStepCharge t
  | .i32Const => ruleStepCharge t
  | .drop => ruleStepCharge t
  | .iBinOp => { ruleStepCharge t with scalarOps := 1 }
  | .iBinOpTrap => ruleStepCharge t
  | .iTestOp => { ruleStepCharge t with scalarOps := 1 }
  | .iRelOp => { ruleStepCharge t with scalarOps := 1 }
  | .localGet => ruleStepCharge t
  | .localSet => ruleStepCharge t
  | .localTee => ruleStepCharge t
  | .globalGet => ruleStepCharge t
  | .globalSet => ruleStepCharge t
  | .load => { ruleStepCharge t with bytesRead := i32TransferBytes }
  | .loadTrap => ruleStepCharge t
  | .store => { ruleStepCharge t with bytesWritten := i32TransferBytes }
  | .storeTrap => ruleStepCharge t
  | .memorySize => ruleStepCharge t
  | .memoryGrowSucceed deltaPages _ =>
      { ruleStepCharge t with memoryGrowPages := deltaPages }
  | .memoryGrowRefuse => ruleStepCharge t
  | .blockEnter => ruleStepCharge t
  | .loopEnter => ruleStepCharge t
  | .ifFalse => dispatchCharge t
  | .ifTrue => dispatchCharge t
  | .brLoop => dispatchCharge t
  | .brBlock => dispatchCharge t
  | .brIfFalse => ruleStepCharge t
  | .brIfLoop => dispatchCharge t
  | .brIfBlock => dispatchCharge t
  | .throwTag _ =>
      { dispatchCharge t with
          gcObjectsAllocated := 1
          gcBytesInitialized := exceptionObjectBytes t.layout }
  | .exitLabel => ruleStepCharge t
  | .returnGemm _ => { dispatchCharge t with outputBytes := i32TransferBytes }
  | .enterGemm installedBytes =>
      { dispatchCharge t with
          preparationSteps := t.installationPreparationUnit
          bytesWritten := t.installedByteWriteUnit * installedBytes }
  | .installTrap => ruleStepCharge t

/-- **SPEC section 7.5, the contribution law as a relation.**  One constructor
per costed event; there is no default arm, so an event added without a decided
charge has no contribution at all and `wasm_cost_table_total` fails. -/
inductive EventContribution (t : CostTableBody) :
    CostedEvent → Cost.DynamicVector → Prop
  | unreachableTrap : EventContribution t .unreachableTrap (ruleStepCharge t)
  | nop : EventContribution t .nop (ruleStepCharge t)
  | i32Const : EventContribution t .i32Const (ruleStepCharge t)
  | drop : EventContribution t .drop (ruleStepCharge t)
  | iBinOp : EventContribution t .iBinOp { ruleStepCharge t with scalarOps := 1 }
  | iBinOpTrap : EventContribution t .iBinOpTrap (ruleStepCharge t)
  | iTestOp : EventContribution t .iTestOp { ruleStepCharge t with scalarOps := 1 }
  | iRelOp : EventContribution t .iRelOp { ruleStepCharge t with scalarOps := 1 }
  | localGet : EventContribution t .localGet (ruleStepCharge t)
  | localSet : EventContribution t .localSet (ruleStepCharge t)
  | localTee : EventContribution t .localTee (ruleStepCharge t)
  | globalGet : EventContribution t .globalGet (ruleStepCharge t)
  | globalSet : EventContribution t .globalSet (ruleStepCharge t)
  | load : EventContribution t .load
      { ruleStepCharge t with bytesRead := i32TransferBytes }
  | loadTrap : EventContribution t .loadTrap (ruleStepCharge t)
  | store : EventContribution t .store
      { ruleStepCharge t with bytesWritten := i32TransferBytes }
  | storeTrap : EventContribution t .storeTrap (ruleStepCharge t)
  | memorySize : EventContribution t .memorySize (ruleStepCharge t)
  | memoryGrowSucceed (deltaPages previousPages : Nat) :
      EventContribution t (.memoryGrowSucceed deltaPages previousPages)
        { ruleStepCharge t with memoryGrowPages := deltaPages }
  | memoryGrowRefuse : EventContribution t .memoryGrowRefuse (ruleStepCharge t)
  | blockEnter : EventContribution t .blockEnter (ruleStepCharge t)
  | loopEnter : EventContribution t .loopEnter (ruleStepCharge t)
  | ifFalse : EventContribution t .ifFalse (dispatchCharge t)
  | ifTrue : EventContribution t .ifTrue (dispatchCharge t)
  | brLoop : EventContribution t .brLoop (dispatchCharge t)
  | brBlock : EventContribution t .brBlock (dispatchCharge t)
  | brIfFalse : EventContribution t .brIfFalse (ruleStepCharge t)
  | brIfLoop : EventContribution t .brIfLoop (dispatchCharge t)
  | brIfBlock : EventContribution t .brIfBlock (dispatchCharge t)
  | throwTag (ev : ExceptionValue) :
      EventContribution t (.throwTag ev)
        { dispatchCharge t with
            gcObjectsAllocated := 1
            gcBytesInitialized := exceptionObjectBytes t.layout }
  | exitLabel : EventContribution t .exitLabel (ruleStepCharge t)
  | returnGemm (v : UInt32) :
      EventContribution t (.returnGemm v)
        { dispatchCharge t with outputBytes := i32TransferBytes }
  | enterGemm (installedBytes : Nat) :
      EventContribution t (.enterGemm installedBytes)
        { dispatchCharge t with
            preparationSteps := t.installationPreparationUnit
            bytesWritten := t.installedByteWriteUnit * installedBytes }
  | installTrap : EventContribution t .installTrap (ruleStepCharge t)

/-- The relation is the graph of the structural match. -/
theorem eventContribution_iff (t : CostTableBody) (ce : CostedEvent)
    (v : Cost.DynamicVector) :
    EventContribution t ce v ↔ eventContribution t ce = v := by
  constructor
  · intro h; cases h <;> rfl
  · intro h
    subst h
    cases ce <;> (unfold eventContribution; constructor)

/-- **SPEC section 7.5, cost-table totality and exclusivity.**  Every costed
event has exactly one contribution.  `EventContribution` has one constructor per
costed event and no default arm, so both halves are real: totality fails if a
rule is left uncosted, exclusivity fails if a rule is costed twice. -/
theorem wasm_cost_table_total (t : CostTableBody) (event : CostedEvent) :
    ∃ contribution : Cost.DynamicVector,
      EventContribution t event contribution ∧
        ∀ other : Cost.DynamicVector,
          EventContribution t event other → other = contribution := by
  refine ⟨eventContribution t event, (eventContribution_iff t event _).mpr rfl, ?_⟩
  intro y hy
  exact ((eventContribution_iff t event y).mp hy).symm

/-! ### Consequences of the contribution law -/

/-- Every relational Core `Step` contributes exactly one `wasmRuleSteps`
unit. -/
theorem eventContribution_wasmRuleSteps (P : Profile) (ce : CostedEvent) :
    (eventContribution P.costTableBody ce).wasmRuleSteps = 1 := by
  have h : P.costTableBody.ruleStepUnit = 1 := P.lawful.ruleStepUnit
  cases ce <;> simp [eventContribution, ruleStepCharge, dispatchCharge,
    Cost.DynamicVector.zero, h]

/-- `dispatchSteps` counts exactly the branch, if-arm, return, exception
transfer and harness phase-transition rules. -/
def IsDispatchRule : RuleId → Bool
  | .ifFalse | .ifTrue | .brLoop | .brBlock | .brIfLoop | .brIfBlock
  | .throwTag | .returnGemm | .enterGemm => true
  | _ => false

theorem eventContribution_dispatchSteps (t : CostTableBody) (ce : CostedEvent) :
    (eventContribution t ce).dispatchSteps =
      if IsDispatchRule ce.rule then 1 else 0 := by
  cases ce <;> rfl

/-- `scalarOps` counts exactly the completed scalar primitive numeric
operations.  A trapping division contributes its attempted rule step and no
completed operation. -/
theorem eventContribution_scalarOps (t : CostTableBody) (ce : CostedEvent) :
    (eventContribution t ce).scalarOps =
      match ce with
      | .iBinOp | .iTestOp | .iRelOp => 1
      | _ => 0 := by
  cases ce <;> rfl

/-- A trapping load contributes the attempted rule step but no completed
transfer. -/
theorem eventContribution_loadTrap_bytesRead (t : CostTableBody) :
    (eventContribution t .loadTrap).bytesRead = 0 := rfl

/-- A trapping store contributes the attempted rule step but no completed
transfer. -/
theorem eventContribution_storeTrap_bytesWritten (t : CostTableBody) :
    (eventContribution t .storeTrap).bytesWritten = 0 := rfl

/-- A trapping raw installation contributes the attempted rule step but no
completed transfer. -/
theorem eventContribution_installTrap_bytesWritten (t : CostTableBody) :
    (eventContribution t .installTrap).bytesWritten = 0 := rfl

/-- A refused `memory.grow` allocates no page. -/
theorem eventContribution_memoryGrowRefuse_pages (t : CostTableBody) :
    (eventContribution t .memoryGrowRefuse).memoryGrowPages = 0 := rfl

/-- A completed load reads exactly the four ABI bytes of the `i32` it
transfers. -/
theorem eventContribution_load_bytesRead (t : CostTableBody) :
    (eventContribution t .load).bytesRead = 4 := rfl

/-- A completed store writes exactly the four ABI bytes of the `i32` it
transfers. -/
theorem eventContribution_store_bytesWritten (t : CostTableBody) :
    (eventContribution t .store).bytesWritten = 4 := rfl

/-- Raw installation contributes one `preparationSteps` and one `bytesWritten`
unit per installed byte. -/
theorem eventContribution_enterGemm (P : Profile) (n : Nat) :
    (eventContribution P.costTableBody (.enterGemm n)).preparationSteps = 1 ∧
    (eventContribution P.costTableBody (.enterGemm n)).bytesWritten = n := by
  refine ⟨P.lawful.installationPreparationUnit, ?_⟩
  show P.body.costTableBody.installedByteWriteUnit * n = n
  rw [P.lawful.installedByteWriteUnit, Nat.one_mul]

/-- An exception transfer allocates exactly one exception object of the
canonical abstract width. -/
theorem eventContribution_throwTag_gc (P : Profile) (ev : ExceptionValue) :
    (eventContribution P.costTableBody (.throwTag ev)).gcObjectsAllocated = 1 ∧
    (eventContribution P.costTableBody (.throwTag ev)).gcBytesInitialized = 24 := by
  refine ⟨rfl, ?_⟩
  show exceptionObjectBytes P.costTableBody.layout = 24
  rw [P.layout_eq]
  exact canonical_exceptionObjectBytes

/-- The peak coordinates are never charged by an event: SPEC section 7.5 makes
them maxima over the configuration sequence, and `foldTrace` takes them from the
snapshots alone. -/
theorem eventContribution_peaks_zero (t : CostTableBody) (ce : CostedEvent) :
    (eventContribution t ce).peakStackValues = 0 ∧
    (eventContribution t ce).peakPages = 0 ∧
    (eventContribution t ce).peakGcLiveBytes = 0 := by
  cases ce <;> exact ⟨rfl, rfl, rfl⟩

/-- No execution rule of the declared subset charges `instantiationSteps`:
allocation and static initialization happen before the first `Step`. -/
theorem eventContribution_instantiationSteps_zero (t : CostTableBody)
    (ce : CostedEvent) : (eventContribution t ce).instantiationSteps = 0 := by
  cases ce <;> rfl

/-- **Scope.**  No execution rule of the declared subset charges
`vectorLaneOps`, because `Wasm.Step` has no `v128` rule: the subset performs no
vector operation.  This does **not** establish that vector work is free.  The
released lane law is `wholeVectorShuffleCharge`, which charges sixteen byte
lanes for a whole-vector shuffle. -/
theorem eventContribution_vectorLaneOps_zero (t : CostTableBody)
    (ce : CostedEvent) : (eventContribution t ce).vectorLaneOps = 0 := by
  cases ce <;> rfl

/-- **Scope.**  No execution rule of the declared subset charges
`tableElementsAllocated`, because the released `Store` has no table instance.
This does **not** establish that table operations are free. -/
theorem eventContribution_tableElementsAllocated_zero (t : CostTableBody)
    (ce : CostedEvent) : (eventContribution t ce).tableElementsAllocated = 0 := by
  cases ce <;> rfl

/-! ### The lane and GC laws the subset does not exercise

SPEC section 7.5 fixes the charge of a whole-vector shuffle and of every GC
allocation.  Those laws are implemented here against the pinned canonical
widths, so that enabling the corresponding rules is a matter of adding a costed
event, not of inventing a charge. -/

/-- SPEC section 7.5: a whole-vector shuffle contributes sixteen byte lanes. -/
def wholeVectorShuffleCharge (t : CostTableBody) : Cost.DynamicVector :=
  { ruleStepCharge t with vectorLaneOps := t.wholeVectorShuffleLanes }

theorem wholeVectorShuffleCharge_lanes (P : Profile) :
    (wholeVectorShuffleCharge P.costTableBody).vectorLaneOps = 16 :=
  P.wholeVectorShuffleLanes_eq

/-- SPEC section 7.5: an operation over `lanes` active 128-bit lanes charges
exactly those lanes. -/
def vectorLaneCharge (t : CostTableBody) (lanes : Nat) : Cost.DynamicVector :=
  { ruleStepCharge t with vectorLaneOps := lanes }

theorem vectorLaneCharge_exact (t : CostTableBody) (lanes : Nat) :
    (vectorLaneCharge t lanes).vectorLaneOps = lanes := rfl

/-- SPEC section 7.5: a GC structure allocation contributes one
`gcObjectsAllocated` and the canonical abstract width of the object to
`gcBytesInitialized`. -/
def structAllocationCharge (t : CostTableBody) (s : StructType) :
    Cost.DynamicVector :=
  { ruleStepCharge t with
      gcObjectsAllocated := 1
      gcBytesInitialized := t.layout.structSize s }

/-- SPEC section 7.5: a GC array allocation of `length` elements charges the
header plus `length` strides, rounded. -/
def arrayAllocationCharge (t : CostTableBody) (a : ArrayType) (length : Nat) :
    Cost.DynamicVector :=
  { ruleStepCharge t with
      gcObjectsAllocated := 1
      gcBytesInitialized := t.layout.arraySize a length }

/-- SPEC section 7.5: an exception-object allocation charges its 16-byte header
plus its declared-order payload, rounded. -/
def exceptionAllocationCharge (t : CostTableBody) (tag : TagType) :
    Cost.DynamicVector :=
  { dispatchCharge t with
      gcObjectsAllocated := 1
      gcBytesInitialized := t.layout.exceptionSize tag }

/-- A runtime-sized array allocation is never a unit-cost free store: its
charge grows with its length. -/
theorem arrayAllocationCharge_ge (P : Profile) (a : ArrayType) (length : Nat) :
    length * P.costTableBody.layout.arrayStride a.element ≤
      (arrayAllocationCharge P.costTableBody a length).gcBytesInitialized :=
  GcLayoutConstants.length_mul_arrayStride_le_arraySize P.layout_positive a length

/-- The charge of an exception object of the declared subset's tag is the live
width the configuration snapshots report. -/
theorem exceptionAllocationCharge_value (t : CostTableBody) :
    (exceptionAllocationCharge t exceptionValueTagType).gcBytesInitialized =
      exceptionObjectBytes t.layout := rfl

/-! ## Folding a costed trace

SPEC section 7.5: "`peakStackValues`, `peakPages`, and every other peak are
maxima over the complete costed configuration sequence, while all nonpeak
coordinates are sums." -/

/-- The peak contribution of one visited configuration. -/
def snapshotPeak (s : ConfigResourceSnapshot) : Cost.DynamicVector :=
  { Cost.DynamicVector.zero with
      peakStackValues := s.liveValueSlots
      peakPages := s.memoryPages
      peakGcLiveBytes := s.liveGcBytes }

/-- The summed part of a costed trace's cost. -/
def traceContribution (t : CostTableBody) (ct : List CostedEvent) :
    Cost.DynamicVector :=
  ct.foldl (fun acc ce => Cost.sequentialCompose acc (eventContribution t ce))
    Cost.DynamicVector.zero

/-- The maximised part of a costed run's cost. -/
def peakOverConfigs (configs : List ConfigResourceSnapshot) :
    Cost.DynamicVector :=
  configs.foldl (fun acc s => Cost.ComponentwiseMax acc (snapshotPeak s))
    Cost.DynamicVector.zero

/-- **SPEC section 7.5.**  The exact mixed sum/maximum fold over the event
contributions and the visited configurations. -/
def foldTrace (t : CostTableBody) (configs : List ConfigResourceSnapshot)
    (ct : List CostedEvent) : Cost.DynamicVector :=
  Cost.sequentialCompose (traceContribution t ct) (peakOverConfigs configs)

/-! ### Coverage: no charge escapes the fold -/

theorem foldl_seq_le_self {α : Type} (f : α → Cost.DynamicVector) :
    ∀ (l : List α) (i : Cost.DynamicVector),
      Cost.DynamicVector.ComponentwiseLE i
        (l.foldl (fun acc x => Cost.sequentialCompose acc (f x)) i) := by
  intro l
  induction l with
  | nil => intro i; exact Cost.DynamicVector.componentwiseLE_refl i
  | cons x xs ih =>
    intro i
    exact Cost.DynamicVector.componentwiseLE_trans
      (Cost.sequentialCompose_le_left i (f x))
      (ih (Cost.sequentialCompose i (f x)))

theorem le_foldl_seq {α : Type} (f : α → Cost.DynamicVector) :
    ∀ (l : List α) (i : Cost.DynamicVector) (a : α), a ∈ l →
      Cost.DynamicVector.ComponentwiseLE (f a)
        (l.foldl (fun acc x => Cost.sequentialCompose acc (f x)) i) := by
  intro l
  induction l with
  | nil => intro i a h; exact absurd h (by simp)
  | cons x xs ih =>
    intro i a h
    rcases List.mem_cons.mp h with rfl | hmem
    · exact Cost.DynamicVector.componentwiseLE_trans
        (Cost.sequentialCompose_le_right i (f a))
        (foldl_seq_le_self f xs (Cost.sequentialCompose i (f a)))
    · exact ih _ a hmem

theorem foldl_max_le_self {α : Type} (f : α → Cost.DynamicVector) :
    ∀ (l : List α) (i : Cost.DynamicVector),
      Cost.DynamicVector.ComponentwiseLE i
        (l.foldl (fun acc x => Cost.ComponentwiseMax acc (f x)) i) := by
  intro l
  induction l with
  | nil => intro i; exact Cost.DynamicVector.componentwiseLE_refl i
  | cons x xs ih =>
    intro i
    refine Cost.DynamicVector.componentwiseLE_trans ?_
      (ih (Cost.ComponentwiseMax i (f x)))
    intro dc
    rw [Cost.value_componentwiseMax]
    exact Nat.le_max_left _ _

theorem le_foldl_max {α : Type} (f : α → Cost.DynamicVector) :
    ∀ (l : List α) (i : Cost.DynamicVector) (a : α), a ∈ l →
      Cost.DynamicVector.ComponentwiseLE (f a)
        (l.foldl (fun acc x => Cost.ComponentwiseMax acc (f x)) i) := by
  intro l
  induction l with
  | nil => intro i a h; exact absurd h (by simp)
  | cons x xs ih =>
    intro i a h
    rcases List.mem_cons.mp h with rfl | hmem
    · refine Cost.DynamicVector.componentwiseLE_trans ?_
        (foldl_max_le_self f xs (Cost.ComponentwiseMax i (f a)))
      intro dc
      rw [Cost.value_componentwiseMax]
      exact Nat.le_max_right _ _
    · exact ih _ a hmem

/-- **Every charged event appears in the trace cost.** -/
theorem eventContribution_le_traceContribution (t : CostTableBody)
    {ce : CostedEvent} {ct : List CostedEvent} (h : ce ∈ ct) :
    Cost.DynamicVector.ComponentwiseLE (eventContribution t ce)
      (traceContribution t ct) :=
  le_foldl_seq (eventContribution t) ct Cost.DynamicVector.zero ce h

/-- **Every visited configuration's peaks appear in the run cost.** -/
theorem snapshotPeak_le_peakOverConfigs {s : ConfigResourceSnapshot}
    {configs : List ConfigResourceSnapshot} (h : s ∈ configs) :
    Cost.DynamicVector.ComponentwiseLE (snapshotPeak s)
      (peakOverConfigs configs) :=
  le_foldl_max snapshotPeak configs Cost.DynamicVector.zero s h

theorem eventContribution_le_foldTrace (t : CostTableBody)
    (configs : List ConfigResourceSnapshot) {ce : CostedEvent}
    {ct : List CostedEvent} (h : ce ∈ ct) :
    Cost.DynamicVector.ComponentwiseLE (eventContribution t ce)
      (foldTrace t configs ct) :=
  Cost.DynamicVector.componentwiseLE_trans
    (eventContribution_le_traceContribution t h)
    (Cost.sequentialCompose_le_left _ _)

theorem snapshotPeak_le_foldTrace (t : CostTableBody)
    {configs : List ConfigResourceSnapshot} (ct : List CostedEvent)
    {s : ConfigResourceSnapshot} (h : s ∈ configs) :
    Cost.DynamicVector.ComponentwiseLE (snapshotPeak s)
      (foldTrace t configs ct) :=
  Cost.DynamicVector.componentwiseLE_trans (snapshotPeak_le_peakOverConfigs h)
    (Cost.sequentialCompose_le_right _ _)

/-- **The peaks are real maxima over the configuration sequence.**  Every
visited configuration's live value slots, pages and live managed bytes are
dominated by the recorded peaks. -/
theorem peaks_dominate_every_configuration (P : Profile)
    {initial : Config} {ct : List CostedEvent} {visited : List Config}
    {final : Config} {configs : List ConfigResourceSnapshot}
    (_hred : CostedReduces initial ct visited final)
    (hcfg : configs = visited.map (snapshotOf P.costTableBody.layout))
    {c : Config} (hmem : c ∈ visited) :
    c.liveValueSlots ≤ (foldTrace P.costTableBody configs ct).peakStackValues ∧
    c.livePages ≤ (foldTrace P.costTableBody configs ct).peakPages ∧
    c.liveGcBytes P.costTableBody.layout ≤
      (foldTrace P.costTableBody configs ct).peakGcLiveBytes := by
  have hs : snapshotOf P.costTableBody.layout c ∈ configs := by
    rw [hcfg]; exact List.mem_map_of_mem hmem
  have hle := snapshotPeak_le_foldTrace P.costTableBody ct hs
  exact ⟨hle .peakStackValues, hle .peakPages, hle .peakGcLiveBytes⟩

/-! ## Static cost

SPEC section 7.5 fixes `decodeCost` and `validationCost` independently of any
decoder or validator implementation. -/

/-- SPEC section 7.5: one unit per consumed byte and one terminal accept/reject
unit, independent of decoder implementation. -/
theorem decodeCost_eq (P : Profile) (bytes : ByteArray) :
    P.costTableBody.decodeCost bytes = bytes.size + 1 :=
  P.decodeCost_eq bytes

/-- SPEC section 7.5: one unit per node plus one unit per premise edge of the
unique canonical declarative-validation derivation. -/
theorem validationCost_eq (P : Profile) (nodes edges : Nat) :
    P.costTableBody.validationCost nodes edges = nodes + edges :=
  P.validationCost_eq nodes edges

/-- The static coordinates of one costed artifact. -/
def staticCost (t : CostTableBody) (bytes : ByteArray)
    (validationNodes validationEdges staticDataBytes : Nat) :
    Cost.StaticVector :=
  { moduleBytes := bytes.size
    decodeSteps := t.decodeCost bytes
    validationSteps := t.validationCost validationNodes validationEdges
    staticDataBytes := staticDataBytes }

theorem staticCost_decodeSteps (P : Profile) (bytes : ByteArray)
    (n e d : Nat) :
    (staticCost P.costTableBody bytes n e d).decodeSteps = bytes.size + 1 :=
  P.decodeCost_eq bytes

theorem staticCost_validationSteps (P : Profile) (bytes : ByteArray)
    (n e d : Nat) :
    (staticCost P.costTableBody bytes n e d).validationSteps = n + e :=
  P.validationCost_eq n e

theorem staticCost_moduleBytes (t : CostTableBody) (bytes : ByteArray)
    (n e d : Nat) : (staticCost t bytes n e d).moduleBytes = bytes.size := rfl

/-! ## The pinned cost-table rows (SPEC section 7.5)

`Wasm/Profile.lean` carries the row *data* (`Wasm.canonicalRuleRows`,
`Wasm.canonicalInitializationRows`); it cannot name `RuleId`, because it is
imported by this file rather than importing it.  This section supplies the
pinned identifier of every rule, restates the row of every rule, and proves the
three conditions SPEC section 7.5 requires of `buildCanonicalCostTable`'s
argument: the rows are a *cover*, they are *duplicate free*, and the cover is
*exact*.  It then proves the rows are not decorative, by showing every row's
contribution is exactly what the contribution law charges. -/

namespace RuleId

/-- The pinned identifier of a Core rule. -/
def name : RuleId → String
  | .unreachable => "core3/step/unreachable"
  | .nop => "core3/step/nop"
  | .i32Const => "core3/step/i32.const"
  | .drop => "core3/step/drop"
  | .iBinOp => "core3/step/i32.binop"
  | .iBinOpTrap => "core3/step/i32.binop-trap"
  | .iTestOp => "core3/step/i32.testop"
  | .iRelOp => "core3/step/i32.relop"
  | .localGet => "core3/step/local.get"
  | .localSet => "core3/step/local.set"
  | .localTee => "core3/step/local.tee"
  | .globalGet => "core3/step/global.get"
  | .globalSet => "core3/step/global.set"
  | .load => "core3/step/i32.load"
  | .loadTrap => "core3/step/i32.load-trap"
  | .store => "core3/step/i32.store"
  | .storeTrap => "core3/step/i32.store-trap"
  | .memorySize => "core3/step/memory.size"
  | .memoryGrowSucceed => "core3/step/memory.grow-succeed"
  | .memoryGrowRefuse => "core3/step/memory.grow-refuse"
  | .block => "core3/step/block"
  | .loop => "core3/step/loop"
  | .ifFalse => "core3/step/if-false"
  | .ifTrue => "core3/step/if-true"
  | .brLoop => "core3/step/br-loop"
  | .brBlock => "core3/step/br-block"
  | .brIfFalse => "core3/step/br_if-false"
  | .brIfLoop => "core3/step/br_if-loop"
  | .brIfBlock => "core3/step/br_if-block"
  | .throwTag => "core3/step/throw"
  | .exitLabel => "core3/step/exit-label"
  | .returnGemm => "core3/step/return-gemm"
  | .enterGemm => "core3/step/enter-gemm"
  | .installTrap => "core3/step/install-trap"

/-- The pinned cost-table row of a Core rule. -/
def row : RuleId → CostRuleRow
  | .unreachable =>
      canonicalRow "core3/step/unreachable" canonicalRuleStepContribution
  | .nop => canonicalRow "core3/step/nop" canonicalRuleStepContribution
  | .i32Const =>
      canonicalRow "core3/step/i32.const" canonicalRuleStepContribution
  | .drop => canonicalRow "core3/step/drop" canonicalRuleStepContribution
  | .iBinOp =>
      canonicalRow "core3/step/i32.binop"
        { canonicalRuleStepContribution with scalarOps := 1 }
  | .iBinOpTrap =>
      canonicalRow "core3/step/i32.binop-trap" canonicalRuleStepContribution
  | .iTestOp =>
      canonicalRow "core3/step/i32.testop"
        { canonicalRuleStepContribution with scalarOps := 1 }
  | .iRelOp =>
      canonicalRow "core3/step/i32.relop"
        { canonicalRuleStepContribution with scalarOps := 1 }
  | .localGet =>
      canonicalRow "core3/step/local.get" canonicalRuleStepContribution
  | .localSet =>
      canonicalRow "core3/step/local.set" canonicalRuleStepContribution
  | .localTee =>
      canonicalRow "core3/step/local.tee" canonicalRuleStepContribution
  | .globalGet =>
      canonicalRow "core3/step/global.get" canonicalRuleStepContribution
  | .globalSet =>
      canonicalRow "core3/step/global.set" canonicalRuleStepContribution
  | .load =>
      canonicalRow "core3/step/i32.load"
        { canonicalRuleStepContribution with bytesRead := canonicalTransferBytes }
  | .loadTrap =>
      canonicalRow "core3/step/i32.load-trap" canonicalRuleStepContribution
  | .store =>
      canonicalRow "core3/step/i32.store"
        { canonicalRuleStepContribution with
            bytesWritten := canonicalTransferBytes }
  | .storeTrap =>
      canonicalRow "core3/step/i32.store-trap" canonicalRuleStepContribution
  | .memorySize =>
      canonicalRow "core3/step/memory.size" canonicalRuleStepContribution
  | .memoryGrowSucceed =>
      canonicalRow "core3/step/memory.grow-succeed"
        { canonicalRuleStepContribution with memoryGrowPages := 1 }
  | .memoryGrowRefuse =>
      canonicalRow "core3/step/memory.grow-refuse" canonicalRuleStepContribution
  | .block => canonicalRow "core3/step/block" canonicalRuleStepContribution
  | .loop => canonicalRow "core3/step/loop" canonicalRuleStepContribution
  | .ifFalse => canonicalRow "core3/step/if-false" canonicalDispatchContribution
  | .ifTrue => canonicalRow "core3/step/if-true" canonicalDispatchContribution
  | .brLoop => canonicalRow "core3/step/br-loop" canonicalDispatchContribution
  | .brBlock => canonicalRow "core3/step/br-block" canonicalDispatchContribution
  | .brIfFalse =>
      canonicalRow "core3/step/br_if-false" canonicalRuleStepContribution
  | .brIfLoop =>
      canonicalRow "core3/step/br_if-loop" canonicalDispatchContribution
  | .brIfBlock =>
      canonicalRow "core3/step/br_if-block" canonicalDispatchContribution
  | .throwTag =>
      canonicalRow "core3/step/throw"
        { canonicalDispatchContribution with
            gcObjectsAllocated := 1
            gcBytesInitialized := canonicalExceptionBytes }
  | .exitLabel =>
      canonicalRow "core3/step/exit-label" canonicalRuleStepContribution
  | .returnGemm =>
      canonicalRow "core3/step/return-gemm"
        { canonicalDispatchContribution with
            outputBytes := canonicalTransferBytes }
  | .enterGemm =>
      canonicalRow "core3/step/enter-gemm"
        { canonicalDispatchContribution with
            preparationSteps := 1
            bytesWritten := 1 }
  | .installTrap =>
      canonicalRow "core3/step/install-trap" canonicalRuleStepContribution

@[simp] theorem row_ruleId (r : RuleId) : (row r).ruleId = name r := by
  cases r <;> rfl

end RuleId

/-- Duplicate-free identifiers on a finite cover give an injective naming: no
two distinct rules can share a pinned identifier. -/
theorem nodup_map_inj {α β : Type} {f : α → β} :
    ∀ {l : List α}, (l.map f).Nodup → ∀ {a b : α}, a ∈ l → b ∈ l → f a = f b →
      a = b := by
  intro l
  induction l with
  | nil => intro _ a b ha _ _; exact absurd ha (by simp)
  | cons x xs ih =>
    intro hnd a b ha hb hf
    rw [List.map_cons] at hnd
    have hnd' := List.nodup_cons.mp hnd
    rcases List.mem_cons.mp ha with rfl | ha'
    · rcases List.mem_cons.mp hb with rfl | hb'
      · rfl
      · exact absurd (hf ▸ List.mem_map_of_mem hb') hnd'.1
    · rcases List.mem_cons.mp hb with rfl | hb'
      · exact absurd (hf ▸ List.mem_map_of_mem ha') hnd'.1
      · exact ih hnd'.2 ha' hb' hf

/-- **SPEC section 7.5, exact cover.**  The rule rows of the canonical cost
table are exactly the rows of the pinned rule identifiers, in the pinned order:
no rule is missing, and no row names anything that is not a rule. -/
theorem canonicalCostTable_exact_cover :
    canonicalCostTableUnits.ruleRows = RuleId.all.map RuleId.row := rfl

/-- The identifier-level form of the exact cover. -/
theorem canonicalCostTable_exact_cover_ids :
    canonicalCostTableUnits.ruleRows.map CostRuleRow.ruleId =
      RuleId.all.map RuleId.name := rfl

/-- **SPEC section 7.5, duplicate freedom.**  No rule identifier can be charged
two different costs. -/
theorem canonicalCostTable_ruleRows_nodup :
    (canonicalCostTableUnits.ruleRows.map CostRuleRow.ruleId).Nodup :=
  canonicalRuleRows_nodup

/-- The row of a rule is the row its identifier looks up. -/
theorem canonicalCostTable_rowFor (r : RuleId) :
    canonicalCostTableUnits.rowFor? r.name = some r.row := by
  have hmem : r.row ∈ canonicalCostTableUnits.ruleRows := by
    rw [canonicalCostTable_exact_cover]
    exact List.mem_map_of_mem (RuleId.mem_all r)
  have h :=
    CostTableBody.rowFor?_eq_of_mem canonicalCostTable_ruleRows_nodup hmem
  rwa [RuleId.row_ruleId] at h

/-- **SPEC section 7.5, coverage.**  Every pinned Core rule identifier has a
row.  This is the property CO-006 reported violated: with `ruleRows = []`,
`rowFor?` returned `none` for every rule. -/
theorem canonicalCostTable_covers_every_rule (r : RuleId) :
    (canonicalCostTableUnits.rowFor? r.name).isSome := by
  rw [canonicalCostTable_rowFor]; rfl

/-- No two distinct Core rules share a pinned identifier. -/
theorem RuleId.name_injective : Function.Injective RuleId.name := by
  intro a b h
  have hnd : (RuleId.all.map RuleId.name).Nodup := by
    rw [← canonicalCostTable_exact_cover_ids]
    exact canonicalCostTable_ruleRows_nodup
  exact nodup_map_inj hnd (RuleId.mem_all a) (RuleId.mem_all b) h

/-! ### The rows are the contribution law, not a description of it -/

/-- The operand-determined multiplicity of a costed event.  Exactly two rules
of the declared subset transfer a quantity that is an *operand* rather than a
constant of the rule: `memory.grow` transfers the requested page count, and the
harness entry writes the installed raw byte count.  Every other rule has
multiplicity one. -/
def CostedEvent.parametricMultiplicity : CostedEvent → Nat
  | .memoryGrowSucceed deltaPages _ => deltaPages
  | .enterGemm installedBytes => installedBytes
  | _ => 1

/-- A row's contribution scaled by an operand-determined multiplicity: only the
two per-unit coordinates scale. -/
def scaleParametric (v : Cost.DynamicVector) (k : Nat) : Cost.DynamicVector :=
  { v with
      memoryGrowPages := v.memoryGrowPages * k
      bytesWritten := v.bytesWritten * k }

@[simp] theorem scaleParametric_one (v : Cost.DynamicVector) :
    scaleParametric v 1 = v := by
  cases v; simp [scaleParametric]

/-- **The rows are exact.**  For every costed event, the contribution the cost
law charges is exactly the contribution stored in the row of the rule that
fired, scaled by that event's operand-determined multiplicity.  A row that
understated or overstated any of the sixteen coordinates would break this. -/
theorem canonicalCostTable_charges_exactly (ce : CostedEvent) :
    eventContribution canonicalCostTableUnits ce =
      scaleParametric (ce.rule.row).contribution ce.parametricMultiplicity := by
  cases ce with
  | memoryGrowSucceed deltaPages previousPages =>
    show { ruleStepCharge canonicalCostTableUnits with
             memoryGrowPages := deltaPages } = _
    simp [scaleParametric, RuleId.row, CostedEvent.rule,
      CostedEvent.parametricMultiplicity, canonicalRow,
      canonicalRuleStepContribution, ruleStepCharge, canonicalCostTableUnits,
      Cost.DynamicVector.zero]
  | enterGemm installedBytes =>
    show { dispatchCharge canonicalCostTableUnits with
             preparationSteps := canonicalCostTableUnits.installationPreparationUnit
             bytesWritten :=
               canonicalCostTableUnits.installedByteWriteUnit * installedBytes } = _
    simp [scaleParametric, RuleId.row, CostedEvent.rule,
      CostedEvent.parametricMultiplicity, canonicalRow,
      canonicalDispatchContribution, canonicalRuleStepContribution,
      dispatchCharge, ruleStepCharge, canonicalCostTableUnits,
      Cost.DynamicVector.zero]
  | _ => rfl

/-- The specialisation to the thirty-two rules whose charge is constant: the
stored row *is* the charge. -/
theorem canonicalCostTable_charges_row (ce : CostedEvent)
    (h : ce.parametricMultiplicity = 1) :
    eventContribution canonicalCostTableUnits ce = (ce.rule.row).contribution := by
  rw [canonicalCostTable_charges_exactly, h, scaleParametric_one]

/-- Every row charges exactly one `wasmRuleSteps` unit: SPEC section 7.5's
"every relational Core `Step` contributes one `wasmRuleSteps` unit", read off
the table.  No row is a zero charge. -/
theorem canonicalCostTable_row_wasmRuleSteps (r : RuleId) :
    (r.row).contribution.wasmRuleSteps = 1 := by cases r <;> rfl

/-- `dispatchSteps` is one exactly on the dispatching rules and zero
elsewhere. -/
theorem canonicalCostTable_row_dispatchSteps (r : RuleId) :
    (r.row).contribution.dispatchSteps = if IsDispatchRule r then 1 else 0 := by
  cases r <;> rfl

/-- `scalarOps` is one exactly on the three completed scalar primitive numeric
rules; the trapping division charges none. -/
theorem canonicalCostTable_row_scalarOps (r : RuleId) :
    (r.row).contribution.scalarOps =
      match r with
      | .iBinOp | .iTestOp | .iRelOp => 1
      | _ => 0 := by
  cases r <;> rfl

/-- Every row's `vectorLaneOps` is zero, exactly: the declared executed subset
contains no SIMD rule.  The 128-bit lane law itself is
`Wasm.vectorLaneCharge` / `Wasm.wholeVectorShuffleCharge`, and
`Wasm.wholeVectorShuffleCharge_lanes` proves the whole-vector shuffle charges
sixteen lanes. -/
theorem canonicalCostTable_row_vectorLaneOps (r : RuleId) :
    (r.row).contribution.vectorLaneOps = 0 := by cases r <;> rfl

/-- Every row's `tableElementsAllocated` is zero, exactly: release validation
rejects every module carrying a table, element or data section
(`Wasm.Module.checkClosed`), so no table, element or data rule is reachable. -/
theorem canonicalCostTable_row_tableElementsAllocated (r : RuleId) :
    (r.row).contribution.tableElementsAllocated = 0 := by cases r <;> rfl

/-- The transferred byte counts of the completed memory rules are the exact ABI
width. -/
theorem canonicalCostTable_row_transfer :
    (RuleId.load.row).contribution.bytesRead = 4 ∧
      (RuleId.store.row).contribution.bytesWritten = 4 ∧
      (RuleId.returnGemm.row).contribution.outputBytes = 4 := by decide

/-- **Traps contribute the attempted rule step and no completed transfer.**
Every trapping or refusing rule stores a zero in every transfer coordinate. -/
theorem canonicalCostTable_row_trap_no_transfer (r : RuleId)
    (h : r = .iBinOpTrap ∨ r = .loadTrap ∨ r = .storeTrap ∨
      r = .memoryGrowRefuse ∨ r = .installTrap) :
    (r.row).contribution.bytesRead = 0 ∧
      (r.row).contribution.bytesWritten = 0 ∧
      (r.row).contribution.memoryGrowPages = 0 ∧
      (r.row).contribution.scalarOps = 0 ∧
      (r.row).contribution.wasmRuleSteps = 1 := by
  rcases h with h | h | h | h | h <;> subst h <;> exact ⟨rfl, rfl, rfl, rfl, rfl⟩

/-- The exception row carries the canonical abstract exception width, and it is
the same number `Wasm.exceptionObjectBytes` computes from the pinned GC layout:
24 abstract bytes, not a unit-cost free store. -/
theorem canonicalCostTable_row_exception :
    (RuleId.throwTag.row).contribution.gcObjectsAllocated = 1 ∧
      (RuleId.throwTag.row).contribution.gcBytesInitialized =
        exceptionObjectBytes canonicalGcLayout := by
  refine ⟨rfl, ?_⟩
  rw [canonical_exceptionObjectBytes]
  rfl

/-- The pinned transfer width really is the ABI image width. -/
theorem canonicalTransferBytes_eq_i32TransferBytes :
    canonicalTransferBytes = i32TransferBytes := rfl

/-! ### The harness initialization events -/

/-- The finite harness initialization events of SPEC section 7.5: exactly the
steps `Wasm.initialConfig` performs, and the three `Wasm.InstantiationFault`
outcomes it can report. -/
inductive InitEventId
  /-- Release validation of the decoded module. -/
  | validateModule
  /-- Allocation of memory zero at its declared minimum size. -/
  | allocateMemory
  /-- Allocation of the globals at their constant initializers. -/
  | allocateGlobals
  /-- Resolution of the exported `gemm` function index. -/
  | resolveGemmExport
  /-- Construction of the harness control frame. -/
  | buildHarnessFrame
  /-- Selection of the optional start function. -/
  | selectStartFunction
  /-- The module failed release validation. -/
  | faultInvalidModule
  /-- Allocation or static initialization failed. -/
  | faultAllocationFailed
  /-- The module has no exported `gemm` function. -/
  | faultMissingGemmExport
  deriving DecidableEq, Repr, Inhabited

namespace InitEventId

/-- The finite cover of the initialization event identifiers. -/
def all : List InitEventId :=
  [ .validateModule, .allocateMemory, .allocateGlobals, .resolveGemmExport
  , .buildHarnessFrame, .selectStartFunction
  , .faultInvalidModule, .faultAllocationFailed, .faultMissingGemmExport ]

theorem mem_all (e : InitEventId) : e ∈ all := by cases e <;> simp [all]

theorem all_nodup : all.Nodup := by decide

theorem all_length : all.length = 9 := rfl

/-- The three fault events are exactly the three `Wasm.InstantiationFault`
outcomes: the initialization event set is anchored to the machine, not
invented. -/
def ofFault : InstantiationFault → InitEventId
  | .invalidModule => .faultInvalidModule
  | .allocationFailed => .faultAllocationFailed
  | .missingGemmExport => .faultMissingGemmExport

theorem ofFault_injective : Function.Injective ofFault := by
  intro a b h
  cases a <;> cases b <;> first | rfl | exact absurd h (by decide)

/-- The pinned identifier of an initialization event. -/
def name : InitEventId → String
  | .validateModule => "core3/init/validate-module"
  | .allocateMemory => "core3/init/allocate-memory"
  | .allocateGlobals => "core3/init/allocate-globals"
  | .resolveGemmExport => "core3/init/resolve-gemm-export"
  | .buildHarnessFrame => "core3/init/build-harness-frame"
  | .selectStartFunction => "core3/init/select-start-function"
  | .faultInvalidModule => "core3/init/fault-invalid-module"
  | .faultAllocationFailed => "core3/init/fault-allocation-failed"
  | .faultMissingGemmExport => "core3/init/fault-missing-gemm-export"

/-- The pinned cost-table row of an initialization event. -/
def row : InitEventId → CostRuleRow
  | .validateModule =>
      canonicalRow "core3/init/validate-module" canonicalInstantiationContribution
  | .allocateMemory =>
      canonicalRow "core3/init/allocate-memory"
        { canonicalInstantiationContribution with memoryGrowPages := 1 }
  | .allocateGlobals =>
      canonicalRow "core3/init/allocate-globals"
        { canonicalInstantiationContribution with bytesWritten := 1 }
  | .resolveGemmExport =>
      canonicalRow "core3/init/resolve-gemm-export"
        canonicalInstantiationContribution
  | .buildHarnessFrame =>
      canonicalRow "core3/init/build-harness-frame"
        { canonicalInstantiationContribution with preparationSteps := 1 }
  | .selectStartFunction =>
      canonicalRow "core3/init/select-start-function"
        canonicalInstantiationContribution
  | .faultInvalidModule =>
      canonicalRow "core3/init/fault-invalid-module"
        canonicalInstantiationContribution
  | .faultAllocationFailed =>
      canonicalRow "core3/init/fault-allocation-failed"
        canonicalInstantiationContribution
  | .faultMissingGemmExport =>
      canonicalRow "core3/init/fault-missing-gemm-export"
        canonicalInstantiationContribution

@[simp] theorem row_ruleId (e : InitEventId) : (row e).ruleId = name e := by
  cases e <;> rfl

end InitEventId

/-- **SPEC section 7.5, exact cover of the initialization events.** -/
theorem canonicalCostTable_init_exact_cover :
    canonicalCostTableUnits.initializationRows =
      InitEventId.all.map InitEventId.row := rfl

theorem canonicalCostTable_init_exact_cover_ids :
    canonicalCostTableUnits.initializationRows.map CostRuleRow.ruleId =
      InitEventId.all.map InitEventId.name := rfl

theorem canonicalCostTable_initializationRows_nodup :
    (canonicalCostTableUnits.initializationRows.map CostRuleRow.ruleId).Nodup :=
  canonicalInitializationRows_nodup

theorem canonicalCostTable_initRowFor (e : InitEventId) :
    canonicalCostTableUnits.initRowFor? e.name = some e.row := by
  have hmem : e.row ∈ canonicalCostTableUnits.initializationRows := by
    rw [canonicalCostTable_init_exact_cover]
    exact List.mem_map_of_mem (InitEventId.mem_all e)
  have h :=
    CostTableBody.initRowFor?_eq_of_mem
      canonicalCostTable_initializationRows_nodup hmem
  rwa [InitEventId.row_ruleId] at h

/-- No two distinct initialization events share a pinned identifier. -/
theorem InitEventId.name_injective : Function.Injective InitEventId.name := by
  intro a b h
  have hnd : (InitEventId.all.map InitEventId.name).Nodup := by
    rw [← canonicalCostTable_init_exact_cover_ids]
    exact canonicalCostTable_initializationRows_nodup
  exact nodup_map_inj hnd (InitEventId.mem_all a) (InitEventId.mem_all b) h

/-- **SPEC section 7.5, coverage of the initialization events.** -/
theorem canonicalCostTable_covers_every_init_event (e : InitEventId) :
    (canonicalCostTableUnits.initRowFor? e.name).isSome := by
  rw [canonicalCostTable_initRowFor]; rfl

/-- Every initialization row charges one instantiation step and one rule step,
matching `Cost.Event.instantiationStep`, and no dispatch step: SPEC section 7.5
restricts `dispatchSteps` to the branch, call, return, exception-transfer and
phase-transition rules. -/
theorem canonicalCostTable_init_row_steps (e : InitEventId) :
    (e.row).contribution.instantiationSteps = 1 ∧
      (e.row).contribution.wasmRuleSteps = 1 ∧
      (e.row).contribution.dispatchSteps = 0 := by
  cases e <;> exact ⟨rfl, rfl, rfl⟩

/-- The initialization charge is exactly `Cost.Event.instantiationStep`'s: the
initialization rows do not invent a second accounting convention. -/
theorem canonicalInstantiationContribution_eq :
    canonicalInstantiationContribution = Cost.Event.instantiationStep.charge :=
  rfl

/-! ## The canonical declarative-validation derivation size (SPEC section 7.5)

`CostTableBody.validationCost` charges one unit per node and one per premise
edge of the canonical declarative-validation derivation, but takes the node and
edge counts as numbers.  This section computes them from the module, so that
the charge is a function of the artifact rather than of an oracle.

The derivation counted is the one `Wasm.DeclarativelyValid` is: a root node
with a premise edge to each of the five closed conditions (`checkClosed`,
`checkMems`, `exportsMemory`, `checkGemmExport`, `checkStart`), one to each
global's validity node, one to each tag's validity node and one to each
function's validity node; and every function node has one premise edge to the
root of its body's `Wasm.ExprTyping` derivation.  The `ExprTyping` and
`InstrTyping` node and edge counts below are read directly off the constructors
of those two mutual inductives: `ExprTyping.nil` is a leaf, `ExprTyping.cons`
has two premises, `InstrTyping.block` and `InstrTyping.loop` have one, and
`InstrTyping.ifThenElse` has two. -/

mutual

/-- Nodes of the canonical `Wasm.InstrTyping` derivation of one instruction.

Every structured control instruction recurses into its body, including
`try_table`, which `Wasm.InstrTyping` does not admit: counting its body keeps
the number an upper bound on the derivation of any extension of the subset
rather than a number that would shrink when the subset grows.  Every other
instruction outside the declared subset has no derivation at all and is counted
as the single (failing) leaf node the derivation attempt occupies.  The
function is therefore total on the whole syntax, valid or not. -/
def instrDerivationNodes : Instr → Nat
  | .block _ body => 1 + exprDerivationNodes body
  | .loop _ body => 1 + exprDerivationNodes body
  | .ifThenElse _ thenBody elseBody =>
      1 + exprDerivationNodes thenBody + exprDerivationNodes elseBody
  | .tryTable _ _ body => 1 + exprDerivationNodes body
  | _ => 1

/-- Nodes of the canonical `Wasm.ExprTyping` derivation of a sequence. -/
def exprDerivationNodes : Expr → Nat
  | .nil => 1
  | .cons i e => 1 + instrDerivationNodes i + exprDerivationNodes e

end

mutual

/-- Premise edges of the canonical `Wasm.InstrTyping` derivation of one
instruction. -/
def instrDerivationEdges : Instr → Nat
  | .block _ body => 1 + exprDerivationEdges body
  | .loop _ body => 1 + exprDerivationEdges body
  | .ifThenElse _ thenBody elseBody =>
      2 + exprDerivationEdges thenBody + exprDerivationEdges elseBody
  | .tryTable _ _ body => 1 + exprDerivationEdges body
  | _ => 0

/-- Premise edges of the canonical `Wasm.ExprTyping` derivation of a
sequence. -/
def exprDerivationEdges : Expr → Nat
  | .nil => 0
  | .cons i e => 2 + instrDerivationEdges i + exprDerivationEdges e

end

theorem instrDerivationNodes_pos (i : Instr) : 0 < instrDerivationNodes i := by
  cases i <;> simp [instrDerivationNodes] <;> omega

theorem exprDerivationNodes_pos (e : Expr) : 0 < exprDerivationNodes e := by
  cases e <;> simp [exprDerivationNodes] <;> omega

/-- A longer instruction sequence has a strictly larger derivation: the node
count is monotone in the syntax, not a constant. -/
theorem exprDerivationNodes_lt_cons (i : Instr) (e : Expr) :
    exprDerivationNodes e < exprDerivationNodes (.cons i e) := by
  have h := instrDerivationNodes_pos i
  simp [exprDerivationNodes]
  omega

theorem exprDerivationEdges_le_cons (i : Instr) (e : Expr) :
    exprDerivationEdges e ≤ exprDerivationEdges (.cons i e) := by
  simp [exprDerivationEdges]

namespace Module

/-- The node count of the canonical declarative-validation derivation. -/
def validationNodes (m : Module) : Nat :=
  6 + m.globals.length + m.tags.length +
    (m.funcs.map (fun f => 1 + exprDerivationNodes f.body)).sum

/-- The premise-edge count of the canonical declarative-validation
derivation. -/
def validationEdges (m : Module) : Nat :=
  5 + m.globals.length + m.tags.length +
    (m.funcs.map (fun f => 1 + exprDerivationEdges f.body)).sum

theorem validationNodes_pos (m : Module) : 0 < m.validationNodes := by
  unfold validationNodes; omega

theorem validationNodes_cons_global (m : Module) (g : Global) :
    { m with globals := g :: m.globals }.validationNodes = m.validationNodes + 1 := by
  unfold validationNodes
  simp
  omega

theorem validationNodes_cons_tag (m : Module) (t : TagType) :
    { m with tags := t :: m.tags }.validationNodes = m.validationNodes + 1 := by
  unfold validationNodes
  simp
  omega

theorem validationNodes_cons_func (m : Module) (f : Func) :
    { m with funcs := f :: m.funcs }.validationNodes =
      m.validationNodes + (1 + exprDerivationNodes f.body) := by
  unfold validationNodes
  simp [List.sum_cons]
  omega

theorem validationEdges_cons_func (m : Module) (f : Func) :
    { m with funcs := f :: m.funcs }.validationEdges =
      m.validationEdges + (1 + exprDerivationEdges f.body) := by
  unfold validationEdges
  simp [List.sum_cons]
  omega

end Module

/-- **SPEC section 7.5, `Wasm.validationCost`.**  One unit per node plus one
unit per premise edge of the canonical declarative-validation derivation of
`m`, charged at the profile's node and edge units. -/
def validationCost (t : CostTableBody) (m : Module) : Nat :=
  t.validationCost m.validationNodes m.validationEdges

/-- Totality: `validationCost` is a total function of the cost table and the
module.  It is not partial, not `Option`-valued and has no side condition; it
is defined on every module, valid or not. -/
theorem validationCost_total (t : CostTableBody) (m : Module) :
    ∃ n : Nat, validationCost t m = n := ⟨_, rfl⟩

/-- Under any lawful profile the charge is exactly nodes plus edges. -/
theorem validationCost_module_eq (P : Profile) (m : Module) :
    validationCost P.costTableBody m = m.validationNodes + m.validationEdges :=
  P.validationCost_eq _ _

/-- **Monotone in the node and edge counts.** -/
theorem validationCost_mono (t : CostTableBody) {m m' : Module}
    (hn : m.validationNodes ≤ m'.validationNodes)
    (he : m.validationEdges ≤ m'.validationEdges) :
    validationCost t m ≤ validationCost t m' :=
  Nat.add_le_add (Nat.mul_le_mul_left _ hn) (Nat.mul_le_mul_left _ he)

/-- **Monotone in the syntax: adding a function never lowers the charge.** -/
theorem validationCost_le_cons_func (t : CostTableBody) (m : Module) (f : Func) :
    validationCost t m ≤ validationCost t { m with funcs := f :: m.funcs } := by
  refine validationCost_mono t ?_ ?_
  · rw [Module.validationNodes_cons_func]; omega
  · rw [Module.validationEdges_cons_func]; omega

/-- **Monotone in the syntax: adding a global never lowers the charge.** -/
theorem validationCost_le_cons_global (t : CostTableBody) (m : Module)
    (g : Global) :
    validationCost t m ≤
      validationCost t { m with globals := g :: m.globals } := by
  refine validationCost_mono t ?_ ?_
  · rw [Module.validationNodes_cons_global]; omega
  · unfold Module.validationEdges; simp

/-- **Monotone in the syntax: adding a tag never lowers the charge.** -/
theorem validationCost_le_cons_tag (t : CostTableBody) (m : Module)
    (tag : TagType) :
    validationCost t m ≤ validationCost t { m with tags := tag :: m.tags } := by
  refine validationCost_mono t ?_ ?_
  · rw [Module.validationNodes_cons_tag]; omega
  · unfold Module.validationEdges; simp

/-- **The charge is never silently zero.**  Under any lawful profile every
module — in particular every module that validates — costs at least one
validation unit: the root of the derivation is always there. -/
theorem validationCost_pos (P : Profile) (m : Module) :
    0 < validationCost P.costTableBody m := by
  rw [validationCost_module_eq]
  have := m.validationNodes_pos
  omega

/-- The statement in the form SPEC section 7.5 asks for: a validating module
costs at least one unit. -/
theorem validationCost_pos_of_valid (P : Profile) (m : Module)
    (_ : DeclarativelyValid m) : 0 < validationCost P.costTableBody m :=
  validationCost_pos P m

/-- Nor is it a constant: a module with one more function strictly costs
more. -/
theorem validationCost_lt_cons_func (P : Profile) (m : Module) (f : Func) :
    validationCost P.costTableBody m <
      validationCost P.costTableBody { m with funcs := f :: m.funcs } := by
  rw [validationCost_module_eq, validationCost_module_eq,
    Module.validationNodes_cons_func,
    Module.validationEdges_cons_func]
  omega

/-! ## The static bytes instantiation materialises (SPEC section 7.5) -/

/-- The abstract static bytes of one data segment: its raw initializer. -/
def dataStaticBytes (d : Data) : Nat := d.init.length

/-- The abstract static bytes of one element segment: one reference per
element. -/
def elemStaticBytes (P : Profile) (e : Elem) : Nat :=
  e.init.length * P.costTableBody.layout.referenceWidth

/-- The abstract static bytes of one global: the canonical width of its value
type. -/
def globalStaticBytes (P : Profile) (g : Global) : Nat :=
  P.costTableBody.layout.valTypeWidth g.type.valType

/-- The abstract static bytes of one declared memory: its declared initial
size, in bytes. -/
def memStaticBytes (mem : Mem) : Nat := mem.type.limits.min * pageSize

/-- The abstract static bytes of one declared table: one reference per declared
initial element. -/
def tableStaticBytes (P : Profile) (tb : Table) : Nat :=
  tb.type.limits.min * P.costTableBody.layout.referenceWidth

/-- **SPEC section 7.5, `Wasm.instantiatedStaticBytes`.**  The static bytes
instantiation materialises: the data segments, the element segments, the
globals, and the declared initial sizes of the memories and tables. -/
def instantiatedStaticBytes (P : Profile) (m : Module) : Nat :=
  (m.datas.map dataStaticBytes).sum +
  (m.elems.map (elemStaticBytes P)).sum +
  (m.globals.map (globalStaticBytes P)).sum +
  (m.mems.map memStaticBytes).sum +
  (m.tables.map (tableStaticBytes P)).sum

/-- Totality: defined on every profile and every module, with no side
condition. -/
theorem instantiatedStaticBytes_total (P : Profile) (m : Module) :
    ∃ n : Nat, instantiatedStaticBytes P m = n := ⟨_, rfl⟩

theorem instantiatedStaticBytes_cons_data (P : Profile) (m : Module) (d : Data) :
    instantiatedStaticBytes P { m with datas := d :: m.datas } =
      instantiatedStaticBytes P m + dataStaticBytes d := by
  unfold instantiatedStaticBytes
  simp [List.sum_cons]
  omega

theorem instantiatedStaticBytes_cons_global (P : Profile) (m : Module)
    (g : Global) :
    instantiatedStaticBytes P { m with globals := g :: m.globals } =
      instantiatedStaticBytes P m + globalStaticBytes P g := by
  unfold instantiatedStaticBytes
  simp [List.sum_cons]
  omega

theorem instantiatedStaticBytes_cons_mem (P : Profile) (m : Module) (mem : Mem) :
    instantiatedStaticBytes P { m with mems := mem :: m.mems } =
      instantiatedStaticBytes P m + memStaticBytes mem := by
  unfold instantiatedStaticBytes
  simp [List.sum_cons]
  omega

theorem instantiatedStaticBytes_cons_elem (P : Profile) (m : Module) (e : Elem) :
    instantiatedStaticBytes P { m with elems := e :: m.elems } =
      instantiatedStaticBytes P m + elemStaticBytes P e := by
  unfold instantiatedStaticBytes
  simp [List.sum_cons]
  omega

theorem instantiatedStaticBytes_cons_table (P : Profile) (m : Module)
    (tb : Table) :
    instantiatedStaticBytes P { m with tables := tb :: m.tables } =
      instantiatedStaticBytes P m + tableStaticBytes P tb := by
  unfold instantiatedStaticBytes
  simp [List.sum_cons]
  omega

/-- **Monotone: materialising one more data segment never lowers the count.** -/
theorem instantiatedStaticBytes_le_cons_data (P : Profile) (m : Module)
    (d : Data) :
    instantiatedStaticBytes P m ≤
      instantiatedStaticBytes P { m with datas := d :: m.datas } := by
  rw [instantiatedStaticBytes_cons_data]; omega

/-- **Monotone: one more global never lowers the count.** -/
theorem instantiatedStaticBytes_le_cons_global (P : Profile) (m : Module)
    (g : Global) :
    instantiatedStaticBytes P m ≤
      instantiatedStaticBytes P { m with globals := g :: m.globals } := by
  rw [instantiatedStaticBytes_cons_global]; omega

/-- **Monotone: one more declared memory never lowers the count.** -/
theorem instantiatedStaticBytes_le_cons_mem (P : Profile) (m : Module)
    (mem : Mem) :
    instantiatedStaticBytes P m ≤
      instantiatedStaticBytes P { m with mems := mem :: m.mems } := by
  rw [instantiatedStaticBytes_cons_mem]; omega

/-- **Monotone: one more element segment never lowers the count.** -/
theorem instantiatedStaticBytes_le_cons_elem (P : Profile) (m : Module)
    (e : Elem) :
    instantiatedStaticBytes P m ≤
      instantiatedStaticBytes P { m with elems := e :: m.elems } := by
  rw [instantiatedStaticBytes_cons_elem]; omega

/-- **Monotone: one more declared table never lowers the count.** -/
theorem instantiatedStaticBytes_le_cons_table (P : Profile) (m : Module)
    (tb : Table) :
    instantiatedStaticBytes P m ≤
      instantiatedStaticBytes P { m with tables := tb :: m.tables } := by
  rw [instantiatedStaticBytes_cons_table]; omega

/-- Strictly monotone where it must be: a global really is charged, because
every canonical width is positive. -/
theorem instantiatedStaticBytes_lt_cons_global (P : Profile) (m : Module)
    (g : Global) :
    instantiatedStaticBytes P m <
      instantiatedStaticBytes P { m with globals := g :: m.globals } := by
  rw [instantiatedStaticBytes_cons_global]
  have h : 0 < globalStaticBytes P g :=
    GcLayoutConstants.valTypeWidth_pos P.layout_positive _
  omega

/-- **Adequacy: the count is not an understatement of what allocation really
materialises.**  A successfully allocated store's memory is exactly the
declared minimum size, and that is one of the summands. -/
theorem allocated_memory_size_le_instantiatedStaticBytes (P : Profile)
    {m : Module} {s : Store} {mem : Mem} (hm : m.mems = [mem])
    (h : Store.alloc m = some s) :
    s.memory.size ≤ instantiatedStaticBytes P m := by
  rw [Store.alloc_memory_size hm h]
  unfold instantiatedStaticBytes
  rw [hm]
  simp [memStaticBytes]
  omega

end WasmGemmGnaf.Wasm
