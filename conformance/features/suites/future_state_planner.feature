@status:enforced
Feature: Bounded future-state optimization and planning over graph transitions
  The future-state optimizer must search finite action trajectories reaching goal regions while avoiding forbidden constraint states and enforcing horizon bounds.

  @RF-08 @build
  Scenario: Plan a 2-step trajectory to a goal region avoiding hazards
    Given a start state "s0", intermediate state "s1", and goal state "s2"
    When the bounded graph planner computes a trajectory
    Then the action sequence ["step1", "step2"] reaches "s2" in 2 steps
    And a PlanWitness recording accepted transitions and plan CID is emitted

  @RF-08 @build
  Scenario: Reject trajectories that enter forbidden constraint states
    Given an intermediate state "s1" marked as forbidden
    When the bounded graph planner attempts to plan a trajectory through "s1"
    Then planning fails with a frontier exhausted error and zero forbidden states entered

  @RF-08 @build
  Scenario: Reject initial state inside a forbidden region
    Given a start state "s0" marked as forbidden
    When planning is initiated from "s0"
    Then planning fails immediately with an initial state forbidden error
