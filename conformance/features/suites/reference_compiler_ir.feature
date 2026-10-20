@status:enforced
Feature: Reference floating-point semantic compiler and intermediate representation (IR)
  The reference floating-point graph compiler must compile observations into a deterministic IR capable of answering state transitions and emissions prior to Boolean lowering.

  @RF-25 @build
  Scenario: Compile pinned mini-corpus into deterministic reference IR with CIDs
    Given a pinned mini-corpus of 2 text observations
    When the reference compiler pipeline executes all 5 stages
    Then a valid ReferenceGraphIr is produced with content CID
    And the IR contains observations, states, regions, and objective reports

  @RF-25 @build
  Scenario: Answer state transition and emission queries directly from reference IR
    Given a compiled ReferenceGraphIr containing states "state_0" and "state_1"
    When a state transition query is executed for "state_0" under action "next"
    Then the transition returns state "state_1"
    And the emission prediction for "state_0" returns token probabilities

  @RF-25 @build
  Scenario: Run differential harness comparison against baseline teacher loss
    Given a compiled ReferenceGraphIr with teacher loss 0.25
    When compared against baseline teacher loss 0.26 with tolerance 0.05
    Then the differential comparison passes cleanly
