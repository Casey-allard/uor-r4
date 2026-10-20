# Frozen 13.13M R4/Spin parameter-capacity campaign (#1019)

- **Current status:** `OPTIONAL_PAUSED / MODEL_SUBGATES_PASS /
  MPS_UNAVAILABLE_HARDWARE_BUDGET / FULL_CAMPAIGN_NOT_RUN`.
- **Contract-freeze status (historical):** `FROZEN_PRE_RUN_CONTRACT / NOT_RUN`.
- **Owner:** [#1019](https://github.com/UOR-Foundation/uor-r4/issues/1019)
  under attention issue #973 and programme root #820.
- **Predecessor:**
  [#1017](r4_softmax_quality_capacity_continuation_1017.md).
- **Machine-readable contract:**
  [`r4_softmax_parameter_capacity_1019_raw.json`](r4_softmax_parameter_capacity_1019_raw.json).
- **Local evidence root:** `.uor-models/research/issue-1019/` (ignored bulk
  population, smoke, parity, and hardware-probe artifacts now exist; full
  training, final qualification, reveal, generation, and replay do not).

## Current signed preflight result — 2026-08-31

> **Current result:** population `PASS`; fixed overfit smoke `PASS`; random-export
> all-twelve-layer Python/Rust preflight parity `PASS`. The signed MPS hardware probe
> ended `UNAVAILABLE_HARDWARE_BUDGET` with `main_run_authorized=false`, so the
> full campaign remains `NOT_RUN`. This is a hardware-time terminal, not an
> attention or parameter-capacity falsification.

The compact machine-readable evidence record is
[`r4_softmax_parameter_capacity_preflight_1019_raw.json`](r4_softmax_parameter_capacity_preflight_1019_raw.json).
The frozen contract below is retained unchanged as the pre-run declaration;
its `NOT_RUN` statements describe the state at contract freeze. This section
appends the later preflight evidence and is the current status.

### Population — `PASS`

The current semantic verifier accepted the signed population and training-view
envelopes, all bound artifacts, exact predecessor boundaries, freshness rules,
and the still-sealed confirmation commitment.

- Dataset manifest CID:
  `blake3:6efbffeb1b6cb20ae9bbcda03428a4b820824224c578a168cd9a65f616f3dd5c`.
- Training-view manifest CID:
  `blake3:bb090c4b87fb62e71ce073c2e4df525745109e71e0db3e9846852a696af5501e`.
- Split-policy CID:
  `blake3:54f0886d3e906a4aeeaa9328ff236440d61d9f16b2f92dcb8c05cac96e54d1aa`;
  tokenizer CID:
  `blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`.
- Train: `275,251,200` token IDs, `275,250,944` complete-context scored
  next tokens, `1,347,394` stories. Development: `250,000` token IDs,
  `249,856` scored next tokens, `1,251` stories. Confirmation: `249,880`
  stored token IDs, `249,856` scored next tokens, `1,197` stories, plus `120`
  sealed prompt token IDs, exactly the `250,000`-token reveal cap.
- The content-bound #1017 development/test last source-story ordinals are
  `47,293`/`48,856`; the new development/test tranches begin at
  `47,299`/`48,874`. All ten published prompt-story CIDs were excluded and
  predecessor sealed-artifact reads remained zero.

### Fixed overfit smoke — `PASS`

The exact `13,130,784`-parameter, 12-layer arm passed the frozen smoke:
`64` sequences and `400` optimizer steps reduced loss from
`8.366631031036377` to `1.508070632815361`, a reduction of
`0.8197517462857979` (81.9751746286%) against the required `0.80`. Elapsed
time was `288.06345129199326` seconds (`0.080017625359` hours) against the
600-second ceiling, with zero attention-off executions.

- Signed smoke result CID:
  `blake3:d1f2e3b3a2d269fbccca1ccdd2f9439392b5a06a3a48f0f90a0825029f1508ec`.
- Bound smoke-admission manifest CID:
  `blake3:bbc5eb7420a2e1bbc8391e5089cb129f77a1387f9d15071498336544fef382e7`.
- Trainer implementation tree CID:
  `blake3:9f1cd533b2e057a5b41bc81f2641b08de42f07c6badb98356333a2de9efb0707`.

### Random-export all-twelve-layer Rust preflight parity — `PASS`

The 32-token enabled prefix selected all `12` layers. Every layer recorded
`32` enabled applications; causal, projection, R4, and output-policy audits
were exact, with zero future reads. Python and Rust both selected token `16`.
Maximum absolute logit delta was exactly
`0.000044345855712890625`, below the frozen `0.005` limit. Provider, Ollama,
and prior-trace reads were all zero, and `attention_off_executions` remained
zero.

- Rust qualification decision CID:
  `blake3:dfe39b41eb39d9f737af003fc7fa1b21c52c1290823849b2dd3d09ce0de53bbb`.
- Python prefix result CID:
  `blake3:0454063d1eb645efe0e76fe044347ed7f940ab7e7eb3f0f6c27e41207256ab09`.
- Rust enabled-audit CID:
  `blake3:1d46be99fc6d3ad6d17d18bf5fd3f3eef10f061056ee2d226b5ea8829e14c7be`.

### Signed MPS hardware admission — `UNAVAILABLE_HARDWARE_BUDGET`

The signed 200-step probe presented `3,276,800` train tokens. Total measured
time was `717.50556925009` seconds (`0.199307102569` hours), including
`698.2614083748776` seconds in the optimizer loop, `18.91384125011973`
seconds for one complete development evaluation, and `0.3303196250926703`
seconds for checkpoint reload. The raw full-run projection was
`59,513.153594312025` seconds (`16.531431553976` hours); after the frozen
`1.25` safety factor it was `74,391.44199289003` seconds
(`20.664289442469` hours), above the `28,800`-second (`8`-hour) ceiling.
Therefore `time_passed=false`.

Peak accelerator memory was `2,673,278,976` bytes (`2.489685058594` GiB) of
`12,713,115,648` reported bytes (`11.840011596680` GiB), a fraction of
`0.21027724831721753`, below the `0.80` ceiling; therefore
`memory_passed=true`. The probe checkpoint is explicitly
`partial_checkpoint_interpretable=false`.

- Signed hardware result CID:
  `blake3:8de57ebef53cda52b62baa619f87566042c59bac2a399ebf6588e53f78e4daf7`.
- Probe-contract CID:
  `blake3:c67a2c4bed089aa8349eb9c6643ee01edbbf2cbbd9bb5d675499539afbbf805c`.
- Probe-checkpoint CID:
  `blake3:9bd6395790ec2cfcca93f7cc7606125c2d783b74319595ec602904f5b486c58d`;
  signed sidecar result CID:
  `blake3:a37360da48fe2b303fa94ffd802bc0e5621d77effd91f77ab4fb49ffcc0da8db`.
- Signed elapsed-sample result CID:
  `blake3:5e1989e2b5d6b3c58a8b9ac6c57f56b223f0cf7aa00a81b198199079b46de5f9`.

### Decision boundary after preflight

Full training, full-training checkpoint selection/export, final Python/Rust
parity, the one-time sealed reveal, generation, deterministic replay, and
finalization are all `NOT_RUN`. Consequently no #1019 held-out NLL, coherent
generation, replay, or capacity-quality claim exists. The positive smoke and
all-layer parity mean this preflight did not falsify the ordinary attention
path or the 13.13M capacity hypothesis; they do not establish either final
quality or geometry advantage.

The signed MPS terminal applies only to the frozen eight-hour offline
PyTorch/MPS implementation. It does not redirect UOR's CPU-native deployed
architecture/runtime and no longer authorizes the historical CUDA branch.
Apple Accelerate/BLAS and MPS remain permitted only for local offline training,
compilation, and bounded tests; CUDA and external GPU execution are out of
scope. One subsequent isolated exact-shape MPS fast-path test used 10 warmup and
40 measured steps with fused AdamW plus deferred logging. It measured
`4.485223 s/step`, slower than the signed `3.491307 s/step`; `fused=True` was
removed immediately. This is a bounded fast-path negative, not a model-quality
or attention result. Preserve the passed population, smoke, and parity
artifacts, but stop #1019 tuning and full-run work. #1019 is optional and
paused; no recurring optimization or research gate follows from this result.

#1019 is an optional, paused quality-capacity improvement. It does not block
using or productizing the working #1017 7.15M coherent-generation prototype. The
simple local entry point is `r4 generate --prompt "..."`, which defaults to
`.uor-models/research/issue-1017/export`. That path remains a bounded,
source-backed, floating-point/matmul/softmax prototype; it does not establish
geometry advantage, transformerlessness, correctness, reasoning, frontier
quality, browser/WASM readiness, or release readiness. This #1017 path is now
the active next step.

### #1041 product-boundary follow-up — 2026-09-01

The active-next-step wording immediately above is now historical. #1039 exposed
the frozen #1017 checkpoint through a disabled-by-default loopback raw-
completion surface, and #1041 performed its frozen normal-use decision. All
dashboard/endpoint mechanics passed; narrative continuation passed `2/3`, but
both supplied-history binding comparisons failed. Terminal
`KEEP_RAW_CONTINUATION_ONLY` retains the path only as a bounded source-backed
single-turn story-continuation reference. It does not authorize a history
serializer, multi-turn/chat adapter, retraining, or prompt widening. See the
[#1041 record](r4_softmax_local_normal_use_1041.md).

## Frozen decision and evidence status at contract freeze (historical)

#1017 completed the only authorized exposure continuation of the
7,155,360-parameter model. It preserved load-bearing ordinary causal softmax
attention in coherent R4/Spin frames and passed parity, all-layer causal/R4
audits, subject-or-scene retention `5/5`, and normalized replay `5/5`, but its
fresh sealed NLL `1.5727521962806827` failed the strict `<1.50` quality gate.
That checkpoint will not receive more exposure, learning-rate tuning, another
seed, or another reveal.

#1019 was the one allowed parameter-capacity decision at contract freeze. It
changes only decoder
depth from six to twelve layers. The attention mechanism, R4 block structure,
tokenizer, split discipline, sampler, and Rust all-layer evidence path remain
unchanged. This is a language-quality campaign, not another attention
experiment, geometry comparison, architecture search, or learning-rate search.

At contract freeze, no #1019 population, preflight, training, checkpoint
selection, export, Rust qualification, sealed reveal, generation, replay, or
finalization result existed. Every such gate was then `NOT_RUN`. This paragraph
is preserved as the historical pre-run declaration; the current signed
preflight result is recorded above.

## Frozen model

Initialize exactly once from seed `1019`; do not transplant or reshape #1017
weights.

| Field | Frozen value |
|---|---:|
| Parameters | exactly `13,130,784` |
| Vocabulary | `4,096` |
| Hidden width | `288` |
| Decoder layers | `12` |
| Query / KV heads | `6 / 6` |
| Head width | `48` |
| R4 blocks per head | exactly `12` |
| FFN width | `768` |
| Context | `256` |
| Initialization seed | `1019` |

The tied embedding/language-model head, bias-free RMSNorm, RoPE, SwiGLU,
learned Q/K/V/O, complete-prefix scaled dot product, stable causal softmax,
weighted-value aggregation, tokenizer, sampling policy, story split, and Rust
coherent R4/Spin generation path are inherited unchanged.

## Frozen population and training

- Reproduce the pinned TinyStories source, train-only tokenizer, and story-level
  split policy.
- Materialize exactly `275,251,200` train token IDs from canonical train
  buckets, preserving approximately `20.9623` tokens per parameter.
- Materialize a new, disjoint 250,000-token development tranche after the
  consumed #1014/#1017 development populations.
- Seal a new, disjoint 250,000-token confirmation tranche after the consumed
  #1014/#1017 test populations. Reserve 249,880 tokens for NLL and five fresh
  24-token prompts.
- Exclude all ten previously published prompt-story CIDs.
- Deny training and selection access to confirmation paths.

Run exactly `16,800` optimizer steps with batch `16`, gradient accumulation
`4`, and context `256`: `16,384` tokens per optimizer step and
`275,251,200` total tokens. Use AdamW betas `0.9/0.95`, weight decay `0.1`,
gradient clipping `1.0`, a 100-step warmup to `3e-4`, one cosine decay to
`3e-5`, complete-development evaluation every 400 steps, and minimum
complete-development NLL for selection. Resume may continue only a
byte-identical interrupted run. No sweep, alternate capacity, seed, schedule,
precision, or retry is permitted.

## Hardware admission and run contract

**Metric to move:** fresh sealed enabled NLL from
`1.5727521962806827` to `<1.50`. The required movement is
`0.0727521962806827` nats/token, approximately 4.63%.

**Reachability ceiling:** the capacity change participates in every scored
token, so 100% of scored positions are reachable. This establishes decision
reachability, not a predicted scaling effect.

**Cheap instrument:** before full training, require all of:

1. exact closed-form parameter count and R4-head divisibility;
2. exact population availability and train/development/confirmation
   disjointness;
3. at least 80% loss reduction while overfitting 64 fixed sequences;
4. enabled Python/Rust random-export top-1 identity and maximum absolute logit
   delta `<=0.005`; and
5. a 200-step backend throughput/memory probe with checkpoint behavior and a
   measured full-run projection.

The projection scales the two measured regular checkpoint saves, adds the
slower measured save another 43 times as the worst-case initial/updated-best
checkpoint cost, and includes 44 complete development evaluations before the
`1.25` safety factor.

**Historical hardware decision at contract freeze (superseded):** launch on the current MPS host only when the measured
projection, including a `1.25` safety factor, is at most eight hours. Peak
accelerator memory must also be no more than 80% of the backend's reported
available or recommended maximum. Otherwise require one pinned deterministic
single-CUDA float32 environment
with TF32 disabled, binding the GPU, driver, dependency/container tree,
deterministic-algorithm mode, and throughput result before training. If neither
backend qualifies, stop `UNAVAILABLE_HARDWARE_BUDGET`; a partial checkpoint is
not evidence. No paid external launch is authorized without explicit owner
approval.

**Historical cost estimate at contract freeze (superseded):** prior M1 evidence projects this rung above the eight-hour
local ceiling, so a local full campaign is not authorized until a fresh probe
contradicts that estimate. Any external campaign must project to at most eight
hours and separately satisfy the monetary-approval gate.

The CPU-native scope correction above supersedes this historical CUDA/external
branch without changing the frozen contract or its observed evidence. No CUDA
or external GPU probe or training run is an active #1019 action.

## Qualification and one-time reveal

After exact training completion and export, but before confirmation reveal:

1. compare one enabled-only 32-token development prefix between Python and
   Rust;
2. require identical top-1 and maximum absolute logit delta `<=0.005`;
3. require all twelve learned layers plus exact causal, projection, R4, and
   output-policy audits; and
4. require zero future, teacher, provider, Ollama, prior-trace, or prior-sealed
   reads.

Only after those gates pass, open confirmation exactly once: score #1019 and
score the frozen #1017 enabled checkpoint once on that same tranche as a
reporting baseline. Only if both NLL thresholds pass, run five candidate Rust
continuations with seeds 3019 through 3023, replay each once, and finalize the
reports once. A negative NLL terminal ends the rung before generation. Do not
execute an attention-off arm; #1019 tests capacity over already-established
attention.

## Definition of Done and outcome branches

Every row is currently `NOT_RUN`. A positive result requires all of:

- exact 16,800-step completion;
- candidate sealed NLL `<1.50` and lower than #1017 on the same new tranche;
- enabled Python/Rust top-1 identity and maximum logit delta `<=0.005`;
- all twelve layers executing through coherent R4/Spin transport;
- exact causal/projection/R4/output-policy work with zero future reads;
- zero teacher, provider, Ollama, prior-trace, or prior-sealed-test reads;
- subject-or-scene retention at least `4/5`;
- valid UTF-8 and no period-1 through period-4 loop for all outputs; and
- exact normalized reload replay `5/5`.

If all rows pass, freeze #1019 as the first ordinary locally executable
R4/Spin quality baseline and open one separate #973 issue for the frozen
multi-resonance/softmax-replacement mechanism against it.

If any row fails after a valid complete campaign, close this bespoke
micro-model ladder. Do not run a third size, exposure extension, seed, or
learning-rate variant. Require a source-pinned, publicly reproduced TinyStories
architecture/training-recipe decision before geometry replacement.

If the hardware admission fails, record `UNAVAILABLE_HARDWARE_BUDGET`; do not
reinterpret the absence of a run as a model-quality negative.

## Claim boundary

The planned campaign uses floating point, multiplication, allocation,
autograd, dense full-prefix dot products, and ordinary softmax during training
and inference. Even a positive result would not establish geometry advantage,
a capacity scaling law, transformerlessness, multiplication-free/table-native
execution, correctness, reasoning, chat, browser/WASM readiness, release
readiness, or frontier capability. #954 remains blocked until #973 records the
follow-on attention-mechanism decision.
