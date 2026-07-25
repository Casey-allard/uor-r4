# Language: en
Feature: Compiler thread-pool, jobs configuration, and oversubscription policy
  As a system architect
  I want explicit compiler concurrency controls with precedence resolution and dedicated named thread pools
  So that multicore compilation operates deterministically without oversubscription or uncontrolled global state

  Scenario: Resolve jobs configuration with CLI argument precedence
    Given a compiler jobs configuration request with CLI argument 4 and environment variable "16"
    When jobs precedence resolution is evaluated
    Then the resolved thread count is 4 with source "CliArg"

  Scenario: Resolve jobs configuration with environment variable fallback
    Given a compiler jobs configuration request with no CLI argument and environment variable "6"
    When jobs precedence resolution is evaluated
    Then the resolved thread count is 6 with source "EnvVar"

  Scenario: Reject invalid zero thread count with typed error
    Given a compiler jobs configuration request with CLI argument 0
    When jobs precedence resolution is evaluated
    Then resolution fails with a zero jobs forbidden error

  Scenario: Reject invalid non-numeric thread count string with typed error
    Given a compiler jobs configuration request with no CLI argument and environment variable "invalid_num"
    When jobs precedence resolution is evaluated
    Then resolution fails with an invalid job count error for "invalid_num"
