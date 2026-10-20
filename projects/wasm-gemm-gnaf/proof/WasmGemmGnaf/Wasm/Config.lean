/-
  Wasm/Config.lean --- frames, labels, stacks, instantiation and the harness
  control frame.

  Normative source: SPEC.md section 7.1 ("`Config` owns frames, labels, stacks,
  threads of control, instantiation, start invocation, and export calls") and
  section 7.2, which fixes the initial configuration protocol: "deterministic
  allocation, global/table/memory/data initialization, and construction of a
  harness control frame before executing a module start function; the costed
  harness then explores every permitted start-function branch, copies the raw
  invocation bytes only after a normal start return, and invokes `gemm`".

  The configuration is a single thread of control: a store, the harness
  descriptor, a frame of `i32` locals, an operand stack, the remaining
  instruction sequence, a control stack of labels, the harness phase, the
  snapshot taken at entry to `gemm`, and a terminal status.  Every value of the
  declared subset is an `i32`, so an operand stack is a list of words.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Store

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Terminal statuses -/

/-- The Core traps reachable in the declared subset. -/
inductive Trap
  /-- `unreachable` was executed. -/
  | unreachable
  /-- A memory access left the allocated bytes. -/
  | outOfBounds
  /-- An unsigned division or remainder had a zero divisor. -/
  | divideByZero
  deriving DecidableEq, Repr, Inhabited

/-- An uncaught Core exception: its tag index and its single `i32` payload. -/
structure ExceptionValue where
  /-- The tag index. -/
  tag : Nat
  /-- The payload word. -/
  payload : UInt32
  deriving DecidableEq, Repr, Inhabited

/-- The status of a configuration.  A configuration reduces exactly while it is
`running`; the other three statuses are terminal and mutually exclusive. -/
inductive Status
  /-- Still reducing. -/
  | running
  /-- The invocation returned the given `i32`. -/
  | returned (value : UInt32)
  /-- The invocation trapped. -/
  | trapped (trap : Trap)
  /-- The invocation terminated with an uncaught exception. -/
  | thrown (exceptionValue : ExceptionValue)
  deriving DecidableEq, Repr, Inhabited

/-- The terminal statuses, as observed by `Run`. -/
inductive TerminalStatus
  | returned (value : UInt32)
  | trapped (trap : Trap)
  | thrown (exceptionValue : ExceptionValue)
  deriving DecidableEq, Repr, Inhabited

/-! ## Control stack -/

/-- One entry of the control stack: a label.  `body` is the instruction
sequence re-entered by a branch to a `loop`; `cont` is the sequence resumed on
normal exit or on a branch to a `block`; `height` is the operand-stack height
at the moment the label was pushed, which is the height a branch restores. -/
structure CtrlEntry where
  /-- Whether the label belongs to a `loop`. -/
  isLoop : Bool
  /-- The loop body, re-entered by a branch to a `loop` label. -/
  body : List Instr
  /-- The continuation resumed on exit or on a branch to a `block` label. -/
  cont : List Instr
  /-- The operand-stack height restored by a branch to this label. -/
  height : Nat
  deriving DecidableEq, Inhabited

/-- Restore an operand stack to a recorded height by discarding the values
pushed since the label was entered.  The stack is stored top first. -/
def truncStack (st : List UInt32) (h : Nat) : List UInt32 :=
  st.drop (st.length - h)

theorem truncStack_length {st : List UInt32} {h : Nat} (hle : h ≤ st.length) :
    (truncStack st h).length = h := by
  simp [truncStack]
  omega

theorem truncStack_self (st : List UInt32) : truncStack st st.length = st := by
  simp [truncStack]

/-! ## The harness -/

/-- The harness control frame of SPEC section 7.2: the exported `gemm` body,
the raw invocation bytes to be installed, and the arguments passed to `gemm`. -/
structure Harness where
  /-- The body of the exported `gemm` function. -/
  gemmBody : List Instr
  /-- The number of declared locals of `gemm`, beyond its two parameters. -/
  gemmNumLocals : Nat
  /-- The address at which the raw invocation bytes are installed. -/
  rawPtr : Nat
  /-- The raw invocation bytes. -/
  rawBytes : List UInt8
  /-- The `(ptr, len)` arguments passed to `gemm`. -/
  args : List UInt32
  deriving DecidableEq, Inhabited

/-- The harness phase.  `beforeEntry` covers allocation, the module start
function and the raw-byte installation; `afterEntry` covers the exported
`gemm` call.  The boundary is crossed exactly once. -/
inductive Phase
  | beforeEntry
  | afterEntry
  deriving DecidableEq, Repr, Inhabited

/-! ## Configurations -/

/-- A configuration of the released machine. -/
structure Config where
  /-- The runtime store. -/
  store : Store
  /-- The harness descriptor, which never changes during a run. -/
  harness : Harness
  /-- The locals of the current frame. -/
  locals : List UInt32
  /-- The operand stack, top first. -/
  stack : List UInt32
  /-- The instruction sequence still to execute. -/
  code : List Instr
  /-- The control stack of labels, innermost first. -/
  ctrl : List CtrlEntry
  /-- The harness phase. -/
  phase : Phase
  /-- The store snapshot taken at entry to `gemm`, if that boundary was
  crossed. -/
  entry? : Option ObservableStore
  /-- The status. -/
  status : Status
  deriving DecidableEq, Inhabited

namespace Config

/-- A configuration is terminal exactly when it is not running. -/
def IsTerminal (c : Config) : Prop := c.status ≠ .running

instance instDecidableIsTerminal (c : Config) : Decidable c.IsTerminal := by
  unfold IsTerminal; infer_instance

/-- The terminal status of a terminal configuration. -/
def terminalStatus? (c : Config) : Option TerminalStatus :=
  match c.status with
  | .running => none
  | .returned v => some (.returned v)
  | .trapped t => some (.trapped t)
  | .thrown e => some (.thrown e)

theorem terminalStatus?_eq_none_iff (c : Config) :
    c.terminalStatus? = none ↔ c.status = .running := by
  unfold terminalStatus?
  cases c.status <;> simp

theorem terminalStatus?_isSome_iff (c : Config) :
    (c.terminalStatus?).isSome ↔ c.IsTerminal := by
  unfold terminalStatus? IsTerminal
  cases c.status <;> simp

/-- Halting, trapping and throwing are mutually exclusive and exhaust the
terminal configurations. -/
theorem isTerminal_iff (c : Config) :
    c.IsTerminal ↔
      ((∃ v, c.status = .returned v) ∨ (∃ t, c.status = .trapped t) ∨
        (∃ e, c.status = .thrown e)) := by
  unfold IsTerminal
  cases c.status <;> simp

/-- The observable store of a configuration. -/
def observable (c : Config) : ObservableStore := c.store.observable

end Config

/-! ## Events

An event labels one reduction.  Every permitted nondeterministic choice is
recorded in the event, so two distinct successors of one configuration always
carry distinct labels. -/

/-- The label of one reduction. -/
inductive Event
  /-- One ordinary deterministic rule step. -/
  | step
  /-- One dispatch step: a taken branch, or the selection of an `if` arm. -/
  | branch
  /-- A `memory.grow` attempt with the embedder's permitted outcome. -/
  | growAttempt (outcome : Num.GrowOutcome)
  /-- The harness phase transition into the exported `gemm`. -/
  | enterGemm
  /-- A trap. -/
  | trapEvent (trap : Trap)
  /-- An uncaught exception. -/
  | throwEvent (exceptionValue : ExceptionValue)
  /-- A normal return of the exported `gemm`. -/
  | returnEvent (value : UInt32)
  deriving DecidableEq, Repr, Inhabited

/-! ## Numeric evaluation used by the reduction rules

`evalIBinOp` returns `none` for an operator outside the declared subset and
`some none` for a trapping application.  `evalIRelOp` is total. -/

/-- The declared-subset `i32` binary operators. -/
def evalIBinOp : IBinOp → UInt32 → UInt32 → Option (Option UInt32)
  | .add, a, b => some (some (Num.i32Add a b))
  | .sub, a, b => some (some (Num.i32Sub a b))
  | .mul, a, b => some (some (Num.i32Mul a b))
  | .divU, a, b => some (Num.i32DivU a b)
  | .remU, a, b => some (Num.i32RemU a b)
  | .and, a, b => some (some (Num.i32And a b))
  | .or, a, b => some (some (Num.i32Or a b))
  | .xor, a, b => some (some (Num.i32Xor a b))
  | .shl, a, b => some (some (Num.i32Shl a b))
  | .shrU, a, b => some (some (Num.i32ShrU a b))
  | _, _, _ => none

/-- `evalIBinOp` is defined exactly on the operators admitted by validation. -/
theorem evalIBinOp_isSome_iff (op : IBinOp) (a b : UInt32) :
    (evalIBinOp op a b).isSome = true ↔ SubsetIBinOp op = true := by
  cases op <;> simp [evalIBinOp, SubsetIBinOp]

/-- The admitted binary operators trap exactly on an unsigned division or
remainder by zero. -/
theorem evalIBinOp_eq_some_none_iff (op : IBinOp) (a b : UInt32) :
    evalIBinOp op a b = some none ↔ ((op = .divU ∨ op = .remU) ∧ b = 0) := by
  cases op <;>
    simp [evalIBinOp, Num.i32DivU_eq_none_iff, Num.i32RemU_eq_none_iff] <;>
    (try simp [Num.i32DivU, Num.i32RemU]) <;> (try split <;> simp_all)

/-- The `i32` relational operators. -/
def evalIRelOp : IRelOp → UInt32 → UInt32 → UInt32
  | .eq, a, b => Num.i32Eq a b
  | .ne, a, b => Num.i32Ne a b
  | .ltS, a, b => Num.i32LtS a b
  | .ltU, a, b => Num.i32LtU a b
  | .gtS, a, b => Num.i32GtS a b
  | .gtU, a, b => Num.i32GtU a b
  | .leS, a, b => Num.i32LeS a b
  | .leU, a, b => Num.i32LeU a b
  | .geS, a, b => Num.i32GeS a b
  | .geU, a, b => Num.i32GeU a b

theorem evalIRelOp_eq_self (a : UInt32) : evalIRelOp .eq a a = 1 :=
  Num.i32Eq_self a

/-! ## Initialization -/

/-- An initialization fault of SPEC section 7.4. -/
inductive InstantiationFault
  /-- The module failed release validation. -/
  | invalidModule
  /-- Allocation or static initialization failed. -/
  | allocationFailed
  /-- The module has no exported `gemm` function. -/
  | missingGemmExport
  deriving DecidableEq, Repr, Inhabited

/-- A raw invocation: the address at which the raw bytes are installed and the
bytes themselves. -/
structure RawInvocation where
  /-- The installation address. -/
  ptr : Nat
  /-- The raw bytes. -/
  bytes : List UInt8
  deriving DecidableEq, Repr, Inhabited

/-- The body of a defined function as an instruction list. -/
def Func.code (f : Func) : List Instr := f.body.toList

/-- The pre-start harness configuration of SPEC section 7.5: allocation and
static initialization are complete, the harness control frame is built, and the
module start function is about to run.  The start function, the raw-byte
installation and the exported call are ordinary `Step` transitions. -/
def initialConfig (m : Module) (raw : RawInvocation) : Except InstantiationFault Config :=
  if validate m = false then .error .invalidModule
  else
    match Store.alloc m with
    | none => .error .allocationFailed
    | some store =>
        match Module.gemmIndex? m with
        | none => .error .missingGemmExport
        | some gi =>
            match m.funcs[gi]? with
            | none => .error .missingGemmExport
            | some gemmFunc =>
                let harness : Harness :=
                  { gemmBody := gemmFunc.code
                    gemmNumLocals := gemmFunc.locals.length
                    rawPtr := raw.ptr
                    rawBytes := raw.bytes
                    args := [UInt32.ofNat raw.ptr,
                             UInt32.ofNat raw.bytes.length] }
                match m.start with
                | none =>
                    .ok { store, harness, locals := [], stack := [], code := []
                          ctrl := [], phase := .beforeEntry, entry? := none
                          status := .running }
                | some si =>
                    match m.funcs[si]? with
                    | none => .error .invalidModule
                    | some startFunc =>
                        .ok { store, harness
                              locals := List.replicate startFunc.locals.length 0
                              stack := [], code := startFunc.code
                              ctrl := [], phase := .beforeEntry, entry? := none
                              status := .running }

/-- Initialization stops before the start function: the initial configuration
is running, has crossed no entry boundary, and has an empty control stack and
operand stack. -/
theorem initialConfig_shape {m : Module} {raw : RawInvocation} {c : Config}
    (h : initialConfig m raw = .ok c) :
    c.status = .running ∧ c.phase = .beforeEntry ∧ c.entry? = none ∧
      c.ctrl = [] ∧ c.stack = [] := by
  unfold initialConfig at h
  split at h
  · exact absurd h (by simp)
  · split at h
    · exact absurd h (by simp)
    · split at h
      · exact absurd h (by simp)
      · split at h
        · exact absurd h (by simp)
        · split at h
          · injection h with h; subst h; exact ⟨rfl, rfl, rfl, rfl, rfl⟩
          · split at h
            · exact absurd h (by simp)
            · injection h with h; subst h; exact ⟨rfl, rfl, rfl, rfl, rfl⟩

/-- Initialization rejects exactly the modules that fail release validation,
before any allocation is attempted. -/
theorem initialConfig_invalid {m : Module} {raw : RawInvocation}
    (h : validate m = false) : initialConfig m raw = .error .invalidModule := by
  simp [initialConfig, h]

end WasmGemmGnaf.Wasm
