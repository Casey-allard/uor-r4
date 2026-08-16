import WasmGemmGnaf.Atlas.Certificate
import WasmGemmGnaf.Foundation.Termination
set_option autoImplicit false

/-!
# Atlas: semantic closure (SPEC §12.1)

SPEC §12.1 requires the Atlas state to carry the *least* semantic closure, and
records `Atlas.SealCheckTag.closureLeast` for it.  UOR-GNAF §18 lists

  *that a fixed point is the least derivation closure*

as an explicit **non-claim**.  This file discharges it.

## What is proved

The closure operator `Closure.Cl R` is a terminating saturation over a finite
canonical judgment set: it iterates the immediate-consequence operator
`Closure.step R` with the explicit well-founded measure
`Closure.residual R A = #{ rule conclusions not yet in A }`, whose strict
decrease at every unsaturated state is `Closure.residual_lt`.  The number of
iterations is the concrete budget `Closure.closureFuel R`, and
`Foundation.Termination.iterateUntil_sound` turns "the measure decreases" into
"the result is saturated".

* `Closure.Cl_extensive`, `Closure.Cl_monotone`, `Closure.Cl_idempotent` — the
  three closure laws.
* `Atlas.semantic_closure_merge` — UOR-GNAF **Theorem 11.3**,
  `Cl (A ∪ B) = Cl (Cl A ∪ Cl B)`, derived from an abstract statement
  (`Closure.closure_merge_law`) that assumes only the three laws.
* `Atlas.semantic_closure_least` — the closure is the **least** set that is
  closed under the rules and contains `A`, together with
  `Atlas.semantic_closure_eq_derivable`, which identifies it pointwise with the
  inductively defined derivation relation `Closure.Derivable`.  That equality is
  the statement UOR-GNAF §18 refuses to assume.

## Optimizer conclusions are excluded by typing

SPEC §12.1: "Optimizer conclusions such as best, dominated, or selected SHALL
NOT be premises in semantic closure."

The premise list of a `SemanticRule` has type `List SemanticJudgment`.
`SemanticJudgment.kind` ranges over `SemanticKind`, which has no `best`,
`dominated` or `selected` constructor; the optimizer verdicts live in the
*separate* type `OptimizerJudgment`, and there is no coercion between them.  So
an optimizer conclusion cannot be written as a premise at all.

Because a "type-level" argument is worthless if the two types collapse after
canonical encoding — identities are bytes — the separation is also proved at the
byte level: `Atlas.semanticJudgment_bytes_ne_optimizer` shows the two encodings
are never equal, hence `Atlas.premise_ne_optimizer_conclusion` and
`Atlas.closure_fact_ne_optimizer_conclusion`.

## Anti-vacuity

`Atlas.closureLeastCheck` (Atlas/Certificate.lean) is a *bookkeeping* check.
`Atlas.closureBodySupportCheck_accepts_underivable` proves — with a concrete
witness — that its structural conditions accept a fact that is **not** in the
least closure of the empty base: a cyclic derivation edge supports itself.  So a
passing `closureLeastCheck` is evidence of a fixed point, not of leastness;
leastness must come from `Atlas.semantic_closure_least` applied to the rule set
that produced the facts.
-/

namespace WasmGemmGnaf.Atlas

open WasmGemmGnaf.Foundation

/-! ## Generic least-closure machinery

The machinery is polymorphic in the judgment carrier `J`.  Nothing here can
introduce an optimizer premise: the *instantiation* used for semantic closure
fixes `J := SemanticJudgment`, and that type has no optimizer constructor. -/

namespace Closure

/-- An inference rule: a finite list of premises entailing one conclusion. -/
structure Rule (J : Type) where
  premises : List J
  conclusion : J
  deriving DecidableEq

/-- A set of judgments, as its decidable characteristic function. -/
abbrev FactSet (J : Type) := J → Bool

variable {J : Type} [DecidableEq J]

/-! ### Set algebra -/

/-- Inclusion of fact sets. -/
def FactSub (A B : FactSet J) : Prop := ∀ x, A x = true → B x = true

omit [DecidableEq J] in
theorem FactSub.refl (A : FactSet J) : FactSub A A := fun _ h => h

omit [DecidableEq J] in
theorem FactSub.trans {A B C : FactSet J} (h₁ : FactSub A B) (h₂ : FactSub B C) :
    FactSub A C := fun x h => h₂ x (h₁ x h)

omit [DecidableEq J] in
/-- Mutual inclusion is equality: fact sets are extensional. -/
theorem factSet_ext {A B : FactSet J} (h₁ : FactSub A B) (h₂ : FactSub B A) : A = B := by
  funext x
  by_cases hA : A x = true
  · rw [hA, h₁ x hA]
  · have hA' : A x = false := by simpa using hA
    by_cases hB : B x = true
    · exact absurd (h₂ x hB) (by rw [hA']; simp)
    · have hB' : B x = false := by simpa using hB
      rw [hA', hB']

/-- Union of fact sets. -/
def factUnion (A B : FactSet J) : FactSet J := fun x => A x || B x

omit [DecidableEq J] in
theorem factSub_union_left (A B : FactSet J) : FactSub A (factUnion A B) := by
  intro x h; simp [factUnion, h]

omit [DecidableEq J] in
theorem factSub_union_right (A B : FactSet J) : FactSub B (factUnion A B) := by
  intro x h; simp [factUnion, h]

omit [DecidableEq J] in
theorem factUnion_least {A B C : FactSet J} (hA : FactSub A C) (hB : FactSub B C) :
    FactSub (factUnion A B) C := by
  intro x h
  simp only [factUnion, Bool.or_eq_true] at h
  rcases h with h | h
  · exact hA x h
  · exact hB x h

/-! ### Closedness -/

/-- `S` is closed under `R`: whenever every premise of a rule holds, so does its
conclusion.  Stated for an arbitrary predicate so that leastness can be proved
against *every* closed set, not only the decidable ones. -/
def ClosedUnder (R : List (Rule J)) (S : J → Prop) : Prop :=
  ∀ r ∈ R, (∀ p ∈ r.premises, S p) → S r.conclusion

/-- Closedness of a decidable fact set. -/
def ClosedFacts (R : List (Rule J)) (A : FactSet J) : Prop :=
  ClosedUnder R (fun x => A x = true)

/-! ### The immediate-consequence operator -/

/-- One rule application: `x` is an immediate consequence of `A` under `R`. -/
def derives (R : List (Rule J)) (A : FactSet J) (x : J) : Bool :=
  R.any (fun r => decide (r.conclusion = x) && r.premises.all A)

theorem derives_iff (R : List (Rule J)) (A : FactSet J) (x : J) :
    derives R A x = true ↔
      ∃ r ∈ R, r.conclusion = x ∧ ∀ p ∈ r.premises, A p = true := by
  simp [derives, List.any_eq_true, List.all_eq_true]

/-- The immediate-consequence operator: add every conclusion whose premises are
already present. -/
def step (R : List (Rule J)) (A : FactSet J) : FactSet J :=
  fun x => A x || derives R A x

theorem step_extensive (R : List (Rule J)) (A : FactSet J) : FactSub A (step R A) := by
  intro x h; simp [step, h]

theorem step_mono (R : List (Rule J)) {A B : FactSet J} (h : FactSub A B) :
    FactSub (step R A) (step R B) := by
  intro x hx
  simp only [step, Bool.or_eq_true] at hx ⊢
  rcases hx with hx | hx
  · exact Or.inl (h x hx)
  · refine Or.inr ?_
    rw [derives_iff] at hx ⊢
    obtain ⟨r, hr, hc, hp⟩ := hx
    exact ⟨r, hr, hc, fun p hpm => h p (hp p hpm)⟩

/-- A set closed under `R` absorbs the immediate consequences of any of its
subsets. -/
theorem step_preserves {R : List (Rule J)} {A : FactSet J} {S : J → Prop}
    (hS : ClosedUnder R S) (hA : ∀ x, A x = true → S x) :
    ∀ x, step R A x = true → S x := by
  intro x hx
  simp only [step, Bool.or_eq_true] at hx
  rcases hx with hx | hx
  · exact hA x hx
  · rw [derives_iff] at hx
    obtain ⟨r, hr, hc, hp⟩ := hx
    exact hc ▸ hS r hr (fun p hpm => hA p (hp p hpm))

/-! ### The termination measure

Every judgment the operator can add is the conclusion of some rule, so the
finite list `conclusions R` bounds all progress.  The measure counts how many
of those are still missing. -/

/-- The finite canonical judgment set the closure can add to `A`. -/
def conclusions (R : List (Rule J)) : List J := R.map (·.conclusion)

theorem derives_conclusion_mem {R : List (Rule J)} {A : FactSet J} {x : J}
    (h : derives R A x = true) : x ∈ conclusions R := by
  rw [derives_iff] at h
  obtain ⟨r, hr, hc, _⟩ := h
  exact hc ▸ List.mem_map_of_mem hr

/-- The number of elements of `l` that `A` does not contain. -/
def countMissing (A : FactSet J) : List J → Nat
  | [] => 0
  | x :: t => (if A x = true then 0 else 1) + countMissing A t

omit [DecidableEq J] in
theorem countMissing_le_length (A : FactSet J) :
    ∀ l : List J, countMissing A l ≤ l.length := by
  intro l
  induction l with
  | nil => simp [countMissing]
  | cons x t ih =>
    simp only [countMissing, List.length_cons]
    split <;> omega

omit [DecidableEq J] in
theorem countMissing_mono {A B : FactSet J} (h : FactSub A B) :
    ∀ l : List J, countMissing B l ≤ countMissing A l := by
  intro l
  induction l with
  | nil => simp [countMissing]
  | cons x t ih =>
    simp only [countMissing]
    by_cases hA : A x = true
    · rw [if_pos hA, if_pos (h x hA)]; omega
    · rw [if_neg hA]; split <;> omega

omit [DecidableEq J] in
theorem countMissing_lt {A B : FactSet J} (h : FactSub A B) (x : J)
    (hA : A x = false) (hB : B x = true) :
    ∀ l : List J, x ∈ l → countMissing B l < countMissing A l := by
  intro l
  induction l with
  | nil => intro hx; exact absurd hx (by simp)
  | cons y t ih =>
    intro hx
    simp only [countMissing]
    rcases List.mem_cons.mp hx with rfl | hx'
    · rw [if_pos hB, if_neg (show ¬ (A x = true) by simp [hA])]
      have := countMissing_mono h t
      omega
    · have hlt := ih hx'
      by_cases hAy : A y = true
      · rw [if_pos hAy, if_pos (h y hAy)]; omega
      · rw [if_neg hAy]; split <;> omega

/-- The explicit well-founded measure: the number of rule conclusions not yet
present. -/
def residual (R : List (Rule J)) (A : FactSet J) : Nat :=
  countMissing A (conclusions R)

/-- The concrete step budget. -/
def closureFuel (R : List (Rule J)) : Nat := (conclusions R).length

/-- The conclusions that are derivable but not yet present. -/
def pending (R : List (Rule J)) (A : FactSet J) : List J :=
  (conclusions R).filter (fun x => derives R A x && !(A x))

/-- Saturation is decidable: nothing new is derivable. -/
def saturated (R : List (Rule J)) (A : FactSet J) : Bool := (pending R A).isEmpty

theorem mem_pending_iff {R : List (Rule J)} {A : FactSet J} {x : J} :
    x ∈ pending R A ↔ x ∈ conclusions R ∧ derives R A x = true ∧ A x = false := by
  simp only [pending, List.mem_filter, Bool.and_eq_true, Bool.not_eq_true']

/-- A saturated set is closed under the rules. -/
theorem saturated_closed {R : List (Rule J)} {A : FactSet J}
    (h : saturated R A = true) : ClosedFacts R A := by
  intro r hr hp
  by_cases hA : A r.conclusion = true
  · exact hA
  · exfalso
    have hA' : A r.conclusion = false := by simpa using hA
    have hd : derives R A r.conclusion = true := by
      rw [derives_iff]
      exact ⟨r, hr, rfl, hp⟩
    have hmem : r.conclusion ∈ pending R A :=
      mem_pending_iff.mpr ⟨derives_conclusion_mem hd, hd, hA'⟩
    have : pending R A = [] := List.isEmpty_iff.mp h
    rw [this] at hmem
    exact absurd hmem (by simp)

/-! ### Strict decrease of the measure -/

/-- **The termination argument.**  At an unsaturated state the measure strictly
decreases. -/
theorem residual_lt (R : List (Rule J)) (A : FactSet J) (h : saturated R A = false) :
    residual R (step R A) < residual R A := by
  have hne : pending R A ≠ [] := by
    intro hnil
    rw [saturated, hnil] at h
    simp at h
  obtain ⟨x, hx⟩ : ∃ x, x ∈ pending R A := by
    cases hp : pending R A with
    | nil => exact absurd hp hne
    | cons y t => exact ⟨y, List.mem_cons_self⟩
  obtain ⟨hmem, hd, hA⟩ := mem_pending_iff.mp hx
  exact countMissing_lt (step_extensive R A) x hA (by simp [step, hd])
    (conclusions R) hmem

omit [DecidableEq J] in
theorem residual_le_fuel (R : List (Rule J)) (A : FactSet J) :
    residual R A ≤ closureFuel R := countMissing_le_length A (conclusions R)

/-! ### The closure operator -/

/-- **The least-closure operator.**  Saturation of `A` under `R`, computed by
structural iteration on the explicit budget `closureFuel R`; the budget suffices
because `residual` strictly decreases (`residual_lt`). -/
def Cl (R : List (Rule J)) (A : FactSet J) : FactSet J :=
  Termination.iterateUntil (step R) (saturated R) (closureFuel R) A

/-- The computed closure really is saturated: the budget was sufficient. -/
theorem Cl_saturated (R : List (Rule J)) (A : FactSet J) :
    saturated R (Cl R A) = true :=
  Termination.iterateUntil_sound (step R) (saturated R) (residual R)
    (fun X hX => residual_lt R X hX) (closureFuel R) A (residual_le_fuel R A)

theorem iter_extensive (R : List (Rule J)) : ∀ (n : Nat) (A : FactSet J),
    FactSub A (Termination.iterateUntil (step R) (saturated R) n A) := by
  intro n
  induction n with
  | zero => intro A; exact FactSub.refl A
  | succ n ih =>
    intro A
    rw [Termination.iterateUntil_succ]
    by_cases h : saturated R A = true
    · simp only [h, if_true]; exact FactSub.refl A
    · have h' : saturated R A = false := by simpa using h
      simp only [h', Bool.false_eq_true, if_false]
      exact FactSub.trans (step_extensive R A) (ih (step R A))

theorem iter_least (R : List (Rule J)) (S : J → Prop) (hS : ClosedUnder R S) :
    ∀ (n : Nat) (A : FactSet J), (∀ x, A x = true → S x) →
      ∀ x, Termination.iterateUntil (step R) (saturated R) n A x = true → S x := by
  intro n
  induction n with
  | zero => intro A hA x hx; exact hA x hx
  | succ n ih =>
    intro A hA x hx
    rw [Termination.iterateUntil_succ] at hx
    by_cases h : saturated R A = true
    · rw [if_pos h] at hx; exact hA x hx
    · have h' : saturated R A = false := by simpa using h
      rw [if_neg (by simp [h'])] at hx
      exact ih (step R A) (step_preserves hS hA) x hx

/-- **Extensivity**: `A ⊆ Cl A`. -/
theorem Cl_extensive (R : List (Rule J)) (A : FactSet J) : FactSub A (Cl R A) :=
  iter_extensive R (closureFuel R) A

/-- **Leastness**, in its strongest form: every predicate that is closed under
`R` and contains `A` contains the closure. -/
theorem Cl_least (R : List (Rule J)) (A : FactSet J) (S : J → Prop)
    (hS : ClosedUnder R S) (hA : ∀ x, A x = true → S x) :
    ∀ x, Cl R A x = true → S x :=
  iter_least R S hS (closureFuel R) A hA

/-- The closure is closed under the rules. -/
theorem Cl_closed (R : List (Rule J)) (A : FactSet J) : ClosedFacts R (Cl R A) :=
  saturated_closed (Cl_saturated R A)

/-- Leastness among decidable fact sets. -/
theorem Cl_least_facts (R : List (Rule J)) (A B : FactSet J)
    (hB : ClosedFacts R B) (hAB : FactSub A B) : FactSub (Cl R A) B :=
  Cl_least R A (fun x => B x = true) hB hAB

/-- **Monotonicity**: `A ⊆ B → Cl A ⊆ Cl B`.  Proved *from* leastness. -/
theorem Cl_monotone (R : List (Rule J)) {A B : FactSet J} (h : FactSub A B) :
    FactSub (Cl R A) (Cl R B) :=
  Cl_least_facts R A (Cl R B) (Cl_closed R B) (FactSub.trans h (Cl_extensive R B))

/-- **Idempotence**: `Cl (Cl A) = Cl A`. -/
theorem Cl_idempotent (R : List (Rule J)) (A : FactSet J) : Cl R (Cl R A) = Cl R A :=
  factSet_ext
    (Cl_least_facts R (Cl R A) (Cl R A) (Cl_closed R A) (FactSub.refl _))
    (Cl_extensive R (Cl R A))

/-- The closure is a fixed point of the immediate-consequence operator. -/
theorem Cl_fixed_point (R : List (Rule J)) (A : FactSet J) :
    step R (Cl R A) = Cl R A := by
  refine factSet_ext ?_ (step_extensive R (Cl R A))
  intro x hx
  simp only [step, Bool.or_eq_true] at hx
  rcases hx with hx | hx
  · exact hx
  · by_cases hc : Cl R A x = true
    · exact hc
    · exfalso
      have hc' : Cl R A x = false := by simpa using hc
      have hmem : x ∈ pending R (Cl R A) :=
        mem_pending_iff.mpr ⟨derives_conclusion_mem hx, hx, hc'⟩
      have hnil : pending R (Cl R A) = [] := List.isEmpty_iff.mp (Cl_saturated R A)
      rw [hnil] at hmem
      exact absurd hmem (by simp)

/-! ### The merge law (UOR-GNAF Theorem 11.3)

Stated abstractly first: it needs *only* extensivity, monotonicity and
idempotence, exactly as the draft's proof does. -/

omit [DecidableEq J] in
/-- **UOR-GNAF Theorem 11.3**, abstract form: any extensive, monotone,
idempotent operator on fact sets satisfies `Cl (A ∪ B) = Cl (Cl A ∪ Cl B)`. -/
theorem closure_merge_law (Cl' : FactSet J → FactSet J)
    (ext : ∀ A, FactSub A (Cl' A))
    (mono : ∀ A B, FactSub A B → FactSub (Cl' A) (Cl' B))
    (idem : ∀ A, Cl' (Cl' A) = Cl' A)
    (A B : FactSet J) :
    Cl' (factUnion A B) = Cl' (factUnion (Cl' A) (Cl' B)) := by
  refine factSet_ext ?_ ?_
  · -- extensivity gives `A ∪ B ⊆ Cl' A ∪ Cl' B`; apply monotonicity.
    exact mono _ _ (factUnion_least
      (FactSub.trans (ext A) (factSub_union_left (Cl' A) (Cl' B)))
      (FactSub.trans (ext B) (factSub_union_right (Cl' A) (Cl' B))))
  · -- monotonicity gives `Cl' A ∪ Cl' B ⊆ Cl' (A ∪ B)`; apply monotonicity and idempotence.
    have h : FactSub (factUnion (Cl' A) (Cl' B)) (Cl' (factUnion A B)) :=
      factUnion_least
        (mono _ _ (factSub_union_left A B))
        (mono _ _ (factSub_union_right A B))
    have h2 := mono _ _ h
    rw [idem] at h2
    exact h2

/-- **UOR-GNAF Theorem 11.3** for the least-closure operator. -/
theorem Cl_merge (R : List (Rule J)) (A B : FactSet J) :
    Cl R (factUnion A B) = Cl R (factUnion (Cl R A) (Cl R B)) := by
  refine factSet_ext ?_ ?_
  · -- `A ∪ B ⊆ Cl A ∪ Cl B`, then monotonicity.
    exact Cl_monotone R (factUnion_least
      (FactSub.trans (Cl_extensive R A) (factSub_union_left (Cl R A) (Cl R B)))
      (FactSub.trans (Cl_extensive R B) (factSub_union_right (Cl R A) (Cl R B))))
  · -- `Cl A ∪ Cl B ⊆ Cl (A ∪ B)`, then monotonicity and idempotence.
    have h : FactSub (factUnion (Cl R A) (Cl R B)) (Cl R (factUnion A B)) :=
      factUnion_least
        (Cl_monotone R (factSub_union_left A B))
        (Cl_monotone R (factSub_union_right A B))
    have h2 := Cl_monotone R h
    rw [Cl_idempotent] at h2
    exact h2

/-! ### The closure is exactly the derivation closure -/

/-- Inductive derivability from `A` under `R`. -/
inductive Derivable (R : List (Rule J)) (A : FactSet J) : J → Prop
  | base {x : J} : A x = true → Derivable R A x
  | rule {r : Rule J} : r ∈ R → (∀ p ∈ r.premises, Derivable R A p) →
      Derivable R A r.conclusion

omit [DecidableEq J] in
theorem Derivable_closed (R : List (Rule J)) (A : FactSet J) :
    ClosedUnder R (Derivable R A) := fun _ hr hp => Derivable.rule hr hp

/-- Everything the computed closure contains is genuinely derivable. -/
theorem Cl_sound (R : List (Rule J)) (A : FactSet J) (x : J) (h : Cl R A x = true) :
    Derivable R A x :=
  Cl_least R A (Derivable R A) (Derivable_closed R A) (fun _ hx => Derivable.base hx) x h

/-- Everything derivable is in the computed closure. -/
theorem Cl_complete (R : List (Rule J)) (A : FactSet J) (x : J)
    (h : Derivable R A x) : Cl R A x = true := by
  induction h with
  | base hx => exact Cl_extensive R A _ hx
  | rule hr _ ih => exact Cl_closed R A _ hr ih

/-- **The closure is the derivation closure**, pointwise. -/
theorem Cl_eq_derivable (R : List (Rule J)) (A : FactSet J) (x : J) :
    Cl R A x = true ↔ Derivable R A x :=
  ⟨Cl_sound R A x, Cl_complete R A x⟩

/-- Every derivable judgment is a base fact or a rule conclusion: the closure
never leaves the finite canonical judgment set. -/
theorem derivable_mem (R : List (Rule J)) (A : List J) (x : J)
    (h : Derivable R (fun y => A.any (fun z => decide (z = y))) x) :
    x ∈ A ++ conclusions R := by
  induction h with
  | @base y hy =>
    simp only [List.any_eq_true, decide_eq_true_eq] at hy
    obtain ⟨z, hz, rfl⟩ := hy
    exact List.mem_append_left _ hz
  | rule hr _ _ => exact List.mem_append_right _ (List.mem_map_of_mem hr)

/-! ### Finite list form -/

/-- The characteristic function of a finite list of judgments. -/
def ofList (A : List J) : FactSet J := fun x => A.any (fun z => decide (z = x))

theorem ofList_iff (A : List J) (x : J) : ofList A x = true ↔ x ∈ A := by
  simp only [ofList, List.any_eq_true, decide_eq_true_eq]
  exact ⟨fun ⟨z, hz, hzx⟩ => hzx ▸ hz, fun h => ⟨x, h, rfl⟩⟩

/-- The closure of a finite fact list, as a finite list. -/
def closureList (R : List (Rule J)) (A : List J) : List J :=
  (A ++ conclusions R).filter (fun x => Cl R (ofList A) x)

theorem mem_closureList_iff (R : List (Rule J)) (A : List J) (x : J) :
    x ∈ closureList R A ↔ Cl R (ofList A) x = true := by
  simp only [closureList, List.mem_filter]
  constructor
  · intro h; exact h.2
  · intro h
    exact ⟨derivable_mem R A x (Cl_sound R (ofList A) x h), h⟩

end Closure

/-! ## The semantic judgment carrier (SPEC §12.1)

The point of this section is the *type*: a closure premise is a
`SemanticJudgment`, and `SemanticKind` has no optimizer constructor. -/

/-- The closed set of **semantic** judgment kinds.  There is deliberately no
`best`, `dominated` or `selected` constructor here. -/
inductive SemanticKind
  | objectWellFormed
  | shapeCompatible
  | edgeWellTyped
  | normalForm
  | typeAssignment
  | equalUnderRewrite
  deriving DecidableEq, Repr, Inhabited

/-- The closed set of **optimizer** verdicts.  A separate type: SPEC §12.1
forbids these as closure premises, and nothing coerces them into
`SemanticKind`. -/
inductive OptimizerKind
  | best
  | dominated
  | selected
  deriving DecidableEq, Repr, Inhabited

namespace SemanticKind

/-- Canonical tag byte.  Semantic tags occupy `0 … 5`. -/
def tag : SemanticKind → UInt8
  | objectWellFormed => 0
  | shapeCompatible => 1
  | edgeWellTyped => 2
  | normalForm => 3
  | typeAssignment => 4
  | equalUnderRewrite => 5

theorem tag_injective : Function.Injective tag := by
  intro a b h; cases a <;> cases b <;> simp_all [tag]

def all : List SemanticKind :=
  [objectWellFormed, shapeCompatible, edgeWellTyped, normalForm, typeAssignment,
   equalUnderRewrite]

theorem mem_all (k : SemanticKind) : k ∈ all := by cases k <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance : Foundation.Fintype SemanticKind where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

def bytes (k : SemanticKind) : List UInt8 := [k.tag]

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.prefixFree_of_constLength bytes 1 (fun k => by cases k <;> rfl)
    (fun _ _ h => tag_injective (by simpa [bytes] using h))

end SemanticKind

namespace OptimizerKind

/-- Canonical tag byte.  Optimizer tags occupy `128 …`, disjoint from the
semantic tags by construction. -/
def tag : OptimizerKind → UInt8
  | best => 128
  | dominated => 129
  | selected => 130

theorem tag_injective : Function.Injective tag := by
  intro a b h; cases a <;> cases b <;> simp_all [tag]

def all : List OptimizerKind := [best, dominated, selected]

theorem mem_all (k : OptimizerKind) : k ∈ all := by cases k <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance : Foundation.Fintype OptimizerKind where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

def bytes (k : OptimizerKind) : List UInt8 := [k.tag]

end OptimizerKind

/-- **The tag universes are disjoint.**  No semantic kind is spelled like an
optimizer verdict, even after erasure to canonical bytes. -/
theorem semanticKind_tag_ne_optimizerKind (k : SemanticKind) (o : OptimizerKind) :
    k.tag ≠ o.tag := by
  cases k <;> cases o <;> decide

/-- A semantic judgment: a semantic kind applied to a subject and arguments.
This is the only type a closure premise can have. -/
structure SemanticJudgment where
  kind : SemanticKind
  subject : CanonicalObjectId
  arguments : List CanonicalObjectId
  deriving DecidableEq

/-- An optimizer conclusion.  It is *not* a `SemanticJudgment`, so it cannot be
a premise of any `SemanticRule`. -/
structure OptimizerJudgment where
  kind : OptimizerKind
  subject : CanonicalObjectId
  arguments : List CanonicalObjectId
  deriving DecidableEq

namespace SemanticJudgment

def toTuple (j : SemanticJudgment) :
    SemanticKind × CanonicalObjectId × List CanonicalObjectId :=
  (j.kind, j.subject, j.arguments)

theorem toTuple_injective : Function.Injective toTuple := by
  intro a b h
  cases a; cases b
  simp only [toTuple, Prod.mk.injEq] at h
  simp only [SemanticJudgment.mk.injEq]
  exact h

/-- Canonical prefix-free encoding. -/
def bytes (j : SemanticJudgment) : List UInt8 :=
  Bytes.pairBytes SemanticKind.bytes
    (Bytes.pairBytes CanonicalObjectId.bytes Enc.idList) j.toTuple

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  (Bytes.pairBytes_prefixFree SemanticKind.bytes_prefixFree
    (Bytes.pairBytes_prefixFree CanonicalObjectId.bytes_prefixFree
      Enc.idList_prefixFree)).comp toTuple_injective

theorem bytes_injective : Function.Injective bytes := bytes_prefixFree.injective

theorem bytes_head (j : SemanticJudgment) :
    bytes j = j.kind.tag ::
      (CanonicalObjectId.bytes j.subject ++ Enc.idList j.arguments) := by
  simp [bytes, Bytes.pairBytes, SemanticKind.bytes, toTuple]

/-- The frozen canonical schema of a semantic judgment (structural index 5). -/
def identitySchema : CanonicalSchema SemanticJudgment :=
  CanonicalSchema.ofPrefixFree 1 CanonicalDomainTag.atlasState
    (leafTag 5 "Atlas.SemanticJudgment") (leafTag_size_pos 5 _)
    bytes bytes_prefixFree

end SemanticJudgment

namespace OptimizerJudgment

def toTuple (j : OptimizerJudgment) :
    OptimizerKind × CanonicalObjectId × List CanonicalObjectId :=
  (j.kind, j.subject, j.arguments)

/-- Canonical encoding, in the same shape as `SemanticJudgment.bytes` — which is
the point: even in the same shape the two are never equal. -/
def bytes (j : OptimizerJudgment) : List UInt8 :=
  Bytes.pairBytes OptimizerKind.bytes
    (Bytes.pairBytes CanonicalObjectId.bytes Enc.idList) j.toTuple

theorem bytes_head (j : OptimizerJudgment) :
    bytes j = j.kind.tag ::
      (CanonicalObjectId.bytes j.subject ++ Enc.idList j.arguments) := by
  simp [bytes, Bytes.pairBytes, OptimizerKind.bytes, toTuple]

end OptimizerJudgment

/-- **No optimizer conclusion is a semantic judgment**, at the byte level. -/
theorem semanticJudgment_bytes_ne_optimizer
    (j : SemanticJudgment) (o : OptimizerJudgment) : j.bytes ≠ o.bytes := by
  rw [SemanticJudgment.bytes_head, OptimizerJudgment.bytes_head]
  intro h
  exact semanticKind_tag_ne_optimizerKind j.kind o.kind (List.cons.inj h).1

/-- The identity of a semantic judgment. -/
def semanticJudgmentId (j : SemanticJudgment) : CanonicalObjectId :=
  CanonicalObjectId.ofTyped (Identity SemanticJudgment.identitySchema j)

theorem semanticJudgmentId_eq_iff {a b : SemanticJudgment} :
    semanticJudgmentId a = semanticJudgmentId b ↔ a = b :=
  CanonicalObjectId.ofTyped_Identity_eq_iff SemanticJudgment.identitySchema

theorem semanticJudgmentId_injective : Function.Injective semanticJudgmentId :=
  fun _ _ h => semanticJudgmentId_eq_iff.mp h

/-! ## Atlas semantic closure -/

/-- A semantic inference rule.  **Its premises are `SemanticJudgment`s**: SPEC
§12.1's prohibition on optimizer premises is discharged by this type. -/
abbrev SemanticRule := Closure.Rule SemanticJudgment

/-- A semantic rule set. -/
abbrev SemanticRuleSet := List SemanticRule

/-- A semantic fact set. -/
abbrev SemanticFacts := Closure.FactSet SemanticJudgment

/-- **SPEC §12.1**: the least semantic closure of `A` under `R`. -/
def semanticClosure (R : SemanticRuleSet) (A : SemanticFacts) : SemanticFacts :=
  Closure.Cl R A

/-- The closure of a finite fact list, as a finite list. -/
def semanticClosureFacts (R : SemanticRuleSet) (A : List SemanticJudgment) :
    List SemanticJudgment := Closure.closureList R A

/-- The closure fact identities, in canonical judgment order. -/
def semanticClosureFactIds (R : SemanticRuleSet) (A : List SemanticJudgment) :
    List CanonicalObjectId := (semanticClosureFacts R A).map semanticJudgmentId

/-- The closure root of a computed semantic closure. -/
def semanticClosureRoot (R : SemanticRuleSet) (A : List SemanticJudgment) :
    ClosureRoot := ⟨semanticClosureFactIds R A⟩

theorem mem_semanticClosureFacts_iff (R : SemanticRuleSet) (A : List SemanticJudgment)
    (x : SemanticJudgment) :
    x ∈ semanticClosureFacts R A ↔ semanticClosure R (Closure.ofList A) x = true :=
  Closure.mem_closureList_iff R A x

/-! ### The four laws -/

/-- **Extensivity** (SPEC §12.1). -/
theorem semantic_closure_extensive (R : SemanticRuleSet) (A : SemanticFacts) :
    Closure.FactSub A (semanticClosure R A) := Closure.Cl_extensive R A

/-- **Monotonicity** (SPEC §12.1). -/
theorem semantic_closure_monotone (R : SemanticRuleSet) {A B : SemanticFacts}
    (h : Closure.FactSub A B) :
    Closure.FactSub (semanticClosure R A) (semanticClosure R B) :=
  Closure.Cl_monotone R h

/-- **Idempotence** (SPEC §12.1). -/
theorem semantic_closure_idempotent (R : SemanticRuleSet) (A : SemanticFacts) :
    semanticClosure R (semanticClosure R A) = semanticClosure R A :=
  Closure.Cl_idempotent R A

/-- **UOR-GNAF Theorem 11.3, the merge law**:
`Cl (A ∪ B) = Cl (Cl A ∪ Cl B)`. -/
theorem semantic_closure_merge (R : SemanticRuleSet) (A B : SemanticFacts) :
    semanticClosure R (Closure.factUnion A B) =
      semanticClosure R (Closure.factUnion (semanticClosure R A) (semanticClosure R B)) :=
  Closure.Cl_merge R A B

/-- The semantic closure is a fixed point of the immediate-consequence
operator. -/
theorem semantic_closure_fixed_point (R : SemanticRuleSet) (A : SemanticFacts) :
    Closure.step R (semanticClosure R A) = semanticClosure R A :=
  Closure.Cl_fixed_point R A

/--
**SPEC §12.1 — `Atlas.semantic_closure_least`.**

The semantic closure is the *least* set that is closed under the rule set and
contains `A`: it is closed, it contains `A`, and it is contained in every
predicate with those two properties.  The third component quantifies over an
arbitrary `Prop`-valued predicate, so no decidability side condition weakens it.

This is what SPEC §12.1 demands in place of a mere fixed point, and what
UOR-GNAF §18 lists as a non-claim to be discharged rather than assumed.
-/
theorem semantic_closure_least (R : SemanticRuleSet) (A : SemanticFacts) :
    Closure.ClosedFacts R (semanticClosure R A) ∧
    Closure.FactSub A (semanticClosure R A) ∧
    (∀ S : SemanticJudgment → Prop, Closure.ClosedUnder R S →
      (∀ x, A x = true → S x) → ∀ x, semanticClosure R A x = true → S x) :=
  ⟨Closure.Cl_closed R A, Closure.Cl_extensive R A, fun S hS hA => Closure.Cl_least R A S hS hA⟩

/-- The closure coincides with inductive derivability: the computed fixed point
**is** the derivation closure, not merely some fixed point above `A`. -/
theorem semantic_closure_eq_derivable (R : SemanticRuleSet) (A : SemanticFacts)
    (x : SemanticJudgment) :
    semanticClosure R A x = true ↔ Closure.Derivable R A x :=
  Closure.Cl_eq_derivable R A x

/-! ### Optimizer conclusions are not premises (SPEC §12.1) -/

/-- Every premise of every semantic rule differs, as canonical bytes, from every
optimizer conclusion.  The `SemanticJudgment` type makes this unavoidable; the
theorem checks that the typing survives canonical encoding. -/
theorem premise_ne_optimizer_conclusion (r : SemanticRule) (p : SemanticJudgment)
    (_hp : p ∈ r.premises) (o : OptimizerJudgment) : p.bytes ≠ o.bytes :=
  semanticJudgment_bytes_ne_optimizer p o

/-- No fact of any semantic closure is an optimizer conclusion. -/
theorem closure_fact_ne_optimizer_conclusion (R : SemanticRuleSet) (A : SemanticFacts)
    (x : SemanticJudgment) (_hx : semanticClosure R A x = true) (o : OptimizerJudgment) :
    x.bytes ≠ o.bytes :=
  semanticJudgment_bytes_ne_optimizer x o

/-- Optimizer verdicts cannot even be *named* as closure premises: a rule's
premise list has element type `SemanticJudgment`, whose kind is drawn from the
six semantic kinds. -/
theorem premise_kind_is_semantic (r : SemanticRule) (p : SemanticJudgment)
    (_hp : p ∈ r.premises) : p.kind ∈ SemanticKind.all :=
  SemanticKind.mem_all p.kind

/-! ## Anti-vacuity: what `closureLeastCheck` does *not* establish

`Atlas.closureLeastCheck` (Atlas/Certificate.lean) checks two structural
conditions on the recorded closure body: that the fact set is closed under the
recorded derivation edges, and that every fact carries a supporting edge whose
premises are facts.  Neither condition rules out *cyclic self-support*, so the
check is satisfied by fact sets that are strictly larger than the least closure.

The witness below is machine-checked. -/

/-- The two structural conditions of `closureLeastCheck`, isolated from the root
bookkeeping. -/
def closureBodySupportCheck (c : SemanticClosureBody) : Bool :=
  c.derivations.all (fun d =>
    !(subsetId d.premises c.facts) || memId c.facts d.conclusion) &&
  c.facts.all (fun f =>
    c.derivations.any (fun d =>
      decide (d.conclusion = f) && subsetId d.premises c.facts))

/-- `closureLeastCheck` is exactly root bookkeeping conjoined with the two
structural conditions. -/
theorem closureLeastCheck_eq_root_and_support (s : UnsealedState) (core : SealCore) :
    closureLeastCheck s core =
      (decide (core.closureRoot = s.body.semanticClosure.root) &&
        decide (s.body.semanticClosure.root.factIds = s.body.semanticClosure.facts) &&
        closureBodySupportCheck s.body.semanticClosure) := by
  simp only [closureLeastCheck, closureBodySupportCheck, Bool.and_assoc]

/-- The rule set a recorded derivation-edge list denotes. -/
def derivationRules (ds : List DerivationEdge) : List (Closure.Rule CanonicalObjectId) :=
  ds.map (fun d => ⟨d.premises, d.conclusion⟩)

/-- A concrete identifier used only by the anti-vacuity witness below. -/
def witnessId : CanonicalObjectId :=
  ⟨0, CanonicalDomainTag.generic, ByteArray.empty, ByteArray.empty⟩

/-- A recorded closure body whose single fact is supported only by itself. -/
def cyclicClosureBody : SemanticClosureBody where
  facts := [witnessId]
  derivations := [⟨witnessId, [witnessId], witnessId⟩]
  root := ⟨[witnessId]⟩

theorem cyclicClosureBody_passes : closureBodySupportCheck cyclicClosureBody = true := by
  simp [closureBodySupportCheck, cyclicClosureBody, subsetId, memId]

/-- Nothing is derivable from the empty base under the cyclic rule. -/
theorem cyclic_not_derivable (x : CanonicalObjectId) :
    ¬ Closure.Derivable (derivationRules cyclicClosureBody.derivations)
        (fun _ => false) x := by
  intro h
  induction h with
  | base hx => exact absurd hx (by simp)
  | @rule r hr _ ih =>
    simp only [derivationRules, cyclicClosureBody, List.map_cons, List.map_nil,
      List.mem_singleton] at hr
    subst hr
    exact ih witnessId (by simp)

/--
**Anti-vacuity scope lemma (SPEC §12.1, UOR-GNAF §18).**

The structural conditions of `closureLeastCheck` accept a recorded fact that is
**not** in the least closure of the empty base under the recorded derivation
edges.  A passing `closureLeastCheck` therefore witnesses *a* fixed point, never
*the least* one; leastness must be established by
`Atlas.semantic_closure_least` for the rule set that produced the facts.
-/
theorem closureBodySupportCheck_accepts_underivable :
    ∃ c : SemanticClosureBody,
      closureBodySupportCheck c = true ∧
      ∃ f ∈ c.facts,
        Closure.Cl (derivationRules c.derivations) (fun _ => false) f = false := by
  refine ⟨cyclicClosureBody, cyclicClosureBody_passes, witnessId, by simp [cyclicClosureBody], ?_⟩
  by_cases h : Closure.Cl (derivationRules cyclicClosureBody.derivations)
      (fun _ => false) witnessId = true
  · exact absurd (Closure.Cl_sound _ _ _ h) (cyclic_not_derivable witnessId)
  · simpa using h

end WasmGemmGnaf.Atlas
