@status:enforced
Feature: Normative R4G1 skip-mix serving lane (S0 reconciliation)
  ADR-0001 assigns every production candidate and served token to
  R4G1Runtime. R4Engine may resolve D4 policy but may not substitute its own
  token. These build scenarios establish structural reachability. The separate
  #933 CID-bound full census RATIFIES only the exact canonical bundle,
  population, R4G1Runtime greedy decode, and schema-2 envelope; it does not turn
  this structural row into a universal quality claim.

  @RF-31 @build
  Scenario: artifacts without skip-mix sections preserve normative candidates exactly
    Given a synthetic R4G1 artifact without SKMX or PSIB
    When the normative runtime predicts legacy and served candidates for the same window
    Then the served-candidate projection is identical and has no lane attribution

  @RF-31 @build
  Scenario: a planted SKMX partner reaches the sole normative selector
    Given a synthetic R4G1 artifact with a planted SKMX partner outside the base shortlist
    When the normative runtime predicts served candidates for the planted window
    Then the planted partner is the winner and skip-mix attribution names the base winner

  @RF-31 @build
  Scenario: PSIB is used only when the primary joint row is absent
    Given a synthetic R4G1 artifact with a planted PSIB fallback partner
    When the normative runtime predicts served candidates for a window without a matching SKMX row
    Then the fallback partner is the winner with skip-mix attribution

  @RF-31 @build
  Scenario: skip-mix work uses distinct tokens in the newest eight-token compiler window
    Given a synthetic R4G1 artifact with planted partners outside and inside the compiler window
    When the normative runtime predicts served candidates for a window with more than eight distinct tokens
    Then only the in-window planted partner can affect the winner

  @RF-31 @build
  Scenario: default sampled serving consumes the normative ranked candidates
    Given a planted partner reachable only through R4G1Runtime skip-mix candidates
    When the default sampled production adapter decodes with a pinned seed
    Then the sampled candidate source is the normative served-candidate list

  @RF-31 @build
  Scenario: deployed quality evidence fails closed on any binding mismatch
    Given a full-census deployed-quality report bound to R4G1Runtime and its exact inputs
    When one graph, artifact, corpus, tokenizer, partition, selector, census, or internal absent-section identity binding is changed
    Then production validation rejects the report with a typed mismatch

  @RF-31 @build
  Scenario: D4 policy cannot substitute a reference-scorer token
    Given a production window permitted by token-free D4 policy
    When R4G1Runtime selects the normative served candidates
    Then the production token is the normative winner and no policy token exists
