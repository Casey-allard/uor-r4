# R4 direct retained-readout prompt-capacity result (#973)

- **Issue:** [#973](https://github.com/UOR-Foundation/uor-r4/issues/973)
- **Candidate:** `R4DirectRetainedReadoutV1` /
  `R4DirectRetainedReadoutLanguagePathV1`
- **Frozen control:** qualified `R4RetainedLanguagePathV1`, executed through
  the equal-work candidate path with fixed readout gain `g = 0`
- **Terminal:** `DIRECT_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`
- **Result CID:**
  `blake3:71dd85e610dcc50b74cb2bb2068e5a1a433ac5df5db2a4f8fde22fb41735889c`
- **Decision:** record the partial result; do not generate, retry, widen, or
  lower this candidate

## Decision

The direct retained-state readout improved both fresh held-out language fit and
the independently frozen prompt-swap score relative to qualified V1. Candidate
prompt gain was `0.0215897894` nats per target token versus `0.0076304198`
for V1, a positive delta of `0.0139593696`, and it won `343/512`
directions. That is real positive movement, but it missed both predeclared
effect-size gates: absolute gain `0.0433216988` and candidate-minus-V1 gain
`0.0253415693`. State-off collapsed exactly to zero, and causal, replay,
artifact, and forbidden-read controls passed. The binding result is therefore
`PARTIAL`, not qualification.

## Exact frozen mechanism

Let `h_t` be the unchanged final hidden state, `a_{l,t}` each layer's
existing post-output-projection and post-state-off retained residual, `N` the
existing learned final RMSNorm, and `E` the existing tied embedding/head.
The only candidate change was

```text
logits_t = E @ (N(h_t) + g * N(sum_l a_{l,t}))
```

The candidate fixed `g = 1`; the matched V1 control fixed `g = 0` while
executing the same collection, accumulation, normalization, gating, residual,
and vocabulary-head work. There was one tied vocabulary matrix multiply and
no new learned parameter or persistent state.

Everything else remained frozen: exact-H4 addresses and transport, key/value
recurrence, decay and delta-write gates, occupied-slot softmax, read-before-
write order, residual/MLP path, parameter names and initialization, training
slice and order, seed `9738`, and the `2,730`-step dose. The predecessor
artifact was not retrained.

| Ledger field | Frozen value |
|---|---:|
| Learned parameters | `252,160` |
| Full-context state | `23,040` f32 values / `92,160` bytes |
| Validity state | `240` bits |
| Construction windows | `43,680` |
| Training decisions | `5,241,600` |
| Optimizer steps | `2,730` |

## Frozen population and provenance

`R4RetainedPromptSwapContrastV2` was selected strictly after the revealed V1
boundary at source story ordinal `153,977`. It contains `256` pairs,
`512` bidirectional comparisons, and `8,192` scored continuation tokens.
Each pair has 48-token prompts with the same last four token IDs but different
complete prompts and different 16-token continuations. Selection used strict
UTF-8, prohibited token IDs 0, 1, and 2 in the first 64 content tokens, used
the development split defined by raw-story BLAKE3 modulo 100 in `90..94`,
examined `4,395` eligible stories, and ended at source ordinal `241,074`.

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
fixed, then was revealed exactly once.

The fresh language slice was separately frozen at source token offset
`155,532,141`, after the prior paired-H4 held-out range. It contains
`249,986` tokens, `2,066` windows, and `247,920` decisions; its source
story ordinals are `845,784..847,140` and capacity-story ordinals are
`761,588..762,818`. Its CID is
`blake3:1d9df266c6e08c813c262dc671906d3909baf08024d77c5f2d9bfc0bcd4548c1`;
the bound index CID is
`blake3:0032889e32b38801476223c5bed7e401d77b61afbbd6cf9afddaceee18e2136e`.

## Evidence identities

| Envelope or artifact | CID |
|---|---|
| Preparation | `blake3:1b8bcb10fcc1023e4a02e4a06751e8f0ad154d31b16cabf02a25c54ee43e6d4a` |
| Implementation tree | `blake3:d0b05d7fddff1208de7e0d9153b787db5faf24785c427ec8ed6bb6ed247ef4ac` |
| Prompt commitment | `blake3:f8f826f24ee464d09b36ae65df4d052c0d42a4380705c09d36b9f085b0b28d40` |
| Prompt population | `blake3:258f143eedbbb7067dc512db929a42166ad8a492fc059542409f419a3b46942e` |
| CPU probe | `blake3:0e8360e0c3f67524664c4cb0a8ae4d88b54d7f77e2e60e1427324afb5e959e5f` |
| Execution plan | `blake3:16afa1dc185ad0451cf8f0a9927fe7ca00d1209d73b120e114dd28c7a082e61d` |
| Started | `blake3:7b7654660f4f40f85588b1515fbdf75b5d9a3059a1198cfe2e2f08e6d7cd2c95` |
| Run contract | `blake3:07fda18240d6577149b6e4b3dcd08205da38e5dad19e372d31ef74d0fd2b8996` |
| Qualified V1 artifact | `blake3:d1417b325e7a545057cd38e9f1a723933a3682801877433d20e98774a5e9172d` |
| Candidate artifact | `blake3:6c66f5542a4513c610819b79210792cfe75c8afcdd13572b433ebddac23d688c` |
| One-time reveal | `blake3:693767eb8156eee49507d7f72c2e786a326e7e61f68bd4f04d3820692bf9c839` |
| Terminal result | `blake3:71dd85e610dcc50b74cb2bb2068e5a1a433ac5df5db2a4f8fde22fb41735889c` |
| Independent verification | `blake3:b8ad3b6fa6d6ab9e429b3bd8d2a5060215d15230cd272e7272f27b7eef54785b` |

The one-candidate run used Apple Accelerate on CPU with four threads. CUDA was
forbidden and MPS was not used. The five-step probe projected
`1,639.941006` seconds under the `3,000`-second admission ceiling; the
complete `2,730`-step run took `1,313.036999` seconds. Prompt evaluation
took `35.226951` seconds under its `300`-second ceiling. The independent
verifier reproduced all 13 bound comparisons from fresh model instances with
no optimizer, optimizer steps, or training-batch reads.

The canonical one-line envelopes are:

- [preparation](r4_direct_retained_readout_prompt_capacity_preparation_973_raw.json)
- [population commitment](r4_direct_retained_readout_prompt_capacity_population_commitment_973_raw.json)
- [CPU probe](r4_direct_retained_readout_prompt_capacity_probe_973_raw.json)
- [started](r4_direct_retained_readout_prompt_capacity_started_973_raw.json)
- [one-time reveal](r4_direct_retained_readout_prompt_capacity_reveal_973_raw.json)
- [terminal result](r4_direct_retained_readout_prompt_capacity_result_973_raw.json)
- [independent verification](r4_direct_retained_readout_prompt_capacity_independent_verification_973_raw.json)

## Fresh general-language result

| Arm | NLL (nats/token) | Top-1 |
|---|---:|---:|
| Direct-readout candidate | `3.7374367989` | `0.3154243304` (`78,200/247,920`) |
| Frozen V1 | `3.9010778353` | `0.2963294611` (`73,466/247,920`) |
| Candidate, state off | `4.8608654036` | `0.2340311391` |
| Frozen V1, state off | `4.2333325731` | `0.2288883511` |

The candidate improved NLL by `0.1636410364` nats and top-1 by
`1.9094869` percentage points relative to V1. Every frozen fresh-language
gate passed, including load-bearing state-off and zero forbidden reads.
Artifact reload was exact; stationary/direct replay differed by at most
`0.0000076294` in logits and passed the frozen replay tolerance.

## Prompt-capacity result

For each direction `d`, the frozen score was

```text
g_d = (log P(y_d | p_d) - log P(y_d | paired_prompt_d)) / 16
G   = mean(g_d) over 512 directions
```

| Arm | Own NLL | Crossed NLL | Mean gain `G` | Wins |
|---|---:|---:|---:|---:|
| Frozen V1 | `3.7415367661` | `3.7491671859` | `0.0076304198` | `313/512` |
| Candidate | `3.5521331251` | `3.5737229145` | `0.0215897894` | `343/512` |
| Frozen V1, state off | `4.1225959960` | `4.1225959960` | `0` | `0/512` |
| Candidate, state off | `4.7902186257` | `4.7902186257` | `0` | `0/512` |

The candidate-minus-V1 gain was `0.0139593696`. It passed directional wins,
own-prompt NLL non-regression, exact replay, state-off collapse, and forbidden-
read gates. It missed the absolute-gain threshold by `0.0217319094` and the
capacity-delta threshold by `0.0113821997`. The binding prompt verdict is
`PROMPT_CONDITIONING_PARTIAL`.

## Consequence, nonclaims, and sole next experiment

This result supports only the narrow empirical statement that exposing the
existing retained residual directly to the tied head increased prompt-swap gain
and fresh-language quality relative to V1, but not by the frozen required
effect sizes. It does not qualify coherent or autonomous generation, reasoning,
correctness, H4 superiority, geometry-native or exact/table lowering,
browser/WASM integration, release readiness, or a general or frontier model.
Those scopes remain `NOT_RUN` or unestablished. It does not revoke the
separate ordinary causal-attention result or qualified retained V1.

Per the frozen branch contract, this candidate receives no generation,
retry, widening, scalar tuning, or lowering run. The sole fresh successor is
`R4LayerwiseNormalizedRetainedReadoutLanguagePathV1`, changing only the
readout from

```text
E @ (N(h_t) + g * N(sum_l a_{l,t}))
```

to

```text
E @ (N(h_t) + (g / sqrt(L)) * sum_l N(a_{l,t}))
```

with `L = 2`, candidate `g = 1`, and matched control `g = 0`. It adds no
learned parameters or state, keeps one tied vocabulary matrix multiply, and
freezes the same recurrence, data, initialization, deterministic order,
`2,730`-step dose, and unchanged language/prompt/control thresholds before
execution. It must use a CID- and story-disjoint V3 prompt population selected
after V2 plus a new fresh held-out slice; V2 is never scored or tuned again.
If any unchanged gate misses, the parameter-free readout ladder stops and #973
pivots to learned associative binding/readout. There is no scalar tweak and no
third normalization attempt.
