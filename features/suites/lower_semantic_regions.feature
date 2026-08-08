@status:enforced
Feature: Lowering reference semantic regions into Boolean, mask, popcount, and fixed-point programs
  Reference floating-point semantic regions and transition scores must lower into integer-only bitmask, popcount, and Q8.8 fixed-point ScoreQ programs with traceable lowering witnesses.

  @RF-12 @build
  Scenario: Lower a reference semantic region into integer bitmask and popcount predicate
    Given a reference semantic region with signature [true, false, true, true] and Hamming radius 1.0
    When the region is lowered into a LoweredBooleanRegion
    Then the integer predicate evaluates to true for signatures within Hamming distance 1
    And evaluates to false for signatures outside Hamming distance 1
    And a LoweringWitnessEntry is recorded

  @RF-12 @build
  Scenario: Quantize reference transition scores into Q8.8 fixed-point ScoreQ values with saturation
    Given floating-point scores 1.5, 500.0, and -500.0
    When scores are quantized into Q8.8 fixed-point representation
    Then 1.5 quantizes to 384 without saturation
    And extreme scores saturate at i16 MAX and i16 MIN

  @RF-12 @build
  Scenario: Reject reference regions exceeding maximum bitmask capacity
    Given a reference region with a 100-bit signature
    When region lowering is attempted
    Then lowering fails with an unrepresentable region error
