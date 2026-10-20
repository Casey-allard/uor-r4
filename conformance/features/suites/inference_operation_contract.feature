@status:enforced
Feature: Inference operation contract
  @RF-11 @build
  Scenario: contract document and module versions agree
    Given the normative inference operation contract document
    When the machine-readable inference operation contract version is loaded
    Then the document and module contract versions agree
