/-!
# ONNX subgraph recursion — bounded descent (ADR-057).

`GRAPH` / `GRAPHS` node attributes embed subgraphs; the canonicalizer
recurses carrying a `Nat` depth bound.
-/
namespace UorAddr.Onnx

/-- Descend subgraph nesting, decrementing the bound. -/
def descend : Nat → Nat
  | 0     => 0
  | n + 1 => descend n

/-- `recurse_terminates_at_descent_bound` — total descent to the floor. -/
theorem recurse_terminates_at_descent_bound (n : Nat) : descend n = 0 := by
  induction n with
  | zero => rfl
  | succ k ih => simpa [descend] using ih

end UorAddr.Onnx
