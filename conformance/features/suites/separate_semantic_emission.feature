@status:enforced
Feature: Separate semantic reasoning and state transitions from language emission
  The semantic operating system kernel must execute pure state transitions and belief updates without token generation, adapting text output via an explicit emission adapter.

  @RF-28 @build
  Scenario: Execute pure semantic state transitions without token emission
    Given an initial state "s0" and a valid 2-step transition sequence to "s2"
    When pure semantic reasoning is executed by the reasoning engine
    Then a valid SemanticStateTrace is produced without generating tokens
    And the trace overall status is Coherent

  @RF-28 @build
  Scenario: Emit language response from verified semantic state trace
    Given a verified coherent SemanticStateTrace from "s0" to "s2"
    When passed to the language emission adapter
    Then a LanguageEmissionResult is produced containing text and token probabilities
    And a multi-dimensional certification report evaluates state coherence and language fidelity separately

  @RF-28 @build
  Scenario: Reject contradictory semantic states before language emission
    Given a transition sequence leading to a Contradictory state
    When pure semantic reasoning is executed by the reasoning engine
    Then execution fails with a contradictory state error before token emission
