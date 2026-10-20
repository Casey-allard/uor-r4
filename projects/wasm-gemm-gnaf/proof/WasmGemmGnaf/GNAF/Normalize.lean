import WasmGemmGnaf.GNAF.Typing
set_option autoImplicit false

/-!
# GNAF: normalization (SPEC §11.3)

SPEC §11.3 requires normalization to be terminating, semantics preserving, cost
nonincreasing and idempotent.  All four are proved here:

| SPEC §11.3 | theorem |
|---|---|
| terminating | `normalize` is structurally recursive (total by construction) |
| semantics preserving | `GNAF.normalize_semantics` |
| cost nonincreasing | `GNAF.normalize_cost_le` |
| idempotent | `GNAF.normalize_idempotent` |

`normalize` is a *structural canonicalisation*: it re-associates sequential
composition to the right, erases empty plans, erases zero-trip loops and
zero-byte scratch allocations, and recurses through every control construct.
Idempotence is proved the honest way — via a decidable normal-form predicate
`IsNormal`, with `normalize_isNormal` and `normalize_eq_self_of_isNormal` —
rather than by re-running the function.

## What is deliberately *not* claimed

* SPEC §11.3's `normalize_canonical` is **omitted**.  It would assert that two
  semantically equivalent normal plans are equal, and that is false for this
  normalizer: `setReg 0 5; setReg 0 7` and `setReg 0 7` are both normal, denote
  the same configuration transformer, and are different plans.  By authority
  §3.3 the proved claim class here is therefore **normal-form**, not
  **canonical**; claiming otherwise would be a false theorem.
* `normalize` erases a zero-trip `loopNest`, whose extent is a plan literal, but
  it never erases a `loopReg`: that node's trip count is read from a register at
  run time, so no syntactic test decides whether it runs, and deleting it would
  not preserve semantics.  A `loopReg` is normal as soon as its body is.
* `normalize` does not descend into `opaqueProcess`.  Its retained certificate
  (SPEC §11.1) pins the exact body, and rewriting the body would invalidate the
  certificate the typing rule checks.  Opaque nodes are therefore normal by
  definition and left untouched.
-/

namespace WasmGemmGnaf.GNAF

open WasmGemmGnaf.Foundation

namespace Plan

/-! ## Atoms -/

/-- The empty plan. -/
def isNop : Plan → Bool
  | nop => true
  | _ => false

theorem isNop_iff (p : Plan) : isNop p = true ↔ p = nop := by
  cases p <;> simp [isNop]

/-- A plan that is neither empty nor a sequential composition: the units a
normal form is a right-nested sequence of. -/
def isAtom : Plan → Bool
  | nop => false
  | seq _ _ => false
  | _ => true

theorem isAtom_ne_nop {p : Plan} (h : isAtom p = true) : p ≠ nop := by
  cases p <;> simp_all [isAtom]

/-! ## Sequential concatenation of normal plans -/

/-- Concatenate two plans into right-nested form, dropping empty plans. -/
def app : Plan → Plan → Plan
  | nop, q => q
  | seq a b, q => seq a (app b q)
  | p, q => if isNop q then p else seq p q

@[simp] theorem app_nop_left (q : Plan) : app nop q = q := rfl

@[simp] theorem app_seq_left (a b q : Plan) : app (seq a b) q = seq a (app b q) := rfl

/-- The defining equation of `app` on an atom. -/
theorem app_atom {p : Plan} (h : isAtom p = true) (q : Plan) :
    app p q = if isNop q then p else seq p q := by
  cases p <;> simp_all [isAtom, app]

theorem app_atom_nop {p : Plan} (h : isAtom p = true) : app p nop = p := by
  rw [app_atom h]; rfl

theorem app_atom_ne_nop {p q : Plan} (h : isAtom p = true) (hq : q ≠ nop) :
    app p q = seq p q := by
  rw [app_atom h]
  have : isNop q = false := by
    cases q <;> simp_all [isNop]
  rw [this]
  rfl

/-- Concatenating onto a nonempty plan is nonempty. -/
theorem app_ne_nop {p : Plan} (hp : p ≠ nop) (q : Plan) : app p q ≠ nop := by
  cases p with
  | nop => exact absurd rfl hp
  | seq a b => simp [app]
  | _ =>
    rw [app_atom (by simp [isAtom])]
    cases hq : isNop q <;> simp

/-! ## Behaviour and charges of concatenation -/

/-- Concatenation denotes exactly sequential composition of behaviour. -/
theorem eval_app : ∀ (p q : Plan) (m : Machine), (app p q).eval m = q.eval (p.eval m) := by
  intro p
  induction p with
  | nop => intro q m; simp
  | seq a b iha ihb => intro q m; simp [ihb]
  | _ => intro q m; cases hq : isNop q <;> simp_all [app, isNop_iff]

/-- Concatenation charges exactly the sum of the charges. -/
theorem charges_app : ∀ (p q : Plan), (app p q).charges = Charges.add p.charges q.charges := by
  intro p
  induction p with
  | nop => intro q; simp [charges]
  | seq a b iha ihb => intro q; simp [charges, ihb, Charges.add_assoc]
  | _ => intro q; cases hq : isNop q <;> simp_all [app, isNop_iff, charges]

/-- Concatenation preserves typing (authority §5.7: the boundary interfaces are
threaded, not guessed). -/
theorem hasType_app : ∀ (a : Plan) {s u t : Sig} {b : Plan},
    HasType s a u → HasType u b t → HasType s (app a b) t := by
  intro a
  induction a with
  | nop =>
    intro s u t b ha hb
    cases ha
    simpa using hb
  | seq x y ihx ihy =>
    intro s u t b ha hb
    cases ha with
    | seq hx hy => exact .seq hx (ihy hy hb)
  | _ =>
    intro s u t b ha hb
    by_cases hq : b = Plan.nop
    · subst hq
      cases hb
      rwa [app_atom_nop (by simp [isAtom])]
    · rw [app_atom_ne_nop (by simp [isAtom]) hq]
      exact .seq ha hb

end Plan

/-! ## Normal forms -/

/-- The decidable normal-form predicate: right-nested nonempty sequences, no
zero-trip loop, no zero-byte scratch allocation. -/
def isNormalB : Plan → Bool
  | .nop => true
  | .seq a b => Plan.isAtom a && isNormalB a && isNormalB b && !Plan.isNop b
  | .classifyRaw v i u e => isNormalB v && isNormalB i && isNormalB u && isNormalB e
  | .dispatchLayout r c g => isNormalB r && isNormalB c && isNormalB g
  | .branch _ t f => isNormalB t && isNormalB f
  | .pack _ _ _ _ => true
  | .unpack _ _ _ _ => true
  | .storeReg _ _ _ _ => true
  | .loadReg _ _ _ _ => true
  | .loopNest ax body => !(ax.extent == 0) && isNormalB body
  | .loopReg _ _ _ body => isNormalB body
  | .tiled _ _ _ body => isNormalB body
  | .reduce _ _ _ _ => true
  | .allocScratch n body => !(n == 0) && isNormalB body
  | .setReg _ _ => true
  | .scalarOp _ _ _ _ => true
  | .vectorOp _ _ _ _ _ => true
  | .emitTable _ _ => true
  | .tableLoad _ _ _ => true
  | .setStatus _ => true
  | .buildOutput _ => true
  | .opaqueProcess _ _ => true

/-- A plan is normal when the decidable check accepts it. -/
def IsNormal (p : Plan) : Prop := isNormalB p = true

instance : DecidablePred IsNormal := fun p => by unfold IsNormal; infer_instance

/-- The claim class of this normalizer is *normal form* (authority §3.3):
soundness and idempotence are proved, completeness of the quotient is not
claimed. -/
def IsCanonical (p : Plan) : Prop := IsNormal p

/-- Concatenating normal plans yields a normal plan. -/
theorem isNormalB_app : ∀ (p q : Plan), isNormalB p = true → isNormalB q = true →
    isNormalB (Plan.app p q) = true := by
  intro p
  induction p with
  | nop => intro q _ hq; simpa using hq
  | seq a b iha ihb =>
    intro q hp hq
    simp only [isNormalB, Bool.and_eq_true] at hp
    obtain ⟨⟨⟨ha, hna⟩, hnb⟩, hbnop⟩ := hp
    rw [Plan.app_seq_left]
    simp only [isNormalB, Bool.and_eq_true]
    refine ⟨⟨⟨ha, hna⟩, ihb q hnb hq⟩, ?_⟩
    have hb : b ≠ Plan.nop := by
      intro hc
      rw [hc] at hbnop
      simp [Plan.isNop] at hbnop
    have hne := Plan.app_ne_nop hb q
    cases hc : Plan.app b q <;> simp_all [Plan.isNop]
  | _ =>
    intro q hp hq
    cases hqn : Plan.isNop q <;>
      simp_all [Plan.app, Plan.isNop_iff, isNormalB, Plan.isAtom]

/-! ## Normalization -/

/-- Structural canonicalisation (SPEC §11.3).  Opaque process nodes are left
untouched: their retained certificate pins the exact body. -/
def normalize : Plan → Plan
  | .nop => .nop
  | .seq a b => Plan.app (normalize a) (normalize b)
  | .classifyRaw v i u e =>
      .classifyRaw (normalize v) (normalize i) (normalize u) (normalize e)
  | .dispatchLayout r c g => .dispatchLayout (normalize r) (normalize c) (normalize g)
  | .branch co t f => .branch co (normalize t) (normalize f)
  | .pack a b c d => .pack a b c d
  | .unpack a b c d => .unpack a b c d
  | .storeReg d m w s => .storeReg d m w s
  | .loadReg d s m w => .loadReg d s m w
  | .loopNest ax body => if ax.extent = 0 then .nop else .loopNest ax (normalize body)
  | .loopReg ir er mp body => .loopReg ir er mp (normalize body)
  | .tiled o ti ex body => .tiled o ti ex (normalize body)
  | .reduce c acc l r => .reduce c acc l r
  | .allocScratch n body =>
      if n = 0 then normalize body else .allocScratch n (normalize body)
  | .setReg d v => .setReg d v
  | .scalarOp o d a b => .scalarOp o d a b
  | .vectorOp o l d a b => .vectorOp o l d a b
  | .emitTable i d => .emitTable i d
  | .tableLoad t i d => .tableLoad t i d
  | .setStatus st => .setStatus st
  | .buildOutput r => .buildOutput r
  | .opaqueProcess sp body => .opaqueProcess sp body

/-! ### Normalization produces normal plans -/

theorem normalize_isNormal (p : Plan) : IsNormal (normalize p) := by
  unfold IsNormal
  induction p with
  | nop => rfl
  | seq a b iha ihb => exact isNormalB_app _ _ iha ihb
  | classifyRaw v i u e ihv ihi ihu ihe =>
    simp only [normalize, isNormalB, Bool.and_eq_true]
    exact ⟨⟨⟨ihv, ihi⟩, ihu⟩, ihe⟩
  | dispatchLayout r c g ihr ihc ihg =>
    simp only [normalize, isNormalB, Bool.and_eq_true]
    exact ⟨⟨ihr, ihc⟩, ihg⟩
  | branch co t f iht ihf =>
    simp only [normalize, isNormalB, Bool.and_eq_true]
    exact ⟨iht, ihf⟩
  | pack a b c d => rfl
  | unpack a b c d => rfl
  | storeReg dst map w src => rfl
  | loadReg dst src map w => rfl
  | loopNest ax body ih =>
    simp only [normalize]
    by_cases h : ax.extent = 0
    · rw [if_pos h]; rfl
    · rw [if_neg h]
      simp only [isNormalB, Bool.and_eq_true]
      exact ⟨by simpa using h, ih⟩
  | loopReg ir er mp body ih => simpa [normalize, isNormalB] using ih
  | tiled o ti ex body ih => simpa [normalize, isNormalB] using ih
  | reduce c acc l r => rfl
  | allocScratch n body ih =>
    simp only [normalize]
    by_cases h : n = 0
    · rw [if_pos h]; exact ih
    · rw [if_neg h]
      simp only [isNormalB, Bool.and_eq_true]
      exact ⟨by simpa using h, ih⟩
  | setReg d v => rfl
  | scalarOp o d a b => rfl
  | vectorOp o l d a b => rfl
  | emitTable i d => rfl
  | tableLoad t i d => rfl
  | setStatus st => rfl
  | buildOutput r => rfl
  | opaqueProcess sp body ih => rfl

/-- Normalization is the identity on normal plans. -/
theorem normalize_eq_self_of_isNormal : ∀ (p : Plan), IsNormal p → normalize p = p := by
  intro p
  unfold IsNormal
  induction p with
  | nop => intro _; rfl
  | seq a b iha ihb =>
    intro h
    simp only [isNormalB, Bool.and_eq_true] at h
    obtain ⟨⟨⟨ha, hna⟩, hnb⟩, hbnop⟩ := h
    have hb : b ≠ Plan.nop := by
      intro hc; rw [hc] at hbnop; simp [Plan.isNop] at hbnop
    simp only [normalize, iha hna, ihb hnb]
    exact Plan.app_atom_ne_nop ha hb
  | classifyRaw v i u e ihv ihi ihu ihe =>
    intro h
    simp only [isNormalB, Bool.and_eq_true] at h
    simp only [normalize, ihv h.1.1.1, ihi h.1.1.2, ihu h.1.2, ihe h.2]
  | dispatchLayout r c g ihr ihc ihg =>
    intro h
    simp only [isNormalB, Bool.and_eq_true] at h
    simp only [normalize, ihr h.1.1, ihc h.1.2, ihg h.2]
  | branch co t f iht ihf =>
    intro h
    simp only [isNormalB, Bool.and_eq_true] at h
    simp only [normalize, iht h.1, ihf h.2]
  | pack a b c d => intro _; rfl
  | unpack a b c d => intro _; rfl
  | storeReg dst map w src => intro _; rfl
  | loadReg dst src map w => intro _; rfl
  | loopNest ax body ih =>
    intro h
    simp only [isNormalB, Bool.and_eq_true] at h
    have hx : ¬ ax.extent = 0 := by simpa using h.1
    simp only [normalize, if_neg hx, ih h.2]
  | loopReg ir er mp body ih => intro h; simp only [normalize, ih h]
  | tiled o ti ex body ih => intro h; simp only [normalize, ih h]
  | reduce c acc l r => intro _; rfl
  | allocScratch n body ih =>
    intro h
    simp only [isNormalB, Bool.and_eq_true] at h
    have hx : ¬ n = 0 := by simpa using h.1
    simp only [normalize, if_neg hx, ih h.2]
  | setReg d v => intro _; rfl
  | scalarOp o d a b => intro _; rfl
  | vectorOp o l d a b => intro _; rfl
  | emitTable i d => intro _; rfl
  | tableLoad t i d => intro _; rfl
  | setStatus st => intro _; rfl
  | buildOutput r => intro _; rfl
  | opaqueProcess sp body ih => intro _; rfl

/-- **SPEC §11.3: normalization is idempotent.** -/
theorem normalize_idempotent (p : Plan) : normalize (normalize p) = normalize p :=
  normalize_eq_self_of_isNormal (normalize p) (normalize_isNormal p)

/-- Normal plans are exactly the fixed points of `normalize`. -/
theorem isNormal_iff_normalize_eq (p : Plan) : IsNormal p ↔ normalize p = p := by
  constructor
  · exact normalize_eq_self_of_isNormal p
  · intro h; rw [← h]; exact normalize_isNormal p

/-! ### Normalization preserves behaviour -/

theorem eval_normalize : ∀ (p : Plan) (m : Machine), (normalize p).eval m = p.eval m := by
  intro p
  induction p with
  | nop => intro m; rfl
  | seq a b iha ihb =>
    intro m
    simp only [normalize, Plan.eval_app, Plan.eval_seq, iha, ihb]
  | classifyRaw v i u e ihv ihi ihu ihe =>
    intro m
    simp only [normalize, Plan.eval_classifyRaw]
    cases m.classify
    · exact ihv m
    · exact ihi m
    · exact ihu m
    · exact ihe m
  | dispatchLayout r c g ihr ihc ihg =>
    intro m
    simp only [normalize, Plan.eval_dispatchLayout]
    cases m.layoutClass
    · exact ihr m
    · exact ihc m
    · exact ihg m
  | branch co t f iht ihf =>
    intro m
    simp only [normalize, Plan.eval_branch]
    by_cases hc : co.eval m = true
    · rw [if_pos hc, if_pos hc]; exact iht m
    · rw [if_neg hc, if_neg hc]; exact ihf m
  | pack a b c d => intro m; rfl
  | unpack a b c d => intro m; rfl
  | storeReg dst map w src => intro m; rfl
  | loadReg dst src map w => intro m; rfl
  | loopReg ir er mp body ih =>
    intro m
    simp only [normalize]
    rw [eval_loopReg, eval_loopReg, funext ih]
  | loopNest ax body ih =>
    intro m
    simp only [normalize]
    by_cases h : ax.extent = 0
    · rw [if_pos h, Plan.eval_nop, Plan.eval_loopNest_extent_zero ax body m h]
    · rw [if_neg h, Plan.eval_loopNest, Plan.eval_loopNest, funext ih]
  | tiled o ti ex body ih =>
    intro m
    simp only [normalize]
    rw [Plan.eval_tiled, Plan.eval_tiled, funext ih]
  | reduce c acc l r => intro m; rfl
  | allocScratch n body ih =>
    intro m
    simp only [normalize]
    by_cases h : n = 0
    · rw [if_pos h, h, Plan.eval_allocScratch_zero]; exact ih m
    · rw [if_neg h, Plan.eval_allocScratch, Plan.eval_allocScratch]; exact ih _
  | setReg d v => intro m; rfl
  | scalarOp o d a b => intro m; rfl
  | vectorOp o l d a b => intro m; rfl
  | emitTable i d => intro m; rfl
  | tableLoad t i d => intro m; rfl
  | setStatus st => intro m; rfl
  | buildOutput r => intro m; rfl
  | opaqueProcess sp body ih => intro m; rfl

/-- **SPEC §11.3: normalization preserves semantics.** -/
theorem normalize_semantics (p : Plan) : Eval (normalize p) = Eval p :=
  funext (eval_normalize p)

/-- Normalization produces a semantically equivalent plan. -/
theorem normalize_semanticallyEquivalent (p : Plan) :
    SemanticallyEquivalent (normalize p) p := normalize_semantics p

/-! ### Normalization does not increase cost -/

theorem charges_normalize_le : ∀ p : Plan, Charges.LE (normalize p).charges p.charges := by
  intro p
  induction p with
  | nop => exact Charges.le_refl _
  | seq a b iha ihb =>
    simp only [normalize, Plan.charges_app, Plan.charges_seq]
    exact Charges.add_mono iha ihb
  | classifyRaw v i u e ihv ihi ihu ihe =>
    simp only [normalize, Plan.charges]
    exact Charges.add_mono (Charges.le_refl _)
      (Charges.add_mono (Charges.add_mono ihv ihi) (Charges.add_mono ihu ihe))
  | dispatchLayout r c g ihr ihc ihg =>
    simp only [normalize, Plan.charges]
    exact Charges.add_mono (Charges.le_refl _)
      (Charges.add_mono ihr (Charges.add_mono ihc ihg))
  | branch co t f iht ihf =>
    simp only [normalize, Plan.charges]
    exact Charges.add_mono (Charges.le_refl _) (Charges.add_mono iht ihf)
  | pack a b c d => exact Charges.le_refl _
  | unpack a b c d => exact Charges.le_refl _
  | storeReg dst map w src => exact Charges.le_refl _
  | loadReg dst src map w => exact Charges.le_refl _
  | loopReg ir er mp body ih =>
    simp only [normalize, Plan.charges]
    exact Charges.add_mono (Charges.le_refl _) (Charges.iterate_mono ih _)
  | loopNest ax body ih =>
    simp only [normalize]
    by_cases h : ax.extent = 0
    · rw [if_pos h]; exact Charges.zero_le _
    · rw [if_neg h]
      simp only [Plan.charges]
      exact Charges.add_mono (Charges.le_refl _) (Charges.iterate_mono ih _)
  | tiled o ti ex body ih =>
    simp only [normalize, Plan.charges]
    exact Charges.add_mono (Charges.le_refl _) (Charges.iterate_mono ih _)
  | reduce c acc l r => exact Charges.le_refl _
  | allocScratch n body ih =>
    simp only [normalize]
    by_cases h : n = 0
    · rw [if_pos h]
      exact Charges.le_trans ih (Charges.le_add_left _ _)
    · rw [if_neg h]
      simp only [Plan.charges]
      exact Charges.add_mono (Charges.le_refl _) ih
  | setReg d v => exact Charges.le_refl _
  | scalarOp o d a b => exact Charges.le_refl _
  | vectorOp o l d a b => exact Charges.le_refl _
  | emitTable i d => exact Charges.le_refl _
  | tableLoad t i d => exact Charges.le_refl _
  | setStatus st => exact Charges.le_refl _
  | buildOutput r => exact Charges.le_refl _
  | opaqueProcess sp body ih => exact Charges.le_refl _

/-- **SPEC §11.3: normalization does not increase the certified cost.** -/
theorem normalize_cost_le (p : Plan) : (normalize p).certifiedCost ≤ p.certifiedCost :=
  Plan.certifiedCost_le_of_charges_le (charges_normalize_le p)

/-- Normalization does not increase the resource bound either. -/
theorem normalize_stepBound_le (p : Plan) : (normalize p).stepBound ≤ p.stepBound :=
  (charges_normalize_le p).2.2.1

/-! ### Normalization preserves typing -/

/-- A well-typed plan normalizes to a plan with the same interface. -/
theorem normalize_hasType : ∀ {p : Plan} {s t : Sig}, HasType s p t →
    HasType s (normalize p) t := by
  intro p
  induction p with
  | nop => intro s t h; exact h
  | seq a b iha ihb =>
    intro s t h
    cases h with
    | seq hx hy => exact Plan.hasType_app _ (iha hx) (ihb hy)
  | classifyRaw v i u e ihv ihi ihu ihe =>
    intro s t h
    cases h with
    | classifyRaw h1 h2 h3 h4 => exact .classifyRaw (ihv h1) (ihi h2) (ihu h3) (ihe h4)
  | dispatchLayout r c g ihr ihc ihg =>
    intro s t h
    cases h with
    | dispatchLayout h1 h2 h3 => exact .dispatchLayout (ihr h1) (ihc h2) (ihg h3)
  | branch co x y ihx ihy =>
    intro s t h
    cases h with
    | branch hco h1 h2 => exact .branch hco (ihx h1) (ihy h2)
  | pack a b c d => intro s t h; exact h
  | unpack a b c d => intro s t h; exact h
  | storeReg dst map w src => intro s t h; exact h
  | loadReg dst src map w => intro s t h; exact h
  | loopReg ir er mp body ih =>
    intro s t h
    cases h with
    | loopReg h1 h2 hbody => exact .loopReg h1 h2 (ih hbody)
  | loopNest ax body ih =>
    intro s t h
    cases h with
    | loopNest hreg hbody =>
      simp only [normalize]
      by_cases hx : ax.extent = 0
      · rw [if_pos hx]; exact .nop _
      · rw [if_neg hx]; exact .loopNest hreg (ih hbody)
  | tiled o ti ex body ih =>
    intro s t h
    cases h with
    | tiled hpos hbody => exact .tiled hpos (ih hbody)
  | reduce c acc l r => intro s t h; exact h
  | allocScratch n body ih =>
    intro s t h
    cases h with
    | allocScratch hbody =>
      simp only [normalize]
      by_cases hx : n = 0
      · rw [if_pos hx]
        subst hx
        refine ih ?_
        have hs : ({ s with scratch := s.scratch + 0 } : Sig) = s := by cases s; rfl
        rwa [hs] at hbody
      · rw [if_neg hx]; exact .allocScratch (ih hbody)
  | setReg d v => intro s t h; exact h
  | scalarOp o d a b => intro s t h; exact h
  | vectorOp o l d a b => intro s t h; exact h
  | emitTable i d => intro s t h; exact h
  | tableLoad t i d => intro s t h; exact h
  | setStatus st => intro s t h; exact h
  | buildOutput r => intro s t h; exact h
  | opaqueProcess sp body ih => intro s t h; exact h

/-- Normalization of a checked plan (SPEC §11.3's `normalize` on
`CheckedPlan`). -/
def CheckedPlan.normalize {s t : Sig} (c : CheckedPlan s t) : CheckedPlan s t where
  plan := WasmGemmGnaf.GNAF.normalize c.plan
  typed := normalize_hasType c.typed

/-- SPEC §11.3 on checked plans: semantics preserved. -/
theorem CheckedPlan.normalize_semantics {s t : Sig} (c : CheckedPlan s t) :
    c.normalize.eval = c.eval :=
  WasmGemmGnaf.GNAF.normalize_semantics c.plan

/-- SPEC §11.3 on checked plans: cost nonincreasing. -/
theorem CheckedPlan.normalize_cost_le {s t : Sig} (c : CheckedPlan s t) :
    c.normalize.certifiedCost ≤ c.certifiedCost :=
  WasmGemmGnaf.GNAF.normalize_cost_le c.plan

/-- SPEC §11.3 on checked plans: idempotent. -/
theorem CheckedPlan.normalize_idempotent {s t : Sig} (c : CheckedPlan s t) :
    c.normalize.normalize.plan = c.normalize.plan :=
  WasmGemmGnaf.GNAF.normalize_idempotent c.plan

end WasmGemmGnaf.GNAF
