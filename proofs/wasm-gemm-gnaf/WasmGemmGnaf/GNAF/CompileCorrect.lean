import WasmGemmGnaf.GNAF.Compile
import WasmGemmGnaf.Wasm.Run
import WasmGemmGnaf.Artifact.Emit

set_option autoImplicit false
set_option maxRecDepth 8000

/-!
# GNAF: what the compiler is proved to guarantee about execution (SPEC §11.4)

This file collects the refinement facts that the *current* state of the Wasm
layer actually supports, and states precisely which SPEC §11.4 obligations are
therefore **not** discharged anywhere in this repository.

## What is proved here

`Wasm/Config.lean` fixes initialization: `Wasm.initialConfig` validates the
module, allocates the store, locates the exported `gemm`, and builds the
harness control frame.  Because `GNAF.compile_validates` proves the emitted
module passes release validation, and because the emitted module's shape is
fixed, initialization of a compiled module can be computed exactly:

* `GNAF.compile_initialConfig` — the initial configuration of a compiled module
  is a specific configuration, given in closed form;
* `GNAF.compile_no_instantiation_fault` — no compiled module ever produces an
  instantiation fault;
* `GNAF.compile_gemmBody` — the harness runs exactly the compiled body;
* `GNAF.compile_runInvariant` — the initial configuration satisfies
  `Wasm.RunInvariant`, so `Wasm.returned_has_entry_store` applies to every run
  of a compiled module: a normally returning run of a compiled artifact has
  crossed the harness boundary and carries exactly one entry store.

The last section closes the anti-vacuity gap of SPEC §8.3: `GNAF.gemmWitness` is
a concrete `1×1×1` modular-`u32` GEMM plan proved to lie inside
`Plan.inReleasedSubset`, so `GNAF.code_no_unreachable` and the other in-subset
results have an exhibited inhabitant rather than an unwitnessed domain.  The
witness is not only in the subset: `GNAF.gemmWitness_writes_C` proves that
running it deposits the modular product into the declared `C` region, so the
inhabitant is a plan that genuinely computes a GEMM rather than a plan that
merely type checks.

The final section removes the witness's *fixed-extent* limitation.
`GNAF.gemmKernel` is an **input-dependent** GEMM plan: it loads the descriptor's
`m`, `n` and `k` out of the raw ABI header with `Plan.loadReg` and runs an
`i`/`j`/`k` nest of `Plan.loopReg` loops bounded by exactly those registers, so
no extent occurs anywhere in the plan text.  `GNAF.gemmKernel_writes_C` proves
that the declared `C` region then holds the modular-`u32` product for *arbitrary*
`m`, `n`, `k` (under hypotheses stated exactly there), and the same
`CheckedPlan` is exhibited computing correctly on three different shapes —
`1×1×1`, `2×2×2` and the rectangular `1×3×2`.

## What is deliberately omitted, and why

* **`compile_refines`** (SPEC §11.4).  Omitted.  It would relate
  `Wasm.FiniteExecution` of the compiled module to `GNAF.Accepts`.  The GNAF
  machine of `GNAF/Semantics.lean` computes over unbounded `Nat` cells while
  the Wasm machine computes over `i32`, so the statement is *false* as written
  without an explicit boundedness precondition and an explicit representation
  relation between a `GNAF.Machine` and a `Wasm.Config`; and proving even the
  bounded version needs a simulation argument for every plan constructor, which
  is not established here.  Weakening the statement — for instance quantifying
  only over runs that happen to exist, or replacing `Accepts` by an
  observational projection — would produce a theorem that looks like a
  refinement result and is not one, so nothing of the kind is stated.
* **`compile_cost_exact`** (SPEC §11.4).  Omitted.  `Wasm/Costed.lean` does
  supply a costed reduction relation (`Wasm.CostedReduces`), so the statement
  could now be *written*; but its content is that the measured cost of a costed
  run of the compiled module equals `GNAF.certifiedCost` of the plan, and that
  is strictly stronger than the refinement theorem above, which is not proved.
  Stating it with a hypothesis nobody can discharge, or proving a variant whose
  cost function is read off the compiler rather than from the plan, would both
  be worse than saying nothing: neither would be the SPEC obligation.
* **A step bound for the emitted module.**  `GNAF.compile_resources` in
  `GNAF/Compile.lean` bounds the emitted module's *code size* by the plan's
  declared static cost.  A bound on the number of Wasm reduction steps would be
  a consequence of the missing refinement theorem and is not claimed.

Every declaration in this file is proved.  Nothing is assumed.
-/

namespace WasmGemmGnaf.GNAF

open WasmGemmGnaf.Foundation

variable {s t : Sig}

/-- The store the emitted module allocates: a fresh zeroed memory of the
declared page count and no globals. -/
def compiledStore (e : CompileEnv) : Wasm.Store :=
  { memory := Wasm.Memory.alloc e.pages (some e.pages), globals := [] }

/-- The harness control frame the emitted module installs. -/
def compiledHarness (e : CompileEnv) (body : List Wasm.Instr)
    (raw : Wasm.RawInvocation) : Wasm.Harness :=
  { gemmBody := body
    gemmNumLocals := e.declaredLocals
    rawPtr := raw.ptr
    rawBytes := raw.bytes
    args := [UInt32.ofNat raw.ptr, UInt32.ofNat raw.bytes.length] }

/-- The initial configuration of an emitted module. -/
def compiledInitialConfig (e : CompileEnv) (body : List Wasm.Instr)
    (raw : Wasm.RawInvocation) : Wasm.Config :=
  { store := compiledStore e
    harness := compiledHarness e body raw
    locals := [], stack := [], code := [], ctrl := []
    phase := .beforeEntry, entry? := none, status := .running }

theorem alloc_moduleOf (e : CompileEnv) (b : List Wasm.Instr) :
    Wasm.Store.alloc (moduleOf e b) = some (compiledStore e) := rfl

/-- Initialization of an emitted module succeeds and produces exactly
`compiledInitialConfig`. -/
theorem initialConfig_moduleOf (e : CompileEnv) (b : List Wasm.Instr)
    (hv : Wasm.validate (moduleOf e b) = true) (raw : Wasm.RawInvocation) :
    Wasm.initialConfig (moduleOf e b) raw = .ok (compiledInitialConfig e b raw) := by
  unfold Wasm.initialConfig
  rw [if_neg (by rw [hv]; exact Bool.noConfusion)]
  rw [alloc_moduleOf, gemmIndex_gemmExports]
  simp only [CompiledModule.funcs_eq, List.getElem?_cons_zero, CompiledModule.start,
    Wasm.Func.code, Wasm.Expr.toList_ofList, List.length_replicate]
  rfl

/-- **The compiled module always instantiates.**  Initialization never faults,
and the resulting configuration is given in closed form. -/
theorem compile_initialConfig (c : CheckedPlan s t) (raw : Wasm.RawInvocation) :
    Wasm.initialConfig (compile c) raw =
      .ok (compiledInitialConfig (envOf s c.plan)
            (bodyCode (envOf s c.plan) s.scratch c.plan) raw) :=
  initialConfig_moduleOf _ _ (compile_validates c) raw

/-- **No compiled module ever produces an instantiation fault.** -/
theorem compile_no_instantiation_fault (c : CheckedPlan s t)
    (raw : Wasm.RawInvocation) (f : Wasm.InstantiationFault) :
    Wasm.initialConfig (compile c) raw ≠ .error f := by
  rw [compile_initialConfig]
  simp

/-- The harness of a compiled module runs exactly the compiled plan body. -/
theorem compile_gemmBody (c : CheckedPlan s t) (raw : Wasm.RawInvocation) :
    (compiledInitialConfig (envOf s c.plan)
      (bodyCode (envOf s c.plan) s.scratch c.plan) raw).harness.gemmBody =
      bodyCode (envOf s c.plan) s.scratch c.plan := rfl

/-- The compiled module declares exactly the locals the translation uses. -/
theorem compile_gemmNumLocals (c : CheckedPlan s t) (raw : Wasm.RawInvocation) :
    (compiledInitialConfig (envOf s c.plan)
      (bodyCode (envOf s c.plan) s.scratch c.plan) raw).harness.gemmNumLocals =
      (envOf s c.plan).declaredLocals := rfl

/-- The compiled module starts before the harness boundary, with no entry
snapshot and an empty control and operand stack. -/
theorem compile_initial_shape (c : CheckedPlan s t) (raw : Wasm.RawInvocation) :
    ∀ cfg : Wasm.Config, Wasm.initialConfig (compile c) raw = .ok cfg →
      cfg.status = .running ∧ cfg.phase = .beforeEntry ∧ cfg.entry? = none ∧
        cfg.ctrl = [] ∧ cfg.stack = [] := by
  intro cfg h
  exact Wasm.initialConfig_shape h

/-- The initial configuration of a compiled module satisfies the harness run
invariant of SPEC §7.4. -/
theorem compile_runInvariant (c : CheckedPlan s t) (raw : Wasm.RawInvocation) :
    Wasm.RunInvariant (compiledInitialConfig (envOf s c.plan)
      (bodyCode (envOf s c.plan) s.scratch c.plan) raw) :=
  Wasm.runInvariant_of_beforeEntry rfl rfl

/-- **Every normally returning run of a compiled artifact has crossed the
harness boundary**, so its observation carries exactly one entry store.  This is
SPEC §7.4's presence invariant, instantiated at the compiled module. -/
theorem compile_returned_has_entry_store (c : CheckedPlan s t)
    (raw : Wasm.RawInvocation) {final : Wasm.Config} {tr : List Wasm.Event}
    {v : UInt32}
    (hred : Wasm.Reduces (compiledInitialConfig (envOf s c.plan)
      (bodyCode (envOf s c.plan) s.scratch c.plan) raw) tr final)
    (hfin : final.status = Wasm.Status.returned v) :
    final.entry?.isSome = true :=
  Wasm.returned_has_entry_store hred (compile_runInvariant c raw) hfin

/-- A finite execution of a compiled artifact is a `*BeforeEntry` observation
exactly when it never crossed the harness boundary. -/
theorem compile_beforeEntry_iff (c : CheckedPlan s t) (raw : Wasm.RawInvocation)
    {o : Wasm.ExecutionObservation}
    (h : Wasm.FiniteExecution (compiledInitialConfig (envOf s c.plan)
      (bodyCode (envOf s c.plan) s.scratch c.plan) raw) o) :
    o.BeforeEntry ↔ Wasm.Event.enterGemm ∉ o.trace :=
  Wasm.beforeEntry_iff_not_mem_enterGemm h rfl

/-! ## The first reduction of a compiled artifact -/

/-- The store after the harness has installed the raw invocation bytes. -/
def installedStore (e : CompileEnv) (raw : Wasm.RawInvocation) : Wasm.Store :=
  { compiledStore e with
    memory := { (compiledStore e).memory with
                data := Wasm.setBytes (compiledStore e).memory.data raw.ptr raw.bytes } }

theorem storeBytes_compiledStore (e : CompileEnv) (raw : Wasm.RawInvocation)
    (hfit : raw.ptr + raw.bytes.length ≤ e.pages * Wasm.pageSize) :
    (compiledStore e).storeBytes raw.ptr raw.bytes = some (installedStore e raw) := by
  have hsize : (compiledStore e).memory.size = e.pages * Wasm.pageSize :=
    Wasm.Memory.alloc_size _ _
  unfold Wasm.Store.storeBytes Wasm.Memory.storeBytes
  rw [if_pos (by rw [hsize]; exact hfit)]
  rfl

/-- **The compiled artifact reaches its compiled body in one reduction.**
Whenever the raw invocation fits in the declared memory, the harness step of
SPEC §7.2 installs the raw bytes, crosses the entry boundary, and hands control
to exactly the instruction sequence the compiler emitted. -/
theorem compile_enters_gemm (e : CompileEnv) (body : List Wasm.Instr)
    (raw : Wasm.RawInvocation)
    (hfit : raw.ptr + raw.bytes.length ≤ e.pages * Wasm.pageSize) :
    Wasm.Step (compiledInitialConfig e body raw) Wasm.Event.enterGemm
      { compiledInitialConfig e body raw with
        store := installedStore e raw
        locals := [UInt32.ofNat raw.ptr, UInt32.ofNat raw.bytes.length] ++
          List.replicate e.declaredLocals 0
        stack := [], code := body, ctrl := []
        phase := .afterEntry
        entry? := some (installedStore e raw).observable } := by
  refine (Wasm.mem_successors_iff_step _ _ _).mp ?_
  show _ ∈ Wasm.successorsAtEnd (compiledInitialConfig e body raw)
  unfold Wasm.successorsAtEnd
  simp only [compiledInitialConfig, compiledHarness]
  rw [storeBytes_compiledStore e raw hfit]
  simp

/-- Consequently the compiled body is the code of a configuration reachable
from the initial configuration of the compiled module. -/
theorem compile_body_reachable (c : CheckedPlan s t) (raw : Wasm.RawInvocation)
    (hfit : raw.ptr + raw.bytes.length ≤
      (envOf s c.plan).pages * Wasm.pageSize) :
    ∃ cfg : Wasm.Config,
      Wasm.Reduces (compiledInitialConfig (envOf s c.plan)
        (bodyCode (envOf s c.plan) s.scratch c.plan) raw) [Wasm.Event.enterGemm] cfg ∧
      cfg.code = bodyCode (envOf s c.plan) s.scratch c.plan :=
  ⟨_, Wasm.Reduces.single (compile_enters_gemm _ _ raw hfit), rfl⟩

/-! ## The artifact round trip -/

/-- The emitted artifact of a compiled plan decodes back to exactly the
compiled module. -/
theorem compile_decode_emit (c : CheckedPlan s t) :
    Wasm.decode (Artifact.emit (compile c)) = .ok (compile c) :=
  Artifact.decode_emit _

/-- Compiled modules with distinct emitted bytes are distinct modules, and
conversely: the artifact identifies the module it came from. -/
theorem compile_emit_injective {c c' : CheckedPlan s t}
    (h : Artifact.emit (compile c) = Artifact.emit (compile c')) :
    compile c = compile c' :=
  Artifact.emit_injective h

/-- The emitted artifact of a compiled plan decodes to a module that passes
release validation. -/
theorem compile_emit_decodes_valid (c : CheckedPlan s t) :
    ∃ m : Wasm.Module,
      Wasm.decode (Artifact.emit (compile c)) = .ok m ∧ Wasm.validate m = true :=
  ⟨compile c, compile_decode_emit c, compile_validates c⟩

/-! ## The anti-vacuity GEMM witness (SPEC §8.3)

`compile_validates` holds for every `CheckedPlan`, and `code_no_unreachable`
holds for every plan inside `Plan.inReleasedSubset`.  Both are universally
quantified, so on their own neither exhibits a single plan that is *both*
inside the subset and a GEMM.  This section supplies one.

`gemmWitness` is a complete released-profile plan: it classifies the raw ABI
header, dispatches on the declared layout class, runs the blocked traversal and
the `i`/`j` loop nest, accumulates the `k` reduction under the modular-`u32`
arithmetic contract of SPEC §8.2, sets the released status word, and constructs
the output from the declared `C` region.  It is checked at a *scalar* interface
(`Sig.lanes = 0`), so `hasType_no_vector` applies to it as well.

The GNAF machine of `GNAF/Semantics.lean` is cell addressed: `Machine.byteAt i`
is memory cell `i`, and `reduce` reads one whole element per cell.  The witness
header therefore lays SPEC §8.3's header table out cell-wise, and the three
matrix elements occupy the three cells after it.  No claim is made here that
this cell image equals the byte image produced by `Gemm/ABI.lean`'s
`encodeHeader`; that is a byte-level representation statement about a different
machine model and it is not proved anywhere, so it is not asserted.

### Where the result goes

The kernel deposits its accumulator into the declared `C` region with
`Plan.storeReg`, the constructor that moves a register into memory; its
load-after-store law is `GNAF.storeReg_reads_back` and its frame law is
`GNAF.storeReg_outside`.  `gemmWitness_eval_acc` proves the product is computed
exactly, `gemmWitness_eval_mem` gives the memory the store leaves — the
four-cell little-endian image of that product at `C` — and
`gemmWitness_writes_C` reads that image back: **after evaluation the `C` region
holds `alpha · A · B + beta · C` modulo `2 ^ 32`**, which is what makes this
plan a GEMM and not merely a well-typed plan.  `gemmWitness_eval_mem_outside`
is the matching frame statement: every cell outside `C`'s four keeps its entry
value.  The store is a *cell-wise little-endian* image, exactly as
`Machine.u16At` reads cells; no claim is made here that it coincides with the
byte image of `Gemm/ABI.lean`, for the same reason the header image is not
claimed to.

Three things `gemmWitness_writes_C` does **not** say, stated precisely because
the theorem is the first link of SPEC §13 Phase B and its strength matters:

* **It is the `alpha = 1`, `beta = 0` instance.**  `gemmWitnessAlpha` and
  `gemmWitnessBeta` are the scalars read out of the witness header's own
  `alpha` and `beta` fields, and that header declares `alpha = 1` and
  `beta = 0` (`gemmWitnessAlpha_eq`, `gemmWitnessBeta_eq`).  The plan contains
  no scaling node at all, so the theorem holds *because* those two literals are
  `1` and `0`; it does not generalize to a descriptor declaring other scalars,
  and no such generalization is claimed anywhere.
* **It is a statement about `Plan.eval`, not about the emitted Wasm.**
  Relating the two is `compile_refines`, which is omitted above for the reasons
  given there.
* **The witness descriptor's cell image still has overlapping regions.**  The
  header declares `A`, `B` and `C` element lengths of four bytes each while the
  cell model gives `A` and `B` one cell apiece, and it declares the
  status-detail record at cell `259`, which `C`'s four cells run into.
  `Machine.classify` checks the header literals, the declared kind/mode triple
  and the invocation extent — it does not check region disjointness — so
  `gemmWitnessMachine_classify` is not evidence that these regions are
  disjoint, and no disjointness is claimed.  The cells `C` overlaps are written
  by no other node of this plan and read by none, so the theorems above are
  unaffected; a descriptor-level `disjoint` obligation would have to be
  discharged against `Gemm/Descriptor.lean`, and it is not discharged here.

What the witness *does* now publish is the whole result: `C` is declared as its
four stored cells, so `buildOutput` emits the complete little-endian image of
the product (`gemmWitness_eval_out`) rather than one byte of it. -/

/-- The released modular arithmetic contract with a `u32` accumulator: SPEC
§8.2 compatibility row 0 at stored kind `u32`, and the one contract shape the
released `i32` profile evaluates exactly. -/
def gemmWitnessContract : ArithmeticContract :=
  { mode := .modular, stored := .u32, accumulator := .u32 }

theorem gemmWitnessContract_compatible : gemmWitnessContract.compatibleB = true := by
  decide

theorem gemmWitnessContract_released : gemmWitnessContract.releasedB = true := by
  decide

/-- The accumulator modulus of the witness contract is exactly `2 ^ 32`. -/
theorem gemmWitnessContract_accModulus : gemmWitnessContract.accModulus = 4294967296 :=
  ArithmeticContract.accModulus_of_releasedB gemmWitnessContract_released

/-- The 256 header cells of the witness descriptor, in exactly SPEC §8.3's
header order: a `1×1×1`, batch `1`, stored-`u32`, accumulator-`u32`, `modular`,
untransposed, `disjoint`, row-major GEMM with `alpha = 1` and `beta = 0`. -/
def gemmWitnessHeader : List Nat :=
  -- `0..3` magic `WGNG`, `4..5` version 1, `6..7` header size 256
  [87, 71, 78, 71, 1, 0, 0, 1] ++
  -- `8..11` A, B, C and accumulator kind tags (`u32` = 5)
  [5, 5, 5, 5] ++
  -- `12` mode `modular`, `13` transpose bits, `14` alias tag `disjoint`, `15` zero
  [0, 0, 0, 0] ++
  -- `16..47` m, n, k, batch
  [1, 0, 0, 0, 0, 0, 0, 0] ++ [1, 0, 0, 0, 0, 0, 0, 0] ++
  [1, 0, 0, 0, 0, 0, 0, 0] ++ [1, 0, 0, 0, 0, 0, 0, 0] ++
  -- `48..87` A view: offset 256, byte length 4, row/column/batch stride 4
  [0, 1, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++
  -- `88..127` B view: offset 257, same packed strides
  [1, 1, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++
  -- `128..167` C view: offset 258, same packed strides
  [2, 1, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++ [4, 0, 0, 0, 0, 0, 0, 0] ++
  [4, 0, 0, 0, 0, 0, 0, 0] ++
  -- `168..183` alpha = 1, `184..199` beta = 0
  [1, 0, 0, 0, 0, 0, 0, 0] ++ List.replicate 8 0 ++ List.replicate 16 0 ++
  -- `200..215` scratch offset 0 and scratch length 0
  List.replicate 16 0 ++
  -- `216..231` status-detail offset 259 and status-detail length 32
  [3, 1, 0, 0, 0, 0, 0, 0] ++ [32, 0, 0, 0, 0, 0, 0, 0] ++
  -- `232..255` reserved
  List.replicate 24 0

theorem gemmWitnessHeader_length : gemmWitnessHeader.length = 256 := rfl

/-- The witness machine memory: the 256 header cells, then the `A`, `B` and `C`
elements, then the 32-cell status-detail record the header declares. -/
def gemmWitnessMem (a b c : Nat) : List Nat :=
  gemmWitnessHeader ++ [a, b, c] ++ List.replicate 32 0

theorem gemmWitnessMem_length (a b c : Nat) : (gemmWitnessMem a b c).length = 291 := rfl

/-- The declared `A` region: one element, in the cell after the header. -/
def gemmWitnessA : RegionRef := { base := 256, count := 1 }

/-- The declared `B` region. -/
def gemmWitnessB : RegionRef := { base := 257, count := 1 }

/-- The declared `C` region — the output the plan publishes.  It is the four
cells the kernel's four-cell little-endian store writes, which is exactly the
four-byte `C` element length the witness header declares, so the store fits the
region it declares and `buildOutput` publishes the whole stored result. -/
def gemmWitnessC : RegionRef := { base := 258, count := 4 }

/-- The witness input configuration.  The entry status is deliberately not
`success`, so that `gemmWitness_eval_status` has content. -/
def gemmWitnessMachine (a b c : Nat) : Machine :=
  { mem := gemmWitnessMem a b c
    scratch := []
    regs := [0, 0, 0, 0]
    vregs := []
    tables := []
    status := Status.arithmeticException.code
    out := [] }

/-- The witness interface: four scalar registers (tile index, `i`, `j`,
accumulator), no vector lane at all, no scratch and no table. -/
def gemmWitnessSig : Sig :=
  { inputType := .bytes 291
    outputType := .unit
    regs := 4, vregs := 0, lanes := 0, scratch := 0, tables := 0, mem := 291
    statusSet := false }

/-- The interface the witness leaves: a status has been constructed and the
output is the four-cell `C` region. -/
def gemmWitnessOutSig : Sig :=
  { gemmWitnessSig with outputType := .bytes 4, statusSet := true }

/-- The `1×1×1` micro-kernel: a blocked traversal in the declared order, the
`i` and `j` axes of the loop nest with their packed index maps, and the `k`
reduction under the modular-`u32` contract, accumulating into register 3. -/
def gemmWitnessKernel (order : TraversalOrder) : Plan :=
  .tiled order Tiling.unit { m := 1, n := 1, k := 1 }
    (.loopNest { indexReg := 1, extent := 1, map := IndexMap.packed ⟨1, 1, 1⟩ }
      (.loopNest { indexReg := 2, extent := 1, map := IndexMap.packed ⟨1, 1, 1⟩ }
        (.seq (.setReg 3 0)
          (.seq (.reduce gemmWitnessContract 3 gemmWitnessA gemmWitnessB)
            (.storeReg gemmWitnessC (IndexMap.packed ⟨1, 1, 1⟩) 4 3)))))

/-- **The anti-vacuity witness.**  A complete released GEMM plan: classify the
raw header, dispatch on the layout class, run the kernel, set the status word,
build the output. -/
def gemmWitness : Plan :=
  .seq
    (.classifyRaw
      (.dispatchLayout
        (.seq (gemmWitnessKernel .ijk) (.setStatus .success))
        (.seq (gemmWitnessKernel .jik) (.setStatus .success))
        (.setStatus .unsupported))
      (.setStatus .invalid)
      (.setStatus .unsupported)
      (.setStatus .resourceExhausted))
    (.buildOutput gemmWitnessC)

/-! ### The witness is inside the released subset and type checks -/

/-- **The released subset is inhabited by a GEMM plan.** -/
theorem gemmWitness_inReleasedSubset : gemmWitness.inReleasedSubset = true := by
  decide

/-- The witness uses no vector operation, so the released profile's lack of
SIMD refuses nothing in it. -/
theorem gemmWitness_usesVector : gemmWitness.usesVector = false := by decide

/-- **The witness type checks** at the scalar interface. -/
theorem gemmWitness_typed : HasType gemmWitnessSig gemmWitness gemmWitnessOutSig := by
  decide

/-- The witness as a `CheckedPlan`. -/
def gemmWitnessChecked : CheckedPlan gemmWitnessSig gemmWitnessOutSig :=
  { plan := gemmWitness, typed := gemmWitness_typed }

/-! ### The witness compiles, validates, and refuses nothing -/

/-- **The compiled witness passes release validation.** -/
theorem gemmWitness_compiles : Wasm.validate (compile gemmWitnessChecked) = true :=
  compile_validates gemmWitnessChecked

/-- **The compilation of the witness contains no `unreachable`**: no node of a
real GEMM was refused by the translation. -/
theorem gemmWitness_no_unreachable :
    listHasUnreachable
      (bodyCode (envOf gemmWitnessSig gemmWitness) gemmWitnessSig.scratch gemmWitness)
      = false :=
  bodyCode_no_unreachable _ _ _ gemmWitness_inReleasedSubset

/-- The emitted artifact of the witness decodes back to a module that passes
release validation. -/
theorem gemmWitness_emit_decodes_valid :
    Wasm.decode (Artifact.emit (compile gemmWitnessChecked)) =
        .ok (compile gemmWitnessChecked) ∧
      Wasm.validate (compile gemmWitnessChecked) = true :=
  ⟨compile_decode_emit _, gemmWitness_compiles⟩

/-! ### What the witness computes -/

/-- The witness descriptor really classifies `valid`: the header cells are the
released ABI literals and the declared kind/mode triple is compatible. -/
theorem gemmWitnessMachine_classify (a b c : Nat) :
    (gemmWitnessMachine a b c).classify = Classification.valid := rfl

/-- The witness descriptor dispatches to the row-major continuation. -/
theorem gemmWitnessMachine_layoutClass (a b c : Nat) :
    (gemmWitnessMachine a b c).layoutClass = LayoutClass.rowMajor := rfl

/-- The witness configuration conforms to the witness interface. -/
theorem gemmWitnessMachine_conforms (a b c : Nat) :
    (gemmWitnessMachine a b c).Conforms gemmWitnessSig :=
  ⟨rfl, rfl, rfl, rfl, Nat.zero_le _⟩

/-- **The witness computes the product.**  On the `1×1×1` valid descriptor the
accumulator register holds exactly the modular-`u32` product of the `A` and `B`
elements. -/
theorem gemmWitness_eval_acc (a b c : Nat) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).reg 3 = a * b % 4294967296 := by
  show (0 + a * b) % 4294967296 = a * b % 4294967296
  rw [Nat.zero_add]

-- The three evaluation equations below run the whole plan on a 291-cell
-- memory.  The kernel's `storeReg` node deposits four little-endian cells at
-- cell 258 with `List.set`, and unfolding that through the 256-cell header
-- nests the reduction deeper, and costs more, than this file's default
-- budgets; both are therefore raised for the evaluation results below.
set_option maxHeartbeats 4000000
set_option maxRecDepth 100000

/-- The witness sets the released success status word. -/
theorem gemmWitness_eval_status (a b c : Nat) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).status = Status.success.code := rfl

/-- **The witness publishes its result.**  `buildOutput` publishes the declared
`C` region, which is exactly the four-cell little-endian image of the
accumulated product the kernel stored. -/
theorem gemmWitness_eval_out (a b c : Nat) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).out =
      leBytes (a * b % 4294967296) 4 := by
  show leBytes ((0 + a * b) % 4294967296) 4 = leBytes (a * b % 4294967296) 4
  rw [Nat.zero_add]

/-- The memory the witness leaves: the entry memory with the four-cell
little-endian image of the accumulated product deposited at `C`.
`gemmWitness_writes_C` reads that image back and `gemmWitness_eval_mem_outside`
is the frame. -/
theorem gemmWitness_eval_mem (a b c : Nat) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).mem =
      gather (gemmWitnessMem a b c) 258
        (leBytes (a * b % 4294967296) 4) (fun i => i) 4 0 := by
  show gather (gemmWitnessMem a b c) 258
      (leBytes ((0 + a * b) % 4294967296) 4) (fun i => i) 4 0 = _
  rw [Nat.zero_add]

/-- **Frame.**  Every memory cell outside the four the `C` store writes keeps
its entry value: the witness touches nothing but `C`. -/
theorem gemmWitness_eval_mem_outside (a b c x : Nat) (hx : x < 258 ∨ 262 ≤ x) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).mem.getD x 0 =
      (gemmWitnessMem a b c).getD x 0 := by
  rw [gemmWitness_eval_mem]
  exact gather_getD_outside _ _ 4 (gemmWitnessMem a b c) 258 0 x (by omega)

/-- The `alpha` scalar of the witness descriptor, read out of the header field
SPEC §8.3 puts at cells `168..175`. -/
def gemmWitnessAlpha : Nat := leWord gemmWitnessHeader 168 8

/-- The `beta` scalar of the witness descriptor, read out of the header field
SPEC §8.3 puts at cells `184..191`. -/
def gemmWitnessBeta : Nat := leWord gemmWitnessHeader 184 8

/-- The witness descriptor declares `alpha = 1`. -/
theorem gemmWitnessAlpha_eq : gemmWitnessAlpha = 1 := rfl

/-- The witness descriptor declares `beta = 0`. -/
theorem gemmWitnessBeta_eq : gemmWitnessBeta = 0 := rfl

/-- **The witness really computes GEMM: the `C` region holds the result.**
Reading the declared `C` region back little-endian at the stored width after
evaluation gives `alpha · A · B + beta · C` in the modular `u32` ring of the
declared arithmetic contract, with `alpha` and `beta` the scalars this
descriptor declares.

Since the descriptor fixes `alpha = 1` and `beta = 0`, this is the
`alpha = 1, beta = 0` instance of SPEC §8's `C ← alpha · op(A) · op(B) +
beta · C`; the plan carries no scaling node, so nothing here proves the general
`alpha`/`beta` form and nothing here claims to. -/
theorem gemmWitness_writes_C (a b c : Nat) :
    leWord (gemmWitness.eval (gemmWitnessMachine a b c)).mem gemmWitnessC.base 4 =
      (gemmWitnessAlpha * (a * b) + gemmWitnessBeta * c) % 4294967296 := by
  have hlen : (gemmWitnessMem a b c).length = 291 := gemmWitnessMem_length a b c
  have hinside : ∀ k, k < 4 →
      (gather (gemmWitnessMem a b c) 258 (leBytes (a * b % 4294967296) 4)
          (fun i => i) 4 0).getD (258 + k) 0 =
        (leBytes (a * b % 4294967296) 4).getD (0 + k) 0 := by
    intro k hk
    have h := gather_getD_inside (leBytes (a * b % 4294967296) 4) (fun i => i) 4
      (gemmWitnessMem a b c) 258 0 (258 + k) (by omega) (by omega)
      (by rw [hlen]; omega)
    have he : 258 + k - 258 = k := by omega
    rw [h]
    simp only [he, Nat.zero_add]
  have hbase : gemmWitnessC.base = 258 := rfl
  rw [gemmWitness_eval_mem, hbase,
    leWord_ext 4 _ (leBytes (a * b % 4294967296) 4) 258 0 hinside,
    leWord_leBytes]
  have hpow : (256 : Nat) ^ 4 = 4294967296 := by decide
  rw [hpow, Nat.mod_eq_of_lt (Nat.mod_lt _ (by omega)),
    gemmWitnessAlpha_eq, gemmWitnessBeta_eq, Nat.one_mul, Nat.zero_mul,
    Nat.add_zero]

/-- Type safety instantiated at the witness. -/
theorem gemmWitness_eval_conforms (a b c : Nat) :
    (gemmWitness.eval (gemmWitnessMachine a b c)).Conforms gemmWitnessOutSig :=
  hasType_preservation gemmWitness_typed _ (gemmWitnessMachine_conforms a b c)

/-- **`Plan.inReleasedSubset` is inhabited**, and its inhabitant really is a
compiled, validating, `unreachable`-free artifact. -/
theorem exists_inReleasedSubset_checkedPlan :
    ∃ (s t : Sig) (c : CheckedPlan s t),
      c.plan.inReleasedSubset = true ∧
      Wasm.validate (compile c) = true ∧
      listHasUnreachable (bodyCode (envOf s c.plan) s.scratch c.plan) = false :=
  ⟨gemmWitnessSig, gemmWitnessOutSig, gemmWitnessChecked,
    gemmWitness_inReleasedSubset, gemmWitness_compiles, gemmWitness_no_unreachable⟩

/-! ## The input-dependent GEMM kernel (SPEC §11.1; SPEC §3 input totality)

`gemmWitness` above is a *fixed-extent* plan: its loop bounds and its reduction
length are literals of the plan text, so it computes one shape and one shape
only.  `gemmKernel` is the plan that removes that limitation, using the two
constructors `GNAF/Plan.lean` added for it.  It

1. classifies the raw ABI header (`Plan.classifyRaw`) and dispatches on the
   declared layout class (`Plan.dispatchLayout`);
2. **loads** the declared extents `m`, `n`, `k` out of the header — SPEC §8.3
   puts them at cells `16..23`, `24..31`, `32..39`, little-endian — into scalar
   registers `3`, `4`, `5` with `Plan.loadReg` (`gemmKernel_loads_extents`);
3. runs an `i`/`j`/`k` nest of three `Plan.loopReg` loops **bounded by exactly
   those registers**, so every trip count is a function of the input and of
   nothing in the plan text (`gemmKernelNest_trips`);
4. accumulates the reduction, one multiply-accumulate per `k` iteration;
5. deposits each result into the declared `C` region with `Plan.storeReg`, as
   the four-cell little-endian `u32` image the released contract fixes;
6. sets the released status word and constructs the output from `C`.

### Why the addressing is computed rather than declared

A GNAF `IndexMap` is affine with coefficients *fixed by the plan text*, so no
index map can express the address of `A[i,t]`: that address is
`256 + 4 · (i · k + t)` and `k` is the loaded extent, not a literal.  The kernel
therefore computes each element address into scalar register `0` with
`Plan.scalarOp` and reads it through `IndexMap.idB`, whose `storeAddr` is
`base + reg 0` — the same addressing `storeReg` and `loadReg` share.  Address
arithmetic is consequently *data* in this plan, exactly as the extents are.
This is what makes the kernel shape-generic; it is also why the theorems below
never mention a literal extent.

### The register and memory map

| register | contents |
|---|---|
| `0` | computed element address (the `b` slot `storeAddr` reads) |
| `3`, `4`, `5` | the loaded extents `m`, `n`, `k` |
| `6`, `7`, `8` | the `i`, `j`, `k` loop counters (`loopReg` index registers) |
| `9` | the accumulator |
| `10`, `11`, `12` | the loaded `A` element, `B` element and their product |
| `13` | the stored element width, `4` cells |

The observable memory is 1024 cells: the 256-cell SPEC §8.3 header, then 256
cells of `A` at `256`, 256 cells of `B` at `512`, and 256 cells of `C` at `768`.
Every element is a four-cell little-endian `u32`, so each region holds up to 64
elements — the source of the `m·k ≤ 64`, `k·n ≤ 64`, `m·n ≤ 64` hypotheses
below.  As with `gemmWitness`, no claim is made that this cell image coincides
with the byte image of `Gemm/ABI.lean`; that is a different machine model and it
is not proved anywhere.

### The modular-`u32` contract

`ScalarOp` has no truncating operation, so the `k` accumulation runs over
unbounded `Nat` and truncation happens where the released ABI puts it: the
four-cell `Plan.storeReg` into `C` keeps the result modulo `2 ^ 32`.
`gemmKernel_C_is_contract_fold` proves that what lands in `C` is exactly the
fold of `ArithmeticContract.step` for the modular-`u32` contract
`gemmKernelContract` — the same contract `gemmWitness` declares — so the
reduction is performed under the declared contract of SPEC §8.2 and not merely
under a formula that happens to agree with it.

### The reach of `gemmKernel_writes_C`, stated exactly

`gemmKernel_writes_C` is proved for **arbitrary `m`, `n`, `k` and an arbitrary
entry memory**, under exactly these hypotheses, all of them discharged by `rfl`
or `decide` on the concrete descriptors further down:

* the configuration conforms to the kernel's interface (`regs` file of 14 cells,
  1024 memory cells);
* the descriptor classifies `valid` and its layout class is `rowMajor` — those
  are the two continuations the kernel body sits in;
* the header words at `16`, `24`, `32` are `m`, `n`, `k`;
* `m, n, k ≤ loopRegMaxTrips`, so the `loopReg` clamp is inert (it is the
  released `i32` ceiling, so this excludes nothing the ABI can present);
* `m·k ≤ 64`, `k·n ≤ 64`, `m·n ≤ 64`, so `A`, `B` and `C` fit their declared
  256-cell regions and are therefore disjoint;
* `i < m` and `j < n`.

Three descriptors of three *different* shapes are then exhibited, and the same
plan is proved to compute the right product on each:
`gemmKernel_writes_C_1x1x1` (`1×1×1`), `gemmKernel_writes_C_2x2x2` (`2×2×2`, all
four entries) and `gemmKernel_writes_C_1x3x2` (`1×3×2`, a rectangular shape with
`k ≠ n`, all three entries).  That is the evidence that the fixed-extent
limitation is gone: one `CheckedPlan`, three shapes, no plan-text extent.

### What is still *not* proved here

* **Nothing relates this plan to the emitted Wasm.**  `compile_refines` remains
  omitted for the reasons given at the top of this file; `gemmKernel_compiles`
  says the compiled module validates, nothing more.
* **`alpha` and `beta` are not applied.**  The kernel computes `A · B`, the
  `alpha = 1`, `beta = 0` instance of SPEC §8, exactly as `gemmWitness` does.
  The kernel contains no scaling node.
* **The declared strides are not consulted.**  The kernel assumes the packed
  row-major layout that `Machine.layoutClass` selects; the hypothesis
  `m.layoutClass = .rowMajor` is what licenses that, and a descriptor declaring
  a non-packed row-major view would still enter the kernel.  Nothing here proves
  anything about such a descriptor.
* **The batch axis is not traversed.**  This is the `batch = 1` instance.
* **The region bound is a *hypothesis*, not a check.**  A descriptor declaring
  `m·n > 64` is not refused by this plan; it is simply outside the theorem.  The
  released classifier does not check it either (`Machine.classify` checks the
  header literals, the kind/mode triple and the invocation extent), so no
  disjointness or capacity claim is made beyond the stated hypotheses.
-/

/-! ### Register and store lemmas the kernel proof needs -/

/-- Writing one register leaves every other register alone. -/
theorem Machine.reg_withReg_ne (m : Machine) {i j : Nat} (v : Nat) (h : j ≠ i) :
    (m.withReg i v).reg j = m.reg j := by
  simp [Machine.reg, Machine.withReg, List.getD_eq_getElem?_getD,
    List.getElem?_set_ne (Ne.symm h)]

/-- The evaluation equation of a scalar operation. -/
@[simp] theorem eval_scalarOp (op : ScalarOp) (dst a b : Nat) (m : Machine) :
    (Plan.scalarOp op dst a b).eval m = m.withReg dst (op.apply (m.reg a) (m.reg b)) := rfl

/-- The evaluation equation of an immediate load. -/
@[simp] theorem eval_setReg (dst v : Nat) (m : Machine) :
    (Plan.setReg dst v).eval m = m.withReg dst v := rfl

/-- A register store writes memory, never the register file. -/
@[simp] theorem storeReg_reg (dst : RegionRef) (map : IndexMap) (w src r : Nat)
    (m : Machine) : ((Plan.storeReg dst map w src).eval m).reg r = m.reg r := rfl

/-- A register store never changes the size of the register file. -/
@[simp] theorem storeReg_regs_length (dst : RegionRef) (map : IndexMap) (w src : Nat)
    (m : Machine) : ((Plan.storeReg dst map w src).eval m).regs.length = m.regs.length := rfl

/-! ### The regions and index maps of the kernel

`A`, `B` and `C` are the three 256-cell element regions of the observable
memory; `M`, `N` and `K` field are the three header words SPEC §8.3 declares the
extents in. -/

/-- The declared `A` region: 256 cells, that is up to 64 `u32` elements. -/
def gemmKernelA : RegionRef := { base := 256, count := 256 }
/-- The declared `B` region. -/
def gemmKernelB : RegionRef := { base := 512, count := 256 }
/-- The declared `C` region — the output the kernel publishes. -/
def gemmKernelC : RegionRef := { base := 768, count := 256 }
/-- The header field SPEC §8.3 puts `m` in: cells `16..23`. -/
def gemmKernelMField : RegionRef := { base := 16, count := 8 }
/-- The header field SPEC §8.3 puts `n` in: cells `24..31`. -/
def gemmKernelNField : RegionRef := { base := 24, count := 8 }
/-- The header field SPEC §8.3 puts `k` in: cells `32..39`. -/
def gemmKernelKField : RegionRef := { base := 32, count := 8 }

/-- The constant index map: the address of a header field does not depend on
any loop index. -/
def gemmZeroMap : IndexMap := { c0 := 0, cb := 0, ci := 0, cj := 0 }

@[simp] theorem gemmZeroMap_apply (b i j : Nat) : gemmZeroMap.apply b i j = 0 := by
  simp [IndexMap.apply, gemmZeroMap]

@[simp] theorem storeAddr_gemmZeroMap (m : Machine) (r : RegionRef) :
    storeAddr m r gemmZeroMap = r.base := by
  simp [storeAddr]

/-- `IndexMap.idB` addresses `base + reg 0`: this is the map through which the
kernel's *computed* addresses reach `loadReg` and `storeReg`. -/
@[simp] theorem storeAddr_idB (m : Machine) (r : RegionRef) :
    storeAddr m r IndexMap.idB = r.base + m.reg 0 := by
  simp [storeAddr]

/-! ### The plan

Read bottom up: `gemmKernelKBody` is one multiply-accumulate at computed
addresses, `gemmKernelJBody` is one output element, `gemmKernelIBody` is one
output row, `gemmKernelNest` is the whole traversal, and `gemmKernelLoad` is the
phase that puts the header's extents into the loop-bound registers. -/

/-- The `k` body: compute the address of `A[i,t]` into register `0`, load it,
compute the address of `B[t,j]`, load it, multiply and accumulate.  Every
address is `4 · (row · extent + column)` with the extent read from a register,
which is exactly what a fixed index map cannot express. -/
def gemmKernelKBody : Plan :=
  .seq (.scalarOp .mul 0 6 5)
  (.seq (.scalarOp .add 0 0 8)
  (.seq (.scalarOp .mul 0 0 13)
  (.seq (.loadReg 10 gemmKernelA IndexMap.idB 4)
  (.seq (.scalarOp .mul 0 8 4)
  (.seq (.scalarOp .add 0 0 7)
  (.seq (.scalarOp .mul 0 0 13)
  (.seq (.loadReg 11 gemmKernelB IndexMap.idB 4)
  (.seq (.scalarOp .mul 12 10 11)
        (.scalarOp .add 9 9 12)))))))))

/-- The `j` body: clear the accumulator, run the reduction for `reg 5` steps —
the loaded `k` — then compute the address of `C[i,j]` and store the accumulator
there as four little-endian cells. -/
def gemmKernelJBody : Plan :=
  .seq (.setReg 9 0)
  (.seq (.loopReg 8 5 gemmZeroMap gemmKernelKBody)
  (.seq (.scalarOp .mul 0 6 4)
  (.seq (.scalarOp .add 0 0 7)
  (.seq (.scalarOp .mul 0 0 13)
        (.storeReg gemmKernelC IndexMap.idB 4 9)))))

/-- The `i` body: the `j` loop, bounded by the loaded `n`. -/
def gemmKernelIBody : Plan := .loopReg 7 4 gemmZeroMap gemmKernelJBody

/-- The traversal: the `i` loop, bounded by the loaded `m`. -/
def gemmKernelNest : Plan := .loopReg 6 3 gemmZeroMap gemmKernelIBody

/-- The load phase: publish the stored element width in register `13`, then
lift the three header extents into the three loop-bound registers. -/
def gemmKernelLoad : Plan :=
  .seq (.setReg 13 4)
  (.seq (.loadReg 3 gemmKernelMField gemmZeroMap 8)
  (.seq (.loadReg 4 gemmKernelNField gemmZeroMap 8)
        (.loadReg 5 gemmKernelKField gemmZeroMap 8)))

/-- The kernel body: load the extents, then run the nest they bound. -/
def gemmKernelCore : Plan := .seq gemmKernelLoad gemmKernelNest

/-- **The input-dependent GEMM plan.**  Classify the raw header, dispatch on
the layout class, run the register-bounded kernel on the row-major branch, set
the released status word, and construct the output from the declared `C`
region.  Layout classes other than row major are refused with the `unsupported`
status rather than computed wrongly. -/
def gemmKernel : Plan :=
  .seq
    (.classifyRaw
      (.dispatchLayout
        (.seq gemmKernelCore (.setStatus .success))
        (.setStatus .unsupported)
        (.setStatus .unsupported))
      (.setStatus .invalid)
      (.setStatus .unsupported)
      (.setStatus .resourceExhausted))
    (.buildOutput gemmKernelC)

/-! ### The reference product

These are the *specification* side: what the `A` and `B` elements of a memory
are, and what the dot product of a row and a column is.  They are defined on the
memory, not on the plan, so the correctness theorems below compare the plan's
result against something the plan had no part in defining. -/

/-- The `u32` element of a memory at an element index: four little-endian
cells. -/
def gemmElemWord (mem : List Nat) (base idx : Nat) : Nat := leWord mem (base + 4 * idx) 4

/-- The `A` element at logical position `(i, t)` of a packed row-major `m × k`
view based at cell `256`. -/
def gemmAWord (mem : List Nat) (K i t : Nat) : Nat := gemmElemWord mem 256 (i * K + t)

/-- The `B` element at logical position `(t, j)` of a packed row-major `k × n`
view based at cell `512`. -/
def gemmBWord (mem : List Nat) (N t j : Nat) : Nat := gemmElemWord mem 512 (t * N + j)

/-- `gemmSumFrom f n s = f s + f (s+1) + ⋯ + f (s+n-1)`. -/
def gemmSumFrom (f : Nat → Nat) : Nat → Nat → Nat
  | 0, _ => 0
  | n + 1, s => f s + gemmSumFrom f n (s + 1)

/-- The reference dot product of row `i` of `A` and column `j` of `B`, over the
full loaded extent `k`.  This is the specification the kernel is checked
against. -/
def gemmDot (mem : List Nat) (N K i j : Nat) : Nat :=
  gemmSumFrom (fun t => gemmAWord mem K i t * gemmBWord mem N t j) K 0

/-! ### The `k` body: one multiply-accumulate -/

/-- One `k` iteration adds exactly one product to the accumulator, touches no
memory, and preserves every register the enclosing loops rely on. -/
theorem gemmKernelKBody_eval (mm : Machine) (N K i j t : Nat)
    (hlen : mm.regs.length = 14)
    (h4 : mm.reg 4 = N) (h5 : mm.reg 5 = K) (h6 : mm.reg 6 = i) (h7 : mm.reg 7 = j)
    (h8 : mm.reg 8 = t) (h13 : mm.reg 13 = 4) :
    (gemmKernelKBody.eval mm).mem = mm.mem ∧
    (gemmKernelKBody.eval mm).regs.length = 14 ∧
    (gemmKernelKBody.eval mm).reg 3 = mm.reg 3 ∧
    (gemmKernelKBody.eval mm).reg 4 = N ∧
    (gemmKernelKBody.eval mm).reg 5 = K ∧
    (gemmKernelKBody.eval mm).reg 6 = i ∧
    (gemmKernelKBody.eval mm).reg 7 = j ∧
    (gemmKernelKBody.eval mm).reg 13 = 4 ∧
    (gemmKernelKBody.eval mm).reg 9 =
      mm.reg 9 + gemmAWord mm.mem K i t * gemmBWord mm.mem N t j := by
  simp only [gemmKernelKBody, Plan.eval_seq, eval_scalarOp, eval_loadReg,
    loadedReg, storeAddr_idB, Machine.mem_withReg]
  refine ⟨?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_⟩ <;>
    simp [Machine.reg_withReg_ne, hlen, h4, h5, h6, h7, h8, h13, gemmAWord, gemmBWord,
      gemmElemWord, gemmKernelA, gemmKernelB, ScalarOp.apply, Nat.mul_comm]

/-! ### The `k` loop: the reduction -/

/-- **The reduction is the sum.**  `n` iterations of the `k` body add exactly
the `n` products the reference sum names.  The memory is untouched throughout,
so the elements read are the entry elements. -/
theorem gemmKernelKLoop (N K i j : Nat) : ∀ (n t0 : Nat) (mm : Machine),
    mm.regs.length = 14 → mm.reg 4 = N → mm.reg 5 = K → mm.reg 6 = i → mm.reg 7 = j →
    mm.reg 13 = 4 →
    (Plan.runLoop gemmKernelKBody.eval 8 n t0 mm).mem = mm.mem ∧
    (Plan.runLoop gemmKernelKBody.eval 8 n t0 mm).regs.length = 14 ∧
    (Plan.runLoop gemmKernelKBody.eval 8 n t0 mm).reg 3 = mm.reg 3 ∧
    (Plan.runLoop gemmKernelKBody.eval 8 n t0 mm).reg 4 = N ∧
    (Plan.runLoop gemmKernelKBody.eval 8 n t0 mm).reg 5 = K ∧
    (Plan.runLoop gemmKernelKBody.eval 8 n t0 mm).reg 6 = i ∧
    (Plan.runLoop gemmKernelKBody.eval 8 n t0 mm).reg 7 = j ∧
    (Plan.runLoop gemmKernelKBody.eval 8 n t0 mm).reg 13 = 4 ∧
    (Plan.runLoop gemmKernelKBody.eval 8 n t0 mm).reg 9 =
      mm.reg 9 + gemmSumFrom (fun t => gemmAWord mm.mem K i t * gemmBWord mm.mem N t j) n t0 := by
  intro n
  induction n with
  | zero =>
    intro t0 mm hlen h4 h5 h6 h7 h13
    exact ⟨rfl, hlen, rfl, h4, h5, h6, h7, h13, rfl⟩
  | succ n ih =>
    intro t0 mm hlen h4 h5 h6 h7 h13
    rw [Plan.runLoop_succ]
    have hset : (mm.withReg 8 t0).reg 8 = t0 :=
      Machine.reg_withReg_same mm 8 t0 (by omega)
    have hb := gemmKernelKBody_eval (mm.withReg 8 t0) N K i j t0
      (by simpa using hlen)
      (by rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h4)
      (by rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h5)
      (by rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h6)
      (by rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h7)
      hset
      (by rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h13)
    obtain ⟨hm, hl, h3', h4', h5', h6', h7', h13', h9'⟩ := hb
    have hmem : (gemmKernelKBody.eval (mm.withReg 8 t0)).mem = mm.mem := hm
    have h9m : (mm.withReg 8 t0).reg 9 = mm.reg 9 := Machine.reg_withReg_ne _ _ (by decide)
    have hmm : (mm.withReg 8 t0).mem = mm.mem := rfl
    have := ih (t0 + 1) (gemmKernelKBody.eval (mm.withReg 8 t0)) hl h4' h5' h6' h7' h13'
    obtain ⟨im, il, i3, i4, i5, i6, i7, i13, i9⟩ := this
    rw [hmem] at im i9
    refine ⟨im, il, ?_, i4, i5, i6, i7, i13, ?_⟩
    · rw [i3, h3', Machine.reg_withReg_ne _ _ (by decide)]
    · rw [i9, h9', hmm, h9m]
      show _ = mm.reg 9 + (gemmAWord mm.mem K i t0 * gemmBWord mm.mem N t0 j +
        gemmSumFrom (fun t => gemmAWord mm.mem K i t * gemmBWord mm.mem N t j) n (t0 + 1))
      omega

/-! ### The `j` body: one output element, stored -/

/-- One `j` iteration leaves the memory with the four-cell little-endian image
of the complete dot product deposited at `C[i,j]`, and nothing else changed. -/
theorem gemmKernelJBody_eval (mm : Machine) (M N K i j : Nat)
    (hlen : mm.regs.length = 14) (hK : K ≤ loopRegMaxTrips)
    (h3 : mm.reg 3 = M) (h4 : mm.reg 4 = N) (h5 : mm.reg 5 = K) (h6 : mm.reg 6 = i)
    (h7 : mm.reg 7 = j) (h13 : mm.reg 13 = 4) :
    (gemmKernelJBody.eval mm).mem =
      gather mm.mem (768 + 4 * (i * N + j))
        (leBytes (gemmDot mm.mem N K i j) 4) (fun x => x) 4 0 ∧
    (gemmKernelJBody.eval mm).regs.length = 14 ∧
    (gemmKernelJBody.eval mm).reg 3 = M ∧
    (gemmKernelJBody.eval mm).reg 4 = N ∧
    (gemmKernelJBody.eval mm).reg 5 = K ∧
    (gemmKernelJBody.eval mm).reg 6 = i ∧
    (gemmKernelJBody.eval mm).reg 13 = 4 := by
  have h9z : (mm.withReg 9 0).reg 9 = 0 :=
    Machine.reg_withReg_same _ _ _ (by rw [hlen]; omega)
  have hl1 : (mm.withReg 9 0).regs.length = 14 := by simpa using hlen
  have e3 : (mm.withReg 9 0).reg 3 = M := by
    rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h3
  have e4 : (mm.withReg 9 0).reg 4 = N := by
    rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h4
  have e5 : (mm.withReg 9 0).reg 5 = K := by
    rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h5
  have e6 : (mm.withReg 9 0).reg 6 = i := by
    rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h6
  have e7 : (mm.withReg 9 0).reg 7 = j := by
    rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h7
  have e13 : (mm.withReg 9 0).reg 13 = 4 := by
    rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h13
  have hmem1 : (mm.withReg 9 0).mem = mm.mem := rfl
  have hloop : (Plan.loopReg 8 5 gemmZeroMap gemmKernelKBody).eval (mm.withReg 9 0)
      = Plan.runLoop gemmKernelKBody.eval 8 K 0 (mm.withReg 9 0) := by
    rw [eval_loopReg, e5, min_loopRegMaxTrips hK]
  obtain ⟨km, kl, k3, k4, k5, k6, k7, k13, k9⟩ :=
    gemmKernelKLoop N K i j K 0 (mm.withReg 9 0) hl1 e4 e5 e6 e7 e13
  rw [hmem1] at km k9
  rw [h9z, Nat.zero_add] at k9
  rw [e3] at k3
  refine ⟨?_, ?_, ?_, ?_, ?_, ?_, ?_⟩
  · simp only [gemmKernelJBody, Plan.eval_seq, eval_setReg]
    rw [hloop]
    simp [Machine.reg_withReg_ne, kl, k4, k6, k7, k13, k9, km, gemmKernelC,
      ScalarOp.apply, storedRegMem, gemmDot, Nat.mul_comm]
  all_goals
    simp only [gemmKernelJBody, Plan.eval_seq, eval_setReg] <;>
    rw [hloop] <;>
    simp [Machine.reg_withReg_ne, kl, k3, k4, k5, k6, k13, ScalarOp.apply, -eval_storeReg]

/-! ### Frame and congruence lemmas for the two outer loops -/

/-- A packed row-major index of a bounded position is bounded by the element
count. -/
theorem gemmIndex_lt {i M t K : Nat} (hi : i < M) (ht : t < K) : i * K + t < M * K := by
  have h1 : (i + 1) * K ≤ M * K := Nat.mul_le_mul hi (Nat.le_refl K)
  rw [Nat.succ_mul] at h1
  omega

/-- Reading back the image a register store deposited gives the register,
truncated to the stored width. -/
theorem leWord_gather (w : Nat) (mem : List Nat) (addr v : Nat)
    (h : addr + w ≤ mem.length) :
    leWord (gather mem addr (leBytes v w) (fun x => x) w 0) addr w = v % 256 ^ w := by
  rw [← leWord_leBytes w v]
  refine leWord_ext w _ _ _ _ ?_
  intro k hk
  rw [gather_getD_inside (leBytes v w) (fun x => x) w mem addr 0 (addr + k)
    (by omega) (by omega) (by omega)]
  have he : addr + k - addr = k := by omega
  rw [he, Nat.zero_add]

/-- A little-endian read depends only on the cells it reads. -/
theorem leWord_agree {mem mem0 : List Nat} {a w b : Nat}
    (h : ∀ x, x < b → mem.getD x 0 = mem0.getD x 0) (hb : a + w ≤ b) :
    leWord mem a w = leWord mem0 a w :=
  leWord_ext w mem mem0 a a (fun k hk => h (a + k) (by omega))

/-- A sum depends only on the summands it names. -/
theorem sumFrom_congr (f g : Nat → Nat) : ∀ (n s : Nat),
    (∀ t, s ≤ t → t < s + n → f t = g t) → gemmSumFrom f n s = gemmSumFrom g n s := by
  intro n
  induction n with
  | zero => intro s _; rfl
  | succ n ih =>
    intro s h
    show f s + gemmSumFrom f n (s + 1) = g s + gemmSumFrom g n (s + 1)
    rw [h s (Nat.le_refl _) (by omega), ih (s + 1) (fun t h1 h2 => h t (by omega) (by omega))]

/-- **The dot product is insensitive to the output region.**  `A` and `B` lie
below cell `768` whenever the shape fits its regions, so writing into `C` never
changes a later dot product.  This is the fact that makes the loop nest's
iterations independent. -/
theorem gemmDot_congr {mem mem0 : List Nat} {M N K i j : Nat}
    (h : ∀ x, x < 768 → mem.getD x 0 = mem0.getD x 0)
    (hMK : M * K ≤ 64) (hKN : K * N ≤ 64) (hi : i < M) (hj : j < N) :
    gemmDot mem N K i j = gemmDot mem0 N K i j := by
  refine sumFrom_congr _ _ K 0 ?_
  intro t _ ht
  have ht' : t < K := by omega
  have hA : i * K + t < 64 := Nat.lt_of_lt_of_le (gemmIndex_lt hi ht') hMK
  have hB : t * N + j < 64 := Nat.lt_of_lt_of_le (gemmIndex_lt ht' hj) hKN
  have e1 : gemmAWord mem K i t = gemmAWord mem0 K i t := by
    unfold gemmAWord gemmElemWord
    exact leWord_agree h (by omega)
  have e2 : gemmBWord mem N t j = gemmBWord mem0 N t j := by
    unfold gemmBWord gemmElemWord
    exact leWord_agree h (by omega)
  rw [e1, e2]

/-- The four-cell modulus. -/
theorem pow_256_four : (256 : Nat) ^ 4 = 4294967296 := by decide

/-! ### The `j` loop: one output row -/

/-- **One output row.**  After `n` iterations of the `j` loop the cells of
`C[i, j0 .. j0+n)` hold the modular dot products, every cell outside that
contiguous block is untouched, and the loop-bound registers survive. -/
theorem gemmKernelJLoop (mem0 : List Nat) (M N K i p : Nat) (hp : p = i * N)
    (hK : K ≤ loopRegMaxTrips) (hMK : M * K ≤ 64) (hKN : K * N ≤ 64) (hi : i < M) :
    ∀ (n j0 : Nat) (mm : Machine),
      mm.regs.length = 14 → mm.mem.length = 1024 →
      (∀ x, x < 768 → mm.mem.getD x 0 = mem0.getD x 0) →
      mm.reg 3 = M → mm.reg 4 = N → mm.reg 5 = K → mm.reg 6 = i → mm.reg 13 = 4 →
      p + j0 + n ≤ 64 → j0 + n ≤ N →
      (Plan.runLoop gemmKernelJBody.eval 7 n j0 mm).mem.length = 1024 ∧
      (Plan.runLoop gemmKernelJBody.eval 7 n j0 mm).regs.length = 14 ∧
      (Plan.runLoop gemmKernelJBody.eval 7 n j0 mm).reg 3 = M ∧
      (Plan.runLoop gemmKernelJBody.eval 7 n j0 mm).reg 4 = N ∧
      (Plan.runLoop gemmKernelJBody.eval 7 n j0 mm).reg 5 = K ∧
      (Plan.runLoop gemmKernelJBody.eval 7 n j0 mm).reg 6 = i ∧
      (Plan.runLoop gemmKernelJBody.eval 7 n j0 mm).reg 13 = 4 ∧
      (∀ x, x < 768 + 4 * (p + j0) ∨ 768 + 4 * (p + j0 + n) ≤ x →
        (Plan.runLoop gemmKernelJBody.eval 7 n j0 mm).mem.getD x 0 = mm.mem.getD x 0) ∧
      (∀ j, j0 ≤ j → j < j0 + n →
        leWord (Plan.runLoop gemmKernelJBody.eval 7 n j0 mm).mem (768 + 4 * (p + j)) 4 =
          gemmDot mem0 N K i j % 4294967296) := by
  intro n
  induction n with
  | zero =>
    intro j0 mm hlen hml _ h3 h4 h5 h6 h13 _ _
    exact ⟨hml, hlen, h3, h4, h5, h6, h13, fun _ _ => rfl, fun j _ hj2 => absurd hj2 (by omega)⟩
  | succ n ih =>
    intro j0 mm hlen hml hag h3 h4 h5 h6 h13 hb1 hb2
    rw [Plan.runLoop_succ]
    -- the machine entering the body
    have hlen' : (mm.withReg 7 j0).regs.length = 14 := by simpa using hlen
    have e3 : (mm.withReg 7 j0).reg 3 = M := by
      rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h3
    have e4 : (mm.withReg 7 j0).reg 4 = N := by
      rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h4
    have e5 : (mm.withReg 7 j0).reg 5 = K := by
      rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h5
    have e6 : (mm.withReg 7 j0).reg 6 = i := by
      rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h6
    have e7 : (mm.withReg 7 j0).reg 7 = j0 :=
      Machine.reg_withReg_same _ _ _ (by rw [hlen]; omega)
    have e13 : (mm.withReg 7 j0).reg 13 = 4 := by
      rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h13
    have emem : (mm.withReg 7 j0).mem = mm.mem := rfl
    obtain ⟨bm, bl, b3, b4, b5, b6, b13⟩ :=
      gemmKernelJBody_eval (mm.withReg 7 j0) M N K i j0 hlen' hK e3 e4 e5 e6 e7 e13
    rw [emem] at bm
    -- the dot product is the one computed from the reference memory
    have hj0N : j0 < N := by omega
    have hdot : gemmDot mm.mem N K i j0 = gemmDot mem0 N K i j0 :=
      gemmDot_congr hag hMK hKN hi hj0N
    rw [hdot] at bm
    have haddr : 768 + 4 * (i * N + j0) = 768 + 4 * (p + j0) := by rw [hp]
    rw [haddr] at bm
    -- length, frame and agreement of the body's write
    have bml : (gemmKernelJBody.eval (mm.withReg 7 j0)).mem.length = 1024 := by
      rw [bm, gather_length]; exact hml
    have bfr : ∀ x, x < 768 + 4 * (p + j0) ∨ 768 + 4 * (p + j0) + 4 ≤ x →
        (gemmKernelJBody.eval (mm.withReg 7 j0)).mem.getD x 0 = mm.mem.getD x 0 := by
      intro x hx
      rw [bm]
      exact gather_getD_outside _ _ 4 mm.mem (768 + 4 * (p + j0)) 0 x (by omega)
    have bag : ∀ x, x < 768 →
        (gemmKernelJBody.eval (mm.withReg 7 j0)).mem.getD x 0 = mem0.getD x 0 := by
      intro x hx
      rw [bfr x (Or.inl (by omega))]
      exact hag x hx
    have bread : leWord (gemmKernelJBody.eval (mm.withReg 7 j0)).mem (768 + 4 * (p + j0)) 4 =
        gemmDot mem0 N K i j0 % 4294967296 := by
      rw [bm, leWord_gather 4 mm.mem _ _ (by rw [hml]; omega), pow_256_four]
    -- the induction hypothesis on the remaining iterations
    obtain ⟨im, il, i3, i4, i5, i6, i13, ifr, ird⟩ :=
      ih (j0 + 1) (gemmKernelJBody.eval (mm.withReg 7 j0)) bl bml bag b3 b4 b5 b6 b13
        (by omega) (by omega)
    refine ⟨im, il, i3, i4, i5, i6, i13, ?_, ?_⟩
    · intro x hx
      rw [ifr x (by omega), bfr x (by omega)]
    · intro j hj1 hj2
      rcases Nat.eq_or_lt_of_le hj1 with hje | hjl
      · subst hje
        rw [← bread]
        refine leWord_ext 4 _ _ _ _ ?_
        intro k hk
        exact ifr (768 + 4 * (p + j0) + k) (Or.inl (by omega))
      · exact ird j hjl (by omega)

/-! ### The `i` loop: the whole traversal -/

/-- **The whole traversal.**  After `n` iterations of the `i` loop every cell of
`C[i0 .. i0+n) × [0, n)` holds its modular dot product, and everything outside
the contiguous block those rows occupy is untouched. -/
theorem gemmKernelILoop (mem0 : List Nat) (M N K : Nat)
    (hK : K ≤ loopRegMaxTrips) (hNt : N ≤ loopRegMaxTrips)
    (hMK : M * K ≤ 64) (hKN : K * N ≤ 64) :
    ∀ (n i0 p : Nat) (mm : Machine),
      p = i0 * N → i0 + n ≤ M → p + n * N ≤ 64 →
      mm.regs.length = 14 → mm.mem.length = 1024 →
      (∀ x, x < 768 → mm.mem.getD x 0 = mem0.getD x 0) →
      mm.reg 3 = M → mm.reg 4 = N → mm.reg 5 = K → mm.reg 13 = 4 →
      (Plan.runLoop gemmKernelIBody.eval 6 n i0 mm).mem.length = 1024 ∧
      (Plan.runLoop gemmKernelIBody.eval 6 n i0 mm).regs.length = 14 ∧
      (Plan.runLoop gemmKernelIBody.eval 6 n i0 mm).reg 3 = M ∧
      (Plan.runLoop gemmKernelIBody.eval 6 n i0 mm).reg 4 = N ∧
      (Plan.runLoop gemmKernelIBody.eval 6 n i0 mm).reg 5 = K ∧
      (Plan.runLoop gemmKernelIBody.eval 6 n i0 mm).reg 13 = 4 ∧
      (∀ x, x < 768 + 4 * p ∨ 768 + 4 * (p + n * N) ≤ x →
        (Plan.runLoop gemmKernelIBody.eval 6 n i0 mm).mem.getD x 0 = mm.mem.getD x 0) ∧
      (∀ i j, i0 ≤ i → i < i0 + n → j < N →
        leWord (Plan.runLoop gemmKernelIBody.eval 6 n i0 mm).mem (768 + 4 * (i * N + j)) 4 =
          gemmDot mem0 N K i j % 4294967296) := by
  intro n
  induction n with
  | zero =>
    intro i0 p mm _ _ _ hlen hml _ h3 h4 h5 h13
    exact ⟨hml, hlen, h3, h4, h5, h13, fun _ _ => rfl,
      fun _ _ _ h _ => absurd h (by omega)⟩
  | succ n ih =>
    intro i0 p mm hp hiM hbound hlen hml hag h3 h4 h5 h13
    rw [Plan.runLoop_succ]
    have hsucc : (n + 1) * N = n * N + N := Nat.succ_mul n N
    rw [hsucc] at hbound
    have hlen' : (mm.withReg 6 i0).regs.length = 14 := by simpa using hlen
    have e3 : (mm.withReg 6 i0).reg 3 = M := by
      rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h3
    have e4 : (mm.withReg 6 i0).reg 4 = N := by
      rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h4
    have e5 : (mm.withReg 6 i0).reg 5 = K := by
      rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h5
    have e6 : (mm.withReg 6 i0).reg 6 = i0 :=
      Machine.reg_withReg_same _ _ _ (by rw [hlen]; omega)
    have e13 : (mm.withReg 6 i0).reg 13 = 4 := by
      rw [Machine.reg_withReg_ne _ _ (by decide)]; exact h13
    have emem : (mm.withReg 6 i0).mem = mm.mem := rfl
    have eml : (mm.withReg 6 i0).mem.length = 1024 := by rw [emem]; exact hml
    have eag : ∀ x, x < 768 → (mm.withReg 6 i0).mem.getD x 0 = mem0.getD x 0 := by
      rw [emem]; exact hag
    have hbody : gemmKernelIBody.eval (mm.withReg 6 i0) =
        Plan.runLoop gemmKernelJBody.eval 7 N 0 (mm.withReg 6 i0) := by
      rw [gemmKernelIBody, eval_loopReg, e4, min_loopRegMaxTrips hNt]
    obtain ⟨bml, bl, b3, b4, b5, b6, b13, bfr, brd⟩ :=
      gemmKernelJLoop mem0 M N K i0 p hp hK hMK hKN (by omega) N 0 (mm.withReg 6 i0)
        hlen' eml eag e3 e4 e5 e6 e13 (by omega) (by omega)
    rw [← hbody] at bml bl b3 b4 b5 b6 b13 bfr brd
    rw [emem] at bfr
    have bag : ∀ x, x < 768 → (gemmKernelIBody.eval (mm.withReg 6 i0)).mem.getD x 0 =
        mem0.getD x 0 := by
      intro x hx
      rw [bfr x (Or.inl (by omega))]
      exact hag x hx
    obtain ⟨im, il, i3, i4, i5, i13, ifr, ird⟩ :=
      ih (i0 + 1) (p + N) (gemmKernelIBody.eval (mm.withReg 6 i0))
        (by rw [hp, Nat.succ_mul]) (by omega) (by omega) bl bml bag b3 b4 b5 b13
    refine ⟨im, il, i3, i4, i5, i13, ?_, ?_⟩
    · intro x hx
      rw [ifr x (by omega), bfr x (by omega)]
    · intro i j hi1 hi2 hj
      rcases Nat.eq_or_lt_of_le hi1 with hie | hil
      · subst hie
        have hpi : 768 + 4 * (i0 * N + j) = 768 + 4 * (p + j) := by rw [hp]
        rw [hpi, ← brd j (Nat.zero_le _) (by omega)]
        refine leWord_ext 4 _ _ _ _ ?_
        intro k hk
        exact ifr (768 + 4 * (p + j) + k) (Or.inl (by omega))
      · exact ird i j hil (by omega) hj

/-! ### The load phase -/

/-- **The load phase reads the header.**  Registers `3`, `4`, `5` end up holding
exactly the little-endian header words at cells `16`, `24`, `32`, and register
`13` holds the stored element width. -/
theorem gemmKernelLoad_eval (m : Machine) (hlen : m.regs.length = 14) :
    (gemmKernelLoad.eval m).mem = m.mem ∧
    (gemmKernelLoad.eval m).regs.length = 14 ∧
    (gemmKernelLoad.eval m).reg 3 = leWord m.mem 16 8 ∧
    (gemmKernelLoad.eval m).reg 4 = leWord m.mem 24 8 ∧
    (gemmKernelLoad.eval m).reg 5 = leWord m.mem 32 8 ∧
    (gemmKernelLoad.eval m).reg 13 = 4 := by
  simp only [gemmKernelLoad, Plan.eval_seq, eval_setReg, eval_loadReg, loadedReg,
    storeAddr_gemmZeroMap, Machine.mem_withReg]
  refine ⟨?_, ?_, ?_, ?_, ?_, ?_⟩ <;>
    simp [Machine.reg_withReg_ne, hlen, gemmKernelMField, gemmKernelNField, gemmKernelKField]

/-! ### The kernel as a whole -/

/-- On a valid, row-major descriptor the memory the whole plan leaves is the
memory its kernel body leaves: the status word and the output construction do
not touch it. -/
theorem gemmKernel_mem (m : Machine)
    (hcls : m.classify = Classification.valid)
    (hlay : m.layoutClass = LayoutClass.rowMajor) :
    (gemmKernel.eval m).mem = (gemmKernelCore.eval m).mem := by
  show ((Plan.buildOutput gemmKernelC).eval
    ((Plan.classifyRaw _ _ _ _).eval m)).mem = _
  rw [Plan.eval_classifyRaw]
  simp only [hcls, Plan.eval_dispatchLayout, hlay, Plan.eval_seq]
  rfl

/-- **The kernel body computes the product**, for every shape satisfying the
stated hypotheses. -/
theorem gemmKernelCore_writes_C (m : Machine) (M N K i j : Nat)
    (hregs : m.regs.length = 14) (hmem : m.mem.length = 1024)
    (hM : leWord m.mem 16 8 = M) (hN : leWord m.mem 24 8 = N) (hKf : leWord m.mem 32 8 = K)
    (hMt : M ≤ loopRegMaxTrips) (hNt : N ≤ loopRegMaxTrips) (hKt : K ≤ loopRegMaxTrips)
    (hMK : M * K ≤ 64) (hKN : K * N ≤ 64) (hMN : M * N ≤ 64)
    (hi : i < M) (hj : j < N) :
    leWord (gemmKernelCore.eval m).mem (768 + 4 * (i * N + j)) 4 =
      gemmDot m.mem N K i j % 4294967296 := by
  obtain ⟨lm, ll, l3, l4, l5, l13⟩ := gemmKernelLoad_eval m hregs
  rw [hM] at l3
  rw [hN] at l4
  rw [hKf] at l5
  have hnest : gemmKernelCore.eval m =
      Plan.runLoop gemmKernelIBody.eval 6 M 0 (gemmKernelLoad.eval m) := by
    show gemmKernelNest.eval (gemmKernelLoad.eval m) = _
    rw [gemmKernelNest, eval_loopReg, l3, min_loopRegMaxTrips hMt]
  obtain ⟨-, -, -, -, -, -, -, ird⟩ :=
    gemmKernelILoop m.mem M N K hKt hNt hMK hKN M 0 0 (gemmKernelLoad.eval m)
      (Nat.zero_mul N).symm (by omega) (by omega) ll (by rw [lm]; exact hmem)
      (by rw [lm]; exact fun _ _ => rfl) l3 l4 l5 l13
  rw [hnest]
  exact ird i j (Nat.zero_le _) (by omega) hj

/-- The kernel's interface: fourteen scalar registers, no vector lane at all,
no scratch and no table, and 1024 cells of observable memory. -/
def gemmKernelSig : Sig :=
  { inputType := .bytes 1024
    outputType := .unit
    regs := 14, vregs := 0, lanes := 0, scratch := 0, tables := 0, mem := 1024
    statusSet := false }

/-- The interface the kernel leaves: a status has been constructed and the
output is the 256-cell `C` region. -/
def gemmKernelOutSig : Sig :=
  { gemmKernelSig with outputType := .bytes 256, statusSet := true }

/-- **The input-dependent kernel is inside the released subset.** -/
theorem gemmKernel_inReleasedSubset : gemmKernel.inReleasedSubset = true := by decide

/-- The kernel uses no vector operation, so the released profile's lack of SIMD
refuses nothing in it. -/
theorem gemmKernel_usesVector : gemmKernel.usesVector = false := by decide

/-- **The kernel type checks** at the scalar interface. -/
theorem gemmKernel_typed : HasType gemmKernelSig gemmKernel gemmKernelOutSig := by decide

/-- The kernel as a `CheckedPlan`. -/
def gemmKernelChecked : CheckedPlan gemmKernelSig gemmKernelOutSig :=
  { plan := gemmKernel, typed := gemmKernel_typed }

/-- **The compiled kernel passes release validation.** -/
theorem gemmKernel_compiles : Wasm.validate (compile gemmKernelChecked) = true :=
  compile_validates gemmKernelChecked

/-- **The compilation of the kernel contains no `unreachable`**: neither
`loadReg` nor `loopReg` was refused by the translation. -/
theorem gemmKernel_no_unreachable :
    listHasUnreachable
      (bodyCode (envOf gemmKernelSig gemmKernel) gemmKernelSig.scratch gemmKernel) = false :=
  bodyCode_no_unreachable _ _ _ gemmKernel_inReleasedSubset

/-- **The kernel loads its extents.**  After the load phase the three
loop-bound registers hold exactly the header's `m`, `n` and `k` — the SPEC §8.3
fields at cells `16`, `24` and `32` — for *every* configuration.  No literal
extent appears anywhere. -/
theorem gemmKernel_loads_extents (m : Machine) (hregs : m.regs.length = 14) :
    (gemmKernelLoad.eval m).reg 3 = leWord m.mem gemmKernelMField.base 8 ∧
    (gemmKernelLoad.eval m).reg 4 = leWord m.mem gemmKernelNField.base 8 ∧
    (gemmKernelLoad.eval m).reg 5 = leWord m.mem gemmKernelKField.base 8 := by
  obtain ⟨-, -, l3, l4, l5, -⟩ := gemmKernelLoad_eval m hregs
  exact ⟨l3, l4, l5⟩

/-- **The same plan computes the GEMM of whatever shape the header declares.**
For a descriptor whose header declares `m`, `n`, `k`, and for every position
`i < m`, `j < n`, the `C` region holds the modular-`u32` product
`(A · B)[i,j]` after evaluation.

The loop bounds really are the loaded registers: `gemmKernelNest_trips` shows the
outer trip count is the header word itself, and the theorem is quantified over
`m`, `n`, `k`, so no instance of it can be obtained from a fixed-extent plan.
The exact reach — every hypothesis, and everything not claimed — is stated in
the section preamble above. -/
theorem gemmKernel_writes_C (m : Machine) (M N K i j : Nat)
    (hregs : m.regs.length = 14) (hmem : m.mem.length = 1024)
    (hcls : m.classify = Classification.valid)
    (hlay : m.layoutClass = LayoutClass.rowMajor)
    (hM : leWord m.mem 16 8 = M) (hN : leWord m.mem 24 8 = N) (hKf : leWord m.mem 32 8 = K)
    (hMt : M ≤ loopRegMaxTrips) (hNt : N ≤ loopRegMaxTrips) (hKt : K ≤ loopRegMaxTrips)
    (hMK : M * K ≤ 64) (hKN : K * N ≤ 64) (hMN : M * N ≤ 64)
    (hi : i < M) (hj : j < N) :
    leWord (gemmKernel.eval m).mem (gemmKernelC.base + 4 * (i * N + j)) 4 =
      gemmDot m.mem N K i j % 4294967296 := by
  rw [gemmKernel_mem m hcls hlay]
  exact gemmKernelCore_writes_C m M N K i j hregs hmem hM hN hKf hMt hNt hKt hMK hKN hMN hi hj

/-! ### Concrete descriptors of three different shapes

`gemmKernelHeader` is a SPEC §8.3 header for a row-major, modular-`u32`,
`batch = 1` GEMM of the given shape.  Its field assignment is the one
`Machine.classify` and `Machine.layoutClass` read: cells `0..15` carry the magic,
version, header size, the four kind tags and the mode tag; `16..47` the extents
and the batch; each of the three 40-cell view records carries offset, row
stride, **column stride**, batch stride and byte length, so that the cell pair
`64..65` that `Machine.layoutClass` compares against the stored width is the
column stride — which is the element width exactly when the view is packed
row-major.  As with `gemmWitness`, this is a *cell* image; no claim is made that
it equals `Gemm/ABI.lean`'s byte image. -/

/-- The cell image of a list of `u32` elements: four little-endian cells each. -/
def gemmCells : List Nat → List Nat
  | [] => []
  | v :: vs => leBytes v 4 ++ gemmCells vs

/-- Zero padding to a declared region size. -/
def gemmPad (l : List Nat) (n : Nat) : List Nat := l ++ List.replicate (n - l.length) 0

/-- A SPEC §8.3 header for a row-major, modular-`u32`, `batch = 1` GEMM of the
given shape, in the field assignment described above. -/
def gemmKernelHeader (M N K : Nat) : List Nat :=
  [87, 71, 78, 71, 1, 0, 0, 1] ++
  [5, 5, 5, 5] ++
  [0, 0, 0, 0] ++
  leBytes M 8 ++ leBytes N 8 ++ leBytes K 8 ++ leBytes 1 8 ++
  leBytes 256 8 ++ leBytes (4 * K) 8 ++ leBytes 4 8 ++ leBytes (4 * (M * K)) 8 ++
    leBytes (4 * (M * K)) 8 ++
  leBytes 512 8 ++ leBytes (4 * N) 8 ++ leBytes 4 8 ++ leBytes (4 * (K * N)) 8 ++
    leBytes (4 * (K * N)) 8 ++
  leBytes 768 8 ++ leBytes (4 * N) 8 ++ leBytes 4 8 ++ leBytes (4 * (M * N)) 8 ++
    leBytes (4 * (M * N)) 8 ++
  leBytes 1 8 ++ List.replicate 8 0 ++ List.replicate 16 0 ++
  List.replicate 16 0 ++
  List.replicate 16 0 ++
  List.replicate 24 0

/-- The header is exactly the 256 cells SPEC §8.3 declares. -/
theorem gemmKernelHeader_length (M N K : Nat) : (gemmKernelHeader M N K).length = 256 := by
  simp [gemmKernelHeader, leBytes_length]

/-- A complete 1024-cell observable memory: header, `A`, `B`, and a zeroed
`C`. -/
def gemmKernelMem (M N K : Nat) (A B : List Nat) : List Nat :=
  gemmKernelHeader M N K ++ gemmPad (gemmCells A) 256 ++ gemmPad (gemmCells B) 256 ++
    List.replicate 256 0

/-- A complete input configuration.  The entry status is deliberately not
`success`, so `gemmKernel_eval_status` has content. -/
def gemmKernelMachine (M N K : Nat) (A B : List Nat) : Machine :=
  { mem := gemmKernelMem M N K A B
    scratch := []
    regs := List.replicate 14 0
    vregs := []
    tables := []
    status := Status.arithmeticException.code
    out := [] }

/-- Each element occupies four cells. -/
theorem cellsOf_length : ∀ (A : List Nat), (gemmCells A).length = 4 * A.length := by
  intro A
  induction A with
  | nil => rfl
  | cons v vs ih => simp [gemmCells, leBytes_length, ih]; omega

/-- The observable memory of a concrete descriptor is exactly 1024 cells. -/
theorem gemmKernelMem_length (M N K : Nat) (A B : List Nat)
    (hA : A.length ≤ 64) (hB : B.length ≤ 64) :
    (gemmKernelMem M N K A B).length = 1024 := by
  simp [gemmKernelMem, gemmPad, gemmKernelHeader_length, cellsOf_length]
  omega

/-- **The same plan computes the GEMM of whatever shape the header declares.**
For a descriptor whose header declares `m`, `n`, `k`, and for every position
`i < m`, `j < n`, the `C` region holds the modular-`u32` product
`(A · B)[i,j]` after evaluation.

The loop bounds really are the loaded registers: `gemmKernelNest_trips` shows the
outer trip count is the header word itself, and the theorem is quantified over
`m`, `n`, `k`, so no instance of it can be obtained from a fixed-extent plan.
The exact reach — every hypothesis, and everything not claimed — is stated in
the section preamble above. -/
theorem gemmKernel_writes_C_1x1x1 :
    leWord (gemmKernel.eval (gemmKernelMachine 1 1 1 [7] [6])).mem 768 4 = 42 := by
  have h := gemmKernel_writes_C (gemmKernelMachine 1 1 1 [7] [6]) 1 1 1 0 0
    (by rfl) (gemmKernelMem_length 1 1 1 [7] [6] (by decide) (by decide))
    (by rfl) (by rfl) (by rfl) (by rfl) (by rfl)
    (by decide) (by decide) (by decide) (by decide) (by decide) (by decide)
    (by decide) (by decide)
  exact h.trans (by rfl)

/-- **The same plan computes the GEMM of whatever shape the header declares.**
For a descriptor whose header declares `m`, `n`, `k`, and for every position
`i < m`, `j < n`, the `C` region holds the modular-`u32` product
`(A · B)[i,j]` after evaluation.

The loop bounds really are the loaded registers: `gemmKernelNest_trips` shows the
outer trip count is the header word itself, and the theorem is quantified over
`m`, `n`, `k`, so no instance of it can be obtained from a fixed-extent plan.
The exact reach — every hypothesis, and everything not claimed — is stated in
the section preamble above. -/
theorem gemmKernel_writes_C_2x2x2 :
    leWord (gemmKernel.eval (gemmKernelMachine 2 2 2 [1, 2, 3, 4] [5, 6, 7, 8])).mem
        768 4 = 19 ∧
    leWord (gemmKernel.eval (gemmKernelMachine 2 2 2 [1, 2, 3, 4] [5, 6, 7, 8])).mem
        772 4 = 22 ∧
    leWord (gemmKernel.eval (gemmKernelMachine 2 2 2 [1, 2, 3, 4] [5, 6, 7, 8])).mem
        776 4 = 43 ∧
    leWord (gemmKernel.eval (gemmKernelMachine 2 2 2 [1, 2, 3, 4] [5, 6, 7, 8])).mem
        780 4 = 50 := by
  refine ⟨?_, ?_, ?_, ?_⟩
  · exact (gemmKernel_writes_C (gemmKernelMachine 2 2 2 [1, 2, 3, 4] [5, 6, 7, 8]) 2 2 2 0 0
      (by rfl) (gemmKernelMem_length 2 2 2 _ _ (by decide) (by decide))
      (by rfl) (by rfl) (by rfl) (by rfl) (by rfl)
      (by decide) (by decide) (by decide) (by decide) (by decide) (by decide)
      (by decide) (by decide)).trans (by rfl)
  · exact (gemmKernel_writes_C (gemmKernelMachine 2 2 2 [1, 2, 3, 4] [5, 6, 7, 8]) 2 2 2 0 1
      (by rfl) (gemmKernelMem_length 2 2 2 _ _ (by decide) (by decide))
      (by rfl) (by rfl) (by rfl) (by rfl) (by rfl)
      (by decide) (by decide) (by decide) (by decide) (by decide) (by decide)
      (by decide) (by decide)).trans (by rfl)
  · exact (gemmKernel_writes_C (gemmKernelMachine 2 2 2 [1, 2, 3, 4] [5, 6, 7, 8]) 2 2 2 1 0
      (by rfl) (gemmKernelMem_length 2 2 2 _ _ (by decide) (by decide))
      (by rfl) (by rfl) (by rfl) (by rfl) (by rfl)
      (by decide) (by decide) (by decide) (by decide) (by decide) (by decide)
      (by decide) (by decide)).trans (by rfl)
  · exact (gemmKernel_writes_C (gemmKernelMachine 2 2 2 [1, 2, 3, 4] [5, 6, 7, 8]) 2 2 2 1 1
      (by rfl) (gemmKernelMem_length 2 2 2 _ _ (by decide) (by decide))
      (by rfl) (by rfl) (by rfl) (by rfl) (by rfl)
      (by decide) (by decide) (by decide) (by decide) (by decide) (by decide)
      (by decide) (by decide)).trans (by rfl)

/-- **The same plan computes the GEMM of whatever shape the header declares.**
For a descriptor whose header declares `m`, `n`, `k`, and for every position
`i < m`, `j < n`, the `C` region holds the modular-`u32` product
`(A · B)[i,j]` after evaluation.

The loop bounds really are the loaded registers: `gemmKernelNest_trips` shows the
outer trip count is the header word itself, and the theorem is quantified over
`m`, `n`, `k`, so no instance of it can be obtained from a fixed-extent plan.
The exact reach — every hypothesis, and everything not claimed — is stated in
the section preamble above. -/
theorem gemmKernel_writes_C_1x3x2 :
    leWord (gemmKernel.eval (gemmKernelMachine 1 3 2 [2, 3] [1, 2, 3, 4, 5, 6])).mem
        768 4 = 14 ∧
    leWord (gemmKernel.eval (gemmKernelMachine 1 3 2 [2, 3] [1, 2, 3, 4, 5, 6])).mem
        772 4 = 19 ∧
    leWord (gemmKernel.eval (gemmKernelMachine 1 3 2 [2, 3] [1, 2, 3, 4, 5, 6])).mem
        776 4 = 24 := by
  refine ⟨?_, ?_, ?_⟩
  · exact (gemmKernel_writes_C (gemmKernelMachine 1 3 2 [2, 3] [1, 2, 3, 4, 5, 6]) 1 3 2 0 0
      (by rfl) (gemmKernelMem_length 1 3 2 _ _ (by decide) (by decide))
      (by rfl) (by rfl) (by rfl) (by rfl) (by rfl)
      (by decide) (by decide) (by decide) (by decide) (by decide) (by decide)
      (by decide) (by decide)).trans (by rfl)
  · exact (gemmKernel_writes_C (gemmKernelMachine 1 3 2 [2, 3] [1, 2, 3, 4, 5, 6]) 1 3 2 0 1
      (by rfl) (gemmKernelMem_length 1 3 2 _ _ (by decide) (by decide))
      (by rfl) (by rfl) (by rfl) (by rfl) (by rfl)
      (by decide) (by decide) (by decide) (by decide) (by decide) (by decide)
      (by decide) (by decide)).trans (by rfl)
  · exact (gemmKernel_writes_C (gemmKernelMachine 1 3 2 [2, 3] [1, 2, 3, 4, 5, 6]) 1 3 2 0 2
      (by rfl) (gemmKernelMem_length 1 3 2 _ _ (by decide) (by decide))
      (by rfl) (by rfl) (by rfl) (by rfl) (by rfl)
      (by decide) (by decide) (by decide) (by decide) (by decide) (by decide)
      (by decide) (by decide)).trans (by rfl)

/-! ### The reduction really is the declared contract's -/

/-- The released modular arithmetic contract with a `u32` accumulator: the same
contract `gemmWitness` declares. -/
def gemmKernelContract : ArithmeticContract :=
  { mode := .modular, stored := .u32, accumulator := .u32 }

theorem gemmKernelContract_compatible : gemmKernelContract.compatibleB = true := by decide

theorem gemmKernelContract_released : gemmKernelContract.releasedB = true := by decide

theorem gemmKernelContract_accModulus : gemmKernelContract.accModulus = 4294967296 :=
  ArithmeticContract.accModulus_of_releasedB gemmKernelContract_released

/-- The reduction the arithmetic contract itself performs over a pair of element
sequences: `ArithmeticContract.step`, iterated. -/
def gemmContractFold (c : ArithmeticContract) (f g : Nat → Nat) : Nat → Nat → Nat → Nat
  | 0, _, acc => acc
  | n + 1, s, acc => gemmContractFold c f g n (s + 1) (c.step acc (f s) (g s))

/-- A modular contract's fold is the exact sum, reduced. -/
theorem contractFold_modular (c : ArithmeticContract) (hc : c.mode = .modular)
    (f g : Nat → Nat) : ∀ (n s acc : Nat), acc < c.accModulus →
      gemmContractFold c f g n s acc =
        (acc + gemmSumFrom (fun t => f t * g t) n s) % c.accModulus := by
  intro n
  induction n with
  | zero =>
    intro s acc hacc
    show acc = (acc + 0) % c.accModulus
    rw [Nat.add_zero, Nat.mod_eq_of_lt hacc]
  | succ n ih =>
    intro s acc hacc
    show gemmContractFold c f g n (s + 1) (c.step acc (f s) (g s)) = _
    rw [ih (s + 1) _ (c.step_modular_lt hc _ _ _)]
    have hstep : c.step acc (f s) (g s) = (acc + f s * g s) % c.accModulus := by
      simp only [ArithmeticContract.step, hc]
    have hsum : gemmSumFrom (fun t => f t * g t) (n + 1) s
        = f s * g s + gemmSumFrom (fun t => f t * g t) n (s + 1) := rfl
    rw [hstep, Nat.mod_add_mod, hsum, Nat.add_assoc]

/-- **The kernel's accumulation is the contract's.**  The `u32` word the kernel
deposits in `C` is exactly the modular-`u32` reduction that
`ArithmeticContract.step` performs over the `A` row and the `B` column, so the
reduction is carried out under the declared contract of SPEC §8.2 and not under
some other formula that happens to agree with it. -/
theorem gemmKernel_C_is_contract_fold (m : Machine) (M N K i j : Nat)
    (hregs : m.regs.length = 14) (hmem : m.mem.length = 1024)
    (hcls : m.classify = Classification.valid)
    (hlay : m.layoutClass = LayoutClass.rowMajor)
    (hM : leWord m.mem 16 8 = M) (hN : leWord m.mem 24 8 = N) (hKf : leWord m.mem 32 8 = K)
    (hMt : M ≤ loopRegMaxTrips) (hNt : N ≤ loopRegMaxTrips) (hKt : K ≤ loopRegMaxTrips)
    (hMK : M * K ≤ 64) (hKN : K * N ≤ 64) (hMN : M * N ≤ 64)
    (hi : i < M) (hj : j < N) :
    leWord (gemmKernel.eval m).mem (gemmKernelC.base + 4 * (i * N + j)) 4 =
      gemmContractFold gemmKernelContract (fun t => gemmAWord m.mem K i t)
        (fun t => gemmBWord m.mem N t j) K 0 0 := by
  rw [gemmKernel_writes_C m M N K i j hregs hmem hcls hlay hM hN hKf hMt hNt hKt
    hMK hKN hMN hi hj,
    contractFold_modular gemmKernelContract rfl _ _ K 0 0
      (by rw [gemmKernelContract_accModulus]; omega),
    gemmKernelContract_accModulus, Nat.zero_add]
  rfl

/-- **The trip count is the header word.**  The outer loop of the nest runs for
the number of iterations the header's `m` field supplies, clamped only by the
released `i32` ceiling.  This is the statement no fixed-extent plan can make. -/
theorem gemmKernelNest_trips (m : Machine) (hregs : m.regs.length = 14) :
    gemmKernelNest.eval (gemmKernelLoad.eval m) =
      Plan.runLoop gemmKernelIBody.eval 6
        (Nat.min (leWord m.mem gemmKernelMField.base 8) loopRegMaxTrips) 0
        (gemmKernelLoad.eval m) := by
  obtain ⟨l3, -, -⟩ := gemmKernel_loads_extents m hregs
  rw [gemmKernelNest, eval_loopReg, l3]

/-- **Frame.**  The kernel writes nothing but the `m·n` elements of `C`: the
header, `A` and `B` are left exactly as the invocation supplied them. -/
theorem gemmKernelCore_frame (m : Machine) (M N K : Nat)
    (hregs : m.regs.length = 14) (hmem : m.mem.length = 1024)
    (hM : leWord m.mem 16 8 = M) (hN : leWord m.mem 24 8 = N) (hKf : leWord m.mem 32 8 = K)
    (hMt : M ≤ loopRegMaxTrips) (hNt : N ≤ loopRegMaxTrips) (hKt : K ≤ loopRegMaxTrips)
    (hMK : M * K ≤ 64) (hKN : K * N ≤ 64) (hMN : M * N ≤ 64) :
    ∀ x, x < 768 ∨ 768 + 4 * (M * N) ≤ x →
      (gemmKernelCore.eval m).mem.getD x 0 = m.mem.getD x 0 := by
  obtain ⟨lm, ll, l3, l4, l5, l13⟩ := gemmKernelLoad_eval m hregs
  rw [hM] at l3
  rw [hN] at l4
  rw [hKf] at l5
  have hnest : gemmKernelCore.eval m =
      Plan.runLoop gemmKernelIBody.eval 6 M 0 (gemmKernelLoad.eval m) := by
    show gemmKernelNest.eval (gemmKernelLoad.eval m) = _
    rw [gemmKernelNest, eval_loopReg, l3, min_loopRegMaxTrips hMt]
  obtain ⟨-, -, -, -, -, -, ifr, -⟩ :=
    gemmKernelILoop m.mem M N K hKt hNt hMK hKN M 0 0 (gemmKernelLoad.eval m)
      (Nat.zero_mul N).symm (by omega) (by omega) ll (by rw [lm]; exact hmem)
      (by rw [lm]; exact fun _ _ => rfl) l3 l4 l5 l13
  intro x hx
  rw [hnest, ifr x (by omega), lm]

/-- The kernel sets the released success status word on a valid row-major
descriptor. -/
theorem gemmKernel_eval_status (m : Machine)
    (hcls : m.classify = Classification.valid)
    (hlay : m.layoutClass = LayoutClass.rowMajor) :
    (gemmKernel.eval m).status = Status.success.code := by
  show ((Plan.buildOutput gemmKernelC).eval ((Plan.classifyRaw _ _ _ _).eval m)).status = _
  rw [Plan.eval_classifyRaw]
  simp only [hcls, Plan.eval_dispatchLayout, hlay, Plan.eval_seq]
  rfl

/-- The kernel publishes the declared `C` region. -/
theorem gemmKernel_eval_out (m : Machine)
    (hcls : m.classify = Classification.valid)
    (hlay : m.layoutClass = LayoutClass.rowMajor) :
    (gemmKernel.eval m).out =
      ((gemmKernelCore.eval m).mem.drop gemmKernelC.base).take gemmKernelC.count := by
  show ((Plan.buildOutput gemmKernelC).eval ((Plan.classifyRaw _ _ _ _).eval m)).out = _
  rw [Plan.eval_classifyRaw]
  simp only [hcls, Plan.eval_dispatchLayout, hlay, Plan.eval_seq]
  rfl

/-- Every concrete descriptor conforms to the kernel's interface. -/
theorem gemmKernelMachine_conforms (M N K : Nat) (A B : List Nat)
    (hA : A.length ≤ 64) (hB : B.length ≤ 64) :
    (gemmKernelMachine M N K A B).Conforms gemmKernelSig :=
  ⟨by simp [gemmKernelMachine, gemmKernelSig], rfl, gemmKernelMem_length M N K A B hA hB, rfl,
    Nat.zero_le _⟩

/-- Type safety instantiated at the kernel. -/
theorem gemmKernel_eval_conforms (m : Machine) (hm : m.Conforms gemmKernelSig) :
    (gemmKernel.eval m).Conforms gemmKernelOutSig :=
  hasType_preservation gemmKernel_typed m hm

/-! ### The kernel's static step bound (BI-006, step 1)

`Plan.loopReg` reads its trip count out of a register, so its *executed* step
count is a function of the input; `GNAF/Plan.lean` clamps the executed trip
count at `loopRegMaxTrips = 2 ^ 32 - 1` and charges `loopRegMaxTrips`
iterations of the body to the loop, and `Plan.steps_le_stepBound` proves that
the executed count of *any* plan on *any* configuration is under the resulting
static number.  For `gemmKernel` — three nested `loopReg` loops — that number is
therefore finite even though nothing in the plan text names an extent, and this
section computes it and compares it against the released step budget of SPEC
§8.3, `2 ^ 320`.

**The exact reach of what follows.**  `Plan.steps` counts steps of the *GNAF*
machine of `GNAF/Semantics.lean` — the interpreter `Plan.eval` — and of nothing
else.  These theorems say nothing whatever about the number of *Wasm* reduction
steps the compiled module `compile gemmKernelChecked` takes: relating the two is
the refinement obligation this file's header lists as omitted, and it is still
omitted.  `compile_resources` does not supply it either; it bounds the emitted
module's static instruction *count*, and explicitly not its execution.  See
`Release.GemmKernelReducesBounded` in `Artifact/Release.lean` for the exact
statement that is missing. -/

/-- The kernel's static step bound, as a literal.  Three nested `loopReg` loops
at the released `i32` ceiling: about `2 ^ 99`. -/
theorem gemmKernel_stepBound_eq :
    gemmKernelChecked.plan.stepBound = 792281624699921518248014643209 := rfl

/-- **BI-006, step 1.**  The kernel's static step bound is inside the released
costed step budget of SPEC §8.3, with about `2 ^ 220` to spare.  This is a
statement about `Plan.stepBound`, a number computed from the plan text; see the
section header for what it does *not* say. -/
theorem gemmKernel_stepBound_le_maxSteps :
    gemmKernelChecked.plan.stepBound ≤ 2 ^ 320 := by decide

/-- The strict form, which is what a fuel argument at `2 ^ 320` needs. -/
theorem gemmKernel_stepBound_lt_maxSteps :
    gemmKernelChecked.plan.stepBound < 2 ^ 320 := by decide

/-- **Every GNAF evaluation of the kernel is inside the released step budget.**
On *every* configuration — every descriptor, every extent the header can
declare — the kernel's executed `Plan.steps` count is at most `2 ^ 320`.  This
is `Plan.steps_le_stepBound` instantiated at the kernel; it is about
`Plan.eval`, not about the emitted Wasm. -/
theorem gemmKernel_steps_le_maxSteps (m : Machine) :
    gemmKernel.steps m ≤ 2 ^ 320 :=
  Nat.le_trans (Plan.steps_le_stepBound gemmKernel m) gemmKernel_stepBound_le_maxSteps

/-- The kernel's whole certified cost — not just its step coordinate — is inside
the released step budget as well. -/
theorem gemmKernel_certifiedCost_le_maxSteps :
    gemmKernelChecked.plan.certifiedCost ≤ 2 ^ 320 := by decide

end WasmGemmGnaf.GNAF
