/-
  Charged cost events.
  Normative source: SPEC.md section 9 ("Every artifact event is derived from a
  formal WebAssembly trace or canonical module byte object.  Packing, layout
  conversion, dispatch, table lookup, any verification performed by the module,
  and output writes are charged.") and section 9.3 ("positive accounting for
  every semantic transition").

  Every declaration in this file is proved. Nothing is assumed.
-/
import WasmGemmGnaf.Cost.Vector

set_option autoImplicit false

namespace WasmGemmGnaf.Cost

/--
  A single charged event of the UOR-Wasm cost model.

  Every constructor denotes one semantic transition of a costed execution, and
  therefore charges at least one `wasmRuleSteps` in addition to the resource it
  names.  There is no uncharged event: an execution step that does not appear
  here cannot be performed by a costed trace.
-/
inductive Event
  /-- One instantiation step (store allocation, element/data initialisation). -/
  | instantiationStep
  /-- One dispatch step (indirect call, table lookup, entry selection). -/
  | dispatchStep
  /-- One preparation step (packing, layout conversion, descriptor decoding). -/
  | preparationStep
  /-- One plain WebAssembly reduction step. -/
  | ruleStep
  /-- One scalar arithmetic operation. -/
  | scalarOp
  /-- One vector operation over `lanes` lanes. -/
  | vectorLaneOp (lanes : Nat)
  /-- A memory/table read of `count` bytes. -/
  | readBytes (count : Nat)
  /-- A memory/table write of `count` bytes. -/
  | writeBytes (count : Nat)
  /-- `memory.grow` by `pages`, leaving `resultingPages` pages allocated. -/
  | growMemory (pages : Nat) (resultingPages : Nat)
  /-- Allocation of `count` table elements. -/
  | allocateTableElements (count : Nat)
  /-- Allocation of `count` GC objects, initialising `initializedBytes`, leaving
      `resultingLiveBytes` bytes live. -/
  | allocateGcObjects (count : Nat) (initializedBytes : Nat)
      (resultingLiveBytes : Nat)
  /-- A stack push leaving `depth` values on the operand stack. -/
  | pushStackValues (depth : Nat)
  /-- Emission of `count` observable output bytes. -/
  | emitOutput (count : Nat)
  deriving Repr, DecidableEq, Inhabited

namespace Event

/-- The dynamic cost charged by a single event. -/
def charge : Event → DynamicVector
  | .instantiationStep =>
      { DynamicVector.zero with instantiationSteps := 1, wasmRuleSteps := 1 }
  | .dispatchStep =>
      { DynamicVector.zero with dispatchSteps := 1, wasmRuleSteps := 1 }
  | .preparationStep =>
      { DynamicVector.zero with preparationSteps := 1, wasmRuleSteps := 1 }
  | .ruleStep =>
      { DynamicVector.zero with wasmRuleSteps := 1 }
  | .scalarOp =>
      { DynamicVector.zero with scalarOps := 1, wasmRuleSteps := 1 }
  | .vectorLaneOp lanes =>
      { DynamicVector.zero with vectorLaneOps := lanes, wasmRuleSteps := 1 }
  | .readBytes count =>
      { DynamicVector.zero with bytesRead := count, wasmRuleSteps := 1 }
  | .writeBytes count =>
      { DynamicVector.zero with bytesWritten := count, wasmRuleSteps := 1 }
  | .growMemory pages resultingPages =>
      { DynamicVector.zero with
          memoryGrowPages := pages, peakPages := resultingPages
          wasmRuleSteps := 1 }
  | .allocateTableElements count =>
      { DynamicVector.zero with
          tableElementsAllocated := count, wasmRuleSteps := 1 }
  | .allocateGcObjects count initializedBytes resultingLiveBytes =>
      { DynamicVector.zero with
          gcObjectsAllocated := count
          gcBytesInitialized := initializedBytes
          peakGcLiveBytes := resultingLiveBytes
          wasmRuleSteps := 1 }
  | .pushStackValues depth =>
      { DynamicVector.zero with peakStackValues := depth, wasmRuleSteps := 1 }
  | .emitOutput count =>
      { DynamicVector.zero with outputBytes := count, wasmRuleSteps := 1 }

/--
  The weight an event claims: the additive part of its charge.  The three peak
  coordinates are deliberately excluded, because `sequentialCompose` maximises
  rather than accumulates them; a peak claims no additive weight.
-/
def weight (e : Event) : Nat := e.charge.additiveTotal

/-- Every event charges at least one WebAssembly rule step. -/
theorem one_le_charge_wasmRuleSteps (e : Event) : 1 ≤ e.charge.wasmRuleSteps := by
  cases e <;> exact Nat.le_refl 1

/-- Every charged event claims a strictly positive weight. -/
theorem one_le_weight (e : Event) : 1 ≤ e.weight := by
  cases e <;> simp only [weight, charge, DynamicVector.additiveTotal,
    DynamicVector.zero] <;> omega

theorem weight_le_charge_total (e : Event) : e.weight ≤ e.charge.total :=
  DynamicVector.additiveTotal_le_total e.charge

end Event

/--
  SPEC 9.3, positive accounting: composing a charged event onto a running cost
  increases the running total by at least the weight that event claims.  No
  charged transition can be absorbed for free.
-/
theorem transition_accounting_positive (v : DynamicVector) (e : Event) :
    v.total + e.weight ≤ (sequentialCompose v e.charge).total :=
  total_sequentialCompose_ge v e.charge

/-- Consequently every charged transition strictly increases the running
total. -/
theorem transition_accounting_strict (v : DynamicVector) (e : Event) :
    v.total < (sequentialCompose v e.charge).total :=
  Nat.lt_of_lt_of_le
    (Nat.lt_of_lt_of_le (Nat.lt_succ_self _)
      (Nat.add_le_add_left (Event.one_le_weight e) v.total))
    (transition_accounting_positive v e)

/-- A charged transition never decreases any coordinate. -/
theorem transition_monotone (v : DynamicVector) (e : Event) :
    DynamicVector.ComponentwiseLE v (sequentialCompose v e.charge) :=
  sequentialCompose_le_left v e.charge

end WasmGemmGnaf.Cost
