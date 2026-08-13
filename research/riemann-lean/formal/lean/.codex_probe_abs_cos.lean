import PrimeRiemannBridgeMathlib

example (x y : Real) : |Real.cos x - Real.cos y| ≤ |x - y| := by
  simpa using Real.abs_cos_sub_cos_le x y
