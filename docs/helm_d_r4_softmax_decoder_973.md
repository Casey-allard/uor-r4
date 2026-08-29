# HELM-D-R4 full-decoder softmax parity (#973)

Status:
`PASS_HELM_D_R4_GAUGE_SOFTMAX_FULL_DECODER_PARITY_ADVANCE_TO_INTRINSIC_R4`.

This is the first positive full-language-decoder attention result in the active
#973 direction. It establishes that an unchanged ordinary causal attention
function can be represented in UOR R4 blocks with cumulative Spin/H4 local
frames and causal K/V transport. It does not establish an R4 predictive
advantage, intrinsic geometric attention, softmax removal, transformerless
serving, or language-model quality at scale.

## Frozen identities and scope

- HELM-D architectural source:
  `Graph-and-Geometric-Learning/helm@7501deca8f413848bfef804be64ce874b72a3cd7`;
  its actual Lorentz-inner-product score, causal softmax, and normalized
  Lorentz centroid are translated as a compact semantic reference. No upstream
  checkpoint or paper-result parity is claimed.
- Provenance: [audit boundary](../third_party/helm-d-reference/README.md) and
  [machine-readable pin](../third_party/helm-d-reference/UPSTREAM.toml).
- Ordinary donor:
  `blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5`;
  SmolLM2-135M, width 576, 30 layers, 9 query heads, 3 KV heads, head width
  64, vocabulary 49,152.
- Natural-language population:
  `blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf`;
  the promoted runner streamed and verified this CID from the 12,539,119 raw
  corpus bytes rather than trusting the sibling manifest alone;
  the first deterministic D3-held-out document was ID 12, “Autonomous
  communities of Spain.”
- Bounded evaluation: three teacher-forced next-token positions plus a
  two-token greedy continuation. This is a mechanism/parity rung, not a
  perplexity or coherence benchmark.
- Execution: the exact donor executor resolved `available_parallelism` to
  eight workers. The byte-final promoted run took 70.56 seconds for five matched
  arms after the release binary was built.

The donor retains learned Q/K/V, RoPE, complete causal support, stable softmax,
linear value aggregation, `W_o`, residual/FFN blocks, final normalization, and
the LM head. Each 64-lane head is split into sixteen R4 blocks. For orthogonal
model-frame basis `F_i`, local encoding is `F_i^T x`, source-to-query transport
is `F_i^T F_j`, and the aggregate is decoded with `F_i` before unchanged
`W_o`. The discrete Spin/H4 frame identity is exact; the current vector action
is an offline f64/f32 oracle.

## Predeclared gates

- per-logit allowance: `0.02 + 0.001 * max(abs(donor), abs(R4))`;
- mean absolute logit delta: at most `0.002`;
- teacher-forced next-token loss delta: at most `0.002` nats per position;
- identical teacher-forced top-1 and greedy decoded continuation;
- byte-exact independent replay for both donor and coherent R4 arms;
- exact dense causal-work ledger with zero future reads; and
- exact implementation-owned R4 evidence binding the policy, H4 frame-table
  offsets, block actions, and intervention count; and
- equal-work source-frame permutation must cause a maximum logit delta of at
  least `0.02`, proving that the transport seam is live.

## Result

| Check | Observed | Verdict |
| --- | ---: | --- |
| Full-vocabulary logits compared | 196,608 | PASS |
| Maximum absolute donor/R4 logit delta | 0.00001049041748046875 | PASS |
| Mean absolute donor/R4 logit delta | 0.0000022742100540540378 | PASS |
| Teacher-forced top-1 matches | 3/3 | PASS |
| Teacher-forced loss tolerance | 3/3 | PASS |
| Donor replay | bit-exact logits/state/decode | PASS |
| Coherent R4 replay | bit-exact logits/state/decode | PASS |
| Donor and coherent decoded continuation | `, and` / `, and` | PASS |
| Source-frame-permuted continuation | `[[` | live control |
| Source-frame-permuted maximum logit delta | 23.08442449569702 | PASS |
| Coherent key/value transports | 2,700 / 2,700 | exercised |
| Coherent query/output transforms | 1,080 / 1,080 | exercised |
| Coherent R4 blocks encoded / key / value / output | 103,680 / 43,200 / 43,200 / 17,280 | PASS |
| Permuted source-frame block actions | 77,760 | PASS |
| Implementation policy and H4 frame audit | exact reviewed snapshot | PASS |
| Future reads | 0 | PASS |

The canonical result payload CID is
`blake3:05eaad210198fbe39a0645c25b0c890c55d5f3d3dd8a1710472e976a637e2a07`.
The complete evidence is
[`helm_d_r4_softmax_decoder_result_973.json`](helm_d_r4_softmax_decoder_result_973.json).

The decision-bearing command was:

```text
cargo test -p uor-r4-core --release \
  --test helm_d_r4_softmax_decoder_973 --offline -- --ignored --nocapture
```

## Interpretation and next action

Ordinary causal softmax attention now works through the complete local decoder
while its Q/K/V state is expressed and transported in R4/Spin frames. The
destructive control proves that coherent source-to-query frame transport is
functionally necessary on this bounded run. This is the required positive
reference, but it is deliberately gauge-equivalent to the donor and therefore
cannot be cited as a geometric advantage.

The next #973 action is now authorized: freeze and train one intrinsic R4
attention arm that replaces only donor compatibility and linear value
aggregation with a declared R4 distance and geometric weighted centroid. It
must retain this real-language effect and compare against the donor,
gauge-equivalent R4 reference, and equal-budget Euclidean/plain controls.
Multi-resonance softmax replacement, recurrence, scale, E8 expansion, and exact
runtime lowering remain blocked until that intrinsic arm qualifies.

## Attempt 02 successor update — 2026-08-29

This record's bounded evidence and the `HELM-D-R4` gauge-equivalent ordinary-
softmax PASS remain unchanged. The separately trained intrinsic Lorentz-R4
successor stopped before D3 at
`UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT` (result CID
`blake3:da2a63323d6211b8d581e5a4ed75d788eb919ff0f210d2e3beb8a749ee1bc64f`):
normalized-barycenter covariance was `9.1214e-8` against the frozen `1e-8`
limit, and construction-validation NLL was diagnostically worse than the donor
by `1.2531` and the matched flat control by `0.20893` nats/token. No reveal
marker or held-out result exists. No Attempt 03 is authorized under this freeze;
any further intrinsic work must be a newly frozen, source-faithful
learned-manifold successor. Multi-resonance, recurrence, lowering, scale, and
#954 remain blocked. See the
[owning intrinsic record](intrinsic_lorentz_r4_attention_973.md) and the
[compact result summary](intrinsic_lorentz_r4_attention_attempt_02_summary_973.json).
