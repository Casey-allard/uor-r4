@status:enforced
Feature: Normative CPU-only, multiplication-free, zero-allocation inference contract
  The normative inference execution contract governs the permitted operation set, zero-allocation steady state, and CPU-only execution target.

  @RF-10 @build
  Scenario: Validate inference contract document and module parity
    Given the normative inference contract specification
    When audited by the inference contract verifier
    Then contract version "1.0.0" is verified with 0 steady-state allocations
    And the contract audit certification status is verified

  @RF-10 @build
  Scenario: Enforce permitted operation set and reject forbidden operations
    Given a hot-path inference activity
    When an operation class is audited
    Then permitted bitwise and integer operations are accepted
    And forbidden float and multiplication operations are rejected
