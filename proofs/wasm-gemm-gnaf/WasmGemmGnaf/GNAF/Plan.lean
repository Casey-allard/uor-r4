import WasmGemmGnaf.GNAF.Edge
import WasmGemmGnaf.Cost.Objective
set_option autoImplicit false

/-!
# GNAF: the plan language (SPEC §11.1)

`Plan` is the constructive proposal language.  SPEC §11.1 requires it to
represent complete systems, and every item of its list is a constructor here:

| SPEC §11.1 item | constructor |
|---|---|
| raw-input classification | `classifyRaw` |
| shape/layout dispatch | `dispatchLayout` |
| packing and unpacking | `pack`, `unpack` |
| loop nests and index maps | `loopNest` (fixed extent) and `loopReg` (extent read from a register), both carrying an `IndexMap` |
| input load | `loadReg` (lift a memory word into a register) |
| blocking, tiling and traversal order | `tiled` (carrying `Tiling` and `TraversalOrder`) |
| reductions and arithmetic contracts | `reduce` (carrying an `ArithmeticContract`) |
| scratch allocation | `allocScratch` |
| scalar operations | `setReg`, `scalarOp` |
| standardized Wasm vector operations | `vectorOp` |
| code/data tables | `emitTable`, `tableLoad` |
| status construction | `setStatus` |
| output construction | `storeReg` (deposit a register into memory), `buildOutput` |

SPEC §11.1 also admits an embedded validated process as a canonical opaque node
**only when its exact semantics, resource behavior and cost certificate are
retained**, and insists that "opaque" is not "trusted".  `opaqueProcess`
therefore carries the exact `body` whose semantics it denotes together with a
*declared* cost and step certificate; `GNAF/Typing.lean` refuses the node unless
the declaration equals the value recomputed from the body.  Nothing about the
node is taken on trust.

This file also fixes the cost algebra: one recursive `charges` function assigns
each plan its nine primitive charges, and `costVector` maps them into the SPEC
§9.1 coordinates so that `certifiedCost` is the canonical objective's score.
-/

namespace WasmGemmGnaf.GNAF

open WasmGemmGnaf.Foundation

/-! ## Primitive operations -/

/-- The scalar operations of the plan language. -/
inductive ScalarOp
  | add | mul | sub | max | min
  deriving DecidableEq, Repr, Inhabited

namespace ScalarOp

/-- The exact meaning of a scalar operation on machine words.  Subtraction is
truncated, as `Nat` subtraction is; the plan language never has a negative
register. -/
def apply : ScalarOp → Nat → Nat → Nat
  | add, a, b => a + b
  | mul, a, b => a * b
  | sub, a, b => a - b
  | max, a, b => Nat.max a b
  | min, a, b => Nat.min a b

def all : List ScalarOp := [add, mul, sub, max, min]

theorem mem_all (o : ScalarOp) : o ∈ all := by cases o <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance : Foundation.Fintype ScalarOp where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

theorem apply_comm : ∀ (o : ScalarOp), (o = add ∨ o = mul ∨ o = max ∨ o = min) →
    ∀ a b : Nat, o.apply a b = o.apply b a := by
  intro o h a b
  rcases h with h | h | h | h <;> subst h <;>
    simp [apply, Nat.add_comm, Nat.mul_comm, Nat.max_comm, Nat.min_comm]

@[simp] theorem apply_add (a b : Nat) : add.apply a b = a + b := rfl
@[simp] theorem apply_mul (a b : Nat) : mul.apply a b = a * b := rfl

end ScalarOp

/-- The standardized Wasm vector (v128 lanewise) operations of the plan
language.  Each is defined by its lane-level scalar meaning; no lane may have a
meaning the scalar operation does not. -/
inductive VectorOp
  | addLanes | mulLanes | subLanes | maxLanes | minLanes
  deriving DecidableEq, Repr, Inhabited

namespace VectorOp

/-- The lane-level meaning of a vector operation. -/
def lane : VectorOp → ScalarOp
  | addLanes => .add
  | mulLanes => .mul
  | subLanes => .sub
  | maxLanes => .max
  | minLanes => .min

def all : List VectorOp := [addLanes, mulLanes, subLanes, maxLanes, minLanes]

theorem mem_all (o : VectorOp) : o ∈ all := by cases o <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance : Foundation.Fintype VectorOp where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

/-- The lane map is injective: two different vector operations never have the
same lane meaning. -/
theorem lane_injective : Function.Injective lane := by
  intro a b h; cases a <;> cases b <;> simp_all [lane]

end VectorOp

/-! ## Classification and conditions -/

/-- The raw-input classification outcome (SPEC §8.3): the four descriptor-level
classes on which a plan dispatches. -/
inductive Classification
  | valid | invalid | unsupported | resourceExhausted
  deriving DecidableEq, Repr, Inhabited

namespace Classification

/-- The released status a classification returns (SPEC §8.3). -/
def status : Classification → Status
  | valid => .success
  | invalid => .invalid
  | unsupported => .unsupported
  | resourceExhausted => .resourceExhausted

theorem status_injective : Function.Injective status := by
  intro a b h; cases a <;> cases b <;> simp_all [status]

def all : List Classification := [valid, invalid, unsupported, resourceExhausted]

theorem mem_all (c : Classification) : c ∈ all := by cases c <;> simp [all]

theorem all_nodup : all.Nodup := by decide

instance : Foundation.Fintype Classification where
  elemsList := all
  complete := mem_all
  nodupList := all_nodup

instance decidableForall (p : Classification → Prop) [DecidablePred p] :
    Decidable (∀ c, p c) :=
  decidable_of_iff (∀ c ∈ all, p c) ⟨fun h c => h c (mem_all c), fun h c _ => h c⟩

end Classification

/-- The closed set of decidable machine conditions a plan may branch on. -/
inductive Cond
  | statusIs (status : Status)
  | regEq (reg value : Nat)
  | regLt (reg value : Nat)
  | scratchAtLeast (bytes : Nat)
  deriving DecidableEq, Repr, Inhabited

namespace Cond

/-- Registers a condition reads. -/
def readsReg : Cond → Option Nat
  | regEq r _ => some r
  | regLt r _ => some r
  | _ => none

end Cond

/-! ## Loop axes and extents -/

/-- One axis of a loop nest: the register carrying the index, the extent, and
the index map used by the body (SPEC §11.1). -/
structure LoopAxis where
  indexReg : Nat
  extent : Nat
  map : IndexMap
  deriving DecidableEq, Repr, Inhabited

/-- The released static trip limit of a register-driven loop (`Plan.loopReg`).

A `loopReg` node reads its trip count out of a scalar register, so its trip
count is a function of the *input*, not of the plan text.  A plan language whose
step bound is a static number therefore cannot admit an unclamped
register-driven loop: `GNAF/Resource.lean` proves
`Plan.steps_le_stepBound`, and no fixed natural number bounds
`m.reg extentReg` over all configurations.  The evaluator of
`GNAF/Semantics.lean` consequently runs a register-driven loop
`Nat.min (m.reg extentReg) loopRegMaxTrips` times.

The constant is the released `i32` ceiling, and that is not a coincidence:
`GNAF/Compile.lean` represents every scalar register as an `i32` local, so a
register value above `loopRegMaxTrips` has no representation in the released
profile at all.  The clamp is therefore exactly the point at which the target
stops being able to express the extent, and it is vacuous on every configuration
the released ABI can present (`maxRawExtent` is the same number). -/
def loopRegMaxTrips : Nat := 4294967295

/-- The three GEMM extents a blocked traversal ranges over. -/
structure Extents where
  m : Nat
  n : Nat
  k : Nat
  deriving DecidableEq, Repr, Inhabited

/-- The retained certificate of an embedded opaque process (SPEC §11.1).  It is
*declared*, never trusted: `GNAF/Typing.lean` recomputes both numbers from the
retained body and rejects the node when they differ. -/
structure OpaqueSpec where
  processId : Nat
  declaredSteps : Nat
  declaredBytes : Nat
  deriving DecidableEq, Repr, Inhabited

/-! ## Plans -/

/-- A GNAF plan: the constructive proposal language of SPEC §11.1. -/
inductive Plan
  /-- The empty plan. -/
  | nop
  /-- Sequential composition. -/
  | seq (first second : Plan)
  /-- Raw-input classification: one continuation per SPEC §8.3 class. -/
  | classifyRaw (onValid onInvalid onUnsupported onExhausted : Plan)
  /-- Shape/layout dispatch: one continuation per layout class. -/
  | dispatchLayout (onRowMajor onColMajor onGeneral : Plan)
  /-- Decidable machine condition. -/
  | branch (cond : Cond) (onTrue onFalse : Plan)
  /-- Packing: copy a memory region into scratch through an index map. -/
  | pack (src dst : RegionRef) (map : IndexMap) (width : Nat)
  /-- Unpacking: copy a scratch region back into memory. -/
  | unpack (src dst : RegionRef) (map : IndexMap) (width : Nat)
  /-- Output store: write scalar register `src` into the memory region `dst`, at
  the address the index map gives under the current loop indices, in `width`
  little-endian bytes.  This is the constructor that lets a plan *deposit* a
  computed value — the accumulator of a `reduce` — into the observable store, so
  that "output construction" of SPEC §11.1 has something to construct from. -/
  | storeReg (dst : RegionRef) (map : IndexMap) (width : Nat) (src : Nat)
  /-- Input load: read `width` little-endian bytes of the memory region `src`,
  at the address the index map gives under the current loop indices, into scalar
  register `dst`.  This is the exact dual of `storeReg` — it reuses the same
  `Machine.storeAddr` addressing — and it is the constructor that lets a plan
  move a descriptor field (the ABI header's `m`, `n`, `k`) out of the observable
  store and into the register file, so that a plan can *compute with* its input
  and not merely branch on it. -/
  | loadReg (dst : Nat) (src : RegionRef) (map : IndexMap) (width : Nat)
  /-- A loop-nest axis with its index map. -/
  | loopNest (axis : LoopAxis) (body : Plan)
  /-- A loop whose trip count is read from a register rather than fixed by the
  plan text: iterate `body` with the counter in `indexReg`, for the number of
  iterations register `extentReg` holds when the loop is entered (clamped to
  `loopRegMaxTrips`, which the released `i32` profile can never exceed).  The
  extent is read *once*, before the loop, so evaluation is the existing
  `Plan.runLoop` on a natural number and termination stays structural. -/
  | loopReg (indexReg : Nat) (extentReg : Nat) (map : IndexMap) (body : Plan)
  /-- A blocked traversal in a declared order. -/
  | tiled (order : TraversalOrder) (tiling : Tiling) (extents : Extents) (body : Plan)
  /-- A dot-product reduction under an exact arithmetic contract. -/
  | reduce (contract : ArithmeticContract) (acc : Nat) (lhs rhs : RegionRef)
  /-- Scratch allocation for the enclosed plan. -/
  | allocScratch (bytes : Nat) (body : Plan)
  /-- Load an immediate into a register. -/
  | setReg (dst value : Nat)
  /-- A scalar operation. -/
  | scalarOp (op : ScalarOp) (dst lhs rhs : Nat)
  /-- A standardized Wasm vector operation on the given number of lanes. -/
  | vectorOp (op : VectorOp) (lanes dst lhs rhs : Nat)
  /-- A code/data table definition. -/
  | emitTable (index : Nat) (data : List Nat)
  /-- A load from a code/data table. -/
  | tableLoad (table index dst : Nat)
  /-- Status construction. -/
  | setStatus (status : Status)
  /-- Output construction from a memory region. -/
  | buildOutput (src : RegionRef)
  /-- A canonical opaque process node retaining its exact body and its declared
  resource/cost certificate. -/
  | opaqueProcess (spec : OpaqueSpec) (body : Plan)
  deriving DecidableEq, Repr, Inhabited

/-! ### Two natural-number maximum facts used by the measures -/

theorem max_le_add (a b : Nat) : Nat.max a b ≤ a + b :=
  Nat.max_le.mpr ⟨Nat.le_add_right a b, Nat.le_add_left b a⟩

theorem max_le_max {a a' b b' : Nat} (ha : a ≤ a') (hb : b ≤ b') :
    Nat.max a b ≤ Nat.max a' b' :=
  Nat.max_le.mpr
    ⟨Nat.le_trans ha (Nat.le_max_left _ _), Nat.le_trans hb (Nat.le_max_right _ _)⟩

namespace Plan

/-! ### Structural measures -/

/-- The number of nodes of a plan: the explicit well-founded measure used by
`GNAF/Resource.lean`. -/
def size : Plan → Nat
  | nop => 1
  | seq a b => 1 + a.size + b.size
  | classifyRaw v i u e => 1 + v.size + i.size + u.size + e.size
  | dispatchLayout r c g => 1 + r.size + c.size + g.size
  | branch _ t f => 1 + t.size + f.size
  | pack _ _ _ _ => 1
  | unpack _ _ _ _ => 1
  | storeReg _ _ _ _ => 1
  | loadReg _ _ _ _ => 1
  | loopNest _ body => 1 + body.size
  | loopReg _ _ _ body => 1 + body.size
  | tiled _ _ _ body => 1 + body.size
  | reduce _ _ _ _ => 1
  | allocScratch _ body => 1 + body.size
  | setReg _ _ => 1
  | scalarOp _ _ _ _ => 1
  | vectorOp _ _ _ _ _ => 1
  | emitTable _ _ => 1
  | tableLoad _ _ _ => 1
  | setStatus _ => 1
  | buildOutput _ => 1
  | opaqueProcess _ body => 1 + body.size

theorem size_pos (p : Plan) : 0 < p.size := by
  cases p <;> simp [size] <;> omega

/-- The nesting depth of a plan. -/
def depth : Plan → Nat
  | nop => 0
  | seq a b => 1 + Nat.max a.depth b.depth
  | classifyRaw v i u e =>
      1 + Nat.max (Nat.max v.depth i.depth) (Nat.max u.depth e.depth)
  | dispatchLayout r c g => 1 + Nat.max r.depth (Nat.max c.depth g.depth)
  | branch _ t f => 1 + Nat.max t.depth f.depth
  | pack _ _ _ _ => 0
  | unpack _ _ _ _ => 0
  | storeReg _ _ _ _ => 0
  | loadReg _ _ _ _ => 0
  | loopNest _ body => 1 + body.depth
  | loopReg _ _ _ body => 1 + body.depth
  | tiled _ _ _ body => 1 + body.depth
  | reduce _ _ _ _ => 0
  | allocScratch _ body => 1 + body.depth
  | setReg _ _ => 0
  | scalarOp _ _ _ _ => 0
  | vectorOp _ _ _ _ _ => 0
  | emitTable _ _ => 0
  | tableLoad _ _ _ => 0
  | setStatus _ => 0
  | buildOutput _ => 0
  | opaqueProcess _ body => 1 + body.depth

theorem depth_lt_size (p : Plan) : p.depth < p.size := by
  induction p with
  | nop => simp [depth, size]
  | seq a b iha ihb =>
    have h := max_le_add a.depth b.depth
    simp only [depth, size]; omega
  | classifyRaw v i u e ihv ihi ihu ihe =>
    have h1 := max_le_add v.depth i.depth
    have h2 := max_le_add u.depth e.depth
    have h3 := max_le_add (Nat.max v.depth i.depth) (Nat.max u.depth e.depth)
    simp only [depth, size]; omega
  | dispatchLayout r c g ihr ihc ihg =>
    have h1 := max_le_add c.depth g.depth
    have h2 := max_le_add r.depth (Nat.max c.depth g.depth)
    simp only [depth, size]; omega
  | branch _ t f iht ihf =>
    have h := max_le_add t.depth f.depth
    simp only [depth, size]; omega
  | pack _ _ _ _ => simp [depth, size]
  | unpack _ _ _ _ => simp [depth, size]
  | storeReg _ _ _ _ => simp [depth, size]
  | loadReg _ _ _ _ => simp [depth, size]
  | loopNest _ body ih => simp only [depth, size]; omega
  | loopReg _ _ _ body ih => simp only [depth, size]; omega
  | tiled _ _ _ body ih => simp only [depth, size]; omega
  | reduce _ _ _ _ => simp [depth, size]
  | allocScratch _ body ih => simp only [depth, size]; omega
  | setReg _ _ => simp [depth, size]
  | scalarOp _ _ _ _ => simp [depth, size]
  | vectorOp _ _ _ _ _ => simp [depth, size]
  | emitTable _ _ => simp [depth, size]
  | tableLoad _ _ _ => simp [depth, size]
  | setStatus _ => simp [depth, size]
  | buildOutput _ => simp [depth, size]
  | opaqueProcess _ body ih => simp only [depth, size]; omega

end Plan

/-! ## The charge algebra

One recursive function assigns every plan its nine primitive charges.  The
SPEC §9.1 cost vector is then a fixed projection of them, so every cost
inequality in `GNAF/Normalize.lean` reduces to nine natural-number
inequalities. -/

/-- The nine primitive charges of a plan. -/
structure Charges where
  moduleBytes : Nat
  dataBytes : Nat
  steps : Nat
  scalarOps : Nat
  laneOps : Nat
  bytesRead : Nat
  bytesWritten : Nat
  scratchPeak : Nat
  outputBytes : Nat
  deriving DecidableEq, Repr, Inhabited

namespace Charges

def zero : Charges :=
  { moduleBytes := 0, dataBytes := 0, steps := 0, scalarOps := 0, laneOps := 0
    bytesRead := 0, bytesWritten := 0, scratchPeak := 0, outputBytes := 0 }

/-- Accumulation of charges.  Every coordinate adds, including the scratch
peak: `Plan.eval` never releases scratch, so an additive peak is the sound
composition here (SPEC §9.1's maximum rule is the *tighter* rule and would not
bound this evaluator). -/
def add (a b : Charges) : Charges :=
  { moduleBytes := a.moduleBytes + b.moduleBytes
    dataBytes := a.dataBytes + b.dataBytes
    steps := a.steps + b.steps
    scalarOps := a.scalarOps + b.scalarOps
    laneOps := a.laneOps + b.laneOps
    bytesRead := a.bytesRead + b.bytesRead
    bytesWritten := a.bytesWritten + b.bytesWritten
    scratchPeak := a.scratchPeak + b.scratchPeak
    outputBytes := a.outputBytes + b.outputBytes }

/-- Repetition: the dynamic charges scale with the iteration count, the static
charges do not. -/
def iterate (c : Charges) (n : Nat) : Charges :=
  { moduleBytes := c.moduleBytes
    dataBytes := c.dataBytes
    steps := c.steps * n
    scalarOps := c.scalarOps * n
    laneOps := c.laneOps * n
    bytesRead := c.bytesRead * n
    bytesWritten := c.bytesWritten * n
    scratchPeak := c.scratchPeak * n
    outputBytes := c.outputBytes * n }

/-- The charge of one emitted node: its code bytes and one executed step. -/
def node (bytes : Nat) : Charges := { zero with moduleBytes := bytes, steps := 1 }

@[simp] theorem add_zero_left (c : Charges) : add zero c = c := by
  cases c; simp [add, zero]

@[simp] theorem add_zero_right (c : Charges) : add c zero = c := by
  cases c; simp [add, zero]

theorem add_assoc (a b c : Charges) : add (add a b) c = add a (add b c) := by
  cases a; cases b; cases c
  simp [add, Nat.add_assoc]

@[simp] theorem iterate_zero_steps (c : Charges) : (iterate c 0).steps = 0 := by
  simp [iterate]

@[simp] theorem iterate_zero_scalarOps (c : Charges) : (iterate c 0).scalarOps = 0 := by
  simp [iterate]

@[simp] theorem iterate_moduleBytes (c : Charges) (n : Nat) :
    (iterate c n).moduleBytes = c.moduleBytes := rfl

@[simp] theorem iterate_one (c : Charges) : iterate c 1 = c := by
  cases c; simp [iterate]

/-- Componentwise order on charges. -/
def LE (a b : Charges) : Prop :=
  a.moduleBytes ≤ b.moduleBytes ∧ a.dataBytes ≤ b.dataBytes ∧
  a.steps ≤ b.steps ∧ a.scalarOps ≤ b.scalarOps ∧ a.laneOps ≤ b.laneOps ∧
  a.bytesRead ≤ b.bytesRead ∧ a.bytesWritten ≤ b.bytesWritten ∧
  a.scratchPeak ≤ b.scratchPeak ∧ a.outputBytes ≤ b.outputBytes

instance (a b : Charges) : Decidable (LE a b) := by unfold LE; infer_instance

theorem le_refl (a : Charges) : LE a a :=
  ⟨Nat.le_refl _, Nat.le_refl _, Nat.le_refl _, Nat.le_refl _, Nat.le_refl _,
   Nat.le_refl _, Nat.le_refl _, Nat.le_refl _, Nat.le_refl _⟩

theorem le_trans {a b c : Charges} (hab : LE a b) (hbc : LE b c) : LE a c :=
  ⟨Nat.le_trans hab.1 hbc.1, Nat.le_trans hab.2.1 hbc.2.1,
   Nat.le_trans hab.2.2.1 hbc.2.2.1, Nat.le_trans hab.2.2.2.1 hbc.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.1 hbc.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.1 hbc.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.1 hbc.2.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.2.1 hbc.2.2.2.2.2.2.2.1,
   Nat.le_trans hab.2.2.2.2.2.2.2.2 hbc.2.2.2.2.2.2.2.2⟩

theorem zero_le (a : Charges) : LE zero a :=
  ⟨Nat.zero_le _, Nat.zero_le _, Nat.zero_le _, Nat.zero_le _, Nat.zero_le _,
   Nat.zero_le _, Nat.zero_le _, Nat.zero_le _, Nat.zero_le _⟩

theorem add_mono {a a' b b' : Charges} (ha : LE a a') (hb : LE b b') :
    LE (add a b) (add a' b') :=
  ⟨Nat.add_le_add ha.1 hb.1, Nat.add_le_add ha.2.1 hb.2.1,
   Nat.add_le_add ha.2.2.1 hb.2.2.1, Nat.add_le_add ha.2.2.2.1 hb.2.2.2.1,
   Nat.add_le_add ha.2.2.2.2.1 hb.2.2.2.2.1,
   Nat.add_le_add ha.2.2.2.2.2.1 hb.2.2.2.2.2.1,
   Nat.add_le_add ha.2.2.2.2.2.2.1 hb.2.2.2.2.2.2.1,
   Nat.add_le_add ha.2.2.2.2.2.2.2.1 hb.2.2.2.2.2.2.2.1,
   Nat.add_le_add ha.2.2.2.2.2.2.2.2 hb.2.2.2.2.2.2.2.2⟩

theorem le_add_right (a b : Charges) : LE a (add a b) :=
  ⟨Nat.le_add_right _ _, Nat.le_add_right _ _, Nat.le_add_right _ _,
   Nat.le_add_right _ _, Nat.le_add_right _ _, Nat.le_add_right _ _,
   Nat.le_add_right _ _, Nat.le_add_right _ _, Nat.le_add_right _ _⟩

theorem le_add_left (a b : Charges) : LE b (add a b) :=
  ⟨Nat.le_add_left _ _, Nat.le_add_left _ _, Nat.le_add_left _ _,
   Nat.le_add_left _ _, Nat.le_add_left _ _, Nat.le_add_left _ _,
   Nat.le_add_left _ _, Nat.le_add_left _ _, Nat.le_add_left _ _⟩

theorem iterate_mono {a b : Charges} (h : LE a b) (n : Nat) :
    LE (iterate a n) (iterate b n) :=
  ⟨h.1, h.2.1,
   Nat.mul_le_mul h.2.2.1 (Nat.le_refl n),
   Nat.mul_le_mul h.2.2.2.1 (Nat.le_refl n),
   Nat.mul_le_mul h.2.2.2.2.1 (Nat.le_refl n),
   Nat.mul_le_mul h.2.2.2.2.2.1 (Nat.le_refl n),
   Nat.mul_le_mul h.2.2.2.2.2.2.1 (Nat.le_refl n),
   Nat.mul_le_mul h.2.2.2.2.2.2.2.1 (Nat.le_refl n),
   Nat.mul_le_mul h.2.2.2.2.2.2.2.2 (Nat.le_refl n)⟩

end Charges

namespace Plan

/-- The primitive charges of a plan (SPEC §9.1 coordinates, before projection).

Branch charges are *summed* over the continuations: every continuation is
emitted, and summing is a sound upper bound for the executed branch. -/
def charges : Plan → Charges
  | nop => Charges.zero
  | seq a b => Charges.add a.charges b.charges
  | classifyRaw v i u e =>
      Charges.add (Charges.node 4)
        (Charges.add (Charges.add v.charges i.charges)
          (Charges.add u.charges e.charges))
  | dispatchLayout r c g =>
      Charges.add (Charges.node 3)
        (Charges.add r.charges (Charges.add c.charges g.charges))
  | branch _ t f => Charges.add (Charges.node 2) (Charges.add t.charges f.charges)
  | pack src _ _ width =>
      { Charges.node 3 with
        bytesRead := src.count * width, bytesWritten := src.count * width
        scratchPeak := src.count * width }
  | unpack src _ _ width =>
      { Charges.node 3 with
        bytesRead := src.count * width, bytesWritten := src.count * width }
  | storeReg _ _ width _ => { Charges.node 3 with bytesWritten := width }
  | loadReg _ _ _ width => { Charges.node 3 with bytesRead := width }
  | loopNest axis body => Charges.add (Charges.node 2) (body.charges.iterate axis.extent)
  | loopReg _ _ _ body =>
      Charges.add (Charges.node 2) (body.charges.iterate loopRegMaxTrips)
  | tiled _ tiling extents body =>
      Charges.add (Charges.node 3)
        (body.charges.iterate (tiling.totalTiles extents.m extents.n extents.k))
  | reduce _ _ lhs rhs =>
      { Charges.node 3 with
        steps := 1 + Nat.min lhs.count rhs.count
        scalarOps := Nat.min lhs.count rhs.count
        bytesRead := 2 * Nat.min lhs.count rhs.count }
  | allocScratch bytes body =>
      Charges.add { Charges.node 2 with scratchPeak := bytes } body.charges
  | setReg _ _ => { Charges.node 1 with scalarOps := 1 }
  | scalarOp _ _ _ _ => { Charges.node 1 with scalarOps := 1 }
  | vectorOp _ lanes _ _ _ => { Charges.node 2 with laneOps := lanes }
  | emitTable _ data => { Charges.node 1 with dataBytes := data.length }
  | tableLoad _ _ _ => { Charges.node 2 with bytesRead := 1 }
  | setStatus _ => { Charges.node 1 with bytesWritten := 4 }
  | buildOutput src => { Charges.node 2 with outputBytes := src.count }
  | opaqueProcess _ body => Charges.add (Charges.node 4) body.charges

/-- The static coordinates of SPEC §9.1. -/
def staticCost (p : Plan) : Cost.StaticVector :=
  { moduleBytes := p.charges.moduleBytes
    decodeSteps := p.charges.moduleBytes
    validationSteps := p.charges.moduleBytes
    staticDataBytes := p.charges.dataBytes }

/-- The dynamic coordinates of SPEC §9.1. -/
def dynamicCost (p : Plan) : Cost.DynamicVector :=
  { instantiationSteps := p.charges.dataBytes
    dispatchSteps := p.charges.steps
    preparationSteps := p.charges.scratchPeak
    wasmRuleSteps := p.charges.steps
    scalarOps := p.charges.scalarOps
    vectorLaneOps := p.charges.laneOps
    bytesRead := p.charges.bytesRead
    bytesWritten := p.charges.bytesWritten
    memoryGrowPages := p.charges.scratchPeak
    tableElementsAllocated := p.charges.dataBytes
    gcObjectsAllocated := 0
    gcBytesInitialized := 0
    peakStackValues := p.charges.scalarOps
    peakPages := p.charges.scratchPeak
    peakGcLiveBytes := 0
    outputBytes := p.charges.outputBytes }

/-- The complete SPEC §9.1 artifact cost vector of a plan. -/
def costVector (p : Plan) : Cost.ArtifactVector :=
  { static := p.staticCost
    dynamicSum := p.dynamicCost
    dynamicMax := p.dynamicCost }

/-- The certified cost of a plan under the canonical objective (SPEC §9.2). -/
def certifiedCost (p : Plan) : Nat := Cost.CanonicalObjective.score p.costVector

/-- The static step bound of a plan: the resource contract discharged in
`GNAF/Resource.lean`. -/
def stepBound (p : Plan) : Nat := p.charges.steps

/-- The static scratch bound of a plan. -/
def scratchBound (p : Plan) : Nat := p.charges.scratchPeak

@[simp] theorem stepBound_nop : (nop).stepBound = 0 := rfl

@[simp] theorem charges_nop : (nop).charges = Charges.zero := rfl

@[simp] theorem charges_seq (a b : Plan) :
    (seq a b).charges = Charges.add a.charges b.charges := rfl

end Plan

/-! ## Structural notions used by normalization -/

namespace Plan

/-- The immediate-subplan relation: the edges along which `Plan.eval` recurses.
`GNAF/Resource.lean` proves it well founded with the explicit measure
`Plan.size`. -/
inductive Sub : Plan → Plan → Prop
  | seqL (a b : Plan) : Sub a (seq a b)
  | seqR (a b : Plan) : Sub b (seq a b)
  | classify0 (v i u e : Plan) : Sub v (classifyRaw v i u e)
  | classify1 (v i u e : Plan) : Sub i (classifyRaw v i u e)
  | classify2 (v i u e : Plan) : Sub u (classifyRaw v i u e)
  | classify3 (v i u e : Plan) : Sub e (classifyRaw v i u e)
  | layout0 (r c g : Plan) : Sub r (dispatchLayout r c g)
  | layout1 (r c g : Plan) : Sub c (dispatchLayout r c g)
  | layout2 (r c g : Plan) : Sub g (dispatchLayout r c g)
  | branchT (co : Cond) (t f : Plan) : Sub t (branch co t f)
  | branchF (co : Cond) (t f : Plan) : Sub f (branch co t f)
  | loopBody (ax : LoopAxis) (b : Plan) : Sub b (loopNest ax b)
  | loopRegBody (ir er : Nat) (mp : IndexMap) (b : Plan) : Sub b (loopReg ir er mp b)
  | tiledBody (o : TraversalOrder) (ti : Tiling) (ex : Extents) (b : Plan) :
      Sub b (tiled o ti ex b)
  | allocBody (n : Nat) (b : Plan) : Sub b (allocScratch n b)
  | opaqueBody (s : OpaqueSpec) (b : Plan) : Sub b (opaqueProcess s b)

/-- Every immediate subplan is strictly smaller. -/
theorem Sub.size_lt {a b : Plan} (h : Sub a b) : a.size < b.size := by
  cases h <;> simp only [size] <;> omega

end Plan

end WasmGemmGnaf.GNAF
