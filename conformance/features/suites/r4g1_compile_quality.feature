@status:enforced
Feature: R4G1 compilation quality gates
  Compilation must fail clearly when inputs are missing or quality is below baseline.

  @RF-21 @build
  Scenario: report missing corpus metadata
    Given the configured corpus metadata path is missing
    When R4G1 compilation inputs are validated
    Then compilation fails with the missing metadata error

  @RF-21 @build
  Scenario: reject a graph below the TLA baseline
    Given a graph quality report below the TLA baseline
    When the R4G1 quality gate validates the report
    Then the quality gate rejects the graph below baseline

  @RF-21 @build
  Scenario: reject a graph at the TLA baseline but below the pinned floor
    Given a graph quality report at or above the TLA baseline
    When the R4G1 quality gate validates the report
    Then the quality gate rejects the graph for digression

  @RF-21 @build
  Scenario: accept a graph at the pinned quality anchors
    Given a graph quality report at the pinned quality anchors
    When the R4G1 quality gate validates the report
    Then the quality gate accepts the graph

  @RF-21 @build
  Scenario: accept a dynamic teacher graph at its same-corpus TLA baseline
    Given a graph quality report using a same-corpus TLA quality profile
    When the R4G1 quality gate validates the report
    Then the quality gate accepts the graph

  @RF-21 @build
  Scenario: reject a graph that digresses in bits per token
    Given a graph quality report with digressed bits per token
    When the R4G1 quality gate validates the report
    Then the quality gate rejects the graph for digression

  @RF-21 @build
  Scenario: reject a graph that digresses in top-1 agreement
    Given a graph quality report with digressed top-1 agreement
    When the R4G1 quality gate validates the report
    Then the quality gate rejects the graph for digression
