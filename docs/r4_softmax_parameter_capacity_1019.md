# Frozen 13.13M R4/Spin parameter-capacity campaign (#1019)

- **Status:** `FROZEN_PRE_RUN_CONTRACT / NOT_RUN`.
- **Owner:** [#1019](https://github.com/UOR-Foundation/uor-r4/issues/1019)
  under attention issue #973 and programme root #820.
- **Predecessor:**
  [#1017](r4_softmax_quality_capacity_continuation_1017.md).
- **Machine-readable contract:**
  [`r4_softmax_parameter_capacity_1019_raw.json`](r4_softmax_parameter_capacity_1019_raw.json).
- **Planned local evidence root:** `.uor-models/research/issue-1019/`
  (ignored bulk populations, checkpoints, parity, reveal, generation, and
  replay reports; none exists as qualifying evidence at contract freeze).

## Decision and current evidence status

#1017 completed the only authorized exposure continuation of the
7,155,360-parameter model. It preserved load-bearing ordinary causal softmax
attention in coherent R4/Spin frames and passed parity, all-layer causal/R4
audits, subject-or-scene retention `5/5`, and normalized replay `5/5`, but its
fresh sealed NLL `1.5727521962806827` failed the strict `<1.50` quality gate.
That checkpoint will not receive more exposure, learning-rate tuning, another
seed, or another reveal.

#1019 is the one allowed parameter-capacity decision. It changes only decoder
depth from six to twelve layers. The attention mechanism, R4 block structure,
tokenizer, split discipline, sampler, and Rust all-layer evidence path remain
unchanged. This is a language-quality campaign, not another attention
experiment, geometry comparison, architecture search, or learning-rate search.

No #1019 population, preflight, training, checkpoint selection, export, Rust
qualification, sealed reveal, generation, replay, or finalization result is
reported here. Every such gate is `NOT_RUN`. A frozen configuration is not
evidence that the model trains, fits the hardware budget, meets the NLL gate,
or produces coherent text.

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

**Hardware decision:** launch on the current MPS host only when the measured
projection, including a `1.25` safety factor, is at most eight hours. Peak
accelerator memory must also be no more than 80% of the backend's reported
available or recommended maximum. Otherwise require one pinned deterministic
single-CUDA float32 environment
with TF32 disabled, binding the GPU, driver, dependency/container tree,
deterministic-algorithm mode, and throughput result before training. If neither
backend qualifies, stop `UNAVAILABLE_HARDWARE_BUDGET`; a partial checkpoint is
not evidence. No paid external launch is authorized without explicit owner
approval.

**Cost estimate:** prior M1 evidence projects this rung above the eight-hour
local ceiling, so a local full campaign is not authorized until a fresh probe
contradicts that estimate. Any external campaign must project to at most eight
hours and separately satisfy the monetary-approval gate.

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
