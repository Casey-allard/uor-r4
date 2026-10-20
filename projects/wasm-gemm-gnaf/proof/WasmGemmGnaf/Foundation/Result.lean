import WasmGemmGnaf.Foundation.Identity
import WasmGemmGnaf.Foundation.Order
set_option autoImplicit false

/-!
# Foundation: the result algebra (SPEC §6.1)

Every result type here is *closed* and *payload bearing*: no outcome is a bare
tag, and every failure names the structural object it is about.

## The `optimal` constructor

SPEC §6.1: "No constructor named `optimal` may be created without an exact
global certificate whose verifier reconstructs the final proposition."

`GlobalCertificate value` is therefore an index-carrying structure holding only
the *verifiable syntactic* components of such a certificate — schema, scope
identities, the candidate identity together with a proof that it really is the
identity of `value`, the replayable partition/coverage roots, and the scores.

It deliberately carries **no** field of the form `IsGlobalOptimal value`.  The
project rules forbid storing a theorem's conclusion as an unproved structure
field, and the release-level optimality proposition is not available in the
Foundation layer.  Consequently this file provides no smart constructor that
manufactures a certificate: a `GlobalCertificate` can only be assembled by
supplying every syntactic component, and `candidate_eq` — the one link that
cannot be faked — must be discharged by the caller.
-/

namespace WasmGemmGnaf.Foundation

/-! ## Failure payloads -/

/-- The set of canonical objects a check could not resolve. -/
structure MissingSet where
  missing : List CanonicalObjectId
  deriving DecidableEq, Inhabited

/-- An exhausted resource, with its limit and the demand that exceeded it. -/
structure ResourceReport where
  resource : CanonicalObjectId
  limit : Nat
  demanded : Nat
  deriving DecidableEq

/-- An internal failure, located at a canonical site. -/
structure FailureReport where
  site : CanonicalObjectId
  code : Nat
  context : List CanonicalObjectId
  deriving DecidableEq

/-- A hole in a coverage argument: which domain, up to which score bound, and
which canonical objects were left uncovered. -/
structure CoverageGap where
  domain : CanonicalDomainTag
  scoreBound : Nat
  coveredRoot : ByteArray
  uncovered : List CanonicalObjectId
  deriving DecidableEq

/-! ## Checks -/

/-- The closed payload-bearing result of a check (SPEC §6.1). -/
inductive CheckResult (α ε : Type)
  | complete (value : α)
  | rejected (error : ε)
  | unresolved (missing : MissingSet)
  | unsupported (feature : FeatureId)
  | resourceExhausted (report : ResourceReport)
  | internalFailure (report : FailureReport)

namespace CheckResult

variable {α β ε : Type}

/-- The successfully checked value, when there is one. -/
def value? : CheckResult α ε → Option α
  | complete v => some v
  | _ => none

def isComplete (r : CheckResult α ε) : Bool :=
  match r with
  | complete _ => true
  | _ => false

/-- Transport the payload of a completed check. -/
def map (f : α → β) : CheckResult α ε → CheckResult β ε
  | complete v => complete (f v)
  | rejected e => rejected e
  | unresolved m => unresolved m
  | unsupported x => unsupported x
  | resourceExhausted r => resourceExhausted r
  | internalFailure r => internalFailure r

/-- Sequence two checks, propagating every non-complete outcome unchanged. -/
def bind (r : CheckResult α ε) (f : α → CheckResult β ε) : CheckResult β ε :=
  match r with
  | complete v => f v
  | rejected e => rejected e
  | unresolved m => unresolved m
  | unsupported x => unsupported x
  | resourceExhausted rep => resourceExhausted rep
  | internalFailure rep => internalFailure rep

@[simp] theorem value?_complete (v : α) : (complete v : CheckResult α ε).value? = some v := rfl

@[simp] theorem isComplete_complete (v : α) :
    (complete v : CheckResult α ε).isComplete = true := rfl

@[simp] theorem map_complete (f : α → β) (v : α) :
    (complete v : CheckResult α ε).map f = complete (f v) := rfl

@[simp] theorem bind_complete (v : α) (f : α → CheckResult β ε) :
    (complete v : CheckResult α ε).bind f = f v := rfl

theorem map_id (r : CheckResult α ε) : r.map id = r := by
  cases r <;> rfl

theorem map_map (f : α → β) {γ : Type} (g : β → γ) (r : CheckResult α ε) :
    (r.map f).map g = r.map (fun a => g (f a)) := by
  cases r <;> rfl

/-- `isComplete` is exactly the presence of a value. -/
theorem isComplete_iff (r : CheckResult α ε) :
    r.isComplete = true ↔ ∃ v, r.value? = some v := by
  cases r <;> simp [isComplete, value?]

end CheckResult

/-! ## Global certificates -/

/-- The verifiable syntactic components of a global-optimality certificate for
`value` (SPEC §6.1).

Every field is data a verifier can recompute or replay.  `candidate_eq` is the
*only* proof obligation, and it is a decidable structural equation, not the
optimality proposition. -/
structure GlobalCertificate {α : Type} (value : α) where
  /-- The frozen schema under which `value` is named. -/
  schema : CanonicalSchema α
  /-- Scope identities the certificate is stated against. -/
  profile : ProfileId
  problem : ProblemId
  objective : ObjectiveId
  /-- The canonical identity of the candidate. -/
  candidate : CanonicalObjectId
  /-- The candidate really is `value`, structurally. -/
  candidate_eq : candidate = CanonicalObjectId.ofTyped (Identity schema value)
  /-- The candidate's score under `objective`. -/
  score : Nat
  /-- The lower bound the coverage argument establishes. -/
  lowerBound : Nat
  /-- Root of the replayable partition of the competitor domain. -/
  partitionRoot : ByteArray
  /-- Root of the replayable coverage enumeration. -/
  coverageRoot : ByteArray
  /-- The enumeration bound induced by the objective's `boundOfScore`. -/
  enumerationBound : Nat

namespace GlobalCertificate

/-- A certificate is genuinely about its value: the candidate identity, together
with the schema, determines the value it certifies. -/
theorem value_of_candidate {α : Type} {v w : α}
    (c : GlobalCertificate v) (d : GlobalCertificate w)
    (hs : c.schema = d.schema) (hc : c.candidate = d.candidate) : v = w := by
  have h : CanonicalObjectId.ofTyped (Identity c.schema v) =
      CanonicalObjectId.ofTyped (Identity d.schema w) := by
    rw [← c.candidate_eq, ← d.candidate_eq, hc]
  rw [hs] at h
  exact (CanonicalObjectId.ofTyped_Identity_eq_iff d.schema).mp h

/-- The certificate's candidate identity is recomputable from `value`. -/
theorem candidate_recomputable {α : Type} {v : α} (c : GlobalCertificate v) :
    c.candidate = CanonicalObjectId.ofTyped (Identity c.schema v) :=
  c.candidate_eq

end GlobalCertificate

/-! ## Frontiers and negative certificates -/

/-- A nonempty frontier presented in the canonical byte order. -/
structure NonemptyCanonicalFrontier (α : Type) where
  schema : CanonicalSchema α
  head : α
  rest : List α
  ordered : CanonicalSorted ((head :: rest).map schema.encode)

namespace NonemptyCanonicalFrontier

variable {α : Type}

/-- The frontier's elements, in canonical order. -/
def elements (f : NonemptyCanonicalFrontier α) : List α := f.head :: f.rest

theorem elements_ne_nil (f : NonemptyCanonicalFrontier α) : f.elements ≠ [] := by
  simp [elements]

/-- Every one-element frontier is canonically ordered. -/
def singleton (schema : CanonicalSchema α) (a : α) : NonemptyCanonicalFrontier α where
  schema := schema
  head := a
  rest := []
  ordered := by simp

@[simp] theorem elements_singleton (schema : CanonicalSchema α) (a : α) :
    (singleton schema a).elements = [a] := rfl

end NonemptyCanonicalFrontier

/-- Replayable evidence that the feasible set is empty. -/
structure InfeasibilityCertificate where
  profile : ProfileId
  problem : ProblemId
  objective : ObjectiveId
  /-- The score bound up to which the domain was exhausted. -/
  searchBound : Nat
  /-- Root of the replayable exhaustion enumeration. -/
  exhaustedDomainRoot : ByteArray
  /-- Identities of the refuted candidates. -/
  refuted : List CanonicalObjectId
  deriving DecidableEq

/-- Replayable evidence that the infimum is approached but never attained. -/
structure NonattainmentCertificate where
  profile : ProfileId
  problem : ProblemId
  objective : ObjectiveId
  /-- The infimum the scores approach. -/
  infimum : Nat
  /-- Root of the replayable strictly descending witness sequence. -/
  descendingWitnessRoot : ByteArray
  /-- Scores of the witnesses, in the order the verifier replays them. -/
  witnessScores : List Nat
  deriving DecidableEq

/-! ## Solves -/

/-- The closed payload-bearing result of a solve (SPEC §6.1).

There is intentionally no smart constructor producing `optimal`: the caller must
supply a complete `GlobalCertificate`. -/
inductive SolveResult (α : Type)
  | optimal (value : α) (certificate : GlobalCertificate value)
  | pareto (frontier : NonemptyCanonicalFrontier α)
  | infeasible (proof : InfeasibilityCertificate)
  | nonattained (proof : NonattainmentCertificate)
  | incomplete (coverage : CoverageGap)
  | unsealed (head : StateIdentity)
  | resourceExhausted (report : ResourceReport)
  | internalFailure (report : FailureReport)

namespace SolveResult

variable {α : Type}

/-- The optimal value, when the solve produced one. -/
def optimalValue? : SolveResult α → Option α
  | optimal v _ => some v
  | _ => none

def isOptimal (r : SolveResult α) : Bool :=
  match r with
  | optimal _ _ => true
  | _ => false

@[simp] theorem optimalValue?_optimal (v : α) (c : GlobalCertificate v) :
    (optimal v c).optimalValue? = some v := rfl

@[simp] theorem isOptimal_optimal (v : α) (c : GlobalCertificate v) :
    (optimal v c).isOptimal = true := rfl

theorem isOptimal_iff (r : SolveResult α) :
    r.isOptimal = true ↔ ∃ v, r.optimalValue? = some v := by
  cases r <;> simp [isOptimal, optimalValue?]

/-- An `optimal` outcome always exposes its certificate's candidate identity,
which the verifier recomputes from the value. -/
theorem optimal_candidate_recomputable {v : α} (c : GlobalCertificate v) :
    c.candidate = CanonicalObjectId.ofTyped (Identity c.schema v) :=
  c.candidate_eq

end SolveResult

end WasmGemmGnaf.Foundation
