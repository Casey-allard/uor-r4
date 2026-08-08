@status:enforced
Feature: Semantic state space and typed transition dynamics
  Semantic state space S and typed transition dynamics T: S x A -> S must evaluate preconditions, constraints, belief likelihoods, and trajectory step bounds deterministically.

  @RF-27 @build
  Scenario: Evaluate valid action transition T(s, a)
    Given an initial semantic state "s0" with vector [0.0, 0.0] and signature [0]
    When a semantic action "move_right" with delta vector [1.0, 0.0] and mask flip [1] is applied
    Then the transition succeeds with target state "s0_move_right"
    And the target state has vector [1.0, 0.0] and signature [1]

  @RF-27 @build
  Scenario: Enforce precondition failure on forbidden initial state
    Given an initial semantic state "s_invalid" with negative vector [-1.0, 0.0]
    When an action requiring non-negative coordinates is applied
    Then the transition fails with a precondition error

  @RF-27 @build
  Scenario: Reject transitions into constraint-forbidden regions
    Given a hazard constraint centered at [5.0, 5.0] with radius 1.0
    And an initial state at [0.0, 0.0]
    When an action attempts to step to [5.0, 5.0]
    Then the transition fails with a forbidden state error

  @RF-27 @build
  Scenario: Satisfy goals and evaluate belief likelihoods
    Given a goal target region centered at [10.0, 10.0] with radius 2.0 and minimum confidence 0.8
    When a state "s_target" at [10.0, 11.0] with confidence 0.9 is evaluated
    Then the goal is satisfied by the state
    And the belief likelihood is higher than a state at [0.0, 0.0]

  @RF-27 @build
  Scenario: Enforce maximum step limits on bounded trajectories
    Given a trajectory with maximum 2 steps
    When 3 step actions are applied sequentially
    Then the 3rd step fails with a maximum steps exceeded error
