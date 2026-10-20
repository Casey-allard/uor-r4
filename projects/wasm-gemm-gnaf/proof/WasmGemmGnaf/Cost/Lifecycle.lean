/-
  Lifecycle cost vectors, the frozen lifecycle fold, and the native lifecycle
  cost polynomial.
  Normative source: SPEC.md sections 9.1 (`Cost.LifecycleVector`) and 16
  (`Atlas.LifecycleSizeVector`, `Cost.PrimitiveCostTable`,
  `Atlas.nativeLifecycleBound`, `Cost.sumLifecycle`).

  Atlas search, theorem construction, and release certification costs use these
  vectors.  They are never added to `Cost.ArtifactVector`; see SPEC 9.2.

  Every declaration in this file is proved. Nothing is assumed.
-/
import WasmGemmGnaf.Cost.Aggregate
import WasmGemmGnaf.Cost.Objective

set_option autoImplicit false

namespace WasmGemmGnaf.Cost

/-! ## The lifecycle vector -/

/-- SPEC 9.1, `Cost.LifecycleVector`. -/
structure LifecycleVector where
  authorityCheckSteps      : Nat
  canonicalizationSteps    : Nat
  canonicalNovelObjects    : Nat
  canonicalNovelEdges      : Nat
  closureSteps             : Nat
  indexSteps               : Nat
  attentionBucketsTouched  : Nat
  dependencyObjectsVisited : Nat
  partitionSteps           : Nat
  partitionCellsChanged    : Nat
  verifierSteps            : Nat
  sealSteps                : Nat
  querySelectionSteps      : Nat
  migrationSteps           : Nat
  retainedStateBytes       : Nat
  peakWorkingBytes         : Nat
  artifact                 : ArtifactVector
  deriving Repr, DecidableEq, Inhabited

namespace LifecycleVector

def zero : LifecycleVector :=
  { authorityCheckSteps := 0, canonicalizationSteps := 0
    canonicalNovelObjects := 0, canonicalNovelEdges := 0
    closureSteps := 0, indexSteps := 0, attentionBucketsTouched := 0
    dependencyObjectsVisited := 0, partitionSteps := 0
    partitionCellsChanged := 0, verifierSteps := 0, sealSteps := 0
    querySelectionSteps := 0, migrationSteps := 0, retainedStateBytes := 0
    peakWorkingBytes := 0, artifact := ArtifactVector.zero }

/--
  SPEC 16, the frozen lifecycle fold step.

  Every step/count coordinate is natural addition in prefix order;
  `retainedStateBytes` and `peakWorkingBytes` are componentwise maxima;
  artifact static coordinates are charged at each actual artifact
  construction, artifact dynamic sums are added, and artifact dynamic peaks
  are maxima.
-/
def combine (a b : LifecycleVector) : LifecycleVector :=
  { authorityCheckSteps := a.authorityCheckSteps + b.authorityCheckSteps
    canonicalizationSteps := a.canonicalizationSteps + b.canonicalizationSteps
    canonicalNovelObjects := a.canonicalNovelObjects + b.canonicalNovelObjects
    canonicalNovelEdges := a.canonicalNovelEdges + b.canonicalNovelEdges
    closureSteps := a.closureSteps + b.closureSteps
    indexSteps := a.indexSteps + b.indexSteps
    attentionBucketsTouched := a.attentionBucketsTouched + b.attentionBucketsTouched
    dependencyObjectsVisited :=
      a.dependencyObjectsVisited + b.dependencyObjectsVisited
    partitionSteps := a.partitionSteps + b.partitionSteps
    partitionCellsChanged := a.partitionCellsChanged + b.partitionCellsChanged
    verifierSteps := a.verifierSteps + b.verifierSteps
    sealSteps := a.sealSteps + b.sealSteps
    querySelectionSteps := a.querySelectionSteps + b.querySelectionSteps
    migrationSteps := a.migrationSteps + b.migrationSteps
    retainedStateBytes := max a.retainedStateBytes b.retainedStateBytes
    peakWorkingBytes := max a.peakWorkingBytes b.peakWorkingBytes
    artifact :=
      { static := StaticVector.add a.artifact.static b.artifact.static
        dynamicSum := ComponentwiseAdd a.artifact.dynamicSum b.artifact.dynamicSum
        dynamicMax := ComponentwiseMax a.artifact.dynamicMax b.artifact.dynamicMax } }

theorem combine_zero_left (a : LifecycleVector) : combine zero a = a := by
  obtain ⟨_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, art⟩ := a
  obtain ⟨s, ds, dm⟩ := art
  cases s; cases ds; cases dm
  simp [combine, zero, StaticVector.add, ComponentwiseAdd, ComponentwiseMax,
    ArtifactVector.zero, StaticVector.zero, DynamicVector.zero]

theorem combine_zero_right (a : LifecycleVector) : combine a zero = a := by
  obtain ⟨_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, art⟩ := a
  obtain ⟨s, ds, dm⟩ := art
  cases s; cases ds; cases dm
  simp [combine, zero, StaticVector.add, ComponentwiseAdd, ComponentwiseMax,
    ArtifactVector.zero, StaticVector.zero, DynamicVector.zero]

theorem combine_assoc (a b c : LifecycleVector) :
    combine (combine a b) c = combine a (combine b c) := by
  simp only [combine, LifecycleVector.mk.injEq, ArtifactVector.mk.injEq]
  refine ⟨Nat.add_assoc .., Nat.add_assoc .., Nat.add_assoc .., Nat.add_assoc ..,
    Nat.add_assoc .., Nat.add_assoc .., Nat.add_assoc .., Nat.add_assoc ..,
    Nat.add_assoc .., Nat.add_assoc .., Nat.add_assoc .., Nat.add_assoc ..,
    Nat.add_assoc .., Nat.add_assoc .., Nat.max_assoc .., Nat.max_assoc ..,
    StaticVector.add_assoc .., componentwiseAdd_assoc ..,
    componentwiseMax_assoc ..⟩

/-- Componentwise order on lifecycle vectors. -/
def ComponentwiseLE (a b : LifecycleVector) : Prop :=
  a.authorityCheckSteps ≤ b.authorityCheckSteps ∧
  a.canonicalizationSteps ≤ b.canonicalizationSteps ∧
  a.canonicalNovelObjects ≤ b.canonicalNovelObjects ∧
  a.canonicalNovelEdges ≤ b.canonicalNovelEdges ∧
  a.closureSteps ≤ b.closureSteps ∧
  a.indexSteps ≤ b.indexSteps ∧
  a.attentionBucketsTouched ≤ b.attentionBucketsTouched ∧
  a.dependencyObjectsVisited ≤ b.dependencyObjectsVisited ∧
  a.partitionSteps ≤ b.partitionSteps ∧
  a.partitionCellsChanged ≤ b.partitionCellsChanged ∧
  a.verifierSteps ≤ b.verifierSteps ∧
  a.sealSteps ≤ b.sealSteps ∧
  a.querySelectionSteps ≤ b.querySelectionSteps ∧
  a.migrationSteps ≤ b.migrationSteps ∧
  a.retainedStateBytes ≤ b.retainedStateBytes ∧
  a.peakWorkingBytes ≤ b.peakWorkingBytes ∧
  Cost.ComponentwiseLE a.artifact b.artifact

theorem componentwiseLE_refl (a : LifecycleVector) : ComponentwiseLE a a :=
  ⟨Nat.le_refl .., Nat.le_refl .., Nat.le_refl .., Nat.le_refl .., Nat.le_refl ..,
   Nat.le_refl .., Nat.le_refl .., Nat.le_refl .., Nat.le_refl .., Nat.le_refl ..,
   Nat.le_refl .., Nat.le_refl .., Nat.le_refl .., Nat.le_refl .., Nat.le_refl ..,
   Nat.le_refl .., Cost.componentwiseLE_refl _⟩

theorem componentwiseLE_trans {a b c : LifecycleVector}
    (hab : ComponentwiseLE a b) (hbc : ComponentwiseLE b c) :
    ComponentwiseLE a c :=
  ⟨Nat.le_trans hab.1 hbc.1,
   Nat.le_trans hab.2.1 hbc.2.1,
   Nat.le_trans hab.2.2.1 hbc.2.2.1,
   Nat.le_trans hab.2.2.2.1 hbc.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.1 hbc.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.1 hbc.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.1 hbc.2.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.2.1 hbc.2.2.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.2.2.1 hbc.2.2.2.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.2.2.2.1 hbc.2.2.2.2.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.2.2.2.2.1 hbc.2.2.2.2.2.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.2.2.2.2.2.1 hbc.2.2.2.2.2.2.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.2.2.2.2.2.2.1 hbc.2.2.2.2.2.2.2.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.2.2.2.2.2.2.2.1 hbc.2.2.2.2.2.2.2.2.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1 hbc.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1
     hbc.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.1,
   Cost.componentwiseLE_trans hab.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2
     hbc.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2.2⟩

end LifecycleVector

/-! ## The frozen lifecycle fold -/

/-- The maximum of a list of naturals, with `0` for the empty list. -/
def maxOf (l : List Nat) : Nat := l.foldr Nat.max 0

theorem maxOf_nil : maxOf [] = 0 := rfl

theorem maxOf_cons (x : Nat) (l : List Nat) : maxOf (x :: l) = max x (maxOf l) := rfl

theorem le_maxOf {x : Nat} : ∀ {l : List Nat}, x ∈ l → x ≤ maxOf l := by
  intro l
  induction l with
  | nil => intro h; exact absurd h (by simp)
  | cons y t ih =>
      intro h
      rw [maxOf_cons]
      rcases List.mem_cons.mp h with rfl | hmem
      · exact Nat.le_max_left _ _
      · exact Nat.le_trans (ih hmem) (Nat.le_max_right _ _)

/--
  SPEC 16, `Cost.sumLifecycle`: the frozen mixed fold over lifecycle prefix
  costs, in prefix order.  The empty fold is the all-zero vector.
-/
def sumLifecycle (l : List LifecycleVector) : LifecycleVector :=
  l.foldr LifecycleVector.combine LifecycleVector.zero

theorem sumLifecycle_nil : sumLifecycle [] = LifecycleVector.zero := rfl

theorem sumLifecycle_cons (v : LifecycleVector) (l : List LifecycleVector) :
    sumLifecycle (v :: l) = LifecycleVector.combine v (sumLifecycle l) := rfl

theorem sumLifecycle_singleton (v : LifecycleVector) : sumLifecycle [v] = v := by
  rw [sumLifecycle_cons, sumLifecycle_nil, LifecycleVector.combine_zero_right]

/-- Fold law for an additively combined coordinate. -/
theorem sumLifecycle_add_coord (f : LifecycleVector → Nat)
    (hzero : f LifecycleVector.zero = 0)
    (hcomb : ∀ a b, f (LifecycleVector.combine a b) = f a + f b) :
    ∀ l : List LifecycleVector, f (sumLifecycle l) = (l.map f).sum := by
  intro l
  induction l with
  | nil => rw [sumLifecycle_nil, hzero]; rfl
  | cons v t ih =>
      rw [sumLifecycle_cons, hcomb, ih, List.map_cons, List.sum_cons]

/-- Fold law for a maximum-combined coordinate. -/
theorem sumLifecycle_max_coord (f : LifecycleVector → Nat)
    (hzero : f LifecycleVector.zero = 0)
    (hcomb : ∀ a b, f (LifecycleVector.combine a b) = max (f a) (f b)) :
    ∀ l : List LifecycleVector, f (sumLifecycle l) = maxOf (l.map f) := by
  intro l
  induction l with
  | nil => rw [sumLifecycle_nil, hzero]; rfl
  | cons v t ih =>
      rw [sumLifecycle_cons, hcomb, ih, List.map_cons, maxOf_cons]

/-! ### The fourteen step/count coordinates are sums -/

theorem sumLifecycle_authorityCheckSteps (l : List LifecycleVector) :
    (sumLifecycle l).authorityCheckSteps
      = (l.map (·.authorityCheckSteps)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_canonicalizationSteps (l : List LifecycleVector) :
    (sumLifecycle l).canonicalizationSteps
      = (l.map (·.canonicalizationSteps)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_canonicalNovelObjects (l : List LifecycleVector) :
    (sumLifecycle l).canonicalNovelObjects
      = (l.map (·.canonicalNovelObjects)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_canonicalNovelEdges (l : List LifecycleVector) :
    (sumLifecycle l).canonicalNovelEdges = (l.map (·.canonicalNovelEdges)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_closureSteps (l : List LifecycleVector) :
    (sumLifecycle l).closureSteps = (l.map (·.closureSteps)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_indexSteps (l : List LifecycleVector) :
    (sumLifecycle l).indexSteps = (l.map (·.indexSteps)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_attentionBucketsTouched (l : List LifecycleVector) :
    (sumLifecycle l).attentionBucketsTouched
      = (l.map (·.attentionBucketsTouched)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_dependencyObjectsVisited (l : List LifecycleVector) :
    (sumLifecycle l).dependencyObjectsVisited
      = (l.map (·.dependencyObjectsVisited)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_partitionSteps (l : List LifecycleVector) :
    (sumLifecycle l).partitionSteps = (l.map (·.partitionSteps)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_partitionCellsChanged (l : List LifecycleVector) :
    (sumLifecycle l).partitionCellsChanged
      = (l.map (·.partitionCellsChanged)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_verifierSteps (l : List LifecycleVector) :
    (sumLifecycle l).verifierSteps = (l.map (·.verifierSteps)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_sealSteps (l : List LifecycleVector) :
    (sumLifecycle l).sealSteps = (l.map (·.sealSteps)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_querySelectionSteps (l : List LifecycleVector) :
    (sumLifecycle l).querySelectionSteps = (l.map (·.querySelectionSteps)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_migrationSteps (l : List LifecycleVector) :
    (sumLifecycle l).migrationSteps = (l.map (·.migrationSteps)).sum :=
  sumLifecycle_add_coord _ rfl (fun _ _ => rfl) l

/-! ### Retained state and peak working bytes are componentwise maxima -/

theorem sumLifecycle_retainedStateBytes (l : List LifecycleVector) :
    (sumLifecycle l).retainedStateBytes = maxOf (l.map (·.retainedStateBytes)) :=
  sumLifecycle_max_coord _ rfl (fun _ _ => rfl) l

theorem sumLifecycle_peakWorkingBytes (l : List LifecycleVector) :
    (sumLifecycle l).peakWorkingBytes = maxOf (l.map (·.peakWorkingBytes)) :=
  sumLifecycle_max_coord _ rfl (fun _ _ => rfl) l

/-! ### Artifact coordinates -/

/-- Artifact static coordinates and dynamic sums accumulate additively. -/
theorem sumLifecycle_artifact_add (co : ArtifactCoordinate)
    (hmax : co.IsDynamicMax = false) (l : List LifecycleVector) :
    co.value (sumLifecycle l).artifact
      = (l.map (fun v => co.value v.artifact)).sum := by
  refine sumLifecycle_add_coord (fun v => co.value v.artifact) ?_ ?_ l
  · cases co <;> rfl
  · intro a b
    cases co <;> first | rfl | exact absurd hmax (by decide)

/-- Artifact dynamic peaks accumulate by maximum. -/
theorem sumLifecycle_artifact_max (co : ArtifactCoordinate)
    (hmax : co.IsDynamicMax = true) (l : List LifecycleVector) :
    co.value (sumLifecycle l).artifact
      = maxOf (l.map (fun v => co.value v.artifact)) := by
  refine sumLifecycle_max_coord (fun v => co.value v.artifact) ?_ ?_ l
  · cases co <;> rfl
  · intro a b
    cases co <;> first | rfl | exact absurd hmax (by decide)

/-- Every prefix cost is dominated by the lifecycle total. -/
theorem le_sumLifecycle_add_coord (f : LifecycleVector → Nat)
    (hzero : f LifecycleVector.zero = 0)
    (hcomb : ∀ a b, f (LifecycleVector.combine a b) = f a + f b)
    {v : LifecycleVector} {l : List LifecycleVector} (hv : v ∈ l) :
    f v ≤ f (sumLifecycle l) := by
  rw [sumLifecycle_add_coord f hzero hcomb]
  exact le_sum_of_mem (List.mem_map_of_mem hv)

theorem le_sumLifecycle_max_coord (f : LifecycleVector → Nat)
    (hzero : f LifecycleVector.zero = 0)
    (hcomb : ∀ a b, f (LifecycleVector.combine a b) = max (f a) (f b))
    {v : LifecycleVector} {l : List LifecycleVector} (hv : v ∈ l) :
    f v ≤ f (sumLifecycle l) := by
  rw [sumLifecycle_max_coord f hzero hcomb]
  exact le_maxOf (List.mem_map_of_mem hv)

/-! ## Truncated difference (SPEC 16 regret) -/

namespace StaticVector

/-- Componentwise truncated subtraction. -/
def sub (a b : StaticVector) : StaticVector :=
  { moduleBytes := a.moduleBytes - b.moduleBytes
    decodeSteps := a.decodeSteps - b.decodeSteps
    validationSteps := a.validationSteps - b.validationSteps
    staticDataBytes := a.staticDataBytes - b.staticDataBytes }

end StaticVector

namespace DynamicVector

/-- Componentwise truncated subtraction. -/
def sub (a b : DynamicVector) : DynamicVector :=
  { instantiationSteps := a.instantiationSteps - b.instantiationSteps
    dispatchSteps := a.dispatchSteps - b.dispatchSteps
    preparationSteps := a.preparationSteps - b.preparationSteps
    wasmRuleSteps := a.wasmRuleSteps - b.wasmRuleSteps
    scalarOps := a.scalarOps - b.scalarOps
    vectorLaneOps := a.vectorLaneOps - b.vectorLaneOps
    bytesRead := a.bytesRead - b.bytesRead
    bytesWritten := a.bytesWritten - b.bytesWritten
    memoryGrowPages := a.memoryGrowPages - b.memoryGrowPages
    tableElementsAllocated := a.tableElementsAllocated - b.tableElementsAllocated
    gcObjectsAllocated := a.gcObjectsAllocated - b.gcObjectsAllocated
    gcBytesInitialized := a.gcBytesInitialized - b.gcBytesInitialized
    peakStackValues := a.peakStackValues - b.peakStackValues
    peakPages := a.peakPages - b.peakPages
    peakGcLiveBytes := a.peakGcLiveBytes - b.peakGcLiveBytes
    outputBytes := a.outputBytes - b.outputBytes }

end DynamicVector

namespace ArtifactVector

/-- Componentwise truncated subtraction. -/
def sub (a b : ArtifactVector) : ArtifactVector :=
  { static := StaticVector.sub a.static b.static
    dynamicSum := DynamicVector.sub a.dynamicSum b.dynamicSum
    dynamicMax := DynamicVector.sub a.dynamicMax b.dynamicMax }

theorem value_sub (co : ArtifactCoordinate) (a b : ArtifactVector) :
    co.value (sub a b) = co.value a - co.value b := by
  cases co <;> rfl

end ArtifactVector

/-- SPEC 16: the componentwise truncated difference of two lifecycle totals,
the carrier of the regret comparison against the canonical full rebuild. -/
def truncatedDifference (a b : LifecycleVector) : LifecycleVector :=
  { authorityCheckSteps := a.authorityCheckSteps - b.authorityCheckSteps
    canonicalizationSteps := a.canonicalizationSteps - b.canonicalizationSteps
    canonicalNovelObjects := a.canonicalNovelObjects - b.canonicalNovelObjects
    canonicalNovelEdges := a.canonicalNovelEdges - b.canonicalNovelEdges
    closureSteps := a.closureSteps - b.closureSteps
    indexSteps := a.indexSteps - b.indexSteps
    attentionBucketsTouched := a.attentionBucketsTouched - b.attentionBucketsTouched
    dependencyObjectsVisited :=
      a.dependencyObjectsVisited - b.dependencyObjectsVisited
    partitionSteps := a.partitionSteps - b.partitionSteps
    partitionCellsChanged := a.partitionCellsChanged - b.partitionCellsChanged
    verifierSteps := a.verifierSteps - b.verifierSteps
    sealSteps := a.sealSteps - b.sealSteps
    querySelectionSteps := a.querySelectionSteps - b.querySelectionSteps
    migrationSteps := a.migrationSteps - b.migrationSteps
    retainedStateBytes := a.retainedStateBytes - b.retainedStateBytes
    peakWorkingBytes := a.peakWorkingBytes - b.peakWorkingBytes
    artifact := ArtifactVector.sub a.artifact b.artifact }

/-- No regret against a dominating comparator. -/
theorem truncatedDifference_self (a : LifecycleVector) :
    truncatedDifference a a = LifecycleVector.zero := by
  obtain ⟨_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, art⟩ := a
  obtain ⟨s, ds, dm⟩ := art
  cases s; cases ds; cases dm
  simp [truncatedDifference, LifecycleVector.zero, ArtifactVector.sub,
    ArtifactVector.zero, StaticVector.sub, StaticVector.zero,
    DynamicVector.sub, DynamicVector.zero]

/-! ## Lifecycle sizes and the native cost polynomial -/

/-- SPEC 16, `Atlas.LifecycleSizeVector`: the first-order size inputs of the
native lifecycle bound. -/
structure LifecycleSizeVector where
  horizon                    : Nat
  authorityBytesChecked      : Nat
  deltaBytes                 : Nat
  requestBytesChecked        : Nat
  canonicalNovelObjects      : Nat
  canonicalNovelEdges        : Nat
  closureDerivations         : Nat
  attentionBucketsTouched    : Nat
  dependencyImpactObjects    : Nat
  partitionCellsChanged      : Nat
  certificateBytesChecked    : Nat
  retainedStateBytes         : Nat
  sealCount                  : Nat
  sealInputBytesScanned      : Nat
  queryCount                 : Nat
  attentionCandidatesVisited : Nat
  migrationObjects           : Nat
  peakWorkingBytes           : Nat
  artifactWork               : ArtifactVector
  deriving Repr, DecidableEq, Inhabited

/-- SPEC 16, `Cost.PrimitiveCostTable`: the canonical primitive coefficients.
Every coefficient is a positive natural; see `AllCoefficientsPositive`. -/
structure PrimitiveCostTable where
  cAuthority    : Nat
  cCanonicalize : Nat
  cClosure      : Nat
  cIndex        : Nat
  cDependency   : Nat
  cPartition    : Nat
  cVerify       : Nat
  cSeal         : Nat
  cSealScan     : Nat
  cQuery        : Nat
  cMigration    : Nat
  deriving Repr, DecidableEq, Inhabited

namespace PrimitiveCostTable

/-- SPEC 16: "Every coefficient is a positive natural in the canonical
primitive-cost table."  A decidable syntactic side condition on the table. -/
def AllCoefficientsPositive (t : PrimitiveCostTable) : Prop :=
  1 ≤ t.cAuthority ∧ 1 ≤ t.cCanonicalize ∧ 1 ≤ t.cClosure ∧ 1 ≤ t.cIndex ∧
  1 ≤ t.cDependency ∧ 1 ≤ t.cPartition ∧ 1 ≤ t.cVerify ∧ 1 ≤ t.cSeal ∧
  1 ≤ t.cSealScan ∧ 1 ≤ t.cQuery ∧ 1 ≤ t.cMigration

instance instDecidableAllCoefficientsPositive (t : PrimitiveCostTable) :
    Decidable (AllCoefficientsPositive t) := by
  unfold AllCoefficientsPositive; infer_instance

/-- The all-ones table, which is positive. -/
def ones : PrimitiveCostTable :=
  { cAuthority := 1, cCanonicalize := 1, cClosure := 1, cIndex := 1
    cDependency := 1, cPartition := 1, cVerify := 1, cSeal := 1
    cSealScan := 1, cQuery := 1, cMigration := 1 }

theorem ones_positive : AllCoefficientsPositive ones := by
  refine ⟨?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_⟩ <;> exact Nat.le_refl 1

end PrimitiveCostTable

/--
  SPEC 16, `Atlas.nativeLifecycleBound`: the fixed componentwise polynomial.

  Authority checking is bounded by `cAuthority * authorityBytesChecked`;
  canonicalization by
  `cCanonicalize * deltaBytes * (log2 (retainedStateBytes + 2) + 1)`;
  closure by `cClosure * closureDerivations`; indexing by
  `cIndex * (canonicalNovelObjects + canonicalNovelEdges +
  attentionBucketsTouched)`; invalidation by
  `cDependency * dependencyImpactObjects`; partition work by
  `cPartition * partitionCellsChanged`; verification and sealing by their fixed
  coefficients times `certificateBytesChecked`, `sealCount` and the cumulative
  `sealInputBytesScanned`; query work by `cQuery * (queryCount +
  requestBytesChecked + attentionCandidatesVisited)`; migration by
  `cMigration * migrationObjects`.  Artifact work is exactly
  `size.artifactWork`, and the retained-state and peak-working-byte output
  coordinates are respectively `size.retainedStateBytes` and
  `size.peakWorkingBytes`, not absorbed into constants.
-/
def nativeLifecycleBound (table : PrimitiveCostTable)
    (size : LifecycleSizeVector) : LifecycleVector :=
  { authorityCheckSteps := table.cAuthority * size.authorityBytesChecked
    canonicalizationSteps :=
      table.cCanonicalize * size.deltaBytes
        * (Nat.log2 (size.retainedStateBytes + 2) + 1)
    canonicalNovelObjects := size.canonicalNovelObjects
    canonicalNovelEdges := size.canonicalNovelEdges
    closureSteps := table.cClosure * size.closureDerivations
    indexSteps :=
      table.cIndex
        * (size.canonicalNovelObjects + size.canonicalNovelEdges
            + size.attentionBucketsTouched)
    attentionBucketsTouched := size.attentionBucketsTouched
    dependencyObjectsVisited := table.cDependency * size.dependencyImpactObjects
    partitionSteps := table.cPartition * size.partitionCellsChanged
    partitionCellsChanged := size.partitionCellsChanged
    verifierSteps := table.cVerify * size.certificateBytesChecked
    sealSteps :=
      table.cSeal * size.sealCount + table.cSealScan * size.sealInputBytesScanned
    querySelectionSteps :=
      table.cQuery
        * (size.queryCount + size.requestBytesChecked
            + size.attentionCandidatesVisited)
    migrationSteps := table.cMigration * size.migrationObjects
    retainedStateBytes := size.retainedStateBytes
    peakWorkingBytes := size.peakWorkingBytes
    artifact := size.artifactWork }

/-- The bound's artifact work is exactly the size vector's artifact work: no
lifecycle coefficient can absorb runtime artifact cost. -/
theorem nativeLifecycleBound_artifact (table : PrimitiveCostTable)
    (size : LifecycleSizeVector) :
    (nativeLifecycleBound table size).artifact = size.artifactWork := rfl

/-- Retained state bytes are reported, not absorbed into a constant. -/
theorem nativeLifecycleBound_retainedStateBytes (table : PrimitiveCostTable)
    (size : LifecycleSizeVector) :
    (nativeLifecycleBound table size).retainedStateBytes
      = size.retainedStateBytes := rfl

/-- Peak working bytes are reported, not absorbed into a constant. -/
theorem nativeLifecycleBound_peakWorkingBytes (table : PrimitiveCostTable)
    (size : LifecycleSizeVector) :
    (nativeLifecycleBound table size).peakWorkingBytes
      = size.peakWorkingBytes := rfl

/-- With a positive cost table, the bound charges at least one step per checked
authority byte. -/
theorem authorityBytes_le_nativeLifecycleBound {table : PrimitiveCostTable}
    (hpos : table.AllCoefficientsPositive) (size : LifecycleSizeVector) :
    size.authorityBytesChecked
      ≤ (nativeLifecycleBound table size).authorityCheckSteps :=
  Nat.le_mul_of_pos_left _ hpos.1

/-- With a positive cost table, the bound charges at least one step per checked
certificate byte. -/
theorem certificateBytes_le_nativeLifecycleBound {table : PrimitiveCostTable}
    (hpos : table.AllCoefficientsPositive) (size : LifecycleSizeVector) :
    size.certificateBytesChecked
      ≤ (nativeLifecycleBound table size).verifierSteps :=
  Nat.le_mul_of_pos_left _ hpos.2.2.2.2.2.2.1

end WasmGemmGnaf.Cost
