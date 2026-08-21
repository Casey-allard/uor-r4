@status:enforced
Feature: Typed selective-prediction serving surfaces (legacy-coverage mode)
  The #838 typed status schema is served on every production surface in
  legacy-coverage mode: abstentions carry the typed cause, calibration
  presence fails closed, and no surface hides an abstention.

  @RF-30 @build
  Scenario: CLI abstention record carries the typed cause and coverage
    Given a deployed D4 abstention with the novel policy label
    When the CLI abstention record is built
    Then the record reads outcome abstention with cause and coverage distributionally-novel and carries no confidence field

  @RF-30 @build
  Scenario: native declined response carries the typed selective block
    Given a serving cascade whose R4G1 tier abstained
    When the native selective block is built
    Then it reports status abstention with cause distributionally-novel and null confidence and evidence
    And a cascade that only failed reports no selective block

  @RF-30 @build
  Scenario: OpenAI-compatible abstention is a typed structured error
    Given a serving cascade whose R4G1 tier abstained
    When the OpenAI-compatible surface envelopes the abstention
    Then the response is HTTP 422 with the vendored selective-prediction error type and the typed abstention code

  @RF-30 @build
  Scenario: streaming abstention terminates with a typed error event
    Given a typed abstention code
    When the streaming decline frames are built
    Then no content chunk is emitted and the frames are one typed error event then the DONE marker

  @RF-30 @build
  Scenario: present selective-calibration data fails closed
    Given a bundle directory carrying a selective-calibration sidecar
    When the selective-calibration probe inspects the bundle
    Then the probe reports a hard incompatibility and an empty directory reports none

  @RF-30 @build
  Scenario: wasm boundary returns typed values and never traps
    Given the wasm graph bundle is not installed
    When the typed wasm response surface is invoked
    Then it returns a typed hard-incompatibility value instead of trapping
