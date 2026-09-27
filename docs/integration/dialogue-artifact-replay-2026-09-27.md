# Retained dialogue artifact: offline output recovery

References #973. Fourth-lab scope under the owner's native, multiplier-free
serving direction. This work uses offline floating Rust inference to inspect
already-learned parameters; it does not widen the integer serving contract.

## Decision and retained context

The R1d dialogue study saved 21 named F32 arrays but supplied no command to
reload them for generated responses. Its width 576/read 64/context 256,
vocabulary 4096 Quaternion model is a separate response-learning vehicle from
the active width256 TinyStories reader study. The retained parameters have
SHA-256 `95e3fbb06cb39b4354bd40c722873dac088d47554707777b291e588922a1a822`
and occupy 21,719,304 bytes. The fit report has SHA-256
`a192916c17d8d124e9a32c21c3b31ef6b622890b148d37549eb219d11cd641fa`.
Both remain in
`/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/r1d/curve-2`.

The report labels the original executable's source
`5109861c93dee0e39d3aa189c325b3a13c4ededc` and executable SHA-256
`84ec4ebe5d95b28346501129394e3cb211483c9bf2b471caa8abbc4313463e4b`.
New replay source and executable identities are recorded separately. Comparing
the legacy and current ordinary Dot/Quaternion operations supports a scoped
compatibility case; a current full-window/incremental check does not reproduce
the historical backend bit for bit.

The source review follows the
[historical Dot expression](https://github.com/UOR-Foundation/uor-r4/blob/5109861c93dee0e39d3aa189c325b3a13c4ededc/crates/uor-r4-training/src/joint_model.rs#L1028)
into the [current Dot dispatch](https://github.com/UOR-Foundation/uor-r4/blob/2d3ee5f6fa1b3135a1f9bbff2f29d1d818edc4b1/crates/uor-r4-training/src/joint_model.rs#L1191).
The recurrent transport, normalization, copy and output expressions retain their
order. The later scan prototype is optional and was not selected by the
[dialogue constructor](https://github.com/UOR-Foundation/uor-r4/blob/d22e2b50da85e527efcd4f60acc1e82644194b91/crates/uor-r4-training/src/dialogue.rs#L502).

The prior negative is preserved: after 2,237 updates and 9,162,752 sampled targets,
reported response NLL 3.0678578 remained above the matched count comparator's
2.4656426. No automatic longer fit follows. The stronger R1d interpretation
that capacity, rather than exposure, is established as the cause is not
identified by that record. Its all-point fitted floor is degenerate (-50.5),
second-half and last-eight floors differ (2.511 versus 2.941), its floorless fit
disagrees, and the observed curve is still decreasing. R1c/R1d share the
model/data seed; changed batch and F32 reduction create a distinct trajectory,
not an independent-seed replication. Close training/development loss also does
not exclude objective or data issues: arbitrary 256-token response windows can
begin inside a response without its complete user request.

The fit report's `supervised_targets` field duplicates nominal sampled positions.
Summing the retained learning curve's 2,237 consecutive per-update records gives
9,162,752 sampled positions and **6,280,627 response-supervised positions**.
Keep these quantities distinct; neither counts unique corpus coverage. This is
a retrospective accounting correction, not a changed training run or criterion.

A spending rule may support declining a longer run without establishing
convergence or a universal model-size floor. The methodological distinction
between fitting an observed curve and validating its extrapolation is supported
by [Alabdulmohsin, Neyshabur and Zhai, *Revisiting Neural Scaling Laws in Language
and Vision*](https://arxiv.org/html/2209.06640). That literature is not evidence
that this particular model would improve with more training.

## Implementation boundary

`training::dialogue_artifact` imports only the retained legacy continuous
Full/Dot/Quaternion configuration. It verifies the sealed report, complete
tensor inventory, element offsets, shapes, finite values and parameter bytes
before construction. Missing legacy read/admission metadata is explicitly
interpreted as Dot/Full for this schema. The model remains private; the API
exposes immutable provenance and exact-token generation. Existing canonical
checkpoint/training and integer width validation are unchanged.

The corpus manifest binds tokenizer identity and train/development token
hashes to the fit report. Mask hashes are retrospective identity observations:
the historical report did not bind those masks. The shared tokenizer's literal
role protocol encodes separators, markers and content independently.

The separate `dialogue-artifact-replay` example consumes only the existing 38
development requests from the old panel,58assistant responses including actual
intermediate replies. It skips expected-answer and fresh fields. It uses
greedy 32-token output with the existing cycle stop, exact generated IDs,
one inserted BOS, and an explicit caller EOS before another request when the
model stopped without EOS. It never truncates the complete 256-token input plus
output horizon. Each turn replays its exact prefix in a fresh session; this is
not a persistent-session speed measurement.

This changes the old panel's presentation and output cap. It is an open
development output observation, not that panel's official pass/fail result or
a held-out qualification. Raw bytes/IDs, every prediction record, stop reasons,
prefix comparison, source/executable/artifact identities and actual work are
retained in a new exclusively claimed, sealed attempt. Interrupted attempts
remain separate.

## Necessary observation and distinct decisions

Check the loader's real format/provenance risks and exact-token interface, then
compare current full-window and incremental probabilities on one loaded
development prefix (tolerance 1e-5). Inspect every actual response. Responsive
fragments preserve an integration candidate; generic or repetitive output adds
a learned-model negative without identifying its cause. Unresolved loading is
unavailable evidence. None of these automatically starts training, integer
export, a seed/decoder sweep, or a new acceptance campaign.

The [prospective work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5853030114)
records the 90-minute complete cap, 600-second/1 GiB inference ceiling, prospective
short-probe concurrency refinement, fixed requests and shared resource limits.
The frozen Dot/Lorentz/Affine study and other labs' artifacts remain separate.

## Executed result

The [source-bound receipt](../evidence/dialogue-r1d-replay-2026-09-27.json) retains
all 58 response texts, IDs, stops, input history and complete-report identities.
Source `e3fc0a587115d34b41a0fc9edacc09745ccc0746` passed all four focused checks.
The initial attempt on `7e67e2c2` passed three and failed one before model
construction: its synthetic tokenizer lacked the required ByteLevel declaration.
Only that fixture line changed. Both attempts and their cost remain recorded.

The optimized CPU-Accelerate replay completed all 38 requests and 58 assistant
turns in 18.132 seconds, with 58,572,800 bytes maximum child RSS. There were
1,766 generated selections, 4,331 generation step calls, plus a separate 15-step
incremental and 15-position full-forward comparison. Across 61,440 compared
probabilities, the maximum absolute delta was 9.6857548e-7 and there were no
argmax disagreements. This establishes consistency of the new executable's two
paths on that prefix, not historical backend identity.

Actual output is **response-shaped but not useful dialogue**. It frequently
echoes the user's topic inside repeated email-like language, omits requested
facts, drifts and repeats. None of the eight factual replies gives the requested
fact. Some third-turn replies reuse a fact from history (Momo, teacher, blue car,
piano; the color reply also mixes blue and green), but this is not a controlled
memory-utility result or a set of reliable answers. It uses model EOS 11 times,
the existing cycle stop 4 times and the token cap 43 times. A capped continuation's
unobserved future is unknown; no extra decoding was performed to seek a better
ending.

| Request | Actual response excerpt | Stop |
|---|---|---|
| What is the capital of France? | `Yes, there are the capital of France of France, France, France,` | Cycle (period 3) |
| Write a sentence about a dog. | `I'm excited to share a dog.` followed by a generic email closing | Model EOS |
| What is my cat's name? | `Yes, I'm sure to sleep.` followed by `your cat is named Momo` inside a drifting reply | 32-token cap |
| What is my name? (after Alex was supplied) | `Yes, there are many a more concise and effective way to make it more effectively.` | 32-token cap |

The model and loader are retained as a diagnostic baseline. The observed weak
answers do not identify capacity, data, objective, exposure or read geometry as
the cause. No further fit, decoder sweep, integer width change or promotion is
justified by this observation alone. The active reader, termination and stack
studies retain their distinct decisions; use their complete results to choose
the next native-learning intervention. This closes the fit→reload→actual-output
gap for this retained artifact and leaves practical chat open.

Build/check work totaled 1,067.794 seconds including the failed fixture attempt;
the corrected cached checks and executable build took 173.724 seconds. Unused
design time was prospectively reallocated to the build envelope while keeping
the 90-minute phase and cumulative ceilings unchanged. Preparation, review,
coordination and delivery elapsed is charged once through the shared ledger;
the receipt distinguishes those costs from actual model inference.
