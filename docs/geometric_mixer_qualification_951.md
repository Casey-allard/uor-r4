# G1 one-layer geometric mixer qualification (#951)

Date: 2026-08-26

Verdict: **REDESIGN_REPRESENTATION**

Claim scope: bounded empirical qualification of one experimental layer-29
mixer and its tokenizer-bound memory adapter. This record does not claim an
all-layer transformerless decoder, broad language quality, production
readiness, performance, or multiplication-free execution.

The machine-readable dataset bindings, per-round losses, held-out controls,
support/memory observations, and 32-token transcripts are retained in
[`geometric_mixer_qualification_951_raw.json`](geometric_mixer_qualification_951_raw.json).
The deterministic fitted parameters are retained as negative evidence in
[`geometric_mixer_checkpoint_951.json`](geometric_mixer_checkpoint_951.json);
they are **not** accepted for all-layer promotion.

## Frozen identities and configuration

- Source: `HuggingFaceTB/SmolLM2-135M-Instruct` revision
  `7e27bd9f95328f0f3b08261d1252705110c806f8`
- Source-weights CID:
  `blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5`
- Tokenizer CID:
  `blake3:944d1262d516abd56a8156dd3058a73a1bf3dc19419527592d854d162f288073`
- G0 base mixer identity:
  `blake3:a63eed546dc6b88a6806f43a238f458f2ec276c7baf6034452ed21fc9baf1fc9`
- G0 base memory-adapter identity:
  `blake3:7fa11963f23f0e7f6b69a62129a2c01f8de71099d68a622b515a35e3ceffba44`
- Frozen dataset CID:
  `blake3:b9ce10e7010724ffd8c42acd327e497f5f52559eefdf53f0de85aba289bdff97`
- Seed: `95120260826`
- Projection owner: `uor-matmul exact GEMM` at pinned revision
  `b13c98449948174f590e337c4dc25dfc394a07d0`, four fixed workers
- Retained negative checkpoint content digest:
  `blake3:8c84b8fe5b2d19be635fccf505b23a92e5daae1d3a93eda8d3465387ac75e139`
- Retained negative mixer identity:
  `blake3:65f9f3a6e4db1690acf1d56ed5cbe0352a02f7ff36f1a992347e447652ecb18c`
- Retained negative memory-adapter identity:
  `blake3:5ec83ccdd60b7fb7db985cef2b719fbe42e897285753fcdcc62e360c1d96897c`
- Report digest:
  `blake3:98cae8e27808b7a5bd4e330bfd389a13f4720643da8315224e0510b89ab72469`

All source parameters remained frozen. The local command loaded the pinned
snapshot read-only and made no Ollama, hosted-provider, or network inference
call. The only learned parameters were the existing mixer query/key/value and
output projections, biases, output gain, and bounded-support logits.

## Declared objective and population

The exact predeclared loss was:

```text
0.55 * mean_squared(operator-target)/(mean_square(target)+1e-6)
+ 0.25 * sampled_next_token_cross_entropy/ln(16)
+ 0.20 * support_cross_entropy/ln(candidate_count)
```

The operator target was the frozen source layer-29 attention output. The token
term used the true next token plus 15 deterministic negatives through frozen
source embedding rows. The support target assigned 80% mass to the frozen
source-attention top support and 20% to one persistent-memory span.

The dataset was frozen before fitting: 18 training and 9 held-out positions
selected from G0-P1/G0-P2 teacher prefixes and the deterministic G0-P1 student
prefix. The held-out partition contained six teacher and three student
positions. Every example had one real persistent-memory span and a matched
memory-permuted arm. The selection/split rule and every residual, Q/K/V, and
source-logit CID are retained in the raw report.

## Source-free hard preflight

The real trace set was not opened until all three gates passed. The preflight
record explicitly reports `source_trace_opened: false`.

| Gate | Bound | Observation | Verdict |
| --- | --- | --- | --- |
| Tiny overfit | at most 64 examples, at most 500 steps, at least 50% loss reduction | 8 examples, 320 steps; `0.008245710516348481 -> 0.00023744947884551948` (97.1203% reduction) | PASS |
| Finite difference | `output_projection[0]`, epsilon `0.001`, allowed error `0.002078976249322295` | analytic `0.003943587187677622`, numeric `0.003948807716369629`, absolute error `0.000005220528692007065` | PASS |
| Checkpoint round trip | digest preserved and focused inference bit-identical | before/after `blake3:cf899bef600e97cd8f03fc89da4eabf17e2aac172b7fa79817b367d86283b579` | PASS |

Preflight report digest:
`blake3:01c9510ca16d79f0fdc78d64f45b1bca2de5e3c4ab5a4f09eacbb1baef31001a`.

## Bounded fitting result

The command used the full predeclared maximum of three rounds because the
primary gate had not passed. Each round completed 80 deterministic full-batch
steps and remained far below its one-hour ceiling.

| Round | Wall seconds | Total loss | Operator term | Sampled-token term | Support term | Round reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Initial | - | 1.0046638350 | 0.9999999470 | 0.9999985298 | 1.0233210822 | - |
| 1 | 72.205 | 0.9284835392 | 0.9999996556 | 0.9999993576 | 0.6424192422 | 7.5827% |
| 2 | 72.683 | 0.9277007315 | 0.9999996987 | 0.9999992086 | 0.6385053760 | 0.0843% |
| 3 | 72.681 | 0.9274201393 | 0.9999995861 | 0.9999990894 | 0.6371028324 | 0.0302% |

No step clipped the declared maximum gradient norm of 2.0. The maximum
observed gradient norm was 1.66459465.

The support term learned, but the operator-alignment and sampled-token terms
remained effectively flat at 1.0. This is the central negative observation:
the current representation can change bounded support and memory use, but it
did not learn the frozen source operator or sampled next-token target on this
population.

## Held-out matched controls

The primary rule was
`(coordinate_permuted_loss - real_loss) / coordinate_permuted_loss >= 0.05`.

| Population/arm | Real loss | Coordinate-permuted loss | Relative real advantage | Gate |
| --- | ---: | ---: | ---: | --- |
| All held-out (9) | 0.9414138397 | 0.9851714902 | **4.4416278%** | FAIL: required 5% |
| Teacher prefixes (6) | 0.9433101018 | 0.9867582321 | 4.4031181% | FAIL: required 5% |
| Student prefixes (3) | 0.9376213153 | 0.9819980065 | 4.5190205% | FAIL: required 5% |

The real sampled-token normalized loss was `0.9999990463`; coordinate
permutation produced `0.9999985033`, so the token-derived metric did not
improve. Full source-logit vectors are content-bound per example in the raw
report. A new full-vocabulary real-versus-permuted delta census was **NOT
RUN** after the terminal primary gate; the inherited G0 reachability result
remains 48,896 changed logits at the layer-29 seam and is not reinterpreted as
G1 quality evidence.

Memory was load-bearing on its predeclared metric:

- real memory probability: `0.10343818697664472`
- memory-permuted probability: `0.04907133450938596`
- real-minus-permuted delta: `0.05436685246725877`
- real held-out loss: `0.9414138396581014`
- memory-permuted held-out loss: `0.9467250638537936`
- relative real advantage over memory permutation: `0.0056101020227266535`

Real support covered 13 distinct prefix positions and one memory span across
the held-out population; it did not collapse to one prefix position. The
current bounded population intentionally contained one persistent-memory span
per example, so diversity across multiple distinct memories was not exercised.

## Five-prompt retained transcripts

All five real-treatment prompts produced distinct 32-token sequences. All
were UTF-8 decodable, support stayed bounded to four, and none entered a
period-1 through period-4 cycle.

| ID | Retained rendered response | Distinct prefix positions | Mean memory weight | Cycle 1-4 |
| --- | --- | ---: | ---: | --- |
| G1-P1 | “Plants need sunlight to undergo photosynthesis, a process that converts light energy into chemical energy, which is then used to produce glucose and other essential nutrients.” | 9 | 0.297995 | none |
| G1-P2 | “Here are three practical tips for staying organized at work: 1. **Set Clear Goals and Prioritize Tasks** To stay organized, it's essential to” | 12 | 0.295042 | none |
| G1-P3 | “In the heart of a vibrant metropolis, the sun's rays danced across the streets, casting a warm glow over the city. The air was thick with the scent” | 12 | 0.361111 | none |
| G1-P4 | “When you ride a bicycle, you're essentially moving through a series of circular motion. To maintain balance, your body and the bicycle work together to distribute the weight” | 11 | 0.302680 | none |
| G1-P5 | “Dear [Name], I hope this message finds you well. I am here to support you in your new role as a team member, and I am” | 6 | 0.266257 | none |

Frozen G0 grammaticality/prompt-responsiveness operator review: **NOT
EXERCISED**. The primary 5% machine gate had already failed, so the run took
the predeclared terminal-negative branch rather than using human review to
override it. The raw transcript is retained for diagnosis.

The matched G1-P1 disabled, coordinate-permuted, and memory-permuted controls
used the same checkpoint, prompt, support budget, and 32-token greedy decode.
They are retained in the raw report. Coordinate and memory permutation both
changed support/memory weights; neither is treated as a promotion result.

## Terminal gates and downstream state

- source-free preflight and checkpoint round trip: PASS
- bounded prefix support without single-position collapse: PASS
- load-bearing memory metric: PASS
- teacher/student metrics reported separately: PASS
- five distinct real sequences and no period-1 through period-4 cycle: PASS
- frozen 4/5 human rollout rubric: NOT EXERCISED after terminal machine failure
- at least 5% held-out real-over-coordinate-permuted advantage: **FAIL**

Decision: the one-layer checkpoint does not qualify for progressive layer
replacement. Issue #958, **G1R: redesign the layer-29 geometric
representation after support-only fitting**, is the concrete replacement
design. It is a native child of #949 and is blocked by #951 until this record
lands. Issue #952 is natively blocked by both #951 and #958; after #951 closes,
#958 becomes the next eligible design issue and #952 remains blocked. This task
does not begin #958 or #952.

Broad BDD, Gate C, kappa reproduction, fuzz, Kani, corpus-scale evaluation,
full teacher parity, and exhaustive certification were **NOT_RUN** by design.
