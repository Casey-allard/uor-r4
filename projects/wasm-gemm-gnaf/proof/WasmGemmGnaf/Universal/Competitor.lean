/-
  The universal competitor universe: extensional definitions.
  Normative source: SPEC.md section 10.1 (with section 8.3 for the raw carrier).

  SCOPE — read this before citing anything here.

  This file transcribes SPEC section 10.1 with its quantifiers unchanged.  Every
  competitor predicate below quantifies over *all* raw invocations and *all*
  maximal executions; nothing here is artifact-specific, selector-specific, or
  conclusion-dependent.

  THE SEAM.  Three files of the mandated tree do not exist yet:
  `Wasm/Costed.lean` (the costed WebAssembly semantics), `Gemm/Reference.lean`
  (the reference acceptance relation) and `Gemm/Problem.lean` (the problem
  record).  Rather than invent them — which would silently substitute a wrong or
  artifact-shaped reference relation into the scope-critical definitions — the
  pieces they owe are carried as *parameters*:

    * `Universal.Semantics P`     — the event cost map and the two static cost
                                    counters of SPEC section 7.5;
    * `Universal.CostedMachine`   — costed initialization and the bounded
                                    all-branch costed explorer;
    * `Universal.Problem P`       — the problem's resource contract, workload
                                    repetitions, and reference relation;
    * `Universal.Decider`         — `Universal.evaluate` of SPEC section 10.1.

  This is the technique already used by `Cost/Proper.lean` (opaque `Profile` and
  `Problem` indices) and `Cost/Aggregate.lean` (the `decodes`, `decodeSteps`,
  `validationSteps`, `staticDataBytes` parameters).  No field of any of these
  records is a *conclusion*: none asserts correctness, feasibility, coverage,
  optimality, or finiteness of anything.  They are the data and the executable
  functions that the missing layers must supply.  When those layers land, each
  record is inhabited once with the frozen definitions and nothing in this file
  changes.

  Every declaration in this file is either a definition or a proved theorem.
  Nothing is assumed.
-/
import WasmGemmGnaf.Foundation.Result
import WasmGemmGnaf.Foundation.Order
import WasmGemmGnaf.Foundation.Finite
import WasmGemmGnaf.Wasm.Binary
import WasmGemmGnaf.Wasm.Validate
import WasmGemmGnaf.Wasm.Fuel
import WasmGemmGnaf.Wasm.Profile
import WasmGemmGnaf.Gemm.Classify
import WasmGemmGnaf.Cost.Aggregate
import WasmGemmGnaf.Cost.Trace

set_option autoImplicit false

namespace WasmGemmGnaf.Universal

/-! ## Profile validity (SPEC §10.1)

`ProfileValid` is defined here from the *existing* release validator and the
profile's own first-order limit table.  It is not a callback. -/

/-- Every index-space population of the module is inside the profile's declared
limit table (SPEC §7.2). -/
def WithinProfileLimits (P : Wasm.Profile) (m : Wasm.Module) : Bool :=
  decide (m.types.length ≤ P.body.limits.maxTypes) &&
  decide (m.funcs.length ≤ P.body.limits.maxFunctions) &&
  decide (m.tables.length ≤ P.body.limits.maxTables) &&
  decide (m.mems.length ≤ P.body.limits.maxMemories) &&
  decide (m.globals.length ≤ P.body.limits.maxGlobals) &&
  decide (m.tags.length ≤ P.body.limits.maxTags) &&
  decide (m.elems.length ≤ P.body.limits.maxElementSegments) &&
  decide (m.datas.length ≤ P.body.limits.maxDataSegments) &&
  decide (m.imports.length ≤ P.body.limits.maxImports) &&
  decide (m.exports.length ≤ P.body.limits.maxExports)

/-- SPEC §10.1's `Wasm.validateUnder`: the release validator together with the
profile's limit table. -/
def validateUnder (P : Wasm.Profile) (m : Wasm.Module) : Bool :=
  Wasm.validate m && WithinProfileLimits P m

/-- SPEC §10.1's `HasExactGemmExports`: the required memory and `gemm` exports
are present with the pinned ABI type, and no other export exists. -/
def HasExactGemmExports (_P : Wasm.Profile) (m : Wasm.Module) : Prop :=
  Wasm.Module.exportsMemory m = true ∧
  Wasm.Module.checkGemmExport m = true ∧
  ∀ e ∈ m.exports, e.name = Wasm.gemmExportName ∨ e.name = Wasm.memoryExportName

/-- **SPEC §10.1**, `Universal.ProfileValid`.  Quantifier-for-quantifier. -/
def ProfileValid (P : Wasm.Profile) (bytes : ByteArray) : Prop :=
  ∃ module : Wasm.Module,
    Wasm.decode bytes = .ok module ∧
    validateUnder P module = true ∧
    module.imports = [] ∧
    HasExactGemmExports P module

/-! ## The costed-semantics seam (`Wasm/Costed.lean`) -/

/-- The costed WebAssembly semantics owed by `Wasm/Costed.lean`: the charge of
one reduction event and the two static cost counters of SPEC §7.5.  Pure data
and pure functions; no proposition. -/
structure Semantics (_P : Wasm.Profile) where
  /-- SPEC §7.5: the charge of one relational reduction event. -/
  costEvent : Wasm.Event → Cost.Event
  /-- SPEC §7.5: `Wasm.validationCost` applied to the module's derivation. -/
  validationSteps : Wasm.Module → Nat
  /-- SPEC §7.5: `Wasm.instantiatedStaticBytes`. -/
  staticDataBytes : Wasm.Module → Nat

/-- **SPEC §10.1**, `Wasm.InitializationObservation`: the configuration a costed
GEMM invocation starts from, and the charge of reaching it. -/
structure InitializationObservation (_P : Wasm.Profile) where
  /-- The configuration handed to the reduction relation. -/
  initial : Wasm.Config
  /-- The charge of decoding, instantiating, installing and entering. -/
  cost : Cost.DynamicVector

/-- **SPEC §10.1**, `Wasm.RelationalPrefix`: a reduction of exactly `n` labelled
steps out of `initial`. -/
structure RelationalPrefix (_initial : Wasm.Config) (n : Nat) where
  /-- The labels of the prefix. -/
  events : List Wasm.Event
  /-- The configuration the prefix reaches. -/
  final : Wasm.Config
  /-- The prefix has exactly `n` steps. -/
  lengthEq : events.length = n

/-- A relational prefix is valid when it really is a reduction of `initial`. -/
def RelationalPrefix.Valid {initial : Wasm.Config} {n : Nat}
    (p : RelationalPrefix initial n) : Prop :=
  Wasm.Reduces initial p.events p.final

/-- SPEC §10.1's `NonterminalPrefix` payload: a *valid* prefix that overruns the
step budget. -/
structure NonterminalPrefix (initial : Wasm.Config) (n : Nat) where
  /-- The overrunning prefix. -/
  witness : RelationalPrefix initial n
  /-- Its reduction proof. -/
  valid : witness.Valid

/-- **SPEC §10.1**, `Wasm.CostedExecutionObservation`: a finite execution of
`initial` together with the exact charge of its trace. -/
structure CostedExecutionObservation {P : Wasm.Profile} (W : Semantics P)
    (initial : Wasm.Config) where
  /-- The observation reached. -/
  observation : Wasm.ExecutionObservation
  /-- Its reduction proof: the observation is a real execution of `initial`. -/
  execution : Wasm.FiniteExecution initial observation
  /-- The charge of the trace. -/
  cost : Cost.DynamicVector
  /-- The charge is exactly the trace cost; no freedom is left. -/
  costExact : cost = Cost.traceCost (observation.trace.map W.costEvent)

/-- The coverage obligation of a completed costed exploration: every listed
entry is a real execution, and every real execution inside the bound is
listed.  This mirrors `Wasm.CoversEveryMaximalFiniteBranch`. -/
def CostedCoverage {P : Wasm.Profile} (W : Semantics P) (bound : Nat)
    (initial : Wasm.Config)
    (observations :
      Foundation.NonemptyCanonicalFrontier (CostedExecutionObservation W initial)) :
    Prop :=
  (∀ o ∈ observations.elements, Wasm.FiniteExecution initial o.observation) ∧
  (∀ o : Wasm.ExecutionObservation, Wasm.FiniteExecution initial o →
    o.trace.length ≤ bound → ∃ c ∈ observations.elements, c.observation = o)

/-- **SPEC §10.1**, the result of `Wasm.exploreAllCosted`. -/
inductive CostedTreeResult {P : Wasm.Profile} (W : Semantics P) (bound : Nat)
    (initial : Wasm.Config)
  /-- Every branch terminated inside the bound. -/
  | complete
      (observations :
        Foundation.NonemptyCanonicalFrontier (CostedExecutionObservation W initial))
      (coverage : CostedCoverage W bound initial observations)
  /-- Some branch was still running after `bound + 1` steps. -/
  | nonterminalPrefix (overrun : NonterminalPrefix initial (bound + 1))
  /-- Instantiation failed before any reduction. -/
  | initializationFailure (fault : Wasm.InstantiationFault)

/-- The two executable costed entry points owed by `Wasm/Costed.lean`. -/
structure CostedMachine {P : Wasm.Profile} (W : Semantics P) where
  /-- SPEC §10.1's `Wasm.initialGemmInvocationCosted`. -/
  initialGemmInvocationCosted :
    Wasm.Module → Wasm.Invocation P →
      Except Foundation.FailureReport (InitializationObservation P)
  /-- SPEC §10.1's `Wasm.exploreAllCosted`. -/
  exploreAllCosted :
    (bound : Nat) → (initial : Wasm.Config) → CostedTreeResult W bound initial

/-! ## The problem seam (`Gemm/Problem.lean`, `Gemm/Reference.lean`) -/

/-- The problem-level data SPEC §8 owes: the resource contract, the workload
repetition count, and the reference acceptance relation.  `Accepts` is exactly
SPEC §10.1's `Gemm.Reference.Accepts G raw observation`: it is a field of the
*problem*, as SPEC §8.4 requires, never of the artifact. -/
structure Problem (P : Wasm.Profile) where
  /-- SPEC §8.2: the per-invocation costed step budget. -/
  maxSteps : Nat
  /-- SPEC §8.2: the per-invocation resource limit vector. -/
  limit : Cost.DynamicVector
  /-- SPEC §9.2: how many times the workload is charged. -/
  workloadRepetitions : Nat
  /-- SPEC §8.4: the reference acceptance relation. -/
  Accepts : Gemm.RawInvocation P → Wasm.ExecutionObservation → Prop

/-- **SPEC §8.3**, `Gemm.Problem.RawInvocation`.  The raw-input carrier is fixed
*before* classification and cannot be narrowed by the problem. -/
def Problem.RawInvocation {P : Wasm.Profile} (_problem : Problem P) : Type :=
  Gemm.RawInvocation P

/-- **SPEC §8.3**, `problem_raw_invocation_definitional`.  The problem's raw
carrier is the fixed carrier, definitionally: no problem instance can shrink
the input domain. -/
theorem problem_raw_invocation_definitional {P : Wasm.Profile}
    (problem : Problem P) : problem.RawInvocation = Gemm.RawInvocation P :=
  rfl

/-- **SPEC §8.3**, `Gemm.toWasmInvocation`.  The harness places exactly
`raw.body.bytes` at `raw.body.ptr` and synthesizes nothing else. -/
def toWasmInvocation {P : Wasm.Profile} (raw : Gemm.RawInvocation P) :
    Wasm.Invocation P :=
  Wasm.Invocation.gemmRaw raw.body.toWasm raw.lawful.toWasmInvocationLawful

/-- Everything the extensional definitions need, in one record: the costed
semantics, the costed machine, and the problem. -/
structure Setting (P : Wasm.Profile) where
  /-- The costed WebAssembly semantics. -/
  semantics : Semantics P
  /-- The costed initialization and exploration entry points. -/
  machine : CostedMachine semantics
  /-- The exact GEMM problem. -/
  problem : Problem P

/-! ## Costed evaluation (SPEC §10.1) -/

section Evaluation

variable {P : Wasm.Profile}

/-- **SPEC §10.1**, `Universal.InputEvaluation`: the exact result of the bounded
all-successor tree on one raw invocation. -/
structure InputEvaluation (S : Setting P) (module : Wasm.Module)
    (raw : Gemm.RawInvocation P) where
  /-- The configuration the invocation starts from. -/
  initial : Wasm.Config
  /-- The costed initialization observation. -/
  initialization : InitializationObservation P
  /-- It starts at `initial`. -/
  initialConfigEq : initialization.initial = initial
  /-- It is the machine's costed initialization of this module and invocation. -/
  initialEq :
    S.machine.initialGemmInvocationCosted module (toWasmInvocation raw) =
      .ok initialization
  /-- The canonically ordered, nonempty list of costed observations. -/
  observations :
    Foundation.NonemptyCanonicalFrontier
      (CostedExecutionObservation S.semantics initial)
  /-- The tree completed inside the problem's step budget with exactly those
  observations. -/
  treeComplete : ∃ coverage,
    S.machine.exploreAllCosted S.problem.maxSteps initial =
      .complete observations coverage
  /-- The invocation's resource vector. -/
  resourceVector : Cost.DynamicVector
  /-- It is exactly initialization composed with the worst branch. -/
  resourceExact : resourceVector =
    Cost.sequentialCompose initialization.cost
      (Cost.maxOverCosts (observations.elements.map (·.cost)))

/-- **SPEC §10.1**, `CoversEveryMaximalExecution`: no maximal execution of the
initial configuration escapes the recorded costed observations.  A divergent
execution has no finite observation, so it falsifies this predicate. -/
def InputEvaluation.CoversEveryMaximalExecution {S : Setting P}
    {module : Wasm.Module} {raw : Gemm.RawInvocation P}
    (ie : InputEvaluation S module raw) : Prop :=
  ∀ execution : Wasm.MaximalExecution ie.initial,
    ∃ costed ∈ ie.observations.elements,
      (match execution with
        | .finite o _ _ => o = costed.observation
        | .diverges _ _ _ _ => False)

variable [Foundation.Fintype (Gemm.RawInvocation P)]

/--
  **SPEC §10.1**, `Universal.SystemEvaluation`.

  The `Foundation.Fintype` instance on the raw carrier is SPEC §8.3's
  representability bound, supplied uniformly by the problem layer; it is the
  same instance `Cost.ExactAggregateCost` already requires.  It is a property of
  the *input domain*, not of any competitor, and it does not narrow the
  competitor quantifier.
-/
structure SystemEvaluation (S : Setting P) (bytes : ByteArray) where
  /-- The decoded module. -/
  module : Wasm.Module
  /-- It is exactly what `bytes` decodes to. -/
  decodeEq : Wasm.decode bytes = .ok module
  /-- One input evaluation for *every* raw invocation. -/
  perInput : ∀ raw : Gemm.RawInvocation P, InputEvaluation S module raw
  /-- Each of them covers every maximal execution. -/
  observationsComplete : ∀ raw, (perInput raw).CoversEveryMaximalExecution
  /-- The complete charged system cost. -/
  cost : Cost.CompleteSystemCost
  /-- It is the exact aggregate of SPEC §9.1. -/
  costExact :
    Cost.ExactAggregateCost bytes (Wasm.decode bytes = .ok module)
      (P.body.costTableBody.decodeCost bytes)
      (S.semantics.validationSteps module)
      (S.semantics.staticDataBytes module)
      S.problem.workloadRepetitions
      (fun raw => (perInput raw).resourceVector) cost

/-- **SPEC §10.1**, `Universal.EvaluationResult`. -/
inductive EvaluationResult (S : Setting P) (bytes : ByteArray)
  /-- The tree completed on every input. -/
  | complete (evaluation : SystemEvaluation S bytes)
  /-- The bytes are not profile valid. -/
  | profileFailure (report : Foundation.FailureReport)
  /-- Costed initialization failed on some input. -/
  | initializationFailure (raw : Gemm.RawInvocation P)
      (report : Foundation.FailureReport)
  /-- Some input overran the step budget. -/
  | nonterminal (raw : Gemm.RawInvocation P) (initial : Wasm.Config)
      (overrun : NonterminalPrefix initial (S.problem.maxSteps + 1))
  /-- Some input exhausted a declared resource. -/
  | resourceExhausted (raw : Gemm.RawInvocation P)
      (report : Foundation.ResourceReport)

/-- SPEC §10.1's `Universal.evaluate`: the implemented finite decoder,
validator, input enumerator and all-branch explorer.  It is a total function of
the bytes alone — it receives no artifact, no selector and no conclusion. -/
structure Decider (S : Setting P) where
  /-- The evaluator. -/
  evaluate : (bytes : ByteArray) → EvaluationResult S bytes

/-- **SPEC §10.1**, `Universal.SystemEvaluationRel`. -/
def SystemEvaluationRel (S : Setting P) (D : Decider S) (bytes : ByteArray)
    (evaluation : SystemEvaluation S bytes) : Prop :=
  D.evaluate bytes = .complete evaluation

end Evaluation

/-! ## The extensional semantic predicates (SPEC §10.1)

These are the predicates the release theorem quantifies over.  None of them
mentions an evaluation, a certificate, an artifact or a selector. -/

section Semantic

variable {P : Wasm.Profile}

/-- **SPEC §10.1**, `Universal.StartsCostedInvocation`. -/
def StartsCostedInvocation (S : Setting P) (bytes : ByteArray)
    (raw : Gemm.RawInvocation P)
    (initialization : InitializationObservation P) (initial : Wasm.Config) :
    Prop :=
  ∃ module : Wasm.Module,
    Wasm.decode bytes = .ok module ∧
    validateUnder P module = true ∧
    S.machine.initialGemmInvocationCosted module (toWasmInvocation raw) =
      .ok initialization ∧
    initialization.initial = initial

/--
  **SPEC §10.1**, `Wasm.IsMaximalExecutionFrom`.

  `Wasm.MaximalExecution initial` already carries the reduction and maximality
  proofs out of `initial` in both of its constructors, so the separate side
  condition is definitionally satisfied.  It is kept as a named predicate so the
  transcription of §10.1 is literal, and — decisively — so that the universal
  quantifier over executions below is *unrestricted*: it ranges over every
  inhabitant of `Wasm.MaximalExecution initial`, which is the strongest reading.
-/
def IsMaximalExecutionFrom (initial : Wasm.Config)
    (_execution : Wasm.MaximalExecution initial) : Prop :=
  True

/-- The side condition never removes an execution from the quantifier. -/
theorem isMaximalExecutionFrom_holds (initial : Wasm.Config)
    (execution : Wasm.MaximalExecution initial) :
    IsMaximalExecutionFrom initial execution :=
  trivial

/-- **SPEC §10.1**, `MaximalExecution.HasFiniteObservation`: true for a
returned, trapped or uncaught-exception finite maximal execution, and false for
`diverges`. -/
def HasFiniteObservation {initial : Wasm.Config}
    (execution : Wasm.MaximalExecution initial)
    (observation : Wasm.ExecutionObservation) : Prop :=
  match execution with
  | .finite o _ _ => o = observation
  | .diverges _ _ _ _ => False

/-- A divergent maximal execution has no finite observation. -/
theorem not_hasFiniteObservation_diverges {initial : Wasm.Config}
    (events : Nat → Wasm.Event) (configs : Nat → Wasm.Config)
    (starts : configs 0 = initial)
    (step : ∀ i, Wasm.Step (configs i) (events i) (configs (i + 1)))
    (observation : Wasm.ExecutionObservation) :
    ¬ HasFiniteObservation (Wasm.MaximalExecution.diverges events configs starts step)
      observation :=
  fun h => h

/-- A finite maximal execution has exactly its own observation. -/
theorem hasFiniteObservation_finite {initial : Wasm.Config}
    {o : Wasm.ExecutionObservation} (run : Wasm.FiniteExecution initial o)
    (maximal : Wasm.IsTerminalObservation o)
    (observation : Wasm.ExecutionObservation) :
    HasFiniteObservation (Wasm.MaximalExecution.finite o run maximal) observation ↔
      o = observation :=
  Iff.rfl

/-- **SPEC §10.1**, `MaximalExecution.CostedAs`. -/
def CostedAs {S : Setting P} {initial : Wasm.Config}
    (execution : Wasm.MaximalExecution initial)
    (costed : CostedExecutionObservation S.semantics initial) : Prop :=
  HasFiniteObservation execution costed.observation

/-- **SPEC §10.1**, `Universal.IsMaximalExecution`. -/
def IsMaximalExecution (S : Setting P) (bytes : ByteArray)
    (raw : Gemm.RawInvocation P)
    (initialization : InitializationObservation P) {initial : Wasm.Config}
    (execution : Wasm.MaximalExecution initial) : Prop :=
  StartsCostedInvocation S bytes raw initialization initial ∧
  IsMaximalExecutionFrom initial execution

/-- **SPEC §10.1**, `Universal.SemanticCorrectAt`.  Existence *and* universality:
it cannot be vacuously true for an empty run relation. -/
def SemanticCorrectAt (S : Setting P) (bytes : ByteArray)
    (raw : Gemm.RawInvocation P) : Prop :=
  (∃ initialization : InitializationObservation P,
    ∃ initial : Wasm.Config,
    ∃ execution : Wasm.MaximalExecution initial,
      IsMaximalExecution S bytes raw initialization execution) ∧
  ∀ (initialization : InitializationObservation P)
    (initial : Wasm.Config) (execution : Wasm.MaximalExecution initial),
    IsMaximalExecution S bytes raw initialization execution →
    ∃ observation,
      HasFiniteObservation execution observation ∧
      S.problem.Accepts raw observation

/-- **SPEC §10.1**, `Universal.SemanticCorrect`: over *every* raw invocation. -/
def SemanticCorrect (S : Setting P) (bytes : ByteArray) : Prop :=
  ∀ raw : Gemm.RawInvocation P, SemanticCorrectAt S bytes raw

/-- **SPEC §10.1**, `Universal.SemanticWithinResourcesAt`: no valid prefix
overruns the step budget, and every maximal execution is charged inside the
problem's limit. -/
def SemanticWithinResourcesAt (S : Setting P) (bytes : ByteArray)
    (raw : Gemm.RawInvocation P) : Prop :=
  (¬ ∃ initialization : InitializationObservation P,
    ∃ initial : Wasm.Config,
    ∃ pre : RelationalPrefix initial (S.problem.maxSteps + 1),
      StartsCostedInvocation S bytes raw initialization initial ∧ pre.Valid) ∧
  (∀ (initialization : InitializationObservation P)
    (initial : Wasm.Config) (execution : Wasm.MaximalExecution initial),
    IsMaximalExecution S bytes raw initialization execution →
    ∃ costed : CostedExecutionObservation S.semantics initial,
      CostedAs (S := S) execution costed ∧
      Cost.DynamicVector.ComponentwiseLE
        (Cost.sequentialCompose initialization.cost costed.cost)
        S.problem.limit)

/-- **SPEC §10.1**, `Universal.SemanticWithinResources`. -/
def SemanticWithinResources (S : Setting P) (bytes : ByteArray) : Prop :=
  ∀ raw : Gemm.RawInvocation P, SemanticWithinResourcesAt S bytes raw

/-- **SPEC §10.1**, `InvocationConforms`. -/
def InvocationConforms (S : Setting P) (bytes : ByteArray)
    (raw : Gemm.RawInvocation P) : Prop :=
  ProfileValid P bytes ∧
  SemanticCorrectAt S bytes raw ∧
  SemanticWithinResourcesAt S bytes raw

/--
  **SPEC §10.1**, `invocation_conforms_iff_extensional`.

  This is definitionally true, and that is the point: `InvocationConforms` adds
  *nothing* beyond the three extensional predicates.  It carries no certificate,
  no evaluation, no selector, and no artifact-specific side condition.  If a
  later edit smuggles an extra conjunct into `InvocationConforms`, this proof
  breaks.
-/
theorem invocation_conforms_iff_extensional (S : Setting P) (bytes : ByteArray)
    (raw : Gemm.RawInvocation P) :
    InvocationConforms S bytes raw ↔
      ProfileValid P bytes ∧
      SemanticCorrectAt S bytes raw ∧
      SemanticWithinResourcesAt S bytes raw :=
  Iff.rfl

/-- The pointwise conjunction of the three predicates is exactly conformance at
every raw invocation. -/
theorem invocation_conforms_forall (S : Setting P) (bytes : ByteArray) :
    (∀ raw : Gemm.RawInvocation P, InvocationConforms S bytes raw) ↔
      (Gemm.RawInvocation P → ProfileValid P bytes) ∧
      SemanticCorrect S bytes ∧ SemanticWithinResources S bytes := by
  constructor
  · intro h
    exact ⟨fun raw => (h raw).1, fun raw => (h raw).2.1, fun raw => (h raw).2.2⟩
  · intro h raw
    exact ⟨h.1 raw, h.2.1 raw, h.2.2 raw⟩

/-- Semantic correctness at every input is exactly `SemanticCorrect`. -/
theorem semanticCorrect_iff (S : Setting P) (bytes : ByteArray) :
    SemanticCorrect S bytes ↔ ∀ raw, SemanticCorrectAt S bytes raw :=
  Iff.rfl

/-- Resource conformance at every input is exactly `SemanticWithinResources`. -/
theorem semanticWithinResources_iff (S : Setting P) (bytes : ByteArray) :
    SemanticWithinResources S bytes ↔ ∀ raw, SemanticWithinResourcesAt S bytes raw :=
  Iff.rfl

/-- A semantically correct module really does have a maximal execution on every
input: `SemanticCorrectAt` is not vacuous. -/
theorem semanticCorrectAt_execution_exists {S : Setting P} {bytes : ByteArray}
    {raw : Gemm.RawInvocation P} (h : SemanticCorrectAt S bytes raw) :
    ∃ initialization : InitializationObservation P, ∃ initial : Wasm.Config,
      ∃ execution : Wasm.MaximalExecution initial,
        IsMaximalExecution S bytes raw initialization execution :=
  h.1

/-- Every maximal execution of a semantically correct module is accepted by the
problem's reference relation. -/
theorem semanticCorrectAt_accepts {S : Setting P} {bytes : ByteArray}
    {raw : Gemm.RawInvocation P} (h : SemanticCorrectAt S bytes raw)
    {initialization : InitializationObservation P} {initial : Wasm.Config}
    (execution : Wasm.MaximalExecution initial)
    (hmax : IsMaximalExecution S bytes raw initialization execution) :
    ∃ observation, HasFiniteObservation execution observation ∧
      S.problem.Accepts raw observation :=
  h.2 initialization initial execution hmax

/-- Semantic correctness rejects divergence: a divergent maximal execution
started by the bytes contradicts `SemanticCorrectAt`. -/
theorem semanticCorrectAt_not_diverges {S : Setting P} {bytes : ByteArray}
    {raw : Gemm.RawInvocation P} (h : SemanticCorrectAt S bytes raw)
    {initialization : InitializationObservation P} {initial : Wasm.Config}
    (events : Nat → Wasm.Event) (configs : Nat → Wasm.Config)
    (starts : configs 0 = initial)
    (step : ∀ i, Wasm.Step (configs i) (events i) (configs (i + 1)))
    (hstart : StartsCostedInvocation S bytes raw initialization initial) :
    False := by
  obtain ⟨observation, hobs, _⟩ :=
    h.2 initialization initial
      (Wasm.MaximalExecution.diverges events configs starts step) ⟨hstart, trivial⟩
  exact hobs

/-- Resource conformance forbids a valid prefix that overruns the step budget. -/
theorem semanticWithinResourcesAt_no_overrun {S : Setting P} {bytes : ByteArray}
    {raw : Gemm.RawInvocation P} (h : SemanticWithinResourcesAt S bytes raw) :
    ¬ ∃ initialization : InitializationObservation P, ∃ initial : Wasm.Config,
      ∃ pre : RelationalPrefix initial (S.problem.maxSteps + 1),
        StartsCostedInvocation S bytes raw initialization initial ∧ pre.Valid :=
  h.1

/-- Resource conformance charges every maximal execution inside the limit. -/
theorem semanticWithinResourcesAt_charge {S : Setting P} {bytes : ByteArray}
    {raw : Gemm.RawInvocation P} (h : SemanticWithinResourcesAt S bytes raw)
    {initialization : InitializationObservation P} {initial : Wasm.Config}
    (execution : Wasm.MaximalExecution initial)
    (hmax : IsMaximalExecution S bytes raw initialization execution) :
    ∃ costed : CostedExecutionObservation S.semantics initial,
      CostedAs (S := S) execution costed ∧
      Cost.DynamicVector.ComponentwiseLE
        (Cost.sequentialCompose initialization.cost costed.cost)
        S.problem.limit :=
  h.2 initialization initial execution hmax

/-- `ProfileValid` really does produce a decoded, validated, closed module. -/
theorem profileValid_module {P : Wasm.Profile} {bytes : ByteArray}
    (h : ProfileValid P bytes) :
    ∃ module : Wasm.Module,
      Wasm.decode bytes = .ok module ∧ validateUnder P module = true ∧
      module.imports = [] ∧ HasExactGemmExports P module :=
  h

/-- `ProfileValid` is decode-functional: the module it names is unique. -/
theorem profileValid_module_unique {bytes : ByteArray}
    {a b : Wasm.Module} (ha : Wasm.decode bytes = .ok a)
    (hb : Wasm.decode bytes = .ok b) : a = b := by
  rw [ha] at hb
  exact (Except.ok.injEq _ _ ▸ hb).symm ▸ rfl

end Semantic

end WasmGemmGnaf.Universal
