# #973 multi-resonance attention-sieve reuse audit

- **Date:** 2026-08-28
- **Issue:** #973
- **Programme root:** #820
- **Decision:** [ADR-0005](adr/0005-predictive-geometric-connection-memory.md)
- **Verdict:** `REUSABLE_GEOMETRIC_SUBSTRATE_PRESENT_ATTENTION_SIEVE_NOT_IMPLEMENTED`
- **Execution status:** read-only repository audit; no resonance qualification run

## Result first

The project's multi-resonance, sin/cos, S3/Hopf, fiber, torsion, and spherical-
harmonic work is a viable basis for replacing the dense reference's softmax
weighting law. It is not yet an implemented attention operator. Current Rust
resonance fields are structural summaries or telemetry: they do not compute a
query-conditioned positive kernel, retain its normalization denominator, or
aggregate transported values. Historical harmonic routers still terminate in
ordinary softmax.

The correct next use of this work is conditional. Direct-attention V2 is
non-promotable, and fresh equal-manifold-budget V3 rejected the current mixed-gauge H4
connection at 3/12 while plain attention reached 12/12 and an inference-time
coherent alternative-connection swap reached 10/12. `ConnectionGaugeCovarianceV4`
Phase I subsequently passed, and its independent 24-case target-free population
plus salted commitment are frozen in PR #1001. Protect that freeze and reveal
once, then bind the actual paired-H4/E8 hierarchy and
fiber/torsion state. Only after that oracle qualifies may its Q/K/V/O roles,
connection, causal mask, support, and outputs be frozen while replacing only
the normalized weighting law with a finite positive resonance kernel. Sin/cos
supply bounded mode coordinates. Tan does not supply an additional independent
mode and must not be used as a global basis.

## Reusable implementation substrate

- `prime_route_attention.rs` defines the fixed-point `UnitS3Q30`, its derived
  `UnitS2Q30` Hopf observation, and `SpinTorsionState`. The S3 sign and the
  fiber/torsion state remain present; Hopf/S2 alone is not a rebuild identity.
- `canonical_lexical_ingestion.rs` exposes `AttentionLevelTrace`, including
  S3, Hopf, fiber, torsion, zeta phase, chart state, and paired-H4/E8
  coordinates for each causal hierarchy level.
- The same module already implements the bounded sin/cos chart and deterministic
  pole switch. Its current chart witness records `tangent_evaluated=false`; it
  is not a learned tangent attention operator.
- `research/riemann-lean/lie_phase_harmonic_probe.py` contains historical low-
  order real S2 harmonic and SU(2) feature formulas. They are probes, not causal
  Q/K/V attention evidence.
- `run_router_harmonic_generalization_v1.py` contains a historical finite
  cyclic sin/cos feature bank. Its transition and readout still call softmax,
  so it is a feature-encoding reference rather than the desired replacement.

The existing `cosine_resonance_q30` hierarchy field is eight componentwise
zeta-phase cosines accumulated by addition. It has no candidate/query-relative
kernel, nonnegative attention weights, denominator, or transported-value
numerator. It must not be renamed or promoted as attention.

## R3 and fiber boundary

Two three-dimensional objects must not be conflated:

1. `T_G S3` is a moving three-dimensional tangent space whose coordinate vector
   is meaningful only with its bound S3 basepoint and frame. Transporting all
   inputs into the current query frame preserves that relationship.
2. The Hopf observation is an S2 direction embedded in R3. It is many-to-one
   and loses the S1 fiber and signed Spin distinction unless those fields are
   carried separately.

An R3 mode implementation is therefore admissible only when the artifact binds
the S3 basepoint/frame plus fiber and torsion. Otherwise use S3/SU(2) modes
directly. S2 spherical harmonics by themselves are insufficient for the full
R4 spin state.

## Tan boundary

There is no Rust `tan` evaluation in the active crates. Historical frame-sweep
records found tan undefined at the adjacent quarter-turn cases where signed
sine carries its largest directional signal. Those records also show that tan
adds no information beyond the ordered `(cos,sin)` pair. The active chart is
therefore correct to use bounded sin/cos values plus an explicit pole switch.
Tan may appear only as a named, bounded local chart coordinate; it is excluded
from the global resonance kernel.

## Required normalized kernel

For the frozen direct-oracle logit `ell(t,i)`, construct a finite spectral
amplitude `A_M` that approximates `exp(ell(t,i)/2)`, then define a pointwise
positive kernel:

```text
w(t,i) = weight_floor + abs(A_M(query_t, transported_key_i))^2
D_t    = sum_i w(t,i)
N_t    = sum_i w(t,i) * transported_value_i
alpha  = w / D_t
read_t = N_t / D_t
```

This form preserves nonnegative weights, exact normalization, and transported-
value semantics. Adding epsilon only after summing the denominator is not
equivalent: the weights would no longer sum to one. The artifact must instead
certify a pointwise weight floor, a denominator floor, or a deterministic
uniform fallback.

A project-aligned first candidate is a finite Fejer-windowed S3/SU(2) amplitude
followed by modulus-square. An alternative is S2 spherical harmonics tensored
with explicit fiber/torsion Fourier modes, including the parity needed to keep
signed Spin antipodes distinct. Radial/tangent-magnitude and any frozen
positional terms must be included if the oracle logit uses them.

Replacing `exp` with a trigonometric score while retaining every query-to-prefix
comparison remains quadratic. The efficiency result begins only when the
finite compound feature map permits `N_t` and `D_t` to be updated once per
observed token and read without rescanning the prefix.

## Frozen comparison after the direct oracle qualifies

Freeze the direct oracle before opening the resonance outcome. Compare:

- stable-softmax geometric oracle;
- full S3/SU(2) resonance;
- S2-only resonance;
- fiber-zeroed and fiber-permuted controls;
- mode-, transport-, order-, and value-permuted controls; and
- an equal-state non-geometric positive-feature kernel.

Predeclare the mode ladder and report minimum weight, minimum denominator,
weight-sum error, kernel error, weight-distribution error, aggregate-value
error, candidate-decision agreement, next-token loss/top-1, future reads,
deterministic replay, state bytes, and operation count. The replacement fails
if a weight becomes negative, the denominator contract fails, causal replay
fails, or the frozen oracle's effect is not preserved within the predeclared
tolerance. If fiber/S3 controls do not lose the effect, the full Spin fiber has
not earned causal credit.

## Runtime boundary

Compiler-side float, multiplication, and trigonometric evaluation are allowed
while qualifying the feature map. Serving-time trigonometric functions are not.
After the recurrence preserves the oracle, mode evaluation, connection action,
and reciprocal normalization must lower to bounded H4/Q29/integer tables,
bit-plane conditional additions, or another witnessed implementation permitted
by the [inference operation contract](transformerless/INFERENCE_OPERATION_CONTRACT.md).

## Research context

- [Transformers are RNNs](https://arxiv.org/abs/2006.16236) gives the factored
  numerator/denominator recurrence for kernel attention.
- [Rethinking Attention with Performers](https://arxiv.org/abs/2009.14794)
  demonstrates positive feature approximations to the softmax kernel.
- [Geometric Deep Learning](https://arxiv.org/abs/2104.13478) describes
  spherical harmonics and Wigner-D/Fourier machinery for equivariant geometric
  processing.

These results support separate parts of the design. None establishes the UOR
construction, its paired-E8 binding, or its runtime contract.

## Successor direction (2026-08-29)

V4 subsequently completed terminal-negative at `13/24`, without adequate
separation from its destructive controls. V4 will not be rerun or retuned. The
HELM-D-R4 became the full-decoder, gauge-equivalent ordinary-causal-softmax
reference with R4/Spin frame transport. Its parity gate now passes; the verdict
and scope are authoritative only in the
[HELM-D-R4 result](helm_d_r4_softmax_decoder_result_973.json). The active #973
successor is intrinsic R4 distance and normalized-centroid attention, followed
conditionally by multi-resonance replacement and recurrent lowering.
