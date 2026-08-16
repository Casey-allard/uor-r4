import WasmGemmGnaf.Conformance.Registry
import WasmGemmGnaf.Conformance.Manifest
import WasmGemmGnaf.Conformance.AxiomAudit
set_option autoImplicit false

/-!
# Conformance: the release gate (SPEC §20.2)

SPEC §20.2: "From a clean checkout with network disabled, `just vv` SHALL fail
unless all of the following hold", followed by thirteen numbered conditions.

`ReleaseGate` is the first-order record with exactly one Bool field per
numbered condition; `GateCondition` is the closed index type of the thirteen
conditions; `passes` is their decidable conjunction.

The anti-vacuity property is `passes_iff_holds` / `passes_iff_forall`: the gate
is `true` **exactly** when every one of the thirteen holds.  Together with
`condition_necessary` (any single false condition fails the gate) and
`allTrue_passes` (the gate is not constantly false), this rules out a gate that
is satisfied for the wrong reason.
-/

namespace WasmGemmGnaf.Conformance

open WasmGemmGnaf.Foundation

/-! ## The thirteen conditions -/

/-- The thirteen numbered release-gate conditions of SPEC §20.2. -/
inductive GateCondition
  /-- 1. Toolchain, authority, handwritten source, generated proof-input,
  pre-final declaration-environment and dependency identities match their
  ordered acyclic manifests. -/
  | manifestIdentitiesMatch
  /-- 2. The required claim graph is nonempty, acyclic and complete. -/
  | claimGraphComplete
  /-- 3. Lean builds with no placeholder or unexpected axiom. -/
  | buildAxiomClean
  /-- 4. The concrete WebAssembly and GEMM semantics are built. -/
  | concreteSemanticsBuilt
  /-- 5. Universal sublevel and outside-sublevel coverage are proved. -/
  | sublevelCoverageProved
  /-- 6. The committed artifact exists and its bytes match the proved value. -/
  | artifactBytesMatch
  /-- 7. Artifact decode, validation, ABI, correctness, resource and cost
  theorems hold. -/
  | artifactTheoremsHold
  /-- 8. Universal lower bound and attainment theorems hold. -/
  | lowerBoundAndAttainment
  /-- 9. `released_wasm_gemm_gnaf_global_optimal` exists with the exact recorded
  proposition and accepted axiom set. -/
  | finalTheoremRecorded
  /-- 10. The Atlas seal reconstructs from retained canonical objects. -/
  | atlasSealReconstructs
  /-- 11. Mutation suites demonstrate that each decisive gate rejects a planted
  fault. -/
  | mutationSuitesReject
  /-- 12. Two clean emissions produce byte-identical artifact, seal, manifest
  and generated documentation. -/
  | doubleEmissionIdentical
  /-- 13. The worktree remains clean after verification. -/
  | worktreeClean
  deriving DecidableEq, Repr, Inhabited

namespace GateCondition

/-- The complete enumeration of gate conditions, in SPEC order. -/
def all : List GateCondition :=
  [manifestIdentitiesMatch, claimGraphComplete, buildAxiomClean,
   concreteSemanticsBuilt, sublevelCoverageProved, artifactBytesMatch,
   artifactTheoremsHold, lowerBoundAndAttainment, finalTheoremRecorded,
   atlasSealReconstructs, mutationSuitesReject, doubleEmissionIdentical,
   worktreeClean]

theorem mem_all (c : GateCondition) : c ∈ all := by cases c <;> simp [all]

theorem all_nodup : all.Nodup := by decide

/-- **There are exactly thirteen conditions.** -/
theorem all_length : all.length = 13 := rfl

instance : Fintype GateCondition where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

/-- The SPEC §20.2 number of each condition. -/
def number : GateCondition → Nat
  | manifestIdentitiesMatch => 1
  | claimGraphComplete => 2
  | buildAxiomClean => 3
  | concreteSemanticsBuilt => 4
  | sublevelCoverageProved => 5
  | artifactBytesMatch => 6
  | artifactTheoremsHold => 7
  | lowerBoundAndAttainment => 8
  | finalTheoremRecorded => 9
  | atlasSealReconstructs => 10
  | mutationSuitesReject => 11
  | doubleEmissionIdentical => 12
  | worktreeClean => 13

theorem number_injective : Function.Injective number := by
  intro a b h
  cases a <;> cases b <;> simp_all [number]

theorem number_pos (c : GateCondition) : 0 < c.number := by cases c <;> decide

theorem number_le_thirteen (c : GateCondition) : c.number ≤ 13 := by cases c <;> decide

end GateCondition

/-! ## The gate record -/

/-- The release gate of SPEC §20.2: one Bool per numbered condition. -/
structure ReleaseGate where
  /-- 1. Ordered acyclic manifest identities match. -/
  manifestIdentitiesMatch : Bool
  /-- 2. Claim graph nonempty, acyclic and complete. -/
  claimGraphComplete : Bool
  /-- 3. No placeholder or unexpected axiom. -/
  buildAxiomClean : Bool
  /-- 4. Concrete WebAssembly and GEMM semantics built. -/
  concreteSemanticsBuilt : Bool
  /-- 5. Sublevel and outside-sublevel coverage proved. -/
  sublevelCoverageProved : Bool
  /-- 6. Committed artifact bytes match the proved value. -/
  artifactBytesMatch : Bool
  /-- 7. Artifact decode/validation/ABI/correctness/resource/cost theorems. -/
  artifactTheoremsHold : Bool
  /-- 8. Universal lower bound and attainment. -/
  lowerBoundAndAttainment : Bool
  /-- 9. Final theorem present with the exact proposition and axiom set. -/
  finalTheoremRecorded : Bool
  /-- 10. Atlas seal reconstructs. -/
  atlasSealReconstructs : Bool
  /-- 11. Mutation suites reject planted faults. -/
  mutationSuitesReject : Bool
  /-- 12. Two clean emissions are byte-identical. -/
  doubleEmissionIdentical : Bool
  /-- 13. Worktree clean after verification. -/
  worktreeClean : Bool
  deriving DecidableEq, Repr

namespace GateCondition

/-- The value of a numbered condition in a gate. -/
def holds : GateCondition → ReleaseGate → Bool
  | manifestIdentitiesMatch, g => g.manifestIdentitiesMatch
  | claimGraphComplete, g => g.claimGraphComplete
  | buildAxiomClean, g => g.buildAxiomClean
  | concreteSemanticsBuilt, g => g.concreteSemanticsBuilt
  | sublevelCoverageProved, g => g.sublevelCoverageProved
  | artifactBytesMatch, g => g.artifactBytesMatch
  | artifactTheoremsHold, g => g.artifactTheoremsHold
  | lowerBoundAndAttainment, g => g.lowerBoundAndAttainment
  | finalTheoremRecorded, g => g.finalTheoremRecorded
  | atlasSealReconstructs, g => g.atlasSealReconstructs
  | mutationSuitesReject, g => g.mutationSuitesReject
  | doubleEmissionIdentical, g => g.doubleEmissionIdentical
  | worktreeClean, g => g.worktreeClean

end GateCondition

namespace ReleaseGate

/-- The thirteen condition values, in SPEC order. -/
def conditionValues (g : ReleaseGate) : List Bool :=
  GateCondition.all.map (fun c => c.holds g)

theorem conditionValues_length (g : ReleaseGate) : (g.conditionValues).length = 13 := rfl

/-- The decidable conjunction of the thirteen conditions. -/
def passes (g : ReleaseGate) : Bool :=
  g.manifestIdentitiesMatch && g.claimGraphComplete && g.buildAxiomClean &&
  g.concreteSemanticsBuilt && g.sublevelCoverageProved && g.artifactBytesMatch &&
  g.artifactTheoremsHold && g.lowerBoundAndAttainment && g.finalTheoremRecorded &&
  g.atlasSealReconstructs && g.mutationSuitesReject && g.doubleEmissionIdentical &&
  g.worktreeClean

/-- The thirteen conditions as a proposition. -/
def Holds (g : ReleaseGate) : Prop :=
  g.manifestIdentitiesMatch = true ∧
  g.claimGraphComplete = true ∧
  g.buildAxiomClean = true ∧
  g.concreteSemanticsBuilt = true ∧
  g.sublevelCoverageProved = true ∧
  g.artifactBytesMatch = true ∧
  g.artifactTheoremsHold = true ∧
  g.lowerBoundAndAttainment = true ∧
  g.finalTheoremRecorded = true ∧
  g.atlasSealReconstructs = true ∧
  g.mutationSuitesReject = true ∧
  g.doubleEmissionIdentical = true ∧
  g.worktreeClean = true

/-- **The anti-vacuity property: the gate passes exactly when all thirteen
conditions hold.** -/
theorem passes_iff_holds (g : ReleaseGate) : g.passes = true ↔ g.Holds := by
  simp [passes, Holds, Bool.and_eq_true, and_assoc]

instance : DecidablePred Holds := fun g =>
  decidable_of_iff (g.passes = true) (passes_iff_holds g)

/-- The same statement, quantified over the closed index type of conditions. -/
theorem passes_iff_forall (g : ReleaseGate) :
    g.passes = true ↔ ∀ c : GateCondition, c.holds g = true := by
  rw [passes_iff_holds]
  constructor
  · intro h c
    obtain ⟨h1, h2, h3, h4, h5, h6, h7, h8, h9, h10, h11, h12, h13⟩ := h
    cases c <;> assumption
  · intro h
    exact ⟨h .manifestIdentitiesMatch, h .claimGraphComplete, h .buildAxiomClean,
      h .concreteSemanticsBuilt, h .sublevelCoverageProved, h .artifactBytesMatch,
      h .artifactTheoremsHold, h .lowerBoundAndAttainment, h .finalTheoremRecorded,
      h .atlasSealReconstructs, h .mutationSuitesReject, h .doubleEmissionIdentical,
      h .worktreeClean⟩

/-- The same statement, over the list of condition values. -/
theorem passes_iff_all_values (g : ReleaseGate) :
    g.passes = true ↔ ∀ b ∈ g.conditionValues, b = true := by
  rw [passes_iff_forall]
  constructor
  · intro h b hb
    obtain ⟨c, _, rfl⟩ := List.mem_map.mp hb
    exact h c
  · intro h c
    exact h _ (List.mem_map_of_mem (GateCondition.mem_all c))

/-- **Necessity: a single false condition fails the gate.** -/
theorem condition_necessary {g : ReleaseGate} {c : GateCondition}
    (h : c.holds g = false) : g.passes = false := by
  cases hp : g.passes with
  | false => rfl
  | true =>
    have := (passes_iff_forall g).mp hp c
    rw [h] at this
    cases this

/-! ### The thirteen necessity lemmas -/

theorem necessary_manifestIdentitiesMatch {g : ReleaseGate}
    (h : g.manifestIdentitiesMatch = false) : g.passes = false :=
  condition_necessary (c := .manifestIdentitiesMatch) h

theorem necessary_claimGraphComplete {g : ReleaseGate}
    (h : g.claimGraphComplete = false) : g.passes = false :=
  condition_necessary (c := .claimGraphComplete) h

theorem necessary_buildAxiomClean {g : ReleaseGate}
    (h : g.buildAxiomClean = false) : g.passes = false :=
  condition_necessary (c := .buildAxiomClean) h

theorem necessary_concreteSemanticsBuilt {g : ReleaseGate}
    (h : g.concreteSemanticsBuilt = false) : g.passes = false :=
  condition_necessary (c := .concreteSemanticsBuilt) h

theorem necessary_sublevelCoverageProved {g : ReleaseGate}
    (h : g.sublevelCoverageProved = false) : g.passes = false :=
  condition_necessary (c := .sublevelCoverageProved) h

theorem necessary_artifactBytesMatch {g : ReleaseGate}
    (h : g.artifactBytesMatch = false) : g.passes = false :=
  condition_necessary (c := .artifactBytesMatch) h

theorem necessary_artifactTheoremsHold {g : ReleaseGate}
    (h : g.artifactTheoremsHold = false) : g.passes = false :=
  condition_necessary (c := .artifactTheoremsHold) h

theorem necessary_lowerBoundAndAttainment {g : ReleaseGate}
    (h : g.lowerBoundAndAttainment = false) : g.passes = false :=
  condition_necessary (c := .lowerBoundAndAttainment) h

theorem necessary_finalTheoremRecorded {g : ReleaseGate}
    (h : g.finalTheoremRecorded = false) : g.passes = false :=
  condition_necessary (c := .finalTheoremRecorded) h

theorem necessary_atlasSealReconstructs {g : ReleaseGate}
    (h : g.atlasSealReconstructs = false) : g.passes = false :=
  condition_necessary (c := .atlasSealReconstructs) h

theorem necessary_mutationSuitesReject {g : ReleaseGate}
    (h : g.mutationSuitesReject = false) : g.passes = false :=
  condition_necessary (c := .mutationSuitesReject) h

theorem necessary_doubleEmissionIdentical {g : ReleaseGate}
    (h : g.doubleEmissionIdentical = false) : g.passes = false :=
  condition_necessary (c := .doubleEmissionIdentical) h

theorem necessary_worktreeClean {g : ReleaseGate}
    (h : g.worktreeClean = false) : g.passes = false :=
  condition_necessary (c := .worktreeClean) h

/-! ### Non-vacuity -/

/-- The gate with every condition discharged. -/
def allTrue : ReleaseGate where
  manifestIdentitiesMatch := true
  claimGraphComplete := true
  buildAxiomClean := true
  concreteSemanticsBuilt := true
  sublevelCoverageProved := true
  artifactBytesMatch := true
  artifactTheoremsHold := true
  lowerBoundAndAttainment := true
  finalTheoremRecorded := true
  atlasSealReconstructs := true
  mutationSuitesReject := true
  doubleEmissionIdentical := true
  worktreeClean := true

/-- The gate with no condition discharged. -/
def allFalse : ReleaseGate where
  manifestIdentitiesMatch := false
  claimGraphComplete := false
  buildAxiomClean := false
  concreteSemanticsBuilt := false
  sublevelCoverageProved := false
  artifactBytesMatch := false
  artifactTheoremsHold := false
  lowerBoundAndAttainment := false
  finalTheoremRecorded := false
  atlasSealReconstructs := false
  mutationSuitesReject := false
  doubleEmissionIdentical := false
  worktreeClean := false

/-- The gate is satisfiable: it is not constantly false. -/
theorem allTrue_passes : allTrue.passes = true := rfl

/-- The gate is not constantly true. -/
theorem allFalse_fails : allFalse.passes = false := rfl

/-- Clear exactly one condition of a gate. -/
def setFalse : GateCondition → ReleaseGate → ReleaseGate
  | .manifestIdentitiesMatch, g => { g with manifestIdentitiesMatch := false }
  | .claimGraphComplete, g => { g with claimGraphComplete := false }
  | .buildAxiomClean, g => { g with buildAxiomClean := false }
  | .concreteSemanticsBuilt, g => { g with concreteSemanticsBuilt := false }
  | .sublevelCoverageProved, g => { g with sublevelCoverageProved := false }
  | .artifactBytesMatch, g => { g with artifactBytesMatch := false }
  | .artifactTheoremsHold, g => { g with artifactTheoremsHold := false }
  | .lowerBoundAndAttainment, g => { g with lowerBoundAndAttainment := false }
  | .finalTheoremRecorded, g => { g with finalTheoremRecorded := false }
  | .atlasSealReconstructs, g => { g with atlasSealReconstructs := false }
  | .mutationSuitesReject, g => { g with mutationSuitesReject := false }
  | .doubleEmissionIdentical, g => { g with doubleEmissionIdentical := false }
  | .worktreeClean, g => { g with worktreeClean := false }

theorem holds_setFalse (c : GateCondition) (g : ReleaseGate) :
    c.holds (setFalse c g) = false := by
  cases c <;> rfl

theorem holds_setFalse_of_ne (c d : GateCondition) (g : ReleaseGate) (h : d ≠ c) :
    d.holds (setFalse c g) = d.holds g := by
  cases c <;> cases d <;> first | rfl | exact absurd rfl h

/-- **Every one of the thirteen conditions is individually load-bearing**:
clearing exactly one condition of a fully discharged gate — leaving the other
twelve true — makes the release gate fail. -/
theorem condition_load_bearing (c : GateCondition) :
    (setFalse c allTrue).passes = false ∧
      (∀ d : GateCondition, d ≠ c → d.holds (setFalse c allTrue) = true) := by
  refine ⟨condition_necessary (holds_setFalse c allTrue), ?_⟩
  intro d hd
  rw [holds_setFalse_of_ne c d allTrue hd]
  cases d <;> rfl

/-! ### Connecting the gate to the checked models -/

/-- Condition 2 of SPEC §20.2 is discharged by a well-formed registry: nonempty,
acyclic and complete (no orphan dependency, unique IDs). -/
def claimGraphConditionOf (reg : ClaimRegistry) : Bool := reg.wellFormedB

theorem claimGraphConditionOf_acyclic {reg : ClaimRegistry}
    (h : claimGraphConditionOf reg = true) : ClaimRegistry.Acyclic reg :=
  ClaimRegistry.wellFormed_acyclic h

/-- Condition 3 of SPEC §20.2 is discharged by an acceptable transitive axiom
set. -/
def axiomConditionOf (l : List CollectedAxiom) : Bool := axiomSetAcceptableB l

/-- A planted placeholder axiom fails condition 3, hence the whole gate. -/
theorem placeholderAxiom_fails_gate {g : ReleaseGate} {l : List CollectedAxiom}
    (hlink : g.buildAxiomClean = axiomConditionOf l)
    (hsorry : CollectedAxiom.sorryAx ∈ l) : g.passes = false := by
  refine necessary_buildAxiomClean ?_
  rw [hlink, axiomConditionOf]
  exact sorryAx_rejectedB hsorry

/-- A project-declared axiom fails condition 3, hence the whole gate. -/
theorem projectAxiom_fails_gate {g : ReleaseGate} {l : List CollectedAxiom}
    {n : String} (hlink : g.buildAxiomClean = axiomConditionOf l)
    (haxiom : CollectedAxiom.projectAxiom n ∈ l) : g.passes = false := by
  refine necessary_buildAxiomClean ?_
  rw [hlink, axiomConditionOf]
  exact projectAxiom_rejectedB haxiom

/-- A registry with a duplicate claim ID fails condition 2, hence the whole
gate. -/
theorem duplicate_claim_fails_gate {g : ReleaseGate} {reg : ClaimRegistry}
    {r s : ClaimRow} (hlink : g.claimGraphComplete = claimGraphConditionOf reg)
    (hr : r ∈ reg.rows) (hs : s ∈ reg.rows) (hne : r ≠ s) (hid : r.id = s.id) :
    g.passes = false := by
  refine necessary_claimGraphComplete ?_
  rw [hlink, claimGraphConditionOf]
  exact ClaimRegistry.duplicate_id_rejected hr hs hne hid

end ReleaseGate

end WasmGemmGnaf.Conformance
