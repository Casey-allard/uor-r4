/-
  Wasm/Run.lean --- observable terminal stores and maximal executions.

  Normative source: SPEC.md section 7.4, which fixes `ExecutionObservation`
  with exactly five constructors, its projections, and `MaximalExecution` with
  its `finite` and `diverges` constructors.

  SPEC section 7.4 also states the presence invariant that this file proves:

      `trappedBeforeEntry` and `thrownBeforeEntry` are the only constructors for
      a terminal start/initialization branch and deliberately carry no entry
      store; `returned`, `trappedAfterEntry`, and `thrownAfterEntry` carry
      exactly one.  ...  The relational run rules prove those presence
      invariants, so no sentinel or arbitrary store can enter a functional
      evaluation.

  Both halves are proved: the constructor-level statement
  (`gemmEntryObservableStore?_eq_none_iff`,
  `gemmEntryObservableStore?_isSome_iff`) and the *run-level* statement, that
  the entry store of an observation of an actual reduction sequence is present
  exactly when the harness boundary occurs in its trace
  (`entry_isSome_iff_mem_enterGemm`), together with the invariant that a
  normally returned run has necessarily crossed it
  (`returned_has_entry_store`).

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Step

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## Semantic observations -/

/-- The deterministic semantic observation of a run: its terminal status, the
final ABI-visible store and the externally visible non-memory effects. -/
structure SemanticObservation where
  /-- The terminal status. -/
  terminal : TerminalStatus
  /-- The final ABI-visible store. -/
  finalObservableStore : ObservableStore
  /-- The externally visible non-memory effects. -/
  effects : ObservableEffects
  deriving DecidableEq, Repr, Inhabited

/-- The semantic projection of a normal return. -/
def projectReturned (value : UInt32) (store : ObservableStore)
    (effects : ObservableEffects) : SemanticObservation :=
  { terminal := .returned value, finalObservableStore := store, effects }

/-- The semantic projection of a trap. -/
def projectTrap (trap : Trap) (store : ObservableStore)
    (effects : ObservableEffects) : SemanticObservation :=
  { terminal := .trapped trap, finalObservableStore := store, effects }

/-- The semantic projection of an uncaught exception. -/
def projectThrown (exceptionValue : ExceptionValue) (store : ObservableStore)
    (effects : ObservableEffects) : SemanticObservation :=
  { terminal := .thrown exceptionValue, finalObservableStore := store, effects }

/-- **SPEC section 7.4.**  An uncaught Core exception remains observably
distinct from a trap. -/
theorem projectTrap_ne_projectThrown (trap : Trap) (exceptionValue : ExceptionValue)
    (store : ObservableStore) (effects : ObservableEffects) :
    projectTrap trap store effects ≠ projectThrown exceptionValue store effects := by
  simp [projectTrap, projectThrown]

/-- A normal return is observably distinct from a trap. -/
theorem projectReturned_ne_projectTrap (value : UInt32) (trap : Trap)
    (store : ObservableStore) (effects : ObservableEffects) :
    projectReturned value store effects ≠ projectTrap trap store effects := by
  simp [projectReturned, projectTrap]

/-! ## Execution observations -/

/-- **SPEC section 7.4.**  The five terminal observations of a run. -/
inductive ExecutionObservation
  /-- The exported `gemm` returned normally. -/
  | returned
      (trace : List Event)
      (gemmEntryObservableStore : ObservableStore)
      (value : UInt32)
      (finalObservableStore : ObservableStore)
      (effects : ObservableEffects)
  /-- The run trapped during initialization, the start function, or the raw
  installation, before the entry boundary. -/
  | trappedBeforeEntry
      (trace : List Event)
      (trap : Trap)
      (finalObservableStore : ObservableStore)
      (effects : ObservableEffects)
  /-- The run trapped inside the exported `gemm`. -/
  | trappedAfterEntry
      (trace : List Event)
      (gemmEntryObservableStore : ObservableStore)
      (trap : Trap)
      (finalObservableStore : ObservableStore)
      (effects : ObservableEffects)
  /-- The run terminated with an uncaught exception before the entry
  boundary. -/
  | thrownBeforeEntry
      (trace : List Event)
      (exceptionValue : ExceptionValue)
      (finalObservableStore : ObservableStore)
      (effects : ObservableEffects)
  /-- The run terminated with an uncaught exception inside `gemm`. -/
  | thrownAfterEntry
      (trace : List Event)
      (gemmEntryObservableStore : ObservableStore)
      (exceptionValue : ExceptionValue)
      (finalObservableStore : ObservableStore)
      (effects : ObservableEffects)
  deriving DecidableEq, Repr, Inhabited

namespace ExecutionObservation

/-- The trace of a run. -/
def trace : ExecutionObservation → List Event
  | .returned trace _ _ _ _ => trace
  | .trappedBeforeEntry trace _ _ _ => trace
  | .trappedAfterEntry trace _ _ _ _ => trace
  | .thrownBeforeEntry trace _ _ _ => trace
  | .thrownAfterEntry trace _ _ _ _ => trace

/-- The final ABI-visible store of a run. -/
def finalObservableStore : ExecutionObservation → ObservableStore
  | .returned _ _ _ store _ => store
  | .trappedBeforeEntry _ _ store _ => store
  | .trappedAfterEntry _ _ _ store _ => store
  | .thrownBeforeEntry _ _ store _ => store
  | .thrownAfterEntry _ _ _ store _ => store

/-- The externally visible non-memory effects of a run. -/
def effects : ExecutionObservation → ObservableEffects
  | .returned _ _ _ _ effects => effects
  | .trappedBeforeEntry _ _ _ effects => effects
  | .trappedAfterEntry _ _ _ _ effects => effects
  | .thrownBeforeEntry _ _ _ effects => effects
  | .thrownAfterEntry _ _ _ _ effects => effects

/-- The store snapshot taken at entry to the exported `gemm`, when the boundary
was crossed. -/
def gemmEntryObservableStore? : ExecutionObservation → Option ObservableStore
  | .returned _ entry _ _ _ => some entry
  | .trappedBeforeEntry _ _ _ _ => none
  | .trappedAfterEntry _ entry _ _ _ => some entry
  | .thrownBeforeEntry _ _ _ _ => none
  | .thrownAfterEntry _ entry _ _ _ => some entry

/-- The unmasked semantic observation of a run. -/
def unmaskedSemantic : ExecutionObservation → SemanticObservation
  | .returned _ _ value store effects => projectReturned value store effects
  | .trappedBeforeEntry _ trap store effects => projectTrap trap store effects
  | .trappedAfterEntry _ _ trap store effects => projectTrap trap store effects
  | .thrownBeforeEntry _ exceptionValue store effects =>
      projectThrown exceptionValue store effects
  | .thrownAfterEntry _ _ exceptionValue store effects =>
      projectThrown exceptionValue store effects

/-- The observation belongs to a terminal start/initialization branch. -/
def BeforeEntry (o : ExecutionObservation) : Prop :=
  (∃ tr t s e, o = .trappedBeforeEntry tr t s e) ∨
  (∃ tr v s e, o = .thrownBeforeEntry tr v s e)

/-- **SPEC section 7.4, presence invariant (absence half).**  The entry store
is absent exactly for the two `*BeforeEntry` constructors. -/
theorem gemmEntryObservableStore?_eq_none_iff (o : ExecutionObservation) :
    o.gemmEntryObservableStore? = none ↔ o.BeforeEntry := by
  cases o <;> simp [gemmEntryObservableStore?, BeforeEntry]

/-- **SPEC section 7.4, presence invariant (presence half).**  The entry store
is present exactly for `returned`, `trappedAfterEntry` and
`thrownAfterEntry`. -/
theorem gemmEntryObservableStore?_isSome_iff (o : ExecutionObservation) :
    o.gemmEntryObservableStore?.isSome = true ↔ ¬ o.BeforeEntry := by
  cases o <;> simp [gemmEntryObservableStore?, BeforeEntry]

theorem gemmEntryObservableStore?_returned (tr : List Event)
    (entry : ObservableStore) (v : UInt32) (store : ObservableStore)
    (eff : ObservableEffects) :
    (ExecutionObservation.returned tr entry v store eff).gemmEntryObservableStore? =
      some entry := rfl

theorem gemmEntryObservableStore?_trappedAfterEntry (tr : List Event)
    (entry : ObservableStore) (t : Trap) (store : ObservableStore)
    (eff : ObservableEffects) :
    (ExecutionObservation.trappedAfterEntry tr entry t store
      eff).gemmEntryObservableStore? = some entry := rfl

theorem gemmEntryObservableStore?_thrownAfterEntry (tr : List Event)
    (entry : ObservableStore) (v : ExceptionValue) (store : ObservableStore)
    (eff : ObservableEffects) :
    (ExecutionObservation.thrownAfterEntry tr entry v store
      eff).gemmEntryObservableStore? = some entry := rfl

theorem gemmEntryObservableStore?_trappedBeforeEntry (tr : List Event) (t : Trap)
    (store : ObservableStore) (eff : ObservableEffects) :
    (ExecutionObservation.trappedBeforeEntry tr t store
      eff).gemmEntryObservableStore? = none := rfl

theorem gemmEntryObservableStore?_thrownBeforeEntry (tr : List Event)
    (v : ExceptionValue) (store : ObservableStore) (eff : ObservableEffects) :
    (ExecutionObservation.thrownBeforeEntry tr v store
      eff).gemmEntryObservableStore? = none := rfl

end ExecutionObservation

/-! ## Reduction sequences -/

/-- The reflexive transitive closure of `Step`, carrying its trace. -/
inductive Reduces : Config → List Event → Config → Prop
  | refl (c : Config) : Reduces c [] c
  | cons {c e c' tr c''} (h : Step c e c') (hr : Reduces c' tr c'') :
      Reduces c (e :: tr) c''

namespace Reduces

theorem single {c c' : Config} {e : Event} (h : Step c e c') : Reduces c [e] c' :=
  .cons h (.refl c')

theorem trans {a b c : Config} {tr₁ tr₂ : List Event} (h₁ : Reduces a tr₁ b)
    (h₂ : Reduces b tr₂ c) : Reduces a (tr₁ ++ tr₂) c := by
  induction h₁ with
  | refl _ => simpa using h₂
  | cons h _ ih => exact .cons h (ih h₂)

theorem length_eq_zero {a b : Config} {tr : List Event} (h : Reduces a tr b)
    (htr : tr = []) : a = b := by
  cases h with
  | refl _ => rfl
  | cons _ _ => exact absurd htr (by simp)

end Reduces

/-! ## Building an observation from a terminal configuration -/

/-- The observation recorded by a terminal configuration reached along `trace`.
`none` is returned for a running configuration and for the impossible shape of a
normal return without a crossed entry boundary; `returned_has_entry_store`
below shows the latter cannot arise from an actual run. -/
def observationOfConfig (trace : List Event) (c : Config) :
    Option ExecutionObservation :=
  match c.status, c.entry? with
  | .running, _ => none
  | .returned v, some entry =>
      some (.returned trace entry v c.observable ObservableEffects.none')
  | .returned _, none => none
  | .trapped t, none =>
      some (.trappedBeforeEntry trace t c.observable ObservableEffects.none')
  | .trapped t, some entry =>
      some (.trappedAfterEntry trace entry t c.observable ObservableEffects.none')
  | .thrown ev, none =>
      some (.thrownBeforeEntry trace ev c.observable ObservableEffects.none')
  | .thrown ev, some entry =>
      some (.thrownAfterEntry trace entry ev c.observable ObservableEffects.none')

/-- An observation is only produced by a terminal configuration. -/
theorem isTerminal_of_observationOfConfig {trace : List Event} {c : Config}
    {o : ExecutionObservation} (h : observationOfConfig trace c = some o) :
    c.IsTerminal := by
  unfold observationOfConfig at h
  unfold Config.IsTerminal
  cases hs : c.status with
  | running => rw [hs] at h; cases c.entry? <;> simp at h
  | returned v => simp
  | trapped t => simp
  | thrown ev => simp

/-- The trace recorded in an observation is the one it was built with. -/
theorem trace_observationOfConfig {trace : List Event} {c : Config}
    {o : ExecutionObservation} (h : observationOfConfig trace c = some o) :
    o.trace = trace := by
  unfold observationOfConfig at h
  cases hs : c.status <;> cases he : c.entry? <;> rw [hs, he] at h <;>
    simp at h <;> (try subst h) <;> rfl

/-- The entry store of the observation is exactly the configuration's
snapshot. -/
theorem gemmEntry_observationOfConfig {trace : List Event} {c : Config}
    {o : ExecutionObservation} (h : observationOfConfig trace c = some o) :
    o.gemmEntryObservableStore? = c.entry? := by
  unfold observationOfConfig at h
  cases hs : c.status <;> cases he : c.entry? <;> rw [hs, he] at h <;>
    simp at h <;> (try subst h) <;>
    simp [ExecutionObservation.gemmEntryObservableStore?]

/-! ## Finite and maximal executions -/

/-- A finite execution of `initial` producing `observation`. -/
def FiniteExecution (initial : Config) (observation : ExecutionObservation) : Prop :=
  ∃ final : Config,
    Reduces initial observation.trace final ∧
      observationOfConfig observation.trace final = some observation

/-- An observation is maximal: every configuration that produces it is
terminal, so the run cannot be extended. -/
def IsTerminalObservation (observation : ExecutionObservation) : Prop :=
  ∀ final : Config,
    observationOfConfig observation.trace final = some observation →
      final.IsTerminal

/-- Every observation is maximal: `observationOfConfig` never produces one from
a running configuration. -/
theorem isTerminalObservation (observation : ExecutionObservation) :
    IsTerminalObservation observation :=
  fun _ h => isTerminal_of_observationOfConfig h

/-- **SPEC section 7.4.**  A maximal execution of `initial`: either a finite
run ending in a maximal observation, or a divergent reduction sequence. -/
inductive MaximalExecution (initial : Config)
  /-- A finite maximal run. -/
  | finite
      (observation : ExecutionObservation)
      (run : FiniteExecution initial observation)
      (maximal : IsTerminalObservation observation)
  /-- A divergent run. -/
  | diverges (events : Nat → Event) (configs : Nat → Config)
      (starts : configs 0 = initial)
      (step : ∀ i, Step (configs i) (events i) (configs (i + 1)))

/-- Every finite execution yields a maximal execution. -/
def MaximalExecution.ofFinite {initial : Config} {o : ExecutionObservation}
    (run : FiniteExecution initial o) : MaximalExecution initial :=
  .finite o run (isTerminalObservation o)

/-! ## The run-level presence invariant

SPEC section 7.4: "The trace phase tags prove that this boundary occurs exactly
once on a normally started invocation." -/

/-- Only the harness entry rule changes the entry snapshot. -/
theorem Step.entry_eq_of_ne_enterGemm {c : Config} {e : Event} {c' : Config}
    (h : Step c e c') (he : e ≠ Event.enterGemm) : c'.entry? = c.entry? := by
  cases h <;> first | rfl | exact absurd rfl he

/-- The entry snapshot, once taken, is never discarded. -/
theorem Step.entry_isSome_mono {c : Config} {e : Event} {c' : Config}
    (h : Step c e c') (hc : c.entry?.isSome = true) : c'.entry?.isSome = true := by
  cases h <;> simp_all

/-- The harness entry rule takes the snapshot. -/
theorem Step.entry_isSome_of_enterGemm {c c' : Config}
    (h : Step c Event.enterGemm c') : c'.entry?.isSome = true := by
  cases h <;> simp_all

theorem Reduces.entry_isSome_mono {a b : Config} {tr : List Event}
    (h : Reduces a tr b) (ha : a.entry?.isSome = true) :
    b.entry?.isSome = true := by
  induction h with
  | refl _ => exact ha
  | cons hs _ ih => exact ih (hs.entry_isSome_mono ha)

/-- **SPEC section 7.4, run-level presence invariant.**  Starting from a
configuration that has not crossed the harness boundary, the entry snapshot is
present at the end of a reduction sequence exactly when that sequence contains
the harness phase-transition event. -/
theorem Reduces.entry_isSome_iff_mem_enterGemm {a b : Config} {tr : List Event}
    (h : Reduces a tr b) (ha : a.entry? = none) :
    b.entry?.isSome = true ↔ Event.enterGemm ∈ tr := by
  induction h with
  | refl c => simp [ha]
  | @cons c e c' tr' c'' hs hr ih =>
    by_cases he : e = Event.enterGemm
    · subst he
      constructor
      · intro _; exact List.mem_cons_self
      · intro _
        exact hr.entry_isSome_mono hs.entry_isSome_of_enterGemm
    · rw [ih (by rw [hs.entry_eq_of_ne_enterGemm he, ha])]
      simp [Ne.symm he]

/-- **SPEC section 7.4.**  The observation of a run that starts before the
harness boundary carries an entry store exactly when the boundary was crossed
in its trace. -/
theorem entry_isSome_iff_mem_enterGemm {initial : Config}
    {o : ExecutionObservation} (h : FiniteExecution initial o)
    (h0 : initial.entry? = none) :
    o.gemmEntryObservableStore?.isSome = true ↔ Event.enterGemm ∈ o.trace := by
  obtain ⟨final, hred, hobs⟩ := h
  rw [gemmEntry_observationOfConfig hobs]
  exact hred.entry_isSome_iff_mem_enterGemm h0

/-- Consequently the observation is a `*BeforeEntry` one exactly when the run
never crossed the harness boundary. -/
theorem beforeEntry_iff_not_mem_enterGemm {initial : Config}
    {o : ExecutionObservation} (h : FiniteExecution initial o)
    (h0 : initial.entry? = none) :
    o.BeforeEntry ↔ Event.enterGemm ∉ o.trace := by
  rw [← ExecutionObservation.gemmEntryObservableStore?_eq_none_iff]
  have hiff := entry_isSome_iff_mem_enterGemm h h0
  cases hg : o.gemmEntryObservableStore? with
  | none => rw [hg] at hiff; simp at hiff ⊢; exact hiff
  | some s => rw [hg] at hiff; simp at hiff ⊢; exact hiff

/-! ### A normal return has necessarily crossed the boundary -/

/-- The run invariant of the harness protocol: a post-entry configuration
carries its snapshot, and only a post-entry configuration can have returned. -/
def RunInvariant (c : Config) : Prop :=
  (c.phase = Phase.afterEntry → c.entry?.isSome = true) ∧
  (∀ v : UInt32, c.status = Status.returned v → c.phase = Phase.afterEntry)

theorem Step.runInvariant {c : Config} {e : Event} {c' : Config} (h : Step c e c')
    (hinv : RunInvariant c) : RunInvariant c' := by
  obtain ⟨h1, h2⟩ := hinv
  cases h <;> exact ⟨by simp_all, by simp_all⟩

theorem Reduces.runInvariant {a b : Config} {tr : List Event} (h : Reduces a tr b)
    (ha : RunInvariant a) : RunInvariant b := by
  induction h with
  | refl _ => exact ha
  | cons hs _ ih => exact ih (hs.runInvariant ha)

/-- The pre-start harness configuration satisfies the run invariant. -/
theorem runInvariant_of_beforeEntry {c : Config}
    (hp : c.phase = Phase.beforeEntry) (hs : c.status = Status.running) :
    RunInvariant c := by
  refine ⟨fun h => absurd (hp ▸ h) (by simp), fun v h => ?_⟩
  rw [hs] at h
  exact absurd h (by simp)

/-- **SPEC section 7.4.**  A run that returns normally has crossed the harness
boundary, so its observation carries exactly one entry store: no sentinel or
arbitrary store can enter a functional evaluation. -/
theorem returned_has_entry_store {initial final : Config} {tr : List Event}
    {v : UInt32} (h : Reduces initial tr final) (ha : RunInvariant initial)
    (hfin : final.status = Status.returned v) : final.entry?.isSome = true :=
  let hb := h.runInvariant ha
  hb.1 (hb.2 v hfin)

/-- Hence a normally returning run always produces a `returned` observation
carrying its entry store. -/
theorem observation_returned_of_status {tr : List Event} {initial final : Config}
    {v : UInt32} (h : Reduces initial tr final) (ha : RunInvariant initial)
    (hfin : final.status = Status.returned v) :
    ∃ entry : ObservableStore,
      observationOfConfig tr final =
        some (.returned tr entry v final.observable ObservableEffects.none') := by
  have hs := returned_has_entry_store h ha hfin
  cases he : final.entry? with
  | none => rw [he] at hs; simp at hs
  | some entry =>
    refine ⟨entry, ?_⟩
    unfold observationOfConfig
    rw [hfin, he]

end WasmGemmGnaf.Wasm
