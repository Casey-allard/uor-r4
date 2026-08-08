@status:enforced
Feature: Unsupervised intervention and counterfactual behavioral probes
  Behavioral probes must evaluate sensitivity to causal interventions, invariance under nuisance variation, and enforce anti-memorization guards.

  @RF-01 @build
  Scenario: Evaluate invariance under surface variation and sensitivity under goal changes
    Given a baseline observation "Context text sample"
    When an invariant surface variation probe and a sensitive goal change probe are evaluated
    Then both invariance and sensitivity expectations pass cleanly
    And the anti-memorization guard succeeds

  @RF-01 @build
  Scenario: Reject surface-memorizing tables under goal change intervention
    Given a sensitive goal change probe that produces zero output divergence
    When the probe suite is evaluated by the behavioral harness
    Then evaluation fails with a memorization detected error

  @RF-01 @build
  Scenario: Reject out-of-bound affected span ranges
    Given an observation of length 15
    When an intervention record is created with span [0..20]
    Then record creation fails with a span out of bounds error
