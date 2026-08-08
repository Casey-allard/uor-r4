Feature: Teacher parity and benchmarks for the compiled transformerless runtimes
  These scenarios run the pinned SmolLM2-135M teacher side by side with both
  compiled transformerless runtimes (the legacy TLS store and the R4G1 graph
  engine) on the same token histories. Every accuracy and speed figure here is
  an Empirical Criterion measured on pinned fixtures and pinned against a
  conservative threshold; no equivalence with the teacher is claimed. The
  corpus-replay scenario (S6) compares predictions against the recorded
  teacher labels in the compiled corpus records; it does not re-run the live
  teacher. When the pinned fixtures are absent (as in CI), each scenario
  vacuously passes,
  following the same skip convention as the kappa-reproduction suite — check
  fixture presence before trusting a green run.

  @RF-29 @build
  Scenario: S1 provenance — every parity input is content-addressed by blake3 kappa
    Given the pinned SmolLM2 teacher and compiled transformerless bundle are present
    When the provenance of every parity input is recorded
    Then every parity input carries a blake3 kappa and the graph provenance matches the compiled artifact

  @RF-29 @build
  Scenario: S2 teacher-forced replay accuracy of the legacy TLS store
    Given the pinned SmolLM2 teacher and compiled transformerless bundle are present
    When the legacy TLS store is replayed against the teacher on pinned prompts
    Then the legacy store parity metrics meet the pinned empirical criteria

  @RF-29 @build
  Scenario: S3 teacher-forced replay accuracy of the R4G1 graph engine
    Given the pinned SmolLM2 teacher and compiled transformerless bundle are present
    When the R4G1 graph engine is replayed against the teacher on pinned prompts
    Then the R4G1 graph parity metrics meet the pinned empirical criteria
    And graph abstentions during replay stay within the pinned bound

  @RF-29 @build
  Scenario: S4 free-running generation speed against the teacher
    Given the pinned SmolLM2 teacher and compiled transformerless bundle are present
    When free-running generation is timed for the teacher and both compiled runtimes
    Then both compiled runtimes sustain a higher token rate than the teacher

  @RF-29 @build
  Scenario: S5 compiled runtime kernel invariants
    Given the pinned SmolLM2 teacher and compiled transformerless bundle are present
    When the compiled runtime kernel invariants are examined
    Then the kernel op census contains no multiply or divide operation
    And the compiled prediction hot path performs zero heap allocations
    And prediction witnesses agree with plain predictions

  @RF-29 @build
  Scenario: S6 in-distribution corpus replay against recorded teacher labels
    Given the pinned SmolLM2 teacher and compiled transformerless bundle are present
    When the corpus records are replayed against the recorded teacher labels
    Then the in-distribution parity metrics meet the pinned empirical criteria

  @RF-29 @build
  Scenario: S7 certifier FMM candidate on novel contexts
    Given the pinned SmolLM2 teacher and compiled transformerless bundle are present
    When the certifier FMM candidate is replayed against the teacher on pinned prompts
    Then the FMM candidate produces a reproducible novel-context measurement
