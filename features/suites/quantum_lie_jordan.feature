Feature: Quantum Geometric Lie-Jordan Splitting & Universal Product Kernel
  @RF-20 @build
  Scenario: Decompose endomorphism operator into Lie symmetries and Jordan observables
    Given a Clifford generator matrix operator in 16D Cayley-Dickson space
    When Lie-Jordan decomposition is performed on the operator
    Then the Lie component is strictly anti-Hermitian
    And the Jordan component is strictly Hermitian
    And the reconstructed operator matches the original matrix

  @RF-20 @build
  Scenario: Execute hot-path integer universal product kernel without float or multiplication
    Given a pair of 8-bit integer operator state bytes
    When the hot-path universal product kernel is evaluated for Lie anti-Hermitian symmetry
    Then the result matches the bitwise XOR and rotation transformation
    And zero floating-point operations or multiplications are executed
