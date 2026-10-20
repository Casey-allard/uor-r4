/-
  Costed traces: sequences of charged events and their accumulated cost.
  Normative source: SPEC.md section 9.1 (`Cost.sequentialCompose`) and
  section 9.3 (positive accounting for every semantic transition).

  Every declaration in this file is proved. Nothing is assumed.
-/
import WasmGemmGnaf.Cost.Event

set_option autoImplicit false

namespace WasmGemmGnaf.Cost

/-- A costed trace is the finite sequence of charged events of one execution,
in execution order. -/
abbrev Trace := List Event

/-- Accumulate a trace onto a running cost, in prefix order. -/
def runCost (init : DynamicVector) : Trace → DynamicVector
  | [] => init
  | e :: rest => runCost (sequentialCompose init e.charge) rest

/-- The cost of a trace: its accumulation from the zero vector. -/
def traceCost (t : Trace) : DynamicVector := runCost DynamicVector.zero t

theorem runCost_nil (init : DynamicVector) : runCost init [] = init := rfl

theorem runCost_cons (init : DynamicVector) (e : Event) (rest : Trace) :
    runCost init (e :: rest) = runCost (sequentialCompose init e.charge) rest := rfl

/-- Accumulation from an arbitrary starting cost is composition with the trace's
own cost. -/
theorem runCost_eq_compose : ∀ (t : Trace) (init : DynamicVector),
    runCost init t = sequentialCompose init (traceCost t)
  | [], init => (sequentialCompose_zero_right init).symm
  | e :: rest, init => by
      have h1 : runCost (sequentialCompose init e.charge) rest
          = sequentialCompose (sequentialCompose init e.charge) (traceCost rest) :=
        runCost_eq_compose rest _
      have h2 : runCost (sequentialCompose DynamicVector.zero e.charge) rest
          = sequentialCompose (sequentialCompose DynamicVector.zero e.charge)
              (traceCost rest) :=
        runCost_eq_compose rest _
      show runCost (sequentialCompose init e.charge) rest
          = sequentialCompose init
              (runCost (sequentialCompose DynamicVector.zero e.charge) rest)
      rw [h1, h2, sequentialCompose_zero_left, sequentialCompose_assoc]

theorem traceCost_nil : traceCost [] = DynamicVector.zero := rfl

theorem traceCost_cons (e : Event) (rest : Trace) :
    traceCost (e :: rest) = sequentialCompose e.charge (traceCost rest) := by
  rw [traceCost, runCost_cons, runCost_eq_compose, sequentialCompose_zero_left]

/-- Trace cost is a homomorphism from concatenation to sequential
composition. -/
theorem traceCost_append (t s : Trace) :
    traceCost (t ++ s) = sequentialCompose (traceCost t) (traceCost s) := by
  induction t with
  | nil => rw [List.nil_append, traceCost_nil, sequentialCompose_zero_left]
  | cons e rest ih =>
      rw [List.cons_append, traceCost_cons, ih, traceCost_cons,
        sequentialCompose_assoc]

theorem traceCost_singleton (e : Event) : traceCost [e] = e.charge := by
  rw [traceCost_cons, traceCost_nil, sequentialCompose_zero_right]

/-- Extending a trace never decreases any coordinate. -/
theorem traceCost_mono_append (t s : Trace) :
    DynamicVector.ComponentwiseLE (traceCost t) (traceCost (t ++ s)) := by
  rw [traceCost_append]
  exact sequentialCompose_le_left _ _

/-- Every event of a trace is charged in that trace's cost. -/
theorem charge_le_traceCost {e : Event} : ∀ {t : Trace}, e ∈ t →
    DynamicVector.ComponentwiseLE e.charge (traceCost t) := by
  intro t
  induction t with
  | nil => intro h; exact absurd h (by simp)
  | cons e' rest ih =>
      intro h
      rw [traceCost_cons]
      rcases List.mem_cons.mp h with rfl | hmem
      · exact sequentialCompose_le_left _ _
      · exact DynamicVector.componentwiseLE_trans (ih hmem)
          (sequentialCompose_le_right _ _)

/-- The total weight of a trace: the sum of the weights its events claim. -/
def traceWeight (t : Trace) : Nat := (t.map Event.weight).sum

theorem traceWeight_nil : traceWeight [] = 0 := rfl

theorem traceWeight_cons (e : Event) (rest : Trace) :
    traceWeight (e :: rest) = e.weight + traceWeight rest := by
  simp [traceWeight]

/-- Positive accounting, accumulated: a run's total cost dominates the running
cost it started from plus every weight it charges along the way. -/
theorem runCost_total_ge : ∀ (t : Trace) (init : DynamicVector),
    init.total + traceWeight t ≤ (runCost init t).total
  | [], init => by
      rw [runCost_nil, traceWeight_nil]
      exact Nat.le_of_eq (Nat.add_zero _)
  | e :: rest, init => by
      rw [runCost_cons, traceWeight_cons]
      have step : init.total + e.weight
          ≤ (sequentialCompose init e.charge).total :=
        transition_accounting_positive init e
      have tail := runCost_total_ge rest (sequentialCompose init e.charge)
      omega

/-- SPEC 9.3: the cost of a trace is at least the total weight it charges. -/
theorem traceWeight_le_traceCost_total (t : Trace) :
    traceWeight t ≤ (traceCost t).total := by
  have h := runCost_total_ge t DynamicVector.zero
  rw [DynamicVector.total_zero, Nat.zero_add] at h
  exact h

/-- A trace of `n` charged events costs at least `n`. -/
theorem length_le_traceCost_total (t : Trace) : t.length ≤ (traceCost t).total := by
  refine Nat.le_trans ?_ (traceWeight_le_traceCost_total t)
  induction t with
  | nil => exact Nat.le_refl 0
  | cons e rest ih =>
      rw [List.length_cons, traceWeight_cons]
      have := Event.one_le_weight e
      omega

/-- A nonempty trace costs strictly more than nothing. -/
theorem traceCost_pos_of_ne_nil {t : Trace} (h : t ≠ []) :
    0 < (traceCost t).total := by
  cases t with
  | nil => exact absurd rfl h
  | cons e rest =>
      have hlen := length_le_traceCost_total (e :: rest)
      simp only [List.length_cons] at hlen
      omega

end WasmGemmGnaf.Cost
