# #973 bounded sparse quaternion-cube development fit

**Date:** 2026-09-04
**Status:** `RESOURCE_UNAVAILABLE_FULL_CONTEXT_CUBE_FIT`

This task-level status classifies the two observed raw
`RESOURCE_UNAVAILABLE_PROJECTION_STOP` outcomes below. It is a resource
admission result, not a model result.

## Decision attempted

The task kept `R4H4FrameQuaternionCubeResidualV1`, the sparse recurrent reader,
the 120-token context, and the accepted ordinary artifact fixed. The candidate
was to receive 128 AdamW updates over 2,048 open `train.u16` windows while the
accepted sparse-plus-dense-SwiGLU artifact remained an immutable comparator.
The final validation store, held-out inputs, teacher, source model, instruction
data, width, selector, and nonlinear law were outside the task.

The cube forward reaches 18 learned tensors containing 215,296 values. Its six
retained dense-SwiGLU tensors contain 36,864 values and are unreachable from the
candidate prediction, so the implementation now freezes them and omits them
from the optimizer while retaining all 252,160 values in the artifact format.
The successful-completion path checks that the six tensors remain
byte-identical; that check was not reached by either stopped launch.

The fixed execution contract was:

- CPU f32 with four threads and deterministic Torch algorithms;
- 128 updates, batch size 16, 245,760 causal targets, seed-9738 order;
- the existing AdamW schedule, clipping, and weight decay;
- an 840-second whole-process wall per launch, 4-GiB peak-RSS ceiling, and
  8-MiB output ceiling;
- a full 120-token backward gate before optimizer update one;
- candidate/comparator seen-data pre-scoring before the dose, followed by
  post-fit scoring, exact artifact reload, and the two already-open prompt
  comparisons only after the complete dose;
- zero automatic retries and at most one direct resource correction.

## Observed result

The first launch used a 256-window seen-data monitor. Its full-context backward
gate completed, all 18 active tensors passed the finite/nonzero gradient check,
and optimizer update one reported loss `10.436132`, gradient norm `6.284435`,
elapsed `78.177` seconds, and process peak RSS `1,641,512,960` bytes. The
runner's conservative update-8 projection rule did not admit the remaining work
plus the declared margin within the 840-second wall. No artifact or decision
JSON was written.

The one policy-permitted correction reduced repeated monitor scoring to 64
windows and its post-fit reserve from 180 to 90 seconds. It did not change the
model, training windows, 128-update dose, optimizer, thresholds, or hard wall.
Setup fell materially: update one reported the same loss and gradient norm to
six decimals at `25.757` seconds with observed peak RSS `1,663,418,368` bytes.
The run again reached update 8, and the same conservative admission rule again
rejected continuation. No artifact or decision JSON was written. Neither stop
record captured the exact update-8 projection inputs; the runner now emits them
before any future projection stop.

Both synthesized stop records are preserved outside Git under
`.uor-models/research/issue-973-quaternion-cube-r4-v1/bounded-fit/`. The prior
no-fit comparison and both of its correction directories are unchanged. The
final validation and held-out stores were not opened.

- first-stop SHA-256: `f40b9119e70ad98263f8fd935171d72fc29d1252efbad2919d592f32a7ed0309`
- corrected-stop SHA-256: `83720b59084fd6e224343b63e59f7a343cadc2ec11459f52b39295edc8476edd`

## Evidence boundary

**Mathematical proof:** no new proof was attempted. The earlier real-quaternion
properties of the fixed cube law remain the mathematical result. Reaching
backward does not prove convergence or language utility.

**Measured behavior:** a finite backward completed in each launch for an exact
120-token forward that exercised recurrent eviction, summary folding, sparse
admission, both cube layers, and the tied vocabulary loss. All 18 aggregate
active tensor gradients were finite and nonzero. The two launches reported the
same six-decimal update-one loss and gradient norm. Elapsed time to update one
was 52.420 seconds lower after the resource correction, while the corrected
128-update completion rule still rejected continuation under the hard wall.
Memory did not cause either stop at the observed checkpoints.

**Unverified hypothesis:** a lean training forward that avoids unused
per-token attention-weight materialization and precomputes the deterministic
metadata-only sparse selections may make the same fixed dose fit the wall. No
such optimized forward was implemented or timed in this task.

No fitted artifact exists. The 128-update train-loss decision, seen-data
NLL/top-1, prompt continuation, dense competitiveness, H4 advantage,
generalization, instruction following, reasoning, coding, table lowering, and
release behavior remain `NOT_RUN`. Each process produced an in-memory
eight-update partial trajectory, but only update one's six-decimal progress was
preserved. The result is not evidence that the cube architecture cannot learn.
It says only that the declared update-8 completion projection did not admit
continuing this implementation toward the 128-update decision inside the
bounded wall; actual 128-update runtime remains unmeasured.

## Closure and next action

This bounded execution task is complete at
`RESOURCE_UNAVAILABLE_FULL_CONTEXT_CUBE_FIT`. Issue #973 remains open and
assigned because the scale/data stage still lacks a fitted assembled model.

The next implementation action is to add a lean differentiable training forward
for the same model that omits unused attention-weight outputs and accepts
precomputed metadata-only sparse selections, while preserving full recurrent
computation and the inference path. Its sole admission condition is that
the unchanged update-8 completion projection fits the 840-second wall before
the 128-update decision is launched again.
