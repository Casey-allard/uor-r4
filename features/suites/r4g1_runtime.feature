@status:enforced
Feature: Strict R4G1 server selection and browser research-demo policy
  The server defaults to R4G1 without fallback; the browser defaults to the geometric research demo while keeping R4G1 explicit.

  @RF-23 @build
  Scenario: select R4G1 when no engine was saved
    Given the browser has no saved engine selection
    When the server resolves the synthesis engine
    Then the selected engine is R4G1

  @RF-23 @build
  Scenario: use legacy only when explicitly selected
    Given the browser explicitly selected the legacy engine
    When the server resolves the synthesis engine
    Then the selected engine is Legacy TLA/TLS

  @RF-23 @build
  Scenario: show the geometric research demo as the active browser option
    Then the browser UI selects the geometric research demo, keeps R4G1 unselected, and does not offer automatic fallback

  @RF-23 @build
  Scenario: fail explicitly when R4G1 is unavailable
    Given the R4G1 runtime is unavailable
    When the R4G1 chat endpoint builds its unavailable response
    Then it returns HTTP 503 without invoking a fallback engine
