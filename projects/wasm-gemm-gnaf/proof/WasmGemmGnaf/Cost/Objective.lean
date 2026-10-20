/-
  Objective bodies: the first-order, canonically encodable weight vector and the
  weighted evaluation it denotes.
  Normative source: SPEC.md sections 9.2 - 9.3.

  Every declaration in this file is proved. Nothing is assumed.
-/
import WasmGemmGnaf.Cost.Vector

set_option autoImplicit false

namespace WasmGemmGnaf.Cost

open ArtifactCoordinate

/-! ## Auxiliary list lemmas -/

/-- Auxiliary: a member of a `Nat` list never exceeds its sum. -/
theorem le_sum_of_mem : ∀ {l : List Nat} {x : Nat}, x ∈ l → x ≤ l.sum
  | _ :: t, x, h => by
      cases h with
      | head => exact Nat.le_add_right _ t.sum
      | tail _ hmem =>
          exact Nat.le_trans (le_sum_of_mem hmem) (Nat.le_add_left _ _)

/-- Auxiliary: pointwise-dominated `Nat` lists have ordered sums. -/
theorem sum_le_sum_of_pointwise {α : Type} (f g : α → Nat)
    (h : ∀ a, f a ≤ g a) :
    ∀ l : List α, (l.map f).sum ≤ (l.map g).sum
  | [] => Nat.le_refl 0
  | x :: t => by
      simp only [List.map_cons, List.sum_cons]
      exact Nat.add_le_add (h x) (sum_le_sum_of_pointwise f g h t)

/-! ## Coordinate-indexed weight records -/

/-- SPEC 9.3: the static half of an objective body's weight vector. -/
structure StaticCoordinateVector (α : Type) where
  moduleBytes     : α
  decodeSteps     : α
  validationSteps : α
  staticDataBytes : α
  deriving Repr, DecidableEq, Inhabited

/-- SPEC 9.3: the dynamic half of an objective body's weight vector. -/
structure DynamicCoordinateVector (α : Type) where
  instantiationSteps     : α
  dispatchSteps          : α
  preparationSteps       : α
  wasmRuleSteps          : α
  scalarOps              : α
  vectorLaneOps          : α
  bytesRead              : α
  bytesWritten           : α
  memoryGrowPages        : α
  tableElementsAllocated : α
  gcObjectsAllocated     : α
  gcBytesInitialized     : α
  peakStackValues        : α
  peakPages              : α
  peakGcLiveBytes        : α
  outputBytes            : α
  deriving Repr, DecidableEq, Inhabited

namespace StaticCoordinateVector

/-- The constant static weight vector. -/
def const {α : Type} (a : α) : StaticCoordinateVector α :=
  { moduleBytes := a, decodeSteps := a, validationSteps := a
    staticDataBytes := a }

end StaticCoordinateVector

namespace DynamicCoordinateVector

/-- The constant dynamic weight vector. -/
def const {α : Type} (a : α) : DynamicCoordinateVector α :=
  { instantiationSteps := a, dispatchSteps := a, preparationSteps := a
    wasmRuleSteps := a, scalarOps := a, vectorLaneOps := a
    bytesRead := a, bytesWritten := a, memoryGrowPages := a
    tableElementsAllocated := a, gcObjectsAllocated := a
    gcBytesInitialized := a, peakStackValues := a, peakPages := a
    peakGcLiveBytes := a, outputBytes := a }

end DynamicCoordinateVector

/-! ## Objective bodies -/

/-- SPEC 9.3, `Cost.ObjectiveBody`.  A first-order, canonically encodable
release input: no function and no proof is stored. -/
structure ObjectiveBody where
  version           : Nat
  staticWeights     : StaticCoordinateVector Nat
  dynamicSumWeights : DynamicCoordinateVector Nat
  dynamicMaxWeights : DynamicCoordinateVector Nat
  tieOrder          : TieOrderTag
  deriving Repr, DecidableEq, Inhabited

namespace ObjectiveBody

/-- The weight an objective body assigns to a charged coordinate. -/
def weight (body : ObjectiveBody) : ArtifactCoordinate → Nat
  | .staticModuleBytes => body.staticWeights.moduleBytes
  | .staticDecodeSteps => body.staticWeights.decodeSteps
  | .staticValidationSteps => body.staticWeights.validationSteps
  | .staticDataBytes => body.staticWeights.staticDataBytes
  | .sumInstantiationSteps => body.dynamicSumWeights.instantiationSteps
  | .sumDispatchSteps => body.dynamicSumWeights.dispatchSteps
  | .sumPreparationSteps => body.dynamicSumWeights.preparationSteps
  | .sumWasmRuleSteps => body.dynamicSumWeights.wasmRuleSteps
  | .sumScalarOps => body.dynamicSumWeights.scalarOps
  | .sumVectorLaneOps => body.dynamicSumWeights.vectorLaneOps
  | .sumBytesRead => body.dynamicSumWeights.bytesRead
  | .sumBytesWritten => body.dynamicSumWeights.bytesWritten
  | .sumMemoryGrowPages => body.dynamicSumWeights.memoryGrowPages
  | .sumTableElementsAllocated => body.dynamicSumWeights.tableElementsAllocated
  | .sumGcObjectsAllocated => body.dynamicSumWeights.gcObjectsAllocated
  | .sumGcBytesInitialized => body.dynamicSumWeights.gcBytesInitialized
  | .sumPeakStackValues => body.dynamicSumWeights.peakStackValues
  | .sumPeakPages => body.dynamicSumWeights.peakPages
  | .sumPeakGcLiveBytes => body.dynamicSumWeights.peakGcLiveBytes
  | .sumOutputBytes => body.dynamicSumWeights.outputBytes
  | .maxInstantiationSteps => body.dynamicMaxWeights.instantiationSteps
  | .maxDispatchSteps => body.dynamicMaxWeights.dispatchSteps
  | .maxPreparationSteps => body.dynamicMaxWeights.preparationSteps
  | .maxWasmRuleSteps => body.dynamicMaxWeights.wasmRuleSteps
  | .maxScalarOps => body.dynamicMaxWeights.scalarOps
  | .maxVectorLaneOps => body.dynamicMaxWeights.vectorLaneOps
  | .maxBytesRead => body.dynamicMaxWeights.bytesRead
  | .maxBytesWritten => body.dynamicMaxWeights.bytesWritten
  | .maxMemoryGrowPages => body.dynamicMaxWeights.memoryGrowPages
  | .maxTableElementsAllocated => body.dynamicMaxWeights.tableElementsAllocated
  | .maxGcObjectsAllocated => body.dynamicMaxWeights.gcObjectsAllocated
  | .maxGcBytesInitialized => body.dynamicMaxWeights.gcBytesInitialized
  | .maxPeakStackValues => body.dynamicMaxWeights.peakStackValues
  | .maxPeakPages => body.dynamicMaxWeights.peakPages
  | .maxPeakGcLiveBytes => body.dynamicMaxWeights.peakGcLiveBytes
  | .maxOutputBytes => body.dynamicMaxWeights.outputBytes

end ObjectiveBody

/-- SPEC 9.2: the score is the positive natural-weight sum of every unbounded
charged coordinate. -/
def evaluate (body : ObjectiveBody) (c : ArtifactVector) : Nat :=
  (ArtifactCoordinate.all.map (fun co => body.weight co * co.value c)).sum

/-- SPEC 9.3: an objective body is admissible only if every one of the 36
coordinate weights is positive.  This is a decidable syntactic side condition
on the body, not a stored conclusion. -/
def EveryCoordinateWeightPositive (body : ObjectiveBody) : Prop :=
  ∀ co : ArtifactCoordinate, 1 ≤ body.weight co

/-- Decision procedure for weight positivity. -/
def everyCoordinateWeightPositiveCheck (body : ObjectiveBody) : Bool :=
  ArtifactCoordinate.all.all (fun co => decide (1 ≤ body.weight co))

theorem everyCoordinateWeightPositive_iff (body : ObjectiveBody) :
    EveryCoordinateWeightPositive body ↔
      everyCoordinateWeightPositiveCheck body = true := by
  constructor
  · intro h
    rw [everyCoordinateWeightPositiveCheck, List.all_eq_true]
    intro co _
    exact decide_eq_true (h co)
  · intro h co
    rw [everyCoordinateWeightPositiveCheck, List.all_eq_true] at h
    exact of_decide_eq_true (h co (ArtifactCoordinate.mem_all co))

instance instDecidableEveryCoordinateWeightPositive (body : ObjectiveBody) :
    Decidable (EveryCoordinateWeightPositive body) :=
  decidable_of_iff _ (everyCoordinateWeightPositive_iff body).symm

/--
  SPEC 9.2: with every weight at least one, every single coordinate is bounded
  by the score.  This is what makes every score sublevel finite.
-/
theorem coordinate_le_evaluate {body : ObjectiveBody}
    (hpos : EveryCoordinateWeightPositive body) (c : ArtifactVector)
    (co : ArtifactCoordinate) : co.value c ≤ evaluate body c := by
  refine Nat.le_trans (Nat.le_mul_of_pos_left _ (hpos co)) ?_
  exact le_sum_of_mem
    (List.mem_map_of_mem (a := co) (ArtifactCoordinate.mem_all co))

/-- SPEC 9.3: weighted evaluation is monotone for the componentwise order. -/
theorem evaluate_monotone (body : ObjectiveBody) {a b : ArtifactVector}
    (h : ComponentwiseLE a b) : evaluate body a ≤ evaluate body b :=
  sum_le_sum_of_pointwise _ _
    (fun co => Nat.mul_le_mul_left (body.weight co) (h co))
    ArtifactCoordinate.all

/-- Evaluation of the zero vector is zero. -/
theorem sum_map_const_zero {α : Type} (l : List α) :
    (l.map (fun _ => 0)).sum = 0 := by
  induction l with
  | nil => rfl
  | cons _ t ih => simp only [List.map_cons, List.sum_cons, Nat.zero_add]; exact ih

theorem evaluate_zero (body : ObjectiveBody) :
    evaluate body ArtifactVector.zero = 0 := by
  simp only [evaluate, ArtifactCoordinate.value_zero, Nat.mul_zero]
  exact sum_map_const_zero _

/-! ## The canonical all-weights-one score

SPEC 9.2: "For the first release, every coordinate weight is exactly one."  The
score is then the plain natural-number sum of all 36 charged coordinates.  The
canonical `ObjectiveBody` realising it, and the proof that it is a
`ProperObjective`, are in `Cost/CanonicalObjective.lean`. -/

/-- SPEC 9.2 / 9.3: the canonical release score. -/
def CanonicalObjective.score (c : ArtifactVector) : Nat :=
  (ArtifactCoordinate.all.map (fun co => co.value c)).sum

/--
  SPEC 9.2, `coordinate_le_score`: for the canonical all-weights-one objective
  every coordinate value is bounded by the score.  This is the property that
  makes every score sublevel finite and supplies the all-Wasm coverage
  construction.
-/
theorem coordinate_le_score (c : ArtifactVector) :
    ∀ co : ArtifactCoordinate, co.value c ≤ CanonicalObjective.score c := by
  intro co
  exact le_sum_of_mem
    (List.mem_map_of_mem (a := co) (ArtifactCoordinate.mem_all co))

/-- Module bytes are bounded by the score: the size half of properness. -/
theorem moduleBytes_le_score (c : ArtifactVector) :
    c.static.moduleBytes ≤ CanonicalObjective.score c :=
  coordinate_le_score c .staticModuleBytes

/-- Executed rule steps are bounded by the score. -/
theorem wasmRuleSteps_le_score (c : ArtifactVector) :
    c.dynamicSum.wasmRuleSteps ≤ CanonicalObjective.score c :=
  coordinate_le_score c .sumWasmRuleSteps

/-- Auxiliary: sums of pointwise-dominated `Nat` lists are ordered. -/
theorem sum_le_sum {l : List ArtifactCoordinate} {a b : ArtifactVector}
    (hle : ComponentwiseLE a b) :
    (l.map (fun co => co.value a)).sum ≤ (l.map (fun co => co.value b)).sum :=
  sum_le_sum_of_pointwise _ _ hle l

/-- SPEC 9.3, `Cost.ProperObjective.monotone`, for the canonical objective. -/
theorem CanonicalObjective.monotone {a b : ArtifactVector}
    (h : ComponentwiseLE a b) :
    CanonicalObjective.score a ≤ CanonicalObjective.score b :=
  sum_le_sum h

end WasmGemmGnaf.Cost
