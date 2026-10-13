# Proof Step Update (2026-02-17): Lean O1/O4 Theorem-Shaped Predicates

- Updated `research/formal/lean/PrimeRiemannBridge.lean`.
- Replaced remaining placeholder axioms with theorem-shaped predicates:
  - Added `structure O1Constants` and `def O1Closed (c : O1Constants) : Prop`.
  - Added `structure O4Constants` and `def O4Closed (c : O4Constants) : Prop`.
- O1/O4 predicates now lock constants from `proof_constant_provenance_2026-02-17`.
- Updated final theorem signature:
  - `O5_final_implication (c1 : O1Constants) (c2 : O2Constants) (c3 : O3Constants) (c4 : O4Constants)`
  - assumptions now use `O1Closed c1`, `O2Closed c2`, `O3Closed c3`, `O4Closed c4`.
