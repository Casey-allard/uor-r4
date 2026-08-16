/-
  Search partitions of the universal competitor space.
  Normative source: SPEC.md section 10.4.

  SCOPE — read this before citing anything here.

  This file transcribes SPEC 10.4: the canonical partition scope, the canonical
  partition body and its exact denotation, and the five-constructor
  `PartitionResult`.  It proves the three things a partition layer is actually
  for:

    * the `split` rank decrease really makes recursive coverage well founded —
      an explicit natural measure, an explicit step relation, a `WellFounded`
      theorem, and a recursive cover function defined by that measure together
      with the theorem that its leaves cover every denoted byte string;
    * `PartitionBody.Denotes` is decidable, with an exact Boolean checker;
    * only the first four constructors may occur in a sealed certificate, and
      `incomplete` propagates to `Foundation.SolveResult.incomplete` — encoded
      as proved functions, not as prose.

  WHAT THIS FILE DOES NOT DO.

  It constructs **no** `dominated` instance for the release scope.  The
  `dominated` constructor's `memberLowerBound` field is exactly the undischarged
  universal lower bound of UOR-GNAF section 19.3; an inhabitant would be a fake
  proof of the decisive premise.  The constructor exists (a verifier must be
  able to *accept* one), and nothing here manufactures one.

  It also proves nothing about which byte strings a *particular* partition tree
  covers.  `coverLeaves_covers` is conditional on the parent's own denotation:
  it says refinement loses nothing, not that the root denotes everything.  See
  `coverLeaves_covers_scope` for the machine-checked statement of that limit.

  Every declaration in this file is either a definition or a kernel-checked
  theorem.  Nothing is assumed: no placeholder proof, no project axiom, and no
  compiled-evaluation decision procedure appears anywhere below.
-/
import WasmGemmGnaf.Universal.Sublevel
import WasmGemmGnaf.Foundation.Termination
import WasmGemmGnaf.Gemm.Problem

set_option autoImplicit false

namespace WasmGemmGnaf

open WasmGemmGnaf.Foundation

/-! ## Canonical identity of a lawful GEMM problem (SPEC 10.4, `Gemm.ProblemId`)

`Gemm/Problem.lean` ships `Gemm.Problem` and `Gemm.ProblemLawful` but no
identity function, and SPEC 10.4's `PartitionScope` needs one.  A lawful problem
is *pinned*: `ProblemLawful P body` says `body = canonicalWGNGv1ProblemBody P
body.workloadRepetitions`, so the profile identity together with the workload
multiplicity determines the whole first-order body.  The canonical identity is
therefore taken over exactly that first-order pair, and `ProblemId_eq_iff`
proves — it does not assume — that this is a faithful identity on `Problem P`. -/

namespace Gemm

/-- The first-order body a lawful problem's identity is taken over: the profile
identity and the workload multiplicity.  Every other field of `ProblemBody` is
a function of these two by `ProblemLawful`. -/
structure ProblemIdBody where
  /-- The canonical identity of the profile the problem is stated over. -/
  profile : Foundation.ProfileId
  /-- SPEC 9.2: how many times the workload is charged. -/
  workloadRepetitions : Nat
  deriving DecidableEq

namespace ProblemIdBody

/-- The flattening used to build the canonical encoder. -/
def toTuple (b : ProblemIdBody) : Foundation.CanonicalObjectId × Nat :=
  (b.profile, b.workloadRepetitions)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [ProblemIdBody.mk.injEq]
  exact h

/-- The canonical prefix-free encoding of a problem identity body. -/
def bytes (b : ProblemIdBody) : List UInt8 :=
  Bytes.pairBytes CanonicalObjectId.bytes Bytes.natBytes b.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
    Bytes.natBytes_prefixFree).comp toTuple_injective

/-- The frozen canonical schema of a problem identity body. -/
def identitySchema : CanonicalSchema ProblemIdBody :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.gemmProblem
    (TypeTag.leaf "Gemm.ProblemIdBody".toUTF8.toList)
    (TypeTag.leaf_size_pos _)
    bytes bytes_prefixFree

end ProblemIdBody

/-- **SPEC 10.4**, `Gemm.ProblemId`: the canonical identity of a lawful GEMM
problem. -/
def ProblemId {P : Wasm.Profile} (problem : Problem P) : Foundation.ProblemId :=
  CanonicalObjectId.ofTyped
    (Identity ProblemIdBody.identitySchema
      ⟨Wasm.ProfileId P, problem.workloadRepetitions⟩)

/-- Problem identity is faithful: it is a structural comparison of lawful
problems, not a digest whose collision resistance would have to be assumed. -/
theorem ProblemId_eq_iff {P : Wasm.Profile} {a b : Problem P} :
    ProblemId a = ProblemId b ↔ a = b := by
  constructor
  · intro h
    have h' :
        (⟨Wasm.ProfileId P, a.workloadRepetitions⟩ : ProblemIdBody) =
          ⟨Wasm.ProfileId P, b.workloadRepetitions⟩ :=
      (CanonicalObjectId.ofTyped_Identity_eq_iff ProblemIdBody.identitySchema).mp h
    have hrep : a.workloadRepetitions = b.workloadRepetitions :=
      congrArg ProblemIdBody.workloadRepetitions h'
    refine Problem.eq_of_body_eq ?_
    rw [a.lawful, b.lawful]
    exact congrArg _ hrep
  · intro h; rw [h]

theorem ProblemId_injective {P : Wasm.Profile} :
    Function.Injective (ProblemId (P := P)) :=
  fun _ _ h => ProblemId_eq_iff.mp h

theorem ProblemId_domain {P : Wasm.Profile} (problem : Problem P) :
    (ProblemId problem).domain = CanonicalDomainTag.gemmProblem := rfl

end Gemm

/-! ## Canonical identity of an objective (SPEC 10.4, `Cost.ObjectiveId`)

`Cost.ProperObjective` carries a first-order `body : ObjectiveBody`, a
positivity proof, a `boundOfScore` *function* and a sublevel proof.  SPEC 16 is
explicit that "no function or proof is hashed", so the objective identity is
taken over the first-order body alone, and `ObjectiveId_eq_iff` says exactly
that and no more. -/

namespace Cost

namespace ObjectiveBody

/-- The flattening used to build the canonical encoder: the version and the
thirty-six charged coordinate weights, in `ArtifactCoordinate.all` order.
`tieOrder` carries no information — `TieOrderTag` has one constructor — and is
recovered by case analysis in `toTuple_injective`. -/
def toTuple (b : ObjectiveBody) : Nat × List Nat :=
  (b.version,
    [ b.staticWeights.moduleBytes, b.staticWeights.decodeSteps
    , b.staticWeights.validationSteps, b.staticWeights.staticDataBytes
    , b.dynamicSumWeights.instantiationSteps, b.dynamicSumWeights.dispatchSteps
    , b.dynamicSumWeights.preparationSteps, b.dynamicSumWeights.wasmRuleSteps
    , b.dynamicSumWeights.scalarOps, b.dynamicSumWeights.vectorLaneOps
    , b.dynamicSumWeights.bytesRead, b.dynamicSumWeights.bytesWritten
    , b.dynamicSumWeights.memoryGrowPages
    , b.dynamicSumWeights.tableElementsAllocated
    , b.dynamicSumWeights.gcObjectsAllocated
    , b.dynamicSumWeights.gcBytesInitialized
    , b.dynamicSumWeights.peakStackValues, b.dynamicSumWeights.peakPages
    , b.dynamicSumWeights.peakGcLiveBytes, b.dynamicSumWeights.outputBytes
    , b.dynamicMaxWeights.instantiationSteps, b.dynamicMaxWeights.dispatchSteps
    , b.dynamicMaxWeights.preparationSteps, b.dynamicMaxWeights.wasmRuleSteps
    , b.dynamicMaxWeights.scalarOps, b.dynamicMaxWeights.vectorLaneOps
    , b.dynamicMaxWeights.bytesRead, b.dynamicMaxWeights.bytesWritten
    , b.dynamicMaxWeights.memoryGrowPages
    , b.dynamicMaxWeights.tableElementsAllocated
    , b.dynamicMaxWeights.gcObjectsAllocated
    , b.dynamicMaxWeights.gcBytesInitialized
    , b.dynamicMaxWeights.peakStackValues, b.dynamicMaxWeights.peakPages
    , b.dynamicMaxWeights.peakGcLiveBytes, b.dynamicMaxWeights.outputBytes ])

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  obtain ⟨va, sa, dsa, dma, ta⟩ := a
  obtain ⟨vb, sb, dsb, dmb, tb⟩ := b
  cases ta; cases tb
  cases sa; cases sb; cases dsa; cases dsb; cases dma; cases dmb
  simp_all [toTuple]

/-- The canonical prefix-free encoding of an objective body. -/
def bytes (b : ObjectiveBody) : List UInt8 :=
  Bytes.pairBytes Bytes.natBytes (Bytes.listBytes Bytes.natBytes) b.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
    (Bytes.listBytes_prefixFree Bytes.natBytes_prefixFree)).comp toTuple_injective

/-- The frozen canonical schema of an objective body. -/
def identitySchema : CanonicalSchema ObjectiveBody :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.costObjective
    (TypeTag.leaf "Cost.ObjectiveBody".toUTF8.toList)
    (TypeTag.leaf_size_pos _)
    bytes bytes_prefixFree

end ObjectiveBody

/-- **SPEC 10.4**, `Cost.ObjectiveId`: the canonical identity of an objective,
taken over its first-order body. -/
def ObjectiveId {Profile Problem : Type} {P : Profile} {G : Problem}
    (objective : ProperObjective P G) : Foundation.ObjectiveId :=
  CanonicalObjectId.ofTyped (Identity ObjectiveBody.identitySchema objective.body)

/-- Objective identity is exactly first-order body identity.  It deliberately
does *not* separate two objectives that differ only in the `boundOfScore`
function: SPEC 16 forbids hashing a function. -/
theorem ObjectiveId_eq_iff {Profile Problem : Type} {P : Profile} {G : Problem}
    {a b : ProperObjective P G} : ObjectiveId a = ObjectiveId b ↔ a.body = b.body :=
  CanonicalObjectId.ofTyped_Identity_eq_iff ObjectiveBody.identitySchema

theorem ObjectiveId_domain {Profile Problem : Type} {P : Profile} {G : Problem}
    (objective : ProperObjective P G) :
    (ObjectiveId objective).domain = CanonicalDomainTag.costObjective := rfl

end Cost

namespace Universal

/-! ## Canonical lists (SPEC 10.4)

SPEC 10.4 writes `CanonicalList α` and `NonemptyCanonicalList α`.  The nonempty
form already exists as `Foundation.NonemptyCanonicalFrontier`, and is reused
verbatim; the possibly-empty form is its sibling. -/

/-- A finite list presented in the canonical byte order of a frozen schema. -/
structure CanonicalList (α : Type) where
  /-- The frozen schema the order is taken in. -/
  schema : CanonicalSchema α
  /-- The elements, in canonical order. -/
  elements : List α
  /-- They really are canonically ordered. -/
  ordered : CanonicalSorted (elements.map schema.encode)

/-- SPEC 10.4's `NonemptyCanonicalList`: the existing canonical frontier. -/
abbrev NonemptyCanonicalList (α : Type) := Foundation.NonemptyCanonicalFrontier α

namespace CanonicalList

variable {α : Type}

/-- The empty canonical list under a schema. -/
def empty (schema : CanonicalSchema α) : CanonicalList α where
  schema := schema
  elements := []
  ordered := by simp

@[simp] theorem empty_elements (schema : CanonicalSchema α) :
    (empty schema).elements = [] := rfl

/-- Every one-element list is canonically ordered. -/
def singleton (schema : CanonicalSchema α) (a : α) : CanonicalList α where
  schema := schema
  elements := [a]
  ordered := by simp

@[simp] theorem singleton_elements (schema : CanonicalSchema α) (a : α) :
    (singleton schema a).elements = [a] := rfl

end CanonicalList

/-! ## The constraint language (SPEC 10.4, `CanonicalConstraintSet`)

SPEC 10.4 lets a partition be cut "by canonical module-byte prefix, validation
type, control-flow summary, semantic summary, or proved lower bound".  The
constraint grammar below is deliberately **decidable and syntactic**: every
constructor's `Holds` is a computable predicate of the bytes and the pinned
profile.  That is what makes `PartitionBody.Denotes` decidable, which is in turn
what makes a partition checkable rather than merely asserted.  A constraint
whose truth is not decidable has no constructor here on purpose: it could not be
verified, only believed. -/

/-- One decidable syntactic constraint on a competitor byte string. -/
inductive ByteConstraint
  /-- The byte string has size at most `n`. -/
  | sizeAtMost (n : Nat)
  /-- The byte string has size at least `n`. -/
  | sizeAtLeast (n : Nat)
  /-- The byte string has size exactly `n`. -/
  | sizeEq (n : Nat)
  /-- The byte at `index` is `value`; an out-of-range index never holds. -/
  | byteEq (index : Nat) (value : UInt8)
  /-- The byte string begins with `prefix`. -/
  | prefixEq (bytePrefix : ByteArray)
  /-- The byte string decodes to a WebAssembly module. -/
  | decodes
  /-- It decodes and validates under the scope's pinned profile. -/
  | validates
  deriving DecidableEq

namespace ByteConstraint

/-- The flattening used to build the canonical encoder. -/
def toTuple : ByteConstraint → Nat × Nat × UInt8 × ByteArray
  | sizeAtMost n => (0, n, 0, Bytes.pack [])
  | sizeAtLeast n => (1, n, 0, Bytes.pack [])
  | sizeEq n => (2, n, 0, Bytes.pack [])
  | byteEq i v => (3, i, v, Bytes.pack [])
  | prefixEq p => (4, 0, 0, p)
  | decodes => (5, 0, 0, Bytes.pack [])
  | validates => (6, 0, 0, Bytes.pack [])

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a <;> cases b <;> simp_all [toTuple]

/-- The canonical prefix-free encoding of a constraint. -/
def bytes (c : ByteConstraint) : List UInt8 :=
  Bytes.pairBytes Bytes.natBytes
    (Bytes.pairBytes Bytes.natBytes
      (Bytes.pairBytes Bytes.u8Bytes Bytes.byteArrayBytes)) c.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
    (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
      (Bytes.pairBytes_prefixFree Bytes.u8Bytes_prefixFree
        Bytes.byteArrayBytes_prefixFree))).comp toTuple_injective

/-- The frozen canonical schema of a constraint. -/
def identitySchema : CanonicalSchema ByteConstraint :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.generic
    (TypeTag.leaf "Universal.ByteConstraint".toUTF8.toList)
    (TypeTag.leaf_size_pos _)
    bytes bytes_prefixFree

/-- **SPEC 10.4**, `constraint.Holds bytes`: the exact denotation of a
constraint, relative to the scope's pinned profile. -/
def Holds (P : Wasm.Profile) : ByteConstraint → ByteArray → Prop
  | sizeAtMost n, b => b.size ≤ n
  | sizeAtLeast n, b => n ≤ b.size
  | sizeEq n, b => b.size = n
  | byteEq i v, b => b.toList[i]? = some v
  | prefixEq p, b => b.toList.take p.size = p.toList
  | decodes, b => ∃ m : Wasm.Module, Wasm.decode b = .ok m
  | validates, b =>
      ∃ m : Wasm.Module, Wasm.decode b = .ok m ∧ validateUnder P m = true

/-- The exact Boolean check for `decodes`. -/
def decodesCheck (b : ByteArray) : Bool :=
  match Wasm.decode b with
  | .ok _ => true
  | .error _ => false

theorem decodesCheck_iff (b : ByteArray) :
    decodesCheck b = true ↔ ∃ m : Wasm.Module, Wasm.decode b = .ok m := by
  cases h : Wasm.decode b with
  | ok m => simp [decodesCheck, h]
  | error e => simp [decodesCheck, h]

/-- Deciding whether the bytes decode. -/
instance instDecidableDecodes (b : ByteArray) :
    Decidable (∃ m : Wasm.Module, Wasm.decode b = .ok m) :=
  decidable_of_iff _ (decodesCheck_iff b)

/-- The exact Boolean check for `validates`. -/
def validatesCheck (P : Wasm.Profile) (b : ByteArray) : Bool :=
  match Wasm.decode b with
  | .ok m => validateUnder P m
  | .error _ => false

theorem validatesCheck_iff (P : Wasm.Profile) (b : ByteArray) :
    validatesCheck P b = true ↔
      ∃ m : Wasm.Module, Wasm.decode b = .ok m ∧ validateUnder P m = true := by
  cases h : Wasm.decode b with
  | ok m => simp [validatesCheck, h]
  | error e => simp [validatesCheck, h]

/-- Deciding whether the bytes decode and validate. -/
instance instDecidableValidates (P : Wasm.Profile) (b : ByteArray) :
    Decidable (∃ m : Wasm.Module, Wasm.decode b = .ok m ∧ validateUnder P m = true) :=
  decidable_of_iff _ (validatesCheck_iff P b)

/-- **Every constraint is decidable.**  A partition cell can therefore be
checked, not merely asserted. -/
instance instDecidableHolds (P : Wasm.Profile) (c : ByteConstraint) (b : ByteArray) :
    Decidable (Holds P c b) := by
  cases c <;> unfold Holds <;> infer_instance

end ByteConstraint

/-- **SPEC 10.4**, `CanonicalConstraintSet`: a canonically ordered finite set of
decidable constraints. -/
structure CanonicalConstraintSet where
  /-- The constraints, in canonical order. -/
  constraints : List ByteConstraint
  /-- They really are canonically ordered. -/
  ordered : CanonicalSorted (constraints.map ByteConstraint.identitySchema.encode)

namespace CanonicalConstraintSet

/-- The empty constraint set. -/
def empty : CanonicalConstraintSet := ⟨[], by simp⟩

@[simp] theorem empty_constraints : empty.constraints = [] := rfl

theorem eq_of_constraints_eq {a b : CanonicalConstraintSet}
    (h : a.constraints = b.constraints) : a = b := by
  cases a; cases b; cases h; rfl

/-- The canonical prefix-free encoding of a constraint set. -/
def bytes (s : CanonicalConstraintSet) : List UInt8 :=
  Bytes.listBytes ByteConstraint.bytes s.constraints

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.listBytes_prefixFree ByteConstraint.bytes_prefixFree).comp
    (fun _ _ h => eq_of_constraints_eq h)

/-- Every constraint of the set holds of `bytes`. -/
def Holds (P : Wasm.Profile) (s : CanonicalConstraintSet) (bytes : ByteArray) : Prop :=
  ∀ constraint ∈ s.constraints, ByteConstraint.Holds P constraint bytes

instance instDecidableHolds (P : Wasm.Profile) (s : CanonicalConstraintSet)
    (bytes : ByteArray) : Decidable (Holds P s bytes) := by
  unfold Holds; infer_instance

end CanonicalConstraintSet

/-! ## The partition scope (SPEC 10.4) -/

/-- **SPEC 10.4**, `PartitionScopeBody`: the proof-free identity preimage of a
partition scope.  Every field is first order. -/
structure PartitionScopeBody where
  /-- The pinned profile's canonical identity. -/
  profileId : Foundation.ProfileId
  /-- The pinned problem's canonical identity. -/
  problemId : Foundation.ProblemId
  /-- The pinned objective's canonical identity. -/
  objectiveId : Foundation.ObjectiveId
  /-- The resource sublevel the search is confined to. -/
  sublevel : Cost.ResourceBounds
  /-- The incumbent score the search must beat or match. -/
  baselineScore : Nat

variable {P : Wasm.Profile} [Foundation.Fintype (Gemm.RawInvocation P)]

/--
  **SPEC 10.4**, `PartitionScope`.

  The resolved scope, together with the three identity equalities that bind its
  proof-free `body` to the resolved profile, problem and objective.  Those
  equalities are the reason the body can be used as an identity preimage while
  the resolved objects stay verifier inputs.

  Two repository-specific fields beyond SPEC's list.  First, this repository
  splits SPEC's `Gemm.Problem` into the first-order `Gemm.Problem` (which owns
  the identity) and `Universal.Setting` (which owns the costed machine and the
  reference relation, see the seam note in `Universal/Competitor.lean`);
  `stepsEq` and `repetitionsEq` pin the two together, so the identity really is
  the identity of the problem the semantic predicates are stated over.  Second,
  the decider is a field because `SystemEvaluationRel` needs it.
-/
structure PartitionScope (P : Wasm.Profile)
    [Foundation.Fintype (Gemm.RawInvocation P)] where
  /-- The costed semantics, machine and problem seam. -/
  setting : Setting P
  /-- The first-order lawful GEMM problem. -/
  gemmProblem : Gemm.Problem P
  /-- The finite evaluator of SPEC 10.1. -/
  decider : Decider setting
  /-- The proper objective. -/
  objective : Cost.ProperObjective P setting.problem
  /-- The proof-free identity preimage. -/
  body : PartitionScopeBody
  /-- SPEC 10.4, identity equality 1. -/
  profileIdEq : body.profileId = Wasm.ProfileId P
  /-- SPEC 10.4, identity equality 2. -/
  problemIdEq : body.problemId = Gemm.ProblemId gemmProblem
  /-- SPEC 10.4, identity equality 3. -/
  objectiveIdEq : body.objectiveId = Cost.ObjectiveId objective
  /-- The seam problem's step budget is the lawful problem's step budget. -/
  stepsEq : setting.problem.maxSteps = gemmProblem.maxSteps
  /-- The seam problem's workload multiplicity is the lawful one. -/
  repetitionsEq :
    setting.problem.workloadRepetitions = gemmProblem.workloadRepetitions

namespace PartitionScope

/-- The pinned profile of a scope. -/
def profile (_scope : PartitionScope P) : Wasm.Profile := P

/-- **SPEC 10.4**, `PartitionScope.sublevel`. -/
def sublevel (scope : PartitionScope P) : Cost.ResourceBounds :=
  scope.body.sublevel

/-- **SPEC 10.4**, `PartitionScope.baselineScore`. -/
def baselineScore (scope : PartitionScope P) : Nat :=
  scope.body.baselineScore

@[simp] theorem profile_eq (scope : PartitionScope P) : scope.profile = P := rfl

/-- The scope body's profile identity is the resolved profile's identity: the
identity preimage cannot name a different profile. -/
theorem body_profileId (scope : PartitionScope P) :
    scope.body.profileId = Wasm.ProfileId scope.profile := scope.profileIdEq

/-- The scope body's problem identity is the resolved problem's identity. -/
theorem body_problemId (scope : PartitionScope P) :
    scope.body.problemId = Gemm.ProblemId scope.gemmProblem := scope.problemIdEq

/-- The scope body's objective identity is the resolved objective's identity. -/
theorem body_objectiveId (scope : PartitionScope P) :
    scope.body.objectiveId = Cost.ObjectiveId scope.objective := scope.objectiveIdEq

/-- Two scopes with the same body have the same lawful problem. -/
theorem gemmProblem_eq_of_body_eq {a b : PartitionScope P}
    (h : a.body = b.body) : a.gemmProblem = b.gemmProblem := by
  have hid := congrArg PartitionScopeBody.problemId h
  rw [a.problemIdEq, b.problemIdEq] at hid
  exact Gemm.ProblemId_eq_iff.mp hid

/-- Two scopes with the same body name the same objective identity.  (The
objective *bodies* cannot be compared without also fixing the seam problem the
objective is indexed by, so the identity is the sharpest statement available
here; `Cost.ObjectiveId_eq_iff` turns it into body equality whenever the two
scopes share a setting.) -/
theorem objectiveId_eq_of_body_eq {a b : PartitionScope P}
    (h : a.body = b.body) :
    Cost.ObjectiveId a.objective = Cost.ObjectiveId b.objective := by
  have hid := congrArg PartitionScopeBody.objectiveId h
  rw [a.objectiveIdEq, b.objectiveIdEq] at hid
  exact hid

end PartitionScope

/-! ## The partition body and its exact denotation (SPEC 10.4) -/

/-- **SPEC 10.4**, `PartitionBody`: one cell of the search partition. -/
structure PartitionBody (_scope : PartitionScope P) where
  /-- The canonical module-byte prefix the cell fixes. -/
  bytePrefix : ByteArray
  /-- The maximum size of the remaining suffix. -/
  suffixLengthBound : Nat
  /-- The decidable semantic constraints the cell imposes. -/
  semanticConstraints : CanonicalConstraintSet
  /-- The refinement rank: `split` must strictly decrease it. -/
  rank : Nat

namespace PartitionBody

variable {scope : PartitionScope P}

/--
  **SPEC 10.4**, `PartitionBody.Denotes`.

  SPEC writes the field name `prefix`, which is a Lean keyword; it is spelled
  `bytePrefix` here and nowhere else does the definition differ.
-/
def Denotes (p : PartitionBody scope) (bytes : ByteArray) : Prop :=
  ∃ suffix : ByteArray,
    bytes = p.bytePrefix ++ suffix ∧
    suffix.size ≤ p.suffixLengthBound ∧
    ∀ constraint ∈ p.semanticConstraints.constraints,
      ByteConstraint.Holds P constraint bytes

/-- The flattening used to build the canonical encoder. -/
def toTuple (p : PartitionBody scope) :
    ByteArray × Nat × List ByteConstraint × Nat :=
  (p.bytePrefix, p.suffixLengthBound, p.semanticConstraints.constraints, p.rank)

theorem toTuple_injective : Function.Injective (toTuple (scope := scope)) := by
  intro a b h
  obtain ⟨pa, sa, ⟨ca, hca⟩, ra⟩ := a
  obtain ⟨pb, sb, ⟨cb, hcb⟩, rb⟩ := b
  simp only [toTuple, Prod.mk.injEq] at h
  obtain ⟨h1, h2, h3, h4⟩ := h
  subst h1; subst h2; subst h3; subst h4
  rfl

/-- The canonical prefix-free encoding of a partition body. -/
def bytes (p : PartitionBody scope) : List UInt8 :=
  Bytes.pairBytes Bytes.byteArrayBytes
    (Bytes.pairBytes Bytes.natBytes
      (Bytes.pairBytes (Bytes.listBytes ByteConstraint.bytes) Bytes.natBytes))
    p.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree (bytes (scope := scope)) :=
  (Bytes.pairBytes_prefixFree Bytes.byteArrayBytes_prefixFree
    (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
      (Bytes.pairBytes_prefixFree
        (Bytes.listBytes_prefixFree ByteConstraint.bytes_prefixFree)
        Bytes.natBytes_prefixFree))).comp toTuple_injective

/-- The frozen canonical schema of a partition body. -/
def identitySchema (scope : PartitionScope P) :
    CanonicalSchema (PartitionBody scope) :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.generic
    (TypeTag.leaf "Universal.PartitionBody".toUTF8.toList)
    (TypeTag.leaf_size_pos _)
    bytes bytes_prefixFree

/-- **SPEC 10.4**: the canonical identity of a partition cell.  Every result
identity binds this. -/
def PartitionBodyId (p : PartitionBody scope) : Foundation.CanonicalObjectId :=
  CanonicalObjectId.ofTyped (Identity (identitySchema scope) p)

theorem PartitionBodyId_eq_iff {a b : PartitionBody scope} :
    PartitionBodyId a = PartitionBodyId b ↔ a = b :=
  CanonicalObjectId.ofTyped_Identity_eq_iff (identitySchema scope)

/-! ### `Denotes` is decidable -/

/-- The exact Boolean checker for `Denotes`. -/
def denotesCheck (p : PartitionBody scope) (bytes : ByteArray) : Bool :=
  decide (bytes.toList.take p.bytePrefix.size = p.bytePrefix.toList) &&
  decide (p.bytePrefix.size ≤ bytes.size) &&
  decide (bytes.size - p.bytePrefix.size ≤ p.suffixLengthBound) &&
  decide (CanonicalConstraintSet.Holds P p.semanticConstraints bytes)

/-- **The checker is exact**: it accepts precisely the denoted byte strings.
This is the fact that makes a partition cell verifiable. -/
theorem denotesCheck_iff (p : PartitionBody scope) (bytes : ByteArray) :
    denotesCheck p bytes = true ↔ p.Denotes bytes := by
  simp only [denotesCheck, Bool.and_eq_true, decide_eq_true_eq]
  constructor
  · rintro ⟨⟨⟨htake, hsize⟩, hbound⟩, hcon⟩
    refine ⟨Bytes.pack (bytes.toList.drop p.bytePrefix.size), ?_, ?_, hcon⟩
    · apply Bytes.toList_injective
      rw [Bytes.toList_append, Bytes.toList_pack, ← htake, List.take_append_drop]
    · rw [Bytes.size_pack, List.length_drop, Bytes.length_toList]
      exact hbound
  · rintro ⟨suffix, hbytes, hsuffix, hcon⟩
    subst hbytes
    have hlist : (p.bytePrefix ++ suffix).toList
        = p.bytePrefix.toList ++ suffix.toList := Bytes.toList_append _ _
    refine ⟨⟨⟨?_, ?_⟩, ?_⟩, hcon⟩
    · rw [hlist, ← Bytes.length_toList p.bytePrefix, List.take_left]
    · simp
    · simp only [ByteArray.size_append]
      omega

instance instDecidableDenotes (p : PartitionBody scope) (bytes : ByteArray) :
    Decidable (p.Denotes bytes) :=
  decidable_of_iff _ (denotesCheck_iff p bytes)

/-- Denotation forces the cell's own size bound: a cell denotes only byte
strings no longer than its prefix plus its suffix bound. -/
theorem size_le_of_denotes {p : PartitionBody scope} {bytes : ByteArray}
    (h : p.Denotes bytes) :
    bytes.size ≤ p.bytePrefix.size + p.suffixLengthBound := by
  obtain ⟨suffix, hbytes, hsuffix, -⟩ := h
  subst hbytes
  simp only [ByteArray.size_append]
  omega

/-- Denotation forces every recorded constraint. -/
theorem constraints_of_denotes {p : PartitionBody scope} {bytes : ByteArray}
    (h : p.Denotes bytes) :
    ∀ constraint ∈ p.semanticConstraints.constraints,
      ByteConstraint.Holds P constraint bytes := by
  obtain ⟨-, -, -, hcon⟩ := h
  exact hcon

end PartitionBody

/-! ## Disjointness of denotations (SPEC 10.4) -/

/-- Two cells are disjoint when no byte string is denoted by both. -/
def DisjointDenotations {scope : PartitionScope P}
    (a b : PartitionBody scope) : Prop :=
  ∀ bytes : ByteArray, ¬ (a.Denotes bytes ∧ b.Denotes bytes)

/-- **SPEC 10.4**, `PairwiseDisjointDenotations`. -/
def PairwiseDisjointDenotations {scope : PartitionScope P}
    (children : NonemptyCanonicalList (PartitionBody scope)) : Prop :=
  List.Pairwise DisjointDenotations children.elements

/-- A pairwise-disjoint family really does separate distinct cells. -/
theorem disjoint_of_pairwise {scope : PartitionScope P}
    {children : NonemptyCanonicalList (PartitionBody scope)}
    (h : PairwiseDisjointDenotations children) {a b : PartitionBody scope}
    {bytes : ByteArray}
    (hpair : List.Pairwise DisjointDenotations children.elements →
      DisjointDenotations a b)
    (ha : a.Denotes bytes) (hb : b.Denotes bytes) : False :=
  hpair h bytes ⟨ha, hb⟩

/-! ## Checked byte results (SPEC 10.4) -/

/-- The outcome a member of an exhausted cell was checked to.  This is a
*recorded* outcome: `PartitionResult.exhausted`'s `exact` field constrains only
`member.bytes`, never `member.outcome`, and `exhausted_exact_blind_to_outcome`
below proves exactly that. -/
inductive ByteCheckOutcome
  /-- The bytes do not decode and validate under the pinned profile. -/
  | profileInvalid
  /-- The bytes were evaluated to this score. -/
  | evaluated (score : Nat)
  deriving DecidableEq, Repr

/-- **SPEC 10.4**, `CheckedByteResult`: one enumerated member of an exhausted
cell. -/
structure CheckedByteResult where
  /-- The member's byte string. -/
  bytes : ByteArray
  /-- The recorded check outcome. -/
  outcome : ByteCheckOutcome
  deriving DecidableEq

namespace CheckedByteResult

/-- The flattening used to build the canonical encoder. -/
def toTuple (r : CheckedByteResult) : ByteArray × Nat × Nat :=
  (r.bytes,
    match r.outcome with
    | .profileInvalid => (0, 0)
    | .evaluated s => (1, s))

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  obtain ⟨ba, oa⟩ := a
  obtain ⟨bb, ob⟩ := b
  cases oa <;> cases ob <;> simp_all [toTuple]

/-- The canonical prefix-free encoding of a checked byte result. -/
def bytesEnc (r : CheckedByteResult) : List UInt8 :=
  Bytes.pairBytes Bytes.byteArrayBytes
    (Bytes.pairBytes Bytes.natBytes Bytes.natBytes) r.toTuple

theorem bytesEnc_prefixFree : Bytes.PrefixFree bytesEnc :=
  (Bytes.pairBytes_prefixFree Bytes.byteArrayBytes_prefixFree
    (Bytes.pairBytes_prefixFree Bytes.natBytes_prefixFree
      Bytes.natBytes_prefixFree)).comp toTuple_injective

/-- The frozen canonical schema of a checked byte result. -/
def identitySchema : CanonicalSchema CheckedByteResult :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.generic
    (TypeTag.leaf "Universal.CheckedByteResult".toUTF8.toList)
    (TypeTag.leaf_size_pos _)
    bytesEnc bytesEnc_prefixFree

end CheckedByteResult

/-! ## The five partition results (SPEC 10.4) -/

/--
  **SPEC 10.4**, `PartitionResult`, with exactly five constructors and exactly
  the proof fields SPEC gives them.

  `dominated` is the one that carries the undischarged content: its
  `memberLowerBound` field is the universal lower bound of UOR-GNAF section
  19.3.  The constructor is present because a verifier must be able to accept
  such evidence; **this repository builds no inhabitant of it for the release
  scope**, and none may be added without a proof.
-/
inductive PartitionResult {scope : PartitionScope P} (parent : PartitionBody scope)
  /-- The cell was enumerated exactly. -/
  | exhausted
      (members : CanonicalList CheckedByteResult)
      (exact : ∀ bytes, parent.Denotes bytes ↔
        ∃ member ∈ members.elements, member.bytes = bytes)
  /-- Every conforming member of the cell scores at least `lowerBound`, and the
  incumbent already beats that bound. -/
  | dominated
      (lowerBound : Nat)
      (memberLowerBound : ∀ bytes : ByteArray,
        parent.Denotes bytes →
        ProfileValid P bytes →
        SemanticCorrect scope.setting bytes →
        SemanticWithinResources scope.setting bytes →
        ∀ evaluation : SystemEvaluation scope.setting bytes,
          SystemEvaluationRel scope.setting scope.decider bytes evaluation ∧
          lowerBound ≤ scope.objective.score evaluation.cost)
      (strict : scope.baselineScore < lowerBound)
  /-- Nothing in the cell is even profile valid. -/
  | empty
      (proof : ∀ bytes : ByteArray, parent.Denotes bytes → ¬ ProfileValid P bytes)
  /-- The cell was refined into strictly lower-ranked children that cover it
  and do not overlap. -/
  | split
      (children : NonemptyCanonicalList (PartitionBody scope))
      (cover : ∀ bytes : ByteArray, parent.Denotes bytes →
        ∃ child ∈ children.elements, child.Denotes bytes)
      (disjoint : PairwiseDisjointDenotations children)
      (decreases : ∀ child ∈ children.elements, child.rank < parent.rank)
  /-- The cell was not discharged.  This constructor blocks release. -/
  | incomplete (gap : Foundation.CoverageGap)

namespace PartitionResult

variable {scope : PartitionScope P} {parent : PartitionBody scope}

/-- The `split` children of a result, when it is a split. -/
def children? (r : PartitionResult parent) :
    Option (NonemptyCanonicalList (PartitionBody scope)) :=
  match r with
  | split children _ _ _ => some children
  | _ => none

/-- The coverage gap of a result, when it is `incomplete`. -/
def gap? (r : PartitionResult parent) : Option Foundation.CoverageGap :=
  match r with
  | incomplete gap => some gap
  | _ => none

/-- **SPEC 10.4**: "Only the first four constructors may occur in a sealed
global certificate."  This is the decidable predicate that says so. -/
def isSealable (r : PartitionResult parent) : Bool :=
  match r with
  | exhausted _ _ => true
  | dominated _ _ _ => true
  | empty _ => true
  | split _ _ _ _ => true
  | incomplete _ => false

@[simp] theorem isSealable_exhausted (m : CanonicalList CheckedByteResult) (e) :
    (exhausted (parent := parent) m e).isSealable = true := rfl

@[simp] theorem isSealable_dominated (b : Nat) (m) (s) :
    (dominated (parent := parent) b m s).isSealable = true := rfl

@[simp] theorem isSealable_empty (p) : (empty (parent := parent) p).isSealable = true := rfl

@[simp] theorem isSealable_split (c) (co) (d) (dec) :
    (split (parent := parent) c co d dec).isSealable = true := rfl

@[simp] theorem isSealable_incomplete (gap : Foundation.CoverageGap) :
    (incomplete (parent := parent) gap).isSealable = false := rfl

@[simp] theorem gap?_incomplete (gap : Foundation.CoverageGap) :
    (incomplete (parent := parent) gap).gap? = some gap := rfl

/-- Sealability is exactly the absence of a coverage gap. -/
theorem isSealable_iff_gap?_none (r : PartitionResult parent) :
    r.isSealable = true ↔ r.gap? = none := by
  cases r <;> simp [isSealable, gap?]

/-- **Only the first four constructors may occur in a sealed certificate**, in
the form a verifier uses it: a sealable result is one of the four, with its
proof fields intact. -/
theorem sealable_is_one_of_four (r : PartitionResult parent)
    (h : r.isSealable = true) :
    (∃ members exact, r = exhausted members exact) ∨
    (∃ lowerBound memberLowerBound strict, r = dominated lowerBound memberLowerBound strict) ∨
    (∃ proof, r = empty proof) ∨
    (∃ children cover disjoint decreases, r = split children cover disjoint decreases) := by
  cases r with
  | exhausted members exact => exact Or.inl ⟨members, exact, rfl⟩
  | dominated b m s => exact Or.inr (Or.inl ⟨b, m, s, rfl⟩)
  | empty p => exact Or.inr (Or.inr (Or.inl ⟨p, rfl⟩))
  | split c co d dec => exact Or.inr (Or.inr (Or.inr ⟨c, co, d, dec, rfl⟩))
  | incomplete gap => exact absurd h (by simp)

/-- An `incomplete` result is never sealable. -/
theorem not_isSealable_incomplete (gap : Foundation.CoverageGap) :
    (incomplete (parent := parent) gap).isSealable = false := rfl

/-! ### `incomplete` propagates to `SolveResult.incomplete` -/

/--
  **SPEC 10.4**: "`incomplete` propagates to `SolveResult.incomplete` and blocks
  release."  This is that propagation as a total function, not a comment: the
  sealed answer survives only when the result is one of the first four.
-/
def propagate {α : Type} (r : PartitionResult parent)
    (sealedAnswer : Foundation.SolveResult α) : Foundation.SolveResult α :=
  match r with
  | incomplete gap => .incomplete gap
  | _ => sealedAnswer

/-- The propagation really fires on `incomplete`. -/
@[simp] theorem propagate_incomplete {α : Type} (gap : Foundation.CoverageGap)
    (sealedAnswer : Foundation.SolveResult α) :
    propagate (incomplete (parent := parent) gap) sealedAnswer =
      .incomplete gap := rfl

/-- The propagation is transparent on the four sealable constructors. -/
theorem propagate_of_isSealable {α : Type} {r : PartitionResult parent}
    (h : r.isSealable = true) (sealedAnswer : Foundation.SolveResult α) :
    propagate r sealedAnswer = sealedAnswer := by
  cases r <;> first | rfl | exact absurd h (by simp)

/-- **`incomplete` blocks release**: whatever the sealed answer was, an
`incomplete` partition result makes the solve non-optimal. -/
theorem propagate_incomplete_not_optimal {α : Type}
    (gap : Foundation.CoverageGap) (sealedAnswer : Foundation.SolveResult α) :
    (propagate (incomplete (parent := parent) gap) sealedAnswer).isOptimal
      = false := rfl

/-- Propagation preserves an optimal answer exactly when the result is
sealable — the converse half, so that no `incomplete` can slip through. -/
theorem isOptimal_propagate_iff {α : Type} (r : PartitionResult parent)
    {v : α} (c : Foundation.GlobalCertificate v) :
    (propagate r (.optimal v c)).isOptimal = true ↔ r.isSealable = true := by
  cases r <;> simp [propagate, isSealable, Foundation.SolveResult.isOptimal]

end PartitionResult

/-! ### A whole cover: sealing or propagating

A sealed global certificate quotes a *family* of partition results, not one.
`PartitionOutcome` pairs a cell with its result so the family is a plain list,
and `sealOrPropagate` is the total function that seals only when every member
is one of SPEC's first four constructors. -/

/-- One cell together with its result. -/
structure PartitionOutcome (scope : PartitionScope P) where
  /-- The cell. -/
  parent : PartitionBody scope
  /-- Its result. -/
  result : PartitionResult parent

namespace PartitionOutcome

variable {scope : PartitionScope P}

/-- Every outcome of the family is one of SPEC's first four constructors. -/
def AllSealable (outcomes : List (PartitionOutcome scope)) : Prop :=
  ∀ o ∈ outcomes, o.result.isSealable = true

instance instDecidableAllSealable (outcomes : List (PartitionOutcome scope)) :
    Decidable (AllSealable outcomes) := by
  unfold AllSealable; infer_instance

/-- The first coverage gap the family reports, if any. -/
def firstGap? (outcomes : List (PartitionOutcome scope)) :
    Option Foundation.CoverageGap :=
  match outcomes with
  | [] => none
  | o :: rest =>
      match o.result.gap? with
      | some gap => some gap
      | none => firstGap? rest

/-- **SPEC 10.4**, the propagation rule for a whole cover: the sealed answer
survives exactly when no cell reported a gap; otherwise the solve is
`incomplete` at the first gap. -/
def sealOrPropagate {α : Type} (outcomes : List (PartitionOutcome scope))
    (sealedAnswer : Foundation.SolveResult α) : Foundation.SolveResult α :=
  match firstGap? outcomes with
  | some gap => .incomplete gap
  | none => sealedAnswer

/-- No gap is reported exactly when every result is sealable. -/
theorem firstGap?_eq_none_iff (outcomes : List (PartitionOutcome scope)) :
    firstGap? outcomes = none ↔ AllSealable outcomes := by
  induction outcomes with
  | nil => simp [firstGap?, AllSealable]
  | cons o rest ih =>
      rw [firstGap?]
      cases hg : o.result.gap? with
      | some gap =>
          simp only [reduceCtorEq, false_iff]
          intro hall
          have := (PartitionResult.isSealable_iff_gap?_none o.result).mp
            (hall o (List.mem_cons_self ..))
          rw [this] at hg
          simp at hg
      | none =>
          rw [ih]
          constructor
          · intro hrest x hx
            rcases List.mem_cons.mp hx with rfl | hmem
            · exact (PartitionResult.isSealable_iff_gap?_none x.result).mpr hg
            · exact hrest x hmem
          · intro hall x hx
            exact hall x (List.mem_cons_of_mem _ hx)

/-- A fully sealable family passes the sealed answer through unchanged. -/
theorem sealOrPropagate_of_allSealable {α : Type}
    {outcomes : List (PartitionOutcome scope)} (h : AllSealable outcomes)
    (sealedAnswer : Foundation.SolveResult α) :
    sealOrPropagate outcomes sealedAnswer = sealedAnswer := by
  rw [sealOrPropagate, (firstGap?_eq_none_iff outcomes).mpr h]

/-- **A single `incomplete` cell blocks release.**  If any cell reports a gap,
the family's answer is `SolveResult.incomplete`, whatever the sealed answer
was. -/
theorem sealOrPropagate_incomplete {α : Type}
    {outcomes : List (PartitionOutcome scope)} (h : ¬ AllSealable outcomes)
    (sealedAnswer : Foundation.SolveResult α) :
    ∃ gap, sealOrPropagate outcomes sealedAnswer = .incomplete gap := by
  rw [sealOrPropagate]
  cases hg : firstGap? outcomes with
  | some gap => exact ⟨gap, rfl⟩
  | none => exact absurd ((firstGap?_eq_none_iff outcomes).mp hg) h

/-- The sharp form: an optimal answer survives the cover exactly when every
result is one of SPEC's first four constructors. -/
theorem isOptimal_sealOrPropagate_iff {α : Type}
    (outcomes : List (PartitionOutcome scope)) {v : α}
    (c : Foundation.GlobalCertificate v) :
    (sealOrPropagate outcomes (.optimal v c)).isOptimal = true ↔
      AllSealable outcomes := by
  rw [← firstGap?_eq_none_iff, sealOrPropagate]
  cases hg : firstGap? outcomes with
  | some gap => simp [Foundation.SolveResult.isOptimal]
  | none => simp [Foundation.SolveResult.isOptimal]

end PartitionOutcome

/-! ## The split rank decrease makes recursive coverage well founded

This is the point of the file.  A refinement strategy answers every cell; the
`split` constructor's `decreases` field is the *only* thing that stops the
refinement from running forever.  Below: the explicit step relation, its
explicit natural measure, the `WellFounded` theorem, the recursive cover
function defined by that measure, and the theorem that its leaves cover
everything the root denotes. -/

/-- A refinement strategy: a total answer for every cell of the scope. -/
abbrev PartitionStrategy (scope : PartitionScope P) :=
  (p : PartitionBody scope) → PartitionResult p

variable {scope : PartitionScope P}

/-- `SplitStep strategy child parent` holds when the strategy splits `parent`
and `child` is one of the children it produced. -/
def SplitStep (strategy : PartitionStrategy scope)
    (child parent : PartitionBody scope) : Prop :=
  ∃ (children : NonemptyCanonicalList (PartitionBody scope))
    (cover : ∀ bytes : ByteArray, parent.Denotes bytes →
      ∃ c ∈ children.elements, c.Denotes bytes)
    (disjoint : PairwiseDisjointDenotations children)
    (decreases : ∀ c ∈ children.elements, c.rank < parent.rank),
      strategy parent = .split children cover disjoint decreases ∧
      child ∈ children.elements

/-- **The rank strictly decreases along every split step.**  This is SPEC
10.4's `decreases` field, read as a property of the step relation. -/
theorem splitStep_rank_lt {strategy : PartitionStrategy scope}
    {child parent : PartitionBody scope} (h : SplitStep strategy child parent) :
    child.rank < parent.rank := by
  obtain ⟨_children, _cover, _disjoint, decreases, _heq, hmem⟩ := h
  exact decreases child hmem

/--
  **SPEC 10.4**: "The strict rank decrease makes recursive coverage well
  founded."  Proved, with `rank` as the explicit natural measure.
-/
theorem splitStep_wellFounded (strategy : PartitionStrategy scope) :
    WellFounded (SplitStep strategy) :=
  Foundation.Termination.wellFounded_of_measure PartitionBody.rank (SplitStep strategy)
    (fun _ _ h => splitStep_rank_lt h)

/-- The bundled measured step relation of `Foundation/Termination.lean`. -/
def splitMeasured (strategy : PartitionStrategy scope) :
    Foundation.Termination.Measured (PartitionBody scope) where
  measure := PartitionBody.rank
  step := SplitStep strategy
  decreasing := fun _ _ h => splitStep_rank_lt h

/-- No cell is its own split child: an immediate corollary of the measure. -/
theorem splitStep_irrefl (strategy : PartitionStrategy scope)
    (p : PartitionBody scope) : ¬ SplitStep strategy p p :=
  fun h => Nat.lt_irrefl _ (splitStep_rank_lt h)

/--
  Recursive coverage: refine the cell until every branch reaches a non-`split`
  result, and return the leaves.  Terminating by the explicit measure `rank`,
  discharged from the `split` constructor's own `decreases` field.
-/
def coverLeaves (strategy : PartitionStrategy scope) (p : PartitionBody scope) :
    List (PartitionBody scope) :=
  match strategy p with
  | .split children _ _ hdec =>
      children.elements.attach.flatMap
        (fun c => coverLeaves strategy c.val)
  | _ => [p]
termination_by p.rank
decreasing_by exact hdec c.val c.property

/-- Unfolding at a `split`. -/
theorem coverLeaves_split {strategy : PartitionStrategy scope}
    {p : PartitionBody scope}
    {children : NonemptyCanonicalList (PartitionBody scope)}
    {cover : ∀ bytes : ByteArray, p.Denotes bytes →
      ∃ c ∈ children.elements, c.Denotes bytes}
    {disjoint : PairwiseDisjointDenotations children}
    {decreases : ∀ c ∈ children.elements, c.rank < p.rank}
    (h : strategy p = .split children cover disjoint decreases) :
    coverLeaves strategy p =
      children.elements.attach.flatMap (fun c => coverLeaves strategy c.val) := by
  rw [coverLeaves, h]

/-- Unfolding at a leaf. -/
theorem coverLeaves_leaf {strategy : PartitionStrategy scope}
    {p : PartitionBody scope} (h : (strategy p).children? = none) :
    coverLeaves strategy p = [p] := by
  rw [coverLeaves]
  cases hr : strategy p with
  | exhausted _ _ => rfl
  | dominated _ _ _ => rfl
  | empty _ => rfl
  | incomplete _ => rfl
  | split c co d dec =>
      rw [hr] at h
      exact absurd h (by simp [PartitionResult.children?])

/-- Auxiliary: coverage below a rank bound. -/
theorem coverLeaves_covers_aux (strategy : PartitionStrategy scope) :
    ∀ (n : Nat) (p : PartitionBody scope), p.rank ≤ n →
      ∀ bytes : ByteArray, p.Denotes bytes →
        ∃ q ∈ coverLeaves strategy p, q.Denotes bytes := by
  intro n
  induction n with
  | zero =>
      intro p hrank bytes hden
      cases hr : strategy p with
      | split children cover disjoint decreases =>
          exact absurd (Nat.lt_of_lt_of_le
            (decreases children.head (by simp [Foundation.NonemptyCanonicalFrontier.elements]))
            hrank) (Nat.not_lt_zero _)
      | exhausted _ _ =>
          exact ⟨p, by rw [coverLeaves_leaf (by rw [hr]; rfl)]; simp, hden⟩
      | dominated _ _ _ =>
          exact ⟨p, by rw [coverLeaves_leaf (by rw [hr]; rfl)]; simp, hden⟩
      | empty _ =>
          exact ⟨p, by rw [coverLeaves_leaf (by rw [hr]; rfl)]; simp, hden⟩
      | incomplete _ =>
          exact ⟨p, by rw [coverLeaves_leaf (by rw [hr]; rfl)]; simp, hden⟩
  | succ n ih =>
      intro p hrank bytes hden
      cases hr : strategy p with
      | split children cover disjoint decreases =>
          obtain ⟨child, hchild, hcden⟩ := cover bytes hden
          have hcrank : child.rank ≤ n :=
            Nat.le_of_lt_succ (Nat.lt_of_lt_of_le (decreases child hchild) hrank)
          obtain ⟨q, hq, hqden⟩ := ih child hcrank bytes hcden
          refine ⟨q, ?_, hqden⟩
          rw [coverLeaves_split hr]
          exact List.mem_flatMap.mpr ⟨⟨child, hchild⟩, List.mem_attach _ _, hq⟩
      | exhausted _ _ =>
          exact ⟨p, by rw [coverLeaves_leaf (by rw [hr]; rfl)]; simp, hden⟩
      | dominated _ _ _ =>
          exact ⟨p, by rw [coverLeaves_leaf (by rw [hr]; rfl)]; simp, hden⟩
      | empty _ =>
          exact ⟨p, by rw [coverLeaves_leaf (by rw [hr]; rfl)]; simp, hden⟩
      | incomplete _ =>
          exact ⟨p, by rw [coverLeaves_leaf (by rw [hr]; rfl)]; simp, hden⟩

/--
  **Recursive coverage is sound.**  Refinement loses nothing: every byte string
  the root cell denotes is denoted by one of the leaves the terminating
  recursion returns.

  Note the shape of the statement: it is *conditional* on `p.Denotes bytes`.
  It does not say the root denotes every competitor — see
  `coverLeaves_covers_scope`.
-/
theorem coverLeaves_covers (strategy : PartitionStrategy scope)
    (p : PartitionBody scope) (bytes : ByteArray) (h : p.Denotes bytes) :
    ∃ q ∈ coverLeaves strategy p, q.Denotes bytes :=
  coverLeaves_covers_aux strategy p.rank p (Nat.le_refl _) bytes h

/-- Every leaf the recursion returns is a real cell of the same scope, and its
denotation is contained in nothing wider than its own size bound. -/
theorem coverLeaves_leaf_size {strategy : PartitionStrategy scope}
    {p q : PartitionBody scope} (_hq : q ∈ coverLeaves strategy p)
    {bytes : ByteArray} (h : q.Denotes bytes) :
    bytes.size ≤ q.bytePrefix.size + q.suffixLengthBound :=
  PartitionBody.size_le_of_denotes h

/-! ### Anti-vacuity: what recursive coverage does NOT establish

`coverLeaves_covers` is an implication out of `p.Denotes bytes`.  A cell that
denotes nothing satisfies it vacuously, and a refinement of a root that misses
most of `ByteArray` still satisfies it.  The lemma below makes that limit
machine checked rather than a remark: the cover function is blind to every byte
string the root does not denote. -/

/--
  **Scope of recursive coverage.**  There is a scope, a cell and a byte string
  such that the cell's leaves cover it *not at all* — because the cell denotes
  nothing containing it.  Hence `coverLeaves_covers` cannot be read as a
  statement about all competitor bytes.

  Concretely: a cell with suffix bound `0` and an empty prefix denotes only the
  empty byte string, so it denotes no nonempty competitor, however the strategy
  refines it.
-/
theorem coverLeaves_covers_scope (_strategy : PartitionStrategy scope)
    (p : PartitionBody scope)
    (hprefix : p.bytePrefix.size = 0) (hbound : p.suffixLengthBound = 0)
    (bytes : ByteArray) (hne : 0 < bytes.size) : ¬ p.Denotes bytes := by
  intro h
  have := PartitionBody.size_le_of_denotes h
  omega

/-! ### Anti-vacuity: `exhausted` says nothing about recorded outcomes

SPEC 10.4's `exact` field of `exhausted` equates *denotation* with *membership
by byte string*.  It never mentions `CheckedByteResult.outcome`.  A reader could
mistake the recorded outcomes for verified ones; the theorem below proves they
are not, by showing the obligation is invariant under arbitrarily rewriting
every outcome. -/

/-- **`exhausted` is blind to recorded outcomes.**  Rewriting every member's
recorded outcome leaves SPEC's `exact` obligation exactly as true, or as false,
as it was.  A recorded outcome is therefore data, never evidence. -/
theorem exhausted_exact_blind_to_outcome {parent : PartitionBody scope}
    (members : List CheckedByteResult)
    (f : CheckedByteResult → ByteCheckOutcome) :
    (∀ bytes : ByteArray, parent.Denotes bytes ↔
        ∃ member ∈ members, member.bytes = bytes) ↔
      (∀ bytes : ByteArray, parent.Denotes bytes ↔
        ∃ member ∈ members.map (fun m => (⟨m.bytes, f m⟩ : CheckedByteResult)),
          member.bytes = bytes) := by
  have hmem : ∀ bytes : ByteArray,
      (∃ member ∈ members, member.bytes = bytes) ↔
        ∃ member ∈ members.map (fun m => (⟨m.bytes, f m⟩ : CheckedByteResult)),
          member.bytes = bytes := by
    intro bytes
    constructor
    · rintro ⟨m, hm, rfl⟩
      exact ⟨⟨m.bytes, f m⟩, List.mem_map_of_mem hm, rfl⟩
    · rintro ⟨m, hm, hb⟩
      obtain ⟨m', hm', rfl⟩ := List.mem_map.mp hm
      exact ⟨m', hm', hb⟩
  constructor
  · intro h bytes; exact (h bytes).trans (hmem bytes)
  · intro h bytes; exact (h bytes).trans (hmem bytes).symm

end Universal

end WasmGemmGnaf
