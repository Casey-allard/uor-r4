@status:enforced
Feature: Promoted skip-mix serving lane (S1)
  The deployed serving decision (R4Engine::predict_decision) routes next-token
  selection through the promoted skip-mix lane. Compiled bundles carry the
  optional SKMX joint table and PSIB fallback sections, and the lane discovers
  a served candidate from those tables for the window's own tokens; on a bundle
  without the sections the decision is byte-identical to the base
  (absent-section identity). Deployed causal evidence: #822/#908
  (+28.45permille, docs/skipmix_endtoend_causal_908.md). Reroute:
  crates/uor-r4-api/src/engine.rs; compile-path fit:
  crates/uor-r4-graph-cli/src/lib.rs.

  @RF-31 @build
  Scenario: the skip-mix joint table surfaces a co-occurrence partner
    Given a skip-mix joint table binding content 10 and last token 20 to partner 99
    When the deployed lane looks up content 10 with last token 20
    Then partner 99 is surfaced as a supported skip-mix candidate

  @RF-31 @build
  Scenario: an unbound joint key surfaces no skip-mix candidate
    Given a skip-mix joint table binding content 10 and last token 20 to partner 99
    When the deployed lane looks up content 30 with last token 40
    Then no skip-mix candidate is surfaced

  @RF-31 @build
  Scenario: the psi-bag fallback surfaces an unconditioned content partner
    Given a psi-bag fallback binding content 10 to partner 77
    When the deployed lane consults the psi-bag for content 10
    Then partner 77 is surfaced as a supported fallback candidate
