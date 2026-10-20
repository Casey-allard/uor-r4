# #1019 — A1Q-H5: optional local-M1 13.13M R4/Spin capacity prototype

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-31T12:19:23Z
- **Updated:** 2026-08-31T17:58:08Z
- **Closed:** 2026-08-31T17:58:08Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1019
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Current status

`OPTIONAL_PAUSED / MODEL_SUBGATES_PASS / MPS_UNAVAILABLE_HARDWARE_BUDGET / FULL_CAMPAIGN_NOT_RUN`

Parent: #973. Programme root: #820. Predecessor: #1017.

Ordinary scaled-dot-product stable-softmax attention is already established and load-bearing in the bounded learned R4/Spin path. #1017 reached 149,995,520 training tokens, exact enabled Rust replay, all six causal R4/Spin layers, and 5/5 subject-or-scene continuations. Its sealed NLL was 1.572752, narrowly missing the predeclared 1.50 threshold; that threshold does not prevent using the coherent generator.

## Evidence retained from #1019

- Exact 13,130,784-parameter, 12-layer model population: PASS.
- Fixed 400-step overfit smoke: PASS, with 81.9752% loss reduction.
- Random-export all-twelve-layer Python/Rust preflight parity: PASS, maximum absolute logit delta 0.0000443459.
- Signed MPS probe: memory PASS at 21.03%; time UNAVAILABLE under the former eight-hour contract, with a 20.66-hour safety projection.
- Full training, final qualification, reveal, generation, and replay: NOT_RUN.

The MPS terminal describes only that frozen offline trainer implementation. It is not a negative result for attention, the CPU-native UOR architecture, or the working #1017 generator.

## Final local-M1 decision

One bounded exact-shape fast-path test used 10 warmup plus 40 measured optimizer steps. The fused-AdamW/deferred-logging candidate measured 4.485223 seconds per step versus the signed 3.491307-second baseline, 28.47% slower. The fused optimizer switch was removed.

Stop #1019 tuning and do not launch its full campaign. Do not create another backend, hardware ladder, parameter sweep, or recurring test campaign from this result.

## Active owner direction

- UOR deployment remains CPU-native.
- CUDA and external GPU execution are out of scope.
- Apple Accelerate/BLAS and MPS may accelerate local offline training, compilation, and bounded developer work only; they are not deployed-runtime dependencies.
- #1019 is optional and does not block #973, inference, or use of #1017.
- Productize the working #1017 checkpoint through `r4 generate --prompt "..."` now.
- On Apple Silicon, use the opt-in `local-inference-accelerate` CPU-BLAS build.
  The actual M1 four-token comparison preserved generated IDs, text, output CID,
  and attention-audit CID while reducing generation from 3.060506042 seconds to
  0.116236875 seconds and wall time from 3.41 seconds to 0.52 seconds.
- Prototype iteration uses one targeted compile and one real behavior check. Broad qualification and release gates wait until useful behavior exists.

## Nonclaims

The current generator remains source-backed, floating-point, matmul, and ordinary-softmax based. It does not establish geometry advantage, transformerlessness, correctness, reasoning, frontier quality, browser/WASM readiness, or release readiness.

Repository record: `docs/r4_softmax_parameter_capacity_1019.md`.

References #973
References #820


## Comments (6)

### comment 5480353202 — 2026-08-31T15:12:14Z — Casey-allard

Implementation landed through protected PR #1020 at main commit 81380d3f.

Merged scope: exact 13,130,784-parameter / 16,800-step lifecycle, fresh post-#1017 population builder, signed smoke and all-12-layer Rust parity, conservative MPS hardware projection, bound MPS-first CUDA fallback, authenticated checkpoints/resume, sealed reveal, PASS-only generation/replay, and synchronized living documentation.

Focused local evidence before merge: 73 Python integrity tests, 8 Rust local-generation tests, 1 Rust CLI parser test, fmt, scoped clippy with warnings denied, claim wording, JSON/Python syntax, and an independent post-fix audit all passed. The repository PR/queue checks were transport acknowledgements; they are not being used as substantive QA claims.

Execution status remains NOT_RUN for population, accelerator smoke, hardware probe, full training, reveal, and generation. Next action is now running the merged create-once population and MPS smoke/parity, followed by the signed 200-step hardware gate. A full run starts only if that gate authorizes <=8h and <=80% memory.

### comment 5480867228 — 2026-08-31T15:53:29Z — Casey-allard

## #1019 preflight complete — MPS full run not authorized

Observed against merged main `81380d3fa574f429b9b77814a3c5a8afb78bce4f`; the sealed confirmation remains mode `000` and unopened.

- Population **PASS**: exactly `275,251,200` train IDs, `250,000` development IDs, and `249,880 + 120 = 250,000` sealed confirmation IDs; fresh post-#1017 development/test boundaries and all ten prior prompt CIDs excluded. Dataset manifest CID: `blake3:6efbffeb1b6cb20ae9bbcda03428a4b820824224c578a168cd9a65f616f3dd5c`.
- 64-sequence MPS overfit **PASS**: loss `8.366631 -> 1.508071`, reduction `81.9752% >= 80%`, exact `13,130,784` parameters. Result CID: `blake3:d1f2e3b3a2d269fbccca1ccdd2f9439392b5a06a3a48f0f90a0825029f1508ec`.
- All-12-layer Rust smoke parity **PASS**: Python/Rust top-1 both token `16`; maximum absolute logit delta `0.0000443459 <= 0.005`; causal/projection/R4/output-policy audits exact, with zero future/provider/Ollama/prior-trace reads. Decision CID: `blake3:dfe39b41eb39d9f737af003fc7fa1b21c52c1290823849b2dd3d09ce0de53bbb`.
- The signed 200-step MPS probe completed checkpoint save/reload and complete-development timing. Memory passed at `2,673,278,976 / 12,713,115,648` bytes (`21.03% <= 80%`). Time failed: raw full-run projection `59,513.15 s` (`16.53 h`), safety projection `74,391.44 s` (`20.66 h`) versus the frozen `8 h` ceiling. MPS terminal: `UNAVAILABLE_HARDWARE_BUDGET`; result CID: `blake3:8de57ebef53cda52b62baa619f87566042c59bac2a399ebf6588e53f78e4daf7`.

Decision: the model feasibility preflight passed, but `train-capacity --backend mps` is not authorized. No full-training NLL, final parity, reveal, generation, or replay was run. Established ordinary R4/Spin softmax attention is not revoked, and the 12-layer capacity hypothesis is not falsified—it remains untested at full scale.

The sole permitted continuation is the bound deterministic single-CUDA float32 fallback, using this failed-MPS result as its prerequisite. External compute and any monetary spend require explicit owner authorization before launch. If no authorized CUDA environment is supplied, record hardware unavailability rather than a model-quality negative.


### comment 5481169118 — 2026-08-31T16:18:23Z — Casey-allard

The signed preflight record and every current project guidance surface are now merged in #1021 at `be43ae98bfe2da4f62b4aa266108d18b169465fd`. Model-side population/smoke/random-export Rust preflight subgates passed; MPS remains `UNAVAILABLE_HARDWARE_BUDGET` on time, with memory passing. Full training, final parity, sealed reveal, generation, and replay remain `NOT_RUN`. This issue stays open and assigned. The only permitted next execution is the bound deterministic single-CUDA `f32` probe after explicit owner authorization for external compute and any spend.

### comment 5481517552 — 2026-08-31T16:47:54Z — Casey-allard

Owner scope correction (2026-08-31): the earlier CUDA continuation is withdrawn. UOR deployment remains CPU-native. Apple Accelerate/BLAS and MPS may be used only to speed local offline training, compilation, and bounded developer checks; CUDA and external GPU execution are out of scope. The signed MPS result remains valid but applies only to the former frozen eight-hour offline trainer contract. It does not negate attention, CPU-native feasibility, or the working #1017 local Rust generator. #1019 is now an optional capacity prototype and does not block use/productization of #1017 or #973. We will reuse the completed population/smoke/parity evidence, make only the three identified M1 speed changes, and use one short timing check rather than another recurring gate ladder. Full hardening waits until useful decoded behavior works.

### comment 5481645221 — 2026-08-31T16:59:23Z — Casey-allard

Bounded local-M1 fast-path decision (2026-08-31): STOPPED_SLOWER_THAN_SIGNED_BASELINE. One isolated exact-shape run used 10 warmup plus 40 measured optimizer steps with fused MPS AdamW, deferred loss transfer, and reduced ledger fsync. It took 179.408929 seconds, or 4.485223 seconds/step, versus the signed baseline 3.491307 seconds/step (1.284683x; 28.47% slower). The fused optimizer change was removed immediately. No population, smoke, parity, reveal, or broad suite was rerun. Per the prototype-first owner direction, there will be no M1 tweak loop and no CUDA/external path. #1019 is optional and paused; the active work is the already-working #1017 local Rust generator through `r4 generate`.

### comment 5482348073 — 2026-08-31T17:58:07Z — Casey-allard

Closed after protected PR #1022 merged as 7f5d4525. The useful outcome is retained: population, overfit, and all-layer parity passed; the frozen 13.13M MPS campaign is not worth its projected 20.66 hours, and the isolated fused-MPS candidate was slower. No CUDA fallback, backend ladder, parameter sweep, or rerun follows. The working #1017 model is now the product path, with opt-in Apple Accelerate CPU BLAS measured on this M1 at identical generated IDs/output CID/attention-audit CID and 26.33x lower recorded generation time. This closure is a scope decision, not a negative result for attention or the CPU-native architecture.


---
_Comment/review ids in this file: 5480353202, 5480867228, 5481169118, 5481517552, 5481645221, 5482348073_

