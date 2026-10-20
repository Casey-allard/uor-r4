@status:enforced
Feature: Packed zero-allocation CPU inference kernels over immutable graph arrays
  Inference step calculation over immutable R4G1 GraphView artifacts must use zero-allocation, bounded, stack-resident packed kernels.

  @RF-13 @build
  Scenario: Bounded active-frontier expansion and eviction
    Given a zeroed packed active frontier of capacity 4
    When node 1 with score 100 and node 2 with score 200 are advanced into the frontier
    Then the active frontier count is 2 and contains both nodes

  @RF-13 @build
  Scenario: Accumulate candidate shortlist without double counting
    Given an empty packed candidate shortlist of capacity 4
    When node 10 is accumulated into the shortlist twice
    Then the shortlist count is 1 and contains node 10

  @RF-13 @build
  Scenario: Decode top-K predictions with canonical tie-breaking
    Given a candidate set with duplicate scores and distinct IDs
    When decoded by the packed top-K kernel
    Then the top predictions are sorted by score descending and ID ascending
