import WasmGemmGnaf.GNAF.Resource
set_option autoImplicit false

/-!
# GNAF: typing (SPEC §11.2)

SPEC §11.2: "Every plan SHALL carry exact input/output types and resource
transitions."  A `Sig` is that interface — the exact input and output type
instances together with the register, vector-register, lane, scratch, table and
memory resources, and the status flag.  `HasType s p t` is the typing judgment:
running `p` at interface `s` leaves interface `t`.

The judgment is *decidable*: `infer` is a total decision procedure, proved

* sound (`infer_sound`: whatever it accepts really is derivable), and
* complete (`infer_complete`: whatever is derivable it accepts),

so `hasType_iff_infer` makes the relation and the procedure interchangeable and
`instDecidableHasType` follows.  Nothing is assumed about the procedure.

The opaque-process rule is where SPEC §11.1's "opaque SHALL not mean trusted"
becomes formal: `HasType` admits `opaqueProcess spec body` only when the
declared step and byte certificate equal the values *recomputed* from the
retained body.

The final section proves the judgment sound for the semantics of
`GNAF/Semantics.lean`: `Machine.Conforms` links a configuration to an
interface, and `hasType_preservation` shows a well-typed plan maps conforming
configurations to conforming configurations.
-/

namespace WasmGemmGnaf.GNAF

open WasmGemmGnaf.Foundation

/-! ## Interfaces -/

/-- The exact interface of a plan (SPEC §11.2): input and output type
instances, plus the complete resource state. -/
structure Sig where
  /-- The type instance of the raw input. -/
  inputType : ObjType
  /-- The type instance of the constructed output. -/
  outputType : ObjType
  /-- Size of the scalar register file. -/
  regs : Nat
  /-- Size of the vector register file. -/
  vregs : Nat
  /-- Maximum admitted vector lane count. -/
  lanes : Nat
  /-- Declared scratch extent. -/
  scratch : Nat
  /-- Number of code/data tables. -/
  tables : Nat
  /-- Observable memory extent. -/
  mem : Nat
  /-- Whether a status has been constructed. -/
  statusSet : Bool
  deriving DecidableEq, Repr, Inhabited

namespace Cond

/-- A condition is well typed at an interface when every register it reads
exists. -/
def wellTyped (c : Cond) (s : Sig) : Bool :=
  match c with
  | regEq r _ => decide (r < s.regs)
  | regLt r _ => decide (r < s.regs)
  | _ => true

end Cond

/-! ## The typing judgment -/

/-- `HasType s p t`: at input interface `s`, plan `p` is well typed and leaves
output interface `t` (SPEC §11.2). -/
inductive HasType : Sig → Plan → Sig → Prop
  | nop (s : Sig) : HasType s .nop s
  | seq {s u t : Sig} {a b : Plan} :
      HasType s a u → HasType u b t → HasType s (.seq a b) t
  | classifyRaw {s t : Sig} {v i u e : Plan} :
      HasType s v t → HasType s i t → HasType s u t → HasType s e t →
      HasType s (.classifyRaw v i u e) t
  | dispatchLayout {s t : Sig} {r c g : Plan} :
      HasType s r t → HasType s c t → HasType s g t →
      HasType s (.dispatchLayout r c g) t
  | branch {s t : Sig} {co : Cond} {x y : Plan} :
      co.wellTyped s = true → HasType s x t → HasType s y t →
      HasType s (.branch co x y) t
  | pack {s : Sig} {src dst : RegionRef} {map : IndexMap} {w : Nat} :
      src.Fits s.mem → dst.Fits s.scratch → 0 < w →
      HasType s (.pack src dst map w) s
  | unpack {s : Sig} {src dst : RegionRef} {map : IndexMap} {w : Nat} :
      src.Fits s.scratch → dst.Fits s.mem → 0 < w →
      HasType s (.unpack src dst map w) s
  | storeReg {s : Sig} {dst : RegionRef} {map : IndexMap} {w src : Nat} :
      dst.Fits s.mem → 0 < w → src < s.regs → 2 < s.regs →
      HasType s (.storeReg dst map w src) s
  | loadReg {s : Sig} {dst : Nat} {src : RegionRef} {map : IndexMap} {w : Nat} :
      src.Fits s.mem → 0 < w → dst < s.regs → 2 < s.regs →
      HasType s (.loadReg dst src map w) s
  | loopNest {s : Sig} {axis : LoopAxis} {body : Plan} :
      axis.indexReg < s.regs → HasType s body s → HasType s (.loopNest axis body) s
  | loopReg {s : Sig} {ir er : Nat} {map : IndexMap} {body : Plan} :
      ir < s.regs → er < s.regs → HasType s body s →
      HasType s (.loopReg ir er map body) s
  | tiled {s : Sig} {o : TraversalOrder} {ti : Tiling} {ex : Extents} {body : Plan} :
      ti.Positive → HasType s body s → HasType s (.tiled o ti ex body) s
  | reduce {s : Sig} {c : ArithmeticContract} {acc : Nat} {lhs rhs : RegionRef} :
      acc < s.regs → c.compatibleB = true → lhs.Fits s.mem → rhs.Fits s.mem →
      HasType s (.reduce c acc lhs rhs) s
  | allocScratch {s t : Sig} {n : Nat} {body : Plan} :
      HasType { s with scratch := s.scratch + n } body t →
      HasType s (.allocScratch n body) t
  | setReg {s : Sig} {dst v : Nat} : dst < s.regs → HasType s (.setReg dst v) s
  | scalarOp {s : Sig} {op : ScalarOp} {dst a b : Nat} :
      dst < s.regs → a < s.regs → b < s.regs → HasType s (.scalarOp op dst a b) s
  | vectorOp {s : Sig} {op : VectorOp} {lanes dst a b : Nat} :
      0 < lanes → lanes ≤ s.lanes → dst < s.vregs → a < s.vregs → b < s.vregs →
      HasType s (.vectorOp op lanes dst a b) s
  | emitTable {s : Sig} {idx : Nat} {data : List Nat} :
      idx < s.tables → HasType s (.emitTable idx data) s
  | tableLoad {s : Sig} {tbl idx dst : Nat} :
      tbl < s.tables → dst < s.regs → HasType s (.tableLoad tbl idx dst) s
  | setStatus (s : Sig) (st : Status) :
      HasType s (.setStatus st) { s with statusSet := true }
  | buildOutput {s : Sig} {src : RegionRef} :
      s.statusSet = true → src.Fits s.mem →
      HasType s (.buildOutput src) { s with outputType := .bytes src.count }
  | opaqueProcess {s t : Sig} {spec : OpaqueSpec} {body : Plan} :
      spec.declaredSteps = body.stepBound →
      spec.declaredBytes = body.charges.moduleBytes →
      HasType s body t → HasType s (.opaqueProcess spec body) t

/-! ## The decision procedure -/

/-- Join of two inferred interfaces: defined only when both exist and agree. -/
def joinSig : Option Sig → Option Sig → Option Sig
  | some a, some b => if a = b then some a else none
  | _, _ => none

theorem joinSig_eq_some {x y : Option Sig} {t : Sig} :
    joinSig x y = some t ↔ x = some t ∧ y = some t := by
  cases x with
  | none => simp [joinSig]
  | some a =>
    cases y with
    | none => simp [joinSig]
    | some b =>
      by_cases h : a = b
      · subst h; simp [joinSig]
      · simp [joinSig, h]
        intro ht
        subst ht
        exact fun hb => absurd hb.symm h

/-- The typing decision procedure: it computes the unique output interface, or
rejects. -/
def infer : Sig → Plan → Option Sig
  | s, .nop => some s
  | s, .seq a b =>
      match infer s a with
      | some u => infer u b
      | none => none
  | s, .classifyRaw v i u e =>
      joinSig (joinSig (infer s v) (infer s i)) (joinSig (infer s u) (infer s e))
  | s, .dispatchLayout r c g =>
      joinSig (infer s r) (joinSig (infer s c) (infer s g))
  | s, .branch co x y =>
      if co.wellTyped s = true then joinSig (infer s x) (infer s y) else none
  | s, .pack src dst _ w =>
      if src.Fits s.mem ∧ dst.Fits s.scratch ∧ 0 < w then some s else none
  | s, .unpack src dst _ w =>
      if src.Fits s.scratch ∧ dst.Fits s.mem ∧ 0 < w then some s else none
  | s, .storeReg dst _ w src =>
      if dst.Fits s.mem ∧ 0 < w ∧ src < s.regs ∧ 2 < s.regs then some s else none
  | s, .loadReg dst src _ w =>
      if src.Fits s.mem ∧ 0 < w ∧ dst < s.regs ∧ 2 < s.regs then some s else none
  | s, .loopNest axis body =>
      if axis.indexReg < s.regs ∧ infer s body = some s then some s else none
  | s, .loopReg ir er _ body =>
      if ir < s.regs ∧ er < s.regs ∧ infer s body = some s then some s else none
  | s, .tiled _ ti _ body =>
      if ti.Positive ∧ infer s body = some s then some s else none
  | s, .reduce c acc lhs rhs =>
      if acc < s.regs ∧ c.compatibleB = true ∧ lhs.Fits s.mem ∧ rhs.Fits s.mem then
        some s else none
  | s, .allocScratch n body => infer { s with scratch := s.scratch + n } body
  | s, .setReg dst _ => if dst < s.regs then some s else none
  | s, .scalarOp _ dst a b =>
      if dst < s.regs ∧ a < s.regs ∧ b < s.regs then some s else none
  | s, .vectorOp _ lanes dst a b =>
      if 0 < lanes ∧ lanes ≤ s.lanes ∧ dst < s.vregs ∧ a < s.vregs ∧ b < s.vregs then
        some s else none
  | s, .emitTable idx _ => if idx < s.tables then some s else none
  | s, .tableLoad tbl _ dst => if tbl < s.tables ∧ dst < s.regs then some s else none
  | s, .setStatus _ => some { s with statusSet := true }
  | s, .buildOutput src =>
      if s.statusSet = true ∧ src.Fits s.mem then
        some { s with outputType := .bytes src.count } else none
  | s, .opaqueProcess spec body =>
      if spec.declaredSteps = body.stepBound ∧
          spec.declaredBytes = body.charges.moduleBytes then infer s body else none

/-- **Soundness.**  Everything the decision procedure accepts is derivable. -/
theorem infer_sound : ∀ (p : Plan) (s t : Sig), infer s p = some t → HasType s p t := by
  intro p
  induction p with
  | nop =>
    intro s t h
    rw [infer] at h
    cases h
    exact .nop s
  | seq a b iha ihb =>
    intro s t h
    rw [infer] at h
    cases hu : infer s a with
    | none => rw [hu] at h; exact absurd h (by simp)
    | some u =>
      rw [hu] at h
      exact .seq (iha s u hu) (ihb u t h)
  | classifyRaw v i u e ihv ihi ihu ihe =>
    intro s t h
    rw [infer, joinSig_eq_some, joinSig_eq_some, joinSig_eq_some] at h
    exact .classifyRaw (ihv s t h.1.1) (ihi s t h.1.2) (ihu s t h.2.1) (ihe s t h.2.2)
  | dispatchLayout r c g ihr ihc ihg =>
    intro s t h
    rw [infer, joinSig_eq_some, joinSig_eq_some] at h
    exact .dispatchLayout (ihr s t h.1) (ihc s t h.2.1) (ihg s t h.2.2)
  | branch co x y ihx ihy =>
    intro s t h
    rw [infer] at h
    by_cases hc : co.wellTyped s = true
    · rw [if_pos hc, joinSig_eq_some] at h
      exact .branch hc (ihx s t h.1) (ihy s t h.2)
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | pack src dst map w =>
    intro s t h
    rw [infer] at h
    by_cases hc : src.Fits s.mem ∧ dst.Fits s.scratch ∧ 0 < w
    · rw [if_pos hc] at h; cases h; exact .pack hc.1 hc.2.1 hc.2.2
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | unpack src dst map w =>
    intro s t h
    rw [infer] at h
    by_cases hc : src.Fits s.scratch ∧ dst.Fits s.mem ∧ 0 < w
    · rw [if_pos hc] at h; cases h; exact .unpack hc.1 hc.2.1 hc.2.2
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | storeReg dst map w src =>
    intro s t h
    rw [infer] at h
    by_cases hc : dst.Fits s.mem ∧ 0 < w ∧ src < s.regs ∧ 2 < s.regs
    · rw [if_pos hc] at h; cases h; exact .storeReg hc.1 hc.2.1 hc.2.2.1 hc.2.2.2
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | loadReg dst src map w =>
    intro s t h
    rw [infer] at h
    by_cases hc : src.Fits s.mem ∧ 0 < w ∧ dst < s.regs ∧ 2 < s.regs
    · rw [if_pos hc] at h; cases h; exact .loadReg hc.1 hc.2.1 hc.2.2.1 hc.2.2.2
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | loopNest axis body ih =>
    intro s t h
    rw [infer] at h
    by_cases hc : axis.indexReg < s.regs ∧ infer s body = some s
    · rw [if_pos hc] at h; cases h; exact .loopNest hc.1 (ih s s hc.2)
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | loopReg ir er map body ih =>
    intro s t h
    rw [infer] at h
    by_cases hc : ir < s.regs ∧ er < s.regs ∧ infer s body = some s
    · rw [if_pos hc] at h; cases h; exact .loopReg hc.1 hc.2.1 (ih s s hc.2.2)
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | tiled o ti ex body ih =>
    intro s t h
    rw [infer] at h
    by_cases hc : ti.Positive ∧ infer s body = some s
    · rw [if_pos hc] at h; cases h; exact .tiled hc.1 (ih s s hc.2)
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | reduce c acc lhs rhs =>
    intro s t h
    rw [infer] at h
    by_cases hc : acc < s.regs ∧ c.compatibleB = true ∧ lhs.Fits s.mem ∧ rhs.Fits s.mem
    · rw [if_pos hc] at h; cases h; exact .reduce hc.1 hc.2.1 hc.2.2.1 hc.2.2.2
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | allocScratch n body ih =>
    intro s t h
    rw [infer] at h
    exact .allocScratch (ih _ t h)
  | setReg dst v =>
    intro s t h
    rw [infer] at h
    by_cases hc : dst < s.regs
    · rw [if_pos hc] at h; cases h; exact .setReg hc
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | scalarOp op dst a b =>
    intro s t h
    rw [infer] at h
    by_cases hc : dst < s.regs ∧ a < s.regs ∧ b < s.regs
    · rw [if_pos hc] at h; cases h; exact .scalarOp hc.1 hc.2.1 hc.2.2
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | vectorOp op lanes dst a b =>
    intro s t h
    rw [infer] at h
    by_cases hc : 0 < lanes ∧ lanes ≤ s.lanes ∧ dst < s.vregs ∧ a < s.vregs ∧ b < s.vregs
    · rw [if_pos hc] at h; cases h
      exact .vectorOp hc.1 hc.2.1 hc.2.2.1 hc.2.2.2.1 hc.2.2.2.2
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | emitTable idx data =>
    intro s t h
    rw [infer] at h
    by_cases hc : idx < s.tables
    · rw [if_pos hc] at h; cases h; exact .emitTable hc
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | tableLoad tbl idx dst =>
    intro s t h
    rw [infer] at h
    by_cases hc : tbl < s.tables ∧ dst < s.regs
    · rw [if_pos hc] at h; cases h; exact .tableLoad hc.1 hc.2
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | setStatus st =>
    intro s t h
    rw [infer] at h
    cases h
    exact .setStatus s st
  | buildOutput src =>
    intro s t h
    rw [infer] at h
    by_cases hc : s.statusSet = true ∧ src.Fits s.mem
    · rw [if_pos hc] at h; cases h; exact .buildOutput hc.1 hc.2
    · rw [if_neg hc] at h; exact absurd h (by simp)
  | opaqueProcess spec body ih =>
    intro s t h
    rw [infer] at h
    by_cases hc : spec.declaredSteps = body.stepBound ∧
        spec.declaredBytes = body.charges.moduleBytes
    · rw [if_pos hc] at h; exact .opaqueProcess hc.1 hc.2 (ih s t h)
    · rw [if_neg hc] at h; exact absurd h (by simp)

/-- **Completeness.**  Everything derivable is accepted by the decision
procedure. -/
theorem infer_complete {s : Sig} {p : Plan} {t : Sig} (h : HasType s p t) :
    infer s p = some t := by
  induction h with
  | nop s => rfl
  | seq _ _ iha ihb => rw [infer, iha]; exact ihb
  | classifyRaw _ _ _ _ ihv ihi ihu ihe =>
    rw [infer, joinSig_eq_some, joinSig_eq_some, joinSig_eq_some]
    exact ⟨⟨ihv, ihi⟩, ihu, ihe⟩
  | dispatchLayout _ _ _ ihr ihc ihg =>
    rw [infer, joinSig_eq_some, joinSig_eq_some]
    exact ⟨ihr, ihc, ihg⟩
  | branch hco _ _ ihx ihy =>
    rw [infer, if_pos hco, joinSig_eq_some]
    exact ⟨ihx, ihy⟩
  | pack h1 h2 h3 => rw [infer, if_pos ⟨h1, h2, h3⟩]
  | unpack h1 h2 h3 => rw [infer, if_pos ⟨h1, h2, h3⟩]
  | storeReg h1 h2 h3 h4 => rw [infer, if_pos ⟨h1, h2, h3, h4⟩]
  | loadReg h1 h2 h3 h4 => rw [infer, if_pos ⟨h1, h2, h3, h4⟩]
  | loopNest h1 _ ih => rw [infer, if_pos ⟨h1, ih⟩]
  | loopReg h1 h2 _ ih => rw [infer, if_pos ⟨h1, h2, ih⟩]
  | tiled h1 _ ih => rw [infer, if_pos ⟨h1, ih⟩]
  | reduce h1 h2 h3 h4 => rw [infer, if_pos ⟨h1, h2, h3, h4⟩]
  | allocScratch _ ih => rw [infer]; exact ih
  | setReg h1 => rw [infer, if_pos h1]
  | scalarOp h1 h2 h3 => rw [infer, if_pos ⟨h1, h2, h3⟩]
  | vectorOp h1 h2 h3 h4 h5 => rw [infer, if_pos ⟨h1, h2, h3, h4, h5⟩]
  | emitTable h1 => rw [infer, if_pos h1]
  | tableLoad h1 h2 => rw [infer, if_pos ⟨h1, h2⟩]
  | setStatus s st => rfl
  | buildOutput h1 h2 => rw [infer, if_pos ⟨h1, h2⟩]
  | opaqueProcess h1 h2 _ ih => rw [infer, if_pos ⟨h1, h2⟩]; exact ih

/-- The judgment and the procedure are interchangeable. -/
theorem hasType_iff_infer (s : Sig) (p : Plan) (t : Sig) :
    HasType s p t ↔ infer s p = some t :=
  ⟨infer_complete, infer_sound p s t⟩

/-- The typing judgment is decidable. -/
instance instDecidableHasType (s : Sig) (p : Plan) (t : Sig) : Decidable (HasType s p t) :=
  decidable_of_iff _ (hasType_iff_infer s p t).symm

/-- Typing is deterministic: the output interface is a function of the input
interface and the plan. -/
theorem hasType_deterministic {s : Sig} {p : Plan} {t t' : Sig}
    (h : HasType s p t) (h' : HasType s p t') : t = t' := by
  have e1 := infer_complete h
  have e2 := infer_complete h'
  rw [e1] at e2
  exact Option.some.inj e2

/-! ## Properties of the resource transition -/

/-- A well-typed plan never shrinks the declared scratch. -/
theorem hasType_scratch_le {s : Sig} {p : Plan} {t : Sig} (h : HasType s p t) :
    s.scratch ≤ t.scratch := by
  induction h with
  | nop s => exact Nat.le_refl _
  | seq _ _ iha ihb => exact Nat.le_trans iha ihb
  | classifyRaw _ _ _ _ ihv _ _ _ => exact ihv
  | dispatchLayout _ _ _ ihr _ _ => exact ihr
  | branch _ _ _ ihx _ => exact ihx
  | pack => exact Nat.le_refl _
  | unpack => exact Nat.le_refl _
  | storeReg => exact Nat.le_refl _
  | loadReg => exact Nat.le_refl _
  | loopNest => exact Nat.le_refl _
  | loopReg => exact Nat.le_refl _
  | tiled => exact Nat.le_refl _
  | reduce => exact Nat.le_refl _
  | allocScratch _ ih => exact Nat.le_trans (Nat.le_add_right _ _) ih
  | setReg => exact Nat.le_refl _
  | scalarOp => exact Nat.le_refl _
  | vectorOp => exact Nat.le_refl _
  | emitTable => exact Nat.le_refl _
  | tableLoad => exact Nat.le_refl _
  | setStatus => exact Nat.le_refl _
  | buildOutput => exact Nat.le_refl _
  | opaqueProcess _ _ _ ih => exact ih

/-- A well-typed plan never changes the size of the register files, the lane
limit, the table count or the memory extent: the typing judgment fixes those
resources. -/
theorem hasType_fixed_resources {s : Sig} {p : Plan} {t : Sig} (h : HasType s p t) :
    t.regs = s.regs ∧ t.vregs = s.vregs ∧ t.lanes = s.lanes ∧ t.tables = s.tables ∧
      t.mem = s.mem ∧ t.inputType = s.inputType := by
  induction h with
  | nop s => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | seq _ _ iha ihb =>
    exact ⟨ihb.1.trans iha.1, ihb.2.1.trans iha.2.1, ihb.2.2.1.trans iha.2.2.1,
      ihb.2.2.2.1.trans iha.2.2.2.1, ihb.2.2.2.2.1.trans iha.2.2.2.2.1,
      ihb.2.2.2.2.2.trans iha.2.2.2.2.2⟩
  | classifyRaw _ _ _ _ ihv _ _ _ => exact ihv
  | dispatchLayout _ _ _ ihr _ _ => exact ihr
  | branch _ _ _ ihx _ => exact ihx
  | pack => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | unpack => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | storeReg => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | loadReg => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | loopNest => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | loopReg => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | tiled => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | reduce => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | allocScratch _ ih => exact ih
  | setReg => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | scalarOp => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | vectorOp => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | emitTable => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | tableLoad => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | setStatus => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | buildOutput => exact ⟨rfl, rfl, rfl, rfl, rfl, rfl⟩
  | opaqueProcess _ _ _ ih => exact ih

/-- The opaque-process rule is not a trust rule: a well-typed opaque node's
declared certificate is exactly the value recomputed from its retained body. -/
theorem opaqueProcess_certificate_exact {s t : Sig} {spec : OpaqueSpec} {body : Plan}
    (h : HasType s (.opaqueProcess spec body) t) :
    spec.declaredSteps = body.stepBound ∧
      spec.declaredBytes = body.charges.moduleBytes := by
  cases h with
  | opaqueProcess h1 h2 _ => exact ⟨h1, h2⟩

/-! ## Checked plans -/

/-- A plan together with its typing derivation (SPEC §11.3's `CheckedPlan`). -/
structure CheckedPlan (s t : Sig) where
  plan : Plan
  typed : HasType s plan t

namespace CheckedPlan

variable {s t : Sig}

/-- The certified cost of a checked plan (SPEC §11.3). -/
def certifiedCost (c : CheckedPlan s t) : Nat := c.plan.certifiedCost

/-- The resource bound of a checked plan (SPEC §11.3). -/
def resourceBound (c : CheckedPlan s t) : Nat := c.plan.stepBound

/-- The behaviour of a checked plan. -/
def eval (c : CheckedPlan s t) : Machine → Machine := Eval c.plan

/-- Building a checked plan by running the decision procedure. -/
def check? (s : Sig) (p : Plan) (t : Sig) : Option (CheckedPlan s t) :=
  if h : HasType s p t then some ⟨p, h⟩ else none

theorem check?_isSome_iff (s : Sig) (p : Plan) (t : Sig) :
    (check? s p t).isSome = true ↔ HasType s p t := by
  unfold check?
  by_cases h : HasType s p t <;> simp [h]

end CheckedPlan

/-! ## Type safety -/

namespace Machine

/-- A configuration conforms to an interface when its stores have the declared
sizes and its scratch is at least the declared extent. -/
def Conforms (m : Machine) (s : Sig) : Prop :=
  m.regs.length = s.regs ∧ m.vregs.length = s.vregs ∧ m.mem.length = s.mem ∧
    m.tables.length = s.tables ∧ s.scratch ≤ m.scratch.length

theorem conforms_withReg {m : Machine} {s : Sig} (h : m.Conforms s) (r v : Nat) :
    (m.withReg r v).Conforms s := by
  obtain ⟨h1, h2, h3, h4, h5⟩ := h
  exact ⟨by simpa using h1, h2, h3, h4, h5⟩

end Machine

/-- Conformance is preserved by a loop whose body preserves it. -/
theorem runLoop_conforms {f : Machine → Machine} {s : Sig}
    (hf : ∀ m, m.Conforms s → (f m).Conforms s) (r : Nat) :
    ∀ (n i : Nat) (m : Machine), m.Conforms s → (Plan.runLoop f r n i m).Conforms s := by
  intro n
  induction n with
  | zero => intro i m h; exact h
  | succ n ih =>
    intro i m h
    rw [Plan.runLoop_succ]
    exact ih (i + 1) _ (hf _ (Machine.conforms_withReg h r i))

/-- **Type safety.**  A well-typed plan maps configurations conforming to its
input interface to configurations conforming to its output interface. -/
theorem hasType_preservation {s : Sig} {p : Plan} {t : Sig} (h : HasType s p t) :
    ∀ m : Machine, m.Conforms s → (p.eval m).Conforms t := by
  induction h with
  | nop s => intro m hm; exact hm
  | seq _ _ iha ihb => intro m hm; exact ihb _ (iha m hm)
  | classifyRaw _ _ _ _ ihv ihi ihu ihe =>
    intro m hm
    rw [Plan.eval_classifyRaw]
    cases m.classify
    · exact ihv m hm
    · exact ihi m hm
    · exact ihu m hm
    · exact ihe m hm
  | dispatchLayout _ _ _ ihr ihc ihg =>
    intro m hm
    rw [Plan.eval_dispatchLayout]
    cases m.layoutClass
    · exact ihr m hm
    · exact ihc m hm
    · exact ihg m hm
  | @branch s t co x y _ _ _ ihx ihy =>
    intro m hm
    rw [Plan.eval_branch]
    by_cases hc : co.eval m = true
    · rw [if_pos hc]; exact ihx m hm
    · rw [if_neg hc]; exact ihy m hm
  | @pack s src dst map w _ _ _ =>
    intro m hm
    obtain ⟨h1, h2, h3, h4, h5⟩ := hm
    refine ⟨h1, h2, h3, h4, ?_⟩
    show s.scratch ≤ (packedScratch m src dst map).length
    rw [packedScratch_length]
    exact h5
  | @unpack s src dst map w _ _ _ =>
    intro m hm
    obtain ⟨h1, h2, h3, h4, h5⟩ := hm
    refine ⟨h1, h2, ?_, h4, h5⟩
    show (unpackedMem m src dst map).length = s.mem
    rw [unpackedMem_length]
    exact h3
  | @storeReg s dst map w src _ _ _ _ =>
    intro m hm
    obtain ⟨h1, h2, h3, h4, h5⟩ := hm
    refine ⟨h1, h2, ?_, h4, h5⟩
    show (storedRegMem m dst map w src).length = s.mem
    rw [storedRegMem_length]
    exact h3
  | @loadReg s dst src map w _ _ _ _ =>
    intro m hm
    exact Machine.conforms_withReg hm _ _
  | @loopNest s axis body _ _ ih =>
    intro m hm
    rw [Plan.eval_loopNest]
    exact runLoop_conforms ih axis.indexReg axis.extent 0 m hm
  | @loopReg s ir er map body _ _ _ ih =>
    intro m hm
    rw [eval_loopReg]
    exact runLoop_conforms ih ir (Nat.min (m.reg er) loopRegMaxTrips) 0 m hm
  | @tiled s o ti ex body _ _ ih =>
    intro m hm
    rw [Plan.eval_tiled]
    exact runLoop_conforms ih 0 (ti.totalTiles ex.m ex.n ex.k) 0 m hm
  | @reduce s c acc lhs rhs _ _ _ _ =>
    intro m hm
    rw [Plan.eval]
    obtain ⟨h1, h2, h3, h4, h5⟩ := hm
    split
    · exact ⟨h1, h2, h3, h4, h5⟩
    · exact Machine.conforms_withReg ⟨h1, h2, h3, h4, h5⟩ _ _
  | @allocScratch s t n body _ ih =>
    intro m hm
    rw [Plan.eval_allocScratch]
    refine ih _ ?_
    obtain ⟨h1, h2, h3, h4, h5⟩ := hm
    refine ⟨h1, h2, h3, h4, ?_⟩
    show s.scratch + n ≤ (m.scratch ++ List.replicate n 0).length
    simp only [List.length_append, List.length_replicate]
    omega
  | setReg _ => intro m hm; exact Machine.conforms_withReg hm _ _
  | scalarOp _ _ _ => intro m hm; exact Machine.conforms_withReg hm _ _
  | @vectorOp s op lanes dst a b _ _ _ _ _ =>
    intro m hm
    obtain ⟨h1, h2, h3, h4, h5⟩ := hm
    exact ⟨h1, by simpa using h2, h3, h4, h5⟩
  | @emitTable s idx data hidx =>
    intro m hm
    obtain ⟨h1, h2, h3, h4, h5⟩ := hm
    exact ⟨h1, h2, h3, by simpa using h4, h5⟩
  | tableLoad _ _ => intro m hm; exact Machine.conforms_withReg hm _ _
  | setStatus s st => intro m hm; exact hm
  | buildOutput _ _ => intro m hm; exact hm
  | opaqueProcess _ _ _ ih => intro m hm; exact ih m hm

/-- Type safety for checked plans. -/
theorem CheckedPlan.preservation {s t : Sig} (c : CheckedPlan s t) (m : Machine)
    (hm : m.Conforms s) : (c.eval m).Conforms t :=
  hasType_preservation c.typed m hm

end WasmGemmGnaf.GNAF
