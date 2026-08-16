/-
  `Universal.Correct`: the evaluator-side correctness predicate.
  Normative source: SPEC.md section 10.1.

  "`Correct` includes execution existence through `observations.Nonempty`; it
  cannot be vacuously true for an empty run relation."

  Dependency firewall (SPEC §10.1): this file imports only
  `Universal/Competitor.lean`.  It does not import `GNAF`, `Atlas`, `Artifact`,
  `Universal/LowerBound`, `Universal/Argmin`, or `Theorems`.

  Every declaration in this file is either a definition or a proved theorem.
-/
import WasmGemmGnaf.Universal.Competitor

set_option autoImplicit false

namespace WasmGemmGnaf.Universal

variable {P : Wasm.Profile}

/-- The nonemptiness conjunct of `Correct` is *always* discharged: the costed
observation list of an `InputEvaluation` is a `NonemptyCanonicalFrontier`, so it
carries at least one execution by construction.  `Correct` therefore cannot be
satisfied by an empty run relation. -/
theorem observations_ne_nil {S : Setting P} {module : Wasm.Module}
    {raw : Gemm.RawInvocation P} (ie : InputEvaluation S module raw) :
    ie.observations.elements ≠ [] :=
  Foundation.NonemptyCanonicalFrontier.elements_ne_nil _

/-- Every input evaluation exhibits at least one costed execution. -/
theorem exists_observation {S : Setting P} {module : Wasm.Module}
    {raw : Gemm.RawInvocation P} (ie : InputEvaluation S module raw) :
    ∃ o : CostedExecutionObservation S.semantics ie.initial,
      o ∈ ie.observations.elements :=
  ⟨ie.observations.head, by simp [Foundation.NonemptyCanonicalFrontier.elements]⟩

variable [Foundation.Fintype (Gemm.RawInvocation P)]

/-- **SPEC §10.1**, `Universal.Correct`.

The quantifier is over *every* raw invocation and *every* recorded costed
observation of that invocation.  A trap, uncaught exception, stuck state or
initialization failure appears as a recorded observation that the problem's
reference relation rejects, so it falsifies this predicate. -/
def Correct {S : Setting P} {bytes : ByteArray}
    (evaluation : SystemEvaluation S bytes) : Prop :=
  ∀ raw : Gemm.RawInvocation P,
    (evaluation.perInput raw).observations.elements ≠ [] ∧
    ∀ observation ∈ (evaluation.perInput raw).observations.elements,
      S.problem.Accepts raw observation.observation

/-- Because nonemptiness is automatic, `Correct` is *exactly* the acceptance
clause.  This documents that the predicate carries no hidden strength beyond
the reference relation. -/
theorem correct_iff_accepts {S : Setting P} {bytes : ByteArray}
    (evaluation : SystemEvaluation S bytes) :
    Correct evaluation ↔
      ∀ raw : Gemm.RawInvocation P,
        ∀ observation ∈ (evaluation.perInput raw).observations.elements,
          S.problem.Accepts raw observation.observation := by
  constructor
  · intro h raw o ho
    exact (h raw).2 o ho
  · intro h raw
    exact ⟨observations_ne_nil _, h raw⟩

/-- A correct evaluation accepts at least one observation on every input: the
conclusion is never vacuous. -/
theorem correct_exists_accepted {S : Setting P} {bytes : ByteArray}
    {evaluation : SystemEvaluation S bytes} (h : Correct evaluation)
    (raw : Gemm.RawInvocation P) :
    ∃ o : CostedExecutionObservation S.semantics (evaluation.perInput raw).initial,
      o ∈ (evaluation.perInput raw).observations.elements ∧
      S.problem.Accepts raw o.observation := by
  obtain ⟨o, ho⟩ := exists_observation (evaluation.perInput raw)
  exact ⟨o, ho, (h raw).2 o ho⟩

/-- Every accepted observation of a correct evaluation is a genuine finite
execution of the initial configuration, not a bookkeeping entry. -/
theorem correct_observation_is_execution {S : Setting P} {bytes : ByteArray}
    (evaluation : SystemEvaluation S bytes) (raw : Gemm.RawInvocation P)
    {o : CostedExecutionObservation S.semantics (evaluation.perInput raw).initial}
    (_ho : o ∈ (evaluation.perInput raw).observations.elements) :
    Wasm.FiniteExecution (evaluation.perInput raw).initial o.observation :=
  o.execution

/-- `Correct` is closed under the pointwise reading: it is a statement about
every raw invocation separately, with no cross-input state.  (SPEC §10.1: "No
mutable state carries from one raw invocation to the next in `ArtifactGlobal`.") -/
theorem correct_pointwise {S : Setting P} {bytes : ByteArray}
    {evaluation : SystemEvaluation S bytes} (h : Correct evaluation)
    (raw : Gemm.RawInvocation P) :
    (evaluation.perInput raw).observations.elements ≠ [] ∧
    ∀ observation ∈ (evaluation.perInput raw).observations.elements,
      S.problem.Accepts raw observation.observation :=
  h raw

end WasmGemmGnaf.Universal
