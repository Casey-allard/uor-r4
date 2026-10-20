@status:enforced
Feature: Compositional-planning benchmarks and typed plan-witness semantics
  The frozen S4 compositional-planning reference model (docs/compositional_planning_spec_844.md)
  generates deterministic task instances with replayable gold plans, verifies plan witnesses
  independently, rejects invalid intermediate steps and missing evidence, reports honest typed
  declines, and is stable under relabeling. Reference-only / off-serving-path; teacher-forced
  scope (S3 boundary); ordinal confidence, not calibrated (S2 boundary).

  @RF-32 @build
  Scenario: The gold plan for a graph-navigation task verifies
    Given a graph-navigation compositional-planning task with seed 0
    When the gold plan is verified
    Then the plan-witness verdict is valid

  @RF-32 @build
  Scenario: The verifier rejects a path that enters a forbidden region
    Given a graph-navigation compositional-planning task with seed 0
    When a two-step east path is submitted
    Then the plan-witness verdict is invalid

  @RF-32 @build
  Scenario: Relabeling preserves the plan and its action sequence
    Given a constraint-satisfaction compositional-planning task with seed 1
    When the task is relabeled
    Then the relabeled gold plan verdict is valid
    And the relabeled action sequence equals the original

  @RF-32 @build
  Scenario: A multi-hop-evidence plan stripped of cited evidence is invalid
    Given a multi-hop-evidence compositional-planning task with seed 2
    When the gold plan's cited evidence is removed and verified
    Then the plan-witness verdict is invalid

  @RF-32 @build
  Scenario: An unsolvable episode reports a typed decline
    Given a graph-navigation compositional-planning task with seed 0
    When the gold plan is marked as a no-plan decline and verified
    Then the plan-witness verdict is a typed decline
