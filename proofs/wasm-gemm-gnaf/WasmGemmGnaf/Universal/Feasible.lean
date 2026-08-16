/-
  `Universal.Feasible`: the evaluator-side resource predicate.
  Normative source: SPEC.md section 10.1.

  "Resource feasibility therefore measures every bounded terminal execution
  independently of semantic correctness."

  Dependency firewall (SPEC §10.1): this file imports only
  `Universal/Competitor.lean`.  It does not import `GNAF`, `Atlas`, `Artifact`,
  `Universal/LowerBound`, `Universal/Argmin`, or `Theorems`.

  Every declaration in this file is either a definition or a proved theorem.
-/
import WasmGemmGnaf.Universal.Competitor
import WasmGemmGnaf.Universal.Correct

set_option autoImplicit false

namespace WasmGemmGnaf.Universal

variable {P : Wasm.Profile} [Foundation.Fintype (Gemm.RawInvocation P)]

/-- **SPEC §10.1**, `Universal.Feasible`.

The quantifier is over *every* raw invocation.  `resourceVector` is the
initialization charge composed with the componentwise maximum over every
recorded branch, so this bounds the worst permitted execution of the invocation,
not a chosen one. -/
def Feasible {S : Setting P} {bytes : ByteArray}
    (evaluation : SystemEvaluation S bytes) : Prop :=
  ∀ raw : Gemm.RawInvocation P,
    Cost.DynamicVector.ComponentwiseLE
      (evaluation.perInput raw).resourceVector S.problem.limit

/-- A feasible evaluation pays for initialization inside the limit: the
initialization charge of every input is bounded. -/
theorem feasible_initialization_le {S : Setting P} {bytes : ByteArray}
    {evaluation : SystemEvaluation S bytes} (h : Feasible evaluation)
    (raw : Gemm.RawInvocation P) :
    Cost.DynamicVector.ComponentwiseLE
      (evaluation.perInput raw).initialization.cost S.problem.limit := by
  have hexact := (evaluation.perInput raw).resourceExact
  refine Cost.DynamicVector.componentwiseLE_trans ?_ (h raw)
  rw [hexact]
  exact Cost.sequentialCompose_le_left _ _

/-- A feasible evaluation pays for the worst recorded branch inside the limit. -/
theorem feasible_branchMax_le {S : Setting P} {bytes : ByteArray}
    {evaluation : SystemEvaluation S bytes} (h : Feasible evaluation)
    (raw : Gemm.RawInvocation P) :
    Cost.DynamicVector.ComponentwiseLE
      (Cost.maxOverCosts
        ((evaluation.perInput raw).observations.elements.map (·.cost)))
      S.problem.limit := by
  have hexact := (evaluation.perInput raw).resourceExact
  refine Cost.DynamicVector.componentwiseLE_trans ?_ (h raw)
  rw [hexact]
  exact Cost.sequentialCompose_le_right _ _

/-- Every *individual* recorded branch of a feasible evaluation is inside the
limit.  This is the sense in which feasibility measures every bounded terminal
execution, not just an average or a selected one. -/
theorem feasible_observation_le {S : Setting P} {bytes : ByteArray}
    {evaluation : SystemEvaluation S bytes} (h : Feasible evaluation)
    (raw : Gemm.RawInvocation P)
    {o : CostedExecutionObservation S.semantics (evaluation.perInput raw).initial}
    (ho : o ∈ (evaluation.perInput raw).observations.elements) :
    Cost.DynamicVector.ComponentwiseLE o.cost S.problem.limit := by
  refine Cost.DynamicVector.componentwiseLE_trans ?_ (feasible_branchMax_le h raw)
  exact Cost.le_maxOverCosts (List.mem_map_of_mem ho)

/-- Initialization composed with any recorded branch stays inside the limit —
the exact shape SPEC §10.1's `SemanticWithinResourcesAt` charges. -/
theorem feasible_compose_le {S : Setting P} {bytes : ByteArray}
    {evaluation : SystemEvaluation S bytes} (h : Feasible evaluation)
    (raw : Gemm.RawInvocation P)
    {o : CostedExecutionObservation S.semantics (evaluation.perInput raw).initial}
    (ho : o ∈ (evaluation.perInput raw).observations.elements) :
    Cost.DynamicVector.ComponentwiseLE
      (Cost.sequentialCompose (evaluation.perInput raw).initialization.cost o.cost)
      S.problem.limit := by
  refine Cost.DynamicVector.componentwiseLE_trans ?_ (h raw)
  rw [(evaluation.perInput raw).resourceExact]
  exact Cost.sequentialCompose_mono_right _
    (Cost.le_maxOverCosts (List.mem_map_of_mem ho))

/-- `Feasible` is pointwise in the raw invocation: no cross-input amortization
is available. -/
theorem feasible_pointwise {S : Setting P} {bytes : ByteArray}
    {evaluation : SystemEvaluation S bytes} (h : Feasible evaluation)
    (raw : Gemm.RawInvocation P) :
    Cost.DynamicVector.ComponentwiseLE
      (evaluation.perInput raw).resourceVector S.problem.limit :=
  h raw

/-- `Correct` and `Feasible` are independent predicates over the same
evaluation: neither is stated in terms of the other.  Their conjunction is
recorded here so the two reflection equivalences of SPEC §10.1 have a single
name to target. -/
def CorrectAndFeasible {S : Setting P} {bytes : ByteArray}
    (evaluation : SystemEvaluation S bytes) : Prop :=
  Correct evaluation ∧ Feasible evaluation

/-- The conjunction projects to each component. -/
theorem correctAndFeasible_left {S : Setting P} {bytes : ByteArray}
    {evaluation : SystemEvaluation S bytes} (h : CorrectAndFeasible evaluation) :
    Correct evaluation := h.1

/-- The conjunction projects to each component. -/
theorem correctAndFeasible_right {S : Setting P} {bytes : ByteArray}
    {evaluation : SystemEvaluation S bytes} (h : CorrectAndFeasible evaluation) :
    Feasible evaluation := h.2

end WasmGemmGnaf.Universal
