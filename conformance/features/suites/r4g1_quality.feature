@status:enforced
Feature: R4G1 generated-response quality
  The active R4G1 path must reject pathological text before it reaches the browser.

  @RF-22 @build
  Scenario: reject the repetitive geometric response to hello
    Given the R4G1 runtime returned the browser's repetitive hello response
    When the server validates the generated response
    Then the response is rejected as unusable

  @RF-22 @build
  Scenario: reject replacement-character gibberish
    Given the R4G1 runtime returned replacement-character gibberish
    When the server validates the generated response
    Then the response is rejected as unusable

  @RF-22 @build
  Scenario: reject low-readability symbol output
    Given the R4G1 runtime returned low-readability symbol output
    When the server validates the generated response
    Then the response is rejected as unusable

  @RF-22 @build
  Scenario: reject a long identical-character run
    Given the R4G1 runtime returned a long identical-character run
    When the server validates the generated response
    Then the response is rejected as unusable

  @RF-22 @build
  Scenario: accept a concise readable response to hello
    Given the R4G1 runtime returned a concise readable hello response
    When the server validates the generated response
    Then the response is accepted as usable

  @RF-22 @build
  Scenario: allow ordinary repeated words
    Given the R4G1 runtime returned a readable response with ordinary repetition
    When the server validates the generated response
    Then the response is accepted as usable
