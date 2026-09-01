# R4 layerwise-normalized retained-readout prompt-capacity result (#973)

- **Issue:** [#973](https://github.com/UOR-Foundation/uor-r4/issues/973)
- **Candidate:** `R4LayerwiseNormalizedRetainedReadoutV1` /
  `R4LayerwiseNormalizedRetainedReadoutLanguagePathV1`
- **Frozen control:** qualified `R4RetainedLanguagePathV1`, executed through
  the equal-work candidate path with fixed readout gain `g = 0`
- **Terminal:** `LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`
- **Result CID:**
  `blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`
- **Decision:** end the parameter-free readout ladder and pivot to a freshly
  frozen learned associative binding/readout

## Decision

The layerwise-normalized retained-state readout improved both fresh held-out
language fit and the independently frozen V3 prompt-swap score relative to the
qualified V1 control. Candidate prompt gain was `0.0286980210` nats per target
token versus `0.0073316237` for V1, a positive delta of `0.0213663973`, and it
won `339/512` directions. This is valid positive movement, but it missed both
predeclared effect-size gates: absolute gain `0.0433216988` and
candidate-minus-V1 gain `0.0253415693`. State-off collapsed exactly to zero,
and causal, replay, artifact, forbidden-read, wall-clock, and independent-
verification controls passed. The binding result is therefore `PARTIAL`, not
qualification.

## Exact frozen mechanism

Let `h_t` be the unchanged final hidden state, `a_{l,t}` each layer's existing
post-output-projection and post-state-off retained residual, `N` the existing
learned final RMSNorm, `E` the existing tied embedding/head, and `L = 2` the
frozen layer count. The only candidate change was

```text
logits_t = E @ (N(h_t) + (g / sqrt(L)) * sum_l N(a_l,t))
```

The candidate fixed `g = 1`; the matched V1 control fixed `g = 0` while
executing the same per-layer normalization, accumulation, scaling, residual
addition, and vocabulary-head work. There was one tied vocabulary matrix
multiply and no new learned parameter or persistent state.

Everything else remained frozen: exact-H4 addresses and transport, key/value
recurrence, decay and delta-write gates, occupied-slot softmax, read-before-
write order, residual/MLP path, parameter names and initialization, training
slice and deterministic order, seed `9738`, optimizer schedule, and the
`2,730`-step dose. The qualified predecessor artifact was not retrained.

| Ledger field | Frozen value |
|---|---:|
| Learned parameters | `252,160` |
| Full-context state | `23,040` f32 values / `92,160` bytes |
| Validity state | `240` bits |
| Construction windows | `43,680` |
| Training decisions | `5,241,600` |
| Optimizer steps | `2,730` |

## Frozen populations and provenance

`R4RetainedPromptSwapContrastV3` was selected strictly after the revealed V2
boundary at source story ordinal `241,074`. The first selected story ordinal
was `241,100`; selection examined `4,143` eligible development stories and
ended at ordinal `324,230`. The population contains `256` pairs, `512`
bidirectional comparisons, and `8,192` scored continuation tokens. Each pair
has 48-token prompts with the same final four token IDs but different complete
prompts and different 16-token continuations.

V3 excluded the exact CID-bound union of all `512` V1 and all `512` V2
stories. The `1,024`-story exclusion witness is
`blake3:d33160871a87a9cec7912f394d7546b63f5ebc6323eacc58a65244e9f1148c1c`;
the prior population CIDs are
`blake3:c11a7c935139ca169460b90c01392d7c9e0929e4c10710e76e6c8f74cbdf0340`
and
`blake3:258f143eedbbb7067dc512db929a42166ad8a492fc059542409f419a3b46942e`.
Selection used strict UTF-8, prohibited token IDs 0, 1, and 2 in the first 64
content tokens, and used the development split defined by raw-story BLAKE3
modulo 100 in `90..94`.

The source was `roneneldan/TinyStories`,
`TinyStoriesV2-GPT4-train.txt`, revision
`f54c09fd23315a6f9c86f9dc80f725de7d8f9c64`, with `2,227,753,162`
bytes and SHA-256
`6418d412de72888f52b5142c761ac21a582f7d1166f0bfbdb5f03ccfdec90443`.
The tokenizer CID was
`blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`;
the split-policy CID was
`blake3:54f0886d3e906a4aeeaa9328ff236440d61d9f16b2f92dcb8c05cac96e54d1aa`.
The population directory remained mode `000` until both artifact CIDs were
fixed; its immutable reveal marker records one reveal.

The fresh language slice was independently frozen at source-token offset
`155,782,142`. It contains `249,986` tokens, `2,066` windows, and `247,920`
decisions; its source-story ordinals are `847,141..848,492` and capacity-story
ordinals are `762,819..764,049`. Its CID is
`blake3:79e5e74e3e85f10ed8eb44ea7c37fca7fceba4e2cb2c227db0f37340fcf4d0f3`.
The bound index CID is
`blake3:0032889e32b38801476223c5bed7e401d77b61afbbd6cf9afddaceee18e2136e`;
the `1,231` held-out story CIDs have witness
`blake3:eda2d5f1c7ba2ac10f1725842307b8e911f59237cee2f412533fa34041f06b4a`.
The preparation verified both ordinal separation and zero prompt/held-out
story-CID intersection.

## Evidence identities

| Envelope or artifact | CID |
|---|---|
| Preparation | `blake3:1b3c25176da7c816f7821e503a1f0419f1c896e08a21f408a1ff3ab37e7ded10` |
| Implementation tree | `blake3:18100792c6086879e3af8ae54618494168f0c305b36bac0e298971db393f7b5a` |
| Prompt commitment | `blake3:59f77f532f8870ccbc44dae8b72b44f9816efdc2c7ddc37f6185aa1713bb8de1` |
| Prompt population | `blake3:165be397b73041afd39aa65ae796400ea539399f8586729ad19a168c4daa9e93` |
| CPU probe | `blake3:f001e0a7421bbfa565e7f0fa045a3515ec82fc494d404105c3db5e7e4e2f9f55` |
| Execution plan | `blake3:16afa1dc185ad0451cf8f0a9927fe7ca00d1209d73b120e114dd28c7a082e61d` |
| Started | `blake3:90bf245f0b263c675c656c8672114ab6cd3a88ad9e1618dae605ba6113667005` |
| Run contract | `blake3:69c30a15c6aea1400dcbc3b14aae823cad050c5d903a7403e2b8b685a075c775` |
| Qualified V1 artifact | `blake3:d1417b325e7a545057cd38e9f1a723933a3682801877433d20e98774a5e9172d` |
| Candidate artifact | `blake3:8d31e15c355aade1ccc2592dc5fb1caf14a5f056862621e7b467858569a1c1e4` |
| One-time reveal | `blake3:079bee84db32513c5d6c0cb54cbff1e70b163902efa934d950204090985b3f5a` |
| Terminal result | `blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532` |
| Independent verification | `blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa` |

The one-candidate run used Apple Accelerate on CPU with four threads. CUDA was
forbidden and MPS was not used. The five-step probe projected
`1,728.141247` seconds under the `3,000`-second admission ceiling; the complete
`2,730`-step run took `1,447.763973` seconds. Prompt evaluation took
`37.117315` seconds under its `300`-second ceiling. The independent verifier
reproduced all 13 bound comparisons from fresh model instances with no
optimizer, optimizer steps, or training-batch reads. The candidate artifact
was `1,010,792` bytes and its terminal record binds
`fixed_before_prompt_reveal = true`.

The canonical one-line envelopes are:

- [preparation](r4_layerwise_normalized_retained_readout_prompt_capacity_preparation_973_raw.json)
- [population commitment](r4_layerwise_normalized_retained_readout_prompt_capacity_population_commitment_973_raw.json)
- [CPU probe](r4_layerwise_normalized_retained_readout_prompt_capacity_probe_973_raw.json)
- [started](r4_layerwise_normalized_retained_readout_prompt_capacity_started_973_raw.json)
- [one-time reveal](r4_layerwise_normalized_retained_readout_prompt_capacity_reveal_973_raw.json)
- [terminal result](r4_layerwise_normalized_retained_readout_prompt_capacity_result_973_raw.json)
- [independent verification](r4_layerwise_normalized_retained_readout_prompt_capacity_independent_verification_973_raw.json)

## Fresh general-language result

| Arm | NLL (nats/token) | Top-1 |
|---|---:|---:|
| Candidate, initial | `8.3238252820` | `0.0000766376` (`19/247,920`) |
| Layerwise-readout candidate | `3.7126411677` | `0.3166182640` (`78,496/247,920`) |
| Frozen V1 | `3.8850003883` | `0.2972813811` (`73,702/247,920`) |
| Candidate, state off | `5.0621787313` | `0.2335471120` (`57,901/247,920`) |
| Frozen V1, state off | `4.2143048622` | `0.2311269764` (`57,301/247,920`) |

Training improved candidate NLL by `4.6111841143` nats and top-1 by
`31.6541626` percentage points from initialization. Relative to frozen V1,
the candidate improved NLL by `0.1723592206` nats and top-1 by `1.9336883`
percentage points. Turning retained state off cost `1.3495375637` NLL nats
and `20,595` correct decisions. Every frozen fresh-language gate passed,
including final NLL, learning effect, V1 non-regression, load-bearing state,
and zero forbidden reads. Artifact reload was exact; stationary/artifact
direct replay differed by at most `0.0000076294` in logits and passed the
frozen replay tolerance.

## Prompt-capacity result

For each direction `d`, the frozen score was

```text
g_d = (log P(y_d | p_d) - log P(y_d | paired_prompt_d)) / 16
G   = mean(g_d) over 512 directions
```

| Arm | Own NLL | Crossed NLL | Mean gain `G` | Wins |
|---|---:|---:|---:|---:|
| Frozen V1 | `3.6930405921` | `3.7003722158` | `0.0073316237` | `298/512` |
| Candidate | `3.4798765288` | `3.5085745497` | `0.0286980210` | `339/512` |
| Frozen V1, state off | `4.0741507185` | `4.0741507185` | `0` | `0/512` |
| Candidate, state off | `4.9831804348` | `4.9831804348` | `0` | `0/512` |

The candidate-minus-V1 gain was `0.0213663973`. It passed directional wins,
own-prompt NLL non-regression, exact replay, state-off collapse, positive-gain,
and forbidden-read gates. It missed the absolute-gain threshold by
`0.0146236778` and the capacity-delta threshold by `0.0039751720`. The binding
prompt verdict is `PROMPT_CONDITIONING_PARTIAL`.

## Consequence and nonclaims

This result supports only the narrow empirical statement that moving
normalization inside the two retained-layer reads improved V3 matched-prompt
gain and fresh-language quality relative to the equal-work qualified V1
control, but not by the frozen required effect sizes. V3 is distinct from the
earlier V1 and V2 prompt populations; raw scores across those populations are
not treated as a matched comparison.

The result does not qualify this candidate as a prompt-capacity or geometric-
attention mechanism. It does not establish coherent or autonomous generation,
reasoning, correctness, H4 superiority, geometry-native or exact/table
lowering, browser/WASM integration, release readiness, or a general or
frontier model. Generation, reasoning, lowering, and geometry-native lowering
remain `NOT_RUN`. It does not revoke the separate ordinary causal-attention
result or the qualified retained V1 result.

Per the frozen terminal branch, this candidate receives no generation, retry,
widening, scalar tuning, `g = 2`, third parameter-free normalization placement,
second fitted trajectory, or lowering run. The parameter-free readout ladder
ends here. The sole successor direction is a freshly frozen learned associative
binding/readout with new independent evaluation data; the revealed V3
population is not tuned or scored as a new held-out criterion again.
