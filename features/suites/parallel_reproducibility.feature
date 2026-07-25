Feature: Normative reproducibility and canonical artifact byte equality under compiler parallelism

  Scenario: Assert verbatim normative reproducibility invariant statement
    Given the normative reproducibility invariant specification
    Then the invariant statement matches the Issue 167 verbatim acceptance criteria

  Scenario: Verify sequential vs multicore parallel thread sweep byte equality
    Given a dataset of integer observation items
    When evaluated by the parallel reproducibility harness across thread counts 1, 2, and 4
    Then all thread count outputs produce 100% bit-identical byte digests
