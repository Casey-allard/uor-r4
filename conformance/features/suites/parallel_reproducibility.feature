Feature: Normative reproducibility and canonical artifact byte equality under compiler parallelism

  @RF-15 @build
  Scenario: Assert verbatim normative reproducibility invariant statement
    Given the normative reproducibility invariant specification
    Then the invariant statement matches the Issue 167 verbatim acceptance criteria

  @RF-15 @build
  Scenario: Verify sequential vs multicore parallel thread sweep byte equality
    Given a dataset of integer observation items
    When evaluated by the parallel reproducibility harness across thread counts 1, 2, and 4
    Then all thread count outputs produce 100% bit-identical byte digests

  @RF-15 @build
  Scenario: Pin identical cover-edge output under fragment interleavings
    Given worker-local immutable discovery fragments for cover edge discovery
    When fragments are merged after arbitrary completion/interleaving order
    Then canonical stable sort and dedup produce one byte-identical edge sequence
