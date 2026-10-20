/-
  Wasm/Fuel.lean --- the bounded all-branch execution-tree explorer.

  Normative source: SPEC.md section 7.4:

      The executable object is a bounded execution-tree explorer, not a one-path
      interpreter ... Soundness SHALL map every returned observation to a
      relational execution.  Completeness SHALL prove that every relational
      branch of length at most the bound occurs in the tree, and that `complete`
      contains every maximal branch.  `nonterminalPrefix` proves the module
      cannot satisfy a smaller all-branch step sublevel.  Completeness is
      relative to an explicit bound ... No general halting oracle is permitted.

  `exploreTree` recurses over *every* element of `successors`, so a
  configuration with two permitted `memory.grow` outcomes contributes both
  subtrees.  The two required theorems are proved:

      Wasm.runFuel_sound
      Wasm.runFuel_complete_with_bound

  Deviation from the SPEC signature, disclosed: SPEC's `complete` constructor
  carries a `NonemptyCanonicalList`, and its `nonterminalPrefix` constructor
  carries only a trace.  Here `complete` carries a plain list (the released
  machine admits stuck configurations, for which the observation list of a
  subtree is genuinely empty) and `nonterminalPrefix` additionally carries the
  observations found so far together with their coverage proof.  Without the
  latter, `runFuel_complete_with_bound` would be *false*: one branch may
  terminate inside the bound while another is still running past it.

  Every declaration in this file is proved.  Nothing is assumed.
-/
import WasmGemmGnaf.Wasm.Run

set_option autoImplicit false

namespace WasmGemmGnaf.Wasm

open WasmGemmGnaf.Foundation

/-! ## The explorer -/

/-- Every observation reachable from `c` within `fuel` reduction steps, with
`tr` the trace already accumulated.  The recursion visits *every* permitted
successor, so all nondeterministic branches are explored. -/
def exploreTree : Nat → List Event → Config → List ExecutionObservation
  | 0, tr, c => (observationOfConfig tr c).toList
  | n + 1, tr, c =>
      if c.status = Status.running then
        (successors c).flatMap (fun p => exploreTree n (tr ++ [p.1]) p.2)
      else (observationOfConfig tr c).toList

/-- Every trace of exactly `fuel` further steps from `c` that is still running
at the end. -/
def prefixes : Nat → List Event → Config → List (List Event)
  | 0, tr, c => if c.status = Status.running then [tr] else []
  | n + 1, tr, c =>
      if c.status = Status.running then
        (successors c).flatMap (fun p => prefixes n (tr ++ [p.1]) p.2)
      else []

@[simp] theorem exploreTree_zero (tr : List Event) (c : Config) :
    exploreTree 0 tr c = (observationOfConfig tr c).toList := rfl

theorem exploreTree_succ_of_running {c : Config} (n : Nat) (tr : List Event)
    (h : c.status = Status.running) :
    exploreTree (n + 1) tr c =
      (successors c).flatMap (fun p => exploreTree n (tr ++ [p.1]) p.2) := by
  simp [exploreTree, h]

theorem exploreTree_succ_of_terminal {c : Config} (n : Nat) (tr : List Event)
    (h : ¬ c.status = Status.running) :
    exploreTree (n + 1) tr c = (observationOfConfig tr c).toList := by
  simp [exploreTree, h]

theorem mem_option_toList {α : Type} {x : Option α} {a : α} :
    a ∈ x.toList ↔ x = some a := by
  cases x <;> simp [Option.toList, eq_comm]

/-! ## Soundness -/

/-- Every observation the explorer returns is produced by an actual reduction
sequence. -/
theorem exploreTree_sound :
    ∀ (fuel : Nat) (tr : List Event) (c : Config) (o : ExecutionObservation),
      o ∈ exploreTree fuel tr c →
      ∃ (suffix : List Event) (final : Config),
        o.trace = tr ++ suffix ∧ Reduces c suffix final ∧
          observationOfConfig o.trace final = some o := by
  intro fuel
  induction fuel with
  | zero =>
    intro tr c o hm
    rw [exploreTree_zero, mem_option_toList] at hm
    refine ⟨[], c, ?_, .refl c, ?_⟩
    · rw [trace_observationOfConfig hm]; simp
    · rw [trace_observationOfConfig hm]; exact hm
  | succ n ih =>
    intro tr c o hm
    by_cases hs : c.status = Status.running
    · rw [exploreTree_succ_of_running n tr hs, List.mem_flatMap] at hm
      obtain ⟨p, hp, hmem⟩ := hm
      obtain ⟨suffix, final, htr, hred, hobs⟩ := ih (tr ++ [p.1]) p.2 o hmem
      refine ⟨p.1 :: suffix, final, ?_, ?_, hobs⟩
      · rw [htr]; simp
      · exact .cons ((mem_successors_iff_step c p.1 p.2).mp (by simpa using hp)) hred
    · rw [exploreTree_succ_of_terminal n tr hs, mem_option_toList] at hm
      refine ⟨[], c, ?_, .refl c, ?_⟩
      · rw [trace_observationOfConfig hm]; simp
      · rw [trace_observationOfConfig hm]; exact hm

/-! ## Completeness -/

/-- Every relational branch of length at most the fuel occurs in the tree. -/
theorem exploreTree_complete :
    ∀ (fuel : Nat) (tr suffix : List Event) (c final : Config)
      (o : ExecutionObservation),
      Reduces c suffix final →
      observationOfConfig (tr ++ suffix) final = some o →
      suffix.length ≤ fuel →
      o ∈ exploreTree fuel tr c := by
  intro fuel
  induction fuel with
  | zero =>
    intro tr suffix c final o hred hobs hlen
    have hsuf : suffix = [] := List.length_eq_zero_iff.mp (Nat.le_zero.mp hlen)
    subst hsuf
    have : c = final := hred.length_eq_zero rfl
    subst this
    rw [exploreTree_zero, mem_option_toList]
    simpa using hobs
  | succ n ih =>
    intro tr suffix c final o hred hobs hlen
    cases hred with
    | refl _ =>
      have hterm : ¬ c.status = Status.running := by
        have := isTerminal_of_observationOfConfig (trace := tr ++ []) hobs
        exact this
      rw [exploreTree_succ_of_terminal n tr hterm, mem_option_toList]
      simpa using hobs
    | @cons _ e c'' tr'' _ hstep hrest =>
      have hs : c.status = Status.running := hstep.running
      rw [exploreTree_succ_of_running n tr hs, List.mem_flatMap]
      refine ⟨(e, c''), (mem_successors_iff_step c e c'').mpr hstep, ?_⟩
      refine ih (tr ++ [e]) tr'' c'' final o hrest ?_ ?_
      · simpa using hobs
      · simp only [List.length_cons] at hlen
        omega

/-! ## The prefix witness -/

/-- Every prefix reported has exactly the explored length. -/
theorem prefixes_length :
    ∀ (fuel : Nat) (tr : List Event) (c : Config) (t : List Event),
      t ∈ prefixes fuel tr c → t.length = tr.length + fuel := by
  intro fuel
  induction fuel with
  | zero =>
    intro tr c t hm
    unfold prefixes at hm
    split at hm
    · simp at hm; subst hm; simp
    · simp at hm
  | succ n ih =>
    intro tr c t hm
    unfold prefixes at hm
    split at hm
    · rw [List.mem_flatMap] at hm
      obtain ⟨p, _, hmem⟩ := hm
      have := ih (tr ++ [p.1]) p.2 t hmem
      simp at this
      omega
    · simp at hm

/-- Every reported prefix is a real reduction sequence that is still running at
its end: the module cannot terminate on that branch within the bound. -/
theorem prefixes_sound :
    ∀ (fuel : Nat) (tr : List Event) (c : Config) (t : List Event),
      t ∈ prefixes fuel tr c →
      ∃ (suffix : List Event) (final : Config),
        t = tr ++ suffix ∧ Reduces c suffix final ∧
          final.status = Status.running := by
  intro fuel
  induction fuel with
  | zero =>
    intro tr c t hm
    unfold prefixes at hm
    split at hm
    · rename_i hs
      simp at hm
      subst hm
      exact ⟨[], c, by simp, .refl c, hs⟩
    · simp at hm
  | succ n ih =>
    intro tr c t hm
    unfold prefixes at hm
    split at hm
    · rw [List.mem_flatMap] at hm
      obtain ⟨p, hp, hmem⟩ := hm
      obtain ⟨suffix, final, ht, hred, hrun⟩ := ih (tr ++ [p.1]) p.2 t hmem
      refine ⟨p.1 :: suffix, final, ?_, ?_, hrun⟩
      · rw [ht]; simp
      · exact .cons ((mem_successors_iff_step c p.1 p.2).mp (by simpa using hp)) hred
    · simp at hm

theorem mem_of_head? {α : Type} {l : List α} {x : α} (h : l.head? = some x) :
    x ∈ l := by
  cases l with
  | nil => simp at h
  | cons a t => simp at h; subst h; exact List.mem_cons_self

/-! ## The execution-tree result -/

/-- A trace of exactly `n` events. -/
structure TraceOfLength (n : Nat) where
  /-- The events. -/
  events : List Event
  /-- Their number. -/
  length_eq : events.length = n

/-- The observations cover every maximal finite branch of length at most the
bound: each listed observation is a real execution, and every real execution
short enough to fit the bound is listed. -/
def CoversEveryMaximalFiniteBranch (bound : Nat) (initial : Config)
    (observations : List ExecutionObservation) : Prop :=
  (∀ o ∈ observations, FiniteExecution initial o) ∧
  (∀ o : ExecutionObservation, FiniteExecution initial o →
    o.trace.length ≤ bound → o ∈ observations)

/-- **SPEC section 7.4.**  The result of a bounded all-branch exploration. -/
inductive ExecutionTreeResult (bound : Nat) (initial : Config)
  /-- Every branch terminated inside the bound. -/
  | complete (observations : List ExecutionObservation)
      (allBranchesMaximal : CoversEveryMaximalFiniteBranch bound initial observations)
  /-- Some branch was still running after `bound + 1` steps; the observations
  found so far are still covered. -/
  | nonterminalPrefix (observations : List ExecutionObservation)
      (allBranchesMaximal : CoversEveryMaximalFiniteBranch bound initial observations)
      (trace : TraceOfLength (bound + 1))
  /-- Initialization failed before any reduction. -/
  | initializationFailure (fault : InstantiationFault)

/-- The observations recorded by an exploration result. -/
def ExecutionTreeResult.observations {bound : Nat} {initial : Config} :
    ExecutionTreeResult bound initial → List ExecutionObservation
  | .complete obs _ => obs
  | .nonterminalPrefix obs _ _ => obs
  | .initializationFailure _ => []

/-- Membership of an observation in an exploration result. -/
def TreeContains {bound : Nat} {initial : Config}
    (r : ExecutionTreeResult bound initial) (o : ExecutionObservation) : Prop :=
  o ∈ r.observations

theorem covers_exploreTree (bound : Nat) (initial : Config) :
    CoversEveryMaximalFiniteBranch bound initial
      (exploreTree (bound + 1) [] initial) := by
  constructor
  · intro o hm
    obtain ⟨suffix, final, htr, hred, hobs⟩ :=
      exploreTree_sound (bound + 1) [] initial o hm
    refine ⟨final, ?_, hobs⟩
    rw [htr]
    simpa using hred
  · intro o hrun hlen
    obtain ⟨final, hred, hobs⟩ := hrun
    exact exploreTree_complete (bound + 1) [] o.trace initial final o hred
      (by simpa using hobs) (by omega)

/-- **SPEC section 7.4.**  The bounded all-branch execution-tree explorer. -/
def exploreAll (bound : Nat) (initial : Config) : ExecutionTreeResult bound initial :=
  match hp : (prefixes (bound + 1) [] initial).head? with
  | some t =>
      .nonterminalPrefix (exploreTree (bound + 1) [] initial)
        (covers_exploreTree bound initial)
        ⟨t, by
          have := prefixes_length (bound + 1) [] initial t (mem_of_head? hp)
          simpa using this⟩
  | none =>
      .complete (exploreTree (bound + 1) [] initial) (covers_exploreTree bound initial)

theorem observations_exploreAll (bound : Nat) (initial : Config) :
    (exploreAll bound initial).observations = exploreTree (bound + 1) [] initial := by
  unfold exploreAll
  split <;> rfl

/-- **SPEC section 7.4.**  Soundness: every observation the explorer returns is
an actual finite execution of the initial configuration. -/
theorem runFuel_sound {bound : Nat} {initial : Config}
    {observation : ExecutionObservation}
    (hmember : TreeContains (exploreAll bound initial) observation) :
    FiniteExecution initial observation := by
  unfold TreeContains at hmember
  rw [observations_exploreAll] at hmember
  exact (covers_exploreTree bound initial).1 observation hmember

/-- **SPEC section 7.4.**  Completeness relative to the explicit bound: every
relational branch of length at most the bound occurs in the tree. -/
theorem runFuel_complete_with_bound {bound : Nat} {initial : Config}
    {observation : ExecutionObservation}
    (hrun : FiniteExecution initial observation)
    (hlen : observation.trace.length ≤ bound) :
    TreeContains (exploreAll bound initial) observation := by
  unfold TreeContains
  rw [observations_exploreAll]
  exact (covers_exploreTree bound initial).2 observation hrun hlen

/-- A reported nonterminal prefix is a real reduction sequence of exactly
`bound + 1` steps that has not terminated: the module cannot satisfy a smaller
all-branch step sublevel. -/
theorem nonterminalPrefix_sound {bound : Nat} {initial : Config}
    {obs : List ExecutionObservation}
    {cov : CoversEveryMaximalFiniteBranch bound initial obs}
    {t : TraceOfLength (bound + 1)}
    (h : exploreAll bound initial = .nonterminalPrefix obs cov t) :
    ∃ final : Config, Reduces initial t.events final ∧
      final.status = Status.running := by
  unfold exploreAll at h
  split at h
  · rename_i tprefix hp
    injection h with _ hte
    have hmem := mem_of_head? hp
    obtain ⟨suffix, final, ht, hred, hrun⟩ :=
      prefixes_sound (bound + 1) [] initial tprefix hmem
    refine ⟨final, ?_, hrun⟩
    have hev : t.events = tprefix := by rw [← hte]
    rw [hev, ht]
    simpa using hred
  · exact absurd h (by simp)

/-! ## Exploring a module -/

/-- Explore every permitted branch of a module invocation, reporting an
initialization fault when the module cannot be instantiated. -/
def exploreModule (bound : Nat) (m : Module) (raw : RawInvocation) :
    Except InstantiationFault (Σ' initial : Config, ExecutionTreeResult bound initial) :=
  match initialConfig m raw with
  | .error f => .error f
  | .ok initial => .ok ⟨initial, exploreAll bound initial⟩

theorem exploreModule_error_iff {bound : Nat} {m : Module} {raw : RawInvocation}
    {f : InstantiationFault} :
    exploreModule bound m raw = .error f ↔ initialConfig m raw = .error f := by
  unfold exploreModule
  cases h : initialConfig m raw with
  | error f' => simp
  | ok initial => simp

/-- An invalid module is rejected before any reduction is explored. -/
theorem exploreModule_invalid {bound : Nat} {m : Module} {raw : RawInvocation}
    (h : validate m = false) :
    exploreModule bound m raw = .error .invalidModule :=
  exploreModule_error_iff.mpr (initialConfig_invalid h)

end WasmGemmGnaf.Wasm
