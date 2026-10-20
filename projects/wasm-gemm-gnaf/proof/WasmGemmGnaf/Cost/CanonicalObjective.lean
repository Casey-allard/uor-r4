/-
  The canonical release objective body.
  Normative source: SPEC.md sections 9.2 - 9.3.

  "`Release.costObjective.body` SHALL assign weight one to every constructor of
  `Cost.ArtifactCoordinate` and set `tieOrder` to
  `.unsignedByteLexicographic`."

  Every declaration in this file is proved. Nothing is assumed.
-/
import WasmGemmGnaf.Cost.Proper

set_option autoImplicit false

namespace WasmGemmGnaf.Cost

/-- SPEC 9.3: the canonical release objective body.  Every one of the 36
coordinate weights is exactly one and the tie order is the unsigned
byte-lexicographic order. -/
def canonicalObjectiveBody : ObjectiveBody :=
  { version := 1
    staticWeights := StaticCoordinateVector.const 1
    dynamicSumWeights := DynamicCoordinateVector.const 1
    dynamicMaxWeights := DynamicCoordinateVector.const 1
    tieOrder := .unsignedByteLexicographic }

/-- SPEC 9.3: weight one on every constructor of `ArtifactCoordinate`. -/
theorem canonicalObjectiveBody_weight (co : ArtifactCoordinate) :
    canonicalObjectiveBody.weight co = 1 := by
  cases co <;> rfl

/-- SPEC 9.3: the canonical tie order. -/
theorem canonicalObjectiveBody_tieOrder :
    canonicalObjectiveBody.tieOrder = .unsignedByteLexicographic := rfl

/-- The canonical body is admissible: every coordinate weight is positive. -/
theorem canonicalObjectiveBody_valid :
    EveryCoordinateWeightPositive canonicalObjectiveBody := by
  intro co
  rw [canonicalObjectiveBody_weight]
  exact Nat.le_refl 1

/-- The canonical weighted evaluation is the plain coordinate sum. -/
theorem evaluate_canonicalObjectiveBody (c : ArtifactVector) :
    evaluate canonicalObjectiveBody c = CanonicalObjective.score c := by
  simp only [evaluate, CanonicalObjective.score, canonicalObjectiveBody_weight,
    Nat.one_mul]

/-- SPEC 9.3: the canonical objective is a `ProperObjective` for any profile
and problem.  Its declared resource bound is the uniform coordinate bound, and
`sublevelBound` is discharged by `coordinate_le_score`; nothing is assumed. -/
def canonicalObjective {Profile Problem : Type} (P : Profile) (G : Problem) :
    ProperObjective P G :=
  { body := canonicalObjectiveBody
    bodyValid := canonicalObjectiveBody_valid
    boundOfScore := uniformBounds
    sublevelBound := fun h => uniform_sublevelBound canonicalObjectiveBody_valid h }

theorem canonicalObjective_body {Profile Problem : Type} (P : Profile)
    (G : Problem) : (canonicalObjective P G).body = canonicalObjectiveBody := rfl

theorem canonicalObjective_score {Profile Problem : Type} (P : Profile)
    (G : Problem) (c : CompleteSystemCost) :
    (canonicalObjective P G).score c = CanonicalObjective.score c :=
  evaluate_canonicalObjectiveBody c

/-- SPEC 9.2, `coordinate_le_score`, stated for the canonical objective as a
`ProperObjective`. -/
theorem canonicalObjective_coordinate_le_score {Profile Problem : Type}
    (P : Profile) (G : Problem) (c : CompleteSystemCost)
    (co : ArtifactCoordinate) : co.value c ≤ (canonicalObjective P G).score c :=
  ProperObjective.coordinate_le_score (canonicalObjective P G) c co

/-- SPEC 9.3, monotonicity of the canonical score. -/
theorem canonicalObjective_monotone {Profile Problem : Type} (P : Profile)
    (G : Problem) {a b : CompleteSystemCost} (h : ComponentwiseLE a b) :
    (canonicalObjective P G).score a ≤ (canonicalObjective P G).score b :=
  ProperObjective.monotone (canonicalObjective P G) h

/--
  SPEC 9.3: the canonical score's sublevels are finite.  Every cost vector
  scoring at most `u` occurs in the explicit list `sublevelEnumeration u`.
-/
theorem canonical_sublevel_finite (u : Nat) (c : ArtifactVector)
    (h : CanonicalObjective.score c ≤ u) : c ∈ sublevelEnumeration u :=
  mem_sublevelEnumeration (fun co => Nat.le_trans (coordinate_le_score c co) h)

/-- The canonical score is zero exactly on the zero cost vector: no charged
work is invisible to the objective. -/
theorem canonical_score_eq_zero_iff (c : ArtifactVector) :
    CanonicalObjective.score c = 0 ↔ c = ArtifactVector.zero := by
  constructor
  · intro h
    refine ArtifactVector.coords_injective (fun co => ?_)
    rw [ArtifactCoordinate.value_zero]
    exact Nat.le_zero.mp (h ▸ coordinate_le_score c co)
  · intro h
    rw [h, ← evaluate_canonicalObjectiveBody]
    exact evaluate_zero canonicalObjectiveBody

/-- The canonical tie-break relation on committed bytes: a total order that no
locale, hash order, or iteration order can perturb. -/
def canonicalTieBreak : ByteArray → ByteArray → Prop :=
  canonicalObjectiveBody.tieOrder.Order

theorem canonicalTieBreak_total (a b : ByteArray) :
    canonicalTieBreak a b ∨ canonicalTieBreak b a :=
  TieOrderTag.order_total _ a b

theorem canonicalTieBreak_antisymm {a b : ByteArray}
    (hab : canonicalTieBreak a b) (hba : canonicalTieBreak b a) : a = b :=
  TieOrderTag.order_antisymm _ hab hba

theorem canonicalTieBreak_trans {a b c : ByteArray}
    (hab : canonicalTieBreak a b) (hbc : canonicalTieBreak b c) :
    canonicalTieBreak a c :=
  TieOrderTag.order_trans _ hab hbc

theorem canonicalTieBreak_refl (a : ByteArray) : canonicalTieBreak a a :=
  TieOrderTag.order_refl _ a

end WasmGemmGnaf.Cost
