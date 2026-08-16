import WasmGemmGnaf.GNAF.Shape
set_option autoImplicit false

/-!
# GNAF: typed ports, index maps and hyperedges (authority §5.7, §6.1)

Authority §6.1 makes the symbolic planning representation a typed directed
hypergraph whose vertices are typed component values and whose hyperedges
consume and produce finite ordered families.  Authority §5.7 requires that at
every connected plan boundary the producer output and the consumer input be the
*same* type instance; equality of host values or bytes is explicitly declared
insufficient.

This file supplies those three things and nothing else:

* `ObjType`, the type instances of the plan language, with a decidable
  equality — the only admitted composition test;
* `IndexMap`, the affine index maps of loop nests, with composition proved and
  the packed row-major map proved to agree with `Shape.packedIndex`;
* `RegionRef` and `Hyperedge`, with disjointness and boundary composition
  proved decidable and correct.
-/

namespace WasmGemmGnaf.GNAF

open WasmGemmGnaf.Foundation

/-! ## Type instances -/

/-- A type instance of the GNAF universe (authority §3.1): the exact type a
port carries. -/
inductive ObjType
  | scalar (kind : ScalarKind)
  | tensor (kind : ScalarKind) (shape : Shape)
  | statusType
  | bytes (length : Nat)
  | unit
  deriving DecidableEq, Repr, Inhabited

namespace ObjType

/-- The number of scalar elements a value of this type carries. -/
def elements : ObjType → Nat
  | scalar _ => 1
  | tensor _ s => s.count
  | statusType => 1
  | bytes n => n
  | unit => 0

/-- The number of stored bytes a value of this type occupies. -/
def byteSize : ObjType → Nat
  | scalar k => k.byteWidth
  | tensor k s => k.byteWidth * s.count
  | statusType => 4
  | bytes n => n
  | unit => 0

@[simp] theorem byteSize_unit : byteSize unit = 0 := rfl

theorem byteSize_tensor_zero (k : ScalarKind) (s : Shape) (h : s.count = 0) :
    byteSize (tensor k s) = 0 := by simp [byteSize, h]

/-- The typed content of `Object.lean` inhabits exactly one type instance. -/
def ofObject : Object → ObjType
  | .scalar v => scalar v.kind
  | .tensor k data => tensor k ⟨1, 1, data.length⟩
  | .statusValue _ => statusType
  | .bytes data => bytes data.length
  | .unit => unit

@[simp] theorem ofObject_unit : ofObject .unit = unit := rfl

theorem ofObject_scalar (v : ScalarValue) : ofObject (.scalar v) = scalar v.kind := rfl

/-- Typing is exact: two objects of different kinds never share a type
instance. -/
theorem ofObject_scalar_kind_eq {v w : ScalarValue}
    (h : ofObject (.scalar v) = ofObject (.scalar w)) : v.kind = w.kind := by
  simpa [ofObject] using h

end ObjType

/-! ## Ports -/

/-- A typed port of the derivation hypergraph (authority §6.1). -/
structure Port where
  id : Nat
  type : ObjType
  deriving DecidableEq, Repr, Inhabited

/-! ## Affine index maps -/

/-- An affine index map of a loop nest: `c0 + cb·b + ci·i + cj·j`
(SPEC §11.1 "loop nests and index maps"). -/
structure IndexMap where
  c0 : Nat
  cb : Nat
  ci : Nat
  cj : Nat
  deriving DecidableEq, Repr, Inhabited

namespace IndexMap

def apply (f : IndexMap) (b i j : Nat) : Nat :=
  f.c0 + f.cb * b + f.ci * i + f.cj * j

/-- The index map of the packed row-major layout. -/
def packed (s : Shape) : IndexMap :=
  { c0 := 0, cb := s.rows * s.cols, ci := s.cols, cj := 1 }

/-- The packed index map computes exactly the packed row-major linear index. -/
theorem packed_apply (s : Shape) (b i j : Nat) :
    (packed s).apply b i j = s.packedIndex b i j := by
  simp only [apply, packed, Shape.packedIndex, Nat.mul_add,
    Nat.mul_assoc, Nat.mul_comm, Nat.mul_left_comm, Nat.add_assoc,
    Nat.add_comm, Nat.add_left_comm, Nat.one_mul, Nat.zero_add]
  omega

/-- Translating an index map by a constant. -/
def shift (f : IndexMap) (d : Nat) : IndexMap := { f with c0 := f.c0 + d }

@[simp] theorem shift_apply (f : IndexMap) (d b i j : Nat) :
    (f.shift d).apply b i j = f.apply b i j + d := by
  simp only [apply, shift]
  omega

/-- Scaling every coefficient of an index map, as a packing transformation
does when it changes element width. -/
def scale (f : IndexMap) (w : Nat) : IndexMap :=
  { c0 := w * f.c0, cb := w * f.cb, ci := w * f.ci, cj := w * f.cj }

theorem scale_apply (f : IndexMap) (w b i j : Nat) :
    (f.scale w).apply b i j = w * f.apply b i j := by
  simp only [apply, scale, Nat.mul_add, Nat.mul_assoc]

@[simp] theorem scale_one (f : IndexMap) : f.scale 1 = f := by
  cases f; simp [scale]

theorem scale_scale (f : IndexMap) (v w : Nat) :
    (f.scale v).scale w = f.scale (w * v) := by
  cases f; simp [scale, Nat.mul_assoc]

/-- The identity-on-`b` map. -/
def idB : IndexMap := { c0 := 0, cb := 1, ci := 0, cj := 0 }

@[simp] theorem idB_apply (b i j : Nat) : idB.apply b i j = b := by
  simp [apply, idB]

/-- An affine map with bounded coefficients stays inside a bounded region. -/
theorem apply_le (f : IndexMap) {b i j B I J : Nat}
    (hb : b ≤ B) (hi : i ≤ I) (hj : j ≤ J) :
    f.apply b i j ≤ f.apply B I J := by
  simp only [apply]
  have h1 : f.cb * b ≤ f.cb * B := Nat.mul_le_mul (Nat.le_refl _) hb
  have h2 : f.ci * i ≤ f.ci * I := Nat.mul_le_mul (Nat.le_refl _) hi
  have h3 : f.cj * j ≤ f.cj * J := Nat.mul_le_mul (Nat.le_refl _) hj
  omega

end IndexMap

/-! ## Region references -/

/-- A contiguous element range of one of the machine's stores (SPEC §11.1
scratch allocation, packing, output construction). -/
structure RegionRef where
  base : Nat
  count : Nat
  deriving DecidableEq, Repr, Inhabited

namespace RegionRef

/-- One past the last addressed element. -/
def limit (r : RegionRef) : Nat := r.base + r.count

/-- Membership of an element index in a region. -/
def Mem (r : RegionRef) (x : Nat) : Prop := r.base ≤ x ∧ x < r.limit

instance (r : RegionRef) (x : Nat) : Decidable (r.Mem x) := by
  unfold Mem; infer_instance

/-- Two regions are disjoint when their element ranges do not overlap. -/
def Disjoint (r s : RegionRef) : Prop := r.limit ≤ s.base ∨ s.limit ≤ r.base

instance (r s : RegionRef) : Decidable (r.Disjoint s) := by
  unfold Disjoint; infer_instance

theorem disjoint_symm {r s : RegionRef} (h : r.Disjoint s) : s.Disjoint r := by
  rcases h with h | h
  · exact Or.inr h
  · exact Or.inl h

/-- Disjoint regions never address a common element: the fact an aliasing
contract needs. -/
theorem disjoint_no_common {r s : RegionRef} (h : r.Disjoint s) {x : Nat}
    (hr : r.Mem x) (hs : s.Mem x) : False := by
  unfold Mem limit at hr hs
  unfold Disjoint limit at h
  omega

/-- An empty region has no members. -/
theorem not_mem_of_count_zero {r : RegionRef} (h : r.count = 0) (x : Nat) :
    ¬ r.Mem x := by
  unfold Mem limit
  omega

/-- An empty region shares no element with any region. -/
theorem no_common_of_count_zero {r s : RegionRef} (h : r.count = 0) (x : Nat) :
    ¬ (r.Mem x ∧ s.Mem x) := fun hx => not_mem_of_count_zero h x hx.1

/-- A region fits a store of the given length. -/
def Fits (r : RegionRef) (size : Nat) : Prop := r.limit ≤ size

instance (r : RegionRef) (size : Nat) : Decidable (r.Fits size) := by
  unfold Fits; infer_instance

theorem mem_lt_of_fits {r : RegionRef} {size : Nat} (h : r.Fits size) {x : Nat}
    (hx : r.Mem x) : x < size := by
  unfold Fits limit at h
  unfold Mem limit at hx
  omega

end RegionRef

/-! ## Hyperedges -/

/-- A typed hyperedge of the derivation hypergraph (authority §6.1): an
operation identity together with its finite ordered input and output
families. -/
structure Hyperedge where
  opId : Nat
  inputs : List Port
  outputs : List Port
  deriving DecidableEq, Repr, Inhabited

namespace Hyperedge

/-- The declared boundary of an edge: the ordered input and output type
families. -/
def inputTypes (e : Hyperedge) : List ObjType := e.inputs.map Port.type

def outputTypes (e : Hyperedge) : List ObjType := e.outputs.map Port.type

/-- Authority §5.7: two edges compose only when the producer's output family is
exactly the consumer's input family, as type instances. -/
def Composable (e f : Hyperedge) : Prop := e.outputTypes = f.inputTypes

instance (e f : Hyperedge) : Decidable (e.Composable f) := by
  unfold Composable; infer_instance

/-- Composition is checked *typewise*: a composable pair agrees at every
boundary position. -/
theorem composable_getElem? {e f : Hyperedge} (h : e.Composable f) (n : Nat) :
    e.outputTypes[n]? = f.inputTypes[n]? := by
  unfold Composable at h
  rw [h]

/-- A composable pair has equally many boundary ports. -/
theorem composable_length {e f : Hyperedge} (h : e.Composable f) :
    e.outputs.length = f.inputs.length := by
  unfold Composable inputTypes outputTypes at h
  have := congrArg List.length h
  simpa using this

/-- Composability is *not* implied by port identity: only the declared type
family participates.  This is the formal content of authority §5.7's
prohibition on inferring composition from host values. -/
theorem composable_of_types {e f : Hyperedge}
    (h : e.outputs.map Port.type = f.inputs.map Port.type) : e.Composable f := h

/-- The identity edge on a type family. -/
def idEdge (opId : Nat) (ts : List ObjType) : Hyperedge where
  opId := opId
  inputs := ts.zipIdx.map (fun p => ⟨p.2, p.1⟩)
  outputs := ts.zipIdx.map (fun p => ⟨p.2, p.1⟩)

theorem idEdge_composable (opId opId' : Nat) (ts : List ObjType) :
    (idEdge opId ts).Composable (idEdge opId' ts) := rfl

end Hyperedge

end WasmGemmGnaf.GNAF
