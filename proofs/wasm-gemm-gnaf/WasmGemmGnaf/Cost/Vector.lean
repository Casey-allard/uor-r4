/-
  Cost vectors: the complete UOR-Wasm cost vector.
  Normative source: SPEC.md section 9.1.

  Every declaration in this file is proved. Nothing is assumed.
-/
import WasmGemmGnaf.Foundation.Order

set_option autoImplicit false

namespace WasmGemmGnaf.Cost

/-! ## Static coordinates -/

/-- SPEC 9.1 static cost coordinates. -/
structure StaticVector where
  moduleBytes     : Nat
  decodeSteps     : Nat
  validationSteps : Nat
  staticDataBytes : Nat
  deriving Repr, DecidableEq, Inhabited

namespace StaticVector

def zero : StaticVector :=
  { moduleBytes := 0, decodeSteps := 0, validationSteps := 0
    staticDataBytes := 0 }

/-- Componentwise sum of static vectors (SPEC 16: static coordinates are
charged at each actual artifact construction). -/
def add (a b : StaticVector) : StaticVector :=
  { moduleBytes := a.moduleBytes + b.moduleBytes
    decodeSteps := a.decodeSteps + b.decodeSteps
    validationSteps := a.validationSteps + b.validationSteps
    staticDataBytes := a.staticDataBytes + b.staticDataBytes }

theorem add_zero_left (a : StaticVector) : add zero a = a := by
  cases a; simp [add, zero]

theorem add_zero_right (a : StaticVector) : add a zero = a := by
  cases a; simp [add, zero]

theorem add_assoc (a b c : StaticVector) : add (add a b) c = add a (add b c) := by
  cases a; cases b; cases c; simp [add, Nat.add_assoc]

end StaticVector

/-! ## Dynamic coordinates -/

/-- SPEC 9.1 dynamic cost coordinates, in normative order. -/
structure DynamicVector where
  instantiationSteps     : Nat
  dispatchSteps          : Nat
  preparationSteps       : Nat
  wasmRuleSteps          : Nat
  scalarOps              : Nat
  vectorLaneOps          : Nat
  bytesRead              : Nat
  bytesWritten           : Nat
  memoryGrowPages        : Nat
  tableElementsAllocated : Nat
  gcObjectsAllocated     : Nat
  gcBytesInitialized     : Nat
  peakStackValues        : Nat
  peakPages              : Nat
  peakGcLiveBytes        : Nat
  outputBytes            : Nat
  deriving Repr, DecidableEq, Inhabited

namespace DynamicVector

def zero : DynamicVector :=
  { instantiationSteps := 0, dispatchSteps := 0, preparationSteps := 0
    wasmRuleSteps := 0, scalarOps := 0, vectorLaneOps := 0
    bytesRead := 0, bytesWritten := 0, memoryGrowPages := 0
    tableElementsAllocated := 0, gcObjectsAllocated := 0
    gcBytesInitialized := 0, peakStackValues := 0, peakPages := 0
    peakGcLiveBytes := 0, outputBytes := 0 }

end DynamicVector

/-- The sixteen dynamic coordinates, as an index type. -/
inductive DynamicCoordinate
  | instantiationSteps | dispatchSteps | preparationSteps | wasmRuleSteps
  | scalarOps | vectorLaneOps | bytesRead | bytesWritten
  | memoryGrowPages | tableElementsAllocated | gcObjectsAllocated
  | gcBytesInitialized | peakStackValues | peakPages | peakGcLiveBytes
  | outputBytes
  deriving Repr, DecidableEq, Inhabited

namespace DynamicCoordinate

def value : DynamicCoordinate → DynamicVector → Nat
  | .instantiationSteps, v => v.instantiationSteps
  | .dispatchSteps, v => v.dispatchSteps
  | .preparationSteps, v => v.preparationSteps
  | .wasmRuleSteps, v => v.wasmRuleSteps
  | .scalarOps, v => v.scalarOps
  | .vectorLaneOps, v => v.vectorLaneOps
  | .bytesRead, v => v.bytesRead
  | .bytesWritten, v => v.bytesWritten
  | .memoryGrowPages, v => v.memoryGrowPages
  | .tableElementsAllocated, v => v.tableElementsAllocated
  | .gcObjectsAllocated, v => v.gcObjectsAllocated
  | .gcBytesInitialized, v => v.gcBytesInitialized
  | .peakStackValues, v => v.peakStackValues
  | .peakPages, v => v.peakPages
  | .peakGcLiveBytes, v => v.peakGcLiveBytes
  | .outputBytes, v => v.outputBytes

/-- The three coordinates that `sequentialCompose` maximises rather than adds. -/
def IsPeak : DynamicCoordinate → Bool
  | .peakStackValues => true
  | .peakPages => true
  | .peakGcLiveBytes => true
  | _ => false

def all : List DynamicCoordinate :=
  [ .instantiationSteps, .dispatchSteps, .preparationSteps, .wasmRuleSteps
  , .scalarOps, .vectorLaneOps, .bytesRead, .bytesWritten
  , .memoryGrowPages, .tableElementsAllocated, .gcObjectsAllocated
  , .gcBytesInitialized, .peakStackValues, .peakPages, .peakGcLiveBytes
  , .outputBytes ]

theorem mem_all (dc : DynamicCoordinate) : dc ∈ all := by
  cases dc <;> simp [all]

theorem all_length : all.length = 16 := rfl

theorem value_zero (dc : DynamicCoordinate) : dc.value DynamicVector.zero = 0 := by
  cases dc <;> rfl

end DynamicCoordinate

namespace DynamicVector

/-- Componentwise order on dynamic vectors. -/
def ComponentwiseLE (a b : DynamicVector) : Prop :=
  ∀ dc : DynamicCoordinate, dc.value a ≤ dc.value b

theorem componentwiseLE_refl (a : DynamicVector) : ComponentwiseLE a a :=
  fun _ => Nat.le_refl _

theorem componentwiseLE_trans {a b c : DynamicVector}
    (hab : ComponentwiseLE a b) (hbc : ComponentwiseLE b c) :
    ComponentwiseLE a c :=
  fun dc => Nat.le_trans (hab dc) (hbc dc)

theorem componentwiseLE_antisymm {a b : DynamicVector}
    (hab : ComponentwiseLE a b) (hba : ComponentwiseLE b a) : a = b := by
  cases a; cases b
  have h0 := Nat.le_antisymm (hab .instantiationSteps) (hba .instantiationSteps)
  have h1 := Nat.le_antisymm (hab .dispatchSteps) (hba .dispatchSteps)
  have h2 := Nat.le_antisymm (hab .preparationSteps) (hba .preparationSteps)
  have h3 := Nat.le_antisymm (hab .wasmRuleSteps) (hba .wasmRuleSteps)
  have h4 := Nat.le_antisymm (hab .scalarOps) (hba .scalarOps)
  have h5 := Nat.le_antisymm (hab .vectorLaneOps) (hba .vectorLaneOps)
  have h6 := Nat.le_antisymm (hab .bytesRead) (hba .bytesRead)
  have h7 := Nat.le_antisymm (hab .bytesWritten) (hba .bytesWritten)
  have h8 := Nat.le_antisymm (hab .memoryGrowPages) (hba .memoryGrowPages)
  have h9 := Nat.le_antisymm (hab .tableElementsAllocated) (hba .tableElementsAllocated)
  have h10 := Nat.le_antisymm (hab .gcObjectsAllocated) (hba .gcObjectsAllocated)
  have h11 := Nat.le_antisymm (hab .gcBytesInitialized) (hba .gcBytesInitialized)
  have h12 := Nat.le_antisymm (hab .peakStackValues) (hba .peakStackValues)
  have h13 := Nat.le_antisymm (hab .peakPages) (hba .peakPages)
  have h14 := Nat.le_antisymm (hab .peakGcLiveBytes) (hba .peakGcLiveBytes)
  have h15 := Nat.le_antisymm (hab .outputBytes) (hba .outputBytes)
  simp only [DynamicCoordinate.value] at *
  subst h0; subst h1; subst h2; subst h3; subst h4; subst h5; subst h6
  subst h7; subst h8; subst h9; subst h10; subst h11; subst h12; subst h13
  subst h14; subst h15
  rfl

theorem zero_componentwiseLE (a : DynamicVector) : ComponentwiseLE zero a := by
  intro dc
  rw [DynamicCoordinate.value_zero]
  exact Nat.zero_le _

/-- Total charged work in a dynamic vector: the sum of all sixteen
coordinates. -/
def total (v : DynamicVector) : Nat :=
  v.instantiationSteps + v.dispatchSteps + v.preparationSteps + v.wasmRuleSteps
    + v.scalarOps + v.vectorLaneOps + v.bytesRead + v.bytesWritten
    + v.memoryGrowPages + v.tableElementsAllocated + v.gcObjectsAllocated
    + v.gcBytesInitialized + v.peakStackValues + v.peakPages
    + v.peakGcLiveBytes + v.outputBytes

/-- The additive part of the total: the thirteen coordinates that
`sequentialCompose` adds.  A composed run always retains at least this much of
the second operand's charge. -/
def additiveTotal (v : DynamicVector) : Nat :=
  v.instantiationSteps + v.dispatchSteps + v.preparationSteps + v.wasmRuleSteps
    + v.scalarOps + v.vectorLaneOps + v.bytesRead + v.bytesWritten
    + v.memoryGrowPages + v.tableElementsAllocated + v.gcObjectsAllocated
    + v.gcBytesInitialized + v.outputBytes

theorem additiveTotal_le_total (v : DynamicVector) : v.additiveTotal ≤ v.total := by
  simp only [additiveTotal, total]; omega

theorem total_zero : total zero = 0 := rfl

theorem additiveTotal_zero : additiveTotal zero = 0 := rfl

theorem total_mono {a b : DynamicVector} (h : ComponentwiseLE a b) :
    a.total ≤ b.total := by
  have h0 := h .instantiationSteps
  have h1 := h .dispatchSteps
  have h2 := h .preparationSteps
  have h3 := h .wasmRuleSteps
  have h4 := h .scalarOps
  have h5 := h .vectorLaneOps
  have h6 := h .bytesRead
  have h7 := h .bytesWritten
  have h8 := h .memoryGrowPages
  have h9 := h .tableElementsAllocated
  have h10 := h .gcObjectsAllocated
  have h11 := h .gcBytesInitialized
  have h12 := h .peakStackValues
  have h13 := h .peakPages
  have h14 := h .peakGcLiveBytes
  have h15 := h .outputBytes
  simp only [DynamicCoordinate.value] at *
  simp only [total]
  omega

end DynamicVector

/-! ## Sequential composition -/

/-- SPEC 9.1. Sums for cumulative coordinates, maxima for the three peaks. -/
def sequentialCompose (first second : DynamicVector) : DynamicVector :=
  { instantiationSteps := first.instantiationSteps + second.instantiationSteps
    dispatchSteps := first.dispatchSteps + second.dispatchSteps
    preparationSteps := first.preparationSteps + second.preparationSteps
    wasmRuleSteps := first.wasmRuleSteps + second.wasmRuleSteps
    scalarOps := first.scalarOps + second.scalarOps
    vectorLaneOps := first.vectorLaneOps + second.vectorLaneOps
    bytesRead := first.bytesRead + second.bytesRead
    bytesWritten := first.bytesWritten + second.bytesWritten
    memoryGrowPages := first.memoryGrowPages + second.memoryGrowPages
    tableElementsAllocated :=
      first.tableElementsAllocated + second.tableElementsAllocated
    gcObjectsAllocated := first.gcObjectsAllocated + second.gcObjectsAllocated
    gcBytesInitialized := first.gcBytesInitialized + second.gcBytesInitialized
    peakStackValues := max first.peakStackValues second.peakStackValues
    peakPages := max first.peakPages second.peakPages
    peakGcLiveBytes := max first.peakGcLiveBytes second.peakGcLiveBytes
    outputBytes := first.outputBytes + second.outputBytes }

theorem sequentialCompose_zero_left (v : DynamicVector) :
    sequentialCompose DynamicVector.zero v = v := by
  cases v; simp [sequentialCompose, DynamicVector.zero]

theorem sequentialCompose_zero_right (v : DynamicVector) :
    sequentialCompose v DynamicVector.zero = v := by
  cases v; simp [sequentialCompose, DynamicVector.zero]

theorem sequentialCompose_assoc (a b c : DynamicVector) :
    sequentialCompose (sequentialCompose a b) c
      = sequentialCompose a (sequentialCompose b c) := by
  cases a; cases b; cases c
  simp [sequentialCompose, Nat.add_assoc, Nat.max_assoc]

/-- Composition never loses the first operand's charge. -/
theorem sequentialCompose_le_left (a b : DynamicVector) :
    DynamicVector.ComponentwiseLE a (sequentialCompose a b) := by
  intro dc
  cases dc <;>
    simp only [DynamicCoordinate.value, sequentialCompose] <;>
    first
      | exact Nat.le_add_right _ _
      | exact Nat.le_max_left _ _

/-- Composition never loses the second operand's charge. -/
theorem sequentialCompose_le_right (a b : DynamicVector) :
    DynamicVector.ComponentwiseLE b (sequentialCompose a b) := by
  intro dc
  cases dc <;>
    simp only [DynamicCoordinate.value, sequentialCompose] <;>
    first
      | exact Nat.le_add_left _ _
      | exact Nat.le_max_right _ _

/-- SPEC 9.1: sequential composition is monotone in both arguments. -/
theorem sequentialCompose_mono {a a' b b' : DynamicVector}
    (ha : DynamicVector.ComponentwiseLE a a')
    (hb : DynamicVector.ComponentwiseLE b b') :
    DynamicVector.ComponentwiseLE (sequentialCompose a b) (sequentialCompose a' b') := by
  intro dc
  have hA := ha dc
  have hB := hb dc
  cases dc <;>
    simp only [DynamicCoordinate.value, sequentialCompose] at hA hB ⊢ <;>
    omega

theorem sequentialCompose_mono_left {a a' : DynamicVector} (b : DynamicVector)
    (ha : DynamicVector.ComponentwiseLE a a') :
    DynamicVector.ComponentwiseLE (sequentialCompose a b) (sequentialCompose a' b) :=
  sequentialCompose_mono ha (DynamicVector.componentwiseLE_refl b)

theorem sequentialCompose_mono_right (a : DynamicVector) {b b' : DynamicVector}
    (hb : DynamicVector.ComponentwiseLE b b') :
    DynamicVector.ComponentwiseLE (sequentialCompose a b) (sequentialCompose a b') :=
  sequentialCompose_mono (DynamicVector.componentwiseLE_refl a) hb

/-- Positive accounting: composing a charge onto a running cost increases the
running total by at least the additive part of that charge. -/
theorem total_sequentialCompose_ge (a b : DynamicVector) :
    a.total + b.additiveTotal ≤ (sequentialCompose a b).total := by
  simp only [DynamicVector.total, DynamicVector.additiveTotal, sequentialCompose]
  have p0 := Nat.le_max_left a.peakStackValues b.peakStackValues
  have p1 := Nat.le_max_left a.peakPages b.peakPages
  have p2 := Nat.le_max_left a.peakGcLiveBytes b.peakGcLiveBytes
  omega

/-! ## The complete artifact vector -/

/-- SPEC 9.1 complete artifact cost. -/
structure ArtifactVector where
  static     : StaticVector
  dynamicSum : DynamicVector
  dynamicMax : DynamicVector
  deriving Repr, DecidableEq, Inhabited

abbrev CompleteSystemCost := ArtifactVector

namespace ArtifactVector

def zero : ArtifactVector :=
  { static := StaticVector.zero
    dynamicSum := DynamicVector.zero
    dynamicMax := DynamicVector.zero }

end ArtifactVector

/-- SPEC 9.1. All 36 charged coordinates. -/
inductive ArtifactCoordinate
  | staticModuleBytes | staticDecodeSteps | staticValidationSteps
  | staticDataBytes
  | sumInstantiationSteps | sumDispatchSteps | sumPreparationSteps
  | sumWasmRuleSteps | sumScalarOps | sumVectorLaneOps
  | sumBytesRead | sumBytesWritten | sumMemoryGrowPages
  | sumTableElementsAllocated | sumGcObjectsAllocated | sumGcBytesInitialized
  | sumPeakStackValues | sumPeakPages | sumPeakGcLiveBytes
  | sumOutputBytes
  | maxInstantiationSteps | maxDispatchSteps | maxPreparationSteps
  | maxWasmRuleSteps | maxScalarOps | maxVectorLaneOps
  | maxBytesRead | maxBytesWritten | maxMemoryGrowPages
  | maxTableElementsAllocated | maxGcObjectsAllocated | maxGcBytesInitialized
  | maxPeakStackValues | maxPeakPages | maxPeakGcLiveBytes
  | maxOutputBytes
  deriving Repr, DecidableEq, Inhabited

namespace ArtifactCoordinate

def value : ArtifactCoordinate → ArtifactVector → Nat
  | .staticModuleBytes, c => c.static.moduleBytes
  | .staticDecodeSteps, c => c.static.decodeSteps
  | .staticValidationSteps, c => c.static.validationSteps
  | .staticDataBytes, c => c.static.staticDataBytes
  | .sumInstantiationSteps, c => c.dynamicSum.instantiationSteps
  | .sumDispatchSteps, c => c.dynamicSum.dispatchSteps
  | .sumPreparationSteps, c => c.dynamicSum.preparationSteps
  | .sumWasmRuleSteps, c => c.dynamicSum.wasmRuleSteps
  | .sumScalarOps, c => c.dynamicSum.scalarOps
  | .sumVectorLaneOps, c => c.dynamicSum.vectorLaneOps
  | .sumBytesRead, c => c.dynamicSum.bytesRead
  | .sumBytesWritten, c => c.dynamicSum.bytesWritten
  | .sumMemoryGrowPages, c => c.dynamicSum.memoryGrowPages
  | .sumTableElementsAllocated, c => c.dynamicSum.tableElementsAllocated
  | .sumGcObjectsAllocated, c => c.dynamicSum.gcObjectsAllocated
  | .sumGcBytesInitialized, c => c.dynamicSum.gcBytesInitialized
  | .sumPeakStackValues, c => c.dynamicSum.peakStackValues
  | .sumPeakPages, c => c.dynamicSum.peakPages
  | .sumPeakGcLiveBytes, c => c.dynamicSum.peakGcLiveBytes
  | .sumOutputBytes, c => c.dynamicSum.outputBytes
  | .maxInstantiationSteps, c => c.dynamicMax.instantiationSteps
  | .maxDispatchSteps, c => c.dynamicMax.dispatchSteps
  | .maxPreparationSteps, c => c.dynamicMax.preparationSteps
  | .maxWasmRuleSteps, c => c.dynamicMax.wasmRuleSteps
  | .maxScalarOps, c => c.dynamicMax.scalarOps
  | .maxVectorLaneOps, c => c.dynamicMax.vectorLaneOps
  | .maxBytesRead, c => c.dynamicMax.bytesRead
  | .maxBytesWritten, c => c.dynamicMax.bytesWritten
  | .maxMemoryGrowPages, c => c.dynamicMax.memoryGrowPages
  | .maxTableElementsAllocated, c => c.dynamicMax.tableElementsAllocated
  | .maxGcObjectsAllocated, c => c.dynamicMax.gcObjectsAllocated
  | .maxGcBytesInitialized, c => c.dynamicMax.gcBytesInitialized
  | .maxPeakStackValues, c => c.dynamicMax.peakStackValues
  | .maxPeakPages, c => c.dynamicMax.peakPages
  | .maxPeakGcLiveBytes, c => c.dynamicMax.peakGcLiveBytes
  | .maxOutputBytes, c => c.dynamicMax.outputBytes

/-- The sixteen coordinates read off `dynamicMax`, which aggregate by
componentwise maximum rather than by addition. -/
def IsDynamicMax : ArtifactCoordinate → Bool
  | .maxInstantiationSteps | .maxDispatchSteps | .maxPreparationSteps
  | .maxWasmRuleSteps | .maxScalarOps | .maxVectorLaneOps
  | .maxBytesRead | .maxBytesWritten | .maxMemoryGrowPages
  | .maxTableElementsAllocated | .maxGcObjectsAllocated
  | .maxGcBytesInitialized | .maxPeakStackValues | .maxPeakPages
  | .maxPeakGcLiveBytes | .maxOutputBytes => true
  | _ => false

/-- Finite-list coverage for the coordinate index (SPEC 6.3). -/
def all : List ArtifactCoordinate :=
  [ .staticModuleBytes, .staticDecodeSteps, .staticValidationSteps
  , .staticDataBytes
  , .sumInstantiationSteps, .sumDispatchSteps, .sumPreparationSteps
  , .sumWasmRuleSteps, .sumScalarOps, .sumVectorLaneOps
  , .sumBytesRead, .sumBytesWritten, .sumMemoryGrowPages
  , .sumTableElementsAllocated, .sumGcObjectsAllocated, .sumGcBytesInitialized
  , .sumPeakStackValues, .sumPeakPages, .sumPeakGcLiveBytes
  , .sumOutputBytes
  , .maxInstantiationSteps, .maxDispatchSteps, .maxPreparationSteps
  , .maxWasmRuleSteps, .maxScalarOps, .maxVectorLaneOps
  , .maxBytesRead, .maxBytesWritten, .maxMemoryGrowPages
  , .maxTableElementsAllocated, .maxGcObjectsAllocated, .maxGcBytesInitialized
  , .maxPeakStackValues, .maxPeakPages, .maxPeakGcLiveBytes
  , .maxOutputBytes ]

/-- `all` is a complete cover of the coordinate index. -/
theorem mem_all (co : ArtifactCoordinate) : co ∈ all := by
  cases co <;> simp [all]

theorem all_length : all.length = 36 := rfl

theorem all_nodup : all.Nodup := by decide

theorem value_zero (co : ArtifactCoordinate) : co.value ArtifactVector.zero = 0 := by
  cases co <;> rfl

end ArtifactCoordinate

namespace ArtifactVector

/-- Rebuild an artifact vector from its 36 coordinate values. -/
def ofCoords (f : ArtifactCoordinate → Nat) : ArtifactVector :=
  { static :=
      { moduleBytes := f .staticModuleBytes
        decodeSteps := f .staticDecodeSteps
        validationSteps := f .staticValidationSteps
        staticDataBytes := f .staticDataBytes }
    dynamicSum :=
      { instantiationSteps := f .sumInstantiationSteps
        dispatchSteps := f .sumDispatchSteps
        preparationSteps := f .sumPreparationSteps
        wasmRuleSteps := f .sumWasmRuleSteps
        scalarOps := f .sumScalarOps
        vectorLaneOps := f .sumVectorLaneOps
        bytesRead := f .sumBytesRead
        bytesWritten := f .sumBytesWritten
        memoryGrowPages := f .sumMemoryGrowPages
        tableElementsAllocated := f .sumTableElementsAllocated
        gcObjectsAllocated := f .sumGcObjectsAllocated
        gcBytesInitialized := f .sumGcBytesInitialized
        peakStackValues := f .sumPeakStackValues
        peakPages := f .sumPeakPages
        peakGcLiveBytes := f .sumPeakGcLiveBytes
        outputBytes := f .sumOutputBytes }
    dynamicMax :=
      { instantiationSteps := f .maxInstantiationSteps
        dispatchSteps := f .maxDispatchSteps
        preparationSteps := f .maxPreparationSteps
        wasmRuleSteps := f .maxWasmRuleSteps
        scalarOps := f .maxScalarOps
        vectorLaneOps := f .maxVectorLaneOps
        bytesRead := f .maxBytesRead
        bytesWritten := f .maxBytesWritten
        memoryGrowPages := f .maxMemoryGrowPages
        tableElementsAllocated := f .maxTableElementsAllocated
        gcObjectsAllocated := f .maxGcObjectsAllocated
        gcBytesInitialized := f .maxGcBytesInitialized
        peakStackValues := f .maxPeakStackValues
        peakPages := f .maxPeakPages
        peakGcLiveBytes := f .maxPeakGcLiveBytes
        outputBytes := f .maxOutputBytes }}

/-- The 36 coordinates determine the artifact vector. -/
theorem ofCoords_value (c : ArtifactVector) :
    ofCoords (fun co => co.value c) = c := by
  cases c with
  | mk s ds dm => cases s; cases ds; cases dm; rfl

theorem value_ofCoords (f : ArtifactCoordinate → Nat) (co : ArtifactCoordinate) :
    co.value (ofCoords f) = f co := by
  cases co <;> rfl

/-- The coordinate map is injective. -/
theorem coords_injective {a b : ArtifactVector}
    (h : ∀ co : ArtifactCoordinate, co.value a = co.value b) : a = b := by
  rw [← ofCoords_value a, ← ofCoords_value b]
  simp only [ofCoords]
  simp only [h]

end ArtifactVector

/-- Componentwise order on artifact cost vectors (SPEC 9.3). -/
def ComponentwiseLE (a b : ArtifactVector) : Prop :=
  ∀ co : ArtifactCoordinate, co.value a ≤ co.value b

theorem componentwiseLE_refl (a : ArtifactVector) : ComponentwiseLE a a :=
  fun _ => Nat.le_refl _

theorem componentwiseLE_trans {a b c : ArtifactVector}
    (hab : ComponentwiseLE a b) (hbc : ComponentwiseLE b c) :
    ComponentwiseLE a c :=
  fun co => Nat.le_trans (hab co) (hbc co)

theorem componentwiseLE_antisymm {a b : ArtifactVector}
    (hab : ComponentwiseLE a b) (hba : ComponentwiseLE b a) : a = b :=
  ArtifactVector.coords_injective (fun co => Nat.le_antisymm (hab co) (hba co))

namespace ArtifactVector

/-- Componentwise order on *artifact* cost vectors, indexed by the 36 charged
coordinates.  Distinct from `DynamicVector.ComponentwiseLE`. -/
abbrev ComponentwiseLE (a b : ArtifactVector) : Prop := Cost.ComponentwiseLE a b

theorem componentwiseLE_refl (a : ArtifactVector) : ComponentwiseLE a a :=
  Cost.componentwiseLE_refl a

theorem componentwiseLE_trans {a b c : ArtifactVector}
    (hab : ComponentwiseLE a b) (hbc : ComponentwiseLE b c) :
    ComponentwiseLE a c :=
  Cost.componentwiseLE_trans hab hbc

theorem componentwiseLE_antisymm {a b : ArtifactVector}
    (hab : ComponentwiseLE a b) (hba : ComponentwiseLE b a) : a = b :=
  Cost.componentwiseLE_antisymm hab hba

end ArtifactVector

/-! ## Tie order -/

/-- SPEC 9.1 / 9.3: the canonical tie-break order tag. -/
inductive TieOrderTag
  | unsignedByteLexicographic
  deriving Repr, DecidableEq, Inhabited

namespace TieOrderTag

/-- The order denoted by a tie-order tag.  The only tag denotes the unsigned
byte-lexicographic order proved total in `Foundation.Order`. -/
def Order : TieOrderTag → ByteArray → ByteArray → Prop
  | .unsignedByteLexicographic => Foundation.CanonicalBytesLE

theorem order_refl (t : TieOrderTag) (a : ByteArray) : t.Order a a := by
  cases t; exact Foundation.CanonicalBytesLE.refl a

theorem order_trans (t : TieOrderTag) {a b c : ByteArray}
    (hab : t.Order a b) (hbc : t.Order b c) : t.Order a c := by
  cases t; exact Foundation.CanonicalBytesLE.trans hab hbc

theorem order_antisymm (t : TieOrderTag) {a b : ByteArray}
    (hab : t.Order a b) (hba : t.Order b a) : a = b := by
  cases t; exact Foundation.CanonicalBytesLE.antisymm hab hba

theorem order_total (t : TieOrderTag) (a b : ByteArray) :
    t.Order a b ∨ t.Order b a := by
  cases t; exact Foundation.CanonicalBytesLE.total a b

instance instDecidableOrder (t : TieOrderTag) (a b : ByteArray) :
    Decidable (t.Order a b) := by
  cases t; exact Foundation.instDecidableCanonicalBytesLE a b

end TieOrderTag

end WasmGemmGnaf.Cost
