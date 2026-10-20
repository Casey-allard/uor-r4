import WasmGemmGnaf.GNAF.Semantics
import WasmGemmGnaf.Foundation.Termination
set_option autoImplicit false

/-!
# GNAF: termination and resource bounds (SPEC §11.2)

SPEC §11.2 requires `GNAF/Resource.lean` to prove termination and bounds.  Both
are proved here against an *explicit* well-founded measure, never asserted:

* `Plan.subWellFounded` — the immediate-subplan relation `Plan.Sub`, which is
  exactly the relation `Plan.eval` recurses along, is well founded, because
  `Plan.size` strictly decreases along it (`Foundation.Termination`).
* `Plan.subChain_length_le` — the concrete budget: every chain of immediate
  subplans out of `p` is shorter than `p.size`, so evaluation cannot descend
  forever.
* `Plan.steps_le_stepBound` — the *executed* step count of a plan on any
  configuration is bounded by its static `stepBound`, which is the `steps`
  coordinate of its charge vector.
* `Plan.eval_regs_length`, `Plan.eval_mem_length`,
  `Plan.eval_scratch_length_le` — the resource transition of a plan: register
  and memory stores keep their size, and scratch grows by at most the static
  scratch bound.

The last section supplies the cost-order plumbing used by
`GNAF/Normalize.lean`: a componentwise charge inequality implies a
componentwise SPEC §9.1 cost-vector inequality, hence a canonical-objective
score inequality.
-/

namespace WasmGemmGnaf.GNAF

open WasmGemmGnaf.Foundation

namespace Plan

/-! ## Termination -/

/-- The immediate-subplan relation is well founded, with the explicit measure
`Plan.size`. -/
theorem subWellFounded : WellFounded Sub :=
  Foundation.Termination.wellFounded_of_measure size Sub (fun _ _ h => h.size_lt)

/-- The bundled measured step relation of `Foundation.Termination`. -/
def evalMeasured : Foundation.Termination.Measured Plan where
  measure := size
  step := Sub
  decreasing := fun _ _ h => h.size_lt

@[simp] theorem evalMeasured_measure (p : Plan) : evalMeasured.measure p = p.size := rfl

theorem evalMeasured_wf : WellFounded evalMeasured.step := evalMeasured.wf

/-- A chain of immediate subplans. -/
def SubChain : Plan → List Plan → Prop
  | _, [] => True
  | p, q :: rest => Sub q p ∧ SubChain q rest

theorem subChain_descending : ∀ (l : List Plan) (p : Plan),
    SubChain p l → Foundation.Termination.Descending size p l := by
  intro l
  induction l with
  | nil => intro p _; simp
  | cons q rest ih =>
    intro p h
    rw [SubChain] at h
    exact ⟨h.1.size_lt, ih q h.2⟩

/-- The concrete termination budget: an evaluation cannot descend through more
than `p.size` nested subplans. -/
theorem subChain_length_le (p : Plan) (l : List Plan) (h : SubChain p l) :
    l.length ≤ p.size :=
  Foundation.Termination.descending_length_le size l p (subChain_descending l p h)

/-! ## Executed step counts -/

/-- The number of steps a loop actually performs. -/
def loopSteps (f : Machine → Machine) (g : Machine → Nat) (r : Nat) :
    Nat → Nat → Machine → Nat
  | 0, _, _ => 0
  | n + 1, i, m => g (m.withReg r i) + loopSteps f g r n (i + 1) (f (m.withReg r i))

@[simp] theorem loopSteps_zero (f : Machine → Machine) (g : Machine → Nat)
    (r i : Nat) (m : Machine) : loopSteps f g r 0 i m = 0 := rfl

/-- A loop whose body costs at most `B` steps costs at most `n · B`. -/
theorem loopSteps_le (f : Machine → Machine) (g : Machine → Nat) (r B : Nat)
    (hg : ∀ m, g m ≤ B) : ∀ (n i : Nat) (m : Machine), loopSteps f g r n i m ≤ B * n := by
  intro n
  induction n with
  | zero => intro i m; simp
  | succ n ih =>
    intro i m
    rw [loopSteps, Nat.mul_succ]
    have h1 := hg (m.withReg r i)
    have h2 := ih (i + 1) (f (m.withReg r i))
    omega

/-- The number of primitive steps a plan actually executes on a configuration.
It mirrors `Plan.eval` exactly: only the taken branch is counted. -/
def steps : Plan → Machine → Nat
  | nop, _ => 0
  | seq a b, m => a.steps m + b.steps (a.eval m)
  | classifyRaw v i u e, m =>
      1 + (match m.classify with
        | .valid => v.steps m
        | .invalid => i.steps m
        | .unsupported => u.steps m
        | .resourceExhausted => e.steps m)
  | dispatchLayout r c g, m =>
      1 + (match m.layoutClass with
        | .rowMajor => r.steps m
        | .colMajor => c.steps m
        | .general => g.steps m)
  | branch co t f, m => 1 + (if co.eval m then t.steps m else f.steps m)
  | pack _ _ _ _, _ => 1
  | unpack _ _ _ _, _ => 1
  | storeReg _ _ _ _, _ => 1
  | loadReg _ _ _ _, _ => 1
  | loopNest axis body, m =>
      1 + loopSteps body.eval body.steps axis.indexReg axis.extent 0 m
  | loopReg indexReg extentReg _ body, m =>
      1 + loopSteps body.eval body.steps indexReg
        (Nat.min (m.reg extentReg) loopRegMaxTrips) 0 m
  | tiled _ tiling extents body, m =>
      1 + loopSteps body.eval body.steps 0 (tiling.totalTiles extents.m extents.n extents.k) 0 m
  | reduce _ _ lhs rhs, _ => 1 + Nat.min lhs.count rhs.count
  | allocScratch bytes body, m =>
      1 + body.steps { m with scratch := m.scratch ++ List.replicate bytes 0 }
  | setReg _ _, _ => 1
  | scalarOp _ _ _ _, _ => 1
  | vectorOp _ _ _ _ _, _ => 1
  | emitTable _ _, _ => 1
  | tableLoad _ _ _, _ => 1
  | setStatus _, _ => 1
  | buildOutput _, _ => 1
  | opaqueProcess _ body, m => 1 + body.steps m

@[simp] theorem steps_nop (m : Machine) : nop.steps m = 0 := rfl

@[simp] theorem steps_seq (a b : Plan) (m : Machine) :
    (seq a b).steps m = a.steps m + b.steps (a.eval m) := rfl

theorem steps_loopNest (axis : LoopAxis) (body : Plan) (m : Machine) :
    (loopNest axis body).steps m =
      1 + loopSteps body.eval body.steps axis.indexReg axis.extent 0 m := by
  simp [steps]

theorem steps_loopReg (ir er : Nat) (map : IndexMap) (body : Plan) (m : Machine) :
    (loopReg ir er map body).steps m =
      1 + loopSteps body.eval body.steps ir
        (Nat.min (m.reg er) loopRegMaxTrips) 0 m := by
  simp [steps]

theorem steps_tiled (o : TraversalOrder) (t : Tiling) (e : Extents) (body : Plan)
    (m : Machine) :
    (tiled o t e body).steps m =
      1 + loopSteps body.eval body.steps 0 (t.totalTiles e.m e.n e.k) 0 m := by
  simp [steps]

/-- **Resource bound.**  The steps a plan executes on any configuration never
exceed its static step bound. -/
theorem steps_le_stepBound (p : Plan) : ∀ m : Machine, p.steps m ≤ p.stepBound := by
  induction p with
  | nop => intro m; simp [stepBound, charges, Charges.zero]
  | seq a b iha ihb =>
    intro m
    have h1 := iha m
    have h2 := ihb (a.eval m)
    simp only [steps_seq, stepBound, charges, Charges.add] at *
    omega
  | classifyRaw v i u e ihv ihi ihu ihe =>
    intro m
    have h1 := ihv m; have h2 := ihi m; have h3 := ihu m; have h4 := ihe m
    simp only [stepBound, charges, Charges.add, Charges.node, Charges.zero] at *
    rw [steps]
    cases m.classify <;> simp <;> omega
  | dispatchLayout r c g ihr ihc ihg =>
    intro m
    have h1 := ihr m; have h2 := ihc m; have h3 := ihg m
    simp only [stepBound, charges, Charges.add, Charges.node, Charges.zero] at *
    rw [steps]
    cases m.layoutClass <;> simp <;> omega
  | branch co t f iht ihf =>
    intro m
    have h1 := iht m; have h2 := ihf m
    simp only [stepBound, charges, Charges.add, Charges.node, Charges.zero] at *
    rw [steps]
    by_cases hc : co.eval m = true <;> simp [hc] <;> omega
  | pack _ _ _ _ => intro m; simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | unpack _ _ _ _ => intro m; simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | storeReg _ _ _ _ =>
    intro m; simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | loadReg _ _ _ _ =>
    intro m; simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | loopReg ir er map body ih =>
    intro m
    have hloop := loopSteps_le body.eval body.steps ir body.stepBound ih
      (Nat.min (m.reg er) loopRegMaxTrips) 0 m
    have hmono : body.stepBound * Nat.min (m.reg er) loopRegMaxTrips ≤
        body.stepBound * loopRegMaxTrips :=
      Nat.mul_le_mul_left _ (Nat.min_le_right _ _)
    simp only [steps_loopReg, stepBound, charges, Charges.add, Charges.node,
      Charges.zero, Charges.iterate] at *
    omega
  | loopNest axis body ih =>
    intro m
    have hloop := loopSteps_le body.eval body.steps axis.indexReg body.stepBound ih
      axis.extent 0 m
    simp only [steps_loopNest, stepBound, charges, Charges.add, Charges.node,
      Charges.zero, Charges.iterate] at *
    omega
  | tiled o t e body ih =>
    intro m
    have hloop := loopSteps_le body.eval body.steps 0 body.stepBound ih
      (t.totalTiles e.m e.n e.k) 0 m
    simp only [steps_tiled, stepBound, charges, Charges.add, Charges.node,
      Charges.zero, Charges.iterate] at *
    omega
  | reduce c acc lhs rhs =>
    intro m
    simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | allocScratch bytes body ih =>
    intro m
    have h := ih { m with scratch := m.scratch ++ List.replicate bytes 0 }
    simp only [steps, stepBound, charges, Charges.add, Charges.node, Charges.zero] at *
    omega
  | setReg _ _ => intro m; simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | scalarOp _ _ _ _ => intro m; simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | vectorOp _ _ _ _ _ => intro m; simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | emitTable _ _ => intro m; simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | tableLoad _ _ _ => intro m; simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | setStatus _ => intro m; simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | buildOutput _ => intro m; simp [steps, stepBound, charges, Charges.node, Charges.zero]
  | opaqueProcess spec body ih =>
    intro m
    have h := ih m
    simp only [steps, stepBound, charges, Charges.add, Charges.node, Charges.zero] at *
    omega

/-- The step bound is exactly the `steps` coordinate of the charge vector, so
the resource bound and the cost vector are the *same* number. -/
theorem stepBound_eq_charges (p : Plan) : p.stepBound = p.charges.steps := rfl

/-! ## Store-shape invariants (the resource transition) -/

theorem runLoop_length_invariant (f : Machine → Machine) (μ : Machine → Nat)
    (hf : ∀ m, μ (f m) = μ m) (hreg : ∀ (m : Machine) (r v : Nat), μ (m.withReg r v) = μ m)
    (r : Nat) : ∀ (n i : Nat) (m : Machine), μ (runLoop f r n i m) = μ m := by
  intro n
  induction n with
  | zero => intro i m; rfl
  | succ n ih =>
    intro i m
    rw [runLoop_succ, ih, hf, hreg]

/-- The scalar register file never changes size. -/
theorem eval_regs_length (p : Plan) : ∀ m : Machine, (p.eval m).regs.length = m.regs.length := by
  induction p with
  | nop => intro m; rfl
  | seq a b iha ihb => intro m; rw [Plan.eval_seq, ihb, iha]
  | classifyRaw v i u e ihv ihi ihu ihe =>
    intro m; rw [eval_classifyRaw]; cases m.classify <;> simp [ihv, ihi, ihu, ihe]
  | dispatchLayout r c g ihr ihc ihg =>
    intro m; rw [eval_dispatchLayout]; cases m.layoutClass <;> simp [ihr, ihc, ihg]
  | branch co t f iht ihf =>
    intro m; rw [Plan.eval_branch]; by_cases hc : co.eval m = true <;> simp [hc, iht, ihf]
  | pack _ _ _ _ => intro m; rfl
  | unpack _ _ _ _ => intro m; rfl
  | storeReg _ _ _ _ => intro m; rfl
  | loadReg _ _ _ _ => intro m; simp [eval, Machine.withReg]
  | loopNest axis body ih =>
    intro m
    rw [Plan.eval_loopNest]
    exact runLoop_length_invariant body.eval (fun m => m.regs.length) ih
      (fun m r v => by simp) axis.indexReg axis.extent 0 m
  | loopReg ir er map body ih =>
    intro m
    rw [eval_loopReg]
    exact runLoop_length_invariant body.eval (fun m => m.regs.length) ih
      (fun m r v => by simp) ir (Nat.min (m.reg er) loopRegMaxTrips) 0 m
  | tiled o t e body ih =>
    intro m
    rw [Plan.eval_tiled]
    exact runLoop_length_invariant body.eval (fun m => m.regs.length) ih
      (fun m r v => by simp) 0 (t.totalTiles e.m e.n e.k) 0 m
  | reduce c acc lhs rhs =>
    intro m
    rw [eval]
    split <;> simp [Machine.withReg]
  | allocScratch bytes body ih => intro m; rw [Plan.eval_allocScratch]; simp [ih]
  | setReg _ _ => intro m; simp [eval, Machine.withReg]
  | scalarOp _ _ _ _ => intro m; simp [eval, Machine.withReg]
  | vectorOp _ _ _ _ _ => intro m; simp [eval, Machine.withVreg]
  | emitTable _ _ => intro m; rfl
  | tableLoad _ _ _ => intro m; simp [eval, Machine.withReg]
  | setStatus _ => intro m; rfl
  | buildOutput _ => intro m; rfl
  | opaqueProcess spec body ih => intro m; rw [Plan.eval_opaqueProcess]; exact ih m

/-- Observable memory never changes size: a plan cannot grow the raw
invocation. -/
theorem eval_mem_length (p : Plan) : ∀ m : Machine, (p.eval m).mem.length = m.mem.length := by
  induction p with
  | nop => intro m; rfl
  | seq a b iha ihb => intro m; rw [Plan.eval_seq, ihb, iha]
  | classifyRaw v i u e ihv ihi ihu ihe =>
    intro m; rw [eval_classifyRaw]; cases m.classify <;> simp [ihv, ihi, ihu, ihe]
  | dispatchLayout r c g ihr ihc ihg =>
    intro m; rw [eval_dispatchLayout]; cases m.layoutClass <;> simp [ihr, ihc, ihg]
  | branch co t f iht ihf =>
    intro m; rw [Plan.eval_branch]; by_cases hc : co.eval m = true <;> simp [hc, iht, ihf]
  | pack _ _ _ _ => intro m; rfl
  | unpack src dst map w => intro m; simp [eval]
  | storeReg dst map w src => intro m; simp [eval]
  | loadReg dst src map w => intro m; rfl
  | loopNest axis body ih =>
    intro m
    rw [Plan.eval_loopNest]
    exact runLoop_length_invariant body.eval (fun m => m.mem.length) ih
      (fun m r v => rfl) axis.indexReg axis.extent 0 m
  | loopReg ir er map body ih =>
    intro m
    rw [eval_loopReg]
    exact runLoop_length_invariant body.eval (fun m => m.mem.length) ih
      (fun m r v => rfl) ir (Nat.min (m.reg er) loopRegMaxTrips) 0 m
  | tiled o t e body ih =>
    intro m
    rw [Plan.eval_tiled]
    exact runLoop_length_invariant body.eval (fun m => m.mem.length) ih
      (fun m r v => rfl) 0 (t.totalTiles e.m e.n e.k) 0 m
  | reduce c acc lhs rhs =>
    intro m
    rw [eval]
    split <;> rfl
  | allocScratch bytes body ih => intro m; rw [Plan.eval_allocScratch]; simp [ih]
  | setReg _ _ => intro m; rfl
  | scalarOp _ _ _ _ => intro m; rfl
  | vectorOp _ _ _ _ _ => intro m; rfl
  | emitTable _ _ => intro m; rfl
  | tableLoad _ _ _ => intro m; rfl
  | setStatus _ => intro m; rfl
  | buildOutput _ => intro m; rfl
  | opaqueProcess spec body ih => intro m; rw [Plan.eval_opaqueProcess]; exact ih m

/-- Auxiliary: a loop growing its scratch by at most `B` per iteration grows it
by at most `n · B` in total. -/
theorem runLoop_scratch_le (f : Machine → Machine) (B : Nat)
    (hf : ∀ m, (f m).scratch.length ≤ m.scratch.length + B) (r : Nat) :
    ∀ (n i : Nat) (m : Machine),
      (runLoop f r n i m).scratch.length ≤ m.scratch.length + B * n := by
  intro n
  induction n with
  | zero => intro i m; simp
  | succ n ih =>
    intro i m
    rw [runLoop_succ, Nat.mul_succ]
    have h1 := hf (m.withReg r i)
    have h2 := ih (i + 1) (f (m.withReg r i))
    simp only [Machine.scratch_withReg] at h1
    omega

/-- **Scratch bound.**  Evaluation grows the private scratch by at most the
plan's static scratch bound. -/
theorem eval_scratch_length_le (p : Plan) :
    ∀ m : Machine, (p.eval m).scratch.length ≤ m.scratch.length + p.scratchBound := by
  induction p with
  | nop => intro m; simp [scratchBound, charges, Charges.zero]
  | seq a b iha ihb =>
    intro m
    have h1 := iha m
    have h2 := ihb (a.eval m)
    simp only [Plan.eval_seq, scratchBound, charges, Charges.add] at *
    omega
  | classifyRaw v i u e ihv ihi ihu ihe =>
    intro m
    have h1 := ihv m; have h2 := ihi m; have h3 := ihu m; have h4 := ihe m
    simp only [scratchBound, charges, Charges.add, Charges.node, Charges.zero] at *
    rw [eval_classifyRaw]
    cases m.classify <;> simp <;> omega
  | dispatchLayout r c g ihr ihc ihg =>
    intro m
    have h1 := ihr m; have h2 := ihc m; have h3 := ihg m
    simp only [scratchBound, charges, Charges.add, Charges.node, Charges.zero] at *
    rw [eval_dispatchLayout]
    cases m.layoutClass <;> simp <;> omega
  | branch co t f iht ihf =>
    intro m
    have h1 := iht m; have h2 := ihf m
    simp only [scratchBound, charges, Charges.add, Charges.node, Charges.zero] at *
    rw [Plan.eval_branch]
    by_cases hc : co.eval m = true <;> simp [hc] <;> omega
  | pack src dst map w =>
    intro m
    simp [eval, scratchBound, charges, Charges.node, Charges.zero]
  | unpack _ _ _ _ =>
    intro m; simp [eval, scratchBound, charges, Charges.node, Charges.zero]
  | storeReg _ _ _ _ =>
    intro m; simp [eval, scratchBound, charges, Charges.node, Charges.zero]
  | loadReg _ _ _ _ =>
    intro m; simp [eval, scratchBound, charges, Charges.node, Charges.zero]
  | loopReg ir er map body ih =>
    intro m
    have hloop := runLoop_scratch_le body.eval body.scratchBound ih ir
      (Nat.min (m.reg er) loopRegMaxTrips) 0 m
    have hmono : body.scratchBound * Nat.min (m.reg er) loopRegMaxTrips ≤
        body.scratchBound * loopRegMaxTrips :=
      Nat.mul_le_mul_left _ (Nat.min_le_right _ _)
    simp only [eval_loopReg, scratchBound, charges, Charges.add, Charges.node,
      Charges.zero, Charges.iterate] at *
    omega
  | loopNest axis body ih =>
    intro m
    have hloop := runLoop_scratch_le body.eval body.scratchBound ih axis.indexReg
      axis.extent 0 m
    simp only [Plan.eval_loopNest, scratchBound, charges, Charges.add, Charges.node,
      Charges.zero, Charges.iterate] at *
    omega
  | tiled o t e body ih =>
    intro m
    have hloop := runLoop_scratch_le body.eval body.scratchBound ih 0
      (t.totalTiles e.m e.n e.k) 0 m
    simp only [Plan.eval_tiled, scratchBound, charges, Charges.add, Charges.node,
      Charges.zero, Charges.iterate] at *
    omega
  | reduce c acc lhs rhs =>
    intro m
    rw [eval]
    split <;> simp [Machine.withReg, scratchBound, charges, Charges.node, Charges.zero]
  | allocScratch bytes body ih =>
    intro m
    have h := ih { m with scratch := m.scratch ++ List.replicate bytes 0 }
    simp only [Plan.eval_allocScratch, scratchBound, charges, Charges.add,
      Charges.node, Charges.zero] at *
    simp only [List.length_append, List.length_replicate] at h
    omega
  | setReg _ _ => intro m; simp [eval, scratchBound, charges, Charges.node, Charges.zero]
  | scalarOp _ _ _ _ =>
    intro m; simp [eval, scratchBound, charges, Charges.node, Charges.zero]
  | vectorOp _ _ _ _ _ =>
    intro m; simp [eval, Machine.withVreg, scratchBound, charges, Charges.node, Charges.zero]
  | emitTable _ _ => intro m; simp [eval, scratchBound, charges, Charges.node, Charges.zero]
  | tableLoad _ _ _ =>
    intro m; simp [eval, scratchBound, charges, Charges.node, Charges.zero]
  | setStatus _ => intro m; simp [eval, scratchBound, charges, Charges.node, Charges.zero]
  | buildOutput _ => intro m; simp [eval, scratchBound, charges, Charges.node, Charges.zero]
  | opaqueProcess spec body ih =>
    intro m
    have h := ih m
    simp only [Plan.eval_opaqueProcess, scratchBound, charges, Charges.add,
      Charges.node, Charges.zero] at *
    omega

/-! ## From charges to the canonical objective -/

/-- A componentwise charge inequality gives a componentwise SPEC §9.1 cost
inequality. -/
theorem costVector_mono {p q : Plan} (h : Charges.LE p.charges q.charges) :
    Cost.ArtifactVector.ComponentwiseLE p.costVector q.costVector := by
  intro co
  obtain ⟨h1, h2, h3, h4, h5, h6, h7, h8, h9⟩ := h
  cases co <;>
    simp only [Cost.ArtifactCoordinate.value, costVector, staticCost, dynamicCost] <;>
    first
      | exact h1 | exact h2 | exact h3 | exact h4 | exact h5 | exact h6 | exact h7
      | exact h8 | exact h9 | exact Nat.le_refl 0

/-- Hence a certified-cost inequality under the canonical objective. -/
theorem certifiedCost_le_of_charges_le {p q : Plan} (h : Charges.LE p.charges q.charges) :
    p.certifiedCost ≤ q.certifiedCost :=
  Cost.CanonicalObjective.monotone (costVector_mono h)

/-- The step bound is one of the charged coordinates, so it is bounded by the
certified cost. -/
theorem stepBound_le_certifiedCost (p : Plan) : p.stepBound ≤ p.certifiedCost :=
  Cost.coordinate_le_score p.costVector .sumWasmRuleSteps

/-- Therefore the executed step count of a plan on any configuration is bounded
by its certified cost. -/
theorem steps_le_certifiedCost (p : Plan) (m : Machine) : p.steps m ≤ p.certifiedCost :=
  Nat.le_trans (p.steps_le_stepBound m) (p.stepBound_le_certifiedCost)

end Plan

end WasmGemmGnaf.GNAF
