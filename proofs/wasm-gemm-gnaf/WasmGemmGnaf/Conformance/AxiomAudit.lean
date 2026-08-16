import WasmGemmGnaf.Conformance.Claim
set_option autoImplicit false

/-!
# Conformance: the permitted trust base (SPEC §4)

SPEC §4 fixes the disclosed logical trust base and forbids the project from
containing or depending on the placeholder axiom behind an unfinished proof, on
project-declared axioms, on an FFI truth oracle, on kernel-bypassing
evaluation, or on an assumed compiler-correctness proposition.
`Classical.choice` SHALL NOT produce executable witnesses, and any
unexpected entry in the collected set makes the release gate fail.

`PermittedAxiom` is the closed set of the three Lean core logical axioms that
transitive collection may report.  `CollectedAxiom` is what a collector can
actually report: one of those three, the placeholder axiom, a project-declared
axiom, or any other foreign axiom.  `AxiomSetAcceptable` is decidable, and the
rejection of the placeholder and of *every* project axiom is proved.
-/

namespace WasmGemmGnaf.Conformance

open WasmGemmGnaf.Foundation

/-! ## The permitted axioms -/

/-- The three Lean core logical axioms the trust base may disclose. -/
inductive PermittedAxiom
  | propext
  | quotSound
  | classicalChoice
  deriving DecidableEq, Repr, Inhabited

namespace PermittedAxiom

/-- The complete enumeration of permitted axioms. -/
def all : List PermittedAxiom := [propext, quotSound, classicalChoice]

theorem mem_all (p : PermittedAxiom) : p ∈ all := by cases p <;> simp [all]

theorem all_nodup : all.Nodup := by decide

theorem all_length : all.length = 3 := rfl

instance : Fintype PermittedAxiom where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

/-- The Lean name of a permitted axiom. -/
def name : PermittedAxiom → String
  | propext => "propext"
  | quotSound => "Quot.sound"
  | classicalChoice => "Classical.choice"

theorem name_injective : Function.Injective name := by
  intro a b h
  cases a <;> cases b <;> first
    | rfl
    | exact absurd h (by decide)

/-- `Classical.choice` is the only permitted axiom that is not computable;
SPEC §4 forbids it from producing executable witnesses. -/
def isChoice : PermittedAxiom → Bool
  | classicalChoice => true
  | _ => false

theorem isChoice_iff (p : PermittedAxiom) : isChoice p = true ↔ p = classicalChoice := by
  cases p <;> simp [isChoice]

end PermittedAxiom

/-! ## Collected axioms -/

/-- An axiom name as reported by transitive axiom collection. -/
inductive CollectedAxiom
  /-- One of the three permitted Lean core axioms. -/
  | permitted (p : PermittedAxiom)
  /-- The placeholder axiom Lean records for an unfinished proof. -/
  | sorryAx
  /-- An axiom declared by this project. -/
  | projectAxiom (name : String)
  /-- Any other axiom: a dependency's axiom, an FFI oracle, an assumed
  compiler-correctness proposition. -/
  | foreignAxiom (name : String)
  deriving DecidableEq, Repr, Inhabited

namespace CollectedAxiom

/-- The reported Lean name. -/
def name : CollectedAxiom → String
  | permitted p => p.name
  | sorryAx => "sorryAx"
  | projectAxiom n => n
  | foreignAxiom n => n

/-- Whether a single collected axiom is inside the disclosed trust base. -/
def isPermitted : CollectedAxiom → Bool
  | permitted _ => true
  | _ => false

/-- The Prop-level form. -/
def IsPermitted (a : CollectedAxiom) : Prop := ∃ p : PermittedAxiom, a = permitted p

instance : DecidablePred IsPermitted := fun a =>
  match a with
  | permitted p => isTrue ⟨p, rfl⟩
  | sorryAx => isFalse (fun ⟨_, h⟩ => by cases h)
  | projectAxiom _ => isFalse (fun ⟨_, h⟩ => by cases h)
  | foreignAxiom _ => isFalse (fun ⟨_, h⟩ => by cases h)

theorem isPermitted_iff (a : CollectedAxiom) : isPermitted a = true ↔ IsPermitted a := by
  cases a <;> simp [isPermitted, IsPermitted]

/-- `sorryAx` is never permitted: a placeholder proof is not a trust base. -/
theorem not_isPermitted_sorryAx : ¬ IsPermitted sorryAx := by
  intro ⟨_, h⟩; cases h

theorem isPermitted_sorryAx_false : isPermitted sorryAx = false := rfl

/-- **No project-declared axiom is ever permitted**, whatever its name. -/
theorem not_isPermitted_projectAxiom (n : String) : ¬ IsPermitted (projectAxiom n) := by
  intro ⟨_, h⟩; cases h

theorem isPermitted_projectAxiom_false (n : String) :
    isPermitted (projectAxiom n) = false := rfl

/-- No foreign axiom — dependency axiom, FFI oracle, assumed compiler
correctness — is ever permitted. -/
theorem not_isPermitted_foreignAxiom (n : String) : ¬ IsPermitted (foreignAxiom n) := by
  intro ⟨_, h⟩; cases h

theorem isPermitted_foreignAxiom_false (n : String) :
    isPermitted (foreignAxiom n) = false := rfl

/-- Whether the axiom would make a witness non-executable (SPEC §4:
`Classical.choice` SHALL NOT produce executable witnesses). -/
def isChoice : CollectedAxiom → Bool
  | permitted p => p.isChoice
  | _ => false

end CollectedAxiom

/-! ## Acceptable axiom sets -/

/-- The decidable audit of a collected axiom set. -/
def axiomSetAcceptableB (l : List CollectedAxiom) : Bool :=
  l.all CollectedAxiom.isPermitted

/-- A collected axiom set is acceptable when every name in it is one of the
three disclosed Lean core axioms. -/
def AxiomSetAcceptable (l : List CollectedAxiom) : Prop :=
  ∀ a ∈ l, CollectedAxiom.IsPermitted a

instance : DecidablePred AxiomSetAcceptable := fun l =>
  decidable_of_iff (axiomSetAcceptableB l = true) (by
    simp [axiomSetAcceptableB, AxiomSetAcceptable, List.all_eq_true,
      CollectedAxiom.isPermitted_iff])

theorem axiomSetAcceptableB_iff (l : List CollectedAxiom) :
    axiomSetAcceptableB l = true ↔ AxiomSetAcceptable l := by
  simp [axiomSetAcceptableB, AxiomSetAcceptable, List.all_eq_true,
    CollectedAxiom.isPermitted_iff]

/-- The empty axiom set — an axiom-free declaration — is acceptable. -/
theorem axiomSetAcceptable_nil : AxiomSetAcceptable [] := by
  intro a ha; cases ha

/-- Any set of permitted axioms is acceptable. -/
theorem axiomSetAcceptable_permitted (l : List PermittedAxiom) :
    AxiomSetAcceptable (l.map CollectedAxiom.permitted) := by
  intro a ha
  obtain ⟨p, _, rfl⟩ := List.mem_map.mp ha
  exact ⟨p, rfl⟩

/-- **An unpermitted axiom anywhere in the set fails the audit.** -/
theorem not_axiomSetAcceptable_of_mem {l : List CollectedAxiom} {a : CollectedAxiom}
    (ha : a ∈ l) (hn : ¬ CollectedAxiom.IsPermitted a) : ¬ AxiomSetAcceptable l :=
  fun h => hn (h a ha)

theorem axiomSetAcceptableB_eq_false_of_mem {l : List CollectedAxiom}
    {a : CollectedAxiom} (ha : a ∈ l) (hn : ¬ CollectedAxiom.IsPermitted a) :
    axiomSetAcceptableB l = false := by
  cases h : axiomSetAcceptableB l with
  | false => rfl
  | true => exact absurd ((axiomSetAcceptableB_iff l).mp h) (not_axiomSetAcceptable_of_mem ha hn)

/-- **`sorryAx` is rejected.** -/
theorem sorryAx_rejected {l : List CollectedAxiom} (h : CollectedAxiom.sorryAx ∈ l) :
    ¬ AxiomSetAcceptable l :=
  not_axiomSetAcceptable_of_mem h CollectedAxiom.not_isPermitted_sorryAx

theorem sorryAx_rejectedB {l : List CollectedAxiom} (h : CollectedAxiom.sorryAx ∈ l) :
    axiomSetAcceptableB l = false :=
  axiomSetAcceptableB_eq_false_of_mem h CollectedAxiom.not_isPermitted_sorryAx

/-- **A project-declared axiom is rejected**, whatever its name. -/
theorem projectAxiom_rejected {l : List CollectedAxiom} {n : String}
    (h : CollectedAxiom.projectAxiom n ∈ l) : ¬ AxiomSetAcceptable l :=
  not_axiomSetAcceptable_of_mem h (CollectedAxiom.not_isPermitted_projectAxiom n)

theorem projectAxiom_rejectedB {l : List CollectedAxiom} {n : String}
    (h : CollectedAxiom.projectAxiom n ∈ l) : axiomSetAcceptableB l = false :=
  axiomSetAcceptableB_eq_false_of_mem h (CollectedAxiom.not_isPermitted_projectAxiom n)

/-- A foreign axiom (FFI oracle, assumed compiler correctness, dependency
axiom) is rejected. -/
theorem foreignAxiom_rejected {l : List CollectedAxiom} {n : String}
    (h : CollectedAxiom.foreignAxiom n ∈ l) : ¬ AxiomSetAcceptable l :=
  not_axiomSetAcceptable_of_mem h (CollectedAxiom.not_isPermitted_foreignAxiom n)

/-- The singleton `sorryAx` audit fails outright. -/
theorem axiomSetAcceptableB_sorryAx :
    axiomSetAcceptableB [CollectedAxiom.sorryAx] = false := rfl

/-- The three-axiom disclosed trust base passes. -/
theorem axiomSetAcceptableB_trustBase :
    axiomSetAcceptableB (PermittedAxiom.all.map CollectedAxiom.permitted) = true := rfl

/-! ## Executable witnesses (SPEC §4)

"`Classical.choice` SHALL NOT produce executable witnesses." -/

/-- The stricter audit for a declaration that must produce an executable
witness: acceptable *and* choice-free. -/
def executableAxiomSetAcceptableB (l : List CollectedAxiom) : Bool :=
  axiomSetAcceptableB l && !(l.any CollectedAxiom.isChoice)

theorem executableAxiomSetAcceptableB_iff (l : List CollectedAxiom) :
    executableAxiomSetAcceptableB l = true ↔
      (AxiomSetAcceptable l ∧ ∀ a ∈ l, CollectedAxiom.isChoice a = false) := by
  simp only [executableAxiomSetAcceptableB, Bool.and_eq_true, Bool.not_eq_true',
    axiomSetAcceptableB_iff]
  constructor
  · intro ⟨h1, h2⟩
    refine ⟨h1, ?_⟩
    intro a ha
    cases hc : CollectedAxiom.isChoice a with
    | false => rfl
    | true =>
      have : l.any CollectedAxiom.isChoice = true := List.any_eq_true.mpr ⟨a, ha, hc⟩
      rw [h2] at this; cases this
  · intro ⟨h1, h2⟩
    refine ⟨h1, ?_⟩
    cases hany : l.any CollectedAxiom.isChoice with
    | false => rfl
    | true =>
      obtain ⟨a, ha, hc⟩ := List.any_eq_true.mp hany
      rw [h2 a ha] at hc; cases hc

/-- An executable witness may not depend on `Classical.choice`. -/
theorem classicalChoice_rejected_for_executable {l : List CollectedAxiom}
    (h : CollectedAxiom.permitted .classicalChoice ∈ l) :
    executableAxiomSetAcceptableB l = false := by
  cases he : executableAxiomSetAcceptableB l with
  | false => rfl
  | true =>
    have := ((executableAxiomSetAcceptableB_iff l).mp he).2 _ h
    cases this

/-- `sorryAx` is rejected for executable witnesses too. -/
theorem sorryAx_rejected_for_executable {l : List CollectedAxiom}
    (h : CollectedAxiom.sorryAx ∈ l) : executableAxiomSetAcceptableB l = false := by
  simp [executableAxiomSetAcceptableB, sorryAx_rejectedB h]

end WasmGemmGnaf.Conformance
