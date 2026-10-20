/-
  Proper objectives and finite score sublevels.
  Normative source: SPEC.md section 9.3.

  "An objective that leaves code size, execution, memory, advice, or another
  unbounded resource free SHALL not instantiate `ProperObjective` and SHALL not
  feed the global theorem."

  Every declaration in this file is proved. Nothing is assumed.
-/
import WasmGemmGnaf.Cost.Objective

set_option autoImplicit false

namespace WasmGemmGnaf.Cost

/-! ## Resource bounds -/

/-- A componentwise upper bound on every charged coordinate. -/
structure ResourceBounds where
  coordinateBound : ArtifactCoordinate → Nat

/-- A cost vector lies within a resource bound when every charged coordinate
does. -/
def Within (bounds : ResourceBounds) (c : CompleteSystemCost) : Prop :=
  ∀ co : ArtifactCoordinate, co.value c ≤ bounds.coordinateBound co

/-- The uniform bound: every coordinate at most `u`. -/
def uniformBounds (u : Nat) : ResourceBounds := ⟨fun _ => u⟩

theorem within_uniformBounds_iff (u : Nat) (c : CompleteSystemCost) :
    Within (uniformBounds u) c ↔ ∀ co : ArtifactCoordinate, co.value c ≤ u :=
  Iff.rfl

instance instDecidableWithinUniform (u : Nat) (c : CompleteSystemCost) :
    Decidable (Within (uniformBounds u) c) :=
  decidable_of_iff (ArtifactCoordinate.all.all (fun co => decide (co.value c ≤ u)) = true)
    (by
      constructor
      · intro h co
        rw [List.all_eq_true] at h
        exact of_decide_eq_true (h co (ArtifactCoordinate.mem_all co))
      · intro h
        rw [List.all_eq_true]
        intro co _
        exact decide_eq_true (h co))

/-! ## Proper objectives

The `Profile`/`Problem` indices of SPEC 9.3 are carried as opaque parameters:
the WebAssembly and GEMM layers instantiate them with `Wasm.Profile` and
`Gemm.Problem P` without any change to this file. -/

/-- SPEC 9.3, `Cost.ProperObjective`. -/
structure ProperObjective {Profile : Type} {Problem : Type}
    (P : Profile) (G : Problem) where
  body : ObjectiveBody
  bodyValid : EveryCoordinateWeightPositive body
  boundOfScore : Nat → ResourceBounds
  sublevelBound : ∀ {c : CompleteSystemCost} {u : Nat},
    evaluate body c ≤ u → Within (boundOfScore u) c

namespace ProperObjective

variable {Profile Problem : Type} {P : Profile} {G : Problem}

/-- SPEC 9.3, `Cost.ProperObjective.score`. -/
def score (objective : ProperObjective P G) (cost : CompleteSystemCost) : Nat :=
  evaluate objective.body cost

/-- SPEC 9.3, `Cost.ProperObjective.monotone`. -/
theorem monotone (objective : ProperObjective P G) {a b : CompleteSystemCost}
    (h : ComponentwiseLE a b) : objective.score a ≤ objective.score b :=
  evaluate_monotone objective.body h

/-- Every charged coordinate is bounded by the score of any proper
objective. -/
theorem coordinate_le_score (objective : ProperObjective P G)
    (c : CompleteSystemCost) (co : ArtifactCoordinate) :
    co.value c ≤ objective.score c :=
  coordinate_le_evaluate objective.bodyValid c co

/-- Module bytes are bounded by the score of any proper objective: no proper
objective leaves code size free. -/
theorem moduleBytes_le_score (objective : ProperObjective P G)
    (c : CompleteSystemCost) : c.static.moduleBytes ≤ objective.score c :=
  coordinate_le_score objective c .staticModuleBytes

/-- Every coordinate of a sublevel member is bounded by the sublevel. -/
theorem coordinate_le_of_score_le (objective : ProperObjective P G)
    {c : CompleteSystemCost} {u : Nat} (h : objective.score c ≤ u)
    (co : ArtifactCoordinate) : co.value c ≤ u :=
  Nat.le_trans (coordinate_le_score objective c co) h

/-- A proper objective's own declared bound holds on its sublevels. -/
theorem within_boundOfScore (objective : ProperObjective P G)
    {c : CompleteSystemCost} {u : Nat} (h : objective.score c ≤ u) :
    Within (objective.boundOfScore u) c :=
  objective.sublevelBound h

end ProperObjective

/-! ## Finite enumeration of bounded coordinate assignments

The sublevel `{c | score c ≤ u}` injects into the assignments of the 36 charged
coordinates to values in `{0, …, u}`, a finite product.  The enumeration below
realises that product as an explicit list. -/

/-- Every assignment of values `≤ u` to the indices in `l`, extended by zero
outside `l`. -/
def boundedFunctions {ι : Type} [DecidableEq ι] (u : Nat) : List ι → List (ι → Nat)
  | [] => [fun _ => 0]
  | i :: rest =>
      (List.range (u + 1)).flatMap (fun v =>
        (boundedFunctions u rest).map (fun f j => if j = i then v else f j))

theorem boundedFunctions_nil {ι : Type} [DecidableEq ι] (u : Nat) :
    boundedFunctions (ι := ι) u [] = [fun _ => 0] := rfl

theorem boundedFunctions_cons {ι : Type} [DecidableEq ι] (u : Nat) (i : ι)
    (rest : List ι) :
    boundedFunctions u (i :: rest) =
      (List.range (u + 1)).flatMap (fun v =>
        (boundedFunctions u rest).map (fun f j => if j = i then v else f j)) := rfl

/-- Completeness of the enumeration: every zero-extended assignment bounded by
`u` on a duplicate-free index list occurs in it. -/
theorem mem_boundedFunctions {ι : Type} [DecidableEq ι] (u : Nat) :
    ∀ (l : List ι), l.Nodup → ∀ f : ι → Nat,
      (∀ i ∈ l, f i ≤ u) → (∀ j, j ∉ l → f j = 0) →
      f ∈ boundedFunctions u l := by
  intro l
  induction l with
  | nil =>
      intro _ f _ hzero
      have : f = fun _ => 0 := funext (fun j => hzero j (by simp))
      rw [boundedFunctions_nil, this]
      simp
  | cons i rest ih =>
      intro hnodup f hbound hzero
      have hi : i ∉ rest := (List.nodup_cons.mp hnodup).1
      have hrest : rest.Nodup := (List.nodup_cons.mp hnodup).2
      -- the assignment with `i` reset to zero lives in the tail enumeration
      have hg : (fun j => if j = i then 0 else f j) ∈ boundedFunctions u rest := by
        refine ih hrest _ ?_ ?_
        · intro j hj
          have hne : j ≠ i := fun h => hi (h ▸ hj)
          simp only [hne, if_false]
          exact hbound j (List.mem_cons_of_mem _ hj)
        · intro j hj
          by_cases hji : j = i
          · simp [hji]
          · simp only [hji, if_false]
            exact hzero j (fun hmem => by
              rcases List.mem_cons.mp hmem with rfl | h
              · exact hji rfl
              · exact hj h)
      rw [boundedFunctions_cons, List.mem_flatMap]
      refine ⟨f i, ?_, ?_⟩
      · rw [List.mem_range]
        exact Nat.lt_succ_of_le (hbound i (List.mem_cons_self ..))
      · rw [List.mem_map]
        refine ⟨_, hg, ?_⟩
        funext j
        by_cases hji : j = i
        · simp [hji]
        · simp [hji]

/-- Auxiliary: a `flatMap` whose blocks all have the same length. -/
theorem length_flatMap_const {α β : Type} (k : Nat) (g : α → List β)
    (h : ∀ a, (g a).length = k) :
    ∀ l : List α, (l.flatMap g).length = l.length * k
  | [] => by simp
  | x :: t => by
      rw [List.flatMap_cons, List.length_append, h x,
        length_flatMap_const k g h t, List.length_cons, Nat.succ_mul,
        Nat.add_comm]

/-- The enumeration realises the finite product `{0, …, u}^l`. -/
theorem boundedFunctions_length {ι : Type} [DecidableEq ι] (u : Nat) :
    ∀ l : List ι, (boundedFunctions u l).length = (u + 1) ^ l.length
  | [] => rfl
  | i :: rest => by
      rw [boundedFunctions_cons,
        length_flatMap_const (boundedFunctions u rest).length _
          (fun _ => List.length_map ..),
        List.length_range, boundedFunctions_length u rest, List.length_cons,
        Nat.pow_succ]
      exact Nat.mul_comm _ _

/-- The 36-coordinate assignments bounded by `u`. -/
def boundedCoordinateFunctions (u : Nat) : List (ArtifactCoordinate → Nat) :=
  boundedFunctions u ArtifactCoordinate.all

theorem mem_boundedCoordinateFunctions (u : Nat) (f : ArtifactCoordinate → Nat)
    (h : ∀ co, f co ≤ u) : f ∈ boundedCoordinateFunctions u :=
  mem_boundedFunctions u ArtifactCoordinate.all ArtifactCoordinate.all_nodup f
    (fun co _ => h co)
    (fun co hco => absurd (ArtifactCoordinate.mem_all co) hco)

/-- An explicit finite list containing every cost vector whose coordinates are
all bounded by `u`. -/
def sublevelEnumeration (u : Nat) : List ArtifactVector :=
  (boundedCoordinateFunctions u).map ArtifactVector.ofCoords

/-- Finiteness of a coordinatewise-bounded set of cost vectors. -/
theorem mem_sublevelEnumeration {u : Nat} {c : ArtifactVector}
    (h : ∀ co : ArtifactCoordinate, co.value c ≤ u) :
    c ∈ sublevelEnumeration u := by
  have hf := mem_boundedCoordinateFunctions u (fun co => co.value c) h
  have hmem : ArtifactVector.ofCoords (fun co => co.value c)
      ∈ sublevelEnumeration u := List.mem_map_of_mem hf
  rwa [ArtifactVector.ofCoords_value] at hmem

/--
  SPEC 9.3: every score sublevel of a proper objective is finite.

  `sublevelEnumeration u` is an explicit finite list containing every cost
  vector whose score is at most `u`.  The proof is the injection into the
  finite product of the 36 coordinate ranges `{0, …, u}`: by
  `ProperObjective.coordinate_le_score` each coordinate of a sublevel member is
  bounded by `u`, and `ArtifactVector.ofCoords_value` shows the coordinate map
  is a retraction, so no sublevel member escapes the enumeration.
-/
theorem objective_sublevel_finite {Profile Problem : Type} {P : Profile}
    {G : Problem} (objective : ProperObjective P G) (u : Nat)
    (c : CompleteSystemCost) (h : objective.score c ≤ u) :
    c ∈ sublevelEnumeration u :=
  mem_sublevelEnumeration
    (fun co => ProperObjective.coordinate_le_of_score_le objective h co)

/-- The sublevel enumeration is exhaustive for `Within` bounds as well. -/
theorem within_uniform_mem_sublevelEnumeration {u : Nat} {c : CompleteSystemCost}
    (h : Within (uniformBounds u) c) : c ∈ sublevelEnumeration u :=
  mem_sublevelEnumeration h

/-- The sublevel enumeration is exactly the finite product of the 36 coordinate
ranges `{0, …, u}`. -/
theorem sublevelEnumeration_length (u : Nat) :
    (sublevelEnumeration u).length = (u + 1) ^ 36 := by
  rw [sublevelEnumeration, List.length_map, boundedCoordinateFunctions,
    boundedFunctions_length, ArtifactCoordinate.all_length]

/-- The coordinate map of a coordinatewise-bounded cost vector, valued in the
finite product `ArtifactCoordinate → Fin (u+1)`. -/
def sublevelCoords (u : Nat) (c : ArtifactVector)
    (h : ∀ co : ArtifactCoordinate, co.value c ≤ u) :
    ArtifactCoordinate → Fin (u + 1) :=
  fun co => ⟨co.value c, Nat.lt_succ_of_le (h co)⟩

/-- That map is injective: the sublevel embeds into a finite product. -/
theorem sublevelCoords_injective {u : Nat} {a b : ArtifactVector}
    {ha : ∀ co : ArtifactCoordinate, co.value a ≤ u}
    {hb : ∀ co : ArtifactCoordinate, co.value b ≤ u}
    (h : sublevelCoords u a ha = sublevelCoords u b hb) : a = b := by
  refine ArtifactVector.coords_injective (fun co => ?_)
  have := congrFun h co
  simpa [sublevelCoords, Fin.ext_iff] using this

/-- Every proper objective admits the uniform bound: it is a legitimate
`boundOfScore`. -/
theorem uniform_sublevelBound {body : ObjectiveBody}
    (hpos : EveryCoordinateWeightPositive body) {c : CompleteSystemCost}
    {u : Nat} (h : evaluate body c ≤ u) : Within (uniformBounds u) c :=
  fun co => Nat.le_trans (coordinate_le_evaluate hpos c co) h

end WasmGemmGnaf.Cost
