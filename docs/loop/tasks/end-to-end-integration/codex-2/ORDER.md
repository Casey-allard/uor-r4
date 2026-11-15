# ORDER: end-to-end-integration, codex-2

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-1](../codex-1/ORDER.md). Reviewed head: `0e7191cc4`.

## Why this unit exists

The owner's reading is right: attention needs no softmax. The `Read` stage can select by a
rotation-equivariant score and aggregate by the rank weights the engine already uses. ADR-0007
placed the stage; this unit proves the arithmetic before anything depends on it.

The construction under test is spin-1: a token's state is a unit quaternion `u ∈ S³` (its world
pose), the relative pose of a pair is `q_ij = u_i⁻¹ u_j`, a local feature `a ∈ R³` transforms as the
spin-1 representation, and the score is `⟨a_i, q_ij · a_j · q_ij⁻¹⟩`. Under a change of local frame
`u → u g`, `a → D(g)⁻¹ a`, the score must be **exactly invariant**. That is the property the whole
SH/2I programme rests on; if it fails, the tables or the pose convention are wrong.

## Claim record

`loop claim end-to-end-integration --lab-topic sh-spin-probe` was refused with exit 2 because
`crates/uor-r4-integer/src/stack/**` overlaps claude-384's PR #3983. This unit adds only a new test
file and docs, touches none of that path, adds no dependency and does not change `Cargo.lock`; it
continues under the owner's D85 direction.

## Method

- Exact arithmetic in `Z[φ]` (`a + bφ`, `φ² = φ + 1`), no floating point.
- The 120 elements of the binary icosahedral group 2I as quaternions with numerators in `Z[φ]` over
  2: the 16 of the 24-cell and the 96 with two `φ` coordinates.
- The spin-1 action by the quaternion sandwich, not a matrix: `R(q) v = q v q⁻¹`, exact as a
  numerator over the right power of two.
- The score `s_ij = ⟨a_i, q_ij a_j q_ij⁻¹⟩` as an exact rational, compared by cross-multiplication.
- The frame-change test: for every `g ∈ 2I`, `u' = u g`, `a' = D(g)⁻¹ a = R(g⁻¹) a`, and
  `s(u_i', u_j', a_i', a_j') = s(u_i, u_j, a_i, a_j)` exactly.
- A closed-form control: the score is not constant (it separates pairs), and the aggregation is the
  rank rule `1/(i+1)` in Q31, with no exponent.

## Files in scope

- `crates/uor-r4-integer/tests/sh_spin_probe.rs` (new file)
- `docs/loop/tasks/end-to-end-integration/codex-2/**`

Out of scope: any source or schema change, serving wiring, the feature map from tokens to
coefficients, spins `l ≥ 2`, a mechanism status, a dependency, or a GPU run.

## Done when

1. The 120 icosian quaternions are generated exactly, distinct, and unit-norm in `Z[φ]`; the group is
   closed under multiplication over a sample.
2. The spin-1 frame-change invariance holds with **exact integer equality** for every `g ∈ 2I` on at
   least three pairs and three feature triples.
3. The score separates pairs (a non-constant control) and the rank aggregation uses no exponent.
4. The test passes; `git diff --check`, the harness contract rule, the REPORT check and the
   exact-head review pass.

## Cancel

Stop if the exact representation cannot be built without floats, if the invariance fails (report the
counterexample instead of weakening the test), or if the unit needs a source change. Do not build the
`l ≥ 2` ladder here.

STOP and report when done.
