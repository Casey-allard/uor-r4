@status:enforced
Feature: Bounded semantic-transition planning with replayable plan witnesses
  The portable bounded planner (docs/bounded_semantic_transitions_spec_843.md) reads the
  packed PSCH/PTRN/PGOL sections through R4G1Runtime::plan_bounded, plans within fixed
  capacities using only P-4 operations and caller-owned scratch, emits a self-contained
  witness that replays without any model output, and declines with a typed reason rather
  than truncating. Portable-runtime/certifier scope; no deployed serving surface calls this
  planner. LIMITED capability: the arm is established on 12/20 development cells where a
  baseline exists to beat, not on greedy-solvable tasks; final-partition evidence is unavailable.

  @RF-33 @build
  Scenario: A bounded episode reaches the goal without entering a forbidden region
    Given a packed grid planning artifact with a forbidden cell on the direct path
    When the portable planner runs a bounded episode
    Then a plan is emitted and no step enters a forbidden region

  @RF-33 @build
  Scenario: The emitted witness replays valid from its own bytes
    Given a packed grid planning artifact with a forbidden cell on the direct path
    When the portable planner runs a bounded episode
    Then the emitted witness replays as valid

  @RF-33 @build
  Scenario: A right answer through an invalid intermediate step is rejected
    Given a witness whose terminal state satisfies the goal but whose path crosses a forbidden region
    When the witness is replayed independently
    Then the replay verdict is invalid at the offending step

  @RF-33 @build
  Scenario: An unreachable goal declines rather than fabricating a plan
    Given a packed grid planning artifact whose goal lies beyond the horizon
    When the portable planner runs a bounded episode
    Then the episode declines with no plan and emits no steps

  @RF-33 @build
  Scenario: A budget beyond the frozen capacity declines
    Given a packed grid planning artifact with a forbidden cell on the direct path
    When the portable planner runs an episode whose horizon exceeds the frozen capacity
    Then the episode declines for capacity

  @RF-33 @build
  Scenario: An artifact without planning sections serves exactly as before
    Given an artifact carrying no planning sections
    When the engine is asked for a bounded plan
    Then no planning result is produced and serving is unchanged
