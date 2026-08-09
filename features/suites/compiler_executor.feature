Feature: Deterministic compiler executor abstraction

  @RF-02 @build
  Scenario: Execute task array with positional output equivalence between sequential and Rayon executors
    Given a batch of 100 integer input items
    When mapped by the sequential reference compiler executor
    And mapped by the Rayon parallel multicore compiler executor
    Then both mapped output vectors are positionally identical

  @RF-02 @build
  Scenario: A worker panic propagates to the caller
    Given a batch of integer input items where item 5 panics
    Then mapping the batch propagates the worker panic
