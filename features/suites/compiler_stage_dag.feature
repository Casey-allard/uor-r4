Feature: Compiler stage ownership and parallelization DAG

  Scenario: Verify 28-stage compiler inventory classification
    Given the normative compiler stage DAG inventory
    When evaluated for completeness
    Then exactly 28 pipeline stages are fully classified across the 4 concurrency classes

  Scenario: Protect the sequential canonical finalization spine
    Given the normative compiler stage DAG inventory
    When the sequential canonical finalization spine is queried
    Then exactly 6 stages belong to the sequential canonical finalization spine
    And stage IDs "S12", "S24", "S25", "S26", "S27", and "S28" are strictly single-threaded
