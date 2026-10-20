@status:enforced
Feature: Von Neumann quantum density operator and cover induction
  The quantum cover criterion (issue #108) must match analytical entropy
  formulas and accept only partitions with real information gain.

  @RF-18 @build
  Scenario: maximum-entropy operator attains the ln n bound
    Given a maximum-entropy density operator of dimension 8
    When its von Neumann entropy is computed
    Then the entropy equals the natural logarithm of 8

  @RF-18 @build
  Scenario: pure state carries no entropy
    Given a density operator with a pure distribution
    When its von Neumann entropy is computed
    Then the entropy is zero

  @RF-18 @build
  Scenario: an informative partition is accepted for cover induction
    Given observations whose halves predict disjoint tokens
    When the quantum entropy gain of the aligned split is evaluated
    Then the gain equals ln 2 and the partition is accepted

  @RF-18 @build
  Scenario: a noise partition is rejected for cover induction
    Given observations whose halves predict disjoint tokens
    When the quantum entropy gain of the interleaved split is evaluated
    Then the gain is zero and the partition is rejected
