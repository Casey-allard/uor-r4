# REPORT: end-to-end-integration, codex-2

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic sh-spin-probe` was refused with exit 2 because
  `crates/uor-r4-integer/src/stack/**` overlaps claude-384's PR #3983 (and the earlier claim on
  `one-model-end-to-end` is held by the same PR). This unit adds only a new test file and docs,
  touches none of that path, adds no dependency and does not change `Cargo.lock`; it continues under
  the owner's D85 direction.
- **The first 2I generation was wrong.** Using `(1, 1, φ, φ)` for the 96 elements gives norm
  `1 + φ/2`, not 1. The unit-norm test caught it immediately. The correct set is the even
  permutations of `(±1, ±φ, ±1/φ, 0)`, whose norm is `(1 + φ² + 1/φ²)/4 = 1`.
- **The eight axis elements needed numerators `±2`, not `±1`,** to satisfy `|n|² = den²` at the
  shared denominator 2.
- **This is spin 1 only.** The spherical-harmonic ladder (`l ≥ 2`), the map from a token's state to
  coefficients, and any serving wiring are not built. The score is a pairwise invariant; it is not
  a trained objective and no retrieval quality is measured.
- **The cost is counted, not measured.** The rank aggregation demonstrates that softmax is not
  needed; it is not a read implementation.
- The exact `Z[φ]` arithmetic uses `i128` and grows with the number of products; the probe keeps
  operands small. A serving form needs fixed-width bounds, which this unit does not establish.

## What changed

- `crates/uor-r4-integer/tests/sh_spin_probe.rs` (new, test-only):
  - exact `Z[φ]` (`φ² = φ + 1`) and cross-multiplied rational arithmetic;
  - the 120 binary-icosahedral quaternions as numerators over 2, unit-norm and closed under
    multiplication (144 products checked);
  - the spin-1 action by the quaternion sandwich `q v q⁻¹` (no matrix, no float);
  - the pair score `⟨a_i, q_ij a_j q_ij⁻¹⟩` with `q_ij = q_i⁻¹ q_j`;
  - the frame-change test `u → u g`, `a → D(g)⁻¹ a` for every `g ∈ 2I`;
  - a non-constant control and a rank-weighted, exponent-free aggregation.

Size: about 380 lines of test and the ORDER/REPORT.

## Measurements

- 120 elements generated, all with `|n|² = 4`; 144 sampled products remain in the group.
- **1,080 exact frame-change comparisons** (3 pairs × 3 feature triples × 120 group elements), every
  one bit-identical by cross-multiplied rational equality. No tolerance is used.
- The score separates pairs (the non-constant control holds).
- No exponent appears: the aggregation is `1/(rank+1)` normalized to Q31.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --test sh_spin_probe` — `3` passed.
- `rustfmt --check` on the new file — clean.
- `cargo clippy --locked --no-deps -p uor-r4-integer --test sh_spin_probe` — no warning in the new
  file (the crate's other test targets have pre-existing failures).
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: the probe tests the property the design needs (exact frame invariance), not a
  proxy, and the failure mode (wrong 2I set) was caught by the strongest test, not hidden.
- `scope-guard`: a new test file only; no source, schema, dependency or mechanism status.
- `test-freeze`: a new file; no existing test changed.

## Still open

- `l = 2..3`: the spherical-harmonic basis and the Wigner-D tables for 2I, with the same exactness
  test at each `l`.
- The feature map: how a token's 4-word state and lane state become SH coefficients. The cheap
  version is a lookup into a precomputed SH codebook; the learned version is a projection.
- The serving form: fixed-width integer bounds for the score, a selected support (window + top-k),
  and the `ReadOut` adapter.
- A retrieval probe with planted symmetry, comparing the spin score against popcount and L1
  controls; this is where a quality claim could start.
- The ADR-0007 `Read` stage gains `sh-spin` as a new arm once `l ≥ 2` and the feature map exist.

## Plain summary

The core arithmetic of the proposed non-softmax read is exact: 120 icosian poses, the quaternion
sandwich for spin-1 features, and a pair score that is bit-identical under every one of the 120
changes of local frame, with no float and no exponent. Two generation errors were caught by the
unit-norm test before anything depended on them. The spherical-harmonic ladder, the token-to-
coefficient map and the served form are the next units.
