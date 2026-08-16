/-
  Wasm/Step.lean --- the relational reduction and its successor enumerator.

  Normative source: SPEC.md section 7.1 ("`Step` owns every reduction rule";
  "`successors` enumerates every permitted nondeterministic successor.  A single
  evaluator path is not a replacement for this list.") and section 7.4.

  Two objects are defined here and proved equal in extension:

  * `Wasm.Step : Config → Event → Config → Prop`, an inductive relation with one
    constructor per reduction rule, each stated by equations on the fields of
    the configuration it reduces; and
  * `Wasm.successors : Config → List (Event × Config)`, an executable
    enumeration.

  `mem_successors_iff_step` is the anti-cheat property: the list is exactly the
  relation, so no evaluator path can stand in for the nondeterministic relation
  and no rule can be silently dropped from the enumerator.  `memory.grow` is the
  released profile's permitted nondeterminism and genuinely produces two
  successors whenever growth is possible; `successors_nodup` shows the two are
  distinct rather than a duplicated single outcome.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Config

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## The executable successor enumerator -/

/-- The successors of a branch to label `n`, with `st` the operand stack after
the branch instruction has consumed its operands. -/
def branchTo (c : Config) (st : List UInt32) (n : Nat) : List (Event × Config) :=
  match c.ctrl[n]? with
  | none => []
  | some k =>
      if k.isLoop = true then
        [(Event.branch,
          { c with code := k.body, ctrl := k :: c.ctrl.drop (n + 1)
                   stack := truncStack st k.height })]
      else
        [(Event.branch,
          { c with code := k.cont, ctrl := c.ctrl.drop (n + 1)
                   stack := truncStack st k.height })]

/-- The successors of a configuration whose instruction sequence is exhausted:
exit a label, cross the harness entry boundary, or return. -/
def successorsAtEnd (c : Config) : List (Event × Config) :=
  match c.ctrl with
  | k :: ks => [(Event.step, { c with code := k.cont, ctrl := ks })]
  | [] =>
      match c.phase with
      | .afterEntry =>
          [(Event.returnEvent (c.stack.headD 0),
            { c with status := .returned (c.stack.headD 0) })]
      | .beforeEntry =>
          match c.store.storeBytes c.harness.rawPtr c.harness.rawBytes with
          | some s' =>
              [(Event.enterGemm,
                { c with store := s'
                         locals := c.harness.args ++
                           List.replicate c.harness.gemmNumLocals 0
                         stack := [], code := c.harness.gemmBody, ctrl := []
                         phase := .afterEntry, entry? := some s'.observable })]
          | none =>
              [(Event.trapEvent .outOfBounds,
                { c with status := .trapped .outOfBounds })]

/-- The successors of a configuration whose next instruction is `i`. -/
def successorsOfInstr (c : Config) (i : Instr) (rest : List Instr) :
    List (Event × Config) :=
  match i with
  | .unreachable =>
      [(Event.trapEvent .unreachable, { c with status := .trapped .unreachable })]
  | .nop => [(Event.step, { c with code := rest })]
  | .i32Const v =>
      [(Event.step, { c with code := rest, stack := wrapI32 v :: c.stack })]
  | .drop =>
      match c.stack with
      | _ :: st => [(Event.step, { c with code := rest, stack := st })]
      | [] => []
  | .iBinOp w op =>
      match w, c.stack with
      | .i32, b :: a :: st =>
          match evalIBinOp op a b with
          | some (some v) => [(Event.step, { c with code := rest, stack := v :: st })]
          | some none =>
              [(Event.trapEvent .divideByZero,
                { c with status := .trapped .divideByZero })]
          | none => []
      | _, _ => []
  | .iTestOp w op =>
      match w, op, c.stack with
      | .i32, .eqz, a :: st =>
          [(Event.step, { c with code := rest, stack := Num.i32Eqz a :: st })]
      | _, _, _ => []
  | .iRelOp w op =>
      match w, c.stack with
      | .i32, b :: a :: st =>
          [(Event.step, { c with code := rest, stack := evalIRelOp op a b :: st })]
      | _, _ => []
  | .localGet idx =>
      match c.locals[idx]? with
      | some v => [(Event.step, { c with code := rest, stack := v :: c.stack })]
      | none => []
  | .localSet idx =>
      match c.stack, c.locals[idx]? with
      | v :: st, some _ =>
          [(Event.step,
            { c with code := rest, stack := st, locals := c.locals.set idx v })]
      | _, _ => []
  | .localTee idx =>
      match c.stack, c.locals[idx]? with
      | v :: _, some _ =>
          [(Event.step, { c with code := rest, locals := c.locals.set idx v })]
      | _, _ => []
  | .globalGet idx =>
      match c.store.globals[idx]? with
      | some g => [(Event.step, { c with code := rest, stack := g.value :: c.stack })]
      | none => []
  | .globalSet idx =>
      match c.stack, c.store.globals[idx]? with
      | v :: st, some g =>
          [(Event.step,
            { c with code := rest, stack := st
                     store := { c.store with
                       globals := c.store.globals.set idx { g with value := v } } })]
      | _, _ => []
  | .load t narrow arg =>
      match t, narrow, c.stack with
      | .i32, none, a :: st =>
          match c.store.loadBytes (a.toNat + arg.offset) 4 with
          | some bs =>
              [(Event.step, { c with code := rest, stack := bytesToU32 bs :: st })]
          | none =>
              [(Event.trapEvent .outOfBounds,
                { c with status := .trapped .outOfBounds })]
      | _, _, _ => []
  | .store t narrow arg =>
      match t, narrow, c.stack with
      | .i32, none, v :: a :: st =>
          match c.store.storeBytes (a.toNat + arg.offset) (u32ToBytes v) with
          | some s' =>
              [(Event.step, { c with code := rest, stack := st, store := s' })]
          | none =>
              [(Event.trapEvent .outOfBounds,
                { c with status := .trapped .outOfBounds })]
      | _, _, _ => []
  | .memorySize mem =>
      match mem with
      | 0 =>
          [(Event.step,
            { c with code := rest
                     stack := UInt32.ofNat c.store.memory.pages :: c.stack })]
      | _ + 1 => []
  | .memoryGrow mem =>
      match mem, c.stack with
      | 0, d :: st =>
          match c.store.memory.grow d.toNat with
          | some m' =>
              [(Event.growAttempt (.grown c.store.memory.pages),
                { c with code := rest
                         stack := UInt32.ofNat c.store.memory.pages :: st
                         store := { c.store with memory := m' } }),
               (Event.growAttempt .refused,
                { c with code := rest, stack := 4294967295 :: st })]
          | none =>
              [(Event.growAttempt .refused,
                { c with code := rest, stack := 4294967295 :: st })]
      | _, _ => []
  | .block bt body =>
      match bt with
      | .empty =>
          [(Event.step,
            { c with code := body.toList
                     ctrl := { isLoop := false, body := body.toList, cont := rest
                               height := c.stack.length } :: c.ctrl })]
      | _ => []
  | .loop bt body =>
      match bt with
      | .empty =>
          [(Event.step,
            { c with code := body.toList
                     ctrl := { isLoop := true, body := body.toList, cont := rest
                               height := c.stack.length } :: c.ctrl })]
      | _ => []
  | .ifThenElse bt thenBody elseBody =>
      match bt, c.stack with
      | .empty, cnd :: st =>
          if cnd = 0 then
            [(Event.branch,
              { c with code := elseBody.toList, stack := st
                       ctrl := { isLoop := false, body := elseBody.toList
                                 cont := rest, height := st.length } :: c.ctrl })]
          else
            [(Event.branch,
              { c with code := thenBody.toList, stack := st
                       ctrl := { isLoop := false, body := thenBody.toList
                                 cont := rest, height := st.length } :: c.ctrl })]
      | _, _ => []
  | .br n => branchTo c c.stack n
  | .brIf n =>
      match c.stack with
      | cnd :: st =>
          if cnd = 0 then [(Event.step, { c with code := rest, stack := st })]
          else branchTo c st n
      | [] => []
  | .throw tag =>
      match c.stack with
      | v :: _ =>
          [(Event.throwEvent ⟨tag, v⟩,
            { c with status := .thrown ⟨tag, v⟩ })]
      | [] => []
  | _ => []

/-- **SPEC section 7.1.**  Every permitted nondeterministic successor of a
configuration. -/
def successors (c : Config) : List (Event × Config) :=
  if c.status = Status.running then
    match c.code with
    | [] => successorsAtEnd c
    | i :: rest => successorsOfInstr c i rest
  else []

/-! ## The relational reduction -/

/-- The reduction relation of the released machine.  Every constructor is one
Core rule, stated by equations on the fields of the configuration it reduces.
`memoryGrowSucceed` and `memoryGrowRefuse` are simultaneously applicable
whenever growth is possible: that is the profile's permitted nondeterminism. -/
inductive Step : Config → Event → Config → Prop
  | unreachable {c rest} (hs : c.status = Status.running)
      (hc : c.code = Instr.unreachable :: rest) :
      Step c (Event.trapEvent .unreachable) { c with status := .trapped .unreachable }
  | nop {c rest} (hs : c.status = Status.running) (hc : c.code = Instr.nop :: rest) :
      Step c Event.step { c with code := rest }
  | i32Const {c rest v} (hs : c.status = Status.running)
      (hc : c.code = Instr.i32Const v :: rest) :
      Step c Event.step { c with code := rest, stack := wrapI32 v :: c.stack }
  | drop {c rest x st} (hs : c.status = Status.running)
      (hc : c.code = Instr.drop :: rest) (hst : c.stack = x :: st) :
      Step c Event.step { c with code := rest, stack := st }
  | iBinOp {c rest op a b st v} (hs : c.status = Status.running)
      (hc : c.code = Instr.iBinOp .i32 op :: rest) (hst : c.stack = b :: a :: st)
      (hv : evalIBinOp op a b = some (some v)) :
      Step c Event.step { c with code := rest, stack := v :: st }
  | iBinOpTrap {c rest op a b st} (hs : c.status = Status.running)
      (hc : c.code = Instr.iBinOp .i32 op :: rest) (hst : c.stack = b :: a :: st)
      (hv : evalIBinOp op a b = some none) :
      Step c (Event.trapEvent .divideByZero)
        { c with status := .trapped .divideByZero }
  | iTestOp {c rest a st} (hs : c.status = Status.running)
      (hc : c.code = Instr.iTestOp .i32 .eqz :: rest) (hst : c.stack = a :: st) :
      Step c Event.step { c with code := rest, stack := Num.i32Eqz a :: st }
  | iRelOp {c rest op a b st} (hs : c.status = Status.running)
      (hc : c.code = Instr.iRelOp .i32 op :: rest) (hst : c.stack = b :: a :: st) :
      Step c Event.step { c with code := rest, stack := evalIRelOp op a b :: st }
  | localGet {c rest idx v} (hs : c.status = Status.running)
      (hc : c.code = Instr.localGet idx :: rest) (hv : c.locals[idx]? = some v) :
      Step c Event.step { c with code := rest, stack := v :: c.stack }
  | localSet {c rest idx v st w} (hs : c.status = Status.running)
      (hc : c.code = Instr.localSet idx :: rest) (hst : c.stack = v :: st)
      (hw : c.locals[idx]? = some w) :
      Step c Event.step
        { c with code := rest, stack := st, locals := c.locals.set idx v }
  | localTee {c rest idx v st w} (hs : c.status = Status.running)
      (hc : c.code = Instr.localTee idx :: rest) (hst : c.stack = v :: st)
      (hw : c.locals[idx]? = some w) :
      Step c Event.step { c with code := rest, locals := c.locals.set idx v }
  | globalGet {c rest idx g} (hs : c.status = Status.running)
      (hc : c.code = Instr.globalGet idx :: rest)
      (hg : c.store.globals[idx]? = some g) :
      Step c Event.step { c with code := rest, stack := g.value :: c.stack }
  | globalSet {c rest idx v st g} (hs : c.status = Status.running)
      (hc : c.code = Instr.globalSet idx :: rest) (hst : c.stack = v :: st)
      (hg : c.store.globals[idx]? = some g) :
      Step c Event.step
        { c with code := rest, stack := st
                 store := { c.store with
                   globals := c.store.globals.set idx { g with value := v } } }
  | load {c rest arg a st bs} (hs : c.status = Status.running)
      (hc : c.code = Instr.load .i32 none arg :: rest) (hst : c.stack = a :: st)
      (hb : c.store.loadBytes (a.toNat + arg.offset) 4 = some bs) :
      Step c Event.step { c with code := rest, stack := bytesToU32 bs :: st }
  | loadTrap {c rest arg a st} (hs : c.status = Status.running)
      (hc : c.code = Instr.load .i32 none arg :: rest) (hst : c.stack = a :: st)
      (hb : c.store.loadBytes (a.toNat + arg.offset) 4 = none) :
      Step c (Event.trapEvent .outOfBounds) { c with status := .trapped .outOfBounds }
  | store {c rest arg v a st s'} (hs : c.status = Status.running)
      (hc : c.code = Instr.store .i32 none arg :: rest)
      (hst : c.stack = v :: a :: st)
      (hb : c.store.storeBytes (a.toNat + arg.offset) (u32ToBytes v) = some s') :
      Step c Event.step { c with code := rest, stack := st, store := s' }
  | storeTrap {c rest arg v a st} (hs : c.status = Status.running)
      (hc : c.code = Instr.store .i32 none arg :: rest)
      (hst : c.stack = v :: a :: st)
      (hb : c.store.storeBytes (a.toNat + arg.offset) (u32ToBytes v) = none) :
      Step c (Event.trapEvent .outOfBounds) { c with status := .trapped .outOfBounds }
  | memorySize {c rest} (hs : c.status = Status.running)
      (hc : c.code = Instr.memorySize 0 :: rest) :
      Step c Event.step
        { c with code := rest, stack := UInt32.ofNat c.store.memory.pages :: c.stack }
  | memoryGrowSucceed {c rest d st m'} (hs : c.status = Status.running)
      (hc : c.code = Instr.memoryGrow 0 :: rest) (hst : c.stack = d :: st)
      (hg : c.store.memory.grow d.toNat = some m') :
      Step c (Event.growAttempt (.grown c.store.memory.pages))
        { c with code := rest, stack := UInt32.ofNat c.store.memory.pages :: st
                 store := { c.store with memory := m' } }
  | memoryGrowRefuse {c rest d st} (hs : c.status = Status.running)
      (hc : c.code = Instr.memoryGrow 0 :: rest) (hst : c.stack = d :: st) :
      Step c (Event.growAttempt .refused)
        { c with code := rest, stack := 4294967295 :: st }
  | block {c rest body} (hs : c.status = Status.running)
      (hc : c.code = Instr.block .empty body :: rest) :
      Step c Event.step
        { c with code := body.toList
                 ctrl := { isLoop := false, body := body.toList, cont := rest
                           height := c.stack.length } :: c.ctrl }
  | loop {c rest body} (hs : c.status = Status.running)
      (hc : c.code = Instr.loop .empty body :: rest) :
      Step c Event.step
        { c with code := body.toList
                 ctrl := { isLoop := true, body := body.toList, cont := rest
                           height := c.stack.length } :: c.ctrl }
  | ifFalse {c rest thenBody elseBody cnd st} (hs : c.status = Status.running)
      (hc : c.code = Instr.ifThenElse .empty thenBody elseBody :: rest)
      (hst : c.stack = cnd :: st) (hcnd : cnd = 0) :
      Step c Event.branch
        { c with code := elseBody.toList, stack := st
                 ctrl := { isLoop := false, body := elseBody.toList, cont := rest
                           height := st.length } :: c.ctrl }
  | ifTrue {c rest thenBody elseBody cnd st} (hs : c.status = Status.running)
      (hc : c.code = Instr.ifThenElse .empty thenBody elseBody :: rest)
      (hst : c.stack = cnd :: st) (hcnd : ¬ cnd = 0) :
      Step c Event.branch
        { c with code := thenBody.toList, stack := st
                 ctrl := { isLoop := false, body := thenBody.toList, cont := rest
                           height := st.length } :: c.ctrl }
  | brLoop {c rest n k} (hs : c.status = Status.running)
      (hc : c.code = Instr.br n :: rest) (hk : c.ctrl[n]? = some k)
      (hl : k.isLoop = true) :
      Step c Event.branch
        { c with code := k.body, ctrl := k :: c.ctrl.drop (n + 1)
                 stack := truncStack c.stack k.height }
  | brBlock {c rest n k} (hs : c.status = Status.running)
      (hc : c.code = Instr.br n :: rest) (hk : c.ctrl[n]? = some k)
      (hl : ¬ k.isLoop = true) :
      Step c Event.branch
        { c with code := k.cont, ctrl := c.ctrl.drop (n + 1)
                 stack := truncStack c.stack k.height }
  | brIfFalse {c rest n cnd st} (hs : c.status = Status.running)
      (hc : c.code = Instr.brIf n :: rest) (hst : c.stack = cnd :: st)
      (hcnd : cnd = 0) :
      Step c Event.step { c with code := rest, stack := st }
  | brIfLoop {c rest n cnd st k} (hs : c.status = Status.running)
      (hc : c.code = Instr.brIf n :: rest) (hst : c.stack = cnd :: st)
      (hcnd : ¬ cnd = 0) (hk : c.ctrl[n]? = some k) (hl : k.isLoop = true) :
      Step c Event.branch
        { c with code := k.body, ctrl := k :: c.ctrl.drop (n + 1)
                 stack := truncStack st k.height }
  | brIfBlock {c rest n cnd st k} (hs : c.status = Status.running)
      (hc : c.code = Instr.brIf n :: rest) (hst : c.stack = cnd :: st)
      (hcnd : ¬ cnd = 0) (hk : c.ctrl[n]? = some k) (hl : ¬ k.isLoop = true) :
      Step c Event.branch
        { c with code := k.cont, ctrl := c.ctrl.drop (n + 1)
                 stack := truncStack st k.height }
  | throwTag {c rest tag v st} (hs : c.status = Status.running)
      (hc : c.code = Instr.throw tag :: rest) (hst : c.stack = v :: st) :
      Step c (Event.throwEvent ⟨tag, v⟩) { c with status := .thrown ⟨tag, v⟩ }
  | exitLabel {c k ks} (hs : c.status = Status.running) (hc : c.code = [])
      (hk : c.ctrl = k :: ks) :
      Step c Event.step { c with code := k.cont, ctrl := ks }
  | returnGemm {c} (hs : c.status = Status.running) (hc : c.code = [])
      (hk : c.ctrl = []) (hp : c.phase = .afterEntry) :
      Step c (Event.returnEvent (c.stack.headD 0))
        { c with status := .returned (c.stack.headD 0) }
  | enterGemm {c s'} (hs : c.status = Status.running) (hc : c.code = [])
      (hk : c.ctrl = []) (hp : c.phase = .beforeEntry)
      (hi : c.store.storeBytes c.harness.rawPtr c.harness.rawBytes = some s') :
      Step c Event.enterGemm
        { c with store := s'
                 locals := c.harness.args ++
                   List.replicate c.harness.gemmNumLocals 0
                 stack := [], code := c.harness.gemmBody, ctrl := []
                 phase := .afterEntry, entry? := some s'.observable }
  | installTrap {c} (hs : c.status = Status.running) (hc : c.code = [])
      (hk : c.ctrl = []) (hp : c.phase = .beforeEntry)
      (hi : c.store.storeBytes c.harness.rawPtr c.harness.rawBytes = none) :
      Step c (Event.trapEvent .outOfBounds) { c with status := .trapped .outOfBounds }

/-! ## The enumerator is exactly the relation -/

theorem branchTo_nodup (c : Config) (st : List UInt32) (n : Nat) :
    (branchTo c st n).Nodup := by
  unfold branchTo
  split
  · simp
  · split <;> simp

theorem successorsAtEnd_nodup (c : Config) : (successorsAtEnd c).Nodup := by
  unfold successorsAtEnd
  repeat' split
  all_goals simp

theorem successorsOfInstr_nodup (c : Config) (i : Instr) (rest : List Instr) :
    (successorsOfInstr c i rest).Nodup := by
  unfold successorsOfInstr
  repeat' split
  all_goals simp [branchTo_nodup]

/-- **SPEC section 7.1.**  The successor enumeration is duplicate free. -/
theorem successors_nodup (c : Config) : (successors c).Nodup := by
  unfold successors
  split
  · split
    · exact successorsAtEnd_nodup c
    · exact successorsOfInstr_nodup c _ _
  · simp

theorem mem_successorsAtEnd_step {c : Config} {e : Event} {c' : Config}
    (hs : c.status = Status.running) (hcode : c.code = [])
    (hm : (e, c') ∈ successorsAtEnd c) : Step c e c' := by
  cases c with
  | mk store harness locals stack code ctrl phase entry status =>
    unfold successorsAtEnd at hm
    cases ctrl with
    | cons k ks =>
      simp at hm
      obtain ⟨rfl, rfl⟩ := hm
      exact .exitLabel hs hcode rfl
    | nil =>
      cases phase with
      | afterEntry =>
        simp at hm
        obtain ⟨rfl, rfl⟩ := hm
        have hstep := Step.returnGemm hs hcode rfl rfl
        simpa using hstep
      | beforeEntry =>
        cases hi : store.storeBytes harness.rawPtr harness.rawBytes with
        | some s' =>
          simp [hi] at hm
          obtain ⟨rfl, rfl⟩ := hm
          exact .enterGemm hs hcode rfl rfl hi
        | none =>
          simp [hi] at hm
          obtain ⟨rfl, rfl⟩ := hm
          exact .installTrap hs hcode rfl rfl hi

theorem mem_branchTo_step_br {c : Config} {e : Event} {c' : Config} {n : Nat}
    {rest : List Instr} (hs : c.status = Status.running)
    (hcode : c.code = Instr.br n :: rest)
    (hm : (e, c') ∈ branchTo c c.stack n) : Step c e c' := by
  cases hk : c.ctrl[n]? with
  | none => simp [branchTo, hk] at hm
  | some k =>
    by_cases hl : k.isLoop = true
    · simp [branchTo, hk, hl] at hm
      obtain ⟨rfl, rfl⟩ := hm
      exact .brLoop hs hcode hk hl
    · simp [branchTo, hk, hl] at hm
      obtain ⟨rfl, rfl⟩ := hm
      exact .brBlock hs hcode hk hl

theorem mem_branchTo_step_brIf {c : Config} {e : Event} {c' : Config} {n : Nat}
    {rest : List Instr} {cnd : UInt32} {st : List UInt32}
    (hs : c.status = Status.running) (hcode : c.code = Instr.brIf n :: rest)
    (hst : c.stack = cnd :: st) (hcnd : ¬ cnd = 0)
    (hm : (e, c') ∈ branchTo c st n) : Step c e c' := by
  cases hk : c.ctrl[n]? with
  | none => simp [branchTo, hk] at hm
  | some k =>
    by_cases hl : k.isLoop = true
    · simp [branchTo, hk, hl] at hm
      obtain ⟨rfl, rfl⟩ := hm
      exact .brIfLoop hs hcode hst hcnd hk hl
    · simp [branchTo, hk, hl] at hm
      obtain ⟨rfl, rfl⟩ := hm
      exact .brIfBlock hs hcode hst hcnd hk hl

theorem mem_successorsOfInstr_step {c : Config} {e : Event} {c' : Config}
    {i : Instr} {rest : List Instr} (hs : c.status = Status.running)
    (hcode : c.code = i :: rest) (hm : (e, c') ∈ successorsOfInstr c i rest) :
    Step c e c' := by
  cases c with
  | mk store harness locals stack code ctrl phase entry status =>
  unfold successorsOfInstr at hm
  cases i
  case unreachable =>
    simp at hm; obtain ⟨rfl, rfl⟩ := hm; exact .unreachable hs hcode
  case nop =>
    simp at hm; obtain ⟨rfl, rfl⟩ := hm; exact .nop hs hcode
  case i32Const v =>
    simp at hm; obtain ⟨rfl, rfl⟩ := hm; exact .i32Const hs hcode
  case drop =>
    cases stack with
    | nil => simp at hm
    | cons x st =>
      simp at hm; obtain ⟨rfl, rfl⟩ := hm
      exact .drop hs hcode rfl
  case iBinOp w op =>
    cases w
    case i64 => simp at hm
    case i32 =>
      cases stack with
      | nil => simp at hm
      | cons b tail =>
        cases tail with
        | nil => simp at hm
        | cons a st =>
          cases hv : evalIBinOp op a b with
          | none => simp [hv] at hm
          | some r =>
            cases r with
            | none =>
              simp [hv] at hm; obtain ⟨rfl, rfl⟩ := hm
              exact .iBinOpTrap hs hcode rfl hv
            | some v =>
              simp [hv] at hm; obtain ⟨rfl, rfl⟩ := hm
              exact .iBinOp hs hcode rfl hv
  case iTestOp w op =>
    cases w
    case i64 => simp at hm
    case i32 =>
      cases op
      case eqz =>
        cases stack with
        | nil => simp at hm
        | cons a st =>
          simp at hm; obtain ⟨rfl, rfl⟩ := hm
          exact .iTestOp hs hcode rfl
  case iRelOp w op =>
    cases w
    case i64 => simp at hm
    case i32 =>
      cases stack with
      | nil => simp at hm
      | cons b tail =>
        cases tail with
        | nil => simp at hm
        | cons a st =>
          simp at hm; obtain ⟨rfl, rfl⟩ := hm
          exact .iRelOp hs hcode rfl
  case localGet idx =>
    cases hv : locals[idx]? with
    | none => simp [hv] at hm
    | some v =>
      simp [hv] at hm; obtain ⟨rfl, rfl⟩ := hm
      exact .localGet hs hcode hv
  case localSet idx =>
    cases stack with
    | nil => simp at hm
    | cons v st =>
      cases hw : locals[idx]? with
      | none => simp [hw] at hm
      | some w =>
        simp [hw] at hm; obtain ⟨rfl, rfl⟩ := hm
        exact .localSet hs hcode rfl hw
  case localTee idx =>
    cases stack with
    | nil => simp at hm
    | cons v st =>
      cases hw : locals[idx]? with
      | none => simp [hw] at hm
      | some w =>
        simp [hw] at hm; obtain ⟨rfl, rfl⟩ := hm
        exact .localTee hs hcode rfl hw
  case globalGet idx =>
    cases hg : store.globals[idx]? with
    | none => simp [hg] at hm
    | some g =>
      simp [hg] at hm; obtain ⟨rfl, rfl⟩ := hm
      exact .globalGet hs hcode hg
  case globalSet idx =>
    cases stack with
    | nil => simp at hm
    | cons v st =>
      cases hg : store.globals[idx]? with
      | none => simp [hg] at hm
      | some g =>
        simp [hg] at hm; obtain ⟨rfl, rfl⟩ := hm
        exact .globalSet hs hcode rfl hg
  case load t narrow arg =>
    cases t
    case i64 => simp at hm
    case f32 => simp at hm
    case f64 => simp at hm
    case i32 =>
      cases narrow
      case some nw => simp at hm
      case none =>
        cases stack with
        | nil => simp at hm
        | cons a st =>
          cases hb : store.loadBytes (a.toNat + arg.offset) 4 with
          | none =>
            simp [hb] at hm; obtain ⟨rfl, rfl⟩ := hm
            exact .loadTrap hs hcode rfl hb
          | some bs =>
            simp [hb] at hm; obtain ⟨rfl, rfl⟩ := hm
            exact .load hs hcode rfl hb
  case store t narrow arg =>
    cases t
    case i64 => simp at hm
    case f32 => simp at hm
    case f64 => simp at hm
    case i32 =>
      cases narrow
      case some nw => simp at hm
      case none =>
        cases stack with
        | nil => simp at hm
        | cons v tail =>
          cases tail with
          | nil => simp at hm
          | cons a st =>
            cases hb : store.storeBytes (a.toNat + arg.offset) (u32ToBytes v) with
            | none =>
              simp [hb] at hm; obtain ⟨rfl, rfl⟩ := hm
              exact .storeTrap hs hcode rfl hb
            | some s' =>
              simp [hb] at hm; obtain ⟨rfl, rfl⟩ := hm
              exact .store hs hcode rfl hb
  case memorySize mem =>
    cases mem with
    | zero => simp at hm; obtain ⟨rfl, rfl⟩ := hm; exact .memorySize hs hcode
    | succ k => simp at hm
  case memoryGrow mem =>
    cases mem with
    | succ k => simp at hm
    | zero =>
      cases stack with
      | nil => simp at hm
      | cons d st =>
        cases hg : store.memory.grow d.toNat with
        | none =>
          simp [hg] at hm; obtain ⟨rfl, rfl⟩ := hm
          exact .memoryGrowRefuse hs hcode rfl
        | some m' =>
          simp [hg] at hm
          rcases hm with ⟨rfl, rfl⟩ | ⟨rfl, rfl⟩
          · exact .memoryGrowSucceed hs hcode rfl hg
          · exact .memoryGrowRefuse hs hcode rfl
  case block bt body =>
    cases bt
    case value vt => simp at hm
    case typeIndex ti => simp at hm
    case empty =>
      simp at hm; obtain ⟨rfl, rfl⟩ := hm; exact .block hs hcode
  case loop bt body =>
    cases bt
    case value vt => simp at hm
    case typeIndex ti => simp at hm
    case empty =>
      simp at hm; obtain ⟨rfl, rfl⟩ := hm; exact .loop hs hcode
  case ifThenElse bt thenBody elseBody =>
    cases bt
    case value vt => simp at hm
    case typeIndex ti => simp at hm
    case empty =>
      cases stack with
      | nil => simp at hm
      | cons cnd st =>
        by_cases hcnd : cnd = 0
        · simp [hcnd] at hm; obtain ⟨rfl, rfl⟩ := hm
          exact .ifFalse hs hcode rfl hcnd
        · simp [hcnd] at hm; obtain ⟨rfl, rfl⟩ := hm
          exact .ifTrue hs hcode rfl hcnd
  case br n => exact mem_branchTo_step_br hs hcode hm
  case brIf n =>
    cases stack with
    | nil => simp at hm
    | cons cnd st =>
      by_cases hcnd : cnd = 0
      · simp [hcnd] at hm; obtain ⟨rfl, rfl⟩ := hm
        exact .brIfFalse hs hcode rfl hcnd
      · simp only [hcnd, if_false] at hm
        exact mem_branchTo_step_brIf hs hcode rfl hcnd hm
  case throw tag =>
    cases stack with
    | nil => simp at hm
    | cons v st =>
      simp at hm; obtain ⟨rfl, rfl⟩ := hm
      exact .throwTag hs hcode rfl
  all_goals simp at hm

/-- **SPEC section 7.1, the anti-cheat property.**  The executable successor
enumeration is exactly the reduction relation. -/
theorem mem_successors_iff_step (c : Config) (e : Event) (c' : Config) :
    (e, c') ∈ successors c ↔ Step c e c' := by
  constructor
  · intro hm
    unfold successors at hm
    by_cases hs : c.status = Status.running
    · rw [if_pos hs] at hm
      cases hcode : c.code with
      | nil => rw [hcode] at hm; exact mem_successorsAtEnd_step hs hcode hm
      | cons i rest =>
        rw [hcode] at hm
        exact mem_successorsOfInstr_step hs hcode hm
    · rw [if_neg hs] at hm; simp at hm
  · intro hd
    cases hd with
    | unreachable hs hc => simp [successors, successorsOfInstr, hs, hc]
    | nop hs hc => simp [successors, successorsOfInstr, hs, hc]
    | i32Const hs hc => simp [successors, successorsOfInstr, hs, hc]
    | drop hs hc hst => simp [successors, successorsOfInstr, hs, hc, hst]
    | iBinOp hs hc hst hv => simp [successors, successorsOfInstr, hs, hc, hst, hv]
    | iBinOpTrap hs hc hst hv => simp [successors, successorsOfInstr, hs, hc, hst, hv]
    | iTestOp hs hc hst => simp [successors, successorsOfInstr, hs, hc, hst]
    | iRelOp hs hc hst => simp [successors, successorsOfInstr, hs, hc, hst]
    | localGet hs hc hv => simp [successors, successorsOfInstr, hs, hc, hv]
    | localSet hs hc hst hw => simp [successors, successorsOfInstr, hs, hc, hst, hw]
    | localTee hs hc hst hw => simp [successors, successorsOfInstr, hs, hc, hst, hw]
    | globalGet hs hc hg => simp [successors, successorsOfInstr, hs, hc, hg]
    | globalSet hs hc hst hg => simp [successors, successorsOfInstr, hs, hc, hst, hg]
    | load hs hc hst hb => simp [successors, successorsOfInstr, hs, hc, hst, hb]
    | loadTrap hs hc hst hb => simp [successors, successorsOfInstr, hs, hc, hst, hb]
    | store hs hc hst hb => simp [successors, successorsOfInstr, hs, hc, hst, hb]
    | storeTrap hs hc hst hb => simp [successors, successorsOfInstr, hs, hc, hst, hb]
    | memorySize hs hc => simp [successors, successorsOfInstr, hs, hc]
    | memoryGrowSucceed hs hc hst hg =>
        simp [successors, successorsOfInstr, hs, hc, hst, hg]
    | @memoryGrowRefuse rest d st hs hc hst =>
        cases hg : c.store.memory.grow d.toNat with
        | none => simp [successors, successorsOfInstr, hs, hc, hst, hg]
        | some m' => simp [successors, successorsOfInstr, hs, hc, hst, hg]
    | block hs hc => simp [successors, successorsOfInstr, hs, hc]
    | loop hs hc => simp [successors, successorsOfInstr, hs, hc]
    | ifFalse hs hc hst hcnd => simp [successors, successorsOfInstr, hs, hc, hst, hcnd]
    | ifTrue hs hc hst hcnd => simp [successors, successorsOfInstr, hs, hc, hst, hcnd]
    | brLoop hs hc hk hl =>
        simp [successors, successorsOfInstr, branchTo, hs, hc, hk, hl]
    | brBlock hs hc hk hl =>
        simp [successors, successorsOfInstr, branchTo, hs, hc, hk, hl]
    | brIfFalse hs hc hst hcnd =>
        simp [successors, successorsOfInstr, hs, hc, hst, hcnd]
    | brIfLoop hs hc hst hcnd hk hl =>
        simp [successors, successorsOfInstr, branchTo, hs, hc, hst, hcnd, hk, hl]
    | brIfBlock hs hc hst hcnd hk hl =>
        simp [successors, successorsOfInstr, branchTo, hs, hc, hst, hcnd, hk, hl]
    | throwTag hs hc hst => simp [successors, successorsOfInstr, hs, hc, hst]
    | exitLabel hs hc hk => simp [successors, successorsAtEnd, hs, hc, hk]
    | returnGemm hs hc hk hp => simp [successors, successorsAtEnd, hs, hc, hk, hp]
    | enterGemm hs hc hk hp hi =>
        simp [successors, successorsAtEnd, hs, hc, hk, hp, hi]
    | installTrap hs hc hk hp hi =>
        simp [successors, successorsAtEnd, hs, hc, hk, hp, hi]

/-! ## Consequences -/

/-- A terminal configuration has no successors. -/
theorem successors_eq_nil_of_terminal {c : Config} (h : c.IsTerminal) :
    successors c = [] := by
  unfold successors Config.IsTerminal at *
  rw [if_neg h]

/-- A terminal configuration does not reduce. -/
theorem not_step_of_terminal {c : Config} (h : c.IsTerminal) (e : Event)
    (c' : Config) : ¬ Step c e c' := by
  intro hd
  have := (mem_successors_iff_step c e c').mpr hd
  rw [successors_eq_nil_of_terminal h] at this
  simp at this

/-- Reduction implies the configuration was running. -/
theorem Step.running {c : Config} {e : Event} {c' : Config} (h : Step c e c') :
    c.status = Status.running := by
  cases h <;> assumption

/-- Whenever a `memory.grow` can succeed, the relation really is
nondeterministic: two distinct successors exist. -/
theorem memoryGrow_nondeterministic {c : Config} {rest : List Instr}
    {d : UInt32} {st : List UInt32} {m' : Memory}
    (hs : c.status = Status.running) (hc : c.code = Instr.memoryGrow 0 :: rest)
    (hst : c.stack = d :: st) (hg : c.store.memory.grow d.toNat = some m') :
    ∃ e₁ c₁ e₂ c₂, Step c e₁ c₁ ∧ Step c e₂ c₂ ∧ (e₁, c₁) ≠ (e₂, c₂) :=
  ⟨_, _, _, _, Step.memoryGrowSucceed hs hc hst hg,
    Step.memoryGrowRefuse hs hc hst, by simp⟩

end WasmGemmGnaf.Wasm
