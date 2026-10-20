@status:enforced
Feature: Make graph invariant ownership and loader validation explicit
  Every normative graph invariant must have a declared owner and validation stage, rejecting malformed artifacts for loader-owned invariants before execution.

  @RF-09 @build
  Scenario: Validate full graph invariant ownership matrix mapping
    Given the normative graph invariant inventory
    When mapped to the ownership matrix
    Then all 8 graph invariants have declared primary owners and validation stages

  @RF-09 @build
  Scenario: Reject graph artifact exceeding maximum node degree limit
    Given a graph artifact with maximum node degree 12 against limit 10
    When validated by the loader invariant verifier
    Then validation fails with a degree limit exceeded error

  @RF-09 @build
  Scenario: Reject graph artifact containing dangling edge references
    Given a graph artifact with 5 nodes and an edge referencing target node 99
    When validated by the loader invariant verifier
    Then validation fails with a dangling reference error

  @RF-09 @build
  Scenario: Reject graph artifact with duplicate evidence contributions
    Given a graph node containing duplicate evidence ID 101
    When validated by the loader invariant verifier
    Then validation fails with a duplicate evidence error
