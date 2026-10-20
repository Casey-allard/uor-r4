/-!
# GGUF metadata-ARRAY recursion — bounded descent (ADR-057).

A metadata `ARRAY` value may nest; the parser descends carrying a `Nat`
descent bound and bottoms out at the streamed digest. Modelled as a
structurally-recursive descent that terminates at the bound for every
input.
-/
namespace UorAddr.Gguf

/-- Descend the ARRAY nesting, decrementing the bound; bottoms out at 0. -/
def descend : Nat → Nat
  | 0     => 0
  | n + 1 => descend n

/-- `recurse_terminates_at_descent_bound` — the descent is total and
reaches the floor for every bound. -/
theorem recurse_terminates_at_descent_bound (n : Nat) : descend n = 0 := by
  induction n with
  | zero => rfl
  | succ k ih => simpa [descend] using ih

end UorAddr.Gguf
