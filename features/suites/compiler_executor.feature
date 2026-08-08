Feature: Deterministic compiler executor abstraction

  @RF-02 @build
  Scenario: Execute task array with positional output equivalence between sequential and Rayon executors
    Given a batch of 100 integer input items
    When mapped by the sequential reference compiler executor
    And mapped by the Rayon parallel multicore compiler executor
    Then both mapped output vectors are positionally identical

  @RF-02 @build
  Scenario: Aggregate errors deterministically by lowest input index
    Given a batch of integer input items where item 3 returns a worker error
    When mapped by the Rayon parallel multicore compiler executor
    Then execution returns a worker error at input index 2

  @RF-02 @build
  Scenario: Contain worker panics without aborting the host process
    Given a batch of integer input items where item 5 panics
    When mapped by the Rayon parallel multicore compiler executor
    Then execution returns a worker panic error at input index 4
