# Language: en
Feature: Compiler memory-budget and backpressure model for multicore compilation
  As a system architect
  I want concurrency-aware memory budget modeling and bounded queue backpressure limiting
  So that multicore compilation operates with bounded peak RSS and deterministic error rejection under memory constraints

  Scenario: Derive concurrency-aware memory budget for worker threads
    Given a memory budget request of 268435456 bytes for 4 worker threads
    When memory budget derivation is evaluated
    Then the derived worker thread count is 4 with per-worker scratch of 4194304 bytes

  Scenario: Reject memory budget below mandatory minimum threshold with typed error
    Given a memory budget request of 10485760 bytes for 4 worker threads
    When memory budget derivation is evaluated
    Then memory budget derivation fails with a budget too small error

  Scenario: Enforce bounded task capacity in backpressure limiter
    Given an in-flight backpressure limiter with capacity 1
    When 2 task slot acquisitions are attempted sequentially
    Then the 1st acquisition succeeds and the 2nd acquisition fails with a backpressure limit reached error
