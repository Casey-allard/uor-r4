import WasmGemmGnaf.Gemm.Observe
set_option autoImplicit false
set_option exponentiation.threshold 400

/-!
# Gemm: the first-order problem body and the release problem (SPEC §8.3 end)

> `Gemm/Problem.lean` SHALL construct one first-order literal
> `Release.gemmProblemBody` containing exactly the feature, scalar,
> compatibility, arithmetic, ABI, layout, alias, observation, status-precedence,
> resource, and workload-repetition values in this section. …
> `canonicalWGNGv1ProblemBody` is a literal record constructor … every
> projection reduces by `rfl` to the numeric ABI, kind/mode table, arithmetic
> relation, status formula, resource value, and observation rule printed above.
> It takes no callback, typeclass-selected arithmetic, or artifact-derived
> argument.

Everything below is first order.  `ProblemBody` has no function field, no
typeclass field and no proposition field, so no theorem can substitute a
different table through it; `ProblemLawful` pins every one of those fields to
the value already implemented in `Gemm/*`, and `Problem.checked` refuses a body
that differs in a single row.

Each projection theorem in the `## Projections` section below is closed by
`rfl` and prints the literal numbers of SPEC §8.3.
-/

namespace WasmGemmGnaf.Gemm

/-! ## The observation rule as a first-order table (SPEC §8.3) -/

/-- Which of the three writable ranges one terminal phase sanctions. -/
structure WriteSelector where
  /-- May the phase write `C`? -/
  c : Bool
  /-- May the phase write the status-detail record? -/
  statusDetail : Bool
  /-- May the phase write the candidate-private scratch range? -/
  scratch : Bool
  deriving DecidableEq, Repr, Inhabited

namespace WriteSelector

/-- The byte ranges a selector names, in the fixed order `C`, status, scratch. -/
def regions (w : WriteSelector) (R : Regions) : List ByteRange :=
  (if w.c then [R.c] else []) ++ (if w.statusDetail then [R.statusDetail] else []) ++
    (if w.scratch then [R.scratch] else [])

end WriteSelector

/-- SPEC §8.3's closed phase-dependent observation rule, as a first-order table.
`∅`, `status`, `C ∪ status ∪ scratch`, `status ∪ scratch`. -/
def releaseSanctionedWriteTable : List (TerminalPhase × WriteSelector) :=
  [ (.noTrustworthyStatus, ⟨false, false, false⟩)
  , (.descriptorRejected,  ⟨false, true,  false⟩)
  , (.success,             ⟨true,  true,  true⟩)
  , (.runtimeFailure,      ⟨false, true,  true⟩) ]

/-- Look a phase up in a first-order selector table; an absent phase sanctions
nothing. -/
def selectorFor (t : List (TerminalPhase × WriteSelector)) (p : TerminalPhase) :
    WriteSelector :=
  match t.find? (fun row => row.1 == p) with
  | some row => row.2
  | none => ⟨false, false, false⟩

/-- **The table is the function.**  Reading the release table reproduces
`Gemm.sanctionedWriteRegions` of `Gemm/Observe.lean` exactly, for every phase. -/
theorem selectorFor_release (R : Regions) (p : TerminalPhase) :
    (selectorFor releaseSanctionedWriteTable p).regions R =
      sanctionedWriteRegions R p := by
  cases p <;> rfl

/-! ## The status-precedence classes (SPEC §8.3) -/

/-- SPEC §8.3's total, fixed classification precedence. -/
inductive PrecedenceClass
  /-- Malformed magic/version/header size/reserved bits, or truncated bytes. -/
  | malformedOrTruncated
  /-- Unknown or profile-disabled kind/mode tags. -/
  | unsupportedTag
  /-- Incompatible tags, descriptor overflow, illegal views, alias violations. -/
  | invalidDescriptor
  /-- Failure of the fixed resource predicate. -/
  | resourceExhausted
  /-- Everything that survives. -/
  | valid
  deriving DecidableEq, Repr, Inhabited

/-- The released precedence order, first matching class first. -/
def releasePrecedence : List PrecedenceClass :=
  [.malformedOrTruncated, .unsupportedTag, .invalidDescriptor, .resourceExhausted,
   .valid]

/-! ## The problem body -/

/--
  **SPEC §8.3.**  The complete first-order problem body.

  Every field is a number, a finite table of numbers, or a finite table of
  closed enumeration constructors.  There is no function field, no typeclass
  parameter and no proposition, so the identity of the problem is exactly the
  identity of this record.
-/
structure ProblemBody (_P : Wasm.ProfileBody) where
  /-- ABI magic `WGNG`, little-endian. -/
  abiMagic : Nat
  /-- ABI version. -/
  abiVersion : Nat
  /-- ABI header size in bytes. -/
  abiHeaderSize : Nat
  /-- Status-detail record size in bytes. -/
  statusDetailBytes : Nat
  /-- The scalar-kind tag column of SPEC §8.3. -/
  kindTagTable : List (ScalarKind × Nat)
  /-- The arithmetic-mode tag column. -/
  modeTagTable : List (ArithmeticMode × Nat)
  /-- The aliasing tag column. -/
  aliasTagTable : List (AliasingTag × Nat)
  /-- The six sanctioned function results. -/
  statusCodeTable : List (StatusCode × Nat)
  /-- The eleven status-detail field codes. -/
  fieldCodeTable : List (FieldCode × Nat)
  /-- Allowed-kind bitset for A, B and C. -/
  allowedKindBitset : Nat
  /-- Allowed-kind bitset for the accumulator. -/
  allowedAccumulatorKindBitset : Nat
  /-- Allowed arithmetic-mode bitset. -/
  allowedModeBitset : Nat
  /-- SPEC §8.2's closed compatibility table, in displayed row order. -/
  compatibilityTable : List CompatRow
  /-- The canonical quiet NaN payload of every released floating kind. -/
  canonicalQuietNaNTable : List (ScalarKind × Nat)
  /-- The fixed classification precedence. -/
  precedence : List PrecedenceClass
  /-- The closed phase-dependent observation rule. -/
  sanctionedWriteTable : List (TerminalPhase × WriteSelector)
  /-- SPEC §8.2's six release resource constants. -/
  resources : ResourceContract
  /-- SPEC §9.2's workload multiplicity. -/
  workloadRepetitions : Nat
  deriving DecidableEq, Repr, Inhabited

/-- The kind/tag column of SPEC §8.3, in tag order. -/
def releaseKindTagTable : List (ScalarKind × Nat) :=
  ScalarKind.all.map (fun k => (k, k.tag))

/-- The mode/tag column of SPEC §8.3. -/
def releaseModeTagTable : List (ArithmeticMode × Nat) :=
  ArithmeticMode.all.map (fun m => (m, m.tag))

/-- The alias/tag column of SPEC §8.3. -/
def releaseAliasTagTable : List (AliasingTag × Nat) :=
  AliasingTag.all.map (fun a => (a, a.tag))

/-- The status-code column of SPEC §8.3. -/
def releaseStatusCodeTable : List (StatusCode × Nat) :=
  StatusCode.all.map (fun s => (s, s.code))

/-- The field-code column of SPEC §8.3. -/
def releaseFieldCodeTable : List (FieldCode × Nat) :=
  FieldCode.all.map (fun f => (f, f.code))

/-- The canonical quiet NaN payload of every released floating kind
(SPEC §8.2). -/
def releaseCanonicalQuietNaNTable : List (ScalarKind × Nat) :=
  [ (.binary16, 0x7e00), (.bfloat16, 0x7fc0), (.binary32, 0x7fc00000)
  , (.binary64, 0x7ff8000000000000) ]

/--
  **SPEC §8.3.**  The canonical WGNG-v1 problem body: a literal record
  constructor.  Its only argument besides the profile body is the workload
  multiplicity of SPEC §9.2.  It takes no callback, no typeclass-selected
  arithmetic and no artifact-derived argument.
-/
def canonicalWGNGv1ProblemBody (P : Wasm.ProfileBody) (workloadRepetitions : Nat) :
    ProblemBody P where
  abiMagic := magicValue
  abiVersion := abiVersion
  abiHeaderSize := abiHeaderSize
  statusDetailBytes := statusDetailBytes
  kindTagTable := releaseKindTagTable
  modeTagTable := releaseModeTagTable
  aliasTagTable := releaseAliasTagTable
  statusCodeTable := releaseStatusCodeTable
  fieldCodeTable := releaseFieldCodeTable
  allowedKindBitset := allowedKindBitset
  allowedAccumulatorKindBitset := allowedAccumulatorKindBitset
  allowedModeBitset := allowedModeBitset
  compatibilityTable := compatibilityTable
  canonicalQuietNaNTable := releaseCanonicalQuietNaNTable
  precedence := releasePrecedence
  sanctionedWriteTable := releaseSanctionedWriteTable
  resources := releaseResourceContract
  workloadRepetitions := workloadRepetitions

/-! ## Projections

Every one of these is closed by `rfl`, and every right-hand side is a literal
printed in SPEC §8.2 or §8.3.  If a row, rounding rule, status, stride rule,
resource constant or workload multiplicity were replaced, one of these would
stop being true by `rfl`. -/

section Projections

variable (P : Wasm.ProfileBody) (w : Nat)

theorem canonical_abiMagic :
    (canonicalWGNGv1ProblemBody P w).abiMagic = 0x474e4757 := rfl

theorem canonical_abiVersion : (canonicalWGNGv1ProblemBody P w).abiVersion = 1 := rfl

theorem canonical_abiHeaderSize :
    (canonicalWGNGv1ProblemBody P w).abiHeaderSize = 256 := rfl

theorem canonical_statusDetailBytes :
    (canonicalWGNGv1ProblemBody P w).statusDetailBytes = 32 := rfl

theorem canonical_kindTagTable :
    (canonicalWGNGv1ProblemBody P w).kindTagTable =
      [(.i8, 0), (.u8, 1), (.i16, 2), (.u16, 3), (.i32, 4), (.u32, 5), (.i64, 6),
       (.u64, 7), (.binary16, 8), (.bfloat16, 9), (.binary32, 10), (.binary64, 11),
       (.exactDyadic, 12)] := rfl

theorem canonical_modeTagTable :
    (canonicalWGNGv1ProblemBody P w).modeTagTable =
      [(.modular, 0), (.checked, 1), (.strictFloat, 2), (.exactDyadicRoundOnce, 3)] :=
  rfl

theorem canonical_aliasTagTable :
    (canonicalWGNGv1ProblemBody P w).aliasTagTable =
      [(.disjoint, 0), (.cMayAliasAAfterRead, 1), (.cMayAliasBAfterRead, 2)] := rfl

theorem canonical_statusCodeTable :
    (canonicalWGNGv1ProblemBody P w).statusCodeTable =
      [(.success, 0), (.invalid, 1), (.unsupported, 2), (.resourceExhausted, 3),
       (.checkedOverflow, 4), (.arithmeticException, 5)] := rfl

theorem canonical_fieldCodeTable :
    (canonicalWGNGv1ProblemBody P w).fieldCodeTable =
      [(.none, 0), (.header, 1), (.version, 2), (.kind, 3), (.arithmeticMode, 4),
       (.dimension, 5), (.view, 6), (.alias, 7), (.resource, 8), (.overflow, 9),
       (.arithmetic, 10)] := rfl

theorem canonical_allowedKindBitset :
    (canonicalWGNGv1ProblemBody P w).allowedKindBitset = 0x0fff := rfl

theorem canonical_allowedAccumulatorKindBitset :
    (canonicalWGNGv1ProblemBody P w).allowedAccumulatorKindBitset = 0x1fff := rfl

theorem canonical_allowedModeBitset :
    (canonicalWGNGv1ProblemBody P w).allowedModeBitset = 0x0f := rfl

theorem canonical_compatibilityTable :
    (canonicalWGNGv1ProblemBody P w).compatibilityTable = Gemm.compatibilityTable :=
  rfl

theorem canonical_compatibilityTable_rows :
    (canonicalWGNGv1ProblemBody P w).compatibilityTable.map
        (fun r => (r.index, r.mode, r.storedKinds, r.accumulatorKinds)) =
      [ (0, .modular, [.i8, .u8, .i16, .u16, .i32, .u32], [.u32])
      , (1, .modular, [.i64, .u64], [.u64])
      , (2, .checked, [.i8, .i16, .i32, .i64], [.i64])
      , (3, .checked, [.u8, .u16, .u32, .u64], [.u64])
      , (4, .strictFloat, [.binary16, .bfloat16], [.binary32, .binary64])
      , (5, .strictFloat, [.binary32], [.binary32, .binary64])
      , (6, .strictFloat, [.binary64], [.binary64])
      , (7, .exactDyadicRoundOnce, [.binary16, .bfloat16, .binary32, .binary64],
         [.exactDyadic]) ] := rfl

theorem canonical_canonicalQuietNaNTable :
    (canonicalWGNGv1ProblemBody P w).canonicalQuietNaNTable =
      [(.binary16, 0x7e00), (.bfloat16, 0x7fc0), (.binary32, 0x7fc00000),
       (.binary64, 0x7ff8000000000000)] := rfl

theorem canonical_precedence :
    (canonicalWGNGv1ProblemBody P w).precedence =
      [.malformedOrTruncated, .unsupportedTag, .invalidDescriptor,
       .resourceExhausted, .valid] := rfl

theorem canonical_sanctionedWriteTable :
    (canonicalWGNGv1ProblemBody P w).sanctionedWriteTable =
      [ (.noTrustworthyStatus, ⟨false, false, false⟩)
      , (.descriptorRejected,  ⟨false, true,  false⟩)
      , (.success,             ⟨true,  true,  true⟩)
      , (.runtimeFailure,      ⟨false, true,  true⟩) ] := rfl

/-- **SPEC §8.2's release resource constants**, one by one. -/
theorem canonical_maxInvocationBytes :
    (canonicalWGNGv1ProblemBody P w).resources.maxInvocationBytes = 2 ^ 32 - 1 := rfl

theorem canonical_maxPages :
    (canonicalWGNGv1ProblemBody P w).resources.maxPages = 65536 := rfl

theorem canonical_maxTableElements :
    (canonicalWGNGv1ProblemBody P w).resources.maxTableElements = 2 ^ 32 - 1 := rfl

theorem canonical_maxHeapBytes :
    (canonicalWGNGv1ProblemBody P w).resources.maxHeapBytes = 2 ^ 32 - 1 := rfl

theorem canonical_maxValueSlots :
    (canonicalWGNGv1ProblemBody P w).resources.maxValueSlots = 2 ^ 64 - 1 := rfl

theorem canonical_maxCostedSteps :
    (canonicalWGNGv1ProblemBody P w).resources.maxCostedSteps = 2 ^ 320 := rfl

theorem canonical_workloadRepetitions :
    (canonicalWGNGv1ProblemBody P w).workloadRepetitions = w := rfl

/-- The observation rule stored in the body is the observation rule of
`Gemm/Observe.lean`, for every phase and every region assignment. -/
theorem canonical_observation_rule (R : Regions) (p : TerminalPhase) :
    (selectorFor (canonicalWGNGv1ProblemBody P w).sanctionedWriteTable p).regions R =
      sanctionedWriteRegions R p :=
  selectorFor_release R p

end Projections

/-! ## Lawfulness and the problem -/

/-- A body is lawful exactly when it is the canonical WGNG-v1 body for its own
workload multiplicity: every table, bitset, rounding constant and resource
value is pinned. -/
def ProblemLawful (P : Wasm.ProfileBody) (body : ProblemBody P) : Prop :=
  body = canonicalWGNGv1ProblemBody P body.workloadRepetitions

instance instDecidableProblemLawful (P : Wasm.ProfileBody) (body : ProblemBody P) :
    Decidable (ProblemLawful P body) :=
  inferInstanceAs (Decidable (_ = _))

theorem canonical_lawful (P : Wasm.ProfileBody) (w : Nat) :
    ProblemLawful P (canonicalWGNGv1ProblemBody P w) := rfl

/-- **SPEC §8.1/§8.3.**  A problem is a first-order body plus a proof that it is
the pinned one.  The proof is never stored as data. -/
structure Problem (P : Wasm.Profile) where
  /-- The first-order body. -/
  body : ProblemBody P.body
  /-- It is the canonical WGNG-v1 body. -/
  lawful : ProblemLawful P.body body

namespace Problem

variable {P : Wasm.Profile}

/-- Build a problem from a body whose lawfulness is decided. -/
def checked (P : Wasm.Profile) (body : ProblemBody P.body)
    (lawful : ProblemLawful P.body body := by decide) : Problem P :=
  { body := body, lawful := lawful }

@[simp] theorem checked_body (P : Wasm.Profile) (body : ProblemBody P.body)
    (h : ProblemLawful P.body body) : (checked P body h).body = body := rfl

/-- Two problems with the same body are equal. -/
theorem eq_of_body_eq {p q : Problem P} (h : p.body = q.body) : p = q := by
  cases p; cases q; cases h; rfl

/-- **SPEC §8.3**, `Gemm.Problem.RawInvocation`.  The raw-input carrier is fixed
before classification and cannot be narrowed by the problem implementation. -/
def RawInvocation (_problem : Problem P) : Type := Gemm.RawInvocation P

/-- The regions a raw invocation declares.  A raw input whose header does not
even decode declares nothing, so it sanctions nothing. -/
def regionsOf (problem : Problem P) (raw : problem.RawInvocation) : Regions :=
  match decodeHeader raw.body.bytes with
  | some h => (descriptorOf h).regions
  | none => ⟨⟨0, 0⟩, ⟨0, 0⟩, ⟨0, 0⟩, ⟨0, 0⟩, ⟨0, 0⟩, ⟨0, 0⟩⟩

/-- The problem's own per-invocation costed step budget (SPEC §8.2). -/
def maxSteps (problem : Problem P) : Nat := problem.body.resources.maxCostedSteps

/-- The problem's workload multiplicity (SPEC §9.2). -/
def workloadRepetitions (problem : Problem P) : Nat :=
  problem.body.workloadRepetitions

/-- The four released arithmetic modes.  Approximation and reassociation are
outside the first release, so every released mode is deterministic. -/
def deterministicModes : List ArithmeticMode := ArithmeticMode.all

/-- **SPEC §8.4**, `problem.ModeDeterministic`.  The mode the raw input selects
admits no unobserved freedom: no reassociation, no approximation, and a
canonicalized NaN payload. -/
def ModeDeterministic (_problem : Problem P) (raw : Gemm.RawInvocation P) : Prop :=
  ∀ h : RawHeader, decodeHeader raw.body.bytes = some h →
    (descriptorOf h).mode ∈ deterministicModes

/-- Every released mode is deterministic: the predicate is satisfied by every
raw invocation of the released problem.  It is not vacuous — it would fail for a
mode outside `ArithmeticMode.all`, which is exactly what an approximate GEMM
would need. -/
theorem modeDeterministic_of_release (problem : Problem P)
    (raw : Gemm.RawInvocation P) : problem.ModeDeterministic raw :=
  fun _ _ => ArithmeticMode.mem_all _

end Problem

/-- **SPEC §8.3**, `problem_raw_invocation_definitional`. -/
theorem problem_raw_invocation_definitional {P : Wasm.Profile} (problem : Problem P) :
    problem.RawInvocation = Gemm.RawInvocation P := rfl

/-- **SPEC §8.3**, `Gemm.toWasmInvocation`.  The harness places exactly
`raw.body.bytes` at `raw.body.ptr` and synthesizes no other descriptor data. -/
def toWasmInvocation {P : Wasm.Profile} (_problem : Problem P)
    (raw : Gemm.RawInvocation P) : Wasm.Invocation P :=
  Wasm.Invocation.gemmRaw raw.body.toWasm raw.lawful.toWasmInvocationLawful

/-- Every projection of a lawful problem's body is the pinned release value. -/
theorem lawful_body_eq {P : Wasm.Profile} (problem : Problem P) :
    problem.body =
      canonicalWGNGv1ProblemBody P.body problem.body.workloadRepetitions :=
  problem.lawful

theorem lawful_compatibilityTable {P : Wasm.Profile} (problem : Problem P) :
    problem.body.compatibilityTable = Gemm.compatibilityTable :=
  congrArg ProblemBody.compatibilityTable (lawful_body_eq problem)

theorem lawful_resources {P : Wasm.Profile} (problem : Problem P) :
    problem.body.resources = releaseResourceContract :=
  congrArg ProblemBody.resources (lawful_body_eq problem)

theorem lawful_sanctionedWriteTable {P : Wasm.Profile} (problem : Problem P) :
    problem.body.sanctionedWriteTable = releaseSanctionedWriteTable :=
  congrArg ProblemBody.sanctionedWriteTable (lawful_body_eq problem)

theorem lawful_abi {P : Wasm.Profile} (problem : Problem P) :
    problem.body.abiMagic = 0x474e4757 ∧ problem.body.abiVersion = 1 ∧
      problem.body.abiHeaderSize = 256 ∧ problem.body.statusDetailBytes = 32 :=
  ⟨congrArg ProblemBody.abiMagic (lawful_body_eq problem),
   congrArg ProblemBody.abiVersion (lawful_body_eq problem),
   congrArg ProblemBody.abiHeaderSize (lawful_body_eq problem),
   congrArg ProblemBody.statusDetailBytes (lawful_body_eq problem)⟩

end WasmGemmGnaf.Gemm

/-! ## The released problem

SPEC §8.3 fixes the release value as

```lean
def Release.gemmProblemBody : Gemm.ProblemBody Release.wasmProfile.body :=
  Gemm.canonicalWGNGv1ProblemBody Release.wasmProfile.body (workloadRepetitions := 1)
def Release.gemmProblem : Gemm.Problem Release.wasmProfile :=
  Gemm.Problem.checked Release.wasmProfile Release.gemmProblemBody
theorem Release.gemmProblem_body : Release.gemmProblem.body = Release.gemmProblemBody
```

`Release.wasmProfile` is SPEC §7.2's checked literal over
`Release.wasmCostTableBody`, the cost table built from the *vendored*
conformance map (SPEC §7.5).  Neither exists in this repository yet, and
neither may be invented here: a fabricated release cost table would silently
change the pinned profile identity that the whole release proposition is stated
against.  The released problem is therefore given below as the profile-indexed
family it is, so that `Release.gemmProblemBody := Release.gemmProblemBodyFor
Release.wasmProfile` is a one-line instantiation the moment SPEC §7.2's literal
lands.  Nothing about the GEMM problem itself is deferred: the body is complete,
literal and pinned.
-/

namespace WasmGemmGnaf.Release

open WasmGemmGnaf

/-- The released problem body over a profile: SPEC §8.3's literal, with
`workloadRepetitions = 1`. -/
def gemmProblemBodyFor (P : Wasm.Profile) : Gemm.ProblemBody P.body :=
  Gemm.canonicalWGNGv1ProblemBody P.body (workloadRepetitions := 1)

/-- The released GEMM problem over a profile. -/
def gemmProblemFor (P : Wasm.Profile) : Gemm.Problem P :=
  Gemm.Problem.checked P (gemmProblemBodyFor P) (Gemm.canonical_lawful P.body 1)

/-- **SPEC §8.3**, `Release.gemmProblem_body`. -/
theorem gemmProblemFor_body (P : Wasm.Profile) :
    (gemmProblemFor P).body = gemmProblemBodyFor P := rfl

/-- SPEC §8.2: the workload is charged exactly once. -/
theorem gemmProblemFor_workloadRepetitions (P : Wasm.Profile) :
    (gemmProblemFor P).body.workloadRepetitions = 1 := rfl

/-- SPEC §8.2's six release resource constants, on the released problem. -/
theorem gemmProblemFor_resources (P : Wasm.Profile) :
    (gemmProblemFor P).body.resources.maxInvocationBytes = 2 ^ 32 - 1 ∧
    (gemmProblemFor P).body.resources.maxPages = 65536 ∧
    (gemmProblemFor P).body.resources.maxTableElements = 2 ^ 32 - 1 ∧
    (gemmProblemFor P).body.resources.maxHeapBytes = 2 ^ 32 - 1 ∧
    (gemmProblemFor P).body.resources.maxValueSlots = 2 ^ 64 - 1 ∧
    (gemmProblemFor P).body.resources.maxCostedSteps = 2 ^ 320 :=
  ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩

end WasmGemmGnaf.Release
