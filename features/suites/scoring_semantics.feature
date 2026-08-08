@status:enforced
Feature: Specify fixed-point scoring semantics and deterministic tie-breaking
  Pre-quantized residual scores must accumulate via saturating integer operations with overlap residualization and canonical tie-breaking.

  @RF-26 @build
  Scenario: Accumulate pre-quantized residuals with saturating integer arithmetic
    Given a zeroed score accumulator
    When a root prior residual of 1000 and a child correction of 500 are accumulated
    Then the final score is 1500 with zero heap allocations

  @RF-26 @build
  Scenario: Enforce overlap residualization and no-double-counting rule
    Given a score accumulator containing evidence contribution 42
    When the same evidence contribution 42 is accumulated again
    Then the duplicate evidence is ignored and the score remains unchanged

  @RF-26 @build
  Scenario: Enforce deterministic candidate tie-breaking protocol
    Given candidate A with score 500 and ID 10
    And candidate B with score 500 and ID 20
    When candidates are compared by the deterministic tie-breaker
    Then candidate A ranks higher than candidate B
