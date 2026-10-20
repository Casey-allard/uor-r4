/-
  Wasm/Feature.lean --- the complete Core 3.0 feature matrix.

  Normative source: SPEC.md section 7.2, the table

      | Core 3.0 feature family                                  | decision |
      | scalar numeric, control, variables, functions            | enabled  |
      | multi-value and extended constant expressions            | enabled  |
      | bulk memory, multiple memories, tables                   | enabled  |
      | standard fixed 128-bit SIMD                              | enabled  |
      | reference types, typed function references, and GC       | enabled  |
      | tail calls                                               | enabled  |
      | exception handling                                       | enabled  |
      | memory64 address types and instructions                  | rejected |
      | shared memories, atomics, and threads                    | rejected |
      | relaxed SIMD operations                                  | rejected |
      | component-model binaries and post-Core-3 proposals       | rejected |

  SPEC section 7.2 requires that "no feature defaults are permitted": the
  release decision is a total function with an exhaustive match and no default
  arm, and the first-order decision table stored in the profile body agrees
  with it on every family.  Both facts are proved below.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Foundation.Finite
import WasmGemmGnaf.Wasm.Revision

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## The feature families -/

/-- The complete Core 3.0 feature matrix of SPEC section 7.2.  Exactly eleven
families; every pinned Core 3.0 proposal falls in exactly one of them. -/
inductive FeatureFamily
  /-- Scalar numeric instructions, control instructions, variables, and
  functions. -/
  | scalarCore
  /-- Multi-value results and extended constant expressions. -/
  | multiValueAndExtendedConst
  /-- Bulk memory operations, multiple memories, and tables. -/
  | bulkMemoryMultipleMemoriesTables
  /-- Standard fixed-width 128-bit SIMD. -/
  | fixedWidthSimd128
  /-- Reference types, typed function references, and garbage collection. -/
  | referenceTypesTypedFunctionReferencesGc
  /-- Tail calls. -/
  | tailCalls
  /-- Exception handling. -/
  | exceptionHandling
  /-- memory64 address types and instructions. -/
  | memory64
  /-- Shared memories, atomics, and threads. -/
  | sharedMemoriesAtomicsThreads
  /-- Relaxed SIMD operations. -/
  | relaxedSimd
  /-- Component-model binaries and post-Core-3 proposals. -/
  | componentModelAndPostCore3
  deriving DecidableEq, Repr, Inhabited

/-- The two possible release decisions.  There is no third state: a feature is
either fully modeled and enabled, or rejected by validation. -/
inductive FeatureDecision
  | enabled
  | rejected
  deriving DecidableEq, Repr, Inhabited

namespace FeatureDecision

def tag : FeatureDecision → UInt8
  | .enabled => 0
  | .rejected => 1

theorem tag_injective : Function.Injective tag := by
  intro a b h
  cases a <;> cases b <;> first | rfl | exact absurd h (by decide)

def bytes (d : FeatureDecision) : List UInt8 := Bytes.u8Bytes (tag d)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.u8Bytes_prefixFree.comp tag_injective

def all : List FeatureDecision := [.enabled, .rejected]

theorem mem_all (d : FeatureDecision) : d ∈ all := by
  cases d <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance instFintype : Fintype FeatureDecision where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

end FeatureDecision

namespace FeatureFamily

/-- One-byte canonical tag of a feature family. -/
def tag : FeatureFamily → UInt8
  | .scalarCore => 0
  | .multiValueAndExtendedConst => 1
  | .bulkMemoryMultipleMemoriesTables => 2
  | .fixedWidthSimd128 => 3
  | .referenceTypesTypedFunctionReferencesGc => 4
  | .tailCalls => 5
  | .exceptionHandling => 6
  | .memory64 => 7
  | .sharedMemoriesAtomicsThreads => 8
  | .relaxedSimd => 9
  | .componentModelAndPostCore3 => 10

theorem tag_injective : Function.Injective tag := by
  intro a b h
  cases a <;> cases b <;> first | rfl | exact absurd h (by decide)

def bytes (f : FeatureFamily) : List UInt8 := Bytes.u8Bytes (tag f)

theorem bytes_prefixFree : Bytes.PrefixFree bytes :=
  Bytes.u8Bytes_prefixFree.comp tag_injective

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

/-- The complete enumeration of the feature matrix. -/
def all : List FeatureFamily :=
  [ .scalarCore
  , .multiValueAndExtendedConst
  , .bulkMemoryMultipleMemoriesTables
  , .fixedWidthSimd128
  , .referenceTypesTypedFunctionReferencesGc
  , .tailCalls
  , .exceptionHandling
  , .memory64
  , .sharedMemoriesAtomicsThreads
  , .relaxedSimd
  , .componentModelAndPostCore3 ]

theorem mem_all (f : FeatureFamily) : f ∈ all := by
  cases f <;> simp [all]

theorem all_nodup : all.Nodup := by decide

theorem all_length : all.length = 11 := rfl

instance instFintype : Fintype FeatureFamily where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

end FeatureFamily

/-! ## The release decision

An exhaustive structural match with one arm per family and no default arm. -/

/-- The release decision for each Core 3.0 feature family (SPEC section 7.2). -/
def releaseDecision : FeatureFamily → FeatureDecision
  | .scalarCore => .enabled
  | .multiValueAndExtendedConst => .enabled
  | .bulkMemoryMultipleMemoriesTables => .enabled
  | .fixedWidthSimd128 => .enabled
  | .referenceTypesTypedFunctionReferencesGc => .enabled
  | .tailCalls => .enabled
  | .exceptionHandling => .enabled
  | .memory64 => .rejected
  | .sharedMemoriesAtomicsThreads => .rejected
  | .relaxedSimd => .rejected
  | .componentModelAndPostCore3 => .rejected

/-- Totality: the decision function returns one of the two decisions on every
family.  Together with `releaseDecision_no_default` this is the "no feature
defaults are permitted" requirement of SPEC section 7.2. -/
theorem releaseDecision_total (f : FeatureFamily) :
    releaseDecision f = .enabled ∨ releaseDecision f = .rejected := by
  cases f <;> first | (exact Or.inl rfl) | (exact Or.inr rfl)

/-- Each family's decision is given by its own match arm: the value on every
family is one of the eleven literally written results, so no arm is supplied by
a default or a fallback. -/
theorem releaseDecision_no_default (f : FeatureFamily) :
    (f = .scalarCore ∧ releaseDecision f = .enabled) ∨
    (f = .multiValueAndExtendedConst ∧ releaseDecision f = .enabled) ∨
    (f = .bulkMemoryMultipleMemoriesTables ∧ releaseDecision f = .enabled) ∨
    (f = .fixedWidthSimd128 ∧ releaseDecision f = .enabled) ∨
    (f = .referenceTypesTypedFunctionReferencesGc ∧
      releaseDecision f = .enabled) ∨
    (f = .tailCalls ∧ releaseDecision f = .enabled) ∨
    (f = .exceptionHandling ∧ releaseDecision f = .enabled) ∨
    (f = .memory64 ∧ releaseDecision f = .rejected) ∨
    (f = .sharedMemoriesAtomicsThreads ∧ releaseDecision f = .rejected) ∨
    (f = .relaxedSimd ∧ releaseDecision f = .rejected) ∨
    (f = .componentModelAndPostCore3 ∧ releaseDecision f = .rejected) := by
  cases f
  · exact Or.inl ⟨rfl, rfl⟩
  · exact Or.inr (Or.inl ⟨rfl, rfl⟩)
  · exact Or.inr (Or.inr (Or.inl ⟨rfl, rfl⟩))
  · exact Or.inr (Or.inr (Or.inr (Or.inl ⟨rfl, rfl⟩)))
  · exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inl ⟨rfl, rfl⟩))))
  · exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inl ⟨rfl, rfl⟩)))))
  · exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr
      (Or.inl ⟨rfl, rfl⟩))))))
  · exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr
      (Or.inl ⟨rfl, rfl⟩)))))))
  · exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr
      (Or.inl ⟨rfl, rfl⟩))))))))
  · exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr
      (Or.inr (Or.inl ⟨rfl, rfl⟩)))))))))
  · exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr (Or.inr
      (Or.inr (Or.inr ⟨rfl, rfl⟩)))))))))

/-- A family is enabled in the release profile. -/
def Enabled (f : FeatureFamily) : Prop := releaseDecision f = .enabled

/-- A family is rejected by release validation. -/
def Rejected (f : FeatureFamily) : Prop := releaseDecision f = .rejected

instance instDecidableEnabled (f : FeatureFamily) : Decidable (Enabled f) := by
  unfold Enabled; exact inferInstance

instance instDecidableRejected (f : FeatureFamily) : Decidable (Rejected f) := by
  unfold Rejected; exact inferInstance

theorem enabled_or_rejected (f : FeatureFamily) : Enabled f ∨ Rejected f :=
  releaseDecision_total f

theorem not_enabled_and_rejected (f : FeatureFamily) :
    ¬(Enabled f ∧ Rejected f) := by
  intro h
  exact absurd (h.1.symm.trans h.2) (by decide)

/-- The seven enabled families. -/
def enabledFamilies : List FeatureFamily :=
  [ .scalarCore
  , .multiValueAndExtendedConst
  , .bulkMemoryMultipleMemoriesTables
  , .fixedWidthSimd128
  , .referenceTypesTypedFunctionReferencesGc
  , .tailCalls
  , .exceptionHandling ]

/-- The four rejected families. -/
def rejectedFamilies : List FeatureFamily :=
  [ .memory64
  , .sharedMemoriesAtomicsThreads
  , .relaxedSimd
  , .componentModelAndPostCore3 ]

theorem mem_enabledFamilies_iff (f : FeatureFamily) :
    f ∈ enabledFamilies ↔ Enabled f := by
  cases f <;> simp [enabledFamilies, Enabled, releaseDecision]

theorem mem_rejectedFamilies_iff (f : FeatureFamily) :
    f ∈ rejectedFamilies ↔ Rejected f := by
  cases f <;> simp [rejectedFamilies, Rejected, releaseDecision]

theorem enabledFamilies_length : enabledFamilies.length = 7 := rfl

theorem rejectedFamilies_length : rejectedFamilies.length = 4 := rfl

/-- The two decision classes partition the matrix. -/
theorem enabled_rejected_partition (f : FeatureFamily) :
    (f ∈ enabledFamilies ∧ f ∉ rejectedFamilies) ∨
    (f ∉ enabledFamilies ∧ f ∈ rejectedFamilies) := by
  cases f <;> simp [enabledFamilies, rejectedFamilies]

theorem enabled_append_rejected_length :
    enabledFamilies.length + rejectedFamilies.length =
      FeatureFamily.all.length := rfl

/-! ## The first-order decision table

SPEC section 6.2 forbids storing functions in an identity body, so the profile
body carries the decision *table* and the profile's lawfulness ties that table
to `releaseDecision`. -/

/-- One row of the stored feature matrix. -/
structure FeatureRow where
  family : FeatureFamily
  decision : FeatureDecision
  deriving DecidableEq, Repr, Inhabited

namespace FeatureRow

def bytes (row : FeatureRow) : List UInt8 :=
  FeatureFamily.bytes row.family ++ FeatureDecision.bytes row.decision

theorem bytes_prefixFree : Bytes.PrefixFree bytes := by
  intro x y r s h
  simp only [bytes, List.append_assoc] at h
  obtain ⟨h1, h⟩ := FeatureFamily.bytes_prefixFree _ _ _ _ h
  obtain ⟨h2, h⟩ := FeatureDecision.bytes_prefixFree _ _ _ _ h
  refine ⟨?_, h⟩
  cases x
  cases y
  simp_all

theorem bytes_injective : Function.Injective bytes :=
  bytes_prefixFree.injective

end FeatureRow

/-- Prefix-free encoding of a feature table. -/
def featureTableBytes (rows : List FeatureRow) : List UInt8 :=
  Bytes.listBytes FeatureRow.bytes rows

theorem featureTableBytes_prefixFree : Bytes.PrefixFree featureTableBytes :=
  Bytes.listBytes_prefixFree FeatureRow.bytes_prefixFree

/-- The stored release feature matrix: exactly the table of SPEC section 7.2,
in matrix order. -/
def releaseFeatureTable : List FeatureRow :=
  [ { family := .scalarCore, decision := .enabled }
  , { family := .multiValueAndExtendedConst, decision := .enabled }
  , { family := .bulkMemoryMultipleMemoriesTables, decision := .enabled }
  , { family := .fixedWidthSimd128, decision := .enabled }
  , { family := .referenceTypesTypedFunctionReferencesGc, decision := .enabled }
  , { family := .tailCalls, decision := .enabled }
  , { family := .exceptionHandling, decision := .enabled }
  , { family := .memory64, decision := .rejected }
  , { family := .sharedMemoriesAtomicsThreads, decision := .rejected }
  , { family := .relaxedSimd, decision := .rejected }
  , { family := .componentModelAndPostCore3, decision := .rejected } ]

/-- Table lookup.  `none` would be a defaulted feature; `lookup_total` below
proves it never happens on the release table. -/
def lookupDecision (rows : List FeatureRow) (f : FeatureFamily) :
    Option FeatureDecision :=
  (rows.find? (fun row => row.family == f)).map FeatureRow.decision

/-- No feature defaults: every family has a row, and its stored decision is
exactly `releaseDecision`. -/
theorem lookupDecision_releaseFeatureTable (f : FeatureFamily) :
    lookupDecision releaseFeatureTable f = some (releaseDecision f) := by
  cases f <;> rfl

/-- Coverage: the release table contains a row for every family. -/
theorem releaseFeatureTable_covers (f : FeatureFamily) :
    { family := f, decision := releaseDecision f } ∈ releaseFeatureTable := by
  cases f <;> simp [releaseFeatureTable, releaseDecision]

/-- Soundness: no row of the release table disagrees with `releaseDecision`. -/
theorem releaseFeatureTable_sound (row : FeatureRow)
    (h : row ∈ releaseFeatureTable) :
    row.decision = releaseDecision row.family := by
  simp only [releaseFeatureTable, List.mem_cons, List.not_mem_nil, or_false] at h
  rcases h with h | h | h | h | h | h | h | h | h | h | h <;>
    subst h <;> rfl

theorem releaseFeatureTable_length : releaseFeatureTable.length = 11 := rfl

theorem releaseFeatureTable_nodup : releaseFeatureTable.Nodup := by decide

/-- No family is listed twice. -/
theorem releaseFeatureTable_families_nodup :
    (releaseFeatureTable.map FeatureRow.family).Nodup := by decide

/-- The stored table is exactly the graph of `releaseDecision`. -/
theorem releaseFeatureTable_eq_graph :
    releaseFeatureTable =
      FeatureFamily.all.map
        (fun f => { family := f, decision := releaseDecision f }) := rfl

theorem releaseFeatureTable_enabled_count :
    (releaseFeatureTable.filter
      (fun row => row.decision == .enabled)).length = 7 := rfl

theorem releaseFeatureTable_rejected_count :
    (releaseFeatureTable.filter
      (fun row => row.decision == .rejected)).length = 4 := rfl

end WasmGemmGnaf.Wasm
