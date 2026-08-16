import WasmGemmGnaf.Foundation.Identity
import WasmGemmGnaf.Foundation.Finite
set_option autoImplicit false

/-!
# Conformance: claim levels and claim rows (SPEC §17.1, §17.2)

SPEC §17.1 fixes exactly five claim levels and states the honesty rule: *only*
`formalProof` supports words such as "proved", "theorem", or "globally
optimal".  That rule is mechanised here as `SupportsProofLanguage`, a predicate
that holds for `formalProof` and is *proved* to fail for each of the other four
levels — tests, benchmarks, citations and measurements cannot promote a claim.

SPEC §17.2 fixes the fields of a row of `model/claims.json`.  `ClaimRow` is a
first-order record with one field per listed item; every field is data a
conformance tool can recompute or compare, and no field is a proposition.
-/

namespace WasmGemmGnaf.Conformance

open WasmGemmGnaf.Foundation

/-! ## Claim levels (SPEC §17.1) -/

/-- The five claim levels of SPEC §17.1. -/
inductive ClaimLevel
  | authority
  | buildEvidence
  | formalProof
  | measurement
  | «open»
  deriving DecidableEq, Repr, Inhabited

namespace ClaimLevel

/-- The complete enumeration of claim levels. -/
def all : List ClaimLevel :=
  [authority, buildEvidence, formalProof, measurement, «open»]

theorem mem_all (l : ClaimLevel) : l ∈ all := by cases l <;> simp [all]

theorem all_nodup : all.Nodup := by decide

theorem all_length : all.length = 5 := rfl

instance : Fintype ClaimLevel where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

/-- Structural index of a claim level. -/
def index : ClaimLevel → Nat
  | authority => 0
  | buildEvidence => 1
  | formalProof => 2
  | measurement => 3
  | «open» => 4

theorem index_injective : Function.Injective index := by
  intro a b h
  cases a <;> cases b <;> simp_all [index]

end ClaimLevel

/-! ## The honesty rule (SPEC §17.1)

"Only `formalProof` supports words such as *proved*, *theorem*, or *globally
optimal*.  Tests, benchmarks, external citations, and measurements cannot
promote a claim to that level." -/

/-- `SupportsProofLanguage l` holds exactly when a claim at level `l` may be
stated with proof language. -/
def SupportsProofLanguage : ClaimLevel → Prop
  | .formalProof => True
  | _ => False

/-- `formalProof` supports proof language. -/
theorem supportsProofLanguage_formalProof :
    SupportsProofLanguage .formalProof := trivial

/-- An authority citation is not a proof. -/
theorem not_supportsProofLanguage_authority :
    ¬ SupportsProofLanguage .authority := fun h => h

/-- Build evidence is not a proof. -/
theorem not_supportsProofLanguage_buildEvidence :
    ¬ SupportsProofLanguage .buildEvidence := fun h => h

/-- A measurement is not a proof. -/
theorem not_supportsProofLanguage_measurement :
    ¬ SupportsProofLanguage .measurement := fun h => h

/-- An open question is not a proof. -/
theorem not_supportsProofLanguage_open :
    ¬ SupportsProofLanguage ClaimLevel.open := fun h => h

/-- The predicate holds at exactly one level. -/
theorem supportsProofLanguage_iff (l : ClaimLevel) :
    SupportsProofLanguage l ↔ l = .formalProof := by
  cases l <;> simp [SupportsProofLanguage]

/-- No level other than `formalProof` supports proof language. -/
theorem not_supportsProofLanguage_of_ne {l : ClaimLevel} (h : l ≠ .formalProof) :
    ¬ SupportsProofLanguage l := fun hs => h ((supportsProofLanguage_iff l).mp hs)

/-- Two levels that both support proof language are equal: the level is unique. -/
theorem supportsProofLanguage_unique {l m : ClaimLevel}
    (hl : SupportsProofLanguage l) (hm : SupportsProofLanguage m) : l = m := by
  rw [(supportsProofLanguage_iff l).mp hl, (supportsProofLanguage_iff m).mp hm]

/-- The decidable form of the honesty rule. -/
def supportsProofLanguageB : ClaimLevel → Bool
  | .formalProof => true
  | _ => false

theorem supportsProofLanguageB_iff (l : ClaimLevel) :
    supportsProofLanguageB l = true ↔ SupportsProofLanguage l := by
  cases l <;> simp [supportsProofLanguageB, SupportsProofLanguage]

instance : DecidablePred SupportsProofLanguage := fun l =>
  decidable_of_iff _ (supportsProofLanguageB_iff l)

/-- Exactly one of the five levels satisfies the decidable form. -/
theorem supportsProofLanguageB_filter :
    ClaimLevel.all.filter supportsProofLanguageB = [ClaimLevel.formalProof] := by
  decide

/-! ## Claim identifiers -/

/-- A claim identifier: a family tag (`"GO"`, `"UV"`, …, see SPEC §17.3) and an
ordinal within that family, so `GO-1` is `⟨"GO", 1⟩`. -/
structure ClaimId where
  familyTag : String
  ordinal : Nat
  deriving DecidableEq, Repr, Inhabited

/-- The rendered claim identifier, e.g. `GO-1`. -/
def ClaimId.render (i : ClaimId) : String :=
  i.familyTag ++ "-" ++ toString i.ordinal

/-! ## Claim status -/

/-- Registry status of a row.  SPEC §17.2 requires a status field; the
enumeration here is the closed set the conformance tool reports. -/
inductive ClaimStatus
  | pending
  | satisfied
  | failed
  | notApplicable
  deriving DecidableEq, Repr, Inhabited

/-! ## Formal claim row (SPEC §17.2) -/

/-- A recorded axiom name, as reported by transitive axiom collection. -/
abbrev AxiomLabel := String

/-- One row of `model/claims.json` (SPEC §17.2).

Every item of the SPEC list is a field:

* unique claim ID and exact statement — `id`, `statement`;
* claim level — `level`;
* proposition canonical bytes and identity — `propositionBytes`,
  `propositionIdentity`;
* exact Lean declaration and source module — `leanDeclaration`, `sourceModule`;
* Wasm profile, GEMM problem, cost objective, universe and artifact identities —
  `profile`, `problem`, `objective`, `universe`, `artifact`;
* direct proof-dependency IDs — `dependencies`;
* collected transitive axiom names — `transitiveAxioms`;
* checker command — `checkerCommand`;
* falsifier fixture and expected rejection — `falsifierFixture`,
  `expectedRejection`;
* status and release applicability — `status`, `releaseApplicable`. -/
structure ClaimRow where
  /-- Unique claim ID. -/
  id : ClaimId
  /-- The exact statement, as published. -/
  statement : String
  /-- The claim level (SPEC §17.1). -/
  level : ClaimLevel
  /-- Canonical bytes of the proposition. -/
  propositionBytes : ByteArray
  /-- Canonical identity of the proposition. -/
  propositionIdentity : CanonicalObjectId
  /-- The exact Lean declaration name. -/
  leanDeclaration : String
  /-- The Lean module the declaration lives in. -/
  sourceModule : String
  /-- Wasm profile identity the claim is stated against. -/
  profile : ProfileId
  /-- GEMM problem identity. -/
  problem : ProblemId
  /-- Cost objective identity. -/
  objective : ObjectiveId
  /-- Competitor universe identity. -/
  universeIdentity : CanonicalObjectId
  /-- Artifact identity. -/
  artifact : CanonicalObjectId
  /-- Direct proof-dependency IDs. -/
  dependencies : List ClaimId
  /-- Collected transitive axiom names. -/
  transitiveAxioms : List AxiomLabel
  /-- The command that checks this row. -/
  checkerCommand : String
  /-- The falsifier fixture this row is mutated against. -/
  falsifierFixture : String
  /-- The rejection the falsifier fixture must produce. -/
  expectedRejection : String
  /-- Registry status. -/
  status : ClaimStatus
  /-- Whether the row applies to the release gate. -/
  releaseApplicable : Bool
  deriving DecidableEq

namespace ClaimRow

/-- A row may use proof language exactly when its level is `formalProof`. -/
def MayUseProofLanguage (r : ClaimRow) : Prop := SupportsProofLanguage r.level

instance : DecidablePred MayUseProofLanguage := fun r =>
  inferInstanceAs (Decidable (SupportsProofLanguage r.level))

theorem mayUseProofLanguage_iff (r : ClaimRow) :
    MayUseProofLanguage r ↔ r.level = .formalProof :=
  supportsProofLanguage_iff r.level

/-- A measurement row may never be stated with proof language. -/
theorem not_mayUseProofLanguage_of_measurement {r : ClaimRow}
    (h : r.level = .measurement) : ¬ MayUseProofLanguage r := by
  rw [mayUseProofLanguage_iff, h]
  exact fun h' => by cases h'

/-- A build-evidence row may never be stated with proof language. -/
theorem not_mayUseProofLanguage_of_buildEvidence {r : ClaimRow}
    (h : r.level = .buildEvidence) : ¬ MayUseProofLanguage r := by
  rw [mayUseProofLanguage_iff, h]
  exact fun h' => by cases h'

/-- An authority row may never be stated with proof language. -/
theorem not_mayUseProofLanguage_of_authority {r : ClaimRow}
    (h : r.level = .authority) : ¬ MayUseProofLanguage r := by
  rw [mayUseProofLanguage_iff, h]
  exact fun h' => by cases h'

/-- An open row may never be stated with proof language. -/
theorem not_mayUseProofLanguage_of_open {r : ClaimRow}
    (h : r.level = ClaimLevel.open) : ¬ MayUseProofLanguage r := by
  rw [mayUseProofLanguage_iff, h]
  exact fun h' => by cases h'

end ClaimRow

end WasmGemmGnaf.Conformance
